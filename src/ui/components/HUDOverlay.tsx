import { useMemo } from "react";
import { useGameStore } from "../store";
import styles from "../styles/HUD.module.css";

const TYPE_COLORS: Record<string, string> = {
  Standard: "var(--neon-green)",
  Sprint: "var(--neon-cyan)",
  Exoatmospheric: "var(--neon-magenta)",
  AreaDenial: "var(--neon-orange)",
};

function typeAbbrev(typeName: string) {
  switch (typeName) {
    case "Sprint":
      return "SPR";
    case "Exoatmospheric":
      return "EXO";
    case "AreaDenial":
      return "ADN";
    default:
      return "STD";
  }
}

export function HUDOverlay() {
  const hud = useGameStore((state) => state.hud);
  const phase = useGameStore((state) => state.phase);

  const cityRatio = hud.citiesTotal > 0 ? hud.citiesAlive / hud.citiesTotal : 1;
  const cityTone = cityRatio > 0.5 ? "good" : cityRatio > 0.25 ? "warn" : "danger";

  const contactsLine = useMemo(() => {
    if (phase !== "WaveActive" || hud.contactsTotal === 0) return "";
    const breakdown =
      hud.contactsGlow > 0 ? ` (${hud.contactsRadar}R/${hud.contactsGlow}G)` : "";
    return `CONTACTS: ${hud.contactsTotal}${breakdown}`;
  }, [hud.contactsGlow, hud.contactsRadar, hud.contactsTotal, phase]);

  const weatherLine =
    hud.weather && hud.weather !== "Clear"
      ? `${hud.weather.toUpperCase()} - WIND ${Math.abs(hud.windX).toFixed(0)}m/s ${
          hud.windX > 0 ? ">>" : "<<"
        }`
      : "";

  const battery = hud.battery;
  const batteryLabel = battery
    ? `BAT-${battery.index + 1} [${battery.ammo}/${battery.maxAmmo}] ${typeAbbrev(
        battery.typeName
      )}`
    : "BAT- --";

  const batteryColor = battery
    ? battery.ammo > 0
      ? TYPE_COLORS[battery.typeName] ?? "var(--neon-cyan)"
      : "var(--hot-pink)"
    : "var(--neon-cyan)";

  return (
    <div className={styles.hudRoot}>
      <div className={styles.left}>
        <div className={styles.wave}>WAVE: {hud.waveNumber || "--"}</div>
        {contactsLine && <div className={styles.line}>{contactsLine}</div>}
        <div className={styles.line} style={{ color: batteryColor }}>
          {batteryLabel}
        </div>
        {weatherLine && (
          <div className={styles.line} data-tone={hud.weather === "Severe" ? "danger" : "warn"}>
            {weatherLine}
          </div>
        )}
        {hud.muted && <div className={styles.lineMuted}>[MUTED]</div>}
      </div>

      <div className={styles.right}>
        <div className={styles.cities} data-tone={cityTone}>
          CITIES: {hud.citiesAlive}/{hud.citiesTotal}
        </div>
        <div className={styles.resources}>RESOURCES: ${hud.resources}</div>
      </div>
    </div>
  );
}
