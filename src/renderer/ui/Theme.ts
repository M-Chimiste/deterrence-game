// Neon Arcade Color Palette

// Primary UI colors
export const NEON_CYAN = 0x00ffff;
export const NEON_MAGENTA = 0xff00ff;
export const ELECTRIC_BLUE = 0x4488ff;
export const NEON_GREEN = 0x39ff14;
export const HOT_PINK = 0xff1493;
export const NEON_ORANGE = 0xff6600;
export const SOLAR_YELLOW = 0xffff00;

// Panel/background colors
export const PANEL_DARK = 0x0a0a1a;
export const PANEL_BORDER = 0x1a1a3a;
export const DIM_TEXT = 0x446688;
export const DEEP_BG = 0x050510;

// Interceptor type colors (saturated neon)
export const TYPE_COLORS: Record<string, number> = {
  Standard: NEON_GREEN,
  Sprint: NEON_CYAN,
  Exoatmospheric: NEON_MAGENTA,
  AreaDenial: NEON_ORANGE,
};

// Missile colors
export const MISSILE_BODY = HOT_PINK;
export const MISSILE_CORE = 0xffffff;
export const MIRV_BODY = 0xff0044;
export const MIRV_CORE = SOLAR_YELLOW;
export const GLOW_CONTACT = NEON_ORANGE;

// Explosion/particle colors
export const EXPLOSION_COLORS = [0xffffff, SOLAR_YELLOW, NEON_ORANGE, HOT_PINK];

// Trail dim variants (for gradient tail)
export const MISSILE_TRAIL_DIM = 0x660033;
export const TYPE_TRAIL_DIM: Record<string, number> = {
  Standard: 0x0a3300,
  Sprint: 0x003333,
  Exoatmospheric: 0x330033,
  AreaDenial: 0x331a00,
};

// UI font
export const FONT_FAMILY = "Courier New";
