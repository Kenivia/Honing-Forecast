<script setup lang="ts">
import { computed } from "vue";
import { get_icon_path } from "@/Utils/Helpers";
import { Currency, CurrencyResult } from "./LoadStorage";
import { BAND_NAMES, CURRENCY_LABELS, CURRENCY_ORDER } from "./Manifest";
import { currency_edits, shown_currency } from "./ScanStore";
import { status_border } from "./ScannerUIutils";

defineProps<{ active: Currency | null }>();
const emit = defineEmits<{ hover: [Currency]; pick: [Currency] }>();

// what the outlines of currencies and slots mean, as squares as tall as a currency's
const KEYS = [
  { status: "Good", edited: false, label: "Good" },
  { status: "NeedHover", edited: false, label: "Needs hover" },
  { status: "Error", edited: false, label: "Error" },
  { status: "Good", edited: true, label: "Set by hand" },
].map(({ status, edited, label }) => {
  const border = status_border(status, edited);
  const side = `calc(1.5rem + 2 * ${border.borderWidth})`;
  return { label, style: { ...border, width: side, height: side } };
});

// one not seen yet needs a hover of the game's currency bar
const shown = computed(() =>
  CURRENCY_ORDER.map(
    (currency): CurrencyResult =>
      shown_currency(currency) ?? {
        currency,
        status: "NeedHover",
        reason: "Not seen yet.",
        value: null,
        bands: null,
      },
  ),
);
</script>

<template>
  <div
    class="flex flex-wrap items-start gap-x-8 gap-y-2 text-sm"
    role="group"
    aria-label="Currencies"
  >
    <div
      v-for="item in shown"
      :key="item.currency"
      class="flex max-w-80 flex-col gap-1"
    >
      <button
        class="flex w-fit cursor-pointer items-center gap-1 px-1.5 py-0.5"
        :style="{
          ...status_border(item.status, !!currency_edits[item.currency]),
          outline:
            active === item.currency ? '3px dashed var(--text-bright)' : 'none',
        }"
        @mouseenter="emit('hover', item.currency)"
        @click="emit('pick', item.currency)"
      >
        <span :class="{ 'text-(--text-muted)': item.value == null }">
          {{ item.value?.toLocaleString() ?? "?" }}
        </span>
        <img
          :src="get_icon_path(CURRENCY_LABELS[item.currency])"
          class="generic-icon h-5 w-5"
          :alt="CURRENCY_LABELS[item.currency]"
          :title="CURRENCY_LABELS[item.currency]"
        />
      </button>
      <span
        v-if="item.currency === 'Gold' && item.bands"
        class="text-xs whitespace-nowrap text-(--text-muted)"
      >
        {{
          item.bands
            .map(
              (amount, band) =>
                `${BAND_NAMES[band]} ${amount.toLocaleString()}`,
            )
            .join(", ")
        }}
      </span>
      <span v-else-if="item.reason" class="text-xs text-(--text-muted)">
        {{ item.reason }}
      </span>
    </div>
    <!-- equal columns, as wide as the longest label -->
    <div class="ml-40 grid auto-cols-fr grid-flow-col items-center gap-x-4">
      <div
        v-for="key in KEYS"
        :key="key.label"
        class="flex items-center gap-1.5 text-(--text-muted)"
      >
        <span class="shrink-0" :style="key.style" />
        {{ key.label }}
      </div>
    </div>
  </div>
</template>
