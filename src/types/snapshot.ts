export type EntityType = "Missile" | "Interceptor" | "Shockwave" | "City" | "Battery";

export interface ShockwaveExtra {
  Shockwave: {
    radius: number;
    max_radius: number;
  };
}

export interface CityExtra {
  City: {
    health: number;
    max_health: number;
  };
}

export interface BatteryExtra {
  Battery: {
    ammo: number;
    max_ammo: number;
  };
}

export interface InterceptorExtra {
  Interceptor: {
    burn_remaining: number;
    burn_time: number;
    interceptor_type: string;
  };
}

export interface MissileExtra {
  Missile: {
    is_mirv: boolean;
    detected_by_radar: boolean;
    detected_by_glow: boolean;
  };
}

export type EntityExtra = ShockwaveExtra | CityExtra | BatteryExtra | InterceptorExtra | MissileExtra;

export interface EntitySnapshot {
  id: number;
  entity_type: EntityType;
  x: number;
  y: number;
  rotation: number;
  vx: number;
  vy: number;
  extra: EntityExtra | null;
}

export interface StateSnapshot {
  tick: number;
  wave_number: number;
  phase: string;
  entities: EntitySnapshot[];
  weather?: string;
  wind_x?: number;
}
