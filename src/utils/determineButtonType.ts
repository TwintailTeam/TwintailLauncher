export type ButtonType = "download" | "update" | "launch" | "resume";

interface DetermineParams {
  currentInstall: any;
  installSettings: any;
  gameManifest: any;
  preloadAvailable: boolean;
  resumeStates: { updating?: boolean; downloading?: boolean; preloading?: boolean; repairing?: boolean };
}

export function determineButtonType({
  currentInstall,
  installSettings,
  gameManifest,
  preloadAvailable,
  resumeStates,
}: DetermineParams): ButtonType {
  let buttonType: ButtonType = "download";
  const hasResume = !!(resumeStates?.updating || resumeStates?.downloading || resumeStates?.preloading || resumeStates?.repairing);

  if (!currentInstall) return "download";
  if (!installSettings || !gameManifest) return hasResume ? "resume" : "launch";

  const isUpdateNeeded = (installSettings.version !== gameManifest.latest_version)
    && !preloadAvailable
    && !installSettings.ignore_updates;

  if (isUpdateNeeded) {
    if (gameManifest.latest_version !== null) {
      return hasResume ? "resume" : "update";
    } else {
      buttonType = hasResume ? "resume" : "launch";
    }
  } else {
    buttonType = hasResume ? "resume" : "launch";
  }
  return buttonType;
}
