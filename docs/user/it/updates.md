[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · **Italiano** · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Aggiornamenti

GitPoulpi cerca le release più recenti e, a seconda di come è stato installato, sostituisce da sé i propri file oppure affida l'aggiornamento al programma che lo ha installato. Nulla viene mai sostituito o riavviato senza una tua azione.

## Il chip e il pannello

La barra del titolo mostra la versione in esecuzione (`v1.0.0`) tra il nome dell'app e il percorso del repository. Quando è nota una release più recente il chip diventa `v0.2.0 disponibile` nel colore di accento.

Premi `Ctrl+N` e poi `u`, esegui **Aggiornamenti: Mostra aggiornamento** dalla palette dei comandi (`Ctrl+P`), oppure clicca il chip mentre riporta `disponibile` (o mentre un aggiornamento è in corso): si apre il pannello degli aggiornamenti. Senza una release più recente, un clic sul chip apre invece la scheda Note di rilascio. È una finestra sopra l'app: `Esc`, `q`, la croce di chiusura o un clic all'esterno la chiude. Mostra:

- la versione in esecuzione e **Installato con**: come GitPoulpi è arrivato su questo computer (da questo dipende tutto ciò che segue);
- a che punto è l'aggiornamento (ricerca, download con una percentuale, pronto, non riuscito);
- i pulsanti della procedura di questa installazione, e il comando esatto quando ce n'è uno;
- una riga del tuo amministratore, quando ne ha impostata una.

Ogni pulsante ha un tasto (indicato nella riga di suggerimento del pannello) e un comando della palette, quindi il pannello funziona senza mouse:

| Pulsante | Tasto | Comando della palette |
|---|---|---|
| **Aggiorna e riavvia** (o **Esegui l'aggiornamento**) | `u` | Aggiornamenti: Aggiorna ora |
| **Copia comando** | `c` | Aggiornamenti: Copia comando di aggiornamento |
| **Apri nel terminale** | `t` | Aggiornamenti: Esegui comando di aggiornamento in un terminale |
| **Torna alla versione precedente** | `v` | Aggiornamenti: Torna alla versione precedente |
| **Note di rilascio** | `n` | Aiuto: Note di rilascio |
| **Salta questa versione** | `s` | Aggiornamenti: Salta questa versione |
| **Apri avviso** (solo con un avviso di sicurezza) | `a` | Aggiornamenti: Apri avviso di sicurezza |
| **Cerca ora** | `r` | Aggiornamenti: Cerca aggiornamenti ora |
| **Chiudi** | `Esc` | |

Un comando che non si applica a questa installazione appare in grigio nella palette, con il motivo (ad esempio «questa installazione si aggiorna in un altro modo: vedi «Aggiornamenti: Mostra aggiornamento»»).

## Cosa succede, per installazione

| Installato con | Si controlla su | Cosa fa il pannello | Tornare indietro |
|---|---|---|---|
| Un archivio di rilascio, o `install.sh` | Release di GitHub | **Aggiorna e riavvia**: GitPoulpi scarica e verifica l'archivio, si chiude, sostituisce i propri file e si riapre nello stesso terminale. | **Torna alla versione precedente** rimette al loro posto i file precedenti. |
| Il programma di installazione di Windows, solo per me | Release di GitHub | Lo stesso, senza richieste. Su Windows si riapre in una nuova finestra. | **Torna alla versione precedente** li rimette a posto. |
| Il programma di installazione di Windows, tutti gli utenti | Release di GitHub | Lo stesso, ma Windows chiede l'approvazione di un amministratore. Le sessioni di GitPoulpi degli altri utenti sul computer vengono chiuse, con la perdita di ciò che avevano digitato e non salvato. Se rifiuti, nulla cambia. | **Torna alla versione precedente** li rimette a posto, con la stessa richiesta. |
| Chocolatey | Il feed community di Chocolatey | **Esegui l'aggiornamento** esegue `choco upgrade gitpoulpi -y` (Windows chiede l'approvazione di un amministratore). | Il pannello mostra `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, feed interno | Il feed della tua organizzazione | **Esegui l'aggiornamento** esegue `choco upgrade gitpoulpi -y --source=<feed>`. I parametri del pacchetto non vengono inviati di nuovo, quindi una configurazione distribuita resta intatta. | La stessa riga `choco install` con il feed. |
| winget | Release di GitHub | **Esegui l'aggiornamento** esegue `winget upgrade --id GitPoulpi.GitPoulpi --exact --silent` con i contratti accettati. winget chiede da sé l'approvazione di un amministratore quando l'installazione è per tutta la macchina. | Il pannello mostra `winget install --id GitPoulpi.GitPoulpi --version <previous>`. |
| Scoop | Release di GitHub | **Esegui l'aggiornamento** esegue `scoop update gitpoulpi` dopo che GitPoulpi si è chiuso del tutto. | **Torna alla versione precedente** esegue `scoop reset gitpoulpi@<previous>`. |
| Homebrew | Release di GitHub | **Esegui l'aggiornamento** esegue `brew upgrade gitpoulpi` in questo terminale. | Il pannello mostra `brew install gitpoulpi@<previous>`. |
| apt | Release di GitHub | Il pannello mostra `sudo apt update && sudo apt install --only-upgrade gitpoulpi` con **Copia comando** e **Apri nel terminale**. GitPoulpi non esegue mai `sudo` al posto tuo. Senza il repository apt di GitPoulpi su questo computer dice di scaricare invece il nuovo `.deb`. | Il pannello mostra `sudo apt install gitpoulpi=<previous>`. |
| Un pacchetto MSI | Release di GitHub, o la fonte del tuo amministratore | Nulla da eseguire: l'installazione è gestita dal tuo amministratore, di cui il pannello mostra il messaggio. | Non offerto. |
| L'AUR | Release di GitHub | Il pannello mostra `yay -S gitpoulpi-bin` (oppure `paru`, il primo che trova; senza un helper, i passaggi `git clone` e `makepkg -si`). | Non offerto. |
| cargo | Release di GitHub | Il pannello mostra `cargo binstall gitpoulpi-cli` quando `cargo-binstall` è installato, altrimenti `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | Lo stesso comando con il tag precedente. |
| Un'immagine Docker | Release di GitHub | Nulla da eseguire: un'immagine si sostituisce. Il pannello mostra `docker pull poulpilab/gitpoulpi:<version>` e `:latest`. Vedi [Eseguire in Docker](docker.md). | Il pannello mostra `docker pull poulpilab/gitpoulpi:<previous>`. |
| Un archivio di rilascio in una cartella in cui non puoi scrivere | Release di GitHub | Il pannello dice che GitPoulpi non può sostituirsi da sé lì e rimanda a **Scarica**. | Non offerto. |
| Una build di sviluppo (`cargo build`) | nessuna | Non si aggiorna mai. Il chip mostra la versione e nient'altro; il pannello dice che non c'è alcun registro di installazione. | Non offerto. |

Le righe «mostra» non eseguono mai nulla da sole. **Apri nel terminale** apre il tuo terminale con il comando già digitato, e tu premi Invio.

Le release arrivano sempre dal repository GitHub del progetto, a meno che un amministratore indirizzi il controllo altrove (un feed interno, un mirror o un `latest.json`), vedi [Distribuzione aziendale](enterprise-distribution.md). Le note di rilascio provengono dalla release o dal changelog incorporato in GitPoulpi, a seconda di quale è raggiungibile.

## Notifica e auto

`updates.mode` ([Impostazioni > Sincronizzazione](settings.md#sincronizzazione)):

- **Notifica** (predefinito): una volta al giorno GitPoulpi controlla in background e mostra il chip. Nulla viene scaricato finché non premi il pulsante.
- **Auto**: dove GitPoulpi sostituisce i propri file (archivio di rilascio, programma di installazione di Windows) scarica e verifica anche l'aggiornamento in background non appena ne trova uno. Il chip riporta allora **Riavvia per aggiornare**, e il riavvio resta una tua scelta. Dove un gestore di pacchetti possiede l'aggiornamento, **Auto** si comporta come **Notifica**.
- **Disattivato**: non viene mai fatta alcuna richiesta e il chip mostra solo la versione.

L'impostazione **Canale di aggiornamento** (`updates.channel`) sceglie tra **Stabile** e **Pre-release** (anche le release candidate). Un controllo avviato con **Cerca ora** ignora il limite di una volta al giorno. Senza connessione di rete, o finché il repository è privato e non è memorizzato alcun token github.com, un controllo in background fallisce in silenzio con una riga nel log dell'applicazione.

## Riavvio e cosa vedi dopo

Premere **Aggiorna e riavvia** o **Esegui l'aggiornamento** salva la sessione e chiude la finestra. Un piccolo programma separato, `gitpoulpi-updater`, fa il lavoro, così i file di GitPoulpi non sono in uso mentre vengono sostituiti e un gestore di pacchetti non è bloccato da un genitore in attesa. Prima ferma i daemon in background di GitPoulpi (si chiudono anche le altre finestre e i server MCP del tuo utente), esegue l'aggiornamento, verifica che il nuovo `gitpoulpi --version` stampi la versione attesa e riavvia GitPoulpi: nello stesso terminale su Linux e macOS, in una nuova finestra su Windows. Se la verifica fallisce rimette a posto i vecchi file.

Al successivo avvio la barra di stato mostra una riga:

- `Aggiornato alla v1.4.2`
- `Aggiornamento non riuscito: <motivo>. La versione precedente è ancora installata.`
- `Aggiornamento annullato: l'approvazione dell'amministratore è stata rifiutata. Nulla è cambiato.`

Viene eseguito un solo aggiornamento alla volta; un file di blocco nella cartella dati di GitPoulpi (e uno nella cartella di installazione, quando è scrivibile) ne impedisce un secondo, e un blocco più vecchio di dieci minuti viene ignorato, quindi un aggiornamento andato in crash non blocca mai GitPoulpi per sempre. La CLI e il daemon non partono finché esiste un blocco recente.

## Tornare alla versione precedente

Dove GitPoulpi sostituisce i propri file, **Torna alla versione precedente** rimette a posto i file precedenti e riavvia. Restano come `<name>.old` per un minuto dopo l'avvio della nuova versione, poi vengono eliminati, quindi il ritorno indietro serve per la prima esecuzione dopo un aggiornamento. Dove un gestore di pacchetti possiede l'installazione, il ritorno indietro è il comando della tabella: il pannello lo mostra o lo esegue. Tornare indietro modifica i file di GitPoulpi, non un repository, quindi non fa parte del journal di annullamento di git.

## Cosa viene verificato e rifiutato

Prima di sostituire qualcosa GitPoulpi scarica l'archivio, la sua firma `.minisig` e il suo `.sha256`. Verifica prima la firma, rispetto alla chiave di rilascio del progetto incorporata in GitPoulpi, e il commento attendibile della firma deve nominare questo archivio e questa versione. Poi verifica l'hash. Rifiuta: una release senza `.minisig` (una release più vecchia, pubblicata prima che le release fossero firmate, non può essere installata tramite l'updater, il che include fissare una versione o tornare a una di esse), una firma di un'altra chiave, una firma per un altro file o un'altra versione, una release senza `.sha256`, un hash che non corrisponde, un archivio il cui nome non è quello di questa piattaforma (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), un archivio il cui marcatore nomina un'altra versione o piattaforma, e un link di download fuori dagli host delle release di GitHub (ogni passaggio di reindirizzamento viene controllato) o dal mirror configurato da un amministratore. La sostituzione è tutto o niente: se un altro programma tiene aperto uno degli eseguibili, GitPoulpi nomina il file, non cambia nulla e lascia il download per un altro tentativo. Su Windows, un aggiornamento per tutti gli utenti viene riverificato dal passaggio con privilegi elevati, che scarica da sé `.minisig` e `.sha256` e controlla di nuovo entrambi, così un file sostituito nel frattempo nella cartella dei download dell'utente viene rifiutato. I gestori di pacchetti (Chocolatey, winget, Scoop, Homebrew, apt, AUR) verificano da sé i propri pacchetti: lì GitPoulpi non scarica nulla.

## Avvisi di sicurezza

Accanto a ogni release il progetto pubblica `security.json`: l'elenco degli avvisi di sicurezza (un identificatore, una gravità, le versioni interessate come intervallo, ad esempio `>=1.2.0, <1.4.3`, la versione corretta, un link e un riepilogo di una frase). Lo stesso controllo giornaliero che cerca una nuova versione lo scarica anche, dallo stesso posto della release: la pagina delle release del repository per GitHub, la cartella di `latest.json` per un mirror, GitHub per il feed pubblico di Chocolatey. Un avviso viene usato solo quando `security.json.minisig` si verifica rispetto alla chiave del progetto; un file non firmato o firmato male non viene mai mostrato (una riga nel Registro attività dice che è stato ignorato).

Quando un avviso nomina la versione in esecuzione, il chip diventa rosso e riporta **Avviso di sicurezza: questa versione è interessata**, su ogni installazione, comprese quelle gestite da un gestore di pacchetti. Il pannello degli aggiornamenti elenca allora ogni avviso con gravità, riepilogo, versione corretta e link; **Apri avviso** (`a`, oppure **Aggiornamenti: Apri avviso di sicurezza** nella palette) apre nel browser il link di quello più grave. Dove un gestore di pacchetti possiede l'installazione il pannello aggiunge «Aggiorna con il tuo gestore di pacchetti.»

Con i controlli degli aggiornamenti disattivati (`updates.mode = "off"`), con una policy dell'amministratore il cui `check` è `none`, e per una build creata con `cargo build`, non viene scaricato nulla e non viene mostrato alcun avviso. Il feed NuGet di un'organizzazione non fa contattare GitHub a GitPoulpi: i suoi avvisi provengono solo dal mirror `download_base` della policy.

## Impostazioni bloccate da un amministratore

Un amministratore può impostare la modalità di aggiornamento, il canale, una versione fissata, dove controllare e se GitPoulpi possa aggiornarsi da sé, in una policy valida per tutta la macchina. Nelle Impostazioni quelle righe mostrano **Impostato dal tuo amministratore**, il pannello mostra il loro messaggio e, quando l'aggiornamento automatico è disattivato, il pannello offre i comandi o una nota invece dei pulsanti che sostituiscono i file. I file di policy illeggibili falliscono in modo sicuro: solo notifica, nessun aggiornamento automatico. Il lato dell'amministratore è in [Distribuzione aziendale](enterprise-distribution.md#il-file-di-policy-e-il-registro-di-sistema).

## Ambiente

`GITPOULPI_UPDATE_API` indirizza il controllo a un'altra radice dell'API (usata dai test).
