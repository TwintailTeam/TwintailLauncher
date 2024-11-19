import "./App.css";
import {useEffect, useState} from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/addrepo/AddRepo.tsx";
import SidebarIcon from "./components/SidebarIcon.tsx";
import {invoke} from "@tauri-apps/api/core";
import SidebarSettings from "./components/SidebarSettings.tsx";
import {Rocket, Settings} from "lucide-react";

// TODO: Replace with what Tukan sent
const GAMES = [
    {
        id: "hk4e",
        name: "Genshin Impact",
        icon: "https://fastcdn.hoyoverse.com/static-resource-v2/2024/05/30/2e81b12276a2260c10133124e6ead00e_127960437642481968.png",
        // TODO: Logo is white
        logo: "https://i.imgur.com/OyHaAAn.png",
        banner: "https://fastcdn.hoyoverse.com/static-resource-v2/2024/09/27/1791451c6bd8d9e8d42baca4464ccbb2_189747566222151851.webp"
    },
    {
        id: "nap",
        name: "Zenless Zone Zero",
        icon: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/16/db01dfe29c36a283b774d295b2322b54_44421242156638826.png",
        logo: "https://i.imgur.com/wBuiHDO.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/06/17/107ec31f7c257bc451f37012d3abbff8_5998717199893026638.webp"
    },
    {
        id: "hkrpg",
        name: "Honkai: Star Rail",
        icon: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/15/39992ccaeef09cedf09009d8901a81d2_7633332456941254056.png",
        logo: "https://i.imgur.com/irBhTMR.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/10/15/687c88f0ab6bdae5d566134acbc2dbbd_1744689659015529003.webp"
    },
    {
        id: "bh3",
        name: "Honkai Impact 3rd",
        icon: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/16/3526a039dddb4c8ca9d1083555f5781b_6607259190236081778.png",
        logo: "https://i.imgur.com/6HOcClJ.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/09/13/1aa568b2f1d93bf5e9ed30fefb4b7de7_2138801250860115276.webp"
    },
]

const ENABLE_COMMANDS = false

function App() {
    const [openPopup, setOpenPopup] = useState<POPUPS>(POPUPS.NONE);
    const [currentGame, setCurrentGame] = useState<string>("")

    const [repos, setRepos] = useState([])
    const [games, setGames] = useState<any[]>([])

    useEffect(() => {
        invoke("list_repositories").then(r => {
            console.log(r || "nope")
            if (r === null) {
                setRepos([])
            } else {
                setRepos(JSON.parse(r as string))
            }
        }).catch(() => {
            console.error("AAA")
        }).then(() => {
            console.log("Hi?")
        })

        // example how to argument lol even how to validate null as good example
        /*invoke("get_manifests_by_repository_id", {repositoryId: ""}).then(r => {
            if (r === null) {
                console.log("its null")
            } else {
                console.log(JSON.parse(r as string))
            }
        })*/
    }, [])

    useEffect(() => {
        setGames([])
        repos.forEach(r => {
            invoke("list_manifests_by_repository_id", { repositoryId: r.id }).then(m => {
                if (m === null) {
                    console.log("its null")
                } else {
                    setGames([...games, ...JSON.parse(m as string)])
                }

            })
        })
    }, [repos])

    useEffect(() => {
        if (games.length > 0 && currentGame == "") {
            setCurrentGame(games[0].id)
        }
    }, [games])

    return (
        <main className="w-full h-screen flex flex-row bg-transparent">
            <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0 -z-10" src={(GAMES.filter(v => v.id == currentGame)[0] || {banner: ""}).banner} />
            <div className="h-full w-16 p-2 bg-black/50 flex flex-col gap-4 items-center fixed-backdrop-blur-md justify-between">
                <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                    {currentGame != "" && (ENABLE_COMMANDS ? games : GAMES).map((game) => {
                        return (
                            <SidebarIcon key={game.id} popup={openPopup} icon={game.icon} name={game.name} id={game.id} setCurrentGame={setCurrentGame} setOpenPopup={setOpenPopup} />
                        )
                    })}
                </div>
                <SidebarSettings popup={openPopup} setOpenPopup={setOpenPopup} />
            </div>
            {/*<h1 className="self-start text-4xl text-bla font-black z-10">KeqingLauncher (InDev)</h1>*/}
            {currentGame != "" && (ENABLE_COMMANDS ? games : GAMES).filter(v => v.id == currentGame)[0].logo ?
                <img className="h-24 ml-8 mt-8 pointer-events-none" src={currentGame != "" && (ENABLE_COMMANDS ? games : GAMES).filter(v => v.id == currentGame)[0].logo}/> /*<React.Fragment></React.Fragment>*/ :
                <h1 className="text-3xl font-black text-white ml-8 mt-8">{currentGame != "" && (ENABLE_COMMANDS ? games : GAMES).filter(v => v.id == currentGame)[0].name}</h1>
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
