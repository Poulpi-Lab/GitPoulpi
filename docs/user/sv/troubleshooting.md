[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · **Svenska** · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Felsökning och vanliga frågor

Börja med de snabba svaren nedan; de längre avsnitten tar upp Git LFS och
grafritningen. Om inget av det hjälper, skapa ett ärende i GitHub-repositoryt med
ditt operativsystem, din terminal, `gitpoulpi --version` och vad du gjorde.

## När något misslyckas

En misslyckad åtgärd visar ingen popup och inget inramat kort: **statusraden** säger det, i rött, med rubriken och orsaken (*fel: Fetch misslyckades: Remoten kunde inte nås.*), och meddelandet ligger kvar tills ett annat ersätter det. Knappen Aktivitet längst till höger räknar de fel du ännu inte har tittat på. Klicka på det röda meddelandet, eller kör palettkommandot `Aktivitetslogg: Öppna det senaste felet`, för att öppna Aktivitetsloggen på den posten. Varje misslyckande du själv har utlöst fungerar så, inte bara git-åtgärder:

- **git-åtgärder**: en fetch, en push, en pull, en checkout, en merge, en rebase, en commit, en stash...
- **En kloning** (och att skapa ett nytt repository): ”Kloningen misslyckades”.
- **Ett anrop till GitHub eller GitLab**: repositorylistorna i väljarna och i dialogen Klona, att skapa en pull request eller ett ärende, kontroll av en token eller en SSH-nyckel på sidan Integrationer: ”Begäran till värdtjänsten misslyckades”.
- **Ett externt program**: redigeraren, diff- eller merge-verktyget, filhanteraren, webbläsaren, terminalen eller startaren, en bildredigerare, knapparna *Testa* i Inställningar: ”Det externa programmet misslyckades”.
- **En AI-begäran** (ett commit-meddelande, en förklaring, ett förslag för en konflikt...): ”AI-begäran misslyckades”.
- **En fil som inte kunde skrivas**: Spara i den inbyggda redigeraren, merge-redigeraren, en inställning som inte kunde sparas: ”Det gick inte att spara”.

Det röda meddelandet ligger på statusraden oavsett vilken vy som är längst fram: en repositoryflik, Hem, Radar, Arbetsytor, Inställningar (på skärmens nedersta rad), under Aktivitetsloggen eller en dialog. Där statusraden är täckt (Hem, Radar, Arbetsytor, Inställningar) skrivs meddelandet över dess nedersta rad tills du har tittat på felet. Det följer med när du byter flik. En åtgärd som fortfarande pågår när du byter flik följs inte: öppna Aktivitetsloggen för att se hur den slutade.

Listorna i sektionerna PULL REQUESTS och ÄRENDEN som inte kan uppdateras behåller i stället en egen rad i sektionen (de uppdaterar sig själva, ett meddelande var några sekunder vore bara brus). En dialog som fortfarande är öppen (Klona, Skapa pull request...) behåller också ett eget meddelande på en rad, så att du kan rätta fältet och försöka igen.

Posten i Aktivitetsloggen innehåller allt som popupen tidigare visade:

- **Rubrik och orsak.** ”Fetch misslyckades”, sedan en mening som med vanliga ord säger varför, sedan vad du kan göra härnäst. Känns igen: remoten har commits som du inte har (*gör en pull först*), inloggningsuppgifter avvisade (*kontrollera Inställningar > Integrationer eller din SSH-nyckel*), ett stopp på konflikter, lokala ändringar som skulle skrivas över (*committa, stasha eller släng dem*), remoten kan inte nås, och en annan git-process som håller ett lås på repositoryt. Allt annat visar gits egen första rad. En kloning eller ett anrop till värden som misslyckas på samma sätt (inloggningsuppgifter avvisade, värden kan inte nås) formuleras likadant; andra misslyckanden får en mening om vad som misslyckades (”Repositoryt kunde inte klonas.”, ”Programmet kunde inte köras.”).
- **Detaljerna**, som visas när posten är öppen (`Enter`, eller klicket som öppnade den): åtgärden, git-kommandot som kördes, dess avslutskod och de sista raderna av det som skrevs ut (för det inbyggda git-biblioteket dess felklass och kod). Inloggningsuppgifter, token och `Authorization`-huvuden maskeras innan något visas, loggas eller kopieras.
- **Kopiera** (knappen, eller `c`) lägger ett textblock på urklipp, gjort för att klistras in i ett samtal med någon, eller en AI, som kan förklara det: GitPoulpis version, systemet, terminalen, backend, den misslyckade åtgärden med sina detaljer och git-versionen.
- **En knapp för nästa steg**, när det finns ett, bredvid Kopiera (eller `r`): *Pull först* (en push som remoten skulle avvisa), *Stash och försök igen* (lokala ändringar hindrar en checkout, merge, rebase eller pull), *Försök igen* (remoten kunde inte nås), *Öppna Integrationer* (inloggningsuppgifter). Den kör steget och stänger fönstret. För ett misslyckande i ett annat repository än fliken längst fram erbjuds bara *Öppna Integrationer*: ett nytt försök skulle köras i fel repository.
- Utan mus har kommandopaletten `Aktivitetslogg: Öppna det senaste felet`, `Aktivitetslogg: Kopiera det senaste felet` och `Aktivitetslogg: Kör den föreslagna åtgärden` (postens knapp för åtgärden, för felet som statusraden visar).

En push som remoten avvisade för att den hade gått vidare behåller sin egen dialogruta (pull först, force push med lease, avbryt); fakta finns ändå i aktivitetsloggen.

### Aktivitetsloggen

Knappen **Aktivitet** längst till höger i statusraden (ett tal på den räknar de fel du ännu inte har tittat på), `Ctrl+N` och sedan `Shift+A`, och `Visa: Aktivitetslogg` i paletten öppnar fönstret. `Ctrl+N` visar ett märke på varje flik, ruta, fält, kryssruta och knapp i fönstret (och på listans första post); att trycka på tecknet gör vad ett klick på det gör. Två flikar, **Git-åtgärder** och **Program**, som du byter med `Tab`, ett klick, `g` (Git-åtgärder) eller `p` (Program), och en ruta **Omfång** under dem som gäller båda (se nedan).

- **Program**: programmets egna loggrader, fönstrets och bakgrundsdaemonens, nyast först. Rutan **Nivå** (en rullgardinslista: Fel, Varning, Info, Felsökning, Spårning; `l` öppnar den) säger hur mycket som visas; att be om mer detalj än vad som spelas in höjer också inspelningsnivån (och sparar den som `log.level`). Under den har **Visa** en ruta per nivå (Fel, Varning, Info, Felsökning, Spårning, var och en med hur många rader den innehåller) för att dölja eller visa raderna på den nivån, till exempel för att bara se felen, och **Alla** / **Ingen** kryssar i eller ur varje ruta. Tangenterna `1` till `5` växlar nivåerna i den ordningen, `a` är Alla, `n` är Ingen; paletten har `Aktivitetslogg: visa eller dölj nivå ...`, `Aktivitetslogg: visa alla nivåer` och `Aktivitetslogg: visa ingen nivå`. Nivåer som är mer utförliga än valet i rutan Nivå är nedtonade (inget spelas in för dem; håll muspekaren över en för att se varför). Kryssen kommer ihåg tills du avslutar, och bara fliken Program har dem. **Filter** (`/`) behåller raderna som innehåller din text. **Kopiera allt (filtrerat)** kopierar det som visas (kryssen och filtret gäller båda); **Öppna loggmappen** öppnar mappen i din filhanterare. **Rensa loggen** (`x`, eller palettkommandot `Aktivitetslogg: Rensa programloggen`) frågar först och tömmer sedan loggfilerna för alla sparade dagar och listan på skärmen; det går inte att ångra och står inte i ångrahistoriken. En lång rad kapas aldrig: se "Långa rader" nedan.

  ![Aktivitetsloggen, fliken Program: rutan Nivå, rutorna Visa per nivå med Alla och Ingen, filtret och loggraderna](../images/activity-log.png)

- **Git-åtgärder** (fliken som fönstret öppnas på): åtgärderna, nyast först, var och en med tiden då den kördes och hur lång tid den tog (`Push - 123 ms`): öppna repositoryt, läsa in commit-grafen, fetch, pull, push, commit, checkout, merge, rebase, stash, klona. Misslyckanden, statusmeddelanden, en kloning som misslyckades och en arbetsytas Fetch all listas också här; `Enter` eller ett klick öppnar en (dess orsak, nästa steg, kommando, avslutskod och utdata), `c` eller dess knapp Kopiera kopierar den, `r` eller dess knapp kör det föreslagna steget. `Tab` (eller `p`) byter till fliken Program; `Left`/`Right` rullar långa rader i sidled.

  ![Aktivitetsloggen, fliken Git-åtgärder: varje åtgärd med sin tid och längd, en misslyckad fetch i rött](../images/activity-log-repository.png)

- **Vilken åtgärd som gav ett fel.** Varje misslyckande säger vad som gav det, i posten, i Programloggen och i den kopierade rapporten. Åtgärden namnges som en person skulle göra det (`Remove remote`, `Fetch all remotes`, `Hosting request: create pull request`, `Token command`) med sina huvudargument (`remote=cm6`, en gren, en sökväg; aldrig en meddelandetext, och URL:er och tokens maskeras som överallt annars), sedan repositoryt, vem som bad om den (`TUI`, `CLI` eller `MCP`) och åtgärds-id:t, numret som daemonen gav begäran (och dess jobbnummer för en fetch, push eller pull). På fliken Git-åtgärder lyder postens rubrik `Remove remote failed (remote=cm6) - ... [CLI #12]` och dess detaljer börjar med `Åtgärd:`, `Repository:`, `Begärd av:` och `Åtgärds-id:`; en åtgärd utan egen titel får som titel vad den var (`Remove remote failed`). På fliken Program börjar misslyckanderaden med samma ord och en tagg: `Remove remote failed (remote=cm6) [op #12, CLI, repo name]: backend error: no such remote: cm6`. En sådan rad öppnar sin åtgärd: klicka på den, eller markera den och tryck `Enter` (paletten: `Aktivitetslogg: Visa radens åtgärd`). `]` och `[` markerar nästa och föregående misslyckanderad (`Aktivitetslogg: Nästa misslyckade åtgärd` / `Aktivitetslogg: Föregående misslyckade åtgärd`); med en rad markerad flyttar piltangenterna markeringen i stället för att rulla. Posten som öppnas är fönstrets egen när fönstret körde åtgärden; för ett misslyckande i CLI eller hos en agent mot samma session gör fönstret en post av raden. Den kopierade felrapporten har raderna `operation:`, `arguments:`, `repository:`, `requested by:` och `operation id:`, så den kan klistras in för sig. Från ett skript är ett misslyckat kommandos `--json`-fel oförändrat; daemonens svar på en misslyckad begäran bär samma fakta i ett tillagt fält `context`.

- **Långa rader** (båda flikarna): en rubrik eller en detaljrad som är bredare än fönstret kapas aldrig. En vågrät rullningslist visas under listan; rulla i sidled med `Left` / `Right` (kan bindas om som `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + hjulet eller ett vågrätt hjul, palettkommandona `Aktivitetslogg: Rulla åt vänster` / `Aktivitetslogg: Rulla åt höger`, eller listen (dra handtaget, eller tryck på banan för att få handtaget under pekaren). `Home` går tillbaka till början, och när du byter flik börjar den andra fliken vid vänsterkanten. Radbruten text (en sammanfattning, en föreslagen åtgärd) förblir radbruten.

- **Omfång** (en rullgardinslista under flikarna, `s` öppnar den, eller palettkommandot `Aktivitetslogg: Välj omfång`): vad båda flikarna handlar om. Raderna är **Det här repositoryt (namn)** (repositoryt för fliken du öppnade loggen från; erbjuds inte från Hem, som inte har något), **Alla repositories**, varje **arbetsyta** och varje repository som fönstret känner till. Från en repositoryflik öppnas loggen på *Det här repositoryt*, från Hem (knappen Aktivitet, `Ctrl+N` och sedan `Shift+A`) på *Alla repositories*; `Ctrl+N` och sedan `Shift+G` och palettkommandot `Visa: Global aktivitetslogg` öppnar alltid på *Alla repositories*, och det gör också ett klick på det röda felet för ett misslyckande vars repository inte är fliken längst fram (en misslyckad kloning, till exempel). Ditt senaste val kommer ihåg under sessionen, separat för de två sätten att öppna den. På fliken Program hör en rad till ett repository när programmet skrev den om ett (en misslyckad push, en åtgärds tid); under *Alla repositories* anger varje sådan rad vems den är (`[namn]`). En rad som inte handlar om något repository alls (uppstart, inställningar, daemonen själv) och en post som inte hör till något (en misslyckad sparning) är **hela appen**: de visas under varje omfång, märkta `[hela appen]` när omfånget är snävare än *Alla repositories*, så ett filter döljer aldrig ett misslyckande. Paletten har också `Aktivitetslogg: Git-åtgärder` och `Aktivitetslogg: Programlogg` för flikarna.

- **Kopiera diagnostik** (en knapp på båda flikarna, `d`, alltid hela loggen oavsett omfång; och `Visa: Kopiera diagnostik` i paletten) kopierar version, system, terminal och backend, det senaste misslyckandet med sina detaljer och de senaste 200 loggraderna på den valda nivån: blocket att ge en AI när något är fel.

Tiderna mäts av fönstret (från begäran till daemonens svar, vägen fram och tillbaka inräknad), så det som CLI:t eller MCP gör tidtas inte. `log.timings` (på som standard) stänger av tiderna, `log.extended` (av som standard) listar också bakgrundsarbetet, den periodiska uppdateringen och auto-fetch; båda finns i Inställningar > Allmänt bredvid de andra loggininställningarna, och varje åtgärd skrivs ändå till programloggen (`info` för det du bad om, `debug` för bakgrunden).

Loggfiler heter `gitpoulpi-YYYY-MM-DD.log`, sparas en vecka som standard och högst tio år (`log.retention_days`, 1 till 3650), rensas när GitPoulpi startar och roteras vid `log.max_file_mb` (5 MiB). Var de finns: [konfigurationsreferensen](config-reference.md#log-programloggen). Från ett skript bär ett misslyckat kommandos `--json`-fel samma fakta under `detail`; se [CLI-referensen](cli-reference.md).

Ett klick på det röda meddelandet öppnar loggen på *Alla repositories* med den posten öppen. Listan Git-åtgärder gäller sessionen; fliken Program behåller programmets logg över omstarter.

**Bakgrundsjobb.** Fetch, push, pull, kloning eller LFS-överföring, en arbetsytas Fetch all / Pull all / Clone missing, en AI-förfrågan och en pull request eller ett ärende som skickas är jobb: högra kanten av statusraden visar det som pågår med ett `[x]` som avbryter det, och ett klick på etiketten (eller `Ctrl+N` och sedan `Shift+J`, kommandot `Jobb: visa bakgrundsjobben`) öppnar panelen med en Avbryt-knapp per jobb (även `c` eller `Del`). En kloning körs i bakgrunden; en AI-förfrågan och en genomgång av arbetsytan överlever ett flikbyte. Ett jobb bortom punkten där det inte längre är säkert att stoppa är nedtonat med orsaken; ett avbrutet jobb lämnar repositoryt som det var.

## Snabba svar

**Verktygsfältet och sidofältet visar rutor eller frågetecken i stället för
ikoner.** Använd ett Nerd Font eller något typsnitt med emoji-stöd i din
terminal. Grafen och texten fungerar ändå utan dem.

**Commit-grafen består av tecken, inte bilder.** Det är den automatiska
reserven: bilder kräver en terminal som talar kittys grafikprotokoll (kitty,
Ghostty, herdr med kitty-grafik på) eller iTerm2-protokollet (WezTerm, Rio,
mintty) eller sixel (Windows Terminal, foot, Tabby, Konsole, och tmux när terminalen bakom tar sixel). Windows-konsolen, zellij och screen får tecken. Se avsnittet om
grafen nedan för att tvinga fram det ena eller andra, [Välja terminal](terminals.md) och [Multiplexers](multiplexers.md).

**Musen gör ingenting, eller det går inte att markera text med musen.**
GitPoulpi fångar musen så att klicken når det. För att markera text med själva
terminalen, håll ned `Shift` medan du drar (i de flesta terminaler). Du kan
också markera inuti diffen (dra, sedan `y`). För att stänga av musfångst, använd
*Inställningar > Tangentbord & mus > Mus aktiverad* (kräver omstart).
Tangentbordet når varje kommando.

**En genväg fungerar inte.** Din terminal, tmux eller ett AutoHotkey-skript kan
ta tangenten innan GitPoulpi ser den (`Ctrl+,` kommer ofta fram som ett vanligt
`,` i Linux- och macOS-terminaler; `Ctrl+Shift+N` ser ut som `Ctrl+N`). Använd
alternativen (`Ctrl+N` och sedan `,` öppnar Inställningar; `Ctrl+P` når varje
kommando), eller bind om tangentkombinationen: se
[Inställningar](settings.md#tangentbord--mus).

**Jag kommer inte ihåg en genväg.** Tryck på `Ctrl+P` och skriv vad du vill;
varje resultat visar sin genväg. `?` listar allt.

**Ett alternativ är nedtonat i Inställningar.** Det beror på ett annat
alternativ. Raden säger vilket (till exempel kräver *Styrka på radfärgning* att
*Bakgrund för grafspår* är satt till *Radfärg*).

**”Auto-stash väntar” över Log.** GitPoulpi stashade dina ändringar för att köra
en operation och kunde inte lägga tillbaka dem. Använd bannerns **Poppa nu**,
**Visa stash**, **Börja om** (ångra operationen och lägg tillbaka dina ändringar)
eller **Drop**; se [Auto-stash](log-and-history.md#auto-stash).

**En agent säger `no_viewer` eller `deferred`.** `no_viewer`: inget
GitPoulpi-fönster är öppet för det repositoryt (eller *Låt AI-agenter styra
vyn* är av). `deferred`: fönstret visar ett chip; `Ctrl+N Enter` godkänner det.
Se [MCP och agenter](mcp-and-agents.md#att-visa-saker-för-användaren).

**Jag öppnade GitPoulpi och ser ett tomt fönster eller `log error: backend
error`.** Kontrollera att du startade det i ett Git-repository (annars hamnar du
på fliken Hem: använd **Bläddra**, **Klona** eller **Init**). Kör sedan
`gitpoulpi daemon status`; `gitpoulpi daemon restart` startar om
bakgrundsprocessen för det här repositoryt. Tryck på `r` för att uppdatera.

**Två fönster på samma repository beter sig konstigt.** De delar medvetet en
daemon per repository, så en ändring i det ena syns i det andra. Om en daemon
verkar hänga stoppar `gitpoulpi daemon stop --all` varje sessions daemon; nästa
fönster startar en ny.

**En commit, pull eller push frågar efter ett lösenord, eller misslyckas med att
autentisera.** GitPoulpi använder din `git`s egen credential helper och ssh-agent
för nätverket. Se först till att `git fetch` fungerar i en vanlig terminal.
Tokens för GitHub/GitLab till pull requests är separata och valfria: se
[Värdtjänster och ärendehantering](hosting-integration.md).

**En fetch, pull eller push misslyckas med ”Den konfigurerade credential helpern misslyckades”, ”SSH-värdnyckeln för remoten är ännu inte känd” eller en avvisad inloggning, och dess post i Aktivitetsloggen har en knapp ”SSH & inloggning...”.**
Misslyckandet kommer från hur det här repositoryt loggar in. Knappen öppnar
Inställningar > SSH & inloggning, där du väljer en annan källa för det här
repositoryt: gits egna helpers med repositoryts helpers ignorerade, ett
integrationskonto, en nyckelfil (med en known hosts-fil), eller en egen helper
eller ett eget tokenkommando; **Testa anslutning** kontrollerar sedan varje
remote. För en helper som inte kunde köras visar `git config --show-origin
--get-all credential.helper` var den är satt.

**”GitPoulpi skickade inte kontots token till ...”.** Repositoryts remote pekar
på en server som inte är den adress det valda kontot är bundet till (ofta en
`url.<x>.insteadOf`-omskrivning i repositoryts egen config), så token hölls
tillbaka. Rätta remoten eller välj en annan källa på sidan.

**”Det här repositoryts tokenkommando misslyckades”, ”... har ändrats sedan du bekräftade det”, ”Det här repositoryts inloggning kan inte användas”.**
Sidans val kan inte genomföras: tokenkommandot skrev inte ut någon token
(dess sista felrader citeras, med hemligheter borttagna), ett skript det kör
har ändrats sedan du bekräftade det (bekräfta kommandot igen på sidan), eller
valet pekar på en nyckelfil eller ett konto som inte längre finns. Varje
förklaring har knappen **SSH & inloggning...**.

**OpenSSH säger ”bad permissions” för en nyckelfil.** Andra konton kan läsa
nyckeln. Under Inställningar > SSH & inloggning visar raden **Privat nyckelfil**
en varning med **Begränsa till mitt konto**.

**”stagear...” blir kvar på statusraden.** En bakgrundsuppdatering körs fortfarande;
den försvinner när fillistan uppdateras. `r` tvingar fram en uppdatering.

**Ett mycket stort repository är långsamt.** Sänk *Första commits i grafen* på
sidan Inställningar > Log & graf (Log läser in fler medan du rullar när *Lazy
loading av commits i grafen* är på); dölj Logs kolumn *Ändringar* och stäng av
avatarer; stäng av *Ångra aktiverat* om du inte behöver ångra (då förs ingen
bokföring alls vid staging och commits). `GITPOULPI_TIMING=1` skriver tider per
begäran till `gitpoulpi-timing.log` i temp-mappen, värt att bifoga i en
prestandarapport.

**AI-knapparna saknas eller är nedtonade.** AI är av som standard; se
[AI-hjälp](ai.md#felsökning).

**GitPoulpi stängdes av sig självt.** Varje avslut loggas: leta efter
`gitpoulpi-tui-exit.log` (och `gitpoulpi-tui-panic.txt` efter en krasch) i
konfigurationsmappen (`gitpoulpi config path` skriver ut inställningsfilen, som
ligger där), och bifoga den i ett ärende.

**Var finns mina inställningar?** `gitpoulpi config path`. Tillstånd per
repository ligger i `.gitpoulpi/repo-state.toml` inuti repositoryt.

**Skickar GitPoulpi något någonstans?** Bara det du ber om: git-operationer till
dina remotes, hosting-förfrågningar efter att du kopplat ett konto, AI-förfrågningar
efter att du tryckt på en AI-knapp och, om du slår på avatarer, en hash av
författarnas e-postadresser till Gravatar. Det finns ingen telemetri. Se
[AI-hjälp](ai.md#integritet).

## Varje skrivning misslyckas: ”Unable to create index.lock”

Ett git-kommando som dog (en krasch, en dödad terminal) kan lämna kvar
`.git/index.lock`, och då misslyckas varje ändring i repositoryt. När filen är
äldre än 10 sekunder visar GitPoulpi en remsa ovanför Log: **Ta bort
index.lock** (palett: *Repository: Ta bort föråldrad index.lock*; CLI:
`gitpoulpi index-lock remove`). Den frågar först och tar bara bort den filen.
GitPoulpi kan inte på ett pålitligt sätt avgöra om ett annat git-program
fortfarande körs, så se till att inget gör det (en IDE, en annan terminal) innan
du bekräftar. I en länkad worktree är låset worktreens eget, i
`.git/worktrees/<name>`.

## ”git am” stannade och Rebase Abort gör ingenting

En `am` (tillämpa patchar från en brevlåda) är inte en rebase. En stannad
`git am` har sin egen remsa, **Tillämpar patchar (git am)**, med Lös konflikter...,
Fortsätt, Hoppa över och Avbryt (palett: *git am: Fortsätt / Hoppa över den här
patchen / Avbryt*; CLI: `conflicts am-continue|am-skip|am-abort`). Avbryt
frågar först, kastar patcharna som hittills tillämpats och sätter tillbaka
grenen där den var. De går inte att ångra, precis som rebase-knapparna.

## Ångra är nedtonat och säger ”Kan inte ångra ...” (blockerad post)

Den senaste journalposten har fastnat: repositoryt gick vidare utanför
GitPoulpi, eller så avbröts posten. Peka på Ångra för orsaken; om den har
fastnat för gott pekar verktygstipset på **Redigera: Glöm blockerad ångrapost**
(palett) eller `gitpoulpi undo-forget`. Den frågar först och tar bara bort posten
från journalen; repositoryt rörs inte. Se [Ångra och gör om](undo-redo.md).

## Stora filer visas som tre rader text (”LFS files show as pointers”)

En fil som borde vara en bild, en modell eller en stor binärfil innehåller
`version https://git-lfs.github.com/spec/v1` och en rad `oid sha256:...`: den är en
Git LFS-**pekare**, och det riktiga innehållet har inte hämtats (eller så är
LFS-filtren inte installerade).

- Titta i Status-rutan: ett repository som använder LFS medan `git-lfs`
  saknas visar en gul banner. Installera programmet
  (https://git-lfs.com; Windows: ingår i Git för Windows eller
  `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS:
  `brew install git-lfs`) och öppna om eller tryck på `r`.
- `gitpoulpi lfs status` säger om programmet är installerat, om filtren och
  kroken är uppsatta och vilka mönster som spåras; `gitpoulpi lfs files` listar
  pekarfilerna och vilka som är `local`.
- Med `git-lfs` installerat, kör **`LFS: Pulla objekt`** i paletten (eller
  `gitpoulpi lfs pull`): det laddar ned objekten och ersätter pekarna. Det
  installerar först LFS-filtren för det här repositoryt om det inte finns några.
- Om nedladdningen misslyckas namnger meddelandet orsaken (inloggningsuppgifter
  för remoten, en server som inte längre har objektet). GitPoulpi behöver inget
  eget konto: LFS använder remotens vanliga inloggningsuppgifter (credential
  helper eller ssh-agent).
- En kloning av ett LFS-repository misslyckas aldrig på grund av LFS; om
  nedladdningssteget inte kunde köras finns klonen där med pekare, och `LFS:
  Pulla objekt` slutför den senare. `gitpoulpi clone --no-lfs` hoppar över
  nedladdningen med avsikt.
- Du kan stänga av allt som rör LFS i Inställningar > Git & kodning > Git LFS
  (eller `[git] lfs_enabled = false`).

## ”git-lfs not found” / ”Git LFS is not installed”

Meddelandet `Git LFS is not installed. This repository stores large files with Git
LFS; ...` (eller en dialogruta med en knapp *Öppna installationssidan*, eller gits
eget `git-lfs: command not found` / `'lfs' is not a git command`) betyder att
repositoryt är konfigurerat för LFS - dess `.gitattributes`, `.git/config`-filter
eller `pre-push`-krok anropar `git-lfs` - och programmet finns inte på den `PATH`
som GitPoulpi ser. Installera det (länk ovan). På Windows medföljer programmet Git
för Windows; om `git lfs version` fungerar i en terminal men GitPoulpi ändå säger
att det saknas, starta GitPoulpi från en terminal som ser samma `PATH`. För att
dölja LFS för GitPoulpi utan att röra gits eget beteende, stäng av
`git.lfs_enabled`.

## Grafen ritas med tecken, inte som en bild (herdr, tmux, andra terminaler)

GitPoulpi ritar kolumnen Graf och författaravatarer som pixlar bara där det är
säkert att terminalen visar dem: iTerm2-inlinebilder i ren WezTerm, Rio, mintty och iTerm2 äldre än 3.7.7; Kitty-grafikprotokollet i kitty, Ghostty, iTerm2 3.7.7 eller nyare och **herdr när herdrs egen
konfiguration sätter `kitty_graphics = true`** (`[terminal]`, eller
`[experimental]` på äldre herdr) och herdr körs inuti WezTerm, kitty eller
Ghostty. Sixel-bilder används i Windows Terminal, foot, Tabby, Konsole och mlterm (och i tmux när terminalen bakom tar sixel). Överallt annars ritar det tecken, vilket alltid är korrekt.

- I herdr med Kitty-grafik på men en konfiguration som GitPoulpi inte hittar (eller
  som förlitar sig på herdrs standard), sätt `ui.image_protocol = "kitty"`
  (Inställningar > Log & graf > Bildprotokoll) och starta om.
- I en terminal som visar rutor eller skräp där grafen ska vara, sätt
  `ui.image_protocol = "none"` eller `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` stänger av bilder oavsett konfigurationen.

**herdr på distans (`herdr --remote <host>`, eller `ssh` och sedan `herdr`):
textgrafen är standard.** Över SSH körs GitPoulpi på fjärrvärden och kan bara
läsa fjärrvärdens miljö och herdrs `config.toml`. Om en Kitty-bild visas avgörs av
den *lokala* herdr-klienten (dess egen `kitty_graphics`-inställning och dess egen
terminal), som GitPoulpi inte kan se: när klienten inte ritar bilderna förblir
cellerna tomma. GitPoulpi ritar därför textgrafen så fort det upptäcker herdr över
SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Om din lokala herdr har
`kitty_graphics = true` och en terminal som ritar Kitty-bilder (kitty, Ghostty),
välj det med `ui.graph_renderer = "image"` (eller `ui.image_protocol = "kitty"`) på
fjärrvärden. Bilder skickas inline (`t=d`), vilket fungerar över vilken bytström
som helst. Om kolumnen är tom, sätt `ui.graph_renderer = "text"`.

**En ljusare rektangel bakom spåren.** Bilden är genomskinlig (RGBA-PNG med rak
alfa) överallt där inget ritas, så terminalens bakgrund och markeringsbandet
syns igenom. Terminaler med Kitty-protokollet, **WezTerm** och **Rio** komponerar den
alfan; GitPoulpi reserverar då vanliga tomma celler under bilden och skickar den
efter dem. Varje annan terminal med iTerm2-protokollet (äldre iTerm2, mintty)
är inte känd för att blanda PNG-alfa, så där har bilden en ogenomskinlig
bakgrund i temats färg, vilket kan synas som en rektangel i en terminal vars
bakgrund skiljer sig från den; `ui.graph_renderer = "text"` tar bort den. Med
`ui.graph_lane_background = "off"` målas inget bakom spåren på de genomskinliga
vägarna.

**Windows: grafkolumnen är tom, ligger på fel rader, eller skärmen rivs (WezTerm 20240203, Rio, mintty).** Pseudokonsolen bakom de terminalerna tappar eller felplacerar bildsekvenserna ([Välja terminal](terminals.md#windows)). Sätt `ui.graph_renderer = "text"`, eller använd Windows Terminal.

I tmux, zellij, screen och herdr, se [Multiplexers](multiplexers.md) för vad som släpps igenom och konfigurationen att lägga till.

Se `ui.graph_renderer` och `ui.image_protocol` i
[konfigurationsreferensen](config-reference.md).
