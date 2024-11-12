import {Plus, X} from "lucide-react";
import RepoManifestCombo from "./RepoManifestCombo.tsx";
import {POPUPS} from "../POPUPS.ts";

const repos = {
    "TeamKeqing/launcher-manifests": [
        "Genshin Impact (Global)",
        "Genshin Impact (China)",
        "Honkai: Star Rail (Global)",
        "Honkai: Star Rail (China)",
        "Honkai Impact 3rd (Global)",
        "Honkai Impact 3rd (China)",
        "Zenless Zone Zero (Global)",
        "Zenless Zone Zero (China)",
    ],
    "FOREVEREALIZE/tof-manifests": [
        "Tower of Fantasy"
    ],
    "FOREVEREALIZE/wuwa-manifests": [
        "Wuthering Waves"
    ]
}

export default function RepoManager({setOpenPopup}: {setOpenPopup: (popup: POPUPS) => void}) {
    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Repositories and Manifests</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-1 items-center p-2 bg-blue-600 rounded-lg" onClick={() => {setOpenPopup(POPUPS.ADDREPO)}}>
                    <Plus className="stroke-[4px]"/>
                    <span className="font-semibold translate-y-px">Add Repository</span>
                </button>
            </div>
            <div className="rounded-lg w-full">
                {Object.entries(repos).map((repo, idx, arr) => (
                    <RepoManifestCombo key={repo[0]} name={repo[0]} items={repo[1]} roundTop={idx == 0}
                                       roundBottom={idx == arr.length - 1}/>
                ))}
            </div>
        </div>
    )
}
