import { createApp } from "vue"
import { createPinia } from "pinia"
import Tooltip from "primevue/tooltip"

import App from "./App.vue"
import "./theme.css"
import "./index.css"
import "./redesign.css"
import router from "./Router"

const pinia = createPinia()
const app = createApp(App)
app.use(pinia)
app.use(router)
app.mount("#root")
app.directive("tooltip", Tooltip)
