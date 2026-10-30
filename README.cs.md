[English](README.md) · **Čeština** · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="assets/logo/gitpoulpi-512.png" alt="Chobotnice GitPoulpi" width="128"></p>

**Git klient pro terminál, který se chová jako desktopová aplikace.**
GitPoulpi nabízí plnohodnotný graf commitů, na který lze klikat, který lze
přetahovat a otevírat pravým tlačítkem, paletu příkazů ovládanou z klávesnice,
jež dosáhne na každou funkci, integraci s hostingy kódu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; v cloudu i self-managed) a s issue trackery (Jira, Trello, Linear, YouTrack, Redmine nebo složkou poznámek v Markdownu) a stejné
operace jako skriptovatelné CLI a jako MCP server, takže jej AI agent ovládá
úplně stejně jako vy. GitPoulpi je inspirován GitKrakenem a od základu
přepsán v Rustu: nativní binární soubory, nenáročné na disk i paměť, bez Electronu, bez účtu a bez telemetrie. Běží ve vašem oblíbeném terminálu ([v kterém?](docs/user/cs/terminals.md)), protože někteří z nás v něm bydlí.

![GitPoulpi zobrazující graf commitů s větvemi, tagy a stashem, vlevo postranní panel Refs a vpravo rozpracované změny s formulářem commitu](docs/user/images/hero.png)

- **Myš a klávesnice zcela rovnocenně.** Klikejte, dvakrát klikejte, otevírejte
  pravé tlačítko a přetahujte refs na commity; nebo stiskněte `Ctrl+P` a
  napište, co chcete. Každé gesto myši má klávesový protějšek a každý příkaz je
  v paletě.
- **Skutečný graf commitů.** Větve, merge, tagy, stashe i pracovní kopie v
  jednom posouvaném Logu, vykreslené jako pixely v terminálech, které to
  umějí, a jako znaky všude jinde. Mnoho tmavých i světlých barevných motivů,
  avatary autorů, hledání.
- **Přesný stage.** Přidejte do stage, odeberte ze stage nebo zahoďte soubor,
  složku, hunk či jediný řádek přímo z diffu. V jednom sloupci nebo vedle
  sebe, s minimapou.
- **Bezpečné úpravy historie.** Upravte libovolný commit (zprávu, autora,
  data), proveďte interaktivní rebase s kontrolní obrazovkou, přepište mnoho
  commitů najednou s náhledem a jedním vrácením, proveďte cherry-pick několika
  commitů, fast-forward, vytvářejte worktrees a patche.
- **Zabudované hostingové služby a issue trackery.** Procházejte, stahujte (checkout) a vytvářejte pull a merge requesty (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) a kontrolujte změny v Gerritu. Procházejte a vytvářejte issues, work items a karty (také Jira, Trello, Linear, YouTrack, Redmine a lokální poznámky v Markdownu), chcete-li, na tabuli Kanban. Klonujte ze svého účtu a spravujte klíče SSH.
- **CLI a MCP pro agenty.** `gitpoulpi <příkaz> --json` pokrývá vše, co dělá
  rozhraní, a `gitpoulpi-mcp` to nabízí jako nástroje MCP. Jeden démon na
  pozadí pro každý repozitář udržuje všechna rozhraní živá a synchronizovaná.
- **Volitelná AI, nikdy povinná.** Ve výchozím stavu vypnutá a bez účtu:
  použijte vlastní API klíč nebo lokální agentní CLI, například `claude -p`.
  Generujte zprávy commitů, vysvětlujte změny, pište popisy pull requestů,
  rozdělte rozpracované změny do několika commitů a nechte si navrhnout pravidla
  pro přepis historie.
- **Git LFS, submoduly, worktrees, patche, blame a historie souboru, diffy
  obrázků, zabudovaný editor.**

## Snímky obrazovky

| | |
|---|---|
| ![Graf Logu s barevným motivem Material a zapnutým podbarvením řádků](docs/user/images/theme-material.png) **Graf commitů**, zde v motivu Material s podbarvením řádků. | ![Diff změněného souboru s tlačítky Stage hunk a Zahodit hunk u každého hunku](docs/user/images/diff.png) **Diff** se stage a zahozením po hunku nebo po řádku. |
| ![Paleta příkazů filtrovaná na slovo stash](docs/user/images/palette.png) **Paleta příkazů** (`Ctrl+P`) dosáhne na každý příkaz. | ![Editor commitů navrhující čtyři commity pro rozpracované změny, připravené ke kontrole](docs/user/images/composer.png) **Editor commitů** dá ke kontrole commity navržené AI, než se cokoli commitne. |
| ![Nabídka pravého tlačítka u commitu s akcemi cherry-pick, revert, reset, rebase, větev a tag](docs/user/images/context-menu.png) **Pravé tlačítko** na čemkoli otevře jeho akce. | ![Stránka AI v Nastavení s profily, instrukcemi pro jednotlivé akce a přepínačem zapnutí](docs/user/images/settings-ai.png) **Nastavení** s hledáním, včetně volitelné stránky AI. |

Další obrázky s vysvětlením najdete v [uživatelské dokumentaci](docs/user/cs/README.md).

## Instalace

Potřebujete systémový `git`; `git-lfs` je volitelný.

- **S instalátorem:** `...-setup.exe` pro Windows nebo `install.sh` pro Linux a macOS (pro uživatele, bez root), přiložené ke každému vydání. Sestavení nainstalované z archivu vydání se umí aktualizovat samo z čipu verze ve stavovém řádku.
- **Z archivu vydání:** rozbalte jej a postupujte podle `INSTALL.txt` uvnitř.
- **Ze zdrojových kódů:**

  ```
  git clone https://github.com/Poulpi-Lab/GitPoulpi.git
  cd GitPoulpi
  cargo build --release
  ```

  Binární soubory skončí v `target/release/`: `gitpoulpi` (CLI, a bez
  podpříkazu také rozhraní), `gitpoulpi-tui` a `gitpoulpi-mcp`.
- **Vlastní archiv vydání:** `cargo run -p xtask -- dist` vytvoří
  `target/dist/gitpoulpi-<version>-<target>.zip` (Windows) nebo `.tar.gz` se
  souborem `.sha256`.
- **Menší binární soubor jen s tím, co potřebujete:** `./build-custom.sh`
  (Windows: `build-custom`) se zeptá, které jazyky, integrace a funkce (AI, MCP,
  LFS, worktrees, submoduly, stahování avatarů) sestavit; viz
  [docs/user/cs/custom-builds.md](docs/user/cs/custom-builds.md).

Úplné podrobnosti, volitelné nástroje a poznámky pro jednotlivé systémy:
[docs/user/cs/installation.md](docs/user/cs/installation.md).

## Rychlý start

```
cd path/to/your/repository
gitpoulpi-tui          # or just: gitpoulpi
```

- `?` otevře nápovědu se všemi klávesami a gesty, `Ctrl+P` paletu příkazů,
  `Ctrl+,` Nastavení, `q` program ukončí.
- Kliknutím na commit zobrazíte jeho změny; kliknutím na horní řádek Logu
  (rozpracované změny) můžete přidávat soubory do stage a commitovat.
- V terminálu, který to podporuje (kitty, Ghostty, WezTerm, iTerm2 (a také Windows Terminal a foot přes sixel)), se graf
  kreslí jako pixely. V ostatních (běžná konzole Windows, tmux) se
  automaticky přepne na znaky.

Postup krok za krokem je v části [První kroky](docs/user/cs/getting-started.md)
a [prohlídka rozhraní](docs/user/cs/interface-tour.md) pojmenovává každou oblast
obrazovky.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

AI je volitelná a ve výchozím stavu vypnutá: vlastní AI GitPoulpi zapnete v
*Nastavení > AI* ([AI asistence](docs/user/cs/ai.md)); externímu agentovi
dovolíte GitPoulpi ovládat přes [MCP a agenty](docs/user/cs/mcp-and-agents.md)
(oprávnění, každý nástroj) a skill agenta
[`skills/gitpoulpi/SKILL.md`](skills/gitpoulpi/SKILL.md).

## Dokumentace

Kompletní uživatelská dokumentace začíná na **[docs/user/cs/README.md](docs/user/cs/README.md)**:

| | |
|---|---|
| [První kroky](docs/user/cs/getting-started.md) | Instalace, otevření repozitáře, první commit |
| [Prohlídka rozhraní](docs/user/cs/interface-tour.md) | Každá oblast okna s názvem |
| [Práce s Logem](docs/user/cs/log-and-history.md) | Graf, refs, větve, tagy, stashe, rebase, cherry-pick |
| [Stage a commit](docs/user/cs/staging-and-committing.md) | Stav, diff, hunky a řádky, editor commitů, editor |
| [Nastavení](docs/user/cs/settings.md) | Každá stránka a volba Nastavení |
| [AI asistence](docs/user/cs/ai.md) | Nastavení zabudované AI, každá akce AI, soukromí |
| [Hostingové služby a issue trackery](docs/user/cs/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokeny, propojení repozitáře, pull a merge requesty |
| [CLI](docs/user/cs/cli-reference.md) a [MCP](docs/user/cs/mcp-and-agents.md) | Skriptování, nebo ovládání agentem (oprávnění, každý nástroj) |
| [Klávesové zkratky](docs/user/cs/keybindings.md) (a každý příkaz palety), [myš](docs/user/cs/mouse-and-gestures.md), [konfigurace](docs/user/cs/config-reference.md), [LFS](docs/user/cs/lfs.md), [zpět](docs/user/cs/undo-redo.md), [řešení potíží](docs/user/cs/troubleshooting.md) | Reference a nápověda |

Vývojáři: [CLAUDE.md](CLAUDE.md) a `docs/architecture/overview.md` popisují
crates a návrh; [CHANGELOG.md](CHANGELOG.md) uvádí, co je v jednotlivých
vydáních.

## Licence

[MIT](LICENSE).

## Poděkování a licence třetích stran

GitPoulpi staví na open-source knihovnách a používá barevné hodnoty z publikovaných motivů. Jejich licence a poděkování, včetně autorů motivů, jsou v [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt). Soubor je součástí každého vydání; zobrazí ho také okno O aplikaci a příkaz `gitpoulpi licenses`.

## Ochranné známky

GitKraken je ochranná známka svého příslušného vlastníka. GitPoulpi je nezávislý projekt, který není s tímto vlastníkem spojen ani jím schválen či sponzorován; název je uveden pouze pro popis kompatibility a inspirace.
