 
import { emit } from "@tauri-apps/api/event";
import { DownloadIcon, Settings } from "lucide-react";
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
  refreshDownloadButtonInfo: (existingInstall?: boolean) => void | Promise<void>;
  onOpenInstallSettings: () => Promise<void> | void;
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
    refreshDownloadButtonInfo,
    onOpenInstallSettings,
  } = props;

  return (
    <div
      className="flex flex-row absolute bottom-8 right-16 gap-4 animate-slideInRight"
      style={{ animationDelay: "900ms" }}
    >
        {currentInstall !== "" && preloadAvailable ? (
          <button
            className="p-2.5 rounded-full bg-purple-500/70 hover:bg-purple-500/80 border border-white/30 shadow-lg shadow-purple-900/20 disabled:cursor-not-allowed disabled:brightness-75 disabled:saturate-100 disabled:hover:bg-purple-500/70 transition-colors"
            disabled={disablePreload}
            onClick={() => {
              emit("start_game_preload", {
                install: currentInstall,
                biz: "",
                lang: "",
              }).then(() => {});
            }}
          >
            <TooltipIcon
              side={"top"}
              text={"Predownload update"}
              icon={
                <DownloadIcon className="w-8 h-8 text-white/90" />
              }
            />
          </button>
        ) : null}
        {currentInstall !== "" ? (
          <button
            id={`install_settings_btn`}
            className={`p-2.5 rounded-full shadow-lg disabled:cursor-not-allowed disabled:brightness-75 disabled:saturate-100 transition-colors focus:outline-none 
              ${buttonType === "update"
                ? "bg-green-600 hover:bg-green-700 border border-white/30 shadow-green-900/20 focus:ring-2 focus:ring-green-400/60"
                : buttonType === "resume"
                ? "bg-amber-600 hover:bg-amber-700 border border-white/30 shadow-amber-900/20 focus:ring-2 focus:ring-amber-400/60"
                : "bg-purple-600 hover:bg-purple-700 border border-white/30 shadow-purple-900/20 focus:ring-2 focus:ring-purple-400/60"}
            `}
            disabled={disableInstallEdit}
            onClick={() => onOpenInstallSettings()}
          >
            <TooltipIcon
              side={"top"}
              text={"Install settings"}
              icon={
                <Settings className="w-8 h-8 text-white" />
              }
            />
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
          refreshDownloadButtonInfo={refreshDownloadButtonInfo}
          buttonType={buttonType}
        />
    </div>
  );
}
