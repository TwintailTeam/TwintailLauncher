import {POPUPS} from "../popups/POPUPS.ts";
import {ArrowRightIcon} from "lucide-react";

export default function SubMenu({setOpenPopup, name, page}: {name: string, page: POPUPS, setOpenPopup: (popup: POPUPS) => void}) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-8 hover:cursor-pointer">
            <button className={"flex w-full h-8 items-center justify-between hover:cursor-pointer hover:bg-white/10 rounded"} onClick={() => setOpenPopup(page)}>
                <span className="text-white text-sm">{name}</span>
                <div className={"inline-flex flex-row items-center justify-center text-white px-2"}>
                    <ArrowRightIcon/>
                </div>
            </button>
        </div>
    )
}
