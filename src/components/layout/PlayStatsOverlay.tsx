import { CalendarDays, Clock3 } from "lucide-react";
import { translate } from "../../utils/i18n";

interface PlayStatsOverlayProps {
    lastPlayedTime?: string | number;
    totalPlaytime?: string | number;
    isVisible: boolean;
    appLang?: string;
}

function formatLastPlayed(value?: string | number, appLang?: string): string {
    if (value === undefined || value === null || value === "") return translate("play_stats.never");
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed <= 0) return translate("play_stats.never");

    const timestampMs = parsed > 1_000_000_000_000 ? parsed : parsed * 1000;
    const date = new Date(timestampMs);
    if (Number.isNaN(date.getTime())) return translate("play_stats.never");

    const now = new Date();
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const startOfDate = new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
    const dayDiff = Math.floor((startOfToday - startOfDate) / 86400000);

    if (dayDiff === 0) return translate("play_stats.today");
    if (dayDiff === 1) return translate("play_stats.yesterday");

    const intlLocale = appLang ? appLang.replace(/_/g, "-") : undefined;
    return date.toLocaleDateString(intlLocale, {
        month: "short",
        day: "numeric",
        year: "numeric",
    });
}

function formatPlaytimeHours(value?: string | number): string {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed <= 0) return `0 ${translate("play_stats.minutes")}`;

    const totalSeconds = Math.floor(parsed);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);

    const hourLabel = hours === 1 ? translate("play_stats.hour") : translate("play_stats.hours");
    const minuteLabel = minutes === 1 ? translate("play_stats.minute") : translate("play_stats.minutes");

    if (hours <= 0) return `${new Intl.NumberFormat().format(minutes)} ${minuteLabel}`;
    if (minutes <= 0) return `${new Intl.NumberFormat().format(hours)} ${hourLabel}`;
    return `${new Intl.NumberFormat().format(hours)} ${hourLabel} ${minutes} ${minuteLabel}`;
}

export default function PlayStatsOverlay({
    lastPlayedTime,
    totalPlaytime,
    isVisible,
    appLang,
}: PlayStatsOverlayProps) {
    if (!isVisible) return null;

    return (
        <div className="absolute top-4 right-6 z-20 animate-slideInRight pointer-events-none" style={{ animationDelay: "120ms" }}>
            <div className="rounded-xl border border-white/10 bg-black/50 shadow-lg">
                <div className="flex items-stretch divide-x divide-white/10">
                    <div className="px-3 py-2.5 min-w-[175px]">
                        <div className="flex items-center gap-2.5">
                            <Clock3 className="w-5 h-5 text-white/55 flex-shrink-0" />
                            <div className="min-w-0">
                                <div className="text-[11px] font-semibold uppercase tracking-wider text-white/60">{translate("play_stats.play_time")}</div>
                                <div className="mt-0.5 text-sm font-semibold text-white">{formatPlaytimeHours(totalPlaytime)}</div>
                            </div>
                        </div>
                    </div>
                    <div className="px-3 py-2.5 min-w-[175px]">
                        <div className="flex items-center gap-2.5">
                            <CalendarDays className="w-5 h-5 text-white/55 flex-shrink-0" />
                            <div className="min-w-0">
                                <div className="text-[11px] font-semibold uppercase tracking-wider text-white/60">{translate("play_stats.last_played")}</div>
                                <div className="mt-0.5 text-sm font-semibold text-white">{formatLastPlayed(lastPlayedTime, appLang)}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
