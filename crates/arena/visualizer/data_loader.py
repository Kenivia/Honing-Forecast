from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import json

import pandas as pd


HISTORY_COLUMNS = [
    "model",
    "test_case",
    "family",
    "metric_type",
    "trial_num",
    "time",
    "iter",
    "metric",
]


@dataclass
class DataBundle:
    history_df: pd.DataFrame
    test_cases: list[str]
    families: list[str]
    models: list[str]
    base_metrics: list[str]


def model_from_filename(filename: str) -> str:
    if "_" in filename:
        return filename.split("_", 1)[0]
    return filename


def test_case_family(test_case: str) -> str:
    if len(test_case) <= 6:
        return test_case
    return test_case[:-6]


def _parse_history(history: list) -> list[tuple[float, int, float]]:
    parsed: list[tuple[float, int, float]] = []
    for entry in history:
        if not isinstance(entry, (list, tuple)) or len(entry) < 3:
            continue
        try:
            time_val = float(entry[0])
            iter_val = int(entry[1])
            metric_val = float(entry[2])
        except (TypeError, ValueError):
            continue
        parsed.append((time_val, iter_val, metric_val))
    return parsed


def load_results(results_dir: Path) -> DataBundle:
    rows: list[dict] = []
    results_dir = Path(results_dir)

    for jsonl_path in sorted(results_dir.glob("*.jsonl")):
        model = model_from_filename(jsonl_path.stem)
        try:
            with jsonl_path.open("r", encoding="utf-8") as handle:
                for line in handle:
                    raw = line.strip()
                    if not raw:
                        continue
                    try:
                        record = json.loads(raw)
                    except json.JSONDecodeError:
                        continue

                    test_case = record.get("test_case")
                    metric_type = record.get("metric_type")
                    trial_num = record.get("trial_num")
                    performance = record.get("performance") or {}
                    best_history = performance.get("best_history") or []

                    if test_case is None or metric_type is None:
                        continue
                    if trial_num is None:
                        trial_num = -1
                    try:
                        trial_num = int(trial_num)
                    except (TypeError, ValueError):
                        trial_num = -1

                    family = test_case_family(str(test_case))
                    parsed_history = _parse_history(best_history)
                    for time_val, iter_val, metric_val in parsed_history:
                        rows.append(
                            {
                                "model": model,
                                "test_case": str(test_case),
                                "family": family,
                                "metric_type": str(metric_type),
                                "trial_num": trial_num,
                                "time": time_val,
                                "iter": iter_val,
                                "metric": metric_val,
                            }
                        )
        except OSError:
            continue

    if rows:
        history_df = pd.DataFrame(rows)
    else:
        history_df = pd.DataFrame(columns=HISTORY_COLUMNS)

    test_cases = sorted(history_df["test_case"].unique()) if not history_df.empty else []
    families = sorted(history_df["family"].unique()) if not history_df.empty else []
    models = sorted(history_df["model"].unique()) if not history_df.empty else []

    base_metrics = sorted(
        {
            metric.replace("MC_", "")
            for metric in history_df["metric_type"].unique()
            if metric
        }
    ) if not history_df.empty else []

    return DataBundle(
        history_df=history_df,
        test_cases=test_cases,
        families=families,
        models=models,
        base_metrics=base_metrics,
    )
