import { reactive } from "vue";

// Detected by the scanner unless given; value is null while auto. Same shape as Rust's Detected.
// Shared by all Stream instances, never persisted.
export interface Detected<T> {
  value: T | null;
  manual: boolean;
}

export const scan_settings = reactive({
  forced_21_9: { value: null, manual: false } as Detected<boolean>,
  brightness: { value: null, manual: false } as Detected<number>,
});
