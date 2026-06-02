<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  let message = $state("Time's up!");
  let label = $state("");
  let color = $state("#0b0b0f");
  let opacity = $state(0.78);
  let sound = $state(true);

  let audioCtx = null;
  let beepTimer = null;

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
    try {
      audioCtx = new (window.AudioContext || window.webkitAudioContext)();
      const beep = () => {
        const osc = audioCtx.createOscillator();
        const gain = audioCtx.createGain();
        osc.type = "sine";
        osc.frequency.value = 880;
        gain.gain.setValueAtTime(0.0001, audioCtx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.25, audioCtx.currentTime + 0.02);
        gain.gain.exponentialRampToValueAtTime(0.0001, audioCtx.currentTime + 0.3);
        osc.connect(gain).connect(audioCtx.destination);
        osc.start();
        osc.stop(audioCtx.currentTime + 0.32);
      };
      beep();
      beepTimer = setInterval(beep, 1000);
    } catch (e) {
      console.error("alarm", e);
    }
  }

  function stopAlarm() {
    if (beepTimer) clearInterval(beepTimer);
    beepTimer = null;
    if (audioCtx) audioCtx.close().catch(() => {});
    audioCtx = null;
  }

  async function dismiss() {
    stopAlarm();
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
      message = cfg.message || message;
      label = cfg.label || "";
      color = cfg.color || color;
      opacity = cfg.opacity ?? opacity;
      sound = cfg.sound ?? sound;
    }
    if (sound) startAlarm();
    window.addEventListener("keydown", onKey);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKey);
    stopAlarm();
  });
</script>

<div class="overlay" style="background:{bg}" role="button" tabindex="0">
  <div class="content">
    {#if label}<p class="goal">{label}</p>{/if}
    <p class="msg">{message}</p>
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
  .goal {
    font-size: 24px;
    font-weight: 400;
    color: #d4af5a;
    margin: 0 0 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    text-shadow: 0 2px 20px rgba(0, 0, 0, 0.5);
  }
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
    transition: transform 0.3s ease, background 0.3s ease;
  }
  .dismiss:hover { background: #e0bf72; transform: scale(1.03); }
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
