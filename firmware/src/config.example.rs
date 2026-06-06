//! Build-time configuration: WiFi credentials, Anthropic API key, model.
//!
//! Copy this file to `config.rs` (which is GITIGNORED) and fill in real values
//! before flashing. NEVER commit real secrets — `config.rs` is excluded from git
//! by the repo-root `.gitignore` (`/firmware/src/config.rs`).
//!
//! These are plain `&str` consts baked into the firmware image. They live in
//! flash, not in source control, once you fill in `config.rs`.

/// SSID of the WPA2 network the terminal joins.
pub const WIFI_SSID: &str = "your-ssid";

/// WPA2 passphrase for [`WIFI_SSID`].
pub const WIFI_PASSWORD: &str = "your-password";

/// Anthropic API key, sent as the `x-api-key` header. Format: `sk-ant-...`.
pub const ANTHROPIC_API_KEY: &str = "sk-ant-REPLACE_ME";

/// Claude model id used for both Send and Expand requests.
pub const MODEL: &str = "claude-opus-4-8";
