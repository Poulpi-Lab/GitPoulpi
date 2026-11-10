[English](../troubleshooting.md) · **Čeština** · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Řešení potíží a FAQ

Začněte rychlými odpověďmi níže; delší sekce se věnují Git LFS a vykreslování
grafu. Pokud nic nepomůže, založte issue v repozitáři na GitHubu s uvedením
svého OS, terminálu, výstupu `gitpoulpi --version` a toho, co jste dělali.

## Když něco selže

Selhaná operace neukáže žádné vyskakovací okno ani rámeček: říká to **stavový řádek**, červeně, s názvem a důvodem (*chyba: Fetch: selhání: Remote je nedostupný.*), a zůstane tam, dokud ho nenahradí jiná zpráva. Tlačítko Aktivita na jeho pravém konci počítá chyby, na které jste se ještě nepodívali. Kliknutím na červenou zprávu, nebo příkazem palety `Protokol aktivity: Otevřít poslední chybu`, otevřete Protokol aktivity na tom záznamu. Takto funguje každé selhání, které jste vyvolali, nejen operace gitu:

- **Operace gitu**: fetch, push, pull, checkout, merge, rebase, commit, stash...
- **Klonování** (a vytvoření nového repozitáře): „Klonování selhalo“.
- **Volání GitHubu nebo GitLabu**: seznamy repozitářů ve výběrech a v dialogu Klonovat, vytvoření pull requestu nebo issue, kontrola tokenu nebo klíče SSH na stránce Integrace: „Požadavek na hostingovou službu selhal“.
- **Externí program**: editor, nástroj pro diff nebo merge, správce souborů, prohlížeč, terminál nebo spouštěč, editor obrázků, tlačítka *Vyzkoušet* a *Otestovat* v Nastavení: „Externí program selhal“.
- **Požadavek na AI** (zpráva commitu, vysvětlení, návrh řešení konfliktu...): „Požadavek na AI selhal“.
- **Soubor, který se nepodařilo zapsat**: Uložit ve vestavěném editoru, editor merge, nastavení, které se nepodařilo uložit: „Uložení selhalo“.

Červená zpráva je na stavovém řádku v každém zobrazení: na kartě repozitáře, v Domů, v Radaru, v Pracovních prostorech, v Nastavení (na spodním řádku obrazovky), pod Protokolem aktivity i pod dialogem. Tam, kde je stavový řádek zakrytý (Domů, Radar, Pracovní prostory, Nastavení), se zpráva píše přes jeho spodní řádek, dokud se na chybu nepodíváte. Sleduje vás z jedné karty na druhou. Operace, která ještě běží, když kartu přepnete, se nesleduje: jak skončila, uvidíte v Protokolu aktivity.

Seznamy v sekcích PULL REQUESTS a ISSUES, které se nepodaří obnovit, mají místo toho vlastní řádek v sekci (obnovují se samy, zpráva každých pár sekund by byla jen šum). Dialog, který je ještě otevřený (Klonovat, Vytvořit pull request...), si také ponechá vlastní jednořádkovou zprávu, takže můžete opravit pole a zkusit to znovu.

Záznam v Protokolu aktivity obsahuje všechno, co dřív ukazovalo vyskakovací okno:

- **Nadpis a důvod.** „Fetch: selhání“, pak jedna věta, která prostými slovy řekne proč, pak co dělat dál. Rozpoznává se: remote má commity, které nemáte (*nejdřív pull*), odmítnuté přihlašovací údaje (*zkontrolujte Nastavení > Integrace nebo svůj klíč SSH*), zastavení na konfliktech, místní změny, které by byly přepsány (*commitněte, stashněte nebo zahoďte*), nedostupný remote a jiný proces git, který drží zámek repozitáře. Cokoli jiného ukáže první řádek od samotného gitu. Klonování nebo volání hostingové služby, které selže stejným způsobem (odmítnuté přihlašovací údaje, nedostupný hostitel), je formulováno stejně; ostatní selhání dostanou jednu větu o tom, co selhalo („Repozitář se nepodařilo naklonovat.“, „Program se nepodařilo spustit.“).
- **Podrobnosti**, ukázané, když je záznam otevřený (`Enter`, nebo kliknutí, které ho otevřelo): operace, spuštěný příkaz git, jeho návratový kód a poslední řádky toho, co vypsal (u vestavěné knihovny gitu její třída chyby a kód). Přihlašovací údaje, tokeny a hlavičky `Authorization` se maskují dřív, než se cokoli zobrazí, zapíše do protokolu nebo zkopíruje.
- **Kopírovat** (tlačítko, nebo `c`) dá do schránky blok textu určený k vložení do rozhovoru s někým, nebo s AI, kdo ho dokáže vysvětlit: verzi GitPoulpi, systém, terminál, backend, selhanou operaci s jejími podrobnostmi a verzi gitu.
- **Tlačítko dalšího kroku**, když existuje, vedle Kopírovat (nebo `r`): *Nejdřív Pull* (push, který by remote odmítl), *Stash a zkusit znovu* (místní změny překážejí checkoutu, merge, rebase nebo pull), *Zkusit znovu* (remote byl nedostupný), *Otevřít Integrace* (přihlašovací údaje). Provede krok a zavře okno. U selhání jiného repozitáře, než je karta v popředí, se nabízí jen *Otevřít Integrace*: opakování by běželo ve špatném repozitáři.
- Bez myši nabízí paleta příkazů `Protokol aktivity: Otevřít poslední chybu`, `Protokol aktivity: Kopírovat poslední chybu` a `Protokol aktivity: Provést navrženou akci` (tlačítko nápravy záznamu pro chybu, kterou ukazuje stavový řádek).

Push, který remote odmítl, protože se posunul dál, si ponechává vlastní dialog (nejdřív pull, force push s lease, zrušit); fakta jsou přesto v Protokolu aktivity.

### Protokol aktivity

Tlačítko **Aktivita** na pravém konci stavového řádku (číslo na něm počítá chyby, na které jste se ještě nepodívali), `Ctrl+N` a pak `Shift+A` a `Zobrazení: Protokol aktivity` v paletě otevřou okno. `Ctrl+N` ukáže odznak na každé kartě, poli, zaškrtávacím políčku a tlačítku okna (i na první položce seznamu); stisknutí znaku udělá totéž co kliknutí na ně. Dvě karty, **Operace gitu** a **Aplikace**, přepínané `Tab`, kliknutím, `g` (Operace gitu) nebo `p` (Aplikace), a pod nimi pole **Rozsah**, které platí pro obě (viz níže):

- **Aplikace**: vlastní řádky protokolu programu, okna i daemona na pozadí, nejnovější první. Pole **Úroveň** (rozbalovací seznam: Chyba, Varování, Info, Ladění, Trasování; `l` ho otevře) říká, kolik se zobrazí; požadavek na větší podrobnost, než se zaznamenává, zvýší i úroveň záznamu (a uloží ji jako `log.level`). Pod ním má **Zobrazit** jedno zaškrtávací políčko na úroveň (Chyba, Varování, Info, Ladění, Trasování, každé s počtem řádků, které obsahuje), aby šlo řádky té úrovně skrýt či zobrazit, například abyste viděli jen chyby, a **Vše** / **Nic** zaškrtnou či odškrtnou všechna políčka. Klávesy `1` až `5` přepínají úrovně v tomto pořadí, `a` je Vše, `n` je Nic; paleta nabízí `Protokol aktivity: zobrazit nebo skrýt úroveň ...`, `Protokol aktivity: zobrazit všechny úrovně` a `Protokol aktivity: nezobrazovat žádnou úroveň`. Úrovně podrobnější než volba v poli Úroveň jsou zašedlé (nic se pro ně nezaznamenává; najeďte na jednu, abyste viděli proč). Zaškrtnutí se pamatují do ukončení a mají je jen karta Aplikace. **Filtr** (`/`) ponechá řádky, které obsahují váš text. **Kopírovat vše (filtrované)** zkopíruje, co je vidět (platí zaškrtnutí i filtr); **Otevřít složku protokolů** otevře složku ve správci souborů. Dlouhý řádek se nikdy neořízne: viz „Dlouhé řádky“ níže.

  ![Protokol aktivity, karta Aplikace: pole Úroveň, zaškrtávací políčka Zobrazit po úrovních s Vše a Nic, filtr a řádky protokolu](../images/activity-log.png)

**Vymazat protokol** (`x`, nebo příkaz palety `Protokol aktivity: Vymazat protokol aplikace`) je na kartě **Aplikace**: nejdřív se zeptá, pak vyprázdní soubory logu všech uchovávaných dnů i seznam na obrazovce. Nelze vrátit zpět a není v historii undo.

- **Operace gitu** (karta, na které se okno otevře): operace, nejnovější první, každá s časem, kdy proběhla, a dobou trvání (`Push - 123 ms`): otevření repozitáře, načtení grafu commitů, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Selhání, stavové zprávy, selhané klonování a Fetch all pracovního prostoru se tu vypisují také; `Enter` nebo kliknutí otevře záznam (jeho důvod, další krok, příkaz, návratový kód a výstup), `c` nebo jeho tlačítko Kopírovat ho zkopíruje, `r` nebo jeho tlačítko provede navržený krok. `Tab` (nebo `p`) přepne na kartu Aplikace; `Left`/`Right` posouvají dlouhé řádky do stran.

  ![Protokol aktivity, karta Operace gitu: každá operace s časem a dobou trvání, selhaný fetch červeně](../images/activity-log-repository.png)

- **Dlouhé řádky** (obě karty): záhlaví nebo řádek detailu širší než okno se nikdy neořízne. Pod seznamem se objeví vodorovný posuvník; posouvejte do stran přes `Left` / `Right` (předělitelné jako `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + kolečko nebo vodorovné kolečko, příkazy palety `Protokol aktivity: Posunout doleva` / `Protokol aktivity: Posunout doprava`, nebo posuvník (táhněte jezdec, nebo stiskněte dráhu, aby jezdec skočil pod ukazatel). `Home` vrátí na začátek a přepnutí karty začne druhou kartu u levého okraje. Zalamovaný text (shrnutí, navržená akce) zůstává zalomený.

- **Rozsah** (rozbalovací seznam pod kartami, `s` ho otevře, nebo příkaz palety `Protokol aktivity: Zvolit rozsah`): čeho se obě karty týkají. Řádky jsou **Tento repozitář (název)** (repozitář karty, ze které jste protokol otevřeli; z Domů se nenabízí, protože tam žádný není), **Všechny repozitáře**, každý **pracovní prostor** a každý repozitář, který okno zná. Z karty repozitáře se protokol otevře na *Tento repozitář*, z Domů (tlačítko Aktivita, `Ctrl+N` a pak `Shift+A`) na *Všechny repozitáře*; `Ctrl+N` a pak `Shift+G` a příkaz palety `Zobrazení: Globální protokol aktivity` se otevřou vždy na *Všechny repozitáře*, stejně jako kliknutí na červenou chybu selhání, jehož repozitář není kartou v popředí (například selhané klonování). Poslední volba se pamatuje do konce relace, zvlášť pro oba způsoby otevření. Na kartě Aplikace patří řádek k repozitáři, když o něm program psal (selhaný push, doba operace); pod *Všechny repozitáře* každý takový řádek říká, čí je (`[název]`). Řádek, který nepatří žádnému repozitáři (spuštění, nastavení, samotný daemon), a záznam, který nepatří žádnému (selhané uložení), jsou **celoaplikační**: zobrazují se pod každým rozsahem, a když je rozsah užší než *Všechny repozitáře*, mají značku `[celá aplikace]`, takže filtr nikdy neskryje selhání. Paleta má i `Protokol aktivity: Operace gitu` a `Protokol aktivity: Protokol aplikace` pro karty.

- **Kopírovat diagnostiku** (tlačítko na obou kartách, `d`, vždy celý protokol, ať je rozsah jakýkoli, a `Zobrazení: Kopírovat diagnostiku` v paletě) zkopíruje verzi, systém, terminál a backend, poslední selhání s podrobnostmi a posledních 200 řádků protokolu na zvolené úrovni: blok, který dáte AI, když je něco špatně.

Doby měří okno (od požadavku po odpověď daemona, včetně cesty tam a zpět), takže to, co dělá CLI nebo MCP, se neměří. `log.timings` (ve výchozím stavu zapnuto) doby vypne, `log.extended` (ve výchozím stavu vypnuto) vypisuje i práci na pozadí, pravidelné obnovení a auto-fetch; obojí je v Nastavení > Obecné vedle ostatních nastavení protokolu a každá operace se do protokolu aplikace stejně zapisuje (`info` pro to, oč jste požádali, `debug` pro pozadí).

Soubory protokolu se jmenují `gitpoulpi-YYYY-MM-DD.log`, uchovávají se ve výchozím stavu týden a nejvýše deset let (`log.retention_days`, 1 až 3650), čistí se při spuštění GitPoulpi a přesouvají se při `log.max_file_mb` (5 MiB). Kde leží: [referenční příručce konfigurace](config-reference.md#log-protokol-aplikace). Ze skriptu nese chyba `--json` neúspěšného příkazu stejná fakta pod `detail`; viz [Reference CLI](cli-reference.md).

Kliknutí na červenou zprávu otevře protokol na *Všechny repozitáře* se záznamem otevřeným. Seznam Operací gitu patří relaci; karta Aplikace uchovává protokol programu i po restartech.

**Úlohy na pozadí.** Fetch, push, pull, klonování nebo přenos LFS, Fetch all / Pull all / Clone missing pracovního prostoru, požadavek na AI a odesílaný pull request nebo issue jsou úlohy: pravý konec stavového řádku ukazuje běžící úlohu s `[x]`, které ji zruší, a kliknutí na její popisek (nebo `Ctrl+N` a pak `Shift+J`, příkaz palety `Úlohy: zobrazit úlohy na pozadí`) otevře panel s tlačítkem Zrušit u každé úlohy (také `c` nebo `Del`). Klonování běží na pozadí; požadavek na AI a procházení pracovního prostoru přežijí přepnutí karty a po návratu do jejich repozitáře se použijí. Úloha za bodem, odkud už zastavení není bezpečné, je zašedlá s uvedeným důvodem; zrušená úloha nechá repozitář, jak byl.

## Rychlé odpovědi

**Panel nástrojů a postranní panel ukazují místo ikon čtverečky nebo otazníky.**
Použijte v terminálu Nerd Font nebo libovolné písmo s pokrytím emoji. Graf a
text fungují i bez nich.

**Graf commitů je z písmen a znaků, ne z obrázků.** To je automatický
záložní režim: obrázky vyžadují terminál mluvící protokolem kitty graphics
(kitty, Ghostty, herdr se zapnutou kitty graphics) nebo protokolem iTerm2
(WezTerm, iTerm2, mintty), nebo sixel (Windows Terminal, foot, Konsole). Konzole Windows, tmux a zellij dostanou znaky.
Jak vynutit jedno či druhé, viz sekce o grafu níže a [Výběr terminálu](terminals.md).

**Myš nic nedělá, nebo výběr textu myší nefunguje.**
GitPoulpi zachytává myš, aby k němu kliknutí dorazila. Chcete-li vybírat text
samotným terminálem, podržte při tažení `Shift` (ve většině terminálů). Vybírat
lze také uvnitř diffu (tažení, pak `y`). Zachytávání myši vypnete v
*Nastavení > Klávesnice a myš > Myš zapnuta* (je třeba restart). Klávesnice
dosáhne na každý příkaz.

**Zkratka nefunguje.** Terminál, tmux nebo skript AutoHotkey může klávesu
převzít dřív, než ji GitPoulpi uvidí (`Ctrl+,` často dorazí jako obyčejná `,`
v terminálech Linuxu a macOS; `Ctrl+Shift+N` vypadá jako `Ctrl+N`). Použijte
alternativy (`Ctrl+N` a poté `,` otevře Nastavení; `Ctrl+P` dosáhne na každý
příkaz) nebo akord přemapujte: viz [Nastavení](settings.md#klávesnice-a-myš).

**Nepamatuji si zkratku.** Stiskněte `Ctrl+P` a napište, co chcete; každý
výsledek ukazuje svou zkratku. `?` vypíše vše.

**Volba v Nastavení je zašedlá.** Závisí na jiné volbě. Řádek říká, na které
(například *Síla podbarvení řádku* potřebuje, aby *Pozadí drah grafu* bylo nastaveno na
*Podbarvení řádku*).

**„Auto-stash pending“ nad Logem.** GitPoulpi stashnul vaše změny, aby mohl
provést operaci, a nedokázal je vrátit. Použijte tlačítka banneru **Pop teď**,
**Zobrazit stash**, **Začít znovu** (vrátí operaci a vrátí vaše změny) nebo
**Drop**; viz [Auto-stash](log-and-history.md#auto-stash).

**Agent hlásí `no_viewer` nebo `deferred`.** `no_viewer`: pro daný repozitář
není otevřeno žádné okno GitPoulpi (nebo je vypnuto *Povolit AI agentům
ovládat zobrazení*). `deferred`: okno ukazuje čip; `Ctrl+N Enter` jej přijme.
Viz [MCP a agenti](mcp-and-agents.md#zobrazování-věcí-uživateli).

**Po otevření GitPoulpi vidím prázdné okno nebo `log error: backend error`.**
Ověřte, že jste jej spustili uvnitř repozitáře Git (jinak skončíte na kartě
Domů: použijte **Procházet**, **Klonovat** nebo **Init**). Poté spusťte
`gitpoulpi daemon status`; `gitpoulpi daemon restart` restartuje proces na
pozadí pro tento repozitář. Obnovu vynutíte klávesou `r`.

**Dvě okna na stejném repozitáři se chovají divně.** Záměrně sdílejí jednoho
daemona na repozitář, takže změna v jednom se projeví v druhém. Zdá-li se, že
daemon uvázl, `gitpoulpi daemon stop --all` zastaví daemony všech relací; další
okno spustí nového.

**Commit, pull nebo push žádá heslo, nebo selže ověření.**
GitPoulpi pro síť používá credential helper a ssh-agent vašeho `git`. Nejprve
se přesvědčte, že `git fetch` funguje v běžném terminálu. Tokeny GitHub/GitLab
pro pull requesty jsou oddělené a nepovinné: viz [Hostingové služby a issue trackery](hosting-integration.md).

**Fetch, pull nebo push selže s „The configured credential helper failed“, „The SSH host key of the remote is not known yet“ nebo odmítnutým přihlášením a jeho záznam v Protokolu aktivity má tlačítko „SSH a přihlášení...“.**
Selhání pochází z toho, jak se tento repozitář přihlašuje. Tlačítko otevře
Nastavení > SSH a přihlášení, kde pro tento repozitář vyberete jiný zdroj:
vlastní pomocníky gitu s ignorovanými pomocníky repozitáře, účet integrace,
soubor klíče (se souborem známých hostitelů), nebo vlastní helper či příkaz pro
token; **Otestovat připojení** pak zkontroluje každý remote. U pomocníka, který
nešel spustit, ukáže `git config --show-origin --get-all credential.helper`,
kde je nastaven.

**„GitPoulpi did not send the account token to ...“.** Remote repozitáře
ukazuje na server, který není adresou, ke které je zvolený účet vázán (často
přepis `url.<x>.insteadOf` v konfiguraci samotného repozitáře), takže se token
zadržel. Opravte remote nebo na stránce zvolte jiný zdroj.

**„This repository's token command failed“, „... changed since you confirmed it“, „This repository's sign-in cannot be used“.**
Volby ze stránky nelze provést: příkaz pro token nevytiskl žádný token (jeho
poslední chybové řádky jsou citovány, bez tajemství), skript, který spouští, se
od potvrzení změnil (potvrďte příkaz na stránce znovu), nebo volba odkazuje na
soubor klíče či účet, které už neexistují. Každé z těchto hlášení má tlačítko
**SSH a přihlášení...**.

**OpenSSH hlásí „bad permissions“ u souboru klíče.** Klíč mohou číst jiné účty.
V Nastavení > SSH a přihlášení řádek **Soubor soukromého klíče** ukazuje varování
s tlačítkem **Omezit na můj účet**.

**„staging...“ zůstává ve stavovém řádku.** Stále běží obnovení na pozadí; zmizí, až se
aktualizuje seznam souborů. `r` vynutí obnovu.

**Velmi velký repozitář je pomalý.** Snižte *Počáteční commity v grafu* na
stránce Nastavení > Log a graf (Log načítá další při posouvání, je-li zapnuto
*Líné načítání commitů v grafu*); skryjte sloupec *Změny* v Logu a vypněte
avatary; vypněte *Zpět povoleno*, pokud zpět nepotřebujete (pak se při stage a
commitech nevede žádná evidence).
`GITPOULPI_TIMING=1` zapisuje časování jednotlivých požadavků do
`gitpoulpi-timing.log` v dočasné složce, což stojí za to přiložit k hlášení o
výkonu.

**Tlačítka AI chybí nebo jsou zašedlá.** AI je ve výchozím stavu vypnuto; viz
[Asistence AI](ai.md#řešení-potíží).

**GitPoulpi se zavřel sám.** Každé ukončení se zaznamenává: hledejte
`gitpoulpi-tui-exit.log` (a `gitpoulpi-tui-panic.txt` po pádu) v konfiguračním
adresáři (`gitpoulpi config path` vypíše soubor nastavení, který je právě
tam), a přiložte jej k issue.

**Kde jsou moje nastavení?** `gitpoulpi config path`. Stav jednotlivých
repozitářů je v `.gitpoulpi/repo-state.toml` uvnitř repozitáře.

**Posílá GitPoulpi něco někam?** Jen to, o co požádáte: operace gitu na vaše
remoty, požadavky na hosting po připojení účtu, požadavky AI po stisknutí
tlačítka AI a, pokud zapnete avatary, hash e-mailů autorů službě Gravatar.
Žádná telemetrie neexistuje. Viz [Asistence AI](ai.md#soukromí).

## Každý zápis selže: „Unable to create index.lock“

Příkaz git, který zhavaroval (pád, zabitý terminál), může po sobě zanechat
`.git/index.lock` a pak selže každá změna repozitáře. Jakmile je soubor starší
než 10 sekund, GitPoulpi nad Logem ukáže pruh: **Odstranit index.lock**
(paleta: *Repozitář: Odstranit zastaralý index.lock*; CLI:
`gitpoulpi index-lock remove`). Nejprve se zeptá a smaže jen ten soubor.
GitPoulpi nedokáže spolehlivě poznat, zda jiný program git ještě běží, takže
před potvrzením zkontrolujte, že žádný neběží (IDE, jiný terminál). V
připojeném worktree je zámek vlastní zámek worktree v `.git/worktrees/<name>`.

## „git am“ se zastavil a Rebase Abort nic nedělá

`am` (aplikace záplat z mailboxu) není rebase. Zastavený `git am` má vlastní
pruh, **Aplikace patchů (git am)**, s tlačítky Vyřešit konflikty, Pokračovat,
Přeskočit a Přerušit (paleta: *git am: Pokračovat / Přeskočit tento patch /
Přerušit*; CLI: `conflicts am-continue|am-skip|am-abort`). Přerušit se nejprve
zeptá, zahodí dosud aplikované záplaty a vrátí větev tam, kde byla. Stejně jako
tlačítka rebase nejsou vratné.

## Zpět je zašedlé a říká „Nelze vrátit ...“ (blokovaná položka)

Nejnovější položka žurnálu uvázla: repozitář se posunul mimo GitPoulpi, nebo
byla položka přerušena. Najeďte na Zpět pro důvod; pokud uvázla natrvalo,
tooltip odkáže na **Úpravy: Zapomenout blokovanou položku vrácení** (paleta)
nebo `gitpoulpi undo-forget`. Nejprve se zeptá a jen odstraní položku ze
žurnálu; repozitáře se nedotkne. Viz [Zpět a znovu](undo-redo.md).

## Velké soubory se zobrazují jako tři řádky textu („soubory LFS se zobrazují jako ukazatele“)

Soubor, který má být obrázek, model nebo velká binárka, obsahuje `version https://git-lfs.github.com/spec/v1` a řádek `oid sha256:...`: je to **ukazatel** Git LFS a skutečný obsah nebyl stažen (nebo nejsou nainstalovány filtry LFS).

- Podívejte se na panel Stav: repozitář, který používá LFS, zatímco chybí `git-lfs`, ukazuje jantarový banner. Nainstalujte program (https://git-lfs.com; Windows: součást Git for Windows nebo `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) a znovu otevřete nebo stiskněte `r`.
- `gitpoulpi lfs status` říká, zda je program nainstalován, zda jsou nastaveny filtry a hook a které vzory se sledují; `gitpoulpi lfs files` vypíše soubory s ukazateli a které z nich jsou `local`.
- S nainstalovaným `git-lfs` spusťte v paletě **`LFS: Pull objektů`** (nebo `gitpoulpi lfs pull`): stáhne objekty a nahradí ukazatele. Pokud pro tento repozitář nejsou žádné filtry LFS, nejprve je nainstaluje.
- Pokud stahování selže, zpráva uvede důvod (přihlašovací údaje k remotu, server, který už objekt nemá). GitPoulpi nepotřebuje vlastní účet: LFS používá obvyklé přihlašovací údaje vašeho remotu (credential helper nebo ssh-agent).
- Klon repozitáře s LFS nikdy neselže kvůli LFS; pokud se krok stahování nemohl provést, klon existuje s ukazateli a `LFS: Pull objektů` jej později dokončí. `gitpoulpi clone --no-lfs` stahování přeskočí záměrně.
- Veškeré LFS můžete vypnout v Nastavení > Git a kódování > Git LFS (nebo `[git] lfs_enabled = false`).

## „git-lfs not found“ / „Git LFS is not installed“

Zpráva `Git LFS is not installed. This repository stores large files with Git LFS; ...` (nebo dialog s tlačítkem *Otevřít stránku instalace*, nebo vlastní `git-lfs: command not found` / `'lfs' is not a git command` z gitu) znamená, že repozitář je nakonfigurován pro LFS - jeho `.gitattributes`, filtry v `.git/config` nebo hook `pre-push` volají `git-lfs` - a program není v `PATH`, který GitPoulpi vidí. Nainstalujte jej (odkaz výše). Na Windows se program dodává s Git for Windows; pokud `git lfs version` v terminálu funguje, ale GitPoulpi stále říká, že chybí, spusťte GitPoulpi z terminálu, který vidí stejný `PATH`. Chcete-li LFS před GitPoulpi skrýt a vlastní chování gitu ponechat beze změny, vypněte `git.lfs_enabled`.

## Graf je vykreslen znaky, ne jako obrázek (herdr, tmux, jiné terminály)

GitPoulpi kreslí sloupec Graph a avatary autorů jako pixely jen tam, kde si je jistý, že je terminál zobrazí: vložené obrázky iTerm2 ve „holém“ WezTerm, iTerm2 a mintty; protokol Kitty graphics v kitty, Ghostty a **herdr, když vlastní konfigurace herdr nastavuje `kitty_graphics = true`** (`[terminal]`, nebo `[experimental]` u starších herdr) a herdr běží uvnitř WezTerm, kitty nebo Ghostty. Obrázky sixel se používají ve Windows Terminal, foot, Konsole a mlterm. Všude jinde kreslí znaky, což je vždy správně.

- V herdr se zapnutým Kitty graphics, ale s konfigurací, kterou GitPoulpi nenajde (nebo při spoléhání na výchozí hodnotu herdr), nastavte `ui.image_protocol = "kitty"` (Nastavení > Log a graf > Protokol obrázků) a restartujte.
- V terminálu, který na místě grafu ukazuje čtverečky nebo změť znaků, nastavte `ui.image_protocol = "none"` nebo `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` vypne obrázky bez ohledu na konfiguraci.

**herdr vzdáleně (`herdr --remote <host>`, nebo `ssh` a poté `herdr`): výchozí je textový graf.** Přes SSH běží GitPoulpi na vzdáleném hostiteli a může číst jen prostředí vzdáleného stroje a `config.toml` herdr. O tom, zda se obrázek Kitty objeví, rozhoduje *lokální* klient herdr (jeho vlastní nastavení `kitty_graphics` a jeho vlastní terminál), který GitPoulpi nevidí: když klient obrázky nekreslí, buňky zůstanou prázdné. GitPoulpi proto kreslí textový graf vždy, když zjistí herdr přes SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Pokud má váš lokální herdr `kitty_graphics = true` a terminál kreslící obrázky Kitty (kitty, Ghostty), zapněte je na vzdáleném stroji pomocí `ui.graph_renderer = "image"` (nebo `ui.image_protocol = "kitty"`). Obrázky se posílají inline (`t=d`), což funguje přes jakýkoli proud bajtů. Pokud je sloupec prázdný, nastavte `ui.graph_renderer = "text"`.

**Světlejší obdélník za drahami.** Obrázek je průhledný (PNG RGBA s přímým alfa kanálem) všude, kde se nic nekreslí, takže prosvítá pozadí terminálu a výběrový pruh. Terminály s protokolem Kitty a **WezTerm** tuto průhlednost skládají; GitPoulpi pak pod obrázek vyhradí běžné prázdné buňky a pošle jej až za nimi. Všechny ostatní terminály s protokolem iTerm2 (samotný iTerm2, Rio, mintty) alfa PNG podle všeho nemíchají, takže tam obrázek nese neprůhledné pozadí v barvě motivu, které se může jevit jako obdélník v terminálu s jiným pozadím; `ui.graph_renderer = "text"` jej odstraní. S `ui.graph_lane_background = "off"` se na průhledných cestách za drahami nemaluje nic.

Viz `ui.graph_renderer` a `ui.image_protocol` v [referenční příručce konfigurace](config-reference.md).
