<script setup lang="ts">
import { get_icon_path } from "@/Utils/Helpers";
</script>

<template>
  <img
    :src="get_icon_path('Warning')"
    class="h-5 min-h-5 w-5 min-w-5"
    v-tooltip.right="
      'Costs for Vambrace are calculated using no juice or special leaps because I don\'t have the data. If you have this info, or have access to Korean game files, please let me know! (Discord / Github bottom left of the page)  '
    "
    v-once
  />
</template>
