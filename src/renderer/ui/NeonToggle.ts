import { Container, Graphics } from "pixi.js";
import { PANEL_BORDER, DIM_TEXT } from "./Theme";

export interface NeonToggleOptions {
  width?: number;
  height?: number;
  value?: boolean;
  color?: number;
  onChange?: (value: boolean) => void;
}

export class NeonToggle extends Container {
  private track: Graphics;
  private thumbContainer: Container;
  private thumbGlow: Graphics;
  private thumb: Graphics;
  private _value: boolean;
  private _color: number;
  private toggleWidth: number;
  private toggleHeight: number;
  private thumbRadius: number;
  private targetThumbX: number;
  private currentThumbX: number;
  private onChange: ((value: boolean) => void) | null;

  constructor(options: NeonToggleOptions) {
    super();

    this.toggleWidth = options.width ?? 48;
    this.toggleHeight = options.height ?? 24;
    this._value = options.value ?? false;
    this._color = options.color ?? 0x39ff14;
    this.onChange = options.onChange ?? null;
    this.thumbRadius = (this.toggleHeight - 6) / 2;

    // Track
    this.track = new Graphics();
    this.addChild(this.track);

    // Thumb container for smooth animation
    this.thumbContainer = new Container();
    this.addChild(this.thumbContainer);

    this.thumbGlow = new Graphics();
    this.thumbContainer.addChild(this.thumbGlow);

    this.thumb = new Graphics();
    this.thumbContainer.addChild(this.thumb);

    const thumbOff = this.toggleHeight / 2;
    const thumbOn = this.toggleWidth - this.toggleHeight / 2;
    this.targetThumbX = this._value ? thumbOn : thumbOff;
    this.currentThumbX = this.targetThumbX;

    this.draw();
    this.setupInteraction();
  }

  private setupInteraction() {
    this.eventMode = "static";
    this.cursor = "pointer";
    this.hitArea = { contains: (x: number, y: number) => {
      return x >= -4 && x <= this.toggleWidth + 4
        && y >= -4 && y <= this.toggleHeight + 4;
    }};

    this.on("pointerdown", () => {
      this._value = !this._value;
      const thumbOff = this.toggleHeight / 2;
      const thumbOn = this.toggleWidth - this.toggleHeight / 2;
      this.targetThumbX = this._value ? thumbOn : thumbOff;
      this.draw();
      this.onChange?.(this._value);
    });
  }

  private draw() {
    const w = this.toggleWidth;
    const h = this.toggleHeight;
    const r = h / 2;
    const c = this._color;

    // Track
    this.track.clear();
    this.track.roundRect(0, 0, w, h, r);

    if (this._value) {
      this.track.fill({ color: c, alpha: 0.25 });
      this.track.roundRect(0, 0, w, h, r);
      this.track.stroke({ width: 2, color: c, alpha: 0.8 });
    } else {
      this.track.fill({ color: PANEL_BORDER, alpha: 0.6 });
      this.track.roundRect(0, 0, w, h, r);
      this.track.stroke({ width: 1, color: DIM_TEXT, alpha: 0.4 });
    }

    // Thumb position
    const tx = this.currentThumbX;
    const ty = h / 2;

    // Thumb glow
    this.thumbGlow.clear();
    if (this._value) {
      this.thumbGlow.circle(0, 0, this.thumbRadius + 3);
      this.thumbGlow.fill({ color: c, alpha: 0.15 });
    }

    // Thumb
    this.thumb.clear();
    this.thumb.circle(0, 0, this.thumbRadius);
    this.thumb.fill({ color: this._value ? c : DIM_TEXT, alpha: 0.9 });
    this.thumb.circle(0, 0, this.thumbRadius * 0.45);
    this.thumb.fill({ color: 0xffffff, alpha: this._value ? 0.7 : 0.3 });

    this.thumbContainer.x = tx;
    this.thumbContainer.y = ty;
  }

  tick(dt: number) {
    // Smooth thumb slide
    const diff = this.targetThumbX - this.currentThumbX;
    if (Math.abs(diff) > 0.5) {
      this.currentThumbX += diff * Math.min(dt * 0.2, 1);
      this.draw();
    }
  }

  get value(): boolean {
    return this._value;
  }

  set value(v: boolean) {
    this._value = v;
    const thumbOff = this.toggleHeight / 2;
    const thumbOn = this.toggleWidth - this.toggleHeight / 2;
    this.targetThumbX = v ? thumbOn : thumbOff;
    this.currentThumbX = this.targetThumbX; // Snap (no animation for external set)
    this.draw();
  }
}
