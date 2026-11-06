[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · **Nederlands** · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# AI-ondersteuning

GitPoulpi heeft nooit een account nodig, en **AI staat standaard uit**. Als AI aan
staat, kan het commit- en stash-berichten schrijven, wijzigingen uitleggen, beschrijvingen
van pull requests schrijven, oplossingen voor merge-conflicten voorstellen en je
lokale wijzigingen over meerdere commits verdelen. Het model is van jou: een API-sleutel die
je zelf bezit, een lokale server, of een agent-CLI die al is geïnstalleerd (zoals
`claude -p`).

Elke AI-bediening toont de paarse `✦`. Zolang AI uit staat:

- Wordt er nergens iets naartoe gestuurd.
- Wordt er geen AI-bediening getoond: geen `✦`-knop, geen `AI:`-opdracht in het palet,
  geen AI-menurij en geen AI-toetsaanwijzing. De AI-toetsen doen niets.
- Toont de AI-pagina van de repository alleen "AI staat uit (Instellingen > AI)".
- Toont de MCP-server de `ai_*`-tools niet. De CLI-opdrachten `gitpoulpi ai ...`
  weigeren met dezelfde melding.

**AI inschakelen** werkt meteen. Je hoeft niet te herstarten.

Er zijn twee manieren om AI met GitPoulpi te gebruiken:

| | Ingebouwde AI (deze pagina) | Een externe agent |
|---|---|---|
| Wat | GitPoulpi vraagt een model om een commit-bericht, een uitleg, ... | Claude Code, Cursor, Codex... besturen GitPoulpi als tool |
| Instellen in | Instellingen > AI (`[ai]`) | de MCP-configuratie van de agent, limieten in `[mcp]` |
| Handleiding | hieronder | [MCP en agents](mcp-and-agents.md) |

## De AI configureren

1. `Ctrl+,` > **AI** > vink **AI inschakelen** aan.
2. De profielen (verbindingen) staan als lijst op het tabblad **Profielen (verbindingen)**, links op de pagina (op een smalle terminal
   erboven): kies er een om het rechts te bewerken (`Enter` gaat naar de rijen ervan;
   **AI: Profiel bewerken...** in het palet springt naar de lijst). **+ Toevoegen** maakt er
   nog een, **Dupliceren** kopieert het geselecteerde profiel (zonder zijn sleutel) en
   **Verwijderen** vraagt eerst om bevestiging. Kies de **Provider** en vul de velden in:

   | Provider | Velden | Standaarden |
   |---|---|---|
   | **Lokale agent** (geen sleutel, geen eigen account) | **Agenttype** (welke agent het is), **Opdracht** (het exacte programma: op je `PATH`, een alias of wrapperscript, of een volledig pad), **Argumenten** die het niet-interactief maken, en optionele **Omgevingsvariabelen**. De prompt gaat naar zijn stdin, het antwoord wordt van stdout gelezen. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **API-sleutel**, optioneel **Model** en **Basis-URL** | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **API-sleutel**, Model, Basis-URL | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **API-sleutel**, Model, Basis-URL | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **OpenAI-compatibel** (Ollama, LM Studio, vLLM, een bedrijfsgateway) | **Basis-URL** (`.../v1`), **Model**, optionele sleutel | `http://localhost:11434/v1` (Ollama; LM Studio is `http://localhost:1234/v1`), `llama3` |

   **Presets** (je kunt altijd een andere waarde typen):

   ![De keuzelijst Agenttype van een profiel voor een lokale agent, met Claude Code, OpenAI Codex CLI, Gemini CLI, opencode en de andere agent-CLI's](../images/settings-ai-presets.png)

   - **Lokale agent > Agenttype** is een keuzelijst van de gangbare agent-CLI's, de
     meest gebruikte eerst: Claude Code, OpenAI Codex CLI, Gemini CLI, opencode, Cursor
     Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose, Crush, Amp, of
     *Overig (geen suggesties)*. Het type staat **los van de opdracht**: het
     bepaalt alleen welke suggesties je krijgt. Als je er een kiest, wordt **Opdracht**
     (zolang die nog de standaard van het vorige type is) en **Argumenten** vooraf
     ingevuld met de headless aanroep (`claude -p`, `codex exec -`, `gemini -p ...`, ...).
     **Opdracht** is vrije tekst: schrijf `/opt/bin/work-claude`, een wrapperscript
     of het doel van een alias, en het type, met zijn opties, modellijst en
     variabelen, blijft. Als **Agenttype** leeg is, leidt GitPoulpi het af uit
     de opdracht.
   - **Argumentopties** voegt opties toe of verwijdert ze die passen bij het eenmalige
     gebruik van GitPoulpi, elk met een reden van één regel: geen tools, geen opgeslagen
     sessie, platte-tekstuitvoer, lage redeneerinspanning, alleen-lezenmodus, geen
     toestemmingsvragen, geen updatecontrole of analytics. De lijst verschilt per agent
     omdat de agents verschillen (slechts sommige kunnen tools vanaf de opdrachtregel
     uitzetten; bij opencode is dat een omgevingsvariabele). **Model** toont de modellen
     van die agent en schrijft de juiste vlag (`--model haiku` voor Claude Code, `-m gpt-6-luna`
     voor Codex, `--model provider/model` voor opencode); *(standaard - geen vlag)*
     verwijdert hem. Amp heeft geen modelvlag. `aider` leest zijn prompt van
     `/dev/stdin`, dus het werkt alleen op Linux en macOS.
   - **Omgevingsvariabelen** gelden alleen voor het proces van die agent, nooit voor
     GitPoulpi of andere profielen. **Omgevingsvariabele toevoegen** biedt de
     gedocumenteerde variabelen die voor deze agent van belang zijn, elk met wat het doet
     (telemetrie en updatecontroles uitzetten, inspanningsniveau, configuratiemap,
     API-sleutel, basis-URL of lokale host, ...); je kunt ook `NAME` of
     `NAME=value` typen. Elke variabele heeft dan een eigen rij **Env NAME**: de
     waarde staat als knop (druk erop om een nieuwe te typen; ze wordt opgeslagen als je het
     veld verlaat) en een knop **[x]** verwijdert de variabele. Een waarde mag `${env:NAME}` zijn om een van de eigen variabelen
     van GitPoulpi te lenen (handig voor API-sleutels, zodat ze niet in
     `config.toml` worden geschreven). Waarden die op geheimen lijken (namen die eindigen op
     `_KEY`, `_TOKEN`, `_SECRET`, ... of bekende sleutelvoorvoegsels) verschijnen als stippen en
     worden helemaal opnieuw getypt. Waarden worden nooit gelogd.
   - **HTTP-providers > Providerpreset** vult de basis-URL en een model in voor
     OpenAI, Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek, Mistral, Groq,
     Together, Fireworks, Cerebras, Perplexity, Moonshot, Qwen (DashScope), Zhipu
     (GLM) en de lokale servers Ollama, LM Studio, llama.cpp, vLLM, Jan en
     LocalAI. **Model** is een keuzelijst met de modellen van die provider, vrije
     tekst toegestaan.

3. Druk op **Verbinding testen**: *Laatste test* toont `verbonden: het model antwoordde`
   of `mislukt: <reden>`.
4. Kies bij meerdere profielen onderaan het **Standaardprofiel**; elke
   actie kan nog steeds een eigen profiel noemen.

![Het blok Profielen van de AI-pagina voor een lokale agent: Agenttype (Claude Code), Opdracht, Argumenten, Model, twee Env-rijen, Omgevingsvariabele toevoegen en verwijderen, Verbinding testen en Standaardprofiel](../images/settings-ai-agent.png)

**API-sleutels** voer je in met `Enter` op de rij **API-sleutel**; ze worden opgeslagen in de
credential helper van je systeem (Git Credential Manager, libsecret / `pass`, Keychain)
onder `ai:<profile id>`, nooit in `config.toml`. Zonder helper zegt GitPoulpi dat
in plaats van te doen alsof het hem bewaart. *Instellingen > SSH & inloggegevens > Alle
gebruikersnamen en wachtwoorden vergeten* verwijdert ze.

Hetzelfde in `config.toml` (al het andere staat in de [configuratienaslag](config-reference.md#ai-ingebouwde-ai-standaard-uit)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # vrij: een alias, een wrapperscript, een absoluut pad
harness = "claude"        # welke agent het is; leeg = afgeleid uit `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # alleen voor het proces van deze agent; ${env:NAME} leent die van ons
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Uitvoermodus: headless of virtuele terminal

Een lokaal agentprofiel draait zijn agent op twee manieren, te kiezen met
**Uitvoermodus** (getoond voor Claude Code, OpenAI Codex CLI en Gemini CLI):

- **Headless (niet-interactief)**, de standaard: de eenmalige modus van de
  agent (`claude -p`), prompt op stdin, antwoord op stdout.
- **Interactief (virtuele terminal)**: GitPoulpi start het eigen scherm van de
  agent in een verborgen terminal, typt het verzoek in zoals een mens dat doet,
  wacht op het antwoord en sluit de agent. Sommige agents factureren of
  beperken hun headless modus anders dan hun interactieve; welke regel voor
  jou geldt, en wat de voorwaarden van de provider over beide zeggen, is aan de
  provider, dus raadpleeg die van hen.

Elke modus houdt zijn eigen **Argumenten**, **Argumentopties** en **Model**:
van modus wisselen verliest niets, en de interactieve modus biedt alleen de
opties die zinvol zijn zonder de headless vlag. Twee rijen gelden voor beide
modi:

- **Werkmap**: *Speciale map* is een lege map die van GitPoulpi is, dus de
  agent ziet geen project en vraagt je één keer om hem te vertrouwen. *Huidige
  repository* geeft de agent de eigen context van het project (bijvoorbeeld
  zijn `CLAUDE.md`), en hij vraagt één keer per repository om vertrouwen.
  *Standaard* is de speciale map in interactieve modus en, in headless modus,
  waar GitPoulpi draait, zoals voorheen.
- **Time-out (seconden)**: hoe lang de agent erover mag doen voordat GitPoulpi
  opgeeft en hem stopt. Leeg betekent 120 seconden headless en 180 interactief.

**Vragen bij de eerste start beantwoord je zelf.** Een agent die nog nooit in
een map heeft gedraaid, vraagt eerst iets (deze map vertrouwen, inloggen, een
thema kiezen). GitPoulpi beantwoordt nooit iets voor je: het verzoek stopt met
een bericht, en **De agent openen...** (op het profiel, of *AI: De agent
openen* in het commandopalet) opent de agent één keer, in een zichtbare
terminal, in de werkmap die hij zal gebruiken. Beantwoord de vraag, sluit de
agent en probeer het opnieuw; het antwoord wordt door de agent onthouden.

Het antwoord wordt voor Claude Code gelezen uit het sessietranscript van de
agent (de exacte tekst, daarna verwijderd) en voor Codex en Gemini van het
scherm tussen twee markeringsregels waar het verzoek om vraagt. Er draaien
hooguit twee virtuele-terminalverzoeken tegelijk, en Esc tijdens een verzoek
stopt de agent.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # de headless set, behouden
run_mode = "terminal"               # headless (standaard) | terminal
terminal_args = ["--effort", "low"] # de interactieve set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 of afwezig: 120 headless, 180 terminal
```

## Instructies

Tekst die je toevoegt aan wat het model te horen krijgt. De ingebouwde prompt van elke
actie (die het formaat van het antwoord vastlegt) komt altijd eerst en kan niet worden
bewerkt; daarna vier niveaus, van het meest algemene naar het meest specifieke, elk alleen
als het niet leeg is:

1. **Globale instructies** (elke actie). De chips *Snel invoegen* van de editor voegen veelgebruikte zinnen in:
   *Antwoord in `<taal>`*, *Antwoorden beknopt houden*, *Gebiedende wijs gebruiken*,
   *Technische toon*, *Informele toon*.
2. **Werkruimte-instructies** (elke actie, elke repository van de
   [werkruimte](workspaces.md#ai-instructies-van-een-werkruimte)): voor alle acties,
   en optioneel per actie. Bewerkt in het formulier Bewerken van de werkruimte.
3. **Repository-instructies** (elke actie, alleen deze repository), met
   *Combineren met globaal*: **Aan globaal toevoegen** (erna) of **Globaal vervangen**
   (in plaats ervan).
4. **Instructies voor de actie** (*Commit-berichten genereren*, *Wijzigingen
   uitleggen*, *Stash-berichten genereren*, *Pull request*, *Commit-composer*,
   *Conflictoplossing*, *Codereview*), elk
   blok met zijn eigen chips in zijn editor (bijv. *Conventional-commits-formaat*, *Limiet van 50
   tekens*) en zijn eigen keuzelijst **Profiel** (`Standaard (<naam>)` volgt het
   standaardprofiel).

`Enter` op een rij *Eigen instructies* opent een editor met meerdere regels (`Ctrl+S` slaat op,
`Esc` annuleert). Boven de tekst staat een rij *Snel invoegen* met de chips van die sectie: klik op een chip om zijn zin bij de cursor in te voegen. Met het toetsenbord gaat `Tab` naar de rij, kiezen `Left`/`Right` een chip, voegt `Enter` of `Space` hem in en keer je terug naar de tekst (`Tab` of `Esc` keert terug zonder in te voegen). Een chip waarvan de zin al in de instructies staat, is gedimd en hem opnieuw invoegen doet niets. De tekst is een volwaardige editor: een klik plaatst de cursor, slepen of `Shift`+pijltjes selecteren, `Ctrl+A` selecteert alles, `Ctrl+C` / `Ctrl+X` kopiëren en knippen de selectie, `Ctrl`+pijltjes springen per woord, en `Home` / `End`, `PageUp` / `PageDown` en het muiswiel bewegen door lange tekst. De pagina zelf toont van elke instructie alleen een voorbeeld.

![De AI-pagina: globale instructies, daarna de blokken Commit-berichten genereren en Wijzigingen uitleggen met een profielkeuzelijst en eigen instructies](../images/settings-ai.png)

**Per repository**: *Instellingen > Repositoryspecifieke instellingen > AI* (terwijl een
repository open is) bevat de repository-instructies en, per actie, een profiel
en instructies die de globale overschrijven. Een leeg veld gebruikt de globale
instelling (daarin grijs getoond); **Terugzetten naar globaal** verwijdert een overschrijving. Opgeslagen in
het `.gitpoulpi/repo-state.toml` van de repository, en ook door de CLI toegepast.

![De AI-pagina van de repository: Repository-instructies met de keuzelijst Combineren met globaal, daarna een profiel en instructies voor elke actie](../images/settings-repo-ai.png)

### Hoe de niveaus combineren

De verstuurde volgorde is: ingebouwde prompt, **globaal**, **werkruimte**, **repository**,
daarna de eigen instructies van de actie. De modus van een niveau bepaalt wat er met de
niveaus erboven gebeurt:

- **Toevoegen** (de standaard): de tekst van het niveau komt na de meer algemene.
- **Vervangen**: de tekst van het niveau wordt gebruikt *in plaats van* elk meer algemeen niveau
  (globaal, en de werkruimte als de repository vervangt). Een niveau zonder tekst
  vervangt nooit iets.

De eigen instructies van de actie volgen hetzelfde idee: de globale voor die
actie, daarna die van de werkruimte voor die actie (erna toegevoegd, of ze vervangend),
en een eigen overschrijving van de repository voor de actie wint van beide. De
gebruikte werkruimte is de actieve werkruimte van het venster als die de
repository bevat, anders de eerste werkruimte die haar bevat; op de opdrachtregel
is het die in `GITPOULPI_WORKSPACE`, anders de eerste.

Om precies te zien wat er wordt verstuurd, niveau voor niveau, voer je
`gitpoulpi ai prompt --action <actie> --repo .` uit (`base`, `global`, `workspace`,
`repository`, `task` en het uiteindelijke `system_prompt`). Het roept geen model aan en
werkt met AI uitgeschakeld. Stel de instructies van een werkruimte in vanuit een script met
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (of `--clear`).

### Promptvariabelen

Elke instructietekst (globaal, werkruimte, repository, per actie) mag `${naam}`-variabelen bevatten.
Ze worden vervangen door hun waarde vlak voordat de prompt wordt
verstuurd (de opgeslagen tekst houdt de `${...}`). Een naam die niet in de tabel staat,
wordt verstuurd zoals geschreven en de editor waarschuwt ervoor; schrijf `$${` voor een letterlijke `${`.

| Variabele | Wordt vervangen door | Voorbeeld |
|---|---|---|
| `${uilang}` | De UI-taal, met zijn Engelse naam | `French` |
| `${repo}` | De mapnaam van de repository | `gitpoulpi` |
| `${branch}` | De huidige branch | `main` |
| `${ref}` / `${commit}` | Korte sha van de commit waar de actie over gaat (*Uitleggen*), anders HEAD | `4c0443d` |
| `${commit_subject}` | Onderwerpregel van die commit | `feat: add search` |
| `${date}` | Lokale datum, ISO | `2026-10-03` |
| `${time}` | Lokale tijd, `HH:MM` | `14:05` |
| `${datetime}` | Lokale datum en tijd | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | De standaard-remote (`origin`, anders de eerste) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (een andere host) of `none` | `github` |
| `${files}` | Aantal bestanden in de diff van de actie | `3` |
| `${action}` | De actie die wordt uitgevoerd | `commit_message` |

In de instructie-editor: typ `${` en een popup toont de passende variabelen
(`Up`/`Down` kiezen, `Enter` of `Tab` voegt in, `Esc` sluit hem); of gebruik de
keuzelijst **Variabelen** linksonder (`Ctrl+T`, of *AI: Promptvariabele invoegen* in het
opdrachtenpalet), die bij de cursor invoegt. Elke regel heeft een
beschrijving, getoond in de keuzelijst en op de regel boven de knoppen terwijl je er
met de muis boven staat of hem markeert.

## Wat het kan

| Actie | In het venster | CLI |
|---|---|---|
| **Commit-bericht** | Stage wijzigingen en druk op de `✦` rechts van het veld Samenvatting, `Ctrl+G i`, of palet `AI: Commit-bericht genereren`. Samenvatting en beschrijving worden voor je ingevuld om te bewerken; er wordt niets gecommit. | `ai commit-message` (drukt af), `commit --ai` (commit) |
| **Wijzigingen uitleggen** | De `[✦]` rechtsboven in het paneel met lokale wijzigingen (`Ctrl+G w`: de geselecteerde commit, anders de lokale wijzigingen), of het menu van een commit *Wijzigingen uitleggen (AI)*. Een popup: pijlen scrollen, `c` kopieert, `Esc` sluit. | `ai explain [rev] [--wip]` |
| **Stash-bericht** | `Ctrl+G m` opent de stash-prompt; `Ctrl+G` erin vraagt het model. Palet `AI: Stash-bericht genereren` doet beide. | `stash push --ai` |
| **Titel en beschrijving van een pull request** | *Titel en beschrijving genereren* in het formulier Pull request maken (de commits en diff van de branch worden lokaal gelezen). | |
| **Oplossing van merge-conflicten** | In de merge-conflicteditor: de werkbalkknop `✦ Voorstel`, het rechtsklikmenu van een conflict *✦ Oplossing voorstellen (AI)*, `Alt+I`, of palet `AI: Oplossing voor merge-conflict voorstellen`. Het voorstel vervangt het gebied van het *huidige conflict* in de uitvoer als één bewerking (als bewerkt gemarkeerd, één ongedaanmaakstap); er wordt niets opgeslagen of gestaged tot je opslaat. | `ai resolve-conflict <pad> [--conflict N]` (drukt af) |
| **Codereview** | In de [review van een pull of merge request](review.md#-ai-conceptreview): de knop `✦ Review opstellen...`, `s`, of palet `Review: ✦ Review opstellen...` (het opent de reviewopties, `Enter` voert uit). Het model leest het verschil, de beschrijving en de bestaande opmerkingen van het verzoek (en standaard de `CLAUDE.md` en `AGENTS.md` van het project: zie [Reviewcontext](#reviewcontext)) en stelt opmerkingen voor; jij accepteert, bewerkt of negeert elke. Er bereikt niets de host voordat je de review indient. | `hosting review suggest <n>` (drukt JSON af, plaatst niets), `ai review-context` (de standaardwaarden) |
| **Een commit splitsen** | Het menu van een commit *Commit splitsen (AI)...*, of palet `AI: Geselecteerde commit splitsen...`. De splitseditor opent met de indeling van het model al ingevuld; zie hieronder. | `ai split-plan <rev>` (drukt af) |
| **Script voor het herschrijven van de geschiedenis** | In de [werkbank voor het herschrijven van de geschiedenis](history-rewrite.md#een-script-schrijven-met-ai): de knop `✦ Schrijven met AI...` van de scripteditor, of palet `Herschrijven: Een script schrijven met AI...`. Een dialoogvenster toont precies wat er wordt verstuurd; het script komt gemarkeerd in de editor en wordt nooit voor je uitgevoerd of toegepast. | `ai rewrite-script "<instructie>" [--sample N]` (drukt af) |
| **Regels voor geschiedenis herschrijven** | In de [werkbank voor het herschrijven van de geschiedenis](history-rewrite.md#regels-maken-met-ai): de knop `✦ Regels maken met AI...`, `Alt+I`, of palet `Herschrijven: Regels maken met AI...`. Een dialoogvenster met meerdere regels toont precies wat er wordt verstuurd; de regels en instellingen komen in de werkbank en er wordt niets voor je als voorbeeld getoond of toegepast. | `ai rewrite-rules "<beschrijving>" [--context]` (drukt af) |
| **Commit-composer** | Het menu van de WIP-rij *Commits samenstellen (AI)...*, `Ctrl+G C`, palet `AI: Commits samenstellen uit lokale wijzigingen...` | |

`Esc` annuleert een lopend verzoek (het proces van een lokale agent wordt beëindigd).

### Oplossing van merge-conflicten

Het model krijgt het pad en de taal van het bestand, de base-, huidige (A) en
inkomende (B) blokken van het conflict met de labels die de editor toont, tot 20 regels
al samengevoegde tekst ervoor en erna, en je instructies (globaal,
repository, en die van *Conflictoplossing* in Instellingen > AI). Het antwoordt met de
opgeloste tekst van dat ene conflict in een codeblok; een antwoord met conflictmarkeringen
of begeleidende tekst in plaats van code wordt één keer teruggestuurd en daarna geweigerd. Een conflict van meer dan
ongeveer 40 KB wordt niet verstuurd. De bediening is verborgen zolang AI uit staat, en een voorstel
slaat niets op, stagt niets en lost niets op: bekijk de uitvoer, bewerk hem en sla op zoals gewoonlijk.

![Het commit-formulier na de knop met de glinstering: een samenvatting en beschrijving geschreven door het model, klaar om te bewerken](../images/ai-commit-message.png)

![De pop-up Lokale wijzigingen uitleggen: een uitleg en een lijst met dingen om dubbel te controleren](../images/ai-explain.png)

### De commit-composer

Het model verdeelt de lokale wijzigingen over meerdere commits (elk gewijzigd bestand in
precies één; een ongeldig plan wordt één keer teruggestuurd en daarna geweigerd). Je bekijkt het plan
voordat er iets wordt gecommit:

![De review van de composer: vier voorgestelde commits met hun bestanden, en de knoppen Reset, Annuleren en 4 commits maken](../images/composer.png)

| Toets | Muis | Doet |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | klik op een rij | beweeg over commits en bestanden |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | de pijlen, slepen en neerzetten | verplaats een commit; verplaats een bestand naar een andere commit |
| `Right` / `Left`, `*` | de pijl van de rij | klap een commit uit / in, allemaal (`ui.composer_files` bepaalt het begin) |
| `e` / `Enter` | het potlood | bewerk het bericht |
| `d` / `Delete` | `drop` / `herstellen` | laat een commit weg (zijn bestanden blijven ongecommit) |
| `n` | `+ eigen commit` | verplaats een bestand naar een eigen commit |
| `r` | `Reset` | terug naar het plan van het model |
| `Ctrl+S` / `a` | `N commits maken` | maak ze aan, op volgorde |
| `Esc` | `Annuleren` | verlaat, niets gedaan |

Bij het aanmaken wordt per commit gestaged en gecommit, na elkaar; de werkmap wordt
nooit aangeraakt. Een bestand dat maar gedeeltelijk is gestaged, wordt als geheel gecommit. Als een stap mislukt, stopt het
en meldt het hoeveel commits zijn gemaakt.

### Een commit splitsen

Het model leest de wijzigingen van de commit, genummerd zoals de [splitseditor](log-and-history.md#een-commit-splitsen) ze nummert,
en stelt meerdere commits voor, elk met een bericht; het kan hunks en losse regels groeperen. Een
antwoord dat een wijziging weglaat, er een twee keer toewijst of iets noemt wat niet bestaat, wordt geweigerd.
In beide gevallen opent de splitseditor: met het voorstel ingevuld of, als het model faalde, met de reden in een
notitie, zodat je handmatig kunt splitsen. Er wordt niets gesplitst tot je op **Splitsen** drukt (`Ctrl+S`);
verplaats eerst wijzigingen, hernoem commits of voeg er een toe als de indeling niet klopt. Het profiel en de
instructies zijn die van *Een commit splitsen* in Instellingen > AI.

### Een script voor het herschrijven van de geschiedenis

Je beschrijft wat de herschrijving moet doen ("verplaats elke commit die 's nachts is gemaakt naar de volgende
ochtend", "vervang mijn oude e-mailadres door het nieuwe, alleen vóór 2024") en het model schrijft een
[Lua-script](history-rewrite.md#lua-scripts) voor de Script-regel, met de header, `rewrite_commit(c, ctx)` en de
hulpfuncties van de sandbox. De daemon compileert het antwoord; als het niet compileert, gaat de fout één keer terug
naar het model. Wat overblijft, staat in de editor: lees het, test het en bekijk dan het voorbeeld. Details en het
dialoogvenster staan in [Geschiedenis herschrijven](history-rewrite.md#een-script-schrijven-met-ai). Het profiel en de
instructies zijn die van *Script voor geschiedenis herschrijven* in Instellingen > AI.

### Regels voor geschiedenis herschrijven

Je beschrijft in je eigen woorden hoe de geschiedenis moet worden herschreven ("mijn oude e-mailadres
a@b.org vervangen door c@d.org", "secrets.txt uit de hele geschiedenis verwijderen") en het model stelt een
bereik, enkele instellingen en een lijst [herschrijfregels](history-rewrite.md#regels-maken-met-ai) voor, in
de eigen woordenschat van de werkbank. **Het bereidt alleen voor**: het antwoord vult de werkbank, er wordt
geen voorbeeld gemaakt en niets toegepast, en een item dat niet bruikbaar is, wordt met de reden vermeld terwijl
de geldige behouden blijven. Een Script-regel wordt nooit op deze manier gemaakt. Desgewenst gaan het bereik, de
namen van branches en tags en de auteurs van de nieuwste commits mee (een vakje, standaard uit, getoond in het
paneel "wat wordt verstuurd"). Het profiel en de instructies zijn die van *Regels voor geschiedenis
herschrijven* in Instellingen > AI.

## Waar de sleutel naartoe gaat

Een API-sleutel (OpenAI, Anthropic, OpenRouter, een OpenAI-compatibele server) is gebonden aan het adres waarvoor hij is opgeslagen: de oorsprong van de basis-URL van het profiel, vastgelegd in `trust.toml` in de privégegevensmap die alleen jij kunt lezen. Als de basis-URL later ergens anders naartoe wijst, wordt het verzoek geweigerd voordat er iets wordt verstuurd en blijft de sleutel waar hij is. Voer `gitpoulpi ai trust <profiel> --accept-origin <url>` uit (of accepteer het in de app) om de sleutel bewust naar het nieuwe adres te verplaatsen.

- Gewone `http://` is toegestaan op deze computer (Ollama, LM Studio op `localhost`); overal elders is `gitpoulpi ai trust <profiel> --allow-insecure-http on` nodig.
- Een zelf gehoste server waarvan het certificaat niet naar een bekende autoriteit leidt, wordt alleen geaccepteerd voor de sleutelvingerafdruk die je vastpint met `--accept-certificate <sha256>`; de TLS-handshake bewijst nog steeds dat de server die sleutel heeft.
- Een profiel dat al een sleutel had voordat deze controle bestond, wordt stilletjes vastgelegd, vanaf zijn huidige basis-URL, de eerste keer dat het wordt gebruikt.
- Een lokale agent (`claude -p` en dergelijke) heeft geen sleutel en geen HTTP, dus er valt niets te binden.

## Reviewcontext

Een codereviewconcept (`✦`, of `hosting review suggest`) leest het verschil, de titel, de beschrijving en de
bestaande opmerkingen van het verzoek. Het kan ook de eigen afspraken van het project krijgen, in een blok
**Projectafspraken** vóór het verschil, en een lokale agent kan in de repository werken. De standaardwaarden zijn:

| Instelling | Standaard | Betekenis |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: de *Werkmap* van het AI-profiel beslist. `yes`: een lokale agent draait voor de review in de repository, wat het profiel ook zegt. `no`: niet daar (een profiel op *Repository* gebruikt in plaats daarvan de speciale map). HTTP-profielen hebben geen werkmap, dus voor hen verandert het niets. |
| `include_claude_md` | aan | Geef het model de `CLAUDE.md` van de repository. |
| `include_agents_md` | aan | Geef het model de `AGENTS.md` van de repository. |
| `extra_file` | geen | Nog een bestand: een pad relatief aan de repositorywortel, een absoluut pad of een link. |

Ze zijn gelaagd zoals de instructies, waarbij de meest specifieke per veld wint: de globale `[ai.review]`-tabel,
dan de eigen waarden van de repository (een veld dat niet is ingesteld erft), dan de keuzes van één run. Een
niet-gekloonde repository bewaart zijn eigen waarden per `host/project` in `remote-reviews.toml`, naast zijn
reviewconcepten.

Een repository kan het globale extra bestand ook **uitzetten**: haar `no_extra_file = true` (het selectievakje **Geen extra bestand** in *Repositoryspecifieke instellingen > AI*, in het dialoogvenster ✦ Reviewopties en de vlag `--no-extra-file` van `ai review-context`) betekent "geen extra bestand voor deze repository", wat `[ai.review]` ook noemt. Een **leeg** veld **Extra bestand** is niet hetzelfde: het erft het globale bestand. Een bestand van de repository en de schakelaar sluiten elkaar uit (de een instellen wist de ander), en de schakelaar wint als een met de hand bewerkt bestand beide bevat. Een oudere `extra_file = ""` van een repository betekent nog steeds geen.

Waar de bestanden vandaan komen:

- `CLAUDE.md`, `AGENTS.md` en een relatief extra bestand worden gelezen uit de **wortel** van de repository op de
  branch waarop het verzoek mikt, nooit uit de werkmap (die op een andere branch kan staan of lokale
  wijzigingen kan hebben). Met een kloon uit de basiscommit van het verzoek; zonder uit de API van de host.
- Een absoluut pad wordt van deze machine gelezen.
- Een link naar een bestand van een codehost (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea en Forgejo `.../src/branch/<branch>/<path>`, of een raw-link) wordt gelezen via het integratieprofiel
  waarvan de host overeenkomt, zodat een privérepository werkt met de token die je al gaf. Een link die geen
  profiel past, of elk ander adres, wordt anoniem opgehaald. Een branchnaam met een schuine streep kan niet
  uit een link worden gelezen: gebruik een pad, of een link op een branch zonder.
- Een ontbrekende `CLAUDE.md` of `AGENTS.md` wordt stil overgeslagen; een ontbrekend extra bestand wordt gemeld
  (`context.problems` in de JSON van de CLI).
- Elk bestand wordt op **8 KB** ingekort en alle bestanden samen op **16 KB**, met een zichtbare
  melding `[CLAUDE.md cut: showing ... of ... bytes]` in wat het model leest.

Draait een lokale agent in de repository, dan zegt de prompt hem dat de uitgecheckte bestanden misschien niet bij de
head van het verzoek passen en dat het verschil leidend is.

In de reviewweergave opent `s` het dialoogvenster **✦ Reviewopties** met deze waarden ingevuld: die van de repository boven
de globale. Wijzig ze alleen voor deze run, of vink *Opslaan als standaard van deze repository* aan om ze te bewaren (zie
[Reviewopties](review.md#reviewopties)). De standaardwaarden kunnen ook worden bewerkt in *Instellingen > AI* (het blok *Codereview*,
vier rijen) en per repository in *Repositoryspecifiek > AI*, waar elke rij *Globaal gebruiken* kan kiezen
en een knop *Terugzetten naar globaal* de eigen waarden van de repository laat vallen; de zoekfunctie van Instellingen vindt ze (`CLAUDE.md
meegeven`, `Extra bestand`). Alles wat het model niet kon krijgen (een ontbrekend extra bestand) of dat is ingekort om te passen, wordt
op de conceptkaart getoond.

Vanaf de opdrachtregel:

```sh
gitpoulpi ai review-context                      # de globale standaardwaarden en wat een review krijgt
gitpoulpi ai review-context --agents-md off      # een globale standaard opslaan
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # weer de globale standaardwaarden volgen
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` heeft geen model nodig en werkt met AI uit. `ai prompt --action code_review` drukt de
effectieve context af als `review_context`. Dezelfde vier instellingen zijn `ai.review.*` in
`config.toml` ([naslag](config-reference.md#ai-ingebouwde-ai-standaard-uit)).

## Privacy

- Er wordt niets verstuurd tot je op een knop drukt of een opdracht uitvoert. Geen AI
  op de achtergrond, geen telemetrie, geen GitPoulpi-account.
- Wat wordt verstuurd: de prompt (ingebouwd, daarna je instructies) en de diff waar je
  om vroeg, afgekapt op ongeveer 60 KB (met een opmerking daarover); voor een pull request
  ook de commit-onderwerpen van de branch.
- Een verzoek om een herschrijfscript verstuurt je instructie en het vaste scriptcontract (de API van de
  scriptsandbox). Auteur, datum en onderwerp van maximaal 20 commits gaan **alleen** mee als je *Een steekproef
  meesturen* in het dialoogvenster aanvinkt (standaard uit; de CLI heeft daarvoor `--sample N` nodig).
  Bestandsinhoud en geheime waarden worden nooit verstuurd.
- Het gaat alleen naar het profiel dat voor die actie is gekozen: die API-host, of de
  stdin van je lokale agentopdracht (dat dan doet wat die agent doet).
- API-sleutels worden door het `gitpoulpi`-proces zelf gebruikt en bereiken nooit de daemon,
  de andere clients van het venster of een MCP-client.
- Gegenereerde tekst wordt nooit uit zichzelf gecommit, gepusht of toegepast.

### Wat uit een verzoek wordt gehaald

Elk verzoek, welke actie het ook verstuurt en welke aanbieder het ook ontvangt (een
gehoste API of een lokale agent), gaat eerst door dezelfde opschoonstap:

- **Geheimen worden gemaskeerd** (`[ai.privacy] redact_secrets`, standaard aan). Een token
  met een bekend voorvoegsel, een privésleutelblok, een JWT, een wachtwoord in een URL en een
  regel `password = waarde` waarvan de waarde willekeurig lijkt, worden vervangen door
  `[REDACTED:<regel>]`. Het is niet uitputtend: het vindt de gangbare vormen (dezelfde
  scanner als `rewrite scan-secrets`).
- **Gevoelige bestanden worden weggelaten** (`[ai.privacy] exclude_sensitive_files`, standaard
  aan). De diff van een bestand dat overeenkomt met `[privacy] sensitive_patterns` (`.env`
  en `.env.*` behalve de bestanden `.example`, `.sample` en `.template`, `*.pem`,
  `*.key`, SSH-privésleutels, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`,
  `*.cer`, `*.der`, `*.kdbx`) maakt geen deel uit van het verzoek. Een gesplitst deel of een
  bestandslijst van de composer die zo'n bestand noemt, wordt op dezelfde manier ingekort, dus een plan
  ervoor kan als onvolledig worden geweigerd: stage of splits dat bestand zelf.

Een van beide uitzetten is een keuze, en Instellingen > AI toont er een rode regel onder zolang
het uit staat.

**Wat wordt verstuurd, voordat het wordt verstuurd.** Voordat een verzoek weggaat, toont de TUI het
dialoogvenster **Wat wordt verstuurd**: de instructies en het verzoek na het opschonen (hetzelfde
paneel als in de dialogen voor het herschrijven van de geschiedenis, scrollbaar, `Alt+W` toont of verbergt het), hoeveel
geheimen zijn vervangen en welke bestanden zijn weggelaten (een regel en een gemaskeerd
voorbeeld zoals `ghp_****abcd`, nooit een waarde), een vakje **Niet meer tonen voor
<actie>**, en **Verzenden** / **Annuleren** (`Esc`, het sluitkruisje en een klik
buiten het venster annuleren; er verlaat niets je computer voordat je Verzenden kiest). Het vakje schrijft
`[ai.sent_preview] <actie> = false`; Instellingen > AI > *Toon wat wordt verstuurd
vóór...* heeft één schakelaar per actie en **Alles opnieuw tonen**. De twee dialogen voor het
herschrijven van de geschiedenis tonen hun verzoek al, dus die hebben geen schakelaar.

**Op de opdrachtregel** bevat elk `ai ...`-antwoord (en `hosting review suggest`)
`privacy: {redactions: N, excluded_files: [...]}`, en `--print-sent`
drukt de opgeschoonde systeem- en gebruikerstekst af, als `{sent: [{system, user}], privacy,
dry_run: true}`, zonder het model aan te roepen (het vereist `ai.enabled = true`, geen sleutel
en geen netwerk).

**Een grens.** Een lokale agent waarvan de werkmap de repository is
(`workdir = repository`, of *De agent in de repository laten draaien* voor een review) kan
de bestanden zelf lezen: het opschonen dekt wat GitPoulpi hem stuurt, niet wat hij
zelf opent. Gebruik de werkmap `dedicated` (*Speciale map*) om hem buiten de
repository te houden.

## Probleemoplossing

| Symptoom | Oplossing |
|---|---|
| Nergens een `✦` | AI staat uit: *Instellingen > AI > AI inschakelen*. |
| De `✦` is grijs | Er is niets gestaged (commit-bericht). |
| *Verbinding testen*: command not found | De **Opdracht** van de lokale agent staat niet op de `PATH` die GitPoulpi ziet: start GitPoulpi waar `claude --version` werkt, of geef het volledige pad. |
| Een lokale agent hangt of toont een interactief scherm | Voeg in de uitvoermodus *Headless* zijn headless vlag (`-p` voor Claude Code) toe aan **Argumenten**, of kies zijn **Agenttype** opnieuw om de headless aanroep te herstellen. Het interactieve scherm is waarvoor de uitvoermodus *Interactief (virtuele terminal)* dient. |
| "Use "Open the agent"..." | De agent stelde een vraag bij de eerste start (de map vertrouwen, inloggen). Gebruik **De agent openen...** op het profiel, beantwoord de vraag, sluit de agent en probeer het opnieuw. |
| Een verzoek via de virtuele terminal loopt op een time-out | Het bericht eindigt met de laatste regels van het scherm van de agent: een toestemmingsvraag, een login of een updatevraag staat daar. Beantwoord die met **De agent openen...**, of verhoog de **Time-out**. |
| `failed:` 401 | Verkeerde sleutel, of een sleutel voor een andere provider: voer hem opnieuw in. |
| `failed:` 404 | Het **Model** bestaat daar niet: wis het om de standaard te gebruiken. |
| `failed:` connection error (Ollama, LM Studio) | De server draait niet, of de **Basis-URL** heeft de verkeerde poort of mist `/v1`. |
| De sleutel toont `(niet ingesteld)` nadat je hem hebt getypt | Geen credential helper: configureer er een (`git config --global credential.helper ...`) en voer hem opnieuw in. |
| Verkeerde taal of stijl | Voeg instructies toe; met de chips Snel invoegen van de editor gaat dat snel. |
| De composer zegt dat het plan ongeldig is | Het model heeft een bestand overgeslagen of herhaald: probeer opnieuw of gebruik een sterker model. |

## Oudere configuratiebestanden

Een oudere `config.toml` met `[[ai.presets]]` en `[ai.defaults]` laadt nog steeds:

- De standaardpreset van elke actie wordt zijn `[ai.actions.<actie>]`
  (`system_prompt` wordt `instructions`).
- Elke oude preset blijft bewaard in `[[ai.legacy_presets]]`. Instellingen > AI toont ze
  onder *Oude presets*, met *Archief verwijderen*.
- De oude `[preferences.ai_presets]` van een repository wordt een
  overschrijving op repositoryniveau.

De ingebouwde prompt komt nu altijd eerst. Als een oude prompt een ingebouwde
prompt kopieerde, verwijder dan het gekopieerde deel en houd alleen wat je wilt toevoegen.
