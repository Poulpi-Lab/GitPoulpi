[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · **Italiano** · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Scegliere un terminale

GitPoulpi funziona in qualsiasi terminale con supporto del mouse, ma dà il meglio in
uno che disegna immagini, mostra il true color e invia la tastiera completa. Questa
pagina classifica i terminali per sistema operativo, dice che cosa offre ciascuno e quale
impostazione cambiare. I fatti sono stati verificati sulla documentazione, sul changelog o
sul tracker di ogni progetto nell'ottobre 2026 (vedi [Fonti](#fonti)); nulla di tutto questo è
stato provato su ogni terminale. Tutto ciò che è segnato come
**(non verificato)** non ha potuto essere confermato da una fonte primaria.

## Le scelte migliori

| SO | Scelte migliori, in ordine | Perché |
|---|---|---|
| **Windows** | **WezTerm**, poi **Windows Terminal** | WezTerm disegna il grafo e gli avatar come immagini, con trasparenza. Windows Terminal ha la migliore integrazione con il sistema (è il predefinito del launcher); disegna gli avatar come immagini sixel e il grafo a caratteri. |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, poi **foot** | kitty e Ghostty offrono immagini e tastiera completa. Anche WezTerm offre le immagini. foot disegna gli avatar come immagini sixel, è veloce e ha la tastiera completa. Su Omarchy scegli Ghostty o kitty per le immagini. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, poi **iTerm2** | Stesso ragionamento. Terminal.app funziona ma non ha né immagini né true color. |

Su qualsiasi sistema, **tmux, zellij e screen** nascondono a GitPoulpi il protocollo
di immagini del terminale, quindi GitPoulpi disegna caratteri (vedi [Multiplexer](#multiplexer)).

## Che cosa usa GitPoulpi

- **Immagini** (colonna del grafo, avatar degli autori, diff di immagini e Vista file): il
  **protocollo grafico di kitty** con segnaposto Unicode, oppure le **immagini inline di iTerm2**
  (`OSC 1337`), oppure **sixel**, in questo ordine di preferenza. Il rilevamento è
  negato per impostazione predefinita, e parte da `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` e dalle
  variabili dei multiplexer, e non legge mai la tastiera (quindi non chiede nemmeno al
  terminale i suoi attributi di dispositivo). kitty e Ghostty usano il protocollo di kitty.
  WezTerm usa il protocollo di iTerm2 (di proposito) e fonde la trasparenza del PNG. iTerm2,
  Rio e mintty usano il protocollo di iTerm2 con uno sfondo opaco. Windows Terminal, foot,
  Konsole e mlterm usano **sixel**. Senza nessuno dei tre, gli avatar sono il badge con le iniziali.
- **Sixel** è usato per la **colonna del grafo** (corsie, nodi e, con `ui.avatars = "graph"`, gli avatar nei cerchi dei commit) e altrove per gli **avatar** (Log, dettaglio del commit, Cronologia del file, Blame, vista immagine). Un sixel non ha alfa, id né ridimensionamento: l'immagine misura esattamente `colonne x larghezza della cella` per `righe x altezza della cella` pixel, è disegnata solo dove c'è qualcosa, sopra celle vuote che mantengono lo sfondo; l'immagine codificata è in cache, ricodificata solo quando cambiano corsie, scorrimento o zoom, e rinviata solo se una cella sotto di essa è stata ridisegnata. Un sixel non si può cancellare: GitPoulpi scrive del testo sulle sue celle per rimuoverlo. Il sixel richiede la vera dimensione della cella in pixel: a Windows Terminal viene chiesta una volta all'avvio e dopo uno zoom (`CSI 16 t`, risposta in circa 10 ms, mai bloccante se tace); altrove viene dalla dimensione della finestra o da `GITPOULPI_TERMINAL_CELL_PX=10x20`. Un terminale che disegna sixel ma non è riconosciuto (un terminale di VS Code con `terminal.integrated.enableImages`, uno xterm avviato con `-ti vt340`) richiede `ui.image_protocol = "sixel"`. Windows Terminal prima della 1.22 non disegna sixel: lì imposta `ui.image_protocol = "none"` (il grafo resta a caratteri).
- **True color**: i temi sono RGB. `ui.color_depth = "auto"` scende a 256 colori con un
  `TERM` di tipo `*-256color` senza `COLORTERM=truecolor` (e in Terminal.app), e a 16
  colori sulla console Linux. kitty, WezTerm e Windows Terminal contano come true color.
- **Mouse**: segnalazione mouse SGR con pulsanti, trascinamento e movimento (feedback al
  passaggio, drag and drop, clic destro, doppio clic).
- **Tastiera**: il **protocollo tastiera di kitty** (solo il flag disambiguate) viene richiesto
  all'avvio con un timeout di 100 ms. Fa arrivare `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` e
  `Ctrl+Shift+Z`. Senza di esso restano i tasti tradizionali (`Ctrl+Y` ripete, le varianti con `Alt`
  sostituiscono alcune combinazioni). Vedi
  [Scorciatoie da tastiera](keybindings.md#tasti-che-il-terminale-non-può-inviare-e-il-protocollo-tastiera-kitty).
- **Appunti**: le azioni di copia scrivono `OSC 52`.
- **Forma del puntatore del mouse** (puntatore di ridimensionamento sui divisori, mano sugli
  elementi cliccabili): `OSC 22`, un extra dove è implementato.
- **Eventi di focus** abilitati (i tooltip si chiudono quando la finestra perde il focus).
  GitPoulpi non richiede bracketed paste, collegamenti OSC 8 né undercurl.
  Solo sul percorso delle immagini, ogni fotogramma è avvolto in un aggiornamento
  sincronizzato (modalità 2026, ignorata dai terminali che non la conoscono) così un'immagine non
  lampeggia mai sopra un popup.
- **Emoji e simboli**: GitPoulpi usa solo glifi con presentazione emoji predefinita
  o forme BMP a larghezza singola, perché terminali e font non concordano sui selettori di
  variazione.

## Tabella delle capacità

«Immagini» è ciò che GitPoulpi disegna oggi in quel terminale (protocollo **kitty**,
**iTerm2** o **sixel**, oppure **testo**), non ciò che il terminale potrebbe fare.

| Terminale | SO | Immagini (grafo, avatar, diff di immagini) | Truecolor | Tastiera kitty (combinazioni complete) | Copia OSC 52 | Puntatore OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | sì | sì | sì | sì |
| **Ghostty** (1.3) | Linux, macOS (nessuna build per Windows) | kitty | sì | sì | sì | sì, dalla 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, con trasparenza | sì | solo con `enable_kitty_keyboard = true` (predefinito `false`) | sì (solo scrittura) | no (la pull request è ancora aperta) |
| **foot** (1.28) | Linux (Wayland) | sixel (grafo e avatar; nessuna grafica kitty) | sì | sì, dalla 1.10.3 | sì (`security.osc52`) | sì, dalla 1.12 |
| **Alacritty** (0.17) | tutti | testo (nessun supporto alle immagini a monte) | sì | sì, dalla 0.13 | sì (scrittura; incolla disattivato di base) | no (non implementato) |
| **Konsole** | Linux | sixel (grafo e avatar) | sì | in aggiunta nel 2026 (merge request !1188); rilascio (non verificato) | solo scrittura, integrato a metà 2024 | (non verificato) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | testo | sì | no (non è in nessun VTE rilasciato) | no (VTE lo ignora) | no |
| **iTerm2** (3.7) | macOS | iTerm2, sfondo opaco | sì | sì, dalla 3.6.9 (controlla Profiles > Keys) | disattivato di base: attiva «Applications in terminal may access clipboard» | (non verificato) |
| **Terminal.app** | macOS | testo | no (256 colori) | no | (non verificato) | (non verificato) |
| **Windows Terminal** (1.25) | Windows | sixel (grafo e avatar) | sì | sì, stabile dalla 1.25 (ott. 2026); Preview dalla 1.25 (mar. 2026) | sì (scrittura; solo con il focus) | no (non implementato) |
| **Rio** | Windows, Linux, macOS | iTerm2, sfondo opaco (parla anche la grafica kitty) | sì | sì (attivo di base) | sì | no (non implementato) |
| **Warp** | macOS, Linux, Windows | testo (non rilevato) | sì | sì | sì (rispetta la sequenza) | (non verificato) |
| **Terminale di VS Code** (xterm.js) | tutti | testo (le immagini sixel e iTerm2 esistono dietro `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` usa il sixel) | sì | (non verificato) | (non verificato) | no |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, sfondo opaco | sì | (non verificato; non documentato) | (non verificato; non documentato) | sì |
| **Console di Windows** (conhost, cmd, finestra di PowerShell) | Windows | testo | solo Windows 10/11 recenti | non serve: segnala da sé i modificatori | no | no |
| **ConEmu** | Windows | testo | (non verificato) | (non verificato) | (non verificato) | (non verificato) |
| **tmux, zellij, screen** | tutti | testo, sempre | servono impostazioni | servono impostazioni | servono impostazioni | no |

Le celle «no (non implementato)» di OSC 22 vengono dall'assenza nel changelog e nella
documentazione di un progetto più le segnalazioni degli utenti, non da una prova; leggile come
«non documentato». La pagina non tabula la larghezza di emoji e selettori di variazione:
cambia con il terminale e il font, ed è per questo che GitPoulpi evita i glifi VS16 (vedi
[Che cosa usa GitPoulpi](#che-cosa-usa-gitpoulpi)).

## Terminale per terminale

### Windows

- **WezTerm.** Il miglior supporto alle immagini su Windows. Lascia `ui.image_protocol` su `auto`.
  In `wezterm.lua` imposta `config.enable_kitty_keyboard = true` per le combinazioni complete e un
  font con fallback per le emoji, per esempio
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  L'ultima versione stabile etichettata di WezTerm è 20240203-110809-5046fc22 (feb. 2024); le
  correzioni più recenti escono nelle nightly. WezTerm non ha ancora la forma del puntatore OSC 22.
- **Windows Terminal.** True color, mouse, OSC 52 e buone emoji. GitPoulpi disegna
  gli avatar come immagini sixel (Windows Terminal 1.22 o successivo); il grafo è a caratteri. Scegli un Nerd Font in *Settings > Profiles > Defaults > Appearance > Font face*.
  Tieni premuto `Shift` per selezionare il testo mentre il mouse è catturato. La forma del
  puntatore non cambia. Su una build precedente alla 1.25 (senza protocollo tastiera kitty) usa le varianti con `Alt`.
- **Rio** funziona con uno sfondo opaco per le immagini; meno testato.
- **mintty** (Git Bash, MSYS2, Cygwin) disegna le immagini con il protocollo di iTerm2 e uno
  sfondo opaco; meno testato.
- **Console di Windows, ConEmu, cmd**: da evitare. Le immagini vengono disegnate male o per niente.
  Il doppio clic sull'exe riapre GitPoulpi nel terminale che hai scelto
  (*Impostazioni > Launcher*).

### Linux

- **kitty.** Tutto funziona senza impostazioni. `Shift+clic` è di kitty stesso, quindi usa
  `Alt+clic` per estendere una selezione ([Mouse e gesti](mouse-and-gestures.md)).
- **Ghostty.** Tutto funziona senza impostazioni. Se le icone compaiono come buchi, aggiungi un font
  emoji o Nerd Font a `font-family` (ripeti la chiave per aggiungere fallback).
- **WezTerm.** Immagini con il protocollo di iTerm2, trasparenza fusa. Stessa nota su
  `enable_kitty_keyboard` che su Windows.
- **foot.** Grafo a caratteri, tastiera completa, veloce. Gli avatar sono immagini sixel, il
  grafo è a caratteri. Imposta `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`.
- **Alacritty.** Grafo a caratteri, tastiera kitty dalla 0.13. Alacritty non ha supporto
  alle immagini.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Funzionano con i caratteri e i tasti tradizionali: usa
  le varianti con `Alt` e `Ctrl+Y` per ripetere (una Konsole recente potrebbe aggiungere il protocollo
  tastiera kitty). Le azioni di copia in VTE (`OSC 52`) non mettono nulla negli appunti. Un metodo di
  input (IBus, Fcitx) potrebbe intercettare l'interruttore di stage `Ctrl+Space` - **(non verificato)**;
  riassegnalo in *Impostazioni > Tastiera e mouse*.
- **Wayland, Hyprland, Omarchy.** Il suggerimento sul puntatore del mouse (`OSC 22`) funziona in kitty, Ghostty e
  foot. La voce del launcher di Omarchy passa da `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: come su Linux.
- **iTerm2.** Immagini con sfondo opaco (un rettangolo visibile se il tuo sfondo
  differisce da quello del tema; `ui.graph_renderer = "text"` lo elimina). Attiva
  *Settings > General > Selection > Applications in terminal may access clipboard* per
  la copia, e *Profiles > Keys > Left Option key: Esc+* per le combinazioni con `Alt`.
- **Terminal.app.** Solo 256 colori: GitPoulpi sceglie da sé `ui.color_depth = 256` e i
  temi risultano più piatti. Nessuna immagine. Spunta *Use Option as Meta key* per le combinazioni con `Alt`.

### VS Code e Warp

Il terminale di VS Code (`TERM_PROGRAM=vscode`) e Warp non sono riconosciuti come terminali
con immagini, quindi GitPoulpi disegna caratteri. Entrambi sono utilizzabili. In VS Code attiva `terminal.integrated.enableImages` e imposta
`ui.image_protocol = "sixel"` per avere gli avatar come immagini. Warp documenta il
supporto al protocollo tastiera kitty; quello del terminale di VS Code è **(non verificato)**.

## Multiplexer

GitPoulpi rileva tmux, zellij, screen e herdr (`TMUX`, `ZELLIJ`, `STY`, `TERM=screen*`
o `tmux*`, `HERDR_*`) e disegna **caratteri**, perché un multiplexer inoltra celle,
non sequenze di immagini. Per le altre funzioni, in `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Per forzare le immagini attraverso un multiplexer che le inoltra, imposta `ui.image_protocol` su
`kitty` o `iterm2`. herdr con `kitty_graphics = true`
dentro kitty, Ghostty o WezTerm viene rilevato automaticamente; vedi
[Risoluzione dei problemi](troubleshooting.md#il-grafo-è-disegnato-con-caratteri-non-come-immagine-herdr-tmux-altri-terminali).
`GITPOULPI_TERMINAL_IMAGE=off` disattiva tutte le immagini.

tmux non inoltra da sé il protocollo tastiera kitty: `extended-keys` gli fa
inviare i tasti modificati come sequenze `CSI u` o `xterm`. zellij 0.41 e successivi parlano il
protocollo tastiera kitty e inoltrano OSC 52, secondo il suo changelog.

## Che cosa si perde, e come si compensa

| Manca | Che cosa ottieni al suo posto |
|---|---|
| Protocollo di immagini | Grafo a caratteri. L'avatar diventa il badge colorato con le iniziali (una lettera) qualunque cosa dica `ui.avatar_fallback`: un identicon compresso in celle di testo è irriconoscibile. Il diff di immagini e la Vista file usano mezzi blocchi colorati, oppure una scheda informativa. |
| True color | I colori vengono mappati sulla tavolozza a 256 o a 16 colori (`ui.color_depth`). |
| Protocollo tastiera kitty | Tasti tradizionali; alcune combinazioni arrivano come un altro tasto. Le Impostazioni avvisano, e tu riassegni o usi le varianti con `Alt`. |
| OSC 52 | Le azioni di copia non mettono nulla negli appunti: seleziona con il mouse (tieni premuto `Shift`) e usa la copia del terminale. |
| OSC 22 | Il puntatore non cambia; il divisore o il pulsante sotto il mouse si illumina comunque. |
| Font emoji | Riquadri o buchi al posto delle icone; il grafo e il testo funzionano comunque. Installa un Nerd Font o un font emoji e indicalo come fallback. |

## Impostazioni da conoscere

| Impostazione | Uso |
|---|---|
| `ui.image_protocol` | `auto` (predefinito), `kitty`, `iterm2`, `sixel`, `none`. Imposta `none` se vedi caratteri spuri dove dovrebbero esserci le immagini. Riavvio. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` è sempre corretto. Riavvio. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` oppure `off`, se un terminale o un multiplexer si comporta male. Riavvio. |
| `ui.mouse_enabled` | Disattivato lascia il mouse al terminale. Riavvio. |
| `ui.launch_terminal` | Il terminale che aprono i collegamenti del sistema ([Impostazioni](settings.md)). |

Vedi anche [Immagini](image-view.md), [Scorciatoie da tastiera](keybindings.md) e
[Risoluzione dei problemi](troubleshooting.md).

## Fonti

Verificate nell'ottobre 2026. Fonti primarie salvo indicazione contraria.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty keyboard dalla [matrice di terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secondaria)
- Windows Terminal: [annuncio della 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secondaria), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 da un [approfondimento sull'advisory OSC 52 di Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secondaria)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (niente truecolor): [discussione Apple](https://discussions.apple.com/thread/255315820) (secondaria)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Siti di confronto (secondari, usati solo per un controllo incrociato): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Ancora non verificati: sixel in Konsole e mlterm (preso dagli elenchi di funzionalità dei progetti, non ricontrollato su una versione rilasciata e mai provato su una finestra reale in nessun terminale); OSC 22 in Konsole, iTerm2, Terminal.app e Warp; OSC 52 in Terminal.app,
VS Code e mintty; il protocollo tastiera kitty in VS Code, mintty e ConEmu; ogni cella di
ConEmu; e la larghezza delle emoji per terminale. La versione in cui Konsole include il protocollo
tastiera kitty non è stata confermata.
