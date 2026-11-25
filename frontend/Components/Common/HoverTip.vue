<script setup lang="ts">
import {
  computed,
  onBeforeUnmount,
  onUpdated,
  ref,
  useSlots,
  watch,
} from "vue";
import {
  autoUpdate,
  flip,
  offset,
  shift,
  useFloating,
  type Placement,
} from "@floating-ui/vue";

// Hover card around the default slot. Content is `text` or the #tip slot.
// Sticky cards stay open while the cursor is on them, so they can hold buttons.
// `anchor` pins the card to a viewport point (a graph's hovered dot) instead of the wrapper.
type Side = "top" | "bottom" | "left" | "right";

const props = withDefaults(
  defineProps<{
    text?: string | null;
    placement?: Side;
    sticky?: boolean;
    maxWidth?: number;
    anchor?: (() => { x: number; y: number }) | null;
  }>(),
  {
    text: null,
    placement: "bottom",
    sticky: true,
    maxWidth: 300,
    anchor: null,
  },
);

// preferred side, then the rest in this order
const FALLBACKS: Record<Side, Placement[]> = {
  right: ["left", "bottom", "top"],
  bottom: ["top"],
  top: ["bottom"],
  left: ["right", "bottom", "top"],
};

const slots = useSlots();
const wrapper = ref<HTMLElement | null>(null);
const floating = ref<HTMLElement | null>(null);
const content = ref<HTMLElement | null>(null);
const open = ref(false);

const reference = computed(() => {
  const anchor = props.anchor;
  if (!anchor) return wrapper.value;
  return {
    getBoundingClientRect: () => {
      const { x, y } = anchor();
      return {
        x,
        y,
        left: x,
        top: y,
        right: x,
        bottom: y,
        width: 0,
        height: 0,
      };
    },
  };
});

const { floatingStyles, placement, isPositioned } = useFloating(
  reference,
  floating,
  {
    open,
    placement: () => props.placement,
    transform: false,
    whileElementsMounted: autoUpdate,
    middleware: () => [
      offset(8),
      flip({ fallbackPlacements: FALLBACKS[props.placement] }),
      shift({ padding: 8 }),
    ],
  },
);

const side = computed(() => placement.value.split("-")[0] as Side);
const has_tip = computed(() => !!props.text || !!slots.tip);

// a wrapped box stays at its max width in CSS; shrink it to the widest laid-out line
onUpdated(() => {
  const el = content.value;
  if (!el) return;
  el.style.width = "";
  const range = document.createRange();
  range.selectNodeContents(el);
  const rects = [...range.getClientRects()];
  if (!rects.length) return;
  const left = Math.min(...rects.map((r) => r.left));
  el.style.width = `${Math.ceil(Math.max(...rects.map((r) => r.right)) - left)}px`;
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
  if (props.sticky) return set_open(false, 50);
  clearTimeout(timer);
  open.value = false;
}
function stay() {
  clearTimeout(timer);
}

// touch has no hover: a tap opens the card, a tap anywhere else closes it
function tap(event: PointerEvent) {
  if (event.pointerType !== "touch" || !has_tip.value) return;
  clearTimeout(timer);
  open.value = true;
}
function outside(event: PointerEvent) {
  const target = event.target as Node;
  if (wrapper.value?.contains(target) || floating.value?.contains(target))
    return;
  clearTimeout(timer);
  open.value = false;
}
const emit = defineEmits<{ closed: [] }>();
watch(open, (is_open) => {
  if (is_open) return document.addEventListener("pointerdown", outside);
  document.removeEventListener("pointerdown", outside);
  emit("closed");
});
onBeforeUnmount(() => {
  stay();
  document.removeEventListener("pointerdown", outside);
});
</script>

<template>
  <span
    ref="wrapper"
    @mouseenter="enter"
    @mouseleave="leave"
    @pointerdown="tap"
    class="align-middl inline-flex"
  >
    <slot />
    <Teleport to="body">
      <div
        v-if="open && has_tip"
        ref="floating"
        :data-side="side"
        class="tooltip-card z-10000 w-max text-left font-normal"
        :class="{ invisible: !isPositioned, 'pointer-events-none': !sticky }"
        :style="{ ...floatingStyles, maxWidth: `${maxWidth}px` }"
        @mouseenter="stay"
        @mouseleave="leave"
      >
        <div ref="content" class="text-balance whitespace-pre-line">
          <slot name="tip">{{ text }}</slot>
        </div>
      </div>
    </Teleport>
  </span>
</template>
