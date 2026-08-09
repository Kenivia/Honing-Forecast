<script setup lang="ts">
import { OneIconConfig, ScaledPosition } from "../ScannerConfigStorage";
import { draw_icon } from "../ScannerUIutils";

const props = defineProps<{
  icon: OneIconConfig;
  canMoveUp: boolean;
  canMoveDown: boolean;
}>();

const emit = defineEmits<{
  "update:name": [value: string];
  "update:tag": [value: string];
  "update:position": [value: ScaledPosition];
  "import-position": [value: OneIconConfig];
  "move-up": [];
  "move-down": [];
  delete: [];
}>();

function on_name_change(event: Event) {
  const value = (event.target as HTMLInputElement).value.trim();
  if (value) emit("update:name", value);
}

function on_tag_change(event: Event) {
  const value = (event.target as HTMLInputElement).value.trim();
  emit("update:tag", value);
}

function on_top_left_x_change(event: Event) {
  const value = Number((event.target as HTMLInputElement).value) || 0;
  emit("update:position", {
    ...props.icon.offset,
    top_left: [value, props.icon.offset.top_left[1]],
  });
}

function on_top_left_y_change(event: Event) {
  const value = Number((event.target as HTMLInputElement).value) || 0;
  emit("update:position", {
    ...props.icon.offset,
    top_left: [props.icon.offset.top_left[0], value],
  });
}

function on_width_change(event: Event) {
  const value = Number((event.target as HTMLInputElement).value) || 0;
  emit("update:position", { ...props.icon.offset, width: value });
}

function on_height_change(event: Event) {
  const value = Number((event.target as HTMLInputElement).value) || 0;
  emit("update:position", { ...props.icon.offset, height: value });
}
</script>

<template>
  <div class="flex items-center gap-3 rounded-md bg-zinc-800/50 p-2">
    <canvas
      :ref="
        (el) =>
          draw_icon(
            el as HTMLCanvasElement,
            icon.data as ArrayLike<number>,
            icon.offset.width,
            icon.offset.height,
          )
      "
      :width="icon.offset.width"
      :height="icon.offset.height"
      class="rounded-none border border-zinc-600"
      style="image-rendering: pixelated"
    />

    <div class="flex flex-wrap items-end gap-3">
      <div class="flex flex-col items-start">
        <div>Name</div>
        <input
          :value="icon.name"
          type="text"
          placeholder="Name"
          class="w-full rounded bg-zinc-900 px-2 py-1 text-sm"
          @change="on_name_change"
        />
      </div>
      <div class="flex flex-col items-start">
        <div>Tag</div>
        <input
          :value="icon.tag"
          type="text"
          placeholder="Tag"
          class="w-full rounded bg-zinc-900 px-2 py-1 text-xs text-zinc-400"
          @change="on_tag_change"
        />
      </div>

      <div class="flex flex-wrap gap-2">
        <label class="flex flex-col gap-1 text-xs">
          Top left X
          <input
            :value="icon.offset.top_left[0]"
            type="number"
            class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
            @change="on_top_left_x_change"
          />
        </label>
        <label class="flex flex-col gap-1 text-xs">
          Top left Y
          <input
            :value="icon.offset.top_left[1]"
            type="number"
            class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
            @change="on_top_left_y_change"
          />
        </label>
        <label class="flex flex-col gap-1 text-xs">
          Width
          <input
            :value="icon.offset.width"
            type="number"
            min="0"
            class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
            @change="on_width_change"
          />
        </label>
        <label class="flex flex-col gap-1 text-xs">
          Height
          <input
            :value="icon.offset.height"
            type="number"
            min="0"
            class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
            @change="on_height_change"
          />
        </label>
      </div>
    </div>

    <div class="flex items-center gap-1">
      <button
        v-if="canMoveUp"
        class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
        title="Move up"
        @click="emit('move-up')"
      >
        ↑
      </button>
      <button
        v-if="canMoveDown"
        class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
        title="Move down"
        @click="emit('move-down')"
      >
        ↓
      </button>
      <button
        class="rounded-md bg-zinc-700 px-2 py-1 text-xs hover:bg-zinc-600"
        title="Import position"
        @click="emit('import-position', icon)"
      >
        Import pos
      </button>
      <button
        class="rounded-md bg-red-600 px-2 py-1 text-xs hover:bg-red-500"
        title="Delete"
        @click="emit('delete')"
      >
        Delete
      </button>
    </div>
  </div>
</template>
