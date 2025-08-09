import {Plus, X} from "lucide-react";
import RepoManifestCombo from "./RepoManifestCombo.tsx";
import {POPUPS} from "../POPUPS.ts";

export default function RepoManager({repos, setOpenPopup, fetchRepositories}: {repos: any, setOpenPopup: (popup: POPUPS) => void, fetchRepositories: () => void}) {
    return (
        <div className="rounded-lg h-full w-1/2 bg-black/70 fixed-backdrop-blur-md border border-white/20 flex flex-col p-6 gap-6 overflow-scroll scrollbar-none">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Repositories and Manifests</h1>
                <X className="text-white hover:text-gray-400 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex-row-reverse flex">
                <button className="flex flex-row gap-2 items-center py-2 px-4 bg-purple-600 hover:bg-purple-700 rounded-lg" onClick={() => {setOpenPopup(POPUPS.ADDREPO)}}>
                    <Plus className="stroke-[3px]"/>
                    <span className="font-semibold">Add Repository</span>
                </button>
            </div>
            <div className="rounded-lg w-full overflow-y-auto overflow-scroll scrollbar-none">
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
