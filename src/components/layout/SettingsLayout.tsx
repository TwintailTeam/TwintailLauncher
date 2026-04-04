import React, { useState, useEffect } from "react";
import { X } from "lucide-react";
import { CachedImage } from "../common/CachedImage.tsx";


interface SettingsLayoutProps {
    title: string;
    onClose: () => void;
    children: React.ReactNode;
    banner?: string;
    icon?: string;
    imageVersion?: number; // Used to force image re-load after network recovery
}

export const SettingsLayout = ({ title, onClose, children, banner, icon, imageVersion = 0 }: SettingsLayoutProps) => {
    // Defer content rendering to allow animation to start smoothly
    const [_isReady, setIsReady] = useState(false);

    useEffect(() => {
        // Use requestAnimationFrame to defer content until after first paint
        const raf = requestAnimationFrame(() => {
            setIsReady(true);
        });
        return () => cancelAnimationFrame(raf);
    }, []);

    return (
        <div className="rounded-2xl w-[85vw] max-w-7xl h-[80vh] bg-[#09090b] border border-white/10 flex flex-col overflow-hidden shadow-2xl animate-zoom-in relative group/settings" style={{ willChange: 'transform, opacity', backfaceVisibility: 'hidden', WebkitBackfaceVisibility: 'hidden', transform: 'translateZ(0)' }}>

            {/* Header / Hero Section */}
            {banner ? (
                <div className="relative h-48 shrink-0 overflow-hidden">
                    {/* Banner Image */}
                    <div className="absolute inset-0 bg-zinc-900">
                        <CachedImage key={`banner-v${imageVersion}`} src={banner} className="w-full h-full object-cover object-center opacity-80" />
                        {/* Extended past bottom edge to fix WebKitGTK subpixel rendering gap */}
                        <div className="absolute top-0 left-0 right-0 -bottom-1 bg-gradient-to-b from-black/20 via-black/40 to-[#09090b]" />
                    </div>

                    {/* Content */}
                    <div className="absolute inset-x-0 bottom-0 p-8 flex items-end justify-between">
                        <div className="flex items-center gap-6">
                            {icon && (
                                <div className="w-20 h-20 rounded-2xl overflow-hidden shadow-2xl border border-white/10 bg-black/80 transition-transform duration-500 hover:scale-105">
                                    <CachedImage key={`icon-v${imageVersion}`} src={icon} className="w-full h-full object-cover" />
                                </div>
                            )}
                            <div className="mb-1">
                                <h1 className="text-4xl font-bold text-white tracking-tight drop-shadow-md">
                                    {title}
                                </h1>
                            </div>
                        </div>
                    </div>

                    {/* Close Button (Floating) */}
                    <button
                        onClick={onClose}
                        className="absolute top-6 right-6 p-2 rounded-full bg-black/60 hover:bg-white/10 border border-white/5 transition-all duration-200 hover:scale-105 group-hover/settings:opacity-100 opacity-0"
                    >
                        <X className="w-6 h-6 text-white/70 group-hover:text-white" />
                    </button>
                </div>
            ) : (
                /* Standard Header */
                <div className="flex flex-row items-center justify-between px-8 py-6 border-b border-white/5 bg-white/5">
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-white/60 bg-clip-text text-transparent">
                        {title}
                    </h1>
                    <button
                        onClick={onClose}
                        className="p-2 rounded-full bg-black/20 hover:bg-white/10 border border-white/5 transition-all duration-200 opacity-0 group-hover/settings:opacity-100 hover:scale-105"
                    >
                        <X className="w-6 h-6 text-white/50 group-hover:text-white transition-colors" />
                    </button>
                </div>
            )}

            {/* Main Content Area (Grid) */}
            <div className="flex-1 overflow-hidden relative">
                {children}
            </div>
        </div>
    );
};
