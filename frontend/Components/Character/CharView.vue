<script setup lang="ts">
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
import { useProfilesStore } from "@/Stores/CharacterProfile"
import { storeToRefs } from "pinia"
import { nextTick, onUnmounted, watch } from "vue"
import { useRoute, useRouter } from "vue-router"

const route = useRoute()
const router = useRouter()

const profile_store = useProfilesStore()
const { active_profile } = storeToRefs(profile_store)

const match = profile_store.profiles.findIndex((c) => c.char_name === (route.params.characterName as string))
if (match >= 0) {
    profile_store.switchProfile(match)
} else {
    router.replace({ name: "char", params: { characterName: profile_store.profiles[0].char_name } })
    profile_store.switchProfile(0)
}
watch(
    () => route.params.characterName as string,
    (name) => {
        const match = profile_store.profiles.findIndex((c) => c.char_name === name)
        if (match >= 0) {
            if (profile_store.active_profile_index !== match) {
                // this happens one invalid names (routre param written to by the one-off code, triggering the watcher) i believe, idk how to prevent that but this works
                active_profile.value.optimizer_worker_bundle.cancel()
                active_profile.value.histogram_worker_bundle.cancel()
                active_profile.value.evaluation_worker_bundle.cancel()

                profile_store.switchProfile(match)
            }
        } else {
            router.replace({ name: "char", params: { characterName: profile_store.profiles[0].char_name } })
            profile_store.switchProfile(0)
        }
    },
)

onUnmounted(() => {
    // kill workers when going to market / roster view
    active_profile.value.optimizer_worker_bundle.cancel()
    active_profile.value.histogram_worker_bundle.cancel()
    active_profile.value.evaluation_worker_bundle.cancel()
})
</script>

<template>
    <div class="hf-main-stage" :key="active_profile.char_name">
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
