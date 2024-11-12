import React, {useState} from "react";
import {ChevronDown} from "lucide-react";
import RepoManifestItem from "./RepoManifestItem.tsx";

export default function RepoManifestCombo({name, items, roundTop, roundBottom}: { name: string, items: string[], roundTop: boolean, roundBottom: boolean }) {
    const [isFolded, setIsFolded] = useState<boolean>(true);

    return (
        <React.Fragment>
            <div
                className={`w-full bg-neutral-800 h-14 flex flex-row items-center justify-between p-4 ${roundTop ? "rounded-t-lg" : ""} ${roundBottom && isFolded ? "rounded-b-lg" : ""}`}>
                <span className="text-white">{name}</span>
                <ChevronDown color="white"
                             onClick={() => setIsFolded(!isFolded)}
                             className={`h-10 w-10 hover:bg-white/10 border-x-4 border-y-5 border-transparent transition rounded-lg cursor-pointer duration-500 ${isFolded ? "rotate-00" : "rotate-180"}`}/>
            </div>
            <div
                className={`w-full transition-all duration-500 overflow-hidden bg-neutral-900 gap-4 flex flex-col items-center justify-between px-4 ${isFolded ? "py-0" : "py-4"} ${roundBottom ? "rounded-b-lg" : ""}`}
                style={{
                    // height: isFolded ? "0px" : (((24 + 16) * items.length) - 16) + "px"
                    // height: isFolded ? "0%" : "10%",
                    maxHeight: isFolded ? "0px" : (items.length * 64) + "px", // Big value
                }}
            >
                {items.map((name) => (
                    <RepoManifestItem name={name} key={name}/>
                ))}

            </div>
        </React.Fragment>
    )
}
