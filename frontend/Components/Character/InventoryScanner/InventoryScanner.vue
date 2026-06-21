<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const videoRef = ref<HTMLVideoElement | null>(null);
const stream = ref<MediaStream | null>(null);
const error = ref<string | null>(null);
const status = ref<"idle" | "capturing" | "stopped">("idle");

async function startCapture() {
  try {
    error.value = null;
    const s = await navigator.mediaDevices.getDisplayMedia({
      video: true,
      audio: false,
    });
    stream.value = s;
    status.value = "capturing";

    if (videoRef.value) {
      videoRef.value.srcObject = s;
    }

    // Handle the user stopping via the browser's built-in "Stop sharing" button
    s.getVideoTracks()[0].addEventListener("ended", stopCapture);
  } catch (e: unknown) {
    if (e instanceof Error && e.name !== "NotAllowedError") {
      error.value = e.message;
    }
    status.value = "stopped";
  }
}

function stopCapture() {
  stream.value?.getTracks().forEach((t) => t.stop());
  stream.value = null;
  if (videoRef.value) {
    videoRef.value.srcObject = null;
  }
  status.value = "stopped";
}

onMounted(startCapture);
onUnmounted(stopCapture);
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
        ref="videoRef"
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
        @click="startCapture"
      >
        {{ status === "stopped" ? "Capture Again" : "Start Capture" }}
      </button>
      <button
        class="rounded-md bg-zinc-700 px-3 py-1.5 text-sm transition-colors hover:bg-zinc-600 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="status !== 'capturing'"
        @click="stopCapture"
      >
        Stop
      </button>
    </div>
  </div>
</template>
