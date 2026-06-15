# Locket

A whole AI chat terminal that fits in your pocket. It runs on a Hack Club
[Sprig](https://sprig.hackclub.com) — which is really a Raspberry Pi Pico WH in
a little handheld shell. No phone, no computer: you type on the 8 buttons, it
talks to an AI over WiFi, and the answer streams onto the 160×128 screen.

The name comes from the first thing it ever wrote. I asked it for a poem about a
tiny pocket computer and it came back with *"a library shrunken to a locket of
light."* That stuck.

It's all software. A stock Sprig plays games off an SD card; Locket throws that
out and runs its own firmware (written in Rust). Nothing is soldered on and no
parts are added — it's the same board you'd buy, just reprogrammed.

## What it can do

You hold a conversation with an AI and it remembers the thread, not just the
last message. You can switch the AI provider right on the device — **OpenRouter**
if you have a key, or **Hack Club AI** which is free for Hack Clubbers — and pick
the model from there too.

Typing is the fun part. With only 8 buttons you'd expect it to be miserable, but
it predicts words as you go, so most of the time you tap a few letters and grab
the whole word. There are three colour themes to flip between (a green
phosphor-terminal look, a clean dark one, and a Game Boy palette), and it
remembers which one you like.

A few other things it picked up along the way:

- AI mini-games — a text adventure, 20 Questions, and trivia, with the model
  running the show.
- It can type the reply straight into your laptop over USB, like a tiny keyboard.
- Little clicks and a chime through the speaker, plus brightness and volume
  sliders.
- Your settings and your last conversation are saved to flash, so they're still
  there after you unplug it.

## Typing on eight buttons

The buttons are two clusters: a left D-pad (`W` `A` `S` `D`) and a right one
(`I` `J` `K` `L`). A letter is two taps — first you tap the group the letter
lives in, then you tap which one. The screen shows you the groups, so there's
nothing to memorise.

While you're on the "pick a letter" screen, the right-hand buttons do something
else: they grab whole predicted words. So you can finish a word with the left
pad or jump straight to a suggestion with the right pad. `L` is your space bar,
and when there's a word ready to complete it shows an `L` badge — tap `L` and it
fills it in.

Everything else hides behind a hold: press and hold `L` and the bottom of the
screen turns into an action menu (send, expand-with-AI, backspace, caps,
newline, clear, settings).

## Build your own

Your WiFi details and API key get compiled into the firmware, so you build your
own copy rather than downloading a prebuilt one. First time through it's about
fifteen minutes, most of which is the computer compiling while you get a drink.

**1. Get the tools** (once). Install Rust from <https://rustup.rs>, then add the
chip Locket uses and the tool that makes flashable files:

```sh
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

**2. Grab the code:**

```sh
git clone https://github.com/Swamstick911/Locket.git
cd Locket
```

**3. Add your details.** Copy the example config (the real one is gitignored, so
your keys never get pushed) and open it up:

```sh
cp firmware/src/config.example.rs firmware/src/config.rs
```

Put in your WiFi name and password — it has to be a **2.4 GHz** network, the
Pico can't see 5 GHz. Then fill in at least one provider in the `PROVIDERS` list:

- **OpenRouter** — make an account at <https://openrouter.ai>, create a key
  (`sk-or-…`) at <https://openrouter.ai/keys>, and put a model id from
  <https://openrouter.ai/models> in its `models`. You'll want a little credit on
  the account or it'll politely refuse.
- **Hack Club AI** — free if you're in the Hack Club Slack. Sign in at
  <https://ai.hackclub.com>, grab your key and one of the model ids it lists, and
  drop them in. (Leave the key as `""` if it doesn't ask for one.)

You can leave both in and switch between them on the device later.

**4. Build it:**

```sh
cd firmware
cargo build --release
```

The first build pulls in a lot and takes a few minutes. After that it's quick.

**5. Flash it.** Hold the **BOOTSEL** button on the board and, still holding,
plug it into USB — a drive called `RPI-RP2` shows up, and you can let go. Make
the `.uf2` and drop it on that drive:

```sh
elf2uf2-rs target/thumbv6m-none-eabi/release/sprig-llm-firmware sprig.uf2
```

Copy `sprig.uf2` onto `RPI-RP2`. It reboots itself and you're in. To update it
later, just do step 5 again.

After that it runs on its own — feed it a USB wall plug or a power bank, no
computer required. (Plugging into a laptop works too, and that's how the
type-the-reply-to-my-PC trick works.)

If you just want to poke at the logic without any hardware, the keyboard and
parsing code is host-testable:

```sh
cargo test -p sprig-llm-core
```

## Using it

- **Type a letter:** tap the group, then the letter.
- **Grab a suggested word:** on the letter screen, use the right pad (`I`/`J`/`K`).
- **Space / accept the highlighted word:** `L`.
- **Action menu:** hold `L`, then `A` send · `S` expand-with-AI · `W` backspace ·
  `D` caps · `J` newline · `K` clear · `I` settings · `L` cancel.
- **In Settings:** `W`/`I` and `S`/`K` move up and down, `D` changes a value
  (next theme, next provider, brighter…), `A` goes the other way.
- **Reading a reply:** `W`/`I` and `S`/`K` scroll, `D` types it into your PC, any
  other button takes you back to the keyboard.

## How it's built

It's Rust, `no_std`, on top of [Embassy](https://embassy.dev) for the async
runtime and the RP2040 drivers. WiFi is the `cyw43` driver over PIO-SPI;
networking is `embassy-net` with DHCP. Chats stream over TLS 1.3 with
`reqwless` + `embedded-tls`, talking the OpenAI chat-completions protocol (which
is why both OpenRouter and Hack Club AI work — they speak the same thing). The
screen is driven by `st7735-lcd` + `embedded-graphics`, audio is bit-banged I²S
out of a PIO program, and settings live in the top sector of flash.

The interesting bit is split out into `crates/core`: the keyboard state machine,
the word prediction, and a tiny hand-rolled JSON/SSE parser that does no
allocation (there's no heap). That crate runs and is tested on a normal computer,
which is the only way I stayed sane debugging the typing logic.

```
crates/core/         the no_std brains: keyboard, prediction, JSON/SSE parsing
firmware/            the Embassy firmware that runs on the Pico WH
tools/dict-builder/  turns a word-frequency list into the on-device dictionary
docs/                design notes
```

## Honest caveats

- The TLS connection is encrypted but **doesn't verify the server's
  certificate** — the version of `reqwless` I'm on has no hook for it. On your
  own home WiFi that's fine; I wouldn't trust it on a sketchy network. The API
  key also lives in the firmware image. Both get better once the embedded-TLS
  crates I depend on grow up a bit.
- It's tuned for short back-and-forth, not essays. RAM is tight, so a very long
  conversation quietly forgets its oldest messages.
- While a reply streams in, the screen can dip slightly on weak power — it's the
  WiFi radio pulling current. A decent power bank or wall plug sorts it out.

## License

MIT. Build one, remix it, do whatever you like.
