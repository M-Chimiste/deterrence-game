import { Application, Container, Graphics, Text, TextStyle } from "pixi.js";
import type { StateSnapshot } from "../types/snapshot";
import type { WaveCompleteEvent } from "../types/events";
import {
  NEON_CYAN, NEON_GREEN, HOT_PINK, SOLAR_YELLOW,
  NEON_ORANGE, PANEL_DARK, FONT_FAMILY, TYPE_COLORS,
} from "./ui/Theme";

export class HUD {
  private container: Container;
  private waveText: Text;
  private citiesText: Text;
  private resourcesText: Text;
  private phaseText: Text;
  private batteryText: Text;
  private weatherText: Text;
  private waveCompleteContainer: Container;
  private waveCompletePanel: Graphics;
  private waveCompleteText: Text;

  constructor(app: Application, worldWidth: number, _worldHeight: number) {
    this.container = new Container();
    app.stage.addChild(this.container);

    const style = new TextStyle({
      fontFamily: FONT_FAMILY,
      fontSize: 16,
      fill: NEON_CYAN,
    });

    // Title
    const titleStyle = new TextStyle({
      fontFamily: FONT_FAMILY,
      fontSize: 20,
      fill: NEON_CYAN,
      fontWeight: "bold",
    });
    const title = new Text({ text: "DETERRENCE", style: titleStyle });
    title.x = worldWidth / 2 - 60;
    title.y = 8;
    this.container.addChild(title);

    // Wave indicator
    this.waveText = new Text({ text: "WAVE: --", style });
    this.waveText.x = 10;
    this.waveText.y = 8;
    this.container.addChild(this.waveText);

    // Cities remaining
    this.citiesText = new Text({ text: "CITIES: -/-", style });
    this.citiesText.x = worldWidth - 160;
    this.citiesText.y = 8;
    this.container.addChild(this.citiesText);

    // Resources
    this.resourcesText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 14,
        fill: SOLAR_YELLOW,
      }),
    });
    this.resourcesText.x = worldWidth - 160;
    this.resourcesText.y = 30;
    this.container.addChild(this.resourcesText);

    // Phase indicator
    this.phaseText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 14,
        fill: SOLAR_YELLOW,
      }),
    });
    this.phaseText.x = 10;
    this.phaseText.y = 30;
    this.container.addChild(this.phaseText);

    // Battery selection indicator
    this.batteryText = new Text({
      text: "BAT-1 [--]",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 14,
        fill: NEON_CYAN,
      }),
    });
    this.batteryText.x = 10;
    this.batteryText.y = 52;
    this.container.addChild(this.batteryText);

    // Weather indicator
    this.weatherText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 14,
        fill: SOLAR_YELLOW,
      }),
    });
    this.weatherText.x = 10;
    this.weatherText.y = 70;
    this.container.addChild(this.weatherText);

    // Wave complete overlay
    this.waveCompleteContainer = new Container();
    this.waveCompleteContainer.visible = false;
    app.stage.addChild(this.waveCompleteContainer);

    // Dark panel behind wave complete text
    this.waveCompletePanel = new Graphics();
    this.waveCompletePanel.roundRect(worldWidth / 2 - 200, 230, 400, 200, 8);
    this.waveCompletePanel.fill({ color: PANEL_DARK, alpha: 0.9 });
    this.waveCompletePanel.roundRect(worldWidth / 2 - 200, 230, 400, 200, 8);
    this.waveCompletePanel.stroke({ width: 1, color: NEON_CYAN, alpha: 0.5 });
    this.waveCompleteContainer.addChild(this.waveCompletePanel);

    this.waveCompleteText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 16,
        fill: NEON_CYAN,
        align: "center",
      }),
    });
    this.waveCompleteText.anchor.set(0.5);
    this.waveCompleteText.x = worldWidth / 2;
    this.waveCompleteText.y = 330;
    this.waveCompleteContainer.addChild(this.waveCompleteText);
  }

  get visible(): boolean {
    return this.container.visible;
  }

  set visible(v: boolean) {
    this.container.visible = v;
    this.waveCompleteContainer.visible = v && this.waveCompleteContainer.visible;
  }

  update(snapshot: StateSnapshot) {
    this.waveText.text = `WAVE: ${snapshot.wave_number || "--"}`;

    // Count alive cities
    const cities = snapshot.entities.filter((e) => e.entity_type === "City");
    const aliveCities = cities.filter((e) => {
      if (e.extra && "City" in e.extra) {
        return (e.extra as { City: { health: number } }).City.health > 0;
      }
      return true;
    });
    this.citiesText.text = `CITIES: ${aliveCities.length}/${cities.length}`;

    // Dynamic city color based on health ratio
    const cityRatio = cities.length > 0 ? aliveCities.length / cities.length : 1;
    this.citiesText.style.fill = cityRatio > 0.5 ? NEON_GREEN : cityRatio > 0.25 ? NEON_ORANGE : HOT_PINK;

    // Phase text with detection breakdown
    if (snapshot.phase === "WaveActive") {
      const missiles = snapshot.entities.filter(
        (e) => e.entity_type === "Missile"
      );
      if (missiles.length > 0) {
        let radarCount = 0;
        let glowCount = 0;
        for (const m of missiles) {
          if (m.extra && "Missile" in m.extra) {
            const data = (m.extra as { Missile: { detected_by_radar: boolean; detected_by_glow: boolean } }).Missile;
            if (data.detected_by_radar) radarCount++;
            else if (data.detected_by_glow) glowCount++;
          }
        }
        const breakdown = glowCount > 0 ? ` (${radarCount}R/${glowCount}G)` : "";
        this.phaseText.text = `CONTACTS: ${missiles.length}${breakdown}`;
      } else {
        this.phaseText.text = "";
      }
    } else if (snapshot.phase === "WaveResult") {
      this.phaseText.text = "WAVE COMPLETE \u2014 ENTER TO CONTINUE";
    } else if (snapshot.phase === "Strategic") {
      this.phaseText.text = "CLICK OR ENTER TO START NEXT WAVE";
    } else {
      this.phaseText.text = snapshot.phase;
    }

    // Weather indicator
    if (snapshot.weather && snapshot.weather !== "Clear") {
      const windDir = (snapshot.wind_x ?? 0) > 0 ? ">>" : "<<";
      const windSpeed = Math.abs(snapshot.wind_x ?? 0).toFixed(0);
      this.weatherText.text = `${snapshot.weather.toUpperCase()} \u2014 WIND: ${windSpeed}m/s ${windDir}`;
      this.weatherText.style.fill = snapshot.weather === "Severe" ? HOT_PINK : SOLAR_YELLOW;
    } else {
      this.weatherText.text = "";
    }
  }

  updateResources(resources: number) {
    this.resourcesText.text = `$${resources}`;
  }

  updateBatterySelection(batteryId: number, ammo: number, maxAmmo: number, typeName?: string) {
    const tn = typeName ?? "Standard";
    const abbrev = this.typeAbbrev(tn);
    this.batteryText.text = `BAT-${batteryId + 1} [${ammo}/${maxAmmo}] ${abbrev}`;
    const typeColor = TYPE_COLORS[tn] ?? NEON_CYAN;
    this.batteryText.style.fill = ammo > 0 ? typeColor : HOT_PINK;
  }

  private typeAbbrev(typeName: string): string {
    switch (typeName) {
      case "Sprint": return "SPR";
      case "Exoatmospheric": return "EXO";
      case "AreaDenial": return "ADN";
      default: return "STD";
    }
  }

  showWaveComplete(event: WaveCompleteEvent) {
    const totalMissiles = event.missiles_destroyed + event.missiles_impacted;
    const efficiency =
      event.interceptors_launched > 0
        ? Math.round(
            (event.missiles_destroyed / event.interceptors_launched) * 100
          )
        : 0;

    this.waveCompleteText.text = [
      `--- WAVE ${event.wave_number} COMPLETE ---`,
      ``,
      `Missiles Destroyed: ${event.missiles_destroyed}/${totalMissiles}`,
      `Missiles Impacted:  ${event.missiles_impacted}`,
      `Interceptors Used:  ${event.interceptors_launched}`,
      `Efficiency:         ${efficiency}%`,
      `Cities Remaining:   ${event.cities_remaining}`,
      ``,
      `Press ENTER or Click to Continue`,
    ].join("\n");

    this.waveCompleteContainer.visible = true;
  }

  hideWaveComplete() {
    this.waveCompleteContainer.visible = false;
  }

  updateMuteState(muted: boolean) {
    if (muted) {
      this.weatherText.text = this.weatherText.text
        ? `${this.weatherText.text}  [MUTED]`
        : "[MUTED]";
    }
  }
}
