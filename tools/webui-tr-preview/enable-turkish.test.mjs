// Dosya Yolu: REPO_ROOT/tools/webui-tr-preview/enable-turkish.test.mjs
// Amac: Turkce kayit ekleme degisikliklerini ag baglantisiz test eder
// Modul - JavaScript
// Version: 0.1.0
// Aciklama: Idempotence, kaynak kaymasi ve eksik dosya negatif testleri
// Bagimli Oldugu Katman: Testing
import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { applyTurkishPreview } from './enable-turkish.mjs';

function fixture() {
    const root = mkdtempSync(join(tmpdir(), 'suwayomi-tr-'));
    const i18n = join(root, 'src/i18n');
    mkdirSync(join(i18n, 'locales'), { recursive: true });
    writeFileSync(join(i18n, 'index.ts'), "export const i18nResources = [\n    'ru',\n    'vi',\n] as const;\n");
    writeFileSync(join(root, 'lingui.config.ts'), "locales: ['ru', 'vi', 'zh-Hans'],\n");
    writeFileSync(join(i18n, 'locales/tr.po'), 'msgid "About"\nmsgstr "Hakkında"\n"Language: tr"\n');
    writeFileSync(join(i18n, 'locales/en.po'), 'msgid "About"\nmsgstr "About"\n');
    return root;
}
test('registers Turkish without deleting other locales and is idempotent', () => {
    const root = fixture();
    try {
        assert.equal(applyTurkishPreview(root), 2);
        assert.equal(applyTurkishPreview(root), 0);
        assert.match(readFileSync(join(root, 'src/i18n/index.ts'), 'utf8'), /'ru',\n    'tr',\n    'vi'/);
    } finally { rmSync(root, { recursive: true, force: true }); }
});
test('upstream format drift fails before modifying files', () => {
    const root = fixture();
    try {
        writeFileSync(join(root, 'lingui.config.ts'), "locales: ['ru', 'xx', 'vi'],\n");
        assert.throws(() => applyTurkishPreview(root), /UNEXPECTED_UPSTREAM_LOCALE_FORMAT/);
        assert.doesNotMatch(readFileSync(join(root, 'src/i18n/index.ts'), 'utf8'), /'tr'/);
    } finally { rmSync(root, { recursive: true, force: true }); }
});
test('missing translation catalog fails', () => {
    const root = fixture();
    try {
        rmSync(join(root, 'src/i18n/locales/tr.po'));
        assert.throws(() => applyTurkishPreview(root), /WEBUI_SOURCE_MISSING/);
    } finally { rmSync(root, { recursive: true, force: true }); }
});
