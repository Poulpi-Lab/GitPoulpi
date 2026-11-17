[English](../conversations.md) · **Čeština** · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Konverzace

Pohled konverzace issue, pracovní položky, karty nebo pull / merge requestu. Vyžaduje účet a propojený repozitář: viz [Hostingové služby a issue trackery](hosting-integration.md); seznamy, které ji otevírají, jsou v [Pull requestech a issues](pull-requests-and-issues.md); kontrola kódu je v [Kontrole pull requestu](review.md).

**Konverzace** je vše, co se řeklo o jedné issue, pracovní položce, kartě, změně nebo pull / merge requestu: popis, komentáře, kontroly, vlákna ke kódu a události (štítky, přiřazení, změny stavu, pushe). GitPoulpi ji přečte od hostitele, ukáže naformátovanou a nechá vás odpovědět, aniž byste opustili okno.

### Otevření

| Odkud | Jak |
|---|---|
| Řádek ISSUES | `Enter` nebo dvojklik; **Otevřít konverzaci** v nabídce řádku; paleta *Hosting: Otevřít konverzaci issue...* (seznam issues na obrazovce). **Otevřít v prohlížeči** v nabídce řádku ponechá prohlížeč. |
| Řádek PULL REQUESTY na Azure DevOps nebo Bitbucketu | `Enter` nebo **Otevřít konverzaci**. Na GitHubu, GitLabu, Gitea, Forgejo a Gerritu řádek otevře [kontrolu](review.md), jejíž stránka Přehled je touž konverzací (časová osa, klávesy a pole pro komentář pod ní). |
| Karta [tabule issues](pull-requests-and-issues.md#tabule-issues) | `Enter` nebo dvojklik. |
| [Radar](radar.md) | `i` na řádku issue nebo žádosti, **Otevřít konverzaci** v nabídce řádku, nebo *Radar: Otevřít konverzaci issue*. |
| Odkaz v jiné konverzaci | Klikněte na něj (nebo `l` a pak ho vyberte): `#7`, `!12`, `ABC-12` propojeného zdroje se otevřou v aplikaci a zařadí se do zásobníku Zpět. |
| Agent | `gitpoulpi ui show issue <n> --view conversation` (také `ui show pr`), viz [referenční příručka CLI](cli-reference.md). |

**Repozitáře, které nejsou naklonované.** Řádek Radaru z repozitáře, který nemáte naklonovaný, také otevře jeho konverzaci: pohled potřebuje jen profil integrace a projekt, takže ho Radar hostí nad svým oknem a `Esc` vrátí Radar. Co potřebuje klon, se vynechá: odkaz na commit se otevře na hostiteli místo v Logu a *Nechat AI číst repozitář* je zašedlé s důvodem (viz [kapitola o AI](ai.md#pomoc-s-psaním-na-hostiteli)). Žádost naklonovaného repozitáře má místo toho svou kontrolu (`e`) a **Otevřít konverzaci** v Radaru u ní řekne, že tu není co číst; issue naklonovaného repozitáře ji otevře na kartě toho repozitáře.

Issue u hostitele, který žádnou ke čtení nemá (Bitbucket a Gerrit issues nemají), otevře prohlížeč jako dřív a sestavení bez funkce `conversation` dělá totéž všude ([vlastní sestavení](custom-builds.md)).

### Pohled

Pohled přes celé okno: záhlaví (stav, kdo ji otevřel a kdy, štítky, počet komentářů, **Zobrazit události**, **Zpět**, **Obnovit**, **Otevřít v prohlížeči**), řádek **Odesíláte jako**, časová osa a pod ní pole komentáře.

- Časová osa se čte od nejstaršího po nejnovější a začíná nejnovějším oknem (nejvýš 300 záznamů). **Načíst starší** nahoře (`m`, paleta *Konverzace: Načíst starší záznamy*) přečte okno před ním, s počtem toho, co ještě zbývá, když ho hostitel řekne.
- Komentář ukazuje autora, datum, štítek `upraveno`, štítek `bot` u botů, `interní` u soukromých poznámek a tlačítka **Odpovědět**, **Citovat**, **Upravit**, **Smazat** a **Odkazy**, která vybraný záznam dovoluje. Vlákno ke kódu ukazuje `path:line`, `vyřešeno` nebo `nevyřešeno`, `zastaralé` a své komentáře. Kontrola ukazuje svůj verdikt.
- Události jsou jeden tlumený řádek každá (*carol přidal(a) štítek lfs*). **Zobrazit události** (`v`) je skryje; volba se uchovává pro každý repozitář. Hostitelé bez historie událostí (YouTrack, pracovní položky Azure DevOps, lokální poznámky) takové políčko nemají.
- Text je vykreslený Markdown: nadpisy, vnořené a úkolové seznamy, citace, upozornění ve stylu GitHubu, zvýrazněný kód, tabulky, odkazy, obrázky jako `[image: alt]`. Text, který přišel ve vlastním značkování hostitele, se nejdřív převede (viz [Převod značkování](#převod-značkování)). Blok kódu `mermaid` má tlačítko **Otevřít diagram**, které ukáže zdroj diagramu v posuvném prohlížeči (`Enter` ho zkopíruje); paleta má *Konverzace: Otevřít diagram záznamu*. Vyžaduje sestavení s `conversation` i `mermaid`.
- V přehledu kontroly je `cesta:řádek` vlákna nad kódem odkaz na kód (`G`, *Konverzace: Otevřít kód vlákna*; viz [Kontrola](review.md)); v konverzaci otevřené samostatně (issue, řádek Radaru) není stránka s kódem, na kterou by se dalo přejít.
- Pohled se znovu čte každých 60 sekund a na `F5` (**Obnovit**).

**Klávesy.** Každá má tlačítko, řádek nabídky nebo řádek palety (*Konverzace: ...*) a `Ctrl+N` označí tlačítka čísly.

| Klávesa | Akce |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Předchozí / další záznam |
| `PageUp`, `PageDown`, `Home`, `End`, kolečko, posuvník | Posun |
| `Tab`, `Shift+Tab` | Časová osa, pole komentáře, potom tlačítka záhlaví |
| `r` | Odpovědět: do vlákna vybraného záznamu, nebo s citací jeho textu, kde hostitel vlákna nemá |
| `Q` | Citovat vybraný záznam v novém komentáři |
| `c` | Nový komentář |
| `e`, `d` | Upravit, smazat jeden z vašich komentářů (před smazáním přijde otázka) |
| `l`, `Enter` | Odkazy záznamu: `Enter` následuje jediný, nebo otevře jejich nabídku |
| `y`, `o` | Zkopírovat adresu záznamu, otevřít ji v prohlížeči |
| `n` | Další nepřečtený záznam |
| `v`, `m`, `F5` | Zobrazit nebo skrýt události, Načíst starší, Obnovit |
| `Alt+Left`, `Backspace` | Zpět podle zásobníku Zpět |
| `q`, `Esc` | Zavřít (nejdřív se zeptá, když pole obsahuje text) |

### Psaní

**Nový komentář** (`c`, nebo `Tab` do pole), **Odpovědět** (`r`), **Citovat** (`Q`) a **Upravit** (`e`) používají jediný Markdown editor: **Psát**, **Náhled** a **Vedle sebe** (`Alt+P` přepíná Psát a Náhled, `Alt+L` zapíná Vedle sebe a vyžaduje okno široké 100 sloupců). `Ctrl+Enter` (také `Ctrl+S` a `Alt+Enter`) nebo **Odeslat** odešle, **Zrušit** pole zahodí a `Alt+E` otevře **Napsat pomocí AI**, kde je AI zapnutá ([kapitola o AI](ai.md#pomoc-s-psaním-na-hostiteli)). Náhled vykresluje stejný vykreslovač jako časovou osu, se stejnými odkazy. Když hostitel Markdown nepřijímá, zápatí jmenuje značkování, ve kterém se text odešle (*Odesláno jako Jira wiki markup*, *Odesláno jako Textile*).

- Komentář se přidá do časové osy hned a zaujme své místo, když hostitel odpoví. Pokud hostitel odmítne, záznam zmizí a pole se znovu otevře s vaším textem a důvodem.
- Text, který jste neodeslali, se uchovává pro každou položku (`remote-conversations.toml` ve složce konfigurace) a je tam, až to pole příště otevřete; odeslání nebo zahození ho odstraní. Zavření pole, které obsahuje text, se nejdřív zeptá.
- **Odpovědět** na vlákno (vlákno kontroly, diskuse GitLabu, komentář Linearu, vlákno Bitbucketu nebo Azure DevOps) odešle do tohoto vlákna. Kde hostitel má jen plochý seznam, **Odpovědět** začne nový komentář s vybraným záznamem v citaci (`> **autor**`), stejně jako **Citovat**.
- **Interní poznámka** (zaškrtávací pole, `Alt+I`): na GitLabu a Redmine je komentář viditelný jen členům projektu. Ukazuje se jen u hostitelů, kteří takové poznámky mají, a jen u nového komentáře.
- Odeslání je akce navenek: nic se nezapisuje do žurnálu a **z GitPoulpi nelze nic vrátit zpět**. Bez připojení pole zůstane, **Odeslat** je zašedlé s důvodem a záhlaví říká, že se ukazuje uložená kopie.

**Kdo smí upravovat nebo mazat.** **Upravit** a **Smazat** mají jen vaše vlastní komentáře. Nejdřív rozhoduje vlastní slovo hostitele, když ho dává (`viewerCanUpdate` a `viewerCanDelete` GitHubu, jinde `can_edit`), pak váš login proti autorovi komentáře, porovnaný bez ohledu na velikost písmen; komentáře bota App musí navíc pocházet z účtu bota. Dokud login bota GitHub App není znám, počítají se jen komentáře, které odeslala tato instalace (repozitář si uchovává jejich id): nic se neodhaduje podle jména. Ve vlákně **Upravit** a **Smazat** působí na poslední komentář vlákna, který je váš. Hostitel, který neumí upravovat ani mazat (viz [tabulka](#co-který-hostitel-neumí)), ukazuje tlačítka zašedlá s důvodem.

### Odesílání jako

Řádek pod záhlavím říká, jako kdo se vaše zprávy odesílají, a tentýž řádek je v každém poli:

| Řádek | Význam |
|---|---|
| *Odesíláte jako ann (osobní token)* | Token profilu: váš vlastní login (`GET /user`). |
| *Odesíláte jako ci-bot (příkaz tokenu repozitáře)* | Pro API hostitele se používá [příkaz tokenu](hosting-integration.md#8-vystupování-jako-bot-github-app) repozitáře. |
| *Odesíláte jako my-app[bot] (bot GitHub App)* | Instalační token. Jméno pochází z **Odesílat jako**, jinak od hostitele. |
| *Odesíláte jako bot GitHub App (jeho jméno zatím není známo: nastavte Odesílat jako v Nastavení)* | Žádné jméno nenastaveno, žádné zatím nezjištěno. |
| *Odesíláte jako ... (bot projektu)* | Přístupový token projektu nebo skupiny GitLabu. |
| *Píšete jako kai (lokální poznámka)* | Složka poznámek Markdown: `local.me` profilu, jinak připojený uživatel. |
| *Zatím není známo, jako kdo budete odesílat. Nastavte Odesílat jako v Nastavení.* | Nic se nepodařilo zjistit. |

**Odesílat jako** je jméno: u GitHub App je to `my-app[bot]`. Nastavte ho pro všechny repozitáře v **Nastavení > Integrace > Odesílat jako** (`general.hosting_post_as`), nebo pro jeden repozitář v **Nastavení > Nastavení repozitáře > SSH a přihlášení > Hosting**, což ho přepíše. Když je prázdné, GitPoulpi se zeptá GitHubu (GraphQL `viewer { login }`, na což instalační token může, ale nemusí odpovědět) a když to nejde, zjistí jméno z autora prvního komentáře, který odešle, a uchová ho s repozitářem. Osobní token ho nikdy nepotřebuje. Viz [Vystupování jako bot GitHub App](hosting-integration.md#8-vystupování-jako-bot-github-app).

### Křížové odkazy

Odkazy v konverzaci jsou podtržené a klikací. Nejdřív se porovnají propojené zdroje ISSUES a PULL REQUESTY repozitáře, takže `ABC-12` propojeného projektu Jira se otevře v aplikaci i uvnitř komentáře z GitHubu; to, co přidá vlastní gramatika hostitele, otevře svou stránku po potvrzení odkazu. Id commitu se otevře v Logu, když je repozitář naklonovaný a id má aspoň 7 znaků, jinak na hostiteli. Nic uvnitř úseku kódu, bloku v ohrazení, odkazu Markdown, e-mailové adresy nebo URL není odkazem a `C#1`, `UTF-8`, `SHA-256` a `ISO-8601` také ne.

| Hostitel | Rozpoznává |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, id commitů |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (sourozenecký projekt), `@user`, `~label` a `%milestone` jako štítky, id commitů |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, id commitů |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, zmínky `{uuid}`, id commitů |
| Gerrit | hodnoty `Change-Id` (`I` a 40 hexadecimálních číslic), id commitů |
| Redmine | `#7`, revize `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, jen pro projekt nebo tým propojeného zdroje |
| Trello | adresy karet (`trello.com/c/...`) |
| Lokální poznámky | `[[note]]`, `[[note\|alias]]` a relativní odkazy `.md` na jinou poznámku složky |

`gitpoulpi hosting markdown --body T --id I` vypíše, co z textu aplikace udělá, včetně odkazů.

### Převod značkování

Hostitelé, kteří Markdown neumějí, se převádějí oběma směry, takže všude čtete a píšete Markdown. Co převodník nerozumí, zůstane čitelným textem.

- **Jira** (Cloud i Data Center) ukládá komentáře jako wiki markup přes API v2. Co čtete, se převede na Markdown; co píšete, se převede na wiki markup a zápatí říká *Odesláno jako Jira wiki markup*.
- **Redmine** formátuje text jako Textile nebo Markdown, což API neříká: řekne to GitPoulpi řádek profilu **Formát textu serveru** (Nastavení > Integrace, `markup`, ve výchozím stavu Textile). Textile se pro čtení převádí na Markdown a pro psaní zpět na Textile.
- **Azure DevOps Server** vrací komentáře pracovních položek jako HTML a jako HTML je i přijímá: zobrazují se jako Markdown z něj převedený a zapisují se jako HTML. Azure DevOps Services a vlákna pull requestů přijímají Markdown.
- **Gerrit** je prostý text: nevykresluje se jako Markdown a co napíšete, se odešle tak, jak je.
- Každý importér nejdřív odstraní řídicí a escape sekvence, takže komentář nemůže pohnout terminálem.

### Uložená kopie, Načíst starší, nepřečtené

- Každá konverzace, kterou otevřete, se uloží (`<state dir>/conversations/<integration>/<project>/<kind>-<n>.json`, jen pro uživatele, bez tokenu, nejvýš 2 MB na soubor, uchovává se 50 nejnovějších souborů). Vykreslí se hned po otevření položky a na pozadí se obnoví. Když hostitele nelze zastihnout, uložená kopie zůstane na obrazovce pod textem *Offline: zobrazena kopie uložená před 3 min (důvod)* a Odeslat je zašedlé.
- **Nepřečtené:** co jste viděli naposledy, se pamatuje pro každou konverzaci (uchovává se posledních 200). Novější záznamy od někoho jiného dostanou akcentový proužek a štítek `nové` a `n` skočí na další. Konverzace, kterou otevřete poprvé, nemá nic nového.

### Co který hostitel neumí

Každé `ne` níže je skutečný limit: ovládací prvek je skrytý nebo zašedlý s důvodem a CLI odpoví chybou jmenující hostitele ještě před odesláním.

| Hostitel | Položky | Odpověď ve vlákně | Úprava a smazání vlastních | Vyřešení vlákna | Události | Interní poznámky | Zapisuje se jako |
|---|---|---|---|---|---|---|---|
| GitHub | issues, pull requesty | pull requesty | obojí | pull requesty | ano | ne | GitHub Markdown |
| GitLab | issues, merge requesty | ano (diskuse) | obojí | merge requesty | ano | ano | GitLab Markdown |
| Gitea | issues, pull requesty | pull requesty | obojí | pull requesty | ano | ne | Markdown |
| Forgejo | issues, pull requesty | pull requesty | obojí | ne (hostitel nemá koncový bod) | ano | ne | Markdown |
| Azure DevOps | pracovní položky, pull requesty | pull requesty | obojí | ne (stav se ukazuje, nemění) | jen pull requesty | ne | Markdown (HTML u pracovních položek Serveru) |
| Bitbucket (Cloud, Data Center) | jen pull requesty | ano | obojí | ne (ukazuje se, nemění) | ano | ne | Markdown |
| Gerrit | jen změny | ano | ne (nelze upravit ani smazat) | ano | ano | ne | prostý text |
| Jira | issues | ne (plochý seznam) | obojí | ne | ano (changelog) | ne (vyžaduje roli v projektu) | wiki markup |
| Linear | issues | ano (odpovědi na komentář) | obojí | ne | ano | ne | Markdown |
| YouTrack | issues | ne | obojí | ne | ne | ne | Markdown |
| Redmine | issues | ne | jen úprava (Redmine 5 a novější), nikdy smazání | ne | ano (žurnály) | ano (soukromé poznámky) | Markdown nebo Textile |
| Trello | karty | ne | obojí | ne | ano (přesuny) | ne | Markdown |
| Lokální poznámky | poznámky | ne | vlastní lokálního autora | ne | ne | ne | Markdown |

Také: žádný hostitel nedovolí z konverzace založit nové vlákno na řádku (udělejte to v kontrole) a reakce se ukazují jen na GitHubu. Text delší, než hostitel přijme, se odmítne (GitHub 65 536 znaků, GitLab 1 000 000, Azure DevOps 150 000, Jira 32 767, Trello 16 384): CLI to zkontroluje předem, v aplikaci se ukáže vlastní zpráva hostitele a váš text se uchová. Lokální poznámka drží své komentáře v závěrečné sekci `## Comments` samotného souboru poznámky, každý jako záznam `### autor - čas`, takže je vidí i Obsidian a shell.

Na příkazové řádce jsou tytéž konverzace `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit` a `comment-delete` a `hosting review conversation` (viz [referenční příručka CLI](cli-reference.md)); agenti mají nástroje MCP stejných jmen (viz [MCP a agenti](mcp-and-agents.md)).
