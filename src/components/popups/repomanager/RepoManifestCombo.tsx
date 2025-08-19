import {useState, useEffect} from "react";
import {ChevronDown} from "lucide-react";
import RepoManifestItem from "./RepoManifestItem.tsx";

export default function RepoManifestCombo({name, items, roundTop, roundBottom, fetchRepositories}: { name: string, items: string[], roundTop: boolean, roundBottom: boolean, fetchRepositories: () => void}) {
    const [isFolded, setIsFolded] = useState<boolean>(true);
    const [isVisible, setIsVisible] = useState<boolean>(false);

    // Handle mount/unmount for closing animation
    useEffect(() => {
        if (!isFolded) {
            const timeout = setTimeout(() => setIsVisible(true), 100);
            return () => clearTimeout(timeout);
        } else {
            setIsVisible(false);
        }
    }, [isFolded]);

    return (
        <div
            className={`w-full overflow-hidden ${roundTop ? "rounded-t-xl" : ""} ${roundBottom ? "rounded-b-xl" : ""}`}
            style={{
                boxShadow: isFolded ? "0 2px 8px 0 rgba(0,0,0,0.10)" : "0 8px 24px 0 rgba(0,0,0,0.18)",
                transition: "box-shadow 0.4s cubic-bezier(.4,0,.2,1)",
                background: "linear-gradient(to bottom right, #23272aCC 0%, #2c2f33B3 100%)"
            }}
        >
            <div
                className={`w-full h-14 flex flex-row items-center justify-between p-4 transition-all duration-400 ease-in-out ${roundTop ? "rounded-t-xl" : ""}`}
                style={{
                    borderBottomLeftRadius: (!isFolded && roundBottom) ? "0" : roundBottom ? "0.75rem" : undefined,
                    borderBottomRightRadius: (!isFolded && roundBottom) ? "0" : roundBottom ? "0.75rem" : undefined,
                    borderTopLeftRadius: roundTop ? "0.75rem" : undefined,
                    borderTopRightRadius: roundTop ? "0.75rem" : undefined,
                    border: "1px solid rgba(255,255,255,0.07)",
                    borderBottom: "1px solid rgba(255,255,255,0.10)",
                    background: "transparent",
                    transition: "border-radius 0.5s cubic-bezier(.4,0,.2,1), border-bottom 0.4s cubic-bezier(.4,0,.2,1)"
                }}
            >
                <span className="text-white">{name}</span>
                <span
                    className="h-10 w-10 flex items-center justify-center hover:bg-white/20 border-x-4 border-y-5 border-transparent transition rounded-xl cursor-pointer duration-400"
                    onClick={() => setIsFolded(!isFolded)}
                >
                    <ChevronDown
                        color="white"
                        className={`transition-transform duration-400 ${isFolded ? "rotate-0" : "rotate-180"}`}
                    />
                </span>
            </div>
            <div
                className={`w-full overflow-hidden transition-all duration-250 ease-in-out`}
                style={{
                    maxHeight: isVisible ? `${items.length * 80 + 32}px` : "0px",
                    opacity: isVisible ? 1 : 0,
                    transform: isVisible ? "translateY(0) scaleY(1)" : "translateY(-16px) scaleY(0.98)",
                    transition: "max-height 0.25s cubic-bezier(.4,0,.2,1), opacity 0.2s cubic-bezier(.4,0,.2,1), transform 0.25s cubic-bezier(.4,0,.2,1), border-radius 0.25s cubic-bezier(.4,0,.2,1)",
                    borderBottomLeftRadius: isVisible ? (roundBottom ? "0.75rem" : undefined) : (roundBottom ? "0.75rem" : undefined),
                    borderBottomRightRadius: isVisible ? (roundBottom ? "0.75rem" : undefined) : (roundBottom ? "0.75rem" : undefined),
                    background: "transparent"
                }}
            >
                <div
                    className={`flex flex-col gap-4 p-4 transition-all duration-400 delay-75 ${isVisible ? "transform translate-y-0 opacity-100" : "transform -translate-y-4 opacity-0"}`}
                    style={{
                        transition: "all 0.4s cubic-bezier(.4,0,.2,1)"
                    }}
                >
                    {items.map((name1: any) => {
                        return (
                            <RepoManifestItem name={name1.display_name} key={name1.id} id={name1.id} enabled={name1.enabled} fetchRepositories={fetchRepositories} repo={name} />
                        )
                    })}
                </div>
            </div>
        </div>
    )
}
