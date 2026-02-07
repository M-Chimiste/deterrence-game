export interface PingResponse {
  message: string;
  tick: number;
}

export interface LaunchInterceptorCommand {
  battery_id: number;
  target_x: number;
  target_y: number;
}

export interface ArcPrediction {
  points: [number, number][];
  time_to_target: number;
  reaches_target: boolean;
}

export interface SaveMetadata {
  slot_name: string;
  wave_number: number;
  timestamp: number;
  resources: number;
}
