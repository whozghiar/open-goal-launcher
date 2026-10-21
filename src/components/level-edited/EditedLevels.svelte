<script lang="ts">
  import IconArrowLeft from "~icons/mdi/arrow-left";
  import { Button, Spinner } from "flowbite-svelte";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { navigate, route } from "/src/router";
  import { asJobType } from "$lib/job/jobs";
  import {
    levelEditedInfo,
    levelEditedLaunch,
    type LevelEditedInfo,
  } from "$lib/rpc/features";

  let info: LevelEditedInfo | undefined = $state(undefined);
  let loaded = $state(false);

  onMount(async () => {
    info = await levelEditedInfo();
    loaded = true;
  });
</script>

<div class="flex flex-col min-h-full flex-1 bg-[#1e1e1e] p-4 gap-2">
  <div class="flex flex-row gap-2">
    <Button
      outline
      class="border-solid rounded text-white hover:dark:text-slate-900 hover:bg-white font-semibold px-2 py-2"
      onclick={() =>
        navigate(`/:game_name/`, {
          params: { game_name: route.params.game_name ?? "jak3" },
        })}
      aria-label={$_("features_backToGamePage_buttonAlt")}
    >
      <IconArrowLeft />
    </Button>
    <Button
      disabled={!info}
      class="border-solid rounded bg-orange-400 hover:bg-orange-600 text-sm text-slate-900 font-semibold px-5 py-2"
      onclick={async () => {
        if (info) {
          await openPath(info.dir);
        }
      }}>{$_("levelEdited_openFolder")}</Button
    >
  </div>
  <h2 class="font-bold">{$_("levelEdited_heading")}</h2>
  <p class="text-sm">{$_("levelEdited_description")}</p>
  {#if !loaded}
    <Spinner color="yellow" size={"12"} />
  {:else if !info || info.levels.length === 0}
    <p class="text-sm text-neutral-400">{$_("levelEdited_noLevels")}</p>
  {:else}
    {#each info.levels as level (level)}
      <div
        class="flex flex-row items-center gap-2 rounded bg-[#141414] px-4 py-2"
      >
        <p class="grow font-semibold">{level}</p>
        <Button
          class="border-solid rounded bg-[#13294b] hover:bg-[#183763] text-sm text-white font-semibold px-5 py-2"
          onclick={async () => await levelEditedLaunch(level)}
          >{$_("levelEdited_play")}</Button
        >
        <Button
          class="border-solid rounded bg-slate-900 hover:bg-slate-800 text-sm text-white font-semibold px-5 py-2"
          onclick={async () => {
            const confirmed = await confirm(
              $_("levelEdited_remove_confirmation", {
                values: { level },
              }),
              { title: "OpenGOAL Launcher", kind: "warning" },
            );
            if (confirmed) {
              navigate("/job/:job_type", {
                params: { job_type: asJobType("removeEditedLevel") },
                search: { levelName: level, returnTo: route.pathname },
              });
            }
          }}>{$_("levelEdited_remove")}</Button
        >
      </div>
    {/each}
  {/if}
</div>
