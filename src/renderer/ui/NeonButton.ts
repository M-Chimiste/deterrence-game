import { Container, Graphics, Text, TextStyle } from "pixi.js";
import { PANEL_DARK, DIM_TEXT, FONT_FAMILY } from "./Theme";

export interface NeonButtonOptions {
  label: string;
  width?: number;
  height?: number;
  fontSize?: number;
  color?: number;
  disabled?: boolean;
  onClick?: () => void;
}

export class NeonButton extends Container {
  private bg: Graphics;
  private glow: Graphics;
  private labelText: Text;
  private _color: number;
  private _disabled: boolean;
  private _hovered: boolean = false;
  private _pressed: boolean = false;
  private animTimer: number = 0;
  private targetScale: number = 1.0;
  private btnWidth: number;
  private btnHeight: number;
  private _onClick: (() => void) | null;

  constructor(options: NeonButtonOptions) {
    super();

    const fontSize = options.fontSize ?? 18;
    this._color = options.color ?? 0x00ffff;
    this._disabled = options.disabled ?? false;
    this._onClick = options.onClick ?? null;
    this.btnHeight = options.height ?? 44;

    // Create label text to measure width
    this.labelText = new Text({
      text: options.label,
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize,
        fill: this._color,
        fontWeight: "bold",
      }),
    });
    this.labelText.anchor.set(0.5);

    this.btnWidth = options.width ?? Math.max(this.labelText.width + 40, 120);

    // Position label at center
    this.labelText.x = this.btnWidth / 2;
    this.labelText.y = this.btnHeight / 2;

    // Glow layer (behind bg)
    this.glow = new Graphics();
    this.addChild(this.glow);

    // Background + border
    this.bg = new Graphics();
    this.addChild(this.bg);

    this.addChild(this.labelText);

    // Set pivot to center for scale animations
    this.pivot.set(this.btnWidth / 2, this.btnHeight / 2);

    this.draw();
    this.setupInteraction();
  }

  private setupInteraction() {
    // Hit area is the full button rect
    this.eventMode = this._disabled ? "none" : "static";
    this.cursor = this._disabled ? "default" : "pointer";
    this.hitArea = { contains: (x: number, y: number) => {
      return x >= 0 && x <= this.btnWidth && y >= 0 && y <= this.btnHeight;
    }};

    this.on("pointerover", () => {
      if (this._disabled) return;
      this._hovered = true;
      this.targetScale = 1.02;
      this.draw();
    });
    this.on("pointerout", () => {
      this._hovered = false;
      this._pressed = false;
      this.targetScale = 1.0;
      this.draw();
    });
    this.on("pointerdown", () => {
      if (this._disabled) return;
      this._pressed = true;
      this.targetScale = 0.98;
      this.draw();
      this._onClick?.();
    });
    this.on("pointerup", () => {
      this._pressed = false;
      this.targetScale = this._hovered ? 1.02 : 1.0;
      this.draw();
    });
  }

  private draw() {
    const c = this._color;
    const w = this.btnWidth;
    const h = this.btnHeight;
    const r = 6; // corner radius

    // --- Glow layer ---
    this.glow.clear();
    if (!this._disabled) {
      const glowAlpha = this._hovered
        ? 0.15 + Math.sin(this.animTimer * 4.2) * 0.08
        : 0.06;

      // 3 concentric glow rects
      for (let i = 3; i >= 1; i--) {
        const offset = i * 2;
        this.glow.roundRect(-offset, -offset, w + offset * 2, h + offset * 2, r + offset);
        this.glow.fill({ color: c, alpha: glowAlpha * (1 - i * 0.25) });
      }
    }

    // --- Background ---
    this.bg.clear();

    if (this._disabled) {
      // Disabled state
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.fill({ color: PANEL_DARK, alpha: 0.5 });
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.stroke({ width: 1, color: DIM_TEXT, alpha: 0.3 });
      this.labelText.style.fill = DIM_TEXT;
      this.labelText.alpha = 0.5;
    } else if (this._pressed) {
      // Pressed state
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.fill({ color: c, alpha: 0.3 });
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.stroke({ width: 2, color: 0xffffff });
      this.labelText.style.fill = 0xffffff;
      this.labelText.alpha = 1;
    } else if (this._hovered) {
      // Hover state
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.fill({ color: c, alpha: 0.15 });
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.stroke({ width: 2, color: c });
      this.labelText.style.fill = 0xffffff;
      this.labelText.alpha = 1;
    } else {
      // Idle state
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.fill({ color: PANEL_DARK, alpha: 0.85 });
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.stroke({ width: 1, color: c, alpha: 0.6 });
      this.labelText.style.fill = c;
      this.labelText.alpha = 1;
    }
  }

  tick(dt: number) {
    this.animTimer += dt * 0.016; // Convert frame delta to seconds-ish

    // Smooth scale transition
    const current = this.scale.x;
    const diff = this.targetScale - current;
    if (Math.abs(diff) > 0.001) {
      const lerped = current + diff * Math.min(dt * 0.15, 1);
      this.scale.set(lerped);
    }

    // Redraw glow pulse when hovered
    if (this._hovered) {
      this.draw();
    }
  }

  setLabel(text: string) {
    this.labelText.text = text;
  }

  setDisabled(disabled: boolean) {
    this._disabled = disabled;
    this.eventMode = disabled ? "none" : "static";
    this.cursor = disabled ? "default" : "pointer";
    this._hovered = false;
    this._pressed = false;
    this.targetScale = 1.0;
    this.draw();
  }

  setColor(color: number) {
    this._color = color;
    this.draw();
  }

  setActive(active: boolean) {
    // For radio-group style: active = filled background
    if (active && !this._disabled) {
      this.bg.clear();
      const w = this.btnWidth;
      const h = this.btnHeight;
      const r = 6;
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.fill({ color: this._color, alpha: 0.25 });
      this.bg.roundRect(0, 0, w, h, r);
      this.bg.stroke({ width: 2, color: this._color });
      this.labelText.style.fill = 0xffffff;
    } else {
      this.draw();
    }
  }

  set onClick(fn: (() => void) | null) {
    this._onClick = fn;
  }
}
