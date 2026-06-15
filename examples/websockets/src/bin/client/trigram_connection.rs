use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message::{self, Bytes}, futures::{WebSocket}};
use gloo_utils::errors::JsError;
use std::{cell::RefCell, rc::Rc};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use websockets::trigram::Trigram;

// a connection to a trigram that lives on a server. the `trigram` signal
// updates when we get a state update from the server or we use the `flip`
// method to request a state change
#[derive(Clone)]
pub struct TrigramConnection {
    pub trigram: Signal<Option<Trigram>>,
    sender_opt: Option<Rc<RefCell<SplitSink<WebSocket, Message>>>>,
}

impl TrigramConnection {
    // given a binary message, interpret the three lowest-order bits of the
    // first byte as a trigram, and store that trigram in the given signal.
    // silently ignore empty messages and messages that aren't in binary.
    // silently disregard any data beyond the three lowest-order bits of the
    // first byte
    fn set_from_message(trigram: Signal<Option<Trigram>>, message: Message) {
        if let Bytes(data) = message {
            if let Some(&trigram_code) = data.first() {
                trigram.set(Some(Trigram::from(trigram_code)));
            }
        }
    }

    pub fn new(url: &'static str) -> (Self, Result<(), JsError>) {
        // create a local trigram state
        let trigram = create_signal(None);

        // monitor the global trigram state through a WebSocket connection
        let mut sender_opt = None;
        let connection_status = match WebSocket::open(url) {
            Ok(socket) => {
                let (sender, mut receiver) = socket.split();
                sender_opt = Some(Rc::new(RefCell::new(sender)));

                // listen for trigram state updates
                spawn_local_scoped(async move {
                    while let Some(Ok(message)) = receiver.next().await {
                        Self::set_from_message(trigram, message);
                    }
                });

                Ok(())
            },
            Err(error) => Err(error),
        };

        (Self { trigram, sender_opt }, connection_status)
    }

    // update the local state by flipping the specified line, and ask the server
    // to mirror the change. to ensure consistency in case of crossed messages,
    // we'll get a confirmation update when the server fulfills the flip request
    pub fn flip(&self, line: u8) {
        let Self { trigram, sender_opt } = self;

        // change the local state
        trigram.update(
            |tri_opt| tri_opt.map(
                |mut tri| tri.flip(line)
            )
        );

        // request a global change, if we have a server connection
        if let Some(sender) = sender_opt {
            let sender_for_async = sender.clone();
            spawn_local_scoped(async move {
                let _ = sender_for_async
                    .borrow_mut()
                    .send(Bytes(vec![line]))
                    .await;
            });
        }
    }
}
