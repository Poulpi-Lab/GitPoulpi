[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · **Svenska** · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Köra i Docker

Avbilden `poulpilab/gitpoulpi` innehåller de officiella releasebinärerna (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) på `debian:bookworm-slim`, med `git`, `git-lfs`, `openssh-client`, `less` och `tini`. Den körs på `linux/amd64` och `linux/arm64`. Den finns på Docker Hub; samma taggar finns också på `ghcr.io/poulpi-lab/gitpoulpi`, och allt nedan fungerar med båda namnen.

## Starta gränssnittet

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` ger gränssnittet en terminal. Utan den har programmet inget att rita på.
- `-v "$PWD":/work` monterar den aktuella mappen som arbetskatalog (`/work`). Öppna ett repository därifrån, eller montera flera repositories under `/work`.
- `-v gitpoulpi-config:/config` är en namngiven volym som behåller dina inställningar och sessioner mellan körningar.
- `--user "$(id -u):$(id -g)"` kör som du, så att filerna som GitPoulpi skapar i dina repositories behåller din ägare på värden i stället för `root`. Varje mapp som avbilden skriver i är skrivbar för alla, så vilket användar-id som helst fungerar.

## Volymer och sökvägar

| Sökväg | Vad den innehåller |
|---|---|
| `/work` | Arbetskatalogen. Bind-montera dina repositories här. |
| `/config` | Inställningar (`config.toml`), sessioner och, som undermappar, data och cache (`GITPOULPI_CONFIG_DIR=/config`). Använd en namngiven volym eller en bind-montering för att behålla dem. |
| `/data`, `/cache` | Deklarerade som volymer också, för verktyg som följer XDG-variablerna (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Den är ingen volym: en ny för varje container. |

Utan en volym för `/config` startar varje `--rm`-körning med standardinställningar. För att utgå från din egen `config.toml` monterar du den: `-v "$PWD/config.toml":/config/config.toml:ro` (skrivskyddad betyder att Inställningar inte kan spara).

## Repositories som ägs av en annan användare

Git nekar ett repository vars ägare skiljer sig från användaren som kör det ("dubious ownership"). Med `--user` som ovan stämmer ägaren. När den inte gör det (en bind-montering från ett annat konto, en Docker Desktop-delning) litar du på varje katalog med:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Startpunkten kör då `git config --global --add safe.directory '*'` innan den startar gränssnittet. Använd det bara för mappar du litar på, eftersom det upphäver gits skydd för hela containern.

## SSH

Vidarebefordra din SSH-agent i stället för att kopiera nycklar in i containern.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) exponerar värdens agent på en fast sökväg inuti sin VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Med `--user` måste socketen vara läsbar för ditt användar-id, vilket den är när den tillhör dig. Containern har ingen `known_hosts`: den första anslutningen till en värd ber dig godta dess nyckel, eller montera din egen fil med `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Git-identitet och inloggningsuppgifter

- **Identitet och inställningar:** montera din Git-konfiguration skrivskyddad: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Eller sätt commit-identiteten i en GitPoulpi-profil (Inställningar > Profiler), som ligger i `/config`.
- **HTTPS-uppgifter:** avbilden har ingen credential helper som en nyckelring. Montera antingen ett uppgiftslager (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, med `credential.helper = store` i den monterade `.gitconfig`), eller använd en SSH-remote med agenten vidarebefordrad.
- **Värdtokens** (pull requests, ärenden): utan en credential helper håller GitPoulpi en token i minnet bara under sessionen. För en varaktig lösning skickar du den via miljön med `-e GITPOULPI_TOKEN_<ID>=...`, där `<ID>` är integrationens id med versaler och icke-alfanumeriska tecken som `_` (se [Värdtjänster och ärendehantering](hosting-integration.md)). Själva integrationerna ligger i `config.toml` under `/config`.

## Terminal, färger och mus

Färger och musstöd kommer från din terminalemulator. Docker för vidare `TERM` från `-t`; lägg till `-e TERM -e COLORTERM` när färgerna eller true color-temat ser fel ut, till exempel `-e COLORTERM=truecolor`. Musklick, drag och hjulet kommer fram via terminalen som vanligt. Commit-grafen ritas med tecken; bilder (kitty, iTerm2, sixel) vidarebefordras inte genom varje terminal och multiplexer, se [Välja terminal](terminals.md). Att ändra storlek på terminalfönstret ändrar gränssnittets storlek: containern får storleksändringen.

## CLI och MCP

Startpunkten startar gränssnittet. De andra programmen nås genom att åsidosätta den:

```
# the scriptable CLI (add --json for scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# the MCP server, over standard input and output, for an agent that launches it
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Varje `docker run` är en egen container med sin egen GitPoulpi-bakgrundsdaemon, så ändringar gjorda via CLI:t syns inte live i ett gränssnitt som körs i en annan container. För att styra ett gränssnitt från CLI:t kör du båda i samma container: starta gränssnittet med `--name gp` och använd `docker exec gp gitpoulpi ...`. För en agent registrerar du `docker run -i --rm --entrypoint gitpoulpi-mcp ...` som MCP-serverns kommando, se [MCP och agenter](mcp-and-agents.md); MCP-behörighetsbrytarna i den monterade `config.toml` gäller som vanligt.

## Taggar och uppdatering

| Tagg | Betydelse |
|---|---|
| `X.Y.Z` | En exakt release. |
| `X.Y` | Den senaste patchen av den minorversionen (bara stabila releaser). |
| `latest` | Den senaste stabila releasen. |

En förhandsversion får bara sin exakta tagg `X.Y.Z-...`.

En avbild ersätts, den uppdateras aldrig på plats. GitPoulpi i en container vet det (dess installationspost säger "en Docker-avbild"): [uppdateringspanelen](updates.md) visar versionen och kommandona och ersätter ingenting. För att uppdatera:

```
docker pull poulpilab/gitpoulpi
```

Starta sedan en ny container. Dina inställningar ligger kvar i volymen `/config`. För att gå tillbaka kör du en äldre tagg: `docker pull poulpilab/gitpoulpi:<previous>`. För att fästa en version i skript använder du den exakta taggen i stället för `latest`. En organisation kan spegla avbilden i sitt eget register, se [Företagsdistribution](enterprise-distribution.md#spegel-av-docker-registret).
