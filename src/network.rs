use crate::command::{execute_command, parse_request, CommandRegistry};
use crate::storage::Storage;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub async fn start(storage: Storage, command_registry: Arc<CommandRegistry>) {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Server listening on port 6379...");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        println!("New client connected.");
        let storage = storage.clone();
        let command_registry = Arc::clone(&command_registry);
        tokio::spawn(async move {
            handle_client(stream, storage, command_registry).await;
        });
    }
}

async fn handle_client(mut stream: TcpStream, storage: Storage, command_registry: Arc<CommandRegistry>) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(size) => {
                let (command_name, args) = parse_request(&buffer[..size]);
                let response = execute_command(command_name, args, storage.clone(), command_registry.clone());
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            Err(e) => {
                eprintln!("An error occurred: {}", e);
                break;
            }
        }
    }
    println!("Client disconnected.");
}
