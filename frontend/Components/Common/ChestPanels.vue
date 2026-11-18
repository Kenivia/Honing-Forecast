<script setup lang="ts">
import { computed } from "vue";
import { BAND_COLORS, BAND_NAMES } from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import { amount_text, state_fits, type Chest } from "@/Utils/Chest";
import ChestIcon from "./ChestIcon.vue";

// The chests still to be opened, as two lists side by side in one card; each list, and each
// column in a list, is half the card, one under another below 480px. Which chest goes where is
// the page's to say. The default slot is the foot of the card, below both lists, `controls` sits
// in the header. With `states`, a selection chest whose state fits shows how many go to each
// option (provisional: the histogram's latest state). A `forbidden` chest is shown greyed, with
// its allow button; only a roster or tradable chest can be forbidden, editing or not.
const props = defineProps<{
  selection: Chest[];
  obtain_all: Chest[];
  selection_title?: string;
  obtain_all_title?: string;
  editing?: boolean;
  hide_count?: boolean;
  states?: Map<string, number[]>;
  forbidden?: string[];
}>();
const emit = defineEmits<{
  count: [key: string, event: Event];
  remove: [key: string];
  forbid: [key: string];
}>();
const is_forbidden = (chest: Chest) => !!props.forbidden?.includes(chest.key);
const can_forbid = (chest: Chest) => is_forbidden(chest) || chest.band > 0;
const BUTTON =
  "flex h-7 w-7 items-center justify-center rounded border border-(--border-main) text-(--text-muted) hover:text-(--text-bright)";
// a public svg drawn in the text color
const mask = (name: string) => ({
  mask: `url(/Icons/${name}.svg) center / contain no-repeat`,
});
// half the width, one column below 480px
const HALF = "min(100%, max(480px, calc(50% - 1rem)))";
const open = defineModel<boolean>("open", { default: true });
const show_obtain_all = defineModel<boolean>("show_obtain_all", {
  default: false,
});
// fits its content by default, one fixed width otherwise, where a list alone keeps its half
const wide = computed(() => props.editing || show_obtain_all.value);

const split = (chest: Chest) => {
  const state = props.states?.get(chest.key);
  return state_fits(chest, state) ? state : undefined;
};
// the options to show, with the split in the first list; an option split 0 is left out
const shown = (chest: Chest, at: number) => {
  const state = at === 0 ? split(chest) : undefined;
  return chest.options
    .map((option, index) => ({ option, n: state?.[index] }))
    .filter((item) => item.n !== 0);
};

const panels = computed(() => [
  {
    title: props.selection_title ?? "Selection chests",
    chests: props.selection,
  },
  ...(show_obtain_all.value
    ? [
        {
          title: props.obtain_all_title ?? "Obtain-all chests",
          chests: props.obtain_all,
        },
      ]
    : []),
]);
</script>

<template>
  <div class="card-shell text-sm" :class="{ 'w-258!': wide }">
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
        Show all chests ({{ obtain_all.length }})
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
        class="flex max-w-full min-w-0 flex-col gap-1"
        :style="{ flex: `${wide ? 0 : 1} 1 ${HALF}` }"
        role="group"
        :aria-label="panel.title"
      >
        <div class="font-semibold text-(--text-bright)">
          {{ panel.title }}
        </div>
        <span v-if="!panel.chests.length" class="text-(--text-muted)">
          None.
        </span>
        <div
          v-else
          class="grid max-h-80 gap-x-8 overflow-auto"
          :style="{
            gridTemplateColumns: `repeat(auto-fill, minmax(${HALF}, 1fr))`,
          }"
        >
          <div
            v-for="chest in panel.chests"
            :key="chest.key"
            class="grid items-center gap-x-3 border-b border-(--border-very-muted) py-1"
            :class="
              hide_count
                ? 'grid-cols-[4rem_minmax(0,1fr)_auto]'
                : 'grid-cols-[4rem_4rem_minmax(0,1fr)_auto]'
            "
          >
            <input
              v-if="!hide_count"
              type="number"
              min="0"
              class="generic-input h-7! w-16 pl-1"
              :class="{ 'opacity-50': is_forbidden(chest) }"
              :disabled="!editing || is_forbidden(chest)"
              :aria-label="`${chest.title} count`"
              :value="chest.count"
              @change="emit('count', chest.key, $event)"
            />
            <span
              class="flex flex-col"
              :class="{ 'opacity-50': is_forbidden(chest) }"
            >
              <ChestIcon :chest="chest" />
              <span
                class="annotation"
                :style="{ color: `var(${BAND_COLORS[chest.band]})` }"
              >
                {{ BAND_NAMES[chest.band] }}
              </span>
            </span>
            <span v-if="is_forbidden(chest)" class="text-(--text-very-muted)">
              Forbidden on this character
            </span>
            <span v-else class="flex flex-wrap items-center gap-2">
              <template
                v-for="({ option, n }, index) in shown(chest, at)"
                :key="index"
              >
                <span
                  v-if="index || n === undefined"
                  class="text-(--text-very-muted)"
                >
                  {{ index ? (n === undefined ? "or" : ",") : ":" }}
                </span>
                <span v-if="n !== undefined">
                  {{ index ? "open" : "Open" }} {{ n.toLocaleString() }}
                  {{ n === 1 ? "box" : "boxes" }} as
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
                  <span :class="{ 'opacity-50': n !== undefined }">
                    x{{ amount_text(amount) }}
                  </span>
                </span>
              </template>
            </span>
            <span class="flex gap-1">
              <button
                v-if="can_forbid(chest)"
                :class="BUTTON"
                :aria-label="
                  is_forbidden(chest)
                    ? `Allow ${chest.title} on this character`
                    : `Forbid ${chest.title} on this character`
                "
                :title="is_forbidden(chest) ? 'Allow' : 'Forbid'"
                @click="emit('forbid', chest.key)"
              >
                <template v-if="is_forbidden(chest)">✓</template>
                <span
                  v-else
                  class="h-4 w-4 bg-current"
                  :style="mask('ban-solid-full')"
                />
              </button>
              <!-- keeps the delete buttons in line -->
              <span v-else-if="editing" class="w-7" />
              <button
                v-if="editing"
                :class="BUTTON"
                :aria-label="`Delete ${chest.title}`"
                title="Delete"
                @click="emit('remove', chest.key)"
              >
                <span
                  class="h-4 w-4 bg-current"
                  :style="mask('trash-solid-full')"
                />
              </button>
            </span>
          </div>
        </div>
      </div>
      <div v-if="$slots.default" class="w-full">
        <slot />
      </div>
    </div>
  </div>
</template>
