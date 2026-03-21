<script setup lang="ts">
import { computed, ref } from "vue"

const props = defineProps<{
    labelText: string
    tooltipText?: string | null
    checkEligibility?: () => boolean | null
    showTooltipOnlyOnDisabled?: boolean | null
    warning?: boolean
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
            :class="{ disabled: !eligible, warning }"
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
    background: var(--serca-blue);
    color: white;
    transition: all 0.2s ease;
}
.tier-button.warning {
    color: rgb(255, 114, 114);
}

.tier-button:hover {
    background: var(--serca-blue-hover);
}

/* DISABLED STYLE */
.tier-button.disabled {
    background: var(--hf-bg-hover);
    color: var(--text-muted);
    cursor: not-allowed;
}

/* TOOLTIP STYLE */
.tooltip {
    position: absolute;
    bottom: 120%;
    left: 50%;
    transform: translateX(-50%);
    background: var(--tooltip-bg);
    color: var(--text-bright);
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 12px;
    white-space: nowrap;
    pointer-events: none;
}
</style>
