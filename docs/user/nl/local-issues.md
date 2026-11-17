[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · **Nederlands** · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Lokale issues en Obsidian

Bewaar je issues als een map met Markdown-notities: geen account, geen token, geen netwerk. De map kan in de
repository staan (`docs/issues/`, samen met de code gecommit) of ergens op schijf, bijvoorbeeld een submap van een
[Obsidian](https://obsidian.md)-vault. GitPoulpi leest en schrijft de notities. Obsidian, elke editor of een shell kan ze
tegelijk bewerken, en de lijsten volgen.

![Lokale Markdown-issues in de zijbalk, met het rijmenu van een notitie: de notitie openen, sluiten, zijn labels of mijlpaal bewerken](../images/local-issues.png)

## Instellen

1. **Instellingen > Integraties > Issuetrackers > Lokale Markdown > Profiel toevoegen.** Typ de map, of druk op
   **Bladeren...** en kies hem. Een relatieve map (`docs/issues`) wordt gelezen ten opzichte van elke repository waaraan hij
   gekoppeld is; een absolute (`/home/me/Vault/Projects/Issues`) is overal dezelfde. Er is geen token in te vullen.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. Open in een repository de sectie **ISSUES** van de zijbalk en kies de provider **Local Markdown - <naam>** (de map
   wordt ermee gekozen; de keuzelijst Repository toont de map en zijn directe submappen als je een andere wilt).
   - CLI: `gitpoulpi hosting link issues --id <profiel>` (de map van het profiel is de standaard-`--project`).

Niets wordt uit zichzelf gekoppeld: zoals elke issuebron is hij per repository opt-in.

## Opmerkingen bij een notitie

`Enter` op de rij van een notitie in de sectie ISSUES, of op zijn kaart, opent zijn gesprek: de tekst van de notitie, daarna de
opmerkingen. Opmerkingen worden in het notitiebestand zelf bewaard, in een laatste sectie `## Comments`, elk als een item
`### <auteur> - <UTC-tijd>` (een bewerkt item krijgt `(edited <tijd>)` erbij). Niets boven die kop wordt ooit aangeraakt, dus
Obsidian en een shell lezen en bewerken hetzelfde bestand. Je kunt de opmerkingen van de lokale auteur schrijven, bewerken en
verwijderen (de `local.me` van het profiel, anders de verbonden gebruiker, de naam die *Aan mij toegewezen* gebruikt); die van
niemand anders. Het vak is de gewone Markdown-composer met een voorbeeld, en *Schrijven als <naam> (lokale notitie)* noemt de
auteur. Er gaat niets over het netwerk. Een kop `## Comments` die je met de hand schreef, wordt als zodanig gelezen.

## De notities

Eén notitie per issue, genoemd `<id>-<slug>.md` (`12-crash-on-empty-name.md`), met een frontmatter-blok:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Vrije Markdown. GitPoulpi herschrijft de inhoud nooit.
```

- Nieuwe issues krijgen `max(id) + 1`.
- GitPoulpi schrijft alleen de eigenschappen die het zelf beheert (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Elke andere eigenschap, de inhoud, de regeleinden en de volgorde van de sleutels blijven byte voor byte behouden.** Schrijven is
  atomair (een tijdelijk bestand, daarna een hernoeming).
- Lezen is vergevingsgezind: een notitie zonder `id` neemt hem uit haar bestandsnaam, een notitie zonder `status` is `todo`, een notitie
  zonder frontmatter neemt haar titel uit de eerste `# kop`. Een bestand dat geen issue is (`readme.md`) wordt genegeerd.
- Een status is vrije tekst. **Open** betekent "niet de klaar-status".

### Opmerkingen in een notitie

Een opmerking bij een notitie wordt in de notitie zelf geschreven, in een laatste sectie `## Comments` (geen netwerk, en het
leest prima in Obsidian):

```markdown
(de tekst van de notitie)

## Comments

### kai - 2026-11-15T10:00:00Z
Eerste opmerking, **elke Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
De tweede.
```

- Een item begint bij een regel die precies `### <auteur> - <UTC-tijd>` is, met ` (edited <UTC-tijd>)` na een bewerking; zijn
  tekst loopt tot het volgende item. De sectie is de laatste kop `## Comments` buiten een codeblok en loopt tot het einde van
  het bestand, dus schrijf er niets achter.
- Een opmerkingsregel die als een item of als de kop zou worden gelezen, wordt met een backslash ervoor opgeslagen (`\### ...`,
  wat Markdown als dezelfde tekst toont) en zonder die teruggelezen.
- Plaatsen voegt aan het einde toe en laat alles erboven byte voor byte ongemoeid (regeleinden inbegrepen); een bewerking of
  verwijdering herschrijft alleen de sectie. Een bericht verplaatst ook de `updated` van de notitie (een notitie zonder
  frontmatter krijgt er geen).
- De auteur is de lokale auteur van het profiel (de "ik" van de instellingen Lokale issues). Je kunt alleen je eigen
  opmerkingen bewerken of verwijderen; de id van een opmerking is zijn tijd en auteur, en twee berichten in dezelfde seconde
  worden onderscheiden door het tweede een seconde op te schuiven.
- Er zijn geen antwoorden, geen interne opmerkingen en geen gebeurtenisgeschiedenis voor een notitie.

## Wat je kunt doen

| In de sectie ISSUES | Toetsenbord / palet |
|---|---|
| Nieuw issue (de `+`, of `n`): titel, beschrijving, status, toegewezenen, labels, mijlpaal | `Hosting: Nieuw issue...` |
| Toegewezenen / labels / mijlpaal bewerken via het rijmenu. Kies uit wat de map al gebruikt, of **typ een nieuw label of een nieuwe login** en neem de rij `"..." toevoegen` | `Hosting: Labels bewerken...` en zijn broers |
| **Issue sluiten** / **Issue heropenen** (zet de klaar-status, of de eerste andere) | `Lokale issues: Issue sluiten...` / `Issue heropenen...` |
| **Notitie openen** (je standaardprogramma voor `.md`) | `Integraties: Issue openen...` |
| **Openen in Obsidian** (alleen als de map in een vault staat, dus met een map `.obsidian/` erboven) | `Lokale issues: Openen in Obsidian...` |
| Filters: Open / Gesloten / Alle, "Aan mij toegewezen" | de filterkeuzelijst |

"Aan mij toegewezen" komt overeen met de instelling `me` van het profiel, anders met de verbonden naam, anders met de profielnaam.

Branches en commits werken zoals bij GitHub: "Branch maken van issue" geeft `12-crash-on-empty-name`, en `#12` in een commit-
bericht of `12-...` in een branchnaam wordt herkend.

Wijzigingen buiten GitPoulpi (Obsidian, een shell, een `git pull`) verschijnen binnen ongeveer een seconde: de bestandsnamen
en wijzigingstijden van de map worden op een achtergrondthread bekeken, nooit op de render-thread.

## Configuratiebestand

In `config.toml`, naast het profiel (elke sleutel is optioneel):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # de volgorde van de kolommen van het bord
done_status = "done"                            # sluit een issue
reference_prefix = "LOC"                         # getoond als LOC-12 in plaats van #12
me = "kai"                                       # wie "aan mij toegewezen" betekent
```

Statussen die notities gebruiken maar die niet in `statuses` staan, volgen op de genoemde. `statuses` is de volgorde van de kolommen van het
[bord](#het-kanbanbord) (`gitpoulpi hosting issue options` drukt hem af).

## Het kanbanbord

![Het bord van een notitiemap: één kolom per status, de kaarten met hun toegewezenen, labels en mijlpaal](../images/issue-board.png)

Als je alleen werkt, of in een team dat zijn issues naast de code bewaart, is de map ook een kanbanbord.
**Kop van ISSUES > Bord**, of **Issues: Bord openen** in het palet, opent het over het venster; `Esc` sluit het.

- **Kolommen** zijn de statussen: eerst die in `statuses`, in die volgorde, daarna elke andere status die een notitie gebruikt.
  Een lege kolom wordt toch getoond. Elke kop draagt zijn aantal kaarten.
- **Kaarten** tonen het issuenummer, de titel (twee regels), de toegewezenen als initialen, de labels en de mijlpaal.
  Als de kolommen niet allemaal passen, scrolt het bord zijwaarts (de balk eronder, het wiel met `Shift`, of de pijlen).
- **Een kaart verplaatsen** doe je door hem naar een andere kolom te slepen (de kolom licht op onder de aanwijzer), met
  `Shift+Left` / `Shift+Right`, of met **Bord: Kaart verplaatsen naar...** (`m`), een keuzelijst van de kolommen. De kaart verplaatst
  direct; de `status` van de notitie wordt herschreven zoals in [de sectie over de notities](#de-notities), verder niets in het bestand. Als het schrijven
  mislukt, gaat de kaart terug en staat de fout op de statusbalk.
- **Een kaart openen** doe je met `Enter` of een dubbelklik : het [gesprek](conversations.md) van de notitie opent in het venster, met de tekst van de notitie en zijn opmerkingen (**Notitie openen** in het kaartmenu opent het bestand in je standaardprogramma voor `.md`). `e` bewerkt zijn
  toegewezenen, labels of mijlpaal, `.` of een rechtsklik opent het kaartmenu, `n` start een nieuw issue in de geselecteerde
  kolom (het formulier keert terug naar het bord als het sluit).
- **Filteren** kan op toegewezene (`a`), label (`l`), mijlpaal (`M`) of tekst (`/`); `x` wist ze. De filters zijn
  keuzelijsten: typ om te verfijnen, `Enter` kiest.
- **Live**: een notitie die vanuit Obsidian of een shell wordt gewijzigd, verplaatst zijn kaart binnen een seconde.

Elk muisgebaar heeft een toetsenbordtoets en een paletopdracht (**Bord: ...**, vermeld op de pagina
[Sneltoetsen](keybindings.md)).

## Obsidian Bases

**Palet: `Lokale issues: Obsidian Base maken`** (CLI: `gitpoulpi hosting issue base`) schrijft eenmalig `issues.base` naast de
notities: een *tabel*weergave en een *kanban*weergave gegroepeerd op `status`, in de volgorde van `statuses`. Open het in Obsidian om
dezelfde issues daar als bord te zien. GitPoulpi bewerkt nooit een bestaand `.base`-bestand (het jouwe of een eerder gemaakt): het
weigert en zegt dat. Het formaat volgt de Bases-syntaxis van Obsidian
(<https://obsidian.md/help/bases/syntax>); de kanbanweergave vraagt een recente Obsidian.

## CLI en MCP

Dezelfde opdrachten als bij elke issuebron werken erop: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Vier meer zijn voor notities:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` en `hosting issue base`.
Zie de [CLI-naslag](cli-reference.md).

## Grenzen

- Een nieuwe **mijlpaal** kan niet in de kiezer worden getypt (kies er een die de map al gebruikt), maar `milestone:` in een notitie kan
  alles zijn en verschijnt zodra ze is opgeslagen.
- Alleen de notities direct in de map zijn issues (geen submappen).
- Er is geen verwijderen: verwijder de notitie in je bestandsbeheer of Obsidian (de prullenbak daarvan is van jou).
