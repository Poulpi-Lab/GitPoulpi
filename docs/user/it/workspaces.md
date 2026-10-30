[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · **Italiano** · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Aree di lavoro

Un'**area di lavoro** è un gruppo di repository con un nome: quelli su cui lavori
insieme (un front end, la sua API, la documentazione). Ricorda quali di essi sono
aperti come schede e può avere un **profilo di commit predefinito**, così i commit in
quei repository usano l'identità giusta senza che tu cambi profilo a mano. Un
repository può stare in più aree di lavoro, e non stare in nessuna va benissimo:
«Nessuna area di lavoro» è il semplice elenco di schede di sempre.

Le aree di lavoro sono salvate in `workspaces.toml`, accanto a `config.toml`, e sono
condivise da ogni sessione, dalla finestra, dalla CLI e da MCP (`gitpoulpi workspace
...`, vedi il [riferimento della CLI](cli-reference.md)). Un'area di lavoro
aggiunta, rinominata o rimossa da una shell o con `gitpoulpi workspace ...` mentre
la finestra è aperta compare da sola entro circa un secondo (la scheda Aree di lavoro, il chip della barra delle schede e il suo elenco), senza premere alcun
tasto. Chiudere o eliminare un'area di lavoro non tocca mai un repository sul disco. Le modifiche alle aree di
lavoro sono impostazioni, come i preferiti e i profili: non fanno parte di
[Annulla e ripeti](undo-redo.md), ed Elimina chiede prima conferma.

## Il chip dell'area di lavoro

Quando hai almeno un'area di lavoro, sulla barra delle schede compare un chip subito
dopo il `⌂`: un punto colorato e il nome dell'area di lavoro attiva, oppure «Nessuna
area di lavoro» (attenuato). Su una barra stretta si riduce al punto, e su una barra
molto stretta sparisce.

![Il menu a discesa delle aree di lavoro sopra la barra delle schede, con Nessuna area di lavoro, Orbit (spuntata), Side projects, Archive e Gestisci aree di lavoro, con il chip Orbit e il suo profilo «Work - dall'area di lavoro Orbit» nella barra degli strumenti](../images/workspace-switch.png)

- **Cliccalo**, premi `Alt+W`, oppure esegui `Area di lavoro: Cambia...` dalla palette
  per aprire un menu a discesa: le tue aree di lavoro, «Nessuna area di lavoro» e
  «Gestisci aree di lavoro...» (che apre la scheda Aree di lavoro).
- **Cambiare scambia le schede.** Le schede che avevi aperte vengono conservate per
  l'area di lavoro che lasci, e tornano quelle della destinazione (la prima volta, i
  suoi repository che esistono). Torna indietro e tutto è dov'era. La finestra si
  riavvia come in un cambio di scheda; il demone resta.
- **L'area di lavoro viene ricordata**: al prossimo avvio si torna in essa.
- **Chiudere l'ultima scheda** di un'area di lavoro la lascia attiva e mostra la
  scheda Home.
- **Il `+` (nuova scheda) mentre un'area di lavoro è attiva** aggiunge il repository
  che scegli all'elenco di schede e non chiede nulla. Per metterlo nell'area di lavoro
  vera e propria, usa «Aggiungi all'area di lavoro» nel menu del clic destro della
  scheda (o `Area di lavoro: Aggiungi la scheda all'area di lavoro`); «Rimuovi
  dall'area di lavoro» lo toglie di nuovo e lascia la scheda aperta.

Senza ancora un'area di lavoro non c'è il chip: creane prima una dalla scheda Aree di lavoro o
dalla palette.

## La scheda Aree di lavoro

Aprila dal menu del `+`, con `Alt+J` o con `Area di lavoro: Gestisci...` nella palette; la sua `x` (o `Ctrl+W`) la chiude, e resta aperta tra un avvio e l'altro finché non la chiudi. Ogni area di lavoro è una riga
con il suo punto colorato, il nome, quanti repository elenca (e quanti mancano), il
suo profilo predefinito, e «attiva» quando la finestra è in essa. `Destra`, `Space` o
un clic la espande nei suoi repository.

![La scheda Aree di lavoro: Orbit espansa nei suoi due repository, Side projects, e Archive segnata «1 mancanti»](../images/workspaces.png)

| Per area di lavoro | Tasto | |
|---|---|---|
| Apri | `Enter` | La rende attiva e apre le sue schede. |
| Modifica... | `e` | Il modulo qui sotto. |
| Elimina... | `x` | Chiede prima conferma. I repository non vengono toccati. |
| Fetch tutto | `Shift+F` | Un repository dopo l'altro, avanzamento nella barra di stato, errori elencati alla fine. |
| Pull tutto | `Shift+P` | Lo stesso, solo fast-forward: non si ferma mai su un merge. |
| Clona i mancanti | `m` | Clona ogni repository mancante che ha un remoto registrato. |
| Nuova area di lavoro... | `n` | |
| Dalle schede aperte... | `a` | Una nuova area di lavoro con le tue schede aperte spuntate. |

Un repository **mancante** (la sua cartella non c'è più) resta nell'elenco, segnato,
con **Individua...** (`l` o `Enter`: scegli dove si trova ora), **Clona** (quando il
suo URL `origin` è stato registrato al momento dell'aggiunta) e **Rimuovi** (`x`).
Ogni riga ha anche un menu `⋮` (clic destro, `.` o `Shift+F10`) con le stesse azioni,
e ogni azione è nella palette sotto `Area di lavoro:`.

## Il modulo Nuova / Modifica

![Il modulo Modifica area di lavoro: nome, colore, descrizione, profilo predefinito e l'elenco di spunta dei repository](../images/workspace-edit.png)

- **Nome**: obbligatorio, univoco.
- **Colore**: un menu a discesa di colori del tema; il punto sul chip e sulle righe.
- **Descrizione**: testo libero.
- **Profilo**: un menu a discesa dei tuoi `[[profiles]]`, oppure «Nessuno». È
  facoltativo.
- **✦ Istruzioni IA**: solo con l'IA attiva, vedi [sotto](#istruzioni-ia-di-unarea-di-lavoro).
- **Repository**: un elenco di spunta di ogni repository che GitPoulpi conosce (schede
  aperte, preferiti, recenti, quelli dell'area di lavoro stessa, compresi i
  mancanti). `Space` spunta una riga; **Aggiungi cartella...** (`Ctrl+B`) sceglie un
  repository dal disco. L'URL di origine di ogni nuovo repository viene letto dal
  repository e registrato (senza alcuna credenziale): è ciò che Clona usa in seguito.

## Istruzioni IA di un'area di lavoro

Con l'IA attiva (*Impostazioni > IA*), il modulo ha una riga **✦ Istruzioni IA** con
un pulsante **Modifica...** (`Enter` sulla riga). Apre un piccolo editor di ciò che
viene detto all'IA in **ogni repository dell'area di lavoro**: scegli *Ogni azione
IA* o una singola azione (messaggio di commit, spiegazione, messaggio di stash,
pull request, compositore di commit, risoluzione dei conflitti, revisione del
codice) nell'elenco **Per**, scegli **Combina** - *Aggiungi alle globali* (inviate
dopo le istruzioni globali) o *Sostituisci le globali* (inviate al loro posto) - e
scrivi il testo. **Applica** mantiene l'intero insieme, **Salva** nel modulo lo
scrive in `workspaces.toml` insieme all'area di lavoro; **Annulla** scarta ciò che
hai digitato. Con l'IA disattivata la riga è nascosta e le istruzioni dell'area di
lavoro restano come sono.

Le parole dell'area di lavoro stanno tra le istruzioni globali e quelle proprie del
repository (vedi [Istruzioni IA](ai.md#istruzioni) per l'ordine e per cosa fa
*sostituisci*). Un repository che appartiene a più aree di lavoro usa l'area di
lavoro attiva della finestra quando la contiene, altrimenti la prima che lo
contiene. Da uno script: `gitpoulpi workspace set-ai-instructions <id> [--action <a>]
[--mode append|replace] (--text ... | --clear)`; `workspace show` le stampa come
`ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # ogni azione IA in quest'area di lavoro
text = "Use the Acme glossary."
mode = "add"                       # oppure "replace"

[workspace.ai_overrides.code_review]   # una sola azione
text = "Flag missing tests."
mode = "add"
```

## Il profilo predefinito

I commit sono firmati dal primo di questi che si applica:

1. il profilo che hai scelto **a mano** per quel repository (il campo del profilo
   nella barra degli strumenti),
2. il profilo dell'**area di lavoro attiva**, quando il repository è tra i suoi
   membri,
3. `general.active_profile`.

Il campo del profilo nella barra degli strumenti dice da dove viene: «Work -
dall'area di lavoro Acme». Il campo compare anche quando hai un solo profilo, purché
il predefinito di un'area di lavoro sia in vigore. Un'area di lavoro il cui profilo
non esiste più semplicemente smette di applicarlo. Nella CLI, `commit --workspace
<id>` (o `GITPOULPI_WORKSPACE`) si risolve allo stesso modo.

## Tasti

| Tasto | Azione |
|---|---|
| `Alt+W` | Apre il menu a discesa delle aree di lavoro |
| `Alt+J` | Apri la scheda Aree di lavoro |
| `n`, `a` | Nuova area di lavoro, nuova dalle schede aperte |
| `e` | Modifica l'area di lavoro |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch tutto, Pull tutto, Clona i mancanti, Individua |

Tutti possono essere riassegnati in Impostazioni > Tastiera e mouse; l'elenco completo
è in [Scorciatoie da tastiera](keybindings.md).
