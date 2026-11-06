[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · **Nederlands** · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# GitPoulpi in Docker draaien

De image `poulpilab/gitpoulpi` bevat de officiële release-binaries (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) op `debian:bookworm-slim`, met `git`, `git-lfs`, `openssh-client`, `less` en `tini`. Hij draait op `linux/amd64` en `linux/arm64`. Hij staat op Docker Hub; dezelfde tags staan ook op `ghcr.io/poulpi-lab/gitpoulpi`, en alles hieronder werkt met beide namen.

## De UI starten

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` geeft de UI een terminal. Zonder die opties heeft het programma niets om op te tekenen.
- `-v "$PWD":/work` koppelt de huidige map als werkmap (`/work`). Open daar een repository, of koppel meerdere repositories onder `/work`.
- `-v gitpoulpi-config:/config` is een benoemd volume dat je instellingen en sessies tussen runs bewaart.
- `--user "$(id -u):$(id -g)"` draait als jijzelf, zodat de bestanden die GitPoulpi in je repositories maakt je eigenaar op de host houden in plaats van `root`. Elke map waarnaar de image schrijft is voor iedereen beschrijfbaar, dus elke gebruikers-id werkt.

## Volumes en paden

| Pad | Wat het bevat |
|---|---|
| `/work` | De werkmap. Koppel hier je repositories met een bind mount. |
| `/config` | Instellingen (`config.toml`), sessies en, als submappen, gegevens en cache (`GITPOULPI_CONFIG_DIR=/config`). Gebruik een benoemd volume of een bind mount om ze te bewaren. |
| `/data`, `/cache` | Ook als volumes gedeclareerd, voor tools die de XDG-variabelen volgen (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Het is geen volume: elke container krijgt een verse. |

Zonder volume voor `/config` begint elke `--rm`-run met standaardinstellingen. Om met je eigen `config.toml` te beginnen, koppel je die: `-v "$PWD/config.toml":/config/config.toml:ro` (alleen-lezen betekent dat de Instellingen niets kunnen opslaan).

## Repositories van een andere gebruiker

Git weigert een repository waarvan de eigenaar verschilt van de gebruiker die het uitvoert ("dubious ownership"). Met `--user` zoals hierboven klopt de eigenaar. Als dat niet zo is (een bind mount van een ander account, een Docker Desktop-share), vertrouw je elke map met:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Het toegangspunt voert dan `git config --global --add safe.directory '*'` uit voordat het de UI start. Gebruik dit alleen voor mappen die je vertrouwt, want het heft de bescherming van git voor de hele container op.

## SSH

Stuur je SSH-agent door in plaats van sleutels in de container te kopiëren.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) stelt de agent van de host beschikbaar op een vast pad in zijn VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Met `--user` moet de socket leesbaar zijn voor je gebruikers-id, en dat is hij als hij van jou is. De container heeft geen `known_hosts`: bij de eerste verbinding met een host moet je de sleutel ervan accepteren, of koppel je eigen bestand met `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Git-identiteit en inloggegevens

- **Identiteit en instellingen:** koppel je Git-configuratie alleen-lezen: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Of stel de commitidentiteit in een GitPoulpi-profiel in (Instellingen > Profielen), dat in `/config` staat.
- **HTTPS-inloggegevens:** de image heeft geen credential helper zoals een sleutelhanger. Koppel een credential store (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, met `credential.helper = store` in de gekoppelde `.gitconfig`), of gebruik een SSH-remote met de agent doorgestuurd.
- **Hosting-tokens** (pull requests, issues): zonder credential helper houdt GitPoulpi een token alleen in het geheugen voor de sessie. Voor een blijvende opzet geef je het mee via de omgeving met `-e GITPOULPI_TOKEN_<ID>=...`, waarbij `<ID>` de id van de integratie is in hoofdletters met niet-alfanumerieke tekens als `_` (zie [Integraties met hosting en issuetrackers](hosting-integration.md)). De integraties zelf staan in `config.toml` onder `/config`.

## Terminal, kleuren en muis

Kleuren en muisondersteuning komen van je terminalemulator. Docker geeft `TERM` door vanuit `-t`; voeg `-e TERM -e COLORTERM` toe als kleuren of het true-colorthema er verkeerd uitzien, bijvoorbeeld `-e COLORTERM=truecolor`. Muisklikken, slepen en het wiel komen zoals gewoonlijk via de terminal binnen. De commitgrafiek wordt met tekens getekend; afbeeldingen (kitty, iTerm2, sixel) worden niet door elke terminal en multiplexer doorgestuurd, zie [Een terminal kiezen](terminals.md). Het formaat van het terminalvenster wijzigen verandert het formaat van de UI: de container ontvangt de formaatwijziging.

## CLI en MCP

Het toegangspunt start de UI. De andere programma's bereik je door het te overschrijven:

```
# de scriptbare CLI (voeg --json toe voor scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# de MCP-server, via standaardinvoer en -uitvoer, voor een agent die hem start
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Elke `docker run` is een eigen container met een eigen achtergronddaemon van GitPoulpi, dus wijzigingen via de CLI verschijnen niet live in een UI die in een andere container draait. Om één UI vanuit de CLI te bedienen, draai je beide in dezelfde container: start de UI met `--name gp` en gebruik `docker exec gp gitpoulpi ...`. Registreer voor een agent `docker run -i --rm --entrypoint gitpoulpi-mcp ...` als opdracht van de MCP-server, zie [MCP en agents](mcp-and-agents.md); de MCP-rechtenschakelaars in de gekoppelde `config.toml` gelden zoals gewoonlijk.

## Tags en bijwerken

| Tag | Betekenis |
|---|---|
| `X.Y.Z` | Eén exacte release. |
| `X.Y` | De laatste patch van die minor-versie (alleen stabiele releases). |
| `latest` | De laatste stabiele release. |

Een pre-release krijgt alleen zijn exacte tag `X.Y.Z-...`.

Een image wordt vervangen, nooit ter plekke bijgewerkt. GitPoulpi in een container weet dat (zijn installatiegegevens zeggen "een Docker-image"): het [updatepaneel](updates.md) toont de versie en de opdrachten en vervangt niets. Om bij te werken:

```
docker pull poulpilab/gitpoulpi
```

Start daarna een nieuwe container. Je instellingen blijven in het volume `/config`. Om terug te gaan, draai je een oudere tag: `docker pull poulpilab/gitpoulpi:<vorige>`. Gebruik in scripts de exacte tag in plaats van `latest` om een versie vast te pinnen. Een organisatie kan de image op een eigen registry spiegelen, zie [Distributie voor organisaties](enterprise-distribution.md#spiegel-van-een-docker-registry).
