<script setup lang="ts">
import {
  CharProfile,
  create_default_char_profile,
  recreate_char_profile,
} from "@/Stores/CharacterProfile";
import {
  create_default_owned_input_column,
  useRosterStore,
} from "@/Stores/RosterConfig";
import {
  achieved_ilevel,
  format_char_name,
  pending_ilevel,
} from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { ref } from "vue";
import { RouterLink } from "vue-router";
import Sidebar from "@/Components/Common/Sidebar.vue";

const roster_store = useRosterStore();
const { roster_config, roster_ids } = storeToRefs(roster_store);

const names = ref(roster_config.value.profiles.map((y) => y.char_name));

function add_new_char(roster_id: number) {
  let new_char = create_default_char_profile();
  new_char.char_name = format_char_name(
    "Newchar",
    roster_config.value.profiles.length,
  );
  new_char.roster_id = roster_id;
  names.value.push(new_char.char_name);
  roster_store.add_profile(new_char);
}
function add_new_roster(roster_id: number) {
  let new_char = create_default_char_profile();
  new_char.char_name = format_char_name(
    "Newchar",
    roster_config.value.profiles.length,
  );
  new_char.roster_id = roster_id;
  names.value.push(new_char.char_name);
  roster_config.value.profiles.push(new_char);

  roster_config.value.roster_mats_owned[roster_id] =
    create_default_owned_input_column();
  roster_config.value.tradable_mats_owned[roster_id] =
    create_default_owned_input_column();
}

function duplicate(index) {
  let this_parsed = {
    ...create_default_char_profile(),
    ...roster_config.value.profiles[index],
  };

  let new_char = recreate_char_profile(JSON.parse(JSON.stringify(this_parsed)));
  new_char.char_name = format_char_name(
    "Newchar",
    roster_config.value.profiles.length,
  );
  names.value.push(new_char.char_name);
  roster_store.add_profile(new_char);
}

function delete_profile(index, roster_id) {
  // console.log(this_roster_profiles.length)
  if (roster_config.value.active_profile_index >= index) {
    roster_store.switch_profile(Math.max(index - 1, 0));
  }
  roster_config.value.profiles.splice(index, 1);
  names.value.splice(index, 1);

  if (
    roster_config.value.profiles.filter((x) => x.roster_id == roster_id)
      .length === 0
  ) {
    delete roster_config.value.roster_mats_owned[roster_id];
    delete roster_config.value.tradable_mats_owned[roster_id];
  }

  // console.log(this_roster_profiles.length)
}
</script>

<template>
  <Sidebar :width="969" header="Roster setup">
    <template #sidebar
      ><div class="side-bar-item">uwuowo importing coming soon...</div>

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
      <div class="flex flex-row flex-wrap justify-around gap-4">
        <div
          v-for="(roster_id, roster_index) in roster_ids"
          class="card-shell flex h-fit flex-col"
          :key="roster_id"
        >
          <div v-if="roster_config.profiles.length > 1" class="card-header">
            <span class="card-title"> Roster {{ roster_index + 1 }}</span>
          </div>
          <div class="card-body">
            <div
              v-for="[profile, profile_index] in roster_config.profiles
                .map((x, index): [CharProfile, number] => [x, index])
                .filter((y) => y[0].roster_id === roster_id)"
              class="char-row flex flex-row items-start justify-around border-b border-(--border-muted)"
              :key="profile_index"
            >
              <div>
                <input
                  v-model="names[profile_index]"
                  @change="
                    ((names[profile_index] = format_char_name(
                      names[profile_index],
                      profile_index,
                    )),
                    (profile.char_name = names[profile_index]))
                  "
                  class="generic-input bg-(--bg-bright)!"
                />

                <div class="char-meta flex flex-col">
                  <label class="text-no-wrap text-(--achieved)"
                    >Achieved ilevel: {{ achieved_ilevel(profile) }}</label
                  >
                  <label class="text-no-wrap text-(--pending)"
                    >Pending ilevel: {{ pending_ilevel(profile) }}</label
                  >
                </div>
              </div>
              <div class="flex flex-col">
                <RouterLink
                  :to="{
                    name: 'char',
                    params: { characterName: profile.char_name },
                  }"
                  class="generic-button"
                >
                  Go to character
                </RouterLink>
                <button
                  class="generic-button"
                  @click="() => duplicate(profile_index)"
                >
                  Make a copy
                </button>
                <button
                  v-if="roster_config.profiles.length > 1"
                  class="generic-button btn-cancel"
                  @click="() => delete_profile(profile_index, roster_id)"
                >
                  Delete
                </button>
              </div>
            </div>
            <button
              class="generic-button w-full"
              @click="() => add_new_char(roster_id)"
            >
              Add new character
            </button>
          </div>
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

.char-meta {
  width: 200px;
}

.char-row {
  margin-bottom: 1rem;
  gap: 4px;
  flex-wrap: wrap;
}
</style>
