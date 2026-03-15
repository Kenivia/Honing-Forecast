import json
import csv
import sys


def enhance_to_csv(input_path: str, output_path: str) -> None:
    with open(input_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    enhance_list = data["enhance"]

    fieldnames = [
        "nextLevel",
        "feedShard",
        "feedSilver",
        "successRate",
        # "blueJuiceRate",
        # "blueJuiceMaxCount",
        # "blueCount",
        "redJuiceRate",
        "redJuiceMaxCount",
        "redCount",
        "leapsCount",
        "fusionCount",
        "shard",
        "silver",
        "gold",
        "specialEnhanceNeed",
    ]

    with open(output_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()

        for entry in enhance_list:
            # Additives: assumed single "Blue juice" entry
            additive = entry["additives"][0]

            # Materials: index by itemName
            materials = {m["itemName"]: m["count"] for m in entry["materials"]}

            writer.writerow({
                "nextLevel":          entry["nextLevel"],
                "feedShard":          entry["feedShard"],
                "feedSilver":         entry["feedSilver"],
                "successRate":        entry["successRate"],
                # "blueJuiceRate":      additive["rate"],
                # "blueJuiceMaxCount":  additive["maxCount"],
                # "blueCount":          materials["Blue"],
                    "redJuiceRate":      additive["rate"],
                "redJuiceMaxCount":  additive["maxCount"],
                "redCount":          materials["red"],
                "leapsCount":         materials["leaps"],
                
                "leapsCount":         materials["leaps"],
                "fusionCount":        materials["fusion"],
                "shard":              entry["shard"],
                "silver":             entry["silver"],
                "gold":               entry["gold"],
                "specialEnhanceNeed": entry["specialEnhanceNeed"],
            })

    print(f"Done — wrote {len(enhance_list)} rows to '{output_path}'.")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python convert_enhance.py <input.json> <output.csv>")
        sys.exit(1)

    enhance_to_csv(sys.argv[1], sys.argv[2])