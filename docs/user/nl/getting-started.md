[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · **Nederlands** · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Aan de slag

Deze pagina brengt je van een verse installatie naar je eerste commit. De
[rondleiding door de interface](interface-tour.md) benoemt elk deel van het
venster. [Instellingen](settings.md) somt elke optie op.

## 1. Installeren en starten

Volg [Installatie](installation.md). Open daarna een terminal in een
Git-repository en voer uit:

```
gitpoulpi
```

GitPoulpi opent de repository
die de huidige map bevat. Buiten een repository opent het het tabblad **Start**,
waar je een repository kunt zoeken, klonen of initialiseren (zie
[stap 7](#7-repositories-openen-klonen-en-initialiseren)).

De pictogrammen vereisen een Nerd Font of een lettertype met emoji. Zie je
vierkantjes, kijk dan bij [Probleemoplossing](troubleshooting.md).

![Het hoofdvenster: links de zijbalk Refs, in het midden de Log-grafiek en rechts de lokale wijzigingen met het commit-formulier](../images/hero.png)

## 2. Drie toetsen om de rest te leren

| Toets | Opent |
|---|---|
| `?` | De **help**: elke toets en elk muisgebaar. Typ om te filteren. `Ctrl+D` opent deze documentatie in je browser. |
| `Ctrl+P` | Het **opdrachtenpalet**: typ om een opdracht te vinden, `Enter` voert hem uit. Elke rij toont de sneltoets. |
| `Ctrl+,` | De **Instellingen**. Het tandwiel in de titelbalk, links van de ×, opent ze ook. |

Je kunt alles met het toetsenbord of met de muis doen. Het palet bereikt elke
opdracht. Zie [Sneltoetsen](keybindings.md) en
[Muis en gebaren](mouse-and-gestures.md).

![Het opdrachtenpalet gefilterd op het woord stage, met staging-opdrachten en hun sneltoetsen](../images/palette.png)

## 3. Bekijk de geschiedenis

Het midden van het venster is de **Log**: één rij per commit, de nieuwste
eerst. Hij toont de branches en tags als gekleurde pillen, de grafiek, het
bericht, de auteur, de datum en de korte hash.

- Klik op een commit. De details en gewijzigde bestanden verschijnen rechts.
- Klik op een gewijzigd bestand. De diff opent in het midden. `Esc` sluit hem.
- `j` / `k` of de pijltoetsen verplaatsen de selectie. `f` zoekt in de commits.
- Rechtsklik op een commit voor het menu.

![De Log met een merge-commit geselecteerd: rechts het bericht, de auteur en de twee bestanden die hij toevoegt](../images/log-graph.png)

## 4. Maak een wijziging en stage hem

Bewerk een bestand in je editor. De **bovenste rij van de Log** toont nu de
lokale wijzigingen: een gestippelde rij met het aantal gewijzigde bestanden. Klik
erop. Het rechterpaneel toont **Unstaged bestanden** en **Staged bestanden**.

- Beweeg over een bestand en klik op de knop **Bestand stagen**. Of klik op
  **Alle wijzigingen stagen**.
- Klik op een bestandsnaam om de diff te openen. Daar kun je één **hunk**
  stagen (**Hunk stagen of unstagen**) of één **regel**.

Zie [Stagen en committen](staging-and-committing.md).

![De diff van een gewijzigd bestand, met knoppen voor stagen en verwerpen boven elke hunk](../images/diff.png)

## 5. Commit

Typ een samenvatting, en eventueel een beschrijving, in het commit-formulier
rechtsonder. Klik dan op de groene knop **Wijzigingen in N bestanden committen**
of druk op `Ctrl+Enter` (of `Ctrl+G Enter`, vanaf elke plek). Vink **Vorige commit amenden** aan om je wijzigingen
aan de laatste commit toe te voegen.

Als je [AI](ai.md) hebt ingeschakeld, schrijft de knop `✦` naast de
samenvatting een bericht op basis van de gestagede wijzigingen. GitPoulpi commit
nooit totdat je op **Wijzigingen in N bestanden committen** klikt.

## 6. Deel je werk

De werkbalk heeft knoppen **Pull** en **Push**, elk met een `▾`-lijst. Vanaf het
toetsenbord druk je op `Ctrl+G` en dan op een letter: `Ctrl+G p` doet een pull,
`Ctrl+G Shift+P` een push. Na `Ctrl+G` toont de statusbalk de letters.

GitPoulpi gebruikt de authenticatie die je `git` al gebruikt (credential helper,
ssh-agent). Daarvoor is geen eigen account nodig. Om pull requests te zien en te
maken verbind je eenmalig een hostingaccount. Zie
[Hosting en issuetrackers](hosting-integration.md).

## 7. Repositories openen, klonen en initialiseren

Klik op het huisje aan het linkeruiteinde van de tabbalk, of druk op
`Ctrl+O`. Het tabblad **Start** toont je geopende, favoriete en recente
repositories, met de knoppen **Bladeren**, **Klonen** en **Initialiseren**.
Elke repository opent in een eigen tabblad. `Ctrl+W` sluit het huidige
tabblad.

![Het tabblad Start met de geopende en recente repositories en de knoppen Bladeren, Klonen en Initialiseren](../images/home.png)

**Klonen** vraagt om een URL en een map, of laat je een repository kiezen uit een verbonden account (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; zie [Klonen vanaf een verbonden account](hosting-integration.md#klonen-vanaf-een-verbonden-account)). **Initialiseren** maakt van een map
een nieuwe repository.
In alle drie de dialogen (en in de mapkiezer die ze openen) toont `Ctrl+N` een badge op elk veld en elke knop: druk op het teken om de cursor in het veld te zetten of de knop in te drukken.

| ![Het kloondialoogvenster: waarheen klonen, de URL, de mapnaam en de knoppen Annuleren en Klonen](../images/clone.png) | ![Het dialoogvenster Een repository initialiseren: de map, de naam en de eerste branch](../images/init.png) |
|---|---|

### Gedeeltelijke checkout

Een grote repository hoeft niet helemaal op schijf te belanden. Vink in het dialoogvenster Klonen **Gedeeltelijke checkout (alleen sommige mappen)** aan en typ de gewenste mappen, gescheiden door spaties of komma's (`src docs/guide`). Alleen die mappen en de bestanden in de hoofdmap van de repository worden uitgecheckt; de volledige geschiedenis wordt toch gedownload, dus de Log, blame en branches werken zoals gewoonlijk. Zolang het vakje is aangevinkt is minstens één map nodig.

![Het dialoogvenster Gedeeltelijke checkout met twee getypte mappen, en de knoppen Annuleren, Uitschakelen en Toepassen](../images/sparse-checkout.png)

Elke repository kan later alsnog worden omgeschakeld: open het opdrachtenpalet (`Ctrl+P`) en voer **Repository: Gedeeltelijke checkout...** uit, of gebruik **Gedeeltelijke checkout...** in het menu van het repositorytabblad. Het dialoogvenster toont de nu uitgecheckte mappen in één veld: voeg namen toe of verwijder ze en druk op **Toepassen** (`Enter`), of op **Uitschakelen (volledige checkout)** om elk bestand terug te brengen. Zolang een werkmap gedeeltelijk is, toont de statusbalk een klein label `deels`. Dezelfde bediening is ook een pagina van *Instellingen > Gedeeltelijke checkout* (een lijst met paden, één per regel, met de knoppen **Starten**, **Stoppen** en **Vernieuwen**).

Opmerkingen:

- Het is de cone-modus van git (mappen, geen patronen); `*`, `..` en absolute paden worden geweigerd.
- Het wijzigen van de set wordt niet door Ongedaan maken vastgelegd: er gaat niets verloren, en **Uitschakelen** herstelt elk bestand. Git kan weigeren een map te laten vallen die niet-gecommitte wijzigingen bevat (de fout verschijnt op de statusbalk); commit of stash ze eerst.
- Hetzelfde vanuit scripts en agents: `gitpoulpi clone <url> --sparse src docs` en `gitpoulpi sparse-checkout status | set | add | disable` (zie de [CLI-naslag](cli-reference.md)); de MCP-server heeft dezelfde tools.

## Hoe verder

- [Rondleiding door de interface](interface-tour.md): de naam van alles wat je
  ziet.
- [Werken met de Log](log-and-history.md) en
  [Stagen en committen](staging-and-committing.md): het dagelijkse werk.
- [Thema's](themes.md) en [Instellingen](settings.md).
- [AI-ondersteuning](ai.md) en [Hosting en issuetrackers](hosting-integration.md):
  de twee optionele integraties.
- [CLI-naslag](cli-reference.md) en [MCP en agents](mcp-and-agents.md): dezelfde
  handelingen vanuit scripts en agents.
- [Probleemoplossing](troubleshooting.md).
