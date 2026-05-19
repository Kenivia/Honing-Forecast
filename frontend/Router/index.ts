import { createRouter, createWebHistory } from "vue-router";
import CharView from "../Components/Character/CharView.vue";
import MarketView from "@/Components/Market/MarketView.vue";
import RosterView from "@/Components/RosterView.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog";
import ChangeLogsView from "@/Components/ChangeLogsView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: () => {
        const roster_store = useRosterStore();
        const first = roster_store.all_profiles[0];
        return first ? `/${first.char_name}` : "/roster-setup"; // there should always be at least one tho
      },
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
      path: "/change-logs",
      name: "change-logs-root",
      redirect: () => ({
        name: "change-logs",
        params: { version: LATEST_VERSION },
      }),
      children: [
        {
          path: ":version",
          name: "change-logs",
          component: ChangeLogsView,
          beforeEnter: (to) => {
            if (
              (to.params.version as string) !== "WIP" &&
              !ALL_VERSIONS.includes(to.params.version as string)
            ) {
              return {
                name: "change-logs",
                params: { version: LATEST_VERSION },
              };
            }
          },
        },
        {
          path: ":pathMatch(.*)*",
          redirect: () => `/change-logs/${LATEST_VERSION}`,
        },
      ],
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
        // {
        //     path: "instructions",
        //     name: "instructions",
        //     component: CharView,
        // },
        {
          path: "guide",
          name: "char-guide",
          component: CharView,
        },
        {
          path: ":pathMatch(.*)*",
          redirect: (c) => `/${c.params.characterName}/calc`,
        },
      ],
    },
  ],
});

export default router;
