import React from "react";
import { LucideIcon } from "lucide-react";

export interface SettingsTab {
    id: string;
    label: string;
    icon: LucideIcon;
    color: string; // "blue", "green", "yellow", "purple", "orange", "pink", "red", "emerald"
}

interface SettingsSidebarProps {
    tabs: SettingsTab[];
    activeTab: string;
    onTabChange: (id: string) => void;
}

// Pre-defined color styles to avoid runtime string manipulation hackiness
const colorMap: Record<string, { accent: string, glow: string, icon: string, shadow: string }> = {
    blue: {
        accent: "bg-blue-500",
        glow: "bg-blue-500/10",
        icon: "text-blue-400",
        shadow: "shadow-[0_0_15px_rgba(59,130,246,0.5)]"
    },
    green: {
        accent: "bg-green-500",
        glow: "bg-green-500/10",
        icon: "text-green-400",
        shadow: "shadow-[0_0_15px_rgba(34,197,94,0.5)]"
    },
    yellow: {
        accent: "bg-yellow-500",
        glow: "bg-yellow-500/10",
        icon: "text-yellow-400",
        shadow: "shadow-[0_0_15px_rgba(234,179,8,0.5)]"
    },
    purple: {
        accent: "bg-purple-500",
        glow: "bg-purple-500/10",
        icon: "text-purple-400",
        shadow: "shadow-[0_0_15px_rgba(168,85,247,0.5)]"
    },
    orange: {
        accent: "bg-orange-500",
        glow: "bg-orange-500/10",
        icon: "text-orange-400",
        shadow: "shadow-[0_0_15px_rgba(249,115,22,0.5)]"
    },
    pink: {
        accent: "bg-pink-500",
        glow: "bg-pink-500/10",
        icon: "text-pink-400",
        shadow: "shadow-[0_0_15px_rgba(236,72,153,0.5)]"
    },
    red: {
        accent: "bg-red-500",
        glow: "bg-red-500/10",
        icon: "text-red-400",
        shadow: "shadow-[0_0_15px_rgba(239,68,68,0.5)]"
    },
    emerald: {
        accent: "bg-emerald-500",
        glow: "bg-emerald-500/10",
        icon: "text-emerald-400",
        shadow: "shadow-[0_0_15px_rgba(16,185,129,0.5)]"
    },
    default: {
        accent: "bg-zinc-500",
        glow: "bg-zinc-500/10",
        icon: "text-zinc-400",
        shadow: "shadow-[0_0_15px_rgba(113,113,122,0.3)]"
    }
};

export const SettingsSidebar = ({ tabs, activeTab, onTabChange }: SettingsSidebarProps) => {
    const [indicatorStyle, setIndicatorStyle] = React.useState({ top: 0, height: 48 });
    const tabsRef = React.useRef<{ [key: string]: HTMLButtonElement | null }>({});
    const rafRef = React.useRef<number | null>(null);

    React.useEffect(() => {
        // Cancel any pending RAF
        if (rafRef.current) {
            cancelAnimationFrame(rafRef.current);
        }

        // Use RAF to prevent flickering on Linux
        rafRef.current = requestAnimationFrame(() => {
            const activeElement = tabsRef.current[activeTab];
            if (activeElement) {
                setIndicatorStyle({
                    top: activeElement.offsetTop,
                    height: activeElement.offsetHeight
                });
            }
        });

        return () => {
            if (rafRef.current) {
                cancelAnimationFrame(rafRef.current);
            }
        };
    }, [activeTab, tabs]);

    const activeTabObj = tabs.find(t => t.id === activeTab) || tabs[0];
    const styles = colorMap[activeTabObj.color] || colorMap.default;

    return (
        <div className="w-64 flex-shrink-0 h-full border-r border-white/5 bg-transparent p-4 flex flex-col gap-2 overflow-y-auto scrollbar-none relative">
            {/* Floating Active Indicator Background */}
            <div
                className={`absolute left-4 right-4 rounded-xl transition-all duration-300 ease-out pointer-events-none will-change-transform ${styles.glow}`}
                style={{
                    top: indicatorStyle.top,
                    height: indicatorStyle.height,
                    width: 'calc(100% - 2rem)', // account for padding
                }}
            >
                {/* Active Line (Accent Bar) */}
                <div className={`absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 rounded-r-full transition-all duration-300 will-change-transform ${styles.accent} ${styles.shadow}`} />
            </div>

            {tabs.map((tab) => {
                const isActive = activeTab === tab.id;
                const Icon = tab.icon;
                const tabStyles = colorMap[tab.color] || colorMap.default;

                return (
                    <button
                        key={tab.id}
                        ref={el => tabsRef.current[tab.id] = el}
                        onClick={() => onTabChange(tab.id)}
                        className={`
                            relative w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all duration-200 group z-10
                            ${isActive
                                ? "text-white"
                                : "text-zinc-400 hover:text-zinc-100 hover:bg-white/5"
                            }
                        `}
                    >
                        {/* Icon - smooth transition for color */}
                        <Icon
                            className={`w-5 h-5 transition-colors duration-300 transform group-hover:scale-110 ${isActive ? tabStyles.icon : "text-zinc-500 group-hover:text-zinc-300"}`}
                        />

                        <div className="flex flex-col">
                            <span className="font-medium tracking-wide text-sm">{tab.label}</span>
                        </div>
                    </button>
                );
            })}
        </div>
    );
};
