import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft} from "lucide-react";
import CheckBox from "../../common/CheckBox.tsx";
import {useState} from "react";
import SelectMenu from "../../common/SelectMenu.tsx";

interface IProps {
    install: any,
    gameSwitches: any,
    gameFps: any,
    fetchInstallSettings: (id: string) => void,
    setOpenPopup: (popup: POPUPS) => void
}

export default function FpsUnlockSettings({setOpenPopup, install, gameSwitches, gameFps, fetchInstallSettings}: IProps) {
    const [isClosing, setIsClosing] = useState(false);

    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setOpenPopup(POPUPS.INSTALLSETTINGS);
    }, 220);
    };

    return (
        <div className={`rounded-xl w-[90vw] max-w-xl max-h-[75vh] bg-zinc-900 border border-white/20 flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out`}>
            <div className="flex flex-row items-center justify-between">
                <div className="flex flex-row items-center gap-4 mb-6">
                    <ArrowLeft className="text-gray-400 hover:text-white hover:bg-white/10 rounded-lg p-3 w-12 h-12 transition-all duration-200 cursor-pointer" onClick={handleClose}/>
                    <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-red-200 bg-clip-text text-transparent">FPS Unlocker settings</h1>
                </div>
            </div>
            <div className="bg-zinc-900/60 border border-white/20 rounded-xl p-6 flex flex-col gap-2 shadow-inner">
                {(gameSwitches.fps_unlocker) ? <CheckBox enabled={install.use_fps_unlock} name={"Enable FPS Unlocker"} id={"tweak_fps_unlock"} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Load and inject fps unlocking into the game. Pick FPS in the menu bellow."}/> : null}
                {(gameSwitches.fps_unlocker) ? <SelectMenu id={"install_fps_value"} name={"FPS value"} multiple={false} options={gameFps} selected={`${install.fps_value}`} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Target FPS to unlock game to."} setOpenPopup={setOpenPopup}/> : null}
            </div>
            <div className="flex justify-center pt-4 border-t border-white/10"></div>
        </div>
    )
}
