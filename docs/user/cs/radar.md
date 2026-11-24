[English](../radar.md) · **Čeština** · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# Radar

Radar je samostatná karta hned za kartou Domů: jeden seznam toho, co na vás čeká napříč **více repozitáři**:
pull (nebo merge) requesty, které jste napsali, ty, které žádají o vaši kontrolu,
issues přiřazené vám nebo vytvořené vámi a repozitáře, kde jste nechali
rozpracovanou práci. Je to okno Domů se zobrazením Radar navrchu, aby nic
pod ním nemohlo ukrást klik ani klávesu.

Potřebuje profil hostitele kódu (viz
[Hostingové služby a issue trackery](hosting-integration.md); seznamy jednoho repozitáře jsou v [Pull requestech a issues](pull-requests-and-issues.md) a diskuse řádku se otevře jako [konverzace](conversations.md)) a repozitáře s ním
propojené, nebo profil se zapnutými seznamy za celý účet (viz „Mimo rozsah“ níže).
Nepropojený repozitář se nikdy nedotazuje: místo toho ukáže řádek
**Propojit...** (viz „Repozitáře, které mají co říct“ níže).

![Radar nad dvěma repozitáři: připnutá žádost, pak Čekají na mou kontrolu, Moje pull requesty, Přiřazené mně, issues a rozpracovaná práce](../images/radar.png)

**Zkontrolovat...** na řádku pull nebo merge requestu (nebo `e`) otevře repozitář a v něm
kontrolu žádosti v aplikaci ([Kontrola pull requestu](review.md)). U žádosti repozitáře, který jste naklonovali, ale nikdy
nepropojili s hostitelem, se klon nejdřív propojí s projektem žádosti (pokud už nemá vlastní propojení pull requestu), takže
kontrola začne v jeho kartě.

### Sloupec Akce

Vpravo od řádku žádosti nebo issue dělají malá tlačítka to, co klávesa na vybraném řádku (klik řádek vybere a akci
spustí; ukazatel nechte na tlačítku pro jeho název a klávesu):

```
Wide:    #58  Round invoice totals  feature/login → main   alice   2d   Review  [globe]  [talk]  ⊕  ×  ⇣  ⋮
Narrow:  #58  Round invoice totals  feature/login → main                 [rev]  [globe]  [talk]  ⋮
Issue:   #12  Crash on empty list                         bob     5h   Start work  [globe]  [talk]  ×  ⋮
```

| Tlačítko | Klávesa | Paleta |
|---|---|---|
| **Zkontrolovat** (žádost, která čeká na vaši kontrolu; u nenaklonované se zeptá jak) | `e` | `Radar: Zkontrolovat žádost...` |
| **Zahájit práci** (naklonovaná issue) | `n` | `Radar: Začít pracovat na issue...` |
| zeměkoule: otevřít stránku na hostiteli | `o` | `Radar: Otevřít v prohlížeči` |
| glyf řádků: otevřít konverzaci | `i` | `Radar: Otevřít konverzaci issue` |
| `»` **Označit jako připravené** (konceptová žádost) | `y` | `Radar: Označit jako připravené` |
| `⊕` **Merge** (žádost) | `m` | `Radar: Merge pull request...` |
| `×` **Zavřít** (žádost nebo issue) | `x` | `Radar: Zavřít...` |
| `⇣` **Klonovat** (řádek, jehož repozitář není naklonovaný) | `g` | `Radar: Klonovat repozitář...` |

Řádek bez akce nechá její buňku prázdnou, takže každé tlačítko zůstává na každém řádku ve stejném sloupci. Když je okno
úzké, slova Zkontrolovat a Zahájit práci se změní na glyfy dřív, než zmizí jakýkoli jiný sloupec; tlačítka Připraveno, Merge,
Zavřít a Klonovat opustí sloupec dřív než název, ale zůstanou v nabídce řádku (`⋮`, pravé tlačítko, nebo `Shift+F10`), kde je akce,
která nelze provést, zašedlá s důvodem. Sloupec je automatický (není to jeden ze sloupců s nastavitelnou šířkou). Dvojklik nebo
`Enter` na žádosti, která není naklonovaná, se zeptá, jak ji zkontrolovat, stejně jako **Zkontrolovat**.

### Větve žádosti

Řádek žádosti ukazuje `zdroj → cíl` ve sloupci **Větve** hned za repozitářem (`alice:feature/login → main`, když je zdrojová větev
ve forku); řádek issue nechá buňku prázdnou a Radar bez žádosti, která své větve pojmenovává, takový sloupec nemá. Když je buňka
příliš úzká, ořízne se nejdřív zdroj (výpustkou) a cíl zůstane celý, dokud to jde. Sloupec se nejdřív zužuje, než zmizí: v úzkém
okně odchází jako poslední z volitelných sloupců (po stavu, autorovi a stáří, před repozitářem a názvem); když zmizí, celé větve
zůstanou v tooltipu názvu a na stavovém řádku vybraného řádku. Široký sloupec autora, který jste si nastavili, ustoupí názvu
a větvím. Sloupec Akce se nikdy nevytlačí. Ukazatel na oříznutém textu ukáže celý. GitHub a Gitea vypisují žádosti bez větví, takže je Radar
čte po načtení seznamu (jednou pro každou žádost, dokud se nezmění). Hostitel, který větve nepojmenoval, tam nic neukáže. Řádek issue
pojmenuje svůj repozitář ve stejném čipu jako řádek žádosti (název složky klonu, nebo `owner/name`, když není naklonovaný).

### Jednání na hostiteli

**Merge**, **Zavřít** a **Označit jako připravené** fungují na GitHubu, GitLabu, Gitea a Forgejo (jiní hostitelé je zašedí s „Not supported
by ...“; issues lze zavřít na týchž hostitelích).

- **Merge** se nejdřív zeptá: okno pojmenuje žádost, její projekt a větve a nabídne metodu (merge commit, squash
  a rebase, kde ho hostitel má; GitLab provede merge nebo squash). Odmítne se s důvodem u konceptu, žádosti, která je
  v konfliktu s cílem, nebo selhávajících kontrol; odmítnutí hostitelem (chybí oprávnění, pravidlo větve) se ukáže na
  stavovém řádku.
- **Zavřít** se nejdřív zeptá a zavře bez merge.
- **Označit jako připravené** vyjme koncept z konceptu (GitLab a Gitea: značka `Draft:` / `WIP:` zmizí z názvu). Na nic
  se neptá: na hostiteli je vratné.
- **Klonovat** otevře obyčejný dialog Klonovat na projektu řádku (URL je vyplněná, výběr složky je obvyklý). Jakmile
  klon existuje, Radar řádek na něj umístí při dalším čtení remote (znovu ho otevřete nebo Obnovte) a Klonovat pak
  řekne, kde klon je.

Merge a Zavřít mění hostitele a **z GitPoulpi je nelze vrátit zpět** (okno to říká); zavřenou žádost znovu otevřete na hostiteli.
Po úspěšném volání se seznamy načtou znovu, takže sloučený nebo zavřený řádek zmizí a připravený ztratí čip `draft`.

## Otevření

- **`Alt+R`** (přeřaditelné: `global.radar`), odkudkoli.
- Paleta: `Radar: Otevřít`.
- Karta **Radar** hned za Domů v řádku karet.
- Řádek **Radar** v nabídce **`+`**, dokud je karta zavřená.

Z karty repozitáře vás to přenese do okna Domů s otevřeným Radarem; `Esc`
nebo **Zpět** vrátí na seznam repozitářů a `⌂` dělá totéž.

## Zavření karty nebo vypnutí Radaru

Karta se zavírá jako karta repozitáře: tlačítkem **x**, prostředním tlačítkem myši, `Ctrl+W`, když je zobrazena, nebo příkazem palety `Radar: Zavřít kartu`. Domů je jediná karta, kterou nelze zavřít. Zavřená karta zůstane pro danou session zavřená i po restartu, dokud Radar znovu neotevřete (`Alt+R`, `Radar: Otevřít` nebo nabídka `+`); pak se vrátí hned za Domů. Zavření nemění žádné nastavení.

Pokud jej nepoužíváte, vypněte jej v **Nastavení > Obecné > Povolit Radar** (`general.radar_enabled`, viz [referenční přehled konfigurace](config-reference.md)). Při vypnutí zmizí karta, její příkazy v paletě a `Alt+R` a GitPoulpi pro něj nic nestahuje. Projeví se hned, bez restartu.

## Co pokrývá: rozsah

Combobox **Rozsah** vybírá, které repozitáře se dotazují:

- **pracovní prostor** podle názvu (jeho repozitáře, i ty bez otevřené karty);
- **Otevřené karty**: repozitáře otevřené jako karty této relace.
- **Výchozí složka pro klonování** a **Výchozí složka pro procházení**: každý repozitář nalezený ve složce nastavené v
  [Nastavení](config-reference.md) (`repos.clone_root`, `repos.browse_root`), ať má kartu, nebo ne.

Otevře se na naposledy zvoleném rozsahu (uchovává se v `radar.toml`), pokud ještě existuje;
před první volbou, nebo když byl ten pracovní prostor smazán, na aktivním pracovním prostoru relace,
jinak na otevřených kartách. Volba platí jen pro tento pohled: nemění aktivní pracovní prostor okna.
Mají-li dva repozitáře stejný název složky, přidá se k čipu nadřazená složka, abyste je rozlišili.

### Dva rozsahy podle složky

Rozsah podle složky hledá repozitáře **do hloubky dvou úrovní**: složky přímo v ní a složky uvnitř
těch, které samy repozitáři nejsou (`~/src/work/app`). Skryté složky se přeskakují, do repozitáře se nikdy
nevstupuje a vypíše se nejvýše 200 repozitářů. Hledání běží na pozadí, nikdy ve vlákně, které kreslí, a jeho
výsledek se drží do obnovení: tlačítko Obnovit, `r`, `F5` (obnovovací klávesa okna, předefinovatelná) nebo `Radar: Obnovit` prohledá složku znovu
(repozitář naklonovaný od té doby se objeví).

Hosting zůstává dobrovolný pro každý repozitář: repozitář ze složky, který **není propojený** s hostitelem,
se nikdy nedotazuje. Přispívá jen svým řádkem rozpracované práce, má pod Repozitáři řádek `nepropojeno` s
**Propojit...** (otevře sekci pull requestů toho repozitáře, kde vyberete poskytovatele a projekt) a stavový
řádek je počítá (`3 nepropojené repozitáře (jen rozpracovaná práce)`).

Složka, která není nastavená, je v seznamu zašedlá (`Výchozí složka pro procházení (nastavte v Nastavení)`);
její výběr otevře její řádek v Nastavení (Nastavení > Obecné) a rozsah nechá, jaký byl. Složka pro klonování
se počítá jako nastavená díky výchozí hodnotě podle systému (složka, na které začíná dialog Klonovat), dokud
tato složka existuje; složka pro procházení výchozí hodnotu nemá. Seznam rozsahů se čte z nastavení při každém
otevření, takže změna tam platí hned.

## Seznam

| Skupina | Co obsahuje |
|---|---|
| Připnuté | Vše, co jste připnuli, ať je to cokoli, v pořadí, v jakém jste připínali |
| Čekají na mou kontrolu | Žádosti, které žádají o vaši kontrolu (a které jste nenapsali) |
| Moje pull requesty | Žádosti, které jste napsali; čip říká `koncept` u konceptu |
| Přiřazené mně | Žádosti přiřazené vám |
| Zmiňující mě | Otevřené žádosti a issues, které vás zmiňují a nejsou už v některé skupině výše (issue, které vám není přiřazeno ani jste ho nevytvořili, se tu ukáže jen tehdy, když vás zmiňuje) |
| Ostatní pull requesty | Zbývající otevřené žádosti |
| Issues přiřazené mně | Otevřené issues přiřazené vám |
| Issues vytvořené mnou | Otevřené issues, které jste vytvořili |
| Rozpracované | Repozitáře v rozsahu s necommitnutými změnami nebo neodeslanými commity |
| Odložené | Položky, které jste na chvíli skryli; sbalené, dokud je neotevřete |
| Repozitáře | Repozitáře, které mají co říct (offline, token k obnovení, nepropojené) |

Slova „pull request“ a „merge request“ se řídí běžným nastavením terminologie a
hostitelem.

Každý řádek ukazuje druh položky (`⇄` žádost, `●` issue, `✎` rozpracovaná
práce), čip repozitáře (v barvě pracovního prostoru), číslo, název, autora, stáří
a, je-li místo, štítky. Řádek rozpracované práce ukazuje větev a co v ní čeká:
`✎` změněné soubory, `+` nové soubory, `↑` commity k odeslání, `↓` commity ke
stažení.

Počty jsou v každém nadpisu. Uvnitř skupiny jde první naposledy změněná položka.

Stavový řádek pod filtrem ukazuje spinner, když hostitelé odpovídají déle než
chvíli. Uložené seznamy (stejná cache, kterou používá postranní panel) se
ukážou hned a nahradí se, jakmile dorazí odpovědi.

### Sloupce

Čip repozitáře, větve, `#` (číslo a značky kontrol), autor a stáří jsou **sloupce s nastavitelnou šířkou**. Ponechaný na automatice je sloupec repozitáře
tak široký jako nejdelší název na obrazovce (nejvýše 40 buněk a nejvýše třetina okna), takže `owner/name`
i `group/sub/project` z účtového seznamu se na běžném terminálu neořežou. Název dostane zbytek. Název, který se
nevejde, končí třemi tečkami (`group/sub...`), a:

- **Zastavte ukazatel** na uříznutém repozitáři, názvu, autorovi nebo seznamu štítků a tooltip ukáže celý text.
  Vyberte řádek a stavový řádek pod filtrem také pojmenuje celý projekt i název, takže klávesnice vidí totéž.
- **Přetáhněte oddělovač** (buňku hned za názvem repozitáře, za `#` nebo těsně před autorem či stářím) a sloupec změní šířku;
  **dvojklikem** na něj sloupec znovu přizpůsobíte. Seznam má **řádek záhlaví** (repozitář, větve, `#`,
  název, autor, stáří) a oddělovače se řídí volbou `ui.column_resize` jako v Logu: s `line` (výchozí)
  v oddělovači na řádku záhlaví leží tlumené `│` a dokud je ukazatel na oddělovači (v záhlaví nebo na
  kterémkoli řádku seznamu), dokud je sloupec vybrán klávesami nebo dokud táhnete, kreslí se přes celý
  seznam tenká akcentní čára; tažení odkudkoli z ní mění šířku. Po pustění čára zmizí, pokud ukazatel
  na oddělovači stále není (pak zmizí s ukazatelem): tažení myší nikdy nenechá sloupec vybraný. S `header` není
  značka ani čára: první buňka za oddělovačem na řádku záhlaví se rozsvítí a je jediným místem, odkud se dá
  táhnout. Nad oddělovačem se ukazatel mění na ukazatel změny velikosti v terminálech, které to umějí. Čip
  stavu má pevnou šířku, stejně tak stáří (nejširší zobrazené stáří, dokud šířku nenastavíte; stáří ořezané
  úzkým sloupcem ukáže své úplné datum v tooltipu), takže autor začíná na každém řádku ve stejné buňce.
- Nebo použijte klávesy: **`C`** vybere sloupec (znovu, nebo `Tab`, pro další), **`]`** ho rozšíří o dvě
  buňky, **`[`** ho zúží, **`0`** vrátí všechny sloupce na automatiku, `Esc` skončí. Paleta má totéž:
  `Radar: Vybrat sloupec`, `Radar: Rozšířit sloupec`, `Radar: Zúžit sloupec`, `Radar: Obnovit šířky sloupců`.

Šířky se uchovávají v `radar.toml` (`[columns]`, `repo`, `branches`, `number`, `author` a `age`, v buňkách; soubor bez `branches` se načte jako dřív, sloupec se pak přizpůsobí obsahu do 28 buněk) a přežijí restart. Jsou to stejná
gesta pro změnu velikosti jako u sloupců Logu.

### Kontroly a konflikty

Řádek žádosti nese za svým číslem dvě malé značky, když hostitel něco řekl:

| Značka | Význam |
|---|---|
| `✓` | kontroly head commitu prošly |
| `✗` | některá kontrola selhala |
| `●` | kontroly čekají ve frontě nebo běží |
| `!` (vedle ní) | žádost je v konfliktu se svou cílovou větví |

Zastavte ukazatel na značkách a tooltip to řekne slovy („Kontroly selhaly“, „Konflikt s main“);
vyberte řádek a stavový řádek pod filtrem řekne totéž, takže to vidí i klávesnice. Žádost
bez značky je taková, o které hostitel nic neřekl (není nastavené CI, nebo se na ni neptal).

- **GitLab** posílá stav pipeline a příznak konfliktu spolu se seznamem merge requestů: každý řádek
  má své značky.
- **GitHub** ne: seznam nemá ani jedno. GitPoulpi se ptá po jednotlivých žádostech (na samotnou žádost kvůli
  stavu sloučení, na kombinovaný stav head commitu a jeho check runs: tři volání), nejvýše pro
  **15 otevřených žádostí na repozitář**, vašich nejdřív. Když GitHub některé z těch volání odmítne (limit
  požadavků), ostatní zůstanou bez značky do dalšího obnovení. Odpovědi se ukládají spolu s cache hostingu.

- **Gitea a Forgejo** se čtou jako GitHub: seznam nese příznak konfliktu (`mergeable`), stav commitu se
  zjišťuje po jednotlivých žádostech (nejvýše 15 na repozitář, vašich nejdřív) a issues jsou stejné
  skupiny. Zmínky používají vyhledávání serveru (`mentioned_by`).

### Zmínky

**Zmiňující mě** a filtr **Zobrazit: Zmiňující mě** používají to, co hostitel o zmínkách ví. GitHub se
ptá přes své vyhledávání (`mentions:`), GitLab přes váš seznam úkolů: počítají se jen zmínky, které tam
stále **čekají**, takže úkol, který jste označili jako hotový, ze skupiny zmizí.

Uzavřené a sloučené žádosti a uzavřené issues se nevypisují.

## Zobrazení a filtry

Seznam zužují čtyři comboboxy a textové pole. Každá volba je rozbalovací
nabídka: klikněte na ni, nebo použijte její klávesu.

- **Zobrazení** (`v`): Vše, žádosti, Issues, WIP (rozpracovaná práce), Odložené.
- **Zobrazit** (`f`): Všichni, Moje (žádosti, které jste napsali, issues, které
  jste vytvořili), Přiřazené mně, Vyžádaná kontrola, Zmiňující mě.
- **Rozsah** (`w`): viz výše.
- **Profil** (`P`): Všechny profily, nebo jeden profil integrace z Nastavení > Integrace (pojmenovaný tak, jak
  ho pojmenovává Nastavení, s přidaným poskytovatelem, když si dva profily dělí název). Zúží každou sekci, řádky
  repozitářů i řádky celého účtu, na řádky tohoto profilu (řádek rozpracované práce profil nemá, takže zvolený profil
  ho skryje). Vždy jen jeden profil. Volba se pamatuje v `radar.toml`; profil, který už neexistuje, se vrátí na
  Všechny profily. Combobox je vždy přítomen jako ostatní tři a úzké okno ho (a pak popisky) zkrátí, místo aby ho
  vynechalo. Paleta: `Radar: Filtrovat podle profilu...`. Když profil skryje všechno, prázdný seznam to řekne.
- **Text filtru** (`/` nebo `Ctrl+F`): fuzzy shoda v repozitáři, čísle, názvu,
  autorovi, větvi a štítcích. `Enter` vrátí klávesy seznamu a text ponechá,
  `Esc` ho smaže.

`Ctrl+N` zobrazí odznak u filtru, každého ze čtyř rozbalovacích seznamů, tlačítek Obnovit a Zpět a každého nadpisu sekce na obrazovce. Stiskněte znak odznaku: filtr dostane klávesnici, rozbalovací seznam se otevře, tlačítko se provede a nadpis dostane výběr (`Enter` nebo `Space` ho pak sbalí). `Esc` zruší. Viz [Odznaky](keybindings.md#odznaky).

## Akce

Vyberte řádek a použijte jeho nabídku (pravé tlačítko, `⋮` na konci řádku, `.`
nebo `Menu`), klávesu, nebo paletu (`Radar: ...`). Akce myší má vždy
klávesu a dvojče v paletě.

| Akce | Klávesa | Paleta |
|---|---|---|
| Otevřít repozitář (jeho karta se otevře, nebo se na ni přepne; u žádosti, která není naklonovaná, se zeptá, jak ji zkontrolovat) | `Enter` / dvojklik | `Radar: Otevřít repozitář` |
| Zkontrolovat... (žádost; u nenaklonované se zeptá jak) | `e` | `Radar: Zkontrolovat žádost...` |
| Zkontrolovat žádost přes API hostitele (bez klonu) | `a` v otázce | `Radar: Zkontrolovat přes API` |
| Naklonovat repozitář, pak zkontrolovat žádost | `c` v otázce | `Radar: Naklonovat a zkontrolovat...` |
| Otevřít v prohlížeči | `o` | `Radar: Otevřít v prohlížeči` |
| Checkout větve (žádosti) | `c` | `Radar: Checkout větve` |
| Vytvořit worktree... (žádosti) | `t` | `Radar: Vytvořit worktree...` |
| Zobrazit v repozitáři (jeho seznam) | `s` | `Radar: Zobrazit v repozitáři` |
| Začít pracovat na issue... (issue) | `n` | `Radar: Začít pracovat na issue...` |
| Připnout nahoru, nebo Odepnout | `p` / klik na `★` | `Radar: Připnout nebo odepnout` |
| Upravit řešitele... / recenzenty... / štítky..., Nastavit milník... (GitHub, GitLab) | nabídka řádku | `Radar: Upravit řešitele...`, `Radar: Upravit recenzenty...`, `Radar: Upravit štítky...`, `Radar: Nastavit milník...` |
| Odložit... | `z` | `Radar: Odložit...` |
| Přestat odkládat | `u` | |
| Zobrazit nebo skrýt žádosti nenaklonovaných repozitářů | `a` | `Radar: Zobrazit nenaklonované žádosti` |
| Zobrazit nebo skrýt koncepty řádků za celý účet | `d` | `Radar: Zobrazit koncepty` |
| Zapnout žádosti o kontrolu za celý účet u hostitele | `T` / `Enter` na jeho řádku s nápovědou / **Zapnout** | `Radar: Zapnout žádosti o kontrolu za celý účet...` |
| Obnovit | `r`, `F5` | `Radar: Obnovit` |
| Nabídka řádku | `.` | `Radar: Nabídka řádku...` |
| Zpět na Domů | `Esc` | `Radar: Zpět na úvod` |

Akce, které potřebují vlastní okno repozitáře (checkout, worktree, zobrazit,
začít pracovat, propojit, výběry metadat), nejdřív přepnou na jeho kartu a proběhnou tam, jakmile
se načte:

- **Checkout** používá vlastní checkout postranního panelu: větev, když je
  lokálně známa, jinak se head žádosti načte do lokální větve (například u forku).
- **Vytvořit worktree...** otevře dialog Vytvořit worktree na větvi žádosti.
- **Zobrazit v repozitáři** vybere položku v seznamu PULL REQUESTS nebo ISSUES.
- **Začít pracovat na issue** otevře výzvu Nová větev s názvem vytvořeným z issue
  (`12-fix-the-thing`).

### Připínání a odkládání

**Připnutí** drží položku nahoře. **Odložení** ji skryje na **1 hodinu**, **4
hodiny**, **Do zítřka** (8:00 následujícího rána vašeho místního času, i když ještě není 8:00) nebo **Dokud se nezmění** (hostitel ohlásí
aktualizaci položky, nebo se u řádku rozpracované práce pohnou jeho počítadla).
Obojí se uchovává v `radar.toml` ve složce konfigurace, podle hostitele,
repozitáře, druhu a čísla, a sdílí to každá relace na počítači.

### Repozitáře, které mají co říct

Repozitář s problémem dostane na konci seznamu vlastní řádek a jeho uložené
položky zůstanou na obrazovce:

- **offline, stav před 5m**: hostitele nebylo možné dosáhnout;
- **Znovu připojit**: token chybí nebo byl odmítnut; tlačítko otevře Nastavení >
  Integrace;
- **v tomto kole pozastaveno**: hostitel řekl, ať zpomalíme (nebo odmítl token),
  takže se GitPoulpi v tomto kole nedotazoval znovu na ostatní repozitáře;
- **nepropojeno, navrhován GitHub**: remote repozitáře odpovídá profilu, ale
  hosting se zapíná pro každý repozitář zvlášť, takže se na nic nedotazuje, dokud
  ho nepropojíte. **Propojit...** vás přenese do sekce PULL REQUESTS repozitáře,
  kde vyberete poskytovatele a projekt.

Najednou se dotazují nejvýše čtyři repozitáře a nikdy ne ve vlákně, které
kreslí obrazovku.

## Mimo rozsah: řádky za celý účet

Rozsah pokrývá repozitáře, které jste naklonovali. Žádost čekající na vaši kontrolu v repozitáři, který jste
nikdy neklonovali, tam není vidět. **Řádky za celý účet** se ptají přímo hostitele, jednou za profil, a
připojí se k seznamu bez ohledu na rozsah: týkají se vás, ne složky.

Jsou **ve výchozím stavu vypnuté** (vyhledávání u hostitele stojí kvótu API). Dokud připojený hostitel kódu nemá
zapnutý žádný ze svých seznamů, nahoře v seznamu je pro něj řádek:

`GitHub: vypsat žádosti čekající na vaši kontrolu, i v repozitářích, které jste nenaklonovali  [Zapnout]`

Klikněte na **Zapnout**, stiskněte `Enter` na tom řádku, stiskněte `T`, nebo spusťte `Radar: Zapnout žádosti o kontrolu
za celý účet...`: zapíše `integrations.radar.review_requests = true` pro daný profil do `config.toml` a
hned se zeptá hostitele. Ostatní seznamy (každá otevřená žádost vašich jmenných prostorů, issues přiřazené vám,
koncepty, jmenné prostory) se nastavují příkazem `gitpoulpi hosting integrations radar` nebo v nastavení profilu; viz
[referenční příručku konfigurace](config-reference.md) a [referenční příručku CLI](cli-reference.md). Odpovídají GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center a Gerrit; Bitbucket Cloud nemá koncový bod pro celý účet,
takže se jeho žádosti ukazují jen pro propojené repozitáře rozsahu.

**Jmenné prostory** profilu omezují všechny jeho seznamy na některé repozitáře (Nastavení > Integrace > Radar: textová oblast s jednou položkou na řádek a zaškrtávátko **Regulární výrazy**). Řádek odpovídá celé cestě repozitáře, když se jí rovná nebo je její předponou končící na `/`: `orbit` ponechá `orbit/products/x` a vyřadí `orbitx/y` i `training/y`. Se zapnutým zaškrtávátkem je každý řádek regulární výraz spuštěný nad celou cestou tak, jak ho zapíšete (`^orbit/(products|infra)/`); neplatný řádek se ukáže pod polem a neuloží se. Prázdné znamená bez omezení.

Jak se řádky řadí:

- Řádek, který rozsah už uvádí (stejný hostitel, projekt, druh a číslo), se objeví jednou: **vyhrává řádek
  rozsahu**.
- Řádek, jehož projekt jste někde naklonovali (nedávné repozitáře, pracovní prostory, složky, které našly
  rozsahy podle složky; porovnává se podle URL remote), nese ten klon: **Enter**, checkout, worktree a
  ostatní ho otevřou jako každý řádek.
- Každý další řádek je **nenaklonovaný**: čip repozitáře je ztlumený a ukazuje projekt (`acme/app`) a
  stavový řádek po jeho výběru říká `Nenaklonované`. **Otevřít v prohlížeči** funguje, **Zkontrolovat** se zeptá,
  jak ho zkontrolovat (níže), a checkout, worktree, zobrazit v repozitáři a začít pracovat říkají
  `nejdřív repozitář naklonujte`.

### Kontrola žádosti

**Zkontrolovat** (`e`) u žádosti z vlastních repozitářů rozsahu otevře kartu toho repozitáře s kontrolou
v ní. U žádosti z řádku za celý účet záleží cesta na tom, zda je znám klon, a na nastavení
profilu **Žádost z nenaklonovaného repozitáře** (Nastavení > Integrace, `review_mode`
v [referenční příručce konfigurace](config-reference.md)):

| Nastavení | Klon je znám | Žádný klon není znám |
|---|---|---|
| Zeptat se (výchozí) | zeptá se: **Otevřít místní klon** (`l`) nebo **Zkontrolovat přes API** (`a`) | zeptá se: **Naklonovat a zkontrolovat** (`c`) nebo **Zkontrolovat přes API** (`a`) |
| Naklonovat | otevře klon | otevře dialog Klonovat |
| Zkontrolovat přes API | zkontroluje přes API | zkontroluje přes API |

Otázka je malý dialog: písmena vybírají, **Enter** vybere zvýrazněný řádek, `Esc`, **Zrušit**,
`✕` a klik mimo ho zavřou a **Zapamatovat pro GitHub** (`r`) zapíše nastavení profilu, aby se
otázka už nekladla (řádek v Nastavení ho změní zpět). `Ctrl+N` označí jeho ovládací prvky.

- **Zkontrolovat přes API** otevře kontrolu v okně Domů, nad Radarem, s kódem převzatým
  od hostitele (viz [Kontrola bez klonu](review.md#kontrola-bez-klonu)). `Esc` nebo `✕` vrátí
  Radar; řádky palety *Kontrola: ...* na ní fungují.
- **Naklonovat a zkontrolovat** otevře dialog Klonovat na připojeném účtu, odkud řádek pochází (stejný seznam
  repozitářů, seskupený podle vlastníka), s vybraným repozitářem žádosti a vyplněnou výchozí složkou pro klony;
  změňte jedno nebo druhé a naklonujte, se stejným přihlášením jako při jakémkoli klonování z toho účtu. Pokud
  repozitář v seznamu účtu není (nejste jeho členem), seznam zůstane otevřený s hledáním jeho názvu a poznámka to
  oznámí; u hostitele, který repozitáře vypsat neumí, se dialog otevře na **Klonovat z URL** s vyplněnou adresou.
  Když klon dorazí, nový repozitář se propojí s hostitelem
  (jeho zdroj PULL REQUESTS a jeho zdroj ISSUES, má-li hostitel issues a žádný není nastaven), otevře se jeho karta
  a v ní kontrola dané žádosti. Zrušení dialogu nebo selhání klonu nepropojí nic.
- Žádosti Azure DevOps a Bitbucket nemají kontrolu v aplikaci: **Zkontrolovat** otevře stránku žádosti
  v prohlížeči a řekne to na stavovém řádku.

Řádky palety *Radar: Zkontrolovat přes API* a *Radar: Naklonovat a zkontrolovat...* otázku přeskočí.

Dvě zaškrtávací políčka vedle filtru (objeví se, jakmile se ptáme na seznam za celý účet), uložená v
`radar.toml`:

| Políčko | Klávesa | Výchozí | Účinek |
|---|---|---|---|
| `[x] Nenaklonované` | `a` | zapnuto | Vypíše řádky repozitářů bez známého klonu. Vypnuto zůstanou jen řádky známých klonů. |
| `[ ] Koncepty` | `d` | vypnuto | Vypíše koncepty žádostí z řádků za celý účet. Koncepty vlastních repozitářů rozsahu se vypisují vždy. |

Obě jsou na označení `Ctrl+N` a v paletě (`Radar: Zobrazit nenaklonované žádosti`, `Radar: Zobrazit koncepty`).

Odpověď profilu se uchovává v `radar-account.json` **5 minut**: když Radar otevřete znovu v této
době, ukáže řádky, aniž by se ptal hostitele. **Obnovit** (`r` nebo `F5`) se ptá vždy: každého repozitáře i každého seznamu celého účtu, ať je poslední odpověď jakkoli stará. Hostitel, který řekl, ať zpomalíme,
nebo odmítl token, se ve stejném kole znovu nedotazuje. Pro každý profil a druh se uchová nejvýše 300 řádků
a řádek pod **Repozitáře** říká, kolik jich bylo dalších, nebo proč se profil nepodařilo přečíst
(**Znovu připojit** otevře Nastavení > Integrace).

## Omezení

- Řádky za celý účet pocházejí z profilů GitHub, GitLab (včetně self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket Data Center a Gerrit (ne Bitbucket Cloud). Issues přiřazené vám pocházejí z prvních
  pěti; Bitbucket a Gerrit issues nemají.
- Seznamy jednotlivých repozitářů používají profily GitHub, GitLab (včetně self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket a Gerrit; kontroly, konflikty a zmínky se zjišťují jen pro první čtyři.
- Zmínky na GitLabu pocházejí jen z čekajících úkolů; kontroly na GitHubu, Gitea a Forgejo se zjišťují pro 15 žádostí na repozitář.
