from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import pandas as pd
import plotly.graph_objects as go


@dataclass
class TrialOptions:
    options: list[dict]
    default_value: str | None
    has_mc: bool


def empty_figure(message: str) -> go.Figure:
    fig = go.Figure()
    fig.add_annotation(
        text=message,
        x=0.5,
        y=0.5,
        xref="paper",
        yref="paper",
        showarrow=False,
        font={"size": 14},
    )
    fig.update_layout(
        xaxis={"visible": False},
        yaxis={"visible": False},
        margin={"l": 40, "r": 40, "t": 40, "b": 40},
    )
    return fig


def build_trial_options(history_df: pd.DataFrame, test_case: str, base_metric: str) -> TrialOptions:
    if history_df.empty or not test_case or not base_metric:
        return TrialOptions(options=[], default_value=None, has_mc=False)

    filtered = history_df[
        (history_df["test_case"] == test_case)
        & (history_df["metric_type"] == base_metric)
    ]
    trials = sorted(filtered["trial_num"].unique())

    has_mc = not history_df[
        (history_df["test_case"] == test_case)
        & (history_df["metric_type"] == f"MC_{base_metric}")
    ].empty

    options = [{"label": f"Trial {trial}", "value": f"trial:{trial}"} for trial in trials]
    if trials:
        options.append({"label": "Average across trials", "value": "avg"})
    if has_mc:
        options.append({"label": "MC", "value": "mc"})

    default_value = options[0]["value"] if options else None
    return TrialOptions(options=options, default_value=default_value, has_mc=has_mc)


def _axis_range(values: pd.Series) -> list[float] | None:
    if values.empty:
        return None
    return [float(values.min()), float(values.max())]


def _build_common_layout(fig: go.Figure, title: str) -> go.Figure:
    fig.update_layout(
        title=title,
        legend={
            "x": 1.02,
            "y": 1.0,
            "xanchor": "left",
            "yanchor": "top",
        },
        margin={"l": 60, "r": 240, "t": 60, "b": 60},
    )
    return fig


def build_trial_figure(
    history_df: pd.DataFrame,
    test_case: str,
    base_metric: str,
    trial_selection: str,
    x_axis: str,
) -> go.Figure:
    if history_df.empty:
        return empty_figure("No data found in Results folder.")

    if trial_selection == "mc":
        metric_type = f"MC_{base_metric}"
        filtered = history_df[
            (history_df["test_case"] == test_case)
            & (history_df["metric_type"] == metric_type)
        ]
    elif trial_selection == "avg":
        metric_type = base_metric
        filtered = history_df[
            (history_df["test_case"] == test_case)
            & (history_df["metric_type"] == metric_type)
        ]
    else:
        try:
            trial_num = int(trial_selection.split(":", 1)[1])
        except (IndexError, ValueError, AttributeError):
            return empty_figure("Select a valid trial.")
        metric_type = base_metric
        filtered = history_df[
            (history_df["test_case"] == test_case)
            & (history_df["metric_type"] == metric_type)
            & (history_df["trial_num"] == trial_num)
        ]

    if filtered.empty:
        return empty_figure("No data for the selected filters.")

    x_column = "time" if x_axis == "time" else "iter"

    all_for_range = history_df[
        (history_df["test_case"] == test_case)
        & (
            (history_df["metric_type"] == base_metric)
            | (history_df["metric_type"] == f"MC_{base_metric}")
        )
    ]

    x_range = _axis_range(all_for_range[x_column])
    y_range = _axis_range(all_for_range["metric"])
    best_known = all_for_range["metric"].max() if not all_for_range.empty else None

    fig = go.Figure()
    for model in sorted(filtered["model"].unique()):
        model_df = filtered[filtered["model"] == model]
        if trial_selection == "avg":
            model_df = (
                model_df.groupby("iter", as_index=False)["metric"]
                .mean()
                .sort_values("iter")
            )
            fig.add_trace(
                go.Scatter(
                    x=model_df["iter"],
                    y=model_df["metric"],
                    mode="lines",
                    name=model,
                )
            )
        else:
            model_df = model_df.sort_values(x_column)
            fig.add_trace(
                go.Scatter(
                    x=model_df[x_column],
                    y=model_df["metric"],
                    mode="lines",
                    name=model,
                )
            )

    if best_known is not None:
        fig.add_hline(
            y=float(best_known),
            line_dash="dash",
            line_color="green",
            annotation_text="best known solution",
            annotation_position="top right",
        )

    fig.update_xaxes(title=x_column)
    fig.update_yaxes(title=f"{base_metric} metric")

    if x_range:
        fig.update_xaxes(range=x_range)
    if y_range:
        fig.update_yaxes(range=y_range)

    title = f"{test_case} - {metric_type}"
    return _build_common_layout(fig, title)


def build_aggregate_figure(
    history_df: pd.DataFrame,
    family: str,
    base_metric: str,
) -> go.Figure:
    if history_df.empty:
        return empty_figure("No data found in Results folder.")

    family_df = history_df[
        (history_df["family"] == family)
        & (history_df["metric_type"] == base_metric)
    ]

    if family_df.empty:
        return empty_figure("No data for the selected family.")

    best_known = (
        family_df.groupby("test_case")["metric"]
        .max()
        .rename("best_known")
    )
    family_df = family_df.join(best_known, on="test_case")
    family_df = family_df[family_df["best_known"] > 0]

    if family_df.empty:
        return empty_figure("No valid data for aggregation (best known <= 0).")

    family_df = family_df.assign(
        pct_diff=(family_df["metric"] - family_df["best_known"]) / family_df["best_known"]
    )

    fig = go.Figure()
    global_min = None

    for model in sorted(family_df["model"].unique()):
        model_df = family_df[family_df["model"] == model]

        def geo_pct(series: pd.Series) -> float:
            values = 1.0 + series.to_numpy(dtype=float)
            values = values[values > 0]
            if values.size == 0:
                return float("nan")
            return float(np.exp(np.log(values).mean()) - 1.0)

        trial_grouped = (
            model_df.groupby(["trial_num", "iter"])["pct_diff"]
            .apply(geo_pct)
            .reset_index(name="trial_geo")
        )

        trial_grouped = trial_grouped.dropna(subset=["trial_geo"])
        if trial_grouped.empty:
            continue

        per_iter = trial_grouped.groupby("iter")["trial_geo"]
        mean_series = per_iter.mean()
        count_series = per_iter.count()
        std_series = per_iter.std(ddof=0)
        min_series = per_iter.min()
        max_series = per_iter.max()

        iter_values = mean_series.index.to_numpy()
        mean_values = mean_series.to_numpy()
        count_values = count_series.to_numpy()
        std_values = std_series.fillna(0).to_numpy()
        min_values = min_series.to_numpy()
        max_values = max_series.to_numpy()

        err_plus = np.zeros_like(mean_values, dtype=float)
        err_minus = np.zeros_like(mean_values, dtype=float)
        for idx, count in enumerate(count_values):
            if count >= 3:
                err_plus[idx] = std_values[idx]
                err_minus[idx] = std_values[idx]
            elif count == 2:
                err_plus[idx] = max_values[idx] - mean_values[idx]
                err_minus[idx] = mean_values[idx] - min_values[idx]

        if mean_values.size == 0:
            continue

        min_with_err = (mean_values - err_minus).min()
        if global_min is None or min_with_err < global_min:
            global_min = float(min_with_err)

        fig.add_trace(
            go.Scatter(
                x=iter_values,
                y=mean_values,
                mode="lines",
                name=model,
                error_y={
                    "type": "data",
                    "array": err_plus,
                    "arrayminus": err_minus,
                    "visible": True,
                },
            )
        )

    if not fig.data:
        return empty_figure("No data for the selected family.")

    fig.update_xaxes(title="iter")
    fig.update_yaxes(title=f"{base_metric} geo mean % diff vs best")

    if global_min is not None:
        padding = abs(global_min) * 0.1 if global_min != 0 else 0.1
        fig.update_yaxes(range=[global_min - padding, 0.1])

    title = f"{family} - aggregate"
    return _build_common_layout(fig, title)
