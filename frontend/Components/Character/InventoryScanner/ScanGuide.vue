<script setup lang="ts">
import { ref } from "vue";

const LIMITATIONS = [
  "Only full screen and borderless are supported.",
  "It may not work below 1080p.",
  "HDR and other brightness adjustments will break it.",
  "Non-default fonts may or may not work.",
  "Huge (bigger than 'largest') cursor sizes may break things.",
  "Only the English client is supported for now. It probably won't work with other languages.",
  "A lossless capture is required.",
  "T3 materials are not detected.",
  "Enhanced 19-20 books are detected as ordinary 19-20 books.",
];
const show_limitations = ref(false);
</script>

<template>
  <div class="card-shell w-150! max-w-full text-base">
    <div class="card-header">
      <span class="card-title">How to use</span>
    </div>
    <div class="card-body flex flex-col gap-3">
      <div>
        <div class="text-(--text-bright)">Step 1</div>
        <div class="pl-4">
          Press <span class="text-blue-400">Share screen</span>. Share the LOST
          ARK window. (it may not appear as an option while you're in a loading
          screen)
        </div>
      </div>
      <div>
        <div class="text-(--text-bright)">Step 2</div>
        <div class="pl-4">
          Open your storage or just your inventory, then hover items in game.
          You only need to do it for items with an
          <span class="text-(--series-fusion)">Orange</span> border, and only
          ~300ms per hover is needed. You should hover until no
          <span class="text-(--series-fusion)">Orange</span> slots remain.
        </div>
      </div>

      <div>
        <div class="text-(--text-bright)">Step 3</div>
        <div class="pl-4">
          Check the manifest below, and "export to calculator" when ready. If
          you don't export, it won't be saved on reload.
        </div>
      </div>

      <div>
        <button
          class="flex items-center gap-1 text-(--text-bright) hover:text-(--text-main)"
          :aria-expanded="show_limitations"
          @click="show_limitations = !show_limitations"
        >
          <span class="text-[1.1rem] font-semibold">{{
            show_limitations ? "▾" : "▸"
          }}</span>
          Limitations
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
</template>
