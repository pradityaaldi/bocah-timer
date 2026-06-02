<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { onMount } from "svelte";

  const DEFAULTS = {
    message: "Time's up!",
    color: "#0b0b0f",
    opacity: 0.78,
    loop: false,
    sound: true,
  };

  // --- persisted settings ---
  let settings = $state(load());
  function load() {
    try {
      return { ...DEFAULTS, ...JSON.parse(localStorage.getItem("settings") || "{}") };
    } catch {
      return { ...DEFAULTS };
    }
  }
  $effect(() => {
    localStorage.setItem("settings", JSON.stringify(settings));
  });

  // --- timer input ---
  let minutes = $state(25);
  let seconds = $state(0);
  let label = $state(localStorage.getItem("lastLabel") || "");

  // --- live state from backend ---
  let running = $state(false);
  let remaining = $state(0); // seconds left when running
  let autostart = $state(false);
  let showSettings = $state(false);
  let shown = $state(false); // popover enter/exit animation

  const fmt = (s) => {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
  };

  onMount(async () => {
    try {
      autostart = await isEnabled();
    } catch {}
    const state = await invoke("get_state").catch(() => null);
    if (state) {
      running = state.running;
      remaining = state.remaining;
      if (state.label) label = state.label;
    }
    await listen("tick", (e) => {
      remaining = e.payload;
      running = true;
    });
    await listen("finished", () => {
      running = false;
      remaining = 0;
    });
    await listen("stopped", () => {
      running = false;
      remaining = 0;
    });

    // popover enter animation when the window gains focus (shown by the tray)
    const win = getCurrentWindow();
    if (await win.isFocused().catch(() => false)) shown = true;
    win.onFocusChanged(({ payload: focused }) => {
      if (focused) shown = true;
    });

    // exit animation, then actually hide the native window
    await listen("anim-hide", () => {
      shown = false;
      setTimeout(() => invoke("hide_window"), 170);
    });

    // Esc hides the popover window (with animation)
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        if (showSettings) {
          showSettings = false;
        } else {
          shown = false;
          setTimeout(() => invoke("hide_window"), 170);
        }
      }
    });
  });

  async function start() {
    const total = Math.max(1, minutes * 60 + seconds);
    const goal = label.trim();
    localStorage.setItem("lastLabel", goal);
    await invoke("start_timer", {
      durationSecs: total,
      label: goal,
      settings: $state.snapshot(settings),
    });
    running = true;
    remaining = total;
  }

  async function stop() {
    await invoke("stop_timer");
    running = false;
    remaining = 0;
  }

  async function toggleAutostart() {
    try {
      if (autostart) {
        await disable();
        autostart = false;
      } else {
        await enable();
        autostart = true;
      }
    } catch (e) {
      console.error("autostart", e);
    }
  }
</script>

<main>
  <div class="pop" class:shown>
  <div class="card">
  <header>
    <div class="brand">
      <span class="brand-name">TIMER</span>
      <span class="brand-by">by praditya</span>
    </div>
    <button class="icon" onclick={() => (showSettings = !showSettings)} title="Settings">
      {#if showSettings}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20z" />
        </svg>
      {:else}
        <svg class="gear" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.48.48 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.48.48 0 0 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.03-1.58zM12 15.6a3.6 3.6 0 1 1 0-7.2 3.6 3.6 0 0 1 0 7.2z" />
        </svg>
      {/if}
    </button>
  </header>

  <div class="body">
  {#if showSettings}
    <section class="settings">
      <label>
        <span>Overlay message</span>
        <input type="text" bind:value={settings.message} maxlength="80" />
      </label>
      <label class="row">
        <span>Overlay color</span>
        <input type="color" bind:value={settings.color} />
      </label>
      <label>
        <span>Opacity · {Math.round(settings.opacity * 100)}%</span>
        <input type="range" min="0.3" max="1" step="0.01" bind:value={settings.opacity} />
      </label>
      <label class="row">
        <span>Loop / repeat</span>
        <input type="checkbox" bind:checked={settings.loop} />
      </label>
      <label class="row">
        <span>Alarm sound</span>
        <input type="checkbox" bind:checked={settings.sound} />
      </label>
      <label class="row">
        <span>Start at login</span>
        <input type="checkbox" checked={autostart} onchange={toggleAutostart} />
      </label>
    </section>
  {:else}
    <section class="timer">
      <div class="hero">TIMER</div>
      {#if running}
        {#if label}<p class="goal">{label}</p>{/if}
        <div class="countdown">{fmt(remaining)}</div>
        <p class="hint">{settings.loop ? "Loop on" : "Counting down…"}</p>
        <button class="primary danger" onclick={stop}>Stop</button>
      {:else}
        <input
          class="goal-input"
          type="text"
          placeholder="Tujuan (mis. habis ini mandi)"
          bind:value={label}
          maxlength="80"
        />
        <div class="inputs">
          <label>
            <input type="number" min="0" max="999" bind:value={minutes} />
            <span>min</span>
          </label>
          <label>
            <input type="number" min="0" max="59" bind:value={seconds} />
            <span>sec</span>
          </label>
        </div>
        <button class="primary" onclick={start}>Start</button>
      {/if}
    </section>
  {/if}
  </div>
  </div>
  <svg class="notch" width="60" height="10" viewBox="0 0 60 10" aria-hidden="true">
    <path class="notch-fill" d="M0,10 C21,10 25,2 30,2 C35,2 39,10 60,10 Z" />
    <path class="notch-line" d="M0,10 C21,10 25,2 30,2 C35,2 39,10 60,10" />
  </svg>
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0;
    background: transparent;
    color: #f4f4f5;
    font-family: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, sans-serif;
    -webkit-user-select: none;
    user-select: none;
    overflow: hidden;
  }
  main {
    position: relative;
    padding: 13px 18px 24px;
  }
  .pop {
    position: relative;
    transform: scale(0);
    transform-origin: top center;
    /* exit: quick clean shrink */
    transition: transform 0.16s cubic-bezier(0.4, 0, 1, 1);
  }
  .pop.shown {
    transform: scale(1);
    /* enter: scale-up with a soft overshoot */
    transition: transform 0.28s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  /* WARP-style curved notch that points up at the tray icon */
  .notch {
    position: absolute;
    top: -9px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2;
  }
  .notch-fill {
    fill: #18191a;
  }
  .notch-line {
    fill: none;
    stroke: #52525e;
    stroke-width: 1;
  }
  .card {
    background: #000;
    border: 1px solid #52525e;
    border-radius: 10px;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 9px 16px;
    background: #18191a;
  }
  .brand {
    display: flex;
    flex-direction: column;
    line-height: 1;
  }
  .brand-name {
    font-size: 17px;
    font-weight: 800;
    letter-spacing: 0.03em;
    color: #f4f4f5;
  }
  .brand-by {
    font-size: 9px;
    font-weight: 700;
    color: #f4f4f5;
    margin-top: 0;
  }
  .body {
    padding: 20px 18px 24px;
    background: #000;
  }
  .hero {
    text-align: center;
    font-size: 36px;
    font-weight: 800;
    letter-spacing: 0.03em;
    margin: 4px 0 22px;
    color: #f24b22;
  }
  .icon {
    background: none;
    border: none;
    color: #c4c4cc;
    cursor: pointer;
    padding: 2px 2px;
    display: flex;
  }
  .icon svg {
    width: 21px;
    height: 21px;
    fill: currentColor;
    display: block;
  }
  .icon svg.gear {
    transform: rotate(30deg);
  }
  .icon:hover { color: #fff; }

  .timer { text-align: center; }
  .goal-input {
    width: 100%;
    box-sizing: border-box;
    background: #16161c;
    border: 1px solid #2a2a33;
    color: #f4f4f5;
    padding: 9px 11px;
    border-radius: 8px;
    font-size: 13px;
    text-align: center;
    margin-bottom: 16px;
  }
  .goal-input::placeholder { color: #52525b; }
  .goal {
    font-size: 14px;
    color: #f4f4f5;
    margin: 0 0 6px;
    font-weight: 500;
  }
  .inputs {
    display: flex;
    gap: 12px;
    justify-content: center;
    margin-bottom: 18px;
  }
  .inputs label {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .inputs input {
    width: 72px;
    font-size: 30px;
    text-align: center;
    background: #16161c;
    border: 1px solid #2a2a33;
    color: #f4f4f5;
    padding: 8px 4px;
    border-radius: 8px;
    -moz-appearance: textfield;
  }
  .inputs span { font-size: 11px; color: #71717a; text-transform: uppercase; }

  .countdown {
    font-size: 52px;
    font-variant-numeric: tabular-nums;
    font-weight: 200;
    margin: 8px 0 2px;
    color: #d4af5a;
  }
  .hint { font-size: 11px; color: #71717a; margin: 0 0 16px; }

  .primary {
    width: 100%;
    padding: 11px;
    font-size: 14px;
    font-weight: 600;
    background: #d4af5a;
    color: #0b0b0f;
    border: none;
    border-radius: 8px;
    cursor: pointer;
  }
  .primary:hover { background: #e0bf72; }
  .primary.danger { background: #3a3a44; color: #f4f4f5; }
  .primary.danger:hover { background: #4a4a55; }

  .settings { display: flex; flex-direction: column; gap: 12px; }
  .settings label { display: flex; flex-direction: column; gap: 5px; font-size: 12px; color: #a1a1aa; }
  .settings label.row { flex-direction: row; align-items: center; justify-content: space-between; }
  .settings input[type="text"] {
    background: #16161c;
    border: 1px solid #2a2a33;
    color: #f4f4f5;
    padding: 7px 9px;
    border-radius: 7px;
    font-size: 13px;
  }
  .settings input[type="range"] { accent-color: #d4af5a; }
  .settings input[type="color"] { width: 40px; height: 26px; border: none; background: none; padding: 0; }
  .settings input[type="checkbox"] { width: 16px; height: 16px; accent-color: #d4af5a; }
</style>
