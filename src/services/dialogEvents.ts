import { listen, UnlistenFn, emit } from "@tauri-apps/api/event";

export interface DialogPayload {
    dialog_type: "error" | "warning" | "info" | "confirm";
    title: string;
    message: string;
    buttons?: string[] | string;
    callback_id?: string;
}

export type ShowDialogFn = (payload: DialogPayload) => void;

/**
 * Registers a listener for the 'show_dialog' event emitted from Rust.
 * Returns an unlisten function to clean up the listener.
 */
export async function registerDialogListener(showDialog: ShowDialogFn): Promise<UnlistenFn> {
    return await listen<DialogPayload>("show_dialog", (event) => {showDialog(event.payload);});
}

/**
 * Emits a dialog response back to Rust when a button is clicked.
 * Only emits if a callback_id was provided in the original dialog.
 */
export function emitDialogResponse(callbackId: string, buttonIndex: number): void {
    emit("dialog_response", { callback_id: callbackId, button_index: buttonIndex });
}
