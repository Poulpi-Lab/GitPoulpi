[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · **Italiano** · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installazione

GitPoulpi ha tre eseguibili:

| Eseguibile | Cos'è |
|---|---|
| `gitpoulpi-tui` | La UI da terminale. |
| `gitpoulpi` | L'interfaccia a riga di comando (`--json` per gli script). Senza sottocomando apre la UI. |
| `gitpoulpi-mcp` | Il server MCP per gli agenti IA. Vedi [MCP e agenti](mcp-and-agents.md). |

Accanto a questi, l'archivio contiene `gitpoulpi-launch`, un minuscolo programma che i collegamenti eseguono per mostrare una finestra di GitPoulpi già aperta (`gitpoulpi launch`). Su Windows non ha una finestra di console, quindi un clic non ne fa mai lampeggiare una. Viene installato, aggiornato e rimosso insieme agli altri.

## Cosa serve

- Un **`git` di sistema** nel tuo `PATH`. GitPoulpi usa libgit2 per la maggior
  parte delle operazioni e il tuo `git` per il resto.
  Serve la versione **2.34 o successiva**. Su macOS va bene il `git` dei Command Line Tools di Apple, e anche quello di Homebrew.
- Un **terminale** con supporto del mouse. kitty, Ghostty, WezTerm, Rio, iTerm2 (e anche Windows Terminal, foot e Tabby, tramite sixel)
  disegnano il grafo dei commit come immagini. Gli altri terminali (la
  console di Windows, zellij, screen e la maggior parte degli altri) mostrano un grafo a
  caratteri. Vedi [Scegliere un terminale](terminals.md) e [Multiplexer](multiplexers.md).
- Un **Nerd Font** o un font con le emoji, per le icone della barra degli
  strumenti e della barra laterale.
- Facoltativo: **`git-lfs`** 3.0 o successivo, per i repository Git LFS. Senza,
  i file LFS compaiono come piccoli file puntatore e GitPoulpi ti spiega
  perché. Vedi [Git LFS](lfs.md) per come installarlo.
- Facoltativo: un **credential helper** (Git Credential Manager, libsecret,
  `pass`, Keychain), per conservare i token degli hosting e le chiavi API
  dell'IA. Vedi [Integrazioni di hosting e issue tracker](hosting-integration.md).

## Da un archivio di release

Un archivio di release è `gitpoulpi-<version>-<target>.zip` (Windows) o
`.tar.gz` (Linux, macOS), con accanto un file `.sha256`. Verifica l'archivio
con il file `.sha256`, decomprimilo e metti i tre eseguibili in una cartella
nel tuo `PATH` (tieni `gitpoulpi-updater` e il marcatore `.gitpoulpi-dist` accanto a
loro: il marcatore dice `channel=archive`, quindi la build può aggiornarsi da sola,
vedi [Aggiornamenti](updates.md)). `INSTALL.txt` nell'archivio riporta gli stessi passaggi.

### Verificare un download

Ogni file di rilascio è firmato con la chiave [minisign](https://jedisct1.github.io/minisign/) del progetto, e GitPoulpi controlla da sé quella firma prima di installare un aggiornamento. Per un file scaricato a mano, il `.minisig` accanto è la firma. La chiave pubblica (id chiave `B461FFCBF6F6A4F2`) è inclusa nel repository come `packaging/minisign.pub`:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Leggi il commento attendibile che il comando stampa: deve essere `file:<il nome del file> version:<la versione che ti aspetti>`. Una firma valida ma che nomina un altro file o un'altra versione non è una firma per il tuo.

Dove la release è stata compilata con le attestazioni di provenienza della build (la variabile `ATTESTATIONS` del repository), la GitHub CLI verifica dove è stato compilato un file:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Il file `.sha256` intercetta solo un download danneggiato. La firma è ciò che mostra che il file è del progetto.

## Con un installer

Ogni release include anche un installer per ogni sistema. Entrambi fanno le stesse scelte
(collegamenti, PATH, un alias del comando come `gp`) chiamando lo stesso `gitpoulpi`
installato, cioè il codice che sta dietro Impostazioni > Launcher, così ciò che
l'installer configura e ciò che mostrano le Impostazioni coincidono sempre.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

La prima finestra chiede per chi è l'installazione:

| Scelta | Dove | Diritti di amministratore | Cosa significa per gli aggiornamenti |
|---|---|---|---|
| **Solo per me** (predefinita) | `%LOCALAPPDATA%\Programs\GitPoulpi` | nessuno | GitPoulpi sostituisce i propri file: **Aggiorna e riavvia** nel [pannello degli aggiornamenti](updates.md), senza richieste. |
| **Per tutti gli utenti** | `Program Files\GitPoulpi` | una richiesta all'installazione | Ogni aggiornamento chiede l'approvazione di un amministratore (una richiesta di Windows), e le sessioni di GitPoulpi degli altri utenti sul computer vengono chiuse prima. |

La stessa scelta funziona in modo silenzioso: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` oppure `/ALLUSERS`.

Le caselle: collegamento nel menu Start, collegamento sul Desktop, aggiunta al PATH. Una pagina chiede l'**alias del comando**: `gp` di base, un nome qualsiasi, vuoto per nessuno (`/ALIAS=nome` o `/ALIAS=` da riga di comando). PowerShell usa già `gp` per `Get-ItemProperty`, quindi lì scegli un altro nome oppure usa `gitpoulpi`.

L'installer copia gli eseguibili (compreso `gitpoulpi-updater`, il piccolo programma che scambia i file durante un aggiornamento) e il marcatore `.gitpoulpi-dist`. Il marcatore registra come è stato installato GitPoulpi (`inno-user` o `inno-machine`), ed è ciò che il pannello degli aggiornamenti mostra come **Installato con**. Eseguire di nuovo il setup mantiene le tue scelte (ambito, collegamenti, alias).

Si disinstalla da Impostazioni di Windows > App: esegue `gitpoulpi uninstall` (alias, voci del PATH, collegamenti, voci del menu del file manager) ed elimina i file, compresi gli avanzi `.old` dell'aggiornamento. Le tue impostazioni e i tuoi repository restano.

### Linux e macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Sceglie l'archivio per il tuo sistema e la tua CPU, ne controlla il `.sha256`, installa (canale `archive`: GitPoulpi si aggiorna sul posto) in
`~/.local/opt/gitpoulpi` e collega `gitpoulpi` e `gitpoulpi-mcp` in `~/.local/bin`. Non serve
root e non viene toccato nulla fuori dalla tua home. Senza `--yes` chiede dell'alias (di base
`gp`), del PATH e dei collegamenti.

| Opzione | Effetto |
|---|---|
| `--prefix DIR` | Installa i file del programma in `DIR` (assoluta). |
| `--version X.Y.Z` | Installa questa release invece dell'ultima. |
| `--archive FILE` | Usa un archivio locale; `FILE.sha256` deve stargli accanto (`--no-verify` lo salta). Non si usa la rete. |
| `--alias NOME` / `--no-alias` | Fa avviare GitPoulpi con `NOME` (`gitpoulpi alias create`), oppure nessun alias. |
| `--no-path` | Non aggiunge GitPoulpi al PATH delle nuove shell (altrimenti `gitpoulpi path add`). |
| `--desktop` | Crea anche i collegamenti del menu delle applicazioni e del Desktop. |
| `--yes` | Non chiede mai nulla: valgono le opzioni e i valori predefiniti (alias `gp`, PATH sì, collegamenti no). |
| `--uninstall` | Esegue `gitpoulpi uninstall`, poi rimuove i collegamenti e la cartella. |

La ricerca dell'ultima release richiede che il repository sia pubblico; per uno privato
scarica l'archivio con un browser e usa `--archive`.

## Con un gestore di pacchetti

Ogni canale qui sotto scrive lo stesso marcatore `.gitpoulpi-dist` accanto ai programmi, con il proprio nome di canale. GitPoulpi lo legge per sapere a chi spettano gli aggiornamenti: spettano al gestore di pacchetti, quindi GitPoulpi non scambia mai i file alle sue spalle. Il pannello degli aggiornamenti (clicca la versione nella barra del titolo) esegue allora il comando del gestore di pacchetti, oppure lo mostra così puoi copiarlo. Vedi [Aggiornamenti](updates.md).

| Canale | Installazione | Note |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Si installa per tutti gli utenti, quindi gli aggiornamenti chiedono l'approvazione di un amministratore. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` poi `scoop install gitpoulpi` | Per utente, senza diritti di amministratore. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Il tap è `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | vedi [il repository apt](#apt-debian-e-ubuntu) | Repository firmato; un `.deb` è allegato anche a ogni release. |
| **MSI** (Windows, gestito) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Per gli amministratori: tutti gli utenti, nessun aggiornamento dall'app. Vedi [Distribuzione aziendale](enterprise-distribution.md#msi-con-intune-o-criteri-di-gruppo). |
| **cargo** | `cargo binstall gitpoulpi-cli` | Scarica l'archivio di rilascio invece di compilare. |
| **cargo, da crates.io** | `cargo install --locked gitpoulpi-cli` | Compila; richiede una toolchain Rust. Installa `gitpoulpi`, che senza sottocomando apre l'interfaccia a terminale e fa anche da server MCP (`gitpoulpi mcp`). `gitpoulpi-tui` e `gitpoulpi-mcp` sono gli stessi programmi come crate autonomi. |
| **cargo, dai sorgenti** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compila; richiede una toolchain Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Vedi [Eseguire in Docker](docker.md). |

I nomi qui sopra sono quelli su cui pubblica il processo di rilascio (vedi [Releasing](../../developer/releasing.md)): un canale non ancora attivo risponde «not found», e l'archivio di rilascio o l'installer funzionano sempre.

Si applica la gestione del PATH propria di ciascun gestore di pacchetti: Chocolatey, Scoop e Homebrew mettono da soli i programmi nel PATH, e nessuno di loro crea l'alias `gp` (esegui `gitpoulpi alias create gp`, oppure Impostazioni > Launcher).

### apt (Debian e Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Il repository supporta `amd64` e `arm64` ed è firmato con una chiave che appartiene al progetto. I programmi vanno in `/usr/lib/gitpoulpi` con collegamenti in `/usr/bin`. Il pacchetto dipende da `git` e raccomanda `git-lfs`. Senza il repository, installa il `.deb` di una release con `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi ti dice allora di scaricare tu il `.deb` successivo, perché offre comandi `apt` solo quando trova il repository in `/etc/apt/sources.list.d/gitpoulpi.*`.

## Dai sorgenti

Serve una toolchain Rust stabile recente (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Gli eseguibili sono in `target/release/`. Il profilo `release` compila
rapidamente. Per gli eseguibili più piccoli e veloci usa il profilo `dist`, che
richiede più tempo di compilazione:

```
cargo build --profile dist
```

Gli script `build-opti` nella radice del repository eseguono questa build per i
tre eseguibili e ne stampano le dimensioni:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

Per compilare solo le lingue e le funzioni che usi, vedi
[Build personalizzate](custom-builds.md).

## Crea tu stesso un archivio di release

```
cargo run -p xtask -- dist
```

Compila il profilo `dist` e scrive
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) o `.tar.gz` (altri
sistemi). L'archivio contiene i tre eseguibili, `INSTALL.txt`,
`README.md`, `LICENSE` e `CHANGELOG.md`. Accanto viene scritto un file
`.sha256`.

## Aggiornamenti

La barra del titolo mostra la versione in uso (`v1.0.0`) tra il nome dell'applicazione e il percorso del repository. Quando è nota una release più recente (o un aggiornamento è in corso), un clic sulla versione apre il pannello degli aggiornamenti con la versione, come è stato installato GitPoulpi e cosa fare per la nuova release; altrimenti un clic apre la scheda Note di rilascio. `Ctrl+N` e poi `u`, oppure **Aggiornamenti: Mostra aggiornamento** nella palette dei comandi (`Ctrl+P`), aprono sempre il pannello degli aggiornamenti, e **Aiuto: Note di rilascio** apre sempre il changelog.

Una volta al giorno, in background, GitPoulpi controlla nel luogo in cui sono pubblicate le sue release (di base GitHub) se ce n'è una più recente (`updates.mode`, [Impostazioni > Sincronizzazione](settings.md#sincronizzazione)). Quando c'è, il chip riporta `v0.2.0 disponibile` nel colore d'accento.

Cosa fa il pannello dipende da come è stato installato GitPoulpi: un archivio di rilascio o l'installer di Windows sostituisce i propri file, un gestore di pacchetti esegue il proprio comando, un'installazione gestita o in un container dice come gli aggiornamenti la raggiungono. Una build fatta con `cargo build` non si aggiorna mai. La descrizione completa, con il ritorno alla versione precedente, `auto` contro `notify` e ciò che un amministratore può bloccare, è in [Aggiornamenti](updates.md).

## Verifica che funzioni

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` stampa la versione e il commit da cui è stata fatta la build. Poi
segui [Per iniziare](getting-started.md).

## Disinstallazione

`gitpoulpi uninstall` (ciò che eseguono entrambi gli installer) rimuove gli alias del comando, le voci
del PATH e i collegamenti creati da GitPoulpi, poi i file del programma di un'installazione da release;
rifiuta di agire su una build senza il marcatore `.gitpoulpi-dist` (una build di cargo), e `--keep-files`
lascia i file del programma. Non è disponibile per gli agenti MCP. Senza un installer, elimina i tre
eseguibili. GitPoulpi conserva anche questi file:

| Cosa | Dove |
|---|---|
| Impostazioni (`config.toml`) e sessioni | La cartella di configurazione del tuo sistema operativo. `GITPOULPI_CONFIG_DIR` la cambia. |
| Stato per repository | `.gitpoulpi/` dentro ogni repository. |
| Cache (avatar) | La cartella della cache del tuo sistema operativo. |
| Token e chiavi API | Il tuo credential helper. **Impostazioni > SSH e credenziali > Dimentica tutti i nomi utente e le password** li rimuove. |

Vedi il [Riferimento della configurazione](config-reference.md) per i percorsi esatti.
