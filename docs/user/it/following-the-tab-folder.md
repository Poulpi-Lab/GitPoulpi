[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · **Italiano** · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Seguire la cartella della scheda attiva

GitPoulpi può avere più repository aperti come schede. La shell da cui l'hai
avviato resta nella sua cartella: un programma non può cambiare la cartella
della shell che lo ha lanciato. GitPoulpi offre due modi per ottenere una shell
nella cartella della scheda attiva:

- **Esci e cd qui**: GitPoulpi esce e la tua shell passa alla cartella della
  scheda attiva. Serve una piccola funzione di shell, stampata da
  `gitpoulpi shell-init`.
- **Apri in**: apre la cartella della scheda attiva in un terminale, in un
  riquadro di un multiplexer, in un editor o nel gestore file. Non serve
  configurare la shell. Vedi [Apri in](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ Ricerca globale ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **Apri in** è nella barra del titolo, dopo il percorso del repository. Viene
  nascosto quando la finestra è troppo stretta.
- **Esci e cd qui** è all'estremità destra della barra delle schede. Compare
  solo quando valgono entrambe queste condizioni:
  - GitPoulpi è stato avviato tramite la funzione di shell (qui sotto).
  - La scheda attiva non è la cartella in cui hai avviato GitPoulpi.
- I comandi sono sempre nella palette (`Ctrl+P`): `Scheda: Esci e cd qui`,
  `Scheda: Apri in...` (`Alt+O`) e `Scheda: Apri nella voce predefinita`
  (`Ctrl+Alt+O`). Senza la funzione di shell, `Scheda: Esci e cd qui` ti dice
  come configurarla.

Un'uscita normale (`q`) non cambia mai la cartella della tua shell. Lo fa solo
**Esci e cd qui**, a meno che tu scelga `auto` (più avanti).

## Configurare la funzione di shell

`gitpoulpi shell-init <shell>` stampa una funzione chiamata `gp`. Usa `--name`
per sceglierne un altro nome. La funzione esegue
`gitpoulpi --cwd-file <temp file> ...`. Quando esci con **Esci e cd qui**,
GitPoulpi scrive la cartella in quel file e la funzione passa a quella cartella.
GitPoulpi non installa né modifica nulla: aggiungi tu una riga al file di
avvio della tua shell.

| Shell | Aggiungi a | Riga |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | il tuo `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Poi avvia GitPoulpi con `gp` invece di `gitpoulpi`. Gli argomenti vengono
passati (`gp -s work`) e lo stato di uscita è quello di `gitpoulpi`.

**Un `gp` esistente non viene mai sostituito senza avviso.** Se la tua shell ha
già un comando, un alias o una funzione chiamata `gp`, lo script stampa un
avviso e non definisce nulla. Scegli un altro nome, per esempio
`gitpoulpi shell-init bash --name gpl`. In PowerShell la funzione sostituisce
l'alias integrato `gp` (`Get-ItemProperty`) e un alias che punta a
`gitpoulpi`. Ogni altro `gp` viene mantenuto, con un avviso.

Ogni comando `gitpoulpi` accetta `--cwd-file`, ma solo la TUI scrive il file.

## Impostazione: Segui la cartella della scheda attiva

Impostazioni > Apri in > **Segui la cartella della scheda attiva**
(`ui.tab_dir_follow`):

| Valore | Comportamento |
|---|---|
| `buttons` (predefinito) | **Esci e cd qui** e **Apri in** come descritto sopra. |
| `auto` | Nessun pulsante **Esci e cd qui**. **Apri in** resta. Ogni uscita porta la tua shell nella cartella della scheda attiva (se hai avviato GitPoulpi con la funzione di shell). La voce predefinita di **Apri in** viene eseguita ogni volta che apri una nuova scheda. |

In modalità `auto`, la voce predefinita viene eseguita solo per le schede che
apri tu: dalla scheda Home, da un worktree, da un clone o da un init. Non viene
eseguita per le schede ripristinate all'avvio, per la prima scheda, né quando
passi a una scheda già aperta.

Le vecchie impostazioni `ui.terminal_launch` e `ui.terminal_launch_command`
sono state spostate nell'elenco **Apri in**. Vedi
[Apri in](open-in.md#chiavi-obsolete).
