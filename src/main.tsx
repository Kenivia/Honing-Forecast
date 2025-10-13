import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import "./Frontend/theme.css"
import "./Frontend/index.css"
import App from "./Frontend/App.tsx"

createRoot(document.getElementById("root")).render(
    <StrictMode>
        <App />
    </StrictMode>
)
