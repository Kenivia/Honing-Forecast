import { ref, computed, onMounted, onUnmounted } from "vue"
import { NARROW_WIDTH } from "./Constants"

export function useMediaIsNarrow(width = NARROW_WIDTH) {
    const query = `(max-width: ${String(width)}px)`
    const isNarrow = ref(window.matchMedia(query).matches)

    const media = window.matchMedia(query)

    const listener = () => {
        isNarrow.value = media.matches
    }

    onMounted(() => media.addEventListener("change", listener))
    onUnmounted(() => media.removeEventListener("change", listener))
    return { isNarrow }
}
