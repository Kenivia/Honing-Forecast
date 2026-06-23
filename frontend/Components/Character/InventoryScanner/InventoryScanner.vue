<script setup lang="ts">
import { ScannerState, WasmOp } from "@/WasmInterface/WasmWorker";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { ref, onMounted, onUnmounted } from "vue";

const video_ref = ref<HTMLVideoElement | null>(null);
const stream = ref<MediaStream | null>(null);
const error = ref<string | null>(null);
const status = ref<"idle" | "capturing" | "stopped">("idle");

const cropper_worker_bundle = ref(null);

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

    // console.log("buffer size", width * height * 4);

    const processor = new MediaStreamTrackProcessor({ track });
    console.log(
      cropper_worker_bundle.value,
      cropper_worker_bundle.value.result,
    );
    const new_scanner_state = cropper_worker_bundle.value.result ?? {};
    new_scanner_state.buffer = { size: width * height * 4 };
    cropper_worker_bundle.value.debounced_start(
      WasmOp.Reserve,
      {
        readable: processor.readable,
        scanner_state: new_scanner_state,
      },
      (scanner_state) => cropper_loop(scanner_state),
      0,
      false,
    );
  } catch (e: unknown) {
    if (e instanceof Error && e.name !== "NotAllowedError") {
      throw e;
    }
    status.value = "stopped";
  }
}

async function cropper_loop(scanner_state: ScannerState) {
  if (
    cropper_worker_bundle.value === null ||
    cropper_worker_bundle.value.worker === null ||
    cropper_worker_bundle.value.result === null
  ) {
    console.log("no more cropper");
    return;
  }

  cropper_worker_bundle.value.debounced_start(
    WasmOp.Cropper,
    scanner_state,
    (scanner_state) => cropper_loop(scanner_state),
    0,
    false,
  );
}

function stop_capture() {
  console.log(
    "stop",
    cropper_worker_bundle.value,
    cropper_worker_bundle.value.result,
  );
  cropper_worker_bundle.value.debounced_start(
    WasmOp.Dealloc,
    cropper_worker_bundle.value.result,
    cropper_worker_bundle.value.cancel,
    0,
    false,
  );

  stream.value?.getTracks().forEach((t) => t.stop());
  stream.value = null;
  if (video_ref.value) {
    video_ref.value.srcObject = null;
  }
  status.value = "stopped";
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
          'bg-zinc-500/20 text-zinc-400':
            status === 'idle' || status === 'stopped',
        }"
      >
        {{
          status === "capturing"
            ? "● Live"
            : status === "idle"
              ? "Initialising…"
              : "Stopped"
        }}
      </span>
    </div>

    <!-- Preview -->
    <div
      class="relative aspect-video w-full overflow-hidden rounded-lg bg-black"
    >
      <video
        ref="video_ref"
        autoplay
        muted
        playsinline
        class="h-full w-full object-contain"
        :class="{ hidden: status !== 'capturing' }"
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
        {{ status === "stopped" ? "Capture Again" : "Start Capture" }}
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
