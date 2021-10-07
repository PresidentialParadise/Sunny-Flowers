use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError},
    model::channel::Message,
};
use tracing::{span, Instrument, event, Level};

use crate::utils::{SunnyError};
use crate::sunny_log;

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    let span = span!(Level::WARN, "dispatch_error_hook", ?msg, ?error);
    async move {
        match error {
            DispatchError::CheckFailed(_check, reason) => {
                sunny_log!(&SunnyError::from(reason), ctx, msg, Level::WARN);
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
                sunny_log!(reason, ctx, msg, Level::WARN);
            } else {
                event!(Level::ERROR, "Unknown error in {}: {}", cmd_name, why);
            }
        }
    }.instrument(span).await
}
