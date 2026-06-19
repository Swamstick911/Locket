# Locket Website — Design

**Date:** 2026-06-19
**Status:** Approved (direction + structure), building

## Context

Locket shipped to Hack Club but the submission was rejected because a demo
**video link** isn't accepted — the reviewer said to either put a README with
the video embedded, or (if feeling fancy) a proper website with the demo + build
instructions. We're doing the website. It doubles as the project's landing page
and its install guide, and it should look hand-crafted — like the maker built it,
not a generated template.

## Direction

A single scrolling page styled as **Locket's own screen**. The **Terminal /
Phosphor** look is the default landing aesthetic (green-on-black, monospace,
scanlines, blinking cursor, a short boot sequence). A theme toggle re-skins the
whole site between the same three themes the firmware has — **Phosphor** (default),
**Modern Dark**, and **Game Boy** — so the site mirrors the device.

## Page structure (top to bottom)

1. **Hero** — a brief boot/typing sequence resolving to **Locket**, the poem
   line *"a quiet cosmos I carry in my pocket,"* and a blinking cursor. Theme
   toggle in the top-right.
2. **Demo** — the user's demo video in a small device/screen bezel. The headline
   feature of the page.
3. **What it is** — a short `man locket`-style blurb + the feature highlights.
4. **Build one** — the setup/build/flash steps as copy-paste terminal commands
   with copy buttons (content mirrors the README so they stay in sync).
5. **Use it** — the button/controls map.
6. **Footer** — repo link, MIT, the poem in full, "made by Swamstick911".

## Tech & layout

- **Vanilla HTML + CSS + a little JS.** No framework, no build step — easy to
  co-edit, trivial to host, and hand-written reads as the opposite of vibecoded.
- Lives in **`site/`** at the repo root: `site/index.html`, `site/style.css`,
  `site/app.js`, plus `site/demo.mp4` (user provides) and any assets.
- **Hosted locally for now**; Vercel later (it deploys static files straight from
  `site/` with no build command).
- **Theme system:** CSS custom properties per theme (`--bg`, `--surface`,
  `--text`, `--dim`, `--accent`, `--warn`, …) mirroring the firmware `Theme`
  struct. The toggle swaps a `data-theme` attribute on `<html>`; choice persists
  in `localStorage`. Default = phosphor.
- **Demo video:** a `<video controls poster>` pointing at `site/demo.mp4`.
- Responsive: works down to phone width (the terminal hero stays legible).

## Verification

- Open `site/index.html` in a browser via Playwright; screenshot and refine.
- Confirm all three themes render and the toggle persists across reload.
- Check phone width (~390px), that the video plays, copy buttons copy, and every
  link (repo, openrouter, ai.hackclub.com) is correct.

## Notes

- Instruction content is lifted from the current `README.md` (already accurate to
  the provider-switcher build) so the two don't drift.
- Commits follow the project style: `type(scope): Past-tense description`, no AI
  trailers, authored as Swamstick911.
