mod trigram_broadcast;

use std::io;

use axum::Router;
use axum::extract::ws::Message::{self, Binary};
use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tokio::net::TcpListener;
use trigram_broadcast::TrigramBroadcast;

async fn connect_client(
    socket_upgrade: WebSocketUpgrade,
    State(trigram_broadcast): State<TrigramBroadcast>,
) -> impl IntoResponse {
    println!("Client connected");
    socket_upgrade.on_upgrade(|socket| manage_messages(socket, trigram_broadcast))
}

async fn manage_messages(socket: WebSocket, trigram_broadcast: TrigramBroadcast) {
    let (mut client_sender, mut client_receiver) = socket.split();

    // send the new client the current state. if sending fails, bail out
    let trigram_code_init = u8::from(&trigram_broadcast);
    let status_of_send_init = client_sender
        .send(Message::binary(vec![trigram_code_init]))
        .await;
    if status_of_send_init.is_err() {
        return;
    }

    // receive state updates from the trigram broadcast and pass them on to the
    // client. if our client is the one that requested the flip, it will have
    // updated its local state already, but confirmation is crucial to ensure
    // consistency in the case of crossed messages
    let mut broadcast_receiver = trigram_broadcast.subscribe();
    let mut forward_state_updates = tokio::spawn(async move {
        while let Ok(trigram_code) = broadcast_receiver.recv().await {
            // forward the state update to the client
            let status_of_send = client_sender
                .send(Message::binary(vec![trigram_code]))
                .await;

            // if the socket sender has stopped working, end the task
            if status_of_send.is_err() {
                println!("Client dropped: sender failed");
                break;
            }
        }
    });

    // receive and fulfill flip requests from the client
    let mut receive_flip_requests = tokio::spawn(async move {
        // keep listening for messages until the socket receiver stops working
        while let Some(Ok(message)) = client_receiver.next().await {
            // silently ignore messages that aren't in binary
            if let Binary(data) = message {
                // silently ignore empty messages, and discard everything after
                // the first byte
                if let Some(&line_index) = data.first() {
                    // fulfill the flip request and broadcast a state update,
                    // which the forwarding task will pass on to each connected
                    // client. if the line index is out of range, nothing
                    // happens to the trigram, although the state update
                    // still goes out
                    trigram_broadcast.flip(line_index);
                }
            }
        }
        println!("Client dropped: receiver failed");
    });

    // if one of the tasks ends, abort the other
    tokio::select! {
        _ = &mut forward_state_updates => receive_flip_requests.abort(),
        _ = &mut receive_flip_requests => forward_state_updates.abort(),
    };
}

#[tokio::main]
pub async fn main() -> io::Result<()> {
    // create a shared state
    let trigram_broadcast = TrigramBroadcast::new();

    // build a router
    let router = Router::new()
        .route("/", get(connect_client))
        .with_state(trigram_broadcast);

    // start listening for TCP connections
    const PORT: &str = "1110";
    let listener = TcpListener::bind(format!("localhost:{PORT}")).await?;
    println!("Listening for WebSocket connections on port {PORT}");

    axum::serve(listener, router).await
}
