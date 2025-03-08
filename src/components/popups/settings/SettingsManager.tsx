import {X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import CheckBox from "./CheckBox.tsx";
import FolderInput from "./FolderInput.tsx";
import TextInput from "./TextInput.tsx";

export default function SettingsManager({setOpenPopup}: {setOpenPopup: (popup: POPUPS) => void}) {
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
                    <CheckBox enabled={false} name={"Auto update 3rd party repositories"}/>
                    <FolderInput name={"Default game install location"} clearable={true} folder={true} id={"default_game_path"}/>
                    <FolderInput name={"XXMI location"} clearable={true} folder={true} id={"default_xxmi_path"}/>
                    <FolderInput name={"FPS Unlocker location"} clearable={true} folder={true} id={"default_fpsunlock_path"}/>
                    <FolderInput name={"Jadeite location"} clearable={true} folder={true} id={"default_jadeite_path"}/>
                    <TextInput name={"Testtt"} value={""} readOnly={false}/>
                </div>
            </div>
    )
}
