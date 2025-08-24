import TextInputPart from "./TextInputPart.tsx";
import {invoke} from "@tauri-apps/api/core";
import HelpTooltip from "./HelpTooltip.tsx";

export default function TextInput({ id, name, value, placeholder, readOnly, pattern, install, fetchInstallSettings, helpText}: { id: string, name: string, value: string, placeholder?: string, readOnly: boolean, pattern?: string, install?: string, helpText: string, fetchInstallSettings?: (id: string) => void }) {

    return (
        <div className="flex w-full items-center gap-4 max-sm:flex-col max-sm:items-stretch">
            <span className="text-white text-sm flex items-center gap-1 w-56 shrink-0 max-sm:w-full">{name}
                <HelpTooltip text={helpText}/>
            </span>
            <div className={"overflow-ellipsis inline-flex flex-row items-center justify-end ml-auto w-[320px]"}>
                <TextInputPart id={id} initalValue={value} placeholder={placeholder} readOnly={readOnly} isPicker={false} pattern={pattern} onChange={(e) => {
                    switch (id) {
                        case "install_env_vars": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_env_vars", {envVars: `${e}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                        case "install_pre_launch_cmd": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_pre_launch_cmd", {cmd: `${e}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                        case "install_launch_cmd": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_launch_cmd", {cmd: `${e}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                        case "install_launch_args": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_launch_args", {args: `${e}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                    }
                }} />
            </div>
        </div>
    )
}
