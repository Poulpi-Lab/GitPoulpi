[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · **Svenska** · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage och commit

Hur du ser vad du har ändrat, väljer exakt vad som ska med i en commit, skriver
commiten och tittar närmare på filer (blame, historik, den inbyggda
redigeraren). Termer som *WIP-rad* och *hunk* förklaras i
[rundturen i gränssnittet](interface-tour.md).

## Lokala ändringar

Ocommittat arbete är **första raden i Log** (streckad cirkel, `// WIP`, med
räknarna `✎ n` ändrade och `+ n` tillagda). Markera den så visar högerpanelen
de lokala ändringarna:

![Lokala ändringar: Unstaged- och Staged-filer till höger, med commit-formuläret under](../images/hero.png)

- **Unstaged-filer**: redigerade, tillagda (ospårade) och borttagna filer som
  ännu inte är med i nästa commit.
- **Staged-filer**: det nästa commit kommer att innehålla.
- **Filer som bytt namn eller flyttats** är en enda rad, `nytt <- gammalt`, i båda listorna (en fil som raderats på
  disk och en ny med minst 50 % samma innehåll räknas som en flytt redan innan du stagear dem). Stage, Unstage och
  Förkasta verkar på båda namnen: att stagea stagear borttagningen och den nya filen, så git ser en enda namnändring.
  Att öppna raden visar innehållsdiffen mot det gamla namnet, med rubriken `gammalt -> nytt (N % lika)`; en flytt
  utan ändringar säger **Ingen ändring av innehållet**. Detsamma gäller en commits filer, och `gitpoulpi status` /
  `gitpoulpi diff` rapporterar `renamed_from`.
- **Sökväg** eller **Träd** (`m`) listar filer platt eller i mappar. **Fäll
  ihop alla** (`z`) fäller ihop mapparna. **Visa alla filer** (`a`) visar hela
  trädet i stället för bara de ändrade filerna. Högerklicka på listans tomma yta för
  **Fäll ut alla** / **Fäll ihop alla**, på en mapprad för **Fäll ut alla undermappar** /
  **Fäll ihop alla undermappar**; tangenterna `=`, `-`, `*`, `_` (se
  [Mus och gester](mouse-and-gestures.md)).
- Rubriken säger hur många filer som ändrats på vilken gren.

### Stagea, unstagea, förkasta

| För att | Gör så här |
|---|---|
| Stagea en fil | Håll pekaren över dess rad och klicka på **Stage fil**, eller markera den och tryck `Ctrl+Space`, eller `Ctrl+G s` |
| Stagea en mapp | **Stage mapp** på mappraden |
| Stagea allt | **Stage alla ändringar** i avsnittsrubriken, eller `Ctrl+G a` |
| Unstagea | Samma knappar och tangenter i avsnittet Staged (`Ctrl+G u`) |
| Förkasta en fils ändringar | Högerklicka på filen > **Förkasta ändringar...** (frågar först; `Ctrl+Z` tar tillbaka ändringarna) |

### Känsliga filer

Att stagea en fil, en mapp eller allt kontrollerar först vad det skulle lägga till. En fil vars
namn matchar `[privacy] sensitive_patterns` (`.env` och `.env.*` utom filerna
`.example`, `.sample` och `.template`, `*.pem`, `*.key`, privata SSH-nycklar,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), eller
vars text innehåller en hemlighet som skannern känner igen, stageas inte. Frågan **Känsliga
filer** listar varje fil med sin orsak (ett mönster, eller slaget av hemlighet
med en maskerad förhandsvisning, aldrig värdet) och erbjuder **1. Stage ändå** eller **2. Avbryt,
stage ingenting**. Inget svar är markerat förrän du trycker på en piltangent eller en siffra,
så ett förlupet Enter svarar ingenting; `Esc`, stängningskorset och ett klick utanför avbryter.
Det är samma fråga från tangentbordet, en knapp, en meny, paletten eller en
drag och släpp, eftersom de alla skickar samma begäran. Att avbryta stagear ingenting,
inte ens de vanliga filerna i samma stage.
En hemlighet som hittats i en fil anger sin rad (`config/app.env:12`). För att titta innan du bestämmer dig, klicka på
ett fynd (eller gå mellan dem med `Tab` / `Shift+Tab`, `Ctrl+J` / `Ctrl+K`) och tryck på knappen **Visa i filen**, `S`,
eller paletten **Känsliga filer: Visa i filen**: filen öppnas i filvyn rullad till den raden med raden markerad, och
frågan kommer tillbaka orörd när du stänger filvyn (`Esc`). Att titta stagear ingenting.

Textfiler upp till 1 MiB var och 32 MiB totalt läses; större filer och binärfiler
bedöms bara efter namn. `[privacy] staging_guard = false` stänger av kontrollen.
Att stagea en hunk eller några rader kontrolleras inte: du arbetar redan med den filen.

Högerklicka på en fil för dess meny: stage eller unstage, förkasta, kopiera
sökvägen, filhistorik och blame, öppna i din externa redigerare, ditt
diff-verktyg eller din filhanterare (ställs in i
[Inställningar > Diff & redigerare](settings.md#diff--redigerare)), och mer.

## Ignorera filer och behandla dem som binära eller som text

Högerklicka på en fil > **Ignorera** för att ignorera den eller dess filändelse, eller på en mapprad > **Ignorera alla filer i mappen**. GitPoulpi frågar först vilken fil regeln ska skrivas i: repositoryts `.gitignore` (standard), `.gitignore` i varje mapp ner till filens egen, eller `.git/info/exclude` (endast den här klonen, aldrig committad). Filer som inte finns än markeras som *ny* och skapas när du bekräftar. Raden skrivs relativt den valda filen: `src/a/b.txt` blir `/a/b.txt` i `src/.gitignore`. Flytta med piltangenterna, skriv för att filtrera listan, `Enter` skriver, `Esc` avbryter. `Ctrl+Z` ångrar en skrivning till en `.gitignore`; `.git/info/exclude` ligger utanför arbetskatalogen och kan inte ångras.

![Listan som frågar i vilken fil en ignoreringsregel hamnar: repositoryts .gitignore, .gitignore i filens mapp eller .git/info/exclude, var och en med raden som skulle skrivas](../images/ignore-picker.png)

Högerklicka på en fil > **Behandla som binär** (eller **Behandla som text** när git redan behandlar den som binär) skriver en `.gitattributes`-regel: `binary` eller `text diff`. Välj själva filen eller alla filer med dess filändelse, och sedan vilken `.gitattributes` som ska skrivas, med samma lista som ovan (och `.git/info/attributes` endast för den här klonen). En regel som redan skrivits för samma mönster ersätts och motsägs aldrig. Filens diff uppdateras direkt och statusraden säger vad som skrevs var (och varnar om en annan regel fortfarande har företräde). En mapprad har **Behandla filer i mappen som** > **Binär** eller **Text**. Samma finns i kommandopaletten (**Lokala ändringar: Behandla vald fil som binär...**) och på kommandoraden (`gitpoulpi treat binary|text <sökvägar>`). Filvyn följer också regeln: en fil du behandlar som text visar sina rader även om den innehåller en NUL-byte, och en fil som git diffar som binär visar binärmeddelandet.

## Menyer, flerval och konvertering i fillistorna

Fillistorna i högerpanelen (Unstaged, Staged, en commits ändrade filer och **Visa alla filer**) har samma menyer och gester. Högerklicka på en fil, en mapprad eller den tomma delen av en lista, eller tryck `.` (`Shift+F10`, Meny-tangenten) på raden med fokus, eller kör **Filer: Öppna menyn för raden eller markeringen** från paletten.

- **En fil i en commit** (och en fil i **Visa alla filer**): **Öppna diff**, **Öppna fil**, **Filhistorik**, **Fil-blame**, öppna i din redigerare, i standardprogrammet eller i **Öppna med...**, **Visa i mapp**, **Redigera fil**, **Kopiera filsökväg**, **Kopiera fullständig filsökväg**, **Ignorera**, **Behandla som binär** eller **Behandla som text**, **Konvertera**, **Återställ filen från denna commit** (frågar först; liksom `git checkout <commit> -- <fil>` stageas filen också, och `Ctrl+Z` ger tillbaka den förra filen) och **Ta bort fil...**. En rad som gäller arbetskatalogen är nedtonad, med orsak, när filen inte finns i arbetskatalogen. **Diff mot arbetskatalogen** öppnar diffen mellan versionen i commiten och filen på disk. **Skapa patch från filändringar...** skriver en patch av filens ändring i commiten (ett urval och en mapprad har den också; i **Visa alla filer** på de lokala ändringarna skapar den en patch av filens ändringar i arbetskatalogen).
- **En mapprad**: **Öppna mapp**, **Kopiera filsökväg**, **Ignorera alla filer i mappen**, **Behandla filer i mappen som**, **Konvertera**, **Skapa en fil i den här mappen**.
- **Den tomma delen av en lista**: **Skapa fil** och **Öppna repositoryts mapp**. Listan Unstaged lägger till **Stage alla ändringar** och **Förkasta alla ändringar...**, listan Staged **Unstage alla ändringar**.

![Högerklicksmenyn för en fil i en commit: öppna diffen, filhistoriken och blame, öppna i en redigerare eller standardprogrammet, kopiera sökvägar, ignorera, konvertera, återställ från committen och ta bort](../images/commit-file-menu.png)

### Markera flera filer

Gesterna är desamma som i Log. **Ctrl+klick** lägger till en rad i markeringen eller tar bort den. **Shift+klick** (**Alt+klick** i terminaler som behåller Shift+klick) markerar intervallet från ankaret. Ett vanligt klick börjar om och öppnar diffen som förut. `Esc` rensar markeringen. Med tangentbordet växlar `Space` eller `Insert` den aktuella raden i varje lista (i Unstaged och Staged stagear `Ctrl+Space` markeringen, eller den aktuella raden) och `Shift+Up` / `Shift+Down` utökar intervallet. En mapprad markerar alla filer under den, hopfälld eller inte. Markerade rader är färgmarkerade och statusraden räknar dem. `Ctrl+A` markerar alla rader i den aktiva listan.

Högerklicka på en markerad rad (eller tryck `.`) för menyn för hela markeringen, med rubriken **N filer**: stage eller unstage, förkasta, ignorera, behandla som binär eller text, konvertera, kopiera sökvägar, återställ från commiten, ta bort. Stage, unstage och återställning är en åtgärd vardera; förkasta och ta bort frågar först och anger hur många filer. Ett högerklick på en rad utanför markeringen börjar om markeringen med den raden.

![Fyra filer markerade i listan Unstaged-filer, med menyn med rubriken 4 filer: stage, förkasta, ignorera, behandla som binär eller text, konvertera, kopiera sökvägarna och ta bort](../images/file-pane-multiselect.png)

### Konvertera teckenkodning, radslut och indrag

**Konvertera** finns i menyn för en fil, en mapp eller en markering. `gitpoulpi convert` gör detsamma från kommandoraden.

- **Teckenkodning**: UTF-8, UTF-8 BOM, UTF-16 LE och BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Radslut**: LF, CRLF eller CR.
- **Indrag**: inledande blanksteg som mellanslag eller tabbar, med tabbens bredd.

Filerna i arbetskatalogen skrivs om i bakgrunden med samma funktioner som redigerarens statusrad. En fil som inte är giltig UTF-8 och saknar byteordningsmärke läses som Windows-1252, precis som i redigeraren. Binärfiler, saknade filer och filer som målkodningen inte kan återge hoppas över och rapporteras (*12 filer konverterade, 3 hoppades över (3 binära)*) och skadas aldrig; en fil som inte kan skrivas ger ett felmeddelande. Hela omgången är en post i journalen för Ångra: ett `Ctrl+Z` ger tillbaka alla filer. En fil som är öppen i redigeraren blockerar konverteringen tills den stängs.

## Läsa en diff

Klicka på ett filnamn (eller tryck `d` på det) så ersätter dess diff Log i
mitten. `Esc` eller `q` tar tillbaka Log.

![En diff med två hunks, var och en med knapparna Förkasta hunk och Stage hunk](../images/diff.png)

Raden ovanför den byter vy:

- **Inline** visar den enhetliga diffen; **Split** lägger före och efter sida vid
  sida (`m` växlar mellan layouterna).
- **Hunks** (`Shift+H`) visar bara de ändrade hunks med en rad kontext.
  Stäng av det för att se hela filen.
- **WS** (`s`) döljer ändringar som bara skiljer sig i blanksteg.
- **Bryt** (`w`) bryter långa rader i stället för att kapa dem.
- **Karta** (`Shift+M`) visar en minikarta över hela filen bredvid rullningslisten.
  Klicka på den för att hoppa. Andra förekomster av ett markerat ord färgas och
  markeras på rullningslisten.
- `n` och `p` (eller pilarna på raden) hoppar mellan hunks.
- **Auto** (sista knappen, `Shift+C`) är teckenkodningen filen läses i. Klicka på den (eller tryck
  `Shift+C`, eller kör *Diff: Välj teckenkodning...* från paletten) och välj en i listan:
  **Autoavkänning** först, sedan UTF-8, UTF-8 BOM, UTF-16 LE och BE, Windows-1252, ISO-8859-1,
  ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 och EUC-KR (skriv för att
  söka i listan). Filens två sidor läses om, som exakta bytes, i den teckenkodningen, så en
  Latin-1-fil visar sina accenter och en UTF-16-fil (som git kallar binär) visar sina rader. Valet
  kommer ihåg för filens sökväg tills du avslutar, i den här vyn och i
  [granskningen](review.md). Medan en teckenkodning är vald är hunkarna den här vyns egna, så stagea och
  förkasta är avstängda (välj **Autoavkänning** för att kunna stagea igen). En långsam läsning säger *läser som ...* i
  rubriken efter en stund.

![Samma diff i split-layouten, före och efter sida vid sida](../images/diff-split.png)

Du kan markera text med musen (dra, eller dubbelklicka på ett ord) och kopiera
den med `y` eller `Ctrl+C`; bara filtexten kopieras, aldrig radnummer eller
markörerna `+` och `-`.

### Stagea en hunk eller en enskild rad

I en diff av arbetskatalogen har varje hunkrubrik **Stage hunk** och **Förkasta
hunk** (på Staged-sidan, **Unstage hunk**).

- `Space` stagear hunken under markören.
- För att stagea några få rader, tryck `v` för att starta en markering, flytta
  med `j`/`k` och tryck `Enter`. Bara *ändrade* rader räknas.
- `x` förkastar hunken och `Shift+X` de markerade raderna. Båda frågar först, och
  `Ctrl+Z` tar tillbaka ändringarna.
- Högerklicka (eller `.`) på en hunk för dess meny: stage, unstage, förkasta,
  filhistorik, blame, kopiera.

En commits diff är skrivskyddad. En ändrad bild visas som bild; se
[Bilder](image-view.md).

## Skriva commiten

Commit-formuläret ligger nederst i högerpanelen, och dess **Sammanfattning** är också
fältet på raden med lokala ändringar i Log: det du skriver i det ena syns i det andra.

![Commit-formuläret ifyllt med ett AI-skrivet meddelande: sammanfattning med teckenräknare, beskrivning och den gröna commit-knappen](../images/ai-commit-message.png)

- **Sammanfattning** (räknaren visar hur många av 72 tecken som är kvar) och
  **Beskrivning**. `Enter` i sammanfattningen flyttar till beskrivningen, `Enter` där
  lägger till en rad.
- **Commita ändringar i N filer** (den gröna knappen) eller `Ctrl+Enter` committar de
  stageade ändringarna. Om inget är stageat säger knappen det och gör ingenting.
- **Amend föregående commit** (`Shift+A`) viker in de stageade ändringarna och
  det nya meddelandet i den senaste commiten. En commit som redan är pushad
  kommer då att kräva en force push (med lease).
- **Hoppa över hooks** (`Shift+H`, eller **Commit: Växla att hoppa över hooks** i paletten) committar
  utan att köra repositoryts hooks `pre-commit` och `commit-msg`. Det gäller en
  enskild commit; **Inställningar > Repo-specifika inställningar > Commit** gör att det är
  ikryssat från början. Visas när raden har plats.
- `c` i Status-rutan flyttar markören till formuläret. `Esc` lämnar det.
- `Ctrl+G c` och `Ctrl+G Enter` (**Git: Commit...** och **Git: Commita stagade
  ändringar nu** i paletten) är ett och samma kommando, commit-knappen varifrån
  som helst. Med en sammanfattning i formuläret och något stagat committar det
  direkt, tar hänsyn till **Amend föregående commit** och **Hoppa över hooks**,
  och flyttar inte fokus. Annars committar det ingenting: det flyttar markören
  till formulärets sammanfattning (och visar rutan om den var dold eller
  hopfälld) och statusraden säger vilken genväg som committar (som standard
  `Ctrl+Enter`). Om inget är stagat säger det så. Inget skrivs på statusraden.
  Medan en AI-förfrågan fortfarande pågår committar det ingenting och säger det
  (`Esc` avbryter förfrågan). `Ctrl+Z` tar tillbaka commiten som
  vilken annan som helst.
- Per repository kan [Inställningar](settings.md#repo-specifikt-commit) **pusha efter
  varje commit**, låta formuläret börja med en **commit-mall** (och ta bort dess
  `#`-kommentarer ur meddelandet) och göra en merge till en **squash**: formuläret
  öppnas då med gits squash-meddelande.
- Med en värdintegration länkad infogar `Alt+R` i formuläret en
  ärendereferens; se [Värdtjänster och ärendehantering](hosting-integration.md).

Med [AI](ai.md) på skriver knappen **`✦`** till höger om sammanfattningen ett meddelande
utifrån de stageade ändringarna (`Ctrl+G i`), och **Komponera commits med AI**
delar upp ditt arbete i flera commits, se nedan. Texten hamnar i formuläret för
att du ska redigera den; inget committas förrän du trycker på commit-knappen.

**Komponera commits med AI** sitter längst till höger på Amend-raden och krymper med högerpanelen i stället för att
försvinna: hela etiketten, sedan **Komponera med AI**, sedan bara **`✦`**. Den behåller sitt klick, sitt
verktygstips och sin tangent (`Ctrl+G Shift+C`) i varje bredd; etiketterna Amend och Hoppa över hooks viker sig för den,
aldrig tvärtom.

## Commit-composern: dela upp dina ändringar i commits

*(Kräver att AI är påslaget: se [AI-hjälp](ai.md).)* Tryck **Komponera commits
med AI** (eller `Ctrl+G C`, eller högerklicka på raden med lokala ändringar).
Modellen föreslår flera commits, var och en med sina filer; varje ändrad fil
finns i exakt en commit, och en ogiltig plan skickas tillbaka till modellen en
gång. Förslaget öppnas som en granskning där inget har committats ännu:

![Commit-composern: fyra föreslagna commits, var och en utfällbar till sina filer, med reglage för upp, ned, redigera och drop samt knapparna Återställ, Avbryt och Skapa 4 commits](../images/composer.png)

| För att | Gör så här |
|---|---|
| Fälla ut eller ihop en commit | `Left` / `Right`, eller klicka på vinkeln; `*` växlar alla |
| Flytta en commit upp eller ned | `▲` `▼` på dess rad, eller `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Flytta en fil till en annan commit | Dra den på commiten (från listan, eller från listan med lokala ändringar till höger); eller `m` på filen och välj commiten i rullgardinen; eller `Alt+Up` / `Alt+Down` på filen |
| Lägga en fil i en egen commit | `n` på filen, eller knappen `+ egen commit` |
| Skriva om en commit | Pennan, `e`, `Enter` eller ett dubbelklick: redigera meddelandet, `Enter` behåller det, `Esc` förkastar |
| Utelämna en commit | `drop` (`d` eller `Delete`); dess filer förblir ocommittade. `återställ` tar tillbaka den |
| Börja om | **Återställ** (`r`) går tillbaka till modellens plan |
| Skapa dem | **Skapa N commits** (`Ctrl+S`, `Ctrl+Enter` eller `a`) |
| Lämna | **Avbryt** eller `Esc`: inget gjort |

Att skapa commits unstagear först allt och stagear sedan, för varje commit, dess
filer och committar, så din arbetskatalog rörs aldrig. Om ett steg misslyckas
stannar processen och säger hur många commits som gjordes. Hela filer stageas: en
fil som bara var delvis stagead committas hel. Välj vilken profil och vilka
instruktioner composern använder i *Inställningar > AI*; se
[AI-hjälp](ai.md#commit-composern).

## Filvy, Blame och Filhistorik

Vyraden har **Filvy** (filens innehåll), **Diff-vy**, **Blame** och väljaren
`Fil │ Alla` för [commit-kolumnen](interface-tour.md#commit-kolumnen) (dess lista
`Fil` är Filhistoriken); samma finns i en fils högerklicksmeny. De visar filen som den var
vid commiten du kom från, eller som ocommittad (`@ WIP`) när du öppnade den från
de lokala ändringarna.

- **Blame** och **Filhistoriken** är fullständiga vyer i sig: se [Blame](log-and-history.md#blame) och [Filhistorik](log-and-history.md#filhistorik) i *Arbeta med Log*. `Enter` på en blame-hunk eller en historikrad går till den commiten i Log.
- `y` eller `Ctrl+C` kopierar den markerade texten (i Blame, bara koden).
- `Shift+G` hoppar till sista raden. `q` stänger vyn.

## Den inbyggda redigeraren

**Redigera** (`e`) öppnar filens *kopia i arbetskatalogen* i en liten redigerare
över mittrutan: syntaxfärger, radnummer, ångra och gör om, sök och en statusrad
(position, radslut, indrag, språk). Den redigerar alltid arbetskatalogen, även när
du öppnade den från en gammal commits diff.

![Redigeraren som visar en Rust-fil med syntaxmarkering och radnummer; verktygsfältet har Spara, Ångra, Gör om och Sök](../images/editor.png)

| Tangent | Åtgärd |
|---|---|
| `Ctrl+S` | Spara (filen skrivs till en temporär fil och döps om, så den blir aldrig halvskriven; radslut, BOM och det sista radbrytningstecknet bevaras) |
| `Ctrl+Z`, `Ctrl+Y` | Ångra, gör om |
| `Ctrl+F`, `F3` | Sök, sök nästa |
| `Tab`, `Shift+Tab` | Dra in, dra ut (efter filens egen stil) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Markera allt, kopiera, klipp ut, klistra in |
| `Esc` | Stäng. Med osparade ändringar frågar den **Spara** (`s`), **Förkasta** (`d`) eller **Avbryt** (`c`) |

Binärfiler vägras. Tabbstorlek, radslut, syntaxmarkering, radnummer och
radbrytning finns i *Inställningar > Diff & redigerare*; se
[Inställningar](settings.md#diff--redigerare).

Med **Följ markeringen** ikryssat (rutan i redigerarens rad, även *Redigerare: Växla följ markeringen* i paletten och *Inställningar > Diff & redigerare*) öppnas, medan redigeraren är öppen, en annan fil som du markerar i högerpanelen som arbetskatalogens kopia i redigeraren i stället för som diff eller Filvy. Vid osparade ändringar frågas först **Spara**, **Förkasta** eller **Avbryt**; Avbryt lämnar dig kvar i den aktuella filen. En fil som inte kan redigeras (binär, för stor, borttagen) öppnas i sin vanliga vy och statusraden säger varför. När du stänger redigeraren hamnar du i vyn för den markerade filen. Ikryssat som standard. Detta gäller arbetskatalogens ändringar: väljer du en fil från en commit eller en jämförelse medan du redigerar lämnar du redigeraren (efter frågan **Spara**, **Förkasta** eller **Avbryt** vid osparade ändringar) och ser filens diff, och commit-kolumnen, som redigeraren inte har, kommer tillbaka med den.

Statusraden till höger visar filens **teckenkodning**, **radslut** och **indrag**, sedan markörens
position och syntaxen. De tre första är knappar: var och en öppnar en rullgardinsmeny (piltangenter eller
hjul, skriv för att filtrera, `Enter` väljer, `Esc` stänger). Paletten (`Ctrl+P`) har samma menyer: *Redigerare: Ändra teckenkodning*, *Redigerare: Öppna igen med teckenkodning*, *Redigerare: Konvertera till teckenkodning*, *Redigerare: Konvertera radbrytningar*, *Redigerare: Ändra indrag*.

![Redigerarens teckenkodningsmeny öppen under UTF-8-knappen i raden: öppna filen igen med en annan kodning eller konvertera den](../images/editor-encoding-menu.png)

![Redigerarens radslutsmeny öppen under LF-knappen: konvertera filen till LF, CRLF eller CR](../images/editor-eol-menu.png)

- **Teckenkodning.** En fil öppnas i den teckenkodning den anger (en BOM: UTF-8 eller UTF-16), annars som
  UTF-8, annars som Windows-1252, och fältet säger vilken det är. **Öppna igen med teckenkodning** läser
  filens byte på nytt med en annan teckenkodning (den frågar först om det finns osparade ändringar): använd
  den när gissningen blev fel. **Konvertera till teckenkodning** behåller texten och ändrar teckenkodningen
  som filen sparas i (UTF-8, UTF-8 BOM, UTF-16 LE och BE, Windows-1252, ISO-8859-1 och -15, Windows-1250,
  Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR). Inget skrivs förrän du sparar, och Ångra tar
  tillbaka det. Om texten har tecken som målet inte rymmer säger statusraden hur många de är och var det
  första finns, och inget konverteras; sparandet vägrar på samma sätt och skriver aldrig `?`.
- **Radslut.** Visar `LF`, `CRLF`, `CR` eller `Blandad` när filen använder flera. Menyn konverterar alla
  radbrytningar i filen till LF, CRLF eller CR i ett enda Ångra-steg. Nya rader du skriver använder filens eget
  radslut. Ditt val går före inställningen *Radslut*.
- **Indrag.** Visar `Blanksteg: 4` eller `Tabbar: 4`. Menyn bestämmer vad `Tab` och automatiskt indrag infogar
  (blanksteg eller tabbar), storleken (1 till 8: indragssteget och tabbens bredd i den här filen) och
  konverterar de inledande blanktecknen på varje rad till blanksteg eller tabbar, i ett enda Ångra-steg.

## Stash, patchar och Git LFS

- **Stash** och **Pop** finns i verktygsfältet; se
  [Arbeta med Log](log-and-history.md#grenar-tags-och-refs-sidofältet).
- **Skapa patch** och **Tillämpa patch** finns i commit- och refmenyerna; CLI:t
  har `gitpoulpi patch`.
- Stora filer som lagras med Git LFS visas som små pekarfiler tills du gör pull på
  deras innehåll; se [Git LFS](lfs.md).

## Från kommandoraden

Allt på den här sidan går att skripta:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # efter nekandet nedan
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` nekar en känslig fil på samma sätt: ingenting stageas, slutkoden
är 2 och `--json` skriver ut `{error: "sensitive_files", files: [{path, reason}]}`
(en regel och en maskerad förhandsvisning, aldrig ett värde). `--allow-sensitive` är
nytt försök. En MCP-agent nekas med samma svar och kan inte skicka `allow_sensitive` om inte
`[mcp] allow_sensitive_staging = true`: den får veta att den ska fråga dig.

Se [CLI-referensen](cli-reference.md).
