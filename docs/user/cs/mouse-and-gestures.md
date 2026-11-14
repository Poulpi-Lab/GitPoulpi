[English](../mouse-and-gestures.md) · **Čeština** · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Myš a gesta

Vše, co lze dělat myší, má také klávesový ekvivalent. Viz
[Klávesové zkratky](keybindings.md). Názvy oblastí jsou v
[prohlídce rozhraní](interface-tour.md).

## Obecná pravidla

- **Kliknutím** na řádek ho vyberete. Funguje to v každém panelu, seznamu,
  nabídce a dialogu.
- **Dvojklikem** na řádek ho otevřete nebo spustíte. Dvojklik na lokální větev,
  tag nebo pilulku ref ji přepne (checkout). Dvojklik na řádek repozitáře
  repozitář otevře.
- **Kliknutím pravým tlačítkem** na řádek otevřete jeho nabídku. `.`,
  `Shift+F10` nebo klávesa Menu otevře stejnou nabídku pro vybraný řádek.
- **Kolečko** posouvá oblast pod ukazatelem. V seznamu posouvá jen zobrazení.
  Výběr zůstává, kde je, i když se posune mimo viditelnou část. Výběr přesune
  jen klik nebo klávesa.
- **Shift+kolečko** posouvá do stran, v diffu, v zobrazení Soubor a v editoru,
  když je zalamování řádků vypnuté.
- **Najetím** na tlačítko, řádek nebo jezdec posuvníku ho rozsvítíte. Mnoho
  terminálů neumí měnit ukazatel myši, takže toto rozsvícení je často jediným
  znamením, že na něco lze kliknout. Zastavením ukazatele na tlačítku nebo
  buňce Logu se zobrazí tooltip.
- **Kliknutí mimo** textové pole z něj odejde. To, co jste napsali, zůstane.
  Filtr Refs použije napsané, stejně jako `Enter`. Vyhledávací lišta Logu se
  zavře, stejně jako `Esc`.

## Zavírání oken a dialogů

Každé okno s rámem a každý dialog má vpravo nahoře na rámu `✕` a všude platí
stejná pravidla:

- **Kliknutím na `✕`** okno zavřete. Totéž dělá `Esc`, který nejdřív zavře
  nejvnitřnější věc (otevřený rozbalovací seznam, textové pole, potvrzení) a
  teprve pak okno.
- **Kliknutím mimo** dialog, panel nápovědy či informací nebo vyskakovací okno ho
  zavřete. Kliknutí mimo nikdy nezavře celé zobrazení (Nastavení, zobrazení Diff,
  zobrazení Soubor, editory, Kontrolu, Tabuli): to nemá žádné "mimo".
- Kliknutí **dovnitř** panelu ho nikdy nezavře.
- Formulář s textem, který jste napsali (nové issue, pull nebo merge request),
  kliknutí mimo ignoruje, aby o text nešlo přijít omylem. `✕` a `Esc` se nejdřív
  zeptají **Zahodit změny?**; **Upravovat dál** vás vrátí zpět. Formulář bez
  napsaného textu se zavře hned, stejně jako jednořádkový dialog.
- Paleta příkazů, výběry (karta, motiv, jazyk), malé dialogy Kontroly, Tabule,
  merge a editoru zpráv i potvrzovací výzva mají `✕` také. Kontextové nabídky a
  rozbalovací seznamy ho nemají: zavírají se klávesou `Esc` nebo kliknutím mimo.

## Seznamy v Nastavení

Seznamy v Nastavení (Profily, profily AI, Workflow, Otevřít v, Obrázky) ovládáte
myší takto. **Kliknutím na podkartu** (Obecné nebo seznam) na ni přepnete; totéž dělá `Alt+Left` / `Alt+Right`.

- **Kliknutím** na řádek ho vyberete, **dvojklikem** přejdete na jeho pole,
  **kolečkem** posouváte bez výběru.
- **Stiskněte řádek a táhněte** ho nahoru či dolů, abyste změnili pořadí:
  proužek ukazuje, kam dopadne, uvolnění ho pustí, `Esc` akci zruší. Klávesový
  protějšek je `Alt+Up` / `Alt+Down`.
- **Klikněte na `+ Přidat`, `Duplikovat`, `Odebrat`, `↑`, `↓`** pod seznamem.
  Klávesy: `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; viz
  [Nastavení](settings.md#seznamy).

## Seznamy ve vyskakovacích oknech

Tato pravidla platí pro každý seznam ve vyskakovacím okně s filtrovacím polem:
paleta příkazů, její výběry (motiv, jazyk, barvy grafu), výběr karty,
rozbalovací seznamy s vyhledáváním, seznam odkazů na issues a nápověda.

- **Kliknutím** na řádek ho vyberete.
- **Dvojklikem** na řádek ho spustíte nebo zvolíte, jako `Enter`.
- **Kolečko** posouvá seznam. Výběr nepřesouvá.
- Když je seznam delší než okno, na jeho pravém okraji se objeví **posuvník**.
  Táhněte jezdec nebo klikněte na dráhu.

Krátký rozbalovací seznam bez filtrovacího pole zvolí řádek prvním kliknutím.
Kontextová nabídka spustí řádek prvním kliknutím. Klávesy pro tyto seznamy jsou
v [Klávesových zkratkách](keybindings.md#filtrovatelné-vyskakovací-seznamy).

## Posuvníky

Každá oblast, která se může posouvat, ukazuje na svém okraji tenký posuvník,
když se její obsah nevejde: Log, sekce Refs, seznamy souborů, diff, zobrazení
Soubor, editor a seznamy ve vyskakovacích oknech.

- **Táhněte jezdec** pro posun.
- **Kliknutím na dráhu** přenesete jezdec pod ukazatel: zobrazení tam skočí
  úměrně a podržením tlačítka táhnete dál.
- **Držte tlačítko na koncové šipce** pro nepřetržité posouvání. Začne po krátké
  prodlevě. Odsunutím ze šipky pozastavíte.
- Posuvník posouvá jen zobrazení. Výběr nikdy nepřesouvá.

V diffu a v editoru svislý posuvník ukazuje také celý soubor v malých značkách:
zelená pro přidané řádky, červená pro odebrané, jantarová pro změněné, `◆` pro
řádek s kurzorem a `▌` pro každé další místo, kde se vybraný text vyskytuje.
Editor merge označuje každý konflikt na svých posuvnících. Kliknutím na značku
tam přejdete.

## Selhání a Protokol aktivity

- **Klikněte na červenou chybu** ve stavovém řádku a otevřete Protokol aktivity na jejím záznamu; **klikněte na Kopírovat / navržené tlačítko** (Zkusit znovu, Nejdřív Pull, Stash a zkusit znovu, Otevřít Integrace) na záznamu.
- **Klikněte na Aktivita** na pravém konci stavového řádku a otevřete Protokol aktivity. Kliknutím na kartu přepnete, kliknutím na zaškrtávací políčko úrovně (nebo **Vše** / **Nic**) tyto řádky zobrazíte či skryjete, kliknutím na záznam karty Repozitář otevřete jeho podrobnosti, kliknutím na **Vymazat protokol** na kartě Aplikace (nejdřív se zeptá) ho vyprázdníte. Dlouhé řádky posouvejte do stran přes `Shift` + kolečko nebo vodorovný posuvník. Viz [Protokol aktivity](troubleshooting.md#protokol-aktivity).
- **Klikněte na `[x]`** u ukazatele úloh vpravo ve stavovém řádku a zrušíte běžící úlohu, **klik na jeho popisek** otevře panel úloh. Tlačítko **Aktivita** vpravo dole na kartě Domů (nebo `Ctrl+N Shift+G`) otevře Protokol aktivity na volbě Všechny repozitáře (jeho pole Rozsah, rozbalovací seznam pod kartami, vybírá repozitář nebo pracovní prostor).

## Změna velikosti

- **Táhněte dělič** mezi postranním panelem Refs, Logem a pravým panelem. Klávesnice je posouvá po krocích: `Ctrl+N Left` / `Ctrl+N Right` posouvají dělič Refs | Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` dělič Log | pravý panel (dělič jde tam, kam šipka ukazuje).
  Přetažením za minimální šířku panel sbalíte. Klávesnicovými protějšky jsou `Ctrl+N Left` / `Ctrl+N Right` (dělič mezi panelem Refs a Logem) a `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (dělič mezi Logem a pravým panelem), každý o jeden krok, dělič jde tam, kam šipka ukazuje; paleta je má jako `Navigace: Rozšířit panel Refs`, `... Zúžit panel Refs`, `... Rozšířit panel Stav` a `... Zúžit panel Stav` a `Navigace: Sbalit/rozbalit panel Refs` / `... Sbalit/rozbalit panel Stav`.
- **Táhněte spodní okraj** sekce Refs pro změnu její výšky.
- **Táhněte spodní okraj** bloku zprávy commitu v pravém panelu.
- **Táhněte okraj mezi dvěma záhlavími Logu** (ve výchozím stavu mezeru, ze záhlaví nebo odkudkoli z tabulky; Nastavení > Log a graf > Změna velikosti sloupců) pro změnu šířky sloupce (z klávesnice: `Shift+C` vybere sloupec, pak `]` / `[`).
- **Dvojklik na okraj mezi dvěma záhlavími Logu** přizpůsobí sloupec nalevo jeho obsahu (z klávesnice: `Shift+C` pro výběr sloupce, pak `=`; `+` přizpůsobí všechny sloupce). Poslední sloupec má úchyt na své poslední buňce: tažením změníte jeho šířku, rozdíl převezme sloupec Zpráva.
- **Posouvání Logu do stran** (rozložení sloupců `scroll`; Nastavení > Log a graf > Rozložení sloupců, nebo zaškrtávátko `Posouvat do stran` ve výběru sloupců): vodorovné kolečko nebo `Shift`+kolečko nad tabulkou, lišta na spodním okraji (tažením jezdce, kliknutím na prázdnou dráhu, kliknutím na koncové šipky), nebo `Shift+Left` / `Shift+Right`. Ve výchozím rozložení `fit` sloupce vyplní panel a není co posouvat.
- **Táhněte záhlaví Logu** pro přesun sloupce: duch jeho názvu sleduje ukazatel, záhlaví, které opustil, je ztlumené a akcentní proužek označuje místo, kam by dopadl. Sloupce se přesunou při pustění; `Esc` to zruší a ukazatel během přenášení ukazuje otevřenou ruku (z klávesnice: `Shift+C` vybere sloupec, pak `<` / `>`).

GitPoulpi si pamatuje každou velikost. Viz
[Velikosti se pamatují](interface-tour.md#velikosti-se-pamatují).

## Záhlaví okna, řádek karet a panel nástrojů

- **Klikněte na 🐙** v záhlaví okna pro otevření **O aplikaci GitPoulpi**. V tomto okně
  sledují zorničky chobotnice ukazatel; kliknutím na logo ho roztočíte do další barvy,
  pravým tlačítkem na logo otevřete seznam barev, kliknutím na odkaz ho otevřete.
- **Klikněte na Otevřít v ▾** v záhlaví okna pro seznam programů. Viz
  [Otevřít v](open-in.md).
- **Klikněte na verzi** (`v1.0.0`, nebo `vX.Y.Z k dispozici`) v záhlaví okna pro otevření karty Poznámky k vydání.
- **Klikněte na lupu a `Globální hledání`** v záhlaví okna, těsně vlevo od ozubeného kola, pro
  [globální hledání](search.md).
- **Klikněte na ozubené kolo** v záhlaví okna, vlevo od ×, pro Nastavení.
- **Klikněte na ×** na pravém konci záhlaví okna pro ukončení.
- **Klikněte na kartu** pro přepnutí na ni. **Klikněte na její `x`** (ukazuje se
  na aktivní kartě a na kartě pod ukazatelem) pro zavření. **Přetáhněte kartu**
  do strany a změníte její pořadí. **Pravým tlačítkem na kartu** otevřete nabídku karty.
- **Prostřední tlačítko na kartě** ji zavře, jako její `x`.
- **Kliknutí na `‹` nebo `›`** na koncích karet (zobrazí se, když se nevejdou všechny) posune řádek o jednu kartu; podržením tlačítka posouváte dál, nebo použijte kolečko nad řádkem. Přetažení karty na šipku řádek také posouvá. Řádek se také sám posune, aby ukázal aktuální kartu, kdykoli se změní.
- **Klikněte na `+`** za poslední kartou pro nabídku: Nová karta (`Alt+T`) a, dokud jsou jejich karty zavřené, Radar a Pracovní prostory (`Alt+J`).
- **Klikněte na ikonu domu** pro kartu Domů.
- **Klikněte na kartu Radar nebo Pracovní prostory** pro její otevření; zavře ji její `x`, prostřední tlačítko nebo `Ctrl+W`. **Klikněte na čip pracovního prostoru** (`Alt+W`) pro přepnutí pracovního prostoru.
- **Klikněte na Konec a cd sem** (zobrazí se vpravo od karet, když jste GitPoulpi spustili přes obálku shellu `gp`) pro ukončení a ponechání shellu ve složce aktivní karty. Viz [Sledování složky aktivní karty](following-the-tab-folder.md).
- **Klikněte na tlačítko panelu nástrojů** pro jeho spuštění. **Klikněte na
  `▾`** vedle Pull, Push nebo LFS pro jeho seznam. **Klikněte na `⋯`** pro
  tlačítka, která se nevešla.
- **Klikněte na combobox profilu** na pravém konci panelu nástrojů pro výběr
  identity commitů pro tento repozitář.

## Karta Domů

- **Kliknutím** na řádek ho vyberete. **Dvojklikem** repozitář otevřete.
- **Klikněte na hvězdičku** pro přidání nebo odebrání oblíbeného.
- **Klikněte na Otevřít, Přepnout, Zavřít nebo Odebrat** na řádku, nebo na
  **`⋮`** pro nabídku řádku. **Pravým tlačítkem** na řádek otevřete stejnou
  nabídku.
- **Klikněte na název sekce** pro její sbalení.
- **Klikněte na Procházet, Klonovat, Init, Sbalit vše nebo Rozbalit vše**
  nahoře a na **Zavřít všechny karty** nebo **Odebrat vše** v názvu sekce.

Ve výběru složky **Procházet**: klikněte na místo nebo složku pro přechod
tam, dvojklikem na složku ji otevřete, kliknutím na pole cesty napíšete cestu.
Zaškrtněte **Zobrazit sousední složky** (`Alt+S`, pamatuje se) a levý strom uvádí i
ostatní složky každé otevřené úrovně: složky na otevřené cestě mají `▾`, ostatní
`▸` a kliknutí na některou vás tam přenese. V **Klonovat** a **Init**: kliknutím na pole do něj píšete a kliknutím na
tlačítko nebo zaškrtávací políčko je použijete.

## Postranní panel Refs

- **Kliknutím** na řádek ho vyberete. **Dvojklikem** na lokální větev nebo tag
  ji přepnete (checkout). **Dvojklikem** na worktree nebo submodul ho otevřete
  na kartě. Tlačítko **+** v záhlaví WORKTREES worktree vytvoří; pravým
  tlačítkem na worktree získáte Otevřít na nové kartě, Otevřít v, Merge a Odebrat ([Worktrees](worktrees.md)).
- **Najeďte** na řádek, aby se vlevo zobrazila ikona. **Kliknutím** na ni ref
  v Logu skryjete, nebo znovu zobrazíte. Při zapnutém **Solo** na stejném
  místě dáte ref do solo. Na řádku složky ikona působí na každý ref ve složce.
- **Klikněte na Ukončit solo** nebo **Zobrazit vše** v názvu postranního panelu
  pro zrušení všech solo nebo všech skrytých refs.
- **Pravým tlačítkem** na řádek otevřete jeho nabídku: checkout, merge,
  fast-forward, rebase, vytvoření worktree, přejmenování, smazání, skrytí, solo
  a další. Řádek samotného remote má **Fetch**, **Upravit**, **Odebrat** a další
  akce remote. Stash má **Pop**, **Apply** a **Drop**. Zašedlý řádek říká, proč
  nelze spustit.
- **Klikněte na `+`** v názvu REMOTE pro přidání remote.
- **Klikněte na řádek složky** pro její sbalení nebo rozbalení.

### Pull requesty a issues

| Gesto | Akce |
|---|---|
| Klik na řádek | Vybere ho. |
| Dvojklik na řádek | Otevře ho v prohlížeči. |
| Pravé tlačítko na řádek | Jeho nabídka: otevřít, kopírovat URL nebo číslo, checkout zdrojové větve, vytvořit větev z issue. |
| Klik na `+` v názvu sekce | Otevře formulář **Vytvořit pull request** nebo **Nová issue**. |
| Klik na řádek filtru (`Open ▾`) | Zvolí, které požadavky nebo issues se vypíší. |
| Klik na název skupiny | Sbalí nebo rozbalí skupinu. |
| Klik na řádek poskytovatele nebo repozitáře (ISSUES) | Zvolí zdroj issues. **Žádný (odpojit tento repozitář)** ho odebere. |
| Klik na řádek dotazu (ISSUES) | Napíšete dotaz JQL nebo WIQL (Jira, Azure DevOps), nebo zvolíte seznamy Trello. |
| Klik na čip panelu nástrojů (`GH #12`, `GL !53`, ...) | Otevře pull request v prohlížeči. Pravým tlačítkem na něj otevřete **Otevřít v prohlížeči**, **Kopírovat URL**, **Obnovit** a **Přepnout profil**. |
| Klik na tlumený čip `+ PR` | Založí pull request pro odeslanou větev. |

Viz [Hostingové služby a issue trackery](hosting-integration.md).

## Log

- **Kliknutím** na commit ho vyberete. Jeho detaily se zobrazí v pravém panelu.
- **Ctrl+klik** přidá commit do výběru nebo ho odebere.
  **Shift+klik** vybere rozsah (**Alt+klik** v terminálech, které si Shift+klik
  ponechávají pro vlastní výběr textu, například kitty a Windows Terminal). Pravým tlačítkem
  na vybraný commit provedete cherry-pick všech. Výběr může zahrnovat řádek WIP a stashe;
  dva a více řádků změní pravý panel na porovnání, viz
  [Porovnání commitů](log-and-history.md#porovnání-commitů).
- **Kliknutím na dlaždici** panelu porovnání přesunete kurzor Logu na ten řádek;
  **kliknutím na Prohodit** vyměníte konce Od a Do. Prosté kliknutí na řádek porovnání opustí.
- **Klikněte na řádek WIP** pro pracovní změny. **Klikněte na jeho text `// WIP`**
  a napíšete tam souhrn commitu.
- **Najeďte** na buňku pro úplný text: zprávu, autora a e-mail, úplné datum,
  úplný hash.
- **Najeďte na odznak `+N`** pro další refs na commitu. Přesuňte se do
  vyskakovacího okna a klikněte tam na ref. Dvojklikem na ref ve vyskakovacím
  okně ho přepnete (checkout).
- **Dvojklikem na pilulku ref** ji přepnete (checkout).
- **Pravým tlačítkem na commit** otevřete nabídku commitu. Má jednu podnabídku
  pro každý ref na commitu.
- **Pravým tlačítkem na pilulku ref** otevřete jednu nabídku s akcemi ref
  a akcemi commitu.
- **Pravým tlačítkem na záhlaví** zobrazíte nebo skryjete sloupce: zaškrtávací seznam (`[x]` / `[ ]`) s ✕ v titulku; klikněte na řádek nebo použijte šipky a `Mezerník` / `Enter`, pak ho zavřete ✕, `Esc` nebo kliknutím mimo (paleta: `Zobrazení: Log: Vybrat zobrazené sloupce`). Pod zaškrtávátky: `Přizpůsobit <sloupec> obsahu` (sloupec, na který jste klikli pravým), `Přizpůsobit všechny sloupce obsahu` a `Obnovit šířky sloupců`. Klávesnicový protějšek pravého kliku: `Shift+C` vybere sloupec, pak `.` (nebo `Shift+F10`) otevře stejnou nabídku na něm.
- **Sloupce Radaru** fungují stejně: táhněte oddělovač za sloupcem repozitáře, `#`, autora nebo stáří pro změnu šířky, **dvojklikem na něj** sloupec přizpůsobíte jeho nejširší položce (`C` vybere sloupec, pak `=`; `+` přizpůsobí všechny sloupce) a **pravým tlačítkem na název sloupce** (nebo `C`, pak `.`) otevřete `Přizpůsobit <sloupec> obsahu` a `Přizpůsobit všechny sloupce obsahu`.
- **Klikněte na `↑`, `↓` nebo `✕`** ve vyhledávací liště pro přechod na
  předchozí nebo další shodu, nebo pro zavření lišty.
- **Klikněte na `#12`, `ABC-12` nebo `AB#12`** ve zprávě commitu v pravém panelu
  pro otevření této issue nebo pull requestu v prohlížeči.

### Sekce levého panelu

**Složky** (větve a tagy pojmenované `a/b/c`, soubory pravého panelu ve stromovém
zobrazení) se sbalují a rozbalují najednou: **pravé tlačítko na záhlaví nebo prázdném
místě sekce**, která složky má, nabídne **Rozbalit vše** / **Sbalit vše** (a
**Zobrazit/skrýt sekce...**, což vede k seznamu níže) a **pravé tlačítko na řádku
složky** nabídne **Rozbalit všechny podsložky** / **Sbalit všechny podsložky**
(složka a vše pod ní). Klávesy, stejně v panelu Refs, v seznamech Unstaged a Staged
i v souborech změněných commitem: `=` rozbalí vše, `-` sbalí vše, `*` (nebo
`Shift+Right`) rozbalí složku pod kurzorem a vše pod ní, `_` (nebo `Shift+Left`) ji
sbalí. Paleta: *Refs: Rozbalit všechny složky*, *Refs: Sbalit všechny složky*,
*Refs: Rozbalit vybranou složku a její podsložky*, *Refs: Sbalit vybranou složku a
její podsložky* a čtyři dvojčata *Soubory:* pro seznam souborů s fokusem. Dokud je
aktivní filtr názvů (filtr Refs, *Filtrovat soubory* commitu), je každá složka s
nálezem zobrazena rozbalená, ať jste sbalili cokoli, a vaše vlastní sbalení se
nemění ani neukládá: zrušte filtr a vrátí se. Sbalení provedené během filtru platí
pro to, co je zobrazeno, a s filtrem se zapomene.

**Pravé tlačítko na záhlaví sekce** (sekce bez složek, nebo prázdná část levého panelu) otevře seznam jeho
sekcí: Lokální, Remote, Worktrees, Stashes, Tags, Pull requests, Issues a Submoduly. Kliknutí sekci odškrtne
nebo zaškrtne; nabídka zůstane otevřená. Skrytá sekce se nekreslí a klávesnice ji přeskakuje. Poslední
viditelnou sekci odškrtnout nelze. Volba se pamatuje pro každý repozitář zvlášť, i po restartu; repozitář bez vlastní volby se řídí výchozím nastavením (globální `ui.sidebar_sections_hidden`). *Použít pro všechny repozitáře* udělá z aktuální volby výchozí a *Obnovit výchozí* zahodí vlastní volbu tohoto repozitáře (paleta: *Levý panel: Použít tyto sekce pro všechny repozitáře*). Paleta: *Levý panel: Zobrazit/skrýt sekce...*,
po jednom příkazu *Zobrazit/skrýt* pro každou sekci, *Obnovit výchozí sekce*.

### Přetahování refs

Stiskněte tlačítko na větvi v postranním panelu Refs, nebo na pilulce ref v Logu,
a přetáhněte ji na jinou větev. Stavový řádek říká, co nesete. Cíl, který může drop
přijmout, se rozsvítí. Uvolněním získáte nabídku. `Esc` přetažení zruší.

| Přetažení | Nabídka |
|---|---|
| Lokální větev X na lokální větev Y | **Merge X do Y**, **Rebase X na Y**, **Interaktivní rebase X na Y...**, **Fast-forward Y na X** |
| Lokální větev na vzdálenou větev | **Push X** do tohoto remote |
| Vzdálená větev na lokální větev Y | **Merge**, **Rebase Y na ni**, **Fast-forward Y na ni** a **Pull**, když je to upstream větve Y |

Když akce mění větev, která není přepnutá, GitPoulpi ji nejdřív přepne (checkout).
Pokud git checkout odmítne, nic se nezmění. Tagy nelze přetahovat a nemohou
drop přijmout.

Klávesnice: „označit a pustit“ udělá totéž co tažení, pro libovolnou dvojici refs.
V panelu Refs stiskněte `m` na větvi (paleta: `Refs: Označit jako zdroj přetažení`).
Ref ukáže `◆` a je podtržený, také na svých štítcích v Logu, a stavový řádek řekne,
kam jít. Vyberte jiný ref v libovolné sekci a stiskněte `Shift+M` (paleta:
`Refs: Pustit sem označený ref...`): objeví se stejná nabídka jako po tažení a její
řádky se provedou přesně jako po tažení. Značka se spotřebuje, jakmile se nabídka
otevře; dřív ji zruší `Esc`, nebo `m` na označeném refu. Příkaz palety
`Refs: Pustit vybraný ref na přepnutou větev...` otevře nabídku pro vybraný ref
a aktuální větev.

## Pravý panel

### Pracovní změny

- **Kliknutím** na soubor otevřete jeho diff. **Kliknutím na stejný soubor
  znovu** se vrátíte do Logu. Pokud je soubor otevřený v editoru s neuloženými
  změnami, GitPoulpi se nejdřív zeptá.
- **Najeďte** na řádek, aby se zobrazilo jeho tlačítko **Stage soubor** nebo
  **Unstage soubor**. Řádek složky má **Stage složku** nebo **Unstage složku**.
- **Klikněte na Stage všechny změny** nebo **Unstage všechny změny** v názvu
  sekce.
- **Klikněte na název sekce** (`Soubory unstaged`, `Soubory staged`) pro její
  sbalení.
- **Klikněte na Cesta nebo Strom** pro plochý seznam nebo složky. **Klikněte na
  Sbalit vše** nebo **Rozbalit vše** pro sbalení všech složek.
- **Klikněte na `✖`** vlevo od názvu pro zahození všech změn (nejdřív se
  zeptá). **Klikněte na `✦`** vpravo pro vysvětlení změn pomocí AI.
- **Pravým tlačítkem na soubor** otevřete jeho nabídku: stage, zahodit, ignorovat,
  LFS, stash souboru, historie, blame, otevřít v externím programu, kopírovat
  cestu, vytvořit patch, upravit, smazat. Soubor v konfliktu začíná řádky
  **Vyřešit pomocí ...**.
- **Pravým tlačítkem na řádek složky** otevřete nabídku složky: stage, zahodit,
  ignorovat, stash, vytvořit patch, otevřít složku, vytvořit v ní soubor.
- **Pravým tlačítkem na prázdnou část** Souborů unstaged otevřete **Vytvořit soubor**.
- **Ignorovat**, **Považovat za binární** a **Považovat za textový** (nabídky souboru a složky) nejdřív otevřou seznam souborů `.gitignore` nebo `.gitattributes`, do nichž pravidlo může patřit. Klikněte na řádek a stiskněte `Enter` (nebo použijte šipky); `Esc` nebo kliknutí mimo zruší bez zápisu.
- **Ctrl+klik** na řádek souboru nebo složky ho přidá do výběru nebo odebere, **Shift+klik** (**Alt+klik**) vybere rozsah; stejně ve změněných souborech commitu a v Zobrazit všechny soubory. Obyčejný klik začne znovu. **Pravý klik na vybraném řádku**: nabídka všech vybraných souborů.
- **Pravý klik na prázdné části** seznamu nabízí také **Otevřít složku repozitáře**; Unstaged přidává **Stage všechny změny** a **Zahodit všechny změny...**, Staged přidává **Unstage všechny změny**.
- **Převést** (nabídky souboru, složky a výběru) přepíše kódování, konce řádků nebo odsazení souborů na disku; jedno `Ctrl+Z` vrátí celou dávku.

### Formulář commitu

- **Klikněte** na Souhrn nebo Popis a pište do něj.
- **Klikněte na Amend předchozího commitu** pro zapnutí nebo vypnutí amend.
- **Klikněte na Commit změn v N souborech** pro commit. Když je tlačítko
  zašedlé, kliknutí řekne proč (například *Stage změny, ať můžete commitnout*).
- **Klikněte na `✦`** vpravo od Souhrnu pro napsání zprávy pomocí AI.
- **Klikněte na `#▾`** na konci Souhrnu pro seznam issues. Kliknutím na jednu
  vložíte její odkaz.
- **Klikněte na Sestavit commity pomocí AI** pro otevření commit composeru.

### Detaily commitu

- **Kliknutím** na změněný soubor otevřete jeho diff. **Kliknutím znovu** se
  vrátíte do Logu.
- **Táhněte** spodní okraj zprávy pro změnu její velikosti. Kolečko nad zprávou
  posouvá dlouhý popis.
- **Pravý klik** na souhrn, popis nebo blok autora nabídne *Upravit souhrn...*,
  *Upravit popis...* nebo *Upravit autora...*; **dvojklik** na jeden z nich (nebo na
  datum vytvoření) otevře celý dialog Úprava commitu s kurzorem v tom poli. Klávesové
  ekvivalenty jsou `Ctrl+G Shift+E` (všechna pole) a `Ctrl+G Shift+A` (autor); souhrn a popis jsou v paletě.
- **Pravý klik** na změněný soubor, řádek složky, prázdnou část seznamu nebo výběr otevře nabídku (otevřít, historie, blame, otevřít v programu, kopírovat cesty, ignorovat, brát jako binární nebo textový, převést, obnovit z commitu, smazat). **Ctrl+klik** a **Shift+klik** vyberou více souborů.

Viz [Stagování a commitování](staging-and-committing.md).

## Diff a zobrazení Soubor

- **Klikněte na tlačítko** v liště nad diffem: Soubor, Diff, Blame,
  přepínač sloupce `Historie │ Vše`, Upravit, `↑` `↓`, Sjednocené, Vedle sebe, Hunky, WS, Zalom., Mapa, Kódování. Zapnuté tlačítko je
  zvýrazněné. Kliknutí na `Historie` nebo `Vše` ukáže ten seznam ve sloupci
  commitů; kliknutí na už vybraný segment sloupec skryje (viz
  [Sloupec commitů](interface-tour.md#sloupec-commitů)).
- **Klikněte na `✕`** v názvu pro zavření souboru.
- **Najeďte na hunk** pro zobrazení **Stage hunk** nebo **Unstage hunk** a
  **Zahodit hunk**.
- **Najeďte na změněný řádek** pro zobrazení jeho vlastních tlačítek **+ Stage**,
  **− Unstage** a **✖ Zahodit** na pravém konci řádku.
- **Pravým tlačítkem na řádek** otevřete nabídku hunku: stage, unstage nebo
  zahodit hunk či vybrané řádky, historie souboru, blame, kopírovat.
- **Klikněte na minimapu** pro přesun zobrazení na daný řádek. Podržte tlačítko
  a pohybem posouvejte.

Každé zahození se nejdřív zeptá. Když zahození smaže nesledované soubory, odstraní se s nimi i složky, které se tím úplně vyprázdní (nikdy složka, která stále drží sledovaný, ignorovaný nebo skrytý soubor, ani složka, která byla prázdná už předtím); Zpět vrátí soubory i s jejich složkami.

### Blame a historie souboru

- **Blame.** Nechte ukazatel nad anotací pro kartu hunku; klik vybere hunk, pravý klik otevře jeho nabídku. Přetažením okraje sloupce anotací změníte jeho šířku (uloží se). Kliknutím nebo tažením po minimapě posouváte.
- **Historie souboru.** Klik vybere commit; `Shift+klik` vybere rozsah k porovnání; dvojklik ukáže commit v Logu; pravý klik otevře nabídku řádku; přetažením okraje mezi seznamem a diffem změníte šířku; ✕ na seznamu ho skryje (kliknutím na proužek `›` ho vrátíte); kliknutím na `Soubor` / `Diff` / `Blame` v liště přepnete, co ukazuje pravá strana, a přepínačem `Historie │ Vše` vyberete seznam sloupce. Kolečko posouvá polovinu pod ukazatelem. Každé má klávesový ekvivalent ([Blame](log-and-history.md#blame), [Historie souboru](log-and-history.md#historie-souboru)).

### Sloupec commitů (seznam Vše)

- **Najeďte na řádek** pro tooltip Logu (celá zpráva, jméno a e-mail autora, úplné datum, podle části pod ukazatelem).
- **Pravým tlačítkem na řádek** otevřete jeho nabídku: zobrazit ho v Logu, diff souboru v tom commitu, soubor v tom commitu, jeho blame, pak nabídku Logu pro ten commit. Klávesa nabídky dělá totéž z klávesnice.
- **Táhněte hranici v záhlaví** (`ui.column_resize`: ve výchozím stavu jednobuňkový oddělovač mezi dvěma částmi, v záhlaví označený tlumeným `│` a při najetí ukazatele vykreslený jako akcentní čára přes celý sloupec, tažení odkudkoli z ní mění šířku; s `header` první písmeno dalšího názvu, rozsvícené, když je na něm ukazatel) pro změnu šířky části vlevo od ní: dráhy, zpráva, autor, datum, refs, změny nebo SHA. Buňka hned za poslední částí je hranicí poslední části.
- **Dvojklik na hranici v záhlaví** přizpůsobí část vlevo od ní jejímu obsahu, stejně jako hranice Logu: šířka, která celý ukáže nejširší řádek, nikdy širší než sloupec (z klávesnice: klikněte na řádek nebo `Ctrl+N`, aby sloupec dostal klávesy, `Tab` vybere část, pak `=`; `+` přizpůsobí všechny části; paleta má `Přizpůsobit sloupec obsahu` a `Přizpůsobit všechny sloupce obsahu`). Zpráva si ponechá, co zbude ostatním částem.
- **Táhněte název v záhlaví** přes jiný pro přesun části na jeho místo: stejný duch, ztlumený zdroj a akcentní proužek jako v záhlaví Logu a část se přesune při pustění (`Esc` to zruší; `<` / `>` na klávesnici, po `Tab`, totéž o jedno místo).
- **Klikněte pravým tlačítkem na záhlaví** a vyberte, které části se zobrazí (Ref, Graf, Autor, Datum, Změny, SHA; Zpráva je vždy zobrazena); zaškrtávací seznam má ✕ v titulku. Pod ním: `Přizpůsobit <část> obsahu` (část, na kterou jste klikli pravým) a `Přizpůsobit všechny sloupce obsahu`. Paleta má `vybrat sloupce` (nabídka z klávesnice, na části vybrané `Tab`) a přepínač pro každou část.
- **Shift+kolečko** nebo vodorovné kolečko posouvá řádky do stran, když jsou širší než sloupec.
- **Táhněte pravý okraj** sloupce pro změnu jeho šířky. **Klikněte na proužek `›`** (nebo na ✕ na sloupci, nebo **na vybraný segment** přepínače `Historie │ Vše` v liště) pro jeho vrácení nebo skrytí.

### Výběr a kopírování textu

Můžete vybírat a kopírovat text diffu, zobrazení Soubor, Blame a Historie souboru.

- **Stiskněte a táhněte** pro výběr textu. Výběr zůstává na stejném textu, když
  posouváte.
- **Dvojklikem** vyberete slovo.
- **Prostý klik** text nevybere. Přesune kurzor nebo vybere řádek a zruší výběr
  textu.
- **`y` nebo `Ctrl+C`** zkopíruje výběr. Bez výběru zkopírují aktuální řádek.
- **Pravým tlačítkem** v diffu otevřete **Kopírovat výběr**, **Kopírovat řádek**
  a **Kopírovat hunk**.

GitPoulpi kopíruje jen text souboru. Čísla řádků, značky `+` a `-`, záhlaví
hunků a sloupce Blame se nekopírují. V rozvržení Vedle sebe zůstává výběr na straně, kde jste
tažení začali.

V diffu tažení také vybere řádky, které protíná, jako `v`. `Enter` pak provede
stage těchto řádků. `Esc` nejdřív zruší výběr textu, pak výběr řádků, pak
zavře diff.

Výběr textu také podbarví každé další místo, kde se tento text v diffu
a v editoru vyskytuje.

## Textová pole

Platí pro každé textové pole: filtry, vyhledávací lištu, paletu, Souhrn
a Popis, pole dialogů a hodnoty v Nastavení.

- **Kliknutím** tam umístíte kurzor.
- **Stiskněte a táhněte** pro výběr textu.
- **Dvojklikem** vyberete slovo.
- Psaním nahradíte výběr. `Backspace` nebo `Delete` ho odstraní.

Klávesy pro textová pole jsou v [Klávesových zkratkách](keybindings.md).

## Editor

- **Kliknutím** tam umístíte kurzor. **Shift+klik** rozšíří výběr.
- **Tažením** vybíráte. Tažením za horní nebo dolní okraj posouváte.
- **Dvojklikem** vyberete slovo.
- **Kolečko** posouvá bez přesunu kurzoru.
- **Klikněte na Uložit, Zpět, Znovu nebo Hledat** v liště. V režimu hledání
  **`↑`** a **`↓`** přecházejí na předchozí a další shodu.
- **Klikněte na `✕`** pro zavření. GitPoulpi se nejdřív zeptá, pokud jsou
  neuložené změny.

## Obrázky

Myš v zobrazení obrázků viz [Obrázky](image-view.md).

## Přesun a změna velikosti dialogů

Dialogy lze přesouvat a větším oknům měnit velikost:

- **Přetažením záhlaví** dialog přesunete (zavírací křížek si nechává své kliknutí). Paleta příkazů a výběry nemají samostatné záhlaví: táhněte za horní okraj.
- **Přetažením levého, pravého nebo spodního okraje**, nebo spodního rohu (úchyt `◢`), změníte velikost okna. Dialog zůstává na obrazovce a nikdy není menší než své minimum (60 x 16 u Historie vrácení, u ostatních menší). Okno s upravenou velikostí využije místo: textová pole a seznamy rostou, text se zalamuje na novou šířku.
- Tažení pokračuje, i když ukazatel opustí dialog, a uvolnění mimo dialog ho nikdy nezavře.
- **Přetažením dělicích čar** uvnitř zobrazení Reflog / Ztracená práce v Historii vrácení změníte jejich velikost; dvojklik na dělicí čáru ji obnoví.
- Klávesnicové protějšky jsou `Ctrl+N` a pak `m` (šipky přesouvají, `Shift`+šipky mění velikost, `0` obnoví výchozí, `Enter` nebo `Esc` režim opustí) a `Alt`+šipky pro dělicí čáry; paleta má `Dialog: Přesunout nebo změnit velikost` a `Dialog: Obnovit pozici a velikost`. Působí na dialog nahoře. `Ctrl+N` funguje stejně nad každou plochou: zobrazí odznak na ovládacích prvcích okna, dialogu či vyskakovacího okna v popředí (pole, seznamy, karty, tlačítka a zavírací křížek), klávesa po něm stiskne daný prvek, `m` zahájí režim uspořádání a `Esc` ho zruší; stavový řádek uvádí `<odznak>`, `m=uspořádat` a `Esc`. Platí to i pro paletu příkazů, výběry, hledání pracovního prostoru a nápovědu, kde `Ctrl+N` už neposouvá seznam dolů. `Alt+G` (`palette.arrange`) je další způsob, platný nad každým dialogem a vyskakovacím oknem, jak uspořádat okno v popředí bez předpony.
- **Jedno nastavení** (`ui.dialog_geometry`, Nastavení > Vzhled > Rozvržení) platí pro všechny dialogy: každý si pamatuje své místo a velikost pro tuto relaci, na disku pro příští spuštění, nebo vůbec.

| Dialog | Přesun | Změna velikosti |
|---|---|---|
| Historie vrácení / Reflog / Ztracená práce, Log aktivity, panel úloh, výběr složky | ano | ano |
| Paleta příkazů, výběry karty / motivu / jazyka, hledání pracovního prostoru | ano | ano (seznam roste s výškou, řádky se šířkou) |
| Klonování (seznam repozitářů), Pull request a Nové issue (popis), "Co se odesílá" (panel), Mermaid (náhled), možnosti kontroly (pole, jen šířka) | ano | ano |
| Editory dlouhých textů v Nastavení (šablony, háčky, prompty AI), pole AI ve formuláři pracovního prostoru | ano | ano (textové pole) |
| Přepis historie: dva pomocné dialogy AI | ano | ano (popis, panel odesílaného) |
| O aplikaci, panel aktualizací, bezpečnostní kontrola, předpovězené konflikty, "Nad limitem AI", smazání tagu, citlivé soubory | ano | ne (velikost se řídí textem) |
| Init, Worktree, Sparse checkout, dialog Flow, dialogy toku issue, formulář zprávy commitu (reword), formulář pracovního prostoru, výběry štítků / milníků | ano | ne |
| Potvrzení (každé "Jste si jistí?") | ano | ne |
| Dialogy Kontroly a Tabule: jejich malé dialogy (komentář, dokončení, výběr, sloupce) | ano | ne |
| Vyskakovací okna stránky Klávesové zkratky (potvrzení obnovení, zachycení klávesy), vyskakovací okno odpovědi AI | ano | ne |
| Nastavení: výsledky a potvrzení ověření repozitáře | ano | ne |
| Přepis historie: dialogy purge, identity a otázek | ano | ne |
| Vyskakovací okno přesunu souboru v editoru commitu, upozornění merge | ano | ne |

Nelze přesouvat: celostránkové obrazovky (Nastavení, Nápověda, stránky Kontroly a Tabule, které už vyplňují okno), kontextové nabídky a rozbalovací seznamy (visí na tom, co je otevřelo), popisky a malá vyskakovací okna uvnitř stránky (formulář vzdálených repozitářů, editor rozdělení commitu).

Ukazatel ukazuje tvar přesunu nad záhlavím a tvar změny velikosti nad okraji a rohy (terminály s `OSC 22`).

## Tvar ukazatele myši

Některé terminály nechají GitPoulpi měnit ukazatel myši: ukazatel změny
velikosti nad děliči, nad hranicí sloupce v Logu, v Radaru a ve sloupci commitů
(i při jejím tažení), ručičku nad položkami, na které lze kliknout. Funguje to
v kitty, foot, Ghostty, mintty a sestavení WezTerm s upstream PR #8092 (sekvence
`OSC 22`). Posílá se jen při změně tvaru, uvnitř tmux jen zabalená pro průchod
(`allow-passthrough on`; kitty a foot ji ukážou, Ghostty ne) a nikdy v screen,
zellij nebo herdr. Při ukončení GitPoulpi se obnoví výchozí ukazatel. V jiných
terminálech (Rio, Alacritty, Tabby, iTerm2, Terminal.app, Windows Terminal, xterm)
se ukazatel nemění; viz [Výběr terminálu](terminals.md#tvar-ukazatele-myši-osc-22).
Dělič nebo tlačítko pod ukazatelem se rozsvítí v každém terminálu.

Stejný druh komponenty dává všude v aplikaci stejný ukazatel.

- **Tlačítka jsou skutečná tlačítka**, v Nastavení (Integrace: Upravit, Odpojit,
  Odebrat, Procházet, Smazat, tlačítka úryvků v editoru instrukcí...) stejně jako
  v dialozích: odsazené popisky na vyplněném pozadí, které se pod ukazatelem
  zesvětlí, při zakázání ztlumené, nikdy text v `[ ]`. Jediné zbylé závorky jsou
  značky zaškrtávátek `[x]` / `[ ]`, což jsou zaškrtávátka a berou si ruku jako
  každý jiný ovládací prvek.
- **Ukazující ruka** je na ovládacích prvcích: tlačítka, zaškrtávací políčka a
  přepínače, comboboxy (v Nastavení, v panelech issues a pull requestů, v Radaru,
  v dialozích), karty a jejich křížky, tlačítka `+`, odkazy a reference (`#12`,
  `!45`), hlavičky skupin a složek s jejich šipkou rozbalení, pilulky ref,
  posuvníky a tlačítka Aktivita, úlohy a verze ve stavovém řádku. Zaškrtávací
  políčko nebo tlačítko uvnitř řádku si ruku ponechává.
- **Šipka** je na řádcích. Řádek se rozsvítí celý, když je na něm ukazatel (stejným
  světlem v každém seznamu), a sám o sobě ovládacím prvkem není: řádky panelů Refs,
  Worktrees, Stashes a Submodules, Log a sloupec commitů (ruku bere jen pilulka
  ref), řádky diffu, změněné soubory, položky pull requestů, issues a Radaru a řádky
  seznamů v dialozích a panelech (klonování, výběr složky, Workspaces, karty
  tabule, panel Jobs, výběry přiřazených osob, štítků a milníků, kroky rebase,
  soubory a commity revize, volby nabídek a comboboxů).
- **Textový kurzor (I-beam)** je na vstupních polích, přes celé pole: vyhledávací
  pole (globální hledání, Nastavení, paleta příkazů, Home), filtry (Refs, soubory,
  Radar, Log aktivity, hledání v panelu issues), textové, číselné a plošné řádky
  Nastavení (i při úpravě), pole dialogů, pole komentářů revize, composer commitů,
  text editoru souborů a upravitelný výsledek editoru sloučení (jeho panely jen pro
  čtení si ponechávají šipku). V detailech commitu jsou zpráva a popis upravitelné,
  takže na nich svítí I-beam a celý blok se lehce rozsvítí.
- **V detailech commitu** jsou informační řádky ovládací prvky: klikem na
  `commit: <id>` zkopírujete krátké id, dvojklikem na řádek autora nebo data ho
  upravíte (označí je ruka a lehká značka). Řádek statistik jen informuje a ponechává
  šipku.

Ukazatele změny velikosti zůstávají nad děliči a hranicemi sloupců a otevřená ruka
se ukáže jen při přenášení refu.

## Kopírování do schránky

Akce kopírování vloží text do systémové schránky počítače, na kterém GitPoulpi
běží, a tam, kde terminál tuto funkci prokazatelně respektuje, ho pošlou i
terminálu (OSC 52). Přes SSH nebo bez displeje se posílá jen OSC 52: pokud ho váš
terminál nepodporuje (iTerm2, dokud není zapnuto oprávnění ke schránce,
Terminal.app, GNU screen), nic se nezkopíruje a stavový řádek řekne, že text byl
jen odeslán terminálu. Viz [Výběr terminálu](terminals.md#schránka-akce-kopírování).
