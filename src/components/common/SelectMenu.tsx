import {invoke} from "@tauri-apps/api/core";


export default function SelectMenu({ id, name, options, selected, install, biz, dir, disk, fetchInstallSettings, fetchSettings, fetchDownloadSizes}: { id: string, name: string, options: any, selected: any, install?: string, biz?: string, dir?: () => string, disk?: any, fetchInstallSettings?: (id: string) => void, fetchSettings?: () => void, fetchDownloadSizes?: (biz: any, version: any, dir: any) => void }) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm">{name}</span>
            <div className="inline-flex flex-row items-center justify-center">
                <select defaultValue={(selected === "") ? "" : selected} id={id} className={"w-full focus:outline-none h-8 rounded-lg bg-white/20 text-white px-2 pr-32 placeholder-white/50 appearance-none cursor-pointer"} onChange={(e) => {
                    switch (id) {
                        case "game_version": {
                            if (fetchDownloadSizes !== undefined && dir !== undefined) {
                                fetchDownloadSizes(biz, `${e.target.value}`, dir());

                                // @ts-ignore
                                let btn = document.getElementById("game_download_btn");
                                if (disk.free_disk_space_raw < disk.game_decompressed_size_raw) {
                                    // @ts-ignore
                                    btn.setAttribute("disabled", "");
                                } else {
                                    // @ts-ignore
                                    btn.removeAttribute("disabled");
                                }
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
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                        case "install_dxvk_version": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_dxvk_version", {version: `${e.target.value}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
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
