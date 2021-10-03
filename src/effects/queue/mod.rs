//! # Queue Effects
//! These effects affect the queue in some way or another.

mod pause;
mod play;
mod remove_at;
mod resume;
mod shuffle;
mod skip;
mod stop;
mod swap;

pub use pause::pause;
pub use play::{play, EnqueueAt};
pub use remove_at::remove_at;
pub use resume::resume;
pub use shuffle::shuffle;
pub use skip::skip;
pub use stop::stop;
pub use swap::swap;
