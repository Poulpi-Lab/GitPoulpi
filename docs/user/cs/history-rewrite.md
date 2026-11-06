[English](../history-rewrite.md) · **Čeština** · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Přepis historie

Přepis historie mění commity, které už existují: kdo je napsal a kdy, jejich
zprávy, soubory, které obsahují, a jejich podpisy. GitPoulpi to dělá pomocí
**sady** (rozsah a seřazený seznam pravidel) a tří kroků, z nichž prostřední se
nikdy nepřeskakuje:

1. **Náhled** spočítá novou historii, aniž by se čehokoli dotkl: počty, co udělalo
   které pravidlo, větve a tagy, které by se přesunuly, jeden řádek na každý
   změněný commit.
2. **Provést** přesune vaše lokální větve a tagy na ni. Je to jeden záznam
   [historie vrácení zpět](undo-redo.md): **Vrátit zpět** vrátí každý přesunutý
   ref (i soubory v pracovním stromu) tak, jak byl.
3. **Push** (force push s lease) a **Purge** jsou samostatné, uvážené kroky, protože
   je nelze vrátit zpět: viz [Odeslání výsledku](#odeslání-výsledku)
   a [Smazání staré historie](#smazání-staré-historie-purge).

Nástroj pro přepis, paleta, příkazy `gitpoulpi rewrite` a nástroje MCP používají
stejný stroj, takže náhled vytvořený v jednom z nich provede kterýkoli jiný.
[Z příkazové řádky](#z-příkazové-řádky) vypisuje přepínače;
[Vytvořit pravidla pomocí AI](#vytvořit-pravidla-pomocí-ai) vyplní nástroj podle popisu;
[Návrhy od agentů AI](#návrhy-od-agentů-ai) říkají, co agent smí a nesmí.

**Pravidla podle toho, co mění**

| Skupina | Pravidla | Část |
|---|---|---|
| Autoři | Nahradit autora, Anonymizovat autory, Použít .mailmap | [Autoři a .mailmap](#autoři-a-mailmap) |
| Data | Posunout data, Změnit časové pásmo, Omezit na hodiny, Datum committera = datum autora | [Data](#data) |
| Zprávy | Hledat a nahradit, Smazat řádky, Přidat nebo odebrat trailer, Opravit kódování zpráv | [Zprávy](#zprávy) |
| Obsah souborů a cesty | Hledat a nahradit v souborech, Odstranit soubory, Odstranit velké soubory, Přesunout nebo přejmenovat cesty, Převést konce řádků, Odebrat příznak spustitelnosti, Převést kódování souborů | [Soubory a cesty](#soubory-a-cesty) |
| LFS | Převést soubory do LFS, Vrátit soubory z LFS | [Přesun souborů do Git LFS nebo z něj](#přesun-souborů-do-git-lfs-nebo-z-něj) |
| Podpisy | Odstranit podpisy (a nové podepsání při Provedení) | [Podpisy](#podpisy) |
| Skript | Lua, shell, Python | [Pravidlo Skript](#pravidlo-skript) |

**Než začnete**

- Náhled nic nemění. Provedení mění jen vaše lokální refy a nikdy nic neodesílá.
- Přepsané commity dostanou nová id. Všichni, kdo mají starou historii, se to
  musí dozvědět: poznámku napíše [Odeslání výsledku](#odeslání-výsledku).
- Tajná hodnota, která byla odeslána, je prozrazená. **Nejdřív ji zrušte a vydejte
  novou**, teprve potom přepište historii, proveďte force push a purge.
- Commity, které zůstanou, jak jsou, si ponechají svá id i podpisy.

## Nástroj pro přepis

`Ctrl+G Shift+H` (nebo **Přepis: Otevřít nástroj pro přepis historie** v paletě)
otevře nástroj pro přepis nad Logem. Nemá tlačítko na panelu nástrojů: pokročilé
akce najdete v paletě (každý řádek `Přepis: ...` ho otevře s už přidaným
pravidlem), v podnabídce **Přepsat historii** v nabídce commitu (*Od tohoto
commitu po HEAD...*, *Vybrané commity...*, když je vybráno víc commitů,
*Otevřít nástroj pro přepis...*) a za zkratkou.

`Ctrl+N` ukáže odznak u každého ovládacího prvku nástroje (rozsah, pravidla, pole, karty, tlačítka); stisk odznaku udělá to, co klik na ten prvek, a `Esc` odznaky odstraní (viz [Odznaky](keybindings.md#odznaky)).

![Pracovní stůl pro přepis historie: lišta rozsahu, seznam dvou pravidel Nahradit autora s poli vybraného pravidla a souhrn náhledu s počtem commitů, které se změní](../images/history-rewrite.png)

- **Lišta rozsahu:** které refy se přesunou (aktuální větev, všechny lokální větve,
  vybrané větve a tagy, vybrané commity), políčko *Tagy* a po *Od tohoto commitu
  po HEAD...* políčko, které začne za rodičem toho commitu.
- **Pravidla:** číslovaný seznam, pravidla se provádějí v pořadí. Políčko pravidlo
  zapne nebo vypne, šipky ho posouvají, křížek ho smaže (*Obnovit smazané pravidlo*
  nebo `Ctrl+Z` ho vrátí). *Přidat pravidlo* nabízí druhy podle skupin. Pravidlo,
  které potřebuje program, který nemáte, zůstane zašedlé i s důvodem.
- **Nastavení pravidla:** kliknutí na textové pole ho upraví; kliknutí na pole
  s více hodnotami otevře jejich seznam; políčka se přepínají.
- **Náhled** (`F5`) spočítá novou historii, aniž by se čehokoli dotkl, a ukazuje
  ukazatel průběhu. Karty: **Souhrn** (počty, co udělalo které pravidlo, varování),
  **Commity** (jeden řádek na změněný commit, stránkované z daemona; filtr
  nabízí změněné, odstraněné nebo odeslané), **Větve a tagy** (co se přesouvá
  a chráněné větve, které už jsou odeslané, každá s políčkem), **Dopad** (co
  znamená přepsání odeslané historie: force push s lease, spolupracovníci,
  podpisy), **Tajné hodnoty**, **Identity** a **Velikost** (viz níže).
- **Soubory commitu:** `Enter` (nebo dvojklik) na řádku v Commitech otevře
  soubory, které v něm pravidla obsahu změnila, a vedle nich diff vybraného
  souboru. Zavře ho křížek, `Esc` nebo kliknutí mimo. Odstraněné řádky pravidla
  pro tajné hodnoty jsou skryté.
- **Provést** (`Ctrl+S` nebo `Ctrl+Enter`) je dostupné, jakmile náhled odpovídá
  pravidlům. Otázky bez výchozí odpovědi se kladou pokaždé: co se stane s podpisy
  podepsaných commitů a jak se vyjme složka. Panel s výsledkem nabízí **Vrátit
  zpět**, které vrátí každý přesunutý ref tak, jak byl.
- **Uložit / Načíst** uloží sadu pravidel do repozitáře (sdílenou) nebo jen pro vás;
  tajné hodnoty se nikdy neukládají.

Každé tlačítko má klávesu: seznam je v Nastavení > Klávesnice (kontext „Nástroj
pro přepis historie“) a v příručce `keybindings.md`.

### Autoři a .mailmap

- **Nahradit autora** vybere autora nebo committera podle jména a/nebo e-mailu
  (doslovně, nebo regulárním výrazem) a nastaví nové jméno a/nebo e-mail. *Týká
  se* vybírá autora, committera, nebo oba; *Také autoři značek* zařídí, aby
  anotované tagy, které se znovu vytvářejí, šly s nimi. Původní committer
  zůstane, pokud běh neříká *já, teď*.
- **Anonymizovat autory** nahradí každou identitu očíslovaným zástupcem
  (`Contributor 1`, `contributor-1@anonymous.invalid`; oba vzory můžete změnit,
  `{n}` je číslo). Stejná osoba dostane v jednom běhu vždy stejné číslo;
  identity, které uvedete jako *ponechat*, zůstanou beze změny.
- **Karta Identity** (v paletě také **Přepis: Sloučit identity (.mailmap)...**) je
  způsob, jak opravit člověka, který commitoval pod více jmény a e-maily,
  **aniž by se cokoli přepisovalo**. Vypíše každé jméno a e-mail z rozsahu na liště
  rozsahu s počtem commitů, které napsal a které commitoval, a s těmi, které už
  `.mailmap` repozitáře mapuje (`mapováno na Jméno <e-mail>`). Zaškrtněte
  identity, které jsou jedna osoba (`Space` nebo klik; *Vybrat vše* /
  *Zrušit výběr*), pak **Sloučit do...** (`m`): dialog se zeptá, která z nich
  zůstane (první je ta s nejvíce commity), a **přidá do `.mailmap` jeden řádek
  za každou další identitu** (`Jméno <e-mail> Staré jméno <stary@e-mail>`; řádek,
  který tam už je, se neopakuje, a chybí-li soubor, vytvoří se). Nic se
  necommituje: soubor `.mailmap` commitněte sami, aby se sdílel. Log, blame
  i historie souboru ukážou kanonické jméno okamžitě (nastavení `log.use_mailmap`,
  ve výchozím stavu zapnuté; uložené commity se nikdy nemění, takže není co
  odesílat vynuceně). Stavový řádek řekne, co se zapsalo, a nabídne **Vrátit
  zpět** (`u`), které vrátí předchozí `.mailmap` (je to jeden záznam historie
  vrácení zpět jako každá jiná akce). `r` načte seznam znovu.
- **`.mailmap` místo přepisu.** Když pravidlo autora mění commity, které už jsou
  na remote, Souhrn řekne *Soubor .mailmap opraví zobrazení bez přepisování.
  Použít místo toho .mailmap?* a nabídne tlačítko **Použít .mailmap** (`m` na
  Souhrnu). Otevře kartu Identity se zaškrtnutými identitami, které pravidlo
  zasahuje, a identitu, kterou by pravidlo nastavilo, nabídne jako tu, která
  se ponechá, i když ji zatím žádný commit nenese.
- **Pravidlo `.mailmap`** (*Přepis: Použít .mailmap na historii...* nebo
  `rewrite preview --mailmap`) přepisuje doopravdy: autoři a committeři
  (podle vaší volby) dostanou jména, která jim dává `.mailmap` pracovního stromu,
  tedy to, co ukazuje `git log --use-mailmap`. Použijte ho, když má samotná
  historie nést kanonická jména, například před jejím zveřejněním; je to přepis
  jako každý jiný (náhled, Provést, force push).
- **Pravidla identity a trailery.** Pravidlo identity může změnit i osoby
  uvedené v trailerech: zaškrtněte **Mapovat i trailery** (`--map-trailers`).
  Každý trailer, jehož hodnota je přesně `Jméno <e-mail>` a odpovídá poli
  *E-mail je* / *Jméno je* z pravidla, dostane nové jméno a e-mail, ať pravidlo
  s autorem a committerem dělá cokoli. Ve výchozím stavu je vypnuto: `Signed-off-by`
  zaznamenává, kdo commit podepsal, takže jeho změna je samostatné rozhodnutí.

### Data

Čtyři pravidla pro data působí na datum autora, datum committera, nebo obě
(*Data* v panelu pravidla).

- **Posunout data** posune každé datum o určitou hodnotu (`+2h`, `-1d`, `90m`,
  `1d 2h`). Časové pásmo, v němž byl každý commit napsán, zůstane.
- **Změnit časové pásmo** zapíše data s jiným posunem od UTC. *Zachovat* říká,
  co zůstane: **okamžik** (stejný časový bod, hodiny ukazují jinak), nebo
  **hodiny** (stejný místní čas, okamžik se změní).
- **Omezit na hodiny** přesune každý commit, který leží mimo povolené hodiny
  (například `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`), na nejbližší povolený
  čas, nebo jen dřívější, nebo jen pozdější (*Přesunout na*). *Nikdy dříve než
  rodič* zachová pořadí historie; *Seed* dělá malý náhodný rozptyl opakovatelným:
  stejný seed nad stejnou historií dá stejná data.
- **Datum committera = datum autora** nastaví committera (jméno, e-mail a datum)
  na autora, což je to, co `git commit --amend` nebo rebase zrušily.

Data jsou pravidla jako ostatní: náhled, kontrola karty **Commity** (každý řádek
ukazuje staré a nové datum), pak provedení. Id commitu závisí na jeho datech,
takže pravidlo pro data změní každý commit po prvním, kterého se dotkne.

### Zprávy

- **Hledat a nahradit ve zprávách** a **Smazat řádky zpráv** berou doslovný
  text nebo regulární výraz (*Regulární výraz*, *Rozlišovat velikost písmen*);
  prázdná náhrada maže. Zprávy, které nejsou platné UTF-8, se jimi nikdy
  nemění: pokud je chcete upravovat, spusťte nejdřív **Opravit kódování zpráv**.
- **Trailery.** *Přidat nebo odebrat trailer* (*Přepis: Přidat nebo odebrat
  trailery...* v paletě, nebo `--trailer-add`, `--trailer-remove`,
  `--trailer-map`) upravuje řádky `Klíč: hodnota` na konci zprávy, které zná
  `git interpret-trailers` (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...).
  Rozbalovací seznam **Akce** vybírá *Přidat* (**Trailer** je klíč, jeho
  **Hodnota** a **Pokud už existuje**: *Přidat (pokud není shodný)*, *Nahradit
  stávající*, nebo *Commit nechat beze změny*), *Odebrat* (každý trailer toho
  klíče, nebo jen ty, jejichž hodnota obsahuje **Hodnota obsahuje**) nebo *Změnit
  hodnotu* (**Hledat v hodnotě** se změní na **Nahradit za**, například starý
  e-mail novým v `Signed-off-by`). Klíče se porovnávají bez ohledu na velikost
  písmen. Mění se jen řádky trailerů, zbytek zprávy zůstane přesně, jak byl,
  a zpráva, která zatím žádné trailery nemá, dostane prázdný řádek a vlastní
  blok. Karta Commity ukazuje každou změnu jako `+Signed-off-by: ...` /
  `-Co-authored-by: ...`. Poslední odstavec zprávy je její blok trailerů
  (nikdy ne titulek), takže řádek jako `Note: see the ticket` v těle trailerem
  není, pokud není sám v posledním odstavci spolu s dalšími trailery.
- **Id commitů ve zprávách.** **Aktualizovat id commitů** (`--update-message-hashes`,
  ve výchozím stavu vypnuto; na konci panelů pravidel, která upravují zprávy)
  zařídí, aby id zapsaná ve zprávách následovala přepis: zkrácené id o 7 a více
  hexadecimálních znacích (`This reverts commit 1a2b3c4`, `see 1a2b3c4d5e`),
  které pojmenovává přesně jeden commit přepsané historie, se nahradí novým id
  toho commitu, zkráceným na stejnou délku. Id, které odpovídá více commitům,
  žádnému, nebo commitu, který přepis vytvoří až později, zůstane, jak je; Souhrn
  řekne, kolik se jich nahradilo a kolik ponechalo.
- **Opravit kódování zpráv** (*Přepis: Opravit kódování zpráv...* nebo
  `--fix-message-encoding [--fix-name-encoding]`) převede zprávy do UTF-8,
  ve dvou případech:
  - Commit s hlavičkou `encoding` (`ISO-8859-1`, `Windows-1252`, `Shift_JIS`,
    cokoli, co git a tabulky kódování znají) se dekóduje a zapíše jako UTF-8
    a hlavička zmizí: text se všude čte stejně, bez jakéhokoli převodu. Jména
    autora a committera toho commitu byla ve stejném kódování a dekódují se s ním,
    takže commit nikdy nezůstane napůl převedený.
  - **Mojibake**, tedy UTF-8, které bylo kdysi přečteno jako Latin-1 nebo
    Windows-1252 (`RÃ©Ã©criture` místo `Réécriture`), se opraví **jen tehdy, když
    převod tam a zpět přesně sedí**: text se musí zakódovat zpět na bajty, které
    jsou platné UTF-8, a stejné poškození výsledku musí dát původní text. Zpráva,
    která jen obsahuje `Ã©` vedle skutečných písmen nebo znaky mimo stará
    abecedy, tím neprojde a zůstane, jak je.

  *Opravit i jména* (`--fix-name-encoding`) opraví stejným způsobem poškozená
  jména autorů a committerů. Co nelze určit (neznámý název kódování, bajty, které
  v deklarovaném kódování nejsou platné, zpráva, která není UTF-8 a nic
  nedeklaruje, text, který vypadá poškozeně, ale není to prokazatelné), se
  **ponechá a započítá**: Souhrn řekne *n zpráv zůstalo beze změny: jejich
  kódování nelze určit* a číslo je v `counts.encoding_left`. Karta Commity ukazuje
  každou opravu se starou a novou zprávou a řádkem *kódování* (`ISO-8859-1` se
  změní na `UTF-8`).

### Soubory a cesty

- **Pravidla pro obsah souborů** (*Hledat a nahradit v souborech*, *Odstranit
  soubory*, *Odstranit velké soubory*): panel kontroluje každou hodnotu při
  psaní a důvod ukáže pod polem (vzor, který nejde zkompilovat, špatný vzor cesty,
  velikost pod jeden bajt). Velikosti berou jednotky (`500 KiB`, `5 MiB`, `1 GiB`).
  Vzor cesty bez lomítka odpovídá v libovolné hloubce. *Tajné hodnoty* se zadávají
  do maskovaného pole, použijí se jen pro daný běh a nikdy se neukládají; náhled
  skryje řádky, které je obsahovaly. *Prázdné commity* určuje, zda se commit,
  který pravidla nechala prázdný, odstraní (výchozí), nebo ponechá. Binární soubory
  a soubory nad limitem velikosti zůstanou beze změny a v Souhrnu se započítají.
- **Přesun cest.** *Přesunout nebo přejmenovat cesty* (*Přepis: Přesunout nebo
  přejmenovat cesty...* v paletě) má tři druhy, které vybíráte v seznamu
  **Přesunout**:
  - *Přejmenovat cestu* (**Z**, **Do**; `--move-path FROM=TO`): stejné soubory
    a složky se objeví na nové cestě v každém commitu, který měl tu starou;
    nic jiného se nemění a každý soubor si ponechá své id. Commit, kde nová cesta
    už existuje, se nikdy nepřepíše: náhled se odmítne a vypíše commity a cesty,
    které kolidují (zvolte volný název, nebo nejdřív přesuňte existující cestu).
    Jen přesné cesty, žádné zástupné znaky.
  - *Přesunout vše do složky* (**Složka**; `--into-subdir DIR`): každá cesta
    každého commitu dostane předponu. Hodí se ke slučování repozitářů bez kolizí
    názvů.
  - *Vyjmout složku jako kořen* (**Složka**; `--extract DIR`): složka se stane
    kořenem, commity, které se jí nedotýkají, se zahodí (commit, který ji vytváří,
    nemá rodiče), merge, které už nic nespojují, se zjednoduší a větve a tagy,
    které z té složky nic nemají, se smažou. **Pokaždé** se zeptáme, jak to
    skončí:
    - *Do nového klonu, otevřeného na nové kartě* (`--extract-mode clone`):
      v zvolené složce se vytvoří kopie tohoto repozitáře (nový repozitář dostane
      uvnitř složku pojmenovanou po vyjmuté složce), tam se přepíše a vyčistí,
      aby obsahoval jen novou historii, každou lokální větev a tag, které tu složku
      mají. Tento repozitář se nemění. Nový repozitář nemá žádný remote (originál
      není jeho upstream).
    - *Na místě: přepíše se historie tohoto repozitáře* (`--extract-mode
      in-place`): obvyklý přepis, náhled, Provést, vrácení zpět. Větve bez té
      složky se smažou (vrácení zpět je vrátí); přepnutá větev ji mít musí.
    - *Jako submodul tohoto repozitáře* (`--extract-mode submodule`): nový
      repozitář se vytvoří jako u klonu a pak **jeden nový commit** na aktuální
      větvi nahradí složku jejím submodulem (`.gitmodules` a odkaz); složka
      zůstane jako přepnutý submodul. Historie tohoto repozitáře se **nepřepisuje**:
      jeho staré commity si soubory té složky ponechají, takže se nic, co na ně
      ukazuje (tagy, jiné klony), nerozbije. URL v `.gitmodules` je lokální cesta
      k novému repozitáři: až bude na hostiteli, změňte ji (`git config -f
      .gitmodules submodule.<složka>.url`). Upravené nebo nesledované soubory ve
      složce to odmítnou. Vrácení zpět vrátí obyčejnou složku.

    Souhrn řekne, že náhled ukazuje historii nového repozitáře a do které složky
    půjde. Pravidla, která následují po vyjmutí (přejmenování, odstranění), se
    provedou na vyjmutém stromu; u klonu nebo submodulu musí být vyjmutí prvním
    pravidlem pro cesty. Pravidla pro cesty běží po pravidlech pro obsah, takže
    glob pro odstranění pojmenuje cesty tak, jak byly.

#### Konce řádků, příznak spustitelnosti a kódování souborů

Tři pravidla ve skupině **Obsah** nabídky Přidat (nebo řádky palety *Přepis:
Převést konce řádků...*, *Odebrat příznak spustitelnosti...* a *Přepis: Převést
kódování souborů...*). Všechna tři berou **Soubory** (globy; vzor bez lomítka
odpovídá v libovolné hloubce; prázdné znamená každý soubor) a **Kromě souborů**.
Mění jen to, co globy pojmenují, v každém commitu rozsahu. Binární soubory
(bajt NUL v prvních 8 KiB) se nikdy nemění: Souhrn je započítá jako *zůstaly
beze změny*.

- **Převést konce řádků** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Převést na*: LF (Linux, macOS), CRLF (Windows) nebo
  CR (klasický Mac). Každý konec řádku v odpovídajícím textovém souboru se změní
  na tento, takže soubor se smíšenými konci vyjde jednotný; soubory, které už
  jsou správně, si ponechají svou verzi. *Řídit se .gitattributes* (ve
  výchozím stavu vypnuto, `--eol-follow-attributes`) čte kořenový `.gitattributes`
  každého commitu: soubory označené `-text` nebo `binary` se přeskočí a soubory
  označené `eol=lf` nebo `eol=crlf` si ponechají tento konec místo toho, který
  jste zvolili.
- **Odebrat příznak spustitelnosti** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Odpovídající soubory přejdou z režimu 100755 na 100644; jejich
  obsah se nemění. Je potřeba alespoň jeden vzor souborů: odebrání příznaku
  každému spustitelnému souboru by rozbilo vaše skripty.
- **Převést kódování souborů** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Převést na*: UTF-8 (výchozí), UTF-8 s BOM, UTF-16 LE nebo
  BE (zapsané s BOM), Windows-1252 nebo ISO-8859-1. *Soubory jsou v kódování*:
  ve výchozím stavu se kódování každého souboru zjistí (BOM, jinak platné UTF-8,
  jinak *záložní*, Windows-1252, pokud ho nezměníte); pojmenováním kódování ho
  vynutíte, ale špatná volba dá špatný text (UTF-8 přečtené jako Latin-1
  zdvojí každou diakritiku). Soubor, který zdrojové kódování nepřečte, nebo
  který obsahuje znak, jejž cíl nepojme (znak eura v ISO-8859-1), zůstane přesně
  tak, jak je, a v Souhrnu se započítá.

Pravidla běží v pořadí seznamu, na stejném souboru: převod souboru do UTF-16
a následná žádost o konce řádků ho nechá být (už to není prostý text). Provedení
je jeden záznam v seznamu vrácení zpět jako každý přepis.

### Hledání tajných hodnot

- **Hledání tajných hodnot:** `gitpoulpi rewrite scan-secrets` (a nástroj MCP
  `rewrite_scan_secrets`) projde historii jednou a vypíše předpony tokenů
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), soukromé
  klíče, JWT, hesla v URL a řádky `password = hodnota`, které vypadají náhodně,
  každý se souborem, prvním a posledním commitem a zakrytým náhledem, třeba
  `ghp_****abcd`. Není vyčerpávající a hodnota se nikdy nezobrazí, nezaloguje
  ani neuloží. `--remove-finding ID` změní nález v pravidlo, které ho odstraní
  z každého commitu. Tajná hodnota, která už byla odeslána, je prozrazená:
  nejdřív ji zrušte a vydejte novou, teprve potom přepište historii, proveďte
  force push a purge.
- **Karta Tajné hodnoty** dělá totéž přímo v nástroji pro rozsah z lišty rozsahu:
  **Hledat tajné hodnoty** (`s`) spustí prohledávání jako úlohu (ukazatel průběhu
  se jmenuje *Prohledávání*, zastaví ho **Zrušit** nebo `Esc`). Každý nález je
  řádek s druhem, zakrytým náhledem, souborem a počtem dalších souborů, které
  ho obsahují; pod seznamem je první a poslední commit, v němž byl vidět, a kolik
  commitů a verzí souborů ho drží. Karta uvádí, kolik binárních a příliš
  velkých souborů se nečetlo, a že prohledávání není vyčerpávající. Všechny
  nálezy jsou zpočátku zaškrtnuté; **Odstranit z historie** (`r`) z těch
  zaškrtnutých vytvoří jedno pravidlo, které je jmenuje podle id (nikdy podle
  hodnoty) a které si prohlédnete a provedete jako každé jiné. Po přepisu nebo
  změně rozsahu nálezy zastarají: karta to řekne, prohledejte znovu. Nástroj
  hodnotu tajemství nikdy nedrží, takže ji ani nemůže zobrazit.

### Hledání toho, co váží

- **Hledání toho, co váží:** `gitpoulpi rewrite analyze [--top N]` (a nástroj MCP
  `rewrite_analyze`) je analýza velikosti jako `filter-repo --analyze`: největší
  soubory s cestou, velikostí, prvním a posledním commitem a údajem, zda jsou
  ještě v HEAD, největší složky, soubory smazané z HEAD, které historie stále
  uchovává, a součty. Čte jen velikosti a cesty, nikdy obsah souborů, a nic nemění.
- **Karta Velikost** dělá totéž z nástroje (`F7` nebo *Analyzovat velikosti
  historie* v paletě ji otevře a spustí): **Analyzovat velikosti** (`s`) je úloha
  (*Analyzování*, zastaví ji **Zrušit** nebo `Esc`). **Zobrazit** vybere seznam
  (`v`): *Soubory* (největší unikátní verze souborů, každá označená *v HEAD*,
  *stará verze* souboru, který tam stále je, nebo *smazáno*), *Složky* (vše, co
  bylo kdy uloženo pod každou z nich) nebo *Smazané* (cesty, které z HEAD zmizely
  a historie je drží). **Řadit** (`o`) podle velikosti, cesty, nebo smazané
  první. Zaškrtněte řádky (`Space`, `Alt+A` / `Alt+U` pro všechny / žádné): karta
  řekne, kolik drží, poctivě jako „až asi“, protože místo se uvolní až po
  přepisu, vymazání starých objektů a `gc`. **Odstranit z historie** (`r`)
  z vybraných cest vytvoří pravidlo pro cesty (speciální znaky se escapují, cesta,
  která už v pravidle je, se nepřidá dvakrát); **Odstranit každý soubor nad ...**
  (`b`, seznam Soubory) napíše pravidlo velikosti těsně pod nejmenším zaškrtnutým
  souborem. Prohlédněte si je a proveďte jako každé jiné pravidlo. Seznamy ukazují
  200 největších řádků; CLI řekne, kolik jich je navíc, a `--top` je vypíše. Po
  přepisu nebo změně rozsahu řádky zastarají: analyzujte znovu.

### Podpisy

- **Podpisy:** přepsaný commit si nemůže ponechat svůj podpis (podepisoval staré
  bajty), takže když se přepisují podepsané commity, zeptáme se pokaždé:
  *Odstranit podpisy* (nejdřív se ukáže počet), *Znovu podepsat jen dříve
  podepsané commity*, nebo *Znovu podepsat každý přepsaný commit*. Výchozí
  odpověď není. Commity, které zůstanou, jak jsou, si podpisy ponechají. Nové
  podepsání používá vaše vlastní nastavení, stejně jako podepsaný commit:
  `gpg.format` (`openpgp`, `ssh` nebo `x509`), `user.signingkey` a `gpg.program` /
  `gpg.ssh.program` / `gpg.x509.program`. Čtvrtý řádek otázky, **Zkontrolovat
  podepisování** (`4`), jednou podepíše zkušební data a ověří je: řekne, jak
  dlouho trvá jeden podpis, a odhadne čas běhu pro počet commitů, na které se
  ptáme, nebo proč to nefunguje (chybí klíč, uzamčený agent, nenainstalovaný
  program, spolu s návodem k instalaci). Náhled dělá stejnou zkoušku dřív, než
  čte historii, a s tímto důvodem se odmítne, takže rozbité nastavení nikdy
  nenechá napůl podepsaný přepis. Commity se podepisují **během náhledu** (jejich
  id obsahují podpis), takže náhled, který znovu podepisuje, trvá o dobu
  podepisování déle, Souhrn řekne *Znovu podepsané commity: n, za 12,5 s*
  a může se otevřít výzva k zadání hesla: nejdřív odemkněte svého agenta
  (Zkontrolovat podepisování dělá přesně to). **Zrušit** zastaví mezi dvěma
  podpisy a nic po sobě nezanechá. Znovu se vytvářejí jen podpisy commitů;
  podepsaný tag o podpis stále přijde.
- **Pravidlo *Odstranit podpisy*** (nabídka Přidat, skupina *Podpisy*; *Přepis:
  Odstranit podpisy...* v paletě; `--remove-signatures`) odstraní podpis každého
  commitu v rozsahu, i tam, kde se jinak nic nemění, a na nové podepsání se
  nezeptá. Z CLI: `rewrite check-signing` a
  `--signatures strip|resign-signed|resign-all`.

### Vrácení přepisu zpět

**Vrátit zpět** (tlačítko panelu s výsledkem, tlačítko *Zpět* na panelu nástrojů
s `Ctrl+Z` v Logu, nebo `gitpoulpi undo`) vrátí každou přesunutou větev, tag
a poznámku, a s nimi i soubory v pracovním stromu, ze starých commitů, které
přepis drží. Zeptá se nejdřív, když je přepsaná větev už na svém upstreamu
(lokální vrácení push nevrací), a když máte necommitnuté změny: ty se ponechají
a soubor, který jste změnili a který by stará historie přepsala, vrácení místo
toho zastaví (soubory se přesouvají tak, jak je přesouvá `reset --keep`). Vrácení
zpět přestane být u přepisu možné, jakmile jeho starou historii
[vymažete](#smazání-staré-historie-purge); seznam historie ho pak označí
*vymazáno*.

## Přesun souborů do Git LFS nebo z něj

Dvě pravidla ve skupině *LFS* nabídky Přidat přepíšou historii tak, aby velké
soubory žily v [Git LFS](lfs.md) od začátku (nebo ho opustily). Potřebují program
`git-lfs`: bez něj jsou pravidla zašedlá s návodem k instalaci a náhled se
odmítne. Je to náš vlastní stroj, ne `git lfs migrate`, takže dostanete náhled,
rozsah, vrácení zpět i pojistku při push jako u každého jiného pravidla.

- **Převést soubory do LFS** (*Přepis: Převést soubory do LFS...* nebo
  `--lfs-import PATTERN` s `--lfs-min-size BYTES`). Každý soubor, který odpovídá
  vzorům (`*.psd`, `assets/**`; vzor bez lomítka odpovídá v libovolné hloubce)
  a je alespoň tak velký, jak říká *Jen soubory od velikosti* (prázdné: každý
  odpovídající), se v každém commitu, který ho obsahuje, stane ukazatelem LFS.
  Jeho obsah jde při Provedení do lokálního úložiště LFS (`.git/lfs/objects`,
  jednou pro každý odlišný obsah) a kořenový `.gitattributes` dostane
  `<vzor> filter=lfs diff=lfs merge=lfs -text` v každém přepsaném commitu, který
  ukazatel drží (vytvoří se, když žádný není, ostatní řádky zůstanou a nikdy
  se neduplikují). Soubor, který už je ukazatelem, zůstane. Souhrn řekne
  *3 soubory (12 MiB) se přesunou do LFS. Objektů LFS k zápisu: 3.* a karta
  Commity vypíše u každého commitu *přesouvá big.psd do LFS*.
- **Vrátit soubory z LFS** (*Přepis: Vrátit soubory z LFS...* nebo
  `--lfs-export PATTERN`). Ukazatele, které odpovídají, se znovu stanou
  skutečnými soubory a řádky `.gitattributes` pro tyto vzory se odstraní.
  Objekty musí být v lokálním úložišti LFS: když některé chybí, náhled se
  odmítne se seznamem a tlačítkem **Stáhnout objekty LFS**, které je stáhne
  z `origin`; pak spusťte náhled znovu. Import následovaný exportem stejných
  vzorů vrátí úplně stejné commity.

Po Provedení jsou soubory v pracovním stromu skutečný obsah (ne ukazatele)
a **Vrátit zpět** vrátí starou historii. Panel s výsledkem má **Odeslat objekty
LFS** (`l`): totéž jako `LFS: Push All Objects`, takže novou historii lze odeslat,
aniž by pozdější push uvízl na chybějících objektech (obyčejný push přepsaných
větví je nahraje také přes hook pre-push, který Provedení nainstaluje). Poznámka
pro spolupracovníky říká, že potřebují git-lfs a `git lfs pull`.

Dobré vědět: Provedení nainstaluje filtry LFS repozitáře (`git lfs install
--local`), pokud chybějí, protože git je potřebuje k porovnání skutečných
souborů s ukazateli. Zapisuje se jen kořenový `.gitattributes`, nikdy vnořený.
Soubor *pod* minimální velikostí, který vzoru odpovídá, zůstane obyčejným
souborem, ale git ho bude při dalším přidání chtít změnit na ukazatel, takže
`git status` ho ukazuje jako změněný, dokud ho necommitnete. Import velké
soubory streamuje, export drží v paměti vždy jen jeden.

## Odeslání výsledku

Provedení přepisu nikdy nic neodesílá. Přepsané větve a tagy, které byly na
remote, potřebují force push, a ten nikdy nevynucuje naslepo: každý ref se odešle
s lease na id, které remote mělo při náhledu (ne podle toho, co říká vaše
sledovací větev teď), takže push kolegy mezitím je u tohoto refu odmítnut a nepřepíše
se. Refy jdou jedním push typu všechno, nebo nic (`--atomic`), pokud to server
dovoluje. Z příkazové řádky:

```
gitpoulpi rewrite push --dry-run        # zeptá se remote, nic neodešle
gitpoulpi rewrite push --yes            # odešle
gitpoulpi rewrite note                  # co mají ostatní spustit
gitpoulpi rewrite maps --out ./maps     # commit-map a ref-map
```

Chráněné větve (nastavení `rewrite.protected_branches`, výchozí `main`, `master`,
`develop`, `release/*`) se při odesílání nechají být, a to jak při přepisu, tak
znovu při push, pokud je nejmenujete (`--include-protected main`) nebo nezahrnete
všechny. O tom, zda force push přijme, stále rozhoduje vlastní ochrana větví
hostitele; jeho odmítnutí se ukáže u každého refu.

V nástroji to všechno dělá panel s výsledkem po Provedení:

- **Odeslat přepsané refy na remote** vypíše refy, které přepis přesunul a které
  remote má, každý s políčkem (`Space`). Chráněné větve začínají nezaškrtnuté
  a je u nich *chráněný*: zaškrtněte je po jedné, nebo použijte *Vybrat vše* /
  *Zrušit výběr* (`Alt+A` / `Alt+U`). **Push přepsané větve** (`p`) odešle ty
  zaškrtnuté; odpověď je jeden řádek na ref (*odesláno*, *už je aktuální*,
  *odmítnuto: remote se od náhledu posunul*, *odmítnuto: chráněno hostitelem*,
  *přeskočeno: chráněná větev*, *neodesláno*). Odmítnutý řádek má **Fetch
  a zkontrolovat** (`f`): nástroj se zavře a remote se načtou, takže Log ukáže,
  co ostatní odeslali. Nic se nepřepíše: lease chrání jejich práci a před dalším
  pokusem znovu spustíte náhled.
- **Kopírovat poznámku** (`c`) dá poznámku pro spolupracovníky do schránky, v jazyce
  rozhraní s git příkazy ponechanými v angličtině.
- **Otevřít mapu commitů** (`m`) ukáže, kde je mapa commitů a mapa refů; klik na
  cestu (nebo `1` / `2`) ji zkopíruje.

Push nelze vrátit zpět: když přepis potom vrátíte příkazem `undo`, změní se jen váš
repozitář, a vrácení to řekne dřív, než se zeptá. Pošlete poznámku všem, kdo
mají starou historii: uvádí starý a nový konec každé větve a tagu a příkazy
(`git fetch`, pak `git reset --hard origin/<branch>` nebo `git rebase --onto`).
Obě mapy se u běhu uchovávají 24 hodin; zkopírujte si je, pokud je potřebuje
čištění repozitáře v GitLabu nebo nějaký skript.

## Když hostitel chrání větev

Váš hostitel může větev chránit sám (ochrana větví a rulesets na GitHubu,
chráněné větve na GitLabu, politiky větví v Azure DevOps, omezení větví
v Bitbucketu) a odmítnout force push, který přepis potřebuje. Když je repozitář
**propojený** se svým hostitelem (sekce pull requestů v postranním panelu, viz
[Hostingové služby a issue trackery](hosting-integration.md)), nástroj se hostitele zeptá,
které větve chrání, s vaším vlastním tokenem:

- Tyto větve jsou **ve výchozím stavu z přepisu vynechány**, přesně jako chráněné
  větve z `rewrite.protected_branches` a z workflow: karta Refs je vypíše
  s políčkem u každé a tlačítky *Vybrat vše* / *Zrušit výběr* a řekne proč:
  *chráněno na GitHub*.
- Když token ochranu přečíst nemůže (chybí mu oprávnění, nebo nejste
  správce), nástroj řekne *Ochrana u hostitele neznámá* a nic nepovažuje za
  chráněné na slovo hostitele. Přepis to nikdy neblokuje. Repozitář, který
  není propojený, se nikdy neptá.

Když chráněná větev překáží (náhled se odmítl, protože je chráněná každá větev,
nebo hostitel push odmítl: *odmítnuto: chráněno hostitelem*), tlačítko **Jak to
obejít** (`Alt+H`) vysvětlí tři cesty ven, s přesnou stránkou nastavení vašeho
poskytovatele a repozitáře (odkaz vyberte a zkopírujte ho klávesou `c`, nebo ho
otevřete klávesou `o`). GitPoulpi nic z toho nikdy neudělá za vás:

1. **Dočasně zrušit ochranu** v nastavení hostitele, provést push a pak ji vrátit.
2. **Odeslat přepis pod novým názvem větve**, pak z této větve udělat výchozí
   větev (nebo přejmenovat) a starou smazat, až ji nikdo nebude používat.
3. **Jako poslední možnost smazat vzdálený repozitář a vytvořit ho znovu**
   a odeslat vše. Tím přijdete o pull / merge requesty a jejich recenze, issues,
   propojení forků se starou historií, hvězdičky a sledující, nastavení CI, tajné
   hodnoty a proměnné a webhooky a deploy klíče.

Z CLI se stejný text vrací jako `protected_by_host` v JSON příkazů `rewrite
preview` / `rewrite run` (když hostitel chrání větev z rozsahu), odmítnutého
`rewrite push` a `rewrite note`: `provider`, `branches`, `ways` (každá se svými
`steps` a `links`), `recreate_loses` a celé vysvětlení jako `text`.

## Smazání staré historie (purge)

Po přepisu jsou staré commity v repozitáři pořád: vrácení přepisu zpět je drží
(právě to ho dělá vratným) a drží je i reflogy. Pokud jste přepisovali kvůli
tajné hodnotě, ta je stále ve složce `.git`. Purge starou historii smaže nadobro:

```
gitpoulpi rewrite purge                 # zeptá se: vypíše, co se smaže a ztratí (návratový kód 3)
gitpoulpi rewrite purge --yes           # provede to
```

Zahodí pojistky pro vrácení zpět u přepisu, vyprší záznamy reflogu, které uvádějí
staré commity, spustí `git gc --prune=now` a pak ověří, že je každý starý commit
a každá verze souboru opravdu pryč. Potvrzení předem řekne, o co přijdete:
**vrácení zpět už nebude možné** u přepisu (a u každé dřívější operace, jejíž
vrácení také potřebovalo staré commity); zůstanou v seznamu historie označené jako
vymazané. Purge nelze po spuštění zastavit ani vrátit zpět. Remote se nedotkne.

Cokoli, co starou historii stále drží, purge zablokuje a pak se nic nezmění.
Seznam jmenuje každou věc a co s ní: větev nebo tag, které nebyly součástí
přepisu, stash na staré historii (zahoďte ho, nebo ho ponechte a smiřte se
s tím, že stará historie zůstane), jiný worktree, jehož HEAD na ní je, a sledovací
větve remote, které na starou historii stále ukazují (odešlete přepsané větve,
pak proveďte fetch). Váš seznam stashů a reflogy jiných větví se nikdy
nemění.

Tajná hodnota, která už byla odeslána, je prozrazená: **nejdřív ji zrušte a vydejte
novou**. Po purge odpověď vypíše, co ještě potřebuje hostitel, pro každý remote:
GitHub drží staré commity dostupné v zobrazeních z mezipaměti a v refech pull
requestů (kontaktujte GitHub Support, nebo repozitář smažte a vytvořte znovu),
GitLab má čištění repozitáře, které bere `commit-map` (`gitpoulpi rewrite maps
--out DIR`), plus údržbu, Azure DevOps a Bitbucket potřebují žádost o podporu
nebo nový repozitář a všichni, kdo mají klon nebo fork, mají starou historii dál.

### Obrazovka purge

V nástroji má panel s výsledkem přepisu tlačítko **Vymazat starou historii...**
(klávesa `e`) a paleta má **Přepis: Vymazat starou historii...** (pracuje s nejnovějším
přepisem, který ještě není vymazaný, i když jste nástroj mezitím zavřeli). Obrazovka
vypíše, co purge smaže a o co přijdete (operace, jejichž vrácení zpět zaniká
spolu s ním; ty označené *přijde i o vrácení zpět* nejsou přepisy, ale potřebovaly
staré commity), potom co ještě drží starou historii, vždy jeden řádek:

- sledovací větev remote: **Fetch** je aktualizuje z remote;
- stash: **Zahodit** se nejdřív zeptá, protože stash je práce, kterou jste
  nikdy necommitovali;
- větev, tag, worktree nebo jiný ref: **Kopírovat název**, pak to vyřešte tam,
  kde to žije (větev přepište nebo smažte, v worktree přepněte na přepsanou větev).

Kontrola se po každé opravě spustí znovu a **Zkontrolovat znovu** (`r`) ji spustí na
požádání. **Vymazat** (`p`) je dostupné, jen když nic nezbývá. Položí poslední
otázku, číslovanou a bez výchozí odpovědi (`1` smaže nadobro, `2` nechá vše tak,
jak bylo), a pak se spustí; obrazovka ukazuje krok, ve kterém je, a říká, že ji
nelze zastavit. Výsledek řekne, že stará historie je z tohoto klonu pryč, nebo
kolik starých objektů tam ještě je. Pod ním jsou kroky, které je třeba udělat
na každém remote (odkaz lze zkopírovat). Křížek, `Esc` a klik mimo obrazovku
zavřou, kromě doby, kdy purge běží.

Přes MCP nástroj pro purge neexistuje, dokud v tabulce `[mcp]` souboru
`config.toml` nenastavíte `allow_history_purge = true`.

## Pravidlo Skript

*Přidat pravidlo > Skript* (nebo **Přepis: Spustit skript...** v paletě) přidá
pravidlo se skriptem v Lua a otevře jeho editor nad nástrojem; `Enter` na řádku
*Skript* pravidla ho otevře znovu. Editor začíná z malé šablony.

- **Jazyk** (řádek v panelu pravidla): *Lua (vestavěné)*, *Shellový skript*,
  *Python* nebo *Lua (nainstalované)*. Cokoli kromě vestavěné Lua je
  [externí skript](#externí-skripty-shell-python-lua): bez izolace, a jazyk,
  který je v nastavení vypnutý nebo jehož interpret chybí, je v seznamu zašedlý
  s důvodem vedle sebe (jeho výběr řekne, co dělat). Změna jazyka u skriptu, který
  jste ještě nepsali, vymění šablonu.
- **Skript** je textový editor s čísly řádků. Co napíšete, je okamžitě pravidlo;
  nic se nepotvrzuje.
- Pod ním řádek **kontroly** říká `OK: funkce rewrite_commit nalezena`, nebo vypíše
  problémy s jejich řádkem (`script:3: ...`; číslo je na okraji červené).
  Spustí se chvíli poté, co přestanete psát, nikdy při držené klávese.
- **Parametry** mají jeden řádek na každý řádek `--! param` v hlavičce skriptu,
  s popiskem vedle pole: text, číslo, nebo políčko pro ano/ne. Hodnota, kterou
  vymažete, se vrátí na výchozí z hlavičky; textový parametr bez výchozí hodnoty
  se musí před Náhledem vyplnit.
- **Soubor** (cesta uvnitř repozitáře, na začátek `.gitpoulpi/rewrite/script.lua`)
  s **Načíst ze souboru...** (nahradí text) a **Uložit do souboru...**.
- **Vyzkoušet na výběru** spustí skript na commitech vybraných v Logu, nebo na
  20 nejnovějších, když není vybrán žádný, a nic nezapisuje. Tabulka vypíše každý
  commit (krátké id), každé pole, které by změnil, jako `staré -> nové`,
  a řádky `log(...)` ze skriptu. **Zrušit** přestane čekat. Když skript selže,
  ukáže se commit a důvod.
- Křížek, klik mimo editor a `Esc` ho zavřou; `Tab` a `Shift+Tab` přecházejí mezi
  editorem, parametry, souborem, tlačítky a výsledky; `F5` zkouší.

Náhled odmítne skript, který se nezkompiluje: panel Náhledu řekne
*skript selhal* s řádkem (nebo u selhání na commitu s jeho krátkým id a důvodem).
V sestavení bez skriptů zůstane druh v nabídce Přidat, zašedlý.

## Napsat skript pomocí AI

Se zapnutou AI ([Pomoc AI](ai.md)) má editor skriptu tlačítko `✦ Napsat pomocí AI...`
a paleta má `✦ Přepis: Napsat skript pomocí AI...`, které otevře nástroj na novém
pravidle Skript s otevřeným dialogem. Před stisknutím **Odeslat** se nic neodešle:
dialog při psaní ukazuje přesný text, který odejde.

- **Co má skript dělat?** Vlastními slovy: *posunout každý commit vytvořený v noci
  na následující ráno*, *nahradit můj starý e-mail novým, jen před rokem 2024*.
- **Odeslat vzorek N commitů** (vypnuto): po zaškrtnutí se přidá autor, datum
  a předmět commitů, které by použilo „Vyzkoušet na výběru“ (váš výběr v Logu, jinak
  20 nejnovějších), aby model viděl, jak vaše data vypadají. Nikdy obsah souborů ani těla
  commitů. S vypnutým políčkem žádná data o commitech počítač neopustí.
- **Co se odesílá** je panel jen pro čtení na o něco tmavším pozadí, při otevření dialogu
  sbalený (dialog je pak nízký). Zobrazíte ho kliknutím na jeho záhlaví, klávesou `Alt+W`,
  nebo přejdete `Tab` a stisknete `Enter`: vypíše pokyny (pevný kontrakt skriptu, pak vaše
  pokyny pro AI) a váš požadavek a má vlastní posuvník (přetáhněte ho, použijte kolečko myši
  nad panelem, nebo šipky, `PageUp`, `PageDown`, `Home` a `End`, dokud má fokus). Profil,
  který je dostane, je na řádku nad ním. Dialog se zvětší, aby panel pojal, v mezích obrazovky.
- **Odeslat** (`Tab` na něj, `Enter`). Zatímco model pracuje, dialog čeká a `Esc`
  požadavek zruší. Když selže, dialog zůstane s důvodem, abyste to mohli zkusit znovu.

Odpověď zkontroluje stejná kontrola daemona jako ručně napsaný skript; chyba kompilace se
modelu vrátí jednou. Skript pak **nahradí text v editoru** (dialog to řekne, když
editor obsahuje něco jiného než úvodní šablonu) a titulek editoru zní
*✦ napsáno AI: před náhledem si ho přečtěte*, dokud text nezměníte. Nikdy se za vás
nevyzkouší, nezobrazí v náhledu ani neprovede: přečtěte si ho, stiskněte **Vyzkoušet na výběru**,
pak Náhled jako u každého skriptu. Skript běží ve stejné izolaci jako ten, který jste napsali
(žádné soubory, hodiny ani síť).

Z příkazové řádky: `gitpoulpi ai rewrite-script "<instrukce>" [--sample N]` vypíše
skript a výsledek `rewrite script check` nad ním jako JSON a nic nespouští (návratový kód 2,
když se stále nezkompiluje). Použije se profil *Skript pro přepis historie* z Nastavení > AI.

## Vytvořit pravidla pomocí AI

Se zapnutou AI ([Pomoc AI](ai.md)) má nástroj tlačítko `✦ Vytvořit pravidla pomocí AI...`
vedle Uložit a Načíst, klávesu `Alt+I` (*Vytvořit pravidla přepisu pomocí AI* v Nastavení >
Klávesnice, kontext „Nástroj pro přepis historie“) a paleta má `✦ Přepis: Vytvořit pravidla pomocí AI...`.
S vypnutou AI, nebo v sestavení bez AI, tlačítko ani řádek palety nejsou. Vlastními slovy popíšete,
jak se má historie přepsat (*nahradit můj starý e-mail a@b.org adresou c@d.org*, *odstranit
secrets.txt z celé historie*, *posunout má data o dvě hodiny*), a model navrhne pravidla
a nastavení, která **vyplní nástroj**. Před stisknutím **Odeslat** se nic neodešle: dialog při
psaní ukazuje přesný text, který odejde.

- **Jak se má historie přepsat?** Víceřádkové pole: `Enter` je nový řádek,
  `Ctrl+Enter` nebo `Ctrl+S` odešle, `Esc` a křížek zruší.
- **Odeslat rozsah, názvy větví a tagů a autory historie** (vypnuto): po zaškrtnutí se
  přidá rozsah z lišty rozsahu, názvy větví a tagů a různí autoři (jméno a e-mail)
  nejnovějších commitů, které Log ukazuje (nejvýše 20 commitů), aby model mohl váš e-mail
  přesně pojmenovat (`ai rewrite-rules --context` na příkazové řádce místo toho čte 200
  nejnovějších commitů). Nikdy obsah souborů ani zprávy commitů. S vypnutým políčkem počítač
  opustí jen váš popis a pevné pokyny.
- **Nahradit stávající pravidla** (jen když nástroj už pravidla obsahuje; vypnuto): ve
  výchozím stavu se nová pravidla přidají za ta, která máte; po zaškrtnutí je nahradí
  (rozsah a nastavení, která odpověď uvádí, se použijí v obou případech).
- **Co se odesílá** je panel jen pro čtení na o něco tmavším pozadí, oddělený od polí nad
  ním, a při otevření dialogu **sbalený** (dialog je pak nízký). Zobrazíte ho kliknutím na
  jeho záhlaví, klávesou `Alt+W`, nebo přejdete `Tab` a stisknete `Enter`: vypíše pokyny
  (slovník pravidel, pak vaše pokyny pro AI) a váš požadavek a má vlastní posuvník
  (přetáhněte ho, použijte kolečko myši nad panelem, nebo šipky, `PageUp`, `PageDown`,
  `Home` a `End`, dokud má fokus). Profil, který je dostane, je na řádku nad ním. Dialog se
  zvětší, aby panel pojal, v mezích obrazovky. Pole popisu dostane vlastní posuvník, když
  text přeroste jeho výšku.

Model smí použít každý druh pravidla z této stránky kromě **Skript** (kód: *Napsat skript
pomocí AI* ukáže skript ke kontrole) a podoby *Hledat a nahradit v souborech* pro tajné
hodnoty (tajné hodnoty se modelu nikdy nepředají: použijte kartu Tajné hodnoty). Rozsah smí
nastavit na aktuální větev, všechny lokální větve a tagy, nebo na pojmenované větve a tagy,
které existují, a volby *Tagy*, *committer*, *prázdné commity* a *zkrácená id ve zprávách*.
Co nástroj vyjádřit neumí (omezení data, třeba *od března*, rozsah commitů), se nehádá:
model to řekne v poznámce. O tom, co se stane s podpisy, model nikdy nerozhoduje: zeptá se
vás Provést.

**AI jen připravuje pravidla a nastavení.** Nikdy nic nezobrazí v náhledu, neprovede ani
nespustí: odpověď se zkopíruje do nástroje jako načtený recept, náhled na obrazovce přestane
platit a Náhled i Provést (s jejich otázkami a potvrzeními) jsou jako vždy na vás. Každé
pravidlo se zvlášť zpracuje a zkontroluje stejnou kontrolou, jakou používá editor i stroj;
pravidlo, které neprojde, se vynechá. Když se něco vynechalo nebo model připojil poznámku,
po vložení pravidel se ukáže seznam (*Vynecháno (platná pravidla zůstala)*, každá položka
se svou pozicí, názvem a důvodem) a `Enter` nebo `Esc` ho zavře. Přečtěte si pravidla,
upravte je, pak Náhled.

Z příkazové řádky: `gitpoulpi ai rewrite-rules "<popis>" [--context]` vypíše návrh jako JSON
(`recipe`, který čte `rewrite preview --recipe-json`, `problems`, `notes` a `applied: false`)
a nic nespouští (návratový kód 2, když se něco vynechalo). Nástroj MCP je
`ai_rewrite_rules`. Použije se profil *Pravidla pro přepis historie* z Nastavení > AI.

## Skripty v Lua

Pravidlo skriptu spustí vaši funkci na každém commitu. Je to obyčejné Lua 5.4
v sandboxu: bez souborů, hodin, procesů, modulů a bez `pcall`; stejný skript
nad stejnou historií dá vždy stejný výsledek a volání, které přesáhne
10 milionů instrukcí, 64 MiB nebo 5 sekund, skončí chybou, která uvede
commit.

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

`c` obsahuje `id`, `parents`, `tree`, `author` a `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` a `signed`. `ctx` obsahuje `index`, `total`,
`params`, `state` (vrácením `state = {...}` předáte data dalšímu commitu) a
`window.before` / `window.after` (nastavte `--! window: N`). Vracejte jen to, co se mění:
`author`, `committer`, `message`. Pomocné funkce: `regex_replace`, `regex_match`,
`stable_hash`, `rand`, `weekday`, `hour`, `log` a pro zprávy `trailers(message)`
(seznam `{key, value}`), `set_trailer(message, key, value)` (nahradí trailery
toho klíče; čtvrtý argument `"add"` nebo `"skip"` to změní),
`remove_trailer(message, key)` a `at("2024-03-05 18:30", offset_minutes)`
(datum s posunem jako čas Unix).

Z příkazové řádky: `gitpoulpi rewrite script check my.lua` skript zkompiluje,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` ukáže,
co by změnil, aniž by cokoli zapsal, a
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b`
naplánuje celý přepis.

## Externí skripty (shell, Python, Lua)

Stejný kontrakt jako u skriptu v Lua, ale spouští ho program na vašem počítači:
*Shellový skript* (bash nebo sh; ve Windows PowerShell, nebo libovolný `.ps1`),
*Python* a *Lua (nainstalované)* (vaše vlastní `lua`, ne vestavěná; nemá knihovnu
JSON, takže šablona čte řádek vzorem). **Nejsou v izolaci: externí skript běží
s vašimi oprávněními** a může číst nebo měnit cokoli, co můžete vy. Říká to
panel pravidla i titulek editoru.

Jak komunikuje: GitPoulpi spustí skript jednou za běh a posílá mu na standardní
vstup jeden požadavek JSON na řádek (pole commitu, `params`, `state` vrácený
poslední odpovědí); skript odpovídá jedním řádkem JSON na standardním výstupu,
se stejnými poli, jaká vrací skript v Lua (`author`, `committer`, `message`,
`state`, `log`), nebo `{"error": "text"}`, když to vzdá. Diagnostiku vypisujte
na standardní chybový výstup: objeví se jako řádky `log` daného commitu. Řádky
hlavičky začínají `#!` u shellu a Pythonu (`#! name: ...`,
`#! param old: text "Old"`; řádek shebang zůstává shebangem) a `--!` u Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Co dostane.** Prázdné prostředí kromě `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` a `LANG`; novou dočasnou složku jako pracovní
složku (s vlastní kopií skriptu); ne cestu k repozitáři. **Časové limity:**
`rewrite.script_timeout_ms` (5000) pro každou odpověď a
`rewrite.script_total_timeout_s` (3600) pro celý průchod. Když se limit překročí,
skript spadne, vypíše něco, co není JSON, nebo se úloha zruší, GitPoulpi skript
i vše, co spustil, ukončí a přepis se zastaví s commitem a důvodem; nic se nepřesunulo.

**Které jazyky existují.** Vypisuje je `rewrite.external_scripts` v `config.toml`:
ve výchozím stavu `["shell"]`; `"python"` a `"lua"` přidejte sami v `config.toml`
(`[rewrite]`, `external_scripts = ["shell", "python"]`; seznam nelze nastavit
přes `gitpoulpi config set`). Jazyk, který v seznamu není, je v nástroji zašedlý
a CLI i MCP ho odmítnou (`scripts_disabled`); jazyk ze seznamu, jehož interpret
nelze najít, je zašedlý s návodem k instalaci (`tool_missing`).

**Důvěra.** I když je jazyk zapnutý, skript se spustí, až když jste důvěřovali jeho
přesnému textu. Skript, který napíšete v nástroji, je váš a běží hned; skript,
který přišel se sdílenou sadou nebo souborem repozitáře, ne: Náhled a *Vyzkoušet
na výběru* se zastaví a zeptají se *Důvěřovat tomuto skriptu?* se dvěma
číslovanými odpověďmi, z nichž žádná není předvybraná (`1` důvěřovat a pokračovat,
`2` nedůvěřovat, `Esc` to nechá být). Důvěra si pamatuje SHA-256 textu pro tento
repozitář, takže změněný skript se zeptá znovu a naklonovaný repozitář na vás
nemůže sám spustit kód. Z příkazové řádky soubor přečtěte a pak
`gitpoulpi rewrite script trust FILE` (nebo `--trust` u `preview`, `run`,
`script check` a `script test`, které důvěřují a pokračují). Agent přes MCP
nemůže důvěřovat ničemu: smí spustit jen skript, kterému jste dříve důvěřovali.

**Kontrola.** Řádek kontroly pod editorem u nedůvěryhodného skriptu říká
*Zatím není důvěryhodný* (nic se nespustilo); důvěryhodný GitPoulpi spustí jednou
s vymyšleným commitem (`"dry_run": true`) a ověří, že odpověď odpovídá kontraktu.
`gitpoulpi rewrite script check FILE` dělá totéž a hlásí `trust`, `hash`
a `interpreter`.

## Z příkazové řádky

`gitpoulpi rewrite` (a nástroje MCP stejných názvů) používá stejný stroj, pro
skripty a agenty. **Sada** je soubor TOML nebo JSON, ten, který ukládá nástroj
(`--recipe FILE`, nebo `-` pro standardní vstup), a přepínače přidávají pravidla
za ta, která jsou v sadě.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plán, nic nemění
gitpoulpi rewrite apply --run ID --digest D           # provede plán, který jste zkontrolovali
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # vrátí vše zpět
```

- Pravidla jako přepínače: `--set-name` / `--set-email` s `--match-email` (identita),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. Rozsah je aktuální větev, `--all-local`, nebo
  `--ref R` (lze opakovat), od `--since REV`.
- **Otázky bez výchozí odpovědi** jsou přepínače: `--signatures
  strip|resign-signed|resign-all`, když se mění podepsané commity, a
  `--extract-mode` pro vyjmutí. Bez nich je odpovědí odmítnutí nebo
  `needs_choice` (návratový kód 3), nikdy odhad.
- **Návratové kódy:** 0 hotovo; 1 chyba (nečitelná sada, špatné přepínače); 2 odmítnuto
  s důvodem (`stale`, `signature_choice_needed`, `dirty_paths_touched`,
  `empty_scope`, ...); 3 otázka k zodpovězení, nebo odeslané refy, které potřebují `--yes`.
- `preview` drží plán v repozitáři, takže `apply` funguje z pozdějšího
  příkazu, po restartu daemona i z jiného rozhraní; plán, který už neodpovídá
  svému digestu, je `stale`.

[Reference CLI](cli-reference.md) obsahuje každý přepínač a
[MCP a agenti](mcp-and-agents.md) říká, které nástroje agent dostane (nástroj
pro purge neexistuje, dokud ho nezapnete, a agent nemůže skriptu důvěřovat).

## Návrhy od agentů AI

Agent může přepis **navrhnout**, nikdy ho nespustí. `gitpoulpi ui propose rewrite
--recipe FILE|-` (nástroj MCP `ui_propose_rewrite`, vypnutý, dokud v `[mcp]`
není `allow_ui_propose = true`) otevře nástroj ve vašem běžícím okně s pravidly
sady už v seznamu a fialovým pruhem, *✦ Navrhuje <agent>: přepis historie
(3 pravidla) · Ctrl+Enter přijme · Esc odmítne*.

- **Nic se nespustí.** Žádný náhled, žádné provedení, žádný push, žádný purge:
  návrh jen vyplní pravidla. Přečtete si je, případně upravíte, spustíte
  **Náhled** a provedete vlastní klávesou. Zavření nástroje návrh odmítne.
- **Co do něj agent dát nesmí:** skript (spustil by se při náhledu), více než
  50 pravidel, tajné hodnoty (z sady se před odesláním odstraní). Takový návrh
  se s důvodem odmítne.
- Agent si výsledek přečte v `ui state` (`accepted`, když jste pravidla provedli
  tak, jak byla navržena, `accepted_edited`, když jste je předtím změnili,
  `rejected`, když jste nástroj zavřeli). Vždy jeden návrh; odmítne se, dokud je
  otevřený jiný váš dialog.

## Nástroje

Většina přepisu historie nepotřebuje nic kromě Gitu: změna autorů a
committerů, použití `.mailmap`, anonymizace, posun nebo omezení dat na hodiny,
přepis zpráv a trailerů, nahrazení obsahu, odstranění cest nebo velkých
bloků a přesun cest probíhá uvnitř GitPoulpi.

**Skripty v Lua jsou vestavěné.** Interpret je součástí GitPoulpi, takže
skript v Lua nepotřebuje žádný nástroj ani nastavení. Skripty v shellu, Pythonu
a nainstalované Lua potřebují svůj interpret (viz tabulka níže) a jsou popsány
v části [Externí skripty](#externí-skripty-shell-python-lua).

Několik funkcí používá externí program. GitPoulpi ho vyhledá (program
se hledá v `PATH`, ve Windows i přes `PATHEXT` a ve vlastním `usr/bin` Git for
Windows, a musí do 2 sekund odpovědět na `--version`); odpověď se minutu
drží v mezipaměti a „Zkontrolovat znovu“ se zeptá znovu. Pravidlo, které
potřebuje chybějící program, zůstane v seznamu zašedlé, s důvodem a návodem
k instalaci, a Náhled se odmítne dřív, než se cokoli spustí.

| Funkce | Nástroj | Poznámky |
|---|---|---|
| Nové podepsání commitů | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) nebo `gpgsm` (`gpg.format = x509`) | Podepisování SSH a X.509 vyžaduje také `user.signingkey`. Nastavení `gpg.program`, `gpg.ssh.program` a `gpg.x509.program` se respektují. |
| Převod do LFS (obě pravidla LFS) | `git-lfs` 3.0 nebo novější | Stejná kontrola jako v panelu LFS. |
| Skripty v Pythonu | `python` 3.8 nebo novější (ve Windows `py -3`) | Ve výchozím stavu vypnuto: přidejte `"python"` do `rewrite.external_scripts`. |
| Skripty v Lua (nainstalované) | `lua5.4`, `lua54`, `lua` nebo `luajit` | Ve výchozím stavu vypnuto: přidejte `"lua"` do `rewrite.external_scripts`. |
| Shellové skripty | `bash`, `pwsh` (PowerShell) nebo `sh` | Ve výchozím stavu zapnuto (`"shell"`). Skript v PowerShellu (soubor `.ps1`, shebang `pwsh` nebo jakýkoli shellový skript ve Windows) potřebuje `pwsh` nebo `powershell`; skript POSIX potřebuje `bash` nebo `sh` (ve Windows je poskytuje Git for Windows). |

Návody k instalaci, nejdřív Windows:

- Windows: Git for Windows obsahuje `gpg`, `ssh-keygen`, `bash`, `sh` a
  Git LFS. Jinak `winget install GnuPG.GnuPG`, Nastavení > Volitelné
  funkce > Klient OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: balíčky distribuce (`gnupg`, `openssh`, `git-lfs`,
  `python3`, `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` a
  `sh` jsou součástí systému.
