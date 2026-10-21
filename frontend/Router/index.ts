import { createRouter, createWebHistory } from "vue-router";
import CharView from "../Components/Character/CharView.vue";
import Calc from "@/Components/Character/Calc.vue";
import Guide from "@/Components/Character/Guide.vue";
import Scanner from "@/Components/Character/InventoryScanner/Scanner.vue";
import MarketView from "@/Components/Market/MarketView.vue";
import RosterView from "@/Components/RosterView.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog";
import ChangeLogsView from "@/Components/ChangeLogsView.vue";
import Income from "@/Components/Character/Income/Income.vue";
import Choices from "@/Components/Character/Choices/Choices.vue";
import { guard_switch } from "@/Components/Character/InventoryScanner/ScanSwitch";

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
        { path: "calc", name: "calc", component: Calc },
        { path: "guide", name: "char-guide", component: Guide },
        { path: "scanner", name: "char-scanner", component: Scanner },
        // Income is dev only until released
        ...(import.meta.env.DEV ? [{ path: "income", name: "char-income", component: Income }] : []),
        { path: "choices", name: "char-choices", component: Choices },
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

// Every navigation, not only route entry, so param-only changes are checked too. Keeps the sub-page.
router.beforeEach((to) => {
  const version = to.params.version as string | undefined;
  if (
    to.name === "change-logs" &&
    version !== "WIP" &&
    !ALL_VERSIONS.includes(version)
  )
    return { name: "change-logs", params: { version: LATEST_VERSION } };
  const name = to.params.characterName as string | undefined;
  if (name === undefined) return;
  const store = useRosterStore();
  if (store.profile_index_by_name(name) < 0)
    return {
      name: to.name,
      params: { ...to.params, characterName: store.all_profiles[0].char_name },
      query: to.query,
      hash: to.hash,
    };
});
router.beforeEach(guard_switch);

export default router;
