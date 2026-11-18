import { reactive } from "vue";

// Detected by the scanner unless given; value is null while auto. Same shape as Rust's Detected.
// Shared by all Stream instances. Saved in this browser on apply (`save_settings`), apart from
// the config since they belong to the machine, not the roster.
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

const SETTINGS_KEY = "scanner_settings";
const FONT_KEY = "scanner_font";

const DEFAULT_SETTINGS = () => ({
  forced_21_9: { value: null, manual: false } as Detected<boolean>,
  brightness: { value: null, manual: false } as Detected<number>,
  font: "English" as (typeof NUMBER_FONTS)[number] | "Custom",
  // the picked file's name, for a custom font
  font_file: null as string | null,
});

const to_base64 = (bytes: Uint8Array) => {
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000)
    out += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(out);
};
const from_base64 = (text: string) =>
  Uint8Array.from(atob(text), (c) => c.charCodeAt(0));

function load_settings() {
  const out = DEFAULT_SETTINGS();
  try {
    Object.assign(out, JSON.parse(localStorage.getItem(SETTINGS_KEY)));
    if (out.font === "Custom") {
      const font = localStorage.getItem(FONT_KEY);
      if (font) custom_font.bytes = from_base64(font);
    }
  } catch {}
  // a custom font that could not be kept is picked again
  if (out.font === "Custom" && !custom_font.bytes)
    Object.assign(out, { font: "English", font_file: null });
  return out;
}

// the picked file itself, kept apart since it is not a setting to compare or copy
export const custom_font: { bytes: Uint8Array | null } = { bytes: null };

export const scan_settings = reactive(load_settings());

// anything off auto or the default font
export const settings_overridden = (s: typeof scan_settings) =>
  s.forced_21_9.manual || s.brightness.manual || s.font !== "English";

// A font too big for storage is used this session only.
export function save_settings(font_changed: boolean) {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(scan_settings));
    if (scan_settings.font !== "Custom" || font_changed)
      localStorage.removeItem(FONT_KEY);
    if (scan_settings.font === "Custom" && font_changed)
      localStorage.setItem(FONT_KEY, to_base64(custom_font.bytes));
  } catch (e) {
    console.error("could not save scanner settings", e);
  }
}
