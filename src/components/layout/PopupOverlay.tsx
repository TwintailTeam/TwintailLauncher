import { POPUPS } from "../popups/POPUPS";
import { PAGES } from "../pages/PAGES";
import { useEffect } from "react";
import RepoManager from "../popups/repomanager/RepoManager";
import AddRepo from "../popups/repomanager/AddRepo";
import DownloadGame from "../popups/DownloadGame";
import GameSettings from "../popups/GameSettings.tsx";

export type PopupOverlayProps = {
  openPopup: POPUPS;
  setOpenPopup: (p: POPUPS) => void;

  // Repo manager
  reposList: any[];
  fetchRepositories: () => any;

  // Global settings
  fetchSettings: () => any;
  globalSettings: any;

  // Download game
  downloadSizes: any;
  runnerVersions: any[];
  dxvkVersions: any[];
  gameVersions: any[];
  runners: any[];
  installedRunners: any[];
  fetchInstalledRunners: () => any;
  gameIcon: string;
  gameBackground: string;
  currentGame: string;
  displayName: string;
  openDownloadAsExisting: boolean;
  fetchDownloadSizes: (
    biz: any,
    version: any,
    lang: any,
    path: any,
    region_filter: any,
    callback: (data: any) => void
  ) => void;
  pushInstalls: (...args: any[]) => any;
  setBackground: (f: string) => void;
  setCurrentInstall: (id: string) => void;

  // Install settings
  gamesinfo: any[];
  installSettings: any;
  gameManifest: any;
  setCurrentGame: (biz: string) => void;
  fetchInstallSettings: (installId: string) => Promise<any> | any;
  installGameSwitches: any;
  installGameFps: any[];
  installGameGraphicsApi: any;

  // Delete confirmation
  installs: any[];
  setDisplayName: (name: string) => void;
  setGameIcon: (icon: string) => void;

  // Page navigation
  setCurrentPage: (page: PAGES) => void;

  // Network recovery
  imageVersion?: number;
};

export default function PopupOverlay(props: PopupOverlayProps) {
  const {
    openPopup,
    setOpenPopup,
    reposList,
    fetchRepositories,
    globalSettings,
    downloadSizes,
    runnerVersions,
    dxvkVersions,
    gameVersions,
    installedRunners,
    gameIcon,
    gameBackground,
    currentGame,
    displayName,
    openDownloadAsExisting,
    fetchDownloadSizes,
    pushInstalls,
    setBackground,
    setCurrentInstall,
    gamesinfo,
    installSettings,
    gameManifest,
    setCurrentGame,
    fetchInstallSettings,
    installGameSwitches,
    installGameFps,
    installGameGraphicsApi,
    installs,
    setCurrentPage,
    setDisplayName,
    setGameIcon,
    imageVersion = 0,
  } = props;

  // ESC to close and scroll lock while a popup is open
  useEffect(() => {
    if (openPopup !== POPUPS.NONE) {
      const onKey = (e: KeyboardEvent) => {
        if (e.key === "Escape") setOpenPopup(POPUPS.NONE);
      };
      document.addEventListener("keydown", onKey);
      const prevOverflow = document.body.style.overflow;
      document.body.style.overflow = "hidden";
      return () => {
        document.removeEventListener("keydown", onKey);
        document.body.style.overflow = prevOverflow;
      };
    }
  }, [openPopup, setOpenPopup]);

  const isOpen = openPopup !== POPUPS.NONE;

  return (
    <div
      role="dialog"
      aria-modal={isOpen}
      className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-50 flex ${isOpen ? "animate-backdrop-in" : "pointer-events-none"}`}
      style={{
        backfaceVisibility: 'hidden',
        WebkitBackfaceVisibility: 'hidden',
        transform: 'translateZ(0)',
        // Use visibility for instant hide (no flash), opacity transition handled by animate-backdrop-in
        visibility: isOpen ? 'visible' : 'hidden',
        opacity: isOpen ? 1 : 0,
        background: 'rgba(0,0,0,0.6)'
      }}
      onClick={(e) => { if (e.target === e.currentTarget) { setOpenPopup(POPUPS.NONE); } }}
    >
      {/* Content wrapper - individual popups handle their own animations (zoom-in/scaleIn) */}
      <div>
        {openPopup == POPUPS.REPOMANAGER && (
          <RepoManager
            repos={reposList}
            setOpenPopup={setOpenPopup}
            fetchRepositories={fetchRepositories}
          />
        )}
        {openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={setOpenPopup} />}
        {openPopup == POPUPS.DOWNLOADGAME && (
          <DownloadGame
            fetchDownloadSizes={fetchDownloadSizes}
            disk={downloadSizes}
            runnerVersions={runnerVersions}
            dxvkVersions={dxvkVersions}
            versions={gameVersions}
            icon={gameIcon}
            background={gameBackground}
            biz={currentGame}
            displayName={displayName}
            settings={globalSettings}
            setOpenPopup={setOpenPopup}
            pushInstalls={pushInstalls}
            setBackground={setBackground}
            setCurrentInstall={setCurrentInstall}
            openAsExisting={openDownloadAsExisting}
            setCurrentPage={setCurrentPage}
            imageVersion={imageVersion}
          />
        )}

        {openPopup == POPUPS.INSTALLSETTINGS && (
          <GameSettings
            installedRunners={installedRunners}
            installSettings={installSettings}
            gameManifest={gameManifest}
            setOpenPopup={setOpenPopup}
            setCurrentInstall={setCurrentInstall}
            setCurrentGame={setCurrentGame}
            setBackground={setBackground}
            setDisplayName={setDisplayName}
            setGameIcon={setGameIcon}
            pushInstalls={pushInstalls}
            fetchInstallSettings={fetchInstallSettings}
            prefetchedSwitches={installGameSwitches}
            prefetchedFps={installGameFps}
            prefetchedGraphicsApi={installGameGraphicsApi}
            installs={installs}
            setCurrentPage={setCurrentPage}
            gamesinfo={gamesinfo}
            imageVersion={imageVersion}
          />
        )}
      </div>
    </div>
  );
}
