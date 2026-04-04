import React, { useState, useEffect, useRef } from "react";
import { isVideoUrl, isLinux } from "../../utils/imagePreloader";

interface BackgroundOption {
    src: string;
    label: string;
    isDynamic: boolean;
}

interface BackgroundControlsProps {
    currentBackground: string;
    availableBackgrounds: BackgroundOption[];
    onBackgroundChange: (src: string) => void;
    isVisible: boolean;
}

const BackgroundControls: React.FC<BackgroundControlsProps> = ({
    currentBackground,
    availableBackgrounds,
    onBackgroundChange,
    isVisible,
}) => {
    const [isPaused, setIsPaused] = useState(false);
    const [currentIndex, setCurrentIndex] = useState(0);
    const [isBackgroundHovered, setIsBackgroundHovered] = useState(false);
    const hoveredStateRef = useRef(false);
    const currentBg = availableBackgrounds.find(bg => bg.src === currentBackground) || availableBackgrounds[currentIndex];
    const isDynamicBackground = currentBg?.isDynamic && isVideoUrl(currentBg.src);

    // Update current index when background changes externally and reset pause state
    useEffect(() => {
        const index = availableBackgrounds.findIndex((bg) => bg.src === currentBackground);
        if (index !== -1) {
            setCurrentIndex(index);
        }
        setIsPaused(false);
    }, [currentBackground, availableBackgrounds]);

    // Handle pause/play for dynamic backgrounds, including newly swapped video nodes.
    useEffect(() => {
        if (!isVisible) return;
        let cancelled = false;
        let retries = 0;

        const applyPlaybackState = () => {
            if (cancelled) return;
            const videoElement = document.getElementById("app-bg");
            if (videoElement && videoElement instanceof HTMLVideoElement) {
                if (isPaused) {
                    videoElement.pause();
                } else {
                    videoElement.play().catch(() => { });
                }
                return;
            }
            if (isDynamicBackground && retries < 30) {
                retries++;
                requestAnimationFrame(applyPlaybackState);
            }
        };

        applyPlaybackState();
        return () => { cancelled = true; };
    }, [isPaused, currentBackground, isDynamicBackground, isVisible]);

    // Listen for mouse movement over the background area
    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            // Check if mouse is over the background area (not over sidebar, popups, etc.)
            const target = e.target as HTMLElement;

            // Check if hovering over the controls themselves
            const isOverControls = target.closest('[data-background-controls="true"]') !== null;

            const isOverBackground =
                isOverControls ||
                target.id === "app-bg" ||
                target.closest(".absolute.inset-0.-z-10") !== null ||
                (e.clientX > 64 && !target.closest(".z-40") && !target.closest(".z-50"));

            if (hoveredStateRef.current !== isOverBackground) {
                hoveredStateRef.current = isOverBackground;
                setIsBackgroundHovered(isOverBackground);
            }
        };

        const handleMouseLeave = () => {
            if (hoveredStateRef.current) {
                hoveredStateRef.current = false;
                setIsBackgroundHovered(false);
            }
        };

        document.addEventListener("mousemove", handleMouseMove);
        document.addEventListener("mouseleave", handleMouseLeave);

        return () => {
            document.removeEventListener("mousemove", handleMouseMove);
            document.removeEventListener("mouseleave", handleMouseLeave);
        };
    }, []);

    const handlePrevious = () => {
        const newIndex = (currentIndex - 1 + availableBackgrounds.length) % availableBackgrounds.length;
        setCurrentIndex(newIndex);
        setIsPaused(false);
        onBackgroundChange(availableBackgrounds[newIndex].src);
    };

    const handleNext = () => {
        const newIndex = (currentIndex + 1) % availableBackgrounds.length;
        setCurrentIndex(newIndex);
        setIsPaused(false);
        onBackgroundChange(availableBackgrounds[newIndex].src);
    };

    const handleTogglePause = () => {
        setIsPaused(!isPaused);
    };
    const hasMultipleBackgrounds = availableBackgrounds.length > 1;

    if (!isVisible || availableBackgrounds.length === 0) return null;

    // On Linux, if there's only one background (static), don't show controls at all
    if (isLinux && !hasMultipleBackgrounds) return null;

    return (
        <div
            data-background-controls="true"
            className={`fixed bottom-4 left-1/2 -translate-x-1/2 z-30 pointer-events-auto transition-all duration-300 ease-out ${isBackgroundHovered ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2 pointer-events-none'
                }`}
        >
            <div className="flex items-center gap-2">
                {/* Previous Button - always takes up space */}
                <button
                    onClick={handlePrevious}
                    disabled={!hasMultipleBackgrounds}
                    className={`w-8 h-8 rounded-full bg-black/50 border border-white/20 hover:bg-black/70 hover:border-white/40 transition-all duration-200 flex items-center justify-center group shadow-xl ${hasMultipleBackgrounds ? 'opacity-100' : 'opacity-0 pointer-events-none'
                        }`}
                    title="Previous Background"
                >
                    <svg
                        className="w-3.5 h-3.5 text-white/90 group-hover:text-white transition-colors"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15 19l-7-7 7-7" />
                    </svg>
                </button>

                {/* Middle Control - Always render to prevent layout shift */}
                {isDynamicBackground ? (
                    <button
                        onClick={handleTogglePause}
                        className="w-10 h-10 rounded-full bg-black/50 border border-white/20 hover:bg-black/70 hover:border-white/40 transition-all duration-200 flex items-center justify-center group shadow-xl"
                        title={isPaused ? "Play" : "Pause"}
                    >
                        {isPaused ? (
                            <svg
                                className="w-4 h-4 text-white/90 group-hover:text-white transition-colors ml-0.5"
                                fill="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path d="M8 5v14l11-7z" />
                            </svg>
                        ) : (
                            <svg
                                className="w-4 h-4 text-white/90 group-hover:text-white transition-colors"
                                fill="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
                            </svg>
                        )}
                    </button>
                ) : (
                    // Static indicator to maintain spacing/layout
                    <div
                        className="w-10 h-10 rounded-full bg-black/30 border border-white/10 flex items-center justify-center cursor-default"
                        title="Static Background"
                    >
                        <svg
                            className="w-4 h-4 text-white/50"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                    </div>
                )}

                {/* Next Button - always takes up space */}
                <button
                    onClick={handleNext}
                    disabled={!hasMultipleBackgrounds}
                    className={`w-8 h-8 rounded-full bg-black/50 border border-white/20 hover:bg-black/70 hover:border-white/40 transition-all duration-200 flex items-center justify-center group shadow-xl ${hasMultipleBackgrounds ? 'opacity-100' : 'opacity-0 pointer-events-none'
                        }`}
                    title="Next Background"
                >
                    <svg
                        className="w-3.5 h-3.5 text-white/90 group-hover:text-white transition-colors"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" />
                    </svg>
                </button>
            </div>
        </div>
    );
};

export default BackgroundControls;
