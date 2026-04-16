mod ac;
mod websocket;

pub use self::ac::get_ac_signature;
pub use self::websocket::generate_websocket_signature;

use crate::constants::HOST;
use anyhow::{Result, bail};

pub use common::utils::{
    current_unix_millis, current_unix_seconds, generate_ms_token, generate_numeric_id,
};

/// # Errors
///
/// Returns an error if the HOST constant does not start with https://.
pub fn host_without_scheme() -> Result<&'static str> {
    let Some(stripped) = HOST.strip_prefix("https://") else {
        bail!("HOST does not start with https://");
    };
    Ok(stripped)
}
