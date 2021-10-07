use std::sync::Arc;

use crate::emit;
use serenity::prelude::Mutex;
use tracing::{event, Level};

pub async fn deafen(call_m: Arc<Mutex<songbird::Call>>) {
    let mut call = call_m.lock().await;

    if call.is_deaf() {
        event!(Level::INFO, "Client already deafened");
    } else {
        let res = call.deafen(true).await;
        emit!(res, Level::INFO);
    }
}
