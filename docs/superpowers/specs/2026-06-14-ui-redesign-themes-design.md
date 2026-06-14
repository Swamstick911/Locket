# UI Redesign + Theme Switcher — Design

**Date:** 2026-06-14
**Status:** Approved (design), pending spec review
**Scope:** `firmware/src/display.rs` (heavy), `firmware/src/main.rs` (render call sites, Settings, menu), `firmware/src/storage.rs` (persist theme). The `crates/core` logic crate and its 47 tests are **untouched**.

## Goal

Redesign the on-device UI to be attractive and practical, and add a user-selectable **theme**. Three themes ship: **Phosphor Terminal** (default), **Modern Dark**, **Game Boy**. The user picks under Settings; the choice persists across power-offs. The visual structure (header bar / divider / content / footer hint bar) is shared by every screen; switching theme swaps ~8 colors and repaints.

## Constraints (unchanged hardware reality)

- 160×128 ST7735, `Rgb565`, `embedded-graphics`, `no_std`, allocation-free (heapless buffers).
- Drawing primitives only: filled/stroked rectangles, lines, pixels, monospace bitmap text.
- Streaming repaints must stay partial/in-place (no full-screen clear per chunk) so the WiFi/TCP task isn't starved — preserve the existing `response_stream` approach.
- RGB565 caveats: prefer flat fills over gradients (banding); off-black/off-white over pure #000/#FFF; verify panel byte-order/inversion before final color tuning.

## 1. Theme system

A small value type carrying the palette, defined by **function** (so light themes like Game Boy map cleanly):

```rust
#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Rgb565,        // page background
    pub surface: Rgb565,   // header/footer bar + selection-bar fill
    pub text: Rgb565,      // primary text
    pub text_inv: Rgb565,  // text drawn ON surface/selection (inverse)
    pub dim: Rgb565,       // secondary/structural text
    pub divider: Rgb565,   // 1px separators, slider tracks, inactive icons
    pub accent: Rgb565,    // focus rail, highlights, active icon, caret
    pub warn: Rgb565,      // errors / alerts
    pub name: &'static str,
}
```

Three `const` themes (authoring hex → `Rgb565::new(r>>3, g>>2, b>>3)`; values may be tuned on the physical panel):

| Role | Phosphor (default) | Modern Dark | Game Boy |
|------|--------------------|-------------|----------|
| bg | `#0A140A` | `#121212` | `#9BBC0F` |
| surface | `#0F3D1E` | `#242424` | `#306230` |
| text | `#33FF66` | `#E6E6E6` | `#0F380F` |
| text_inv | `#0A140A` | `#121212` | `#9BBC0F` |
| dim | `#1F8A3A` | `#9AA0A6` | `#306230` |
| divider | `#0F3D1E` | `#2E2E2E` | `#0F380F` |
| accent | `#AFFFC0` | `#4DD0E1` | `#0F380F` |
| warn | `#FFB000` | `#FF6B6B` | `#C2603A` |

The active theme is passed by reference (`&Theme`) into the `Ui` drawing methods. No global mutable state (no_std friendly). `main` owns the current `Theme` and threads it through every `Ui::*` call. The old module-level `BG/FG/ACCENT/DIM/WARN` constants are removed.

## 2. Shared layout shell

Two new helpers used by every full-screen view, plus icon helpers:

- `header_bar(target, theme, title, icons)` — fills the top band (`STATUS_TOP..DRAFT_TOP-1`) with `surface`, draws `title` (left, `text_inv`) and right-aligned status icons, then a 1px `divider` line under it.
- `footer_hints(target, theme, text)` — a bottom band with a 1px `divider` above it and `dim` button-legend text (e.g. `‹ HOLD L=ACTIONS · L=SPACE ›`). This eats ~14px at the bottom; content area shrinks accordingly.
- `wifi_bars(target, theme, x, y, strength)` — 3–4 ascending vertical bars; filled bars in `text`, empty in `divider`.
- `persona_glyph(target, theme, x, y, persona)` — a 1-letter/bracket badge in `accent`.

Updated band layout (the footer is new):

```
y=0    ┌ HEADER BAR (surface) ~14px ┐  title + wifi/persona icons
y=13   ├ 1px divider ───────────────┤
       │ CONTENT AREA               │  list / chat / compose
y=113  ├ 1px divider ───────────────┤
y=114  └ FOOTER HINT BAR ~14px ─────┘  button legends (dim)
```

## 3. Per-screen components

**Chat/reply view** (`response`, `response_stream`, `response_scrolled`)
- Header shows `CHAT` + wifi + persona glyph; footer shows reply controls (`‹ ▲▼ SCROLL · D=TYPE-PC · L=BACK ›`).
- Each turn prefixed by a speaker glyph: user `›` in `dim`, AI persona glyph in `accent`; reply body in `text`.
- A blinking block caret (`accent`) at the write position **while streaming**; replaced by the token-count line when done. Streaming stays in-place (no full clear) — the caret and tail repaint only their dirty region.
- Keep word-wrap; preserve scroll + scrollbar (restyled to theme colors).

**Settings menu** (`menu`)
- Header `SETTINGS` + footer `‹ A/D = ADJUST · OK = SELECT ›`.
- Selected row: full-width `surface`/`accent` bar with `text_inv` text **and** a 2px `accent` left focus rail.
- New **Theme** row showing the theme name + a small color swatch (filled rect in that theme's `text`/`accent`).
- Sliders (brightness/volume) restyled: `divider` track, `accent` fill.
- Row order (recomputed constants): `Model(0) Persona(1) Theme(2) MaxTokens(3) Brightness(4) Volume(5) NewConversation(6)` then quick prompts, games, `Back`.

**Compose / typing view** (`render` → `draw_status`/`draw_draft`/`draw_guide`)
- Header `COMPOSE` + wifi + persona; footer shows the active layer's hints.
- Draft rendered inside a framed input field (1px `divider` border); typed text in `text`, ghost completion in `dim`, blinking caret in `accent`.
- Predictions shown as up to 3 **chips** (bordered rects), focused chip filled `accent`/`text_inv` — replaces today's single cramped suggestion line.
- Letter-cross / action-layer guides retained, recolored to theme.

## 4. Typography

Add `embedded_graphics::mono_font::ascii::FONT_10X20` for screen **titles only** (header bar text on home/primary screens) and possibly the token/score "hero" numbers. Body stays `FONT_6X10`. One extra font only; cost is flash, no asset pipeline. Hierarchy otherwise via color brightness, UPPERCASE labels, and whitespace.

## 5. Blinking caret mechanism

The main loop currently repaints only on input change. Add a `blink` phase counter incremented each loop tick (`TICK_MS`); toggle caret visibility ~every 500ms. Only the caret cell (compose input) / tail (streaming) is repainted on a blink tick — not the whole screen — so it's cheap and doesn't starve WiFi. If a blink-only repaint proves awkward mid-stream, fall back to a steady caret while streaming + an animated header spinner (`| / - \`).

## 6. Persistence (storage v3)

`Persisted` gains `theme: u8`. `STORAGE_VERSION` 2 → **3**. `load` accepts versions 1, 2, 3 and migrates:
- v1 payload (model, persona, tokens, count): default brightness=10, volume=5, **theme=0**.
- v2 payload (… brightness, volume, count): default **theme=0**.
- v3 payload: read `theme` byte.
`save` writes the theme byte. Default theme index 0 = Phosphor. `Settings` gains `theme: usize`; `Settings::theme() -> &'static Theme` returns `&THEMES[theme]`.

## 7. Testing / acceptance

1. `cargo build --release` green for `thumbv6m-none-eabi`, no warnings; `cargo test -p sprig-llm-core` still 47/47.
2. On-device: each of the 3 themes renders correctly on chat, settings, and compose screens; colors legible; no inversion/byte-order surprises (tune hex if needed).
3. Streaming a reply stays smooth (no flicker, no `Transport` drop) — confirms in-place repaint preserved.
4. Change theme → it persists across a power cycle; an existing (v2) flash save still loads with theme defaulting to Phosphor.
5. Brightness/volume sliders still function; USB type-to-PC unaffected.

## 8. Out of scope

Typing logic, networking, audio behavior, game prompts, and the core crate are unchanged — this is visual/structural only. No new external dependencies.

## 9. Risks

- **Signature churn:** threading `&Theme` through many `Ui` methods touches lots of call sites — mechanical but broad. Mitigate by doing it in one pass and leaning on the compiler.
- **Flash size:** one extra font + 3 theme tables; expected small, will confirm the image still fits 2 MB.
- **Color tuning:** RGB565 + cheap panel may need on-device hex tweaks (esp. grays / Game Boy greens).
- **Footer band** reduces content height by ~14px; verify chat/menu still show enough rows (≈8 content rows after header+footer).
