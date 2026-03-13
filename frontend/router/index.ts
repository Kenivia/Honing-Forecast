import { createRouter, createWebHistory } from "vue-router"
import CharView from "../Components/Character/CharView.vue"
import RosterView from "../Components/Roster/RosterView.vue"

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            name: "roster",
            component: RosterView,
        },
        {
            path: "/YourChar",
            name: "char",
            component: CharView,
        },
        // { path: "/:pathMatch(.*)*", redirect: "/" },
    ],
})

export default router
