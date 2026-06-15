mod trigram;

// --- client ---

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
use client::main;

// --- server ---

#[cfg(not(feature = "client"))]
mod server;

#[cfg(not(feature = "client"))]
use server::main;
