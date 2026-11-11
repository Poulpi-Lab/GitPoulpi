[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · **Deutsch** · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Konfigurationsreferenz

Jeder Schlüssel der `config.toml`, sein Standardwert und was er bewirkt. Sie müssen die
Datei selten von Hand bearbeiten: Der [Einstellungsbildschirm](settings.md) (`Ctrl+,`)
schreibt sie beim Ändern, und `gitpoulpi config set` ebenfalls.

## Wo sie liegt und wie ein Wert gewählt wird

| | |
|---|---|
| Datei | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` gibt sie aus. |
| Ein anderer Ordner | `GITPOULPI_CONFIG_DIR=<dir>` verschiebt den ganzen Konfigurationsordner (die Datei, `sessions/`, die Port-Dateien des Daemons): eine portable Installation, ein Test oder ein Agent, der von Ihren eigenen Einstellungen ferngehalten wird. |
| Vorrang | eingebauter Standard, dann `config.toml`, dann die Variable `GITPOULPI_*`, dann eine Kommandozeilenoption. Der höchste gewinnt. Ein falscher Wert (eine falsch geschriebene Auswahl in der Datei, eine fehlerhafte Variable) ist ein Fehler, nie still ignoriert. |
| Variablen | Schlüssel großschreiben und Punkte in Unterstriche verwandeln: `general.auto_prune` wird zu `GITPOULPI_GENERAL_AUTO_PRUNE`. Nur für Schlüssel, die GitPoulpi tatsächlich liest (nicht die unten als *nicht genutzt* markierten, auch keine Listen wie `ui.log_columns_hidden`). |
| Optionen | `--backend git2\|system` überschreibt `general.default_backend` für einen Daemon, den dieser Befehl startet. |
| Tokens und Schlüssel | Nie in dieser Datei. Hosting-Tokens und KI-API-Schlüssel gehen an den Credential-Helper Ihres Systems (Git Credential Manager, libsecret / `pass`, Keychain); die Datei enthält nur eine Referenz (`token_ref`). Ohne Helper schlägt das Speichern eines Tokens mit einer Meldung fehl, und das Token hält nur für die Sitzung. |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set` hält die Datei schlank (nur Schlüssel, die jemand gesetzt hat) und behält
Schlüssel, nach denen nicht gefragt wurde, verwirft aber Kommentare. Es lehnt Listen und
ganze Tabellen ab, außer `keybindings.*` (mehrere Tastenfolgen durch Leerzeichen
getrennt; leer hebt die Zuordnung auf). Bearbeiten Sie `[[profiles]]`, `[[integrations]]`
und `[[ai.profiles]]` in den Einstellungen oder von Hand.

In den Tabellen bedeutet **Neustart**, dass der Wert beim Start gelesen wird; alles andere
gilt sofort (oder bei der nächsten Aktualisierung, dem nächsten Fetch oder dem Schließen der
Einstellungen).

## `[general]`

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, mit Rückfall auf das Programm `git` für Staging, Merge, Rebase, Stash und Netz) oder `system` (nur das installierte `git`). Ein laufender Daemon behält sein Backend: **starten Sie ihn neu** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Wenn lokale Änderungen einen Checkout, Pull, Merge oder Rebase blockieren: `ask` (ein Dialog in der TUI; die CLI und MCP scheitern und nennen `--autostash`), `always` (stashen, ausführen, poppen) oder `never` (ablehnen). Siehe [Auto-Stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | Der Auto-Stash nimmt auch nicht verfolgte Dateien mit. |
| `general.initial_commits_in_graph` | `5000` | Commits, die das Log zuerst lädt (mindestens 500). Höher kostet bei riesigen Repositories Speicher und Zeichenzeit. |
| `general.lazy_load_commits_in_graph` | `true` | Ältere Commits laden, wenn Sie nahe ans Ende scrollen. Aus: Das Log endet bei der Anfangszahl. |
| `general.commits_per_lazy_page` | `1000` | Commits pro Seite im Hintergrund (mindestens 50). |
| `general.max_commits_in_graph` | `100000` | Wo das verzögerte Laden endet (etwa 300 Byte pro Commit); `0` = keine Grenze. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All im Hintergrund alle N Minuten, 0 bis 60; `0` = aus. **Neustart.** |
| `general.auto_fetch_interval_unit` | `"auto"` | Die Einheit, in der Einstellungen das Intervall zeigt: `minutes`, `hours` oder `auto`. Nur Anzeige: Das Intervall sind immer `auto_fetch_interval_minutes` Minuten. `auto` (und eine Konfiguration ohne den Schlüssel) wählt die größte Einheit, die die Minuten ohne Rest teilt (60 lesen sich als „1 Stunde“, 45 als „45 Minuten“); eine Einheit, die nicht passt, wird ignoriert. Einstellungen schreibt beide Schlüssel; ein älteres GitPoulpi ignoriert die Einheit und liest weiter die Minuten. |
| `general.fetch_on_tab_open` | `true` | Fetch All, wenn ein Tab geöffnet wird (übersprungen, wenn GitPoulpi dieses Repository vor weniger als 60 s gefetcht hat). |
| `general.auto_prune` | `false` | Fetches bereinigen auch Remote-Tracking-Branches, die es nicht mehr gibt. |
| `general.auto_prune_tags` | `false` | Ein Fetch löscht auch die lokalen Tags, die das Remote gelöscht hat (nur Tags, die dort lagen; ein Tag, den Sie nie gepusht haben, bleibt erhalten). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Aktualisierung der Listen von Pull Requests / Issues und des Chips der Symbolleiste, 0 bis 120; `0` = nur auf Anforderung und bei Ereignissen. |
| `general.hosting_refresh_unit` | `"auto"` | Die Einheit, in der Einstellungen die Aktualisierung zeigt: `minutes`, `hours` oder `auto`, gelesen wie `general.auto_fetch_interval_unit`. Nur Anzeige. |
| `general.keep_submodules_up_to_date` | `false` | Nach einem Checkout oder Pull `git submodule update --init --recursive`, wenn ein Submodul nicht mehr passt. Siehe [Submodule](submodules.md). |
| `general.default_branch_name` | `"main"` | Erster Branch eines mit Init angelegten Repositorys; leer lässt git entscheiden. Benennt nichts um. |
| `general.delete_orig_files_after_merge` | `false` | Nach einem gelungenen Merge die Sicherungsdateien `*.orig` löschen. |
| `general.remember_tabs` | `true` | Die Tabs der letzten Sitzung wieder öffnen. Aus, öffnet ein Start in einem Repository nur dieses Repository, und ein Start außerhalb eines Repositorys öffnet nur Home. **Neustart.** |
| `general.radar_enabled` | `true` | Das [Radar](radar.md): sein Tab, seine Paletteneinträge, `Alt+R` und sein Abrufen im Hintergrund. Ausgeschaltet verschwindet das alles, und dafür wird nichts abgerufen. Wirkt sofort; in einem Build ohne die Funktion `radar` nicht angezeigt. |
| `general.startup_tab_in_repo` | `"current-repo"` | Der Tab, auf dem ein Start öffnet, wenn der Ordner, in dem er beginnt, in einem Repository liegt: `"current-repo"` (der Tab dieses Repositorys, bei Bedarf neu geöffnet), `"last"` (der beim letzten Beenden aktive Tab), `"home"` oder `"radar"`. Ein mit `--repo` angegebenes Repository öffnet immer seinen eigenen Tab. `"last"` ist Home bei ausgeschaltetem `remember_tabs` oder wenn nichts gemerkt wurde; `"radar"` ist Home, wenn das Radar aus ist oder im Build fehlt. Wird einmal beim Start gelesen; erfordert einen Neustart. |
| `general.startup_tab_outside_repo` | `"home"` | Der Tab, auf dem ein Start öffnet, wenn der Ordner, in dem er beginnt, kein Repository ist (etwa ein Benutzerordner): `"home"`, `"last"` oder `"radar"`. `"last"` ist Home bei ausgeschaltetem `remember_tabs` oder wenn nichts gemerkt wurde; `"radar"` ist Home, wenn das Radar aus ist oder im Build fehlt. Wird einmal beim Start gelesen; erfordert einen Neustart. |
| `general.startup_tab_launcher` | `"last"` | Der Tab, auf dem ein Start öffnet, wenn GitPoulpi über die Desktop- oder Startmenü-Verknüpfung gestartet wird, die es angelegt hat (sie übergibt `--launcher`), egal in welchem Ordner: `"home"`, `"last"` oder `"radar"`. Hat Vorrang vor den beiden Einstellungen darüber. `"last"` ist Home bei ausgeschaltetem `remember_tabs` oder wenn nichts gemerkt wurde; `"radar"` ist Home, wenn das Radar aus ist oder im Build fehlt. Wird einmal beim Start gelesen; erfordert einen Neustart. |
| `general.group_worktree_tabs` | `true` | Die Tabs der verknüpften Worktrees eines Repositorys direkt hinter dem Tab dieses Repositorys halten: wenn ein Worktree geöffnet wird, wenn ein Worktree-Tab weggezogen wird (er springt zu seiner Gruppe zurück; der Repository-Tab ist der Kopf der Gruppe, seine Worktrees folgen ihm) und wenn die Einstellung eingeschaltet wird. Ausgeschaltet bleiben die Tabs dort, wo Sie sie ablegen. |
| `general.undo_depth` | `100` | Wie viele Operationen Rückgängig zurückgehen kann; `0` = unbegrenzt. |
| `general.conflict_detection` | `true` | Proaktive Konflikterkennung: ein Abzeichen im Log, wenn ein Merge des aktuellen Branches in den Branch, von dem er abzweigte, Konflikte verursachen würde, im Hintergrund berechnet (ein Merge im Arbeitsspeicher: an Ihren Dateien, dem Index und den Refs ändert sich nichts). Ein Repository kann sie für sich ausschalten (`[preferences] conflict_detection`). Die Basis-Branches, mit denen sie vergleicht, haben eigene Listen: [`[conflict_prevention]`](#conflict_prevention-basis-branch-listen). Siehe [Merge-Konflikte lösen](merge-conflicts.md#konfliktvermeidung). |
| `general.dev_mode` | fehlt (automatisch) | Für alle, die GitPoulpi aus einem Checkout bauen; nie in den Einstellungen gezeigt. Fehlt er: an, wenn die laufende exe in `<checkout>/target/<profile>/` liegt (`Cargo.toml` und `crates/gitpoulpi-tui` neben `target`). `true` / `false` erzwingt es. Der Entwicklermodus schaltet nur `ui.launcher_build` frei. |
| `general.journal_outside` | `true` | Aufzeichnen, was eine Shell oder ein KI-Agent an Branches, Tags, Stashes und HEAD tut, als Einträge des Rückgängig-Verlaufs, als „außerhalb“ markiert, sodass `Ctrl+Z` es zurücknehmen kann. Änderungen an Dateien werden nie erfasst. Setzt `undo_enabled` voraus. Siehe [Rückgängig und Wiederholen](undo-redo.md#änderungen-außerhalb-von-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: keine Rückgängig-Buchführung (am schnellsten), der Rückgängig-Verlauf wird gelöscht, die Schaltflächen Rückgängig/Wiederholen verschwinden. Siehe [Rückgängig und Wiederholen](undo-redo.md). |
| `general.active_profile` | `""` | ID der Identität aus `[[profiles]]`, mit der Commits gemacht werden. Leer oder ein Profil mit leerem Namen oder leerer E-Mail: `user.name` / `user.email` von git. |
| `general.external_editor` | `""` | Befehl für den Eintrag *Im Editor öffnen* einer Datei, z. B. `code -r {file}`. Leer: VS Code, falls installiert, sonst der Standard des Systems. |
| `general.external_diff_tool` | `""` | Befehl für *Im externen Diff-Tool öffnen*: `{local}` (committete / HEAD-Version), `{remote}` (Version des Arbeitsverzeichnisses / des Index). Leer: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Befehl für *Im externen Merge-Tool öffnen* auf einer Konfliktkarte (eine binäre, LFS- oder zu große Datei): `{base}`, `{local}` (A, aktuell), `{remote}` (B, eingehend) und `{merged}` (die Datei, in die das Tool das Ergebnis schreibt; GitPoulpi stagt sie, wenn das Tool erfolgreich endet). Leer: `git mergetool --no-prompt`. |
| `general.file_explorer` | je OS | Befehl für *Im Ordner anzeigen*: `{path}` (die Datei), `{dir}` (ihr Ordner). Standards: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Die drei Befehlsvorlagen werden zuerst in Argumente zerlegt (`"..."` fasst ein Wort
zusammen), dann werden die Platzhalter gefüllt, sodass Pfade mit Leerzeichen ein
Argument bleiben. Es ist keine Shell beteiligt.

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log und Graph**

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (ein Bild, wo das Terminal eines zeichnen kann, sonst Zeichen), `image`, `text`. **Neustart.** |
| `ui.image_protocol` | `"auto"` | Wie Bilder (Graph, Avatare) zum Terminal gelangen: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = Kitty-Protokoll in kitty, Ghostty und einem herdr mit `kitty_graphics = true`; iTerm2 in WezTerm, iTerm2, Rio, mintty; Sixel (nur Avatare) in Windows Terminal, foot, Konsole, mlterm; keines in tmux, zellij, screen und unbekannten Terminals. **Neustart.** |
| `ui.theme` | `"system"` | Das Farbdesign: ein eingebauter Slug (`gitpoulpi theme list` listet alle), der Slug einer Datei in `<config>/themes/` oder `system` (folgt dem hellen oder dunklen Hintergrund des Terminals, beim Start erkannt; der alte Wert `default` bedeutet dasselbe). `config set` lehnt unbekannte Namen ab. Siehe [Designs](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Spurpalette: `theme` (dem Design folgen) oder ein beliebiger Name einer Spurpalette: `gitpoulpi`, `light`, `material`, `serie` und die Paletten der anderen Designs. |
| `ui.graph_lane_background` | `"off"` | `off` oder `row_tint` (ein schwaches Band in der Spurfarbe hinter dem Graphen, plus ein Balken an seinem Rand). **App: Zeilenfärbung des Graphen umschalten** (`Ctrl+N Shift+T`) schaltet es um; **App: Farbschema des Graphen wechseln...** (`Ctrl+N Shift+Y`) wählt `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (nur mit `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` und `right` zeichnen eine Spalte; ein Fenster unter 80 Spalten fällt auf `bottom` zurück. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: ob ein Dialog, den Sie verschoben oder in der Größe geändert haben (Titelleiste oder Ränder ziehen, oder `Ctrl+N m`), seinen Platz behält. Eine Einstellung für jeden Dialog (jeder behält darunter seine eigene Position und Größe): bis GitPoulpi geschlossen wird (Standard), auf der Festplatte für den nächsten Start (alle Dialoge in der einen `dialog_geometry.txt` im Datenordner) oder gar nicht (jeder Dialog öffnet sich an Standardposition und in Standardgröße). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: wo die Schaltflächen entlang der Leiste sitzen (links, Mitte, rechts einer Zeile; oben, Mitte, unten einer Spalte). |
| `ui.toolbar_pr_pinned` | `true` | Die Pull-Request-Schaltfläche ist am Ende der Symbolleiste fixiert; `false` lässt sie mit den anderen Schaltflächen fließen. |
| `ui.statusbar_position` | `"bottom"` | `top` (über den Repository-Tabs) oder `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (die letzte Zeile des Fensters, unter der Statuszeile) oder `left` (eine Spalte am linken Rand; unter 80 Spalten bleibt sie oben). |
| `ui.graph_show_stashes` | `true` | Eine Zeile pro Stash im Log. |
| `ui.ref_connector` | `true` | Eine dünne Linie von einer Ref-Pille zu ihrem Commit. |
| `ui.column_resize` | `"line"` | Wie eine Spaltengrenze im Log, im Radar und in der Commit-Spalte der Diff-, Datei- und Blame-Ansichten gegriffen wird. `line`: Jede Grenze ist eine einzellige Lücke mit einer kleinen Markierung in der Kopfzeile; solange der Zeiger auf der Grenze steht, in der Kopfzeile oder irgendwo in der Tabelle, wird über die volle Höhe eine dünne Akzentlinie gezeichnet, und ein Ziehen von jeder Stelle daran ändert die Größe. `header`: keine Lücke und keine Markierung; das erste Zeichen der Kopfzelle nach der Grenze leuchtet auf, und ein Ziehen von dort ändert die Größe (nur in der Kopfzeile). Die Tasten zum Einstellen und die Paletteneinträge sind in beiden Modi gleich. |
| `ui.ref_tree` | `true` | Branches und Tags an `/` in Ordnern in der Refs-Seitenleiste gruppieren (`y` schaltet es um). |
| `ui.graph_ghost_ref` | `true` | Die Ref-Spalte des Logs zeigt eine blasse Pille mit dem Namen des Branches (sonst Remote-Branch), zu dem ein überfahrener Commit gehört, wenn der Commit keine eigene Ref hat und die Spalte Platz hat; nie ein Tag. Ein gelöschter Branch wird aus dem Betreff des Merge-Commits abgeleitet, der ihn zusammengeführt hat, und durchgestrichen gezeichnet. Fährt man über die Pille, erscheint ihre Art. |
| `ui.reveal_on_ref_click` | `true` | Ein einfacher Klick auf eine Ref (Branch, Tag, Remote-Branch, Stash) im Refs-Bereich scrollt den Log zu ihrem Commit und lässt die Zeile aufblinken; die Commit-Auswahl bleibt unverändert. |
| `ui.log_wip_counts_in_message` | `true` | WIP-Änderungszähler in der WIP-Nachricht, solange die Spalte Änder. ausgeblendet ist. |
| `ui.log_columns_layout` | `"fit"` | Wie das Log seine Spalten anordnet. `fit`: Jede sichtbare Spalte passt genau in den Bereich, und nur `message` ist flexibel; ist der Bereich schmal, weichen ganze Spalten. `scroll`: Die Spalten behalten ihre Breiten (`message` auch), nichts weicht, und die Tabelle scrollt seitlich, wenn sie breiter als der Bereich ist. |
| `ui.log_column_widths_scroll` | leer | Die Spaltenbreiten des Layouts `scroll`, Name -> Zellen (`message` erlaubt, anders als `ui.log_column_widths`, das das Layout `fit` bleibt und nie `message` enthält). Geschrieben, wenn Sie im Layout `scroll` eine Spalte ziehen, in der Größe ändern oder anpassen. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | leer | Die Log-Spalten (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), geschrieben, wenn Sie eine Spalte ausblenden, ziehen oder in der Größe ändern. `message` lässt sich nicht ausblenden. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | leer | Die Teile der [Commit-Spalte](interface-tour.md#die-commit-spalte) neben einem Diff, für sich und getrennt von denen des Logs: die angezeigten Teile nach Namen (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; leer bedeutet Spuren, Nachricht, Autor und Datum), ihre Reihenfolge von links nach rechts mit denselben Namen (leer ist die eben genannte Reihenfolge; ein ausgelassener Name folgt in dieser Reihenfolge) und die gezogenen Breiten in Zellen je Teil (ein ausgelassener Teil ist automatisch), geschrieben, wenn Sie die Teile wählen, verschieben oder ihre Größe ändern. `message` lässt sich nicht ausblenden. Eine Sitzungsdatei, die vor diesen Schlüsseln geschrieben wurde, wird einmal in sie eingelesen. |
| `ui.sidebar_sections_hidden` | leer | Die ausgeblendeten Abschnitte der linken Seitenleiste (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), geschrieben über das Rechtsklickmenü der Leiste oder die Palette. Mindestens ein vorhandener Abschnitt bleibt immer sichtbar. Das ist der Standard: Jedes Repository kann eigene Abschnitte ausblenden (in seiner Statusdatei gespeichert), und dieser Schlüssel gilt für ein Repository ohne eigene Wahl; *Für alle Repositories verwenden* schreibt ihn. |
| `ui.sidebar_sections_scope` | `"repository"` | Wessen Auswahl die Abschnitte der linken Seitenleiste sind: `repository` (jedes Repository behält seine eigene, `ui.sidebar_sections_hidden` ist der Standard) oder `global` (jedes Repository zeigt `ui.sidebar_sections_hidden`, und eine Änderung wird dort geschrieben). Beim Umschalten geht keine der beiden Auswahlen verloren: Die Auswahlen der einzelnen Repositories bleiben in ihren Statusdateien, solange `global` aktiv ist. Einstellungen: *Abschnitte der Seitenleiste*. |
| `ui.language` | `"auto"` | Die Sprache der Oberfläche: `auto` (das System-Locale, sonst Englisch), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Live.** `GITPOULPI_LANG` überschreibt sie. Eine Sprache, die nicht in diesem Build ist, zeigt Englisch. **App: Sprache wechseln...** (`Ctrl+N g`) setzt sie ebenfalls, mit Live-Vorschau. Die Ausgabe von CLI und MCP bleibt Englisch. |
| `ui.translate_git_terms` | `false` | `false`: Commit, Push, Pull, Stash, Tag... bleiben in jeder Sprache Englisch. `true`: das Git-Vokabular der gewählten Sprache (Französisch „remiser“, „étiquette“, „valider“...). **Live.** |
| `ui.git_terms_english` | `false` | Nur ja, ko und zh-CN: zeigt die englischen Git-Wörter (stage, unstage, staged, unstaged...) statt ihrer Umschrift. Ohne Wirkung, solange `ui.translate_git_terms` `true` ist. **Live.** |
| `ui.mouse_enabled` | `true` | `false` überlässt die Maus (und die Textauswahl) dem Terminal. **Neustart.** |
| `ui.mouse_scroll_lines` | `3` | Log-Zeilen pro Mausrad-Raste, 1 bis 20. **Neustart.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` fragt das Terminal nach dem Kitty-Tastaturprotokoll (damit `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` ankommen); `off` fragt nie. **Neustart.** |
| `ui.logo_color` | `""` | Farbvariante des Logos im Info-Fenster, ein Slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); leer oder unbekannt bedeutet den Standard des Builds (`lavender`). Geschrieben, wenn Sie auf das Logo klicken oder im Info-Fenster eine Farbe wählen. |

**Avatare** (standardmäßig aus; sie einzuschalten sendet einen SHA-256-Hash der
Autoren-E-Mails an Gravatar und fragt Ihr verbundenes GitHub/GitLab-Konto)

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (im Commit-Kreis des Bildgraphen), `author` (in der Spalte Autor), `off`. Das alte `ui.show_avatars` (`true`) wird weiterhin als `author` gelesen, solange `ui.avatars` fehlt. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Suchreihenfolge: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Ein GitLab-Host wird nur über ein verbundenes Profil gefragt. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (lokal erzeugt) oder `initials`. Auf einem Terminal ohne Bildprotokoll wird immer das Initialen-Abzeichen verwendet. |

**Anzeige von Diff und Editor**

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` oder `split` (`m` in einem Diff). |
| `ui.diff_hunks_only` | `true` | `false` zeigt die ganze Datei mit den Änderungen an Ort und Stelle (`H`). |
| `ui.diff_show_whitespace` | `true` | Änderungen nur an Leerraum anzeigen (`s`). |
| `ui.diff_word_wrap` | `false` | Lange Diff-Zeilen umbrechen (`w`; Einstellungen > Diff & Editor). |
| `ui.minimap` | `false` | Eine Minimap neben der Bildlaufleiste von Diff und Editor (`M`). |
| `ui.mermaid_detail` | `refs` | Detail von „Als Mermaid gitGraph kopieren“ im Log: `structure`, `refs`, `subjects`, `full` (wird vom Popup gemerkt; der Standard von `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Richtung dieses Graphen: `lr` (von links nach rechts) oder `tb` (von oben nach unten). |
| `ui.mermaid_fence` | `false` | Das kopierte Mermaid-Skript mit einem `mermaid`-Codeblock umschließen. |
| `ui.mermaid_subject_width` | `40` | Commit-Betreffzeilen werden in den Mermaid-IDs auf so viele Zeichen gekürzt (`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` oder `16`: mit wie vielen Farben gezeichnet wird. `auto` ist Truecolor, außer auf Apple Terminal (256), der Linux-Konsole (16) und `*-256color`-Terminals ohne `COLORTERM` (256). Live. Siehe [Designs](themes.md). |
| `ui.input_color` | `"theme"` | Hintergrund von Textfeldern: `theme` (der des Designs), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | Die Farbe `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | Der KI-Commit-Composer öffnet mit den Dateien jedes Vorschlags `expanded` oder `collapsed`. |
| `ui.blame_side` | `"right"` | Auf welcher Seite der Blame-Ansicht die Annotationsspalte (Avatar, Autor, Alter, Zusammenfassung) sitzt: `right` (Standard) oder `left`. |
| `ui.blame_width` | `46` | Breite der Blame-Annotationsspalte in Zellen (24 bis 90). Ziehen Sie ihren Rand oder drücken Sie in der Ansicht `<` / `>`; die neue Breite wird hier gespeichert. |
| `ui.pr_terminology` | `"auto"` | `auto` (Pull Request bei GitHub, Merge Request bei GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Blendet den Symbolleisten-Chip des jüngsten Pull oder Merge Requests des ausgecheckten Branches aus, sobald er geschlossen oder gemergt ist; stattdessen erscheint der blasse Chip „+ PR“ (einen neuen starten), als hätte der Branch keinen. `false`: Der Chip zeigt immer den jüngsten Request, egal in welchem Zustand. |

Die Diff-Schalter, die Sie in einem Diff ändern, werden hier sofort gespeichert und von
jedem danach geöffneten Diff verwendet.

**Datumsformate**: `ui.date_format` (`"YYYY-MM-DD"`, die Datumsspalte Blame / Dateiverlauf),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, die Spalte Datum des Logs) und
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, die Commit-Details, der Tooltip des
Datums und *Kopieren > Datum*). Platzhalter: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (Wochentag, voll / abgekürzt) `HH` `mm` `ss`; alles andere wird unverändert
kopiert. Monats- und Wochentagsnamen folgen `ui.language`, und ein nach einer Tageszahl
geschriebener Monat steht im Genitiv, wo die Sprache einen hat (Russisch, Ukrainisch,
Polnisch, Tschechisch). Eine Folge `D MMMM YYYY` wird auf Japanisch, Chinesisch und
Koreanisch mit dem Jahr zuerst geschrieben (`2024年3月5日`). `ui.date_locale` (`"system"`,
der Standard, oder `"english"`) hält die Namen unabhängig von der Sprache auf Englisch.

**Agenten, Terminals und der Tab-Ordner**

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | Das Fenster akzeptiert überhaupt `ui show` / `ui propose` von Agenten (Einstellungen > Agenten & MCP > *KI-Agenten die Ansicht steuern lassen*). Aus: Kein Agent erreicht es. Siehe [MCP und Agenten](mcp-and-agents.md#dem-benutzer-dinge-zeigen). |
| `ui.launch` | `"off"` | Ob ein Agent ein GitPoulpi-Fenster öffnen darf, wenn keines offen ist: `off`, eine Voreinstellung (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) oder `custom`. |
| `ui.launch_command` | `""` | Die Vorlage `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. In Argumente zerlegt, keine Shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (die Tab-Leiste bietet *Beenden & cd hierher* an) oder `auto` (jedes Beenden ändert den Ordner der Shell, und ein neuer Tab führt den Standardeintrag von *Öffnen in* aus). Siehe [Dem Ordner des Tabs folgen](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Veraltet, siehe unten und [Öffnen in](open-in.md). Wie das alte *Terminal hier* ein Terminal öffnete: `auto` (die Voreinstellung `ui.launch`, sonst `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), eine Voreinstellung oder `custom`. |
| `ui.terminal_launch_command` | `""` | Die Vorlage `custom`; `{repo}` ist der Ordner. |
| `ui.open_in` | *(fehlt)* | Die Liste [Öffnen in](open-in.md), ein Array von Tabellen `[[ui.open_in]]` mit `label` und `command`. Fehlend bedeutet „noch nicht konfiguriert“: Die Liste wird berechnet aus den alten Schlüsseln `ui.terminal_launch*` (ein Wert ungleich `auto` wird der erste Eintrag), dem Datei-Explorer des Betriebssystems, den aus der Umgebung erkannten Multiplexern und Terminals (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) und den im `PATH` gefundenen Editoren. Das erste Speichern aus den Einstellungen schreibt die Liste. `open_in = []` ist eine leere Liste. Mit `config set` nicht setzbar (die Datei bearbeiten). |
| `ui.open_in_default` | `""` | Bezeichnung des Eintrags, den „Im Standardeintrag öffnen“ ausführt; leer oder unbekannt bedeutet den ersten Eintrag. |
| `ui.launch_terminal` | `"auto"` | Das Terminal, in dem die Desktop-Verknüpfungen (Einstellungen > Starter) und der Neustart unter Windows GitPoulpi starten. `auto` wählt unter dem Installierten (Windows: Windows Terminal, sonst WezTerm, Alacritty, die einfache Konsole; Linux: `xdg-terminal-exec`, sonst das erste gefundene Terminal; macOS: iTerm2, Ghostty, WezTerm oder kitty, falls installiert, sonst Terminal), `default` ist das des Betriebssystems, oder eine Terminal-ID wie `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Nur im Entwicklermodus (siehe `general.dev_mode`; nur dann unter Einstellungen > Starter gezeigt, sonst ignoriert). `"current"`: Verknüpfungen und der Befehls-Alias starten den Build, der sie angelegt hat. `"newest"`: Sie starten den zuletzt gebauten von `target/dist` und `target/release` des Checkouts, bei jedem Klick neu entschieden, und das Starten eines der beiden Builds schreibt sie nie neu. |
| `ui.command_alias` | `""` | Der Name des Befehls, der GitPoulpi in einem Terminal startet, zum Beispiel `gp` (Einstellungen > Starter, `gitpoulpi alias create`). Leer: keiner. Nur Buchstaben, Ziffern, `-`, `_` und `.`. Es ist lediglich der Name, den die Schaltfläche **Erstellen: Befehlsalias** verwendet; es existiert nichts, bis Sie ihn erstellen. |
| `ui.command_alias_add` | `""` | Der Name des zweiten Befehls, zum Beispiel `gpa`, der `gitpoulpi --add` ausführt: Er öffnet den aktuellen Ordner als neuen Tab im bereits laufenden GitPoulpi (Einstellungen > Starter, `gitpoulpi alias create gpa --add`). Leer: keiner. Dieselben Namensregeln; wird von der Schaltfläche **Erstellen** zusammen mit `ui.command_alias` angelegt. |
| `ui.relaunch_in_terminal` | `true` | Nur Windows: Wird die exe per Doppelklick in einer eigenen Konsole gestartet, öffnet GitPoulpi sich in `ui.launch_terminal` neu und beendet sich. Aus: Es bleibt in dieser Konsole. |
| `ui.reuse_window` | `true` | Ein Klick auf eine GitPoulpi-Verknüpfung (`gitpoulpi launch`) zeigt das Fenster eines bereits geöffneten GitPoulpi, statt ein weiteres zu starten. Aus: Jeder Klick startet ein neues Fenster. |

Die **Befehle von `ui.open_in`** werden zuerst in Argumente zerlegt (Leerraum, `'...'`, `"..."`), dann werden die Variablen in jedem Argument ersetzt, sodass ein Ordner mit Leerzeichen ein Argument bleibt; es ist keine Shell beteiligt, außer der Befehl startet eine (`sh -c '... "$GITPOULPI_CWD"'`). Variablen: `${cwd}` (der Ordner des Tabs), `${repo}` (Repository-Wurzel), `${name}` (Basisname des Ordners), `${branch}` (leer bei losgelöstem HEAD), `${web_url}` (die Webseite des Remotes `origin`, wenn es ein GitHub- oder GitLab-Projekt ist, sonst leer; ein Eintrag, der sie nutzt, weigert sich zu laufen, wenn sie leer ist), `${env:NAME}` (leer, wenn nicht gesetzt). Das alte `{repo}` bedeutet weiterhin `${cwd}`. Jedes andere `${...}` ist ein Fehler, der es nennt. Beispiel:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Veraltet: `ui.terminal_launch` und `ui.terminal_launch_command` werden weiterhin gelesen (sie bestücken die Liste, solange `ui.open_in` fehlt: `custom` wird zu „Terminal (custom)“ mit `{repo}` umgeschrieben zu `${cwd}`, eine Voreinstellungs-ID wird zu dieser Voreinstellung), sind aber durch `ui.open_in` ersetzt.

### Bilder

| Schlüssel | Standard | Bedeutung |
|---|---|---|
| `ui.image_view` | `true` | Eine Binärdatei, die ein Bild ist (PNG, JPEG, GIF, WebP, BMP, ...), in der Diff-Ansicht und der Dateiansicht als Bild anzeigen. Aus: der einfache Binärhinweis. |
| `ui.image_view_max_mib` | `32` | Größere Bilder (in MiB) werden nicht dekodiert; stattdessen erscheint eine Infokarte. |
| `ui.image_view_max_unit` | `"auto"` | Die Einheit, in der Einstellungen die Grenze zeigt: `mib`, `gib` (1024 MiB) oder `auto`. Nur Anzeige: Die Grenze sind immer `image_view_max_mib` MiB, 1 bis 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | Auf einem Terminal ohne Bildprotokoll: `halfblock` (farbige Halbblock-Zeichen) oder `card` (nur Infokarte). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom, ab dem das Vergrößern eckige Pixel zeigt statt zu glätten: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Die Vorher- und Nachher-Bereiche teilen sich einen Zoom und eine Scroll-Position. |
| `ui.image_editors` | *(fehlt)* | Die Bildeditoren für „Bearbeiten in“, ein Array von Tabellen `[[ui.image_editors]]` mit `label` und `command`. Fehlend bedeutet „noch nicht konfiguriert“: Die Liste sind die auf diesem Rechner erkannten Editoren (Programme im `PATH`, Flatpak-Installationen, bekannte Installationsordner unter Windows, `/Applications` unter macOS), dann die Standard-App des Betriebssystems. Das erste Speichern aus den Einstellungen schreibt die Liste. `image_editors = []` ist eine leere Liste. Mit `config set` nicht setzbar (die Datei bearbeiten). |
| `ui.image_editor_default` | `""` | Bezeichnung des Editors, den „Im Standardeditor bearbeiten“ ausführt; leer oder unbekannt bedeutet den ersten Eintrag. |

Die **Befehle von `ui.image_editors`** werden zuerst in Argumente zerlegt (Leerraum, `'...'`, `"..."`), dann werden die Variablen in jedem Argument ersetzt, sodass ein Pfad mit Leerzeichen ein Argument bleibt; es ist keine Shell beteiligt. Variablen: `${file}` (der absolute Pfad des Bildes), `${dir}` (sein Ordner), `${name}` (sein Dateiname), `${repo}` (die Repository-Wurzel, leer bei einer temporären Kopie), `${env:NAME}` (leer, wenn nicht gesetzt). Jedes andere `${...}` ist ein Fehler, der es nennt. Ein Bild, das nicht die Datei des Arbeitsverzeichnisses ist (die Version eines Commits, der Index, eine gelöschte Datei), wird als **schreibgeschützte temporäre Kopie** geöffnet, sodass ein Editor das Repository nicht berühren kann. Editoren ohne Kommandozeile (Affinity Photo aus dem Microsoft Store, manche Photoshop-Installationen) stehen nicht in den Voreinstellungen: Schreiben Sie einen eigenen Eintrag. Beispiel:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Noch nicht genutzt** (akzeptiert, gespeichert, aber nichts liest sie):
`ui.graph_colorization` (aus den Einstellungen entfernt; beim
Laden weiterhin akzeptiert), `ui.date_word_format` (aus den Einstellungen entfernt; beim
Laden weiterhin akzeptiert). Das alte
`ui.diff_view_mode` wird einmal als `diff_layout` / `diff_hunks_only` gelesen.

## `[editor]`

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `editor.tab_size` | `4` | Spalten pro Tabstopp, 1 bis 8 (Editor und Diffs). |
| `editor.eol` | `"auto"` | Zeilenende, das der eingebaute Editor schreibt: `auto` (das der Datei behalten), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Sprachfarben im Editor. |
| `editor.line_numbers` | `true` | Zeilennummern im Editor und in den Diffs. |
| `editor.follow_selection` | `true` | Solange der Editor offen ist, öffnet die Auswahl einer anderen Datei im rechten Bereich diese ebenfalls im Editor, nachdem gefragt wurde, ob ungespeicherte Änderungen gespeichert werden sollen. Eine Datei, die sich nicht bearbeiten lässt, öffnet sich in ihrer gewohnten Ansicht. Eine Datei eines Commits verlässt den Editor für ihren Diff. |

## `[merge]`

Der Editor für Merge-Konflikte ([Merge-Konflikte lösen](merge-conflicts.md)). Einstellungen > Diff & Editor zeigt ihn.

| Schlüssel | Standard | Bedeutung |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | Die Breite des Bereichs des Editors in Spalten, ab der A und B nebeneinander gezeigt werden (lange Zeilen scrollen seitwärts: Shift+Mausrad, Shift+Pfeile). Schmaler zeigt er jeweils eine Seite hinter Tabs. `40` ist der niedrigste Wert, `0` heißt immer nebeneinander (der Hinweis unter 40 Spalten bleibt). Der Editor blendet die Bereiche Refs und Status aus, solange er offen ist, sein Bereich ist also meist das ganze Fenster. |

## `[git]`, `[credentials]`, `[ssh]`

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `git.pull_mode` | `"merge"` | Was Pull tut (Schaltfläche, `Ctrl+G p`, `Git: Pull`): `merge` (Fast-Forward, wenn möglich), `ff_only`, `rebase`. Wird git immer ausdrücklich übergeben. Das alte `fetch_all` wird als `merge` gelesen. |
| `git.push_behavior` | `"direct"` | `direct` pusht zum Standard-Remote; `ask` zeigt zuerst das Remote-Menü. |
| `git.merge_fast_forward` | `"never"` | Was ein von Ihnen gestarteter Merge (Branch-Menü, Drag-and-drop, Palette, `gitpoulpi merge`) tut, wenn sich der Branch vorspulen ließe: `never` erzeugt immer einen Merge-Commit (`--no-ff`), `allowed` spult vor, wenn möglich (Gits Standard, `merge.ff` gilt), `only` spult vor oder lehnt ab. Die eigene Wahl eines Repositorys (Einstellungen > Merge-Optionen, `merge_fast_forward` in seiner `repo-state.toml`) gewinnt; `--ff`, `--no-ff` und `--ff-only` der CLI gewinnen gegen beide. Live. |
| `git.default_encoding` | `"utf-8"` | Kodierung für Text, der **kein** gültiges UTF-8 ist (Dateiinhalte, Blame, Diffs, Commit-Nachrichten ohne Header `encoding`): jedes WHATWG-Label (`windows-1252`, `shift_jis`, ...). Nur Anzeige; Staging kodiert nie um. Ein Repository kann es überschreiben. |
| `git.lfs_enabled` | `true` | Git LFS anzeigen und steuern. Ein Repository kann es überschreiben. Siehe [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Vor einem Pull oder Push eines Repositorys, das LFS nutzt, seine LFS-Filter / den Pre-Push-Hook installieren (`git lfs install --local`, nur dieses Repository), wenn sie fehlen. Siehe [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Nicht genutzt*: `core.autocrlf` / `core.eol` eines Repositorys sind eigene Git-Einstellungen, bearbeitet unter Einstellungen > Git & Kodierung (oder `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | HTTPS-Authentifizierung für Fetch, Pull, Push, Klonen: `credential_helper` (der von git), `prompt` (Helper ignorieren; git fragt in einem Terminal, scheitert also im Hintergrund-Daemon), `none` (nur öffentliche Remotes). Alte Werte `encrypted_file` werden als `credential_helper` gelesen. |
| `ssh.key_type` | `"ed25519"` | Typ, den *Schlüsselpaar erzeugen* anlegt: `ed25519`, `rsa` (4096 Bit) oder `ecdsa` (NIST P-256); die Datei heißt `gitpoulpi_<typ>`. |
| `ssh.private_key_path`, `ssh.public_key_path` | nicht gesetzt | Von *Schlüsselpaar erzeugen* eingetragen. Ist `ssh.private_key_path` gesetzt, führen Fetch, Pull, Push und Klonen git mit `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes` aus; `ssh.public_key_path` tut dasselbe (ssh wählt den passenden Agent-Schlüssel), wenn der private Pfad leer ist. Ein `GIT_SSH_COMMAND`/`GIT_SSH` in der Umgebung oder ein `core.sshCommand` in der Git-Konfiguration (die Zuordnung pro Repository, `gitpoulpi ssh-command set --key`) gewinnt. Vor jeder Übertragung gelesen, sodass Änderungen für die nächste gelten. |

## `[[integrations]]` (Hosting und Issue-Tracker)

Geschrieben von Einstellungen > Integrationen oder `gitpoulpi hosting integrations add`.
Jede Art hat einen Client (nur persönliche Tokens, kein OAuth).

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

Ein selbst gehosteter Server kann einfaches http nutzen: `url = "http://localhost:3000"`
bleibt wie eingegeben (Schema und Port gehen in jede Anfrage ein, und ein Remote auf
diesem Host passt zum Profil). Ein über einfaches http gesendetes Token wird
unverschlüsselt übertragen, nutzen Sie http daher nur in einem lokalen oder
vertrauenswürdigen Netz.

`kind` akzeptiert außerdem `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` und `gerrit`
(Gitea und Forgejo teilen einen Client; ein Forgejo-Profil hat als Standard `https://codeberg.org`). Jira,
Trello, Linear, YouTrack und Redmine sind nur Issue-Tracker; Gerrit ist nur fürs Review (seine Changes sind die
Pull Requests, Issues hat es nicht). Bitbucket Cloud hat keinen Issue-Tracker.

`account` ist die nicht geheime Identität neben dem Token, bei den meisten Arten leer und
weggelassen: die E-Mail des Atlassian-Kontos für `jira_cloud` und `bitbucket` (ein leeres
Bitbucket-Konto bedeutet ein Zugriffstoken für Workspace oder Repository, als Bearer
gesendet), der API-Schlüssel für `trello`, der Benutzername für `gerrit` (das Token ist sein HTTP-
Passwort, gesendet als HTTP-Basic). Das Token selbst bleibt das einzige Geheimnis. Bei
`azure_devops` ist die `url` `https://dev.azure.com/<organization>`; eine URL
`https://<organization>.visualstudio.com` wird als dieselbe Organisation gelesen. Bei
`azure_devops_server` ist es die Wurzel der Collection, etwa
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (das verbundene Konto) und `ssh` (die Schlüsselwahl des Profils) werden für Sie
geschrieben. Das Token stammt aus `GITPOULPI_TOKEN_<ID>` (ID großgeschrieben,
Nicht-Alphanumerisches als `_`), sonst dem gespeicherten Token, sonst Ihrem Credential-Helper
für diesen Host, wenn nur ein Profil ihn nutzt (nur GitHub- und GitLab-Profile: die anderen
Arten lesen nie den Credential-Helper). Einzelheiten: [Hosting- und Issue-Tracker-Integrationen](hosting-integration.md).
Die alte Tabelle `[[hosting]]` ist *nicht genutzt*.

Bei `local_markdown` (ein Ordner mit Markdown-Notizen als Issue-Quelle) ist `url` der
Ordner, relativ zu jedem Repository oder absolut, und es gibt kein Token. Seine
optionale Tabelle `[integrations.local]` setzt `statuses` (die Spaltenreihenfolge des Boards,
Standard `todo`, `doing`, `done`), `done_status` (Standard `done`),
`reference_prefix` (`LOC` zeigt `LOC-12`) und `me` (wer „Mir zugewiesen“ ist).
Siehe [Lokale Issues und Obsidian](local-issues.md).

### `[integrations.radar]` (kontoweite Listen)

Optionale Tabelle eines Code-Host-Profils (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; nicht Bitbucket Cloud). Fehlt sie,
ist jeder Schlüssel aus. Geschrieben von `gitpoulpi hosting integrations radar`; gelesen von
`gitpoulpi hosting radar` und dem Radar, und bearbeitet in Einstellungen > Integrationen (der Block Radar eines verbundenen Profils).

```toml
[integrations.radar]
review_requests = true    # requests waiting for my review, every repository
open_requests = false     # every open request of my namespaces
include_drafts = false    # keep draft requests in these lists
assigned_issues = true    # open issues assigned to me, every repository
namespaces = ["acme"]     # orgs / groups to search for open requests
review_mode = "ask"       # ask | clone | api
```

| Schlüssel | Standard | Bedeutung |
|---|---|---|
| `review_requests` | `false` | Offene Requests, die um Ihr Review bitten, ob das Repository geklont ist oder nicht |
| `open_requests` | `false` | Jeder offene Request Ihres eigenen Namespace und jeder Organisation oder Gruppe, in der Sie Mitglied sind (oder der `namespaces`), nie „alles, was der Token sieht“ |
| `include_drafts` | `false` | Entwurfs-Requests bleiben aus diesen Listen draußen, solange dies nicht an ist |
| `assigned_issues` | `false` | Offene Issues, die Ihnen zugewiesen sind, in jedem Repository |
| `namespaces` | `[]` | Organisationen / Gruppen, die für `open_requests` durchsucht werden; leer heißt Ihr eigener Namespace plus jede Mitgliedschaft (GitHub: höchstens 12 werden durchsucht, GitLab: 10 Gruppen und 10 eigene Projekte, Gitea, Forgejo und Gerrit: 10; Azure DevOps nimmt Projektnamen, Bitbucket Data Center Projektschlüssel, Gerrit Projektpfad-Präfixe) |
| `review_mode` | `"ask"` | Was das Prüfen eines Requests tut, dessen Repository nicht geklont ist: `ask`, `clone` (klonen, dann prüfen) oder `api` (über die API des Hosts prüfen) |

Die Antworten werden pro Profil fünf Minuten in `radar-account.json` neben
`config.toml` zwischengespeichert; die Datei ist ein Cache und kann gefahrlos gelöscht werden.

### Der Vertrauensspeicher (nicht in `config.toml`)

Wofür jede Integration und jedes KI-Profil mit API-Schlüssel als vertrauenswürdig galt (seine Adresse,
die Konto-ID, ein angehefteter Zertifikatschlüssel, ob einfaches http erlaubt wurde), liegt in
`trust.toml` im privaten Datenordner, der nur dem Benutzer gehört (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` unter Windows, `~/.local/share/gitpoulpi/private/`
unter Linux). Er ist bewusst nicht Teil der `config.toml`: Wer die
Konfiguration bearbeiten kann, darf das Vertrauen nicht durch Bearbeiten erweitern können. Ändern Sie ihn mit
`gitpoulpi hosting integrations trust` und `gitpoulpi ai trust` (siehe
[die Sicherheitsprüfung](hosting-integration.md#die-sicherheitsprüfung)); die Datei wird
von GitPoulpi neu geschrieben, nicht von Hand.

## `[mcp]` (was KI-Agenten dürfen)

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `mcp.allow_mutations` | `false` | Aus: MCP-Tools, die irgendetwas ändern, werden abgelehnt; Lesezugriffe, `ui_*` und Vorschläge funktionieren weiter. `true` lässt Agenten das Repository ändern. |
| `mcp.mutation_allowlist` | `[]` | Nicht leer: Nur diese Tool-Namen dürfen etwas ändern (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: lehnt die `ui_*`-Tools ab, die Ihr Fenster bewegen (nicht `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (das Fenster zeigt einen Chip, den Sie bestätigen), `auto` (angewandt, außer Sie sind beschäftigt), `off`. `--ui-control` am Serverbefehl überschreibt es pro Agent. Gilt auch für `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | `ui_propose_*` / `gitpoulpi ui propose` erlauben: vorausgefüllte Dialoge, die Sie selbst bestätigen. |
| `mcp.allow_history_purge` | `false` | Nur in der `config.toml` (keine Zeile in den Einstellungen). `true` führt das Werkzeug `rewrite_purge` für Agenten auf (es braucht weiterhin `allow_mutations`). Aus sieht ein Agent das Werkzeug gar nicht: Ein Bereinigen löscht den alten Verlauf eines Umschreibens, dessen Rückgängig und seine Reflogs endgültig. CLI und TUI unterliegen dieser Sperre nicht. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` und `gitpoulpi-mcp` verweigern den Start, mit einer Meldung, die diesen Schlüssel nennt. Einstellungen > Agenten & MCP > *MCP-Server*. Wird gelesen, wenn ein Agent den Server startet. |
| `mcp.bind` | `"127.0.0.1:8808"` | Die Adresse, auf der `--http` lauscht, wenn Sie keine angeben (`--http ADDR` / `--bind` gewinnen). Nur Loopback, außer `--allow-remote`. Leer oder das alte `"stdio"` bedeuten den Standard. Einstellungen > Agenten & MCP > *Port des MCP-Servers* ändert den Port und behält den Host. |
| `mcp.http_auth` | `true` | `--http` beantwortet nur eine Anfrage mit `Authorization: Bearer <token>` (dem Token von `gitpoulpi mcp token`). `false` entfernt die Prüfung; der Server warnt dann auf stderr, und `--allow-remote` wird abgelehnt. Einstellungen > Agenten & MCP > *HTTP-Authentifizierung*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (ein einmal erzeugtes Zertifikat im privaten Datenordner, sein SHA-256-Fingerabdruck wird beim Start ausgegeben) oder `files` (`tls_cert` und `tls_key`). Bei eingeschaltetem TLS beendet der Server TLS selbst, und es gibt keinen unverschlüsselten Listener. Einstellungen > Agenten & MCP > *TLS*. |
| `mcp.tls_cert` | `""` | PEM-Zertifikatskette, die mit `tls = "files"` verwendet wird. |
| `mcp.tls_key` | `""` | PEM-Privatschlüssel, der mit `tls = "files"` verwendet wird. |
| `mcp.behind_proxy` | `false` | Sie erklären, dass ein Tunnel oder Reverse-Proxy vor dem Endpunkt die Verschlüsselung übernimmt. Das lässt sich innerhalb des Servers nicht prüfen, daher muss es für `--allow-remote` ohne TLS gesetzt sein. Der Abschnitt zwischen Proxy und Server ist in diesem Netzwerk Klartext. Einstellungen > Agenten & MCP > *Hinter Tunnel oder Reverse-Proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Nur in `config.toml`. Aus: Die MCP-Stage-Tools bieten `allow_sensitive` nicht an, und ein Staging, das eine markierte Datei hinzufügen würde (siehe `[privacy]`), wird mit einer Meldung abgelehnt, die den Agenten anweist, Sie zu fragen. An: Ein Agent darf `allow_sensitive` übergeben. Die CLI und die TUI unterliegen dem nicht: Sie fragen Sie. |

`allow_mutations`, `mutation_allowlist` und `allow_ui` sperren nur den MCP-Server; die CLI
ist Ihre eigene Shell und wird von ihnen nicht gesperrt. Siehe
[MCP und Agenten](mcp-and-agents.md#berechtigungen).

## `[ai]` (eingebaute KI, standardmäßig aus)

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `ai.enabled` | `false` | Der Schalter. Aus zeigt sich nirgends etwas von der KI, und nichts wird gesendet. |
| `ai.default_profile` | `"default"` | ID des Profils, das Aktionen nutzen, wenn sie keines nennen. |
| `ai.global_instructions` | `""` | Text, der dem Prompt jeder Aktion hinzugefügt wird. Darf `${repo}`, `${uilang}`, ... nutzen (siehe [KI > Prompt-Variablen](ai.md#prompt-variablen)); dasselbe gilt für die Anweisungen pro Aktion und pro Repository. |
| `ai.actions.<action>.profile` | `""` | Profil für eine Aktion; leer = der Standard. Aktionen: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. Ein Workspace (`workspaces.toml`, `[workspace.ai_instructions]` und `[workspace.ai_overrides.<action>]`) und ein Repository können diese Anweisungen ergänzen oder ersetzen: siehe [KI](ai.md#anweisungen). |
| `ai.actions.<action>.instructions` | `""` | Text, der nur für diese Aktion hinzugefügt wird. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` oder `no`: ob der lokale Agent eines Code-Review-Entwurfs für diesen Lauf im Repository arbeitet und dabei das `workdir` des KI-Profils überstimmt. Ein Repository kann ein eigenes setzen (`gitpoulpi ai review-context --repo`). Siehe [KI > Review-Kontext](ai.md#review-kontext). |
| `ai.review.include_claude_md` | `true` | Einem Code-Review-Entwurf die `CLAUDE.md` des Repositorys geben (Wurzel, gelesen auf dem Branch, auf den die Anfrage zielt; bei 8 KB gekürzt). |
| `ai.review.include_agents_md` | `true` | Dasselbe für `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Eine weitere Datei für einen Code-Review-Entwurf: ein Pfad relativ zur Repository-Wurzel, ein absoluter Pfad oder ein Link auf eine Datei eines Code-Hosts (gelesen über das passende Integrationsprofil, sonst anonym). Alle Dateien zusammen werden bei 16 KB gekürzt. |
| `ai.privacy.redact_secrets` | `true` | Ersetzt jedes Geheimnis, das der Scanner in einer Anfrage erkennt (ein Token, einen privaten Schlüsselblock, ein Passwort in einer URL, einen `password = ...`-Wert, der zufällig aussieht), durch `[REDACTED:<rule>]`, bevor sie hinausgeht. Einstellungen > KI zeigt eine rote Warnung, solange es aus ist. |
| `ai.privacy.exclude_sensitive_files` | `true` | Lässt den Diff jeder Datei, die zu `privacy.sensitive_patterns` passt, aus dem Gesendeten heraus. Ein lokaler Agent, der im Repository-Ordner arbeitet, kann Dateien weiterhin selbst lesen: siehe [KI > Datenschutz](ai.md#datenschutz). |
| `ai.limits.conflict_bytes` | `40000` | Höchstzahl Bytes, die ein Merge-Konflikt (seine Blöcke Basis, aktuell und eingehend zusammen) haben darf, wenn „Lösung vorschlagen“ ihn sendet. Darüber schlägt GitPoulpi nicht fehl: Es warnt und bietet *Trotzdem senden* (nur diese Anfrage), *Einstellung öffnen* und *Abbrechen* an; ein Konflikt wird nie gekürzt. Das Limit ist GitPoulpis eigene Schranke, nicht die des Modells: Erhöhen Sie es, wenn Ihr Modell genug Platz hat. Zwischen 1000 und 50000000 gehalten; einstellbar unter Einstellungen > KI > Limits. Die übrigen Limits unten funktionieren genauso und stehen alle in dieser Gruppe. |
| `ai.limits.diff_bytes` | `60000` | Höchstzahl Bytes des Diffs, den eine Nachricht zu einem Commit, eine Erklärung, ein Stash-Name, eine Pull-Request-Beschreibung, ein Commit-Plan oder eine Commit-Aufteilung tragen darf, und des gesamten Diffs eines Code-Reviews. Darüber wird nichts stillschweigend gekürzt: In der TUI bietet *Über dem KI-Limit* *Alles senden* (nur diese Anfrage, nichts gekürzt), *Gekürzt senden* (das Ende wird ausgelassen, wie bisher), *Einstellung öffnen* und *Abbrechen* an; CLI und MCP kürzen und melden es in `truncated` (`--send-all` sendet alles). Zwischen 1000 und 50000000 gehalten. |
| `ai.limits.review_file_bytes` | `12000` | Höchstzahl Bytes des Diffs einer Datei in einem Code-Review. Darüber dieselbe Frage und dieselben zwei Antworten; *Gekürzt senden* kürzt diese Datei mit einem sichtbaren Hinweis. Gleicher Bereich. |
| `ai.limits.review_text_bytes` | `4000` | Höchstzahl Bytes der Beschreibung des begutachteten Pull Requests. Darüber kürzt *Gekürzt senden* die Beschreibung mit einem Hinweis `[text cut]`. Gleicher Bereich. |
| `ai.limits.review_threads_bytes` | `6000` | Höchstzahl Bytes der Liste bereits vorhandener Review-Kommentare (damit das Modell sie nicht wiederholt). Darüber kürzt *Gekürzt senden* die Liste. Gleicher Bereich. |
| `ai.limits.review_context_file_bytes` | `8192` | Höchstzahl Bytes einer Projekt-Kontextdatei (`CLAUDE.md`, `AGENTS.md`, die zusätzliche Datei), die ein Code-Review erhält. Darüber kürzt *Gekürzt senden* die Datei mit einem Hinweis. Gleicher Bereich. |
| `ai.limits.review_context_bytes` | `16384` | Höchstzahl Bytes aller Kontextdateien zusammen. Darüber kürzt *Gekürzt senden* die Datei, die das Limit überschreitet, und lässt die danach aus. Gleicher Bereich. |
| `ai.sent_preview.<action>` | `true` | Ein Schlüssel je Aktion (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): „Was gesendet wird“ anzeigen, bevor die Anfrage hinausgeht. „Für ... nicht mehr anzeigen“ im Dialog schreibt `false`; Einstellungen > KI > *Gesendetes anzeigen vor...* schaltet sie wieder ein. |

Die eigene Tabelle `[preferences.ai_review]` eines Repositorys (in seiner `repo-state.toml` oder, bei einem nicht
geklonten Repository, der Eintrag in `remote-reviews.toml`) nimmt dieselben Schlüssel, alle optional, plus
`no_extra_file = true`: keine zusätzliche Datei für dieses Repository, was auch immer `ai.review.extra_file` sagt. Ein
nicht gesetzter Schlüssel, ein nicht gesetztes `extra_file` und ein fehlendes `no_extra_file` erben den globalen Wert;
`extra_file` auf eine Datei zu setzen löscht den Schalter und umgekehrt.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness` wählt die Vorschläge, die Einstellungen > KI anbietet (Argumente ohne
Oberfläche, Optionen, Modelloption und -liste, Umgebungsvariablen) unabhängig von
`command`; beide Schlüssel werden aus der Datei weggelassen, wenn sie leer sind.
Umgebungswerte werden nie protokolliert und in den Einstellungen verborgen, wenn sie wie
Geheimnisse aussehen.

Einstellungen > KI bietet Voreinstellungen für `harness`/`command`/`args`/`env` (die
gängigen Agenten-CLIs mit ihren Optionen ohne Oberfläche, ihrer Modelloption und nützlichen
Umgebungsvariablen) und für `kind`/`base_url`/`model` (die gehosteten und lokalen
HTTP-Anbieter); siehe [KI > Die KI konfigurieren](ai.md#die-ki-konfigurieren).

Ein Profil namens *Default* wird angelegt, wenn es keines gibt. Alte Dateien mit
`[[ai.presets]]` / `[ai.defaults]` werden beim Laden migriert und in `[[ai.legacy_presets]]`
aufbewahrt. Alles andere: [KI-Unterstützung](ai.md).

## `[repos]`, `[session]`

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `repos.clone_root` | `""` | Wo der Klon-Dialog beginnt; leer = `%USERPROFILE%\source\repos` oder `~/src`. |
| `repos.clone_start_source` | `"last"` | Die Quelle, bei der der Klon-Dialog beginnt: `last` (Standard, oder leer) = die Quelle, die beim letzten Schließen angezeigt wurde, `url` = immer Per URL klonen, oder die ID eines Integrationsprofils (`id` in seinem `[[integrations]]`-Eintrag). Ein Profil, das fehlt, deaktiviert oder nicht verbunden ist, fällt auf Per URL klonen zurück. Die letzte Quelle selbst ist Zustand und steht in `clone-source.toml` im Konfigurationsordner, nicht hier. |
| `repos.browse_root` | `""` | Wo Durchsuchen sich öffnet; leer = der zuletzt gewählte Ordner, dann Home. |
| `repos.show_hidden` | `false` | Die Ordnerauswahl zeigt versteckte Ordner (`Alt+H` schaltet um und speichert es). |
| `repos.show_siblings` | `false` | Der linke Baum der Ordnerauswahl listet auch die anderen Ordner jeder geöffneten Ebene, als echte Hierarchie (`Alt+S` schaltet um und speichert es). |
| `session.per_repo_state_dir` | `".gitpoulpi"` | *Nicht genutzt*: Der Ordner ist immer `.gitpoulpi`. |

## `[log]` (das Anwendungsprotokoll)

Was der Tab „Anwendung“ des Aktivitätsprotokolls zeigt und was „Diagnose kopieren“ zitiert. Einzelheiten: [Wenn etwas fehlschlägt](troubleshooting.md#wenn-etwas-fehlschlägt).

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `log.level` | `"info"` | Die ausführlichste Zeile, die noch aufgezeichnet wird: `error`, `warn`, `info`, `debug` oder `trace`. Das Stufenfeld im Aktivitätsprotokoll ändert sie (und speichert sie hier). |
| `log.retention_days` | `7` | Tage, für die Protokolldateien aufbewahrt werden, der heutige eingeschlossen: 1 bis 3650, zehn Jahre (ein größerer oder kleinerer Wert wird begrenzt). Immer eine Anzahl Tage, in welcher Einheit Einstellungen sie auch zeigt. Ältere Dateien werden beim Start von GitPoulpi gelöscht. Die Datei eines Tages ist außerdem durch `log.max_file_mb` begrenzt, der ungünstigste Fall liegt also bei etwa `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | Die Einheit, in der Einstellungen `log.retention_days` zeigt: `days`, `weeks`, `months`, `years` oder `auto`. Nur Anzeige, sie ändert nie, wie lange Dateien aufbewahrt werden. Eine Woche sind 7 Tage, ein Monat 30 Tage (kein Kalendermonat, also immer dieselbe Zahl von Tagen, wann auch angewendet) und ein Jahr 365 Tage. `auto`, und eine Konfiguration aus der Zeit vor diesem Schlüssel, wählt die größte Einheit, die die Tage ohne Rest teilt (14 Tage lesen sich als „2 Wochen“, 45 Tage als „45 Tage“). Eine Einheit, die die Tage nicht teilt (`retention_days = 10` mit `weeks`), wird ignoriert, und die Tage werden gezeigt. Einstellungen schreibt beide Schlüssel; ein älteres GitPoulpi ignoriert die Einheit und liest weiter die Tage. |
| `log.max_file_mb` | `5` | Die Datei eines Tages wird bei dieser Größe in eine einzige `.old`-Sicherung rotiert, ein Tag belegt also nie mehr als das Doppelte. |
| `log.max_file_unit` | `"auto"` | Die Einheit, in der Einstellungen die Größe zeigt: `mib`, `gib` (1024 MiB) oder `auto`. Nur Anzeige: Die Größe sind immer `max_file_mb` MiB, 1 bis 1024. |
| `log.timings` | `true` | Der Tab Git-Vorgänge des Aktivitätsprotokolls zeigt, wie lange jeder Vorgang gedauert hat (`Push - 123 ms`: Repository öffnen, Aktualisieren, Fetch, Pull, Push, Commit, Checkout, Merge, Rebase, Stash, Klonen). Die Dauer misst das Fenster, von der Anfrage bis zur Antwort des Daemons, sie enthält also die IPC-Laufzeit. Aus: keine Dauer und keine Einträge für erfolgreiche Vorgänge. Fehlschläge werden immer aufgelistet. |
| `log.extended` | `false` | „Erweiterte Protokollierung“: Der Tab Git-Vorgänge listet auch die Hintergrundvorgänge (die periodische Aktualisierung, die den Status abfragt, Auto-Fetch) mit ihrer Dauer auf. Sie werden immer auf Stufe `debug` ins Anwendungsprotokoll geschrieben. |
| `log.commit_count` | `"system-git"` | Wie die Statuszeile die Gesamtzahl für `geladen / gesamt` ermittelt. `"system-git"` führt `git rev-list --count` aus (ein Prozess, auf einem normalen Rechner am schnellsten: Git liest die Commit-Graph-Datei). `"libgit2"` zählt im Prozess mit einem libgit2-Revwalk und startet nichts: Wählen Sie es, wenn jeder Befehlszeilenprozess langsam startet, typischerweise weil eine Antivirensoftware mit Echtzeitprüfung jeden einzelnen untersucht; bei 50 000 Commits dauert es im Hintergrund etwa 200 ms statt 12 ms, aber es wird kein Prozess gestartet. `"off"` zählt nie: Die Statuszeile zeigt nur die geladenen Commits. Wirkt sofort. Eine nach Pfad gefilterte Zählung nutzt immer das System-Git. |
| `log.use_mailmap` | `true` | Jeden Autor im Log, in Blame und im Dateiverlauf so zeigen, wie ihn die `.mailmap` des Repositorys (und `mailmap.file`) nennt: Der Daemon fragt Git nach `%aN`/`%aE` (System-Backend) oder wendet dieselben Dateien selbst an (libgit2-Backend), die Antwort ist also identisch. Die gespeicherten Commits werden nicht geändert und nichts wird umgeschrieben. Die Suche im Log nach Autor findet die Namen, wie sie angezeigt werden. Wirkt sofort; das Log liest innerhalb seiner 2-s-Abfrage neu. |

Die Dateien heißen `gitpoulpi-YYYY-MM-DD.log` und liegen im Statusordner des Betriebssystems (`~/.local/state/gitpoulpi/logs` unter Linux; `<cache>\logs` unter Windows und macOS; `<config>/cache/logs` mit `GITPOULPI_CONFIG_DIR`). Tokens, Passwörter und `Authorization`-Header werden maskiert, bevor etwas geschrieben wird.

## `[search]` (die Suche im Workspace)

Was `gitpoulpi search` (und sein MCP-Tool) durchsucht. Details: [Workspaces und Suche in der CLI](cli-reference.md#workspaces-und-suche).

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (neueste zuerst, über jeden Branch, Remote-Branch und Tag), deren Nachrichten der Suchindex pro Repository im Speicher hält, je etwa 100 Byte plus die Nachricht. Ein größerer Verlauf wird dort abgeschnitten, und die Antwort sagt, dass sie `partial` ist. `0` schaltet die Commit-Suche aus (Branches, Tags, Stashes und Worktrees werden weiter durchsucht). Live gelesen: Die nächste Suche befolgt eine Änderung. |

## `[rewrite]` (Verlauf umschreiben)

Was `gitpoulpi rewrite` und die Werkbank zum Umschreiben des Verlaufs schützen. Details: [Verlauf umschreiben](history-rewrite.md).

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branches (Namen oder Muster, bei denen `*` beliebigen Text trifft), die ein Umschreiben in Ruhe lässt, wenn sie schon gepusht sind, außer der Lauf nennt sie (`--include-protected BRANCH`; `--include-all-protected` für den Push; das Kästchen pro Branch in der Werkbank). Kommt zu den Basen hinzu, die die `workflow.toml` eines Repositorys als `protected` markiert. Der Push eines Umschreibens überspringt sie ebenso. Das ist der eigene Schutz von GitPoulpi: Ob der Force-Push angenommen wird, entscheidet weiterhin der Branch-Schutz des Hosts. Wirkt sofort: Die nächste Vorschau beachtet eine Änderung. |
| `rewrite.external_scripts` | `["shell"]` | Welche externen Skriptsprachen die Skriptregel verwenden darf: beliebig aus `"shell"` (bash oder sh; pwsh oder powershell für PowerShell-Skripte), `"python"`, `"lua"` (ein installiertes Lua; das eingebaute Lua ist immer an und braucht keinen Eintrag). **Ein externes Skript hat keine Sandbox: Es läuft mit Ihren Rechten.** Eine Sprache, die nicht in der Liste steht, ist in der Werkbank ausgegraut und wird von CLI, MCP und Daemon abgelehnt (`scripts_disabled`). Auch wenn sie in der Liste steht, läuft ein externes Skript aus dem Repository oder einem geteilten Rezept erst, nachdem Sie ihm vertraut haben (`gitpoulpi rewrite script trust FILE` oder die Frage in der Werkbank); das Vertrauen hängt am Inhalts-Hash des Skripts, ein geändertes Skript fragt also erneut. Python schalten Sie in der `config.toml` ein (eine Liste lässt sich nicht mit `config set` setzen): `[rewrite]`, dann `external_scripts = ["shell", "python"]`. Wirkt sofort. |
| `rewrite.script_timeout_ms` | `5000` | Wie lange ein externes Skript Zeit hat, einen Commit zu beantworten. Bei einer Zeitüberschreitung wird der ganze Prozessbaum beendet und das Umschreiben schlägt fehl (`script_failed`, mit dem Commit); nichts hat sich bewegt. `0` gilt als der Standard. Wirkt sofort. |
| `rewrite.script_timeout_unit` | `"auto"` | Die Einheit, in der Einstellungen das Zeitlimit zeigt: `milliseconds`, `seconds` oder `auto`. Nur Anzeige: Das Limit sind immer `script_timeout_ms` Millisekunden. Einstellungen akzeptiert 1 bis 600000 (zehn Minuten); `0` gilt weiter als der Standard. Wirkt sofort. |
| `rewrite.script_total_timeout_s` | `3600` | Wie lange ein Durchlauf eines externen Skripts insgesamt dauern darf. `0` gilt als der Standard. Wirkt sofort. |
| `rewrite.script_total_timeout_unit` | `"auto"` | Die Einheit, in der Einstellungen das Zeitlimit zeigt: `seconds`, `minutes`, `hours` oder `auto`. Nur Anzeige: Das Limit sind immer `script_total_timeout_s` Sekunden. Einstellungen akzeptiert 1 bis 86400 (ein Tag); `0` gilt weiter als der Standard. Wirkt sofort. |

## `[updates]` (die Suche nach einem neueren Release)

Was der Versions-Chip der Titelleiste bei neueren Releases tut. Details: [Updates](updates.md).

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` fragt einmal täglich im Hintergrund bei GitHub nach, ob es ein neueres Release gibt, und macht aus dem Versions-Chip `vX.Y.Z verfügbar`. `"auto"` lädt das Update zusätzlich von selbst herunter und legt es bereit, wenn dieser Build aus einem Release-Archiv stammt (vor dem Neustart wird weiterhin gefragt); bei jedem anderen Build verhält es sich wie `"notify"`. `"off"` stellt nie eine Anfrage. Wirkt sofort. |
| `updates.channel` | `"stable"` | `"stable"` berücksichtigt nur vollständige Releases; `"prerelease"` bietet auch Release Candidates an (`-rc.1`). Wirkt sofort. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | Das `owner/name` auf github.com, dessen Releases geprüft werden. Solange dieses Repository privat ist, antwortet GitHub mit 404, sofern kein Token für github.com gespeichert ist (Einstellungen > Integrationen); ohne eines scheitert die Prüfung leise und schreibt eine Zeile ins Anwendungsprotokoll. |

Ein Administrator kann `mode`, `channel` (als `release_channel`) und `repo` sowie einige weitere Update-Optionen in einer computerweiten Richtliniendatei oder der Windows-Registry festlegen. Ein Schlüssel, den die Richtlinie setzt, gewinnt gegenüber der `config.toml` und erscheint in den Einstellungen als **Vom Administrator festgelegt**. `mode` in der Richtlinie ist eine Obergrenze: Sie können einen niedrigeren Wert wählen, nie einen höheren. Siehe [Unternehmensverteilung](enterprise-distribution.md#die-richtliniendatei-und-die-registry).

## `[privacy]` (sensible Dateien)

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Pfadmuster im gitignore-Stil. Ein Muster ohne `/` passt auf den Dateinamen in beliebiger Tiefe, eines mit `/` auf den ganzen Pfad (ein führendes `/` verankert es an der Wurzel); `*` und `?` bleiben innerhalb eines Ordners, `**` überschreitet Ordner; ein `!` erlaubt wieder, was ein früheres Muster erfasst hat, und die letzte Übereinstimmung gewinnt; Groß-/Kleinschreibung wird ignoriert. Wird vom Staging-Schutz und von `ai.privacy.exclude_sensitive_files` verwendet. |
| `privacy.staging_guard` | `true` | Beim Stagen einer Datei, die zu einem Muster passt oder deren Inhalt ein erkanntes Geheimnis enthält, wird zuerst gefragt (TUI), es wird abgelehnt, außer `--allow-sensitive` ist angegeben (CLI) oder `mcp.allow_sensitive_staging` erlaubt dem Agenten, `allow_sensitive` zu übergeben (MCP). Textdateien bis zu 1 MiB je Datei und 32 MiB insgesamt werden gelesen; Binärdateien werden nur nach Namen geprüft. Siehe [Staging und Committen](staging-and-committing.md#sensible-dateien). |

## `[conflict_prevention]` (Basis-Branch-Listen)

Welche Branches die proaktive Konflikterkennung als Basis des aktuellen Branches annehmen darf, ist eine Liste von Mustern (siehe [Merge-Konflikte lösen](merge-conflicts.md#konfliktvermeidung)). Listen können benannt und geteilt werden; Einstellungen > Konfliktvermeidung bearbeitet sie.

| Schlüssel | Standard | Werte und Wirkung |
|---|---|---|
| `conflict_prevention.default_list` | `""` | Der Name der Liste, die ein Repository verwendet, wenn es keine eigene hat. Leer oder ein Name, den keine Liste hat, bedeutet das integrierte `**` (jeder Branch). Wird bei jeder Prüfung neu gelesen, eine Änderung gilt also für geöffnete Repositories ohne Neustart. |
| `conflict_prevention.lists` | keine | Die benannten Listen, je eine Tabelle: `[conflict_prevention.lists.release]` mit `patterns = ["origin/release/**", "!origin/release/old"]`. Leere `patterns` erlauben jeden Branch. Das Bearbeiten einer Liste gilt für jedes Repository, das ihr folgt. |

Welcher Liste ein Repository folgt, steht in seiner `repo-state.toml` (unten). Vorrang: die eigenen Muster des Repositorys, dann die benannte Liste, der es folgt, dann `default_list`, dann `**`.

## `[keybindings]`

Belegt die Tastenkürzel der TUI neu; Einstellungen > Tastatur & Maus bearbeitet sie.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<Kontext>] <Aktion> = "<Tastenfolge>"`; ein Wert ersetzt die Standards dieser
Aktion, das Entfernen der Zeile stellt sie wieder her. Tastenfolgen: `Ctrl+Shift+N`,
`Alt+n`, `F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Ein fehlerhafter Eintrag
blockiert nie etwas: Die TUI ignoriert ihn, behält den Standard und sagt das in der
Statuszeile. Kontexte, Aktions-IDs, Standards und die Konfliktregeln:
[Tastenkürzel](keybindings.md).

## Einstellungen pro Repository: `.gitpoulpi/repo-state.toml`

Im Repository aufbewahrt; der Ordner ignoriert sich selbst (eine `.gitignore` mit `*`),
sodass er nie als Änderung erscheint. Von der TUI und der CLI geschrieben; Sie bearbeiten
ihn selten.

| Schlüssel | Was es ist | Gesetzt mit |
|---|---|---|
| `[preferences] encoding` | Überschreibt hier `git.default_encoding` | Einstellungen > Git & Kodierung |
| `[preferences] lfs_enabled` | Überschreibt hier `git.lfs_enabled` | Einstellungen > Git & Kodierung |
| `[preferences] profile` | Die ID aus `[[profiles]]`, die hier Commits signiert; fehlend bedeutet `general.active_profile` | das Profil-Kombinationsfeld der Symbolleiste, **Repository: Profil wechseln...** |
| `[preferences] autocrlf`, `eol` | *Noch nicht genutzt* | Einstellungen > Git & Kodierung |
| `[preferences] push_after_commit` | Den Branch nach jedem Commit an seinen Upstream pushen (aus, wenn nicht vorhanden) | Einstellungen > Repository-spezifische Einstellungen > Commit |
| `[preferences] skip_hooks_by_default` | „Hooks überspringen“ des Commit-Formulars ist vorab angehakt (aus, wenn nicht vorhanden) | Einstellungen > Repository-spezifische Einstellungen > Commit |
| `[preferences] squash_merge` | Der Merge eines lokalen Branches macht einen Squash: stagt, kein Merge-Commit (aus, wenn nicht vorhanden) | Einstellungen > Repository-spezifische Einstellungen > Merge-Optionen |
| `[preferences] template_apply`, `template_strip_comments` | Ein leeres Commit-Formular mit der Vorlage vorbelegen; `#`-Kommentarzeilen beim Commit entfernen (beide aus, wenn nicht vorhanden) | Einstellungen > Repository-spezifische Einstellungen > Nachrichtenvorlage |
| `[preferences] conflict_detection` | Proaktive Konflikterkennung für dieses Repository; fehlend folgt `general.conflict_detection` | Einstellungen > Repository-spezifische Einstellungen > Konfliktvermeidung |
| `[preferences] conflict_base_branches` | Die eigenen Muster des Repositorys für Basis-Branches, eines pro Zeile (weiter als **Eigene Liste** verwendet) | Einstellungen > Repository-spezifische Einstellungen > Konfliktvermeidung |
| `[preferences] conflict_base_source` | Woher die Basis-Branches kommen: `default` (die globale Standardliste), `list` (`conflict_base_list`) oder `own` (`conflict_base_branches`). Fehlend: die eigenen Muster, wenn es welche gibt, sonst der Standard | Einstellungen > Repository-spezifische Einstellungen > Konfliktvermeidung |
| `[preferences] conflict_base_list` | Die `[conflict_prevention.lists.<name>]`, der dieses Repository folgt, wenn die Quelle `list` ist; eine gelöschte Liste fällt auf den Standard zurück | Einstellungen > Repository-spezifische Einstellungen > Konfliktvermeidung |
| `commit-template.txt` (eine Datei neben `repo-state.toml`) | Die Commit-Vorlage; `commit.template` von Git zeigt darauf | Einstellungen > Repository-spezifische Einstellungen > Nachrichtenvorlage |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: die KI-Anweisungen dieses Repositorys | Einstellungen > Repository-spezifische Einstellungen > KI |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` für eine Aktion hier | Einstellungen > Repository-spezifische Einstellungen > KI |
| `default_remote` | Das Remote, das Push/Pull/Fetch per Klick nutzen | Remote-Menü, `gitpoulpi remote default <name>` |
| `hosting_integration` | Das hier festgelegte Profil aus `[[integrations]]` | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Das Anbieter-Projekt, das die PR- / Issue-Listen lesen, als `{ integration, project, filter }`. `filter` ist optional: eine anbietereigene Zusatzklausel (ein Jira-JQL-Fragment, eine Azure-WIQL-Bedingung, Trello-Listennamen), per UND an die Listenabfrage gehängt; GitHub und GitLab ignorieren sie. | Abschnitte der Seitenleiste, `gitpoulpi hosting link` |
| `hidden_remotes` | Als Ganzes im Graph ausgeblendete Remotes (jeder Branch, auch die später abgerufenen) | das Auge und das Menü des Remotes, `gitpoulpi refs hide --remote` |
| `ref_visibility` | Ausgeblendete / solo gestellte Refs | Refs-Seitenleiste, `gitpoulpi refs hide\|show\|solo` |
| `folds`, `collapsed_*_ref_folders` | Eingeklappte Abschnitte der Seitenleiste sowie Branch- und Tag-Ordner. `[folds]` enthält auch `status_folders` (eingeklappte Ordner der Dateibäume Unstaged / Staged) und `commit_folders` (eingeklappte Ordner des Baums der geänderten Dateien eines Commits, für jeden Commit derselbe), jeweils auf 500 Pfade begrenzt, die ältesten fallen zuerst weg | die Seitenleiste, die Dateibäume |
| `pending_auto_stash` | Ein Stash, den GitPoulpi erstellt und nicht zurückgespielt hat | vom Daemon geschrieben; siehe [Auto-Stash](log-and-history.md#auto-stash) |

## Weitere Umgebungsvariablen

| Variable | Wirkung |
|---|---|
| `GITPOULPI_CONFIG_DIR` | Der Konfigurationsordner (siehe oben). |
| `GITPOULPI_TOKEN_<ID>` | Ein Hosting-Token für das Profil `<ID>`, vor dem gespeicherten (CI, einmalige Nutzung). |
| `GITPOULPI_GIT` | Die auszuführende `git`-Datei (Standard: `git` im `PATH`; unter Windows das echte `mingw64\bin\git.exe` hinter dem Shim `cmd\git.exe` von Git for Windows, was pro Aufruf einen Prozess spart). |
| `GITPOULPI_LANG=fr` | Die Sprache der Oberfläche, vor `ui.language` (jeder Wert von `ui.language`; `qps` ist ein Pseudo-Locale für Übersetzer). Ein unbekannter oder leerer Wert wird ignoriert. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Überhaupt keine Bilder, was auch immer die Konfiguration sagt. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Hinweise zur Bildauflösung (Zellgröße in Pixeln; Oversampling, Standard 2, `1` spart CPU). |
| `GITPOULPI_NO_BROWSER=1` | Nie einen Webbrowser starten (die URL wird trotzdem angezeigt). |
| `GITPOULPI_TIMING=1` | Anfragezeiten nach `gitpoulpi-timing.log` im temporären Ordner schreiben (Leistungsberichte). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Nicht auf Enter warten, nachdem die TUI bei einem Fehler beendet wurde. |
