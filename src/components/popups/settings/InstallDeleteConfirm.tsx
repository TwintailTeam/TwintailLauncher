import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft, Trash2Icon} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";
import CheckBox from "../../common/CheckBox.tsx";

interface IProps {
    games: any,
    install: any,
    setOpenPopup: (popup: POPUPS) => void,
    pushInstalls: () => void,
    setCurrentInstall: (id: string) => void,
    setCurrentGame: (id: string) => void,
    setBackground: (id: string) => void
}

export default function InstallDeleteConfirm({setOpenPopup, install, games, pushInstalls, setCurrentGame, setCurrentInstall, setBackground}: IProps) {
    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <div className="flex flex-row items-center gap-2">
                    <ArrowLeft className="text-white cursor-pointer" onClick={() => {
                        setOpenPopup(POPUPS.INSTALLSETTINGS);
                    }}/>
                    <h1 className="text-white font-bold text-2xl">Confirm your action</h1>
                </div>
                <div className="flex flex-row-reverse left-3">
                    <button className="flex flex-row gap-1 items-center p-2 bg-red-600 rounded-lg" onClick={() => {
                        setOpenPopup(POPUPS.NONE);
                        let wpd = false;

                        let prefixtoggle = document.getElementById("uninstall_prefix_delete");
                        if (prefixtoggle !== null) {
                            // @ts-ignore
                            wpd = prefixtoggle.checked;
                        }

                        invoke("remove_install", {id: install.id, wipePrefix: wpd}).then(r => {
                            if (r) {
                                pushInstalls();
                                setCurrentInstall("");
                                setCurrentGame(games[0].biz);
                                setBackground(games[0].assets.game_background);
                                // @ts-ignore
                                document.getElementById(games[0].biz).focus();
                            } else {
                                console.error("Uninstall error!");
                            }
                        });
                    }}><Trash2Icon/><span className="font-semibold translate-y-px">Yes, uninstall</span>
                    </button>
                </div>
            </div>
            <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    <p className={"text-white text-start self-start"}>
                        Are you sure you want to uninstall and remove <span className={"text-blue-500 font-bold"}>{install.name}</span> installation?<br/>
                        <span className={"text-red-500 font-extrabold"}>This is irreversible action ENTIRE installation will be wiped and can not be undone!</span><br/>
                        This action will <span className={"text-red-500 font-bold"}>NOT</span> remove:
                        {(window.navigator.platform.includes("Linux")) ? <li className={"list-none"}>- <span className={"text-blue-500"}>Wine prefix</span> associated with this installation (Can be deleted with checkbox below)</li> : null}
                        {(window.navigator.platform.includes("Linux")) ? <li className={"list-none"}>- <span className={"text-blue-500"}>Wine runner</span> used with this installation</li> : null}
                        {(window.navigator.platform.includes("Linux")) ? <li className={"list-none"}>- <span className={"text-blue-500"}>DXVK</span> used with this installation</li> : null}
                        <li className={"list-none"}>- <span className={"text-blue-500"}>Any tweaks</span> enabled for this installation</li>
                    </p>
                {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={false} name={"Delete associated wine prefix"} id={"uninstall_prefix_delete"} helpText={"Enabling this will delete wine prefix associated with this game installation/"}/> : null}
            </div>
            </div>
    )
}
