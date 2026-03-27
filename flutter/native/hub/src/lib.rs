//! Wrenflow hub crate — entry point for Rust logic.
//! Communicates with Flutter/Dart via rinf signals.

mod actors;
pub mod signals;

use actors::create_actors;
use rinf::{dart_shutdown, write_interface};
use tokio::spawn;

write_interface!();

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Spawn the actor system
    spawn(create_actors());

    // Keep running until Dart shuts down
    dart_shutdown().await;
}
