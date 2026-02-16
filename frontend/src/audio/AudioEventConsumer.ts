/**
 * Maps game audio events from snapshots to SFX engine calls.
 */

import type { AudioEvent } from "../ipc/state";
import type { SfxEngine } from "./SfxEngine";

export function consumeAudioEvents(events: AudioEvent[], sfx: SfxEngine): void {
  for (const event of events) {
    switch (event.type) {
      case "NewContact":
        sfx.newContact();
        break;
      case "ContactLost":
        sfx.contactLost();
        break;
      case "VetoClockStart":
        sfx.vetoClockStart();
        break;
      case "VetoClockWarning":
        sfx.vetoClockWarning(event.remaining_secs);
        break;
      case "BirdAway":
        sfx.birdAway();
        break;
      case "Splash":
        if (event.result === "Hit") {
          sfx.splashHit();
        } else {
          sfx.splashMiss();
        }
        break;
      case "VampireImpact":
        sfx.vampireImpact();
        break;
      case "ThreatEvaluated":
        // No distinct sound for now; visual feedback handled by UI
        break;
    }
  }
}
