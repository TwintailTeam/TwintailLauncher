import { CachedImage } from "../common/CachedImage";

interface GameInfoOverlayProps {
    displayName: string;
    gameIcon: string;
    version?: string;
    hasUpdate?: boolean;
    isVisible: boolean;
    imageVersion?: number; // Used to force image re-load after network recovery
}

export default function GameInfoOverlay({
    displayName,
    gameIcon,
    version,
    hasUpdate,
    isVisible,
    imageVersion = 0
}: GameInfoOverlayProps) {
    if (!isVisible || !displayName) return null;

    return (
        <div className="absolute bottom-8 left-24 max-w-md animate-slideUp z-10" style={{ animationDelay: '200ms' }}>
            <div className="bg-black/50 rounded-xl border border-white/10 p-4 shadow-lg">
                <div className="flex items-center gap-3">
                    {gameIcon && (
                        <div className="w-12 h-12 rounded-lg overflow-hidden border border-white/10 flex-shrink-0">
                            <CachedImage key={`icon-v${imageVersion}`} src={gameIcon} className="w-full h-full object-cover" alt="Game Icon" />
                        </div>
                    )}
                    <div className="min-w-0">
                        <h1 className="text-lg font-semibold text-white truncate">{displayName}</h1>
                        <div className="flex items-center gap-2 mt-0.5">
                            {version && (
                                <span className="text-sm text-white/50">v{version}</span>
                            )}
                            {hasUpdate && (
                                <span className="px-2 py-0.5 rounded-md bg-purple-500/15 text-purple-400 text-xs font-medium border border-purple-500/30 shadow-[0_0_8px_rgba(147,51,234,0.2)]">
                                    Update Available
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
