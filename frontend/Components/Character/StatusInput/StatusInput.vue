<script setup lang="ts">
import TickboxGrid from "./TickboxGrid.vue"
</script>

<template>
    <div class="hf-honing-row">
        <section class="hf-normal-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Normal Honing</div>
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="normal" />
            </div>
        </section>

        <section class="hf-advanced-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Advanced Honing</div>
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="adv" />
            </div>
        </section>
    </div>
</template>
<style>
.hf-normal-card {
    min-width: 0;
    width: min(100%, 800px);
}
.hf-advanced-card {
    width: min(100%, 262px);
    min-width: 0;
}
.hf-honing-row {
    gap: 12px;
    align-items: center;
    min-width: 0;
}
</style>
