import type { SupportedGame } from "$lib/rpc/bindings/SupportedGame";
import type { ModInfo } from "$lib/rpc/bindings/ModInfo";
import { writable } from "svelte/store";
interface AvailableUpdate {
  updateAvailable: boolean;
  versionNumber: string | undefined;
}

interface UpdateState {
  selectedTooling: AvailableUpdate;
  selectedLevelEditor: AvailableUpdate;
  selectedAnimationStudio: AvailableUpdate;
}

export type UpdateKey = keyof UpdateState;

export const UpdateStore = writable<UpdateState>({
  selectedTooling: {
    updateAvailable: false,
    versionNumber: undefined,
  },
  selectedLevelEditor: {
    updateAvailable: false,
    versionNumber: undefined,
  },
  selectedAnimationStudio: {
    updateAvailable: false,
    versionNumber: undefined,
  },
});
