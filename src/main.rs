use log::{debug, info, warn};
use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    vec,
};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use axum_reverse_proxy::ReverseProxy;
use regex::regex;
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

fn handle_process_output(line: String, tx: &broadcast::Sender<String>) {
    println!(">> {}", line);
    if regex!(r"Rocket has launched from").is_match(&line) {
        info!("Reloading application");
        if tx.receiver_count() > 0 {
            let result = tx.send("Reload".into());
            if let Err(error) = result {
                warn!("Error sending to channel {:?}", error)
            }
        }
    }
}

fn start_process(tx: broadcast::Sender<String>) {
    let mut arg_command: Vec<String> = env::args().map(|s| s.to_string()).collect();
    arg_command.remove(0); // remove path

    let mut args: Vec<String> = vec!["watch".to_owned(), "-x".to_owned(), "run".to_owned()];
    args.extend(arg_command);

    let mut proc = Command::new("cargo")
        .env("ROCKET_PORT", "3000")
        .env("ENABLE_HOT_RELOAD", "true")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process!");
    info!("Successfully started process with PID {}", proc.id());

    if let Some(stdout) = proc.stdout.take() {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => handle_process_output(line, &tx),
                Err(e) => warn!("Error reading line from process: {}", e),
            }
        }
    }
}

async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    debug!("WebSocketUpgrade!");
    ws.on_failed_upgrade(|error| warn!("Error upgrading websocket: {}", error))
        .on_upgrade(|socket| handle_socket(socket, state.tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<String>) {
    let mut rx = tx.subscribe();

    while let Ok(_) = rx.recv().await {
        let result = socket.send(Message::Text("Reload".into())).await;
        match result {
            Err(error) => warn!("Error while sending to websocket: {}", error),
            Ok(_) => debug!("Send success"),
        }
    }
}
#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(16);
    let state = AppState { tx: tx.clone() };

    let proxy: Router = ReverseProxy::new("", "http://127.0.0.1:3000").into();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
        .merge(proxy);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    info!("Serving proxy on http://127.0.0.1:8000");

    std::thread::spawn(move || start_process(tx.clone()));
    axum::serve(listener, app).await.unwrap();
}
