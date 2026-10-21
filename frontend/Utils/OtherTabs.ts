import { useBroadcastChannel } from "@vueuse/core";
import { ref, watch } from "vue";

// Every tab saves the whole config, so a second one overwrites the first's changes. Tabs
// greet each other here, once per page load, and OtherTabBanner warns while another is open.
type Message = { id: string; kind: "hello" | "here" | "bye" };

const id = crypto.randomUUID();
export const other_tabs = ref<string[]>([]);
const { data, post } = useBroadcastChannel<Message, Message>({
  name: "honing-forecast-tabs",
});

watch(data, (message) => {
  if (!message || message.id === id) return;
  if (message.kind === "bye") {
    other_tabs.value = other_tabs.value.filter((other) => other !== message.id);
    return;
  }
  if (!other_tabs.value.includes(message.id)) other_tabs.value.push(message.id);
  if (message.kind === "hello") post({ id, kind: "here" });
});

post({ id, kind: "hello" });
window.addEventListener("pagehide", () => post({ id, kind: "bye" }));
