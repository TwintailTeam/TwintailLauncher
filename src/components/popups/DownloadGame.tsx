import {
    Check,
    ChevronDown,
    DownloadCloudIcon,
    Folder,
    Gamepad2Icon,
    HardDrive,
    HardDriveDownloadIcon, Monitor, Terminal,
    X
} from "lucide-react";
import { POPUPS } from "./POPUPS.ts";
import { PAGES } from "../pages/PAGES.ts";
import { CachedImage } from "../common/CachedImage";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { useState, useEffect, useRef } from "react";
import { formatBytes } from "../../utils/progress";
import { open } from "@tauri-apps/plugin-dialog";
import {ModernPathInput, ModernSelect, ModernToggle} from "../common/SettingsComponents.tsx";

interface IProps {
    disk: any;
    setOpenPopup: any;
    displayName: string;
    settings: any;
    biz: string;
    versions: { value: string; name: string; background?: string; liveBackground?: string }[];
    background: string;
    icon: string;
    pushInstalls: () => void;
    runnerVersions: any[];
    dxvkVersions: any[];
    setCurrentInstall: (installId: string) => void;
    setBackground: (background: string) => void;
    fetchDownloadSizes: (biz: any, version: any, lang: any, path: any, region_filter: any, callback: (data: any) => void) => void;
    openAsExisting?: boolean;
    setCurrentPage: (page: PAGES) => void;
    imageVersion?: number; // Used to force image re-load after network recovery
}

export default function DownloadGame({ disk, setOpenPopup, displayName, settings, biz, versions, background, icon, pushInstalls, runnerVersions, dxvkVersions, setCurrentInstall, fetchDownloadSizes, openAsExisting, setCurrentPage, imageVersion = 0 }: IProps) {
    const [skipGameDownload, setSkipGameDownload] = useState(!!openAsExisting);
    useEffect(() => { setSkipGameDownload(!!openAsExisting); }, [openAsExisting]);
    const [selectedGameVersion, setSelectedGameVersion] = useState(versions?.[0]?.value || "");
    const [isVersionOpen, setIsVersionOpen] = useState(false);

    // Local state for popup banner (don't change main app background)
    const [popupBanner, setPopupBanner] = useState(() => {
        const selectedVersion = versions.find(v => v.value === versions?.[0]?.value);
        return selectedVersion?.background || background;
    });

    // @ts-ignore
    const [selectedAudioLang, setSelectedAudioLang] = useState("en-us");
    // @ts-ignore
    const [selectedRegionCode, setSelectedRegionCode] = useState("glb_official");

    const [selectedRunnerVersion, setSelectedRunnerVersion] = useState(runnerVersions?.[0]?.value || "");
    // @ts-ignore
    const [selectedDxvkVersion, setSelectedDxvkVersion] = useState(dxvkVersions?.[0]?.value || "");

    const dropdownRef = useRef<HTMLDivElement>(null);

    // Close dropdown on outside click
    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsVersionOpen(false);
            }
        }
        document.addEventListener("mousedown", handleClickOutside);
        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
        };
    }, [dropdownRef]);

    // Controlled State for Install Path
    const [installPath, setInstallPath] = useState(`${settings.default_game_path}/${biz}`);
    // Controlled State for Proton Prefix Path
    const [runnerPrefixPath, setRunnerPrefixPath] = useState(`${settings.default_runner_prefix_path}/${biz}`);
    // Controlled State for skip version updates
    const [skipVersionUpdates, setSkipVersionUpdates] = useState(false);
    // Controlled State for disable hash validation
    const [disableHashValidation, setDisableHashValidation] = useState(false);

    // Update path effect to fetch sizes
    useEffect(() => {
        if (installPath && selectedGameVersion && fetchDownloadSizes) {
            fetchDownloadSizes(biz, selectedGameVersion, selectedAudioLang, installPath, selectedRegionCode, () => { });
        }
    }, [installPath, selectedGameVersion, selectedAudioLang, selectedRegionCode]);

    // Animation state
    const [isClosing, setIsClosing] = useState(false);

    // Disk space logic
    const freeSpace = parseFloat(disk.free_disk_space_raw || 0);
    const totalSpace = parseFloat(disk.total_disk_space_raw || 0);
    const requiredSpace = parseFloat(disk.game_decompressed_size_raw || 0);
    const usedSpace = totalSpace > 0 ? totalSpace - freeSpace : 0;
    const hasEnoughSpace = skipGameDownload || (freeSpace > requiredSpace);
    const usedPercent = totalSpace > 0 ? (usedSpace / totalSpace) * 100 : 0;
    const gamePercent = totalSpace > 0 ? Math.min((requiredSpace / totalSpace) * 100, 100 - usedPercent) : 0;

    // Update button state when skipGameDownload changes
    useEffect(() => {
        const btn = document.getElementById("game_dl_btn");
        if (btn) {
            if (skipGameDownload) {
                btn.removeAttribute("disabled");
            } else {
                if (!hasEnoughSpace) {
                    btn.setAttribute("disabled", "");
                } else {
                    btn.removeAttribute("disabled");
                }
            }
        }
    }, [skipGameDownload, hasEnoughSpace]);

    const handleBrowse = async () => {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: installPath,
        });
        if (selected && typeof selected === "string") {
            setInstallPath(selected);
        }
    };

    const handleInstall = () => {
        setIsClosing(true);
        setTimeout(() => {
            let install_path = installPath;
            let gvv = selectedGameVersion;
            let vpp = selectedAudioLang;
            let rvv = selectedRunnerVersion || "none";
            let dvv = selectedDxvkVersion || "none";
            invoke("add_install", {
                manifestId: biz,
                version: gvv,
                audioLang: vpp,
                name: displayName,
                directory: install_path,
                runnerPath: "none",
                dxvkPath: "none",
                runnerVersion: rvv,
                dxvkVersion: dvv,
                gameIcon: icon,
                gameBackground: popupBanner,
                ignoreUpdates: skipVersionUpdates,
                skipHashCheck: disableHashValidation,
                useJadeite: false,
                useXxmi: false,
                useFpsUnlock: false,
                envVars: "",
                preLaunchCommand: "",
                launchCommand: "",
                fpsValue: "60",
                runnerPrefix: runnerPrefixPath,
                launchArgs: "",
                skipGameDl: skipGameDownload,
                regionCode: selectedRegionCode
            }).then((r: any) => {
                if (r.success) {
                    pushInstalls();
                    setCurrentInstall(r.install_id as string);
                    setTimeout(() => {
                        let installui = document.getElementById(r.install_id);
                        if (installui) installui.focus();
                        if (!skipGameDownload && (!r.skip_dl || !r.steam_imported)) {
                            emit("start_game_download", { install: r.install_id, biz: biz, lang: vpp, region: selectedRegionCode }).then(() => { });
                        }
                    }, 20);
                } else {
                    console.error("Download error!");
                }
            });
            setOpenPopup(POPUPS.NONE);
        }, 300);
    };

    return (
        <div className={`relative w-[95vw] max-w-4xl h-auto max-h-[90vh] flex flex-col overflow-hidden rounded-2xl shadow-2xl border border-white/10 bg-[#0c0c0c] group/download ${isClosing ? 'animate-zoom-out' : 'animate-zoom-in'}`} style={{ willChange: 'transform, opacity', backfaceVisibility: 'hidden', WebkitBackfaceVisibility: 'hidden' as any, transform: 'translateZ(0)' }}>
            {/* Hero Header */}
            <div className="relative h-48 w-full flex-shrink-0">
                {popupBanner && (
                    <div className="absolute inset-0 bg-zinc-900">
                        <CachedImage key={`banner-v${imageVersion}`} src={popupBanner} className="w-full h-full object-cover opacity-80" alt="Game Background" />
                        {/* Extended past bottom edge to fix WebKitGTK subpixel rendering gap */}
                        <div className="absolute top-0 left-0 right-0 -bottom-1 bg-gradient-to-b from-black/20 via-black/40 to-[#0c0c0c]" />
                    </div>
                )}

                <div className="absolute top-4 right-4 z-20">
                    <button onClick={() => { setIsClosing(true); setTimeout(() => setOpenPopup(POPUPS.NONE), 200); }} className="p-2 rounded-full bg-black/60 hover:bg-white/10 border border-white/5 transition-all duration-200 hover:scale-105 opacity-0 group-hover/download:opacity-100">
                        <X className="w-6 h-6 text-white/70 group-hover:text-white" />
                    </button>
                </div>

                <div className="absolute bottom-6 left-8 z-10 flex items-end gap-6">
                    {icon && (
                        <div className="w-24 h-24 rounded-2xl overflow-hidden shadow-2xl border-2 border-white/10 bg-black/80 hidden sm:block">
                            <CachedImage key={`icon-v${imageVersion}`} src={icon} className="w-full h-full object-cover" alt="Icon" />
                        </div>
                    )}
                    <div className="mb-1">
                        <h1 className="text-4xl font-bold text-white tracking-tight drop-shadow-md">{skipGameDownload ? "Add Installation" : "Install Game"}</h1>
                        <div className="flex items-center gap-3 mt-1 relative">
                            <span className="text-white/80 font-medium text-lg">{displayName}</span>
                            {/* Version Pill */}
                            <div className="relative" ref={dropdownRef}>
                                <button onClick={() => setIsVersionOpen(!isVersionOpen)} className="flex items-center gap-2 px-3 py-1 rounded-full bg-purple-500/20 hover:bg-purple-500/30 border border-purple-500/30 hover:border-purple-500/50 transition-all cursor-pointer group">
                                    <span className="text-purple-300 font-bold text-sm">{selectedGameVersion}</span>
                                    <ChevronDown size={14} className={`text-purple-400 transition-transform duration-200 ${isVersionOpen ? 'rotate-180' : ''}`} />
                                </button>
                                {/* Dropdown */}
                                {isVersionOpen && (
                                    <div className="absolute top-full left-0 mt-2 w-48 max-h-60 overflow-y-auto bg-[#1a1a1a] border border-white/10 rounded-xl shadow-2xl z-50 py-1 scrollbar-thin scrollbar-track-transparent scrollbar-thumb-white/20 hover:scrollbar-thumb-white/30">
                                        {versions.map((v) => (
                                            <button key={v.value} onClick={() => {
                                                    setSelectedGameVersion(v.value);
                                                    setIsVersionOpen(false);
                                                    // Update local popup banner (don't change main app background)
                                                    const newBackground = v.background;
                                                    if (newBackground) {
                                                        setPopupBanner(newBackground);
                                                    }
                                                }} className="w-full px-4 py-2 text-left hover:bg-white/10 flex items-center justify-between group">
                                                <span className={`text-sm ${selectedGameVersion === v.value ? 'text-white font-bold' : 'text-zinc-400 group-hover:text-white'}`}>
                                                    {v.name}
                                                </span>
                                                {selectedGameVersion === v.value && <Check size={14} className="text-purple-400" />}
                                            </button>
                                        ))}
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Content Body */}
            <div className="flex-1 overflow-y-auto overflow-x-hidden p-8 space-y-8 bg-[#0c0c0c]/80 scrollbar-thin scrollbar-track-transparent scrollbar-thumb-white/20 hover:scrollbar-thumb-white/30">
                {/* Location Section - Custom Card Design */}
                <div className="space-y-4">
                    <h3 className="text-white/90 font-semibold text-lg flex items-center gap-2">
                        <HardDrive className="text-purple-400" size={20} /> Installation Destination
                    </h3>

                    <div className="bg-gradient-to-br from-white/5 to-white/0 rounded-xl border border-white/10 overflow-hidden">
                        <div className="p-4 flex items-center gap-4">
                            <div className="w-12 h-12 rounded-lg bg-black/40 flex items-center justify-center shrink-0 border border-white/5">
                                <Folder className="text-white/50" size={24} />
                            </div>
                            <div className="flex-1 min-w-0">
                                <div className="text-xs text-white/40 uppercase font-bold tracking-wider mb-1">Install Path</div>
                                <div className="text-white/90 font-medium truncate font-mono text-sm" title={installPath}>
                                    {installPath}
                                </div>
                            </div>
                            <button onClick={handleBrowse} className="px-4 py-2 rounded-lg bg-white/5 hover:bg-white/10 text-white border border-white/5 hover:border-white/20 transition-all text-sm font-medium">
                                Change
                            </button>
                        </div>
                        {/* Disk Space Visual - Integrated */}
                        {!skipGameDownload && (
                            <div className="bg-black/20 px-4 pb-4 border-t border-white/5">
                                <div className="flex items-center justify-between pt-3 mb-2.5">
                                    <span className="text-xs text-white/40 uppercase font-bold tracking-wider">Storage</span>
                                    {totalSpace > 0 && <span className="text-xs text-white/20 font-mono">{formatBytes(totalSpace)} drive</span>}
                                </div>
                                {totalSpace > 0 ? (
                                    <>
                                        {/* 3-segment bar: used | game | free */}
                                        <div className="relative w-full h-2 bg-white/5 rounded-full overflow-hidden">
                                            <div className="absolute left-0 top-0 h-full bg-gradient-to-r from-blue-400 to-blue-600 transition-all duration-700" style={{ width: `${Math.min(usedPercent, 100)}%` }} />
                                            <div className={`absolute top-0 h-full transition-all duration-700 ${hasEnoughSpace ? 'bg-gradient-to-r from-violet-400 to-purple-600' : 'bg-gradient-to-r from-red-400 to-red-600'}`} style={{ left: `${Math.min(usedPercent, 100)}%`, width: `${gamePercent}%` }} />
                                        </div>
                                        {/* Legend */}
                                        <div className="flex items-center justify-between mt-2.5 text-xs gap-x-3 gap-y-1 flex-wrap">
                                            <div className="flex items-center gap-1.5">
                                                <span className="w-2 h-2 rounded-sm bg-blue-500/50 shrink-0" />
                                                <span className="text-blue-400">Used</span>
                                                <span className="text-blue-300 font-medium tabular-nums">{formatBytes(usedSpace)}</span>
                                            </div>
                                            <div className="flex items-center gap-1.5">
                                                <span className={`w-2 h-2 rounded-sm shrink-0 ${hasEnoughSpace ? 'bg-violet-400' : 'bg-red-400'}`} />
                                                <span className={hasEnoughSpace ? 'text-purple-300/70' : 'text-red-300/70'}>Game</span>
                                                <span className={`font-medium tabular-nums ${hasEnoughSpace ? 'text-purple-300' : 'text-red-400'}`}>{formatBytes(requiredSpace)}</span>
                                            </div>
                                            {hasEnoughSpace ? (
                                                <div className="flex items-center gap-1.5">
                                                    <span className="w-2 h-2 rounded-sm bg-emerald-400/50 shrink-0" />
                                                    <span className="text-emerald-400/70">Free after</span>
                                                    <span className="text-emerald-400 font-medium tabular-nums">{formatBytes(Math.max(0, freeSpace - requiredSpace))}</span>
                                                </div>
                                            ) : (
                                                <div className="flex items-center gap-1 text-red-400">
                                                    <X size={11} />
                                                    <span className="font-medium">Need {formatBytes(requiredSpace - freeSpace)} more</span>
                                                </div>
                                            )}
                                        </div>
                                    </>
                                ) : (
                                    <>
                                        <div className="w-full bg-zinc-800/50 rounded-full h-1.5 overflow-hidden">
                                            <div className={`h-full rounded-full transition-all duration-500 ${hasEnoughSpace ? 'bg-gradient-to-r from-purple-500 to-blue-500' : 'bg-red-500'}`} style={{ width: `${Math.min(100, Math.max(2, freeSpace > 0 ? (requiredSpace / freeSpace) * 100 : 0))}%` }} />
                                        </div>
                                        <div className="flex justify-between text-xs mt-2">
                                            <span className={hasEnoughSpace ? 'text-purple-300' : 'text-red-400'}>{formatBytes(requiredSpace)} required</span>
                                            <span className="text-zinc-500">{formatBytes(freeSpace)} free</span>
                                        </div>
                                        {!hasEnoughSpace && (
                                            <p className="text-red-400 text-xs mt-1 font-medium flex items-center gap-1">
                                                <X size={12} /> Insufficient disk space
                                            </p>
                                        )}
                                    </>
                                )}
                            </div>
                        )}
                    </div>
                </div>
                {biz === "bh3_global" && (
                    <div className="space-y-4">
                        <h3 className="text-white/90 font-semibold text-lg flex items-center gap-2">
                            <Gamepad2Icon className="text-emerald-400" size={20} /> Game specific
                        </h3>
                        <ModernSelect
                            label="Region"
                            description="Game region you want downloaded."
                            value={selectedRegionCode || ""}
                            options={[{ name: "Europe & America", value: "glb_official" }, { name: "Japan", value: "jp_official" }, { name: "Korea", value: "kr_official" }, { name: "SEA", value: "overseas_official" }, { name: "Traditional Chinese", value: "asia_official" }]}
                            onChange={(val) => setSelectedRegionCode(val)}
                        />
                    </div>
                )}
                {window.navigator.platform.includes("Linux") && (
                    <div className="space-y-4">
                        <h3 className="text-white/90 font-semibold text-lg flex items-center gap-2">
                            <Monitor className="text-orange-400" size={20} /> Linux options
                        </h3>
                        <div className="flex flex-col gap-2">
                            <ModernSelect
                                label="Runner Version"
                                description="Select the Wine/Proton version to download with the game."
                                value={selectedRunnerVersion || ""}
                                options={runnerVersions}
                                onChange={(val) => setSelectedRunnerVersion(val)}
                            />
                            <button onClick={() => {
                                setOpenPopup(POPUPS.NONE);
                                setCurrentPage(PAGES.RUNNERS);
                            }} className="text-purple-400 hover:text-purple-300 text-sm font-medium transition-colors text-left px-1 underline-offset-2 hover:underline">
                                → Manage Runners
                            </button>
                        </div>
                        <ModernPathInput
                            label="Prefix Path"
                            description="Path to the Wine/Proton prefix."
                            value={runnerPrefixPath || ""}
                            onChange={(val) => setRunnerPrefixPath(val)}
                        />
                    </div>
                )}
                <div className="space-y-4">
                    <h3 className="text-white/90 font-semibold text-lg flex items-center gap-2">
                        <Terminal className="text-blue-400" size={20} /> Advanced options
                    </h3>
                    <ModernToggle
                        label="Skip Version Checks"
                        description="Don't check for game updates."
                        checked={skipVersionUpdates || false}
                        onChange={(val) => setSkipVersionUpdates(val)}
                    />
                    <ModernToggle
                        label="Skip Hash Validation"
                        description="Skip file verification during repairs (faster but less safe)."
                        checked={disableHashValidation || false}
                        onChange={(val) => setDisableHashValidation(val)}
                    />
                </div>
            </div>

            {/* Footer Actions */}
            <div className="p-6 bg-[#0c0c0c]/90 border-t border-white/5 flex flex-col-reverse sm:flex-row justify-end gap-3 z-20">
                <button onClick={() => { setIsClosing(true); setTimeout(() => setOpenPopup(POPUPS.NONE), 200); }} className="px-6 py-3 rounded-xl text-white/50 hover:text-white hover:bg-white/5 transition-all font-medium text-sm">
                    Cancel
                </button>
                <button
                    id="game_dl_btn"
                    onClick={handleInstall}
                    disabled={!hasEnoughSpace && !skipGameDownload}
                    className={`
                        relative overflow-hidden px-8 py-3 rounded-xl font-bold text-white shadow-lg transition-all duration-200 transform hover:scale-[1.02] active:scale-[0.98]
                        disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none
                        ${skipGameDownload
                            ? 'bg-emerald-600 hover:bg-emerald-500 shadow-emerald-900/20'
                            : 'bg-purple-600 hover:bg-purple-500 shadow-purple-900/30'
                        }
                    `}
                >
                    <div className="flex items-center gap-3 relative z-10">
                        {skipGameDownload ? <HardDriveDownloadIcon size={20} /> : <DownloadCloudIcon size={20} />}
                        <span>{skipGameDownload ? "Locate Game" : "Install Game"}</span>
                    </div>
                </button>
            </div>

        </div>
    );
}
