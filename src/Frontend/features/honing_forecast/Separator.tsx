import React, { useState, useEffect } from 'react';
import './separator.css';


type Page = 'chance-to-cost' | 'cost-to-chance' | 'gamba';

type SeparatorProps = {
    activePage: Page;
    onPageChange: (_page: Page) => void;
};

export default function Separator({ activePage, onPageChange }: SeparatorProps) {
    const [_clickedButton, setClickedButton] = useState<Page | null>(activePage);

    useEffect(() => {
        setClickedButton(activePage);
    }, [activePage]);

    const handleButtonClick = (page: Page) => {
        onPageChange(page);
        setClickedButton(page);
    };

    return (
        <div className="hf-separator">
            <button
                className={`hf-btn ${activePage === 'chance-to-cost' ? 'selected' : ''}`}
                onClick={() => handleButtonClick('chance-to-cost')}
            >
                <span className="hf-label">Chance mode</span>
                <div className="hf-help">
                    I want to have x% chance to pass, how much mats will I need?
                </div>
            </button>

            <button
                className={`hf-btn ${activePage === 'cost-to-chance' ? 'selected' : ''}`}
                onClick={() => handleButtonClick('cost-to-chance')}
            >
                <span className="hf-label">Budget mode</span>
                <div className="hf-help">
                    I have this much mats, what are my odds of success?
                </div>
            </button>

            <button
                className={`hf-btn ${activePage === 'gamba' ? 'selected' : ''}`}
                onClick={() => handleButtonClick('gamba')}
            >
                <span className="hf-label">Gamba simulator</span>
                <div className="hf-help">Simulate a honing session without hurting your wallet or your soul</div>
            </button>
        </div>
    );
}
