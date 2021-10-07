use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError},
    model::channel::Message,
};
use tracing::{event, span, Instrument, Level};

use crate::sunny_log;
use crate::utils::SunnyError;

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    let span = span!(Level::WARN, "dispatch_error_hook", ?msg, ?error);
    async move {
        match error {
            DispatchError::CheckFailed(_check, reason) => {
                sunny_log!(&SunnyError::from(reason), ctx, msg, Level::WARN);
            }
            _ => {
                event!(Level::ERROR, ?error, "unknown dispatch error");
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
                event!(Level::ERROR, %cmd_name, %why, "Unknown error");
            }
        }
    }
    .instrument(span)
    .await
}
