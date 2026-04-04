import React, { useRef, useState } from "react";
import ReactDOM from "react-dom";
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

type SidebarIconProps = {
    icon: string,
    name: string,
    id: string,
    background: string,
    enabled: boolean,
    currentGame?: string,
    setCurrentGame: (a: string) => void,
    setOpenPopup: (a: POPUPS) => void,
    popup: POPUPS,
    setDisplayName: (name: string) => void,
    setBackground: (file: string) => void,
    setCurrentInstall: (a: string) => void,
    setGameIcon: (a: string) => void,
    sizeClass?: string,
    variant?: "default" | "floating",
}

// @ts-ignore
export default function SidebarIconManifest({ icon, name, id, setGameIcon, setCurrentGame, setCurrentInstall, setOpenPopup, popup, setDisplayName, setBackground, background, enabled, sizeClass = "w-12", variant = "default", currentGame }: SidebarIconProps) {
    const [isOpen, setIsOpen] = useState(false);

    const arrowRef = useRef(null);
    const { refs, floatingStyles, context } = useFloating({
        open: isOpen,
        onOpenChange: setIsOpen,
        middleware: [offset(10), flip(), shift(), arrow({
            element: arrowRef
        })],
        whileElementsMounted: autoUpdate,
        placement: "bottom",
    });

    const hover = useHover(context, { move: false });
    const { getReferenceProps, getFloatingProps } = useInteractions([hover]);

    const baseImgClass = `aspect-square ${sizeClass} rounded-lg cursor-pointer outline-none`;
    const defaultDecor = `hover:border-2 hover:border-purple-600 focus:border-2 focus:border-purple-600`;
    const floatingDecor = `p-1 bg-white/5 ring-1 ring-white/10 hover:ring-purple-500/50 focus:ring-2 focus:ring-purple-500/60 shadow-md hover:shadow-purple-500/20 transition-all duration-200`;

    const composedClass = `${baseImgClass} ${variant === "floating" ? floatingDecor : defaultDecor}`;

    return (
        <React.Fragment>
            {(enabled) ? <img ref={refs.setReference} {...getReferenceProps()} id={id} className={composedClass} srcSet={undefined} loading={"lazy"} decoding={"async"} src={icon} tabIndex={0} onClick={() => {
                setOpenPopup(POPUPS.NONE)
                setCurrentInstall("")
                setCurrentGame(id)
                setDisplayName(name)
                setGameIcon(icon)
                // @ts-ignore
                document.getElementById(id).focus();
            }} alt={"?"} /> : null}
            {(enabled && isOpen && popup == POPUPS.NONE) ?
                (typeof window !== "undefined" && window.document &&
                    ReactDOM.createPortal(
                        <div ref={refs.setFloating} style={floatingStyles} {...getFloatingProps()} className="bg-black/75 rounded-md p-2 min-w-max z-50">
                            <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
                            <span className="text-white z-50">{name}</span>
                        </div>,
                        window.document.body
                    )
                ) : null}
        </React.Fragment>
    )
}
