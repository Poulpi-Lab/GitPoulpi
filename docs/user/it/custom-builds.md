[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · **Italiano** · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Build personalizzate

Per chi vuole un binario più piccolo: uno script interattivo chiede quali
lingue, integrazioni di hosting, temi, colori del logo e funzioni includere, poi compila solo
quelli con il profilo ottimizzato. La build normale e la release ufficiale
restano invariate (tutto attivo).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd or PowerShell (scripts/build-custom.ps1)
```

L'output finisce in `target/custom/`, mai sopra l'ufficiale `target/dist/`. Lo
script stampa la dimensione di ogni binario e la differenza rispetto a `target/dist/` quando
quella build esiste. Scrive anche `BUILD-FEATURES.txt` accanto ai binari.

## Cosa scegli

| Scelta | Se la disattivi |
|---|---|
| Lingue | Solo l'inglese (sempre compilato) più quelle che spunti. Il selettore delle Impostazioni elenca solo le lingue compilate. |
| Temi | GitPoulpi Dark, GitPoulpi Light e Terminal sono sempre compilati (`system` richiede Dark e Light). Ogni altro tema si sceglie per famiglia (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` le stampa). I temi non selezionati non vengono compilati: mancano dal combo delle Impostazioni e da `gitpoulpi theme list`, e una configurazione che ne nomina uno ripiega come per un tema sconosciuto, con il solito avviso. |
| Colori del logo | Il polpo a pixel della finestra Informazioni esiste in 20 varianti di colore (`--list-logo-colors`). Scegli quelle da compilare (almeno una), quella predefinita (Informazioni la mostra per prima; è anche il valore a cui ripiega un `ui.logo_color` memorizzato che non è compilato; deve essere una di quelle compilate; predefinito ufficiale: lavender) e il colore dell'icona dell'eseguibile Windows (qualsiasi variante, predefinito: quello predefinito del logo). Con una sola variante il logo gira comunque al clic ma resta di quel colore, e l'elenco dei colori (`c`, clic destro) non viene offerto. |
| Integrazioni (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Senza nessuna: solo clone tramite URL; niente issue, né pull/merge request. |
| IA | Niente pulsanti, comandi né strumenti `ai` della CLI/MCP. |
| MCP | Niente binario `gitpoulpi-mcp` né comando `gitpoulpi mcp`. I comandi agente `ui` restano. |
| LFS, Worktree, Sottomoduli | Le schermate di gestione e i comandi corrispondenti spariscono; viene restituita una risposta tipizzata "not built into this binary". |
| Download degli avatar | Solo badge con le iniziali, nessun download dalla rete. |
| Anteprima immagini | I file immagine mostrano il semplice avviso binario; **Modifica in** funziona comunque. |
| Radar | Niente scheda Radar, `Alt+R` né comando della palette; il Radar non si apre mai. Richiede un'integrazione (senza, è comunque inerte). |
| Mermaid | Niente Copia come Mermaid gitGraph (menu del Log, `Shift+Y`, palette) né `log --format mermaid` nella CLI/MCP. |
| Board | Niente vista Bacheca Kanban (Intestazione ISSUE > Bacheca, palette) né comando `hosting board`. Richiede un'integrazione. |
| Review | Niente vista di revisione del codice nell'app (menu della riga, Enter su una riga di pull request, palette, `Revisione` del Radar) né assistente IA per la revisione. I comandi CLI/MCP `hosting review` restano (li usano Gerrit e gli agenti). Richiede un'integrazione. |
| Pannello della cronologia (riscrittura avanzata della cronologia: autori, date, messaggi, contenuto) | Niente Pannello di riscrittura della cronologia: niente comandi di palette `Riscrittura:`, niente combinazione di apertura del pannello, tasti o sezione di aiuto, niente voci «Riscrivi la cronologia» nei menu del Log, niente pulsante «Modifica la cronologia in profondità...» nella finestra Amend, niente «Crea regole di riscrittura con l'IA» né IA per gli script, niente limiti degli script nella pagina Sincronizzazione delle impostazioni e niente blocchi IA di riscrittura della cronologia nelle impostazioni IA. I comandi CLI/MCP `rewrite`, `ai rewrite-script`, `ai rewrite-rules` e `ui propose rewrite` spariscono dall'albero dei comandi. Le identità `.mailmap` (`mailmap`, `log.use_mailmap`) non ne fanno parte e restano. |
| Ricerca globale | Niente pulsante Ricerca globale nella barra del titolo, niente `Ctrl+Alt+F`, niente comando di palette `Cerca: Area di lavoro...` e niente comando CLI/MCP `gitpoulpi search`. La barra di ricerca del Log (`ui show search`) è un'altra funzione e resta. |
| Aree di lavoro | La finestra lavora con il solo elenco di schede predefinito («Nessuna area di lavoro»): niente chip dell'area di lavoro, niente scheda Aree di lavoro, niente `Alt+W` / `Alt+J`, niente comandi di palette `Area di lavoro:`, niente voci «Aggiungi all'area di lavoro» sulle schede, niente comandi CLI/MCP `gitpoulpi workspace`, niente opzione `--workspace` su `commit` e `search`. Schede e sessioni funzionano come sempre. Un `workspaces.toml` scritto da un'altra build non viene né letto né modificato; il profilo predefinito e le istruzioni IA di un'area di lavoro non si applicano. Il Radar e la Ricerca globale coprono le schede aperte. |
| Script di riscrittura (Lua incorporato) | Le regole Script di una riscrittura della cronologia vengono rifiutate con una risposta tipizzata «non incluso in questo binario», e non viene collegato alcun Lua. Il Pannello della cronologia in sé resta (ha una scelta propria). Priva di significato, e segnalata con un avviso, quando il Pannello della cronologia è disattivato. |

I tuoi file di configurazione sono condivisi tra le build: un'impostazione per qualcosa che non è
compilato viene conservata e ignorata.

## Interattivo

Lo script Bash usa liste di controllo `whiptail` o `dialog` (tre schermate, poi un
riepilogo), oppure prompt numerati quando nessuno dei due esiste. Lo script PowerShell usa
`Out-GridView` quando disponibile, altrimenti prompt. Le tue scelte precedenti sono
già spuntate.

## Non interattivo

Qualsiasi argomento salta le domande e va dritto a
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Opzione | Significato |
|---|---|
| `--langs fr,de` / `all` / `en` | Lingue oltre all'inglese. I codici sono i nomi dei file di localizzazione (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Provider: GitHub, GitLab (anche self-managed), Azure DevOps (Services e Server), Bitbucket (Cloud e Data Center) (hosting), Gitea e Forgejo incluso Codeberg (`gitea`) (hosting), Jira, Trello, Linear, YouTrack (Cloud e Server), Redmine (issue tracker), Gerrit Code Review (`gerrit`, solo revisione), issue Markdown locali (`local-issues`, una cartella o un vault Obsidian). Ciascuno di `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` è una feature cargo a sé; dimensioni in [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Famiglie di temi da compilare (predefinito `all`). `--list-themes` stampa le famiglie. |
| `--logo-colors all` / `violet,teal` | Varianti di colore del logo da compilare (predefinito `all`, mantenute nell'ordine della tabella, che è l'ordine di rotazione). `--list-logo-colors` le stampa (`*` = predefinito ufficiale). |
| `--logo-default teal` | La variante predefinita; deve essere tra quelle compilate (predefinito: lavender, altrimenti la prima compilata). |
| `--icon-color sun` | Colore dell'icona dell'eseguibile Windows (predefinito: quello predefinito del logo). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review` | Ciascuna è attiva per impostazione predefinita. |
| `--[no-]history-rewrite` | Il Pannello della cronologia (riscrittura avanzata della cronologia: autori, date, messaggi, contenuto). Attivo per impostazione predefinita. |
| `--[no-]global-search` | Ricerca globale (pulsante nella barra del titolo, `Ctrl+Alt+F`, `gitpoulpi search`). Attiva per impostazione predefinita. |
| `--[no-]workspaces` | Aree di lavoro (gruppi di repository con un nome). Attive per impostazione predefinita; disattivate lasciano solo l'elenco di schede predefinito. |
| `--[no-]rewrite-scripts` | Lua incorporato per le regole Script di una riscrittura. Attivo per impostazione predefinita; ignorato con un avviso se il Pannello della cronologia è disattivato. |
| `--save <name\|file>` | Salva le scelte risolte come selezione salvata (un nome o un percorso di file). |
| `--preset <name\|file>` | Parte da una selezione salvata (un nome o un percorso di file); le altre opzioni la sovrascrivono, ovunque si trovino nella riga. |
| `--list-presets`, `--delete-preset <name>` | Mostra le selezioni salvate e la loro cartella / elimina una per nome. |
| `--last` | Riusa `target/custom-build/last-choices.env`; le altre opzioni lo sovrascrivono. |
| `--full` | Nessuna selezione (tutto attivo). |
| `--profile custom\|dist\|release` | Predefinito `custom` (le impostazioni di `dist`, directory di output separata). |
| `--dry-run` | Stampa l'esatto comando cargo e si ferma. |
| `-- <cargo args>` | Passati a cargo, per esempio `--target x86_64-unknown-linux-musl`. |

Nomi PowerShell: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

La selezione dei temi arriva alla build come variabile d'ambiente
`GITPOULPI_THEMES` (`all`, `none` o un elenco di famiglie separate da virgola), letta dallo
script di build di `gitpoulpi-config`, che elimina le altre palette in fase di compilazione;
impostala tu stesso per un semplice `cargo build` se preferisci. `gitpoulpi
--version` stampa una riga `themes:` con l'insieme compilato.

Le scelte del logo sono le variabili d'ambiente `GITPOULPI_LOGO_COLORS`
(`all` o un elenco di slug separati da virgola), `GITPOULPI_LOGO_DEFAULT` e
`GITPOULPI_ICON_COLOR`, lette dagli script di build di `gitpoulpi-tui` (che
filtra `assets/logo/colourways.json` nella tabella che la finestra Informazioni
incorpora) e dei tre crate binari (l'icona). La fonte dei colori
resta `assets/logo/colourways.json`.

### Icona dell'eseguibile Windows

Quando il target è Windows, gli eseguibili `gitpoulpi`, `gitpoulpi-tui` e `gitpoulpi-mcp`
ricevono il polpo come icona, generata in fase di compilazione (non servono
Python né ImageMagick) nel colore di icona scelto, in dimensioni da 16 a 128 px.
Viene incorporata con il crate `winresource` e richiede il compilatore di risorse
della piattaforma (`rc.exe` dal Windows SDK con MSVC, `windres` con GNU). Se manca,
la build stampa solo un avviso e l'eseguibile mantiene l'icona predefinita. Con la toolchain GNU solo `gitpoulpi` porta l'icona (la risorsa
altrimenti verrebbe collegata due volte). Le build per Linux e macOS ignorano tutto questo
tranne la tabella dei colori.

## Selezioni salvate

Per ricompilare una nuova versione senza rifare tutta la selezione, salvala con un nome e riparti da lì in seguito. Le opzioni date insieme a `--preset` continuano a sovrascrivere i valori salvati.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Le selezioni salvate sono semplici file `CHIAVE=valore` (il formato di `last-choices.env`, leggibile da una shell). Si trovano nella cartella `custom-builds` della directory di configurazione di GitPoulpi, fuori da `target/`, quindi `cargo clean` non le elimina:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` per impostazione predefinita)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` sposta quella cartella insieme alla configurazione. `--preset` e `--save` accettano anche un percorso di file (un valore che contiene `/` o `\`, o che termina con `.env`), per tenere una selezione in un repository di dotfiles.

Una selezione scritta da un'altra versione di GitPoulpi si carica comunque. Una chiave sconosciuta, o una lingua, un'integrazione, un tema o un colore che non esiste più, viene ignorata con un avviso. Una chiave assente dal file (una funzione aggiunta in seguito) prende il valore predefinito, e un'integrazione aggiunta in seguito viene inclusa; entrambe le cose sono segnalate sullo standard error.

Quando esiste una selezione salvata, la modalità interattiva propone prima «Parti da» (le scelte precedenti, una selezione salvata o tutto attivo) e alla fine chiede se salvare la selezione corrente con un nome.

## Cosa fa risparmiare

Misurato su Linux il 2026-10-29 al commit `65999ef5c`, dalla build completa, con la differenza rispetto a essa tra parentesi (dettagli in `docs/developer/binary-size.md`):

| Build (dalla `full` salvo indicazione contraria) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Completa (tutto attivo) | 64,6 | 50,5 | 31,7 |
| Solo inglese | 52,0 (-12,6) | 40,2 (-10,3) | 31,7 (0,0) |
| MCP disattivato | 61,6 (-3,0) | 49,6 (-0,8) | non compilato |
| IA disattivata | 62,6 (-2,0) | 48,6 (-1,8) | 30,5 (-1,3) |
| Tutte le integrazioni disattivate | 56,3 (-8,2) | 43,7 (-6,7) | 26,7 (-5,0) |
| IA + integrazioni + download avatar disattivati | 52,2 (-12,3) | 39,9 (-10,6) | 23,0 (-8,8) |
| Pannello della cronologia disattivato (`--no-history-rewrite`) | 61,6 (-3,0) | 48,9 (-1,5) | 28,5 (-3,2) |
| Script di riscrittura disattivati | 63,1 (-1,5) | 49,0 (-1,5) | 31,1 (-0,7) |
| Ricerca globale disattivata | 63,5 (-1,1) | 49,5 (-1,0) | 31,6 (-0,1) |
| Aree di lavoro disattivate | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,2) |
| Radar disattivato | 63,4 (-1,2) | 49,3 (-1,2) | 31,7 (0,0) |
| Board disattivata | 63,6 (-1,0) | 49,5 (-1,0) | 31,7 (0,0) |
| Review disattivata | 63,2 (-1,4) | 49,0 (-1,4) | 31,7 (0,0) |
| Mermaid disattivato | 63,6 (-1,0) | 49,5 (-0,9) | 31,6 (-0,1) |
| LFS disattivato | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,3) |
| Worktree disattivato | 63,7 (-0,9) | 49,6 (-0,8) | 31,7 (0,0) |
| Sottomoduli disattivati | 63,7 (-0,9) | 49,6 (-0,9) | 31,6 (-0,1) |
| Download avatar disattivato | 63,7 (-0,9) | 49,6 (-0,9) | 31,7 (0,0) |
| Anteprima immagini disattivata | 63,3 (-1,2) | 49,2 (-1,3) | 31,7 (0,0) |
| Tutto disattivato, solo inglese (LFS mantenuto) | 35,2 (-29,4) | 28,9 (-21,5) | non compilato |
| **Tutto disattivato, solo inglese** | **34,9 (-29,7)** | **28,8 (-21,6)** | non compilato |

## Note

- Una build personalizzata è lenta come ogni build `dist` (LTO completo, qualche minuto).
- Funziona senza sccache, come `build-opti`.
- I codici lingua vengono verificati rispetto alle localizzazioni fornite; un codice sconosciuto viene
  rifiutato con l'elenco di quelli validi.
