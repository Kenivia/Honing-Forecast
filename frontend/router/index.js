import { createRouter, createWebHistory } from "vue-router"
import CharView from "../Character/CharView.vue"
import RosterView from "../Roster/RosterView.vue"

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

// The below example doesn't use $reset because restaurantsStore is a setup store
// $reset() does not work for setup stores, only option stores
// So I've created a function resetRestaurantsStore() that does the same thing

// router.beforeEach((to) => {
//   const restaurantsStore = useRestaurantsStore();
//   if (to.name === "home") restaurantsStore.resetRestaurantsStore();
// });

export default router
