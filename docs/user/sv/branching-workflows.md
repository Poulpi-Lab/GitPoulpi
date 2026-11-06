[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · **Svenska** · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Grenarbetsflöden

Ett grenarbetsflöde talar om för GitPoulpi hur ditt team använder grenar: vilka som
lever för alltid (`main`, `develop`, `production`), vilka som är kortlivade
ämnesgrenar (`feature/...`, `hotfix/...`), hur de namnges och var var och en
startar och slutar. Det är Git Flow gjort anpassningsbart: fasta namn, prefix och
suffix, eller mönster med platshållare, så att grenarna en bot skapar från ett
ärende känns igen precis som dina egna.

Den här sidan går igenom modellen, filen, mallarna, identifiering och import, de
skrivskyddade kommandona, åtgärderna som startar, slutför, synkar och promotar
grenar (CLI, MCP och appen) samt appens skärmar: inställningsredigeraren, den
grupperade sidopanelen, `Flow`-menyn och dess dialoger.

## Begrepp

- En **basgren** är långlivad. En bas kan ha en **förälder** (`develop` in i
  `main`); en stam har ingen. En bas kan lista flera kandidatnamn
  (`main` eller `master`, `develop` eller `dev`): den som finns används.
- En **grentyp** (ämnestyp) är en sorts kortlivad gren (feature, bugfix, release,
  hotfix, support...). Den har ett **namnmönster**, en **förälder** (var den
  startar och vad den slutförs in i) och ett sätt att slutföras.
- Varje gren **klassas**: en bas (efter namn), en ämnesgren av någon typ (efter
  mönster) eller oklassad.
- **En grens förälder** är, i ordning: den som registrerades när GitPoulpi skapade
  grenen (`git config branch.<name>.gitpoulpi-parent`), annars Git Towns
  `git-town-branch.<name>.parent` när den finns, annars föräldern för dess typ.

## Var det lagras

| Vad | Var | Committas |
|---|---|---|
| Det delade arbetsflödet | `.gitpoulpi/workflow.toml` i repositoryt | Ja, av dig: GitPoulpi skriver den bara när du ber om det |
| Dina egna justeringar | `[workflow]` i `.gitpoulpi/repo-state.toml` | Nej (den filen ignoreras) |

Tillståndskatalogen ignorerar sig själv (`.gitpoulpi/.gitignore`), utom
`workflow.toml`, som är avsedd att committas. Det gällande arbetsflödet är den
delade filen med din åsidosättning ovanpå: en bas med samma namn, eller en ämnestyp med
samma id, ersätts; nya läggs till; `disable_bases` och `disable_topics`
tar bort delade bara för dig.

Att spara över en befintlig fil behåller nycklarna som den här versionen inte känner
till (en nyare GitPoulpis, en kollegas), men inte kommentarer eller nyckelordning.

## Filen

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (upplysande)

[[base]]
name = "main"
names = ["master"]             # andra stavningar av samma gren
protected = true               # tas aldrig bort, direkta commits är misstänkta

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # en miljögren (GitLab Flow): promotas nedströms

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # fler namn som också matchar; används aldrig för att skapa
# regex = "^feat-(?P<slug>.+)$"  # matchar också; namngivna grupper blir fält
parent = "develop"             # var den startar, och standardmålet vid slutförande
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # frivilliga skalkommandon, visas alltid innan de körs
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # hostdrivna flöden, se nedan
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` och `hooks` styr [åtgärderna](#åtgärder) nedan;
`issue_flow` styr de [hostdrivna flödena](#hostdrivna-flöden).

### Namnmönster

Text med `{platshållare}`:

| Platshållare | Matchar | Skrivs ut från |
|---|---|---|
| `{slug}` | fri text; får innehålla `/` när den är sist | texten, med gemener, accenter borttagna, `-` mellan orden, högst 50 tecken |
| `{issue}` | siffror (`123`) | `123` eller `#123` |
| `{issue_key}` | en trackernyckel (`PROJ-123`) | som angiven |
| `{user}` | ett namn utan `/` | slugifierat |
| `{date}` | `2026-10-16` eller `20261016` | som angiven |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | som angiven |
| `{n}` | siffror | som angiven |

Allt annat är bokstavligt, så prefix och suffix är bara text:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Varje
platshållare får förekomma en gång, och två platshållare behöver text emellan. Ett
mönster måste ge giltiga grennamn (reglerna i `git check-ref-format`):
inga blanksteg, inga `~ ^ : ? * [ \`, ingen `..`, ingen komponent som börjar med `.` eller slutar
med `.lock`.

Ett `regex` är bara till för matchning (det är förankrat i båda ändar); använd det för namn
som inget mönster kan uttrycka. Namngivna grupper blir grenens fält.

### Vilken ämnesgren vinner

1. En gren som heter som en bas (något av dess namn) är den basen.
2. Annars är varje mönster i varje ämnestyp som matchar en kandidat: `pattern`,
   sedan `patterns`, sedan `regex`, typ för typ i filens ordning. Det som fastställer
   flest bokstavliga tecken vinner (`release/{version}` slår `{user}/{slug}`
   som slår en ensam `{slug}`); ett regex räknar sitt bokstavliga prefix. Vid lika
   går den tidigare i filen först.
3. Annars är grenen oklassad.

Ett trasigt mönster eller regex rapporteras som en varning och hoppas över; resten
fortsätter fungera.

## Mallar

| Mall | Baser | Ämnen |
|---|---|---|
| `git-flow` | `main` eller `master`; `develop` eller `dev` (förälder main) | `feature/` och `bugfix/` (från develop), `release/` (från develop, in i main och develop, taggad), `hotfix/` (från main, in i main och develop, taggad), `support/` (från main); pluralformerna (`features/`, `releases/`...) matchar också |
| `github-flow` | `main` | `feature/` eller vilket namn som helst, från main, merge request in i main, squash, borttagen efteråt |
| `gitlab-flow` | `main`; `pre-production` och `production` (miljögrenar, kedjan main, pre-production, production) | `feature/` eller vilket namn som helst från main; `release/{version}` som cherry-pick-mål |
| `trunk` | `main` | kortlivad `{user}/{slug}` från main, rebasead |
| `custom` | inga | inga |

Miljögrenar matchar också som `preprod`, `staging` och `prod`. En mall är en
startpunkt: skriv ut den med `flow config init` och redigera filen.

## Identifiering och import

`gitpoulpi flow detect` rankar mallarna för grenarna som repositoryt har,
med en poäng av 100 och de konkreta namnen som hittades (`master` för `main`, `dev`
för `develop`, `features/` för `feature/`). Baser räknas för 60 poäng, andelen
övriga grenar som mallens specifika mönster känner igen för 40, och en
långlivad gren från en annan mall (en `develop` i en GitHub Flow-kandidat)
drar av 25.

`gitpoulpi flow import --from git-flow` läser de klassiska git-flow-inställningarna
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) från
repositoryts egen `.git/config`; `--from git-town` läser `git-town.main-branch` och
`git-town.perennial-branches` (varje grens förälder,
`git-town-branch.<name>.parent`, plockas upp som en registrerad förälder). Inget
tillämpas i det tysta: kommandot skriver ut TOML, och `--write` skapar
`.gitpoulpi/workflow.toml` bara om den inte finns.

## Kommandon

Alla tar `--json`; MCP-servern exponerar dem som verktyg (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` läser; åtgärderna
beskrivs nedan).

| Kommando | Vad det gör |
|---|---|
| `gitpoulpi flow status` | Varje lokal och remote gren: slag (bas, ämne, oklassad), grentyp och fält, förälder och varifrån den kom (`recorded`, `git_town`, `type`, `base`), och för lokala grenar hur många commits den ligger före (`ahead`) och efter (`behind`) sin förälder. Även varningarna som hittats i definitionen, den rankade identifieringen och andra verktygs inställningar som kan importeras. |
| `gitpoulpi flow templates` | Mallarna ovan, med sin fullständiga definition. Kräver inget repository. |
| `gitpoulpi flow detect` | De rankade mallarna med baserna och ämnena som hittades. |
| `gitpoulpi flow config [--show-effective]` | Den delade filen (dess sökväg, om den finns, det tolkade arbetsflödet, okända nycklar), eller med `--show-effective` arbetsflödet som dina grenar klassas med. `flow config show` är detsamma. |
| `gitpoulpi flow config init --template <t> [--write]` | Skriv ut en mallens TOML; `--write` skapar `.gitpoulpi/workflow.toml` och vägrar skriva över en. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Som beskrivet ovan. |

Före/efter-räkningarna beräknas för högst 300 lokala grenar per anrop.

En arbetsflödesfil som inte kan läsas (felaktig TOML, ett värde av fel typ) är ett fel
som nämner filen; den ignoreras aldrig i det tysta.

## Åtgärder

`start`, `finish`, `sync`, `promote` och `set-parent` agerar på repositoryt. Var
och en **planerar** först: den listar varje git-steg den tänker ta, med git-kommandot
som varje motsvarar, varningarna, vad som inte kan ångras och allt som hindrar
den. Inget ändras förrän du bekräftar. Varje körning är **en post i
ångrahistoriken** (`gitpoulpi undo` tar tillbaka grenarna, taggen, den borttagna
grenen, den registrerade föräldern och checkouten tillsammans).

| Kommando | Vad det gör |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Namnger grenen efter typens mönster (`flow start feature "Login form"` ger `feature/login-form`; `flow start release version=1.2.0` ger `release/1.2.0`; ett ensamt ord är `slug`; `user` och `date` blir som standard du och i dag), skapar den från typens förälder eller `--from` (en gren, en remote-gren eller en tag), checkar ut den och registrerar dess förälder. `--publish` pushar den med `-u` (ångras inte av `undo`). |
| `gitpoulpi flow finish [<branch>]` | För varje `finish`-mål för typen, i ordning: checka ut målet och merga grenen med målets strategi (`merge`: alltid en merge-commit; `squash`: en commit med titeln `--title` eller från grennamnet; `rebase`: gör rebase av grenen på målet och gör sedan fast-forward; `fast-forward`), skapa taggen när målet har en (`tag = "v{version}"`, annoterad), checka sedan ut föräldern och ta bort den lokala grenen när typen säger det (`--delete` / `--keep` åsidosätter det; `--delete-remote` tar också bort den på remote, kan inte ångras). Med `finish_mode = "merge-request"` (eller `--mode merge-request`) pushar den bara grenen och öppnar pull eller merge request in i föräldern, med `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` och `--integration` (kan inte ångras; inget mergas eller tas bort lokalt). `--mode local-merge` åsidosätter en `merge-request`-typ, och en typ som säger `ask` kräver ett av de två. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | För in förälderns commits i grenen: en merge, eller en rebase när grenens strategi är `rebase` (eller `--rebase`). Med `--stack` också grenarna vars förälder är den här (registrerade med `set-parent`), rekursivt, i ordning. Går tillbaka till grenen du var på. |
| `gitpoulpi flow promote <env-branch>` | Miljögrenar i GitLab Flow: mergar grenen den ligger nedströms om (`main` in i `pre-production`, `pre-production` in i `production`) in i den, ett steg, och går tillbaka. Gör fast-forward när det går. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Skriver (eller tar bort) `branch.<name>.gitpoulpi-parent`. Vägrar en cykel. Ångringsbar. |

En gren som startats någon annanstans slutförs där: när den registrerade föräldern
(från `--from` eller `set-parent`) skiljer sig från typens är det första slutmålet
den registrerade föräldern.

### Bekräfta, planera, skripta

| Flagga | Betydelse |
|---|---|
| `--dry-run` | Skriv ut planen som JSON (`{"plan": ...}`) och stanna. Slutkod 1 när något hindrar den. |
| `--yes` | Kör utan att fråga. |
| ingen av dem | I en terminal: planen visas och du blir tillfrågad. Annars (ett skript, `--json`, en agent) kommer planen tillbaka som `{"needs_confirmation": {"plan": ...}}` med slutkod 3 och inget ändrat. `start` utan `--publish` eller hooks, `set-parent` och `--abort` kräver ingen bekräftelse. |
| `--run-hooks` / `--no-hooks` | Se [hooks](#hooks). |
| `--autostash` | Stasha lokala ändringar som blockerar en checkout eller merge och lägg tillbaka dem efteråt, oavsett vad `general.auto_stash` säger. |

En blockerad plan nämner orsaken: en basgren där en ämnesgren behövs, ett namn
som redan finns, ett saknat fält (`release/{version}` kräver `version=...`),
en tag som finns, en målgren som ligger efter sin remote (`--allow-behind`
fortsätter ändå), en merge eller rebase som redan pågår, ett arbetsflöde som inte är
konfigurerat.

Med inställningen **auto-stash** (`general.auto_stash`) stashas lokala ändringar som
blockerar en checkout eller merge och återställs runt åtgärden precis som vid en
vanlig checkout; med `never` stannar åtgärden innan något ändras.

### Konflikter

En merge eller rebase som stannar på konflikter lämnar det vanliga konfliktläget (konfliktbannern,
`gitpoulpi conflicts ...`) och flödet kommer ihåg. Slutkod 2,
`status: "conflicted"`, sökvägarna i `conflicts`. Lös filerna och
markera dem som lösta, och sedan:

- `gitpoulpi flow finish --continue` commitar mergen (eller fortsätter
  rebasen, eller commitar squashen med dess meddelande) och kör de återstående stegen;
- `gitpoulpi flow finish --abort` avbryter mergen eller rebasen, går tillbaka till
  grenen flödet startade på och glömmer resten. Steg som redan
  slutförts (det första målet vid ett dubbelt slutförande, med sin tag) ligger kvar: `undo`
  tar tillbaka dem.

Ett flöde som stoppats av en misslyckad hook återupptas på samma sätt (`--continue` kör det
steget igen). Den konfliktfyllda körningen och dess fortsättning är två ångerposter.

### Merge requests

Daemonen pushar grenen; att öppna begäran kräver din hosting-token, så
CLI gör det med samma kod som `gitpoulpi hosting pr` (en integration
som matchar remote, annars värdens förifyllda webbsida, utskriven som en URL).
Resultatet bär `merge_request` (vad som begärdes) och `opened` (vad värden
svarade), eller `open_error` när grenen är pushad men begäran inte kunde
öppnas.

### Hooks

`[hooks]` i arbetsflödesfilen listar skalkommandon (`after_start`,
`before_finish`). De kommer från en fil som ditt team delar, så de **körs aldrig
om du inte ber om det**: `--run-hooks`, eller ja på frågan en terminal ställer efter
att ha listat dem. Planen listar dem alltid, med `hooks_run: false` när de inte
kommer att köras. En hook körs i ett skal med repositoryt som arbetskatalog och
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` satta; dess exakta kommando, slutkod
och hela utdata finns i resultatet. En hook som avslutar med annat än noll stoppar
flödet där (en `before_finish`-hook körs innan något ändrats, så det är ett vanligt
fel; efter en `after_start`-hook finns grenen och flödet kan
fortsättas eller avbrytas). `--no-hooks` hoppar över dem utan att fråga. En hook kan göra
vad som helst, så `undo` tar bara tillbaka git-ändringarna.

**Förtroende.** Kommandona i en incheckad `[hooks]`-sektion kommer från den som kan
pusha till repositoryt, så de behöver också *ditt* förtroende, en gång per version av
listan. Förtroendet hålls per repository genom hooksens hash (en ändring av något
kommando, någon fas eller ordningen frågar igen) och kontrolleras av daemonen före det första
steget i en körning och innan ett fortsatt flöde går vidare, så en nekad körning har inte ändrat
något. Hooks i din egen åsidosättning (`[workflow]` i ditt repositorytillstånd, sidan
Inställningar) är dina och frågar aldrig.

- **CLI:** `gitpoulpi flow trust-hooks` skriver ut kommandona, deras `hash` och
  om de är `trusted`; `gitpoulpi flow trust-hooks --hash <hash>` litar på
  dem, och bara när filen på disk fortfarande har den hashen. `--run-hooks` på
  ej betrodda hooks skriver ut ett nekande (`{"refused": {"reason": "hooks_not_trusted",
  ...}}`, slutkod 2) med kommandona och den kommandoraden. En terminal som
  får frågan "Run them too?" får också frågan om den ska lita på dem.
- **TUI:** förhandsvisningen listar hooksen och säger att de inte är betrodda ännu. När du
  trycker på åtgärdsknappen frågar den sedan "Det här repositoryts arbetsflöde kör de här kommandona på
  din dator. Lita på dem?" med de exakta kommandona i en rullningsbar ruta
  (mushjul, dragbar rullningslist, PgUp/PgDn/Home/End) och två svar:
  **1 Lita på och kör** och **2 Kör utan hooks**. Ingenting är markerat förrän du
  trycker på en piltangent eller en siffra; Esc, Avbryt eller stängningskorset går tillbaka utan att
  svara. När filen har ändrats sedan du litade på den står det i titeln "Hooks har ändrats
  sedan du litade på dem" och den gamla listan visas under den nya.
- **Agenter:** det finns inget verktyg för att lita på hooks. `run_hooks: true` på ej betrodda
  hooks nekas med kommandona, och en människa kör `flow trust-hooks`.

### För agenter (MCP)

Verktygen är `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` och
`flow_set_parent` (finish, sync och promote är annoterade som destruktiva, så
`allow_mutations` styr dem). De är plan-och-kör: anropa med
`dry_run: true` för att få planen (ett vanligt resultat) och sedan igen med `yes: true`
för att köra den. Utan någon av dem vägras anropet och bär planen, så inget
körs av misstag. Hooks körs bara med ett uttryckligt `run_hooks: true`; planen
listar dem (`hooks`, `hooks_run`) så att agenten kan visa användaren kommandona
först. Incheckade hooks måste betros av en människa (`flow trust-hooks`, som saknar
verktyg): `run_hooks: true` på ej betrodda hooks returnerar `refused` med
kommandona och kör ingenting. Efter en konflikt, `continue_: true` (flaggan är
`--continue`) eller `abort: true`.

### Ångra

| Åtgärd | Vad `undo` gör |
|---|---|
| start | Tar bort grenen, glömmer dess förälder, checkar ut den förra grenen igen. |
| finish | Återställer varje gren den flyttat, taggen, den borttagna grenen och checkouten, i ett steg. En pushad gren, en borttagning på remote och en öppnad merge request ligger kvar (planen säger det). |
| sync | Lägger tillbaka varje uppdaterad gren, filerna inräknade för den utcheckade. |
| promote | Lägger tillbaka miljögrenen. |
| set-parent | Återställer den tidigare registrerade föräldern. |

## Hostdrivna flöden

Team där ett ärende startar arbetet: ärendet skrivs från en mall, och
hosten (GitLab, GitHub, Gitea, Forgejo) eller en bot skapar grenen och merge
requesten. GitPoulpi skapar ärendet, väntar på det hosten skapar och plockar upp det:
en lokal gren som spårar den på remote, dess förälder registrerad (begärans
mål), utcheckad. Det kräver att avsnittet ISSUES är länkat till en tracker
(`gitpoulpi hosting link issues`); länken för PULL REQUESTS säger var grenarna
och begärandena finns (ISSUES-projektet när den inte är länkad).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # en bot triggas ofta av en etikett
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # en fil i repositoryt
branch_from_issue = "{type}/{issue}-{slug}"   # vad hosten eller boten döper grenen till
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # hur länge `wait` frågar
merge_request = true                 # host-creates-branch: också en utkastbegäran
assignees = []
```

`{type}` i `branch_from_issue` är ämnestypens id; resten är de vanliga
[namnmönstrets](#namnmönster) token (`{issue}`, `{slug}`, `{issue_key}`...). En
gren hör till ärende 12 när den matchar det mönstret med `{issue}` lika med 12
(`feature/12-login` gör det, `feature/120-x` gör det inte).

| Läge | Efter att ärendet skapats |
|---|---|
| `local` | Inget på hosten. Med `--pick-up` en `flow start` av typen med ärendets nummer och titel (ångringsbar som vilken start som helst). |
| `host-creates-branch` | GitPoulpi skapar grenen **på hosten** från typens förälder (GitLab: grenarnas API; GitHub: `createLinkedBranch`, så att ärendet visar den under Development; Gitea/Forgejo: grenarnas API) och en **utkast**begäran som säger `Closes #12`. En host som vägrar en tom begäran (GitHub: inga commits ännu) lämnar grenen och säger det som en varning. |
| `bot` | Inget: en bot (en webhook-hanterare, ett CI-jobb, en GitHub App) skapar grenen och begäran. `--wait` frågar tills `wait_for` är uppfyllt. |

Vad `wait` tittar på, för varje leverantör: hostens egna länkar (GitLabs
`issues/:iid/related_merge_requests` och `closed_by`; GitHubs länkade grenar och
tidslinjens `connected` / `cross-referenced` pull requests), grenarna vars namn
matchar `branch_from_issue`, och de öppna begäranden vars källgren gör det. Den frågar
med växande paus (1, 2, 3, 5, 8, sedan var 10:e s) tills `wait_for` är uppfyllt eller
`timeout_secs` löper ut, och en host som är långsam eller tillfälligt onåbar efter ett
svar provas igen. En källa som hosten inte erbjuder (en Gitea har inga ärendelänkar) hoppas över; när
varje källa misslyckas är det första felet svaret.

| Kommando | Vad det gör |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Skapar ärendet från mallen (`issue_title`, `issue_labels`, beskrivningsfilen, `--body` i stället för den) och fortsätter som läget säger. `--dry-run` skriver ut vad som skulle skapas (`{"plan": ...}`) och skapar ingenting. `--pick-up` väntar först i `bot`-läge. Slutkod 4: väntan tog slut (ärendet finns; `flow issue wait` fortsätter vänta). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Frågar efter vad hosten skapat för ett ärende, med förlopp på stderr i en terminal; Ctrl+C avbryter. `--once` tittar en enda gång. Slutkod 4: inget ännu. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Hämtar, skapar den lokala grenen som spårar `origin/<branch>`, registrerar dess förälder och checkar ut den. `#12` slår upp ärendets gren eller begäran på hosten, `!5` en begäran, allt annat är ett grennamn. Föräldern är begärans mål (`--parent` åsidosätter det), annars typens förälder. Kräver ingen `--yes`: det är lokalt och ångringsbart, som `start`. En `--dry-run` hämtar ändå (den flyttar bara remote-spårande refs, och planen behöver se grenen). |

`pick-up` är en post i ångrahistoriken: `gitpoulpi undo` tar bort grenen,
glömmer föräldern och checkar ut den förra grenen igen (hämtningen ligger kvar, som
varje fetch). Det planeras som de andra åtgärderna: `--dry-run` skriver ut dess plan (`create_branch`,
`set_upstream`, `checkout`, `record_parent`) och de vanliga hindren gäller (lokala
ändringar följer inställningen för auto-stash). Begäran är
`{"action": "pick_up", "branch": ..., "parent": ...}`.

Ärenden som ett flöde skapat **bevakas** (`watched_issues` i `.gitpoulpi/repo-state.toml`,
committas aldrig): efter varje fetch matchar appen remote-grenarna mot
`branch_from_issue` och säger till när en dök upp för ett bevakat ärende. En
pick-up avslutar det.

### För agenter (hostdrivet)

Verktygen är `flow_issue_new`, `flow_issue_wait` och `flow_pick_up` (skriv, nätverk;
`allow_mutations` styr dem). `flow_issue_new` med `dry_run: true` returnerar
planen; `flow_issue_wait` blockerar upp till `timeout` sekunder, så ge en agent en kort
och anropa igen (`once: true` svarar direkt). Båda returnerar JSON med `found`
(`branches`, `requests`) och `timed_out`.

### Hostdrivna flöden i appen

Med ett arbetsflöde och en länkad tracker får Flow-menyn *Nytt <Typ>-ärende...* (ett per
typ; palett `Flow: Nytt <Typ>-ärende...`, och `Flow: Nytt ärende...`) och *Plocka upp en gren som
hosten skapat...* (palett `Flow: Plocka upp gren från hosten...`).

- **Nytt ärende** frågar efter typen (en rullgardin) och titeln; det visar var
  beskrivningen kommer från och vem som skapar grenen. *Plocka upp grenen när den finns*
  är på som standard. *Skapa ärende* skapar det och visar sedan, i `bot`-läge, **väntekortet**:
  en snurra, vad det väntar på, förfluten tid och tiden det ger upp, samt
  **Sluta vänta** (`Esc`). Att sluta vänta förlorar inte ärendet: det förblir bevakat. När
  grenen finns plockas den upp och statusraden säger det.
- **En gren dök upp** är ett kort som öppnas när en fetch hämtat grenen för ett
  bevakat ärende (när ingen annan dialog är öppen): *Plocka upp* tar över den, *Senare* stänger
  den (den erbjuds inte igen förrän appen startats om).
- **Plocka upp** tar `#12`, `!5` eller ett grennamn och visar vad den hittade innan den
  plockar upp den.

Allt körs i bakgrunden; ett anrop som varar mer än 150 ms visar en
snurra.

### Prova det på en testbänk

`scripts/fake-gitlab-bot.sh` är en stand-in-bot för Docker-GitLab
(`scripts/test-bench.sh`): den bevakar ett projekt efter ärenden med en etikett och skapar
`<type>/<iid>-<slug>` (med en commit) och en utkastbegäran som stänger ärendet.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` serverar samma
sak offline (ett GitLab-format-API över ett riktigt bare-repository, med boten).

## I appen

![Sidopanelen med ett Git Flow-arbetsflöde: main och develop markerade som basgrenar, feature-grenar i en Feature-mapp, resten under Övriga](../images/workflow-sidebar.png)

![Starta Feature: typen, slug, startpunkten och grennamnet den kommer att skapa](../images/workflow-start.png)

![Inställningar > Repository > Arbetsflöde: mallen, identifiering och basgrenseditorn](../images/workflow-settings.png)

Allt nedan visas bara för ett repository som har ett arbetsflöde (en
`.gitpoulpi/workflow.toml`, eller en åsidosättning av dig). Utan ett ändras inget:
sidopanelen är det vanliga trädet och det finns ingen `Flow`-knapp.

### Inställningar > Arbetsflöde

*Inställningar > Repo-specifika inställningar > Arbetsflöde* redigerar definitionen. Den arbetar på ett
utkast och sparar ingenting förrän du trycker på en Spara-knapp.

- **Mall** är en rullgardin med de färdiga mallarna; att välja en ersätter baserna och
  grentyperna nedan med mallen (dina hooks och ärendeflöde ligger kvar).
- **Identifiera från grenarna** läser det här repositoryts grennamn och visar den
  närmaste mallen med sin poäng, basnamnen den hittade (`master` för `main`),
  hur många grenar varje grentyp känner igen, med exempel, och poängen för
  varje mall. **Använd det identifierade arbetsflödet** läser in det i utkastet.
- När repositoryt innehåller ett annat verktygs konfiguration (Git Flows `gitflow.*`,
  Git Towns `git-town.*`) erbjuds **Importera det** och läser in den importerade
  definitionen i utkastet. Inget tillämpas förrän du trycker på den.
- **Basgrenar**, **Grentyper** och deras **Slutföranderegler** är en enda
  [lista](settings.md#listor) i grupper på fliken **Regler**; fliken **Allmänt**
  innehåller resten (mall, identifiering, hooks, ärendeflöde). Välj ett objekt för att redigera det till höger (namn och
  andra namn, förälder, skyddad, merge-strategi, tagg vid merge, miljögren; etikett, namnmönster, andra mönster, reguljärt uttryck, förälder,
  slutförandeläge, ta bort efter slutförande). **+ Lägg till ▾** väljer vad som läggs till (en
  slutföranderegel hör till den valda grentypen); lägga till, duplicera, ta bort och dra fungerar som i varje
  lista i Inställningar, och objekt flyttas inom sin grupp.
  Raden **Namnmönster** listar token (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`); under den visar en förhandsvisning live de befintliga grenar som
  mönstret matchar och namnet det skulle skapa, och ett mönster som inte går att tolka
  säger varför i rött, direkt under fältet (inget som skrivits går förlorat).
- **Hooks** är skalkommandon, ett per rad (`Efter start`, `Före slutförande`),
  redigerade i en flerradig redigerare (`Ctrl+S` sparar, `Esc` avbryter). De kör
  kommandon på din dator: sidan säger det, och dialogerna visar dem innan
  de körs.
- **Ärendeflöde** bär fälten som de [hostdrivna flödena](#hostdrivna-flöden) använder
  (läge, ärendets titel och etiketter, grenmönster, vad som ska väntas på), lagrade i
  arbetsflödesfilen.
- **Problem** listar vad definitionen gör fel (en förälder som inte
  finns, ett trasigt mönster).
- Ett fält som sitter fast längst ned på sidan visar **● Osparat** (eller **Inga ändringar**).
  **Spara i .gitpoulpi/workflow.toml (delad)** skriver utkastet till den delade
  filen (committa den), behåller nycklar den här versionen inte känner till, och viker in din egen
  åsidosättning i den. **Spara bara för mig** behåller bara skillnaden mot den delade filen
  i ditt repositorytillstånd (`.gitpoulpi/repo-state.toml`, committas inte). **Förkasta
  ändringarna** läser det sparade igen. Att stänga Inställningar med ett osparat utkast frågar först.

### Sidopanelen

LOCAL och REMOTE grupperas efter arbetsflödet: basgrenarna först, var och en med ett
`[bas]`-märke (`[bas, skyddad]` när den är skyddad), sedan en mapp per ämnestyp
uppkallad efter sin etikett med antalet grenar (`Feature (3)`), sedan
grenarna som ingen typ gör anspråk på under `Övriga`. Mappar fälls ihop som vilken mapp som helst; de vanliga
träden kommer tillbaka med *Flow > Visa grenar i vanliga listor* (raden i Flow-menyn) eller
palettkommandot `Flow: Växla grupperade grenar`. Att hålla över en gren visar dess typ,
dess förälder och hur långt efter (och före) föräldern den ligger.

### Flow-menyn och dess dialoger

Verktygsradens **Flow**-knapp (`Ctrl+G Shift+W`, palett `Flow: Öppna menyn`) listar
*Starta <Typ>...* för varje ämnestyp, *Slutför aktuell gren...*,
*Synka med föräldern...*, *Promota...* (miljögrenar i GitLab Flow), *Ange
förälder...*, växlaren för gruppering och *Arbetsflödesinställningar...*. Paletten har
samma kommandon (`Flow: Starta Feature...` genereras för varje typ).

- **Starta** är ett formulär byggt från typens namnmönster: ett fält per token. `{issue}`
  och `{issue_key}` följs av en rullgardin med den länkade trackerns ärenden, som också
  fyller `{slug}` från ärendets titel; `{version}` föreslår versionen efter din
  högsta release-tag; `{user}` är din aktiva profil; `{date}` är i dag. En rad
  visar grennamnet mönstret ger (eller varför det inte går: en ogiltig ref, ett namn som
  redan är taget). Starta från är en rullgardin (typens förälder först), och *Publicera grenen*
  pushar den nya grenen.
- **Slutför**, **Synka**, **Promota** och **Ange förälder** frågar efter det de behöver och
  visar sedan **förhandsvisningen** som daemonen tar fram (samma plan som `gitpoulpi flow ... --dry-run`
  skriver ut): varje steg i ordning, varningar (ett mål som ligger efter sin remote, grenar staplade
  på den här), och **hinder** (ett mål som inte finns, ett mål som ligger efter sin
  remote) som stänger av Kör-knappen och säger varför. Steg som inte kan tas tillbaka
  (en push, en borttagning av remote-gren, en merge request) markeras med `!` och färg, med
  en rad ”Ångra tar inte tillbaka:”. Hooks som skulle köras listas under en
  varning, med en kryssruta *Kör hooks* (av som standard). När typens slutförandeläge
  är `ask` frågar Slutför först *Slutför genom*: merga lokalt, eller öppna en pull eller
  merge request.
- Åtgärden körs sedan med ett märke per steg (en snurra efter 150 ms) och dialogen
  visar hur varje steg gick. Hela åtgärden är **en Ångra-post**: `Ctrl+Z` tar
  tillbaka starten, slutförandet, synken eller promotionen på en gång (Ångrahistoriken lyder
  `Flow finish feature/login`); pushar, borttagningar på remote och merge requests ligger kvar, som
  förhandsvisningen sade.
- En **konflikt** stoppar åtgärden med **Fortsätt** och **Avbryt**. Dialogen kan
  stängas för att lösa konflikten med de vanliga konfliktverktygen och sedan öppnas igen med
  palettkommandot `Flow: Fortsätt eller avbryt stoppad åtgärd...` (också en rad i Flow-
  menyn medan en åtgärd är stoppad); Fortsätt vägrar medan filer fortfarande har konflikter
  och behåller båda knapparna. Avbryt går tillbaka till grenen åtgärden startade på; steg
  som redan gjorts ligger kvar (undo tar tillbaka dem).
- Ett slutförande via **merge request** pushar grenen och öppnar sedan det vanliga *Skapa pull
  request*-formuläret (*Skapa merge request* på GitLab) förifyllt: målet är grenens förälder, titeln kommer från
  grenen (och ärendet). Metadataknapparna (tilldelade, granskare, etiketter, milstolpe)
  fungerar som i vilken pull request som helst. När ingen integration svarar för remote öppnas
  jämförelse-URL:en i webbläsaren i stället (och visas på statusraden).
- Steg- och hindertexterna är engelska, som CLI:ns; dialogens egen text är
  översatt. En arbetskatalog med ändringar följer inställningen `auto-stash`.

Alla dessa går att nå från tangentbordet och paletten, och varje
rullgardin är en kombinationsruta (Enter öppnar den, att skriva filtrerar, Enter väljer).

De hostdrivna raderna (*Nytt <Typ>-ärende...*, *Plocka upp en gren som hosten skapat...*) beskrivs
[ovan](#hostdrivna-flöden-i-appen).
