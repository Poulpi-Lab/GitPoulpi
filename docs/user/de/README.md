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

## Eine Funktion finden

| Ich möchte... | Seite | So erreichen Sie es in der App |
|---|---|---|
| An zwei Branches gleichzeitig arbeiten | [Worktrees](worktrees.md) | Der Abschnitt **WORKTREES** der Refs-Seitenleiste (`+` erstellt einen; ausgeblendet, bis es einen gibt: Palette **Seitenleiste: Abschnitt Worktrees anzeigen**); Palette **Refs: Worktree erstellen...** |
| Mit verschachtelten Repositories arbeiten | [Submodule](submodules.md) | Der Abschnitt **SUBMODULE** der Refs-Seitenleiste; Palette **Git: Alle Submodule aktualisieren**, **Refs: Submodul aktualisieren (ausgewählt)** |
| Einen Merge-Konflikt lösen | [Merge-Konflikte lösen](merge-conflicts.md) | `Enter` auf einer `!`-Zeile des Status-Bereichs, die Schaltfläche **Konflikte lösen...** des Banners oder `Shift+E` in der Diff-Ansicht; Palette **Konflikte: Merge-Editor öffnen** |
| Einen Pull oder Merge Request prüfen | [Einen Pull oder Merge Request prüfen](review.md) | `Enter` auf einer Zeile des Abschnitts **PULL REQUESTS**, `e` auf einer Radar-Zeile, `Alt+R` in der Workspace-Suche; Palette **Hosting: Anfrage prüfen...** |
| Pull Requests und Issues durchsuchen, erstellen und verwalten | [Pull Requests und Issues](pull-requests-and-issues.md) | Die Abschnitte **PULL REQUESTS** und **ISSUES** der Refs-Seitenleiste (`n` erstellt einen); Palette **Pull Requests: Pull Request erstellen...**, **Hosting: Neues Issue...**. Verbinden Sie zuerst ein Konto: [Hosting und Issue-Tracker](hosting-integration.md) |
| Die Diskussion eines Issues oder Requests lesen und beantworten | [Unterhaltungen](conversations.md) | `Enter` auf einer Issue-Zeile, einer Board-Karte oder einer Radar-Zeile; Palette **Hosting: Issue-Unterhaltung öffnen...** |
| Meine Issues als Board sehen | [Das Issue-Board](pull-requests-and-issues.md#das-issue-board) | Die Schaltfläche **Board** in der Kopfzeile von ISSUES; Palette **Issues: Board öffnen** |
| Issues als Markdown-Notizen führen, ohne Token | [Lokale Issues und Obsidian](local-issues.md) | Der Abschnitt ISSUES, Anbieter `Local Markdown - <Name>`; Palette **Lokale Issues: In Obsidian öffnen...** |
| Autoren, Daten, Nachrichten oder Dateien über viele Commits umschreiben | [Verlauf umschreiben](history-rewrite.md) | `Ctrl+G Shift+H`; Palette **Umschreiben: Werkbank für den Verlauf** |
| Einen Vorgang rückgängig machen (oder wiederholen) | [Rückgängig und Wiederholen](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` öffnet den Rückgängig-Verlauf; Palette **Bearbeiten: Rückgängig** |
| Große Dateien mit Git LFS verfolgen | [Git LFS](lfs.md) | Palette **LFS: Muster verfolgen...**, **LFS: Für dieses Repository initialisieren** |
| Einen Branch, Commit, Pull Request oder ein Issue über Repositories hinweg finden | [Suche](search.md) | `Ctrl+Alt+F` oder die Schaltfläche Globale Suche in der Titelleiste; `f` durchsucht den Commit-Graphen des aktuellen Repositorys |
| Alles, was mich braucht, in einer Liste sehen | [Das Radar](radar.md) | `Alt+R`; Palette **Radar: Öffnen** |
| Repositories gruppieren und gemeinsam abrufen oder holen | [Workspaces](workspaces.md) | `Alt+W` wechselt, `Alt+J` öffnet den Tab Workspaces; Palette **Workspace: Verwalten...** |
| KI-Hilfe bekommen (Commit-Nachrichten, Erklärungen, Beschreibungen) | [KI-Unterstützung](ai.md) | `Ctrl+,` > **KI** > **KI aktivieren**, dann schreibt `Ctrl+G i` eine Commit-Nachricht; Palette **KI: ...** |
| Einen beliebigen Befehl oder eine Taste finden | [Tastenkürzel](keybindings.md) | `?` in der App listet die Tasten, `Ctrl+P` findet jeden Befehl |

## Täglicher Gebrauch

4. **[Mit dem Log arbeiten](log-and-history.md)**: den Graphen lesen, Suche,
   Branches, Tags, Stashes, Refs ziehen, Rebase, Cherry-Pick,
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
8. **[Worktrees](worktrees.md)**: ein zweiter ausgecheckter Ordner auf einem
   anderen Branch: der Abschnitt WORKTREES, verknüpfte Worktrees erstellen,
   öffnen, zusammenführen und entfernen.
9. **[Submodule](submodules.md)**: der Abschnitt SUBMODULE, ihre
   Aktualisierung, und sie mit Checkouts und Pulls synchron halten.
10. **[Rückgängig und Wiederholen](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, was
    sich rückgängig machen lässt und was nicht, wann es ablehnt oder nachfragt,
    wie weit es zurückgeht, es abschalten.
11. **[Git LFS](lfs.md)**: große Dateien, Tracking, Sperren, und was ohne
    installiertes `git-lfs` passiert.
12. **[Designs](themes.md)**: die eingebauten Designs, jedes Design mit jeder
    Spurpalette, Live-Vorschau, eigene Design-Dateien, `gitpoulpi theme`.
13. **[Workspaces](workspaces.md)**: benannte Gruppen von Repositories mit
    eigenen Tabs und einem optionalen Standard-Commit-Profil, der Chip in der
    Tab-Leiste, der Tab Workspaces, Alle abrufen, Alle holen,
    Fehlende klonen.
14. **[Suche](search.md)**: eine Fuzzy-Suche (`Ctrl+Alt+F`) über Branches, Tags,
    Worktrees, Stashes, Commits, Pull/Merge Requests und Issues aller
    Repositories des Workspace oder der offenen Tabs.
15. **[Das Radar](radar.md)**: die Pull und Merge Requests, Issues und
    laufende Arbeit eines Workspace (oder Ihrer offenen Tabs) in einer Liste:
    Filter, Anheften, Zurückstellen, und was jede Zeile kann.
16. **[Pull Requests und Issues](pull-requests-and-issues.md)**: die Abschnitte
    PULL REQUESTS und ISSUES der Seitenleiste, das Issue-Board, einen Pull
    Request oder ein Issue erstellen, der Chip der Symbolleiste, Referenzen in
    Commit-Nachrichten.
17. **[Unterhaltungen](conversations.md)**: die Diskussion eines Issues oder
    Requests im Fenster lesen und beantworten, Senden als, was jeder Host nicht
    kann.
18. **[Einen Pull Request prüfen](review.md)**: die Review-Ansicht mit
    Kommentar-Threads, einem ausstehenden Review und Genehmigen / Änderungen
    anfordern.
19. **[Öffnen in](open-in.md)**: die Liste der Titelleiste, die den Ordner des
    aktiven Tabs in einem Terminal, Multiplexer, Editor oder Dateimanager
    öffnet.
20. **[Dem Ordner des aktiven Tabs folgen](following-the-tab-folder.md)**:
    „Beenden und hier cd“, und der Wrapper `gitpoulpi shell-init` (`gp`) für
    bash, zsh, fish und PowerShell.
    Welches Terminal GitPoulpi am besten darstellt: **[Ein Terminal wählen](terminals.md)**.
    Was durch tmux, zellij, screen, herdr und Terminal-Panes hindurchkommt:
    **[Multiplexer](multiplexers.md)**.

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

21. **[Tastenkürzel](keybindings.md)**: jede Taste, nach Bereich, und jeder
    Palettenbefehl. In der App zeigt `?` dieselbe Liste und `Ctrl+P` findet
    jeden Befehl.
22. **[Maus und Gesten](mouse-and-gestures.md)**: klicken, ziehen, markieren
    und kopieren, Bildlaufleisten und Hover-Rückmeldung.

## Einstellungen und Integrationen

23. **[Einstellungen](settings.md)**: jede Seite und Option des
    Einstellungsbildschirms, das Suchfeld, Profile, und wie Sie Tastenkürzel
    neu belegen.
24. **[KI-Unterstützung](ai.md)**: GitPoulpis eigene KI (standardmäßig aus):
    ein lokaler Agent wie `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama
    oder LM Studio; Anweisungen; Commit-Nachrichten, Erklärungen,
    Pull-Request-Beschreibungen und der Commit-Composer; Datenschutz und
    Fehlerbehebung.
25. **[Hosting und Issue-Tracker](hosting-integration.md)**: GitHub, GitLab, Gitea und Forgejo,
    Azure DevOps, Bitbucket, Gerrit und die Issue-Tracker:
    Tokens und ihre Berechtigungen,
    verbinden, ein Repository verknüpfen, SSH-Schlüssel, bekannte Grenzen. Dazu
    **[Lokale Issues und Obsidian](local-issues.md)**: ein Ordner mit Markdown-
    Notizen als Issue-Quelle, ohne Token, mit einer Obsidian Base.

## Automatisierung

26. **[CLI-Referenz](cli-reference.md)**: jeder `gitpoulpi`-Befehl, mit
    `--json` für Skripte.
27. **[MCP und Agenten](mcp-and-agents.md)**: einen externen Agenten (Claude
    Code, Cursor, Codex, ...) GitPoulpi bedienen lassen: den Server
    registrieren, `.mcp.json`, die Berechtigungsschalter und ihre
    Standardwerte, jedes Tool, und wie ein Agent Ihnen Dinge in Ihrem Fenster
    zeigt. Der fertige Agenten-Skill ist
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referenz und Hilfe

28. **[Konfigurationsreferenz](config-reference.md)**: jeder Schlüssel der
    `config.toml`, sein Standardwert und seine Wirkung, die Einstellungen pro
    Repository, Umgebungsvariablen.
29. **[Fehlerbehebung und FAQ](troubleshooting.md)**: Symbole, Graph-Darstellung,
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
