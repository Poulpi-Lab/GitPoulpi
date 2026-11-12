[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · **Nederlands** · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Zoeken

`Ctrl+Alt+F` (of `Ctrl+P`, dan `Zoeken: Werkruimte...`) opent een fuzzy-zoeker
over **elke repository waarin je werkt**: de repositories van de actieve
[werkruimte](workspaces.md) als er een is, anders de open tabbladen. De knop **Globaal zoeken** (het
vergrootglas met zijn label in de titelbalk, direct links van het tandwiel) opent hem ook. Hij vindt:

- **repositories** (op naam, dan op pad),
- **branches** (lokaal en remote-tracking) en **tags**,
- **worktrees** (op map, dan op pad en branch),
- **stashes** (op bericht),
- **commits** (op onderwerp; de tekst van een commit wordt alleen doorzocht
  als het onderwerp niet overeenkomt en je drie tekens of meer typte),
- **pull of merge requests** en **issues** (op titel, `#nummer`, auteur en
  branch).

![De zoekpopup boven twee open repositories: branches, een stash en een commit die overeenkomen met "main", elk met zijn repository rechts](../images/workspace-search.png)

Resultaten zijn gegroepeerd per soort, binnen elke groep de beste treffer
eerst. Elke rij heeft het pictogram van de soort, de overeenkomende tekens
gemarkeerd, een gedimd detail (de upstream van een branch, de auteur en branch
van een aanvraag, de positie van een stash) en rechts de naam van de
repository, zodat twee repositories met een branch `main` nooit door elkaar
raken. In de titel verschijnt een spinner zolang een repository nog zijn
commits indexeert (de eerste zoekopdracht in een grote repository kan even
duren); de andere soorten zijn er meteen en de commits komen erbij zodra ze
klaar zijn.

## Erheen gaan

| Toets | Doet |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Door de resultaten bewegen (de groepskoppen worden overgeslagen). |
| `Enter` | Gaat naar het resultaat: het tabblad van de repository opent (of wordt geactiveerd), dan wordt het resultaat getoond. |
| `Esc` | Sluit het zoeken. |
| `Alt+1` .. `Alt+8` | Een soort aan- of uitzetten (de chips boven de lijst, in hun volgorde). `Alt+0` zet ze allemaal aan. |

Wat "erheen gaan" betekent:

| Resultaat | Enter |
|---|---|
| Branch of tag | Selecteert hem in het paneel Refs en zijn commit in de Log. |
| Commit | Selecteert hem in de Log, en laadt eerst meer van de Log als hij ouder is dan wat geladen is. |
| Stash | Selecteert zijn rij in de sectie STASHES. |
| Worktree | Opent hem als tabblad. |
| Pull/merge request, issue | Selecteert zijn rij in de zijbalk (in het tabblad van die repository). |
| Repository | Wisselt naar haar tabblad (en opent het als het niet open is). |

`Ctrl+N Backspace` (of `Terug` in het palet) brengt je terug naar waar je was,
zoals bij elke weergavewissel die een agent voor je doet. Erheen gaan checkt
niets uit, merget niets en wijzigt niets.

Met de muis: een klik op een rij selecteert hem, een dubbelklik gaat erheen,
een klik op een chip zet die soort aan of uit, het wiel scrolt en een klik
erbuiten sluit de popup.

## Beperken met een voorvoegsel

Begin de zoekopdracht met een voorvoegsel om maar één soort te doorzoeken,
wat de chips ook zeggen:

| Voorvoegsel | Soort |
|---|---|
| `r:` | repositories |
| `b:` | branches |
| `t:` | tags |
| `w:` | worktrees |
| `s:` | stashes |
| `c:` | commits |
| `pr:` (of `mr:`, `p:`) | pull of merge requests |
| `i:` | issues |
| `#` | pull/merge requests en issues (`#12` vindt nummer 12) |

`fix: login` is een gewone zoekopdracht, geen voorvoegsel: alleen één enkele
letter (of `pr:` / `mr:`) gevolgd door een dubbele punt, of een beginnende `#`,
telt mee.

## Hoe er wordt gematcht

Dezelfde drie niveaus als het opdrachtenpalet: je tekst letterlijk, dan
letterlijk ergens, dan zijn tekens in volgorde maar uit elkaar. Een eerdere
treffer en een kortere tekst winnen. Hoofdletters en kleine letters zijn gelijk.

## Pull requests en issues

Ze komen uit wat GitPoulpi al voor elke repository heeft geladen (de lijsten
achter de zijbalksecties PULL REQUESTS en ISSUES), bewaard in een klein
cachebestand in de map `.gitpoulpi/` van de repository. Het zoeken **roept
GitHub of GitLab nooit aan**: een repository waarvan de aanvragen nooit in een
venster zijn geladen heeft er geen om te vinden, en wat je ziet is zo vers als
de laatste verversing van de zijbalk. Zonder gekoppeld account valt er niets te
vinden, en niets in te stellen.

## Vanaf de opdrachtregel

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` doet
hetzelfde en toont JSON (zie de
[CLI-referentie](cli-reference.md#werkruimtes-en-zoeken)); de MCP-server heeft
een tool `search`, zodat een agent kan vinden wat jij kunt vinden.
`search.commit_limit` in
[Instellingen](config-reference.md#search-het-zoeken-in-de-werkruimte) begrenst hoeveel
commits van elke repository worden geïndexeerd (`0` zet het zoeken in commits
uit).

## De toets wijzigen

`Zoeken: Werkruimte...` is een opdracht van het palet en de toets is de actie
`global.workspace_search`: herbind hem onder Instellingen > Toetsenbord & muis,
of in de tabel `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
