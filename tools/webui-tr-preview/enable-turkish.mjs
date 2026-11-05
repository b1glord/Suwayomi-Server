// Dosya Yolu: REPO_ROOT/tools/webui-tr-preview/enable-turkish.mjs
// Amac: WebUI Turkce PO katalogunu ayri bir test derlemesinde etkinlestirir
// Modul - JavaScript
// Version: 0.1.0
// Aciklama: Pinlenmis upstream kaynakta dar ve guvenli degisiklik
// Bagimli Oldugu Katman: Tool
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';

export function applyTurkishPreview(webuiDir) {
    const root = resolve(webuiDir);
    const index = join(root, 'src/i18n/index.ts');
    const config = join(root, 'lingui.config.ts');
    const turkish = join(root, 'src/i18n/locales/tr.po');
    const english = join(root, 'src/i18n/locales/en.po');
    for (const needed of [index, config, turkish, english]) {
        if (!existsSync(needed)) throw new Error('WEBUI_SOURCE_MISSING: ' + needed);
    }
    const replacements = [
        { path: index, before: "    'ru',\n    'vi',", after: "    'ru',\n    'tr',\n    'vi'," },
        { path: config, before: "'ru', 'vi',", after: "'ru', 'tr', 'vi'," },
    ];
    const plan = replacements.map(({ path, before, after }) => {
        const original = readFileSync(path, 'utf8');
        const matches = original.split(before).length - 1;
        if (original.includes(after) && matches === 0) return { path, modified: original, changed: false };
        if (matches !== 1) throw new Error('UNEXPECTED_UPSTREAM_LOCALE_FORMAT: ' + path);
        return { path, modified: original.replace(before, after), changed: true };
    });
    if (!readFileSync(turkish, 'utf8').includes('Language: tr')) throw new Error('TURKISH_PO_INVALID');
    if (!readFileSync(english, 'utf8').includes('msgid "About"')) throw new Error('ENGLISH_PO_INVALID');
    for (const item of plan) {
        if (item.changed) writeFileSync(item.path, item.modified, 'utf8');
    }
    return plan.filter((x) => x.changed).length;
}
if (process.argv[1] && resolve(process.argv[1]) === resolve(import.meta.filename)) {
    const count = applyTurkishPreview(process.argv[2] || 'webui');
    console.log('Turkish preview registered; files changed:', count);
}
