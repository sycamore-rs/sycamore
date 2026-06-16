use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use websockets::Trigram;

// a trigram that broadcasts an update whenever its state changes. this update
// can be passed on to clients
#[derive(Clone)]
pub struct TrigramBroadcast {
    trigram: Arc<Mutex<Trigram>>,
    broadcaster: Arc<broadcast::Sender<u8>>,
}

impl TrigramBroadcast {
    pub fn new() -> Self {
        // when we open a broadcast channel, we have to choose the capacity of
        // its message backlog. as long as there's room in the backlog, messages
        // are held until they've been received by every subscriber
        let (broadcaster, _) = broadcast::channel(1024);
        Self {
            trigram: Arc::new(Mutex::new(Trigram::new())),
            broadcaster: Arc::new(broadcaster),
        }
    }

    // create a broadcast receiver that gets an update with the new trigram code
    // every time the trigram changes
    pub fn subscribe(&self) -> broadcast::Receiver<u8> {
        self.broadcaster.subscribe()
    }

    // toggle the `n`th line of the trigram between broken and unbroken, doing
    // nothing if the line index `n` is out of range. send the new trigram state
    // to all our broadcast subscribers
    pub fn flip(&self, n: u8) {
        // lock the trigram so that other threads can't mess with it while we're
        // doing the flip. the lock will be released when the guard goes out of
        // scope and gets dropped
        //
        //   Rust Atomics and Locks, by Mara Bos
        //   Chapter 1 > "Locking: Mutexes and RwLocks" > "Rust's Mutex"
        //   https://mara.nl/atomics/basics.html#rusts-mutex
        //
        let Self { trigram, broadcaster } = self;
        let mut trigram_guarded = trigram.lock().unwrap();
        trigram_guarded.flip(n);

        // log the flip
        println!("Flipped line {n}, yielding {trigram_guarded}");

        // send the new state to our subscribers. the `send` call returns a
        // status, which we discard because there's nothing useful we can do
        // with it
        let trigram_code = u8::from(*trigram_guarded);
        let _ = broadcaster.send(trigram_code);
    }
}

impl From<&TrigramBroadcast> for u8 {
    fn from(trigram_broadcast: &TrigramBroadcast) -> Self {
        let trigram_guarded = trigram_broadcast.trigram.lock().unwrap();
        u8::from(*trigram_guarded)
    }
}
