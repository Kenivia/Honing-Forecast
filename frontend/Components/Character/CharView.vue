<script setup lang="ts">
import ControlPanel from "@/Components/Character/ControlPanel.vue"
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
import { useProfilesStore } from "@/stores/CharacterProfile"

import { watch } from "vue"
import { useRoute, useRouter } from "vue-router"

const route = useRoute()
const router = useRouter()
const profile_store = useProfilesStore()

// Route param → active character
watch(
    () => route.params.characterName as string,
    (name) => {
        const match = profile_store.profiles.findIndex((c) => c.char_name === name)
        if (match >= 0) {
            profile_store.active_profile_index = match
        } else {
            router.replace({ name: "char", params: { characterName: profile_store.profiles[0].char_name } })
        }
    },
    { immediate: true },
)
</script>

<template>
    <div class="hf-main-stage">
        <StatusInput />
        <MaterialDist />
        <Instructions />
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
