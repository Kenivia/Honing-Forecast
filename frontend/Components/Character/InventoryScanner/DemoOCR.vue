<script setup lang="ts">
import { onBeforeUnmount, ref, watch, markRaw, type Ref } from "vue";
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

const pageSegMode = ref("8");
const charWhitelist = ref("0123456789+.");

// ---------------------------------------------------------------------------
// Filter framework
//
// Each preprocessing step (binarization, denoising, whatever comes next) is
// a `FilterDef`: a toggle, a name/description for the UI, zero or more
// numeric params (rendered as sliders), and an `apply` function that mutates
// a native-resolution canvas in place. To add a new filter: declare its refs,
// build a `FilterDef` for it, and push it into the `filters` array below —
// the settings UI and the processing pipeline both drive off that array, so
// nothing else needs to change.
//
// Filters run in array order, all at native resolution, before upscaling —
// some (like outline-aware extraction) depend on pixel adjacency that
// upscaling would blur. They're independent toggles rather than a mutually
// exclusive radio group: enabling more than one just chains them, which is
// harmless for the two built in here (thresholding an already-binary image
// is a no-op) and keeps the model simple for whatever gets added next.
// ---------------------------------------------------------------------------

interface FilterParam {
  id: string;
  label: string;
  min: number;
  max: number;
  step?: number;
  suffix?: string;
  value: Ref<number>;
}

interface FilterDef {
  id: string;
  name: string;
  description?: string;
  enabled: Ref<boolean>;
  params: FilterParam[];
  /** Mutates `canvas` in place. Called at native resolution, before upscaling. */
  apply: (canvas: HTMLCanvasElement) => void;
}

// --- Filter: global luminosity threshold ------------------------------------

const colorfulnessThreshold = ref(40);

const colorfulnessMaskFilter: FilterDef = {
  id: "colorfulness-mask",
  name: "Colorfulness Mask",
  description:
    "Whites out colored regions, leaving grayscale/near-neutral pixels untouched",
  enabled: ref(true),
  params: [
    {
      id: "threshold",
      label: "Colorfulness threshold",
      min: 0,
      max: 255,
      step: 1,
      suffix: "",
      value: colorfulnessThreshold,
    },
  ],
  apply: (canvas: HTMLCanvasElement) => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;
    const imageData = ctx.getImageData(0, 0, width, height);
    const data = imageData.data;
    const threshold = colorfulnessThreshold.value;

    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];

      const max = Math.max(r, g, b);
      const min = Math.min(r, g, b);
      const chroma = max - min; // 0 = perfectly gray, higher = more saturated
      const saturation = max === 0 ? 0 : chroma / max; // 0-1, normalizes for darkness
      if (saturation * 255 > threshold) {
        data[i] = 0; //Math.max(0, (1 - saturation) ** threshold * data[i]);
        data[i + 1] = 0; //Math.max(0, (1 - saturation) ** threshold * data[i + 1]);
        data[i + 2] = 0; //Math.max(0, (1 - saturation) ** threshold * data[i + 2]);
      }
    }

    ctx.putImageData(imageData, 0, 0);
  },
};

const luminosityThreshold = ref(100);
const thresholdFilter: FilterDef = {
  id: "threshold",
  name: "Luminosity threshold (binarize)",
  description:
    "Simple global cutoff: pixels at or above the threshold become white, everything darker becomes black.",
  enabled: ref(true),
  params: [
    {
      id: "threshold",
      label: "Threshold",
      min: 0,
      max: 255,
      value: luminosityThreshold,
    },
  ],
  apply: (canvas) => {
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) {
      throw new Error("Could not get 2D canvas context");
    }
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageData.data;

    for (let i = 0; i < data.length; i += 4) {
      const luminosity =
        0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2];
      const value = luminosity >= luminosityThreshold.value ? luminosity : 0;
      data[i] = value;
      data[i + 1] = value;
      data[i + 2] = value;
    }

    ctx.putImageData(imageData, 0, 0);
  },
};

const floodTolerance = ref(100);

const backgroundFloodFillFilter: FilterDef = {
  id: "background-flood-fill",
  name: "Background Flood Fill",
  description:
    "Fills the white region connected to the image border with pure black, leaving enclosed white holes untouched",
  enabled: ref(true),
  params: [
    {
      id: "tolerance",
      label: "White match tolerance",
      min: 0,
      max: 255,
      step: 1,
      suffix: "",
      value: floodTolerance,
    },
  ],
  apply: (canvas: HTMLCanvasElement) => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;
    const imageData = ctx.getImageData(0, 0, width, height);
    const data = imageData.data;
    const tolerance = floodTolerance.value;

    const isWhite = (idx: number): boolean => {
      const r = data[idx];
      const g = data[idx + 1];
      const b = data[idx + 2];
      return (
        255 - r <= tolerance && 255 - g <= tolerance && 255 - b <= tolerance
      );
    };

    const visited = new Uint8Array(width * height);
    // Queue of pixel indices (x + y*width). Seed with border pixels that are white,
    // simulating a 1px white margin: any real border pixel that's white is reachable
    // from "outside" by definition, since the virtual margin is always white.
    const queue: number[] = [];

    const tryEnqueue = (x: number, y: number) => {
      if (x < 0 || x >= width || y < 0 || y >= height) return;
      const pos = y * width + x;
      if (visited[pos]) return;
      const idx = pos * 4;
      if (!isWhite(idx)) return;
      visited[pos] = 1;
      queue.push(pos);
    };

    for (let x = 0; x < width; x++) {
      tryEnqueue(x, 0);
      tryEnqueue(x, height - 1);
    }
    for (let y = 0; y < height; y++) {
      tryEnqueue(0, y);
      tryEnqueue(width - 1, y);
    }

    // BFS, 4-connectivity
    let head = 0;
    while (head < queue.length) {
      const pos = queue[head++];
      const x = pos % width;
      const y = (pos / width) | 0;

      tryEnqueue(x + 1, y);
      tryEnqueue(x - 1, y);
      tryEnqueue(x, y + 1);
      tryEnqueue(x, y - 1);
    }

    // Replace visited pixels with pure white
    for (let pos = 0; pos < visited.length; pos++) {
      if (!visited[pos]) continue;
      const idx = pos * 4;
      data[idx] = 0;
      data[idx + 1] = 0;
      data[idx + 2] = 0;
    }

    ctx.putImageData(imageData, 0, 0);
  },
};

const speckBrightnessThreshold = ref(75);
const speckSizeThreshold = ref(2);

const speckRemovalFilter: FilterDef = {
  id: "speck-removal",
  name: "Speck Removal",
  description:
    "Removes small clusters of dark pixels (specks/noise) based on luminosity, regardless of color",
  enabled: ref(true),
  params: [
    {
      id: "brightness",
      label: "Brightness threshold",
      min: 0,
      max: 255,
      step: 1,
      suffix: "",
      value: speckBrightnessThreshold,
    },
    {
      id: "size",
      label: "Max speck size (pixels)",
      min: 1,
      max: 5,
      step: 1,
      suffix: "px",
      value: speckSizeThreshold,
    },
  ],
  apply: (canvas: HTMLCanvasElement) => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;
    const imageData = ctx.getImageData(0, 0, width, height);
    const data = imageData.data;
    const brightnessThreshold = speckBrightnessThreshold.value;
    const sizeThreshold = speckSizeThreshold.value;

    const luminosity = (idx: number): number => {
      // Standard luma weights; ignores hue/saturation entirely
      const r = data[idx];
      const g = data[idx + 1];
      const b = data[idx + 2];
      return 0.299 * r + 0.587 * g + 0.114 * b;
    };

    const isDark = (pos: number): boolean =>
      luminosity(pos * 4) > brightnessThreshold;

    const visited = new Uint8Array(width * height);
    const clusterBuffer: number[] = []; // reused scratch buffer for each component

    const collectCluster = (startPos: number) => {
      clusterBuffer.length = 0;
      clusterBuffer.push(startPos);
      visited[startPos] = 1;

      let head = 0;
      while (head < clusterBuffer.length) {
        const pos = clusterBuffer[head++];
        const x = pos % width;
        const y = (pos / width) | 0;

        // 4-connectivity neighbors
        if (x + 1 < width) tryVisit(x + 1, y);
        if (x - 1 >= 0) tryVisit(x - 1, y);
        if (y + 1 < height) tryVisit(x, y + 1);
        if (y - 1 >= 0) tryVisit(x, y - 1);
      }

      function tryVisit(nx: number, ny: number) {
        const npos = ny * width + nx;
        if (visited[npos]) return;
        if (!isDark(npos)) return;
        visited[npos] = 1;
        clusterBuffer.push(npos);
      }
    };

    for (let pos = 0; pos < width * height; pos++) {
      if (visited[pos]) continue;
      if (!isDark(pos)) {
        visited[pos] = 1; // mark bright pixels visited too, so we skip them forever
        continue;
      }

      collectCluster(pos);

      if (clusterBuffer.length <= sizeThreshold) {
        // Speck: whiten every pixel in this cluster
        for (const p of clusterBuffer) {
          const idx = p * 4;
          data[idx] = 0;
          data[idx + 1] = 0;
          data[idx + 2] = 0;
        }
      }
      // else: leave cluster as-is, it's a real stroke/character piece
    }

    ctx.putImageData(imageData, 0, 0);
  },
};

const cropTolerance = ref(0);

const autoCropBlackFilter: FilterDef = {
  id: "auto-crop-black",
  name: "Auto Crop (Black)",
  description: "Crops the image to the first non-black pixel from each edge.",
  enabled: ref(true),
  params: [
    {
      id: "tolerance",
      label: "Black Tolerance",
      min: 0,
      max: 255,
      step: 1,
      suffix: "",
      value: cropTolerance,
    },
  ],
  apply: (canvas: HTMLCanvasElement) => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;
    const imageData = ctx.getImageData(0, 0, width, height);
    const data = imageData.data;
    const tolerance = cropTolerance.value;

    let minX = width;
    let minY = height;
    let maxX = -1;
    let maxY = -1;

    // Single-pass scan to find the bounding box of non-black pixels
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const i = (y * width + x) * 4;
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        const a = data[i + 3]; // Alpha channel

        // A pixel is "not black" if it is visible (alpha > 0) AND its RGB values exceed the tolerance
        if (a > 0 && (r > tolerance || g > tolerance || b > tolerance)) {
          if (x < minX) minX = x;
          if (x > maxX) maxX = x;
          if (y < minY) minY = y;
          if (y > maxY) maxY = y;
        }
      }
    }

    // Check if the image was completely black (or completely empty)
    if (minX > maxX || minY > maxY) {
      // Optional: Clear the canvas completely or leave it alone.
      // Here we set it to a 1x1 transparent pixel so it doesn't crash.
      canvas.width = 1;
      canvas.height = 1;
      return;
    }

    const cropWidth = maxX - minX + 1;
    const cropHeight = maxY - minY + 1;

    // If a crop is actually needed, resize the canvas and draw the cropped region
    if (cropWidth !== width || cropHeight !== height) {
      // Grab only the cropped region from the context
      const croppedData = ctx.getImageData(minX, minY, cropWidth, cropHeight);

      // Resizing the canvas clears its current context/state
      canvas.width = cropWidth;
      canvas.height = cropHeight;

      // Put the cropped data back into the top-left corner
      ctx.putImageData(croppedData, 0, 0);
    }
  },
};

// Add new filters here.
const filters: FilterDef[] = [
  // colorExtractionFilter,
  // referenceDiffFilter,

  // edgeEnhancementFilter,
  colorfulnessMaskFilter,
  thresholdFilter,
  backgroundFloodFillFilter,
  speckRemovalFilter,
  autoCropBlackFilter,
];

const upscaleFactor = ref(2);
const h_border = ref(12);
const v_border = ref(12);
type PadColor = "auto" | "white" | "black";
const padColor = ref<PadColor>("auto");

let engine: OCREngine | null = null;
let modelLoaded = false;

async function ensureEngine(): Promise<OCREngine> {
  if (!engine) {
    engine = await createOCREngine();
  }
  return engine;
}

async function ensureModelLoaded(ocr: OCREngine): Promise<void> {
  if (modelLoaded) {
    return;
  }
  const response = await fetch(props.modelUrl);
  const buffer = await response.arrayBuffer();
  ocr.loadModel(buffer);
  modelLoaded = true;
}

function drawToCanvas(
  source: ImageBitmap | HTMLCanvasElement,
  width: number,
  height: number,
  smooth = true,
): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) {
    throw new Error("Could not get 2D canvas context");
  }
  ctx.imageSmoothingEnabled = smooth;
  if (smooth) {
    ctx.imageSmoothingQuality = "high";
  }
  ctx.drawImage(source, 0, 0, width, height);
  return canvas;
}

function addBorder(
  canvas: HTMLCanvasElement,
  h_border: number,
  v_border: number,
  mode: PadColor,
): HTMLCanvasElement {
  if (h_border <= 0 && v_border <= 0) {
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
  bordered.width = canvas.width + h_border * 2;
  bordered.height = canvas.height + v_border * 2;
  const ctx = bordered.getContext("2d");
  if (!ctx) {
    throw new Error("Could not get 2D canvas context");
  }
  ctx.fillStyle = fillColor;
  ctx.fillRect(0, 0, bordered.width, bordered.height);
  ctx.drawImage(canvas, h_border, v_border);
  return bordered;
}

/**
 * Runs every enabled filter (in `filters` order) against a native-resolution
 * copy of `original`, then upscales and pads the result.
 *
 * Kept synchronous (no createImageBitmap/await) so it's safe to call from
 * both the upload handler and a reactive watcher without worrying about
 * watchEffect-style dependency tracking across an await boundary.
 */
function buildProcessedCanvas(original: ImageBitmap): HTMLCanvasElement {
  const native = drawToCanvas(original, original.width, original.height);

  // let anyFilterApplied = false;
  for (const filter of filters) {
    if (filter.enabled.value) {
      filter.apply(native);
      // anyFilterApplied = true;
    }
  }

  const upscaled = drawToCanvas(
    native,
    native.width * upscaleFactor.value,
    native.height * upscaleFactor.value,
    true,
  );

  return addBorder(upscaled, h_border.value, v_border.value, padColor.value);
}

// ---------------------------------------------------------------------------
// Multi-image queue
//
// Each uploaded image gets an `OcrItem`. `original` is the decoded
// ImageBitmap for that item — wrapped in `markRaw` so Vue doesn't try to
// proxy it (same reasoning as worker objects elsewhere in this codebase:
// it's an opaque native handle, not plain reactive data).
// ---------------------------------------------------------------------------

interface OcrItem {
  id: string;
  fileName: string;
  original: ImageBitmap;
  previewUrl: string;
  processedPreviewUrl: string | null;
  resultText: string | null;
  error: string | null;
  isProcessing: boolean;
}

const items = ref<OcrItem[]>([]);

let nextId = 0;
function makeId(): string {
  nextId += 1;
  return `img-${nextId}`;
}

/**
 * Rebuilds the processed image for a single item and runs it back through
 * OCR, updating that item's preview, result text, and error state in place.
 */
async function reprocessItem(item: OcrItem): Promise<void> {
  item.error = null;
  item.isProcessing = true;

  try {
    const ocr = await ensureEngine();

    const stage = buildProcessedCanvas(item.original);
    item.processedPreviewUrl = stage.toDataURL();
    const recognitionImage = await createImageBitmap(stage);

    await ensureModelLoaded(ocr);

    ocr.setVariable("tessedit_pageseg_mode", pageSegMode.value);
    ocr.setVariable("tessedit_char_whitelist", charWhitelist.value);

    ocr.loadImage(recognitionImage);
    item.resultText = ocr.getText();
  } catch (err) {
    item.error = err instanceof Error ? err.message : String(err);
  } finally {
    item.isProcessing = false;
  }
}

let reprocessDebounceHandle: ReturnType<typeof setTimeout> | null = null;

/**
 * Reprocesses every item currently in the queue. Run sequentially (rather
 * than Promise.all) since all items share a single OCREngine instance.
 */
async function reprocessAll(): Promise<void> {
  for (const item of items.value) {
    await reprocessItem(item);
  }
}

function scheduleReprocessAll(): void {
  if (reprocessDebounceHandle !== null) {
    clearTimeout(reprocessDebounceHandle);
  }
  // Short debounce so dragging a slider doesn't re-run preprocessing (and
  // a full OCR pass over every image) on every intermediate tick.
  reprocessDebounceHandle = setTimeout(() => {
    reprocessDebounceHandle = null;
    void reprocessAll();
  }, 50);
}

watch(
  [
    ...filters.flatMap((filter) => [
      filter.enabled,
      ...filter.params.map((param) => param.value),
    ]),
    upscaleFactor,
    h_border,
    v_border,
    padColor,
    charWhitelist,
    pageSegMode,
  ],
  scheduleReprocessAll,
);

async function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const files = input.files ? Array.from(input.files) : [];
  input.value = "";
  if (files.length === 0) {
    return;
  }

  for (const file of files) {
    const original = markRaw(await createImageBitmap(file));
    const item: OcrItem = {
      id: makeId(),
      fileName: file.name,
      original,
      previewUrl: URL.createObjectURL(file),
      processedPreviewUrl: null,
      resultText: null,
      error: null,
      isProcessing: false,
    };
    items.value.push(item);
    await reprocessItem(item);
  }
}

function removeItem(id: string): void {
  const index = items.value.findIndex((item) => item.id === id);
  if (index === -1) {
    return;
  }
  const [item] = items.value.splice(index, 1);
  URL.revokeObjectURL(item.previewUrl);
  item.original.close();
}

onBeforeUnmount(() => {
  engine?.destroy();
  if (reprocessDebounceHandle !== null) {
    clearTimeout(reprocessDebounceHandle);
  }
  for (const item of items.value) {
    URL.revokeObjectURL(item.previewUrl);
    item.original.close();
  }
});
</script>

<template>
  <div class="mx-auto flex max-w-4xl flex-col gap-4 bg-white p-6">
    <div>
      <h1 class="text-lg font-semibold text-neutral-900">OCR demo</h1>
      <p class="text-sm text-neutral-500">
        Upload images to run them through tesseract-wasm.
      </p>
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Page segmentation mode
        <select
          v-model="pageSegMode"
          class="rounded-none border border-neutral-300 px-2 py-1.5 text-sm text-neutral-800"
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
        />
      </label>
    </div>

    <!-- One card per filter — add to the `filters` array in <script> and it
         shows up here automatically. -->
    <div
      v-for="filter in filters"
      :key="filter.id"
      class="flex flex-col gap-3 rounded-none border border-neutral-200 px-4 py-3"
    >
      <label
        class="flex items-center gap-2 text-sm font-medium text-neutral-700"
      >
        <input v-model="filter.enabled.value" type="checkbox" />
        {{ filter.name }}
      </label>
      <p v-if="filter.description" class="text-xs text-neutral-400">
        {{ filter.description }}
      </p>

      <div
        v-if="filter.params.length"
        class="grid grid-cols-1 gap-3"
        :class="filter.params.length > 1 ? 'sm:grid-cols-3' : 'sm:grid-cols-1'"
      >
        <label
          v-for="param in filter.params"
          :key="param.id"
          class="flex flex-col gap-1 text-sm text-neutral-600"
        >
          {{ param.label }}: {{ param.value.value }}{{ param.suffix ?? "" }}
          <input
            v-model.number="param.value.value"
            type="range"
            :min="param.min"
            :max="param.max"
            :step="param.step ?? 1"
            :disabled="!filter.enabled.value"
          />
        </label>
      </div>
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
        />
      </label>

      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Horizontal Border: {{ h_border }}px
        <input
          v-model.number="h_border"
          type="range"
          min="0"
          max="40"
          step="1"
        />
      </label>
      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Vertical Border: {{ v_border }}px
        <input
          v-model.number="v_border"
          type="range"
          min="0"
          max="40"
          step="1"
        />
      </label>

      <label class="flex flex-col gap-1 text-sm text-neutral-600">
        Border color
        <select
          v-model="padColor"
          class="rounded-none border border-neutral-300 px-2 py-1.5 text-sm text-neutral-800"
          :disabled="h_border === 0 && v_border === 0"
        >
          <option value="auto">Auto (match corner pixel)</option>
          <option value="white">White</option>
          <option value="black">Black</option>
        </select>
      </label>
    </div>

    <!-- Results table -->
    <div
      v-if="items.length"
      class="overflow-x-auto rounded-none border border-neutral-200"
    >
      <table class="w-full border-collapse text-sm">
        <thead>
          <tr class="border-b border-neutral-200 bg-neutral-50 text-left">
            <th class="px-3 py-2 font-medium text-neutral-600">Input</th>
            <th class="px-3 py-2 font-medium text-neutral-600">Sent to OCR</th>
            <th class="px-3 py-2 font-medium text-neutral-600">Result</th>
            <th class="px-3 py-2"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in items"
            :key="item.id"
            class="border-b border-neutral-100 align-top last:border-b-0"
          >
            <td class="px-3 py-2">
              <img
                :src="item.previewUrl"
                :alt="`${item.fileName} preview`"
                class="max-h-32 w-fit max-w-40 rounded-none border border-neutral-200 object-contain"
              />
              <div class="mt-1 max-w-40 truncate text-xs text-neutral-400">
                {{ item.fileName }}
              </div>
            </td>
            <td class="px-3 py-2">
              <img
                v-if="item.processedPreviewUrl"
                :src="item.processedPreviewUrl"
                :alt="`${item.fileName} preprocessed`"
                class="max-h-32 w-fit max-w-[10rem] rounded-none border border-neutral-200 object-contain"
              />
              <span v-else class="text-xs text-neutral-400">—</span>
            </td>
            <td class="px-3 py-2">
              <div v-if="item.isProcessing" class="text-xs text-neutral-500">
                Running OCR…
              </div>
              <div
                v-else-if="item.error"
                class="rounded-none border border-red-200 bg-red-50 px-2 py-1 text-xs text-red-700"
              >
                {{ item.error }}
              </div>
              <pre
                v-else
                class="max-w-xs font-mono text-xs whitespace-pre-wrap text-neutral-800"
                >{{ item.resultText || "(no text detected)" }}</pre
              >
            </td>
            <td class="px-3 py-2 text-right">
              <button
                type="button"
                class="rounded-none border border-neutral-300 px-2 py-1 text-xs text-neutral-600 hover:border-red-300 hover:text-red-700"
                @click="removeItem(item.id)"
              >
                Remove
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <label
      class="flex cursor-pointer flex-col items-center gap-2 rounded-lg border border-dashed border-neutral-300 px-6 py-8 text-sm text-neutral-500 transition hover:border-neutral-400 hover:bg-neutral-50"
    >
      <span>Click to choose one or more images</span>
      <input
        type="file"
        accept="image/*"
        multiple
        class="hidden"
        @change="handleFileChange"
      />
    </label>
  </div>
</template>
