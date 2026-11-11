[English](../undo-redo.md) · **Čeština** · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Zpět a znovu

**Vždy se můžete vrátit.** Každá změna, kterou GitPoulpi ve vašem repozitáři
udělá, se zaznamenává (ve výchozím stavu až 100 akcí, `general.undo_depth`) a
stejně tak každá pozice, na které větev byla, včetně posunů provedených jinými
programy. Když se zmýlíte, práci vám vrátí jedna z těchto cest:

| Chcete | Použijte |
|---|---|
| vzít zpět poslední věc | `Ctrl+Z` (znovu: `Ctrl+Y` nebo `Ctrl+Shift+Z`) nebo tlačítka **Zpět** a **Znovu** na panelu nástrojů |
| vzít zpět víc věcí najednou, nebo jednu věc z dávné doby | **Historii vrácení** (`Alt+Z`, šipka vedle Zpět, `Úpravy: Historie vrácení...`) |
| získat zpět větev, tag, stash, soubor nebo hunk, které jste odstranili před několika akcemi, **bez vracení toho, co jste udělali od té doby** | **Obnovit jen tuto** v Historii vrácení |
| posunout větev zpět tam, kde byla, i když ji posunul jiný program | **Reflog** větve (nabídka refu, `Reflog...`) |
| najít commity, ke kterým už nevede žádná větev | **Ztracená práce** (`Úpravy: Zachránit ztracené commity...`) |
| vzít zpět, co shell nebo **AI agent** udělal s větvemi, tagy, stashi nebo HEADem | tentýž `Ctrl+Z`: tyto změny jsou také v historii, označené jako **externí** (viz [Změny provedené mimo GitPoulpi](#změny-provedené-mimo-gitpoulpi)) |

Po každé akci, která něco ničí a jde ji vrátit (zahození změn, smazání souboru,
smazání větve nebo tagu, hard reset, zahození stashe, odebrání remotu), to
řekne stavový řádek: „zahození změn: hotovo - Ctrl+Z pro vrácení“ s klávesou
tak, jak ji máte přiřazenou. Tooltip tlačítka Zpět říká, kolik akcí můžete jít
zpět.

`Ctrl+Z` vezme zpět poslední věc, kterou GitPoulpi s vaším repozitářem udělal.
`Ctrl+Y` (nebo `Ctrl+Shift+Z`) ji vrátí. Panel nástrojů má tlačítka **Zpět** a
**Znovu**, paleta má `Úpravy: Zpět` a `Úpravy: Znovu` a tooltip každého
tlačítka říká, co by vzalo zpět („Vrátit commit ‚fix typo‘ (a81b9e4)“).

![Panel nástrojů po Stage all changes: Zpět svítí a jeho tooltip jmenuje operaci, kterou by vzalo zpět](../images/undo-toolbar.png)

Znovu operaci nikdy nespouští znovu. Obnoví zaznamenaný stav, takže znovu
provedený commit je přesně tentýž commit, se stejným id a stejným časovým
razítkem.

`Ctrl+Z` nedělá nic, dokud má klávesnici textové pole (zpráva commitu, pole
filtru, editor, který si drží vlastní zpět pro text).

## Historie vrácení

`Alt+Z` (přiřaditelná), šipka vedle tlačítka Zpět nebo `Úpravy: Historie
vrácení...` otevře panel se všemi zaznamenanými akcemi, nejnovější první: ikona
druhu akce (commit, větev, tag, stash, reset, merge a spol., zahození, staging,
konfigurace, externí změna), co akce udělala (ve vašem jazyce), před jak dlouhou
dobou a v jakém je stavu (**lze obnovit samostatně**, **blokováno** s důvodem,
nebo **vráceno** a připraveno k opakování). Když položky zasahují do více dnů,
každý den začíná tenký nadpis (Dnes, Včera, pak datum). Záhlaví je počítá („Lze
vrátit 12 akcí (až 100)“).

Akce, které jste vrátili, jsou **nad** nejnovější akcí, pod děličem („vráceno -
Znovu to obnoví“), ztlumené, a ta, kterou Znovu přinese zpět jako další, je
nejblíž děliči. Výběr začíná na nejnovější akci, ne na nich. Šipky se pohybují
(nadpisy a dělič se přeskakují), `1` / `2` / `3` (nebo `Ctrl+Tab` /
`Ctrl+Shift+Tab`) přepínají mezi třemi zobrazeními, `Esc` zavře panel. `Enter`
nikdy nespustí akci: každé tlačítko má své písmeno, které ukazuje řádek nápovědy
pod tlačítky. Pro vybraný řádek:

- **Vrátit až sem** (`a`): vrátí tuto akci i všechny novější, nejnovější první,
  po jediném potvrzení, které je všechny vypíše. Krok, který daemon pokládá za
  riskantní, se před spuštěním zeptá znovu.
- **Obnovit jen tuto** (`o`): přinese zpět to, co tato jedna akce odstranila, a
  zachová vše, co se udělalo po ní. Smazaná větev nebo tag se vrátí na commit,
  který měly (je-li jejich název mezitím obsazený, budete vyzváni k jinému,
  volný název je předvyplněný); zahozený stash se vrátí na vrch seznamu
  stashů; zahozený nebo smazaný soubor se zapíše zpět na místo, pokud se od té
  doby nezměnil, jinak vedle něj jako `název.restored` (nic se nikdy
  nepřepíše); odebraný remote, upstream, řádek pro ignorování nebo řádek
  atributů se přidá znovu. Reset, amend, rebase, merge nebo commit vytvoří
  větev `restore/<větev>-<sha>` na starém vrcholu - vaše aktuální větev a
  soubory se nikdy nepohnou. Potvrzení přesně ukáže, co se stane, a obnovení je
  nová akce: `Ctrl+Z` ji vezme zpět.
- **Zopakovat až sem** (`z`) na vráceném řádku.
- **Obnovit main tam, kde ji nechala akce commit** (`t`, nebo `Alt+Shift+Z` /
  `Úpravy: Obnovit větev tam, kde ji nechala blokovaná akce` mimo panel): když
  větev posunul jiný program a zablokoval Zpět, reflog stále ví, kde ji akce
  nechala.

Pravé tlačítko na řádek (nebo `.`) nabídne tytéž akce jako nabídku.

![Historie vrácení po smazání větve a dvou commitech: řádek smazané větve je vybraný a lze ho obnovit samostatně](../images/undo-history.png)

## Reflog

Nabídka refu lokální větve má **Reflog...**; `Úpravy: Reflog...` ukáže reflog
HEAD a Historie vrácení má pro něj kartu. Vypíše pozice, na kterých větev byla,
nejnovější první: krátké SHA, zaznamenaný důvod, před jak dlouhou dobou, kdo -
včetně posunů provedených v shellu. Zpráva vybrané pozice a změněné soubory se
ukážou vedle seznamu. **Zobrazit v Logu** (`z`), **Vytvořit větev tady** (`b`),
**Vytvořit tag tady** (`t`), **Použít na pracovní strom** (`p`), **Uložit jako
stash** (`u`) (poslední tři fungují jako ve Ztracené práci níže) a
**Obnovit `<větev>` na tuto pozici** (`o`). Obnovení lze vrátit. Je-li větev
checkoutnutá a máte lokální změny, které by přesun musel přepsat, rozbalovací
seznam se zeptá, co s nimi: ponechat moje změny, soft, mixed, nebo hard. Hard
změny v souborech zahodí, ale nejdřív je uloží, takže je `Ctrl+Z` stále vrátí
(říká to červeně).

Reflog se čte po 200 pozicích. Delší pokračuje sám: pohyb kurzoru (nebo
posouvání) poblíž konce seznamu načte další stránku a poslední řádek zní
**... Načíst další**, dokud nějaká je. **Načíst další** (`n`, nebo `Reflog:
Načíst další` v paletě) ji načte hned a stavový řádek řekne „načtené pozice: N“.
Repozitáře používající formát reftable gitu se čtou jako každé jiné.

![Reflog větve main: každá pozice, na které byla, nejnovější první, s popisem vybrané pozice vpravo](../images/reflog.png)

## Ztracená práce

`Úpravy: Zachránit ztracené commity...` (také v nabídce commitu v Logu) najde
commity, ke kterým nevede žádná větev, tag ani stash: ty, které ještě zná
některý reflog (commity smazané větve), ty, které drží jen pin vrácení, a
osiřelé. Seznam se objeví hned a pomalejší hledání osiřelých commitů se
doplňuje, zatímco se točí spinner; rozhraní nikdy neblokuje.

![Ztracená práce: dva commity smazané větve (nalezené přes reflog) a osiřelé commity](../images/lost-work.png)

**Vytvořit větev tady** (`b`) obnoví commit pod názvem, který zvolíte,
**Vytvořit tag tady** (`t`) ho připne tagem (výchozí název `restored-<krátké
id>`), **Cherry-pick** (`c`) ho použije na aktuální větev. Ztracený commit není
v Logu, takže toto zobrazení nemá „Zobrazit v Logu“ (zobrazení Reflog ho má pro
commity, které jsou stále dosažitelné).

**Obnovit změny** vrátí to, co commit změnil, bez commitování: **Použít na
pracovní strom** (`p`) přehraje změny commitu na soubory, vše nebo nic (index a
HEAD zůstanou; merge commit se odmítne). Když se změny nedají použít čistě, nic
se nedotkne a dialog nabídne **Uložit jako stash** (`u`), který stejné změny
zachová jako nový stash; tato akce je vždy dostupná i samostatně. Obě se
zapisují do žurnálu: `Ctrl+Z` vrátí soubory zpět, nebo stash znovu zahodí. Totéž
dělají `gitpoulpi apply-commit <rev>` a `gitpoulpi stash from-commit <rev> [-m M]`
a zobrazení Reflog je má také.

### Panely, hunky a okno

Reflog a Ztracená práce ukazují vlevo seznam commitů a vpravo hlavičku
vybraného commitu, **každý soubor, který změnil**, a hunky vybraného souboru
(jen hunky, zalomené na šířku panelu). Seznam souborů je ten z pravého panelu
hlavního okna: `Cesty` nebo `Strom` (`v`, nebo klikněte na přepínač), složka se
sbalí pomocí `Enter`, `Left` nebo `Right`, kliknutí nebo šipky vyberou soubor
a jeho hunky se objeví pod ním (daemon se ptá mimo vlákno rozhraní; posledních
několik souborů se drží v paměti). Seznam commitů, seznam souborů a hunky mají
každý posuvník, který lze stisknout a táhnout.

- `Tab` / `Shift+Tab` procházejí panely (seznam, soubory, hunky); šipky,
  `PageUp`, `PageDown`, `Home` a `End` působí na panel, který má klávesnici
  (jeho okraj svítí). `Enter` na souboru přejde na jeho hunky.
- **Děliče**: přetáhněte ten mezi seznamem a pravým panelem, nebo ten mezi
  soubory a hunky; dvojklik na dělič ho obnoví. Klávesnice: `Alt+Left` /
  `Alt+Right`, `Alt+Up` / `Alt+Down`, `Alt+0` obnoví oba. Paleta má
  `Reflog / Ztracená práce: Obnovit velikosti panelů`, `Další panel` a
  `Seznam souborů jako cesty nebo strom`.
- `Ctrl+N` ukáže své odznaky nad třemi kartami zobrazení a třemi panely, jako
  v hlavním okně; klávesa odznaku tam skočí.
- **Přesun a změna velikosti dialogu**: přetažením záhlaví ho přesunete,
  přetažením levého, pravého a spodního okraje nebo spodního rohu (úchyt `◢`)
  změníte velikost (nikdy pod 60 x 16, vždy na obrazovce). Zavírací křížek si
  nechává své kliknutí a uvolnění tlačítka mimo dialog ho nikdy nezavře.
  Klávesnicový protějšek je `Ctrl+N` a pak `m`: šipky přesouvají, `Shift`+šipky
  mění velikost, `0` obnoví výchozí, `Enter` nebo `Esc` režim opustí (okraj to
  říká, dokud je zapnutý). Paleta má `Dialog: Přesunout nebo změnit velikost` a
  `Dialog: Obnovit pozici a velikost`. Zda se místo zachová, určuje
  `ui.dialog_geometry` (*Nastavení > Vzhled > Rozvržení > Pamatovat si pozici a
  velikost dialogů*): `session` (výchozí), `saved` (obnoví se při příštím
  spuštění) nebo `never`.

## Z palety příkazů

Každá akce řádku ve všech třech zobrazeních je také řádkem palety příkazů, takže
panel nepotřebuje myš ani písmeno, které byste si museli pamatovat. Stiskněte
`Ctrl+P`, když je panel otevřený: paleta se otevře nad ním a její řádky
**Historie vrácení:**, **Reflog:** a **Ztracená práce:** působí na vybraný řádek
zobrazení, které je na obrazovce.

| Zobrazení | Řádky palety |
|---|---|
| Historie vrácení | `Historie vrácení: Vrátit až sem...`, `Zopakovat až sem...`, `Obnovit jen tuto...`, `Obnovit větev tam, kde byla...`, `Obnovit seznam`, `Zavřít` |
| Reflog | `Reflog: Zobrazit v Logu`, `Vytvořit větev tady...`, `Vytvořit tag tady...`, `Použít změny na pracovní strom`, `Uložit změny jako stash`, `Obnovit na tuto pozici...`, `Načíst další`, `Obnovit seznam` |
| Ztracená práce | `Ztracená práce: Vytvořit větev tady...`, `Vytvořit tag tady...`, `Cherry-pick`, `Použít změny na pracovní strom`, `Uložit změny jako stash`, `Hledat znovu` |

Řádky jsou zašedlé (s důvodem), dokud jejich zobrazení není otevřené; tři otevírací
řádky, `Úpravy: Historie vrácení...`, `Úpravy: Reflog...` a `Úpravy: Zachránit
ztracené commity...`, také přepnou už otevřený panel na své zobrazení. Řádek,
který výběr nezvládne, to řekne ve stavovém řádku místo toho, aby neudělal nic:
„tato akce je už vrácena: použijte Zopakovat až sem“, důvod, proč nelze položku
obnovit samostatně, „zpět lze přesunout jen větev nebo HEAD“ a podobně. Řádky s
`...` otevřou stejné potvrzení nebo dotaz na název jako tlačítko.

## Co lze vrátit

| Oblast | Operace |
|---|---|
| Commity | commit, amend, reword, AI composer commitů (všechny jeho commity jako jeden krok) |
| Větve a tagy | vytvoření, smazání, přejmenování, nastavení nebo zrušení upstreamu, vytvoření sledující větve, vytvoření / smazání / anotace tagu, fast-forward, checkout pull requestu jako větve |
| Pohyb po repozitáři | checkout větve nebo commitu (včetně odpojeného HEAD) |
| Historie | merge (fast-forward nebo merge commit), rebase (obyčejný i interaktivní, až skončí), cherry-pick, revert, reset (soft, mixed i hard), lokální část pullu |
| Pracovní změny | stage a unstage (soubor, složka, hunk, řádky, vše), zahození (soubor, složka, hunk, řádky, checkout souboru), aplikace patche nebo změn commitu (`apply-commit`), **smazání souboru z disku**, **ignorování** (soubor, přípona nebo složka: řádek v `.gitignore`, který jste zvolili), **považování za binární nebo textový** (řádek v `.gitattributes`) |
| Stashe | push, apply, pop, drop, uložení změn commitu jako stashe (`stash from-commit`) |
| Remoty a konfigurace | přidání, odebrání, přejmenování, změna URL nebo push URL remotu, Git LFS track a untrack |
| Na serveru | **smazání větve nebo tagu na remote** (z nabídky refu, z palety, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, nebo dokončení workflow, které maže vzdálenou větev): viz níže |

Dokud merge, cherry-pick nebo revert stojí na konfliktech, lze vrátit i
**Uložit a označit jako vyřešené**, stage a unstage (každé se vezme zpět před
tím, které leží pod ním) a také **Přerušit merge** (nebo cherry-pick, revert):
vrácení přerušení přinese zpět celou zastavenou operaci s jejími konflikty (tři
verze každého nevyřešeného souboru), již vyřešenými a staged soubory,
konfliktními značkami v pracovním stromu a zprávou merge; znovu ji zase
přeruší. Stage a řešení přestanou být vratná, jakmile operace skončí (commitem
nebo přerušením). *Unstage* a *Unstage All* se nikdy nedotknou souboru, který je
stále v konfliktu: nemá žádnou staged změnu, kterou by šlo vzít zpět.

Vše, co zahození nebo hard reset odhodí, se nejprve uloží, takže vrácení obnoví
přesné bajty, včetně nesledovaných souborů, bitů spustitelnosti a symbolických
odkazů. Vrácení zahozeného stashe vrátí tentýž stash se stejným id commitu
(skončí na vrcholu seznamu stashů jako `stash@{0}`, ne na své staré pozici).
Vrácení odebrání remotu vrátí jeho remote-tracking větve, jeho alias
`origin/HEAD` a nastavení upstreamu větví, které jej sledovaly.

### Smazání větve nebo tagu na remote

Před smazáním se GitPoulpi zeptá serveru, co větev nebo tag právě drží, a zaznamená
to: remote, název a přesný commit (u anotovaného tagu samotný objekt tagu, takže
jeho anotace zůstane). Smazání se pak provede jen tehdy, když server stále drží
tuto hodnotu.

**Zpět ji pushne zpět** pod stejným názvem, na stejný commit, bez jakéhokoli
vynucení. Protože je to změna na serveru, který vidí další lidé, Zpět **se vždy
zeptá** a řekne to („Vrácení znovu vytvoří větev topic na origin na
3f2a1c0..."); říkají to i kroky položky (`gitpoulpi undo --dry-run`). Používá
stejné přihlašovací údaje jako push. Remote-tracking větev se vrátí s ní.
**Znovu** ji smaže znovu, se stejnou jistotou.

Zpět odmítne a nic nezmění, když:

- větev nebo tag byly na remote od té doby **vytvořeny znovu** nebo posunuty:
  jsou na jiném commitu, než který byl zaznamenán, a push by přepsal cizí práci
  (odmítnutí uvádí oba commity);
- commit (nebo objekt tagu) **už není ve vašem repozitáři** (byl odstraněn
  garbage collectionem): vrácení nelze provést.

„Smazat lokálně i na remote“ je **jedna položka**: Zpět obnoví lokální i vzdálenou
větev nebo tag společně, Znovu obojí smaže znovu. Smazání jen vzdálené větve
lokální ponechá; vrácení to dělá také. Dokončení workflow, které maže i vzdálenou
větev, lze vrátit jako celek.

## Co vrátit nelze a proč

- **Většinu věcí na serveru**: push, push tagu, LFS push, lock a unlock, otevření
  pull requestu. Tento stav sdílejí další lidé; vrácení pushe by byl force-push
  nebo smazání na jejich straně a GitPoulpi ho za vás neudělá. (Smazání vzdálené
  větve nebo tagu je výjimka: vrácení jen vrátí, co server držel, a odmítne, když
  se tam cokoli změnilo.) Zpět se kvůli takové operaci také nikdy neblokuje.
- **Fetch**: pouze posouvá remote-tracking refy, které zrcadlí server. Další
  fetch by je posunul znovu.
- **Přerušení rebase**: zpět se pro něj nenabízí (rebase posouvá větve a drží
  vlastní stav; přerušení je samo cestou zpět). Přerušení **merge,
  cherry-picku nebo revertu** se *zaznamenává*, viz výše.
- **Vytvoření worktree**, **vytvoření souboru patche**, clone, init, akce
  hostingu (pull requesty, issues) a volání AI: nejsou součástí historie
  repozitáře.
- **Vytvoření nového prázdného souboru** z nabídky souboru, automatické
  odstranění souborů `*.orig` po merge a uložení souboru ve vestavěném editoru
  se nezaznamenávají. Stejně tak změna **příkazu SSH** (`core.sshCommand`) a změna **částečného checkoutu**
  (nic se neztratí; *Vypnout* vrátí každý soubor).
- **Zápis do `.git/info/exclude` nebo `.git/info/attributes`** (volba „jen tento klon“ v seznamech pro ignorování a považování za binární/textové): tyto soubory jsou mimo pracovní strom, a proto se nezaznamenávají.
- **Úpravy souborů v pracovním stromu provedené mimo GitPoulpi** (editor,
  skript, AI agent píšící kód) se nikdy nezachycují: žádný jejich snímek
  neexistuje, takže `git reset --hard` nebo `git checkout -- soubor` provedený
  v shellu ztratí necommitnuté úpravy nadobro. To, co se ze shellové práce
  zachycuje, žije v refech: viz další oddíl.

## Změny provedené mimo GitPoulpi

Když pracujete s AI agentem (nebo s shellem vedle GitPoulpi), většina práce s
gitem probíhá mimo okno: commity, nové větve a worktrees, smazané větve, resety,
rebase, stashe. GitPoulpi i ty vidí. Daemon si drží poslední refy, které viděl
(lokální větve, tagy, HEAD, seznam stashů, seznam worktrees) a kdykoli se
repozitář změnil a nebylo to dílem samotného GitPoulpi, zaznamená **jednu
položku na ucelenou změnu** v Historii vrácení, označenou jako **externí**, s
důvodem, který git zapsal (`commit: fix login`, `rebase (finish)`, `reset:
moving to HEAD~1`, `checkout: moving from a to b`), a se jménem, které git
zaznamenal pro toho, kdo změnu udělal. GitPoulpi nepozná shell od AI agenta a
nikdy nehádá; položka říká jen to, že vznikla mimo.

- **Dávka je jedna položka.** Dvacet commitů udělaných mezi dvěma pohledy je
  jedna položka od první hodnoty k poslední; rebase je jedna položka,
  zaznamenaná, když skončí (dokud je rebase nebo merge zastavený, nezaznamenává
  se nic).
- **Smazané refy se připnou.** Větev, tag nebo stash, který zmizel, se drží
  naživu pod `refs/gitpoulpi/undo/` na svém starém commitu dřív, než ho cokoli
  může odstranit, takže **Obnovit jen tuto** (nebo `Ctrl+Z`) vrátí větev, kterou
  agent smazal, i po reflogu a garbage collection.
- **Když byl GitPoulpi zavřený.** Snímek se ukládá vedle žurnálu
  (`.git/gitpoulpi/refs-snapshot.json`), takže to, co se stalo, když GitPoulpi
  neběžel, se najde při dalším startu a označí „když byl GitPoulpi zavřený“.
- **`Ctrl+Z` na ně funguje jako na kteroukoli položku**: refy se vrátí, smazaná
  větev se vytvoří znovu, zahozený stash se uloží znovu, externí checkout znovu
  přepne starou větev. Přepnutá větev se vrací jako `git reset --keep`: soubory
  ji následují, a když v cestě stojí některá vaše lokální změna, **odmítne a nic
  nezmění**. Zopakování ji provede znovu. Funguje i **Obnovit jen tuto**: smazaná
  větev se vrátí a pozdější akce zůstanou; přesunutá větev se vrátí jako
  `restore/<větev>-<sha>`.
- **Stavový řádek to řekne jednou**: „zaznamenána externí změna - Ctrl+Z ji
  vrátí“, když dorazí nová externí položka a okno je v popředí.
- Protože je externí změna teď nejnovější položkou, dřívější odmítnutí
  „zablokováno externí změnou“ většinou mizí: externí změnu nejdřív vrátíte.

Co se nezaznamenává: posuny remote-tracking větví při samotném fetchi (zrcadlí
server), skryté refy samotného GitPoulpi, worktree přidaný nebo odebraný bez
jakékoli změny větve (položka větve, kterou nese, říká „přidán worktree
<cesta>“; vrácení posouvá jen refy, worktree neodstraní) a cokoli uvnitř
souborů. `general.journal_outside = false` (Nastavení > Obecné, „Zaznamenávat
změny provedené mimo GitPoulpi“) to vypne: nic externího se nezaznamenává a
změna provedená mimo zablokuje nejnovější položku jako dřív.

## Kdy zpět odmítne

Před jednáním GitPoulpi ověří, že vše, čeho se operace dotkla, je stále přesně
takové, jak to operace zanechala. Pokud ne, **nezmění nic** a řekne, co se
pohnulo:

- `main` je na 3f2a1c0, ne na a81b9e4, kde jej Commit nechal (například commit
  udělaný v shellu);
- od té doby se změnil soubor, index, stash nebo sekce konfigurace;
- probíhá merge, rebase nebo cherry-pick;
- operace nikdy nedoběhla, protože GitPoulpi byl za jejího běhu zastaven
  (položka je označena jako *interrupted*, nikdy se automaticky nevrací a
  `gitpoulpi undo --list` ukáže, co se chystala změnit, abyste to mohli
  opravit ručně).

Zpět je přísně last-in, first-out: blokovaná nejnovější položka blokuje i
starší, protože vracet starší operaci pod novější, která stále platí, by bylo
hádání. Pokud věci vrátíte do původního stavu sami, položka je opět platná.
Změny, které se nedotýkají ničeho, čeho se dotkla položka (commit v shellu na
jiné větvi, fetch), nic neblokují.

### Blokovaná položka, kterou nelze vyčistit: Forget

Když nejnovější položka uvízla natrvalo (repozitář se posunul mimo GitPoulpi
nebo položka nikdy nedoběhla), nelze vrátit ani nic pod ní. Tooltip Zpět a
odmítnutí to pak řeknou a uvedou cestu ven: **Úpravy: Zapomenout blokovanou
položku vrácení** v paletě (nejprve se zeptá) nebo `gitpoulpi undo-forget`
v příkazové řádce. Zapomenutí jen odstraní tuto jednu položku z žurnálu;
repozitáře se nikdy nedotkne a položku už nelze vrátit ani znovu provést.
Položka pod ní se pak stane nejnovější.

Dva případy se vyřeší samy:

- konfliktní merge nebo rebase, který dokončíte čistým gitem: položka se uzavře
  při příštím čtení stavu repozitáře v GitPoulpi;
- soubory konfliktního stash pop nebo apply, které vyřešíte pomocí **Uložit a
  označit jako vyřešené**: Zpět vrátí konflikt, dokud je vyřešený soubor stále
  staged a necommitnutý.

## Potvrzení

Zpět se ptá jen tehdy, když je to riskantní:

- commit (nebo merge, rebase, cherry-pick, revert), který berete zpět, je už na
  upstreamu větve: vrácení přepisuje historii, kterou jste zveřejnili, a
  výsledek byste museli force-pushnout;
- máte necommitnuté změny a zpět posouvá pracovní strom: vaše změny zůstanou
  zachovány, ale zpět se zastaví beze změny, pokud je některá z nich v souboru,
  který musí přepsat.

- položka smazala větev nebo tag na remote: vrácení ji **pushne zpět na server**
  (viz výše), takže se vždy zeptá, stejně jako Znovu, které ji tam smaže znovu.

Znovu a každé vrácení pouhého posunu refu se nikdy neptá: opačný směr je na
jedinou klávesu.

## Jak daleko zpět

`general.undo_depth` (Nastavení > Obecné) je počet uchovávaných operací.
Výchozí hodnota je **100**; `0` znamená neomezeno. Snížení hned zapomene
nejstarší položky. Zahájení nové operace po vrácení zahodí to, co jste mohli
provést znovu, jako v každém editoru.

Historie se drží pro každý worktree zvlášť (připojený worktree má vlastní) v
adresáři `.git` (`.git/gitpoulpi/journal.json`), takže přežije zavření programu a
nikdy není součástí `git status` ani commitu.

## Vypnutí

`general.undo_enabled = false` (Nastavení > Obecné, „Zpět povoleno“) vypne celou
funkci (a s ní i zaznamenávání externích změn). Nic se nezaznamenává ani neuchovává, takže stage, zahazování a
commitování jsou o něco rychlejší. Tlačítka Zpět a Znovu zmizí a `Ctrl+Z`
řekne „Zpět je vypnuto“.

**Vypnutí smaže historii**: soubor žurnálu a skryté refy popsané níže se smažou
při prvním požadavku, který dorazí k daemonu, dokud je funkce vypnutá. Zapnutí
zpět začíná s prázdnou historií.

## Co se ukládá

Aby vrácení nikdy nemohlo ztratit data, každý commit, objekt tagu, stash nebo
snímek, který by položka mohla potřebovat, drží naživu skrytý ref pod
`refs/gitpoulpi/undo/`, dokud je položka v historii. Ve výpisech samotného
GitPoulpi jsou vyloučeny, ale `git log --all` nebo `git for-each-ref` v shellu
je ukáže a `git push --mirror` by je odeslal (použijte místo toho `git push
--all`). Smažou se, když položka z historie vypadne nebo když se zpět vypne.
Nic jiného se nikdy nemaže.

## Z příkazové řádky a MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` popíše kroky a nic nezmění.
- `--list` vypíše žurnál, nejnovější poslední (`--all` přidá položky, které už
  nelze provést znovu, ale stále se uchovávají), s id a stavem každé položky.
- `--expect <id>` způsobí, že volání odmítne jako *stale*, pokud ta položka už
  není nejnovější: skript (nebo druhé okno) tak nikdy nevrátí něco jiného, než
  na co se díval.
- `undo-forget` odstraní blokovanou nejnovější položku ze žurnálu (`--redo`:
  nejnovější položku pro znovu). Odmítne, když ta položka není blokovaná, a bez
  `--yes` skončí s kódem **3**. Pro MCP je to mutace, ale ne destruktivní:
  repozitáře se nedotkne.
- Kód ukončení **0** hotovo, **2** odmítnuto (JSON uvádí důvod, například
  `RemoteRefMoved`, když byla smazaná vzdálená větev vytvořena znovu), **3**
  riskantní a vyžaduje `--yes` (vrácení smazání vzdálené větve nebo tagu, které
  pushuje na server).

Návrat bez vracení všeho, co přišlo potom:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (také: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (také: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` ukáže u každé položky, co by „obnovit jen tuto“ mohlo přinést
  zpět (`restore.items`), nebo proč nic (`restore.not_restorable`), a kdo ji
  udělal: `"source": "gitpoulpi"`, nebo `"source": "outside"` s `outside.message`
  (důvod od gitu) a `outside.committer` u změny provedené shellem nebo AI
  agentem. `undo`, `redo` a `undo-restore` berou externí položky jako kteroukoli
  jinou.
- `undo-restore <id>` přinese zpět to, co tato jedna položka odstranila
  (smazanou větev nebo tag, zahozený stash, zahozený nebo smazaný soubor,
  odebraný remote s jeho tracking větvemi a upstreamy, řádek pro ignorování) a
  zachová všechny pozdější operace. Nikdy nepřepisuje: soubor, který se od té
  doby změnil, dostane uchovanou verzi vedle sebe jako `<název>.restored`;
  obsazený název větve je odmítnutí (kód 2), které navrhne volný, zadejte ho
  přes `--name`; po resetu, amendu nebo rebase vytvoří novou větev na starém
  vrcholu a vaši větev nikdy nepřesune. Obnovení je jedna položka žurnálu,
  takže ho `gitpoulpi undo` vezme zpět.
- `reflog` vypíše, kde `HEAD` nebo větev byly, s důvodem od gitu, včetně toho,
  co udělal shell. `reflog-restore` přesune větev (nebo `HEAD`) na jednu z těch
  pozic; checkoutnutá větev s necommitnutými změnami potřebuje `--mode`. I to
  se zapisuje do žurnálu.
- `lost-commits` vypíše commity, ke kterým nevede žádná větev, tag, remote ani
  stash. Chcete-li získat zpět práci jednoho z nich bez commitování:
  `apply-commit <rev>` (na pracovní strom, vše nebo nic) nebo
  `stash from-commit <rev>` (jako stash); obojí se zapisuje do žurnálu.
- Když posun v shellu zablokuje nejnovější položku, odmítnutí nese pozici
  reflogu, která drží to, co položka očekává (`reason.RefMoved.restore`), k
  předání do `reflog-restore`.

Dva příkazy existují proto, aby výše uvedené akce šly vrátit z každého
rozhraní: `gitpoulpi rm-file <paths>` (smazání souborů z pracovního stromu) a
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (připojí řádek do
`.gitignore`; nic se nestane, když tam řádek už je; zpět vrátí soubor do
původního stavu, nebo ho odstraní, pokud jej příkaz vytvořil, a odmítne, pokud
byl soubor od té doby upraven).

Server MCP má jeden nástroj na příkaz, takže `undo`, `redo`, `rm-file` a
`ignore` jsou také nástroje. `undo`, `redo` a `rm-file` jsou zařazeny jako
destruktivní (hlídané jako `reset`, viz [MCP a agenti](mcp-and-agents.md));
`ignore` je běžná mutace. Žurnál je soubor v repozitáři, takže `gitpoulpi undo`
v shellu vezme zpět to, co udělalo okno, a okno může vrátit to, co udělal
skript.
