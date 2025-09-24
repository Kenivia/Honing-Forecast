// Graph.test.tsx
import React from 'react'
import { render, screen, cleanup } from '@testing-library/react'
import { userEvent } from '@testing-library/user-event'
import { describe, test, expect, vi, beforeEach } from 'vitest'
import Graph from './Graph.tsx'

// Mock visx XYChart components so tests don't try to render native canvas/svg guts.
// We keep children rendering so our DOM structure remains predictable.
vi.mock('@visx/xychart', () => {
    const Dummy = ({ children }: any) => children
    return {
        XYChart: Dummy,
        AnimatedAxis: Dummy,
        AnimatedGrid: Dummy,
        AnimatedLineSeries: ({ children, ...props }: any) => <div data-testid={`line-${props.dataKey}`}>{children}</div>,
        Tooltip: ({ children }: any) => <div data-testid="tooltip">{children}</div>,
        darkTheme: {},
    }
})

// localPoint is used in the component but not required for our tests; mock a noop.
vi.mock('@visx/event', () => ({
    localPoint: () => ({ x: 0, y: 0 }),
}))

describe('Graph component (unit tests)', () => {
    const baseLabels = ['Red', 'Blue', 'Leaps', 'Shards', 'Oreha', 'Gold', 'Silver']

    beforeEach(() => {
        cleanup()
    })

    test('renders title and legend buttons only for series that pass keepMask', () => {
        // Create counts so that only indices 0 and 5 have more than one positive bin (keepMask true).
        // Each series length = 3 (small for tests)
        const counts = [
            [1, 2, 0], // positive bins = 2 -> keep
            [0, 0, 0], // no positives -> not kept
            [0, 0, 0],
            [0, 1, 0], // only 1 positive -> NOT kept (needs >1)
            [0, 0, 0],
            [3, 4, 0], // positive bins = 2 -> keep (Gold)
            [0, 0, 0],
        ]

        render(
            <Graph
                title="My Graph"
                labels={baseLabels}
                counts={counts}
                mins={[0, 0, 0, 0, 0, 0, 0]}
                maxs={[10, 10, 10, 10, 10, 10, 10]}
                width={400}
                height={200}
            />
        )

        // Title appears
        expect(screen.getByText('My Graph')).toBeInTheDocument()

        // Only labels with keepMask true should produce legend buttons => 'Red' and 'Gold'
        expect(screen.queryByText('Red')).toBeInTheDocument()
        expect(screen.queryByText('Gold')).toBeInTheDocument()
        // A label that should not be kept should not render in legend
        expect(screen.queryByText('Blue')).not.toBeInTheDocument()
        expect(screen.queryByText('Leaps')).not.toBeInTheDocument()
        expect(screen.queryByText('Shards')).not.toBeInTheDocument()
        expect(screen.queryByText('Oreha')).not.toBeInTheDocument()
        expect(screen.queryByText('Silver')).not.toBeInTheDocument()
    })

    test('shows "Nothing to plot, tick an upgrade!" when counts are missing and hasSelection false', () => {
        render(
            <Graph
                labels={baseLabels}
                counts={null}
                mins={null}
                maxs={null}
                hasSelection={false}
                isLoading={false}
                width={300}
                height={150}
            />
        )

        expect(screen.getByText('Nothing to plot, tick an upgrade!')).toBeInTheDocument()
    })

    test('shows "All your ticks have 100% success rate(+1 to +3)" when no visible series and hasSelection true (not loading)', () => {
        // Provide counts such that keepMask will be false for everything (e.g., all zeros or single positive)
        const counts = [
            [1, 0, 0], // only 1 positive -> not enough (needs >1)
            [0, 0, 0],
            [0, 0, 0],
            [0, 1, 0], // only 1 positive
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
        ]

        render(
            <Graph
                labels={baseLabels}
                counts={counts}
                mins={[0, 0, 0, 0, 0, 0, 0]}
                maxs={[1, 1, 1, 1, 1, 1, 1]}
                hasSelection={true}
                isLoading={false}
                width={300}
                height={150}
            />
        )

        expect(screen.getByText('1. All your ticks have 100% success rate(+1 to +3)')).toBeInTheDocument()
    })

    test('clicking a legend button toggles its visibility (opacity change)', async () => {
        const counts = [
            [1, 2, 0], // keep
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [2, 3, 0], // keep (Gold)
            [0, 0, 0],
        ]

        render(
            <Graph
                labels={baseLabels}
                counts={counts}
                mins={[0, 0, 0, 0, 0, 0, 0]}
                maxs={[10, 10, 10, 10, 10, 10, 10]}
                width={400}
                height={200}
            />
        )

        const redButton = screen.getByRole('button', { name: /Red/i })
        // initial visible for Red in the component is true, so opacity should be 1 (string includes 'opacity: 1' inline style)
        // Note: style is inline; testing-library exposes it via getComputedStyle; simpler to check attribute style contains opacity: 1
        expect(redButton.getAttribute('style')?.includes('opacity: 1')).toBe(true)

        // click to toggle visibility -> should flip to opacity 0.4
        await userEvent.click(redButton)
        expect(redButton.getAttribute('style')?.includes('opacity: 0.4')).toBe(true)

        // click again -> back to opacity 1
        await userEvent.click(redButton)
        expect(redButton.getAttribute('style')?.includes('opacity: 1')).toBe(true)
    })
})
