[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · **Svenska** · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Skriva om historiken

Att skriva om historiken ändrar commits som redan finns: vem som skrev dem och
när, deras meddelanden, filerna de innehåller och deras signaturer. GitPoulpi gör
det med ett **recept** (ett omfång och en ordnad lista med regler) och tre steg
där det mittersta aldrig hoppas över:

1. **Förhandsgranska** beräknar den nya historiken utan att röra något: antal,
   vad varje regel gjorde, vilka grenar och taggar som skulle flyttas och en rad
   per ändrad commit.
2. **Tillämpa** flyttar dina lokala grenar och taggar dit. Det är en post i
   [ångrahistoriken](undo-redo.md): **Ångra** sätter tillbaka varje flyttad ref
   (och de utcheckade filerna).
3. **Push** (en force push med lease) och **Rensa** är separata, medvetna steg,
   eftersom de inte går att ångra: se [Skicka resultatet](#skicka-resultatet) och
   [Radera den gamla historiken](#radera-den-gamla-historiken-rensning).

Omskrivningsvyn, paletten, kommandona `gitpoulpi rewrite` och MCP-verktygen kör
samma motor, så en förhandsgranskning som gjorts i en av dem är det en annan
tillämpar. [Från kommandoraden](#från-kommandoraden) listar flaggorna;
[Skapa regler med AI](#skapa-regler-med-ai) fyller omskrivningsvyn utifrån en beskrivning;
[Förslag från AI-agenter](#förslag-från-ai-agenter) säger vad en agent får och
inte får göra här.

**Reglerna, efter vad de ändrar**

| Grupp | Regler | Avsnitt |
|---|---|---|
| Författare | Byt ut en författare, Anonymisera författare, Tillämpa .mailmap | [Författare och .mailmap](#författare-och-mailmap) |
| Datum | Förskjut datum, Byt tidszon, Begränsa till vissa timmar, Committerdatum = författardatum | [Datum](#datum) |
| Meddelanden | Sök och ersätt, Ta bort rader, Lägg till eller ta bort en trailer, Rätta meddelandenas teckenkodning | [Meddelanden](#meddelanden) |
| Filinnehåll och sökvägar | Sök och ersätt i filer, Ta bort filer, Ta bort stora filer, Flytta eller byt namn på sökvägar, Konvertera radslut, Ta bort körbehörigheten, Konvertera filernas teckenkodning | [Filer och sökvägar](#filer-och-sökvägar) |
| LFS | Migrera filer till LFS, Ta tillbaka filer från LFS | [Flytta filer till eller från Git LFS](#flytta-filer-till-eller-från-git-lfs) |
| Signaturer | Ta bort signaturer (och omsignering vid Tillämpa) | [Signaturer](#signaturer) |
| Skript | Lua, shell, Python | [Skriptregeln](#skriptregeln) |

**Innan du börjar**

- En förhandsgranskning ändrar ingenting. Tillämpa ändrar bara dina lokala refs
  och pushar aldrig.
- Omskrivna commits får nya id. Alla som har den gamla historiken måste få veta
  det: [Skicka resultatet](#skicka-resultatet) skriver noteringen.
- En hemlighet som har pushats är läckt. **Rotera den först**, skriv sedan om,
  force-pusha och rensa.
- Commits som förblir som de är behåller sina id och sina signaturer.

## Omskrivningsvyn

`Ctrl+G Shift+H` (eller **Omskrivning: Öppna omskrivningsvyn** i paletten)
öppnar omskrivningsvyn över Log. Den har ingen knapp i verktygsfältet: de
avancerade åtgärderna finns i paletten (varje rad `Omskrivning: ...` öppnar vyn
med den regeln redan tillagd), i undermenyn **Skriv om historiken** i
commit-menyn (*Från denna commit till HEAD...*, *Valda commits...* när flera är
markerade, *Öppna omskrivningsvyn...*) och bakom tangentkombinationen.

`Ctrl+N` visar ett märke på varje kontroll i omskrivningsvyn (omfång, regler, fält, flikar, knappar); att trycka på ett märke gör vad ett klick på kontrollen gör, och `Esc` tar bort märkena (se [Märken](keybindings.md#märken)).

![Arbetsbänken för att skriva om historiken: omfångsfältet, en lista med två regler av typen Byt ut en författare, fälten för den valda regeln och förhandsgranskningens sammanfattning av hur många commits som ändras](../images/history-rewrite.png)

- **Omfångsraden:** vilka refs som flyttas (aktuell gren, alla lokala grenar,
  valda grenar och taggar, de markerade commits), rutan *Taggar* och, efter
  *Från denna commit till HEAD...*, en ruta för att börja efter den commitens
  förälder.
- **Regler:** en numrerad lista som tillämpas i ordning. Rutan slår på eller av
  en regel, pilarna flyttar den, krysset tar bort den (*Återställ den borttagna
  regeln* eller `Ctrl+Z` tar tillbaka den). *Lägg till regel* visar sorterna
  grupperade. En regel som kräver ett program du saknar förblir nedtonad, med
  orsaken.
- **Regelinställningar:** ett klick på ett textfält redigerar det; ett klick på
  ett fält med flera värden öppnar dess lista; rutor växlar.
- **Förhandsgranskning** (`F5`) beräknar den nya historiken utan att röra något
  och visar en förloppsrad. Flikarna: **Sammanfattning** (antal, vad varje regel
  gjorde, varningar), **Commits** (en rad per ändrad commit, hämtad sida för sida
  från daemonen; filtret visar ändrade, borttagna eller pushade), **Grenar och
  taggar** (vad som flyttas, och de skyddade grenar som redan är pushade, var och
  en med en ruta), **Påverkan** (vad det innebär att skriva om pushad
  historik: en force push med lease, medarbetare, signaturer), **Hemligheter**,
  **Identiteter** och **Storlek** (se nedan).
- **Filerna i en commit:** `Enter` (eller dubbelklick) på en rad i Commits öppnar
  de filer som innehållsreglerna ändrade i den och, bredvid dem, diffen för den
  markerade. Krysset, `Esc` eller ett klick utanför stänger den. Borttagna rader
  från en hemlighetsregel är dolda.
- **Tillämpa** (`Ctrl+S` eller `Ctrl+Enter`) går att använda när
  förhandsgranskningen stämmer med reglerna. Frågor som saknar standardsvar
  ställs varje gång: vad som händer med signaturerna på signerade commits och
  hur en mapp extraheras. Resultatpanelen erbjuder **Ångra**, som sätter tillbaka
  varje flyttad ref.
- **Spara / Läs in** lagrar ett recept i repositoryt (delat) eller bara för dig;
  hemliga värden sparas aldrig.

Varje knapp har en tangent: listan finns i Inställningar > Tangentbord & mus
(kontexten *Omskrivningsvyn*) och i guiden `keybindings.md`.

### Författare och .mailmap

- **Byt ut en författare** matchar en författare eller committer på namn och/eller
  e-post (en bokstavlig text eller ett reguljärt uttryck) och sätter ett nytt
  namn och/eller en ny e-postadress. *Gäller* väljer författaren, committern
  eller båda; *Även författarna till taggar* låter de annoterade taggar som
  skapas på nytt följa med. Den ursprungliga committern behålls om inte körningen
  säger *jag, nu*.
- **Anonymisera författare** ersätter varje identitet med en numrerad
  platshållare (`Contributor 1`, `contributor-1@anonymous.invalid`; båda
  mönstren kan du ändra, med `{n}` för numret). Samma person får alltid samma
  nummer i en körning; identiteter du listar som *lämna orörda* lämnas som de är.
- **Fliken Identiteter** (även **Omskrivning: Slå ihop identiteter (.mailmap)...**
  i paletten) är sättet att rätta en person som har committat under flera namn
  och e-postadresser **utan att skriva om något**. Den listar varje namn och
  e-postadress i omfånget på omfångsraden med hur många commits var och en har
  skrivit och committat, och vilka som repositoryts `.mailmap` redan mappar
  (`kopplad till Name <email>`). Markera de identiteter som är samma person
  (`Space`, eller ett klick; *Markera alla* / *Avmarkera alla*) och välj sedan
  **Slå ihop till...** (`m`): en dialog frågar vilken av dem som ska stanna kvar
  (den med flest commits står först) och **lägger till en rad per övrig identitet
  i `.mailmap`** (`Name <email> Old Name <old@email>`; en rad som redan finns
  upprepas inte, filen skapas om den saknas). Ingenting committas: committa
  `.mailmap` själv för att dela den. Log, blame och filhistorik visar det
  kanoniska namnet direkt (inställningen `log.use_mailmap`, på som standard; de
  lagrade commits ändras aldrig, så det finns inget att force-pusha). Statusraden
  säger vad som skrevs och erbjuder **Ångra** (`u`), som sätter tillbaka den
  tidigare `.mailmap` (det är en post i ångrahistoriken, som alla andra
  åtgärder). `r` läser listan igen.
- **En `.mailmap` i stället för en omskrivning.** När en författarregel ändrar
  commits som redan finns på en remote säger Sammanfattningen *En .mailmap rättar
  visningen utan att skriva om något. Använda .mailmap i stället?* med en knapp
  **Använd .mailmap** (`m` på Sammanfattningen). Den öppnar fliken Identiteter
  med de identiteter som regeln träffar markerade, och med identiteten som
  regeln skulle sätta föreslagen som den att behålla, även om ingen commit har
  den ännu.
- **`.mailmap`-regeln** (*Omskrivning: Tillämpa .mailmap på historiken...*, eller
  `rewrite preview --mailmap`) skriver faktiskt om: författare och committers
  (du väljer) blir de namn som worktreens `.mailmap` anger, det resultat som
  `git log --use-mailmap` visar. Använd den när själva historiken ska bära de
  kanoniska namnen, till exempel före en publicering; det är en omskrivning som
  alla andra (förhandsgranska, Tillämpa, force push).
- **Identitetsregler och trailers.** En identitetsregel kan också ändra de
  personer som nämns i trailers: markera **Ändra även trailers**
  (`--map-trailers`). Varje trailer vars värde är exakt `Name <email>` och
  matchar regelns *E-posten är* / *Namnet är* får det nya namnet och den nya
  e-postadressen, oavsett vad regeln gör med författaren och committern. Den är
  av som standard: en `Signed-off-by` visar vem som har signerat, så att ändra
  den är ett eget beslut.

### Datum

De fyra datumreglerna verkar på författardatumet, committerdatumet eller båda
(*Datum* i regelns ruta).

- **Förskjut datum** flyttar varje datum med ett belopp (`+2h`, `-1d`, `90m`,
  `1d 2h`). Tidszonen som varje commit skrevs i behålls.
- **Byt tidszon** skriver datumen med en annan UTC-förskjutning. *Behåll* säger
  vad som står kvar: **tidpunkten** (samma ögonblick, klockan visar något annat)
  eller **klockan** (samma lokala tid, ögonblicket ändras).
- **Begränsa till vissa timmar** flyttar varje commit som hamnar utanför de
  tillåtna timmarna (till exempel `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) till
  den närmaste tillåtna tiden, eller bara tidigare, eller bara senare (*Flytta
  till*). *Aldrig före föräldern* bevarar historikens ordning; *Startvärdet* gör
  den lilla slumpmässiga spridningen upprepbar: samma startvärde på samma
  historik ger samma datum.
- **Committerdatum = författardatum** sätter committern (namn, e-post och datum)
  till författarens, vilket är det en `git commit --amend` eller en rebase har
  ångrat.

Datum är regler som alla andra: förhandsgranska, kontrollera fliken **Commits**
(varje rad visar det gamla och det nya datumet) och tillämpa sedan. En commits id
beror på dess datum, så en datumregel ändrar varje commit efter den första den
rör.

### Meddelanden

- **Sök och ersätt i meddelanden** och **Ta bort meddelanderader** tar en
  bokstavlig text eller ett reguljärt uttryck (*Reguljärt uttryck*, *Skilj på
  versaler*); en tom ersättning tar bort. Ett meddelande som inte är giltig UTF-8
  rörs aldrig av dem: kör **Rätta meddelandenas teckenkodning** först om du vill
  redigera det.
- **Trailers.** *Lägg till eller ta bort en trailer* (*Omskrivning: Lägg till
  eller ta bort trailers...* i paletten, eller `--trailer-add`,
  `--trailer-remove`, `--trailer-map`) redigerar raderna av typen `Key: value`
  i slutet av ett meddelande, de som `git interpret-trailers` känner till
  (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...). Listan **Åtgärd** väljer
  *Lägg till* (nyckeln **Trailer**, dess **Värde** och **Om den redan finns**:
  *Lägg till (om den inte är identisk)*, *Ersätt de befintliga* eller *Lämna
  commiten orörd*), *Ta bort* (varje trailer med den nyckeln, eller bara de vars
  värde innehåller **Värdet innehåller**) eller *Ändra värdet* (**Sök i värdet**
  blir **Ersätt med**, till exempel en gammal e-postadress mot en ny i
  `Signed-off-by`). Nycklar matchas utan hänsyn till versaler. Bara
  trailerraderna ändras, resten av meddelandet blir exakt som det var, och ett
  meddelande som inte har några trailers ännu får en tom rad och ett eget block.
  Fliken Commits visar varje ändring som `+Signed-off-by: ...` /
  `-Co-authored-by: ...`. Ett meddelandes sista stycke är dess trailerblock
  (aldrig rubriken), så en rad som `Note: see the ticket` i brödtexten är ingen
  trailer om den inte står ensam i sista stycket tillsammans med andra trailers.
- **Commit-id i meddelanden.** **Uppdatera id-referenser till commits**
  (`--update-message-hashes`, av som standard; längst ned i rutorna för de regler
  som redigerar meddelanden) får id:n som står i meddelanden att följa med i
  omskrivningen: ett förkortat id på 7 eller fler hextecken (`This reverts commit
  1a2b3c4`, `see 1a2b3c4d5e`) som pekar ut exakt en commit i den omskrivna
  historiken ersätts med den commitens nya id, kortat till samma längd. Ett som
  matchar flera commits, ingen, eller en commit som omskrivningen skapar senare
  lämnas som det är; Sammanfattningen säger hur många som ersattes och hur många
  som lämnades.
- **Rätta meddelandenas teckenkodning** (*Omskrivning: Rätta meddelandenas
  teckenkodning...*, eller `--fix-message-encoding [--fix-name-encoding]`) kodar
  om meddelanden till UTF-8, i två fall:
  - En commit med en `encoding`-rubrik (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, allt som git och kodningstabellerna känner till) avkodas och
    skrivs som UTF-8, och rubriken tas bort: texten läses likadant överallt, utan
    någon konvertering. Författarens och committerns namn i den commiten hade
    samma kodning och avkodas med den, så en commit lämnas aldrig halvt
    konverterad.
  - **Mojibake**, UTF-8 som en gång lästs som Latin-1 eller Windows-1252
    (`RÃ©Ã©criture` i stället för `Réécriture`), lagas **bara när omvandlingen går
    att göra om exakt**: texten måste kodas tillbaka till byte som är giltig
    UTF-8, och att förvanska resultatet på samma sätt måste ge originalet. Ett
    meddelande som bara innehåller `Ã©` bredvid riktiga bokstäver, eller tecken
    utanför de gamla alfabeten, klarar inte det och lämnas som det är.

  *Rätta även namn* (`--fix-name-encoding`) lagar förvanskade författar- och
  committernamn på samma sätt. Det som inte går att avgöra (ett okänt kodningsnamn,
  byte som inte är giltiga i den angivna kodningen, ett meddelande som inte är
  UTF-8 och inte anger något, text som ser förvanskad ut men inte bevisligen är
  det) **lämnas orört och räknas**: Sammanfattningen säger *n meddelanden
  lämnades som de är: deras teckenkodning går inte att avgöra*, och
  `counts.encoding_left` har antalet. Fliken Commits visar varje rättelse med det
  gamla och det nya meddelandet och en *teckenkodning*-rad (`ISO-8859-1` blir
  `UTF-8`).

### Filer och sökvägar

- **Regler för filinnehåll** (*Sök och ersätt i filer*, *Ta bort filer*, *Ta bort
  stora filer*): rutan kontrollerar varje värde medan du skriver och visar
  orsaken under fältet (ett mönster som inte går att kompilera, ett ogiltigt
  sökvägsmönster, en storlek under en byte). Storlekar tar enheter (`500 KiB`,
  `5 MiB`, `1 GiB`). Ett sökvägsmönster utan snedstreck matchar på alla djup.
  *Hemliga värden* skrivs i ett maskerat fält, används bara för den här körningen
  och sparas aldrig; förhandsgranskningen döljer raderna som innehöll dem.
  *Tomma commits* avgör om en commit som reglerna lämnade tom tas bort
  (standard) eller behålls. Binärfiler och filer över storleksgränsen lämnas som
  de är och räknas i Sammanfattningen.
- **Flytta sökvägar.** *Flytta eller byt namn på sökvägar* (*Omskrivning: Flytta
  eller byt namn på sökvägar...* i paletten) har tre sorter, som väljs i listan
  **Flytta**:
  - *Byt namn på en sökväg* (**Från**, **Till**; `--move-path FROM=TO`): samma
    filer och mappar dyker upp på den nya sökvägen i varje commit som hade den
    gamla; inget annat ändras, varje fil behåller sitt id. En commit där den nya
    sökvägen redan finns skrivs aldrig över: förhandsgranskningen avvisas och
    listar commits och sökvägar som krockar (välj ett ledigt namn, eller flytta
    den befintliga först). Bara exakta sökvägar, inga jokertecken.
  - *Flytta allt in i en mapp* (**Mapp**; `--into-subdir DIR`): varje sökväg i
    varje commit får prefixet. Användbart för att slå ihop repositories utan
    namnkrockar.
  - *Gör en mapp till roten* (**Mapp**; `--extract DIR`): mappen blir roten,
    commits som inte rör den tas bort (en commit som skapar den får ingen
    förälder), merges som inte längre sammanför något förenklas, och grenar och
    taggar som inte har något av mappen tas bort. Du får **varje gång** frågan hur
    det ska sluta:
    - *Till en ny klon, öppnad i en ny flik* (`--extract-mode clone`): en kopia av
      det här repositoryt skapas i en mapp du väljer (det nya repositoryt får en
      mapp uppkallad efter den extraherade inuti), skrivs om där och städas så
      att det bara innehåller den nya historiken, varje lokal gren och tag som har
      mappen. Det här repositoryt rörs inte. Det nya repositoryt har ingen remote
      (originalet är inte dess upstream).
    - *På plats: det här repositoryts historik skrivs om* (`--extract-mode
      in-place`): den vanliga omskrivningen, förhandsgranska, Tillämpa, ångra.
      Grenar utan mappen tas bort (ångra tar tillbaka dem); den utcheckade
      grenen måste ha den.
    - *Som en submodul i det här repositoryt* (`--extract-mode submodule`): det
      nya repositoryt skapas som i klonfallet, sedan ersätter **en ny commit** på
      aktuell gren mappen med en submodul av det (`.gitmodules` och länken),
      mappen finns kvar som en utcheckad submodul. Det här repositoryts historik
      skrivs **inte** om: dess gamla commits behåller mappens filer, så inget som
      pekar på dem (taggar, andra kloner) går sönder. URL:en i `.gitmodules` är
      det nya repositoryts lokala sökväg: ändra den (`git config -f .gitmodules
      submodule.<mapp>.url`) när det finns på en värd. Ändrade eller ospårade
      filer i mappen avvisar det. Ångra sätter tillbaka den vanliga mappen.

    Sammanfattningen säger att förhandsgranskningen visar det nya repositoryts
    historik och vilken mapp det hamnar i. Regler som kommer efter en extraktion
    (ett namnbyte, en borttagning) gäller det extraherade trädet; för en klon
    eller en submodul måste extraktionen vara den första sökvägsregeln.
    Sökvägsregler körs efter innehållsreglerna, så en borttagningsglob anger
    sökvägarna som de var.

#### Radslut, körbehörighet och filkodningar

Tre regler i gruppen **Innehåll** i Lägg till-menyn (eller paletten med raderna
*Omskrivning: Konvertera radslut...*, *Ta bort körbehörigheten...* och
*Omskrivning: Konvertera filernas teckenkodning...*). Alla tre tar **Filer**
(globs; ett mönster utan snedstreck matchar på alla djup; tomt betyder varje fil)
och **Utom filerna**. De ändrar bara det globarna pekar ut, i varje commit i
omfånget. Binärfiler (en NUL-byte bland de första 8 KiB) rörs aldrig:
Sammanfattningen räknar dem som *lämnade som de är*.

- **Konvertera radslut** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Konvertera till*: LF (Linux, macOS), CRLF (Windows)
  eller CR (klassisk Mac). Varje radslut i en matchande textfil blir det
  slutet, så en fil med blandade radslut blir enhetlig; filer som redan är rätt
  behåller sin version orörd. *Följ .gitattributes* (av som standard,
  `--eol-follow-attributes`) läser rotens `.gitattributes` i varje commit: filer
  markerade `-text` eller `binary` hoppas över och filer markerade `eol=lf` eller
  `eol=crlf` behåller det radslutet i stället för det du valde.
- **Ta bort körbehörigheten** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Matchande filer går från läge 100755 till 100644; deras innehåll
  rörs inte. Minst ett filmönster krävs: att ta bort den från alla körbara filer
  skulle förstöra dina skript.
- **Konvertera filernas teckenkodning** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Konvertera till*: UTF-8 (standard), UTF-8 med BOM, UTF-16 LE eller
  BE (skrivs med en BOM), Windows-1252 eller ISO-8859-1. *Filerna är i*: som
  standard identifieras varje fils kodning (en BOM, annars giltig UTF-8, annars
  *Reserven*, Windows-1252 om du inte ändrar den); ange kodningen för att tvinga
  fram den, men ett felaktigt val ger fel text (UTF-8 läst som Latin-1 dubblerar
  varje accent). En fil som källkodningen inte kan läsa, eller som innehåller ett
  tecken som målet inte kan lagra (ett eurotecken i ISO-8859-1), lämnas exakt som
  den är och räknas i Sammanfattningen.

Reglerna körs i listans ordning, på samma fil: att konvertera en fil till UTF-16
och sedan be om radslut lämnar den orörd (den är inte längre vanlig text). Att
tillämpa är en post i ångralistan som vilken omskrivning som helst.

### Hitta hemligheter

- **Hitta hemligheter:** `gitpoulpi rewrite scan-secrets` (och MCP-verktyget
  `rewrite_scan_secrets`) läser historiken en gång och listar tokenprefix
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), privata
  nycklar, JWT, lösenord i URL:er och slumpmässigt utseende rader av typen
  `password = value`, var och en med sin fil, första och sista commit och en
  maskerad förhandsvisning som `ghp_****abcd`. Sökningen är inte uttömmande, och
  värdet visas, loggas eller sparas aldrig. `--remove-finding ID` gör ett fynd
  till en regel som tar bort det ur varje commit. En hemlighet som har pushats är
  läckt: rotera den först, skriv sedan om, force-pusha och rensa.
- **Fliken Hemligheter** gör samma sak i omskrivningsvyn, för omfånget på
  omfångsraden: **Sök efter hemligheter** (`s`) kör sökningen som ett jobb
  (förloppsraden heter *Söker*, **Avbryt** eller `Esc` stoppar den). Varje fynd
  är en rad med sin sort, den maskerade förhandsvisningen, filen och hur många
  andra som innehåller det; under listan: första och sista commit där det sågs
  och hur många commits och filversioner som innehåller det. Fliken berättar hur
  många binärfiler och för stora filer som inte lästes, och att sökningen inte är
  uttömmande. Alla fynd är markerade från början; **Ta bort ur historiken** (`r`)
  gör de markerade till en enda regel som pekar ut dem med id (aldrig med värde),
  som du förhandsgranskar och tillämpar som vilken annan regel som helst. Fynd
  blir inaktuella efter en omskrivning eller en ändring av omfånget: fliken säger
  det, sök igen. Omskrivningsvyn håller aldrig en hemlighets värde, så den kan
  inte visa det.

### Hitta det som väger
- **Hitta det som väger:** `gitpoulpi rewrite analyze [--top N]` (och MCP-verktyget
  `rewrite_analyze`) är storleksanalysen, som `filter-repo --analyze`: de största
  filerna med sökväg, storlek, första och sista commit och om de fortfarande
  finns i HEAD, de största mapparna, filerna som tagits bort från HEAD men som
  historiken fortfarande lagrar, samt totalerna. Den läser bara storlekar och
  sökvägar, aldrig filinnehåll, och ändrar ingenting.
- **Fliken Storlek** gör samma sak i omskrivningsvyn (`F7`, eller *Analysera
  historikens storlek* i paletten, öppnar den och kör): **Analysera storlekar**
  (`s`) är ett jobb (*Analyserar*, **Avbryt** eller `Esc` stoppar det). **Visa**
  väljer listan (`v`): *Filer* (de största unika filversionerna, var och en
  märkt *i HEAD*, *gammal version* av en fil som fortfarande finns kvar, eller
  *borttagen*), *Mappar* (allt som någonsin lagrats under var och en) eller
  *Borttagna* (sökvägar som är borta från HEAD men som historiken behåller).
  **Sortera** (`o`) sker efter storlek, efter sökväg eller borttagna först.
  Markera rader (`Space`, `Alt+A` / `Alt+U` för alla / ingen): fliken säger hur
  mycket de rymmer, ärligt som ”upp till cirka”, eftersom utrymmet kommer
  tillbaka först efter omskrivningen, en rensning av de gamla objekten och en
  `gc`. **Ta bort ur historiken** (`r`) gör de markerade sökvägarna till en
  sökvägsregel (specialtecken skyddas, en sökväg som redan finns i en regel
  läggs inte till två gånger); **Ta bort varje fil över ...** (`b`, listan
  Filer) skriver en storleksregel strax under den minsta markerade filen.
  Förhandsgranska och tillämpa dem som vilken annan regel som helst. Listorna
  visar de 200 största raderna; kommandoraden säger hur många fler det finns och
  `--top` listar dem. Rader blir inaktuella efter en omskrivning eller en ändring
  av omfånget: analysera igen.

### Signaturer

- **Signaturer:** en omskriven commit kan inte behålla sin signatur (den signerade
  de gamla byten), så när signerade commits skrivs om får du frågan varje gång:
  *Ta bort signaturerna* (antalet visas först), *Signera om bara de commits som
  var signerade* eller *Signera om varje omskriven commit*. Det finns inget
  standardval. Commits som förblir som de är behåller sina. Omsigneringen
  använder din egen uppsättning, på samma sätt som en signerad commit gör:
  `gpg.format` (`openpgp`, `ssh` eller `x509`), `user.signingkey` och
  `gpg.program` / `gpg.ssh.program` / `gpg.x509.program`. Frågans fjärde rad,
  **Kontrollera signeringen** (`4`), signerar en dummy-nyttolast en gång och
  verifierar den: den berättar hur lång tid en signatur tar och uppskattar
  körningen för antalet commits frågan gäller, eller varför det inte fungerar
  (nyckel saknas, agenten är låst, programmet är inte installerat, med hur man
  installerar det). Förhandsgranskningen gör samma test innan den läser
  historiken och vägrar med den orsaken, så en trasig uppsättning lämnar aldrig
  en halvsignerad omskrivning. Commits signeras **under förhandsgranskningen**
  (deras id innehåller signaturen), så en förhandsgranskning som signerar om tar
  signeringstiden, Sammanfattningen säger *Omsignerade commits: n, på 12,5 s*, och
  en lösenfrasfråga kan öppnas då: lås upp din agent först (Kontrollera
  signeringen gör exakt det). **Avbryt** stannar mellan två signaturer och lämnar
  ingenting kvar. Bara commitsignaturer skapas på nytt; en signerad tag förlorar
  fortfarande sin signatur.
- **Regeln *Ta bort signaturer*** (Lägg till-menyn, gruppen *Signaturer*;
  *Omskrivning: Ta bort signaturer...* i paletten; `--remove-signatures`) tar bort
  signaturen på varje commit i omfånget, även där inget annat ändras, och frågar
  inget om omsignering. Från CLI: `rewrite check-signing` och
  `--signatures strip|resign-signed|resign-all`.

### Ångra en omskrivning

**Ångra** (resultatpanelens knapp, verktygsfältets *Ångra* med `Ctrl+Z` i Log,
eller `gitpoulpi undo`) sätter tillbaka varje flyttad gren, tag och notering, och
de utcheckade filerna med dem, från de gamla commits som omskrivningen fäster.
Den frågar först när en omskriven gren redan finns på sin upstream (att ångra
lokalt ångrar inte en push) och när du har oincheckade ändringar: de behålls, och
en fil du ändrat som den gamla historiken skulle skriva över stoppar i stället
ångrandet (filerna flyttas på samma sätt som `reset --keep` flyttar dem). Att
ångra går inte längre för en omskrivning när du [rensar](#radera-den-gamla-historiken-rensning)
dess gamla historik; historiklistan markerar den då som *rensad*.

## Flytta filer till eller från Git LFS

Två regler i gruppen *LFS* i Lägg till-menyn skriver om historiken så att stora
filer ligger i [Git LFS](lfs.md) från början (eller lämnar det). De kräver
programmet `git-lfs`: utan det är reglerna nedtonade med installationstipset och
en förhandsgranskning vägras. De är vår egen motor, inte `git lfs migrate`, så du
får förhandsgranskningen, omfånget, ångra och push-skyddet som vilken annan regel
som helst.

- **Migrera filer till LFS** (*Omskrivning: Migrera filer till LFS...*, eller
  `--lfs-import PATTERN` med `--lfs-min-size BYTES`). Varje fil som matchar
  mönstren (`*.psd`, `assets/**`; ett mönster utan snedstreck matchar på alla
  djup) och är minst så stor som *Bara filer på minst* (tomt: varje träff) blir en
  LFS-pekare i varje commit som innehåller den. Dess innehåll läggs i det lokala
  LFS-lagret (`.git/lfs/objects`, en gång per distinkt innehåll) när du tillämpar,
  och rotens `.gitattributes` får `<pattern> filter=lfs diff=lfs merge=lfs -text`
  i varje omskriven commit som innehåller en pekare (skapas om den saknas, övriga
  rader rörs inte, dubbleras aldrig). En fil som redan är en pekare lämnas orörd.
  Sammanfattningen säger *3 filer (12 MiB) flyttas till LFS. LFS-objekt att skriva:
  3.* och fliken Commits listar *flyttar big.psd till LFS* per commit.
- **Ta tillbaka filer från LFS** (*Omskrivning: Ta tillbaka filer från LFS...*,
  eller `--lfs-export PATTERN`). Pekare som matchar blir de riktiga filerna igen,
  och raderna i `.gitattributes` för de mönstren tas bort. Objekten måste finnas i
  det lokala LFS-lagret: när några saknas vägras förhandsgranskningen med listan
  och en knapp **Ladda ned LFS-objekt** som hämtar dem från `origin`;
  förhandsgranska igen efteråt. En import följd av en export av samma mönster ger
  exakt samma commits tillbaka.

Efter Tillämpa är de utcheckade filerna det riktiga innehållet (inte pekare) och
**Ångra** sätter tillbaka den gamla historiken. Resultatpanelen har **Skicka
LFS-objekt** (`l`): samma sak som `LFS: Push All Objects`, så att den nya
historiken kan pushas utan att en senare push stannar på saknade objekt (en vanlig
push av de omskrivna grenarna laddar också upp dem via pre-push-hooken som
Tillämpa installerar). Noteringen till medarbetare säger att de behöver git-lfs
och `git lfs pull`.

Bra att veta: Tillämpa installerar repositoryts LFS-filter (`git lfs install
--local`) när de saknas, eftersom git behöver dem för att jämföra de riktiga
filerna med pekarna. Bara rotens `.gitattributes` skrivs, aldrig en nästlad. En
fil *under* minimistorleken som matchar ett mönster förblir en vanlig fil, men git
vill göra om den till en pekare vid nästa add, så `git status` visar den som
ändrad tills du committar den. Importen strömmar stora filer, exporten håller en
fil i minnet åt gången.
## Skicka resultatet

Att tillämpa en omskrivning pushar aldrig. Omskrivna grenar och taggar som fanns
på en remote kräver en force push, och pushen forcerar aldrig blint: varje ref
skickas med en lease på det id som remoten hade när du förhandsgranskade (inte
det din spårande gren säger nu), så en kollegas push däremellan avvisas för just
den refen i stället för att skrivas över, och refs skickas i en allt-eller-inget-push
(`--atomic`) där servern tillåter det. Från kommandoraden:

```
gitpoulpi rewrite push --dry-run        # frågar remoten, skickar ingenting
gitpoulpi rewrite push --yes            # skickar
gitpoulpi rewrite note                  # vad de andra måste köra
gitpoulpi rewrite maps --out ./maps     # commit-map och ref-map
```

Skyddade grenar (inställningen `rewrite.protected_branches`, standard `main`,
`master`, `develop`, `release/*`) lämnas ifred vid push, både vid omskrivningen
och vid pushen, om du inte anger dem (`--include-protected main`) eller tar med
alla. Värdens egen grenskydd avgör fortfarande om den tar emot en force push;
dess avslag visas per ref.

I omskrivningsvyn gör resultatpanelen efter en Tillämpa allt detta:

- **Skicka de omskrivna refs till remoten** listar de refs som omskrivningen
  flyttade och som en remote har, var och en med en ruta (`Space`). Skyddade
  grenar börjar omarkerade, med *skyddad* bredvid: markera dem en och en, eller
  använd *Markera alla* / *Avmarkera alla* (`Alt+A` / `Alt+U`). **Push omskrivna
  grenar** (`p`) skickar de markerade; svaret är en rad per ref (*pushad*,
  *redan uppdaterad*, *avvisad: remoten har ändrats sedan förhandsgranskningen*,
  *avvisad: skyddad av värden*, *hoppades över: skyddad gren*, *inte pushad*). En
  avvisad rad har **Fetch och granska** (`f`): omskrivningsvyn stängs och
  remotes hämtas, så att Log visar vad de andra har pushat. Ingenting skrivs
  över: leasen skyddar deras arbete, och du förhandsgranskar igen innan du
  försöker på nytt.
- **Kopiera noteringen** (`c`) lägger noteringen till medarbetare i
  urklipp, på gränssnittets språk runt de oöversatta git-kommandona.
- **Öppna commit-kartan** (`m`) visar var commit-kartan och ref-kartan finns;
  ett klick på en sökväg (eller `1` / `2`) kopierar den.

En push går inte att ångra: om du gör `undo` på omskrivningen efteråt ändras
bara ditt eget repository, och ångra-åtgärden säger det innan den frågar. Skicka
noteringen till alla som har den gamla historiken: den listar gammal och ny topp
för varje gren och tag samt kommandona (`git fetch`, sedan
`git reset --hard origin/<branch>` eller `git rebase --onto`). De två kartorna
sparas i 24 timmar tillsammans med körningen; kopiera ut dem om GitLabs
repositoryrensning eller ett skript behöver dem.

## När värden skyddar en gren

Din värd kan skydda en gren på egen hand (GitHubs grenskydd och rulesets,
GitLabs skyddade grenar, Azure DevOps grenpolicyer, Bitbucket-begränsningar för
grenar) och vägra den force push som en omskrivning behöver. När repositoryt är
**länkat** till sin värd (avsnittet med pull requests i sidopanelen, se
[Hosting-integration](hosting-integration.md)) frågar omskrivningsvyn värden vilka
grenar den skyddar, med din egen token:

- De grenarna **lämnas utanför omskrivningen som standard**, precis som de
  skyddade grenarna i `rewrite.protected_branches` och arbetsflödet: fliken Refs
  listar dem med en ruta var och knapparna *Markera alla* / *Avmarkera alla*, och
  säger varför: *skyddad på GitHub*.
- Om token inte kan läsa skyddet (den saknar behörigheten, eller du är inte
  administratör) säger omskrivningsvyn *Värdens skydd är okänt* och betraktar
  ingenting som skyddat på värdens ord. Omskrivningen blockeras aldrig av det. Ett
  repository som inte är länkat tillfrågas aldrig.

När en skyddad gren är i vägen (förhandsgranskningen vägrades eftersom varje gren
är skyddad, eller värden avvisade pushen: *avvisad: skyddad av värden*) förklarar
knappen **Så kommer du förbi** (`Alt+H`) de tre vägarna ut, med exakt den
inställningssida som hör till din leverantör och ditt repository (markera en länk
och kopiera den med `c`, eller öppna den med `o`). GitPoulpi gör aldrig någon av
dem åt dig:

1. **Häv skyddet tillfälligt** i värdens inställningar, gör en push och sätt
   sedan tillbaka det.
2. **Gör en push av omskrivningen under ett nytt grennamn**, gör sedan den grenen
   till standardgren (eller döp om) och ta bort den gamla när ingen använder den.
3. **Som sista utväg, ta bort och skapa om remote-repositoryt** och pusha allt.
   Det här förlorar pull / merge requests och deras granskningar, ärendena,
   forkarnas koppling till den gamla historiken, stjärnor och bevakare,
   CI-inställningar, hemligheter och variabler samt webhooks och deploy-nycklar.

Från CLI kommer samma text tillbaka som `protected_by_host` i JSON från
`rewrite preview` / `rewrite run` (när värden skyddar en gren i omfånget), från en
vägrad `rewrite push` och från `rewrite note`: `provider`, `branches`, `ways`
(var och en med sina `steps` och `links`), `recreate_loses` och hela förklaringen
som `text`.
## Radera den gamla historiken (rensning)

Efter en omskrivning finns de gamla commits kvar i ditt repository: ångra av
omskrivningen behåller dem (det är det som gör den ångringsbar), och det gör
reflogs också. Om du skrev om för att bli av med en hemlighet ligger hemligheten
fortfarande i mappen `.git`. Rensningen raderar den gamla historiken för gott:

```
gitpoulpi rewrite purge                 # frågar: listar vad som raderas och går förlorat (slutkod 3)
gitpoulpi rewrite purge --yes           # gör det
```

Den tar bort omskrivningens ångra-fästen, låter de reflog-poster som nämner de
gamla commits gå ut, kör `git gc --prune=now` och kontrollerar sedan att varje
gammal commit och filversion verkligen är borta. Bekräftelsen säger i förväg vad
du förlorar: **Ångra går inte längre** för omskrivningen (och för tidigare
åtgärder vars ångra också behövde de gamla commits); de finns kvar i
historiklistan, märkta som rensade. En rensning går inte att stoppa när den väl
har börjat och går inte att ångra. Den rör ingen remote.

Allt som fortfarande håller den gamla historiken stoppar rensningen, och då
ändras ingenting. Listan namnger varje sådant med vad man ska göra: en gren eller
tag som inte ingick i omskrivningen, en stash på den gamla historiken (ta bort den,
eller behåll den och acceptera att den gamla historiken blir kvar), en annan
worktree vars HEAD ligger på den, och de remote-spårande grenar som fortfarande
pekar på den gamla historiken (pusha de omskrivna grenarna och gör sedan fetch).
Din stash-lista och andra grenars reflogs rörs aldrig.

En hemlighet som har pushats är läckt: **rotera den först**. Efter rensningen
listar svaret vad värden fortfarande behöver, per remote: GitHub håller gamla
commits nåbara i cachade vyer och pull request-refs (kontakta GitHub Support,
eller ta bort och skapa om repositoryt), GitLab har en repositoryrensning som
tar `commit-map` (`gitpoulpi rewrite maps --out DIR`) plus underhåll, Azure
DevOps och Bitbucket kräver ett supportärende eller ett nytt repository, och
alla med en klon eller en fork har fortfarande den gamla historiken.

### Rensningsskärmen

I omskrivningsvyn har resultatpanelen efter en omskrivning en knapp **Rensa
gammal historik...** (tangent `e`), och paletten har **Omskrivning: Rensa
gammal historik...** (den arbetar på den nyaste omskrivningen som inte är
rensad ännu, också efter att du stängt omskrivningsvyn). Skärmen listar vad
rensningen raderar och vad du förlorar (åtgärderna vars ångra försvinner med
den; de märkta *förlorar också Ångra* är inga omskrivningar men behövde de
gamla commits) och sedan vad som fortfarande håller den gamla historiken, en
rad var:

- en remote-spårande gren: **Fetch** uppdaterar dem från remotes;
- en stash: **Drop** frågar först, eftersom en stash är arbete du aldrig
  committat;
- en gren, tag, worktree eller annan ref: **Kopiera namn**, och åtgärda den
  sedan där den finns (skriv om eller ta bort grenen, checka ut en omskriven
  gren i worktreen).

Kontrollen körs igen efter varje åtgärd, och **Kontrollera igen** (`r`) kör den
på begäran. **Rensa** (`p`) går bara att använda när inget återstår. Den ställer
en sista fråga, numrerad och utan standardval (`1` rensar för gott, `2` lämnar
allt som det var) och kör sedan; skärmen visar vilket steg den är i och säger att
den inte kan stoppas. Utfallet säger att den gamla historiken är borta från den
här klonen, eller hur många gamla objekt som finns kvar. Under det står stegen
att ta på varje remote (en länk går att kopiera). Krysset, `Esc` och ett klick
utanför stänger skärmen, utom medan rensningen körs.

Via MCP finns rensningsverktyget inte om du inte sätter `allow_history_purge =
true` i tabellen `[mcp]` i `config.toml`.

## Skriptregeln

*Lägg till regel > Skript* (eller **Omskrivning: Kör ett skript...** i paletten)
lägger till en Lua-skriptregel och öppnar dess redigerare över omskrivningsvyn;
`Enter` på regelns rad *Skript* öppnar den igen. Redigeraren utgår från en liten
mall.

- **Språk** (en rad i regelns ruta): *Lua (inbyggt)*, *Skalskript*, *Python*
  eller *Lua (installerat)*. Allt utom det inbyggda Lua är ett
  [externt skript](#externa-skript-shell-python-lua): utan sandlåda, och ett
  språk som är avstängt i inställningarna eller vars tolk saknas är nedtonat i
  listan med orsaken bredvid (att välja det säger vad man ska göra). Om du byter
  språk på ett skript du ännu inte har skrivit byts mallen ut.
- **Skript** är en textredigerare med radnummer. Det du skriver är regeln
  direkt; inget behöver bekräftas.
- Under den säger raden **kontroll** `OK: rewrite_commit hittades`, eller listar
  problemen med deras rad (`script:3: ...`; numret är rött i marginalen). Den
  körs en stund efter att du slutat skriva, aldrig medan en tangent hålls nere.
- **Parametrar** har en rad per `--! param`-rad i skriptets huvud, med sin
  etikett bredvid fältet: text, ett tal eller en ruta för sant/falskt. Ett värde
  du rensar går tillbaka till huvudets standard; en textparameter utan standard
  måste fyllas i före Förhandsgranska.
- **Fil** (en sökväg inne i repositoryt, `.gitpoulpi/rewrite/script.lua` till
  att börja med) med **Läs in från fil...** (ersätter texten) och **Spara till
  fil...**.
- **Testa på urvalet** kör skriptet på de commits du markerat i Log, eller på de
  20 nyaste när ingen är markerad, och skriver ingenting. Tabellen listar varje
  commit (kort id), varje fält den skulle ändra som `gammalt -> nytt` och
  skriptets `log(...)`-rader. **Avbryt** slutar vänta. Om skriptet misslyckas
  visas commiten och orsaken.
- Krysset, ett klick utanför redigeraren och `Esc` stänger den; `Tab` och
  `Shift+Tab` flyttar mellan redigeraren, parametrarna, filen, knapparna och
  resultaten; `F5` testar.

Förhandsgranska vägrar ett skript som inte kompilerar: Förhandsgranska-rutan
säger *ett skript misslyckades* med raden (eller, vid ett misslyckande på en
commit, dess korta id och orsaken). I en version utan skript finns sorten kvar i
Lägg till-menyn, nedtonad.

## Skriv ett skript med AI

Med AI påslaget ([AI-stöd](ai.md)) har skriptredigeraren en knapp
`✦ Skriv med AI...`, och paletten har `✦ Omskrivning: Skriv ett skript med AI...`,
som öppnar omskrivningsvyn på en ny skriptregel med dialogen öppen. Ingenting
skickas förrän du trycker på **Skicka**: dialogen visar medan du skriver exakt
den text som går iväg.

- **Vad ska skriptet göra?** Med dina egna ord: *flytta varje commit som gjorts
  på natten till nästa morgon*, *byt ut min gamla e-postadress mot den nya, men
  bara före 2024*.
- **Skicka ett urval på N commits** (av): när rutan är markerad läggs författare,
  datum och ämne för de commits som ”Testa på urvalet” skulle använda (ditt val i
  Log, annars de 20 nyaste) till, så att modellen ser hur dina data ser ut.
  Aldrig filinnehåll eller commitmeddelandenas brödtext. Med rutan av lämnar inga
  commitdata datorn.
- **Det som skickas** listar instruktionerna (det fasta skriptkontraktet och sedan
  dina AI-instruktioner) och din begäran, och vilken profil som tar emot dem.
  `Tab` flyttar dit, pilarna rullar.
- **Skicka** (`Tab` dit, `Enter`). Medan modellen arbetar väntar dialogen, och
  `Esc` avbryter begäran. Om den misslyckas står dialogen kvar med orsaken så att
  du kan försöka igen.

Svaret kontrolleras av samma daemonkontroll som ett skript du skrivit själv; ett
kompileringsfel skickas tillbaka till modellen en gång. Skriptet **ersätter sedan
texten i redigeraren** (dialogen säger det när redigeraren innehåller något annat
än startmallen) och redigerarens rubrik lyder *✦ skrivet av AI: läs det före
Förhandsgranska* tills du ändrar texten. Det testas, förhandsgranskas eller
tillämpas aldrig åt dig: läs det, tryck på **Testa på urvalet** och förhandsgranska
sedan som med vilket skript som helst. Skriptet körs i samma sandlåda som ett du
skrivit själv (inga filer, ingen klocka, inget nätverk).

Från kommandoraden: `gitpoulpi ai rewrite-script "<instruktion>" [--sample N]`
skriver ut skriptet och resultatet av `rewrite script check` på det som JSON och
kör ingenting (slutkod 2 om det fortfarande inte kompilerar). Profilen *Skript för
historikomskrivning* under Inställningar > AI används.

## Skapa regler med AI

Med AI påslaget ([AI-stöd](ai.md)) har omskrivningsvyn en knapp
`✦ Skapa regler med AI...` bredvid Spara och Läs in, tangenten `Alt+I` (*Skapa
omskrivningsregler med AI* under Inställningar > Tangentbord & mus, kontexten
*Omskrivningsvyn*), och paletten har `✦ Omskrivning: Skapa regler med AI...`. Med
AI avslaget, eller i en version utan AI, finns knappen och palettraden inte. Du
beskriver med dina egna ord hur historiken ska skrivas om (*ersätt min gamla
e-postadress a@b.org med c@d.org*, *ta bort secrets.txt ur hela historiken*,
*flytta mina datum två timmar*) och modellen föreslår regler och inställningar,
som **fyller omskrivningsvyn**. Ingenting skickas förrän du trycker på
**Skicka**: dialogen visar medan du skriver exakt den text som går iväg.

- **Hur ska historiken skrivas om?** Ett fält med flera rader: `Enter` ger en ny
  rad, `Ctrl+Enter` eller `Ctrl+S` skickar, `Esc` och krysset avbryter.
- **Skicka omfånget, namnen på grenar och tags och historikens författare** (av):
  när rutan är markerad läggs omfånget på omfångsraden till, liksom namnen på
  grenar och taggar och de olika författarna (namn och e-post) till de nyaste
  commits som Log visar (högst 20 commits), så att modellen kan ange din
  e-postadress exakt (kommandoradens `ai rewrite-rules --context` läser i stället
  de 200 nyaste commits). Aldrig filinnehåll eller commitmeddelanden. Med rutan av
  lämnar bara din beskrivning och de fasta instruktionerna datorn.
- **Ersätt de nuvarande reglerna** (bara när omskrivningsvyn redan innehåller
  regler; av): som standard läggs de nya reglerna till efter dem du har; med
  rutan markerad ersätter de dem (omfånget och inställningarna som svaret anger
  tillämpas i båda fallen).
- **Det som skickas** listar instruktionerna (regelvokabulären och sedan dina
  AI-instruktioner), din begäran och vilken profil som tar emot dem. `Tab`
  flyttar dit, pilarna rullar.

Modellen får använda alla regelsorter på den här sidan utom **Skript** (kod:
*Skriv ett skript med AI* visar skriptet för granskning) och formen med hemliga
värden i *Sök och ersätt i filer* (hemliga värden lämnas aldrig till en modell:
använd fliken Hemligheter). Den kan sätta omfånget till den aktuella grenen, alla
lokala grenar och taggar, eller namngivna grenar och taggar som finns, och
alternativen *Taggar*, *committer*, *tomma commits* och *förkortade id i
meddelanden*. Det som omskrivningsvyn inte kan uttrycka (en datumgräns som *sedan
mars*, ett intervall av commits) gissas inte: modellen säger det i en kommentar.
Vad som händer med signaturerna avgör aldrig modellen: du får frågan vid
Tillämpa.

**AI:n förbereder bara regler och inställningar.** Den förhandsgranskar,
tillämpar eller kör aldrig något: svaret kopieras in i omskrivningsvyn som ett
recept du har läst in, förhandsgranskningen på skärmen blir inaktuell, och
Förhandsgranska och Tillämpa (med sina frågor och bekräftelser) är dina, som
alltid. Varje regel tolkas och kontrolleras för sig av samma kontroll som
redigeraren och motorn använder; en regel som inte klarar den utelämnas. När
något utelämnades, eller modellen gjorde en kommentar, visas en lista när
reglerna är på plats (*Utelämnade (de giltiga reglerna behölls)*, var och en med
sin position, sitt namn och orsaken), och `Enter` eller `Esc` stänger den. Läs
reglerna, redigera dem och förhandsgranska sedan.

Från kommandoraden: `gitpoulpi ai rewrite-rules "<beskrivning>" [--context]`
skriver ut förslaget som JSON (`recipe`, som `rewrite preview --recipe-json`
läser, `problems`, `notes` och `applied: false`) och kör ingenting (slutkod 2 när
något utelämnades). MCP-verktyget är `ai_rewrite_rules`. Profilen *Regler för
historikomskrivning* under Inställningar > AI används.

## Lua-skript

En skriptregel kör din funktion på varje commit. Det är vanlig Lua 5.4 i en
sandlåda: inga filer, ingen klocka, inga processer, inga moduler, ingen `pcall`;
samma skript på samma historik ger alltid samma resultat, och ett anrop som
överskrider 10 miljoner instruktioner, 64 MiB eller 5 sekunder avbryts med ett
fel som anger commiten.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` innehåller `id`, `parents`, `tree`, `author` och `committer` (`name`,
`email`, `time`, `offset_minutes`), `message` och `signed`. `ctx` innehåller
`index`, `total`, `params`, `state` (returnera `state = {...}` för att föra data
vidare till nästa commit) och `window.before` / `window.after` (ange
`--! window: N`). Returnera bara det som ändras: `author`, `committer`,
`message`. Hjälpfunktioner: `regex_replace`, `regex_match`, `stable_hash`, `rand`,
`weekday`, `hour`, `log`, och för meddelanden `trailers(message)` (en lista med
`{key, value}`), `set_trailer(message, key, value)` (ersätter trailers med den
nyckeln; ett fjärde argument `"add"` eller `"skip"` ändrar det),
`remove_trailer(message, key)` och `at("2024-03-05 18:30", offset_minutes)` (ett
datum i en förskjutning som Unix-tid).

Från kommandoraden: `gitpoulpi rewrite script check my.lua` kompilerar det,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` visar
vad det skulle ändra utan att skriva något, och
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b`
planerar hela omskrivningen.

## Externa skript (shell, Python, Lua)

Samma kontrakt som ett Lua-skript, men körs av ett program på din dator:
*Skalskript* (bash eller sh; PowerShell i Windows, eller vilken `.ps1` som
helst), *Python* och *Lua (installerat)* (en egen `lua`, inte den inbyggda; den
har inget JSON-bibliotek, så mallen läser raden med ett mönster). **De körs utan
sandlåda: ett externt skript körs med dina rättigheter** och kan läsa eller
ändra allt du kan. Regelns ruta och redigerarens rubrik säger det.

Så pratar det: GitPoulpi startar skriptet en gång per körning och skickar det en
JSON-begäran per rad på dess standard in (en commits fält, `params`, den `state`
som förra svaret gav); det svarar med en JSON-rad på standard ut, med samma
fält som ett Lua-skript returnerar (`author`, `committer`, `message`, `state`,
`log`), eller `{"error": "text"}` för att ge upp. Skriv diagnostik på standard
fel: den visas som commitens `log`-rader. Huvudraderna börjar med `#!` för shell
och Python (`#! name: ...`, `#! param old: text "Old"`; en shebang-rad förblir
en shebang) och med `--!` för Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Vad det får.** En tom miljö utom `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` och `LANG`; en ny temporär mapp som arbetsmapp
(med en egen kopia av skriptet); inte repositoryts sökväg. **Tidsgränser:**
`rewrite.script_timeout_ms` (5000) för varje svar och
`rewrite.script_total_timeout_s` (3600) för hela passet. När en gräns
överskrids, skriptet kraschar, skriver ut något som inte är JSON eller jobbet
avbryts dödar GitPoulpi skriptet och allt det startade, och omskrivningen
stoppas med commiten och orsaken; ingenting har flyttats.

**Vilka språk som finns.** `rewrite.external_scripts` i `config.toml` listar
dem: `["shell"]` som standard; lägg själv till `"python"` och `"lua"` i
`config.toml` (`[rewrite]`, `external_scripts = ["shell", "python"]`; en lista
går inte att sätta med `gitpoulpi config set`). Ett språk som inte finns i
listan är nedtonat i omskrivningsvyn och vägras av CLI och MCP
(`scripts_disabled`); ett listat språk vars tolk inte hittas är nedtonat med
installationstipset (`tool_missing`).

**Förtroende.** Även när ett språk är påslaget körs ett skript först efter att du
litat på dess exakta text. Ett skript du skriver i omskrivningsvyn är ditt och
körs direkt; ett som följer med ett delat recept eller en fil i repositoryt är
det inte: Förhandsgranska och *Testa på urvalet* stannar och frågar *Lita på det
här skriptet?* med två numrerade svar och inget förvalt (`1` lita på det och
fortsätt, `2` gör inte det, `Esc` lämnar det). Förtroendet kommer ihåg textens
SHA-256 för det här repositoryt, så ett ändrat skript frågar igen, och ett
klonat repository kan inte köra kod hos dig på egen hand. Från kommandoraden:
läs filen och kör sedan `gitpoulpi rewrite script trust FIL` (eller `--trust` på
`preview`, `run`, `script check` och `script test`, som litar på skriptet och
fortsätter). En agent via MCP kan inte lita på något: den kan bara köra ett
skript som du litat på tidigare.

**Kontroll.** Kontrollraden under redigeraren säger *Ännu inte betrott* för ett
obetrott skript (inget startades); för ett betrott startar GitPoulpi det en gång
med en påhittad commit (`"dry_run": true`) och kontrollerar att svaret följer
kontraktet. `gitpoulpi rewrite script check FIL` gör detsamma och rapporterar
`trust`, `hash` och `interpreter`.

## Från kommandoraden

`gitpoulpi rewrite` (och MCP-verktygen med samma namn) kör samma motor, för
skript och agenter. Ett **recept** är en TOML- eller JSON-fil, den som
omskrivningsvyn sparar (`--recipe FILE`, eller `-` för standard in), och flaggor
lägger till regler efter receptets egna.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # planera, ändrar ingenting
gitpoulpi rewrite apply --run ID --digest D           # kör planen du granskat
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # sätter tillbaka allt
```

- Regler som flaggor: `--set-name` / `--set-email` med `--match-email`
  (identitet), `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`,
  `--trailer-add`, `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. Omfånget är aktuell gren, `--all-local` eller `--ref R`
  (kan upprepas), från `--since REV`.
- **Frågor utan standardsvar** är flaggor: `--signatures
  strip|resign-signed|resign-all` när signerade commits ändras, och
  `--extract-mode` för en extraktion. Utan dem blir svaret ett avslag eller ett
  `needs_choice` (slutkod 3), aldrig en gissning.
- **Slutkoder:** 0 klart; 1 ett fel (oläsbart recept, felaktiga flaggor); 2
  avvisat, med orsaken (`stale`, `signature_choice_needed`,
  `dirty_paths_touched`, `empty_scope`, ...); 3 en fråga att besvara, eller
  pushade refs som kräver `--yes`.
- `preview` behåller planen i repositoryt, så `apply` fungerar från ett senare
  kommando, efter en omstart av daemonen eller från en annan frontend; en plan som
  inte längre stämmer med sin digest är `stale`.

[CLI-referensen](cli-reference.md) har varje flagga, och
[MCP och agenter](mcp-and-agents.md) säger vilka verktyg en agent får
(rensningsverktyget finns inte om du inte slår på det, och en agent kan inte lita
på ett skript).

## Förslag från AI-agenter

En agent kan **föreslå** en omskrivning, aldrig köra en. `gitpoulpi ui propose
rewrite --recipe FILE|-` (MCP-verktyget `ui_propose_rewrite`, avstängt tills
`[mcp] allow_ui_propose = true`) öppnar omskrivningsvyn i ditt pågående fönster
med receptets regler redan i listan och en lila banderoll, *✦ Föreslagen av
<agent>: en omskrivning av historiken (3 regler) · Ctrl+Enter godtar · Esc
avvisar*.

- **Ingenting körs.** Ingen förhandsgranskning, ingen tillämpning, ingen push,
  ingen rensning: förslaget fyller bara i reglerna. Du läser dem, redigerar dem om
  du vill, **förhandsgranskar** och tillämpar med din egen tangent. Att stänga
  omskrivningsvyn avvisar förslaget.
- **Vad agenten inte får lägga i det:** ett skript (det skulle köras vid
  förhandsgranskningen), fler än 50 regler, hemliga värden (de tas bort ur
  receptet innan det skickas). Ett sådant förslag avvisas med orsaken.
- Agenten kan läsa utfallet i `ui state` (`accepted` när du tillämpade reglerna
  som de föreslogs, `accepted_edited` när du ändrade dem först, `rejected` när du
  stängde det). Ett förslag i taget; det avvisas medan en annan av dina dialoger är
  öppen.
## Verktyg

Det mesta av en historikomskrivning kräver inget utöver Git: att byta författare
och committers, tillämpa en `.mailmap`, anonymisera, förskjuta datum eller
begränsa dem till vissa tider, skriva om meddelanden och trailers, ersätta
innehåll, ta bort sökvägar eller stora blobbar och flytta sökvägar sker allt
inuti GitPoulpi.

**Lua-skript är inbyggda.** Tolken ingår i GitPoulpi, så ett Lua-skript
behöver varken något verktyg eller någon installation. Shell-, Python- och
installerade Lua-skript behöver sin tolk (se tabellen nedan) och beskrivs under
[Externa skript](#externa-skript-shell-python-lua).

Några funktioner använder ett externt program. GitPoulpi letar efter det
(programmet hittas i din `PATH`, i Windows även via `PATHEXT` och Git för
Windows egen `usr/bin`, och måste svara på `--version` inom 2 sekunder); svaret
cachas i en minut och ”Kontrollera igen” frågar på nytt. En regel som kräver ett
program som saknas blir kvar i listan, nedtonad, med orsaken och hur man
installerar det, och Förhandsgranska vägrar innan något körs.

| Funktion | Verktyg | Anmärkningar |
|---|---|---|
| Signera om commits | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) eller `gpgsm` (`gpg.format = x509`) | SSH- och X.509-signering kräver också `user.signingkey`. `gpg.program`, `gpg.ssh.program` och `gpg.x509.program` respekteras. |
| LFS-migrering (de två LFS-reglerna) | `git-lfs` 3.0 eller senare | Samma kontroll som i LFS-rutan. |
| Python-skript | `python` 3.8 eller senare (`py -3` i Windows) | Avstängt som standard: lägg till `"python"` i `rewrite.external_scripts`. |
| Lua-skript (installerat) | `lua5.4`, `lua54`, `lua` eller `luajit` | Avstängt som standard: lägg till `"lua"` i `rewrite.external_scripts`. |
| Skalskript | `bash`, `pwsh` (PowerShell) eller `sh` | På som standard (`"shell"`). Ett PowerShell-skript (en `.ps1`-fil, en `pwsh`-shebang eller vilket skalskript som helst i Windows) kräver `pwsh` eller `powershell`; ett POSIX-skript kräver `bash` eller `sh` (i Windows ger Git för Windows dem). |

Installationstips, Windows först:

- Windows: Git för Windows innehåller `gpg`, `ssh-keygen`, `bash`, `sh` och
  Git LFS. Annars `winget install GnuPG.GnuPG`, Inställningar > Valfria
  funktioner > OpenSSH-klient, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: distributionens paket (`gnupg`, `openssh`, `git-lfs`, `python3`,
  `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` och
  `sh` är inbyggda.
