<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { BAND_COLORS, BAND_NAMES } from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import { snapshot_of } from "@/Utils/Scans";
import { PRESET_CHOICES, signed_gold, worth_color } from "@/Utils/Choices";
import {
  raids_for,
  RAIDS,
  type auto_income,
  type Bag,
  type SourcesConfig,
} from "@/Utils/IncomeSources";
import { base_job, option_job } from "../Choices/ChoiceJobs";

// What the Auto income counts, and the settings of each source.
const props = defineProps<{
  config: SourcesConfig;
  result: ReturnType<typeof auto_income>;
  item_level: number;
}>();
const { roster_config, active_profile } = storeToRefs(useRosterStore());
const profile = active_profile;

const set = <K extends keyof SourcesConfig>(key: K, value: SourcesConfig[K]) =>
  (profile.value.income_sources[key] = value);
const set_number = <K extends keyof SourcesConfig>(
  key: K,
  input: HTMLInputElement,
) => {
  const amount = Math.max(0, parse_locale_float(input.value) || 0);
  set(key, amount as SourcesConfig[K]);
  input.value = String(amount);
};
// the raids are stored whole once edited
const raids = () =>
  (profile.value.income_sources.raids ??= JSON.parse(
    JSON.stringify(props.config.raids),
  ));

const raid_list = computed(() => raids_for(props.item_level));
function pick_raid(slot: number, group: string) {
  const list = raid_list.value.find((x) => x[0].group === group);
  const raid = list.findLast((x) => x.difficulty !== "Solo") ?? list.at(-1);
  raids()[slot] = {
    group,
    difficulty: raid.difficulty,
    bonus: raid.gates.map(() => false),
  };
}
const add_raid = () => {
  const taken = new Set(raids().map((pick) => pick.group));
  const next = raid_list.value.find((list) => !taken.has(list[0].group));
  if (next) {
    raids().push({ group: "", difficulty: "", bonus: [] });
    pick_raid(raids().length - 1, next[0].group);
  }
};
const raid_of = (group: string, difficulty: string) =>
  RAIDS.find((x) => x.group === group && x.difficulty === difficulty);

// the Choices page's worth of a gate's bonus chest, when it has been run
const now = computed(() =>
  snapshot_of(roster_config.value, profile.value.roster_id, "Manual"),
);
const base = computed(() => base_job(profile.value, now.value));
function advice(group: string, difficulty: string, gate: number) {
  const choice = PRESET_CHOICES.find(
    (x) =>
      x.group === group &&
      x.title === `Gate ${gate}` &&
      x.difficulty?.split(" / ").includes(difficulty),
  );
  if (!choice || base.value.metric === undefined) return undefined;
  const job = option_job(profile.value, now.value, base.value, choice, 1);
  return job.metric === undefined ? undefined : job.metric - base.value.metric;
}

const source = (key: string) =>
  props.result.sources.find((one) => one.key === key);
const set_pick = (key: string, at: number) =>
  (profile.value.income_sources.picks = {
    ...props.config.picks,
    [key]: at,
  });
function toggle_row(name: string) {
  const off = new Set(props.config.off);
  if (!off.delete(name)) off.add(name);
  set("off", [...off]);
}

// what a source gives in its bands, a week or over the grants
function gives(key: string) {
  const one = source(key);
  const bags = one.grants.length
    ? one.grants.reduce(
        (sum, grant) =>
          sum.map((bag, band) => {
            const out = { ...bag };
            for (const [label, amount] of Object.entries(grant.bags[band]))
              out[label] = (out[label] ?? 0) + amount;
            return out;
          }),
        [{}, {}, {}] as Bag[],
      )
    : one.weekly;
  return bags.flatMap((bag, band) =>
    Object.entries(bag)
      .filter(([, amount]) => Math.round(amount))
      .map(([label, amount]) => ({ label, amount, band })),
  );
}
const number = (amount: number) =>
  Math.round(amount).toLocaleString("en-US");
const date = (at: number) => new Date(at).toLocaleDateString();

const SIMPLE: {
  key: string;
  field?: keyof SourcesConfig;
  unit?: string;
  toggle?: keyof SourcesConfig;
}[] = [
  { key: "chaos", field: "chaos", unit: "runs a week" },
  { key: "guardian", field: "guardian", unit: "runs a week" },
  { key: "cube", field: "cube", unit: "tickets a week" },
  { key: "hourglass", toggle: "hourglass" },
  { key: "playtime", field: "playtime", unit: "days a week" },
  { key: "mission", field: "weekly_mission", unit: "claims a week" },
  { key: "guild", toggle: "guild" },
];
</script>

<template>
  <div
    class="card-shell card-body flex flex-col gap-3 text-sm"
    role="group"
    aria-label="Income sources"
  >
    <div class="flex flex-wrap items-center gap-x-4 gap-y-1">
      <span class="card-title">Auto income</span>
      <span class="text-(--text-muted)">
        At item level {{ Math.floor(item_level) }}. From the game files, and Tylobic's
        sheet for Kurzan Front, the Chaos Rift and guardians. Amounts are per
        week unless dated; gems are not counted.
      </span>
    </div>
    <span class="text-(--text-very-muted)">
      Not counted yet: Paradise Hell (work in progress), the login track.
    </span>

    <!-- raids -->
    <div class="flex flex-col gap-2">
      <div class="flex flex-wrap items-center gap-x-4 gap-y-1">
        <span class="text-(--text-bright)">Raids</span>
        <label class="flex items-center gap-1.5">
          <input
            :checked="config.gold_earner"
            type="checkbox"
            @change="set('gold_earner', !config.gold_earner)"
          />
          Gold-earning character
        </label>
        <button
          v-if="config.raids.length < 3"
          class="generic-button"
          @click="add_raid"
        >
          Add raid
        </button>
      </div>
      <div
        v-for="(pick, slot) in config.raids"
        :key="slot"
        class="flex flex-col gap-1 border-l border-(--border-muted) pl-3"
      >
        <div class="flex flex-wrap items-center gap-2">
          <select
            class="selector"
            :value="pick.group"
            :aria-label="`Raid ${slot + 1}`"
            @change="pick_raid(slot, ($event.target as HTMLSelectElement).value)"
          >
            <option
              v-for="list in raid_list"
              :key="list[0].group"
              :value="list[0].group"
            >
              {{ list[0].group }}
            </option>
          </select>
          <button
            v-for="raid in raid_list.find((x) => x[0].group === pick.group) ??
            []"
            :key="raid.difficulty"
            class="generic-button"
            :class="{ 'bg-(--gold-muted)!': pick.difficulty === raid.difficulty }"
            :title="`Item level ${raid.item_level}`"
            @click="
              raids()[slot] = {
                ...pick,
                difficulty: raid.difficulty,
                bonus: raid.gates.map(() => false),
              }
            "
          >
            {{ raid.difficulty }}
          </button>
          <button
            class="generic-button"
            :aria-label="`Remove raid ${slot + 1}`"
            @click="raids().splice(slot, 1)"
          >
            ✕
          </button>
        </div>
        <div class="flex flex-wrap gap-x-5 gap-y-1">
          <label
            v-for="(gate, at) in raid_of(pick.group, pick.difficulty)?.gates ??
            []"
            :key="gate.gate"
            class="flex items-center gap-1.5"
            :title="
              gate.bonus
                ? `Bonus chest for ${gate.bonus.cost.toLocaleString('en-US')} gold`
                : ''
            "
          >
            <input
              :checked="pick.bonus[at]"
              :disabled="!gate.bonus"
              type="checkbox"
              @change="
                raids()[slot].bonus = pick.bonus.map((x, i) =>
                  i === at ? !x : x,
                )
              "
            />
            Gate {{ gate.gate }} bonus chest
            <span class="text-(--gold)">
              {{ gate.bonus?.cost.toLocaleString("en-US") }}
            </span>
            <span
              v-if="advice(pick.group, pick.difficulty, gate.gate) !== undefined"
              :style="{
                color: worth_color(
                  false,
                  advice(pick.group, pick.difficulty, gate.gate),
                ),
              }"
            >
              {{ signed_gold(advice(pick.group, pick.difficulty, gate.gate)) }}
            </span>
            <RouterLink
              v-else
              class="link text-(--text-very-muted)"
              :to="{ name: 'char-choices', params: $route.params }"
            >
              worth: see Choices
            </RouterLink>
          </label>
        </div>
        <div class="flex flex-wrap items-center gap-x-3">
          <span
            v-for="{ label, amount, band } in gives(`raid ${slot}`)"
            :key="label + band"
            class="flex items-center gap-1"
            :title="`${label}, ${BAND_NAMES[band]}`"
          >
            <img :src="get_icon_path(label)" class="generic-icon h-5 w-5" />
            <span :style="{ color: `var(${BAND_COLORS[band]})` }">
              {{ number(amount) }}
            </span>
          </span>
        </div>
      </div>
    </div>

    <!-- the other sources: a row each -->
    <div
      class="grid grid-cols-[minmax(12rem,auto)_auto_1fr] items-start gap-x-4 gap-y-2"
    >
      <template v-for="row in SIMPLE" :key="row.key">
        <div class="flex flex-col">
          <span class="text-(--text-bright)">{{ source(row.key).title }}</span>
          <span class="text-xs text-(--text-very-muted)">
            {{ source(row.key).detail }}
          </span>
        </div>
        <div class="flex flex-col gap-1">
          <label v-if="row.field" class="flex items-center gap-1.5">
            <input
              class="generic-input h-6 w-14"
              :value="config[row.field]"
              :aria-label="`${source(row.key).title} ${row.unit}`"
              @change="set_number(row.field, $event.target as HTMLInputElement)"
            />
            {{ row.unit }}
          </label>
          <label v-if="row.key === 'chaos'" class="flex items-center gap-1.5">
            <input
              class="generic-input h-6 w-14"
              :value="config.chaos_rested"
              aria-label="Rested runs a week"
              @change="
                set_number('chaos_rested', $event.target as HTMLInputElement)
              "
            />
            of them rested
          </label>
          <label v-if="row.toggle" class="flex items-center gap-1.5">
            <input
              :checked="!!config[row.toggle]"
              type="checkbox"
              @change="set(row.toggle, !config[row.toggle])"
            />
            Counted
          </label>
          <select
            v-for="pick in source(row.key).picks"
            :key="pick.key"
            class="selector max-w-64"
            :value="pick.at"
            :aria-label="pick.item.name"
            :title="pick.item.name"
            @change="
              set_pick(pick.key, +($event.target as HTMLSelectElement).value)
            "
          >
            <option
              v-for="(option, at) in pick.item.options"
              :key="at"
              :value="at"
            >
              {{ option.name }}
            </option>
          </select>
        </div>
        <div class="flex flex-wrap items-center gap-x-3">
          <span
            v-for="{ label, amount, band } in gives(row.key)"
            :key="label + band"
            class="flex items-center gap-1"
            :title="`${label}, ${BAND_NAMES[band]}`"
          >
            <img :src="get_icon_path(label)" class="generic-icon h-5 w-5" />
            <span :style="{ color: `var(${BAND_COLORS[band]})` }">
              {{ number(amount) }}
            </span>
          </span>
        </div>
      </template>

      <!-- dated: the Crucible and the event shop -->
      <template v-for="key in ['crucible', 'event']" :key="key">
        <div class="flex flex-col">
          <span class="text-(--text-bright)">{{ source(key).title }}</span>
          <span class="text-xs text-(--text-very-muted)">
            {{ source(key).detail }}
          </span>
        </div>
        <div class="flex flex-col gap-1">
          <label class="flex items-center gap-1.5">
            <input
              :checked="key === 'crucible' ? config.crucible : config.event_shop"
              type="checkbox"
              @change="
                key === 'crucible'
                  ? set('crucible', !config.crucible)
                  : set('event_shop', !config.event_shop)
              "
            />
            Counted
          </label>
          <label v-if="key === 'crucible'" class="flex items-center gap-1.5">
            <input
              class="generic-input h-6 w-14"
              :value="config.crucible_cleared"
              aria-label="Crucible stages cleared"
              @change="
                set_number('crucible_cleared', $event.target as HTMLInputElement)
              "
            />
            stages cleared
          </label>
          <select
            v-for="pick in source(key).picks"
            :key="pick.key"
            class="selector max-w-64"
            :value="pick.at"
            :aria-label="pick.item.name"
            :title="pick.item.name"
            @change="
              set_pick(pick.key, +($event.target as HTMLSelectElement).value)
            "
          >
            <option
              v-for="(option, at) in pick.item.options"
              :key="at"
              :value="at"
            >
              {{ option.name }}
            </option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-xs text-(--text-very-muted)">
            In all{{
              source(key).grants.length
                ? `, ${source(key).grants.length} times from ${date(source(key).grants[0].at)}`
                : ""
            }}
          </span>
          <div class="flex flex-wrap items-center gap-x-3">
            <span
              v-for="{ label, amount, band } in gives(key)"
              :key="label + band"
              class="flex items-center gap-1"
              :title="`${label}, ${BAND_NAMES[band]}`"
            >
              <img :src="get_icon_path(label)" class="generic-icon h-5 w-5" />
              <span :style="{ color: `var(${BAND_COLORS[band]})` }">
                {{ number(amount) }}
              </span>
            </span>
          </div>
        </div>
      </template>

      <!-- the medal shop and its rows -->
      <div class="flex flex-col">
        <span class="text-(--text-bright)">{{ source("medal").title }}</span>
        <span class="text-xs text-(--text-very-muted)">
          {{ source("medal").detail }}
        </span>
      </div>
      <div class="flex flex-col gap-1">
        <label class="flex items-center gap-1.5">
          <input
            :checked="config.medal_shop"
            type="checkbox"
            @change="set('medal_shop', !config.medal_shop)"
          />
          Counted
        </label>
        <label
          v-for="row in result.medal_rows"
          :key="row.name"
          class="flex items-center gap-1.5 text-(--text-muted)"
        >
          <input
            :checked="!config.off.includes(`${row.shop}/${row.name}`)"
            type="checkbox"
            @change="toggle_row(`${row.shop}/${row.name}`)"
          />
          {{ row.name }} ×{{ row.limit }}, {{ row.cost }} each
        </label>
      </div>
      <div class="flex flex-wrap items-center gap-x-3">
        <span
          v-for="{ label, amount, band } in gives('medal')"
          :key="label + band"
          class="flex items-center gap-1"
          :title="`${label}, ${BAND_NAMES[band]}`"
        >
          <img :src="get_icon_path(label)" class="generic-icon h-5 w-5" />
          <span :style="{ color: `var(${BAND_COLORS[band]})` }">
            {{ number(amount) }}
          </span>
        </span>
      </div>
    </div>
  </div>
</template>
