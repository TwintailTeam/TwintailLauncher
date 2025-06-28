import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft, ChevronDown, X} from "lucide-react";
import React, {useState} from "react";

export default function AddRepo({setOpenPopup}: {setOpenPopup: (popup: POPUPS) => void}) {
    const [advanncedOptionsOpen, setAdvanncedOptionsOpen] = useState<boolean>(false);

    return (
        <div className="rounded-lg h-full w-3/4 flex flex-col p-4 bg-neutral-700 gap-4 overflow-scroll scrollbar-none">
            <div className="flex flex-row items-center gap-2">
                <ArrowLeft className="text-neutral-500 hover:text-neutral-700 cursor-pointer" onClick={() => {
                    setOpenPopup(POPUPS.REPOMANAGER);
                }}/>
                <h1 className="text-white text-stroke font-bold text-2xl">Add a Repository</h1>
                <div className="flex-grow">{/* Spacer */}</div>
                <X className="text-neutral-500 hover:text-neutral-700 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>

            <div>{/* Spacer */}</div>

            <input type="text"
                   className="focus:outline-none h-10 rounded-lg bg-white/20 text-white px-4 placeholder-white/50"
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
					   className="focus:outline-none h-10 rounded-lg bg-white/20 text-white px-4 placeholder-white/50"
					   placeholder="Override Branch"/>
				<input type="text"
					   className="focus:outline-none h-10 rounded-lg bg-white/20 text-white px-4 placeholder-white/50"
					   placeholder="Override Name"/>
			</React.Fragment>}
            <div className="flex-grow">{/* Spacer */}</div>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-1 items-center p-2 bg-blue-600 rounded-lg" onClick={() => {
                    setOpenPopup(POPUPS.REPOMANAGER);
                }}>
                    <span className="font-semibold translate-y-px">Add Repository</span>
                </button>
            </div>
        </div>
    )
}
