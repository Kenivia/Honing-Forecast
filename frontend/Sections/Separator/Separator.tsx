import React, { useState, useEffect } from "react"
import "./Separator.css"

type Page = "optimize" | "distribution" | "gamba" | "forecast" //"chance-to-cost" |

type SeparatorProps = {
    activePage: Page
    onPageChange: (_page: Page) => void
    setAutoRunOptimizer: React.Dispatch<React.SetStateAction<boolean>>
}

export default function Separator({ activePage, onPageChange, setAutoRunOptimizer }: SeparatorProps) {
    const [_clickedButton, setClickedButton] = useState<Page | null>(activePage)
    const [hasAutoStartedOptimizer, setHasAutoStartedOptimizer] = useState<boolean>(false)

    useEffect(() => {
        setClickedButton(activePage)
    }, [activePage])

    const handleButtonClick = (page: Page) => {
        if (page !== "optimize") {
            setAutoRunOptimizer(false)
        } else if (!hasAutoStartedOptimizer) {
            // setAutoRunOptimizer(true)
            setHasAutoStartedOptimizer(true)
        }
        onPageChange(page)
        setClickedButton(page)
    }

    const getSeparatorClass = () => {
        if (activePage === "optimize") return "hf-separator optimize-active"
        if (activePage === "distribution") return "hf-separator budget-active"
        if (activePage === "gamba") return "hf-separator gamba-active"
        if (activePage === "forecast") return "hf-separator longterm-active"
        return "hf-separator"
    }

    const getButtonClass = (page: Page) => {
        const baseClass = "hf-btn"
        if (activePage === page) {
            if (page === "optimize") return `${baseClass} optimize-selected`
            if (page === "distribution") return `${baseClass} budget-selected`
            if (page === "gamba") return `${baseClass} gamba-selected`
            if (page === "forecast") return `${baseClass} longterm-selected`
        }
        return baseClass
    }

    return (
        <div className={getSeparatorClass()}>
            <button className={getButtonClass("distribution")} onClick={() => handleButtonClick("distribution")}>
                <span className="hf-label">Cool graphs</span>
                <div className="hf-help">How much will the upgrades cost?</div>
            </button>
            <button className={getButtonClass("optimize")} onClick={() => handleButtonClick("optimize")}>
                <span className="hf-label">Optimizer</span>
                <div className="hf-help">How should I use my free taps & juice?</div>
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
