import {X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";

export default function SettingsInstall({setOpenPopup, displayName, install}: {install: string, displayName: any, setOpenPopup: (popup: POPUPS) => void}) {
    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">{displayName}</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    <p>{install}</p>
                </div>
            </div>
    )
}
