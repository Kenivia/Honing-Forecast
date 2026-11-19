<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { SlotAddress } from "./LoadStorage";
import {
  chests_for,
  edits,
  retry_slot,
  set_edit,
  slot_images,
  slot_key,
  slots,
} from "./ScanStore";
import {
  bound_alike,
  CHEST_GROUPS,
  choice_of,
  choices,
  item_name,
  SHARED_ICONS,
  split_choice,
} from "./Manifest";
import { STATUS_NAMES } from "./ScannerUIutils";
import SlotCell from "./SlotCell.vue";

const props = defineProps<{ address: SlotAddress | null; items: string[] }>();

const TRADABILITY = [
  ["Tradable", "Tradable"],
  ["RosterBound", "Roster bound"],
  ["CharBound", "Character bound"],
];
const TRADABILITY_NAMES = Object.fromEntries(TRADABILITY);

const key = computed(() => (props.address ? slot_key(props.address) : ""));
const slot = computed(() => slots.value.get(key.value));
const edit = computed(() => edits.value[key.value]);

// for an icon several materials share, the one its tooltip said it is; or what the last export
// had it set to by hand
const read_item = computed(() => {
  const icon = slot.value?.icon_name_score?.[0] ?? null;
  if (icon && slot.value.item !== icon) return slot.value.item;
  return (SHARED_ICONS[icon] && slot.value.label) || icon;
});
const item = computed(() => (edit.value ? edit.value.item : read_item.value));

// the chests the slot was read as: its tooltip's, else what its icon can be
const read_variants = computed(() => {
  const read =
    props.address && read_item.value
      ? chests_for(props.address, read_item.value)
      : [];
  return read.length
    ? read.flatMap((chest) => chest.variants)
    : (slot.value?.variants ?? []);
});
// the item as a choice of the list, a chest's group where it is known
const read_choice = computed(
  () => read_item.value && choice_of(read_item.value, read_variants.value),
);
const choice = computed(() =>
  edit.value
    ? edit.value.item && choice_of(edit.value.item, edit.value.variants ?? [])
    : read_choice.value,
);

// the icons the slot scored close to, a shared icon as its materials
const candidates = computed(() => {
  const icon = slot.value?.icon_name_score?.[0];
  if (!icon) return [];
  return [icon, ...slot.value.alternatives]
    .flatMap((name) => SHARED_ICONS[name] ?? choices(name))
    .filter((name) => props.items.includes(name));
});
const likely_only = ref(true);
// the item picked stays offered, likely or not
const options = computed(() => {
  if (!likely_only.value || !candidates.value.length) return props.items;
  const picked = draft.item && !candidates.value.includes(draft.item);
  return picked ? [...candidates.value, draft.item] : candidates.value;
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
const read_tradability = computed(() =>
  slot.value?.tradability ? TRADABILITY_NAMES[slot.value.tradability] : "-",
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
// what the draft started from, so a change lights up the save button
const loaded = ref("");
const changed = computed(() => JSON.stringify(draft) !== loaded.value);
watch(
  // also when the slot is saved, retried or first read
  [key, edit, () => !slot.value],
  () => {
    draft.item = choice.value ?? "";
    draft.amount = edit.value ? edit.value.amount : (slot.value?.value ?? null);
    draft.tradability = tradability.value ?? group_bind() ?? "";
    loaded.value = JSON.stringify(draft);
  },
  { immediate: true },
);

// a chest group binds its chests one way, which then needs no picking
function group_bind() {
  const { group } = split_choice(draft.item);
  return group && props.address
    ? bound_alike(group, props.address.inventory_type)
    : null;
}
function pick_item() {
  draft.tradability = group_bind() ?? draft.tradability;
}

// an item needs its amount and tradability, a chest which of its icon's chests it is; an
// irrelevant slot needs none
const unsettled = computed(() => !!CHEST_GROUPS[draft.item]);
const no_amount = computed(
  () => !!draft.item && (draft.amount === null || (draft.amount as any) === ""),
);
const no_tradability = computed(
  () => !!draft.item && !unsettled.value && !draft.tradability,
);
const incomplete = computed(
  () => unsettled.value || no_amount.value || no_tradability.value,
);
const missing_style = (missing: boolean) =>
  missing ? { borderColor: "var(--warning)" } : {};

function save() {
  const { item, group } = split_choice(draft.item);
  set_edit({
    address: props.address,
    item: item || null,
    amount: item ? Math.max(0, Math.round(Number(draft.amount))) : null,
    tradability: item ? draft.tradability : null,
    variants: group ?? (item === read_item.value ? read_variants.value : []),
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
    variants: read_variants.value,
  });
}
</script>

<template>
  <div
    class="card-shell card-body flex min-h-32 flex-wrap gap-x-10 gap-y-4 text-sm"
  >
    <span v-if="!address" class="text-(--text-muted)">
      Hover a slot (on the website) to see what the scanner has read. Click one
      to keep it selected for editing.
    </span>
    <template v-else>
      <div class="flex w-60 flex-col gap-2">
        <div class="flex gap-3">
          <SlotCell
            class="h-16 w-16 shrink-0 cursor-default!"
            :info="slot"
            :edit="edit"
            :image="slot_images.get(key)"
            :active="false"
          />
          <div class="flex min-w-0 flex-col">
            <span class="text-base wrap-break-word text-(--text-bright)">
              {{
                item_name(choice) ??
                (slot || edit ? "No known item" : "Not seen yet")
              }}
            </span>
            <span :style="{ color: status_color }">{{ status }}</span>
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

      <div
        class="grid h-fit grid-cols-[auto_11rem_auto] items-center gap-x-4 gap-y-1"
        role="group"
        aria-label="Read and corrected"
      >
        <span></span>
        <span class="text-(--text-bright)">Read by scanner</span>
        <span class="text-(--text-bright)">Correct it</span>

        <span class="text-(--text-muted)">Item</span>
        <span class="wrap-break-word">{{ item_name(read_choice) ?? "-" }}</span>
        <div class="flex items-center gap-2">
          <select
            v-model="draft.item"
            class="selector w-56"
            :style="missing_style(unsettled)"
            aria-label="Item"
            @change="pick_item"
          >
            <option value="">Irrelevant</option>
            <!-- memoized: Vue rewrites every bound option value on each render, which resets an open dropdown's highlight -->
            <option
              v-for="name in options"
              :key="name"
              v-memo="[name]"
              :value="name"
            >
              {{ item_name(name) }}
            </option>
          </select>
          <label
            class="flex items-center gap-1 text-(--text-muted)"
            :class="{ 'cursor-not-allowed opacity-50': !candidates.length }"
          >
            <input
              v-model="likely_only"
              type="checkbox"
              :disabled="!candidates.length"
            />
            Likely candidates only
          </label>
        </div>

        <span class="text-(--text-muted)">Amount</span>
        <span>{{ slot?.value ?? "-" }}</span>
        <input
          v-if="draft.item"
          v-model="draft.amount"
          type="number"
          min="0"
          class="generic-input h-7! w-24 pl-1"
          :style="missing_style(no_amount)"
          aria-label="Amount"
        />
        <span v-else></span>

        <span class="text-(--text-muted)">Tradability</span>
        <span>{{ read_tradability }}</span>
        <select
          v-if="draft.item"
          v-model="draft.tradability"
          class="selector w-fit"
          :style="missing_style(no_tradability)"
          aria-label="Tradability"
        >
          <option value="" disabled>Choose</option>
          <option
            v-for="[value, label] in TRADABILITY"
            :value="value"
            :key="label"
            v-memo="[value]"
          >
            {{ label }}
          </option>
        </select>
        <span v-else></span>

        <span class="text-(--text-muted)">Number on icon</span>
        <span>{{ slot?.amount ?? "-" }}</span>
        <span></span>

        <span class="text-(--text-muted)">Tooltip amount</span>
        <span>{{ slot?.tooltip_amount ?? "-" }}</span>
        <span></span>

        <div class="col-start-3 flex flex-col gap-1 pt-1">
          <span v-if="incomplete" class="text-(--warning)">
            {{
              unsettled
                ? "Pick which chest this is."
                : no_amount && no_tradability
                  ? "Amount and tradability are needed."
                  : no_amount
                    ? "The amount is needed."
                    : "The tradability is needed."
            }}
          </span>
          <div class="flex gap-2">
            <button
              class="generic-button"
              :class="{
                'cursor-not-allowed! opacity-50': incomplete,
                'bg-(--accent-blue)! hover:bg-(--accent-blue-hover)!':
                  changed && !incomplete,
              }"
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
      </div>

      <div v-if="slot_chests.length" class="flex flex-col gap-1">
        <span class="text-(--text-bright)">Chest contents</span>
        <div v-for="(chest, index) in slot_chests" :key="index">
          {{ chest.title }}
          <span class="text-(--text-muted)">
            x{{ chest.amount ?? "?" }}, {{ chest.kind }},
            {{ TRADABILITY_NAMES[chest.tradability] ?? "tradability unknown"
            }}{{ chest.irrelevant ? ", irrelevant" : "" }}
          </span>
          <div
            v-for="(content, i) in chest.contents.slice(0, 10)"
            :key="i"
            class="pl-3 text-(--text-muted)"
          >
            {{ content.item }} x{{ content.amount }}
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
