<script setup lang="ts">
import {
  CAPTURE_FPS,
  file_to_stream,
} from "@/Components/Character/InventoryScanner/FramePassing";
import {
  ocr_pending,
  queue_ocr,
  start_ocr,
  stop_ocr,
  take_ocr_results,
} from "./OcrRelay";
import { has_progress, reset_scan, take_edits } from "./ScanStore";
import { useRosterStore } from "@/Stores/RosterConfig";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { storeToRefs } from "pinia";
import {
  ref,
  reactive,
  computed,
  onMounted,
  onUnmounted,
  toRaw,
  watch,
} from "vue";
import { useIntervalFn } from "@vueuse/core";
import { getModel, getScannerConfig, ScanResult } from "./LoadStorage";
import {
  add_capture,
  CAPTURE_EXTENSION,
  captured_bytes,
  clear_capture,
  download_capture,
  open_capture,
  read_record,
  recording,
} from "./Capture";
import { scan_settings } from "./ScanSettings";
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

// a capture file being run again in place of a stream, see Capture.ts
let replay: {
  file: File;
  first: number;
  offset: number;
  width: number;
  height: number;
} | null = null;
const replaying = ref(false);
const replay_note = ref("");

// Edited here and applied with a restart, since everything scanned depends on them.
const copy_settings = () => structuredClone(toRaw(scan_settings));
const draft = reactive(copy_settings());
const draft_changed = computed(
  () => JSON.stringify(draft) !== JSON.stringify(scan_settings),
);
const draft_valid = computed(() => {
  const { manual, value } = draft.brightness;
  return !manual || (typeof value === "number" && value >= 0 && value <= 100);
});
const aspect_choice = computed({
  get: () =>
    draft.forced_21_9.manual
      ? draft.forced_21_9.value
        ? "21:9"
        : "16:9"
      : "auto",
  set: (choice) => {
    draft.forced_21_9 =
      choice === "auto"
        ? { value: null, manual: false }
        : { value: choice === "21:9", manual: true };
  },
});
const brightness_choice = computed({
  get: () => (draft.brightness.manual ? "manual" : "auto"),
  set: (choice) => {
    draft.brightness =
      choice === "auto"
        ? { value: null, manual: false }
        : { value: detected.value.brightness ?? 50, manual: true };
  },
});
function apply_settings() {
  Object.assign(scan_settings, structuredClone(toRaw(draft)));
  restart_scanner();
}
function discard_settings() {
  Object.assign(draft, copy_settings());
}

// what the scanner last reported, shown beside a setting left on auto
const detected = ref<{
  forced_21_9: boolean | null;
  brightness: number | null;
}>({ forced_21_9: null, brightness: null });
function note_detected(result: ScanResult) {
  const brightness =
    result.brightness == null ? null : Math.round(result.brightness);
  if (
    detected.value.forced_21_9 !== result.forced_21_9 ||
    detected.value.brightness !== brightness
  ) {
    detected.value = { forced_21_9: result.forced_21_9, brightness };
  }
}

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

// async function start_upload(event: Event) {
//   const input = event.target as HTMLInputElement;
//   const file = input.files[0];
//   input.value = "";
//   if (!file) return;

//   error.value = null;
//   if (file.name.endsWith(CAPTURE_EXTENSION)) return start_replay(file);
//   const fake = await file_to_stream(file);
//   begin(fake.stream, fake.width, fake.height, fake.cleanup);
// }

// async function start_replay(file: File) {
//   stop_capture();
//   const { start, first } = await open_capture(file);
//   replay = { file, first, offset: first, ...start };
//   Object.assign(scan_settings, {
//     forced_21_9: start.forced_21_9,
//     brightness: start.brightness,
//   });
//   discard_settings();
//   replaying.value = true;
//   status.value = "capturing";
//   start_scanner();
// }

// the scan starts over, since a capture can only be run again from an empty state
function toggle_recording() {
  recording.value = !recording.value;
  if (recording.value) restart_scanner();
}

function begin(
  s: MediaStream,
  width: number,
  height: number,
  cleanup: (() => void) | null = null,
) {
  stop_capture();
  source.start(s, width, height, cleanup);
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
  // what was recorded belongs to the state that is gone
  clear_capture();
  replay_note.value = "";
  detected.value = { forced_21_9: null, brightness: null };
  if (replay) replay.offset = replay.first;

  const { width, height } = replay ?? source.size;
  const [config, model] = await Promise.all([getScannerConfig(), getModel()]);
  // the state is built once here and then stays inside the worker's wasm
  const new_scanner_state = {
    settings: copy_settings(),
    buffer: { width, height, size: width * height * 4 },
    config: toRaw(config),
    record: recording.value && !replay,
  };
  // The model only goes to the OCR workers, the templates to them too. A replay has its texts.
  if (!replay) start_ocr(toRaw(model), toRaw(config));
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
  recording.value = false;
  if (!source.stream && !replay) return;
  replay = null;
  replaying.value = false;
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
  if (replay) return replay_loop(id, full);
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
      record: recording.value,
    },
    (result: ScanResult, timings) => {
      // the scanner waits for these texts whether or not this reply is used
      queue_ocr(result.ocr_jobs);
      if (result.capture) add_capture(result.capture);
      if (done || id !== loop_id || result.full !== full) return;
      done = true;
      const result_start = performance.now();
      // the next frame goes out before any UI work
      cropper_loop(id);
      note_detected(result);
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

// The scan loop over a capture file: each scan record goes in as it was, as fast as it is scanned.
async function replay_loop(id: number, full: boolean) {
  const record = await read_record(replay.file, replay.offset);
  if (id !== loop_id) return;
  if (record?.tag !== "F") {
    cropper_running.value = false;
    replay_note.value ||= "Replay finished, every scan ended as recorded.";
    // the page's account of how the recorded session ended
    if (record) {
      (globalThis as any).__capture_reference = JSON.parse(
        new TextDecoder().decode(record.payload),
      );
    }
    return;
  }
  replay.offset = record.next;
  let done = false;
  bundle.value.debounced_start(
    WasmOp.Cropper,
    { replay: record.payload, full },
    (result: ScanResult, timings) => {
      if (done || id !== loop_id || result.full !== full) return;
      done = true;
      if (timings.replay_mismatch != null) {
        replay_note.value = `Replay differs from the recording, first at scan ${timings.replay_mismatch}.`;
      }
      cropper_loop(id);
      note_detected(result);
      props.process_result(result);
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

    <div
      v-if="status === 'capturing' && !replaying"
      class="flex flex-wrap items-center gap-2"
    >
      <span> Aspect: </span>
      <select v-model="aspect_choice" class="selector" aria-label="Aspect">
        <option value="auto">Auto</option>
        <option value="16:9">16:9</option>
        <option value="21:9">Forced 21:9</option>
      </select>
      <span
        v-if="!draft.forced_21_9.manual"
        class="text-sm text-(--text-muted)"
      >
        {{
          detected.forced_21_9 == null
            ? "detecting"
            : detected.forced_21_9
              ? "21:9"
              : "16:9"
        }}
      </span>
      <span class="ml-4"> Brightness: </span>
      <select
        v-model="brightness_choice"
        class="selector"
        aria-label="Brightness"
      >
        <option value="auto">Auto</option>
        <option value="manual">Manual</option>
      </select>
      <input
        v-if="draft.brightness.manual"
        v-model.number="draft.brightness.value"
        class="generic-input number-border w-13"
        type="number"
        min="0"
        max="100"
        aria-label="In-game brightness"
      />
      <span v-else class="text-sm text-(--text-muted)">
        {{ detected.brightness ?? "detecting" }}
      </span>
    </div>

    <div
      v-if="status === 'capturing' && !replaying && draft_changed"
      class="flex flex-wrap items-center gap-2"
    >
      <span class="text-sm text-(--warning)">
        Applying restarts the scanner and clears everything scanned so far,
        corrections included.
      </span>
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="!draft_valid"
        @click="apply_settings"
      >
        Apply
      </button>
      <button
        class="rounded-md bg-zinc-700 px-3 py-1.5 text-sm transition-colors hover:bg-zinc-600"
        @click="discard_settings"
      >
        Cancel
      </button>
    </div>

    <div class="flex gap-2">
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status === 'capturing'"
        @click="start_capture"
      >
        Share screen
      </button>
      <!-- <label
        class="cursor-pointer rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500"
      >
        Upload image / video / capture
        <input
          type="file"
          accept="image/*,video/*,.hfcap"
          aria-label="Upload image or video"
          class="hidden"
          @change="start_upload"
        />
      </label> -->
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
        v-if="status === 'capturing' && !replaying"
        class="self-center text-sm text-(--text-muted)"
        aria-label="Lines waiting for OCR"
      >
        {{ ocr_pending }} in OCR queue
      </span>
      <span
        v-if="status === 'idle' && has_progress"
        class="self-center text-sm text-(--warning)"
      >
        Starting a capture wipes the current scan.
      </span>
    </div>
    <p v-if="replay_note" class="text-sm text-(--text-muted)">
      {{ replay_note }}
    </p>
    <div class="flex flex-wrap items-center gap-2">
      <button
        class="rounded-md bg-zinc-700 px-3 py-1.5 text-sm transition-colors hover:bg-zinc-600 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status !== 'capturing' || replaying"
        @click="toggle_recording"
      >
        {{ recording ? "Stop recording" : "Record debug capture" }}
      </button>
      <button
        class="rounded-md bg-zinc-700 px-3 py-1.5 text-sm transition-colors hover:bg-zinc-600 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="recording || !captured_bytes"
        @click="download_capture"
      >
        Download capture
      </button>
      <span
        v-if="captured_bytes"
        class="text-sm text-(--text-muted)"
        aria-label="Capture size"
      >
        {{ (captured_bytes / 1e6).toFixed(0) }} MB
        {{ recording ? "recorded" : "ready" }}
      </span>
    </div>
    <p class="text-xs text-(--text-muted)">
      A debug capture restarts the scan and records everything on the shared
      screen, for sending in when something is read wrong. Stop recording,
      correct the slots by hand, then download.
    </p>
    <button @click="toggle_cropper" class="generic-button">
      {{ cropper_running ? "stop cropper" : "start scropper" }}
    </button>
  </div>
</template>
