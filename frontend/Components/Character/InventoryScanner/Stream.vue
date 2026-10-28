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
import { has_progress, reset_scan, scan_wipes, take_edits } from "./ScanStore";
import { wipe_scan } from "./ScanSwitch";
import { WasmOp } from "@/WasmInterface/WasmTypes";
import {
  ref,
  reactive,
  computed,
  onMounted,
  onUnmounted,
  toRaw,
  watch,
} from "vue";
import { until, useIntervalFn } from "@vueuse/core";
import {
  download_note,
  getModel,
  getScannerConfig,
  ScanResult,
} from "./LoadStorage";
import {
  begin_capture_file,
  CaptureStart,
  CAPTURE_EXTENSION,
  captured_bytes,
  clear_capture,
  download_capture,
  in_recorder,
  open_capture,
  read_record,
  record_input,
  recorder_fell_behind,
  recording,
  SAME_FRAME,
} from "./Capture";
import { scan_settings } from "./ScanSettings";
import { show_debug } from "./ScannerUIutils";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const props = defineProps<{
  process_result: (result: ScanResult) => void;
}>();
const status = defineModel<"idle" | "capturing">("status", { default: "idle" });

const bundle = ref(runtime.cropper);
// the stream outlives this component so permission is only asked once
const source = runtime.frame_source;

const error = ref<string | null>(null);

const show_limitations = ref(false);
const LIMITATIONS = [
  "Only full screen and borderless are supported.",
  "It may not work below 1080p.",
  "HDR and other brightness adjustments (or any kind of filter) will break it - unfortunately this means color blind mode does not work",
  "Non-default fonts may or may not work.",
  "Huge (bigger than 'largest') cursor sizes may break things.",
  "Only the English client is supported for now.",
  "A lossless capture is required.",
  "T3 materials are not detected.",
  "Enhanced 19-20 books are detected as ordinary 19-20 books.",
];

// A capture file being run again in place of a stream, see Capture.ts. It is scanned as this
// build does, OCR and all, each frame waiting until no line is out for OCR and the slots without a
// time budget, so a text is back one scan after it was sent and runs repeat, like capture_replay.
let replay:
  | (CaptureStart & {
      file: File;
      first: number;
      offset: number;
      // scans of the last frame repeated while the last texts come in
      waits: number;
    })
  | null = null;
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
// idle, they are only kept for the next capture
function apply_settings() {
  Object.assign(scan_settings, structuredClone(toRaw(draft)));
  if (status.value === "capturing") restart_scanner();
}
const details = ref(false);
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
    begin(s);
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

// uploads, like debug captures, are behind "Show debug options"

async function start_upload(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files[0];
  input.value = "";
  if (!file) return;

  error.value = null;
  if (file.name.endsWith(CAPTURE_EXTENSION)) return start_replay(file);
  const fake = await file_to_stream(file);
  begin(fake.stream, fake.cleanup);
}

async function start_replay(file: File) {
  stop_capture();
  const { start, first } = await open_capture(file);
  replay = {
    file,
    first,
    offset: first,
    waits: 0,
    ...start,
  };
  Object.assign(scan_settings, {
    forced_21_9: start.forced_21_9,
    brightness: start.brightness,
  });
  discard_settings();
  replaying.value = true;
  status.value = "capturing";
  start_scanner();
}

// the scan starts over, since a capture can only be run again from an empty state
function toggle_recording() {
  recording.value = !recording.value;
  if (recording.value) restart_scanner();
}

function begin(s: MediaStream, cleanup: (() => void) | null = null) {
  stop_capture();
  source.start(s, cleanup);
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
  if (replay) Object.assign(replay, { offset: replay.first, waits: 0 });

  let loaded;
  try {
    loaded = await Promise.all([
      replay ?? source.measure(),
      getScannerConfig(),
      getModel(),
    ]);
  } catch {
    if (status.value === "idle") return;
    stop_capture();
    error.value =
      "Could not download the scanner's files. Check your connection and try again.";
    return;
  }
  const [size, config, model] = loaded;
  // stopped while downloading
  if (!size || status.value === "idle") return;
  const { width, height } = size;
  // the state is built once here and then stays inside the worker's wasm
  const new_scanner_state = {
    settings: copy_settings(),
    buffer: { width, height, size: width * height * 4 },
    config: toRaw(config),
  };
  if (recording.value && !replay)
    begin_capture_file({ width, height, ...copy_settings() });
  // the model only goes to the OCR workers, the templates to them too
  start_ocr(toRaw(model), toRaw(config));
  bundle.value.debounced_start(
    WasmOp.Reserve,
    new_scanner_state,
    (result) => result && toggle_cropper(),
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
  // The share's reported size can be wrong (Chrome gave 3440x1920 for a 3440x1440 screen) and
  // can change mid-share, so the frames decide; the scan starts over at the new size.
  const { width, height } = frame.visibleRect;
  if (width !== source.size.width || height !== source.size.height) {
    frame.close();
    source.resize(width, height);
    restart_scanner();
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
      forward: recording.value,
    },
    (result: ScanResult, timings) => {
      if (!result) return;
      // the scanner waits for these texts whether or not this reply is used
      queue_ocr(result.ocr_jobs);
      if (result.pixels) record_input(result.pixels);
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

// how many times a re-run scans its last frame again while texts are still out
const REPLAY_WAITS = 60;
const ocr_done = () => until(ocr_pending).toBe(0);

// The scan loop over a capture file: each frame's input goes in once its texts are back.
async function replay_loop(id: number, full: boolean) {
  const input = await read_record(replay.file, replay.offset);
  if (id !== loop_id) return;
  if (input?.tag !== "I") return end_replay(id, full, input);
  replay.offset = input.next;
  await ocr_done();
  if (id !== loop_id) return;
  replay_scan(id, full, input.payload, take_ocr_results());
}

// Past the last frame. A re-run scans it again until the texts it waits for are in.
async function end_replay(
  id: number,
  full: boolean,
  end: { tag: string; payload: Uint8Array } | null,
) {
  if (replay.waits < REPLAY_WAITS) {
    await ocr_done();
    if (id !== loop_id) return;
    const texts = take_ocr_results();
    if (texts.length || replay.waits === 0) {
      replay.waits++;
      // the buffer is moved to the worker, so the constant is not sent itself
      return replay_scan(id, full, SAME_FRAME.slice(), texts);
    }
  }
  cropper_running.value = false;
  replay_note.value = "Replay finished, re-run on this build.";
  // how the recorded session ended on the page
  if (end?.tag === "E") {
    (globalThis as any).__capture_reference = JSON.parse(
      new TextDecoder().decode(end.payload),
    );
  }
}

function replay_scan(
  id: number,
  full: boolean,
  input: Uint8Array,
  ocr_results: [number, string][],
) {
  let done = false;
  bundle.value.debounced_start(
    WasmOp.Cropper,
    { replay: input, full, ocr_results },
    (result: ScanResult) => {
      if (!result) return;
      queue_ocr(result.ocr_jobs);
      if (done || id !== loop_id || result.full !== full) return;
      done = true;
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

// a wiped scan starts over on the stream or replay still running
watch(scan_wipes, () => (source.stream || replay) && start_scanner());
</script>

<template>
  <div class="card-shell w-fit max-w-full">
    <div class="card-header">
      <span class="card-title">Screen Capture</span>
    </div>
    <div class="card-body flex flex-col gap-4">
      <p v-if="error" class="max-w-150 text-xs text-(--warning)">
        {{ error }}
      </p>
      <p v-if="download_note" class="max-w-150 text-sm text-(--text-muted)">
        Downloading scanner data: {{ download_note }}.
      </p>

      <div class="flex w-full gap-2">
        <button
          class="generic-button rounded-lg! bg-(--accent-blue)! px-4! py-1.5! enabled:hover:bg-(--accent-blue-hover)!"
          :disabled="status === 'capturing'"
          @click="start_capture"
        >
          Share screen
        </button>
        <label
          v-if="show_debug"
          class="generic-button rounded-lg! bg-(--accent-blue)! px-4! py-1.5! hover:bg-(--accent-blue-hover)!"
        >
          Upload capture
          <input
            type="file"
            accept="image/*,video/*,.hfcap"
            aria-label="Upload image or video"
            class="hidden"
            @change="start_upload"
          />
        </label>
        <button
          class="generic-button rounded-lg! px-4! py-1.5!"
          :disabled="status !== 'capturing'"
          @click="stop_capture"
        >
          Stop
        </button>
        <button
          class="generic-button rounded-lg! px-4! py-1.5!"
          :disabled="status !== 'capturing' && !has_progress"
          @click="wipe_scan"
        >
          Reset scan
        </button>
        <span
          v-if="status === 'idle' && has_progress"
          class="self-center text-sm text-(--warning)"
        >
          Starting a new capture wipes the current scan.
        </span>
      </div>
      <button
        class="flex w-fit items-center gap-1 text-sm text-(--text-muted) hover:text-(--text-bright)"
        :aria-expanded="details"
        @click="details = !details"
      >
        <span class="text-[1.1rem] font-semibold">{{
          details ? "▾" : "▸"
        }}</span>
        {{ details ? "Hide details" : "Show details" }}
      </button>
      <div
        v-if="details && status === 'capturing'"
        class="flex gap-4 text-sm text-(--text-muted)"
      >
        <span aria-label="Frames scanned per second">
          {{ sent_fps }} fps scanned
        </span>
        <span v-if="!replaying" aria-label="Lines waiting for OCR">
          {{ ocr_pending }} in OCR queue
        </span>
      </div>
      <template v-if="details && !replaying">
        <div
          class="grid w-fit grid-cols-[auto_auto_auto] items-center gap-x-2 gap-y-1 text-sm"
        >
          <span>Aspect</span>
          <select v-model="aspect_choice" class="selector" aria-label="Aspect">
            <option value="auto">Auto</option>
            <option value="16:9">16:9</option>
            <option value="21:9">21:9 (native or forced)</option>
          </select>
          <span class="text-(--text-muted)">
            {{
              draft.forced_21_9.manual
                ? ""
                : detected.forced_21_9 == null
                  ? status === "capturing"
                    ? "detecting..."
                    : "-"
                  : detected.forced_21_9
                    ? "21:9"
                    : "16:9"
            }}
          </span>
          <span>Brightness</span>
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
          <span v-else class="text-(--text-muted)">
            {{
              detected.brightness ??
              (status === "capturing" ? "detecting..." : "-")
            }}
            <template v-if="detected.brightness != null">
              (within 5 ish should be fine)
            </template>
          </span>
        </div>
        <div
          v-if="draft_changed"
          class="flex max-w-150 flex-wrap items-center gap-2"
        >
          <span v-if="status === 'capturing'" class="text-sm text-(--warning)">
            Applying restarts the scanner and wipes the current scan.
          </span>
          <button
            class="generic-button bg-(--accent-blue)! px-[5px]! py-[3px]! text-lg enabled:hover:bg-(--accent-blue-hover)!"
            :disabled="!draft_valid"
            @click="apply_settings"
          >
            Apply
          </button>
          <button
            class="generic-button px-[5px]! py-[3px]! text-lg"
            @click="discard_settings"
          >
            Cancel
          </button>
        </div>
      </template>
      <p v-if="replay_note" class="text-sm text-(--text-muted)">
        {{ replay_note }}
      </p>

      <!-- as wide as the rest of the card makes it, the spacer widens it with limitations open -->
      <div>
        <div v-if="show_limitations" class="h-0 w-100" />
        <div class="w-0 min-w-full">
          <div class="text-(--text-bright)">Troubleshooting</div>
          <div class="pl-4">
            Check the aspect ratio and brightness under "Show details" above. If
            they look right and it still doesn't work, it's probably my fault.
            Please report it on
            <a href="https://discord.gg/KWDpQyvgzc" class="link">Discord</a> or
            <a href="https://github.com/Kenivia/Honing-Forecast" class="link"
              >GitHub</a
            >.
          </div>
          <div>
            <button
              class="mt-2 flex items-center gap-1 text-(--text-bright) hover:text-(--text-main)"
              :aria-expanded="show_limitations"
              @click="show_limitations = !show_limitations"
            >
              <span class="font-semibold">{{
                show_limitations ? "▾" : "▸"
              }}</span>
              Known Limitations
            </button>
            <ul
              v-if="show_limitations"
              aria-label="Limitations"
              class="flex flex-col gap-0.5 pl-4"
            >
              <li v-for="limitation in LIMITATIONS" :key="limitation">
                - {{ limitation }}
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- debug captures, behind the sidebar's "Show debug options" -->
      <template v-if="show_debug">
        <div class="flex flex-wrap items-center gap-2">
          <button
            class="generic-button px-[5px]! py-[3px]! text-lg"
            :disabled="status !== 'capturing' || replaying"
            @click="toggle_recording"
          >
            {{ recording ? "Stop recording" : "Record debug capture" }}
          </button>
          <button
            class="generic-button px-[5px]! py-[3px]! text-lg"
            :disabled="recording || in_recorder > 0 || !captured_bytes"
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
        <p
          v-if="recorder_fell_behind"
          class="max-w-150 text-sm text-(--warning)"
        >
          Recording stopped: this computer could not keep up with recording
          every frame. What was recorded so far can still be downloaded.
        </p>
      </template>
    </div>
  </div>
</template>
