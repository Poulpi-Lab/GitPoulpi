[English](../config-reference.md) · **Čeština** · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Referenční příručka konfigurace

Každý klíč v `config.toml`, jeho výchozí hodnota a co dělá. Soubor je jen
zřídka potřeba upravovat ručně: [obrazovka Nastavení](settings.md) (`Ctrl+,`) ho
zapisuje při každé vaší změně a totéž dělá `gitpoulpi config set`.

## Kde je a jak se hodnota vybírá

| | |
|---|---|
| Soubor | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` ho vypíše. |
| Jiná složka | `GITPOULPI_CONFIG_DIR=<dir>` přesune celou konfigurační složku (soubor, `sessions/`, soubory s porty daemona): přenosná instalace, test nebo agent držený stranou od vašeho nastavení. |
| Přednost | vestavěná výchozí hodnota, pak `config.toml`, pak proměnná `GITPOULPI_*`, pak přepínač příkazové řádky. Vyhrává nejvyšší. Chybná hodnota (překlep ve volbě v souboru, špatná proměnná) je chyba, nikdy se tiše neignoruje. |
| Proměnné | Klíč napište velkými písmeny a tečky změňte na podtržítka: `general.auto_prune` je `GITPOULPI_GENERAL_AUTO_PRUNE`. Jen pro klíče, které GitPoulpi skutečně čte (ne ty označené níže *nepoužívá se*, ani seznamy jako `ui.log_columns_hidden`). |
| Přepínače | `--backend git2\|system` přebíjí `general.default_backend` pro daemona, kterého tento příkaz spustí. |
| Tokeny a klíče | Nikdy v tomto souboru. Tokeny hostingu a klíče API AI jdou do systémového credential helperu (Git Credential Manager, libsecret / `pass`, Keychain); soubor obsahuje jen odkaz (`token_ref`). Bez helperu uložení tokenu selže se zprávou a token vydrží jen po dobu relace. |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set` udržuje soubor řídký (jen klíče, které někdo nastavil) a zachovává
klíče, na které se neptáte, ale zahazuje komentáře. Odmítá seznamy a celé
tabulky, kromě `keybindings.*` (více akordů oddělených mezerami; prázdné zruší
vazbu). `[[profiles]]`, `[[integrations]]` a `[[ai.profiles]]` upravujte v
Nastavení nebo ručně.

V tabulkách **Restart** znamená, že se hodnota čte při spuštění; vše ostatní se
projeví okamžitě (nebo při dalším obnovení, fetch či zavření Nastavení).

## `[general]`

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, pro staging, merge, rebase, stash a síť se vrací k programu `git`) nebo `system` (jen nainstalovaný `git`). Běžící daemon si backend ponechává: **restartujte** ho (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Když místní změny blokují checkout, pull, merge nebo rebase: `ask` (dialog v TUI; CLI a MCP selžou a uvedou `--autostash`), `always` (stash, provedení, pop) nebo `never` (odmítnutí). Viz [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | Auto-stash bere i nesledované soubory. |
| `general.initial_commits_in_graph` | `5000` | Commity, které Log načte jako první (alespoň 500). Vyšší hodnota stojí paměť a čas vykreslování u obřích repozitářů. |
| `general.lazy_load_commits_in_graph` | `true` | Načítá starší commity při posunu ke konci. Vypnuto: Log končí u počátečního počtu. |
| `general.commits_per_lazy_page` | `1000` | Commitů na jednu stránku načtenou na pozadí (alespoň 50). |
| `general.max_commits_in_graph` | `100000` | Kde líné načítání končí (přibližně 300 bajtů na commit); `0` = bez omezení. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch všech remote na pozadí každých N minut, 0 až 60; `0` = vypnuto. **Restart.** |
| `general.auto_fetch_interval_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje interval: `minutes`, `hours` nebo `auto`. Jen zobrazení: interval je vždy `auto_fetch_interval_minutes` minut. `auto` (a konfigurace bez tohoto klíče) vybere největší jednotku, která minuty dělí beze zbytku (60 se čte „1 hodina“, 45 „45 minut“); jednotka, která nesedí, se ignoruje. Nastavení zapisuje oba klíče; starší GitPoulpi jednotku ignoruje a minuty čte dál. |
| `general.fetch_on_tab_open` | `true` | Fetch všech remote při otevření karty (přeskočí se, pokud GitPoulpi tento repozitář načetl před méně než 60 s). |
| `general.auto_prune` | `true` | Fetch také odstraní remote-tracking větve, které zmizely. |
| `general.auto_prune_tags` | `false` | Fetch také smaže lokální tagy, které remote smazal (jen tagy, které na něm byly; tag, který jste nikdy neodeslali, zůstane). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Obnovení seznamů pull requestů / issues a čipu na panelu nástrojů, 0 až 120; `0` = jen na požádání a při událostech. |
| `general.hosting_refresh_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje obnovení: `minutes`, `hours` nebo `auto`, čte se jako `general.auto_fetch_interval_unit`. Jen zobrazení. |
| `general.hosting_post_as` | `""` | Jméno, pod kterým se zprávy odesílají hostiteli, když je token repozitáře botův (token GitHub App), např. `my-app[bot]`: zobrazuje se v každém textovém poli a slouží k rozpoznání vašich vlastních komentářů. Prázdné: zeptá se hostitele, jinak se naučí z prvního odeslání. Repozitář ho může přepsat (Nastavení, SSH a přihlášení repozitáře). |
| `general.keep_submodules_up_to_date` | `false` | Po checkoutu nebo pull spustí `git submodule update --init --recursive`, když je submodul mimo synchronizaci. Viz [Submoduly](submodules.md). |
| `general.default_branch_name` | `"main"` | První větev repozitáře vytvořeného pomocí Init; prázdné nechá rozhodnout git. Nic nepřejmenovává. |
| `general.delete_orig_files_after_merge` | `false` | Po merge, který se dokončí, smaže záložní soubory `*.orig`. |
| `general.remember_tabs` | `true` | Znovu otevře karty z poslední relace. Když je vypnuto, spuštění uvnitř repozitáře otevře jen tento repozitář a spuštění mimo repozitář otevře jen kartu Domů. **Restart.** |
| `general.radar_enabled` | `true` | [Radar](radar.md): jeho karta, příkazy palety, `Alt+R` a načítání na pozadí. Vypnuto skryje všechno a nic se pro něj nenačítá. Platí okamžitě; v sestavení bez funkce `radar` se nezobrazuje. |
| `general.startup_tab_in_repo` | `"current-repo"` | Karta, na které se aplikace otevře, když složka, v níž se spouští, leží uvnitř repozitáře: `"current-repo"` (karta tohoto repozitáře, v případě potřeby otevřená), `"last"` (karta aktivní při posledním ukončení), `"home"` nebo `"radar"`. Repozitář zadaný přes `--repo` vždy otevře vlastní kartu. `"last"` je Domů při vypnutém `remember_tabs` nebo když si nic nepamatuje; `"radar"` je Domů, když je Radar vypnutý nebo není v sestavení. Čte se jednou při startu; vyžaduje restart. |
| `general.startup_tab_outside_repo` | `"home"` | Karta, na které se aplikace otevře, když složka, v níž se spouští, není repozitář (například uživatelská složka): `"home"`, `"last"` nebo `"radar"`. `"last"` je Domů při vypnutém `remember_tabs` nebo když si nic nepamatuje; `"radar"` je Domů, když je Radar vypnutý nebo není v sestavení. Čte se jednou při startu; vyžaduje restart. |
| `general.startup_tab_launcher` | `"last"` | Karta, na které se aplikace otevře, když ji spustí zástupce na ploše nebo v nabídce Start, který GitPoulpi vytvořil (předává `--launcher`), ať je složka jakákoli: `"home"`, `"last"` nebo `"radar"`. Má přednost před dvěma nastaveními výše. `"last"` je Domů při vypnutém `remember_tabs` nebo když si nic nepamatuje; `"radar"` je Domů, když je Radar vypnutý nebo není v sestavení. Čte se jednou při startu; vyžaduje restart. |
| `general.group_worktree_tabs` | `true` | Karty propojených worktree repozitáře drží hned za kartou tohoto repozitáře: při otevření worktree, při odtažení karty worktree (vrátí se ke své skupině; karta repozitáře je hlavou skupiny a její worktree ji následují) a při zapnutí tohoto nastavení. Vypnuto nechá karty, kam je dáte. |
| `general.undo_depth` | `100` | Kolik operací může Zpět vrátit; `0` = neomezeně. |
| `general.conflict_detection` | `true` | Proaktivní detekce konfliktů: odznak v Logu, když by merge aktuální větve do větve, ze které vznikla, skončil konfliktem; počítá se na pozadí (merge v paměti: vaše soubory, index ani refy se nemění). Repozitář ji může pro sebe vypnout (`[preferences] conflict_detection`). Základní větve, se kterými se porovnává, mají vlastní seznamy: [`[conflict_prevention]`](#conflict_prevention-seznamy-základních-větví). Viz [Řešení konfliktů při slučování](merge-conflicts.md#prevence-konfliktů). |
| `general.dev_mode` | chybí (automaticky) | Pro ty, kdo GitPoulpi sestavují z klonu repozitáře; v Nastavení se nikdy nezobrazuje. Chybí: zapnuto, když spuštěný program leží v `<checkout>/target/<profile>/` (`Cargo.toml` a `crates/gitpoulpi-tui` vedle `target`). `true` / `false` ho vynutí. Režim vývojáře odemyká jen `ui.launcher_build`. |
| `general.journal_outside` | `true` | Zaznamenává, co shell nebo AI agent udělá s větvemi, tagy, stashi a HEADem, jako položky historie vrácení označené jako externí, takže je `Ctrl+Z` může vzít zpět. Úpravy souborů se nikdy nezachycují. Vyžaduje `undo_enabled`. Viz [Zpět a znovu](undo-redo.md#změny-provedené-mimo-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: žádná evidence pro vracení (nejrychlejší), historie vracení se smaže, tlačítka Zpět/Znovu zmizí. Viz [Zpět a znovu](undo-redo.md). |
| `general.active_profile` | `""` | Id identity z `[[profiles]]`, se kterou se vytvářejí commity. Prázdné, nebo profil s prázdným jménem či e-mailem: `user.name` / `user.email` gitu. |
| `general.external_editor` | `""` | Příkaz pro položku souboru *Otevřít v externím editoru*, např. `code -r {file}`. Prázdné: VS Code, je-li nainstalován, jinak výchozí program systému. |
| `general.external_diff_tool` | `""` | Příkaz pro *Otevřít v externím nástroji diff*: `{local}` (commitnutá / HEAD verze), `{remote}` (verze z pracovního stromu / indexu). Prázdné: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Příkaz pro *Otevřít v externím nástroji merge* na kartě konfliktu (binární, LFS nebo příliš velký soubor): `{base}`, `{local}` (A, aktuální), `{remote}` (B, příchozí) a `{merged}` (soubor, do kterého nástroj zapíše výsledek; GitPoulpi ho přidá do indexu, když se nástroj ukončí úspěšně). Prázdné: `git mergetool --no-prompt`. |
| `general.file_explorer` | podle OS | Příkaz pro *Zobrazit ve složce*: `{path}` (soubor), `{dir}` (jeho složka). Výchozí: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Tři šablony příkazů se nejprve rozdělí na argumenty (`"..."` seskupí slovo) a
teprve poté se doplní zástupné symboly, takže cesty s mezerami zůstanou jedním
argumentem. Žádný shell se nepoužívá.

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log a graf**

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (obrázek tam, kde ho terminál umí nakreslit, jinak znaky), `image`, `text`. **Restart.** |
| `ui.image_protocol` | `"auto"` | Jak se obrázky (graf, avatary) dostanou do terminálu: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protokol Kitty v kitty, Ghostty, iTerm2 3.7.7 a novějším a v herdr s `kitty_graphics = true`; protokol iTerm2 ve WezTerm a Rio (průhlednost se míchá), starším iTerm2 a mintty (neprůhledné pozadí); sixel (sloupec grafu i avatary) ve Windows Terminal, foot, Tabby, Konsole, mlterm, v tmux, když terminál za ním sixel umí, a v každém terminálu, jehož odpověď při startu sixel uvádí (xterm spuštěný s `-ti vt340`, terminál VS Code se zapnutými obrázky); znaky v zellij, screen a neznámých terminálech. V tmux vyžadují obrázky kitty zde `kitty` plus `allow-passthrough on` ([Multiplexery](multiplexers.md)). Kde pseudokonzole terminálu obrázky zahazuje nebo špatně umisťuje (WezTerm 20240203, Rio a mintty ve Windows), použijte `ui.graph_renderer = "text"` ([Výběr terminálu](terminals.md#windows)). **Restart.** |
| `ui.theme` | `"system"` | Barevný motiv: vestavěný slug (`gitpoulpi theme list` vypíše všechny), slug souboru v `<config>/themes/`, nebo `system` (řídí se světlým či tmavým pozadím terminálu zjištěným při spuštění; stará hodnota `default` znamená totéž). `config set` odmítne neznámé názvy. Viz [Motivy](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Paleta drah: `theme` (podle motivu) nebo libovolný název palety drah: `gitpoulpi`, `light`, `material`, `serie` a palety ostatních motivů. |
| `ui.graph_lane_background` | `"off"` | `off` nebo `row_tint` (slabý pruh barvy dráhy za grafem a lišta na jeho okraji). **Aplikace: Přepnout zbarvení řádků grafu** (`Ctrl+N Shift+Y`) ho přepíná; **Aplikace: Změnit barevný motiv grafu...** (`Ctrl+N Shift+T`) volí `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (jen s `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` a `right` kreslí sloupec; okno užší než 80 sloupců se vrátí na `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: zda dialog, který jste přesunuli nebo jemuž jste změnili velikost (tažením za záhlaví nebo okraje, nebo `Ctrl+N m`), drží své místo. Jedno nastavení pro všechny dialogy (každý si pod ním drží své vlastní místo a velikost): do zavření GitPoulpi (výchozí), na disku pro příští spuštění (všechny dialogy v jednom `dialog_geometry.txt` ve složce s daty), nebo vůbec (každý dialog se otevře na výchozím místě a ve výchozí velikosti). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: kde leží tlačítka podél panelu (vlevo, uprostřed, vpravo v řádku; nahoře, uprostřed, dole ve sloupci). |
| `ui.toolbar_pr_pinned` | `true` | Tlačítko pull requestu je připnuté na konci panelu nástrojů; `false` ho nechá plynout spolu s ostatními tlačítky. |
| `ui.statusbar_position` | `"bottom"` | `top` (nad kartami repozitářů) nebo `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (poslední řádek okna, pod stavovým řádkem) nebo `left` (sloupec u levého okraje; pod 80 sloupců zůstává nahoře). |
| `ui.graph_show_stashes` | `true` | Řádek pro každý stash v Logu. |
| `ui.ref_connector` | `true` | Tenká čára od štítku refu k jeho commitu. |
| `ui.column_resize` | `"line"` | Jak se chytá hranice sloupce v Logu, v Radaru a ve sloupci commitů zobrazení diff, Soubor a blame. `line`: každá hranice je jednobuňková mezera s malou značkou v záhlaví; dokud je ukazatel na hranici, v záhlaví nebo kdekoli dole v tabulce, vykreslí se přes celou výšku tenká akcentní čára a tažení odkudkoli z ní mění šířku. `header`: bez mezery a bez značky; rozsvítí se první znak buňky záhlaví za hranicí a tažení odtud mění šířku (jen v záhlaví). Klávesy pro změnu šířky a řádky v paletě jsou v obou režimech stejné. |
| `ui.ref_tree` | `true` | Seskupí větve a tagy podle `/` do složek v postranním panelu Refs (přepíná `y`). |
| `ui.graph_ghost_ref` | `true` | Sloupec Ref v Logu ukazuje vybledlou pilulku s názvem větve (jinak vzdálené větve), do které patří najetý commit, pokud commit nemá vlastní ref a ve sloupci je místo; nikdy tag. Smazaná větev se odvodí z předmětu slučovacího commitu, který ji sloučil, a kreslí se přeškrtnutě. Najetí na pilulku ukáže její druh. |
| `ui.reveal_on_ref_click` | `true` | Jedno kliknutí na ref (větev, tag, vzdálenou větev, stash) v panelu Refs posune Log na jeho commit a řádek krátce zvýrazní; výběr commitu se nezmění. |
| `ui.log_wip_counts_in_message` | `true` | Počty změn WIP ve zprávě WIP, když je sloupec Změny skrytý. |
| `ui.log_columns_layout` | `"fit"` | Jak Log rozkládá své sloupce. `fit`: každý viditelný sloupec se do panelu vejde přesně a pružný je jen `message`; když je panel úzký, celé sloupce ustoupí. `scroll`: sloupce si drží šířky (i `message`), nic neustupuje a tabulka se posouvá do stran, když je širší než panel. |
| `ui.log_column_widths_scroll` | prázdné | Zastaralé: šířky sloupců Logu jsou nově pro každý repozitář, v jeho `repo-state.toml` (`[log_columns]`, mapy `fit` a `scroll`, název -> buňky; `scroll` může obsahovat `message`, `fit` nikdy). Repozitář, který nemá nic uloženo, použije zastaralé `ui.log_column_widths` / `ui.log_column_widths_scroll`, pokud je máte, jinak se při prvním otevření přizpůsobí obsahu. |
| `ui.log_columns_hidden`, `ui.log_columns_order` | prázdné | Zobrazené sloupce Logu (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`) a jejich pořadí, zapisují se při skrytí nebo přetažení sloupce; sdílí je všechny repozitáře. `message` nelze skrýt. `ui.log_column_widths` je zastaralé (viz výše). |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | prázdné | Části [sloupce commitů](interface-tour.md#sloupec-commitů) vedle diffu, samostatně a nezávisle na Logu: zobrazené části podle názvu (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; prázdné znamená dráhy, zprávu, autora a datum), jejich pořadí zleva doprava stejnými názvy (prázdné je právě vypsané pořadí; vynechaný název následuje v tomto pořadí) a přetažené šířky v buňkách pro každou část (vynechaná část je automatická); zapisují se, když části zvolíte, přesunete nebo jim nastavíte velikost. `message` nelze skrýt. Soubor relace zapsaný před vznikem těchto klíčů se do nich jednou načte. |
| `ui.sidebar_sections_hidden` | prázdné | Sekce levého panelu, které jste skryli (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`); zapisuje se z nabídky pod pravým tlačítkem na panelu nebo z palety. Alespoň jedna existující sekce zůstane vždy zobrazena. Toto je výchozí hodnota: každý repozitář může skrýt vlastní sekce (uložené v jeho souboru stavu) a tento klíč platí pro repozitář bez vlastní volby; zapíše ho *Použít pro všechny repozitáře*. |
| `ui.sidebar_sections_scope` | `"repository"` | Čí volba určuje sekce levého panelu: `repository` (každý repozitář má vlastní, `ui.sidebar_sections_hidden` je výchozí) nebo `global` (každý repozitář ukazuje `ui.sidebar_sections_hidden` a změna se zapíše tam). Přepínání nikdy neztratí žádnou ze sad: volby jednotlivých repozitářů zůstávají v jejich souborech stavu, dokud je zapnuto `global`. Nastavení: *Sekce levého panelu*. |
| `ui.language` | `"auto"` | Jazyk rozhraní: `auto` (národní prostředí systému, jinak angličtina), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Živě.** Přebíjí ho `GITPOULPI_LANG`. Jazyk, který v tomto sestavení není, zobrazuje angličtinu. Nastavuje ho i **Aplikace: Změnit jazyk...** (`Ctrl+N l`), se živým náhledem. Výstup CLI a MCP zůstává anglicky. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... zůstávají ve všech jazycích anglicky. `true`: slovník gitu zvoleného jazyka (francouzsky „remiser“, „étiquette“, „valider“...). **Živě.** |
| `ui.git_terms_english` | `false` | Jen ja, ko a zh-CN: zobrazí anglická slova gitu (stage, unstage, staged, unstaged...) místo jejich přepisu. Nemá účinek, dokud je `ui.translate_git_terms` `true`. **Živě.** |
| `ui.mouse_enabled` | `true` | `false` ponechá myš (a výběr textu) terminálu. **Restart.** |
| `ui.mouse_scroll_lines` | `3` | Řádků Logu na jedno cvaknutí kolečka, 1 až 20. **Restart.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` požádá terminál o klávesnicový protokol kitty (takže dorazí `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`; jen Unix, tmux neodpovídá nikdy: viz [Multiplexery](multiplexers.md#tmux)); `off` nepožádá nikdy. **Restart.** |
| `ui.logo_color` | `""` | Barevné provedení loga v okně O aplikaci, slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); prázdné nebo neznámé znamená výchozí hodnotu sestavení (`lavender`). Zapisuje se kliknutím na logo nebo výběrem barvy v O aplikaci. |

**Avatary** (ve výchozím stavu vypnuty; jejich zapnutí odešle hash SHA-256
e-mailů autorů službě Gravatar a zeptá se vašeho připojeného účtu
GitHub/GitLab)

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (v kolečku commitu obrázkového grafu), `author` (ve sloupci Autor), `off`. Starý `ui.show_avatars` (`true`) se čte jako `author`, dokud `ui.avatars` chybí. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Pořadí vyhledávání: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Hostitel GitLab se dotazuje jen přes připojený profil. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (generovaný lokálně) nebo `initials`. Na terminálu bez protokolu obrázků se vždy použije odznak s iniciálami. |

**Zobrazení diffu a editoru**

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` nebo `split` (`m` v diffu). |
| `ui.diff_hunks_only` | `true` | `false` zobrazí celý soubor se změnami na svých místech (`H`). |
| `ui.diff_show_whitespace` | `true` | Zobrazí změny jen v bílých znacích (`s`). |
| `ui.diff_word_wrap` | `false` | Zalamuje dlouhé řádky diffu (`w`; Nastavení > Diff a editor). |
| `ui.minimap` | `false` | Minimapa vedle posuvníku diffu a editoru (`M`). |
| `ui.mermaid_detail` | `refs` | Úroveň detailu Log: Zkopírovat jako Mermaid gitGraph: `structure`, `refs`, `subjects`, `full` (pamatuje se z vyskakovacího okna; výchozí hodnota `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Směr tohoto grafu: `lr` (zleva doprava) nebo `tb` (shora dolů). |
| `ui.mermaid_fence` | `false` | Obalit zkopírovaný skript Mermaid blokem kódu `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Předměty commitů se v id Mermaid zkrátí na tolik znaků (`subjects`, `full`). |
| `ui.text_cursor` | `"terminal"` | Kde se ukazuje kurzor aktivního textového pole: `terminal` tam umístí vlastní kurzor terminálu (blikající svislou čárku, takže se jím řídí nastavení kurzoru terminálu i okno s návrhy metody zadávání; skrytý, když klávesnici nemá žádné pole, a při ukončení obnovený na váš vlastní styl kurzoru), `drawn` vykreslí vlastní znak kurzoru GitPoulpi a kurzor terminálu nechá skrytý. Okamžitě. |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` nebo `16`: kolika barvami se kreslí. `auto` je truecolor, kromě Apple Terminal (256), konzole Linuxu (16) a terminálů `*-256color` bez `COLORTERM` (256). Živě. Viz [Motivy](themes.md). |
| `ui.input_color` | `"theme"` | Pozadí textových polí: `theme` (vlastní z motivu), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | Barva `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | AI composer commitů se otevře se soubory každého návrhu `expanded` (rozbalenými) nebo `collapsed` (sbalenými). |
| `ui.blame_side` | `"right"` | Na které straně zobrazení Blame je sloupec anotací (avatar, autor, stáří, shrnutí): `right` (výchozí) nebo `left`. |
| `ui.blame_width` | `46` | Šířka sloupce anotací v Blame v buňkách (24 až 90). Přetáhněte jeho okraj nebo stiskněte v zobrazení `<` / `>`; nová šířka se uloží sem. |
| `ui.pr_terminology` | `"auto"` | `auto` (Pull Request na GitHubu, Merge Request na GitLabu), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Skryje čip na panelu nástrojů s nejnovější žádostí o sloučení (pull nebo merge request) větve, která je právě v checkoutu, jakmile je zavřená nebo sloučená; místo něj se zobrazí tlumený čip „+ PR“ (založit novou), jako by větev žádnou neměla. `false`: čip vždy ukazuje nejnovější žádost bez ohledu na její stav. |

Přepínače diffu, které změníte uvnitř diffu, se zde okamžitě uloží a použijí se
ve všech diffech otevřených později.

**Formáty data**: `ui.date_format` (`"YYYY-MM-DD"`, sloupec s datem v Blame / Historii souboru),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, sloupec Datum v Logu) a
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, podrobnosti commitu, tooltip
Datum a *Kopírovat > Datum*). Tokeny: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (den v týdnu, plný / zkrácený) `HH` `mm` `ss`; cokoli jiného se
zkopíruje beze změny. Názvy měsíců a dnů v týdnu se řídí `ui.language` a měsíc
zapsaný za číslem dne je v genitivu, má-li jej daný jazyk (ruština, ukrajinština,
polština, čeština). Úsek `D MMMM YYYY` se v japonštině, čínštině a korejštině
zapisuje s rokem na prvním místě (`2024年3月5日`). `ui.date_locale` (`"system"`,
výchozí, nebo `"english"`) ponechá názvy anglicky bez ohledu na jazyk.

**Agenti, terminály a složka karty**

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | Okno vůbec přijímá `ui show` / `ui propose` od agentů (Nastavení > Agenti a MCP > *Povolit AI agentům ovládat zobrazení*). Vypnuto: žádný agent okno neovlivní. Viz [MCP a agenti](mcp-and-agents.md#zobrazování-věcí-uživateli). |
| `ui.launch` | `"off"` | Zda smí agent otevřít okno GitPoulpi, když žádné není otevřené: `off`, předvolba (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) nebo `custom`. |
| `ui.launch_command` | `""` | Šablona `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Rozdělí se na argumenty, bez shellu. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (řada karet nabízí *Konec a cd sem*) nebo `auto` (každé ukončení změní složku shellu a nová karta spustí výchozí položku *Otevřít v*). Viz [Sledování složky karty](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Zastaralé, viz níže a [Otevřít v](open-in.md). Jak starý *Terminál zde* otevíral terminál: `auto` (předvolba `ui.launch`, jinak `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), předvolba, nebo `custom`. |
| `ui.terminal_launch_command` | `""` | Šablona `custom`; `{repo}` je složka. |
| `ui.open_in` | *(chybí)* | Seznam [Otevřít v](open-in.md), pole tabulek `[[ui.open_in]]` s `label` a `command`. Chybějící znamená „ještě nenakonfigurováno“: seznam se spočítá ze starých klíčů `ui.terminal_launch*` (hodnota jiná než `auto` se stane první položkou), správce souborů OS, multiplexerů a terminálů zjištěných z prostředí (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) a editorů nalezených na `PATH`. První uložení z Nastavení seznam zapíše. `open_in = []` je prázdný seznam. Nelze nastavit pomocí `config set` (upravte soubor). |
| `ui.open_in_default` | `""` | Popisek položky, kterou spouští „Otevřít ve výchozím“; prázdný nebo neznámý znamená první položku. |
| `ui.launch_terminal` | `"auto"` | Terminál, ve kterém zástupce na ploše (Nastavení > Spouštěč) a znovuspuštění ve Windows spouštějí GitPoulpi. `auto` vybírá z nainstalovaných (Windows: Windows Terminal, jinak WezTerm, Alacritty, běžná konzole; Linux: `xdg-terminal-exec`, jinak první nalezený terminál; macOS: iTerm2, Ghostty, WezTerm nebo kitty, jsou-li nainstalovány, jinak Terminal), `default` je terminál systému, nebo id terminálu, například `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Jen v režimu vývojáře (viz `general.dev_mode`; jen tehdy se zobrazí v Nastavení > Spouštěč, jinak se ignoruje). `"current"`: zástupci a alias příkazu spouštějí sestavení, které je vytvořilo. `"newest"`: spouštějí naposledy sestavené z `target/dist` a `target/release` v klonu, rozhoduje se při každém kliknutí, a spuštění kteréhokoli sestavení je nikdy nepřepíše. |
| `ui.command_alias` | `""` | Název příkazu, kterým se GitPoulpi spouští z terminálu, například `gp` (Nastavení > Spouštěč, `gitpoulpi alias create`). Prázdné: žádný. Jen písmena, číslice, `-`, `_` a `.`. Je to jen název, který používá tlačítko **Vytvořit: alias příkazu**; dokud ho nevytvoříte, nic neexistuje. |
| `ui.command_alias_add` | `""` | Název druhého příkazu, například `gpa`, který spouští `gitpoulpi --add`: otevře aktuální složku jako novou kartu již spuštěného GitPoulpi (Nastavení > Spouštěč, `gitpoulpi alias create gpa --add`). Prázdné: žádný. Stejná pravidla pro název; tlačítko **Vytvořit** ho vytvoří spolu s `ui.command_alias`. |
| `ui.relaunch_in_terminal` | `true` | Jen Windows: když se exe spustí dvojklikem ve vlastní konzoli, GitPoulpi se znovu otevře v `ui.launch_terminal` a skončí. Terminál, který exe spouští přímo (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio, Tabby, Alacritty `-e`), je také vlastní konzole, ale ne klasické okno konzole: nikdy se znovu nespouští. Vypnuto: zůstane v té konzoli. |
| `ui.reuse_window` | `true` | Kliknutí na zástupce GitPoulpi (`gitpoulpi launch`) zobrazí okno už otevřeného GitPoulpi místo spuštění dalšího. Vypnuto: každé kliknutí spustí nové okno. |

**Příkazy `ui.open_in`** se nejprve rozdělí na argumenty (mezery, `'...'`, `"..."`), pak se uvnitř každého argumentu dosadí proměnné, takže složka s mezerami zůstane jedním argumentem; žádný shell se nepoužívá, pokud ho příkaz sám nespustí (`sh -c '... "$GITPOULPI_CWD"'`). Proměnné: `${cwd}` (složka karty), `${repo}` (kořen repozitáře), `${name}` (základní název složky), `${branch}` (prázdné při odpojeném HEAD), `${web_url}` (webová stránka remote `origin`, je-li to projekt na GitHubu nebo GitLabu, jinak prázdné; položka, která ji používá, se při prázdné hodnotě odmítne spustit), `${env:NAME}` (prázdné, není-li nastaveno). Staré `{repo}` stále znamená `${cwd}`. Jakékoli jiné `${...}` je chyba s jeho názvem. Příklad:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Zastaralé: `ui.terminal_launch` a `ui.terminal_launch_command` se stále čtou (zakládají seznam, dokud `ui.open_in` chybí: `custom` se stane „Terminal (custom)“ s `{repo}` přepsaným na `${cwd}`, id předvolby se stane touto předvolbou), ale jsou nahrazeny `ui.open_in`.

### Obrázky

| Klíč | Výchozí | Význam |
|---|---|---|
| `ui.image_view` | `true` | Zobrazí binární soubor, který je obrázkem (PNG, JPEG, GIF, WebP, BMP, ...), jako obrázek v zobrazení Diff a v zobrazení Soubor. Vypnuto: prosté upozornění na binární soubor. |
| `ui.image_view_max_mib` | `32` | Obrázky větší než tato hodnota (MiB) se nedekódují; místo nich se zobrazí informační karta. |
| `ui.image_view_max_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje limit: `mib`, `gib` (1024 MiB) nebo `auto`. Jen zobrazení: limit je vždy `image_view_max_mib` MiB, 1 až 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | V terminálu bez protokolu obrázků: `halfblock` (barevné znaky půlbloků) nebo `card` (jen informační karta). |
| `ui.image_view_pixelated_from` | `"2x"` | Přiblížení, od kterého zvětšení ukazuje čtvercové pixely místo vyhlazení: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Panely před a po sdílejí jedno přiblížení a jednu polohu posunu. |
| `ui.image_editors` | *(chybí)* | Editory obrázků pro „Upravit v“, pole tabulek `[[ui.image_editors]]` s `label` a `command`. Chybějící znamená „ještě nenakonfigurováno“: seznam tvoří editory zjištěné v tomto počítači (programy na `PATH`, instalace Flatpak, známé instalační složky ve Windows, `/Applications` v macOS) a poté výchozí aplikace OS. První uložení z Nastavení seznam zapíše. `image_editors = []` je prázdný seznam. Nelze nastavit pomocí `config set` (upravte soubor). |
| `ui.image_editor_default` | `""` | Popisek editoru, který spouští „Upravit ve výchozím“; prázdný nebo neznámý znamená první položku. |

**Příkazy `ui.image_editors`** se nejprve rozdělí na argumenty (mezery, `'...'`, `"..."`), pak se uvnitř každého argumentu dosadí proměnné, takže cesta s mezerami zůstane jedním argumentem; žádný shell se nepoužívá. Proměnné: `${file}` (absolutní cesta obrázku), `${dir}` (jeho složka), `${name}` (jeho název souboru), `${repo}` (kořen repozitáře, u dočasné kopie prázdné), `${env:NAME}` (prázdné, není-li nastaveno). Jakékoli jiné `${...}` je chyba s jeho názvem. Obrázek, který není souborem pracovního stromu (verze z commitu, index, smazaný soubor), se otevře jako **dočasná kopie jen pro čtení**, takže editor se nemůže dotknout repozitáře. Editory bez příkazové řádky (Affinity Photo z Microsoft Store, některé instalace Photoshopu) v předvolbách nejsou: napište vlastní položku. Příklad:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Zatím se nepoužívá** (přijímá se, ukládá se, ale nic je nečte):
`ui.graph_colorization` (odstraněno z Nastavení;
při načtení se stále přijímá), `ui.date_word_format` (odstraněno z Nastavení;
při načtení se stále přijímá). Starý `ui.diff_view_mode`
se jednou přečte jako `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `editor.tab_size` | `4` | Sloupců na zarážku tabulátoru, 1 až 8 (editor a diffy). |
| `editor.eol` | `"auto"` | Konec řádku, který zapisuje vestavěný editor: `auto` (ponechá podle souboru), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Barvy jazyka v editoru. |
| `editor.line_numbers` | `true` | Čísla řádků v editoru a diffech. |
| `editor.follow_selection` | `true` | Dokud je editor otevřený, výběr jiného souboru v pravém panelu jej otevře i v editoru, a to po dotazu, zda uložit neuložené změny. Soubor, který nelze upravovat, se otevře v obvyklém zobrazení; soubor commitu opustí editor a otevře se v diffu. |

## `[merge]`

Editor konfliktů merge ([Řešení konfliktů merge](merge-conflicts.md)). Zobrazuje ho Nastavení > Diff a editor.

| Klíč | Výchozí | Význam |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | Šířka oblasti editoru ve sloupcích, od které se A a B zobrazují vedle sebe (dlouhé řádky se posouvají do stran: Shift+kolečko, Shift+šipky). Při menší šířce zobrazuje vždy jednu stranu za kartami. `40` je nejnižší hodnota, `0` znamená vždy vedle sebe (upozornění pod 40 sloupců zůstává). Editor při otevření skrývá panely Refs a Stav, takže jeho oblastí je obvykle celé okno. |

## `[git]`, `[credentials]`, `[ssh]`

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `git.pull_mode` | `"merge"` | Co dělá Pull (tlačítko, `Ctrl+G p`, `Git: Pull`): `merge` (fast-forward, je-li to možné), `ff_only`, `rebase`. Gitu se vždy předává výslovně. Starý `fetch_all` se čte jako `merge`. |
| `git.push_behavior` | `"direct"` | `direct` pushuje do výchozího remote; `ask` nejprve zobrazí nabídku remote. |
| `git.push_group_preflight` | `true` | Před pushem větve na několik remote nejdřív zkontroluje každý cíl (`ls-remote`, plus fetch toho remote jen tehdy, když se liší), takže problém na pozdějším remote se najde před prvním pushem. |
| `git.merge_fast_forward` | `"never"` | Co udělá merge, který spustíte (nabídka větve, přetažení, paleta, `gitpoulpi merge`), když lze větev posunout fast-forwardem: `never` vždy vytvoří merge commit (`--no-ff`), `allowed` provede fast-forward, když lze (výchozí chování gitu, platí `merge.ff`), `only` provede fast-forward, nebo odmítne. Vlastní volba repozitáře (Nastavení > Možnosti merge, `merge_fast_forward` v jeho `repo-state.toml`) má přednost; přepínače CLI `--ff`, `--no-ff` a `--ff-only` mají přednost před oběma. Okamžitě. |
| `git.default_encoding` | `"utf-8"` | Kódování textu, který **není** platné UTF-8 (obsah souborů, blame, diffy, zprávy commitů bez hlavičky `encoding`): libovolný popisek WHATWG (`windows-1252`, `shift_jis`, ...). Jen zobrazení; staging nikdy nepřekóduje. Repozitář ho může přepsat. |
| `git.lfs_enabled` | `true` | Zobrazí a ovládá Git LFS. Repozitář ho může přepsat. Viz [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Před pull nebo push repozitáře používajícího LFS nainstaluje jeho filtry LFS / hook pre-push (`git lfs install --local`, jen tento repozitář), pokud chybějí. Viz [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Nepoužívá se*: `core.autocrlf` / `core.eol` repozitáře jsou vlastní nastavení gitu, upravují se v Nastavení > Git a kódování (nebo `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Autentizace HTTPS pro fetch, pull, push, clone: `credential_helper` (gitu), `prompt` (helpery se ignorují; git se zeptá v terminálu, takže v daemonu na pozadí selže), `none` (jen veřejné remote). Staré hodnoty `encrypted_file` se čtou jako `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Typ vytvářený akcí *Vygenerovat pár klíčů*: `ed25519`, `rsa` (4096 bitů), `ecdsa` (NIST P-256); soubor je `gitpoulpi_<type>`. Samotné SSH vždy používá váš ssh-agent a konfiguraci ssh. |
| `ssh.private_key_path`, `ssh.public_key_path` | nenastaveno | Zapisuje *Vygenerovat pár klíčů*. Je-li `ssh.private_key_path` nastaveno, fetch, pull, push a clone spouštějí git s `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` dělá totéž (ssh vybere odpovídající klíč agenta), když je cesta k privátnímu klíči prázdná. Přednost má `GIT_SSH_COMMAND`/`GIT_SSH` v prostředí nebo `core.sshCommand` v git konfiguraci (připnutí pro repozitář, `gitpoulpi ssh-command set --key`). Čte se před každým přenosem, takže se změny uplatní při dalším. |

## `[[integrations]]` (hosting a issue trackery)

Zapisuje Nastavení > Integrace nebo `gitpoulpi hosting integrations add`. Každý
druh má klienta (pouze osobní tokeny, žádné OAuth).

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

Server hostovaný vlastními silami může běžet na prostém http: `url = "http://localhost:3000"`
se ponechá tak, jak je zadáno (schéma i port se použijí u každého požadavku a remote
na tomto hostiteli odpovídá profilu). Token odeslaný přes prosté http putuje
nešifrovaně, proto používejte http jen v místní nebo důvěryhodné síti.

`kind` přijímá také `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` a `gerrit`
(Gitea a Forgejo sdílejí jednoho klienta; profil Forgejo má výchozí `https://codeberg.org`). Jira,
Trello, Linear, YouTrack a Redmine jsou pouze issue trackery; Gerrit slouží jen ke kontrole (jeho změny
jsou pull requesty, issues nemá). Bitbucket Cloud nemá issue tracker.

`account` je neutajovaná identita vedle tokenu, u většiny druhů prázdná a
vynechaná: e-mail účtu Atlassian pro `jira_cloud` a `bitbucket` (prázdný účet
Bitbucket znamená token workspace nebo repozitáře, odesílaný jako Bearer), klíč
API pro `trello`, uživatelské jméno pro `gerrit` (token je jeho HTTP heslo, odesílané jako HTTP basic).
Samotný token zůstává jediným tajemstvím. U `azure_devops` je
`url` `https://dev.azure.com/<organization>`; URL
`https://<organization>.visualstudio.com` se čte jako táž organizace. U
`azure_devops_server` je to kořen kolekce, například
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (připojený účet) a `ssh` (volba klíče profilu) se zapisují za vás. Token
pochází z `GITPOULPI_TOKEN_<ID>` (id velkými písmeny, nealfanumerické znaky jako
`_`), jinak z uloženého tokenu, jinak z vašeho credential helperu pro daného
hostitele, když ho používá jen jeden profil (pouze profily GitHub a GitLab:
ostatní druhy credential helper nikdy nečtou). Podrobnosti: [Hostingové služby a issue trackery](hosting-integration.md). Stará tabulka
`[[hosting]]` se *nepoužívá*.

U `local_markdown` (složka poznámek Markdown jako zdroj issues) je `url` složka,
relativní k repozitáři nebo absolutní, a token neexistuje. Její volitelná tabulka
`[integrations.local]` nastavuje `statuses` (pořadí sloupců tabule, výchozí `todo`, `doing`,
`done`), `done_status` (výchozí `done`), `reference_prefix` (`LOC` zobrazí `LOC-12`)
a `me` (koho znamená „přiřazené mně“). Viz [Lokální issues a Obsidian](local-issues.md).

### `[integrations.radar]` (seznamy za celý účet)

Volitelná tabulka profilu hostitele kódu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; ne Bitbucket Cloud). Chybějící
tabulka znamená, že je každý klíč vypnutý. Zapisuje ji `gitpoulpi hosting integrations radar`; čtou ji
`gitpoulpi hosting radar` a Radar a upravuje se v Nastavení > Integrace (blok Radar připojeného profilu).

```toml
[integrations.radar]
review_requests = true    # žádosti čekající na mou kontrolu, každý repozitář
open_requests = false     # každá otevřená žádost mých jmenných prostorů
include_drafts = false    # ponechat koncepty žádostí v těchto seznamech
assigned_issues = true    # otevřené issues přiřazené mně, každý repozitář
namespaces = ["acme", "ptm/produits"]  # jedna položka na řádek v Nastavení; prázdné = bez omezení
namespaces_regex = false  # true: každá položka je regulární výraz nad celou cestou repozitáře
review_mode = "ask"       # ask | clone | api
```

| Klíč | Výchozí | Význam |
|---|---|---|
| `review_requests` | `false` | Otevřené žádosti, které žádají o vaši kontrolu, ať je repozitář naklonovaný, nebo ne |
| `open_requests` | `false` | Každá otevřená žádost vašeho vlastního jmenného prostoru a každé organizace nebo skupiny, jejímž jste členem (nebo `namespaces`), nikdy „vše, co token vidí“ |
| `include_drafts` | `false` | Koncepty žádostí jsou z těchto seznamů vynechány, pokud to není zapnuto |
| `assigned_issues` | `false` | Otevřené issues přiřazené vám, každý repozitář |
| `namespaces` | `[]` | Které repozitáře si každý seznam ponechá (žádosti ke kontrole, přiřazené issues, otevřené žádosti); prázdné znamená bez omezení a `open_requests` pak prohledá váš vlastní jmenný prostor plus každé členství (GitHub: prohledá se nejvýše 12, GitLab: 10 skupin a 10 vlastních projektů, Gitea, Forgejo a Gerrit: 10). Položka odpovídá celé cestě repozitáře (`skupina/podskupina/projekt`, Azure DevOps `Project/repo`, Bitbucket Data Center `KEY/repo`), když se jí rovná nebo je její předponou končící na `/`: `ptm` ponechá `ptm/produits/x`, ne `ptmx/y`; velikost písmen se ignoruje. Nastavení ukazuje jednu položku na řádek; starší soubor může držet jeden řetězec oddělený čárkami (`"ptm, formation"`), který se stále načte a uloží zpět jako seznam |
| `namespaces_regex` | `false` | Číst každou položku `namespaces` jako regulární výraz porovnávaný s celou cestou repozitáře tak, jak ji zapisujete (takže `^ptm/(produits\|infra)/` kotví na začátku; bez `^` nebo `$` odpovídá kdekoli v cestě; `(?i)` přidáte pro ignorování velikosti písmen). `open_requests` pak prohledá vaše členství a výrazy filtrují řádky. Neplatný výraz se ukáže pod polem v Nastavení a neuloží se; v ručně upraveném souboru nechá seznamy prázdné s poznámkou |
| `review_mode` | `"ask"` | Co udělá kontrola žádosti, jejíž repozitář není naklonovaný: `ask`, `clone` (naklonovat, pak zkontrolovat) nebo `api` (zkontrolovat přes API hostitele) |

Odpovědi se pro každý profil na pět minut ukládají do mezipaměti `radar-account.json` vedle
`config.toml`; soubor je mezipaměť, lze ho bezpečně smazat.

### Úložiště důvěry (není v `config.toml`)

Čemu byla každá integrace a profil AI s klíčem API důvěřována (její adresa,
id účtu, připnutý klíč certifikátu, zda bylo povoleno prosté http) žije v
`trust.toml` v soukromé datové složce jen pro uživatele (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` ve Windows, `~/.local/share/gitpoulpi/private/`
v Linuxu). Záměrně není součástí `config.toml`: kdo může upravovat
konfiguraci, nesmí moci rozšířit důvěru jejím úpravou. Mění se příkazy
`gitpoulpi hosting integrations trust` a `gitpoulpi ai trust` (viz
[kontrola zabezpečení](hosting-integration.md#kontrola-zabezpečení)); soubor
přepisuje GitPoulpi, ne ruční úprava.

## `[mcp]` (co smějí dělat AI agenti)

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `mcp.allow_mutations` | `false` | Vypnuto: nástroje MCP, které cokoli mění, se odmítnou; čtení, `ui_*` a návrhy dál fungují. `true` agentům dovolí měnit repozitář. |
| `mcp.mutation_allowlist` | `[]` | Neprázdné: měnit smějí jen tyto názvy nástrojů (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: odmítne nástroje `ui_*`, které hýbou vaším oknem (ne `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (okno ukáže čip, který potvrdíte), `auto` (použije se, pokud nejste zaneprázdněni), `off`. `--ui-control` u příkazu serveru ho přebíjí pro daného agenta. Platí i pro `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Povolí `ui_propose_*` / `gitpoulpi ui propose`: předvyplněné dialogy, které potvrdíte sami. |
| `mcp.allow_history_purge` | `false` | Jen v `config.toml` (žádný řádek v Nastavení). `true` zpřístupní agentům nástroj `rewrite_purge` (stále vyžaduje `allow_mutations`). Vypnuto, agent ten nástroj vůbec nevidí: purge smaže nadobro starou historii přepisu, její vrácení zpět i její reflogy. CLI a TUI touto volbou omezeny nejsou. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` a `gitpoulpi-mcp` odmítnou start se zprávou pojmenovávající tento klíč. Nastavení > Agenti a MCP > *Server MCP*. Čte se, když agent spouští server. |
| `mcp.bind` | `"127.0.0.1:8808"` | Adresa, na které `--http` naslouchá, když žádnou nezadáte (`--http ADDR` / `--bind` mají přednost). Jen loopback, pokud není `--allow-remote`. Prázdné nebo starý `"stdio"` znamenají výchozí. Nastavení > Agenti a MCP > *Port serveru MCP* upraví port a ponechá hostitele. |
| `mcp.http_auth` | `true` | `--http` odpovídá jen na požadavek s `Authorization: Bearer <token>` (token z `gitpoulpi mcp token`). `false` kontrolu odstraní; server pak varuje na stderr a `--allow-remote` se odmítne. Nastavení > Agenti a MCP > *Ověřování HTTP*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (certifikát vygenerovaný jednou v soukromé datové složce, jeho otisk SHA-256 se vypíše při startu) nebo `files` (`tls_cert` a `tls_key`). Se zapnutým TLS server TLS ukončuje sám a žádný prostý posluchač neexistuje. Nastavení > Agenti a MCP > *TLS*. |
| `mcp.tls_cert` | `""` | Řetěz certifikátů PEM používaný s `tls = "files"`. |
| `mcp.tls_key` | `""` | Soukromý klíč PEM používaný s `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Prohlašujete, že šifrování zajišťuje tunel nebo reverzní proxy před endpointem. Zevnitř serveru to nelze ověřit, takže `--allow-remote` bez TLS vyžaduje, aby to bylo nastaveno. Úsek mezi proxy a serverem je v té síti prostý text. Nastavení > Agenti a MCP > *Za tunelem nebo reverzní proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Jen v `config.toml`. Vypnuto, nástroje MCP pro stage nenabízejí `allow_sensitive` a přidání do indexu, které by zahrnulo označený soubor (viz `[privacy]`), se odmítne se zprávou, že se agent má zeptat vás. Zapnuto, agent smí předat `allow_sensitive`. CLI a TUI touto volbou omezeny nejsou: ptají se vás. |

`allow_mutations`, `mutation_allowlist` a `allow_ui` hlídají jen server MCP;
CLI je váš vlastní shell a těmito klíči se neřídí. Viz [MCP a
agenti](mcp-and-agents.md#oprávnění).

## `[ai]` (vestavěná AI, ve výchozím stavu vypnuta)

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `ai.enabled` | `false` | Přepínač. Vypnuto: nikde se nic z AI nezobrazuje a nic se neodesílá. |
| `ai.default_profile` | `"default"` | Id profilu, který akce používají, když žádný nepojmenují. |
| `ai.global_instructions` | `""` | Text přidaný do promptu každé akce. Může používat `${repo}`, `${uilang}`, ... (viz [AI > Proměnné promptu](ai.md#proměnné-promptu)); totéž platí pro instrukce jednotlivých akcí a repozitáře. |
| `ai.actions.<action>.profile` | `""` | Profil pro jednu akci; prázdné = výchozí. Akce: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`. Pracovní prostor (`workspaces.toml`, `[workspace.ai_instructions]` a `[workspace.ai_overrides.<action>]`) i repozitář mohou tyto instrukce doplnit nebo nahradit: viz [AI](ai.md#instrukce). |
| `ai.actions.<action>.instructions` | `""` | Text přidaný jen pro tuto akci. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` nebo `no`: zda místní agent konceptu kontroly kódu pracuje pro daný běh uvnitř repozitáře, což přepíše `workdir` profilu AI. Repozitář může mít vlastní hodnotu (`gitpoulpi ai review-context --repo`). Viz [AI > Kontext kontroly](ai.md#kontext-kontroly). |
| `ai.review.include_claude_md` | `true` | Předat konceptu kontroly kódu soubor `CLAUDE.md` repozitáře (kořen, čtený ve větvi, na kterou žádost míří; oříznutý na 8 KB). |
| `ai.review.include_agents_md` | `true` | Totéž pro `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Ještě jeden soubor pro koncept kontroly kódu: cesta relativní ke kořeni repozitáře, absolutní cesta nebo odkaz na soubor na hostiteli kódu (čte se přes odpovídající profil integrace, jinak anonymně). Všechny soubory dohromady se ořezávají na 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Nahradí každou tajnou hodnotu, kterou skener v požadavku pozná (token, blok soukromého klíče, heslo v URL, hodnotu `password = ...`, která vypadá náhodně), za `[REDACTED:<rule>]`, než požadavek odejde. Nastavení > AI ukazuje červené varování, dokud je vypnuto. |
| `ai.privacy.exclude_sensitive_files` | `true` | Vynechá z odesílaného diff každého souboru odpovídajícího `privacy.sensitive_patterns`. Lokální agent, který pracuje ve složce repozitáře, si soubory stále může přečíst sám: viz [AI > Soukromí](ai.md#soukromí). |
| `ai.limits.conflict_bytes` | `40000` | Nejvíc bajtů, které smí mít jeden konflikt merge (jeho bloky base, current a incoming dohromady), když ho odesílá „Navrhnout řešení“. Nad limitem GitPoulpi neselže: upozorní a nabídne *Odeslat přesto* (jen tento požadavek), *Otevřít nastavení* a *Zrušit*; konflikt se nikdy neřeže. Limit je vlastní pojistka GitPoulpi, ne limit modelu: zvyšte ho, pokud na to váš model má kapacitu. Drží se mezi 1000 a 50000000; nastavitelné v Nastavení > AI > Limity. Ostatní limity níže fungují stejně a jsou všechny v té skupině. |
| `ai.limits.diff_bytes` | `60000` | Nejvíc bajtů diffu, které smí nést zpráva commitu, vysvětlení, název stashe, popis pull requestu, plán commitů nebo rozdělení commitu, a celého diffu revize kódu. Nad limitem se nic neřeže potichu: v TUI *Nad limitem AI* nabídne *Odeslat vše* (jen tento požadavek, nic se neřeže), *Odeslat zkráceně* (konec se vynechá, jako dřív), *Otevřít nastavení* a *Zrušit*; CLI a MCP zkracují a hlásí to v `truncated` (`--send-all` odešle všechno). Drží se mezi 1000 a 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Nejvíc bajtů diffu jednoho souboru v revizi kódu. Nad limitem stejná otázka a stejné dvě odpovědi; *Odeslat zkráceně* ten soubor zkrátí s viditelnou poznámkou. Stejný rozsah. |
| `ai.limits.review_text_bytes` | `4000` | Nejvíc bajtů popisu revidovaného pull requestu. Nad limitem *Odeslat zkráceně* popis ořízne s poznámkou `[text cut]`. Stejný rozsah. |
| `ai.limits.review_threads_bytes` | `6000` | Nejvíc bajtů seznamu již existujících komentářů revize (uvádějí se, aby je model neopakoval). Nad limitem *Odeslat zkráceně* seznam ořízne. Stejný rozsah. |
| `ai.limits.review_context_file_bytes` | `8192` | Nejvíc bajtů jednoho kontextového souboru projektu (`CLAUDE.md`, `AGENTS.md`, další soubor) předaného revizi kódu. Nad limitem *Odeslat zkráceně* soubor zkrátí s poznámkou. Stejný rozsah. |
| `ai.limits.review_context_bytes` | `16384` | Nejvíc bajtů všech kontextových souborů dohromady. Nad limitem *Odeslat zkráceně* zkrátí soubor, který ho překročí, a následující vynechá. Stejný rozsah. |
| `ai.limits.hosting_reply_context_bytes` | `49152` | Nejvíc bajtů všeho, co navržená odpověď (nebo popis issue) dostane kromě vašeho vlastního textu: položka a její konverzace. Nad limit se nejdřív vynechají nejstarší komentáře a ořez se hlásí v `truncated`. Stejný rozsah jako u ostatních bajtových limitů. |
| `ai.limits.hosting_reply_file_bytes` | `8192` | Nejvíc bajtů jednoho souboru repozitáře, který dostane navržená odpověď. Používá se, když AI čte repozitář. Stejný rozsah. |
| `ai.limits.hosting_reply_map_bytes` | `6144` | Nejvíc bajtů mapy repozitáře, kterou dostane navržená odpověď. Používá se, když AI čte repozitář. Stejný rozsah. |
| `ai.limits.hosting_reply_hits_per_term` | `20` | Nejvíc nálezů ponechaných na jeden hledaný výraz navržené odpovědi. Používá se, když AI čte repozitář. Drží se mezi 1 a 200. |
| `ai.limits.hosting_reply_terms` | `8` | Nejvíc hledaných výrazů, které navržená odpověď vyhledá. Používá se, když AI čte repozitář. Drží se mezi 1 a 50. |
| `ai.hosting_reply.base_prompt` | `""` | Základní prompt obou psacích akcí (`hosting_reply`, `issue_body`). Prázdné použije vestavěný; text zde ho nahradí a úrovně pokynů po něm stále následují. Upravuje se v Nastavení > AI > Odpovědi na hostiteli. |
| `ai.hosting_reply.read_repository` | `false` | Zda volba „Nechat AI číst repozitář“ začíná zaškrtnutá. Ve výchozím stavu vypnuto: repozitář se čte, jen když ji pro dané použití zaškrtnete. Lokální agent nebo profil API ho pak čte tak, jak popisuje [kapitola o AI](ai.md#přístup-k-repozitáři). |
| `ai.hosting_reply.reuse_agent_session` | `true` | Zachová relaci lokálního agenta mezi dvěma odpověďmi, když to jeho harness podporuje. Projeví se, jakmile smí AI číst repozitář. |
| `ai.hosting_reply.context_ttl_minutes` | `30` | Jak dlouho se kontext repozitáře sestavený pro odpověď znovu používá. Drží se mezi 1 a 1440. Projeví se, jakmile smí AI číst repozitář. |
| `ai.sent_preview.<action>` | `true` | Jeden klíč na akci (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`, `hosting_reply`, `issue_body`): před odesláním požadavku ukáže "Co se odešle". "Příště nezobrazovat pro ..." v dialogu zapíše `false`; Nastavení > AI > *Zobrazit, co se odesílá, před...* je znovu zapne. `rewrite_script` a `rewrite_rules` se přijímají, ale nemají účinek: dialogy přepisu historie vždy ukazují vlastní podokno. `worktree_name` se také přijímá, ale nemá účinek: odesílá jen to, co jste napsali v dialogu Vytvořit worktree, a názvy lokálních větví. |

Vlastní tabulka repozitáře `[preferences.ai_review]` (v jeho `repo-state.toml`, nebo záznam repozitáře, který není
naklonovaný, v `remote-reviews.toml`) má stejné klíče, všechny nepovinné, a navíc `no_extra_file = true`: pro tento
repozitář žádný další soubor, ať `ai.review.extra_file` říká cokoli. Nenastavený klíč, nenastavený `extra_file`
a chybějící `no_extra_file` zdědí globální hodnotu; nastavení `extra_file` na soubor vymaže přepínač a naopak.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness` vybírá návrhy, které nabízí Nastavení > AI (bezobslužné argumenty,
volby, přepínač a seznam modelů, proměnné prostředí) nezávisle na `command`;
oba klíče se při prázdné hodnotě ze souboru vynechají. Hodnoty prostředí se
nikdy nelogují a v Nastavení se skrývají, když vypadají jako tajemství.

Nastavení > AI nabízí předvolby pro `harness`/`command`/`args`/`env` (běžná CLI agentů s jejich
bezobslužnými přepínači, přepínačem modelu a užitečnými proměnnými prostředí) a pro `kind`/`base_url`/`model` (hostované
i místní poskytovatele HTTP); viz [AI > Konfigurace](ai.md#konfigurace-ai).

Profil s názvem *Default* se vytvoří, pokud žádný není. Staré soubory s
`[[ai.presets]]` / `[ai.defaults]` se při načtení migrují a uchovají v
`[[ai.legacy_presets]]`. Vše ostatní: [Asistence AI](ai.md).

## `[repos]`, `[session]`

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `repos.clone_root` | `""` | Odkud začíná dialog Klonovat; prázdné = `%USERPROFILE%\source\repos` nebo `~/src`. |
| `repos.clone_start_source` | `"last"` | Zdroj, na kterém se dialog Klonovat otevře: `last` (výchozí, nebo prázdné) = ten, který zobrazoval při posledním zavření, `url` = vždy Klonovat z URL, nebo id profilu integrace (`id` v jejím záznamu `[[integrations]]`). Profil, který zmizel, je vypnutý nebo není připojený, se vrátí na Klonovat z URL. Poslední zdroj samotný je stav, uložený v `clone-source.toml` ve složce konfigurace, ne zde. |
| `repos.browse_root` | `""` | Kde se otevře Procházet; prázdné = naposledy zvolená složka, pak domovská. |
| `repos.show_hidden` | `false` | Výběr složky zobrazuje skryté složky (`Alt+H` to přepíná a ukládá). |
| `repos.show_siblings` | `false` | Levý strom výběru složky uvádí i ostatní složky každé otevřené úrovně, jako skutečnou hierarchii (`Alt+S` to přepíná a ukládá). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Kde si GitPoulpi pamatuje data o jednom repozitáři (filtry refů, stav sbalení, osobní přepsání workflow, mezipaměť hostingu): `in_git_dir` = `<git dir>/gitpoulpi` (sdíleno všemi worktree, v `git status` se nikdy neukáže), `in_worktree` = `<repo>/.gitpoulpi` (jediné místo, kde tým může commitnout `workflow.toml`), `user_data` = `<datová složka OS>/repos/<name>-<id>` (`%APPDATA%\GitPoulpi\data` ve Windows, `$XDG_DATA_HOME/gitpoulpi` v Linuxu, `~/Library/Application Support/GitPoulpi` v macOS). Stará hodnota `".gitpoulpi"` dál znamená `in_worktree`. Změna nic nepřesouvá zpět: data nalezená ve starší složce `<repo>/.gitpoulpi` se přesunou na zvolené místo, když jsou poprvé potřeba. |

## `[log]` (protokol aplikace)

Co zobrazuje karta Aplikace v Protokolu aktivity a co cituje „Kopírovat diagnostiku“. Podrobnosti: [Když něco selže](troubleshooting.md#když-něco-selže).

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `log.level` | `"info"` | Nejpodrobnější řádek, který se zaznamenává: `error`, `warn`, `info`, `debug` nebo `trace`. Pole úrovně v Protokolu aktivity ji změní (a uloží sem). |
| `log.retention_days` | `7` | Počet dnů, po které se uchovávají soubory protokolu, včetně dneška: 1 až 3650, tedy deset let (větší nebo menší hodnota se ořízne). Vždy jde o počet dnů, ať ho Nastavení ukazuje v jakékoli jednotce. Starší soubory se smažou při spuštění GitPoulpi. Soubor jednoho dne je navíc omezen `log.max_file_mb`, takže nejhorší případ je asi `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje `log.retention_days`: `days`, `weeks`, `months`, `years` nebo `auto`. Jen zobrazení, nikdy nemění, jak dlouho se soubory uchovávají. Týden je 7 dní, měsíc je 30 dní (ne kalendářní měsíc, takže je to vždy stejný počet dní) a rok je 365 dní. `auto`, a také konfigurace zapsaná před vznikem tohoto klíče, vybere největší jednotku, která dny dělí beze zbytku (14 dní se čte „2 týdny“, 45 dní „45 dní“). Jednotka, která dny nedělí (`retention_days = 10` s `weeks`), se ignoruje a zobrazí se dny. Nastavení zapisuje oba klíče; starší GitPoulpi jednotku ignoruje a dny čte dál. |
| `log.max_file_mb` | `5` | Soubor jednoho dne se při této velikosti přesune do jedné zálohy `.old`, takže den nikdy nezabere víc než dvojnásobek. |
| `log.max_file_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje velikost: `mib`, `gib` (1024 MiB) nebo `auto`. Jen zobrazení: velikost je vždy `max_file_mb` MiB, 1 až 1024. |
| `log.timings` | `true` | Karta Operace gitu v Protokolu aktivity ukazuje, jak dlouho každá operace trvala (`Push - 123 ms`: otevření repozitáře, obnovení, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). Doby měří okno, od požadavku po odpověď daemona, takže zahrnují cestu IPC tam a zpět. Vypnuto skryje doby a záznamy úspěšných operací. Selhání se vypisují vždy. |
| `log.extended` | `false` | „Rozšířené protokolování“: karta Operace gitu vypisuje i operace na pozadí (pravidelné obnovení, které se ptá na stav, auto-fetch) s jejich dobami. Do protokolu aplikace se vždy zapisují na úrovni `debug`. |
| `log.commit_count` | `"system-git"` | Jak stavový řádek zjistí celkový počet v údaji `načteno / celkem`. `"system-git"` spustí `git rev-list --count` (jeden proces, na běžném počítači nejrychlejší: git čte soubor commit-graph). `"libgit2"` počítá uvnitř procesu průchodem libgit2 a nespouští nic: zvolte ho, když je spuštění každého procesu příkazového řádku pomalé, typicky proto, že každý kontroluje antivirový program s kontrolou v reálném čase; u 50 000 commitů to na pozadí trvá asi 200 ms místo 12 ms, ale žádný proces se nespustí. `"off"` nepočítá nikdy: lišta ukazuje jen načtené commity. Platí živě. Počet omezený cestou vždy používá systémový git. |
| `log.use_mailmap` | `true` | Zobrazí každého autora tak, jak ho pojmenovává `.mailmap` repozitáře (a `mailmap.file`), v Logu, v blame i v historii souboru: daemon se zeptá gitu na `%aN`/`%aE` (systémový backend) nebo použije tytéž soubory sám (backend libgit2), takže odpověď je stejná. Uložené commity se nemění a nic se nepřepisuje. Hledání v Logu podle autora odpovídá jménům tak, jak se zobrazují. Platí živě; Log se znovu načte během svého 2s dotazování. |

Soubory se jmenují `gitpoulpi-YYYY-MM-DD.log` a leží ve složce stavu operačního systému (`~/.local/state/gitpoulpi/logs` na Linuxu; `<cache>\logs` ve Windows a macOS; `<config>/cache/logs` s `GITPOULPI_CONFIG_DIR`). Tokeny, hesla a hlavičky `Authorization` se maskují dřív, než se cokoli zapíše.

## `[search]` (hledání v pracovním prostoru)

Co prohledává `gitpoulpi search` (a jeho nástroj MCP). Podrobnosti: [Pracovní prostory a hledání v CLI](cli-reference.md#pracovní-prostory-a-hledání).

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `search.commit_limit` | `200000` | Commity (nejnovější první, přes každou větev, vzdálenou větev a tag), jejichž zprávy index hledání drží v paměti pro každý repozitář, zhruba 100 bajtů plus zpráva. Delší historie se tu ořízne a odpověď říká, že je `partial`. `0` vypne hledání commitů (větve, tagy, stashe a worktrees se prohledávají dál). Čte se živě: příští hledání změnu respektuje. |

## `[rewrite]` (přepis historie)

Co `gitpoulpi rewrite` a nástroj pro přepis historie chrání. Podrobnosti: [Přepis historie](history-rewrite.md).

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Větve (názvy nebo vzory, kde `*` odpovídá libovolnému textu), které přepis nechá být, pokud už jsou odeslané, dokud je běh nevyjmenuje (`--include-protected BRANCH`; `--include-all-protected` pro push; políčko u každé větve v nástroji). Přidávají se k základním větvím, které `workflow.toml` repozitáře označí jako `protected`. Push přepisu je přeskočí stejně. Je to vlastní pojistka GitPoulpi: o přijetí force push stále rozhoduje ochrana větví u hostitele. Čte se živě: další náhled se změnou řídí. |
| `rewrite.external_scripts` | `["shell"]` | Které jazyky externích skriptů smí pravidlo Skript používat: libovolné z `"shell"` (bash nebo sh; pwsh nebo powershell pro skripty v PowerShellu), `"python"`, `"lua"` (nainstalovaná Lua; vestavěná Lua je vždy zapnutá a záznam nepotřebuje). **Externí skript není v izolaci: běží s vašimi oprávněními.** Jazyk, který není v seznamu, je v nástroji zašedlý a CLI, MCP i daemon ho odmítnou (`scripts_disabled`). I když je v seznamu, externí skript z repozitáře nebo ze sdílené sady se spustí až poté, co mu důvěřujete (`gitpoulpi rewrite script trust FILE` nebo otázka v nástroji); důvěra se váže na hash obsahu skriptu, takže změněný skript se zeptá znovu. Python zapnete úpravou `config.toml` (seznam nelze nastavit přes `config set`): `[rewrite]` a pak `external_scripts = ["shell", "python"]`. Čte se živě. |
| `rewrite.script_timeout_ms` | `5000` | Jak dlouho má externí skript na odpověď u jednoho commitu. Při překročení se ukončí celý strom procesů a přepis selže (`script_failed` s názvem commitu); nic se nepřesunulo. `0` se čte jako výchozí hodnota. Čte se živě. |
| `rewrite.script_timeout_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje časový limit: `milliseconds`, `seconds` nebo `auto`. Jen zobrazení: limit je vždy `script_timeout_ms` milisekund. Nastavení přijímá 1 až 600000 (deset minut); `0` se stále čte jako výchozí hodnota. Čte se živě. |
| `rewrite.script_total_timeout_s` | `3600` | Jak dlouho smí trvat celý jeden průchod externího skriptu. `0` se čte jako výchozí hodnota. Čte se živě. |
| `rewrite.script_total_timeout_unit` | `"auto"` | Jednotka, v níž Nastavení ukazuje časový limit: `seconds`, `minutes`, `hours` nebo `auto`. Jen zobrazení: limit je vždy `script_total_timeout_s` sekund. Nastavení přijímá 1 až 86400 (den); `0` se stále čte jako výchozí hodnota. Čte se živě. |

## `[updates]` (hledání novějšího vydání)

Co dělá verzový čip v záhlaví okna s novějšími vydáními. Podrobnosti: [Aktualizace](updates.md).

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` se jednou denně na pozadí zeptá GitHubu, zda existuje novější vydání, a změní verzový čip na `vX.Y.Z k dispozici`. `"auto"` navíc samo stáhne a připraví aktualizaci, pokud sestavení pochází z archivu vydání (před restartem se stále zeptá); u jiného sestavení se chová jako `"notify"`. `"off"` neodešle nikdy žádný požadavek. Platí živě. |
| `updates.channel` | `"stable"` | `"stable"` bere v úvahu jen plná vydání; `"prerelease"` nabízí i kandidáty na vydání (`-rc.1`). Platí živě. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | `owner/name` na github.com, jehož vydání se kontrolují. Dokud je repozitář soukromý, GitHub odpovídá 404, pokud pro github.com není uložen token (Nastavení > Integrace); bez něj kontrola tiše selže a zapíše jeden řádek do protokolu aplikace. |

Správce může v souboru politiky platné pro celý počítač nebo v registru Windows nastavit `mode`, `channel` (jako `release_channel`) a `repo`, plus několik dalších voleb aktualizací. Klíč, který politika nastaví, má přednost před `config.toml` a v Nastavení se zobrazí jako **Nastaveno vaším správcem**. `mode` v politice je strop: můžete zvolit nižší hodnotu, nikdy vyšší. Viz [Podniková distribuce](enterprise-distribution.md#soubor-politiky-a-registr).

## `[privacy]` (citlivé soubory)

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Vzory cest ve stylu gitignore. Vzor bez `/` odpovídá názvu souboru v jakékoli hloubce, vzor s `/` celé cestě (úvodní `/` ho ukotví v kořeni); `*` a `?` zůstávají uvnitř jedné složky, `**` složky překračuje; `!` znovu povolí to, čemu odpovídal dřívější vzor, a vyhrává poslední shoda; velikost písmen se ignoruje. Používá je ochrana přidávání do indexu a `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Přidání souboru do indexu, který odpovídá vzoru nebo jehož obsah obsahuje rozpoznanou tajnou hodnotu, se nejprve zeptá (TUI), je odmítnuto, pokud není zadáno `--allow-sensitive` (CLI), nebo pokud `mcp.allow_sensitive_staging` nedovolí agentovi předat `allow_sensitive` (MCP). Čtou se textové soubory do 1 MiB každý a 32 MiB celkem; binární soubory se porovnávají jen podle názvu. Viz [Přidávání do indexu a commitování](staging-and-committing.md#citlivé-soubory). |

## `[conflict_prevention]` (seznamy základních větví)

Které větve smí proaktivní detekce konfliktů vzít za základ aktuální větve, je seznam vzorů (viz [Řešení konfliktů při slučování](merge-conflicts.md#prevence-konfliktů)). Seznamy mohou mít názvy a být sdílené; upravuje je Nastavení > Prevence konfliktů.

| Klíč | Výchozí | Hodnoty a účinek |
|---|---|---|
| `conflict_prevention.default_list` | `""` | Název seznamu, který repozitář používá, když nemá vlastní. Prázdný, nebo název, který žádný seznam nemá, znamená vestavěné `**` (každá větev). Čte se znovu při každé kontrole, takže změna platí pro otevřené repozitáře bez restartu. |
| `conflict_prevention.lists` | žádné | Pojmenované seznamy, každý jako jedna tabulka: `[conflict_prevention.lists.release]` s `patterns = ["origin/release/**", "!origin/release/old"]`. Prázdné `patterns` povolí každou větev. Úprava seznamu platí pro každý repozitář, který ho sleduje. |

Který seznam repozitář sleduje, je v jeho `repo-state.toml` (níže). Pořadí přednosti: vlastní vzory repozitáře, pak pojmenovaný seznam, který sleduje, pak `default_list`, pak `**`.

## `[keybindings]`

Přiřazuje zkratky TUI znovu; upravuje je Nastavení > Klávesnice a myš.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; hodnota nahradí výchozí klávesy
akce, odstranění řádku je obnoví. Akordy: `Ctrl+Shift+N`, `Alt+n`, `F2`,
`PageDown`, `Space`, `,`, `P` (= `Shift+p`). Chybný záznam nikdy nic neblokuje:
TUI ho ignoruje, ponechá výchozí a oznámí to ve stavovém řádku. Kontexty, id
akcí, výchozí hodnoty a pravidla konfliktů: [Klávesové zkratky](keybindings.md).

## Nastavení pro jednotlivé repozitáře: `.gitpoulpi/repo-state.toml`

Uloženo uvnitř repozitáře; složka ignoruje sama sebe (`.gitignore` s `*`), takže
se nikdy nezobrazí jako změna. Zapisuje ji TUI a CLI; upravovat ji je potřeba
zřídka.

| Klíč | Co to je | Nastavuje se |
|---|---|---|
| `[preferences] encoding` | Zde přepíše `git.default_encoding` | Nastavení > Git a kódování |
| `[preferences] lfs_enabled` | Zde přepíše `git.lfs_enabled` | Nastavení > Git a kódování |
| `[preferences] profile` | Id z `[[profiles]]`, které zde podepisuje commity; chybějící znamená `general.active_profile` | výběr profilu na panelu nástrojů, **Repozitář: Změnit profil...** |
| `[preferences] autocrlf`, `eol` | *Zatím se nepoužívá* | Nastavení > Git a kódování |
| `[preferences] push_after_commit` | Po každém commitu poslat větev do jejího upstreamu (bez záznamu vypnuto) | Nastavení > Nastavení repozitáře > Commit |
| `[preferences] skip_hooks_by_default` | Formulář commitu začíná se zaškrtnutým Přeskočit hooky (bez záznamu vypnuto) | Nastavení > Nastavení repozitáře > Commit |
| `[preferences] squash_merge` | Merge lokální větve dělá squash: přidá do indexu, bez merge commitu (bez záznamu vypnuto) | Nastavení > Nastavení repozitáře > Možnosti merge |
| `[preferences] template_apply`, `template_strip_comments` | Předvyplnit prázdný formulář commitu šablonou; při commitu odstranit řádky komentářů `#` (obojí bez záznamu vypnuto) | Nastavení > Nastavení repozitáře > Šablona zprávy |
| `[preferences] conflict_detection` | Proaktivní detekce konfliktů pro tento repozitář; bez záznamu platí `general.conflict_detection` | Nastavení > Nastavení repozitáře > Prevence konfliktů |
| `[preferences] conflict_base_branches` | Vlastní vzory Základních větví repozitáře, jeden na řádek (dál se používají jako **Vlastní seznam**) | Nastavení > Nastavení repozitáře > Prevence konfliktů |
| `[preferences] conflict_base_source` | Odkud Základní větve pocházejí: `default` (globální výchozí seznam), `list` (`conflict_base_list`) nebo `own` (`conflict_base_branches`). Bez záznamu: vlastní vzory, pokud nějaké jsou, jinak výchozí | Nastavení > Nastavení repozitáře > Prevence konfliktů |
| `[preferences] conflict_base_list` | `[conflict_prevention.lists.<name>]`, který tento repozitář sleduje, když je zdrojem `list`; smazaný seznam se vrátí k výchozímu | Nastavení > Nastavení repozitáře > Prevence konfliktů |
| `commit-template.txt` (soubor vedle `repo-state.toml`) | Šablona commitu; ukazuje na ni `commit.template` gitu | Nastavení > Nastavení repozitáře > Šablona zprávy |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: instrukce AI tohoto repozitáře | Nastavení > Nastavení repozitáře > AI |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` pro jednu akci zde | Nastavení > Nastavení repozitáře > AI |
| `[preferences] merge_fast_forward` | Přepíše zde `git.merge_fast_forward`: `never`, `allowed` nebo `only` | Nastavení > Nastavení repozitáře > Možnosti merge |
| `[preferences] default_push_targets` | Remotes, na které nová větev také pushuje; zkopírují se do větve, až poprvé dostane upstream | Nastavení > Nastavení repozitáře > Git a kódování; viz [Několik remote](multiple-remotes.md) |
| `[preferences.ai_review]`, `[preferences.ai_hosting_reply]` | Vlastní hodnoty tohoto repozitáře pro klíče `[ai.review]` a `[ai.hosting_reply]`; chybějící klíče dědí globální | Nastavení > Nastavení repozitáře > AI |
| `hosting_post_as`, `hosting_identity` | Jméno, pod kterým se zde vaše zprávy odesílají hostiteli (přepisuje `general.hosting_post_as`), a login, který se GitPoulpi naučí z první zprávy odeslané botovým tokenem, když žádný není nastaven | Nastavení > Nastavení repozitáře > SSH a přihlášení; viz [Konverzace](conversations.md#odesílání-jako) |
| `assignment_blocked` | V issues a pull requestech tohoto repozitáře nelze z GitPoulpi nikomu nic přiřadit | `gitpoulpi hosting assignment --block \| --allow` |
| `board_labels` | Štítky, které jsou sloupci tabule issues u hostitele bez workflow, v pořadí tabule | tlačítko **Sloupce** tabule |
| `conversation_seen`, `conversation_events`, `posted_comment_ids` | Co jste naposledy přečetli v každé konverzaci (zbytek je nepřečtený), zda se zobrazují systémové události, a komentáře, které tato instalace odeslala (ty smí bot upravit nebo smazat) | zapisuje pohled konverzace |
| `reviews` | Místní stav revize v aplikaci pro každý požadavek: čekající komentáře, zobrazené soubory | zapisuje pohled revize |
| `default_remote` | Remote, který používají jednoklikové Push/Pull/Fetch | nabídka remote, `gitpoulpi remote default <name>` |
| `hosting_integration` | Zde připnutý profil `[[integrations]]` | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Projekt poskytovatele, který čtou seznamy PR / issues, jako `{ integration, project, filter }`. `filter` je volitelný: další klauzule v syntaxi poskytovatele (fragment JQL Jiry, podmínka WIQL Azure, názvy seznamů Trella) připojená přes AND k dotazu na seznam; GitHub a GitLab ji ignorují. | sekce postranního panelu, `gitpoulpi hosting link` |
| `ref_visibility` | Skryté / solo refs | postranní panel Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotes skryté v grafu jako celek (každá větev, i ty načtené později) | ⊙ a nabídka remote, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Sbalené sekce postranního panelu a složky větví a tagů. `[folds]` obsahuje také `status_folders` (sbalené složky stromů souborů Unstaged / Staged) a `commit_folders` (sbalené složky stromu změněných souborů commitu, stejné pro každý commit), každé omezené na 500 cest, nejstarší se zahazují první | postranní panel, stromy souborů |
| `pending_auto_stash` | Stash, který GitPoulpi vytvořil a nevrátil | zapisuje daemon; viz [Auto-stash](log-and-history.md#auto-stash) |

## Další proměnné prostředí

| Proměnná | Účinek |
|---|---|
| `GITPOULPI_CONFIG_DIR` | Konfigurační složka (viz výše). |
| `GITPOULPI_TOKEN_<ID>` | Token hostingu pro profil `<ID>`, nad uloženým (CI, jednorázové použití). |
| `GITPOULPI_GIT` | Spouštěný program `git` (výchozí: `git` na `PATH`; ve Windows skutečný `mingw64\bin\git.exe` za zástupcem `cmd\git.exe` z Git for Windows, což ušetří jeden proces na volání). |
| `GITPOULPI_LANG=fr` | Jazyk rozhraní, nad `ui.language` (libovolná hodnota `ui.language`; `qps` je pseudo-locale pro překladatele). Neznámá nebo prázdná hodnota se ignoruje. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Žádné obrázky vůbec, ať konfigurace říká cokoli. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Nápovědy k rozlišení obrázku (velikost buňky v pixelech; nadvzorkování, výchozí 2, `1` šetří CPU). |
| `GITPOULPI_NO_BROWSER=1` | Nikdy nespouštět webový prohlížeč (URL se stále zobrazí). |
| `GITPOULPI_TIMING=1` | Zapisovat časování požadavků do `gitpoulpi-timing.log` v dočasné složce (zprávy o výkonu). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Nečekat na Enter po ukončení TUI s chybou. |
