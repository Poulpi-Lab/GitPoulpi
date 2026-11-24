[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · **Nederlands** · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS ("Large File Storage") houdt grote bestanden (ontwerpbestanden, video's, datasets) buiten de geschiedenis van een repository: git bewaart in plaats daarvan een *pointer* van drie regels tekst, en de echte inhoud staat op de LFS-server en in een lokale cache onder `.git/lfs`.

GitPoulpi heeft voor dit alles geen account nodig. LFS gebruikt de inloggegevens die je remote al gebruikt (je git credential helper, of de ssh-agent).

## Wat het programma `git-lfs` nodig heeft, en wat niet

GitPoulpi voert het standaardprogramma `git-lfs` uit voor alles wat bestanden omzet of verplaatst (install, track, fetch, pull, push, locks).

LFS-*toestand* lezen heeft geen programma nodig: of een repository LFS gebruikt, welke bestanden pointers zijn, hoe groot ze zijn en of het echte object al lokaal is opgeslagen.

| | heeft `git-lfs` nodig |
|---|---|
| `gitpoulpi lfs status` (meldt dat het programma ontbreekt in plaats van te falen) | nee |
| `gitpoulpi lfs files` | nee |
| install / track / untrack / prune | ja |
| fetch / pull / push | ja |
| locks | ja |

## `git-lfs` installeren

- **Windows**: het wordt meegeleverd met Git for Windows; of `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Al het andere: https://git-lfs.com

Versie 3.0 of nieuwer is vereist. Na het installeren merkt GitPoulpi het binnen een minuut (of meteen bij `lfs status`).

## Als `git-lfs` ontbreekt

GitPoulpi crasht of stopt er nooit door:

- `gitpoulpi lfs status` slaagt en zegt `git-lfs: not installed`, met de installatielink.
- Een handeling die het programma nodig heeft faalt met een duidelijke melding die eindigt met de installatielink (`https://git-lfs.com`) en een afsluitstatus ongelijk aan nul; via MCP geeft de tool die melding terug.
- Een repository die LFS gebruikt opent nog steeds; haar grote bestanden verschijnen gewoon als pointers totdat het programma is geïnstalleerd.

## LFS-bestanden bekijken

```
gitpoulpi lfs status            # programma, patronen, filters, hook
gitpoulpi lfs files             # pointerbestanden in de index, met grootte en lokaal/niet gedownload
gitpoulpi lfs files --at v1.0   # ... zoals bij een revisie
gitpoulpi show big.bin --at HEAD  # meldt "Git LFS object, 12.3 MB, sha256:..." in plaats van pointertekst
```

## Een repository instellen: install, track, untrack

```
gitpoulpi lfs install                 # filters + hooks voor DEZE repository (git lfs install --local)
gitpoulpi lfs install --user          # in plaats daarvan in je globale git-configuratie (nooit stilzwijgend)
gitpoulpi lfs track '*.psd'           # *.psd via LFS laten lopen
gitpoulpi lfs track '*.psd' --lockable   # ... en ze alleen-lezen maken totdat ze vergrendeld zijn
gitpoulpi lfs track '*.bin' --renormalize  # ... en reeds gecommitte *.bin opnieuw als pointers stagen
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # oude, al gepushte objecten uit de lokale opslag verwijderen
```

- `install` schrijft standaard in de eigen `.git/config` en hooks van de repository, dus je globale git-configuratie wordt nooit aangeraakt tenzij je `--user` meegeeft.
- `track` en `untrack` bewerken `.gitattributes` en laten het **unstaged**: het verschijnt als een gewone wijziging in de werkmap die je zelf commit. Bestanden die *na* het tracken worden toegevoegd, worden bij hun volgende commit als pointers opgeslagen.
- Bestanden die **al zijn gecommit** als gewone inhoud blijven gewoon in de geschiedenis. `--renormalize` stage ze opnieuw zodat je volgende commit ze door pointers vervangt. Om ze ook in elke eerdere commit te verplaatsen, gebruik je de herschrijfregel *Bestanden naar LFS migreren* (zie [Geschiedenis herschrijven](history-rewrite.md#bestanden-naar-of-uit-git-lfs-verplaatsen)); die is van GitPoulpi zelf, niet `git lfs migrate`.
- Zet het patroon tussen aanhalingstekens zodat je shell het niet uitbreidt.
- Via MCP zijn dit gewone mutaties (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), onderworpen aan de `[mcp]`-afscherming in `config.toml`.

## Objecten verplaatsen: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # objecten naar de lokale opslag downloaden; werkmap onaangeroerd
gitpoulpi lfs pull  [--remote origin]           # fetch, daarna de pointerbestanden door de echte inhoud vervangen
gitpoulpi lfs push  [--remote origin] [--all]   # de objecten van de huidige branch uploaden (of van elke ref met --all)
```

- De remote is standaard de standaardremote van de repository, zoals bij `fetch` en `push`.
- De inloggegevens zijn de gebruikelijke van je remote (credential helper, ssh-agent). GitPoulpi vraagt nooit om een eigen account.
- `pull` installeert de LFS-filters voor deze repository als ze ontbreken: zonder die downloadt git-lfs de objecten maar slaat het vervangen van de pointers over, en een latere checkout zou pointers terugzetten.
- MCP behandelt `lfs push` zoals `push`: een destructieve tool (hij wijzigt de remote).
- Het venster toont de voortgang van een overdracht (bestanden en bytes) op de statusbalk. De CLI wacht op het resultaat.

### De inhoud uitchecken: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: echte inhoud over de pointerbestanden schrijven, uit de lokale opslag
```

Het downloadt nooit (dat is `lfs pull`); objecten die ontbreken in de lokale opslag blijven pointers. Net als `pull` installeert het eerst de LFS-filters van de repository als er geen zijn geconfigureerd. Zoals elke LFS-handeling is het niet ongedaan te maken; zie [Ongedaan maken en opnieuw](undo-redo.md). Via MCP is het `lfs_checkout` (een schrijfactie, niet destructief).

### De LFS-knop van de werkbalk, en pull / push

De werkbalk heeft na **Pop** een knop **`LFS ▾`** (verborgen als `git.lfs_enabled` uit staat, en in een repository waar LFS niet is geïnitialiseerd en ook niet door zijn `.gitattributes` wordt gebruikt: zie [Instellingen > LFS](settings.md#repositoryspecifiek-lfs)). Een klik opent een keuzelijst: **Fetch: alle LFS-bestanden** (`lfs fetch --all`), **Alle LFS-bestanden uitchecken** (`lfs checkout`), **Pull: alle LFS-bestanden** (`lfs pull`), **Push: alle LFS-bestanden** (`lfs push --all`, de objecten van elke ref) en **Lokale LFS-opslag opruimen** (`lfs prune`). Elke rij is ook een opdracht in het palet (`LFS: Objecten fetchen`, `LFS: Alle bestanden uitchecken`, `LFS: Objecten pullen`, `LFS: Push (alle objecten)`, `LFS: Lokale objecten opruimen`). `LFS: Menu openen` opent de keuzelijst. Elk heeft een `Ctrl+G`-toetsenreeks: `l` voor het menu, dan `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` en `Shift+V` voor de rijen. Je kunt ze wijzigen in *Instellingen > Toetsenbord & muis*. Als `git-lfs` ontbreekt, leidt elke rij naar het installatiedialoogvenster.

![De LFS-keuzelijst van de werkbalk: alle LFS-bestanden fetchen, uitchecken, pullen en pushen, en de lokale opslag opruimen](../images/lfs-menu.png)

**Pull en push regelen LFS zelf.** In een repository die LFS gebruikt, met `git-lfs` geïnstalleerd, controleert een gewone `pull` eerst of de LFS-filters van de repository zijn geïnstalleerd, en een gewone `push` controleert haar `pre-push`-hook. Als er een ontbreekt, voert GitPoulpi vóór de overdracht `git lfs install --local` uit (alleen deze repository) en zegt de statusbalk "LFS-filters geïnstalleerd". Zonder de hook zou een push geen LFS-objecten uploaden. Zet het uit met `git.lfs_autofix_filters = false` (*Instellingen > Git & codering > LFS-filters installeren vóór pull en push*). Een gewone `push` uploadt nog steeds alleen de objecten van de gepushte branch, nooit die van elke ref; prune is nooit impliciet.

## Bestandsvergrendelingen

Sommige bestanden kunnen niet worden gemerged (een Photoshop-bestand, een level, een binair asset). Git LFS-*locks* laten één persoon zeggen "ik bewerk dit": niemand anders kan het vergrendelen of er wijzigingen naartoe pushen totdat het is ontgrendeld.

```
gitpoulpi lfs locks  [--remote origin]              # wie welk bestand vasthoudt (die van jou zijn gemarkeerd)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Vergrendelen is een functie van de **LFS-server** (GitHub, GitLab en Gitea hebben het). Het wordt bereikt met de eigen inloggegevens van de remote; GitPoulpi heeft geen eigen account.
- Een server zonder vergrendeling laat `lock` falen met een melding die dat zegt (de getypeerde `LfsLockingUnsupported`). `locks` op zo'n server toont niets, wat git-lfs zelf ook meldt.
- Een patroon tracken met `--lockable` (`lfs track '*.psd' --lockable`) maakt die bestanden alleen-lezen totdat je ze vergrendelt.
- `unlock` van andermans lock wordt geweigerd; `--force` verbreekt hem, daarom behandelt MCP `lfs_unlock` als een destructieve tool. `lfs_locks` is alleen-lezen.

## Een LFS-repository klonen

`gitpoulpi clone <url>` (en het kloondialoogvenster van de TUI) checkt de repository uit *zonder* de LFS-filters uit te voeren, en downloadt dan, als de repository LFS gebruikt en `git-lfs` is geïnstalleerd, de objecten met `git lfs pull` (de filters voor die repository installerend als er geen zijn geconfigureerd) en toont dat als een eigen fase, *LFS-objecten downloaden*. `--no-lfs` slaat die stap over en laat pointers staan.

Een clone **faalt nooit vanwege LFS**: als `git-lfs` ontbreekt, of een download mislukt, slaagt de clone toch en blijven de grote bestanden pointers; de LFS-status van de repository zegt waarom, en `gitpoulpi lfs pull` maakt het werk af zodra het probleem is opgelost.

## In de terminal-UI

- **De tag `LFS`.** Een bestand dat met Git LFS is opgeslagen heeft een kleine tag `LFS` achter zijn naam in de lijsten Unstaged en Staged van het Statuspaneel en in de lijst met gewijzigde bestanden van een commit. **Alleen LFS** (`o` met het Statuspaneel of het Log gefocust, het selectievakje in de werkbalk van de bestandslijsten, of **Bestanden: Alleen LFS in-/uitschakelen** in het palet) beperkt die lijsten tot de LFS-bestanden. Het selectievakje staat aan het rechteruiteinde van de filterrij (Statuspaneel: in de werkbalk, of in het overloopmenu ervan als het smal is); in de bestandslijst van een commit krimpt het op een smal paneel tot `[ ] LFS` in plaats van te verdwijnen. Het verschijnt alleen als de getoonde bestanden er hebben. In het Statuspaneel en bij de gewijzigde bestanden van een commit telt een bestand als LFS als zijn pad een pointer in de index is (één gebundelde `git`-aanroep per verversing, niets voor een repository zonder `filter=lfs`-attributen). Met **Alle bestanden tonen** aan in het Log wordt de hele boom bij de geselecteerde revisie getoond en zijn de LFS-bestanden die van die revisie, gelezen uit haar eigen blobs (per revisie in de cache, buiten de render-thread, geen `git-lfs`-programma nodig), dus een bestand dat daar LFS is maar niet in de index, krijgt de tag en blijft behouden door het filter.
- **De diffkaart.** De diff van een LFS-bestand openen toont een kaart in plaats van pointertekst: `Git LFS-object`, `1.2 MB -> 1.4 MB`, de korte oude en nieuwe oids, en of het nieuwe object lokaal is opgeslagen of `niet gedownload`. Druk op `r` voor de ruwe pointerdiff, nogmaals `r` om terug te keren. Voor een LFS-bestand geldt alleen staging op bestandsniveau (er zijn geen hunks of regels om te stagen). Onder de kaart openen **Openen in standaardprogramma** en **Openen in** (`o`, `Shift+O` of het palet) het echte bestand, nooit de pointer: zie [Openen in](open-in.md#git-lfs-bestanden).
- **De Bestandsweergave** van een LFS-bestand zegt `Git LFS object, 12.3 MB, sha256:ab12cd34...` en waar je de inhoud kunt krijgen.
- **De banner.** Voor een repository die LFS gebruikt terwijl `git-lfs` ontbreekt of ouder is dan 3.0, zegt een banner in het Statuspaneel dat. Klik erop (of gebruik de opdracht *LFS: Installatiepagina openen* van het palet) om https://git-lfs.com te openen. Verder verandert er niets: het venster opent, diffs werken, grote bestanden verschijnen gewoon als pointers.
- **Het bestandsmenu en het palet.** Rechtsklik op een gewijzigd bestand in het Statuspaneel: *Volgen met LFS: dit bestand* / *deze extensie* voor een gewoon bestand; *Bestand vergrendelen (LFS)*, *Bestand ontgrendelen (LFS)* of *Geforceerd ontgrendelen* voor een LFS-bestand, afhankelijk van wie de lock houdt. De rij van een vergrendeld bestand zegt `vergrendeld door jou` / `vergrendeld door <naam>`. Alles is ook een opdracht in het palet (`Ctrl+P`, typ `LFS`): initialiseren, een patroon tracken of untracken, objecten fetchen, pullen of pushen, prune, locks tonen, de installatiepagina openen.
- **Een mislukte actie legt zichzelf uit.** Als een LFS-actie faalt omdat `git-lfs` ontbreekt, bevat een dialoogvenster de melding en een knop *Installatiepagina openen*.
- **Instellingen.** *Instellingen > LFS* (per repository) toont of LFS is geïnitialiseerd, somt de gevolgde patronen op en heeft **LFS hier instellen**. *Git & codering* (per repository) heeft de schakelaar **Git LFS**, een alleen-lezen rij **git-lfs-programma** en een knop **LFS initialiseren voor deze repository**.
- **Git LFS** uit (`git.lfs_enabled = false`, zie de [configuratienaslag](config-reference.md)) verbergt dit alles. De eigen LFS-filters van git blijven werken.
- De LFS-toestand wordt kort nadat de repository opent gelezen, na elke wijziging, bij een handmatige vernieuwing, als `.gitattributes` verandert, en elke tien seconden voor een repository die LFS gebruikt.

## Bekende beperkingen

- Een repository waarvan de LFS-patronen alleen in een *geneste* `.gitattributes` staan (niet de versie in de hoofdmap, niet `.git/info/attributes`) wordt pas als LFS-gebruiker herkend zodra hij een LFS-objectopslag (`.git/lfs`) heeft, niet eerder.
- `git lfs migrate` zelf wordt niet gebruikt. Bestanden over de hele geschiedenis naar of uit LFS verplaatsen doen de herschrijfregels *Bestanden naar LFS migreren* / *Bestanden uit LFS terughalen*: ze werken op de lokale kloon, alleen de `.gitattributes` in de root wordt geschreven, en de objecten moeten daarna nog worden gepusht (*LFS-objecten versturen* in het resultaatpaneel, of `lfs push --all`).
