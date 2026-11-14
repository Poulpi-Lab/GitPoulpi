[English](../README.md) · [Čeština](../cs/README.md) · **Deutsch** · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# GitPoulpi-Benutzerdokumentation

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="Der GitPoulpi-Oktopus" width="128"></p>

GitPoulpi ist ein Git-Client für das Terminal mit anklickbarem Commit-Graphen,
Befehlspalette, Anbindung an Hosting-Dienste und Issue-Tracker, optionaler
KI-Hilfe sowie einer CLI und einem MCP-Server, die alles tun, was die
Oberfläche kann. Diese Seiten stehen in Lesereihenfolge: Beginnen Sie oben und
springen Sie zu den Referenzseiten, wenn Sie sie brauchen.

![Das GitPoulpi-Hauptfenster](../images/hero.png)

## Hier anfangen

1. **[Installation](installation.md)**: was Sie brauchen, Release-Archive,
   Paketmanager, Bauen aus dem Quellcode. Für eine kleinere Binärdatei siehe
   **[Eigene Builds](custom-builds.md)**. Wie GitPoulpi sich selbst aktuell hält
   (der Versions-Chip, das Update-Fenster, Zurückgehen): **[Updates](updates.md)**.
   Zum Betrieb in einem Container: **[In Docker ausführen](docker.md)**. Für
   Administratoren, interne Feeds, Spiegel und die Update-Richtlinie:
   **[Unternehmensverteilung](enterprise-distribution.md)**.
2. **[Erste Schritte](getting-started.md)**: Ihre ersten fünf Minuten, vom
   Start bis zum ersten Commit.
3. **[Rundgang durch die Oberfläche](interface-tour.md)**: der Name jedes
   Bereichs des Fensters, mit Bildern.

## Täglicher Gebrauch

4. **[Mit dem Log arbeiten](log-and-history.md)**: den Graphen lesen, Suche,
   Branches, Tags, Stashes, Worktrees, Refs ziehen, Rebase, Cherry-Pick,
   Reword und Amend, Banner, Pull und Push, Auto-Stash.
5. **[Staging und Committen](staging-and-committing.md)**: die Änderungen,
   Diffs, Hunks und einzelne Zeilen stagen, Commits schreiben, der
   Commit-Composer, Blame, Dateiverlauf und der eingebaute Editor.
6. **[Bilder](image-view.md)**: Bilder in der Diff-Ansicht und der
   Dateiansicht, vorher und nachher, Zoom, Bearbeiten mit.
7. **[Merge-Konflikte lösen](merge-conflicts.md)**: der Merge-Editor (A und B
   nebeneinander über der Ausgabe), Zeilen und Blöcke auswählen, Karten für
   binäre, LFS-, gelöschte, Symlink- und Submodul-Konflikte, und wie ein Agent
   eine Lösung vorschlägt.
8. **[Rückgängig und Wiederholen](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, was
   sich rückgängig machen lässt und was nicht, wann es ablehnt oder nachfragt,
   wie weit es zurückgeht, es abschalten.
9. **[Git LFS](lfs.md)**: große Dateien, Tracking, Sperren, und was ohne
   installiertes `git-lfs` passiert.
10. **[Submodule](submodules.md)**: der Abschnitt SUBMODULES, ihre
    Aktualisierung, und sie mit Checkouts und Pulls synchron halten.
11. **[Designs](themes.md)**: die eingebauten Designs, jedes Design mit jeder
    Spurpalette, Live-Vorschau, eigene Design-Dateien, `gitpoulpi theme`.
12. **[Workspaces](workspaces.md)**: benannte Gruppen von Repositories mit
    eigenen Tabs und einem optionalen Standard-Commit-Profil, der Chip in der
    Tab-Leiste, der Tab Workspaces, Alle abrufen, Alle holen,
    Fehlende klonen.
13. **[Suche](search.md)**: eine Fuzzy-Suche (`Ctrl+Alt+F`) über Branches, Tags,
    Worktrees, Stashes, Commits, Pull/Merge Requests und Issues aller
    Repositories des Workspace oder der offenen Tabs.
14. **[Das Radar](radar.md)**: die Pull und Merge Requests, Issues und
    laufende Arbeit eines Workspace (oder Ihrer offenen Tabs) in einer Liste:
    Filter, Anheften, Zurückstellen, und was jede Zeile kann.
    Eine dieser Anfragen in der App lesen und prüfen, mit Kommentar-Threads,
    einem ausstehenden Review und Genehmigen / Änderungen anfordern:
    **[Einen Pull Request prüfen](review.md)**.
15. **[Öffnen in](open-in.md)**: die Liste der Titelleiste, die den Ordner des
    aktiven Tabs in einem Terminal, Multiplexer, Editor oder Dateimanager
    öffnet.
16. **[Dem Ordner des aktiven Tabs folgen](following-the-tab-folder.md)**:
    „Beenden und hier cd“, und der Wrapper `gitpoulpi shell-init` (`gp`) für
    bash, zsh, fish und PowerShell.
    Welches Terminal GitPoulpi am besten darstellt: **[Ein Terminal wählen](terminals.md)**.
    Was durch tmux, zellij, screen, herdr und Terminal-Panes hindurchkommt:
    **[Multiplexer](multiplexers.md)**.

Außerdem: **[Worktrees](worktrees.md)**: verknüpfte Worktrees erstellen, öffnen, zusammenführen und entfernen.

Außerdem: **[Mehrere Remotes für einen Branch](multiple-remotes.md)**: Push-Ziele und Pull-
Quellen, der Remotes-Dialog, Push an alle oder an ausgewählte Remotes, ein Remote auflösen,
das sich bewegt hat, Fortsetzen nach einem Konflikt, die CLI und ein Alias für reines git.

**[Branching-Workflows](branching-workflows.md)**, die Vorlagen Git Flow,
GitHub Flow, GitLab Flow und Trunk-based oder Ihre eigenen Basen, Branch-Namensmuster
und Topic-Typen (`gitpoulpi flow ...`).

**[Verlauf umschreiben](history-rewrite.md)**: die Werkbank (`Ctrl+G Shift+H`), die
Autoren, Daten, Nachrichten und Dateiinhalte über viele Commits hinweg umschreibt, mit
Vorschau, Lua-Skripten und einem einzigen Rückgängig.

## Tastatur und Maus

17. **[Tastenkürzel](keybindings.md)**: jede Taste, nach Bereich, und jeder
    Palettenbefehl. In der App zeigt `?` dieselbe Liste und `Ctrl+P` findet
    jeden Befehl.
18. **[Maus und Gesten](mouse-and-gestures.md)**: klicken, ziehen, markieren
    und kopieren, Bildlaufleisten und Hover-Rückmeldung.

## Einstellungen und Integrationen

19. **[Einstellungen](settings.md)**: jede Seite und Option des
    Einstellungsbildschirms, das Suchfeld, Profile, und wie Sie Tastenkürzel
    neu belegen.
20. **[KI-Unterstützung](ai.md)**: GitPoulpis eigene KI (standardmäßig aus):
    ein lokaler Agent wie `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama
    oder LM Studio; Anweisungen; Commit-Nachrichten, Erklärungen,
    Pull-Request-Beschreibungen und der Commit-Composer; Datenschutz und
    Fehlerbehebung.
21. **[Hosting und Issue-Tracker](hosting-integration.md)**: GitHub, GitLab, Gitea und Forgejo,
    Azure DevOps, Bitbucket, Gerrit und die Issue-Tracker:
    Tokens und ihre Berechtigungen,
    verbinden, ein Repository verknüpfen, Pull Requests und Issues,
    Referenzen in Commit-Nachrichten, SSH-Schlüssel, bekannte Grenzen. Dazu
    **[Lokale Issues und Obsidian](local-issues.md)**: ein Ordner mit Markdown-
    Notizen als Issue-Quelle, ohne Token, mit einer Obsidian Base.

## Automatisierung

22. **[CLI-Referenz](cli-reference.md)**: jeder `gitpoulpi`-Befehl, mit
    `--json` für Skripte.
23. **[MCP und Agenten](mcp-and-agents.md)**: einen externen Agenten (Claude
    Code, Cursor, Codex, ...) GitPoulpi bedienen lassen: den Server
    registrieren, `.mcp.json`, die Berechtigungsschalter und ihre
    Standardwerte, jedes Tool, und wie ein Agent Ihnen Dinge in Ihrem Fenster
    zeigt. Der fertige Agenten-Skill ist
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referenz und Hilfe

24. **[Konfigurationsreferenz](config-reference.md)**: jeder Schlüssel der
    `config.toml`, sein Standardwert und seine Wirkung, die Einstellungen pro
    Repository, Umgebungsvariablen.
25. **[Fehlerbehebung und FAQ](troubleshooting.md)**: Symbole, Graph-Darstellung,
    Tasten, die nicht ankommen, hängende Daemons, Geschwindigkeit,
    LFS-Zeiger.

## Screenshots

Jedes Bild in diesen Seiten ist eine echte Aufnahme eines Release-Builds, auf
einem kleinen Demo-Repository mit erfundenen Autoren. Die Bilder liegen in
[`images/`](../images/). `scripts/docs-screenshots.sh` baut die
Demo-Repositories und nimmt sie neu auf. Sie entstehen im Terminal `kitty`,
das den Commit-Graphen und die Autoren-Avatare als Bilder zeichnet. Terminals
ohne Grafikprotokoll zeigen stattdessen einen Zeichen-Graphen.

## Für Mitwirkende

[CLAUDE.md](../../../CLAUDE.md), die [Architekturübersicht](../../architecture/overview.md)
und die [Roadmap](../../roadmap.md) beschreiben den Entwurf und das, was
geplant ist; [CHANGELOG.md](../../../CHANGELOG.md) listet auf, was sich in
jedem Release geändert hat.

---

GitKraken ist eine Marke des jeweiligen Inhabers. GitPoulpi ist ein unabhängiges Projekt, steht in keiner Verbindung zu diesem und wird von ihm weder unterstützt noch gesponsert; der Name wird nur genannt, um Kompatibilität und Inspiration zu beschreiben.
