<script setup lang="ts">
import {
  CAPTURE_FPS,
  file_to_stream,
} from "@/Components/Character/InventoryScanner/FramePassing";
import { queue_ocr, start_ocr, stop_ocr, take_ocr_results } from "./OcrRelay";
import { has_progress, reset_scan, take_edits } from "./ScanStore";
import { useRosterStore } from "@/Stores/RosterConfig";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { storeToRefs } from "pinia";
import { ref, computed, onMounted, onUnmounted, toRaw, watch } from "vue";
import { useIntervalFn } from "@vueuse/core";
import { getModel, getScannerConfig, ScanResult } from "./LoadStorage";
import {
  auto_select_resolution,
  game_resolution,
  RESOLUTIONS_16_9,
  RESOLUTIONS_21_9,
} from "./Resolution";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const props = defineProps<{
  process_result: (result: ScanResult) => void;
}>();
const status = defineModel<"idle" | "capturing">("status", { default: "idle" });

const roster_store = useRosterStore();
const { roster_config } = storeToRefs(roster_store);

const bundle = ref(runtime.cropper);
// the stream outlives this component so permission is only asked once
const source = runtime.frame_source;

const error = ref<string | null>(null);

const resolutions = computed(() =>
  game_resolution.ultrawide ? RESOLUTIONS_21_9 : RESOLUTIONS_16_9,
);
const selected_resolution = computed({
  get: () => `${game_resolution.width}x${game_resolution.height}`,
  set: (value) => {
    [game_resolution.width, game_resolution.height] = value
      .split("x")
      .map(Number);
  },
});

// templates and remembered slots are scale-specific, so start over
function restart_scanner() {
  pause_cropper();
  bundle.value?.cancel_and_clear_prev_result();
  start_scanner();
}

async function start_capture() {
  try {
    error.value = null;
    const s = await navigator.mediaDevices.getDisplayMedia({
      video: {
        frameRate: { max: CAPTURE_FPS },
        // width: { max: 1280 },
        // height: { max: 720 },
      },
      audio: false,
    });
    const settings = s.getVideoTracks()[0].getSettings();
    begin(s, settings.width, settings.height);
  } catch (e: unknown) {
    if (
      e instanceof Error &&
      e.name !== "NotAllowedError" &&
      e.name !== "InvalidStateError"
    ) {
      error.value = e.message;
    }
  }
}

async function start_upload(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files[0];
  input.value = "";
  if (!file) return;

  error.value = null;
  const fake = await file_to_stream(file);
  begin(fake.stream, fake.width, fake.height, fake.cleanup);
}

function begin(
  s: MediaStream,
  width: number,
  height: number,
  cleanup: (() => void) | null = null,
) {
  stop_capture();
  source.start(s, width, height, cleanup);
  auto_select_resolution(width, height);
  attach();
  start_scanner();
}

// follow the store's stream in this component
function attach() {
  status.value = "capturing";
  source.stream
    .getVideoTracks()[0]
    .addEventListener("ended", stop_capture, { once: true });
}

async function start_scanner() {
  pause_cropper();
  // the new worker knows nothing of what the grid still shows
  reset_scan();
  runtime.ensure_cropper();
  bundle.value = runtime.cropper;

  const { width, height } = source.size;
  const [config, model] = await Promise.all([getScannerConfig(), getModel()]);
  // the state is built once here and then stays inside the worker's wasm
  const new_scanner_state = {
    screen_info: {
      game_width: game_resolution.width,
      game_height: game_resolution.height,
      forced_21_9: game_resolution.forced_21_9,
    },
    buffer: { width, height, size: width * height * 4 },
    config: toRaw(config),
  };
  // the model only goes to the OCR worker
  start_ocr(toRaw(model));
  bundle.value.debounced_start(
    WasmOp.Reserve,
    new_scanner_state,
    toggle_cropper,
    0,
    false,
  );
}

function stop_capture() {
  pause_cropper();
  status.value = "idle";
  if (!source.stream) return;
  // terminating the worker frees the frame buffer and the Rust statics with it
  bundle.value?.cancel();
  stop_ocr();
  source.stop();
}

const cropper_running = ref(false);
// bumped whenever the loop is stopped, so a loop waiting on a frame knows to quit
let loop_id = 0;

function pause_cropper() {
  loop_id++;
  cropper_running.value = false;
}

function toggle_cropper() {
  if (cropper_running.value) {
    pause_cropper();
  } else {
    cropper_running.value = true;
    cropper_loop(++loop_id, true);
  }
}
// frames handed to the scanner within the last second
let sent_at: number[] = [];
const sent_fps = ref(0);
useIntervalFn(() => {
  const now = performance.now();
  sent_at = sent_at.filter((at) => now - at < 1000);
  sent_fps.value = sent_at.length;
}, 250);

// per-scan timings in ms, kept on window for profiling
const scan_timings: any[] = ((globalThis as any).__scan_timings ??= []);

// `full` asks for every image again, since whoever shows the results may have missed some
async function cropper_loop(id: number, full = false) {
  const loop_start = performance.now();
  const frame = await source.read();
  const read = performance.now() - loop_start;
  const frame_time = frame?.timestamp;
  if (id !== loop_id) {
    frame?.close();
    return;
  }
  if (!frame) {
    console.log("no more cropper");
    cropper_running.value = false;
    return;
  }

  sent_at.push(performance.now());
  // one result per op, a late reply from an earlier op must not fork the loop
  let done = false;
  bundle.value.debounced_start(
    WasmOp.Cropper,
    {
      frame,
      full,
      ocr_results: take_ocr_results(),
      edits: take_edits(),
    },
    (result: ScanResult, timings) => {
      // the scanner waits for these texts whether or not this reply is used
      queue_ocr(result.ocr_jobs);
      if (done || id !== loop_id || result.full !== full) return;
      done = true;
      const result_start = performance.now();
      // the next frame goes out before any UI work
      cropper_loop(id);
      props.process_result(result);
      scan_timings.push({
        ...timings,
        // same clock as the OCR records, so the two can be lined up
        at: result_start,
        read,
        frame_time,
        process_result: performance.now() - result_start,
        total: result_start - loop_start,
      });
      if (scan_timings.length > 5000) scan_timings.shift();
    },
    0,
    false,
  );
}

// Let a parent trigger capture imperatively if it needs to (e.g. reuse
// elsewhere without the buttons below).
defineExpose({ start_capture, stop_capture });

onMounted(() => {
  if (!source.stream) return;
  attach();
  if (bundle.value?.result?.buffer?.pointer == null) {
    start_scanner();
  } else {
    toggle_cropper();
  }
});
// the stream and worker stay alive in the store
onUnmounted(pause_cropper);

// CharView clears the scanner state on character change
watch(
  () => roster_config.value.active_profile_index,
  () => source.stream && start_scanner(),
);
</script>

<template>
  <div class="card-shell card-body flex flex-col gap-4">
    <div class="flex items-center gap-3">
      <span class="text-sm font-semibold">Screen Capture</span>
      <span
        class="rounded-full px-2 py-0.5 text-xs"
        :class="{
          'bg-green-500/20 text-green-400': status === 'capturing',
          'bg-zinc-500/20 text-zinc-400': status === 'idle',
        }"
      >
        {{ status === "capturing" ? "● Live" : "Idle" }}
      </span>
    </div>

    <p
      v-if="error"
      class="rounded bg-red-500/10 px-3 py-2 text-xs text-red-400"
    >
      {{ error }}
    </p>

    <div v-if="status === 'capturing'" class="flex items-center gap-2">
      <span> Game resolution: </span>
      <select
        v-model="selected_resolution"
        class="selector"
        aria-label="Game resolution"
        @change="restart_scanner"
      >
        <option v-for="[w, h] in resolutions" :key="`${w}x${h}`">
          {{ w }}x{{ h }}
        </option>
      </select>
      <label class="flex items-center gap-1">
        <input
          type="checkbox"
          v-model="game_resolution.forced_21_9"
          :disabled="game_resolution.ultrawide"
          @change="restart_scanner"
        />
        Forced 21:9
      </label>
    </div>

    <div class="flex gap-2">
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status === 'capturing'"
        @click="start_capture"
      >
        Share screen
      </button>
      <label
        class="cursor-pointer rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500"
      >
        Upload image / video
        <input
          type="file"
          accept="image/*,video/*"
          aria-label="Upload image or video"
          class="hidden"
          @change="start_upload"
        />
      </label>
      <button
        class="rounded-md bg-zinc-700 px-3 py-1.5 text-sm transition-colors hover:bg-zinc-600 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status !== 'capturing'"
        @click="stop_capture"
      >
        Stop
      </button>
      <span
        v-if="status === 'capturing'"
        class="self-center text-sm text-(--text-muted)"
        aria-label="Frames scanned per second"
      >
        {{ sent_fps }} fps scanned
      </span>
      <span
        v-if="status === 'idle' && has_progress"
        class="self-center text-sm text-(--warning)"
      >
        Starting a capture wipes the current scan.
      </span>
    </div>
    <button @click="toggle_cropper" class="generic-button">
      {{ cropper_running ? "stop cropper" : "start scropper" }}
    </button>
  </div>
</template>
