import { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';

export default function TTLVersion() {
    const [version, setVersion] = useState('');
    const branch = import.meta.env.MODE || "unknown";
    useEffect(() => {
        getVersion().then(setVersion);
    }, []);
    return (
        <>
            Version: <span className={"text-purple-600 font-bold"}>{version}</span> | Branch: <span className={"text-orange-600 font-bold"}>{branch}</span>
        </>
    );
}