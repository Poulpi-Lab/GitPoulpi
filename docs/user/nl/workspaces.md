[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · **Nederlands** · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Werkruimtes

Een **werkruimte** is een benoemde groep repositories: die waaraan je samen
werkt (een front-end, de bijbehorende API, de documentatie). Ze onthoudt welke
daarvan als tabblad openstaan en kan een **standaard commitprofiel** dragen,
zodat commits in die repositories de juiste identiteit gebruiken zonder dat je
met de hand wisselt. Een repository kan in meerdere werkruimtes zitten, en in
geen enkele zitten is prima: "Geen werkruimte" is de gewone tabbladenlijst.

Werkruimtes staan in `workspaces.toml`, naast `config.toml`, gedeeld door elke
sessie, het venster, de CLI en MCP (`gitpoulpi workspace ...`, zie de
[CLI-referentie](cli-reference.md)). Een werkruimte die vanuit een shell of met `gitpoulpi workspace ...` wordt toegevoegd, hernoemd of
verwijderd terwijl het venster open is, verschijnt vanzelf binnen ongeveer een seconde (het tabblad Werkruimtes, de chip van de
tabbalk en zijn lijst), zonder toetsaanslag. Een werkruimte sluiten of verwijderen raakt
nooit een repository op schijf aan. Wijzigingen aan werkruimtes zijn
instellingen, net als favorieten en profielen: ze maken geen deel uit van
[Ongedaan maken en opnieuw](undo-redo.md), en Verwijderen vraagt eerst
bevestiging.

## De werkruimte-chip

Zodra je een werkruimte hebt, verschijnt er een chip op de tabbalk, direct na de
`⌂`: een gekleurde stip en de naam van de actieve werkruimte, of "Geen
werkruimte" (gedimd). Op een smalle balk krimpt hij tot de stip, en op een heel
smalle verdwijnt hij.

![De werkruimte-keuzelijst boven de tabbalk, met Geen werkruimte, Orbit (aangevinkt), Side projects, Archive en Werkruimtes beheren, met de chip Orbit en zijn profiel "Work - uit werkruimte Orbit" in de werkbalk](../images/workspace-switch.png)

- **Klik erop**, druk op `Alt+W`, of voer `Werkruimte: Wisselen...` uit het
  palet uit om een keuzelijst te openen: je werkruimtes, "Geen werkruimte" en
  "Werkruimtes beheren..." (dat het tabblad Werkruimtes
  opent).
- **Wisselen verwisselt de tabbladen.** De tabbladen die je open had blijven
  bewaard voor de werkruimte die je verlaat, en die van het doel komen terug
  (de eerste keer de repositories ervan die bestaan). Wissel terug en alles
  staat waar je het liet. Het herstart het venster zoals bij een
  tabbladwissel; de daemon blijft draaien.
- **De werkruimte wordt onthouden**: de volgende start komt erin terug.
- **Het laatste tabblad sluiten** van een werkruimte houdt de werkruimte actief
  en toont het tabblad Start.
- **De `+` (nieuw tabblad) terwijl een werkruimte actief is** voegt de gekozen
  repository toe aan de tabbladenlijst en vraagt niets. Om hem in de
  werkruimte zelf te zetten, gebruik je "Aan werkruimte toevoegen" in het
  rechtsklikmenu van het tabblad (of `Werkruimte: Tabblad aan werkruimte
  toevoegen`); "Uit werkruimte verwijderen" haalt hem er weer uit en laat het
  tabblad open.

Zonder werkruimte is er geen chip: maak er eerst een via het tabblad Werkruimtes of
het palet.

## Het tabblad Werkruimtes

Open het via het menu van de `+`, met `Alt+J` of met `Werkruimte: Beheren...` in het palet; de `x` (of `Ctrl+W`) sluit het, en het blijft na een herstart open tot je het sluit.
Elke werkruimte is een rij met haar kleurstip, naam, hoeveel repositories ze
bevat (en hoeveel ontbreken), haar standaardprofiel, en "actief" als het venster
erin zit. `Right`, `Space` of een klik klapt haar uit tot haar repositories.

![Het tabblad Werkruimtes: Orbit uitgeklapt tot zijn twee repositories, Side projects, en Archive met de markering "1 ontbreekt"](../images/workspaces.png)

| Per werkruimte | Toets | |
|---|---|---|
| Openen | `Enter` | Maakt haar actief en opent haar tabbladen. |
| Bewerken... | `e` | Het formulier hieronder. |
| Verwijderen... | `x` | Vraagt eerst. Repositories worden niet aangeraakt. |
| Alles fetchen | `Shift+F` | Eén repository na de andere, voortgang op de statusbalk, mislukkingen aan het eind opgesomd. |
| Alles pullen | `Shift+P` | Hetzelfde, alleen fast-forward: stopt nooit op een merge. |
| Ontbrekende repositories klonen | `m` | Kloont elke ontbrekende repository met een vastgelegde remote. |
| Nieuwe werkruimte... | `n` | |
| Van open tabbladen... | `a` | Een nieuwe werkruimte met je open tabbladen aangevinkt. |

Een **ontbrekende** repository (haar map is weg) blijft in de lijst, gemarkeerd,
met **Zoeken...** (`l` of `Enter`: kies waar ze nu staat), **Klonen** (als haar
`origin`-URL bij het toevoegen is vastgelegd) en **Verwijderen** (`x`). Elke rij
heeft ook een `⋮`-menu (rechtsklik, `.` of `Shift+F10`) met dezelfde acties, en
elke actie staat in het palet onder `Werkruimte:`.

## Het formulier Nieuw / Bewerken

![Het formulier Werkruimte bewerken: naam, kleur, beschrijving, standaardprofiel en de checklist van repositories](../images/workspace-edit.png)

- **Naam**: verplicht, uniek.
- **Kleur**: een keuzelijst met themakleuren; de stip op de chip en de rijen.
- **Beschrijving**: vrije tekst.
- **Profiel**: een keuzelijst met je `[[profiles]]`, of "Geen". Optioneel.
- **✦ AI-instructies**: alleen met AI aan, zie [hieronder](#ai-instructies-van-een-werkruimte).
- **Repositories**: een checklist van elke repository die GitPoulpi kent (open
  tabbladen, favorieten, recent, die van de werkruimte zelf, ontbrekende
  inbegrepen). `Space` vinkt een rij aan; **Map toevoegen...** (`Ctrl+B`) kiest
  een repository van schijf. De origin-URL van elke nieuwe repository wordt uit
  de repository gelezen en vastgelegd (zonder inloggegevens); Klonen gebruikt
  die later.

## AI-instructies van een werkruimte

Met AI aan (*Instellingen > AI*) heeft het formulier een rij **✦ AI-instructies** met een knop **Bewerken...** (`Enter` op
de rij). Die opent een kleine editor voor wat de AI te horen krijgt in **elke repository van de werkruimte**: kies *Elke AI-
actie* of één actie (commit-bericht, uitleg, stash-bericht, pull request, commit-composer, conflictoplossing,
codereview) in de lijst **Voor**, kies **Combineren** - *Toevoegen aan globaal* (verstuurd na de globale instructies) of
*Globaal vervangen* (verstuurd in plaats daarvan) - en schrijf de tekst. **Toepassen** houdt de hele set, **Opslaan** in het formulier
schrijft ze met de werkruimte naar `workspaces.toml`; **Annuleren** laat vallen wat je typte. Met AI uit is de rij
verborgen en blijven de instructies van de werkruimte zoals ze zijn.

De woorden van de werkruimte staan tussen de globale en de eigen instructies van de repository (zie
[AI-instructies](ai.md#instructies) voor de volgorde en voor wat *vervangen* doet). Een repository die bij
meerdere werkruimtes hoort, gebruikt de actieve werkruimte van het venster als die haar bevat, anders de eerste die dat doet. Vanuit een
script: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` toont ze als `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # elke AI-actie in deze werkruimte
text = "Use the Acme glossary."
mode = "add"                       # of "replace"

[workspace.ai_overrides.code_review]   # één actie
text = "Flag missing tests."
mode = "add"
```

## Het standaardprofiel

Commits worden ondertekend door het eerste van deze dat van toepassing is:

1. het profiel dat je **met de hand** voor die repository koos (het
   profielveld van de werkbalk),
2. het profiel van de **actieve werkruimte**, als de repository er lid van is,
3. `general.active_profile`.

Het profielveld op de werkbalk zegt waar het vandaan komt: "Work - uit
werkruimte Acme". Het veld verschijnt ook als je maar één profiel hebt, zolang
het standaardprofiel van een werkruimte geldt. Een werkruimte waarvan het
profiel niet meer bestaat houdt gewoon op te gelden. In de CLI lost
`commit --workspace <id>` (of `GITPOULPI_WORKSPACE`) op dezelfde manier op.

## Toetsen

| Toets | Actie |
|---|---|
| `Alt+W` | De werkruimte-keuzelijst openen |
| `Alt+J` | Tabblad Werkruimtes openen |
| `n`, `a` | Nieuwe werkruimte, nieuw van de open tabbladen |
| `e` | De werkruimte bewerken |
| `Shift+F`, `Shift+P`, `m`, `l` | Alles fetchen, alles pullen, ontbrekende klonen, zoeken |

Alle toetsen zijn te herbinden in Instellingen > Toetsenbord & muis; de volledige lijst
staat in [Sneltoetsen](keybindings.md).
