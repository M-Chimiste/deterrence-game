/**
 * TypeScript mirrors of Rust state types.
 * These MUST be kept in sync with deterrence-core/src/state.rs.
 */

export interface GameStateSnapshot {
  time: SimTime;
  phase: GamePhase;
  doctrine: DoctrineMode;
  tracks: TrackView[];
  engagements: EngagementView[];
  own_ship: OwnShipView;
  radar: RadarView;
  vls: VlsView;
  illuminators: IlluminatorView[];
  alerts: Alert[];
  audio_events: AudioEvent[];
  score: ScoreView;
}

export interface SimTime {
  tick: number;
  elapsed_secs: number;
}

export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface TrackView {
  track_number: number;
  position: Position;
  bearing: number;
  range: number;
  altitude: number;
  speed: number;
  heading: number;
  classification: Classification;
  iff_status: IffStatus;
  quality: number;
  hooked: boolean;
  history: Position[];
}

export interface EngagementView {
  engagement_id: number;
  track_number: number;
  phase: EngagementPhase;
  weapon_type: WeaponType;
  pk: number;
  veto_remaining_secs: number;
  veto_total_secs: number;
  illuminator_channel: number | null;
  time_to_intercept: number;
  result: InterceptResult | null;
}

export interface OwnShipView {
  position: Position;
}

export interface RadarView {
  mode: RadarMode;
  energy_total: number;
  energy_search: number;
  energy_track: number;
  sweep_angle: number;
  sector_center: number;
  sector_width: number;
  active_track_count: number;
}

export interface VlsView {
  cells: CellStatus[];
  ready_standard: number;
  ready_extended_range: number;
  ready_point_defense: number;
  total_ready: number;
  total_capacity: number;
}

export interface IlluminatorView {
  channel_id: number;
  status: IlluminatorStatus;
  assigned_engagement: number | null;
  queue_depth: number;
}

export interface ScoreView {
  threats_killed: number;
  threats_total: number;
  interceptors_fired: number;
  assets_protected: boolean;
}

export interface Alert {
  level: AlertLevel;
  message: string;
  tick: number;
}

// --- Enums (as string unions matching Rust serde serialization) ---

export type GamePhase =
  | "MainMenu"
  | "MissionBriefing"
  | "Active"
  | "Paused"
  | "MissionComplete";

export type DoctrineMode = "Manual" | "AutoSpecial" | "AutoComposite";

export type Classification =
  | "Unknown"
  | "Pending"
  | "AssumedFriend"
  | "Friend"
  | "Neutral"
  | "Suspect"
  | "Hostile";

export type IffStatus =
  | "NoResponse"
  | "FriendlyResponse"
  | "NoValidResponse"
  | "Suspicious";

export type RadarMode = "Search" | "TrackWhileScan" | "BurnThrough";

export type WeaponType = "Standard" | "ExtendedRange" | "PointDefense";

export type EngagementPhase =
  | "SolutionCalc"
  | "Ready"
  | "Launched"
  | "Midcourse"
  | "Terminal"
  | "Intercept"
  | "Complete"
  | "Aborted";

export type IlluminatorStatus = "Idle" | "Active" | "TimeSharing";

export type InterceptResult = "Hit" | "Miss";

export type AlertLevel = "Info" | "Warning" | "Critical";

export type CellStatus =
  | { Ready: WeaponType }
  | { Assigned: WeaponType }
  | "Expended"
  | "Empty";

export type AudioEvent =
  | { type: "NewContact"; bearing: number; track_number: number }
  | { type: "ContactLost"; track_number: number }
  | {
      type: "ThreatEvaluated";
      track_number: number;
      classification: Classification;
    }
  | { type: "VetoClockStart"; engagement_id: number; duration_secs: number }
  | {
      type: "VetoClockWarning";
      engagement_id: number;
      remaining_secs: number;
    }
  | { type: "BirdAway"; weapon_type: WeaponType }
  | { type: "Splash"; result: InterceptResult; track_number: number }
  | { type: "VampireImpact"; bearing: number };
