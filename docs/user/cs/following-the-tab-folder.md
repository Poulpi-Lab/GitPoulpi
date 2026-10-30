[English](../following-the-tab-folder.md) · **Čeština** · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Sledování složky aktivní karty

GitPoulpi může mít otevřeno několik repozitářů jako karty. Shell, ze kterého jste jej spustili, zůstává ve své složce: program nemůže změnit složku svého rodičovského shellu. GitPoulpi nabízí dva způsoby, jak získat shell ve složce aktivní karty:

- **Konec a cd sem**: GitPoulpi se ukončí a váš shell přejde do složky aktivní karty. K tomu je potřeba malá funkce shellu, kterou vypíše `gitpoulpi shell-init`.
- **Otevřít v**: otevře složku aktivní karty v terminálu, panelu multiplexeru, editoru nebo správci souborů. Není třeba nic nastavovat v shellu. Viz [Otevřít v](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Otevřít v ▾ ── ⌕ Globální hledání ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Konec a cd sem           │
```

- **Otevřít v** je v záhlaví okna, za cestou k repozitáři. Je-li okno příliš úzké, skryje se.
- **Konec a cd sem** je na pravém konci řádku karet. Zobrazí se jen tehdy, když platí obojí:
  - GitPoulpi byl spuštěn pomocí funkce shellu (viz níže).
  - Aktivní karta není složka, ve které jste GitPoulpi spustili.
- Příkazy jsou vždy v paletě (`Ctrl+P`): `Karta: Ukončit a cd sem`, `Karta: Otevřít v…` (`Alt+O`) a `Karta: Otevřít ve výchozí položce` (`Ctrl+Alt+O`). Bez funkce shellu vám `Karta: Ukončit a cd sem` poradí, jak ji nastavit.

Běžné ukončení (`q`) nikdy nezmění složku vašeho shellu. Dělá to jen **Konec a cd sem**, pokud nezvolíte `auto` (viz níže).

## Nastavení funkce shellu

`gitpoulpi shell-init <shell>` vypíše funkci s názvem `gp`. Jiný název zvolíte pomocí `--name`. Funkce spustí `gitpoulpi --cwd-file <temp file> ...`. Když ukončíte pomocí **Konec a cd sem**, GitPoulpi zapíše složku do onoho souboru a funkce do ní přejde. GitPoulpi nic neinstaluje ani neupravuje: do spouštěcího souboru svého shellu přidáte jeden řádek.

| Shell | Přidejte do | Řádek |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | váš `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Poté spouštějte GitPoulpi příkazem `gp` místo `gitpoulpi`. Argumenty se předávají dál (`gp -s work`) a návratový kód je návratový kód `gitpoulpi`.

**Existující `gp` se nikdy nepřepíše bez varování.** Pokud váš shell už má příkaz, alias nebo funkci s názvem `gp`, skript vypíše varování a nic nedefinuje. Zvolte jiný název, například `gitpoulpi shell-init bash --name gpl`. V PowerShellu funkce nahradí vestavěný alias `gp` (`Get-ItemProperty`) a alias, který ukazuje na `gitpoulpi`. Jakýkoli jiný `gp` se ponechá, s varováním.

Každý příkaz `gitpoulpi` přijímá `--cwd-file`, ale soubor zapisuje jen TUI.

## Nastavení: Sledovat složku aktivní karty

Nastavení > Otevřít v > **Sledovat složku aktivní karty** (`ui.tab_dir_follow`):

| Hodnota | Chování |
|---|---|
| `buttons` (výchozí) | **Konec a cd sem** a **Otevřít v**, jak je popsáno výše. |
| `auto` | Bez tlačítka **Konec a cd sem**. **Otevřít v** zůstává. Každé ukončení přepne váš shell do složky aktivní karty (pokud jste GitPoulpi spustili pomocí funkce shellu). Výchozí položka **Otevřít v** se spustí pokaždé, když otevřete novou kartu. |

V režimu `auto` se výchozí položka spustí jen pro karty, které otevřete sami: z karty Domů, z worktree, z klonu nebo z init. Nespustí se pro karty obnovené při startu, pro první kartu ani při přepnutí na již otevřenou kartu.

Starší nastavení `ui.terminal_launch` a `ui.terminal_launch_command` byla přesunuta do seznamu **Otevřít v**. Viz [Otevřít v](open-in.md#starší-klíče).
