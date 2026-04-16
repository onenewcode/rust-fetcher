use common::constants::{DEFAULT_USER_AGENT as COMMON_UA, DOUYIN_HOST as COMMON_HOST};

pub const USER_AGENT: &str = COMMON_UA;
pub const HOST: &str = COMMON_HOST;
pub const LIVE_URL: &str = "https://live.douyin.com/";
pub const DEFAULT_WSS_BASE: &str = "wss://webcast100-ws-web-lf.douyin.com/webcast/im/push/v2/";
pub const WEBCAST_VERSION_CODE: &str = "180800";
pub const WEBCAST_SDK_VERSION: &str = "1.0.15";
pub const WEBCAST_UPDATE_VERSION_CODE: &str = "1.0.15";
pub const HEARTBEAT_INTERVAL_SECS: u64 = 5;
pub const FIRST_LIVE_FRAME_TIMEOUT_SECS: u64 = 15;
pub const MS_TOKEN_LENGTH: usize = 182;
pub const DEFAULT_USER_UNIQUE_ID: &str = "7319483754668557238";
pub const RECOMMENDED_LOGIN_COOKIE_KEYS: &[&str] = &[
    "sessionid",
    "sessionid_ss",
    "sid_tt",
    "sid_guard",
    "uid_tt",
    "passport_csrf_token",
    "odin_tt",
];
