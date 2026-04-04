import { AlertCircle, AlertTriangle, Info, HelpCircle, X } from "lucide-react";
import { DialogType, DialogButton } from "../../context/DialogContext";

interface DialogProps {
    type: DialogType;
    title: string;
    message: string;
    buttons: DialogButton[];
    isClosing: boolean;
    onButtonClick: (index: number) => void;
    onClose: () => void;
}

const iconMap: Record<DialogType, React.ReactNode> = {
    error: <AlertCircle className="w-8 h-8 text-red-500" />,
    warning: <AlertTriangle className="w-8 h-8 text-amber-500" />,
    info: <Info className="w-8 h-8 text-blue-400" />,
    confirm: <HelpCircle className="w-8 h-8 text-purple-400" />,
};

const titleGradientMap: Record<DialogType, string> = {
    error: "from-white to-red-200",
    warning: "from-white to-amber-200",
    info: "from-white to-blue-200",
    confirm: "from-white to-purple-200",
};

export default function Dialog({
    type,
    title,
    message,
    buttons,
    isClosing,
    onButtonClick,
    onClose,
}: DialogProps) {
    return (
        <div
            className={`rounded-2xl w-[90vw] max-w-lg bg-[#0c0c0c] border border-white/10 flex flex-col p-6 overflow-hidden shadow-2xl ${isClosing ? "animate-zoom-out" : "animate-zoom-in"
                }`}
        >
            {/* Header */}
            <div className="flex flex-row items-center justify-between mb-4">
                <div className="flex flex-row items-center gap-4">
                    <div className="p-2 rounded-xl bg-white/5">{iconMap[type]}</div>
                    <h2
                        className={`text-white font-bold text-xl bg-gradient-to-r ${titleGradientMap[type]} bg-clip-text text-transparent`}
                    >
                        {title}
                    </h2>
                </div>
                <button
                    onClick={onClose}
                    className="text-gray-400 hover:text-white hover:bg-white/10 rounded-lg p-2 transition-all duration-200"
                >
                    <X className="w-5 h-5" />
                </button>
            </div>

            {/* Message */}
            <div className="bg-zinc-900/60 border border-white/10 rounded-xl p-4 mb-4">
                <p className="text-gray-200 text-sm leading-relaxed whitespace-pre-wrap">
                    {message}
                </p>
            </div>

            {/* Buttons */}
            <div className="flex justify-end gap-3">
                {buttons.map((button, index) => {
                    let buttonClass =
                        "px-5 py-2.5 rounded-xl font-medium transition-all duration-200 transform hover:scale-105 text-sm";

                    if (button.variant === "primary") {
                        buttonClass +=
                            " bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 text-white";
                    } else if (button.variant === "danger") {
                        buttonClass +=
                            " bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 text-white";
                    } else {
                        buttonClass +=
                            " bg-white/5 hover:bg-white/10 text-gray-300 border border-white/10";
                    }

                    return (
                        <button
                            key={index}
                            className={buttonClass}
                            onClick={() => {
                                if (button.onClick) {
                                    button.onClick();
                                }
                                if (!button.preventClose) {
                                    onButtonClick(index);
                                }
                            }}
                        >
                            {button.label}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
