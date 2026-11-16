[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · **Svenska** · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Granska en pull request eller merge request

Granska en pull request (GitHub, Gitea, Forgejo), en merge request (GitLab) eller en Gerrit-ändring (se [Gerrit](#gerrit))
**utan att lämna repositoryts fönster**. Granskningsvyn visar beskrivningen, konversationen, de ändrade
filerna med kommentarstrådarna under sina rader och commits, och det är här du skriver granskningen: kommentarer,
godkännande och begäran om ändringar. Den kräver en profil för någon av de värdarna (se
[Värdtjänster och ärendehantering](hosting-integration.md)) och ett repository länkat till den; andra leverantörer
erbjuder inga granskningar. Värdens begränsningar respekteras: GitLab har inget omdöme *Begär ändringar* och tar en rad per kommentar.
Den här sidan kallar alla dessa en *begäran*.

![Granskningsvyn, sidan Översikt: begärans huvud med sina metadataknappar, beskrivningen och konversationen med ett skickat godkännande](../images/review-overview.png)

![Sidan Filer: de ändrade filerna med sina rutor för visad, diffen med en kommentarstråd under sin rad och en löst tråd utanför de visade raderna](../images/review-files.png)

## Öppna en granskning

Alla vägar in leder till samma vy:

- listan **PULL REQUESTS** i sidofältet: markera en rad och tryck **Enter**, eller högerklicka på den och välj
  **Granska...**;
- **Radar**: radens meny, **Granska...**, eller tangenten `e` (repositoryt öppnas och granskningen med det;
  en begäran från ett repository du inte har klonat frågar om det ska klonas eller om begäran ska granskas via API:et,
  se [Granska utan en klon](#granska-utan-en-klon));
- **arbetsytesökningen** (`Ctrl+Alt+F`): markera en pull eller merge request och tryck **`Alt+R`**;
- paletten: **Hosting: Granska begäran...** listar begäranden att välja bland.

Vyn fyller fönstret över repositoryt (det är ett läge i repositoryfliken, inte ett separat fönster).
**`Esc`** eller `✕` i hörnet lämnar den. Medan den öppnas läses begäran från värden och dess kod
hämtas med git i bakgrunden; en snurra visas om det tar mer än ett ögonblick.

## Huvudet

Alltid synligt: numret och titeln, tillståndet (Open, Draft, Merged, Closed), grenarna
(`main ← feature/moons`), författaren, kontrollerna (`✓ kontroller godkända`, `✗ kontroller misslyckades`, `● kontroller pågår`),
om den går att slå ihop utan konflikter eller har konflikter, storleken och **`3 väntande kommentarer`** när du har skrivit några.

De fyra metadataknapparna - **Tilldelade**, **Granskare**, **Etiketter**, **Milstolpe** - öppnar samma väljare som
sidofältet (se [Tilldelade, granskare, etiketter och milstolpe](hosting-integration.md#tilldelade-granskare-etiketter-och-milstolpe));
huvudet visar resultatet så fort värden har tagit emot det. Åtgärdsknapparna: **Checka ut** (den vanliga
utcheckningen av begärans gren, som också stänger vyn), **Öppna i webbläsaren**, **Uppdatera**
(**`F5`**) och **Slutför granskning**.

## Tre sidor

En segmenterad kontroll i övre kanten växlar sida: **Översikt**, **Filer**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, eller ett klick).

### Översikt

Översikten är **begärans konversation**, samma tidslinje och samma kommentarsruta som ett ärende har
([Konversationer](hosting-integration.md#5c-konversationer)): beskrivningen först (*öppnade detta*), sedan kommentarerna,
trådarna på kod med sitt diff-utdrag, de skickade granskningarna med sitt omdöme (`godkänd`, `begärde
ändringar`, `kommenterade`, `avfärdad`; på GitLab är omdömena dess godkännanden) och händelserna, äldst först, i det
nyaste fönstret med **Läs in äldre** överst. Kommentarsrutan sitter under tidslinjen (**Ny kommentar**):
**Skriv**, **Förhandsvy** och **Delad**, **Avbryt** och **Skicka**. **En kommentar som skrivs här går direkt till värden**
(inget är väntande: väntande kommentarer är de på kod, som skickas med granskningen), visas i listan med en gång
och tar sin plats när värden svarar; om värden vägrar öppnas rutan igen med din text och orsaken.

Tangenterna är konversationsvyns: **`Up`** / **`Down`** (och `Ctrl+J` / `Ctrl+K`) flyttar mellan poster,
**`r`** svarar i den valda postens tråd, **`Q`** citerar den i en ny kommentar, **`c`** skriver en ny
kommentar, **`e`** / **`d`** redigerar och tar bort dina egna kommentarer (en fråga kommer före en borttagning), **`l`** eller `Enter`
öppnar en posts länkar, **`g`** dess diagram, **`y`** kopierar dess adress, **`o`** öppnar den i webbläsaren, **`n`**
går till nästa olästa post, **`v`** visar eller döljer händelser, **`m`** läser in äldre poster, `Tab` växlar mellan
tidslinjen och rutan, och **Intern anteckning** (`Alt+I`) finns på de värdar som har sådana anteckningar. Knapparna för den
valda posten (**Svara**, **Citera**, **Redigera**, **Ta bort**, **Länkar**) gör detsamma, och paletten har raderna
*Konversation: ...* bredvid raderna *Granskning: ...*. En länk frågas om innan den öppnas, som överallt i
granskningen.

Där en konversationstangent och en granskningstangent möts har konversationen bokstaven på den här sidan: `l` (etiketter), `m`
(milstolpe), `Q` (Gerrits abandon), `e` / `d`, `r`. Etiketter, milstolpe och abandon finns kvar på sidhuvudets knappar och
i paletten; `a` (tilldelade), `p` (granskare), `f` (**Slutför granskning**), `b` (**Checka ut**), `P`, `D`, `U` och
`Alt+1` / `Alt+2` / `Alt+3` behåller sin betydelse här. Medan rutan har tangentbordet är varje tangent rutans, utom
`Alt+1` / `Alt+2` / `Alt+3`; `Esc` ger tillbaka tangentbordet till tidslinjen och ett andra `Esc` stänger granskningen.

En kommentar som skrivits i ett annat fönster eller av CLI:t visas här av sig själv: varje skrivning i GitPoulpi (det här
fönstret inräknat) berättar det för sessionen, och fönstren som visar samma begäran läser den igen.

Kommentarsrutan är den Markdown-redigerare som delas med [konversationsvyn](hosting-integration.md#skriva)
(också rutan för ett svar i en tråd, en kommentar på en rad och sammanfattningen i **Slutför granskning**): `Ctrl+Enter` (också
`Ctrl+S`, `Alt+Enter`) skickar, och där AI är på öppnar `Alt+E` **Skriv med AI**
([AI-kapitlet](ai.md#skrivhjälp-på-en-värd)). Förhandsvyn visar texten så som värden gör det, referenser inräknade
(`#7`, `!12`, `ABC-12`, commit-id:n); en osänd text sparas per begäran.

### Filer

De ändrade filerna till vänster (statusbokstav, `+tillagda -borttagna`, en `●n`-räknare för öppna trådar, en `✎n`-räknare för
dina väntande kommentarer) och diffen för den valda filen till höger. Diffen kommer från git: begärans
head hämtas till `refs/gitpoulpi/pr/<n>` (aldrig en gren, inget checkas ut) och jämförs med
merge-basen, så radnumren i marginalen är exakt de som kommentarerna använder.

- **`[ ]` före en fil**: markera den som *visad* (klicka, eller tryck **`v`** / `Space` i listan). Titeln räknar dem.
  Visad-markeringar sparas per repository och överlever en omstart.
- **Trådar** sitter direkt under sin rad: första kommentaren, svar och knapparna **Svara** och
  **Lös** / **Öppna igen**. En löst tråd viks ihop till en rad; **`z`** (eller `Enter` på den, eller ett klick på
  `▾`) viker ihop och ut valfri tråd.
- **Föråldrade trådar**, och trådar på rader som diffen inte visar, listas efter hunkarna under
  *Kommentarer utanför de visade raderna*, med märket `föråldrad` när koden har ändrats under dem.
- **`c`** kommenterar raden markören står på (det lilla `+` i början av raden gör detsamma med
  musen). **`Shift+Up` / `Shift+Down`** (eller en dragning över raderna) markerar först flera rader och
  kommentaren omfattar intervallet. GitLab tar en rad per kommentar, och vyn säger det i stället för att markera.
- **`r`** svarar i tråden under markören, **`x`** löser den eller öppnar den igen. Båda skickas till värden **direkt**.
- Tangenter: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` flyttar; `Tab`, `Right` och `Enter` går från fillistan
  till diffen, `Left` och `Tab` tillbaka; `]` och `[` hoppar till nästa och föregående fil.

### Diffens visningsalternativ

Diffrutan på **Filer** och **Commits** är [Diff-vyns](staging-and-committing.md) egen, med samma fält på sin första
rad och samma ihågkomna val (de är en inställning: det du väljer här är vad Diff-vyn öppnas med,
och tvärtom):

- **Inline / Delad**: en kolumn, eller före och efter sida vid sida. I Delad sitter en tråd, en väntande kommentar eller ett
  förslag under den halva den hör till: en kommentar på en borttagen rad under den vänstra halvan, en kommentar på en
  tillagd eller oförändrad rad under den högra halvan. Klicka på halvan du menar innan du trycker **`c`** på en rad som
  visar både en borttagning och en tillägg; tangentbordet kommenterar den nya sidan.
- **Hunks**: bara ändringarna med sin kontext, eller hela filen med ändringarna på plats. I hela filen sitter en
  tråd på en rad utanför ändringarna under sin rad i stället för i *Kommentarer utanför de visade raderna*.
  Filens text läses i bakgrunden; hunkarna står kvar på skärmen med en rad som säger det tills den kommer (på
  sidan Commits läses upp till 40 filer i en commit på det här sättet).
- **WS**: visa eller ignorera ändringar som bara skiljer sig i blanksteg.
- **Wrap**: bryt långa rader.
- **Upp- / nedpilar**: föregående och nästa hunk.
- **Map** (**`M`**): minikartan bredvid rullningslisten, en inställning med Diff-vyns. Klicka eller dra på den för att
  rulla.
- **Auto** (**`E`**, eller *Granskning: Välj teckenkodning...*): teckenkodningen filen läses i, som i Diff-vyn:
  Autoavkänning eller en av teckenkodningarna i listan, vald för filen rutan visar (på sidan Commits filen
  under markören). Båda sidorna av filen läses om i den teckenkodningen, i bakgrunden, och
  valet delas med Diff-vyn för samma sökväg. En fil som inte kan läsas så säger varför ovanför
  sin diff.
- **`⋯`**: när rutan är för smal för alla knappar ligger de som inte får plats bakom en `⋯` i slutet av
  fältet; dess lista kör var och en som om den klickades. Deras tangenter och palettrader fungerar som vanligt.

Rullningslistens skala visar var saker finns: ändringarna (bara hela filen, som i Diff-vyn), en punkt `●`
för varje tråd, väntande kommentar och förslag, samt markören.

Långa rader kapas vid rutans kant (slå på **Wrap** för att se dem hela) och rullas i sidled: **`Right`**
och **`Left`** på diffen (`Left` vid vänsterkanten går tillbaka till fillistan), hjulet i sidled eller `Shift`
med hjulet, rullningslisten längs nedre kanten, eller *Granskning: Rulla diff åt vänster / höger*.

### Markera text och rader

Var en dragning börjar avgör vad den markerar. En dragning som börjar **i koden** markerar text, som i Diff-vyn
(dubbelklick markerar ett ord); **`Ctrl+C`** (eller *Granskning: Kopiera markerad text eller rad*) kopierar exakt de
tecknen, utan radnummer eller märken, och utan textmarkering kopierar den raden under markören. En
dragning som börjar **i marginalen** (radnumren, märket, `+`) markerar rader, för att kommentera dem
som förut, och ett dubbelklick i marginalen startar fortfarande en kommentar på den raden.

Diff-vyns tangenter `n`, `p`, `m`, `w`, `s` är granskningens egna, så i granskningen är växlarna versaler:
**`V`** layout, **`H`** hunks eller hela filen, **`S`** blanksteg, **`W`** radbrytning, **`J`** / **`K`** nästa och
föregående hunk, **`E`** teckenkodningen, **`M`** minikartan. Var och en har en palettrad (*Granskning: Växla mellan inline och delad vy*,
...).

### Commits

Bara begärans commits (inte historiken bakom den), nyaste först, och diffen för den valda
(dess ändringar mot sin förälder), med alla dess filer under varandra och samma visningsalternativ.

## Skriva en granskning

En kommentar på kod är **väntande**: den sparas på din maskin (i repositoryts tillståndsfil, så den överlever en
omstart) och räknas i huvudet. Inget skickas förrän du slutför granskningen. En väntande kommentar visas
under sin rad med märket `Väntande - rad 12` och två knappar: **`e`** (Redigera) och **`d`** (Ta bort).

**Slutför granskning** (**`f`**, knappen eller paletten) öppnar ett litet formulär: en sammanfattning (valfri) och en
lista **Omdöme**: *Kommentera*, *Godkänn*, *Begär ändringar*. Bara det värden tillåter listas: GitLab har inget
*Begär ändringar*. **Skicka granskning** (`Ctrl+S`) skickar sammanfattningen, omdömet och alla väntande kommentarer i
ett svep. **Förkasta granskning...** kastar de väntande kommentarerna efter en bekräftelse.

Vad som skickas direkt, utan att vänta på granskningen: svar, att lösa och öppna igen samt kommentarer i
konversationen. Kommentarrutan säger vilken sort det är (*Den här kommentaren publiceras direkt* eller *Den här kommentaren läggs till i din
väntande granskning*). Om värden vägrar ett svar öppnas rutan igen med dina ord.

Om begäran fick nya commits efter att du skrev väntande kommentarer varnar vyn för att deras rader kan ha
flyttat (GitHub och GitLab placerar en kommentar på commiten du läste; en granskning av ett äldre head vägras).

## Gerrit

En Gerrit-server ansluts som de andra värdarna (se [Värdtjänster och ärendehantering](hosting-integration.md): serverns
URL, användarnamn, HTTP-lösenord) och länkas under **PULL REQUESTS** till ett *projekt*. Dess ändringar listas som pull
requests (`#7`, chip `GR`) och granskas i samma vy, med skillnaderna Gerrit för med sig:

- **Patch sets.** En ändring har numrerade patch sets (varje uppladdning av commiten är ett). Huvudet har en
  kombinationsruta **`Patch set N`** i övre kanten, bredvid sidflikarna (**`P`**, eller palettraden *Granskning: Välj
  patch set...*). Den börjar på det senaste; väljer du ett annat läses det patch setets filer och kod, och
  trådarna skrivna på andra patch sets listas som *föråldrade*. Granskningen du slutför skickas på patch setet
  som visas.
- **Jämför med ett annat patch set.** En andra kombinationsruta till vänster om den, **`Jämför: bas`** (**`D`**, eller *Granskning: Jämför
  patch sets...*), listar basen och varje patch set som är äldre än det som visas. Välj patch set 1 medan patch set 3
  visas och Filer-sidan listar, och visar diffen för, bara det patch set 3 ändrade sedan patch set 1 (serverns egen
  skillnad, så att en rebase på ett nyare mål inte syns som ändringar); Commits-sidan visar den enda
  skillnaden. Båda patch seten hämtas till `refs/gitpoulpi/pr/<n>` och `refs/gitpoulpi/pr/<n>-ps<k>`. Trådar
  skrivna på andra patch sets än det som visas är *föråldrade*. Att välja ett äldre patch set att läsa än det
  jämförda återställer jämförelsen till basen. CLI-motsvarigheten är `review show|fetch --compare-with K`.
- **Skicka in och överge.** När servern säger att ändringen kan skickas in (rösterna och kontrollerna den behöver
  finns) ansluter sig en knapp **Skicka in** till huvudets knappar; **Överge** finns kvar så länge du får överge ändringen.
  Båda (tangenterna **`U`** och **`Q`**, palett *Granskning: Skicka in ändring...* / *Granskning: Överge ändring...*) frågar först, i
  en popup som säger att åtgärden sker på servern och **inte kan ångras från GitPoulpi** (en mergad ändring
  förblir mergad; en övergiven kan återställas i Gerrits webbgränssnitt). Att tacka nej skickar ingenting; vid lyckat resultat
  läser vyn ändringen igen. CLI-motsvarigheterna är `gitpoulpi hosting review submit-change <n>` och
  `hosting review abandon <n> [--message M]`.
- **Röster i stället för ett omdöme.** **Slutför granskning** visar en kombinationsruta per etikett du får rösta på - `Code-Review`
  (-2 till +2) och `Verified` där projektet har den - och listar bara värdena servern tillåter **för den här
  ändringen och dig** (en utvecklare utan rättigheten kan inte välja +2). Den börjar på din nuvarande röst; bara en röst du
  ändrade skickas. Sammanfattningen och de väntande kommentarerna går med i samma anrop. Utkast du sparade i Gerrits
  webbgränssnitt publiceras med granskningen.
- **Trådar** är Gerrits kommentarskedjor. En tråd är löst när dess nyaste kommentar säger `unresolved: false`;
  **lös** skickar ett svar `Done` som gör det, **öppna igen** ett svar `Reopened`. Svar och att lösa går till servern
  direkt, som på de andra värdarna.
- **Push för granskning.** Gerrit skapar en ändring när du pushar till `refs/for/<branch>`. När avsnittet PULL REQUESTS
  är länkat till ett Gerrit-projekt frågar avsnittets **`+`**-knapp och palettraden **Hosting: Push för
  granskning...** efter målgrenen och pushar `HEAD` till `refs/for/<branch>` (ett ämne kan anges på
  kommandoraden). Pushen kan inte ångras från GitPoulpi (överge ändringen i Gerrit); meddelandet säger det. CLI-
  motsvarigheten är `gitpoulpi hosting review push`.
- Metadataknapparna är skrivskyddade för Gerrit (granskarna visas), det finns inget omdöme *Begär ändringar*
  (en röst på -1 eller -2 är samma sak) och kontrollchipet är etiketten `Verified`.

## ✦ AI-granskningsutkast

![Två ✦-förslag under sina rader på sidan Filer, vart och ett med Godta, Redigera och Avfärda, bredvid en befintlig tråd](../images/review-ai.png)

Med AI påslaget (*Inställningar > AI*, se [AI](ai.md)) har huvudet en knapp **`✦ Granskningsutkast...`** (även **`s`**,
och palettraden *Granskning: ✦ Granskningsutkast...*). Den öppnar [granskningsalternativen](#granskningsalternativ) och
**`Enter`** där frågar modellen. Det är det enda som frågar modellen: att öppna en granskning
gör det aldrig. Modellen läser begärans titel och beskrivning, kommentarerna som redan finns (så att den
inte upprepar dem) och diffen, och svarar med ett **granskningsutkast**: texten att skicka, omdömet den rekommenderar (Godkänn, Begär ändringar eller Kommentar), en övergripande sammanfattning och en lista med föreslagna kommentarer. En
lång diff kapas för modellen - per fil och totalt - och Översikt säger hur många filer som kortades ned
eller utelämnades. Medan den arbetar visar statusraden en snurra efter en stund och **`Esc`** (eller samma knapp)
avbryter den. *Kodgranskning*-instruktionerna på de [fyra nivåerna](ai.md#instruktioner) gäller, så ett team eller en
arbetsyta kan säga hur en bra granskning ser ut.

### Granskningsalternativ

`s` öppnar dialogen **✦ Granskningsalternativ**, ifylld med repositoryts standardvärden (de globala där
repositoryt saknar egna). `Enter` (eller knappen **Kör**) startar körningen med det dialogen visar; `Esc`, **Avbryt**
eller krysset stänger den och frågar ingenting.

| Kontroll | Gör |
|---|---|
| **Kör agenten i repositoryt** | *Profilinställning* (AI-profilens arbetsmapp avgör), *Ja* eller *Nej*. Gråad, med orsaken under, för en HTTP-profil (den har ingen arbetsmapp) och när begäran granskas via API:et utan klon. |
| **CLAUDE.md**, **AGENTS.md** | Ge modellen den filen i repositoryt, läst på den gren begäran riktar sig mot. När agenten körs i repositoryt läser den dem själv, och dialogen säger det. |
| **Extra fil** | En fil till: en sökväg i repositoryt, en absolut sökväg eller en länk till en fil hos en kodvärd. **Bläddra...** (`Ctrl+B`, eller `Enter` på knappen) öppnar en filväljare som startar i repositoryts rot (eller bredvid den fil som redan anges); en fil i repositoryt skrivs som en sökväg relativt roten, en utanför som en absolut sökväg. Att skriva i fältet fungerar som förut. |
| **Ingen extra fil** | Ingen extra fil för den här körningen, även när den globala inställningen eller repositoryt anger en; att skriva en fil eller välja en med **Bläddra...** avmarkerar den. Med **Spara som detta repositorys standardvärden** blir det repositoryts omkopplare *Ingen extra fil*. Ett tomt fält räknas här som ingen, eftersom dialogen börjar från det körningen skulle ärva. |
| **Spara som detta repositorys standardvärden** | Behåller också värdena för nästa granskningar: i repositoryts tillståndsfil, eller i `remote-reviews.toml` för ett repository som inte är klonat. Bara ett värde som skiljer sig från det globala behålls, så resten fortsätter följa *Inställningar > AI*. |

Allt nås med `Tab`, `Space` växlar en kryssruta och sökvägsfältet är ett vanligt textfält. Samma
fyra inställningar finns i *Inställningar > AI* (*Kodgranskning*) och i *Repo-specifika inställningar > AI*; se
[Granskningskontext](ai.md#granskningskontext). Det som inte kunde ges till modellen (en extra fil som saknas eller
inte går att läsa) och varje fil som kortades ned för att rymmas inom gränserna listas på utkastkortet.

Förslag är **inte kommentarer**. De visas på två ställen:

- **Under sin rad i Filer**, som spökkommentarer märkta `✦` med en allvarlighetsgrad (`info`, `varning`, `problem`) och
  tre knappar: **Godta**, **Redigera** och **Avfärda**. Ett förslag om en rad som diffen inte visar
  behålls, tappas aldrig: det listas efter hunkarna under *AI-förslag om hela filen* och, när det
  godtagits, hamnar det på filens första ändrade rad med orden *(om rad N)* framför.
- **I Översikt**, i ett block ovanför konversationen (bara efter en körning): sammanfattningen, sedan listan med samma
  knappar och **Godta alla** / **Avfärda alla**.

**Godta** (`y`) gör ett förslag till en vanlig **väntande kommentar** (du kan fortfarande redigera eller ta bort den);
**Redigera** (`w`) öppnar kommentarrutan med modellens ord först och godtar det du lämnar kvar; **Avfärda** (`n`)
släpper det. **Godta alla** (`A`) fyller fortfarande bara den väntande listan. På sidan Filer verkar tangenterna på
förslaget under markören, i Översikt på det första i listan (`y` och `n` är konversationens *Kopiera länk* och
*Nästa olästa* igen när listan är tom). **Använd AI-sammanfattning** (`i`, eller knappen
i formuläret Slutför granskning) lägger modellens sammanfattning i granskningstexten; den infogas aldrig av sig själv.
**Inget skickas till värden förrän du skickar granskningen.**

Översikt visar utkastet som ett kort överst i det blocket, ovanför förslagen: omdömeschippet, granskningstexten och tre
knappar.

- **✦ Använd utkastet...** (`u`) godtar varje öppet förslag som en väntande kommentar och öppnar *Slutför granskning* redan
  ifyllt med utkastets text och omdöme. Du läser fortfarande, ändrar och skickar det själv; att avbryta
  formuläret behåller utkastet. Värdens gränser respekteras: GitLab har ingen *Begär ändringar*, så formuläret säger det och
  faller tillbaka på *Kommentar*; på Gerrit blir omdömet en Code-Review-röst på +1 eller -1, bara när servern
  tillåter det värdet för dig (annars sätts ingen röst och formuläret säger det).
- **Redigera text...** (`t`) ändrar utkastets granskningstext innan du använder det.
- **Kassera utkastet** (`N`) kastar texten, omdömet och de återstående förslagen.

Utkastet sparas per begäran, så det finns kvar när du stänger och öppnar granskningen igen. När begärans
head har flyttats sedan modellen skrev det säger kortet *Begäran har ändrats sedan det här utkastet skrevs.*
Att skicka granskningen förbrukar utkastet.

## Allt har en tangent och en palettrad

| Åtgärd | Tangent | Palett |
|---|---|---|
| Översikt / Filer / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Granskning: Visa översikt*, *Visa filer*, *Visa Commits* |
| Kommentera raden eller intervallet | `c` | *Granskning: Kommentera rad...* |
| Kommentera konversationen | `c` i Översikt, `C` | *Granskning: Kommentera konversationen...* (öppnar Översiktens ruta), *Konversation: Ny kommentar* |
| Svara i tråden | `r` | *Granskning: Svara i tråd...* |
| Lös eller öppna igen | `x` | *Granskning: Lös eller öppna tråd igen* |
| Vik ihop eller ut en tråd | `z` | *Granskning: Fäll ihop eller ut tråd* |
| Redigera / ta bort en väntande kommentar | `e` / `d` | *Granskning: Redigera väntande kommentar...*, *Ta bort väntande kommentar* |
| Markera filen som visad | `v` | *Granskning: Markera fil som visad* |
| Inline / Delad | `V` | *Granskning: Växla mellan inline och delad vy* |
| Hela filen / bara hunkarna | `H` | *Granskning: Växla mellan hela filen och bara hunks* |
| Ändringar med bara blanksteg | `S` | *Granskning: Växla ändringar med bara blanksteg* |
| Radbrytning | `W` | *Granskning: Växla radbrytning* |
| Nästa / föregående hunk | `J` / `K` | *Granskning: Nästa hunk*, *Föregående hunk* |
| Nästa / föregående fil | `]` / `[` | *Granskning: Nästa fil*, *Föregående fil* |
| Slutför / förkasta granskningen | `f` | *Granskning: Slutför granskning...*, *Förkasta väntande granskning...* |
| Uppdatera | `F5`, `Ctrl+R` | *Granskning: Uppdatera* |
| Öppna i webbläsaren / checka ut | `o` / `b` | *Granskning: Öppna i webbläsaren*, *Checka ut gren* |
| Tilldelade / granskare / etiketter / milstolpe | `a` / `p` / `l` / `m` | *Granskning: Redigera tilldelade...* och dess tre syskon |
| Välj patch set (Gerrit) | `P` | *Granskning: Välj patch set...* |
| Jämför med ett äldre patch set (Gerrit) | `D` | *Granskning: Jämför patch sets...* |
| Skicka in / överge ändringen (Gerrit) | `U` / `Q` | *Granskning: Skicka in ändring...*, *Granskning: Överge ändring...* |
| ✦ Granskningsutkast... (AI på; öppnar granskningsalternativen) | `s` | *Granskning: ✦ Granskningsutkast...* |
| Godta / redigera och godta / avfärda ett förslag | `y` / `w` / `n` | *Granskning: Godta förslag*, *Redigera och godta förslag...*, *Avfärda förslag* |
| Godta / avfärda alla förslag | `A` / `N` | *Granskning: Godta alla förslag*, *Avfärda alla förslag* |
| Använd AI-sammanfattningen i granskningstexten | `i` | *Granskning: Använd AI-sammanfattning i granskningstexten* |
| Använd ✦-utkastet (fyller i Slutför granskning) | `u` | *Granskning: ✦ Använd utkastet...* |
| Redigera utkastets text | `t` | *Granskning: Redigera ✦-utkastets text...* |
| Kassera ✦-utkastet | `N` | *Granskning: Kassera ✦-utkastet* |
| Stäng | `Esc` | *Granskning: Stäng* |

Öppna paletten (`Ctrl+P`) medan vyn är öppen: dess rader verkar på den och säger varför en inte gör något
just nu (”placera markören på en kodrad först”).

## Granska utan en klon

En begäran från ett repository som inte finns på din dator kan ändå granskas. Från Radar frågar **Granska**
(`e`) på dess rad hur (se [Radar](radar.md#granska-en-begäran)); **Granska via API:et** öppnar
samma vy i Hem-fönstret, över Radar. Allt på den här sidan fungerar likadant, med de här
skillnaderna:

- Koden (de ändrade filerna, varje fils diff, begärans commits, en commits diff, hela filen) läses
  från värdens API, så första blicken på en stor begäran tar en stund, och en fil som värden inte
  skickar någon diff för (en binär eller en mycket stor) säger det i stället för att visa en.
- Dina väntande kommentarer sparas i `remote-reviews.toml` i konfigurationsmappen, ordnade efter värd
  och projekt (`host/project`), i stället för i ett repositorys eget tillstånd. De finns kvar när du
  stänger och öppnar granskningen igen, och att skicka granskningen förbrukar dem.
- Det finns inget teckenkodningsval (*Inte tillgängligt utan en klon*) och ingen **Checka ut** (den säger åt dig att klona
  repositoryt först).
- `Esc` eller `✕` i hörnet lämnar vyn och ger tillbaka Radar. Paletten (`Ctrl+P`) når
  vyns rader som i ett repositorys flik.

Vill du behålla klonen i stället väljer du **Klona, granska sedan** (eller palettens *Radar: Klona och granska...*):
dialogen Klona öppnas på begärans anslutna konto med dess repository markerat, och granskningen
öppnas i det nya repositoryts flik när klonen har landat.

## Från kommandoraden och för agenter

Samma granskning finns som `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(och MCP-verktygen som görs av dem): se [CLI-referens](cli-reference.md). `threads` listar också
omdömena som redan skickats (`reviews`). `suggest <n>` frågar modellen (AI på) och skriver ut dess `verdict`, `body`, sammanfattning och
förslag som JSON utan att publicera något, så ett skript eller en agent kan välja kommentarerna att ge till
`submit --comment`.

Inget av detta kräver en klon: `--project owner/name` (med `--id` när flera integrationer är konfigurerade)
kör `show`, `threads`, `comment`, `reply`, `resolve`, `submit` och `suggest` mot ett projekt som den aktuella
katalogen inte vet något om. `hosting review diff <n>` skriver ut diffen med sina hunkar (`--api` läser den från
värden även i en klon), och `hosting file <path> --project P [--ref R]` läser en fil vid en revision, till
exempel projektets `CLAUDE.md`. Båda läser värdens API: GitHub, GitLab, Gitea, Forgejo och Gerrit.
