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
- Un **terminale** con supporto del mouse. kitty, Ghostty, WezTerm e iTerm2 (e anche Windows Terminal e foot, tramite sixel)
  disegnano il grafo dei commit come immagini. Gli altri terminali (la
  console di Windows, tmux e la maggior parte degli altri) mostrano un grafo a
  caratteri. Vedi [Scegliere un terminale](terminals.md).
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
nel tuo `PATH`. `INSTALL.txt` nell'archivio riporta gli stessi passaggi.

## Con un installer

Ogni release include anche un installer per ogni sistema. Entrambi fanno le stesse scelte
(collegamenti, PATH, un alias del comando come `gp`) chiamando lo stesso `gitpoulpi`
installato, cioè il codice che sta dietro Impostazioni > Generali > Launcher, così ciò che
l'installer configura e ciò che mostrano le Impostazioni coincidono sempre.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Si installa per l'utente corrente, senza richiesta di amministratore, in
`%LOCALAPPDATA%\Programs\GitPoulpi` (la prima finestra propone l'installazione per tutti
gli utenti e la cartella di installazione). Le caselle: collegamento nel menu Start,
collegamento sul Desktop, aggiunta al PATH. Una pagina chiede l'**alias del comando**: `gp`
di base, un nome qualsiasi, vuoto per nessuno. PowerShell usa già `gp` per
`Get-ItemProperty`, quindi lì scegli un altro nome oppure usa `gitpoulpi`.

L'installer copia i tre eseguibili e il marcatore `.gitpoulpi-dist`, quindi
[Aggiorna ora](#aggiornare-sul-posto) funziona esattamente come con lo zip. Un'installazione
per utente è scrivibile, quindi può aggiornarsi da sola; un'installazione per tutti gli
utenti in Program Files richiede privilegi elevati per essere sostituita, quindi reinstallala
con il nuovo setup. Si disinstalla da Impostazioni di Windows > App: esegue `gitpoulpi uninstall`
(alias, voci del PATH, collegamenti) ed elimina i file, compresi gli avanzi `.old` dell'aggiornamento.
Le tue impostazioni e i tuoi repository restano.

### Linux e macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Sceglie l'archivio per il tuo sistema e la tua CPU, ne controlla il `.sha256`, installa in
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

La barra del titolo mostra la versione in uso (`v1.0.0`) tra il nome dell'applicazione e il percorso del repository.
Cliccala, oppure esegui **Aiuto: Note di rilascio** dalla palette dei comandi (`Ctrl+P`):
si apre una scheda Note di rilascio con il changelog, scorso fino alla versione in uso.

Una volta al giorno, in background, GitPoulpi chiede a GitHub se esiste una release più recente
(`updates.mode`, [Impostazioni > Sincronizzazione](settings.md#sincronizzazione)). Quando c'è,
il chip riporta `v0.2.0 disponibile` nel colore d'accento e un clic apre la scheda Note di rilascio
con le note di quella release in cima. La scheda ha tre pulsanti, ciascuno con un tasto e un
comando di palette:

- **Download** (`d`, *Aggiornamenti: Download*) apre la pagina della release nel browser.
- **Salta questa versione** (`s`, *Aggiornamenti: Salta questa versione*) nasconde l'avviso
  finché non compare una release più recente.
- **Cerca ora** (`r`, *Aggiornamenti: Cerca aggiornamenti ora*) chiede di nuovo subito.

`updates.mode = "off"` non fa nessuna richiesta. Senza connessione di rete, o finché il repository
è privato e non è memorizzato alcun token github.com, la ricerca fallisce in silenzio (una riga nel
registro dell'applicazione, nulla nell'interfaccia). `GITPOULPI_UPDATE_API` indirizza la ricerca a
un'altra radice dell'API (la usano i test).

### Aggiornare sul posto

Una build installata da un archivio di release può sostituirsi da sola. L'archivio porta un piccolo
file `.gitpoulpi-dist` accanto agli eseguibili; una build in esecuzione è «aggiornabile» solo quando
quel file sta accanto al suo eseguibile (e ne nomina la piattaforma). Una build fatta con `cargo`,
una build personalizzata copiata altrove o un pacchetto che ha lasciato fuori il file non si
aggiorna mai da sola: la sua scheda Note di rilascio lo dice e offre solo **Download**.

Per una build aggiornabile la scheda aggiunge:

- **Aggiorna ora** (`u`, *Aggiornamenti: Aggiorna ora*) scarica in background l'archivio e il suo
  `.sha256` (un'attività nell'indicatore della barra di stato, con la sua percentuale, annullabile come
  ogni attività), controlla l'hash, decomprime l'archivio e mette ogni nuovo file accanto al vecchio
  come `<nome>.new`. Nulla di ciò che è in esecuzione viene toccato. In modalità `auto` la stessa cosa
  avviene da sola appena una ricerca trova una release più recente.
- **Riavvia per aggiornare** (`R`, *Aggiornamenti: Riavvia per aggiornare*; il chip dice lo stesso
  quando il download è pronto) è sempre una tua azione, mai automatica. Ferma il daemon autonomo della
  sessione (uno incorporato se ne va con la finestra), scambia i file e avvia la nuova versione con gli
  stessi argomenti e la stessa sessione.
- **Torna alla versione precedente** (`v`, *Aggiornamenti: Torna alla versione precedente*) rimette i file della
  versione precedente e riavvia. I file della versione precedente restano come `<nome>.old` finché la
  nuova versione non è in esecuzione da un minuto, poi vengono eliminati, quindi il pulsante serve per
  quel primo avvio. Non fa parte del journal di annullamento di git: cambia i file di GitPoulpi, non un
  repository.

Come vengono sostituiti i file: su Linux e macOS ogni nuovo file viene rinominato sopra il vecchio
(in modo atomico; un programma in esecuzione tiene la sua vecchia copia finché non termina). Su Windows
un `.exe` in esecuzione non si può sovrascrivere, quindi il vecchio viene rinominato in `<nome>.old` e il
nuovo spostato al suo posto; un `.old` ancora in esecuzione viene eliminato da un avvio successivo. Lo
scambio è tutto o niente: se un altro programma tiene aperto uno degli eseguibili (per esempio un client
MCP che esegue `gitpoulpi-mcp`), GitPoulpi nomina il file, non cambia nulla e lascia il download pronto
per un altro tentativo.

Cosa viene rifiutato, ciascuno con un messaggio: una release senza file `.sha256`, un hash che non
corrisponde, un file di archivio il cui nome non è quello della piattaforma (`gitpoulpi-<version>-<target>.<zip|tar.gz>`),
un archivio il cui marcatore nomina un'altra versione o un'altra piattaforma, e qualsiasi link di download
fuori da questi host: `github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` e `github-releases.githubusercontent.com` di GitHub (la prima
richiesta e ogni salto di reindirizzamento; un reindirizzamento altrove interrompe il download). Gli archivi
di un repository privato richiedono una sessione del browser, quindi lì il download fallisce finché il
repository non diventa pubblico.

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
