import { createRouter, createWebHistory } from "vue-router";
import type { RouteLocation } from "vue-router";
import CharView from "../Components/Character/CharView.vue";
import Calc from "@/Components/Character/Calc.vue";
import Guide from "@/Components/Info/Guide.vue";
import Scanner from "@/Components/Character/InventoryScanner/Scanner.vue";
import MarketView from "@/Components/Market/MarketView.vue";
import RosterView from "@/Components/RosterView.vue";
import { REGION_SLUGS, useRosterStore } from "@/Stores/RosterConfig";
import type { CharProfile } from "@/Stores/CharacterProfile";
import { ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog";
import InfoView from "@/Components/Info/InfoView.vue";
import ChangeLog from "@/Components/Info/ChangeLog.vue";
import Income from "@/Components/Character/Income/Income.vue";
import Choices from "@/Components/Character/Choices/Choices.vue";
import { guard_switch } from "@/Components/Character/InventoryScanner/ScanSwitch";
import AuthCallback from "@/Components/Common/Uwuowo/AuthCallback.vue";
import UpgradeOrder from "@/Components/Info/Docs.vue";

const REGIONS = Object.values(REGION_SLUGS).join("|");
const char_path = (p: CharProfile) => {
  const { region, characterName } = useRosterStore().char_params(p);
  return `/chars/${region}/${encodeURIComponent(characterName)}`;
};
const base = (c: RouteLocation) =>
  `/chars/${c.params.region}/${encodeURIComponent(c.params.characterName as string)}`;
// Redirects run before the store has loaded on the first navigation, so this only guesses the
// region; the guard below moves it to wherever that name is.
const old_char = (name: string | string[], rest: string | string[]) =>
  `/chars/NA/${encodeURIComponent(name as string)}/${rest ?? ""}`;

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/:chars(chars)?",
      redirect: () => {
        const store = useRosterStore();
        const first = store.all_profiles[0];
        return first ? char_path(first) : "/roster-setup"; // there should always be at least one tho
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
      path: "/info",
      name: "info",
      component: InfoView,
      redirect: { name: "info-guide" },
      children: [
        { path: "guide", name: "info-guide", component: Guide },
        {
          path: "overview",
          name: "info-overview",
          component: UpgradeOrder,
        },
        {
          path: "change-logs",
          name: "change-logs-root",
          redirect: () => ({
            name: "change-logs",
            params: { version: LATEST_VERSION },
          }),
        },
        {
          path: "change-logs/:version",
          name: "change-logs",
          component: ChangeLog,
        },
        {
          path: "change-logs/:pathMatch(.*)*",
          redirect: () => `/info/change-logs/${LATEST_VERSION}`,
        },
        { path: ":pathMatch(.*)*", redirect: { name: "info-guide" } },
      ],
    },
    // old urls
    {
      path: "/change-logs/:version?",
      redirect: (to) =>
        `/info/change-logs/${to.params.version || LATEST_VERSION}`,
    },
    {
      path: "/change-logs/:pathMatch(.*)*",
      redirect: () => `/info/change-logs/${LATEST_VERSION}`,
    },
    {
      path: "/auth/callback",
      name: "auth-callback",
      component: AuthCallback,
    },
    {
      path: `/chars/:region(${REGIONS})/:characterName`,
      name: "char",
      component: CharView,
      redirect: (c) => `${base(c)}/calc`,
      children: [
        { path: "calc", name: "calc", component: Calc },
        { path: "guide", redirect: { name: "info-guide" } },
        { path: "scanner", name: "char-scanner", component: Scanner },
        // Income is dev only until released
        ...(import.meta.env.DEV
          ? [{ path: "income", name: "char-income", component: Income }]
          : []),
        { path: "choices", name: "char-choices", component: Choices },
        { path: ":x/guide", redirect: { name: "info-guide" } },
        {
          path: ":x/scanner",
          redirect: (c) => `${base(c)}/scanner`,
        },
        {
          path: ":pathMatch(.*)*",
          redirect: (c) => `${base(c)}/calc`,
        },
      ],
    },
    // old character urls without a region: the first character of that name
    {
      path: "/chars/:characterName/:rest(.*)?",
      redirect: (c) => old_char(c.params.characterName, c.params.rest),
    },
    // before /chars
    {
      path: "/:characterName/:rest(.*)?",
      redirect: (c) => old_char(c.params.characterName, c.params.rest),
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
  const region = to.params.region as string | undefined;
  if (name === undefined || region === undefined) return;
  const store = useRosterStore();
  if (store.profile_index_by_route(region, name) < 0) {
    const found = store.profile_index_by_name(name);
    return {
      name: to.name,
      params: {
        ...to.params,
        ...store.char_params(store.all_profiles[Math.max(0, found)]),
      },
      query: to.query,
      hash: to.hash,
    };
  }
});
router.beforeEach(guard_switch);

// SEO: canonical always on the .com (character pages all render the same to a crawler, so they point at /)
const head_tag = (sel: string, tag: string, attrs: Record<string, string>) => {
  let el = document.head.querySelector(sel);
  if (!el) {
    el = document.createElement(tag);
    document.head.appendChild(el);
  }
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
  return el;
};
router.afterEach((to) => {
  const path = to.path.startsWith("/chars/") ? "/" : to.path;
  head_tag("link[rel=canonical]", "link", {
    rel: "canonical",
    href: "https://honing-forecast.com" + path,
  });
  if (to.name === "auth-callback")
    head_tag("meta[name=robots]", "meta", {
      name: "robots",
      content: "noindex",
    });
  else document.head.querySelector("meta[name=robots]")?.remove();
});

export default router;
