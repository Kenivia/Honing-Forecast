import { useRosterStore } from "@/stores/roster_config";
import { storeToRefs } from "pinia";
import { nextTick } from "vue";

export async function force_rerender() {
    const { roster_config } = storeToRefs(useRosterStore());

    roster_config.value.market_rerender_trigger = false;
    await nextTick();
    roster_config.value.market_rerender_trigger = true;
}
