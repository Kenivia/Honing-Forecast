import React from 'react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup } from '@testing-library/react'
import '@testing-library/jest-dom/vitest';
import HoningForecastUI from './HoningForecastUI.tsx'

// Mock heavy/charting and worker wiring to keep unit test light
vi.mock('../../components/Graph.tsx', () => ({
    __esModule: true,
    default: ({ title }: any) => <div data-testid="graph">{title || 'Graph'}</div>,
}))

vi.mock('../../components/SpreadsheetGrid.tsx', () => ({
    __esModule: true,
    default: ({ labels, sheet_values }: any) => (
        <div data-testid="sheet">
            {labels.map((l: string) => (
                <div key={l}>{l}:{String(sheet_values?.[l] ?? '')}</div>
            ))}
        </div>
    ),
}))

vi.mock('../../components/Icon.tsx', () => ({
    __esModule: true,
    default: ({ iconName }: any) => <span data-testid="icon">{iconName}</span>,
}))

vi.mock('../../worker_setup.ts', () => ({
    __esModule: true,
    SpawnWorker: (_payload: any, which_one: 'CostToChance' | 'ChanceToCost') => {
        // Resolve with minimal shape expected by UI
        const result =
            which_one === 'ChanceToCost'
                ? { Red: 0, Blue: 0, Leaps: 0, Shards: 0, Oreha: 0, Gold: 0, 'Silver(WIP)': 0, 'Red juice': 0, 'Blue juice': 0, run_time: 0, actual_prob: 0 }
                : { chance: 0, hist_counts: Array.from({ length: 7 }, () => Array(10).fill(0)), hist_mins: Array(7).fill(0), hist_maxs: Array(7).fill(1), run_time: 0, budgets_red_remaining: 0, budgets_blue_remaining: 0 }
        const worker = { terminate: () => { } } as unknown as Worker
        return { worker, promise: Promise.resolve(result) }
    },
}))

beforeEach(() => {
    vi.useFakeTimers()
    // Clear storage to avoid state leakage between tests
    localStorage.clear()
})

afterEach(() => {
    // Flush any pending debounced effects and clear timers to avoid teardown errors
    vi.runOnlyPendingTimers()
    vi.clearAllTimers()
    vi.useRealTimers()
    cleanup()
})

describe('HoningForecastUI', () => {
    it('renders heading and key sections', () => {
        render(<HoningForecastUI />)
        expect(screen.getByText('Honing Forecast')).toBeInTheDocument()
        expect(screen.getByText('Normal Honing')).toBeInTheDocument()
        expect(screen.getByText('Advanced Honing')).toBeInTheDocument()
        expect(screen.getByText('Chance to Cost')).toBeInTheDocument()
        expect(screen.getByText('Cost to Chance')).toBeInTheDocument()
        expect(screen.getAllByTestId('graph')).toHaveLength(2)
    })

    it('updates desired chance input', () => {
        render(<HoningForecastUI />)
        const input = screen.getAllByPlaceholderText('0')[0] as HTMLInputElement
        fireEvent.change(input, { target: { value: '55.5' } })
        expect(input.value).toBe('55.5')
    })

    it('toggles express event checkbox', () => {
        render(<HoningForecastUI />)
        const checkbox = screen.getByLabelText('Express event') as HTMLInputElement
        expect(checkbox.checked).toBe(true)
        fireEvent.click(checkbox)
        expect(checkbox.checked).toBe(false)
    })
})


