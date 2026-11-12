[English](../open-in.md) · [Čeština](../cs/open-in.md) · **Deutsch** · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Öffnen in

Die Schaltfläche **Öffnen in ▾** in der Titelleiste, neben dem Repository-Pfad, öffnet den Ordner des aktiven Tabs
anderswo: in Ihrem Terminal, einem neuen Multiplexer-Tab oder -Bereich, einem
Editor, dem Dateimanager. Sie bestimmen, was in der Liste steht. Nichts ist fest
verdrahtet: Jeder Eintrag besteht aus einer Bezeichnung und einem Befehl.

## Benutzung

- **Klicken Sie auf Öffnen in ▾**: Eine Liste klappt auf. Ein Klick auf einen
  Eintrag führt ihn aus. Der Standardeintrag ist mit `•` markiert; die letzte
  Zeile, **Konfigurieren...**, öffnet die [Einstellungen](settings.md#öffnen-in) auf der Seite Öffnen in.

  ![Die Liste Öffnen in: Terminal, zwei Editoren, ein Git-Werkzeug und der Dateimanager, der Standard mit einem Punkt markiert, und am Ende Konfigurieren](../images/open-in-menu.png)

- **Tastatur**: `Alt+O` öffnet dieselbe Liste (`Up`/`Down`, `Enter`, `Esc`; bei
  mehr als sieben Einträgen tippen Sie zum Suchen). `Ctrl+Alt+O` führt den
  Standardeintrag sofort aus. Palette (`Ctrl+P`): `Tab: Öffnen in...` und
  `Tab: Im Standardeintrag öffnen`. Die Palettenbefehle funktionieren auch, wenn die
  Titelleiste keinen Platz für die Schaltfläche hat. Beide Tasten lassen sich
  neu belegen; siehe [Tastenkürzel](keybindings.md).
- **Rechtsklick auf einen Repository-Tab**: Das Tab-Menü hat ein Untermenü
  **Öffnen in** mit denselben Einträgen.
- Die Liste beginnt bei dem Eintrag, den Sie in dieser Sitzung zuletzt benutzt
  haben.
- Die Statuszeile zeigt sofort „Wird in ... geöffnet...“. Scheitert das Programm innerhalb
  von etwa 1,5 Sekunden (zum Beispiel tmux ohne Server oder eine falsche
  Option), erscheint die erste Zeile seiner Fehlerausgabe in der Statuszeile.
  Ein nicht installiertes Programm wird sofort genannt.

Bis Sie Ihre eigene Liste speichern, wird der Standardeintrag nach dem gewählt, was erkannt wird: innerhalb von herdr „Herdr: new tab“; unter Omarchy das neue Fenster von kitty (kitty ist dessen Terminal); sonst der Dateimanager.

Die Schaltfläche erscheint auf jedem Repository-Tab, auch im Ordner, in dem
GitPoulpi gestartet wurde, und im Modus `ui.tab_dir_follow = "auto"`. Im Modus
`auto` läuft der Standardeintrag außerdem von selbst, wenn Sie einen neuen Tab
öffnen (nicht für die beim Start wiederhergestellten Tabs). Siehe
[Dem Ordner des aktiven Tabs folgen](following-the-tab-folder.md).

## Wie ein Eintrag aussieht

In der `config.toml` eine Tabelle `[[ui.open_in]]` pro Eintrag:

```toml
[ui]
open_in_default = "Herdr: new tab"   # a label; empty means the first entry

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Sie schreiben das selten von Hand: Die Einstellungsseite bearbeitet es. Jeder
Schlüssel steht in der [Konfigurationsreferenz](config-reference.md).

### Befehle und Variablen

Der Befehl wird zuerst in Argumente zerlegt (Leerzeichen, `'...'` und `"..."`
fassen Wörter zusammen), dann werden die Variablen in jedem Argument ersetzt.
Ein Ordner mit einem Leerzeichen im Namen bleibt also ein einziges Argument.
**Es ist keine Shell beteiligt**, es sei denn, Ihr Befehl startet eine.

| Variable | Wird ersetzt durch |
|---|---|
| `${cwd}` | Der Ordner des Tabs. |
| `${repo}` | Die Wurzel des Repositorys (weicht bei einem Worktree-Tab von `${cwd}` ab). |
| `${name}` | Der Basisname des Ordners. |
| `${branch}` | Der ausgecheckte Branch; leer bei losgelöstem HEAD. |
| `${web_url}` | Die Webseite des Remotes origin, wenn es GitHub oder GitLab ist; sonst leer. |
| `${env:NAME}` | Die Umgebungsvariable `NAME`; leer, wenn nicht gesetzt. |

Das alte `{repo}` funktioniert weiterhin und bedeutet `${cwd}`. Jedes andere
`${...}` ist ein Fehler, der die Variable nennt.

Das Programm wird mit dem Ordner des Tabs als Arbeitsverzeichnis gestartet und
erhält drei Umgebungsvariablen, praktisch für ein Shell-Snippet:

| Variable | Wert |
|---|---|
| `GITPOULPI_CWD` | Der Ordner des Tabs. |
| `GITPOULPI_REPO` | Die Wurzel des Repositorys. |
| `GITPOULPI_BRANCH` | Der Branch, oder leer. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

Unter Windows wird das Programm über `PATH` und `PATHEXT` gesucht, sodass `code`
und `cursor` (`.cmd`-Dateien) funktionieren und kein Konsolenfenster aufblitzt.

## Voreinstellungen

Einstellungen > Öffnen in > **Voreinstellung hinzufügen** listet fertige Einträge
nur für Ihr Betriebssystem (Windows-Voreinstellungen erscheinen nie unter Linux
und so weiter), nach Kategorie: Die Zeile beginnt mit der Kategorie,
`Explorer: ...`, `Multiplexer: ...`, `Terminal: ...`, `Editor: ...`,
`Git-Werkzeug: ...`, `KI-Agent: ...`, `Web: ...`. Ganz oben stehen die **Erkannt**-Einträge der berechneten
Standardliste. Innerhalb jeder Kategorie kommen zuerst die sofort nutzbaren
Voreinstellungen (Sie laufen in diesem Multiplexer oder Terminal, und das
Programm liegt im `PATH`), dann die installierten, dann der Rest. Tippen Sie in
das Kombinationsfeld, um zu suchen, zum Beispiel `lazygit` oder `Git-Werkzeug:`.
**Im aktuellen Tab testen** probiert einen Eintrag aus, bevor Sie sich darauf
verlassen.

![Die Liste + Hinzufügen der Seite Öffnen in: erkannte Einträge zuerst, dann Explorer, Multiplexer und die anderen Kategorien, jede Voreinstellung mit ihrer Kategorie als Präfix](../images/settings-open-in-presets.png)

Voreinstellungen vom Typ „Werkzeug in einem Bereich ausführen“ (`lazygit in
Herdr tab`, `yazi in tmux window`, `Claude Code in Herdr split`...) werden nur
angeboten, wenn der Host (herdr, tmux, zellij, WezTerm, kitty,
`xdg-terminal-exec`, Windows Terminal) und das Werkzeug installiert sind.
`$EDITOR in ...` startet den Editor, den Ihre Variable `EDITOR` nennt (mit
Rückfall auf `vi` in herdr), in herdr oder tmux.

Die Werkzeug-Voreinstellungen von herdr erstellen den Tab oder Split und tippen
dann den Befehl in dessen Shell, sodass das Werkzeug im Ordner startet und der
Bereich nach seinem Ende offen bleibt. Die `KI-Agent`-Einträge nutzen
`herdr agent start`, sodass herdr den Zustand des Agenten verfolgt.

| Kategorie | Voreinstellungen (`V` = durch Ausführen oder anhand der installierten Hilfe geprüft) |
|---|---|
| Explorer | Windows `explorer.exe` (V), macOS Finder (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger in einem Bereich/Tab/Terminal |
| Multiplexer | herdr Tab, Workspace, Split rechts/unten (V); tmux Fenster, Split rechts/unten, abgetrennte Sitzung (V); zellij Bereich, Tab, schwebend; WezTerm Tab, Split, neues Fenster; kitty Tab, Split-Fenster, OS-Fenster (V: Optionen aus der Hilfe; braucht `allow_remote_control`); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (Fenster, Tab, Split und je ein Tab für PowerShell 7, Windows PowerShell, Eingabeaufforderung, WSL), Fenster für PowerShell 7 / Windows PowerShell / Eingabeaufforderung / WSL, Git Bash |
| Editor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` in einem Bereich/Tab/Terminal |
| Git-Werkzeug | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui in einem Bereich/Tab/Terminal (die von herdr, tmux, kitty und `xdg-terminal-exec` mit V) |
| KI-Agent | Claude Code, Codex, Gemini CLI, opencode in einem herdr-Tab/-Split (V, mit Claude), tmux, zellij, WezTerm, kitty, neues Terminal |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL unter Windows: `wsl.exe --cd ${cwd}` akzeptiert den Windows-Pfad unverändert,
und der `wsl.exe`-Tab von Windows Terminal startet im Ordner, den WSL unter
`/mnt/` abbildet.

Der Explorer, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual
Studio und Zed (Zed gibt es nur für Unix) erscheinen nur dort, wo es sie gibt;
tmux, zellij, cmux, screen, die Fernsteuerung von kitty und die über `sh`
gesteuerten herdr-Voreinstellungen gibt es nur für Unix; herdr (einfache
Voreinstellungen), WezTerm, Alacritty, VS Code, Cursor und die JetBrains-IDEs
werden überall angeboten.

### Variable `${web_url}`

Die Webseite des Remotes `origin` des Repositorys, wenn es ein GitHub- oder
GitLab-Projekt ist (github.com, gitlab.com oder ein selbst gehosteter Host mit
einem Eintrag in `[[integrations]]`); sonst leer, und dann bricht ein Eintrag, der
sie nutzt, mit einer Meldung ab, statt eine leere Adresse zu öffnen. Sie ist
optional: Keine Voreinstellung braucht sie außer `Repository page on the web`.

## Die Einstellungsseite

Einstellungen > **Öffnen in** bearbeitet die Liste. Die Einträge stehen als
[Liste](settings.md#listen) links (der Standardeintrag ist mit `★` markiert), die Felder
des ausgewählten Eintrags rechts; `+ Hinzufügen`, `Duplizieren`, `Entfernen`, `↑` / `↓`
und das Ziehen einer Zeile verwalten die Liste.

| Zeile | Was sie tut |
|---|---|
| **Standardeintrag** | Der Eintrag, den **Im Standardeintrag öffnen** ausführt. Er ist in der Liste Öffnen in mit `★` markiert. |
| **Bezeichnung** | Der in der Liste gezeigte Name. Eindeutig, nicht leer. |
| **Befehl** | Der Befehl, mit den Variablen-Chips (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) zum Einfügen. |
| **Im aktuellen Tab testen** | Führt den Eintrag jetzt im Ordner des aktiven Tabs aus. |
| **+ Hinzufügen ▾** | Hängt einen fertigen Eintrag an (erkannte zuerst) oder **Eigenen**, einen leeren Eintrag, dessen Befehl als Nächstes bearbeitet wird. |
| **Erkannte Standardwerte wiederherstellen** | Ersetzt die Liste durch das, was auf diesem Rechner erkannt wird, und löscht den Standard. |

Änderungen werden beim Bearbeiten gespeichert. Die erste Änderung schreibt das
ganze Array `[[ui.open_in]]` in die `config.toml`; bis dahin wird die Liste
berechnet (siehe [Alte Schlüssel](#alte-schlüssel)), und die Datei hat keinen
Schlüssel `open_in`. Die Suche der Einstellungen findet die Seite ebenfalls.

## Binärdateien: Öffnen in

Eine Binärdatei (ein PDF, ein Word-Dokument, eine Tabelle, ein Archiv ...) hat keinen Text, den die Diff-Ansicht oder die Dateiansicht zeigen könnte. Kennt das Betriebssystem eine Anwendung für ihren Typ, zeigt die Ansicht neben dem üblichen Hinweis eine Schaltfläche **Öffnen in**. Ihre Liste enthält zuerst die Anwendung, mit der das System diesen Typ öffnet (mit einem Punkt markiert), danach jede weitere Anwendung, die das System dafür registriert hat (die systemeigene Liste *Öffnen mit*). Unter Linux ist das der MIME-Typ des Dateinamens, `mimeapps.list` für die Standardanwendung und die `.desktop`-Dateien für die übrigen; unter Windows die Registry (die Wahl des Benutzers und die für die Endung registrierten Programme); unter macOS nur die Standardanwendung.

`o` startet die Standardanwendung und `Shift+O` öffnet die Liste. Beide stehen in der Befehlspalette (*Datei: In Standardanwendung öffnen*, *Datei: Öffnen mit...*), und **Öffnen mit...** steht im Kontextmenü einer Datei im Status-Bereich und in der Dateiliste eines Commits. Die Suche läuft im Hintergrund, einmal pro Endung: die Ansicht zeigt währenddessen „Suche nach Apps...“. Ein Typ, für den das System keine Anwendung kennt, und reiner Text behalten den einfachen Hinweis und haben keine Schaltfläche.

![Eine PDF-Datei in der Dateiansicht mit ihrer Liste Öffnen in: zuerst die Standardanwendung, mit einem Punkt markiert, dann die übrigen Anwendungen, die das System für den Typ kennt](../images/open-with.png)

Die Datei des Arbeitsverzeichnisses wird der Anwendung unverändert übergeben. Die Version eines Commits (oder des Index) wird in eine **schreibgeschützte temporäre Kopie** geschrieben, mit einem Hinweis in der Statuszeile, damit eine Anwendung das Repository nicht ändern kann. Beides ändert das Repository nicht: kein Journal-Eintrag, Rückgängig bleibt unberührt.

Die Liste des Menüs öffnet sich neben dem Menü, auf einer Höhe mit der Zeile **Öffnen mit...**, wie ein Untermenü (links, wenn rechts kein Platz ist), mit der Maus wie mit der Tastatur. Die Liste der Bildeditoren (**Bearbeiten in...**), aus einem Dateimenü geöffnet, verhält sich ebenso.

### Git-LFS-Dateien

Die Git-LFS-Objektkarte (siehe [Git LFS](lfs.md)) trägt unter ihrem Text dieselben zwei Aktionen: **Im Standardprogramm öffnen** und **Öffnen in**, mit denselben Tasten (`o`, `Shift+O`) und Palettenzeilen. Sie öffnen die echte Datei, nie den Zeiger: die Datei des Arbeitsverzeichnisses unverändert, oder eine schreibgeschützte temporäre Kopie des Objekts aus dem lokalen LFS-Speicher für die Version eines Commits oder des Index. Ein Objekt, das noch nicht heruntergeladen ist, hat keine Schaltflächen auf der Karte, und die Statuszeile sagt, dass der Inhalt noch nicht heruntergeladen ist. Dieselbe Ablehnung gilt für eine Datei des Arbeitsverzeichnisses, die nur den Zeiger enthält: **Im Standardprogramm öffnen**, **Öffnen in** und **Bearbeiten in**, von der Karte oder aus jedem Dateimenü, sagen, dass der Inhalt nicht heruntergeladen ist, statt den Zeigertext zu öffnen (laden Sie ihn mit den Zeilen Pull oder Checkout der LFS-Werkzeugleiste herunter und versuchen Sie es dann erneut). Das Öffnen ist schreibgeschützt: kein Journal-Eintrag, nichts rückgängig zu machen.

### Browser unter Windows

Chrome und Edge registrieren `--single-argument %1`, wodurch sie alles nach dem Schalter als eine einzige Adresse lesen, Anführungszeichen eingeschlossen. GitPoulpi übergibt den Pfad an ein solches Programm wörtlich und ohne Anführungszeichen, wie es Windows tut, sodass eine Datei mit Leerzeichen im Pfad geöffnet wird, statt mit `http://"c/...` zu scheitern. Jedes andere Programm erhält den Pfad als ein gewöhnliches Argument.

## Alte Schlüssel

Die alten Schlüssel `ui.terminal_launch` und `ui.terminal_launch_command` werden
nur gelesen, solange `ui.open_in` fehlt. Dann ist die berechnete Liste:

1. Ihre alte Terminal-Wahl, falls sie nicht `auto` ist (`custom` wird zu
   „Terminal (custom)“ mit `{repo}` umgeschrieben zu `${cwd}`; eine
   Voreinstellungs-ID wird zu dieser Voreinstellung). Ein Wert `auto` wird zum
   ersten Eintrag, „Default terminal“.
2. der Datei-Explorer des Betriebssystems,
3. die aus der Umgebung erkannten Multiplexer und Terminals,
4. die im `PATH` gefundenen Editoren.

Speichern Sie die Liste aus den Einstellungen, übernimmt `ui.open_in`, und die
alten Schlüssel werden ignoriert. Die Palettenbefehle **Tab: Im Terminal öffnen** und **Tab: Im
Dateimanager öffnen** nutzen diese Liste nicht.

## Fehlerbehebung

- *„{label}: Programm X nicht gefunden (PATH geprüft)“*: Das Programm liegt nicht
  im `PATH`, den GitPoulpi sieht. Starten Sie GitPoulpi aus einer Shell, die es
  hat, oder schreiben Sie den vollständigen Pfad.
- *Eintrag für tmux/zellij/herdr scheitert sofort*: Diese sprechen mit einem
  laufenden Server, funktionieren also nur, wenn GitPoulpi darin läuft.
- *Eintrag für einen kitty-Tab scheitert*: Er braucht `allow_remote_control` in
  der `kitty.conf`.
- *Windows-Explorer*: `explorer.exe` endet auch bei Erfolg mit Code 1; GitPoulpi
  wertet das als Erfolg.
