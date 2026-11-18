<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { Chart, LinearScale, Tooltip } from "chart.js";
import { MatrixController, MatrixElement } from "chartjs-chart-matrix";
import { cssVar } from "@/Utils/Helpers";

Chart.register(MatrixController, MatrixElement, LinearScale, Tooltip);

// A grid of cells at x = 1..nx, y = 1..ny, coloured by value.
const props = defineProps<{
  nx: number;
  ny: number;
  value: (x: number, y: number) => number;
  color: (v: number) => string;
  tooltip: (x: number, y: number) => string[];
  x_label: string;
  y_label: string;
}>();

const canvas = ref<HTMLCanvasElement | null>(null);
// box height that makes the plot area nx:ny, so every cell is square
const height = ref(300);
const square = {
  id: "square",
  afterLayout(chart: Chart) {
    const a = chart.chartArea;
    const want = (a.width * props.ny) / props.nx + chart.height - a.height;
    if (Math.abs(want - height.value) > 1) height.value = want;
  },
};
let chart: Chart | null = null;

function config() {
  const muted = cssVar("--text-muted", "#b7b7b7");
  const data = [];
  for (let x = 1; x <= props.nx; x++)
    for (let y = 1; y <= props.ny; y++)
      data.push({ x, y, v: props.value(x, y) });
  const axis = (n: number, title: string) => ({
    type: "linear",
    min: 0.5,
    max: n + 0.5,
    offset: false,
    title: { display: true, text: title, color: muted },
    reverse: false,
    ticks: {
      color: muted,
      callback: (v: number) => (Number.isInteger(v) ? v : ""),
    },
    grid: { display: false },
  });
  return {
    type: "matrix" as const,
    plugins: [square],
    data: {
      datasets: [
        {
          data,
          backgroundColor: (c: any) => props.color(c.raw?.v ?? 0),
          borderWidth: 0,
          width: ({ chart }: any) =>
            (chart.chartArea?.width ?? 0) / props.nx + 1,
          height: ({ chart }: any) =>
            (chart.chartArea?.height ?? 0) / props.ny + 1,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false as const,
      scales: {
        x: axis(props.nx, props.x_label),
        y: axis(props.ny, props.y_label),
      },
      plugins: {
        legend: { display: false },
        tooltip: {
          displayColors: false,
          callbacks: {
            title: () => "",
            label: (item: any) => props.tooltip(item.raw.x, item.raw.y),
          },
        },
      },
    },
  };
}

onMounted(() => {
  chart = new Chart(canvas.value, config() as any);
});
watch(
  () => [
    props.nx,
    props.ny,
    props.value,
    props.color,
    props.tooltip,
    props.x_label,
    props.y_label,
  ],
  () => {
    const next = config();
    chart.data = next.data as any;
    chart.options = next.options as any;
    chart.update();
  },
);
onBeforeUnmount(() => chart?.destroy());
</script>

<template>
  <div class="relative w-full" :style="{ height: `${height}px` }">
    <canvas ref="canvas"></canvas>
  </div>
</template>
