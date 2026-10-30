[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · **Svenska** · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Sökning

`Ctrl+Alt+F` (eller `Ctrl+P` och sedan `Sök: Arbetsyta...`) öppnar en
fuzzy-sökare över **varje repository du arbetar i**: den aktiva
[arbetsytans](workspaces.md) repositories när det finns en, annars de öppna
flikarna. Knappen **Global sökning** (förstoringsglaset och dess etikett i titelraden, till
vänster om kugghjulet) öppnar den också. Den hittar:

- **repositories** (efter namn, sedan efter sökväg),
- **grenar** (lokala och remote-spårande) och **taggar**,
- **worktrees** (efter mapp, sedan efter sökväg och gren),
- **stashar** (efter meddelande),
- **commits** (efter rubrikrad; en commits brödtext söks bara när rubrikraden
  inte matchar och du skrev tre tecken eller fler),
- **pull/merge requests** och **ärenden** (efter titel, `#nummer`, författare och
  gren).

![Sökrutan över två öppna repositories: grenar, en stash och en commit som matchar ”main”, var och en med sitt repository till höger](../images/workspace-search.png)

Resultaten är grupperade efter typ, bästa träff först inom varje grupp. Varje rad
har typens ikon, de matchade tecknen framhävda, en dämpad detalj (en grens
upstream, författaren och grenen för en begäran, en stashs position) och
repositoryts namn till höger, så att två repositories med en gren `main` aldrig
förväxlas. En snurra visas i titeln när ett repository fortfarande indexerar sina
commits (den första sökningen i ett stort repository kan ta en stund); de andra
typerna finns direkt och commits ansluter så fort de är klara.

## Gå dit

| Tangent | Gör |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `PageUp` / `PageDown`, `Home` / `End` | Flyttar genom resultaten (grupprubrikerna hoppas över). |
| `Enter` | Går till resultatet: repositoryts flik öppnas (eller växlas till), sedan visas resultatet. |
| `Esc` | Stänger sökningen. |
| `Alt+1` .. `Alt+8` | Slår en typ på eller av (chipen ovanför listan, i deras ordning). `Alt+0` slår på alla. |

Vad ”går dit” betyder:

| Resultat | Enter |
|---|---|
| Gren eller tagg | Markerar den i Refs-rutan och dess commit i Log. |
| Commit | Markerar den i Log, och läser först in mer av Log när den är äldre än det som är inläst. |
| Stash | Markerar dess rad i avsnittet STASHES. |
| Worktree | Öppnar den som en flik. |
| Pull/merge request, ärende | Markerar dess rad i sidopanelen (i det repositoryts flik). |
| Repository | Växlar till dess flik (och öppnar den när den inte är öppen). |

`Ctrl+N Backspace` (eller `Tillbaka` i paletten) går tillbaka dit du var, som vid
varje vybyte en agent gör åt dig. Inget checkas ut, slås ihop eller ändras av att
gå någonstans.

Med musen: ett klick på en rad markerar den, ett dubbelklick går dit, ett klick
på ett chip slår den typen på eller av, hjulet rullar och ett klick utanför
stänger rutan.

## Avgränsa med ett prefix

Börja frågan med ett prefix för att bara söka en typ, vad chipen än säger:

| Prefix | Typ |
|---|---|
| `r:` | repositories |
| `b:` | grenar |
| `t:` | taggar |
| `w:` | worktrees |
| `s:` | stashar |
| `c:` | commits |
| `pr:` (eller `mr:`, `p:`) | pull/merge requests |
| `i:` | ärenden |
| `#` | pull/merge requests och ärenden (`#12` hittar nummer 12) |

`fix: login` är en vanlig fråga, inte ett prefix: bara en ensam bokstav (eller
`pr:` / `mr:`) följd av kolon, eller ett inledande `#`, räknas.

## Hur den matchar

Samma tre nivåer som kommandopaletten: din text ordagrant, sedan ordagrant var
som helst, sedan dess tecken i ordning men isär. En tidigare träff och en kortare
text vinner. Versaler och gemener är lika.

## Pull requests och ärenden

De kommer från det GitPoulpi redan har läst in för varje repository (listorna
bakom sidopanelens avsnitt PULL REQUESTS och ÄRENDEN), sparat i en liten cachefil
i repositoryts mapp `.gitpoulpi/`. Sökningen **anropar aldrig GitHub eller
GitLab**: ett repository vars begäranden aldrig lästes in i ett fönster har inga
att hitta, och det du ser är lika färskt som sidopanelens senaste uppdatering.
Utan ett länkat konto finns inget att hitta och inget att konfigurera.

## Från kommandoraden

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` gör
detsamma och skriver ut JSON (se [CLI-referensen](cli-reference.md#arbetsytor-och-sökning));
MCP-servern har ett verktyg `search`, så en agent kan hitta det du kan.
`search.commit_limit` i [Inställningar](config-reference.md#search-arbetsytesökningen)
begränsar hur många commits i varje repository som indexeras (`0` stänger av
commitsökning).

## Ändra tangenten

`Sök: Arbetsyta...` är ett palettkommando och tangenten är åtgärden
`global.workspace_search`: bind om den under Inställningar > Tangentbord & mus,
eller i tabellen `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
