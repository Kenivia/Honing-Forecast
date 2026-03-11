<script setup lang="ts">
import ControlPanel from "@/Character/ControlPanel.vue"
import Instructions from "@/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Character/StatusInput/StatusInput.vue"
</script>

<template>
    <div class="hf-main-stage">
        <StatusInput />
        <ControlPanel />
        <MaterialDist />
        <Instructions />
    </div>
</template>
