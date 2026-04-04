import { useRef, useState } from "react";
import { createPortal } from "react-dom";
import {
    arrow,
    autoUpdate,
    flip,
    FloatingArrow,
    offset,
    safePolygon,
    shift,
    useDismiss,
    useFocus,
    useFloating,
    useHover,
    useInteractions,
    useRole
} from "@floating-ui/react";
import { CircleHelp } from "lucide-react";

export default function HelpTooltip({ text }: { text: string }) {
    const [open, setOpen] = useState(false);
    const arrowRef = useRef(null);
    const { refs, floatingStyles, context } = useFloating({
        open,
        onOpenChange: setOpen,
        middleware: [offset(10), flip(), shift({ padding: 12 }), arrow({ element: arrowRef })],
        placement: "top",
        whileElementsMounted: autoUpdate,
    });
    const hover = useHover(context, {
        move: false,
        handleClose: safePolygon({ buffer: 2 })
    });
    const focus = useFocus(context);
    const dismiss = useDismiss(context);
    const role = useRole(context, { role: "tooltip" });
    const { getReferenceProps, getFloatingProps } = useInteractions([hover, focus, dismiss, role]);

    return (
        <>
            <span
                className="relative inline-flex items-center justify-center transition-all duration-200"
                ref={refs.setReference}
                {...getReferenceProps({ tabIndex: 0 })}
            >
                <CircleHelp className="w-4 h-4 text-zinc-500 hover:text-purple-400 hover:scale-110 transition-all duration-200 cursor-help drop-shadow-sm hover:drop-shadow-[0_0_8px_rgba(168,85,247,0.4)]" />
            </span>
            {open && createPortal(
                <div
                    ref={refs.setFloating}
                    style={{ ...floatingStyles, animation: 'fadeIn 150ms ease-out' }}
                    className="z-[9999] rounded-xl border border-white/10 bg-zinc-900/90 px-3.5 py-2.5 text-[12px] text-zinc-100 shadow-[0_14px_35px_rgba(0,0,0,0.45)] w-[min(92vw,420px)]"
                    {...getFloatingProps()}
                >
                    <div className="help-tooltip-scroll max-h-64 overflow-y-auto pr-1">
                        <p className="leading-[1.35rem] whitespace-pre-line break-words">{text}</p>
                    </div>
                    <FloatingArrow ref={arrowRef} context={context} className="fill-zinc-900/90" />
                </div>,
                document.body
            )}
        </>
    )
}
