import {Trash2Icon, WrenchIcon, X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import FolderInput from "../../common/FolderInput.tsx";
import CheckBox from "../../common/CheckBox.tsx";
import TextInput from "../../common/TextInput.tsx";
import SelectMenu from "../../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";

export default function SettingsInstall({setOpenPopup, installSettings, pushInstalls, runnerVersions, dxvkVersions}: {runnerVersions: any, dxvkVersions: any, installSettings: any, setOpenPopup: (popup: POPUPS) => void, pushInstalls: () => void}) {
    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">{installSettings.name}</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-1 items-center p-2 bg-red-600 rounded-lg" onClick={() => {
                    setOpenPopup(POPUPS.NONE);
                    invoke("remove_install", {id: installSettings.id}).then(r => {
                        if (r) {
                            pushInstalls();
                        } else {
                            console.error("Uninstall error!");
                        }
                    });
                }}>
                    <Trash2Icon/>
                    <span className="font-semibold translate-y-px">Uninstall</span>
                </button>
                <button className="flex flex-row gap-1 me-2 items-center p-2 bg-blue-600 rounded-lg" onClick={() => {
                    setOpenPopup(POPUPS.NONE);
                    console.log("repair game!");
                }}>
                    <WrenchIcon/>
                    <span className="font-semibold translate-y-px">Repair install</span>
                </button>
            </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    <FolderInput name={"Install location"} clearable={true} value={`${installSettings.directory}`} folder={true} id={"install_game_path2"}/>
                    <CheckBox enabled={installSettings.ignore_updates} name={"Skip version update check"} id={"skip_version_updates2"}/>
                    <CheckBox enabled={installSettings.skip_hash_check} name={"Skip hash validation"} id={"skip_hash_validation2"}/>
                    {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={installSettings.use_jadeite} name={"Inject Jadeite"} id={"tweak_jadeite"}/> : null}
                    <CheckBox enabled={installSettings.use_xxmi} name={"Inject XXMI"} id={"tweak_xxmi"}/>
                    <CheckBox enabled={installSettings.use_fps_unlock} name={"Inject FPS Unlocker"} id={"tweak_fps_unlock"}/>
                    <SelectMenu id={"install_fps_value"} name={"FPS value"} options={[{value: "60", name: "60"}]}/>
                    <TextInput name={"Environment variables"} value={installSettings.env_vars} readOnly={false} id={"install_env_vars"} placeholder={"DXVK_HUD=fps;DXVK_LOG=none;"}/>
                    <TextInput name={"Pre launch command"} value={installSettings.pre_launch_command} readOnly={false} id={"install_pre_launch_cmd"} placeholder={"%command%"}/>
                    <TextInput name={"Launch command"} value={installSettings.launch_command} readOnly={false} id={"install_launch_cmd"} placeholder={"%command%"}/>
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_runner_version"} name={"Runner version"} options={runnerVersions}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner path"} clearable={true} value={`${installSettings.runner_path}`} folder={true} id={"install_runner_path"}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_dxvk_version"} name={"DXVK version"} options={dxvkVersions}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"DXVK path"} clearable={true} value={`${installSettings.dxvk_path}`} folder={true} id={"install_dxvk_path"}/> : null}
                </div>
            </div>
    )
}
