use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError},
    model::channel::Message,
};

use crate::utils::SunnyError;

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::CheckFailed(check, reason) => {
            SunnyError::from(reason).unpack(ctx, msg, check).await;
        }
        _ => eprintln!("Unknown dispatch error: {:?}", &error),
    }
}

#[hook]
pub async fn after_hook(
    ctx: &Context,
    msg: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>,
) {
    // Print out an error if it happened
    if let Err(why) = error {
        if let Some(reason) = why.downcast_ref::<SunnyError>() {
            reason.unpack(ctx, msg, cmd_name).await;
        } else {
            eprintln!("Unknown Error in {}: {}", cmd_name, why);
        }
    }
}
