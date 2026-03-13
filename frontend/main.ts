import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import "./theme.css"
import "./index.css"
import "./redesign.css"
import router from "./router"
const original = window.btoa
window.btoa = function (...args) {
    console.trace("btoa called with:", args[0])
    return original.apply(this, args)
}
const pinia = createPinia()
const app = createApp(App)
app.use(pinia)
app.use(router)
app.mount("#root")
