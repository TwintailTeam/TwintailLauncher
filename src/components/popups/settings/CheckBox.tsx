import {useState} from "react";
import {invoke} from "@tauri-apps/api/core";

export default function CheckBox({ name, enabled, fetchSettings}: { name: string, enabled: boolean, fetchSettings: () => void}) {
    const [isEnabled, setIsEnabled] = useState<boolean>(enabled);

    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm">{name}</span>
            <div className={`w-12 h-6 rounded-full relative transition-all ${isEnabled ? "bg-blue-600" : "bg-white/10"} cursor-pointer`}
                onClick={() => {
                    invoke("update_settings_third_party_repo_updates", {enabled: !isEnabled}).then(() => {
                        setIsEnabled(!isEnabled);
                        fetchSettings();
                    });
                }}>
                <div className={`h-full aspect-square rounded-full bg-white transition-all absolute ${isEnabled ? "translate-x-full" : "translate-x-0"}`}/>
            </div>
        </div>
    )
}
