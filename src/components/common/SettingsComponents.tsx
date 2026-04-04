import React from "react";
import { ChevronDown, FolderOpen } from "lucide-react";
import { open } from '@tauri-apps/plugin-dialog';
import HelpTooltip from "./HelpTooltip.tsx";

// --- Card Components ---

export const SettingsSection = ({ title, children }: { title: string, children: React.ReactNode }) => (
    <div className="mb-10 animate-fadeIn">
        <h2 className="text-lg font-semibold text-white/90 mb-4 px-1">{title}</h2>
        <div className="flex flex-col gap-4">
            {children}
        </div>
    </div>
);

export const SettingsCard = ({ children, className = "", onClick }: { children: React.ReactNode, className?: string, onClick?: () => void }) => (
    <div className={`bg-zinc-900/85 border border-white/5 rounded-xl p-5 hover:border-white/10 transition-colors ${className}`} onClick={onClick}>
        {children}
    </div>
);

// --- Form Components ---

interface SettingsControlProps {
    label: string;
    description?: string;
    helpText?: string;
    descriptionClassName?: string;
}

export const ModernToggle = ({ label, description, helpText, descriptionClassName, checked, disabled = false, onChange }: SettingsControlProps & { checked: boolean, disabled?: boolean, onChange: (val: boolean) => void }) => {
    const interactive = !disabled;

    return (
        <SettingsCard
            className={`flex flex-row items-center justify-between ${interactive ? "group cursor-pointer" : "cursor-not-allowed opacity-70"}`}
            onClick={interactive ? () => onChange(!checked) : undefined}
        >
            <div className="flex flex-col gap-1 pr-4">
                <div className="flex items-center gap-2">
                    <label className={`text-base font-medium transition-colors pointer-events-none ${interactive ? "text-white group-hover:text-purple-100" : "text-zinc-200"}`}>{label}</label>
                    {description && description.includes("help") ? null : (helpText && <div onClick={e => e.stopPropagation()}><HelpTooltip text={helpText} /></div>)}
                </div>
                {description && <span className={`text-sm pointer-events-none ${descriptionClassName ?? "text-zinc-400"}`}>{description}</span>}
            </div>

            <div
                onClick={(e) => {
                    e.stopPropagation();
                    if (!interactive) return;
                    onChange(!checked);
                }}
                className={`
                    w-12 h-7 rounded-full transition-all duration-300 relative flex items-center shadow-inner
                    ${interactive ? "cursor-pointer" : "cursor-not-allowed"}
                    ${checked ? (interactive ? "bg-purple-600 shadow-[0_0_15px_rgba(147,51,234,0.4)]" : "bg-purple-700/70") : "bg-zinc-800"}
                `}>
                <div className={`
                    w-5 h-5 rounded-full shadow-md transform transition-all duration-300 absolute
                    ${interactive ? "bg-white" : "bg-zinc-300"}
                    ${checked ? "translate-x-6" : "translate-x-1"}
                `} />
            </div>
        </SettingsCard>
    );
};

export const ModernInput = ({ label, description, helpText, value, onChange, onBlur, ...props }: SettingsControlProps & React.InputHTMLAttributes<HTMLInputElement>) => {
    // Use local state to allow typing without immediate save
    const [localValue, setLocalValue] = React.useState(value?.toString() ?? "");

    // Sync local value when prop changes (e.g., after fetchSettings)
    React.useEffect(() => {
        setLocalValue(value?.toString() ?? "");
    }, [value]);

    const handleSave = () => {
        // Only trigger onChange (save) when user is done editing
        if (onChange && localValue !== value?.toString()) {
            // Create a synthetic event for compatibility
            const syntheticEvent = {
                target: { value: localValue }
            } as React.ChangeEvent<HTMLInputElement>;
            onChange(syntheticEvent);
        }
        onBlur?.(undefined as unknown as React.FocusEvent<HTMLInputElement>);
    };

    return (
        <SettingsCard>
            <div className="flex flex-col gap-3">
                <div className="flex flex-col gap-1">
                    <div className="flex items-center gap-2">
                        <label className="text-base font-medium text-white">{label}</label>
                        {helpText && <HelpTooltip text={helpText} />}
                    </div>
                    {description && <span className="text-sm text-zinc-400">{description}</span>}
                </div>
                <input
                    {...props}
                    value={localValue}
                    onChange={(e) => setLocalValue(e.target.value)}
                    onBlur={handleSave}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") {
                            handleSave();
                            (e.target as HTMLInputElement).blur();
                        }
                    }}
                    className="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-white placeholder-zinc-600 focus:outline-none focus:border-purple-500/50 focus:ring-1 focus:ring-purple-500/30 transition-all font-mono text-sm [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none leading-[1.5] [-webkit-line-box-contain:glyphs]"
                />
            </div>
        </SettingsCard>
    );
};

export const ModernPathInput = ({ label, description, value, onChange, folder = true, extensions, ...props }: SettingsControlProps & { value: string, onChange: (val: string) => void, folder?: boolean, extensions?: string[] }) => {
    // Use local state to allow typing without immediate save
    const [localValue, setLocalValue] = React.useState(value ?? "");

    // Sync local value when prop changes (e.g., after fetchSettings or folder picker)
    React.useEffect(() => {
        setLocalValue(value ?? "");
    }, [value]);

    const handleSave = () => {
        // Only trigger onChange (save) when user is done editing and value changed
        if (localValue !== value) {
            onChange(localValue);
        }
    };

    const handleBrowse = async () => {
        try {
            const selected = await open({
                directory: folder,
                multiple: false,
                filters: extensions ? [{ name: 'Allowed files', extensions }] : undefined
            });

            if (selected) {
                const newPath = Array.isArray(selected) ? selected[0] : selected;
                setLocalValue(newPath);
                onChange(newPath); // Folder picker saves immediately
            }
        } catch (e) {
            console.error("Failed to open dialog", e);
        }
    };

    return (
        <SettingsCard>
            <div className="flex flex-col gap-3">
                <div className="flex flex-col gap-1">
                    <div className="flex items-center gap-2">
                        <label className="text-base font-medium text-white">{label}</label>
                        {props.helpText && <HelpTooltip text={props.helpText} />}
                    </div>
                    {description && <span className="text-sm text-zinc-400">{description}</span>}
                </div>
                <div className="flex gap-2">
                    <input
                        value={localValue}
                        onChange={(e) => setLocalValue(e.target.value)}
                        onBlur={handleSave}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") {
                                handleSave();
                                (e.target as HTMLInputElement).blur();
                            }
                        }}
                        className="flex-1 bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-white placeholder-zinc-600 focus:outline-none focus:border-purple-500/50 focus:ring-1 focus:ring-purple-500/30 transition-all font-mono text-sm truncate leading-[1.5] [-webkit-line-box-contain:glyphs]"
                    />
                    <button
                        onClick={handleBrowse}
                        className="bg-zinc-800 hover:bg-zinc-700 text-white p-2.5 rounded-lg border border-white/5 transition-colors">
                        <FolderOpen className="w-5 h-5" />
                    </button>
                </div>
            </div>
        </SettingsCard>
    );
};

export const ModernSelect = ({ label, description, options, value, onChange, ...props }: SettingsControlProps & {
    value: string,
    onChange: (val: string) => void,
    // Support both { value, label } and { value, name } formats for compatibility
    options: { value: string, label?: string, name?: string }[]
}) => {
    return (
        <SettingsCard>
            <div className="flex flex-col gap-3">
                <div className="flex flex-col gap-1">
                    <div className="flex items-center gap-2">
                        <label className="text-base font-medium text-white">{label}</label>
                        {props.helpText && <HelpTooltip text={props.helpText} />}
                    </div>
                    {description && <span className="text-sm text-zinc-400">{description}</span>}
                </div>
                <div className="relative">
                    <select
                        value={value}
                        onChange={(e) => onChange(e.target.value)}
                        className="w-full appearance-none bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:border-purple-500/50 focus:ring-1 focus:ring-purple-500/30 transition-all cursor-pointer"
                    >
                        {options.map(opt => (
                            <option key={opt.value} value={opt.value} className="bg-zinc-900 text-white">
                                {opt.label ?? opt.name ?? opt.value}
                            </option>
                        ))}
                    </select>
                    <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-500 pointer-events-none" />
                </div>
            </div>
        </SettingsCard>
    );
};
