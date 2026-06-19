// --- theme switching (mirrors the device: phosphor / modern / gameboy) ---
const THEMES = ["phosphor", "modern", "gameboy"];
const root = document.documentElement;
const themeButtons = document.querySelectorAll("[data-set-theme]");

function applyTheme(name) {
  if (!THEMES.includes(name)) name = "phosphor";
  root.setAttribute("data-theme", name);
  try { localStorage.setItem("locket-theme", name); } catch (e) {}
  themeButtons.forEach((b) =>
    b.setAttribute("aria-pressed", b.dataset.setTheme === name ? "true" : "false")
  );
}

themeButtons.forEach((b) =>
  b.addEventListener("click", () => applyTheme(b.dataset.setTheme))
);

let saved = "phosphor";
try { saved = localStorage.getItem("locket-theme") || "phosphor"; } catch (e) {}
applyTheme(saved);

// --- hero typing flourish (progressive: content is visible without JS) ---
const typed = document.getElementById("typed");
if (typed) {
  const text = "./about";
  let i = 0;
  typed.textContent = "";
  const tick = () => {
    typed.textContent = text.slice(0, i++);
    if (i <= text.length) setTimeout(tick, 70);
  };
  setTimeout(tick, 350);
}

// --- interactive terminal section ---
const termIn = document.getElementById("termIn");
const termOut = document.getElementById("termOut");
const terminalEl = document.getElementById("terminal");
if (termIn && termOut && terminalEl) {
  const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  const print = (html, cls) => {
    const d = document.createElement("div");
    d.className = "line" + (cls ? " " + cls : "");
    d.innerHTML = html;
    termOut.appendChild(d);
    terminalEl.scrollTop = terminalEl.scrollHeight;
  };
  const goto = (id, msg) => { print(msg); const el = document.getElementById(id); if (el) el.scrollIntoView({ behavior: "smooth" }); };

  const CMDS = {
    help: () => print("commands: <b>help about poem demo build use github theme whoami clear</b><br>theme &lt;phosphor|dark|gameboy&gt;"),
    about: () => print("Locket — a whole AI chat terminal that fits in your pocket. A Hack Club Sprig, reprogrammed in Rust. Try <b>demo</b> to watch it or <b>build</b> to make one."),
    poem: () => print("A library shrunken to a locket of light,<br>Circuits hum low in the denim night;<br>With thumb-sized thunder I summon the net,<br>A quiet cosmos I carry in my pocket.", "accent"),
    demo: () => goto("demo", "rolling the demo…"),
    build: () => goto("build", "here's how to build your own…"),
    use: () => goto("use", "the controls…"),
    github: () => { print("opening the repo ↗"); window.open("https://github.com/Swamstick911/Locket", "_blank", "noopener"); },
    repo: () => CMDS.github(),
    whoami: () => print("a curious human, poking at a pocket AI's website."),
    vercel: () => print("soon — this'll be live on Vercel."),
    clear: () => { termOut.innerHTML = ""; },
  };

  function run(raw) {
    const input = raw.trim();
    if (!input) return;
    print('<span class="prompt">locket@web</span>:~$ ' + esc(input));
    const [cmd, ...args] = input.split(/\s+/);
    const c = cmd.toLowerCase();
    if (c === "theme") {
      const map = { phosphor: "phosphor", dark: "modern", modern: "modern", gameboy: "gameboy", gb: "gameboy" };
      const want = map[(args[0] || "").toLowerCase()];
      if (want) { applyTheme(want); print("theme → " + want); }
      else print("usage: theme &lt;phosphor|dark|gameboy&gt;", "warn");
    } else if (CMDS[c]) {
      CMDS[c]();
    } else {
      print("command not found: " + esc(c) + " — try <b>help</b>", "warn");
    }
  }

  print("Locket web terminal — type <b>help</b> to begin.", "muted");
  termIn.addEventListener("keydown", (e) => { if (e.key === "Enter") { run(termIn.value); termIn.value = ""; } });
  terminalEl.addEventListener("click", () => termIn.focus());
}

// --- the device screen: cycle through firmware "scenes" on a loop ---
const scr = document.getElementById("scr");
if (scr) {
  const scenes = ["boot", "compose", "reply"];
  let si = 0;
  setInterval(() => {
    si = (si + 1) % scenes.length;
    scr.setAttribute("data-scene", scenes[si]);
  }, 2600);
}

// --- copy buttons on code blocks ---
document.querySelectorAll("pre.code .copy").forEach((btn) => {
  btn.addEventListener("click", async () => {
    const code = btn.parentElement.querySelector("code");
    const text = code ? code.innerText : "";
    try {
      await navigator.clipboard.writeText(text);
    } catch (e) {
      // fallback for older browsers / file:// without clipboard API
      const r = document.createRange();
      r.selectNodeContents(code);
      const sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(r);
      try { document.execCommand("copy"); } catch (_) {}
      sel.removeAllRanges();
    }
    const old = btn.textContent;
    btn.textContent = "copied";
    btn.classList.add("done");
    setTimeout(() => { btn.textContent = old; btn.classList.remove("done"); }, 1400);
  });
});
