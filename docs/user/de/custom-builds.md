[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · **Deutsch** · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Eigene Builds

Für alle, die eine kleinere Binärdatei wollen: Ein interaktives Skript fragt,
welche Sprachen, Hosting-Integrationen, Designs, Logo-Farben und Funktionen
enthalten sein sollen, und baut dann nur diese mit dem optimierten Profil. Der
normale Build und das offizielle Release bleiben unverändert (alles an).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd oder PowerShell (scripts/build-custom.ps1)
```

Das Ergebnis landet in `target/custom/`, nie über dem offiziellen
`target/dist/`. Das Skript gibt die Größe jeder Binärdatei aus und den
Unterschied zu `target/dist/`, wenn dieser Build existiert. Es schreibt außerdem
`BUILD-FEATURES.txt` neben die Binärdateien.

## Was Sie auswählen

| Auswahl | Wenn Sie sie abschalten |
|---|---|
| Sprachen | Nur Englisch (wird immer gebaut) plus die angehakten. Die Auswahl in den Einstellungen listet nur gebaute Sprachen. |
| Designs | GitPoulpi Dark, GitPoulpi Light und Terminal werden immer gebaut (`system` braucht Dark und Light). Jedes andere Design wird nach Familie gewählt (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` gibt sie aus). Nicht gewählte Designs werden nicht einkompiliert: Sie fehlen in der Auswahlliste der Einstellungen und in `gitpoulpi theme list`, und eine Konfiguration, die eines nennt, fällt zurück wie bei einem unbekannten Design, mit der üblichen Warnung. |
| Logo-Farben | Den Pixel-Oktopus im Fenster „Über GitPoulpi“ gibt es in 20 Farbvarianten (`--list-logo-colors`). Wählen Sie die einzukompilierenden (mindestens eine), die Standardvariante (das Fenster „Über GitPoulpi“ zeigt sie zuerst; auf sie fällt auch ein gespeichertes `ui.logo_color` zurück, das nicht einkompiliert ist; sie muss zu den einkompilierten gehören; offizieller Standard: Lavendel) und die Farbe des Windows-Programmsymbols (jede Variante, Standard: die Logo-Standardvariante). Mit nur einer Variante dreht sich das Logo weiter beim Klick, behält aber diese Farbe, und die Farbliste (`c`, Rechtsklick) wird nicht angeboten. |
| Integrationen (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Ohne jede: nur Klonen per URL; keine Issues, keine Pull Requests oder Merge Requests. |
| KI | Keine KI-Schaltflächen, -Befehle oder `ai`-CLI/MCP-Tools. |
| MCP | Keine Binärdatei `gitpoulpi-mcp` und kein Befehl `gitpoulpi mcp`. Die `ui`-Agentenbefehle bleiben. |
| LFS, Worktree, Submodule | Die zugehörigen Verwaltungsbildschirme und Befehle entfallen; als Antwort kommt ein typisiertes „not built into this binary“. |
| Avatar-Download | Nur Initialen-Abzeichen, kein Netzabruf. |
| Bildvorschau | Bilddateien zeigen den einfachen Binärhinweis; **Bearbeiten in** funktioniert weiterhin. |
| Radar | Kein Radar-Tab, kein `Alt+R` und kein Palettenbefehl; das Radar öffnet sich nie. Braucht eine Integration (ohne sie ist es ohnehin wirkungslos). |
| Mermaid | Kein Kopieren als Mermaid-gitGraph (Log-Menü, `Shift+Y`, Palette) und kein `log --format mermaid` in CLI/MCP. Auch kein Button **Diagramm öffnen** unter einem `mermaid`-Block einer [Unterhaltung](hosting-integration.md#5c-unterhaltungen) (der Block bleibt Code) und keine Palettenzeile *Unterhaltung: Diagramm des Eintrags öffnen*. |
| Unterhaltung | Keine [Unterhaltungsansicht](hosting-integration.md#5c-unterhaltungen) eines Issues oder Requests in der App: Enter auf einer Issue-Zeile, einer Board-Karte oder einer Radar-Zeile öffnet wie zuvor den Browser, es gibt keine Zeile *Unterhaltung öffnen* und keinen Palettenbefehl *Unterhaltung: ...*, und kein **Mit KI schreiben** in einer Unterhaltung (die Textfelder des Reviews, des Issue-Formulars und des Pull-Request-Formulars behalten es). Die Unterhaltungsbefehle von CLI und MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) und `ai hosting-reply` bleiben. Braucht eine Integration. |
| Board | Keine Kanban-Board-Ansicht (Schaltfläche in der Kopfzeile von ISSUES, Palette) und kein Befehl `hosting board`. Braucht eine Integration. |
| Review | Die Übersichtsseite des Reviews ist die Unterhaltungsansicht, daher enthält ein Build mit Review auch die Unterhaltungsansicht, was auch immer *Unterhaltung* sagt. Keine Code-Review-Ansicht in der App (Zeilenmenü, Enter auf einer Pull-Request-Zeile, Palette, `Review` im Radar) und keine KI-Hilfe beim Review. Die CLI/MCP-Befehle `hosting review` bleiben (Gerrit und Agenten nutzen sie). Braucht eine Integration. |
| Werkbank für den Verlauf (erweitertes Umschreiben des Verlaufs: Autoren, Daten, Nachrichten, Inhalt) | Keine Werkbank zum Umschreiben des Verlaufs: keine `Umschreiben:`-Palettenbefehle, kein Tastenkürzel, keine Tasten oder Hilfeabschnitt der Werkbank, keine Zeilen „Verlauf umschreiben“ in den Log-Menüs, keine Schaltfläche „Verlauf tiefer ändern...“ im Ändern-Dialog, kein „Regeln mit KI erstellen“ und keine KI für Skripte, keine Skriptgrenzen auf der Seite Sync der Einstellungen und keine KI-Blöcke zum Umschreiben in den KI-Einstellungen. Die CLI/MCP-Befehle `rewrite`, `ai rewrite-script`, `ai rewrite-rules` und `ui propose rewrite` entfallen im Befehlsbaum. `.mailmap`-Identitäten (`mailmap`, `log.use_mailmap`) gehören nicht dazu und bleiben. |
| Globale Suche | Keine Schaltfläche Globale Suche in der Titelleiste, kein `Ctrl+Alt+F`, kein Palettenbefehl `Suche: Workspace...` und kein CLI/MCP-Befehl `gitpoulpi search`. Die Suchleiste des Logs (`ui show search`) ist eine andere Funktion und bleibt. |
| Workspaces | Das Fenster arbeitet mit der einen Standard-Tabliste („Kein Workspace“): kein Workspace-Chip, kein Tab Workspaces, kein `Alt+W` / `Alt+J`, keine `Workspace:`-Palettenbefehle, keine Tab-Zeilen „Zum Workspace hinzufügen“, keine CLI/MCP-Befehle `gitpoulpi workspace`, kein Flag `--workspace` bei `commit` und `search`. Tabs und Sitzungen funktionieren wie gewohnt. Eine von einem anderen Build geschriebene `workspaces.toml` wird weder gelesen noch geändert; das Standardprofil und die KI-Anweisungen eines Workspace gelten nicht. Das Radar und die Globale Suche decken die geöffneten Tabs ab. |
| Rewrite-Skripte (eingebettetes Lua) | Skript-Regeln einer Verlaufsumschreibung werden mit einer typisierten Antwort „nicht in dieses Binary eingebaut“ abgelehnt, und es wird kein Lua gelinkt. Die Werkbank für den Verlauf selbst bleibt (sie hat eine eigene Auswahl). Ohne Bedeutung und mit Warnung, wenn die Werkbank für den Verlauf aus ist. |

Ihre Konfigurationsdateien werden von allen Builds gemeinsam genutzt: Eine
Einstellung für etwas, das nicht gebaut ist, bleibt erhalten und wird ignoriert.

## Interaktiv

Das Bash-Skript nutzt Auswahllisten von `whiptail` oder `dialog` (drei
Bildschirme, dann eine Zusammenfassung) oder nummerierte Abfragen, wenn keines
vorhanden ist. Das PowerShell-Skript nutzt `Out-GridView`, wenn verfügbar,
sonst Abfragen. Ihre bisherigen Auswahlen sind vorab angehakt.

## Nicht interaktiv

Jedes Argument überspringt die Fragen und führt direkt
`cargo run -p xtask -- build-custom` aus:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Option | Bedeutung |
|---|---|
| `--langs fr,de` / `all` / `en` | Sprachen außer Englisch. Die Kürzel sind die Namen der Sprachdateien (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Anbieter: GitHub, GitLab (auch selbst gehostet), Azure DevOps (Services und Server), Bitbucket (Cloud und Data Center) (Hosting), Gitea und Forgejo einschließlich Codeberg (`gitea`) (Hosting), Jira, Trello, Linear, YouTrack (Cloud und Server), Redmine (Issue-Tracker), Gerrit Code Review (`gerrit`, nur Review), lokale Markdown-Issues (`local-issues`, ein Ordner oder Obsidian-Vault). `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello` und `youtrack` sind jeweils ein eigenes Cargo-Feature; Größen in [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Einzukompilierende Design-Familien (Standard `all`). `--list-themes` gibt die Familien aus. |
| `--logo-colors all` / `violet,teal` | Einzukompilierende Logo-Farbvarianten (Standard `all`, in der Reihenfolge der Tabelle, die zugleich die Drehreihenfolge ist). `--list-logo-colors` gibt sie aus (`*` = offizieller Standard). |
| `--logo-default teal` | Die Standardvariante; muss zu den einkompilierten gehören (Standard: Lavendel, sonst die erste einkompilierte). |
| `--icon-color sun` | Farbe des Windows-Programmsymbols (Standard: die Logo-Standardvariante). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Jede ist standardmäßig an. |
| `--[no-]history-rewrite` | Die Werkbank für den Verlauf (erweitertes Umschreiben des Verlaufs: Autoren, Daten, Nachrichten, Inhalt). Standardmäßig an. |
| `--[no-]global-search` | Globale Suche (Schaltfläche in der Titelleiste, `Ctrl+Alt+F`, `gitpoulpi search`). Standardmäßig an. |
| `--[no-]workspaces` | Workspaces (benannte Gruppen von Repositories). Standardmäßig an; aus bleibt nur die Standard-Tabliste. |
| `--[no-]rewrite-scripts` | Eingebettetes Lua für die Skript-Regeln einer Umschreibung. Standardmäßig an; wird mit einer Warnung ignoriert, wenn die Werkbank für den Verlauf aus ist. |
| `--save <name\|file>` | Speichert die aufgelöste Auswahl als gespeicherte Auswahl (ein Name oder ein Dateipfad). |
| `--preset <name\|file>` | Geht von einer gespeicherten Auswahl aus (ein Name oder ein Dateipfad); die anderen Optionen überschreiben sie, egal wo sie in der Zeile stehen. |
| `--list-presets`, `--delete-preset <name>` | Gibt die gespeicherten Auswahlen und ihren Ordner aus / löscht eine per Name. |
| `--last` | Verwendet `target/custom-build/last-choices.env` erneut; andere Optionen überschreiben es. |
| `--full` | Gar keine Auswahl (alles an). |
| `--profile custom\|dist\|release` | Standard `custom` (die Einstellungen von `dist`, getrenntes Ausgabeverzeichnis). |
| `--dry-run` | Gibt den genauen Cargo-Befehl aus und hält an. |
| `-- <cargo args>` | Wird an Cargo weitergereicht, zum Beispiel `--target x86_64-unknown-linux-musl`. |

PowerShell-Namen: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

Die Design-Auswahl erreicht den Build über die Umgebungsvariable
`GITPOULPI_THEMES` (`all`, `none` oder eine kommagetrennte Liste von Familien),
gelesen vom Build-Skript von `gitpoulpi-config`, das die übrigen Paletten zur
Kompilierzeit weglässt; setzen Sie sie bei Bedarf selbst für ein einfaches
`cargo build`. `gitpoulpi --version` gibt eine Zeile `themes:` mit der gebauten Menge aus.

Die Logo-Auswahl sind die Umgebungsvariablen `GITPOULPI_LOGO_COLORS` (`all`
oder eine kommagetrennte Liste von Slugs), `GITPOULPI_LOGO_DEFAULT` und
`GITPOULPI_ICON_COLOR`, gelesen von den Build-Skripten von `gitpoulpi-tui`
(es filtert `assets/logo/colourways.json` in die Tabelle, die das Fenster „Über GitPoulpi“
einbettet) und der drei Binär-Crates (das Symbol). Die Quelle der Farben
bleibt `assets/logo/colourways.json`.

### Windows-Programmsymbol

Ist das Ziel Windows, erhalten die Programme `gitpoulpi`, `gitpoulpi-tui` und
`gitpoulpi-mcp` den Oktopus als Symbol, beim Build erzeugt (ohne Python oder
ImageMagick) in der gewählten Symbolfarbe, in Größen von 16 bis 128 px. Es wird
mit der Crate `winresource` eingebettet und braucht den Ressourcen-Compiler der
Plattform (`rc.exe` aus dem Windows SDK bei MSVC, `windres` bei GNU). Fehlt er,
gibt der Build nur eine Warnung aus, und das Programm behält das
Standardsymbol. Mit der GNU-Toolchain trägt nur `gitpoulpi` das Symbol (die
Ressource würde sonst doppelt gelinkt). Builds für Linux und macOS ignorieren
das alles bis auf die Farbtabelle.

## Gespeicherte Auswahlen

Um eine neue Version zu bauen, ohne die ganze Auswahl zu wiederholen, speichern Sie sie unter einem Namen und gehen später von ihr aus. Zusammen mit `--preset` angegebene Optionen überschreiben die gespeicherten Werte weiterhin.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Gespeicherte Auswahlen sind einfache `SCHLÜSSEL=Wert`-Dateien (das Format von `last-choices.env`, von einer Shell lesbar). Sie liegen im Ordner `custom-builds` des GitPoulpi-Konfigurationsverzeichnisses, außerhalb von `target/`, sodass `cargo clean` sie nicht löscht:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (standardmäßig `~/.config/gitpoulpi/custom-builds/`)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` verschiebt diesen Ordner zusammen mit der Konfiguration. `--preset` und `--save` akzeptieren auch einen Dateipfad (ein Wert mit `/` oder `\` oder mit der Endung `.env`), um eine Auswahl in einem Dotfiles-Repository aufzubewahren.

Eine von einer anderen GitPoulpi-Version geschriebene Auswahl lässt sich weiterhin laden. Ein unbekannter Schlüssel oder eine Sprache, Integration, ein Theme oder eine Farbe, die es nicht mehr gibt, wird mit einer Warnung ignoriert. Ein Schlüssel, der in der Datei fehlt (eine inzwischen hinzugekommene Funktion), erhält seinen Standardwert, und eine inzwischen hinzugekommene Integration wird aufgenommen; beides wird auf der Standardfehlerausgabe gemeldet.

Gibt es eine gespeicherte Auswahl, bietet der interaktive Modus zuerst „Ausgehen von“ an (frühere Auswahl, eine gespeicherte Auswahl oder alles an) und fragt am Ende, ob die aktuelle Auswahl unter einem Namen gespeichert werden soll.

## Was es spart

Gemessen unter Linux am 2026-10-29 beim Commit `65999ef5c`, vom vollständigen Build aus, die Differenz dazu in Klammern
(Details in `docs/developer/binary-size.md`):

| Build (ausgehend von `full`, sofern nicht anders angegeben) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Vollständig (alles an) | 64.6 | 50.5 | 31.7 |
| Nur Englisch | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP aus | 61.6 (-3.0) | 49.6 (-0.8) | nicht gebaut |
| KI aus | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| Alle Integrationen aus | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| KI + Integrationen + Avatar-Download aus | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| Werkbank für den Verlauf aus (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Rewrite-Skripte aus | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Globale Suche aus | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Workspaces aus | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar aus | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board aus | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review aus | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid aus | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS aus | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree aus | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Submodule aus | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Avatar-Download aus | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Bildansicht aus | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Alles aus, nur Englisch (LFS bleibt) | 35.2 (-29.4) | 28.9 (-21.5) | nicht gebaut |
| **Alles aus, nur Englisch** | **34.9 (-29.7)** | **28.8 (-21.6)** | nicht gebaut |

## Hinweise

- Ein eigener Build ist langsam wie jeder `dist`-Build (volles LTO, einige
  Minuten).
- Er läuft ohne sccache, wie `build-opti`.
- Sprachkürzel werden gegen die ausgelieferten Sprachdateien geprüft; ein
  unbekanntes Kürzel wird mit der Liste der gültigen abgelehnt.
