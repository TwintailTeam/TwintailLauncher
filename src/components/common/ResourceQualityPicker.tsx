import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, Download, Layers } from "lucide-react";
import { SettingsCard } from "./SettingsComponents.tsx";
import { formatBytes } from "../../utils/progress";
import { translate } from "../../utils/i18n";

// Wuthering Waves ships its textures as SD/HD/UHD packs on top of a shared common pack
export type ResourceQuality = "sd" | "hd" | "uhd";
export const RESOURCE_QUALITIES: ResourceQuality[] = ["sd", "hd", "uhd"];
export const DEFAULT_RESOURCE_QUALITY: ResourceQuality = "hd";
export const qualityLabel = (q: string) => q.toUpperCase();

export interface ResourceQualityInfo {
    supported: boolean;
    selected: ResourceQuality;
    installed: ResourceQuality[];
    sizes: Partial<Record<ResourceQuality, { download_bytes: number; install_bytes: number }>>;
    version: string;
}

// Fetches the quality packs Kuro offers (sizes) and, for an existing install, what is on disk
export function useResourceQualityInfo(biz: string, installId?: string, refreshKey?: unknown) {
    const [info, setInfo] = useState<ResourceQualityInfo | null>(null);
    useEffect(() => {
        let alive = true;
        invoke<ResourceQualityInfo>("get_resource_quality_info", { biz, installId: installId ?? null })
            .then((r) => { if (alive) setInfo(r); })
            .catch((e) => console.error("Failed to load resource quality info:", e));
        return () => { alive = false; };
    }, [biz, installId, refreshKey]);
    return info;
}

interface TilesProps {
    value: ResourceQuality;
    onChange: (q: ResourceQuality) => void;
    sizes?: ResourceQualityInfo["sizes"];
    installed?: ResourceQuality[];
    disabled?: boolean;
}

export function ResourceQualityTiles({ value, onChange, sizes, installed = [], disabled = false }: TilesProps) {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3" role="radiogroup" aria-label={translate("resource_quality.title")}>
            {RESOURCE_QUALITIES.map((q) => {
                const active = q === value;
                const size = sizes?.[q]?.install_bytes;
                const isInstalled = installed.includes(q);
                return (
                    <button
                        key={q}
                        type="button"
                        role="radio"
                        aria-checked={active}
                        disabled={disabled}
                        onClick={() => { if (!active) onChange(q); }}
                        className={`relative flex flex-col items-start gap-1.5 rounded-xl border p-4 text-left transition-all duration-200
                            ${active
                                ? "border-purple-500/60 bg-purple-500/15 shadow-[0_0_15px_rgba(147,51,234,0.25)]"
                                : "border-white/10 bg-black/30 hover:border-white/20 hover:bg-white/5"}
                            ${disabled ? "cursor-not-allowed opacity-60" : "cursor-pointer"}`}
                    >
                        <div className="flex w-full items-center justify-between gap-2">
                            <span className={`text-lg font-bold tracking-tight ${active ? "text-white" : "text-white/80"}`}>{qualityLabel(q)}</span>
                            <span className={`flex h-5 w-5 shrink-0 items-center justify-center rounded-full border transition-colors ${active ? "border-purple-400 bg-purple-600" : "border-white/20"}`}>
                                {active && <Check size={12} className="text-white" />}
                            </span>
                        </div>
                        <div className="flex flex-wrap gap-1.5">
                            {q === DEFAULT_RESOURCE_QUALITY && (
                                <span className="rounded-full border border-purple-500/30 bg-purple-500/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-purple-300">{translate("resource_quality.recommended")}</span>
                            )}
                            {isInstalled && (
                                <span className="rounded-full border border-emerald-400/30 bg-emerald-500/15 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-emerald-300">{translate("resource_quality.installed")}</span>
                            )}
                        </div>
                        <span className="text-xs text-zinc-400">{translate(`resource_quality.option.${q}`)}</span>
                        <span className={`font-mono text-xs tabular-nums ${active ? "text-purple-300" : "text-white/40"}`}>{size ? formatBytes(size) : "—"}</span>
                    </button>
                );
            })}
        </div>
    );
}

interface CardProps {
    biz: string;
    installId: string;
    steamImported?: boolean;
    onSelect: (q: ResourceQuality) => Promise<void> | void;
    onApply: () => void;
}

// Game settings card: picking a tile saves the preference, Apply queues the download/removal
export function ResourceQualityCard({ biz, installId, steamImported = false, onSelect, onApply }: CardProps) {
    const [refresh, setRefresh] = useState(0);
    const info = useResourceQualityInfo(biz, installId, refresh);
    const [selected, setSelected] = useState<ResourceQuality | null>(null);
    useEffect(() => { if (info) setSelected(info.selected); }, [info]);
    if (!info?.supported || !selected) return null;

    const installed = info.installed;
    const pending = !steamImported && installed.length > 0 && !installed.includes(selected);
    const installedLabel = installed.map(qualityLabel).join(" + ");
    const size = info.sizes?.[selected]?.download_bytes;

    const pick = async (q: ResourceQuality) => {
        setSelected(q);
        await onSelect(q);
        setRefresh((r) => r + 1);
    };

    return (
        <SettingsCard>
            <div className="flex flex-col gap-4">
                <div className="flex items-start gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-white/5 bg-black/40">
                        <Layers className="text-purple-400" size={20} />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-base font-medium text-white">{translate("resource_quality.title")}</label>
                        <span className={`text-sm ${steamImported ? "text-yellow-300 font-medium" : "text-zinc-400"}`}>
                            {steamImported ? translate("resource_quality.steam_managed") : translate("resource_quality.description")}
                        </span>
                    </div>
                </div>
                <ResourceQualityTiles value={selected} onChange={pick} sizes={info.sizes} installed={installed} disabled={steamImported} />
                {pending && (
                    <div className="flex flex-col gap-3 rounded-lg border border-purple-500/20 bg-purple-500/10 p-3 sm:flex-row sm:items-center">
                        <span className="flex-1 text-sm text-purple-100/80">
                            {size
                                ? translate("resource_quality.pending", { installed: installedLabel, target: qualityLabel(selected), size: formatBytes(size) })
                                : translate("resource_quality.pending_no_size", { installed: installedLabel, target: qualityLabel(selected) })}
                        </span>
                        <button
                            type="button"
                            onClick={onApply}
                            className="flex shrink-0 items-center justify-center gap-2 rounded-lg bg-purple-600 px-4 py-2 text-sm font-bold text-white shadow-lg shadow-purple-900/30 transition-all hover:bg-purple-500 active:scale-[0.98]"
                        >
                            <Download size={16} />
                            {translate("resource_quality.apply")}
                        </button>
                    </div>
                )}
            </div>
        </SettingsCard>
    );
}
