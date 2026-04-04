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
import { PAGES } from "../pages/PAGES.ts";
import InstallContextMenu from "../InstallContextMenu.tsx";

type SidebarIconProps = {
    icon: string,
    name: string,
    id: string,
    background: string,
    enabled: boolean,
    hasUpdate?: boolean,
    isActive?: boolean,
    setCurrentInstall: (a: string) => void,
    setOpenPopup: (a: POPUPS) => void,
    popup: POPUPS,
    currentPage?: PAGES,
    setCurrentPage?: (page: PAGES) => void,
    setDisplayName: (name: string) => void,
    setBackground: (file: string) => void,
    setGameIcon: (a: string) => void,
    installSettings?: any,
    onOpenInstallSettings?: () => void,
    onRefreshSettings?: () => void,
    // Drag and drop props
    index?: number,
    onDragStart?: (index: number) => void,
    onDragEnd?: () => void,
    onDragOver?: (index: number) => void,
    onDrop?: (index: number) => void,
    isDragging?: boolean,
    isDragTarget?: boolean,
}

// @ts-ignore
export default function SidebarIconInstall({ icon, name, id, setCurrentInstall, setGameIcon, setOpenPopup, popup, currentPage, setCurrentPage, setDisplayName, setBackground, background, enabled, hasUpdate, installSettings, onOpenInstallSettings, onRefreshSettings, index, onDragStart, onDragEnd, onDragOver, onDrop, isDragging, isDragTarget }: SidebarIconProps) {
    const [isOpen, setIsOpen] = useState(false);
    const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);

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
    const { getReferenceProps, getFloatingProps } = useInteractions([hover]);

    const handleContextMenu = (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setContextMenu({ x: e.clientX, y: e.clientY });
    };

    return (
        <React.Fragment>
            {enabled ? (
                <div
                    className="relative flex flex-col items-center"
                    style={{ width: 48, minWidth: 48 }} // Fixed width to prevent layout shifts
                    onDragOver={(e) => {
                        e.preventDefault();
                        e.dataTransfer.dropEffect = 'move';
                        if (index !== undefined && onDragOver) {
                            // Detect if cursor is in top or bottom half of the component
                            const rect = e.currentTarget.getBoundingClientRect();
                            const midY = rect.top + rect.height / 2;
                            const isBottomHalf = e.clientY > midY;
                            // If on bottom half, signal drop target as next index (drop below this item)
                            onDragOver(isBottomHalf ? index + 1 : index);
                        }
                    }}
                    onDrop={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        if (index !== undefined && onDrop) {
                            // Use same logic as dragOver to determine actual drop target
                            const rect = e.currentTarget.getBoundingClientRect();
                            const midY = rect.top + rect.height / 2;
                            const isBottomHalf = e.clientY > midY;
                            onDrop(isBottomHalf ? index + 1 : index);
                        }
                    }}
                >
                    {/* Drop indicator - animated placeholder that appears above */}
                    {/* pointer-events-none ensures animation doesn't interfere with drag calculations */}
                    <div
                        className={`w-12 flex items-center justify-center transition-all duration-200 ease-out overflow-hidden pointer-events-none ${isDragTarget ? 'h-14 mb-1' : 'h-0'}`}
                    >
                        {isDragTarget && (
                            <div className="w-12 h-12 rounded-lg border-2 border-dashed border-purple-500/70 bg-purple-500/10 flex items-center justify-center animate-pulse">
                                <div className="w-6 h-0.5 rounded-full bg-purple-500/50" />
                            </div>
                        )}
                    </div>
                    <div
                        className={`relative w-12 h-12 flex-shrink-0 transition-opacity duration-150 ${isDragging ? 'opacity-25' : 'opacity-100'}`}
                        ref={refs.setReference}
                        {...getReferenceProps()}
                        draggable={index !== undefined && onDragStart !== undefined}
                        onDragStart={(e) => {
                            if (index !== undefined && onDragStart) {
                                e.dataTransfer.effectAllowed = 'move';
                                e.dataTransfer.setData('text/plain', index.toString());
                                // Small delay to capture drag image before opacity change
                                setTimeout(() => onDragStart(index), 0);
                            }
                        }}
                        onDragEnd={() => {
                            if (onDragEnd) onDragEnd();
                        }}
                        onDragOver={(e) => {
                            e.preventDefault();
                            e.dataTransfer.dropEffect = 'move';
                            if (index !== undefined && onDragOver) onDragOver(index);
                        }}
                        onDrop={(e) => {
                            e.preventDefault();
                            if (index !== undefined && onDrop) onDrop(index);
                        }}
                    >
                        <img
                            id={`${id}`}
                            className={`block w-full h-full rounded-lg cursor-pointer hover:border-purple-600 hover:border-2 focus:border-2 focus:border-purple-600 outline-none disabled:cursor-not-allowed disabled:border-0 ${isDragging ? 'cursor-grabbing' : 'cursor-grab'}`}
                            srcSet={undefined}
                            loading={"lazy"}
                            decoding={"async"}
                            src={icon}
                            tabIndex={0}
                            draggable={false}
                            onContextMenu={handleContextMenu}
                            onClick={() => {
                                let elem = document.getElementById(id);
                                // @ts-ignore
                                if (elem.hasAttribute("disabled")) { }
                                else {
                                    setOpenPopup(POPUPS.NONE)
                                    if (setCurrentPage) setCurrentPage(PAGES.NONE)
                                    setCurrentInstall(id)
                                    setDisplayName(name)
                                    setGameIcon(icon)
                                    setBackground(background)
                                    // Defer focus to after React renders — calling focus() synchronously
                                    // blocks the JS thread on WebKitGTK via AT-SPI2 D-Bus before React
                                    // can commit state updates, causing the entire swap to lag ~1s.
                                    setTimeout(() => { elem?.focus({ preventScroll: true }); }, 0);
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
                </div>
            ) : null}
            {(enabled && isOpen && popup == POPUPS.NONE && currentPage === PAGES.NONE) ?
                (typeof window !== "undefined" && window.document &&
                    ReactDOM.createPortal(
                        <div ref={refs.setFloating} style={floatingStyles} {...getFloatingProps()} className="bg-black/75 rounded-md p-2 min-w-max z-50">
                            <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
                            <span className="text-white z-50">{name}</span>
                        </div>,
                        window.document.body
                    )
                ) : null}
            {contextMenu && installSettings && onOpenInstallSettings && onRefreshSettings && (
                <InstallContextMenu
                    installId={id}
                    installSettings={installSettings}
                    x={contextMenu.x}
                    y={contextMenu.y}
                    onClose={() => setContextMenu(null)}
                    onOpenSettings={onOpenInstallSettings}
                    onRefreshSettings={onRefreshSettings}
                />
            )}
        </React.Fragment>
    )
}
