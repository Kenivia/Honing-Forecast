import ChangeLogsView from "@/Components/ChangeLogsView.vue"

export const all_change_logs = import.meta.glob("@/Changelogs/*.vue")

export const CHANGE_LOGS_ROUTES = Object.entries(all_change_logs).map(([path, component]) => {
    const version = path.replace("/frontend/Changelogs/", "").replace(".vue", "")
    return {
        path: `${version}`,
        name: `change-logs-${version}`,
        ChangeLogsView,
    }
})
function sort_versions(versions: string[]): string[] {
    return [...versions].sort((a, b) => {
        const parse = (v: string) => v.replace(/^v/, "").split(".").map(Number)
        const [aMajor, aMinor, aPatch] = parse(a)
        const [bMajor, bMinor, bPatch] = parse(b)

        return bMajor - aMajor || bMinor - aMinor || bPatch - aPatch
    })
}

export const ALL_VERSIONS = sort_versions(
    Object.entries(all_change_logs).map(([path]) => {
        return path.replace("/frontend/Changelogs/", "").replace(".vue", "")
    }),
)
export const LATEST_VERSION = ALL_VERSIONS[0]

export function minor_version_equal(a: string, b: string) {
    const a_split = a.replace("v", "").split(".")
    const b_split = b.replace("v", "").split(".")

    return a_split[0] === b_split[0] && a_split[1] === b_split[1]
}
// const latest_change_log =
