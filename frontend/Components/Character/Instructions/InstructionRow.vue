<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES } from "@/Utils/Constants"
import { formatSig, get_piece_name, iconPath } from "@/Utils/Helpers"
import { State, Upgrade, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed } from "vue"
import InstructionRow from "./InstructionRow.vue"

const props = defineProps<{
    upgrade: Upgrade
    perform_order: number
    special_invalid_index: number
}>()

const free_tap_this_upgrade = computed(() => {
    return props.perform_order < props.special_invalid_index
})

interface Streak {
    juice: boolean
    book: boolean
    count: number
}

function aggregateStreaks(): Streak[] | null {
    if (props.upgrade.state.length === 0) return []
    const streaks: Streak[] = []
    let current: Streak | null = null
    if (props.upgrade.is_normal_honing) {
        for (const [juice, book] of props.upgrade.state.slice(props.upgrade.alr_failed, props.upgrade.normal_dist.length)) {
            const hasBook = book > 0
            if (current && current.juice === juice && current.book === hasBook) {
                current.count++
            } else {
                current = { juice, book: hasBook, count: 1 }
                streaks.push(current)
            }
        }
    } else {
        for (let index = 0; index < Math.max(...props.upgrade.state.map((x) => x[1])); index++) {
            let juice = index < props.upgrade.state[0][1]
            let hasBook = index < props.upgrade.state[1][1]
            if (current && current.juice === juice && current.book === hasBook) {
                current.count++
            } else {
                current = { juice, book: hasBook, count: 1 }
                streaks.push(current)
            }
        }
    }
    return streaks
}
let streaks = computed(aggregateStreaks)

function streakHtml(streak: Streak): string {
    const count = ` for ${streak.count}`
    if (!streak.juice && !streak.book) return `<span style="color: white;">Nothing</span>${count}`
    if (streak.juice && !streak.book) return `<span style="color: var(${props.upgrade.is_weapon ? "--series-red" : "--series-blue"});">Juice</span>${count}`
    if (!streak.juice && streak.book) return `<span style="color: var(--series-books);">Book</span>${count}`
    return `<span style="color: var(${props.upgrade.is_weapon ? "--series-red" : "--series-blue"});">Juice</span>+<span style="color: var(--series-books);">Book</span>${count}`
}
const final_html_arr = computed(() => streaks.value.map((x) => streakHtml(x)))
</script>
<template>
    <div class="hf-upgrade-row">
        <div class="hf-upgrade-order">{{ props.perform_order + 1 }}</div>
        <div class="hf-upgrade-meta">
            <div class="hf-upgrade-topline">
                <span class="hf-upgrade-name">{{ get_piece_name(upgrade) + " +" + String(upgrade.upgrade_index + 1) }}</span>
                <span v-if="free_tap_this_upgrade" class="hf-upgrade-free"> Free tap {{ formatSig(upgrade.this_special_chance * 100, 3) }}% </span>
            </div>
            <div class="streak-display">
                <span v-for="(_, i) in streaks" :key="i" class="streak-label" v-html="final_html_arr[i]" />
            </div>
        </div>
    </div>
</template>
<style>
.streak-display {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 0.5rem;
}

.streak-label {
    white-space: nowrap;
}
</style>
