import {useState} from "react";

export default function RepoManifestItem({name}: { name: string }) {
    const [isEnabled, setIsEnabled] = useState<boolean>(false);

    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm">{name}</span>
            <div
                className="w-12 h-6 rounded-full relative bg-white/10 cursor-pointer"
                onClick={() => setIsEnabled(!isEnabled)}>
                <div
                    className={`h-full aspect-square rounded-full bg-white transition-all absolute ${isEnabled ? "translate-x-full" : "translate-x-0"}`}/>
            </div>
        </div>
    )
}
