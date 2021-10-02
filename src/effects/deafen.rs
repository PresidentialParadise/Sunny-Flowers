use std::sync::Arc;

use serenity::prelude::Mutex;

use crate::utils::Emitable;

pub async fn deafen(call_m: Arc<Mutex<songbird::Call>>) {
    let mut call = call_m.lock().await;

    if call.is_deaf() {
        println!("Client already deafened");
    } else {
        call.deafen(true).await.emit();
    }
}
