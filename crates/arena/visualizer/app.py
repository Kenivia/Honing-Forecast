from __future__ import annotations

from pathlib import Path

from dash import Dash, dcc, html, Input, Output, State

from data_loader import load_results
from processing import build_trial_options, build_trial_figure, build_aggregate_figure, empty_figure


ROOT = Path(__file__).resolve().parents[1]
RESULTS_DIR = ROOT / "Results"
DATA = load_results(RESULTS_DIR)


def _first_or_none(items: list[str]) -> str | None:
    return items[0] if items else None


DEFAULT_METRIC = "Avg" if "Avg" in DATA.base_metrics else _first_or_none(DATA.base_metrics)

app = Dash(__name__)
app.title = "Honing results Explorer"

app.layout = html.Div(
    style={"display": "flex", "flexDirection": "row", "gap": "16px"},
    children=[
        html.Div(
            style={
                "minWidth": "240px",
                "maxWidth": "280px",
                "padding": "16px",
                "borderRight": "1px solid #ddd",
            },
            children=[
                html.H4("Controls"),
                html.Div(
                    children=[
                        html.Label("Family"),
                        dcc.Dropdown(
                            id="family-dropdown",
                            options=[{"label": fam, "value": fam} for fam in DATA.families],
                            value=_first_or_none(DATA.families),
                            clearable=False,
                        ),
                    ],
                ),
                html.Div(
                    style={"marginTop": "12px"},
                    children=[
                        html.Label("Test case"),
                        dcc.Dropdown(
                            id="test-case-dropdown",
                            options=[{"label": tc, "value": tc} for tc in DATA.test_cases],
                            value=_first_or_none(DATA.test_cases),
                            clearable=False,
                        ),
                    ],
                ),
                html.Div(
                    style={"marginTop": "12px"},
                    children=[
                        html.Label("Aggregate family"),
                        dcc.Checklist(
                            id="aggregate-toggle",
                            options=[{"label": "Enable", "value": "aggregate"}],
                            value=[],
                        ),
                    ],
                ),
                html.Div(
                    style={"marginTop": "12px"},
                    children=[
                        html.Label("Metric"),
                        dcc.RadioItems(
                            id="metric-dropdown",
                            options=[
                                {"label": metric, "value": metric}
                                for metric in DATA.base_metrics
                            ],
                            value=DEFAULT_METRIC,
                        ),
                    ],
                ),
                html.Div(
                    style={"marginTop": "12px"},
                    children=[
                        html.Label("Trial / Mode"),
                        dcc.Dropdown(id="trial-dropdown", clearable=False),
                    ],
                ),
                html.Div(
                    style={"marginTop": "12px"},
                    children=[
                        html.Label("X axis"),
                        dcc.RadioItems(
                            id="x-axis-radio",
                            options=[
                                {"label": "time", "value": "time"},
                                {"label": "iter", "value": "iter"},
                            ],
                            value="time",
                        ),
                    ],
                ),
            ],
        ),
        html.Div(
            style={"flex": "1", "padding": "16px"},
            children=[
                dcc.Graph(
                    id="main-graph",
                    figure=empty_figure("Loading data..."),
                    style={"height": "80vh"},
                )
            ],
        ),
    ],
)
\
    
@app.callback(
    Output("trial-dropdown", "options"),
    Output("trial-dropdown", "value"),
    Input("test-case-dropdown", "value"),
    Input("metric-dropdown", "value"),
    State("trial-dropdown", "value"),
)
def update_trial_options(test_case: str, base_metric: str, current_value: str | None):
    options_bundle = build_trial_options(DATA.history_df, test_case, base_metric)
    values = [option["value"] for option in options_bundle.options]
    if current_value in values:
        return options_bundle.options, current_value
    return options_bundle.options, options_bundle.default_value


@app.callback(
    Output("test-case-dropdown", "options"),
    Output("test-case-dropdown", "value"),
    Input("family-dropdown", "value"),
    State("test-case-dropdown", "value"),
)
def update_test_cases(family: str, current_test_case: str | None):
    if not family:
        options = [{"label": tc, "value": tc} for tc in DATA.test_cases]
    else:
        family_cases = sorted(
            DATA.history_df[DATA.history_df["family"] == family]["test_case"].unique()
        )
        options = [{"label": tc, "value": tc} for tc in family_cases]

    values = [option["value"] for option in options]
    if current_test_case in values:
        return options, current_test_case
    return options, values[0] if values else None


@app.callback(
    Output("main-graph", "figure"),
    Output("x-axis-radio", "value"),
    Output("x-axis-radio", "disabled"),
    Output("test-case-dropdown", "disabled"),
    Input("trial-dropdown", "value"),
    Input("test-case-dropdown", "value"),
    Input("family-dropdown", "value"),
    Input("metric-dropdown", "value"),
    Input("x-axis-radio", "value"),
    Input("aggregate-toggle", "value"),
)
def update_graph(
    trial_selection: str,
    test_case: str,
    family: str,
    base_metric: str,
    x_axis: str,
    aggregate_toggle: list[str],
):
    is_aggregate = "aggregate" in (aggregate_toggle or [])
    if is_aggregate:
        family_value = family or _first_or_none(DATA.families)
        fig = build_aggregate_figure(DATA.history_df, family_value if family_value is not None else "", base_metric)
        return fig, "iter", True, True

    if not test_case or not base_metric:
        return empty_figure("Select a test case and metric."), x_axis, False, False

    if trial_selection == "avg":
        fig = build_trial_figure(DATA.history_df, test_case, base_metric, trial_selection, "iter")
        return fig, "iter", True, False

    fig = build_trial_figure(DATA.history_df, test_case, base_metric, trial_selection, x_axis)
    return fig, x_axis, False, False


if __name__ == "__main__":
    app.run(debug=True)
