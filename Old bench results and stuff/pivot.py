import pandas as pd

INPUT_CSV = "output3.csv"
OUTPUT_CSV = "output3_pivoted.csv"

# Load
df = pd.read_csv(INPUT_CSV)
df = df[["test_case", "metric_type", "trial_num", "best"]]

# Map metric types to a unified metric name
METRIC_MAP = {
    "MC_SA": "SA",
    "SA": "SA",
    "MC_Avg": "Avg",
    "Avg": "Avg",
}

df["metric_type_unified"] = df["metric_type"].map(METRIC_MAP)

# Safety check: catch unexpected metric types
if df["metric_type_unified"].isna().any():
    bad = df.loc[df["metric_type_unified"].isna(), "metric_type"].unique()
    raise ValueError(f"Unknown metric_type(s): {bad}")

# Pivot using the unified metric type
pivoted = (
    df
    .pivot_table(
        index=["test_case", "metric_type_unified"],
        columns="trial_num",
        values="best",
        aggfunc="first"
    )
    .reset_index()
)

# Rename columns: 0 -> best_0, ...
pivoted.columns = [
    f"best_{int(c)}" if isinstance(c, (int, float)) else c
    for c in pivoted.columns
]

# Final column order
final_cols = ["test_case", "metric_type_unified"] + [f"best_{i}" for i in range(6)]
pivoted = pivoted.reindex(columns=final_cols)

# Rename metric column back to desired name
pivoted = pivoted.rename(columns={"metric_type_unified": "metric_type"})

# Optional: sort for cleanliness
pivoted = pivoted.sort_values(["test_case", "metric_type"])

# Save
pivoted.to_csv(OUTPUT_CSV, index=False)

print(f"Wrote merged + pivoted CSV to {OUTPUT_CSV}")
