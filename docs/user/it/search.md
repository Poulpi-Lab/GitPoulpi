[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · **Italiano** · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Ricerca

`Ctrl+Alt+F` (oppure `Ctrl+P`, poi `Cerca: Area di lavoro...`) apre un cercatore fuzzy
su **ogni repository in cui stai lavorando**: i repository dell'[area di
lavoro](workspaces.md) attiva quando ce n'è una, altrimenti
le schede aperte. Lo apre anche il pulsante **Ricerca globale** (la lente con la sua etichetta nella barra del titolo,
subito a sinistra dell'ingranaggio). Trova:

- **repository** (per nome, poi per percorso),
- **branch** (locali e di tracciamento remoto) e **tag**,
- **worktree** (per cartella, poi per percorso e branch),
- **stash** (per messaggio),
- **commit** (per oggetto; il corpo di un commit è cercato solo quando il suo oggetto
  non corrisponde e hai digitato almeno tre caratteri),
- **pull o merge request** e **issue** (per titolo, `#numero`, autore e branch).

![Il popup di ricerca su due repository aperti: branch, uno stash e un commit che corrispondono a «main», ciascuno con il proprio repository a destra](../images/workspace-search.png)

I risultati sono raggruppati per tipo, con la corrispondenza migliore per prima
all'interno di ogni gruppo. Ogni riga ha l'icona del tipo, i caratteri corrispondenti
evidenziati, un dettaglio attenuato (l'upstream di un branch, l'autore e il branch di
una richiesta, la posizione di uno stash) e il nome del repository a destra, così due
repository con un branch `main` non si confondono mai. Nel titolo compare un indicatore
di attività quando un repository sta ancora indicizzando i suoi commit (la prima
ricerca in un repository grande può richiedere un momento); gli altri tipi ci sono
subito e i commit si aggiungono appena sono pronti.

## Andarci

| Tasto | Fa |
|---|---|
| `Su` / `Giù`, `PgUp` / `PgDn`, `Home` / `End` | Si sposta tra i risultati (le intestazioni dei gruppi vengono saltate). |
| `Enter` | Va al risultato: la scheda del repository si apre (o viene attivata), poi il risultato viene mostrato. |
| `Esc` | Chiude la ricerca. |
| `Alt+1` .. `Alt+8` | Attiva o disattiva un tipo (i chip sopra l'elenco, nel loro ordine). `Alt+0` li attiva tutti. |

Cosa significa «andarci»:

| Risultato | Enter |
|---|---|
| Branch o tag | Lo seleziona nel pannello Refs e il suo commit nel Log. |
| Commit | Lo seleziona nel Log, caricando prima più Log quando è più vecchio di ciò che è caricato. |
| Stash | Seleziona la sua riga nella sezione STASH. |
| Worktree | Lo apre come scheda. |
| Pull/merge request, issue | Seleziona la sua riga nella barra laterale (nella scheda di quel repository). |
| Repository | Passa alla sua scheda (aprendola quando non è aperta). |

`Ctrl+N Backspace` (o `Indietro` nella palette) torna dov'eri, come ogni cambio di
vista che fa un agente per te. Niente viene sottoposto a checkout, unito o modificato
andando da qualche parte.

Con il mouse: un clic su una riga la seleziona, un doppio clic ci va, un clic su un
chip attiva o disattiva quel tipo, la rotellina scorre e un clic fuori chiude il
popup.

## Restringere con un prefisso

Inizia la query con un prefisso per cercare un solo tipo, qualunque cosa dicano i
chip:

| Prefisso | Tipo |
|---|---|
| `r:` | repository |
| `b:` | branch |
| `t:` | tag |
| `w:` | worktree |
| `s:` | stash |
| `c:` | commit |
| `pr:` (o `mr:`, `p:`) | pull o merge request |
| `i:` | issue |
| `#` | pull/merge request e issue (`#12` trova il numero 12) |

`fix: login` è una query normale, non un prefisso: conta solo una singola lettera (o
`pr:` / `mr:`) seguita da due punti, oppure un `#` iniziale.

## Come corrisponde

Gli stessi tre livelli della palette dei comandi: il tuo testo alla lettera, poi alla
lettera in qualsiasi punto, poi i suoi caratteri nell'ordine ma separati. Vincono una
corrispondenza più precoce e un testo più corto. Maiuscole e minuscole sono uguali.

## Pull request e issue

Vengono da ciò che GitPoulpi ha già caricato per ogni repository (gli elenchi dietro le
sezioni PULL REQUEST e ISSUE della barra laterale), conservato in un piccolo file di
cache nella cartella `.gitpoulpi/` del repository. La ricerca **non chiama mai GitHub
o GitLab**: un repository le cui richieste non sono mai state caricate in una finestra
non ne ha da trovare, e ciò che vedi è aggiornato quanto l'ultimo aggiornamento della
barra laterale. Senza un account collegato non c'è nulla da trovare, e nulla da
configurare.

## Dalla riga di comando

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` fa lo stesso e
stampa JSON (vedi il [riferimento della CLI](cli-reference.md#aree-di-lavoro-e-ricerca));
il server MCP ha uno strumento `search`, così un agente può trovare ciò che trovi tu.
`search.commit_limit` in [Impostazioni](config-reference.md#search-la-ricerca-nellarea-di-lavoro)
limita quanti commit di ogni repository vengono indicizzati (`0` disattiva la ricerca
dei commit).

## Cambiare il tasto

`Cerca: Area di lavoro...` è un comando della palette e il tasto è l'azione
`global.workspace_search`: riassegnalo in Impostazioni > Tastiera e mouse, o nella
tabella `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
