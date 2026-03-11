<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed } from "vue"

const profile_store = useProfilesStore()
const active_profile: CharProfile = profile_store.activeProfile()

const roster_store = uesRosterStore()

const optimizer_worker = active_profile.optimizer_worker_bundle
const optimizer_busy = optimizer_worker.status === "running" || optimizer_worker.status === "pending"
const has_run_optimizer = active_profile.has_run_optimizer
const auto_start_optimizer = active_profile.auto_start_optimizer
const optimizer_progress = optimizer_worker.est_progress_percentage
</script>
<template>
    <section class="hf-card hf-optimizer-card">
        <div class="hf-compact-row">
            <section class="hf-card hf-tap-card">
                <div class="hf-card-header">
                    <div class="hf-card-title"><span class="hf-card-title-dot" />Tap Instructions</div>
                    <span class="hf-card-hint">Top to bottom execution</span>
                </div>
                <div class="hf-card-body">
                    <p class="hf-copy">Empty state means no book and no breath on that tap.</p>
                    <p class="hf-copy" v-if="allowManualState">Progress updates enabled: update progress and success after each relevant outcome.</p>
                    <p class="hf-copy" v-else>Enable progress updates in Controls for better optimization and live progression tracking.</p>

                    <div v-if="orderedUpgradeIndices.length" class="hf-upgrade-editor">
                        <div v-for="(upgradeIndex, orderIndex) in orderedUpgradeIndices" :key="`editor-${upgradeIndex}`" class="hf-upgrade-row">
                            <div class="hf-upgrade-order">{{ orderIndex + 1 }}</div>
                            <div class="hf-upgrade-meta">
                                <div class="hf-upgrade-topline">
                                    <span class="hf-upgrade-name">{{ pieceName(upgradeArr[upgradeIndex]) }}</span>
                                    <span v-if="isFreeTapUpgrade(upgradeIndex)" class="hf-upgrade-free">
                                        Free tap {{ formatSig(freeTapChance(upgradeIndex) * 100, 3) }}%
                                    </span>
                                </div>
                                <span class="hf-upgrade-plan">{{ getNextTapInstruction(upgradeIndex) }}</span>

                                <div class="hf-upgrade-actions-row" v-if="getNextTapPlan(upgradeIndex)?.status === 'active'">
                                    <span class="hf-action-chip tap">
                                        Tap {{ getNextTapPlan(upgradeIndex)?.tapIndex }}/{{ getNextTapPlan(upgradeIndex)?.pityLength }}
                                    </span>
                                    <span v-if="getNextTapPlan(upgradeIndex)?.useJuice" class="hf-action-chip juice">
                                        <img
                                            :src="iconPath(getNextTapPlan(upgradeIndex)?.juiceLabel ?? '')"
                                            :alt="getNextTapPlan(upgradeIndex)?.juiceLabel ?? ''"
                                        />
                                        {{ getNextTapPlan(upgradeIndex)?.juiceLabel }}
                                    </span>
                                    <span v-if="(getNextTapPlan(upgradeIndex)?.bookTier ?? 0) > 0" class="hf-action-chip book">
                                        <img
                                            :src="iconPath(getNextTapPlan(upgradeIndex)?.bookLabel ?? '')"
                                            :alt="getNextTapPlan(upgradeIndex)?.bookLabel ?? ''"
                                        />
                                        {{ getNextTapPlan(upgradeIndex)?.bookLabel }}
                                    </span>
                                    <span
                                        v-if="!getNextTapPlan(upgradeIndex)?.useJuice && (getNextTapPlan(upgradeIndex)?.bookTier ?? 0) === 0"
                                        class="hf-action-chip muted"
                                    >
                                        No juice / no book
                                    </span>
                                </div>
                            </div>

                            <div v-if="allowManualState" class="hf-upgrade-controls">
                                <label>
                                    Progress
                                    <input
                                        type="number"
                                        min="0"
                                        :max="getPityLength(upgradeArr[upgradeIndex])"
                                        :value="getUpgradeProgress(upgradeIndex)"
                                        @input="onUpgradeProgressInput(upgradeIndex, $event)"
                                    />
                                </label>
                                <button class="hf-mini-btn" @click="toggleUpgradeSucceeded(upgradeIndex)">
                                    {{ getUpgradeSucceeded(upgradeIndex) ? "Undo" : "Succeed" }}
                                </button>
                            </div>
                            <div v-else class="hf-upgrade-controls hf-upgrade-controls-readonly">
                                <span class="hf-upgrade-readonly-note">Progress updates disabled</span>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </section>
</template>
