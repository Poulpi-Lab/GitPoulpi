[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · **Italiano** · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Più remoti per un branch

Molti repository vivono su più di un host: uno principale (`origin`), uno specchio su un altro host, un backup. GitPoulpi permette a un branch di fare push su tutti con un clic e, se vuoi, di fare pull da più di uno, senza cambiare ciò che fa git semplice. Non serve alcun account e tutto è salvato nel `.git/config` del repository, quindi git semplice e gli altri strumenti continuano a funzionare.

## Destinatari di push e origini di pull

- L'**upstream** è la nozione propria di git (`branch.<name>.remote` e `merge`). È il remoto *primario* del branch. GitPoulpi non cambia mai il suo funzionamento.
- Un **destinatario di push** è un remoto aggiuntivo su cui il branch fa push. Il pulsante Push invia il branch all'upstream e a ogni destinatario.
- Un'**origine di pull** è un remoto aggiuntivo da cui un pull prende anche i commit. È disattivata finché non la attivi (la casella **Pull anche**): unire due remoti che non hai scelto non è un'impostazione predefinita che qualcuno vuole.

Un destinatario può usare un altro nome di branch su quel remoto (`backup` riceve `feat-x`).

## Il dialogo Remoti

Fai clic destro su un branch e scegli **Remoti di <branch>...** (presente anche nel menu a discesa Push, nella palette come `Git: Remoti del branch...`, e con un clic sul badge `⇉3` sulla riga del branch nella sezione Refs). Elenca tutti i remoti del repository:

| Colonna | Significato |
|---|---|
| Remoto, Branch | Il remoto e il nome del branch lì. **Rinomina sul remoto...** cambia quel nome per la riga selezionata. |
| Push | Spuntala per fare del remoto un destinatario di push di questo branch. |
| Pull anche | Spuntala perché un pull integri anche quel remoto. L'upstream viene sempre scaricato. |
| Stato | La posizione del branch rispetto a esso: allineato, avanti, indietro, nuovo, estraneo. |

Una riga segnata «remoto assente» indica un remoto che il branch menziona ancora ma che il repository non ha più; **Rimuovi** lo dimentica. Lo stesso dialogo imposta l'upstream (l'elenco **Upstream**) e **Rimuovi upstream** lo cancella. **Applica** scrive tutto come un'unica modifica annullabile.

Un repository può anche dare a ogni nuovo branch gli stessi destinatari aggiuntivi: **Impostazioni > Git e codifica** (la pagina specifica del repository), «Altri destinatari di push per i nuovi branch» (una casella per remoto). Vengono copiati in un branch quando ottiene per la prima volta un upstream; i branch esistenti non vengono toccati.

## Fare push

- **Pulsante Push** su un branch con destinatari aggiuntivi: fa push su tutti. Quando ogni remoto accetta semplicemente i nuovi commit, parte subito e la vista di avanzamento mostra ogni remoto che diventa un segno di spunta. Se serve una decisione (un remoto ha commit che tu non hai), si apre invece il dialogo del piano.
- **Push ▾** (la freccia accanto al pulsante; anche il pulsante stesso quando **Pulsante Push** è su **chiedi** nelle Impostazioni):
  - `Push su tutti (origin, gitlab, backup)`
  - `Push solo su <remote>`, uno per ogni remoto del branch
  - `Push sui remoti scelti...`: sempre presente, per qualsiasi branch, anche con un solo remoto
  - `Remoti di <branch>...`
- Il menu del clic destro del branch e la palette (`Git: Push su tutti i destinatari`, `Git: Push sui remoti scelti...`) offrono le stesse azioni con gli stessi nomi.

**Push sui remoti scelti...** apre un elenco di tutti i remoti del repository, spuntati con l'upstream e i destinatari del branch. Vale solo per questo push; spunta **Ricorda per questo branch** per salvare la scelta come destinatari di push del branch.

### Il piano

Prima di un push di gruppo GitPoulpi controlla ogni remoto (una richiesta rapida, e un download da quel remoto solo se differisce), così un problema sul terzo remoto emerge prima che parta il primo push. Il dialogo elenca ogni remoto con la posizione del branch e ciò che farà:

| Situazione | Azione predefinita | Offerte anche |
|---|---|---|
| Il remoto non ha nulla del branch, o gli mancano dei commit | Push | Salta |
| Il remoto è avanti (non hai nulla di nuovo per lui) | Fast-forward da esso | Salta |
| Il remoto differisce perché hai fatto rebase o amend di ciò che avevi inviato | Push forzato (lease) | Merge, Rebase, Salta |
| Il remoto ha commit di qualcun altro | Merge da esso (Rebase quando **Pulsante Pull** è su rebase) | Rebase, Push forzato (lease), Salta |
| La cronologia non ha nulla in comune con la tua | Salta | Push forzato (lease) |
| L'host ha rifiutato il branch (protetto) | Salta | nessuna: anche forzare verrebbe rifiutato |

L'azione di ogni riga è un elenco che puoi cambiare (Sinistra/Destra lo scorre, Enter lo apre, `s` salta la riga). Un merge mostra se andrebbe in conflitto e in quali file. Un **push forzato** viene sempre confermato, nomina il remoto e quanti commit sostituisce, usa sempre un lease (mai un force semplice) e non si può annullare: il risultato indica quale commit è stato sostituito e il comando che lo rimette a posto. **Salta i problemi** imposta su Salta ogni riga con problemi. **Esegui** (Ctrl+Enter) avvia.

I merge, i fast-forward e i rebase vanno per primi, nell'ordine dell'elenco, così il tuo branch raggiunge il suo stato finale una volta sola; poi i push partono un remoto alla volta, l'upstream per primo. Non esiste un «tutto o niente» atomico tra i remoti: ognuno finisce con il proprio risultato, e un errore su uno (rete, un hook) non ferma gli altri. **Riprova i falliti** (o `Git: Riprova i push falliti`) li riprova. Chiudere la vista di avanzamento non ferma mai l'esecuzione; `Git: Mostra il push di gruppo` la riporta.

### Conflitti e Continua

Se un merge o un rebase dell'esecuzione si ferma per conflitti, il dialogo si chiude e subentrano il normale banner del merge e l'editor dei conflitti. Una striscia sul Log dice `Push di gruppo in pausa (gitlab): risolvi e concludi, poi Continua.` con i pulsanti **Continua** e **Interrompi gruppo**.

- Risolvi i conflitti e concludi il merge o il rebase (in GitPoulpi o con git semplice).
- **Continua** resta in grigio finché non l'hai fatto e **non viene mai premuto al posto tuo**: nulla prosegue da solo. Premilo (o esegui `Git: Continua il push di gruppo`) e l'esecuzione fa di nuovo push su ogni remoto dalla tua nuova punta. Dopo un rebase, un remoto che ha ancora i vecchi commit torna al dialogo del piano (ora serve un push forzato) invece di essere forzato in silenzio.
- **Interrompi gruppo** (con conferma; `Git: Interrompi il push di gruppo`) scarta il merge o il rebase avviato dall'esecuzione e dimentica l'esecuzione. I push già fatti restano.

La striscia sopravvive a un riavvio di GitPoulpi e la CLI vede la stessa esecuzione salvata.

## Fare pull

- Il pulsante **Pull** scarica l'upstream esattamente come prima, a meno che il branch abbia origini di pull: allora scarica l'upstream e ogni origine, in ordine, ciascuna nella modalità impostata da **Pulsante Pull** nelle Impostazioni (merge, solo fast-forward o rebase). Un'origine che non può essere portata avanti con un fast-forward viene unita (o ribasata) come l'upstream; con solo fast-forward, un'origine che richiede di più chiede prima.
- **Pull ▾** aggiunge `Pull da tutte le origini (origin + gitlab)` quando ci sono origini, e `Pull solo da <remote>` per ogni remoto del branch: un pull una tantum del branch di quel remoto, qualunque cosa dicano le caselle.
- La palette ha `Git: Pull da tutte le origini` e `Git: Pull da un remoto...`.
- Un pull che si ferma per conflitti lascia la stessa striscia, con l'etichetta di un pull di gruppo.

## Impostazioni

Tutte e quattro sono in **Impostazioni > Git e codifica**.

| Impostazione | Predefinito | Significato |
|---|---|---|
| **Pulsante Push** | diretto | diretto fa push subito; chiedi apre prima il menu Push. |
| **Pulsante Pull** | merge | La modalità del pulsante Pull (merge, solo fast-forward o rebase) e delle origini di pull. |
| **Controlla ogni remoto prima di un push** | attivo | Controlla ogni remoto prima di un push di gruppo (la CLI lo segue; `--no-preflight` lo disattiva per una chiamata). |
| **Altri destinatari di push per i nuovi branch** (per repository) | nessuno | I remoti su cui un nuovo branch fa push anche. |

## Dalla riga di comando

```
gitpoulpi branch upstream add gitlab --pull      # un destinatario di push che è anche un'origine di pull
gitpoulpi branch upstream list                    # come è configurato il branch
gitpoulpi push --all-targets                      # upstream più ogni destinatario
gitpoulpi push --to gitlab --to backup:feat-x     # esattamente questi, questa volta
gitpoulpi pull --all-sources                      # upstream più ogni origine di pull
gitpoulpi remote-group plan                       # cosa farebbe un push di gruppo, senza cambiare nulla
gitpoulpi remote-group continue                   # dopo aver risolto e concluso un arresto
```

Quando un remoto richiede una decisione il comando risponde con il piano (uscita 3) e non cambia nulla; rispondi con `--resolve gitlab=merge`. Vedi [il riferimento della CLI](cli-reference.md) per ogni flag. Il server MCP ha gli stessi strumenti.

## Con git semplice

I destinatari sono semplici chiavi di configurazione, quindi uno script o un alias può usarli senza GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # un valore per riga: remote o remote:branch

# push del branch corrente sul suo upstream e su ogni destinatario
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Un valore è `<remote>` (stesso nome di branch) oppure `<remote>:<branch>`, che è ciò che risolve la riga `d=${t#*:}`. L'alias semplice non controlla prima i remoti, non offre un merge e non può continuare dopo un conflitto: per questo esiste il dialogo.

## Da sapere

- Due remoti che puntano allo stesso indirizzo vengono notati («gitlab e mirror fanno push sullo stesso URL»); il secondo viene saltato per impostazione predefinita.
- Un host con un branch protetto rifiuta un push forzato; la riga lo dice e offre solo Salta.
