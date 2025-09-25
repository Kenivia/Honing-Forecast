// Graph dimensions constants
export const GRAPH_WIDTH = 800
export const GRAPH_HEIGHT = 400
export const SMALL_GRAPH_WIDTH = 640
export const SMALL_GRAPH_HEIGHT = 320

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
        width: 100,
        textAlign: "right",
        paddingRight: 8,
        color: "var(--text-secondary)",
        whiteSpace: "nowrap",
        overflow: "visible",
        textOverflow: "ellipsis",
    },
    inputCell: {
        flex: 1,
        padding: "6px 45px",
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

// Column definitions for spreadsheet grids
export const createColumnDefs = (autoOptimization: boolean) => {
    const chanceToCostColumnDefs = [
        {
            headerName: "Estimated cost",
            field: "budget",
            editable: false,
            flex: 1,
            cellStyle: { background: "var(--grid-cell-bg-readonly)", padding: "6px 8px" },
        },
    ]

    const costToChanceColumnDefs = autoOptimization
        ? [{ headerName: "Budget Input", field: "budget", editable: true, flex: 1, cellStyle: { background: "var(--grid-cell-bg)", padding: "6px 8px" } }]
        : [
              { headerName: "Budget Input", field: "budget", editable: true, flex: 1, cellStyle: { background: "var(--grid-cell-bg)", padding: "6px 8px" } },
              { headerName: "Gold Value", field: "matsValue", editable: true, flex: 1, cellStyle: { background: "var(--grid-cell-bg)", padding: "6px 8px" } },
          ]

    return { chanceToCostColumnDefs, costToChanceColumnDefs }
}
