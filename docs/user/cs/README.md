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

1. **[Instalace](installation.md)**: co potřebujete, archivy vydání, správci
   balíčků, sestavení ze zdrojových kódů. Pro menší binární soubor viz
   **[Vlastní sestavení](custom-builds.md)**. Jak se GitPoulpi udržuje aktuální
   (čip verze, panel aktualizací, návrat): **[Aktualizace](updates.md)**.
   Spuštění v kontejneru: **[Běh v Dockeru](docker.md)**. Pro správce, interní
   zdroje, zrcadla a politiku aktualizací:
   **[Podniková distribuce](enterprise-distribution.md)**.
2. **[První kroky](getting-started.md)**: vašich prvních pět minut, od spuštění
   po první commit.
3. **[Prohlídka rozhraní](interface-tour.md)**: název každé oblasti okna, s
   obrázky.

## Najděte funkci

| Chci... | Stránka | Jak se k tomu v aplikaci dostat |
|---|---|---|
| Pracovat na dvou větvích najednou | [Worktrees](worktrees.md) | Sekce **WORKTREES** v postranním panelu Refs (`+` jeden vytvoří; skrytá, dokud nějaký není: paleta **Levý panel: Zobrazit sekci Worktrees**); paleta **Refs: Vytvořit worktree...** |
| Pracovat s vnořenými repozitáři | [Submoduly](submodules.md) | Sekce **SUBMODULY** v postranním panelu Refs; paleta **Git: Aktualizovat všechny submoduly**, **Refs: Aktualizovat submodul (vybraný)** |
| Vyřešit merge konflikt | [Řešení merge konfliktů](merge-conflicts.md) | `Enter` na řádku `!` v panelu Stav, tlačítko banneru **Vyřešit konflikty...** nebo `Shift+E` v zobrazení Diff; paleta **Konflikty: Otevřít editor merge** |
| Zkontrolovat pull nebo merge request | [Kontrola pull requestu](review.md) | `Enter` na řádku sekce **PULL REQUESTY**, `e` na řádku Radaru, `Alt+R` ve vyhledávání pracovního prostoru; paleta **Hosting: Zkontrolovat žádost...** |
| Procházet, vytvářet a spravovat pull requesty a issues | [Pull requesty a issues](pull-requests-and-issues.md) | Sekce **PULL REQUESTY** a **ISSUES** v postranním panelu Refs (`n` jeden vytvoří); paleta **Pull requesty: Vytvořit pull request...**, **Hosting: Nové issue...**. Nejdřív připojte účet: [Hostingové služby a issue trackery](hosting-integration.md) |
| Číst a odpovídat v diskusi k issue nebo žádosti | [Konverzace](conversations.md) | `Enter` na řádku issue, na kartě tabule nebo na řádku Radaru; paleta **Hosting: Otevřít konverzaci issue...** |
| Vidět své issues jako tabuli | [Tabule issues](pull-requests-and-issues.md#tabule-issues) | Tlačítko **Tabule** v záhlaví ISSUES; paleta **Issues: Otevřít tabuli** |
| Vést issues jako poznámky v Markdownu, bez tokenu | [Lokální issues a Obsidian](local-issues.md) | Sekce ISSUES, poskytovatel `Local Markdown - <name>`; paleta **Lokální issues: Otevřít v Obsidianu...** |
| Přepsat autory, data, zprávy nebo soubory v mnoha commitech | [Přepis historie](history-rewrite.md) | `Ctrl+G Shift+H`; paleta **Přepis: Otevřít nástroj pro přepis historie** |
| Vrátit zpět (nebo znovu) operaci | [Zpět a znovu](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` otevře historii zpět; paleta **Úpravy: Zpět** |
| Sledovat velké soubory pomocí Git LFS | [Git LFS](lfs.md) | Paleta **LFS: Sledovat vzor...**, **LFS: Inicializovat pro tento repozitář** |
| Najít větev, commit, pull request nebo issue napříč repozitáři | [Hledání](search.md) | `Ctrl+Alt+F` nebo tlačítko Globální hledání v záhlaví okna; `f` hledá v grafu commitů aktuálního repozitáře |
| Vidět vše, co čeká na mě, v jednom seznamu | [Radar](radar.md) | `Alt+R`; paleta **Radar: Otevřít** |
| Seskupit repozitáře a načítat nebo pullovat je společně | [Pracovní prostory](workspaces.md) | `Alt+W` přepíná, `Alt+J` otevře kartu Pracovní prostory; paleta **Pracovní prostor: Spravovat...** |
| Získat pomoc od AI (zprávy commitů, vysvětlení, popisy) | [AI asistence](ai.md) | `Ctrl+,` > **AI** > **Zapnout AI**, pak `Ctrl+G i` napíše zprávu commitu; paleta **AI: ...** |
| Najít libovolný příkaz nebo klávesu | [Klávesové zkratky](keybindings.md) | `?` v aplikaci vypíše klávesy, `Ctrl+P` najde libovolný příkaz |

## Každodenní používání

4. **[Práce s Logem](log-and-history.md)**: čtení grafu, hledání, větve, tagy,
   stashe, přetahování refs, rebase, cherry-pick, změna zprávy a úprava
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
8. **[Worktrees](worktrees.md)**: druhá přepnutá složka na jiné větvi: sekce
   WORKTREES, vytváření, otevírání, merge a odebírání propojených worktrees.
9. **[Submoduly](submodules.md)**: sekce SUBMODULY, jejich aktualizace a
   udržování v souladu s checkouty a pully.
10. **[Zpět a znovu](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, co lze a co nelze
    vrátit, kdy to odmítne nebo se zeptá, jak daleko do minulosti sahá, jak to
    vypnout.
11. **[Git LFS](lfs.md)**: velké soubory, sledování, zámky a co se stane bez
    nainstalovaného `git-lfs`.
12. **[Motivy](themes.md)**: vestavěné motivy, libovolný motiv s libovolnou
    paletou drah, živý náhled, vlastní soubory motivů, `gitpoulpi theme`.
13. **[Pracovní prostory](workspaces.md)**: pojmenované skupiny repozitářů s
    vlastními kartami a volitelným výchozím profilem commitů, čip na liště
    karet, karta Pracovní prostory, Fetch vše, Pull vše,
    Naklonovat chybějící.
14. **[Hledání](search.md)**: jeden fuzzy vyhledávač (`Ctrl+Alt+F`) přes větve,
    tagy, worktrees, stashe, commity, pull/merge requesty a issues všech
    repozitářů pracovního prostoru nebo otevřených karet.
15. **[Radar](radar.md)**: pull/merge requesty, issues a rozpracovaná
    práce pracovního prostoru (nebo vašich otevřených karet) v jednom seznamu:
    filtry, připnutí, odložení a co umí každý řádek.
16. **[Pull requesty a issues](pull-requests-and-issues.md)**: sekce PULL
    REQUESTY a ISSUES v postranním panelu, tabule issues, vytvoření pull
    requestu nebo issue, čip na panelu nástrojů, odkazy ve zprávách commitů.
17. **[Konverzace](conversations.md)**: čtení a odpovídání v diskusi k issue
    nebo žádosti přímo v okně, odesílání jako, co který hostitel neumí.
18. **[Kontrola pull requestu](review.md)**: pohled kontroly s vlákny
    komentářů, čekající kontrolou a Schválit / Požádat o změny.
19. **[Otevřít v](open-in.md)**: seznam v záhlaví okna, který otevře složku
    aktivní karty v terminálu, multiplexeru, editoru nebo správci souborů.
20. **[Sledování složky aktivní karty](following-the-tab-folder.md)**:
    „Konec a cd sem“ a obálka `gitpoulpi shell-init` (`gp`) pro bash, zsh,
    fish a PowerShell.
    Který terminál GitPoulpi zobrazí nejlépe: **[Výběr terminálu](terminals.md)**.
    Co projde přes tmux, zellij, screen, herdr a panely terminálu:
    **[Multiplexery](multiplexers.md)**.

Také: **[Několik remote pro jednu větev](multiple-remotes.md)**: cíle push a zdroje
pull, dialog Remote, Push na všechny nebo vybrané remote, řešení remote, který se posunul,
Pokračovat po konfliktu, CLI a alias pro obyčejný git.

**[Workflow větvení](branching-workflows.md)**, předvolby Git Flow, GitHub
Flow, GitLab Flow a Trunk-based nebo vlastní základní větve, vzory názvů větví

**[Přepis historie](history-rewrite.md)**: nástroj pro přepis (`Ctrl+G Shift+H`),
který přepisuje autory, data, zprávy a obsah souborů v mnoha commitech, s náhledem,
skripty v Lua a jedním vrácením zpět.

## Klávesnice a myš

21. **[Klávesové zkratky](keybindings.md)**: každá klávesa podle oblasti a
    každý příkaz palety. V aplikaci totéž ukáže `?` a `Ctrl+P` najde libovolný
    příkaz.
22. **[Myš a gesta](mouse-and-gestures.md)**: klikání, přetahování, výběr a
    kopírování, posuvníky a zpětná vazba při najetí.

## Nastavení a integrace

23. **[Nastavení](settings.md)**: každá stránka a volba obrazovky Nastavení,
    vyhledávací pole, profily a jak přemapovat zkratky.
24. **[AI asistence](ai.md)**: vlastní AI GitPoulpi (ve výchozím stavu
    vypnutá): lokální agent jako `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama nebo LM Studio; instrukce; zprávy commitů, vysvětlení, popisy pull
    requestů a editor commitů; soukromí a řešení potíží.
25. **[Hostingové služby a issue trackery](hosting-integration.md)**: GitHub, GitLab, Gitea a Forgejo,
    Azure DevOps, Bitbucket, Gerrit a issue trackery: tokeny a jejich rozsahy, připojení, propojení repozitáře, klíče SSH, známá omezení. Také
    **[Lokální issues a Obsidian](local-issues.md)**: složka poznámek Markdown
    jako zdroj issues, bez tokenu, s Obsidian Base.

## Automatizace

26. **[Referenční příručka CLI](cli-reference.md)**: každý příkaz `gitpoulpi`, s `--json`
    pro skripty.
27. **[MCP a agenti](mcp-and-agents.md)**: nechte externího agenta (Claude
    Code, Cursor, Codex, ...) ovládat GitPoulpi: registrace serveru,
    `.mcp.json`, přepínače oprávnění a jejich výchozí hodnoty, každý nástroj a
    jak vám agent věci ukazuje ve vašem okně. Hotový skill agenta je
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Reference a nápověda

28. **[Referenční příručka konfigurace](config-reference.md)**: každý klíč `config.toml`,
    jeho výchozí hodnota a účinek, nastavení pro jednotlivé repozitáře,
    proměnné prostředí.
29. **[Řešení potíží a FAQ](troubleshooting.md)**: ikony, vykreslování grafu,
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
