import { createApp } from "vue"
import App from "./App.vue"
import "./theme.css"
import "./index.css"
import "./redesign.css"
import { createPinia } from "pinia"

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
createApp(App).mount("#root")
