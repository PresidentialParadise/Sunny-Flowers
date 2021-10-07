use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError},
    model::channel::Message,
};
use tracing::{span, Instrument, event, Level};

use crate::utils::SunnyError;

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    let span = span!(Level::WARN, "dispatch_error_hook", ?msg, ?error);
    async move {
        match error {
            DispatchError::CheckFailed(check, reason) => {
                SunnyError::from(reason).unpack(ctx, msg, check).await;
            }
            _ => { 
                event!(Level::ERROR, "Unknown dispatch error: {:?}", error)
             }
        }
    }
    .instrument(span)
    .await
}

#[hook]
pub async fn after_hook(
    ctx: &Context,
    msg: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>,
) {
    let span = span!(Level::WARN, "after_hook", ?msg, ?cmd_name);

    async move {
        // Print out an error if it happened
        if let Err(why) = error {
            if let Some(reason) = why.downcast_ref::<SunnyError>() {
                reason.unpack(ctx, msg, cmd_name).await;
            } else {
                event!(Level::ERROR, "Unknown error in {}: {}", cmd_name, why);
            }
        }
    }.instrument(span).await
}
