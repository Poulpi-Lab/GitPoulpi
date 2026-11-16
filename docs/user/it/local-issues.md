[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · **Italiano** · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Issue locali e Obsidian

Tieni le tue issue come una cartella di note Markdown: nessun account, nessun token, nessuna rete. La cartella può stare dentro il
repository (`docs/issues/`, committata insieme al codice) o in un punto qualsiasi del disco, ad esempio una sottocartella di un
vault [Obsidian](https://obsidian.md). GitPoulpi legge e scrive le note. Obsidian, qualsiasi editor o una shell possono modificarle
nello stesso momento, e gli elenchi seguono.

![Issue Markdown locali nella barra laterale, con il menu della riga di una nota: apri la nota, chiudila, modifica le sue etichette o la sua milestone](../images/local-issues.png)

## Configurarle

1. **Impostazioni > Integrazioni > Issue tracker > Local Markdown > Aggiungi profilo.** Digita la cartella, oppure premi
   **Sfoglia...** e sceglila. Una cartella relativa (`docs/issues`) viene letta rispetto a ogni repository a cui è collegata; una
   assoluta (`/home/me/Vault/Projects/Issues`) è la stessa ovunque. Non c'è nessun token da inserire.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. In un repository, apri la sezione **ISSUE** della barra laterale, scegli il provider **Local Markdown - <nome>** (la cartella
   viene scelta con esso; l'elenco dei repository mostra la cartella e le sue sottocartelle dirette se ne vuoi un'altra).
   - CLI: `gitpoulpi hosting link issues --id <profile>` (la cartella del profilo è il `--project` predefinito).

Nulla viene collegato da solo: come ogni sorgente di issue, si attiva per repository.

## Commenti su una nota

`Enter` sulla riga di una nota nella sezione ISSUE, o sulla sua card, ne apre la conversazione: il testo della nota, poi i
commenti. I commenti sono conservati nel file stesso della nota, in una sezione finale `## Comments`, una voce
`### <autore> - <ora UTC>` ciascuno (una voce modificata aggiunge `(edited <ora>)`). Nulla sopra quell'intestazione viene mai toccato, quindi Obsidian e una shell
leggono e modificano lo stesso file. Puoi scrivere, modificare ed eliminare i commenti dell'autore locale (il `local.me` del profilo,
altrimenti l'utente connesso, il nome che usa *Assegnate a me*); quelli di nessun altro. La casella è il consueto
compositore Markdown con anteprima, e *Scrivi come <nome> (nota locale)* indica l'autore. Nulla passa dalla rete. Una nota con un'intestazione `## Comments` scritta a mano viene letta come tale.

## Le note

Una nota per issue, chiamata `<id>-<slug>.md` (`12-crash-on-empty-name.md`), con un blocco frontmatter:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- Le nuove issue ricevono `max(id) + 1`.
- GitPoulpi scrive solo le proprietà che gli appartengono (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Ogni altra proprietà, il corpo, le interruzioni di riga e l'ordine delle chiavi vengono conservati byte per byte.** Le scritture sono
  atomiche (un file temporaneo, poi una rinomina).
- La lettura è tollerante: una nota senza `id` lo prende dal nome del file, una nota senza `status` è `todo`, una nota
  senza frontmatter prende il titolo dal primo `# titolo`. Un file che non è una issue (`readme.md`) viene ignorato.
- Uno stato è testo libero. **Aperta** significa «non nello stato di completamento».

### Commenti in una nota

Un commento su una nota viene scritto nella nota stessa, in una sezione finale `## Comments` (nessuna rete, e si legge bene in
Obsidian):

```markdown
(il testo della nota)

## Comments

### kai - 2026-11-15T10:00:00Z
Primo commento, **qualsiasi Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
Il secondo.
```

- Una voce inizia a una riga che è esattamente `### <autore> - <ora UTC>`, con ` (edited <ora UTC>)` dopo una modifica;
  il suo testo arriva fino alla voce successiva. La sezione è l'ultima intestazione `## Comments` fuori da un blocco di codice e arriva
  fino alla fine del file, quindi non scrivere nulla dopo.
- Una riga di commento che verrebbe letta come una voce o come l'intestazione viene memorizzata con una barra rovesciata iniziale (`\### ...`, che
  Markdown mostra come lo stesso testo) e riletta senza.
- Pubblicare aggiunge in fondo e lascia tutto ciò che sta sopra byte per byte (fini riga compresi); una modifica o un'eliminazione
  riscrive solo la sezione. Una pubblicazione sposta anche `updated` della nota (a una nota senza frontmatter non ne viene dato uno).
- L'autore è l'autore locale del profilo (il «me» delle impostazioni delle issue locali). Puoi modificare o eliminare solo i tuoi
  commenti; l'id di un commento è la sua ora più l'autore, e due pubblicazioni nello stesso secondo si distinguono spostando
  la seconda avanti di un secondo.
- Per una nota non ci sono risposte, commenti interni né cronologia degli eventi.

## Cosa puoi fare

| Nella sezione ISSUE | Tastiera / palette |
|---|---|
| Nuova issue (il `+`, o `n`): titolo, descrizione, stato, assegnatari, etichette, milestone | `Hosting: Nuova issue...` |
| Modifica assegnatari / etichette / milestone dal menu della riga. Scegli tra ciò che la cartella usa già, oppure **digita una nuova etichetta o un nuovo login** e prendi la riga `Aggiungi «...»` | `Hosting: Modifica etichette...` e simili |
| **Chiudi issue** / **Riapri issue** (imposta lo stato di completamento, o il primo altro) | `Issue locali: Chiudi issue...` / `Riapri issue...` |
| **Apri nota** (il tuo programma predefinito per i `.md`) | `Integrazioni: Apri issue...` |
| **Apri in Obsidian** (solo quando la cartella è dentro un vault, cioè c'è una cartella `.obsidian/` sopra di essa) | `Issue locali: Apri in Obsidian...` |
| Filtri: Aperte / Chiuse / Tutte, «Assegnate a me» | l'elenco a discesa dei filtri |

«Assegnate a me» corrisponde all'impostazione `me` del profilo, altrimenti al nome connesso, altrimenti al nome del profilo.

I branch e i commit funzionano come per GitHub: «Crea un branch dalla issue» dà `12-crash-on-empty-name`, e `#12` in un messaggio di
commit o `12-...` in un nome di branch viene riconosciuto.

Le modifiche fatte fuori da GitPoulpi (Obsidian, una shell, un `git pull`) compaiono entro circa un secondo: i nomi dei file della cartella
e le date di modifica vengono controllati su un thread in background, mai sul thread di rendering.

## File di configurazione

In `config.toml`, accanto al profilo (ogni chiave è facoltativa):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # l'ordine delle colonne della bacheca
done_status = "done"                            # chiude una issue
reference_prefix = "LOC"                         # mostrato come LOC-12 invece di #12
me = "kai"                                       # cosa significa «assegnate a me»
```

Gli stati usati dalle note ma non elencati in `statuses` seguono quelli elencati. `statuses` è l'ordine delle colonne della
[bacheca](#la-bacheca-kanban) (`gitpoulpi hosting issue options` lo stampa).

## La bacheca Kanban

![La bacheca di una cartella di note: una colonna per stato, le card con i loro assegnatari, etichette e milestone](../images/issue-board.png)

Se lavori da solo, o in un team che tiene le issue accanto al codice, la cartella è anche una bacheca Kanban.
**Intestazione ISSUE > Bacheca**, o **Issue: Apri la bacheca** della palette, la apre sopra la finestra; `Esc` la chiude.

- Le **colonne** sono gli stati: prima quelli elencati in `statuses`, in quell'ordine, poi ogni altro stato che una nota usa.
  Una colonna vuota viene comunque mostrata. Ogni intestazione porta il suo conteggio di card.
- Le **card** mostrano il numero della issue, il titolo (due righe), gli assegnatari come iniziali, le etichette e la milestone.
  Quando le colonne non entrano tutte, la bacheca scorre lateralmente (la barra sotto di essa, la rotellina con `Shift`, o le frecce).
- **Sposta una card** trascinandola su un'altra colonna (la colonna si illumina sotto il puntatore), con
  `Shift+Left` / `Shift+Right`, o con **Bacheca: Sposta la card in...** (`m`), un elenco a discesa delle colonne. La card si sposta
  subito; lo `status` della nota viene riscritto come nella [sezione sulle note](#le-note), nient'altro nel file. Se la scrittura
  fallisce la card torna indietro e l'errore è nella barra di stato.
- **Apri** una card con `Enter` o un doppio clic : si apre nella finestra la [conversazione](hosting-integration.md#5c-conversazioni) della nota, con il suo testo e i suoi commenti (**Apri nota** nel menu della card apre il file nel tuo programma predefinito per i `.md`). `e` modifica i suoi
  assegnatari, etichette o milestone, `.` o un clic destro apre il menu della card, `n` avvia una nuova issue nella colonna
  selezionata (il modulo torna alla bacheca quando si chiude).
- **Filtra** per assegnatario (`a`), etichetta (`l`), milestone (`M`) o testo (`/`); `x` li azzera. I filtri sono
  elenchi a discesa: digita per restringere, `Enter` sceglie.
- **Dal vivo**: una nota cambiata da Obsidian o da una shell sposta la sua card entro un secondo.

Ogni gesto del mouse ha un tasto e un comando della palette (**Bacheca: ...**, elencati nella pagina delle
[scorciatoie da tastiera](keybindings.md)).

## Obsidian Bases

**Palette: `Issue locali: Crea Base di Obsidian`** (CLI: `gitpoulpi hosting issue base`) scrive `issues.base` accanto alle
note, una sola volta: una vista *tabella* e una vista *kanban* raggruppata per `status`, nell'ordine di `statuses`. Aprilo in Obsidian per
vedere là le stesse issue come bacheca. GitPoulpi non modifica mai un file `.base` esistente (tuo o di una volta precedente): si
rifiuta e lo dice. Il formato segue la sintassi delle Bases di Obsidian
(<https://obsidian.md/help/bases/syntax>); la vista kanban richiede un Obsidian recente.

## CLI e MCP

Gli stessi comandi di ogni sorgente di issue funzionano su di essa: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Altri quattro sono per le note:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` e `hosting issue base`.
Vedi il [riferimento della CLI](cli-reference.md).

## Limiti

- Una nuova **milestone** non si può digitare nel selettore (scegline una che la cartella usa già), ma `milestone:` in una nota può
  essere qualsiasi cosa e compare appena viene salvata.
- Solo le note direttamente nella cartella sono issue (non le sottocartelle).
- Non c'è nessuna eliminazione: rimuovi la nota nel tuo gestore file o in Obsidian (il suo cestino è tuo).
