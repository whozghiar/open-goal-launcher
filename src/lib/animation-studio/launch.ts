import { unwrapFunctionStore, format } from "svelte-i18n";
import { launchAnimationStudio } from "$lib/rpc/features";
import { toastStore } from "$lib/stores/ToastStore";
import { navigate } from "/src/router";
import { config } from "/src/state/config.svelte";

const $format = unwrapFunctionStore(format);

// OpenGOAL Animation Studio is a program of its own, downloaded per version like Jak Level Editor:
// without an active version, its page of the settings offers them
export async function openAnimationStudio() {
  if (!config?.animationStudioVersion) {
    navigate("/settings/:tab", { params: { tab: "animation-studio" } });
    return;
  }
  if (!(await launchAnimationStudio())) {
    toastStore.makeToast($format("toasts_animationStudioAlreadyOpen"), "info");
  }
}
