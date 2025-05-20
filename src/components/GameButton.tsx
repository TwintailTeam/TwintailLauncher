import {DownloadIcon, HardDriveDownloadIcon, Rocket} from "lucide-react";
import {emit} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api/core";

interface IProps {
    currentInstall: any,
    globalSettings: any,
    buttonType: string,
    refreshDownloadButtonInfo: () => void
}
export default function GameButton({currentInstall, globalSettings, buttonType, refreshDownloadButtonInfo}: IProps) {
    return (
        <>
            {buttonType === "launch" && (
                <button id={`launch_game_btn`} className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg disabled:bg-gray-500 hover:bg-blue-700" onClick={() => {
                    setTimeout(() => {
                        invoke("game_launch", {id: currentInstall}).then((r: any) => {
                            if (r) {
                                // @ts-ignore
                                document.getElementById(`${currentInstall}`).focus();
                                switch (globalSettings.launcher_action) {
                                    case "exit": {
                                        setTimeout(() => { emit("launcher_action_exit", null).then(() => {}); }, 500);
                                    }
                                        break;
                                    case "minimize": {
                                        setTimeout(() => { emit("launcher_action_minimize", null).then(() => {}); }, 500);
                                    }
                                        break;
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
                                            lt.innerText = `Launch!`;
                                        }, 10000);
                                    }
                                        break;
                                }
                            } else {
                                console.error("Launch error!");
                            }
                        })
                    }, 20);
                }}><Rocket/><span id={"launch_game_txt"} className="font-semibold translate-y-px">Launch!</span>
                </button>
            )}
            {buttonType === "download" && (
            <button id={"download_game_btn"} className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg hover:bg-blue-700" onClick={() => {
                refreshDownloadButtonInfo();
            }}><HardDriveDownloadIcon/><span className="font-semibold translate-y-px">Download</span>
            </button>
            )}
            {buttonType === "update" && (
                <button id={"update_game_btn"} className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg hover:bg-blue-700" onClick={() => {
                    console.log("update...");
                }}><DownloadIcon/><span className="font-semibold translate-y-px">Update</span>
                </button>
            )}
        </>
    )
}