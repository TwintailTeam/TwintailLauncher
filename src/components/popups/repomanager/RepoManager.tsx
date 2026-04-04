import { X } from "lucide-react";
import RepoManifestCombo from "./RepoManifestCombo.tsx";
import { useState } from "react";
import { POPUPS } from "../POPUPS.ts";

export default function RepoManager({ repos, setOpenPopup, fetchRepositories }: { repos: any, setOpenPopup: (popup: POPUPS) => void, fetchRepositories: () => void }) {
    const [isClosing] = useState(false);

    return (
        <div className={`rounded-2xl w-[90vw] max-w-4xl max-h-[85vh] bg-[#0c0c0c] border border-white/10 flex flex-col p-6 overflow-hidden shadow-2xl ${isClosing ? 'animate-zoom-out' : 'animate-zoom-in'}`}>
            <div className="flex flex-row items-center justify-between mb-6">
                <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-blue-200 bg-clip-text text-transparent">Repositories and Manifests</h1>
                <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-12 h-12 transition-all duration-200 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)} />
            </div>
            <div className="rounded-xl w-full overflow-y-auto overflow-x-hidden hover-scrollbar flex-1">
                {repos.map((repo: any, idx: number) => {
                    return (
                        <RepoManifestCombo key={repo.id} name={repo.github_id} items={repo.manifests} roundTop={idx == 0} roundBottom={idx == repos.length - 1} fetchRepositories={fetchRepositories} />
                    )
                })
                }
            </div>
        </div>
    )
}
