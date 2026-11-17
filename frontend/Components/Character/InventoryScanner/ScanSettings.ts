import { reactive } from "vue";

// Detected by the scanner unless given; value is null while auto. Same shape as Rust's Detected.
// Shared by all Stream instances, never persisted.
export interface Detected<T> {
  value: T | null;
  manual: boolean;
}

// The fonts the game ships, cut down to digits (scanner_setup/game_files/fonts.py), one of which
// the slot numbers are read by; or a file the user picks.
export const NUMBER_FONTS = [
  "English",
  "Korean",
  "Russian",
  "Japanese",
  "Chinese",
  "Taiwanese",
] as const;

export const scan_settings = reactive({
  forced_21_9: { value: null, manual: false } as Detected<boolean>,
  brightness: { value: null, manual: false } as Detected<number>,
  font: "English" as (typeof NUMBER_FONTS)[number] | "Custom",
  // the picked file's name, for a custom font
  font_file: null as string | null,
});

// the picked file itself, kept apart since it is not a setting to compare or copy
export const custom_font: { bytes: Uint8Array | null } = { bytes: null };
