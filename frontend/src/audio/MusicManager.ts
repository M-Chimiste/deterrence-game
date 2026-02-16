/**
 * Phase-based music manager using Howler.js.
 *
 * Each music phase has an intro + loop pair. When the phase changes,
 * the current track fades out and the new intro plays, seamlessly
 * transitioning to its loop on completion.
 */

import { Howl } from "howler";

export type MusicPhase =
  | "menu"
  | "level1"
  | "level2"
  | "level3"
  | "gameover"
  | "silent";

interface MusicTrack {
  intro: Howl;
  loop: Howl;
}

const FADE_MS = 1000;

export class MusicManager {
  private tracks = new Map<string, MusicTrack>();
  private currentPhase: MusicPhase = "silent";
  private volume = 0.3;

  init(): void {
    const defs = [
      { key: "menu", intro: "/music/menu-intro.wav", loop: "/music/menu-loop.wav" },
      { key: "level1", intro: "/music/level1-intro.wav", loop: "/music/level1-loop.wav" },
      { key: "level2", intro: "/music/level2-intro.wav", loop: "/music/level2-loop.wav" },
      { key: "level3", intro: "/music/level3-intro.wav", loop: "/music/level3-loop.wav" },
      { key: "gameover", intro: "/music/gameover-intro.wav", loop: "/music/gameover-loop.wav" },
    ];

    for (const d of defs) {
      const loopHowl = new Howl({
        src: [d.loop],
        loop: true,
        volume: 0,
        preload: true,
      });

      const introHowl = new Howl({
        src: [d.intro],
        loop: false,
        volume: 0,
        preload: true,
        onend: () => {
          if (this.currentPhase === d.key) {
            loopHowl.volume(this.volume);
            loopHowl.play();
          }
        },
      });

      this.tracks.set(d.key, { intro: introHowl, loop: loopHowl });
    }
  }

  setPhase(phase: MusicPhase): void {
    if (phase === this.currentPhase) return;

    // Fade out and stop current track
    const current = this.tracks.get(this.currentPhase);
    if (current) {
      this.fadeAndStop(current.intro);
      this.fadeAndStop(current.loop);
    }

    this.currentPhase = phase;

    if (phase === "silent") return;

    // Play new track intro
    const track = this.tracks.get(phase);
    if (track) {
      track.intro.volume(this.volume);
      track.intro.play();
    }
  }

  setVolume(vol: number): void {
    this.volume = Math.max(0, Math.min(1, vol));
    const track = this.tracks.get(this.currentPhase);
    if (track) {
      if (track.intro.playing()) track.intro.volume(this.volume);
      if (track.loop.playing()) track.loop.volume(this.volume);
    }
  }

  pause(): void {
    const track = this.tracks.get(this.currentPhase);
    if (track) {
      track.intro.pause();
      track.loop.pause();
    }
  }

  resume(): void {
    const track = this.tracks.get(this.currentPhase);
    if (!track) return;
    // Resume whichever was playing
    if (track.loop.seek() > 0) {
      track.loop.play();
    } else {
      track.intro.play();
    }
  }

  private fadeAndStop(howl: Howl): void {
    if (howl.playing()) {
      howl.fade(howl.volume(), 0, FADE_MS);
      setTimeout(() => howl.stop(), FADE_MS);
    }
  }
}
