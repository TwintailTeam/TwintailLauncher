import { POPUPS } from "../popups/POPUPS";
import RepoManager from "../popups/repomanager/RepoManager";
import AddRepo from "../popups/repomanager/AddRepo";
import SettingsGlobal from "../popups/settings/SettingsGlobal";
import DownloadGame from "../popups/DownloadGame";
import SettingsInstall from "../popups/settings/SettingsInstall";
import InstallDeleteConfirm from "../popups/settings/InstallDeleteConfirm";

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
    callback: (data: any) => void
  ) => void;
  pushInstalls: (...args: any[]) => any;
  setBackground: (f: string) => void;
  setCurrentInstall: (id: string) => void;

  // Install settings
  gamesinfo: any[];
  installSettings: any;
  setCurrentGame: (biz: string) => void;
  fetchInstallSettings: (installId: string) => Promise<any> | any;
  installGameSwitches: any;
  installGameFps: any[];

  // Delete confirmation
  installs: any[];
};

export default function PopupOverlay(props: PopupOverlayProps) {
  const {
    openPopup,
    setOpenPopup,
    reposList,
    fetchRepositories,
    fetchSettings,
    globalSettings,
    downloadSizes,
    runnerVersions,
    dxvkVersions,
    gameVersions,
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
    setCurrentGame,
    fetchInstallSettings,
    installGameSwitches,
    installGameFps,
    installs,
  } = props;

  return (
    <div
      className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${
        openPopup == POPUPS.NONE ? "hidden" : "flex bg-white/10"
      }`}
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          setOpenPopup(POPUPS.NONE);
        }
      }}
    >
      {openPopup == POPUPS.REPOMANAGER && (
        <RepoManager
          repos={reposList}
          setOpenPopup={setOpenPopup}
          fetchRepositories={fetchRepositories}
        />
      )}
      {openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={setOpenPopup} />}
      {openPopup == POPUPS.SETTINGS && (
        <SettingsGlobal
          fetchSettings={fetchSettings}
          settings={globalSettings}
          setOpenPopup={setOpenPopup}
        />
      )}
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
        />
      )}
      {openPopup == POPUPS.INSTALLSETTINGS && (
        <SettingsInstall
          games={gamesinfo}
          runnerVersions={runnerVersions}
          dxvkVersions={dxvkVersions}
          installSettings={installSettings}
          setOpenPopup={setOpenPopup}
          pushInstalls={pushInstalls}
          setCurrentInstall={setCurrentInstall}
          setCurrentGame={setCurrentGame}
          setBackground={setBackground}
          fetchInstallSettings={fetchInstallSettings}
          prefetchedSwitches={installGameSwitches}
          prefetchedFps={installGameFps}
        />
      )}
      {openPopup == POPUPS.INSTALLDELETECONFIRMATION && (
        <InstallDeleteConfirm
          installs={installs}
          games={gamesinfo}
          install={installSettings}
          setOpenPopup={setOpenPopup}
          pushInstalls={pushInstalls}
          setCurrentInstall={setCurrentInstall}
          setCurrentGame={setCurrentGame}
          setBackground={setBackground}
        />
      )}
    </div>
  );
}
