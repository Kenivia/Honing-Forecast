<script setup lang="ts">
import TickboxGrid from "./TickboxGrid.vue"
</script>

<template>
    <div class="hf-honing-row">
        <section class="hf-card hf-normal-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Normal Honing</div>
                <span class="hf-card-hint">Build target grid</span>
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="normal" />
            </div>
        </section>

        <section class="hf-card hf-advanced-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Advanced Honing</div>
                <span class="hf-card-hint">Juice on Grace assumed</span>
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="adv" />
            </div>
        </section>
    </div>
</template>
