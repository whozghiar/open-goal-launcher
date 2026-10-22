import {
  levelEditedCompile,
  levelEditedDecompile,
  levelEditedInstallLevel,
  levelEditedLaunch,
  levelEditedPrepareRuntime,
  levelEditedRemoveLevel,
} from "$lib/rpc/features";
import { jobTracker } from "$lib/stores/JobStore";
import { unwrapFunctionStore, format } from "svelte-i18n";

const $format = unwrapFunctionStore(format);

function step(label: string, task: () => Promise<string | null>) {
  return {
    status: "queued" as const,
    label: $format(label),
    task: async () => {
      const error = await task();
      if (error) {
        jobTracker.updateFailureReason(error);
        return false;
      }
      return true;
    },
  };
}

// a level exported by Jak Level Editor (an installed one without levelDir), played in the isolated
// game features/jak3/level_edited, made again first when the active tooling version changed
export async function setupPlayEditedLevel(
  levelDir: string | undefined,
  levelName: string,
) {
  jobTracker.init([
    step("levelEdited_prepare", levelEditedPrepareRuntime),
    step("setup_decompile", levelEditedDecompile),
    ...(levelDir
      ? [
          step("levelEdited_installLevel", () =>
            levelEditedInstallLevel(levelDir),
          ),
        ]
      : []),
    step("setup_compile", levelEditedCompile),
    step("levelEdited_play", () => levelEditedLaunch(levelName)),
  ]);
}

export async function setupRemoveEditedLevel(levelName: string) {
  jobTracker.init([
    step("levelEdited_removeLevel", () => levelEditedRemoveLevel(levelName)),
    step("levelEdited_prepare", levelEditedPrepareRuntime),
    step("setup_decompile", levelEditedDecompile),
    step("setup_compile", levelEditedCompile),
    step("setup_done", async () => null),
  ]);
}
