# Hot Reload for Rust Rocket Projects
Automatically reload the website in your browser if you make changes to your website. This builds upon `cargo watch` which can be used to automatically rebuild your rust web app when you make changes on your code. 
## Usage
```bash
rocket-hr
```
e.g. with specific directories
```bash
rocket-hr -w ./src/ -w ./styles/
```
This will host the project on `127.0.0.1:8000` by default. 

### JS Script
Add this JavaScript to all pages that should hot reload:
```js
const socket = new WebSocket("ws://localhost:8000/ws");
socket.addEventListener("message", (event) => {
    socket.close()
    window.location.reload();
});
```
___Note___: only add it for development builds of your app.
## How does it work?
`rocket-hr` parses the output of the `cargo watch` command that is expected to run a project using rocket. `rocket-hr` hosts its own web sever that forwards all requests to the application run by `cargo watch`. Additionally, it provides a route for a websocket connection. The inserted js listens on that websocket connection and reloads the page upon recieving any message. If the rust program is rebuild, a message is sent over the websocket connection. 

## Future improvements
- don't parse stdout of `cargo watch` but implement own `cargo watch` functionality
    - this also widens compatibility for non rocket projects
- inject reload script automatically
- properly kill cargo watch process as it sometimes leaves a stray process behind which then block the port :3000
