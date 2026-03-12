<script setup lang="ts">
import GoldBreakdown from "./GoldBreakdown.vue"
</script>

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Analysis</div>
            <div class="hf-analysis-tabs">
                <button :class="['hf-analysis-tab', { active: analysisTab === 'distribution' }]" @click="analysisTab = 'distribution'">Distribution</button>
                <button :class="['hf-analysis-tab', { active: analysisTab === 'breakdown' }]" @click="analysisTab = 'breakdown'">Gold Breakdown</button>
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
        </div>
    </section>
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
                        <input v-if="customLeftovers" type="text" :value="matsLeftover[label]" @input="setRecordValue(matsLeftover, label, $event)" />
                    </div>
                </div>

                <div class="hf-table-wrap">
                    <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                        <span />
                        <span>Owned</span>
                        <span>Price</span>
                        <span v-if="customLeftovers">Left</span>
                    </div>
                    <div v-for="labels in JUICE_LABELS" :key="`weapon-${labels[0]}`" class="hf-table-row" :class="{ leftovers: customLeftovers }">
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
                    <div v-for="labels in JUICE_LABELS" :key="`armor-${labels[1]}`" class="hf-table-row" :class="{ leftovers: customLeftovers }">
                        <label class="hf-row-label hf-row-label-books">
                            <span>{{ labels[1] }}</span>
                            <img :src="iconPath(labels[1])" :alt="labels[1]" />
                        </label>
                        <input type="text" :value="armorOwned[labels[1]]" @input="setRecordValue(armorOwned, labels[1], $event)" />
                        <input type="text" :value="armorPrices[labels[1]]" @input="setRecordValue(armorPrices, labels[1], $event)" />
                        <input v-if="customLeftovers" type="text" :value="armorLeftover[labels[1]]" @input="setRecordValue(armorLeftover, labels[1], $event)" />
                    </div>
                </div>
            </div>

            <label class="hf-inline-check" style="margin-top: 10px">
                <input v-model="customLeftovers" type="checkbox" />
                <span>Custom leftover values</span>
            </label>
        </div>
    </section>
    <GoldBreakdown />
</template>
