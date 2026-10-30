[English](../README.md) · **Čeština** · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Uživatelská dokumentace GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="Chobotnice GitPoulpi" width="128"></p>

GitPoulpi je Git klient pro terminál s grafem commitů, na který lze klikat,
paletou příkazů, integrací hostingových služeb a issue trackerů, volitelnou
pomocí AI a serverem CLI a MCP, které umějí vše, co rozhraní. Stránky jsou
řazeny v pořadí čtení: začněte nahoře a k referenčním stránkám skočte, až je
budete potřebovat.

![Hlavní okno GitPoulpi](../images/hero.png)

## Začněte zde

1. **[Instalace](installation.md)**: co potřebujete, archivy vydání, sestavení
   ze zdrojových kódů. Pro menší binární soubor viz
   **[Vlastní sestavení](custom-builds.md)**.
2. **[První kroky](getting-started.md)**: vašich prvních pět minut, od spuštění
   po první commit.
3. **[Prohlídka rozhraní](interface-tour.md)**: název každé oblasti okna, s
   obrázky.

## Každodenní používání

4. **[Práce s Logem](log-and-history.md)**: čtení grafu, hledání, větve, tagy,
   stashe, worktrees, přetahování refs, rebase, cherry-pick, změna zprávy a úprava
   commitu, bannery, pull a push, auto-stash.
5. **[Stagování a commitování](staging-and-committing.md)**: rozpracované změny, diffy,
   stage hunků i jednotlivých řádků, psaní commitů, editor commitů, blame,
   historie souboru a zabudovaný editor.
6. **[Obrázky](image-view.md)**: obrázky v zobrazení diff a v zobrazení
   Soubor, před a po, zvětšení, Upravit v.
7. **[Řešení merge konfliktů](merge-conflicts.md)**: editor merge (A a B vedle
   sebe nad Výstupem), výběr řádků a bloků, karty pro konflikty binárních
   souborů, LFS, smazaných souborů, symlinků a submodulů a jak agent navrhuje
   řešení.
8. **[Zpět a znovu](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, co lze a co nelze
   vrátit, kdy to odmítne nebo se zeptá, jak daleko do minulosti sahá, jak to
   vypnout.
9. **[Git LFS](lfs.md)**: velké soubory, sledování, zámky a co se stane bez
   nainstalovaného `git-lfs`.
10. **[Submoduly](submodules.md)**: sekce SUBMODULY, jejich aktualizace a
    udržování v souladu s checkouty a pully.
11. **[Motivy](themes.md)**: vestavěné motivy, libovolný motiv s libovolnou
    paletou drah, živý náhled, vlastní soubory motivů, `gitpoulpi theme`.
12. **[Pracovní prostory](workspaces.md)**: pojmenované skupiny repozitářů s
    vlastními kartami a volitelným výchozím profilem commitů, čip na liště
    karet, karta Pracovní prostory, Fetch vše, Pull vše,
    Naklonovat chybějící.
13. **[Hledání](search.md)**: jeden fuzzy vyhledávač (`Ctrl+Alt+F`) přes větve,
    tagy, worktrees, stashe, commity, pull/merge requesty a issues všech
    repozitářů pracovního prostoru nebo otevřených karet.
14. **[Radar](radar.md)**: pull/merge requesty, issues a rozpracovaná
    práce pracovního prostoru (nebo vašich otevřených karet) v jednom seznamu:
    filtry, připnutí, odložení a co umí každý řádek.
    Čtení a kontrola takové žádosti přímo v aplikaci, s vlákny komentářů,
    čekající kontrolou a Schválit / Požádat o změny:
    **[Kontrola pull requestu](review.md)**.
15. **[Otevřít v](open-in.md)**: seznam v záhlaví okna, který otevře složku
    aktivní karty v terminálu, multiplexeru, editoru nebo správci souborů.
16. **[Sledování složky aktivní karty](following-the-tab-folder.md)**:
    „Konec a cd sem“ a obálka `gitpoulpi shell-init` (`gp`) pro bash, zsh,
    fish a PowerShell.
    Který terminál GitPoulpi zobrazí nejlépe: **[Výběr terminálu](terminals.md)**.

Také: **[Workflow větvení](branching-workflows.md)**, předvolby Git Flow, GitHub
Flow, GitLab Flow a Trunk-based nebo vlastní základní větve, vzory názvů větví
a typy tématických větví (`gitpoulpi flow ...`).

**[Přepis historie](history-rewrite.md)**: nástroj pro přepis (`Ctrl+G Shift+H`),
který přepisuje autory, data, zprávy a obsah souborů v mnoha commitech, s náhledem,
skripty v Lua a jedním vrácením zpět.

## Klávesnice a myš

17. **[Klávesové zkratky](keybindings.md)**: každá klávesa podle oblasti a
    každý příkaz palety. V aplikaci totéž ukáže `?` a `Ctrl+P` najde libovolný
    příkaz.
18. **[Myš a gesta](mouse-and-gestures.md)**: klikání, přetahování, výběr a
    kopírování, posuvníky a zpětná vazba při najetí.

## Nastavení a integrace

19. **[Nastavení](settings.md)**: každá stránka a volba obrazovky Nastavení,
    vyhledávací pole, profily a jak přemapovat zkratky.
20. **[AI asistence](ai.md)**: vlastní AI GitPoulpi (ve výchozím stavu
    vypnutá): lokální agent jako `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama nebo LM Studio; instrukce; zprávy commitů, vysvětlení, popisy pull
    requestů a editor commitů; soukromí a řešení potíží.
21. **[Hostingové služby a issue trackery](hosting-integration.md)**: GitHub, GitLab, Gitea a Forgejo,
    Azure DevOps, Bitbucket, Gerrit a issue trackery: tokeny a jejich rozsahy, připojení, propojení repozitáře, pull requesty a issues, odkazy ve
    zprávách commitů, klíče SSH, známá omezení. Také
    **[Lokální issues a Obsidian](local-issues.md)**: složka poznámek Markdown
    jako zdroj issues, bez tokenu, s Obsidian Base.

## Automatizace

22. **[Referenční příručka CLI](cli-reference.md)**: každý příkaz `gitpoulpi`, s `--json`
    pro skripty.
23. **[MCP a agenti](mcp-and-agents.md)**: nechte externího agenta (Claude
    Code, Cursor, Codex, ...) ovládat GitPoulpi: registrace serveru,
    `.mcp.json`, přepínače oprávnění a jejich výchozí hodnoty, každý nástroj a
    jak vám agent věci ukazuje ve vašem okně. Hotový skill agenta je
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Reference a nápověda

24. **[Referenční příručka konfigurace](config-reference.md)**: každý klíč `config.toml`,
    jeho výchozí hodnota a účinek, nastavení pro jednotlivé repozitáře,
    proměnné prostředí.
25. **[Řešení potíží a FAQ](troubleshooting.md)**: ikony, vykreslování grafu,
    klávesy, které nedorazí, zaseklé daemony, rychlost, ukazatele LFS.

## Snímky obrazovky

Každý obrázek na těchto stránkách je skutečný snímek vydané verze na malém
ukázkovém repozitáři s vymyšlenými autory. Obrázky jsou v [`images/`](../images/).
`scripts/docs-screenshots.sh` sestaví ukázkové repozitáře a pořídí je znovu.
Jsou pořízeny v terminálu `kitty`, který graf commitů a avatary autorů kreslí
jako obrázky. Terminály bez grafického protokolu místo toho zobrazí znakový
graf.

## Pro přispěvatele

[CLAUDE.md](../../../CLAUDE.md), [přehled architektury](../../architecture/overview.md)
a [roadmapa](../../roadmap.md) popisují návrh a plány;
[CHANGELOG.md](../../../CHANGELOG.md) uvádí, co se změnilo v jednotlivých
vydáních.

---

GitKraken je ochranná známka svého příslušného vlastníka. GitPoulpi je nezávislý projekt, který není s tímto vlastníkem spojen ani jím schválen či sponzorován; název je uveden pouze pro popis kompatibility a inspirace.
