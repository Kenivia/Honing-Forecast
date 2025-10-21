import React, { useState, useEffect } from "react"
import "./separator.css"

type Page = "cost-to-chance" | "gamba" | "forecast" //"chance-to-cost" |

type SeparatorProps = {
    activePage: Page
    onPageChange: (_page: Page) => void
}

export default function Separator({ activePage, onPageChange }: SeparatorProps) {
    const [_clickedButton, setClickedButton] = useState<Page | null>(activePage)

    useEffect(() => {
        setClickedButton(activePage)
    }, [activePage])

    const handleButtonClick = (page: Page) => {
        onPageChange(page)
        setClickedButton(page)
    }

    const getSeparatorClass = () => {
        // if (activePage === "chance-to-cost") return "hf-separator chance-active"
        if (activePage === "cost-to-chance") return "hf-separator budget-active"
        if (activePage === "gamba") return "hf-separator gamba-active"
        if (activePage === "forecast") return "hf-separator longterm-active"
        return "hf-separator"
    }

    const getButtonClass = (page: Page) => {
        const baseClass = "hf-btn"
        if (activePage === page) {
            // if (page === "chance-to-cost") return `${baseClass} chance-selected`
            if (page === "cost-to-chance") return `${baseClass} budget-selected`
            if (page === "gamba") return `${baseClass} gamba-selected`
            if (page === "forecast") return `${baseClass} longterm-selected`
        }
        return baseClass
    }

    return (
        <div className={getSeparatorClass()}>
            {/* <button className={getButtonClass("chance-to-cost")} onClick={() => handleButtonClick("chance-to-cost")}>
                <span className="hf-label">Chance mode</span>
                <div className="hf-help">I want to have x% chance to pass, how much mats will I need?</div>
            </button> */}

            <button className={getButtonClass("cost-to-chance")} onClick={() => handleButtonClick("cost-to-chance")}>
                <span className="hf-label">idk what name this should be</span>
                <div className="hf-help">I want x% chance of success, how much will it cost?</div>
            </button>

            <button className={getButtonClass("forecast")} onClick={() => handleButtonClick("forecast")}>
                <span className="hf-label">Forecast mode</span>
                <div className="hf-help">If I earn this much per week, what are my odds of success in x weeks?</div>
            </button>

            <button className={getButtonClass("gamba")} onClick={() => handleButtonClick("gamba")}>
                <span className="hf-label">Gamba simulator</span>
                <div className="hf-help">Simulate a honing session without hurting your wallet or your soul</div>
            </button>
        </div>
    )
}
