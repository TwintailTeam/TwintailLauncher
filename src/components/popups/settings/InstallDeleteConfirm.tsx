import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft, Trash2Icon} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";
import CheckBox from "../../common/CheckBox.tsx";
import {useState} from "react";

interface IProps {
    games: any,
    installs: any,
    install: any,
    setOpenPopup: (popup: POPUPS) => void,
    pushInstalls: () => void,
    setCurrentInstall: (id: string) => void,
    setCurrentGame: (id: string) => void,
    setBackground: (id: string) => void
}

export default function InstallDeleteConfirm({setOpenPopup, install, games, installs, pushInstalls, setCurrentGame, setCurrentInstall, setBackground}: IProps) {
    const [isClosing, setIsClosing] = useState(false);

    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setOpenPopup(POPUPS.NONE);
    }, 220);
    };

    return (
        <div className={`rounded-xl h-auto w-1/3 bg-gradient-to-br from-black/80 via-black/70 to-black/60 backdrop-blur-xl border border-white/30 shadow-2xl shadow-red-500/20 flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out`}>
            <div className="flex flex-row items-center justify-between">
                <div className="flex flex-row items-center gap-4">
                    <ArrowLeft className="text-gray-400 hover:text-white hover:bg-white/10 rounded-lg p-2 transition-all duration-200 cursor-pointer" onClick={handleClose}/>
                    <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-red-200 bg-clip-text text-transparent">Confirm your action</h1>
                </div>
            </div>
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-sm border border-white/20 rounded-xl p-6 flex flex-col gap-5 shadow-inner">
                <p className={"text-white text-start"}>
                    Are you sure you want to uninstall and remove <span className={"text-purple-400 font-bold"}>{install.name}</span> installation?<br/>
                    <span className={"text-red-500 font-extrabold"}>This is irreversible action ENTIRE installation will be wiped and can not be undone!</span><br/>
                    This action will <span className={"text-red-500 font-bold"}>NOT</span> remove:
                    {(window.navigator.platform.includes("Linux")) ? <li className={"list-none ml-4"}>- <span className={"text-purple-400"}>Wine prefix</span> associated with this installation (Can be deleted with checkbox below)</li> : null}
                    {(window.navigator.platform.includes("Linux")) ? <li className={"list-none ml-4"}>- <span className={"text-purple-400"}>Wine runner</span> used with this installation</li> : null}
                    {(window.navigator.platform.includes("Linux")) ? <li className={"list-none ml-4"}>- <span className={"text-purple-400"}>DXVK</span> used with this installation</li> : null}
                    <li className={"list-none ml-4"}>- <span className={"text-purple-400"}>Any tweaks</span> enabled for this installation</li>
                </p>
                {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={false} name={"Delete associated wine prefix"} id={"uninstall_prefix_delete"} helpText={"Enabling this will delete wine prefix associated with this game installation."}/> : null}
            </div>
            <div className="flex justify-center pt-4 border-t border-white/10">
                <button className="flex flex-row gap-3 items-center py-3 px-8 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-xl shadow-lg shadow-red-500/30 transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                    setOpenPopup(POPUPS.NONE);
                    let wpd = false;

                    let prefixtoggle = document.getElementById("uninstall_prefix_delete");
                    if (prefixtoggle !== null) {
                        // @ts-ignore
                        wpd = prefixtoggle.checked;
                    }

                    // @ts-ignore
                    document.getElementById(`${install.id}`).setAttribute("disabled", "");

                    invoke("remove_install", {id: install.id, wipePrefix: wpd}).then(r => {
                        if (r) {
                            pushInstalls();
                            if (installs.length === 0) {
                                setCurrentInstall("");
                                setCurrentGame(games[0].biz);
                                setBackground(games[0].assets.game_background);
                                // @ts-ignore
                                document.getElementById(games[0].biz).focus();
                            } else {
                                setCurrentInstall(installs[0].id);
                                setCurrentGame(games[0].biz);
                                setBackground(installs[0].game_background);
                                // @ts-ignore
                                document.getElementById(installs[0].id).focus();
                            }
                        } else {
                            console.error("Uninstall error!");
                        }
                    });
                }}><Trash2Icon/><span>Yes, uninstall</span>
                </button>
            </div>
        </div>
    )
}
