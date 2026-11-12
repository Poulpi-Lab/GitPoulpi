[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · **Italiano** · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Scegliere un terminale

GitPoulpi funziona in qualsiasi terminale con supporto del mouse, ma dà il meglio in
uno che disegna immagini, mostra il true color e invia la tastiera completa. Questa
pagina dice che cosa offre ciascun terminale su ciascun sistema operativo e quale
impostazione cambiare.

**Come leggerla.** Le tabelle vengono da una sessione di prove di **novembre 2026**: ogni
terminale è stato pilotato con sequenze di escape grezze e, dove poteva eseguirlo, con
GitPoulpi stesso, su Linux, Windows 11 e macOS 15. Le versioni sono elencate
[più sotto](#versioni-provate). Ogni cella è una di queste:

- **funziona**;
- **parziale** (e che cosa manca);
- **non funziona** (e perché: il *terminale*, il *livello del sistema operativo* o *GitPoulpi*);
- **n/d** (il terminale non esiste su quel sistema operativo);
- **non misurato** (e perché).

I multiplexer (tmux, zellij, screen, herdr, psmux, riquadri) hanno una pagina propria:
[Multiplexer](multiplexers.md).

## Le scelte migliori

| SO | Scelte migliori, in ordine | Perché |
|---|---|---|
| **Windows** | **Windows Terminal**, poi **WezTerm** e **Rio** | Windows Terminal disegna il grafo e gli avatar come immagini sixel, ancorate bene, ed è il predefinito del launcher. WezTerm e Rio li disegnano come immagini iTerm2, ancorate bene dalla correzione ConPTY di novembre 2026 (vedi [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty e Ghostty offrono immagini, forme del puntatore e tastiera completa. foot disegna immagini sixel e ha la tastiera completa e le forme del puntatore. |
| **macOS** | **iTerm2**, poi **kitty** e **Ghostty** | iTerm2 3.7.7 disegna le immagini tramite il protocollo kitty. Terminal.app funziona ma non ha né immagini né true color. kitty e Ghostty si comportano come su Linux (non misurato su macOS). |

Su qualsiasi sistema, **screen** e **zellij** nascondono a GitPoulpi il protocollo di
immagini del terminale, quindi disegna caratteri; tmux lascia passare sixel. Vedi
[Multiplexer](multiplexers.md).

## Che cosa usa GitPoulpi

- **Immagini** (colonna del grafo, avatar degli autori, diff di immagini e Vista file): il
  **protocollo grafico di kitty** con segnaposto Unicode, le **immagini inline di iTerm2**
  (`OSC 1337`) o **sixel**. Il rilevamento è negato per impostazione predefinita, e parte da
  `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` e dalle variabili dei multiplexer. Quando
  l'ambiente non nomina nessuno (uno xterm avviato con `-ti vt340`, un terminale di VS Code
  con le immagini attive, qualsiasi terminale raggiunto via SSH, dove `TERM_PROGRAM` non viene
  inoltrato), decidono le risposte del terminale stesso: GitPoulpi chiede una volta all'avvio
  (vedi «La query di avvio» più sotto) e usa il nome XTVERSION del terminale (kitty,
  Ghostty, WezTerm, iTerm2, mintty, foot) oppure sixel nei suoi attributi di dispositivo.
  Dentro un multiplexer le risposte vengono ignorate, perché tmux dichiara sixel qualunque
  cosa ci sia dietro.
- **Quale protocollo dove.** kitty, Ghostty e iTerm2 3.7.7 o successivo: il protocollo
  kitty. WezTerm e Rio: il protocollo iTerm2, fondendo la trasparenza del PNG. mintty e
  iTerm2 precedente alla 3.7.7: il protocollo iTerm2 con uno sfondo opaco. Windows Terminal,
  foot, Tabby, Konsole e mlterm: **sixel**. Tutto il resto: caratteri, e l'avatar diventa
  il badge con le iniziali. `ui.image_protocol` ne forza uno.
- **iTerm2 3.7.7.** Ogni immagine inline `OSC 1337` apre una finestra modale «Allow
  Terminal-Initiated Display?» che blocca la finestra finché non spunti
  «Remember my choice». Le stesse immagini inviate tramite il protocollo kitty, segnaposto
  Unicode compresi, vengono disegnate senza domande, quindi lì GitPoulpi usa il protocollo
  kitty. `ui.image_protocol = "iterm2"` dà comunque le immagini inline.
- **Sixel** è usato per la **colonna del grafo** (corsie, nodi e, con
  `ui.avatars = "graph"`, gli avatar nei cerchi dei commit) e per gli **avatar** altrove
  (Log, dettaglio del commit, Cronologia del file, Blame, vista immagine). Un sixel non ha
  alfa, id né ridimensionamento, quindi l'immagine misura esattamente `colonne x larghezza
  della cella` per `righe x altezza della cella` pixel, è disegnata solo dove c'è qualcosa,
  sopra celle vuote che mantengono lo sfondo; viene ricodificata solo quando cambiano le corsie,
  lo scorrimento o lo zoom. Un sixel non si può cancellare: GitPoulpi scrive del testo sulle
  sue celle per rimuoverlo. Il sixel richiede la vera dimensione della cella in pixel: Windows
  Terminal, Tabby e mintty rispondono a `CSI 16 t` una volta all'avvio e dopo uno zoom
  (mai bloccante se tacciono); altrove viene dalla dimensione della finestra del terminale,
  o da `GITPOULPI_TERMINAL_CELL_PX=10x20`. Un terminale che disegna
  sixel senza dirlo richiede `ui.image_protocol = "sixel"`. Windows Terminal
  prima della 1.22 non disegna sixel: lì imposta `ui.image_protocol = "none"`.
- **True color**: i temi sono RGB. `ui.color_depth = "auto"` scende a 256 colori con un
  `TERM` di tipo `*-256color` senza `COLORTERM=truecolor` (e in Terminal.app, che non ha
  true color), e a 16 colori sulla console Linux.
- **Mouse**: segnalazione mouse SGR con pulsanti, trascinamento e movimento (feedback al
  passaggio, drag and drop, clic destro, doppio clic).
- **La query di avvio.** Quattro domande partono in una sola scrittura: i flag della tastiera
  kitty (`CSI ? u`), il colore di sfondo (`OSC 11`, usato solo per `ui.theme = system`), il
  nome del terminale (XTVERSION, `CSI > 0 q`) e i suoi attributi di dispositivo (`CSI c`, per ultima:
  ogni terminale risponde, quindi chiude la lettura). La lettura è un solo ciclo `poll` con una
  scadenza di 100 ms, terminato prima che parta il lettore della tastiera. Un terminale che non
  risponde costa i 100 ms una volta e tutto resta come dice l'ambiente. Solo Unix: su
  Windows la console segnala da sola i tasti e decide l'ambiente. Le risposte sono nel
  rapporto sull'ambiente del Registro attività (riga `start-up query:`).
- **Tastiera**: il **protocollo tastiera di kitty** (solo il flag disambiguate) viene richiesto
  all'avvio. Fa arrivare `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` e `Ctrl+Shift+Z`.
  Senza di esso restano i tasti tradizionali (`Ctrl+Y` ripete, le varianti con `Alt`
  sostituiscono alcune combinazioni). Vedi
  [Scorciatoie da tastiera](keybindings.md#tasti-che-il-terminale-non-può-inviare-e-il-protocollo-tastiera-kitty).
- **Appunti**: ogni azione di copia segue un'unica regola. In locale, gli **appunti di sistema**
  della macchina su cui gira GitPoulpi, più `OSC 52` al terminale quando il terminale è
  noto per rispettarlo (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` è anche il ripiego quando gli appunti di sistema falliscono. **Via
  SSH, o senza display** (una console Linux, un container), viene inviato solo `OSC 52`, e
  dentro tmux viene inviato sia avvolto per il passthrough sia in chiaro. La riga di stato dice
  «copiato» solo quando gli appunti di sistema hanno preso il testo; ciò che è andato al terminale dice
  «inviato ... agli appunti del terminale (OSC 52, non confermato)», perché nulla conferma
  che il terminale abbia obbedito. Il rapporto sull'ambiente del Registro attività indica il percorso nella
  riga `clipboard:`.
- **Forma del puntatore del mouse** (puntatore di ridimensionamento sui divisori, mano sugli
  elementi cliccabili, I-beam sulle caselle di testo): `OSC 22`, un extra dove il terminale lo
  implementa. GitPoulpi lo invia sempre (tranne sulla console Linux e dietro screen, zellij e herdr, che lo
  scartano o lo stampano; dietro tmux viene avvolto per il passthrough). Con `TERM_PROGRAM=mintty`
  invia i nomi di cursore propri di mintty (`hand`, `ibeam`, `sizewe`, ...), gli unici che
  mintty legge. **Rio non implementa `OSC 22` su nessun sistema operativo**: il suo sorgente analizza la
  sequenza e la scarta, quindi nessuna impostazione da una parte o dall'altra cambia le cose.
- **Cursore di testo**: con `ui.text_cursor = "terminal"` (il predefinito) il cursore del terminale stesso viene
  messo sul punto di inserimento del campo di testo attivo, come barra lampeggiante (`DECSCUSR`), e nascosto
  altrove; all'uscita torna il tuo stile di cursore. `drawn` disegna invece il cursore di GitPoulpi.
- **Eventi di focus** abilitati (i tooltip si chiudono quando la finestra perde il focus).
  Il **bracketed paste** viene richiesto all'avvio e rilasciato a ogni uscita (Unix), così un
  incolla su più righe arriva come un solo incolla: i suoi a capo non confermano mai un campo e le sue
  lettere non attivano mai scorciatoie. Su Windows la console non ha parentesi di incolla; vedi
  [Windows](#windows) per ciò che GitPoulpi fa al suo posto.
  GitPoulpi non richiede collegamenti OSC 8 né undercurl.
  Solo sul percorso delle immagini, ogni fotogramma è avvolto in un aggiornamento
  sincronizzato (modalità 2026, ignorata dai terminali che non la conoscono) così un'immagine non
  lampeggia mai sopra un popup.
- **Emoji e simboli**: GitPoulpi usa solo glifi con presentazione emoji predefinita
  o forme BMP a larghezza singola, perché terminali e font non concordano sui selettori di
  variazione.

## La matrice

Le righe sono i terminali, le colonne i sistemi operativi. «GitPoulpi» in una cella significa che GitPoulpi stesso è
stato eseguito lì; «sonda» significa che sono state eseguite solo sequenze di escape grezze. Celle **n/d**: kitty non ha
una build nativa per Windows (le release hanno Linux e macOS); Ghostty non ha una build per Windows
(solo la sua libreria libghostty è multipiattaforma); foot è un terminale Wayland (solo Linux);
xterm è un client X11 (Linux, e macOS tramite XQuartz); iTerm2 e Terminal.app sono solo
macOS; Windows Terminal, conhost e mintty sono solo Windows (mintty anche sotto Cygwin e
MSYS2).

**Non misurati su macOS: kitty, Ghostty, WezTerm, Rio e Warp.** Richiedono una GPU
(Metal o OpenGL 3.3). I loro risultati su Linux sono il riferimento per ciò che fa lo stesso programma;
riprovali su un Mac con GPU prima di fidarti di una cella macOS.

### Immagini: quale protocollo sceglie GitPoulpi, e se si vede

| Terminale | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funziona**: protocollo kitty (GitPoulpi) | n/d | non misurato (richiede una GPU) |
| **Ghostty** | **funziona**: protocollo kitty (GitPoulpi) | n/d | non misurato (richiede una GPU) |
| **WezTerm** | **funziona**: protocollo iTerm2 con trasparenza (GitPoulpi). I segnaposto vengono disegnati schiacciati, quindi il protocollo kitty non è usato | **funziona**: protocollo iTerm2 con trasparenza, grafo e avatar sulle loro righe (ufficiale 20240203 e la build della PR a monte #8092). L'ufficiale 20240203 metteva le immagini sulle righe sbagliate prima che GitPoulpi mostrasse il cursore attorno a esse, vedi «Il limite di ConPTY». | non misurato (richiede una GPU) |
| **foot** | **funziona**: sixel con trasparenza (GitPoulpi) | n/d | n/d |
| **Rio** | **funziona**: protocollo iTerm2 con trasparenza (GitPoulpi) | **funziona**: protocollo iTerm2 con trasparenza, esatto alla cella a ogni dimensione di font provata (0.5.28; GitPoulpi). Sixel e kitty non arrivano mai (ConPTY li scarta), quindi non c'è nulla da scegliere | non misurato (richiede una GPU) |
| **Alacritty** | testo: **funziona** (il terminale non supporta le immagini) | testo: **parziale** (nessuna immagine; diff di immagini a blocchi) | testo: **funziona** |
| **Tabby** | **funziona**: sixel con trasparenza (sonda; GitPoulpi non eseguito in Tabby su Linux) | **funziona**: sixel, allineato (GitPoulpi) | **funziona**: sixel, trasparenza mantenuta (GitPoulpi) |
| **Warp** | non misurato (richiede XWayland e decodifica male i tasti sintetici, quindi non si è potuto digitare nulla) | **non funziona** (terminale: nessuna immagine sixel, iTerm2 o kitty); GitPoulpi non misurato (non si è potuto avviare dalla casella di input di Warp) | non misurato (richiede una GPU) |
| **xterm** | sonda: sixel **funziona** con `-ti vt340`; GitPoulpi lo sceglie dagli attributi di dispositivo (la query di avvio è venuta dopo la prova), oppure imposta `ui.image_protocol = "sixel"` | n/d | testo: **funziona**; sixel viene disegnato (sonda) e richiede `ui.image_protocol = "sixel"` |
| **iTerm2** | n/d | n/d | **funziona**: protocollo kitty dalla 3.7.7, senza finestra (GitPoulpi). Le versioni precedenti ricevono le immagini inline con uno sfondo opaco. Sixel viene disegnato ma dipinge opachi i pixel non impostati, quindi `ui.image_protocol = "sixel"` dà un rettangolo pieno |
| **Terminal.app** | n/d | n/d | testo: **funziona** (nessuna immagine) |
| **Windows Terminal** | n/d | **funziona**: sixel (GitPoulpi, 1.25) | n/d |
| **conhost** | n/d | testo: **parziale** (grafo a caratteri, diff di immagini a semiblocchi) | n/d |
| **mintty** | n/d | **non funziona** (terminale dietro ConPTY: nessuna immagine viene disegnata, nemmeno una semplice `OSC 1337` posizionata con il cursore visibile; la colonna del grafo resta vuota; testo e palette vanno bene) | n/d |

### Forma del puntatore del mouse (OSC 22)

| Terminale | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funziona** | n/d | non misurato (richiede una GPU) |
| **Ghostty** | **funziona** | n/d | non misurato (richiede una GPU) |
| **WezTerm** | **non funziona** (terminale: non implementato nella build provata) | ufficiale: **non funziona** (terminale). Build con la PR di WezTerm a monte #8092, che aggiunge le forme del puntatore: **funziona** (mano, I-beam, frecce di ridimensionamento) | non misurato (richiede una GPU) |
| **foot** | **funziona** | n/d | n/d |
| **Rio** | **non funziona** (terminale: analizzato e scartato) | **non funziona** (terminale, stesso motivo; non ConPTY) | **non funziona** (terminale, stesso sorgente; provato solo su Linux e Windows) |
| **Alacritty** | **non funziona** (terminale) | **non funziona** (terminale) | **non funziona** (terminale) |
| **Tabby** | **non funziona** (terminale: xterm.js) | **non funziona** (terminale) | **non funziona** (terminale) |
| **Warp** | non misurato | **non funziona** (terminale) | non misurato |
| **xterm** | **non funziona** (terminale: nomi CSS sconosciuti) | n/d | non misurato (il puntatore era fuori dalla finestra) |
| **iTerm2** | n/d | n/d | **non funziona** (terminale) |
| **Terminal.app** | n/d | n/d | **non funziona** (terminale) |
| **Windows Terminal** | n/d | **non funziona** (terminale) | n/d |
| **conhost** | n/d | **non funziona** (terminale) | n/d |
| **mintty** | n/d | **funziona** con i nomi propri di mintty; GitPoulpi li invia | n/d |

Dove la cella dice «non funziona», il divisore o il pulsante sotto il mouse si illumina comunque.

### Protocollo tastiera

GitPoulpi chiede il protocollo tastiera di kitty su Linux e macOS. Su Windows non lo
chiede: la console segnala da sola i modificatori, e le combinazioni qui sotto sono limitate da ciò che il
terminale tiene per sé.

| Terminale | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funziona** | n/d | non misurato (richiede una GPU) |
| **Ghostty** | **funziona** | n/d | non misurato (richiede una GPU) |
| **WezTerm** | **parziale**: nessuna risposta finché `enable_kitty_keyboard = true` in `wezterm.lua` (predefinito `false`) | **parziale**: WezTerm tiene `Ctrl+Shift+N` (apre la sua finestra); `Ctrl+,` non viene consegnato (build ufficiale). La build della PR #8092 non tiene nessuna delle due combinazioni, ma `Ctrl+,` continua a non aprire le Impostazioni (nessuna risposta tastiera kitty attraverso ConPTY) | non misurato (richiede una GPU) |
| **foot** | **funziona** | n/d | n/d |
| **Rio** | **funziona** (attivo di base) | **parziale**: Rio tiene `Ctrl+Shift+N` (apre la sua finestra) | non misurato (richiede una GPU) |
| **Alacritty** | **funziona** (0.13 e successive) | **parziale**: `Ctrl+Shift+N` arriva come `Ctrl+N`, `Ctrl+,` non viene consegnato | **funziona** |
| **Tabby** | **non funziona** (terminale: nessuna risposta; tiene per sé `Ctrl+Shift+N` e `Ctrl+,`) | **parziale**: Tabby tiene `Ctrl+Shift+N` e `Ctrl+,` | **non funziona** (terminale: nessuna risposta); alcune combinazioni non sono arrivate |
| **Warp** | non misurato | non misurato | non misurato (richiede una GPU) |
| **xterm** | **non funziona** (terminale) | n/d | **non funziona** (terminale) |
| **iTerm2** | n/d | n/d | **funziona** (3.7.7; se no, controlla *Profiles > Keys*) |
| **Terminal.app** | n/d | n/d | **non funziona** (terminale: nessuna risposta; tasti tradizionali) |
| **Windows Terminal** | n/d | **parziale**: WT tiene `Ctrl+Shift+N` (nuova finestra) e `Ctrl+,` (le sue impostazioni) | n/d |
| **conhost** | n/d | non misurato | n/d |
| **mintty** | n/d | **parziale**: nessuna delle due combinazioni ha avuto un effetto visibile | n/d |

Dove una combinazione non arriva, usa le varianti con `Alt`
([Scorciatoie da tastiera](keybindings.md#tasti-che-il-terminale-non-può-inviare-e-il-protocollo-tastiera-kitty)),
oppure riassegna la combinazione in *Impostazioni > Tastiera e mouse*.

### Appunti (azioni di copia)

Una copia usa sempre gli appunti di sistema quando c'è un display locale, quindi queste celle
descrivono il percorso `OSC 52` del terminale, che conta via SSH, nei container e come
ripiego. Ogni prova di copia di GitPoulpi che usava gli appunti di sistema ha funzionato.

| Terminale | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funziona** | n/d | non misurato (richiede una GPU) |
| **Ghostty** | **funziona** (visto dietro tmux, zellij, herdr); da solo: non misurato (una scrittura negli appunti richiede il focus della tastiera, che la prova non gli ha dato) | n/d | non misurato (richiede una GPU) |
| **WezTerm** | **funziona** | **funziona** | non misurato (richiede una GPU) |
| **foot** | non misurato (una scrittura richiede il focus della tastiera, che la prova non gli ha dato) | n/d | n/d |
| **Rio** | non misurato (stesso motivo) | **funziona** | non misurato (richiede una GPU) |
| **Alacritty** | non misurato (stesso motivo) | **non funziona** (appunti invariati con la configurazione predefinita; non indagato) | **funziona** |
| **Tabby** | **funziona** | **funziona** | **funziona** |
| **Warp** | non misurato | **non funziona** (appunti invariati) | non misurato (richiede una GPU) |
| **xterm** | non misurato | n/d | **non funziona** (appunti invariati) |
| **iTerm2** | n/d | n/d | **non funziona** finché *Settings > General > Selection > Applications in terminal may access clipboard* non è attivo |
| **Terminal.app** | n/d | n/d | **non funziona** (terminale) |
| **Windows Terminal** | n/d | **funziona** (con il focus) | n/d |
| **conhost** | n/d | non misurato (nessuna sonda eseguita) | n/d |
| **mintty** | n/d | **non funziona** (appunti invariati con le impostazioni predefinite) | n/d |

### Incolla

Su Linux e macOS GitPoulpi richiede il bracketed paste, quindi un incolla su più righe è un solo incolla.
Non è stato provato terminale per terminale (**non misurato** per terminale). Su Windows la console
non ha parentesi di incolla, quindi un terminale che non le chiede consegna il testo come
pressioni di tasti e ogni a capo sarebbe un `Enter`. GitPoulpi quindi tratta come un solo
incolla le pressioni di tasti che si susseguono entro 30 ms (un `Enter` interno, o otto o più caratteri). I tasti tenuti premuti e
«abc, `Enter`» digitato a mano non sono incolla.

| Terminale | Windows (incolla di due righe nel prompt del nuovo branch) |
|---|---|
| **Windows Terminal** | **funziona**: WT chiede prima «incollare più righe?» (il suo avviso), poi non viene creato alcun branch |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **funziona** (nessun branch creato) |
| **Warp** | non misurato (non si è potuto avviare GitPoulpi al suo interno) |
| **conhost** | non misurato (`Ctrl+V` e `Shift+Insert` inviati dalla prova non hanno incollato nella console classica) |

### Larghezza di emoji e icone

| Terminale | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **funziona** (celle larghe allineate; kitty disegna vuoto l'ingranaggio U+2699, una questione nota di font) | WezTerm, Rio, Tabby, Windows Terminal: **funziona**. Alacritty: **parziale** (emoji come contorni monocromatici) | Alacritty, Tabby: **funziona**. Altri: non misurato (richiedono una GPU) |
| **Tabby** | **funziona** | **funziona** | **funziona** |
| **Warp** | non misurato | **funziona** | non misurato |
| **xterm** | **parziale** (riquadri: il font) | n/d | **parziale** (riquadri: il font) |
| **iTerm2**, **Terminal.app** | n/d | n/d | **funziona** |
| **conhost** | n/d | **parziale**: emoji a colori disegnate, celle larghe allineate | n/d |
| **mintty** | n/d | **funziona** | n/d |

### Versioni provate

kitty 0.49.2 (macOS 0.50.0 non è partito), Ghostty 1.4.0, WezTerm 20260716 su Linux e
20240203 (l'ultima release taggata) su Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) e
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) e 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. I terminali cambiano in fretta: una versione più recente di queste può comportarsi diversamente.

## Terminale per terminale

### Windows

- **Windows Terminal.** Il predefinito del launcher e il miglior supporto alle immagini su Windows:
  grafo e avatar sixel, OSC 52, buone emoji, e risponde alla query della dimensione della cella. Scegli un
  Nerd Font in *Settings > Profiles > Defaults > Appearance > Font face*. Tieni premuto `Shift` per
  selezionare il testo mentre il mouse è catturato. Tiene `Ctrl+Shift+N` e `Ctrl+,`: usa le
  varianti con `Alt` o riassegna. La forma del puntatore non cambia.
- **WezTerm.** In `wezterm.lua`, imposta `config.enable_kitty_keyboard = true` (build Linux e
  macOS) e un font con emoji di ripiego, per esempio
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Le immagini usano il protocollo iTerm2 e non serve impostare nulla: l'ultima release taggata,
  20240203-110809 (feb 2024), le ancora sulle righe giuste (ricontrollato a novembre 2026 con
  l'attuale GitPoulpi; prima della correzione ConPTY descritta più sotto non lo faceva). La PR di WezTerm a monte #8092 aggiunge le forme del puntatore.
- **Rio** (MSI, richiede UAC). Le immagini (grafo, avatar, diff di immagini) usano il protocollo iTerm2
  con trasparenza e cadono esattamente sulle loro celle: misurato sulla 0.5.28 con il font predefinito
  e a 18 e 22,5 px (celle 8x16, 11x21 e 13x26 pixel, sempre pixel interi), un'immagine di 20x10 celle e
  una di 4x27 celle sono venute entrambe esatte alla cella. Non serve impostare nulla. Lo pseudo-terminale
  inghiotte la query della dimensione della cella, quindi GitPoulpi presume che le celle di Rio siano alte il doppio
  della larghezza (vero a tutte e tre le dimensioni); con un font di altra proporzione imposta
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (per esempio `10x24`) per avere marcatori di commit rotondi. Rio
  non ha forme del puntatore su nessun sistema operativo.
- **Tabby.** Grafo e avatar sixel, copia e incolla funzionano. Tabby tiene per sé `Ctrl+Shift+N` e
  `Ctrl+,`.
- **Alacritty.** Richiede OpenGL 3.3. Grafo a caratteri; la copia tramite `OSC 52` non era attiva con la configurazione predefinita.
- **mintty** (Git Bash, MSYS2, Cygwin). Le forme del puntatore funzionano (nomi propri di mintty). Le immagini
  non si vedono (la colonna del grafo resta vuota; mintty 3.8.3 non disegna nessuna immagine `OSC 1337` dietro
  ConPTY nemmeno quando è posizionata da una semplice sonda) e lì non risponde a nessuna query sulla dimensione della cella: imposta
  `ui.graph_renderer = "text"`.
- **Warp.** Non misurato con GitPoulpi: non si è potuto avviare dalla casella di input di Warp. Il
  terminale non ha immagini sixel, iTerm2 o kitty.
- **conhost, cmd, una finestra PowerShell.** Grafo a caratteri, diff di immagini a semiblocchi. Per impostazione predefinita
  GitPoulpi avviato da una finestra così si riapre nel terminale che hai scelto; vedi
  «La regola del rilancio».

**Il limite di ConPTY.** Windows esegue il programma di ogni terminale dietro uno pseudo-terminale
(ConPTY). Le build di Windows Terminal e WezTerm che ne includono uno recente lasciano passare le sequenze
delle immagini; quello dentro Windows (usato da Rio e mintty), e quello incluso
in WezTerm 20240203, no. Ciò che si perde: le sequenze DCS (sixel) e APC (grafica kitty)
vengono scartate, quindi arrivano solo le immagini iTerm2 (`OSC 1337`), ed è per questo che Rio e
WezTerm le usano; `CSI 16 t` (dimensione della cella) riceve risposta solo in Windows Terminal e Tabby.
Ciò che è stato corretto: un `OSC 1337` attraversa ConPTY subito e cade sul cursore che ConPTY ha
dipinto per ultimo, e ConPTY dipinge il cursore solo mentre è visibile. GitPoulpi tiene il cursore
nascosto, quindi le sue immagini cadevano su righe vecchie (il grafo sopra il piè di pagina, uno schermo strappato dopo
lo scorrimento). Ora su Windows mostra il cursore attorno a ogni immagine che scrive, dentro la stessa
scrittura, e lo nasconde di nuovo; Rio e WezTerm 20240203 ancorano ogni immagine alla sua cella
(`pty_conpty_picture_anchor` rilegge il movimento del cursore dietro ConPTY). Nulla cambia
fuori da Windows o per sixel.

**Incolla su Windows.** Vedi la tabella «Incolla» sopra: GitPoulpi riunisce una raffica di pressioni di tasti
in un solo incolla, quindi un incolla su più righe non conferma mai un prompt nei terminali provati.

**La regola del rilancio.** Un doppio clic sull'exe lo avvia in una finestra di console classica
(conhost) tutta sua. Quando `ui.launch_terminal` nomina un altro terminale (predefinito: Windows
Terminal), GitPoulpi si riapre lì e la prima finestra si chiude
(`ui.relaunch_in_terminal`). Lo fa **solo** quando è solo in una finestra di console
classica. Un terminale che esegue direttamente l'exe (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`) è dove vuoi stare, e nulla viene rilanciato; da una shell non è solo,
quindi non succede nulla nemmeno lì. Disattivalo con `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Tutto funziona senza impostazioni. `Shift+clic` è di kitty stesso, quindi usa
  `Alt+clic` per estendere una selezione ([Mouse](mouse-and-gestures.md)).
- **Ghostty.** Tutto funziona senza impostazioni. Se le icone appaiono come buchi, aggiungi un font emoji
  o Nerd Font a `font-family` (ripeti la chiave per aggiungere ripieghi).
- **WezTerm.** Immagini tramite il protocollo iTerm2, trasparenza fusa. Imposta
  `enable_kitty_keyboard = true` per le combinazioni complete. Nessuna forma del puntatore.
- **foot.** Grafo e avatar sixel, forme del puntatore, tastiera completa, veloce. Imposta
  `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`. La copia tramite `OSC 52` è regolata da
  `security.osc52`.
- **Rio.** Immagini tramite il protocollo iTerm2 con trasparenza, tastiera completa. Nessuna
  forma del puntatore (Rio scarta `OSC 22`).
- **Alacritty.** Grafo a caratteri, tastiera kitty dalla 0.13. Nessun supporto alle immagini.
- **Tabby.** Immagini sixel; nessuna tastiera kitty, nessuna forma del puntatore (Electron, xterm.js).
  Tiene `Ctrl+Shift+N` e `Ctrl+,`.
- **xterm.** Avvialo come `xterm -u8 -ti vt340` per sixel (serve UTF-8 per il disegno dei riquadri).
  Riquadri al posto delle emoji: il font.
- **Warp.** Non misurato: richiede XWayland e, nella prova, ha decodificato male i tasti sintetici.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Non fanno parte della sessione di novembre 2026. Funzionano
  con caratteri e tasti tradizionali: usa le varianti con `Alt` e `Ctrl+Y` per ripetere. VTE
  ignora `OSC 52`, quindi la copia via SSH non raggiunge gli appunti. Un metodo di input
  (IBus, Fcitx) può impadronirsi dell'interruttore di stage `Ctrl+Space`; riassegnalo in *Impostazioni >
  Tastiera e mouse*. Konsole è riconosciuto per sixel dalla sua stessa documentazione.
- **Wayland, Hyprland, Omarchy.** Il suggerimento del puntatore (`OSC 22`) funziona in kitty, Ghostty e
  foot. La voce del launcher di Omarchy passa da `xdg-terminal-exec`.

### macOS

- **iTerm2.** Le versioni 3.7.7 e successive disegnano le immagini tramite il protocollo kitty, senza domande.
  Impostazioni utili:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    necessaria per la copia tramite `OSC 52` (altrimenti iTerm2 mostra «the terminal attempted to
    access the clipboard but it was denied»).
  - *Profiles > Keys > Left Option key: Esc+*: necessaria per le combinazioni `Alt`.
  - Se imposti `ui.image_protocol = "iterm2"` (o usi una versione precedente alla 3.7.7), spunta
    «Remember my choice» nella finestra «Allow Terminal-Initiated Display?». Le immagini
    hanno allora uno sfondo opaco (un rettangolo visibile se il tuo sfondo differisce da quello
    del tema; `ui.graph_renderer = "text"` lo elimina).
  - Il protocollo tastiera kitty è attivo di base (3.6.9 e successive).
- **Terminal.app.** Solo 256 colori: GitPoulpi sceglie da sé `ui.color_depth = 256` e i
  temi risultano più piatti. Nessuna immagine, nessuna tastiera kitty, nessuna copia tramite `OSC 52`. Spunta
  *Use Option as Meta key* per le combinazioni `Alt`.
- **Alacritty, Tabby.** Alacritty: grafo a caratteri, tastiera kitty, copia `OSC 52`; imposta
  `option_as_alt = "Both"` per le combinazioni `Alt`. Tabby: grafo e avatar sixel.
- **Option come Meta.** Option compone caratteri (Option+B scrive un segno di integrale) a meno che
  il terminale non lo trasformi in Meta: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. Le combinazioni `Alt` di GitPoulpi lo richiedono, e da nessuna parte è attivo
  di base.
- **kitty, Ghostty, WezTerm, Rio, Warp**: non misurati su macOS (richiedono una GPU). Aspettati il
  comportamento di Linux.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel richiede `ui.image_protocol = "sixel"`.

### VS Code e Warp

Il terminale di VS Code (`TERM_PROGRAM=vscode`) non fa parte della sessione di prove. Viene riconosciuto
dai suoi attributi di dispositivo quando `terminal.integrated.enableImages` è attivo, altrimenti GitPoulpi disegna
caratteri; `ui.image_protocol = "sixel"` forza le immagini. Warp: vedi le note per sistema operativo qui sopra.

## Multiplexer

tmux, zellij, screen, herdr, psmux e i riquadri dei terminali stessi stanno tra GitPoulpi e il
terminale e cambiano ciò che passa. In breve: tmux lascia passare sixel e, con
`allow-passthrough on`, forme del puntatore e immagini kitty; zellij e screen non lasciano passare
né immagini né forme del puntatore; herdr lascia passare le immagini kitty. Tutti i dettagli e la
configurazione da aggiungere sono in [Multiplexer](multiplexers.md).

## Che cosa si perde, e come si compensa

| Manca | Che cosa ottieni al suo posto |
|---|---|
| Protocollo di immagini | Grafo a caratteri. L'avatar diventa il badge colorato con le iniziali (una lettera) qualunque cosa dica `ui.avatar_fallback`: un identicon compresso in celle di testo è irriconoscibile. Il diff di immagini e la Vista file usano semiblocchi colorati, o una scheda informativa. |
| Truecolor | I colori sono mappati sulla tavolozza a 256 o 16 colori (`ui.color_depth`). |
| Protocollo tastiera kitty | Tasti tradizionali; alcune combinazioni arrivano come un altro tasto. Le Impostazioni avvisano, e tu riassegni o usi le varianti con `Alt`. |
| OSC 52 | In locale non cambia nulla (si usano gli appunti di sistema). Via SSH o senza display, le azioni di copia non mettono nulla negli appunti: seleziona con il mouse (tieni `Shift`) e usa la copia del terminale. |
| OSC 22 | Il puntatore non cambia; il divisore o il pulsante sotto il mouse si illumina comunque. |
| Font emoji | Riquadri o buchi al posto delle icone; il grafo e il testo funzionano ancora. Installa un Nerd Font o un font emoji e indicalo come ripiego. |

## Impostazioni da conoscere

| Impostazione | Uso |
|---|---|
| `ui.image_protocol` | `auto` (predefinito), `kitty`, `iterm2`, `sixel`, `none`. Imposta `none` se vedi rumore dove dovrebbero stare le immagini. Riavvio. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` è sempre corretto (usalo dove vale il limite di ConPTY). Riavvio. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` oppure `off`, se un terminale o un multiplexer si comporta male. Riavvio. |
| `ui.text_cursor` | `terminal` (predefinito) oppure `drawn`. |
| `ui.mouse_enabled` | Disattivato lascia il mouse al terminale. Riavvio. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | Il terminale che aprono le scorciatoie del sistema, e se un doppio clic riapre lì GitPoulpi ([Impostazioni](settings.md)). |

Vedi anche [Multiplexer](multiplexers.md), [Immagini](image-view.md),
[Scorciatoie da tastiera](keybindings.md) e [Risoluzione dei problemi](troubleshooting.md).

## Fonti

Documentazione verificata a ottobre 2026, comportamento provato a novembre 2026. Fonti primarie salvo indicazione contraria.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [forme del puntatore](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [sequenze di escape](https://wezterm.org/escape-sequences.html), [pull request OSC 22](https://github.com/wezterm/wezterm/pull/6292), [release](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [release](https://github.com/alacritty/alacritty/releases)
- Konsole: [merge request OSC 52](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [merge request tastiera kitty](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [issue OSC 52](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [issue tastiera kitty](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [download](https://iterm2.com/downloads.html); tastiera kitty dalla [matrice terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secondaria)
- Windows Terminal: [annuncio 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secondaria), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [supporto alle sequenze di escape](https://rioterm.com/docs/escape-sequence-support) (nessuna riga per `OSC 22`), [funzionalità](https://rioterm.com/docs/features), [tastiera kitty](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [app a schermo intero](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 da un [resoconto sull'avviso OSC 52 di Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secondaria)
- VS Code: [terminale avanzato](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [sequenze di controllo](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [suggerimenti](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (senza truecolor): [discussione Apple](https://discussions.apple.com/thread/255315820) (secondaria)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Siti di confronto (secondari, usati solo per un controllo incrociato): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
