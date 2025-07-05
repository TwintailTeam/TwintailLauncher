import {useRef, useState} from "react";
import {arrow, autoUpdate, flip, FloatingArrow, offset, shift, useFloating} from "@floating-ui/react";
import {CircleHelpIcon} from "lucide-react";

export default function HelpTooltip({text}: {text: string}) {
    const [open, setOpen] = useState(false);
    const arrowRef = useRef(null);
    const { refs, floatingStyles, context } = useFloating({
        open,
        onOpenChange: setOpen,
        middleware: [offset(8), flip(), shift({ padding: 8 }), arrow({ element: arrowRef }),],
        placement: "top",
        whileElementsMounted: autoUpdate,
    });

    return (
        <>
        <span className="relative" ref={refs.setReference} tabIndex={0} onFocus={() => setOpen(true)} onBlur={() => setOpen(false)} onMouseEnter={() => setOpen(true)} onMouseLeave={() => setOpen(false)}>
            <CircleHelpIcon className="w-4 h-4 text-gray-400 hover:cursor-pointer hover:text-purple-600"/>
        </span>
        {open && (
            <div ref={refs.setFloating} style={floatingStyles} className="z-50 bg-black/75 text-white text-xs rounded py-1 px-2 whitespace-nowrap shadow-lg">
                {text}
                <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
            </div>
            )}
        </>
    )
}