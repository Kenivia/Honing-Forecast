<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useSlots } from "vue";
import {
  arrow,
  autoUpdate,
  flip,
  offset,
  shift,
  useFloating,
  type Placement,
} from "@floating-ui/vue";

// Wikipedia-style hover card around the default slot. Content is `text` or the #tip slot.
// Sticky cards stay open while the cursor is on them, so they can hold buttons.
type Side = "top" | "bottom" | "left" | "right";

const props = withDefaults(
  defineProps<{
    text?: string | null;
    placement?: Side;
    sticky?: boolean;
    maxWidth?: number;
  }>(),
  { text: null, placement: "bottom", sticky: true, maxWidth: 300 },
);

// preferred side, then its opposite, then below / above
const FALLBACKS: Record<Side, Placement[]> = {
  bottom: ["top-start"],
  top: ["bottom-start"],
  left: ["right-start", "bottom-start", "top-start"],
  right: ["left-start", "bottom-start", "top-start"],
};
// the arrow sits on the card edge facing the reference; these are its two outer edges
const ARROW_EDGE: Record<Side, string> = {
  bottom: "top-[-6px] border-t border-l",
  top: "bottom-[-6px] border-b border-r",
  right: "left-[-6px] border-b border-l",
  left: "right-[-6px] border-t border-r",
};

const slots = useSlots();
const reference = ref<HTMLElement | null>(null);
const floating = ref<HTMLElement | null>(null);
const arrow_el = ref<HTMLElement | null>(null);
const open = ref(false);

const { floatingStyles, placement, middlewareData, isPositioned } = useFloating(
  reference,
  floating,
  {
    open,
    placement: () => `${props.placement}-start` as Placement,
    transform: false,
    whileElementsMounted: autoUpdate,
    middleware: () => [
      offset({ mainAxis: 12, alignmentAxis: -10 }),
      flip({ fallbackPlacements: FALLBACKS[props.placement] }),
      shift({ padding: 8 }),
      arrow({ element: arrow_el, padding: 10 }),
    ],
  },
);

const side = computed(() => placement.value.split("-")[0] as Side);
const has_tip = computed(() => !!props.text || !!slots.tip);
// hidden until placed; a closing card is already placed and plays its leave animation
const visible = computed(() => isPositioned.value || !open.value);
const arrow_pos = computed(() => {
  const a = middlewareData.value.arrow;
  return {
    left: a?.x != null ? `${a.x}px` : "",
    top: a?.y != null ? `${a.y}px` : "",
  };
});

let timer = 0;
function set_open(value: boolean, ms: number) {
  clearTimeout(timer);
  timer = window.setTimeout(() => (open.value = value), ms);
}
function enter() {
  if (has_tip.value) set_open(true, 50);
}
function leave() {
  if (props.sticky) return set_open(false, 0);
  clearTimeout(timer);
  open.value = false;
}
function stay() {
  clearTimeout(timer);
}
onBeforeUnmount(stay);
</script>

<template>
  <span ref="reference" @mouseenter="enter" @mouseleave="leave">
    <slot />
    <Teleport to="body">
      <Transition name="hovertip">
        <div
          v-if="open && has_tip"
          ref="floating"
          :data-side="side"
          class="z-10000 w-max rounded-xs bg-(--bg-muted) p-1 text-left text-sm font-normal text-(--text-main) shadow-[0_30px_90px_-20px_rgba(0,0,0,0.3)] ring-1 ring-(--border-muted)"
          :class="[
            visible ? 'hovertip-in' : 'invisible',
            { 'pointer-events-none': !sticky },
          ]"
          :style="{ ...floatingStyles, maxWidth: `${maxWidth}px` }"
          @mouseenter="stay"
          @mouseleave="leave"
        >
          <div
            ref="arrow_el"
            class="absolute size-3 rotate-45 border-(--border-muted) bg-(--bg-muted)"
            :class="ARROW_EDGE[side]"
            :style="arrow_pos"
          />
          <slot name="tip">{{ text }}</slot>
        </div>
      </Transition>
    </Teleport>
  </span>
</template>

<style scoped>
/* fades in in place; fades out sliding 20px away from the reference */
[data-side="bottom"] {
  --hovertip-from: 0 20px;
}
[data-side="top"] {
  --hovertip-from: 0 -20px;
}
[data-side="left"] {
  --hovertip-from: -20px 0;
}
[data-side="right"] {
  --hovertip-from: 20px 0;
}
@keyframes hovertip-in {
  from {
    opacity: 0;
  }
}
@keyframes hovertip-out {
  to {
    opacity: 0;
  }
}
.hovertip-in {
  animation: hovertip-in 0.067s ease;
}
.hovertip-leave-active {
  animation: hovertip-out 0.067s ease forwards;
}
</style>
