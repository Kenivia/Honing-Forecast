<script setup lang="ts">
import { computed } from "vue";
import { get_icon_path } from "@/Utils/Helpers";
import { Currency, CurrencyResult } from "./LoadStorage";
import { BAND_NAMES, CURRENCY_LABELS } from "./Manifest";
import { currencies } from "./ScanStore";
import { STATUS_COLORS } from "./ScannerUIutils";

// as the game's bar at the top of the screen has them
const ORDER: Currency[] = ["Silver", "Gold", "Shards"];

// before the first result, nothing is known
const shown = computed(() =>
  ORDER.map(
    (currency): CurrencyResult =>
      currencies.value.find((x) => x.currency === currency) ?? {
        currency,
        status: "Pending",
        reason: "Not seen yet.",
        value: null,
        bands: null,
      },
  ),
);
</script>

<template>
  <div
    class="card-shell flex flex-wrap items-start gap-x-8 gap-y-2 px-3 py-2 text-sm"
    role="group"
    aria-label="Currencies"
  >
    <div
      v-for="item in shown"
      :key="item.currency"
      class="flex max-w-80 flex-col gap-1"
    >
      <span
        class="flex w-fit items-center gap-1 border-b-2"
        :style="{ borderColor: STATUS_COLORS[item.status] }"
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
      </span>
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
            .join(" · ")
        }}
      </span>
      <span v-else-if="item.reason" class="text-xs text-(--text-muted)">
        {{ item.reason }}
      </span>
    </div>
  </div>
</template>
