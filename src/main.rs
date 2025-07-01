mod network;
mod command;
mod storage;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let storage = storage::Storage::new();
    let command_registry = Arc::new(command::CommandRegistry::new());
    network::start(storage, command_registry).await;
}
