import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "@/App.vue";
import "@/theme.css";
import "@/shared.css";
import "@/index.css";
import "@/sidebar.css";
import router from "@/Router";

const pinia = createPinia();
const app = createApp(App);
app.use(pinia);
app.use(router);
app.mount("#root");
