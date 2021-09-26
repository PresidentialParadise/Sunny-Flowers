use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError, Reason},
    model::channel::Message,
};

use crate::utils::check_msg;

async fn handle_reason(ctx: &Context, msg: &Message, at: &str, r: &Reason) {
    match r {
        Reason::User(user) => check_msg(msg.reply(&ctx.http, user).await),
        Reason::Log(log) => eprintln!("{}", log),
        Reason::UserAndLog { user, log } => {
            check_msg(msg.reply(&ctx.http, user).await);
            eprintln!("{}", log);
        }
        _ => println!("Unknown reason in {}: {:?}", at, r),
    }
}

#[hook]
pub async fn dispatch_error_hook(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::CheckFailed(check, reason) => handle_reason(ctx, msg, check, &reason).await,
        _ => println!("Unknown dispatch error: {:?}", &error),
    }
}

#[hook]
pub async fn after_hook(
    ctx: &Context,
    msg: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>,
) {
    //  Print out an error if it happened
    if let Err(why) = error {
        if let Some(reason) = why.downcast_ref::<Reason>() {
            handle_reason(ctx, msg, cmd_name, reason).await;
        } else {
            println!("Unknown Error in {}: {:?}", cmd_name, why);
        }
    }
}
