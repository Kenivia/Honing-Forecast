import { createRouter, createWebHistory } from "vue-router"
import CharView from "../Components/Character/CharView.vue"
import { useProfilesStore } from "@/Stores/CharacterProfile"
import MarketView from "@/Components/Roster/MarketView.vue"
import RosterView from "@/Components/RosterView.vue"

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            redirect: () => {
                const profile_store = useProfilesStore()
                const first = profile_store.profiles[0]
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
