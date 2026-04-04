import React, { createContext, useContext, useState, useCallback, useEffect } from "react";

export type DialogType = "error" | "warning" | "info" | "confirm";

export interface DialogButton {
    label: string;
    variant?: "primary" | "secondary" | "danger";
    onClick?: () => void;
    preventClose?: boolean;
}

export interface DialogOptions {
    type: DialogType;
    title: string;
    message: string;
    buttons?: DialogButton[];
    onClose?: (buttonIndex: number) => void;
}

interface DialogState extends DialogOptions {
    isOpen: boolean;
}

interface DialogContextType {
    dialog: DialogState | null;
    showDialog: (options: DialogOptions) => void;
    closeDialog: (buttonIndex?: number) => void;
}

const DialogContext = createContext<DialogContextType | undefined>(undefined);

// Global reference for showing/closing dialogs from outside React components
let globalShowDialog: ((options: DialogOptions) => void) | null = null;
let globalCloseDialog: ((buttonIndex?: number) => void) | null = null;

/**
 * Check if the dialog provider is ready
 */
export function isDialogReady(): boolean {
    return globalShowDialog !== null;
}

/**
 * Wait for the dialog provider to be ready (with timeout)
 */
export async function waitForDialogReady(timeoutMs: number = 5000): Promise<boolean> {
    const startTime = Date.now();
    while (!globalShowDialog && Date.now() - startTime < timeoutMs) {
        await new Promise(resolve => setTimeout(resolve, 50));
    }
    return globalShowDialog !== null;
}

/**
 * Show a dialog and return a promise that resolves with the button index clicked.
 * Can be called from anywhere (services, loaders, etc.)
 * Will wait for dialog provider to be ready before showing.
 */
export async function showDialogAsync(options: Omit<DialogOptions, 'onClose'>): Promise<number> {
    // Wait for dialog provider to be ready
    const ready = await waitForDialogReady();
    if (!ready) {
        console.error("Dialog provider not ready after timeout");
        return -1; // Return -1 to indicate failure (not a valid button index)
    }

    return new Promise((resolve) => {
        globalShowDialog!({
            ...options,
            onClose: (buttonIndex) => resolve(buttonIndex),
        });
    });
}

/**
 * Programmatically close the currently open dialog from outside React components.
 */
export function closeCurrentDialog(buttonIndex: number = 0) {
    if (globalCloseDialog) globalCloseDialog(buttonIndex);
}

export function DialogProvider({ children }: { children: React.ReactNode }) {
    const [dialog, setDialog] = useState<DialogState | null>(null);

    const showDialog = useCallback((options: DialogOptions) => {
        // Default buttons based on type if not provided
        const defaultButtons: DialogButton[] =
            options.type === "confirm"
                ? [
                    { label: "Cancel", variant: "secondary" },
                    { label: "OK", variant: "primary" },
                ]
                : [{ label: "OK", variant: "primary" }];

        setDialog({
            ...options,
            buttons: options.buttons || defaultButtons,
            isOpen: true,
        });
    }, []);

    const closeDialog = useCallback((buttonIndex: number = 0) => {
        setDialog((prev) => {
            if (prev?.onClose) {
                prev.onClose(buttonIndex);
            }
            return null;
        });
    }, []);

    // Set global reference for use outside React components
    useEffect(() => {
        globalShowDialog = showDialog;
        globalCloseDialog = closeDialog;
        return () => {
            globalShowDialog = null;
            globalCloseDialog = null;
        };
    }, [showDialog, closeDialog]);

    return (
        <DialogContext.Provider value={{ dialog, showDialog, closeDialog }}>
            {children}
        </DialogContext.Provider>
    );
}

export function useDialog() {
    const context = useContext(DialogContext);
    if (context === undefined) {
        throw new Error("useDialog must be used within a DialogProvider");
    }
    return context;
}
