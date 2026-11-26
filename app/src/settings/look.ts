export type Theme = "system" | "light" | "dark";
export type PickerStyle = "bear" | "classic" | "sheet" | "compact";
export type PickerPosition = "pointer" | "center";
export type TextSize = "small" | "normal" | "large";
export type Density = "compact" | "normal" | "comfortable";

export type Look = {
  theme: Theme;
  high_contrast: boolean;
  text_size: TextSize;
  picker_style: PickerStyle;
  density: Density;
  picker_position: PickerPosition;
  /// The English family name; `null` is the system's own.
  font: string | null;
};

/// The picker's own measures, in `crates/linkunbound-shell/src/measure.rs`.
export const ROW: Record<Density, number> = { compact: 36, normal: 45, comfortable: 54 };
/// What `fonts_list` answers: «System» first, then every family by the name the person reads.
export type Fonts = { system: string; families: { name: string; label: string }[] };

export const CARD: Record<Density, number> = { compact: 44, normal: 52, comfortable: 60 };
