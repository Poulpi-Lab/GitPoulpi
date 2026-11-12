[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · **Svenska** · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Välja terminal

GitPoulpi fungerar i vilken terminal som helst med musstöd, men ser och känns bäst
i en som ritar bilder, visar true color och skickar hela tangentbordet. Den här sidan
säger vad varje terminal ger dig på varje OS, och vilken inställning du ska ändra.

**Så läser du den.** Tabellerna kommer från en testomgång i **november 2026**: varje
terminal drevs med råa escape-sekvenser och, där den kunde köra det, med
GitPoulpi självt, på Linux, Windows 11 och macOS 15. Versionerna listas
[nedan](#testade-versioner). Varje cell är något av:

- **fungerar**;
- **delvis** (och vad som saknas);
- **fungerar inte** (och varför: *terminalen*, *OS-lagret* eller *GitPoulpi*);
- **n/a** (terminalen finns inte på det OS:et);
- **inte uppmätt** (och varför).

Multiplexers (tmux, zellij, screen, herdr, psmux, paneler) har en egen sida:
[Multiplexers](multiplexers.md).

## Bästa val

| OS | Bästa val, i ordning | Varför |
|---|---|---|
| **Windows** | **Windows Terminal**, sedan **WezTerm** och **Rio** | Windows Terminal ritar grafen och avatarerna som sixel-bilder, förankrade till höger, och är startarens standard. WezTerm och Rio ritar dem som iTerm2-bilder, förankrade till höger sedan ConPTY-rättningen i november 2026 (se [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty och Ghostty ger bilder, pekarformerna och hela tangentbordet. foot ritar sixel-bilder och har hela tangentbordet och pekarformerna. |
| **macOS** | **iTerm2**, sedan **kitty** och **Ghostty** | iTerm2 3.7.7 ritar bilder genom kitty-protokollet. Terminal.app fungerar men har varken bilder eller true color. kitty och Ghostty motsvarar sitt Linux-beteende (inte uppmätt på macOS). |

På alla OS döljer **screen** och **zellij** terminalens bildprotokoll för
GitPoulpi, så det ritar tecken; tmux släpper igenom sixel. Se
[Multiplexers](multiplexers.md).

## Vad GitPoulpi använder

- **Bilder** (grafkolumnen, författaravatarer, bilddiff och filvy): **kitty
  graphics protocol** med Unicode-platshållare, **iTerm2 inline images**
  (`OSC 1337`) eller **sixel**. Identifieringen är nej-som-standard, utifrån `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` och multiplexerns variabler. När
  miljön inte nämner någon (en xterm startad med `-ti vt340`, en VS Code-terminal
  med bilder på, vilken terminal som helst nådd över SSH, där `TERM_PROGRAM` inte
  vidarebefordras) avgör terminalens egna svar: GitPoulpi frågar en gång vid start
  (se ”Startfrågan” nedan) och använder terminalens XTVERSION-namn (kitty,
  Ghostty, WezTerm, iTerm2, mintty, foot) eller sixel i dess enhetsattribut.
  Inuti en multiplexer ignoreras svaren, eftersom tmux påstår sixel oavsett vad
  som sitter bakom den.
- **Vilket protokoll var.** kitty, Ghostty och iTerm2 3.7.7 eller nyare: kitty-protokollet.
  WezTerm och Rio: iTerm2-protokollet, med PNG-bildens genomskinlighet inblandad.
  mintty och iTerm2 äldre än 3.7.7: iTerm2-protokollet med ogenomskinlig
  bakgrund. Windows Terminal, foot, Tabby, Konsole och mlterm: **sixel**.
  Allt annat: tecken, och avataren blir initialmärket.
  `ui.image_protocol` tvingar fram ett.
- **iTerm2 3.7.7.** Varje `OSC 1337`-inline-bild öppnar en modal dialog ”Allow
  Terminal-Initiated Display?” som blockerar fönstret tills du bockar i
  ”Remember my choice”. Samma bilder skickade genom kitty-protokollet, Unicode-platshållare
  inräknade, ritas utan fråga, så GitPoulpi använder kitty-protokollet där.
  `ui.image_protocol = "iterm2"` ger fortfarande inline-bilderna.
- **Sixel** ritas för **grafkolumnen** (lanes, noder och, med
  `ui.avatars = "graph"`, avatarerna i commit-cirklarna) och för
  **avatarerna** på andra håll (Log, commit-detaljer, Filhistorik, Blame, bildvyn). En
  sixel har ingen alfakanal, inga id och ingen skalning, så bilden är exakt `kolumner x
  cellbredd` gånger `rader x cellhöjd` pixlar, ritad bara där något finns, över
  tomma celler som behåller sin bakgrund; den kodas om bara när lanes,
  rullning eller zoom ändras. En sixel går inte att radera: GitPoulpi skriver text
  över dess celler för att ta bort den. Sixel behöver cellens verkliga storlek i pixlar: Windows
  Terminal, Tabby och mintty svarar på `CSI 16 t` en gång vid start och efter en zoom
  (blockerar aldrig om de förblir tysta); på andra håll kommer den från terminalens
  fönsterstorlek eller från `GITPOULPI_TERMINAL_CELL_PX=10x20`. En terminal som ritar
  sixel utan att säga det behöver `ui.image_protocol = "sixel"`. Windows Terminal
  före 1.22 ritar inte sixel; sätt `ui.image_protocol = "none"` där.
- **True color**: teman är RGB. `ui.color_depth = "auto"` går ned till 256 färger vid ett
  `*-256color` `TERM` utan `COLORTERM=truecolor` (och i Terminal.app, som inte har
  true color), och till 16 färger i Linux-konsolen.
- **Mus**: SGR-musrapportering med knappar, drag och rörelse (hover-respons, dra
  och släpp, högerklick, dubbelklick).
- **Startfrågan.** Fyra frågor går ut i en enda skrivning: kitty-tangentbordsflaggorna
  (`CSI ? u`), bakgrundsfärgen (`OSC 11`, används bara för `ui.theme = system`),
  terminalens namn (XTVERSION, `CSI > 0 q`) och dess enhetsattribut (`CSI c`, sist: varje
  terminal svarar på den, så den avslutar läsningen). Läsningen är en enda `poll`-slinga med en tidsgräns på 100 ms,
  klar innan tangentbordsläsaren startar. En terminal som inte svarar alls
  kostar 100 ms en gång och allt förblir som miljön säger. Bara Unix: på
  Windows rapporterar konsolen tangenter själv och miljön avgör. Svaren finns i
  Aktivitetsloggens miljörapport (raden `start-up query:`).
- **Tangentbord**: **kitty keyboard protocol** (bara flaggan disambiguate) begärs vid
  start. Det gör att `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` och `Ctrl+Shift+Z` kommer fram.
  Utan det finns de gamla tangenterna kvar (`Ctrl+Y` gör om, `Alt`-varianter ersätter vissa
  sekvenser). Se
  [Kortkommandon](keybindings.md#tangenter-som-terminalen-inte-kan-skicka-och-kitty-tangentbordsprotokollet).
- **Urklipp**: varje kopieringsåtgärd följer en enda policy. Lokalt: **systemets urklipp**
  på maskinen där GitPoulpi körs, plus `OSC 52` till terminalen när terminalen är
  känd för att respektera det (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` är också reserven när systemets urklipp misslyckas. **Över
  SSH, eller utan skärm** (en Linux-konsol, en container) skickas bara `OSC 52`, och
  inuti tmux skickas den inpackad för genomsläppet likaväl som bar. Statusraden säger
  ”kopierade” bara när systemets urklipp tog texten; det som gick till terminalen säger
  ”skickade ... till terminalens urklipp (OSC 52, ej bekräftad)”, eftersom inget bekräftar
  att terminalen lydde. Aktivitetsloggens miljörapport anger vägen på sin
  `clipboard:`-rad.
- **Musmarkörens form** (storleksmarkör över avdelare, hand över klickbara objekt, I-balk
  över textfält): `OSC 22`, en bonus där terminalen implementerar den. GitPoulpi
  skickar den alltid (utom i Linux-konsolen och bakom screen, zellij och herdr, som sväljer
  eller skriver ut den; bakom tmux packas den in för genomsläppet). Under `TERM_PROGRAM=mintty`
  skickas mintty egna markörnamn (`hand`, `ibeam`, `sizewe`, ...), de enda
  mintty läser. **Rio implementerar inte `OSC 22` på något OS**: dess källkod tolkar
  sekvensen och kastar den, så ingen inställning på någondera sidan ändrar det.
- **Textmarkör**: med `ui.text_cursor = "terminal"` (standard) placeras terminalens egen markör
  vid skrivpositionen i det fokuserade textfältet, som ett blinkande streck (`DECSCUSR`), och döljs
  på andra håll; din egen markörstil kommer tillbaka vid avslut. `drawn` ritar GitPoulpis egen markör i stället.
- **Fokushändelser** är på (verktygstips stängs när fönstret tappar fokus).
  **Bracketed paste** begärs vid start och släpps vid varje väg ut (Unix), så en
  inklistring med flera rader kommer fram som en enda: dess radbrytningar skickar aldrig in ett fält och dess
  bokstäver utlöser aldrig kortkommandon. På Windows har konsolen inga inklistringsparenteser; se
  [Windows](#windows) för vad GitPoulpi gör i stället.
  GitPoulpi begär inte OSC 8-hyperlänkar eller undercurl.
  Bara på bildvägen omsluts varje bildruta av en synkroniserad uppdatering
  (läge 2026, ignoreras av terminaler som inte känner till det) så att en bild
  aldrig blinkar över en popup.
- **Emoji och symboler**: GitPoulpi använder bara tecken med emoji som standardpresentation
  eller enkelbreda BMP-former, eftersom terminaler och typsnitt är oeniga om
  variationsväljare.

## Matrisen

Raderna är terminaler, kolumnerna är OS. ”GitPoulpi” i en cell betyder att GitPoulpi självt
kördes där; ”probe” betyder att bara råa escape-sekvenser kördes. **n/a**-celler: kitty har inget
inbyggt Windows-bygge (releaserna levereras för Linux och macOS); Ghostty har inget Windows-bygge
(bara dess bibliotek libghostty är plattformsoberoende); foot är en Wayland-terminal (bara Linux);
xterm är en X11-klient (Linux, och macOS genom XQuartz); iTerm2 och Terminal.app finns bara för
macOS; Windows Terminal, conhost och mintty finns bara för Windows (mintty också under Cygwin och
MSYS2).

**Inte uppmätt på macOS: kitty, Ghostty, WezTerm, Rio och Warp.** De kräver en GPU
(Metal eller OpenGL 3.3). Deras Linux-resultat är referensen för vad samma program gör;
kör om dem på en Mac med GPU innan du litar på en macOS-cell.

### Bilder: vilket protokoll GitPoulpi väljer, och om det syns

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fungerar**: kitty-protokollet (GitPoulpi) | n/a | inte uppmätt (kräver en GPU) |
| **Ghostty** | **fungerar**: kitty-protokollet (GitPoulpi) | n/a | inte uppmätt (kräver en GPU) |
| **WezTerm** | **fungerar**: iTerm2-protokollet med genomskinlighet (GitPoulpi). Platshållarna ritas hoptryckta, så kitty-protokollet används inte | **fungerar**: iTerm2-protokollet med genomskinlighet, graf och avatarer på sina rader (officiella 20240203 och bygget med uppströms PR #8092). Officiella 20240203 la bilderna på fel rader innan GitPoulpi visade markören runt dem, se ”ConPTY-gränsen”. | inte uppmätt (kräver en GPU) |
| **foot** | **fungerar**: sixel med genomskinlighet (GitPoulpi) | n/a | n/a |
| **Rio** | **fungerar**: iTerm2-protokollet med genomskinlighet (GitPoulpi) | **fungerar**: iTerm2-protokollet med genomskinlighet, cellexakt vid varje provad typsnittsstorlek (0.5.28; GitPoulpi). Sixel och kitty kommer aldrig fram (ConPTY kastar dem), så det finns inget att välja | inte uppmätt (kräver en GPU) |
| **Alacritty** | text: **fungerar** (inget bildstöd i terminalen) | text: **delvis** (inga bilder; bilddiff som block) | text: **fungerar** |
| **Tabby** | **fungerar**: sixel med genomskinlighet (probe; GitPoulpi kördes inte i Tabby på Linux) | **fungerar**: sixel, justerat (GitPoulpi) | **fungerar**: sixel, genomskinlighet bevarad (GitPoulpi) |
| **Warp** | inte uppmätt (den behöver XWayland och avkodar syntetiska tangenttryckningar fel, så inget kunde skrivas) | **fungerar inte** (terminalen: inga sixel-, iTerm2- eller kitty-bilder); GitPoulpi inte uppmätt (kunde inte startas från Warps inmatningsruta) | inte uppmätt (kräver en GPU) |
| **xterm** | probe: sixel **fungerar** med `-ti vt340`; GitPoulpi väljer det utifrån enhetsattributen (startfrågan kom efter körningen), eller sätt `ui.image_protocol = "sixel"` | n/a | text: **fungerar**; sixel ritas (probe) och behöver `ui.image_protocol = "sixel"` |
| **iTerm2** | n/a | n/a | **fungerar**: kitty-protokollet på 3.7.7 och nyare, ingen dialog (GitPoulpi). Äldre versioner får inline-bilderna med ogenomskinlig bakgrund. Sixel ritas men målar oinställda pixlar ogenomskinliga, så `ui.image_protocol = "sixel"` ger en enfärgad rektangel |
| **Terminal.app** | n/a | n/a | text: **fungerar** (inga bilder) |
| **Windows Terminal** | n/a | **fungerar**: sixel (GitPoulpi, 1.25) | n/a |
| **conhost** | n/a | text: **delvis** (teckengraf, bilddiff med halvblock) | n/a |
| **mintty** | n/a | **fungerar inte** (terminal bakom ConPTY: ingen bild ritas alls, inte ens en vanlig `OSC 1337` placerad med markören synlig; grafkolumnen förblir tom; text och palett är fina) | n/a |

### Muspekarens form (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fungerar** | n/a | inte uppmätt (kräver en GPU) |
| **Ghostty** | **fungerar** | n/a | inte uppmätt (kräver en GPU) |
| **WezTerm** | **fungerar inte** (terminalen: inte implementerat i bygget som testades) | officiell: **fungerar inte** (terminalen). Bygge med uppströms WezTerm PR #8092, som lägger till pekarformer: **fungerar** (hand, I-balk, storleksändringspilar) | inte uppmätt (kräver en GPU) |
| **foot** | **fungerar** | n/a | n/a |
| **Rio** | **fungerar inte** (terminalen: tolkas och kastas) | **fungerar inte** (terminalen, samma skäl; inte ConPTY) | **fungerar inte** (terminalen, samma källkod; kördes bara på Linux och Windows) |
| **Alacritty** | **fungerar inte** (terminalen) | **fungerar inte** (terminalen) | **fungerar inte** (terminalen) |
| **Tabby** | **fungerar inte** (terminalen: xterm.js) | **fungerar inte** (terminalen) | **fungerar inte** (terminalen) |
| **Warp** | inte uppmätt | **fungerar inte** (terminalen) | inte uppmätt |
| **xterm** | **fungerar inte** (terminalen: CSS-namn okända) | n/a | inte uppmätt (pekaren var utanför fönstret) |
| **iTerm2** | n/a | n/a | **fungerar inte** (terminalen) |
| **Terminal.app** | n/a | n/a | **fungerar inte** (terminalen) |
| **Windows Terminal** | n/a | **fungerar inte** (terminalen) | n/a |
| **conhost** | n/a | **fungerar inte** (terminalen) | n/a |
| **mintty** | n/a | **fungerar** med mintty egna namn; GitPoulpi skickar dem | n/a |

Där cellen säger ”fungerar inte” tänds ändå avdelaren eller knappen under musen.

### Tangentbordsprotokoll

GitPoulpi ber om kitty-tangentbordsprotokollet på Linux och macOS. På Windows ber det inte
om det: konsolen rapporterar modifierare själv, och sekvenserna nedan begränsas av vad
terminalen behåller för sig själv.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fungerar** | n/a | inte uppmätt (kräver en GPU) |
| **Ghostty** | **fungerar** | n/a | inte uppmätt (kräver en GPU) |
| **WezTerm** | **delvis**: inget svar förrän `enable_kitty_keyboard = true` i `wezterm.lua` (standard `false`) | **delvis**: WezTerm behåller `Ctrl+Shift+N` (öppnar dess fönster); `Ctrl+,` levereras inte (officiellt bygge). Bygget med PR #8092 behåller ingen av sekvenserna, men `Ctrl+,` öppnar ändå inte Inställningar (inget kitty-tangentbordssvar genom ConPTY) | inte uppmätt (kräver en GPU) |
| **foot** | **fungerar** | n/a | n/a |
| **Rio** | **fungerar** (på som standard) | **delvis**: Rio behåller `Ctrl+Shift+N` (öppnar dess fönster) | inte uppmätt (kräver en GPU) |
| **Alacritty** | **fungerar** (0.13 och senare) | **delvis**: `Ctrl+Shift+N` kommer fram som `Ctrl+N`, `Ctrl+,` levereras inte | **fungerar** |
| **Tabby** | **fungerar inte** (terminalen: inget svar; den behåller `Ctrl+Shift+N` och `Ctrl+,` för sig själv) | **delvis**: Tabby behåller `Ctrl+Shift+N` och `Ctrl+,` | **fungerar inte** (terminalen: inget svar); vissa sekvenser kom inte fram |
| **Warp** | inte uppmätt | inte uppmätt | inte uppmätt (kräver en GPU) |
| **xterm** | **fungerar inte** (terminalen) | n/a | **fungerar inte** (terminalen) |
| **iTerm2** | n/a | n/a | **fungerar** (3.7.7; kontrollera *Profiles > Keys* om det inte gör det) |
| **Terminal.app** | n/a | n/a | **fungerar inte** (terminalen: inget svar; gamla tangenter) |
| **Windows Terminal** | n/a | **delvis**: WT behåller `Ctrl+Shift+N` (nytt fönster) och `Ctrl+,` (dess inställningar) | n/a |
| **conhost** | n/a | inte uppmätt | n/a |
| **mintty** | n/a | **delvis**: ingen av sekvenserna hade någon synlig effekt | n/a |

Där en sekvens inte kommer fram, använd `Alt`-varianterna
([Kortkommandon](keybindings.md#tangenter-som-terminalen-inte-kan-skicka-och-kitty-tangentbordsprotokollet)),
eller bind om sekvensen i *Inställningar > Tangentbord & mus*.

### Urklipp (kopieringsåtgärder)

En kopiering använder alltid systemets urklipp när det finns en lokal skärm, så de här cellerna
beskriver terminalens `OSC 52`-väg, som spelar roll över SSH, i containrar och som
reserv. Varje GitPoulpi-kopieringstest som använde systemets urklipp fungerade.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fungerar** | n/a | inte uppmätt (kräver en GPU) |
| **Ghostty** | **fungerar** (sågs bakom tmux, zellij, herdr); bar: inte uppmätt (en urklippsskrivning kräver tangentbordsfokus, vilket testet inte gav den) | n/a | inte uppmätt (kräver en GPU) |
| **WezTerm** | **fungerar** | **fungerar** | inte uppmätt (kräver en GPU) |
| **foot** | inte uppmätt (en skrivning kräver tangentbordsfokus, vilket testet inte gav den) | n/a | n/a |
| **Rio** | inte uppmätt (samma skäl) | **fungerar** | inte uppmätt (kräver en GPU) |
| **Alacritty** | inte uppmätt (samma skäl) | **fungerar inte** (urklipp oförändrat med standardkonfigurationen; inte undersökt) | **fungerar** |
| **Tabby** | **fungerar** | **fungerar** | **fungerar** |
| **Warp** | inte uppmätt | **fungerar inte** (urklipp oförändrat) | inte uppmätt (kräver en GPU) |
| **xterm** | inte uppmätt | n/a | **fungerar inte** (urklipp oförändrat) |
| **iTerm2** | n/a | n/a | **fungerar inte** förrän *Settings > General > Selection > Applications in terminal may access clipboard* är på |
| **Terminal.app** | n/a | n/a | **fungerar inte** (terminalen) |
| **Windows Terminal** | n/a | **fungerar** (medan den har fokus) | n/a |
| **conhost** | n/a | inte uppmätt (ingen probe kördes) | n/a |
| **mintty** | n/a | **fungerar inte** (urklipp oförändrat med dess standardinställningar) | n/a |

### Inklistring

På Linux och macOS begär GitPoulpi bracketed paste, så en inklistring med flera rader är en enda inklistring.
Det kördes inte terminal för terminal (**inte uppmätt** per terminal). På Windows har konsolen
inga inklistringsparenteser, så en terminal som inte frågar först levererar texten som
tangenttryckningar och varje radbrytning skulle bli ett `Enter`. GitPoulpi behandlar därför tangenttryckningar
som följer på varandra inom 30 ms (ett inre `Enter`, eller åtta tecken eller fler) som en
inklistring. Hållna tangenter och ”abc, `Enter`” skrivet för hand är inte inklistringar.

| Terminal | Windows (inklistring på två rader i prompten för ny gren) |
|---|---|
| **Windows Terminal** | **fungerar**: WT frågar ”paste several lines?” först (dess egen varning), därefter skapas ingen gren |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **fungerar** (ingen gren skapas) |
| **Warp** | inte uppmätt (GitPoulpi kunde inte startas i den) |
| **conhost** | inte uppmätt (`Ctrl+V` och `Shift+Insert` skickade av testet klistrade inte in i den klassiska konsolen) |

### Bredd på emoji och ikoner

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **fungerar** (breda celler justerade; kitty ritar kugghjulet U+2699 tomt, en känd typsnittsfråga) | WezTerm, Rio, Tabby, Windows Terminal: **fungerar**. Alacritty: **delvis** (emoji som enfärgade konturer) | Alacritty, Tabby: **fungerar**. Övriga: inte uppmätt (kräver en GPU) |
| **Tabby** | **fungerar** | **fungerar** | **fungerar** |
| **Warp** | inte uppmätt | **fungerar** | inte uppmätt |
| **xterm** | **delvis** (rutor: typsnittet) | n/a | **delvis** (rutor: typsnittet) |
| **iTerm2**, **Terminal.app** | n/a | n/a | **fungerar** |
| **conhost** | n/a | **delvis**: färgemoji ritas, breda celler justerade | n/a |
| **mintty** | n/a | **fungerar** | n/a |

### Testade versioner

kitty 0.49.2 (macOS 0.50.0 startade inte), Ghostty 1.4.0, WezTerm 20260716 på Linux och
20240203 (den senaste taggade releasen) på Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) och
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) och 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Terminaler förändras snabbt: en version nyare än dessa kan skilja sig.

## Per terminal

### Windows

- **Windows Terminal.** Startarens standard och det bästa bildstödet på Windows:
  sixel-graf och avatarer, OSC 52, bra emoji, och den svarar på cellstorleksfrågan. Välj ett
  Nerd Font i *Settings > Profiles > Defaults > Appearance > Font face*. Håll `Shift` för att
  markera text medan musen är fångad. Den behåller `Ctrl+Shift+N` och `Ctrl+,`: använd
  `Alt`-varianterna eller bind om. Pekarformen ändras inte.
- **WezTerm.** Sätt `config.enable_kitty_keyboard = true` i `wezterm.lua` (Linux- och
  macOS-byggen) och ett typsnitt med emoji-reserv, till exempel
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Bilder använder iTerm2-protokollet och inget behöver ställas in: den senast taggade releasen,
  20240203-110809 (feb 2024), förankrar dem på rätt rader (kontrollerad på nytt i november 2026 med
  nuvarande GitPoulpi; före ConPTY-rättningen som beskrivs nedan gjorde den inte det). Uppströms WezTerm PR #8092 lägger till pekarformer.
- **Rio** (MSI, kräver UAC). Bilder (graf, avatarer, bilddiff) använder iTerm2-protokollet
  med genomskinlighet och hamnar exakt på sina celler: uppmätt på 0.5.28 med standardtypsnitt
  och med 18 och 22,5 px (celler på 8x16, 11x21 och 13x26 pixlar, alltid hela pixlar), en bild på 20x10 celler och
  en på 4x27 celler blev båda cellexakta. Inget behöver ställas in. Pseudokonsolen
  sväljer cellstorleksfrågan, så GitPoulpi antar att Rios celler är dubbelt så höga som breda
  (sant vid alla tre storlekar); med ett typsnitt med annan proportion sätt
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (till exempel `10x24`) för att få runda commit-markörer. Rio
  har inga pekarformer på något OS.
- **Tabby.** Sixel-graf och avatarer, kopiering och inklistring fungerar. Tabby behåller `Ctrl+Shift+N` och
  `Ctrl+,` för sig själv.
- **Alacritty.** Kräver OpenGL 3.3. Teckengraf; kopiering via `OSC 52` var inte på med standardkonfigurationen.
- **mintty** (Git Bash, MSYS2, Cygwin). Pekarformer fungerar (mintty egna namn). Bilder
  syns inte (grafkolumnen förblir tom; mintty 3.8.3 ritar ingen `OSC 1337`-bild bakom
  ConPTY, inte ens när den placeras av ett vanligt test) och den svarar inte på någon cellstorleksfråga där: sätt
  `ui.graph_renderer = "text"`.
- **Warp.** Inte uppmätt med GitPoulpi: det kunde inte startas från Warps inmatningsruta. Terminalen
  har inga sixel-, iTerm2- eller kitty-bilder.
- **conhost, cmd, ett PowerShell-fönster.** Teckengraf, bilddiff med halvblock. Som standard
  öppnar sig GitPoulpi, när det startas från ett sådant fönster, igen i den terminal du valt; se
  ”Omstartsregeln”.

**ConPTY-gränsen.** Windows kör varje terminals program bakom en pseudokonsol
(ConPTY). Windows Terminal och WezTerm-byggen som levereras med en ny sådan släpper igenom bildsekvenserna;
den som ligger inuti Windows (används av Rio och mintty), och den som levereras
med WezTerm 20240203, gör det inte. Det som går förlorat: DCS-sekvenserna (sixel) och APC-sekvenserna (kitty graphics)
kastas, så bara iTerm2-bilderna (`OSC 1337`) kommer fram, och därför använder Rio och
WezTerm dem; `CSI 16 t` (cellstorlek) besvaras bara i Windows Terminal och Tabby.
Det som rättades: en `OSC 1337` korsar ConPTY direkt och hamnar vid den markör ConPTY senast
målade, och ConPTY målar markören bara medan den är synlig. GitPoulpi håller markören
dold, så dess bilder hamnade på gamla rader (grafen över sidfoten, en sönderriven skärm efter
rullning). Det visar nu markören runt varje bild det skriver på Windows, inom samma
skrivning, och döljer den igen; Rio och WezTerm 20240203 förankrar varje bild på sin cell
(`pty_conpty_picture_anchor` läser tillbaka markörflytten bakom ConPTY). Inget ändras
utanför Windows eller för sixel.

**Inklistring på Windows.** Se tabellen ”Inklistring” ovan: GitPoulpi viker ihop en skur av tangenttryckningar
till en inklistring, så en inklistring med flera rader skickar aldrig in en prompt i de terminaler som testades.

**Omstartsregeln.** Ett dubbelklick på exe-filen startar den i ett klassiskt konsolfönster
(conhost) för sig. När `ui.launch_terminal` pekar på en annan terminal (standard: Windows
Terminal) öppnar GitPoulpi sig igen där och det första fönstret stängs
(`ui.relaunch_in_terminal`). Det gör det **bara** när det är ensamt i ett klassiskt konsolfönster.
En terminal som kör exe-filen direkt (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`) är där du vill vara, och inget startas om; från ett skal är det
inte ensamt, så inget händer heller. Stäng av det med `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Allt fungerar utan någon inställning. `Shift+klick` är kittys egen,
  så använd `Alt+klick` för att utöka en markering ([Mus](mouse-and-gestures.md)).
- **Ghostty.** Allt fungerar utan någon inställning. Om ikoner visas som hål, lägg till ett emoji-
  eller Nerd Font i `font-family` (upprepa nyckeln för att lägga till reserver).
- **WezTerm.** Bilder via iTerm2-protokollet, genomskinlighet inblandad. Sätt
  `enable_kitty_keyboard = true` för hela tangentsekvenserna. Inga pekarformer.
- **foot.** Sixel-graf och avatarer, pekarformer, hela tangentbordet, snabb. Sätt
  `font=JetBrainsMono Nerd Font:size=11` i `foot.ini`. Kopiering via `OSC 52` styrs av
  `security.osc52`.
- **Rio.** Bilder via iTerm2-protokollet med genomskinlighet, hela tangentbordet. Inga
  pekarformer (Rio kastar `OSC 22`).
- **Alacritty.** Teckengraf, kitty-tangentbord sedan 0.13. Inget bildstöd.
- **Tabby.** Sixel-bilder; inget kitty-tangentbord, inga pekarformer (Electron, xterm.js). Den
  behåller `Ctrl+Shift+N` och `Ctrl+,`.
- **xterm.** Starta den som `xterm -u8 -ti vt340` för sixel (UTF-8 behövs för ramteckningen).
  Rutor i stället för emoji: typsnittet.
- **Warp.** Inte uppmätt: den behöver XWayland och avkodade i testet syntetiska tangenttryckningar fel.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Ingick inte i omgången i november 2026. Fungerar
  med tecken och gamla tangenter: använd `Alt`-varianterna och `Ctrl+Y` för att göra om. VTE
  ignorerar `OSC 52`, så kopiering över SSH når inte urklipp. En inmatningsmetod
  (IBus, Fcitx) kan ta `Ctrl+Space` för stage-växlingen; bind om den i *Inställningar >
  Tangentbord & mus*. Konsole känns igen för sixel utifrån sin egen dokumentation.
- **Wayland, Hyprland, Omarchy.** Pekarantydan (`OSC 22`) fungerar i kitty, Ghostty och
  foot. Omarchys startarpost går via `xdg-terminal-exec`.

### macOS

- **iTerm2.** 3.7.7 och nyare ritar bilder genom kitty-protokollet, utan fråga.
  Inställningar som hjälper:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    behövs för kopiering via `OSC 52` (annars visar iTerm2 ”the terminal attempted to
    access the clipboard but it was denied”).
  - *Profiles > Keys > Left Option key: Esc+*: behövs för `Alt`-sekvenserna.
  - Om du sätter `ui.image_protocol = "iterm2"` (eller använder en version äldre än 3.7.7), bocka i
    ”Remember my choice” i dialogen ”Allow Terminal-Initiated Display?”. Bilderna
    får då en ogenomskinlig bakgrund (en synlig rektangel om din bakgrund skiljer sig från
    temats; `ui.graph_renderer = "text"` tar bort den).
  - Kitty keyboard protocol är på som standard (3.6.9 och nyare).
- **Terminal.app.** Bara 256 färger: GitPoulpi väljer `ui.color_depth = 256` själv och
  teman ser plattare ut. Inga bilder, inget kitty-tangentbord, inget urklipp via `OSC 52`. Bocka i
  *Use Option as Meta key* för `Alt`-sekvenserna.
- **Alacritty, Tabby.** Alacritty: teckengraf, kitty-tangentbord, `OSC 52`-kopiering; sätt
  `option_as_alt = "Both"` för `Alt`-sekvenserna. Tabby: sixel-graf och avatarer.
- **Option som Meta.** Option komponerar tecken (Option+B skriver ett integraltecken) om inte
  terminalen gör den till Meta: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. GitPoulpis `Alt`-sekvenser behöver det, och ingenstans är det på
  som standard.
- **kitty, Ghostty, WezTerm, Rio, Warp**: inte uppmätt på macOS (de kräver en GPU). Räkna med
  Linux-beteendet.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel behöver `ui.image_protocol = "sixel"`.

### VS Code och Warp

VS Code-terminalen (`TERM_PROGRAM=vscode`) ingick inte i omgången. Den känns igen
utifrån sina enhetsattribut när `terminal.integrated.enableImages` är på, annars ritar GitPoulpi
tecken; `ui.image_protocol = "sixel"` tvingar fram bilderna. Warp: se anteckningarna per OS ovan.

## Multiplexers

tmux, zellij, screen, herdr, psmux och terminalernas egna paneler sitter mellan GitPoulpi och
terminalen och ändrar vad som släpps igenom. Kortversionen: tmux släpper igenom sixel och, med
`allow-passthrough on`, pekarformer och kitty-bilder; zellij och screen släpper igenom
varken bilder eller pekarformer; herdr släpper igenom kitty-bilder. Alla detaljer och
konfigurationen att lägga till finns i [Multiplexers](multiplexers.md).

## Vad som försämras, och hur

| Saknas | Vad du får i stället |
|---|---|
| Bildprotokoll | Teckengraf. Avataren blir det färgade initialmärket (en bokstav) oavsett vad `ui.avatar_fallback` säger: en identicon pressad in i teckenceller går inte att känna igen. Bilddiffen och filvyn använder färgade halvblock, eller ett infokort. |
| Truecolor | Färgerna mappas till 256- eller 16-färgspaletten (`ui.color_depth`). |
| Kitty keyboard protocol | Gamla tangenter; vissa sekvenser kommer fram som en annan tangent. Inställningar varnar, och du binder om eller använder `Alt`-varianterna. |
| OSC 52 | Lokalt ändras ingenting (systemets urklipp används). Över SSH eller utan skärm lägger kopieringsåtgärder ingenting i urklipp: markera med musen (håll `Shift`) och använd terminalens egen kopiering. |
| OSC 22 | Pekaren ändras inte; avdelaren eller knappen under musen lyser ändå upp. |
| Emoji-typsnitt | Rutor eller hål i stället för ikoner; grafen och texten fungerar ändå. Installera ett Nerd Font eller ett emoji-typsnitt och ange det som reserv. |

## Inställningar att känna till

| Inställning | Användning |
|---|---|
| `ui.image_protocol` | `auto` (standard), `kitty`, `iterm2`, `sixel`, `none`. Sätt `none` om du ser skräp där bilder ska vara. Kräver omstart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` är alltid rätt (använd det där ConPTY-gränsen gäller). Kräver omstart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` eller `off`, om en terminal eller multiplexer beter sig fel. Kräver omstart. |
| `ui.text_cursor` | `terminal` (standard) eller `drawn`. |
| `ui.mouse_enabled` | Av lämnar musen åt terminalen. Kräver omstart. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | Terminalen som OS-genvägarna öppnar, och om ett dubbelklick öppnar GitPoulpi igen där ([Inställningar](settings.md)). |

Se även [Multiplexers](multiplexers.md), [Bilder](image-view.md),
[Kortkommandon](keybindings.md) och [Felsökning](troubleshooting.md).

## Källor

Dokumentation kontrollerad i oktober 2026, beteende testat i november 2026. Primära källor om inget annat anges.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty keyboard från [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (sekundär)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (sekundär), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (ingen `OSC 22`-rad), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 från en [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (sekundär)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (ingen truecolor): [Apple discussion](https://discussions.apple.com/thread/255315820) (sekundär)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Jämförelsesajter (sekundära, används bara för korskontroll): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
