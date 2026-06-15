//! Build-time configuration: WiFi credentials, LLM API key, model.
//!
//! Copy this file to `config.rs` (which is GITIGNORED) and fill in real values
//! before flashing. NEVER commit real secrets — `config.rs` is excluded from git
//! by the repo-root `.gitignore` (`/firmware/src/config.rs`).
//!
//! These are plain `&str` consts baked into the firmware image. They live in
//! flash, not in source control, once you fill in `config.rs`.
//!
//! The terminal speaks the OpenAI-compatible chat-completions protocol, so it
//! works with any compatible gateway. Two presets:
//!
//! * **OpenRouter** (default) — many models through one key. Key from
//!   <https://openrouter.ai/keys>, models at <https://openrouter.ai/models>.
//!     API_HOST = "openrouter.ai"
//!     API_PATH = "/api/v1/chat/completions"
//!     API_KEY  = "sk-or-..."         MODEL = "deepseek/deepseek-chat"
//!
//! * **Hack Club AI** — free for Hack Clubbers (<https://ai.hackclub.com>).
//!   Sign in there to get your key, then:
//!     API_HOST = "ai.hackclub.com"
//!     API_PATH = "/proxy/v1/chat/completions"
//!     API_KEY  = "<your hack club ai key>"   (leave "" if no key is required)
//!     MODEL    = a model offered there (see the site, e.g. a GPT/Gemini id)

/// SSID of the 2.4 GHz WPA2 network the terminal joins.
pub const WIFI_SSID: &str = "your-ssid";

/// WPA2 passphrase for [`WIFI_SSID`].
pub const WIFI_PASSWORD: &str = "your-password";

/// API host (no scheme), e.g. "openrouter.ai" or "ai.hackclub.com".
pub const API_HOST: &str = "openrouter.ai";

/// Chat-completions path on [`API_HOST`], e.g. "/api/v1/chat/completions".
pub const API_PATH: &str = "/api/v1/chat/completions";

/// API key, sent as `Authorization: Bearer <key>`. Leave "" to send no auth
/// header (for gateways that don't require one).
pub const API_KEY: &str = "sk-or-REPLACE_ME";

/// Model id used for Send and Expand, e.g. "deepseek/deepseek-chat".
pub const MODEL: &str = "deepseek/deepseek-chat";
