import {X} from "lucide-react";
import RepoManifestCombo from "./RepoManifestCombo.tsx";
import {useState} from "react";
import {POPUPS} from "../POPUPS.ts";

export default function RepoManager({repos, setOpenPopup, fetchRepositories}: {repos: any, setOpenPopup: (popup: POPUPS) => void, fetchRepositories: () => void}) {
    const [isClosing, setIsClosing] = useState(false);

    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setOpenPopup(POPUPS.NONE);
    }, 220);
    };

    return (
        <div className={`rounded-xl h-full w-3/5 bg-gradient-to-br from-black/80 via-black/70 to-black/60 backdrop-blur-xl border border-white/30 shadow-2xl shadow-blue-500/20 flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out transition-all`}>
            <div className="flex flex-row items-center justify-between mb-6">
                <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-blue-200 bg-clip-text text-transparent">Repositories and Manifests</h1>
                <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-10 h-10 transition-all duration-200 cursor-pointer" onClick={handleClose}/>
            </div>
            <div className="rounded-xl w-full overflow-y-auto overflow-scroll scrollbar-none flex-1">
                {repos.map((repo:any, idx: number) => {
                    return (
                        <RepoManifestCombo key={repo.id} name={repo.github_id} items={repo.manifests} roundTop={idx == 0} roundBottom={idx == repos.length - 1} fetchRepositories={fetchRepositories} />
                    )
                })
                }
            </div>
        </div>
    )
}
