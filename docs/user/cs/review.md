[English](../review.md) · **Čeština** · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Kontrola pull nebo merge requestu

Zkontrolujte pull request (GitHub, Gitea, Forgejo), merge request (GitLab) nebo změnu Gerrit (viz [Gerrit](#gerrit))
**aniž byste opustili okno repozitáře**. Pohled kontroly ukazuje popis, konverzaci, změněné
soubory s vlákny komentářů pod jejich řádky a commity a je to místo, kde kontrolu píšete: komentáře,
schválení a žádosti o změny. Potřebuje profil jednoho z těchto hostitelů (viz
[Hostingové služby a issue trackery](hosting-integration.md)) a repozitář s ním propojený; jiní poskytovatelé
kontroly nenabízejí. Omezení hostitele se respektují: GitLab nemá rozhodnutí *Požádat o změny* a bere jeden řádek na komentář.
Tato stránka všem říká *žádost*.

![Pohled kontroly, stránka Přehled: záhlaví žádosti s tlačítky metadat, popis a konverzace s odeslaným schválením](../images/review-overview.png)

![Stránka Soubory: změněné soubory s políčky zobrazeno, diff s vláknem komentářů pod řádkem a vyřešené vlákno mimo zobrazené řádky](../images/review-files.png)

## Otevření kontroly

Každá cesta vede do stejného pohledu:

- seznam **PULL REQUESTY** v postranním panelu: vyberte řádek a stiskněte **Enter**, nebo na něj klikněte pravým tlačítkem a zvolte
  **Zkontrolovat...**;
- **Radar**: nabídka řádku, **Zkontrolovat...**, nebo klávesa `e` (otevře se repozitář a s ním kontrola;
  u žádosti z repozitáře, který jste nenaklonovali, se zeptá, zda ho naklonovat, nebo žádost zkontrolovat přes API,
  viz [Kontrola bez klonu](#kontrola-bez-klonu));
- **hledání v pracovním prostoru** (`Ctrl+Alt+F`): zvýrazněte pull nebo merge request a stiskněte **`Alt+R`**;
- paleta: **Hosting: Zkontrolovat žádost...** vypíše žádosti k výběru.

Pohled vyplní okno nad repozitářem (je to režim karty repozitáře, ne samostatné okno).
**`Esc`** nebo `✕` v jeho rohu ho opustí. Při otevírání se žádost čte od hostitele a její kód se
na pozadí stahuje gitem; trvá-li to déle než chvíli, objeví se spinner.

## Záhlaví

Vždy viditelné: číslo a název, stav (Otevřená, Koncept, Sloučená, Zavřená), větve
(`main ← feature/moons`), autor, kontroly (`✓ kontroly procházejí`, `✗ kontroly selhávají`, `● kontroly běží`),
zda se sloučí čistě, nebo je v konfliktu, velikost a **`3 čekající komentáře`**, když jste nějaké napsali.

Čtyři tlačítka metadat - **Řešitelé**, **Recenzenti**, **Štítky**, **Milník** - otevírají stejné výběry jako
postranní panel (viz [Řešitelé, recenzenti, štítky a milník](pull-requests-and-issues.md#řešitelé-recenzenti-štítky-a-milník));
záhlaví ukáže výsledek, jakmile ho hostitel přijme. Tlačítka akcí: **Checkout** (obvyklý
checkout větve žádosti, který pohled také zavře), **Otevřít v prohlížeči**, **Obnovit**
(**`F5`**) a **Dokončit kontrolu**.

## Tři stránky

Segmentový přepínač v horním okraji mění stránku: **Přehled**, **Soubory**, **Commity**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, nebo klik).

### Přehled

Přehled je **konverzace žádosti**: stejná časová osa a stejné pole pro komentář, jaké má issue
([Konverzace](conversations.md)): nejdřív popis (*otevřel(a) toto*), pak komentáře, vlákna ke kódu
s ukázkou diffu, odeslané kontroly s rozhodnutím (`schváleno`, `požadovány změny`, `okomentováno`, `zamítnuto`; na GitLabu
jsou rozhodnutími jeho schválení) a události, od nejstarších, v nejnovějším okně s tlačítkem **Načíst starší** nahoře. Pole
komentáře je ukotveno pod časovou osou (**Nový komentář**): **Psát**, **Náhled** a **Vedle sebe**, **Zrušit** a
**Odeslat**. **Komentář napsaný tady jde hostiteli hned** (nic nečeká: čekající jsou komentáře ke kódu, odesílané s
kontrolou), v seznamu se objeví okamžitě a své místo zaujme, když hostitel odpoví; když ho hostitel odmítne, pole se
znovu otevře s vaším textem a důvodem.

Klávesy jsou klávesy pohledu konverzace: **`Up`** / **`Down`** (a `Ctrl+J` / `Ctrl+K`) se pohybují mezi záznamy, **`r`**
odpovídá do vlákna vybraného záznamu, **`Q`** ho cituje v novém komentáři, **`c`** píše nový komentář, **`e`** / **`d`**
upravují a mažou vaše vlastní komentáře (před smazáním následuje otázka), **`l`** nebo `Enter` otevře odkazy záznamu,
**`g`** jeho diagram, **`y`** zkopíruje jeho adresu, **`o`** ho otevře v prohlížeči, **`n`** přejde na další nepřečtený
záznam, **`v`** zobrazí nebo skryje události, **`m`** načte starší záznamy, `Tab` přepíná mezi časovou osou a polem a
**Interní poznámka** (`Alt+I`) je k dispozici na hostitelích, které takové poznámky mají. Tlačítka vybraného záznamu
(**Odpovědět**, **Citovat**, **Upravit**, **Smazat**, **Odkazy**) dělají totéž a paleta má řádky *Konverzace: ...* vedle
řádků *Kontrola: ...*. Na odkaz se před otevřením zeptá, jako všude v kontrole.

Vlákno nad kódem ukazuje svou `cestu:řádek` jako odkaz (podtržený, skutečné tlačítko). Klik na něj, **`G`** na vybraném
vláknu nebo *Konverzace: Otevřít kód vlákna* v paletě přepne na stránku **Soubory**, otevře ten soubor a umístí kurzor
na komentovaný řádek (na starou nebo novou stranu, jak byl napsán) s otevřeným vláknem pod ním (sbalené se rozbalí).
**Esc**, tlačítko **Zpět na přehled** vlevo od karet stránek nebo *Kontrola: Zpět ke komentáři v přehledu* vrátí na
stejný záznam přehledu. Vlákno, jehož řádek už v diffu není (zastaralé), otevře soubor na nejbližším řádku a stavový
řádek to řekne; vlákno na souboru, který žádost už nemění, oznámí, že soubor není v diffu. V pohledu issue nebo Radaru
téže žádosti není žádný kód k otevření: `G` odpoví, že ho má kontrola.

Kde se klávesa konverzace potká s klávesou kontroly, má na této stránce písmeno konverzace: `l` (štítky), `m`
(milník), `Q` (zrušení u Gerritu), `e` / `d`, `r`. Štítky, milník a zrušení zůstávají na tlačítkách záhlaví a
v paletě; `a` (řešitelé), `p` (recenzenti), `f` (**Dokončit kontrolu**), `b` (**Checkout**), `P`, `D`, `U` a
`Alt+1` / `Alt+2` / `Alt+3` si svůj význam drží i tady. Dokud má pole klávesnici, každá klávesa patří poli, kromě
`Alt+1` / `Alt+2` / `Alt+3`; `Esc` vrátí klávesnici časové ose a druhé `Esc` zavře kontrolu.

Komentář napsaný v jiném okně nebo z CLI se tu objeví sám: každý zápis GitPoulpi (včetně tohoto okna) to oznámí relaci
a okna, která ukazují stejnou žádost, ji načtou znovu.

Pole komentáře je Markdown textové pole sdílené s [pohledem konverzace](conversations.md#psaní) (je to i pole
odpovědi do vlákna, komentáře k řádku a shrnutí **Dokončit kontrolu**): `Ctrl+Enter` (také `Ctrl+S`, `Alt+Enter`)
odešle a tam, kde je AI zapnutá, `Alt+E` otevře **Napsat pomocí AI** ([kapitola o AI](ai.md#pomoc-s-psaním-na-hostiteli)).
Náhled ukazuje text tak, jak ho zobrazí hostitel, včetně odkazů (`#7`, `!12`, `ABC-12`, id commitů); neodeslaný text se
uchovává pro každou žádost zvlášť.

### Soubory

Vlevo změněné soubory (písmeno stavu, `+přidáno -odebráno`, počet `●n` otevřených vláken, počet `✎n`
vašich čekajících komentářů) a vpravo diff vybraného souboru. Diff pochází z gitu: head
žádosti se stáhne do `refs/gitpoulpi/pr/<n>` (nikdy větev, nic se nepřepíná) a porovná s
merge base, takže čísla řádků na okraji jsou přesně ta, která komentáře používají.

- **`[ ]` před souborem**: označí ho jako *zobrazený* (klik, nebo **`v`** / `Space` v seznamu). Titulek je počítá.
  Značky zobrazení se pamatují pro každý repozitář a přežijí restart.
- **Vlákna** leží hned pod svým řádkem: první komentář, odpovědi a tlačítka **Odpovědět** a
  **Vyřešit** / **Znovu otevřít**. Vyřešené vlákno se sbalí na jeden řádek; **`z`** (nebo `Enter` na něm, nebo klik na
  `▾`) sbalí a rozbalí libovolné vlákno.
- **Zastaralá vlákna** a vlákna na řádcích, které diff nezobrazuje, se uvádějí za hunky pod
  *Komentáře mimo zobrazené řádky*, s odznakem `zastaralé`, když se kód pod nimi změnil.
- **`c`** okomentuje řádek, na kterém je kurzor (malé `+` na začátku řádku dělá totéž
  myší). **`Shift+Up` / `Shift+Down`** (nebo tažení přes řádky) nejdřív vyberou několik řádků
  a komentář pokryje rozsah. GitLab bere jeden řádek na komentář, a pohled to řekne místo výběru.
- **`r`** odpoví ve vlákně pod kurzorem, **`x`** ho vyřeší nebo znovu otevře. Obojí se hostiteli odešle **hned**.
- Klávesy: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` pohybují; `Tab`, `Right` a `Enter` přejdou ze seznamu souborů
  do diffu, `Left` a `Tab` zpět; `]` a `[` skočí na další a předchozí soubor.

### Možnosti zobrazení diffu

Panel diffu na stránkách **Soubory** a **Commity** je panel [zobrazení Diff](staging-and-committing.md) se stejnou lištou v prvním
řádku a stejnými pamatovanými volbami (je to jedno nastavení: co zvolíte tady, s tím se zobrazení Diff otevírá,
a naopak):

- **Sjednocené / Vedle sebe**: jeden sloupec, nebo před a po vedle sebe. V rozvržení Vedle sebe leží vlákno, čekající komentář nebo
  návrh pod polovinou, do které patří: komentář k odebranému řádku pod levou polovinou, komentář k
  přidanému nebo nezměněnému řádku pod pravou. Než stisknete **`c`** na řádku, který
  ukazuje odebrání i přidání, klikněte na polovinu, kterou myslíte; klávesnice komentuje novou stranu.
- **Hunky**: jen změny s kontextem, nebo celý soubor se změnami na místě. V celém souboru leží
  vlákno na řádku mimo změny pod svým řádkem místo v *Komentářích mimo zobrazené řádky*.
  Text souboru se čte na pozadí; hunky zůstávají na obrazovce s řádkem, který to říká, dokud nedorazí (na
  stránce Commity se takto čte až 40 souborů commitu).
- **WS**: zobrazit nebo ignorovat změny, které se liší jen v bílých znacích.
- **Zalom.**: zalamovat dlouhé řádky.
- **Šipky nahoru / dolů**: předchozí a další hunk.
- **Map** (**`M`**): minimapa vedle posuvníku, jedno nastavení se zobrazením Diff. Klikem nebo tažením na ní
  se posouvá.
- **Auto** (**`E`**, nebo *Kontrola: Vybrat kódování...*): kódování, ve kterém se soubor čte, jako v zobrazení Diff:
  Automatická detekce nebo jedno z kódování seznamu, zvolené pro soubor, který panel ukazuje (na stránce Commity soubor
  pod kurzorem). Obě strany souboru se v tomto kódování čtou znovu, na pozadí, a
  volba se sdílí se zobrazením Diff pro stejnou cestu. Soubor, který nejde takto přečíst, řekne proč nad
  svým diffem.
- **`⋯`**: když je panel pro všechna tlačítka příliš úzký, ta, která se nevejdou, jsou za `⋯` na konci
  lišty; jeho seznam spustí každé jako po kliknutí. Jejich klávesy a řádky palety fungují jako vždy.

Pravítko posuvníku ukazuje, kde co je: změny (jen celý soubor, jako v zobrazení Diff), tečku `●`
pro každé vlákno, čekající komentář a návrh a kurzor.

Dlouhé řádky se na okraji panelu ořežou (zapněte **Zalom.**, abyste je viděli celé) a posouvají se do stran: **`Right`**
a **`Left`** v diffu (`Left` na levém okraji se vrátí do seznamu souborů), boční kolečko nebo `Shift`
s kolečkem, posuvník podél dolního okraje, nebo *Kontrola: Posunout diff doleva / doprava*.

### Výběr textu a řádků

O tom, co tažení vybere, rozhoduje místo, kde začne. Tažení, které začne **v kódu**, vybere text, jako v zobrazení Diff
(dvojklik vybere slovo); **`Ctrl+C`** (nebo *Kontrola: Kopírovat vybraný text nebo řádek*) zkopíruje přesně ty
znaky, bez čísel řádků a značek, a bez výběru textu zkopíruje řádek pod kurzorem.
Tažení, které začne **v okraji** (čísla řádků, značka, `+`), vybere řádky, abyste je mohli okomentovat
jako dřív, a dvojklik v okraji dál začne komentář k tomu řádku.

Klávesy `n`, `p`, `m`, `w`, `s` zobrazení Diff jsou v kontrole vlastní, takže tam jsou přepínače velkými
písmeny: **`V`** rozvržení, **`H`** hunky nebo celý soubor, **`S`** bílé znaky, **`W`** zalamování, **`J`** / **`K`** další a
předchozí hunk, **`E`** kódování, **`M`** minimapa. Každý má řádek v paletě (*Kontrola: Přepnout sjednocené / vedle sebe*,
...).

### Commity

Jen commity žádosti (ne historie za ní), od nejnovějšího, a diff vybraného z nich
(jeho změny proti rodiči), se všemi jeho soubory pod sebou a stejnými možnostmi zobrazení.

## Psaní kontroly

Komentář ke kódu je **čekající**: uchovává se ve vašem počítači (v souboru stavu repozitáře, takže přežije
restart) a počítá se v záhlaví. Nic se neodešle, dokud kontrolu nedokončíte. Čekající komentář se ukáže
pod svým řádkem s odznakem `Čeká - řádek 12` a dvěma tlačítky: **`e`** (Upravit) a **`d`** (Smazat).

**Dokončit kontrolu** (**`f`**, tlačítko, nebo paleta) otevře malý formulář: shrnutí (volitelné) a
seznam **Rozhodnutí**: *Komentář*, *Schválit*, *Požádat o změny*. Uvádí se jen to, co hostitel dovoluje: GitLab nemá
*Požádat o změny*. **Odeslat kontrolu** (`Ctrl+S`) odešle shrnutí, rozhodnutí a každý čekající komentář
najednou. **Zahodit kontrolu...** po potvrzení čekající komentáře zahodí.

Hned, bez čekání na kontrolu, se odesílá: odpovědi, vyřešení a znovuotevření a komentáře
ke konverzaci. Pole komentáře říká, o jaký druh jde (*Tento komentář se odešle hned* nebo *Tento komentář se přidá do vaší
čekající kontroly*). Když hostitel odpověď odmítne, pole se znovu otevře s vašimi slovy.

Pokud žádost dostala nové commity poté, co jste napsali čekající komentáře, pohled upozorní, že jejich řádky se
mohly posunout (GitHub a GitLab umístí komentář na commit, který jste četli; kontrola starší hlavy se odmítne).

## Gerrit

Server Gerrit se připojuje jako ostatní hostitelé (viz [Hostingové služby a issue trackery](hosting-integration.md): URL
serveru, uživatelské jméno, HTTP heslo) a propojuje se v sekci **PULL REQUESTY** s *projektem*. Jeho změny se uvádějí jako pull
requesty (`#7`, čip `GR`) a kontrolují se ve stejném pohledu, s rozdíly, které Gerrit přináší:

- **Patch sety.** Změna má číslované patch sety (každé nahrání commitu je jeden). Záhlaví má
  combobox **`Patch set N`** v horním okraji vedle karet stránek (**`P`**, nebo řádek palety *Kontrola: Vybrat
  patch set...*). Začíná na nejnovějším; výběr jiného načte soubory a kód toho patch setu a
  vlákna napsaná na jiných patch setech se uvádějí jako *zastaralá*. Kontrola, kterou dokončíte, se odešle na patch set,
  který je zobrazen.
- **Porovnání s jiným patch setem.** Druhý combobox nalevo od něj, **`Porovnat: základ`** (**`D`**, nebo
  *Kontrola: Porovnat patch sety...*), uvádí základ a každý patch set starší než zobrazený. Zvolte patch set 1, zatímco
  je zobrazen patch set 3, a stránka Soubory uvádí a diffuje jen to, co patch set 3 změnil od patch setu 1
  (rozdíl počítá sám server, takže rebase na novější cíl se neukáže jako změny); stránka Commity ukáže tento jediný
  rozdíl. Oba patch sety se stáhnou do `refs/gitpoulpi/pr/<n>` a `refs/gitpoulpi/pr/<n>-ps<k>`. Vlákna
  napsaná na jiných patch setech než zobrazeném jsou *zastaralá*. Zvolíte-li ke čtení starší patch set, než je
  porovnávaný, porovnání se vrátí na základ. Dvojče v CLI je `review show|fetch --compare-with K`.
- **Submit a abandon.** Když server říká, že změnu lze odeslat (jsou tu hlasy a kontroly, které potřebuje), mezi
  tlačítka záhlaví přibude **Submit**; **Abandon** tam je, dokud smíte změnu opustit.
  Obojí (klávesy **`U`** a **`Q`**, paleta *Kontrola: Submit změny...* / *Kontrola: Abandon změny...*) se nejdřív zeptá
  v okně, které říká, že akce proběhne na serveru a **z GitPoulpi ji nelze vrátit** (sloučená změna
  zůstane sloučená; abandonovanou lze obnovit ve webovém rozhraní Gerritu). Odmítnutí nic neodešle; po úspěchu
  zobrazení změnu načte znovu. Dvojčata v CLI jsou `gitpoulpi hosting review submit-change <n>` a
  `hosting review abandon <n> [--message M]`.
- **Hlasy místo rozhodnutí.** **Dokončit kontrolu** ukáže jeden combobox pro každý štítek, pro který smíte hlasovat - `Code-Review`
  (-2 až +2) a `Verified`, kde ho projekt má - a uvádí jen hodnoty, které server povoluje **pro tuto
  změnu a vás** (vývojář bez práva nemůže zvolit +2). Začíná na vašem současném hlasu; odešle se jen hlas, který
  jste změnili. Shrnutí a čekající komentáře jdou stejným voláním. Koncepty, které jste uložili ve webovém
  rozhraní Gerritu, se publikují spolu s kontrolou.
- **Vlákna** jsou řetězce komentářů Gerritu. Vlákno je vyřešené, když jeho nejnovější komentář říká `unresolved: false`;
  **vyřešit** odešle odpověď `Done`, která to udělá, **znovu otevřít** odpověď `Reopened`. Odpovědi a vyřešení jdou na server
  hned, jako u ostatních hostitelů.
- **Odeslání ke kontrole.** Gerrit vytvoří změnu, když odešlete do `refs/for/<branch>`. Když je sekce PULL REQUESTY propojena s projektem Gerritu,
  tlačítko **`+`** sekce a řádek palety **Hosting: Push ke kontrole...** se zeptají na cílovou větev a odešlou `HEAD` do `refs/for/<branch>` (téma lze zadat
  na příkazové řádce). Odeslání nelze z GitPoulpi vrátit (změnu opusťte v Gerritu); zpráva to říká. CLI
  dvojče je `gitpoulpi hosting review push`.
- Tlačítka metadat jsou pro Gerrit jen pro čtení (recenzenti se zobrazují), neexistuje rozhodnutí *Požádat o změny*
  (hlas -1 nebo -2 je totéž) a čip kontrol je štítek `Verified`.

## ✦ Koncept kontroly od AI

![Dva návrhy ✦ pod svými řádky na stránce Soubory, každý s Přijmout, Upravit a Odmítnout, vedle existujícího vlákna](../images/review-ai.png)

Při zapnuté AI (*Nastavení > AI*, viz [AI](ai.md)) má záhlaví tlačítko **`✦ Koncept kontroly...`** (také **`s`**
a řádek palety *Kontrola: ✦ Koncept kontroly...*). Otevře [možnosti kontroly](#možnosti-kontroly) a
**`Enter`** v nich se zeptá modelu. Je to jediná věc, která se ptá modelu: otevření kontroly
to nikdy neudělá. Model čte název a popis žádosti, už existující komentáře (aby se
neopakoval) a diff a odpoví **konceptem kontroly**: textem k odeslání, rozhodnutím, které doporučuje (Schválit, Požádat o změny nebo Komentář), celkovým shrnutím a seznamem navržených komentářů. Dlouhý diff se pro model ořeže - po souborech i celkově - a Přehled řekne, kolik souborů bylo zkráceno
nebo vynecháno. Dokud model pracuje, stavový řádek po chvíli ukáže spinner a **`Esc`** (nebo totéž tlačítko)
ho zruší. Platí instrukce *Kontrola kódu* [čtyř úrovní](ai.md#instrukce), takže tým nebo
pracovní prostor může říct, jak vypadá dobrá kontrola.

### Možnosti kontroly

`s` otevře dialog **✦ Možnosti kontroly**, vyplněný výchozími hodnotami repozitáře (globálními tam, kde je
repozitář nemá). `Enter` (nebo tlačítko **Spustit**) zahájí běh s tím, co dialog ukazuje; `Esc`, **Zrušit**
nebo křížek ho zavře a nic se nezeptá.

| Ovládací prvek | Co dělá |
|---|---|
| **Spustit agenta v repozitáři** | *Podle profilu* (rozhoduje pracovní složka profilu AI), *Ano* nebo *Ne*. Zašedlé, s důvodem pod ním, u profilu HTTP (nemá pracovní složku) a když se žádost kontroluje přes API bez klonu. |
| **CLAUDE.md**, **AGENTS.md** | Předat modelu ten soubor repozitáře, čtený ve větvi, na kterou žádost míří. Když agent běží v repozitáři, přečte si je sám a dialog to říká. |
| **Další soubor** | Ještě jeden soubor: cesta v repozitáři, absolutní cesta nebo odkaz na soubor na hostiteli kódu. **Procházet...** (`Ctrl+B`, nebo `Enter` na tlačítku) otevře výběr souboru, který začíná v kořeni repozitáře (nebo vedle už zadaného souboru); soubor uvnitř repozitáře se zapíše jako cesta relativní ke kořeni, soubor mimo něj jako absolutní cesta. Psaní do pole funguje jako dřív. |
| **Žádný další soubor** | Pro tento běh žádný další soubor, i když ho určuje globální nastavení nebo repozitář; zadání souboru nebo jeho výběr přes **Procházet...** zaškrtnutí zruší. S **Uložit jako výchozí pro tento repozitář** se z toho stane přepínač repozitáře *Žádný další soubor*. Prázdné pole se zde počítá jako žádný soubor, protože dialog začíná u toho, co by běh zdědil. |
| **Uložit jako výchozí pro tento repozitář** | Hodnoty také zachová pro další kontroly: v souboru stavu repozitáře, nebo v `remote-reviews.toml` u repozitáře, který není naklonovaný. Uchová se jen hodnota, která se liší od globální, takže zbytek se dál řídí *Nastavením > AI*. |

Všechno je dosažitelné klávesou `Tab`, `Space` přepíná zaškrtávací políčko a pole cesty je obyčejné textové pole. Stejná
čtyři nastavení jsou v *Nastavení > AI* (*Kontrola kódu*) a v *Nastavení repozitáře: AI*; viz
[Kontext kontroly](ai.md#kontext-kontroly). To, co se modelu nepodařilo předat (chybějící nebo
nečitelný další soubor), a každý soubor, který se oříznul, aby se vešel do limitů, jsou uvedeny na kartě konceptu.

Návrhy **nejsou komentáře**. Ukazují se na dvou místech:

- **Pod svým řádkem na stránce Soubory**, jako duchové komentáře označené `✦` se závažností (`info`, `varování`, `problém`) a
  třemi tlačítky: **Přijmout**, **Upravit** a **Odmítnout**. Návrh k řádku, který diff nezobrazuje, se
  uchová, nikdy se nezahodí: uvádí se za hunky pod *Návrhy AI k celému souboru* a po
  přijetí skončí na prvním změněném řádku souboru se slovy *(k řádku N)* na začátku.
- **V Přehledu**, v bloku nad konverzací (jen po spuštění): shrnutí, potom seznam se stejnými tlačítky a **Přijmout vše** /
  **Odmítnout vše**.

**Přijmout** (`y`) změní návrh v běžný **čekající komentář** (můžete ho dál upravit nebo smazat);
**Upravit** (`w`) otevře pole komentáře se slovy modelu a přijme, co v něm necháte; **Odmítnout** (`n`)
ho zahodí. **Přijmout vše** (`A`) jen naplní seznam čekajících. Na stránce Soubory klávesy působí na
návrh pod kurzorem, v Přehledu na první ze seznamu (`y` a `n` jsou zase *Kopírovat odkaz* a *Další nepřečtený* konverzace, jakmile je seznam prázdný). **Použít shrnutí od AI** (`i`, nebo tlačítko
ve formuláři Dokončit kontrolu) vloží shrnutí modelu do textu kontroly; nikdy se nevloží samo.
**Nic se neodešle hostiteli, dokud kontrolu neodešlete.**

Přehled ukazuje koncept jako kartu na začátku toho bloku, nad návrhy: čip s rozhodnutím, text kontroly a tři
tlačítka.

- **✦ Použít koncept...** (`u`) přijme každý otevřený návrh jako čekající komentář a otevře *Dokončit kontrolu* už
  vyplněné textem a rozhodnutím konceptu. Stále si ho sami přečtete, změníte a odešlete; zrušení
  formuláře koncept zachová. Limity hostitele zůstávají: GitLab nemá *Požádat o změny*, takže to formulář
  řekne a vrátí se k *Komentáři*; u Gerritu se rozhodnutí stane hlasem Code-Review +1 nebo -1, jen když
  server tuto hodnotu pro vás povoluje (jinak se žádný hlas nenastaví a formulář to řekne).
- **Upravit text...** (`t`) změní text kontroly v konceptu, než ho použijete.
- **Zahodit koncept** (`N`) zahodí text, rozhodnutí a zbývající návrhy.

Koncept se uchovává pro každou žádost, takže tam je i po zavření a opětovném otevření kontroly. Když se
hlava žádosti od napsání konceptu modelem posunula, karta řekne *Žádost se od napsání tohoto konceptu změnila.*
Odeslání kontroly koncept spotřebuje.

## Všechno má klávesu a řádek palety

| Akce | Klávesa | Paleta |
|---|---|---|
| Přehled / Soubory / Commity | `Alt+1` / `Alt+2` / `Alt+3` | *Kontrola: Zobrazit přehled*, *Zobrazit soubory*, *Zobrazit commity* |
| Okomentovat řádek nebo rozsah | `c` | *Kontrola: Okomentovat řádek...* |
| Okomentovat konverzaci | `c` v Přehledu, `C` | *Kontrola: Okomentovat konverzaci...* (otevře pole Přehledu), *Konverzace: Nový komentář* |
| Otevřít kód vlákna (přehled) / zpět ke komentáři | `G` / `Esc` | *Konverzace: Otevřít kód vlákna*, *Kontrola: Zpět ke komentáři v přehledu* |
| Odpovědět ve vlákně | `r` | *Kontrola: Odpovědět ve vlákně...* |
| Vyřešit nebo znovu otevřít | `x` | *Kontrola: Vyřešit nebo znovu otevřít vlákno* |
| Sbalit nebo rozbalit vlákno | `z` | *Kontrola: Sbalit nebo rozbalit vlákno* |
| Upravit / smazat čekající komentář | `e` / `d` | *Kontrola: Upravit čekající komentář...*, *Smazat čekající komentář* |
| Označit soubor jako zobrazený | `v` | *Kontrola: Označit soubor jako zobrazený* |
| Sjednocené / Vedle sebe | `V` | *Kontrola: Přepnout sjednocené / vedle sebe* |
| Celý soubor / jen hunky | `H` | *Kontrola: Přepnout celý soubor / jen hunky* |
| Změny jen v bílých znacích | `S` | *Kontrola: Přepnout změny jen v bílých znacích* |
| Zalamování řádků | `W` | *Kontrola: Přepnout zalamování řádků* |
| Další / předchozí hunk | `J` / `K` | *Kontrola: Další hunk*, *Předchozí hunk* |
| Další / předchozí soubor | `]` / `[` | *Kontrola: Další soubor*, *Předchozí soubor* |
| Dokončit / zahodit kontrolu | `f` | *Kontrola: Dokončit kontrolu...*, *Zahodit čekající kontrolu...* |
| Obnovit | `F5`, `Ctrl+R` | *Kontrola: Obnovit* |
| Otevřít v prohlížeči / checkout | `o` / `b` | *Kontrola: Otevřít v prohlížeči*, *Checkout větve* |
| Řešitelé / recenzenti / štítky / milník | `a` / `p` / `l` / `m` | *Kontrola: Upravit řešitele...* a jeho tři sourozenci |
| Vybrat patch set (Gerrit) | `P` | *Kontrola: Vybrat patch set...* |
| Porovnat se starším patch setem (Gerrit) | `D` | *Kontrola: Porovnat patch sety...* |
| Submit / abandon změny (Gerrit) | `U` / `Q` | *Kontrola: Submit změny...*, *Kontrola: Abandon změny...* |
| ✦ Koncept kontroly... (AI zapnutá; otevře možnosti kontroly) | `s` | *Kontrola: ✦ Koncept kontroly...* |
| Přijmout / upravit a přijmout / odmítnout návrh | `y` / `w` / `n` | *Kontrola: Přijmout návrh*, *Upravit a přijmout návrh...*, *Odmítnout návrh* |
| Přijmout / odmítnout každý návrh | `A` / `N` | *Kontrola: Přijmout všechny návrhy*, *Odmítnout všechny návrhy* |
| Použít shrnutí od AI v textu kontroly | `i` | *Kontrola: Použít shrnutí od AI v textu kontroly* |
| Použít koncept ✦ (vyplní Dokončit kontrolu) | `u` | *Kontrola: ✦ Použít koncept...* |
| Upravit text konceptu | `t` | *Kontrola: Upravit text konceptu ✦...* |
| Zahodit koncept ✦ | `N` | *Kontrola: Zahodit koncept ✦* |
| Zavřít | `Esc` | *Kontrola: Zavřít* |

Otevřete paletu (`Ctrl+P`), zatímco je pohled otevřený: její řádky na něm fungují a říkají, proč některý právě
teď nic nedělá („nejdřív umístěte kurzor na řádek kódu“).

## Kontrola bez klonu

Žádost z repozitáře, který na vašem počítači není, lze také zkontrolovat. Z Radaru **Zkontrolovat**
(`e`) na jejím řádku se zeptá jak (viz [Radar](radar.md#kontrola-žádosti)); **Zkontrolovat přes API** otevře
tento stejný pohled v okně Domů, nad Radarem. Všechno na této stránce funguje stejně, s těmito
rozdíly:

- Kód (změněné soubory, diff každého souboru, commity žádosti, diff commitu, celý soubor) se
  čte z API hostitele, takže první pohled na velkou žádost chvíli trvá, a soubor, k němuž hostitel
  diff neposlal (binární nebo velmi velký), to řekne místo zobrazení diffu.
- Vaše čekající komentáře se uchovávají v `remote-reviews.toml` ve složce konfigurace, zařazené pod hostitele
  a projekt (`host/project`), místo ve stavu repozitáře. Jsou tam i po zavření a opětovném otevření
  kontroly a odeslání kontroly je spotřebuje.
- Není na výběr kódování (*Bez klonu není k dispozici*) a není **Checkout** (řekne, ať se
  repozitář nejdřív naklonuje).
- `Esc` nebo `✕` v rohu pohled opustí a vrátí Radar. Paleta (`Ctrl+P`) dosáhne na řádky
  pohledu stejně jako na kartě repozitáře.

Chcete-li klon ponechat, zvolte **Naklonovat a zkontrolovat** (nebo v paletě *Radar: Naklonovat a zkontrolovat...*):
dialog Klonovat se otevře na připojeném účtu dané žádosti s vybraným repozitářem a kontrola se otevře na kartě nového repozitáře, jakmile klon dorazí.

## Z příkazové řádky a pro agenty

Stejná kontrola je dostupná jako `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(a nástroje MCP z nich vytvořené): viz [Referenční příručka CLI](cli-reference.md). `threads` vypisuje také už
odeslaná rozhodnutí (`reviews`). `suggest <n>` se zeptá modelu (AI zapnutá) a vypíše jeho `verdict`, `body`, shrnutí a
návrhy jako JSON, aniž cokoli odešle, takže skript nebo agent může vybrat komentáře, které předá
`submit --comment`.

Nic z toho nepotřebuje klon: `--project owner/name` (s `--id`, je-li nakonfigurováno víc integrací)
spustí `show`, `threads`, `comment`, `reply`, `resolve`, `submit` a `suggest` proti projektu, o kterém aktuální
adresář nic neví. `hosting review diff <n>` vypíše diff s jeho hunky (`--api` ho přečte od hostitele i uvnitř klonu)
a `hosting file <path> --project P [--ref R]` přečte soubor v dané revizi, například
`CLAUDE.md` projektu. Oba čtou API hostitele: GitHub, GitLab, Gitea, Forgejo a Gerrit.
