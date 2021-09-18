mod discord;

use crate::discord::create_bot;

#[tokio::main]
async fn main() {
    create_bot().await;
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("Received Ctrl-C, shutting down."),
        Err(e) => {
            eprintln!("Unable to listen for shutdown signlar {}", e);
        }
    }
}
