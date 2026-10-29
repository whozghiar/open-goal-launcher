interface VersionInfo {
  installed: boolean;
  installedVersion: string | undefined;
}

interface VersionState {
  activeToolingVersion: string | undefined | null;
  activeLevelEditorVersion: string | undefined | null;
  activeAnimationStudioVersion: string | undefined | null;
  activeModVersionInfo: VersionInfo;
  displayModVersion: boolean;
}

export type ActiveVersionKey =
  | "activeToolingVersion"
  | "activeLevelEditorVersion"
  | "activeAnimationStudioVersion";

export const versionState: VersionState = $state({
  activeToolingVersion: undefined,
  activeLevelEditorVersion: undefined,
  activeAnimationStudioVersion: undefined,
  activeModVersionInfo: {
    installed: true,
    installedVersion: undefined,
  },
  displayModVersion: false,
});
