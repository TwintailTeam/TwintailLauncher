import {DownloadIcon, HardDriveDownloadIcon, RefreshCcwIcon, Rocket} from "lucide-react";
import {emit} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api/core";

interface IProps {
    currentInstall: any,
    globalSettings: any,
    buttonType: string,
    refreshDownloadButtonInfo: (existingInstall?: boolean) => void,
    disableRun: boolean,
    disableUpdate: boolean,
    disableDownload: boolean,
    disableResume: boolean,
    resumeStates: any,
    installSettings: any
}
export default function GameButton({currentInstall, globalSettings, buttonType, refreshDownloadButtonInfo, disableUpdate, disableRun, disableDownload, disableResume, resumeStates, installSettings}: IProps) {
    // Compute theme classes and behavior by buttonType
    const theme = (() => {
        switch (buttonType) {
            case "download":
                return { bg: "bg-blue-600 hover:bg-blue-700", border: "border-blue-500", ring: "focus:ring-blue-400/60", shadow: "shadow-blue-900/30", id: "download_game_btn" };
            case "update":
                return { bg: "bg-green-600 hover:bg-green-700", border: "border-green-500", ring: "focus:ring-green-400/60", shadow: "shadow-green-900/30", id: "update_game_btn" };
            case "resume":
                return { bg: "bg-amber-600 hover:bg-amber-700", border: "border-amber-500", ring: "focus:ring-amber-400/60", shadow: "shadow-amber-900/30", id: "resume_btn" };
            case "launch":
            default:
                return { bg: "bg-purple-600 hover:bg-purple-700", border: "border-purple-500", ring: "focus:ring-purple-400/60", shadow: "shadow-purple-900/30", id: "launch_game_btn" };
        }
    })();

    const disabled = buttonType === "launch" ? disableRun
        : buttonType === "download" ? disableDownload
        : buttonType === "update" ? disableUpdate
        : disableResume;

    const label = buttonType === "launch" ? "Play!"
        : buttonType === "download" ? "Download"
        : buttonType === "update" ? "Update"
        : "Resume";

    const Icon = buttonType === "launch" ? Rocket
        : buttonType === "download" ? HardDriveDownloadIcon
        : buttonType === "update" ? DownloadIcon
        : RefreshCcwIcon;

    const handleClick = () => {
        if (buttonType === "launch") {
            setTimeout(() => {
                invoke("game_launch", {id: currentInstall}).then((r: any) => {
                    if (r) {
                        // @ts-ignore
                        document.getElementById(`${currentInstall}`).focus();
                        switch (globalSettings.launcher_action) {
                            case "exit": {
                                setTimeout(() => { emit("launcher_action_exit", null).then(() => {}); }, 3000);
                            } break;
                            case "minimize": {
                                setTimeout(() => { emit("launcher_action_minimize", null).then(() => {}); }, 500);
                            } break;
                            case 'keep': {
                                let lb = document.getElementById("launch_game_btn");
                                let lt = document.getElementById("launch_game_txt");
                                if (lb !== null && lt !== null) {
                                    lb.setAttribute("disabled", "");
                                    lt.innerText = `Launching...`;
                                }
                                setTimeout(() => {
                                    // @ts-ignore
                                    lb.removeAttribute("disabled");
                                    // @ts-ignore
                                    lt.innerText = `Play!`;
                                }, 20000);
                            } break;
                        }
                    } else {
                        console.error("Launch error!");
                    }
                })
            }, 20);
        } else if (buttonType === "download") {
            refreshDownloadButtonInfo();
        } else if (buttonType === "update") {
            emit("start_game_update", {install: currentInstall, biz: "", lang: "", region: ""}).then(() => {});
        } else if (buttonType === "resume") {
            if (resumeStates.downloading) {
                emit("start_game_download", {install: currentInstall, biz: "", lang: "", region: installSettings.region_code}).then(() => {});
            }
            if (resumeStates.updating) {
                emit("start_game_update", {install: currentInstall, biz: "", lang: "", region: ""}).then(() => {});
            }
            if (resumeStates.preloading) {
                emit("start_game_preload", {install: currentInstall, biz: "", lang: "", region: ""}).then(() => {});
            }
            if (resumeStates.repairing) {
                emit("start_game_repair", {install: currentInstall, biz: "", lang: "", region: installSettings.region_code}).then(() => {});
            }
        }
    };

    return (
        <div className="flex flex-col items-center gap-1">
            <button
                id={theme.id}
                disabled={disabled}
                onClick={handleClick}
                className={`flex flex-row gap-3 items-center justify-center w-56 md:w-64 py-3 px-7 md:px-8 rounded-full text-white border ${theme.border} disabled:cursor-not-allowed disabled:brightness-75 disabled:saturate-100 focus:outline-none focus:ring-2 ${theme.bg} ${theme.ring} shadow-lg ${theme.shadow} transition-[background-color,box-shadow,transform] duration-300 ease-out`}
            >
                <Icon className="w-5 h-5 md:w-6 md:h-6 text-white/90"/>
                <span id={buttonType === "launch" ? "launch_game_txt" : undefined} className="font-semibold translate-y-px text-base md:text-lg text-white">{label}</span>
            </button>
            {buttonType === "download" && (
                <button
                    type="button"
                    className="w-56 md:w-64 text-center text-sm text-blue-300 cursor-pointer tw-text-shadow-custom-link-wide whitespace-nowrap"
                    onClick={() => refreshDownloadButtonInfo(true)}
                >
                    Already installed? Use existing installation
                </button>
            )}
        </div>
    )
}