[English](../lfs.md) · [Čeština](../cs/lfs.md) · **Deutsch** · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS („Large File Storage“) hält große Dateien (Design-Dateien, Videos, Datensätze) aus dem Verlauf eines Repositorys heraus: Git speichert stattdessen einen dreizeiligen *Zeiger* (Pointer) als Text, und der eigentliche Inhalt liegt auf dem LFS-Server und in einem lokalen Cache unter `.git/lfs`.

GitPoulpi braucht dafür kein Konto. LFS nutzt die Zugangsdaten, die Ihr Remote ohnehin verwendet (Ihr Git-Credential-Helper oder der ssh-agent).

## Was das Programm `git-lfs` braucht und was nicht

GitPoulpi startet das Standardprogramm `git-lfs` für alles, was Dateien umwandelt oder verschiebt (install, track, fetch, pull, push, Sperren).

Den LFS-*Zustand* zu lesen braucht kein Programm: ob ein Repository LFS nutzt, welche Dateien Zeiger sind, wie groß sie sind und ob das echte Objekt schon lokal gespeichert ist.

| | braucht `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (meldet das Programm als fehlend, statt zu scheitern) | nein |
| `gitpoulpi lfs files` | nein |
| install / track / untrack / prune | ja |
| fetch / pull / push | ja |
| Sperren | ja |

## `git-lfs` installieren

- **Windows**: Es wird mit Git for Windows geliefert; oder `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Alles andere: https://git-lfs.com

Version 3.0 oder neuer ist erforderlich. Nach der Installation bemerkt GitPoulpi es innerhalb einer Minute (oder sofort bei `lfs status`).

## Wenn `git-lfs` fehlt

GitPoulpi stürzt deswegen nie ab und beendet sich nie:

- `gitpoulpi lfs status` gelingt und meldet `git-lfs: not installed`, mit dem Installationslink.
- Ein Vorgang, der das Programm braucht, schlägt mit einer klaren Meldung fehl, die mit dem Installationslink (`https://git-lfs.com`) endet, und mit einem Exit-Status ungleich null; über MCP gibt das Tool diese Meldung zurück.
- Ein Repository, das LFS nutzt, öffnet sich trotzdem; seine großen Dateien erscheinen einfach als Zeiger, bis das Programm installiert ist.

## LFS-Dateien ansehen

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## Ein Repository einrichten: install, track, untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` schreibt standardmäßig in die `.git/config` und die Hooks des Repositorys selbst, sodass Ihre globale Git-Konfiguration nie angetastet wird, außer Sie übergeben `--user`.
- `track` und `untrack` bearbeiten `.gitattributes` und lassen sie **ungestaged**: Sie erscheint als gewöhnliche Änderung im Arbeitsverzeichnis, die Sie selbst committen. Dateien, die *nach* dem Tracking hinzukommen, werden mit ihrem nächsten Commit als Zeiger gespeichert.
- Dateien, die **bereits** als normaler Inhalt committet sind, bleiben im Verlauf normal. `--renormalize` stagt sie neu, sodass Ihr nächster Commit sie durch Zeiger ersetzt. Um sie auch in jedem früheren Commit zu verschieben, dient die Umschreib-Regel *Dateien zu LFS migrieren* (siehe [Verlauf umschreiben](history-rewrite.md#dateien-nach-git-lfs-verschieben-oder-zurückholen)); sie ist die eigene von GitPoulpi, nicht `git lfs migrate`.
- Setzen Sie das Muster in Anführungszeichen, damit Ihre Shell es nicht expandiert.
- Über MCP sind das gewöhnliche Mutationen (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), die der Steuerung `[mcp]` in der `config.toml` unterliegen.

## Objekte bewegen: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- Das Remote ist standardmäßig das Standard-Remote des Repositorys, wie bei `fetch` und `push`.
- Die Zugangsdaten sind die üblichen Ihres Remotes (Credential-Helper, ssh-agent). GitPoulpi verlangt nie ein eigenes Konto.
- `pull` installiert die LFS-Filter für dieses Repository, falls sie fehlen: Ohne sie lädt git-lfs die Objekte herunter, ersetzt aber die Zeiger nicht, und ein späterer Checkout würde die Zeiger zurückbringen.
- MCP behandelt `lfs push` wie `push`: ein destruktives Tool (es ändert das Remote).
- Das Fenster zeigt den Fortschritt einer Übertragung (Dateien und Bytes) in der Statuszeile. Die CLI wartet auf das Ergebnis.

### Den Inhalt auschecken: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

Es lädt nie herunter (das ist `lfs pull`); Objekte, die im lokalen Speicher fehlen, bleiben Zeiger. Wie `pull` installiert es zuerst die LFS-Filter des Repositorys, wenn keine konfiguriert sind. Wie jeder LFS-Vorgang ist es nicht rückgängig zu machen; siehe [Rückgängig und Wiederholen](undo-redo.md). Über MCP heißt es `lfs_checkout` (ein Schreibzugriff, nicht destruktiv).

### Die LFS-Schaltfläche der Symbolleiste, und Pull / Push

Die Symbolleiste hat eine Schaltfläche **`LFS ▾`** nach **Pop** (ausgeblendet, wenn `git.lfs_enabled` aus ist, und in einem Repository, in dem LFS weder initialisiert ist noch von seinen `.gitattributes` genutzt wird: siehe [Einstellungen > LFS](settings.md#repository-spezifisch-lfs)). Ein Klick öffnet ein Dropdown: **Fetch: alle LFS-Dateien** (`lfs fetch --all`), **Alle LFS-Dateien auschecken** (`lfs checkout`), **Pull: alle LFS-Dateien** (`lfs pull`), **Push: alle LFS-Dateien** (`lfs push --all`, die Objekte jeder Ref) und **Lokalen LFS-Speicher bereinigen** (`lfs prune`). Jede Zeile ist auch ein Palettenbefehl (`LFS: Fetch (alle Objekte)`, `LFS: Alle Dateien auschecken`, `LFS: Objekte pullen`, `LFS: Push (alle Objekte)`, `LFS: Lokale Objekte bereinigen`). `LFS: Menü öffnen` öffnet das Dropdown. Jede hat eine `Ctrl+G`-Folge: `l` für das Menü, dann `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` und `Shift+V` für die Zeilen. Sie können sie unter *Einstellungen > Tastatur & Maus* ändern. Fehlt `git-lfs`, führt jede Zeile zum Installationsdialog.

![Das LFS-Dropdown der Symbolleiste: alle LFS-Dateien fetchen, auschecken, pullen und pushen, und den lokalen Speicher bereinigen](../images/lfs-menu.png)

**Pull und Push kümmern sich selbst um LFS.** In einem Repository, das LFS nutzt, prüft bei installiertem `git-lfs` ein einfacher `pull` zuerst, ob die LFS-Filter des Repositorys installiert sind, und ein einfacher `push` prüft seinen `pre-push`-Hook. Fehlt einer, führt GitPoulpi vor der Übertragung `git lfs install --local` aus (nur dieses Repository), und die Statuszeile zeigt „LFS-Filter installiert“. Ohne den Hook würde ein Push keine LFS-Objekte hochladen. Schalten Sie das ab mit `git.lfs_autofix_filters = false` (*Einstellungen > Git & Kodierung > LFS-Filter vor Pull und Push installieren*). Ein einfacher `push` lädt weiterhin nur die Objekte des gepushten Branches hoch, nie die jeder Ref; Bereinigen (prune) geschieht nie implizit.

## Dateisperren

Manche Dateien lassen sich nicht mergen (eine Photoshop-Datei, ein Level, ein binäres Asset). Git-LFS-*Sperren* lassen eine Person sagen „Ich bearbeite das gerade“: Niemand sonst kann sie sperren oder Änderungen daran pushen, bis sie entsperrt ist.

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Sperren ist eine Funktion des **LFS-Servers** (GitHub, GitLab und Gitea bieten sie). Sie wird mit den eigenen Zugangsdaten des Remotes erreicht; GitPoulpi hat kein eigenes Konto.
- Ein Server ohne Sperrfunktion lässt `lock` mit einer entsprechenden Meldung scheitern (der typisierte Fehler `LfsLockingUnsupported`). `locks` listet auf einem solchen Server nichts auf, was git-lfs selbst meldet.
- Ein Muster mit `--lockable` zu tracken (`lfs track '*.psd' --lockable`) macht diese Dateien schreibgeschützt, bis Sie sie sperren.
- `unlock` auf die Sperre eines anderen wird abgelehnt; `--force` bricht sie, weshalb MCP `lfs_unlock` als destruktives Tool behandelt. `lfs_locks` ist nur lesend.

## Ein LFS-Repository klonen

`gitpoulpi clone <url>` (und der Klon-Dialog der TUI) checkt das Repository aus, *ohne* die LFS-Filter auszuführen, lädt dann, wenn das Repository LFS nutzt und `git-lfs` installiert ist, die Objekte mit `git lfs pull` herunter (und installiert die Filter für dieses Repository, falls keine konfiguriert sind) und zeigt das als eigene Phase, *LFS-Objekte herunterladen*. `--no-lfs` überspringt diesen Schritt und lässt Zeiger zurück.

Ein Klon **scheitert nie wegen LFS**: Fehlt `git-lfs` oder schlägt ein Download fehl, gelingt der Klon trotzdem, und die großen Dateien bleiben Zeiger; der LFS-Status des Repositorys nennt den Grund, und `gitpoulpi lfs pull` erledigt den Rest, sobald das Problem behoben ist.

## In der Terminal-Oberfläche

- **Das `LFS`-Etikett.** Eine mit Git LFS gespeicherte Datei trägt hinter ihrem Namen ein kleines Etikett `LFS` in den Listen Unstaged und Staged des Status-Bereichs und in der Liste der geänderten Dateien eines Commits. **Nur LFS** (`o` bei fokussiertem Status-Bereich oder Log, das Kästchen in der Werkzeugleiste der Dateilisten oder **Dateien: Nur LFS umschalten** in der Palette) beschränkt diese Listen auf die LFS-Dateien. Es erscheint nur, wenn das Repository welche hat. Eine Datei zählt als LFS, wenn ihr Pfad im Index ein Zeiger ist, dieselbe Abfrage, die das Etikett speist (ein gebündelter `git`-Aufruf pro Aktualisierung, nichts für ein Repository ohne `filter=lfs`-Attribute).
- **Die Diff-Karte.** Der Diff einer LFS-Datei zeigt statt des Zeigertexts eine Karte: `Git-LFS-Objekt`, `1.2 MB -> 1.4 MB`, die kurzen alten und neuen Oids, und ob das neue Objekt `lokal gespeichert` ist oder `nicht heruntergeladen`. Mit `r` sehen Sie den rohen Zeiger-Diff, mit `r` erneut kehren Sie zurück. Auf eine LFS-Datei lässt sich nur Staging auf Dateiebene anwenden (es gibt keine Hunks oder Zeilen zu stagen). Unter der Karte öffnen **Im Standardprogramm öffnen** und **Öffnen in** (`o`, `Shift+O` oder die Palette) die echte Datei, nie den Zeiger: siehe [Öffnen in](open-in.md#git-lfs-dateien).
- **Die Dateiansicht** einer LFS-Datei zeigt `Git-LFS-Objekt, 12.3 MB, sha256:ab12cd34...` und wo man den Inhalt bekommt.
- **Das Banner.** Für ein Repository, das LFS nutzt, während `git-lfs` fehlt oder älter als 3.0 ist, sagt das ein Banner im Status-Bereich. Klicken Sie darauf (oder nutzen Sie den Paletteneintrag *LFS: Installationsseite öffnen*), um https://git-lfs.com zu öffnen. Sonst ändert sich nichts: Das Fenster öffnet sich, Diffs funktionieren, große Dateien erscheinen einfach als Zeiger.
- **Das Dateimenü und die Palette.** Rechtsklick auf eine geänderte Datei im Status-Bereich: *Mit LFS verfolgen: diese Datei* / *diese Endung* für eine normale Datei; *Datei sperren (LFS)*, *Datei entsperren (LFS)* oder *Zwangsentsperren (LFS, Sperre eines anderen)* für eine LFS-Datei, je nachdem, wer ihre Sperre hält. Die Zeile einer gesperrten Datei zeigt `gesperrt von you` / `gesperrt von <Name>`. Alles ist auch ein Palettenbefehl (`Ctrl+P`, `LFS` tippen): initialisieren, ein Muster tracken oder nicht mehr tracken, Objekte fetchen, pullen oder pushen, bereinigen, Sperren anzeigen, die Installationsseite öffnen.
- **Eine fehlgeschlagene Aktion erklärt sich selbst.** Scheitert eine LFS-Aktion, weil `git-lfs` fehlt, trägt ein Dialog die Meldung und eine Schaltfläche *Installationsseite öffnen*.
- **Einstellungen.** *Einstellungen > LFS* (pro Repository) zeigt, ob LFS initialisiert ist, listet die verfolgten Muster auf und hat **LFS hier einrichten**. *Git & Kodierung* (pro Repository) hat den Schalter **Git LFS**, eine schreibgeschützte Zeile **Programm git-lfs** und eine Schaltfläche **LFS für dieses Repository initialisieren**.
- **Git LFS** aus (`git.lfs_enabled = false`, siehe die [Konfigurationsreferenz](config-reference.md)) blendet all das aus. Die eigenen LFS-Filter von git laufen weiter.
- Der LFS-Zustand wird kurz nach dem Öffnen des Repositorys gelesen, nach jeder Änderung, bei manueller Aktualisierung, wenn sich `.gitattributes` ändert, und alle zehn Sekunden bei einem Repository, das LFS nutzt.

## Bekannte Grenzen

- Ein Repository, dessen LFS-Muster nur in einer *verschachtelten* `.gitattributes` stehen (nicht in der im Wurzelverzeichnis, nicht in `.git/info/attributes`), wird erst als LFS-nutzend erkannt, wenn es einen LFS-Objektspeicher (`.git/lfs`) hat, nicht früher.
- `git lfs migrate` selbst wird nicht verwendet. Dateien über den Verlauf hinweg nach LFS oder aus LFS zu verschieben, übernehmen die Umschreib-Regeln *Dateien zu LFS migrieren* / *Dateien aus LFS zurückholen*: Sie arbeiten am lokalen Klon, geschrieben wird nur die `.gitattributes` an der Wurzel, und die Objekte müssen danach noch gepusht werden (*LFS-Objekte senden* im Ergebnisfenster oder `lfs push --all`).
