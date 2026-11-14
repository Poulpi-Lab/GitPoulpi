[English](../log-and-history.md) · **Čeština** · [Deutsch](../de/log-and-history.md) · [Español](../es/log-and-history.md) · [Français](../fr/log-and-history.md) · [Italiano](../it/log-and-history.md) · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · [Português (Brasil)](../pt-BR/log-and-history.md) · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · [Türkçe](../tr/log-and-history.md) · [Українська](../uk/log-and-history.md) · [简体中文](../zh-CN/log-and-history.md)

# Práce s Logem

Log je střed GitPoulpi: celá historie jako graf, se všemi větvemi, tagy
a stashi. Tato stránka popisuje, jak ho číst, jak v něm hledat a vše, co lze
dělat s commity a refs. Oblasti jsou pojmenovány v
[prohlídce rozhraní](interface-tour.md); přesné klávesy jsou v
[Klávesových zkratkách](keybindings.md) a gesta myší v
[Myši a gestech](mouse-and-gestures.md).

![Log s vybraným merge commitem: jeho detaily a soubory vpravo](../images/log-graph.png)

## Čtení grafu

Každý řádek je commit, nejnovější nahoře. Tečka leží na dráze commitu; čáry
spojují commit s jeho rodiči; merge zobrazuje dvě čáry, které se sbíhají.
Pilulky vlevo pojmenovávají větve a tagy na daném commitu, v barvě dráhy,
s tenkou čárou k tečce. Ikona notebooku označuje lokální větev, zeměkoule
vzdálenou a `✓` větev, kterou máte přepnutou. `+N` znamená, že na ten commit
míří další refs; najetím je vypíšete, nebo použijte nabídku níže.

První řádek jsou **pracovní změny** (přerušovaný kruh: vaše necommitnutá práce,
spojená s commitem, na kterém leží). Stashe se zobrazují jako malé čtverce na
dráze vedle commitu, na kterém vznikly.

### Barvy, podbarvení řádků a avatary

Barvy drah pocházejí z motivu. *Nastavení > Vzhled > Barevný motiv grafu*
spojí libovolný motiv s libovolnou paletou drah a výběr palety v paletě
příkazů *Aplikace: Změnit barevný motiv grafu...* (`Ctrl+N Shift+Y`) ukazuje
každou paletu živě; viz [Motivy](themes.md). *Pozadí drah grafu* nastavené na
**Podbarvení řádku** vykreslí slabý pruh barvy dráhy za sloupcem grafu a tenký proužek
na jeho okraji. *Síla podbarvení řádku* (**Silné**, **Střední** (výchozí),
**Jemné**) určuje, jak jsou pruhy vyblednuté.

| Paleta drah Serie s podbarvením řádků | Paleta drah Material s podbarvením řádků |
|---|---|
| ![Graf Logu v barevném motivu Serie s podbarvením řádků](../images/theme-serie.png) | ![Graf Logu v barevném motivu Material s podbarvením řádků](../images/theme-material.png) |

*Avatary autorů* (obrázek v tečce grafu nebo ve sloupci Autor) jsou ve
výchozím stavu vypnuté. Jejich zapnutím se do Gravataru odešle hash e-mailové
adresy každého autora a dotáže se váš poskytovatel hostingu. *Zdroje avatarů*
a *Náhradní avatar* na stejné stránce určují, odkud se obrázky berou a co se
nakreslí, když žádný není.

Pokud váš terminál neumí zobrazit obrázky, graf se nakreslí znaky.
*Vykreslování grafu* a *Protokol obrázků* vynutí jedno nebo druhé; viz
[Řešení potíží](troubleshooting.md).

Zvýraznění vybraného řádku nebo řádku pod ukazatelem začíná u kruhu commitu.
Dráhy vlevo od něj si ponechávají běžné pozadí.

### Sloupce

Kliknutím pravým tlačítkem na záhlaví vyberete sloupce: **Ref**, **Graf**,
**Zpráva** (vždy zapnutý), **Autor**, **Datum**, **Změny** a **SHA**.
Volitelný sloupec **Změny** ukazuje u každého commitu `n files +přidáno
-odebráno`. Tažením ohraničení mezi dvěma záhlavími změníte šířku sloupce;
volba se pamatuje, v buňkách terminálu. S výchozím `ui.column_resize = "line"` je ohraničením jednobuňková mezera na konci sloupce: v záhlaví v ní leží malá značka a dokud je ukazatel na mezeře (v záhlaví nebo kdekoli dole v tabulce), kreslí se přes celou výšku tabulky tenká akcentní čára a tažení odkudkoli z ní mění šířku (v terminálech, které to umějí, se ukazatel změní na ukazatel změny velikosti). S `"header"` není značka ani čára: první znak buňky záhlaví za ohraničením se při najetí rozsvítí a tažení začíná od něj, jen na řádku záhlaví (Nastavení > Log a graf > Změna velikosti sloupců). Řádky vypadají v obou režimech stejně: mezera je součástí sloupce vlevo od ní, takže výběr a barevné podbarvení řádků jí procházejí. Ref, Graf a ostatní sloupce si při změně velikosti nebo maximalizaci okna drží šířku: roste a zmenšuje se jen **Zpráva** (ve velmi úzkém okně ostatní sloupce dočasně ustoupí, nejdřív Graf, a když okno znovu naroste, vrátí se na vaše šířky). Zmenšíte-li sloupec Graf na nejmenší šířku, jednu tečku commitu, graf se zkompaktní: každý commit leží na jediné dráze pod předchozím, bez čar větví a merge. O buňku širší a strom je zpět. S obrázky autorů v kroužcích commitů (Nastavení > Vzhled > Avatary autorů: graf) je kroužek široký dvě buňky, takže sloupec přestane o buňku dřív (tři buňky) a obrázek nikdy neořízne.

Tažení záhlaví ukáže duchy jeho názvu, který sleduje ukazatel, a akcentní proužek tam, kam by sloupec dopadl; pořadí se změní při pustění a `Esc` ho zruší. Výběr otevřený pravým tlačítkem je zaškrtávací seznam (`[x]` / `[ ]`) s ✕ v titulku; dosáhnete na něj i z palety (`Zobrazení: Log: Vybrat zobrazené sloupce`), pohybujete se v něm šipkami a přepínáte `Mezerníkem` / `Enter`.

Z klávesnice `Shift+C` (nebo `Zobrazení: Sloupec Logu: Vybrat sloupec k přesunu nebo změně šířky` v paletě) vybere sloupec a jeho buňka v záhlaví se rozsvítí. Pak `Tab` / `Shift+Tab` vyberou jiný sloupec, `<` / `>` ho přesunou o jedno místo doleva nebo doprava mezi zobrazenými sloupci (klávesnicový protějšek přetažení jeho záhlaví), `]` / `[` ho rozšíří nebo zúží (protějšek přetažení jeho hranice; poslední sloupec posouvá svůj levý okraj), `0` vrátí všechny šířky na automatické a `Esc` nebo `Enter` výběr ukončí. Šipky se mezitím dál pohybují po commitech. Pořadí a šířky se pamatují jako po tažení a patří samotnému Logu: sloupec commitů vedle diffu má seznam vlastní.

*Přizpůsobení obsahu.* **Dvojklik** na hranici sloupce přizpůsobí sloupec nalevo jeho nejširšímu viditelnému obsahu a záhlaví (široké znaky a emoji se počítají podle buněk, které zabírají). V rozložení fit Log vždy vyplní svůj panel, takže sloupec skončí u toho, co panel unese, jakmile **Zpráva** dostane svůj podíl a každý další sloupec svou nejmenší šířku: stavový řádek pak řekne `Autor potřebuje 34 buněk, vejde se 20.` Totéž z klávesnice: s vybraným sloupcem (`Shift+C`, `Tab`) ho `=` přizpůsobí a `+` přizpůsobí všechny sloupce; paleta má `Zobrazení: Sloupec Logu: Přizpůsobit sloupec obsahu` a `... Přizpůsobit všechny sloupce obsahu`; výběr otevřený pravým tlačítkem pod zaškrtávátky nabízí `Přizpůsobit <sloupec> obsahu` (sloupec pod ukazatelem), `Přizpůsobit všechny sloupce obsahu` a `Obnovit šířky sloupců`. **Zpráva** je pružný sloupec a nemá vlastní šířku, kterou by šlo přizpůsobit: zabere místo, které ostatní nechají.

*Posouvání do stran.* Ve výchozím stavu se každý sloupec vejde do panelu. V rozložení **scroll** (zaškrtávátko výběru `Posouvat do stran (sloupce si drží šířky)`, Nastavení > Log a graf > Rozložení sloupců, paleta `Zobrazení: Log: Přepnout posouvání sloupců do stran`, nebo `s` při vybraném sloupci) si sloupce drží šířky, včetně **Zprávy** (tam má šířku vlastní a její přizpůsobení vezme nejširší shrnutí), při úzkém okně se nic nevzdává a tabulka může být širší než panel. Posouvá ji lišta na spodním okraji: táhněte jezdec, kliknutím na prázdnou dráhu přesuňte jezdec pod ukazatel, nebo použijte koncové šipky. Z klávesnice `Shift+Left` / `Shift+Right` posouvají o čtyři buňky (paleta má `Zobrazení: Log: Posunout sloupce doleva` / `doprava`); totéž udělá vodorovné kolečko nebo `Shift`+kolečko nad tabulkou a výběr sloupce (`Shift+C`, `Tab`) ho posune do zorného pole. Tabulka užší než panel nechá vpravo prázdné místo a pás vybraného řádku sahá k okraji. Každé rozložení má své šířky (`ui.log_column_widths` a `ui.log_column_widths_scroll`); první tažení, klávesa nebo přizpůsobení v rozložení scroll uloží každý sloupec v jeho aktuální šířce, takže se pohne jen ten, který měníte. V rozložení fit přizpůsobení, které se do panelu nevejde, to řekne a odkáže na toto rozložení.

*Úchyt posledního sloupce.* Poslední sloupec má úchyt také, na své poslední buňce (ne na okraji panelu, který dál mění velikost panelů): tažení ho posune o vzdálenost, kterou ukazatel urazil, a rozdíl převezme **Zpráva**. **SHA** je ve výchozím stavu široký 8 buněk (7 číslic a buňka úchytu).

![Log se zobrazeným sloupcem Změny: každý commit uvádí počet souborů a přidaných a odebraných řádků](../images/log-changes-column.png)

### Autoři a `.mailmap`

Když má repozitář soubor `.mailmap` (soubor gitu, který dává jedno jméno a e-mail
člověku, který commitoval pod několika), sloupec **Autor** v Logu, podrobnosti
commitu, Blame a Historie souboru ukazují **kanonickou** identitu, stejně jako
`git log`. Samotné commity zůstávají nedotčené. Nastavení `log.use_mailmap`
(Nastavení > Log a graf, ve výchozím stavu zapnuto) to vypne a ukáže uložená jména.
Hledání podle autora odpovídá jménu a e-mailu tak, jak se zobrazují, takže hledání
kanonického jména najde všechny commity dané osoby. Karta **Identity** nástroje pro
přepis historie vypíše jména a e-maily z historie a zapíše za vás řádky do `.mailmap`,
viz `history-rewrite.md`; z terminálu: `gitpoulpi mailmap list|check|add`.

## Hledání

- **Hledání** (`f`, tlačítko **Hledat** na panelu nástrojů, nebo příkaz
  z palety *Navigace: Hledat commity...*): prohledává celou zprávu commitu,
  jméno a e-mail autora, hash a názvy refs. Co neodpovídá, ztlumí a tvar grafu
  ponechá. `Enter` / `Shift+Enter` přeskočí na další / předchozí shodu.
- **Filtr Refs** (`/` v postranním panelu): zúží seznamy větví, tagů a remote
  podle názvu.
- **Solo** (klávesa `o`, nebo nabídka refu, na refu, složce či remote): zobrazí
  jen commity refs ve stavu solo; více refs ve stavu solo zobrazí jejich
  sjednocení. `Shift+O` solo ukončí.
- **Skrytí** refu (jeho nabídka) ho odebere z grafu: zmizí jeho pilulka i
  commity, kterých dosáhne jen on (commit s checkoutem zůstává vždy, spolu s
  řádkem pracovního stromu). *Skryto n* v záhlaví Refs je spočítá a **Zobrazit
  vše** (`Shift+H`) je vrátí. Celkový počet na stavovém řádku počítá viditelnou
  historii.

## Větve, tagy a postranní panel Refs

Dvojitým kliknutím na lokální větev nebo tag ji přepnete (checkout).
Kliknutím pravým tlačítkem na ref (v postranním panelu, nebo na jeho pilulku
v Logu) otevřete jeho nabídku. Nejužitečnější položky:

| Položka | Co dělá |
|---|---|
| Ukázat v grafu | Posune Log na commit refu (podle potřeby nejdřív načte starší historii) a řádek krátce zvýrazní; výběr zůstane. Jedno kliknutí na ref dělá totéž, pokud není v Nastavení vypnuto *Kliknutí na ref ho ukáže v grafu*. Ref skrytý v grafu (nebo přes Solo) to místo toho oznámí ve stavovém řádku |
| Checkout | Přepne na větev, nebo odpojí HEAD na tagu |
| Merge, Rebase | Provede merge tohoto refu do přepnuté větve, nebo rebase přepnuté větve na tento ref |
| Fast-forward `<větev>` na `<aktuální>` / `<aktuální>` na `<větev>` | Posune větev dopředu, je-li to fast-forward; jinak je zašedlá s důvodem (`rozešlo se, není to fast-forward`) |
| Vytvořit worktree z... | Otevře propojený worktree na vlastní kartě |
| Pull, Push, Fetch, Remote větve <větev>... | Síťové operace nad danou větví (Remote větve <větev>... nastaví upstream a navíc políčka Push a Pull také pro každý remote) |
| Skrýt v grafu / Zobrazit v grafu, Solo | Jen zobrazení; ref skryjí nebo označí jako solo (skrytím zmizí i commity, kterých dosáhne jen ten ref), aniž by se repozitáře dotkly |
| Přejmenovat, Smazat | Smazání se nejdřív zeptá; lze smazat i vzdáleného dvojníka |
| Kopírovat název větve, Kopírovat odkaz na větev na remote | Do schránky |

**Kde tag žije.** Tag může být na tomto počítači, na remote, nebo na obou a řádky
TAGS v postranním panelu (i pilulka tagu v Logu) to říkají značkou za názvem.
Najetím na tag získáte totéž slovy.

| Značka | Význam |
|---|---|
| šipka nahoru | Jen tady: žádný remote ho nemá. Jeho nabídka nabízí *Push* |
| šipka vlevo-vpravo | Tady i na remote, na stejném commitu |
| šipka dolů (řádek ztlumený) | Jen na remote: v tomto repozitáři není (nikdy se nestáhl, nebo byl tady smazán). Jeho nabídka nabízí *Fetch `<tag>` z `<remote>`* a *Smazat `<tag>` na `<remote>`...*; v paletě `Git: Fetch vybraného tagu` |
| šipka nahoru-dolů | Na remote také, ale na jiném commitu (push by byl odmítnut) |

git pro tagy žádné sledovací refy nevede, takže GitPoulpi zjišťuje, co má který
remote, z vašeho posledního **fetch** (jakéhokoli, i automatického) a z vlastních
push a smazání tagů. Při procházení se na síť nikdo neptá; když někdo jiný změnil
tagy na remote, proveďte fetch, abyste to viděli. Stejné informace jsou v
`gitpoulpi tag list` (pole `local`, `remote`, `differs_on`, `where`).

**Smazání tagu.** Když tag má i remote, červené **Smazat** otevře dialog se třemi
číslovanými řádky (`1`-`9`, nebo Nahoru/Dolů a `Enter`; `Esc` zruší) a zvýrazněný
řádek se pod sebou vysvětlí:

1. **Smazat lokálně**: odstraní ho jen z tohoto repozitáře. Provede se hned
   a zpráva řekne, že ho remote stále má. **Příští fetch ho vrátí** (git
   následuje tagy remote), pokud ho nesmažete i na remote.
2. **Smazat na `<remote>`**: odstraní ho tam pro všechny (`git push <remote>
   --delete`). Zeptá se na potvrzení, které jmenuje remote. Váš lokální tag
   zůstane. Nelze vrátit zpět.
3. **Smazat lokálně i na `<remote>`**: obojí, po stejném potvrzení, takže ho
   fetch nemůže vrátit. **Vrácení zpět obnoví jen lokální tag** (tentýž objekt
   tagu); pak je znovu jen lokální a *Push* ho vrátí na remote.

![Dialog Smazat tag se třemi řádky, Smazat lokálně, Smazat na origin a Smazat lokálně i na origin, nad sekcí Tagy v panelu refs, kde každý tag nese značku, zda je jen lokální, nebo i na origin](../images/tag-delete.png)

Při více remote má každý vlastní řádky. Tag jen tady si ponechá obyčejné
potvrzení; tag jen na jednom remote jde rovnou k potvrzení pro tento remote.
CLI: `gitpoulpi tag delete <name> [--local | --remote <r> | --both [--remote <r>]]`.

**Fetch a tagy.** Fetch také obnoví, co má který remote (značky), a stále platí
vlastní následování tagů gitem: tag smazaný jen tady se stáhne znovu. Tag, který
někdo **smazal na remote**, po jakémkoli fetch zmizí z pohledu remote ve značce,
ale zůstane tady jako váš lokální tag (teď *jen tady*), protože git ve výchozím
stavu tagy nemaže. **Nastavení > Synchronizace > Automaticky mazat tagy** (ve
výchozím stavu vypnuto; `gitpoulpi fetch --prune-tags`) smaže i takový lokální
tag, ale jen ten, který na remote byl: tag, který jste nikdy neodeslali, nebo
který jste přesunuli, se nikdy nedotkne. Aby se zjistilo, co se smazalo, je
potřeba git 2.41 nebo novější.

**Nová větev nebo tag.** Tlačítka **Větev** a **Tag** na panelu nástrojů,
`Ctrl+G b` a `Ctrl+G t`, nebo *Vytvořit větev zde...* u commitu se zeptají na
název v řádkovém poli ve sloupci Ref, na řádku cílového commitu. `Enter`
vytvoří, `Esc` zruší a název, který by git odmítl, se ohlásí červeně, aniž by
se pole zavřelo.

U commitu bez refu se při najetí na jeho řádek ve sloupci Ref také zobrazí vybledlá pilulka s názvem větve, do které patří (jinak vzdálené větve), odvozeným z historie; tagy se nikdy jako zástupné nezobrazují. Větev smazaná po sloučení se přesto zobrazí přeškrtnutě pod názvem z předmětu slučovacího commitu (`Merge branch 'x'`, `Merge pull request #N from owner/x`...); squash merge, rebase a fast-forward takovou stopu nezanechají. Najetí na pilulku ukáže její druh. Nic nevytváří: použijte výše uvedený příkaz pro vytvoření větve. Volba **Zobrazit zástupnou větev u najetého commitu** v Nastavení > Log a graf ji vypíná; tam, kde je sloupec Ref příliš úzký, se nekreslí.

**Remote.** `+` v záhlaví REMOTE jeden přidá; kliknutím pravým tlačítkem na
název remote ho můžete fetchnout, zkopírovat jeho webový odkaz, nastavit jako
výchozí, skrýt ho v grafu, upravit jeho URL nebo odebrat. **Skrytí remote**
(oko na jeho řádku, položka nabídky *Skrýt remote v grafu*, nebo `Remote: Skrýt
remote v grafu...` v paletě) skryje každou větev, kterou má nyní nebo později
získá, a commity, kterých dosáhne jen on, z Logu zmizí: způsob, jak
procházet jednu historii bez historie nesouvisejícího archivního remote; vrátíte
ho položkou *Zobrazit remote v grafu*. Zobrazení remote zobrazí i jeho větve,
které byly skryty jednotlivě. Větev skrytého remote to při použití jejího oka
řekne na stavovém řádku: nejdřív zobrazte remote.

**Stashe.** **Stash** odloží vše (včetně nesledovaných souborů); `Ctrl+G m`
se nejdřív zeptá na zprávu (se zapnutým AI ji v tomto poli navrhne `Ctrl+G`).
Kliknutím pravým tlačítkem na řádek stashe, v Logu nebo v sekci STASHES,
zobrazíte **Pop**, **Apply** (ponechat stash) nebo **Drop**.

**Worktree.** *Vytvořit worktree z...* (na větvi, tagu nebo commitu) otevře
dialog, který navrhne složku vedle repozitáře, volitelný název nové větve
a volbu „Otevřít nový worktree na kartě“. Worktree se objeví v sekci
WORKTREES; dvojitým kliknutím jej otevřete a jeho karta se sama zavře, když
je worktree odstraněn.

## Přetahování refs na commity

Můžete stisknout větev (v postranním panelu nebo její
pilulku v Logu) a přetáhnout ji. Za ukazatelem jede stínový štítek a platné
cíle se rozsvítí; po uvolnění se zobrazí nabídka:

- lokální větev **na jinou lokální větev**: Merge, Rebase, Interaktivní
  rebase, Fast-forward;
- lokální větev **na vzdálenou větev**: Push do tohoto remote;
- vzdálená větev **na lokální větev**: Merge, Rebase, Fast-forward, Pull.

`Esc` přetažení zruší. Klávesovým ekvivalentem je příkaz z palety
*Refs: Pustit vybraný ref na přepnutou větev...*.

**Merge, nebo fast-forward.** Když cíl nemá žádný vlastní commit (přetažená větev je jen před ním), merge by mohl cíl
prostě posunout dopředu. Ve výchozím stavu (Nastavení > Git > **Merge fast-forward** = *never*, nebo vlastní volba
repozitáře na jeho stránce Možnosti merge) řádek nabídky zní *Merge x do y (merge commit)* a vytvoří skutečný merge
commit, takže historie větev zachová; chcete-li prosté posunutí, použijte samostatný řádek *Fast-forward y na x* (pro
přímou linii použijte místo toho *Rebase*). S *allowed* řádek Merge zní *(fast-forward)* a s *only*
*(jen fast-forward)*, zašedlý s důvodem, když se větve rozešly. Stejné řádky a popisky jsou i v nabídce samotné větve
a `gitpoulpi merge --ff | --no-ff | --ff-only` volí pro jednotlivý merge.

## Nabídka commitu

Kliknutím pravým tlačítkem na commit (nebo `.` na něm) otevřete jeho nabídku:

![Nabídka commitu: cherry-pick, revert, reset, úprava zprávy, vysvětlení pomocí AI, interaktivní rebase, vytvoření větve, tagu, checkout, worktree, patche a akce kopírování](../images/context-menu.png)

| Položka | Co dělá |
|---|---|
| **Cherry-pick na `<větev>`** | Přehraje commit na přepnuté větvi |
| **Revert commitu...** | Vytvoří commit, který jej vrací. U merge commitu se nejdřív rozbalovací seznam zeptá, který rodič je hlavní linie (strana, která zůstane; rodič 1 je větev, na které merge vznikl), pak následuje obvyklé potvrzení. Zpět vrátí předchozí stav |
| **Reset `<větev>` na tento commit** | Podnabídka s druhem resetu; hard reset žádá o potvrzení |
| **Upravit zprávu commitu...** | Viz níže |
| **Upravit...** | Změní zprávu, jméno, e-mail a datum autora commitu v jednom okně; viz [Úprava commitu](#úprava-commitu-zpráva-autor-data) |
| **Vysvětlit změny (AI)** | Vyskakovací okno s vysvětlením od AI; jen když je [AI](ai.md) zapnuté |
| **Rozdělit commit...** | Rozdělí commit na několik; viz [Rozdělení commitu](#rozdělení-commitu) |
| **Rozdělit commit (AI)...** | Totéž, se seskupením navrženým modelem; jen když je [AI](ai.md) zapnuté |
| **Interaktivní rebase odsud...** | Viz níže |
| **Vytvořit větev zde...**, **Vytvořit tag zde...**, **Vytvořit anotovaný tag zde...** | Řádkové pole pro název |
| **Checkout tohoto commitu**, **Vytvořit worktree zde...** | Odpojený HEAD, nebo worktree |
| **Vytvořit patch z tohoto commitu...**, **odsud do `<větev>`...** | Zapíše soubory `.patch`; *Použít patch* je v nabídce ref |
| **Kopírovat odkaz na commit na remote**, **Kopírovat** > | Do schránky (hash, zpráva a další) |

## Úprava zprávy commitu (reword)

*Upravit zprávu commitu...* (nebo `Ctrl+G e`) otevře celou zprávu:
jednořádkový souhrn s počítadlem, pod ním popis, **Zrušit** a **Aktualizovat
zprávu**. Zprávu lze změnit u kteréhokoli commitu přepnuté větve (HEAD nebo
předka); commity nad ním se vytvoří znovu s vlastními stromy, autory a daty,
merge zůstanou merge a vaše změny staged i unstaged zůstanou nedotčené. Pokud
už byl commit odeslán, okno jantarově upozorní, že budete potřebovat force push
(s lease), a požádá o druhé potvrzení. `Ctrl+Z` (Zpět na panelu nástrojů)
reword vrátí; viz [Zpět a znovu](undo-redo.md). Stavový řádek také uvádí
předchozí špičku.

## Úprava commitu (zpráva, autor, data)

Položka *Upravit...* v nabídce commitu (nebo `Ctrl+G Shift+E`, nebo *Commit: Upravit...* v paletě) otevře jedno okno s vlastnostmi commitu: souhrn, popis a jméno, e-mail a datum autora, každé předvyplněné hodnotami commitu. Změňte, co chcete, a stiskněte **Použít**; odešlou se jen změněná pole. Datum se píše `YYYY-MM-DD HH:MM:SS +HH:MM` a dokud ho neupravíte, zobrazuje se v UTC. Committerem jste vy, teď (jako u reword). Upravit lze každý commit právě vybrané větve, se stejnými pravidly, stejným varováním u už odeslaného commitu a stejným druhým potvrzením jako *Upravit zprávu commitu...*; commity nad ním se vytvoří znovu s vlastními autory a daty a vaše změny ve stagingu i mimo něj zůstanou nedotčené. `Ctrl+Z` (Zpět na panelu nástrojů) operaci vrátí. V sestaveních s přepisem historie tlačítko **Upravit historii hlouběji...** (`Alt+H`) vlevo dole v dialogu ho zavře, aniž by cokoli použilo, a otevře [nástroj pro přepis historie](history-rewrite.md) s rozsahem od tohoto commitu po nejnovější; pokud jste upravili pole, nejdřív se zeptá (`Esc` potvrdí). `Ctrl+N` ukáže odznak u každého pole a tlačítka dialogu; stiskem odznaku dáte kurzor do toho pole nebo stisknete to tlačítko (viz [Odznaky](keybindings.md#odznaky)).

Každá cesta do dialogu ho otevře celý, se všemi poli viditelnými a upravitelnými; liší se jen umístění kurzoru. V panelu podrobností commitu (vpravo) klikněte pravým tlačítkem na souhrn, popis nebo blok autora a zvolte *Upravit souhrn...*, *Upravit popis...* nebo *Upravit autora...*, nebo dvakrát klikněte na souhrn, popis, autora nebo datum vytvoření: dialog se otevře s kurzorem v tom poli. Klávesové ekvivalenty fungují z Logu i z panelu podrobností: `Ctrl+G Shift+E` (všechna pole), `Ctrl+G Shift+A` (autor) a v paletě *Commit: Upravit...*, *Commit: Upravit autora...*, *Commit: Upravit souhrn...* a *Commit: Upravit popis...*. Obě zkratky lze přiřadit jinak (`git_prefix.amend`, `git_prefix.edit_author`; viz [Klávesové zkratky](keybindings.md)). Ze skriptu nebo od agenta: `gitpoulpi amend <commit>` s `--message` (nebo `--summary` / `--description`), `--author-name`, `--author-email`, `--author-date` a odpovídajícími `--committer-*` (nástroj MCP se jmenuje `amend`).

## Rozdělení commitu

*Rozdělit commit...* v nabídce commitu (nebo v paletě: *Git: Rozdělit vybraný commit...*)
rozdělí jeden commit na dva nebo více, od nejstaršího. Rozdělit lze každý commit
přepnuté větve, který není merge (HEAD i předka, také kořenový commit). Editor
rozdělení má dva panely: vlevo **Nové commity** (každý s vlastní zprávou) a vpravo
**Změny** původního commitu, jeden řádek na soubor a hunk, každý s polem `[N v]`, které
určuje, do kterého nového commitu patří.

| Klávesa | Myš | Co dělá |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | klik na řádek | pohyb po změnách (`Shift+Up` / `Shift+Down` rozšíří výběr) |
| `1`-`9` | pole `[N v]` na řádku, pak volba | přiřadí vybranou změnu tomuto novému commitu; `0` ji nechá nepřiřazenou |
| `Enter` / `m` | pole `[N v]` | otevře seznam commitů pro vybranou změnu |
| `n` | `+ Nový commit` | vloží vybranou změnu do nového commitu |
| `v` | | **řádkový režim**: rozdělí hunk po řádcích (tažením přes řádky vyberete rozsah) |
| `Tab` | klik na panel | přepíná mezi commity a změnami |
| `e` / `Enter` (v panelu commitů) | tužka | upraví zprávu tohoto commitu |
| `d` / `Delete` | křížek | odebere nový commit (jeho změny se stanou nepřiřazenými) |
| `Alt+Up` / `Alt+Down` | šipky | posune nový commit nahoru nebo dolů |
| `s` | zaškrtávací pole | podepíše nové commity |
| `Shift+R` | `Obnovit` | zpět na výchozí plán |
| `Ctrl+S` | `Rozdělit na N` | rozdělí |
| `Esc` | křížek, `Zrušit` | odejde; při neuložených úpravách je třeba stisknout podruhé |

Každá změna musí patřit právě do jednoho commitu; editor počítá, co je ještě *Nepřiřazeno*,
a tlačítko zůstává odmítnuté, dokud není počet nula. Soubor, který je binární, mění jen
režim, je v commitu nový nebo smazaný, je submodul nebo soubor LFS, lze přiřadit jen
celý. První commit začíná původní zprávou; každému dalšímu zprávu napište.
Nový commit zachová původního autora a datum; committerem jste vy, teď.

**V repozitáři se nezmění nic kromě větve.** Poslední nový commit má přesně obsah
původního commitu, takže commity nad ním se vytvoří znovu se stejnými stromy, autory
a zprávami (merge zůstanou merge) a vaše změny v indexu i mimo něj zůstanou nedotčené:
špinavý pracovní strom nevadí a konflikt nemůže vzniknout. Hooky se nespouštějí. Dokud
čeká auto-stash, je rozdělení zablokováno.

**Podepisování.** Zaškrtávací pole *Podepsat commity* vychází z `commit.gpgSign`
repozitáře. Nezaškrtnuté: podpisy přepsaných commitů (nových i těch nad nimi) se
zahodí a editor to pod polem říká. Zaškrtnuté: podepíšou se všechny; když podepsání
selže, nezmění se nic.

**Zpět.** Jedno `Ctrl+Z` (Zpět na panelu nástrojů) vrátí původní commit, i se
špinavým pracovním stromem; viz [Zpět a znovu](undo-redo.md). Commit, který už byl
odeslán, pak vyžaduje force push (s lease).

**Z interaktivního rebase.** Každý řádek přehledu rebase má malé tlačítko
**rozdělit...** (`t` na řádku): otevře stejný editor pro daný commit a po dokončení
rozdělení (nebo zrušení) jste zpět na přehledu, už na nové historii. Pokud jste
upravili plán, stiskněte rozdělit podruhé a potvrďte, že se tyto úpravy ztratí.

Pro skripty a agenty: `split-source` a `split-commit` jsou v
[Referenční příručce CLI](cli-reference.md); agent může tento editor také otevřít předvyplněný
k vaší kontrole (`ui propose split`, viz [MCP a agenti](mcp-and-agents.md)).

## Interaktivní rebase

Zvolte **Interaktivní rebase odsud...** u commitu, který je předkem přepnuté
větve, nebo přetáhněte větev na jinou a zvolte *Interaktivní rebase*. Log se
nahradí obrazovkou s kontrolou: jeden řádek na commit, nejnovější nahoře,
každý s úchytem pro tažení a rozbalovacím seznamem akcí.

![Obrazovka interaktivního rebase: čtyři commity, druhý nastavený na Squash a třetí na Drop, s tlačítky Obnovit, Zrušit rebase a Spustit rebase](../images/rebase.png)

| Akce | Klávesa | Účinek |
|---|---|---|
| Pick | `p` | Ponechá commit |
| Reword | `r` | Ponechá ho s novou zprávou |
| Squash | `s` | Sloučí ho do commitu pod ním a spojí zprávy |
| Fixup | `f` | Sloučí ho do commitu pod ním a jeho zprávu zahodí |
| Drop | `d` | Vynechá ho |
| Edit | `e` | Zastaví se tam, abyste ho mohli upravit (amend) |

Tažením řádku (nebo `Ctrl+Up` / `Ctrl+Down`) změníte pořadí. `a` zobrazí náhled
autosquash (commity `squash!` a `fixup!` se přesunou vedle svých cílů). Nic se
nestane, dokud nestisknete **Spustit rebase** (`Ctrl+S`); **Obnovit** vrátí
plán zpět a **Zrušit rebase** (`Esc`) odejde bez jakékoli změny. Když se krok
zastaví na konfliktu nebo na `edit`, banner nad Logem nabídne **Pokračovat**,
**Přeskočit** a **Přerušit**. Rozsah obsahující merge commit nelze rebasovat
interaktivně; položka nabídky uvádí důvod.

## Cherry-pick více commitů

`Ctrl+klik` přepíná commit, `Shift+klik` (nebo `Alt+klik`) vybírá rozsah, `Space` přepíná
aktuální, `Esc` výběr zruší. Kliknutím pravým tlačítkem na vybraný commit
zvolte **Cherry-pick N commitů na `<větev>`...**: otevře se stejná obrazovka
s kontrolou, se stejnými akcemi a změnou pořadí, a **Spustit cherry-pick**
je přehraje popořadě.

## Kopírování commitů jako Mermaid gitGraph

![Vyskakovací okno Zkopírovat jako Mermaid gitGraph nad třemi vybranými commity: volba rozsahu, detailu a směru, volba bloku a živý náhled](../images/log-mermaid.png)

Vyberte commity v Logu (`Ctrl+klik`, `Shift+klik`, `Space`; nebo nechte kurzor na jednom)
a otevřete **Zkopírovat jako Mermaid gitGraph...**: `Shift+Y` v Logu, podnabídka **Kopírovat**
v nabídce commitu, nebo `Log: Zkopírovat jako Mermaid gitGraph...` v paletě. Okno ukáže vygenerovaný skript
[Mermaid](https://mermaid.js.org/syntax/gitgraph.html), živě a s posouváním; `Enter`
(nebo tlačítko **Kopírovat**) vloží přesně tento text do schránky. Vložte ho do README, popisu pull
requestu nebo na [mermaid.live](https://mermaid.live).

Okno má tři comboboxy, zaškrtávací pole a náhled (`Tab` se mezi nimi pohybuje; každý
combobox se otevře klávesou `Enter`, `Space` nebo `Down`):

- **Commity**: výběr, nebo - když je vybrán jeden commit a HEAD z něj vychází - **Od výběru po HEAD**,
  řada commitů mezi nimi (odpověď se bere z načtené části Logu).
- **Detail**: `structure` (commity, větve a merge; id jsou krátká SHA), `refs` (přidá skutečné
  názvy větví jako větve Mermaid, tagy jako `tag:` a commit HEAD jako `type: HIGHLIGHT`),
  `subjects` (id commitu se změní na `<sha> <předmět>`, s escapováním a zkrácené na `ui.mermaid_subject_width`
  znaků) a `full` (k id přidá autora a datum v UTC). V rozbalovacím seznamu se úrovně jmenují
  Struktura, Refs, Předměty a Úplné.
- **Směr**: zleva doprava (`gitGraph LR:`) nebo shora dolů (`gitGraph TB:`).
- **Zahrnout blok ```mermaid** obalí skript blokem kódu Markdown.

Detail, směr a blok se pamatují (`ui.mermaid_detail`, `ui.mermaid_orientation`,
`ui.mermaid_fence`). Stejný generátor běží i z příkazové řádky:
`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence]
[--range A..B | --limit N]` (viz [referenční příručka CLI](cli-reference.md)); server MCP ho nabízí jako
argument `format` nástroje `log`.

`gitGraph` v Mermaid je jazyk přehrávání (commit, branch, checkout, merge), ne popis grafu,
takže generátor dělá několik rozhodnutí, která stojí za to znát:

- Commit pokračuje ve větvi svého prvního vybraného rodiče; ostatní potomci otevřou novou
  větev pojmenovanou podle skutečné větve na něm (`b1`, `b2`... na úrovni `structure`). První větev je `main`
  (nebo skutečný název, přes hlavičku `mainBranchName`) a sleduje řetěz prvních rodičů od HEAD.
- Merge připojí větev druhého rodiče. Když tato větev po rodiči pokročila dál, generátor
  v rodiči založí malou ukazatelovou větev (`<větev>-<sha>`) a sloučí ji, takže
  merge pořád ukazuje na správný commit.
- Rodič mimo výběr se vypustí: commit, kterému nezbyl žádný vybraný rodič, začne vlastní
  linii, takže mezery ve výběru se ukážou jako nepropojené linie.
- Chobotnicový merge (tři a více rodičů) se změní na posloupnost merge; mezilehlé mají
  příponu ` (octopus 1/2)`, poslední nese skutečný commit.
- Cherry-picky se nepoznají; dvojité uvozovky v předmětech se změní na apostrofy (Mermaid je
  neumí escapovat).

## Porovnání commitů

Vyberte dva nebo více řádků Logu: `Ctrl+klik` přepne řádek, `Shift+klik` (nebo `Alt+klik`, `Shift+Up/Down`, `Alt+Up/Down`) vybere rozsah, `Space` přepne aktuální řádek. Vybrat lze commity, řádek **WIP** i řádky **stash**. Pravý panel se změní na panel **Porovnání**:

- záhlaví má tlačítko **Prohodit** a pod ním je každý vybraný řádek vypsaný v pořadí Logu (nejnovější první) jako dvouřádková **dlaždice**: avatar autora (podle nastavení avatarů Logu: obrázek na terminálech, které obrázky umí, jinak iniciály), jméno, jak dávno, krátké id a souhrn. Řádek **WIP** je „Pracovní strom / Necommitnuté změny“, stash ukazuje symbol stashe, `stash@{n}` a svou zprávu. Nejstarší konec nese štítek **Od** a nejnovější štítek **Do** (Prohodit je vymění); u tří a více řádků poznámka říká, že změny jsou sloučené. Seznam dlaždic zabere nejvýš třetinu panelu a pak se posouvá (kolečko, posuvník, nebo přetažením dělicí lišty pod ním změníte velikost);
- kliknutím na dlaždici přesunete kurzor Logu na ten řádek a necháte ho v grafu zablikat; výběr se nezmění, takže `Space` pak ten řádek z výběru odebere;
- pod tím obvyklý seznam změněných souborů (zobrazení cest nebo stromu, filtr, řazení) vypisuje soubory, ve kterých se liší. Kliknutím na soubor, nebo stiskem `Enter`, otevřete jeho diff uprostřed, ve stejných rozloženích sjednoceném nebo vedle sebe, s File Insight (blame, historie, zobrazení souboru) na novějším konci.

![Tři vybrané řádky Logu: panel Porovnání je vypisuje jako dlaždice s avatary a štítky Od / Do nad změněnými soubory](../images/compare-cards.png)

Co se porovnává, závisí na výběru:

| Výběr | Porovnává se |
|---|---|
| dva commity (nebo stash a commit) | starší s novějším, v pořadí Logu, bez ohledu na pořadí klikání |
| řádek WIP a commit nebo stash | ten commit s pracovním stromem: indexované i neindexované změny sledovaných souborů (nesledované soubory nejsou součástí diffu) |
| tři a více commitů | jejich sloučené změny: od chvíle těsně před nejstarším po nejnovější (merge sledují svého prvního rodiče) |

Přejmenování se ukáže jako smazání plus přidání. **Prohodit** (`x`) vymění konce porovnání dvou řádků; u řádku WIP, který je vždy novějším koncem, nebo u rozsahu tří a více řádků není co měnit. Porovnání s řádkem WIP sleduje pracovní strom, jak ukládáte soubory.

Klávesnice: `v` (nebo **Log: Porovnat výběr** v paletě příkazů) přesune fokus do seznamu souborů porovnání; `x` (nebo **Log: Prohodit konce porovnání**) je prohodí; `n` (nebo **Log: přejít na další porovnávaný řádek**) je klávesové dvojče kliknutí na dlaždici: kurzor Logu přeskočí na další vybraný řádek a na konci se vrátí na začátek. `Esc` nebo prosté kliknutí na řádek vrátí zpět k jedinému commitu. Položka **Cherry-pick** počítá jen commity výběru; řádek WIP a stashe se tam ignorují.

Stejné seznamy jsou k dispozici skriptům a agentům: `gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]` (CLI a nástroj MCP `diff`; `--to` je ve výchozím stavu pracovní strom).

## Konflikty a další bannery

Když se merge, rebase, cherry-pick nebo revert zastaví na konfliktu, nad Logem
se objeví banner a soubory v konfliktu mají v panelu stavu červený `!`.
Vyřešte je v editoru merge a pak použijte **Pokračovat** nebo **Commit merge**
v banneru, případně **Přerušit** pro ústup. Viz
[Řešení konfliktů merge](merge-conflicts.md).

Další bannery nad Logem:

| Banner | Tlačítka | Paleta |
|---|---|---|
| *Rebase `<větev>` na `<commit>` · krok/celkem · zastaveno na ...* | **Pokračovat**, **Přeskočit**, **Přerušit** | *Rebase: Pokračovat / Přeskočit / Přerušit* |
| *Aplikace patchů (git am) · patch n z m* (zastavený `git am`) | **Vyřešit konflikty…** (když záplata koliduje), **Pokračovat**, **Přeskočit**, **Přerušit** (nejdřív se zeptá) | *git am: Pokračovat / Přeskočit tuto záplatu / Přerušit* |
| *Zbylý index.lock (stáří n s) blokuje každou změnu v tomto repozitáři* (příkaz git zemřel a nechal `.git/index.lock`) | **Odstranit index.lock** (nejdřív se zeptá; nabízí se, jen když je soubor starý aspoň 10 sekund) | *Repozitář: Odstranit zastaralý index.lock* |
| *Čeká auto-stash: ...* | Viz [Auto-stash](#auto-stash) | *Banner: ...* |

## Blame

Blame ukazuje celý soubor s čísly řádků a barvami syntaxe a sloupcem anotací **vpravo** od kódu (`ui.blame_side = "left"` ho přesune doleva). Otevřete ho z lišty souboru (`Blame`), z nabídky souboru, z palety nebo z řádku historie (`b`). Z pracovního stromu se řádky, které ještě nejsou v commitu, ukazují zvlášť (viz Pruhy).

- **Hunky.** Řada po sobě jdoucích řádků, které naposledy změnil jeden commit, je hunk. Jeho první řádek ukazuje avatar autora (obrázek tam, kde terminál kreslí obrázky, jinak odznak s iniciálami, nic při vypnutých *Avatarech autorů*), autora, jak dávno a shrnutí commitu; ostatní řádky ukazují jen barevný pruh autora. Hunky odděluje tenká čára. Začíná-li hunk nad obrazovkou, jeho anotace se zopakuje na prvním viditelném řádku.
- **Pruhy.** Vedle anotace: barevný pruh autora a **pruh teploty stáří**: teplý pro nové řádky, studený pro staré, vztažený k nejstaršímu a nejnovějšímu řádku samotného souboru (legenda je v titulku). Řádky, které ještě nejsou v commitu, jsou tečkované a říkají „Zatím necommitnuto“.
- **Karta.** Nechte ukazatel chvíli nad anotací, nebo stiskněte `i`, a uvidíte detailní kartu hunku: úplné ID commitu, autora a e-mail, datum, shrnutí a jeho akce. Vybraný hunk je podbarvený.

  ![Blame se zapnutou minimapou a kartou otevřenou u vybraného hunku](../images/blame-card.png)
- **Akce** (řádky karty, kontextová nabídka na hunku, klávesy a `Blame: ...` v paletě): ukázat commit v Logu (`Enter`), zkopírovat jeho ID (`Shift+Y`), ukázat diff commitu pro tento soubor (`d`), Blame na rodiči commitu (`b`, sleduje přejmenování), otevřít Historii souboru (`Shift+H`).
- **Pohyb.** `j` / `k`, PageUp/PageDown, Home, `Shift+G` a kolečko jako všude; `n` / `p` další / předchozí hunk, `Shift+N` / `Shift+P` další / předchozí hunk téhož commitu, `g` přejít na číslo řádku.
- **Vysoké hunky.** Pod shrnutím ukazuje hunk o třech a více řádcích krátké ID commitu a celé datum (tlumeně), aby sloupec nezůstal prázdný.
- **Minimapa.** Při zapnutém `ui.minimap` (tlačítko `Mapa`, `M`) je minimapa souboru vedle kódu, obarvená stejnou teplotou stáří jako pruh; kliknutím nebo tažením podél ní posouváte.
- **Šířka.** Přetáhněte okraj sloupce anotací nebo stiskněte `>` / `<`; šířka se uloží jako `ui.blame_width`.

## Historie souboru

Otevřete ji segmentem `Historie` přepínače `Historie │ Vše` v liště diffu, z nabídky souboru nebo z palety (`Historie souboru: ...`). Střed se rozdělí na dvě poloviny: vlevo commity souboru, nejnovější první, včetně přejmenování; vpravo diff souboru pro vybraný commit se všemi režimy zobrazení Diff (hunky / celý soubor, v řádku / vedle sebe, slovní diff, minimapa, zalamování).

- **Seznam.** Dva řádky na commit. První: písmeno změny (A přidán, M změněn, D smazán, R přejmenován), shrnutí a barevné `+přidáno -odebráno`. Druhý, tlumený: avatar autora (přesně to, co ukazuje sloupec Autor v Logu podle *Avatarů autorů*), autor, relativní datum, krátké ID commitu a `staré -> nové` u commitu, který soubor přejmenoval. Úzký seznam zahodí nejdřív přejmenování, pak ID, pak konec jména autora, nikdy však autora ani datum. Stránky po 100 se dočítají při blížícím se konci; zatím je vidět řádek `načítání dalších…`.
- **Pohyb.** `j` / `k`, šipky, PageUp/PageDown, Home / End, kolečko a klik; diff následuje okamžitě (diffy se čtou na pozadí, ukládají se po řádcích a držená klávesa si vyžádá jen jeden). Řádek před přejmenováním se porovnává pod názvem, který soubor tehdy měl.
- **Diff / Soubor / Blame.** Tlačítka **Soubor**, **Diff** a **Blame** v liště zobrazení volí, co pravá strana pro vybraný commit ukazuje: jeho **zobrazení Diff**, **zobrazení Soubor** (soubor, jak vypadal v tom commitu) nebo jeho **Blame**, vše uvnitř rozdělení se seznamem vlevo, a segment `Historie` přepínače sloupce zůstává rozsvícený. Zvolený režim se při změně řádku zachová a pravá strana sleduje výběr. Klávesy `Alt+1` / `Alt+2` / `Alt+3` (v seznamu také `d` / `o` / `b`), z kterékoli poloviny, a řádky `Historie souboru: ...` v paletě dělají totéž; `Esc` vrátí k Diffu.
- **Sloupec.** Seznam je *sloupec commitů* se seznamem **Historie**: skrytí, zobrazení, šířky a přepínač `Historie │ Vše` popisuje jedno místo, [Sloupec commitů](interface-tour.md#sloupec-commitů). Kliknutí na `Vše` opustí Historii souboru a ponechá pohled na vybraném commitu.

  ![Historie souboru s vybraným commitem uprostřed: jeho diff vpravo](../images/file-history.png)

  ![Historie souboru s pravou stranou v režimu Blame, blame souboru ve vybraném commitu](../images/file-history-blame.png)
- **Porovnání dvou commitů.** `Shift+Up` / `Shift+Down` nebo `Shift+klik` vyberou rozsah; pravá strana ukáže diff souboru mezi oběma konci, každý s vlastní cestou. `Esc` rozsah zahodí.
- **Fokus.** `Tab` (nebo klik) předá klávesnici diffu a zpět; diff si ponechá vlastní klávesy.
- **Akce** (kontextová nabídka, klávesy a `Historie souboru: ...` v paletě): ukázat v Logu (`Enter`, dvojklik; Log se načte tak daleko, jak je třeba), zkopírovat ID (`y`) nebo shrnutí (`Shift+Y`), ukázat diff (`d`), soubor v tom commitu (`o`) nebo jeho blame (`b`) vpravo, porovnat s pracovním stromem (`c`, znovu pro návrat), obnovit soubor do té verze (`Shift+R`, s potvrzením).
- **Zpět.** Obnovení verze je jeden zaznamenaný krok: Zpět (`Ctrl+Z`) vrátí staré bajty souboru.
- **Dělič.** Přetáhněte okraj mezi polovinami nebo `Alt+Right` / `Alt+Left`, `Alt+0` pro obnovení; šířka se pamatuje pro relaci.

## Fetch, pull a push

- **Pull** (`Ctrl+G p`) provede pull podle nastavení *Nastavení > Git a kódování >
  Tlačítko Pull* (`git.pull_mode`: merge, jen fast-forward, rebase; nastavení
  pro každý repozitář zvlášť). Jeho `▾` jednou provede *Fetch všech*, *Pull
  (fast-forward, pokud lze)*, *Pull (jen fast-forward)* nebo *Pull (rebase)*,
  aniž by tlačítko změnilo. Režim se gitu vždy předává explicitně, takže vás
  `pull.rebase` z konfigurace gitu nemůže překvapit.
- **Push** (`Ctrl+G Shift+P`) odešle přepnutou větev do výchozího remote
  a poprvé nastaví upstream. Jeho `▾` jednou odešle do jiného remote, který
  zvolíte. Pokud má remote commity, které vy nemáte, dialog nabídne **Pull
  (merge), poté push**, **Force push (s lease)...** (nikdy holé `--force`) nebo
  **Zrušit**.
- `Ctrl+G f` provede fetch všech remote. V *Nastavení > Synchronizace*
  rozhoduje *Auto-Prune*, zda fetch odstraňuje zastaralé refs (prune),
  a *Interval Auto-Fetch (minuty)* spouští fetch na pozadí.

Vše je také dostupné jako `gitpoulpi fetch`, `pull` a `push` v
[CLI](cli-reference.md).

## Auto-stash

Když necommitnuté změny blokují checkout, pull, merge nebo rebase, GitPoulpi je
umí odložit stranou za vás: odloží je do stashe, provede operaci a pak stash
vrátí (pop). *Nastavení > Synchronizace > Auto-stash před checkoutem, pull, merge nebo rebase*
(`general.auto_stash`):

- **zeptat se** (výchozí): dialog nabídne *Stashnout změny a pokračovat*, *Vždy
  stashnout automaticky* (totéž, a nastaví `general.auto_stash = "always"`,
  takže se už nebudete ptát; zpět to vrátí Nastavení > Synchronizace) nebo
  *Zrušit*. CLI a MCP místo toho selžou a uvedou `--autostash`.
- **vždy**: provede se bez ptaní. **nikdy**: operace se odmítne.

*Zahrnout nesledované soubory do auto-stash* zahrne i nesledované soubory.
Čistý strom stash nikdy neuvidí. Stash, operaci i pop lze každé vrátit zpět.

Pokud stash nelze vrátit (koliduje s výsledkem, nebo se merge či rebase
zastavil na vlastním konfliktu), zůstane v seznamu stashů. **Banner nad
Logem** oznámí *Čeká auto-stash: vaše změny před checkoutem jsou
v stash@{n}*. Jeho tlačítka:

| Tlačítko | Co dělá |
|---|---|
| **Vyřešit konflikty…** | Otevře editor merge (jen po popu s konflikty) |
| **Ponechat výsledek** | Ponechá pracovní strom, jak je, a zahodí kopii ve stashi (jen po popu s konflikty, když už žádný soubor není v konfliktu) |
| **Pop teď** (**Zkusit pop znovu** po neúspěšném popu) | Vrátí stash zpět |
| **Zobrazit stash** | Zobrazí změny stashe |
| **Začít znovu** | Viz níže |
| **Drop** | Zahodí stash (nejdřív se zeptá) |

Každé tlačítko má příkaz v paletě (*Banner: Pop čekajícího auto-stash*,
*Banner: Začít znovu...*, ...). Nic se za vás nezkouší znovu ani nezahazuje.

**Začít znovu** se nejdřív zeptá. Potom:

1. uloží kopii aktuálního stavu odložených souborů pod
   `refs/gitpoulpi-backup/autostash-start-over-<čas>`;
2. zahodí změny jen na těchto souborech (odmítne to a pojmenuje je, pokud mají
   jiné soubory místní změny, které by se mohly ztratit);
3. vrátí zpět operaci, která auto-stash vyvolala, přes žurnál zpět (takže ta
   operace musí být poslední);
4. vrátí stash na původní commit a banner zmizí.

Pokud krok selže, zastaví se se stashem nedotčeným a řekne kde. Dokud je banner
zobrazen a poslední operací je ta, která auto-stash vyvolala, udělá totéž
**Zpět** (`Ctrl+Z`, `gitpoulpi undo`), po potvrzení. Skripty používají
`gitpoulpi autostash start-over [--dry-run] [--yes]` (návratový kód 3 bez
`--yes`).

**Dokud auto-stash čeká, operace posouvající historii se odmítají:** checkout
větve nebo commitu, merge, rebase (i interaktivní), pull, fast-forward,
cherry-pick, revert, reword, sestavení commitů pomocí AI a znovu (redo). Okno
tato tlačítka a položky palety zašedí a uvede důvod. CLI a MCP dostanou chybu
`PendingAutoStash` (návratový kód 1). Vše ostatní zůstává povoleno: čtení,
stage, zahazování, commit, větve, tagy, remote, fetch, push, pokračování /
přeskočení / přerušení rebase, řešení konfliktů, `stash push`, `reset`
a `stash pop` / `apply` / `drop`. Povoleno je i Zpět.

Když auto-stash čeká na merge nebo rebase, který se zastavil na konfliktech,
vrátí se sám, jakmile ta operace skončí (v GitPoulpi, nebo s čistým gitem
a následným obnovením).
