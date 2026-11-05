// Dosya Yolu: REPO_ROOT/tools/webui-tr-preview/enable-turkish.mjs
// Amac: WebUI Turkce PO katalogunu ayri bir test derlemesinde etkinlestirir
// Modul - JavaScript
// Version: 0.1.0
// Aciklama: Pinlenmis upstream kaynakta dar ve guvenli degisiklik
// Bagimli Oldugu Katman: Tool
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { TURKISH_SUGGESTIONS } from './tr-suggestions.mjs';

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
    const trPo = readFileSync(turkish, 'utf8');
    const enPo = readFileSync(english, 'utf8');
    if (!trPo.includes('Language: tr')) throw new Error('TURKISH_PO_INVALID');
    if (!enPo.includes('msgid "About"')) throw new Error('ENGLISH_PO_INVALID');

    const keys = new Set();
    const additions = [];
    for (const [source, translation] of TURKISH_SUGGESTIONS) {
        if (keys.has(source) || !source || !translation || /[\n\r{}]/.test(source)) {
            throw new Error('INVALID_TRANSLATION_SUGGESTION: ' + source);
        }
        keys.add(source);
        const entry = 'msgid ' + JSON.stringify(source) + '\n';
        if (!enPo.includes(entry)) throw new Error('SUGGESTION_SOURCE_NOT_FOUND: ' + source);
        if (!trPo.includes(entry)) {
            additions.push(entry + 'msgstr ' + JSON.stringify(translation) + '\n');
        }
    }
    for (const item of plan) {
        if (item.changed) writeFileSync(item.path, item.modified, 'utf8');
    }
    if (additions.length) {
        writeFileSync(turkish, trPo.trimEnd() + '\n\n' + additions.join('\n') + '\n', 'utf8');
    }
    console.log('Turkish suggestions appended:', additions.length);
    return plan.filter((x) => x.changed).length;
}
if (process.argv[1] && resolve(process.argv[1]) === resolve(import.meta.filename)) {
    const count = applyTurkishPreview(process.argv[2] || 'webui');
    console.log('Turkish preview registered; files changed:', count);
}
