import { reactive } from "vue";

export const RESOLUTIONS_16_9: [number, number][] = [
  [1280, 720],
  [1280, 1024],
  [1366, 768],
  [1600, 900],
  [1680, 1050],
  [1760, 990],
  [1920, 1080],
  [2560, 1440],
  [3840, 2160],
  [7680, 4320],
];

export const RESOLUTIONS_21_9: [number, number][] = [
  [1680, 720],
  [2560, 1080],
  [3440, 1440],
  [3840, 1600],
  [5120, 2160],
];

// What the game runs at, picked by the user. Shared by all Stream instances, never persisted.
export const game_resolution = reactive({
  width: 1920,
  height: 1080,
  forced_21_9: false,
  ultrawide: false,
});

// guess from the capture size, the user can correct it
export function auto_select_resolution(width: number, height: number) {
  const ultrawide = width / height > 2.1;
  const distance = (r: [number, number]) =>
    Math.abs(r[0] - width) + Math.abs(r[1] - height);
  const closest = (ultrawide ? RESOLUTIONS_21_9 : RESOLUTIONS_16_9).reduce(
    (best, r) => (distance(r) < distance(best) ? r : best),
  );
  Object.assign(game_resolution, {
    width: closest[0],
    height: closest[1],
    forced_21_9: ultrawide,
    ultrawide,
  });
}
