import { createRouter, createWebHistory } from "vue-router";
import CharView from "../Components/Character/CharView.vue";
import Calc from "@/Components/Character/Calc.vue";
import Guide from "@/Components/Character/Guide.vue";
import Setup from "@/Components/Character/InventoryScanner/Setup/Setup.vue";
import Scanner from "@/Components/Character/InventoryScanner/Scanner.vue";
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
      beforeEnter: (to) => {
        const roster_store = useRosterStore();
        const name = to.params.characterName as string;
        const match = roster_store.all_profiles.findIndex(
          (c) => c.char_name === name,
        );
        if (match < 0) {
          return {
            name: to.name ?? "char",
            params: { characterName: roster_store.all_profiles[0].char_name },
          };
        }
      },
      children: [
        { path: "calc", name: "calc", component: Calc },
        { path: "guide", name: "char-guide", component: Guide },
        { path: "setup", name: "char-scanner-setup", component: Setup },
        { path: "scanner", name: "char-scanner", component: Scanner },
        {
          path: ":x/guide",
          redirect: (c) => `/${c.params.characterName}/guide`,
        },
        {
          path: ":x/scanner",
          redirect: (c) => `/${c.params.characterName}/scanner`,
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
