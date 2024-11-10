import "./App.css";
import {useState} from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/addrepo/AddRepo.tsx";
import {Settings} from "lucide-react";
import React from "react";

const GAMES = [
    {
        id: "hk4e",
        name: "Genshin Impact",
        icon: "https://play-lh.googleusercontent.com/iP2i_f23Z6I-5hoL2okPS4SxOGhj0q61Iyb0Y1m4xdTsbnaCmrjs7xKRnL6o5R4h-Yg=s48",
        logo: "https://static.wikia.nocookie.net/gensin-impact/images/e/e6/Site-logo.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/10/28/a7541c92a2f326708001acd68a0cb264_946784582385230578.webp"
        // banner: "https://i.pinimg.com/originals/94/c0/46/94c04617c2dad0ad6ac421be5448cc01.jpg"
    },
    {
        id: "nap",
        name: "Zenless Zone Zero",
        icon: "https://play-lh.googleusercontent.com/DEkjrvPufl6TG4Gxq4m8goCSLYiE1bLNOTnlKrJbHDOAWZT40qG3oyALMZJ2BPHJoe8=s48",
        logo: "https://i.imgur.com/IQiQNcF.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/11/04/9339c79f87f2bcb7857c94e2119ca783_5810670824856588817.webp"
        // banner: "https://i.pinimg.com/originals/f7/03/50/f703508f14eaa998f155b922f13b7c2c.png"
    },
    {
        id: "hkrpg",
        name: "Honkai: Star Rail",
        icon: "https://play-lh.googleusercontent.com/cM6aszB0SawZNoAIPvtvy4xsfeFi5iXVBhZB57o-EGPWqE4pbyIUlKJzmdkH8hytuuQ=s48",
        logo: "https://static.wikia.nocookie.net/houkai-star-rail/images/2/29/Honkai_Star_Rail.png",
        banner: "https://launcher-webstatic.hoyoverse.com/launcher-public/2024/10/21/bf1bff6dfb3ee85ebdd90560bd6c3c76_2130545253635554566.webp"
        // banner: "https://cdna.artstation.com/p/assets/images/images/047/299/960/large/grc-koa-the-namelesss-mc.jpg"
    }
]

function App() {
    const [openPopup, setOpenPopup] = useState<POPUPS>(POPUPS.NONE);
    const [currentGame, setCurrentGame] = useState<string>("hk4e")

    return (
        <main className="w-full h-screen flex flex-row bg-black">
            <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0" src={GAMES.filter(v => v.id == currentGame)[0].banner} />
            <div className="h-full w-18 p-4 bg-black/50 z-10 flex flex-col gap-4 items-center backdrop-blur">
                {GAMES.map((game) => {
                    return (
                        <img className="aspect-square w-full rounded-lg cursor-pointer" src={game.icon} onClick={() => {setCurrentGame(game.id)}} />
                    )
                })}
                <div className="flex-grow">{/* Spacer */}</div>
                <Settings className="text-white w-8 h-10 mb-2 cursor-pointer" onClick={() => setOpenPopup(POPUPS.REPOMANAGER)} />
            </div>
            {/*<h1 className="self-start text-4xl text-bla font-black z-10">KeqingLauncher (InDev)</h1>*/}
            {GAMES.filter(v => v.id == currentGame)[0].logo ?
                /*<img className="z-10 h-24 ml-8 mt-8" src={GAMES.filter(v => v.id == currentGame)[0].logo}/>*/ <React.Fragment></React.Fragment> :
                <h1 className="z-10 text-3xl font-black text-white ml-8 mt-8">{GAMES.filter(v => v.id == currentGame)[0].name}</h1>
            }
            <div className={`absolute items-center justify-center top-0 bottom-0 left-0 right-0 p-8 z-20 ${openPopup == POPUPS.NONE ? "hidden" : "flex"}`}>
                {openPopup == POPUPS.REPOMANAGER && <RepoManager setOpenPopup={setOpenPopup}/>}
                {openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={setOpenPopup}/>}
            </div>
        </main>
    )
}

export default App;
