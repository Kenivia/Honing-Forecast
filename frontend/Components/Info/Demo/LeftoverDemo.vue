<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Chart,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
} from "chart.js";
import noUiSlider, { type API } from "nouislider";
import "nouislider/dist/nouislider.css";
import { cssVar } from "@/Utils/Helpers";
import {
  BAND_COLORS,
  BAND_LABELS,
  BAND_VALUES,
  BAND_VALUE_LABELS,
  MAIN_BANDS,
  band_value_allowed,
  type BandPlan,
  type BandValue,
} from "@/Utils/Constants";

Chart.register(LineController, LineElement, PointElement, LinearScale);

// Illustrative net cost (gold spent - leftover value) against material used, no real data.
// Owned bands are used in order at the slope of their leftover value, then the rest is bought.
const MAX = 100;
const SLOPE: Record<BandValue, number> = {
  Worthless: 0,
  TaxedSell: 0.95,
  Market: 1,
};
const plan = ref<BandPlan>([...MAIN_BANDS]);
const colors = [...BAND_COLORS, "--gold"].map((c) => cssVar(c, "#888888"));
const labels = [...BAND_LABELS, "Buy from market"];
const slopes = computed(() => [...plan.value.map((v) => SLOPE[v]), 1]);

// handle positions: end of bound, roster, tradable
const cuts = ref([25, 45, 70]);
const widths = computed(() =>
  [...cuts.value, MAX].map((c, i) => c - (i ? cuts.value[i - 1] : 0)),
);
// drawn from the bottom left, so net cost 0 sits at the owned bands' total value
const points = computed(() => {
  const out = [{ x: 0, y: 0 }];
  widths.value.forEach((w, i) =>
    out.push({ x: out[i].x + w, y: out[i].y + w * slopes.value[i] }),
  );
  return out;
});
const zero = computed(() => points.value[3].y);

// white plot with a dotted net-cost-0 line
const faint = cssVar("--text-very-muted", "#888888");
const backdrop = {
  id: "backdrop",
  beforeDraw(c: Chart) {
    const { ctx, chartArea: a } = c;
    ctx.fillStyle = "white";
    ctx.fillRect(a.left, a.top, a.width, a.height);
  },
  afterDatasetsDraw(c: Chart) {
    const { ctx, chartArea: a, scales } = c;
    const y = scales.y.getPixelForValue(zero.value);
    ctx.save();
    ctx.strokeStyle = faint;
    ctx.fillStyle = faint;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(a.left, y);
    ctx.lineTo(a.right, y);
    ctx.stroke();
    ctx.font = `14px ${Chart.defaults.font.family}`;
    ctx.textBaseline = "bottom";
    ctx.fillText(
      "Above this, you spend gold buying mats from market",
      a.left + 6,
      y - 4,
    );
    ctx.restore();
  },
};

const track = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
let slider: API | null = null;
let chart: Chart | null = null;

onMounted(() => {
  slider = noUiSlider.create(track.value!, {
    start: cuts.value,
    connect: [true, true, true, true],
    range: { min: 0, max: MAX },
    step: 1,
  });
  track
    .value!.querySelectorAll<HTMLElement>(".noUi-connect")
    .forEach((el, i) => (el.style.background = colors[i]));
  slider.on("update", (v) => (cuts.value = v.map(Number)));

  const muted = cssVar("--text-muted", "#b7b7b7");
  const grid = cssVar("--border-very-muted", "#6969693b");
  const axis = (title: string, min: number) => ({
    type: "linear" as const,
    min,
    max: MAX,
    title: { display: true, text: title, color: muted },
    ticks: { display: false },
    grid: { color: grid },
  });
  chart = new Chart(canvas.value!, {
    type: "line",
    plugins: [backdrop],
    data: {
      datasets: [
        {
          data: points.value,
          borderWidth: 3,
          pointRadius: 0,
          segment: { borderColor: (c) => colors[c.p0DataIndex] },
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false,
      events: [],
      scales: {
        x: axis("Material used", 0),
        y: axis("Gold spent − leftover value", -8),
      },
      plugins: { legend: { display: false } },
    },
  });
});
watch(points, (p) => {
  chart!.data.datasets[0].data = p;
  chart!.update();
});
onBeforeUnmount(() => {
  slider?.destroy();
  chart?.destroy();
});
</script>

<template>
  <div class="flex flex-col gap-3 pt-3 pl-5">
    <div ref="track" class="mx-2"></div>
    <div class="flex flex-wrap justify-between gap-x-6 gap-y-1 text-sm">
      <span
        v-for="(label, i) in labels"
        :key="label"
        :style="{ color: colors[i] }"
      >
        {{ label }} (gradient {{ slopes[i] }})
      </span>
    </div>
    <div class="relative h-72 w-full">
      <canvas ref="canvas"></canvas>
    </div>
    <!-- slot text on the left, the selectors on the right, equal halves -->
    <div class="grid grid-cols-2 items-center gap-4">
      <div><slot /></div>
      <div
        class="grid w-fit grid-cols-[auto_auto] items-center gap-x-2 gap-y-0.5 text-sm"
      >
        <template v-for="(label, band) in BAND_LABELS" :key="label">
          <span :style="{ color: colors[band] }">{{ label }}</span>
          <select
            class="selector"
            :aria-label="`${label} leftover value`"
            v-model="plan[band]"
          >
            <option
              v-for="value in BAND_VALUES"
              :key="value"
              :value="value"
              :disabled="!band_value_allowed(plan, band, value)"
            >
              {{ BAND_VALUE_LABELS[value] }}
            </option>
          </select>
        </template>
      </div>
    </div>
  </div>
</template>
