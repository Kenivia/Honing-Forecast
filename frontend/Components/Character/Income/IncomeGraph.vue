<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Chart,
  Legend,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
  type ChartDataset,
} from "chart.js";
import annotationPlugin from "chartjs-plugin-annotation";
import { cssVar } from "@/Utils/Helpers";

Chart.register(
  LineController,
  LineElement,
  PointElement,
  LinearScale,
  Legend,
  Tooltip,
  annotationPlugin,
);

// Values over time on one y axis: each series' points joined, then a dashed projection.
export interface Series {
  label: string;
  color: string;
  points: { x: number; y: number }[];
  line: { x: number; y: number }[];
}

const props = defineProps<{
  series: Series[];
  range: [number, number];
  resets: number[];
  now: number;
  selected: number | null;
}>();
// the plot area in px, so the timeline can line up with it
const emit = defineEmits<{ area: [left: number, right: number] }>();

const canvas = ref<HTMLCanvasElement | null>(null);
let chart: Chart | null = null;

const date = (x: number) =>
  new Date(x).toLocaleDateString(undefined, { month: "short", day: "numeric" });
const amount = (y: number) =>
  y.toLocaleString("en-US", { maximumFractionDigits: 0 });

function config() {
  const muted = cssVar("--text-muted", "#b7b7b7");
  const border = cssVar("--border-very-muted", "#6969693b");
  const datasets: ChartDataset<"line">[] = [];
  const scales: Record<string, any> = {
    x: {
      type: "linear",
      min: props.range[0],
      max: props.range[1],
      afterBuildTicks: (scale) => {
        scale.ticks = props.resets.map((value) => ({ value }));
      },
      ticks: { color: muted, callback: (x: number) => date(x) },
      grid: { color: border },
    },
    y: {
      type: "linear",
      ticks: { color: muted, callback: amount },
      grid: { color: border },
    },
  };
  for (const series of props.series) {
    const color = cssVar(series.color, "#f0f0f0");
    datasets.push({
      label: series.label,
      data: series.points,
      borderColor: color,
      borderWidth: 1.5,
      backgroundColor: color,
      pointRadius: 4,
      pointHoverRadius: 6,
    });
    datasets.push({
      label: `${series.label} projected`,
      data: series.line,
      borderColor: color,
      borderDash: [6, 4],
      borderWidth: 1.5,
      backgroundColor: color,
      pointRadius: 3,
    });
  }
  const line = (x: number, color: string, label?: string) => ({
    type: "line",
    xMin: x,
    xMax: x,
    borderColor: color,
    borderWidth: 1,
    label: label
      ? {
          display: true,
          content: label,
          position: "start",
          color: muted,
          backgroundColor: "transparent",
        }
      : undefined,
  });
  return {
    type: "line" as const,
    data: { datasets },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false as const,
      parsing: false as const,
      scales,
      interaction: { mode: "nearest" as const, intersect: true },
      plugins: {
        legend: {
          labels: {
            color: muted,
            boxHeight: 2,
            filter: (item) => !item.text.endsWith(" projected"),
          },
        },
        tooltip: {
          callbacks: {
            title: (items) => new Date(items[0].parsed.x).toLocaleString(),
            label: (item) => `${item.dataset.label}: ${amount(item.parsed.y)}`,
          },
        },
        annotation: {
          annotations: {
            now: line(props.now, muted, "Now"),
            ...(props.selected === null
              ? {}
              : { selected: line(props.selected, "#ffffff") }),
          },
        },
      },
    },
    plugins: [
      {
        id: "area",
        afterLayout: (chart: Chart) =>
          emit("area", chart.chartArea.left, chart.chartArea.right),
      },
    ],
  };
}

onMounted(() => {
  chart = new Chart(canvas.value, config() as any);
});
watch(
  () => [props.series, props.range, props.resets, props.now, props.selected],
  () => {
    const next = config();
    chart.data = next.data;
    chart.options = next.options as any;
    chart.update();
  },
  { deep: true },
);
onBeforeUnmount(() => chart?.destroy());
</script>

<template>
  <div class="relative h-80 w-full">
    <canvas ref="canvas" aria-label="Income graph"></canvas>
  </div>
</template>
