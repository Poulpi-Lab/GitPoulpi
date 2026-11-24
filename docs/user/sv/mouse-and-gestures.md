[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · **Svenska** · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Mus och gester

Allt du kan göra med musen har också en tangentbordsmotsvarighet. Se
[Kortkommandon](keybindings.md). Namnen på områdena finns i
[rundturen i gränssnittet](interface-tour.md).

## Allmänna regler

- **Klicka** på en rad för att markera den. Det fungerar i varje panel, lista,
  meny och dialogruta.
- **Dubbelklicka** på en rad för att öppna eller köra den. Ett dubbelklick på
  en lokal gren, en tag eller en ref-pill checkar ut den. Ett dubbelklick på en
  repositoryrad öppnar repositoryt.
- **Högerklicka** på en rad för dess meny. `.`, `Shift+F10` eller Meny-tangenten
  öppnar samma meny för den markerade raden.
- **Hjulet** rullar området under pekaren. I en lista flyttar det bara vyn.
  Markeringen stannar där den är, även när den rullar ur bild. Bara ett klick
  eller en tangent flyttar markeringen.
- **Shift+hjul** rullar i sidled, i diffen, Filvyn och redigeraren när radbrytning
  är av.
- **Peka** på en knapp, en rad eller ett rullningslisthandtag så tänds den. Många
  terminaler kan inte ändra muspekaren, så den här belysningen är ofta det enda
  tecknet på att något går att klicka på. Låt pekaren vila på en knapp eller en
  Log-cell för ett verktygstips.
- **Klicka utanför** ett textfält för att lämna det. Det du skrev behålls.
  Refs-filtret tillämpar det du skrev, som `Enter` gör. Log-sökfältet stängs, som
  `Esc` gör.

## Stänga fönster och dialogrutor

Varje inramat fönster och varje dialogruta har ett `✕` uppe till höger i ramen, och
samma regler gäller överallt:

- **Klicka på `✕`** för att stänga. `Esc` gör detsamma och stänger det innersta först
  (en öppen rullgardinslista, ett textfält, en bekräftelse) innan fönstret.
- **Klicka utanför** en dialogruta, en hjälp- eller infopanel eller en popup för att
  stänga den. Ett klick utanför stänger aldrig en helskärmsvy (Inställningar,
  Diff-vyn, Filvyn, redigerarna, granskningen, tavlan): de har ingen utsida.
- Ett klick **inuti** en panel stänger den aldrig.
- Ett musklick (vänster eller höger) medan en `Ctrl+G`- eller `Ctrl+N`-kedja väntar på sin andra tangent avbryter kedjan som `Esc` och fungerar sedan som vanligt (ett klick markerar fortfarande, ett högerklick öppnar fortfarande sin meny).
- Ett formulär som innehåller text du skrivit (ett nytt ärende, en pull eller merge
  request) ignorerar ett klick utanför, så att ett felklick inte kan förlora den.
  `✕` och `Esc` frågar **Förkasta ändringar?** först; **Fortsätt redigera** går
  tillbaka. Ett formulär utan inskriven text stängs direkt, och det gör också en
  dialogruta med en enda rad.
- Kommandopaletten, väljarna (flik, tema, språk), de små dialogrutorna för granskning,
  tavla, sammanslagning och meddelanderedigeraren samt bekräftelsefrågan har också `✕`.
  Snabbmenyer och rullgardinslistor har inget: de stängs med `Esc` eller ett klick
  utanför.

## Listor i Inställningar

Listorna i Inställningar (Profiler, AI-profiler, Arbetsflöde, Öppna i, Bilder) hanteras
med musen så här. **Klicka på en underflik** (Allmänt eller listan) för att växla till den; `Alt+Left` / `Alt+Right` gör detsamma.

- **Klicka** på en rad för att välja den, **dubbelklicka** för att gå till dess fält,
  använd **hjulet** för att rulla utan att välja.
- **Tryck på en rad och dra** den uppåt eller nedåt för att ordna om: ett streck visar
  var den hamnar, släpp lägger dit den och `Esc` avbryter. Tangentbordsmotsvarigheten
  är `Alt+Up` / `Alt+Down`.
- **Klicka på `+ Lägg till`, `Duplicera`, `Ta bort`, `↑`, `↓`** under listan. Tangenter:
  `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; se
  [Inställningar](settings.md#listor).

## Listor i popuprutor

De här reglerna gäller varje lista i en popup med ett filterfält:
kommandopaletten, dess väljare (tema, språk, grafens färger), flikväljaren,
sökbara rullgardinslistor, listan med ärendereferenser och hjälpen.

- **Klicka** på en rad för att markera den.
- **Dubbelklicka** på en rad för att köra eller välja den, som `Enter` gör.
- **Hjulet** rullar listan. Det flyttar inte markeringen.
- När listan är längre än popupen visas en **rullningslist** på dess högra kant.
  Dra i handtaget eller klicka på spåret.

En kort rullgardinslista utan filterfält väljer raden vid första klicket. En
kontextmeny kör raden vid första klicket. Tangenterna för de här listorna står i
[Kortkommandon](keybindings.md#filtrerbara-popuplistor).

## Rullningslister

Varje område som kan rulla visar en tunn rullningslist på sin kant när
innehållet inte ryms: Log, Refs-sektionerna, fillistorna, diffen, Filvyn,
redigeraren och popuplistor.

- **Dra i handtaget** för att rulla.
- **Klicka på spåret** för att flytta handtaget till pekaren: vyn hoppar dit
  proportionellt, och så länge du håller knappen nedtryckt fortsätter du att dra.
- **Håll knappen nedtryckt på en ändpil** för att fortsätta rulla. Det startar
  efter en kort fördröjning. Flytta bort från pilen för att pausa.
- En rullningslist flyttar bara vyn. Den flyttar aldrig markeringen.

I diffen och redigeraren visar den lodräta rullningslisten dessutom hela filen i
små märken: grönt för tillagda rader, rött för borttagna rader, gult för
ändrade rader, `◆` för markörens rad och `▌` för varje annan plats där den
markerade texten förekommer. Merge-redigeraren märker ut varje konflikt på sina
rullningslister. Klicka på ett märke för att gå dit.

## Fel och aktivitetsloggen

- **Klicka på det röda felet** i statusraden för att öppna aktivitetsloggen på dess post; **klicka på Kopiera / den föreslagna knappen** (Försök igen, Pull först, Stash och försök igen, Öppna Integrationer) på posten.
- **Klicka på Aktivitet** längst till höger på statusraden för att öppna aktivitetsloggen. Klicka på en flik för att byta, på en nivåruta (eller **Alla** / **Ingen**) för att visa eller dölja de raderna, på en post i ett repository för att öppna dess detaljer, och på **Rensa loggen** på fliken Program (den frågar först). Rulla långa rader i sidled med `Shift` + hjul eller den vågräta rullningslisten. Se [Aktivitetsloggen](troubleshooting.md#aktivitetsloggen).
- **Klicka på `[x]`** vid jobbindikatorn till höger på statusraden för att avbryta det pågående jobbet, **klicka på etiketten** för att öppna jobbpanelen. Knappen **Aktivitet** nere till höger på startfliken (eller `Ctrl+N Shift+G`) öppnar aktivitetsloggen på Alla repositories (dess ruta Omfång, en rullgardinslista under flikarna, väljer ett repository eller en arbetsyta).

## Ändra storlek

- **Dra i en avdelare** mellan Refs-sidofältet, Log och högra panelen. Tangentbordet flyttar dem ett steg i taget: `Ctrl+N Left` / `Ctrl+N Right` flyttar avdelaren mellan Refs och Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` avdelaren mellan Log och högra panelen (avdelaren går åt det håll pilen pekar).
  Dra den förbi minimibredden för att fälla ihop panelen. Paletten har samma sak som `Navigering: Bredda Refs-rutan`, `... Smalna av Refs-rutan`, `... Bredda Status-rutan` och `... Smalna av Status-rutan`, samt `Navigering: Fäll ihop/ut Refs-rutan` / `... Fäll ihop/ut Status-rutan`.
- **Dra i nedre kanten** av en Refs-sektion för att ändra dess höjd.
- **Dra i nedre kanten** av commit-meddelandeblocket i högra panelen.
- **Dra i gränsen mellan två Log-rubriker** (som standard mellanrummet, från rubriken eller från var som helst nedåt i tabellen; Inställningar > Log & graf > Kolumnstorlek) för att ändra storlek på en kolumn (tangentbordet: `Shift+C` för att välja en kolumn, sedan `]` / `[`).
- **Dubbelklicka på gränsen mellan två Log-rubriker** för att anpassa kolumnen till vänster om den efter dess innehåll (tangentbordet: `Shift+C` för att välja en kolumn, sedan `=`; `+` anpassar alla kolumner). Sista kolumnen har ett grepp på sin egen sista cell: dra i det för att ändra storlek på den kolumnen, så tar kolumnen Meddelande mellanskillnaden.
- **Rulla Log i sidled** (kolumnlayouten `scroll`; Inställningar > Log & graf > Kolumnlayout, eller väljarens ruta `Rulla i sidled`): det horisontella hjulet eller `Shift`+hjul över tabellen, stapeln på nedre kanten (dra i dess handtag, tryck på det bara spåret, klicka på dess ändpilar) eller `Shift+Vänster` / `Shift+Höger`. I standardlayouten `fit` fyller kolumnerna rutan och det finns inget att rulla.
- **Rulla en listruta i sidled** när en rad är bredare än rutan: ett Refs-avsnitt (ett långt namn på en gren, tag eller stash), listorna Unstaged och Staged, commitens (eller en jämförelses) ändrade filer och commit-detaljernas infrarader (id, författare, författardatum, antal). Det horisontella hjulet eller `Shift`+hjul över rutan rullar den; ett fält visas på rutans nedre kant bara medan något flödar över (dra i dess tumme, tryck på det bara spåret, klicka på dess ändpilar), så det tar aldrig en rad. Från tangentbordet rullar `Alt+Left` / `Alt+Right` rutan som har fokus (Log rullar commit-detaljerna; inne i de ändrade filerna, filerna); paletten har `Vy: Rulla panelen åt vänster/höger`. Hela raden rullar (indrag, statusmarkör, vinkel och namn tillsammans); rutans titel, dess kant, rullningslisterna och en markerad Status-rads Stage/Unstage-knapp stannar kvar. Ett namn kapas inte längre med en ellips: dess slut är en rullning bort (en gren med extra push-mål behåller i stället sitt `⇉`-märke i bild, så dess namn viker sig fortfarande). Förskjutningen gäller per ruta, stannar vid den bredaste raden i bild och glöms när du byter repository. Commit-meddelandeblocket radbryter sin text, så det har inget att rulla. Medan ett avsnitt är rullat i sidled bär dess nedre kant fältet, så ändra storlek på avsnittet med dess tangentbordskommandon eller från ett avsnitt som får plats.
- **Dra en Log-rubrik** för att flytta kolumnen: ett spöke av dess namn följer pekaren, rubriken den lämnade dämpas och en accentstapel markerar platsen där den skulle hamna. Kolumnerna flyttas när du släpper; `Esc` avbryter, och pekaren visar den öppna handen medan du bär den (tangentbordet: `Shift+C` för att välja en kolumn, sedan `<` / `>`).

GitPoulpi kommer ihåg varje storlek. Se
[Storlekar sparas](interface-tour.md#storlekar-sparas).

## Titelrad, flikrad och verktygsfält

- **Klicka på <img src="../../../assets/logo/gitpoulpi.png" alt="GitPoulpi-ikon" height="16">** i titelraden för att öppna **Om GitPoulpi**. I Om-rutan följer
  bläckfiskens pupiller pekaren; klicka på logotypen för att snurra den till nästa färg,
  högerklicka på den för listan med färger, klicka på en länk för att öppna den.
- **Klicka på Öppna i ▾** i titelraden för listan med program. Se
  [Öppna i](open-in.md).
- **Klicka på versionen** (`v1.0.0`, eller `vX.Y.Z tillgänglig`) i titelraden för att öppna fliken Versionsanteckningar.
- **Klicka på förstoringsglaset och `Global sökning`** i titelraden, till vänster om kugghjulet, för den
  [globala sökningen](search.md).
- **Klicka på kugghjulet** i titelraden, till vänster om ×, för Inställningar.
- **Klicka på ×** längst till höger i titelraden för att avsluta.
- **Klicka på en flik** för att växla till den. **Klicka på dess `x`** (visas på den
  aktiva fliken och på fliken under pekaren) för att stänga den. **Dra en flik** åt
  sidan för att ändra ordningen. **Högerklicka på en flik** för flikmenyn.
- **Mittenklicka på en flik** för att stänga den, som dess `x`.
- **Klicka på `‹` eller `›`** i ändarna av flikarna (visas när de inte ryms) för att rulla raden en flik; håll knappen nere för att fortsätta rulla, eller använd hjulet över raden. Om du drar en flik till en pil rullar raden också. Raden rullar även av sig själv för att visa den aktiva fliken varje gång den byts.
- **Klicka på `+`** efter den sista fliken för en meny: Ny flik (`Alt+T`) och, medan deras flikar är stängda, Radar och Arbetsytor (`Alt+J`).
- **Klicka på husikonen** för fliken Hem.
- **Klicka på fliken Radar eller Arbetsytor** för att öppna den; dess `x`, ett mittenklick eller `Ctrl+W` stänger den. **Klicka på arbetsytechipet** (`Alt+W`) för att byta arbetsyta.
- **Klicka på Avsluta, cd hit** (visas till höger om flikarna när du startade GitPoulpi via skalomslaget `gp`) för att avsluta och lämna ditt skal i den aktiva flikens mapp. Se [Följa den aktiva flikens mapp](following-the-tab-folder.md).
- **Klicka på en verktygsfältsknapp** för att köra den. **Klicka på `▾`** bredvid
  Pull, Push eller LFS för dess lista. **Klicka på `⋯`** för knapparna som inte
  fick plats.
- **Klicka på profilrutan** längst till höger i verktygsfältet för att välja
  commit-identitet för det här repositoryt.

## Fliken Hem

- **Klicka** på en rad för att markera den. **Dubbelklicka** på den för att
  öppna repositoryt.
- **Klicka på stjärnan** för att lägga till eller ta bort en favorit.
- **Klicka på Öppna, Växla, Stäng eller Ta bort** på en rad, eller på **`⋮`** för
  radens meny. **Högerklicka** på en rad för samma meny.
- **Klicka på en sektionstitel** för att fälla ihop sektionen.
- **Klicka på Bläddra, Klona, Init, Fäll ihop alla eller Fäll ut alla** högst upp,
  och på **Stäng alla flikar** eller **Ta bort alla** på en sektionstitel.

I mappväljaren **Bläddra**: klicka på en plats eller en mapp för att gå dit,
dubbelklicka på en mapp för att öppna den, klicka på sökvägsfältet för att skriva
en sökväg. Kryssa i **Visa syskonmappar** (`Alt+S`, sparas) så listar det vänstra trädet också de andra mapparna på varje öppen nivå: mapparna på den öppna sökvägen visar `▾`, de andra `▸`, och ett klick på en går dit. I **Klona** och **Init**: klicka på ett fält för att skriva i det, och
klicka på en knapp eller en kryssruta för att använda den.

## Refs-sidofältet

- **Klicka** på en rad för att markera den. **Dubbelklicka** på en lokal gren
  eller en tag för att checka ut den. **Dubbelklicka** på en worktree eller en
  submodul för att öppna den i en flik. **+** på rubriken WORKTREES skapar en
  worktree; högerklicka på en worktree för Öppna i ny flik, Öppna i, Merge och
  Ta bort ([Worktrees](worktrees.md)).
- **Peka** på en rad för att visa ikonen till vänster om den. **Klicka** på den
  för att dölja refen i Log, eller för att visa den igen. Medan **Solo** är på
  solar samma plats refen. På en mapprad verkar ikonen på varje ref i mappen.
- **Klicka på Sluta solo** eller **Visa alla** i sidofältets titel för att
  rensa varje solo eller varje dold ref.
- **Högerklicka** på en rad för dess meny: checkout, merge, fast-forward,
  rebase, skapa en worktree, byt namn, ta bort, dölj, solo med mera. En remotes
  egen rad har **Fetch**, **Redigera**, **Ta bort** och andra remote-åtgärder. En
  stash har **Pop**, **Apply** och **Drop**. En nedtonad rad säger varför den
  inte kan köras.
- **Klicka på `+`** på rubriken REMOTE för att lägga till en remote.
- **Klicka på en mapprad** för att fälla ihop eller ut den.

### Pull requests och ärenden

| Gest | Åtgärd |
|---|---|
| Klicka på en rad | Markera den. |
| Dubbelklicka på en rad | Öppna den i webbläsaren. |
| Högerklicka på en rad | Dess meny: öppna, kopiera URL:en eller numret, checka ut källgrenen, skapa en gren från ett ärende. |
| Klicka på `+` på sektionstiteln | Öppna formuläret **Skapa pull request** eller formuläret **Nytt ärende**. |
| Klicka på filterraden (`Open ▾`) | Välj vilka förfrågningar eller ärenden som listas. |
| Klicka på en grupptitel | Fäll ihop eller ut gruppen. |
| Klicka på leverantörs- eller repositoryraden (ÄRENDEN) | Välj ärendekälla. **Ingen (koppla bort det här repositoryt)** tar bort den. |
| Klicka på frågeraden (ÄRENDEN) | Skriv en JQL- eller WIQL-fråga (Jira, Azure DevOps), eller välj Trello-listor. |
| Klicka på verktygsfältets chip (`GH #12`, `GL !53`, ...) | Öppna pull requesten i webbläsaren. Högerklicka på den för **Öppna i webbläsare**, **Kopiera URL**, **Uppdatera** och **Byt profil**. |
| Klicka på det nedtonade chippet `+ PR` | Starta en pull request för den pushade grenen. |

Se [Värdtjänster och ärendehantering](hosting-integration.md).

## Log

- **Klicka** på en commit för att markera den. Dess detaljer visas i högra
  panelen.
- **Ctrl+klick** lägger till en commit i markeringen eller tar bort den.
  **Shift+klick** markerar ett intervall (**Alt+klick** i terminaler som behåller
  Shift+klick för sin egen textmarkering, som kitty och Windows Terminal). Högerklicka på en
  markerad commit för att cherry-picka dem alla. Markeringen kan omfatta raden WIP och stashar;
  två eller fler rader gör högra rutan till en jämförelse, se
  [Jämföra commits](log-and-history.md#jämföra-commits).
- **Klicka på ett kort** i jämförelserutan för att flytta Log-markören till den raden;
  **klicka på Byt plats** för att byta ändarna Från och Till. Ett vanligt klick på en rad
  lämnar jämförelsen.
- **Klicka på WIP-raden** för de lokala ändringarna. **Klicka på dess `// WIP`-text**
  för att skriva commit-sammanfattningen där.
- **Peka** på en cell för hela texten: meddelandet, författaren och e-postadressen,
  hela datumet, hela hashen.
- **Peka på ett `+N`-märke** för de andra refsen på commiten. Flytta in i
  popupen för att klicka på en ref där. Dubbelklicka på en ref i popupen för att
  checka ut den.
- **Dubbelklicka på en ref-pill** för att checka ut den.
- **Högerklicka på en commit** för commitmenyn. Den har en undermeny för varje ref
  på commiten.
- **Högerklicka på en ref-pill** för en meny med refens åtgärder och commitens
  åtgärder.
- **Klicka på tratten i rubriken Författare** (eller `Shift+A` i Log, eller palettens `Log: Filtrera efter författare...`) för att välja vilka författares commits som förblir upplysta; resten tonas ned som vid en sökning. `Enter` eller ett klick på en rad växlar en författare, raden **Rensa filter** (✕) tar bort filtret, `Esc` stänger popupen och behåller det. **Högerklicka på den upplysta tratten** för att rensa det (paletten: `Log: Rensa författarfilter`).
- **Högerklicka på rubriken** för att visa eller dölja kolumner: en checklista (`[x]` / `[ ]`) med ett ✕ i titeln; klicka på en rad eller använd piltangenterna och `Space` / `Enter`, stäng den sedan med ✕, `Esc` eller ett klick utanför (paletten: `Vy: Log: välj vilka kolumner som visas`). Under kryssrutorna: `Anpassa <column> efter innehållet` (kolumnen du högerklickade på), `Anpassa alla kolumner efter innehållet` och `Återställ kolumnbredder`. Tangentbordstvillingen till högerklicket: `Shift+C` för att välja en kolumn, sedan öppnar `.` (eller `Shift+F10`) samma meny på den.
- **Radarns kolumner** fungerar på samma sätt: dra avgränsaren efter kolumnen för repository, `#`, författare eller ålder för att ändra storlek, **dubbelklicka på den** för att anpassa kolumnen till dess bredaste post (`C` väljer en kolumn, sedan `=`; `+` anpassar alla kolumner), och **högerklicka på en kolumntitel** (eller `C`, sedan `.`) för `Anpassa <column> efter innehållet` och `Anpassa alla kolumner efter innehållet`.
- **Klicka på `↑`, `↓` eller `✕`** i sökfältet för att gå till föregående eller
  nästa träff, eller för att stänga fältet.
- **Klicka på `#12`, `ABC-12` eller `AB#12`** i ett commit-meddelande, i högra
  panelen, för att öppna det ärendet eller den pull requesten i webbläsaren.

### Vänsterpanelens avsnitt

**Mappar** (grenar och taggar som heter `a/b/c`, filerna i högra panelen i trädvyn)
fälls ut och ihop på en gång: **högerklicka på rubriken eller på den tomma ytan i ett
avsnitt** som har mappar för **Fäll ut alla** / **Fäll ihop alla** (och
**Visa/dölj avsnitt...**, som leder till kryssruteslistan nedan), och
**högerklicka på en mapprad** för **Fäll ut alla undermappar** / **Fäll ihop alla
undermappar** (mappen och allt under den). Tangenter, i Refs-panelen, listorna
Unstaged och Staged och en commits ändrade filer lika: `=` fäller ut alla, `-`
fäller ihop alla, `*` (eller `Shift+Right`) fäller ut mappen under markören och
allt under den, `_` (eller `Shift+Left`) fäller ihop den. Paletten: *Refs: Fäll ut
alla mappar*, *Refs: Fäll ihop alla mappar*, *Refs: Fäll ut vald mapp och dess
undermappar*, *Refs: Fäll ihop vald mapp och dess undermappar* och de fyra
tvillingarna *Filer:* för den fokuserade fillistan. Medan ett namnfilter är aktivt
(Refs-filtret, en commits *Filtrera filer*) visas varje mapp som innehåller en
träff öppen oavsett vad du hade fällt ihop, och dina egna veckningar ändras eller
sparas inte: rensa filtret så kommer de tillbaka. En veckning du gör medan filtret
är aktivt verkar på det som visas och glöms bort med filtret.

**Högerklicka på en avsnittsrubrik** (ett avsnitt utan mappar, eller den tomma delen av vänsterpanelen) för en kryssruteslista över avsnitten: Lokala, Remote, Worktrees, Stashes, Taggar, Pull requests, Ärenden och Submoduler. Ett klick bockar ur eller i ett avsnitt; menyn förblir öppen. Ett dolt avsnitt ritas inte och tangentbordet hoppar över det. Det sista synliga avsnittet kan inte bockas ur. Valet gäller per arkiv och sparas mellan starter; ett arkiv utan eget val följer standarden (den globala `ui.sidebar_sections_hidden`). *Använd för alla arkiv* gör det aktuella valet till standard, och *Återställ till standard* släpper det här arkivets egna val (palett: *Sidopanel: Använd dessa avsnitt för alla arkiv*). Palett: *Sidopanel: Visa/dölj avsnitt...*, ett *Visa/dölj*-kommando per avsnitt, *Sidopanel: Återställ avsnitten till standard*.

### Dra refs

Tryck på en gren, en remote-gren eller en tag i Refs-sidofältet, eller på en
ref-pill i Log, och dra den till en annan ref. Statusraden säger vad du bär. Ett
mål som kan ta emot släppet tänds. Släpp för att få en meny. `Esc` avbryter
dragningen. Log stannar där den är medan du bär: att trycka på en ref rullar
bara Log till dess commit när det visar sig vara ett klick (hoppet sker när
knappen släpps), så du kan rulla till släppmålet först och sedan plocka upp
refen.

| Dra | Meny |
|---|---|
| Lokal gren X till lokal gren Y | **Merge X i Y**, **Rebase X på Y**, **Interaktiv rebase av X på Y...**, **Fast-forward Y till X** |
| Lokal gren till en remote-gren | **Push X** till den remoten |
| Remote-gren till lokal gren Y | **Merge**, **Rebase Y på den**, **Fast-forward Y till den**, och **Pull** när den är upstream för Y |
| Tag T till valfri gren, remote-gren eller tag | **Flytta tag T hit**; markerad *(inte en fast-forward: T skulle inte längre vara en förfader)* när målet inte härstammar från taggens commit (ditt val); nedtonad *redan där* när den redan ligger på den commiten |
| Tag T till lokal gren Y | även **Merge tag T i Y**, och längre ned den mer sällsynta **Merge Y i tag T och flytta taggen** (en merge-commit av taggens commit och Y, frånkopplad, sedan flyttas taggen till den) |

När åtgärden ändrar en gren som inte är utcheckad checkar GitPoulpi ut den
först. Om git vägrar checkouten ändras ingenting. Bara taggar kan släppas på en
tag, och en gren kan inte släppas på en.

Att flytta en tag behåller dess typ: en lättviktig tag förblir lättviktig, en
annoterad tag skapas på nytt annoterad med sitt meddelande och sin taggarrad
oförändrade (en signerad tag avvisas, dess signatur skulle inte längre stämma).
Bara din lokala tag flyttas: en remote som har taggen behåller den gamla
positionen, taggen blir *differs* och statusraden säger det; pusha den själv när
du menar det. Flytten och merge-och-flytt är vardera ett ångrasteg (ångra sätter
tillbaka det gamla tagobjektet). En merge i en tag som ger konflikter ändrar
ingenting. Palett: `Refs: Flytta vald Tag till vald Commit...` flyttar Refs-rutans
tag till commiten som är vald i Log (den frågar först när det inte är en
fast-forward); varje annat mål går via markera och släpp nedan.

Tangentbord: ”markera, sedan släpp” gör samma sak som en dragning, för vilket
par av refs som helst. Tryck `m` på en gren i Refs-rutan (paletten: `Refs:
Markera som släppkälla`). Refen får en `◆` och understrykning, även på dess
pills i Log, och statusraden säger vart du ska gå. Välj en annan ref, i valfritt
avsnitt, och tryck `Shift+M` (paletten: `Refs: Släpp markerad ref här...`):
samma meny som en dragning öppnar visas, och dess rader körs precis som efter
en dragning. Markeringen är förbrukad när menyn är öppen; `Esc`, eller `m` på
den markerade refen, tar bort den tidigare. Palettkommandot `Refs: Släpp vald
ref på utcheckad gren...` öppnar menyn för den valda refen och den aktuella
grenen.

## Högra panelen

### Lokala ändringar

- **Klicka** på en fil för att öppna dess diff. **Klicka på samma fil igen** för
  att gå tillbaka till Log. Om filen är öppen i redigeraren med osparade
  ändringar frågar GitPoulpi först.
- **Peka** på en rad för att visa dess knapp **Stage fil** eller **Unstage fil**.
  En mapprad har **Stage mapp** eller **Unstage mapp**.
- **Klicka på Stage alla ändringar** eller **Unstage alla ändringar** på en
  sektionstitel.
- **Klicka på en sektionstitel** (`Unstaged-filer`, `Staged-filer`) för att
  fälla ihop den.
- **Klicka på Sökväg eller Träd** för att välja en platt lista eller mappar.
  **Klicka på Fäll ihop alla** eller **Fäll ut alla** för att fälla ihop eller ut
  varje mapp.
- **Klicka på `✗`** till vänster om titeln för att förkasta alla ändringar
  (frågar först). **Klicka på `✦`** till höger för att förklara ändringarna med
  AI.
- **Högerklicka på en fil** för dess meny: stage, förkasta, ignorera, LFS, stasha
  filen, historik, blame, öppna i ett externt program, kopiera sökvägen, skapa en
  patch, redigera, ta bort. En fil med konflikt börjar med raderna **Lös med ...**.
- **Högerklicka på en mapprad** för mappmenyn: stage, förkasta, ignorera, stash,
  skapa en patch, öppna mappen, skapa en fil i den.
- **Högerklicka på den tomma delen** av Unstaged-filer för **Skapa fil**.
- **Ignorera**, **Behandla som binär** och **Behandla som text** (fil- och mappmenyerna) öppnar först en lista över de `.gitignore`- eller `.gitattributes`-filer regeln kan hamna i. Klicka på en rad och tryck `Enter` (eller använd piltangenterna); `Esc` eller ett klick utanför avbryter utan att skriva något.
- **Ctrl+klick** på en fil- eller mapprad lägger till den i markeringen eller tar bort den, **Shift+klick** (**Alt+klick**) markerar ett intervall; likadant i en commits ändrade filer och i Visa alla filer. Ett vanligt klick börjar om. **Högerklick på en markerad rad**: menyn för alla markerade filer.
- **Högerklick på den tomma delen** av en lista erbjuder också **Öppna repositoryts mapp**; Unstaged lägger till **Stage alla ändringar** och **Förkasta alla ändringar...**, Staged lägger till **Unstage alla ändringar**.
- **Konvertera** (fil-, mapp- och markeringsmenyer) skriver om teckenkodning, radslut eller indrag i filerna på disk; ett `Ctrl+Z` ångrar hela omgången.

### Commit-formuläret

- **Klicka** på Sammanfattning eller Beskrivning för att skriva i den.
- **Klicka på Amend föregående commit** för att slå på eller av amend.
- **Klicka på Commita ändringar i N filer** för att commita. När knappen är nedtonad säger ett
  klick varför (till exempel *Stagea ändringar för att commita*).
- **Klicka på `✦`** till höger om Sammanfattning för att skriva meddelandet med AI.
- **Klicka på `#▾`** i slutet av Sammanfattning för listan med ärenden. Klicka på ett för
  att infoga dess referens.
- **Klicka på Komponera commits med AI** för att öppna commit-composern.

### Commit-detaljer

- **Klicka** på en ändrad fil för att öppna dess diff. **Klicka på den igen** för
  att gå tillbaka till Log.
- **Dra** meddelandets nedre kant för att ändra storlek. Hjulet över meddelandet
  rullar en lång beskrivning.
- **Högerklicka** på sammanfattningen, beskrivningen eller författarblocket för
  *Ändra sammanfattning...*, *Ändra beskrivning...* eller *Ändra författare...*;
  **dubbelklicka** på en av dem (eller på författardatumet) för att öppna hela
  dialogen Ändra commit med markören där. Tangentbordsmotsvarigheterna är
  `Ctrl+G Shift+E` (alla fält) och `Ctrl+G Shift+A` (författaren); sammanfattning
  och beskrivning finns i paletten.
- **Högerklick** på en ändrad fil, en mapprad, den tomma delen av listan eller en markering ger dess meny (öppna, historik, blame, öppna i ett program, kopiera sökvägar, ignorera, behandla som binär eller text, konvertera, återställ från commiten, ta bort). **Ctrl+klick** och **Shift+klick** markerar flera filer.

Se [Stage och commit](staging-and-committing.md).

## Diffen och Filvyn

- **Klicka på en knapp** i fältet ovanför diffen: Filvy, Diff-vy, Blame,
  kolumnväljaren `Fil │ Alla`, Redigera, `↑` `↓`, Inline, Split, Hunks, WS, Bryt, Karta,
  Teckenkodning. En knapp som är på är markerad. Ett klick på `Fil` eller `Alla`
  visar den listan i commit-kolumnen; ett klick på den del som redan är vald döljer
  den (se [Commit-kolumnen](interface-tour.md#commit-kolumnen)).
- **Klicka på `✕`** i titeln för att stänga filen.
- **Peka på en hunk** för att visa **Stage hunk** eller **Unstage hunk**, och
  **Förkasta hunk**.
- **Peka på en ändrad rad** för att visa dess egna knappar **+ Stage**,
  **− Unstage** och **✗ Förkasta** längst till höger på raden.
- **Högerklicka på en rad** för hunkmenyn: stage, unstage eller förkasta hunken
  eller de markerade raderna, filhistorik, blame, kopiera.
- **Klicka på minikartan** för att flytta vyn till den raden. Håll knappen nedtryckt
  och flytta för att rulla.

Varje förkastande frågar först. När ett förkastande tar bort ospårade filer försvinner också de mappar som blir helt tomma (aldrig en som fortfarande innehåller en spårad, ignorerad eller dold fil, eller en som redan var tom); Ångra tar tillbaka filerna med sina mappar.

### Blame och filhistorik

- **Blame.** Låt pekaren vila på en annotering för hunkkortet; ett klick väljer en hunk, ett högerklick öppnar dess meny. Dra annoteringskolumnens kant för att ändra bredden (sparas). Klicka eller dra på minimappen för att rulla.
- **Filhistorik.** Ett klick väljer en commit; `Shift+klick` väljer ett intervall att jämföra; ett dubbelklick visar commiten i Log; ett högerklick öppnar radmenyn; dra kanten mellan listan och diffen för att ändra bredden; ✕ på listan döljer den (klicka på remsan med `›` för att få tillbaka den); klicka fältets `Filvy` / `Diff-vy` / `Blame` för att växla vad högra sidan visar, och väljaren `Fil │ Alla` för att välja kolumnens lista. Hjulet rullar halvan under pekaren. Var och en har sin tangentbordsmotsvarighet ([Blame](log-and-history.md#blame), [Filhistorik](log-and-history.md#filhistorik)).

### Commit-kolumnen (listan Alla)

- **Peka på en rad** för Logs verktygstips (hela meddelandet, författarens namn och e-post, hela datumet, beroende på delen under pekaren).
- **Högerklicka på en rad** för dess meny: visa den i Log, commitens diff av filen, filen vid den commiten, dess blame och sedan Logs meny för commiten. Menytangenten gör detsamma från tangentbordet.
- **Dra en gräns i rubrikraden** (`ui.column_resize`: som standard den enda cellens separator mellan två delar, markerad med ett dämpat `│` i rubriken och ritad som en accentlinje nedåt kolumnen medan pekaren är på den, där en dragning från var som helst längs den ändrar storlek; med `header` första bokstaven i nästa namn, tänd medan pekaren är på den) för att ändra storlek på delen till vänster om den: spår, meddelande, författare, datum, refs, ändringar eller SHA. Cellen direkt efter den sista delen är den sista delens gräns.
- **Dubbelklicka på en gräns i rubriken** för att anpassa delen till vänster om den efter dess innehåll, som Logs gränser gör: den bredd som visar dess bredaste rad helt, aldrig bredare än kolumnen (tangentbordet: klicka på en rad eller `Ctrl+N` för att ge kolumnen tangenterna, `Tab` för att välja delen, sedan `=`; `+` anpassar alla delar; paletten har `Anpassa kolumnen efter innehållet` och `Anpassa alla kolumner efter innehållet`). Meddelandet behåller det de andra delarna lämnar kvar.
- **Dra ett namn i rubriken** över ett annat för att flytta delen dit: samma spöke, dämpade källa och accentstapel som Logs rubrik, och delen flyttas när du släpper (`Esc` avbryter; tangentbordets `<` / `>`, efter `Tab`, gör detsamma en plats i taget).
- **Högerklicka på rubriken** för att välja vilka delar som visas (Ref, Graf, Författare, Datum, Ändringar, SHA; Meddelande visas alltid); checklistan har ett ✕ i titeln. Under den: `Anpassa <part> efter innehållet` (delen du högerklickade på) och `Anpassa alla kolumner efter innehållet`. Paletten har `Välj kolumner` (menyn från tangentbordet, på delen som `Tab` valde) och en växling per del.
- **Shift+hjul** eller ett horisontellt hjul rullar raderna i sidled när de är bredare än kolumnen.
- **Dra i den högra kanten** av kolumnen för att ändra dess bredd. **Klicka på remsan `›`** (eller ✕ på kolumnen, eller **klicka på den valda delen** av fältets väljare `Fil │ Alla`) för att få tillbaka den eller dölja den.

### Markera och kopiera text

Du kan markera och kopiera texten i diffen, Filvyn, Blame och Filhistorik.

- **Tryck och dra** för att markera text. Markeringen stannar på samma text när du
  rullar.
- **Dubbelklicka** för att markera ett ord.
- **Ett vanligt klick** markerar inte text. Det flyttar markören eller markerar
  raden, och rensar textmarkeringen.
- **`y` eller `Ctrl+C`** kopierar markeringen. Utan markering kopierar de den
  aktuella raden.
- **Högerklicka** i diffen för **Kopiera markering**, **Kopiera rad** och
  **Kopiera hunk**.

GitPoulpi kopierar bara filens text. Radnummer, `+`- och `-`-märken,
hunkrubriker och Blame-kolumnerna kopieras inte. I Split stannar en markering på
den sida där du började dra.

I diffen markerar en dragning också raderna den korsar, som `v` gör. `Enter`
stagear sedan de raderna. `Esc` rensar först textmarkeringen, sedan
radmarkeringen och stänger därefter diffen.

Att markera text färgar också varje annan plats där samma text förekommer i
diffen och redigeraren.

## Textfält

Det här gäller varje textfält: filter, sökfältet, paletten, Sammanfattning och
Beskrivning, dialogfält och värden i Inställningar.

- **Klicka** för att sätta markören där.
- **Tryck och dra** för att markera text.
- **Dubbelklicka** för att markera ett ord.
- Skriv för att ersätta markeringen. `Backspace` eller `Delete` tar bort den.

Tangenterna för textfält står i [Kortkommandon](keybindings.md).

## Redigeraren

- **Klicka** för att sätta markören där. **Shift+klick** utökar markeringen.
- **Dra** för att markera. Dra förbi övre eller nedre kanten för att rulla.
- **Dubbelklicka** för att markera ett ord.
- **Hjulet** rullar utan att flytta markören.
- **Klicka på Spara, Ångra, Gör om eller Sök** i fältet. I sökläge går **`↑`** och
  **`↓`** till föregående och nästa träff.
- **Klicka på `✕`** för att stänga. GitPoulpi frågar först när det finns
  osparade ändringar.

## Bilder

Se [Bilder](image-view.md) för musen i bildvyn.

## Flytta och ändra storlek på dialogrutor

Dialogrutor kan flyttas, och de större fönstren kan ändras i storlek:

- **Dra i titelraden** för att flytta en dialogruta (stängningskorset behåller sitt klick). Kommandopaletten och väljarna har ingen egen titelrad: dra i den övre kanten.
- **Dra i vänster, höger eller nedre kant**, eller i ett nedre hörn (greppet `◢`), för att ändra storlek på ett fönster. Dialogrutan stannar på skärmen och blir aldrig mindre än sin minimistorlek (60 x 16 för Ångrahistoriken, mindre för de andra). Ett fönster med ändrad storlek utnyttjar utrymmet: textfält och listor växer, och texten radbryts efter den nya bredden.
- Draget fortsätter när pekaren lämnar dialogrutan, och att släppa utanför stänger den aldrig.
- **Dra i avdelarna** i Ångrahistorikens vy Reflog / Förlorat arbete; dubbelklicka på en för att återställa den.
- Tangentbordstvillingarna är `Ctrl+N` och sedan `m` (piltangenterna flyttar, `Shift`+pilar ändrar storlek, `0` återställer, `Enter` eller `Esc` lämnar) och `Alt`+pilar för avdelarna; paletten har `Dialogruta: Flytta/ändra storlek` och `Dialogruta: Återställ position och storlek`. De verkar på dialogrutan överst. `Ctrl+N` fungerar likadant över varje yta: det visar ett märke på kontrollerna i fönstret, dialogrutan eller popupen längst fram (fält, listor, flikar, knappar och stängningskorset), tangenten efter det trycker på den kontrollen, `m` startar arrangeringsläget och `Esc` avbryter; statusraden anger `<märke>`, `m=arrangera` och `Esc`. Det gäller även kommandopaletten, väljarna, arbetsytesökningen och hjälpöverlägget, där `Ctrl+N` inte längre flyttar nedåt i listan. `Alt+G` (`palette.arrange`) är ännu ett sätt, giltigt över varje dialogruta och popup, att arrangera den längst fram utan prefixet.
- **En enda inställning** (`ui.dialog_geometry`, Inställningar > Utseende > Layout) gäller alla dialogrutor: var och en kommer ihåg sin egen plats och storlek under den här sessionen, på disk till nästa start, eller aldrig.

| Dialogruta | Flytta | Ändra storlek |
|---|---|---|
| Ångrahistorik / Reflog / Förlorat arbete, Aktivitetslogg, jobbpanel, mappväljare | ja | ja |
| Kommandopalett, flik- / tema- / språkväljare, arbetsytesökning | ja | ja (listan växer med höjden, raderna med bredden) |
| Klona (förvarslistan), Pull request och Nytt ärende (beskrivningen), "Vad som skickas" (rutan), Mermaid (förhandsvisningen), granskningsalternativ (fälten, bara bredd) | ja | ja |
| Långtextredigerare i Inställningar (mallar, hookar, AI-prompter), arbetsytsformulärets AI-ruta | ja | ja (textfältet) |
| Historikomskrivning: de två AI-hjälpdialogrutorna | ja | ja (beskrivningen, rutan med det som skickas) |
| Om, uppdateringspanel, säkerhetskontroll, förutsedda konflikter, "Över AI-gränsen", borttagning av tagg, känsliga filer | ja | nej (storleken följer texten) |
| Init, Worktree, Sparse checkout, Flow-dialogruta, ärendeflödesdialogrutor, meddelandeformulär (reword) för commit, Arbetsytsformulär, etikett- / milstolpeväljare | ja | nej |
| Bekräftelser (varje "Är du säker?") | ja | nej |
| Gransknings- och tavledialogrutor: deras små dialogrutor (kommentar, avsluta, välj, kolumner) | ja | nej |
| Popupar på sidan Tangentbord (bekräftelse av återställning, tangentfångst), AI-svarspopupen | ja | nej |
| Inställningar: resultat och bekräftelse för förvarsautentisering | ja | nej |
| Historikomskrivning: dialogrutor för rensning, identitet och frågor | ja | nej |
| Flytta-fil-popupen i commit-redigeraren, sammanslagningsmeddelanden | ja | nej |

Kan inte flyttas: helsidesvyerna (Inställningar, Hjälp, sidorna Granskning och Tavla, som redan fyller fönstret), snabbmenyer och rullgardinslistor (de hänger på det som öppnade dem), verktygstips och de små popuparna inne på en sida (formuläret för fjärrförvar, redigeraren för delade commits).

Pekaren visar en flyttform över titelraden och en storleksform över kanterna och hörnen (terminaler med `OSC 22`).

## Muspekarens form

Vissa terminaler låter GitPoulpi ändra muspekaren: en storleksändringspekare över
avdelare, över en kolumngräns i Log, Radar och commit-kolumnen (också medan du drar
den), en hand över klickbara objekt. Det fungerar i kitty, foot, Ghostty,
mintty och ett WezTerm-bygge med uppströms PR #8092 (sekvensen `OSC 22`). Den
skickas bara när formen ändras, inuti tmux bara inpackad för genomsläppet
(`allow-passthrough on`; kitty och foot visar den, Ghostty inte) och aldrig i
screen, zellij eller herdr. Standardpekaren återställs när GitPoulpi avslutas.
I andra terminaler (Rio, Alacritty, Tabby, iTerm2, Terminal.app, Windows
Terminal, xterm) ändras inte pekaren; se
[Välja terminal](terminals.md#muspekarens-form-osc-22). Avdelaren eller
knappen under pekaren tänds i varje terminal.

Samma sorts komponent ger samma pekare överallt i programmet: knapparna i en granskning som visas över Radar (flikar, stängkorset, popupknappar) och Radars egna Åtgärder-knappar visar handen som varje annan knapp.

- **Knappar är riktiga knappar**, i Inställningar (Integrationer: Redigera, Koppla från,
  Ta bort, Bläddra, Radera, snippet-knapparna i instruktionsredigeraren ...)
  liksom i dialoger: etiketter med utfyllnad på en fylld bakgrund som ljusnar
  under pekaren, nedtonade när de är avstängda, aldrig text i `[ ]`. De enda
  hakparenteser som finns kvar är kryssrutornas markeringar `[x]` / `[ ]`, som är
  kryssrutor och får handen som varje annan kontroll.
- **Pekande hand** visas på kontroller: knappar, kryssrutor och växlare,
  kombinationsrutor (i Inställningar, ärende- och pull request-panelerna, Radar,
  dialoger), flikar och deras kryss, `+`-knapparna, länkar och referenser
  (`#12`, `!45`), grupp- och mapprubriker med sin expanderpil, ref-pillren,
  rullningslisterna samt statusradens knappar för Aktivitet, jobb och version.
  En kryssruta eller knapp i en rad behåller handen.
- **Pilen** visas på rader. En rad lyses upp som helhet när pekaren är på den
  (samma ljus i varje lista) och är inte en kontroll i sig: raderna i panelerna
  Refs, Worktrees, Stashes och Submodules, Log och commit-kolumnen (bara ett
  ref-piller får handen), marginalerna i en diff, de ändrade filerna, pull
  request-, ärende- och Radar-posterna samt raderna i listorna i dialoger och
  paneler (klona, mappväljaren, Workspaces, tavlans kort, Jobs-panelen, väljarna
  för ansvarig, etikett och milstolpe, rebase-steg, granskningens filer och
  commits, val i menyer och kombinationsrutor).
- **Textmarkören (I-balk)** visas på inmatningsrutor, över hela rutan:
  sökrutorna (global sökning, Inställningar, kommandopaletten, Home), filtren
  (Refs, filer, Radar, Aktivitetsloggen, ärendepanelens sökning), text-, tal-
  och områdesraderna i Inställningar (också medan du redigerar dem), dialogernas
  fält, granskningens kommentarsrutor, commit-kompositören, filredigerarens text
  och merge-redigerarens redigerbara resultat (dess skrivskyddade paneler behåller
  pilen, eftersom en dragning där inte markerar något). En regel gäller även
  skrivskyddad text: överallt där en musdragning kan markera text är pekaren
  I-balken. Det är koden i en diff (Diff-vyn, Inline och Split, och
  granskningens diffruta), Filvyn och Blames kod. Deras radnummer, markörer och
  hunkrubriker behåller pilen, och en knapp på en rad behåller handen. Detsamma
  gäller konversationen i ett ärende eller en begäran (även granskningens
  Översikt och dess utkastblock), commit-detaljernas meddelande, Release notes,
  Aktivitetsloggen (båda flikarna), hjälppopupen, Markdown-förhandsvisningen i
  kommentarsredigeraren och kommentarerna, de väntande anteckningarna och
  förslagen inuti granskningens diff: en dragning markerar texten (ett vanligt
  klick behåller sin betydelse: en länk öppnas, en knapp agerar, en post i
  Aktivitetsloggen eller en tråd öppnas eller viks), `Ctrl+C` kopierar den, `Esc`
  eller ett klick släpper den. En markering följer sin text när du rullar.
  Commit-meddelandet är också redigerbart (dubbelklick ändrar det), och hela
  blocket lyses upp lätt.
- **I commit-detaljerna** är inforaderna kontroller: klicka på `commit: <id>`
  för att kopiera det korta id:t, dubbelklicka på författar- eller datumraden för
  att ändra den (handen och en lätt markering visar det). Statusraden bara
  informerar och behåller pilen.

Storleksändringspekarna ligger kvar över avdelare och kolumngränser, och den
öppna handen visas bara medan du bär en ref.

## Kopiera till urklipp

Kopieringsåtgärder lägger texten i systemets urklipp på den maskin där GitPoulpi
körs och skickar den, där terminalen är känd för att respektera det, också till
terminalen (OSC 52). Över SSH, eller utan skärm, skickas bara OSC 52: om din
terminal inte stöder det (iTerm2 tills dess urklippsbehörighet är på,
Terminal.app, GNU screen) kopieras ingenting och statusraden säger att texten
bara skickades till terminalen. Se
[Välja terminal](terminals.md#urklipp-kopieringsåtgärder).
