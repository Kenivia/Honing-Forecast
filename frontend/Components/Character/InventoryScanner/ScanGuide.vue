<script setup lang="ts"></script>

<template>
  <div class="card-shell w-150! max-w-full text-base">
    <div class="card-header">
      <span class="card-title">How to use</span>
    </div>
    <div class="card-body flex flex-col gap-3">
      <div>
        <div class="text-(--text-bright)">Step 1</div>
        <div class="pl-4">
          Press <span class="text-blue-400">Share screen</span>. Share the LOST
          ARK window. (it may not appear as an option while you're in a loading
          screen)
        </div>
      </div>
      <div>
        <div class="text-(--text-bright)">Step 2</div>
        <div class="pl-4">
          Open your storage or just your inventory, then hover items in game.
          You only need to do it for items with an
          <span class="text-(--series-fusion)">Orange</span> border, and only a
          few frames per hover is needed. You should hover until no
          <span class="text-(--series-fusion)">Orange</span> slots remain.
        </div>
      </div>

      <div>
        <div class="text-(--text-bright)">Step 3</div>
        <div class="pl-4">
          Check the manifest below, and "export to calculator" when ready. If
          you don't export, it won't be saved on reload.
        </div>
      </div>

      <div>
        <div class="text-(--text-bright)">Note:</div>
        <div class="pl-4">
          Try not to move items or open chests during a scan. This may cause
          duplicate reads. If this happens, you can "Reset scan" and start over,
          or set the extra one to "irrelevant" manually.
        </div>
      </div>
    </div>
  </div>
</template>
