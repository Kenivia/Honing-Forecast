<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { createOCREngine, type OCREngine } from "tesseract-wasm";

interface Props {
  modelUrl?: string;
}

const props = withDefaults(defineProps<Props>(), {
  modelUrl: "/eng.traineddata",
});

interface PsmOption {
  value: string;
  label: string;
}

const psmOptions: PsmOption[] = [
  { value: "3", label: "3 — Fully automatic layout analysis (engine default)" },
  { value: "4", label: "4 — Single column of text, variable sizes" },
  { value: "6", label: "6 — Single uniform block of text" },
  { value: "7", label: "7 — Single text line" },
  { value: "8", label: "8 — Single word" },
  { value: "11", label: "11 — Sparse text, no particular order" },
];

type PadColor = "auto" | "white" | "black";

const pageSegMode = ref("7");
const charWhitelist = ref("");
const applyThreshold = ref(true);
const luminosityThreshold = ref(128);
const upscaleFactor = ref(2);
const borderPx = ref(12);
const padColor = ref<PadColor>("auto");

interface Timings {
  imageDecodeMs: number;
  preprocessMs: number;
  modelLoadMs: number;
  recognizeMs: number;
  totalMs: number;
}

const previewUrl = ref<string | null>(null);
const processedPreviewUrl = ref<string | null>(null);
const resultText = ref<string | null>(null);
const timings = ref<Timings | null>(null);
const isProcessing = ref(false);
const error = ref<string | null>(null);

let engine: OCREngine | null = null;
let modelLoaded = false;

async function ensureEngine(): Promise<OCREngine> {
  if (!engine) {
    engine = await createOCREngine();
  }
  return engine;
}

async function ensureModelLoaded(ocr: OCREngine): Promise<number> {
  if (modelLoaded) {
    return 0;
  }
  const start = performance.now();
  const response = await fetch(props.modelUrl);
  const buffer = await response.arrayBuffer();
  ocr.loadModel(buffer);
  modelLoaded = true;
  return performance.now() - start;
}

function drawToCanvas(
  source: ImageBitmap | HTMLCanvasElement,
  width: number,
  height: number,
): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) {
    throw new Error("Could not get 2D canvas context");
  }
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(source, 0, 0, width, height);
  return canvas;
}

function thresholdCanvas(canvas: HTMLCanvasElement, threshold: number): void {
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) {
    throw new Error("Could not get 2D canvas context");
  }
  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
  const data = imageData.data;

  for (let i = 0; i < data.length; i += 4) {
    const luminosity =
      0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2];
    const value = luminosity >= threshold ? 255 : 0;
    data[i] = value;
    data[i + 1] = value;
    data[i + 2] = value;
  }

  ctx.putImageData(imageData, 0, 0);
}

function addBorder(
  canvas: HTMLCanvasElement,
  border: number,
  mode: PadColor,
): HTMLCanvasElement {
  if (border <= 0) {
    return canvas;
  }

  let fillColor = "#ffffff";
  if (mode === "black") {
    fillColor = "#000000";
  } else if (mode === "auto") {
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const corner = ctx?.getImageData(0, 0, 1, 1).data;
    fillColor = corner && corner[0] > 127 ? "#ffffff" : "#000000";
  }

  const bordered = document.createElement("canvas");
  bordered.width = canvas.width + border * 2;
  bordered.height = canvas.height + border * 2;
  const ctx = bordered.getContext("2d");
  if (!ctx) {
    throw new Error("Could not get 2D canvas context");
  }
  ctx.fillStyle = fillColor;
  ctx.fillRect(0, 0, bordered.width, bordered.height);
  ctx.drawImage(canvas, border, border);
  return bordered;
}

async function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) {
    return;
  }

  error.value = null;
  resultText.value = null;
  timings.value = null;
  processedPreviewUrl.value = null;

  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
  }
  previewUrl.value = URL.createObjectURL(file);

  isProcessing.value = true;
  const totalStart = performance.now();

  try {
    const ocr = await ensureEngine();

    const decodeStart = performance.now();
    const original = await createImageBitmap(file);
    const imageDecodeMs = performance.now() - decodeStart;

    const preprocessStart = performance.now();
    let stage = drawToCanvas(
      original,
      original.width * upscaleFactor.value,
      original.height * upscaleFactor.value,
    );
    if (applyThreshold.value) {
      thresholdCanvas(stage, luminosityThreshold.value);
    }
    stage = addBorder(stage, borderPx.value, padColor.value);
    processedPreviewUrl.value = stage.toDataURL();
    const recognitionImage = await createImageBitmap(stage);
    const preprocessMs = performance.now() - preprocessStart;

    const modelLoadMs = await ensureModelLoaded(ocr);

    ocr.setVariable("tessedit_pageseg_mode", pageSegMode.value);
    ocr.setVariable("tessedit_char_whitelist", charWhitelist.value);

    const recognizeStart = performance.now();
    ocr.loadImage(recognitionImage);
    const text = ocr.getText();
    const recognizeMs = performance.now() - recognizeStart;

    resultText.value = text;
    timings.value = {
      imageDecodeMs,
      preprocessMs,
      modelLoadMs,
      recognizeMs,
      totalMs: performance.now() - totalStart,
    };
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    isProcessing.value = false;
    input.value = "";
  }
}

onBeforeUnmount(() => {
  engine?.destroy();
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
  }
});
</script>

<template>
  <div class="mx-auto flex max-w-2xl flex-col gap-4 bg-white p-6">
    <div>
      <h1 class="text-lg font-semibold text-neutral-900">OCR demo</h1>
      <p class="text-sm text-neutral-500">
        Upload an image to run it through tesseract-wasm.
      </p>
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Page segmentation mode
        <select
          v-model="pageSegMode"
          class="rounded-none border border-neutral-300 px-2 py-1.5 text-sm text-neutral-800"
          :disabled="isProcessing"
        >
          <option
            v-for="option in psmOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </select>
      </label>

      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Character whitelist (optional)
        <input
          v-model="charWhitelist"
          type="text"
          placeholder="e.g. 0123456789"
          class="rounded-none border border-neutral-300 px-2 py-1.5 text-sm text-neutral-800"
          :disabled="isProcessing"
        />
      </label>
    </div>

    <div
      class="grid grid-cols-1 gap-3 rounded-none border border-neutral-200 px-4 py-3 sm:grid-cols-2"
    >
      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Upscale factor: {{ upscaleFactor }}x
        <input
          v-model.number="upscaleFactor"
          type="range"
          min="1"
          max="4"
          step="1"
          :disabled="isProcessing"
        />
      </label>

      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Border: {{ borderPx }}px
        <input
          v-model.number="borderPx"
          type="range"
          min="0"
          max="40"
          step="1"
          :disabled="isProcessing"
        />
      </label>

      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Border color
        <select
          v-model="padColor"
          class="rounded-none border border-neutral-300 px-2 py-1.5 text-sm text-neutral-800"
          :disabled="isProcessing || borderPx === 0"
        >
          <option value="auto">Auto (match corner pixel)</option>
          <option value="white">White</option>
          <option value="black">Black</option>
        </select>
      </label>

      <div class="flex flex-col gap-1 text-sm text-neutral-600">
        <label class="flex items-center gap-2">
          <input
            v-model="applyThreshold"
            type="checkbox"
            :disabled="isProcessing"
          />
          Apply luminosity threshold
        </label>
        <input
          v-model.number="luminosityThreshold"
          type="range"
          min="0"
          max="255"
          :disabled="isProcessing || !applyThreshold"
        />
        <span class="text-xs text-neutral-400"
          >Threshold: {{ luminosityThreshold }}</span
        >
      </div>
    </div>

    <label
      class="flex cursor-pointer flex-col items-center gap-2 rounded-lg border border-dashed border-neutral-300 px-6 py-8 text-sm text-neutral-500 transition hover:border-neutral-400 hover:bg-neutral-50"
    >
      <span>Click to choose an image</span>
      <input
        type="file"
        accept="image/*"
        class="hidden"
        :disabled="isProcessing"
        @change="handleFileChange"
      />
    </label>

    <div v-if="previewUrl" class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <div class="flex flex-col gap-1">
        <div class="text-xs text-neutral-400">Original</div>
        <img
          :src="previewUrl"
          alt="Selected image preview"
          class="max-h-64 w-fit rounded-none border border-neutral-200 object-contain"
        />
      </div>
      <div v-if="processedPreviewUrl" class="flex flex-col gap-1">
        <div class="text-xs text-neutral-400">Sent to OCR</div>
        <img
          :src="processedPreviewUrl"
          alt="Preprocessed image preview"
          class="max-h-64 w-fit rounded-none border border-neutral-200 object-contain"
        />
      </div>
    </div>

    <div v-if="isProcessing" class="text-sm text-neutral-500">Running OCR…</div>

    <div
      v-if="error"
      class="rounded-none border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
    >
      {{ error }}
    </div>
    <pre
      v-if="resultText !== null"
      class="rounded-none border border-neutral-200 bg-neutral-50 px-4 py-3 font-mono text-sm whitespace-pre-wrap text-neutral-800"
      >{{ resultText || "(no text detected)" }}</pre
    >
    <div
      v-if="timings"
      class="grid grid-cols-2 gap-x-6 gap-y-1 rounded-none border border-neutral-200 px-4 py-3 text-sm sm:grid-cols-5"
    >
      <div>
        <div class="text-neutral-400">Decode</div>
        <div class="font-mono text-neutral-800">
          {{ timings.imageDecodeMs.toFixed(1) }} ms
        </div>
      </div>
      <div>
        <div class="text-neutral-400">Preprocess</div>
        <div class="font-mono text-neutral-800">
          {{ timings.preprocessMs.toFixed(1) }} ms
        </div>
      </div>
      <div>
        <div class="text-neutral-400">Model load</div>
        <div class="font-mono text-neutral-800">
          {{ timings.modelLoadMs.toFixed(1) }} ms
        </div>
      </div>
      <div>
        <div class="text-neutral-400">Recognize</div>
        <div class="font-mono text-neutral-800">
          {{ timings.recognizeMs.toFixed(1) }} ms
        </div>
      </div>
      <div>
        <div class="text-neutral-400">Total</div>
        <div class="font-mono text-neutral-800">
          {{ timings.totalMs.toFixed(1) }} ms
        </div>
      </div>
    </div>
  </div>
</template>
