<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue"
import MaterialGraph from "@/Components/MaterialGraph.vue"
import {
    BOTTOM_COLS,
    BOTTOM_ROWS,
    DEFAULT_JUICE_LEFTOVER,
    DEFAULT_JUICE_PRICES,
    DEFAULT_MATS_LEFTOVER,
    DEFAULT_MATS_PRICES,
    GRAPH_COLORS,
    IconMap,
    JUICE_LABELS,
    MATS_LABELS,
    PIECE_NAMES,
    TOP_COLS,
    TOP_ROWS,
} from "@/Utils/Constants.ts"
import { formatSig, piece_display_name } from "@/Utils/Helpers.ts"
import { buildPayload, type InputsValues } from "@/core/payload.ts"
import { runWasmOperation } from "@/core/wasmWorkerClient.ts"

const STORAGE_KEY = "HF_VUE_UI_STATE_V2"

type BoolGrid = boolean[][]
type NumGrid = number[][]
type UpgradeLike = {
    piece_type: number
    upgrade_index: number
    is_normal_honing?: boolean
    is_weapon?: boolean
    prob_dist?: number[]
    state?: [boolean, number][]
    succeeded?: boolean
    unlocked?: boolean
    current_ind?: number
}

function createBoolGrid(rows: number, cols: number, value = false): BoolGrid {
    return Array.from({ length: rows }, () => Array.from({ length: cols }, () => value))
}

function createNumGrid(rows: number, cols: number, value = 0): NumGrid {
    return Array.from({ length: rows }, () => Array.from({ length: cols }, () => value))
}

function createStateGrid() {
    return Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => [] as [boolean, number][]))
}

function cloneGrid<T>(grid: T[][]): T[][] {
    return grid.map((row) => row.slice())
}

function cloneStateGrid(grid: [boolean, number][][][]) {
    return grid.map((row) => row.map((cell) => cell.map((pair) => [Boolean(pair[0]), Number(pair[1])] as [boolean, number])))
}

function arraysEqual(a: number[] | boolean[], b: number[] | boolean[]) {
    if (a.length !== b.length) return false
    for (let index = 0; index < a.length; index++) {
        if (a[index] !== b[index]) return false
    }
    return true
}

function jsonEqual(a: unknown, b: unknown) {
    return JSON.stringify(a) === JSON.stringify(b)
}

function assignRecord(target: Record<string, string>, source: Record<string, string>) {
    for (const key of Object.keys(target)) {
        if (Object.prototype.hasOwnProperty.call(source, key)) {
            target[key] = String(source[key])
        }
    }
}

function clampInt(value: unknown, min: number, max: number) {
    const numeric = Number(value)
    if (!Number.isFinite(numeric)) return min
    return Math.max(min, Math.min(max, Math.floor(numeric)))
}

function cloneStringRecord(record: Record<string, string>) {
    const out: Record<string, string> = {}
    for (const [key, value] of Object.entries(record)) {
        out[key] = String(value)
    }
    return out
}

function cloneBoolGridForPayload(grid: BoolGrid): BoolGrid {
    return grid.map((row) => row.map((value) => Boolean(value)))
}

function cloneNumGridForPayload(grid: NumGrid): NumGrid {
    return grid.map((row) => row.map((value) => (Number.isFinite(Number(value)) ? Number(value) : 0)))
}

function cloneStateGridForPayload(grid: [boolean, number][][][]) {
    return grid.map((row) =>
        row.map((cell) =>
            cell.map((pair) => {
                const enabled = Boolean(pair?.[0])
                const tier = Number(pair?.[1])
                return [enabled, Number.isFinite(tier) ? tier : 0] as [boolean, number]
            }),
        ),
    )
}

function cloneNumberListForPayload(values: number[]) {
    return values.map((value) => {
        const numeric = Number(value)
        return Number.isFinite(numeric) ? numeric : 0
    })
}

function formatWhole(value: number | null | undefined) {
    if (value === null || value === undefined || !Number.isFinite(value)) return "N/A"
    return Math.round(value).toLocaleString("en-US")
}

function metricToText(metric: number | null | undefined) {
    if (metric === null || metric === undefined || !Number.isFinite(metric)) return "N/A"
    return `${Math.round(-metric).toLocaleString("en-US")}g`
}

function breakdownText(value: number | undefined) {
    if (value === undefined || !Number.isFinite(value)) {
        return "N/A"
    }
    if (value <= 0) {
        return `Avg Eqv Cost ${Math.round(-value).toLocaleString("en-US")}g`
    }
    return `Avg Eqv Surplus ${Math.round(value).toLocaleString("en-US")}g`
}

function breakdownClass(value: number | undefined) {
    if (value === undefined || !Number.isFinite(value)) return "muted"
    return value > -0.5 ? "surplus" : "cost"
}

function sortUpgradeIndices(list: number[], upgradeArr: UpgradeLike[], specialInvalidIndex: number) {
    const output: number[] = []
    const copy = upgradeArr.slice()

    for (const [originalIndex, upgradeIndex] of list.entries()) {
        if (!upgradeArr[upgradeIndex]) continue
        if (originalIndex >= specialInvalidIndex) {
            if (!output.includes(upgradeIndex)) {
                output.push(upgradeIndex)
            }
            continue
        }

        const currentUpgrade = upgradeArr[upgradeIndex]
        for (const [index, candidate] of copy.entries()) {
            if (!candidate) continue
            const candidateSucceeded = Boolean(candidate.succeeded)
            const samePiece = candidate.piece_type === currentUpgrade.piece_type
            const sameType = Boolean(candidate.is_normal_honing) === Boolean(currentUpgrade.is_normal_honing)
            const ordered = candidate.upgrade_index <= currentUpgrade.upgrade_index
            if (sameType && samePiece && ordered && !candidateSucceeded && !output.includes(index)) {
                output.push(index)
            }
        }
    }

    return output
}

function normalizeStatePairs(state: unknown): [boolean, number][] {
    if (!Array.isArray(state)) return []
    return state
        .map((pair) => {
            if (!Array.isArray(pair)) return null
            const enabled = Boolean(pair[0])
            const tier = Number(pair[1])
            if (!Number.isFinite(tier)) return null
            return [enabled, tier] as [boolean, number]
        })
        .filter((value): value is [boolean, number] => value !== null)
}

const topGrid = ref<BoolGrid>(createBoolGrid(TOP_ROWS, TOP_COLS))
const bottomGrid = ref<BoolGrid>(createBoolGrid(BOTTOM_ROWS, BOTTOM_COLS))

const matsOwned = reactive<Record<string, string>>(Object.fromEntries(MATS_LABELS.map((label) => [label, "0"])))
const matsPrices = reactive<Record<string, string>>(Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, idx) => [label, DEFAULT_MATS_PRICES[idx]])))
const matsLeftover = reactive<Record<string, string>>(Object.fromEntries(MATS_LABELS.map((label, idx) => [label, DEFAULT_MATS_LEFTOVER[idx] ?? "0"])))

const weaponOwned = reactive<Record<string, string>>(Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], "0"])))
const armorOwned = reactive<Record<string, string>>(Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], "0"])))
const weaponPrices = reactive<Record<string, string>>(Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[0], String(DEFAULT_JUICE_PRICES[idx][0])])))
const armorPrices = reactive<Record<string, string>>(Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[1], String(DEFAULT_JUICE_PRICES[idx][1])])))
const weaponLeftover = reactive<Record<string, string>>(Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[0], String(DEFAULT_JUICE_LEFTOVER[idx][0])])))
const armorLeftover = reactive<Record<string, string>>(Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[1], String(DEFAULT_JUICE_LEFTOVER[idx][1])])))

const expressEvent = ref(true)
const cumulativeGraph = ref(true)
const allowManualState = ref(false)
const customLeftovers = ref(false)
const autoStartOptimizer = ref(false)
const analysisTab = ref<"distribution" | "breakdown">("distribution")
const draggingFreeTapIndex = ref<number | null>(null)
const dragOverFreeTapIndex = ref<number | null>(null)
const topDragActive = ref(false)
const topDragValue = ref(false)
const bottomDragActive = ref(false)
const bottomDragValue = ref(false)
const normalGridScrollRef = ref<HTMLDivElement | null>(null)

const optimizeBusy = ref(false)
const optimizeError = ref<string | null>(null)
const optimizerProgress = ref(0)
const evaluateResult = ref<any>(null)
const histogramResult = ref<any>(null)

const hasRunOptimizer = ref(false)
const bestMetric = ref<number | null>(null)

const bucketCount = ref("100")
const dataSize = ref("100000")
const minResolution = ref(10)
const metricType = ref(1)

const progressGrid = ref<NumGrid>(createNumGrid(TOP_ROWS, TOP_COLS))
const unlockGrid = ref<BoolGrid>(createBoolGrid(TOP_ROWS, TOP_COLS))
const succeededGrid = ref<BoolGrid>(createBoolGrid(TOP_ROWS, TOP_COLS))
const stateBundleGrid = ref<[boolean, number][][][]>(createStateGrid())
const specialState = ref<number[]>([])

const optimizeWorker = ref<Worker | null>(null)
const evaluateWorker = ref<Worker | null>(null)
const histogramWorker = ref<Worker | null>(null)
let optimizeTimer: number | null = null
let evaluateTimer: number | null = null
let histogramTimer: number | null = null
let saveTimer: number | null = null
let suppressAutoRun = false

const advHoneStrategy = computed(() => (expressEvent.value ? "x2 grace" : "No x2 grace"))
const currentMetric = computed<number | null>(() => (typeof evaluateResult.value?.metric === "number" ? evaluateResult.value.metric : null))
const curIsBest = computed(() => {
    if (!hasRunOptimizer.value || bestMetric.value === null || currentMetric.value === null) return false
    return Math.round(currentMetric.value) >= Math.round(bestMetric.value)
})

const avgEqvGoldCost = computed(() => metricToText(currentMetric.value))

const resultStatus = computed(() => {
    if (optimizeBusy.value) return "Optimizer in progress"
    if (!hasRunOptimizer.value) return "Optimizer not run yet"
    if (curIsBest.value) return "Result below is the best known result"
    return "Current configuration is not best known result"
})

const resultStatusClass = computed(() => {
    if (optimizeBusy.value) return "muted"
    if (!hasRunOptimizer.value) return "warn"
    return curIsBest.value ? "best" : "muted"
})

const distributionLabels = computed(() => MATS_LABELS.slice(0, 7).concat(JUICE_LABELS.map((pair) => pair[0]), JUICE_LABELS.map((pair) => pair[1])))
const upgradeArr = computed<UpgradeLike[]>(() => (Array.isArray(evaluateResult.value?.upgrade_arr) ? evaluateResult.value.upgrade_arr : []))
const specialInvalidIndex = computed(() => clampInt(evaluateResult.value?.special_invalid_index ?? 0, 0, 9999))
const latestSpecialProbs = computed<number[]>(() =>
    Array.isArray(evaluateResult.value?.latest_special_probs) ? evaluateResult.value.latest_special_probs.map((value: unknown) => Number(value) || 0) : [],
)

const orderedUpgradeIndices = computed(() => {
    const upgrades = upgradeArr.value
    if (!upgrades.length) return []
    const baseList = specialState.value.length ? specialState.value.filter((value) => upgrades[value]) : upgrades.map((_, index) => index)
    const sorted = sortUpgradeIndices(baseList, upgrades, specialInvalidIndex.value)
    if (!sorted.length) return baseList

    const seen = new Set(sorted)
    for (const index of baseList) {
        if (!seen.has(index)) sorted.push(index)
    }
    return sorted
})

const freeTapRows = computed(() =>
    specialState.value
        .map((upgradeIndex, specialOrder) => ({
            upgradeIndex,
            specialOrder,
            upgrade: upgradeArr.value[upgradeIndex],
            probability: Number(latestSpecialProbs.value[specialOrder] ?? 0),
        }))
        .filter((row) => row.upgrade && row.upgrade.is_normal_honing),
)

const juiceAvail = computed(() => clampInt(evaluateResult.value?.prep_output?.juice_info?.num_avail ?? JUICE_LABELS.length, 0, JUICE_LABELS.length))

const inputsValues = computed<InputsValues>(() => ({
    mats: {
        owned: matsOwned,
        prices: matsPrices,
        leftover: matsLeftover,
    },
    juice: {
        weapon: {
            owned: weaponOwned,
            prices: weaponPrices,
            leftover: weaponLeftover,
        },
        armor: {
            owned: armorOwned,
            prices: armorPrices,
            leftover: armorLeftover,
        },
    },
}))

const autoRunKey = computed(() =>
    JSON.stringify({
        topGrid: topGrid.value,
        bottomGrid: bottomGrid.value,
        matsOwned,
        matsPrices,
        matsLeftover,
        weaponOwned,
        weaponPrices,
        weaponLeftover,
        armorOwned,
        armorPrices,
        armorLeftover,
        expressEvent: expressEvent.value,
        metricType: metricType.value,
        progressGrid: progressGrid.value,
        unlockGrid: unlockGrid.value,
        succeededGrid: succeededGrid.value,
        stateBundleGrid: stateBundleGrid.value,
        specialState: specialState.value,
    }),
)

function iconPath(name: string) {
    return IconMap[name] ?? ""
}

function pieceName(upgrade: UpgradeLike | undefined) {
    if (!upgrade) return "Unknown"
    return piece_display_name(upgrade)
}

function makePayload() {
    const payloadInputs: InputsValues = {
        mats: {
            owned: cloneStringRecord(matsOwned),
            prices: cloneStringRecord(matsPrices),
            leftover: cloneStringRecord(matsLeftover),
        },
        juice: {
            weapon: {
                owned: cloneStringRecord(weaponOwned),
                prices: cloneStringRecord(weaponPrices),
                leftover: cloneStringRecord(weaponLeftover),
            },
            armor: {
                owned: cloneStringRecord(armorOwned),
                prices: cloneStringRecord(armorPrices),
                leftover: cloneStringRecord(armorLeftover),
            },
        },
    }

    return buildPayload({
        topGrid: cloneBoolGridForPayload(topGrid.value),
        bottomGrid: cloneBoolGridForPayload(bottomGrid.value),
        advHoneStrategy: advHoneStrategy.value,
        expressEvent: expressEvent.value,
        bucketCount: bucketCount.value,
        dataSize: dataSize.value,
        inputs: payloadInputs,
        progressGrid: cloneNumGridForPayload(progressGrid.value),
        unlockGrid: cloneBoolGridForPayload(unlockGrid.value),
        succeededGrid: cloneBoolGridForPayload(succeededGrid.value),
        stateBundleGrid: cloneStateGridForPayload(stateBundleGrid.value),
        specialState: cloneNumberListForPayload(specialState.value),
        minResolution: minResolution.value,
        metricType: metricType.value,
    })
}

function terminateWorker(workerRef: { value: Worker | null }) {
    if (!workerRef.value) return
    try {
        workerRef.value.terminate()
    } catch {
        // ignore
    }
    workerRef.value = null
}

function updateBestMetric(res: any) {
    if (typeof res?.metric !== "number") return
    bestMetric.value = bestMetric.value === null ? res.metric : Math.max(bestMetric.value, res.metric)
}

function replaceStateFromResult(res: any) {
    if (!Array.isArray(res?.upgrade_arr)) return

    const nextProgress = cloneGrid(progressGrid.value)
    const nextUnlock = cloneGrid(unlockGrid.value)
    const nextSucceeded = cloneGrid(succeededGrid.value)
    const nextStateGrid = createStateGrid()

    for (const upgrade of res.upgrade_arr as UpgradeLike[]) {
        const row = clampInt(upgrade.piece_type, 0, TOP_ROWS - 1)
        const col = clampInt(upgrade.upgrade_index, 0, TOP_COLS - 1)
        if (typeof upgrade.current_ind === "number" && Number.isFinite(upgrade.current_ind)) {
            nextProgress[row][col] = Math.max(0, Math.floor(upgrade.current_ind))
        }
        if (typeof upgrade.unlocked === "boolean") {
            nextUnlock[row][col] = upgrade.unlocked
        }
        if (typeof upgrade.succeeded === "boolean") {
            nextSucceeded[row][col] = upgrade.succeeded
        }
        nextStateGrid[row][col] = normalizeStatePairs(upgrade.state)
    }

    if (!jsonEqual(progressGrid.value, nextProgress)) {
        progressGrid.value = nextProgress
    }
    if (!jsonEqual(unlockGrid.value, nextUnlock)) {
        unlockGrid.value = nextUnlock
    }
    if (!jsonEqual(succeededGrid.value, nextSucceeded)) {
        succeededGrid.value = nextSucceeded
    }
    if (!jsonEqual(stateBundleGrid.value, nextStateGrid)) {
        stateBundleGrid.value = nextStateGrid
    }
}

function withAutoRunSuppressed(task: () => void) {
    suppressAutoRun = true
    try {
        task()
    } finally {
        window.setTimeout(() => {
            suppressAutoRun = false
        }, 0)
    }
}

function applyResult(res: any) {
    withAutoRunSuppressed(() => {
        evaluateResult.value = res
        if (Array.isArray(res?.special_state) && !arraysEqual(specialState.value, res.special_state)) {
            specialState.value = [...res.special_state]
        }
        replaceStateFromResult(res)
    })
}

function runEvaluate() {
    if (optimizeBusy.value) return
    terminateWorker(evaluateWorker)
    const { worker, promise } = runWasmOperation(makePayload(), "EvaluateAverage")
    evaluateWorker.value = worker
    promise
        .then((res) => {
            applyResult(res)
            updateBestMetric(res)
        })
        .catch(() => {
            // keep previous result on errors
        })
        .finally(() => {
            if (evaluateWorker.value === worker) {
                evaluateWorker.value = null
            }
        })
}

function runHistogram() {
    if (analysisTab.value !== "distribution") return
    terminateWorker(histogramWorker)
    const { worker, promise } = runWasmOperation(makePayload(), "Histogram")
    histogramWorker.value = worker
    promise
        .then((res) => {
            histogramResult.value = res
        })
        .catch(() => {
            // keep previous histogram on errors
        })
        .finally(() => {
            if (histogramWorker.value === worker) {
                histogramWorker.value = null
            }
        })
}

function runOptimize() {
    terminateWorker(optimizeWorker)
    optimizeBusy.value = true
    optimizerProgress.value = 0
    optimizeError.value = null

    const { worker, promise } = runWasmOperation(makePayload(), "OptimizeAverage", (message: any) => {
        if (typeof message?.est_progress_percentage === "number") {
            optimizerProgress.value = message.est_progress_percentage
        }
        if (message?.state_bundle) {
            applyResult(message.state_bundle)
            updateBestMetric(message.state_bundle)
        }
    })

    optimizeWorker.value = worker
    promise
        .then((res) => {
            applyResult(res)
            updateBestMetric(res)
            hasRunOptimizer.value = true
            optimizeError.value = null
        })
        .catch((error) => {
            optimizeError.value = String(error)
        })
        .finally(() => {
            if (optimizeWorker.value === worker) {
                optimizeWorker.value = null
            }
            optimizeBusy.value = false
        })
}

function scheduleEvaluate(delayMs: number) {
    if (evaluateTimer !== null) {
        window.clearTimeout(evaluateTimer)
    }
    evaluateTimer = window.setTimeout(() => {
        evaluateTimer = null
        runEvaluate()
    }, delayMs)
}

function scheduleHistogram(delayMs: number) {
    if (histogramTimer !== null) {
        window.clearTimeout(histogramTimer)
    }
    histogramTimer = window.setTimeout(() => {
        histogramTimer = null
        runHistogram()
    }, delayMs)
}

function scheduleOptimize(delayMs: number) {
    if (optimizeTimer !== null) {
        window.clearTimeout(optimizeTimer)
    }
    optimizeTimer = window.setTimeout(() => {
        optimizeTimer = null
        runOptimize()
    }, delayMs)
}

function cancelOptimize() {
    terminateWorker(optimizeWorker)
    if (optimizeTimer !== null) {
        window.clearTimeout(optimizeTimer)
        optimizeTimer = null
    }
    optimizeBusy.value = false
    optimizerProgress.value = 0
}

function onOptimizeClick() {
    if (optimizeBusy.value) {
        cancelOptimize()
        autoStartOptimizer.value = false
        return
    }
    scheduleOptimize(0)
}

function toInputValue(event: Event) {
    return (event.target as HTMLInputElement).value
}

function setRecordValue(record: Record<string, string>, key: string, event: Event) {
    record[key] = toInputValue(event)
}

function setTopCellValue(row: number, col: number, value: boolean) {
    if (Boolean(topGrid.value[row]?.[col]) === value) return
    const next = cloneGrid(topGrid.value)
    next[row][col] = value
    topGrid.value = next
}

function setTopCell(row: number, col: number) {
    setTopCellValue(row, col, !Boolean(topGrid.value[row]?.[col]))
}

function onTopCellClick(row: number, col: number, event: MouseEvent) {
    if (event.detail !== 0) return
    setTopCell(row, col)
}

function startTopDrag(row: number, col: number, event: PointerEvent) {
    if (event.button !== 0) return
    const nextValue = !Boolean(topGrid.value[row]?.[col])
    setTopCellValue(row, col, nextValue)
    topDragValue.value = nextValue
    topDragActive.value = true
}

function dragTopCell(row: number, col: number) {
    if (!topDragActive.value) return
    setTopCellValue(row, col, topDragValue.value)
}

function stopGridDrag() {
    topDragActive.value = false
    bottomDragActive.value = false
}

function setBottomCellValue(row: number, col: number, value: boolean) {
    if (Boolean(bottomGrid.value[row]?.[col]) === value) return
    const next = cloneGrid(bottomGrid.value)
    next[row][col] = value
    bottomGrid.value = next
}

function setBottomCell(row: number, col: number) {
    setBottomCellValue(row, col, !Boolean(bottomGrid.value[row]?.[col]))
}

function onBottomCellClick(row: number, col: number, event: MouseEvent) {
    if (event.detail !== 0) return
    setBottomCell(row, col)
}

function startBottomDrag(row: number, col: number, event: PointerEvent) {
    if (event.button !== 0) return
    const nextValue = !Boolean(bottomGrid.value[row]?.[col])
    setBottomCellValue(row, col, nextValue)
    bottomDragValue.value = nextValue
    bottomDragActive.value = true
}

function dragBottomCell(row: number, col: number) {
    if (!bottomDragActive.value) return
    setBottomCellValue(row, col, bottomDragValue.value)
}

function isTopColChecked(col: number) {
    return topGrid.value.every((row) => row[col])
}

function isBottomColChecked(col: number) {
    return bottomGrid.value.every((row) => row[col])
}

function toggleTopCol(col: number) {
    const next = cloneGrid(topGrid.value)
    const newState = !isTopColChecked(col)
    for (let row = 0; row < TOP_ROWS; row++) {
        next[row][col] = newState
    }
    topGrid.value = next
    stopGridDrag()
}

function toggleBottomCol(col: number) {
    const next = cloneGrid(bottomGrid.value)
    const newState = !isBottomColChecked(col)
    for (let row = 0; row < BOTTOM_ROWS; row++) {
        next[row][col] = newState
    }
    bottomGrid.value = next
    stopGridDrag()
}

function copyPayload() {
    const payload = JSON.stringify(makePayload(), null, 2)
    navigator.clipboard?.writeText(payload).catch(() => undefined)
}

function resetOptimizerState() {
    cancelOptimize()
    terminateWorker(evaluateWorker)
    terminateWorker(histogramWorker)

    optimizeError.value = null
    optimizerProgress.value = 0
    evaluateResult.value = null
    histogramResult.value = null
    hasRunOptimizer.value = false
    bestMetric.value = null

    progressGrid.value = createNumGrid(TOP_ROWS, TOP_COLS)
    unlockGrid.value = createBoolGrid(TOP_ROWS, TOP_COLS)
    succeededGrid.value = createBoolGrid(TOP_ROWS, TOP_COLS)
    stateBundleGrid.value = createStateGrid()
    specialState.value = []
}

function resetAll() {
    withAutoRunSuppressed(() => {
        topGrid.value = createBoolGrid(TOP_ROWS, TOP_COLS)
        bottomGrid.value = createBoolGrid(BOTTOM_ROWS, BOTTOM_COLS)
        stopGridDrag()

        assignRecord(matsOwned, Object.fromEntries(MATS_LABELS.map((label) => [label, "0"])))
        assignRecord(matsPrices, Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, idx) => [label, DEFAULT_MATS_PRICES[idx]])))
        assignRecord(matsLeftover, Object.fromEntries(MATS_LABELS.map((label, idx) => [label, DEFAULT_MATS_LEFTOVER[idx] ?? "0"])))

        assignRecord(weaponOwned, Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], "0"])))
        assignRecord(armorOwned, Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], "0"])))
        assignRecord(weaponPrices, Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[0], String(DEFAULT_JUICE_PRICES[idx][0])])))
        assignRecord(armorPrices, Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[1], String(DEFAULT_JUICE_PRICES[idx][1])])))
        assignRecord(weaponLeftover, Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[0], String(DEFAULT_JUICE_LEFTOVER[idx][0])])))
        assignRecord(armorLeftover, Object.fromEntries(JUICE_LABELS.map((labels, idx) => [labels[1], String(DEFAULT_JUICE_LEFTOVER[idx][1])])))

        expressEvent.value = true
        cumulativeGraph.value = true
        allowManualState.value = false
        customLeftovers.value = false
        autoStartOptimizer.value = false
        analysisTab.value = "distribution"

        resetOptimizerState()
    })

    scheduleEvaluate(0)
    scheduleHistogram(80)
}

function saveState() {
    const snapshot = {
        topGrid: topGrid.value,
        bottomGrid: bottomGrid.value,
        matsOwned: { ...matsOwned },
        matsPrices: { ...matsPrices },
        matsLeftover: { ...matsLeftover },
        weaponOwned: { ...weaponOwned },
        weaponPrices: { ...weaponPrices },
        weaponLeftover: { ...weaponLeftover },
        armorOwned: { ...armorOwned },
        armorPrices: { ...armorPrices },
        armorLeftover: { ...armorLeftover },
        expressEvent: expressEvent.value,
        cumulativeGraph: cumulativeGraph.value,
        allowManualState: allowManualState.value,
        customLeftovers: customLeftovers.value,
        autoStartOptimizer: autoStartOptimizer.value,
        analysisTab: analysisTab.value,
        progressGrid: progressGrid.value,
        unlockGrid: unlockGrid.value,
        succeededGrid: succeededGrid.value,
        stateBundleGrid: stateBundleGrid.value,
        specialState: specialState.value,
    }

    localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot))
}

function loadState() {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return

    try {
        const parsed = JSON.parse(raw)

        withAutoRunSuppressed(() => {
            if (Array.isArray(parsed.topGrid) && parsed.topGrid.length === TOP_ROWS) topGrid.value = parsed.topGrid
            if (Array.isArray(parsed.bottomGrid) && parsed.bottomGrid.length === BOTTOM_ROWS) bottomGrid.value = parsed.bottomGrid
            stopGridDrag()

            if (parsed.matsOwned) assignRecord(matsOwned, parsed.matsOwned)
            if (parsed.matsPrices) assignRecord(matsPrices, parsed.matsPrices)
            if (parsed.matsLeftover) assignRecord(matsLeftover, parsed.matsLeftover)
            if (parsed.weaponOwned) assignRecord(weaponOwned, parsed.weaponOwned)
            if (parsed.weaponPrices) assignRecord(weaponPrices, parsed.weaponPrices)
            if (parsed.weaponLeftover) assignRecord(weaponLeftover, parsed.weaponLeftover)
            if (parsed.armorOwned) assignRecord(armorOwned, parsed.armorOwned)
            if (parsed.armorPrices) assignRecord(armorPrices, parsed.armorPrices)
            if (parsed.armorLeftover) assignRecord(armorLeftover, parsed.armorLeftover)

            expressEvent.value = Boolean(parsed.expressEvent)
            cumulativeGraph.value = Boolean(parsed.cumulativeGraph)
            allowManualState.value = Boolean(parsed.allowManualState)
            customLeftovers.value = Boolean(parsed.customLeftovers)
            autoStartOptimizer.value = Boolean(parsed.autoStartOptimizer)
            const persistedTab = parsed.analysisTab ?? parsed.activeTab
            analysisTab.value = persistedTab === "breakdown" || persistedTab === "instructions" ? "breakdown" : "distribution"

            if (Array.isArray(parsed.progressGrid) && parsed.progressGrid.length === TOP_ROWS) {
                progressGrid.value = parsed.progressGrid
            }
            if (Array.isArray(parsed.unlockGrid) && parsed.unlockGrid.length === TOP_ROWS) {
                unlockGrid.value = parsed.unlockGrid
            }
            if (Array.isArray(parsed.succeededGrid) && parsed.succeededGrid.length === TOP_ROWS) {
                succeededGrid.value = parsed.succeededGrid
            }
            if (Array.isArray(parsed.stateBundleGrid) && parsed.stateBundleGrid.length === TOP_ROWS) {
                stateBundleGrid.value = parsed.stateBundleGrid.map((row: unknown[]) =>
                    Array.isArray(row) ? row.map((cell) => normalizeStatePairs(cell)) : Array.from({ length: TOP_COLS }, () => []),
                )
            }
            if (Array.isArray(parsed.specialState)) {
                specialState.value = parsed.specialState.map((value: unknown) => Number(value)).filter((value: number) => Number.isFinite(value))
            }
        })
    } catch {
        // ignore invalid local state
    }
}

function scrollNormalGridToRight() {
    const grid = normalGridScrollRef.value
    if (!grid) return
    grid.scrollLeft = grid.scrollWidth
}

function getUpgradeCell(upgradeIndex: number) {
    const upgrade = upgradeArr.value[upgradeIndex]
    if (!upgrade) return null
    const row = clampInt(upgrade.piece_type, 0, TOP_ROWS - 1)
    const col = clampInt(upgrade.upgrade_index, 0, TOP_COLS - 1)
    return { row, col, upgrade }
}

function getUpgradeProgress(upgradeIndex: number) {
    const cell = getUpgradeCell(upgradeIndex)
    if (!cell) return 0
    return Number(progressGrid.value[cell.row]?.[cell.col] ?? 0)
}

function getUpgradeSucceeded(upgradeIndex: number) {
    const cell = getUpgradeCell(upgradeIndex)
    if (!cell) return false
    return Boolean(succeededGrid.value[cell.row]?.[cell.col])
}

function getPityLength(upgrade: UpgradeLike | undefined) {
    if (!upgrade || !Array.isArray(upgrade.prob_dist)) return 1
    return Math.max(1, upgrade.prob_dist.length - 1)
}

function setUpgradeProgress(upgradeIndex: number, rawValue: number) {
    const cell = getUpgradeCell(upgradeIndex)
    if (!cell) return

    const pityLength = getPityLength(cell.upgrade)
    const nextProgressValue = Math.max(0, Math.min(pityLength, Math.floor(rawValue)))

    const nextProgress = cloneGrid(progressGrid.value)
    const nextUnlock = cloneGrid(unlockGrid.value)
    const nextSucceeded = cloneGrid(succeededGrid.value)

    nextProgress[cell.row][cell.col] = nextProgressValue
    if (nextProgressValue > 0) {
        nextUnlock[cell.row][cell.col] = true
    }
    nextSucceeded[cell.row][cell.col] = nextProgressValue >= pityLength

    progressGrid.value = nextProgress
    unlockGrid.value = nextUnlock
    succeededGrid.value = nextSucceeded
}

function toggleUpgradeSucceeded(upgradeIndex: number) {
    const cell = getUpgradeCell(upgradeIndex)
    if (!cell) return

    const pityLength = getPityLength(cell.upgrade)
    const nextUnlock = cloneGrid(unlockGrid.value)
    const nextProgress = cloneGrid(progressGrid.value)
    const nextSucceeded = cloneGrid(succeededGrid.value)

    const currentlySucceeded = Boolean(nextSucceeded[cell.row][cell.col])
    if (currentlySucceeded) {
        nextSucceeded[cell.row][cell.col] = false
        nextProgress[cell.row][cell.col] = Math.min(Math.max(nextProgress[cell.row][cell.col], 0), pityLength - 1)
    } else {
        nextUnlock[cell.row][cell.col] = true
        nextSucceeded[cell.row][cell.col] = true
        nextProgress[cell.row][cell.col] = Math.max(1, Math.min(Math.max(nextProgress[cell.row][cell.col], 1), pityLength))
    }

    unlockGrid.value = nextUnlock
    progressGrid.value = nextProgress
    succeededGrid.value = nextSucceeded
}

function onUpgradeProgressInput(upgradeIndex: number, event: Event) {
    const raw = Number((event.target as HTMLInputElement).value)
    if (!Number.isFinite(raw)) {
        setUpgradeProgress(upgradeIndex, 0)
        return
    }
    setUpgradeProgress(upgradeIndex, raw)
}

function specialIndexOf(upgradeIndex: number) {
    return specialState.value.findIndex((value) => value === upgradeIndex)
}

function isFreeTapUpgrade(upgradeIndex: number) {
    const specialIndex = specialIndexOf(upgradeIndex)
    if (specialIndex < 0 || specialIndex >= specialInvalidIndex.value) return false
    const upgrade = upgradeArr.value[upgradeIndex]
    if (!upgrade || !upgrade.is_normal_honing) return false
    return Number(latestSpecialProbs.value[specialIndex] ?? 0) > 0
}

function freeTapChance(upgradeIndex: number) {
    const specialIndex = specialIndexOf(upgradeIndex)
    if (specialIndex < 0) return 0
    return Number(latestSpecialProbs.value[specialIndex] ?? 0)
}

function getUpgradeStatePairs(upgradeIndex: number) {
    const cell = getUpgradeCell(upgradeIndex)
    if (!cell) return [] as [boolean, number][]
    const current = stateBundleGrid.value[cell.row]?.[cell.col]
    if (!Array.isArray(current)) return [] as [boolean, number][]
    return current.map((pair) => {
        const enabled = Boolean(pair?.[0])
        const tier = Number(pair?.[1])
        return [enabled, Number.isFinite(tier) ? tier : 0] as [boolean, number]
    })
}

function getJuiceLabelForUpgrade(upgrade: UpgradeLike | undefined, tierIndex: number) {
    if (!upgrade) return ""
    const col = upgrade.is_weapon ? 0 : 1
    return JUICE_LABELS[tierIndex]?.[col] ?? ""
}

function getNextTapPlan(upgradeIndex: number) {
    const upgrade = upgradeArr.value[upgradeIndex]
    if (!upgrade) return null
    if (!upgrade.is_normal_honing) {
        return {
            status: "advanced" as const,
            tapIndex: 0,
            pityLength: 0,
            useJuice: false,
            bookTier: 0,
            juiceLabel: "",
            bookLabel: "",
        }
    }

    const pityLength = getPityLength(upgrade)
    const progress = Math.max(0, Math.min(pityLength, getUpgradeProgress(upgradeIndex)))
    if (getUpgradeSucceeded(upgradeIndex)) {
        return {
            status: "completed" as const,
            tapIndex: pityLength,
            pityLength,
            useJuice: false,
            bookTier: 0,
            juiceLabel: "",
            bookLabel: "",
        }
    }

    const tapIndex = Math.min(progress, pityLength - 1)
    const pair = getUpgradeStatePairs(upgradeIndex)[tapIndex] ?? ([false, 0] as [boolean, number])
    const useJuice = Boolean(pair[0])
    const bookTier = clampInt(pair[1], 0, JUICE_LABELS.length - 1)
    const juiceLabel = useJuice ? getJuiceLabelForUpgrade(upgrade, 0) : ""
    const bookLabel = bookTier > 0 ? getJuiceLabelForUpgrade(upgrade, bookTier) : ""

    return {
        status: "active" as const,
        tapIndex: tapIndex + 1,
        pityLength,
        useJuice,
        bookTier,
        juiceLabel,
        bookLabel,
    }
}

function getNextTapInstruction(upgradeIndex: number) {
    const plan = getNextTapPlan(upgradeIndex)
    if (!plan) return "No plan available."
    if (plan.status === "advanced") return "Advanced honing row: use Grace setup."
    if (plan.status === "completed") {
        return `Completed (${plan.pityLength}/${plan.pityLength}).`
    }

    const actions: string[] = []
    if (plan.useJuice && plan.juiceLabel) {
        actions.push(plan.juiceLabel)
    }
    if (plan.bookTier > 0 && plan.bookLabel) {
        actions.push(plan.bookLabel)
    }

    const actionText = actions.length ? `Use ${actions.join(" + ")}` : "No breath or book"
    return `Tap ${plan.tapIndex}/${plan.pityLength}: ${actionText}`
}

function reorderFreeTapRows(fromDisplayIndex: number, toDisplayIndex: number) {
    const normalPositions = specialState.value
        .map((upgradeIndex, position) => ({ upgradeIndex, position }))
        .filter((item) => Boolean(upgradeArr.value[item.upgradeIndex]?.is_normal_honing))

    const lastIndex = normalPositions.length - 1
    if (lastIndex <= 0) return

    const from = clampInt(fromDisplayIndex, 0, lastIndex)
    const to = clampInt(toDisplayIndex, 0, lastIndex)
    if (from === to) return

    const orderedNormalUpgrades = normalPositions.map((item) => item.upgradeIndex)
    const [movedUpgradeIndex] = orderedNormalUpgrades.splice(from, 1)
    orderedNormalUpgrades.splice(to, 0, movedUpgradeIndex)

    const next = [...specialState.value]
    for (let index = 0; index < normalPositions.length; index++) {
        const originalSpecialPosition = normalPositions[index].position
        next[originalSpecialPosition] = orderedNormalUpgrades[index]
    }
    specialState.value = next
}

function onFreeTapDragStart(index: number, event: DragEvent) {
    draggingFreeTapIndex.value = index
    dragOverFreeTapIndex.value = index
    if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = "move"
        event.dataTransfer.setData("text/plain", String(index))
    }
}

function onFreeTapDragOver(index: number) {
    if (draggingFreeTapIndex.value === null) return
    dragOverFreeTapIndex.value = index
}

function onFreeTapDrop(index: number) {
    if (draggingFreeTapIndex.value === null) return
    reorderFreeTapRows(draggingFreeTapIndex.value, index)
    draggingFreeTapIndex.value = null
    dragOverFreeTapIndex.value = null
}

function onFreeTapDragEnd() {
    draggingFreeTapIndex.value = null
    dragOverFreeTapIndex.value = null
}

function goldBreakdownValue(offset: number) {
    const source = evaluateResult.value?.average_breakdown
    if (!Array.isArray(source)) return undefined
    const value = Number(source[offset])
    return Number.isFinite(value) ? value : undefined
}

function clearManualProgressState() {
    progressGrid.value = createNumGrid(TOP_ROWS, TOP_COLS)
    unlockGrid.value = createBoolGrid(TOP_ROWS, TOP_COLS)
    succeededGrid.value = createBoolGrid(TOP_ROWS, TOP_COLS)
}

watch(
    autoRunKey,
    () => {
        if (suppressAutoRun) return

        hasRunOptimizer.value = false
        bestMetric.value = null

        if (autoStartOptimizer.value) {
            scheduleOptimize(220)
        } else {
            scheduleEvaluate(120)
        }

        if (analysisTab.value === "distribution") {
            scheduleHistogram(180)
        }

        if (saveTimer !== null) {
            window.clearTimeout(saveTimer)
        }
        saveTimer = window.setTimeout(() => {
            saveTimer = null
            saveState()
        }, 280)
    },
    { deep: false },
)

watch(autoStartOptimizer, (enabled) => {
    if (enabled && !suppressAutoRun) {
        scheduleOptimize(60)
    }
})

watch(allowManualState, (enabled) => {
    if (enabled) return
    clearManualProgressState()
})

watch(analysisTab, (value) => {
    if (value === "distribution") {
        scheduleHistogram(20)
    }
})

onMounted(() => {
    loadState()
    scheduleEvaluate(0)
    if (analysisTab.value === "distribution") {
        scheduleHistogram(80)
    }
    window.addEventListener("pointerup", stopGridDrag)
    window.addEventListener("blur", stopGridDrag)
    nextTick(() => {
        scrollNormalGridToRight()
        window.requestAnimationFrame(() => {
            scrollNormalGridToRight()
        })
    })
})

onBeforeUnmount(() => {
    terminateWorker(optimizeWorker)
    terminateWorker(evaluateWorker)
    terminateWorker(histogramWorker)
    window.removeEventListener("pointerup", stopGridDrag)
    window.removeEventListener("blur", stopGridDrag)

    if (optimizeTimer !== null) window.clearTimeout(optimizeTimer)
    if (evaluateTimer !== null) window.clearTimeout(evaluateTimer)
    if (histogramTimer !== null) window.clearTimeout(histogramTimer)
    if (saveTimer !== null) window.clearTimeout(saveTimer)
})
</script>

<template>
    <div class="hf-shell">
        <div class="hf-app-wrapper">
            <header class="hf-header">
                <div class="hf-brand">
                    <div class="hf-brand-icon">
                        <img :src="iconPath('Forecast Icon')" alt="Forecast icon" style="width: 34px; height: 34px" />
                    </div>
                    <div>
                        <h1 class="hf-title">Honing Forecast</h1>
                        <div class="hf-subtitle">Lost Ark Upgrade Planner</div>
                    </div>
                </div>
                <div class="hf-header-links">
                    <a href="https://github.com/Kenivia/Honing-Forecast">GitHub</a>
                </div>
            </header>

            <div class="hf-main-content">
                <div class="hf-top-grid">
                    <section class="hf-main-stage">
                        <div class="hf-honing-row">
                            <section class="hf-card hf-normal-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Normal Honing</div>
                                    <span class="hf-card-hint">Build target grid</span>
                                </div>
                                <div class="hf-card-body">
                                    <div class="hf-grid-content">
                                        <div class="hf-label-col">
                                            <div class="hf-label-row" />
                                            <div v-for="piece in PIECE_NAMES" :key="piece" class="hf-label-row">
                                                <div class="hf-equip-label">
                                                    <span>{{ piece }}</span>
                                                    <img :src="iconPath(piece)" :alt="piece" />
                                                </div>
                                            </div>
                                        </div>
                                        <div ref="normalGridScrollRef" class="hf-grid-scroll">
                                            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${TOP_COLS}, 26px)` }">
                                                <button
                                                    v-for="col in TOP_COLS"
                                                    :key="`top-col-${col}`"
                                                    class="hf-cell hf-cell-header"
                                                    :class="{ selected: isTopColChecked(col - 1) }"
                                                    @click="toggleTopCol(col - 1)"
                                                >
                                                    +{{ col }}
                                                </button>
                                            </div>
                                            <div
                                                v-for="row in TOP_ROWS"
                                                :key="`top-row-${row}`"
                                                class="hf-cell-grid"
                                                :style="{ gridTemplateColumns: `repeat(${TOP_COLS}, 26px)` }"
                                            >
                                                <button
                                                    v-for="col in TOP_COLS"
                                                    :key="`top-${row}-${col}`"
                                                    class="hf-cell"
                                                    :class="{ selected: topGrid[row - 1][col - 1] }"
                                                    @pointerdown.prevent="startTopDrag(row - 1, col - 1, $event)"
                                                    @pointerenter="dragTopCell(row - 1, col - 1)"
                                                    @click.prevent="onTopCellClick(row - 1, col - 1, $event)"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>

                            <section class="hf-card hf-advanced-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Advanced Honing</div>
                                    <span class="hf-card-hint">Juice on Grace assumed</span>
                                </div>
                                <div class="hf-card-body">
                                    <div class="hf-grid-content hf-grid-content-compact">
                                        <div class="hf-label-col hf-label-col-compact">
                                            <div class="hf-label-row" />
                                            <div v-for="piece in PIECE_NAMES" :key="`adv-${piece}`" class="hf-label-row">
                                                <div class="hf-equip-label hf-equip-label-compact">
                                                    <img :src="iconPath(piece)" :alt="piece" />
                                                </div>
                                            </div>
                                        </div>
                                        <div>
                                            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${BOTTOM_COLS}, 26px)` }">
                                                <button
                                                    v-for="(bonus, idx) in [10, 20, 30, 40]"
                                                    :key="`bottom-col-${bonus}`"
                                                    class="hf-cell hf-cell-header"
                                                    :class="{ selected: isBottomColChecked(idx) }"
                                                    @click="toggleBottomCol(idx)"
                                                >
                                                    +{{ bonus }}
                                                </button>
                                            </div>
                                            <div
                                                v-for="row in BOTTOM_ROWS"
                                                :key="`bottom-row-${row}`"
                                                class="hf-cell-grid"
                                                :style="{ gridTemplateColumns: `repeat(${BOTTOM_COLS}, 26px)` }"
                                            >
                                                <button
                                                    v-for="col in BOTTOM_COLS"
                                                    :key="`bottom-${row}-${col}`"
                                                    class="hf-cell"
                                                    :class="{ selected: bottomGrid[row - 1][col - 1] }"
                                                    @pointerdown.prevent="startBottomDrag(row - 1, col - 1, $event)"
                                                    @pointerenter="dragBottomCell(row - 1, col - 1)"
                                                    @click.prevent="onBottomCellClick(row - 1, col - 1, $event)"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>
                        </div>

                        <div class="hf-ops-row">
                            <section class="hf-card hf-optimizer-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Action Queue</div>
                                    <span class="hf-card-hint">Optimize, then follow next steps</span>
                                </div>
                                <div class="hf-card-body">
                                    <div class="optimizer-card">
                                        <button
                                            class="hf-optimize-btn"
                                            :style="{
                                                background: optimizeBusy
                                                    ? 'var(--cancel-optimizer-button)'
                                                    : hasRunOptimizer
                                                      ? 'linear-gradient(180deg, #60656f 0%, #4f545f 100%)'
                                                      : 'linear-gradient(180deg, #e6c86f 0%, #cfaf52 100%)',
                                                color: optimizeBusy ? 'var(--text-muted)' : hasRunOptimizer ? 'var(--hf-text-bright)' : '#1b1f25',
                                            }"
                                            @click="onOptimizeClick"
                                        >
                                            {{ optimizeBusy ? "Cancel Optimize" : hasRunOptimizer ? "Re-run Optimizer" : ">>> Optimize <<<" }}
                                        </button>

                                        <label class="hf-inline-check">
                                            <input v-model="autoStartOptimizer" type="checkbox" />
                                            <span>Auto start optimizer</span>
                                        </label>

                                        <div class="hf-metric-card">
                                            <div class="hf-metric-label">Avg eqv gold cost</div>
                                            <div class="hf-metric-value" :class="curIsBest ? 'best' : 'not-best'">{{ avgEqvGoldCost }}</div>
                                            <div class="hf-metric-status" :class="resultStatusClass">{{ resultStatus }}</div>
                                            <div v-if="allowManualState" class="hf-result-note">Cost so far + average future cost</div>
                                        </div>

                                        <div v-if="optimizeError" class="optimizer-error">Error: {{ optimizeError }}</div>

                                        <div v-if="optimizeBusy" class="optimizer-progress">
                                            <span>Optimizer progress: {{ Math.max(optimizerProgress, 0.001).toFixed(2) }}%</span>
                                            <div class="progress-bar">
                                                <div class="progress-fill" :style="{ width: `${optimizerProgress}%` }" />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>

                            <section class="hf-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Controls</div>
                                </div>
                                <div class="hf-card-body hf-options-body">
                                    <div class="hf-options-row">
                                        <button class="hf-header-link-btn" @click="resetAll">Reset All</button>
                                        <button class="hf-header-link-btn" @click="resetOptimizerState">Reset Optimizer</button>
                                    </div>
                                    <button class="hf-header-link-btn" @click="copyPayload">Copy Payload</button>

                                    <div class="hf-divider" />
                                    <label class="hf-inline-check">
                                        <input v-model="expressEvent" type="checkbox" />
                                        <span>Express event</span>
                                    </label>
                                    <label class="hf-inline-check">
                                        <input v-model="cumulativeGraph" type="checkbox" />
                                        <span>Cumulative graph</span>
                                    </label>
                                    <label class="hf-inline-check">
                                        <input v-model="allowManualState" type="checkbox" />
                                        <span>Enable progress updates for better optimization</span>
                                    </label>
                                </div>
                            </section>
                        </div>

                        <div class="hf-compact-row">
                            <section class="hf-card hf-tap-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Tap Instructions</div>
                                    <span class="hf-card-hint">Top to bottom execution</span>
                                </div>
                                <div class="hf-card-body">
                                    <p class="hf-copy">Empty state means no book and no breath on that tap.</p>
                                    <p class="hf-copy" v-if="allowManualState">Progress updates enabled: update progress and success after each relevant outcome.</p>
                                    <p class="hf-copy" v-else>Enable progress updates in Controls for better optimization and live progression tracking.</p>

                                    <div v-if="orderedUpgradeIndices.length" class="hf-upgrade-editor">
                                        <div v-for="(upgradeIndex, orderIndex) in orderedUpgradeIndices" :key="`editor-${upgradeIndex}`" class="hf-upgrade-row">
                                            <div class="hf-upgrade-order">{{ orderIndex + 1 }}</div>
                                            <div class="hf-upgrade-meta">
                                                <div class="hf-upgrade-topline">
                                                    <span class="hf-upgrade-name">{{ pieceName(upgradeArr[upgradeIndex]) }}</span>
                                                    <span v-if="isFreeTapUpgrade(upgradeIndex)" class="hf-upgrade-free">
                                                        Free tap {{ formatSig(freeTapChance(upgradeIndex) * 100, 3) }}%
                                                    </span>
                                                </div>
                                                <span class="hf-upgrade-plan">{{ getNextTapInstruction(upgradeIndex) }}</span>

                                                <div class="hf-upgrade-actions-row" v-if="getNextTapPlan(upgradeIndex)?.status === 'active'">
                                                    <span class="hf-action-chip tap">
                                                        Tap {{ getNextTapPlan(upgradeIndex)?.tapIndex }}/{{ getNextTapPlan(upgradeIndex)?.pityLength }}
                                                    </span>
                                                    <span
                                                        v-if="getNextTapPlan(upgradeIndex)?.useJuice"
                                                        class="hf-action-chip juice"
                                                    >
                                                        <img
                                                            :src="iconPath(getNextTapPlan(upgradeIndex)?.juiceLabel ?? '')"
                                                            :alt="getNextTapPlan(upgradeIndex)?.juiceLabel ?? ''"
                                                        />
                                                        {{ getNextTapPlan(upgradeIndex)?.juiceLabel }}
                                                    </span>
                                                    <span
                                                        v-if="(getNextTapPlan(upgradeIndex)?.bookTier ?? 0) > 0"
                                                        class="hf-action-chip book"
                                                    >
                                                        <img
                                                            :src="iconPath(getNextTapPlan(upgradeIndex)?.bookLabel ?? '')"
                                                            :alt="getNextTapPlan(upgradeIndex)?.bookLabel ?? ''"
                                                        />
                                                        {{ getNextTapPlan(upgradeIndex)?.bookLabel }}
                                                    </span>
                                                    <span
                                                        v-if="!(getNextTapPlan(upgradeIndex)?.useJuice) && (getNextTapPlan(upgradeIndex)?.bookTier ?? 0) === 0"
                                                        class="hf-action-chip muted"
                                                    >
                                                        No juice / no book
                                                    </span>
                                                </div>
                                            </div>

                                            <div v-if="allowManualState" class="hf-upgrade-controls">
                                                <label>
                                                    Progress
                                                    <input
                                                        type="number"
                                                        min="0"
                                                        :max="getPityLength(upgradeArr[upgradeIndex])"
                                                        :value="getUpgradeProgress(upgradeIndex)"
                                                        @input="onUpgradeProgressInput(upgradeIndex, $event)"
                                                    />
                                                </label>
                                                <button class="hf-mini-btn" @click="toggleUpgradeSucceeded(upgradeIndex)">
                                                    {{ getUpgradeSucceeded(upgradeIndex) ? "Undo" : "Succeed" }}
                                                </button>
                                            </div>
                                            <div v-else class="hf-upgrade-controls hf-upgrade-controls-readonly">
                                                <span class="hf-upgrade-readonly-note">Progress updates disabled</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </section>

                            <section class="hf-card hf-freetap-card">
                                <div class="hf-card-header">
                                    <div class="hf-card-title"><span class="hf-card-title-dot" />Free Tap Priority</div>
                                    <span class="hf-card-hint">Reorder if needed</span>
                                </div>
                                <div class="hf-card-body">
                                    <p class="hf-copy">
                                        Keep attempting free taps until you run out, then move on to the next.
                                        The instructions for tapping takes into account the normal taps you may need to do before or in-between free taps.
                                    </p>
                                    <div v-if="freeTapRows.length" class="hf-freetap-table">
                                        <div class="hf-freetap-head">
                                            <span>#</span>
                                            <span>Upgrade</span>
                                            <span>Chance</span>
                                            <span>Actions</span>
                                        </div>
                                        <div
                                            v-for="(row, index) in freeTapRows"
                                            :key="`freetap-${row.upgradeIndex}`"
                                            class="hf-freetap-row"
                                            :class="{
                                                draggable: true,
                                                dragging: draggingFreeTapIndex === index,
                                                'drop-target': dragOverFreeTapIndex === index && draggingFreeTapIndex !== null && draggingFreeTapIndex !== index,
                                            }"
                                            draggable="true"
                                            @dragstart="onFreeTapDragStart(index, $event)"
                                            @dragover.prevent="onFreeTapDragOver(index)"
                                            @dragenter.prevent="onFreeTapDragOver(index)"
                                            @drop.prevent="onFreeTapDrop(index)"
                                            @dragend="onFreeTapDragEnd"
                                        >
                                            <span>{{ index + 1 }}</span>
                                            <span class="hf-freetap-upgrade">{{ pieceName(row.upgrade) }}</span>
                                            <span>{{ row.probability > 0 ? `${formatSig(row.probability * 100, 3)}%` : "-" }}</span>
                                            <div class="hf-freetap-actions">
                                                <span class="hf-drag-pill">Drag</span>
                                                <button class="hf-mini-btn" @click="toggleUpgradeSucceeded(row.upgradeIndex)">
                                                    {{ getUpgradeSucceeded(row.upgradeIndex) ? "Undo" : "Succeed" }}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                    <p v-else class="hf-copy">No available free taps in current state.</p>
                                </div>
                            </section>
                        </div>
                    </section>

                    <aside class="hf-side-lane">
                        <section class="hf-card">
                            <div class="hf-card-header">
                                <div class="hf-card-title"><span class="hf-card-title-dot" />Materials and Prices</div>
                                <span class="hf-card-hint">Owned + market</span>
                            </div>
                            <div class="hf-card-body">
                                <p class="hf-copy">Optimizer minimizes equivalent gold across spent and consumed tradable value.</p>

                                <div class="hf-material-stack">
                                    <div class="hf-table-wrap">
                                        <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                                            <span />
                                            <span>Owned</span>
                                            <span>Price</span>
                                            <span v-if="customLeftovers">Left</span>
                                        </div>
                                        <div v-for="label in MATS_LABELS" :key="`mats-${label}`" class="hf-table-row" :class="{ leftovers: customLeftovers }">
                                            <label class="hf-row-label">
                                                <span>{{ label }}</span>
                                                <img :src="iconPath(label)" :alt="label" />
                                            </label>
                                            <input type="text" :value="matsOwned[label]" @input="setRecordValue(matsOwned, label, $event)" />
                                            <input
                                                v-if="label !== 'Special Leap'"
                                                type="text"
                                                :value="matsPrices[label] ?? ''"
                                                @input="setRecordValue(matsPrices, label, $event)"
                                            />
                                            <div v-else class="hf-input-placeholder" />
                                            <input v-if="customLeftovers" type="text" :value="matsLeftover[label]" @input="setRecordValue(matsLeftover, label, $event)" />
                                        </div>
                                    </div>

                                    <div class="hf-table-wrap">
                                        <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                                            <span />
                                            <span>Owned</span>
                                            <span>Price</span>
                                            <span v-if="customLeftovers">Left</span>
                                        </div>
                                        <div v-for="labels in JUICE_LABELS" :key="`weapon-${labels[0]}`" class="hf-table-row" :class="{ leftovers: customLeftovers }">
                                            <label class="hf-row-label hf-row-label-books">
                                                <span>{{ labels[0] }}</span>
                                                <img :src="iconPath(labels[0])" :alt="labels[0]" />
                                            </label>
                                            <input type="text" :value="weaponOwned[labels[0]]" @input="setRecordValue(weaponOwned, labels[0], $event)" />
                                            <input type="text" :value="weaponPrices[labels[0]]" @input="setRecordValue(weaponPrices, labels[0], $event)" />
                                            <input
                                                v-if="customLeftovers"
                                                type="text"
                                                :value="weaponLeftover[labels[0]]"
                                                @input="setRecordValue(weaponLeftover, labels[0], $event)"
                                            />
                                        </div>
                                    </div>

                                    <div class="hf-table-wrap">
                                        <div class="hf-table-title-row" :class="{ leftovers: customLeftovers }">
                                            <span />
                                            <span>Owned</span>
                                            <span>Price</span>
                                            <span v-if="customLeftovers">Left</span>
                                        </div>
                                        <div v-for="labels in JUICE_LABELS" :key="`armor-${labels[1]}`" class="hf-table-row" :class="{ leftovers: customLeftovers }">
                                            <label class="hf-row-label hf-row-label-books">
                                                <span>{{ labels[1] }}</span>
                                                <img :src="iconPath(labels[1])" :alt="labels[1]" />
                                            </label>
                                            <input type="text" :value="armorOwned[labels[1]]" @input="setRecordValue(armorOwned, labels[1], $event)" />
                                            <input type="text" :value="armorPrices[labels[1]]" @input="setRecordValue(armorPrices, labels[1], $event)" />
                                            <input
                                                v-if="customLeftovers"
                                                type="text"
                                                :value="armorLeftover[labels[1]]"
                                                @input="setRecordValue(armorLeftover, labels[1], $event)"
                                            />
                                        </div>
                                    </div>
                                </div>

                                <label class="hf-inline-check" style="margin-top: 10px">
                                    <input v-model="customLeftovers" type="checkbox" />
                                    <span>Custom leftover values</span>
                                </label>
                            </div>
                        </section>
                    </aside>
                </div>

                <section class="hf-card hf-analysis-pane">
                    <div class="hf-card-header">
                        <div class="hf-card-title"><span class="hf-card-title-dot" />Analysis</div>
                        <div class="hf-analysis-tabs">
                            <button :class="['hf-analysis-tab', { active: analysisTab === 'distribution' }]" @click="analysisTab = 'distribution'">Distribution</button>
                            <button :class="['hf-analysis-tab', { active: analysisTab === 'breakdown' }]" @click="analysisTab = 'breakdown'">Gold Breakdown</button>
                        </div>
                    </div>
                    <div class="hf-card-body">
                        <section v-if="analysisTab === 'distribution'">
                            <div class="hf-dist-desc">Distribution reflects free-tap and juice usage from your current optimizer output.</div>
                            <div class="hf-dist-graphs">
                                <div v-for="(label, index) in distributionLabels" :key="`graph-${label}`" class="hf-graph-row">
                                    <div class="hf-graph-icon">
                                        <img :src="iconPath(label)" :alt="label" />
                                    </div>
                                    <MaterialGraph
                                        :data="histogramResult?.cum_percentiles?.[index] ?? null"
                                        :average="histogramResult?.average?.[index] ?? null"
                                        :secondary-annotation="histogramResult?.budgets?.[index] ?? null"
                                        :color-var="GRAPH_COLORS[index]"
                                        :cumulative="cumulativeGraph"
                                        :height="120"
                                    />
                                </div>
                            </div>
                        </section>

                        <section v-else>
                            <div class="hf-breakdown-grid">
                                <div class="hf-breakdown-table">
                                    <div v-for="(label, index) in MATS_LABELS.slice(0, 7)" :key="`mats-breakdown-${label}`" class="hf-breakdown-row">
                                        <span class="hf-breakdown-label">{{ label }}</span>
                                        <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(index))]">
                                            {{ breakdownText(goldBreakdownValue(index)) }}
                                        </span>
                                    </div>
                                </div>

                                <div class="hf-breakdown-table">
                                    <div v-for="(label, index) in JUICE_LABELS.map((pair) => pair[0])" :key="`weapon-breakdown-${label}`" class="hf-breakdown-row">
                                        <span class="hf-breakdown-label">{{ label }}</span>
                                        <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(7 + index))]">
                                            {{ breakdownText(goldBreakdownValue(7 + index)) }}
                                        </span>
                                    </div>
                                </div>

                                <div class="hf-breakdown-table">
                                    <div
                                        v-for="(label, index) in JUICE_LABELS.map((pair) => pair[1])"
                                        :key="`armor-breakdown-${label}`"
                                        class="hf-breakdown-row"
                                    >
                                        <span class="hf-breakdown-label">{{ label }}</span>
                                        <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(7 + juiceAvail + index))]">
                                            {{ breakdownText(goldBreakdownValue(7 + juiceAvail + index)) }}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            <div class="hf-combined-cost">Combined: {{ metricToText(currentMetric) }}</div>
                        </section>
                    </div>
                </section>
            </div>
        </div>
    </div>
</template>

<style>
.hf-main-content {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
}

.hf-top-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(336px, 368px);
    gap: 14px;
    align-items: start;
    min-width: 0;
}

.hf-main-stage {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-right: 8px;
    min-width: 0;
}

.hf-main-stage > .hf-honing-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(224px, 262px);
    gap: 12px;
    align-items: start;
    min-width: 0;
}

.hf-ops-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(280px, 320px);
    gap: 10px;
    align-items: stretch;
    min-width: 0;
}

.hf-compact-row {
    display: grid;
    grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
    gap: 10px;
    align-items: start;
    min-width: 0;
}

.hf-side-lane {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
}

.hf-card {
    min-width: 0;
}

.hf-main-stage .hf-advanced-card .hf-card-hint {
    white-space: nowrap;
}

.hf-main-stage .hf-advanced-card {
    width: min(100%, 262px);
    min-width: 0;
}

.hf-copy {
    margin: 0 0 8px;
    font-size: 13px;
    color: var(--hf-text-main);
    line-height: 1.35;
}

.hf-grid-content {
    display: flex;
    gap: 4px;
    min-width: 0;
}

.hf-grid-content-compact {
    justify-content: flex-start;
}

.hf-grid-scroll {
    overflow-x: auto;
    min-width: 0;
}

.hf-label-col {
    width: 110px;
    min-width: 110px;
}

.hf-label-col-compact {
    width: 44px;
    min-width: 44px;
}

.hf-label-row {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
}

.hf-equip-label {
    width: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 14px;
    text-align: right;
}

.hf-equip-label-compact {
    justify-content: center;
}

.hf-equip-label img {
    width: 27px;
    height: 27px;
    object-fit: contain;
}

.hf-cell-grid {
    display: grid;
    gap: 0;
    margin-bottom: 2px;
}

.hf-cell-grid-head {
    margin-bottom: 4px;
}

.hf-cell {
    width: 26px;
    height: 26px;
    border: 1px solid var(--checkbox-border);
    border-radius: 2px;
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    line-height: 1;
    user-select: none;
}

.hf-cell.selected {
    background: var(--checkbox-checked-bg);
    color: var(--checkbox-checked-text);
    border-color: var(--checkbox-checked-border);
}

.hf-cell-header {
    font-size: 10px;
}

.hf-optimizer-card .hf-card-body {
    padding-top: 10px;
}

.hf-ops-row > .hf-card {
    height: 100%;
}

.hf-ops-row .hf-optimizer-card .hf-card-body {
    display: flex;
    padding-top: 8px;
    padding-bottom: 8px;
}

.hf-ops-row .optimizer-card {
    gap: 8px;
    padding: 9px 10px;
}

.hf-ops-row .hf-optimize-btn {
    width: 100%;
    min-height: 46px;
    font-size: 24px;
}

.hf-ops-row .hf-inline-check {
    font-size: 12px;
}

.hf-ops-row .hf-metric-card {
    padding: 8px 10px;
    gap: 3px;
}

.hf-ops-row .hf-metric-label {
    font-size: 11px;
}

.hf-ops-row .hf-metric-value {
    font-size: 30px;
}

.hf-ops-row .hf-metric-status {
    font-size: 12px;
}

.hf-ops-row .hf-result-note {
    font-size: 11px;
    text-align: left;
    align-self: flex-start;
    width: 100%;
}

.optimizer-card {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--btn-border);
    border-radius: 8px;
    background: rgba(10, 13, 19, 0.5);
}

.hf-optimize-btn {
    width: 100%;
    min-height: 54px;
    border-radius: 10px;
    border: 1px solid rgba(10, 12, 16, 0.55);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.25);
    cursor: pointer;
    font-family: var(--hf-font-display);
    font-size: 34px;
    letter-spacing: 0.01em;
    white-space: nowrap;
    text-align: center;
    line-height: 1;
}

.hf-metric-card {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    padding: 10px 12px;
    background: rgba(10, 14, 20, 0.45);
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.hf-metric-label {
    color: var(--hf-text-muted);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
}

.hf-metric-value {
    font-family: var(--hf-font-display);
    font-size: 36px;
    line-height: 1;
    font-weight: 700;
}

.hf-metric-value.best {
    color: var(--brighter-optimizer);
}

.hf-metric-value.not-best {
    color: var(--hf-text-bright);
}

.hf-metric-status {
    font-size: 13px;
    line-height: 1.2;
}

.hf-metric-status.warn {
    color: #ff6464;
}

.hf-metric-status.muted {
    color: var(--hf-text-muted);
}

.hf-metric-status.best {
    color: var(--brighter-optimizer);
}

.hf-result-note {
    color: var(--hf-text-muted);
    font-size: 12px;
}

.optimizer-error {
    font-size: 12px;
    color: #ff6464;
}

.optimizer-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
}

.progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--hf-gold-dim), var(--hf-gold));
    transition: width 0.2s ease;
}

.hf-upgrade-editor {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
}

.hf-upgrade-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
    padding: 8px;
    border: 1px solid var(--hf-border-subtle);
    border-radius: 6px;
    background: rgba(10, 13, 19, 0.35);
    min-width: 0;
}

.hf-upgrade-order {
    width: 24px;
    min-width: 24px;
    height: 24px;
    border-radius: 999px;
    border: 1px solid rgba(212, 179, 90, 0.5);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-family: var(--hf-font-display);
    font-size: 12px;
    color: var(--hf-text-bright);
    background: rgba(212, 179, 90, 0.12);
    flex: 0 0 auto;
}

.hf-upgrade-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1 1 auto;
}

.hf-upgrade-topline {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.hf-upgrade-name {
    font-size: 13px;
    color: var(--hf-text-bright);
}

.hf-upgrade-plan {
    font-size: 12px;
    color: var(--hf-text-main);
}

.hf-upgrade-free {
    font-size: 11px;
    color: var(--free-tap);
    border: 1px solid rgba(255, 155, 232, 0.35);
    border-radius: 999px;
    padding: 2px 8px;
}

.hf-upgrade-actions-row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
}

.hf-action-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 7px;
    border: 1px solid var(--hf-border-subtle);
    border-radius: 4px;
    background: rgba(8, 10, 15, 0.45);
    font-size: 11px;
    color: var(--hf-text-main);
    min-width: 0;
}

.hf-action-chip img {
    width: 14px;
    height: 14px;
    object-fit: contain;
}

.hf-action-chip.tap {
    border-color: rgba(212, 179, 90, 0.52);
    color: var(--hf-text-bright);
}

.hf-action-chip.juice {
    background: rgba(79, 195, 195, 0.14);
}

.hf-action-chip.book {
    background: rgba(212, 179, 90, 0.16);
}

.hf-action-chip.muted {
    color: var(--hf-text-muted);
}

.hf-upgrade-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 0 0 auto;
    align-self: center;
}

.hf-upgrade-controls label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--hf-text-main);
}

.hf-upgrade-controls input[type="number"] {
    width: 72px;
    height: 28px;
    background: var(--grid-cell-bg);
    border: 1px solid var(--border-accent);
    color: var(--grid-cell-text);
}

.hf-upgrade-controls-readonly {
    align-self: flex-start;
}

.hf-upgrade-readonly-note {
    font-size: 12px;
    color: var(--hf-text-muted);
}

.hf-mini-btn {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 6px;
    background: var(--hf-bg-raised);
    color: var(--hf-text-main);
    padding: 5px 10px;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
}

.hf-mini-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
}

.hf-freetap-table {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.hf-freetap-head,
.hf-freetap-row {
    display: grid;
    grid-template-columns: 42px minmax(0, 1fr) 90px minmax(0, 220px);
    align-items: center;
    gap: 8px;
    font-size: 13px;
}

.hf-freetap-head {
    color: var(--hf-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 11px;
}

.hf-freetap-row {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 6px;
    padding: 6px 8px;
    background: rgba(10, 13, 19, 0.35);
}

.hf-freetap-row.draggable {
    cursor: grab;
}

.hf-freetap-row.dragging {
    opacity: 0.55;
    border-style: dashed;
}

.hf-freetap-row.drop-target {
    border-color: rgba(212, 179, 90, 0.7);
    background: rgba(212, 179, 90, 0.1);
}

.hf-freetap-upgrade {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.hf-freetap-actions {
    display: inline-flex;
    gap: 5px;
    flex-wrap: wrap;
}

.hf-drag-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    padding: 0 10px;
    border: 1px dashed var(--hf-border-subtle);
    border-radius: 6px;
    font-size: 11px;
    color: var(--hf-text-muted);
    letter-spacing: 0.04em;
    text-transform: uppercase;
}

.hf-tap-card .hf-card-body,
.hf-freetap-card .hf-card-body {
    max-height: 248px;
    overflow: auto;
}

.hf-tap-card .hf-copy {
    margin-bottom: 6px;
}

.hf-tap-card .hf-upgrade-row {
    padding: 7px;
    gap: 8px;
}

.hf-freetap-card .hf-freetap-head,
.hf-freetap-card .hf-freetap-row {
    grid-template-columns: 28px minmax(150px, 1.75fr) 64px minmax(130px, 1fr);
}

.hf-material-stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.hf-table-title-row,
.hf-table-row {
    display: grid;
    grid-template-columns: 1fr 86px 72px;
    gap: 0;
    align-items: stretch;
}

.hf-table-title-row.leftovers,
.hf-table-row.leftovers {
    grid-template-columns: 1fr 86px 72px 72px;
}

.hf-table-title-row {
    margin-bottom: 3px;
}

.hf-table-title-row span {
    font-size: 11px;
    color: var(--hf-text-muted);
    text-align: center;
    line-height: 1.1;
}

.hf-table-title-row span:first-child {
    text-align: right;
    padding-right: 8px;
}

.hf-row-label {
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 13px;
    min-width: 0;
    text-align: right;
    padding-right: 8px;
}

.hf-row-label img {
    width: 22px;
    height: 22px;
    object-fit: contain;
}

.hf-row-label-books {
    line-height: 1.05;
}

.hf-table-row input {
    width: 100%;
    height: 30px;
    border: 1px solid var(--border-accent);
    color: var(--grid-cell-text);
    background: var(--grid-cell-bg);
    padding: 5px 8px;
}

.hf-input-placeholder {
    border: 1px solid transparent;
}

.hf-options-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.hf-options-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
}

.hf-divider {
    height: 1px;
    background: var(--border-accent);
    margin: 2px 0;
}

.hf-inline-check {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 13px;
}

.hf-inline-check.compact {
    font-size: 12px;
}

.hf-inline-check input {
    width: 14px;
    height: 14px;
}

.hf-analysis-pane .hf-card-body {
    min-width: 0;
}

.hf-analysis-tabs {
    display: inline-flex;
    gap: 6px;
}

.hf-analysis-tab {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 999px;
    background: rgba(10, 13, 19, 0.48);
    color: var(--hf-text-main);
    padding: 6px 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
}

.hf-analysis-tab.active {
    background: rgba(212, 179, 90, 0.22);
    border-color: rgba(212, 179, 90, 0.6);
    color: var(--hf-text-bright);
}

.hf-dist-desc {
    margin-bottom: 8px;
    color: var(--hf-text-main);
    font-size: 13px;
}

.hf-dist-graphs {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.hf-graph-row {
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    padding-top: 6px;
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr);
    gap: 8px;
    align-items: center;
    min-width: 0;
}

.hf-graph-row:first-child {
    border-top: none;
    padding-top: 2px;
}

.hf-graph-icon {
    display: grid;
    place-items: center;
}

.hf-graph-icon img {
    width: 30px;
    height: 30px;
    object-fit: contain;
}

.hf-breakdown-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
}

.hf-breakdown-table {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    padding: 8px;
    background: rgba(10, 13, 19, 0.35);
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.hf-breakdown-row {
    display: grid;
    grid-template-columns: 98px minmax(0, 1fr);
    gap: 8px;
    align-items: center;
}

.hf-breakdown-label {
    color: var(--hf-text-main);
    font-size: 13px;
}

.hf-breakdown-value {
    font-size: 12px;
    border: 1px solid var(--border-accent);
    padding: 5px 8px;
    background: var(--grid-cell-bg);
    line-height: 1.25;
}

.hf-breakdown-value.cost {
    color: var(--deficit);
}

.hf-breakdown-value.surplus {
    color: var(--text-success);
}

.hf-breakdown-value.muted {
    color: var(--hf-text-muted);
}

.hf-combined-cost {
    margin-top: 12px;
    font-family: var(--hf-font-display);
    font-size: 34px;
    color: var(--brighter-optimizer);
}

@media (max-width: 1680px) {
    .hf-top-grid {
        grid-template-columns: minmax(0, 1fr) minmax(310px, 344px);
    }

    .hf-main-stage > .hf-honing-row {
        grid-template-columns: minmax(0, 1fr) minmax(214px, 248px);
    }

    .hf-main-stage .hf-advanced-card {
        width: min(100%, 248px);
    }
}

@media (max-width: 1480px) {
    .hf-top-grid {
        grid-template-columns: 1fr;
    }
}

@media (max-width: 1240px) {
    .hf-main-stage > .hf-honing-row,
    .hf-compact-row,
    .hf-ops-row {
        grid-template-columns: 1fr;
    }

    .hf-breakdown-grid {
        grid-template-columns: 1fr;
    }

    .hf-freetap-head,
    .hf-freetap-row {
        grid-template-columns: 36px minmax(0, 1fr) 90px;
    }

    .hf-freetap-actions {
        grid-column: 1 / -1;
    }
}

@media (max-width: 820px) {
    .hf-upgrade-row {
        flex-direction: column;
    }

    .hf-upgrade-controls {
        align-self: flex-start;
        flex-wrap: wrap;
    }

    .hf-optimize-btn {
        font-size: 28px;
    }

    .hf-metric-value,
    .hf-combined-cost {
        font-size: 28px;
    }
}
</style>
