export interface TrackDef {
  id: string;
  introUrl: string;
  loopUrl: string;
}

export const TRACKS: Record<string, TrackDef> = {
  title: { id: "title", introUrl: "/music/title-intro.wav", loopUrl: "/music/title-loop.wav" },
  menu: { id: "menu", introUrl: "/music/menu-intro.wav", loopUrl: "/music/menu-loop.wav" },
  level1: { id: "level1", introUrl: "/music/level1-intro.wav", loopUrl: "/music/level1-loop.wav" },
  level2: { id: "level2", introUrl: "/music/level2-intro.wav", loopUrl: "/music/level2-loop.wav" },
  level3: { id: "level3", introUrl: "/music/level3-intro.wav", loopUrl: "/music/level3-loop.wav" },
  gameover: { id: "gameover", introUrl: "/music/gameover-intro.wav", loopUrl: "/music/gameover-loop.wav" },
};

export function getTrackForPhase(phase: string, waveNumber: number): string | null {
  switch (phase) {
    case "MainMenu":
      return "title";
    case "Strategic":
    case "WaveResult":
      return "menu";
    case "WaveActive":
      return getLevelTrack(waveNumber);
    case "CampaignOver":
      return "gameover";
    default:
      return null;
  }
}

function getLevelTrack(waveNumber: number): string {
  if (waveNumber <= 10) return "level1";
  if (waveNumber <= 20) return "level2";
  return "level3";
}
