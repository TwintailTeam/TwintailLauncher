import React from "react";
import { emit } from "@tauri-apps/api/event";
import { DownloadIcon, Settings } from "lucide-react";
import TooltipIcon from "../common/TooltipIcon";
import GameButton from "../GameButton";
import { POPUPS } from "../popups/POPUPS";

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
  refreshDownloadButtonInfo: () => void | Promise<void>;
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
              <DownloadIcon className="text-yellow-600 hover:text-yellow-700 w-8 h-8" />
            }
          />
        </button>
      ) : null}
      {currentInstall !== "" ? (
        <button
          id={`install_settings_btn`}
          disabled={disableInstallEdit}
          onClick={() => onOpenInstallSettings()}
        >
          <TooltipIcon
            side={"top"}
            text={"Install settings"}
            icon={
              <Settings
                fill={"white"}
                className="hover:stroke-neutral-500 stroke-black w-8 h-8"
              />
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
