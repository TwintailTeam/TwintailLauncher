import {EyeOffIcon, WrenchIcon, X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import CheckBox from "../../common/CheckBox.tsx";
import FolderInput from "../../common/FolderInput.tsx";
import SelectMenu from "../../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import TTLVersion from "../../common/TTLVersion.tsx";
import {useState} from "react";

export default function SettingsGlobal({setOpenPopup, settings, fetchSettings}: {settings: any, fetchSettings: () => void, setOpenPopup: (popup: POPUPS) => void}) {
    const [isClosing, setIsClosing] = useState(false);

    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setOpenPopup(POPUPS.NONE);
    }, 220);
    };

    return (
        <div className={`rounded-xl w-[90vw] max-w-4xl max-h-[85vh] bg-black/50 border border-white/20 flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out transition-all`}>
            <div className="flex flex-row items-center justify-between mb-6">
                <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-orange-200 bg-clip-text text-transparent">Settings</h1>
                <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-12 h-12 transition-all duration-200 cursor-pointer" onClick={handleClose}/>
            </div>
            <div className="w-full overflow-y-auto overflow-x-hidden hover-scrollbar pr-4 -mr-4 flex-1">
                <div className="p-6 flex flex-col gap-5">
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
            <div className="flex justify-center gap-3 pt-6 mt-4 border-t border-white/10">
                <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-orange-600 to-orange-700 hover:from-orange-500 hover:to-orange-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                    setOpenPopup(POPUPS.NONE);
                    invoke("update_extras").then(() => {});
                }}><WrenchIcon/><span>Update extras</span>
                </button>
            {window.navigator.platform.includes("Linux") ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        setOpenPopup(POPUPS.NONE);
                        invoke("block_telemetry_cmd").then(() => {});
                    }}><EyeOffIcon/><span>Block telemetry</span>
                    </button>
                : null}
            </div>
            <div className={"text-center text-white mt-4"}>
                <TTLVersion/>
            </div>
        </div>
    )
}
