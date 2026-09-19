import type { SupportedGame } from "$lib/rpc/bindings/SupportedGame";
import {
  baseGameIsoExists,
  extractIsoForModInstall,
  decompileForModInstall,
  compileForModInstall,
  saveModInstallInfo,
  downloadAndExtractNewMod,
  downloadAndExtractTexturePack,
  updateModTexturePackData,
} from "$lib/rpc/features";
import { setModEnabledTexturePacks } from "$lib/rpc/config";
import type { AttachedTexturePack } from "$lib/features/texture-packs";
import { jobTracker } from "$lib/stores/JobStore";
import { isoPrompt } from "$lib/utils/file-dialogs";
import { unwrapFunctionStore, format } from "svelte-i18n";

const $format = unwrapFunctionStore(format);

export async function setupModInstallation(
  activeGame: SupportedGame,
  modName: string,
  modSourceName: string,
  modDownloadUrl: string | undefined,
  modVersion: string,
  attachedTexturePacks: AttachedTexturePack[] = [],
) {
  jobTracker.init([
    {
      status: "queued",
      label: $format("setup_download"),
      task: async () => {
        if (modDownloadUrl) {
          let error = await downloadAndExtractNewMod(
            activeGame,
            modDownloadUrl,
            modName,
            modSourceName,
          );
          if (error) {
            jobTracker.updateFailureReason(error);
            return false;
          }
        }
        // Download and deploy any texture packs attached to this mod release
        if (attachedTexturePacks && attachedTexturePacks.length > 0) {
          for (const pack of attachedTexturePacks) {
            let error = await downloadAndExtractTexturePack(
              activeGame,
              pack.downloadUrl,
              pack.name,
            );
            if (error) {
              jobTracker.updateFailureReason(error);
              return false;
            }
          }
          const packNames = attachedTexturePacks.map((p) => p.name);
          let errSet = await setModEnabledTexturePacks(
            activeGame,
            modSourceName,
            modName,
            packNames,
          );
          if (errSet) {
            jobTracker.updateFailureReason(errSet);
            return false;
          }
          let errUpdate = await updateModTexturePackData(
            activeGame,
            modSourceName,
            modName,
          );
          if (errUpdate) {
            jobTracker.updateFailureReason(errUpdate);
            return false;
          }
        }
        return true;
      },
    },
    {
      status: "queued",
      label: $format("setup_extractAndVerify"),
      task: async () => {
        const isoAlreadyExtracted = await baseGameIsoExists(activeGame);
        if (!isoAlreadyExtracted) {
          let sourcePath = await isoPrompt(
            $format("setup_prompt_ISOFileLabel"),
            $format("setup_prompt_selectISO"),
          );
          if (sourcePath !== undefined) {
            let error = await extractIsoForModInstall(
              activeGame,
              modName,
              modSourceName,
              sourcePath,
            );
            if (error) {
              jobTracker.updateFailureReason(error);
              return false;
            }
          } else {
            jobTracker.updateFailureReason($format("setup_error_noISO"));
            return false;
          }
        }
        return true;
      },
    },
    {
      status: "queued",
      label: $format("setup_decompile"),
      task: async () => {
        let error = await decompileForModInstall(
          activeGame,
          modName,
          modSourceName,
        );
        if (error) {
          jobTracker.updateFailureReason(error);
          return false;
        }
        return true;
      },
    },
    {
      status: "queued",
      label: $format("setup_compile"),
      task: async () => {
        let error = await compileForModInstall(
          activeGame,
          modName,
          modSourceName,
        );
        if (error) {
          jobTracker.updateFailureReason(error);
          return false;
        }
        return true;
      },
    },
    {
      status: "queued",
      label: $format("setup_done"),
      task: async () => {
        let error = await saveModInstallInfo(
          activeGame,
          modName,
          modSourceName,
          modVersion,
        );
        if (error) {
          jobTracker.updateFailureReason(error);
          return false;
        }
        // Ensure active texture packs remain registered for this mod
        if (attachedTexturePacks && attachedTexturePacks.length > 0) {
          await setModEnabledTexturePacks(
            activeGame,
            modSourceName,
            modName,
            attachedTexturePacks.map((p) => p.name),
          );
        }
        return true;
      },
    },
  ]);
}

export async function setupDecompileModJob(
  activeGame: SupportedGame,
  modName: string,
  modSourceName: string,
) {
  // Check to see if we need to prompt for the ISO or not
  jobTracker.init([
    {
      status: "queued",
      label: $format("setup_decompile"),
      task: async () => {
        let error = await decompileForModInstall(
          activeGame,
          modName,
          modSourceName,
        );
        if (error) {
          jobTracker.updateFailureReason(error);
          return false;
        }
        return true;
      },
    },
    {
      status: "queued",
      label: $format("setup_done"),
      task: async () => {
        return true;
      },
    },
  ]);
}

export async function setupCompileModJob(
  activeGame: SupportedGame,
  modName: string,
  modSourceName: string,
) {
  // Check to see if we need to prompt for the ISO or not
  jobTracker.init([
    {
      status: "queued",
      label: $format("setup_compile"),
      task: async () => {
        let error = await compileForModInstall(
          activeGame,
          modName,
          modSourceName,
        );
        if (error) {
          jobTracker.updateFailureReason(error);
          return false;
        }
        return true;
      },
    },
    {
      status: "queued",
      label: $format("setup_done"),
      task: async () => {
        return true;
      },
    },
  ]);
}
