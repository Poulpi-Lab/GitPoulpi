[English](../multiplexers.md) · **Italiano**

# Multiplexer e riquadri

Un multiplexer sta tra GitPoulpi e il tuo terminale: ridisegna lui le celle e inoltra
solo le sequenze di escape che capisce. Quindi ciò che funziona nel solo terminale
([Scegliere un terminale](terminals.md)) può non funzionare dentro tmux, zellij, screen o herdr.
Questa pagina dice che cosa passa e la configurazione da aggiungere. I risultati vengono dalla stessa
sessione di prove di novembre 2026 delle tabelle dei terminali: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 e
0.46.0, GNU screen 5.0 (macOS include la 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi riconosce un multiplexer da `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` o `tmux*`
e `HERDR_*`. Dentro uno di essi le immagini ripiegano sui caratteri, salvo dove la tabella qui sotto
dice che GitPoulpi rileva un percorso funzionante. `GITPOULPI_TERMINAL_IMAGE=off` disattiva tutte
le immagini, e `ui.image_protocol` forza un protocollo.

## A colpo d'occhio

| | Immagini | Forma del puntatore (OSC 22) | Appunti (OSC 52) | Tasti |
|---|---|---|---|---|
| **tmux** | sixel quando il terminale dietro lo accetta (rilevato); immagini kitty con `allow-passthrough on` e `ui.image_protocol = "kitty"` | con `allow-passthrough on`, in kitty e foot; non in Ghostty | con `set-clipboard on` | nessuna tastiera kitty; `Ctrl+Enter` e `Ctrl+,` non arrivano mai |
| **zellij** | caratteri | no | sì | la tastiera kitty funziona |
| **GNU screen** | caratteri | no | no | tasti tradizionali |
| **herdr** | immagini kitty (rilevate con `kitty_graphics = true`) | no | sì | non misurato |
| **psmux** (Windows) | caratteri (per scelta) | no (Windows Terminal non ce l'ha) | gli appunti di sistema funzionano | non misurato |
| **tmux di MSYS2** (Windows) | caratteri (per scelta) | no | gli appunti di sistema funzionano | non misurato; lo schermo è alterato |
| **Riquadri di WezTerm**, **riquadri di Windows Terminal**, **split di kitty e Ghostty**, **riquadri divisi di iTerm2** | come il terminale stesso | come il terminale | come il terminale | come il terminale |
| **`tmux -CC` di iTerm2** | caratteri | no | no (iTerm2 lo nega di base) | non misurato |

## tmux

**Immagini.**

- **Sixel** è disegnato da tmux stesso, ricodificato per un client che lo supporta. GitPoulpi
  chiede a tmux una volta all'avvio (`tmux display-message -p '#{client_termfeatures}'`,
  limitato a 400 ms) e usa sixel quando il client lo elenca. È il caso dietro foot,
  WezTerm, Rio e Tabby; non è il caso dietro kitty e Ghostty (tmux allora stampa il testo
  «SIXEL IMAGE»). Richiede un tmux compilato con sixel (quelli di Arch e Homebrew lo sono;
  controlla con `tmux -V` e una prova sixel). Nessuna impostazione. La risposta di tmux
  a una query sugli attributi di dispositivo dichiara sixel qualunque cosa ci sia dietro, ed è per questo che GitPoulpi chiede
  invece le funzionalità del client.
- **iTerm2 dietro tmux** dichiara sixel ma dipinge opachi i pixel non impostati, quindi il
  grafo, in gran parte trasparente, diventerebbe un rettangolo pieno. GitPoulpi riconosce iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, il tipo di client) e disegna caratteri.
- Le **immagini kitty** passano dal passthrough DCS di tmux (kitty, Ghostty; viste anche in
  iTerm2 e Rio in una sonda grezza). Richiedono `set -g allow-passthrough on` (tmux 3.3a o
  successivo) **e** `ui.image_protocol = "kitty"` in GitPoulpi, che allora avvolge i caricamenti
  e le cancellazioni. Senza l'opzione di tmux l'involucro viene scartato e il grafo resta vuoto.
- Le immagini inline di iTerm2 passano da tmux ma tmux non può posizionarle: GitPoulpi non le
  usa mai lì.

**Forma del puntatore.** GitPoulpi avvolge `OSC 22` per il passthrough. Raggiunge la finestra in
kitty e foot con `allow-passthrough on`. Ghostty non la mostra quando è avvolta (resta
la freccia). WezTerm, Rio, Alacritty, Tabby, iTerm2 non hanno un `OSC 22` da mostrare.

**Appunti.** `set -g set-clipboard on`. GitPoulpi invia anche la forma avvolta per il passthrough
(funziona con `allow-passthrough on`). Dietro kitty, Ghostty, Alacritty e Tabby
arriva; iTerm2 lo nega finché il suo permesso per gli appunti non è attivo.

**Tasti.** tmux non risponde alla query della tastiera kitty, e `extended-keys` non
fa passare `Ctrl+Enter` né `Ctrl+,`: misurato con tmux 3.8 dentro kitty, entrambi arrivano
come semplici `Enter` e `,` con `extended-keys` off, on o always. Le combinazioni con lettere come
`Ctrl+Shift+Z` arrivano solo con `extended-keys` on o always **e**
`extended-keys-format csi-u` (tmux 3.5 o successivo) **e** un'applicazione che chieda i
tasti estesi; GitPoulpi non lo fa mai, quindi l'impostazione qui non cambia nulla. Dentro tmux usa le varianti
con `Alt`
([Scorciatoie da tastiera](keybindings.md#tasti-che-il-terminale-non-può-inviare-e-il-protocollo-tastiera-kitty)).

**Configurazione** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a o successivo: forme del puntatore, immagini kitty forzate
```

**tmux dentro WSL.** Non misurato. È la build
Linux, quindi aspettati lo stesso di sopra con Windows Terminal come terminale esterno.

## zellij

Immagini: zellij accetta il protocollo kitty ma non i suoi segnaposto Unicode, e non porta
né immagini sixel né iTerm2, quindi GitPoulpi disegna caratteri (un veto deliberato). Forma del puntatore:
inghiottita, quindi GitPoulpi non la invia. Appunti: `OSC 52` viene inoltrato (funziona dietro
kitty, Ghostty, Alacritty, Tabby; iTerm2 e Terminal.app lo negano o lo ignorano). Tasti: zellij
risponde alla query della tastiera kitty, quindi la tastiera completa funziona.

zellij 0.46 apre un popup «Welcome to Zellij» al primo avvio: premi `Esc`. Una sessione che
è terminata mantiene il suo nome e il prossimo avvio con lo stesso nome chiede che cosa fare.
Non serve alcuna configurazione.

## GNU screen

Non passa nulla: immagini e `OSC 22` vengono stampate come caratteri o scartate, `OSC 52` è
ignorato, e la tastiera è quella tradizionale. GitPoulpi disegna caratteri. screen 4.00.03 (la
versione inclusa in macOS) altera anche i colori e i glifi larghi nell'interfaccia; screen 5 è stato
provato solo su Linux. Usa tmux o zellij per una sessione utilizzabile.

## herdr

herdr incorpora un terminale che riemette le **immagini kitty**. GitPoulpi le usa quando il
`config.toml` di herdr dice `kitty_graphics = true` (sotto `[terminal]`, o `[experimental]` nelle versioni
più vecchie) e il terminale attorno a herdr le disegna (kitty, Ghostty, WezTerm): rilevato,
nessuna impostazione. Se la configurazione sta altrove (per esempio sotto `HERDR_HOME`, che GitPoulpi
non legge) imposta `ui.image_protocol = "kitty"`. Via SSH (`herdr --remote`) GitPoulpi
non vede il client locale, quindi disegna caratteri; vedi
[Risoluzione dei problemi](troubleshooting.md#il-grafo-è-disegnato-con-caratteri-non-come-immagine-herdr-tmux-altri-terminali).
`OSC 22` non è inoltrato da herdr 0.9.5, quindi GitPoulpi non lo invia. `OSC 52` arriva
dietro kitty, Ghostty, Alacritty e Tabby. Nella sonda grezza herdr ha disegnato immagini kitty in
kitty, Ghostty e iTerm2, anche con la configurazione predefinita. Tasti: non misurato.

## psmux e tmux di MSYS2 (Windows)

- **psmux** (Rust, ConPTY, linguaggio di comandi di tmux; `winget install psmux`) imposta `TMUX`, quindi
  GitPoulpi lì disegna caratteri per scelta. Dentro Windows Terminal gira normalmente
  e la copia funziona tramite gli appunti di sistema; il puntatore non cambia (Windows
  Terminal non ha `OSC 22`). Gli altri terminali ospitanti non sono stati misurati.
- **tmux da MSYS2 o Cygwin** imposta anch'esso `TMUX` (caratteri per scelta), e nella prova lo
  schermo era alterato: fotogrammi vecchi disegnati uno sull'altro in mintty e in Windows Terminal
  (con e senza `-u`; senza, il disegno dei riquadri stampava `qqqq`). Un programma nativo di Windows
  dentro uno pseudo-terminale MSYS2 non viene ridisegnato correttamente da tmux. Non si può
  correggere in GitPoulpi: usa psmux, i riquadri di Windows Terminal o i riquadri di WezTerm.
- zellij e screen non hanno una release per Windows. herdr 0.9.5 è stato installato ma non eseguito.

## Riquadri del terminale

I riquadri che appartengono al terminale stesso non sono multiplexer per GitPoulpi:

- **Riquadri di WezTerm** (`wezterm cli split-pane`, `wezterm connect`, domini SSH): WezTerm disegna
  tutto, quindi le immagini funzionano (grafo e avatar nel riquadro) e il puntatore si comporta come in
  WezTerm stesso (le forme funzionano in una build con la PR di WezTerm a monte #8092; il divisore dei riquadri
  dà il puntatore di ridimensionamento). Non misurato su Linux né su macOS.
- **Riquadri di Windows Terminal** (`wt ; split-pane`): grafo e avatar sixel funzionano nel riquadro,
  la copia funziona, nessuna forma del puntatore.
- **Split di kitty, Ghostty e iTerm2**: nativi, come il terminale. Non misurato su macOS per
  kitty e Ghostty (richiedono una GPU).

## `tmux -CC` di iTerm2 (modalità di controllo)

iTerm2 apre una finestra nativa per ogni finestra di tmux e tiene una finestra gateway «[tmux bash]».
`TMUX` è impostato, quindi GitPoulpi disegna caratteri nella finestra nativa. Le immagini arrivano
a iTerm2 attraverso la connessione di controllo (la finestra «Allow Terminal-Initiated Display?» è comparsa
per una prova `OSC 1337`), ma GitPoulpi non le usa lì. Il payload kitty di un
`ui.image_protocol = "kitty"` forzato viene stampato nella finestra gateway, quindi lascialo su
`auto`. `OSC 22` e `OSC 52` non arrivano.

## Impostazioni da conoscere

| Impostazione | Uso |
|---|---|
| `ui.image_protocol` | `kitty` forza le immagini kitty attraverso un multiplexer che le inoltra (tmux con `allow-passthrough on`, herdr con una configurazione che GitPoulpi non legge); `none` per i caratteri. |
| `ui.graph_renderer` | `text` è sempre corretto. |
| `ui.kitty_keyboard` | `off` se un multiplexer si comporta male con la query della tastiera. |

Vedi [Scegliere un terminale](terminals.md) per il terminale dietro il multiplexer, e
[Risoluzione dei problemi](troubleshooting.md) quando il grafo è disegnato con caratteri.
