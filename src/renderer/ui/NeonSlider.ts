import { Container, Graphics, Text, TextStyle, FederatedPointerEvent } from "pixi.js";
import { PANEL_BORDER, FONT_FAMILY } from "./Theme";

export interface NeonSliderOptions {
  width?: number;
  height?: number;
  min?: number;
  max?: number;
  value?: number;
  color?: number;
  showLabel?: boolean;
  onChange?: (value: number) => void;
}

export class NeonSlider extends Container {
  private track: Graphics;
  private fill: Graphics;
  private thumb: Graphics;
  private thumbGlow: Graphics;
  private valueLabel: Text | null = null;
  private _value: number;
  private _min: number;
  private _max: number;
  private _color: number;
  private _dragging: boolean = false;
  private sliderWidth: number;
  private sliderHeight: number;
  private thumbRadius: number = 8;
  private onChange: ((value: number) => void) | null;

  constructor(options: NeonSliderOptions) {
    super();

    this.sliderWidth = options.width ?? 200;
    this.sliderHeight = options.height ?? 6;
    this._min = options.min ?? 0;
    this._max = options.max ?? 1;
    this._value = options.value ?? 0.5;
    this._color = options.color ?? 0x00ffff;
    this.onChange = options.onChange ?? null;

    // Track background
    this.track = new Graphics();
    this.addChild(this.track);

    // Fill
    this.fill = new Graphics();
    this.addChild(this.fill);

    // Thumb glow (behind thumb)
    this.thumbGlow = new Graphics();
    this.addChild(this.thumbGlow);

    // Thumb
    this.thumb = new Graphics();
    this.addChild(this.thumb);

    // Value label
    if (options.showLabel !== false) {
      this.valueLabel = new Text({
        text: "",
        style: new TextStyle({
          fontFamily: FONT_FAMILY,
          fontSize: 12,
          fill: this._color,
        }),
      });
      this.valueLabel.anchor.set(0.5, 0);
      this.valueLabel.y = this.sliderHeight / 2 + this.thumbRadius + 4;
      this.addChild(this.valueLabel);
    }

    this.draw();
    this.setupInteraction();
  }

  private setupInteraction() {
    this.eventMode = "static";
    this.cursor = "pointer";
    // Hit area covers full slider + thumb area
    this.hitArea = { contains: (x: number, y: number) => {
      const cy = this.sliderHeight / 2;
      return x >= -this.thumbRadius && x <= this.sliderWidth + this.thumbRadius
        && y >= cy - this.thumbRadius - 4 && y <= cy + this.thumbRadius + 4;
    }};

    this.on("pointerdown", (e: FederatedPointerEvent) => {
      this._dragging = true;
      this.updateFromPointer(e);
    });
    this.on("pointermove", (e: FederatedPointerEvent) => {
      if (this._dragging) {
        this.updateFromPointer(e);
      }
    });
    this.on("pointerup", () => { this._dragging = false; });
    this.on("pointerupoutside", () => { this._dragging = false; });
  }

  private updateFromPointer(e: FederatedPointerEvent) {
    const local = this.toLocal(e.global);
    const ratio = Math.max(0, Math.min(1, local.x / this.sliderWidth));
    this._value = this._min + ratio * (this._max - this._min);
    this.draw();
    this.onChange?.(this._value);
  }

  private draw() {
    const w = this.sliderWidth;
    const h = this.sliderHeight;
    const cy = h / 2;
    const ratio = (this._value - this._min) / (this._max - this._min);
    const fillW = w * ratio;
    const thumbX = fillW;

    // Track
    this.track.clear();
    this.track.roundRect(0, 0, w, h, h / 2);
    this.track.fill({ color: PANEL_BORDER, alpha: 0.8 });

    // Fill
    this.fill.clear();
    if (fillW > 1) {
      this.fill.roundRect(0, 0, fillW, h, h / 2);
      this.fill.fill({ color: this._color, alpha: 0.5 });
    }

    // Thumb glow
    this.thumbGlow.clear();
    this.thumbGlow.circle(thumbX, cy, this.thumbRadius + 4);
    this.thumbGlow.fill({ color: this._color, alpha: this._dragging ? 0.2 : 0.1 });

    // Thumb
    this.thumb.clear();
    this.thumb.circle(thumbX, cy, this.thumbRadius);
    this.thumb.fill({ color: this._color, alpha: 0.9 });
    this.thumb.circle(thumbX, cy, this.thumbRadius * 0.5);
    this.thumb.fill({ color: 0xffffff, alpha: 0.8 });

    // Value label
    if (this.valueLabel) {
      this.valueLabel.text = `${Math.round(this._value * 100)}%`;
      this.valueLabel.x = thumbX;
    }
  }

  get value(): number {
    return this._value;
  }

  set value(v: number) {
    this._value = Math.max(this._min, Math.min(this._max, v));
    this.draw();
  }

  tick(_dt: number) {
    // Could add subtle thumb pulse here if desired
  }
}
