import clsx from "clsx";
import styles from "../../styles/Controls.module.css";

type ButtonVariant = "primary" | "secondary" | "danger";
type ButtonSize = "lg" | "md" | "sm";

interface NeonButtonProps {
  label: string;
  onClick?: () => void;
  disabled?: boolean;
  variant?: ButtonVariant;
  size?: ButtonSize;
  fullWidth?: boolean;
}

export function NeonButton({
  label,
  onClick,
  disabled = false,
  variant = "primary",
  size = "md",
  fullWidth = false,
}: NeonButtonProps) {
  return (
    <button
      type="button"
      className={clsx(
        styles.button,
        styles[`variant-${variant}`],
        styles[`size-${size}`],
        fullWidth && styles.fullWidth
      )}
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
    >
      <span>{label}</span>
    </button>
  );
}
