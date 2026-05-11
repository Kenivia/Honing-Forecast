<!-- <script setup lang="ts">
import {
    ALL_LABELS,
    GRAPH_COLORS,
    T4_MATS_LABELS,
    ANNOTATION_COLORS,
    ANNOTATION_POSITIONS,
    ANNOTATION_LABELS,
    PIECE_NAMES,
} from "@/Utils/Constants";
import { TreatmentPlan } from "@/Stores/CharacterProfile";

import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { AdvProgress, Upgrade, UpgradeStatus, WasmOp } from "@/Utils/Interfaces";
import MaterialGraph from "./MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed, ref, watchEffect } from "vue";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { input_column_to_num } from "@/Utils/InputColumn";
import { start_all_workers } from "../CharWorkerUtils";
import { RouterLink } from "vue-router";
import { to_upgrade_key } from "@/Utils/KeyedUpgrades";
import DetailedInputRow from "./DetailedInputRow.vue";

const props = defineProps<{
    upgrade: Upgrade;
}>();

const { active_profile } = storeToRefs(useRosterStore());

const artisan = ref(props.upgrade.starting_artisan * 100.0);
const current_adv_upgrade = ref(
    props.upgrade.adv_config
        ? Math.floor(props.upgrade.adv_config.start_xp / 10) + props.upgrade.upgrade_index * 10
        : 0,
);
const current_adv_xp = ref(
    props.upgrade.adv_config
        ? (props.upgrade.adv_config.start_xp - Math.floor(props.upgrade.adv_config.start_xp / 10) * 10) * 10
        : 0,
);
const current_grace_progress = ref(props.upgrade.adv_config.start_balls);
const next_free = ref(props.upgrade.adv_config?.next_free ?? false);
const next_big = ref(props.upgrade.adv_config?.next_big ?? false);

function write_progress() {
    let relevant_keyed =
        active_profile.value.keyed_upgrades[
            to_upgrade_key(
                props.upgrade.piece_type,
                props.upgrade.upgrade_index,
                props.upgrade.is_normal_honing,
                active_profile.value.tier,
            )
        ];
    if (props.upgrade.is_normal_honing) {
        artisan.value = Math.max(0, Math.min(artisan.value, 99.99));
        relevant_keyed.starting_artisan = artisan.value / 100.0;
    } else {
        current_adv_upgrade.value = Math.max(
            props.upgrade.upgrade_index * 10,
            Math.min((props.upgrade.upgrade_index + 1) * 10 - 1, current_adv_upgrade.value),
        );
        current_adv_xp.value = Math.floor(Math.max(0, Math.min(90, current_adv_xp.value)) / 10) * 10;
        // console.log(current_grace_progress.value)
        current_grace_progress.value = Math.min(6, Math.max(0, current_grace_progress.value));
        relevant_keyed.adv_progress = [
            (current_adv_upgrade.value - props.upgrade.upgrade_index * 10) * 10 + current_adv_xp.value / 10,
            current_grace_progress.value,
            next_free.value,
            next_big.value,
        ];
    }

    start_all_workers();
}
</script>

<template>
    <input
        v-if="upgrade.is_normal_honing"
        type="number"
        v-model.number="artisan"
        min="0"
        :max="99.99"
        @change="write_progress()"
    />
    <div v-if="!upgrade.is_normal_honing">
        <div class="input-row">
            <label>Current upgrade</label>
            <input
                type="number"
                v-model.number="current_adv_upgrade"
                :min="upgrade.upgrade_index * 10"
                :max="(upgrade.upgrade_index + 1) * 10 - 1"
                @change="write_progress"
            />
        </div>

        <div class="input-row">
            <label>Current xp</label>
            <input
                type="number"
                v-model.number="current_adv_xp"
                min="0"
                max="90"
                step="10"
                style="justify-self: flex-start"
                @change="write_progress"
            />
        </div>
        <div class="input-row grid-4">
            <label>Grace progress</label>
            <input type="number" v-model.number="current_grace_progress" min="0" max="6" @change="write_progress" />

            <label
                v-if="
                    current_grace_progress === 0 &&
                    (current_adv_xp > 0 || current_adv_upgrade > upgrade.upgrade_index * 10)
                "
                class="check-label"
            >
                <input type="checkbox" v-model="next_free" @change="write_progress" /> Next free (Chisel)
            </label>
            <label v-if="current_grace_progress === 6 && upgrade.upgrade_index >= 2" class="check-label">
                <input type="checkbox" v-model="next_big" @change="write_progress" /> Naber's Awl
            </label>
        </div>
    </div>
</template> -->
