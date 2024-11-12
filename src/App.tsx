import "./App.css";
import {useEffect, useState} from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/addrepo/AddRepo.tsx";
import {Settings} from "lucide-react";
import SidebarIcon from "./components/SidebarIcon.tsx";
import {invoke} from "@tauri-apps/api/core";

// TODO: Replace with what Tukan sent
const GAMES = [
    {
        id: "hk4e",
        name: "Genshin Impact",
        icon: "https://fastcdn.hoyoverse.com/static-resource-v2/2024/05/30/2e81b12276a2260c10133124e6ead00e_127960437642481968.png",
        // TODO: Logo is white
        logo: "https://fastcdn.hoyoverse.com/static-resource-v2/2024/05/30/7f777384d29cac1ea0d34b4e16d4487b_2092553358310716519.png",
        banner: "https://fastcdn.hoyoverse.com/static-resource-v2/2024/09/27/1791451c6bd8d9e8d42baca4464ccbb2_189747566222151851.webp"
        // banner: "https://i.pinimg.com/originals/94/c0/46/94c04617c2dad0ad6ac421be5448cc01.jpg"
    },
    {
        id: "nap",
        name: "Zenless Zone Zero",
        icon: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/16/db01dfe29c36a283b774d295b2322b54_44421242156638826.png",
        logo: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/07/03/beab753b7e3d2a50b372697647fa2e72_8064756669861889000.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/06/17/107ec31f7c257bc451f37012d3abbff8_5998717199893026638.webp"
        // banner: "https://i.pinimg.com/originals/f7/03/50/f703508f14eaa998f155b922f13b7c2c.png"
    },
    {
        id: "hkrpg",
        name: "Honkai: Star Rail",
        icon: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/15/39992ccaeef09cedf09009d8901a81d2_7633332456941254056.png",
        logo: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/16/7d03590f49a14d80da4ae256bf34c5f8_5507399597629772203.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/10/15/687c88f0ab6bdae5d566134acbc2dbbd_1744689659015529003.webp"
        // banner: "https://cdna.artstation.com/p/assets/images/images/047/299/960/large/grc-koa-the-namelesss-mc.jpg"
    },
    {
        id: "bh3",
        name: "Honkai Impact 3rd",
        icon: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/04/16/3526a039dddb4c8ca9d1083555f5781b_6607259190236081778.png",
        logo: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/09/24/746d0f9bdea8bec75dcba30363ee4869_3926270346992269590.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/09/13/1aa568b2f1d93bf5e9ed30fefb4b7de7_2138801250860115276.webp"
        // banner: "https://cdna.artstation.com/p/assets/images/images/047/299/960/large/grc-koa-the-namelesss-mc.jpg"
    }
]

const ENABLE_COMMANDS = false

function App() {
    const [openPopup, setOpenPopup] = useState<POPUPS>(POPUPS.NONE);
    const [currentGame, setCurrentGame] = useState<string>("hk4e")

    const [games, setGames] = useState([])

    useEffect(() => {
        invoke("get-games").then(r => {
            setGames(JSON.parse(r as string))
        })
    }, [])

    return (
        <main className="w-full h-screen flex flex-row bg-transparent">
            <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0 -z-10" src={GAMES.filter(v => v.id == currentGame)[0].banner} />
            <div className="h-full w-16 p-2 bg-black/50 flex flex-col gap-4 items-center fixed-backdrop-blur-md">
                {(ENABLE_COMMANDS ? games : GAMES).map((game) => {
                    return (
                        <SidebarIcon key={game.id} icon={game.icon} name={game.name} id={game.id} setCurrentGame={setCurrentGame} />
                    )
                })}
                <div className="flex-grow">{/* Spacer */}</div>
                <Settings className="text-white w-8 h-10 mb-2 cursor-pointer" onClick={() => setOpenPopup(POPUPS.REPOMANAGER)} />
            </div>
            {/*<h1 className="self-start text-4xl text-bla font-black z-10">KeqingLauncher (InDev)</h1>*/}
            {(ENABLE_COMMANDS ? games : GAMES).filter(v => v.id == currentGame)[0].logo ?
                <img className="h-24 ml-8 mt-8" src={(ENABLE_COMMANDS ? games : GAMES).filter(v => v.id == currentGame)[0].logo}/> /*<React.Fragment></React.Fragment>*/ :
                <h1 className="text-3xl font-black text-white ml-8 mt-8">{(ENABLE_COMMANDS ? games : GAMES).filter(v => v.id == currentGame)[0].name}</h1>
            }
            <div className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${openPopup == POPUPS.NONE ? "hidden" : "flex fixed-backdrop-blur-lg bg-white/10"}`}>
                {openPopup == POPUPS.REPOMANAGER && <RepoManager setOpenPopup={setOpenPopup}/>}
                {openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={setOpenPopup}/>}
            </div>
        </main>
    )
}

export default App;
