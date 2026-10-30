[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · **Svenska** · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Arbetsytor

En **arbetsyta** är en namngiven grupp av repositories: de du arbetar med
tillsammans (ett användargränssnitt, dess API, dokumentationen). Den kommer ihåg
vilka av dem som är öppna som flikar och kan bära en **standardprofil för
commits**, så att commits i de repositoryna använder rätt identitet utan att du
byter för hand. Ett repository kan ligga i flera arbetsytor, och att ligga i
ingen är helt i sin ordning: ”Ingen arbetsyta” är den vanliga fliklistan.

Arbetsytor lagras i `workspaces.toml`, bredvid `config.toml`, delade av varje
session, fönstret, CLI:n och MCP (`gitpoulpi workspace ...`, se
[CLI-referensen](cli-reference.md)). En arbetsyta som läggs till, byter namn eller tas bort från ett skal eller med `gitpoulpi workspace ...` medan fönstret är
öppet dyker upp av sig självt inom ungefär en sekund (fliken Arbetsytor, flikradens chip och dess lista), utan
tangenttryckning. Att stänga eller ta bort en arbetsyta rör
aldrig ett repository på disken. Ändringar av arbetsytor är inställningar, som
favoriter och profiler: de ingår inte i [Ångra och gör om](undo-redo.md), och
Ta bort frågar först.

## Arbetsyte-chipet

Så snart du har en arbetsyta dyker ett chip upp i flikraden direkt efter `⌂`: en
färgad punkt och den aktiva arbetsytans namn, eller ”Ingen arbetsyta” (dämpat). I
en smal rad krymper det till punkten, och i en mycket smal försvinner det.

![Arbetsytelistan över flikraden: Ingen arbetsyta, Orbit (markerad), Side projects, Archive och Hantera arbetsytor, med chipet Orbit och dess profil ”Work - från arbetsytan Orbit” i verktygsfältet](../images/workspace-switch.png)

- **Klicka på det**, tryck `Alt+W` eller kör `Arbetsyta: Byt...` i paletten för
  att öppna en lista: dina arbetsytor, ”Ingen arbetsyta” och ”Hantera
  arbetsytor...” (som öppnar fliken Arbetsytor).
- **Att byta byter flikarna.** Flikarna du hade öppna sparas för arbetsytan du
  lämnar, och målets flikar kommer tillbaka (första gången dess repositories som
  finns). Byt tillbaka och allt ligger som du lämnade det. Fönstret startas om
  som vid ett flikbyte; daemonen finns kvar.
- **Arbetsytan kommer ihåg**: nästa start är du tillbaka i den.
- **Att stänga den sista fliken** i en arbetsyta låter arbetsytan vara aktiv och
  visar fliken Hem.
- **`+` (ny flik) medan en arbetsyta är aktiv** lägger repositoryt du väljer i
  fliklistan och frågar inget. För att lägga det i själva arbetsytan, använd
  ”Lägg till i arbetsytan” i flikens högerklicksmeny (eller `Arbetsyta: Lägg till
  fliken i arbetsytan`); ”Ta bort från arbetsytan” tar ut det igen och låter
  fliken vara öppen.

Utan någon arbetsyta än finns inget chip: skapa en från fliken Arbetsytor eller från
paletten först.

## Fliken Arbetsytor

Öppna den från menyn vid `+`, med `Alt+J` eller med `Arbetsyta: Hantera...` i paletten; dess `x` (eller `Ctrl+W`) stänger den, och den förblir öppen mellan starter tills du stänger den. Varje arbetsyta är en rad
med sin färgpunkt, sitt namn, hur många repositories den listar (och hur många
som saknas), sin standardprofil, och ”aktiv” när fönstret är i den. `Right`,
`Space` eller ett klick fäller ut den till dess repositories.

![Fliken Arbetsytor: Orbit utfälld till sina två repositories, Side projects, och Archive markerad ”1 saknas”](../images/workspaces.png)

| Per arbetsyta | Tangent | |
|---|---|---|
| Öppna | `Enter` | Gör den aktiv och öppnar dess flikar. |
| Redigera... | `e` | Formuläret nedan. |
| Ta bort... | `x` | Frågar först. Repositories rörs inte. |
| Fetch alla | `Shift+F` | Ett repository i taget, förloppet på statusraden, misslyckanden listade i slutet. |
| Pull alla | `Shift+P` | Detsamma, bara fast-forward: det stannar aldrig på en merge. |
| Klona de som saknas | `m` | Klonar varje saknat repository som har en sparad remote. |
| Ny arbetsyta... | `n` | |
| Från öppna flikar... | `a` | En ny arbetsyta med dina öppna flikar markerade. |

Ett **saknat** repository (dess mapp är borta) står kvar i listan, markerat, med
**Leta upp...** (`l` eller `Enter`: ange var det finns nu), **Klona** (när dess
`origin`-URL sparades när det lades till) och **Ta bort** (`x`). Varje rad har
också en `⋮`-meny (högerklick, `.` eller `Shift+F10`) med samma åtgärder, och
varje åtgärd finns i paletten under `Arbetsyta:`.

## Formuläret Ny / Redigera

![Formuläret Redigera arbetsyta: namn, färg, beskrivning, standardprofil och kryssrutelistan med repositories](../images/workspace-edit.png)

- **Namn**: obligatoriskt, unikt.
- **Färg**: en lista med temafärger; punkten i chipet och raderna.
- **Beskrivning**: fri text.
- **Profil**: en lista med dina `[[profiles]]`, eller ”Ingen”. Den är valfri.
- **✦ AI-instruktioner**: bara med AI på, se [nedan](#ai-instruktioner-för-en-arbetsyta).
- **Repositories**: en kryssrutelista över varje repository GitPoulpi känner till
  (öppna flikar, favoriter, senaste, arbetsytans egna, saknade inräknade).
  `Space` markerar en rad; **Lägg till mapp...** (`Ctrl+B`) väljer ett repository
  från disken. Varje nytt repositorys origin-URL läses från repositoryt och
  sparas (utan några inloggningsuppgifter), och det är den Klona använder senare.

## AI-instruktioner för en arbetsyta

Med AI påslaget (*Inställningar > AI*) har formuläret en rad **✦ AI-instruktioner** med en knapp **Redigera...** (`Enter` på
raden). Den öppnar en liten redigerare för det AI:n får veta i **varje repository i arbetsytan**: välj *Alla AI-åtgärder*
eller en åtgärd (commit-meddelande, förklara, stash-meddelande, pull request, commit-composer, konfliktlösning,
kodgranskning) i listan **För**, välj **Kombinera** - *Lägg till i globala* (skickas efter de globala instruktionerna) eller
*Ersätt globala* (skickas i stället för dem) - och skriv texten. **Verkställ** behåller hela uppsättningen, **Spara** i formuläret
skriver den till `workspaces.toml` med arbetsytan; **Avbryt** kastar det du skrev. Med AI av är raden
dold och arbetsytans instruktioner lämnas som de är.

Arbetsytans ord sitter mellan de globala och repositoryts egna instruktioner (se
[AI-instruktioner](ai.md#instruktioner) för ordningen och för vad *ersätt* gör). Ett repository som tillhör
flera arbetsytor använder fönstrets aktiva arbetsyta när den innehåller det, annars den första som gör det. Från ett
skript: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` skriver ut dem som `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # varje AI-åtgärd i den här arbetsytan
text = "Use the Acme glossary."
mode = "add"                       # eller "replace"

[workspace.ai_overrides.code_review]   # en åtgärd
text = "Flag missing tests."
mode = "add"
```

## Standardprofilen

Commits signeras av det första av dessa som gäller:

1. profilen du valde **för hand** för det repositoryt (profilfältet i
   verktygsfältet),
2. den **aktiva arbetsytans** profil, när repositoryt är en av dess medlemmar,
3. `general.active_profile`.

Profilfältet i verktygsfältet visar varifrån den kommer: ”Work - från arbetsytan
Acme”. Fältet visas också när du bara har en profil, så länge en arbetsytas
standard gäller. En arbetsyta vars profil inte längre finns slutar helt enkelt
gälla. I CLI:n löser `commit --workspace <id>` (eller `GITPOULPI_WORKSPACE`) på
samma sätt.

## Tangenter

| Tangent | Åtgärd |
|---|---|
| `Alt+W` | Öppna arbetsytelistan |
| `Alt+J` | Öppna fliken Arbetsytor |
| `n`, `a` | Ny arbetsyta, ny från de öppna flikarna |
| `e` | Redigera arbetsytan |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch alla, Pull alla, Klona de som saknas, Leta upp |

Alla kan bindas om i Inställningar > Tangentbord & mus; hela listan finns i
[Kortkommandon](keybindings.md).
