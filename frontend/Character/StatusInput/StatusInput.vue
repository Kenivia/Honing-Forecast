<template>
    <div class="hf-honing-row">
        <section class="hf-card hf-normal-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Normal Honing</div>
                <span class="hf-card-hint">Build target grid</span>
            </div>
            <div class="hf-card-body"></div>
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
</template>
