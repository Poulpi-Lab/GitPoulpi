[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · **Italiano** · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Apri in

Il pulsante **Apri in ▾** nella barra del titolo, accanto al percorso del repository, apre la
cartella della scheda attiva altrove: nel terminale, in una nuova scheda o pannello del
multiplexer, in un editor, nel gestore file. Sei tu a scegliere cosa c'è nell'elenco. Niente è
cablato: ogni voce è un'etichetta e un comando.

## Come si usa

- **Clic su Apri in ▾**: si apre un elenco. Fai clic su una voce per eseguirla. La voce
  predefinita è segnata con `•`; l'ultima riga, **Configura...**, apre
  [Impostazioni](settings.md#apri-in) sulla pagina Apri in.

  ![L'elenco Apri in: Terminale, due editor, uno strumento Git e il gestore file, con la voce predefinita segnata da un pallino e Configura in fondo](../images/open-in-menu.png)

- **Tastiera**: `Alt+O` apre lo stesso elenco (`Su`/`Giù`, `Enter`, `Esc`; con più di
  sette voci, digita per cercare). `Ctrl+Alt+O` esegue subito la voce predefinita.
  Palette (`Ctrl+P`): `Scheda: Apri in...` e `Scheda: Apri nella voce predefinita`.
  I comandi della palette funzionano anche quando nella barra del titolo non c'è spazio per il pulsante.
  Entrambi i tasti sono riassegnabili; vedi [Scorciatoie da tastiera](keybindings.md).
- **Clic destro su una scheda di repository**: il menu della scheda ha un sottomenu **Apri in** con
  le stesse voci.
- L'elenco parte dalla voce che hai usato per ultima in questa sessione.
- La barra di stato mostra subito "apertura in ...". Se il programma fallisce entro circa
  1,5 secondi (per esempio tmux senza server, o un'opzione errata), la prima riga del
  suo output di errore compare nella barra di stato. Un programma non installato viene
  nominato subito.

Finché non salvi un tuo elenco, la voce predefinita viene scelta in base a ciò che viene rilevato: dentro herdr, "Herdr: new tab"; su Omarchy, la nuova finestra di kitty (kitty è il suo terminale); altrimenti il gestore file.

Il pulsante compare su ogni scheda di repository, compresa la cartella in cui è stato
avviato GitPoulpi, e anche in modalità `ui.tab_dir_follow = "auto"`. In modalità `auto` la voce
predefinita viene eseguita anche da sola quando apri una nuova scheda (non per le schede ripristinate
all'avvio). Vedi [Seguire la cartella della scheda attiva](following-the-tab-folder.md).

## Com'è fatta una voce

In `config.toml`, una tabella `[[ui.open_in]]` per voce:

```toml
[ui]
open_in_default = "Herdr: new tab"   # a label; empty means the first entry

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Raramente la scrivi a mano: la modifica la pagina delle Impostazioni. Ogni chiave è nel
[riferimento della configurazione](config-reference.md).

### Comandi e variabili

Il comando viene prima diviso in argomenti (gli spazi, `'...'` e `"..."` raggruppano le
parole), poi le variabili vengono sostituite dentro ogni argomento. Una cartella con uno
spazio nel nome resta quindi un solo argomento. **Non interviene nessuna shell**, a meno che
il tuo comando non ne avvii una.

| Variabile | Sostituita da |
|---|---|
| `${cwd}` | La cartella della scheda. |
| `${repo}` | La radice del repository (differisce da `${cwd}` per una scheda worktree). |
| `${name}` | Il nome base della cartella. |
| `${branch}` | Il branch in checkout; vuoto con HEAD staccato. |
| `${web_url}` | La pagina web del remoto origin se è GitHub o GitLab; altrimenti vuoto. |
| `${env:NAME}` | La variabile d'ambiente `NAME`; vuota se non è impostata. |

Il vecchio `{repo}` funziona ancora e significa `${cwd}`. Qualsiasi altro `${...}` è un errore
che nomina la variabile.

Il programma viene avviato con la cartella della scheda come directory di lavoro e riceve
tre variabili d'ambiente, comode per uno snippet di shell:

| Variabile | Valore |
|---|---|
| `GITPOULPI_CWD` | La cartella della scheda. |
| `GITPOULPI_REPO` | La radice del repository. |
| `GITPOULPI_BRANCH` | Il branch, o vuoto. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

Su Windows il programma viene cercato tramite `PATH` e `PATHEXT`, quindi `code` e
`cursor` (file `.cmd`) funzionano, senza che lampeggi alcuna finestra di console.

## Preimpostazioni

Impostazioni > Apri in > **Aggiungi preimpostazione** elenca voci pronte solo per il tuo
sistema operativo (le preimpostazioni di Windows non compaiono mai su Linux, e così via), per categoria: la
riga inizia con la categoria, `Esplora risorse: ...`, `Multiplexer: ...`, `Terminale: ...`,
`Editor: ...`, `Strumento Git: ...`, `Agente IA: ...`, `Web: ...`. In cima, **Rilevato** sono le
voci dell'elenco predefinito calcolato. All'interno di ogni categoria vengono prima le preimpostazioni che puoi
usare subito (sei dentro quel multiplexer o terminale e il
programma è nel `PATH`), poi quelle installate, poi le altre. Digita nella combobox
per cercare, per esempio `lazygit` o `Strumento Git:`. **Prova sulla scheda corrente** permette di provare una voce
prima di farci affidamento.

![L'elenco + Aggiungi della pagina Apri in: prima le voci rilevate, poi Esplora risorse, Multiplexer e le altre categorie, ogni preimpostazione con il prefisso della sua categoria](../images/settings-open-in-presets.png)

Le preimpostazioni "esegui uno strumento in un pannello" (`lazygit in Herdr tab`, `yazi in tmux window`,
`Claude Code in Herdr split`...) vengono proposte solo quando l'host (herdr, tmux,
zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows Terminal) e lo strumento sono
installati. `$EDITOR in ...` esegue l'editor indicato dalla tua variabile `EDITOR`
(con ripiego su `vi` in herdr) in herdr o tmux.

Le preimpostazioni di herdr per gli strumenti creano la scheda o lo split, poi digitano il comando nella
sua shell, così lo strumento parte nella cartella e il pannello resta aperto dopo che è terminato.
Quelle `Agente IA` usano `herdr agent start`, così herdr tiene traccia dello stato dell'agente.

| Categoria | Preimpostazioni (`V` = verificata eseguendola o confrontandola con l'help installato) |
|---|---|
| Esplora risorse | Windows `explorer.exe` (V), Finder di macOS (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger in un pannello/scheda/terminale |
| Multiplexer | herdr scheda, workspace, split a destra/in basso (V); tmux finestra, split a destra/in basso, sessione staccata (V); zellij pannello, scheda, fluttuante; WezTerm scheda, split, nuova finestra; kitty scheda, finestra in split, finestra del sistema (V: opzioni dall'help; richiede `allow_remote_control`); screen; cmux; psmux |
| Terminale | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (finestra, scheda, split e una scheda ciascuno per PowerShell 7, Windows PowerShell, Prompt dei comandi, WSL), finestre di PowerShell 7 / Windows PowerShell / Prompt dei comandi / WSL, Git Bash |
| Editor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` in un pannello/scheda/terminale |
| Strumento Git | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui in un pannello/scheda/terminale (quelle di herdr, tmux, kitty e `xdg-terminal-exec` sono V) |
| Agente IA | Claude Code, Codex, Gemini CLI, opencode in una scheda/split di herdr (V, con Claude), tmux, zellij, WezTerm, kitty, nuovo terminale |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL su Windows: `wsl.exe --cd ${cwd}` accetta il percorso Windows così com'è, e
la scheda `wsl.exe` di Windows Terminal parte nella cartella, che WSL mappa sotto
`/mnt/`.

Esplora risorse, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual Studio
e Zed (Zed è solo Unix) compaiono solo dove esistono; tmux, zellij, cmux,
screen, il controllo remoto di kitty e le preimpostazioni herdr basate su `sh` sono solo Unix;
herdr (preimpostazioni semplici), WezTerm, Alacritty, VS Code, Cursor e gli IDE JetBrains sono proposti
ovunque.

### Variabile `${web_url}`

La pagina web del remoto `origin` del repository, quando è un progetto GitHub o GitLab
(github.com, gitlab.com, o un host self-managed che ha una voce in
`[[integrations]]`); altrimenti vuota, e allora una voce che la usa si ferma con
un messaggio invece di aprire un indirizzo vuoto. È facoltativa: nessuna preimpostazione ne ha bisogno,
tranne `Repository page on the web`.

## La pagina delle Impostazioni

Impostazioni > **Apri in** modifica l'elenco. Le voci formano un
[elenco](settings.md#elenchi) a sinistra (la voce predefinita è segnata con `★`), i campi
della voce selezionata stanno a destra; `+ Aggiungi`, `Duplica`, `Rimuovi`, `↑` / `↓` e
il trascinamento di una riga gestiscono l'elenco.

| Riga | Cosa fa |
|---|---|
| **Voce predefinita** | La voce che **Apri nella voce predefinita** esegue. È segnata con `•` nell'elenco Apri in. |
| **Etichetta** | Il nome mostrato nell'elenco. Univoco, non vuoto. |
| **Comando** | Il comando, con i chip delle variabili (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) da inserire. |
| **Prova sulla scheda corrente** | Esegue subito la voce sulla cartella della scheda attiva. |
| **+ Aggiungi ▾** | Aggiunge in coda una voce pronta (prima quelle rilevate), oppure **Personalizzata**, una voce vuota di cui si modifica subito il Comando. |
| **Ripristina i valori rilevati** | Sostituisce l'elenco con quanto viene rilevato su questa macchina e cancella il predefinito. |

Le modifiche vengono salvate man mano. La prima modifica scrive l'intero array `[[ui.open_in]]`
in `config.toml`; fino ad allora l'elenco è calcolato (vedi [Chiavi obsolete](#chiavi-obsolete)) e il file
non ha la chiave `open_in`. Anche la ricerca nelle Impostazioni trova la pagina.

## File binari: Apri in

Un file binario (un PDF, un documento Word, un foglio di calcolo, un archivio...) non ha testo da mostrare nella vista Diff o nella vista file. Quando il sistema operativo conosce un'applicazione per il suo tipo, la vista mostra un pulsante **Apri in** accanto all'avviso consueto. La sua lista contiene l'applicazione con cui il sistema apre quel tipo, per prima e contrassegnata da un punto elenco, poi tutte le altre applicazioni che il sistema registra per esso (l'elenco *Apri con* del sistema). Su Linux è il tipo MIME del nome del file, `mimeapps.list` per quella predefinita e i file `.desktop` per le altre; su Windows il registro (la scelta dell'utente e i programmi registrati per l'estensione); su macOS solo l'applicazione predefinita.

`o` esegue l'applicazione predefinita e `Shift+O` apre la lista. Entrambi sono nella palette dei comandi (*File: Apri nell'app predefinita*, *File: Apri con...*), e **Apri con...** è nel menu contestuale di un file del pannello Stato e dell'elenco dei file di un commit. La ricerca avviene in background, una volta per estensione: nel frattempo la vista dice "ricerca delle app...". Un tipo per cui il sistema non conosce alcuna applicazione, e il testo semplice, mantengono l'avviso semplice e non hanno pulsante.

![Un PDF nella vista del file con il suo elenco Apri in: prima l'applicazione predefinita, segnata da un punto, poi le altre applicazioni che il sistema registra per il tipo](../images/open-with.png)

Il file della directory di lavoro è consegnato all'applicazione così com'è. La versione di un commit (o dell'indice) è scritta in una **copia temporanea di sola lettura**, con un avviso nella barra di stato, così un'applicazione non può modificare il repository. Nessuna delle due modifica il repository: nessuna voce nel giornale, Annulla non è coinvolto.

## Chiavi obsolete

Le vecchie chiavi `ui.terminal_launch` e `ui.terminal_launch_command` vengono lette solo
finché `ui.open_in` è assente. Allora l'elenco calcolato è:

1. la tua vecchia scelta del terminale, se non è `auto` (`custom` diventa
   "Terminal (custom)" con `{repo}` riscritto in `${cwd}`; un id di preimpostazione diventa
   quella preimpostazione). Un valore `auto` diventa la prima voce, "Default terminal".
2. il gestore file del sistema operativo,
3. i multiplexer e i terminali rilevati dall'ambiente,
4. gli editor trovati nel `PATH`.

Quando salvi l'elenco dalle Impostazioni, `ui.open_in` prende il sopravvento e le vecchie chiavi
vengono ignorate. I comandi della palette **Scheda: Apri nel terminale** e **Scheda: Apri nel
gestore file** non usano questo elenco.

## Risoluzione dei problemi

- *"program X was not found"*: il programma non è nel `PATH` visto da
  GitPoulpi. Avvia GitPoulpi da una shell che lo ha, oppure scrivi il percorso completo.
- *La voce tmux/zellij/herdr fallisce subito*: questi parlano con un server in esecuzione, quindi
  funzionano solo quando GitPoulpi gira al loro interno.
- *La voce della scheda kitty fallisce*: richiede `allow_remote_control` in `kitty.conf`.
- *Esplora risorse di Windows*: `explorer.exe` termina con codice 1 anche in caso di successo; GitPoulpi
  lo considera un successo.
