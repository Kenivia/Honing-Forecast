<script setup lang="ts">
import { computed, ref } from "vue"

const props = defineProps<{
    labelText: string
    tooltipText?: string | null
    checkEligibility?: () => boolean | null
    showTooltipOnlyOnDisabled?: boolean | null
}>()

const emit = defineEmits(["change-tier"])

const showTooltip = ref(false)

const eligible = computed(() => {
    return props.checkEligibility ? props.checkEligibility() : true
})

function handleClick() {
    if (eligible.value) {
        emit("change-tier")
    }
}
</script>
<template>
    <div class="tier-button-wrapper">
        <button
            class="tier-button"
            :class="{ disabled: !eligible }"
            :disabled="!eligible"
            @click="handleClick"
            @mouseenter="showTooltip = true"
            @mouseleave="showTooltip = false"
        >
            {{ labelText }}
        </button>

        <div v-if="showTooltip && (!showTooltipOnlyOnDisabled || !eligible)" class="tooltip">
            {{ tooltipText }}
        </div>
    </div>
</template>
<style scoped>
/* wrapper needed for tooltip positioning */
.tier-button-wrapper {
    position: relative;
    display: inline-block;
}

/* ENABLED STYLE */
.tier-button {
    padding: 10px 18px;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    background: #2806b1;
    color: white;
    transition: all 0.2s ease;
}

.tier-button:hover {
    background: #4520d8;
}

/* DISABLED STYLE */
.tier-button.disabled {
    background: #cfcfcf;
    color: #777;
    cursor: not-allowed;
}

/* TOOLTIP STYLE */
.tooltip {
    position: absolute;
    bottom: 120%;
    left: 50%;
    transform: translateX(-50%);
    background: #222;
    color: white;
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 12px;
    white-space: nowrap;
    opacity: 0.95;
    pointer-events: none;
}
</style>
