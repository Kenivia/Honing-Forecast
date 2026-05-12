import { ref, onMounted, onUnmounted, Ref } from "vue";

export function useMediaIsNarrow(width: number): Ref<boolean, boolean> {
  const query = `(max-width: ${String(width)}px)`;
  const isNarrow = ref(window.matchMedia(query).matches);

  const media = window.matchMedia(query);

  const listener = () => {
    isNarrow.value = media.matches;
  };

  onMounted(() => media.addEventListener("change", listener));
  onUnmounted(() => media.removeEventListener("change", listener));
  return isNarrow;
}
