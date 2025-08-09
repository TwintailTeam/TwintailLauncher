import {EyeOffIcon, WrenchIcon, X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import CheckBox from "../../common/CheckBox.tsx";
import FolderInput from "../../common/FolderInput.tsx";
import SelectMenu from "../../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import TTLVersion from "../../common/TTLVersion.tsx";

export default function SettingsGlobal({setOpenPopup, settings, fetchSettings}: {settings: any, fetchSettings: () => void, setOpenPopup: (popup: POPUPS) => void}) {
    return (
        <div className="rounded-lg h-auto w-1/2 bg-black/70 fixed-backdrop-blur-md border border-white/20 flex flex-col p-6 gap-6 overflow-scroll scrollbar-none">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Settings</h1>
                <X className="text-white hover:text-gray-400 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse gap-2">
                <button className="flex flex-row gap-2 items-center py-2 px-4 bg-purple-600 hover:bg-purple-700 rounded-lg me-5" onClick={() => {
                    setOpenPopup(POPUPS.NONE);
                    invoke("update_extras").then(() => {});
                }}><WrenchIcon/><span className="font-semibold">Update extras</span>
                </button>
            {window.navigator.platform.includes("Linux") ? <button className="flex flex-row gap-2 items-center py-2 px-4 bg-orange-600 hover:bg-orange-700 rounded-lg" onClick={() => {
                        setOpenPopup(POPUPS.NONE);
                        invoke("block_telemetry_cmd").then(() => {});
                    }}><EyeOffIcon/><span className="font-semibold">Block telemetry</span>
                    </button>
                : null}
            </div>
            <div className="w-full overflow-y-auto overflow-scroll scrollbar-none pr-4 -mr-4">
                <div className="bg-black/20 border border-white/10 rounded-lg p-4 flex flex-col gap-4">
                    <CheckBox enabled={Boolean(settings.third_party_repo_updates)} name={"Auto update 3rd party repositories"} fetchSettings={fetchSettings} id={"third_party_repo_updates"} helpText={"Allow launcher to automatically update 3rd party repositories and their manifests."}/>
                    <FolderInput name={"Default game install location"} clearable={true} value={`${settings.default_game_path}`} folder={true} id={"default_game_path"} fetchSettings={fetchSettings} helpText={"Default base directory where all games will be installed."}/>
                    <FolderInput name={"XXMI location"} clearable={true} folder={true} value={`${settings.xxmi_path}`} id={"default_xxmi_path"} fetchSettings={fetchSettings} helpText={"Location where all XXMI modding tool files will be stored."}/>
                    <FolderInput name={"FPS Unlocker location"} clearable={true} folder={true} value={`${settings.fps_unlock_path}`} id={"default_fps_unlock_path"} fetchSettings={fetchSettings} helpText={"Location where fps unlocker is stored."}/>
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Jadeite location"} clearable={true} folder={true} value={`${settings.jadeite_path}`} id={"default_jadeite_path"} fetchSettings={fetchSettings} helpText={"Location where jadeite patch is stored."}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Default runner location"} clearable={true} folder={true} value={`${settings.default_runner_path}`} id={"default_runner_path"} fetchSettings={fetchSettings} helpText={"Default base directory where all Wine/Proton versions will be stored."}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Default DXVK location"} clearable={true} folder={true} value={`${settings.default_dxvk_path}`} id={"default_dxvk_path"} fetchSettings={fetchSettings} helpText={"Default base directory where all DXVK versions will be stored."}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Default runner prefix location"} clearable={true} folder={true} value={`${settings.default_runner_prefix_path}`} id={"default_prefix_path"} fetchSettings={fetchSettings} helpText={"Default base directory where all Wine/Proton prefixes will be stored."}/> : null}
                    <SelectMenu id={"launcher_action"} name={"After game launch"} multiple={false} options={[{value: "exit", name: "Close launcher"}, {value: "keep", name: "Keep launcher open"}, {value: "minimize", name: "Minimize launcher to tray"}]} selected={`${settings.launcher_action}`} fetchSettings={fetchSettings} helpText={"What will launcher do once it launches a game."} setOpenPopup={setOpenPopup}/>
                </div>
            </div>
            <div className={"text-center text-white"}>
                <TTLVersion/>
            </div>
        </div>
    )
}
