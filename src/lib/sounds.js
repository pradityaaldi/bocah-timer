// Synthesized alarm presets (Web Audio — offline, no audio files needed),
// a bundled meme clip, plus support for a user-supplied custom sound (data URL).
import fahUrl from "./fah.mp3";
import tamatUrl from "./tamatlah.mp3";

export const PRESETS = [
  { id: "beep", name: "Beep", loop: 1000, play: beep },
  { id: "chime", name: "Chime", loop: 1600, play: chime },
  { id: "digital", name: "Digital", loop: 1100, play: digital },
  { id: "tamat", name: "Tamatlah Sudah 💀", url: tamatUrl },
  { id: "fah", name: "Fahhh 😤", url: fahUrl },
];

function osc(ctx, { freq, t, dur, type = "sine", gain = 0.5, detune = 0 }) {
  const o = ctx.createOscillator();
  const g = ctx.createGain();
  o.type = type;
  o.frequency.value = freq;
  o.detune.value = detune;
  const start = ctx.currentTime + t;
  g.gain.setValueAtTime(0.0001, start);
  g.gain.exponentialRampToValueAtTime(gain, start + 0.02);
  g.gain.exponentialRampToValueAtTime(0.0001, start + dur);
  o.connect(g).connect(ctx.destination);
  o.start(start);
  o.stop(start + dur + 0.05);
}

function beep(ctx) {
  osc(ctx, { freq: 880, t: 0, dur: 0.18, type: "square", gain: 0.45 });
  osc(ctx, { freq: 880, t: 0.25, dur: 0.18, type: "square", gain: 0.45 });
}

function chime(ctx) {
  [523, 659, 784].forEach((f, i) =>
    osc(ctx, { freq: f, t: i * 0.18, dur: 0.55, type: "sine", gain: 0.6 }),
  );
}

function digital(ctx) {
  for (let i = 0; i < 4; i++) {
    osc(ctx, { freq: 1000, t: i * 0.12, dur: 0.07, type: "square", gain: 0.4 });
  }
}

// Start the alarm looping. Returns a stop() function.
export function playAlarm({ id, custom } = {}) {
  const preset = PRESETS.find((p) => p.id === id) || PRESETS[0];
  // file-backed sound: bundled meme clip or user custom upload
  const src = id === "custom" && custom ? custom : preset.url;
  if (src) {
    const audio = new Audio(src);
    audio.loop = true;
    audio.play().catch(() => {});
    return () => {
      audio.pause();
      audio.src = "";
    };
  }
  let ctx;
  try {
    ctx = new (window.AudioContext || window.webkitAudioContext)();
  } catch {
    return () => {};
  }
  const tick = () => preset.play(ctx);
  tick();
  const iv = setInterval(tick, preset.loop);
  return () => {
    clearInterval(iv);
    ctx.close().catch(() => {});
  };
}

// Play a single preview pass (used by the Test button). Returns stop().
export function previewAlarm({ id, custom } = {}) {
  const stop = playAlarm({ id, custom });
  // auto-stop the preview after a short window
  const timer = setTimeout(stop, 4000);
  return () => {
    clearTimeout(timer);
    stop();
  };
}
