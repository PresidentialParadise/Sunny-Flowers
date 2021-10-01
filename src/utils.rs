use std::{error::Error, fmt};

use serenity::framework::standard::Reason;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub type SunnyResult<T> = Result<T, SunnyError>;

#[derive(Clone, Debug)]
pub enum SunnyError {
    User(String),
    Log(String),
    UserAndLog { user: String, log: String },
}

impl SunnyError {
    pub fn user(s: &str) -> Self {
        Self::User(s.to_string())
    }

    pub fn log(s: &str) -> Self {
        Self::Log(s.to_string())
    }

    pub fn user_and_log(user: &str, log: &str) -> Self {
        Self::UserAndLog {
            user: user.to_string(),
            log: log.to_string(),
        }
    }

    /// Unpacks and 'handles' the error appropiately
    pub async fn unpack(&self, ctx: &Context, msg: &Message, at: &str) {
        match self {
            Self::User(user) => msg.reply(&ctx.http, user).await.emit(),
            Self::Log(log) => eprintln!("{} errored {}", at, log),
            Self::UserAndLog { user, log } => {
                msg.reply(&ctx.http, user).await.emit();
                eprintln!("{} errored {}", at, log);
            }
        }
    }
}

impl From<Reason> for SunnyError {
    fn from(r: Reason) -> SunnyError {
        match r {
            Reason::Log(s) => SunnyError::Log(s),
            Reason::User(s) => SunnyError::User(s),
            Reason::UserAndLog { user, log } => SunnyError::UserAndLog { user, log },
            _ => SunnyError::Log("Unknown reason".to_owned()),
        }
    }
}

impl From<SunnyError> for Reason {
    fn from(s: SunnyError) -> Reason {
        match s {
            SunnyError::Log(s) => Reason::Log(s),
            SunnyError::User(s) => Reason::User(s),
            SunnyError::UserAndLog { user, log } => Reason::UserAndLog { user, log },
        }
    }
}

impl fmt::Display for SunnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User(s) => write!(f, "user: {}", s),
            Self::Log(s) => write!(f, "log: {}", s),
            Self::UserAndLog { user, log } => write!(f, "user: {}, log: {}", user, log),
        }
    }
}

impl Error for SunnyError {}

pub trait Emitable {
    fn emit(self);
}

impl<T, E> Emitable for Result<T, E>
where
    E: fmt::Display,
{
    fn emit(self) {
        if let Err(e) = self {
            eprintln!("Encountered Error: {}", e);
        }
    }
}
