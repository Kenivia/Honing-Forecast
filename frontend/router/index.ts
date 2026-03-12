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
            path: "/:CharName",
            name: "char",
            component: CharView,
            props: true,
        },
        { path: "/:pathMatch(.*)*", redirect: "/" },
    ],
})

export default router
