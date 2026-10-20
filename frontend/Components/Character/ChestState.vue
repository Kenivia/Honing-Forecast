<script setup lang="ts">
import { computed } from "vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { get_icon_path } from "@/Utils/Helpers";
import { amount_text, type Chest } from "@/Utils/Chest";
import ChestIcon from "@/Components/Common/ChestIcon.vue";

// Provisional: how each chest stack is opened in what the histogram last evaluated, which is
// the optimizer's latest state, the ones it reports while running included.
const props = defineProps<{ chests: Chest[] }>();
const runtime = useRuntimeStore();

// a state is found by its chest's key, so one from before the chests changed still fits
const rows = computed(() => {
  const states: Map<string, number[]> | undefined =
    runtime.histogram.result?.state_bundle.chest_states;
  return props.chests
    .map((chest) => ({ chest, state: states?.get(chest.key) }))
    .filter((row) => row.state?.length === row.chest.options.length);
});
</script>

<template>
  <div
    class="card-shell card-body text-sm"
    role="group"
    aria-label="Chest opening"
  >
    <div class="card-title">Chest opening (provisional)</div>
    <span v-if="!rows.length" class="text-(--text-muted)">
      No chests with a choice to make.
    </span>
    <div
      v-for="row in rows"
      :key="row.chest.key"
      class="flex flex-wrap items-center gap-2 border-b border-(--border-very-muted) py-1"
    >
      <ChestIcon :chest="row.chest" />
      <span>x{{ row.chest.count.toLocaleString() }}</span>
      <template v-for="(option, index) in row.chest.options" :key="index">
        <span class="text-(--text-very-muted)">{{ index ? "," : ":" }}</span>
        <span class="flex items-center gap-1">
          {{ row.state[index].toLocaleString() }} as
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
        </span>
      </template>
    </div>
  </div>
</template>
