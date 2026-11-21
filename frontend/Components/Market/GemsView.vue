<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useTimeAgo } from "@vueuse/core";
import {
  fetch_gems,
  gem_data,
  gem_failed,
  gem_fetching,
} from "@/Utils/GemDataFetcher";

const props = defineProps<{ selected_roster_id: string }>();
const { roster_config } = storeToRefs(useRosterStore());
const region = computed(
  () => roster_config.value.all_regions[props.selected_roster_id],
);
watch(region, (r) => fetch_gems(r), { immediate: true });

const KINDS = [
  { item: "doomfire", label: "Doomfire" },
  { item: "blazing", label: "Blazing" },
];
const LEVELS = [10, 9, 8, 7, 6, 5];
// three gems of one level fuse into one of the next, so a Lv. M gem is 3^(N-M) of a Lv. N
const value_level = ref(10);

const searches = computed(() => gem_data.value[region.value]?.searches);
function lowest(item: string, level: number): number | null {
  const s = searches.value?.find(
    (s) => s.item === item && s.gem_level === level,
  );
  const prices = (s?.rows ?? [])
    .map((r) => r.buy_now)
    .filter((p): p is number => p !== null);
  return prices.length ? Math.min(...prices) : null;
}
const as_level = (price: number, level: number) =>
  price * 3 ** (value_level.value - level);

// the cheapest way to a Lv. N gem, per kind
const best = computed(() =>
  KINDS.map((k) => {
    let out: { level: number; price: number; value: number } | null = null;
    for (const level of LEVELS) {
      const price = lowest(k.item, level);
      if (price === null) continue;
      const value = as_level(price, level);
      if (!out || value < out.value) out = { level, price, value };
    }
    return { ...k, best: out };
  }),
);

const scanned_at = computed(() => {
  const ats = (searches.value ?? []).map((s) => Date.parse(s.at));
  return ats.length ? Math.max(...ats) : 0;
});
const scanned_ago = useTimeAgo(scanned_at);
const fmt = (n: number) => Math.round(n).toLocaleString("en-US");
</script>

<template>
  <div class="flex w-max max-w-full flex-col gap-2">
    <div v-if="region === 'Custom'" class="text-(--text-muted)">
      Gem prices are only scanned for NAE and EUC.
    </div>
    <div v-else-if="!searches" class="text-(--text-muted)">
      {{
        gem_fetching
          ? "Fetching gem prices..."
          : gem_failed
            ? "Failed to fetch gem prices"
            : ""
      }}
    </div>
    <div v-else-if="!searches.length" class="text-(--text-muted)">
      The latest scan has no gems.
    </div>
    <template v-else>
      <div class="card-shell" role="group" aria-label="Highest value">
        <div class="card-header">
          <span class="card-title">Highest value</span>
          <label class="flex items-center gap-2 text-sm text-(--text-muted)">
            Value as Lv.
            <select
              v-model.number="value_level"
              class="selector"
              aria-label="Value level"
            >
              <option v-for="n in 10" :key="n" :value="n">{{ n }}</option>
            </select>
          </label>
        </div>
        <div class="card-body flex flex-row flex-wrap gap-8">
          <div v-for="k in best" :key="k.item" class="flex flex-col">
            <span class="text-(--text-muted)">{{ k.label }}</span>
            <template v-if="k.best">
              <span class="text-lg text-(--gold)">
                Buy Lv. {{ k.best.level }}
              </span>
              <span> {{ fmt(k.best.value) }} per Lv. {{ value_level }} </span>
              <span class="annotation mt-1">
                {{ fmt(k.best.price) }} each
              </span>
            </template>
            <span v-else class="text-(--text-very-muted)">No listings</span>
          </div>
        </div>
      </div>

      <div class="card-shell" role="group" aria-label="Gem prices">
        <table
          class="card-body border-separate border-spacing-x-6 border-spacing-y-1"
        >
          <thead>
            <tr class="text-left text-(--text-muted)">
              <th class="font-normal">Level</th>
              <th v-for="k in KINDS" :key="k.item" class="font-normal">
                {{ k.label }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="level in LEVELS" :key="level">
              <td>Lv. {{ level }}</td>
              <td v-for="k in best" :key="k.item">
                <template v-if="lowest(k.item, level) !== null">
                  <span
                    :class="
                      k.best?.level === level
                        ? 'text-(--gold)'
                        : 'text-(--text-main)'
                    "
                    >{{ fmt(lowest(k.item, level)!) }}</span
                  >
                  <span class="annotation ml-2">
                    {{ fmt(as_level(lowest(k.item, level)!, level)) }} per Lv.
                    {{ value_level }}
                  </span>
                </template>
                <span v-else class="text-(--text-very-muted)">-</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <span class="annotation">Lowest buy-now, scanned {{ scanned_ago }}</span>
    </template>
  </div>
</template>
