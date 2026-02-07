import type { AvailableAction } from "../types/campaign";

export interface GameActions {
  setMuted: (muted: boolean) => void;
  setVolume: (volume: number) => void;
  setSfxVolume: (volume: number) => void;
  setMusicVolume: (volume: number) => void;
  playUiClick: () => void;
  handleStrategicAction: (action: AvailableAction) => void;
}

let actions: GameActions | null = null;

export function registerGameActions(next: GameActions) {
  actions = next;
}

export function getGameActions(): GameActions | null {
  return actions;
}

export function playUiClick() {
  actions?.playUiClick();
}

export function setMuted(muted: boolean) {
  actions?.setMuted(muted);
}

export function setVolume(volume: number) {
  actions?.setVolume(volume);
}

export function setSfxVolume(volume: number) {
  actions?.setSfxVolume(volume);
}

export function setMusicVolume(volume: number) {
  actions?.setMusicVolume(volume);
}

export function handleStrategicAction(action: AvailableAction) {
  actions?.handleStrategicAction(action);
}
