<!--
Dosya Yolu: REPO_ROOT/AGENTS.md
Amac: Fork gelistirme kurallari ve proje sinirlarini belirler
Modul - Markdown
Version: 0.1.0
Aciklama: Upstream uyumu ve merkezi Burak Dev Standard sozlesmesi
Bagimli Oldugu Katman: Documentation
-->
# Suwayomi fork agent rules

Central standard: BURAK DEV STANDARD v3.0.0, pinned to TurkuazSoft/DevStandards
commit 257d0470460cdfce14f11da039ee2aa3a500755b.
https://github.com/TurkuazSoft/DevStandards

Read the central AGENTS.md, standards/00-core.md, architecture.md, security.md,
testing-ci.md and portable-paths.md plus applicable platform rules before changes.

- Upstream is Suwayomi/Suwayomi-Server, base branch master.
- Custom branch is develop/burak-manga. Never merge into master or create a release without user approval.
- Keep upstream runtime and release workflows untouched for local archive-only features.
- deploy/burak-manga is a separate local archive integration. Docker Compose currently runs
  the official upstream stable image, NOT an image compiled from this fork.
- Never commit downloaded manga, runtime data, passwords, tokens, or site credentials.
- Treat websites and third-party extensions as untrusted. Do not bypass access controls,
  CAPTCHA, DRM or other restrictions. Check permissions before downloading content.
- Run focused local tests first, then use GitHub CI as needed. Never report skipped
  Docker/runtime, application smoke, or external integration checks as passed.
- Preserve existing working behavior, real project paths and documentation.
