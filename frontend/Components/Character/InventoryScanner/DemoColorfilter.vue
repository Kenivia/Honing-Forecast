<template>
  <div class="chf-root">
    <div class="chf-controls">
      <label class="chf-file-btn">
        <input type="file" accept="image/*" @change="onFileChange" />
        {{ fileName || "Choose image" }}
      </label>

      <div class="chf-field">
        <label for="chf-hex">Target color</label>
        <div class="chf-color-row">
          <input id="chf-hex-swatch" type="color" v-model="colorPickerValue" />
          <input
            id="chf-hex"
            type="text"
            v-model="hexInput"
            @change="onHexInputChange"
            placeholder="#ff0000"
            maxlength="7"
          />
        </div>
      </div>

      <div class="chf-field">
        <label for="chf-hue">Hue tolerance: {{ hueTolerance }}°</label>
        <input
          id="chf-hue"
          type="range"
          min="0"
          max="10"
          step="0.01"
          v-model.number="hueTolerance"
        />
      </div>

      <div class="chf-field">
        <label for="chf-brightness"
          >Brightness tolerance: {{ brightnessTolerance }}%</label
        >
        <input
          id="chf-brightness"
          type="range"
          min="0"
          max="100"
          step="1"
          v-model.number="brightnessTolerance"
        />
      </div>

      <div class="chf-field">
        <label for="chf-sat"
          >Saturation tolerance: {{ saturationTolerance }}%</label
        >
        <input
          id="chf-sat"
          type="range"
          min="0"
          max="100"
          step="1"
          v-model.number="saturationTolerance"
        />
      </div>

      <label class="chf-checkbox">
        <input type="checkbox" v-model="useSaturation" />
        Also filter by saturation
      </label>

      <button v-if="hasImage" class="chf-download" @click="downloadResult">
        Download result
      </button>
    </div>

    <div class="chf-preview">
      <div v-if="!hasImage" class="chf-empty">Upload an image to begin</div>
      <canvas v-show="hasImage" ref="canvasRef"></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from "vue";

const fileName = ref("");
const hasImage = ref(false);
const canvasRef = ref<HTMLCanvasElement | null>(null);

const hexInput = ref("#ff0000");
const colorPickerValue = ref("#ff0000");
const hueTolerance = ref(30);
const brightnessTolerance = ref(30);
const saturationTolerance = ref(100);
const useSaturation = ref(false);

let originalImageData: ImageData | null = null;
let objectUrl: string | null = null;

const MAX_DIMENSION = 1600;

function hexToRgb(hex: string): [number, number, number] | null {
  const match = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex.trim());
  if (!match) return null;
  return [
    parseInt(match[1], 16),
    parseInt(match[2], 16),
    parseInt(match[3], 16),
  ];
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  const delta = max - min;
  if (delta === 0) return [0, 0, l * 100];
  const s = delta / (1 - Math.abs(2 * l - 1));
  let h: number;
  switch (max) {
    case rn:
      h = ((gn - bn) / delta) % 6;
      break;
    case gn:
      h = (bn - rn) / delta + 2;
      break;
    default:
      h = (rn - gn) / delta + 4;
  }
  h *= 60;
  if (h < 0) h += 360;
  return [h, s * 100, l * 100];
}

function hueDistance(a: number, b: number): number {
  const diff = Math.abs(a - b) % 360;
  return diff > 180 ? 360 - diff : diff;
}

function luminance(r: number, g: number, b: number): number {
  return 0.299 * r + 0.587 * g + 0.114 * b;
}

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  fileName.value = file.name;
  if (objectUrl) URL.revokeObjectURL(objectUrl);
  objectUrl = URL.createObjectURL(file);

  const img = new Image();
  img.onload = () => {
    let width = img.naturalWidth;
    let height = img.naturalHeight;
    const scale = Math.min(1, MAX_DIMENSION / Math.max(width, height));
    width = Math.round(width * scale);
    height = Math.round(height * scale);

    const canvas = canvasRef.value;
    if (!canvas) return;
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(img, 0, 0, width, height);
    originalImageData = ctx.getImageData(0, 0, width, height);
    hasImage.value = true;
    applyFilter();
  };
  img.src = objectUrl;
}

function onHexInputChange() {
  const rgb = hexToRgb(hexInput.value);
  if (rgb) {
    colorPickerValue.value = hexInput.value.startsWith("#")
      ? hexInput.value
      : `#${hexInput.value}`;
    applyFilter();
  }
}

watch(colorPickerValue, (val) => {
  hexInput.value = val;
  applyFilter();
});

watch(
  [hueTolerance, brightnessTolerance, saturationTolerance, useSaturation],
  () => {
    applyFilter();
  },
);

function applyFilter() {
  if (!originalImageData || !canvasRef.value) return;
  const rgb = hexToRgb(hexInput.value);
  if (!rgb) return;

  const [targetH, targetS] = rgbToHsl(rgb[0], rgb[1], rgb[2]);
  const targetLum = luminance(rgb[0], rgb[1], rgb[2]);
  const brightnessRange = (brightnessTolerance.value / 100) * 255;
  const satRange = (saturationTolerance.value / 100) * 100;

  const src = originalImageData;
  const out = new ImageData(
    new Uint8ClampedArray(src.data),
    src.width,
    src.height,
  );
  const data = out.data;

  for (let i = 0; i < data.length; i += 4) {
    const r = data[i];
    const g = data[i + 1];
    const b = data[i + 2];
    const [h, s] = rgbToHsl(r, g, b);
    const lum = luminance(r, g, b);

    const hueOk = hueDistance(h, targetH) <= hueTolerance.value;
    const brightnessOk = Math.abs(lum - targetLum) <= brightnessRange;
    const satOk = !useSaturation.value || Math.abs(s - targetS) <= satRange;

    if (!(hueOk && brightnessOk && satOk)) {
      data[i] = 0;
      data[i + 1] = 0;
      data[i + 2] = 0;
    }
  }

  const ctx = canvasRef.value.getContext("2d");
  if (!ctx) return;
  ctx.putImageData(out, 0, 0);
}

function downloadResult() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const link = document.createElement("a");
  link.download = `filtered-${fileName.value || "image"}.png`;
  link.href = canvas.toDataURL("image/png");
  link.click();
}

onBeforeUnmount(() => {
  if (objectUrl) URL.revokeObjectURL(objectUrl);
});
</script>

<style scoped>
.chf-root {
  display: flex;
  flex-wrap: wrap;
  gap: 1.5rem;
  font-family: system-ui, sans-serif;
}

.chf-controls {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 220px;
  max-width: 280px;
}

.chf-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.chf-field label {
  font-size: 0.85rem;
  font-weight: 600;
}

.chf-color-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.chf-color-row input[type="color"] {
  width: 2.5rem;
  height: 2.25rem;
  padding: 0;
  border: 1px solid #ccc;
  border-radius: 4px;
  cursor: pointer;
}

.chf-color-row input[type="text"] {
  flex: 1;
  padding: 0.4rem 0.5rem;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-family: monospace;
}

input[type="range"] {
  width: 100%;
}

.chf-file-btn {
  display: inline-block;
  padding: 0.5rem 0.75rem;
  border: 1px solid #ccc;
  border-radius: 6px;
  cursor: pointer;
  background: #f5f5f5;
  text-align: center;
  font-size: 0.85rem;
}

.chf-file-btn input[type="file"] {
  display: none;
}

.chf-checkbox {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
}

.chf-download {
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 6px;
  background: #2563eb;
  color: white;
  cursor: pointer;
  font-size: 0.85rem;
}

.chf-download:hover {
  background: #1d4ed8;
}

.chf-preview {
  flex: 1;
  min-width: 300px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(#e5e5e5 0% 25%, #ffffff 0% 50%) 50% /
    20px 20px;
  border-radius: 8px;
  min-height: 300px;
}

.chf-preview canvas {
  max-width: 100%;
  max-height: 70vh;
  display: block;
}

.chf-empty {
  color: #888;
  font-size: 0.9rem;
}
</style>
