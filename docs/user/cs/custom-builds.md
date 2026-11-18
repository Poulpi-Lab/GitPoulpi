[English](../custom-builds.md) · **Čeština** · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Vlastní buildy

Pro každého, kdo chce menší binárku: interaktivní skript se zeptá, které
jazyky, integrace s hostingem, motivy, barvy loga a funkce zahrnout, a pak
sestaví jen je s optimalizovaným profilem. Běžný build i oficiální vydání se
nemění (vše zapnuto).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd or PowerShell (scripts/build-custom.ps1)
```

Výstup skončí v `target/custom/`, nikdy přes oficiální `target/dist/`. Skript
vypíše velikost každé binárky a rozdíl oproti `target/dist/`, pokud ten build
existuje. Vedle binárek také zapíše `BUILD-FEATURES.txt`.

## Co si vybíráte

| Volba | Pokud ji vypnete |
|---|---|
| Jazyky | Jen angličtina (vždy součástí buildu) plus ty, které zaškrtnete. Výběr v Nastavení uvádí jen sestavené jazyky. |
| Motivy | GitPoulpi Dark, GitPoulpi Light a Terminal jsou vždy součástí buildu (`system` potřebuje Dark a Light). Každý další motiv se vybírá po rodinách (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` je vypíše). Nevybrané motivy se nekompilují: chybí v rozbalovacím seznamu v Nastavení i v `gitpoulpi theme list` a konfigurace, která takový motiv uvádí, se chová jako u neznámého motivu, s obvyklým varováním. |
| Barvy loga | Pixelová chobotnice v okně O aplikaci existuje ve 20 barevných provedeních (`--list-logo-colors`). Vyberte ta, která se zkompilují (alespoň jedno), výchozí (okno O aplikaci ho ukáže první; vrací se k němu i uložená hodnota `ui.logo_color`, která není zkompilována; musí patřit mezi zkompilovaná; oficiální výchozí: lavender) a barvu ikony spustitelného souboru pro Windows (libovolné provedení, výchozí: výchozí barva loga). S jediným provedením se logo po kliknutí stále otáčí, ale zůstává v téže barvě, a seznam barev (`c`, pravé tlačítko) se nenabízí. |
| Integrace (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Bez nich: jen klonování z URL; žádné issues, pull/merge requesty. |
| AI | Žádná tlačítka AI, příkazy ani nástroje `ai` v CLI/MCP. |
| MCP | Žádná binárka `gitpoulpi-mcp` a žádný příkaz `gitpoulpi mcp`. Agentní příkazy `ui` zůstávají. |
| LFS, Worktree, Submoduly | Odpovídající obrazovky a příkazy pro správu zmizí; vrací se typovaná odpověď „not built into this binary“. |
| Stahování avatarů | Jen odznaky s iniciálami, žádné stahování ze sítě. |
| Náhled obrázků | Soubory obrázků ukazují obyčejné upozornění na binární soubor; **Upravit v** stále funguje. |
| Radar | Žádná karta Radar, `Alt+R` ani příkaz palety; Radar se nikdy neotevře. Vyžaduje integraci (bez ní je stejně nečinný). |
| Mermaid | Žádné Zkopírovat jako Mermaid gitGraph (nabídka Logu, `Shift+Y`, paleta) a žádné `log --format mermaid` v CLI/MCP. Také žádné tlačítko **Otevřít diagram** pod blokem `mermaid` v [konverzaci](conversations.md) (blok zůstane jako kód) a žádný řádek palety *Konverzace: Otevřít diagram záznamu*. |
| Tabule | Žádné zobrazení tabule Kanban (tlačítko v záhlaví ISSUES, paleta) a žádný příkaz `hosting board`. Vyžaduje integraci. |
| Konverzace | Žádné zobrazení [konverzace](conversations.md) issue nebo žádosti v aplikaci: Enter na řádku issue, na kartě tabule nebo na řádku Radaru otevře prohlížeč jako dřív, není tu řádek *Otevřít konverzaci* ani příkazy palety *Konverzace: ...* a žádné **Napsat pomocí AI** v konverzaci (textová pole kontroly, formuláře issue a formuláře pull requestu si své ponechají). Konverzační příkazy CLI a MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) a `ai hosting-reply` zůstávají. Vyžaduje integraci. |
| Kontrola | Stránka Přehled kontroly je zobrazení konverzace, takže sestavení s Kontrolou obsahuje i zobrazení konverzace, ať říká *Konverzace* cokoli. Žádné zobrazení kontroly kódu v aplikaci (nabídka řádku, Enter na řádku pull requestu, paleta, `Kontrola` v Radaru) a žádná asistence AI při kontrole. Příkazy CLI/MCP `hosting review` zůstávají (používá je Gerrit a agenti). Vyžaduje integraci. |
| Nástroj pro přepis historie (pokročilý přepis historie: autoři, data, zprávy, obsah) | Žádný nástroj pro přepis historie: žádné příkazy palety `Přepis:`, žádný akord, klávesy ani sekce nápovědy nástroje, žádné řádky „Přepis historie“ v nabídkách Logu, žádné tlačítko „Upravit historii hlouběji...“ v dialogu Amend, žádné „Vytvořit pravidla pomocí AI“ ani AI pro skripty, žádné limity skriptů na stránce nastavení Synchronizace a žádné bloky AI pro přepis historie v nastavení AI. Příkazy CLI/MCP `rewrite`, `ai rewrite-script`, `ai rewrite-rules` a `ui propose rewrite` z příkazového stromu zmizí. Identity `.mailmap` (`mailmap`, `log.use_mailmap`) k němu nepatří a zůstávají. |
| Globální hledání | Žádné tlačítko Globální hledání v záhlaví, žádné `Ctrl+Alt+F`, žádný příkaz palety `Hledat: Pracovní prostor...` a žádný příkaz CLI/MCP `gitpoulpi search`. Vyhledávací lišta Logu (`ui show search`) je jiná funkce a zůstává. |
| Pracovní prostory | Okno pracuje s jediným výchozím seznamem karet („Žádný pracovní prostor“): žádný čip pracovního prostoru, žádná karta Pracovní prostory, žádné `Alt+W` / `Alt+J`, žádné příkazy palety `Pracovní prostor:`, žádné řádky karet „Přidat do pracovního prostoru“, žádné příkazy CLI/MCP `gitpoulpi workspace`, žádný přepínač `--workspace` u `commit` a `search`. Karty a relace fungují jako obvykle. Soubor `workspaces.toml` zapsaný jiným sestavením se nečte ani nemění; výchozí profil a instrukce AI pracovního prostoru se neuplatní. Radar a Globální hledání pokrývají otevřené karty. |
| Přepisovací skripty (vestavěné Lua) | Pravidla Script při přepisu historie jsou odmítnuta typovanou odpovědí „není součástí tohoto binárního souboru“ a žádné Lua se nelinkuje. Nástroj pro přepis historie samotný zůstává (má vlastní volbu). Bez významu, s varováním, pokud je nástroj pro přepis historie vypnutý. |

Vaše konfigurační soubory jsou mezi buildy sdílené: nastavení něčeho, co není
sestaveno, se zachová a ignoruje.

## Interaktivně

Skript pro Bash používá seznamy `whiptail` nebo `dialog` (tři obrazovky, pak
souhrn), nebo číslované výzvy, pokud není k dispozici ani jeden. Skript pro
PowerShell používá `Out-GridView`, je-li k dispozici, jinak výzvy. Vaše
předchozí volby jsou předem zaškrtnuté.

## Neinteraktivně

Jakýkoli argument přeskočí otázky a jde rovnou na
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Volba | Význam |
|---|---|
| `--langs fr,de` / `all` / `en` | Jazyky kromě angličtiny. Kódy jsou názvy souborů locale (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Poskytovatelé: GitHub, GitLab (včetně self-managed), Azure DevOps (Services a Server), Bitbucket (Cloud a Data Center) (hosting), Gitea a Forgejo včetně Codeberg (`gitea`) (hosting), Jira, Trello, Linear, YouTrack (Cloud a Server), Redmine (issue trackery), Gerrit Code Review (`gerrit`, jen kontrola), lokální issues v Markdown (`local-issues`, složka nebo vault Obsidian). Každý z `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` je samostatná cargo feature; velikosti v [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Rodiny motivů ke zkompilování (výchozí `all`). `--list-themes` vypíše rodiny. |
| `--logo-colors all` / `violet,teal` | Barevná provedení loga ke zkompilování (výchozí `all`, v pořadí tabulky, což je pořadí otáčení). `--list-logo-colors` je vypíše (`*` = oficiální výchozí). |
| `--logo-default teal` | Výchozí barevné provedení; musí patřit mezi zkompilovaná (výchozí: lavender, jinak první zkompilované). |
| `--icon-color sun` | Barva ikony spustitelného souboru pro Windows (výchozí: výchozí barva loga). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Každá je ve výchozím stavu zapnutá. |
| `--[no-]history-rewrite` | Nástroj pro přepis historie (pokročilý přepis historie: autoři, data, zprávy, obsah). Ve výchozím stavu zapnuto. |
| `--[no-]global-search` | Globální hledání (tlačítko v záhlaví, `Ctrl+Alt+F`, `gitpoulpi search`). Ve výchozím stavu zapnuto. |
| `--[no-]workspaces` | Pracovní prostory (pojmenované skupiny repozitářů). Ve výchozím stavu zapnuto; vypnuto zůstane jen výchozí seznam karet. |
| `--[no-]rewrite-scripts` | Vestavěné Lua pro pravidla Script při přepisu. Ve výchozím stavu zapnuto; s varováním ignorováno, pokud je nástroj pro přepis historie vypnutý. |
| `--save <name\|file>` | Uloží vyřešené volby jako uložený výběr (název nebo cesta k souboru). |
| `--preset <name\|file>` | Vyjde z uloženého výběru (název nebo cesta k souboru); ostatní volby jej přepíší, ať stojí na řádku kdekoli. |
| `--list-presets`, `--delete-preset <name>` | Vypíše uložené výběry a jejich složku / smaže jeden podle názvu. |
| `--last` | Znovu použije `target/custom-build/last-choices.env`; ostatní volby jej přepíší. |
| `--full` | Žádný výběr (vše zapnuto). |
| `--profile custom\|dist\|release` | Výchozí `custom` (nastavení `dist`, samostatný výstupní adresář). |
| `--dry-run` | Vypíše přesný příkaz cargo a skončí. |
| `-- <cargo args>` | Předá se cargu, například `--target x86_64-unknown-linux-musl`. |

Názvy v PowerShellu: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

Výběr motivů se do buildu dostává jako proměnná prostředí
`GITPOULPI_THEMES` (`all`, `none` nebo seznam rodin oddělený čárkami), kterou
čte build skript `gitpoulpi-config` a který při kompilaci ostatní palety
odstraní; nastavte ji sami pro prostý `cargo build`, chcete-li. `gitpoulpi
--version` vypíše řádek `themes:` se sadou, která byla sestavena.

Volby loga jsou proměnné prostředí `GITPOULPI_LOGO_COLORS` (`all` nebo seznam
slugů oddělený čárkami), `GITPOULPI_LOGO_DEFAULT` a `GITPOULPI_ICON_COLOR`,
které čtou build skripty `gitpoulpi-tui` (filtruje `crates/gitpoulpi-logo-build/colourways.json`
do tabulky, kterou okno O aplikaci vkládá) a tří binárních crates (ikona).
Zdrojem barev zůstává `crates/gitpoulpi-logo-build/colourways.json`.

### Ikona spustitelného souboru pro Windows

Když je cílem Windows, spustitelné soubory `gitpoulpi`, `gitpoulpi-tui` a
`gitpoulpi-mcp` dostanou jako ikonu chobotnici, generovanou při buildu (není
třeba Python ani ImageMagick) ve zvolené barvě ikony, velikosti 16 až 128 px.
Vkládá se pomocí crate `winresource` a potřebuje překladač prostředků platformy
(`rc.exe` z Windows SDK u MSVC, `windres` u GNU). Pokud chybí, build jen vypíše
varování a spustitelný soubor si ponechá výchozí ikonu. Na toolchainu GNU nese
ikonu jen `gitpoulpi` (prostředky by se jinak slinkovaly dvakrát). Buildy pro Linux a
macOS toto vše ignorují, kromě tabulky barev.

## Uložené výběry

Chcete-li sestavit novou verzi, aniž byste museli celý výběr dělat znovu, uložte ho pod názvem a později z něj vyjděte. Volby zadané spolu s `--preset` uložené hodnoty nadále přepisují.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Uložené výběry jsou obyčejné soubory `KLÍČ=hodnota` (formát `last-choices.env`, čitelný z shellu). Leží ve složce `custom-builds` v konfiguračním adresáři GitPoulpi, mimo `target/`, takže je `cargo clean` nesmaže:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` ve výchozím stavu)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` přesune tuto složku spolu s konfigurací. `--preset` a `--save` přijímají také cestu k souboru (hodnotu obsahující `/` nebo `\`, nebo končící na `.env`), abyste mohli výběr uchovávat v repozitáři dotfiles.

Výběr zapsaný jinou verzí GitPoulpi se přesto načte. Neznámý klíč, nebo jazyk, integrace, motiv či barva, které už neexistují, se ignorují s varováním. Klíč, který v souboru chybí (funkce přidaná později), dostane výchozí hodnotu, a později přidaná integrace se zahrne; obojí se hlásí na standardním chybovém výstupu.

Pokud existuje uložený výběr, interaktivní režim nejprve nabídne „Začít z“ (předchozí volby, uložený výběr nebo vše zapnuto) a na konci se zeptá, zda uložit aktuální výběr pod názvem.

## Kolik ušetří

Změřeno na Linuxu dne 2026-10-29 na commitu `65999ef5c`, od plného buildu, rozdíl oproti němu v závorkách
(podrobnosti v `docs/developer/binary-size.md`):

| Build (z `full`, pokud není uvedeno jinak) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Plný (vše zapnuto) | 64,6 | 50,5 | 31,7 |
| Jen angličtina | 52,0 (-12,6) | 40,2 (-10,3) | 31,7 (0,0) |
| MCP vypnuto | 61,6 (-3,0) | 49,6 (-0,8) | nesestaveno |
| AI vypnuto | 62,6 (-2,0) | 48,6 (-1,8) | 30,5 (-1,3) |
| Všechny integrace vypnuty | 56,3 (-8,2) | 43,7 (-6,7) | 26,7 (-5,0) |
| AI + integrace + stahování avatarů vypnuto | 52,2 (-12,3) | 39,9 (-10,6) | 23,0 (-8,8) |
| Nástroj pro přepis historie vypnut (`--no-history-rewrite`) | 61,6 (-3,0) | 48,9 (-1,5) | 28,5 (-3,2) |
| Přepisovací skripty vypnuty | 63,1 (-1,5) | 49,0 (-1,5) | 31,1 (-0,7) |
| Globální hledání vypnuto | 63,5 (-1,1) | 49,5 (-1,0) | 31,6 (-0,1) |
| Pracovní prostory vypnuty | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,2) |
| Radar vypnut | 63,4 (-1,2) | 49,3 (-1,2) | 31,7 (0,0) |
| Tabule vypnuta | 63,6 (-1,0) | 49,5 (-1,0) | 31,7 (0,0) |
| Kontrola vypnuta | 63,2 (-1,4) | 49,0 (-1,4) | 31,7 (0,0) |
| Mermaid vypnut | 63,6 (-1,0) | 49,5 (-0,9) | 31,6 (-0,1) |
| LFS vypnuto | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,3) |
| Worktree vypnuto | 63,7 (-0,9) | 49,6 (-0,8) | 31,7 (0,0) |
| Submoduly vypnuty | 63,7 (-0,9) | 49,6 (-0,9) | 31,6 (-0,1) |
| Stahování avatarů vypnuto | 63,7 (-0,9) | 49,6 (-0,9) | 31,7 (0,0) |
| Náhled obrázků vypnut | 63,3 (-1,2) | 49,2 (-1,3) | 31,7 (0,0) |
| Vše vypnuto, jen angličtina (LFS ponecháno) | 35,2 (-29,4) | 28,9 (-21,5) | nesestaveno |
| **Vše vypnuto, jen angličtina** | **34,9 (-29,7)** | **28,8 (-21,6)** | nesestaveno |

## Poznámky

- Vlastní build je pomalý jako každý build `dist` (thin LTO, několik minut).
- Běží bez sccache, stejně jako `build-opti`.
- Kódy jazyků se kontrolují proti dodaným locale; neznámý kód je odmítnut se
  seznamem platných.
