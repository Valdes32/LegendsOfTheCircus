// webapp/src/objects/stores/ScuttleStore.ts
import { reactive } from "vue";
import LocalStore, { LocalKey } from "@/objects/stores/LocalStore.ts";

// ✅ Создаем типизированную обертку
interface UserSettings {
  targetServer?: string;
}

function getTypedUserStore() {
  const store = LocalStore(LocalKey.USER_STORE, {} as UserSettings);
  return {
    get value(): UserSettings | null {
      return store.value as UserSettings || null;
    },
    set value(data: UserSettings) {
      store.value = data;
    }
  };
}

interface ScuttleState {
  targetServer: string;
  running: boolean;
}

export const ScuttleStore = reactive<ScuttleState>({
  targetServer: getTypedUserStore().value?.targetServer ?? "",
  running: false,
});

export function setTargetServer(server: string) {
  ScuttleStore.targetServer = server;
  const store = getTypedUserStore();
  store.value = { ...(store.value ?? {}), targetServer: server };
}
