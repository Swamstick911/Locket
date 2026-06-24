# Locket
Locket is a firmware for [Sprig](https://sprig.hackclub.com) that can make it run an AI agent into it! Written in Rust, yeah a new language for me because I wanted to learn it in a fun way!

### Hmm.. Why the name locket??
This name actually has a great story behind it, when I was making this project and was in its developmental stage, to make sure the AI in it worked, I asked it to write a poem about a small device and it's first line was *A library shrunken to a locket of light* and that actually made me go like *Woaaahhhh!!* and thought to name it **Locket** after this specific line.

### Can I see it working?
Sure! You can see it in the [website](https://locket-azure.vercel.app)!

The video:


https://github.com/user-attachments/assets/8e1b1358-631c-4c56-8978-ff09020e38d2



### Woah! What all things it can do?
Welp, it can do a LOT of thingssssss... *Like?* Like..
- You can (obviously) chat to an AI model
- Change the AI provider between [OpenRouter](https://openrouter.ai/) and [Hack Club AI](https://ai.hackclub.com/) (this is for only teens though..)
- Change the Theme between Phosphor (dark-green terminal type) / Modern Dark (like all terminals are) / Gameboy (You know how old gameboys were)
- Play some games - Adventure, Trivia, etc.
- It can type the response from the Sprig directly to your PC via USB
- Typing and response completion SFX
- Settings are saved in flash so even if you restart your settings will be there

### Hmm.. That's great, but wait, how do we type???
I was waiting for this one! Typing on this is one of the.. *couldn't find the word..* the.. most advanced (my vocabulary finished after this) typing methods I would say. Because it is a **twin-pad two-stage keyboard**.
*What does that mean??*
It means that you have to do two taps per letter, first tap - the group for the letter you have to write, second tap - the letter in the group. The groups with the letters are shown in the screen.
*Oohh.. that's kinda coool!*

### How do I get this thing on my Sprig?
Your Wi-Fi details and API key go into the firmware, so you have to build the copy of firmware yourself with the code instead of a prebuilt one (yeah, that's the case for now). Making for the first time is about 5-10 minutes setup. Most of which is just computer working while you go and touch some grass.. (please do it if you're all day on your PC like me)

1. Get the tools (one time thing). Install Rust from [here](https://rustup.rs,), then add the chip that Sprig uses and the tool that makes flashable files:
```bash
rustup target add thumbc6m-none-eabi
cargo install elf2uf2-rs
```

2. Get the Code files:
```bash
git clone https://github.com/Swamstick911/Locket.git
cd Locket
```

3. Add your credentials. Copy the example config file, if you can't find it, `config.example.rs` is the name. You can do it manually or use terminal to do it using this:
```bash
cp firmware/src/config.example.rs firmware/src/config.rs
```

Put in all the credentials, WiFi SSID, Password and API keys (don't worry I won't be able to hack you or see your credentials).
Just remember, for the WiFi it should be a **2.4GHz** and not 5GHz because the Pico can only handle 2.4GHz.
Also fill atleast something in the `PROVIDERS` list (either OpenRouter or HCAI or both) else it obviously won't work.

4. Build the firmware (obviously not hardware)
```bash
cd firmware
cargo build --release
```
This first build might take a lot of time so you just go and touch some grass (again)

5. Flash the firmware into Sprig. (FINALLY FINISHING)
Hold the BOOTSEL button on the back of the Sprig on the Pico board and while holding it, plug it into your laptop using the USB cable it comes with (recommended) or use any other if you want just don't blame me if that doesn't work.
A `RPI-RP2` drive shows up, and now put it on your desk **GENTLY**. Then make the `.uf2` and drop it in (the Sprig and not your brain pls it won't work else)
```bash
elf2uf2-rs target/thumbv6m-none-eabi/release/locket-firmware locket.uf2
```

Copy that `sprig.uf2` and paste it onto your `RPI-RP2` drive. That's it!
Don't be scared that your sprig is bricked after this, because it is doing something called REBOOTING to make the firmware work

After this nothing's left so you can now talk to an AI bot there in your sprig
I'd prefer use a powerbank or a wall charger for the power supply or plug it in your laptop.
If you've plugged it into your laptop, you can transfer the response that it gets into the text field that is active on your laptop 1:1

If you just want to see the logic without bricking your firmware, you can use:
```bash
cargo test -p sprig-llm-core
```

### How do I use it??
I said that earlier but here's a proper way

- **Type a letter**: tap the group, then the letter.
- **Grab a suggested word**: on the letter screen, use the right pad (I/J/K).
- **Space / accept the highlighted word**: L.
- **Action menu**: hold L, then A send · S expand-with-AI · W backspace · D caps · J newline · K clear · I settings · L cancel.
- **In Settings**: W/I and S/K move up and down, D changes a value (next theme, next provider, brighter…), A goes the other way.
- **Reading a reply**: W/I and S/K scroll, D types it into your PC, any other button takes you back to the keyboard.

### What did you use to make it???
Now that's a good question after a long time. I used Rust, `no_std` and embassy for the async runtime and RP2040 driver. WiFi is the `cyw43` driver over PIO-SPI, networking is `embassy-net` with DHCP. Chats stream over TLS 1.3 with `reqwless` + `embedded-tls`, talking the OpenAI chat-completions protocol (which is why both OpenRouter and Hack Club AI work, they speak the same thing). The screen is driven by `st7735-lcd` + `embedded-graphics`, audio is bit-banged I2S out of a PIO program, and settings live in the top sector of flash.

The interesting part is split out into `crates/core`, the keyboard state machine, the word prediction, and a tiny hand-rolled JSON/SSE parser that does no allocation (there's no heap).

```
crates/core/         the no_std brains: keyboard, prediction, JSON/SSE parsing
firmware/            the Embassy firmware that runs on the Pico WH
tools/dict-builder/  turns a word-frequency list into the on-device dictionary
docs/                design notes
```

### Are there any limiations and warnings??
Obviously there are, everything has limitations and warnings to see before using it

- The TLS connection is encrypted but doesn't verify the server's certificate — the version of `reqwless` I'm on has no hook for it. On your own home WiFi that's fine; I wouldn't trust it on a sketchy network. The API key also lives in the firmware image. Both get better once the embedded-TLS crates I depend on grow up a bit.
- It's tuned for short back-and-forth, not essays. RAM is tight, so a very long conversation quietly forgets its oldest messages.
- While a reply streams in, the screen can dip slightly on weak power — it's the WiFi radio pulling current. A decent power bank or wall plug sorts it out.

---
That's it for the README, now go and build this and open a PR to fix, if there are any issues!
