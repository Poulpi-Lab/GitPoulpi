[English](../pull-requests-and-issues.md) · **Čeština** · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requesty a issues

Čtení, vytváření a správa pull / merge requestů a issues z postranního panelu, tabule a panelu nástrojů. Nejdřív připojte účet a propojte repozitář: viz [Hostingové služby a issue trackery](hosting-integration.md). Vše, co se říká o jedné issue nebo žádosti (komentáře, kontroly, vlákna), je v [Konverzacích](conversations.md); čtení a kontrola žádosti je v [Kontrole pull requestu](review.md).

## Pull requesty a issues v postranním panelu

### PULL REQUESTY

Sekce má záhlaví s počtem a `+` (vytvořit pull request), rozbalovací seznamy
poskytovatele a projektu, vyhledávací pole a rozbalovací seznam stavu
(*Otevřené*, *Merged / zavřené*, *Všechny*). Pak následují čtyři skupiny, každá
se svým počtem:

- **Moje pull requesty**: jste autor.
- **Přiřazené mně**.
- **Čekají na mou kontrolu**: jste požádáni o kontrolu.
- **Všechny pull requesty**: na začátku sbaleno.

Vyhledávací pole filtruje každou skupinu (název, číslo, autor, větev). Každá
změna rozbalovacího seznamu stavu se znovu ptá hostitele.

Řádek začíná glyfem stavu: `●` otevřený (zeleně), `○` koncept, `◆` sloučený
(fialově), `✕` zavřený (červeně). Požadavek aktuálně přepnuté větve je
tučně. Řádky vypadají `#12  title` (GitHub, Bitbucket) nebo `!53  title`
(GitLab, Azure DevOps).

### ISSUES

Název sekce uvádí poskytovatele: `GITHUB ISSUES`, `JIRA ISSUES` a tak dále. U
Azure DevOps zní `... PRACOVNÍ POLOŽKY` a u Trella `... KARTY`. Pod
rozbalovacími seznamy: vyhledávací pole, rozbalovací seznam stavu (*Otevřené*,
*Přiřazené mně*, *Zavřené*, *Všechny*) a issues. Řádek Jiry vypadá
`ABC-12  title`, přičemž typ a stav jsou ztlumeny, je-li postranní panel dost
široký.

![Sekce ISSUES v postranním panelu s otevřenou nabídkou po kliknutí pravým tlačítkem na issue #7, první je Otevřít konverzaci](../images/issues-sidebar.png)

### Akce na řádku

- `Enter` nebo dvojklik otevře řádek: pull request v [kontrole](review.md) (GitHub, GitLab, Gitea, Forgejo, Gerrit), issue a žádost na Azure DevOps nebo Bitbucketu jako [konverzaci](conversations.md) v okně a cokoli jiného ve vašem prohlížeči. **Otevřít v prohlížeči** v nabídce řádku vždy otevře prohlížeč.
- Pravé tlačítko myši nebo `.` otevře nabídku řádku:
  - **Otevřít konverzaci** (issues a žádosti, které nemají pohled kontroly), **Otevřít v prohlížeči**, **Kopírovat URL**,
    **Kopírovat číslo**.
  - U pull requestu: **Upravit na <provider>**, **Checkout zdrojové větve**,
    **Zobrazit commity zdrojové větve**. U požadavku z forku zní checkout
    **Checkout pull request (fork)** a **Zobrazit** je zašedlé.
  - U issue: **Vytvořit větev z issue**. Otevře výzvu k nové větvi s názvem
    jako `12-fix-the-thing` (u Jiry `ABC-123-fix-the-thing`).
  - Na GitHubu, GitLabu, Gitea a Forgejo má řádek pull requestu také **Zkontrolovat...**
    (také **Enter** na řádku): kontrola v aplikaci, viz
    [Kontrola pull requestu](review.md).
  - Na GitHubu, GitLabu, Gitea a Forgejo mají oba druhy řádků také **Upravit řešitele...**,
    **Upravit štítky...** a **Nastavit milník...** (a u pull requestu **Upravit recenzenty...**): viz
    [Řešitelé, recenzenti, štítky a milník](#řešitelé-recenzenti-štítky-a-milník).
- **Upravit na <provider>** má tři další vstupy: tužka `✎` na pravém konci
  řádku pull requestu pod kurzorem nebo vybraného, `e` na vybraném řádku a
  příkaz z palety *Integrace: Upravit propojený pull request u hostitele* (pro
  požadavek aktuálně přepnuté větve).
- Pokud zdrojová větev v repozitáři ještě není, GitPoulpi to oznámí. Nejdřív
  proveďte fetch (`Ctrl+G f`).

**Checkout pull requestu.** GitPoulpi načte požadavek do lokální větve
`pr/<n>` (`mr/<n>` u GitLabu), která ho sleduje, a přepne na ni. U GitHubu,
GitLabu, Gitea a Forgejo to funguje i pro forky (GitPoulpi načte zvláštní head ref hostitele).
Azure DevOps a Bitbucket načtou zdrojovou větev požadavku. Požadavek z forku
se tam odmítne: přidejte fork jako remote a jeho větev přepněte sami. Zpět
vrátí na předchozí větev. CLI: `gitpoulpi hosting checkout-pr <n>`.

**Kontrola pull requestu z příkazové řádky (a agentem).** Na GitHubu
(včetně Enterprise), GitLabu (včetně self-managed), Gitea a Forgejo lze žádost
číst a kontrolovat, aniž byste opustili GitPoulpi: `hosting review show <n>` (soubory,
stav sloučení, kontroly), `hosting review threads <n>` (vlákna kontroly a
konverzace), `hosting review comment`, `hosting review reply`, `hosting review resolve`
a `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Diff pochází z vašeho repozitáře, ne od
hostitele: `hosting review fetch <n>` stáhne head žádosti do
`refs/gitpoulpi/pr/<n>` a její cílovou větev do `refs/gitpoulpi/pr/<n>-base`
(ani jedno není větev, nic se nepřepíná). Čísla řádků jsou z tohoto
diffu: `new` je řádek v headu, `old` řádek v merge base. GitLab
nemá stav kontroly „request changes“: odmítne se, nic se neodešle.
Gitea a Forgejo berou komentář jen k jednomu řádku (žádné rozsahy); vlákno jsou
komentáře k jednomu řádku souboru. Gitea umí vlákno vyřešit, Forgejo 11 ne
(hostitel pro to nemá endpoint: volání to řekne). Odpověď na serveru
bez endpointu pro odpovědi se odešle jako kontrola s jedním komentářem ke stejnému řádku.
Gitea odmítne schválit nebo zamítnout vaši vlastní žádost.
Bez klonu: přidejte `--project owner/name` (GitLab: `group/project`) k `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` a `suggest`, abyste zkontrolovali projekt, který není aktuální repozitář;
`hosting review diff <n> --project P` čte diff (hunky po souborech) z API hostitele a
`hosting file <path> --project P [--ref R]` čte soubor v dané revizi (GitHub, GitLab, Gitea, Forgejo a Gerrit; u Gerritu se diff
patch setu čte soubor po souboru a soubor pochází z větve nebo, pro id commitu, z commitu). Soubor, který hostitel
v diffu neukáže (příliš velký), je označen `omitted`.
Nástroje MCP jsou stejné příkazy (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Řešitelé, recenzenti, štítky a milník

Na GitHubu (včetně Enterprise), GitLabu (včetně self-managed), Gitea a
Forgejo můžete, aniž byste opustili GitPoulpi, změnit, komu je pull request nebo issue přiřazen, kdo ho kontroluje, jeho štítky
a milník. Jiní poskytovatelé nic z toho nezobrazují.

- **Nabídka řádku**: **Upravit řešitele...**, **Upravit recenzenty...** (jen pull
  requesty), **Upravit štítky...**, **Nastavit milník...**.
- **Paleta**: *Hosting: Upravit řešitele...*, *Hosting: Upravit recenzenty...*,
  *Hosting: Upravit štítky...*, *Hosting: Nastavit milník...* nejdřív vypíšou
  žádosti a issues z postranního panelu k výběru. V
  [Radaru](radar.md) stejné čtyři příkazy působí na vybraný řádek.
- Výběr se otevře hned a z hostitele načte lidi nebo štítky projektu (a co
  položka má teď). Po zhruba 150 ms se ukáže řádek se spinnerem.
  Psaním filtrujete (fuzzy shoda), `Up` / `Down` pohybují, `Space` zaškrtne
  nebo odškrtne, `Enter` použije, `Esc` zruší, `Tab` dosáhne na tlačítka. Kliknutí
  na řádek ho zaškrtne a kolečko posouvá. Lidé mají odznak s iniciálami,
  štítky svou barvu.
- Milník je rozbalovací seznam (s volbou **Bez milníku**, která ho zruší).
- Po **Použít** se aktualizuje hostitel a seznamy i Radar ukážou
  změnu po svém dalším obnovení, které začne hned.

Formuláře vytvoření (pull requestu, issue) mají stejné volby jako řadu
tlačítek: `Řešitelé: žádný`, `Recenzenti: žádný`, `Štítky: žádný`,
`Milník: žádný`. Na GitHubu, GitLabu, Gitea a Forgejo tlačítko Štítky nahrazuje psané
pole štítků formuláře issue. Když hostitel nový pull request přijme, ale
odmítne některé jeho štítky nebo lidi, řekne to stavový řádek.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>` a `--assignee`, `--reviewer`, `--label`,
`--milestone` u `hosting pr` a `hosting issue create`; viz
[referenční příručka CLI](cli-reference.md).

### Klávesy v sekcích

| Klávesa | Akce |
|---|---|
| `Enter` / dvojklik | otevře konverzaci v GitPoulpi, kde ji hostitel má (pull request na GitHubu, GitLabu, Gitea nebo Gerritu otevře kontrolu), jinak prohlížeč |
| `o` | otevře řádek v prohlížeči |
| `.` / pravé tlačítko | nabídka řádku |
| `e` | upraví vybraný pull request u hostitele |
| `n` | nový pull request (PULL REQUESTY) nebo nové issue (ISSUES) |
| `v` | otevře rozbalovací seznam stavu |
| `r` | obnoví |
| `Enter` / `Space` na záhlaví | sbalí nebo rozbalí sekci |

### Znovu připojit a offline

Chybí-li token, byl odmítnut nebo mu chybí scope, první řádek sekce zní
`Znovu připojit (token odmítnut)`. Kliknutím nebo klávesou `Enter` otevřete
profil v Nastavení > Integrace.

Když hostitele nelze dosáhnout, poslední seznamy zůstanou na obrazovce a
záhlaví ukazuje jejich stáří (`· 5m old`). Poslední odpověď se také ukládá do
mezipaměti na disku (`.gitpoulpi/hosting-cache.json`), takže se seznamy
zobrazí dřív, než odpoví síť.

## Tabule issues

Tlačítko **Tabule** v záhlaví ISSUES (paleta: **Issues: Otevřít tabuli**) ukáže issues propojeného zdroje jako tabuli Kanban
přes celé okno. Sloupce závisejí na zdroji:

| Zdroj | Sloupce | Přesun karty |
|---|---|---|
| Lokální poznámky Markdown | stavy (`statuses` profilu, potom další používané) | přepíše `status` poznámky |
| GitHub, GitLab, Gitea, Forgejo | **Otevřené**, potom vámi zvolené štítky, potom **Zavřené** | vymění štítek stavu; do **Zavřené** issue zavře, zpět do sloupce ho znovu otevře |
| Trello | otevřené seznamy boardu, zleva doprava | přesune kartu do seznamu |
| Redmine | stavy issues | změní stav issue (platí pravidla workflow Redmine) |
| Jira (Cloud i Data Center) | stavy projektu, nejdřív to-do, nakonec hotovo | provede přechod workflow, který ke sloupci vede (`GET`, pak `POST .../transitions`); bez takového přechodu se karta vrátí a stavový řádek řekne `no transition from X to Y` |
| Azure DevOps (Services i Server) | stavy typů pracovních položek projektu, podle kategorie stavu | nastaví `System.State`; pracovní položka, jejíž typ stav sloupce nemá, se vrátí s důvodem (`a Bug has no state Done`) |

U projektu GitHub nebo GitLab se **Sloupce** (nebo `c`, nebo **Tabule: Nastavit štítky sloupců...**) zeptají na štítky, které jsou
sloupci, v pořadí, například `status::todo, status::doing, status::review`. Navrhnou štítky projektu, které
vypadají jako stavy. Volba se ukládá pro repozitář. Bez štítků je tabule Otevřené / Zavřené. Přesun karty
se dotkne jen vámi uvedených štítků stavu: její další štítky zůstanou. Tabule hostitele uvádí otevřené issues a 60 naposledy
zavřených.

Přechod v Jiře, který žádá povinná pole (například rozlišení), Jira odmítne a karta se vrátí
s vlastní zprávou Jiry: takový přechod proveďte na webu.

Tutéž tabuli mají k dispozici skripty a agenti: `gitpoulpi hosting board` vypíše sloupce s kartami
a `gitpoulpi hosting board move <card> <column>` jednu přesune (viz [referenční příručka CLI](cli-reference.md)).

Karta ukazuje číslo, název, řešitele jako iniciály, štítky a milník. Přetáhněte ji do jiného sloupce,
nebo `Shift+Left` / `Shift+Right`, nebo **Tabule: Přesunout kartu do...** (`m`). Přesune se hned; po zhruba 150 ms spinner
ukáže, že hostitel ještě pracuje, a když hostitel odmítne, karta se vrátí a stavový řádek řekne proč. `Enter` otevře
[konverzaci](conversations.md) issue v GitPoulpi a po jejím zavření se tabule vrátí; `e` upraví řešitele, štítky a milník; `n` založí nové issue ve
vybraném sloupci; `a`, `l`, `M` a `/` filtrují podle řešitele, štítku, milníku a textu, `x` filtry zruší; `r`
znovu načte data od hostitele. Tabule se načte znovu, když se obnoví seznamy postranního panelu. Radar tabuli nemá.

## Vytvoření pull requestu

Formulář otevřete tlačítkem `+` v záhlaví PULL REQUESTY, klávesou `n` v sekci
nebo příkazem z palety *Pull requesty: Vytvořit pull request...*. Nabídka větve
má také **Založit pull request z <branch>...**.

Formulář obsahuje:

- Jednu kartu pro každý připojený profil, který má pull requesty (`Left` /
  `Right` v pásu karet).
- Výběry **Z repozitáře / Větev** a **Do repozitáře / Větev**. Dlouhé seznamy
  lze prohledávat. Můžete zadat větev nebo repozitář, který seznamy nezobrazují.
- **Název**, předvyplněný předmětem posledního commitu.
- **Popis** na více řádků (`Enter` přidá nový řádek, `Tab` pole opustí).
- Tlačítka **Řešitelé**, **Recenzenti**, **Štítky** a **Milník** (GitHub,
  GitLab, Gitea a Forgejo), která otevírají výběr.
- **Odeslat jako koncept**.
- Při zapnuté [AI](ai.md): **Vygenerovat název a popis**.

Formulář se otevře s profilem, repozitářem a větvemi aktuálně přepnuté větve
(`feature -> main`). Jiný zdrojový repozitář vytvoří požadavek z forku.

**Vytvořit pull request** (nebo `Ctrl+S`) požadavek na hostiteli vytvoří.
Výsledek se otevře v prohlížeči a stavový řádek ho pojmenuje.

## Vytvoření issue

Má-li sekce ISSUES poskytovatele **i** projekt, objeví se v jejím záhlaví `+`
(*+ Nové issue*). Dále: `n` v sekci nebo příkaz z palety *Hosting: Nové
issue...*.

Formulář obsahuje **Název** (povinný), **Popis** (více řádků) a **Štítky**
(oddělené čárkou, například `bug, ui`; na GitHubu, GitLabu, Gitea a Forgejo místo toho
výběr otevírají tlačítka Štítky, Řešitelé a Milník). Jira a Azure DevOps přidávají
rozbalovací seznam **Typ**. Trello přidává rozbalovací seznam **Seznam**. Linear a YouTrack
přidávají rozbalovací seznam **Stav** (stav workflow, nebo hodnotu `State`, ve kterém issue
začíná). Redmine přidává rozbalovací seznam **Typ** s trackery projektu (Bug, Feature,
Support...); štítky nemá, takže pro něj tlačítko Štítky není a štítek zadaný
na příkazové řádce se odmítne. Tyto
seznamy formulář při otevření naplní daty od hostitele.

**Vytvořit issue** (nebo `Ctrl+S`) issue na hostiteli vytvoří. `Esc` nebo
**Zrušit** formulář zavře. Pokud hostitel odmítne (odmítnutý token, chybějící
scope, neznámý projekt, neznámý štítek), formulář zobrazí důvod a zachová, co
jste napsali. Po úspěchu stavový řádek pojmenuje nové issue a sekce ISSUES se
obnoví.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Vytvoření na vlastní webové stránce hostitele

Formulář Nové issue i formulář Vytvořit pull request mají vpravo nahoře
tlačítko **Vytvořit na <provider>**. Klávesa je `Alt+W`. V prohlížeči otevře
webový formulář hostitele, předvyplněný tím, co jste dosud zadali. Dialog zůstane
otevřený. Nepotřebuje token a nevolá žádné API, takže funguje i po chybě API.

- GitHub nemá pro *draft* žádný parametr URL, proto se zaškrtávací políčko konceptu
  nepřenáší. GitLab dostane předponu názvu `Draft: `.
- GitLab přijímá štítky jen podle id, proto se přidají jako rychlá akce
  `/label ~"x"` na konci popisu.
- Velmi dlouhý popis se zkrátí, aby URL zůstala pod 6000 znaky, s poznámkou
  `[...truncated: paste the rest]`.
- Bitbucket Data Center nemá adresu webového formuláře, proto nemá tlačítko
  **Vytvořit na <provider>**.

CLI: `gitpoulpi hosting issue create ... --web` a `gitpoulpi hosting pr --web`
vypíší stejnou URL.

## Čip na panelu nástrojů

Na konci panelu nástrojů (ve výchozím stavu vpravo) čip ukazuje požadavek aktuálně přepnuté větve:
`GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) nebo `BB #12`
(Bitbucket). Jeho barva značí stav: zelená otevřený, šedá koncept, fialová
sloučený, červená zavřený.

- Najetím zobrazíte název, stav a `source → target`.
- Kliknutím nebo `Ctrl+G g` otevřete požadavek v prohlížeči.
- Pravým tlačítkem získáte **Otevřít v prohlížeči**, **Kopírovat URL**,
  **Obnovit** a **Přepnout profil**.

Pushnutá větev bez požadavku ukazuje ztlumený čip `+ PR`. Kliknutím založíte
pull request. Čip se nezobrazí, když žádný profil neodpovídá, větev není
pushnutá nebo se hostitele nepodařilo dosáhnout. V úzkém okně je čip skrytý,
aby zbylo místo pro tlačítka panelu nástrojů.

## Kdy se seznamy obnovují

Seznamy se obnovují:

- při otevření repozitáře;
- při změně aktuálně přepnuté větve;
- po push nebo fetch;
- každých N minut podle *Obnovení (minuty): pull requesty a issues*
  (`general.hosting_refresh_minutes`, výchozí 5, `0` časovač vypne;
  Nastavení > Integrace);
- při zavření Nastavení;
- při stisku `r`, spuštění *Integrace: Obnovit pull requesty a issues* nebo
  změně filtru.

Požadavky nikdy neblokují okno.

Hostitelé omezují počet volání. GitPoulpi odesílá zpět `ETag` poslední
odpovědi, takže nezměněný seznam stojí málo (na GitHubu nic). Když hostitel
oznámí, že zbývá málo volání, stavový řádek to uvede. Když nezbývá žádné,
GitPoulpi se tohoto hostitele přestane ptát do obnovení limitu a seznamy zůstanou
beze změny.

## Terminologie

GitHub, Azure DevOps a Bitbucket říkají „pull request“. GitLab říká „merge
request“. *Terminologie žádosti o kontrolu kódu* (`ui.pr_terminology`, v Nastavení >
Integrace) volí slovo pro každý popisek, nabídku, příkaz z palety a zprávu:

- **Auto (podle hostitele)**: řídí se propojeným poskytovatelem repozitáře
  („Pull Request“, dokud není nic propojeno).
- **Pull Request** nebo **Merge Request**: vždy toto slovo.

Změna se projeví okamžitě. CLI a MCP vždy říkají „pull/merge request“. Tato
stránka říká „pull request“.

## Odkazy ve zprávách commitů a názvech větví

Jsou-li issues nebo pull requesty repozitáře propojeny, odkazy ve zprávě commitu
v Podrobnostech commitu jsou podtržené. Kliknutí otevře propojenou stránku.
Klávesnice: příkaz z palety
*Integrace: Otevřít odkaz z commitu (#issue / !request)...* (nabídka, má-li
zpráva několik odkazů). Repozitář bez propojení nemá žádné odkazy.

Co je odkaz, závisí na poskytovateli sekce ISSUES:

| ISSUES propojeno s | Ve zprávě | V názvu větve |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` a `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, jen když je `ABC` klíč propojeného projektu. Text jako `UTF-8`, `SHA-256` nebo `ISO-8601` nikdy není odkaz. | `ABC-123` kdekoli v názvu (`feature/ABC-123-x`), velkými písmeny |
| Linear | `ENG-123`, jen když je `ENG` klíč propojeného týmu (nebo týmu ze seznamu týmů profilu). `UTF-8` nikdy není odkaz. | vlastní `branchName` Linearu, když větev vytvoříte z issue; `ENG-123` kdekoli v názvu se čte zpět |
| YouTrack | `ABC-123`, jen když je `ABC` krátký název propojeného projektu (nebo projektu ze seznamu projektů profilu) | `ABC-123` kdekoli v názvu |
| Redmine | `#123`, jen když je Redmine propojeným zdrojem ISSUES (zdroj pull requestů si ponechává vlastní značku) | `123-fix-login` |
| Trello | adresa `https://trello.com/c/<shortLink>`; `#12` jen když má číslo 12 karta v mezipaměti | `12-fix-login` |

| PULL REQUESTY propojeny s | Ve zprávě |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Pokud `#N` odpovídají oba seznamy, vyhraje issue. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**V konverzacích.** Text issue nebo žádosti se čte s gramatikou jejího hostitele, která je širší než tabulka výše: `owner/repo#7`, `!7`, `GH-7`, `@login`, id commitů a další tvary uvedené v [Křížových odkazech](conversations.md#křížové-odkazy). Nejdřív se porovnají propojené zdroje repozitáře, takže klíč Jiry v komentáři z GitHubu pořád otevře svou issue v aplikaci. `gitpoulpi hosting markdown --body TEXT --id I` ukáže, co aplikace v textu najde.

### Vložení odkazu na issue

V composeru commitů tlačítko `#▾` na pravém konci Souhrnu otevře filtrovatelný
seznam propojených issues. Issue pojmenované aktuální větví je první a je
označeno *(z větve)*. `Enter` nebo kliknutí vloží jeho odkaz (`ABC-123`, `#12`,
`AB#12`) na pozici kurzoru. Klávesnice: `Alt+R` v composeru. Paleta: *Commit:
Vložit odkaz na issue...*. Bez propojeného seznamu issues je tlačítko zašedlé a
uvádí *nejdřív propojte seznam issues v postranním panelu*. GitPoulpi nikdy
nepřidá odkaz sám.
