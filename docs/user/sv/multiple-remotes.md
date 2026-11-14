[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · **Svenska** · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Flera remotes för en gren

Många repositoryn ligger på mer än en värd: en huvudremote (`origin`), en spegel på en annan värd, en backup. GitPoulpi låter en gren pusha till alla med ett klick, och om du vill pulla från fler än en, utan att ändra vad rent git gör. Inget av detta kräver ett konto, och allt sparas i repositoryts egen `.git/config`, så rent git och andra verktyg fortsätter fungera.

## Push-mål och pull-källor

- **Upstream** är gits eget begrepp (`branch.<name>.remote` och `merge`). Den är grenens *primära* remote. GitPoulpi ändrar aldrig hur den fungerar.
- Ett **push-mål** är en extra remote som grenen också pushar till. Push-knappen skickar grenen till upstream och till varje push-mål.
- En **pull-källa** är en extra remote som en pull också hämtar commits från. Den är av tills du slår på den (kryssrutan **Pull också**): att slå ihop två remotes du inte har valt är ingen standard någon vill ha.

Ett mål kan använda ett annat grennamn på den remoten (`backup` får `feat-x`).

## Dialogen Remotes

Högerklicka på en gren och välj **Remotes för <gren>...** (finns också i Push-rullgardinen, i paletten som `Git: Remotes för gren...` och genom att klicka på märket `⇉3` på grenens rad i avsnittet Refs). Den listar varje remote i repositoryt:

| Kolumn | Betydelse |
|---|---|
| Remote, Gren | Remoten och grenens namn där. **Byt namn på remote...** ändrar det namnet för den markerade raden. |
| Push | Kryssa i den för att göra remoten till ett push-mål för den här grenen. |
| Pull också | Kryssa i den för att låta en pull också ta in den remoten. Upstream pullas alltid. |
| Läge | Hur grenen står mot den: i synk, före, efter, ny, orelaterad. |

En rad märkt "remote saknas" anger en remote som grenen fortfarande nämner men som repositoryt inte längre har; **Ta bort** glömmer den. Samma dialog anger upstream (listan **Upstream**) och **Ta bort upstream** rensar den. **Verkställ** skriver allt som en enda ändring som går att ångra.

Ett repository kan också ge varje ny gren samma extra mål: **Inställningar > Git & kodning** (den repo-specifika sidan), "Extra push-mål för nya grenar" (en kryssruta per remote). De kopieras till en gren när den först får en upstream; grenar som redan finns rörs inte.

## Pusha

- **Push-knappen** på en gren med extra mål: pushar till alla. När varje remote helt enkelt tar emot de nya commits körs den genast och förloppsvyn visar hur varje remote får en bock. Om något kräver ett beslut (en remote har commits som du inte har) öppnas i stället planeringsdialogen.
- **Push ▾** (pilen bredvid knappen; också själva knappen när **Push-knapp** är satt till **fråga** i Inställningar):
  - `Push till alla (origin, gitlab, backup)`
  - `Push endast till <remote>`, en per remote för grenen
  - `Push till valda remotes...`: finns alltid, för vilken gren som helst, även en med bara en remote
  - `Remotes för <gren>...`
- Grenens högerklicksmeny och paletten (`Git: Push till alla mål`, `Git: Push till remote...`, `Git: Push till valda remotes...`) erbjuder samma åtgärder med samma namn.

**Push till valda remotes...** öppnar en lista över varje remote i repositoryt, ikryssade enligt grenens upstream och mål. Den gäller bara den här pushen; kryssa i **Kom ihåg för den här grenen** för att spara valet som grenens push-mål.

### Planen

Före en gruppush tittar GitPoulpi på varje remote (en snabb förfrågan, och en nedladdning från den remoten bara när den skiljer sig), så att ett problem på den tredje remoten syns innan den första pushen börjar. Dialogen listar varje remote med hur grenen står mot den och vad som ska göras:

| Situation | Standardåtgärd | Erbjuds också |
|---|---|---|
| Remoten har inget av grenen, eller saknar några commits | Push | Hoppa över |
| Remoten ligger före (du har inget nytt åt den) | Fast-forward från den | Hoppa över |
| Remoten skiljer sig för att du gjort rebase eller amend på det du pushat | Force push (lease) | Merge, Rebase, Hoppa över |
| Remoten har någon annans commits | Merge från den (Rebase när **Pull-knapp** är satt till rebase) | Rebase, Force push (lease), Hoppa över |
| Historiken delar ingenting med din | Hoppa över | Force push (lease) |
| Värden nekade grenen (skyddad) | Hoppa över | inget: en force skulle också nekas |

Varje rads åtgärd är en lista du kan ändra (Vänster/Höger bläddrar i den, Enter öppnar den, `s` hoppar över raden). En merge visar om den skulle ge konflikt och i vilka filer. En **force push** bekräftas alltid, anger remoten och hur många commits den ersätter, använder alltid en lease (aldrig en ren force) och går inte att ångra: resultatet anger vilken commit som ersattes och kommandot som sätter tillbaka den. **Hoppa över problem** sätter varje problemrad till Hoppa över. **Kör** (Ctrl+Enter) startar.

Merges, fast-forwards och rebases körs först, i listans ordning, så att din gren når sitt slutläge en gång; sedan går pusharna ut en remote i taget, upstream först. Det finns ingen atomisk "allt eller inget" över remotes: var och en slutar med sitt eget resultat, och ett fel på en (nätverk, en hook) stoppar inte de andra. **Försök igen** (eller `Git: Försök igen med misslyckade pushar`) försöker med dem igen. Att stänga förloppsvyn stoppar aldrig körningen; `Git: Visa gruppens Push` tar tillbaka den.

### Konflikter och Fortsätt

Om en merge eller rebase i körningen stannar på konflikter stängs dialogen och det vanliga merge-bannret och konflikteditorn tar över. En remsa över Log lyder `Gruppens push pausad (gitlab): lös och avsluta, sedan Fortsätt.` med knapparna **Fortsätt** och **Avbryt gruppen**.

- Lös konflikterna och avsluta merge eller rebase (i GitPoulpi eller med rent git).
- **Fortsätt** är gråad tills det är gjort, och den **trycks aldrig åt dig**: ingenting fortsätter av sig självt. Tryck på den (eller kör `Git: Fortsätt gruppens Push`) så pushar körningen till varje remote igen från din nya topp. Efter en rebase går en remote som fortfarande har de gamla commits tillbaka till planeringsdialogen (den behöver nu en force push) i stället för att forceras i tysthet.
- **Avbryt gruppen** (bekräftas; `Git: Avbryt gruppens Push`) kastar den merge eller rebase körningen startade och glömmer körningen. Pushar som redan gjorts finns kvar.

Remsan överlever en omstart av GitPoulpi, och CLI ser samma sparade körning.

## Pulla

- **Pull-knappen** pullar upstream precis som förut, om inte grenen har pull-källor: då pullar den upstream och varje källa, i ordning, var och en i det läge som **Pull-knapp** anger i Inställningar (merge, bara fast-forward eller rebase). En källa som inte kan fast-forwardas mergas (eller rebasas) som upstream; med bara fast-forward frågar den först när en källa behöver mer.
- **Pull ▾** lägger till `Pull från alla källor (origin + gitlab)` när det finns källor, och `Pull endast från <remote>` för varje remote för grenen: en engångspull av den remotens gren, vad kryssrutorna än säger.
- Paletten har `Git: Pull från alla källor` och `Git: Pull från en remote...`.
- En pull som stannar på konflikter lämnar samma remsa, märkt som en grupppull.

## Inställningar

Alla fyra finns på **Inställningar > Git & kodning**.

| Inställning | Standard | Betydelse |
|---|---|---|
| **Push-knapp** | direkt | direkt pushar genast; fråga öppnar Push-menyn först. |
| **Pull-knapp** | merge | Läget för Pull-knappen (merge, bara fast-forward eller rebase), och för pull-källorna. |
| **Kontrollera varje remote före en push** | på | Titta på varje remote före en gruppush (CLI följer den; `--no-preflight` stänger av den för ett anrop). |
| **Extra push-mål för nya grenar** (per repository) | inga | De remotes en ny gren också pushar till. |

## Från kommandoraden

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

När en remote behöver ett beslut svarar kommandot med planen (exit 3) och ändrar ingenting; besvara den med `--resolve gitlab=merge`. Se [CLI-referensen](cli-reference.md) för varje flagga. MCP-servern har samma verktyg.

## Med rent git

Målen är vanliga konfigurationsnycklar, så ett skript eller ett alias kan använda dem utan GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Ett värde är `<remote>` (samma grennamn) eller `<remote>:<branch>`, vilket är vad raden `d=${t#*:}` reder ut. Det rena aliaset tittar inte på remotes först, erbjuder ingen merge och kan inte fortsätta efter en konflikt: det är vad dialogen är till för.

## Bra att veta

- Två remotes som pekar på samma adress märks ("gitlab och mirror pushar till samma URL."); den andra hoppas över som standard.
- En värd med en skyddad gren nekar en force push; raden säger det och erbjuder bara Hoppa över.
