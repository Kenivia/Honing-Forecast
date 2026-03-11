<template>
    <div class="hf-shell">
        <div class="hf-app-wrapper">
            <div class="hf-main-content">
                <div class="hf-top-grid">
                    <section class="hf-main-stage">
                        <div class="hf-honing-row">
                            <section class="hf-card hf-normal-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Normal Honing</div>
                                    <span class="hf-card-hint">Build target grid</span>
                                </div>
                                <div class="hf-card-body">
                                    <div class="hf-grid-content">
                                        <div class="hf-label-col">
                                            <div class="hf-label-row" />
                                            <div v-for="piece in PIECE_NAMES" :key="piece" class="hf-label-row">
                                                <div class="hf-equip-label">
                                                    <span>{{ piece }}</span>
                                                    <img :src="iconPath(piece)" :alt="piece" />
                                                </div>
                                            </div>
                                        </div>
                                        <div ref="normalGridScrollRef" class="hf-grid-scroll">
                                            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${TOP_COLS}, 26px)` }">
                                                <button
                                                    v-for="col in TOP_COLS"
                                                    :key="`top-col-${col}`"
                                                    class="hf-cell hf-cell-header"
                                                    :class="{ selected: isTopColChecked(col - 1) }"
                                                    @click="toggleTopCol(col - 1)"
                                                >
                                                    +{{ col }}
                                                </button>
                                            </div>
                                            <div
                                                v-for="row in TOP_ROWS"
                                                :key="`top-row-${row}`"
                                                class="hf-cell-grid"
                                                :style="{ gridTemplateColumns: `repeat(${TOP_COLS}, 26px)` }"
                                            >
                                                <button
                                                    v-for="col in TOP_COLS"
                                                    :key="`top-${row}-${col}`"
                                                    class="hf-cell"
                                                    :class="{ selected: topGrid[row - 1][col - 1] }"
                                                    @pointerdown.prevent="startTopDrag(row - 1, col - 1, $event)"
                                                    @pointerenter="dragTopCell(row - 1, col - 1)"
                                                    @click.prevent="onTopCellClick(row - 1, col - 1, $event)"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>

                            <section class="hf-card hf-advanced-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Advanced Honing</div>
                                    <span class="hf-card-hint">Juice on Grace assumed</span>
                                </div>
                                <div class="hf-card-body">
                                    <div class="hf-grid-content hf-grid-content-compact">
                                        <div class="hf-label-col hf-label-col-compact">
                                            <div class="hf-label-row" />
                                            <div v-for="piece in PIECE_NAMES" :key="`adv-${piece}`" class="hf-label-row">
                                                <div class="hf-equip-label hf-equip-label-compact">
                                                    <img :src="iconPath(piece)" :alt="piece" />
                                                </div>
                                            </div>
                                        </div>
                                        <div>
                                            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${BOTTOM_COLS}, 26px)` }">
                                                <button
                                                    v-for="(bonus, idx) in [10, 20, 30, 40]"
                                                    :key="`bottom-col-${bonus}`"
                                                    class="hf-cell hf-cell-header"
                                                    :class="{ selected: isBottomColChecked(idx) }"
                                                    @click="toggleBottomCol(idx)"
                                                >
                                                    +{{ bonus }}
                                                </button>
                                            </div>
                                            <div
                                                v-for="row in BOTTOM_ROWS"
                                                :key="`bottom-row-${row}`"
                                                class="hf-cell-grid"
                                                :style="{ gridTemplateColumns: `repeat(${BOTTOM_COLS}, 26px)` }"
                                            >
                                                <button
                                                    v-for="col in BOTTOM_COLS"
                                                    :key="`bottom-${row}-${col}`"
                                                    class="hf-cell"
                                                    :class="{ selected: bottomGrid[row - 1][col - 1] }"
                                                    @pointerdown.prevent="startBottomDrag(row - 1, col - 1, $event)"
                                                    @pointerenter="dragBottomCell(row - 1, col - 1)"
                                                    @click.prevent="onBottomCellClick(row - 1, col - 1, $event)"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>
                        </div>

                        <div class="hf-ops-row">
                            <section class="hf-card hf-optimizer-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Action Queue</div>
                                    <span class="hf-card-hint">Optimize, then follow next steps</span>
                                </div>
                                <div class="hf-card-body">
                                    <div class="optimizer-card">
                                        <button
                                            class="hf-optimize-btn"
                                            :style="{
                                                background: optimizeBusy
                                                    ? 'var(--cancel-optimizer-button)'
                                                    : hasRunOptimizer
                                                      ? 'linear-gradient(180deg, #60656f 0%, #4f545f 100%)'
                                                      : 'linear-gradient(180deg, #e6c86f 0%, #cfaf52 100%)',
                                                color: optimizeBusy ? 'var(--text-muted)' : hasRunOptimizer ? 'var(--hf-text-bright)' : '#1b1f25',
                                            }"
                                            @click="onOptimizeClick"
                                        >
                                            {{ optimizeBusy ? "Cancel Optimize" : hasRunOptimizer ? "Re-run Optimizer" : ">>> Optimize <<<" }}
                                        </button>

                                        <label class="hf-inline-check">
                                            <input v-model="autoStartOptimizer" type="checkbox" />
                                            <span>Auto start optimizer</span>
                                        </label>

                                        <div class="hf-metric-card">
                                            <div class="hf-metric-label">Avg eqv gold cost</div>
                                            <div class="hf-metric-value" :class="curIsBest ? 'best' : 'not-best'">{{ avgEqvGoldCost }}</div>
                                            <div class="hf-metric-status" :class="resultStatusClass">{{ resultStatus }}</div>
                                            <div v-if="allowManualState" class="hf-result-note">Cost so far + average future cost</div>
                                        </div>

                                        <div v-if="optimizeError" class="optimizer-error">Error: {{ optimizeError }}</div>

                                        <div v-if="optimizeBusy" class="optimizer-progress">
                                            <span>Optimizer progress: {{ Math.max(optimizerProgress, 0.001).toFixed(2) }}%</span>
                                            <div class="progress-bar">
                                                <div class="progress-fill" :style="{ width: `${optimizerProgress}%` }" />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>

                            <section class="hf-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Controls</div>
                                </div>
                                <div class="hf-card-body hf-options-body">
                                    <div class="hf-options-row">
                                        <button class="hf-header-link-btn" @click="resetAll">Reset All</button>
                                        <button class="hf-header-link-btn" @click="resetOptimizerState">Reset Optimizer</button>
                                    </div>
                                    <button class="hf-header-link-btn" @click="copyPayload">Copy Payload</button>

                                    <div class="hf-divider" />
                                    <label class="hf-inline-check">
                                        <input v-model="expressEvent" type="checkbox" />
                                        <span>Express event</span>
                                    </label>
                                    <label class="hf-inline-check">
                                        <input v-model="cumulativeGraph" type="checkbox" />
                                        <span>Cumulative graph</span>
                                    </label>
                                    <label class="hf-inline-check">
                                        <input v-model="allowManualState" type="checkbox" />
                                        <span>Enable progress updates for better optimization</span>
                                    </label>
                                </div>
                            </section>
                        </div>

                        <div class="hf-compact-row">
                            <section class="hf-card hf-tap-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Tap Instructions</div>
                                    <span class="hf-card-hint">Top to bottom execution</span>
                                </div>
                                <div class="hf-card-body">
                                    <p class="hf-copy">Empty state means no book and no breath on that tap.</p>
                                    <p class="hf-copy" v-if="allowManualState">
                                        Progress updates enabled: update progress and success after each relevant outcome.
                                    </p>
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

                            <section class="hf-card hf-freetap-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Free Tap Priority</div>
                                    <span class="hf-card-hint">Reorder if needed</span>
                                </div>
                                <div class="hf-card-body">
                                    <p class="hf-copy">
                                        Keep attempting free taps until you run out, then move on to the next. The instructions for tapping takes into account
                                        the normal taps you may need to do before or in-between free taps.
                                    </p>
                                    <div v-if="freeTapRows.length" class="hf-freetap-table">
                                        <div class="hf-freetap-head">
                                            <span>#</span>
                                            <span>Upgrade</span>
                                            <span>Chance</span>
                                            <span>Actions</span>
                                        </div>
                                        <div
                                            v-for="(row, index) in freeTapRows"
                                            :key="`freetap-${row.upgradeIndex}`"
                                            class="hf-freetap-row"
                                            :class="{
                                                draggable: true,
                                                dragging: draggingFreeTapIndex === index,
                                                'drop-target':
                                                    dragOverFreeTapIndex === index && draggingFreeTapIndex !== null && draggingFreeTapIndex !== index,
                                            }"
                                            draggable="true"
                                            @dragstart="onFreeTapDragStart(index, $event)"
                                            @dragover.prevent="onFreeTapDragOver(index)"
                                            @dragenter.prevent="onFreeTapDragOver(index)"
                                            @drop.prevent="onFreeTapDrop(index)"
                                            @dragend="onFreeTapDragEnd"
                                        >
                                            <span>{{ index + 1 }}</span>
                                            <span class="hf-freetap-upgrade">{{ pieceName(row.upgrade) }}</span>
                                            <span>{{ row.probability > 0 ? `${formatSig(row.probability * 100, 3)}%` : "-" }}</span>
                                            <div class="hf-freetap-actions">
                                                <span class="hf-drag-pill">Drag</span>
                                                <button class="hf-mini-btn" @click="toggleUpgradeSucceeded(row.upgradeIndex)">
                                                    {{ getUpgradeSucceeded(row.upgradeIndex) ? "Undo" : "Succeed" }}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                    <p v-else class="hf-copy">No available free taps in current state.</p>
                                </div>
                            </section>
                        </div>
                    </section>

                    <aside class="hf-side-lane">
                        <section class="hf-card">
                            <div class="hf-card-header">
                                <div class="hf-card-title"><span class="hf-card-title-dot" />Materials and Prices</div>
                                <span class="hf-card-hint">Owned + market</span>
                            </div>
                            <div class="hf-card-body">
                                <p class="hf-copy">Optimizer minimizes equivalent gold across spent and consumed tradable value.</p>

                                <div class="hf-material-stack">
                                    <div class="hf-table-wrap">
                                        <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                                            <span />
                                            <span>Owned</span>
                                            <span>Price</span>
                                            <span v-if="customLeftovers">Left</span>
                                        </div>
                                        <div v-for="label in MATS_LABELS" :key="`mats-${label}`" class="hf-table-row" :class="{ leftovers: customLeftovers }">
                                            <label class="hf-row-label">
                                                <span>{{ label }}</span>
                                                <img :src="iconPath(label)" :alt="label" />
                                            </label>
                                            <input type="text" :value="matsOwned[label]" @input="setRecordValue(matsOwned, label, $event)" />
                                            <input
                                                v-if="label !== 'Special Leap'"
                                                type="text"
                                                :value="matsPrices[label] ?? ''"
                                                @input="setRecordValue(matsPrices, label, $event)"
                                            />
                                            <div v-else class="hf-input-placeholder" />
                                            <input
                                                v-if="customLeftovers"
                                                type="text"
                                                :value="matsLeftover[label]"
                                                @input="setRecordValue(matsLeftover, label, $event)"
                                            />
                                        </div>
                                    </div>

                                    <div class="hf-table-wrap">
                                        <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                                            <span />
                                            <span>Owned</span>
                                            <span>Price</span>
                                            <span v-if="customLeftovers">Left</span>
                                        </div>
                                        <div
                                            v-for="labels in JUICE_LABELS"
                                            :key="`weapon-${labels[0]}`"
                                            class="hf-table-row"
                                            :class="{ leftovers: customLeftovers }"
                                        >
                                            <label class="hf-row-label hf-row-label-books">
                                                <span>{{ labels[0] }}</span>
                                                <img :src="iconPath(labels[0])" :alt="labels[0]" />
                                            </label>
                                            <input type="text" :value="weaponOwned[labels[0]]" @input="setRecordValue(weaponOwned, labels[0], $event)" />
                                            <input type="text" :value="weaponPrices[labels[0]]" @input="setRecordValue(weaponPrices, labels[0], $event)" />
                                            <input
                                                v-if="customLeftovers"
                                                type="text"
                                                :value="weaponLeftover[labels[0]]"
                                                @input="setRecordValue(weaponLeftover, labels[0], $event)"
                                            />
                                        </div>
                                    </div>

                                    <div class="hf-table-wrap">
                                        <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                                            <span />
                                            <span>Owned</span>
                                            <span>Price</span>
                                            <span v-if="customLeftovers">Left</span>
                                        </div>
                                        <div
                                            v-for="labels in JUICE_LABELS"
                                            :key="`armor-${labels[1]}`"
                                            class="hf-table-row"
                                            :class="{ leftovers: customLeftovers }"
                                        >
                                            <label class="hf-row-label hf-row-label-books">
                                                <span>{{ labels[1] }}</span>
                                                <img :src="iconPath(labels[1])" :alt="labels[1]" />
                                            </label>
                                            <input type="text" :value="armorOwned[labels[1]]" @input="setRecordValue(armorOwned, labels[1], $event)" />
                                            <input type="text" :value="armorPrices[labels[1]]" @input="setRecordValue(armorPrices, labels[1], $event)" />
                                            <input
                                                v-if="customLeftovers"
                                                type="text"
                                                :value="armorLeftover[labels[1]]"
                                                @input="setRecordValue(armorLeftover, labels[1], $event)"
                                            />
                                        </div>
                                    </div>
                                </div>

                                <label class="hf-inline-check" style="margin-top: 10px">
                                    <input v-model="customLeftovers" type="checkbox" />
                                    <span>Custom leftover values</span>
                                </label>
                            </div>
                        </section>
                    </aside>
                </div>

                <section class="hf-card hf-analysis-pane">
                    <div class="hf-card-header">
                        <div class="hf-card-title"><span class="hf-card-title-dot" />Analysis</div>
                        <div class="hf-analysis-tabs">
                            <button :class="['hf-analysis-tab', { active: analysisTab === 'distribution' }]" @click="analysisTab = 'distribution'">
                                Distribution
                            </button>
                            <button :class="['hf-analysis-tab', { active: analysisTab === 'breakdown' }]" @click="analysisTab = 'breakdown'">
                                Gold Breakdown
                            </button>
                        </div>
                    </div>
                    <div class="hf-card-body">
                        <section v-if="analysisTab === 'distribution'">
                            <div class="hf-dist-desc">Distribution reflects free-tap and juice usage from your current optimizer output.</div>
                            <div class="hf-dist-graphs">
                                <div v-for="(label, index) in distributionLabels" :key="`graph-${label}`" class="hf-graph-row">
                                    <div class="hf-graph-icon">
                                        <img :src="iconPath(label)" :alt="label" />
                                    </div>
                                    <MaterialGraph
                                        :data="histogramResult?.cum_percentiles?.[index] ?? null"
                                        :average="histogramResult?.average?.[index] ?? null"
                                        :secondary-annotation="histogramResult?.budgets?.[index] ?? null"
                                        :color-var="GRAPH_COLORS[index]"
                                        :cumulative="cumulativeGraph"
                                        :height="120"
                                    />
                                </div>
                            </div>
                        </section>

                        <section v-else>
                            <div class="hf-breakdown-grid">
                                <div class="hf-breakdown-table">
                                    <div v-for="(label, index) in MATS_LABELS.slice(0, 7)" :key="`mats-breakdown-${label}`" class="hf-breakdown-row">
                                        <span class="hf-breakdown-label">{{ label }}</span>
                                        <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(index))]">
                                            {{ breakdownText(goldBreakdownValue(index)) }}
                                        </span>
                                    </div>
                                </div>

                                <div class="hf-breakdown-table">
                                    <div
                                        v-for="(label, index) in JUICE_LABELS.map((pair) => pair[0])"
                                        :key="`weapon-breakdown-${label}`"
                                        class="hf-breakdown-row"
                                    >
                                        <span class="hf-breakdown-label">{{ label }}</span>
                                        <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(7 + index))]">
                                            {{ breakdownText(goldBreakdownValue(7 + index)) }}
                                        </span>
                                    </div>
                                </div>

                                <div class="hf-breakdown-table">
                                    <div
                                        v-for="(label, index) in JUICE_LABELS.map((pair) => pair[1])"
                                        :key="`armor-breakdown-${label}`"
                                        class="hf-breakdown-row"
                                    >
                                        <span class="hf-breakdown-label">{{ label }}</span>
                                        <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(7 + juiceAvail + index))]">
                                            {{ breakdownText(goldBreakdownValue(7 + juiceAvail + index)) }}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            <div class="hf-combined-cost">Combined: {{ metricToText(currentMetric) }}</div>
                        </section>
                    </div>
                </section>
            </div>
        </div>
    </div>
</template>
