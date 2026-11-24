[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · **Svenska** · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# GitPoulpis användardokumentation

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="GitPoulpis bläckfisk" width="128"></p>

GitPoulpi är en Git-klient för terminalen med en klickbar commit-graf, en
kommandopalett, integration med värdtjänster och ärendehantering, valfri AI-hjälp
och en CLI och en MCP-server som gör allt gränssnittet gör. Sidorna ligger i
läsordning: börja överst och hoppa till referenssidorna när du behöver dem.

![GitPoulpis huvudfönster](../images/hero.png)

## Börja här

1. **[Installation](installation.md)**: vad du behöver, releasearkiv,
   pakethanterare, bygga från källkod. För en mindre binär, se
   **[Anpassade byggen](custom-builds.md)**. Hur GitPoulpi håller sig aktuellt
   (versionschipet, uppdateringspanelen, gå tillbaka): **[Uppdateringar](updates.md)**.
   För att köra det i en container: **[Köra i Docker](docker.md)**. För
   administratörer, interna feeds, speglar och uppdateringspolicyn:
   **[Företagsdistribution](enterprise-distribution.md)**.
2. **[Komma igång](getting-started.md)**: dina första fem minuter, från start
   till din första commit.
3. **[Rundtur i gränssnittet](interface-tour.md)**: namnet på varje område i
   fönstret, med bilder.

## Hitta en funktion

| Jag vill... | Sida | Så når du den i appen |
|---|---|---|
| Arbeta på två grenar samtidigt | [Worktrees](worktrees.md) | Avsnittet **WORKTREES** i sidopanelen Refs (`+` skapar en; dolt tills en finns: paletten **Sidopanel: Visa avsnittet Worktrees**); paletten **Refs: Skapa worktree...** |
| Arbeta med kapslade repositories | [Submoduler](submodules.md) | Avsnittet **SUBMODULER** i sidopanelen Refs; paletten **Git: Uppdatera alla submoduler**, **Refs: Uppdatera submodul (vald)** |
| Lösa en merge-konflikt | [Lösa merge-konflikter](merge-conflicts.md) | `Enter` på en `!`-rad i Status-rutan, bannerns knapp **Lös konflikter...**, eller `Shift+E` i Diff-vyn; paletten **Konflikter: Öppna merge-redigeraren** |
| Granska en pull request eller merge request | [Granska en pull request](review.md) | `Enter` på en rad i avsnittet **PULL REQUESTS**, `e` på en Radar-rad, `Alt+R` i arbetsytesökningen; paletten **Hosting: Granska begäran...** |
| Bläddra bland, skapa och hantera pull requests och ärenden | [Pull requests och ärenden](pull-requests-and-issues.md) | Avsnitten **PULL REQUESTS** och **ÄRENDEN** i sidopanelen Refs (`n` skapar en); paletten **Pull requests: Skapa pull request...**, **Hosting: Nytt ärende...**. Anslut först ett konto: [Värdtjänster och ärendehantering](hosting-integration.md) |
| Läsa och besvara diskussionen i ett ärende eller en begäran | [Konversationer](conversations.md) | `Enter` på en ärenderad, ett tavelkort eller en Radar-rad; paletten **Värd: Öppna ärendekonversation...** |
| Se mina ärenden som en tavla | [Ärendetavlan](pull-requests-and-issues.md#ärendetavlan) | Knappen **Tavla** i ÄRENDEN-huvudet; paletten **Ärenden: Öppna tavlan** |
| Ha ärenden som Markdown-anteckningar, utan token | [Lokala ärenden och Obsidian](local-issues.md) | Avsnittet ÄRENDEN, leverantören `Local Markdown - <namn>`; paletten **Lokala ärenden: Öppna i Obsidian...** |
| Skriva om författare, datum, meddelanden eller filer över många commits | [Skriva om historiken](history-rewrite.md) | `Ctrl+G Shift+H`; paletten **Omskrivning: Öppna omskrivningsvyn** |
| Ångra (eller göra om) en åtgärd | [Ångra och gör om](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` öppnar ångrahistoriken; paletten **Redigera: Ångra** |
| Köra flera åtgärder med en tangent (stage, AI-meddelande, commit, push...) | [Sekvenser](sequences.md) | Inställningar > Sekvenser; paletten **Sekvens: <namn>** |
| Spåra stora filer med Git LFS | [Git LFS](lfs.md) | Paletten **LFS: Spåra mönster...**, **LFS: Initiera för det här repositoryt** |
| Hitta en gren, commit, pull request eller ett ärende över flera repositories | [Sökning](search.md) | `Ctrl+Alt+F`, eller knappen Global sökning i titelraden; `f` söker i commit-grafen för det aktuella repositoryt |
| Se allt som behöver mig i en enda lista | [Radar](radar.md) | `Alt+R`; paletten **Radar: Öppna** |
| Gruppera repositories och göra fetch eller pull på dem tillsammans | [Arbetsytor](workspaces.md) | `Alt+W` växlar, `Alt+J` öppnar fliken Arbetsytor; paletten **Arbetsyta: Hantera...** |
| Få AI-hjälp (commit-meddelanden, förklaringar, beskrivningar) | [AI-hjälp](ai.md) | `Ctrl+,` > **AI** > **Aktivera AI**, sedan skriver `Ctrl+G i` ett commit-meddelande; paletten **AI: ...** |
| Hitta vilket kommando eller vilken tangent som helst | [Kortkommandon](keybindings.md) | `?` i appen listar tangenterna, `Ctrl+P` hittar vilket kommando som helst |

## Daglig användning

4. **[Arbeta med Log](log-and-history.md)**: läsa grafen, sökning, grenar, tags,
   stashes, dra refs, rebase, cherry-pick, reword och amend, bannrar, pull
   och push, auto-stash.
5. **[Stage och commit](staging-and-committing.md)**: de lokala ändringarna, diffar,
   stagea hunks och enskilda rader, skriva commits, commit-composern, blame,
   filhistorik och den inbyggda redigeraren.
6. **[Bilder](image-view.md)**: bilder i Diff-vyn och filvyn, före och efter,
   zoom, Redigera i.
7. **[Lösa merge-konflikter](merge-conflicts.md)**: merge-redigeraren (A och B
   sida vid sida över Resultat), välja rader och block, kort för binära, LFS-,
   borttagna, symlänk- och submodulkonflikter, och hur en agent föreslår en
   lösning.
8. **[Worktrees](worktrees.md)**: en andra utcheckad mapp på en annan gren:
   avsnittet WORKTREES, skapa, öppna, merga och ta bort länkade worktrees.
9. **[Submoduler](submodules.md)**: avsnittet SUBMODULER, uppdatera dem och
   hålla dem i takt med checkouts och pulls.
10. **[Ångra och gör om](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, vad som kan och
    inte kan ångras, när det vägrar eller frågar, hur långt tillbaka det går,
    stänga av det.
11. **[Git LFS](lfs.md)**: stora filer, spårning, lås, och vad som händer utan
    installerat `git-lfs`.
12. **[Teman](themes.md)**: de inbyggda temana, valfritt tema med valfri
    filfärgpalett, liveförhandsvisning, dina egna temafiler, `gitpoulpi theme`.
13. **[Arbetsytor](workspaces.md)**: namngivna grupper av repositories med egna
    flikar och en valfri standardprofil för commits, chipet i flikraden, fliken Arbetsytor, Fetch alla, Pull alla, Klona de saknade.
14. **[Sökning](search.md)**: en fuzzy-sökare (`Ctrl+Alt+F`) över grenar, taggar,
    worktrees, stashar, commits, pull/merge requests och ärenden i varje
    repository i arbetsytan eller de öppna flikarna.
15. **[Radar](radar.md)**: pull/merge requests, ärenden och pågående
    arbete i en arbetsyta (eller dina öppna flikar) i en enda lista: filter,
    fästa rader, snoozning, och vad varje rad kan göra.
16. **[Pull requests och ärenden](pull-requests-and-issues.md)**: avsnitten PULL
    REQUESTS och ÄRENDEN i sidopanelen, ärendetavlan, att skapa en pull request
    eller ett ärende, chippet i verktygsfältet, referenser i commit-meddelanden.
17. **[Konversationer](conversations.md)**: läsa och besvara diskussionen i ett
    ärende eller en begäran i fönstret, publicera som, vad varje värd inte kan göra.
18. **[Granska en pull request](review.md)**: granskningsvyn med kommentarstrådar,
    en väntande granskning och Godkänn / Begär ändringar.
19. **[Öppna i](open-in.md)**: titelradens lista som öppnar den aktiva flikens
    mapp i en terminal, multiplexer, redigerare eller filhanterare.
20. **[Följa den aktiva flikens mapp](following-the-tab-folder.md)**:
    ”Quit and cd here”, och omslaget `gitpoulpi shell-init` (`gp`) för bash,
    zsh, fish och PowerShell.
    Vilken terminal som visar GitPoulpi bäst: **[Välja terminal](terminals.md)**.
    Vad som går igenom tmux, zellij, screen, herdr och terminalpaneler:
    **[Multiplexers](multiplexers.md)**.

Också: **[Flera remotes för en gren](multiple-remotes.md)**: push-mål och pull-källor, dialogen
Remotes, Push till alla eller till valda remotes, lösa en remote som flyttat sig, Fortsätt efter en
konflikt, CLI och ett rent git-alias.

**[Grenarbetsflöden](branching-workflows.md)**, förinställningarna Git Flow, GitHub
Flow, GitLab Flow och Trunk-based eller dina egna baser, grennamnsmönster och
grentyper (`gitpoulpi flow ...`).

**[Skriva om historiken](history-rewrite.md)**: omskrivningsvyn (`Ctrl+G Shift+H`) som
skriver om författare, datum, meddelanden och filinnehåll över många commits, med
förhandsgranskning, Lua-skript och en enda ångra.

## Tangentbord & mus

21. **[Kortkommandon](keybindings.md)**: varje tangent, per område, och varje
    palettkommando. I appen visar `?` samma lista och `Ctrl+P` hittar vilket
    kommando som helst.
22. **[Mus och gester](mouse-and-gestures.md)**: klicka, dra, markera och
    kopiera, rullningslister och hovringsrespons.

## Inställningar och integrationer

23. **[Inställningar](settings.md)**: varje sida och alternativ i
    inställningsskärmen, sökrutan, profiler, och hur du ändrar genvägar.
24. **[AI-hjälp](ai.md)**: GitPoulpis egen AI (av som standard): en lokal agent
    som `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama eller LM Studio;
    instruktioner; commit-meddelanden, förklaringar, pull request-beskrivningar
    och commit-composern; integritet och felsökning.
25. **[Värdtjänster och ärendehantering](hosting-integration.md)**: GitHub, GitLab, Gitea och Forgejo,
    Azure DevOps, Bitbucket, Gerrit och ärendehanterarna: tokens och deras behörigheter, ansluta, länka ett repository, SSH-nycklar, kända
    begränsningar. Även
    **[Lokala ärenden och Obsidian](local-issues.md)**: en mapp med
    Markdown-anteckningar som ärendekälla, ingen token, med en Obsidian Base.

## Automation

26. **[CLI-referens](cli-reference.md)**: varje `gitpoulpi`-kommando, med
    `--json` för skript.
27. **[MCP och agenter](mcp-and-agents.md)**: låt en extern agent (Claude Code,
    Cursor, Codex, ...) styra GitPoulpi: registrera servern, `.mcp.json`,
    behörighetsbrytarna och deras standardvärden, varje verktyg, och hur en
    agent visar dig saker i ditt fönster. Den färdiga agentskillen är
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referens och hjälp

28. **[Konfigurationsreferens](config-reference.md)**: varje `config.toml`-nyckel,
    dess standardvärde och effekt, inställningarna per repository,
    miljövariabler.
29. **[Felsökning och vanliga frågor](troubleshooting.md)**: ikoner, grafritning,
    tangenter som inte kommer fram, fastnade daemoner, hastighet, LFS-pekare.

## Skärmbilder

Varje bild på de här sidorna är en riktig skärmdump av ett releasebygge, på ett
litet demo-repository med påhittade författare. Bilderna finns i
[`images/`](../images/). `scripts/docs-screenshots.sh` bygger demo-repositoryna
och tar dem på nytt. De tas i terminalen `kitty`, som ritar commit-grafen och
författaravatarerna som bilder. Terminaler utan grafikprotokoll visar en
teckengraf i stället.

## För bidragsgivare

[CLAUDE.md](../../../CLAUDE.md), [arkitekturöversikten](../../architecture/overview.md)
och [färdplanen](../../roadmap.md) beskriver designen och vad som är planerat;
[CHANGELOG.md](../../../CHANGELOG.md) listar vad som ändrades i varje release.

---

GitKraken är ett varumärke som tillhör respektive ägare. GitPoulpi är ett oberoende projekt och är inte knutet till, godkänt av eller sponsrat av ägaren; namnet nämns endast för att beskriva kompatibilitet och inspiration.

Alla andra produkt- och tjänstenamn som nämns i GitPoulpi och dess dokumentation är varumärken som tillhör respektive ägare, till exempel kodvärdar (GitHub, GitLab, Bitbucket m.fl.), ärendehanteringssystem (Jira, Trello, Linear m.fl.) samt de redigerare och verktyg som GitPoulpi kan öppna (Visual Studio Code, JetBrains IDE:er, lazygit m.fl.). De används endast för att namnge vad GitPoulpi fungerar med; GitPoulpi är inte knutet till, godkänt av eller sponsrat av någon av dem.
