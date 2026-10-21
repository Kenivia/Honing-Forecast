<script setup lang="ts">
import { useBroadcastChannel } from "@vueuse/core";
import { onMounted, ref, watch } from "vue";

// Every tab saves the whole config, so a second one overwrites the first's changes. Tabs
// greet each other here and the warning shows while another is open.
type Message = { id: string; kind: "hello" | "here" | "bye" };

const id = crypto.randomUUID();
const others = ref<string[]>([]);
const { data, post } = useBroadcastChannel<Message, Message>({
  name: "honing-forecast-tabs",
});

watch(data, (message) => {
  if (!message || message.id === id) return;
  if (message.kind === "bye") {
    others.value = others.value.filter((other) => other !== message.id);
    return;
  }
  if (!others.value.includes(message.id)) others.value.push(message.id);
  if (message.kind === "hello") post({ id, kind: "here" });
});

onMounted(() => post({ id, kind: "hello" }));
window.addEventListener("pagehide", () => post({ id, kind: "bye" }));
</script>

<template>
  <div
    v-if="others.length"
    role="alert"
    class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-(--warning-dark) bg-(--bg-muted) px-4 py-2 text-sm"
  >
    <span class="text-(--warning)">
      Honing Forecast is open in another tab. Each tab saves over the other's
      changes, so use one tab at a time.
    </span>
  </div>
</template>
