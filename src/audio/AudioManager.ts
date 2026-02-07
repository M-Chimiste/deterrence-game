import {
  createLaunchSound,
  createDetonationSound,
  createCityDamageSound,
  createWaveChime,
  createMirvSplitSound,
} from "./SoundSynth";

const WORLD_WIDTH = 1280;

/**
 * Manages all game audio using Web Audio API synthesized sounds.
 * Creates AudioContext on first user gesture (browser autoplay policy).
 */
export class AudioManager {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private effectsGain: GainNode | null = null;
  private ambientGain: GainNode | null = null;
  private ambientOsc: OscillatorNode | null = null;
  private _muted: boolean = false;
  private _volume: number = 0.5;

  /** Ensure AudioContext exists (must be called after user gesture) */
  private ensureContext(): AudioContext {
    if (!this.ctx) {
      this.ctx = new AudioContext();
      this.masterGain = this.ctx.createGain();
      this.masterGain.gain.value = this._volume;
      this.masterGain.connect(this.ctx.destination);

      this.effectsGain = this.ctx.createGain();
      this.effectsGain.gain.value = 1.0;
      this.effectsGain.connect(this.masterGain);

      this.ambientGain = this.ctx.createGain();
      this.ambientGain.gain.value = 0.3;
      this.ambientGain.connect(this.masterGain);
    }
    // Resume if suspended (browser policy)
    if (this.ctx.state === "suspended") {
      this.ctx.resume();
    }
    return this.ctx;
  }

  /** Map world X (0..1280) to stereo pan (-1..+1) */
  private panFromWorldX(worldX: number): StereoPannerNode {
    const ctx = this.ensureContext();
    const panner = ctx.createStereoPanner();
    panner.pan.value = (worldX / WORLD_WIDTH) * 2 - 1;
    panner.connect(this.effectsGain!);
    return panner;
  }

  /** Play interceptor launch sound with spatial pan */
  playLaunch(worldX: number) {
    if (this._muted) return;
    const ctx = this.ensureContext();
    const panner = this.panFromWorldX(worldX);
    createLaunchSound(ctx, panner);
  }

  /** Play detonation sound with spatial pan and intensity */
  playDetonation(worldX: number, intensity: number = 1.0) {
    if (this._muted) return;
    const ctx = this.ensureContext();
    const panner = this.panFromWorldX(worldX);
    createDetonationSound(ctx, panner, intensity);
  }

  /** Play city damage sound with spatial pan */
  playCityDamage(worldX: number) {
    if (this._muted) return;
    const ctx = this.ensureContext();
    const panner = this.panFromWorldX(worldX);
    createCityDamageSound(ctx, panner);
  }

  /** Play ascending chime for wave start */
  playWaveStart() {
    if (this._muted) return;
    const ctx = this.ensureContext();
    createWaveChime(ctx, this.effectsGain!, true);
  }

  /** Play descending chime for wave complete */
  playWaveComplete() {
    if (this._muted) return;
    const ctx = this.ensureContext();
    createWaveChime(ctx, this.effectsGain!, false);
  }

  /** Play MIRV split crack with spatial pan */
  playMirvSplit(worldX: number) {
    if (this._muted) return;
    const ctx = this.ensureContext();
    const panner = this.panFromWorldX(worldX);
    createMirvSplitSound(ctx, panner);
  }

  /** Start low ambient hum, modulated by weather */
  startAmbient(weather: string) {
    this.stopAmbient();
    if (this._muted) return;
    const ctx = this.ensureContext();

    this.ambientOsc = ctx.createOscillator();
    this.ambientOsc.type = "sine";
    this.ambientOsc.frequency.value = 55; // Low A1

    // Weather modulates ambient volume
    let vol = 0.05;
    if (weather === "Overcast") vol = 0.08;
    else if (weather === "Storm") vol = 0.12;
    else if (weather === "Severe") vol = 0.18;

    this.ambientGain!.gain.value = vol;
    this.ambientOsc.connect(this.ambientGain!);
    this.ambientOsc.start();
  }

  /** Stop ambient sound */
  stopAmbient() {
    if (this.ambientOsc) {
      this.ambientOsc.stop();
      this.ambientOsc.disconnect();
      this.ambientOsc = null;
    }
  }

  /** Toggle mute state */
  toggleMute(): boolean {
    this._muted = !this._muted;
    if (this.masterGain) {
      this.masterGain.gain.value = this._muted ? 0 : this._volume;
    }
    if (this._muted) {
      this.stopAmbient();
    }
    return this._muted;
  }

  get muted(): boolean {
    return this._muted;
  }

  setVolume(vol: number) {
    this._volume = Math.max(0, Math.min(1, vol));
    if (this.masterGain && !this._muted) {
      this.masterGain.gain.value = this._volume;
    }
  }
}
