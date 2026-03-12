<script setup lang="ts">
import ControlPanel from "@/Components/Character/ControlPanel.vue"
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
</script>

<template>
    <div class="hf-main-stage">
        <StatusInput />
        <ControlPanel />
        <MaterialDist />
        <Instructions />
    </div>
</template>
