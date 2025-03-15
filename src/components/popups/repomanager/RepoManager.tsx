import {Plus, X} from "lucide-react";
import RepoManifestCombo from "./RepoManifestCombo.tsx";
import {POPUPS} from "../POPUPS.ts";

export default function RepoManager({repos, setOpenPopup, fetchRepositories}: {repos: any, setOpenPopup: (popup: POPUPS) => void, fetchRepositories: () => void}) {
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
