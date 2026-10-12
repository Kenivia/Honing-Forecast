<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { get_icon_path } from "@/Utils/Helpers";

// Provisional: how the last optimizer run opened each chest stack.
const runtime = useRuntimeStore();
const { active_profile } = storeToRefs(useRosterStore());

// the payload sends the profile's chests in order, so the states line up by index
const rows = computed(() => {
  const chests = active_profile.value.chests;
  const states: number[][] = runtime.optimizer.result?.chest_state_arr ?? [];
  if (states.length !== chests.length) return [];
  return chests
    .map((chest, index) => ({ chest, state: states[index] }))
    .filter((row) => row.state.length === row.chest.options.length);
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
      No chests, or the optimizer has not run on them yet.
    </span>
    <div
      v-for="row in rows"
      :key="row.chest.key"
      class="flex flex-wrap items-center gap-2 border-b border-(--border-very-muted) py-1"
    >
      <span>{{ row.chest.title }} x{{ row.chest.count.toLocaleString() }}</span>
      <template v-for="(option, index) in row.chest.options" :key="index">
        <span class="text-(--text-very-muted)">{{ index ? "," : ":" }}</span>
        <span class="flex items-center gap-1">
          {{ row.state[index].toLocaleString() }} as
          <img
            v-for="(_, label) in option"
            :key="label"
            :src="get_icon_path(label)"
            class="generic-icon h-6 w-6"
            :alt="label"
            :title="label"
          />
        </span>
      </template>
    </div>
  </div>
</template>
