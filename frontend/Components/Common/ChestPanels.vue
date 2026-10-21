<script setup lang="ts">
import { computed } from "vue";
import { BAND_COLORS, BAND_NAMES } from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import { amount_text, state_fits, type Chest } from "@/Utils/Chest";
import ChestIcon from "./ChestIcon.vue";

// The chests still to be opened, as two lists side by side in one card, the selection one in two
// columns while obtain-all is hidden. Which chest goes where is the page's to say. The default
// slot is the foot of the selection list, `controls` sits in the header. With `states`, a
// selection chest whose state fits shows how many go to each option (provisional: the
// histogram's latest state).
const props = defineProps<{
  selection: Chest[];
  obtain_all: Chest[];
  obtain_all_title?: string;
  editing?: boolean;
  states?: Map<string, number[]>;
}>();
const emit = defineEmits<{
  count: [key: string, event: Event];
  remove: [key: string];
}>();
const open = defineModel<boolean>("open", { default: true });
const show_obtain_all = defineModel<boolean>("show_obtain_all", {
  default: false,
});

const split = (chest: Chest) => {
  const state = props.states?.get(chest.key);
  return state_fits(chest, state) ? state : undefined;
};

const panels = computed(() => [
  {
    title: "Selection chests",
    chests: props.selection,
    columns: show_obtain_all.value ? 1 : 2,
  },
  ...(show_obtain_all.value
    ? [
        {
          title: props.obtain_all_title ?? "Obtain-all chests",
          chests: props.obtain_all,
          columns: 1,
        },
      ]
    : []),
]);
// count, chest, options, delete
const row_span = computed(() => (props.editing ? 4 : 3));
</script>

<template>
  <div class="card-shell text-sm">
    <div class="card-header justify-start! gap-4">
      <button
        class="card-title cursor-pointer"
        :aria-expanded="open"
        @click="open = !open"
      >
        {{ open ? "▾" : "▸" }} Chests
      </button>
      <slot name="controls" />
      <label class="flex items-center gap-1">
        <input v-model="show_obtain_all" type="checkbox" />
        Show obtain-all chests ({{ obtain_all.length }})
      </label>
    </div>
    <div
      class="card-body flex flex-wrap items-start gap-x-8 gap-y-4"
      :class="{ 'invisible h-0 overflow-hidden py-0!': !open }"
      :inert="!open"
    >
      <div
        v-for="(panel, at) in panels"
        :key="panel.title"
        class="flex max-w-full flex-col gap-1"
        role="group"
        :aria-label="panel.title"
      >
        <div class="font-semibold text-(--text-bright)">
          {{ panel.title }}
          <span v-if="at === 0 && states" class="annotation font-normal">
            (split is provisional)
          </span>
        </div>
        <span v-if="!panel.chests.length" class="text-(--text-muted)">
          None.
        </span>
        <div
          v-else
          class="grid max-h-80 items-center gap-x-3 overflow-auto"
          :style="{
            gridTemplateColumns: `repeat(${panel.columns * row_span}, auto)`,
          }"
        >
          <div
            v-for="chest in panel.chests"
            :key="chest.key"
            class="grid grid-cols-subgrid items-center border-b border-(--border-very-muted) py-1"
            :class="{ 'odd:pr-8': panel.columns === 2 }"
            :style="{ gridColumn: `span ${row_span}` }"
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
              <ChestIcon :chest="chest" />
              <span
                class="annotation"
                :style="{ color: `var(${BAND_COLORS[chest.band]})` }"
              >
                {{ BAND_NAMES[chest.band] }}
              </span>
            </span>
            <span class="flex flex-wrap items-center gap-2">
              <template v-for="(option, index) in chest.options" :key="index">
                <span class="text-(--text-very-muted)">
                  {{ index ? (at === 0 && split(chest) ? "," : "or") : ":" }}
                </span>
                <span v-if="at === 0 && split(chest)">
                  {{ split(chest)[index].toLocaleString() }} as
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
            </span>
            <button
              v-if="editing"
              class="generic-button"
              :aria-label="`Delete ${chest.title}`"
              @click="emit('remove', chest.key)"
            >
              x
            </button>
          </div>
        </div>
        <slot v-if="at === 0" />
      </div>
    </div>
  </div>
</template>
