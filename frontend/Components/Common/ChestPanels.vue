<script setup lang="ts">
import { computed } from "vue";
import { BAND_COLORS } from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import type { Chest } from "@/Utils/Chest";

// The chests still to be opened, as two lists. Which chest goes where is the page's to say. The
// slot is the foot of the selection panel.
const props = defineProps<{
  selection: Chest[];
  obtain_all: Chest[];
  obtain_all_title?: string;
  editing?: boolean;
}>();
const emit = defineEmits<{
  count: [key: string, event: Event];
  remove: [key: string];
}>();

const BANDS = ["Char bound", "Roster bound", "Tradable"];

const panels = computed(() => [
  { title: "Selection chests", chests: props.selection },
  {
    title: props.obtain_all_title ?? "Obtain-all chests",
    chests: props.obtain_all,
  },
]);

const amount_text = (amount: number) =>
  amount.toLocaleString("en-US", { maximumFractionDigits: 1 });
</script>

<template>
  <div class="flex flex-wrap items-start gap-3">
    <div
      v-for="(panel, at) in panels"
      :key="panel.title"
      class="card-shell card-body text-sm"
      role="group"
      :aria-label="panel.title"
    >
      <div class="card-title">{{ panel.title }}</div>
      <span v-if="!panel.chests.length" class="text-(--text-muted)">
        None.
      </span>
      <div
        v-for="chest in panel.chests"
        :key="chest.key"
        class="flex flex-wrap items-center gap-2 border-b border-(--border-very-muted) py-1"
      >
        <input
          type="number"
          min="0"
          class="generic-input h-7! w-16 pl-1"
          :disabled="!editing"
          :aria-label="`${chest.title} count`"
          :value="chest.count"
          @change="emit('count', chest.key, $event)"
        />
        <span class="flex flex-col">
          {{ chest.title }}
          <span
            class="annotation"
            :style="{ color: `var(${BAND_COLORS[chest.band]})` }"
          >
            {{ BANDS[chest.band] }}
          </span>
        </span>
        <template v-for="(option, index) in chest.options" :key="index">
          <span class="text-(--text-very-muted)">
            {{ index ? "or" : ":" }}
          </span>
          <span
            v-for="(amount, label) in option"
            :key="label"
            class="flex items-center gap-1"
          >
            <img
              :src="get_icon_path(label)"
              class="generic-icon h-6 w-6"
              :alt="label"
              :title="label"
            />
            x{{ amount_text(amount) }}
          </span>
        </template>
        <button
          v-if="editing"
          class="generic-button"
          :aria-label="`Delete ${chest.title}`"
          @click="emit('remove', chest.key)"
        >
          x
        </button>
      </div>
      <slot v-if="at === 0" />
    </div>
  </div>
</template>
