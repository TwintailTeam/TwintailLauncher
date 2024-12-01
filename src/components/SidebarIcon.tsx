import React, {useRef, useState} from "react";
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
import {POPUPS} from "./popups/POPUPS.ts";

type SidebarIconProps = {
    icon: string,
    name: string,
    id: string,
    setCurrentGame: (a: string) => void,
    setOpenPopup: (a: POPUPS) => void,
    popup: POPUPS,
    setDisplayName: (name: string) => void,
}

export default function SidebarIcon({icon, name, id, setCurrentGame, setOpenPopup, popup, setDisplayName}: SidebarIconProps) {
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

    const {getReferenceProps, getFloatingProps} = useInteractions([
        hover
    ]);

    return (
        <React.Fragment>
            <img ref={refs.setReference} {...getReferenceProps()} className="aspect-square w-12 rounded-lg cursor-pointer" src={icon} onClick={() => {
                setOpenPopup(POPUPS.NONE)
                setCurrentGame(id)
                setDisplayName(name)
            }} alt={"?"}/>

            {(isOpen && popup == POPUPS.NONE) && (
                <div
                    ref={refs.setFloating}
                    style={floatingStyles}
                    {...getFloatingProps()}
                    className="bg-black/75 rounded-md p-2 w-full min-w-max z-50"
                >
                    <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
                    <span className="text-white z-50">{name}</span>
                </div>
            )}
        </React.Fragment>
    )
}
