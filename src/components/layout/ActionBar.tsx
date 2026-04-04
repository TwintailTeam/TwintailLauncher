
import { emit } from "@tauri-apps/api/event";
import { CircleFadingArrowUp, Settings } from "lucide-react";
import TooltipIcon from "../common/TooltipIcon";
import GameButton from "../GameButton";

export type ActionBarProps = {
  currentInstall: string;
  preloadAvailable: boolean;
  disablePreload: boolean;
  disableInstallEdit: boolean;
  disableResume: boolean;
  disableDownload: boolean;
  disableRun: boolean;
  disableUpdate: boolean;
  resumeStates: any;
  globalSettings: any;
  buttonType: any;
  installSettings: any;
  gameManifest: any;
  refreshDownloadButtonInfo: (existingInstall?: boolean) => void | Promise<void>;
  onOpenInstallSettings: () => Promise<void> | void;
  isVisible?: boolean;
  isPausing?: boolean;
};

export default function ActionBar(props: ActionBarProps) {
  const {
    currentInstall,
    preloadAvailable,
    disablePreload,
    disableInstallEdit,
    disableResume,
    disableDownload,
    disableRun,
    disableUpdate,
    resumeStates,
    globalSettings,
    buttonType,
    installSettings,
    gameManifest,
    refreshDownloadButtonInfo,
    onOpenInstallSettings,
    isVisible = true,
    isPausing = false,
  } = props;
  return (
    <div
      className={`flex flex-row absolute bottom-8 right-16 gap-4 transition-all duration-200 ${isVisible ? "opacity-100 translate-y-0 pointer-events-auto animate-slideUp" : "opacity-0 translate-y-2 pointer-events-none"}`}
      style={{ animationDelay: isVisible ? "200ms" : undefined }}
      aria-hidden={!isVisible}
    >
      {currentInstall !== "" && preloadAvailable && installSettings.version == gameManifest.latest_version ? (
        <button
          className={`p-2.5 rounded-full border border-white/20 shadow-lg disabled:cursor-not-allowed disabled:brightness-75 disabled:saturate-100 transition-colors focus:outline-none
            ${buttonType === "pause"
              ? "bg-yellow-600 hover:bg-yellow-700 shadow-yellow-900/20 focus:ring-2 focus:ring-yellow-400/60"
              : buttonType === "update"
                ? "bg-green-600 hover:bg-green-700 shadow-green-900/20 focus:ring-2 focus:ring-green-400/60"
                : buttonType === "resume"
                  ? "bg-amber-600 hover:bg-amber-700 shadow-amber-900/20 focus:ring-2 focus:ring-amber-400/60"
                  : buttonType === "queued"
                    ? "bg-gray-600 hover:bg-gray-700 shadow-gray-900/20 focus:ring-2 focus:ring-gray-400/60"
                    : "bg-purple-600 hover:bg-purple-700 shadow-purple-900/20 focus:ring-2 focus:ring-purple-400/60"}`}
          disabled={disablePreload || buttonType === "queued" || buttonType === "pause"}
          onClick={() => {
            emit("start_game_preload", {
              install: currentInstall,
              biz: "",
              lang: "",
              region: ""
            }).then(() => { });
          }}>
          <TooltipIcon
            side={"top"}
            text={"Predownload update"}
            icon={<CircleFadingArrowUp className="w-8 h-8 text-white/90" />}/>
        </button>
      ) : null}
      {currentInstall !== "" ? (
        <button id={`install_settings_btn`} className={`p-2.5 rounded-full shadow-lg disabled:cursor-not-allowed disabled:brightness-75 disabled:saturate-100 transition-colors focus:outline-none 
              ${buttonType === "pause"
            ? "bg-yellow-600 hover:bg-yellow-700 border border-white/20 shadow-yellow-900/20 focus:ring-2 focus:ring-yellow-400/60"
            : buttonType === "update"
              ? "bg-green-600 hover:bg-green-700 border border-white/20 shadow-green-900/20 focus:ring-2 focus:ring-green-400/60"
              : buttonType === "resume"
                ? "bg-amber-600 hover:bg-amber-700 border border-white/20 shadow-amber-900/20 focus:ring-2 focus:ring-amber-400/60"
                : buttonType === "queued"
                  ? "bg-gray-600 hover:bg-gray-700 border border-white/20 shadow-gray-900/20 focus:ring-2 focus:ring-gray-400/60"
                  : "bg-purple-600 hover:bg-purple-700 border border-white/20 shadow-purple-900/20 focus:ring-2 focus:ring-purple-400/60"}
            `} disabled={disableInstallEdit} onClick={() => onOpenInstallSettings()}>
          <TooltipIcon side={"top"} text={"Game Settings"} icon={<Settings className="w-8 h-8 text-white" />} />
        </button>
      ) : null}
      <GameButton
        resumeStates={resumeStates}
        disableResume={disableResume}
        disableDownload={disableDownload}
        disableRun={disableRun}
        disableUpdate={disableUpdate}
        currentInstall={currentInstall}
        globalSettings={globalSettings}
        installSettings={installSettings}
        refreshDownloadButtonInfo={refreshDownloadButtonInfo}
        buttonType={buttonType}
        isPausing={isPausing}
      />
    </div>
  );
}
