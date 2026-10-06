<script setup lang="ts">
import {
  CAPTURE_FPS,
  file_to_stream,
} from "@/Components/Character/InventoryScanner/FramePassing";
import { queue_ocr, start_ocr, stop_ocr, take_ocr_results } from "./OcrRelay";
import { useRosterStore } from "@/Stores/RosterConfig";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { storeToRefs } from "pinia";
import { ref, computed, onMounted, onUnmounted, toRaw, watch } from "vue";
import {
  getModel,
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
  ScanResult,
} from "./LoadStorage";
import {
  auto_select_resolution,
  game_resolution,
  RESOLUTIONS_16_9,
  RESOLUTIONS_21_9,
} from "./Resolution";

const props = defineProps<{
  boxes?: ScaledPosition[];
  debugging: boolean;
  process_result?: (result: ScanResult) => void;
  should_start_cropper: boolean;
}>();
const status = defineModel<"idle" | "capturing">("status", { default: "idle" });

const roster_store = useRosterStore();
const { roster_config } = storeToRefs(roster_store);

const bundle = ref(roster_config.value.cropper_worker_bundle);
// the stream outlives this component so permission is only asked once
const source = roster_config.value.frame_source;

const video_ref = ref<HTMLVideoElement | null>(null);
const error = ref<string | null>(null);

// Track the real capture resolution so we can correctly place overlay
// boxes even if the stream isn't exactly 1280x720.
const video_natural = ref({ width: 1280, height: 720 });
function on_loaded_metadata() {
  if (video_ref.value) {
    video_natural.value = {
      width: video_ref.value.videoWidth || 1280,
      height: video_ref.value.videoHeight || 720,
    };
  }
}

// Where the video actually renders inside the fixed 1280x720 container,
// given `object-contain` letterboxing.
const display_rect = computed(() => {
  const container_w = 1280;
  const container_h = 720;
  const { width: nw, height: nh } = video_natural.value;
  console.log("natural", video_natural.value);
  const container_ratio = container_w / container_h;
  const natural_ratio = nw / nh || container_ratio;

  let disp_w: number;
  let disp_h: number;
  if (natural_ratio > container_ratio) {
    disp_w = container_w;
    disp_h = container_w / natural_ratio;
  } else {
    disp_h = container_h;
    disp_w = container_h * natural_ratio;
  }

  return {
    disp_w,
    disp_h,
    offset_x: (container_w - disp_w) / 2,
    offset_y: (container_h - disp_h) / 2,
  };
});

// One style object per box in `props.boxes`, converted from capture
// pixels into the actual displayed video rect.
const box_styles = computed(() => {
  const { offset_x, offset_y, disp_w } = display_rect.value;
  const scale = disp_w / video_natural.value.width;

  return (props.boxes ?? []).map((pos) => {
    const [x1, y1] = pos.top_left;

    return {
      left: `${offset_x + x1 * scale}px`,
      top: `${offset_y + y1 * scale}px`,
      width: `${Math.ceil(Math.max(pos.width * scale, 0))}px`,
      height: `${Math.ceil(Math.max(pos.height * scale, 0))}px`,
    };
  });
});

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

// show the store's stream in this component
function attach() {
  status.value = "capturing";
  if (video_ref.value) {
    video_ref.value.srcObject = source.stream;
  }
  source.stream
    .getVideoTracks()[0]
    .addEventListener("ended", stop_capture, { once: true });
}

async function start_scanner() {
  pause_cropper();
  if (roster_config.value.cropper_worker_bundle === null) {
    roster_config.value.cropper_worker_bundle = create_worker_bundle();
  }
  bundle.value = roster_config.value.cropper_worker_bundle;

  const { width, height } = source.size;
  const [config, model] = await Promise.all([getScannerConfig(), getModel()]);
  // the state is built once here and then stays inside the worker's wasm
  const new_scanner_state = {
    debugging: props.debugging,
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
    props.should_start_cropper ? toggle_cropper : null,
    // null,
    0,
    false,
  );
}

function stop_capture() {
  pause_cropper();
  status.value = "idle";
  if (video_ref.value) {
    video_ref.value.srcObject = null;
  }
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

  // one result per op, a late reply from an earlier op must not fork the loop
  let done = false;
  bundle.value.debounced_start(
    WasmOp.Cropper,
    {
      frame,
      debugging: props.debugging,
      full,
      ocr_results: take_ocr_results(),
    },
    (result: ScanResult, timings) => {
      // the scanner waits for these texts whether or not this reply is used
      queue_ocr(result.ocr_jobs);
      if (done || id !== loop_id || result.full !== full) return;
      done = true;
      const result_start = performance.now();
      // the next frame goes out before any UI work
      cropper_loop(id);
      if (props.process_result) {
        props.process_result(result);
      }
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
  } else if (props.should_start_cropper) {
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

    <div
      class="relative overflow-hidden rounded-none"
      :style="
        debugging && status === 'capturing'
          ? 'width: 1280px; height: 720px'
          : 'width: 0; height: 0'
      "
    >
      <video
        ref="video_ref"
        autoplay
        muted
        playsinline
        class="h-full w-full border object-contain"
        :class="{ hidden: !debugging || status !== 'capturing' }"
        @loadedmetadata="on_loaded_metadata"
      />

      <div
        v-if="status !== 'capturing'"
        class="absolute inset-0 flex flex-col items-center justify-center gap-2 text-zinc-500"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="size-10 opacity-40"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <rect x="2" y="3" width="20" height="14" rx="2" stroke-width="1.5" />
          <path stroke-width="1.5" d="M8 21h8M12 17v4" />
        </svg>
        <span class="text-xs">No capture active</span>
      </div>

      <div
        v-for="(style, i) in box_styles"
        :key="i"
        class="pointer-events-none absolute rounded-none border border-red-500"
        :style="style"
      />
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
    </div>
    <button @click="toggle_cropper" class="generic-button">
      {{ cropper_running ? "stop cropper" : "start scropper" }}
    </button>
  </div>
</template>
