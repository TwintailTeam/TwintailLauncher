import {DownloadCloudIcon, FolderOpenIcon, Trash2Icon} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";
import {POPUPS} from "../POPUPS.ts";

export default function RunnerItem({name, id, url, installedRunners, setOpenPopup, fetchInstalledRunners}: { id: string, name: string, url: string, installedRunners: any, setOpenPopup: (popup: POPUPS) => void, fetchInstalledRunners: () => void }) {
    return (
        <div className="flex flex-row items-center justify-between w-full h-8 px-2 py-1 rounded-lg hover:bg-white/5 transition-all duration-200">
            <span className="text-white text-sm">{name}</span>
            <div className="flex flex-row items-center justify-end">
                {installedRunners.some((runner: any) => runner.version === id && runner.is_installed) ? (
                    <>
                        <button className="flex flex-row gap-2 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white hover:text-red-600" id={id} onClick={() => {
                            invoke("remove_installed_runner", {runnerVersion: id}).then(() => {setOpenPopup(POPUPS.NONE);fetchInstalledRunners();});
                        }}><Trash2Icon/><span></span>
                        </button>
                        <button className="flex flex-rowrounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white hover:text-purple-600" id={id} onClick={() => {
                            invoke("open_folder", {runnerVersion: id, manifestId: "", installId: "", pathType: "runner_global"}).then(() => {setOpenPopup(POPUPS.NONE);});
                        }}><FolderOpenIcon/><span></span>
                        </button>
                    </>) : (
                        <button className="flex flex-row rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white hover:text-purple-600" id={id} onClick={() => {
                            invoke("add_installed_runner", {runnerUrl: `${url}`, runnerVersion: id}).then(() => {setOpenPopup(POPUPS.NONE);fetchInstalledRunners();});
                        }}><DownloadCloudIcon/><span></span>
                        </button>)
                }
            </div>
        </div>
    )
}
