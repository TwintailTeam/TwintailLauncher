let translations: Record<string, any> = {};
let fallback: Record<string, any> = {};

export function setTranslations(t: Record<string, any>, fb: Record<string, any>) {
    const { metadata: _t, ...rest } = t;
    const { metadata: _fb, ...fbRest } = fb;
    translations = rest;
    fallback = fbRest;
}

function getNestedValue(obj: Record<string, any>, key: string): string | undefined {
    if (typeof obj[key] === "string") return obj[key];
    const dotIndex = key.indexOf(".");
    if (dotIndex === -1) return undefined;
    const section = key.substring(0, dotIndex);
    const subKey = key.substring(dotIndex + 1);
    const sectionObj = obj[section];
    if (sectionObj == null || typeof sectionObj !== "object") return undefined;
    if (typeof sectionObj[subKey] === "string") return sectionObj[subKey];
    return getNestedValue(sectionObj, subKey);
}

export function translate(key: string, vars?: Record<string, string>): string {
    const value = getNestedValue(translations, key) ?? getNestedValue(fallback, key);
    if (value == null) return key;
    return vars ? value.replace(/\{(\w+)}/g, (_: string, name: string) => vars[name] ?? `{${name}}`) : value;
}
