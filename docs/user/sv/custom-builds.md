[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · **Svenska** · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Anpassade byggen

För den som vill ha en mindre binär: ett interaktivt skript frågar vilka språk,
värdintegrationer, teman, logotypfärger och funktioner som ska ingå och bygger
sedan bara dem med den optimerade profilen. Det vanliga bygget och den officiella
releasen är oförändrade (allt på).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd eller PowerShell (scripts/build-custom.ps1)
```

Utdata hamnar i `target/custom/`, aldrig över den officiella `target/dist/`.
Skriptet skriver ut varje binärs storlek och skillnaden mot `target/dist/` när
det bygget finns. Det skriver också `BUILD-FEATURES.txt` bredvid binärerna.

## Vad du väljer

| Val | Om du stänger av det |
|---|---|
| Språk | Bara engelska (byggs alltid) plus de du kryssar i. Språkväljaren i Inställningar listar bara byggda språk. |
| Teman | GitPoulpi Dark, GitPoulpi Light och Terminal byggs alltid (`system` behöver Dark och Light). Varje annat tema väljs per familj (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` skriver ut dem). Ovalda teman kompileras inte in: de saknas i kombinationsrutan i Inställningar och i `gitpoulpi theme list`, och en konfiguration som namnger ett faller tillbaka som ett okänt tema, med den vanliga varningen. |
| Logotypfärger | Pixelbläckfisken i Om-fönstret finns i 20 färgsättningar (`--list-logo-colors`). Välj de som ska kompileras in (minst en), standardfärgen (Om-fönstret visar den först; det är också vad en lagrad `ui.logo_color` som inte är inkompilerad faller tillbaka på; den måste vara en av de kompilerade; officiell standard: lavender) och färgen på den körbara Windows-filens ikon (vilken färgsättning som helst, standard: logotypens standard). Med en enda färgsättning snurrar logotypen fortfarande vid klick men behåller den färgen, och färglistan (`c`, högerklick) erbjuds inte. |
| Integrationer (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Utan några: bara clone via URL; inga ärenden, inga pull/merge requests. |
| AI | Inga AI-knappar, kommandon eller `ai` CLI/MCP-verktyg. |
| MCP | Ingen binär `gitpoulpi-mcp` och inget kommando `gitpoulpi mcp`. Agentkommandona `ui` finns kvar. |
| LFS, Worktree, Submoduler | De matchande hanteringsskärmarna och kommandona är borta; ett typat svar, ”not built into this binary”, returneras. |
| Nedladdning av avatarer | Bara initialmärken, ingen nätverkshämtning. |
| Bildförhandsvisning | Bildfiler visar det vanliga binärmeddelandet; **Redigera i** fungerar fortfarande. |
| Radar | Ingen Radar-flik, ingen `Alt+R` och inget palettkommando; Radar öppnas aldrig. Kräver en integration (utan någon är den ändå overksam). |
| Mermaid | Ingen Kopiera som Mermaid gitGraph (Log-menyn, `Shift+Y`, paletten) och ingen `log --format mermaid` i CLI/MCP. Ingen knapp **Öppna diagram** heller under ett `mermaid`-block i en [konversation](hosting-integration.md#5c-konversationer) (blocket stannar som kod) och ingen palettrad *Konversation: Öppna postens diagram*. |
| Board | Ingen Kanban-tavlevy (knappen i rubriken ÄRENDEN, paletten) och inget kommando `hosting board`. Kräver en integration. |
| Conversation | Ingen [konversationsvy](hosting-integration.md#5c-konversationer) i appen för ett ärende eller en begäran: Enter på en ärenderad, ett tavelkort eller en Radar-rad öppnar webbläsaren som förut, det finns ingen rad *Öppna konversation* och inget palettkommando *Konversation: ...*, och ingen **Skriv med AI** i en konversation (textrutorna i granskningen, ärendeformuläret och pull request-formuläret behåller sin). Konversationskommandona i CLI och MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) och `ai hosting-reply` finns kvar. Kräver en integration. |
| Review | Granskningens Översikt-sida är konversationsvyn, så ett bygge med Review innehåller också konversationsvyn, vad *Conversation* än säger. Ingen kodgranskningsvy i appen (radmenyn, Enter på en pull request-rad, paletten, Radars `Review`) och inget AI-stöd för granskning. CLI/MCP-kommandona `hosting review` finns kvar (Gerrit och agenter använder dem). Kräver en integration. |
| Omskrivningsvy (avancerad historikomskrivning: författare, datum, meddelanden, innehåll) | Ingen omskrivningsvy: inga `Omskrivning:`-palettkommandon, ingen tangentkombination för vyn, inga tangenter eller hjälpavsnitt, inga rader "Skriv om historiken" i Logs menyer, ingen knapp "Ändra historiken djupare..." i dialogen Ändra, ingen "Skapa regler med AI" eller skript-AI, inga skriptgränser på synkroniseringssidan i inställningarna och inga AI-block för historikomskrivning i AI-inställningarna. CLI/MCP-kommandona `rewrite`, `ai rewrite-script`, `ai rewrite-rules` och `ui propose rewrite` finns inte längre i kommandoträdet. `.mailmap`-identiteter (`mailmap`, `log.use_mailmap`) ingår inte och finns kvar. |
| Global sökning | Ingen knapp Global sökning i titelraden, ingen `Ctrl+Alt+F`, inget palettkommando `Sök: Arbetsyta...` och inget CLI/MCP-kommando `gitpoulpi search`. Sökraden i Log (`ui show search`) är en annan funktion och finns kvar. |
| Arbetsytor | Fönstret arbetar med den enda standardfliklistan ("Ingen arbetsyta"): inget arbetsytechip, ingen flik Arbetsytor, ingen `Alt+W` / `Alt+J`, inga `Arbetsyta:`-palettkommandon, inga flikrader "Lägg till i arbetsytan", inga CLI/MCP-kommandon `gitpoulpi workspace` och ingen flagga `--workspace` på `commit` och `search`. Flikar och sessioner fungerar som vanligt. En `workspaces.toml` skriven av ett annat bygge läses och ändras inte; en arbetsytas standardprofil och AI-instruktioner gäller inte. Radar och Global sökning täcker de öppna flikarna. |
| Omskrivningsskript (inbäddad Lua) | Script-regler i en historikomskrivning avvisas med ett typat svar "inte inbyggt i den här binären", och ingen Lua länkas in. Själva omskrivningsvyn finns kvar (den har ett eget val). Meningslöst, och varnas för, när omskrivningsvyn är av. |

Dina konfigurationsfiler delas mellan byggen: en inställning för något som inte
är byggt behålls och ignoreras.

## Interaktivt

Bash-skriptet använder kryssrutelistor från `whiptail` eller `dialog` (tre skärmar,
sedan en sammanfattning), eller numrerade frågor när ingen av dem finns.
PowerShell-skriptet använder `Out-GridView` när det finns, annars frågor. Dina
tidigare val är förikryssade.

## Icke-interaktivt

Varje argument hoppar över frågorna och går direkt till
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # tidigare val, skriv bara ut cargo-kommandot
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Alternativ | Betydelse |
|---|---|
| `--langs fr,de` / `all` / `en` | Språk förutom engelska. Koderna är locale-filernas namn (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Leverantörer: GitHub, GitLab (även egna installationer), Azure DevOps (Services och Server), Bitbucket (Cloud och Data Center) (värdtjänster), Gitea och Forgejo inklusive Codeberg (`gitea`) (värdtjänster), Jira, Trello, Linear, YouTrack (Cloud och Server), Redmine (ärendehantering), Gerrit Code Review (`gerrit`, bara granskning), lokala Markdown-ärenden (`local-issues`, en mapp eller ett Obsidian-valv). Var och en av `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` är en egen cargo-feature; storlekar i [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Temafamiljer att kompilera in (standard `all`). `--list-themes` skriver ut familjerna. |
| `--logo-colors all` / `violet,teal` | Logotypens färgsättningar att kompilera in (standard `all`, i tabellens ordning, som är snurrordningen). `--list-logo-colors` skriver ut dem (`*` = officiell standard). |
| `--logo-default teal` | Standardfärgsättningen; måste vara bland de kompilerade (standard: lavender, annars den första kompilerade). |
| `--icon-color sun` | Färgen på den körbara Windows-filens ikon (standard: logotypens standard). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Var och en är på som standard. |
| `--[no-]history-rewrite` | Omskrivningsvyn (avancerad historikomskrivning: författare, datum, meddelanden, innehåll). På som standard. |
| `--[no-]global-search` | Global sökning (knappen i titelraden, `Ctrl+Alt+F`, `gitpoulpi search`). På som standard. |
| `--[no-]workspaces` | Arbetsytor (namngivna grupper av repositories). På som standard; av lämnar bara standardfliklistan. |
| `--[no-]rewrite-scripts` | Inbäddad Lua för Script-reglerna i en omskrivning. På som standard; ignoreras med en varning när omskrivningsvyn är av. |
| `--save <name\|file>` | Sparar de uppslagna valen som ett sparat val (ett namn eller en filsökväg). |
| `--preset <name\|file>` | Utgår från ett sparat val (ett namn eller en filsökväg); de andra alternativen åsidosätter det, var de än står på raden. |
| `--list-presets`, `--delete-preset <name>` | Visar de sparade valen och deras mapp / tar bort ett med namn. |
| `--last` | Återanvänd `target/custom-build/last-choices.env`; andra alternativ åsidosätter den. |
| `--full` | Inget urval alls (allt på). |
| `--profile custom\|dist\|release` | Standard `custom` (`dist`-inställningarna, separat utdatakatalog). |
| `--dry-run` | Skriv ut det exakta cargo-kommandot och stanna. |
| `-- <cargo args>` | Skickas till cargo, till exempel `--target x86_64-unknown-linux-musl`. |

PowerShell-namn: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

Temaurvalet når bygget som miljövariabeln `GITPOULPI_THEMES` (`all`, `none` eller
en kommalista med familjer), läst av `gitpoulpi-config`s byggskript, som tar bort
de andra paletterna vid kompilering; sätt den själv för ett vanligt `cargo build`
om du föredrar det. `gitpoulpi --version` skriver ut en rad `themes:` med den uppsättning som byggdes.

Logotypvalen är miljövariablerna `GITPOULPI_LOGO_COLORS`
(`all` eller en kommalista med slugs), `GITPOULPI_LOGO_DEFAULT` och
`GITPOULPI_ICON_COLOR`, lästa av byggskripten för `gitpoulpi-tui` (det filtrerar
`assets/logo/colourways.json` till tabellen som Om-fönstret bäddar in) och för de
tre binärcraten (ikonen). Källan till färgerna förblir
`assets/logo/colourways.json`.

### Den körbara Windows-filens ikon

När målet är Windows får de körbara filerna `gitpoulpi`, `gitpoulpi-tui` och
`gitpoulpi-mcp` bläckfisken som ikon, genererad vid bygget (ingen Python eller
ImageMagick behövs) i den valda ikonfärgen, storlekar 16 till 128 px. Den bäddas
in med craten `winresource` och behöver plattformens resurskompilator
(`rc.exe` från Windows SDK på MSVC, `windres` på GNU). Saknas den skriver bygget
bara ut en varning och den körbara filen behåller standardikonen. På
GNU-toolchainen bär bara `gitpoulpi` ikonen (resursen skulle annars länkas två
gånger). Bygget för Linux och macOS ignorerar allt detta utom färgtabellen.

## Sparade val

För att bygga en ny version utan att göra om hela valet sparar du det under ett namn och utgår från det senare. Alternativ som anges tillsammans med `--preset` åsidosätter fortfarande de sparade värdena.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Sparade val är vanliga `NYCKEL=värde`-filer (formatet för `last-choices.env`, läsbart från ett skal). De ligger i mappen `custom-builds` i GitPoulpis konfigurationskatalog, utanför `target/`, så `cargo clean` tar inte bort dem:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` som standard)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` flyttar den mappen tillsammans med konfigurationen. `--preset` och `--save` tar också en filsökväg (ett värde som innehåller `/` eller `\`, eller slutar på `.env`), så att ett val kan ligga i ett dotfiles-förråd.

Ett val som skrivits av en annan version av GitPoulpi går fortfarande att läsa in. En okänd nyckel, eller ett språk, en integration, ett tema eller en färg som inte längre finns, ignoreras med en varning. En nyckel som saknas i filen (en funktion som lagts till sedan dess) får sitt standardvärde, och en integration som lagts till sedan dess tas med; båda rapporteras på standardfelutdata.

När ett sparat val finns erbjuder det interaktiva läget först "Utgå från" (tidigare val, ett sparat val eller allt på) och frågar till sist om det aktuella valet ska sparas under ett namn.

## Vad det sparar

Uppmätt på Linux 2026-10-29 vid commit `65999ef5c`, från det fulla bygget, skillnaden mot det inom parentes
(detaljer i `docs/developer/binary-size.md`):

| Bygge (från `full` om inget annat anges) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Full (allt på) | 64,6 | 50,5 | 31,7 |
| Bara engelska | 52,0 (-12,6) | 40,2 (-10,3) | 31,7 (0,0) |
| MCP av | 61,6 (-3,0) | 49,6 (-0,8) | inte byggd |
| AI av | 62,6 (-2,0) | 48,6 (-1,8) | 30,5 (-1,3) |
| Alla integrationer av | 56,3 (-8,2) | 43,7 (-6,7) | 26,7 (-5,0) |
| AI + integrationer + avatarhämtning av | 52,2 (-12,3) | 39,9 (-10,6) | 23,0 (-8,8) |
| Omskrivningsvy av (`--no-history-rewrite`) | 61,6 (-3,0) | 48,9 (-1,5) | 28,5 (-3,2) |
| Omskrivningsskript av | 63,1 (-1,5) | 49,0 (-1,5) | 31,1 (-0,7) |
| Global sökning av | 63,5 (-1,1) | 49,5 (-1,0) | 31,6 (-0,1) |
| Arbetsytor av | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,2) |
| Radar av | 63,4 (-1,2) | 49,3 (-1,2) | 31,7 (0,0) |
| Board av | 63,6 (-1,0) | 49,5 (-1,0) | 31,7 (0,0) |
| Review av | 63,2 (-1,4) | 49,0 (-1,4) | 31,7 (0,0) |
| Mermaid av | 63,6 (-1,0) | 49,5 (-0,9) | 31,6 (-0,1) |
| LFS av | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,3) |
| Worktree av | 63,7 (-0,9) | 49,6 (-0,8) | 31,7 (0,0) |
| Submoduler av | 63,7 (-0,9) | 49,6 (-0,9) | 31,6 (-0,1) |
| Avatarhämtning av | 63,7 (-0,9) | 49,6 (-0,9) | 31,7 (0,0) |
| Bildförhandsvisning av | 63,3 (-1,2) | 49,2 (-1,3) | 31,7 (0,0) |
| Allt av, bara engelska (LFS kvar) | 35,2 (-29,4) | 28,9 (-21,5) | inte byggd |
| **Allt av, bara engelska** | **34,9 (-29,7)** | **28,8 (-21,6)** | inte byggd |

## Anteckningar

- Ett anpassat bygge är långsamt som varje `dist`-bygge (fat LTO, några minuter).
- Det körs utan sccache, som `build-opti`.
- Språkkoder kontrolleras mot de levererade locale-filerna; en okänd kod vägras
  med listan över giltiga.
