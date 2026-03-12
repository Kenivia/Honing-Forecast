<script setup>
import { RouterLink, RouterView } from "vue-router"
import router from "./routers"
import { storeToRefs } from "pinia"
import { uesRosterStore } from "./stores/RosterConfig"

const rosterStore = uesRosterStore()

const profileStore = uesProfileStore()
</script>

<template>
    <header>
        <div class="wrapper">
            <nav class="flex justify-between mb-10 pt-10">
                <div class="hf-brand">
                    <div class="hf-brand-icon">
                        <img :src="iconPath('Forecast Icon')" alt="Forecast icon" style="width: 34px; height: 34px" />
                    </div>
                    <div>
                        <h1 class="hf-title">Honing Forecast</h1>
                        <div class="hf-subtitle">Lost Ark Upgrade Planner</div>
                    </div>
                </div>
                <div class="hf-header-links">
                    <a href="https://github.com/Kenivia/Honing-Forecast">GitHub</a>
                </div>

                <div>
                    <RouterLink to="/">Home</RouterLink>
                    <RouterLink to="/about">About</RouterLink>
                </div>
            </nav>
        </div>
    </header>

    <main class="bg-white">
        <RouterView />
    </main>
</template>
