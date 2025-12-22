import React from "react";
import ReactDOM from "react-dom";
import {invoke} from "@tauri-apps/api/core";
import HelpTooltip from "./HelpTooltip.tsx";
import {POPUPS} from "../popups/POPUPS.ts";


export default function SelectMenu({ id, name, options, selected, install, biz, lang, version, dir, fetchInstallSettings, fetchSettings, fetchDownloadSizes, helpText, setOpenPopup, skipGameDownload, onSelect}: { id: string, name: string, options: any, selected: any, multiple: boolean, install?: string, biz?: string, lang?: () => string, version?: () => any, dir?: () => string, helpText: string, fetchInstallSettings?: (id: string) => void, fetchSettings?: () => void, fetchDownloadSizes?: (biz: any, version: any, lang: any, dir: any, callback: (data: any) => void) => void, setOpenPopup: (popup: POPUPS) => void, skipGameDownload?: boolean, onSelect?: (value: string) => void }) {
    // Animation state for closing
    const [animateOut, setAnimateOut] = React.useState(false);
    // Animation state for portal dropdown
    const [animateIn, setAnimateIn] = React.useState(false);
    // Animation state for dropdown visibility
    const [dropdownVisible, setDropdownVisible] = React.useState(false);

    const [search, setSearch] = React.useState("");
    const [open, setOpen] = React.useState(false);
    const [highlighted, setHighlighted] = React.useState(0);
    // Custom dropdown state
    // Ref for click-outside and trigger
    const dropdownRef = React.useRef<HTMLDivElement>(null);
    const triggerRef = React.useRef<HTMLDivElement>(null);

    // Customizable placeholder
    const placeholder = "Select...";

    // Click outside to close
    React.useEffect(() => {
        if (open) {
            setDropdownVisible(true);
            setAnimateOut(false);
        } else if (dropdownVisible) {
            // Start closing animation
            setAnimateOut(true);
            const timeout = setTimeout(() => {
                setDropdownVisible(false);
                setAnimateOut(false);
            }, 200);
            return () => clearTimeout(timeout);
        }
    }, [open, dropdownVisible]);

    // Animate dropdown in when visible
    React.useEffect(() => {
        if (dropdownVisible && !animateOut) {
            setTimeout(() => setAnimateIn(true), 10); // allow portal to mount
        } else {
            setAnimateIn(false);
        }
    }, [dropdownVisible, animateOut]);

    React.useEffect(() => {
        if (!open) return;
        function handleClick(e: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node) &&
                triggerRef.current && !triggerRef.current.contains(e.target as Node)) {
                setOpen(false);
            }
        }
        document.addEventListener("mousedown", handleClick);
        return () => document.removeEventListener("mousedown", handleClick);
    }, [open]);

    // Filter options by search
    const filteredOptions = options.filter((option: any) =>
        option.name.toLowerCase().includes(search.toLowerCase())
    );

    // Get selected option name for display
    const selectedOption = options.find((o: any) => o.value === selected);

    // Handle selection
    function handleSelect(option: any) {
        setOpen(false);
        setSearch("");
        // @ts-ignore
        if (typeof option.value !== "undefined") {
            // Call the onSelect callback if provided
            if (onSelect) {
                onSelect(option.value);
            }
            switch (id) {
                case "game_version": {
                    if (fetchDownloadSizes !== undefined && dir !== undefined && lang !== undefined) {
                        fetchDownloadSizes(biz, `${option.value}`, lang(), dir(), (disk: any) => {
                            let btn = document.getElementById("game_dl_btn");
                            let freedisk = document.getElementById("game_disk_free");
                            if (skipGameDownload || disk.game_decompressed_size_raw <= disk.free_disk_space_raw) {
                                if (btn) btn.removeAttribute("disabled");
                                if (freedisk) {
                                    freedisk.classList.remove("text-red-600");
                                    freedisk.classList.add("text-white");
                                    freedisk.classList.remove("font-bold");
                                }
                            } else {
                                if (btn) btn.setAttribute("disabled", "");
                                if (freedisk) {
                                    freedisk.classList.add("text-red-600");
                                    freedisk.classList.remove("text-white");
                                    freedisk.classList.add("font-bold");
                                }
                            }
                        });
                    }
                }
                break;
                case "game_audio_langs": {
                    if (fetchDownloadSizes !== undefined && dir !== undefined && version !== undefined) {
                        fetchDownloadSizes(biz, version(), `${option.value}`, dir(), (disk: any) => {
                            let btn = document.getElementById("game_dl_btn");
                            let freedisk = document.getElementById("game_disk_free");
                            if (skipGameDownload || disk.game_decompressed_size_raw <= disk.free_disk_space_raw) {
                                if (btn) btn.removeAttribute("disabled");
                                if (freedisk) {
                                    freedisk.classList.remove("text-red-600");
                                    freedisk.classList.add("text-white");
                                    freedisk.classList.remove("font-bold");
                                }
                            } else {
                                if (btn) btn.setAttribute("disabled", "");
                                if (freedisk) {
                                    freedisk.classList.add("text-red-600");
                                    freedisk.classList.remove("text-white");
                                    freedisk.classList.add("font-bold");
                                }
                            }
                        });
                    }
                }
                break;
                case "launcher_action": {
                    if (fetchSettings !== undefined) {
                        invoke("update_settings_launcher_action", {action: `${option.value}`}).then(() => {
                            fetchSettings();
                        });
                    }
                }
                break;
                case "install_fps_value": {
                    if (fetchInstallSettings !== undefined) {
                        invoke("update_install_fps_value", {fps: `${option.value}`, id: install}).then(() => {
                            fetchInstallSettings(install as string);
                        });
                    }
                }
                break;
                case "install_runner_version": {
                    if (fetchInstallSettings !== undefined) {
                        invoke("update_install_runner_version", {version: `${option.value}`, id: install}).then(() => {
                            fetchInstallSettings(install as string);
                            setOpenPopup(POPUPS.NONE);
                        });
                    }
                }
                break;
                case "install_dxvk_version": {
                    if (fetchInstallSettings !== undefined) {
                        invoke("update_install_dxvk_version", {version: `${option.value}`, id: install}).then(() => {
                            fetchInstallSettings(install as string);
                            setOpenPopup(POPUPS.NONE);
                        });
                    }
                }
                break;
                case "tweak_xxmi_hunting": {
                    if (fetchInstallSettings !== undefined) {
                        invoke("update_install_xxmi_config", {xxmiHunting: option.value, id: install}).then(() => {
                            fetchInstallSettings(install as string);
                        });
                    }
                }
                break;
                default:
                    break;
            }
        }
    }

    // Keyboard navigation
    function handleKeyDown(e: React.KeyboardEvent) {
        if (!open) return;
        if (e.key === "ArrowDown") {
            setHighlighted(h => Math.min(h + 1, filteredOptions.length - 1));
        } else if (e.key === "ArrowUp") {
            setHighlighted(h => Math.max(h - 1, 0));
        } else if (e.key === "Enter") {
            if (filteredOptions[highlighted]) handleSelect(filteredOptions[highlighted]);
        } else if (e.key === "Escape") {
            setOpen(false);
        }
    }

    return (
        <div className="flex w-full items-center gap-4 max-sm:flex-col max-sm:items-stretch">
            <span className="text-white text-sm flex items-center gap-1 w-56 shrink-0 max-sm:w-full">{name}
                <HelpTooltip text={helpText}/>
            </span>
            <div ref={dropdownRef} className="inline-flex flex-col items-end justify-end relative ml-auto w-[320px]">
                <div style={{position: 'relative', width: '100%'}}>
                    <div
                        ref={triggerRef}
                        className={`w-full h-10 bg-zinc-800/60 border border-white/30 text-white px-3 pr-10 flex items-center cursor-pointer transition-all duration-200 outline-none rounded-xl ${open ? '' : ''}`}
                        tabIndex={0}
                        style={{userSelect: "none", fontSize: "1rem", position: 'relative'}}
                        onClick={e => {
                            // If click is on the arrow or its area, toggle open/close
                            const arrow = e.target as HTMLElement;
                            if (arrow.classList.contains('dropdown-arrow')) {
                                setOpen(o => !o);
                                return;
                            }
                            if (!open) setOpen(true);
                        }}
                        onKeyDown={e => { if (!open && (e.key === "Enter" || e.key === "ArrowDown")) setOpen(true); }}
                    >
                        {!open ? (
                            <span className="truncate">{selectedOption ? selectedOption.name : placeholder}</span>
                        ) : (
                            <input
                                type="text"
                                className="w-full h-8 bg-transparent text-white px-0 pr-10 flex items-center placeholder-white/50 outline-none border-none rounded-lg"
                                placeholder={placeholder}
                                value={search}
                                autoFocus
                                style={{fontSize: "1rem", background: "none", outline: "none", boxShadow: "none"}}
                                onChange={e => { setSearch(e.target.value); setHighlighted(0); }}
                                onKeyDown={handleKeyDown}
                            />
                        )}
                        <span
                            className="dropdown-arrow absolute right-2 text-white/70 transition-transform duration-200 flex items-center justify-center"
                            style={{transform: open ? "rotate(180deg)" : "rotate(0deg)", top: '50%', translate: '0 -50%', width: '2rem', height: '2rem', cursor: 'pointer'}}
                            onClick={e => { e.stopPropagation(); setOpen(o => !o); }}
                        >
                            ▼
                        </span>
                    </div>
                    {dropdownVisible && ReactDOM.createPortal(
                        (() => {
                            // Calculate position of trigger
                            let rect = {left: 0, top: 0, width: 320, height: 0};
                            if (triggerRef.current) {
                                rect = triggerRef.current.getBoundingClientRect();
                            }
                            // Animation classes
                            const show = animateIn && !animateOut;
                            return (
                                <div
                                    className={`bg-zinc-800 border border-white/30 rounded-xl shadow-lg z-[9999] overflow-hidden transition-all duration-200 ${show ? 'opacity-100 scale-y-100' : ''}${animateOut ? ' opacity-0 scale-y-95 pointer-events-none' : ''}`}
                                    style={{
                                        position: 'fixed',
                                        left: rect.left,
                                        top: rect.top + rect.height + 2,
                                        width: rect.width,
                                        transformOrigin: 'top',
                                        transition: 'opacity 0.2s, transform 0.2s, transform 0.2s',
                                        transform: show ? 'scaleY(1)' : 'scaleY(0.95)',
                                    }}
                                >
                                    <div className="max-h-60 overflow-y-auto scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-zinc-800/60">
                                        {filteredOptions.length === 0 ? (
                                            <div className="px-3 py-2 text-white/80">No matches</div>
                                        ) : (
                                            filteredOptions.map((option: any, idx: number) => {
                                                let rounded = "";
                                                if (highlighted === idx) {
                                                    if (idx === 0) rounded += " rounded-t-lg";
                                                    if (idx === filteredOptions.length - 1) rounded += " rounded-b-lg";
                                                }
                                                return (
                                                    <div
                                                        key={option.value}
                                                        className={`px-3 py-2 cursor-pointer ${highlighted === idx ? "bg-blue-600 text-white" : "hover:bg-white/15 text-white"}${rounded}`}
                                                        onMouseEnter={() => setHighlighted(idx)}
                                                        onMouseDown={e => { e.preventDefault(); handleSelect(option); }}
                                                    >
                                                        {option.name}
                                                    </div>
                                                );
                                            })
                                        )}
                                    </div>
                                </div>
                            );
                        })(),
                        document.body
                    )}
                </div>
            </div>
        </div>
    );
}
