import {useState} from "react";
import {invoke} from "@tauri-apps/api/core";

export default function CheckBox({ id, name, enabled, fetchSettings}: { id: string, name: string, enabled: boolean, fetchSettings?: () => void}) {
    const [isEnabled, setIsEnabled] = useState<boolean>(enabled);

    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm">{name}</span>
            <div className={`w-12 h-6 rounded-full relative transition-all ${isEnabled ? "bg-blue-600" : "bg-white/10"} cursor-pointer`}>
                <input type={"checkbox"} className={`focus:outline-none focus:ring-0 focus:ring-offset-0 h-full aspect-square rounded-full bg-white transition-all absolute appearance-none cursor-pointer ${isEnabled ? 'translate-x-full' : 'translate-x-0'}`} id={id} defaultChecked={isEnabled} onChange={() => {
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
                        case "skip_version_updates": {
                            setIsEnabled(!isEnabled);
                        }
                        break;
                        case "skip_hash_validation": {
                            setIsEnabled(!isEnabled);
                        }
                        break;
                    }
                }}/>
                {/*<div className={`h-full aspect-square rounded-full bg-white transition-all absolute ${isEnabled ? "translate-x-full" : "translate-x-0"}`}/>*/}
            </div>
        </div>
    )
}
