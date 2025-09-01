import { useState, } from "react";
import { CallWorker } from './worker_setup';
// Default export a single React component (drop into src/App.jsx or similar)
export default function UpgradeCalculatorUI() {
    const TOP_ROWS = 6;
    const TOP_COLS = 25;
    const BOTTOM_ROWS = 6;
    const BOTTOM_COLS = 4;

    const INPUT_LABELS = [
        "Red",
        "Blue",
        "Leaps",
        "Shards",
        "Oreha",
        "Gold",
        "Silver(WIP)",
        "Red juice",
        "Blue juice",
        "Special leaps",
    ];

    // Top grid state (6 x 25)
    const [topGrid, setTopGrid] = useState(() =>
        Array.from({ length: TOP_ROWS }, () => Array(TOP_COLS).fill(false))
    );

    // Bottom grid state (6 x 4)
    const [bottomGrid, setBottomGrid] = useState(() =>
        Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false))
    );

    // Inputs column (10 inputs)
    const [budget_inputs, set_budget_inputs] = useState(() =>
        Object.fromEntries(INPUT_LABELS.map((l) => [l, ""]))
    );

    const [desired_chance, set_desired_chance] = useState(() => "") 
    const [adv_hone_strategy, set_adv_hone_strategy_change] = useState(() =>"No juice")

    // Toggle helpers
    const toggleTop = (r, c) => {
        setTopGrid((prev) => {
            const copy = prev.map((row) => row.slice());
            copy[r][c] = !copy[r][c];
            return copy;
        });
    };

    const toggleBottom = (r, c) => {
        setBottomGrid((prev) => {
            const copy = prev.map((row) => row.slice());
            copy[r][c] = !copy[r][c];
            return copy;
        });
    };

    // Input change
    const onBudgetChange = (label, value) => {
        set_budget_inputs((prev) => ({ ...prev, [label]: value }));
    };

    const onDesiredChange = (value) => {
        set_desired_chance(() => (value ));
    };
    const adv_hone_strategy_change = (value) => {
        set_adv_hone_strategy_change(() => (  value ));
    };


    const clearAll = () => {
        setTopGrid(Array.from({ length: TOP_ROWS }, () => Array(TOP_COLS).fill(false)));
        setBottomGrid(Array.from({ length: BOTTOM_ROWS }, () => Array(BOTTOM_COLS).fill(false)));
        set_budget_inputs(Object.fromEntries(INPUT_LABELS.map((l) => [l, ""])));
    };

    const fillRandom = () => {
        setTopGrid(
            Array.from({ length: TOP_ROWS }, () =>
                Array.from({ length: TOP_COLS }, () => Math.random() > 0.7)
            )
        );
        setBottomGrid(
            Array.from({ length: BOTTOM_ROWS }, () =>
                Array.from({ length: BOTTOM_COLS }, () => Math.random() > 0.7)
            )
        );
    };
    const fillMy = () => {
        setTopGrid(
            Array.from({ length: TOP_ROWS }, () =>
                Array.from({ length: TOP_COLS }, (_, ind) => ind == 19 || ind == 20)
            )
        );
        setBottomGrid(
            Array.from({ length: BOTTOM_ROWS }, () =>
                Array.from({ length: BOTTOM_COLS }, (_, ind) => ind == 2)
            )
        );
        set_budget_inputs({ "Red": 431777, "Blue": 1064398, "Leaps": 23748, "Shards": 9010948, "Oreha": 15125, "Gold": 1803792, "Silver(WIP)": 99999999999999, "Red juice": 0, "Blue juice": 0, "Special leaps": 0 })
    };

    const [chance_result, set_chance_result] = useState(null);
    const [cost_result, set_cost_result] = useState(null);
    const [CostToChanceBusy, setCostToChanceBusy] = useState(false);
    const [ChanceToCostBusy, setChanceToCostBusy] = useState(false);

    const HandleCallWorker = async (which_one) => {
        
        try {
            // build a payload from your UI state, e.g. getStateObject()
            if (which_one == "CostToChance") {
                setCostToChanceBusy(true);
            }
            else {
                setChanceToCostBusy(true)
            }
            

            const payload = (which_one == "CostToChance") ? {
                
                normal_hone_ticks: topGrid,
                adv_hone_ticks: bottomGrid,
                budget: budget_inputs
            } :
                {
                    normal_hone_ticks: topGrid,
                    adv_hone_ticks: bottomGrid,
                    desired_chance: desired_chance,
                    adv_hone_strategy: adv_hone_strategy
                };
            const res = await CallWorker(payload, which_one);
            if (which_one == "CostToChance") {
                set_chance_result(res);
            }
            else {
                set_cost_result(res)
            }
        } catch (err) {
            console.error('Worker error', err);
            if (which_one == "CostToChance") {
                set_chance_result({ error: String(err) })
            }
            else {
                set_cost_result({ error: String(err) })
            }
        }
        finally {
            if (which_one == "CostToChance") {
                setCostToChanceBusy(false);
            }
            else {
                setChanceToCostBusy(false)
            }
        }
    };


    // Simple compact styles (so you don't need Tailwind to see a usable layout)
    const styles = {
        container: { fontFamily: "system-ui, Arial, sans-serif", padding: 16, maxWidth: 1100, margin: "0 auto" },
        heading: { marginBottom: 12 },
        gridOuter: { display: "flex", gap: 16, alignItems: "flex-start" },
        topGridWrapper: { overflowX: "auto", },
        grid: (cols) => ({ display: "grid", gridTemplateColumns: `repeat(${cols}, 28px)`, gap: 6 }),
        smallCheckbox: { width: 18, height: 18 },
        bottomGridWrapper: { marginTop: 12, },
        inputsWrapper: { marginLeft: 12, minWidth: 220, display: "flex", flexDirection: "column" },
        inputRow: { display: "flex", alignItems: "center", gap: 8, marginBottom: 8 },
        labelCol: { width: 100, textAlign: "right", paddingRight: 8 },
        controls: { marginTop: 14, display: "flex", gap: 8, flexWrap: "wrap" },
        textarea: { width: "100%", height: 240, marginTop: 12, fontFamily: "monospace", fontSize: 12, padding: 8, borderRadius: 6, border: "1px solid" },
    };

    return (
        <div style={styles.container}>
            <h2 style={styles.heading}>Honing forecast</h2>

            <div style={styles.gridOuter}>
                {/* Left: grids */}
                <div style={{ flex: 1 }}>
                    <div style={styles.topGridWrapper}>
                        <div style={{ marginBottom: 8 }}>Normal honing</div>
                        <div style={styles.grid(TOP_COLS)}>
                            {topGrid.flatMap((row, r) =>
                                row.map((checked, c) => {
                                    const key = `t-${r}-${c}`;
                                    return (
                                        <label key={key} title={`r${r} c${c}`} style={{ display: "inline-block", textAlign: "center" }}>
                                            <input
                                                type="checkbox"
                                                checked={checked}
                                                onChange={() => toggleTop(r, c)}
                                                style={styles.smallCheckbox}
                                            />
                                        </label>
                                    );
                                })
                            )}
                        </div>
                    </div>

                    <div style={styles.bottomGridWrapper}>
                        <div style={{ marginBottom: 8 }}>Advanced honing</div>
                        <div style={styles.grid(BOTTOM_COLS)}>
                            {bottomGrid.flatMap((row, r) =>
                                row.map((checked, c) => (
                                    <label key={`b-${r}-${c}`} style={{ display: "inline-block", textAlign: "center" }}>
                                        <input type="checkbox" checked={checked} onChange={() => toggleBottom(r, c)} style={styles.smallCheckbox} />
                                    </label>
                                ))
                            )}

                        </div>


                    </div>
                    <button onClick={fillMy}>Fill My custom</button>
                    <button onClick={fillRandom}>Fill random (demo)</button>
                    <button onClick={clearAll}>Clear all</button>
                    <select
                        id="my-dropdown"
                        value={adv_hone_strategy}
                        onChange={adv_hone_strategy_change}
                        className="p-2 border rounded"
                    >
                        <option value="No juice">No juice</option>
                        <option value="Full juice on grace">Full juice on grace</option>

                    </select>


                </div>
                <div style={styles.inputsWrapper}>
                    <div style={{ fontWeight: 600, marginBottom: 8 }}>Chance to Cost</div>
                    <div key={"69"} style={styles.inputRow}>
                        <div style={styles.labelCol}>{"Desired chance of success"}</div>

                        <input
                            type="text"
                            value={desired_chance}
                            onChange={(e) => onDesiredChange(e.target.value)}
                            placeholder="0"
                            style={{ flex: 1, padding: "6px 8px", borderRadius: 6, border: "1px solid #ccc" }}
                        />
                        <div style={styles.controls}>
                            <button onClick={() => HandleCallWorker("ChanceToCost")} disabled={ChanceToCostBusy}>{ChanceToCostBusy ? 'Running…' : 'Find chance of success'}</button>
                            

                        </div>
                    </div>
                    <pre>{cost_result ? JSON.stringify(cost_result, null, 2)  : 'No result yet'}</pre>

                </div>
                <div style={styles.inputsWrapper}>
                    <div style={{ fontWeight: 600, marginBottom: 8 }}>Cost to Chance</div>
                    {INPUT_LABELS.map((label) => (
                        <div key={label} style={styles.inputRow}>
                            <div style={styles.labelCol}>{label}</div>
                            <input
                                type="text"
                                value={budget_inputs[label]}
                                onChange={(e) => onBudgetChange(label, e.target.value)}
                                placeholder="0"
                                style={{ flex: 1, padding: "6px 8px", borderRadius: 6, border: "1px solid #ccc" }}
                            />
                        </div>
                    ))}

                    <div style={styles.controls}>
                        <button onClick={() => HandleCallWorker("CostToChance")} disabled={CostToChanceBusy}>{CostToChanceBusy ? 'Running…' : 'Find chance of success'}</button>
                        <pre>{chance_result ? chance_result.chance + "% chance of success\n" + JSON.stringify(chance_result.reason, null, 2) + "\nRun time: " + chance_result.run_time + "s" : 'No result yet'}</pre>

                    </div>

                </div>
                

            </div>

        </div>
    );
}
