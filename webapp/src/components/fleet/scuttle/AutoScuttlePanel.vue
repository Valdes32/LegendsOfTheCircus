<!-- webapp/src/components/fleet/scuttle/AutoScuttlePanel.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { ScuttleStore, setTargetServer } from "@/objects/stores/ScuttleStore.ts";

const localTarget = ref(ScuttleStore.targetServer);
const busy       = ref(false);

async function start() {
  busy.value = true;
  setTargetServer(localTarget.value);
  await invoke("start_scuttle");   // команда Rust
  ScuttleStore.running = true;
  busy.value = false;
}

async function stop() {
  busy.value = true;
  await invoke("stop_scuttle");
  ScuttleStore.running = false;
  busy.value = false;
}
</script>

<template>
  <div class="panel">
    <h3>{{ $t("menu.autoScuttle") }}</h3>

    <label>
      {{ $t("menu.targetServer") }}
      <input v-model="localTarget" placeholder="123-ABCDE" />
    </label>

    <button :disabled="busy || ScuttleStore.running" @click="start">
      {{ $t("menu.start") }}
    </button>
    <button :disabled="busy || !ScuttleStore.running" @click="stop">
      {{ $t("menu.stop") }}
    </button>
  </div>
</template>

<style scoped>
.panel { display: flex; flex-direction: column; gap: 8px; width: 260px; }
button { padding: 6px 12px; }
</style>
