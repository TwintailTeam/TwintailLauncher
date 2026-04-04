import { useEffect, useState } from "react";
import { useDialog } from "../../context/DialogContext";
import Dialog from "../common/Dialog";

export default function DialogOverlay() {
    const { dialog, closeDialog } = useDialog();
    const [isClosing, setIsClosing] = useState(false);

    // Handle ESC key to close dialog
    useEffect(() => {
        if (dialog?.isOpen) {
            const onKey = (e: KeyboardEvent) => {
                if (e.key === "Escape") {
                    handleClose();
                }
            };
            document.addEventListener("keydown", onKey);
            return () => {
                document.removeEventListener("keydown", onKey);
            };
        }
    }, [dialog?.isOpen]);

    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setIsClosing(false);
            closeDialog(0);
        }, 200);
    };

    const handleButtonClick = (index: number) => {
        setIsClosing(true);
        setTimeout(() => {
            setIsClosing(false);
            closeDialog(index);
        }, 200);
    };

    if (!dialog?.isOpen) {
        return null;
    }

    return (
        <div
            role="dialog"
            aria-modal={true}
            className="fixed inset-0 z-[100] flex items-center justify-center animate-fadeIn"
            onClick={(e) => {
                if (e.target === e.currentTarget) {
                    handleClose();
                }
            }}
        >
            {/* Backdrop */}
            <div className="absolute inset-0 bg-black/80" />

            {/* Dialog */}
            <div className="relative z-10">
                <Dialog
                    type={dialog.type}
                    title={dialog.title}
                    message={dialog.message}
                    buttons={dialog.buttons || [{ label: "OK", variant: "primary" }]}
                    isClosing={isClosing}
                    onButtonClick={handleButtonClick}
                    onClose={handleClose}
                />
            </div>
        </div>
    );
}
