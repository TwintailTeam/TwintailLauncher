import React, { useEffect, useRef } from "react";
import SidebarIconManifest from "../sidebar/SidebarIconManifest.tsx";
import { POPUPS } from "../popups/POPUPS.ts";

interface GameInfoItem {
  manifest_enabled: boolean;
  assets: { game_icon: string; game_background: string };
  filename: string;
  icon: string;
  display_name: string;
  biz: string;
}

interface ManifestsPanelProps {
  openPopup: POPUPS;
  manifestsOpenVisual: boolean;
  manifestsInitialLoading: boolean;
  gamesinfo: GameInfoItem[];
  manifestsPanelRef: React.RefObject<HTMLDivElement>;
  setCurrentGame: (id: string) => void;
  setOpenPopup: (p: POPUPS) => void;
  setDisplayName: (name: string) => void;
  setBackground: (src: string) => void;
  setCurrentInstall: (id: string) => void;
  setGameIcon: (src: string) => void;
  onAutoHide?: () => void;
  autoHideMs?: number;
}

const ManifestsPanel: React.FC<ManifestsPanelProps> = ({
  openPopup,
  manifestsOpenVisual,
  manifestsInitialLoading,
  gamesinfo,
  manifestsPanelRef,
  setCurrentGame,
  setOpenPopup,
  setDisplayName,
  setBackground,
  setCurrentInstall,
  setGameIcon,
  onAutoHide,
  autoHideMs = 5000,
}) => {
  // Local timer ref for inactivity-based auto-hide
  const hideTimerRef = useRef<number | undefined>(undefined);

  // Helper to clear any pending timer
  const clearHideTimer = () => {
    if (hideTimerRef.current) {
      window.clearTimeout(hideTimerRef.current);
      hideTimerRef.current = undefined;
    }
  };

  // Schedule auto-hide only when panel is visible and no popup is open
  const scheduleHide = () => {
    clearHideTimer();
    if (manifestsOpenVisual && openPopup === POPUPS.NONE && onAutoHide) {
      hideTimerRef.current = window.setTimeout(() => {
        // Double-check before firing
        if (manifestsOpenVisual && openPopup === POPUPS.NONE) {
          onAutoHide();
        }
      }, autoHideMs);
    }
  };

  // React to visibility/popup state changes
  useEffect(() => {
    scheduleHide();
    return () => clearHideTimer();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [manifestsOpenVisual, openPopup]);

  return (
    <div className="absolute top-0 left-16 right-0 z-20 pointer-events-none">
      <div className="pl-3 pt-2 pr-6">
        <div
          ref={manifestsPanelRef}
          onMouseEnter={clearHideTimer}
          onMouseLeave={scheduleHide}
          className={"relative inline-flex rounded-2xl border border-white/10 bg-black/50 shadow-2xl overflow-hidden pointer-events-auto origin-left"}
          style={{
            clipPath: manifestsOpenVisual ? 'inset(0 0% 0 0)' : 'inset(0 100% 0 0)',
            transition: 'clip-path 400ms ease',
            transform: (openPopup != POPUPS.NONE) ? 'translateY(-8px)' : 'translateY(0)',
            opacity: (openPopup != POPUPS.NONE) ? 0 : 1,
            willChange: 'clip-path, transform, opacity'
          }}
        >
          <div className="flex flex-row items-center gap-2 overflow-x-auto px-3 py-2 scrollbar-none">
            {gamesinfo.map((game, index) => {
              const opening = manifestsOpenVisual;
              const delayMs = manifestsInitialLoading
                ? (index * 100 + 400)
                : (opening ? (index * 60 + 50) : ((gamesinfo.length - index - 1) * 50));
              return (
                <div
                  key={game.biz}
                  className={manifestsInitialLoading ? 'animate-slideInLeft' : ''}
                  style={{
                    transition: 'transform 300ms ease, opacity 300ms ease',
                    transform: opening ? 'translateX(0)' : 'translateX(-20px)',
                    opacity: opening ? 1 : 0,
                    transitionDelay: `${delayMs}ms`
                  }}
                >
                  <SidebarIconManifest
                    variant="floating"
                    sizeClass="w-12"
                    popup={openPopup}
                    icon={game.assets.game_icon}
                    background={game.assets.game_background}
                    name={game.display_name}
                    enabled={game.manifest_enabled}
                    id={game.biz}
                    setCurrentGame={setCurrentGame}
                    setOpenPopup={setOpenPopup}
                    setDisplayName={setDisplayName}
                    setBackground={setBackground}
                    setCurrentInstall={setCurrentInstall}
                    setGameIcon={setGameIcon}
                  />
                </div>
              )
            })}
          </div>
          {/* Edge fades to hint horizontal scroll */}
          <div className="pointer-events-none absolute top-0 left-0 h-full w-10 bg-gradient-to-r from-black/50 to-transparent"/>
          <div className="pointer-events-none absolute top-0 right-0 h-full w-12 bg-gradient-to-l from-black/50 to-transparent"/>
        </div>
      </div>
    </div>
  );
};

export default ManifestsPanel;
