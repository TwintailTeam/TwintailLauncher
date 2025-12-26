import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft} from "lucide-react";
import CheckBox from "../../common/CheckBox.tsx";
import {useState} from "react";
import SelectMenu from "../../common/SelectMenu.tsx";

interface IProps {
    install: any,
    gameSwitches: any,
    fetchInstallSettings: (id: string) => void,
    setOpenPopup: (popup: POPUPS) => void
}

export default function XXMISettings({setOpenPopup, install, gameSwitches, fetchInstallSettings}: IProps) {
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
                    <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-red-200 bg-clip-text text-transparent">XXMI settings</h1>
                </div>
            </div>
            <div className="bg-zinc-900/60 border border-white/20 rounded-xl p-6 flex flex-col gap-2 shadow-inner">
                {(gameSwitches.xxmi) ? <CheckBox enabled={install.use_xxmi} name={"Enable XXMI"} id={"tweak_xxmi"} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Enable and inject XXMI modding tool."}/> : null}
                {(gameSwitches.xxmi) ? null/*<CheckBox enabled={true} name={"Apply Engine.ini tweaks"} id={"tweak_engineini_edit"} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Modify UnrealEngine Engine.ini with performance tweaks for modding."}/>*/ : null}
                {(gameSwitches.xxmi) ? <CheckBox enabled={install.xxmi_config.show_warnings} name={"Show warnings"} id={"tweak_xxmi_sw"} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Show mod config parse warnings, useful for debugging broken mods."}/> : null}
                {(gameSwitches.xxmi) ? <CheckBox enabled={install.xxmi_config.dump_shaders} name={"Dump shaders"} id={"tweak_xxmi_sd"} fetchInstallSettings={fetchInstallSettings} install={install.id} helpText={"Enable shader dumping, useful for mod development."}/> : null}
                {(gameSwitches.xxmi) ? <SelectMenu id={"tweak_xxmi_hunting"} name={"Hunting mode"} options={[{name: "Disabled", value: 0}, {name: "Always enabled", value: 1}, {name: "Soft disabled", value: 2}]} selected={install.xxmi_config.hunting_mode} multiple={false} helpText={"Enable hunting mode for mod development."} setOpenPopup={setOpenPopup} install={install.id} fetchInstallSettings={fetchInstallSettings}/> : null}
            </div>
            <div className="flex justify-center pt-4 border-t border-white/10"></div>
        </div>
    )
}
