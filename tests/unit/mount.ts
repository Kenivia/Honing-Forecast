import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import {
  createMemoryHistory,
  createRouter,
  type RouteRecordRaw,
} from "vue-router";

// The app's route names, each rendering nothing, so components can push / link without loading every page.
const Empty = { render: () => null };
const ROUTES: RouteRecordRaw[] = [
  { path: "/market-mats", name: "market", component: Empty },
  { path: "/roster-setup", name: "roster", component: Empty },
  { path: "/change-logs", name: "change-logs-root", component: Empty },
  { path: "/change-logs/:version", name: "change-logs", component: Empty },
  {
    path: "/:characterName",
    name: "char",
    component: Empty,
    children: [
      { path: "calc", name: "calc", component: Empty },
      { path: "guide", name: "char-guide", component: Empty },
      { path: "scanner", name: "char-scanner", component: Empty },
      { path: "income", name: "char-income", component: Empty },
      { path: "choices", name: "char-choices", component: Empty },
    ],
  },
];

// Mounts with a fresh pinia (also made active) and a memory router at `path`.
export async function mount_app(
  component: any,
  options: any = {},
  path = "/roster-setup",
) {
  const pinia = createPinia();
  setActivePinia(pinia);
  const router = createRouter({
    history: createMemoryHistory(),
    routes: ROUTES,
  });
  await router.push(path);
  await router.isReady();
  const wrapper = mount(component, {
    ...options,
    global: { ...options.global, plugins: [pinia, router] },
  });
  return { wrapper, pinia, router };
}
