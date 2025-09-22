import React from 'react'
import { styles } from './styles.ts'

type HoningControlsProps = {
    controlsLeft: number | null
    fillDemo: () => void
    fillRandom: () => void
    clearAll: () => void
    express_event: boolean
    set_express_event: (_next: boolean) => void
    cumulativeGraph: boolean
    setCumulativeGraph: (_next: boolean) => void
    dataSize: string
    setDataSize: (_v: string) => void
}

export default function HoningControls({ controlsLeft, fillDemo, fillRandom, clearAll, express_event, set_express_event, cumulativeGraph, setCumulativeGraph, dataSize, setDataSize }: HoningControlsProps) {
    return (
        <div style={{ position: 'fixed', left: controlsLeft ?? undefined, right: controlsLeft == null ? 0 : undefined, top: '50%', transform: 'translateY(-50%)', width: 200, zIndex: 1000 }}>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 0, width: 200 }}>
                <h3 style={{ ...styles.sectionTitle, marginTop: '-8px', alignSelf: 'center' }}>Controls</h3>
                <div style={{ ...styles.buttonSection, marginTop: '-8px', width: '200px' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-sm)', width: '100%' }}>
                        <button style={styles.demoButton} onClick={fillDemo}>Fill Demo</button>
                        <button style={styles.demoButton} onClick={fillRandom}>Fill Random</button>
                        <button style={styles.demoButton} onClick={clearAll}>Reset All</button>

                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginTop: '8px' }}>
                            <label htmlFor="express_event" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer' }}>Express event</label>
                            <input type="checkbox" id="express_event" checked={express_event} onChange={(e) => set_express_event(e.target.checked)} style={{ width: '16px', height: '16px', cursor: 'pointer' }} />
                        </div>

                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                            <label htmlFor="cumulative_graph" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer' }}>Cumulative Graph</label>
                            <input type="checkbox" id="cumulative_graph" checked={cumulativeGraph} onChange={(e) => setCumulativeGraph(e.target.checked)} style={{ width: '16px', height: '16px', cursor: 'pointer' }} />
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
                    </div>
                </div>
            </div>
        </div>
    )
}


