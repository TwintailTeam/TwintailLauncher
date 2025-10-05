import {X} from "lucide-react";
import RunnerManifestCombo from "./RunnerManifestCombo.tsx";
import {useState} from "react";
import {POPUPS} from "../POPUPS.ts";

export default function RunnerManager({runners, installedRunners, setOpenPopup, fetchInstalledRunners}: {runners: any, installedRunners: any, setOpenPopup: (popup: POPUPS) => void, fetchInstalledRunners: () => void}) {
    const [isClosing] = useState(false);

    return (
        <div className={`rounded-xl w-[90vw] max-w-4xl max-h-[85vh] bg-zinc-900 border border-white/20 flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out transition-all`}>
            <div className="flex flex-row items-center justify-between mb-6">
                <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-blue-200 bg-clip-text text-transparent">Runner manager</h1>
                <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-12 h-12 transition-all duration-200 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="rounded-xl w-full overflow-y-auto overflow-x-hidden hover-scrollbar flex-1">
                {runners.map((runner:any, idx: number) => {
                    return (
                        <RunnerManifestCombo key={runner.display_name} name={runner.display_name} items={runner.versions} roundTop={idx == 0} roundBottom={idx == runners.length - 1} setOpenPopup={setOpenPopup} installedRunners={installedRunners} fetchInstalledRunners={fetchInstalledRunners} />
                    )
                })
                }
            </div>
        </div>
    )
}
