import React from 'react'
import { styles } from './styles.ts'

type ControlPanelProps = {
    controlsLeft: number | null
    mainScale: number
    fillDemo: () => void
    fillRandom: () => void
    clearAll: () => void
    express_event: boolean
    set_express_event: (_next: boolean) => void
    cumulativeGraph: boolean
    setCumulativeGraph: (_next: boolean) => void
    dataSize: string
    setDataSize: (_v: string) => void
    lockXAxis: boolean
    onToggleLockXAxis: () => void
}

export default function ControlPanel({ controlsLeft: _controlsLeft, mainScale: _mainScale, fillDemo, fillRandom, clearAll, express_event, set_express_event, cumulativeGraph, setCumulativeGraph, dataSize, setDataSize, lockXAxis, onToggleLockXAxis }: ControlPanelProps) {
    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 0, minWidth: 200, flexShrink: 0 }}>
            <h3 style={{ ...styles.sectionTitle, marginTop: '-8px', alignSelf: 'center' }}>Controls</h3>
            <div style={{ ...styles.buttonSection, marginTop: '-8px', width: '200px' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-sm)', width: '100%' }}>
                    <button style={styles.demoButton} onClick={fillDemo}>Fill Demo</button>
                    <button style={styles.demoButton} onClick={fillRandom}>Fill Random</button>
                    <button style={styles.demoButton} onClick={clearAll}>Reset All</button>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginTop: '8px' }}>
                        <label htmlFor="express_event" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer' }}>Express event</label>
                        <input
                            type="checkbox"
                            id="express_event"
                            checked={express_event}
                            onChange={(e) => set_express_event(e.target.checked)}
                            style={{
                                width: '16px',
                                height: '16px',
                                cursor: 'pointer',
                                accentColor: 'var(--control-checked-bg)'
                            }}
                        />
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <label htmlFor="cumulative_graph" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer' }}>Cumulative Graph</label>
                        <input
                            type="checkbox"
                            id="cumulative_graph"
                            checked={cumulativeGraph}
                            onChange={(e) => setCumulativeGraph(e.target.checked)}
                            style={{
                                width: '16px',
                                height: '16px',
                                cursor: 'pointer',
                                accentColor: 'var(--control-checked-bg)'
                            }}
                        />
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <label htmlFor="data_size" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer', textWrap: 'nowrap' }}>Trial count</label>
                        <input type="text" id="data_size" value={dataSize} onChange={(e) => {
                            let v = e.target.value.replace(/[^0-9]/g, '')
                            v = v.replace(/^0+(?=\d)/, '')
                            setDataSize(v)
                        }} onBlur={() => {
                            const n = Math.min(1000000, Math.max(1000, Math.floor(Number(dataSize) || 0)))
                            setDataSize(String(n))
                        }} style={{ width: 80, fontSize: 14, padding: '6px 8px', borderRadius: 6, background: 'var(--input-bg)', color: 'var(--input-text)', border: '1px solid var(--input-border)' }} placeholder="100000" />
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <label htmlFor="lock_x_axis" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer' }}>Lock x-axis</label>
                        <input
                            type="checkbox"
                            title="Lock the x-axis to compare the costs of 2 selections"
                            id="lock_x_axis"
                            checked={lockXAxis}
                            onChange={onToggleLockXAxis}
                            style={{
                                width: '16px',
                                height: '16px',
                                cursor: 'pointer',
                                accentColor: 'var(--control-checked-bg)'
                            }}
                        />
                    </div>
                </div>
            </div>
        </div>
    )
}



