[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · **Italiano** · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Eseguire in Docker

L'immagine `poulpilab/gitpoulpi` contiene i binari ufficiali di rilascio (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) su `debian:bookworm-slim`, con `git`, `git-lfs`, `openssh-client`, `less` e `tini`. Gira su `linux/amd64` e `linux/arm64`. È su Docker Hub.

## Avviare la TUI

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` dà alla TUI un terminale. Senza, il programma non ha nulla su cui disegnare.
- `-v "$PWD":/work` monta la cartella corrente come directory di lavoro (`/work`). Apri da lì un repository, oppure monta più repository sotto `/work`.
- `-v gitpoulpi-config:/config` è un volume con nome che conserva impostazioni e sessioni tra un'esecuzione e l'altra.
- `--user "$(id -u):$(id -g)"` esegue come te, così i file che GitPoulpi crea nei tuoi repository mantengono il proprietario dell'host invece di `root`. Ogni cartella in cui l'immagine scrive è scrivibile da tutti, quindi funziona qualsiasi id utente.

## Volumi e percorsi

| Percorso | Cosa contiene |
|---|---|
| `/work` | La directory di lavoro. Monta qui i tuoi repository con un bind mount. |
| `/config` | Impostazioni (`config.toml`), sessioni e, come sottocartelle, dati e cache (`GITPOULPI_CONFIG_DIR=/config`). Usa un volume con nome o un bind mount per conservarli. |
| `/data`, `/cache` | Dichiarati anch'essi come volumi, per gli strumenti che seguono le variabili XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Non è un volume: ne viene creato uno nuovo per ogni container. |

Senza un volume per `/config` ogni esecuzione con `--rm` parte con le impostazioni predefinite. Per partire da un tuo `config.toml`, montalo: `-v "$PWD/config.toml":/config/config.toml:ro` (sola lettura significa che le Impostazioni non possono salvare).

## Repository di un altro utente

Git rifiuta un repository il cui proprietario è diverso dall'utente che lo esegue («dubious ownership»). Con `--user` come sopra il proprietario coincide. Quando non coincide (un bind mount da un altro account, una condivisione di Docker Desktop), considera attendibili tutte le directory con:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Il punto di ingresso esegue allora `git config --global --add safe.directory '*'` prima di avviare la TUI. Usalo solo per cartelle di cui ti fidi, perché toglie la protezione di git all'intero container.

## SSH

Inoltra il tuo agente SSH invece di copiare le chiavi nel container.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) espone l'agente dell'host a un percorso fisso dentro la sua VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Con `--user` il socket deve essere leggibile dal tuo id utente, e lo è quando ti appartiene. Il container non ha un `known_hosts`: la prima connessione a un host ti chiede di accettarne la chiave, oppure monta il tuo file con `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Identità git e credenziali

- **Identità e impostazioni:** monta la tua configurazione di Git in sola lettura: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Oppure imposta l'identità dei commit in un profilo di GitPoulpi (Impostazioni > Profili), che vive in `/config`.
- **Credenziali HTTPS:** l'immagine non ha un credential helper come un portachiavi. Monta un archivio di credenziali (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, con `credential.helper = store` nel `.gitconfig` montato), oppure usa un remoto SSH con l'agente inoltrato.
- **Token degli host** (pull request, issue): senza un credential helper GitPoulpi tiene un token in memoria solo per la sessione. Per una configurazione duratura passalo tramite l'ambiente con `-e GITPOULPI_TOKEN_<ID>=...`, dove `<ID>` è l'id dell'integrazione in maiuscolo con i caratteri non alfanumerici sostituiti da `_` (vedi [Integrazioni di hosting e issue tracker](hosting-integration.md)). Le integrazioni stesse sono in `config.toml` sotto `/config`.

## Terminale, colori e mouse

Colori e supporto del mouse dipendono dal tuo emulatore di terminale. Docker inoltra `TERM` a partire da `-t`; aggiungi `-e TERM -e COLORTERM` quando i colori o il tema true-color sembrano sbagliati, ad esempio `-e COLORTERM=truecolor`. Clic, trascinamenti e rotellina del mouse arrivano come al solito attraverso il terminale. Il grafo dei commit è disegnato con caratteri; le immagini (kitty, iTerm2, sixel) non vengono inoltrate da ogni terminale e multiplexer, vedi [Scegliere un terminale](terminals.md). Ridimensionare la finestra del terminale ridimensiona la TUI: il container riceve la variazione di dimensione.

## CLI e MCP

Il punto di ingresso avvia la TUI. Gli altri programmi si raggiungono sostituendolo:

```
# la CLI scriptabile (aggiungi --json per gli script)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# il server MCP, su standard input e output, per un agente che lo avvia
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Ogni `docker run` è un container a sé, con il proprio daemon in background di GitPoulpi, quindi le modifiche fatte tramite la CLI non compaiono in tempo reale in una TUI in esecuzione in un altro container. Per pilotare una TUI dalla CLI, esegui entrambe nello stesso container: avvia la TUI con `--name gp` e usa `docker exec gp gitpoulpi ...`. Per un agente, registra `docker run -i --rm --entrypoint gitpoulpi-mcp ...` come comando del server MCP, vedi [MCP e agenti](mcp-and-agents.md); gli interruttori dei permessi MCP nel `config.toml` montato si applicano come al solito.

## Tag e aggiornamento

| Tag | Significato |
|---|---|
| `X.Y.Z` | Una release esatta. |
| `X.Y` | L'ultima patch di quella versione minore (solo release stabili). |
| `latest` | L'ultima release stabile. |

Una pre-release riceve solo il proprio tag esatto `X.Y.Z-...`.

Un'immagine si sostituisce, non si aggiorna sul posto. GitPoulpi dentro un container lo sa (il suo registro di installazione dice «un'immagine Docker»): il [pannello degli aggiornamenti](updates.md) mostra la versione e i comandi e non sostituisce nulla. Per aggiornare:

```
docker pull poulpilab/gitpoulpi
```

Poi avvia un nuovo container. Le tue impostazioni restano nel volume `/config`. Per tornare indietro, esegui un tag più vecchio: `docker pull poulpilab/gitpoulpi:<previous>`. Per fissare una versione negli script, usa il tag esatto invece di `latest`. Un'organizzazione può replicare l'immagine sul proprio registry, vedi [Distribuzione aziendale](enterprise-distribution.md#mirror-del-registry-docker).
