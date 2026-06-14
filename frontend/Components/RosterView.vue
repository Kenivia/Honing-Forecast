<script setup lang="ts">
import {
  CharProfile,
  DEFAULT_CHAR_PROFILE_NO_WORKER,
  init_workers,
} from "@/Stores/CharacterProfile";
import {
  create_default_owned_input_column,
  useRosterStore,
} from "@/Stores/RosterConfig";
import { format_char_name } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { RouterLink } from "vue-router";
import Sidebar from "@/Components/Common/Sidebar.vue";
import Uwuowo from "./Common/Uwuowo/Uwuowo.vue";
import RegionSelector from "./Common/RegionSelector.vue";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher.js";
import { ilevel } from "@/Utils/HoningUtil.js";
import { apply_results } from "./Common/Uwuowo/ApplyResults.js";
import FetchRosterButton from "./Common/Uwuowo/FetchRosterButton.vue";
import { computed } from "vue";

const roster_store = useRosterStore();
const { roster_config, roster_ids, all_profiles } = storeToRefs(roster_store);

function add_new_char(roster_id: number) {
  let new_char = init_workers(DEFAULT_CHAR_PROFILE_NO_WORKER);
  new_char.char_name = format_char_name(
    "Newchar",
    roster_config.value.profiles.length,
  );
  new_char.roster_id = roster_id;
  roster_store.add_profile(new_char);
}
function add_new_roster(roster_id: number) {
  add_new_char(roster_id);

  roster_config.value.roster_mats_owned[roster_id] =
    create_default_owned_input_column();
  roster_config.value.tradable_mats_owned[roster_id] =
    create_default_owned_input_column();
  roster_config.value.all_regions[roster_id] = "nae";
}

function duplicate(index) {
  let this_parsed = {
    ...init_workers(DEFAULT_CHAR_PROFILE_NO_WORKER),
    ...roster_config.value.profiles[index],
  };

  let new_char = init_workers(JSON.parse(JSON.stringify(this_parsed)));
  new_char.char_name = format_char_name(
    "Newchar",
    roster_config.value.profiles.length,
  );
  roster_store.add_profile(new_char);
}

function delete_profile(index, roster_id) {
  // console.log(this_roster_profiles.length)
  if (roster_config.value.active_profile_index >= index) {
    roster_store.switch_profile(Math.max(index - 1, 0));
  }
  roster_config.value.profiles.splice(index, 1);

  if (
    roster_config.value.profiles.filter((x) => x.roster_id == roster_id)
      .length === 0
  ) {
    delete roster_config.value.roster_mats_owned[roster_id];
    delete roster_config.value.tradable_mats_owned[roster_id];
    delete roster_config.value.all_regions[roster_id];
  }

  // console.log(this_roster_profiles.length)
}

const filtered_rosters_lists = computed(() =>
  roster_ids.value.map((roster_id) =>
    roster_config.value.profiles
      .map((x, index) => {
        return { profile: x, profile_index: index };
      })
      .filter((y) => y.profile.roster_id === roster_id),
  ),
);

function swap_profiles(i: number, j: number) {
  [all_profiles.value[i], all_profiles.value[j]] = [
    all_profiles.value[j],
    all_profiles.value[i],
  ];
}
</script>

<template>
  <Sidebar :width="969" header="Roster setup">
    <template #sidebar>
      <button
        class="side-bar-item header-button h-max w-max self-center"
        @click="
          () =>
            add_new_roster(
              Math.max(...roster_config.profiles.map((x) => x.roster_id)) + 1,
            )
        "
      >
        Add new roster
      </button></template
    >
    <template #main>
      <div
        class="flex w-full max-w-full flex-row flex-wrap justify-around gap-4"
      >
        <div
          v-for="(roster_id, roster_index) in roster_ids"
          class="card-shell flex h-fit flex-col pb-2"
          :key="roster_id"
        >
          <div v-if="roster_config.profiles.length > 1" class="card-header">
            <span class="card-title"> Roster {{ roster_index + 1 }}</span>
            <RegionSelector
              :region="roster_config.all_regions[roster_id]"
              :region_change="
                (event) => {
                  const new_region = (event.target as HTMLSelectElement)
                    .value as MarketRegions;
                  // console.log(roster_config.all_regions);
                  // console.log('set roster_id', roster_id);
                  roster_config.all_regions[roster_id] = new_region;
                  // console.log(roster_config.all_regions);
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
              :key="`${profile_pair.profile_index}-${profile_pair.profile.roster_id}`"
            >
              <div class="max-w- flex w-min flex-row flex-wrap justify-around">
                <Uwuowo
                  :profile_index="profile_pair.profile_index"
                  :name="profile_pair.profile.char_name"
                  :name_change="
                    (new_name) => {
                      roster_config.profiles[
                        profile_pair.profile_index
                      ].char_name = new_name;
                    }
                  "
                  :hide_region="true"
                  :region="
                    roster_config.all_regions[profile_pair.profile.roster_id]
                  "
                  :apply="
                    (result, force_t4) => {
                      // console.log(
                      //   roster_config.profiles[profile_index].char_name,
                      //   profile_index,
                      // );
                      // console.log(result, force_t4);
                      apply_results(
                        profile_pair.profile,
                        result,
                        force_t4,
                        true,
                      );
                    }
                  "
                />
                <div class="grid w-full grid-cols-[max-content_1fr]">
                  <button
                    class="arrow-btn arrow-btn-up"
                    v-if="index_in_filtered > 0"
                    @click="
                      () => {
                        const above_profile_index =
                          filtered_rosters_lists[roster_index][
                            index_in_filtered - 1
                          ].profile_index;
                        swap_profiles(
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
                        swap_profiles(
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
                    params: { characterName: profile_pair.profile.char_name },
                  }"
                  class="generic-button"
                >
                  Go to character
                </RouterLink>
                <button
                  class="generic-button"
                  @click="() => duplicate(profile_pair.profile_index)"
                >
                  Make a copy
                </button>
                <button
                  v-if="roster_config.profiles.length > 1"
                  class="generic-button btn-cancel"
                  @click="
                    () => delete_profile(profile_pair.profile_index, roster_id)
                  "
                >
                  Delete
                </button>
              </div>
            </div>

            <FetchRosterButton
              v-if="roster_config.all_regions[roster_id] !== 'Custom'"
              :region="
                roster_config.all_regions[roster_id] === 'nae' ? 'NA' : 'CE'
              "
              :any_char_name="
                filtered_rosters_lists[roster_index][0].profile.char_name
              "
              :apply="
                (result, force_t4, char_name) => {
                  add_new_char(roster_id);
                  all_profiles[all_profiles.length - 1].char_name = char_name;
                  // console.log(char_name);
                  apply_results(
                    all_profiles[all_profiles.length - 1],
                    result,
                    force_t4,
                    true,
                  );
                }
              "
            />
            <button
              class="generic-button w-full"
              @click="() => add_new_char(roster_id)"
            >
              Add new character
            </button>
          </div>
        </div>
        <div class="w-111">
          <button
            class="side-bar-item header-button h-max w-full"
            @click="
              () =>
                add_new_roster(
                  Math.max(...roster_config.profiles.map((x) => x.roster_id)) +
                    1,
                )
            "
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
</style>
