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

## Een functie vinden

| Ik wil... | Pagina | Hoe kom je er in de app |
|---|---|---|
| Op twee branches tegelijk werken | [Worktrees](worktrees.md) | De sectie **WORKTREES** van de Refs-zijbalk (`+` maakt er een; verborgen tot er een bestaat: palet **Linkerpaneel: Worktrees-sectie tonen**); palet **Refs: Worktree maken...** |
| Met geneste repositories werken | [Submodules](submodules.md) | De sectie **SUBMODULES** van de Refs-zijbalk; palet **Git: Alle submodules bijwerken**, **Refs: Submodule bijwerken (geselecteerd)** |
| Een merge-conflict oplossen | [Merge-conflicten oplossen](merge-conflicts.md) | `Enter` op een `!`-rij van het paneel Status, de knop **Conflicten oplossen...** van de banner, of `Shift+E` in de Diff-weergave; palet **Conflicten: Merge-editor openen** |
| Een pull of merge request reviewen | [Een pull request reviewen](review.md) | `Enter` op een rij van de sectie **PULL REQUESTS**, `e` op een Radar-rij, `Alt+R` in de werkruimtezoeker; palet **Hosting: Verzoek reviewen...** |
| Pull requests en issues bekijken, maken en beheren | [Pull requests en issues](pull-requests-and-issues.md) | De secties **PULL REQUESTS** en **ISSUES** van de Refs-zijbalk (`n` maakt er een); palet **Pull requests: Pull request maken...**, **Hosting: Nieuw issue...**. Verbind eerst een account: [Hosting en issuetrackers](hosting-integration.md) |
| De discussie van een issue of verzoek lezen en beantwoorden | [Gesprekken](conversations.md) | `Enter` op een issuerij, een bordkaart of een Radar-rij; palet **Hosting: Issuegesprek openen...** |
| Mijn issues als bord zien | [Het issuebord](pull-requests-and-issues.md#het-issuebord) | De knop **Bord** in de kop van ISSUES; palet **Issues: Bord openen** |
| Issues als Markdown-notities bijhouden, zonder token | [Lokale issues en Obsidian](local-issues.md) | De sectie ISSUES, provider `Local Markdown - <naam>`; palet **Lokale issues: Openen in Obsidian...** |
| Auteurs, datums, berichten of bestanden over veel commits herschrijven | [Geschiedenis herschrijven](history-rewrite.md) | `Ctrl+G Shift+H`; palet **Herschrijven: Werkbank voor geschiedenis** |
| Een bewerking ongedaan maken (of opnieuw doen) | [Ongedaan maken en opnieuw](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` opent de geschiedenis van ongedaan maken; palet **Bewerken: Ongedaan maken** |
| Grote bestanden volgen met Git LFS | [Git LFS](lfs.md) | Palet **LFS: Patroon volgen...**, **LFS: Initialiseren voor deze repository** |
| Een branch, commit, pull request of issue vinden in alle repositories | [Zoeken](search.md) | `Ctrl+Alt+F`, of de knop Globaal zoeken in de titelbalk; `f` zoekt in de commit-grafiek van de huidige repository |
| Alles zien wat mij nodig heeft, in één lijst | [De Radar](radar.md) | `Alt+R`; palet **Radar: Openen** |
| Repositories groeperen en samen fetchen of pullen | [Werkruimtes](workspaces.md) | `Alt+W` wisselt, `Alt+J` opent het tabblad Werkruimtes; palet **Werkruimte: Beheren...** |
| AI-hulp krijgen (commit-berichten, uitleg, beschrijvingen) | [AI-ondersteuning](ai.md) | `Ctrl+,` > **AI** > **AI inschakelen**, dan schrijft `Ctrl+G i` een commit-bericht; palet **AI: ...** |
| Elke opdracht of toets vinden | [Sneltoetsen](keybindings.md) | `?` in de app toont de toetsen, `Ctrl+P` vindt elke opdracht |

## Dagelijks gebruik

4. **[Werken met de Log](log-and-history.md)**: de grafiek lezen, zoeken,
   branches, tags, stashes, refs slepen, rebase, cherry-pick,
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
8. **[Worktrees](worktrees.md)**: een tweede uitgecheckte map op een andere
   branch: de sectie WORKTREES, gekoppelde worktrees maken, openen, mergen en
   verwijderen.
9. **[Submodules](submodules.md)**: de sectie SUBMODULES, ze bijwerken en ze
   gelijk houden met checkouts en pulls.
10. **[Ongedaan maken en opnieuw](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, wat wel
    en niet ongedaan kan worden gemaakt, wanneer het weigert of vraagt, hoe ver
    het teruggaat, uitschakelen.
11. **[Git LFS](lfs.md)**: grote bestanden, tracking, locks en wat er gebeurt
    zonder geïnstalleerde `git-lfs`.
12. **[Thema's](themes.md)**: de ingebouwde thema's, elk thema met elk
    baanpalet, live voorbeeld, je eigen themabestanden, `gitpoulpi theme`.
13. **[Werkruimtes](workspaces.md)**: benoemde groepen van repositories met eigen
    tabbladen en een optioneel standaard commitprofiel, de chip in de tabbalk, het tabblad Werkruimtes, alles fetchen, alles pullen,
    ontbrekende repositories klonen.
14. **[Zoeken](search.md)**: één fuzzy-zoeker (`Ctrl+Alt+F`) over de branches,
    tags, worktrees, stashes, commits, pull/merge requests en issues van elke
    repository van de werkruimte of de open tabbladen.
15. **[De Radar](radar.md)**: de pull/merge requests, issues en het werk
    in uitvoering van een werkruimte (of je open tabbladen) in één lijst: filters,
    pins, uitstellen, en wat elke rij kan doen.
16. **[Pull requests en issues](pull-requests-and-issues.md)**: de secties
    PULL REQUESTS en ISSUES van de zijbalk, het issuebord, een pull request of
    issue maken, de chip in de werkbalk, verwijzingen in commit-berichten.
17. **[Gesprekken](conversations.md)**: de discussie van een issue of verzoek
    in het venster lezen en beantwoorden, plaatsen als, wat elke host niet kan.
18. **[Een pull request reviewen](review.md)**: de reviewweergave met
    opmerkingendiscussies, een concept-review en Goedkeuren / Wijzigingen aanvragen.
19. **[Openen in](open-in.md)**: de lijst in de titelbalk waarmee je de map van
    het actieve tabblad opent in een terminal, multiplexer, editor of bestandsbeheer.
20. **[De map van het actieve tabblad volgen](following-the-tab-folder.md)**:
    "Afsluiten & cd hier" en de `gitpoulpi shell-init`-wrapper (`gp`)
    voor bash, zsh, fish en PowerShell.
    Welke terminal GitPoulpi het best toont: **[Een terminal kiezen](terminals.md)**.
    Wat er door tmux, zellij, screen, herdr en terminalpanes heen komt:
    **[Multiplexers](multiplexers.md)**.

Ook: **[Meerdere remotes voor één branch](multiple-remotes.md)**: push-doelen en pull-bronnen,
het dialoogvenster Remotes, Push naar alle of geselecteerde remotes, een remote oplossen die is
doorgegaan, Doorgaan na een conflict, de CLI en een plain-git-alias.

**[Branchingworkflows](branching-workflows.md)**, de voorinstellingen Git
Flow, GitHub Flow, GitLab Flow en Trunk-based of je eigen basisbranches,
naampatronen voor branches en branchtypen (`gitpoulpi flow ...`).

**[Geschiedenis herschrijven](history-rewrite.md)**: de herschrijfwerkbank
(`Ctrl+G Shift+H`) die auteurs, datums, berichten en bestandsinhoud over veel commits
herschrijft, met voorbeeld, Lua-scripts en één keer ongedaan maken.

## Toetsenbord en muis

21. **[Sneltoetsen](keybindings.md)**: elke toets, per gebied, en elke
    opdracht van het palet. In de app toont `?` dezelfde lijst en vindt `Ctrl+P`
    elke opdracht.
22. **[Muis en gebaren](mouse-and-gestures.md)**: klikken, slepen, selecteren en
    kopiëren, schuifbalken en terugkoppeling onder de aanwijzer.

## Instellingen en integraties

23. **[Instellingen](settings.md)**: elke pagina en optie van het scherm
    Instellingen, het zoekvak, profielen en hoe je sneltoetsen opnieuw toewijst.
24. **[AI-ondersteuning](ai.md)**: de eigen AI van GitPoulpi (standaard uit):
    een lokale agent zoals `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama of
    LM Studio; instructies; commit-berichten, uitleg, beschrijvingen van pull
    requests en de commit-composer; privacy en probleemoplossing.
25. **[Hosting en issuetrackers](hosting-integration.md)**: GitHub, GitLab, Gitea en Forgejo,
    Azure DevOps, Bitbucket, Gerrit en de issuetrackers:
    tokens en hun scopes, verbinden,
    een repository koppelen, SSH-sleutels, bekende beperkingen. Ook
    **[Lokale issues en Obsidian](local-issues.md)**: een map met Markdown-notities
    als issuebron, zonder token, met een Obsidian Base.

## Automatisering

26. **[CLI-naslag](cli-reference.md)**: elke `gitpoulpi`-opdracht, met `--json`
    voor scripts.
27. **[MCP en agents](mcp-and-agents.md)**: laat een externe agent (Claude Code,
    Cursor, Codex, ...) GitPoulpi bedienen: de server registreren, `.mcp.json`,
    de rechtenschakelaars en hun standaardwaarden, elke tool, en hoe een agent
    dingen in je venster laat zien. De kant-en-klare agentskill is
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Naslag en hulp

28. **[Configuratienaslag](config-reference.md)**: elke `config.toml`-sleutel,
    de standaardwaarde en het effect, de instellingen per repository,
    omgevingsvariabelen.
29. **[Probleemoplossing en FAQ](troubleshooting.md)**: pictogrammen,
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

Alle andere product- en dienstnamen die in GitPoulpi en de bijbehorende documentatie worden genoemd, zijn merken van hun respectieve eigenaren, bijvoorbeeld codehosts (GitHub, GitLab, Bitbucket enz.), issuetrackers (Jira, Trello, Linear enz.) en de editors en tools die GitPoulpi kan openen (Visual Studio Code, JetBrains-IDE's, lazygit enz.). Ze worden alleen gebruikt om aan te geven waarmee GitPoulpi werkt; GitPoulpi is niet gelieerd aan, goedgekeurd door of gesponsord door een van hen.
