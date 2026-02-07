import styles from "../../styles/Controls.module.css";

interface NeonSliderProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  onChange: (value: number) => void;
}

export function NeonSlider({
  label,
  value,
  min = 0,
  max = 1,
  step = 0.05,
  onChange,
}: NeonSliderProps) {
  return (
    <label className={styles.slider}>
      <span className={styles.sliderLabel}>{label}</span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}
