<script setup lang="ts">
// Two chasing arms, after "6-dots-scale" of SVG Spinners (MIT, Utkarsh Verma): twelve dots in a
// ring pop up in turn and shrink slowly. Dots half a ring apart share a phase, hence two arms.
const DOTS = Array.from({ length: 12 }, (_, k) => {
  const angle = (k * Math.PI) / 6;
  return {
    cx: 15 + 11.25 * Math.sin(angle),
    cy: 15 - 11.25 * Math.cos(angle),
    delay: `${(k % 6) * 0.1 - 0.6}s`,
  };
});
</script>

<template>
  <svg viewBox="0 0 30 30" fill="currentColor">
    <circle
      v-for="(dot, k) in DOTS"
      :key="k"
      class="dot"
      :cx="dot.cx"
      :cy="dot.cy"
      r="2"
      :style="{ animationDelay: dot.delay }"
    />
  </svg>
</template>

<style scoped>
.dot {
  transform-box: fill-box;
  transform-origin: center;
  animation: chase 0.6s infinite;
}
@keyframes chase {
  0% {
    animation-timing-function: cubic-bezier(0, 1, 0, 1);
    transform: scale(0);
  }
  17% {
    animation-timing-function: cubic-bezier(0.53, 0, 0.61, 0.73);
    transform: scale(1);
  }
  80%,
  100% {
    transform: scale(0);
  }
}
</style>
