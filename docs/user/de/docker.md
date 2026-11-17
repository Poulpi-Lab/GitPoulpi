[English](../docker.md) · [Čeština](../cs/docker.md) · **Deutsch** · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# In Docker ausführen

Das Image `poulpilab/gitpoulpi` enthält die offiziellen Release-Binärdateien (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) auf `debian:bookworm-slim`, mit `git`, `git-lfs`, `openssh-client`, `less` und `tini`. Es läuft auf `linux/amd64` und `linux/arm64`. Es liegt auf Docker Hub.

## Die Oberfläche starten

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` gibt der Oberfläche ein Terminal. Ohne es hat das Programm nichts, worauf es zeichnen kann.
- `-v "$PWD":/work` bindet den aktuellen Ordner als Arbeitsverzeichnis (`/work`) ein. Öffnen Sie von dort ein Repository, oder binden Sie mehrere Repositorys unter `/work` ein.
- `-v gitpoulpi-config:/config` ist ein benanntes Volume, das Ihre Einstellungen und Sitzungen zwischen den Läufen aufbewahrt.
- `--user "$(id -u):$(id -g)"` führt das Programm als Sie aus, sodass die Dateien, die GitPoulpi in Ihren Repositorys anlegt, Ihren Host-Besitzer behalten statt `root`. Jeder Ordner, in den das Image schreibt, ist für alle beschreibbar, daher funktioniert jede Benutzer-ID.

## Volumes und Pfade

| Pfad | Was er enthält |
|---|---|
| `/work` | Das Arbeitsverzeichnis. Binden Sie Ihre Repositorys hier ein (Bind-Mount). |
| `/config` | Einstellungen (`config.toml`), Sitzungen und, als Unterordner, Daten und Cache (`GITPOULPI_CONFIG_DIR=/config`). Verwenden Sie ein benanntes Volume oder einen Bind-Mount, um sie zu behalten. |
| `/data`, `/cache` | Ebenfalls als Volumes deklariert, für Werkzeuge, die den XDG-Variablen folgen (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Es ist kein Volume: In jedem Container ein neues. |

Ohne Volume für `/config` beginnt jeder `--rm`-Lauf mit den Standardeinstellungen. Um mit Ihrer eigenen `config.toml` zu starten, binden Sie sie ein: `-v "$PWD/config.toml":/config/config.toml:ro` (schreibgeschützt bedeutet, dass die Einstellungen nicht speichern können).

## Repositorys eines anderen Benutzers

Git lehnt ein Repository ab, dessen Besitzer sich vom ausführenden Benutzer unterscheidet („dubious ownership“). Mit `--user` wie oben stimmt der Besitzer überein. Wenn nicht (ein Bind-Mount von einem anderen Konto, eine Docker-Desktop-Freigabe), vertrauen Sie jedem Verzeichnis mit:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Der Einstiegspunkt führt dann `git config --global --add safe.directory '*'` aus, bevor er die Oberfläche startet. Verwenden Sie das nur für Ordner, denen Sie vertrauen, da es den Schutz von Git für den ganzen Container aufhebt.

## SSH

Leiten Sie Ihren SSH-Agenten weiter, statt Schlüssel in den Container zu kopieren.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) stellt den Agenten des Hosts unter einem festen Pfad in seiner VM bereit:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Mit `--user` muss der Socket für Ihre Benutzer-ID lesbar sein, was der Fall ist, wenn er Ihnen gehört. Der Container hat keine `known_hosts`: Bei der ersten Verbindung zu einem Host werden Sie gebeten, dessen Schlüssel zu akzeptieren, oder binden Sie Ihre eigene Datei ein mit `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Git-Identität und Zugangsdaten

- **Identität und Einstellungen:** Binden Sie Ihre Git-Konfiguration schreibgeschützt ein: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Oder legen Sie die Commit-Identität in einem GitPoulpi-Profil fest (Einstellungen > Profile), das in `/config` liegt.
- **HTTPS-Zugangsdaten:** Das Image hat keinen Credential-Helper wie einen Schlüsselbund. Binden Sie entweder einen Credential-Store ein (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, mit `credential.helper = store` in der eingebundenen `.gitconfig`), oder verwenden Sie ein SSH-Remote mit weitergeleitetem Agenten.
- **Hosting-Tokens** (Pull Requests, Issues): Ohne Credential-Helper hält GitPoulpi ein Token nur für die Sitzung im Speicher. Für eine dauerhafte Einrichtung übergeben Sie es über die Umgebung mit `-e GITPOULPI_TOKEN_<ID>=...`, wobei `<ID>` die ID der Integration in Großbuchstaben ist, mit Nicht-Alphanumerischem als `_` (siehe [Hosting- und Issue-Tracker-Integrationen](hosting-integration.md)). Die Integrationen selbst stehen in der `config.toml` unter `/config`.

## Terminal, Farben und Maus

Farben und Mausunterstützung kommen von Ihrem Terminal-Emulator. Docker reicht `TERM` von `-t` weiter; fügen Sie `-e TERM -e COLORTERM` hinzu, wenn Farben oder das True-Color-Design falsch aussehen, zum Beispiel `-e COLORTERM=truecolor`. Mausklicks, Ziehen und das Mausrad kommen wie üblich über das Terminal an. Der Commit-Graph wird mit Zeichen gezeichnet; Bilder (kitty, iTerm2, sixel) werden nicht von jedem Terminal und Multiplexer weitergereicht, siehe [Ein Terminal wählen](terminals.md). Wenn Sie die Größe des Terminalfensters ändern, ändert sich auch die Oberfläche: Der Container erhält die Größenänderung.

## CLI und MCP

Der Einstiegspunkt startet die Oberfläche. Die anderen Programme erreichen Sie, indem Sie ihn überschreiben:

```
# the scriptable CLI (add --json for scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# the MCP server, over standard input and output, for an agent that launches it
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Jedes `docker run` ist ein eigener Container mit einem eigenen GitPoulpi-Hintergrund-Daemon, daher erscheinen Änderungen über die CLI nicht live in einer Oberfläche, die in einem anderen Container läuft. Um eine Oberfläche über die CLI zu steuern, führen Sie beide im selben Container aus: Starten Sie die Oberfläche mit `--name gp` und verwenden Sie `docker exec gp gitpoulpi ...`. Für einen Agenten registrieren Sie `docker run -i --rm --entrypoint gitpoulpi-mcp ...` als Befehl des MCP-Servers, siehe [MCP und Agenten](mcp-and-agents.md); die MCP-Berechtigungsschalter in der eingebundenen `config.toml` gelten wie gewohnt.

## Tags und Aktualisieren

| Tag | Bedeutung |
|---|---|
| `X.Y.Z` | Ein genau bestimmtes Release. |
| `X.Y` | Der neueste Patch dieser Nebenversion (nur stabile Releases). |
| `latest` | Das neueste stabile Release. |

Ein Vorabrelease erhält nur seinen genauen Tag `X.Y.Z-...`.

Ein Image wird ersetzt, nie an Ort und Stelle aktualisiert. GitPoulpi in einem Container weiß das (sein Installationseintrag lautet „ein Docker-Image“): Das [Update-Fenster](updates.md) zeigt die Version und die Befehle und ersetzt nichts. Zum Aktualisieren:

```
docker pull poulpilab/gitpoulpi
```

Starten Sie dann einen neuen Container. Ihre Einstellungen bleiben im Volume `/config`. Um zurückzugehen, führen Sie einen älteren Tag aus: `docker pull poulpilab/gitpoulpi:<previous>`. Um in Skripten eine Version festzulegen, verwenden Sie den genauen Tag statt `latest`. Eine Organisation kann das Image in ihrer eigenen Registry spiegeln, siehe [Unternehmensverteilung](enterprise-distribution.md#docker-registry-spiegel).
