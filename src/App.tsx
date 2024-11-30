import "./App.css";
import {useEffect, useState} from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/addrepo/AddRepo.tsx";
import SidebarIcon from "./components/SidebarIcon.tsx";
import {invoke} from "@tauri-apps/api/core";
import SidebarSettings from "./components/SidebarSettings.tsx";
import {Rocket, Settings} from "lucide-react";

function App() {
    const [openPopup, setOpenPopup] = useState<POPUPS>(POPUPS.NONE);
    const [currentGame, setCurrentGame] = useState<string>("")

    const [repos, setRepos] = useState([])
    const [games, setGames] = useState<any[]>([])
    //const [gamesinfo, setGamesInfo] = useState<any[]>([])

    useEffect(() => {
            // TODO: why is this crap looping twenty billion million septillion times into fuckland????
            invoke("list_repositories").then(r => {
                if (r === null) {
                    setRepos([])
                } else {
                    setRepos(JSON.parse(r as string))
                }
            }).catch(e => {
                console.error("Error while listing database repositories information: " + e)
            })
    }, [repos])

    useEffect(() => {
        setGames([])
            repos.forEach(r => {
                invoke("list_manifests_by_repository_id", { repositoryId: r.id }).then(m => {
                    if (m === null) {
                        console.error("Manifest database table contains nothing, some serious fuck up happened!")
                    } else {
                        setGames([...JSON.parse(m as string)])
                    }
                }).catch(e => {
                    console.error("Error while listing database manifest information: " + e)
                })
            })
    }, [repos])

    // TODO: Fix this shit to actually work and can be used to render images in UI...
    /*useEffect(() => {
        setGamesInfo([])
        // Shitty ManifestLoader backend SOMETIMES retrieval returns fuckshit null somehow??? how???
        games.forEach(r => {
            invoke("get_game_manifest_by_filename", { filename: r.filename }).then(m => {
                if (m === null) {
                    console.error("GameManifest repository fetch issue, some serious fuck up happened!")
                } else {
                    let data = [];
                    data.push(JSON.parse(m as string));

                    setGamesInfo(data)
                }
            }).catch(e => {
                console.error("Error while listing game manifest information: " + e)
            })
        })
    }, [games])*/

    useEffect(() => {
        if (games.length > 0 && currentGame == "") {
            setCurrentGame(games[0].id)
        }
    }, [games])

    return (
        <main className="w-full h-screen flex flex-row bg-transparent">
            <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0 -z-10" alt={"?"} src={(games.filter(v => v.id == currentGame)[0] || {banner: ""}).banner}/>
            <div className="h-full w-16 p-2 bg-black/50 flex flex-col gap-4 items-center fixed-backdrop-blur-md justify-between">
                <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                    {currentGame != "" && games.map((game) => {
                        return (
                            <SidebarIcon key={game.id} popup={openPopup} icon={game.icon} name={game.display_name} id={game.id} setCurrentGame={setCurrentGame} setOpenPopup={setOpenPopup} />
                        )
                    })}
                </div>
                <SidebarSettings popup={openPopup} setOpenPopup={setOpenPopup} />
            </div>
            {/*<h1 className="self-start text-4xl text-bla font-black z-10">KeqingLauncher (InDev)</h1>*/}
            {currentGame != "" && games.filter(v => v.id == currentGame)[0].icon ?
                <img className="h-24 ml-8 mt-8 pointer-events-none" src={currentGame != "" && games.filter(v => v.id == currentGame)[0].icon} alt={"?"}/> /*<React.Fragment></React.Fragment>*/ :
                <h1 className="text-3xl font-black text-white ml-8 mt-8">{currentGame != "" && games.filter(v => v.id == currentGame)[0].display_name}</h1>
            }

            <div className="flex flex-row absolute bottom-8 right-16 gap-4">
                <button>
                    <Settings className="text-white w-8 h-8" />
                </button>
                <button className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg" onClick={() => {
                    setOpenPopup(POPUPS.ADDREPO)
                }}>
                    <Rocket/>
                    <span className="font-semibold translate-y-px">Launch!</span>
                </button>
            </div>

            <div
                className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${openPopup == POPUPS.NONE ? "hidden" : "flex fixed-backdrop-blur-lg bg-white/10"}`}>
                {openPopup == POPUPS.REPOMANAGER && <RepoManager setOpenPopup={setOpenPopup}/>}
                {openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={setOpenPopup}/>}
            </div>
        </main>
    )
}

export default App;
