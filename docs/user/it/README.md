[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · **Italiano** · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Documentazione utente di GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="Il polpo di GitPoulpi" width="128"></p>

GitPoulpi è un client Git per il terminale con un grafo dei commit cliccabile,
una palette dei comandi, l'integrazione con hosting e issue tracker, un aiuto
IA facoltativo, e una CLI e un server MCP che fanno tutto ciò che fa la UI.
Queste pagine sono in ordine di lettura: parti dall'inizio e passa alle pagine
di riferimento quando ti servono.

![La finestra principale di GitPoulpi](../images/hero.png)

## Da dove cominciare

1. **[Installazione](installation.md)**: cosa serve, archivi di release,
   gestori di pacchetti, compilazione dai sorgenti. Per un binario più piccolo,
   vedi **[Build personalizzate](custom-builds.md)**. Come GitPoulpi si tiene
   aggiornato (il chip della versione, il pannello degli aggiornamenti, il
   ritorno alla versione precedente): **[Aggiornamenti](updates.md)**. Per
   eseguirlo in un container: **[Eseguire in Docker](docker.md)**. Per gli
   amministratori, feed interni, mirror e la policy di aggiornamento:
   **[Distribuzione aziendale](enterprise-distribution.md)**.
2. **[Per iniziare](getting-started.md)**: i tuoi primi cinque minuti, dall'avvio
   al primo commit.
3. **[Tour dell'interfaccia](interface-tour.md)**: il nome di ogni area della
   finestra, con le immagini.

## Trova una funzione

| Voglio... | Pagina | Come raggiungerla nell'app |
|---|---|---|
| Lavorare su due branch insieme | [Worktrees](worktrees.md) | La sezione **WORKTREES** della barra laterale Refs (`+` ne crea uno; nascosta finché non ne esiste uno: palette **Barra laterale: Mostra la sezione Worktrees**); palette **Refs: Crea un worktree...** |
| Lavorare con repository annidati | [Sottomoduli](submodules.md) | La sezione **SOTTOMODULI** della barra laterale Refs; palette **Git: Aggiorna tutti i sottomoduli**, **Refs: Aggiorna il sottomodulo (selezionato)** |
| Risolvere un conflitto di merge | [Risolvere i conflitti di merge](merge-conflicts.md) | `Enter` su una riga `!` del pannello Stato, il pulsante **Risolvi i conflitti...** del banner, oppure `Shift+E` nella vista Diff; palette **Conflitti: Apri l'editor di merge** |
| Rivedere una pull o merge request | [Rivedere una pull request](review.md) | `Enter` su una riga della sezione **PULL REQUEST**, `e` su una riga del Radar, `Alt+R` nella ricerca dell'area di lavoro; palette **Hosting: Rivedi la richiesta...** |
| Sfogliare, creare e gestire pull request e issue | [Pull request e issue](pull-requests-and-issues.md) | Le sezioni **PULL REQUEST** e **ISSUE** della barra laterale Refs (`n` ne crea una); palette **Pull request: Crea pull request...**, **Hosting: Nuova issue...**. Prima collega un account: [Integrazioni di hosting e issue tracker](hosting-integration.md) |
| Leggere e rispondere alla discussione di una issue o di una richiesta | [Conversazioni](conversations.md) | `Enter` su una riga di issue, su una card della bacheca o su una riga del Radar; palette **Hosting: Apri la conversazione di una issue...** |
| Vedere le mie issue come una bacheca | [La bacheca delle issue](pull-requests-and-issues.md#la-bacheca-delle-issue) | Il pulsante **Bacheca** dell'intestazione ISSUE; palette **Issue: Apri la bacheca** |
| Tenere le issue come note Markdown, senza token | [Issue locali e Obsidian](local-issues.md) | La sezione ISSUE, provider `Local Markdown - <nome>`; palette **Issue locali: Apri in Obsidian...** |
| Riscrivere autori, date, messaggi o file su molti commit | [Riscrittura della cronologia](history-rewrite.md) | `Ctrl+G Shift+H`; palette **Riscrittura: Pannello della cronologia** |
| Annullare (o ripetere) un'operazione | [Annulla e ripeti](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` apre la cronologia degli annullamenti; palette **Modifica: Annulla** |
| Tracciare i file grandi con Git LFS | [Git LFS](lfs.md) | Palette **LFS: Traccia un pattern...**, **LFS: Inizializza per questo repository** |
| Trovare un branch, un commit, una pull request o una issue tra più repository | [Ricerca](search.md) | `Ctrl+Alt+F`, oppure il pulsante Ricerca globale nella barra del titolo; `f` cerca nel grafo dei commit del repository corrente |
| Vedere in un unico elenco tutto ciò che mi riguarda | [Il Radar](radar.md) | `Alt+R`; palette **Radar: Apri** |
| Raggruppare i repository e fare fetch o pull insieme | [Aree di lavoro](workspaces.md) | `Alt+W` cambia, `Alt+J` apre la scheda Aree di lavoro; palette **Area di lavoro: Gestisci...** |
| Avere aiuto dall'IA (messaggi di commit, spiegazioni, descrizioni) | [Assistenza IA](ai.md) | `Ctrl+,` > **IA** > **Attiva l'IA**, poi `Ctrl+G i` scrive un messaggio di commit; palette **IA: ...** |
| Trovare qualsiasi comando o tasto | [Scorciatoie da tastiera](keybindings.md) | `?` nell'app elenca i tasti, `Ctrl+P` trova qualsiasi comando |

## Uso quotidiano

4. **[Lavorare con il Log](log-and-history.md)**: leggere il grafo, ricerca,
   branch, tag, stash, trascinare i ref, rebase, cherry-pick,
   reword e amend, banner, pull e push, auto-stash.
5. **[Stage e commit](staging-and-committing.md)**: le modifiche in corso,
   diff, stage di hunk e singole righe, scrivere i commit, il compositore di
   commit, blame, cronologia dei file e l'editor integrato.
6. **[Immagini](image-view.md)**: le immagini nella vista Diff e nella vista
   File, prima e dopo, zoom, Modifica in...
7. **[Risolvere i conflitti di merge](merge-conflicts.md)**: l'editor di merge
   (A e B affiancati sopra il risultato), scegliere righe e blocchi, schede per
   i conflitti di file binari, LFS, eliminati, symlink e sottomoduli, e come un
   agente propone una risoluzione.
8. **[Worktrees](worktrees.md)**: una seconda cartella in checkout su un altro branch:
   la sezione WORKTREES, creare, aprire, fare il merge e rimuovere i worktree
   collegati.
9. **[Sottomoduli](submodules.md)**: la sezione SOTTOMODULI, aggiornarli e
   mantenerli allineati con checkout e pull.
10. **[Annulla e ripeti](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, cosa si può e non
   si può annullare, quando rifiuta o chiede conferma, fin dove arriva,
   come disattivarlo.
11. **[Git LFS](lfs.md)**: file grandi, tracciamento, blocchi, e cosa succede
   senza `git-lfs` installato.
12. **[Temi](themes.md)**: i temi integrati, qualsiasi tema con qualsiasi
    palette delle corsie, anteprima dal vivo, i tuoi file di tema,
    `gitpoulpi theme`.
13. **[Aree di lavoro](workspaces.md)**: gruppi di repository con nome, con
    schede proprie e un profilo di commit predefinito facoltativo, il chip nella
    barra delle schede, la scheda Aree di lavoro, Fetch All,
    Pull All, Clona i mancanti.
14. **[Ricerca](search.md)**: un unico cercatore fuzzy (`Ctrl+Alt+F`) tra i branch,
    i tag, i worktree, gli stash, i commit, le pull/merge request e le issue di
    ogni repository dell'area di lavoro o delle schede aperte.
15. **[Il Radar](radar.md)**: le pull e merge request, le issue e il lavoro
    in corso di un'area di lavoro (o delle tue schede aperte) in un unico elenco:
    filtri, pin, rinvii, e cosa può fare ogni riga.
16. **[Pull request e issue](pull-requests-and-issues.md)**: le sezioni PULL
    REQUEST e ISSUE della barra laterale, la bacheca delle issue, creare una
    pull request o una issue, il chip della barra degli strumenti, i riferimenti
    nei messaggi di commit.
17. **[Conversazioni](conversations.md)**: leggere e rispondere alla
    discussione di una issue o di una richiesta dentro la finestra, pubblicare
    come, cosa non sa fare ogni host.
18. **[Rivedere una pull request](review.md)**: la vista di revisione con
    discussioni sui commenti, una revisione in sospeso e Approva / Richiedi modifiche.
19. **[Apri in](open-in.md)**: l'elenco nella barra del titolo che apre la
    cartella della scheda attiva in un terminale, un multiplexer, un editor o
    un gestore file.
20. **[Seguire la cartella della scheda attiva](following-the-tab-folder.md)**:
    «Esci e cd qui», e il wrapper `gitpoulpi shell-init` (`gp`) per bash, zsh,
    fish e PowerShell.
    Quale terminale mostra meglio GitPoulpi: **[Scegliere un terminale](terminals.md)**.
    Cosa passa attraverso tmux, zellij, screen, herdr e i pannelli dei terminali:
    **[Multiplexer](multiplexers.md)**.

Inoltre: **[Più remoti per un branch](multiple-remotes.md)**: destinatari di push e origini di
pull, il dialogo Remoti, Push su tutti i remoti o su remoti scelti, risolvere un remoto che si è
mosso, Continua dopo un conflitto, la CLI e un alias di git semplice.

**[Workflow di branching](branching-workflows.md)**, i modelli Git Flow, GitHub Flow, GitLab Flow e
Trunk-based oppure i tuoi branch base, pattern dei nomi dei branch e tipi di topic (`gitpoulpi flow ...`).

**[Riscrittura della cronologia](history-rewrite.md)**: il pannello di riscrittura
(`Ctrl+G Shift+H`) che riscrive autori, date, messaggi e contenuto dei file su molti
commit, con anteprima, script Lua e un solo annulla.

## Tastiera e mouse

21. **[Scorciatoie da tastiera](keybindings.md)**: ogni tasto, per area, e ogni
    comando della palette. Nell'app, `?` mostra lo stesso elenco e `Ctrl+P`
    trova qualsiasi comando.
22. **[Mouse e gesti](mouse-and-gestures.md)**: clic, trascinamento,
    selezione e copia, barre di scorrimento e feedback al passaggio del mouse.

## Impostazioni e integrazioni

23. **[Impostazioni](settings.md)**: ogni pagina e opzione della schermata
    Impostazioni, la casella di ricerca, i profili, e come riassegnare le
    scorciatoie.
24. **[Assistenza IA](ai.md)**: l'IA di GitPoulpi (disattivata per impostazione
    predefinita): un agente locale come `claude -p`, Anthropic, OpenAI,
    OpenRouter, Ollama o LM Studio; istruzioni; messaggi di commit, spiegazioni,
    descrizioni delle pull request e il compositore di commit; privacy e
    risoluzione dei problemi.
25. **[Integrazioni di hosting e issue tracker](hosting-integration.md)**: GitHub, GitLab,
    Gitea e Forgejo, Azure DevOps, Bitbucket, Gerrit e gli issue tracker: token e
    relativi scope, connessione, collegare un repository, chiavi SSH, limiti noti.
    Anche **[Issue locali e Obsidian](local-issues.md)**: una cartella di note
    Markdown come sorgente delle issue, senza token, con una Base di Obsidian.

## Automazione

26. **[Riferimento della CLI](cli-reference.md)**: ogni comando `gitpoulpi`, con
    `--json` per gli script.
27. **[MCP e agenti](mcp-and-agents.md)**: lascia che un agente esterno (Claude
    Code, Cursor, Codex, ...) piloti GitPoulpi: registrare il server,
    `.mcp.json`, gli interruttori dei permessi e i loro valori predefiniti, ogni
    strumento, e come un agente ti mostra le cose nella tua finestra. La skill
    per agenti già pronta è
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Riferimento e aiuto

28. **[Riferimento della configurazione](config-reference.md)**: ogni chiave di
    `config.toml`, il suo valore predefinito e il suo effetto, le impostazioni
    per repository, le variabili d'ambiente.
29. **[Risoluzione dei problemi e FAQ](troubleshooting.md)**: icone, resa del
    grafo, tasti che non arrivano, demoni bloccati, velocità, puntatori LFS.

## Schermate

Ogni immagine in queste pagine è una vera cattura di una build di release, su
un piccolo repository dimostrativo con autori inventati. Le immagini sono in
[`images/`](../images/). `scripts/docs-screenshots.sh` crea i repository
dimostrativi e le cattura di nuovo. Sono fatte nel terminale `kitty`, che
disegna il grafo dei commit e gli avatar degli autori come immagini. I
terminali senza un protocollo grafico mostrano invece un grafo a caratteri.

## Per chi contribuisce

[CLAUDE.md](../../../CLAUDE.md), la [panoramica dell'architettura](../../architecture/overview.md)
e la [roadmap](../../roadmap.md) descrivono il design e ciò che è previsto;
[CHANGELOG.md](../../../CHANGELOG.md) elenca cosa è cambiato in ogni release.

---

GitKraken è un marchio del rispettivo proprietario. GitPoulpi è un progetto indipendente, non affiliato, né approvato o sponsorizzato da esso; il nome è citato solo per descrivere compatibilità e ispirazione.

Tutti gli altri nomi di prodotti e servizi citati in GitPoulpi e nella sua documentazione sono marchi dei rispettivi proprietari, ad esempio gli host di codice (GitHub, GitLab, Bitbucket, ecc.), i sistemi di gestione dei ticket (Jira, Trello, Linear, ecc.) e gli editor e gli strumenti che GitPoulpi può aprire (Visual Studio Code, gli IDE JetBrains, lazygit, ecc.). Sono usati solo per indicare ciò con cui GitPoulpi lavora; GitPoulpi non è affiliato, né approvato o sponsorizzato da nessuno di essi.
