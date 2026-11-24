Una **sequenza** è un elenco con nome di comandi che vengono eseguiti uno dopo l'altro: per
esempio *Stage di tutte le modifiche*, *IA: Genera il messaggio di commit*, *Commit*, *Push*
come un solo tasto **Backup**. Ogni passo è un comando della
[palette dei comandi](keybindings.md); una sequenza è una riga della palette a sé
(`Sequenza: <nome>`), può essere associata a un tasto, e si modifica in
**Impostazioni > Sequenze** o in `config.toml`.

## La pagina Sequenze

**Impostazioni > Sequenze** è una [pagina a elenco](settings.md#elenchi): le sequenze
a sinistra, le righe di quella selezionata a destra. Ogni modifica viene applicata e
scritta subito, e la palette e la mappa dei tasti la seguono senza riavvio.

```
 Sequenze
+- Sequenze --------------+  Modifica della sequenza 1/2
| Questa installazione    |  Nome         Backup
| > Backup        Alt+B   |  Id           backup
|   Worktree e agente     |  Descrizione  Stage di tutto, ...
| Questo repository       |  Ambito       Questa installazione
|   Release               |  Scorciatoia  [ Alt+B ] [ Rimuovi ]
+-------------------------+  Passi        Stage > IA: ... > Commit > Push
| [+ Aggiungi v] [Dup]    |              [ Modifica i passi (4) ] [ Esegui ora ]
| [Rimuovi] [Su] [Giù]    |
+-------------------------+
```

* **+ Aggiungi** offre **Sequenza vuota** e le tre preimpostazioni, **Da una
  preimpostazione > Backup**, **Componi e push** e **Worktree e agente**. Una
  preimpostazione è un punto di partenza: nulla è attivo finché non la aggiungi, e ciò che
  aggiungi è una copia che puoi modificare. Le preimpostazioni hanno il nome nella lingua
  dell'interfaccia quando vengono aggiunte.
* **Duplica**, **Rimuovi** (chiede prima), **Su** / **Giù** o un trascinamento riordinano
  l'elenco (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). L'ordine
  dell'elenco è l'ordine in `config.toml`.
* Una riga con un problema è segnata in rosso e dice perché (nessun passo, un passo che questa
  versione non conosce, una pausa `Mai` a cui manca l'opzione che le serve).
  Una sequenza con problemi **si può salvare**, i passi futuri sconosciuti vengono mantenuti
  così come sono, ma la sua riga della palette resta disattivata finché non viene corretta.

### Le righe di una sequenza

| Riga | Cosa fa |
|---|---|
| **Nome** | Ciò che mostra la palette: `Sequenza: <nome>`. |
| **Id** | Il nome breve usato in `config.toml`, da riga di comando e come tasto della scorciatoia (`a-z`, `0-9`, `_`, `-`). Rinominarlo sposta con sé la scorciatoia, nella stessa scrittura. |
| **Descrizione** | Facoltativa, per te. |
| **Ambito** | Vedi sotto. |
| **Scorciatoia** | Un pulsante che mostra il tasto. `Enter` o un clic apre la stessa cattura del tasto di **Tastiera e mouse**: i conflitti e gli avvisi del terminale compaiono mentre premi, e un tasto che usa un'altra azione viene rifiutato allo stesso modo. **Rimuovi** lo toglie. Il tasto viene salvato come `[keybindings.sequence] <id>`. |
| **Passi** | I passi in ordine. **Modifica i passi** li apre. |
| **Esegui ora** | Esegue la sequenza nella finestra corrente. |

### Ambito

Ogni sequenza ha un ambito, salvato con essa:

| Ambito | Dove viene salvata | Chi la vede |
|---|---|---|
| **Questa installazione** (predefinito) | `[[sequences]]` in `config.toml`, con questa installazione come proprietaria | Solo questa installazione di GitPoulpi. |
| **Tutte le installazioni su questo computer** | `[[sequences]]` in `config.toml`, senza proprietario | Ogni installazione che condivide il file. |
| **Solo questo repository** | `.gitpoulpi/sequences.toml` nel repository corrente | Chiunque abbia il repository. Conta solo dopo che l'hai considerata attendibile (vedi sotto). |

Cambiare l'ambito sposta la definizione: scegliere **Solo questo repository**
la scrive nel file del repository e la rimuove da `config.toml`, e
viceversa. Un ambito di repository richiede un repository aperto.

### Sequenze del repository e fiducia

Il file `.gitpoulpi/sequences.toml` può essere incluso in un commit, quindi può venire da
altre persone, e una sequenza esegue comandi sul tuo computer. Le sequenze di un repository
quindi **non vengono offerte finché non le consideri attendibili**, come i
comandi di un file di workflow in un commit.

Quando un repository ha un file del genere che nessuno ha ancora considerato attendibile, compare un avviso sopra
il Log: *Questo repository definisce 2 sequenze* con **Rivedi**, **Autorizza** e
**Ignora**. Non blocca mai nulla.

- **Rivedi** elenca ogni sequenza con ogni passo, la sua pausa e le sue opzioni,
  in sola lettura. **Autorizza** lì considera attendibile ciò che è stato elencato; **Chiudi** (o `Esc`)
  non considera attendibile nulla. Se l'elenco è più lungo della finestra, Autorizza non viene offerto
  lì (ingrandisci la finestra): non consideri mai attendibile ciò che non hai potuto leggere.
- **Autorizza** (il pulsante, o il comando della palette *Sequenza: Considera attendibili le sequenze di questo
  repository*) considera attendibile il file esattamente com'è ora.
- **Ignora** nasconde l'avviso finché il file non cambia.
- Da tastiera: la palette ha *Sequenza: Rivedi / Considera attendibili / Ignora le sequenze di questo
  repository*.

Ciò che viene registrato è un digest degli id, dei comandi, delle pause e delle
opzioni, tenuto nello stato locale del repository (mai nel repository). Se
uno di questi cambia, per esempio dopo un pull, l'avviso torna come *La sequenza di questo repository è cambiata* (*Le N sequenze di questo repository sono cambiate*) e le sequenze lasciano la palette e i tasti
finché non le consideri di nuovo attendibili. Rinominare una sequenza o modificarne la descrizione
non cambia nulla. Una sequenza che scrivi tu nelle Impostazioni resta attendibile quando
il file era già attendibile (o vuoto) prima della tua modifica.

Una volta attendibili, le sequenze del repository compaiono come `Sequenza: <nome>` nella
palette come le tue e possono avere una scorciatoia nelle Impostazioni. Una sequenza con
lo stesso id di una delle tue è uno scontro: vince la tua e l'altra viene segnalata
come duplicato.

Da riga di comando: `gitpoulpi sequence list` mostra le sequenze del file
con `trusted` e uno stato `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (e `untrust`, `--repo PATH` per un altro
repository) è per una persona, da eseguire dopo aver letto `sequence show`.
`sequence run` rifiuta una sequenza di repository non attendibile. Gli agenti IA non possono
considerarle attendibili: non esiste uno strumento MCP per farlo.

## Passi

**Modifica i passi** sostituisce l'elenco con i passi della sequenza (il titolo
dice **Passi di <nome>**) e le righe con quelle del passo selezionato. **Indietro**
(la croce in cima alle righe) o `Esc` torna alle sequenze.

* **+ Aggiungi** apre **Aggiungi un passo**: ogni comando che una sequenza può eseguire, con il suo
  nome nella palette (quelli dell'IA con la scintilla), filtrato mentre scrivi con la
  corrispondenza della palette. Un passo che questa versione non può eseguire è disattivato con il
  motivo. `Enter` o un clic lo aggiunge in fondo; `Esc` o la croce chiude
  il popup.
* **Duplica**, **Rimuovi**, **Su** / **Giù** e il trascinamento funzionano come per le
  sequenze.
* Un comando distruttivo (eliminare un branch, scartare modifiche...) non viene mai
  offerto: una sequenza non può eseguirne uno.

### Pausa

Ogni passo ha una **Pausa**:

| Pausa | Significato |
|---|---|
| **Predefinita** | Il passo si ferma solo quando serve una risposta, nella sua solita finestra di dialogo (il nome di un branch, un messaggio di commit ancora vuoto...). |
| **Sempre** | Si ferma prima di questo passo e chiede di continuare. |
| **Mai** | Non si ferma mai. Le opzioni del passo devono fornirgli tutto ciò che serve, quindi un passo che chiede un nome richiede che la sua opzione **Nome** sia compilata. |

Un passo che chiede sempre prima di essere eseguito (un push forzato) non offre
**Mai**: la riga dice perché. **Mai** sul passo del messaggio IA è consentito
come scelta esplicita.

### Opzioni

Le righe sotto **Pausa** sono le opzioni del passo: un testo **Nome** o **Messaggio**,
un **Branch**, una casella come **Apri in una nuova scheda**, o un menu a tendina
compilato dalla tua configurazione (un **Remote**, un **Profilo IA**, una voce di **Apri
in**). I testi accettano le variabili `${branch}`, `${date}`, `${time}` e
`${n}` (un numero che rende unico il nome). Svuotare un testo rimuove
l'opzione.

## Creare e modificare con l'IA

Con l'[IA attivata](ai.md#bozza-di-sequenza) (Impostazioni > IA), compaiono due voci:
**Aggiungi > Crea con l'IA...** (anche la riga della palette `Sequenza: Crea con l'IA...`,
che apre le Impostazioni su questa pagina con la finestra di dialogo) e **Modifica con l'IA...** accanto
a **Esegui ora** per la sequenza selezionata. Nessuna delle due esiste mentre l'IA è disattivata.

```
+- Crea una sequenza con l'IA ------------------------------- x -+
| Descrivi la sequenza che vuoi                                  |
| +------------------------------------------------------------+ |
| | stage di tutto, scrivi il messaggio di commit con l'IA,    | |
| | commit, poi push                                           | |
| +------------------------------------------------------------+ |
| Enter aggiunge una riga. Ctrl+Enter invia. Esc chiude.         |
| [ ] Invia il contesto (nomi dei remote, profili IA, etichette) |
|    No: viene inviata solo la tua descrizione.                  |
| L'IA si limita a proporre. Nulla viene salvato prima che...    |
| [ Invia ] [ Annulla ]                                          |
+----------------------------------------------------------------+
```

Scrivi ciò che vuoi con parole tue; **Invia** (o `Ctrl+Enter`, `Ctrl+S`)
interroga il modello, tramite lo stesso profilo, lo stesso indicatore dei job e lo stesso `Esc` di ogni
altra azione IA (e la stessa domanda «Cosa viene inviato»). La risposta è una
**proposta** che leggi prima che accada qualsiasi cosa:

* il nome, l'id, la descrizione e i passi con la loro pausa e le loro opzioni;
* per **Modifica con l'IA**, un diff rispetto alla sequenza attuale: `+` un passo aggiunto,
  `-` un passo rimosso, `~` un passo la cui pausa o le cui opzioni sono cambiate, `~ Nome: "a"
  -> "b"` per un campo;
* **Esclusi**: un passo che il modello ha proposto e che non è consentito (sconosciuto,
  distruttivo, una sequenza dentro una sequenza, una pausa impossibile), con il
  motivo; i passi validi vengono mantenuti;
* le **note** del modello, e una **scorciatoia suggerita** che viene solo mostrata, con
  l'indicazione se il tasto è già occupato. Non viene mai associata: usa poi il pulsante
  **Scorciatoia**.

```
+- Modifica proposta --------------------------------------- x -+
| + aggiunto   - rimosso   ~ modificato                         |
| ~ Nome: "Daily" -> "Daily 2"                                  |
| Passi (3)                                                     |
| + 1. Git: Fetch di tutti i remoti                             |
|   2. Git: Stage di tutte le modifiche                         |
| - Git: Commit...                                              |
| ~ 3. Git: Push: predefiniti -> Pausa: Sempre                  |
| Esclusi                                                       |
| passo 4 `reset_hard`: i passi distruttivi non sono consentiti |
| [ Accetta ] [ Modifica prima di salvare ] [ Rifiuta ] [Indietro] |
+---------------------------------------------------------------+
```

* **Accetta** la scrive con un solo salvataggio (una nuova sequenza, o la sostituzione di
  quella modificata, che mantiene id, ambito e tasto) e la seleziona nell'elenco.
* **Modifica prima di salvare** fa lo stesso e apre i suoi passi così puoi ritoccare
  le righe (questa pagina salva ogni modifica subito, quindi la sequenza è già
  salvata).
* **Rifiuta** non scrive nulla. **Indietro** torna alla descrizione per inviarla di nuovo.
* **Accetta** non è disponibile quando non c'è nulla da accettare: nessun passo utilizzabile, un
  problema che la pagina segnalerebbe, o una modifica che non cambia nulla.

Nulla viene scritto prima di **Accetta**, e da qui una sequenza non viene mai eseguita.
Chiudere la finestra di dialogo (la croce, `Esc`, un clic fuori) chiede «Scartare le modifiche?»
mentre è scritta una descrizione o è aperta una proposta; anche chiudere le Impostazioni lo chiede.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<l'id di questa installazione>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Le chiavi che questa versione non conosce, e i passi che non conosce, vengono mantenuti quando
la pagina salva.

## Eseguire una sequenza

Una sequenza è un elenco con nome di passi, ognuno un comando della palette (stage di tutto, scrivere il messaggio di commit con l'IA, commit, push...). Eseguila dalla palette (`Sequenza: <nome>`), con il tasto che le hai dato in Impostazioni > Tastiera e mouse (`[keybindings.sequence]` in `config.toml`), o da riga di comando (`gitpoulpi sequence run`).

Mentre è in esecuzione, una striscia sopra il Log mostra `<nome> k/n`, il passo, e ciò che attende. L'esecuzione compare anche nel pannello Job come `Sequenza <nome> k/n: <passo>`. Una sola sequenza è in esecuzione per volta; avviarne una seconda dice quale è in esecuzione.

**Dove si ferma a chiedere.** Ogni passo ha un'impostazione `pause` (`default`, `always`, `never`):

| Passo | Predefinita | `pause = "never"` |
|---|---|---|
| Messaggio di commit dell'IA | il messaggio finisce nel campo del messaggio; la striscia dice «Rivedi il messaggio, poi Continua» | prosegue subito |
| Commit | un messaggio vuoto si ferma sempre: il campo prende la tastiera, e il tuo commit (o Continua) prosegue | non disponibile quando non c'è un messaggio da committare |
| Compositore di commit IA | si apre la revisione; Applica prosegue, chiuderla annulla | applica la proposta così com'è |
| Crea worktree | la finestra di dialogo si apre con il nome già compilato | lo crea da `options.name` senza la finestra di dialogo |
| Push | quando `git.push_behavior = ask` si apre il selettore «Push verso...» | fa il push verso `options.remote` |
| Push forzato con lease, stage di file che sembrano segreti | chiede sempre | non consentito |

`pause = "always"` su qualsiasi passo mostra «Prossimo: <passo>» con **Continua** e **Annulla** prima che venga eseguito. Se il push viene rifiutato perché il branch è divergente, si apre la solita finestra di dialogo e la striscia dice «Push rifiutato: scegli nella finestra di dialogo»; fare pull e poi push (o un push forzato con lease) continua l'esecuzione, chiudere la finestra di dialogo la annulla.

**Continua** è il pulsante della striscia, `Sequenza: Continua` nella palette, o il tasto associato a `sequence.continue`. **Annulla** è la croce di chiusura della striscia, `Sequenza: Annulla` (`sequence.cancel`), il pannello Job, o `Esc` quando nient'altro lo vuole (una finestra di dialogo aperta si chiude prima, e questo annulla comunque). Annullare mantiene i passi già eseguiti e lo dice: «<nome> annullata dopo il passo k/n (<passo>); i passi precedenti restano».

**Quando un passo fallisce** l'esecuzione si ferma lì: la riga di stato nomina la sequenza, il passo e l'errore, la striscia diventa rossa con **Dettagli** (la voce del Log delle attività, che elenca le opzioni del passo) e una croce di chiusura, e i passi precedenti restano eseguiti.

**Worktree.** Un passo di creazione del worktree con `open_tab = true` porta l'esecuzione nella scheda del nuovo worktree: i passi successivi (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) agiscono lì. Se è visibile un'altra scheda la striscia dice «in attesa della scheda <nome>» con **Vai lì**. Chiudere quella scheda annulla l'esecuzione. Un'esecuzione non sopravvive alla chiusura di GitPoulpi.

### Un agente che propone un'esecuzione

Con *Lascia che gli agenti IA propongano modifiche da confermare* attivo (`[mcp]
allow_ui_propose`), un agente può chiedere alla tua finestra di eseguire una sequenza
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Compare un banner:
*✦ Proposta da <agente>: esegui la sequenza <nome>* con **Esegui** e **Ignora**.
Nulla viene eseguito finché non premi Esegui (cliccalo, oppure `Ctrl+N` poi `Enter`);
Ignora, `Esc` (quando nient'altro lo vuole) o il comando della palette *Agente: Chiudi la vista in sospeso* lo scarta. L'esecuzione poi procede esattamente come se l'avessi avviata tu,
fermandosi dove dice la sequenza. La proposta viene rifiutata quando la sequenza
non esiste, non è valida, o è una sequenza di repository che non hai reso attendibile.

## Annullare un'esecuzione

Ogni passo che cambia il repository (stage, commit, tag, branch, stash, worktree...) lascia la **propria normale voce di Annulla**, marcata con l'esecuzione, quindi `Ctrl+Z` annulla ancora un passo alla volta e la [Cronologia annullamenti](undo-redo.md#la-cronologia-annullamenti) mostra `⟳ <nome>` su di esse. Per riprendere indietro l'intera esecuzione c'è **Annulla l'intera sequenza**: il pulsante della Cronologia annullamenti (`x`), la riga della palette `Modifica: Annulla l'intera sequenza`, o `gitpoulpi undo --sequence`. Annulla le voci dell'esecuzione dalla più recente, ognuna con i propri controlli e le proprie domande (un commit già inviato chiede prima), si ferma al primo rifiuto indicando il passo, e lascia le più vecchie nel journal. Il messaggio dell'IA e un push non lasciano alcuna voce: un push non viene mai annullato. Viene toccato solo il journal del repository corrente; i passi eseguiti in un worktree si annullano da quel repository. Ripeti resta per singola voce. Dettagli: [Esecuzioni di sequenze](undo-redo.md#esecuzioni-di-sequenze).
