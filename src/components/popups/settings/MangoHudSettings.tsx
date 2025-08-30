import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft} from "lucide-react";
import CheckBox from "../../common/CheckBox.tsx";
import {useState} from "react";
import FolderInput from "../../common/FolderInput.tsx";

interface IProps {
    install: any,
    gameSwitches: any,
    fetchInstallSettings: (id: string) => void,
    setOpenPopup: (popup: POPUPS) => void
}

export default function MangoHudSettings({setOpenPopup, install, fetchInstallSettings}: IProps) {
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
                    <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-red-200 bg-clip-text text-transparent">MangoHUD settings</h1>
                </div>
            </div>
            <div className="bg-zinc-900/60 border border-white/20 rounded-xl p-6 flex flex-col gap-2 shadow-inner">
                {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={install.use_mangohud} name={"Enable MangoHUD"} id={"tweak_mangohud"} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Enable MangoHUD monitor. You need it installed on your system for this to work!"}/> : null}
                {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Config location"} extensions={["conf"]} folder={false} clearable={true} value={`${install.mangohud_config_path}`} id={"mangohud_config_path"} fetchInstallSettings={fetchInstallSettings} install={install.id} setOpenPopup={setOpenPopup} helpText={"What MangoHUD configuration will be loaded"}/> : null}
            </div>
            <div className="flex justify-center pt-4 border-t border-white/10"></div>
        </div>
    )
}
