use std::{error::Error, fmt};

use serenity::framework::standard::Reason;

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
}

#[macro_export]
macro_rules! sunny_log {
    ($err:expr, $ctx:expr, $msg:expr, $lvl:expr) => {
        use crate::emit;

        let error: &SunnyError = $err;
        let ctx: &Context = $ctx;
        let msg: &Message = $msg;
        match error {
            SunnyError::User(user) => emit!(msg.reply(&ctx.http, user).await, $lvl),
            SunnyError::Log(log) => event!($lvl, ?log),
            SunnyError::UserAndLog { user, log } => {
                emit!(msg.reply(&ctx.http, user).await, $lvl);
                event!($lvl, ?log);
            }
        }
    };
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

#[macro_export]
macro_rules! emit {
    ($res:expr, $lvl:expr) => {
        if let Err(e) = $res {
            event!($lvl, %e, "Emit error")
        }
    };
}
