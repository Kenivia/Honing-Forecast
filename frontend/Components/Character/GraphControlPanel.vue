<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig"
import { storeToRefs } from "pinia"
import { ANNOTATION_LABELS, BUTTON_LABELS, CSS_NAMES } from "@/Utils/Constants"

const store = useRosterStore()
const { roster_config, enabled_annotations } = storeToRefs(store)
</script>
<template>
    <section class="hf-control-panel">
        <div class="hf-card-header">
            <div class="hf-card-title" style="padding-left: 0px">Graph options</div>
        </div>
        <div class="hf-card-body hf-options-body">
            <label class="hf-inline-check">
                <input v-model="roster_config.cumulative_graph" type="checkbox" />
                <span>Cumulative graph</span>
            </label>

            <button
                v-for="(label, index) in BUTTON_LABELS"
                :class="[`hf-graph-tab-${CSS_NAMES[index].toLowerCase()}`, { active: enabled_annotations[index] }]"
                @click="enabled_annotations[index] = !enabled_annotations[index]"
                :key="index"
            >
                {{ label }}
            </button>
        </div>
    </section>
</template>
<style scoped>
.hf-control-panel-btn {
    color: var(--text-muted);
    margin-left: 20px;
}
.hf-control-panel {
    min-width: 0;
    overflow-wrap: break-word;
    word-break: normal;
    align-items: center;
    font-size: 0.85rem;
    width: 100%;
}
.hf-inline-check {
    align-items: center;
    display: flex;
    flex-direction: row;
    padding: 2px 0px;
    border-bottom: 1px solid var(--hf-border-subtle);
}
.hf-options-body {
    padding: 4px 0px;
}

.hf-graph-tab-avg,
.hf-graph-tab-bound,
.hf-graph-tab-roster-bound,
.hf-graph-tab-tradable {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 999px;
    color: var(--hf-text-main);
    padding: 0px 8px;
    font-size: 12px;
    cursor: pointer;
}
.hf-graph-tab-avg.active {
    background: var(--hf-graph-average-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-graph-tab-bound.active {
    background: var(--hf-graph-bound-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-graph-tab-roster-bound.active {
    background: var(--hf-graph-roster-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-graph-tab-tradable.active {
    background: var(--hf-graph-tradable-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}
</style>
