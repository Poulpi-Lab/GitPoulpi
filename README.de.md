[English](README.md) · [Čeština](README.cs.md) · **Deutsch** · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="assets/logo/gitpoulpi-512.png" alt="Der GitPoulpi-Oktopus" width="128"></p>

**Ein Git-Client für das Terminal, der sich wie eine Desktop-App anfühlt.**
GitPoulpi bietet einen vollständigen Commit-Graphen, den Sie anklicken, ziehen
und per Rechtsklick bedienen, eine tastaturorientierte Befehlspalette, die jede
Funktion erreicht, die Anbindung an Code-Hosting-Dienste (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; in der Cloud oder selbst gehostet) und Issue-Tracker (Jira, Trello, Linear, YouTrack, Redmine oder ein Ordner mit Markdown-Notizen) sowie dieselben
Operationen als skriptbare CLI und als MCP-Server, damit ein KI-Agent es
genauso bedienen kann wie Sie. GitPoulpi ist von GitKraken inspiriert und
vollständig in Rust neu geschrieben: native Binärdateien, sparsam bei Speicherplatz und Arbeitsspeicher, kein Electron, kein Konto, keine Telemetrie. Es läuft in Ihrem Lieblingsterminal ([welches?](docs/user/de/terminals.md)), denn manche von uns leben darin.

![GitPoulpi zeigt den Commit-Graphen mit Branches, Tags und einem Stash, links die Refs-Seitenleiste, rechts die Änderungen mit dem Commit-Formular](docs/user/images/hero.png)

- **Maus und Tastatur, gleichberechtigt.** Klicken, doppelklicken, rechtsklicken
  und Refs per Drag & Drop auf Commits ziehen; oder `Ctrl+P` drücken und
  tippen, was Sie wollen. Jede Mausgeste hat ihr Tastatur-Gegenstück, und jeder
  Befehl steht in der Palette.
- **Ein echter Commit-Graph.** Branches, Merges, Tags, Stashes und die
  Arbeitskopie in einem scrollenden Log, in Terminals mit Bildunterstützung als
  Pixel gezeichnet, sonst als Zeichen. Viele dunkle und helle Farbdesigns,
  Autoren-Avatare, Suche.
- **Präzise stagen.** Stagen, unstagen oder verwerfen Sie eine Datei, einen
  Ordner, einen Hunk oder eine einzelne Zeile direkt im Diff. Inline oder
  nebeneinander, mit Minimap.
- **Verlauf sicher bearbeiten.** Jeden Commit umformulieren oder ändern
  (Nachricht, Autor, Daten), interaktiv rebasen mit einer Prüfansicht, viele
  Commits auf einmal umschreiben, mit Vorschau und einem einzigen Rückgängig,
  mehrere Commits cherry-picken, Fast-Forward, Worktrees und Patches erstellen.
- **Hosting und Issue-Tracker eingebaut.** Pull und Merge Requests durchsuchen, auschecken und erstellen (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) sowie Gerrit-Changes prüfen. Issues, Work Items und Karten durchsuchen und erstellen (auch Jira, Trello, Linear, YouTrack, Redmine und lokale Markdown-Notizen), auf Wunsch auf einem Kanban-Board. Aus Ihrem Konto klonen und SSH-Schlüssel verwalten.
- **CLI und MCP für Agenten.** `gitpoulpi <Befehl> --json` deckt alles ab, was
  die Oberfläche kann, und `gitpoulpi-mcp` stellt es als MCP-Tools bereit. Ein
  Hintergrund-Daemon pro Repository hält jede Oberfläche aktuell und
  synchron.
- **Optionale KI, nie Pflicht.** Standardmäßig aus und ohne Konto: Bringen Sie
  Ihren eigenen API-Schlüssel oder eine lokale Agenten-CLI wie `claude -p`
  mit. Commit-Nachrichten erzeugen, Änderungen erklären, Pull-Request-
  Beschreibungen schreiben, Ihre Änderungen auf mehrere Commits verteilen und
  Regeln zum Umschreiben des Verlaufs entwerfen.
- **Git LFS, Submodule, Worktrees, Patches, Blame und Dateiverlauf,
  Bild-Diffs, ein eingebauter Editor.**

## Screenshots

| | |
|---|---|
| ![Der Log-Graph mit dem Design Material und aktivierter Zeilenfärbung](docs/user/images/theme-material.png) **Der Commit-Graph**, hier im Design Material mit Zeilenfärbung. | ![Ein Diff einer geänderten Datei mit den Schaltflächen Hunk stagen und Hunk verwerfen an jedem Hunk](docs/user/images/diff.png) **Der Diff**, mit Stagen und Verwerfen pro Hunk oder pro Zeile. |
| ![Die Befehlspalette, gefiltert nach dem Wort stash](docs/user/images/palette.png) **Die Befehlspalette** (`Ctrl+P`) erreicht jeden Befehl. | ![Der Commit-Composer schlägt vier Commits für die Änderungen vor, bereit zur Prüfung](docs/user/images/composer.png) **Der Commit-Composer** lässt von der KI vorgeschlagene Commits prüfen, bevor etwas committet wird. |
| ![Das Rechtsklick-Menü eines Commits mit den Aktionen Cherry-Pick, Revert, Reset, Rebase, Branch und Tag](docs/user/images/context-menu.png) **Rechtsklick** auf alles zeigt seine Aktionen. | ![Die KI-Seite der Einstellungen mit Profilen, Anweisungen pro Aktion und dem Schalter zum Aktivieren](docs/user/images/settings-ai.png) **Die Einstellungen**, mit Suche, einschließlich der optionalen KI-Seite. |

Weitere Bilder mit Erklärungen finden Sie in der [Benutzerdokumentation](docs/user/de/README.md).

## Installation

Sie brauchen ein System-`git`; `git-lfs` ist optional.

- **Mit einem Installer:** ein `...-setup.exe` für Windows oder `install.sh` für Linux und macOS (pro Benutzer, ohne Root), jedem Release beigefügt. Ein aus einem Release-Archiv installierter Build kann sich außerdem über den Versions-Chip der Statusleiste selbst aktualisieren.
- **Aus einem Release-Archiv:** entpacken und der `INSTALL.txt` darin folgen.
- **Aus dem Quellcode:**

  ```
  git clone https://github.com/Poulpi-Lab/GitPoulpi.git
  cd GitPoulpi
  cargo build --release
  ```

  Die Programme landen in `target/release/`: `gitpoulpi` (CLI, und die
  Oberfläche, wenn es ohne Unterbefehl gestartet wird), `gitpoulpi-tui` und
  `gitpoulpi-mcp`.
- **Ein eigenes Release-Archiv bauen:** `cargo run -p xtask -- dist` erzeugt
  `target/dist/gitpoulpi-<version>-<target>.zip` (Windows) oder `.tar.gz`, mit
  einer `.sha256`-Datei.
- **Kleinere Binärdatei nur mit dem, was Sie brauchen:** `./build-custom.sh`
  (Windows: `build-custom`) fragt, welche Sprachen, Integrationen und
  Funktionen (KI, MCP, LFS, Worktrees, Submodule, Avatar-Download) gebaut
  werden sollen; siehe
  [docs/user/de/custom-builds.md](docs/user/de/custom-builds.md).

Alle Details, optionale Werkzeuge und Hinweise je Betriebssystem:
[docs/user/de/installation.md](docs/user/de/installation.md).

## Schnellstart

```
cd path/to/your/repository
gitpoulpi-tui          # or just: gitpoulpi
```

- `?` öffnet die Hilfe mit jeder Taste und Geste, `Ctrl+P` die Befehlspalette,
  `Ctrl+,` die Einstellungen, `q` beendet.
- Klicken Sie auf einen Commit, um seine Änderungen zu sehen; klicken Sie auf
  die oberste Zeile des Logs (die Änderungen), um Dateien zu stagen und zu
  committen.
- In einem Terminal, das es unterstützt (kitty, Ghostty, WezTerm, iTerm2 (und Windows Terminal und foot über Sixel)),
  wird der Graph als Pixel gezeichnet. In anderen (eine einfache
  Windows-Konsole, tmux) fällt er automatisch auf Zeichen zurück.

Die Schritt-für-Schritt-Version steht in [Erste Schritte](docs/user/de/getting-started.md),
und [der Rundgang durch die Oberfläche](docs/user/de/interface-tour.md) benennt
jeden Bereich des Bildschirms.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

KI ist optional und standardmäßig aus: Schalten Sie GitPoulpis eigene KI unter
*Einstellungen > KI* ein ([KI-Unterstützung](docs/user/de/ai.md)); lassen Sie einen
externen Agenten GitPoulpi bedienen mit [MCP und Agenten](docs/user/de/mcp-and-agents.md)
(Berechtigungen, jedes Tool) und dem Agenten-Skill
[`skills/gitpoulpi/SKILL.md`](skills/gitpoulpi/SKILL.md).

## Dokumentation

Die vollständige Benutzerdokumentation beginnt bei **[docs/user/de/README.md](docs/user/de/README.md)**:

| | |
|---|---|
| [Erste Schritte](docs/user/de/getting-started.md) | Installieren, ein Repository öffnen, den ersten Commit machen |
| [Rundgang durch die Oberfläche](docs/user/de/interface-tour.md) | Jeder Bereich des Fensters, benannt |
| [Mit dem Log arbeiten](docs/user/de/log-and-history.md) | Graph, Refs, Branches, Tags, Stashes, Rebase, Cherry-Pick |
| [Staging und Committen](docs/user/de/staging-and-committing.md) | Status, Diff, Hunks und Zeilen, der Composer, der Editor |
| [Einstellungen](docs/user/de/settings.md) | Jede Seite und Option der Einstellungen |
| [KI-Unterstützung](docs/user/de/ai.md) | Die eingebaute KI konfigurieren, jede KI-Aktion, Datenschutz |
| [Hosting und Issue-Tracker](docs/user/de/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: Tokens, ein Repository verknüpfen, Pull und Merge Requests |
| [CLI](docs/user/de/cli-reference.md) und [MCP](docs/user/de/mcp-and-agents.md) | Per Skript steuern oder von einem Agenten bedienen lassen (Berechtigungen, jedes Tool) |
| [Tastenkürzel](docs/user/de/keybindings.md) (und jeder Palettenbefehl), [Maus](docs/user/de/mouse-and-gestures.md), [Konfiguration](docs/user/de/config-reference.md), [LFS](docs/user/de/lfs.md), [Rückgängig](docs/user/de/undo-redo.md), [Fehlerbehebung](docs/user/de/troubleshooting.md) | Referenz und Hilfe |

Entwickler: [CLAUDE.md](CLAUDE.md) und `docs/architecture/overview.md`
beschreiben die Crates und den Entwurf; [CHANGELOG.md](CHANGELOG.md) listet auf,
was in jedem Release steckt.

## Lizenz

[MIT](LICENSE).

## Danksagungen und Lizenzen Dritter

GitPoulpi baut auf Open-Source-Bibliotheken auf und verwendet Farbwerte aus veröffentlichten Designs. Deren Lizenzen und Danksagungen, einschließlich der Autoren der Designs, stehen in [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt). Die Datei wird mit jeder Version ausgeliefert; das Fenster „Über GitPoulpi“ und `gitpoulpi licenses` zeigen sie ebenfalls an.

## Marken

GitKraken ist eine Marke des jeweiligen Inhabers. GitPoulpi ist ein unabhängiges Projekt, steht in keiner Verbindung zu diesem und wird von ihm weder unterstützt noch gesponsert; der Name wird nur genannt, um Kompatibilität und Inspiration zu beschreiben.
