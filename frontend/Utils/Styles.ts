import { Slider, styled } from "@mui/material"

// Graph dimensions constants

export const GRAPH_WIDTH = 800
export const GRAPH_HEIGHT = 400
export const SMALL_GRAPH_WIDTH = 640
export const SMALL_GRAPH_HEIGHT = 320
// Styled Material UI Slider with custom colors
export const StyledSlider = styled(Slider)(() => ({
    width: 100,
    color: "var(--slider-track-active)",
    "& .MuiSlider-track": {
        border: "none",
        backgroundColor: "var(--slider-track-active)",
        height: 6,
    },
    "& .MuiSlider-rail": {
        backgroundColor: "var(--slider-track-bg)",
        height: 6,
    },
    "& .MuiSlider-thumb": {
        backgroundColor: "var(--slider-thumb-bg)",
        border: "2px solid var(--slider-thumb-bg)",
        width: 20,
        height: 20,
        "&:hover, &.Mui-focusVisible": {
            backgroundColor: "var(--slider-thumb-hover)",
            borderColor: "var(--slider-thumb-focus)",
            boxShadow: `0 0 0 8px var(--slider-thumb-shadow)`,
        },
        "&.Mui-active": {
            backgroundColor: "var(--slider-thumb-hover)",
            borderColor: "var(--slider-thumb-focus)",
        },
    },
    "& .MuiSlider-valueLabel": {
        backgroundColor: "var(--accent-hover)",
        color: "black",
        fontSize: "12px",
        // fontWeight: "bold",
        border: "1px solid white",
    },
}))

export const styles: any = {
    pageContainer: {
        minHeight: "100vh",
        background: "var(--bg-primary)",
        display: "grid",
        placeItems: "center",
        padding: "var(--spacing-xl)",
        boxSizing: "border-box",
        paddingBottom: "500px",
    },
    mainContainer: {
        display: "flex",
        flexDirection: "column",
        gap: "var(--spacing-2xl)",
        boxSizing: "border-box",
        margin: 10,
    },
    heading: {
        color: "var(--text-primary)",
        fontSize: "var(--font-size-2xl)",
        fontWeight: "var(--font-weight-bold)",
        marginBottom: "var(--spacing-sm)",
    },
    sectionTitle: {
        color: "var(--text-primary)",
        fontSize: "var(--font-size-lg)",
        fontWeight: "var(--font-weight-semibold)",
    },
    gridSection: {
        background: "var(--bg-secondary)",
        borderRadius: "var(--border-radius)",
        padding: "var(--spacing-xl)",
        border: "1px solid var(--border-primary)",
    },
    buttonSection: {
        background: "var(--bg-secondary)",
        borderRadius: "var(--border-radius)",
        padding: "var(--spacing-xl)",
        border: "1px solid var(--border-primary)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--spacing-md)",
        alignItems: "flex-start",
    },
    inputSection: {
        background: "var(--bg-secondary)",
        borderRadius: "var(--border-radius)",
        padding: "var(--spacing-xl)",
        border: "1px solid var(--border-primary)",
    },
    inputLabelCell: {
        textAlign: "left",
        paddingRight: 8,
        color: "var(--text-secondary)",
        whiteSpace: "nowrap",
        overflow: "visible",
        textOverflow: "ellipsis",
    },
    inputCell: {
        flex: 1,
        padding: "6px 6px",
        border: "1px solid var(--border-accent)",
        background: "transparent",
        color: "var(--text-primary)",
        borderRadius: 0,
    },
    controls: {
        marginTop: 14,
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "flex-end",
        alignItems: "center",
    },
    primaryButton: {
        background: "var(--btn-primary)",
        color: "var(--btn-primary-text)",
        padding: "8px 12px",
        borderRadius: "var(--border-radius-small)",
        border: "none",
        cursor: "pointer",
    },
    successButton: {
        background: "var(--btn-success)",
        color: "var(--btn-success-text)",
        padding: "8px 12px",
        borderRadius: "var(--border-radius-small)",
        border: "none",
        cursor: "pointer",
    },
    demoButton: {
        background: "var(--btn-demo)",
        color: "var(--btn-demo-text)",
        padding: "8px 16px",
        borderRadius: "var(--border-radius-small)",
        border: "none",
        cursor: "pointer",
        fontSize: "var(--font-size-sm)",
    },
}

export type ColumnDef = {
    headerName: string

    editable: boolean
    flex: number
    width: string
    background: string
    backgroundSelected: string
    color: string
    backgroundRanOut?: string
    textAlign?: "left" | "center" | "right"
}

// Column definitions for spreadsheet grids
export const createColumnDefs = () => {
    const chanceToCostColumnDefs: ColumnDef[] = [
        {
            headerName: "Est. cost",
            editable: false,
            flex: 1,
            width: "80px",
            background: "var(--grid-cell-bg-readonly)",
            backgroundSelected: "var(--grid-cell-selected-readonly)",
            color: "var(--text-success)",
        },
    ]

    const matsColumnDef: ColumnDef[] = [
        {
            headerName: "Owned",
            editable: true,
            flex: 1,
            width: "80px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
        {
            headerName: "Market price",
            editable: true,
            flex: 1,
            width: "50px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
        {
            headerName: "",
            editable: true,
            flex: 1,
            width: "100px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
    ]

    const juiceColumnDef: ColumnDef[] = [
        {
            headerName: "Owned",
            editable: true,
            flex: 1,
            width: "80px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
        {
            headerName: "Market price",
            editable: true,
            flex: 1,
            width: "50px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
        {
            headerName: "Leftover value",
            editable: true,
            flex: 1,
            width: "170px",
            background: "var(--grid-cell-bg)",
            backgroundSelected: "var(--grid-cell-selected)",
            color: "var(--grid-cell-text)",
        },
    ]
    const wideMatsColumnDefs: ColumnDef[] = [
        // {
        //     headerName: "Owned",
        //     editable: true,
        //     flex: 1,
        //     width: "80px",
        //     background: "var(--grid-cell-bg)",
        //     backgroundSelected: "var(--grid-cell-selected)",
        //     color: "var(--grid-cell-text)",
        // },

        {
            headerName: "Breakdown",
            editable: false,
            flex: 1,
            width: "200px",
            background: "var(--grid-cell-bg-readonly)",
            backgroundSelected: "var(--grid-cell-selected-readonly)",
            color: "var(--text-success)",
            textAlign: "right",
        },
    ]
    return { chanceToCostColumnDefs, matsColumnDef, juiceColumnDef, wideMatsColumnDefs }
}
