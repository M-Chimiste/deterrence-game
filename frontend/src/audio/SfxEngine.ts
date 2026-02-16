/**
 * Procedural SFX engine using Web Audio API.
 *
 * All sounds are generated from oscillators and noise — no audio files needed.
 * Must be initialized after a user gesture (click/keydown) to satisfy autoplay policy.
 */

export class SfxEngine {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private volume = 0.5;

  init(): void {
    this.ctx = new AudioContext();
    this.masterGain = this.ctx.createGain();
    this.masterGain.gain.value = this.volume;
    this.masterGain.connect(this.ctx.destination);

    if (this.ctx.state === "suspended") {
      this.ctx.resume();
    }
  }

  setVolume(vol: number): void {
    this.volume = Math.max(0, Math.min(1, vol));
    if (this.masterGain) {
      this.masterGain.gain.value = this.volume;
    }
  }

  /** Sonar ping — sine sweep 800->1200Hz, 200ms with decay. */
  newContact(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    const osc = this.ctx!.createOscillator();
    const gain = this.ctx!.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(800, now);
    osc.frequency.exponentialRampToValueAtTime(1200, now + 0.2);

    gain.gain.setValueAtTime(0.4, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);

    osc.connect(gain).connect(this.masterGain!);
    osc.start(now);
    osc.stop(now + 0.25);
  }

  /** Descending tone — sine 1000->500Hz, 300ms. */
  contactLost(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    const osc = this.ctx!.createOscillator();
    const gain = this.ctx!.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(1000, now);
    osc.frequency.exponentialRampToValueAtTime(500, now + 0.3);

    gain.gain.setValueAtTime(0.3, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.35);

    osc.connect(gain).connect(this.masterGain!);
    osc.start(now);
    osc.stop(now + 0.35);
  }

  /** Two 880Hz attention beeps, 100ms each. */
  vetoClockStart(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    for (let i = 0; i < 2; i++) {
      const t = now + i * 0.15;
      const osc = this.ctx!.createOscillator();
      const gain = this.ctx!.createGain();

      osc.type = "sine";
      osc.frequency.value = 880;

      gain.gain.setValueAtTime(0.3, t);
      gain.gain.setValueAtTime(0.3, t + 0.08);
      gain.gain.exponentialRampToValueAtTime(0.001, t + 0.1);

      osc.connect(gain).connect(this.masterGain!);
      osc.start(t);
      osc.stop(t + 0.1);
    }
  }

  /** Urgent beep — higher pitch, faster repetition proportional to urgency. */
  vetoClockWarning(remainingSecs: number): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    // More urgent = higher pitch and faster
    const pitch = remainingSecs <= 1.0 ? 1200 : 1000;
    const count = remainingSecs <= 1.0 ? 3 : 2;
    const gap = remainingSecs <= 1.0 ? 0.1 : 0.12;

    for (let i = 0; i < count; i++) {
      const t = now + i * gap;
      const osc = this.ctx!.createOscillator();
      const gain = this.ctx!.createGain();

      osc.type = "square";
      osc.frequency.value = pitch;

      gain.gain.setValueAtTime(0.2, t);
      gain.gain.setValueAtTime(0.2, t + 0.05);
      gain.gain.exponentialRampToValueAtTime(0.001, t + 0.07);

      osc.connect(gain).connect(this.masterGain!);
      osc.start(t);
      osc.stop(t + 0.07);
    }
  }

  /** Rising sawtooth sweep 200->2000Hz, 500ms — missile launch. */
  birdAway(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    const osc = this.ctx!.createOscillator();
    const gain = this.ctx!.createGain();

    osc.type = "sawtooth";
    osc.frequency.setValueAtTime(200, now);
    osc.frequency.exponentialRampToValueAtTime(2000, now + 0.5);

    gain.gain.setValueAtTime(0.25, now);
    gain.gain.setValueAtTime(0.25, now + 0.3);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.5);

    osc.connect(gain).connect(this.masterGain!);
    osc.start(now);
    osc.stop(now + 0.5);
  }

  /** Sharp noise burst — successful intercept. */
  splashHit(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    // White noise via buffer
    const buffer = this.ctx!.createBuffer(1, this.ctx!.sampleRate * 0.08, this.ctx!.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < data.length; i++) {
      data[i] = Math.random() * 2 - 1;
    }

    const source = this.ctx!.createBufferSource();
    source.buffer = buffer;

    const bandpass = this.ctx!.createBiquadFilter();
    bandpass.type = "bandpass";
    bandpass.frequency.value = 2000;
    bandpass.Q.value = 1.5;

    const gain = this.ctx!.createGain();
    gain.gain.setValueAtTime(0.4, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.08);

    source.connect(bandpass).connect(gain).connect(this.masterGain!);
    source.start(now);
  }

  /** Low tone — missed intercept. Sine 200Hz, 300ms fade. */
  splashMiss(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    const osc = this.ctx!.createOscillator();
    const gain = this.ctx!.createGain();

    osc.type = "sine";
    osc.frequency.value = 200;

    gain.gain.setValueAtTime(0.3, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.3);

    osc.connect(gain).connect(this.masterGain!);
    osc.start(now);
    osc.stop(now + 0.3);
  }

  /** Alarm klaxon — alternating 440/880Hz square wave, 1s. */
  vampireImpact(): void {
    if (!this.ready()) return;
    const now = this.ctx!.currentTime;

    for (let i = 0; i < 6; i++) {
      const t = now + i * 0.16;
      const osc = this.ctx!.createOscillator();
      const gain = this.ctx!.createGain();

      osc.type = "square";
      osc.frequency.value = i % 2 === 0 ? 440 : 880;

      gain.gain.setValueAtTime(0.3, t);
      gain.gain.setValueAtTime(0.3, t + 0.12);
      gain.gain.exponentialRampToValueAtTime(0.001, t + 0.15);

      osc.connect(gain).connect(this.masterGain!);
      osc.start(t);
      osc.stop(t + 0.15);
    }
  }

  private ready(): boolean {
    if (!this.ctx || !this.masterGain) return false;
    if (this.ctx.state === "suspended") {
      this.ctx.resume();
    }
    return true;
  }
}
