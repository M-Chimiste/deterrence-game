import styles from "../../styles/Controls.module.css";

interface NeonToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function NeonToggle({
  label,
  checked,
  onChange,
  disabled = false,
}: NeonToggleProps) {
  return (
    <label className={styles.toggle}>
      <span className={styles.toggleLabel}>{label}</span>
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className={styles.toggleTrack} aria-hidden />
    </label>
  );
}
