//! The ST7735 LCD renderer + the theme system.
//!
//! The Sprig screen is a 160x128 ST7735, driven over `SPI0` with the
//! `st7735-lcd` driver (an embedded-graphics `DrawTarget` in `Rgb565`). Every
//! screen shares one structure — a header bar, a 1px divider, a content area,
//! and (on the list/reply views) a footer hint bar that always shows what the
//! buttons do:
//!
//! ```text
//! +------------------------------------------+  y=0
//! | HEADER: title              wifi  caps    |   surface bar
//! +------------------------------------------+  y=13   1px divider
//! | content (draft / reply / menu)           |
//! +------------------------------------------+  y=114  1px divider
//! | FOOTER: button legend                    |   hint bar (lists/reply)
//! +------------------------------------------+  y=128
//! ```
//!
//! Colours come from a [`Theme`] passed into every draw call, so the whole UI
//! restyles by swapping ~10 colours. Three themes ship: [`PHOSPHOR`] (default),
//! [`MODERN_DARK`], and [`GAME_BOY`].
//!
//! The renderer is allocation-free and `no_std`: all strings are built into
//! fixed `heapless` buffers.

use core::fmt::Write as _;

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyle, MonoTextStyleBuilder,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use heapless::String;
use sprig_llm_core::keyboard::Keyboard;
use sprig_llm_core::predict::Candidates;

/// Physical panel size.
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 128;

// Layout bands (top y of each region).
const STATUS_TOP: i32 = 0;
const DRAFT_TOP: i32 = 14; // content top (just below the header divider at y=13)
const GUIDE_TOP: i32 = 92; // compose-mode guidance zone

// Footer hint bar (used by the menu + reply views).
const FOOTER_H: i32 = 13;
const FOOTER_TOP: i32 = HEIGHT as i32 - FOOTER_H; // 115
const FOOTER_DIV: i32 = FOOTER_TOP - 1; // 114 — divider above the footer

const COLS: usize = (WIDTH as usize) / 6; // 26 monospace columns
const LINE_H: i32 = 11;

/// Content rows that fit on a list/reply screen (header + footer reserved).
const CONTENT_ROWS: usize = ((FOOTER_DIV - (DRAFT_TOP + 2)) / LINE_H) as usize; // 8

/// Build an `Rgb565` from 8-bit-per-channel hex (truncated to 5/6/5 bits).
const fn rgb(r: u8, g: u8, b: u8) -> Rgb565 {
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

/// A palette: ~10 colours that fully define a theme's look. Roles are named by
/// *function* so light themes (Game Boy) map as cleanly as dark ones.
#[derive(Clone, Copy)]
pub struct Theme {
    /// Page background.
    pub bg: Rgb565,
    /// Header / footer bar fill, and the selected-row highlight bar.
    pub surface: Rgb565,
    /// Text drawn on top of `surface` (header titles, selected row).
    pub on_surface: Rgb565,
    /// Primary body text on `bg`.
    pub text: Rgb565,
    /// Secondary / structural text on `bg`.
    pub dim: Rgb565,
    /// 1px dividers, slider tracks, input frames, inactive icons.
    pub divider: Rgb565,
    /// Focus rail, active highlights, caret, active icon, chip fill.
    pub accent: Rgb565,
    /// Text drawn on top of an `accent` fill (focused chip).
    pub on_accent: Rgb565,
    /// Errors / alerts.
    pub warn: Rgb565,
    /// Display name (shown in the Settings "Theme" row).
    pub name: &'static str,
}

/// Retro green-phosphor terminal (default). Near-monochrome, very legible.
pub const PHOSPHOR: Theme = Theme {
    bg: rgb(0x0A, 0x14, 0x0A),
    surface: rgb(0x0F, 0x3D, 0x1E),
    on_surface: rgb(0xAF, 0xFF, 0xC0),
    text: rgb(0x33, 0xFF, 0x66),
    dim: rgb(0x1F, 0x8A, 0x3A),
    divider: rgb(0x16, 0x52, 0x2A),
    accent: rgb(0xAF, 0xFF, 0xC0),
    on_accent: rgb(0x0A, 0x14, 0x0A),
    warn: rgb(0xFF, 0xB0, 0x00),
    name: "Phosphor",
};

/// Clean modern dark theme: off-black surfaces, off-white text, cyan accent.
pub const MODERN_DARK: Theme = Theme {
    bg: rgb(0x12, 0x12, 0x12),
    surface: rgb(0x24, 0x24, 0x24),
    on_surface: rgb(0xE6, 0xE6, 0xE6),
    text: rgb(0xE6, 0xE6, 0xE6),
    dim: rgb(0x9A, 0xA0, 0xA6),
    divider: rgb(0x3A, 0x3A, 0x3A),
    accent: rgb(0x4D, 0xD0, 0xE1),
    on_accent: rgb(0x12, 0x12, 0x12),
    warn: rgb(0xFF, 0x6B, 0x6B),
    name: "Modern Dark",
};

/// Playful Game Boy duotone (light background, dark text).
pub const GAME_BOY: Theme = Theme {
    bg: rgb(0x9B, 0xBC, 0x0F),
    surface: rgb(0x30, 0x62, 0x30),
    on_surface: rgb(0x9B, 0xBC, 0x0F),
    text: rgb(0x0F, 0x38, 0x0F),
    dim: rgb(0x30, 0x62, 0x30),
    divider: rgb(0x30, 0x62, 0x30),
    accent: rgb(0x0F, 0x38, 0x0F),
    on_accent: rgb(0x9B, 0xBC, 0x0F),
    warn: rgb(0xC2, 0x60, 0x3A),
    name: "Game Boy",
};

/// All selectable themes, in the order they cycle in Settings. Index 0 is the
/// default. Keep in sync with the persisted theme index.
pub const THEMES: &[Theme] = &[PHOSPHOR, MODERN_DARK, GAME_BOY];

/// A short, transient status message (e.g. "SENDING").
pub type Status = String<24>;

/// Renders [`Keyboard`] state to any embedded-graphics target.
pub struct Ui;

impl Ui {
    /// Clear the whole screen to the theme background.
    pub fn clear<D>(target: &mut D, theme: &Theme) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.clear(theme.bg)
    }

    // -- small primitives -------------------------------------------------

    fn text<D>(target: &mut D, s: &str, x: i32, y: i32, color: Rgb565) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let style = MonoTextStyle::new(&FONT_6X10, color);
        Text::with_baseline(s, Point::new(x, y), style, Baseline::Top).draw(target)?;
        Ok(())
    }

    /// Faux-bold (draw twice, offset 1px) for headers — gives hierarchy without
    /// shipping a second body font.
    fn text_bold<D>(target: &mut D, s: &str, x: i32, y: i32, color: Rgb565) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Self::text(target, s, x, y, color)?;
        Self::text(target, s, x + 1, y, color)?;
        Ok(())
    }

    fn fill<D>(target: &mut D, x: i32, y: i32, w: u32, h: u32, color: Rgb565) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(target)?;
        Ok(())
    }

    fn hline<D>(target: &mut D, y: i32, color: Rgb565) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Self::fill(target, 0, y, WIDTH, 1, color)
    }

    /// A small "wifi" logo (4 ascending bars) drawn in `color`, right-aligned so
    /// its rightmost bar ends at `right_x`. Purely a connectivity indicator, not
    /// a live signal meter.
    fn wifi_icon<D>(target: &mut D, right_x: i32, color: Rgb565) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let heights = [3i32, 5, 7, 9];
        let base_x = right_x - 11;
        for (i, h) in heights.iter().enumerate() {
            let x = base_x + i as i32 * 3;
            Self::fill(target, x, 11 - h, 2, *h as u32, color)?;
        }
        Ok(())
    }

    /// Header bar: surface fill, a bold title, the wifi logo, and the divider.
    fn header_bar<D>(target: &mut D, theme: &Theme, title: &str) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Self::fill(target, 0, 0, WIDTH, DRAFT_TOP as u32 - 1, theme.surface)?;
        Self::text_bold(target, title, 3, STATUS_TOP + 2, theme.on_surface)?;
        Self::wifi_icon(target, WIDTH as i32 - 3, theme.on_surface)?;
        Self::hline(target, DRAFT_TOP - 1, theme.divider)?;
        Ok(())
    }

    /// Footer hint bar: a divider then dim button-legend text.
    fn footer_hints<D>(target: &mut D, theme: &Theme, hint: &str) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Self::hline(target, FOOTER_DIV, theme.divider)?;
        Self::text(target, hint, 3, FOOTER_TOP + 2, theme.dim)?;
        Ok(())
    }

    /// A boot/connection splash: a big centred title plus a dim subtitle. Uses
    /// the larger `FONT_10X20` (its own screen, so no tight-layout risk).
    pub fn splash<D>(target: &mut D, theme: &Theme, title: &str, subtitle: &str) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.clear(theme.bg)?;
        let big = MonoTextStyle::new(&FONT_10X20, theme.accent);
        let tw = title.len() as i32 * 10;
        Text::with_baseline(title, Point::new((WIDTH as i32 - tw) / 2, 38), big, Baseline::Top)
            .draw(target)?;
        let sw = subtitle.len() as i32 * 6;
        Self::text(target, subtitle, (WIDTH as i32 - sw) / 2, 72, theme.dim)?;
        Ok(())
    }

    // -- compose / keyboard view -----------------------------------------

    /// Redraw every zone of the compose screen from the keyboard state.
    ///
    /// `title` is the active AI persona name; `status` is an optional transient
    /// banner (empty = show the default step prompt).
    pub fn render<D>(
        target: &mut D,
        theme: &Theme,
        kb: &Keyboard,
        title: &str,
        status: &str,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.clear(theme.bg)?;
        Self::draw_status(target, theme, kb, title, status)?;
        Self::draw_draft(target, theme, kb)?;
        Self::draw_guide(target, theme, kb)?;
        Ok(())
    }

    /// Compose-mode header: a state-aware step prompt plus CAPS + suggestion
    /// count, all drawn on the surface bar.
    fn draw_status<D>(
        target: &mut D,
        theme: &Theme,
        kb: &Keyboard,
        title: &str,
        status: &str,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Self::fill(target, 0, 0, WIDTH, DRAFT_TOP as u32 - 1, theme.surface)?;

        let (label, color) = if !status.is_empty() {
            (status, theme.warn)
        } else if kb.action_armed() {
            ("ACTION: pick one", theme.warn)
        } else if kb.active_group().is_some() {
            ("STEP 2: pick letter", theme.accent)
        } else {
            (title, theme.on_surface)
        };

        let mut final_label: String<32> = String::new();
        if label == title {
            let _ = write!(final_label, "AI: {}", label);
        } else {
            let _ = final_label.push_str(label);
        }
        Self::text_bold(target, &final_label, 3, STATUS_TOP + 2, color)?;

        // Right side: CAPS flag + live suggestion count, left of the wifi logo.
        let mut right: String<12> = String::new();
        if kb.caps() {
            let _ = right.push_str("CAPS ");
        }
        let _ = write!(right, "s{}", kb.candidates().len());
        let w = right.len() as i32 * 6;
        let right_edge = WIDTH as i32 - 16; // leave room for the wifi logo
        Self::text(target, &right, right_edge - w, STATUS_TOP + 2, theme.on_surface)?;
        Self::wifi_icon(target, WIDTH as i32 - 3, theme.on_surface)?;

        Self::hline(target, DRAFT_TOP - 1, theme.divider)?;
        Ok(())
    }

    /// Draft zone: a framed input field with wrapped text, dim ghost completion,
    /// and a block caret.
    fn draw_draft<D>(target: &mut D, theme: &Theme, kb: &Keyboard) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Input-field frame around the draft band.
        Rectangle::new(Point::new(1, DRAFT_TOP), Size::new(WIDTH - 2, (GUIDE_TOP - 2 - DRAFT_TOP) as u32))
            .into_styled(PrimitiveStyle::with_stroke(theme.divider, 1))
            .draw(target)?;

        let style = MonoTextStyle::new(&FONT_6X10, theme.text);
        let left = 4i32;
        let right_cols = COLS - 1; // 1 char of padding inside the frame

        let mut x = left;
        let mut y = DRAFT_TOP + 3;
        let mut col = 0usize;
        let mut buf: String<2> = String::new();

        for ch in kb.text().chars() {
            if ch == '\n' || col >= right_cols {
                x = left;
                y += LINE_H;
                col = 0;
                if ch == '\n' {
                    continue;
                }
            }
            if y > GUIDE_TOP - LINE_H {
                break;
            }
            buf.clear();
            let _ = buf.push(ch);
            Text::with_baseline(&buf, Point::new(x, y), style, Baseline::Top).draw(target)?;
            x += 6;
            col += 1;
        }

        // Inline "ghost" completion: the not-yet-typed tail of the top
        // suggestion, dimmed, right at the cursor. Pressing space (L) accepts it.
        if let Some(suffix) = kb.completion_suffix() {
            let ghost = MonoTextStyle::new(&FONT_6X10, theme.dim);
            // A thin caret marks the boundary between typed text and the ghost.
            Self::fill(target, x, y, 1, 9, theme.accent)?;
            for ch in suffix.chars() {
                if col >= right_cols {
                    x = left;
                    y += LINE_H;
                    col = 0;
                }
                if y > GUIDE_TOP - LINE_H {
                    break;
                }
                buf.clear();
                let _ = buf.push(ch);
                Text::with_baseline(&buf, Point::new(x, y), ghost, Baseline::Top).draw(target)?;
                x += 6;
                col += 1;
            }
        } else if y <= GUIDE_TOP - LINE_H {
            // No completion: a solid block cursor.
            Self::fill(target, x, y, 6, 9, theme.accent)?;
        }
        Ok(())
    }

    /// The guidance zone — makes typing self-explanatory; branches on the step.
    fn draw_guide<D>(target: &mut D, theme: &Theme, kb: &Keyboard) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        if kb.action_armed() {
            return Self::draw_action_layer(target, theme);
        }
        match kb.active_group() {
            Some(letters) => Self::draw_letter_cross(target, theme, letters, kb.candidates()),
            None => Self::draw_compose_guide(target, theme, kb),
        }
    }

    /// Step 2: a left-pad letter cross (pick the letter) and, when predictions
    /// exist, a right-hand column of whole words mapped to the right pad
    /// (`I`/`J`/`K` → candidate 1/2/3). So the left pad finishes the letter and
    /// the right pad grabs a whole word — two separate sides.
    fn draw_letter_cross<D>(
        target: &mut D,
        theme: &Theme,
        letters: &str,
        cands: &Candidates,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Left zone: the letter cross, compressed to the left half to leave room
        // for the word column.
        let cells = [
            ("W", 0usize, (42, GUIDE_TOP)),       // up
            ("A", 1, (2, GUIDE_TOP + 11)),        // left
            ("S", 2, (42, GUIDE_TOP + 22)),       // down
            ("D", 3, (74, GUIDE_TOP + 11)),       // right
        ];
        for (btn, idx, (x, y)) in cells.iter() {
            if let Some(ch) = letters.chars().nth(*idx) {
                let mut cell: String<8> = String::new();
                let _ = write!(cell, "{}:{}", btn, ch);
                Self::text_bold(target, &cell, *x, *y, theme.accent)?;
            }
        }

        // Right zone: up to three predicted words, each grabbable with its
        // right-pad key. Only drawn when predictions exist.
        if !cands.is_empty() {
            // Divider between the letter cross and the word column.
            Self::fill(target, 90, GUIDE_TOP, 1, (HEIGHT as i32 - GUIDE_TOP) as u32, theme.divider)?;
            let keys = ["I", "J", "K"];
            for (i, key) in keys.iter().enumerate() {
                if let Some(word) = cands.get(i) {
                    let y = GUIDE_TOP + i as i32 * 11;
                    // Key letter (accent) then the word (text), truncated to fit.
                    Self::text(target, key, 94, y, theme.accent)?;
                    let mut w: String<12> = String::new();
                    for ch in word.as_str().chars() {
                        if w.len() >= 9 {
                            break;
                        }
                        let _ = w.push(ch);
                    }
                    Self::text(target, &w, 104, y, theme.text)?;
                }
            }
        }
        Ok(())
    }

    /// Step 1: the word `L` will complete to (with an `L` key badge), any other
    /// candidates shown dimmed as "keep typing toward these", and the
    /// group→button map. Only the badged word is selectable — by pressing `L`;
    /// the alternatives re-rank as you type more letters.
    fn draw_compose_guide<D>(target: &mut D, theme: &Theme, kb: &Keyboard) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Line 1: only show the suggestion strip when `L` would actually accept a
        // word (i.e. the current word is incomplete). Otherwise `L` is a space.
        if let Some(word) = kb.space_completion() {
            // An "L" key badge, then the exact word L accepts (in accent).
            Self::fill(target, 2, GUIDE_TOP - 1, 11, 11, theme.accent)?;
            Self::text(target, "L", 5, GUIDE_TOP, theme.on_accent)?;
            let mut x = 16i32;
            Self::text(target, word, x, GUIDE_TOP, theme.accent)?;
            x += word.len() as i32 * 6 + 8;
            // Up to two other candidates, dimmed — reachable by typing more.
            for cand in kb.candidates().iter().filter(|c| c.as_str() != word).take(2) {
                let need = cand.len() as i32 * 6;
                if x + need > WIDTH as i32 - 2 {
                    break;
                }
                Self::text(target, cand, x, GUIDE_TOP, theme.dim)?;
                x += need + 8;
            }
        } else {
            Self::text(target, "L = space", 3, GUIDE_TOP, theme.dim)?;
        }

        // Lines 2-3: the group → button map; the L hint reflects what L does now.
        Self::text(target, "Wabcd Aefgh Sijkl Dmnop", 0, GUIDE_TOP + 11, theme.dim)?;
        let line3 = if kb.space_completion().is_some() {
            "Iqrst Juvwx Kyz. L=fill"
        } else {
            "Iqrst Juvwx Kyz., L=spc"
        };
        Self::text(target, line3, 0, GUIDE_TOP + 22, theme.dim)?;
        Ok(())
    }

    /// Action layer (after Hold L): label each button's action.
    fn draw_action_layer<D>(target: &mut D, theme: &Theme) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Self::text(target, "W=back A=send S=expand", 2, GUIDE_TOP, theme.warn)?;
        Self::text(target, "D=caps I=set J=newline", 2, GUIDE_TOP + 11, theme.warn)?;
        Self::text(target, "K=clear  L=cancel", 2, GUIDE_TOP + 22, theme.warn)?;
        Ok(())
    }

    // -- reply / response views ------------------------------------------

    /// Full-screen reply view (used at the start of streaming + for errors).
    /// Clears, draws the header, and wraps the tail of `body` to fit.
    pub fn response<D>(target: &mut D, theme: &Theme, header: &str, body: &str) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.clear(theme.bg)?;
        Self::header_bar(target, theme, header)?;

        // Tail view: keep only the last screenful of wrapped rows.
        let max_rows = ((HEIGHT as i32 - (DRAFT_TOP + 2)) / LINE_H) as usize;
        let style = MonoTextStyle::new(&FONT_6X10, theme.text);
        let mut rows: heapless::Vec<String<COLS>, 64> = heapless::Vec::new();
        wrap_into(body, &mut rows);

        let start = rows.len().saturating_sub(max_rows);
        let mut y = DRAFT_TOP + 2;
        for row in &rows[start..] {
            Text::with_baseline(row, Point::new(3, y), style, Baseline::Top).draw(target)?;
            y += LINE_H;
        }
        Ok(())
    }

    /// Streaming hot-loop repaint — never blanks the panel, so it can't flicker
    /// and won't starve the WiFi task. Overwrites the header and every visible
    /// row in place with opaque (own-background) text, padded to full width.
    pub fn response_stream<D>(
        target: &mut D,
        theme: &Theme,
        header: &str,
        body: &str,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let max_rows = ((HEIGHT as i32 - (DRAFT_TOP + 2)) / LINE_H) as usize;

        // Header band (surface fill + opaque title + wifi logo + divider).
        Self::fill(target, 0, 0, WIDTH, DRAFT_TOP as u32 - 1, theme.surface)?;
        let body_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(theme.text)
            .background_color(theme.bg)
            .build();
        let header_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(theme.on_surface)
            .background_color(theme.surface)
            .build();

        // Header padded so a shorter header overwrites the previous one. Leave
        // the right end clear for the wifi logo.
        let mut hdr: String<COLS> = String::new();
        for ch in header.chars() {
            if hdr.len() >= COLS - 3 {
                break;
            }
            let _ = hdr.push(ch);
        }
        while hdr.len() < COLS - 3 && hdr.push(' ').is_ok() {}
        Text::with_baseline(&hdr, Point::new(3, STATUS_TOP + 2), header_style, Baseline::Top)
            .draw(target)?;
        Self::wifi_icon(target, WIDTH as i32 - 3, theme.on_surface)?;
        Self::hline(target, DRAFT_TOP - 1, theme.divider)?;

        let mut rows: heapless::Vec<String<COLS>, 64> = heapless::Vec::new();
        wrap_into(body, &mut rows);

        let start = rows.len().saturating_sub(max_rows);
        let visible = &rows[start..];
        let mut y = DRAFT_TOP + 2;
        // Redraw a full grid every time, padding each row (and blanking unused
        // rows) so the body region is fully repainted in place, no clear.
        for i in 0..max_rows {
            let mut line: String<COLS> = String::new();
            if let Some(row) = visible.get(i) {
                for ch in row.chars() {
                    if line.push(ch).is_err() {
                        break;
                    }
                }
            }
            while line.len() < COLS && line.push(' ').is_ok() {}
            Text::with_baseline(&line, Point::new(3, y), body_style, Baseline::Top).draw(target)?;
            y += LINE_H;
        }
        Ok(())
    }

    /// Scrollable reply view (the read-after-streaming mode). Renders a window
    /// starting `scroll_lines` from the top, with a scrollbar and footer hints.
    pub fn response_scrolled<D>(
        target: &mut D,
        theme: &Theme,
        header: &str,
        body: &str,
        scroll_lines: usize,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.clear(theme.bg)?;
        Self::header_bar(target, theme, header)?;

        let max_rows = CONTENT_ROWS;
        let style = MonoTextStyle::new(&FONT_6X10, theme.text);
        let mut rows: heapless::Vec<String<COLS>, 256> = heapless::Vec::new();
        wrap_into(body, &mut rows);

        let max_start = rows.len().saturating_sub(max_rows);
        let start = scroll_lines.min(max_start);
        let end = (start + max_rows).min(rows.len());

        let mut y = DRAFT_TOP + 2;
        for row in &rows[start..end] {
            Text::with_baseline(row, Point::new(3, y), style, Baseline::Top).draw(target)?;
            y += LINE_H;
        }

        Self::draw_scrollbar(target, theme, start, max_rows, rows.len())?;
        Self::footer_hints(target, theme, "WS scroll  D=type  L=back")?;
        Ok(())
    }

    fn draw_scrollbar<D>(
        target: &mut D,
        theme: &Theme,
        start_row: usize,
        visible_rows: usize,
        total_rows: usize,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        if total_rows <= visible_rows {
            return Ok(());
        }
        let area_h = FOOTER_DIV - (DRAFT_TOP + 2);
        let bar_h = (area_h * visible_rows as i32 / total_rows as i32).max(4);
        let bar_y =
            DRAFT_TOP + 2 + (area_h - bar_h) * start_row as i32 / (total_rows - visible_rows) as i32;
        // Track (dim) then thumb (accent).
        Self::fill(target, WIDTH as i32 - 3, DRAFT_TOP + 2, 2, area_h as u32, theme.divider)?;
        Self::fill(target, WIDTH as i32 - 3, bar_y, 2, bar_h as u32, theme.accent)?;
        Ok(())
    }

    /// Number of wrapped rows the body produces, and how many fit on one scroll
    /// screen. The main loop uses this to clamp the scroll offset. Must match
    /// [`response_scrolled`]'s row budget.
    pub fn wrapped_row_count(body: &str) -> (usize, usize) {
        let mut rows: usize = 1;
        let mut col: usize = 0;
        for ch in body.chars() {
            if ch == '\n' || col >= COLS {
                rows += 1;
                col = 0;
                if ch == '\n' {
                    continue;
                }
            }
            col += 1;
        }
        (rows, CONTENT_ROWS)
    }

    // -- settings menu ----------------------------------------------------

    /// A scrolling settings menu. `selected` is highlighted with a surface bar +
    /// accent left rail. `sliders` rows render a `0..=10` bar; `swatches` rows
    /// render a small colour square (used by the Theme row).
    pub fn menu<D>(
        target: &mut D,
        theme: &Theme,
        title: &str,
        items: &[&str],
        selected: usize,
        sliders: &[(usize, u8)],
        swatches: &[(usize, Rgb565)],
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.clear(theme.bg)?;
        Self::header_bar(target, theme, title)?;

        let top = DRAFT_TOP + 3;
        let max_rows = CONTENT_ROWS;

        // Scroll so the selected row stays on screen (centred, clamped).
        let start = if items.len() <= max_rows {
            0
        } else {
            selected
                .saturating_sub(max_rows / 2)
                .min(items.len() - max_rows)
        };
        let end = (start + max_rows).min(items.len());

        let mut y = top;
        for i in start..end {
            let is_sel = i == selected;
            if is_sel {
                // Surface highlight bar + accent left rail.
                Self::fill(target, 0, y - 1, WIDTH, LINE_H as u32, theme.surface)?;
                Self::fill(target, 0, y - 1, 2, LINE_H as u32, theme.accent)?;
            }
            let row_color = if is_sel { theme.on_surface } else { theme.text };

            // Label, truncated to leave room for the slider / swatch / scrollbar.
            let mut row: String<COLS> = String::new();
            for ch in items[i].chars() {
                if row.len() >= COLS - 2 {
                    break;
                }
                let _ = row.push(ch);
            }
            Self::text(target, &row, 4, y, row_color)?;

            // Slider bar on the right, if this row is one. On the selected row the
            // track is drawn in `bg` (not `divider`) so it stays visible against
            // the `surface` highlight even in themes where divider≈surface.
            if let Some(&(_, val)) = sliders.iter().find(|(idx, _)| *idx == i) {
                let bar_w = 40u32;
                let bar_x = WIDTH as i32 - bar_w as i32 - 8;
                let fill_w = (bar_w * val.min(10) as u32) / 10;
                let track = if is_sel { theme.bg } else { theme.divider };
                Self::fill(target, bar_x, y + 3, bar_w, 4, track)?;
                if fill_w > 0 {
                    Self::fill(target, bar_x, y + 3, fill_w, 4, theme.accent)?;
                }
            }

            // Colour swatch on the right, if this row has one (Theme row).
            if let Some(&(_, color)) = swatches.iter().find(|(idx, _)| *idx == i) {
                let sw = 9i32;
                let sx = WIDTH as i32 - sw - 8;
                Rectangle::new(Point::new(sx, y), Size::new(sw as u32, sw as u32))
                    .into_styled(PrimitiveStyle::with_stroke(theme.dim, 1))
                    .draw(target)?;
                Self::fill(target, sx + 1, y + 1, (sw - 2) as u32, (sw - 2) as u32, color)?;
            }

            y += LINE_H;
        }

        // "more above / below" markers, then the footer hint bar.
        if start > 0 {
            Self::text(target, "^", WIDTH as i32 - 8, top, theme.dim)?;
        }
        if end < items.len() {
            Self::text(target, "v", WIDTH as i32 - 8, FOOTER_DIV - LINE_H, theme.dim)?;
        }
        Self::footer_hints(target, theme, "WS move  A/D change  L ok")?;
        Ok(())
    }
}

/// Char-wrap `body` into `rows` (honouring `\n`), keeping the most recent rows
/// when the buffer fills. Shared by the reply views.
fn wrap_into<const N: usize>(body: &str, rows: &mut heapless::Vec<String<COLS>, N>) {
    let mut cur: String<COLS> = String::new();
    for ch in body.chars() {
        if ch == '\n' || cur.len() >= COLS {
            if rows.push(cur.clone()).is_err() {
                rows.remove(0);
                let _ = rows.push(cur.clone());
            }
            cur.clear();
            if ch == '\n' {
                continue;
            }
        }
        let _ = cur.push(ch);
    }
    if rows.push(cur.clone()).is_err() {
        rows.remove(0);
        let _ = rows.push(cur);
    }
}
