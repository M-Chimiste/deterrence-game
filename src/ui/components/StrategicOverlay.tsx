import { useMemo } from "react";
import type { AvailableAction, RegionSnapshot } from "../../types/campaign";
import { useGameStore } from "../store";
import { handleStrategicAction, playUiClick } from "../gameActions";
import { NeonButton } from "./controls/NeonButton";
import styles from "../styles/StrategicOverlay.module.css";

function formatAction(action: AvailableAction, regions: RegionSnapshot[]) {
  if (action === "StartWave") return { label: "START WAVE", cost: 0, isStart: true };
  if ("ExpandRegion" in action) {
    const { region_id, cost } = action.ExpandRegion;
    const name = regions.find((r) => r.id === region_id)?.name ?? `Region ${region_id}`;
    return { label: `EXPAND: ${name} ($${cost})`, cost, isStart: false };
  }
  if ("PlaceBattery" in action) {
    const { region_id, cost } = action.PlaceBattery;
    const name = regions.find((r) => r.id === region_id)?.name ?? `Region ${region_id}`;
    return { label: `PLACE BATTERY: ${name} ($${cost})`, cost, isStart: false };
  }
  if ("RestockAllBatteries" in action) {
    const { count, cost } = action.RestockAllBatteries;
    const label = count === 1
      ? `RESTOCK BATTERIES ($${cost})`
      : `RESTOCK ALL BATTERIES x${count} ($${cost})`;
    return { label, cost, isStart: false };
  }
  if ("RepairCity" in action) {
    const { region_id, cost } = action.RepairCity;
    const name = regions.find((r) => r.id === region_id)?.name ?? `Region ${region_id}`;
    return { label: `REPAIR CITY: ${name} ($${cost})`, cost, isStart: false };
  }
  if ("UnlockInterceptor" in action) {
    const { interceptor_type, cost } = action.UnlockInterceptor;
    return { label: `UNLOCK: ${interceptor_type} ($${cost})`, cost, isStart: false };
  }
  if ("UpgradeInterceptor" in action) {
    const { interceptor_type, axis, cost, current_level } = action.UpgradeInterceptor;
    return {
      label: `UPGRADE: ${interceptor_type} ${axis} Lv${current_level + 1} ($${cost})`,
      cost,
      isStart: false,
    };
  }
  return { label: "UNKNOWN ACTION", cost: 0, isStart: false };
}

export function StrategicOverlay() {
  const campaign = useGameStore((state) => state.campaign);
  const hoveredRegionId = useGameStore((state) => state.hoveredRegionId);

  const hoveredRegion = useMemo(() => {
    if (!campaign || hoveredRegionId == null) return null;
    return campaign.regions.find((r) => r.id === hoveredRegionId) ?? null;
  }, [campaign, hoveredRegionId]);

  if (!campaign) {
    return (
      <div className={styles.loading}>
        <div>Awaiting campaign data...</div>
      </div>
    );
  }

  const ownedRegions = campaign.regions.filter((r) => r.owned);
  const totalCities = ownedRegions.reduce((sum, r) => sum + r.cities.length, 0);
  const totalBatteries = ownedRegions.reduce(
    (sum, r) => sum + r.battery_slots.filter((b) => b.occupied).length,
    0
  );
  const emptySlots = ownedRegions.reduce(
    (sum, r) => sum + r.battery_slots.filter((b) => !b.occupied).length,
    0
  );

  return (
    <div className={styles.overlayRoot}>
      <div className={styles.header}>
        <div className={styles.resources}>RESOURCES: ${campaign.resources}</div>
        <div className={styles.title}>STRATEGIC COMMAND</div>
        <div className={styles.wave}>
          {campaign.wave_number > 0
            ? `WAVES SURVIVED: ${campaign.wave_number}`
            : "WAVE: FIRST DEPLOYMENT"}
        </div>
      </div>

      {campaign.wave_income != null && (
        <div className={styles.income}>
          +{campaign.wave_income} INCOME FROM SURVIVING CITIES
        </div>
      )}

      <div className={styles.panel}>
        <div className={styles.panelHeader}>AVAILABLE ACTIONS</div>
        <div className={styles.actions}>
          {campaign.available_actions.map((action, index) => {
            const formatted = formatAction(action, campaign.regions);
            const affordable = formatted.cost === 0 || campaign.resources >= formatted.cost;
            return (
              <NeonButton
                key={`${formatted.label}-${index}`}
                label={formatted.label}
                size={formatted.isStart ? "md" : "sm"}
                variant={
                  formatted.isStart ? "primary" : affordable ? "secondary" : "danger"
                }
                disabled={!affordable}
                fullWidth
                onClick={() => {
                  playUiClick();
                  handleStrategicAction(action);
                }}
              />
            );
          })}
        </div>
      </div>

      <div className={styles.info}>
        <div className={styles.intel}>
          INTEL: {ownedRegions.length} regions secured | {totalCities} cities |{" "}
          {totalBatteries} batteries deployed | {emptySlots} open slots | ENTER=Start Wave
          | F5=Quick Save | F9=Quick Load
        </div>

        {hoveredRegion && (
          <div className={styles.regionPanel}>
            <div className={styles.regionTitle}>{hoveredRegion.name.toUpperCase()}</div>
            <div className={styles.regionMeta}>
              Terrain: {hoveredRegion.terrain} | Cities: {hoveredRegion.cities.length} | Slots:{" "}
              {hoveredRegion.battery_slots.filter((b) => b.occupied).length}/
              {hoveredRegion.battery_slots.length}
            </div>
            {hoveredRegion.owned ? (
              <div className={styles.regionDetails}>
                {hoveredRegion.cities.map((city, idx) => {
                  const ratio = city.health / city.max_health;
                  const tone = ratio > 0.6 ? "good" : ratio > 0.3 ? "warn" : "danger";
                  return (
                    <div key={`city-${idx}`} data-tone={tone}>
                      City {idx + 1}: {Math.round(city.health)}/{city.max_health}
                    </div>
                  );
                })}
              </div>
            ) : hoveredRegion.expandable ? (
              <div className={styles.regionDetails}>
                Expansion Cost: ${hoveredRegion.expansion_cost}
              </div>
            ) : (
              <div className={styles.regionDetails}>Unknown region. Expand to reveal.</div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
