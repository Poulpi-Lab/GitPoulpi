[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · **Nederlands** · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# De map van het actieve tabblad volgen

In GitPoulpi kunnen meerdere repositories als tabbladen openstaan. De shell
waarmee je het hebt gestart blijft in zijn map: een programma kan de map van
zijn bovenliggende shell niet wijzigen. GitPoulpi biedt twee manieren om een
shell in de map van het actieve tabblad te krijgen:

- **Afsluiten & cd hier**: GitPoulpi sluit af en je shell gaat naar de map van
  het actieve tabblad. Daarvoor is een kleine shellfunctie nodig, afgedrukt door
  `gitpoulpi shell-init`.
- **Openen in**: opent de map van het actieve tabblad in een terminal, een
  multiplexerpaneel, een editor of het bestandsbeheer. Geen shell-instelling
  nodig. Zie [Openen in](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Openen in ▾ ── ⌕ Globaal zoeken ⚙ × ┐
│ ⌂  main-repo x  other-repo x                Afsluiten & cd hier         │
```

- **Openen in** staat in de titelbalk, na het pad van de repository. Hij is
  verborgen als het venster te smal is.
- **Afsluiten & cd hier** staat aan het rechteruiteinde van de tabbalk. Hij
  verschijnt alleen als beide waar zijn:
  - GitPoulpi is gestart via de shellfunctie (hieronder).
  - Het actieve tabblad is niet de map waarin je GitPoulpi hebt gestart.
- De opdrachten staan altijd in het palet (`Ctrl+P`):
  `Tabblad: Afsluiten en cd hier`, `Tabblad: Openen in...` (`Alt+O`) en
  `Tabblad: Openen in standaard` (`Ctrl+Alt+O`). Zonder de shellfunctie vertelt
  `Tabblad: Afsluiten en cd hier` je hoe je die instelt.

Een normale afsluiting (`q`) wijzigt nooit de map van je shell. Alleen
**Afsluiten & cd hier** doet dat, tenzij je `auto` kiest (hieronder).

## De shellfunctie instellen

`gitpoulpi shell-init <shell>` drukt een functie met de naam `gp` af. Gebruik
`--name` om een andere naam te kiezen. De functie voert
`gitpoulpi --cwd-file <tijdelijk bestand> ...` uit. Als je afsluit met **Afsluiten &
cd hier**, schrijft GitPoulpi de map in dat bestand en gaat de functie naar die
map. GitPoulpi installeert of bewerkt niets: jij voegt één regel toe aan het
opstartbestand van je shell.

| Shell | Voeg dit toe aan | Regel |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | je `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Start GitPoulpi daarna met `gp` in plaats van `gitpoulpi`. Argumenten worden
doorgegeven (`gp -s work`) en de afsluitstatus is die van `gitpoulpi`.

**Een bestaande `gp` wordt nooit zonder waarschuwing vervangen.** Heeft je shell
al een opdracht, alias of functie met de naam `gp`, dan drukt het script een
waarschuwing af en definieert niets. Kies een andere naam, bijvoorbeeld
`gitpoulpi shell-init bash --name gpl`. In PowerShell vervangt de functie de
ingebouwde alias `gp` (`Get-ItemProperty`) en een alias die naar `gitpoulpi`
wijst. Elke andere `gp` blijft behouden, met een waarschuwing.

Elke `gitpoulpi`-opdracht accepteert `--cwd-file`, maar alleen de TUI schrijft
het bestand.

## Instelling: De map van het actieve tabblad volgen

Instellingen > Openen in > **De map van het actieve tabblad volgen**
(`ui.tab_dir_follow`):

| Waarde | Gedrag |
|---|---|
| `buttons` (standaard) | **Afsluiten & cd hier** en **Openen in** zoals hierboven beschreven. |
| `auto` | Geen knop **Afsluiten & cd hier**. **Openen in** blijft. Elke afsluiting zet je shell in de map van het actieve tabblad (als je GitPoulpi met de shellfunctie hebt gestart). Het standaarditem van **Openen in** wordt uitgevoerd telkens als je een nieuw tabblad opent. |

In de modus `auto` wordt het standaarditem alleen uitgevoerd voor tabbladen die
je zelf opent: vanuit het tabblad Start, een worktree, een kloon of een init. Het
wordt niet uitgevoerd voor de tabbladen die bij het opstarten worden hersteld,
voor het eerste tabblad, of als je overschakelt naar een tabblad dat al open is.

De oudere instellingen `ui.terminal_launch` en `ui.terminal_launch_command`
zijn verplaatst naar de lijst **Openen in**. Zie
[Openen in](open-in.md#verouderde-sleutels).
