[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · **Svenska** · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Konfigurationsreferens

Varje nyckel i `config.toml`, dess standardvärde och vad den gör. Du behöver sällan
redigera filen för hand: [inställningsskärmen](settings.md) (`Ctrl+,`) skriver den
medan du ändrar saker, och det gör `gitpoulpi config set` också.

## Var den finns och hur ett värde väljs

| | |
|---|---|
| Fil | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` skriver ut den. |
| En annan mapp | `GITPOULPI_CONFIG_DIR=<dir>` flyttar hela konfigurationsmappen (filen, `sessions/`, daemonens portfiler): en portabel installation, ett test eller en agent som hålls borta från dina egna inställningar. |
| Företräde | inbyggt standardvärde, sedan `config.toml`, sedan variabeln `GITPOULPI_*`, sedan en kommandoradsflagga. Det högsta vinner. Ett felaktigt värde (ett felstavat val i filen, en dålig variabel) är ett fel, aldrig tyst ignorerat. |
| Variabler | Versalisera nyckeln och gör punkter till understreck: `general.auto_prune` blir `GITPOULPI_GENERAL_AUTO_PRUNE`. Bara för nycklar som GitPoulpi faktiskt läser (inte de som nedan är markerade *används inte*, och inte listor som `ui.log_columns_hidden`). |
| Flaggor | `--backend git2\|system` åsidosätter `general.default_backend` för en daemon som det kommandot startar. |
| Token och nycklar | Aldrig i den här filen. Hosting-token och AI-API-nycklar går till din system-credential helper (Git Credential Manager, libsecret / `pass`, Keychain); filen innehåller bara en referens (`token_ref`). Utan helper misslyckas sparandet av en token med ett meddelande, och token gäller bara under sessionen. |

```
gitpoulpi config path                          # filen, och om den finns ännu
gitpoulpi config get general.auto_stash        # värdet, lagret det kom från (default/file/env/flag) och om något läser det
gitpoulpi --json config get ui                 # en hel sektion
gitpoulpi config set general.auto_stash always # kontrolleras mot schemat och skrivs sedan
```

`config set` håller filen gles (bara nycklar som någon har satt) och behåller nycklar
den inte ombetts röra, men tappar kommentarer. Den vägrar listor och hela tabeller,
utom `keybindings.*` (flera tangentkombinationer åtskilda av mellanslag; tomt tar bort
bindningen). Redigera `[[profiles]]`, `[[integrations]]` och `[[ai.profiles]]` i
Inställningar eller för hand.

I tabellerna betyder **Omstart** att värdet läses vid start; allt annat gäller direkt
(eller vid nästa uppdatering, fetch eller när Inställningar stängs).

## `[general]`

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, med `git`-programmet som reserv för staging, merge, rebase, stash och nätverk) eller `system` (bara det installerade `git`). En körande daemon behåller sin backend: **starta om** den (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | När lokala ändringar blockerar en checkout, pull, merge eller rebase: `ask` (en dialog i TUI:t; CLI och MCP misslyckas och namnger `--autostash`), `always` (stash, kör, pop) eller `never` (vägra). Se [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | Auto-stashen tar också med ospårade filer. |
| `general.initial_commits_in_graph` | `5000` | Commits som Log laddar först (minst 500). Högre kostar minne och ritningstid på stora repositories. |
| `general.lazy_load_commits_in_graph` | `true` | Ladda äldre commits när du rullar nära slutet. Av: Log stannar vid det första antalet. |
| `general.commits_per_lazy_page` | `1000` | Commits per bakgrundssida (minst 50). |
| `general.max_commits_in_graph` | `100000` | Var lazy loading stannar (cirka 300 byte per commit); `0` = ingen gräns. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All i bakgrunden var N:e minut, 0 till 60; `0` = av. **Omstart.** |
| `general.auto_fetch_interval_unit` | `"auto"` | Enheten som Inställningar visar intervallet i: `minutes`, `hours` eller `auto`. Bara visning: intervallet är alltid `auto_fetch_interval_minutes` minuter. `auto` (och en konfiguration utan nyckeln) väljer den största enhet som delar minuterna jämnt (60 visas som ”1 timme”, 45 som ”45 minuter”); en enhet som inte passar ignoreras. Inställningar skriver båda nycklarna; en äldre GitPoulpi ignorerar enheten och läser fortfarande minuterna. |
| `general.fetch_on_tab_open` | `true` | Fetch All när en flik öppnas (hoppas över om GitPoulpi hämtade det repositoryt för mindre än 60 s sedan). |
| `general.auto_prune` | `true` | Fetch rensar också remote-spårande grenar som inte längre finns. |
| `general.auto_prune_tags` | `false` | Ett fetch tar också bort de lokala taggar som remoten har tagit bort (bara taggar som fanns på den; en tagg du aldrig pushat behålls). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Uppdatering av listorna med pull requests / ärenden och verktygsfältets chip, 0 till 120; `0` = bara på begäran och vid händelser. |
| `general.hosting_refresh_unit` | `"auto"` | Enheten som Inställningar visar uppdateringen i: `minutes`, `hours` eller `auto`, läst som `general.auto_fetch_interval_unit`. Bara visning. |
| `general.hosting_post_as` | `""` | Namnet som meddelanden till en värd publiceras under när repositoryts token tillhör en bot (en GitHub App-token), till exempel `my-app[bot]`: visas i varje textruta och används för att känna igen dina egna kommentarer. Tomt: efterfrågas från värden, annars inlärt från det första inlägget. Ett repository kan åsidosätta det (Inställningar, repositoryts SSH och inloggning). |
| `general.keep_submodules_up_to_date` | `false` | Efter en checkout eller pull, `git submodule update --init --recursive` när en submodul inte är i takt. Se [Submoduler](submodules.md). |
| `general.default_branch_name` | `"main"` | Första gren i ett repository som skapas med Init; tomt låter git bestämma. Byter inte namn på något. |
| `general.delete_orig_files_after_merge` | `false` | Efter en merge som lyckas, ta bort `*.orig`-säkerhetskopiorna. |
| `general.remember_tabs` | `true` | Öppna förra sessionens flikar igen. När den är av öppnar en start i ett repository bara det repositoryt, och en start utanför ett repository öppnar bara Hem. **Omstart.** |
| `general.radar_enabled` | `true` | [Radar](radar.md): dess flik, palettkommandon, `Alt+R` och dess hämtning i bakgrunden. Av döljer allt detta och inget hämtas för den. Gäller direkt; visas inte i ett bygge utan funktionen `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | Fliken som en start öppnar på när mappen den startar i ligger i ett repository: `"current-repo"` (det repositoryts flik, öppnad vid behov), `"last"` (fliken som var aktiv vid senaste avslutet), `"home"` eller `"radar"`. Ett repository som anges med `--repo` öppnar alltid sin egen flik. `"last"` blir Hem med `remember_tabs` av eller när inget är sparat; `"radar"` blir Hem när Radar är av eller saknas i bygget. Läses en gång vid start; kräver omstart. |
| `general.startup_tab_outside_repo` | `"home"` | Fliken som en start öppnar på när mappen den startar i inte är ett repository (till exempel en användarmapp): `"home"`, `"last"` eller `"radar"`. `"last"` blir Hem med `remember_tabs` av eller när inget är sparat; `"radar"` blir Hem när Radar är av eller saknas i bygget. Läses en gång vid start; kräver omstart. |
| `general.startup_tab_launcher` | `"last"` | Fliken som en start öppnar på när GitPoulpi startas från genvägen på skrivbordet eller i Startmenyn som det skapade (den skickar `--launcher`), oavsett mapp: `"home"`, `"last"` eller `"radar"`. Har företräde framför de två inställningarna ovan. `"last"` blir Hem med `remember_tabs` av eller när inget är sparat; `"radar"` blir Hem när Radar är av eller saknas i bygget. Läses en gång vid start; kräver omstart. |
| `general.group_worktree_tabs` | `true` | Håll flikarna för ett repositorys länkade worktrees direkt efter repositoryts egen flik: när en worktree öppnas, när en worktree-flik dras bort (den hoppar tillbaka intill sin grupp; repositoryts flik är gruppens huvud och dess worktrees följer den) och när inställningen slås på. Av lämnar flikarna där du placerat dem. |
| `general.undo_depth` | `100` | Hur många operationer Ångra kan gå tillbaka; `0` = obegränsat. |
| `general.conflict_detection` | `true` | Proaktiv konfliktdetektering: ett märke i Log när en merge av den aktuella grenen i grenen den startade från skulle ge konflikter, beräknat i bakgrunden (en merge i minnet: inget ändras i dina filer, ditt index eller dina refs). Ett repository kan stänga av den för sig själv (`[preferences] conflict_detection`). Basgrenarna den jämför med har egna listor: [`[conflict_prevention]`](#conflict_prevention-listor-över-basgrenar). Se [Lösa merge-konflikter](merge-conflicts.md#konfliktförebyggande). |
| `general.dev_mode` | saknas (automatiskt) | För dig som bygger GitPoulpi från en utcheckning; visas aldrig i Inställningar. Saknas: på när den körbara filen som körs ligger i `<checkout>/target/<profile>/` (`Cargo.toml` och `crates/gitpoulpi-tui` bredvid `target`). `true` / `false` tvingar fram läget. Utvecklarläget låser bara upp `ui.launcher_build`. |
| `general.journal_outside` | `true` | Registrera det ett skal eller en AI-agent gör med grenar, taggar, stashar och HEAD som poster i ångrahistoriken, markerade utanför, så att `Ctrl+Z` kan ta tillbaka det. Ändringar av filer fångas aldrig. Kräver `undo_enabled`. Se [Ångra och gör om](undo-redo.md#ändringar-gjorda-utanför-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: ingen bokföring för ångra alls (snabbast), ångrahistoriken raderas, knapparna Ångra/Gör om försvinner. Se [Ångra och gör om](undo-redo.md). |
| `general.active_profile` | `""` | Id för den `[[profiles]]`-identitet commits görs med. Tomt, eller en profil med tomt namn eller e-post: gits `user.name` / `user.email`. |
| `general.external_editor` | `""` | Kommando för en fils *Öppna i redigerare*, t.ex. `code -r {file}`. Tomt: VS Code om det är installerat, annars systemets standard. |
| `general.external_diff_tool` | `""` | Kommando för *Öppna i externt diff-verktyg*: `{local}` (committad / HEAD-version), `{remote}` (arbetskatalogens / indexets version). Tomt: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Kommando för *Öppna i externt merge-verktyg* på ett konfliktkort (en binär, LFS- eller för stor fil): `{base}`, `{local}` (A, nuvarande), `{remote}` (B, inkommande) och `{merged}` (filen som verktyget skriver resultatet till; GitPoulpi stagear den när verktyget avslutas utan fel). Tomt: `git mergetool --no-prompt`. |
| `general.file_explorer` | per OS | Kommando för *Visa i mapp*: `{path}` (filen), `{dir}` (dess mapp). Standardvärden: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

De tre kommandomallarna delas först upp i argument (`"..."` grupperar ett ord), och
sedan fylls platshållarna i, så sökvägar med mellanslag förblir ett enda argument.
Inget skal är inblandat.

```toml
[[profiles]]                # commit-identiteter (Inställningar > Profiler)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log & graf**

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (en bild där terminalen kan rita en, tecken annars), `image`, `text`. **Omstart.** |
| `ui.image_protocol` | `"auto"` | Hur bilder (graf, avatarer) når terminalen: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = Kitty-protokollet i kitty, Ghostty, iTerm2 3.7.7 eller nyare och en herdr med `kitty_graphics = true`; iTerm2-protokollet i WezTerm och Rio (transparens blandas), äldre iTerm2 och mintty (ogenomskinlig bakgrund); sixel (grafkolumnen och avatarerna) i Windows Terminal, foot, Tabby, Konsole, mlterm, i tmux när terminalen bakom tar sixel och i varje terminal vars svar vid start nämner sixel (en xterm startad med `-ti vt340`, en VS Code-terminal med bilder på); tecken i zellij, screen och okända terminaler. I tmux kräver kitty-bilder `kitty` här plus `allow-passthrough on` ([Multiplexers](multiplexers.md)). Där terminalens pseudokonsol tappar eller felplacerar bilder (WezTerm 20240203, Rio och mintty på Windows) använd `ui.graph_renderer = "text"` ([Välja terminal](terminals.md#windows)). **Omstart.** |
| `ui.theme` | `"system"` | Färgtemat: en inbyggd slug (`gitpoulpi theme list` listar alla), sluggen för en fil i `<config>/themes/`, eller `system` (följer terminalens ljusa eller mörka bakgrund, avläst vid start; det gamla värdet `default` betyder detsamma). `config set` avvisar okända namn. Se [Teman](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Spårpalett: `theme` (följ temat) eller vilket spårpalettnamn som helst: `gitpoulpi`, `light`, `material`, `serie` och de andra temans paletter. |
| `ui.graph_lane_background` | `"off"` | `off` eller `row_tint` (ett svagt band i spårfärgen bakom grafen, plus en stapel vid dess kant). **App: Växla radfärgning i grafen** (`Ctrl+N Shift+Y`) växlar den; **App: Byt färgtema för grafen...** (`Ctrl+N Shift+T`) väljer `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (bara med `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` och `right` ritar en kolumn; ett fönster smalare än 80 kolumner faller tillbaka på `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: om en dialogruta du flyttat eller ändrat storlek på (dra i titelraden eller kanterna, eller `Ctrl+N m`) behåller sin plats. En inställning för alla dialogrutor (var och en behåller sin egen plats och storlek): tills GitPoulpi stängs (standard), på disk till nästa start (alla dialogrutor i den enda filen `dialog_geometry.txt` i datamappen) eller inte alls (varje dialogruta öppnas på sin standardplats och i sin standardstorlek). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: var knapparna sitter längs verktygsfältet (vänster, mitten, höger i en rad; överst, mitten, nederst i en kolumn). |
| `ui.toolbar_pr_pinned` | `true` | Pull request-knappen är fäst längst bort i verktygsfältet; `false` låter den flöda med de andra knapparna. |
| `ui.statusbar_position` | `"bottom"` | `top` (ovanför repositoryfliken) eller `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (fönstrets sista rad, under statusraden) eller `left` (en kolumn vid vänsterkanten; under 80 kolumner stannar den överst). |
| `ui.graph_show_stashes` | `true` | En rad per stash i Log. |
| `ui.ref_connector` | `true` | En tunn linje från en ref-pill till dess commit. |
| `ui.column_resize` | `"line"` | Hur en kolumngräns greppas i Log, Radar och commit-kolumnen i diff-, fil- och blame-vyerna. `line`: varje gräns är ett mellanrum på en cell med en liten markering i rubriken; medan pekaren står på gränsen, i rubriken eller var som helst nedåt i tabellen, ritas en tunn accentlinje över hela höjden och en dragning från var som helst längs den ändrar storlek. `header`: inget mellanrum och ingen markering; första tecknet i rubrikcellen efter gränsen lyses upp och en dragning därifrån ändrar storlek (bara i rubriken). Tangentbordets storlekstangenter och palettens rader är desamma i båda lägena. |
| `ui.ref_tree` | `true` | Gruppera grenar och taggar på `/` i mappar i Refs-sidofältet (`y` växlar). |
| `ui.tree_folders_first` | `true` | I trädvyn i filrutorna (Ändringar, detaljer, jämförelse) listar en mapp sina undermappar före sina filer; `false` behåller den vanliga sökvägsordningen. Den platta listan är oförändrad (Inställningar > Log & graf). |
| `ui.graph_ghost_ref` | `true` | Log-vyns Ref-kolumn visar ett bleknat piller med namnet på den gren (annars fjärrgren) som en commit som pekas på tillhör, när commiten saknar egen ref och kolumnen har plats; aldrig en tagg. En borttagen gren härleds ur ämnesraden på den merge-commit som slog ihop den och ritas genomstruken. När du pekar på pillret visas dess typ. |
| `ui.reveal_on_ref_click` | `true` | Ett enkelklick på en ref (gren, tag, remote-gren, stash) i Refs-rutan rullar Log till dess commit och blinkar raden; commit-markeringen flyttas inte. |
| `ui.log_wip_counts_in_message` | `true` | WIP-antal ändringar i WIP-meddelandet medan kolumnen Ändringar är dold. |
| `ui.log_columns_layout` | `"fit"` | Hur Log lägger ut sina kolumner. `fit`: varje synlig kolumn ryms exakt i rutan och bara `message` är flexibel; när rutan är smal viker sig hela kolumner. `scroll`: kolumnerna behåller sina bredder (`message` också), inget viker sig, och tabellen rullar i sidled när den är bredare än rutan. |
| `ui.log_column_widths_scroll` | tom | Äldre: Logs kolumnbredder gäller nu per repository, i dess `repo-state.toml` (`[log_columns]`, mapparna `fit` och `scroll`, namn -> celler; `scroll` kan innehålla `message`, `fit` aldrig). Ett repository utan sparade bredder använder de äldre `ui.log_column_widths` / `ui.log_column_widths_scroll` om du har dem, annars anpassas det efter sitt innehåll vid första öppningen. |
| `ui.log_columns_hidden`, `ui.log_columns_order` | tom | Log-kolumnerna (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`) som visas och deras ordning, skrivs när du döljer eller drar en kolumn; gemensamt för alla repositoryn. `message` kan inte döljas. `ui.log_column_widths` är äldre (se ovan). |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | tom | Delarna i [commit-kolumnen](interface-tour.md#commit-kolumnen) bredvid en diff, för sig och skilda från Logs: de visade delarna med namn (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; tom betyder spår, meddelande, författare och datum), deras ordning från vänster till höger med samma namn (tom är ordningen just nämnd; ett utelämnat namn följer i den ordningen) och de dragna bredderna i celler per del (en del som utelämnas är automatisk), skrivs när du väljer, flyttar eller ändrar storlek på delarna. `message` kan inte döljas. En sessionsfil som skrevs innan nycklarna fanns läses en gång in i dem. |
| `ui.sidebar_sections_hidden` | tom | Sektionerna i vänsterpanelen som du dolt (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), skrivs från panelens högerklicksmeny eller paletten. Minst en befintlig sektion visas alltid. Detta är standarden: varje repository kan dölja egna avsnitt (sparas i dess tillståndsfil) och nyckeln gäller för ett repository utan eget val; *Använd för alla repositories* skriver den. |
| `ui.sidebar_sections_scope` | `"repository"` | Vems val avsnitten i vänsterpanelen är: `repository` (varje repository behåller sitt eget, med `ui.sidebar_sections_hidden` som standard) eller `global` (alla repositories visar `ui.sidebar_sections_hidden`, och en ändring skrivs dit). Att växla förlorar aldrig någon av uppsättningarna: valen per repository ligger kvar i deras tillståndsfiler medan `global` är på. Inställningar: *Avsnitt i sidopanelen*. |
| `ui.language` | `"auto"` | Gränssnittets språk: `auto` (systemets språk, annars engelska), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Direkt.** `GITPOULPI_LANG` åsidosätter det. Ett språk som inte finns i det här bygget visar engelska. **App: Byt språk...** (`Ctrl+N l`) ställer också in det, med live-förhandsvisning. CLI- och MCP-utdata förblir på engelska. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... förblir engelska i alla språk. `true`: det valda språkets git-ordförråd (franska ”remiser”, ”étiquette”, ”valider”...). **Direkt.** |
| `ui.git_terms_english` | `false` | Bara ja, ko och zh-CN: visa de engelska git-orden (stage, unstage, staged, unstaged...) i stället för deras transkription. Ingen effekt medan `ui.translate_git_terms` är `true`. **Direkt.** |
| `ui.mouse_enabled` | `true` | `false` lämnar musen (och textmarkering) åt terminalen. **Omstart.** |
| `ui.mouse_scroll_lines` | `3` | Log-rader per hjulsteg, 1 till 20. **Omstart.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` frågar terminalen efter kitty-tangentbordsprotokollet (så att `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` kommer fram; bara Unix, tmux svarar aldrig: se [Multiplexers](multiplexers.md#tmux)); `off` frågar aldrig. **Omstart.** |
| `ui.logo_color` | `""` | Logotypens färgsättning i Om-fönstret, en slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); tomt eller okänt betyder byggets standard (`lavender`). Skrivs när du klickar på logotypen eller väljer en färg i Om-fönstret. |

**Avatarer** (av som standard; att slå på dem skickar en SHA-256-hash av författarnas
e-postadresser till Gravatar, och frågar ditt anslutna GitHub-/GitLab-konto)

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (i commit-cirkeln i bildgrafen), `author` (i kolumnen Författare), `off`. Den gamla `ui.show_avatars` (`true`) läses fortfarande som `author` medan `ui.avatars` saknas. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Uppslagsordning: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. En GitLab-värd tillfrågas bara via en ansluten profil. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (genereras lokalt) eller `initials`. På en terminal utan bildprotokoll används alltid initialmärket. |

**Diff- och redigerarvisning**

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` eller `split` (`m` i en diff). |
| `ui.diff_hunks_only` | `true` | `false` visar hela filen med ändringarna på plats (`H`). |
| `ui.diff_show_whitespace` | `true` | Visa ändringar som bara rör blanksteg (`s`). |
| `ui.diff_word_wrap` | `false` | Bryt långa diff-rader (`w`; Inställningar > Diff & redigerare). |
| `ui.minimap` | `false` | En minikarta bredvid rullningslisten i diffen och redigeraren (`M`). |
| `ui.mermaid_detail` | `refs` | Detaljnivå för Logs Kopiera som Mermaid gitGraph: `structure`, `refs`, `subjects`, `full` (kommer ihåg från popupen; standard för `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Den grafens riktning: `lr` (vänster till höger) eller `tb` (uppifrån och ned). |
| `ui.mermaid_fence` | `false` | Omge det kopierade Mermaid-skriptet med ett `mermaid`-kodblock. |
| `ui.mermaid_subject_width` | `40` | Ämnesraderna kortas till så här många tecken i Mermaid-id:n (`subjects`, `full`). |
| `ui.text_cursor` | `"terminal"` | Var skrivpositionen i ett fokuserat textfält visas: `terminal` placerar terminalens egen markör där (ett blinkande streck, så terminalens markörinställningar och en inmatningsmetods kandidatfönster följer med; dold när inget fält har tangentbordet och återställd till din egen markörstil vid avslut), `drawn` ritar GitPoulpis egen markörglyf och håller terminalmarkören dold. Direkt. |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` eller `16`: hur många färger som ritas med. `auto` är truecolor utom i Apple Terminal (256), Linux-konsolen (16) och `*-256color`-terminaler utan `COLORTERM` (256). Direkt. Se [Teman](themes.md). |
| `ui.input_color` | `"theme"` | Textfältens bakgrund: `theme` (temats egen), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | Färgen `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | AI-commit-composern öppnas med varje förslags filer `expanded` eller `collapsed`. |
| `ui.blame_side` | `"right"` | Vilken sida av Blame-vyn som bär annoteringskolumnen (avatar, författare, ålder, sammanfattning): `right` (standard) eller `left`. |
| `ui.blame_width` | `46` | Blames annoteringskolumns bredd i celler (24 till 90). Dra dess kant eller tryck `<` / `>` i vyn; den nya bredden sparas här. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request på GitHub, merge request på GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Döljer verktygsradens chip för den utcheckade grenens senaste pull request eller merge request när den är stängd eller mergad; den dämpade chipen "+ PR" (starta en ny) visas i stället, som om grenen inte hade någon. `false`: chipen visar alltid den senaste begäran, oavsett tillstånd. |

De diff-växlare du ändrar inne i en diff sparas här direkt och används av varje diff
som öppnas efteråt.

**Datum**: `ui.date_format` (`"YYYY-MM-DD"`, Blame / Filhistorik-datumkolumnen),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, Logs Datum-kolumn) och
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, commit-detaljerna, Datum-tooltipen
och *Kopiera > Datum*). Tokens: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (veckodag, hel / förkortad) `HH` `mm` `ss`; allt annat kopieras som det
är. Månads- och veckodagsnamn följer `ui.language`, och en månad skriven efter ett
dagsnummer är genitivformen där språket har en (ryska, ukrainska, polska, tjeckiska).
En `D MMMM YYYY`-sekvens skrivs med året först på japanska, kinesiska och koreanska
(`2024年3月5日`). `ui.date_locale` (`"system"`, standard, eller `"english"`) håller
namnen engelska oavsett språk.

**Agenter, terminaler och flikens mapp**

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | Fönstret tar överhuvudtaget emot `ui show` / `ui propose` från agenter (Inställningar > Agenter & MCP > *Låt AI-agenter styra vyn*). Av, ingen agent når det. Se [MCP och agenter](mcp-and-agents.md#att-visa-saker-för-användaren). |
| `ui.launch` | `"off"` | Om en agent får öppna ett GitPoulpi-fönster när inget är öppet: `off`, en förinställning (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) eller `custom`. |
| `ui.launch_command` | `""` | Mallen `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Delas upp i argument, inget skal. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (flikraden erbjuder *Avsluta, cd hit*) eller `auto` (varje avslut byter skalets mapp, och en ny flik kör standardposten i *Öppna i*). Se [Följa den aktiva flikens mapp](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Äldre, se nedan och [Öppna i](open-in.md). Hur den gamla *Terminal här* öppnade en terminal: `auto` (förinställningen `ui.launch`, annars `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), en förinställning eller `custom`. |
| `ui.terminal_launch_command` | `""` | Mallen `custom`; `{repo}` är mappen. |
| `ui.open_in` | *(saknas)* | Listan [Öppna i](open-in.md), en array av tabeller `[[ui.open_in]]` med `label` och `command`. Saknas den betyder det ”inte konfigurerad ännu”: listan beräknas från de äldre `ui.terminal_launch*`-nycklarna (ett värde som inte är `auto` blir första posten), OS:ets filhanterare, multiplexrarna och terminalerna som upptäcks i miljön (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) och redigerarna som finns i `PATH`. Första sparandet från Inställningar skriver listan. `open_in = []` är en tom lista. Kan inte sättas med `config set` (redigera filen). |
| `ui.open_in_default` | `""` | Etiketten på posten som ”Öppna i standard” kör; tomt eller okänt betyder första posten. |
| `ui.launch_terminal` | `"auto"` | Terminalen som skrivbordsgenvägarna (Inställningar > Startare) och omstarten på Windows startar GitPoulpi i. `auto` väljer bland det som är installerat (Windows: Windows Terminal, annars WezTerm, Alacritty, den vanliga konsolen; Linux: `xdg-terminal-exec`, annars den första terminal som hittas; macOS: iTerm2, Ghostty, WezTerm eller kitty om de är installerade, annars Terminal), `default` är operativsystemets egen, eller ett terminal-id som `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Bara i utvecklarläge (se `general.dev_mode`; visas i Inställningar > Startare bara då, ignoreras annars). `"current"`: genvägarna och kommandoaliaset startar det bygge som skapade dem. `"newest"`: de startar det senast byggda av `target/dist` och `target/release` i utcheckningen, avgjort vid varje klick, och att starta något av byggena skriver aldrig om dem. |
| `ui.command_alias` | `""` | Namnet på kommandot som startar GitPoulpi från en terminal, till exempel `gp` (Inställningar > Startare, `gitpoulpi alias create`). Tomt: inget. Bara bokstäver, siffror, `-`, `_` och `.`. Det är bara namnet som knappen **Skapa: kommandoalias** använder; inget finns förrän du skapar det. |
| `ui.command_alias_add` | `""` | Namnet på det andra kommandot, till exempel `gpa`, som kör `gitpoulpi --add`: det öppnar den aktuella mappen som en ny flik i den GitPoulpi som redan körs (Inställningar > Startare, `gitpoulpi alias create gpa --add`). Tomt: inget. Samma namnregler; skapas tillsammans med `ui.command_alias` av knappen **Skapa**. |
| `ui.relaunch_in_terminal` | `true` | Endast Windows: när exe-filen startas med dubbelklick i en egen konsol öppnar GitPoulpi sig igen i `ui.launch_terminal` och avslutas. En terminal som kör exe-filen direkt (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio, Tabby, Alacritty `-e`) är också en egen konsol, men inte ett klassiskt konsolfönster: den startas aldrig om. Av: den stannar i den konsolen. |
| `ui.reuse_window` | `true` | Ett klick på en GitPoulpi-genväg (`gitpoulpi launch`) visar fönstret för en GitPoulpi som redan är öppen i stället för att starta en till. Av: varje klick startar ett nytt fönster. |

**`ui.open_in`-kommandon** delas först upp i argument (blanksteg, `'...'`, `"..."`), och sedan ersätts variablerna inuti varje argument, så en mapp med mellanslag förblir ett enda argument; inget skal är inblandat om inte kommandot startar ett (`sh -c '... "$GITPOULPI_CWD"'`). Variabler: `${cwd}` (flikens mapp), `${repo}` (repositoryts rot), `${name}` (mappens basnamn), `${branch}` (tom när frånkopplad), `${web_url}` (webbsidan för `origin`-remoten när den är ett GitHub- eller GitLab-projekt, annars tom; en post som använder den vägrar köras när den är tom), `${env:NAME}` (tom när den inte är satt). Det gamla `{repo}` betyder fortfarande `${cwd}`. Varje annan `${...}` är ett fel som namnger den. Exempel:

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

Äldre: `ui.terminal_launch` och `ui.terminal_launch_command` läses fortfarande (de ger listan dess första innehåll medan `ui.open_in` saknas: `custom` blir ”Terminal (custom)” med `{repo}` omskrivet till `${cwd}`, ett förinställnings-id blir den förinställningen) men ersätts av `ui.open_in`.

### Bilder

| Nyckel | Standard | Betydelse |
|---|---|---|
| `ui.image_view` | `true` | Visa en binärfil som är en bild (PNG, JPEG, GIF, WebP, BMP, ...) som en bild i Diff-vyn och filvyn. Av: det vanliga binärmeddelandet. |
| `ui.image_view_max_mib` | `32` | Bilder större än detta (MiB) avkodas inte; ett infokort visas i stället. |
| `ui.image_view_max_unit` | `"auto"` | Enheten som Inställningar visar gränsen i: `mib`, `gib` (1024 MiB) eller `auto`. Bara visning: gränsen är alltid `image_view_max_mib` MiB, 1 till 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | På en terminal utan bildprotokoll: `halfblock` (färgade halvblockstecken) eller `card` (bara infokort). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom från vilken förstoring visar fyrkantiga pixlar i stället för utjämning: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Före- och efter-rutorna delar en zoom och rullningsposition. |
| `ui.image_editors` | *(saknas)* | Bildredigerarna för ”Redigera i”, en array av tabeller `[[ui.image_editors]]` med `label` och `command`. Saknas den betyder det ”inte konfigurerad ännu”: listan är redigerarna som upptäcks på den här datorn (program i `PATH`, Flatpak-installationer, kända installationsmappar på Windows, `/Applications` på macOS), sedan OS:ets standardapp. Första sparandet från Inställningar skriver listan. `image_editors = []` är en tom lista. Kan inte sättas med `config set` (redigera filen). |
| `ui.image_editor_default` | `""` | Etiketten på redigeraren som ”Redigera i standard” kör; tomt eller okänt betyder första posten. |

**`ui.image_editors`-kommandon** delas först upp i argument (blanksteg, `'...'`, `"..."`), och sedan ersätts variablerna inuti varje argument, så en sökväg med mellanslag förblir ett enda argument; inget skal är inblandat. Variabler: `${file}` (bildens absoluta sökväg), `${dir}` (dess mapp), `${name}` (dess filnamn), `${repo}` (repositoryts rot, tom för en temporär kopia), `${env:NAME}` (tom när den inte är satt). Varje annan `${...}` är ett fel som namnger den. En bild som inte är arbetskatalogens fil (en commits version, indexet, en borttagen fil) öppnas som en **skrivskyddad temporär kopia**, så en redigerare kan inte röra repositoryt. Redigerare utan kommandorad (Affinity Photo från Microsoft Store, vissa Photoshop-installationer) finns inte bland förinställningarna: skriv en egen post. Exempel:

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

**Används inte ännu** (accepteras, sparas, men inget läser dem):
`ui.graph_colorization` (borttagna från Inställningar;
accepteras fortfarande vid inläsning), `ui.date_word_format` (borttagen från
Inställningar; accepteras fortfarande vid inläsning). Den gamla `ui.diff_view_mode`
läses en gång som `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `editor.tab_size` | `4` | Kolumner per tabbstopp, 1 till 8 (redigerare och diffar). |
| `editor.eol` | `"auto"` | Radslut som den inbyggda redigeraren skriver: `auto` (behåll filens), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Språkfärger i redigeraren. |
| `editor.line_numbers` | `true` | Radnummer i redigeraren och diffarna. |
| `editor.follow_selection` | `true` | Medan redigeraren är öppen öppnas en annan fil som markeras i högerpanelen även i redigeraren, efter att du fått frågan om osparade ändringar ska sparas. En fil som inte kan redigeras öppnas i sin vanliga vy. En fil från en commit lämnar redigeraren för sin diff. |

## `[merge]`

Merge-konfliktredigeraren ([Lösa merge-konflikter](merge-conflicts.md)). Inställningar > Diff & redigerare visar den.

| Nyckel | Standard | Betydelse |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | Bredden, i kolumner, på redigerarens yta från vilken A och B visas sida vid sida (långa rader rullar i sidled: Shift+hjul, Shift+piltangenter). Smalare visar den en sida i taget bakom flikar. `40` är lägsta värde, `0` betyder alltid sida vid sida (meddelandet under 40 kolumner finns kvar). Redigeraren döljer panelerna Refs och Status medan den är öppen, så dess yta är oftast hela fönstret. |

## `[git]`, `[credentials]`, `[ssh]`

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `git.pull_mode` | `"merge"` | Vad Pull (knappen, `Ctrl+G p`, `Git: Pull`) gör: `merge` (fast-forward när det går), `ff_only`, `rebase`. Skickas alltid uttryckligen till git. Det gamla `fetch_all` läses som `merge`. |
| `git.push_behavior` | `"direct"` | `direct` pushar till standardremoten; `ask` visar remote-menyn först. |
| `git.push_group_preflight` | `true` | Innan en gren pushas till flera remotes: kontrollera varje mål först (`ls-remote`, plus en fetch av den remoten bara när den skiljer sig), så att ett problem på en senare remote upptäcks före den första pushen. |
| `git.merge_fast_forward` | `"never"` | Vad en merge du startar (grenmenyn, dra och släpp, paletten, `gitpoulpi merge`) gör när grenen kunde fast-forwardas: `never` gör alltid en merge-commit (`--no-ff`), `allowed` gör fast-forward när det går (gits standard, `merge.ff` gäller), `only` gör fast-forward eller avvisar. Ett repositorys eget val (Inställningar > Alternativ för merge, `merge_fast_forward` i dess `repo-state.toml`) går före; CLI:ts `--ff`, `--no-ff` och `--ff-only` går före båda. Direkt. |
| `git.default_encoding` | `"utf-8"` | Kodning för text som **inte** är giltig UTF-8 (filinnehåll, blame, diffar, commit-meddelanden utan `encoding`-huvud): vilken WHATWG-etikett som helst (`windows-1252`, `shift_jis`, ...). Bara visning; staging kodar aldrig om. Ett repository kan åsidosätta den. |
| `git.lfs_enabled` | `true` | Visa och styr Git LFS. Ett repository kan åsidosätta den. Se [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Före en pull eller push av ett repository som använder LFS, installera dess LFS-filter / pre-push-hook (`git lfs install --local`, bara det här repositoryt) när de saknas. Se [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Används inte*: ett repositorys `core.autocrlf` / `core.eol` är gits egna inställningar, som redigeras i Inställningar > Git & kodning (eller `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | HTTPS-autentisering för fetch, pull, push, clone: `credential_helper` (gits), `prompt` (ignorera helpers; git frågar i en terminal, så det misslyckas i bakgrundsdaemonen), `none` (bara publika remotes). Gamla `encrypted_file`-värden läses som `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Typen som *Generera nyckelpar* skapar: `ed25519`, `rsa` (4096 bitar), `ecdsa` (NIST P-256); filen är `gitpoulpi_<type>`. SSH självt använder alltid din ssh-agent och ssh-konfiguration. |
| `ssh.private_key_path`, `ssh.public_key_path` | ej satt | Noteras av *Generera nyckelpar*. När `ssh.private_key_path` är satt kör fetch, pull, push och clone git med `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` gör samma sak (ssh väljer den matchande agentnyckeln) när den privata sökvägen är tom. En `GIT_SSH_COMMAND`/`GIT_SSH` i miljön eller en `core.sshCommand` i gitkonfigurationen (den fastnålade per repository, `gitpoulpi ssh-command set --key`) vinner. Läses före varje överföring, så ändringar gäller nästa. |

## `[[integrations]]` (hosting och ärendehantering)

Skrivs av Inställningar > Integrationer eller `gitpoulpi hosting integrations add`. Varje
slag har en klient (bara personliga token, ingen OAuth).

```toml
[[integrations]]
id = "corp"                         # det --id tar; token lagras under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # API-roten härleds från den
token_ref = "integration:corp"      # en referens, aldrig token
name = "Corp GitLab"                # valfritt visningsnamn
account = "me@example.com"          # valfritt, se nedan
enabled = true                      # false: ignoreras vid matchning mot värd
```

En egen server kan köras över vanlig http: `url = "http://localhost:3000"` behålls
som du skrev den (schemat och porten följer med i varje anrop, och en remote på den värden
matchar profilen). En token som skickas över vanlig http går okrypterad, så använd
bara http i ett lokalt eller betrott nätverk.

`kind` accepterar också `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` och `gerrit`
(Gitea och Forgejo delar en klient; en Forgejo-profil har `https://codeberg.org` som standard). Jira,
Trello, Linear, YouTrack och Redmine är bara ärendehanterare; Gerrit är bara granskning (dess ändringar är
pull requests, det har inga ärenden). Bitbucket Cloud har ingen ärendehanterare.

`account` är den icke-hemliga identiteten bredvid token, tom och utelämnad för de
flesta slag: Atlassian-kontots e-post för `jira_cloud` och `bitbucket` (ett tomt
Bitbucket-konto betyder en arbetsyte- eller repositoryåtkomsttoken, skickad som
Bearer), API-nyckeln för `trello`, användarnamnet för `gerrit` (token är dess HTTP-
lösenord, skickat som HTTP basic). Själva token förblir den enda hemligheten. För
`azure_devops` är `url` `https://dev.azure.com/<organization>`; en
`https://<organization>.visualstudio.com`-URL läses som samma organisation. För
`azure_devops_server` är det samlingsroten, till exempel
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (det anslutna kontot) och `ssh` (profilens nyckelval) skrivs åt dig. Token
kommer från `GITPOULPI_TOKEN_<ID>` (id versaliserat, icke-alfanumeriska tecken som
`_`), annars den lagrade token, annars din credential helper för den värden när bara
en profil använder den (bara GitHub- och GitLab-profiler: de andra slagen läser aldrig
credential helpern). Detaljer: [Värdtjänster och
ärendehantering](hosting-integration.md). Den gamla tabellen `[[hosting]]` *används
inte*.

För `local_markdown` (en mapp med Markdown-anteckningar som ärendekälla) är `url`
mappen, relativ till varje repository eller absolut, och det finns ingen token. Dess
valfria tabell `[integrations.local]` anger `statuses` (tavlans kolumnordning,
standard `todo`, `doing`, `done`), `done_status` (standard `done`),
`reference_prefix` (`LOC` visar `LOC-12`) och `me` (vem ”tilldelade mig” betyder).
Se [Lokala ärenden och Obsidian](local-issues.md).

### `[integrations.radar]` (kontoövergripande listor)

Valfri tabell i en kodvärdsprofil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; inte Bitbucket Cloud). Saknad
betyder att varje nyckel är av. Skrivs av `gitpoulpi hosting integrations radar`; läses av
`gitpoulpi hosting radar` och Radar, och redigeras i Inställningar > Integrationer (Radar-blocket för en ansluten profil).

```toml
[integrations.radar]
review_requests = true    # begäranden som väntar på min granskning, alla repositories
open_requests = false     # alla öppna begäranden i mina namnrymder
include_drafts = false    # behåll utkast till begäranden i de här listorna
assigned_issues = true    # öppna ärenden tilldelade mig, alla repositories
namespaces = ["acme", "ptm/produits"]  # en post per rad i Inställningar; tomt = ingen gräns
namespaces_regex = false  # true: varje post är ett reguljärt uttryck över hela repositorysökvägen
review_mode = "ask"       # ask | clone | api
```

| Nyckel | Standard | Betydelse |
|---|---|---|
| `review_requests` | `false` | Öppna begäranden som ber om din granskning, oavsett om repositoryt är klonat |
| `open_requests` | `false` | Alla öppna begäranden i din egen namnrymd och i varje organisation eller grupp du tillhör (eller i `namespaces`), aldrig "allt token ser" |
| `include_drafts` | `false` | Utkast till begäranden lämnas utanför de här listorna om inte detta är på |
| `assigned_issues` | `false` | Öppna ärenden tilldelade dig, alla repositories |
| `namespaces` | `[]` | Vilka repositories varje lista behåller (granskningsbegäranden, tilldelade ärenden, öppna begäranden); tomt betyder ingen gräns, och `open_requests` söker då i din egen namnrymd plus varje medlemskap (GitHub: högst 12 genomsöks, GitLab: 10 grupper och 10 ägda projekt, Gitea, Forgejo och Gerrit: 10). En post matchar ett repositorys fulla sökväg (`group/sub/project`, Azure DevOps `Project/repo`, Bitbucket Data Center `KEY/repo`) när den är lika med den eller är ett sökvägsprefix som slutar vid ett `/`: `ptm` behåller `ptm/produits/x`, inte `ptmx/y`; skiftläge ignoreras. Inställningar visar en post per rad; en äldre fil kan innehålla en kommaseparerad sträng (`"ptm, formation"`), som fortfarande läses in och sparas tillbaka som en lista |
| `namespaces_regex` | `false` | Läs varje post i `namespaces` som ett reguljärt uttryck som matchas mot hela repositorysökvägen, som du skriver den (så `^ptm/(produits\|infra)/` förankras i början; utan `^` eller `$` matchar det var som helst i sökvägen; lägg till `(?i)` för att ignorera skiftläge). `open_requests` söker då i dina medlemskap och uttrycken filtrerar raderna. Ett ogiltigt uttryck visas under fältet i Inställningar och sparas inte; i en handredigerad fil lämnar det listorna tomma med en notering |
| `review_mode` | `"ask"` | Vad granskning av en begäran vars repository inte är klonat gör: `ask`, `clone` (klona det, granska sedan) eller `api` (granska via värdens API) |

Svaren cachas per profil i `radar-account.json` bredvid
`config.toml` i fem minuter; filen är en cache och kan tas bort utan risk.

### Förtroendearkivet (finns inte i `config.toml`)

Vad varje integration och AI-profil med API-nyckel betroddes för (dess adress,
konto-id, en fäst certifikatsnyckel, om vanlig http tilläts) ligger i
`trust.toml` i den privata datamappen som bara användaren kan läsa (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` i Windows, `~/.local/share/gitpoulpi/private/`
i Linux). Den är med flit inte en del av `config.toml`: den som kan redigera
konfigurationen får inte kunna vidga förtroendet genom att redigera den. Ändra den med
`gitpoulpi hosting integrations trust` och `gitpoulpi ai trust` (se
[säkerhetskontrollen](hosting-integration.md#säkerhetskontrollen)); filen skrivs
om av GitPoulpi, inte för hand.

## `[mcp]` (vad AI-agenter får göra)

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `mcp.allow_mutations` | `false` | Av: MCP-verktyg som ändrar något vägras; läsningar, `ui_*` och förslag fortsätter fungera. `true` låter agenter ändra repositoryt. |
| `mcp.mutation_allowlist` | `[]` | Icke-tom: bara dessa verktygsnamn får ändra saker (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: vägra de `ui_*`-verktyg som flyttar ditt fönster (inte `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (fönstret visar ett chip som du bekräftar), `auto` (tillämpas om du inte är upptagen), `off`. `--ui-control` på serverkommandot åsidosätter det per agent. Gäller också `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Tillåt `ui_propose_*` / `gitpoulpi ui propose`: förifyllda dialoger som du bekräftar själv. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` (och dess alias `gitpoulpi-mcp`) vägrar starta, med ett meddelande som namnger den här nyckeln. Inställningar > Agenter & MCP > *MCP-server*. Läses när en agent startar servern. |
| `mcp.bind` | `"127.0.0.1:8808"` | Adressen `--http` lyssnar på när du inte anger någon (`--http ADDR` / `--bind` vinner). Bara loopback om inte `--allow-remote`. Tomt eller det gamla `"stdio"` betyder standardadressen. Inställningar > Agenter & MCP > *MCP-serverns port* ändrar porten och behåller värden. |
| `mcp.allow_history_purge` | `false` | Bara i `config.toml` (ingen rad i Inställningar). `true` listar verktyget `rewrite_purge` för agenter (det kräver fortfarande `allow_mutations`). Av ser en agent inte verktyget alls: en rensning raderar den gamla historiken från en historikomskrivning, ångra-posten för den och dess reflogs för gott. CLI:t och TUI:t styrs inte av den. |
| `mcp.http_auth` | `true` | `--http` svarar bara på en begäran med `Authorization: Bearer <token>` (token från `gitpoulpi mcp token`). `false` tar bort kontrollen; servern varnar då på stderr, och `--allow-remote` nekas. Inställningar > Agenter & MCP > *HTTP-autentisering*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (ett certifikat som skapas en gång under den privata datamappen, med sitt SHA-256-fingeravtryck utskrivet vid start) eller `files` (`tls_cert` och `tls_key`). Med TLS på avslutar servern TLS själv och ingen vanlig lyssnare finns. Inställningar > Agenter & MCP > *TLS*. |
| `mcp.tls_cert` | `""` | PEM-certifikatkedja som används med `tls = "files"`. |
| `mcp.tls_key` | `""` | Privat PEM-nyckel som används med `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Du uppger att en tunnel eller omvänd proxy framför endpointen står för krypteringen. Det kan inte kontrolleras inifrån servern, så `--allow-remote` utan TLS kräver att det är satt. Sträckan mellan proxyn och servern är klartext i det nätverket. Inställningar > Agenter & MCP > *Bakom en tunnel eller omvänd proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Bara i `config.toml`. Av: MCP:s stage-verktyg erbjuder inte `allow_sensitive`, och en stage som skulle lägga till en flaggad fil (se `[privacy]`) nekas med ett meddelande som säger åt agenten att fråga dig. På: en agent får skicka `allow_sensitive`. CLI:t och TUI:t styrs inte av den: de frågar dig. |

`allow_mutations`, `mutation_allowlist` och `allow_ui` styr bara MCP-servern; CLI:t är
ditt eget skal och styrs inte av dem. Se [MCP och
agenter](mcp-and-agents.md#behörigheter).

## `[ai]` (inbyggd AI, av som standard)

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `ai.enabled` | `false` | Strömbrytaren. Av, visas ingen AI någonstans och ingenting skickas. |
| `ai.default_profile` | `"default"` | Id för profilen som åtgärder använder när de inte namnger någon. |
| `ai.global_instructions` | `""` | Text som läggs till i varje åtgärds prompt. Får använda `${repo}`, `${uilang}`, ... (se [AI > Promptvariabler](ai.md#promptvariabler)); detsamma gäller instruktionerna per åtgärd och per repository. |
| `ai.actions.<action>.profile` | `""` | Profil för en åtgärd; tomt = standardprofilen. Åtgärder: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`. En arbetsyta (`workspaces.toml`, `[workspace.ai_instructions]` och `[workspace.ai_overrides.<action>]`) och ett repository kan lägga till i eller ersätta de här instruktionerna: se [AI](ai.md#instruktioner). |
| `ai.actions.<action>.instructions` | `""` | Text som läggs till bara för den åtgärden. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` eller `no`: om den lokala agenten för ett kodgranskningsutkast arbetar inuti repositoryt för den körningen, vilket åsidosätter AI-profilens `workdir`. Ett repository kan ange sitt eget (`gitpoulpi ai review-context --repo`). Se [AI > Granskningskontext](ai.md#granskningskontext). |
| `ai.review.include_claude_md` | `true` | Ge ett kodgranskningsutkast repositoryts `CLAUDE.md` (roten, läst på den gren begäran riktar sig mot; kapas vid 8 KB). |
| `ai.review.include_agents_md` | `true` | Detsamma för `AGENTS.md`. |
| `ai.review.extra_file` | `""` | En fil till för ett kodgranskningsutkast: en sökväg relativt repositoryts rot, en absolut sökväg eller en länk till en fil hos en kodvärd (läst via den matchande integrationsprofilen, annars anonymt). Alla filerna tillsammans kapas vid 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Ersätt varje hemlighet som skannern känner igen i en begäran (en token, ett block med en privat nyckel, ett lösenord i en URL, ett `password = ...`-värde som ser slumpmässigt ut) med `[REDACTED:<rule>]` innan den lämnar datorn. Inställningar > AI visar en röd varning medan den är av. |
| `ai.privacy.exclude_sensitive_files` | `true` | Utelämna diffen för varje fil som matchar `privacy.sensitive_patterns` från det som skickas. En lokal agent som arbetar i repositoryts mapp kan fortfarande läsa filer själv: se [AI > Integritet](ai.md#integritet). |
| `ai.limits.conflict_bytes` | `40000` | Flest byte som en merge-konflikt (dess base-, current- och incoming-block tillsammans) får vara när "Föreslå en lösning" skickar den. Över gränsen misslyckas inte GitPoulpi: det varnar och erbjuder *Skicka ändå* (bara den här begäran), *Öppna inställningen* och *Avbryt*; en konflikt klipps aldrig av. Gränsen är GitPoulpis eget skydd, inte modellens: höj den när din modell har utrymme. Hålls mellan 1000 och 50000000; kan ställas in i Inställningar > AI > Gränser. De andra gränserna nedan fungerar likadant och finns alla i den gruppen. |
| `ai.limits.diff_bytes` | `60000` | Flest byte av den diff som ett meddelande till en commit, en förklaring, ett stash-namn, en beskrivning av en pull request, en commit-plan eller en commit-uppdelning får bära, och av hela diffen i en kodgranskning. Över gränsen klipps ingenting av i det tysta: i TUI erbjuder *Över AI-gränsen* *Skicka allt* (bara den här begäran, ingenting klipps), *Skicka avkortat* (slutet utelämnas, som förut), *Öppna inställningen* och *Avbryt*; CLI och MCP trunkerar och rapporterar det i `truncated` (`--send-all` skickar allt). Hålls mellan 1000 och 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Flest byte av en fils diff i en kodgranskning. Över gränsen: samma fråga och samma två svar; *Skicka avkortat* förkortar den filen med en synlig notering. Samma intervall. |
| `ai.limits.review_text_bytes` | `4000` | Flest byte av beskrivningen av den granskade pull requesten. Över gränsen klipper *Skicka avkortat* beskrivningen med en notering `[text cut]`. Samma intervall. |
| `ai.limits.review_threads_bytes` | `6000` | Flest byte av listan med granskningskommentarer som redan finns (ges så att modellen inte upprepar dem). Över gränsen klipper *Skicka avkortat* listan. Samma intervall. |
| `ai.limits.review_context_file_bytes` | `8192` | Flest byte av en kontextfil för projektet (`CLAUDE.md`, `AGENTS.md`, den extra filen) som ges till en kodgranskning. Över gränsen förkortar *Skicka avkortat* filen med en notering. Samma intervall. |
| `ai.limits.review_context_bytes` | `16384` | Flest byte av alla kontextfiler tillsammans. Över gränsen förkortar *Skicka avkortat* filen som korsar gränsen och utelämnar de som kommer efter. Samma intervall. |
| `ai.limits.hosting_reply_context_bytes` | `49152` | Flest byte av allt som ett utkast till svar (eller ärendebeskrivning) får utöver din egen text: objektet och dess konversation. Över gränsen utelämnas de äldsta kommentarerna först och avkortningen anges i `truncated`. Samma intervall som de andra bytegränserna. |
| `ai.limits.hosting_reply_file_bytes` | `8192` | Flest byte av en repositoryfil som ges till ett utkast till svar. Används när AI:n läser repositoryt. Samma intervall. |
| `ai.limits.hosting_reply_map_bytes` | `6144` | Flest byte av repositorykartan som ges till ett utkast till svar. Används när AI:n läser repositoryt. Samma intervall. |
| `ai.limits.hosting_reply_hits_per_term` | `20` | Flest sökträffar som behålls per sökterm i ett utkast till svar. Används när AI:n läser repositoryt. Hålls mellan 1 och 200. |
| `ai.limits.hosting_reply_terms` | `8` | Flest sökterm som ett utkast till svar slår upp. Används när AI:n läser repositoryt. Hålls mellan 1 och 50. |
| `ai.hosting_reply.base_prompt` | `""` | Grundprompten för de två skrivåtgärderna (`hosting_reply`, `issue_body`). Tomt använder den inbyggda; text här ersätter den, och instruktionsnivåerna följer fortfarande efter. Redigeras i Inställningar > AI > Svar på en värd. |
| `ai.hosting_reply.read_repository` | `false` | Om ”Låt AI:n läsa repositoryt” börjar ikryssat. Av som standard: repositoryt läses bara när du kryssar i det för den användningen. En lokal agent eller en API-profil läser det sedan som beskrivs i [AI-kapitlet](ai.md#åtkomst-till-repositoryt). |
| `ai.hosting_reply.reuse_agent_session` | `true` | Behåll en lokal agents session mellan två svar när dess harness stöder det. Gäller när AI:n får läsa repositoryt. |
| `ai.hosting_reply.context_ttl_minutes` | `30` | Hur länge en repositorykontext som byggts för ett svar återanvänds. Hålls mellan 1 och 1440. Gäller när AI:n får läsa repositoryt. |
| `ai.sent_preview.<action>` | `true` | En nyckel per åtgärd (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`, `hosting_reply`, `issue_body`): visa ”Det som skickas” innan begäran går iväg. Dialogens ”Visa inte igen för ...” skriver `false`; Inställningar > AI > *Visa vad som skickas före...* slår på dem igen. `worktree_name` godtas men har ingen verkan: den skickar bara det du skrev i dialogrutan Skapa worktree och de lokala grennamnen. |

Ett repositorys egen tabell `[preferences.ai_review]` (i dess `repo-state.toml`, eller posten för ett repository som inte är
klonat i `remote-reviews.toml`) tar samma nycklar, alla valfria, plus `no_extra_file = true`: ingen extra fil för
det repositoryt, oavsett vad `ai.review.extra_file` säger. En osatt nyckel, ett osatt `extra_file` och ett saknat
`no_extra_file` ärver det globala värdet; att sätta `extra_file` till en fil rensar omkopplaren och tvärtom.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # tomt = slagets standard
base_url = ""           # tomt = slagets standard (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # var API-nyckeln lagras; aldrig nyckeln
command = ""            # bara local_agent: det exakta programmet ("claude", ett alias, ett omslag, en sökväg)
harness = ""            # bara local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; tomt = härleds från `command`
args = []               # bara local_agent, t.ex. ["-p", "--model", "haiku"]
run_mode = "headless"   # bara local_agent: headless (standard) | terminal (agentens interaktiva skärm i en virtuell terminal)
terminal_args = []      # bara local_agent: argumenten för terminalläget (args behåller headless-uppsättningen)
workdir = "default"     # bara local_agent: default | dedicated | repository (default = dedikerad mapp i terminalläge, GitPoulpis mapp i headless-läge)
timeout_secs = 0        # bara local_agent: sekunder innan agenten stoppas; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # bara local_agent, valfritt: variabler som sätts bara för den processen
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = en av GitPoulpis egna variabler
```

`harness` väljer de förslag Inställningar > AI erbjuder (headless-argument,
alternativ, modellflagga och -lista, miljövariabler) oberoende av `command`; båda
nycklarna utelämnas från filen när de är tomma. Miljövärden loggas aldrig och döljs i
Inställningar när de ser ut som hemligheter.

Inställningar > AI erbjuder förinställningar för `harness`/`command`/`args`/`env` (de
vanliga agent-CLI:erna, med sina headless-flaggor, modellflagga och användbara
miljövariabler) och för `kind`/`base_url`/`model` (de hostade och lokala
HTTP-leverantörerna); se [AI > Konfigurera](ai.md#konfigurera-ai).

En profil med namnet *Default* skapas när det inte finns någon. Gamla
`[[ai.presets]]`- / `[ai.defaults]`-filer migreras vid inläsning och behålls i
`[[ai.legacy_presets]]`. Allt annat: [AI-hjälp](ai.md).

## `[repos]`, `[session]`

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `repos.clone_root` | `""` | Var Clone-dialogen börjar; tomt = `%USERPROFILE%\source\repos` eller `~/src`. |
| `repos.clone_start_source` | `"last"` | Källan som Klona-dialogen öppnas på: `last` (standard, eller tomt) = den som visades när dialogen stängdes sist, `url` = alltid Klona med URL, eller ett integrationsprofil-id (`id` i dess `[[integrations]]`-post). En profil som är borta, inaktiverad eller inte ansluten ersätts av Klona med URL. Den senaste källan i sig är tillstånd som sparas i `clone-source.toml` i konfigurationsmappen, inte här. |
| `repos.browse_root` | `""` | Var Bläddra öppnas; tomt = senast valda mapp, sedan hemmappen. |
| `repos.show_hidden` | `false` | Mappväljaren visar dolda mappar (`Alt+H` växlar och sparar det). |
| `repos.show_siblings` | `false` | Mappväljarens vänstra träd listar också de andra mapparna på varje öppen nivå, som en riktig hierarki (`Alt+S` växlar och sparar det). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Var GitPoulpi lagrar det den kommer ihåg om ett repository (reffilter, hopfällningsläge, den personliga arbetsflödesåsidosättningen, hostingcachen): `in_git_dir` = `<git-katalog>/gitpoulpi` (delas av alla worktrees, syns aldrig i `git status`), `in_worktree` = `<repo>/.gitpoulpi` (den enda platsen där ett team kan checka in `workflow.toml`), `user_data` = `<operativsystemets datamapp>/repos/<namn>-<id>` (`%APPDATA%\GitPoulpi\data` i Windows, `$XDG_DATA_HOME/gitpoulpi` i Linux, `~/Library/Application Support/GitPoulpi` i macOS). Det gamla värdet `".gitpoulpi"` betyder fortfarande `in_worktree`. Att ändra det flyttar inget tillbaka: data som hittas i en äldre `<repo>/.gitpoulpi` flyttas till den valda platsen första gången de behövs. |

## `[log]` (programloggen)

Det som fliken Program i aktivitetsloggen visar och som ”Kopiera diagnostik” citerar. Detaljer: [När något misslyckas](troubleshooting.md#när-något-misslyckas).

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `log.level` | `"info"` | Den mest utförliga raden som spelas in: `error`, `warn`, `info`, `debug` eller `trace`. Nivårutan i aktivitetsloggen ändrar den (och sparar den här). |
| `log.retention_days` | `7` | Dagar som loggfiler sparas, dagens inräknad: 1 till 3650, tio år (ett större eller mindre värde justeras). Alltid ett antal dagar, oavsett vilken enhet Inställningar visar det i. Äldre filer tas bort när GitPoulpi startar. En dags fil begränsas också av `log.max_file_mb`, så i värsta fall blir det ungefär `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | Enheten som Inställningar visar `log.retention_days` i: `days`, `weeks`, `months`, `years` eller `auto`. Bara visning, den ändrar aldrig hur länge filer sparas. En vecka är 7 dagar, en månad är 30 dagar (inte en kalendermånad, så samma antal dagar oavsett när den tillämpas) och ett år är 365 dagar. `auto`, och en konfiguration som skrevs innan nyckeln fanns, väljer den största enhet som delar dagarna jämnt (14 dagar visas som ”2 veckor”, 45 dagar som ”45 dagar”). En enhet som inte delar dagarna (`retention_days = 10` med `weeks`) ignoreras och dagarna visas. Inställningar skriver båda nycklarna; en äldre GitPoulpi ignorerar enheten och läser fortfarande dagarna. |
| `log.max_file_mb` | `5` | En dags fil roteras till en `.old`-säkerhetskopia vid den här storleken, så en dag tar aldrig mer än det dubbla. |
| `log.max_file_unit` | `"auto"` | Enheten som Inställningar visar storleken i: `mib`, `gib` (1024 MiB) eller `auto`. Bara visning: storleken är alltid `max_file_mb` MiB, 1 till 1024. |
| `log.timings` | `true` | Fliken Git-åtgärder i aktivitetsloggen visar hur lång tid varje åtgärd tog (`Push - 123 ms`: öppna ett repository, uppdatera, fetch, pull, push, commit, checkout, merge, rebase, stash, klona). Tiderna mäts av fönstret, från begäran till daemonens svar, och inkluderar alltså IPC-vägen fram och tillbaka. Av döljer tiderna och posterna för lyckade åtgärder. Misslyckanden listas alltid. |
| `log.extended` | `false` | ”Utökad loggning”: fliken Git-åtgärder listar också bakgrundsåtgärderna (den periodiska uppdateringen som läser av status, auto-fetch) med deras tider. De skrivs alltid till programloggen på nivån `debug`. |
| `log.use_mailmap` | `true` | Visa varje författare med de namn som repositoryts `.mailmap` (och `mailmap.file`) anger, i Log, blame och filhistorik: daemonen frågar git efter `%aN`/`%aE` (systembackenden) eller tillämpar samma filer själv (libgit2-backenden), så svaret blir detsamma. De lagrade commits ändras inte och ingenting skrivs om. Att söka i Log efter författare matchar namnen som de visas. Gäller direkt; Log läser om inom sin 2 s-avfrågning. |
| `log.commit_count` | `"system-git"` | Hur statusradens `inlästa / totalt` får fram totalen. `"system-git"` kör `git rev-list --count` (en process, snabbast på en vanlig dator: git läser commit-graph-filen). `"libgit2"` räknar i processen med en libgit2-revwalk och startar ingenting: välj det när varje kommandoradsprocess är långsam att starta, vanligen för att ett antivirusprogram med realtidsskanning granskar var och en; med 50 000 commits tar det ungefär 200 ms i bakgrunden i stället för 12 ms, men ingen process startas. `"off"` räknar aldrig: statusraden visar bara de inlästa commits. Gäller direkt. Ett räkneanrop filtrerat på en sökväg använder alltid systemets git. |

Filerna heter `gitpoulpi-YYYY-MM-DD.log` och ligger i operativsystemets tillståndsmapp (`~/.local/state/gitpoulpi/logs` på Linux; `<cache>\logs` på Windows och macOS; `<config>/cache/logs` med `GITPOULPI_CONFIG_DIR`). Token, lösenord och `Authorization`-huvuden maskeras innan något skrivs.

## `[search]` (arbetsytesökningen)

Vad `gitpoulpi search` (och dess MCP-verktyg) söker igenom. Detaljer: [Arbetsytor och sökning i CLI:n](cli-reference.md#arbetsytor-och-sökning).

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (nyast först, över varje gren, remote-gren och tagg) vars meddelanden sökindexet håller i minnet per repository, ungefär 100 byte plus meddelandet var. En längre historik kapas där och svaret anger `partial`. `0` stänger av commitsökning (grenar, taggar, stashar och worktrees söks fortfarande). Läses live: nästa sökning följer en ändring. |

## `[rewrite]` (historikomskrivning)

Det som `gitpoulpi rewrite` och omskrivningsvyn skyddar. Detaljer: [Skriva om historiken](history-rewrite.md).

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Grenar (namn, eller mönster där `*` matchar valfri text) som en omskrivning lämnar ifred när de redan är pushade, om inte körningen listar dem (`--include-protected BRANCH`; `--include-all-protected` för pushen; rutan per gren i omskrivningsvyn). Läggs till de baser som ett repositorys `workflow.toml` markerar som `protected`. Pushen av en omskrivning hoppar över dem på samma sätt. Det här är GitPoulpis eget skydd: värdens grenskydd avgör fortfarande om force pushen godtas. Läses direkt: nästa förhandsgranskning följer en ändring. |
| `rewrite.external_scripts` | `["shell"]` | Vilka externa skriptspråk skriptregeln får använda: valfria av `"shell"` (bash eller sh; pwsh eller powershell för PowerShell-skript), `"python"`, `"lua"` (ett installerat Lua; det inbyggda Lua är alltid på och behöver ingen post). **Ett externt skript körs utan sandlåda: det körs med dina rättigheter.** Ett språk som inte finns i listan är nedtonat i omskrivningsvyn och vägras (`scripts_disabled`) av CLI, MCP och daemonen. Även när det listas körs ett externt skript som kommer från repositoryt eller ett delat recept först efter att du litat på det (`gitpoulpi rewrite script trust FILE`, eller frågan i omskrivningsvyn); förtroendet knyts till skriptets innehållshash, så ett ändrat skript frågar igen. Slå på Python genom att redigera `config.toml` (en lista kan inte sättas med `config set`): `[rewrite]` och sedan `external_scripts = ["shell", "python"]`. Läses direkt. |
| `rewrite.script_timeout_ms` | `5000` | Hur lång tid ett externt skript får på sig att svara för en commit. Vid en timeout dödas hela processträdet och omskrivningen misslyckas (`script_failed`, med commiten angiven); ingenting har flyttats. `0` läses som standardvärdet. Läses direkt. |
| `rewrite.script_timeout_unit` | `"auto"` | Enheten som Inställningar visar tidsgränsen i: `milliseconds`, `seconds` eller `auto`. Bara visning: gränsen är alltid `script_timeout_ms` millisekunder. Inställningar accepterar 1 till 600000 (tio minuter); `0` läses fortfarande som standardvärdet. Läses direkt. |
| `rewrite.script_total_timeout_s` | `3600` | Hur lång tid ett pass av ett externt skript får ta totalt. `0` läses som standardvärdet. Läses direkt. |
| `rewrite.script_total_timeout_unit` | `"auto"` | Enheten som Inställningar visar tidsgränsen i: `seconds`, `minutes`, `hours` eller `auto`. Bara visning: gränsen är alltid `script_total_timeout_s` sekunder. Inställningar accepterar 1 till 86400 (ett dygn); `0` läses fortfarande som standardvärdet. Läses direkt. |

## `[updates]` (söka efter en nyare release)

Vad versionschipet i titelraden gör med nyare releaser. Detaljer: [Uppdateringar](updates.md).

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` frågar GitHub en gång per dygn, i bakgrunden, om det finns en nyare release och gör versionschipet till `vX.Y.Z tillgänglig`. `"auto"` laddar också ned och förbereder uppdateringen själv när det här bygget kom från ett releasearkiv (det frågar fortfarande innan det startar om); för alla andra bygge beter det sig som `"notify"`. `"off"` gör aldrig någon förfrågan. Gäller direkt. |
| `updates.channel` | `"stable"` | `"stable"` beaktar bara fullständiga releaser; `"prerelease"` erbjuder också release candidates (`-rc.1`). Gäller direkt. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | `ägare/namn` på github.com vars releaser kontrolleras. Så länge det repositoryt är privat svarar GitHub 404 om inte en token är lagrad för github.com (Inställningar > Integrationer); utan en misslyckas kontrollen tyst och skriver en rad i programloggen. |

En administratör kan ange `mode`, `channel` (som `release_channel`) och `repo`, plus några fler uppdateringsalternativ, i en policyfil för hela datorn eller i Windows-registret. En nyckel som policyn sätter vinner över `config.toml` och visas i Inställningar som **Inställt av din administratör**. `mode` i policyn är ett tak: du kan välja ett lägre värde, aldrig ett högre. Se [Företagsdistribution](enterprise-distribution.md#policyfilen-och-registret).

## `[privacy]` (känsliga filer)

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Sökvägsmönster i gitignore-stil. Ett mönster utan `/` matchar filnamnet på vilket djup som helst, ett med `/` hela sökvägen (ett inledande `/` förankrar det i roten); `*` och `?` stannar inom en mapp, `**` korsar mappar; ett `!` tillåter på nytt det ett tidigare mönster matchade och sista träffen gäller; skiftläge ignoreras. Används av stagingskyddet och av `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Att stagea en fil som matchar ett mönster, eller vars innehåll har en igenkänd hemlighet, frågar först (TUI), nekas om inte `--allow-sensitive` anges (CLI) eller om inte `mcp.allow_sensitive_staging` låter agenten skicka `allow_sensitive` (MCP). Textfiler upp till 1 MiB var och 32 MiB totalt läses; binärfiler matchas bara på namn. Se [Stage och commit](staging-and-committing.md#känsliga-filer). |

## `[conflict_prevention]` (listor över basgrenar)

Vilka grenar proaktiv konfliktdetektering får ta som bas för den aktuella grenen är en lista med mönster (se [Lösa merge-konflikter](merge-conflicts.md#konfliktförebyggande)). Listor kan ha namn och delas; Inställningar > Konfliktförebyggande redigerar dem.

| Nyckel | Standard | Värden och effekt |
|---|---|---|
| `conflict_prevention.default_list` | `""` | Namnet på listan som ett repository använder när det saknar en egen. Tomt, eller ett namn som ingen lista har, betyder den inbyggda `**` (varje gren). Läses in igen vid varje kontroll, så en ändring gäller öppna repositories utan omstart. |
| `conflict_prevention.lists` | ingen | De namngivna listorna, en tabell var: `[conflict_prevention.lists.release]` med `patterns = ["origin/release/**", "!origin/release/old"]`. Tom `patterns` tillåter varje gren. Att redigera en lista gäller varje repository som följer den. |

Vilken lista ett repository följer finns i dess `repo-state.toml` (nedan). Företräde: repositoryts egna mönster, sedan den namngivna lista det följer, sedan `default_list`, sedan `**`.

## `[keybindings]`

Binder om TUI:ts genvägar; Inställningar > Tangentbord & mus redigerar den.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # standard Ctrl+G
nav_prefix = "Alt+n"             # standard Ctrl+N
settings   = ["Ctrl+,", "F2"]    # en lista = alternativ
tab_menu   = []                  # obunden

[keybindings.git_prefix]         # tangenten som trycks efter git-prefixet
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; ett värde ersätter den åtgärdens
standardvärden, och att ta bort raden återställer dem. Tangentkombinationer:
`Ctrl+Shift+N`, `Alt+n`, `F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). En dålig post
blockerar aldrig något: TUI:t ignorerar den, behåller standardvärdet och säger det på
statusraden. Områden, åtgärds-id:n, standardvärden och konfliktreglerna:
[Kortkommandon](keybindings.md).

## Inställningar per repository: `.gitpoulpi/repo-state.toml`

Förvaras inuti repositoryt; mappen ignorerar sig själv (en `.gitignore` med `*`), så
den visas aldrig som en ändring. Skrivs av TUI:t och CLI:t; du redigerar den sällan.

| Nyckel | Vad det är | Sätts med |
|---|---|---|
| `[preferences] encoding` | Åsidosätter `git.default_encoding` här | Inställningar > Git & kodning |
| `[preferences] lfs_enabled` | Åsidosätter `git.lfs_enabled` här | Inställningar > Git & kodning |
| `[preferences] profile` | `[[profiles]]`-id:t som signerar commits här; saknas det gäller `general.active_profile` | verktygsfältets profilkombolista, **Repository: Byt profil...** |
| `[preferences] autocrlf`, `eol` | *Används inte ännu* | Inställningar > Git & kodning |
| `[preferences] push_after_commit` | Pusha grenen till dess upstream efter varje commit (av när den saknas) | Inställningar > Repo-specifika inställningar > Commit |
| `[preferences] skip_hooks_by_default` | Commit-formulärets Hoppa över hooks är ikryssat från början (av när den saknas) | Inställningar > Repo-specifika inställningar > Commit |
| `[preferences] squash_merge` | En merge av en lokal gren gör squash: stagear, ingen merge-commit (av när den saknas) | Inställningar > Repo-specifika inställningar > Alternativ för merge |
| `[preferences] template_apply`, `template_strip_comments` | Fyll i ett tomt commit-formulär med mallen; ta bort `#`-kommentarsrader när commiten görs (båda av när de saknas) | Inställningar > Repo-specifika inställningar > Meddelandemall |
| `[preferences] conflict_detection` | Proaktiv konfliktdetektering för det här repositoryt; saknas den gäller `general.conflict_detection` | Inställningar > Repo-specifika inställningar > Konfliktförebyggande |
| `[preferences] conflict_base_branches` | Repositoryts egna mönster för Basgrenar, ett per rad (används fortfarande som **Egen lista**) | Inställningar > Repo-specifika inställningar > Konfliktförebyggande |
| `[preferences] conflict_base_source` | Var Basgrenar kommer från: `default` (den globala standardlistan), `list` (`conflict_base_list`) eller `own` (`conflict_base_branches`). Saknas den: egna mönster när det finns några, annars standarden | Inställningar > Repo-specifika inställningar > Konfliktförebyggande |
| `[preferences] conflict_base_list` | Den `[conflict_prevention.lists.<namn>]` som det här repositoryt följer när källan är `list`; en borttagen lista faller tillbaka på standarden | Inställningar > Repo-specifika inställningar > Konfliktförebyggande |
| `commit-template.txt` (en fil bredvid `repo-state.toml`) | Commit-mallen; gits `commit.template` pekar på den | Inställningar > Repo-specifika inställningar > Meddelandemall |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: det här repositoryts AI-instruktioner | Inställningar > Repo-specifika inställningar > AI |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` för en åtgärd här | Inställningar > Repo-specifika inställningar > AI |
| `default_remote` | Remoten som enklicks-Push/Pull/Fetch använder | remote-menyn, `gitpoulpi remote default <name>` |
| `hosting_integration` | `[[integrations]]`-profilen som är fastnålad här | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Leverantörsprojektet som PR- / ärendelistorna läser, som `{ integration, project, filter }`. `filter` är valfritt: en leverantörsspecifik extra klausul (ett Jira JQL-fragment, ett Azure WIQL-villkor, Trello-listnamn) som läggs med AND på listfrågan; GitHub och GitLab ignorerar den. | sidofältets sektioner, `gitpoulpi hosting link` |
| `[preferences] merge_fast_forward` | Åsidosätter `git.merge_fast_forward` här: `never`, `allowed` eller `only` | Inställningar > Repo-specifika inställningar > Alternativ för merge |
| `[preferences] default_push_targets` | De remotes en ny gren också pushar till, kopierade in i grenen när den först får en upstream | Inställningar > Repo-specifika inställningar > Git & kodning; se [Flera remotes](multiple-remotes.md) |
| `[preferences.ai_review]`, `[preferences.ai_hosting_reply]` | Det här repositoryts egna värden för nycklarna i `[ai.review]` och `[ai.hosting_reply]`; saknade nycklar ärver de globala | Inställningar > Repo-specifika inställningar > AI |
| `hosting_post_as`, `hosting_identity` | Namnet dina meddelanden till värden publiceras under i det här repositoryt (åsidosätter `general.hosting_post_as`), och den inloggning GitPoulpi lär sig från det första meddelandet som publiceras med en bottoken när ingen är satt | Inställningar > Repo-specifika inställningar > SSH & inloggning; se [Konversationer](conversations.md#publicera-som) |
| `assignment_blocked` | Inget kan tilldelas från GitPoulpi i det här repositoryts ärenden och pull requests | `gitpoulpi hosting assignment --block \| --allow` |
| `board_labels` | Etiketterna som är kolumnerna på ärendetavlan hos en värd utan arbetsflöde, i tavlans ordning | tavlans knapp **Kolumner** |
| `conversation_seen`, `conversation_events`, `posted_comment_ids` | Vad du senast läste i varje konversation (resten är olästa), om systemhändelser visas, och kommentarerna den här installationen publicerat (de en bot får redigera eller radera) | skrivs av konversationsvyn |
| `reviews` | Granskningens lokala tillstånd per begäran: väntande kommentarer, visade filer | skrivs av granskningsvyn |
| `ref_visibility` | Dolda / soloade refs | Refs-sidofältet, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotes som dolts i grafen som helhet (varje gren, även de som hämtas senare) | remotens ⊙ och meny, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Hopfällda sidofältssektioner samt gren- och taggmappar. `[folds]` innehåller också `status_folders` (hopfällda mappar i Unstaged- / Staged-filträden) och `commit_folders` (hopfällda mappar i en commits träd över ändrade filer, detsamma för varje commit), var och en begränsad till 500 sökvägar, äldsta tas bort först | sidofältet, filträden |
| `pending_auto_stash` | En stash GitPoulpi gjorde och inte har satt tillbaka | skrivs av daemonen; se [Auto-stash](log-and-history.md#auto-stash) |

## Andra miljövariabler

| Variabel | Effekt |
|---|---|
| `GITPOULPI_CONFIG_DIR` | Konfigurationsmappen (se ovan). |
| `GITPOULPI_TOKEN_<ID>` | En hosting-token för profilen `<ID>`, före den lagrade (CI, engångsbruk). |
| `GITPOULPI_GIT` | Den `git`-körbara fil som ska köras (standard: `git` i `PATH`; på Windows den riktiga `mingw64\bin\git.exe` bakom Git for Windows `cmd\git.exe`-shim, vilket sparar en process per anrop). |
| `GITPOULPI_LANG=fr` | Gränssnittsspråket, före `ui.language` (vilket `ui.language`-värde som helst; `qps` är en pseudo-locale för översättare). Ett okänt eller tomt värde ignoreras. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Inga bilder alls, oavsett vad konfigurationen säger. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Tips om bildupplösning (cellstorlek i pixlar; översampling, standard 2, `1` sparar CPU). |
| `GITPOULPI_NO_BROWSER=1` | Starta aldrig en webbläsare (URL:en visas ändå). |
| `GITPOULPI_TIMING=1` | Skriv förfrågningstider till `gitpoulpi-timing.log` i temp-mappen (prestandarapporter). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Vänta inte på Enter efter att TUI:t avslutats med ett fel. |
