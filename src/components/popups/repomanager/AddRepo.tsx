import {POPUPS} from "../POPUPS.ts";
import {ArrowLeft, ChevronDown, X} from "lucide-react";
import React, {useState} from "react";

export default function AddRepo({setOpenPopup}: {setOpenPopup: (popup: POPUPS) => void}) {
    const [advanncedOptionsOpen, setAdvanncedOptionsOpen] = useState<boolean>(false);
    const [isClosing, setIsClosing] = useState(false);

    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setOpenPopup(POPUPS.NONE);
    }, 220);
    };

    return (
        <div className={`rounded-xl h-auto w-1/2 bg-gradient-to-br from-black/80 via-black/70 to-black/60 backdrop-blur-xl border border-white/30 shadow-2xl shadow-green-500/20 flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out`}>
            <div className="flex flex-row items-center gap-4">
                <ArrowLeft className="text-gray-400 hover:text-white hover:bg-white/10 rounded-lg p-3 w-10 h-10 transition-all duration-200 cursor-pointer" onClick={() => {
                    setOpenPopup(POPUPS.REPOMANAGER);
                }}/>
                <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-green-200 bg-clip-text text-transparent">Add a Repository</h1>
                <div className="flex-grow"/>
                <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-10 h-10 transition-all duration-200 cursor-pointer" onClick={handleClose}/>
            </div>

            <div className="flex-1 overflow-y-auto scrollbar-none">
                <div className="space-y-4 mt-6">
                    <input type="text"
                           className="focus:outline-none h-12 rounded-xl bg-black/60 border border-white/30 focus:border-green-400/50 text-white px-4 placeholder-white/50 text-lg backdrop-blur-sm transition-all duration-200"
                           placeholder="Github Repository (i.e. TwintailTeam/KeqingRepo)"/>
                    <div className="flex flex-row gap-2 items-center cursor-pointer" onClick={() => {
                        setAdvanncedOptionsOpen(!advanncedOptionsOpen)
                    }}>
                        <ChevronDown
                            className={`text-white transition-all duration-100 ${advanncedOptionsOpen ? "rotate-180" : "rotate-0"}`}/>
                        <span className="text-white select-none">Advanced options</span>
                    </div>
                    {advanncedOptionsOpen && <React.Fragment>
                        <input type="text"
                               className="focus:outline-none h-12 rounded-xl bg-black/60 border border-white/30 focus:border-green-400/50 text-white px-4 placeholder-white/50 text-lg backdrop-blur-sm transition-all duration-200"
                               placeholder="Override Branch"/>
                        <input type="text"
                               className="focus:outline-none h-12 rounded-xl bg-black/60 border border-white/30 focus:border-green-400/50 text-white px-4 placeholder-white/50 text-lg backdrop-blur-sm transition-all duration-200"
                               placeholder="Override Name"/>
                    </React.Fragment>}
                </div>
            </div>
            <div className="flex justify-center pt-6 mt-4 border-t border-white/10">
                <button className="flex flex-row gap-3 items-center py-3 px-8 bg-gradient-to-r from-green-600 to-green-700 hover:from-green-500 hover:to-green-600 rounded-xl shadow-lg shadow-green-500/30 transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                    setOpenPopup(POPUPS.REPOMANAGER);
                }}>
                    <span>Add Repository</span>
                </button>
            </div>
        </div>
    )
}
