import { createRouter, createWebHistory } from "vue-router"
import CharView from "../Components/Character/CharView.vue"
import MarketView from "@/Components/Market/MarketView.vue"
import RosterView from "@/Components/RosterView.vue"
import { useRosterStore } from "@/Stores/RosterConfig"
import ChangelogView from "@/Components/ChangelogView.vue"

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            redirect: () => {
                const roster_store = useRosterStore()
                const first = roster_store.all_profiles[0]
                return first ? `/${first.char_name}` : "/roster-setup" // there should always be at least one tho
            },
        },
        {
            path: "/:characterName",
            name: "char",
            component: CharView,
            redirect: (c) => `/${c.params.characterName}/calc`,
            children: [
                {
                    path: "calc",
                    name: "calc",
                    component: CharView,
                },
                {
                    path: "instructions",
                    name: "instructions",
                    component: CharView,
                },
            ],
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
        {
            path: "/change-log",
            name: "change-log",
            component: ChangelogView,
        },
    ],
})

export default router
