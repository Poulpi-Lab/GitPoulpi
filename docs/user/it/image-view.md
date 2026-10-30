[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · **Italiano** · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Immagini nella vista Diff e nella Vista file

Un file binario che è un'immagine viene mostrato come immagine, non come avviso
"(no textual changes - binary content...)". La vista Diff mostra il
**prima** e il **dopo**; la Vista file mostra il file com'è; l'editor di merge
mostra i lati di un'immagine in conflitto. Tutto qui è in sola lettura:
guardare un'immagine non cambia mai il repository, quindi non c'è nulla da annullare.

Disattivalo con **Impostazioni > Immagini > Anteprima delle immagini** (`ui.image_view`): torna
il semplice avviso di file binario. Una build personalizzata senza la funzione `image-view`
non ha alcun visualizzatore e la pagina Immagini conserva solo l'elenco degli editor
(vedi [Build personalizzate](custom-builds.md)).

## Quali file

PNG e JPEG sempre; GIF, WebP (il loro primo fotogramma), BMP e ICO/CUR quando la
build ha `image-view`. TIFF, AVIF, HEIC, PSD e RAW sono riconosciuti ma non
decodificati: una scheda informativa indica il formato, la dimensione e le dimensioni in pixel. SVG è testo
e mantiene il diff di testo. Un'immagine oltre **Dimensione max anteprima** (32 MiB per impostazione predefinita)
o oltre 64 megapixel mostra invece una scheda informativa. Un file memorizzato con Git LFS viene
mostrato quando il suo oggetto è nell'archivio locale; altrimenti la scheda dice di
usare prima `LFS > Pull`. La trasparenza è disegnata su una scacchiera.

## Come viene disegnata

In un terminale con un protocollo per le immagini (Kitty e Ghostty, WezTerm e iTerm2,
Rio, mintty) l'immagine è un'immagine vera. Altrove (Windows Terminal semplice,
tmux, un terminale che GitPoulpi non conosce) viene disegnata con caratteri colorati a
mezzo blocco, con metà della risoluzione verticale ma leggibile ovunque; **Senza protocollo
immagini** può essere impostato su *Solo scheda informativa*. **Protocollo immagini** nella
pagina **Log e grafo** ne forza uno.

## La vista Diff

Apri il diff di un file immagine (quello di un commit, la metà non in stage o in stage
delle modifiche in corso). L'intestazione nomina entrambi i file (`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`),
poi i pulsanti dello zoom, **Adatta**, **1:1** e **Modifica in**.

- **Affiancata | In linea** a destra dell'intestazione, o `m`: affiancate, oppure una
  sola immagine alla volta. **Tab**, o un clic su *Prima | Dopo*, mostra l'altra
  nella vista in linea. Uno split troppo stretto per due pannelli viene disegnato in linea
  senza cambiare la tua impostazione (`ui.diff_layout`).
- Il **divisore** di uno split si sposta trascinandolo, con `Alt+Right` /
  `Alt+Left` (il pannello prima prende / cede spazio) e torna al centro con
  un doppio clic o `Alt+0`. La proporzione viene ricordata per la sessione.
- Un file nuovo mostra un solo pannello con la didascalia *Nuovo file*, uno eliminato *File
  eliminato*; entrambi nascondono l'interruttore. Un'immagine non tracciata selezionata nelle modifiche
  in corso viene mostrata in anteprima come file nuovo.
- Il lato prima è il commit genitore (diff di un commit), `HEAD` (in stage) o
  l'indice (non in stage); il lato dopo è il commit, l'indice o il file
  su disco. Se il file su disco cambia mentre il diff è aperto (lo hai salvato da
  un editor, uno strumento lo ha riscritto) l'immagine lo segue entro circa mezzo secondo.

## La Vista file

Apri un'immagine con **Vista file** dal menu di un file o dalla barra del diff: un solo pannello,
il file alla revisione (o su disco per le modifiche in corso).

## Zoom e scorrimento

| | |
|---|---|
| `+` / `-` (o `=`) | Ingrandisci / riduci, attorno al centro del pannello |
| `0` | **Adatta** (predefinito): l'intera immagine, che resta adattata quando la finestra o il divisore vengono ridimensionati |
| `1` | Dimensione reale (1:1) |
| Frecce, `h j k l` | Scorrono di un decimo del pannello; con `Shift` di un pannello intero; `PageUp`/`PageDown`, `Home`/`End` |
| Rotella | Scorre in verticale; `Shift`+rotella in orizzontale; `Ctrl`+rotella ingrandisce attorno al puntatore |
| Trascinamento | Afferra l'immagine e spostala; le barre di scorrimento ai lati funzionano come ovunque |

Una vista ingrandita mantiene la sua scala e il punto al centro quando il pannello viene
ridimensionato; Adatta resta Adatta. Nella vista Diff i due pannelli condividono uno zoom e una
posizione di scorrimento (**Sincronizza zoom e scorrimento prima/dopo**, attivo per impostazione predefinita), così la
stessa parte di entrambe le immagini è in vista; quando hanno dimensioni diverse lo zoom viene mantenuto
relativo a ciascuna. A partire da **Zoom pixelato da** (2x per impostazione predefinita) l'immagine viene
ingrandita a pixel quadrati invece di essere sfumata, il che va bene per icone e pixel art.

## Modifica in

Il pulsante **Modifica in** dell'intestazione elenca gli editor di immagini trovati su questa
macchina (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Anteprima, ... e l'app predefinita
del sistema); `o` esegue quello predefinito e `Shift+O` apre l'elenco.
È anche nel menu del clic destro di un'immagine negli elenchi dei file e nella
palette dei comandi (*Immagine: Modifica in...*). L'elenco e il predefinito si impostano in
**Impostazioni > Immagini**, nello stesso modo in cui funziona [Apri in](open-in.md).

Modificare il file su disco (il lato delle modifiche in corso) è diretto: l'editor salva
sul posto e l'immagine si aggiorna. Qualsiasi altra versione (quella di un commit, l'indice, un
file eliminato) viene scritta in una **copia temporanea in sola lettura**, con un avviso nella
barra di stato, così un editor non può cambiare il repository.

Le applicazioni che il sistema registra per il tipo dell'immagine (il suo elenco *Apri con*) seguono i tuoi editor nella lista, senza ripetere un programma già presente. Gli altri file binari hanno la stessa lista in [Apri in](open-in.md).

## Nell'editor di merge

Un'immagine in conflitto mostra la sua scheda con le immagini **A**, **B** e (quando
c'è) **Base** nei riquadri, affiancate, ciascuna adattata al suo riquadro e in
sola lettura (qui niente zoom; le immagini servono solo ad aiutarti a scegliere). I pulsanti
e i tasti restano invariati: **Tieni A** (`a`), **Prendi B** (`b`), **Prendi base** (`o`),
**Apri nello strumento di merge** (`m`). Le anteprime richiedono una finestra abbastanza larga (circa 70
colonne) e qualche riga libera; se è più stretta, la scheda è quella di solo testo. Vedi
[Risolvere i conflitti di merge](merge-conflicts.md).

## Tasti e impostazioni

I tasti sono in [Scorciatoie da tastiera](keybindings.md#vista-immagine); ognuno è
riassegnabile e raggiungibile dalla palette (`Ctrl+P`, digita *Immagine*). Le opzioni
sono nella pagina **Immagini** delle [Impostazioni](settings.md#immagini) e nel
[riferimento della configurazione](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
