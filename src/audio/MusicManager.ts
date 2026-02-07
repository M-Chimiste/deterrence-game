import { TRACKS, getTrackForPhase } from "./trackConfig";
import type { TrackDef } from "./trackConfig";

const CROSSFADE_DURATION = 1.5; // seconds

/**
 * Handles music playback with seamless intro→loop transitions
 * and crossfade between tracks on phase changes.
 */
export class MusicManager {
  private ctx: AudioContext;
  private musicGain: GainNode;
  private _volume: number = 0.7;

  // Buffer cache: trackId → { intro, loop }
  private bufferCache: Map<string, { intro: AudioBuffer; loop: AudioBuffer }> = new Map();
  private loadingPromises: Map<string, Promise<void>> = new Map();

  // Active playback
  private currentTrackId: string | null = null;
  private introSource: AudioBufferSourceNode | null = null;
  private loopSource: AudioBufferSourceNode | null = null;
  private activeGain: GainNode | null = null;

  // Stale transition guard
  private transitionCounter: number = 0;

  // Pending phase for autoplay resume
  private pendingPhase: string | null = null;
  private pendingWaveNumber: number = 0;

  constructor(ctx: AudioContext, outputNode: GainNode) {
    this.ctx = ctx;
    this.musicGain = ctx.createGain();
    this.musicGain.gain.value = this._volume;
    this.musicGain.connect(outputNode);

    // When AudioContext resumes after user gesture, restart pending music
    this.ctx.addEventListener("statechange", () => {
      if (this.ctx.state === "running" && this.pendingPhase && !this.currentTrackId) {
        this.setPhase(this.pendingPhase, this.pendingWaveNumber);
      }
    });
  }

  async loadTrack(trackDef: TrackDef): Promise<void> {
    if (this.bufferCache.has(trackDef.id)) return;

    // Deduplicate concurrent loads
    const existing = this.loadingPromises.get(trackDef.id);
    if (existing) return existing;

    const promise = (async () => {
      const [introResp, loopResp] = await Promise.all([
        fetch(trackDef.introUrl),
        fetch(trackDef.loopUrl),
      ]);
      const [introArrayBuf, loopArrayBuf] = await Promise.all([
        introResp.arrayBuffer(),
        loopResp.arrayBuffer(),
      ]);
      const [introBuffer, loopBuffer] = await Promise.all([
        this.ctx.decodeAudioData(introArrayBuf),
        this.ctx.decodeAudioData(loopArrayBuf),
      ]);
      this.bufferCache.set(trackDef.id, { intro: introBuffer, loop: loopBuffer });
    })();

    this.loadingPromises.set(trackDef.id, promise);
    try {
      await promise;
    } finally {
      this.loadingPromises.delete(trackDef.id);
    }
  }

  /** Start a track with intro→loop scheduling */
  private playTrack(trackId: string): void {
    const buffers = this.bufferCache.get(trackId);
    if (!buffers) return;

    // Per-track gain node for crossfade envelope
    const channelGain = this.ctx.createGain();
    channelGain.gain.value = 0; // starts at 0, fadeIn ramps up
    channelGain.connect(this.musicGain);

    // Intro source — plays once
    const introSource = this.ctx.createBufferSource();
    introSource.buffer = buffers.intro;
    introSource.connect(channelGain);

    const startTime = this.ctx.currentTime;
    introSource.start(startTime);

    // Loop source — scheduled at exact intro end, loops forever
    const loopSource = this.ctx.createBufferSource();
    loopSource.buffer = buffers.loop;
    loopSource.loop = true;
    loopSource.connect(channelGain);
    loopSource.start(startTime + buffers.intro.duration);

    this.introSource = introSource;
    this.loopSource = loopSource;
    this.activeGain = channelGain;
    this.currentTrackId = trackId;

    // When intro ends, clear the reference
    introSource.onended = () => {
      if (this.introSource === introSource) {
        this.introSource = null;
      }
    };
  }

  /** Crossfade to the appropriate track for the given phase */
  async setPhase(phase: string, waveNumber: number): Promise<void> {
    // Always track the desired phase for autoplay resume
    this.pendingPhase = phase;
    this.pendingWaveNumber = waveNumber;

    const trackId = getTrackForPhase(phase, waveNumber);

    // Same track already playing — no change
    if (trackId === this.currentTrackId) return;

    const thisTransition = ++this.transitionCounter;

    // Fade out current track
    this.fadeOutCurrent();

    // No track for this phase — fade to silence
    if (!trackId) return;

    // Load track if needed
    const trackDef = TRACKS[trackId];
    if (!trackDef) return;

    try {
      await this.loadTrack(trackDef);
    } catch (err) {
      console.warn(`Failed to load music track "${trackId}":`, err);
      return;
    }

    // Stale transition guard — abort if a newer transition happened while loading
    if (this.transitionCounter !== thisTransition) return;

    // Don't start playback if context is suspended — statechange listener will retry
    if (this.ctx.state === "suspended") return;

    // Play and fade in
    this.playTrack(trackId);
    this.fadeIn();

    // Preload the next likely track
    this.preloadNext(phase, waveNumber);
  }

  private fadeOutCurrent(): void {
    if (!this.activeGain) return;

    const gainNode = this.activeGain;
    const intro = this.introSource;
    const loop = this.loopSource;

    // Ramp gain to 0
    const now = this.ctx.currentTime;
    gainNode.gain.cancelScheduledValues(now);
    gainNode.gain.setValueAtTime(gainNode.gain.value, now);
    gainNode.gain.linearRampToValueAtTime(0, now + CROSSFADE_DURATION);

    // Cleanup after fade
    setTimeout(() => {
      for (const src of [intro, loop]) {
        if (src) {
          try { src.stop(); } catch { /* already stopped */ }
          try { src.disconnect(); } catch { /* already disconnected */ }
        }
      }
      try { gainNode.disconnect(); } catch { /* already disconnected */ }
    }, CROSSFADE_DURATION * 1000 + 100);

    // Clear active references
    this.activeGain = null;
    this.introSource = null;
    this.loopSource = null;
    this.currentTrackId = null;
  }

  private fadeIn(): void {
    if (!this.activeGain) return;
    const now = this.ctx.currentTime;
    this.activeGain.gain.setValueAtTime(0, now);
    this.activeGain.gain.linearRampToValueAtTime(1.0, now + CROSSFADE_DURATION);
  }

  private preloadNext(phase: string, waveNumber: number): void {
    let nextTrackId: string | null = null;
    switch (phase) {
      case "MainMenu":
        nextTrackId = "menu";
        break;
      case "Strategic":
        nextTrackId = getTrackForPhase("WaveActive", waveNumber);
        break;
      case "WaveActive":
        nextTrackId = "menu";
        break;
    }
    if (nextTrackId) {
      const def = TRACKS[nextTrackId];
      if (def) this.loadTrack(def).catch(() => { /* ignore preload failures */ });
    }
  }

  setVolume(vol: number): void {
    this._volume = Math.max(0, Math.min(1, vol));
    this.musicGain.gain.value = this._volume;
  }

  get volume(): number {
    return this._volume;
  }

  /** Preload a track for an upcoming phase */
  async preloadForPhase(phase: string, waveNumber: number): Promise<void> {
    const trackId = getTrackForPhase(phase, waveNumber);
    if (trackId) {
      const trackDef = TRACKS[trackId];
      if (trackDef) await this.loadTrack(trackDef);
    }
  }

  /** Stop all music immediately */
  stopAll(): void {
    for (const src of [this.introSource, this.loopSource]) {
      if (src) {
        try { src.stop(); } catch { /* */ }
        try { src.disconnect(); } catch { /* */ }
      }
    }
    if (this.activeGain) {
      try { this.activeGain.disconnect(); } catch { /* */ }
    }
    this.introSource = null;
    this.loopSource = null;
    this.activeGain = null;
    this.currentTrackId = null;
  }
}
