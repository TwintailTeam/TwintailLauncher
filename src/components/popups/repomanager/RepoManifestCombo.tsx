import React, {useState} from "react";
import {ChevronDown} from "lucide-react";
import RepoManifestItem from "./RepoManifestItem.tsx";

export default function RepoManifestCombo({name, items, roundTop, roundBottom, fetchRepositories}: { name: string, items: string[], roundTop: boolean, roundBottom: boolean, fetchRepositories: () => void}) {
    const [isFolded, setIsFolded] = useState<boolean>(true);

    return (
        <React.Fragment>
            <div className={`w-full bg-neutral-800 h-14 flex flex-row items-center justify-between p-4 scrollbar-none overflow-scroll ${roundTop ? "rounded-t-lg" : ""} ${roundBottom && isFolded ? "rounded-b-lg" : ""}`}>
                <span className="text-white">{name}</span>
                <ChevronDown color="white" onClick={() => setIsFolded(!isFolded)} className={`h-10 w-10 hover:bg-white/10 border-x-4 border-y-5 border-transparent transition rounded-lg cursor-pointer duration-500 ${isFolded ? "rotate-00" : "rotate-180"}`}/>
            </div>
            <div className={`w-full transition-all duration-500 overflow-scroll scrollbar-none bg-neutral-900 gap-4 flex flex-col items-center justify-between px-4 ${isFolded ? "py-0" : "py-4"} ${roundBottom ? "rounded-b-lg" : ""}`}
                style={{maxHeight: isFolded ? "0px" : (items.length * 64) + "px"}}>
                {items.map((name1: any) => {
                    return (
                    <RepoManifestItem name={name1.display_name} key={name1.id} id={name1.id} enabled={name1.enabled} fetchRepositories={fetchRepositories} repo={name} />
                )})}
            </div>
        </React.Fragment>
    )
}
