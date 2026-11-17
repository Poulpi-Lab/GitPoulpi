[English](../hosting-integration.md) · **Čeština** · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Hostingové služby a issue trackery

GitPoulpi se připojuje k těmto produktům:

- **Hosting** (pull requesty, issues, klonování z vašeho účtu, klíče SSH):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea a Forgejo
  (včetně Codeberg). **Gerrit Code Review** je hosting jen pro kontrolu: jeho změny jsou
  pull requesty, issues nemá.
- **Issue trackery** (pouze issues): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud a Server) a Redmine.
- **Lokální poznámky v Markdownu** (pouze issues, bez tokenu, bez sítě): složka s poznámkami v Markdownu, například složka vaultu Obsidian, viz [Lokální issues a Obsidian](local-issues.md).

Každá integrace používá **osobní token**. Žádný účet GitPoulpi ani přihlášení
přes OAuth neexistuje. Integrace jsou volitelné a **zapínají se pro každý
repozitář zvlášť**: dokud repozitář nepropojíte, nic se nestahuje. Token se
ukládá do vašeho credential helperu, nikdy do `config.toml`. Běžné operace
`git` (fetch, pull, push) integraci nepotřebují: používají přihlašovací údaje,
které váš `git` už má.

**Kam dál.** Tato stránka je nastavení: tokeny, připojení, propojení
repozitáře, klíče SSH. Jakmile je účet připojen:

- sekce PULL REQUESTY a ISSUES v postranním panelu, tabule issues, vytvoření
  pull requestu nebo issue, čip na panelu nástrojů a odkazy ve zprávách
  commitů: **[Pull requesty a issues](pull-requests-and-issues.md)**;
- pohled jedné issue nebo žádosti v aplikaci s komentáři, psaním a odesíláním
  jako: **[Konverzace](conversations.md)**;
- čtení a kontrola žádosti s vlákny komentářů a čekající kontrolou:
  **[Kontrola pull requestu](review.md)**;
- pull requesty a issues celého workspace v jednom seznamu:
  **[Radar](radar.md)**.

## 1. Vytvoření tokenu u hostitele

Formulář připojení obsahuje odkaz *Vygenerovat token na <host>*. Otevře stránku
tokenů hostitele s již vybranými správnými scopes. Při ručním postupu:

| Hostitel | Kde | Scopes |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repozitáře, pull requesty, issues), `read:user` (kdo jste), `admin:public_key` (jen pro přidání nebo odebrání klíčů SSH) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, na `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Token vytvořte **pro danou organizaci**, ne pro „All accessible organizations“: globální tokeny přestanou 2026-12-01 fungovat. |
| Azure DevOps Server | Stejná stránka na vašem serveru (`https://<server>/<collection>/_usersSettings/tokens`) | Stejné dva scopes |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Pro klíče SSH navíc `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profile picture > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, na `<host>/user/settings/applications` | Čtení `user`; čtení a zápis `repository`, `issue` a `user` (poslední pro klíče SSH). Gitea scopes dodatečně neukazuje, takže GitPoulpi nemůže upozornit na chybějící: odmítnuté volání to řekne. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. Klíč jedná jako vy. |
| YouTrack (Cloud a Server) | Váš profil > Account Security > New token, na `<host>/users/me?tab=account-security` | Scope `YouTrack`. Trvalý token jedná jako vy. |
| Gerrit | Settings > HTTP Credentials > Generate new token (nové servery; starší: Generate new password), na `<host>/settings/#HTTPCredentials` | Žádné. Token je **HTTP heslo** a jedná jako vy; vedle něj se zadává uživatelské jméno. |
| Redmine | My account > API access key > Show, na `<host>/my/account`. Správce musí zapnout **Administration > Settings > API > Enable REST web service**. | Žádné. Klíč jedná jako vy. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (klasický druh, bez scopes) | Žádné. Token jedná jako vy. |
| Jira Data Center | Profile > Personal Access Tokens | Žádné. Token jedná jako vy. |
| Trello | Nejprve **API key** z `trello.com/apps/admin` (vytvořte Power-Up, poté vygenerujte API key). Potom **token** z `trello.com/1/authorize?...&key=<your key>`. | `read`, `write` |

Tokenu dejte název, například „GitPoulpi“, a datum vypršení. Token zkopírujte:
hostitel ho zobrazí jen jednou.

## 2. Připojení

1. Stiskněte `Ctrl+,` a zvolte **Integrace**.
2. Vlevo zvolte produkt. Produkty jsou ve dvou skupinách: **Hosting** a
   **Issue trackery**.
3. Stiskněte **Připojit**. Chcete-li přidat druhý účet téhož produktu, použijte
   **+ Přidat profil**.

   ![Stránka Integrace: produkty vlevo a tlačítko Připojit pro GitHub](../images/settings-integrations.png)

4. Vyplňte formulář a vložte token. Token se při psaní zobrazuje jako prostý
   text, proto se ujistěte, že se nikdo nedívá. Formulář uvádí scopes, které
   potřebuje.

   ![Formulář připojení: pole Osobní přístupový token, scopes repo, read:user a admin:public_key, odkaz na vygenerování tokenu na github.com a tlačítka Připojit a Zrušit](../images/settings-connect.png)

5. Stiskněte **Připojit**. GitPoulpi na pozadí ověří token u hostitele. Stránka
   poté zobrazí kartu s vaším jménem a loginem, zeleně *Připojeno* a sekci
   klíče SSH. Token, kterému chybí některý z požadovaných scopes, se přesto
   připojí a chybějící scopes jsou vypsány červeně.

Na kartě: **Upravit** znovu otevře formulář. **Odpojit** smaže token a ponechá
profil. **Odebrat** smaže obojí po potvrzení.

Klávesnice na stránce Integrace: `Tab` / `Shift+Tab` přepínají mezi seznamem
produktů, pásem profilů a formulářem. `a` přidá profil, `e` ho upraví, `d` nebo
`Delete` ho odebere, `o` otevře stránku tokenu, `Esc` se vrátí o jeden krok.

### Co který formulář požaduje

Odkaz *Vygenerovat token* funguje až po vyplnění hostitele nebo organizace.

- **GitHub, GitLab:** pouze token. **GitHub Enterprise** a
  **GitLab Self-Managed** požadují také **Doménu hostitele**
  (`gitlab.example.com`; https, pokud nezadáte `http://`). Název bez tečky
  (služba Dockeru nebo počítač v místní síti) potřebuje port nebo `http://`: `gitea:3000`,
  `http://nas`. Server hostovaný vlastními silami může
  běžet na prostém http, například místní kontejner na `http://localhost:8080`. Token odeslaný
  přes prosté http putuje nešifrovaně, proto používejte http jen v místní nebo
  důvěryhodné síti.
- **Azure DevOps:** **Organizaci**, tedy název v
  `dev.azure.com/<organization>`. Přijímá se i adresa
  `https://<org>.visualstudio.com`. Dvě organizace vyžadují dva profily.
  Repozitář používá profil organizace, kterou jmenuje jeho remote. **Azure
  DevOps Server** požaduje **URL kolekce**, například
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi pro Azure DevOps nepoužívá
  systémový credential helper: Git Credential Manager tam ukládá přihlašovací
  token, nikoli osobní přístupový token.
- **Bitbucket:** váš **E-mail účtu Atlassian** a API token. Je-li e-mail
  prázdný, token se odešle jako bearer token: použijte to pro token workspace
  nebo repozitáře. GitPoulpi vaše workspaces najde podle tokenu. **Bitbucket
  Data Center:** **URL serveru** a HTTP access token.
- **Gitea, Forgejo:** **Doména hostitele** a token. **Forgejo** začíná na
  `https://codeberg.org`: pro vlastní server ji přepište. Gitea nemá veřejnou
  výchozí hodnotu. Adresa `http://localhost:3000` funguje pro lokální server.
  Oba druhy sdílejí jednoho klienta (Forgejo je fork Gitea se stejným
  `/api/v1`); token se odesílá jako `Authorization: token <token>`. Systémový
  credential helper se pro ně nepoužívá.
- **Jira Cloud:** **URL webu** (`acme.atlassian.net`), váš **E-mail účtu
  Atlassian** a klasický API token. Tokeny Jiry se scopes nejsou podporovány.
  **Jira Data Center:** **URL serveru** a osobní přístupový token.
- **Trello:** **Klíč API** a **token**. Klíč API je veřejný, proto se ukládá do
  `config.toml` (jako `account`). Token je tajný a jde do credential helperu.
  Formulář má dva odkazy: *Získat klíč API v Trellu* a po zadání klíče
  *Vygenerovat token*. Klíč i token se odesílají v hlavičce, nikdy v URL.

- **Linear:** pouze **klíč API**: Linear má jednoho veřejného hostitele, takže
  se nezadává žádná adresa. Klíč se odesílá tak, jak je, v hlavičce `Authorization`
  (Linear u osobních klíčů `Bearer` nebere). Aby odkazy `ENG-12` ve zprávách commitů
  otevíraly správnou stránku, uložte URL profilu s vaším pracovním prostorem
  (`https://linear.app/acme`, z CLI: `--url`); samotné řádky issues vždy nesou přesnou adresu.
- **YouTrack:** **URL webu** (`acme.youtrack.cloud` pro Cloud, jinak váš vlastní
  server, s kontextovou cestou, pokud ji má, například `host/youtrack`) a trvalý
  token, odesílaný jako `Authorization: Bearer`. Stejný klient obsluhuje Cloud
  i Server (`<url>/api`).
- **Gerrit:** **URL serveru** (s kontextovou cestou, pokud ji má), vaše **Uživatelské jméno** a HTTP
  heslo jako token. Putují jako HTTP basic autentizace; GitPoulpi mluví s REST API `/a/`
  serveru a ignoruje řádek `)]}'`, kterým každá odpověď začíná. Uživatelské jméno není tajné a ukládá se
  do `config.toml` (jako `account`), heslo jde do credential helperu. Repozitář se propojuje s *projektem*
  Gerritu v sekci PULL REQUESTY (jeho změny se tam uvádějí); Gerrit nemá issues a změna vzniká
  odesláním (viz *Odeslání ke kontrole* v [Kontrola](review.md#gerrit)).
- **Redmine:** **URL webu** vašeho Redmine (s podcestou, pokud ji má,
  například `host/redmine`) a váš klíč API, odesílaný jako `X-Redmine-API-Key`. Když
  Redmine odpoví 403 nebo stránkou HTML, REST webová služba je vypnutá (požádejte
  správce, nebo ji zapněte v **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack a Redmine jsou pouze issue trackery: žádné pull requesty, žádné repozitáře
ke klonování a žádné klíče SSH. Bitbucket nemá issues (Atlassian Bitbucket
Issues zrušil). Sekci ISSUES repozitáře z Bitbucketu propojte místo toho s
projektem v Jiře.

## 3. Kde se token ukládá

Token se ukládá do systémového **credential helperu** (Git Credential Manager
ve Windows, libsecret nebo `pass` v Linuxu, Keychain v macOS). Používá soukromou
položku, kterou nelze zaměnit s přihlašovacími údaji skutečného hostitele. Nikdy
se nezapisuje do `config.toml` a nikdy se nevypisuje.

Pokud není nakonfigurován žádný credential helper nebo token odmítne, GitPoulpi
ponechá token **v paměti pouze pro tuto relaci** a upozorní vás. Po restartu se
připojte znovu.

GitPoulpi hledá token profilu v tomto pořadí:

1. Proměnná prostředí `GITPOULPI_TOKEN_<PROFILE ID>`. Použijte ji k předání
   tokenu bez ukládání, například v CI.
2. Token uložený pro profil (`token_ref`).
3. Pouze u profilů GitHub a GitLab: systémový credential helper pro hostitele,
   pokud stejného hostitele nepoužívá jiný povolený profil.

Každý profil je položka `[[integrations]]` v `config.toml`. Viz
[referenční příručka konfigurace](config-reference.md#integrations-hosting-a-issue-trackery).

## Kontrola zabezpečení

Uložený token je stejně bezpečný jako adresa, na kterou se posílá. GitPoulpi si proto pro každou integraci pamatuje, k čemu byla připojena, a zastaví se, když se to změní:

- **Adresa.** Při prvním použití integrace (nebo připojení) se její adresa zaznamená do `trust.toml` v soukromé datové složce jen pro uživatele (`<data>/private/`), ne do `config.toml`: úprava konfigurace nikdy nemůže rozšířit to, čemu se důvěřuje. Pokud adresa v konfiguraci už není ta zaznamenaná, nic se na ni neodešle a token se ani nečte. Různé zápisy jedné adresy (`HTTPS://Git.Example.com:443/`) jsou stejná adresa; hostitel s podobným názvem ne.
- **Účet.** Po připojení tokenu a pak zhruba jednou denně (a jednou za běh u tokenu, který pochází z `GITPOULPI_TOKEN_*` nebo z credential helperu) se GitPoulpi hostitele zeptá, komu token patří, a porovná **id** účtu, ne přihlašovací jméno. Token, který nyní patří někomu jinému, se zastaví.
- **Certifikát.** Servery, jejichž certifikát nevede k žádné známé autoritě (soukromá CA, server s vlastním podpisem), se přijímají jen pro přesný klíč, který jste přijali, a TLS handshake stále dokazuje, že server tento klíč drží. Se zapnutým přísným připnutím se klíč musí shodovat, i když certifikát vede k veřejnému kořeni; cenou je, že každá výměna klíče se zeptá znovu, proto je to dobrovolné. Veřejní hostitelé (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) žádný záznam certifikátu nevedou.
- **Prosté http.** Adresa `http://` mimo loopback se odmítne, pokud jste ji pro danou integraci nepovolili.

Každý požadavek je také vázán na vlastní adresu integrace (a u veřejného hostitele na jeho hostitele API, například `api.github.com`): požadavek kamkoli jinam se odmítne dřív, než se odešle, a přesměrování na jinou adresu se nenásleduje.

**Integrace připojené před existencí této kontroly** dostanou svůj záznam tiše, z konfigurace, jaká byla, při prvním použití po upgradu: ponechají si dnešní úroveň důvěry a nic se neptá.

### Z příkazové řádky

Zastavený příkaz skončí s kódem ukončení 2 a vypíše, co se změnilo (viz [Reference CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` ukáže, čemu se důvěřuje; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` a `--allow-insecure-http on|off` přijmou vždy jednu věc, s uvedením přesné hodnoty. Po přijetí nové adresy zadejte token znovu. Odpojení (`hosting logout`) nebo odebrání integrace smaže její token, její záznam důvěry a to, co se z ní uložilo do mezipaměti (uložené seznamy repozitářů dialogu Klonovat, seznamy účtu v Radaru a návrhy vzdálených kontrol jejího hostitele).

Agent (MCP) nemůže přijmout nic: dozví se, že kontrola zabezpečení potřebuje vás.

### V terminálovém rozhraní

Zastavený požadavek otevře dialog **Kontrola zabezpečení integrace** nad tím, co jste dělali. Říká, co se změnilo (stará a nová adresa, starý a nový účet s jejich id, nebo starý a nový certifikát jako malá tabulka s otiskem SHA-256 ve skupinách po čtyřech, subjektem, vydavatelem a platností) a že token nebyl odeslán. Odpovědi jsou číslované:

1. **Odpojit** smaže uložený token, záznam důvěry a to, co se z integrace uložilo do mezipaměti.
2. **Důvěřovat** nové adrese, účtu nebo certifikátu (nešifrovaná adresa: **Povolit prosté http**).
3. **Zrušit** nic nemění.

Žádná odpověď není zvýrazněna, dokud nestisknete šipku nebo číslici, takže náhodné Enter nic nepřijme. Esc a zavírací křížek zruší; klik mimo dialog nedělá nic. Dlouhý text se posouvá kolečkem, Page Up/Page Down nebo posuvníkem. Několik kontrol se řadí do fronty (jedna na integraci a druh změny) a ptá se jedna po druhé. Zrušená kontrola se znovu neobjeví, dokud se nezmění; **Integrace: Zkontrolovat zabezpečení** v paletě příkazů ji přinese zpět.

Po důvěře k nové **adrese** se formulář integrace otevře s aktivním polem tokenu a nápovědou "Doporučujeme token zadat znovu.": uložený token se tam nikdy neposlal. Ve formuláři se záměrně změněná adresa přijme spolu s tokenem zadaným pro ni; bez tokenu formulář o něj požádá.

Formulář má také dva přepínače. **Přísné připnutí certifikátu** (pro adresu https) připne klíč certifikátu, který server předloží při připojení. **Povolit prosté http (nešifrované)** se nabízí jen pro adresu `http://`, která není tento počítač, s červeným varováním, že token putuje jako prostý text. Připojená karta ukazuje připnutý otisk.

### Co smí token a do kdy

Připojení se hostitele také zeptá, co token umí. GitHub (klasické tokeny), GitLab a Bitbucket (když pošle scopes) hlásí své scopes: karta ukáže žlutý řádek "Tento token má víc práv, než GitPoulpi potřebuje" se seznamem scopes nad rámec toho, co GitPoulpi používá, citlivé (například `delete_repo`, `admin:org`, `sudo`) červeně, s radou použít raději jemně nastavený token s datem vypršení. Klasický token GitHubu (`ghp_...`) dostane tuto radu i tehdy, když jsou jeho scopes přesné. Jemně nastavený token GitHubu se na oprávnění zeptat nelze, takže se ukáže jen jeho vypršení. Karta ukazuje datum vypršení, když ho hostitel uvede. Token bez vypršení neukáže žádnou připomínku a ostatní produkty žádné scopes nehlásí a nevyvolají varování. Stejná pole jsou v JSON příkazů `hosting integrations login` a `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Čtou se při připojení, takže je karta ukazuje pro relaci, která se připojila.

## 4. Propojení repozitáře

Propojení je výslovné a platí pro každý repozitář zvlášť. Je-li profil
připojen, postranní panel Refs zobrazí za TAGS sekci **PULL REQUESTY** a sekci
**ISSUES**. Každá sekce má dva rozbalovací seznamy: poskytovatele a projekt.

- Seznam poskytovatelů **navrhuje** účty odpovídající remote repozitáře
  (nejdřív `origin`) jako řádky typu `owner/name · origin · GitHub`. Jedním
  výběrem se nastaví poskytovatel i projekt.
- Nic se nepropojuje automaticky. Jira, Trello, Linear, YouTrack a Redmine se nikdy nenavrhují, protože
  žádný remote je nejmenuje: zvolte je sami.
- **Žádný (odpojit tento repozitář)** na začátku seznamu sekci odpojí. Nic ji
  samo znovu nepropojí.
- Propojení se ukládá do souboru `.gitpoulpi/repo-state.toml` repozitáře.

Obě sekce se propojují **odděleně**. Například pull requesty mohou pocházet z
Azure DevOps a issues z Jiry.

| Poskytovatel | Projekt PULL REQUESTY | Projekt ISSUES |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | stejný |
| Azure DevOps | `Project/Repo` (organizace pochází z profilu) | `Project` (work items) |
| Gitea, Forgejo | `owner/name` | stejný |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | žádný: propojte Jiru |
| Jira | žádný | klíč projektu, například `ABC` |
| Trello | žádný | board |
| Linear | žádný | klíč týmu, například `ENG` (týmy se vypisují) |
| YouTrack | žádný | krátký název projektu, například `ABC` |
| Gerrit | název projektu Gerrit, například `team/tool` | žádný |
| Redmine | žádný | identifikátor projektu, například `bench` (poslední část adresy projektu) |

Seznam projektů ISSUES lze prohledávat. Můžete také zadat projekt, který seznam
nezobrazuje.

**Další dotaz.** Zdroj issues má pod seznamem projektů volitelný řádek filtru.
S projektem se kombinuje (AND):

- Jira: fragment **JQL** (`component = UI AND labels = backend`).
- Azure DevOps: podmínka **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack: **dotaz YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Je zabalen do závorek, takže `or` nemůže rozšířit projekt. Vaše `sort by:`
  nahradí výchozí `sort by: updated desc`.
- Redmine: žádný (stačí filtry stavu a „přiřazené mně“ v sekci).
- Linear: žádný (stačí filtry stavu a „přiřazené mně“ v sekci).
- Trello: rozbalovací seznam, ve kterém zaškrtnete **seznamy**, které se mají
  zobrazit. Bez zaškrtnutí se zobrazí všechny otevřené seznamy.
- GitHub a GitLab ho ignorují.

`Enter` zadaný dotaz ponechá, `Esc` ho zahodí. Paleta:
*Issues: Upravit dotaz / seznamy...*.

Z CLI: `gitpoulpi hosting link pulls --id <profile>` (nebo `issues`),
`--clear` k odpojení, `--filter` pro další dotaz. Viz
[referenční příručka CLI](cli-reference.md#příkazy).

### Klonování z připojeného účtu

Připojený profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) je také zdrojem v dialogu **Klonovat** (karta Domů, tlačítko **Klonovat**, nebo `c` na ní). Každý profil je v seznamu **Klonovat z URL** jako samostatný zdroj (`GitHub · name`, `GitLab · name`); GitHub a GitLab se nikdy nemíchají. Vyberte jeden a načtou se jeho repozitáře seskupené podle vlastníka: nejdřív váš vlastní účet, pak jedna sekce pro každou organizaci nebo skupinu (`quarry/tools` pro podskupinu GitLabu), každá pod záhlavím s počtem. Zámek označuje soukromý repozitář.

![Dialog Klonovat u připojeného účtu GitHub: jedna sekce na vlastníka a zámek u soukromého repozitáře](../images/clone-hosted.png)

Vyhledávací pole filtruje napříč sekcemi (název, úplná cesta a popis); sekce bez shody zmizí. `Enter` na řádku vyplní klon HTTPS URL tohoto repozitáře a názvem složky.

![Dialog Klonovat s vyhledáváním zúženým na jednu organizaci](../images/clone-hosted-search.png)

Seznam profilu, který jste už otevřeli, se zobrazí okamžitě z uložené kopie,
zatímco se hostitel znovu dotazuje (*obnovování...* vedle popisku); když odpověď
dorazí, nové repozitáře se objeví a zmizelé zmizí, a zvýrazněný i zvolený
repozitář zůstanou na svém místě. Pokud obnovení selže, uložený seznam zůstane
s poznámkou a důvod je v Protokolu aktivity.

Pokud hostitel token odmítne, dialog místo seznamu zobrazí *nelze vypsat repozitáře*
s důvodem (a uložená kopie se smaže); ostatní zdroje fungují dál.

## 5. Klíče SSH

### Klíč SSH pro profil

Připojený profil má pod svou kartou sekci **Klíč SSH**. Bez klíče zvolte jednu
z možností:

- **Výchozí klíč**: váš ssh-agent nebo `~/.ssh`. Nic se nemění.
- **Vygenerovat nový klíč**: volitelný název (výchozí
  `GitPoulpi <host> <date>`). Je-li zaškrtnuto **Přidat veřejný klíč na
  <host>**, GitPoulpi ho nahraje, pokud ho hostitel ještě nemá. Klíč se zapíše
  bez hesla do `~/.ssh/gitpoulpi_<profile id>_ed25519` a `.pub`. `ssh-keygen`
  není potřeba. Existující soubor se nikdy nepřepíše.
- **Vlastní klíč**: zadejte cestu nebo **Procházet** a vyberte soubor
  privátního klíče, poté stiskněte **Použít tento klíč**.

Je-li klíč připojen, sekce ukazuje `title (SHA256:...)` s:

- **Odebrat**: odpojí klíč a smaže ho u hostitele, pokud ho přidal GitPoulpi.
- **Smazat**: totéž a navíc smaže soubory klíče, po potvrzení, které uvádí obě
  cesty.

### Klíč SSH pro jeden repozitář

Aby jeden repozitář používal konkrétní klíč, otevřete nabídku karty (`Ctrl+T` nebo
pravé tlačítko na kartě) a zvolte **SSH a přihlášení...**, nebo spusťte příkaz z palety
*Repozitář: SSH a přihlášení...*, a v části SSH vyberte **Soubor klíče** (nebo **Klíč účtu**
pro klíč profilu integrace). Volba se uchovává v soukromé složce GitPoulpi: nic se nezapisuje
do repozitáře ani do konfigurace gitu. Viz
[Nastavení](settings.md#nastavení-repozitáře-ssh-a-přihlášení).

Repozitář, který má stále vlastní `core.sshCommand` z dřívějšího připnutí „Klíč SSH pro tento
repozitář“, ho na této stránce zobrazí jen pro čtení s tlačítkem **Přesunout do GitPoulpi**.

CLI a MCP: `gitpoulpi repo-auth` (volba pro jeden repozitář) a pro samotné připnutí
`core.sshCommand` `gitpoulpi ssh-command show | set --key <file> | clear`.

Globální klíč v Nastavení > SSH a přihlášení (*Cesta k privátnímu klíči* nebo
*Vygenerovat pár klíčů*) platí pro každý repozitář bez vlastní volby. Soubor klíče
nebo připnutí repozitáře má přednost.

## 6. Ochrana větví pro přepis historie

Když přepisujete historii ([nástroj pro přepis](history-rewrite.md)), GitPoulpi se
hostitele zeptá, které větve chrání, aby se z přepisu ve výchozím stavu
vynechaly. Ptá se jen u repozitáře, který jste **propojili** (sekce pull
requestů nebo issues v postranním panelu, nebo připnutý profil), s tokenem toho
profilu, a token, který ochranu přečíst nemůže, dá jen „neznámé“: nic neselže.
Co který poskytovatel odpovídá a co token potřebuje:

| Hostitel | Co se čte | Co token potřebuje |
|---|---|---|
| GitHub (a Enterprise Server) | příznak `protected` větví a aktivní rulesets větví (vzory `~DEFAULT_BRANCH`, `~ALL` a `refs/heads/...`) | klasický token s `repo` (`public_repo` u veřejného) nebo jemně nastavený token s Metadata (čtení). Ruleset, který token přečíst nemůže, se oznámí, klasická pravidla se stále počítají |
| GitLab (gitlab.com i vlastní instance) | `protected_branches` (názvy a vzory s `*`) | `read_api` a roli, která smí vidět nastavení repozitáře |
| Azure DevOps (Services i Server) | zapnuté blokující politiky větví repozitáře (rozsahy `Exact`, `Prefix` a `DefaultBranch`) | Code (čtení). Oprávnění „force push“ je bezpečnostní nastavení, nečte se |
| Bitbucket Cloud | omezení větví, která zastaví push (`push`, `force`, `delete`) | `repository:admin`. Omezení podle modelu větvení se neřeší |
| Bitbucket Data Center | oprávnění větví, která zastaví force push (`fast-forward-only`, `pull-request-only`, `read-only`) | správce repozitáře |

Stránka nastavení ochrany (a výchozí větve a smazání repozitáře) se sestaví z URL
remote repozitáře pro GitHub, GitLab, Azure DevOps a Bitbucket, i když repozitář
propojený není, a zobrazí se se třemi cestami ven, když hostitel push odmítne.

## 7. Seznamy za celý účet v Radaru

Radar obvykle sleduje repozitáře, které jste naklonovali. Profil může také
vypsat, co se vás týká na celém hostiteli, naklonované i ne, třemi přepínači v
tabulce `[integrations.radar]` (všechny ve výchozím stavu vypnuté; viz
[referenční příručka konfigurace](config-reference.md#integrationsradar-seznamy-za-celý-účet)):

- **žádosti o kontrolu**: otevřené žádosti, které žádají o vaši kontrolu;
- **otevřené žádosti**: každá otevřená žádost vašeho vlastního jmenného prostoru a každé organizace nebo
  skupiny, jejímž jste členem, nebo jmenných prostorů, které uvedete. Nikdy ne celý hostitel;
  **Jmenné prostory** profilu (jeden na řádek v Nastavení > Integrace > Radar, volitelně
  jako regulární výrazy) omezují všechny tři seznamy: řádek odpovídá celé cestě repozitáře,
  když se jí rovná nebo je její předponou na hranici `/` (`ptm` ponechá `ptm/produits/x`, ne
  `ptmx/y`), a uplatňuje se na řádky, které hostitel vrátí, a také na straně serveru, kde to
  vyhledávání hostitele dovolí (kvalifikátory GitHubu `user:` / `repo:`);
- **přiřazené issues**: otevřené issues přiřazené vám.

Odpovídají GitHub, GitLab (včetně self-managed), Gitea, Forgejo, Azure DevOps (Services
i Server), Bitbucket Data Center a Gerrit. Jak se každý z nich ptá:

| Hostitel | Žádosti | Issues |
|---|---|---|
| Azure DevOps | aktivní pull requesty celé organizace (`searchCriteria.reviewerId` pro ty, které čekají na vás; žádost, o které jste už hlasovali, se vynechá); když server odmítne cestu na úrovni organizace, ptá se se postupně prvních 20 projektů. `namespaces` jsou názvy projektů | pracovní položky přiřazené vám, které nejsou Done, Closed ani Removed |
| Bitbucket Data Center | váš dashboard: role `REVIEWER`, která ještě neschválila, pro žádosti o kontrolu, `AUTHOR` a `PARTICIPANT` pro otevřené žádosti; `namespaces` jsou klíče projektů | žádné (Data Center nemá issue tracker; poznámka to říká) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; otevřené žádosti jsou `is:open projects:<prefix>` pro každý jmenný prostor (předpona cesty projektu), nebo vaše vlastní a kontrolované změny, když žádný není uveden; změna ve stavu work-in-progress je koncept | žádné |

Bitbucket Cloud nemá řádky za celý účet: nenabízí koncový bod, který by vypsal
žádosti napříč repozitáři, a dotazování každého repozitáře by bylo příliš mnoho
volání. Jeho naklonované repozitáře si ponechávají vlastní seznamy a `gitpoulpi hosting
radar --id <bitbucket>` to říká. Když má Gerrit víc změn, než kolik vrátí v
rámci limitu, seznam je označen jako oříznutý s poznámkou.

Koncepty jsou vynechány, pokud je neponecháte. Každý seznam je omezen: 300 řádků na druh, s
poznámkou, kolik jich bylo dalších, a na GitHubu několik volání vyhledávání za
kolo, protože jeho API vyhledávání povoluje jen 30 za minutu. Jmenný prostor, který token
nemůže číst, se stane poznámkou a ostatní se načtou dál; odmítnutý token nebo
rate limit selže jen u daného profilu. Odpovědi se znovu používají pět minut
(`radar-account.json` ve složce konfigurace).

V Nastavení > Integrace má připojený profil hostitele kódu blok **Radar**
pod svým klíčem SSH: jedno zaškrtávací políčko pro každý seznam (*Žádosti čekající na mou kontrolu*,
*Každá otevřená žádost*, *Zahrnout koncepty*, *Problémy přiřazené mně*, poslední
chybí u hostitelů bez issues), pole *Jmenné prostory* (jeden záznam na řádek: název shodující se s touto
skupinou nebo vlastníkem a vším pod ním, nebo regulární výraz na řádek se zaškrtnutým
*Regulární výrazy*; prázdné znamená váš vlastní a všechny, jejichž jste členem) a volba *Žádost z nenaklonovaného
repozitáře*: *Zeptat se*, *Naklonovat* nebo *Zkontrolovat přes API*.
Vše se ukládá, jakmile to změníte. Hostitelé bez dotazu na celý účet (Bitbucket
Cloud) místo toho ukazují jeden řádek, který to říká. Paleta příkazů blok najde
přes *Nastavení: Možnosti Radaru profilu...* a hledání v Nastavení najde každý
ovládací prvek podle jeho popisku.

Z příkazové řádky: `gitpoulpi hosting integrations radar <id> --review-requests on`
zapne seznam, `gitpoulpi hosting radar --json` seznamy vypíše (a
`--refresh` obejde mezipaměť). AI agenti mají stejné dva příkazy přes MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 8. Vystupování jako bot GitHub App

Projekt, který nesmí ukazovat jméno člověka, může vše na hostiteli dělat přes **GitHub App**: každý komentář, kontrola, uzavření nebo znovuotevření a změna štítků se pak objeví jako `<app-slug>[bot]`. Funguje to pro jeden repozitář a vaše ostatní repozitáře nechává na vašem vlastním tokenu.

1. Vytvořte GitHub App pro organizaci s oprávněními **Issues** a **Pull requests** nastavenými na *Read & write* (a *Contents: read*, aby se pull requesty daly vypisovat a kontrolovat) a nainstalujte ji do repozitáře.
2. Napište malý skript, který vypíše **instalační token** na svůj standardní výstup (soukromý klíč aplikace podepíše krátkodobý JWT, který se vymění za token platný jednu hodinu). GitPoulpi klíč nikdy nevidí.
3. V **Nastavení > Nastavení repozitáře > SSH a přihlášení** zvolte pod HTTPS **Příkaz tokenu**, jako **Příkaz** zadejte skript a pod **Hostitelé** hostitele (`github.com`) a zaškrtněte **Použít pro API hostitele (issues, pull requesty)**. Uložte a potvrďte příkaz (člověk ho vždy potvrzuje, s jeho SHA-256). Z příkazové řádky: `gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`.

Co se pro ten repozitář změní:

- Jeho volání issues a pull requestů (výpis, vytváření, komentování, kontrola, merge, uzavírání, štítky) používají token příkazu, se stejným limitem 30 sekund, cache v paměti a maskováním jako vlastní přihlášení gitu. Když hostitel odpoví 401, protože token vypršel, příkaz se zeptá znovu jednou. Pokud příkaz nelze spustit, volání se odmítne: váš osobní token není nikdy záložní řešení.
- Neexistuje „já“: instalační token se nemůže zeptat, kdo je. Záhlaví sekce označuje profil **(App bot)** a filtry postavené na „já“ (*Přiřazené mně*, *Moje*, *Čeká na mou kontrolu*) nemají co spárovat. Nic se neukazuje jako chyba.
- **Odesílat jako.** Každá zpráva, kterou GitPoulpi odešle, se ukazuje se jménem, jako které se odesílá (*Odesíláte jako my-app[bot] (bot GitHub App)* pod záhlavím [konverzace](conversations.md#odesílání-jako) a v jejím poli komentáře; `posted_as` v odpovědi CLI). S osobním tokenem je to váš login (`GET /user`). S tokenem App je to jméno **Odesílat jako**, například `my-app[bot]`: nastavte ho pro všechny repozitáře v **Nastavení > Integrace** (`general.hosting_post_as`, také `gitpoulpi config set general.hosting_post_as my-app[bot]`) nebo pro jeden repozitář v **Nastavení > Nastavení repozitáře > SSH a přihlášení > Hosting**, což má přednost. Když je prázdné, GitPoulpi se zeptá GitHubu (GraphQL `viewer { login }`) a jinak zjistí jméno z autora prvního komentáře, který odešle, a uchová ho s repozitářem. Podle toho jména se také poznají vaše vlastní komentáře, takže úprava a smazání se nabízejí jen u nich (komentáře App musí navíc pocházet z účtu bota); dokud není známo, lze měnit jen komentáře, které odeslala tato instalace.
- App nelze přiřadit. Nastavení spojte se zablokovaným přiřazováním níže.
- Zapnutí nebo vypnutí **Použít pro API hostitele** vyžaduje člověka, stejně jako nový příkaz.

Tímto způsobem používají token repozitáře jen profily GitHubu; ostatní hostitelé si ponechají vlastní token profilu. (Přístupový token projektu nebo skupiny GitLabu je také bot: je to token profilu, ukazuje se jako *bot projektu* a nic se nastavovat nemusí.)

### Zablokování přiřazování v repozitáři

**Nastavení > Nastavení repozitáře > SSH a přihlášení** má řádek **Přiřazování** (ve výchozím stavu **Povoleno**, nebo **Zablokováno**; uloží se hned). Zablokováno: v issues a pull requestech toho repozitáře je přiřazování nemožné, ať o ně žádá kdokoli:

- výběr řešitele je z formulářů nové issue a nového pull requestu vynechán (s poznámkou proč), *Hosting: Upravit řešitele...* v paletě je zašedlé s důvodem a nabídka řádku to řekne místo otevření;
- každý klient přiřazení odmítne dřív, než se cokoli odešle, včetně řešitelů předaných při vytváření issue nebo pull requestu: CLI odpoví `{"refused": {"reason": "assignment_blocked", ...}}` s návratovým kódem 2 a agent přes MCP dostane totéž odmítnutí;
- už existující řešitelé zůstávají vidět (jen pro čtení).

`gitpoulpi hosting assignment` ukáže nastavení, `--block` ho zapne a `--allow` ho zase vypne; znovu povolit smí jen člověk (server MCP nemůže).

## 9. Z příkazové řádky

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

Každý příkaz najdete v [referenční příručce CLI](cli-reference.md#příkazy).
Token se nikdy nepřijímá jako argument a server MCP `hosting login` nenabízí,
takže token nikdy neprochází agentem. `hosting login` vypíše `stored: "os"`, když
token drží systémové úložiště přihlašovacích údajů, nebo `stored: "session"` plus
`store_error`, když ho úložiště odmítlo a token žije jen v paměti pro tuto relaci.

## Řešení potíží

| Příznak | Řešení |
|---|---|
| *Připojit* selže s chybou ověření | Token je chybný, vypršel nebo byl odvolán. Vytvořte nový. |
| Scope je vypsán červeně | Token ho nemá. Připojení funguje, ale tato funkce ne. Přidejte scope u hostitele, poté **Upravit** a vložte nový token. |
| Postranní panel nemá sekci PULL REQUESTY | Není připojen žádný povolený profil. Připojte jeden a poté propojte repozitář v rozbalovacím seznamu sekce. |
| V seznamu je řádek *Znovu připojit* | Token se ztratil (restart bez credential helperu) nebo vypršel. Kliknutím na řádek otevřete profil. |
| Self-managed instance selhává | Zkontrolujte, že **Doména hostitele** neobsahuje cestu a že váš počítač důvěřuje certifikátu instance. |

## Známá omezení

- **Pull requesty z forků:** Azure DevOps a Bitbucket neumějí udělat checkout
  požadavku z forku (viz [Akce na řádku](pull-requests-and-issues.md#akce-na-řádku)). Azure DevOps neumí
  vytvořit pull request z forku.
- **Trello** zobrazí nejvýše prvních 1000 karet boardu. Zúžit je můžete
  filtrem seznamů. **Jira**, **Linear** a **YouTrack** vypíšou nejvýše 500
  issues na jedno obnovení. Zúžit je můžete fragmentem JQL nebo dotazem YouTrack.
- **Linear** bere jednoho řešitele na issue: když jich ve výběru zvolíte více,
  odešle se první. Jeho „milník“ je **cyklus** týmu (milníky projektů se
  nepoužívají). U **YouTrack** je to pole `Fix versions`; verze nemá číslo, takže
  výběr číslem označí nevydané podle jejich pozice v projektu. Ani jeden
  tracker nemá pull requesty.
- **Redmine** bere jednoho řešitele na issue (první z výběru), nemá štítky
  a vypíše nejvýše 500 issues na jedno obnovení. Řešitelé jsou členové projektu
  (lidé i skupiny) a rozlišují se podle zobrazovaného jména. Sloupce
  tabule Kanban jsou stavy issues a přesun karty stav mění; platí výchozí
  pravidla workflow Redmine (role, která takový přechod nesmí, dostane
  odmítnutí zpět a karta se vrátí). „Resolved“ ve výchozích datech Redmine není uzavřený stav:
  sloupec Zavřené jsou stavy „Closed“ a „Rejected“.
- **Bitbucket Data Center** nemá odkazy do prohlížeče pro nové pull requesty,
  větve ani commity. Stránky pull requestů se otevírají normálně.
- **Azure DevOps** nezobrazuje avatary autorů. Filtry „moje“ a „žádá o review“
  potřebují endpoint, který může osobní přístupové tokeny odmítnout. V takovém
  případě se profil připojí bez uživatelského jména a tyto dva filtry nejsou k
  dispozici. Neplatný token dostane místo chyby přihlašovací stránku; GitPoulpi
  to ohlásí jako odmítnutý token.
- **Gitea a Forgejo:** koncept je žádost, jejíž název začíná `WIP:`
  (server při vytvoření nemá příznak konceptu). Žádost uvádí jako recenzenty
  každého, kdo ji už zkontroloval, a stažení žádosti o kontrolu odebere jen
  čekající. Pull requesty z lokální větve forku se přepínají
  přes `refs/pull/<n>/head`, jako na GitHubu. Zmínky se hledají filtrem
  serveru `mentioned_by`. Server neukazuje scopes tokenu, takže chybějící
  se pozná, až když se volání odmítne.
- **Tabule issues** přesouvá karty u každého zdroje issues. V Jiře přesun provede přechod workflow, který ke sloupci vede;
  přechod, který žádá pole (například rozlišení), Jira odmítne a karta se vrátí i s její zprávou. V Azure DevOps se karta
  vrátí s důvodem, když typ pracovní položky cílový stav nemá.
- **Bitbucket:** app passwords už neexistují; použijte API token. Repozitáře se
  vypisují po workspacích.
- **Jira Cloud:** fungují jen klasické API tokeny (žádné tokeny se scopes, žádné
  OAuth). Nové issue v Jiře dostane popis jako prostý text.
- **Konverzace:** nejdřív se načte nejnovějších 300 záznamů (zbytek přes **Načíst starší**); komentář, úprava nebo smazání odeslané z GitPoulpi je akce navenek a nelze je z něj vrátit zpět; co který hostitel neumí, je v [tabulce](conversations.md#co-který-hostitel-neumí). Na Gerritu nelze zprávu ani komentář upravit či smazat a text Gerritu je prostý.
- Vytvoření pull requestu, work itemu, issue nebo karty probíhá na hostiteli a
  z GitPoulpi **nelze vrátit zpět**. Checkout požadavku lze vrátit zpět
  normálně.
