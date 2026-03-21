<script setup lang="ts">
import { create_default_char_profile, recreate_char_profile, useProfilesStore } from "@/stores/CharacterProfile"
import { useRosterStore } from "@/stores/RosterConfig"
import { achieved_ilevel, pending_ilevel } from "@/Utils/Helpers"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { build_payload } from "@/WasmInterface/payload"
import { storeToRefs } from "pinia"
import { ref, toRaw } from "vue"
import { RouterLink } from "vue-router"

const profile_store = useProfilesStore()
const { roster_config } = storeToRefs(useRosterStore())

const names = ref(profile_store.profiles.map((x) => x.char_name))

function formatCharName(raw: string, index: number): string {
    let result = raw.replace(/ /g, "") //
    // 2. Remove non-alphanumeric (keep underscores)
    result = result.replace(/[^a-zA-Z0-9_]/g, "")
    // 3. Lowercase every letter after the first
    result = result.replace(/(?<=.)[A-Z]/g, (c) => c.toLowerCase()).slice(0, 16)
    // 4. If empty, or already taken by another profile, append index
    const otherNames = profile_store.profiles.filter((_, i) => i !== index).map((x) => x.char_name)
    if (!result || otherNames.includes(result)) {
        result += String(index)
    }
    return result
}

function add_new_char() {
    let new_char = create_default_char_profile()
    new_char.char_name = "NewChar" + String(profile_store.profiles.length + 1)
    names.value.push(new_char.char_name)
    profile_store.profiles.push(new_char)
}

function duplicate(index) {
    let this_parsed = { ...create_default_char_profile(), ...profile_store.profiles[index] }

    let new_char = recreate_char_profile(this_parsed)
    new_char.char_name = "NewChar" + String(profile_store.profiles.length + 1)
    names.value.push(new_char.char_name)
    profile_store.profiles.push(new_char)
    new_char.optimizer_worker_bundle.start(WasmOp.Parser, build_payload(WasmOp.Parser, new_char, roster_config.value))
}

function delete_profile(index) {
    console.log(profile_store.profiles.length)
    profile_store.profiles.splice(index, 1)
    names.value.splice(index, 1)
    console.log(profile_store.profiles.length)
}
</script>

<template>
    <div class="hf-main-stage">
        <section class="hf-card">
            <div v-for="(profile, index) in profile_store.profiles" class="hf-char-row" :key="index">
                <input v-model="names[index]" @change="((names[index] = formatCharName(names[index], index)), (profile.char_name = names[index]))" />
                <div class="hf-char-meta">
                    <label class="hf-achieved-ilevel">Achieved ilevel: {{ achieved_ilevel(profile) }}</label>
                    <label class="hf-pending-ilevel">Pending ilevel: {{ pending_ilevel(profile) }}</label>
                </div>
                <RouterLink :to="{ name: 'char', params: { characterName: profile.char_name } }" class="hf-header-button"> Go to character </RouterLink>

                <button class="hf-header-button" @click="() => duplicate(index)">Make a copy</button>
                <button v-if="profile_store.profiles.length > 1" class="btn-cancel" @click="() => delete_profile(index)">Delete</button>
            </div>
            <button class="hf-new-char" @click="add_new_char">Add new character</button>
        </section>
    </div>
</template>
<style scoped>
/* Base Variables & Structural Setup */

.btn-cancel {
    background: var(--hf-cancel, #d32f2f);
    color: var(--hf-text-bright,);
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
}

.hf-new-char {
    width: 100%;
}

.hf-char-meta {
    display: flex;
    flex-direction: column;
    width: 200px;
    min-width: 180px;
}

.hf-char-row {
    --icon-size: 36px;
    --font-primary: 1rem;
    --font-small: 0.8rem;

    display: flex;
    flex-direction: row;
    margin-bottom: 1rem;
    background: var(--hf-bg-surface, #1e1e1e);
    border-radius: 8px;
    padding: 1rem;
    align-items: center;
    gap: 4px;
}

.hf-upgrade-topline {
    border-bottom: 1px solid var(--separator-color, #333);
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
    font-weight: bold;
    font-size: 1.1rem;
}

.hf-upgrade-content {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
    align-items: stretch;
}

/* --- Left Controls (Order & Taps) --- */
.hf-left-controls {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
}

.order-block,
.free-tap-actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
}

.order-circle {
    width: calc(var(--icon-size) * 2 + 0.25rem); /* Align with the two icon rows visually */
    height: calc(var(--icon-size) * 2 + 0.25rem);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: bold;
    background-color: var(--hf-text-muted, #555);
    color: var(--hf-bg-deep, #000);
}

.order-circle.is-free-tap {
    background-color: var(--hf-free-tap, #4caf50);
}

.order-text,
.action-desc {
    font-size: var(--font-small);
    color: var(--hf-text-muted, #aaa);
    text-align: center;
    text-wrap-mode: wrap;
}

.btn-all-failed {
    height: calc(var(--icon-size) * 3 + 0.25rem);
    background-color: var(--hf-free-tap);
    color: var(--hf-bg-deep, #000);
    border: none;
    border-radius: 8px;
    padding: 0 1rem;
    font-weight: bold;
    cursor: pointer;
    transition: filter 0.2s;
    text-wrap-mode: wrap;
    max-width: 100px;
}
.btn-all-failed:hover {
    filter: brightness(1.2);
}

.btn-expand {
    height: calc(var(--icon-size) * 1 + 0.25rem);
    background-color: var(--hf-text-muted);
    color: var(--hf-bg-deep, #000);
    border: none;
    border-radius: 8px;
    padding: 0 1rem;
    font-weight: bold;
    cursor: pointer;
    transition: filter 0.2s;
    justify-self: center;
}
.btn-expand:hover {
    filter: brightness(1.2);
}
/* --- Scrollable Instructions --- */
.hf-scrollable-instructions {
    display: flex;
    gap: 0.5rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
    flex: 1;
    min-width: 200px;
    max-width: 400px;
    transition: opacity 0.3s;
}

.hf-scrollable-instructions.is-dimmed {
    opacity: 0.4;
}

.instruction-stack {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    min-width: 80px;
}

.icon-slot {
    width: var(--icon-size);
    height: var(--icon-size);
    border: 1px solid rgba(255, 255, 255, 0.1);
    background-color: var(--hf-bg-raised);
}
.icon-slot.should-not-use {
    width: var(--icon-size);
    height: var(--icon-size);
    border: none;
}

.icon-slot img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}

.text-slot {
    text-align: center;
    margin-top: 0.5rem;
}

.line-primary {
    color: white;
    font-size: var(--font-primary);
}

.line-muted {
    color: var(--hf-text-muted, #aaa);
    font-size: var(--font-small);
}

@media (max-width: 900px) {
    .hf-char-row {
        flex-wrap: wrap;
        gap: 8px;
        align-items: flex-start;
    }

    .hf-char-row > input {
        width: 100%;
    }

    .hf-char-meta {
        width: 100%;
        min-width: 0;
    }

    .hf-char-row .hf-header-button,
    .hf-char-row .btn-cancel {
        flex: 1 1 auto;
        text-align: center;
    }
}
</style>
