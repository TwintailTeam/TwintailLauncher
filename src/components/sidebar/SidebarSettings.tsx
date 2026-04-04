import React, { useRef, useState } from "react";
import {
    arrow,
    autoUpdate,
    flip,
    FloatingArrow,
    offset,
    shift,
    useFloating,
    useHover,
    useInteractions
} from "@floating-ui/react";
import { POPUPS } from "../popups/POPUPS.ts";
import { PAGES } from "../pages/PAGES.ts";
import { Settings } from "lucide-react";

export default function SidebarSettings({ popup, currentPage, setCurrentPage }: { setOpenPopup: (a: POPUPS) => void, popup: POPUPS, currentPage?: PAGES, setCurrentPage?: (page: PAGES) => void }) {
    const [isOpen, setIsOpen] = useState(false);

    const arrowRef = useRef(null);
    const { refs, floatingStyles, context } = useFloating({
        open: isOpen,
        onOpenChange: setIsOpen,
        middleware: [offset(25), flip(), shift(), arrow({
            element: arrowRef
        })],
        whileElementsMounted: autoUpdate,
        placement: "right",
    });

    const hover = useHover(context, { move: false });

    const { getReferenceProps, getFloatingProps } = useInteractions([
        hover
    ]);

    const isActive = currentPage === PAGES.SETTINGS;

    return (
        <React.Fragment>
            <div
                ref={refs.setReference}
                {...getReferenceProps()}
                className={`flex items-center justify-center w-10 h-10 rounded-xl cursor-pointer transition-all duration-200 ${isActive ? 'text-purple-400 bg-purple-500/15 shadow-[0_0_12px_rgba(147,51,234,0.3)]' : 'text-white/70 hover:text-white hover:bg-white/5 hover:shadow-[0_0_12px_rgba(147,51,234,0.15)]'} active:scale-95`}
                onClick={() => {
                    if (setCurrentPage) {
                        setCurrentPage(currentPage === PAGES.SETTINGS ? PAGES.NONE : PAGES.SETTINGS);
                    }
                }}
            >
                <Settings className="w-6 h-6" />
            </div>

            {(isOpen && popup == POPUPS.NONE && currentPage === PAGES.NONE) && (
                <div ref={refs.setFloating} style={floatingStyles} {...getFloatingProps()} className="bg-black/75 rounded-md p-2 min-w-max z-50">
                    <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
                    <span className="text-white z-50">Launcher Settings</span>
                </div>
            )}
        </React.Fragment>
    )
}

