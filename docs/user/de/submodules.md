[English](../submodules.md) · [Čeština](../cs/submodules.md) · **Deutsch** · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submodule

Ein Submodul ist ein anderes Repository, das in Ihrem bei einem festen Commit verschachtelt liegt. GitPoulpi listet sie in der Refs-Seitenleiste, aktualisiert sie über die Oberfläche, die CLI oder die Befehlspalette und kann sie mit dem ausgecheckten Commit synchron halten.

## Der Abschnitt SUBMODULE

Hat ein Repository eine `.gitmodules`, erhält die Refs-Seitenleiste einen Abschnitt **SUBMODULE** (unter den anderen; er fehlt, wenn es keine gibt). Eine Zeile pro Submodul: ein Zustandszeichen, sein Pfad und ein kurzer Commit.

| Zeichen | Zustand | Bedeutung |
|---|---|---|
| `✓` | aktuell | ausgecheckt beim Commit, den das Repository festhält |
| `↑` | geändert | bei einem anderen Commit ausgecheckt (es ist weitergerückt, oder Sie haben den Branch gewechselt, ohne es zu aktualisieren) |
| `○` | nicht geklont | in `.gitmodules` deklariert und festgehalten, aber sein Ordner ist leer: aktualisieren Sie es, um es zu klonen |
| `✗` | Konflikt | das Repository hat Merge-Konflikte am Submodul |

![Der Abschnitt SUBMODULE mit drei Zeilen: vendor/almanac nicht geklont (○), vendor/ephemeris aktuell (✓) und vendor/starfield geändert (↑, bei einem anderen Commit als dem festgehaltenen), was auch als geänderte Datei im Status-Bereich erscheint](../images/submodules.png)

| Aktion | Maus | Tastatur |
|---|---|---|
| Das Submodul als eigenen Tab öffnen | Doppelklick auf die Zeile | auswählen, `Enter` (Palette: `Refs: Submodul in Tab öffnen`) |
| Menü: **Aktualisieren (bei Bedarf klonen)**, **Initialisieren**, **In neuem Tab öffnen** | Rechtsklick auf die Zeile | auswählen, die Kontextmenü-Taste (Standard `.`) |
| Das ausgewählte Submodul aktualisieren (klont es bei Bedarf zuerst) | Menü | `Ctrl+P`: `Refs: Submodul aktualisieren (ausgewählt)` |
| Es in `.git/config` eintragen, ohne zu klonen | Menü | `Ctrl+P`: `Refs: Submodul initialisieren (ausgewählt)` |
| Alle Submodule aktualisieren | | `Ctrl+P`: `Git: Alle Submodule aktualisieren` |
| Zum Abschnitt springen | Klick auf seine Überschrift | `Ctrl+P`: `Navigation: Refs > Submodule fokussieren` |
![Das Rechtsklickmenü der geänderten Zeile vendor/starfield: Aktualisieren (bei Bedarf klonen), Initialisieren und In neuem Tab öffnen](../images/submodule-menu.png)

Eine Aktualisierung kann vom Remote des Submoduls fetchen und folgt daher wie jede andere Übertragung **Einstellungen > SSH & Zugangsdaten > HTTPS-Zugangsdaten**. Ein nicht geklontes Submodul lässt sich nicht als Tab öffnen; die Statuszeile sagt das und verweist auf Aktualisieren.

## Submodule aktuell halten

**Einstellungen > Synchronisierung > Submodule aktuell halten** (`general.keep_submodules_up_to_date`, standardmäßig aus). Ist sie an, folgt auf einen erfolgreichen **Checkout** oder **Pull** ein `git submodule update --init --recursive`. Das läuft nur, wenn ein Submodul nicht mehr zu dem passt, was das Repository nun festhält, sodass ein Repository ohne Submodule nicht verlangsamt wird. Schlägt diese Aktualisierung fehl, hat der Checkout oder Pull trotzdem stattgefunden, und der Fehler sagt das.

## Von der Kommandozeile

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Ohne Pfade sind alle Submodule gemeint. Dieselben drei gibt es als MCP-Tools (`submodule_list` ist ein Lesezugriff; `init` und `update` sind Mutationen und folgen `allow_mutations` aus `[mcp]`).

## Hinweise

- Nur die oberste Ebene wird aufgelistet; die Submodule eines Submoduls erscheinen, wenn Sie es als Tab öffnen (oder nutzen Sie `--recursive`, um sie zu aktualisieren).
- Neuere git-Versionen weigern sich, ein Submodul mit *lokalem Pfad* über den Transport `file` zu klonen, es sei denn, `protocol.file.allow` erlaubt es. Das betrifft nur Submodule, deren URL ein lokaler Ordner ist; setzen Sie es in Ihrer globalen Git-Konfiguration, wenn Sie solche nutzen.
