import { useEffect, useRef, useState } from "react";
import { DownloadIcon, HardDriveDownloadIcon, RefreshCcwIcon, Play, PauseIcon, Clock, FolderOpen } from "lucide-react";
import { emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type GameStatus = "idle" | "preparing" | "launching" | "running";

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
    installSettings: any,
    isPausing?: boolean
}
export default function GameButton({ currentInstall, globalSettings, buttonType, refreshDownloadButtonInfo, disableUpdate, disableRun, disableDownload, disableResume, resumeStates, installSettings, isPausing = false }: IProps) {
    const [gameStatus, setGameStatus] = useState<GameStatus>("idle");
    const pollIntervalRef = useRef<number | null>(null);
    const timeoutRef = useRef<number | null>(null);
    const wasRunningRef = useRef<boolean>(false);

    // Cleanup polling and timeout on unmount or when currentInstall changes
    useEffect(() => {
        return () => {
            if (pollIntervalRef.current !== null) {
                clearInterval(pollIntervalRef.current);
                pollIntervalRef.current = null;
            }
            if (timeoutRef.current !== null) {
                clearTimeout(timeoutRef.current);
                timeoutRef.current = null;
            }
        };
    }, [currentInstall]);

    // Reset game status when switching installs
    useEffect(() => {
        setGameStatus("idle");
        wasRunningRef.current = false;
        if (pollIntervalRef.current !== null) {
            clearInterval(pollIntervalRef.current);
            pollIntervalRef.current = null;
        }
        if (timeoutRef.current !== null) {
            clearTimeout(timeoutRef.current);
            timeoutRef.current = null;
        }

        if (!currentInstall) {
            return;
        }

        // Immediately check if the game is already running for this install.
        // If it is, mark running and start polling to track when it stops.
        (async () => {
            try {
                const status = await invoke<string | null>("check_game_running", { id: currentInstall });
                if (status === "running") {
                    wasRunningRef.current = true;
                    setGameStatus("running");
                    startGameStatusPolling();
                } else if (status === "preparing") {
                    setGameStatus("preparing");
                    startGameStatusPolling();
                }
            } catch (e) {}
        })();
    }, [currentInstall]);

    const stopPolling = () => {
        if (pollIntervalRef.current !== null) {
            clearInterval(pollIntervalRef.current);
            pollIntervalRef.current = null;
        }
        if (timeoutRef.current !== null) {
            clearTimeout(timeoutRef.current);
            timeoutRef.current = null;
        }
    };

    const startGameStatusPolling = () => {
        // Clear any existing interval/timeout
        stopPolling();

        // Fallback timeout: if game is never detected after 30 seconds, reset to idle
        // This handles cases where process detection doesn't work (e.g., some Flatpak scenarios)
        timeoutRef.current = window.setTimeout(() => {
            if (!wasRunningRef.current) {
                setGameStatus("idle");
                stopPolling();
            }
        }, 30000);

        // Poll every 2 seconds
        pollIntervalRef.current = window.setInterval(async () => {
            try {
                const status = await invoke<string | null>("check_game_running", { id: currentInstall });

                if (status === "preparing") {
                    setGameStatus("preparing");
                    // Clear the fallback timeout since winetricks is active
                    if (timeoutRef.current !== null) {
                        clearTimeout(timeoutRef.current);
                        timeoutRef.current = null;
                    }
                } else if (status === "running") {
                    wasRunningRef.current = true;
                    setGameStatus("running");
                    // Clear the fallback timeout since we detected the game
                    if (timeoutRef.current !== null) {
                        clearTimeout(timeoutRef.current);
                        timeoutRef.current = null;
                    }
                } else if (wasRunningRef.current) {
                    // Game was running but is no longer running - reset to idle
                    wasRunningRef.current = false;
                    setGameStatus("idle");
                    stopPolling();
                }
                // If status is "idle"/null and game was never running, keep "launching" state
                // This handles the delay between spawn and actual process start
            } catch {
                // On error, keep current state
            }
        }, 2000);
    };
    // Compute theme classes and behavior by buttonType
    const theme = (() => {
        switch (buttonType) {
            case "download":
                return { bg: "bg-blue-600 hover:bg-blue-700", border: "border-blue-500", ring: "focus:ring-blue-400/60", shadow: "shadow-blue-900/30", id: "download_game_btn" };
            case "update":
                return { bg: "bg-green-600 hover:bg-green-700", border: "border-green-500", ring: "focus:ring-green-400/60", shadow: "shadow-green-900/30", id: "update_game_btn" };
            case "resume":
                return { bg: "bg-amber-600 hover:bg-amber-700", border: "border-amber-500", ring: "focus:ring-amber-400/60", shadow: "shadow-amber-900/30", id: "resume_btn" };
            case "pause":
                return { bg: "bg-yellow-600 hover:bg-yellow-700", border: "border-yellow-500", ring: "focus:ring-yellow-400/60", shadow: "shadow-yellow-900/30", id: "pause_btn" };
            case "queued":
                return { bg: "bg-gray-600 hover:bg-gray-700", border: "border-gray-500", ring: "focus:ring-gray-400/60", shadow: "shadow-gray-900/30", id: "queued_btn" };
            case "launch":
            default:
                return { bg: "bg-purple-600 hover:bg-purple-700", border: "border-purple-500", ring: "focus:ring-purple-400/60", shadow: "shadow-purple-900/30", id: "launch_game_btn" };
        }
    })();

    const disabled = buttonType === "launch" ? (disableRun || gameStatus !== "idle")
        : buttonType === "download" ? disableDownload
            : buttonType === "update" ? disableUpdate
                : buttonType === "pause" ? isPausing // Disable while pausing
                    : buttonType === "queued" ? true
                        : disableResume;

    const getLaunchLabel = (): string => {
        switch (gameStatus) {
            case "preparing": return "PREPARING...";
            case "launching": return "LAUNCHING...";
            case "running": return "RUNNING";
            default: return "PLAY";
        }
    };

    const label = buttonType === "launch" ? getLaunchLabel()
        : buttonType === "download" ? "INSTALL"
            : buttonType === "update" ? "UPDATE"
                : buttonType === "pause" ? (isPausing ? "PAUSING..." : "PAUSE")
                    : buttonType === "queued" ? "QUEUED"
                        : "RESUME";

    const Icon = buttonType === "launch" ? Play
        : buttonType === "download" ? HardDriveDownloadIcon
            : buttonType === "update" ? DownloadIcon
                : buttonType === "pause" ? PauseIcon
                    : buttonType === "queued" ? Clock
                        : RefreshCcwIcon;

    const handleClick = () => {
        if (buttonType === "launch") {
            setGameStatus("launching");
            setTimeout(() => {
                invoke("game_launch", { id: currentInstall }).then((r: any) => {
                    if (r) {
                        document.getElementById(`${currentInstall}`)?.focus();
                        switch (globalSettings.launcher_action) {
                            case "exit": {
                                // Start polling, then exit when game is detected
                                startGameStatusPolling();
                                setTimeout(() => { emit("launcher_action_exit", null).then(() => { }); }, 10000);
                            } break;
                            case "minimize": {
                                startGameStatusPolling();
                                setTimeout(() => { emit("launcher_action_minimize", null).then(() => { }); }, 500);
                            } break;
                            case 'keep': {
                                // Start polling for game status
                                startGameStatusPolling();
                            } break;
                        }
                    } else {
                        console.error("Launch error!");
                        setGameStatus("idle");
                    }
                }).catch(() => {
                    setGameStatus("idle");
                });
            }, 20);
        } else if (buttonType === "download") {
            refreshDownloadButtonInfo();
        } else if (buttonType === "update") {
            emit("start_game_update", { install: currentInstall, biz: "", lang: "", region: "" }).then(() => { });
        } else if (buttonType === "pause") {
            invoke("pause_game_download", { installId: currentInstall }).then(() => { });
        } else if (buttonType === "resume") {
            // First try to resume from paused queue (in-memory paused job)
            invoke<boolean>("queue_resume_job", { installId: currentInstall }).then((resumed) => {
                if (resumed) {
                    // Successfully resumed from paused queue
                    return;
                }
                // Fall back to disk-based resume states (app was closed during download)
                if (resumeStates.downloading) {
                    emit("start_game_download", { install: currentInstall, biz: "", lang: "", region: installSettings.region_code }).then(() => { });
                }
                if (resumeStates.updating) {
                    emit("start_game_update", { install: currentInstall, biz: "", lang: "", region: "" }).then(() => { });
                }
                if (resumeStates.preloading) {
                    emit("start_game_preload", { install: currentInstall, biz: "", lang: "", region: "" }).then(() => { });
                }
                if (resumeStates.repairing) {
                    emit("start_game_repair", { install: currentInstall, biz: "", lang: "", region: installSettings.region_code }).then(() => { });
                }
            });
        }
    };

    // Determine if shimmer should show (only for primary action buttons)
    const showShimmer = buttonType === "launch" || buttonType === "download" || buttonType === "update";

    // Determine icon fill (play and pause are filled)
    const iconFill = (buttonType === "launch" || buttonType === "pause") ? "currentColor" : "none";

    return (
        <div className="flex flex-col items-center gap-1">
            <button
                id={theme.id}
                disabled={disabled}
                onClick={handleClick}
                className={`relative overflow-hidden flex flex-row gap-3 items-center justify-center w-56 md:w-64 py-3 px-7 md:px-8 rounded-full text-white border ${theme.border} disabled:cursor-not-allowed disabled:brightness-75 disabled:saturate-100 focus:outline-none focus:ring-2 ${theme.bg} ${theme.ring} shadow-lg ${theme.shadow} transition-[background-color,box-shadow,transform] duration-300 ease-out`}
            >
                {/* Shimmer effect overlay - only for primary buttons */}
                {showShimmer && (
                    <span
                        className="pointer-events-none absolute inset-0 -translate-x-full animate-[shimmer_2s_infinite] bg-gradient-to-r from-transparent via-white/20 to-transparent"
                        aria-hidden="true"
                    />
                )}
                <Icon className="w-5 h-5 md:w-6 md:h-6 text-white/90 ml-1" fill={iconFill}/>
                <span className="font-semibold text-lg md:text-xl text-white tracking-wide">{label}</span>
            </button>
            {buttonType === "download" && (
                <button
                    type="button"
                    className="group flex items-center justify-center gap-1.5 px-4 py-1 rounded-full bg-black/40 hover:bg-black/50 text-center text-xs text-white/90 hover:text-white cursor-pointer whitespace-nowrap transition-all duration-200"
                    style={{ textShadow: '0 1px 3px rgba(0,0,0,0.8)' }}
                    onClick={() => refreshDownloadButtonInfo(true)}
                >
                    <FolderOpen className="w-3.5 h-3.5" />
                    <span className="uppercase tracking-wider group-hover:underline underline-offset-2">Use existing install</span>
                </button>
            )}
        </div>
    )
}
