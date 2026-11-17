[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · **Italiano** · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# Assistenza IA

GitPoulpi non richiede mai un account, e **l'IA è disattivata per impostazione
predefinita**. Quando l'IA è attiva, può scrivere messaggi di commit e di stash,
spiegare le modifiche, scrivere le descrizioni delle pull request, suggerire
risoluzioni dei conflitti di merge e dividere le modifiche in corso in più
commit. Il modello è tuo: una chiave API di tua proprietà, un server locale o
una CLI di agente già installata (come `claude -p`).

Ogni controllo IA mostra la `✦` viola. Finché l'IA è disattivata:

- Non viene inviato nulla da nessuna parte.
- Non compare alcun controllo IA: nessun pulsante `✦`, nessun comando `IA:`
  nella palette, nessuna riga di menu IA e nessun suggerimento di tasti IA. I
  tasti IA non fanno nulla.
- La pagina IA del repository mostra solo «L'IA è disattivata (Impostazioni > IA)».
- Il server MCP non elenca gli strumenti `ai_*`. I comandi CLI `gitpoulpi ai ...`
  rifiutano con lo stesso messaggio.

**Attiva l'IA** ha effetto subito. Non serve riavviare.

Ci sono due modi di usare l'IA con GitPoulpi:

| | IA integrata (questa pagina) | Un agente esterno |
|---|---|---|
| Che cos'è | GitPoulpi chiede a un modello un messaggio di commit, una spiegazione, ... | Claude Code, Cursor, Codex... pilotano GitPoulpi come uno strumento |
| Si configura in | Impostazioni > IA (`[ai]`) | la configurazione MCP dell'agente, limiti in `[mcp]` |
| Guida | qui sotto | [MCP e agenti](mcp-and-agents.md) |

## Configurare l'IA

1. `Ctrl+,` > **IA** > spunta **Attiva l'IA**.
2. I profili sono l'elenco della scheda **Profili (connessioni)**, a sinistra della
   pagina (impilato in alto su un terminale stretto): scegline uno per modificarlo a destra (`Enter` entra
   nelle sue righe; il comando della palette **IA: Modifica profilo...** porta
   all'elenco). **+ Aggiungi** ne crea un altro, **Duplica** copia quello
   selezionato (senza la sua chiave) e **Rimuovi** chiede prima conferma.
   Scegli il **Provider** e compila i suoi campi:

   | Provider | Campi | Valori predefiniti |
   |---|---|---|
   | **Agente locale** (nessuna chiave, nessun account proprio) | **Tipo di agente** (quale agente è), **Comando** (il programma esatto: nel tuo `PATH`, un alias o uno script wrapper, o un percorso completo), gli **Argomenti** che lo rendono non interattivo, e **Variabili d'ambiente** facoltative. Il prompt va sul suo stdin, la risposta si legge dallo stdout. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **Chiave API**, **Modello** e **URL di base** facoltativi | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **Chiave API**, Modello, URL di base | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **Chiave API**, Modello, URL di base | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **Compatibile con OpenAI** (Ollama, LM Studio, vLLM, un gateway aziendale) | **URL di base** (`.../v1`), **Modello**, chiave facoltativa | `http://localhost:11434/v1` (Ollama; LM Studio è `http://localhost:1234/v1`), `llama3` |

   **Preimpostazioni** (puoi sempre digitare un altro valore):

   ![L'elenco Tipo di agente di un profilo di agente locale, con Claude Code, OpenAI Codex CLI, Gemini CLI, opencode e le altre CLI di agenti](../images/settings-ai-presets.png)

   - **Agente locale > Tipo di agente** è un elenco delle CLI di agenti più
     comuni, le più usate per prime: Claude Code, OpenAI Codex CLI, Gemini CLI,
     opencode, Cursor Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose,
     Crush, Amp, oppure *Altro (nessun suggerimento)*. Il tipo è **separato dal
     comando**: sceglie solo quali suggerimenti ricevi. Sceglierne uno
     precompila **Comando** (finché è ancora il valore predefinito del tipo
     precedente) e **Argomenti** con l'invocazione headless (`claude -p`,
     `codex exec -`, `gemini -p ...`, ...). **Comando** è testo libero: scrivi
     `/opt/bin/work-claude`, uno script wrapper o la destinazione di un alias e
     il tipo, con le sue opzioni, l'elenco dei modelli e le variabili, resta.
     Quando **Tipo di agente** è vuoto, GitPoulpi lo deduce dal comando.
   - **Opzioni degli argomenti** aggiunge o rimuove le opzioni adatte all'uso
     one-shot di GitPoulpi, ciascuna con una breve motivazione: nessuno
     strumento, nessuna sessione salvata, output in testo semplice, poco
     ragionamento, modalità di sola lettura, nessuna richiesta di permessi,
     nessun controllo degli aggiornamenti né statistiche d'uso. L'elenco cambia
     da agente ad agente perché gli agenti sono diversi (solo alcuni possono
     disattivare gli strumenti dalla riga di comando; per opencode serve una
     variabile d'ambiente). **Modello** elenca i modelli di quell'agente e
     scrive l'opzione giusta (`--model haiku` per Claude Code, `-m gpt-6-luna`
     per Codex, `--model provider/model` per opencode); *(predefinito - nessuna
     opzione)* la rimuove. Amp non ha un'opzione per il modello. `aider` legge
     il prompt da `/dev/stdin`, quindi funziona solo su Linux e macOS.
   - Le **Variabili d'ambiente** valgono solo per il processo di quell'agente,
     mai per GitPoulpi o per altri profili. **Aggiungi una variabile
     d'ambiente** propone le variabili documentate che contano per quell'agente,
     ciascuna con la sua funzione (disattivare telemetria e controllo degli
     aggiornamenti, livello di impegno, cartella di configurazione, chiave API,
     URL di base o host locale, ...); puoi anche digitare `NOME` o
     `NOME=valore`. Ogni variabile ha poi la propria riga **Env NOME**: il suo valore
     è un pulsante (premilo per digitarne uno nuovo; si salva quando lasci il
     campo) e un pulsante **[x]** elimina la variabile. Un valore può essere `${env:NAME}` per prendere in prestito
     una delle variabili di GitPoulpi stesso (comodo per le chiavi API, così non
     finiscono in `config.toml`). I valori che sembrano segreti (nomi che
     finiscono in `_KEY`, `_TOKEN`, `_SECRET`, ... o prefissi di chiave noti)
     appaiono come pallini e si digitano da zero. I valori non vengono mai
     registrati nei log.
   - **Provider HTTP > Preimpostazione del provider** compila l'URL di base e un
     modello per OpenAI, Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek,
     Mistral, Groq, Together, Fireworks, Cerebras, Perplexity, Moonshot, Qwen
     (DashScope), Zhipu (GLM) e i server locali Ollama, LM Studio, llama.cpp,
     vLLM, Jan e LocalAI. **Modello** è un elenco dei modelli di quel provider,
     con testo libero ammesso.

3. Premi **Prova la connessione**: *Ultima prova* riporta `connesso: il modello
   ha risposto` oppure `non riuscita: <motivo>`.
4. Con più profili, scegli il **Profilo predefinito** in fondo; ogni azione può
   comunque indicare il proprio.

![La parte bassa del blocco Profili della pagina IA per un agente locale: Modalità di esecuzione, Argomenti, Modello, Cartella di lavoro, Timeout, due righe Env, Aggiungi una variabile d'ambiente e Prova la connessione](../images/settings-ai-agent.png)

Le **chiavi API** si inseriscono con `Enter` sulla riga **Chiave API** e sono
conservate nel credential helper del sistema (Git Credential Manager,
libsecret / `pass`, Keychain) sotto `ai:<id del profilo>`, mai in `config.toml`.
Senza un helper, GitPoulpi lo dice invece di far finta di conservarla.
*Impostazioni > SSH e credenziali > Dimentica tutti i nomi utente e le password*
le rimuove.

Lo stesso in `config.toml` (tutto il resto è nel [riferimento della
configurazione](config-reference.md#ai-ia-integrata-disattivata-per-impostazione-predefinita)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # free: an alias, a wrapper script, an absolute path
harness = "claude"        # which agent it is; empty = inferred from `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # this agent's process only; ${env:NAME} borrows ours
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Modalità di esecuzione: headless o terminale virtuale

Un profilo di agente locale esegue il proprio agente in due modi, scelti con
**Modalità di esecuzione** (mostrata per Claude Code, OpenAI Codex CLI e Gemini
CLI):

- **Headless (non interattiva)**, quella predefinita: la modalità a colpo
  singolo dell'agente (`claude -p`), prompt su stdin, risposta su stdout.
- **Interattiva (terminale virtuale)**: GitPoulpi avvia la schermata propria
  dell'agente in un terminale nascosto, vi digita la richiesta come farebbe una
  persona, attende la risposta e chiude l'agente. Alcuni agenti fatturano o
  limitano la modalità headless in modo diverso da quella interattiva; quale
  caso ti riguardi, e che cosa dicano le condizioni del provider su ciascuna
  modalità, dipende dal provider: consulta le sue.

Ogni modalità conserva i propri **Argomenti**, **Opzioni degli argomenti** e
**Modello**: passare da una all'altra non fa perdere nulla, e la modalità
interattiva propone solo le opzioni che hanno senso senza l'opzione headless.
Altre due righe valgono per entrambe le modalità:

- **Cartella di lavoro**: *Cartella dedicata* è una cartella vuota di proprietà
  di GitPoulpi, quindi l'agente non vede alcun progetto e ti chiede una sola
  volta di fidarti. *Repository corrente* dà all'agente il contesto proprio del
  progetto (il suo `CLAUDE.md`, per esempio), e chiede la fiducia una volta per
  repository. *Predefinita* è la cartella dedicata in modalità interattiva e,
  in modalità headless, la cartella da cui gira GitPoulpi, come prima.
- **Timeout (secondi)**: per quanto tempo l'agente può lavorare prima che
  GitPoulpi rinunci e lo fermi. Se vuoto, sono 120 secondi in headless e 180 in
  interattiva.

**Alle domande del primo avvio rispondi tu.** Un agente che non è mai stato
eseguito in una cartella chiede prima qualcosa (fidarsi della cartella,
accedere, scegliere un tema). GitPoulpi non risponde mai al posto tuo: la
richiesta si ferma con un messaggio, e **Apri l'agente...** (sul profilo, o
*IA: Apri l'agente* nella palette dei comandi) apre l'agente una volta, in un
terminale visibile, nella cartella di lavoro che userà. Rispondi alla domanda,
chiudi l'agente e riprova; la risposta viene ricordata dall'agente.

La risposta viene letta dalla trascrizione della sessione dell'agente per
Claude Code (il suo testo esatto, cancellata poi) e dalla schermata tra due
righe marcatore richieste nella richiesta, per Codex e Gemini. Al massimo due
esecuzioni in terminale virtuale avvengono contemporaneamente, e premere Esc
durante una richiesta ferma l'agente.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # the headless set, kept
run_mode = "terminal"               # headless (default) | terminal
terminal_args = ["--effort", "low"] # the interactive set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 or absent: 120 headless, 180 terminal
```

## Istruzioni

Testo che aggiungi a ciò che viene detto al modello. Il prompt integrato di ogni
azione (che fissa il formato della risposta) viene sempre per primo e non si può
modificare; poi quattro livelli, dal più generale al più specifico, ciascuno solo
se non è vuoto:

1. **Istruzioni globali** (ogni azione). I chip *Inserimento rapido*
   dell'editor inseriscono frasi comuni: *Rispondi in <lingua>*,
   *Risposte concise*, *Usa l'imperativo*, *Tono tecnico*, *Tono informale*.
2. **Istruzioni dell'area di lavoro** (ogni azione, ogni repository
   dell'[area di lavoro](workspaces.md#istruzioni-ia-di-unarea-di-lavoro)): per
   tutte le azioni e, se vuoi, per singola azione. Si modificano nel modulo
   Modifica dell'area di lavoro.
3. **Istruzioni del repository** (ogni azione, solo questo repository), con
   *Combina con il globale*: **Aggiungi al globale** (dopo di esse) o
   **Sostituisci il globale** (al posto loro).
4. **Istruzioni per l'azione** (*Generazione del messaggio di commit*,
   *Spiega le modifiche*, *Generazione del messaggio di stash*, *Pull request*,
   *Compositore di commit*, *Risoluzione dei conflitti*, *Revisione del codice*,
   *Dividi un commit*), ogni blocco con i
   propri chip nel suo editor (ad es. *Formato conventional commits*,
   *Limite di 50 caratteri*) e il proprio elenco **Profilo** (`Predefinito (<nome>)` segue il profilo
   predefinito).

`Enter` su una riga *Istruzioni personalizzate* apre un editor su più righe
(`Ctrl+S` salva, `Esc` annulla). Sopra il testo, una riga *Inserimento rapido*
contiene i chip di quella sezione: clicca su uno per inserire la sua frase al
cursore. Da tastiera, `Tab` passa alla riga, `Sinistra`/`Destra` scelgono un
chip, `Enter` o `Spazio` lo inserisce e torna al testo (`Tab` o `Esc` tornano
senza inserire). Un chip la cui frase è già nelle istruzioni appare attenuato e
inserirlo di nuovo non fa nulla. Il testo è un editor completo: il clic posiziona
il cursore, un trascinamento o `Shift`+frecce selezionano, `Ctrl+A` seleziona
tutto, `Ctrl+C` / `Ctrl+X` copiano e tagliano la selezione, `Ctrl`+frecce
saltano di parola, `Home` / `Fine`, `PagSu` / `PagGiù` e la rotellina del mouse
scorrono un testo lungo. La pagina stessa mostra solo un'anteprima di ogni
istruzione.

![La pagina IA: istruzioni globali, poi i blocchi Generazione del messaggio di commit e Spiega le modifiche con un elenco dei profili e istruzioni personalizzate](../images/settings-ai.png)

**Per repository**: *Impostazioni > Impostazioni specifiche del repository > IA*
(con un repository aperto) contiene le istruzioni del repository e, per ogni
azione, un profilo e istruzioni che prevalgono su quelli globali. Un campo vuoto
usa l'impostazione globale (mostrata in grigio al suo interno); **Torna al
globale** rimuove un override. Salvato nel `.gitpoulpi/repo-state.toml` del
repository, e rispettato anche dalla CLI.

![La pagina IA del repository: Istruzioni del repository con l'elenco Combina con il globale, poi un profilo e le istruzioni per ogni azione](../images/settings-repo-ai.png)

### Come si combinano i livelli

L'ordine di invio è: prompt integrato, **globale**, **area di lavoro**,
**repository**, poi le istruzioni proprie dell'azione. La modalità di un livello
decide cosa succede a quelli sopra di lui:

- **Aggiungi** (il predefinito): il testo del livello viene aggiunto dopo quelli
  più generali.
- **Sostituisci**: il testo del livello viene usato *al posto di* ogni livello
  più generale (il globale, e l'area di lavoro quando a sostituire è il
  repository). Un livello senza testo non sostituisce mai nulla.

Le istruzioni proprie dell'azione seguono la stessa idea: quelle globali per
quell'azione, poi quelle dell'area di lavoro per quell'azione (aggiunte dopo, o
al loro posto), e l'override proprio di un repository per l'azione vince su
entrambe. L'area di lavoro usata è quella attiva nella finestra quando contiene
il repository, altrimenti la prima area di lavoro che lo contiene; da riga di
comando è quella in `GITPOULPI_WORKSPACE`, altrimenti la prima.

Per vedere esattamente cosa viene inviato, livello per livello, esegui
`gitpoulpi ai prompt --action <azione> --repo .` (`base`, `global`, `workspace`,
`repository`, `task`, e il `system_prompt` finale). Non chiama nessun modello e
funziona anche con l'IA disattivata. Imposta le istruzioni di un'area di lavoro
da uno script con `gitpoulpi workspace set-ai-instructions <id> [--action <a>]
[--mode append|replace] --text ...` (o `--clear`).

### Variabili del prompt

Ogni testo di istruzioni (globale, dell'area di lavoro, del repository, per azione) può contenere
variabili `${nome}`. Vengono sostituite dal loro valore subito prima dell'invio
del prompt (il testo salvato conserva i `${...}`). Un nome che non è nella
tabella viene inviato così com'è e l'editor lo segnala; scrivi `$${` per un
`${` letterale.

| Variabile | Si espande in | Esempio |
|---|---|---|
| `${uilang}` | La lingua dell'interfaccia, con il suo nome inglese | `French` |
| `${repo}` | Il nome della cartella del repository | `gitpoulpi` |
| `${branch}` | Il branch corrente | `main` |
| `${ref}` / `${commit}` | Sha breve del commit a cui si riferisce l'azione (*Spiega*), altrimenti HEAD | `4c0443d` |
| `${commit_subject}` | Riga dell'oggetto di quel commit | `feat: add search` |
| `${date}` | Data locale, ISO | `2026-10-03` |
| `${time}` | Ora locale, `HH:MM` | `14:05` |
| `${datetime}` | Data e ora locali | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | Il remoto predefinito (`origin`, altrimenti il primo) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (un altro host) o `none` | `github` |
| `${files}` | Numero di file nel diff dell'azione | `3` |
| `${action}` | L'azione in corso | `commit_message` |

Nell'editor delle istruzioni: digita `${` e un popup elenca le variabili
corrispondenti (`Su`/`Giù` scelgono, `Enter` o `Tab` inserisce, `Esc` lo
chiude); oppure usa l'elenco **Variabili** in basso a sinistra (`Ctrl+T`, o
*IA: Inserisci variabile del prompt* nella palette dei comandi), che inserisce
al cursore. Ogni voce ha una descrizione, mostrata nell'elenco e sulla riga
sopra i pulsanti mentre ci passi sopra o la evidenzi.

## Cosa può fare

| Azione | Nella finestra | CLI |
|---|---|---|
| **Messaggio di commit** | Metti in stage le modifiche, premi la `✦` a destra del campo Riepilogo, `Ctrl+G i`, o il comando della palette `IA: Genera il messaggio di commit`. Riepilogo e descrizione vengono compilati perché tu li modifichi; non viene fatto alcun commit. | `ai commit-message` (stampa), `commit --ai` (fa il commit) |
| **Spiega le modifiche** | La `✦` in alto a destra del pannello delle modifiche in corso (`Ctrl+G w`: il commit selezionato, altrimenti le modifiche in corso), o la voce *Spiega le modifiche (IA)* del menu di un commit. Un popup: le frecce scorrono, `c` copia, `Esc` chiude. | `ai explain [rev] [--wip]` |
| **Messaggio di stash** | `Ctrl+G m` apre la finestra dello stash; il suo pulsante IA (`Ctrl+G` nella finestra) interroga il modello. Il comando della palette `IA: Genera il messaggio di stash` fa entrambe le cose. | `stash push --ai` |
| **Titolo e descrizione della pull request** | *Genera titolo e descrizione* nel modulo Crea una pull request (i commit e il diff del branch vengono letti in locale). | |
| **Risoluzione dei conflitti di merge** | Nell'editor dei conflitti di merge: il pulsante `✦ Suggerisci` della barra degli strumenti, la voce del menu con clic destro di un conflitto *✦ Suggerisci una risoluzione (IA)*, `Alt+E`, o il comando della palette `IA: Suggerisci una risoluzione del conflitto di merge`. Il suggerimento sostituisce la regione del *conflitto corrente* nel risultato come un'unica modifica (segnata come modificata, un solo passo di annullamento); nulla viene salvato o messo in stage finché non salvi. | `ai resolve-conflict <path> [--conflict N]` (stampa) |
| **Revisione del codice** | Nella [revisione di una pull o merge request](review.md#bozza-di-revisione-ia): il pulsante `✦ Bozza di revisione...`, `s`, o il comando della palette `Revisione: ✦ Bozza di revisione...` (apre le opzioni di revisione, `Enter` avvia). Il modello legge il diff, la descrizione e i commenti esistenti della richiesta (e, per impostazione predefinita, il `CLAUDE.md` e l'`AGENTS.md` del progetto: vedi [Contesto della revisione](#contesto-della-revisione)) e propone dei commenti; tu accetti, modifichi o ignori ciascuno. Nulla raggiunge l'host prima che tu invii la revisione. | `hosting review suggest <n>` (stampa JSON, non pubblica nulla), `ai review-context` (i valori predefiniti) |
| **Dividere un commit** | La voce *Dividi commit (IA)...* del menu di un commit, o il comando della palette `IA: Dividi il commit selezionato...`. L'editor di divisione si apre con il raggruppamento del modello già compilato; vedi sotto. | `ai split-plan <rev>` (stampa) |
| **Script di riscrittura della cronologia** | Nel [pannello di riscrittura della cronologia](history-rewrite.md#scrivere-uno-script-con-lia): il pulsante `✦ Scrivi con l'IA...` dell'editor degli script, o il comando della palette `Riscrittura: Scrivi uno script con l'IA...`. Una finestra mostra esattamente cosa verrà inviato; lo script arriva nell'editor, contrassegnato, e non viene mai eseguito né applicato al posto tuo. | `ai rewrite-script "<istruzione>" [--sample N]` (stampa) |
| **Regole di riscrittura della cronologia** | Nel [pannello di riscrittura della cronologia](history-rewrite.md#creare-regole-con-lia): il pulsante `✦ Crea regole con l'IA...`, `Alt+E`, o il comando della palette `Riscrittura: Crea regole con l'IA...`. Una finestra su più righe mostra esattamente cosa verrà inviato; le regole e le impostazioni arrivano nel pannello e nulla viene mostrato in anteprima né applicato al posto tuo. | `ai rewrite-rules "<descrizione>" [--context]` (stampa) |
| **Nome del worktree** | Nella [finestra Crea un worktree](worktrees.md#dare-un-nome-con-lia): il pulsante `✦ Nome con l'IA` o `Alt+A` apre una casella dove descrivi la funzionalità, il bug o l'attività; il modello dà un nome al worktree e al suo branch seguendo i nomi dei branch del repository, ed entrambi finiscono nei campi della finestra, da modificare. Vengono inviati solo la tua descrizione e i nomi dei branch locali, quindi non c'è una finestra «Cosa viene inviato». | `ai worktree-name --describe "<task>"` (stampa) |
| **Risposta su un host** e **Descrizione della issue** | Il pulsante **Scrivi con l'IA** (`Alt+E`) di ogni compositore Markdown che invia a un host: un commento, una risposta o una modifica in una conversazione, un commento o un riepilogo di revisione, la descrizione di una nuova issue. La bozza finisce nella casella; nulla viene pubblicato. Vedi [Aiuto alla scrittura su un host](#aiuto-alla-scrittura-su-un-host). | `ai hosting-reply` (stampa una bozza) |
| **Compositore di commit** | La voce *Componi i commit (IA)...* del menu della riga WIP, `Ctrl+G C`, il comando della palette `IA: Componi commits dalle modifiche in corso...` | |

`Esc` annulla una richiesta in corso (il processo di un agente locale viene terminato).

### Risoluzione dei conflitti di merge

Il modello riceve il percorso e il linguaggio del file, i blocchi base, corrente
(A) e in entrata (B) del conflitto con le etichette che mostra l'editor, fino a
20 righe di testo già unito prima e dopo, e le tue istruzioni (globali, del
repository, e quelle di *Risoluzione dei conflitti* in Impostazioni > IA).
Risponde con il testo risolto di quel solo conflitto in un blocco di codice; una
risposta con marcatori di conflitto o chiacchiere al posto del codice viene
rimandata indietro una volta, poi rifiutata. Un conflitto di oltre
40 KB (`ai.limits.conflict_bytes`, *Impostazioni > IA > Limiti*) viene trattenuto dietro
un avviso che ne indica la dimensione e il limite, con *Invia comunque* (solo questa richiesta,
comunque ripulita dai controlli di privacy), *Apri impostazione* e *Annulla*; il limite è una
protezione di GitPoulpi, quindi alzalo quando il tuo modello ha spazio. I controlli sono nascosti finché l'IA è disattivata, e un
suggerimento non salva, non mette in stage e non risolve mai nulla: rivedi il
risultato, modificalo e salva come al solito.

![Il modulo di commit dopo il pulsante scintilla: un riepilogo e una descrizione scritti dal modello, pronti da modificare](../images/ai-commit-message.png)

![Il popup Spiega le modifiche in corso: una spiegazione e un elenco di cose da ricontrollare](../images/ai-explain.png)

### Il compositore di commit

Il modello divide le modifiche in corso in più commit (ogni file modificato in
uno solo; un piano non valido viene rimandato indietro una volta, poi rifiutato). Finché
ci sono file in conflitto non viene chiesto nulla: una finestra li elenca e apre la vista di
risoluzione dei conflitti, perché il compositore ha bisogno di un albero senza conflitti.
Rivedi il piano prima che venga creato qualsiasi commit:

![La revisione del compositore: quattro commit proposti con i loro file, e i pulsanti Reimposta, Annulla e Crea 4 commit](../images/composer.png)

| Tasto | Mouse | Fa |
|---|---|---|
| `Su` / `Giù` (`k` / `j`) | clic su una riga | si sposta tra commit e file |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | le frecce, trascinamento | sposta un commit; sposta un file in un altro commit |
| `Destra` / `Sinistra`, `*` | la freccia della riga | espande / comprime un commit, tutti (`ui.composer_files` imposta lo stato iniziale) |
| `e` / `Enter` | la matita | modifica il messaggio |
| `d` / `Delete` | `drop` / `ripristina` | lascia fuori un commit (i suoi file restano non committati) |
| `n` | `+ commit a sé` | sposta un file in un commit a sé |
| `r` | `Reimposta` | torna al piano del modello |
| `Ctrl+S` / `a` | `Crea N commit` | li crea, in ordine |
| `Esc` | `Annulla` | esce, senza fare nulla |

La creazione mette in stage i file di ogni commit e li committa uno dopo l'altro;
la directory di lavoro non viene mai toccata. Un file messo in stage solo in parte
viene committato per intero. Se un passo fallisce, si ferma e dice quanti commit
sono stati creati.

### Dividere un commit

Il modello legge le modifiche del commit, numerate come le numera l'[editor di divisione](log-and-history.md#dividere-un-commit),
e propone più commit, ciascuno con il suo messaggio; può raggruppare hunk e singole righe.
Una risposta che lascia fuori una modifica, ne assegna una due volte o nomina qualcosa che
non esiste viene rifiutata. In ogni caso si apre l'editor di divisione: con la proposta già
compilata oppure, se il modello ha fallito, con il motivo in una nota, così dividi a mano.
Non viene diviso nulla finché non premi **Dividi** (`Ctrl+S`); se il raggruppamento non va
bene, prima sposta le modifiche, rinomina i commit o aggiungine uno. Il profilo e le
istruzioni sono quelli di *Dividi un commit* in Impostazioni > IA.

### Uno script di riscrittura della cronologia

Descrivi cosa deve fare la riscrittura («sposta alla mattina dopo ogni commit fatto di
notte», «sostituisci la mia vecchia e-mail con la nuova, solo prima del 2024») e il
modello scrive uno [script Lua](history-rewrite.md#script-lua) per la regola Script,
con l'intestazione, `rewrite_commit(c, ctx)` e le funzioni di supporto della sandbox.
Il daemon compila la risposta; se non compila, l'errore torna una volta al modello.
Ciò che ne risulta compare nell'editor: leggilo, provalo, poi genera l'anteprima.
Dettagli e finestra sono in [Riscrittura della cronologia](history-rewrite.md#scrivere-uno-script-con-lia).
Il profilo e le istruzioni sono quelli di *Script di riscrittura della cronologia* in
Impostazioni > IA.

### Regole di riscrittura della cronologia

Descrivi con parole tue come deve essere riscritta la cronologia («sostituisci la mia vecchia
e-mail a@b.org con c@d.org», «rimuovi secrets.txt da tutta la cronologia») e il modello propone
un ambito, alcune impostazioni e un elenco di [regole di riscrittura](history-rewrite.md#creare-regole-con-lia)
nel vocabolario del pannello. **Prepara soltanto**: la risposta riempie il pannello, nulla viene
mostrato in anteprima né applicato, e una voce inutilizzabile viene elencata con il motivo, mentre
quelle valide vengono mantenute. Una regola Script non viene mai creata in questo modo. Se vuoi,
vengono inviati anche l'ambito, i nomi dei branch e dei tag e gli autori dei commit più recenti
(una casella, disattivata per impostazione predefinita, visibile nel riquadro di ciò che viene
inviato). Il profilo e le istruzioni sono quelli di *Regole di riscrittura della cronologia* in
Impostazioni > IA.

### Aiuto alla scrittura su un host

*Risposta su un host* abbozza un commento, una risposta in una discussione, un commento su una riga di codice o il riepilogo di una
revisione; *Descrizione della issue* abbozza la descrizione di una nuova issue. **L'IA scrive solo una bozza.** Il testo va nel
compositore (o viene stampato dalla CLI) perché lo legga, lo modifichi e lo invii tu. Non esiste alcun percorso di codice
da questa funzione a una scrittura sull'host: non può pubblicare, modificare, risolvere né eliminare nulla, la risposta di CLI e MCP dice `"posted": false`,
e un agente locale gira in sola lettura (vedi sotto). Il testo lascia GitPoulpi solo tramite il pulsante **Invia**
(`Ctrl+Enter`) o **Commenta** del compositore.

**Nel compositore Markdown.** Con l'IA attiva, ogni compositore il cui testo va a un host (un commento, una risposta o una
modifica nella [vista di conversazione](conversations.md), un commento di revisione, il riepilogo di Termina la revisione,
la descrizione di una nuova issue e quella di una pull request) ha un pulsante **Scrivi con l'IA** (`Alt+E`, lo
stesso tasto in tutti) all'estremità destra della sua barra delle modalità. Apre un pannello sotto la barra:

- un campo, *Cosa voglio in questa risposta*, scritto al momento dell'uso (`Enter` abbozza). Il suo segnaposto dà un
  esempio del genere («d'accordo, e chiedi un test che fallisce»). Un elenco **Recenti** conserva le istruzioni precedenti (clicca
  una, oppure `Up` / `Down` nel campo; conservate in `ai-hints.toml`);
- *Migliora la mia bozza*, spuntata quando la casella contiene già un testo: l'IA parte da quello invece di scriverne uno nuovo;
- *L'IA può leggere il repository*, **non spuntata ogni volta** a meno che tu non abbia attivato il suo valore predefinito (vedi sotto);
- **Abbozza**, **Ferma** (anche `Esc`) e **Chiudi**, e una riga per l'avanzamento o l'errore.

Il consueto dialogo «Controlla cosa viene inviato» (`[ai.sent_preview] hosting_reply`, `issue_body`, `pull_request_description`) mostra
la richiesta prima che parta. Oltre al testo di sistema e dell'utente mostra un elenco **Contesto del repository** quando il
repository viene letto: ogni parte con la sua dimensione, il totale rispetto al limite (*31.2 KB su 48 KB*), ciò che è stato ridotto per rientrare nel limite,
ciò che è stato escluso perché sensibile e quali parti sono riusate. Per un agente locale dice che l'agente legge il
repository da sé con strumenti di sola lettura, li nomina, indica la cartella di lavoro e se la sessione dell'agente è nuova o
continuata, e ogni dialogo finisce con *L'IA scrive solo una bozza. Nulla viene pubblicato finché non lo invii tu.*
La bozza sostituisce il testo in un solo passo di annullamento e il compositore passa ad Anteprima perché tu la legga prima di inviare;
`Ctrl+Z` ti restituisce il tuo testo. La descrizione di una pull request viene scritta dai commit e dal diff del branch, e accetta
la stessa istruzione; la casella del repository è disattivata lì («non per questo testo»).

Cosa riceve il modello: titolo, stato e descrizione dell'elemento, la discussione a cui rispondi, i commenti recenti (i più recenti
per primi, entro `ai.limits.hosting_reply_context_bytes`, 49152 per impostazione predefinita), il markup che l'host accetta (così una
risposta per Jira torna come Markdown che GitPoulpi converte), ciò che vuoi in questa risposta e, per migliorare un tuo testo,
la tua bozza. Tutto ciò che hanno scritto altre persone è segnato come dato non attendibile nella richiesta, quindi un'istruzione nascosta in un
commento non viene seguita. Oltre il limite nulla viene tagliato di nascosto: ti viene chiesto, e i commenti più vecchi vengono esclusi
per primi (la discussione a cui rispondi viene mantenuta, fino a metà del budget); la CLI segnala il taglio in `truncated` e
`--send-all` lo toglie.

**Il tuo prompt di base.** Le istruzioni da cui partono entrambe le azioni si possono modificare: *Impostazioni > IA > Risposte su un host >
Prompt di base* (`ai.hosting_reply.base_prompt`). Vuoto usa il prompt integrato, che chiede solo il testo, nel
markup dell'host e nella lingua della conversazione a meno che il tuo suggerimento dica altro. Le tue istruzioni globali, di workspace e di
repository vengono comunque dopo, e il testo di altre persone resta segnato come non attendibile qualunque cosa tu scriva
lì. `ai prompt --action hosting_reply` stampa i livelli.

#### Accesso al repository

**Disattivato per impostazione predefinita, e spuntato a ogni uso.** Leggere il repository dà risposte molto migliori a «questo crash succede
nel nostro codice?» ma invia di più, quindi è una scelta che fai ogni volta: *L'IA può leggere il repository* parte non spuntata
in ogni casella. `ai.hosting_reply.read_repository` (*L'IA può leggere il repository per impostazione predefinita*, disattivato) fa solo partire la
casella spuntata; `--read-repo` è la spunta della CLI. La scelta è disattivata, con il motivo quando la attivi, in
tre casi: il repository **non è clonato** (la conversazione del Radar di un altro repository: non c'è nulla da
leggere), l'agente locale scelto non ha **nessuna modalità di sola lettura** di cui GitPoulpi si fidi, oppure il testo **non è scritto dal
repository** (la descrizione di una pull request). Solo la tua spunta lo attiva.

Cosa succede poi dipende dal profilo:

| Profilo | Cosa legge il repository | Cosa viene inviato |
|---|---|---|
| **API** (compatibile OpenAI, Anthropic) | GitPoulpi, tramite il daemon, in sola lettura | Un contesto che costruisce e dà al modello: la mappa del repository (l'albero di primo livello e i percorsi attorno ai file trovati), i file che la conversazione nomina (il file intero, o una finestra di 40 righe attorno alla riga o al simbolo), i risultati di ricerca per identificatori e messaggi di errore nella conversazione e, per una pull request, il suo diff. Entro `ai.limits.hosting_reply_*` (contesto 49152, file 8192, mappa 6144, 20 risultati per termine, 8 termini), tagliato nell'ordine diff, file, risultati, mappa e segnalato. I file che sembrano sensibili vengono esclusi e i segreti nel resto sono mascherati prima (vedi [Privacy](#privacy)). |
| **Agente locale** (Claude Code, Codex), headless | L'agente stesso, nella cartella del repository | Non viene costruito nulla: l'agente parte nel repository con **strumenti di sola lettura** e legge ciò che gli serve. Claude Code riceve `Read`, `Grep` e `Glob` e gli sono negati `Bash`, `Edit`, `Write`, `NotebookEdit`, `WebFetch` e `WebSearch`, senza alcun server MCP caricato (il server di un progetto potrebbe pubblicare); Codex gira nella sua sandbox di sola lettura. Le credenziali di hosting del tuo ambiente (`GH_TOKEN`, `GITHUB_TOKEN`, `GITLAB_TOKEN`, le variabili di Jira, Linear, Azure DevOps e Bitbucket, `GIT_ASKPASS`, `SSH_AUTH_SOCK`) vengono rimosse dal suo ambiente, così nemmeno un agente configurato male può pubblicare o fare push. |

Un agente per cui GitPoulpi non ha una modalità di sola lettura confermata (e qualsiasi agente nella modalità di esecuzione a terminale interattivo) non può
essere ammesso: la scelta resta disattivata con *Questo agente non ha una modalità di sola lettura attendibile.*

**Cache e Aggiorna contesto.** Ciò che è stato raccolto viene riusato perché la risposta successiva sia rapida ed economica:

- Il contesto costruito di un profilo API viene conservato per repository per `ai.hosting_reply.context_ttl_minutes` (*Mantieni un
  contesto del repository per*, 30 per impostazione predefinita). È identificato dal repository, dal suo `HEAD`, dalla selezione e dai limiti,
  e le sue parti sono conservate una per una, quindi la issue successiva dello stesso repository ricostruisce solo le parti che non
  condivide. Nella finestra le cache vivono in memoria (gli ultimi 8 contesti); la CLI le conserva in file nella sua cartella di stato
  privata, quindi un secondo `ai hosting-reply --read-repo` riusa ciò che il primo ha costruito.
- Un nuovo commit sul branch in checkout (`HEAD` si muove), un cambio dei limiti o la scadenza del tempo ricominciano da capo. Lo stesso fa
  **Aggiorna contesto**, il pulsante nel pannello quando il repository viene letto, o `--refresh-context` nella CLI. Una riga sotto la casella dice cosa ha fatto l'ultima bozza: *Contesto creato: 5
  parti, 31.2 KB, HEAD a1b2c3d*, *Contesto riusato (4/5 parti, HEAD a1b2c3d)*, *Cache del provider: 12k token riusati*,
  *Repository letto dall'agente (HEAD a1b2c3d)*, *Sessione agente avviata* o *Sessione agente continuata*.
- **Cache dei prompt di Anthropic.** Per un profilo Anthropic la richiesta viene inviata con il prompt di base e le istruzioni come
  un blocco in cache e il contesto del repository come secondo, e dopo di essi la tua conversazione, il suggerimento e la bozza, così
  una seconda risposta sullo stesso contesto viene letta dalla cache del provider e costa una frazione (anche i seguiti mettono in cache
  il messaggio della conversazione e la bozza più recente, vedi sotto). I contatori di cache della risposta sono ciò che
  *Cache del provider: N token riusati* mostra. I server compatibili OpenAI ricevono lo stesso ordine stabile della
  richiesta, di cui beneficia la loro cache automatica dei prefissi; GitPoulpi non invia nulla in più.
- **L'agente mantiene la sua sessione.** Con **Mantieni la sessione dell'agente tra le risposte**
  (`ai.hosting_reply.reuse_agent_session`, attivo per impostazione predefinita) e *L'IA può leggere il repository* spuntato, un agente locale
  che lo supporta (oggi Claude Code) viene avviato con un id di sessione, poi ripreso con esso, così ciò che ha letto del
  repository resta nella sua cronologia e la risposta successiva parte più in fretta. Gli altri agenti girano in modo one-shot. La sessione è per repository, `HEAD` e
  azione; un nuovo commit o **Aggiorna contesto** ne avvia una nuova; con l'opzione disattivata, o la casella non spuntata, ogni risposta
  è un'esecuzione one-shot che non conserva nulla. La sessione vive nell'archivio proprio dell'agente, GitPoulpi conserva solo il suo id
  (in memoria nella finestra, in un file riservato all'utente per la CLI).

**Seguiti: più breve, un'altra istruzione.** Chiedere di nuovo nello stesso compositore ("rendilo più breve", "aggiungi un
grazie") è un unico scambio. Una sessione Claude Code ripresa lo ricorda da sola. Per tutto il resto (un profilo API, Codex e ogni
altro agente) GitPoulpi lo restituisce: la richiesta porta il messaggio della conversazione della prima richiesta, poi ogni istruzione
precedente e la bozza che le ha risposto come veri turni di chat (una trascrizione nel prompt per un agente, che legge un solo
documento), e infine solo ciò che è nuovo: la tua istruzione e, quando *Migliora la mia bozza* è spuntato, il testo ora nella casella
con le tue modifiche. Così un seguito non parte mai da zero e nulla viene riletto dal repository. "Cosa viene inviato" mostra tutto.
Lo scambio è tenuto entro `ai.limits.hosting_reply_context_bytes` come la conversazione: i seguiti più vecchi vengono tralasciati per
primi, il messaggio della conversazione e la tua bozza più recente restano sempre (`context.history_omitted` nella risposta della CLI
li conta). Dentro uno scambio il contesto del repository resta quello della sua prima richiesta, quindi viene riusato e la cache del
provider regge: per un profilo Anthropic il prompt di base, il contesto del repository, il messaggio della conversazione e la bozza
più recente sono la parte in cache, e un server compatibile OpenAI riceve lo stesso ordine dei messaggi per la sua cache dei prefissi.
Codex potrebbe essere ripreso con `codex exec resume <id>`, ma sceglie da sé l'id della sessione e lo stampa solo tra i suoi eventi JSON
(e `--last` riprenderebbe le tue sessioni), quindi GitPoulpi lo tiene one-shot e fornisce di nuovo la cronologia. Lo scambio vive con il
compositore: finisce quando il compositore si chiude o il testo viene inviato, e anche **Aggiorna contesto** lo scarta (la richiesta
successiva è una prima richiesta). Vi entra solo una richiesta che ha risposto; una fallita o interrotta no. La descrizione di una pull
request viene scritta dal diff e non ha uno scambio.

Nella CLI ogni chiamata è separata, quindi `--continue` riprende lo scambio: continua l'ultimo `ai hosting-reply` per lo stesso
obiettivo in questo repository (stessi `--item`, `--reply-to`, `--path`/`--line` o `--issue-title`), da un piccolo file nella cartella di
stato riservata. Senza `--continue` inizia un nuovo scambio che sostituisce quello conservato. `--continue` senza nulla di conservato per
l'obiettivo è un errore. Il `context` della risposta porta `history_turns` (e `history_omitted`) quando sono andati con essa dei turni
precedenti.

```
gitpoulpi ai hosting-reply --conversation conversation.json --hint "say I will look tomorrow"
gitpoulpi ai hosting-reply --conversation conversation.json --hint "shorter" --continue
gitpoulpi ai hosting-reply --conversation conversation.json --reply-to <thread id> --draft "thx will do"
gitpoulpi ai hosting-reply --conversation conversation.json --read-repo --hint "is this our bug?"
gitpoulpi ai hosting-reply --issue-title "Crash on empty config" --hint "steps to reproduce"
```

`--conversation` accetta un file, `-` per lo standard input, o il JSON stesso: il JSON che stampano `hosting issue comments --json` o
`hosting review conversation --json` (con `item` e voci tipizzate), oppure una pagina più vecchia con un `header` e `entries`,
la forma in cui GitPoulpi salva una conversazione. `--item <ref>` recupera la conversazione dall'host da sé (`7` o `#7` una issue,
`!7` una pull request), invece di `--conversation`. Con `--read-repo`, `--pr-base REV
--pr-head REV` mettono nel contesto il diff di una pull request; il `context` della risposta dice quali parti sono state lette, se
sono state riusate, i token di cache del provider e la sessione dell'agente. Lo strumento MCP `ai_hosting_reply` è lo stesso
comando ed è di sola lettura.

## Dove va la chiave

Una chiave API (OpenAI, Anthropic, OpenRouter, un server compatibile con OpenAI) è legata all'indirizzo per cui è stata memorizzata: l'origine dell'URL di base del profilo, registrata in `trust.toml` nella cartella dati privata riservata all'utente. Se l'URL di base in seguito punta altrove, la richiesta viene rifiutata prima che venga inviato qualsiasi cosa e la chiave resta dov'è. Esegui `gitpoulpi ai trust <profile> --accept-origin <url>` (o accettalo nell'app) per spostare di proposito la chiave al nuovo indirizzo.

- `http://` in chiaro è consentito su questa macchina (Ollama, LM Studio su `localhost`); altrove richiede `gitpoulpi ai trust <profile> --allow-insecure-http on`.
- Un server self-hosted il cui certificato non risale a un'autorità nota viene accettato solo per l'impronta della chiave che fissi con `--accept-certificate <sha256>`; l'handshake TLS dimostra comunque che il server possiede quella chiave.
- Un profilo che aveva già una chiave prima dell'introduzione di questo controllo viene registrato in silenzio, dal suo URL di base attuale, la prima volta che viene usato.
- Un agente locale (`claude -p` e simili) non ha chiave né HTTP, quindi non c'è nulla da legare.

## Contesto della revisione

Una bozza di revisione del codice (`✦`, o `hosting review suggest`) legge il diff, il titolo, la descrizione e i
commenti esistenti della richiesta. Può ricevere anche le convenzioni del progetto, in un blocco
**Project conventions** posto prima del diff, e un agente locale può lavorare dentro il repository. I valori
predefiniti sono:

| Impostazione | Predefinito | Significato |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: decide la *Cartella di lavoro* del profilo IA. `yes`: un agente locale viene eseguito nel repository per la revisione, qualunque cosa dica il profilo. `no`: non lì (un profilo impostato su *Repository* usa invece la cartella dedicata). I profili HTTP non hanno una cartella di lavoro, quindi per loro non cambia nulla. |
| `include_claude_md` | attivo | Fornisce al modello il `CLAUDE.md` del repository. |
| `include_agents_md` | attivo | Fornisce al modello l'`AGENTS.md` del repository. |
| `extra_file` | nessuno | Un file in più: un percorso relativo alla radice del repository, un percorso assoluto o un link. |

Si sovrappongono come le istruzioni, con la meglio la più specifica, campo per campo: prima la tabella globale
`[ai.review]`, poi i valori propri del repository (un campo lasciato non impostato eredita), poi le scelte di una
singola esecuzione. Un repository non clonato conserva i propri valori per `host/project` in
`remote-reviews.toml`, accanto alle sue bozze di revisione.

Un repository può anche **disattivare** il file in più globale: il suo `no_extra_file = true` (la casella
**Nessun file in più** in *Impostazioni specifiche del repository > IA*, nella finestra delle opzioni di ✦ Revisione e
il flag `--no-extra-file` di `ai review-context`) significa «nessun file in più per questo repository», qualunque cosa
indichi `[ai.review]`. Un campo **File aggiuntivo** **vuoto** non è la stessa cosa: eredita il file globale. Un file
del repository e l'interruttore si escludono a vicenda (impostare l'uno azzera l'altro), e l'interruttore vince se un
file modificato a mano contiene entrambi. Un vecchio `extra_file = ""` di un repository significa ancora nessuno.

Da dove vengono i file:

- `CLAUDE.md`, `AGENTS.md` e un file in più con percorso relativo vengono letti dalla **radice** del repository
  sul branch di destinazione della richiesta, mai dalla copia di lavoro (che può essere su un altro branch o
  avere modifiche locali). Con un clone, dal commit base della richiesta; senza, dall'API dell'host.
- Un percorso assoluto viene letto da questa macchina.
- Un link a un file di un host di codice (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea e Forgejo `.../src/branch/<branch>/<path>`, oppure un link raw) viene letto tramite il profilo di
  integrazione il cui host corrisponde, così un repository privato funziona con il token che hai già fornito. Un
  link che nessun profilo riconosce, o qualsiasi altro indirizzo, viene scaricato in modo anonimo. Un nome di
  branch con una barra non può essere letto da un link: usa un percorso, oppure un link su un branch senza barra.
- Un `CLAUDE.md` o `AGENTS.md` mancante viene saltato in silenzio; un file in più mancante viene segnalato
  (`context.problems` nel JSON della CLI).
- Ogni file viene tagliato a **8 KB** e i file insieme a **16 KB**, con una nota visibile
  `[CLAUDE.md cut: showing ... of ... bytes]` in ciò che legge il modello.

Quando un agente locale viene eseguito nel repository, il prompt gli dice che i file di cui è stato fatto il
checkout potrebbero non corrispondere all'head della richiesta e che fa fede il diff.

Nella vista di revisione, `s` apre la finestra **✦ Opzioni di revisione** con questi valori già compilati: quelli
del repository sopra quelli globali. Modificali solo per questa esecuzione, oppure spunta *Salva come predefiniti
di questo repository* per conservarli (vedi [Opzioni di revisione](review.md#opzioni-di-revisione)). I valori
predefiniti si modificano anche in *Impostazioni > IA* (il blocco *Revisione del codice*, quattro righe) e, per
repository, in *Impostazioni specifiche del repository > IA*, dove ogni riga può *usare il globale* e un pulsante
*Torna al globale* elimina i valori propri del repository; la ricerca nelle Impostazioni li trova
(`Includi AGENTS.md`, `File aggiuntivo`). Tutto ciò che non si è potuto dare al modello (un file in più mancante)
o che è stato tagliato per rientrare nei limiti viene mostrato sulla scheda della bozza.

Dalla riga di comando:

```sh
gitpoulpi ai review-context                      # i valori globali e ciò che riceve una revisione
gitpoulpi ai review-context --agents-md off      # salva un valore predefinito globale
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # torna a seguire i valori globali
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` non richiede alcun modello e funziona con l'IA disattivata. `ai prompt --action code_review`
stampa il contesto effettivo come `review_context`. Le stesse quattro impostazioni sono `ai.review.*` in
`config.toml` ([riferimento](config-reference.md#ai-ia-integrata-disattivata-per-impostazione-predefinita)).

## Limiti di input

Ogni input che GitPoulpi mette in una richiesta ha un limite di dimensione, un'impostazione
sotto `[ai.limits]` (*Impostazioni > IA > Limiti*). I limiti sono una protezione di GitPoulpi
(la finestra di contesto cambia da modello a modello): alzali quando il tuo modello ha
spazio, abbassali per tenere piccole le richieste.

| Impostazione (`ai.limits.`) | Predefinito | Cosa limita |
|---|---|---|
| `conflict_bytes` | 40000 | I blocchi base, corrente e in entrata di un conflitto di merge |
| `diff_bytes` | 60000 | Il diff di un messaggio di commit, di una spiegazione, del nome di uno stash, della descrizione di una pull request, di un piano di commit o di una divisione; l'intero diff di una revisione del codice |
| `review_file_bytes` | 12000 | Il diff di un file in una revisione del codice |
| `review_text_bytes` | 4000 | La descrizione della pull request in revisione |
| `review_threads_bytes` | 6000 | I commenti di revisione già esistenti |
| `review_context_file_bytes` | 8192 | Un file di contesto del progetto in una revisione (`CLAUDE.md`, `AGENTS.md`, il file aggiuntivo) |
| `review_context_bytes` | 16384 | Tutti i file di contesto di una revisione insieme |
| `hosting_reply_context_bytes` | 49152 | L'elemento e la conversazione dati a una risposta abbozzata |
| `hosting_reply_file_bytes` | 8192 | Un file del repository dato a una risposta abbozzata (quando l'IA legge il repository) |
| `hosting_reply_map_bytes` | 6144 | La mappa del repository data a una risposta abbozzata |
| `hosting_reply_hits_per_term` | 20 | Risultati di ricerca tenuti per termine (da 1 a 200) |
| `hosting_reply_terms` | 8 | Termini di ricerca cercati (da 1 a 50) |

**Nulla viene tagliato in silenzio.** Quando un input supera il suo limite, la richiesta
aspetta e *Oltre il limite IA* elenca cosa lo supera: quale parte (il diff, un file, la
descrizione, un file di contesto), la sua dimensione, il limite e l'impostazione. Una
revisione elenca in una sola finestra ogni parte che supera il limite. I pulsanti, ciascuno
con il suo tasto:

- *Invia tutto* (`T`): solo questa richiesta, nulla viene tagliato. Passa comunque dai
  controlli di privacy (segreti sostituiti, file sensibili esclusi) e, se è attivo, dal
  riquadro *Cosa viene inviato*, che viene subito dopo.
- *Invia troncato* (`C`): ciò che supera il limite viene tagliato, come è sempre stato; ogni
  riga della finestra dice cosa viene escluso (la fine del diff, gli ultimi hunk, i file che
  non entrano).
- *Apri impostazione* (`A`): annulla la richiesta e apre *Impostazioni > IA > Limiti* sul primo
  limite elencato. Il focus parte da qui: Invio non invia mai.
- *Annulla* (`N`, `Esc`, la croce o un clic all'esterno): non invia nulla.

Un conflitto è l'eccezione: non viene mai tagliato. *Invia comunque* (`I`) lo invia per intero
una volta.

Nulla avvia da solo una richiesta IA, quindi non esiste un'esecuzione in background che possa
chiedere: ogni richiesta è una che hai avviato tu. Un'esecuzione che non può chiedere (la CLI
e MCP) tronca come prima e lo dice; vedi sotto.

**Da riga di comando** nulla può rispondere a una finestra, quindi un input oltre il suo
limite viene troncato come prima e la risposta dice cosa è stato tagliato:
`truncated: [{part, detail?, bytes, limit, setting}]` (presente solo quando qualcosa è stato
tagliato; `part` è uno tra `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file`, `context_total`). `--send-all`
(alias `--no-truncate`) invia invece tutto per intero: su `ai` (tutti i suoi comandi, compreso
`ai resolve-conflict`, dove un conflitto oltre il limite altrimenti viene rifiutato), su
`commit --ai`, su `stash push --ai` e su `hosting review suggest`. Gli strumenti MCP hanno lo
stesso argomento `send_all` e lo stesso campo `truncated`.

## Privacy

- Non viene inviato nulla finché non premi un pulsante o esegui un comando.
  Nessuna IA in background, nessuna telemetria, nessun account GitPoulpi.
- Cosa viene inviato: il prompt (quello integrato, poi le tue istruzioni) e il
  diff di cui hai chiesto; per una pull request, anche gli oggetti dei commit del
  branch. Un diff oltre 60 KB (`ai.limits.diff_bytes`) non viene mai tagliato in
  silenzio: vedi [Limiti di input](#limiti-di-input).
- Una richiesta di script di riscrittura della cronologia invia la tua istruzione e il
  contratto fisso dello script (l'API della sandbox degli script). Autore, data e
  oggetto di al massimo 20 commit partono **solo** se spunti *Invia un campione* nella
  finestra (disattivato di base; la CLI richiede `--sample N`). Il contenuto dei file e
  i valori segreti non vengono mai inviati.
- Va solo al profilo scelto per quell'azione: quell'host API, o lo stdin del tuo
  comando di agente locale (che poi fa ciò che fa quell'agente).
- Le chiavi API sono usate dal processo `gitpoulpi` stesso e non raggiungono mai
  il demone, gli altri client della finestra o un client MCP.
- Il testo generato non viene mai committato, inviato (push) né applicato da solo.

### Cosa viene ripulito da una richiesta

Ogni richiesta, qualunque azione la invii e qualunque provider la riceva (un'API
ospitata o un agente locale), passa prima dallo stesso passaggio di pulizia:

- **I segreti vengono oscurati** (`[ai.privacy] redact_secrets`, attivo per
  impostazione predefinita). Un token con un prefisso noto, un blocco di chiave
  privata, un JWT, una password in un URL e una riga `password = valore` il cui
  valore sembra casuale vengono sostituiti da `[REDACTED:<rule>]`. Non è
  esaustivo: trova le forme più comuni (lo stesso scanner di `rewrite scan-secrets`).
- **I file sensibili vengono esclusi** (`[ai.privacy] exclude_sensitive_files`,
  attivo per impostazione predefinita). Il diff di un file che corrisponde a
  `[privacy] sensitive_patterns` (`.env` e `.env.*` tranne i file `.example`,
  `.sample` e `.template`, `*.pem`, `*.key`, le chiavi private SSH, `*.p12`,
  `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`) non fa
  parte della richiesta. Un pezzo di divisione o un elenco di file del compositore
  che nomina un file simile viene tagliato allo stesso modo, quindi un piano che lo
  riguarda può essere rifiutato come incompleto: metti in stage o dividi tu quel file.

Disattivare l'una o l'altra è una scelta, e Impostazioni > IA mostra una riga rossa
sotto l'impostazione finché è disattivata.

**Cosa viene inviato, prima che venga inviato.** Prima che una richiesta parta, la TUI
mostra il dialogo **Controlla cosa viene inviato**: le istruzioni e la richiesta dopo la
pulizia (lo stesso riquadro dei dialoghi di riscrittura della cronologia, scorrevole,
`Alt+W` lo mostra o lo nasconde), quanti segreti sono stati sostituiti e quali file sono
stati esclusi (una regola e un'anteprima mascherata come `ghp_****abcd`, mai un valore),
una casella **Non mostrare più per <azione>**, e **Invia** / **Annulla** (`Esc`, la croce
di chiusura e un clic all'esterno annullano; nulla esce finché non premi Invia). La casella
scrive `[ai.sent_preview] <action> = false`; Impostazioni > IA > *Mostra cosa viene inviato
prima di...* ha un interruttore per azione e **Mostra di nuovo tutto**. I due dialoghi di
riscrittura della cronologia mostrano già la loro richiesta, quindi non hanno interruttore.

**Da riga di comando**, ogni risposta di `ai ...` (e di `hosting review suggest`) porta
`privacy: {redactions: N, excluded_files: [...]}`, e `--print-sent` stampa il testo di
sistema e dell'utente ripulito, come `{sent: [{system, user}], privacy, dry_run: true}`,
senza chiamare il modello (richiede `ai.enabled = true`, nessuna chiave e nessuna rete).

**Un limite.** Un agente locale la cui cartella di lavoro è il repository
(`workdir = repository`, oppure *Esegui l'agente nel repository* per una revisione) può
leggere i file da sé: la pulizia copre ciò che GitPoulpi gli invia, non ciò che apre per
conto suo. Usa la cartella di lavoro `dedicated` per tenerlo fuori dal repository.

## Risoluzione dei problemi

| Sintomo | Rimedio |
|---|---|
| Nessuna `✦` da nessuna parte | L'IA è disattivata: *Impostazioni > IA > Attiva l'IA*. |
| La `✦` è grigia | Non c'è nulla in stage (messaggio di commit). |
| *Prova la connessione*: comando non trovato | Il **Comando** dell'agente locale non è nel `PATH` che vede GitPoulpi: avvia GitPoulpi dove `claude --version` funziona, oppure dai il percorso completo. |
| Un agente locale si blocca o mostra una schermata interattiva | In modalità *Headless*, aggiungi la sua opzione headless (`-p` per Claude Code) agli **Argomenti**, oppure scegli di nuovo il suo **Tipo di agente** per ripristinare l'invocazione headless. La schermata interattiva è lo scopo della modalità *Interattiva (terminale virtuale)*. |
| "Use "Open the agent"..." | L'agente ha posto una domanda del primo avvio (fidarsi della cartella, accedere). Usa **Apri l'agente...** sul profilo, rispondi, chiudi l'agente e riprova. |
| Una richiesta in terminale virtuale va in timeout | Il messaggio termina con le ultime righe della schermata dell'agente: lì compare una richiesta di permesso, un accesso o una domanda di aggiornamento. Rispondi con **Apri l'agente...**, oppure aumenta il **Timeout**. |
| `non riuscita:` 401 | Chiave sbagliata, o chiave di un altro provider: inseriscila di nuovo. |
| `non riuscita:` 404 | Il **Modello** non esiste lì: cancellalo per usare quello predefinito. |
| `non riuscita:` errore di connessione (Ollama, LM Studio) | Il server non è in esecuzione, oppure l'**URL di base** ha la porta sbagliata o non ha `/v1`. |
| La chiave mostra `(non impostata)` dopo averla digitata | Nessun credential helper: configurane uno (`git config --global credential.helper ...`) e inseriscila di nuovo. |
| *Oltre il limite IA* | Un input della richiesta supera un limite di `[ai.limits]`: scegli *Invia tutto*, *Invia troncato*, *Apri impostazione* o *Annulla* (vedi [Limiti di input](#limiti-di-input)). |
| «Questo conflitto è più grande del limite» | Il conflitto supera `ai.limits.conflict_bytes` (40000 di base). *Invia comunque* lo invia una volta; *Apri impostazione* porta al limite in Impostazioni > IA > Limiti. |
| Lingua o stile sbagliati | Aggiungi istruzioni; i chip *Inserimento rapido* dell'editor lo rendono rapido. |
| Il compositore dice che il piano non è valido | Il modello ha saltato o ripetuto un file: riprova o usa un modello più potente. |

## Vecchi file di configurazione

Un vecchio `config.toml` con `[[ai.presets]]` e `[ai.defaults]` si carica ancora:

- Il preset predefinito di ogni azione diventa il suo `[ai.actions.<action>]`
  (`system_prompt` diventa `instructions`).
- Ogni vecchio preset è conservato in `[[ai.legacy_presets]]`. Impostazioni > IA
  li mostra sotto *Vecchi preset*, con *Elimina l'archivio*.
- Il vecchio `[preferences.ai_presets]` di un repository diventa un override del
  repository.

Il prompt integrato ora viene sempre per primo. Se un vecchio prompt copiava un
prompt integrato, rimuovi la parte copiata e tieni solo ciò che vuoi aggiungere.
