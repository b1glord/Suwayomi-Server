<!--
Dosya Yolu: REPO_ROOT/tools/webui-tr-preview/README.md
Amac: Turkce WebUI test derlemesinin kullanimi ve upstream katki yolu
Modul - Markdown
Version: 0.1.0
Aciklama: WebUI derleme test artifacti; kalici ceviri Weblate akisidir
Bagimli Oldugu Katman: Documentation
-->

# Suwayomi-WebUI Turkce test derlemesi

Bu dal sunucu manga gelistirme dalindan bagimsizdir.
Upstream WebUI kaynagi 020a274e8a35a45195f69210ccac2f3e18502749
commit'inden sabit olarak alinir.

WebUI src/i18n/locales/tr.po dosyasi halihazirda Turkce ceviriler
icerir. Fakat desteklenen diller listesi Weblate'deki %75 tamamlama
esigine gore otomatik olusur. Bu test surumu yalnizca dili listede
gorunur hale getirir. Eksik ceviriler kalabilir; tam Turkcelestirme
iddiasi yoktur.

## Isleyis

- Node birim testleri format degisimi, bos katalog ve tekrar uygulamayi denetler.
- i18nResources ve Lingui config icine tr kodu eklenir.
- upstream PO dosyalarinin icerigi degistirilmez.
- Pinlenmis upstream WebUI derlenir; ZIP ve SHA256 dosyasi CI'da uretilir.
- GitHub Actions artifact; production release ve merge degildir.
- Test ZIP mevcut Docker'a ya da storage/Tachidesk dosyalarina otomatik
  kopyalanmaz; eski arayuzu yedekleyip WebUI surum uyumlulugunu
  dogrulamadan kuruluma gecilmez.

## Resmi ceviri yolu

Turkce ceviriyi resmi olarak tamamlamak icin Weblate kullanilir:
https://hosted.weblate.org/projects/suwayomi/suwayomi-webui/tr/

Weblate cevirileri otomatik olarak upstream PR akisina tasir.
%75 esigine ulasinca upstream resources olusturma islemi tr dilini
normal build'e otomatik ekleyebilir. Bu testteki zoraki dil kaydi
upstream'e PR olarak gonderilmemelidir.

Cloudflare sorunu, manga indirme kuyrugu, eklenti sertifikalari
ve sunucu otomatik guncellemeleri bu testin kapsami disindadir.
