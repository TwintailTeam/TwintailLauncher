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
import {POPUPS} from "../popups/POPUPS.ts";
import {invoke} from "@tauri-apps/api/core";
import {HeartIcon} from "lucide-react";

// Discord icon component
const DiscordIcon = React.forwardRef<SVGSVGElement, React.SVGProps<SVGSVGElement>>((props, ref) => (
    <svg
        ref={ref}
        viewBox="0 0 24 24"
        fill="currentColor"
        xmlns="http://www.w3.org/2000/svg"
        {...props}
    >
        <path d="M20.317 4.492c-1.53-.69-3.17-1.2-4.885-1.49a.075.075 0 0 0-.079.036c-.21.369-.444.85-.608 1.23a18.566 18.566 0 0 0-5.487 0 12.36 12.36 0 0 0-.617-1.23A.077.077 0 0 0 8.562 3c-1.714.29-3.354.8-4.885 1.491a.07.07 0 0 0-.032.027C.533 9.093-.32 13.555.099 17.961a.08.08 0 0 0 .031.055 20.03 20.03 0 0 0 5.993 2.98.078.078 0 0 0 .084-.026 13.83 13.83 0 0 0 1.226-1.963.074.074 0 0 0-.041-.104 13.201 13.201 0 0 1-1.872-.878.075.075 0 0 1-.008-.125c.126-.093.252-.19.372-.287a.075.075 0 0 1 .078-.01c3.927 1.764 8.18 1.764 12.061 0a.075.075 0 0 1 .079.009c.12.098.246.195.373.288a.075.075 0 0 1-.006.125c-.598.344-1.22.635-1.873.877a.075.075 0 0 0-.041.105c.36.687.772 1.341 1.225 1.962a.077.077 0 0 0 .084.028 19.963 19.963 0 0 0 6.002-2.981.076.076 0 0 0 .032-.054c.5-5.094-.838-9.52-3.549-13.442a.06.06 0 0 0-.031-.028zM8.02 15.278c-1.182 0-2.157-1.069-2.157-2.38 0-1.312.956-2.38 2.157-2.38 1.201 0 2.176 1.068 2.157 2.38 0 1.311-.956 2.38-2.157 2.38zm7.975 0c-1.183 0-2.157-1.069-2.157-2.38 0-1.312.955-2.38 2.157-2.38 1.2 0 2.176 1.068 2.156 2.38 0 1.311-.956 2.38-2.156 2.38z"/>
    </svg>
));

export default function SidebarLink({uri, title, iconType, popup}: {uri: string, title: string, iconType: string, popup: POPUPS}) {
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
            <div
                ref={refs.setReference}
                {...getReferenceProps()}
                className={`flex items-center justify-center w-10 h-10 rounded-xl cursor-pointer transition-all duration-200 text-white/70 hover:text-white hover:bg-white/5 hover:shadow-[0_0_12px_rgba(147,51,234,0.15)] active:scale-95 ${iconType === "donate" ? "mb-2" : ""}`}
                onClick={() => {
                    invoke('open_uri', {uri: uri}).then(() => {});
                }}
            >
                {iconType === "discord" && <DiscordIcon className="w-6 h-6" />}
                {iconType === "donate" && <HeartIcon className="w-6 h-6" />}
            </div>
            {(isOpen && popup == POPUPS.NONE) && (
                <div ref={refs.setFloating} style={floatingStyles} {...getFloatingProps()} className="bg-black/75 rounded-md p-2 min-w-max z-50">
                    <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
                    <span className="text-white z-50">{title}</span>
                </div>
            )}
        </React.Fragment>
    )
}
