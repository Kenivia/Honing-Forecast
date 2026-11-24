<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { RouterLink } from "vue-router";
import Sidebar from "@/Components/Common/Sidebar.vue";
import Uwuowo from "./Common/Uwuowo/Uwuowo.vue";
import RegionSelector from "./Common/RegionSelector.vue";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher.js";
import { ilevel } from "@/Utils/HoningUtil.js";
import { apply_results } from "./Common/Uwuowo/ApplyResults.js";
import FetchRosterButton from "./Common/Uwuowo/FetchRosterButton.vue";
import SignInHint from "./Common/Uwuowo/SignInHint.vue";
import { market_to_uwuowo_region } from "./Common/Uwuowo/UwuowoUtils";
import { computed } from "vue";
import { linked_now, signed_in } from "@/Utils/BibleAuth";
import { sync_rosters, syncing } from "@/Utils/BibleSync";
import { char_name_valid } from "@/Utils/Helpers";
import BackupButtons from "./Common/BackupButtons.vue";
import NameInput from "./Common/NameInput.vue";

const roster_store = useRosterStore();
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

// "Fetch remaining" looks the roster up by its first lookup-able name
const any_char_names = computed(() =>
  filtered_rosters_lists.value.map(
    (list) =>
      list.find(({ profile }) => char_name_valid(profile.char_name))?.profile
        .char_name ?? list[0].profile.char_name,
  ),
);

// signed in and any character is in a linked roster: region locked, "fetched" shown in the header
const linked_rosters = computed(() =>
  roster_ids.value.map((roster_id, roster_index) => {
    const region = market_to_uwuowo_region(
      roster_config.value.all_regions[roster_id],
    );
    return (
      region !== null &&
      filtered_rosters_lists.value[roster_index].some(
        ({ profile }) => linked_now(region, profile.char_name) !== null,
      )
    );
  }),
);
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
        class="side-bar-item header-button h-max w-max self-center"
        :disabled="syncing"
        @click="sync_rosters()"
      >
        {{ syncing ? "Syncing..." : "Sync with lostark.bible" }}
      </button>
      <BackupButtons
    /></template>
    <template #main>
      <div
        class="flex w-full max-w-full flex-row flex-wrap justify-around gap-4"
      >
        <div
          v-for="(roster_id, roster_index) in roster_ids"
          class="card-shell flex h-fit flex-col pb-2"
          role="group"
          :aria-label="roster_store.roster_label(roster_id)"
          :key="roster_id"
        >
          <div
            v-if="
              roster_config.profiles.length > 1 || linked_rosters[roster_index]
            "
            class="card-header"
          >
            <button
              v-if="roster_index > 0"
              class="arrow-btn arrow-btn-left"
              aria-label="Move roster left"
              @click="roster_store.move_roster(roster_index, roster_index - 1)"
            />
            <NameInput
              :name="roster_config.roster_names[roster_id] ?? ''"
              :placeholder="roster_store.roster_default_label(roster_id)"
              label="Roster name"
              class="card-title h-8 w-44 px-2"
              @rename="(typed) => roster_store.rename_roster(roster_id, typed)"
            />
            <button
              v-if="roster_index < roster_ids.length - 1"
              class="arrow-btn arrow-btn-right"
              aria-label="Move roster right"
              @click="roster_store.move_roster(roster_index, roster_index + 1)"
            />
            <span
              v-if="linked_rosters[roster_index]"
              class="text-sm text-(--uwuowo)"
              >Your roster on lostark.bible</span
            >
            <RegionSelector
              :region="roster_config.all_regions[roster_id]"
              :locked="linked_rosters[roster_index]"
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
          <div class="w-fit max-w-full px-2">
            <div
              v-for="(
                profile_pair, index_in_filtered
              ) in filtered_rosters_lists[roster_index]"
              class="char-row flex h-max min-h-max flex-row items-start justify-around border-b border-(--border-muted)"
              :key="profile_pair.profile.id"
            >
              <div class="max-w- flex w-min flex-row flex-wrap justify-around">
                <Uwuowo
                  :profile="profile_pair.profile"
                  :hide_region="true"
                  :region="
                    roster_config.all_regions[profile_pair.profile.roster_id]
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
                <div class="grid w-full grid-cols-[max-content_1fr]">
                  <button
                    class="arrow-btn arrow-btn-up"
                    :aria-label="`Move ${roster_store.char_label(profile_pair.profile)} up`"
                    v-if="index_in_filtered > 0"
                    @click="
                      () => {
                        const above_profile_index =
                          filtered_rosters_lists[roster_index][
                            index_in_filtered - 1
                          ].profile_index;
                        roster_store.move_profile(
                          above_profile_index,
                          profile_pair.profile_index,
                        );
                      }
                    "
                  />
                  <div v-else></div>
                  <label class="text-no-wrap text-center text-(--achieved)"
                    >Achieved ilevel:
                    {{ ilevel(profile_pair.profile, "achieved") }}</label
                  >

                  <button
                    class="arrow-btn arrow-btn-down"
                    :aria-label="`Move ${roster_store.char_label(profile_pair.profile)} down`"
                    v-if="
                      index_in_filtered <
                      filtered_rosters_lists[roster_index].length - 1
                    "
                    @click="
                      () => {
                        const above_profile_index =
                          filtered_rosters_lists[roster_index][
                            index_in_filtered + 1
                          ].profile_index;
                        roster_store.move_profile(
                          above_profile_index,
                          profile_pair.profile_index,
                        );
                      }
                    "
                  />
                  <div v-else></div>
                  <label class="text-no-wrap text-center text-(--pending)"
                    >Pending ilevel:
                    {{ ilevel(profile_pair.profile, "pending") }}</label
                  >
                </div>
              </div>
              <div class="flex h-full min-h-full flex-col justify-around">
                <RouterLink
                  :to="{
                    name: 'char',
                    params: roster_store.char_params(profile_pair.profile),
                  }"
                  class="generic-button"
                  :aria-label="`Go to character ${roster_store.char_label(profile_pair.profile)}`"
                >
                  Go to character
                </RouterLink>
                <button
                  class="generic-button"
                  :aria-label="`Make a copy of ${roster_store.char_label(profile_pair.profile)}`"
                  @click="
                    roster_store.duplicate_profile(profile_pair.profile_index)
                  "
                >
                  Make a copy
                </button>
                <button
                  v-if="roster_config.profiles.length > 1"
                  class="generic-button btn-cancel"
                  :aria-label="`Delete ${roster_store.char_label(profile_pair.profile)}`"
                  @click="
                    roster_store.delete_profile(profile_pair.profile_index)
                  "
                >
                  Delete
                </button>
              </div>
            </div>

            <FetchRosterButton
              v-if="roster_config.all_regions[roster_id] !== 'Custom'"
              :region="
                market_to_uwuowo_region(roster_config.all_regions[roster_id])
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
            <SignInHint
              v-if="roster_config.all_regions[roster_id] !== 'Custom'"
            />
            <button
              class="generic-button w-full"
              @click="roster_store.add_char(roster_id)"
            >
              Add new character
            </button>
          </div>
        </div>
        <div class="w-111">
          <button
            class="side-bar-item header-button h-max w-full"
            @click="roster_store.add_roster()"
          >
            Add new roster
          </button>
        </div>
      </div>
    </template>
  </Sidebar>
</template>
<style scoped>
/* Base Variables & Structural Setup */

.btn-cancel {
  background: var(--warning-dark);
  color: var(--text-bright);
}

.char-row {
  margin-bottom: 1rem;
  gap: 4px;
  flex-wrap: wrap;
}

.arrow-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 4px;
  border: 1px solid var(--border-muted);
  cursor: pointer;
  user-select: none;
  background: transparent;
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

.arrow-btn-left::after {
  transform: rotate(135deg);
  margin-left: 2px;
}

.arrow-btn-right::after {
  transform: rotate(-45deg);
  margin-right: 2px;
}
</style>
