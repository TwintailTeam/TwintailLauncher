
export default function TextDisplay({ id, name, value, style }: { id: string, name: string, value: string, style: string }) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-8">
            <span className="text-white text-sm">{name}</span>
            <div className={"overflow-ellipsis inline-flex flex-row items-center justify-center"}>
                <p id={id} className={style}>{value}</p>
            </div>
        </div>
    )
}
