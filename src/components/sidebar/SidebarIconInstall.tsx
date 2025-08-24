import React, {useRef, useState} from "react";
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
import {POPUPS} from "../popups/POPUPS.ts";

type SidebarIconProps = {
    icon: string,
    name: string,
    id: string,
    background: string,
    enabled: boolean,
    hasUpdate?: boolean,
    setCurrentInstall: (a: string) => void,
    setOpenPopup: (a: POPUPS) => void,
    popup: POPUPS,
    setDisplayName: (name: string) => void,
    setBackground: (file: string) => void,
    setGameIcon: (a: string) => void,
}

export default function SidebarIconInstall({icon, name, id, setCurrentInstall, setGameIcon, setOpenPopup, popup, setDisplayName, setBackground, background, enabled, hasUpdate}: SidebarIconProps) {
    const [isOpen, setIsOpen] = useState(false);

    const arrowRef = useRef(null);
    const {refs, floatingStyles, context} = useFloating({
        open: isOpen,
        onOpenChange: setIsOpen,
        middleware: [offset(25), flip(), shift(), arrow({
            element: arrowRef
        })],
        whileElementsMounted: autoUpdate,
        placement: "right",
    });

    const hover = useHover(context, {move: false});
    const {getReferenceProps, getFloatingProps} = useInteractions([hover]);

    return (
        <React.Fragment>
            {enabled ? (
                <div className="relative inline-block w-12 h-12 overflow-visible" ref={refs.setReference} {...getReferenceProps()}>
                    <img
                        id={`${id}`}
                        className={`block w-full h-full rounded-lg cursor-pointer hover:border-purple-600 hover:border-2 focus:border-2 focus:border-purple-600 outline-none disabled:cursor-not-allowed disabled:border-0`}
                        srcSet={undefined}
                        loading={"lazy"}
                        decoding={"async"}
                        src={icon}
                        tabIndex={0}
                        draggable={false}
                        onDragStart={(e) => e.preventDefault()}
                        onClick={() => {
                            let elem = document.getElementById(id);
                            // @ts-ignore
                            if (elem.hasAttribute("disabled")) {}
                            else {
                                setOpenPopup(POPUPS.NONE)
                                setCurrentInstall(id)
                                setDisplayName(name)
                                setBackground(background)
                                setGameIcon(icon)
                                // @ts-ignore
                                elem.focus();
                            }
                        }}
                        alt={"?"}
                    />
                    {hasUpdate ? (
                        <span className="pointer-events-none absolute top-0.5 right-0.5 z-20 flex h-3 w-3">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-purple-500 opacity-90"></span>
                            <span className="relative inline-flex rounded-full h-3 w-3 bg-purple-500 shadow-[inset_0_0_0_1.5px_rgba(233,213,255,0.95),0_0_10px_rgba(168,85,247,1),0_0_20px_rgba(168,85,247,0.8)]"></span>
                        </span>
                    ) : null}
                </div>
            ) : null}
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
