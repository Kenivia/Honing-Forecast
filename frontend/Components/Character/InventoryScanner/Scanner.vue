<script setup lang="ts">
import { computed, ref } from "vue";
import { getScannerConfig, OneIconConfig } from "./LoadStorage.js";
import { process_result } from "./ScanStore";
import SlotGrid from "./SlotGrid.vue";
import Stream from "./Stream.vue";

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

const status = ref<"idle" | "capturing">("idle");
// what a slot can be set to by hand
const items = computed(() =>
  (config.value ?? [])
    .filter((icon) => icon.tag === "Icon")
    .map((icon) => icon.name)
    .sort(),
);

// The debug UI below is switched off for now, script and template both.

// import { nextTick, ref, shallowRef } from "vue";
// import { useThrottleFn } from "@vueuse/core";
// import { zipSync } from "fflate";
// import {
//   Chest,
//   getScannerConfig,
//   Hover,
//   OneIconConfig,
//   ScaledPosition,
//   ScanResult,
//   SlotAddress,
// } from "./LoadStorage.js";
// import Stream from "./Stream.vue";
// import { draw_icon } from "./ScannerUIutils.js";
// import IconDisplay from "./IconDisplay.vue";
// import DemoOCR from "./DemoOCR.vue";
// import DemoColorfilter from "./DemoColorfilter.vue";

// const config = ref<OneIconConfig[] | null>(null);
// getScannerConfig().then((data) => (config.value = data));

// const status = ref<"idle" | "capturing">("idle");
// const boxes = ref<ScaledPosition[]>([]);

// interface DebugRow {
//   icon_name: string;
//   position: ScaledPosition;
//   confidence: number;
//   brightness: number;
//   debug_icons: OneIconConfig[];
// }

// const debug_table = shallowRef<DebugRow[]>([]);

// const debugging = ref(true);
// const hover = ref<Hover | null>(null);

// interface FoundIconRow {
//   key: string;
//   name: string;
//   icon: OneIconConfig;
//   confidence: number;
//   observed_number: OneIconConfig;
//   processed_number: OneIconConfig;
//   amount: string | null;
//   tooltip_amount: string | null;
//   tradability: string | null;
// }

// const found_icons = shallowRef<FoundIconRow[]>([]);
// const chests = shallowRef<Chest[]>([]);

// function chest_location(chest: Chest) {
//   if (chest.slot) {
//     const [row, column] = chest.slot.pos_in_inv;
//     return `${chest.slot.inventory_type} page ${chest.slot.page_num + 1}, row ${row + 1}, column ${column + 1}`;
//   }
//   if (chest.column) {
//     const [inventory, page, column] = chest.column;
//     return `${inventory} page ${page + 1}, column ${column + 1}`;
//   }
//   return "";
// }

// function canvas_to_blob(canvas: HTMLCanvasElement): Promise<Blob> {
//   return new Promise((resolve, reject) => {
//     canvas.toBlob((blob) => {
//       if (blob) {
//         resolve(blob);
//       } else {
//         reject(new Error("Could not convert icon canvas to PNG"));
//       }
//     }, "image/png");
//   });
// }

// async function download_debug_icons() {
//   downloading_debug_icons.value = true;

//   try {
//     const files: Record<string, Uint8Array> = {};
//     const used_names = new Set<string>();

//     for (const row of debug_table.value) {
//       for (const [index, icon] of row.debug_icons.entries()) {
//         const canvas = document.createElement("canvas");
//         canvas.width = icon.offset.width;
//         canvas.height = icon.offset.height;
//         draw_icon(canvas, icon.data, icon.offset.width, icon.offset.height);

//         const blob = await canvas_to_blob(canvas);
//         const safe_name = `${row.icon_name}-${index + 1}`.replace(
//           /[^a-zA-Z0-9._-]/g,
//           "_",
//         );
//         let filename = `${safe_name}.png`;
//         let suffix = 2;
//         while (used_names.has(filename)) {
//           filename = `${safe_name}-${suffix}.png`;
//           suffix += 1;
//         }
//         used_names.add(filename);
//         files[filename] = new Uint8Array(await blob.arrayBuffer());
//       }
//     }

//     const archive = zipSync(files);
//     const url = URL.createObjectURL(
//       new Blob([archive], { type: "application/zip" }),
//     );
//     const link = document.createElement("a");
//     link.href = url;
//     link.download = "debug-icons.zip";
//     document.body.appendChild(link);
//     link.click();
//     document.body.removeChild(link);
//     URL.revokeObjectURL(url);
//   } finally {
//     downloading_debug_icons.value = false;
//   }
// }

// <button
//   class="generic-button"
//   :disabled="downloading_debug_icons"
//   @click="download_debug_icons"
// >
//   {{
//     downloading_debug_icons ? "Creating ZIP..." : "Download debug icons"
//   }}
// </button>

// Images only arrive when they change, so they are kept here. Reusing the same objects
// also lets Vue skip the canvases that did not change.
// const slot_images = new Map<string, OneIconConfig[]>();
// const debug_rows = new Map<string, DebugRow>();
// let latest: ScanResult | null = null;
// let latest_chests: Chest[] = [];
// const render_timings: number[] = ((globalThis as any).__scan_renders ??= []);

// const slot_key = (address: SlotAddress) =>
//   `${address.inventory_type} ${address.page_num} ${address.pos_in_inv}`;

// called for every scan, so it only merges; the tables are updated a few times a second
// function process_result(result: ScanResult) {
//   if (result.full) {
//     slot_images.clear();
//     debug_rows.clear();
//   }
//   for (const slot of result.slots) {
//     if (slot.images) slot_images.set(slot_key(slot.address), slot.images);
//   }
//   for (const [
//     icon_name,
//     position,
//     confidence,
//     brightness,
//     icons,
//   ] of result.debug) {
//     debug_rows.set(icon_name, {
//       icon_name,
//       position,
//       confidence,
//       brightness,
//       debug_icons: icons,
//     });
//   }
//   latest_chests = result.chests ?? latest_chests;
//   latest = result;
//   render();
// }

// const render = useThrottleFn(
//   () => {
//     const start = performance.now();
//     if (!debugging.value) debug_rows.clear();
//     debug_table.value = [...debug_rows.values()].sort((a, b) =>
//       a.icon_name.localeCompare(b.icon_name),
//     );
//     boxes.value = debug_table.value.map((row) => row.position);

//     found_icons.value = latest.slots
//       .map((slot) => {
//         const key = slot_key(slot.address);
//         const [icon, observed_number, processed_number] =
//           slot_images.get(key) ?? [];
//         return {
//           key,
//           name: slot.icon_name_score[0],
//           icon,
//           confidence: slot.icon_name_score[1],
//           observed_number,
//           processed_number,
//           amount: slot.amount,
//           tooltip_amount: slot.tooltip_amount,
//           tradability: slot.tradability,
//         };
//       })
//       .sort(
//         (a, b) => a.name.localeCompare(b.name) || a.key.localeCompare(b.key),
//       );
//     chests.value = latest_chests;
//     hover.value = latest.hover;
//     nextTick(() => {
//       render_timings.push(performance.now() - start);
//       if (render_timings.length > 5000) render_timings.shift();
//     });
//   },
//   250,
//   true,
// );
</script>

<template>
  <div v-if="config" class="flex w-full flex-col gap-3">
    <Stream
      :should_start_cropper="true"
      v-model:status="status"
      :debugging="false"
      :process_result="process_result"
    />
    <SlotGrid :items="items" />
  </div>
  <!--
  <div v-if="config">
    <Stream
      :should_start_cropper="true"
      v-model:status="status"
      :boxes="boxes"
      :debugging="debugging"
      :process_result="process_result"
    />
    <button class="generic-button" @click="debugging = !debugging">
      {{ debugging ? "Hide" : "Show" }} debug info
    </button>
    <div v-if="debugging">
      <table>
        <thead>
          <tr>
            <th>Icon name</th>
            <th>X</th>
            <th>Y</th>
            <th>Confidence</th>
            <th>Brightness</th>
            <th>icon</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in debug_table" :key="row.icon_name">
            <td>{{ row.icon_name }}</td>
            <td>{{ row.position.top_left[0].toFixed(1) }}</td>
            <td>{{ row.position.top_left[1].toFixed(1) }}</td>
            <td>{{ row.confidence.toFixed(3) }}</td>
            <td>{{ row.brightness.toFixed(3) }}</td>
            <td class="flex flex-row">
              <IconDisplay
                :key="index"
                v-for="(icon, index) in row.debug_icons"
                :icon="icon"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="hover">
      Tooltip: {{ hover.title ?? hover.last_read_title }}, x{{ hover.amount }},
      {{ hover.tradability }}
    </div>
    <table>
      <thead>
        <tr>
          <th>Icon</th>
          <th>Name</th>
          <th>Tag</th>
          <th>X</th>
          <th>Y</th>
          <th>Confidience</th>
          <th>Raw number</th>
          <th>Processed</th>
          <th>OCR result</th>
          <th>Tooltip amount</th>
          <th>Tradability</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in found_icons" :key="row.key">
          <td>
            <IconDisplay :icon="row.icon" />
          </td>
          <td>{{ row.name }}</td>
          <td>{{ row.icon?.tag }}</td>
          <td>{{ row.icon?.offset.top_left[0].toFixed(1) }}</td>
          <td>{{ row.icon?.offset.top_left[1].toFixed(1) }}</td>
          <td>{{ row.confidence.toFixed(3) }}</td>
          <td>
            <IconDisplay :icon="row.observed_number" />
          </td>
          <td>
            <IconDisplay :icon="row.processed_number" />
          </td>
          <td>{{ row.amount }}</td>
          <td>{{ row.tooltip_amount }}</td>
          <td>{{ row.tradability }}</td>
        </tr>
      </tbody>
    </table>

    <div>Chests</div>
    <table>
      <thead>
        <tr>
          <th>Title read</th>
          <th>Kind</th>
          <th>Contents</th>
          <th>Tooltip amount</th>
          <th>Tradability</th>
          <th>Location</th>
          <th v-if="debugging">Rows read</th>
          <th v-if="debugging">Row crops</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(chest, index) in chests" :key="index">
          <td>{{ chest.last_read_title }}</td>
          <td>{{ chest.kind }}</td>
          <td>
            <div v-for="(content, i) in chest.contents" :key="i">
              {{ content.item }} x{{ content.amount }}
              {{ content.bound ? "(Bound)" : "" }}
            </div>
          </td>
          <td>{{ chest.amount }}</td>
          <td>{{ chest.tradability }}</td>
          <td>{{ chest_location(chest) }}</td>
          <td v-if="debugging">
            <div v-for="(row, i) in chest.rows" :key="i">
              {{ row.name_read }} | {{ row.count_read }} →
              {{ row.item ?? "ignored" }}
            </div>
          </td>
          <td v-if="debugging">
            <IconDisplay
              v-for="(row, i) in chest.rows"
              :key="i"
              :icon="row.crop"
            />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
   <DemoColorfilter />
  <DemoOCR /> 
  -->
</template>

<!--
table {
  border-collapse: separate;
  border-spacing: 32px 0;
}
-->
