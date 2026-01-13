import json
import csv

input_path = "Result_default, v1_local_0_2026-01-13 21-18-09 +00-00.jsonl"
output_path = "output.csv"

required_fields = ["test_case", "metric_type", "trial_num", "best"]

with open(input_path, "r", encoding="utf-8") as fin, \
     open(output_path, "w", newline="", encoding="utf-8") as fout:

    writer = csv.DictWriter(fout, fieldnames=required_fields)
    writer.writeheader()

    for line_num, line in enumerate(fin, start=1):
        line = line.strip()
        if not line:
            continue

        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            # skip malformed JSON lines
            continue

        # Check all required fields are present
        if not all(field in obj for field in required_fields):
            continue

        try:
            row = {
                "test_case": int(obj["test_case"]),
                "metric_type": str(obj["metric_type"]),
                "trial_num": int(obj["trial_num"]),
                "best": float(obj["best"]),
            }
        except (TypeError, ValueError):
            # skip rows with wrong types
            continue

        writer.writerow(row)

print("Done.")
