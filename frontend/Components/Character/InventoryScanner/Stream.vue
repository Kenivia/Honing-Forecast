<script setup lang="ts">
import { get_readable } from "@/Components/Character/InventoryScanner/FramePassing";
import { useRosterStore } from "@/Stores/RosterConfig";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { storeToRefs } from "pinia";
import { ref, computed, onMounted, onUnmounted, toRaw } from "vue";
import {
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
} from "./ScannerConfigStorage";

const props = defineProps<{ boxes?: ScaledPosition[] }>();
const status = defineModel<"idle" | "capturing">("status", { default: "idle" });

const roster_store = useRosterStore();
const { roster_config } = storeToRefs(roster_store);

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

const video_ref = ref<HTMLVideoElement | null>(null);
const stream = ref<MediaStream | null>(null);
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

// One style object per box in `props.boxes`, converted from the
// 1280x720 reference space into the actual displayed video rect.
const box_styles = computed(() => {
  const { disp_w, disp_h, offset_x, offset_y } = display_rect.value;
  const scale_x = disp_w / 1280;
  const scale_y = disp_h / 720;

  return (props.boxes ?? []).map((pos) => {
    const [x1, y1] = pos.top_left;
    const [x2, y2] = [
      pos.top_left[0] + pos.width,
      pos.top_left[1] + pos.height,
    ];
    return {
      left: `${offset_x + x1 * scale_x}px`,
      top: `${offset_y + y1 * scale_y}px`,
      width: `${Math.max((x2 - x1) * scale_x, 0)}px`,
      height: `${Math.max((y2 - y1) * scale_y, 0)}px`,
    };
  });
});

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

    if (roster_config.value.cropper_worker_bundle === null) {
      roster_config.value.cropper_worker_bundle = create_worker_bundle();
    }

    const bundle = roster_config.value.cropper_worker_bundle;

    const new_scanner_state = bundle.result ?? {
      screen_info: {
        total_width: width,
        total_height: height,
      },
      downscaled_cache: {
        buffer: {
          width: 1280,
          height: 720,
          size: 1280 * 720 * 4,
        },
      },
    };
    new_scanner_state.buffer = {
      width,
      height,
      size: width * height * 4,
    };
    new_scanner_state.config = (toRaw(config.value) ?? []).map((entry) => {
      if ("position" in entry) {
        const { position, ...rest } = entry as OneIconConfig & {
          position: unknown;
        };
        return { ...rest, offset: position };
      }
      return entry;
    });

    console.log(new_scanner_state.config);
    bundle.debounced_start(
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
  const bundle = roster_config.value.cropper_worker_bundle;
  if (bundle !== null) {
    bundle.debounced_start(
      WasmOp.Dealloc,
      bundle.result,
      bundle.cancel,
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

// Let a parent trigger capture imperatively if it needs to (e.g. reuse
// elsewhere without the buttons below).
defineExpose({ start_capture, stop_capture });

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
</template>
