[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · **Italiano** · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Workflow di branching

Un workflow di branching dice a GitPoulpi come il tuo team usa i branch: quali vivono
per sempre (`main`, `develop`, `production`), quali sono topic branch di breve durata
(`feature/...`, `hotfix/...`), come si chiamano e dove ciascuno parte e
finisce. È Git Flow reso personalizzabile: nomi fissi, prefissi e suffissi, oppure
pattern con segnaposto, così i branch che un bot crea da un'issue vengono
riconosciuti come i tuoi.

Questa pagina tratta il modello, il file, i modelli pronti, il rilevamento e l'importazione, i
comandi di sola lettura, le azioni che avviano, terminano, sincronizzano e promuovono i branch
(CLI, MCP e app), e le schermate dell'app: l'editor delle impostazioni, la barra
laterale raggruppata, il menu `Flow` e le sue finestre.

## Concetti

- Un **branch base** è di lunga durata. Un base può avere un **genitore** (`develop` in
  `main`); un trunk non ne ha. Un base può elencare più nomi candidati
  (`main` o `master`, `develop` o `dev`): si usa quello che esiste.
- Un **tipo di topic** è un genere di branch di breve durata (feature, bugfix, release,
  hotfix, support...). Ha un **pattern del nome**, un **genitore** (da dove parte
  e in cosa viene terminato) e un modo per terminare.
- Ogni branch viene **classificato**: un base (per nome), un topic di un certo tipo (per
  pattern), oppure non classificato.
- Il **genitore di un branch** è, in ordine: quello registrato quando GitPoulpi ha creato
  il branch (`git config branch.<name>.gitpoulpi-parent`), altrimenti il
  `git-town-branch.<name>.parent` di Git Town quando c'è, altrimenti il genitore del suo tipo.

## Dove viene salvato

| Cosa | Dove | Committato |
|---|---|---|
| Il workflow condiviso | `.gitpoulpi/workflow.toml` nel repository | Sì, da te: GitPoulpi lo scrive solo quando lo chiedi |
| Le tue personalizzazioni | `[workflow]` in `.gitpoulpi/repo-state.toml` | No (quel file è ignorato) |

La directory di stato ignora sé stessa (`.gitpoulpi/.gitignore`), tranne
`workflow.toml`, che è fatto per essere committato. Il workflow effettivo è il
file condiviso con sopra la tua personalizzazione: un base con lo stesso nome, o un topic con
lo stesso id, viene sostituito; quelli nuovi vengono aggiunti; `disable_bases` e `disable_topics`
rimuovono quelli condivisi solo per te.

Salvare sopra un file esistente conserva le chiavi che questa versione non conosce (quelle di un GitPoulpi
più recente, o di un collega), ma non i commenti né l'ordine delle chiavi.

## Il file

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informativo)

[[base]]
name = "main"
names = ["master"]             # altre grafie dello stesso branch
protected = true               # mai eliminato, i commit diretti sono sospetti

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # un branch di ambiente (GitLab Flow): promosso a valle

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # altri nomi che corrispondono anch'essi; mai usati per creare
# regex = "^feat-(?P<slug>.+)$"  # corrisponde anch'essa; i gruppi con nome diventano campi
parent = "develop"             # da dove parte, e la destinazione di fine predefinita
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # comandi di shell facoltativi, sempre mostrati prima di eseguirli
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # flussi guidati dall'host, vedi sotto
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` e `hooks` guidano le [azioni](#azioni) qui sotto;
`issue_flow` guida i [flussi guidati dall'host](#flussi-guidati-dallhost).

### Pattern dei nomi

Testo con `{segnaposto}`:

| Segnaposto | Corrisponde a | Reso a partire da |
|---|---|---|
| `{slug}` | testo libero; può contenere `/` se è l'ultimo | il testo, in minuscolo, senza accenti, con `-` tra le parole, al massimo 50 caratteri |
| `{issue}` | cifre (`123`) | `123` o `#123` |
| `{issue_key}` | una chiave del tracker (`PROJ-123`) | come indicata |
| `{user}` | un nome senza `/` | trasformato in slug |
| `{date}` | `2026-10-16` o `20261016` | come indicata |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | come indicata |
| `{n}` | cifre | come indicato |

Tutto il resto è letterale, quindi prefissi e suffissi sono semplice testo:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Ogni
segnaposto può comparire una volta, e due segnaposto hanno bisogno di testo in mezzo. Un
pattern deve produrre nomi di branch validi (le regole di `git check-ref-format`):
niente spazi, niente `~ ^ : ? * [ \`, niente `..`, nessun componente che inizi con `.` o finisca
con `.lock`.

Una `regex` serve solo per la corrispondenza (è ancorata a entrambi i capi); usala per i nomi che nessun
pattern sa esprimere. I gruppi con nome diventano i campi del branch.

### Quale topic vince

1. Un branch chiamato come un base (uno qualsiasi dei suoi nomi) è quel base.
2. Altrimenti ogni pattern di ogni topic che corrisponde è un candidato: `pattern`,
   poi `patterns`, poi `regex`, topic per topic nell'ordine del file. Vince quello che fissa
   più caratteri letterali (`release/{version}` batte `{user}/{slug}`
   che batte un semplice `{slug}`); una regex conta il suo prefisso letterale. Un pareggio va
   al primo nel file.
3. Altrimenti il branch non è classificato.

Un pattern o una regex rotti vengono segnalati come avviso e saltati; il resto
continua a funzionare.

## Modelli

| Modello | Base | Topic |
|---|---|---|
| `git-flow` | `main` o `master`; `develop` o `dev` (genitore main) | `feature/` e `bugfix/` (da develop), `release/` (da develop, in main e develop, con tag), `hotfix/` (da main, in main e develop, con tag), `support/` (da main); anche le grafie al plurale (`features/`, `releases/`...) corrispondono |
| `github-flow` | `main` | `feature/` o qualsiasi nome, da main, merge request in main, squash, eliminato poi |
| `gitlab-flow` | `main`; `pre-production` e `production` (branch di ambiente, concatenati main, pre-production, production) | `feature/` o qualsiasi nome da main; `release/{version}` come destinazione di cherry-pick |
| `trunk` | `main` | `{user}/{slug}` di breve durata da main, con rebase |
| `custom` | nessuno | nessuno |

I branch di ambiente corrispondono anche come `preprod`, `staging` e `prod`. Un modello è
un punto di partenza: stampalo con `flow config init`, modifica il file.

## Rilevamento e importazione

`gitpoulpi flow detect` mette in classifica i modelli per i branch che il repository ha,
con un punteggio su 100 e i nomi concreti trovati (`master` per `main`, `dev`
per `develop`, `features/` per `feature/`). I base valgono 60 punti, la quota
degli altri branch che i pattern specifici del modello riconoscono 40, e un
branch di lunga durata di un altro modello (un `develop` in un candidato GitHub Flow)
toglie 25.

`gitpoulpi flow import --from git-flow` legge le impostazioni classiche di git-flow
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) dal
`.git/config` del repository; `--from git-town` legge `git-town.main-branch` e
`git-town.perennial-branches` (il genitore di ogni branch,
`git-town-branch.<name>.parent`, viene raccolto come genitore registrato). Nulla viene
applicato in silenzio: il comando stampa il TOML, e `--write` crea
`.gitpoulpi/workflow.toml` solo se non esiste.

## Comandi

Tutti accettano `--json`; il server MCP li espone come strumenti (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` leggono; le azioni sono
descritte qui sotto).

| Comando | Cosa fa |
|---|---|
| `gitpoulpi flow status` | Ogni branch locale e remoto: genere (base, topic, non classificato), tipo di topic e campi, genitore e da dove viene (`recorded`, `git_town`, `type`, `base`), e per i branch locali di quanti commit è `ahead` (avanti) e `behind` (indietro) rispetto al genitore. Inoltre gli avvisi trovati nella definizione, il rilevamento in classifica e le impostazioni di un altro strumento che si potrebbero importare. |
| `gitpoulpi flow templates` | I modelli qui sopra, con la loro definizione completa. Non serve un repository. |
| `gitpoulpi flow detect` | I modelli in classifica con i base e i topic trovati. |
| `gitpoulpi flow config [--show-effective]` | Il file condiviso (il suo percorso, se esiste, il workflow letto, le chiavi sconosciute), oppure con `--show-effective` il workflow con cui vengono classificati i tuoi branch. `flow config show` è lo stesso. |
| `gitpoulpi flow config init --template <t> [--write]` | Stampa il TOML di un modello; `--write` crea `.gitpoulpi/workflow.toml` e si rifiuta di sovrascriverne uno. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Come descritto sopra. |

I conteggi avanti/indietro sono calcolati per al massimo 300 branch locali per chiamata.

Un file di workflow che non si può leggere (TOML errato, un valore del tipo sbagliato) è un errore
che nomina il file; non viene mai ignorato in silenzio.

## Azioni

`start`, `finish`, `sync`, `promote` e `set-parent` agiscono sul repository. Ognuna
prima **pianifica**: elenca ogni passaggio git che farà, con il comando git
a cui equivale, gli avvisi, ciò che non si può annullare e qualsiasi cosa la
blocchi. Nulla cambia finché non confermi. Ogni esecuzione è **una voce nella cronologia
annullamenti** (`gitpoulpi undo` riprende insieme i branch, il tag, il branch
eliminato, il genitore registrato e il checkout).

| Comando | Cosa fa |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Dà il nome al branch in base al pattern del tipo (`flow start feature "Login form"` crea `feature/login-form`; `flow start release version=1.2.0` crea `release/1.2.0`; una parola sola è lo `slug`; `user` e `date` valgono per impostazione predefinita te e oggi), lo crea dal genitore del tipo o da `--from` (un branch, un branch remoto o un tag), ne fa il checkout e registra il suo genitore. `--publish` ne fa il push con `-u` (non annullato da `undo`). |
| `gitpoulpi flow finish [<branch>]` | Per ogni destinazione `finish` del tipo, in ordine: fa il checkout della destinazione e il merge del branch con la strategia della destinazione (`merge`: sempre un merge commit; `squash`: un commit con titolo `--title` o preso dal nome del branch; `rebase`: rebase del branch sulla destinazione, poi fast-forward; `fast-forward`), crea il tag quando la destinazione ne ha uno (`tag = "v{version}"`, annotato), poi fa il checkout del genitore ed elimina il branch locale quando il tipo lo dice (`--delete` / `--keep` lo sovrascrivono; `--delete-remote` lo elimina anche sul remoto, non annullabile). Con `finish_mode = "merge-request"` (o `--mode merge-request`) fa solo il push del branch e apre la pull o merge request verso il genitore, con `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` e `--integration` (non annullabile; in locale nulla viene unito o eliminato). `--mode local-merge` sovrascrive un tipo `merge-request`, e un tipo che dice `ask` richiede uno dei due. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Porta nel branch i commit del genitore: un merge, o un rebase quando la strategia del branch è `rebase` (o `--rebase`). Con `--stack` anche i branch il cui genitore è questo (registrato con `set-parent`), in modo ricorsivo, in ordine. Torna al branch su cui eri. |
| `gitpoulpi flow promote <env-branch>` | Branch di ambiente di GitLab Flow: fa il merge in esso del branch di cui è a valle (`main` in `pre-production`, `pre-production` in `production`), un passaggio, e torna indietro. Fa un fast-forward quando può. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Scrive (o rimuove) `branch.<name>.gitpoulpi-parent`. Rifiuta un ciclo. Annullabile. |

Un branch partito da un'altra parte finisce là: quando il genitore registrato
(da `--from` o `set-parent`) differisce da quello del tipo, la prima destinazione di fine
è il genitore registrato.

### Confermare, pianificare, automatizzare

| Flag | Significato |
|---|---|
| `--dry-run` | Stampa il piano come JSON (`{"plan": ...}`) e si ferma. Codice di uscita 1 quando qualcosa lo blocca. |
| `--yes` | Esegue senza chiedere. |
| nessuno | In un terminale: il piano viene mostrato e ti viene chiesto. Altrimenti (uno script, `--json`, un agente) il piano torna come `{"needs_confirmation": {"plan": ...}}` con codice di uscita 3 e nulla cambiato. `start` senza `--publish` né hook, `set-parent` e `--abort` non richiedono conferma. |
| `--run-hooks` / `--no-hooks` | Vedi [hook](#hook). |
| `--autostash` | Mette in stash le modifiche locali che bloccano un checkout o un merge e le ripristina dopo, qualunque sia il valore di `general.auto_stash`. |

Un piano bloccato ne nomina il motivo: un branch base dove serve un topic, un nome
che esiste già, un campo mancante (`release/{version}` richiede `version=...`),
un tag che esiste, un branch di destinazione indietro rispetto al suo remoto (`--allow-behind`
prosegue comunque), un merge o un rebase già in corso, un workflow non
configurato.

Con l'impostazione di **auto-stash** (`general.auto_stash`), le modifiche in corso che
bloccano un checkout o un merge vengono messe in stash e ripristinate attorno all'azione esattamente come
per un checkout semplice; con `never` l'azione si ferma prima di cambiare qualsiasi cosa.

### Conflitti

Un merge o un rebase che si ferma sui conflitti lascia il normale stato di conflitto (il
banner dei conflitti, `gitpoulpi conflicts ...`) e il flusso viene ricordato. Codice di uscita
2, `status: "conflicted"`, i percorsi in `conflicts`. Risolvi i file e
segnali come risolti, poi:

- `gitpoulpi flow finish --continue` fa il commit del merge (o continua il
  rebase, o fa il commit dello squash con il suo messaggio) ed esegue i passaggi rimanenti;
- `gitpoulpi flow finish --abort` interrompe il merge o il rebase, torna al
  branch da cui il flusso era partito e dimentica il resto. I passaggi già
  completati (la prima destinazione di un doppio finish, con il suo tag) restano: `undo`
  li riprende.

Un flusso fermato da un hook che fallisce riprende allo stesso modo (`--continue` ripete quel
passaggio). L'esecuzione in conflitto e la sua continuazione sono due voci di annullamento.

### Merge request

Il daemon fa il push del branch; aprire la richiesta richiede il tuo token di hosting, quindi
la CLI lo fa con lo stesso codice di `gitpoulpi hosting pr` (un'integrazione
che corrisponde al remoto, altrimenti la pagina web precompilata dell'host, stampata come URL).
Il risultato porta `merge_request` (ciò che è stato chiesto) e `opened` (ciò che l'host
ha risposto), oppure `open_error` quando il branch è stato inviato ma la richiesta non ha potuto essere
aperta.

### Hook

`[hooks]` nel file del workflow elenca comandi di shell (`after_start`,
`before_finish`). Vengono da un file che il tuo team condivide, quindi **non vengono mai eseguiti
se non lo chiedi**: `--run-hooks`, oppure un sì alla domanda che un terminale pone dopo
averli elencati. Il piano li elenca sempre, con `hooks_run: false` quando non
verranno eseguiti. Un hook viene eseguito in una shell con il repository come directory di lavoro e
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` impostati; il suo comando esatto, il codice di uscita
e l'intero output sono nel risultato. Un hook che esce con un codice diverso da zero ferma il
flusso lì (un hook `before_finish` viene eseguito prima che qualsiasi cosa cambi, quindi è un semplice
errore; dopo un hook `after_start` il branch esiste e il flusso si può
continuare o interrompere). `--no-hooks` li salta senza chiedere. Un hook può fare
qualsiasi cosa, quindi `undo` riprende solo le modifiche git.

**Attendibilità.** I comandi di una sezione `[hooks]` di cui è stato fatto il commit
provengono da chiunque possa fare push nel repository, quindi richiedono anche *la tua*
fiducia, una volta per versione dell'elenco. La fiducia è conservata per repository
tramite l'hash degli hook (la modifica di qualsiasi comando, fase o ordine chiede di
nuovo) e viene verificata dal daemon prima del primo passo di un'esecuzione e prima che un
flusso ripreso prosegua, quindi un'esecuzione rifiutata non ha cambiato nulla. Gli hook
del tuo override (`[workflow]` nello stato del tuo repository, la pagina Impostazioni)
sono tuoi e non chiedono mai.

- **CLI:** `gitpoulpi flow trust-hooks` stampa i comandi, il loro `hash` e se sono
  `trusted`; `gitpoulpi flow trust-hooks --hash <hash>` li considera attendibili, e solo
  quando il file su disco ha ancora quell'hash. `--run-hooks` su hook non attendibili
  stampa un rifiuto (`{"refused": {"reason": "hooks_not_trusted", ...}}`, codice di uscita
  2) con i comandi e quella riga di comando. A un terminale a cui viene chiesto «Eseguirli
  anche?» viene chiesto anche se considerarli attendibili.
- **TUI:** l'anteprima elenca gli hook e dice che non sono ancora attendibili. Premere
  allora il pulsante dell'azione chiede «Il workflow di questo repository esegue questi
  comandi sul tuo computer. Considerarli attendibili?» con i comandi esatti in un riquadro
  scorrevole (rotellina del mouse, barra di scorrimento trascinabile,
  PgSu/PgGiù/Home/Fine) e due risposte: **1 Considera attendibili ed esegui** e **2 Esegui
  senza hook**. Nulla è evidenziato finché non premi una freccia o una cifra; Esc, Annulla
  o la croce di chiusura tornano indietro senza rispondere. Quando il file è cambiato da
  quando lo hai considerato attendibile, il titolo dice «Gli hook sono cambiati da quando
  li hai considerati attendibili» e il vecchio elenco è mostrato sotto il nuovo.
- **Agenti:** non esiste uno strumento per considerare attendibili gli hook.
  `run_hooks: true` su hook non attendibili viene rifiutato con i comandi, e una persona
  esegue `flow trust-hooks`.

### Per gli agenti (MCP)

Gli strumenti sono `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` e
`flow_set_parent` (finish, sync e promote sono annotati come distruttivi, quindi
li governa `allow_mutations`). Sono piano-poi-esecuzione: chiama con
`dry_run: true` per ottenere il piano (un normale risultato), poi di nuovo con `yes: true`
per eseguirlo. Senza nessuno dei due la chiamata viene rifiutata e porta il piano, così nulla
parte per errore. Gli hook vengono eseguiti solo con un esplicito `run_hooks: true`; il piano
li elenca (`hooks`, `hooks_run`) così l'agente può mostrare prima i comandi
all'utente. Gli hook di cui è stato fatto il commit devono essere considerati attendibili
da una persona (`flow trust-hooks`, che non ha uno strumento): `run_hooks: true` su hook
non attendibili restituisce `refused` con i comandi e non esegue nulla. Dopo un conflitto,
`continue_: true` (il flag è `--continue`) oppure `abort: true`.

### Annullamento

| Azione | Cosa fa `undo` |
|---|---|
| start | Elimina il branch, dimentica il suo genitore, rifà il checkout del branch precedente. |
| finish | Ripristina ogni branch spostato, il tag, il branch eliminato e il checkout, in un solo passaggio. Un branch inviato con push, un'eliminazione remota e una merge request aperta restano (il piano lo dice). |
| sync | Rimette a posto ogni branch aggiornato, file compresi per quello con il checkout. |
| promote | Rimette a posto il branch di ambiente. |
| set-parent | Ripristina il genitore registrato precedente. |

## Flussi guidati dall'host

Team in cui un'issue avvia il lavoro: l'issue viene scritta da un modello, e l'
host (GitLab, GitHub, Gitea, Forgejo) o un bot crea il branch e la merge
request. GitPoulpi crea l'issue, attende ciò che l'host crea e lo prende in
carico: un branch locale che traccia quello remoto, con il genitore registrato (la destinazione della
merge request), con il checkout fatto. Richiede la sezione ISSUE collegata a un tracker
(`gitpoulpi hosting link issues`); il collegamento di PULL REQUEST dice dove vivono i branch
e le richieste (il progetto di ISSUE quando non è collegata).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # un bot spesso scatta su un'etichetta
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # un file del repository
branch_from_issue = "{type}/{issue}-{slug}"   # come l'host o il bot chiama il branch
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # per quanto `wait` interroga
merge_request = true                 # host-creates-branch: anche una richiesta in bozza
assignees = []
```

`{type}` in `branch_from_issue` è l'id del tipo di topic; il resto sono i soliti token del
[pattern del nome](#pattern-dei-nomi) (`{issue}`, `{slug}`, `{issue_key}`...). Un
branch appartiene all'issue 12 quando corrisponde a quel pattern con `{issue}` uguale a 12
(`feature/12-login` sì, `feature/120-x` no).

| Modalità | Dopo la creazione dell'issue |
|---|---|
| `local` | Nulla sull'host. Con `--pick-up`, un `flow start` del tipo con il numero e il titolo dell'issue (annullabile come qualsiasi start). |
| `host-creates-branch` | GitPoulpi crea il branch **sull'host** dal genitore del tipo (GitLab: l'API dei branch; GitHub: `createLinkedBranch`, così l'issue lo mostra sotto Development; Gitea/Forgejo: l'API dei branch), e una richiesta **in bozza** che dice `Closes #12`. Un host che rifiuta una richiesta vuota (GitHub: ancora nessun commit) lascia il branch e lo dice come avviso. |
| `bot` | Nulla: un bot (un gestore di webhook, un job di CI, una GitHub App) crea il branch e la richiesta. `--wait` interroga finché `wait_for` è soddisfatto. |

Cosa guarda `wait`, per ogni provider: i collegamenti propri dell'host (GitLab
`issues/:iid/related_merge_requests` e `closed_by`; GitHub i branch collegati e le
pull request `connected` / `cross-referenced` della timeline), i branch il cui nome
corrisponde a `branch_from_issue`, e le richieste aperte il cui branch di origine corrisponde. Interroga
con una pausa crescente (1, 2, 3, 5, 8, poi ogni 10 s) finché `wait_for` è soddisfatto o
`timeout_secs` scade, e un host lento o brevemente irraggiungibile dopo una risposta viene
riprovato. Una fonte che l'host non offre (un Gitea non ha collegamenti alle issue) viene saltata; quando
ogni fonte fallisce, la risposta è il primo errore.

| Comando | Cosa fa |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Crea l'issue dal modello (`issue_title`, `issue_labels`, il file di descrizione, `--body` al suo posto) e prosegue come dice la modalità. `--dry-run` stampa cosa verrebbe creato (`{"plan": ...}`) e non crea nulla. `--pick-up` prima attende in modalità `bot`. Codice di uscita 4: l'attesa è finita (l'issue esiste; `flow issue wait` continua ad attendere). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Interroga ciò che l'host ha creato per un'issue, con l'avanzamento su stderr in un terminale; Ctrl+C annulla. `--once` guarda una sola volta. Codice di uscita 4: ancora nulla. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Fa il fetch, crea il branch locale che traccia `origin/<branch>`, registra il suo genitore e ne fa il checkout. `#12` cerca sull'host il branch o la richiesta dell'issue, `!5` una richiesta, qualsiasi altra cosa è il nome di un branch. Il genitore è la destinazione della richiesta (`--parent` la sovrascrive), altrimenti il genitore del tipo. Non richiede `--yes`: è locale e annullabile, come `start`. Un `--dry-run` fa comunque il fetch (sposta solo i ref di tracciamento remoto, e il piano ha bisogno di vedere il branch). |

`pick-up` è una voce nella cronologia annullamenti: `gitpoulpi undo` elimina il branch,
dimentica il genitore e rifà il checkout del branch precedente (il fetch resta, come
ogni fetch). È pianificato come le altre azioni: `--dry-run` stampa il suo piano (`create_branch`,
`set_upstream`, `checkout`, `record_parent`) e valgono i soliti blocchi (le modifiche in
corso seguono l'impostazione di auto-stash). La richiesta è
`{"action": "pick_up", "branch": ..., "parent": ...}`.

Le issue create da un flusso vengono **osservate** (`watched_issues` in `.gitpoulpi/repo-state.toml`,
mai committato): dopo ogni fetch l'app confronta i branch remoti con
`branch_from_issue` e ti dice quando ne compare uno per un'issue osservata. Una presa in carico
la chiude.

### Per gli agenti (guidati dall'host)

Gli strumenti sono `flow_issue_new`, `flow_issue_wait` e `flow_pick_up` (write, network;
li governa `allow_mutations`). `flow_issue_new` con `dry_run: true` restituisce il
piano; `flow_issue_wait` blocca fino a `timeout` secondi, quindi dai a un agente un valore breve
e richiamalo (`once: true` risponde subito). Entrambi restituiscono JSON con `found`
(`branches`, `requests`) e `timed_out`.

### Flussi guidati dall'host nell'app

Con un workflow e un tracker collegato il menu Flow ha *Nuova issue <Tipo>...* (una per
tipo; palette `Flow: Nuova issue <Tipo>...`, e `Flow: Nuova issue...`) e *Prendi in carico un
branch creato dall'host...* (palette `Flow: Prendi in carico un branch dell'host...`).

- **Nuova issue** chiede il tipo (un elenco a discesa) e il titolo; mostra da dove viene la
  descrizione e chi crea il branch. *Prendi in carico il branch quando c'è*
  è attivo per impostazione predefinita. *Crea issue* la crea, poi in modalità `bot` mostra la **card di
  attesa**: un indicatore di attività, ciò che attende, il tempo trascorso e il tempo a cui rinuncia, e
  **Smetti di attendere** (`Esc`). Smettere non fa perdere l'issue: resta osservata. Quando
  il branch c'è viene preso in carico e la barra di stato lo dice.
- **È comparso un branch** è una card che si apre dopo che un fetch ha portato il branch di
  un'issue osservata (quando nessun'altra finestra è aperta): *Prendi in carico* lo prende in carico, *Più tardi* la chiude
  (non viene riproposta finché l'app non si riavvia).
- **Prendi in carico** accetta `#12`, `!5` o il nome di un branch e mostra cosa ha trovato prima di
  prenderlo in carico.

Tutto viene eseguito in background; una chiamata che dura più di 150 ms mostra un
indicatore di attività.

### Provarlo su un banco di prova

`scripts/fake-gitlab-bot.sh` è un bot sostituto per il GitLab in Docker
(`scripts/test-bench.sh`): osserva un progetto per le issue con un'etichetta e crea
`<type>/<iid>-<slug>` (con un commit) e una richiesta in bozza che chiude l'issue.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` serve la stessa
cosa offline (un'API in forma di GitLab su un vero repository bare, con il bot).

## Nell'app

![La barra laterale con un workflow Git Flow: main e develop segnati come branch base, i branch feature in una cartella Feature, il resto sotto Altri](../images/workflow-sidebar.png)

![Avvia Feature: il tipo, lo slug, il punto di partenza e il nome del branch che verrà creato](../images/workflow-start.png)

![Impostazioni > Impostazioni specifiche del repository > Workflow: il modello, il rilevamento e l'editor dei branch base](../images/workflow-settings.png)

Tutto ciò che segue compare solo per un repository che ha un workflow (un
`.gitpoulpi/workflow.toml`, o una tua personalizzazione). Senza, nulla cambia:
la barra laterale è il solito albero e non c'è il pulsante `Flow`.

### Impostazioni > Workflow

*Impostazioni > Impostazioni specifiche del repository > Workflow* modifica la definizione. Lavora su una
bozza e non salva nulla finché non premi un pulsante Salva.

- **Modello** è un elenco a discesa dei modelli pronti; sceglierne uno sostituisce i base e i
  tipi di topic qui sotto con il modello (i tuoi hook e il flusso delle issue restano).
- **Rileva dai branch** legge i nomi dei branch di questo repository e mostra il
  modello più vicino con il suo punteggio, i nomi dei base che ha trovato (`master` per `main`),
  quanti branch riconosce ogni tipo di topic, con esempi, e il punteggio di
  ogni modello. **Usa il workflow rilevato** lo carica nella bozza.
- Quando il repository contiene la configurazione di un altro strumento (i `gitflow.*` di Git Flow,
  i `git-town.*` di Git Town), viene offerto **Importalo**, che carica la
  definizione importata nella bozza. Nulla viene applicato finché non lo premi.
- I **Branch base**, i **Tipi di topic** e le loro **Regole di termine** formano un
  [elenco](settings.md#elenchi) a gruppi nella scheda **Regole**; la scheda **Generali** contiene il resto (modello,
  rilevamento, hook, issue flow). Seleziona una voce per modificarla a destra (nome e
  altri nomi, genitore, protetto, strategia di merge, tag al merge, branch
  di ambiente; etichetta, pattern del nome, altri pattern, espressione regolare, genitore,
  modalità di termine, elimina dopo il termine). **+ Aggiungi ▾** sceglie che cosa
  aggiungere (una regola di termine appartiene al tipo di topic selezionato); aggiungere,
  duplicare, rimuovere e trascinare funzionano come in ogni elenco delle Impostazioni, e le
  voci si spostano dentro il loro gruppo.
  La riga del **pattern del nome** elenca i token (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`); sotto, un'anteprima in tempo reale mostra i branch esistenti che il
  pattern riconosce e il nome che creerebbe, e un pattern che non si analizza
  ne dice il motivo in rosso, subito sotto il campo (nulla di ciò che hai digitato va perso).
- Gli **Hook** sono comandi di shell, uno per riga (`Dopo l'avvio`, `Prima del termine`),
  modificati in un editor a più righe (`Ctrl+S` salva, `Esc` annulla). Eseguono
  comandi sul tuo computer: la pagina lo dice, e le finestre li mostrano prima che
  vengano eseguiti.
- **Issue flow** contiene i campi che usano i [flussi guidati dall'host](#flussi-guidati-dallhost)
  (modalità, titolo ed etichette dell'issue, pattern del branch, cosa attendere), salvati nel
  file del workflow.
- **Problemi** elenca ciò che la definizione sbaglia (un genitore che non
  esiste, un pattern rotto).
- Una barra fissata in fondo alla pagina mostra **● Non salvato** (oppure **Nessuna modifica**).
  **Salva nel file condiviso** scrive la bozza in `.gitpoulpi/workflow.toml` (fanne il commit),
  conservando le chiavi che questa versione non conosce, e integra la tua personalizzazione.
  **Solo per me** conserva solo la differenza rispetto al file condiviso nello stato del tuo
  repository (`.gitpoulpi/repo-state.toml`, non committato). **Scarta le modifiche** rilegge
  ciò che è salvato. Se chiudi le Impostazioni con una bozza non salvata, ti viene chiesto prima.

### La barra laterale

LOCALI e REMOTI sono raggruppati dal workflow: prima i branch base, ciascuno con un
badge `[base]` (`[base, protetto]` quando è protetto), poi una cartella per ogni tipo di topic
con il nome della sua etichetta e il numero di branch (`Feature (3)`), poi i
branch che nessun tipo reclama sotto `Altri`. Le cartelle si comprimono come qualsiasi cartella; gli alberi
semplici tornano con *Flow > Raggruppa i branch per tipo* (la riga del menu Flow) o con il
comando della palette `Flow: Attiva o disattiva il raggruppamento dei branch`. Passando il mouse su un branch si vede il suo tipo,
il suo genitore e di quanto è indietro (e avanti) rispetto al genitore.

### Il menu Flow e le sue finestre

Il pulsante **Flow** della barra degli strumenti (`Ctrl+G Shift+W`, palette `Flow: Apri il menu`) elenca
*Avvia <Tipo>...* per ogni tipo di topic, *Termina il branch corrente...*,
*Sincronizza con il genitore...*, *Promuovi...* (branch di ambiente di GitLab Flow), *Imposta
il genitore...*, l'interruttore del raggruppamento e *Impostazioni del workflow...*. La palette ha gli
stessi comandi (`Flow: Avvia Feature...` viene generato per ogni tipo).

- **Avvia** è un modulo costruito dal pattern del nome del tipo: un campo per token. `{issue}`
  e `{issue_key}` vengono con un elenco a discesa delle issue del tracker collegato, che
  riempie anche `{slug}` dal titolo dell'issue; `{version}` suggerisce la versione dopo il tuo
  tag di release più alto; `{user}` è il tuo profilo attivo; `{date}` è oggi. Una riga
  mostra il nome del branch che il pattern produce (o perché non può: un ref non valido, un nome
  già in uso). *Parti da* è un elenco a discesa (prima il genitore del tipo), e *Pubblica*
  fa il push del nuovo branch.
- **Termina**, **Sincronizza**, **Promuovi** e **Imposta genitore** chiedono ciò che serve, poi
  mostrano l'**anteprima** elaborata dal daemon (lo stesso piano che `gitpoulpi flow ... --dry-run`
  stampa): ogni passaggio in ordine, avvisi (una destinazione indietro rispetto al suo remoto, branch impilati
  su questo), e **blocchi** (una destinazione che non esiste, una destinazione indietro rispetto al suo
  remoto) che disattivano il pulsante di esecuzione e ne dicono il motivo. I passaggi che non si possono riprendere
  (un push, l'eliminazione di un branch remoto, una merge request) sono segnati con `!` e colorati, con
  una riga «L'annullamento non ripristina: ...». Gli hook che verrebbero eseguiti sono elencati sotto un
  avviso, con una casella *Esegui gli hook* (disattivata per impostazione predefinita). Quando la modalità
  di termine del tipo è `ask`, Termina chiede prima *Termina con*: merge in locale, oppure apri una pull o
  merge request.
- L'azione viene poi eseguita con un segno per passaggio (un indicatore di attività dopo 150 ms) e la finestra
  mostra come è andato ogni passaggio. L'intera azione è **una voce di annullamento**: `Ctrl+Z` riprende
  insieme l'avvio, il termine, la sincronizzazione o la promozione (la cronologia annullamenti legge
  `Flow finish feature/login`); push, eliminazioni remote e merge request restano, come diceva
  l'anteprima.
- Un **conflitto** ferma l'azione con **Continua** e **Interrompi**. La finestra si può
  chiudere per risolvere il conflitto con i soliti strumenti per i conflitti, poi riaprire con il
  comando della palette `Flow: Continua o interrompi l'azione interrotta...` (anche una riga del menu
  Flow mentre un'azione è ferma); Continua rifiuta finché ci sono ancora file in conflitto
  e mantiene entrambi i pulsanti. Interrompi torna al branch da cui l'azione è partita; i passaggi
  già fatti restano (l'annullamento li riprende).
- Un termine tramite **merge request** fa il push del branch, poi apre il solito modulo *Crea pull
  request* (*Crea merge request* su GitLab) precompilato: la destinazione è il genitore del branch, il titolo viene dal
  branch (e dall'issue). I pulsanti dei metadati (assegnatari, revisori, etichette, milestone)
  funzionano come in qualsiasi pull request. Quando nessuna integrazione risponde per il remoto, l'URL
  di confronto viene aperto nel browser (e mostrato nella barra di stato).
- I testi dei passaggi e dei blocchi sono in inglese, come quelli della CLI; il testo proprio della finestra è
  tradotto. Una directory di lavoro con modifiche segue l'impostazione `auto-stash`.

Ognuna di queste è raggiungibile dalla tastiera e dalla palette, e ogni
elenco a discesa è una combobox (Enter la apre, digitare filtra, Enter sceglie).

Le voci guidate dall'host (*Nuova issue <Tipo>...*, *Prendi in carico un branch creato dall'host...*) sono
descritte [sopra](#flussi-guidati-dallhost-nellapp).
