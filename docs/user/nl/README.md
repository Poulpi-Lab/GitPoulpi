[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · **Nederlands** · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Gebruikersdocumentatie van GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="De octopus van GitPoulpi" width="128"></p>

GitPoulpi is een Git-client voor de terminal met een aanklikbare commit-grafiek,
een opdrachtenpalet, integratie met hosting en issuetrackers, optionele
AI-hulp en een CLI en MCP-server die alles doen wat de UI doet. Deze pagina's
staan in leesvolgorde: begin bovenaan en spring naar de naslagpagina's wanneer
je ze nodig hebt.

![Het hoofdvenster van GitPoulpi](../images/hero.png)

## Begin hier

1. **[Installatie](installation.md)**: wat je nodig hebt, releasearchieven,
   pakketbeheerders, bouwen vanuit de broncode. Voor een kleinere binary, zie
   **[Aangepaste builds](custom-builds.md)**. Hoe GitPoulpi zichzelf actueel
   houdt (de versiechip, het updatepaneel, teruggaan): **[Updates](updates.md)**.
   Om het in een container te draaien: **[GitPoulpi in Docker draaien](docker.md)**.
   Voor beheerders, interne feeds, mirrors en het updatebeleid:
   **[Distributie voor organisaties](enterprise-distribution.md)**.
2. **[Aan de slag](getting-started.md)**: je eerste vijf minuten, van opstarten
   tot je eerste commit.
3. **[Rondleiding door de interface](interface-tour.md)**: de naam van elk deel
   van het venster, met afbeeldingen.

## Dagelijks gebruik

4. **[Werken met de Log](log-and-history.md)**: de grafiek lezen, zoeken,
   branches, tags, stashes, worktrees, refs slepen, rebase, cherry-pick,
   reword en amend, banners, pull en push, auto-stash.
5. **[Stagen en committen](staging-and-committing.md)**: de lokale wijzigingen,
   diffs, hunks en losse regels stagen, commits schrijven, de commit-composer,
   blame, bestandsgeschiedenis en de ingebouwde editor.
6. **[Afbeeldingen](image-view.md)**: afbeeldingen in de Diff-weergave en de
   Bestandsweergave, voor en na, zoom, Bewerken in.
7. **[Merge-conflicten oplossen](merge-conflicts.md)**: de merge-editor (A en B
   naast elkaar boven de uitvoer), regels en blokken kiezen, kaarten voor
   conflicten met binaire bestanden, LFS, verwijderde bestanden, symlinks en
   submodules, en hoe een agent een oplossing voorstelt.
8. **[Ongedaan maken en opnieuw](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, wat wel
   en niet ongedaan kan worden gemaakt, wanneer het weigert of vraagt, hoe ver
   het teruggaat, uitschakelen.
9. **[Git LFS](lfs.md)**: grote bestanden, tracking, locks en wat er gebeurt
   zonder geïnstalleerde `git-lfs`.
10. **[Submodules](submodules.md)**: de sectie SUBMODULES, ze bijwerken en ze
    gelijk houden met checkouts en pulls.
11. **[Thema's](themes.md)**: de ingebouwde thema's, elk thema met elk
    baanpalet, live voorbeeld, je eigen themabestanden, `gitpoulpi theme`.
12. **[Werkruimtes](workspaces.md)**: benoemde groepen van repositories met eigen
    tabbladen en een optioneel standaard commitprofiel, de chip in de tabbalk, het tabblad Werkruimtes, alles fetchen, alles pullen,
    ontbrekende repositories klonen.
13. **[Zoeken](search.md)**: één fuzzy-zoeker (`Ctrl+Alt+F`) over de branches,
    tags, worktrees, stashes, commits, pull/merge requests en issues van elke
    repository van de werkruimte of de open tabbladen.
14. **[De Radar](radar.md)**: de pull/merge requests, issues en het werk
    in uitvoering van een werkruimte (of je open tabbladen) in één lijst: filters,
    pins, uitstellen, en wat elke rij kan doen.
    Zo'n verzoek in de app lezen en reviewen, met opmerkingendiscussies, een
    concept-review en Goedkeuren / Wijzigingen aanvragen:
    **[Een pull request reviewen](review.md)**.
15. **[Openen in](open-in.md)**: de lijst in de titelbalk waarmee je de map van
    het actieve tabblad opent in een terminal, multiplexer, editor of bestandsbeheer.
16. **[De map van het actieve tabblad volgen](following-the-tab-folder.md)**:
    "Afsluiten & cd hier" en de `gitpoulpi shell-init`-wrapper (`gp`)
    voor bash, zsh, fish en PowerShell.
    Welke terminal GitPoulpi het best toont: **[Een terminal kiezen](terminals.md)**.
    Wat er door tmux, zellij, screen, herdr en terminalpanes heen komt:
    **[Multiplexers](multiplexers.md)**.

Ook: **[Branchingworkflows](branching-workflows.md)**, de voorinstellingen Git
Flow, GitHub Flow, GitLab Flow en Trunk-based of je eigen basisbranches,
naampatronen voor branches en branchtypen (`gitpoulpi flow ...`).

**[Geschiedenis herschrijven](history-rewrite.md)**: de herschrijfwerkbank
(`Ctrl+G Shift+H`) die auteurs, datums, berichten en bestandsinhoud over veel commits
herschrijft, met voorbeeld, Lua-scripts en één keer ongedaan maken.

## Toetsenbord en muis

17. **[Sneltoetsen](keybindings.md)**: elke toets, per gebied, en elke
    opdracht van het palet. In de app toont `?` dezelfde lijst en vindt `Ctrl+P`
    elke opdracht.
18. **[Muis en gebaren](mouse-and-gestures.md)**: klikken, slepen, selecteren en
    kopiëren, schuifbalken en terugkoppeling onder de aanwijzer.

## Instellingen en integraties

19. **[Instellingen](settings.md)**: elke pagina en optie van het scherm
    Instellingen, het zoekvak, profielen en hoe je sneltoetsen opnieuw toewijst.
20. **[AI-ondersteuning](ai.md)**: de eigen AI van GitPoulpi (standaard uit):
    een lokale agent zoals `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama of
    LM Studio; instructies; commit-berichten, uitleg, beschrijvingen van pull
    requests en de commit-composer; privacy en probleemoplossing.
21. **[Hosting en issuetrackers](hosting-integration.md)**: GitHub, GitLab, Gitea en Forgejo,
    Azure DevOps, Bitbucket, Gerrit en de issuetrackers:
    tokens en hun scopes, verbinden,
    een repository koppelen, pull requests en issues, verwijzingen in
    commit-berichten, SSH-sleutels, bekende beperkingen. Ook
    **[Lokale issues en Obsidian](local-issues.md)**: een map met Markdown-notities
    als issuebron, zonder token, met een Obsidian Base.

## Automatisering

22. **[CLI-naslag](cli-reference.md)**: elke `gitpoulpi`-opdracht, met `--json`
    voor scripts.
23. **[MCP en agents](mcp-and-agents.md)**: laat een externe agent (Claude Code,
    Cursor, Codex, ...) GitPoulpi bedienen: de server registreren, `.mcp.json`,
    de rechtenschakelaars en hun standaardwaarden, elke tool, en hoe een agent
    dingen in je venster laat zien. De kant-en-klare agentskill is
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Naslag en hulp

24. **[Configuratienaslag](config-reference.md)**: elke `config.toml`-sleutel,
    de standaardwaarde en het effect, de instellingen per repository,
    omgevingsvariabelen.
25. **[Probleemoplossing en FAQ](troubleshooting.md)**: pictogrammen,
    grafiekweergave, toetsen die niet aankomen, vastgelopen daemons, snelheid,
    LFS-pointers.

## Schermafbeeldingen

Elke afbeelding op deze pagina's is een echte opname van een releasebuild, op
een kleine demorepository met verzonnen auteurs. De afbeeldingen staan in
[`images/`](../images/). `scripts/docs-screenshots.sh` maakt de
demorepository's en neemt ze opnieuw op. Ze zijn gemaakt in de terminal
`kitty`, die de commit-grafiek en de auteursavatars als afbeeldingen tekent.
Terminals zonder grafisch protocol tonen in plaats daarvan een grafiek van
tekens.

## Voor bijdragers

[CLAUDE.md](../../../CLAUDE.md), het [architectuuroverzicht](../../architecture/overview.md)
en de [roadmap](../../roadmap.md) beschrijven het ontwerp en wat er gepland is;
[CHANGELOG.md](../../../CHANGELOG.md) toont wat er in elke release veranderd is.

---

GitKraken is een merk van de respectieve eigenaar. GitPoulpi is een onafhankelijk project, niet gelieerd aan, goedgekeurd door of gesponsord door deze eigenaar; de naam wordt alleen genoemd om compatibiliteit en inspiratie te beschrijven.
