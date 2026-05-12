<!-- <script setup lang="ts">
import {
    ALL_LABELS,
    GRAPH_COLORS,
    T4_MATS_LABELS,
    ANNOTATION_COLORS,
    ANNOTATION_POSITIONS,
    ANNOTATION_LABELS,
    PIECE_NAMES,
} from "@/Utils/Constants";
import { TreatmentPlan } from "@/Stores/CharacterProfile";
import { get_icon_path, has_upgrades_in_range, metric_to_text } from "@/Utils/Helpers";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { UpgradeStatus, WasmOp } from "@/Utils/Interfaces";
import MaterialGraph from "./MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed, ref, watchEffect } from "vue";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { input_column_to_num } from "@/Utils/InputColumn";
import { start_all_workers } from "../CharWorkerUtils";
import { RouterLink } from "vue-router";
import { get_upgrade_map, to_upgrade_key } from "@/Utils/KeyedUpgrades";
import DetailedInputRow from "./DetailedInputRow.vue";

const { active_profile } = storeToRefs(useRosterStore());

const upgrade_map = computed(() =>
    get_upgrade_map(
        active_profile.value.optimizer_worker_bundle.result?.upgrade_arr ?? null,
        active_profile.value.tier,
    ),
);
const lowest_normal = computed(() =>
    PIECE_NAMES.map(
        (piece_name, piece_type) =>
            upgrade_map.value.get(
                to_upgrade_key(
                    piece_type,
                    (active_profile.value.normal_grid[piece_type] as UpgradeStatus[]).findIndex(
                        (value) => value == UpgradeStatus.Want,
                    ),
                    true,
                    active_profile.value.tier,
                ),
            ) ?? null,
    ),
);
const lowest_adv = computed(() =>
    PIECE_NAMES.map(
        (_, piece_type) =>
            upgrade_map.value.get(
                to_upgrade_key(
                    piece_type,
                    (active_profile.value.adv_grid[piece_type] as UpgradeStatus[]).findIndex(
                        (value) => value == UpgradeStatus.Want,
                    ),
                    false,
                    active_profile.value.tier,
                ),
            ) ?? null,
    ),
);

// console.log(upgrade_map.value, lowest_normal.value, upgrade_map.value.get("0,18,true,0"))
</script>

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title">Detailed inputs</div>
        </div>
        <div class="hf-card-body">
            <div class="hf-dist-scroll">
                <div class="hf-dist-stack">
                    <div class="hf-dist-graphs">
                        <div class="hf-table-title-row"></div>
                        <div v-for="(piece_name, index) in PIECE_NAMES" :key="piece_name" class="hf-mats-row">
                            <div class="hf-equip-label">
                                <span>{{ piece_name }}</span>
                                <img :src="get_icon_path(piece_name)" :alt="piece_name" />
                            </div>
                            <DetailedInputRow v-if="lowest_normal[index]" :upgrade="lowest_normal[index]" />

                            <DetailedInputRow v-if="lowest_adv[index]" :upgrade="lowest_adv[index]" />
                        </div>
                    </div>
                    <div class="hf-dist-graphs"></div>
                </div>
            </div>
        </div>
    </section>
</template>
<style scoped>
.hf-bound-header {
    color: var(--hf-bound);
    text-align: right;
    padding-right: 8px;
}

.hf-average-header {
    color: var(--hf-graph-average-color);
    text-align: center;
}

.hf-gold-header {
    color: var(--hf-gold);
    text-align: center;
}
.hf-gold-header-suffix {
    color: var(--text-very-muted);
    font-size: 12px;
    min-width: 0;
    text-align: left;
    justify-self: right;
    margin-left: auto;
    position: absolute;
    transform: translateX(100%) translateY(4px);
    right: 22px;
}
.hf-hover-hint {
    text-align: center;
    color: var(--hf-text-muted);
    font-size: 11px;
}

.hf-bound-select {
    min-width: 0;
}

.hf-analysis-pane {
    width: min(100%, 992px);
    overflow-x: visible;
    overflow-y: visible;
}

.hf-dist-scroll {
    width: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    /* scrollbar-gutter: stable; */
}

.hf-dist-stack {
    display: flex;
    flex-direction: column;
    width: max-content;
    min-width: 100%;
}

.hf-analysis-pane :deep(.hf-card-header) {
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
}

.hf-analysis-pane :deep(.hf-card-header > div) {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.hf-analysis-tab {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 999px;
    background: rgba(10, 13, 19, 0.48);
    color: var(--hf-text-main);
    padding: 6px 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
}

.hf-analysis-tab.active {
    background: rgba(212, 179, 90, 0.22);
    border-color: rgba(212, 179, 90, 0.6);
    color: var(--hf-text-bright);
}
.hf-dist-graphs {
    --hf-dist-columns: 160px 120px;
    display: grid;
    grid-template-columns: var(--hf-dist-columns);
    align-items: center;
    justify-content: start;
    row-gap: 0;
    min-width: max-content;
}

.hf-table-title-row,
.hf-mats-row {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: var(--hf-dist-columns);
    align-items: center;
    border-bottom: 1px solid var(--border-main);
    min-height: 0;
}

.hf-mats-row.disabled {
    opacity: 0.5;
}

@media (max-width: 900px) {
    .hf-dist-graphs {
        --hf-dist-columns: 100px 70px 112px 78px 78px 150px;
        min-width: max-content;
        width: auto;
    }

    .hf-metric-label {
        grid-column: 1 / span 3;
        font-size: 16px;
        text-align: right;
        gap: 0;
    }

    .hf-metric-status {
        grid-column: 4 / span 2;
        font-size: 22px;
        text-align: left;
        gap: 0;
    }

    .hf-bound-select {
        width: 100%;
        font-size: 11px;
    }

    .hf-bound-header,
    .hf-average-header,
    .hf-gold-header {
        font-size: 11px;
    }

    .hf-average-header,
    .hf-gold-header {
        text-align: center;
    }

    .hf-table-title-row {
        font-size: 11px;
    }

    .hf-hover-hint {
        font-size: 10px;
    }

    .hf-mats-row :deep(.hf-material-cell) {
        --hf-cell-input-width: 64px;
        --hf-cell-label-width: 88px;
        --hf-cell-icon-size: 20px;
    }
}
</style> -->
<template>a</template>
