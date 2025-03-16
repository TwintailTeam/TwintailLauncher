import {X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import FolderInput from "../../common/FolderInput.tsx";
import CheckBox from "../../common/CheckBox.tsx";
import TextInput from "../../common/TextInput.tsx";

export default function SettingsInstall({setOpenPopup, installSettings}: {installSettings: any, setOpenPopup: (popup: POPUPS) => void}) {
    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">{installSettings.name}</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    <FolderInput name={"Install location"} clearable={true} value={`${installSettings.directory}`} folder={true} id={"install_game_path2"}/>
                    <CheckBox enabled={installSettings.ignore_updates} name={"Skip version update check"} id={"skip_version_updates2"}/>
                    <CheckBox enabled={installSettings.skip_hash_check} name={"Skip hash validation"} id={"skip_hash_validation2"}/>
                    <CheckBox enabled={installSettings.use_jadeite} name={"Inject Jadeite"} id={"tweak_jadeite"}/>
                    <CheckBox enabled={installSettings.use_xxmi} name={"Inject XXMI"} id={"tweak_xxmi"}/>
                    <CheckBox enabled={installSettings.use_fps_unlock} name={"Inject FPS Unlocker"} id={"tweak_fps_unlock"}/>
                    <TextInput name={"Environment variables"} value={installSettings.env_vars} readOnly={false} id={"install_env_vars"} placeholder={"DXVK_HUD=fps;DXVK_LOG=none;"}/>
                    <TextInput name={"Pre launch command"} value={installSettings.pre_launch_command} readOnly={false} id={"install_pre_launch_cmd"} placeholder={"%command%"}/>
                    <TextInput name={"Launch command"} value={installSettings.launch_command} readOnly={false} id={"install_launch_cmd"} placeholder={"%command%"}/>
                </div>
            </div>
    )
}
