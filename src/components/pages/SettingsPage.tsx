import {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/core";
import { Settings, Download, Folder, Info, Monitor, ArrowLeft, HeartIcon } from "lucide-react";
import { SettingsSidebar, SettingsTab } from "../sidebar/SettingsSidebar.tsx";
import {
    SettingsSection,
    ModernInput,
    ModernPathInput,
    ModernSelect,
    ModernToggle
} from "../common/SettingsComponents.tsx";
import { PAGES } from "./PAGES";
import {getVersion} from "@tauri-apps/api/app";

interface SettingsPageProps {
    settings: any;
    fetchSettings: () => void;
    setCurrentPage: (page: PAGES) => void;
}

export default function SettingsPage({ settings, fetchSettings, setCurrentPage }: SettingsPageProps) {
    const [activeTab, setActiveTab] = useState("general");

    const tabs: SettingsTab[] = [
        { id: "general", label: "General", icon: Settings, color: "blue" },
        { id: "downloads", label: "Downloads", icon: Download, color: "green" },
        { id: "files", label: "Files & Paths", icon: Folder, color: "yellow" },
        ...(window.navigator.platform.includes("Linux") ? [/*{ id: "integrations", label: "Integrations & Tools", icon: Box, color: "purple" }*/] : []),
        ...(window.navigator.platform.includes("Linux") ? [{ id: "linux", label: "Linux Options", icon: Monitor, color: "orange" }] : []),
        { id: "about", label: "About", icon: Info, color: "pink" },
    ];

    // Helper to update settings
    const updateSetting = async (key: string, value: any) => {
        try {
            if (typeof value === "boolean") {
                await invoke(`update_settings_${key}`, { enabled: value });
            } else if (typeof value === "string" || typeof value === "number") {
                if (key === "download_speed_limit") {
                    await invoke("update_settings_download_speed_limit_cmd", { speedLimit: Number(value) });
                } else if (key === "launcher_action") {
                    await invoke(`update_settings_${key}`, { action: value });
                } else if (key === "xxmi_path") {
                    await invoke("update_settings_default_xxmi_path", { path: value });
                } else if (key === "fps_unlock_path") {
                    await invoke("update_settings_default_fps_unlock_path", { path: value });
                } else if (key === "jadeite_path") {
                    await invoke("update_settings_default_jadeite_path", { path: value });
                } else {
                    await invoke(`update_settings_${key}`, { path: value });
                }
            }
            fetchSettings();
        } catch (e) {
            console.error(`Failed to update setting ${key}:`, e);
        }
    };

    // Track animation class state
    const [animClass, setAnimClass] = useState("animate-fadeIn");

    // Update active tab and determine direction
    const handleTabChange = (newTabId: string) => {
        const oldIndex = tabs.findIndex(t => t.id === activeTab);
        const newIndex = tabs.findIndex(t => t.id === newTabId);
        const direction = newIndex > oldIndex ? "animate-slideUp" : "animate-slideDown";
        setAnimClass(direction);
        setActiveTab(newTabId);
    };

    const [version, setVersion] = useState('');
    const branch = import.meta.env.MODE || "unknown";
    useEffect(() => {
        getVersion().then(setVersion);
    }, []);

    return (
        <div
            className="flex-1 flex flex-col h-full overflow-hidden animate-fadeIn"
            style={{ willChange: 'opacity', backfaceVisibility: 'hidden', transform: 'translateZ(0)' }}
        >
            {/* Page Header */}
            <div className="flex items-center gap-4 px-8 py-5 border-b border-white/5">
                <button
                    onClick={() => setCurrentPage(PAGES.NONE)}
                    className="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/10 transition-all duration-200 hover:scale-105 hover:shadow-[0_0_12px_rgba(147,51,234,0.15)] active:scale-95">
                    <ArrowLeft className="w-5 h-5 text-white/70" />
                </button>
                <div className="flex items-center gap-4">
                    <div className="p-3 bg-purple-500/15 rounded-xl border border-purple-500/20 shadow-[0_0_15px_rgba(147,51,234,0.2)]">
                        <Settings className="w-6 h-6 text-purple-400" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-bold bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent">
                            Launcher Settings
                        </h1>
                        <p className="text-sm text-white/50">Configure your launcher preferences</p>
                    </div>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex flex-1 overflow-hidden">
                <SettingsSidebar tabs={tabs} activeTab={activeTab} onTabChange={handleTabChange} />

                {/* Content Area */}
                <div
                    key={activeTab}
                    className={`flex-1 overflow-y-auto p-8 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent ${animClass}`}
                >
                    {activeTab === "general" && (
                        <SettingsSection title="General Options">
                            <ModernToggle
                                label="Minimize application"
                                description="Hide application to system tray instead of completely closing."
                                checked={Boolean(settings.hide_app_to_tray)}
                                onChange={(val) => updateSetting("hide_app_tray", val)}
                            />
                            <ModernSelect
                                label="After Game Launch"
                                description="Choose what the launcher should do when a game starts."
                                value={`${settings.launcher_action}`}
                                options={[
                                    { value: "exit", label: "Close launcher" },
                                    { value: "keep", label: "Keep launcher open" },
                                    { value: "minimize", label: "Minimize to system tray" }
                                ]}
                                onChange={(val) => updateSetting("launcher_action", val)}
                            />
                            {/*<ModernToggle
                                label="Auto-update 3rd Party Repos"
                                description="Automatically update third-party repositories and their manifests on startup."
                                checked={Boolean(settings.third_party_repo_updates)}
                                onChange={(val) => updateSetting("third_party_repo_updates", val)}
                            />*/}
                        </SettingsSection>
                    )}

                    {activeTab === "downloads" && (
                        <SettingsSection title="Download Manager">
                            <ModernInput
                                label="Download Speed Limit (KB/s)"
                                description="Limit the total download bandwidth. Set to 0 for unlimited."
                                type="number"
                                min={0}
                                value={settings.download_speed_limit ?? 0}
                                onChange={(e) => updateSetting("download_speed_limit", e.target.value)}
                            />
                        </SettingsSection>
                    )}

                    {activeTab === "files" && (
                        <>
                            <SettingsSection title="Games">
                                <ModernPathInput
                                    label="Default Game Install Location"
                                    description="Default base directory where new games will be installed."
                                    value={`${settings.default_game_path}`}
                                    onChange={(val) => updateSetting("default_game_path", val)}
                                />
                            </SettingsSection>
                            <SettingsSection title="External Tools">
                                <ModernPathInput
                                    label="XXMI Location"
                                    description="Directory for XXMI modding tool files."
                                    value={`${settings.xxmi_path}`}
                                    onChange={(val) => updateSetting("xxmi_path", val)}
                                />
                                <ModernPathInput
                                    label="FPS Unlocker Location"
                                    description="Directory where the FPS unlocker is stored."
                                    value={`${settings.fps_unlock_path}`}
                                    onChange={(val) => updateSetting("fps_unlock_path", val)}
                                />
                            </SettingsSection>
                        </>
                    )}

                    {activeTab === "linux" && (
                        <SettingsSection title="Linux Configuration">
                            <ModernPathInput
                                label="Default Runner Location"
                                description="Base directory for Wine/Proton versions."
                                value={`${settings.default_runner_path}`}
                                onChange={(val) => updateSetting("default_runner_path", val)}
                            />
                            {/*<ModernPathInput
                                label="Default DXVK Location"
                                description="Base directory for DXVK versions."
                                value={`${settings.default_dxvk_path}`}
                                onChange={(val) => updateSetting("default_dxvk_path", val)}
                            />*/}
                            <ModernPathInput
                                label="Default Prefix Location"
                                description="Base directory for Wine/Proton prefixes."
                                value={`${settings.default_runner_prefix_path}`}
                                onChange={(val) => updateSetting("default_prefix_path", val)}
                            />
                            <ModernPathInput
                                label="MangoHUD Config"
                                description="Default configuration file for MangoHUD."
                                value={`${settings.default_mangohud_config_path}`}
                                onChange={(val) => updateSetting("default_mangohud_config_path", val)}
                                folder={false}
                                extensions={["conf"]}
                            />
                        </SettingsSection>
                    )}
                    {/*activeTab === "integrations" && (
                        <SettingsSection title="Integrations & Tools">
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4"></div>
                        </SettingsSection>
                    )*/}
                    {activeTab === "about" && (
                        <div className="flex flex-col items-center justify-center h-full text-center p-8 relative overflow-hidden">
                            {/* Ambient background glow */}
                            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-pink-500/10 rounded-full blur-3xl pointer-events-none" />
                            <img src="/launcher-icon.png" className="w-32 h-32 mb-6 rounded-3xl animate-bounce-slow hover:scale-105 transition-transform duration-500" style={{ animationDuration: '3s' }} alt="Twintail Launcher"/>
                            <h1 className="text-4xl font-black bg-gradient-to-r from-white via-pink-200 to-violet-200 bg-clip-text text-transparent mb-2">
                                Twintail Launcher
                            </h1>
                            <div className="mb-8">
                                <span className="text-zinc-300">
                                    Version: <span className={"text-purple-400 font-bold"}>{version}</span> | Branch: <span className={"text-orange-400 font-bold"}>{branch}</span> | Commit: <span className={"text-cyan-400 font-bold"}>{__COMMIT_HASH__}</span>
                                 </span>
                            </div>
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-lg w-full">
                                <button onClick={() => invoke('open_uri', { uri: 'https://github.com/TwintailTeam/TwintailLauncher' })} className="flex items-center justify-center gap-2 p-3 bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/20 rounded-xl transition-all group cursor-pointer">
                                    <svg className="w-5 h-5 text-zinc-400 group-hover:text-white" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                                    </svg>
                                    <span className="text-zinc-300 group-hover:text-white font-medium">GitHub Repository</span>
                                </button>
                                <button onClick={() => invoke('open_uri', { uri: 'https://discord.gg/nDMJDwuj7s' })} className="flex items-center justify-center gap-2 p-3 bg-white/5 hover:bg-[#5865F2]/20 border border-white/5 hover:border-[#5865F2]/50 rounded-xl transition-all group cursor-pointer">
                                    <svg className="w-5 h-5 text-zinc-400 group-hover:text-[#5865F2]" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M20.317 4.492c-1.53-.69-3.17-1.2-4.885-1.49a.075.075 0 0 0-.079.036c-.21.369-.444.85-.608 1.23a18.566 18.566 0 0 0-5.487 0 12.36 12.36 0 0 0-.617-1.23A.077.077 0 0 0 8.562 3c-1.714.29-3.354.8-4.885 1.491a.07.07 0 0 0-.032.027C.533 9.093-.32 13.555.099 17.961a.08.08 0 0 0 .031.055 20.03 20.03 0 0 0 5.993 2.98.078.078 0 0 0 .084-.026 13.83 13.83 0 0 0 1.226-1.963.074.074 0 0 0-.041-.104 13.201 13.201 0 0 1-1.872-.878.075.075 0 0 1-.008-.125c.126-.093.252-.19.372-.287a.075.075 0 0 1 .078-.01c3.927 1.764 8.18 1.764 12.061 0a.075.075 0 0 1 .079.009c.12.098.246.195.373.288a.075.075 0 0 1-.006.125c-.598.344-1.22.635-1.873.877a.075.075 0 0 0-.041.105c.36.687.772 1.341 1.225 1.962a.077.077 0 0 0 .084.028 19.963 19.963 0 0 0 6.002-2.981.076.076 0 0 0 .032-.054c.5-5.094-.838-9.52-3.549-13.442a.06.06 0 0 0-.031-.028zM8.02 15.278c-1.182 0-2.157-1.069-2.157-2.38 0-1.312.956-2.38 2.157-2.38 1.201 0 2.176 1.068 2.157 2.38 0 1.311-.956 2.38-2.157 2.38zm7.975 0c-1.183 0-2.157-1.069-2.157-2.38 0-1.312.955-2.38 2.157-2.38 1.2 0 2.176 1.068 2.156 2.38 0 1.311-.956 2.38-2.156 2.38z" />
                                    </svg>
                                    <span className="text-zinc-300 group-hover:text-white font-medium">Join Discord</span>
                                </button>
                            </div>
                            <p className="mt-12 text-zinc-400 font-medium tracking-wide text-sm opacity-80 flex items-center justify-center">
                                Built with <span className={`text-purple-600 font-bold ml-1 mr-1`}><HeartIcon /></span> by the TwintailTeam
                            </p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
