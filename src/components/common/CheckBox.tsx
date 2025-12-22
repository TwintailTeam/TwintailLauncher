import {useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import HelpTooltip from "./HelpTooltip.tsx";

export default function CheckBox({ id, name, enabled, install, fetchSettings, fetchInstallSettings, helpText, onToggle}: { id: string, name: string, enabled: boolean, install?: string, fetchSettings?: () => void, fetchInstallSettings?: (id: string) => void, helpText: string, onToggle?: (checked: boolean) => void }) {
    const [isEnabled, setIsEnabled] = useState<boolean>(enabled);

    const handleToggle = () => {
        switch (id) {
            case "third_party_repo_updates": {
                if (fetchSettings !== undefined) {
                    invoke("update_settings_third_party_repo_updates", {enabled: !isEnabled}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchSettings();
                    });
                }
            }
            break;
            case "skip_game_dl": {
                setIsEnabled(!isEnabled);
                if (onToggle) {
                    onToggle(!isEnabled);
                }
            }
            break;
            case "skip_version_updates": {
                setIsEnabled(!isEnabled);
            }
            break;
            case "skip_hash_validation": {
                setIsEnabled(!isEnabled);
            }
            break;
            case "skip_version_updates2": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_skip_version_updates", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "skip_hash_validation2": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_skip_hash_valid", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_jadeite": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_use_jadeite", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_xxmi": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_use_xxmi", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_fps_unlock": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_use_fps_unlock", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_gamemode": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_use_gamemode", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_mangohud": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_use_mangohud", {enabled: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_engineini_edit": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_xxmi_config", {engineiniTweaks: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_xxmi_sw": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_xxmi_config", {xxmiSw: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "tweak_xxmi_sd": {
                if (fetchInstallSettings !== undefined) {
                    invoke("update_install_xxmi_config", {xxmiSd: !isEnabled, id: install}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchInstallSettings(install as string)
                    });
                }
            }
            break;
            case "uninstall_prefix_delete": {
                setIsEnabled(!isEnabled);
            }
            break;
        }
    };

    return (
        <div className="flex w-full items-center gap-4 max-sm:flex-col max-sm:items-stretch">
            <span className="text-white text-sm flex items-center gap-1 w-56 shrink-0 max-sm:w-full">{name}
                <HelpTooltip text={helpText}/>
            </span>
            <div className={"inline-flex items-center justify-end ml-auto w-[320px] h-10"}>
                <div 
                    className={`w-12 h-6 rounded-full relative transition-all ${isEnabled ? "bg-purple-600" : "bg-zinc-700/60"} cursor-pointer`}
                    onClick={handleToggle}
                >
                    <input 
                        type="checkbox" 
                        className={`focus:outline-none focus:ring-0 focus:ring-offset-0 h-full aspect-square rounded-full bg-white transition-all absolute appearance-none cursor-pointer pointer-events-none ${isEnabled ? 'translate-x-full' : 'translate-x-0'}`} 
                        id={id} 
                        checked={isEnabled} 
                        readOnly
                    />
                </div>
            </div>
        </div>
    )
}
