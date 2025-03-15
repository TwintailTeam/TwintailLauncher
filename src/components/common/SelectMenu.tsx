

export default function SelectMenu({ id, name, options }: { id: string, name: string, options: any }) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-6">
            <span className="text-white text-sm">{name}</span>
            <div className="inline-flex flex-row items-center justify-center">
                <select id={id} className={"w-full focus:outline-none h-8 rounded-lg bg-white/20 text-white px-2 pr-32 placeholder-white/50 appearance-none cursor-pointer"}>
                    {options.map((option: any) => (
                        <option key={option.value} value={option.value}>{option.name}</option>
                    ))}
                </select>
            </div>
        </div>
    )
}
