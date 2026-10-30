[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · **Italiano** · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Riscrittura della cronologia

Riscrivere la cronologia cambia commit che esistono già: chi li ha scritti e quando,
i loro messaggi, i file che contengono, le loro firme. GitPoulpi lo fa con una
**ricetta** (un ambito e un elenco ordinato di regole) e tre passi, di cui quello
centrale non si salta mai:

1. **Anteprima** calcola la nuova cronologia senza toccare nulla: conteggi, cosa ha
   fatto ogni regola, i branch e i tag che si sposterebbero, una riga per ogni commit
   modificato.
2. **Applica** sposta lì i tuoi branch e tag locali. È una voce della
   [cronologia di annullamento](undo-redo.md): **Annulla** rimette a posto ogni ref
   spostato (e i file in checkout).
3. **Push** (un force push con lease) ed **Elimina** (purge) sono passi separati e
   voluti, perché non si possono annullare: vedi [Inviare il risultato](#inviare-il-risultato)
   ed [Eliminare la vecchia cronologia](#eliminare-la-vecchia-cronologia-purge).

Il pannello, la palette, i comandi `gitpoulpi rewrite` e gli strumenti MCP usano lo
stesso motore, quindi un'anteprima fatta in uno è ciò che applica un altro.
[Dalla riga di comando](#dalla-riga-di-comando) elenca i flag;
[Creare regole con l'IA](#creare-regole-con-lia) riempie il pannello a partire da una descrizione;
[Proposte degli agenti IA](#proposte-degli-agenti-ia) dice cosa può e cosa non può
fare un agente qui.

**Le regole, per ciò che cambiano**

| Gruppo | Regole | Sezione |
|---|---|---|
| Autori | Sostituisci un autore, Rendi anonimi gli autori, Applica .mailmap | [Autori e .mailmap](#autori-e-mailmap) |
| Date | Sposta le date, Cambia fuso orario, Tieni solo in certi orari, Data committer = data autore | [Date](#date) |
| Messaggi | Cerca e sostituisci, Elimina righe, Aggiungi o rimuovi un trailer, Correggi la codifica dei messaggi | [Messaggi](#messaggi) |
| Contenuto dei file e percorsi | Cerca e sostituisci nei file, Rimuovi file, Rimuovi i file grandi, Sposta o rinomina percorsi, Converti i fine riga, Rimuovi il bit eseguibile, Converti la codifica dei file | [File e percorsi](#file-e-percorsi) |
| LFS | Migra i file a LFS, Riporta i file fuori da LFS | [Spostare file dentro o fuori da Git LFS](#spostare-file-dentro-o-fuori-da-git-lfs) |
| Firme | Rimuovi le firme (e nuova firma all'applicazione) | [Firme](#firme) |
| Script | Lua, shell, Python | [La regola Script](#la-regola-script) |

**Prima di iniziare**

- Un'anteprima non cambia nulla. Applica cambia solo i tuoi ref locali e non invia
  mai nulla.
- I commit riscritti ricevono nuovi id. Chiunque abbia la vecchia cronologia va
  avvisato: [Inviare il risultato](#inviare-il-risultato) scrive la nota.
- Un segreto già inviato è compromesso. **Sostituiscilo per prima cosa**, poi
  riscrivi, fai il force push ed elimina la vecchia cronologia.
- I commit che restano invariati mantengono id e firme.

## Il pannello di riscrittura

`Ctrl+G Shift+H` (o **Riscrittura: Pannello della cronologia** nella palette) apre
il pannello di riscrittura sopra il Log. Non ha un pulsante nella barra degli
strumenti: le azioni avanzate stanno nella palette (ogni riga `Riscrittura: ...`
lo apre con quella regola già aggiunta), nel sottomenu **Riscrivi la cronologia**
del menu di un commit (*Da questo commit a HEAD...*, *Commit selezionati...*
quando ne sono selezionati più d'uno, *Apri il pannello di riscrittura...*) e
dietro la scorciatoia.

![Il banco di lavoro per riscrivere la cronologia: la barra dell'ambito, un elenco di due regole Sostituisci un autore con i campi della regola selezionata e il riepilogo dell'anteprima che dice quanti commit cambiano](../images/history-rewrite.png)

`Ctrl+N` mostra un badge su ogni controllo del pannello (ambito, regole, campi, schede, pulsanti); premere un badge fa ciò che farebbe un clic su quel controllo, ed `Esc` toglie i badge (vedi [Badge](keybindings.md#badge)).

- **Barra dell'ambito:** quali ref si spostano (branch corrente, tutti i branch
  locali, branch e tag a scelta, i commit selezionati), la casella *Tag* e, dopo
  *Da questo commit a HEAD...*, una casella per partire dopo il genitore di quel
  commit.
- **Regole:** un elenco numerato, applicato in ordine. La casella attiva o
  disattiva una regola, le frecce la spostano, la croce la elimina (*Ripristina
  la regola eliminata* o `Ctrl+Z` la riporta). *Aggiungi regola* elenca i tipi
  per gruppo. Una regola che richiede un programma che non hai resta in grigio,
  con il motivo.
- **Impostazioni della regola:** un clic su un campo di testo lo modifica; un clic
  su un campo con più valori ne apre l'elenco; le caselle si invertono.
- **Anteprima** (`F5`) calcola la nuova cronologia senza toccare nulla e mostra una
  barra di avanzamento. Le schede: **Riepilogo** (conteggi, cosa ha fatto ogni
  regola, avvisi), **Commit** (una riga per ogni commit modificato, caricata a
  pagine dal daemon; il filtro mostra quelli modificati, rimossi o inviati),
  **Branch e tag** (cosa si sposta, e i branch protetti già inviati, ciascuno con
  una casella), **Impatto** (cosa significa riscrivere una cronologia già
  inviata: un force push con lease, i collaboratori, le firme), **Segreti**,
  **Identità** e **Dimensioni** (vedi sotto).
- **File di un commit:** `Enter` (o doppio clic) su una riga di Commit apre i file
  che le regole sul contenuto hanno modificato in quel commit e, accanto, il diff
  di quello selezionato. La croce, `Esc` o un clic all'esterno lo chiude. Le
  righe rimosse da una regola sui segreti sono nascoste.
- **Applica** (`Ctrl+S` o `Ctrl+Enter`) è disponibile quando l'anteprima
  corrisponde alle regole. Le domande senza un valore predefinito vengono poste
  ogni volta: cosa fare delle firme dei commit firmati e come estrarre una
  cartella. Il pannello del risultato offre **Annulla**, che rimette ogni ref
  com'era.
- **Salva / Carica** conservano una ricetta nel repository (condivisa) o solo per
  te; i valori segreti non vengono mai salvati.

Ogni pulsante ha un tasto: l'elenco è in Impostazioni > Tastiera e mouse
(contesto *Pannello di riscrittura della cronologia*) e nella guida
`keybindings.md`.

### Autori e .mailmap

- **Sostituisci un autore** cerca un autore o un committer per nome e/o e-mail
  (un testo letterale o un'espressione regolare) e imposta un nuovo nome e/o
  una nuova e-mail. *Applica a* sceglie l'autore, il committer o entrambi; *Anche
  gli autori dei tag* fa seguire anche i tag annotati che vengono ricreati. Il
  committer originale resta, a meno che l'esecuzione non dica *io, adesso*.
- **Rendi anonimi gli autori** sostituisce ogni identità con un segnaposto
  numerato (`Contributor 1`, `contributor-1@anonymous.invalid`; entrambi i modelli
  si possono cambiare, con `{n}` per il numero). Nella stessa esecuzione la stessa
  persona riceve sempre lo stesso numero; le identità che indichi come *da
  mantenere* restano come sono.
- **La scheda Identità** (anche **Riscrittura: Unisci le identità (.mailmap)...**
  nella palette) serve a sistemare una persona che ha fatto commit con più nomi ed
  e-mail **senza riscrivere nulla**. Elenca ogni nome ed e-mail dell'ambito nella
  barra dell'ambito con il numero di commit scritti e registrati da ciascuno, e
  quali sono già associati dal `.mailmap` del repository (`associata a Nome
  <e-mail>`). Spunta le identità che sono la stessa persona (`Spazio`, o un clic;
  *Seleziona tutti* / *Deseleziona tutti*), poi **Unisci in...** (`m`): una
  finestra chiede quale di esse resta (in cima quella con più commit) e
  **aggiunge a `.mailmap` una riga per ogni altra identità** (`Nome <e-mail>
  Vecchio Nome <vecchia@e-mail>`; una riga già presente non viene ripetuta, e il
  file viene creato se manca). Non viene fatto alcun commit: includi tu
  `.mailmap` in un commit per condividerlo. Il Log, blame e la cronologia del
  file mostrano subito il nome canonico (impostazione `log.use_mailmap`, attiva di
  base; i commit salvati non cambiano mai, quindi non c'è nulla da inviare con un
  force push). La barra di stato dice cosa è stato scritto e propone **Annulla**
  (`u`), che rimette il `.mailmap` precedente (è una voce della cronologia di
  annullamento, come ogni altra azione). `r` rilegge l'elenco.
- **Un `.mailmap` al posto di una riscrittura.** Quando una regola sugli autori
  modifica commit che sono già su un remoto, il Riepilogo dice *Un .mailmap corregge
  la visualizzazione senza riscrivere nulla. Usare .mailmap?* con un pulsante **Usa
  .mailmap** (`m` nel Riepilogo). Apre la scheda Identità con spuntate le identità
  a cui la regola si applica, e con l'identità che la regola imposterebbe proposta
  come quella da mantenere, anche se nessun commit la porta ancora.
- **La regola `.mailmap`** (*Riscrittura: Applica .mailmap alla cronologia...*, o
  `rewrite preview --mailmap`) riscrive davvero: autori e committer (a tua scelta)
  diventano quelli indicati dal `.mailmap` del worktree, cioè ciò che mostra
  `git log --use-mailmap`. Usala quando è la cronologia stessa a dover portare i
  nomi canonici, ad esempio prima di pubblicarla; è una riscrittura come le altre
  (anteprima, Applica, force push).
- **Regole sulle identità e trailer.** Una regola sulle identità può cambiare anche
  le persone nominate nei trailer: spunta **Trasforma anche i trailer**
  (`--map-trailers`). Ogni trailer il cui valore è esattamente `Nome <e-mail>` e
  corrisponde a *L'e-mail è* / *Il nome è* della regola viene portato al nuovo nome
  e alla nuova e-mail, qualunque cosa la regola faccia ad autore e committer. È
  disattivato di base: un `Signed-off-by` registra chi ha firmato, quindi cambiarlo
  è una decisione a parte.

### Date

Le quattro regole sulle date agiscono sulla data dell'autore, su quella del committer
o su entrambe (*Date* nel riquadro della regola).

- **Sposta le date** sposta ogni data di una quantità (`+2h`, `-1d`, `90m`, `1d 2h`).
  Il fuso orario in cui è stato scritto ogni commit viene mantenuto.
- **Cambia fuso orario** scrive le date con un altro offset UTC. *Mantieni* dice cosa
  resta fermo: l'**istante** (lo stesso momento, l'ora indicata cambia) oppure l'**ora
  indicata** (la stessa ora locale, l'istante cambia).
- **Tieni solo in certi orari** sposta ogni commit che cade fuori dagli orari
  consentiti (ad esempio `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) all'orario
  consentito più vicino, oppure solo prima, o solo dopo (*Sposta verso*). *Mai prima
  del genitore* mantiene l'ordine della cronologia; il *Seed* rende ripetibile la
  piccola dispersione casuale: lo stesso seed sulla stessa cronologia dà le stesse
  date.
- **Data committer = data autore** imposta il committer (nome, e-mail e data) su
  quello dell'autore, che è ciò che un `git commit --amend` o un rebase ha disfatto.

Le date sono regole come le altre: anteprima, controlla la scheda **Commit** (ogni
riga mostra la vecchia e la nuova data), poi applica. L'id di un commit dipende
dalle sue date, quindi una regola sulle date cambia ogni commit dopo il primo che
tocca.

### Messaggi

- **Cerca e sostituisci nei messaggi** ed **Elimina righe dai messaggi** accettano un
  testo letterale o un'espressione regolare (*Espressione regolare*, *Maiuscole e
  minuscole*); una sostituzione vuota elimina. Un messaggio che non è UTF-8 valido
  non viene mai toccato da queste regole: esegui prima **Correggi la codifica dei
  messaggi** se vuoi modificarlo.
- **Trailer.** *Aggiungi o rimuovi un trailer* (*Riscrittura: Aggiungi o rimuovi
  trailer...* nella palette, oppure `--trailer-add`, `--trailer-remove`,
  `--trailer-map`) modifica le righe `Chiave: valore` in fondo a un messaggio, quelle
  che `git interpret-trailers` conosce (`Signed-off-by`, `Co-authored-by`,
  `Reviewed-by`...). L'elenco **Azione** sceglie *Aggiungi* (la chiave **Trailer**,
  il suo **Valore** e **Se esiste già**: *Aggiungi (se non identico)*, *Sostituisci
  quelli esistenti* o *Lascia stare il commit*), *Rimuovi* (ogni trailer di quella
  chiave, oppure solo quelli il cui valore contiene **Il valore contiene**) o *Cambia
  il valore* (**Cerca nel valore** diventa **Sostituisci con**, ad esempio una vecchia
  e-mail con una nuova in `Signed-off-by`). Le chiavi si confrontano senza distinguere
  maiuscole e minuscole. Cambiano solo le righe dei trailer, il resto del messaggio
  resta esattamente com'era, e un messaggio che non ha ancora trailer riceve una
  riga vuota e un blocco tutto suo. La scheda Commit mostra ogni modifica come
  `+Signed-off-by: ...` / `-Co-authored-by: ...`. L'ultimo paragrafo di un messaggio
  è il suo blocco di trailer (mai il titolo), quindi una riga come `Note: see the
  ticket` nel corpo non è un trailer, a meno che non sia da sola nell'ultimo
  paragrafo insieme ad altri trailer.
- **Id dei commit nei messaggi.** **Aggiorna gli id dei commit**
  (`--update-message-hashes`, disattivato di base; in fondo ai riquadri delle regole
  che modificano i messaggi) fa seguire la riscrittura agli id scritti nei messaggi:
  un id abbreviato di 7 o più caratteri esadecimali (`This reverts commit 1a2b3c4`,
  `see 1a2b3c4d5e`) che indica esattamente un commit della cronologia riscritta viene
  sostituito dal nuovo id di quel commit, tagliato alla stessa lunghezza. Uno che
  corrisponde a più commit, a nessuno, o a un commit che la riscrittura crea dopo
  resta com'è; il Riepilogo dice quanti sono stati sostituiti e quanti lasciati.
- **Correggi la codifica dei messaggi** (*Riscrittura: Correggi la codifica dei
  messaggi...*, oppure `--fix-message-encoding [--fix-name-encoding]`) ricodifica i
  messaggi in UTF-8, in due casi:
  - Un commit con un'intestazione `encoding` (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, qualunque codifica che git e le tabelle di codifica conoscano) viene
    decodificato e scritto in UTF-8, e l'intestazione sparisce: il testo si legge
    uguale ovunque, senza conversioni. I nomi di autore e committer di quel commit
    erano nella stessa codifica e vengono decodificati insieme, così un commit non
    resta mai convertito a metà.
  - Il **mojibake**, cioè UTF-8 che era stato letto come Latin-1 o Windows-1252
    (`RÃ©Ã©criture` al posto di `Réécriture`), viene riparato **solo quando la
    conversione di andata e ritorno è esatta**: il testo deve ricodificarsi in byte
    che sono UTF-8 valido, e rovinando il risultato nello stesso modo si deve
    ottenere l'originale. Un messaggio che contiene soltanto `Ã©` accanto a lettere
    vere, o caratteri fuori dai vecchi alfabeti, non supera la prova e resta com'è.

  *Correggi anche i nomi* (`--fix-name-encoding`) ripara allo stesso modo i nomi
  rovinati di autore e committer. Ciò che non si può stabilire (un nome di codifica
  sconosciuto, byte non validi in quella dichiarata, un messaggio che non è UTF-8 e
  non dichiara nulla, un testo che sembra rovinato ma non lo è dimostrabilmente) viene
  **lasciato com'è e conteggiato**: il Riepilogo dice *n messaggi sono stati lasciati
  come sono: la loro codifica non si può stabilire*, e `counts.encoding_left` ha il
  numero. La scheda Commit mostra ogni correzione con il vecchio e il nuovo messaggio
  e una riga *codifica* (`ISO-8859-1` diventa `UTF-8`).

### File e percorsi

- **Regole sul contenuto dei file** (*Cerca e sostituisci nei file*, *Rimuovi file
  dalla cronologia*, *Rimuovi i file grandi*): il pannello controlla ogni valore
  mentre scrivi e mostra il motivo sotto il campo (un pattern che non compila, un
  pattern di percorso errato, una dimensione sotto un byte). Le dimensioni
  accettano le unità (`500 KiB`, `5 MiB`, `1 GiB`). Un pattern di percorso senza
  barra corrisponde a qualsiasi profondità. I *Valori segreti* si scrivono in un
  campo mascherato, valgono solo per l'esecuzione e non vengono mai salvati;
  l'anteprima nasconde le righe che li contenevano. *Commit vuoti* dice se un
  commit rimasto vuoto dopo le regole viene rimosso (predefinito) o mantenuto. I
  file binari e quelli oltre il limite di dimensione restano come sono e vengono
  contati nel Riepilogo.
- **Spostare percorsi.** *Sposta o rinomina percorsi* (*Riscrittura: Sposta o
  rinomina percorsi...* nella palette) ha tre modi, scelti nell'elenco **Sposta**:
  - *Rinomina un percorso* (**Da**, **A**; `--move-path DA=A`): gli stessi file e
    le stesse cartelle compaiono al nuovo percorso in ogni commit che aveva il
    vecchio; nient'altro cambia, ogni file mantiene il proprio id. Un commit in cui il
    nuovo percorso esiste già non viene mai sovrascritto: l'anteprima viene rifiutata
    ed elenca i commit e i percorsi in conflitto (scegli un nome libero, oppure sposta
    prima quello esistente). Solo percorsi esatti, senza caratteri jolly.
  - *Sposta tutto in una cartella* (**Cartella**; `--into-subdir DIR`): ogni percorso
    di ogni commit riceve il prefisso. Utile per unire repository senza conflitti
    di nomi.
  - *Estrai una cartella come radice* (**Cartella**; `--extract DIR`): la cartella
    diventa la radice, i commit che non la toccano vengono scartati (un commit che
    la crea non ha genitore), i merge che non uniscono più nulla vengono semplificati
    e i branch e i tag che non contengono nulla della cartella vengono eliminati. Ti
    viene chiesto **ogni volta** come finire:
    - *In un nuovo clone, aperto in una nuova scheda* (`--extract-mode clone`): una
      copia di questo repository viene creata in una cartella a tua scelta (il nuovo
      repository riceve al suo interno una cartella con il nome di quella estratta),
      riscritta lì e ripulita in modo da contenere solo la nuova cronologia, con ogni
      branch e tag locale che ha la cartella. Questo repository non viene toccato. Il
      nuovo repository non ha remote (l'originale non è il suo upstream).
    - *Sul posto: la cronologia di questo repository viene riscritta* (`--extract-mode
      in-place`): la solita riscrittura, con anteprima, Applica e annullamento. I branch
      senza la cartella vengono eliminati (l'annullamento li riporta); il branch in
      checkout deve averla.
    - *Come sottomodulo di questo repository* (`--extract-mode submodule`): il nuovo
      repository viene creato come nel caso del clone, poi **un nuovo commit** sul
      branch corrente sostituisce la cartella con un sottomodulo (`.gitmodules` e il
      collegamento); la cartella resta come sottomodulo in checkout. La cronologia di
      questo repository **non** viene riscritta: i suoi vecchi commit conservano i file
      della cartella, quindi nulla che li indichi (tag, altri cloni) si rompe. L'URL in
      `.gitmodules` è il percorso locale del nuovo repository: cambialo (`git config
      -f .gitmodules submodule.<cartella>.url`) quando vivrà su un host. File
      modificati o non tracciati nella cartella lo rifiutano. L'annullamento rimette
      la cartella normale.

    Il Riepilogo dice che l'anteprima mostra la cronologia del nuovo repository e in
    quale cartella andrà. Le regole che vengono dopo un'estrazione (una rinomina,
    una rimozione) si applicano all'albero estratto; per un clone o un sottomodulo
    l'estrazione deve essere la prima regola sui percorsi. Le regole sui percorsi
    vengono eseguite dopo quelle sul contenuto, quindi un glob di rimozione indica i
    percorsi com'erano.

#### Fine riga, bit eseguibile e codifiche dei file

Tre regole nel gruppo **Contenuto** del menu Aggiungi (o le righe di palette
*Riscrittura: Converti i fine riga...*, *Rimuovi il bit eseguibile...* e
*Riscrittura: Converti la codifica dei file...*). Tutte e tre accettano **File**
(glob; un pattern senza barra corrisponde a qualsiasi profondità; vuoto significa ogni
file) e **Tranne i file**. Cambiano solo ciò che i glob indicano, in ogni commit
dell'ambito. I file binari (un byte NUL nei primi 8 KiB) non vengono mai toccati: il
Riepilogo li conta come *lasciati come sono*.

- **Converti i fine riga** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Converti in*: LF (Linux, macOS), CRLF (Windows) o CR (Mac
  classico). Ogni fine riga di un file di testo corrispondente diventa quello, quindi un
  file con fine riga misti esce uniforme; i file già corretti mantengono la loro
  versione intatta. *Segui .gitattributes* (disattivato di base,
  `--eol-follow-attributes`) legge il `.gitattributes` nella radice di ogni commit: i
  file marcati `-text` o `binary` vengono saltati e quelli marcati `eol=lf` o `eol=crlf`
  mantengono quel fine riga invece di quello che hai scelto.
- **Rimuovi il bit eseguibile** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). I file corrispondenti passano dalla modalità 100755 alla 100644; il
  loro contenuto non viene toccato. Serve almeno un pattern di file: togliere il bit
  a ogni eseguibile romperebbe i tuoi script.
- **Converti la codifica dei file** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude GLOB...]`).
  *Converti in*: UTF-8 (il predefinito), UTF-8 con BOM, UTF-16 LE o BE (scritti con
  una marca dell'ordine dei byte), Windows-1252 o ISO-8859-1. *I file sono in*: per
  impostazione predefinita la codifica di ogni file viene rilevata (una marca
  dell'ordine dei byte, altrimenti UTF-8 valido, altrimenti quella *di riserva*,
  Windows-1252 se non la cambi); indica la codifica per imporla, ma una scelta
  sbagliata dà testo sbagliato (UTF-8 letto come Latin-1 raddoppia ogni accento). Un
  file che la codifica di origine non sa leggere, o che contiene un carattere che la
  destinazione non può memorizzare (un simbolo dell'euro in ISO-8859-1), resta
  esattamente com'è e viene conteggiato nel Riepilogo.

Le regole vengono eseguite nell'ordine dell'elenco, sullo stesso file: convertire un
file in UTF-16 e poi chiedere i fine riga lo lascia stare (non è più testo semplice).
Applicare è una voce dell'elenco di annullamento come ogni riscrittura.

### Cercare i segreti

- **Cercare i segreti:** `gitpoulpi rewrite scan-secrets` (e lo strumento MCP
  `rewrite_scan_secrets`) legge la cronologia una sola volta ed elenca prefissi di
  token (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic),
  chiavi private, JWT, password negli URL e righe `password = valore` dall'aspetto
  casuale, ciascuno con il suo file, il primo e l'ultimo commit e un'anteprima
  oscurata come `ghp_****abcd`. Non è esaustiva, e il valore non viene mai
  mostrato, registrato né salvato. `--remove-finding ID` trasforma un risultato in
  una regola che lo rimuove da ogni commit. Un segreto già inviato è compromesso:
  sostituiscilo per prima cosa, poi riscrivi, fai il force push ed elimina la
  vecchia cronologia (purge).
- **La scheda Segreti** fa lo stesso dal pannello, per l'ambito della barra
  dell'ambito: **Cerca segreti** (`s`) avvia la ricerca come attività (la barra di
  avanzamento si chiama *Ricerca in corso*; **Annulla** o `Esc` la ferma). Ogni
  risultato è una riga con il suo tipo, l'anteprima oscurata, il file e quanti
  altri lo contengono; sotto l'elenco: il primo e l'ultimo commit in cui è stato
  visto e quanti commit e versioni di file lo contengono. La scheda indica quanti
  file binari e quanti file troppo grandi non sono stati letti, e che la ricerca
  non è esaustiva. All'inizio tutti i risultati sono spuntati; **Rimuovi dalla
  cronologia** (`r`) trasforma quelli spuntati in un'unica regola che li nomina per
  id (mai per valore), da vedere in anteprima e applicare come ogni altra. Dopo una
  riscrittura o un cambio di ambito i risultati diventano obsoleti: la scheda lo
  dice, cerca di nuovo. Il pannello non conosce mai il valore di un segreto, quindi
  non può mostrarlo.

### Trovare cosa pesa

- **Trovare cosa pesa:** `gitpoulpi rewrite analyze [--top N]` (e lo strumento MCP
  `rewrite_analyze`) è l'analisi delle dimensioni, come `filter-repo --analyze`: i
  file più grandi con percorso, dimensione, primo e ultimo commit e se sono ancora
  in HEAD, le cartelle più grandi, i file eliminati da HEAD che la cronologia
  conserva ancora, e i totali. Legge solo dimensioni e percorsi, mai il contenuto
  dei file, e non modifica nulla.
- **La scheda Dimensioni** fa lo stesso dal pannello (`F7`, o *Analizza le
  dimensioni della cronologia* nella palette, la apre e avvia l'analisi):
  **Analizza le dimensioni** (`s`) è un'attività (*Analisi in corso*; **Annulla** o
  `Esc` la ferma). **Mostra** sceglie l'elenco (`v`): *File* (le versioni di file
  uniche più grandi, ognuna marcata *in HEAD*, *vecchia versione* di un file ancora
  presente, o *eliminato*), *Cartelle* (tutto ciò che è mai stato salvato al loro
  interno) o *Eliminati* (percorsi spariti da HEAD che la cronologia conserva).
  **Ordina** (`o`) è per dimensione, per percorso o con gli eliminati per primi.
  Spunta le righe (`Spazio`, `Alt+A` / `Alt+U` per tutte / nessuna): la scheda dice
  quanto contengono, con onestà «fino a circa», perché lo spazio torna disponibile
  solo dopo la riscrittura, l'eliminazione dei vecchi oggetti e un `gc`. **Rimuovi
  dalla cronologia** (`r`) trasforma i percorsi spuntati in una regola sui percorsi
  (caratteri speciali con escape; un percorso già presente in una regola non viene
  aggiunto due volte); **Rimuovi ogni file oltre ...** (`b`, elenco File) scrive una
  regola di dimensione appena sotto il file spuntato più piccolo. Vedili in
  anteprima e applicali come ogni altra regola. Gli elenchi mostrano le 200 righe
  più grandi; la CLI dice quante altre ce ne sono e `--top` le elenca. Dopo una
  riscrittura o un cambio di ambito le righe diventano obsolete: analizza di nuovo.

### Firme

- **Firme:** un commit riscritto non può mantenere la propria firma (firmava i vecchi
  byte), quindi quando vengono riscritti commit firmati ti viene chiesto, ogni volta:
  *Rimuovi le firme* (prima viene mostrato il numero), *Firma di nuovo solo i commit
  che erano firmati* oppure *Firma di nuovo ogni commit riscritto*. Non c'è un
  valore predefinito. I commit che restano invariati mantengono la propria firma. La
  nuova firma usa la tua configurazione, come un commit firmato: `gpg.format`
  (`openpgp`, `ssh` o `x509`), `user.signingkey` e `gpg.program` / `gpg.ssh.program` /
  `gpg.x509.program`. La quarta riga della domanda, **Verifica la firma** (`4`), firma
  una volta un contenuto fittizio e lo verifica: dice quanto dura una firma e stima
  l'esecuzione per il numero di commit di cui si parla, oppure perché non funziona
  (chiave mancante, agente bloccato, programma non installato, con come installarlo).
  L'anteprima fa la stessa prova prima di leggere la cronologia e rifiuta con quel
  motivo, così una configurazione rotta non lascia mai una riscrittura firmata a metà.
  I commit vengono firmati **durante l'anteprima** (i loro id contengono la firma),
  quindi un'anteprima che rifirma richiede il tempo di firma, il Riepilogo dice
  *Commit firmati di nuovo: n, in 12,5 s*, e a quel punto può aprirsi una richiesta di
  passphrase: sblocca prima il tuo agente (Verifica la firma fa esattamente questo).
  **Annulla** si ferma tra due firme e non lascia nulla. Vengono ricreate solo le
  firme dei commit; un tag firmato perde comunque la firma.
- **La regola *Rimuovi le firme*** (menu Aggiungi, gruppo *Firme*; *Riscrittura:
  Rimuovi le firme...* nella palette; `--remove-signatures`) toglie la firma di ogni
  commit dell'ambito, anche dove nient'altro cambia, e non chiede nulla sulla nuova
  firma. Dalla CLI: `rewrite check-signing` e `--signatures
  strip|resign-signed|resign-all`.

### Annullare una riscrittura

**Annulla** (il pulsante del pannello del risultato, *Annulla* della barra degli
strumenti con `Ctrl+Z` nel Log, oppure `gitpoulpi undo`) rimette a posto ogni branch,
tag e nota spostati, e con loro i file in checkout, a partire dai vecchi commit che la
riscrittura tiene agganciati. Chiede prima conferma quando un branch riscritto è già
sul suo upstream (annullare in locale non annulla un push) e quando hai modifiche non
committate: quelle vengono mantenute, e un file che hai modificato e che la vecchia
cronologia sovrascriverebbe ferma invece l'annullamento (i file si spostano come li
sposta `reset --keep`). L'annullamento di una riscrittura smette di essere possibile
quando ne [elimini](#eliminare-la-vecchia-cronologia-purge) la vecchia cronologia;
l'elenco della cronologia la segna allora come *eliminata*.

## Spostare file dentro o fuori da Git LFS

Due regole del gruppo *LFS* del menu Aggiungi riscrivono la cronologia in modo che i
file grandi vivano in [Git LFS](lfs.md) fin dall'inizio (o ne escano). Richiedono il
programma `git-lfs`: senza, le regole sono in grigio con il suggerimento di
installazione e l'anteprima viene rifiutata. Sono un motore nostro, non `git lfs
migrate`, quindi hai l'anteprima, l'ambito, l'annullamento e la protezione del push di
ogni altra regola.

- **Migra i file a LFS** (*Riscrittura: Migra i file a LFS...*, oppure `--lfs-import
  PATTERN` con `--lfs-min-size BYTE`). Ogni file che corrisponde ai pattern (`*.psd`,
  `assets/**`; un pattern senza barra corrisponde a qualsiasi profondità) ed è grande
  almeno quanto *Solo i file di almeno* (vuoto: ogni corrispondenza) diventa un
  puntatore LFS in ogni commit che lo contiene. Il suo contenuto va nell'archivio LFS
  locale (`.git/lfs/objects`, una volta per contenuto distinto) quando applichi, e il
  `.gitattributes` nella radice riceve `<pattern> filter=lfs diff=lfs merge=lfs -text`
  in ogni commit riscritto che contiene un puntatore (creato se non c'è, le altre
  righe intatte, mai duplicato). Un file che è già un puntatore resta com'è. Il
  Riepilogo dice *3 file (12 MiB) passano a LFS. Oggetti LFS da scrivere: 3.* e la
  scheda Commit elenca *sposta big.psd in LFS* per ogni commit.
- **Riporta i file fuori da LFS** (*Riscrittura: Riporta i file fuori da LFS...*,
  oppure `--lfs-export PATTERN`). I puntatori che corrispondono tornano a essere i file
  veri, e le righe di `.gitattributes` per quei pattern vengono tolte. Gli oggetti
  devono essere nell'archivio LFS locale: quando alcuni mancano, l'anteprima viene
  rifiutata con l'elenco e un pulsante **Scarica gli oggetti LFS** che li scarica da
  `origin`; poi genera di nuovo l'anteprima. Un'importazione seguita da
  un'esportazione degli stessi pattern restituisce esattamente gli stessi commit.

Dopo Applica, i file in checkout sono il contenuto vero (non i puntatori) e
**Annulla** rimette la vecchia cronologia. Il pannello del risultato ha **Invia gli
oggetti LFS** (`l`): lo stesso di `LFS: Push di tutti gli oggetti`, così la nuova
cronologia si può inviare senza che un push successivo si blocchi per oggetti mancanti
(anche un normale push dei branch riscritti li carica, tramite l'hook pre-push che
Applica installa). La nota per i collaboratori dice che servono git-lfs e `git lfs
pull`.

Da sapere: Applica installa i filtri LFS del repository (`git lfs install --local`)
quando mancano, perché git ne ha bisogno per confrontare i file veri con i puntatori.
Viene scritto solo il `.gitattributes` nella radice, mai uno annidato. Un file *sotto*
la dimensione minima che corrisponde a un pattern resta un file normale, ma git vorrà
trasformarlo in puntatore al prossimo add, quindi `git status` lo mostra come
modificato finché non lo includi in un commit. L'importazione elabora i file grandi
in streaming, l'esportazione tiene in memoria un file alla volta.

## Inviare il risultato

Applicare una riscrittura non fa mai push. I branch e i tag riscritti che erano su
un remoto richiedono un force push, e il push non forza mai alla cieca: ogni ref
viene inviato con un lease sull'id che il remoto aveva durante l'anteprima (non su
quello che dice ora il tuo branch di tracciamento), così il push di un collega nel
frattempo viene rifiutato per quel ref invece di essere sovrascritto, e i ref
partono in un unico push tutto o niente (`--atomic`) dove il server lo permette. Da
riga di comando:

```
gitpoulpi rewrite push --dry-run        # interroga il remoto, non invia nulla
gitpoulpi rewrite push --yes            # invia
gitpoulpi rewrite note                  # cosa devono eseguire gli altri
gitpoulpi rewrite maps --out ./maps     # commit-map e ref-map
```

I branch protetti (impostazione `rewrite.protected_branches`, predefiniti `main`,
`master`, `develop`, `release/*`) vengono lasciati stare al momento del push, sia
nella riscrittura sia nel push, a meno che tu non li nomini (`--include-protected
main`) o li includa tutti. La protezione dei branch dell'host decide comunque se
accettare un force push; il suo rifiuto viene mostrato per ogni ref.

Nel pannello, il pannello del risultato dopo un'applicazione fa tutto questo:

- **Invia i ref riscritti al remoto** elenca i ref spostati dalla riscrittura che
  un remoto possiede, ciascuno con una casella (`Spazio`). I branch protetti
  partono non spuntati, con accanto *protetto*: spuntali uno a uno, oppure usa
  *Seleziona tutti* / *Deseleziona tutti* (`Alt+A` / `Alt+U`). **Push dei branch
  riscritti** (`p`) invia quelli spuntati; la risposta è una riga per ref
  (*inviato*, *già aggiornato*, *rifiutato: il remoto è cambiato dopo
  l'anteprima*, *rifiutato: protetto dall'host*, *saltato: branch protetto*, *non
  inviato*). Una riga rifiutata ha **Fetch e ricontrolla** (`f`): il pannello si
  chiude e i remoti vengono aggiornati, così il Log mostra cosa hanno inviato gli
  altri. Nulla viene sovrascritto: il lease protegge il loro lavoro, e prima di
  riprovare generi di nuovo l'anteprima.
- **Copia la nota** (`c`) mette negli appunti la nota per i collaboratori, nella
  lingua dell'interfaccia attorno ai comandi git non tradotti.
- **Apri la mappa dei commit** (`m`) mostra dove si trovano la mappa dei commit e
  la mappa dei ref; un clic su un percorso (o `1` / `2`) lo copia.

Un push non si può annullare: se poi fai `undo` della riscrittura, cambia solo il
tuo repository, e l'annullamento lo dice prima di chiedere conferma. Invia la nota
a chiunque abbia la vecchia cronologia: elenca la vecchia e la nuova punta di ogni
branch e tag e i comandi (`git fetch`, poi `git reset --hard origin/<branch>` oppure
`git rebase --onto`). Le due mappe vengono conservate per 24 ore con l'esecuzione;
copiale altrove se la pulizia del repository di GitLab o uno script ne ha bisogno.

## Quando l'host protegge un branch

Il tuo host può proteggere un branch per conto suo (la protezione dei branch e i
ruleset di GitHub, i branch protetti di GitLab, le policy dei branch di Azure DevOps,
le restrizioni dei branch di Bitbucket) e rifiutare il force push che una riscrittura
richiede. Quando il repository è **collegato** al suo host (la sezione delle pull
request della barra laterale, vedi [Integrazione con gli host](hosting-integration.md)),
il pannello chiede all'host quali branch protegge, con il tuo token:

- Quei branch vengono **lasciati fuori dalla riscrittura per impostazione
  predefinita**, esattamente come i branch protetti di `rewrite.protected_branches` e
  del workflow: la scheda Branch e tag li elenca con una casella ciascuno e i
  pulsanti *Seleziona tutti* / *Deseleziona tutti*, e ne dice il motivo: *protetto su
  GitHub*.
- Se il token non riesce a leggere la protezione (manca lo scope, oppure non sei
  amministratore), il pannello dice *Protezione dell'host sconosciuta* e non considera
  nulla come protetto sulla parola dell'host. La riscrittura non viene mai bloccata
  per questo. Un repository non collegato non viene mai interrogato.

Quando un branch protetto è d'intralcio (l'anteprima è stata rifiutata perché ogni
branch è protetto, oppure l'host ha rifiutato il push: *rifiutato: protetto
dall'host*), il pulsante **Come procedere** (`Alt+H`) spiega le tre strade, con la
pagina esatta delle impostazioni del tuo provider e del tuo repository (seleziona un
link e copialo con `c`, oppure aprilo con `o`). GitPoulpi non ne fa mai nessuna al
posto tuo:

1. **Togli temporaneamente la protezione** nelle impostazioni dell'host, fai il push,
   poi rimettila.
2. **Fai il push della riscrittura con un nuovo nome di branch**, poi rendi quel branch
   il branch predefinito (oppure rinomina), ed elimina il vecchio quando nessuno lo usa
   più.
3. **Come ultima risorsa, elimina e ricrea il repository remoto** e invia tutto. Si
   perdono le pull / merge request e le loro revisioni, le issue, il legame dei fork con
   la vecchia cronologia, stelle e osservatori, le impostazioni, i segreti e le variabili
   della CI, e i webhook e le deploy key.

Dalla CLI lo stesso testo torna come `protected_by_host` nel JSON di `rewrite preview` /
`rewrite run` (quando l'host protegge un branch dell'ambito), di un `rewrite push`
rifiutato e di `rewrite note`: `provider`, `branches`, `ways` (ciascuna con i suoi
`steps` e `links`), `recreate_loses` e l'intera spiegazione come `text`.

## Eliminare la vecchia cronologia (purge)

Dopo una riscrittura i vecchi commit sono ancora nel tuo repository: li tiene
l'annullamento della riscrittura (è ciò che lo rende possibile), e li tengono anche
i reflog. Se hai riscritto per liberarti di un segreto, quel segreto è ancora nella
cartella `.git`. L'eliminazione cancella la vecchia cronologia per sempre:

```
gitpoulpi rewrite purge                 # chiede: elenca cosa viene eliminato e perso (codice di uscita 3)
gitpoulpi rewrite purge --yes           # lo fa
```

Rimuove gli ancoraggi dell'annullamento della riscrittura, fa scadere le voci del
reflog che nominano i vecchi commit, esegue `git gc --prune=now` e poi verifica che
ogni vecchio commit e ogni versione di file sia davvero sparita. La conferma dice
prima cosa perdi: **l'annullamento non è più possibile** per la riscrittura (e per
ogni operazione precedente il cui annullamento aveva bisogno degli stessi vecchi
commit); restano nell'elenco della cronologia, marcate come eliminate. Una volta
avviata, l'eliminazione non si può interrompere né annullare. Non tocca alcun
remoto.

Tutto ciò che trattiene ancora la vecchia cronologia blocca l'eliminazione, e in
quel caso non viene modificato nulla. L'elenco nomina ogni elemento con cosa fare:
un branch o un tag che non faceva parte della riscrittura, uno stash sulla vecchia
cronologia (scartalo, oppure tienilo e accetta che la vecchia cronologia resti),
un altro worktree il cui HEAD è su di essa, e i branch di tracciamento remoto che
puntano ancora alla vecchia cronologia (fai il push dei branch riscritti, poi un
fetch). L'elenco degli stash e i reflog degli altri branch non vengono mai toccati.

Un segreto già inviato è compromesso: **sostituiscilo per prima cosa**. Dopo
l'eliminazione, la risposta elenca ciò che serve ancora all'host, per ogni remoto:
GitHub mantiene i vecchi commit raggiungibili nelle viste in cache e nei ref delle
pull request (contatta il supporto GitHub, oppure elimina e ricrea il repository),
GitLab ha una pulizia del repository che prende la `commit-map` (`gitpoulpi rewrite
maps --out DIR`) più la manutenzione, Azure DevOps e Bitbucket richiedono una
richiesta di supporto o un nuovo repository, e chiunque abbia un clone o un fork ha
ancora la vecchia cronologia.

### La schermata di eliminazione

Nel pannello, il pannello del risultato di una riscrittura ha un pulsante **Elimina
la vecchia cronologia...** (tasto `e`), e la palette ha **Riscrittura: Elimina la
vecchia cronologia...** (agisce sull'ultima riscrittura non ancora eliminata, anche
dopo che hai chiuso il pannello). La schermata elenca ciò che l'eliminazione cancella
e ciò che perdi (le operazioni il cui annullamento se ne va con essa; quelle marcate
*perde anche l'annullamento* non sono riscritture ma avevano bisogno dei vecchi
commit), poi ciò che trattiene ancora la vecchia cronologia, una riga ciascuno:

- un branch di tracciamento remoto: **Fetch** li aggiorna dai remoti;
- uno stash: **Scarta** chiede prima conferma, perché uno stash è lavoro che non hai
  mai committato;
- un branch, un tag, un worktree o un altro ref: **Copia il nome**, poi sistemalo
  dove vive (riscrivi o elimina il branch, attiva un branch riscritto nel worktree).

Il controllo viene rieseguito dopo ogni correzione, e **Ricontrolla** (`r`) lo
esegue a richiesta. **Elimina** (`p`) è disponibile solo quando non resta nulla.
Pone un'ultima domanda, numerata e senza risposta predefinita (`1` elimina
definitivamente, `2` lascia tutto com'era), poi parte; la schermata mostra il passo
in corso e dice che non si può interrompere. L'esito dice che la vecchia cronologia
non è più in questo clone, oppure quanti vecchi oggetti ci sono ancora. Sotto,
i passi da fare su ogni remoto (un link si può copiare). La croce, `Esc` e un clic
all'esterno chiudono la schermata, tranne mentre l'eliminazione è in corso.

Via MCP lo strumento di eliminazione non esiste, a meno che tu non imposti
`allow_history_purge = true` nella tabella `[mcp]` di `config.toml`.

## La regola Script

*Aggiungi regola > Script* (o **Riscrittura: Esegui uno script...** nella palette)
aggiunge una regola con uno script Lua e ne apre l'editor sopra il pannello; `Invio`
sulla riga *Script* della regola lo riapre. L'editor parte da un piccolo modello.

- **Linguaggio** (una riga del riquadro della regola): *Lua (integrato)*, *Script di
  shell*, *Python* o *Lua (installato)*. Tutto tranne il Lua integrato è uno
  [script esterno](#script-esterni-shell-python-lua): non isolato, e un linguaggio
  disattivato nelle impostazioni o il cui interprete manca è in grigio nell'elenco,
  con il motivo accanto (sceglierlo dice cosa fare). Cambiare il linguaggio di uno
  script che non hai ancora scritto sostituisce il modello.
- **Script** è un editor di testo con i numeri di riga. Ciò che scrivi è subito la
  regola; non c'è nulla da confermare.
- Sotto, la riga di **controllo** dice `OK: rewrite_commit trovata`, oppure elenca i
  problemi con la loro riga (`script:3: ...`; il numero è in rosso nel margine). Parte
  un istante dopo che smetti di scrivere, mai mentre un tasto è premuto.
- **Parametri** ha una riga per ogni riga `--! param` dell'intestazione dello script,
  con la sua etichetta accanto al campo: testo, un numero, oppure una casella vero/falso.
  Un valore che cancelli torna al predefinito dell'intestazione; un parametro di testo
  senza predefinito va compilato prima dell'anteprima.
- **File** (un percorso dentro il repository, per iniziare
  `.gitpoulpi/rewrite/script.lua`) con **Carica da file...** (sostituisce il testo) e
  **Salva su file...**.
- **Prova sulla selezione** esegue lo script sui commit selezionati nel Log, oppure sui
  20 più recenti se non ce n'è nessuno, e non scrive nulla. La tabella elenca ogni
  commit (id breve), ogni campo che cambierebbe come `vecchio -> nuovo`, e le righe
  `log(...)` dello script. **Annulla** smette di aspettare. Se lo script fallisce,
  vengono mostrati il commit e il motivo.
- La croce, un clic fuori dall'editor e `Esc` lo chiudono; `Tab` e `Maiusc+Tab` si
  spostano tra l'editor, i parametri, il file, i pulsanti e i risultati; `F5` prova.

L'anteprima rifiuta uno script che non compila: il riquadro dell'anteprima dice *uno
script è fallito* con la riga (oppure, per un errore su un commit, il suo id breve e
il motivo). In una build senza script il tipo resta nel menu Aggiungi, in grigio.

## Scrivere uno script con l'IA

Con l'IA attivata ([Assistenza IA](ai.md)) l'editor degli script ha un pulsante
`✦ Scrivi con l'IA...`, e la palette ha `✦ Riscrittura: Scrivi uno script con l'IA...`,
che apre il pannello su una nuova regola Script con la finestra già aperta. Non viene
inviato nulla prima che premi **Invia**: la finestra mostra, mentre scrivi, il testo
esatto che partirà.

- **Cosa deve fare lo script?** Con parole tue: *sposta alla mattina dopo ogni commit
  fatto di notte*, *sostituisci la mia vecchia e-mail con la nuova, solo prima del 2024*.
- **Invia un campione di N commit** (disattivato): se spuntato, vengono aggiunti autore,
  data e oggetto dei commit che «Prova sulla selezione» userebbe (la selezione nel Log,
  altrimenti i 20 più recenti), così il modello vede com'è fatto il tuo dato. Mai il
  contenuto dei file né i corpi dei messaggi. Con la casella disattivata, nessun dato
  dei commit lascia la macchina.
- **Cosa viene inviato** elenca le istruzioni (il contratto fisso dello script, poi le
  tue istruzioni per l'IA) e la tua richiesta, e quale profilo le riceve. `Tab` ci si
  sposta, le frecce lo scorrono.
- **Invia** (`Tab` fino a lì, `Invio`). Mentre il modello lavora la finestra attende, e
  `Esc` annulla la richiesta. Se fallisce, la finestra resta con il motivo, così puoi
  riprovare.

La risposta viene controllata dallo stesso controllo del daemon di uno script scritto
a mano; un errore di compilazione viene rimandato una volta al modello. Lo script
**sostituisce poi il testo nell'editor** (la finestra lo dice quando l'editor contiene
altro che il modello iniziale) e il titolo dell'editor riporta *✦ scritto dall'IA:
leggilo prima dell'anteprima* finché non modifichi il testo. Non viene mai provato,
visto in anteprima né applicato al posto tuo: leggilo, premi **Prova sulla selezione**,
poi genera l'anteprima come per ogni script. Lo script gira nella stessa sandbox di uno
scritto da te (niente file, orologio né rete).

Da riga di comando: `gitpoulpi ai rewrite-script "<istruzione>" [--sample N]` stampa lo
script e il risultato di `rewrite script check` su di esso come JSON, senza eseguire
nulla (codice di uscita 2 se ancora non compila). Viene usato il profilo *Script di
riscrittura della cronologia* di Impostazioni > IA.

## Creare regole con l'IA

Con l'IA attivata ([Assistenza IA](ai.md)) il pannello ha un pulsante `✦ Crea regole con l'IA...`
accanto a Salva e Carica, il tasto `Alt+I` (*Crea regole di riscrittura con l'IA* in Impostazioni >
Tastiera e mouse, contesto *Pannello di riscrittura della cronologia*), e la palette ha
`✦ Riscrittura: Crea regole con l'IA...`. Con l'IA disattivata, o in una build senza IA, il pulsante
e la riga della palette non ci sono. Descrivi con parole tue come deve essere riscritta la
cronologia (*sostituisci la mia vecchia e-mail a@b.org con c@d.org*, *rimuovi secrets.txt da tutta
la cronologia*, *sposta le mie date di due ore*) e il modello propone le regole e le impostazioni,
che **riempiono il pannello**. Non viene inviato nulla prima che premi **Invia**: la finestra
mostra, mentre scrivi, il testo esatto che partirà.

- **Come deve essere riscritta la cronologia?** Un campo su più righe: `Enter` va a capo,
  `Ctrl+Enter` o `Ctrl+S` invia, `Esc` e la crocetta annullano.
- **Invia l'ambito, i nomi dei branch e dei tag e gli autori della cronologia** (disattivato): se
  spuntato, vengono aggiunti l'ambito della barra dell'ambito, i nomi dei branch e dei tag e gli
  autori distinti (nome ed e-mail) dei commit più recenti mostrati dal Log (al massimo 20 commit),
  così il modello può indicare esattamente la tua e-mail (`ai rewrite-rules --context` da riga di
  comando legge invece i 200 commit più recenti). Mai il contenuto dei file né i messaggi
  dei commit. Con la casella disattivata lasciano la macchina solo la tua descrizione e le
  istruzioni fisse.
- **Sostituisci le regole attuali** (solo quando il pannello contiene già delle regole;
  disattivato): per impostazione predefinita le nuove regole vengono aggiunte dopo quelle che hai;
  se spuntato, le sostituiscono (l'ambito e le impostazioni indicati dalla risposta vengono
  applicati in entrambi i casi).
- **Cosa viene inviato** elenca le istruzioni (il vocabolario delle regole, poi le tue istruzioni
  per l'IA), la tua richiesta e il profilo che le riceve. `Tab` ci si sposta, le frecce lo
  scorrono.

Il modello può usare ogni tipo di regola di questa pagina tranne **Script** (è codice: *Scrivere
uno script con l'IA* mostra lo script da rivedere) e la forma a valori segreti di *Cerca e
sostituisci nei file* (i valori segreti non vengono mai consegnati a un modello: usa la scheda
Segreti). Può impostare l'ambito sul branch corrente, su tutti i branch e tag locali, o su branch e
tag esistenti indicati per nome, e le opzioni *Tag*, *committer*, *commit vuoti* e *id abbreviati
nei messaggi*. Ciò che il pannello non sa esprimere (un limite di data come *da marzo*, un
intervallo di commit) non viene indovinato: il modello lo dice in un'osservazione. Cosa succede
alle firme non lo decide mai il modello: ti viene chiesto all'applicazione.

**L'IA prepara soltanto regole e impostazioni.** Non mostra anteprime, non applica e non esegue
mai nulla: la risposta viene copiata nel pannello come una ricetta caricata da te, l'anteprima a
schermo diventa obsoleta, e Anteprima e Applica (con le loro domande e conferme) restano a te,
come sempre. Ogni regola viene analizzata e controllata singolarmente dallo stesso controllo usato
dall'editor e dal motore; una regola che non lo supera viene scartata. Quando qualcosa è stato
scartato, o il modello ha fatto un'osservazione, un elenco lo mostra dopo l'arrivo delle regole
(*Scartate (le regole valide sono state mantenute):*, ciascuna con posizione, nome e motivo) e
`Enter` o `Esc` lo chiude. Leggi le regole, modificale, poi premi Anteprima.

Da riga di comando: `gitpoulpi ai rewrite-rules "<descrizione>" [--context]` stampa la proposta
come JSON (`recipe`, che legge `rewrite preview --recipe-json`, `problems`, `notes` e
`applied: false`) senza eseguire nulla (codice di uscita 2 se qualcosa è stato scartato). Lo
strumento MCP è `ai_rewrite_rules`. Viene usato il profilo *Regole di riscrittura della
cronologia* di Impostazioni > IA.

## Script Lua

Una regola di script esegue la tua funzione su ogni commit. È Lua 5.4 puro in una
sandbox: niente file, orologio, processi, moduli né `pcall`; lo stesso script sulla
stessa cronologia dà sempre lo stesso risultato, e una chiamata che supera 10
milioni di istruzioni, 64 MiB o 5 secondi si interrompe con un errore che indica
il commit.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` contiene `id`, `parents`, `tree`, `author` e `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` e `signed`. `ctx` contiene `index`, `total`,
`params`, `state` (restituisci `state = {...}` per passare dati al commit
successivo) e `window.before` / `window.after` (imposta `--! window: N`).
Restituisci solo ciò che cambia: `author`, `committer`, `message`. Funzioni di
supporto: `regex_replace`, `regex_match`, `stable_hash`, `rand`, `weekday`, `hour`,
`log` e, per i messaggi, `trailers(message)` (un elenco di `{key, value}`),
`set_trailer(message, key, value)` (sostituisce i trailer di quella chiave; un quarto
argomento `"add"` o `"skip"` cambia questo comportamento), `remove_trailer(message,
key)` e `at("2024-03-05 18:30", offset_minutes)` (una data in un offset, come tempo
Unix).

Da riga di comando: `gitpoulpi rewrite script check my.lua` lo compila,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` mostra
cosa cambierebbe senza scrivere nulla, e
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` pianifica
l'intera riscrittura.

## Script esterni (shell, Python, Lua)

Lo stesso contratto di uno script Lua, eseguito da un programma sulla tua macchina:
*Script di shell* (bash o sh; PowerShell su Windows, o qualsiasi `.ps1`), *Python* e
*Lua (installato)* (un `lua` tuo, non quello integrato; non ha una libreria JSON,
quindi il modello legge la riga con un pattern). **Non sono isolati: uno script
esterno viene eseguito con i tuoi permessi** e può leggere o modificare qualsiasi
cosa tu possa. Lo dicono il riquadro della regola e il titolo dell'editor.

Come comunica: GitPoulpi avvia lo script una volta per esecuzione e gli manda una
richiesta JSON per riga sullo standard input (i campi di un commit, `params`, lo
`state` restituito dall'ultima risposta); lo script risponde con una riga JSON sullo
standard output, con gli stessi campi che restituisce uno script Lua (`author`,
`committer`, `message`, `state`, `log`), oppure con `{"error": "testo"}` per
rinunciare. Scrivi la diagnostica sullo standard error: compare come righe `log` del
commit. Le righe dell'intestazione iniziano con `#!` per shell e Python (`#! name: ...`,
`#! param old: text "Old"`; una riga shebang resta una shebang) e con `--!` per Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Cosa riceve.** Un ambiente vuoto tranne `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` e `LANG`; una cartella temporanea nuova come cartella di
lavoro (con una sua copia dello script); non il percorso del repository.
**Limiti di tempo:** `rewrite.script_timeout_ms` (5000) per ogni risposta e
`rewrite.script_total_timeout_s` (3600) per l'intero passaggio. Se vengono superati,
se lo script va in crash, stampa qualcosa che non è JSON o l'attività viene annullata,
GitPoulpi termina lo script e tutto ciò che ha avviato, e la riscrittura si ferma con
il commit e il motivo; nulla si è spostato.

**Quali linguaggi esistono.** Li elenca `rewrite.external_scripts` in `config.toml`:
`["shell"]` di base; aggiungi tu `"python"` e `"lua"` in `config.toml` (`[rewrite]`,
`external_scripts = ["shell", "python"]`; un elenco non si può impostare con
`gitpoulpi config set`). Un linguaggio non elencato è in grigio nel pannello e
rifiutato dalla CLI e da MCP (`scripts_disabled`); uno elencato il cui interprete non
si trova è in grigio con il suggerimento di installazione (`tool_missing`).

**Attendibilità.** Anche quando un linguaggio è attivo, uno script viene eseguito solo
dopo che hai ritenuto attendibile il suo testo esatto. Uno script che scrivi nel pannello
è tuo e parte subito; uno che arriva con una ricetta condivisa o da un file del
repository no: l'anteprima e *Prova sulla selezione* si fermano e chiedono *Considerare
attendibile questo script?* con due risposte numerate e nessuna preselezionata (`1`
considera attendibile e continua, `2` no, `Esc` lo lascia). Considerarlo attendibile
ricorda lo SHA-256 del testo per questo repository, quindi uno script modificato
chiede di nuovo, e un repository clonato non può eseguire codice su di te da solo. Da
riga di comando, leggi il file, poi `gitpoulpi rewrite script trust FILE` (oppure
`--trust` su `preview`, `run`, `script check` e `script test`, che lo considera
attendibile e prosegue). Un agente via MCP non può considerare attendibile nulla:
può eseguire solo uno script che avevi già ritenuto attendibile.

**Controllo.** La riga di controllo sotto l'editor dice *Non ancora considerato
attendibile* per uno script non attendibile (non è stato avviato nulla); per uno
attendibile GitPoulpi lo avvia una volta con un commit inventato (`"dry_run": true`) e
verifica che la risposta rispetti il contratto. `gitpoulpi rewrite script check FILE`
fa lo stesso e riporta `trust`, `hash` e `interpreter`.

## Dalla riga di comando

`gitpoulpi rewrite` (e gli strumenti MCP con gli stessi nomi) esegue lo stesso motore,
per script e agenti. Una **ricetta** è un file TOML o JSON, quello che salva il
pannello (`--recipe FILE`, oppure `-` per lo standard input), e i flag aggiungono
regole dopo quelle della ricetta.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # piano, non cambia nulla
gitpoulpi rewrite apply --run ID --digest D           # esegue il piano che hai controllato
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # rimette tutto a posto
```

- Regole come flag: `--set-name` / `--set-email` con `--match-email` (identità),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. L'ambito è il branch corrente, `--all-local`, oppure
  `--ref R` (ripetibile), a partire da `--since REV`.
- **Le domande senza valore predefinito** sono flag: `--signatures
  strip|resign-signed|resign-all` quando cambiano commit firmati, e
  `--extract-mode` per un'estrazione. Senza di essi la risposta è un rifiuto o un
  `needs_choice` (codice di uscita 3), mai un'ipotesi.
- **Codici di uscita:** 0 fatto; 1 un errore (ricetta illeggibile, flag errati); 2
  rifiutato, con il motivo (`stale`, `signature_choice_needed`,
  `dirty_paths_touched`, `empty_scope`, ...); 3 una domanda a cui rispondere, oppure
  ref già inviati che richiedono `--yes`.
- `preview` conserva il piano nel repository, quindi `apply` funziona da un comando
  successivo, dopo un riavvio del daemon o da un altro frontend; un piano che non
  corrisponde più al suo digest è `stale`.

Il [riferimento della CLI](cli-reference.md) ha ogni flag, e
[MCP e agenti](mcp-and-agents.md) dice quali strumenti riceve un agente (lo
strumento di eliminazione non esiste a meno che tu non lo attivi, e un agente non può
considerare attendibile uno script).

## Proposte degli agenti IA

Un agente può **suggerire** una riscrittura, mai eseguirla. `gitpoulpi ui propose
rewrite --recipe FILE|-` (lo strumento MCP `ui_propose_rewrite`, disattivato finché
`[mcp] allow_ui_propose = true`) apre il pannello nella tua finestra in
esecuzione con le regole della ricetta già nell'elenco e un banner viola, *✦
Proposta da <agente>: una riscrittura della cronologia (3 regole) · Ctrl+Enter
accetta · Esc rifiuta*.

- **Non viene eseguito nulla.** Né anteprima, né applicazione, né push, né
  eliminazione: la proposta riempie soltanto le regole. Le leggi, le modifichi se
  vuoi, generi l'**Anteprima** e applichi con il tuo tasto. Chiudere il pannello
  rifiuta la proposta.
- **Cosa l'agente non può metterci:** uno script (verrebbe eseguito all'anteprima),
  più di 50 regole, valori segreti (vengono tolti dalla ricetta prima dell'invio).
  Una proposta così viene rifiutata con il motivo.
- L'agente può leggere l'esito in `ui state` (`accepted` quando hai applicato le
  regole come proposte, `accepted_edited` quando le hai prima modificate,
  `rejected` quando hai chiuso). Una proposta alla volta; viene rifiutata mentre è
  aperto un altro tuo dialogo.

## Strumenti

Quasi tutta una riscrittura della cronologia non richiede altro che Git:
cambiare autori e committer, applicare un `.mailmap`, rendere anonimi gli autori,
spostare le date o limitarle a certi orari, riscrivere messaggi e trailer,
sostituire contenuti, rimuovere percorsi o file grandi e spostare percorsi
avvengono tutti dentro GitPoulpi.

**Gli script Lua sono integrati.** L'interprete è incorporato in GitPoulpi, quindi
uno script Lua non richiede alcuno strumento né alcuna configurazione. Gli script
di shell, Python e Lua installato hanno bisogno del loro interprete (vedi la tabella
sotto) e sono descritti in [Script esterni](#script-esterni-shell-python-lua).

Alcune funzioni usano un programma esterno. GitPoulpi lo cerca (il programma viene
trovato nel `PATH`, su Windows anche tramite `PATHEXT` e la cartella `usr/bin` di
Git for Windows, e deve rispondere a `--version` entro 2 secondi); la risposta
resta in cache per un minuto e «Ricontrolla» la chiede di nuovo. Una regola che
richiede un programma mancante resta nell'elenco, in grigio, con il motivo e come
installarlo, e l'anteprima rifiuta prima che parta qualsiasi cosa.

| Funzione | Strumento | Note |
|---|---|---|
| Rifirmare i commit | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) o `gpgsm` (`gpg.format = x509`) | La firma SSH e X.509 richiede anche `user.signingkey`. `gpg.program`, `gpg.ssh.program` e `gpg.x509.program` sono rispettati. |
| Migrazione a LFS (le due regole LFS) | `git-lfs` 3.0 o successivo | Lo stesso controllo del pannello LFS. |
| Script Python | `python` 3.8 o successivo (`py -3` su Windows) | Disattivati di base: aggiungi `"python"` a `rewrite.external_scripts`. |
| Script Lua (installato) | `lua5.4`, `lua54`, `lua` o `luajit` | Disattivati di base: aggiungi `"lua"` a `rewrite.external_scripts`. |
| Script di shell | `bash`, `pwsh` (PowerShell) o `sh` | Attivi di base (`"shell"`). Uno script PowerShell (un file `.ps1`, uno shebang `pwsh`, o qualsiasi script di shell su Windows) richiede `pwsh` o `powershell`; uno POSIX richiede `bash` o `sh` (su Windows li fornisce Git for Windows). |

Come installarli, prima Windows:

- Windows: Git for Windows include `gpg`, `ssh-keygen`, `bash`, `sh` e Git LFS.
  Altrimenti `winget install GnuPG.GnuPG`, Impostazioni > Funzionalità
  facoltative > Client OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: i pacchetti della distribuzione (`gnupg`, `openssh`, `git-lfs`,
  `python3`, `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` e `sh` sono
  già inclusi.

