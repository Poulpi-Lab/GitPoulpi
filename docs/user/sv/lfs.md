[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · **Svenska** · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS (”Large File Storage”) håller stora filer (designfiler, videor, dataset) utanför ett repositorys historik: git lagrar en *pekare* på tre rader text i stället, och själva innehållet ligger på LFS-servern och i en lokal cache under `.git/lfs`.

GitPoulpi behöver inget konto för något av detta. LFS använder de uppgifter som din remote redan använder (din git credential helper, eller ssh-agent).

## Vad som kräver programmet `git-lfs`, och vad som inte gör det

GitPoulpi kör standardprogrammet `git-lfs` för allt som konverterar eller flyttar filer (install, track, fetch, pull, push, lås).

Att läsa LFS-*tillståndet* kräver inget extra program: om ett repository använder LFS, vilka filer som är pekare, hur stora de är och om det riktiga objektet redan finns lokalt.

| | kräver `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (rapporterar att programmet saknas i stället för att misslyckas) | nej |
| `gitpoulpi lfs files` | nej |
| install / track / untrack / prune | ja |
| fetch / pull / push | ja |
| lås | ja |

## Installera `git-lfs`

- **Windows**: ingår i Git for Windows; eller `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Allt annat: https://git-lfs.com

Version 3.0 eller nyare krävs. Efter installationen märker GitPoulpi det inom en minut (eller direkt vid `lfs status`).

## När `git-lfs` saknas

GitPoulpi kraschar eller avslutas aldrig på grund av det:

- `gitpoulpi lfs status` lyckas och säger `git-lfs: not installed`, med installationslänken.
- En operation som kräver programmet misslyckas med ett tydligt meddelande som slutar med installationslänken (`https://git-lfs.com`) och en utgångskod som inte är noll; över MCP returnerar verktyget det meddelandet.
- Ett repository som använder LFS öppnas ändå; dess stora filer visas bara som pekare tills programmet är installerat.

## Titta på LFS-filer

```
gitpoulpi lfs status            # programmet, mönster, filter, hook
gitpoulpi lfs files             # pekarfiler i indexet, med storlek och lokal/inte nedladdad
gitpoulpi lfs files --at v1.0   # ... vid en viss revision
gitpoulpi show big.bin --at HEAD  # visar "Git LFS object, 12.3 MB, sha256:..." i stället för pekartext
```

## Ställa in ett repository: install, track, untrack

```
gitpoulpi lfs install                 # filter + hooks för DET HÄR repositoryt (git lfs install --local)
gitpoulpi lfs install --user          # i din globala git-konfiguration i stället (görs aldrig i tysthet)
gitpoulpi lfs track '*.psd'           # skicka *.psd via LFS
gitpoulpi lfs track '*.psd' --lockable   # ... och gör dem skrivskyddade tills de låses
gitpoulpi lfs track '*.bin' --renormalize  # ... och stagea om redan committade *.bin som pekare
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # ta bort gamla, redan pushade objekt från det lokala lagret
```

- `install` skriver som standard i repositoryts egen `.git/config` och hooks, så din globala git-konfiguration rörs aldrig om du inte anger `--user`.
- `track` och `untrack` redigerar `.gitattributes` och lämnar den **ostagad**: den visas som en vanlig ändring i arbetskatalogen som du själv committar. Filer som läggs till *efter* att spårningen satts lagras som pekare vid sin nästa commit.
- Filer som **redan är committade** som vanligt innehåll förblir vanliga i historiken. `--renormalize` stagear om dem så att din nästa commit ersätter dem med pekare. För att flytta dem i varje tidigare commit också, använd omskrivningsregeln *Migrera filer till LFS* (se [Skriva om historiken](history-rewrite.md#flytta-filer-till-eller-från-git-lfs)); den är GitPoulpis egen, inte `git lfs migrate`.
- Sätt mönstret inom citattecken så att ditt skal inte expanderar det.
- Via MCP är dessa vanliga verktyg som ändrar något (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), och de omfattas av `[mcp]`-spärren i `config.toml`.

## Flytta objekt: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # ladda ned objekt till det lokala lagret; arbetskatalogen rörs inte
gitpoulpi lfs pull  [--remote origin]           # fetch, ersätt sedan pekarfilerna med det riktiga innehållet
gitpoulpi lfs push  [--remote origin] [--all]   # ladda upp den aktuella grenens objekt (eller varje refs med --all)
```

- Remoten är som standard repositoryts standardremote, precis som för `fetch` och `push`.
- Uppgifterna är dina vanliga för remoten (credential helper, ssh-agent). GitPoulpi ber aldrig om ett eget konto.
- `pull` installerar LFS-filtren för det här repositoryt om de saknas: utan dem laddar git-lfs ner objekten men hoppar över att ersätta pekarna, och en senare checkout skulle sätta tillbaka pekare.
- MCP behandlar `lfs push` som `push`: ett destruktivt verktyg (det ändrar remoten).
- Fönstret visar förloppet för en överföring (filer och byte) på statusraden. CLI:t väntar på resultatet.

### Checka ut innehållet: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: skriv det riktiga innehållet över pekarfilerna, från det lokala lagret
```

Det laddar aldrig ner något (det gör `lfs pull`); objekt som saknas i det lokala lagret förblir pekare. Precis som `pull` installerar det först repositoryts LFS-filter när inga är konfigurerade. Som alla LFS-operationer går det inte att ångra; se [Ångra och gör om](undo-redo.md). Via MCP heter det `lfs_checkout` (en skrivoperation, inte destruktiv).

### Verktygsfältets LFS-knapp, och pull / push

Verktygsfältet har en knapp **`LFS ▾`** efter **Pop** (dold när `git.lfs_enabled` är av, och i ett repository där LFS varken är initierat eller används av dess `.gitattributes`: se [Inställningar > LFS](settings.md#repo-specifikt-lfs)). Ett klick öppnar en rullgardin: **Fetch: alla LFS-filer** (`lfs fetch --all`), **Checka ut alla LFS-filer** (`lfs checkout`), **Pull: alla LFS-filer** (`lfs pull`), **Push: alla LFS-filer** (`lfs push --all`, objekten för varje ref) och **Rensa lokal LFS-lagring** (`lfs prune`). Varje rad är också ett palettkommando (`LFS: Fetch (alla objekt)`, `LFS: Checka ut alla filer`, `LFS: Pulla objekt`, `LFS: Push (alla objekt)`, `LFS: Rensa lokala objekt`). `LFS: Öppna menyn` öppnar rullgardinen. Var och en har en `Ctrl+G`-sekvens: `l` för menyn, sedan `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` och `Shift+V` för raderna. Du kan ändra dem i *Inställningar > Tangentbord & mus*. När `git-lfs` saknas leder varje rad till installationsdialogen.

![Verktygsfältets LFS-rullgardin: fetch, checkout, pull och push av alla LFS-filer, och rensa det lokala lagret](../images/lfs-menu.png)

**Pull och push hanterar LFS själva.** I ett repository som använder LFS, med `git-lfs` installerat, kontrollerar en vanlig `pull` först att repositoryts LFS-filter är installerade, och en vanlig `push` kontrollerar dess `pre-push`-hook. När något saknas kör GitPoulpi `git lfs install --local` (bara det här repositoryt) före överföringen, och statusraden säger ”LFS-filter installerade”. Utan hooken skulle en push inte ladda upp några LFS-objekt. Stäng av det med `git.lfs_autofix_filters = false` (*Inställningar > Git & kodning > Installera LFS-filter före pull och push*). En vanlig `push` laddar fortfarande bara upp den pushade grenens objekt, aldrig varje refs; prune sker aldrig av sig självt.

## Fillås

Vissa filer går inte att merga (en Photoshop-fil, en spelbana, en binär resurs). Git LFS *lås* låter en person säga ”jag redigerar den här”: ingen annan kan låsa den eller pusha ändringar av den förrän den låses upp.

```
gitpoulpi lfs locks  [--remote origin]              # vem som håller vilken fil (dina är markerade)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Låsning är en funktion hos **LFS-servern** (GitHub, GitLab och Gitea har den). Den nås med remotens egna uppgifter; GitPoulpi har inget eget konto.
- En server utan låsning får `lock` att misslyckas med ett meddelande som säger det (det typade felet `LfsLockingUnsupported`). `locks` på en sådan server listar ingenting, vilket är vad git-lfs självt rapporterar.
- Att spåra ett mönster med `--lockable` (`lfs track '*.psd' --lockable`) gör filerna skrivskyddade tills du låser dem.
- `unlock` av någon annans lås nekas; `--force` bryter det, vilket är anledningen till att MCP behandlar `lfs_unlock` som ett destruktivt verktyg. `lfs_locks` är skrivskyddat.

## Klona ett LFS-repository

`gitpoulpi clone <url>` (och Klona-dialogen i TUI:n) checkar ut repositoryt *utan* att köra LFS-filtren och laddar sedan, när repositoryt använder LFS och `git-lfs` är installerat, ner objekten med `git lfs pull` (och installerar filtren för det repositoryt om inga är konfigurerade) och visar det som en egen fas, *Laddar ned LFS-objekt*. `--no-lfs` hoppar över det steget och lämnar pekare.

En clone **misslyckas aldrig på grund av LFS**: om `git-lfs` saknas, eller en nedladdning misslyckas, lyckas klonen ändå och de stora filerna förblir pekare; repositoryts LFS-status säger varför, och `gitpoulpi lfs pull` slutför jobbet när problemet är åtgärdat.

## I terminalgränssnittet

- **`LFS`-taggen.** En fil som lagras med Git LFS har en liten `LFS`-tagg efter namnet i Status-rutans listor Unstaged-filer och Staged-filer och i en commits lista över ändrade filer. **Bara LFS** (`o` med Status-rutan eller Log i fokus, kryssrutan i fillistornas verktygsfält eller **Filer: Växla bara LFS** i paletten) begränsar listorna till LFS-filerna. Kryssrutan sitter längst till höger på filterraden (Status-rutan: i verktygsfältet, eller i dess överflödesmeny när det är smalt); i en commits fillista krymper den till `[ ] LFS` i en smal ruta i stället för att försvinna. Den visas bara när de listade filerna har några. I Status-rutan och i en commits ändrade filer räknas en fil som LFS när dess sökväg är en pekare i indexet (ett batchat `git`-anrop per uppdatering, inget alls för ett repository utan `filter=lfs`-attribut). Med **Visa alla filer** på i Log listas hela trädet vid den valda revisionen och LFS-filerna är revisionens egna, lästa från dess egna blobbar (cachade per revision, utanför renderingstråden, inget `git-lfs`-program behövs), så en fil som är LFS där men inte i indexet taggas och behålls av filtret.
- **Diff-kortet.** När du öppnar en LFS-fils diff visas ett kort i stället för pekartext: `Git LFS-objekt`, `1.2 MB -> 1.4 MB`, de korta gamla och nya oid:erna, och om det nya objektet är `lagrat lokalt` eller `inte nedladdat`. Tryck `r` för den råa pekardiffen, `r` igen för att komma tillbaka. Bara staging på filnivå gäller för en LFS-fil (det finns inga hunks eller rader att stagea). Under kortet öppnar **Öppna i standardprogram** och **Öppna i** (`o`, `Shift+O` eller paletten) den riktiga filen, aldrig pekaren: se [Öppna i](open-in.md#git-lfs-filer).
- **Filvyn** för en LFS-fil visar `Git LFS object, 12.3 MB, sha256:ab12cd34...` och hur du får tag i innehållet.
- **Bannern.** För ett repository som använder LFS medan `git-lfs` saknas eller är äldre än 3.0 säger en banner i Status-rutan det. Klicka på den (eller använd palettkommandot *LFS: Öppna installationssidan*) för att öppna https://git-lfs.com. Inget annat ändras: fönstret öppnas, diffar fungerar, stora filer visas bara som pekare.
- **Filmenyn och paletten.** Högerklicka på en ändrad fil i Status-rutan: *Spåra med LFS: den här filen* / *den här filändelsen* för en vanlig fil; *Lås fil (LFS)*, *Lås upp fil (LFS)* eller *Tvinga upplåsning (LFS, någon annans lås)* för en LFS-fil, beroende på vem som håller låset. En låst fils rad säger `låst av <namn>`. Allt finns också som palettkommando (`Ctrl+P`, skriv `LFS`): initiera, spåra eller sluta spåra ett mönster, fetcha, pulla eller pusha objekt, rensa, visa lås, öppna installationssidan.
- **En misslyckad åtgärd förklarar sig själv.** Om en LFS-åtgärd misslyckas för att `git-lfs` saknas visar en dialogruta meddelandet och en knapp *Öppna installationssidan*.
- **Inställningar.** *Inställningar > LFS* (per repository) visar om LFS är initierat, listar de spårade mönstren och har **Konfigurera LFS här**. *Git & kodning* (per repository) har omkopplaren **Git LFS**, en skrivskyddad rad **Programmet git-lfs** och en knapp **Initiera LFS för det här repositoryt**.
- Med **Git LFS** av (`git.lfs_enabled = false`, se [konfigurationsreferensen](config-reference.md)) döljs allt detta. Gits egna LFS-filter körs ändå.
- LFS-tillståndet läses strax efter att repositoryt öppnats, efter varje ändring, vid manuell uppdatering, när `.gitattributes` ändras och var tionde sekund för ett repository som använder LFS.

## Kända begränsningar

- Ett repository vars LFS-mönster bara finns i en *nästlad* `.gitattributes` (inte rotens, inte `.git/info/attributes`) upptäcks som LFS-använd först när det har ett LFS-objektlager (`.git/lfs`), inte före.
- `git lfs migrate` i sig används inte. Att flytta filer till eller från LFS i hela historiken görs av omskrivningsreglerna *Migrera filer till LFS* / *Ta tillbaka filer från LFS*: de arbetar på den lokala klonen, bara rotens `.gitattributes` skrivs, och objekten måste fortfarande pushas efteråt (*Skicka LFS-objekt* i resultatpanelen, eller `lfs push --all`).
