<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { SlotAddress } from "./LoadStorage";
import {
  chests,
  edits,
  retry_slot,
  set_edit,
  shown_slot,
  slot_images,
  slot_key,
  slots,
} from "./ScanStore";
import SlotCell from "./SlotCell.vue";

const props = defineProps<{ address: SlotAddress | null; items: string[] }>();

const WINDOW_NAMES = {
  Roster: "Roster storage",
  CharStorage: "Character storage",
  CharInventory: "Inventory",
};
const STATUS_NAMES = {
  Pending: "Reading",
  Good: "Good",
  NeedHover: "Needs hover",
  NeedTradability: "Needs hover",
  Error: "Error",
  Irrelevant: "Irrelevant",
};
const TRADABILITY = [
  ["Tradable", "Tradable"],
  ["RosterBound", "Roster bound"],
  ["CharBound", "Character bound"],
];

const key = computed(() => (props.address ? slot_key(props.address) : ""));
const read = computed(() => slots.value.get(key.value));
const slot = computed(() => shown_slot(read.value));
// no tooltip said so; it is what was assumed for the window
const assumed = computed(() => slot.value !== read.value);
const edit = computed(() => edits.value[key.value]);

const item = computed(() =>
  edit.value ? edit.value.item : (slot.value?.icon_name_score?.[0] ?? null),
);
const status = computed(() => {
  if (edit.value) return "Edited by hand";
  if (!slot.value) return "Not seen yet";
  return STATUS_NAMES[slot.value.status];
});

const status_color = computed(() =>
  edit.value
    ? "var(--series-blue)"
    : {
        Good: "var(--achieved)",
        NeedHover: "var(--series-fusion)",
        NeedTradability: "var(--series-fusion)",
        Error: "var(--warning)",
      }[slot.value?.status],
);

// The chest read in this very slot, else every chest with this icon read in its column: a
// pushed-up tooltip only tells the column.
const slot_chests = computed(() => {
  if (!props.address || !item.value) return [];
  const [, column] = props.address.pos_in_inv;
  const exact = chests.value.filter(
    (chest) => chest.slot && slot_key(chest.slot) === key.value,
  );
  if (exact.length) return exact;
  return chests.value.filter(
    (chest) =>
      chest.icon === item.value &&
      chest.column?.[0] === props.address.inventory_type &&
      chest.column[1] === props.address.page_num &&
      chest.column[2] === column,
  );
});

// what is being typed, started from what the slot shows whenever another slot is picked
const draft = reactive({
  item: "",
  amount: null as number | null,
  tradability: "",
});
watch(
  // also when the slot is saved, retried or first read
  [key, edit, () => !slot.value, assumed],
  () => {
    draft.item = item.value ?? "";
    draft.amount = edit.value ? edit.value.amount : (slot.value?.value ?? null);
    draft.tradability =
      (edit.value ? edit.value.tradability : slot.value?.tradability) ?? "";
  },
  { immediate: true },
);

function save() {
  const amount = Number(draft.amount);
  set_edit({
    address: props.address,
    item: draft.item || null,
    amount:
      draft.amount === null || (draft.amount as any) === ""
        ? null
        : Math.max(0, Math.round(amount)),
    tradability: draft.tradability || null,
  });
}

// keeps what was read and stops the scanner changing it
function accept() {
  set_edit({
    address: props.address,
    item: item.value,
    amount: slot.value?.value ?? null,
    tradability: slot.value?.tradability ?? null,
  });
}
</script>

<template>
  <div class="card-shell card-body flex min-h-32 flex-wrap gap-6 text-sm">
    <span v-if="!address" class="text-(--text-muted)">
      Hover a slot to see what was read. Click one to keep it selected, and again to release it.
    </span>
    <template v-else>
      <div class="flex gap-3">
        <SlotCell
          class="h-16 w-16 cursor-default!"
          :info="slot"
          :edit="edit"
          :image="slot_images.get(key)"
          :active="false"
        />
        <div class="flex flex-col">
          <span class="text-(--text-muted)">
            {{ WINDOW_NAMES[address.inventory_type] }} page
            {{ address.page_num + 1 }}, row {{ address.pos_in_inv[0] + 1 }},
            column {{ address.pos_in_inv[1] + 1 }}
          </span>
          <span class="text-base text-(--text-bright)">
            {{ item ?? (slot || edit ? "No known item" : "Not seen yet") }}
          </span>
          <span :style="{ color: status_color }">{{ status }}</span>
        </div>
      </div>

      <div
        v-if="!edit && slot?.reason"
        class="max-w-80 border-l pl-2"
        :style="{ borderColor: status_color }"
      >
        <div class="text-(--text-muted)">Reason</div>
        {{ slot.reason }}
      </div>

      <div
        class="grid grid-cols-[auto_auto] content-start gap-x-3 text-(--text-muted)"
      >
        <span>Amount</span>
        <span class="text-(--text-main)">
          {{ (edit ? edit.amount : slot?.value) ?? "-" }}
        </span>
        <span>Number on icon</span>
        <span class="text-(--text-main)">{{ slot?.amount ?? "-" }}</span>
        <span>Tooltip amount</span>
        <span class="text-(--text-main)">
          {{ slot?.tooltip_amount ?? "-" }}
        </span>
        <span>Tradability</span>
        <span class="text-(--text-main)">
          {{ (edit ? edit.tradability : slot?.tradability) ?? "-" }}
          {{ !edit && assumed ? "(assumed)" : "" }}
        </span>
      </div>

      <div v-if="slot_chests.length" class="flex flex-col gap-1">
        <div v-for="(chest, index) in slot_chests" :key="index">
          <span class="text-(--text-bright)">
            {{ chest.title ?? chest.last_read_title }}
          </span>
          x{{ chest.amount ?? "?" }}, {{ chest.kind }},
          {{ chest.tradability ?? "tradability unknown" }}
          <div
            v-for="(content, i) in chest.contents"
            :key="i"
            class="pl-3 text-(--text-muted)"
          >
            {{ content.item }} x{{ content.amount }}
            {{ content.bound ? "(Bound)" : "" }}
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-1">
        <div class="flex flex-wrap items-center gap-2">
          <select v-model="draft.item" class="selector" aria-label="Item">
            <option value="">Irrelevant</option>
            <option v-for="name in items" :key="name">{{ name }}</option>
          </select>
          <input
            v-model="draft.amount"
            type="number"
            min="0"
            class="generic-input w-24 pl-1"
            aria-label="Amount"
            placeholder="Amount"
          />
          <select
            v-model="draft.tradability"
            class="selector"
            aria-label="Tradability"
          >
            <option value="">Unknown</option>
            <option v-for="[value, label] in TRADABILITY" :value="value">
              {{ label }}
            </option>
          </select>
        </div>
        <div class="flex gap-2">
          <button class="generic-button" @click="save">Save edit</button>
          <button
            v-if="!edit && slot?.status === 'Error'"
            class="generic-button"
            @click="accept"
          >
            Okay
          </button>
          <button
            v-if="slot || edit"
            class="generic-button"
            @click="retry_slot(address)"
          >
            Retry
          </button>
        </div>
      </div>
    </template>
  </div>
</template>
