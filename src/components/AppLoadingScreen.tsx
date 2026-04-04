import React, { useEffect, useState } from "react";

interface AppLoadingScreenProps {
    progress: number;
    message: string;
    // When true, the overlay will fade out (opacity 0) before unmounting
    fadingOut?: boolean;
    // Callback when user clicks skip during preload phase
    onSkip?: () => void;
}

// Time in ms before showing skip button during preload phase
const SKIP_BUTTON_DELAY_MS = 10000;

const AppLoadingScreen: React.FC<AppLoadingScreenProps> = ({ progress, message, fadingOut, onSkip }) => {
    const [showSkipButton, setShowSkipButton] = useState(false);
    const [preloadStartTime, setPreloadStartTime] = useState<number | null>(null);

    // Track when preload phase starts (progress >= 75)
    const isPreloading = progress >= 75 && progress < 100;

    useEffect(() => {
        if (isPreloading && preloadStartTime === null) {
            setPreloadStartTime(Date.now());
        } else if (!isPreloading) {
            setPreloadStartTime(null);
            setShowSkipButton(false);
        }
    }, [isPreloading, preloadStartTime]);

    // Show skip button after delay during preload phase
    useEffect(() => {
        if (!isPreloading || preloadStartTime === null) return;

        const timer = setTimeout(() => {
            setShowSkipButton(true);
        }, SKIP_BUTTON_DELAY_MS);

        return () => clearTimeout(timer);
    }, [isPreloading, preloadStartTime]);

    const isPreloadPhase = progress >= 75;

    return (
        <main
            className={`fixed inset-0 z-50 w-full h-screen flex flex-col items-center justify-center transition-opacity duration-500 ease-in-out ${fadingOut ? 'opacity-0 pointer-events-none' : 'opacity-100'}`}
            style={{
                backgroundImage: 'radial-gradient(circle at 20% 20%, rgba(90,70,140,0.35), rgba(20,15,30,0.9) 60%), radial-gradient(circle at 80% 80%, rgba(60,100,160,0.25), rgba(10,10,20,0.95) 55%)'
            }}
        >
            <div className="relative z-10 flex flex-col items-center max-w-md w-full px-12 animate-fadeIn">
                {/* Logo & Title Section */}
                <div className="flex flex-col items-center mb-14 space-y-6">
                    <div className="relative w-24 h-24 mb-2">
                        {/* Glow behind logo */}
                        <div className="absolute inset-0 bg-blue-500/20 blur-2xl rounded-full" />
                        <img
                            src="/launcher-icon.png"
                            srcSet="/launcher-icon.png 1x, /launcher-icon-128.png 2x"
                            alt="TwintailLauncher"
                            className="relative w-full h-full object-contain drop-shadow-2xl animate-[pulse_3s_ease-in-out_infinite]"
                            onError={(e) => {
                                e.currentTarget.style.display = 'none';
                            }}
                        />
                    </div>

                    <div className="text-center space-y-1">
                        <h1 className="text-3xl font-bold bg-gradient-to-br from-white via-white to-white/60 bg-clip-text text-transparent tracking-tight">
                            TwintailLauncher
                        </h1>
                    </div>
                </div>

                {/* Progress Section */}
                <div className="w-full space-y-4">
                    {/* Progress Bar Container */}
                    <div className="relative h-1.5 w-full bg-white/[0.06] rounded-full overflow-hidden ring-1 ring-white/[0.05]">
                        {/* Actual Bar */}
                        <div
                            className="absolute top-0 left-0 h-full bg-gradient-to-r from-blue-600 via-blue-400 to-blue-500 rounded-full transition-all duration-300 ease-out shadow-[0_0_12px_rgba(59,130,246,0.6)]"
                            style={{ width: `${Math.max(2, progress)}%` }}
                        >
                            {/* Inner highlight for gloss effect */}
                            <div className="absolute top-0 right-0 bottom-0 w-20 bg-gradient-to-l from-white/30 to-transparent" />
                        </div>
                    </div>

                    {/* Status Text & Percentage */}
                    <div className="flex justify-between items-center text-xs">
                        <span className="text-white/40 uppercase tracking-widest font-medium truncate max-w-[240px]">
                            {message}
                        </span>
                        <span className="text-blue-400 font-mono font-medium">
                            {Math.round(progress)}%
                        </span>
                    </div>
                </div>
            </div>

            {/* Footer / Copyright */}
            <div className="absolute bottom-10 flex flex-col items-center gap-3">
                {/* Skip button - shown after delay during preload phase */}
                {showSkipButton && onSkip && (
                    <button
                        onClick={onSkip}
                        className="px-4 py-1.5 text-xs font-medium text-white/70 hover:text-white bg-white/[0.08] hover:bg-white/[0.15] rounded-full border border-white/[0.1] hover:border-white/[0.2] transition-all duration-200 animate-fadeIn"
                    >
                        Skip &amp; Continue
                    </button>
                )}
                <div className="text-white/[0.15] text-[10px] uppercase tracking-[0.2em] font-medium">
                    {isPreloadPhase ? "Preloading Assets" : "Initializing Environment"}
                </div>
            </div>
        </main>
    );
};

export default AppLoadingScreen;
