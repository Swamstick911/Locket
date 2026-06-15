//! Build-time configuration: WiFi credentials + the AI providers you can switch
//! between on the device.
//!
//! Copy this file to `config.rs` (which is GITIGNORED) and fill in real values
//! before flashing. NEVER commit real secrets — `config.rs` is excluded from git
//! by the repo-root `.gitignore` (`/firmware/src/config.rs`).
//!
//! The terminal speaks the OpenAI-compatible chat-completions protocol, so it
//! works with any compatible gateway. List one or more in `PROVIDERS`; you pick
//! the active one on-device in **Settings → API**. Two presets are below:
//!
//! * **OpenRouter** — many models through one key. Key from
//!   <https://openrouter.ai/keys>; model ids at <https://openrouter.ai/models>.
//! * **Hack Club AI** — free for Hack Clubbers (<https://ai.hackclub.com>). Sign
//!   in there to get your key and the available model ids.
//!
//! Each `Provider` carries its own `models` list (ids differ between gateways),
//! and `key` is sent as `Authorization: Bearer <key>` — leave it `""` for a
//! gateway that needs no key.

use crate::Provider;

/// SSID of the 2.4 GHz WPA2 network the terminal joins.
pub const WIFI_SSID: &str = "your-ssid";

/// WPA2 passphrase for [`WIFI_SSID`].
pub const WIFI_PASSWORD: &str = "your-password";

/// Selectable API providers — switch between them on-device in Settings → API.
/// The first entry is the default. Add, remove, or reorder freely.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        name: "OpenRouter",
        host: "openrouter.ai",
        path: "/api/v1/chat/completions",
        key: "sk-or-REPLACE_ME",
        models: &[
            "deepseek/deepseek-chat",
            "openai/gpt-5",
            "deepseek/deepseek-r1:free",
        ],
    },
    Provider {
        name: "Hack Club AI",
        host: "ai.hackclub.com",
        path: "/proxy/v1/chat/completions",
        // Sign in at https://ai.hackclub.com for your key (or leave "" if none).
        key: "REPLACE_ME_OR_LEAVE_EMPTY",
        // Replace with real model id(s) listed at https://ai.hackclub.com.
        models: &["REPLACE_WITH_A_HACKCLUB_MODEL_ID"],
    },
];
