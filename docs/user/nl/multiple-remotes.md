[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · **Nederlands** · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Meerdere remotes voor één branch

Veel repositories staan op meer dan één host: een hoofdhost (`origin`), een mirror op een andere host, een back-up. Met GitPoulpi pusht een branch met één klik naar al die remotes, en pullt desgewenst van meer dan één, zonder te veranderen wat gewone git doet. Er is geen account voor nodig en alles staat in de eigen `.git/config` van de repository, dus gewone git en andere tools blijven werken.

## Push-doelen en pull-bronnen

- De **upstream** is het eigen begrip van git (`branch.<name>.remote` en `merge`). Het is de *primaire* remote van de branch. GitPoulpi verandert nooit hoe hij werkt.
- Een **push-doel** is een extra remote waarnaar de branch ook pusht. De knop Push stuurt de branch naar de upstream en naar elk push-doel.
- Een **pull-bron** is een extra remote waarvan een pull ook commits neemt. Dit staat uit tenzij je het aanzet (het vakje **Ook Pull**): twee remotes samenvoegen die je niet hebt gekozen, wil niemand als standaard.

Een doel kan een andere branchnaam op die remote gebruiken (`backup` krijgt `feat-x`).

## Het dialoogvenster Remotes

Klik met rechts op een branch en kies **Remotes van <branch>...** (ook in het uitklapmenu van Push, in het palet als `Git: Remotes van branch...`, en door te klikken op het label `⇉3` op de rij van de branch in de sectie Refs). Het toont elke remote van de repository:

| Kolom | Betekenis |
|---|---|
| Remote, Branch | De remote en de branchnaam daar. **Hernoemen op remote...** wijzigt die naam voor de geselecteerde rij. |
| Push | Vink aan om de remote een push-doel van deze branch te maken. |
| Ook Pull | Vink aan om een pull ook die remote te laten integreren. De upstream wordt altijd gepulld. |
| Status | Hoe de branch ertegenover staat: gelijk, voor, achter, nieuw, niet verwant. |

Een rij met "remote verdwenen" noemt een remote die de branch nog vermeldt maar die de repository niet meer heeft; **Verwijderen** vergeet hem. Hetzelfde dialoogvenster stelt de upstream in (de lijst **Upstream**) en **Upstream wissen** wist hem. **Toepassen** schrijft alles weg als één ongedaan te maken wijziging.

Een repository kan ook elke nieuwe branch dezelfde extra doelen geven: **Instellingen > Git & codering** (de pagina voor de repository zelf), "Extra push-doelen voor nieuwe branches" (een vakje per remote). Ze worden naar een branch gekopieerd zodra die voor het eerst een upstream krijgt; bestaande branches blijven ongewijzigd.

## Pushen

- **Knop Push** bij een branch met extra doelen: pusht naar al die doelen. Als elke remote de nieuwe commits gewoon aanneemt, loopt het meteen en toont de voortgangsweergave elke remote die in een vinkje verandert. Is er een beslissing nodig (een remote heeft commits die jij niet hebt), dan opent in plaats daarvan het plandialoogvenster.
- **Push ▾** (de pijl naast de knop; ook de knop zelf als **Push-knop** in de instellingen op **ask** staat):
  - `Push naar alle (origin, gitlab, backup)`
  - `Push alleen naar <remote>`, één per remote van de branch
  - `Push naar geselecteerde remotes...`: altijd aanwezig, voor elke branch, ook voor een met één enkele remote
  - `Remotes van <branch>...`
- Het rechtsklikmenu van de branch en het palet (`Git: Push naar alle doelen`, `Git: Pushen naar remote...`, `Git: Push naar geselecteerde remotes...`) bieden dezelfde acties met dezelfde namen.

**Push naar geselecteerde remotes...** opent een lijst met elke remote van de repository, aangevinkt met de upstream en de doelen van de branch. Het geldt alleen voor deze push; vink **Onthouden voor deze branch** aan om de keuze als push-doelen van de branch op te slaan.

### Het plan

Vóór een gegroepeerde push bekijkt GitPoulpi elke remote (een snelle query, en een download van die remote alleen als hij afwijkt), zodat een probleem op de derde remote zichtbaar is voordat de eerste push begint. Het dialoogvenster toont elke remote met de stand van de branch en wat er gaat gebeuren:

| Situatie | Standaardactie | Ook aangeboden |
|---|---|---|
| De remote heeft niets van de branch, of mist enkele commits | Push | Overslaan |
| De remote loopt voor (jij hebt niets nieuws voor hem) | Fast-forward vanaf hem | Overslaan |
| De remote wijkt af omdat je hebt gerebased of geamendeerd wat je pushte | Force push (met lease) | Merge, Rebase, Overslaan |
| De remote heeft commits van iemand anders | Merge van hem (Rebase als **Pull-knop** op rebase staat) | Rebase, Force push (met lease), Overslaan |
| De geschiedenis heeft niets met de jouwe gemeen | Overslaan | Force push (met lease) |
| De host weigerde de branch (beschermd) | Overslaan | geen: forceren zou ook worden geweigerd |

De actie van elke rij is een lijst die je kunt wijzigen (Links/Rechts loopt erdoorheen, Enter opent haar, `s` slaat de rij over). Een merge toont of hij een conflict zou geven en in welke bestanden. Een **force push** wordt altijd bevestigd, noemt de remote en hoeveel commits hij vervangt, gebruikt altijd een lease (nooit een kale force) en is niet ongedaan te maken: het resultaat vertelt welke commit is vervangen en de opdracht die haar terugzet. **Problemen overslaan** zet elke rij met een probleem op Overslaan. **Uitvoeren** (Ctrl+Enter) start.

De merges, fast-forwards en rebases lopen eerst, in de volgorde van de lijst, zodat je branch zijn eindstand één keer bereikt; daarna gaan de pushes één remote tegelijk uit, de upstream eerst. Er is geen atomair "alles of niets" over remotes heen: elke remote eindigt met zijn eigen resultaat en een mislukking bij één (netwerk, een hook) stopt de andere niet. **Mislukte opnieuw** (of `Git: Mislukte pushes opnieuw proberen`) probeert die opnieuw. De voortgangsweergave sluiten stopt de uitvoering nooit; `Git: Gegroepeerde Push tonen` brengt haar terug.

### Conflicten en Doorgaan

Stopt een merge of rebase van de uitvoering bij conflicten, dan sluit het dialoogvenster en nemen de gewone merge-banner en de conflicteditor het over. Een balk boven de Log meldt `Gegroepeerde push gepauzeerd (gitlab): los op en rond af, dan Doorgaan.` met de knoppen **Doorgaan** en **Groep afbreken**.

- Los de conflicten op en rond de merge of rebase af (in GitPoulpi of met gewone git).
- **Doorgaan** is grijs tot dat klaar is en wordt **nooit voor je ingedrukt**: niets gaat vanzelf verder. Druk erop (of voer `Git: Gegroepeerde Push voortzetten` uit) en de uitvoering pusht opnieuw naar elke remote vanaf je nieuwe tip. Na een rebase komt een remote die de oude commits nog heeft terug in het plandialoogvenster (hij heeft nu een force push nodig) in plaats van stilletjes geforceerd te worden.
- **Groep afbreken** (bevestigd; `Git: Gegroepeerde Push afbreken`) gooit de merge of rebase weg die de uitvoering startte en vergeet de uitvoering. Al gedane pushes blijven.

De balk overleeft een herstart van GitPoulpi en de CLI ziet dezelfde bewaarde uitvoering.

## Pullen

- De knop **Pull** pullt de upstream precies als voorheen, tenzij de branch pull-bronnen heeft: dan pullt hij de upstream en elke bron, op volgorde, elk in de modus van **Pull-knop** in de instellingen (merge, alleen fast-forward of rebase). Een bron die niet kan worden ge-fast-forwarded, wordt gemerged (of gerebased) zoals de upstream; bij alleen fast-forward vraagt een bron die meer nodig heeft eerst.
- **Pull ▾** voegt `Pull van alle bronnen (origin + gitlab)` toe als er bronnen zijn, en `Pull alleen van <remote>` voor elke remote van de branch: een eenmalige pull van de branch van die remote, wat de vakjes ook zeggen.
- Het palet heeft `Git: Pull van alle bronnen` en `Git: Pull van één remote...`.
- Een pull die bij conflicten stopt, laat dezelfde balk achter, gelabeld als gegroepeerde pull.

## Instellingen

Alle vier staan op **Instellingen > Git & codering**.

| Instelling | Standaard | Betekenis |
|---|---|---|
| **Push-knop** | direct | direct pusht meteen; ask opent eerst het menu Push. |
| **Pull-knop** | merge | De modus van de knop Pull (merge, alleen fast-forward of rebase), en van de pull-bronnen. |
| **Elke remote controleren vóór een push** | aan | Elke remote bekijken vóór een gegroepeerde push (de CLI volgt het; `--no-preflight` zet het uit voor één aanroep). |
| **Extra push-doelen voor nieuwe branches** (per repository) | geen | De remotes waarnaar een nieuwe branch ook pusht. |

## Vanaf de opdrachtregel

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

Heeft een remote een beslissing nodig, dan antwoordt de opdracht met het plan (exitcode 3) en verandert niets; beantwoord het met `--resolve gitlab=merge`. Zie [de CLI-referentie](cli-reference.md) voor elke vlag. De MCP-server heeft dezelfde tools.

## Met gewone git

De doelen zijn gewone configuratiesleutels, dus een script of een alias kan ze zonder GitPoulpi gebruiken:

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Een waarde is `<remote>` (dezelfde branchnaam) of `<remote>:<branch>`; dat regelt de regel `d=${t#*:}`. De gewone alias kijkt niet eerst naar de remotes, biedt geen merge aan en kan na een conflict niet doorgaan: daarvoor is het dialoogvenster er.

## Goed om te weten

- Twee remotes die naar hetzelfde adres wijzen, worden opgemerkt ("gitlab en mirror pushen naar dezelfde URL"); de tweede wordt standaard overgeslagen.
- Een host met een beschermde branch weigert een force push; de rij zegt het en biedt alleen Overslaan aan.
