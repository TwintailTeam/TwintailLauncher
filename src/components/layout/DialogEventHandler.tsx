import { useEffect, useState } from "react";
import { useDialog, DialogType } from "../../context/DialogContext";
import {
    registerDialogListener,
    DialogPayload,
    emitDialogResponse,
} from "../../services/dialogEvents";

const DIALOG_TYPES: DialogType[] = ["info", "warning", "error", "confirm"];

/**
 * Component that registers the Tauri event listener for dialogs from Rust.
 * Must be mounted inside DialogProvider.
 */
export default function DialogEventHandler() {
    const { showDialog } = useDialog();
    const [testTypeIndex, setTestTypeIndex] = useState(0);

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const register = async () => {
            unlisten = await registerDialogListener((payload: DialogPayload) => {
                const callbackId = payload.callback_id;
                const parsedButtons: string[] = (() => {
                    if (Array.isArray(payload.buttons)) { return payload.buttons; }
                    if (typeof payload.buttons === "string") {
                        try {
                            const parsed = JSON.parse(payload.buttons);
                            if (Array.isArray(parsed)) { return parsed.map((label) => String(label)); }
                        } catch {}
                        return [payload.buttons];
                    }
                    return ["OK"];
                })();

                // Map Rust payload to React dialog options
                const buttons = parsedButtons.map((label: string, index: number) => ({
                    label,
                    variant: index === parsedButtons.length - 1 ? ("primary" as const) : ("secondary" as const),
                    onClick: callbackId ? () => emitDialogResponse(callbackId, index) : undefined,
                }));
                showDialog({type: payload.dialog_type, title: payload.title, message: payload.message, buttons, onClose: callbackId ? (buttonIndex) => emitDialogResponse(callbackId, buttonIndex) : undefined,});
            });
        };

        register();
        return () => {
            if (unlisten) {unlisten();}
        };
    }, [showDialog]);

    // DEV: Ctrl+Shift+D to trigger test dialog
    useEffect(() => {
        const showTestDialog = (typeIndex: number) => {
            const type = DIALOG_TYPES[typeIndex];
            showDialog({
                type,
                title: `Test Dialog (${type})`,
                message: `This is a ${type.toUpperCase()} dialog.\n\nPress "Next Type" to see the next style, or Ctrl+Shift+D again.`,
                buttons: [
                    { label: "Close", variant: "secondary" },
                    {
                        label: "Next Type →",
                        variant: "primary",
                        preventClose: true,
                        onClick: () => {
                            const nextIndex = (typeIndex + 1) % DIALOG_TYPES.length;
                            setTestTypeIndex(nextIndex);
                            setTimeout(() => showTestDialog(nextIndex), 100);
                        }
                    }
                ],
            });
        };

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.ctrlKey && e.shiftKey && e.key === "D") {
                e.preventDefault();
                showTestDialog(testTypeIndex);
            }
        };
        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [showDialog, testTypeIndex]);

    return null;
}
