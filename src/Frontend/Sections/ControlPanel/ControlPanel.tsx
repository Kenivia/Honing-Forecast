import React from 'react'
import { styles } from '@/Frontend/Utils/Styles.ts'
import LabeledCheckbox from '@/Frontend/Components/LabeledCheckbox.tsx'

type ControlPanelProps = {
    controlsLeft: number | null
    mainScale: number
    fillDemo: () => void
    fillDemoIncome: () => void
    // fillRandom: () => void
    clearAll: () => void
    express_event: boolean
    set_express_event: (_next: boolean) => void
    cumulativeGraph: boolean
    setCumulativeGraph: (_next: boolean) => void
    dataSize: string
    setDataSize: (_v: string) => void
    lockXAxis: boolean
    onToggleLockXAxis: () => void
    useGridInput: boolean
    setUseGridInput: (_next: boolean) => void
}

export default function ControlPanel({ controlsLeft: _controlsLeft, mainScale: _mainScale, fillDemo, fillDemoIncome, clearAll, express_event, set_express_event, cumulativeGraph, setCumulativeGraph, dataSize, setDataSize, lockXAxis, onToggleLockXAxis, useGridInput, setUseGridInput }: ControlPanelProps) {
    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 0, minWidth: 200, flexShrink: 0 }}>
            <h3 style={{ ...styles.sectionTitle, marginTop: '-8px', alignSelf: 'center' }}>Controls</h3>
            <div style={{ ...styles.buttonSection, marginTop: '-8px', width: '200px' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-sm)', width: '100%' }}>
                    <button style={styles.demoButton} onClick={fillDemo}>Fill Demo</button>
                    <button style={styles.demoButton} onClick={fillDemoIncome}>Fill Demo Income</button>
                    {/* <button style={styles.demoButton} onClick={fillRandom}>Fill Random</button> */}
                    <button style={styles.demoButton} onClick={clearAll}>Reset All</button>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginTop: '8px' }}>
                        <LabeledCheckbox
                            label="Express event"
                            checked={express_event}
                            setChecked={set_express_event}
                        />
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <LabeledCheckbox
                            label="Cumulative Graph"
                            checked={cumulativeGraph}
                            setChecked={setCumulativeGraph}
                        />
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <LabeledCheckbox
                            label="Grid input"
                            checked={useGridInput}
                            setChecked={setUseGridInput}
                        />
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <label htmlFor="data_size" style={{ color: 'var(--text-primary)', fontSize: 'var(--font-size-sm)', cursor: 'pointer', textWrap: 'nowrap' }}>Sample size</label>
                        <input type="text" id="data_size" value={dataSize} onChange={(e) => {
                            let v = e.target.value.replace(/[^0-9]/g, '')
                            v = v.replace(/^0+(?=\d)/, '')
                            setDataSize(v)
                        }} onBlur={() => {
                            const n = Math.min(1000000, Math.max(1000, Math.floor(Number(dataSize) || 0)))
                            setDataSize(String(n))
                        }} style={{ width: 80, fontSize: 14, padding: '6px 8px', borderRadius: 6, background: 'var(--input-bg)', color: 'var(--input-text)', border: '1px solid var(--input-border)' }} placeholder="100000" />
                    </div>

                    <button
                        style={{
                            ...styles.demoButton,
                            background: lockXAxis ? 'var(--btn-toggle-lock-selected)' : "var(--btn-demo)",
                            color: lockXAxis ? '#000000' : 'var(--btn-demo-text)',
                            width: '100%'
                        }}
                        onClick={onToggleLockXAxis}
                        title="Lock the x-axis to compare the costs of 2 selections"
                    >
                        {lockXAxis ? 'x-axis Locked' : 'Lock x-axis'}
                    </button>
                </div>
            </div>
        </div>
    )
}



