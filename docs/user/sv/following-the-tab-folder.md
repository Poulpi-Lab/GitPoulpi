[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · **Svenska** · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Följa den aktiva flikens mapp

GitPoulpi kan ha flera repositories öppna som flikar. Skalet du startade det
från stannar i sin mapp: ett program kan inte byta mapp för sitt överordnade
skal. GitPoulpi erbjuder två sätt att få ett skal i den aktiva flikens mapp:

- **Avsluta, cd hit**: GitPoulpi avslutas, och ditt skal byter till den aktiva
  flikens mapp. Det här kräver en liten skalfunktion, utskriven av
  `gitpoulpi shell-init`.
- **Öppna i**: öppnar den aktiva flikens mapp i en terminal, en
  multiplexerruta, en redigerare eller filhanteraren. Ingen skalinstallation
  behövs. Se [Öppna i](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Öppna i ▾ ── ⌕ Global sökning ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Avsluta, cd hit          │
```

- **Öppna i** finns i titelraden, efter repositoryts sökväg. Den döljs när
  fönstret är för smalt.
- **Avsluta, cd hit** finns längst till höger i flikraden. Den visas bara när
  båda stämmer:
  - GitPoulpi startades via skalfunktionen (nedan).
  - Den aktiva fliken är inte mappen du startade GitPoulpi i.
- Kommandona finns alltid i paletten (`Ctrl+P`): `Flik: Avsluta och cd hit`,
  `Flik: Öppna i...` (`Alt+O`) och `Flik: Öppna i standard` (`Ctrl+Alt+O`).
  Utan skalfunktionen berättar `Flik: Avsluta och cd hit` hur du ställer in den.

Ett vanligt avslut (`q`) ändrar aldrig mappen för ditt skal. Bara
**Avsluta, cd hit** gör det, om du inte väljer `auto` (nedan).

## Ställa in skalfunktionen

`gitpoulpi shell-init <shell>` skriver ut en funktion som heter `gp`. Använd
`--name` för att välja ett annat namn. Funktionen kör
`gitpoulpi --cwd-file <tempfil> ...`. När du avslutar med **Avsluta, cd hit**
skriver GitPoulpi mappen i den filen, och funktionen byter till den mappen.
GitPoulpi installerar eller redigerar ingenting: du lägger till en rad i ditt
skals startfil.

| Skal | Lägg till i | Rad |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | din `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Starta sedan GitPoulpi med `gp` i stället för `gitpoulpi`. Argument skickas
vidare (`gp -s work`), och slutstatusen är densamma som `gitpoulpis`.

**Ett befintligt `gp` ersätts aldrig utan varning.** Om ditt skal redan har ett
kommando, alias eller en funktion som heter `gp` skriver skriptet ut en varning
och definierar ingenting. Välj ett annat namn, till exempel
`gitpoulpi shell-init bash --name gpl`. I PowerShell ersätter funktionen det
inbyggda aliaset `gp` (`Get-ItemProperty`) och ett alias som pekar på
`gitpoulpi`. Varje annat `gp` behålls, med en varning.

Varje `gitpoulpi`-kommando accepterar `--cwd-file`, men bara TUI:n skriver
filen.

## Inställning: Följ aktiv fliks mapp

Inställningar > Öppna i > **Följ aktiv fliks mapp** (`ui.tab_dir_follow`):

| Värde | Beteende |
|---|---|
| `buttons` (standard) | **Avsluta, cd hit** och **Öppna i** som beskrivet ovan. |
| `auto` | Ingen knapp **Avsluta, cd hit**. **Öppna i** finns kvar. Varje avslut byter ditt skal till den aktiva flikens mapp (när du startade GitPoulpi med skalfunktionen). Standardposten i **Öppna i** körs varje gång du öppnar en ny flik. |

I läget `auto` körs standardposten bara för flikar som du själv öppnar: från
fliken Hem, en worktree, en klon eller en init. Den körs inte för flikarna som
återställs vid start, för den första fliken, eller när du byter till en flik som
redan är öppen.

De äldre inställningarna `ui.terminal_launch` och `ui.terminal_launch_command`
har flyttats till listan **Öppna i**. Se
[Öppna i](open-in.md#äldre-nycklar).
