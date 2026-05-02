<script>
import { RouterLink } from "vue-router"
</script>

<template>
    <footer class="hf-footer-bar">
        <a href="https://discord.gg/KWDpQyvgzc" class="hf-footer-links">
            <img src="/Icons/Discord.png" alt="Discord" />
            <span>Discord</span>
        </a>
        <a href="https://github.com/Kenivia/Honing-Forecast" class="hf-footer-links">
            <img src="/Icons/GitHub.png" alt="GitHub" />
            <span>GitHub</span>
        </a>
        <a href="https://ko-fi.com/kenivia" class="hf-footer-links">
            <img src="/Icons/kofi.png" alt="Ko-fi" />
            <span>Donate</span>
        </a>
        <RouterLink to="/change-log" class="hf-footer-links">
            <img src="/Icons/branch.png" alt="Ko-fi" />
            <span>v{{ 123 }}</span>
        </RouterLink>
    </footer>
</template>
<style>
.hf-footer-bar {
    display: flex;

    flex-direction: row;
    align-items: center;
    flex-wrap: wrap;
    margin-top: auto;
}

.hf-footer-note {
    color: var(--text-very-muted);
    font-size: 12px;
    text-align: center;
}
.hf-footer-links {
    display: flex;
    gap: 12px;
    align-items: center;
    color: var(--hf-text-muted);

    font: 500 11px/1 var(--hf-font-body);
    padding: 8px 12px;
    text-decoration: underline;

    height: fit-content;
}
.hf-footer-links:hover {
    color: var(--hf-text-bright);
}

.hf-footer-links img {
    height: 10px;
    filter: brightness(0.8);
}
</style>
