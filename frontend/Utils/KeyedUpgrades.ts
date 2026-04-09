import { KeyedUpgrades, OneUpgrade, OneUpgradeKey, StatusGrid } from "./Interfaces"
import { status_to_bool_grid } from "./StatusGrid"

export function to_upgrade_key(piece_type: number, upgrade_index: number, is_normal: boolean, tier: number): OneUpgradeKey {
    return `${piece_type},${upgrade_index},${is_normal},${tier}`
}

export function grids_to_keyed(normal_grid: StatusGrid, adv_grid: StatusGrid, all_keyed: KeyedUpgrades, tier: number) {
    let new_keyed: KeyedUpgrades = {}
    for (const [piece_type, row] of status_to_bool_grid(normal_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, true, tier)
            if (cell) {
                if (key in all_keyed && isOneUpgrade(all_keyed[key])) {
                    new_keyed[key] = all_keyed[key]
                } else {
                    new_keyed[key] = [piece_type, upgrade_index, true, 0, null, false, false, null]
                }
            }
        }
    }
    for (const [piece_type, row] of status_to_bool_grid(adv_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, false, tier)
            if (cell) {
                if (key in all_keyed && isOneUpgrade(all_keyed[key])) {
                    new_keyed[key] = all_keyed[key]
                } else {
                    new_keyed[key] = [piece_type, upgrade_index, false, null, null, false, false, [0, 0, false, false]]
                }
            }
        }
    }
    // console.log(new_keyed)
    return new_keyed
}

function isOneUpgrade(foo: unknown): foo is OneUpgrade {
    if (!Array.isArray(foo) || foo.length !== 8) return false

    const [f0, f1, f2, f3, f4, f5, f6, f7] = foo

    return (
        typeof f0 === "number" &&
        typeof f1 === "number" &&
        typeof f2 === "boolean" &&
        (f3 === null || typeof f3 === "number") &&
        (f4 === null || Array.isArray(f4)) &&
        (f4 === null || f4.every((x) => x.length == 2 && typeof x[0] === "boolean" && typeof x[1] === "number")) &&
        typeof f5 === "boolean" &&
        typeof f6 === "boolean" &&
        (f7 === null || (typeof f7[0] === "number" && typeof f7[1] === "number" && typeof f7[2] === "boolean" && typeof f7[3] === "boolean"))
    )
}
