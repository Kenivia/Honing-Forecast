<script setup lang="ts">
import { get_readable } from "@/Components/Character/InventoryScanner/FramePassing";
import { useRosterStore } from "@/Stores/RosterConfig";
import { ScannerState, WasmOp } from "@/WasmInterface/WasmWorker";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { storeToRefs } from "pinia";
import { ref, computed, onMounted, onUnmounted, toRaw } from "vue";

const { roster_config } = storeToRefs(useRosterStore());
const config = ref(roster_config.value.scanner_config);

export interface OneIconConfig {
  name: string;
  width: number;
  height: number;
  data: unknown;
}

const video_ref = ref<HTMLVideoElement | null>(null);
const stream = ref<MediaStream | null>(null);
const error = ref<string | null>(null);
const status = ref<"idle" | "capturing">("idle");

const cropper_worker_bundle = ref(null);

// --- Setup phase state ---------------------------------------------------
const setup_name = ref("");
const top_left_x = ref(0);
const top_left_y = ref(0);
const bot_right_x = ref(100);
const bot_right_y = ref(100);

// Track the real capture resolution so we can correctly place the overlay
// box even if the stream isn't exactly 1280x720.
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

// The red box style, converting from the 1280x720 reference space the
// coordinates are entered in, into the actual displayed video rect.
const box_style = computed(() => {
  const { disp_w, disp_h, offset_x, offset_y } = display_rect.value;
  const scale_x = disp_w / 1280;
  const scale_y = disp_h / 720;

  const x1 = Number(top_left_x.value) || 0;
  const y1 = Number(top_left_y.value) || 0;
  const x2 = Number(bot_right_x.value) || 0;
  const y2 = Number(bot_right_y.value) || 0;

  return {
    left: `${offset_x + x1 * scale_x}px`,
    top: `${offset_y + y1 * scale_y}px`,
    width: `${Math.max((x2 - x1) * scale_x, 0)}px`,
    height: `${Math.max((y2 - y1) * scale_y, 0)}px`,
  };
});

function confirm_setup() {
  if (!cropper_worker_bundle.value?.result) return;

  const incoming_new_icon = {
    position: {
      top_left: [Number(top_left_x.value), Number(top_left_y.value)],
      bot_right: [Number(bot_right_x.value), Number(bot_right_y.value)],
    },
    name: setup_name.value,
  };

  const modified_state = {
    ...cropper_worker_bundle.value.result,
    config: toRaw(config.value),
    incoming_new_icon,
  };

  cropper_worker_bundle.value.debounced_start(
    WasmOp.Setup,
    modified_state,
    (scanner_state) => {
      config.value = scanner_state.config;
      save_config();
    },
    0,
    false,
  );
}

// --- Config preview -------------------------------------------------------

function draw_icon(canvas: HTMLCanvasElement | null, icon: OneIconConfig) {
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  // `icon.data` may arrive as a plain number array or a typed array
  // depending on serde-wasm-bindgen's serialization; both work here.
  const clamped = new Uint8ClampedArray(icon.data as ArrayLike<number>);
  const image_data = new ImageData(clamped, icon.width, icon.height);
  ctx.putImageData(image_data, 0, 0);
}

// --- Existing capture logic ------------------------------------------------
async function start_capture() {
  try {
    error.value = null;
    const s = await navigator.mediaDevices.getDisplayMedia({
      video: { frameRate: 30 },
      audio: false,
    });
    stream.value = s;
    status.value = "capturing";

    if (video_ref.value) {
      video_ref.value.srcObject = s;
    }

    const [track] = s.getVideoTracks();
    track.addEventListener("ended", stop_capture);

    const width = track.getSettings().width;
    const height = track.getSettings().height;

    if (cropper_worker_bundle.value === null) {
      cropper_worker_bundle.value = create_worker_bundle();
    }

    const new_scanner_state = cropper_worker_bundle.value.result ?? {
      screen_info: {
        total_width: width,
        total_height: height,
      },
    };
    new_scanner_state.buffer = {
      width,
      height,
      size: width * height * 4,
    };
    new_scanner_state.config = []; //toRaw(roster_config.value.scanner_config);

    cropper_worker_bundle.value.debounced_start(
      WasmOp.Reserve,
      {
        readable: get_readable(track),
        scanner_state: new_scanner_state,
      },
      null,
      0,
      false,
    );
  } catch (e: unknown) {
    if (
      e instanceof Error &&
      e.name !== "NotAllowedError" &&
      e.name !== "InvalidStateError"
    ) {
      throw e;
    }
    status.value = "idle";
  }
}

function stop_capture() {
  if (cropper_worker_bundle.value !== null) {
    cropper_worker_bundle.value.debounced_start(
      WasmOp.Dealloc,
      cropper_worker_bundle.value.result,
      cropper_worker_bundle.value.cancel,
      0,
      false,
    );
  }
  stream.value?.getTracks().forEach((t) => t.stop());
  stream.value = null;
  if (video_ref.value) {
    video_ref.value.srcObject = null;
  }
  status.value = "idle";
}
const editing_index = ref<number | null>(null);
const edit_name = ref("");

function get_config_array() {
  return cropper_worker_bundle.value?.result?.config;
}

function save_config() {
  roster_config.value.scanner_config = structuredClone(get_config_array());
}
function delete_icon(index: number) {
  // console.log(config, index, cropper_worker_bundle.value?.result);
  if (!config.value) return;
  config.value.splice(index, 1);
  if (editing_index.value === index) editing_index.value = null;
  save_config();
}

function move_icon_up(index: number) {
  if (!config.value || index <= 0) return;
  [config.value[index - 1], config.value[index]] = [
    config.value[index],
    config.value[index - 1],
  ];
  save_config();
}

function move_icon_down(index: number) {
  if (!config.value || index >= config.value.length - 1) return;
  [config.value[index], config.value[index + 1]] = [
    config.value[index + 1],
    config.value[index],
  ];
  save_config();
}

function start_rename(index: number, current_name: string) {
  editing_index.value = index;
  edit_name.value = current_name;
}

function confirm_rename(index: number) {
  if (!config.value) return;
  const trimmed = edit_name.value.trim();
  if (trimmed) {
    config.value[index].name = trimmed;
  }
  editing_index.value = null;
  save_config();
}

function cancel_rename() {
  editing_index.value = null;
}

onMounted(start_capture);
onUnmounted(stop_capture);
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
        {{
          status === "capturing"
            ? "● Live"
            : status === "idle"
              ? "Initialising…"
              : "idle"
        }}
      </span>
    </div>

    <!-- Preview -->
    <div
      class="relative overflow-hidden rounded-none"
      style="width: 1280px; height: 720px"
    >
      <video
        ref="video_ref"
        autoplay
        muted
        playsinline
        class="h-full w-full border object-contain"
        :class="{ hidden: status !== 'capturing' }"
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

      <!-- Red outline showing current setup coordinates -->
      <div
        v-if="status === 'capturing'"
        class="pointer-events-none absolute rounded-none border border-red-500"
        :style="box_style"
      />
    </div>

    <!-- Error -->
    <p
      v-if="error"
      class="rounded bg-red-500/10 px-3 py-2 text-xs text-red-400"
    >
      {{ error }}
    </p>

    <!-- Controls -->
    <div class="flex gap-2">
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status === 'capturing'"
        @click="start_capture"
      >
        {{ status === "idle" ? "Capture Again" : "Start Capture" }}
      </button>
      <button
        class="rounded-md bg-zinc-700 px-3 py-1.5 text-sm transition-colors hover:bg-zinc-600 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status !== 'capturing'"
        @click="stop_capture"
      >
        Stop
      </button>
    </div>
  </div>

  <!-- Setup phase -->
  <div class="card-shell card-body mt-4 flex flex-col gap-3">
    <span class="text-sm font-semibold">New Icon Setup</span>

    <div class="flex flex-wrap items-end gap-3">
      <label class="flex flex-col gap-1 text-xs">
        Top left X
        <input
          v-model.number="top_left_x"
          type="number"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Top left Y
        <input
          v-model.number="top_left_y"
          type="number"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Bot right X
        <input
          v-model.number="bot_right_x"
          type="number"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Bot right Y
        <input
          v-model.number="bot_right_y"
          type="number"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Name
        <input
          v-model="setup_name"
          type="text"
          class="rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="!setup_name || status !== 'capturing'"
        @click="confirm_setup"
      >
        Confirm Icon
      </button>
    </div>
  </div>

  <div class="mt-4 flex flex-col gap-2">
    <div
      v-for="(icon, index) in config"
      :key="icon.name + index"
      class="flex items-center gap-3 rounded-md bg-zinc-800/50 p-2"
    >
      <canvas
        :ref="(el) => draw_icon(el as HTMLCanvasElement, icon)"
        :width="icon.width"
        :height="icon.height"
        class="rounded-none border border-zinc-600"
        style="image-rendering: pixelated"
      />

      <!-- Name / rename input -->
      <div class="flex-1">
        <input
          v-if="editing_index === index"
          v-model="edit_name"
          type="text"
          class="w-full rounded bg-zinc-900 px-2 py-1 text-sm"
          autofocus
          @keyup.enter="confirm_rename(index)"
          @keyup.escape="cancel_rename"
        />
        <span v-else class="text-sm">{{ icon.name }}</span>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-1">
        <template v-if="editing_index === index">
          <button
            class="rounded-md bg-blue-600 px-2 py-1 text-xs hover:bg-blue-500"
            @click="confirm_rename(index)"
          >
            Save
          </button>
          <button
            class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
            @click="cancel_rename"
          >
            Cancel
          </button>
        </template>
        <template v-else>
          <button
            v-if="index > 0"
            class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
            title="Move up"
            @click="move_icon_up(index)"
          >
            ↑
          </button>
          <button
            v-if="index < config.length - 1"
            class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
            title="Move down"
            @click="move_icon_down(index)"
          >
            ↓
          </button>
          <button
            class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
            title="Rename"
            @click="start_rename(index, icon.name)"
          >
            Rename
          </button>
          <button
            class="rounded-md bg-red-600 px-2 py-1 text-xs hover:bg-red-500"
            title="Delete"
            @click="delete_icon(index)"
          >
            Delete
          </button>
        </template>
      </div>
    </div>
  </div>
</template>
