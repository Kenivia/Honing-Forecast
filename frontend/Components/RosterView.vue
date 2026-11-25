<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import Sidebar from "@/Components/Common/Sidebar.vue";
import Uwuowo from "./Common/Uwuowo/Uwuowo.vue";
import RegionSelector from "./Common/RegionSelector.vue";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher.js";
import { ilevel } from "@/Utils/HoningUtil.js";
import { apply_results } from "./Common/Uwuowo/ApplyResults.js";
import FetchRosterButton from "./Common/Uwuowo/FetchRosterButton.vue";
import RefreshButton from "./Common/Uwuowo/RefreshButton.vue";
import SignInHint from "./Common/Uwuowo/SignInHint.vue";
import { market_to_uwuowo_region } from "./Common/Uwuowo/UwuowoUtils";
import { computed, nextTick, ref } from "vue";
import { VueDraggable } from "vue-draggable-plus";
import { signed_in } from "@/Utils/BibleAuth";
import { sync_rosters, syncing } from "@/Utils/BibleSync";
import { char_name_valid, class_key, get_icon_path } from "@/Utils/Helpers";
import IconButton from "./Common/IconButton.vue";
import { CLASS_KEYS } from "@/Utils/Constants";
import BackupButtons from "./Common/BackupButtons.vue";
import NameInput from "./Common/NameInput.vue";
import HoverTip from "./Common/HoverTip.vue";
import Popup from "./Common/Popup.vue";
import ClassIcon from "./Common/ClassIcon.vue";
import { has_edited_mats } from "@/Stores/RosterConfig";
import { CharProfile } from "@/Stores/CharacterProfile";

const roster_store = useRosterStore();
const router = useRouter();
const { roster_config, roster_ids } = storeToRefs(roster_store);

const filtered_rosters_lists = computed(() =>
  roster_ids.value.map((roster_id) =>
    roster_config.value.profiles
      .map((x, index) => {
        return { profile: x, profile_index: index };
      })
      .filter((y) => y.profile.roster_id === roster_id),
  ),
);

// a character row: 200px name / ilevel column, 8px gap, 4 icon buttons (140px), 14px padding and border
const COL_W = 362;
const GAP = 16;
// card fits min(chars + the add cell, 3) columns plus padding
const card_width = (n: number) => {
  const cols = Math.min(n + 1, 3);
  return `${cols * COL_W + (cols - 1) * GAP + 24}px`;
};

// "Fetch remaining" looks the roster up by its first lookup-able name
const any_char_names = computed(() =>
  filtered_rosters_lists.value.map(
    (list) =>
      list.find(({ profile }) => char_name_valid(profile.char_name))?.profile
        .char_name ?? list[0].profile.char_name,
  ),
);

const has_class = (p: CharProfile) => CLASS_KEYS.has(class_key(p.class_name));

// the card opens the character, except from its inputs and buttons
function open_char(event: MouseEvent, profile: CharProfile) {
  if ((event.target as Element).closest("input, button, select, a, .no-open"))
    return;
  if (window.getSelection()?.toString()) return;
  router.push({ name: "char", params: roster_store.char_params(profile) });
}

// duplicate's tip names the copy until the tip closes
const copied = ref<Record<string, string>>({});
function duplicate(profile: CharProfile) {
  const index = roster_config.value.profiles.indexOf(profile);
  const made = roster_store.duplicate_profile(index);
  copied.value[profile.id] = `${roster_store.char_label(made)} has been made`;
}

// a character with typed materials asks first
const to_delete = ref<CharProfile | null>(null);
function remove(profile: CharProfile) {
  if (has_edited_mats(roster_config.value, profile)) to_delete.value = profile;
  else delete_now(profile);
}
function delete_now(profile: CharProfile) {
  to_delete.value = null;
  const index = roster_config.value.profiles.indexOf(profile);
  if (index >= 0) roster_store.delete_profile(index);
}

// an unknown class is picked by hand; a fetch or sync overwrites it
const CLASS_NAMES = [...CLASS_KEYS].map((k) => k[0].toUpperCase() + k.slice(1));
const class_for = ref<CharProfile | null>(null);
const class_search = ref("");
const shown_classes = computed(() =>
  CLASS_NAMES.filter((n) =>
    n.toLowerCase().includes(class_search.value.trim().toLowerCase()),
  ),
);
const search_box = ref<HTMLInputElement | null>(null);
async function open_picker(profile: CharProfile) {
  class_search.value = "";
  class_for.value = profile;
  await nextTick();
  search_box.value?.focus();
}
function pick_class(name: string) {
  class_for.value.class_name = name;
  class_for.value = null;
}
</script>

<template>
  <Sidebar :width="969" header="Roster setup">
    <template #sidebar>
      <button
        class="side-bar-item header-button h-max w-max self-center"
        @click="roster_store.add_roster()"
      >
        Add new roster
      </button>
      <button
        v-if="signed_in"
        class="side-bar-item header-button relative h-max w-max self-center overflow-hidden text-(--uwuowo)!"
        :disabled="syncing"
        @click="sync_rosters()"
      >
        {{ syncing ? "Syncing..." : "Sync linked roster" }}
        <img
          :src="get_icon_path('Artist Cry')"
          alt=""
          class="pointer-events-none absolute inset-0 h-full w-full object-contain opacity-10"
        />
      </button>
      <BackupButtons
    /></template>
    <template #main>
      <div
        class="flex w-full max-w-full flex-row flex-wrap justify-around gap-4"
      >
        <div
          v-for="(roster_id, roster_index) in roster_ids"
          class="card-shell flex h-fit max-w-full flex-col pb-2"
          :style="{
            width: `max(${card_width(3)}, min(100%, ${card_width(filtered_rosters_lists[roster_index].length)}))`,
          }"
          role="group"
          :aria-label="roster_store.roster_label(roster_id)"
          :key="roster_id"
        >
          <div
            v-if="
              roster_config.profiles.length > 1 ||
              roster_store.roster_linked(roster_id)
            "
            class="card-header flex-wrap gap-2"
          >
            <div class="flex flex-row items-center gap-2">
              <NameInput
                :name="roster_config.roster_names[roster_id] ?? ''"
                :placeholder="roster_store.roster_default_label(roster_id)"
                label="Roster name"
                class="card-title roster-name h-9 w-56 px-1 text-xl!"
                @rename="
                  (typed) => roster_store.rename_roster(roster_id, typed)
                "
              />
              <template v-if="roster_ids.length > 1">
                <button
                  class="icon-button arrow-btn arrow-btn-up"
                  aria-label="Move roster up"
                  :disabled="roster_index === 0"
                  @click="
                    roster_store.move_roster(roster_index, roster_index - 1)
                  "
                />
                <button
                  class="icon-button arrow-btn arrow-btn-down"
                  aria-label="Move roster down"
                  :disabled="roster_index === roster_ids.length - 1"
                  @click="
                    roster_store.move_roster(roster_index, roster_index + 1)
                  "
                />
              </template>
            </div>
            <span
              v-if="roster_store.roster_linked(roster_id)"
              class="text-sm text-(--uwuowo)"
              >Linked roster on lostark.bible</span
            >
            <RegionSelector
              :region="roster_config.all_regions[roster_id]"
              :locked="roster_store.roster_linked(roster_id)"
              :region_change="
                (event) => {
                  const new_region = (event.target as HTMLSelectElement)
                    .value as MarketRegions;
                  roster_store.set_region(roster_id, new_region);
                  start_fetch(new_region);
                }
              "
            />
          </div>
          <div class="w-full max-w-full px-2">
            <VueDraggable
              class="char-grid"
              :style="{ '--col-w': `${COL_W}px`, '--gap': `${GAP}px` }"
              :model-value="filtered_rosters_lists[roster_index]"
              @update:model-value="
                (list) =>
                  roster_store.reorder_roster(
                    roster_id,
                    list.map((x) => x.profile.id),
                  )
              "
              :animation="150"
              ghost-class="drag-ghost"
              handle=".drag-handle"
              draggable=".char-row"
            >
              <div
                v-for="profile_pair in filtered_rosters_lists[roster_index]"
                class="char-row scroll-x @container cursor-pointer rounded border border-(--border-muted) bg-(--bg-main) p-1.5 hover:border-(--border-main)"
                :key="profile_pair.profile.id"
                @click="open_char($event, profile_pair.profile)"
              >
                <div class="grid grid-cols-[200px_auto] gap-x-2 gap-y-1">
                  <div class="flex min-h-17 flex-col justify-between">
                    <Uwuowo
                      :profile="profile_pair.profile"
                      :hide_region="true"
                      :card="true"
                      :region="
                        roster_config.all_regions[
                          profile_pair.profile.roster_id
                        ]
                      "
                      :apply="
                        (result, force_t4, _name, class_name) => {
                          apply_results(
                            profile_pair.profile,
                            result,
                            force_t4,
                            true,
                            class_name,
                          );
                        }
                      "
                    />
                  </div>
                  <div class="row-span-2 flex flex-col gap-1">
                    <div class="grid min-h-0 flex-1 place-items-center">
                      <ClassIcon
                        v-if="has_class(profile_pair.profile)"
                        large
                        class="aspect-square h-full max-h-22 opacity-30"
                        :class_name="profile_pair.profile.class_name"
                      />
                      <button
                        v-else
                        class="aspect-square h-full max-h-22 cursor-pointer opacity-30 hover:opacity-60"
                        :aria-label="`Pick the class of ${roster_store.char_label(profile_pair.profile)}`"
                        title="Pick the class"
                        @click="open_picker(profile_pair.profile)"
                      >
                        <ClassIcon
                          large
                          class="size-full"
                          :class_name="profile_pair.profile.class_name"
                        />
                      </button>
                    </div>
                    <div
                      class="no-open grid w-fit cursor-default grid-cols-[auto_auto] gap-1 self-center @min-[348px]:flex"
                    >
                      <RefreshButton
                        v-if="roster_store.char_linked(profile_pair.profile)"
                        :profile_id="profile_pair.profile.id"
                      />
                      <IconButton
                        icon="arrows-up-down-left-right-solid-full.svg"
                        class="drag-handle cursor-grab!"
                        :aria-label="`Drag ${roster_store.char_label(profile_pair.profile)} to reorder`"
                        title="Drag to reorder"
                      />
                      <HoverTip
                        :text="
                          copied[profile_pair.profile.id] ??
                          'Duplicate this character'
                        "
                        :sticky="false"
                        class="flex"
                        @closed="delete copied[profile_pair.profile.id]"
                      >
                        <IconButton
                          icon="copy-solid-full.svg"
                          :aria-label="`Make a copy of ${roster_store.char_label(profile_pair.profile)}`"
                          @click="duplicate(profile_pair.profile)"
                        />
                      </HoverTip>
                      <IconButton
                        v-if="roster_config.profiles.length > 1"
                        icon="trash-solid-full.svg"
                        :aria-label="`Delete ${roster_store.char_label(profile_pair.profile)}`"
                        title="Delete"
                        @click="remove(profile_pair.profile)"
                      />
                    </div>
                  </div>
                  <div class="flex flex-col justify-end pl-1 text-nowrap">
                    <span class="text-(--achieved)"
                      >Achieved ilevel:
                      {{ ilevel(profile_pair.profile, "achieved") }}</span
                    >
                    <span class="text-(--pending)"
                      >Pending ilevel:
                      {{ ilevel(profile_pair.profile, "pending") }}</span
                    >
                  </div>
                </div>
              </div>
              <div
                class="flex flex-col justify-center gap-2 rounded border border-dashed border-(--border-muted) p-1.5"
              >
                <FetchRosterButton
                  v-if="roster_config.all_regions[roster_id] !== 'Custom'"
                  :region="
                    market_to_uwuowo_region(
                      roster_config.all_regions[roster_id],
                    )
                  "
                  :any_char_name="any_char_names[roster_index]"
                  :apply="
                    (result, force_t4, char_name, class_name) => {
                      const added = roster_store.add_char(roster_id);
                      roster_store.rename_char(added.id, char_name);
                      apply_results(added, result, force_t4, true, class_name);
                    }
                  "
                />
                <button
                  class="generic-button w-full"
                  @click="roster_store.add_char(roster_id)"
                >
                  Add new character
                </button>
                <SignInHint
                  v-if="roster_config.all_regions[roster_id] !== 'Custom'"
                />
              </div>
            </VueDraggable>
          </div>
        </div>
        <div
          :style="{
            width: `${card_width(3)}`,
          }"
        >
          <button
            class="header-button mx-auto! block h-max w-fit px-10!"
            @click="roster_store.add_roster()"
          >
            Add new roster
          </button>
        </div>
      </div>
      <Popup
        :show_popup="to_delete !== null"
        label="Confirm delete"
        @close_popup="to_delete = null"
      >
        <span class="pb-4"
          >Delete
          <span class="font-bold">{{
            to_delete && roster_store.char_label(to_delete)
          }}</span>? Its materials and chests will be lost.</span
        >
        <div class="flex flex-row gap-2">
          <button
            class="generic-button btn-cancel flex-1"
            @click="delete_now(to_delete)"
          >
            Delete
          </button>
          <button class="generic-button flex-1" @click="to_delete = null">
            Cancel
          </button>
        </div>
      </Popup>
      <Popup
        :show_popup="class_for !== null"
        label="Pick a class"
        @close_popup="class_for = null"
      >
        <span class="pb-3"
          >Which class is
          <span class="font-bold">{{
            class_for && roster_store.char_label(class_for)
          }}</span>?</span
        >
        <label class="flex flex-row items-center gap-2 pb-2">
          Search:
          <input
            v-model="class_search"
            class="generic-input h-8 flex-1 px-2"
            aria-label="Search classes"
            ref="search_box"
            @keydown.enter="shown_classes[0] && pick_class(shown_classes[0])"
          />
        </label>
        <div class="grid grid-cols-2 gap-1 sm:grid-cols-3">
          <button
            v-for="name in shown_classes"
            :key="name"
            class="generic-button flex flex-row items-center gap-2"
            @click="pick_class(name)"
          >
            <ClassIcon :class_name="name" />
            {{ name }}
          </button>
          <span v-if="!shown_classes.length" class="text-(--text-muted)"
            >No class matches</span
          >
        </div>
      </Popup>
    </template>
  </Sidebar>
</template>
<style scoped>
.btn-cancel {
  background: var(--warning-dark);
  color: var(--text-bright);
}

/* columns of COL_W, at most 3, fewer as the card narrows */
.char-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, min(var(--col-w), 100%));
  justify-content: center;
  column-gap: var(--gap);
  row-gap: 0.5rem;
}

.drag-ghost {
  opacity: 0.4;
  background: var(--bg-very-muted);
}

/* underline only, like the artisan input */
.roster-name {
  border-color: transparent !important;
  border-bottom-color: var(--border-muted) !important;
  background: transparent !important;
}

.arrow-btn::after {
  content: "";
  display: block;
  width: 6px;
  height: 6px;
  border-right: 1px solid currentColor;
  border-bottom: 1px solid currentColor;
}
.arrow-btn-down::after {
  transform: rotate(45deg);
  margin-bottom: 2px;
}
.arrow-btn-up::after {
  transform: rotate(-135deg);
  margin-top: 2px;
}
</style>
