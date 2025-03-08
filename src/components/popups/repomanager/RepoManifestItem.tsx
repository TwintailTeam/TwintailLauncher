import {useState} from "react";
import {invoke} from "@tauri-apps/api/core";

export default function RepoManifestItem({name, id, enabled}: { id: string, name: string, enabled: boolean}) {
    const [isEnabled, setIsEnabled] = useState<boolean>(enabled);

    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm">{name}</span>
            <div
                className={`w-12 h-6 rounded-full relative transition-all ${isEnabled ? "bg-blue-600" : "bg-white/10"} cursor-pointer`}
                onClick={() => {
                    invoke("update_manifest_enabled", { id: id, enabled: !isEnabled }).then(() => {
                        setIsEnabled(!isEnabled);
                    });
                }}>
                <div className={`h-full aspect-square rounded-full bg-white transition-all absolute ${isEnabled ? "translate-x-full" : "translate-x-0"}`}/>
            </div>
        </div>
    )
}
