import { useState, useMemo } from "react";
import { POPUPS } from "./POPUPS.ts";
import { PAGES } from "../pages/PAGES.ts";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import {
    Folder,
    Play,
    Wrench,
    Trash2,
    AlertTriangle,
    Gauge,
    Sliders,
    Box,
    Monitor,
    Copy,
    Loader2,
    Check,
    X,
    FileCode2,
    LayoutDashboard,
    Terminal, Settings2, Logs
} from "lucide-react";
import { SettingsLayout } from "../layout/SettingsLayout.tsx";
import { SettingsSidebar, SettingsTab } from "../sidebar/SettingsSidebar.tsx";
import { SettingsSection, ModernToggle, ModernInput, ModernPathInput, ModernSelect } from "../common/SettingsComponents.tsx";
import { translate } from "../../utils/i18n";


// Helper for Steam Icon
const SteamIcon = ({ className }: { className?: string }) => (
    <svg className={className} viewBox="0 0 32 32" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
        <path d="M18.102 12.129c0-0 0-0 0-0.001 0-1.564 1.268-2.831 2.831-2.831s2.831 1.268 2.831 2.831c0 1.564-1.267 2.831-2.831 2.831-0 0-0 0-0.001 0h0c-0 0-0 0-0.001 0-1.563 0-2.83-1.267-2.83-2.83 0-0 0-0 0-0.001v0zM24.691 12.135c0-2.081-1.687-3.768-3.768-3.768s-3.768 1.687-3.768 3.768c0 2.081 1.687 3.768 3.768 3.768v0c2.080-0.003 3.765-1.688 3.768-3.767v-0zM10.427 23.76l-1.841-0.762c0.524 1.078 1.611 1.808 2.868 1.808 1.317 0 2.448-0.801 2.93-1.943l0.008-0.021c0.155-0.362 0.246-0.784 0.246-1.226 0-1.757-1.424-3.181-3.181-3.181-0.405 0-0.792 0.076-1.148 0.213l0.022-0.007 1.903 0.787c0.852 0.364 1.439 1.196 1.439 2.164 0 1.296-1.051 2.347-2.347 2.347-0.324 0-0.632-0.066-0.913-0.184l0.015 0.006zM15.974 1.004c-7.857 0.001-14.301 6.046-14.938 13.738l-0.004 0.054 8.038 3.322c0.668-0.462 1.495-0.737 2.387-0.737 0.001 0 0.002 0 0.002 0h-0c0.079 0 0.156 0.005 0.235 0.008l3.575-5.176v-0.074c0.003-3.12 2.533-5.648 5.653-5.648 3.122 0 5.653 2.531 5.653 5.653s-2.531 5.653-5.653 5.653h-0.131l-5.094 3.638c0 0.065 0.005 0.131 0.005 0.199 0 0.001 0 0.002 0 0.003 0 2.342-1.899 4.241-4.241 4.241-2.047 0-3.756-1.451-4.153-3.38l-0.005-0.027-5.755-2.383c1.841 6.345 7.601 10.905 14.425 10.905 8.281 0 14.994-6.713 14.994-14.994s-6.713-14.994-14.994-14.994c-0 0-0.001 0-0.001 0h0z" />
    </svg>
);

interface GameSettingsProps {
    setOpenPopup: (popup: POPUPS) => void;
    setCurrentPage: (page: PAGES) => void;
    setCurrentInstall: (id: string) => void;
    setCurrentGame: (biz: string) => void;
    setBackground: (background: string) => void;
    setDisplayName: (name: string) => void;
    setGameIcon: (icon: string) => void;
    pushInstalls: () => void;
    installSettings: any;
    gameManifest: any;
    fetchInstallSettings: (id: string) => void;
    prefetchedSwitches: any;
    prefetchedFps: any;
    prefetchedGraphicsApi: any;
    installedRunners: any[];
    installs?: any[];
    gamesinfo?: any[]; // Game manifests to look up static backgrounds
    imageVersion?: number; // Used to force image re-load after network recovery
}

export default function GameSettings({
    setOpenPopup,
    setCurrentPage,
    setCurrentInstall,
    setCurrentGame,
    setBackground,
    setDisplayName,
    setGameIcon,
    pushInstalls,
    installSettings,
    gameManifest,
    fetchInstallSettings,
    prefetchedSwitches,
    prefetchedFps,
    prefetchedGraphicsApi,
    installedRunners,
    installs,
    gamesinfo,
    imageVersion = 0
}: GameSettingsProps) {
    const [activeTab, setActiveTab] = useState("general");
    const [authkeyCopyState, setAuthkeyCopyState] = useState<"idle" | "copying" | "copied" | "failed">("idle");
    const [wipePrefixOnUninstall, setWipePrefixOnUninstall] = useState(false);
    const [showUninstallReview, setShowUninstallReview] = useState(false);
    const [uninstallAcknowledged, setUninstallAcknowledged] = useState(false);
    const [keepGameUninstall, setKeepGameUninstall] = useState(false);
    const [isUninstalling, setIsUninstalling] = useState(false);
    const isLinux = window.navigator.platform.includes("Linux");

    const tabs: SettingsTab[] = [
        { id: "general", label: translate("game_settings.tabs.general"), icon: Sliders, color: "blue" },
        { id: "launch", label: translate("game_settings.tabs.launch"), icon: Play, color: "emerald" },
        ...(prefetchedSwitches.xxmi ? [{ id: "xxmi", label: translate("game_settings.tabs.xxmi"), icon: Wrench, color: "pink" }] : []),
        ...(prefetchedSwitches.fps_unlocker ? [{ id: "fps_unlocker", label: translate("game_settings.tabs.fps_unlocker"), icon: Gauge, color: "yellow" }] : []),
        ...(isLinux ? [{ id: "linux", label: translate("game_settings.tabs.linux"), icon: Monitor, color: "orange" }] : []),
        { id: "manage", label: translate("game_settings.tabs.manage"), icon: Box, color: "purple" },
        { id: "uninstall", label: translate("game_settings.tabs.uninstall"), icon: AlertTriangle, color: "red" },
    ];

    // Generic update wrapper that matches backend command conventions
    // Backend commands use: update_install_{key}(app, id: String, {param}: {type})
    // Parameter names vary by command type - see install.rs for exact signatures
    const handleUpdate = async (key: string, value: any) => {
        try {
            const installId = installSettings.id;
            const command = `update_install_${key}`;

            // Build payload based on command type - backend uses 'id' not 'installId'
            let payload: Record<string, any> = { id: installId };

            if (typeof value === "boolean") {
                // Boolean commands use { id, enabled }
                payload.enabled = value;
            } else if (key.includes("path")) {
                // Path commands use { id, path }
                payload.path = value;
            } else if (key === "launch_args") {
                // update_install_launch_args uses { id, args }
                payload.args = value;
            } else if (key === "env_vars") {
                // update_install_env_vars uses { id, env_vars }
                payload.envVars = value;
            } else if (key === "pre_launch_cmd" || key === "launch_cmd") {
                // update_install_pre_launch_cmd and update_install_launch_cmd use { id, cmd }
                payload.cmd = value;
            } else if (key === "runner_version" || key === "dxvk_version") {
                // update_install_runner_version and update_install_dxvk_version use { id, version }
                payload.version = value;
            } else if (key === "fps_value") {
                // update_install_fps_value uses { id, fps }
                payload.fps = value;
            } else if (key === "graphics_api") {
                // update_install_graphics_api uses { id, api }
                payload.api = value;
            }

            await invoke(command, payload);

            // Use requestAnimationFrame to prevent flickering on Linux
            requestAnimationFrame(() => {
                fetchInstallSettings(installId);
                // Refresh the installs list so the play button's runner check
                // sees the updated runner_version immediately (not stale data)
                if (key === "runner_version" || key === "dxvk_version") {
                    pushInstalls();
                }
            });
        } catch (e) {
            console.error(`Failed to update game setting ${key}:`, e);
        }
    }

    // Find images - always use static backgrounds for settings popup
    // Memoize to prevent unnecessary re-renders on Linux
    const banner = useMemo(() => {
        const installInfo = (installs || []).find((i: any) => i.id === installSettings.id);
        const gameInfo = (gamesinfo || []).find((g: any) => g.manifest_id === installSettings.manifest_id) || (gamesinfo || []).find((g: any) => g.biz === installSettings.manifest_id);
        return installSettings.game_background || gameInfo?.assets?.game_background || gameInfo?.background || installInfo?.game_background;
    }, [installs, gamesinfo, installSettings.id, installSettings.manifest_id, installSettings.game_background]);

    const icon = useMemo(() => {
        const installInfo = (installs || []).find((i: any) => i.id === installSettings.id);
        return installInfo?.game_icon || installSettings.game_icon;
    }, [installs, installSettings.id, installSettings.game_icon]);

    const gameBiz = gameManifest?.biz || "";
    const xxmiConfig = installSettings.xxmi_config || {};
    const selectedFps = `${installSettings.fps_value ?? "60"}`;
    const fpsOptionsRaw = (Array.isArray(prefetchedFps) ? prefetchedFps : []).map((opt: any) => ({
        value: `${opt.value}`,
        name: `${opt.name ?? opt.value}`
    }));
    const fpsOptions = fpsOptionsRaw.some((opt: any) => opt.value === selectedFps)
        ? fpsOptionsRaw
        : [{ value: selectedFps, name: selectedFps }, ...fpsOptionsRaw];

    const handleUpdateXxmiConfig = async (payload: Record<string, any>) => {
        try {
            await invoke("update_install_xxmi_config", { id: installSettings.id, ...payload });
            requestAnimationFrame(() => {
                fetchInstallSettings(installSettings.id);
            });
        } catch (e) {
            console.error("Failed to update XXMI config:", e);
        }
    };

    const canUninstall = showUninstallReview && uninstallAcknowledged && !isUninstalling;
    const isAuthkeyCopying = authkeyCopyState === "copying";
    const isAuthkeyCopied = authkeyCopyState === "copied";
    const isAuthkeyFailed = authkeyCopyState === "failed";

    const handleInlineUninstall = async () => {
        if (!canUninstall) return;

        setIsUninstalling(true);
        try {
            const result = await invoke("remove_install", { id: installSettings.id, wipePrefix: wipePrefixOnUninstall, keepGameData: keepGameUninstall });
            if (!result) {
                console.error("Uninstall error!");
                return;
            }

            pushInstalls();

            const remainingInstalls = (installs || []).filter((i: any) => i.id !== installSettings.id);
            const games = gamesinfo || [];

            if (remainingInstalls.length > 0) {
                const next = remainingInstalls[0];
                const nextGame = games.find((g: any) => g.manifest_id === next.manifest_id);

                setCurrentInstall(next.id);
                setCurrentGame(nextGame ? nextGame.biz : (games.length > 0 ? games[0].biz : ""));
                setBackground(next.game_background);
                setDisplayName(next.name);
                setGameIcon(next.game_icon);

                requestAnimationFrame(() => {
                    const el = document.getElementById(next.id);
                    if (el) el.focus();
                });
            } else if (games.length > 0) {
                const firstGame = games[0];
                setCurrentInstall("");
                setCurrentGame(firstGame.biz || "");
                setBackground(firstGame.assets?.game_background || firstGame.background || "");
                setDisplayName(firstGame.display_name || "");
                setGameIcon(firstGame.assets?.game_icon || firstGame.icon || "");

                requestAnimationFrame(() => {
                    const targetId = firstGame.biz || firstGame.manifest_id;
                    const el = targetId ? document.getElementById(targetId) : null;
                    if (el) el.focus();
                });
            } else {
                setCurrentInstall("");
                setCurrentGame("");
                setBackground("");
                setDisplayName("");
                setGameIcon("");
            }

            setOpenPopup(POPUPS.NONE);
        } catch (e) {
            console.error("Failed to uninstall installation:", e);
        } finally {
            setIsUninstalling(false);
        }
    };

    const startUninstallReview = () => {
        setShowUninstallReview(true);
        setUninstallAcknowledged(false);
    };

    const cancelUninstallReview = () => {
        setShowUninstallReview(false);
        setUninstallAcknowledged(false);
    };

    return (
        <SettingsLayout
            title={installSettings.name || "Game Settings"}
            onClose={() => setOpenPopup(POPUPS.NONE)}
            banner={banner}
            icon={icon}
            imageVersion={imageVersion}>
            <div className="flex h-full">
                <SettingsSidebar tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} />

                <div className="flex-1 overflow-y-auto p-8 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent">
                    {activeTab === "general" && (
                        <SettingsSection title={translate("game_settings.general.title")}>
                            <ModernPathInput
                                label={translate("game_settings.general.install_location.label")}
                                description={translate("game_settings.general.install_location.description")}
                                value={`${installSettings.directory}`}
                                onChange={(val) => handleUpdate("game_path", val)}
                            />
                            <div className="grid grid-cols-1 gap-4 mt-4">
                                <ModernToggle
                                    label={translate("game_settings.general.skip_version_checks.label")}
                                    description={installSettings.steam_imported ? translate("game_settings.general.skip_version_checks.description_steam") : translate("game_settings.general.skip_version_checks.description")}
                                    descriptionClassName={installSettings.steam_imported ? "text-yellow-300 font-medium" : undefined}
                                    disabled={installSettings.steam_imported}
                                    checked={installSettings.steam_imported || installSettings.ignore_updates}
                                    onChange={(val) => {
                                        if (installSettings.steam_imported) return;
                                        handleUpdate("skip_version_updates", val);
                                    }}
                                />
                                <ModernToggle
                                    label={translate("game_settings.general.skip_hash_validation.label")}
                                    description={translate("game_settings.general.skip_hash_validation.description")}
                                    checked={installSettings.skip_hash_check}
                                    onChange={(val) => handleUpdate("skip_hash_valid", val)}
                                />
                                <ModernToggle
                                    label={translate("game_settings.general.discord_rpc.label")}
                                    description={translate("game_settings.general.discord_rpc.description")}
                                    checked={installSettings.show_discord_rpc}
                                    onChange={(val) => handleUpdate("show_drpc", val)}
                                />
                            </div>
                        </SettingsSection>
                    )}

                    {activeTab === "launch" && (
                        <SettingsSection title={translate("game_settings.launch.title")}>
                            <div className="flex flex-col gap-4">
                                <ModernToggle
                                    label={translate("game_settings.launch.prevent_idle.label")}
                                    description={translate("game_settings.launch.prevent_idle.description")}
                                    checked={installSettings.disable_system_idle}
                                    onChange={(val) => handleUpdate("disable_system_idle", val)}
                                />
                                {prefetchedSwitches.graphics_api && prefetchedGraphicsApi?.options?.length > 0 && (
                                    <ModernSelect
                                        label={translate("game_settings.launch.graphics_api.label")}
                                        description={translate("game_settings.launch.graphics_api.description")}
                                        value={installSettings.graphics_api || ""}
                                        options={prefetchedGraphicsApi.options}
                                        onChange={(val) => handleUpdate("graphics_api", val)}
                                    />
                                )}
                                <ModernInput
                                    label={translate("game_settings.launch.args.label")}
                                    description={translate("game_settings.launch.args.description")}
                                    value={installSettings.launch_args || ""}
                                    onChange={(e) => handleUpdate("launch_args", e.target.value)}
                                    placeholder="-dx11 -console"
                                />
                                <ModernInput
                                    label={translate("game_settings.launch.env_vars.label")}
                                    description={translate("game_settings.launch.env_vars.description")}
                                    value={installSettings.env_vars || ""}
                                    onChange={(e) => handleUpdate("env_vars", e.target.value)}
                                    placeholder='DXVK_HUD=fps,devinfo;PROTON_LOG=1;SOMETHING="/path/to/thing";'
                                />
                                <ModernInput
                                    label={translate("game_settings.launch.pre_launch_cmd.label")}
                                    placeholder={isLinux ? "/bin/bash -c echo hi" : "cmd.exe"}
                                    description={translate("game_settings.launch.pre_launch_cmd.description")}
                                    value={installSettings.pre_launch_command || ""}
                                    onChange={(e) => handleUpdate("pre_launch_cmd", e.target.value)}
                                    helpText={[
                                        translate("game_settings.launch.help.available_vars"),
                                        `- %steamrt% = ${translate("game_settings.launch.help.var.steamrt")}`,
                                        `- %reaper% = ${translate("game_settings.launch.help.var.reaper")}`,
                                        `- %appid% = ${translate("game_settings.launch.help.var.appid")}`,
                                        `- %runner% = ${translate("game_settings.launch.help.var.runner")}`,
                                        `- %game_exe% = ${translate("game_settings.launch.help.var.game_exe")}`,
                                        `- %runner_dir% = ${translate("game_settings.launch.help.var.runner_dir")}`,
                                        `- %prefix% = ${translate("game_settings.launch.help.var.prefix")}`,
                                        `- %install_dir% = ${translate("game_settings.launch.help.var.install_dir")}`,
                                        `- %steamrt_path% = ${translate("game_settings.launch.help.var.steamrt_path")}`,
                                    ].join("\n")}
                                />
                                <ModernInput
                                    label={translate("game_settings.launch.custom_launch_cmd.label")}
                                    placeholder={isLinux ? "%steamrt% --verb=waitforexitandrun -- %reaper% SteamLaunch AppId=%appid% -- %runner% waitforexitandrun %game_exe%" : "Start-Process -FilePath '%game_exe%' -WorkingDirectory '%install_dir%' -Verb RunAs"}
                                    description={translate("game_settings.launch.custom_launch_cmd.description")}
                                    value={installSettings.launch_command || ""}
                                    onChange={(e) => handleUpdate("launch_cmd", e.target.value)}
                                    helpText={[
                                        translate("game_settings.launch.help.available_vars"),
                                        `- %steamrt% = ${translate("game_settings.launch.help.var.steamrt")}`,
                                        `- %reaper% = ${translate("game_settings.launch.help.var.reaper")}`,
                                        `- %appid% = ${translate("game_settings.launch.help.var.appid")}`,
                                        `- %runner% = ${translate("game_settings.launch.help.var.runner")}`,
                                        `- %game_exe% = ${translate("game_settings.launch.help.var.game_exe")}`,
                                        `- %runner_dir% = ${translate("game_settings.launch.help.var.runner_dir")}`,
                                        `- %prefix% = ${translate("game_settings.launch.help.var.prefix")}`,
                                        `- %install_dir% = ${translate("game_settings.launch.help.var.install_dir")}`,
                                        `- %steamrt_path% = ${translate("game_settings.launch.help.var.steamrt_path")}`,
                                        `- %command% = ${translate("game_settings.launch.help.var.command")}`,
                                    ].join("\n")}
                                />
                            </div>
                        </SettingsSection>
                    )}

                    {activeTab === "linux" && (
                        <>
                        <SettingsSection title={translate("game_settings.linux.title")}>
                            <div className="flex flex-col gap-4">
                                <div className="flex flex-col gap-2">
                                    <ModernSelect
                                        label={translate("game_settings.linux.runner_version.label")}
                                        description={translate("game_settings.linux.runner_version.description")}
                                        value={installSettings.runner_version || ""}
                                        options={installedRunners}
                                        onChange={(val) => handleUpdate("runner_version", val)}
                                    />
                                    <button
                                        onClick={() => {
                                            setOpenPopup(POPUPS.NONE);
                                            setCurrentPage(PAGES.RUNNERS);
                                        }}
                                        className="text-purple-400 hover:text-purple-300 text-sm font-medium transition-colors text-left px-1 underline-offset-2 hover:underline">
                                        → {translate("game_settings.linux.manage_runners")}
                                    </button>
                                </div>
                                <ModernPathInput
                                    label={translate("game_settings.linux.runner_location.label")}
                                    description={translate("game_settings.linux.runner_location.description")}
                                    value={`${installSettings.runner_path}`}
                                    onChange={(val) => handleUpdate("runner_path", val)}
                                />
                                <ModernPathInput
                                    label={translate("game_settings.linux.prefix_location.label")}
                                    description={translate("game_settings.linux.prefix_location.description")}
                                    value={`${installSettings.runner_prefix}`}
                                    onChange={(val) => handleUpdate("prefix_path", val)}
                                />
                                {prefetchedSwitches.jadeite && isLinux && (
                                    <ModernToggle
                                        label={translate("game_settings.linux.jadeite.label")}
                                        description={translate("game_settings.linux.jadeite.description")}
                                        checked={installSettings.use_jadeite}
                                        onChange={(val) => handleUpdate("use_jadeite", val)}
                                    />
                                )}
                                <ModernToggle
                                    label={translate("game_settings.linux.gamemode.label")}
                                    description={translate("game_settings.linux.gamemode.description")}
                                    checked={installSettings.use_gamemode}
                                    onChange={(val) => handleUpdate("use_gamemode", val)}
                                />
                                <ModernToggle
                                    label={translate("game_settings.linux.mangohud.label")}
                                    description={translate("game_settings.linux.mangohud.description")}
                                    checked={!!installSettings.use_mangohud}
                                    onChange={(val) => handleUpdate("use_mangohud", val)}
                                />
                                <ModernPathInput
                                    label={translate("game_settings.linux.mangohud_config.label")}
                                    description={translate("game_settings.linux.mangohud_config.description")}
                                    value={`${installSettings.mangohud_config_path ?? ""}`}
                                    folder={false}
                                    extensions={["conf"]}
                                    onChange={(val) => handleUpdate("mangohud_config_path", val)}
                                />
                            </div>
                        </SettingsSection>
                        </>
                    )}

                    {activeTab === "xxmi" && (
                        <SettingsSection title={translate("game_settings.xxmi.title")}>
                            <div className="flex flex-col gap-4">
                                <ModernToggle
                                    label={translate("game_settings.xxmi.enable.label")}
                                    description={translate("game_settings.xxmi.enable.description")}
                                    checked={!!installSettings.use_xxmi}
                                    onChange={(val) => handleUpdate("use_xxmi", val)}
                                />
                                <ModernToggle
                                    label={translate("game_settings.xxmi.show_warnings.label")}
                                    description={translate("game_settings.xxmi.show_warnings.description")}
                                    checked={!!xxmiConfig.show_warnings}
                                    onChange={(val) => handleUpdateXxmiConfig({ xxmiSw: val })}
                                />
                                <ModernToggle
                                    label={translate("game_settings.xxmi.dump_shaders.label")}
                                    description={translate("game_settings.xxmi.dump_shaders.description")}
                                    checked={!!xxmiConfig.dump_shaders}
                                    onChange={(val) => handleUpdateXxmiConfig({ xxmiSd: val })}
                                />
                                <ModernSelect
                                    label={translate("game_settings.xxmi.hunting_mode.label")}
                                    description={translate("game_settings.xxmi.hunting_mode.description")}
                                    value={`${xxmiConfig.hunting_mode ?? 0}`}
                                    options={[
                                        { value: "0", name: translate("game_settings.xxmi.hunting_mode.disabled") },
                                        { value: "1", name: translate("game_settings.xxmi.hunting_mode.always_enabled") },
                                        { value: "2", name: translate("game_settings.xxmi.hunting_mode.soft_disabled") }
                                    ]}
                                    onChange={(val) => handleUpdateXxmiConfig({ xxmiHunting: Number(val) })}
                                />
                            </div>
                        </SettingsSection>
                    )}

                    {activeTab === "fps_unlocker" && (
                        <SettingsSection title={translate("game_settings.fps_unlocker.title")}>
                            <div className="flex flex-col gap-4">
                                {prefetchedSwitches.fps_unlocker ? (
                                    <>
                                        <ModernToggle
                                            label={translate("game_settings.fps_unlocker.enable.label")}
                                            description={translate("game_settings.fps_unlocker.enable.description")}
                                            checked={!!installSettings.use_fps_unlock}
                                            onChange={(val) => handleUpdate("use_fps_unlock", val)}
                                        />
                                        <ModernSelect
                                            label={translate("game_settings.fps_unlocker.target.label")}
                                            description={translate("game_settings.fps_unlocker.target.description")}
                                            value={selectedFps}
                                            options={fpsOptions}
                                            onChange={(val) => handleUpdate("fps_value", val)}
                                        />
                                    </>
                                ) : (
                                    <div className="rounded-xl border border-white/10 bg-zinc-900/70 p-4 text-sm text-zinc-300">
                                        {translate("game_settings.fps_unlocker.not_available")}
                                    </div>
                                )}
                            </div>
                        </SettingsSection>
                    )}

                    {activeTab === "manage" && (
                        <>
                            <SettingsSection title={translate("game_settings.manage.title")}>
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <button
                                        onClick={() => {
                                            setOpenPopup(POPUPS.NONE);
                                            invoke("open_folder", {
                                                runnerVersion: "",
                                                manifestId: installSettings.manifest_id,
                                                installId: installSettings.id,
                                                pathType: "install"
                                            });
                                        }}
                                        className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                        <Folder className="w-6 h-6 text-purple-400" />
                                        <div className="flex flex-col">
                                            <span className="font-bold">{translate("game_settings.manage.open_game_folder")}</span>
                                            <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_game_folder.description")}</span>
                                        </div>
                                    </button>

                                    {installSettings.use_xxmi && (
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_folder", {
                                                    runnerVersion: "",
                                                    manifestId: installSettings.manifest_id,
                                                    installId: installSettings.id,
                                                    pathType: "mods"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Folder className="w-6 h-6 text-pink-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_mods_folder")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_mods_folder.description")}</span>
                                            </div>
                                        </button>
                                    )}
                                    <button
                                        onClick={() => {
                                            if (installSettings.steam_imported) return;
                                            setOpenPopup(POPUPS.NONE);
                                            emit("start_game_repair", {
                                                install: installSettings.id,
                                                biz: installSettings.manifest_id,
                                                lang: "en-us",
                                                region: installSettings.region_code
                                            });
                                        }}
                                        disabled={installSettings.steam_imported}
                                        className={`flex items-center gap-3 p-4 rounded-xl border border-white/5 transition-colors text-white text-left ${installSettings.steam_imported ? "cursor-not-allowed bg-zinc-900/85 opacity-70" : "bg-zinc-900/85 hover:bg-zinc-900 hover:border-white/10"}`}>
                                        <Wrench className={`w-6 h-6 ${installSettings.steam_imported ? "text-zinc-500" : "text-orange-400"}`}/>
                                        <div className="flex flex-col">
                                            <span className="font-bold">{translate("game_settings.manage.repair_game")}</span>
                                            <span className={`text-xs ${installSettings.steam_imported ? "text-yellow-300 font-medium" : "text-zinc-400"}`}>
                                                {installSettings.steam_imported ? translate("game_settings.manage.steam_managed") : translate("game_settings.manage.repair_game.description")}
                                            </span>
                                        </div>
                                    </button>

                                    {installSettings.shortcut_is_steam ? (
                                        <button
                                            onClick={() => {
                                                if (installSettings.steam_imported) return;
                                                invoke("remove_shortcut", { installId: installSettings.id, shortcutType: "steam" }).then(() => fetchInstallSettings(installSettings.id));
                                            }}
                                            disabled={installSettings.steam_imported}
                                            className={`flex items-center gap-3 p-4 rounded-xl border border-white/5 transition-colors text-white text-left ${installSettings.steam_imported ? "cursor-not-allowed bg-zinc-900/85 opacity-70" : "bg-zinc-900/85 hover:bg-zinc-900 hover:border-white/10"}`}>
                                            <Trash2 className={`w-6 h-6 ${installSettings.steam_imported ? "text-zinc-500" : "text-blue-400"}`}/>
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.remove_from_steam")}</span>
                                                <span className={`text-xs ${installSettings.steam_imported ? "text-yellow-300 font-medium" : "text-zinc-400"}`}>
                                                    {installSettings.steam_imported ? translate("game_settings.manage.steam_managed") : translate("game_settings.manage.shortcut_delete")}
                                                </span>
                                            </div>
                                        </button>
                                    ) : (
                                        <button
                                            onClick={() => {
                                                if (installSettings.steam_imported) return;
                                                invoke("add_shortcut", { installId: installSettings.id, shortcutType: "steam" }).then(() => fetchInstallSettings(installSettings.id));
                                            }}
                                            disabled={installSettings.steam_imported}
                                            className={`flex items-center gap-3 p-4 rounded-xl border border-white/5 transition-colors text-white text-left ${installSettings.steam_imported ? "cursor-not-allowed bg-zinc-900/85 opacity-70" : "bg-zinc-900/85 hover:bg-zinc-900 hover:border-white/10"}`}>
                                            <SteamIcon className={`w-6 h-6 ${installSettings.steam_imported ? "text-zinc-500" : "text-blue-400"}`}/>
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.add_to_steam")}</span>
                                                <span className={`text-xs ${installSettings.steam_imported ? "text-yellow-300 font-medium" : "text-zinc-400"}`}>
                                                    {installSettings.steam_imported ? translate("game_settings.manage.steam_managed") : translate("game_settings.manage.shortcut_create")}
                                                </span>
                                            </div>
                                        </button>
                                    )}

                                    {installSettings.shortcut_path !== "" ? (
                                        <button
                                            onClick={() => {
                                                invoke("remove_shortcut", { installId: installSettings.id, shortcutType: "desktop" }).then(() => fetchInstallSettings(installSettings.id));
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Trash2 className="w-6 h-6 text-blue-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.remove_from_desktop")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.shortcut_delete")}</span>
                                            </div>
                                        </button>
                                    ) : (
                                        <button
                                            onClick={() => {
                                                invoke("add_shortcut", { installId: installSettings.id, shortcutType: "desktop" }).then(() => fetchInstallSettings(installSettings.id));
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Monitor className="w-6 h-6 text-blue-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.add_to_desktop")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.shortcut_create")}</span>
                                            </div>
                                        </button>
                                    )}
                                    {gameBiz && (gameBiz.startsWith("hk4e") || gameBiz.startsWith("hkrpg") || gameBiz.startsWith("nap") || gameBiz.startsWith("bh3") || gameBiz.startsWith("abc") || gameBiz.startsWith("hyg")) && (
                                        <button
                                            onClick={async () => {
                                                if (isAuthkeyCopying) { return; }
                                                setAuthkeyCopyState("copying");
                                                try {
                                                    const copied = await invoke<boolean>("copy_authkey", { id: installSettings.id });
                                                    if (copied) {
                                                        setAuthkeyCopyState("copied");
                                                        setTimeout(() => setAuthkeyCopyState("idle"), 2400);
                                                    } else {
                                                        setAuthkeyCopyState("failed");
                                                        setTimeout(() => setAuthkeyCopyState("idle"), 2600);
                                                    }
                                                } catch (e) {
                                                    console.error("Failed to copy authkey:", e);
                                                    setAuthkeyCopyState("failed");
                                                    setTimeout(() => setAuthkeyCopyState("idle"), 2600);
                                                }
                                            }}
                                            disabled={isAuthkeyCopying}
                                            className={`flex items-center gap-3 p-4 rounded-xl border transition-all text-white text-left ${isAuthkeyCopying ? "bg-purple-900/25 border-purple-400/30 cursor-wait" : ""} ${isAuthkeyCopied ? "bg-emerald-900/25 border-emerald-400/40" : ""} ${isAuthkeyFailed ? "bg-red-900/25 border-red-400/35" : ""} ${!isAuthkeyCopying && !isAuthkeyCopied && !isAuthkeyFailed ? "bg-zinc-800/50 hover:bg-zinc-700/50 border-white/5 hover:border-white/20" : ""}`}>
                                            {isAuthkeyCopying && <Loader2 className="w-6 h-6 text-purple-300 animate-spin" />}
                                            {isAuthkeyCopied && <Check className="w-6 h-6 text-emerald-300" />}
                                            {isAuthkeyFailed && <X className="w-6 h-6 text-red-300" />}
                                            {!isAuthkeyCopying && !isAuthkeyCopied && !isAuthkeyFailed && <Copy className="w-6 h-6 text-purple-400" />}
                                            <div className="flex flex-col">
                                                <span className="font-bold">{isAuthkeyCopying ? translate("game_settings.manage.copy_authkey.copying") : isAuthkeyCopied ? translate("game_settings.manage.copy_authkey.copied") : isAuthkeyFailed ? translate("game_settings.manage.copy_authkey.failed") : translate("game_settings.manage.copy_authkey")}</span>
                                                <span className={`text-xs ${isAuthkeyCopied ? "text-emerald-300" : isAuthkeyFailed ? "text-red-300" : "text-zinc-400"}`}>{isAuthkeyCopying ? translate("game_settings.manage.copy_authkey.copying_description") : isAuthkeyCopied ? translate("game_settings.manage.copy_authkey.copied_description") : isAuthkeyFailed ? translate("game_settings.manage.copy_authkey.failed_description") : <>{translate("game_settings.manage.copy_authkey.description")} <span className="text-purple-400">aivo.minlor.net/hoyo</span></>}</span>
                                            </div>
                                        </button>
                                    )}

                                    {gameBiz && (gameBiz.startsWith("hk4e") || gameBiz.startsWith("hkrpg") || gameBiz.startsWith("nap") || gameBiz.startsWith("bh3") || gameBiz.startsWith("abc") || gameBiz.startsWith("hyg") || gameBiz.startsWith("endfield") || gameBiz.startsWith("pgr") || gameBiz.startsWith("wuwa")) && (
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_folder", {
                                                    runnerVersion: "",
                                                    manifestId: installSettings.manifest_id,
                                                    installId: installSettings.id,
                                                    pathType: "engine_log"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Logs className="w-6 h-6 text-purple-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_engine_log")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_engine_log.description")}</span>
                                            </div>
                                        </button>
                                    )}
                                </div>
                            </SettingsSection>
                            {window.navigator.platform.includes("Linux") && (
                                <SettingsSection title={translate("game_settings.manage.runner_title")}>
                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_folder", {
                                                    runnerVersion: "",
                                                    manifestId: installSettings.manifest_id,
                                                    installId: installSettings.id,
                                                    pathType: "runner"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Folder className="w-6 h-6 text-orange-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_runner_folder")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_runner_folder.description")}</span>
                                            </div>
                                        </button>
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_folder", {
                                                    runnerVersion: "",
                                                    manifestId: installSettings.manifest_id,
                                                    installId: installSettings.id,
                                                    pathType: "runner_prefix"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Folder className="w-6 h-6 text-yellow-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_prefix_folder")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_prefix_folder.description")}</span>
                                            </div>
                                        </button>
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("empty_folder", {
                                                    installId: installSettings.id,
                                                    pathType: "runner_prefix"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Wrench className="w-6 h-6 text-orange-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.repair_prefix")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.repair_prefix.description")}</span>
                                            </div>
                                        </button>
                                            <button
                                                onClick={() => {
                                                    setOpenPopup(POPUPS.NONE);
                                                    invoke("open_in_prefix", {
                                                        installId: installSettings.id,
                                                        pathType: "regedit.exe"
                                                    });
                                                }}
                                                className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                                <FileCode2 className="w-6 h-6 text-purple-400" />
                                                <div className="flex flex-col">
                                                    <span className="font-bold">{translate("game_settings.manage.open_registry")}</span>
                                                    <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_registry.description")}</span>
                                                </div>
                                            </button>
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_in_prefix", {
                                                    installId: installSettings.id,
                                                    pathType: "control.exe"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <LayoutDashboard className="w-6 h-6 text-purple-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_control_panel")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_control_panel.description")}</span>
                                            </div>
                                        </button>
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_in_prefix", {
                                                    installId: installSettings.id,
                                                    pathType: "cmd.exe"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Terminal className="w-6 h-6 text-purple-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_cmd")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_cmd.description")}</span>
                                            </div>
                                        </button>
                                        <button
                                            onClick={() => {
                                                setOpenPopup(POPUPS.NONE);
                                                invoke("open_in_prefix", {
                                                    installId: installSettings.id,
                                                    pathType: "winecfg.exe"
                                                });
                                            }}
                                            className="flex items-center gap-3 p-4 bg-zinc-800/50 hover:bg-zinc-700/50 rounded-xl border border-white/5 transition-all hover:border-white/20 text-white text-left">
                                            <Settings2 className="w-6 h-6 text-purple-400" />
                                            <div className="flex flex-col">
                                                <span className="font-bold">{translate("game_settings.manage.open_winecfg")}</span>
                                                <span className="text-xs text-zinc-400">{translate("game_settings.manage.open_winecfg.description")}</span>
                                            </div>
                                        </button>
                                    </div>
                                </SettingsSection>
                            )}
                        </>
                        )}

                    {activeTab === "uninstall" && (
                        <SettingsSection title={translate("game_settings.uninstall.title")}>
                            <div className="rounded-xl border border-red-500/30 bg-red-950/20 p-5 flex flex-col gap-4">
                                <div className="flex flex-col gap-1">
                                    <span className="text-red-300 font-semibold">{translate("game_settings.uninstall.name")}</span>
                                    <span className="text-sm text-red-200/70">
                                        {translate("game_settings.uninstall.description")}
                                    </span>
                                </div>

                                {!showUninstallReview ? (
                                    <button
                                        onClick={startUninstallReview}
                                        className="flex items-center justify-center gap-2 py-3 px-4 rounded-lg font-semibold bg-red-900/40 hover:bg-red-800/50 border border-red-500/40 text-red-100 transition-colors">
                                        <Trash2 className="w-5 h-5" />
                                        <span>{translate("game_settings.uninstall.review_button")}</span>
                                    </button>
                                ) : (
                                    <div className="flex flex-col gap-4">
                                        <div className="rounded-lg border border-red-500/30 bg-black/30 p-4 text-sm text-red-100/90">
                                            <p>{translate("game_settings.uninstall.reviewing_header", { install_name: installSettings.name })}</p>
                                            <p className="mt-2">{translate("game_settings.uninstall.will_delete")}</p>
                                            <p className="text-red-200/80">{translate("game_settings.uninstall.delete_files")}</p>
                                            <p className="text-red-200/80">{translate("game_settings.uninstall.delete_settings")}</p>
                                            {isLinux && wipePrefixOnUninstall && (<p className="text-red-200/80">{translate("game_settings.uninstall.delete_prefix")}</p>)}
                                            <p className="mt-2">{translate("game_settings.uninstall.wont_delete")}</p>
                                            <p className="text-red-200/80">{translate("game_settings.uninstall.keep_runners")}</p>
                                            {isLinux && !wipePrefixOnUninstall && (<p className="text-red-200/80">{translate("game_settings.uninstall.keep_prefix")}</p>)}
                                        </div>

                                        {isLinux && (
                                            <ModernToggle
                                                label={translate("game_settings.uninstall.delete_prefix_toggle.label")}
                                                description={translate("game_settings.uninstall.delete_prefix_toggle.description")}
                                                checked={wipePrefixOnUninstall}
                                                onChange={setWipePrefixOnUninstall}
                                            />
                                        )}
                                        <ModernToggle
                                            label={translate("game_settings.uninstall.keep_game_data.label")}
                                            description={translate("game_settings.uninstall.keep_game_data.description")}
                                            checked={keepGameUninstall}
                                            onChange={setKeepGameUninstall}
                                        />
                                        <ModernToggle
                                            label={translate("game_settings.uninstall.acknowledge.label")}
                                            description={translate("game_settings.uninstall.acknowledge.description")}
                                            checked={uninstallAcknowledged}
                                            onChange={setUninstallAcknowledged}
                                        />
                                        <div className="flex gap-3">
                                            <button
                                                onClick={cancelUninstallReview}
                                                className="flex-1 py-3 px-4 rounded-lg font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors">
                                                {translate("game_settings.uninstall.cancel")}
                                            </button>
                                            <button
                                                onClick={handleInlineUninstall}
                                                disabled={!canUninstall}
                                                className={`flex-1 flex items-center justify-center gap-2 py-3 px-4 rounded-lg font-semibold transition-colors ${canUninstall ? "bg-red-600 hover:bg-red-500 text-white" : "bg-zinc-800 text-zinc-500 cursor-not-allowed"}`}>
                                                <Trash2 className="w-5 h-5" />
                                                <span>{isUninstalling ? translate("game_settings.uninstall.in_progress") : translate("game_settings.uninstall.confirm")}</span>
                                            </button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </SettingsSection>
                    )}
                </div>
            </div>
        </SettingsLayout>
    );
}
