<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, T4_MATS_LABELS, ANNOTATION_COLORS, ANNOTATION_POSITIONS, ANNOTATION_LABELS } from "@/Utils/Constants"
import { TreatmentPlan } from "@/Stores/CharacterProfile"
import { has_upgrades_in_range, metric_to_text } from "@/Utils/Helpers"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { WasmOp } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { useRosterStore } from "@/Stores/RosterConfig"
import { computed, ref, watchEffect } from "vue"
import { build_payload } from "@/WasmInterface/PayloadBuilder"
import { input_column_to_num } from "@/Utils/InputColumn"
import { start_all_workers } from "../CharWorkerUtils"
import { RouterLink } from "vue-router"

const { active_profile } = storeToRefs(useRosterStore())
const { roster_config, active_roster_mats_owned, active_tradable_mats_owned, enabled_annotations } = storeToRefs(useRosterStore())
</script>

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Detailed inputs</div>
        </div>
        <div class="hf-card-body">
            <div class="hf-dist-scroll">
                <div class="hf-dist-stack">
                    <div class="hf-dist-graphs">
                        <div class="hf-table-title-row"></div>
                        <div
                            v-if="
                                ALL_LABELS[active_profile.tier].length == active_profile.bound_budgets[active_profile.tier].data.length &&
                                // active_profile.optimizer_worker_bundle.result &&
                                active_profile.histogram_worker_bundle.result &&
                                active_profile.histogram_worker_bundle.result &&
                                active_profile.material_rerender_trigger
                            "
                            style="display: contents"
                        ></div>
                    </div>
                    <div class="hf-dist-graphs">
                        <div class="hf-mats-row">
                            <div class="hf-metric-label">
                                {{ total_market_gold_text }}
                            </div>
                            <div class="hf-metric-status">
                                {{ metric_to_text(active_profile.histogram_worker_bundle.result?.metrics_arr[0]) ?? "No Result yet" }}

                                <span style="font-size: 16px">
                                    {{ total_market_gold_suffix }}
                                </span>
                            </div>
                        </div>

                        <div
                            v-if="active_profile.optimizer_treatment_plan == TreatmentPlan.TreatRosterAsBound && active_profile.auto_start_optimizer"
                            class="hf-mats-row"
                        >
                            <div class="hf-metric-label" style="font-size: 16px; color: var(--hf-text-muted)">
                                {{ total_tradable_gold_text }}
                            </div>
                            <div class="hf-metric-status" style="color: var(--hf-text-muted)">
                                {{
                                    metric_to_text(
                                        active_profile.histogram_worker_bundle.result?.metrics_arr[0] -
                                            active_profile.histogram_worker_bundle.result?.metrics_arr[1],
                                    ) ?? "No Result yet"
                                }}

                                <span style="font-size: 16px">
                                    {{ total_tradable_gold_suffix }}
                                </span>
                            </div>
                        </div>
                        <div style="display: flex; flex-direction: row; grid-column: 1 / span 6; align-items: center">
                            <span class="optimizer-progress-label"
                                >Optimizer progress: {{ active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(2) }}%
                            </span>
                            <div class="progress-bar">
                                <div class="progress-fill" :style="{ width: `${active_profile.optimizer_worker_bundle.est_progress_percentage}%` }" />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <Teleport to="body">
            <div v-if="show_special_guide" class="hf-modal-overlay" @click="show_special_guide = false">
                <div class="hf-popup" @click.stop>
                    <span style="font-size: 30px; color: var(--hf-text-bright)">
                        Short answer: Save Special Leaps and convert to Serca, unless you are tapping +25
                    </span>
                    <span style="font-size: 16px; color: var(--hf-text-muted)"> If you're not +20 yet, use it in T4. </span>
                    <img src="/Special convert chart.png" alt="Special convert chart" />
                </div>
            </div>
        </Teleport>
    </section>
    <span>
        The above results assumes that you follow the optimal
        <RouterLink
            class="hf-metric-label"
            style="text-decoration: underline"
            :to="{ name: 'instructions', params: { characterName: active_profile.char_name } }"
        >
            Taps Instructions
        </RouterLink>
    </span>
</template>
<style scoped>
.special-convert-guide {
    color: var(--hf-free-tap);
    font-size: 12px;
    text-decoration-line: underline;
}

.special-convert-guide:hover {
    color: var(--hf-free-tap-faded);
    font-size: 12px;
}
.progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    overflow: hidden;
    /* grid-column: 3 / span 3; */
}
.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--hf-gold-dim), var(--hf-gold));
    transition: width 0.1s ease;
}
.optimizer-progress-label {
    display: flex;
    flex-direction: column;
    font-size: 16px;
    /* grid-column: 1 / span 2; */
    text-align: right;
    padding: 6px;
    text-wrap-mode: nowrap;
}

.hf-metric-label {
    grid-column: span 4;
    width: 100%;
    gap: 30px;
    color: var(--hf-gold);
    font-size: 20px;
    text-align: right;
    padding-right: 8px;
    justify-content: center;
}

.smaller-label {
    font-size: 12px;
    color: var(--hf-text-muted);
}

.hf-metric-status {
    grid-column: 5 / span 2;
    width: 100%;
    gap: 30px;
    color: var(--hf-gold);
    font-size: 30px;
    text-align: left;
    padding-right: 8px;
    justify-content: center;
    text-wrap-mode: nowrap;
}

.hf-question-mark {
    margin-left: 4px;
    /* padding-right: 12px;
    padding-left: 8px; */

    width: 16px; /* Align with the two icon rows visually */
    height: 16px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;

    font-weight: bold;
    background-color: AccentColor;
    color: AccentColorText;
    font-size: 12px;
}
.hf-bound-header {
    color: var(--hf-graph-bound-color);
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
    --hf-dist-columns: 160px 90px 120px 120px 120px 320px;
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
    border-bottom: 1px solid var(--separator-color);
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
</style>
