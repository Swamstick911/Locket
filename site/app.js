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
