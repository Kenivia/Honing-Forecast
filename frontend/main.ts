import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "@/App.vue";
import "@/theme.css";
import "@/shared.css";
import "@/index.css";
import "@/sidebar.css";
import router from "@/Router";

// index.html's domain-move sender is taking this visit to the new site
if (!(window as any).hf_leaving) {
  const pinia = createPinia();
  const app = createApp(App);
  app.use(pinia);
  app.use(router);
  app.mount("#root");
}
