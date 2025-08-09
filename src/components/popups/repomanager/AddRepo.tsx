import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft, ChevronDown, X} from "lucide-react";
import React, {useState} from "react";

export default function AddRepo({setOpenPopup}: {setOpenPopup: (popup: POPUPS) => void}) {
    const [advanncedOptionsOpen, setAdvanncedOptionsOpen] = useState<boolean>(false);

    return (
        <div className="rounded-lg h-auto w-1/2 bg-black/70 fixed-backdrop-blur-md border border-white/20 flex flex-col p-6 gap-4 overflow-scroll scrollbar-none">
            <div className="flex flex-row items-center gap-4">
                <ArrowLeft className="text-gray-400 hover:text-white cursor-pointer" onClick={() => {
                    setOpenPopup(POPUPS.REPOMANAGER);
                }}/>
                <h1 className="text-white font-bold text-2xl">Add a Repository</h1>
                <div className="flex-grow"/>
                <X className="text-white hover:text-gray-400 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>

            <input type="text"
                   className="focus:outline-none h-12 rounded-lg bg-black/20 border border-white/20 text-white px-4 placeholder-white/50 text-lg"
                   placeholder="Github Repository (i.e. TwintailTeam/KeqingRepo)"/>
            <div className="flex flex-row gap-2 items-center cursor-pointer" onClick={() => {
                setAdvanncedOptionsOpen(!advanncedOptionsOpen)
            }}>
                <ChevronDown
                    className={`text-white transition-all ${advanncedOptionsOpen ? "rotate-180" : "rotate-0"}`}/>
                <span className="text-white select-none">Advanced options</span>
            </div>
            {advanncedOptionsOpen && <React.Fragment>
				<input type="text"
					   className="focus:outline-none h-12 rounded-lg bg-black/20 border border-white/20 text-white px-4 placeholder-white/50 text-lg"
					   placeholder="Override Branch"/>
				<input type="text"
					   className="focus:outline-none h-12 rounded-lg bg-black/20 border border-white/20 text-white px-4 placeholder-white/50 text-lg"
					   placeholder="Override Name"/>
			</React.Fragment>}
            <div className="flex-grow"/>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-2 items-center py-2 px-4 bg-purple-600 hover:bg-purple-700 rounded-lg" onClick={() => {
                    setOpenPopup(POPUPS.REPOMANAGER);
                }}>
                    <span className="font-semibold">Add Repository</span>
                </button>
            </div>
        </div>
    )
}
