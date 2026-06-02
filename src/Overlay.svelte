<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { playAlarm } from "./lib/sounds.js";
  import confetti from "canvas-confetti";

  let label = $state("");
  let color = $state("#0b0b0f");
  let opacity = $state(0.78);
  let sound = $state(true);
  let soundId = $state("beep");
  let soundData = $state(null);

  let stopAlarmFn = null;
  let confettiTimer = null;

  function burstConfetti() {
    const opts = { spread: 70, startVelocity: 45, ticks: 220, zIndex: 9999, disableForReducedMotion: false };
    // two side cannons for a celebratory pop
    confetti({ ...opts, particleCount: 90, angle: 60, origin: { x: 0, y: 0.7 } });
    confetti({ ...opts, particleCount: 90, angle: 120, origin: { x: 1, y: 0.7 } });
  }

  function startConfetti() {
    burstConfetti();
    confettiTimer = setInterval(burstConfetti, 2600); // keep celebrating until dismissed
  }

  function stopConfetti() {
    if (confettiTimer) clearInterval(confettiTimer);
    confettiTimer = null;
    confetti.reset();
  }

  function hexToRgba(hex, a) {
    const h = hex.replace("#", "");
    const v = h.length === 3 ? h.split("").map((c) => c + c).join("") : h;
    const r = parseInt(v.slice(0, 2), 16);
    const g = parseInt(v.slice(2, 4), 16);
    const b = parseInt(v.slice(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${a})`;
  }
  const bg = $derived(hexToRgba(color, opacity));

  function startAlarm() {
    stopAlarm();
    stopAlarmFn = playAlarm({ id: soundId, custom: soundData });
  }

  function stopAlarm() {
    if (stopAlarmFn) stopAlarmFn();
    stopAlarmFn = null;
  }

  async function dismiss() {
    stopAlarm();
    stopConfetti();
    await invoke("dismiss_overlay").catch(() => {});
  }

  function onKey(e) {
    if (e.key === "Escape" || e.key === " " || e.code === "Space") {
      e.preventDefault();
      dismiss();
    }
  }

  onMount(async () => {
    const cfg = await invoke("get_overlay_config").catch(() => null);
    if (cfg) {
      label = cfg.label || "";
      color = cfg.color || color;
      opacity = cfg.opacity ?? opacity;
      sound = cfg.sound ?? sound;
      soundId = cfg.soundId || soundId;
      soundData = cfg.soundData ?? null;
    }
    if (sound) startAlarm();
    startConfetti();
    window.addEventListener("keydown", onKey);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKey);
    stopAlarm();
    stopConfetti();
  });
</script>

<div class="overlay" style="background:{bg}" role="button" tabindex="0">
  <div class="content">
    <p class="msg">{label || "Time's up!"}</p>
    <button class="dismiss" onclick={dismiss}>Dismiss</button>
    <p class="kbd">press <kbd>Esc</kbd> or <kbd>Space</kbd></p>
  </div>
</div>

<style>
  :global(html, body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
    -webkit-user-select: none;
    user-select: none;
    cursor: default;
  }
  .overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    font-family: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, sans-serif;
  }
  .content { text-align: center; animation: fade 0.5s ease; }
  .msg {
    font-size: 64px;
    font-weight: 200;
    color: #fff;
    letter-spacing: 0.01em;
    margin: 0 0 36px;
    text-shadow: 0 2px 30px rgba(0, 0, 0, 0.5);
  }
  .dismiss {
    font-size: 18px;
    font-weight: 600;
    color: #0b0b0f;
    background: #d4af5a;
    border: none;
    padding: 14px 48px;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.3s ease;
    animation: shake 2.2s ease-in-out infinite;
  }
  .dismiss:hover { background: #e0bf72; }
  /* shake burst (~0.3s) then pause, looping */
  @keyframes shake {
    0% { transform: translateX(0) rotate(0); }
    2% { transform: translateX(-7px) rotate(-3deg); }
    4% { transform: translateX(7px) rotate(3deg); }
    6% { transform: translateX(-6px) rotate(-2deg); }
    8% { transform: translateX(6px) rotate(2deg); }
    10% { transform: translateX(-3px) rotate(-1deg); }
    12%, 100% { transform: translateX(0) rotate(0); }
  }
  .kbd { margin-top: 22px; font-size: 13px; color: rgba(255, 255, 255, 0.6); }
  kbd {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    padding: 2px 6px;
    font-family: monospace;
  }
  @keyframes fade {
    from { opacity: 0; transform: scale(0.96); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
