<!--
Dosya Yolu: REPO_ROOT/deploy/burak-manga/docs/decisions/0001-suwayomi.md
Amac: Manga arsivi mimari karari
Modul - Markdown
Version: 0.1.0
Aciklama: Suwayomi temeli, kaynak adaylari ve riskler
Bagimli Oldugu Katman: Documentation
-->
# ADR-0001: Suwayomi for the local archive

Date: 2026-10-21
Status: Accepted for local prototype

Decision: Use Suwayomi rather than MediaWiki, Komga, or Kavita for the initial
Docker-based local manga library. Put our integration in deploy/burak-manga
without modifying upstream server logic.

Important distinction: the compose file launches an upstream stable image.
It does not run a custom build of our fork.

The Keiyoushi extension-source repository has code for Ruya Manga (target
www.ruyamanga2.com) and Merlin Scans (target merlintoon.com). The requested
MerlinScans domain is merlinscans.org; compatibility with that domain is
unverified. Neither extension has been runtime-tested here.

Only permitted local source images may be imported into CBZ. Adding external
source integrations requires explicit source authorization, compatibility
testing, and a new focused change. No access-control bypass.
