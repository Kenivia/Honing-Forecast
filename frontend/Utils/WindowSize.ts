import { MaybeRefOrGetter, ref, Ref, toValue, watchEffect } from "vue";

export function useMediaIsNarrow(
  width: MaybeRefOrGetter<number>,
): Ref<boolean> {
  const isNarrow = ref(false);

  watchEffect((onCleanup) => {
    const media = window.matchMedia(`(max-width: ${toValue(width)}px)`);
    isNarrow.value = media.matches;

    const listener = () => {
      isNarrow.value = media.matches;
    };
    media.addEventListener("change", listener);
    onCleanup(() => media.removeEventListener("change", listener));
  });

  return isNarrow;
}
