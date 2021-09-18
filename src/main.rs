mod discord;

use crate::discord::create_bot;

#[tokio::main]
async fn main() {
    create_bot().await;
}
