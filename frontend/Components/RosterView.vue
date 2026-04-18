<script setup lang="ts">
import { CharProfile, create_default_char_profile, recreate_char_profile } from "@/Stores/CharacterProfile"
import { create_default_owned_input_column, useRosterStore } from "@/Stores/RosterConfig"
import { achieved_ilevel, format_char_name, pending_ilevel } from "@/Utils/Helpers"
import { storeToRefs } from "pinia"
import { ref } from "vue"
import { RouterLink } from "vue-router"

const roster_store = useRosterStore()
const { roster_config, roster_ids } = storeToRefs(roster_store)

const names = ref(roster_config.value.profiles.map((y) => y.char_name))

function add_new_char(roster_id: number) {
    let new_char = create_default_char_profile()
    new_char.char_name = format_char_name("Newchar", roster_config.value.profiles.length)
    new_char.roster_id = roster_id
    names.value.push(new_char.char_name)
    roster_store.addProfile(new_char)
}
function add_new_roster(roster_id: number) {
    let new_char = create_default_char_profile()
    new_char.char_name = format_char_name("Newchar", roster_config.value.profiles.length)
    new_char.roster_id = roster_id
    names.value.push(new_char.char_name)
    roster_config.value.profiles.push(new_char)

    roster_config.value.roster_mats_owned[roster_id] = create_default_owned_input_column()
    roster_config.value.tradable_mats_owned[roster_id] = create_default_owned_input_column()
}

function duplicate(index, roster_id: number) {
    let this_parsed = { ...create_default_char_profile(), ...roster_config.value.profiles[index] }

    let new_char = recreate_char_profile(JSON.parse(JSON.stringify(this_parsed)))
    new_char.char_name = format_char_name("Newchar", roster_config.value.profiles.length)
    names.value.push(new_char.char_name)
    roster_store.addProfile(new_char)
}

function delete_profile(index, roster_id) {
    // console.log(this_roster_profiles.length)
    if (roster_config.value.active_profile_index >= index) {
        roster_store.switchProfile(Math.max(index - 1, 0))
    }
    roster_config.value.profiles.splice(index, 1)
    names.value.splice(index, 1)

    if (roster_config.value.profiles.filter((x) => x.roster_id == roster_id).length === 0) {
        delete roster_config.value.roster_mats_owned[roster_id]
        delete roster_config.value.tradable_mats_owned[roster_id]
    }

    // console.log(this_roster_profiles.length)
}
</script>

<template>
    <div class="hf-main-stage">
        <div :class="roster_ids.length > 1 ? 'hf-outer-budget-grid' : 'hf-only-one-roster'">
            <section v-for="(roster_id, roster_index) in roster_ids" class="hf-card" :key="'roster-${roster_id}'">
                <div v-if="roster_config.profiles.length > 1" class="hf-card-header">
                    <div class="hf-card-title">
                        <span class="hf-card-title-dot" />
                        <span class="hf-card-title" /> Roster {{ roster_index + 1 }}
                    </div>
                </div>

                <div
                    v-for="[profile, profile_index] in roster_config.profiles
                        .map((x, index): [CharProfile, number] => [x, index])
                        .filter((y) => y[0].roster_id === roster_id)"
                    class="hf-char-row"
                    :key="'profile-${profile_index}'"
                >
                    <input
                        v-model="names[profile_index]"
                        @change="((names[profile_index] = format_char_name(names[profile_index], profile_index)), (profile.char_name = names[profile_index]))"
                    />
                    <div class="hf-char-meta">
                        <label class="hf-achieved-ilevel">Achieved ilevel: {{ achieved_ilevel(profile) }}</label>
                        <label class="hf-pending-ilevel">Pending ilevel: {{ pending_ilevel(profile) }}</label>
                    </div>
                    <RouterLink :to="{ name: 'char', params: { characterName: profile.char_name } }" class="hf-header-button"> Go to character </RouterLink>

                    <button class="hf-header-button" @click="() => duplicate(profile_index, roster_id)">Make a copy</button>
                    <button v-if="roster_config.profiles.length > 1" class="btn-cancel" @click="() => delete_profile(profile_index, roster_id)">Delete</button>
                </div>
                <button class="hf-new-char" @click="() => add_new_char(roster_id)">Add new character</button>
            </section>
            <button
                class="hf-card,hf-new-char"
                style="align-self: center"
                @click="() => add_new_roster(Math.max(...roster_config.profiles.map((x) => x.roster_id)) + 1)"
            >
                Add new roster
            </button>
        </div>
    </div>
</template>
<style scoped>
/* Base Variables & Structural Setup */
.hf-outer-budget-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    align-items: start;
}
.hf-only-one-roster {
    display: contents;
}
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
