import {X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import CheckBox from "./CheckBox.tsx";
import FolderInput from "./FolderInput.tsx";

export default function SettingsManager({setOpenPopup, settings, fetchSettings}: {settings: any, fetchSettings: () => void, setOpenPopup: (popup: POPUPS) => void}) {
    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Settings</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse">
            </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-900 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`}
                     style={{maxHeight: (20 * 64) + "px"}}>
                    <CheckBox enabled={Boolean(settings.third_party_repo_updates)} name={"Auto update 3rd party repositories"} fetchSettings={fetchSettings}/>
                    <FolderInput name={"Default game install location"} clearable={true} value={`${settings.default_game_path}`} folder={true} id={"default_game_path"} fetchSettings={fetchSettings}/>
                    <FolderInput name={"XXMI location"} clearable={true} folder={true} value={`${settings.xxmi_path}`} id={"default_xxmi_path"} fetchSettings={fetchSettings}/>
                    <FolderInput name={"FPS Unlocker location"} clearable={true} folder={true} value={`${settings.fps_unlock_path}`} id={"default_fps_unlock_path"} fetchSettings={fetchSettings}/>
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Jadeite location"} clearable={true} folder={true} value={`${settings.jadeite_path}`} id={"default_jadeite_path"} fetchSettings={fetchSettings}/> : null}
                </div>
            </div>
    )
}
