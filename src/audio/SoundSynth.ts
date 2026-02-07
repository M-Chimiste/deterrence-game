/** Oscillator/noise-based synthesized sounds — retro aesthetic, no asset files. */

export function createNoiseBuffer(ctx: AudioContext, duration: number = 0.5): AudioBuffer {
  const sampleRate = ctx.sampleRate;
  const length = Math.floor(sampleRate * duration);
  const buffer = ctx.createBuffer(1, length, sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < length; i++) {
    data[i] = Math.random() * 2 - 1;
  }
  return buffer;
}

/** Rising sweep 200 -> 800 Hz, 0.15s — interceptor launch */
export function createLaunchSound(ctx: AudioContext, dest: AudioNode): void {
  const osc = ctx.createOscillator();
  osc.type = "sawtooth";
  osc.frequency.setValueAtTime(200, ctx.currentTime);
  osc.frequency.exponentialRampToValueAtTime(800, ctx.currentTime + 0.15);

  const env = ctx.createGain();
  env.gain.setValueAtTime(0.3, ctx.currentTime);
  env.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.2);

  osc.connect(env);
  env.connect(dest);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + 0.2);
}

/** Low boom 60 Hz + noise burst — detonation */
export function createDetonationSound(ctx: AudioContext, dest: AudioNode, intensity: number = 1.0): void {
  // Low frequency boom
  const osc = ctx.createOscillator();
  osc.type = "sine";
  osc.frequency.setValueAtTime(60, ctx.currentTime);
  osc.frequency.exponentialRampToValueAtTime(20, ctx.currentTime + 0.4);

  const env = ctx.createGain();
  env.gain.setValueAtTime(0.4 * intensity, ctx.currentTime);
  env.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.5);

  osc.connect(env);
  env.connect(dest);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + 0.5);

  // Noise burst
  const noise = ctx.createBufferSource();
  noise.buffer = createNoiseBuffer(ctx, 0.3);

  const filter = ctx.createBiquadFilter();
  filter.type = "lowpass";
  filter.frequency.setValueAtTime(2000, ctx.currentTime);
  filter.frequency.exponentialRampToValueAtTime(200, ctx.currentTime + 0.3);

  const noiseEnv = ctx.createGain();
  noiseEnv.gain.setValueAtTime(0.25 * intensity, ctx.currentTime);
  noiseEnv.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.3);

  noise.connect(filter);
  filter.connect(noiseEnv);
  noiseEnv.connect(dest);
  noise.start(ctx.currentTime);
}

/** Filtered noise rumble — city taking damage */
export function createCityDamageSound(ctx: AudioContext, dest: AudioNode): void {
  const noise = ctx.createBufferSource();
  noise.buffer = createNoiseBuffer(ctx, 0.6);

  const filter = ctx.createBiquadFilter();
  filter.type = "lowpass";
  filter.frequency.setValueAtTime(400, ctx.currentTime);

  const env = ctx.createGain();
  env.gain.setValueAtTime(0.2, ctx.currentTime);
  env.gain.linearRampToValueAtTime(0.0, ctx.currentTime + 0.6);

  noise.connect(filter);
  filter.connect(env);
  env.connect(dest);
  noise.start(ctx.currentTime);
}

/** 3-note chime — wave start/complete */
export function createWaveChime(ctx: AudioContext, dest: AudioNode, ascending: boolean): void {
  const notes = ascending ? [440, 554, 659] : [659, 554, 440];
  const spacing = 0.12;

  for (let i = 0; i < notes.length; i++) {
    const osc = ctx.createOscillator();
    osc.type = "triangle";
    osc.frequency.setValueAtTime(notes[i], ctx.currentTime + i * spacing);

    const env = ctx.createGain();
    env.gain.setValueAtTime(0, ctx.currentTime + i * spacing);
    env.gain.linearRampToValueAtTime(0.2, ctx.currentTime + i * spacing + 0.02);
    env.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + i * spacing + 0.2);

    osc.connect(env);
    env.connect(dest);
    osc.start(ctx.currentTime + i * spacing);
    osc.stop(ctx.currentTime + i * spacing + 0.25);
  }
}

/** High crack/pop — MIRV split */
export function createMirvSplitSound(ctx: AudioContext, dest: AudioNode): void {
  const osc = ctx.createOscillator();
  osc.type = "square";
  osc.frequency.setValueAtTime(1200, ctx.currentTime);
  osc.frequency.exponentialRampToValueAtTime(200, ctx.currentTime + 0.08);

  const env = ctx.createGain();
  env.gain.setValueAtTime(0.25, ctx.currentTime);
  env.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.1);

  osc.connect(env);
  env.connect(dest);
  osc.start(ctx.currentTime);
  osc.stop(ctx.currentTime + 0.12);
}
