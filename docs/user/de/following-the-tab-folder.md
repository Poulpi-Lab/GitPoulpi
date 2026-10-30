[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · **Deutsch** · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Dem Ordner des aktiven Tabs folgen

GitPoulpi kann mehrere Repositories als Tabs geöffnet haben. Die Shell, aus der
Sie es gestartet haben, bleibt in ihrem Ordner: Ein Programm kann den Ordner
seiner übergeordneten Shell nicht ändern. GitPoulpi bietet zwei Wege, eine Shell
im Ordner des aktiven Tabs zu bekommen:

- **Beenden & cd hierher**: GitPoulpi beendet sich, und Ihre Shell wechselt in
  den Ordner des aktiven Tabs. Dafür braucht es eine kleine Shell-Funktion, die
  `gitpoulpi shell-init` ausgibt.
- **Öffnen in**: öffnet den Ordner des aktiven Tabs in einem Terminal, einem
  Multiplexer-Bereich, einem Editor oder im Dateimanager. Eine Shell-Einrichtung
  ist nicht nötig. Siehe [Öffnen in](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Öffnen in ▾ ── ⌕ Globale Suche ⚙ × ┐
│ ⌂  main-repo x  other-repo x                Beenden & cd hierher        │
```

- **Öffnen in** steht in der Titelleiste, hinter dem Repository-Pfad. Bei zu
  schmalem Fenster wird es ausgeblendet.
- **Beenden & cd hierher** steht am rechten Ende der Tab-Leiste. Es erscheint nur,
  wenn beides zutrifft:
  - GitPoulpi wurde über die Shell-Funktion gestartet (unten).
  - Der aktive Tab ist nicht der Ordner, in dem Sie GitPoulpi gestartet haben.
- Die Befehle stehen immer in der Palette (`Ctrl+P`): `Tab: Beenden und cd hierher`,
  `Tab: Öffnen in...` (`Alt+O`) und `Tab: Im Standardeintrag öffnen` (`Ctrl+Alt+O`).
  Ohne die Shell-Funktion erklärt Ihnen `Tab: Beenden und cd hierher`, wie Sie
  sie einrichten.

Ein normales Beenden (`q`) ändert nie den Ordner Ihrer Shell. Nur **Beenden &
cd hierher** tut das, es sei denn, Sie wählen `auto` (unten).

## Die Shell-Funktion einrichten

`gitpoulpi shell-init <shell>` gibt eine Funktion namens `gp` aus. Mit `--name`
wählen Sie einen anderen Namen. Die Funktion führt
`gitpoulpi --cwd-file <temporäre Datei> ...` aus. Beenden Sie mit **Beenden &
cd hierher**, schreibt GitPoulpi den Ordner in diese Datei, und die Funktion
wechselt in diesen Ordner. GitPoulpi installiert oder ändert nichts: Sie
fügen eine Zeile in die Startdatei Ihrer Shell ein.

| Shell | Hinzufügen zu | Zeile |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | Ihr `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Starten Sie GitPoulpi dann mit `gp` statt `gitpoulpi`. Argumente werden
weitergereicht (`gp -s work`), und der Exit-Status ist der von `gitpoulpi`.

**Ein vorhandenes `gp` wird nie ohne Warnung ersetzt.** Hat Ihre Shell schon
einen Befehl, Alias oder eine Funktion namens `gp`, gibt das Skript eine
Warnung aus und definiert nichts. Wählen Sie einen anderen Namen, zum Beispiel
`gitpoulpi shell-init bash --name gpl`. In PowerShell ersetzt die Funktion den
eingebauten Alias `gp` (`Get-ItemProperty`) und einen Alias, der auf
`gitpoulpi` zeigt. Jedes andere `gp` bleibt erhalten, mit einer Warnung.

Jeder `gitpoulpi`-Befehl akzeptiert `--cwd-file`, aber nur die TUI schreibt die
Datei.

## Einstellung: Dem Ordner des aktiven Tabs folgen

Einstellungen > Öffnen in > **Dem Ordner des aktiven Tabs folgen** (`ui.tab_dir_follow`):

| Wert | Verhalten |
|---|---|
| `buttons` (Standard) | **Beenden & cd hierher** und **Öffnen in** wie oben beschrieben. |
| `auto` | Keine Schaltfläche **Beenden & cd hierher**. **Öffnen in** bleibt. Jedes Beenden wechselt Ihre Shell in den Ordner des aktiven Tabs (wenn Sie GitPoulpi mit der Shell-Funktion gestartet haben). Der Standardeintrag von **Öffnen in** läuft jedes Mal, wenn Sie einen neuen Tab öffnen. |

Im Modus `auto` läuft der Standardeintrag nur für Tabs, die Sie selbst öffnen:
aus dem Home-Tab, einem Worktree, einem Klon oder einer Initialisierung. Er
läuft nicht für beim Start wiederhergestellte Tabs, nicht für den ersten Tab
und nicht, wenn Sie zu einem bereits geöffneten Tab wechseln.

Die älteren Einstellungen `ui.terminal_launch` und `ui.terminal_launch_command`
sind in die **Öffnen-in**-Liste verschoben. Siehe
[Öffnen in](open-in.md#alte-schlüssel).
