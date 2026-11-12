[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · **Svenska** · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Öppna i

Knappen **Öppna i ▾** i titelraden, bredvid repositorysökvägen, öppnar den aktiva
flikens mapp någon annanstans: din terminal, en ny multiplexerflik eller ny
ruta, en redigerare, filhanteraren. Du väljer vad som står i listan. Inget är
hårdkodat: varje post är en etikett och ett kommando.

## Använda den

- **Klicka på Öppna i ▾**: en lista fälls ned. Klicka på en post för att köra
  den. Standardposten är markerad med `•`; den sista raden, **Konfigurera...**,
  öppnar [Inställningar](settings.md#öppna-i) på sidan Öppna i.

  ![Listan Öppna i: Terminal, två redigerare, ett Git-verktyg och filhanteraren, standarden markerad med en punkt, och Konfigurera... sist](../images/open-in-menu.png)

- **Tangentbord**: `Alt+O` öppnar samma lista (`Up`/`Down`, `Enter`, `Esc`; med
  fler än sju poster skriver du för att söka). `Ctrl+Alt+O` kör standardposten
  direkt. Palett (`Ctrl+P`): `Flik: Öppna i...` och `Flik: Öppna i standard`.
  Palettkommandona fungerar även när titelraden inte har plats för knappen.
  Båda tangenterna går att binda om; se [Kortkommandon](keybindings.md).
- **Högerklicka på en repositoryflik**: flikmenyn har en undermeny **Öppna i**
  med samma poster.
- Listan börjar på posten du använde senast i den här sessionen.
- Statusraden säger ”öppnar i ...” direkt. Om programmet misslyckas inom ungefär
  1,5 sekunder (till exempel tmux utan server, eller en felaktig flagga) visas
  första raden av dess felutdata på statusraden. Ett program som inte är
  installerat namnges direkt.

Tills du sparar din egen lista väljs standardposten utifrån det som upptäcks: i
herdr ”Herdr: new tab”; på Omarchy kittys nya fönster (kitty är dess terminal);
annars filhanteraren.

Knappen visas på varje repositoryflik, även mappen GitPoulpi startades i, och i
läget `ui.tab_dir_follow = "auto"`. I läget `auto` körs standardposten också av
sig själv när du öppnar en ny flik (inte för flikarna som återställs vid
start). Se [Följa den aktiva flikens mapp](following-the-tab-folder.md).

## Hur en post ser ut

I `config.toml`, en `[[ui.open_in]]`-tabell per post:

```toml
[ui]
open_in_default = "Herdr: new tab"   # en etikett; tomt betyder första posten

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Du skriver sällan det här för hand: sidan i Inställningar redigerar det. Varje
nyckel finns i [konfigurationsreferensen](config-reference.md).

### Kommandon och variabler

Kommandot delas först upp i argument (mellanslag, `'...'` och `"..."` grupperar
ord), därefter ersätts variablerna inuti varje argument. En mapp med ett
mellanslag i namnet förblir därför ett argument. **Inget skal är inblandat** om
inte ditt kommando startar ett.

| Variabel | Ersätts av |
|---|---|
| `${cwd}` | Flikens mapp. |
| `${repo}` | Repositoryts rot (skiljer sig från `${cwd}` för en worktree-flik). |
| `${name}` | Mappens basnamn. |
| `${branch}` | Den utcheckade grenen; tom när HEAD är frånkopplad. |
| `${web_url}` | Origin-remotens webbsida när den är GitHub eller GitLab; annars tom. |
| `${env:NAME}` | Miljövariabeln `NAME`; tom när den inte är satt. |

Den gamla `{repo}` fungerar fortfarande och betyder `${cwd}`. Varje annan
`${...}` är ett fel som namnger variabeln.

Programmet startas med flikens mapp som arbetskatalog och får tre
miljövariabler, praktiska för ett skalsnutt:

| Variabel | Värde |
|---|---|
| `GITPOULPI_CWD` | Flikens mapp. |
| `GITPOULPI_REPO` | Repositoryts rot. |
| `GITPOULPI_BRANCH` | Grenen, eller tom. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

På Windows slås programmet upp via `PATH` och `PATHEXT`, så `code` och `cursor`
(`.cmd`-filer) fungerar, och inget konsolfönster blinkar förbi.

## Förinställningar

Inställningar > Öppna i > **Lägg till förinställning** listar färdiga poster bara
för ditt operativsystem (Windows-förinställningar visas aldrig på Linux, och så
vidare), per kategori: raden börjar med kategorin, `Explorer: ...`,
`Multiplexer: ...`, `Terminal: ...`, `Editor: ...`, `Git tool: ...`,
`AI agent: ...`, `Web: ...`. Högst upp finns **Identifierad**, posterna i den
beräknade standardlistan. Inom varje kategori kommer först de förinställningar du
kan använda direkt (du kör i den multiplexern eller terminalen och programmet
finns på `PATH`), sedan de installerade, sedan resten. Skriv i kombinationsrutan
för att söka, till exempel `lazygit` eller `Git tool:`. **Testa på aktuell flik**
provar en post innan du förlitar dig på den.

![Listan + Lägg till på sidan Öppna i: identifierade poster först, sedan Utforskaren, Multiplexer och de andra kategorierna, varje förinställning med sin kategori som prefix](../images/settings-open-in-presets.png)

Förinställningar av typen ”kör ett verktyg i en ruta” (`lazygit in Herdr tab`,
`yazi in tmux window`, `Claude Code in Herdr split`...) erbjuds bara när värden
(herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows Terminal) och
verktyget är installerade. `$EDITOR in ...` kör redigeraren som din
`EDITOR`-variabel namnger (med `vi` som reserv i herdr) i herdr eller tmux.

Herdrs verktygsförinställningar skapar fliken eller delningen och skriver sedan
in kommandot i dess skal, så verktyget startar i mappen och rutan förblir öppen
när det avslutas. De som heter `AI agent` använder `herdr agent start`, så herdr
följer agentens tillstånd.

| Kategori | Förinställningar (`V` = verifierad genom körning eller mot den installerade hjälpen) |
|---|---|
| Explorer | Windows `explorer.exe` (V), macOS Finder (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; yazi, superfile, nnn, lf, ranger i en ruta/flik/terminal |
| Multiplexer | herdr-flik, workspace, delning höger/ned (V); tmux-fönster, delning höger/ned, frånkopplad session (V); zellij-ruta, flik, flytande; WezTerm-flik, delning, nytt fönster; kitty-flik, delningsfönster, OS-fönster (V: flaggor från hjälpen; kräver `allow_remote_control`); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (fönster, flik, delning och en flik var för PowerShell 7, Windows PowerShell, Command Prompt, WSL), fönster för PowerShell 7 / Windows PowerShell / Command Prompt / WSL, Git Bash |
| Editor | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, `$EDITOR` i en ruta/flik/terminal |
| Git tool | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; lazygit, tig, gitui i en ruta/flik/terminal (de för herdr, tmux, kitty och `xdg-terminal-exec` är V) |
| AI agent | Claude Code, Codex, Gemini CLI, opencode i en herdr-flik/delning (V, med Claude), tmux, zellij, WezTerm, kitty, ny terminal |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL på Windows: `wsl.exe --cd ${cwd}` accepterar Windows-sökvägen som den är, och
Windows Terminals `wsl.exe`-flik startar i mappen, som WSL monterar under `/mnt/`.

Utforskaren, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual Studio
och Zed (Zed finns bara på Unix) visas bara där de finns; tmux, zellij, cmux,
screen, kittys fjärrstyrning och de `sh`-drivna herdr-förinställningarna finns
bara på Unix; herdr (enkla förinställningar), WezTerm, Alacritty, VS Code, Cursor
och JetBrains-IDE:erna erbjuds överallt.

### Variabeln `${web_url}`

Webbsidan för repositoryts `origin`-remote, när det är ett GitHub- eller
GitLab-projekt (github.com, gitlab.com eller en egen driftsatt värd som har en
post i `[[integrations]]`); annars tom, och då stoppar en post som använder den
med ett meddelande i stället för att öppna en tom adress. Den är valfri: ingen
förinställning behöver den utom `Repository page on the web`.

## Sidan i Inställningar

Inställningar > **Öppna i** redigerar listan. Posterna är en [lista](settings.md#listor) till vänster (standardposten är markerad med `★`), den valda postens fält finns till höger; `+ Lägg till`, `Duplicera`, `Ta bort`, `↑` / `↓` och att dra en rad hanterar listan.

| Rad | Vad den gör |
|---|---|
| **Standardpost** | Posten som **Öppna i standard** kör. Den är markerad med `•` i listan Öppna i. |
| **Etikett** | Namnet som visas i listan. Unikt, inte tomt. |
| **Kommando** | Kommandot, med variabelchipsen (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) att infoga. |
| **Testa på aktuell flik** | Kör posten nu på den aktiva flikens mapp. |
| **+ Lägg till ▾** | Lägger till en färdig post (identifierade först), eller **Egen**, en tom post vars Kommando redigeras härnäst. |
| **Återställ identifierade standardvärden** | Ersätter listan med det som upptäcks på den här datorn och rensar standarden. |

Ändringar sparas medan du gör dem. Den första ändringen skriver hela
`[[ui.open_in]]`-arrayen till `config.toml`; tills dess beräknas listan (se
[Äldre nycklar](#äldre-nycklar)) och filen har ingen `open_in`-nyckel.
Sökningen i Inställningar hittar också sidan.

## Binärfiler: Öppna i

En binärfil (en PDF, ett Word-dokument, ett kalkylblad, ett arkiv...) har ingen text att visa i Diff-vyn eller filvyn. När operativsystemet känner till en app för typen visar vyn en knapp **Öppna i** bredvid den vanliga notisen. Listan innehåller först appen som systemet öppnar typen med (markerad med en punkt), sedan alla andra appar som systemet har registrerat för den (systemets egen lista *Öppna med*). På Linux är det filnamnets MIME-typ, `mimeapps.list` för standardappen och `.desktop`-filerna för de andra; på Windows registret (användarens val och de program som är registrerade för filändelsen); på macOS bara standardappen.

`o` kör standardappen och `Shift+O` öppnar listan. Båda finns i kommandopaletten (*Fil: Öppna i standardappen*, *Fil: Öppna med...*), och **Öppna med...** finns i högerklicksmenyn för en fil i Status-rutan och i en commits fillista. Sökningen körs i bakgrunden, en gång per filändelse: vyn säger ”söker efter appar...” medan den pågår. En typ som systemet inte känner någon app för, och vanlig text, behåller den vanliga notisen och har ingen knapp.

![En PDF i filvyn med sin lista Öppna i: först standardprogrammet, markerat med en punkt, sedan de andra program som systemet registrerat för typen](../images/open-with.png)

Filen i arbetskatalogen lämnas till appen som den är. En commits (eller indexets) version skrivs till en **skrivskyddad tillfällig kopia**, med en notis i statusraden, så att en app inte kan ändra repositoryt. Ingen av dem ändrar repositoryt: ingen journalpost, Ångra påverkas inte.

Menyns lista öppnas bredvid menyn, i höjd med raden **Öppna med...**, som en undermeny (till vänster när det inte finns plats till höger), både med musen och från tangentbordet. Listan över bildredigerare (**Redigera i...**) som öppnas från en filmeny gör likadant.

### Git LFS-filer

Kortet för Git LFS-objekt (se [Git LFS](lfs.md)) har samma två åtgärder under sin text: **Öppna i standardprogram** och **Öppna i**, med samma tangenter (`o`, `Shift+O`) och paletträder. De öppnar den riktiga filen, aldrig pekaren: filen i arbetskatalogen som den är, eller en skrivskyddad tillfällig kopia av objektet från det lokala LFS-lagret för en commits eller indexets version. Ett objekt som inte är nedladdat än har inga knappar på kortet, och statusraden säger att innehållet inte är nedladdat än. Samma vägran gäller en fil i arbetskatalogen som fortfarande bara innehåller pekaren: **Öppna i standardprogram**, **Öppna i** och **Redigera i**, från kortet eller vilken filmeny som helst, säger att innehållet inte är nedladdat i stället för att öppna pekartexten (ladda ned det med LFS-verktygsradens pull- eller checkout-rader och försök igen). Att öppna är skrivskyddat: ingen journalpost, inget att ångra.

### Webbläsare i Windows

Chrome och Edge registrerar `--single-argument %1`, vilket gör att de läser allt efter växeln som en enda adress, citattecken inklusive. GitPoulpi skickar sökvägen till ett sådant program ordagrant och utan citattecken, som Windows gör, så en fil med blanksteg i sökvägen öppnas i stället för att misslyckas med `http://"c/...`. Alla andra program får sökvägen som ett vanligt argument.

## Äldre nycklar

De gamla nycklarna `ui.terminal_launch` och `ui.terminal_launch_command` läses
bara medan `ui.open_in` saknas. Då är den beräknade listan:

1. ditt gamla terminalval, om det inte är `auto` (`custom` blir
   ”Terminal (custom)” med `{repo}` omskrivet till `${cwd}`; ett förinställnings-id
   blir den förinställningen). Ett `auto`-värde blir första posten, ”Default
   terminal”.
2. operativsystemets filutforskare,
3. multiplexrar och terminaler som upptäcks i miljön,
4. redigerarna som finns på `PATH`.

När du sparar listan från Inställningar tar `ui.open_in` över och de gamla
nycklarna ignoreras. Palettkommandona **Flik: Öppna i terminal** och **Flik:
Öppna i filhanteraren** använder inte den här listan.

## Felsökning

- *”program X was not found”*: programmet finns inte på den `PATH` som GitPoulpi
  ser. Starta GitPoulpi från ett skal som har det, eller skriv hela sökvägen.
- *post för tmux/zellij/herdr misslyckas direkt*: de pratar med en körande
  server, så de fungerar bara när GitPoulpi körs inuti den.
- *kitty-flikposten misslyckas*: den kräver `allow_remote_control` i `kitty.conf`.
- *Windows Utforskaren*: `explorer.exe` avslutas med kod 1 även vid lyckat
  resultat; GitPoulpi behandlar det som lyckat.
