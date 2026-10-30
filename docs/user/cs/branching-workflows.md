[English](../branching-workflows.md) · **Čeština** · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Workflow větvení

Workflow větvení říká GitPoulpi, jak váš tým používá větve: které žijí
věčně (`main`, `develop`, `production`), které jsou krátkodobé tématické větve
(`feature/...`, `hotfix/...`), jak se jmenují a kde každá začíná a
končí. Je to Git Flow, který si můžete přizpůsobit: pevné názvy, předpony a přípony, nebo
vzory se zástupnými symboly, takže větve, které bot vytvoří z issue, se
poznají stejně jako vaše vlastní.

Tato stránka popisuje model, soubor, předvolby, zjišťování a import,
příkazy jen pro čtení, akce, které větve zahajují, dokončují, synchronizují a povyšují
(CLI, MCP a aplikace) a obrazovky aplikace: editor nastavení, seskupený
postranní panel, nabídku `Flow` a její dialogy.

## Pojmy

- **Základní větev** je dlouhodobá. Základní větev může mít **rodiče** (`develop` do
  `main`); hlavní linie (trunk) žádného nemá. Základní větev může uvádět několik možných názvů
  (`main` nebo `master`, `develop` nebo `dev`): použije se ten, který existuje.
- **Typ tématické větve** je druh krátkodobé větve (feature, bugfix, release,
  hotfix, support...). Má **vzor názvu**, **rodiče** (odkud začíná
  a do čeho se dokončuje) a způsob dokončení.
- Každá větev je **zařazená**: základní (podle názvu), tématická nějakého typu (podle
  vzoru), nebo nezařazená.
- **Rodič větve** je, v tomto pořadí: ten zaznamenaný, když GitPoulpi větev vytvořilo
  (`git config branch.<name>.gitpoulpi-parent`), jinak `git-town-branch.<name>.parent`
  z Git Town, když existuje, jinak rodič jejího typu.

## Kde se ukládá

| Co | Kde | Commitnuté |
|---|---|---|
| Sdílený workflow | `.gitpoulpi/workflow.toml` v repozitáři | Ano, vámi: GitPoulpi ho zapíše, jen když o to požádáte |
| Vaše vlastní úpravy | `[workflow]` v `.gitpoulpi/repo-state.toml` | Ne (tento soubor se ignoruje) |

Stavová složka se ignoruje sama (`.gitpoulpi/.gitignore`), kromě
`workflow.toml`, který je určen k commitování. Účinný workflow je
sdílený soubor s vaším přepsáním navrch: základní větev se stejným názvem nebo typ se
stejným id se nahradí; nové se přidají; `disable_bases` a `disable_topics`
odeberou sdílené položky jen vám.

Uložení přes existující soubor zachová klíče, které tato verze nezná (z novějšího
GitPoulpi nebo od kolegy), ale ne komentáře ani pořadí klíčů.

## Soubor

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informativní)

[[base]]
name = "main"
names = ["master"]             # jiné zápisy téže větve
protected = true               # nikdy se nemaže, přímé commity jsou podezřelé

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # větev prostředí (GitLab Flow): povyšuje se po proudu

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # další názvy, které také odpovídají; nikdy se nepoužijí k vytvoření
# regex = "^feat-(?P<slug>.+)$"  # také odpovídá; pojmenované skupiny se stanou poli
parent = "develop"             # odkud začíná a výchozí cíl dokončení
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # příkazy shellu na výslovné přání, před spuštěním se vždy ukážou
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # toky řízené hostitelem, viz níže
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` a `hooks` řídí [akce](#akce) níže;
`issue_flow` řídí [toky řízené hostitelem](#toky-řízené-hostitelem).

### Vzory názvů

Text se `{zástupnými symboly}`:

| Zástupný symbol | Odpovídá | Vykreslí se z |
|---|---|---|
| `{slug}` | volný text; na konci může obsahovat `/` | textu malými písmeny, bez diakritiky, `-` mezi slovy, nejvýše 50 znaků |
| `{issue}` | číslice (`123`) | `123` nebo `#123` |
| `{issue_key}` | klíč trackeru (`PROJ-123`) | jak je zadán |
| `{user}` | jméno bez `/` | převedeno na slug |
| `{date}` | `2026-10-16` nebo `20261016` | jak je zadáno |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | jak je zadána |
| `{n}` | číslice | jak je zadáno |

Cokoli jiného je doslovné, takže předpony a přípony jsou prostě text:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Každý
zástupný symbol se smí objevit jednou a dva zástupné symboly potřebují mezi sebou text. Vzor
musí vytvářet platné názvy větví (pravidla `git check-ref-format`):
žádné mezery, žádné `~ ^ : ? * [ \`, žádné `..`, žádná část začínající `.` nebo končící
`.lock`.

`regex` slouží jen k rozpoznávání (je ukotven na obou koncích); použijte ho pro názvy, které
žádný vzor nevyjádří. Pojmenované skupiny se stanou poli větve.

### Který typ vyhraje

1. Větev pojmenovaná jako základní větev (kterýkoli z jejích názvů) je tato základní větev.
2. Jinak je kandidátem každý vzor každého typu, který odpovídá: `pattern`,
   potom `patterns`, potom `regex`, typ po typu v pořadí souboru. Vyhrává ten, který určuje
   nejvíc doslovných znaků (`release/{version}` přebije `{user}/{slug}`,
   a to přebije holé `{slug}`); regex se počítá podle své doslovné předpony. Při shodě vyhrává
   dřívější v souboru.
3. Jinak je větev nezařazená.

Porušený vzor nebo regex se ohlásí jako varování a přeskočí; zbytek
funguje dál.

## Šablony

| Šablona | Základní větve | Typy tématických větví |
|---|---|---|
| `git-flow` | `main` nebo `master`; `develop` nebo `dev` (rodič main) | `feature/` a `bugfix/` (z develop), `release/` (z develop, do main a develop, s tagem), `hotfix/` (z main, do main a develop, s tagem), `support/` (z main); odpovídají i množná čísla (`features/`, `releases/`...) |
| `github-flow` | `main` | `feature/` nebo libovolný název, z main, merge request do main, squash, poté smazána |
| `gitlab-flow` | `main`; `pre-production` a `production` (větve prostředí, řetěz main, pre-production, production) | `feature/` nebo libovolný název z main; `release/{version}` jako cíl cherry-picku |
| `trunk` | `main` | krátkodobé `{user}/{slug}` z main, s rebase |
| `custom` | žádné | žádné |

Větve prostředí odpovídají i jako `preprod`, `staging` a `prod`. Šablona je
výchozí bod: vypište ji pomocí `flow config init` a soubor upravte.

## Zjišťování a import

`gitpoulpi flow detect` seřadí šablony podle větví, které repozitář má,
se skóre ze 100 a s konkrétními nalezenými názvy (`master` místo `main`, `dev`
místo `develop`, `features/` místo `feature/`). Základní větve se počítají za 60 bodů, podíl
ostatních větví, které konkrétní vzory šablony rozpoznají, za 40 a dlouhodobá
větev jiné šablony (`develop` v kandidátovi GitHub Flow)
strhne 25.

`gitpoulpi flow import --from git-flow` čte klasické nastavení git-flow
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) z vlastního
`.git/config` repozitáře; `--from git-town` čte `git-town.main-branch` a
`git-town.perennial-branches` (rodič každé větve,
`git-town-branch.<name>.parent`, se převezme jako zaznamenaný rodič). Nic se
nepoužije potichu: příkaz vypíše TOML a `--write` vytvoří
`.gitpoulpi/workflow.toml`, jen když neexistuje.

## Příkazy

Všechny berou `--json`; server MCP je nabízí jako nástroje (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` čtou; akce jsou
popsány níže).

| Příkaz | Co dělá |
|---|---|
| `gitpoulpi flow status` | Každá lokální i vzdálená větev: druh (základní, tématická, nezařazená), typ a pole, rodič a odkud pochází (`recorded`, `git_town`, `type`, `base`) a u lokálních větví o kolik commitů je před svým rodičem (`ahead`) a za ním (`behind`). Také varování nalezená v definici, seřazené zjišťování a nastavení jiných nástrojů, které by šlo importovat. |
| `gitpoulpi flow templates` | Předvolby výše s jejich úplnou definicí. Repozitář nepotřebuje. |
| `gitpoulpi flow detect` | Seřazené šablony s nalezenými základními větvemi a typy. |
| `gitpoulpi flow config [--show-effective]` | Sdílený soubor (jeho cesta, zda existuje, rozparsovaný workflow, neznámé klíče), nebo s `--show-effective` workflow, podle kterého se vaše větve zařazují. `flow config show` je totéž. |
| `gitpoulpi flow config init --template <t> [--write]` | Vypíše TOML šablony; `--write` vytvoří `.gitpoulpi/workflow.toml` a odmítne přepsat existující. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Jak je popsáno výše. |

Počty ahead/behind se počítají nejvýše pro 300 lokálních větví na volání.

Soubor workflow, který nejde přečíst (špatný TOML, hodnota špatného typu), je chyba,
která soubor jmenuje; nikdy se potichu neignoruje.

## Akce

`start`, `finish`, `sync`, `promote` a `set-parent` jednají nad repozitářem. Každá
nejdřív **plánuje**: vypíše každý krok, který provede, s příkazem git,
na který se kroky převádějí, s varováními, tím, co nejde vrátit, a čímkoli, co ji
blokuje. Nic se nezmění, dokud nepotvrdíte. Každé spuštění je **jedna položka v historii
vrácení** (`gitpoulpi undo` vezme zpět větve, tag, smazanou
větev, zaznamenaného rodiče a checkout dohromady).

| Příkaz | Co dělá |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Pojmenuje větev podle vzoru typu (`flow start feature "Login form"` vytvoří `feature/login-form`; `flow start release version=1.2.0` vytvoří `release/1.2.0`; samotné slovo je `slug`; `user` a `date` mají výchozí hodnotu vy a dnešek), vytvoří ji z rodiče typu nebo z `--from` (větev, vzdálená větev nebo tag), přepne na ni a zaznamená jejího rodiče. `--publish` ji pushne s `-u` (`undo` to nevrací). |
| `gitpoulpi flow finish [<branch>]` | Pro každý cíl `finish` typu popořadě: přepne na cíl a sloučí větev strategií cíle (`merge`: vždy merge commit; `squash`: jeden commit s názvem z `--title` nebo z názvu větve; `rebase`: rebase větve na cíl a pak fast-forward; `fast-forward`), vytvoří tag, když ho cíl má (`tag = "v{version}"`, anotovaný), pak přepne na rodiče a smaže lokální větev, když to typ říká (`--delete` / `--keep` to přebijí; `--delete-remote` ji smaže i na remote, nelze vrátit). S `finish_mode = "merge-request"` (nebo `--mode merge-request`) jen pushne větev a otevře pull nebo merge request do rodiče, s `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` a `--integration` (nelze vrátit; lokálně se nic neslučuje ani nemaže). `--mode local-merge` přebije typ s `merge-request` a typ, který říká `ask`, potřebuje jedno z obou. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Přinese commity rodiče do větve: merge, nebo rebase, když je strategie větve `rebase` (nebo `--rebase`). S `--stack` i větve, jejichž rodičem je tato (zaznamenaná pomocí `set-parent`), rekurzivně, popořadě. Vrátí se na větev, na které jste byli. |
| `gitpoulpi flow promote <env-branch>` | Větve prostředí GitLab Flow: sloučí do ní větev, ze které je odvozena (`main` do `pre-production`, `pre-production` do `production`), jedním krokem, a vrátí se. Je-li to možné, udělá fast-forward. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Zapíše (nebo odebere) `branch.<name>.gitpoulpi-parent`. Odmítne cyklus. Lze vrátit. |

Větev zahájená odjinud se tam i dokončí: když se zaznamenaný rodič
(z `--from` nebo `set-parent`) liší od rodiče typu, prvním cílem dokončení
je zaznamenaný rodič.

### Potvrzování, plánování, skriptování

| Přepínač | Význam |
|---|---|
| `--dry-run` | Vypíše plán jako JSON (`{"plan": ...}`) a skončí. Kód ukončení 1, když něco brání. |
| `--yes` | Provede bez ptaní. |
| žádný | V terminálu: plán se ukáže a zeptá se vás. Jinak (skript, `--json`, agent) se plán vrátí jako `{"needs_confirmation": {"plan": ...}}` s kódem ukončení 3 a nic se nezmění. `start` bez `--publish` nebo hooků, `set-parent` a `--abort` potvrzení nepotřebují. |
| `--run-hooks` / `--no-hooks` | Viz [hooky](#hooky). |
| `--autostash` | Odloží lokální změny, které blokují checkout nebo merge, a poté je vrátí, ať říká `general.auto_stash` cokoli. |

Zablokovaný plán jmenuje důvod: základní větev tam, kde je potřeba tématická, název,
který už existuje, chybějící pole (`release/{version}` potřebuje `version=...`),
existující tag, cílová větev pozadu za svým remote (`--allow-behind`
přesto pokračuje), merge nebo rebase už probíhá, nenastavený workflow.

S nastavením **auto-stash** (`general.auto_stash`) se pracovní změny, které
blokují checkout nebo merge, odloží a kolem akce obnoví přesně jako
při běžném checkoutu; s `never` se akce zastaví dřív, než cokoli změní.

### Konflikty

Merge nebo rebase, který se zastaví na konfliktech, nechá běžný stav konfliktu (banner
konfliktu, `gitpoulpi conflicts ...`) a tok se zapamatuje. Kód ukončení
2, `status: "conflicted"`, cesty v `conflicts`. Vyřešte soubory a
označte je za vyřešené, a pak:

- `gitpoulpi flow finish --continue` commitne merge (nebo pokračuje v
  rebase, nebo commitne squash s jeho zprávou) a provede zbývající kroky;
- `gitpoulpi flow finish --abort` přeruší merge nebo rebase, vrátí se na
  větev, na které tok začal, a zapomene zbytek. Již dokončené kroky
  (první cíl dvojitého dokončení, s jeho tagem) zůstanou: `undo`
  je vezme zpět.

Tok zastavený selhávajícím hookem se obnoví stejně (`--continue` spustí tento
krok znovu). Zastavené spuštění a jeho pokračování jsou dvě položky undo.

### Merge requesty

Daemon větev pushne; otevření žádosti potřebuje váš token hostingu, takže
to dělá CLI stejným kódem jako `gitpoulpi hosting pr` (integrace,
která odpovídá remote, jinak předvyplněná webová stránka hostitele, vypsaná jako URL).
Výsledek nese `merge_request` (oč se žádalo) a `opened` (co hostitel
odpověděl), nebo `open_error`, když je větev pushnutá, ale žádost se nepodařilo
otevřít.

### Hooky

`[hooks]` v souboru workflow uvádí příkazy shellu (`after_start`,
`before_finish`). Pocházejí ze souboru, který tým sdílí, takže se **nikdy nespustí,
pokud o to nepožádáte**: `--run-hooks`, nebo ano na otázku, kterou terminál položí po
jejich výpisu. Plán je vždy uvádí, s `hooks_run: false`, když se
nespustí. Hook běží v shellu s repozitářem jako pracovním adresářem a
nastavenými `GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH`; jeho přesný příkaz, kód
ukončení a celý výstup jsou ve výsledku. Hook, který skončí nenulovým kódem, tok
tam zastaví (hook `before_finish` běží dřív, než se cokoli změnilo, takže je to běžná
chyba; po hooku `after_start` větev existuje a tok lze
pokračovat nebo přerušit). `--no-hooks` je přeskočí bez ptaní. Hook může dělat
cokoli, takže `undo` vrací jen změny v gitu.

### Pro agenty (MCP)

Nástroje jsou `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` a
`flow_set_parent` (finish, sync a promote jsou označeny jako destruktivní, takže je
hlídá `allow_mutations`). Jsou to plán, a teprve pak provedení: zavolejte s
`dry_run: true`, abyste dostali plán (běžný výsledek), a pak znovu s `yes: true`,
abyste ho provedli. Bez obojího se volání odmítne a nese plán, takže nic
neběží omylem. Hooky běží jen s výslovným `run_hooks: true`; plán
je uvádí (`hooks`, `hooks_run`), aby agent mohl uživateli nejdřív ukázat příkazy.
Po konfliktu `continue_: true` (přepínač je `--continue`) nebo
`abort: true`.

### Undo

| Akce | Co dělá `undo` |
|---|---|
| start | Smaže větev, zapomene jejího rodiče, znovu přepne na předchozí větev. |
| finish | Obnoví každou větev, kterou přesunul, tag, smazanou větev a checkout, jedním krokem. Pushnutá větev, smazání na remote a otevřený merge request zůstanou (plán to říká). |
| sync | Vrátí každou aktualizovanou větev, u té přepnuté i soubory. |
| promote | Vrátí větev prostředí. |
| set-parent | Obnoví předchozího zaznamenaného rodiče. |

## Toky řízené hostitelem

Týmy, kde práci zahajuje issue: issue se napíše ze šablony a
hostitel (GitLab, GitHub, Gitea, Forgejo) nebo bot vytvoří větev a merge
request. GitPoulpi vytvoří issue, počká na to, co hostitel vytvoří, a převezme to:
lokální větev sledující vzdálenou, se zaznamenaným rodičem (cíl merge
requestu), přepnutou. Potřebuje sekci ISSUES propojenou s trackerem
(`gitpoulpi hosting link issues`); propojení PULL REQUESTY říká, kde větve
a žádosti žijí (projekt ISSUES, když není propojeno).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # bot se často spouští štítkem
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # soubor repozitáře
branch_from_issue = "{type}/{issue}-{slug}"   # jak hostitel nebo bot větev pojmenuje
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # jak dlouho se `wait` dotazuje
merge_request = true                 # host-creates-branch: také koncept žádosti
assignees = []
```

`{type}` v `branch_from_issue` je id typu tématické větve; ostatní jsou obvyklé tokeny
[vzoru názvu](#vzory-názvů) (`{issue}`, `{slug}`, `{issue_key}`...). Větev patří k issue 12,
když odpovídá tomuto vzoru s `{issue}` rovným 12
(`feature/12-login` ano, `feature/120-x` ne).

| Režim | Po vytvoření issue |
|---|---|
| `local` | Nic na hostiteli. S `--pick-up` `flow start` typu s číslem a názvem issue (lze vrátit jako každý start). |
| `host-creates-branch` | GitPoulpi vytvoří větev **na hostiteli** z rodiče typu (GitLab: API větví; GitHub: `createLinkedBranch`, takže issue ji ukáže pod Development; Gitea/Forgejo: API větví) a **koncept** žádosti, který říká `Closes #12`. Hostitel, který prázdnou žádost odmítne (GitHub: zatím žádné commity), nechá větev a řekne to varováním. |
| `bot` | Nic: bot (obsluha webhooku, úloha CI, aplikace GitHub) vytvoří větev a žádost. `--wait` se dotazuje, dokud není splněno `wait_for`. |

Na co se `wait` dívá, u každého poskytovatele: vlastní odkazy hostitele (GitLab
`issues/:iid/related_merge_requests` a `closed_by`; GitHub propojené větve a
pull requesty `connected` / `cross-referenced` z časové osy), větve, jejichž název
odpovídá `branch_from_issue`, a otevřené žádosti, jejichž zdrojová větev odpovídá. Dotazuje se
s rostoucí pauzou (1, 2, 3, 5, 8, pak každých 10 s), dokud není splněno `wait_for` nebo
neskončí `timeout_secs`, a hostitel, který je po odpovědi pomalý nebo krátce nedostupný,
se zkusí znovu. Zdroj, který hostitel nenabízí (Gitea nemá odkazy issues), se přeskočí; když
selžou všechny zdroje, odpovědí je první chyba.

| Příkaz | Co dělá |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Vytvoří issue ze šablony (`issue_title`, `issue_labels`, soubor popisu, místo něj `--body`) a pokračuje podle režimu. `--dry-run` vypíše, co by se vytvořilo (`{"plan": ...}`), a nic nevytvoří. `--pick-up` v režimu `bot` nejdřív počká. Kód ukončení 4: čekání vypršelo (issue existuje; `flow issue wait` čeká dál). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Dotazuje se, co hostitel pro issue vytvořil, s průběhem na stderr v terminálu; Ctrl+C zruší. `--once` se podívá jednou. Kód ukončení 4: zatím nic. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Stáhne, vytvoří lokální větev sledující `origin/<branch>`, zaznamená jejího rodiče a přepne na ni. `#12` vyhledá větev nebo žádost issue na hostiteli, `!5` žádost, cokoli jiného je název větve. Rodičem je cíl žádosti (`--parent` ho přebije), jinak rodič typu. Nepotřebuje `--yes`: je lokální a lze ji vrátit, jako `start`. `--dry-run` stále stahuje (jen posouvá sledující refy remote a plán potřebuje větev vidět). |

`pick-up` je jedna položka v historii vrácení: `gitpoulpi undo` smaže větev,
zapomene rodiče a znovu přepne na předchozí větev (fetch zůstane, jako
každý fetch). Plánuje se jako ostatní akce: `--dry-run` vypíše svůj plán (`create_branch`,
`set_upstream`, `checkout`, `record_parent`) a platí obvyklé blokátory (pracovní
změny se řídí nastavením auto-stash). Požadavek je
`{"action": "pick_up", "branch": ..., "parent": ...}`.

Issues vytvořená tokem se **sledují** (`watched_issues` v `.gitpoulpi/repo-state.toml`,
nikdy se necommitují): po každém fetchi aplikace porovná vzdálené větve s
`branch_from_issue` a řekne vám, když se objeví větev sledovaného issue. Převzetí
to vyřídí.

### Pro agenty (řízené hostitelem)

Nástroje jsou `flow_issue_new`, `flow_issue_wait` a `flow_pick_up` (zápis, síť;
hlídá je `allow_mutations`). `flow_issue_new` s `dry_run: true` vrátí
plán; `flow_issue_wait` blokuje až `timeout` sekund, takže agentovi dejte krátký
a zavolejte znovu (`once: true` odpoví hned). Oba vracejí JSON s `found`
(`branches`, `requests`) a `timed_out`.

### Toky řízené hostitelem v aplikaci

S workflow a propojeným trackerem má nabídka Flow řádky *Nové issue: <Typ>...* (jeden na
typ; paleta `Flow: Nové issue: <Typ>...` a `Flow: Nové issue...`) a *Převzít větev
vytvořenou hostitelem...* (paleta `Flow: Převzít větev hostitele...`).

- **Nové issue** se zeptá na typ (rozbalovací seznam) a název; ukáže, odkud
  popis pochází a kdo větev vytváří. *Převzít větev, až bude existovat*
  je ve výchozím stavu zapnuto. *Vytvořit issue* ho vytvoří a pak v režimu `bot` ukáže **kartu
  čekání**: spinner, na co čeká, uplynulý čas a čas, kdy to vzdá, a
  **Přestat čekat** (`Esc`). Přerušení issue neztratí: zůstane sledované. Když
  je větev tam, převezme se a stavový řádek to řekne.
- **Objevila se větev** je karta, která se otevře poté, co fetch přinesl větev
  sledovaného issue (když není otevřen jiný dialog): *Převzít* ji převezme, *Později* ji zavře
  (do restartu aplikace se už nenabídne).
- **Převzít** bere `#12`, `!5` nebo název větve a před převzetím ukáže, co našlo.

Všechno běží na pozadí; volání, které trvá déle než 150 ms, ukáže
spinner.

### Zkoušení na zkušebně

`scripts/fake-gitlab-bot.sh` je zástupný bot pro Docker GitLab
(`scripts/test-bench.sh`): sleduje projekt kvůli issues se štítkem a vytvoří
`<type>/<iid>-<slug>` (s jedním commitem) a koncept žádosti, který issue uzavře.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` totéž nabízí
offline (API ve stylu GitLabu nad skutečným bare repozitářem, s botem).

## V aplikaci

![Postranní panel s workflow Git Flow: main a develop označené jako základní větve, větve feature ve složce Feature, ostatní pod Ostatní](../images/workflow-sidebar.png)

![Zahájit Feature: typ, slug, výchozí bod a název větve, který se vytvoří](../images/workflow-start.png)

![Nastavení > Repozitář > Workflow: šablona, zjišťování a editor základních větví](../images/workflow-settings.png)

Všechno níže se objeví jen u repozitáře, který má workflow (soubor
`.gitpoulpi/workflow.toml` nebo vaše přepsání). Bez něj se nic nemění:
postranní panel je běžný strom a tlačítko `Flow` tu není.

### Nastavení > Workflow

*Nastavení > Nastavení repozitáře > Workflow* upravuje definici. Pracuje s
konceptem a nic neuloží, dokud nestisknete tlačítko Uložit.

- **Šablona** je rozbalovací seznam předvoleb; výběr jedné nahradí základní větve a
  typy tématických větví níže předvolbou (vaše hooky a tok issue zůstanou).
- **Zjistit z větví** přečte názvy větví tohoto repozitáře a ukáže
  nejbližší šablonu s jejím skóre, nalezené názvy základních větví (`master` místo `main`),
  kolik větví každý typ tématické větve rozpoznává, s příklady, a skóre
  každé šablony. **Použít zjištěný workflow** ho načte do konceptu.
- Když repozitář obsahuje konfiguraci jiného nástroje (`gitflow.*` z Git Flow,
  `git-town.*` z Git Town), nabídne se **Importovat** a načte importovanou
  definici do konceptu. Nic se nepoužije, dokud to nestisknete.
- **Základní větve**, **Typy tématických větví** a jejich **Pravidla dokončení** tvoří jeden
  [seznam](settings.md#seznamy) rozdělený do skupin na kartě **Pravidla**; karta **Obecné**
  obsahuje zbytek (šablona, zjišťování, hooky, tok issue). Vyberte položku a upravujete ji vpravo (název a
  další názvy, rodič, chráněná, strategie sloučení, tag při sloučení, větev
  prostředí; popisek, vzor názvu, další vzory, regulární výraz, rodič, režim
  dokončení, smazat po dokončení). **+ Přidat ▾** nabídne, co přidat (pravidlo
  dokončení patří k vybranému typu tématické větve); přidání, duplikování, odebrání a
  tažení fungují jako v každém seznamu v Nastavení a položky se přesouvají v rámci své skupiny.
  Řádek **vzor názvu** uvádí tokeny (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`); pod ním živý náhled ukazuje existující větve, kterým
  vzor odpovídá, a název, který by vytvořil, a vzor, který se nedá rozparsovat,
  řekne proč červeně, přímo pod polem (nic z napsaného se neztratí).
- **Hooky** jsou příkazy shellu, jeden na řádek (`Po zahájení`, `Před dokončením`),
  upravované ve víceřádkovém editoru (`Ctrl+S` uloží, `Esc` zruší). Spouštějí
  příkazy na vašem počítači: stránka to říká a dialogy je před
  spuštěním ukážou.
- **Tok issue** nese pole, která používají [toky řízené hostitelem](#toky-řízené-hostitelem)
  (režim, název a štítky issue, vzor větve, na co čekat), uložená v souboru
  workflow.
- **Problémy** uvádí, co je v definici špatně (rodič, který
  neexistuje, porušený vzor).
- Lišta připnutá dole na stránce ukazuje **● Neuloženo** (nebo **Beze změn**).
  **Uložit do sdíleného souboru** zapíše koncept do `.gitpoulpi/workflow.toml`
  (commitněte ho), zachová klíče, které tato verze nezná, a zahrne do něj vaše
  přepsání. **Jen pro mě** uchová jen rozdíl oproti sdílenému souboru
  ve stavu vašeho repozitáře (`.gitpoulpi/repo-state.toml`, necommituje se). **Zahodit
  změny** znovu načte uložené. Zavření Nastavení s neuloženým konceptem se nejdřív zeptá.

### Postranní panel

LOKÁLNÍ a REMOTE se seskupují podle workflow: nejdřív základní větve, každá s
odznakem `[základ]` (`[základ, chráněná]`, když je chráněná), pak jedna složka na typ tématické větve
pojmenovaná podle jeho popisku s počtem větví (`Feature (3)`), pak
větve, které žádný typ nepřevzal, pod `Ostatní`. Složky se sbalují jako každá složka; prosté
stromy se vrátí řádkem *Flow > Seskupit větve podle typu* (řádek nabídky Flow) nebo
příkazem palety `Flow: Přepnout seskupení větví`. Najetí na větev ukáže její typ,
jejího rodiče a o kolik je pozadu za rodičem (a před ním).

### Nabídka Flow a její dialogy

Tlačítko **Flow** na panelu nástrojů (`Ctrl+G Shift+W`, paleta `Flow: Otevřít nabídku`) uvádí
*Zahájit <Typ>...* pro každý typ tématické větve, *Dokončit aktuální větev...*,
*Synchronizovat s rodičem...*, *Povýšit...* (větve prostředí GitLab Flow), *Nastavit
rodiče...*, přepínač seskupení a *Nastavení workflow...*. Paleta má
tytéž příkazy (`Flow: Zahájit Feature...` se generuje pro každý typ).

- **Zahájit** je formulář postavený podle vzoru názvu typu: pole na každý token. `{issue}`
  a `{issue_key}` mají rozbalovací seznam issues propojeného trackeru, který také
  vyplní `{slug}` z názvu issue; `{version}` navrhne verzi po vašem
  nejvyšším tagu vydání; `{user}` je váš aktivní profil; `{date}` je dnešek. Řádek
  ukazuje název větve, který vzor vytvoří (nebo proč nejde: neplatný ref, název
  už zabraný). Začít z je rozbalovací seznam (nejdřív rodič typu) a *Publikovat*
  pushne novou větev.
- **Dokončit**, **Synchronizovat**, **Povýšit** a **Nastavit rodiče** se zeptají na to, co potřebují, a pak
  ukážou **náhled**, který daemon vypočítá (stejný plán, který vypíše `gitpoulpi flow ... --dry-run`):
  každý krok popořadě, varování (cíl pozadu za svým remote, větve navázané
  na tuto) a **blokátory** (cíl, který neexistuje, cíl pozadu za svým
  remote), které vypnou tlačítko Spustit a řeknou proč. Kroky, které nejdou vzít zpět
  (push, smazání vzdálené větve, merge request), jsou označeny `!` a obarveny, s řádkem
  „Vrácení neobnoví: ...“. Hooky, které by se spustily, jsou uvedeny pod
  varováním, s políčkem *Spustit hooky* (ve výchozím stavu vypnuto). Když je režim dokončení
  typu `ask`, Dokončit se nejdřív zeptá *Dokončit pomocí*: merge lokálně, nebo otevřít pull nebo
  merge request.
- Akce pak běží se značkou u každého kroku (po 150 ms spinner) a dialog
  ukazuje, jak který krok dopadl. Celá akce je **jedna položka Undo**: `Ctrl+Z` vezme
  zpět start, finish, sync nebo promote najednou (Historie vrácení uvádí
  `Flow finish feature/login`); pushe, smazání na remote a merge requesty zůstanou, jak
  náhled řekl.
- **Konflikt** akci zastaví s tlačítky **Pokračovat** a **Přerušit**. Dialog lze
  zavřít, konflikt vyřešit obvyklými nástroji pro konflikty a pak ho znovu otevřít
  příkazem palety `Flow: Pokračovat v zastavené akci nebo ji přerušit...` (také řádek
  nabídky Flow, dokud je akce zastavena); Pokračovat odmítne, dokud jsou soubory stále v konfliktu,
  a obě tlačítka ponechá. Přerušit se vrátí na větev, na které akce začala; již provedené
  kroky zůstanou (undo je vezme zpět).
- Dokončení přes **merge request** větev pushne a pak otevře obvyklý formulář *Vytvořit pull
  request* (*Vytvořit merge request* na GitLabu), předvyplněný: cílem je rodič větve, název se bere
  z větve (a issue). Tlačítka metadat (řešitelé, recenzenti, štítky, milník)
  fungují jako u každého pull requestu. Když na remote neodpoví žádná integrace, otevře se
  místo toho v prohlížeči URL porovnání (a ukáže se na stavovém řádku).
- Texty kroků a blokátorů jsou anglicky, jako v CLI; vlastní text dialogu je
  přeložený. Pracovní strom se změnami se řídí nastavením `auto-stash`.

Každá z těchto věcí je dosažitelná z klávesnice a z palety a každý
rozbalovací seznam je combobox (Enter ho otevře, psaní filtruje, Enter vybere).

Řádky řízené hostitelem (*Nové issue: <Typ>...*, *Převzít větev vytvořenou hostitelem...*) jsou
popsány [výše](#toky-řízené-hostitelem-v-aplikaci).
