
export default function TextDisplay({ name, value, style }: { name: string, value: string, style: string }) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-8">
            <span className="text-white text-sm">{name}</span>
            <div className={"overflow-ellipsis inline-flex flex-row items-center justify-center"}>
                <p className={style}>{value}</p>
            </div>
        </div>
    )
}
