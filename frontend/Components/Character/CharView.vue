<script setup lang="ts">
import ControlPanel from "@/Components/Character/ControlPanel.vue"
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
</script>

<template>
    <div class="hf-main-stage">
        <StatusInput />

        <MaterialDist /> <ControlPanel />
        <!-- <Instructions /> -->
    </div>
</template>
<style>
.hf-main-stage {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px;
    min-width: 0;
    width: 100%;
    justify-content: center;
    align-items: center;
}
</style>
