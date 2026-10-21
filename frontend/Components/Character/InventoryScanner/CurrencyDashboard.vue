<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { get_icon_path } from "@/Utils/Helpers";
import { Currency } from "./LoadStorage";
import { BAND_NAMES, CURRENCY_LABELS } from "./Manifest";
import { currencies, currency_edits } from "./ScanStore";
import { STATUS_COLORS, STATUS_NAMES } from "./ScannerUIutils";

const props = defineProps<{ currency: Currency }>();

// silver is always roster bound and shards char bound; only gold splits
const BANDS_OF: Record<Currency, number[]> = {
  Silver: [1],
  Gold: [0, 1, 2],
  Shards: [0],
};

const read = computed(() =>
  currencies.value.find((x) => x.currency === props.currency),
);
const edit = computed(() => currency_edits.value[props.currency]);
const bands = computed(() => BANDS_OF[props.currency]);

const status = computed(() =>
  edit.value
    ? "Edited by hand"
    : read.value
      ? STATUS_NAMES[read.value.status]
      : "Not seen yet",
);
const status_color = computed(() =>
  edit.value ? "var(--average)" : STATUS_COLORS[read.value?.status],
);

// what is being typed, started from what is shown whenever another currency is picked
const draft = ref<(number | string | null)[]>([]);
const loaded = ref("");
const changed = computed(() => JSON.stringify(draft.value) !== loaded.value);
watch(
  [() => props.currency, edit, () => !read.value?.bands],
  () => {
    const from = edit.value ?? read.value?.bands;
    draft.value = [0, 1, 2].map((band) => from?.[band] ?? null);
    loaded.value = JSON.stringify(draft.value);
  },
  { immediate: true },
);

const blank = (band: number) =>
  draft.value[band] === null || draft.value[band] === "";
const incomplete = computed(() => bands.value.some(blank));

function save() {
  currency_edits.value[props.currency] = [0, 1, 2].map((band) =>
    bands.value.includes(band)
      ? Math.max(0, Math.round(Number(draft.value[band])))
      : 0,
  );
}
</script>

<template>
  <div
    class="card-shell card-body flex min-h-32 flex-wrap gap-x-10 gap-y-4 text-sm"
  >
    <div class="flex w-60 flex-col gap-2">
      <div class="flex gap-3">
        <img
          :src="get_icon_path(CURRENCY_LABELS[currency])"
          class="generic-icon h-16 w-16 shrink-0"
          :alt="CURRENCY_LABELS[currency]"
        />
        <div class="flex flex-col">
          <span class="text-base text-(--text-bright)">
            {{ CURRENCY_LABELS[currency] }}
          </span>
          <span :style="{ color: status_color }">{{ status }}</span>
        </div>
      </div>
      <div
        v-if="!edit && read?.reason"
        class="border-l pl-2"
        :style="{ borderColor: status_color }"
      >
        {{ read.reason }}
      </div>
    </div>

    <div
      class="grid h-fit grid-cols-[auto_auto_auto] items-center gap-x-4 gap-y-1"
      role="group"
      aria-label="Read and corrected"
    >
      <span></span>
      <span class="text-(--text-bright)">Read</span>
      <span class="text-(--text-bright)">Correct it</span>

      <template v-for="band in bands" :key="band">
        <span class="text-(--text-muted)">{{ BAND_NAMES[band] }}</span>
        <span>{{ read?.bands?.[band]?.toLocaleString() ?? "-" }}</span>
        <input
          v-model="draft[band]"
          type="number"
          min="0"
          class="generic-input h-7! w-32 pl-1"
          :style="blank(band) ? { borderColor: 'var(--warning)' } : {}"
          :aria-label="BAND_NAMES[band]"
        />
      </template>

      <div class="col-start-3 flex flex-col gap-1 pt-1">
        <span v-if="incomplete" class="text-(--warning)">
          Every amount is needed.
        </span>
        <div class="flex gap-2">
          <button
            class="generic-button"
            :class="{
              'cursor-not-allowed! opacity-50': incomplete,
              'bg-blue-600! hover:bg-blue-500!': changed && !incomplete,
            }"
            :disabled="incomplete"
            @click="save"
          >
            Save edit
          </button>
          <button
            v-if="edit"
            class="generic-button"
            @click="delete currency_edits[currency]"
          >
            Remove edit
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
