import { createRouter, createWebHistory } from "vue-router"
import CharView from "../Components/Character/CharView.vue"
import {} from "@/Stores/CharacterProfile"
import MarketView from "@/Components/Roster/MarketView.vue"
import RosterView from "@/Components/RosterView.vue"
import { useRosterStore } from "@/Stores/RosterConfig"

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            redirect: () => {
                const roster_store = useRosterStore()
                const first = roster_store.all_profiles[0][0]
                return first ? `/${first.char_name}` : "/roster-setup" // there should always be at least one tho
            },
        },
        {
            path: "/:characterName",
            name: "char",
            component: CharView,
        },
        {
            path: "/market-mats",
            name: "market",
            component: MarketView,
        },
        {
            path: "/roster-setup",
            name: "roster",
            component: RosterView,
        },
    ],
})

export default router
