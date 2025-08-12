import {invoke} from "@tauri-apps/api/core";
import HelpTooltip from "./HelpTooltip.tsx";
import {POPUPS} from "../popups/POPUPS.ts";


export default function SelectMenu({ id, name, options, selected, multiple, install, biz, lang, version, dir, fetchInstallSettings, fetchSettings, fetchDownloadSizes, helpText, setOpenPopup, skipGameDownload}: { id: string, name: string, options: any, selected: any, multiple: boolean, install?: string, biz?: string, lang?: () => string, version?: () => any, dir?: () => string, helpText: string, fetchInstallSettings?: (id: string) => void, fetchSettings?: () => void, fetchDownloadSizes?: (biz: any, version: any, lang: any, dir: any, callback: (data: any) => void) => void, setOpenPopup: (popup: POPUPS) => void, skipGameDownload?: boolean }) {
    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm flex items-center gap-1">{name}
                <HelpTooltip text={helpText}/>
            </span>
            <div className="inline-flex flex-row items-center justify-center">
                <select defaultValue={(selected === "") ? "" : selected} id={id} multiple={multiple} className={"w-full focus:outline-none h-8 rounded-lg bg-white/20 text-white px-2 pr-32 placeholder-white/50 appearance-none cursor-pointer"} onChange={(e) => {
                    switch (id) {
                        case "game_version": {
                            if (fetchDownloadSizes !== undefined && dir !== undefined && lang !== undefined) {
                                fetchDownloadSizes(biz, `${e.target.value}`, lang(), dir(), (disk) => {
                                    // @ts-ignore
                                    let btn = document.getElementById("game_dl_btn");
                                    // @ts-ignore
                                    let freedisk = document.getElementById("game_disk_free");

                                    // Skip space validation if existing installation is selected
                                    if (skipGameDownload || disk.game_decompressed_size_raw <= disk.free_disk_space_raw) {
                                        // @ts-ignore
                                        btn.removeAttribute("disabled");
                                        // @ts-ignore
                                        freedisk.classList.remove("text-red-600");
                                        // @ts-ignore
                                        freedisk.classList.add("text-white");
                                        // @ts-ignore
                                        freedisk.classList.remove("font-bold");
                                    } else {
                                        // @ts-ignore
                                        btn.setAttribute("disabled", "");
                                        // @ts-ignore
                                        freedisk.classList.add("text-red-600");
                                        // @ts-ignore
                                        freedisk.classList.remove("text-white");
                                        // @ts-ignore
                                        freedisk.classList.add("font-bold");
                                    }
                                });
                            }
                        }
                        break;
                        case "game_audio_langs": {
                            if (fetchDownloadSizes !== undefined && dir !== undefined && version !== undefined) {
                                fetchDownloadSizes(biz, version(), `${e.target.value}`, dir(), (disk) => {
                                    // @ts-ignore
                                    let btn = document.getElementById("game_dl_btn");
                                    // @ts-ignore
                                    let freedisk = document.getElementById("game_disk_free");

                                    // Skip space validation if existing installation is selected
                                    if (skipGameDownload || disk.game_decompressed_size_raw <= disk.free_disk_space_raw) {
                                        // @ts-ignore
                                        btn.removeAttribute("disabled");
                                        // @ts-ignore
                                        freedisk.classList.remove("text-red-600");
                                        // @ts-ignore
                                        freedisk.classList.add("text-white");
                                        // @ts-ignore
                                        freedisk.classList.remove("font-bold");
                                    } else {
                                        // @ts-ignore
                                        btn.setAttribute("disabled", "");
                                        // @ts-ignore
                                        freedisk.classList.add("text-red-600");
                                        // @ts-ignore
                                        freedisk.classList.remove("text-white");
                                        // @ts-ignore
                                        freedisk.classList.add("font-bold");
                                    }
                                });
                            }
                        }
                            break;
                        case "launcher_action": {
                            if (fetchSettings !== undefined) {
                                invoke("update_settings_launcher_action", {action: `${e.target.value}`}).then(() => {
                                    fetchSettings()
                                });
                            }
                        }
                        break;
                        case "install_fps_value": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_fps_value", {fps: `${e.target.value}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                        case "install_runner_version": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_runner_version", {version: `${e.target.value}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string);
                                    setOpenPopup(POPUPS.NONE);
                                });
                            }
                        }
                        break;
                        case "install_dxvk_version": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_dxvk_version", {version: `${e.target.value}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string);
                                    setOpenPopup(POPUPS.NONE);
                                });
                            }
                        }
                        break;
                    }
                }}>
                    {options.map((option: any) => (
                        <option key={option.value} value={option.value}>{option.name}</option>
                    ))}
                </select>
            </div>
        </div>
    )
}
