import { useEffect, useRef, useState } from "react";
import ReactDOM from "react-dom";
import { MonitorIcon, SettingsIcon, Trash2Icon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

// Steam icon component - official Steam logo
const SteamIcon = ({ className }: { className?: string }) => (
    <svg className={className} viewBox="0 0 32 32" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
        <path d="M18.102 12.129c0-0 0-0 0-0.001 0-1.564 1.268-2.831 2.831-2.831s2.831 1.268 2.831 2.831c0 1.564-1.267 2.831-2.831 2.831-0 0-0 0-0.001 0h0c-0 0-0 0-0.001 0-1.563 0-2.83-1.267-2.83-2.83 0-0 0-0 0-0.001v0zM24.691 12.135c0-2.081-1.687-3.768-3.768-3.768s-3.768 1.687-3.768 3.768c0 2.081 1.687 3.768 3.768 3.768v0c2.080-0.003 3.765-1.688 3.768-3.767v-0zM10.427 23.76l-1.841-0.762c0.524 1.078 1.611 1.808 2.868 1.808 1.317 0 2.448-0.801 2.93-1.943l0.008-0.021c0.155-0.362 0.246-0.784 0.246-1.226 0-1.757-1.424-3.181-3.181-3.181-0.405 0-0.792 0.076-1.148 0.213l0.022-0.007 1.903 0.787c0.852 0.364 1.439 1.196 1.439 2.164 0 1.296-1.051 2.347-2.347 2.347-0.324 0-0.632-0.066-0.913-0.184l0.015 0.006zM15.974 1.004c-7.857 0.001-14.301 6.046-14.938 13.738l-0.004 0.054 8.038 3.322c0.668-0.462 1.495-0.737 2.387-0.737 0.001 0 0.002 0 0.002 0h-0c0.079 0 0.156 0.005 0.235 0.008l3.575-5.176v-0.074c0.003-3.12 2.533-5.648 5.653-5.648 3.122 0 5.653 2.531 5.653 5.653s-2.531 5.653-5.653 5.653h-0.131l-5.094 3.638c0 0.065 0.005 0.131 0.005 0.199 0 0.001 0 0.002 0 0.003 0 2.342-1.899 4.241-4.241 4.241-2.047 0-3.756-1.451-4.153-3.38l-0.005-0.027-5.755-2.383c1.841 6.345 7.601 10.905 14.425 10.905 8.281 0 14.994-6.713 14.994-14.994s-6.713-14.994-14.994-14.994c-0 0-0.001 0-0.001 0h0z"/>
    </svg>
);

interface InstallContextMenuProps {
    installId: string;
    installSettings: any;
    x: number;
    y: number;
    onClose: () => void;
    onOpenSettings: () => void;
    onRefreshSettings: () => void;
    fetchInstallData?: (id: string) => Promise<any>;
}

export default function InstallContextMenu({
    installId,
    installSettings: initialSettings,
    x,
    y,
    onClose,
    onOpenSettings,
    onRefreshSettings,
    fetchInstallData
}: InstallContextMenuProps) {
    const menuRef = useRef<HTMLDivElement>(null);
    const [position, setPosition] = useState({ x, y });
    const [installSettings, setInstallSettings] = useState(initialSettings);
    const [isLoading, setIsLoading] = useState(true);

    // Fetch fresh install settings when menu opens
    useEffect(() => {
        const fetchSettings = async () => {
            setIsLoading(true);
            let settings;
            try {
                if (fetchInstallData) {
                    settings = await fetchInstallData(installId);
                } else {
                    const data = await invoke("get_install_by_id", { id: installId });
                    if (data === null) {
                        console.error("Failed to fetch install settings!");
                        setInstallSettings(initialSettings);
                        return;
                    }
                    settings = JSON.parse(data as string);
                }
                setInstallSettings(settings);
            } catch (error) {
                console.error("Failed to fetch install settings:", error);
                setInstallSettings(initialSettings);
            } finally {
                setIsLoading(false);
            }
        };
        fetchSettings();
    }, [installId, initialSettings, fetchInstallData]);

    useEffect(() => {
        // Adjust position if menu would go off screen
        if (menuRef.current) {
            const rect = menuRef.current.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const viewportHeight = window.innerHeight;

            let adjustedX = x;
            let adjustedY = y;

            if (x + rect.width > viewportWidth) {
                adjustedX = viewportWidth - rect.width - 10;
            }

            if (y + rect.height > viewportHeight) {
                adjustedY = viewportHeight - rect.height - 10;
            }

            setPosition({ x: adjustedX, y: adjustedY });
        }
    }, [x, y]);

    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleEscape);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleEscape);
        };
    }, [onClose]);

    const MenuItem = ({ icon: Icon, label, onClick, variant = "default" }: {
        icon: any;
        label: string;
        onClick: () => void;
        variant?: "default" | "primary" | "danger";
    }) => {
        const variants = {
            default: "hover:bg-white/10 text-white/90",
            primary: "hover:bg-blue-600/20 text-blue-300",
            danger: "hover:bg-red-600/20 text-red-300"
        };

        return (
            <button
                className={`w-full flex items-center gap-3 px-4 py-2.5 transition-colors ${variants[variant]} text-sm font-medium`}
                onClick={() => {
                    onClick();
                    onClose();
                }}
            >
                <Icon className="w-4 h-4" />
                <span>{label}</span>
            </button>
        );
    };

    const Separator = () => (
        <div className="h-px bg-white/10 my-1" />
    );

    const handleOpenSettings = () => {
        onOpenSettings();
        onClose();
    };

    const handleAddToSteam = async () => {
        await invoke("add_shortcut", { installId, shortcutType: "steam" });
        onRefreshSettings();
    };

    const handleRemoveFromSteam = async () => {
        await invoke("remove_shortcut", { installId, shortcutType: "steam" });
        onRefreshSettings();
    };

    const handleCreateShortcut = async () => {
        await invoke("add_shortcut", { installId, shortcutType: "desktop" });
        onRefreshSettings();
    };

    const handleDeleteShortcut = async () => {
        await invoke("remove_shortcut", { installId, shortcutType: "desktop" });
        onRefreshSettings();
    };

    return ReactDOM.createPortal(
        <div
            ref={menuRef}
            className="fixed z-[9999] bg-zinc-900/95 backdrop-blur-md border border-white/20 rounded-lg shadow-2xl min-w-[200px] overflow-hidden animate-bg-fade-in"
            style={{
                left: `${position.x}px`,
                top: `${position.y}px`
            }}
        >
            <div className="py-2">
                <MenuItem
                    icon={SettingsIcon}
                    label="Installation Settings"
                    onClick={handleOpenSettings}
                    variant="default"
                />

                <Separator />

                {isLoading ? (
                    <div className="px-4 py-2.5 text-sm text-white/50">Loading...</div>
                ) : (
                    <>
                        {installSettings?.shortcut_is_steam ? (
                            <MenuItem
                                icon={Trash2Icon}
                                label="Remove from Steam"
                                onClick={handleRemoveFromSteam}
                                variant="danger"
                            />
                        ) : (
                            <MenuItem
                                icon={SteamIcon}
                                label="Add to Steam"
                                onClick={handleAddToSteam}
                                variant="primary"
                            />
                        )}

                        {installSettings?.shortcut_path !== "" ? (
                            <MenuItem
                                icon={Trash2Icon}
                                label="Delete Desktop Shortcut"
                                onClick={handleDeleteShortcut}
                                variant="danger"
                            />
                        ) : (
                            <MenuItem
                                icon={MonitorIcon}
                                label="Create Desktop Shortcut"
                                onClick={handleCreateShortcut}
                                variant="primary"
                            />
                        )}
                    </>
                )}
            </div>
        </div>,
        document.body
    );
}
