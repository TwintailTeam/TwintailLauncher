import {useRef, useState} from "react";
import {arrow, autoUpdate, flip, FloatingArrow, offset, shift, useFloating} from "@floating-ui/react";

export default function CollapsableTooltip({text, icon}: {text: string, icon: any}) {
    const [open, setOpen] = useState(false);
    const arrowRef = useRef(null);
    const { refs, floatingStyles, context } = useFloating({
        open,
        onOpenChange: setOpen,
        middleware: [offset(25), flip(), shift(), arrow({
            element: arrowRef
        })],
        placement: "right",
        whileElementsMounted: autoUpdate,
    });

    return (
        <>
        <span className="relative" ref={refs.setReference} tabIndex={0} onFocus={() => setOpen(true)} onBlur={() => setOpen(false)} onMouseEnter={() => setOpen(true)} onMouseLeave={() => setOpen(false)}>
            {icon}
        </span>
        {open && (
            <div ref={refs.setFloating} style={floatingStyles} className="bg-black/75 rounded-md p-2 min-w-max z-50 text-white">
                {text}
                <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
            </div>
            )}
        </>
    )
}