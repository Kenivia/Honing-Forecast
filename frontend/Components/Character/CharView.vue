<script setup lang="ts">
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
import { useProfilesStore } from "@/Stores/CharacterProfile"
import { useRosterStore } from "@/Stores/RosterConfig"
import { grids_to_keyed, StateBundle } from "@/Utils/Interfaces"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { build_material_info, build_payload } from "@/WasmInterface/payload"
import { storeToRefs } from "pinia"

import { nextTick, onWatcherCleanup, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"

const route = useRoute()
const router = useRouter()

const profile_store = useProfilesStore()
const { active_profile } = storeToRefs(profile_store)
const { roster_config } = storeToRefs(useRosterStore())
// Route param → active character

const re_render_char_view_trigger = ref(true)
watch(
    () => profile_store.active_profile_index,
    async () => {
        // console.log("charview rerendered")
        re_render_char_view_trigger.value = false
        await nextTick()
        re_render_char_view_trigger.value = true
    },
)
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
    <div v-if="re_render_char_view_trigger" class="hf-main-stage">
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
