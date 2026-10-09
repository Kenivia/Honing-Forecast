<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { SlotAddress } from "./LoadStorage";
import {
  chests_for,
  edits,
  retry_slot,
  set_edit,
  shown_slot,
  slot_images,
  slot_key,
  slot_place,
  slots,
} from "./ScanStore";
import { item_name, SHARED_ICONS } from "./Manifest";
import SlotCell from "./SlotCell.vue";

const props = defineProps<{ address: SlotAddress | null; items: string[] }>();

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
const TRADABILITY_NAMES = Object.fromEntries(TRADABILITY);

const key = computed(() => (props.address ? slot_key(props.address) : ""));
const read = computed(() => slots.value.get(key.value));
const slot = computed(() => shown_slot(read.value));
// no tooltip said so; it is what was assumed for the window
const assumed = computed(() => slot.value !== read.value);
const edit = computed(() => edits.value[key.value]);

// for an icon several materials share, the one its tooltip said it is
const item = computed(() => {
  if (edit.value) return edit.value.item;
  const icon = slot.value?.icon_name_score?.[0] ?? null;
  return (SHARED_ICONS[icon] && slot.value.label) || icon;
});
const status = computed(() => {
  if (edit.value) return "Edited by hand";
  if (!slot.value) return "Not seen yet";
  return STATUS_NAMES[slot.value.status];
});

const status_color = computed(() =>
  edit.value
    ? "var(--average)"
    : {
        Good: "var(--achieved)",
        NeedHover: "var(--series-fusion)",
        NeedTradability: "var(--series-fusion)",
        Error: "var(--warning)",
      }[slot.value?.status],
);

const tradability = computed(
  () => (edit.value ? edit.value.tradability : slot.value?.tradability) ?? null,
);

const slot_chests = computed(() =>
  props.address ? chests_for(props.address, item.value) : [],
);

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
    draft.tradability = tradability.value ?? "";
  },
  { immediate: true },
);

// an item needs its amount and tradability; an irrelevant slot needs neither
const no_amount = computed(
  () => !!draft.item && (draft.amount === null || (draft.amount as any) === ""),
);
const no_tradability = computed(() => !!draft.item && !draft.tradability);
const incomplete = computed(() => no_amount.value || no_tradability.value);
const missing_style = (missing: boolean) =>
  missing ? { borderColor: "var(--warning)" } : {};

function save() {
  set_edit({
    address: props.address,
    item: draft.item || null,
    amount: draft.item ? Math.max(0, Math.round(Number(draft.amount))) : null,
    tradability: draft.item ? draft.tradability : null,
  });
}

// what was read is itself short of an amount or a tradability, so it cannot be kept as it is
const read_incomplete = computed(
  () =>
    !!item.value &&
    (slot.value?.value == null || slot.value?.tradability == null),
);

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
  <div
    class="card-shell card-body flex min-h-32 flex-wrap gap-x-8 gap-y-4 text-sm"
  >
    <span v-if="!address" class="text-(--text-muted)">
      Hover a slot to see what was read. Click one to keep it selected, and
      again to release it.
    </span>
    <template v-else>
      <div class="flex max-w-80 flex-col gap-2">
        <div class="flex gap-3">
          <SlotCell
            class="h-16 w-16 shrink-0 cursor-default!"
            :info="slot"
            :edit="edit"
            :image="slot_images.get(key)"
            :active="false"
          />
          <div class="flex flex-col">
            <span class="text-base text-(--text-bright)">
              {{
                item_name(item) ??
                (slot || edit ? "No known item" : "Not seen yet")
              }}
            </span>
            <span :style="{ color: status_color }">{{ status }}</span>
            <span class="text-(--text-muted)">
              {{ slot_place(address) }}
            </span>
          </div>
        </div>
        <div
          v-if="!edit && slot?.reason"
          class="border-l pl-2"
          :style="{ borderColor: status_color }"
        >
          {{ slot.reason }}
        </div>
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-(--text-bright)">
          {{ edit ? "Set by hand" : "What was read" }}
        </span>
        <div class="grid grid-cols-[auto_auto] gap-x-3 text-(--text-muted)">
          <span>Amount</span>
          <span class="text-(--text-main)">
            {{ (edit ? edit.amount : slot?.value) ?? "-" }}
          </span>
          <span>Tradability</span>
          <span class="text-(--text-main)">
            {{ TRADABILITY_NAMES[tradability] ?? "-" }}
            {{ !edit && assumed ? "(assumed)" : "" }}
          </span>
          <template v-if="!edit">
            <span>Number on icon</span>
            <span class="text-(--text-main)">{{ slot?.amount ?? "-" }}</span>
            <span>Tooltip amount</span>
            <span class="text-(--text-main)">
              {{ slot?.tooltip_amount ?? "-" }}
            </span>
          </template>
        </div>
      </div>

      <div v-if="slot_chests.length" class="flex flex-col gap-1">
        <span class="text-(--text-bright)">Chest contents</span>
        <div v-for="(chest, index) in slot_chests" :key="index">
          {{ chest.title }}
          <span class="text-(--text-muted)">
            x{{ chest.amount ?? "?" }}, {{ chest.kind }},
            {{ TRADABILITY_NAMES[chest.tradability] ?? "tradability unknown" }}
          </span>
          <div
            v-for="(content, i) in chest.contents"
            :key="i"
            class="pl-3 text-(--text-muted)"
          >
            {{ content.item }} x{{ content.amount }}
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-(--text-bright)">Correct it</span>
        <div
          class="grid grid-cols-[auto_auto] items-center gap-x-3 gap-y-1 text-(--text-muted)"
        >
          <span>Item</span>
          <select v-model="draft.item" class="selector" aria-label="Item">
            <option value="">Irrelevant</option>
            <option v-for="name in items" :key="name" :value="name">
              {{ item_name(name) }}
            </option>
          </select>
          <template v-if="draft.item">
            <span>Amount</span>
            <input
              v-model="draft.amount"
              type="number"
              min="0"
              class="generic-input h-7! w-24 pl-1"
              :style="missing_style(no_amount)"
              aria-label="Amount"
            />
            <span>Tradability</span>
            <select
              v-model="draft.tradability"
              class="selector"
              :style="missing_style(no_tradability)"
              aria-label="Tradability"
            >
              <option value="" disabled>Choose</option>
              <option
                v-for="[value, label] in TRADABILITY"
                :value="value"
                :key="label"
              >
                {{ label }}
              </option>
            </select>
          </template>
        </div>
        <span v-if="incomplete" class="text-(--warning)">
          {{
            no_amount && no_tradability
              ? "Amount and tradability are needed."
              : no_amount
                ? "The amount is needed."
                : "The tradability is needed."
          }}
        </span>
        <div class="flex gap-2">
          <button
            class="generic-button"
            :class="{ 'cursor-not-allowed! opacity-50': incomplete }"
            :disabled="incomplete"
            @click="save"
          >
            Save edit
          </button>
          <button
            v-if="!edit && slot?.status === 'Error'"
            class="generic-button"
            :class="{ 'cursor-not-allowed! opacity-50': read_incomplete }"
            :disabled="read_incomplete"
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
