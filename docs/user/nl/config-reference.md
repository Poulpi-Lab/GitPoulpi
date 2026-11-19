[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · **Nederlands** · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Configuratienaslag

Elke sleutel van `config.toml`, zijn standaardwaarde en wat hij doet. Je hoeft het
bestand zelden met de hand te bewerken: het scherm [Instellingen](settings.md) (`Ctrl+,`)
schrijft het terwijl je dingen wijzigt, en `gitpoulpi config set` doet dat ook.

## Waar het staat en hoe een waarde wordt gekozen

| | |
|---|---|
| Bestand | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` toont het. |
| Andere map | `GITPOULPI_CONFIG_DIR=<dir>` verplaatst de hele configuratiemap (het bestand, `sessions/`, de poortbestanden van de daemon): een draagbare installatie, een test, of een agent die van je eigen instellingen wordt gescheiden. |
| Voorrang | ingebouwde standaard, dan `config.toml`, dan de `GITPOULPI_*`-variabele, dan een opdrachtregelvlag. De hoogste wint. Een verkeerde waarde (een verkeerd gespelde keuze in het bestand, een ongeldige variabele) is een fout en wordt nooit stilzwijgend genegeerd. |
| Variabelen | Zet de sleutel in hoofdletters en maak van de punten underscores: `general.auto_prune` is `GITPOULPI_GENERAL_AUTO_PRUNE`. Alleen voor sleutels die GitPoulpi echt leest (niet die hieronder als *niet gebruikt* zijn gemarkeerd, en ook geen lijsten zoals `ui.log_columns_hidden`). |
| Vlaggen | `--backend git2\|system` overschrijft `general.default_backend` voor een daemon die die opdracht start. |
| Tokens en sleutels | Nooit in dit bestand. Hosting-tokens en AI-API-sleutels gaan naar de credential helper van je systeem (Git Credential Manager, libsecret / `pass`, Keychain); het bestand bevat alleen een verwijzing (`token_ref`). Zonder helper mislukt het opslaan van een token met een melding en geldt het token alleen voor de sessie. |

```
gitpoulpi config path                          # het bestand, en of het al bestaat
gitpoulpi config get general.auto_stash        # de waarde, de laag waaruit ze komt (default/file/env/flag) en of iets haar leest
gitpoulpi --json config get ui                 # een hele sectie
gitpoulpi config set general.auto_stash always # gecontroleerd tegen het schema, dan geschreven
```

`config set` houdt het bestand schaars (alleen sleutels die iemand heeft ingesteld) en
behoudt sleutels waar niet naar gevraagd is, maar laat opmerkingen vallen. Het weigert
lijsten en hele tabellen, behalve `keybindings.*` (meerdere toetscombinaties gescheiden
door spaties; leeg maakt de binding los). Bewerk `[[profiles]]`, `[[integrations]]` en
`[[ai.profiles]]` in Instellingen of met de hand.

In de tabellen betekent **Herstart** dat de waarde bij het opstarten wordt gelezen; al het
andere geldt meteen (of bij de volgende vernieuwing, fetch of het sluiten van Instellingen).

## `[general]`

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, met terugval op het programma `git` voor staging, merge, rebase, stash en netwerk) of `system` (alleen het geïnstalleerde `git`). Een draaiende daemon houdt zijn backend: **herstart** hem (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Als lokale wijzigingen een checkout, pull, merge of rebase blokkeren: `ask` (een dialoog in de TUI; de CLI en MCP mislukken en noemen `--autostash`), `always` (stash, uitvoeren, pop) of `never` (weigeren). Zie [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | De auto-stash neemt ook untracked bestanden mee. |
| `general.initial_commits_in_graph` | `5000` | Commits die de Log eerst laadt (minstens 500). Hoger kost geheugen en tekentijd bij enorme repositories. |
| `general.lazy_load_commits_in_graph` | `true` | Laad oudere commits als je bijna aan het einde scrolt. Uit: de Log stopt bij het begintal. |
| `general.commits_per_lazy_page` | `1000` | Commits per achtergrondpagina (minstens 50). |
| `general.max_commits_in_graph` | `100000` | Waar het lazy laden stopt (ongeveer 300 bytes per commit); `0` = geen limiet. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All op de achtergrond elke N minuten, 0 tot 60; `0` = uit. **Herstart.** |
| `general.auto_fetch_interval_unit` | `"auto"` | De eenheid waarin Instellingen het interval toont: `minutes`, `hours` of `auto`. Alleen weergave: het interval is altijd `auto_fetch_interval_minutes` minuten. `auto` (en een config zonder de sleutel) kiest de grootste eenheid die de minuten gelijk deelt (60 leest als "1 uur", 45 als "45 minuten"); een eenheid die niet past, wordt genegeerd. Instellingen schrijft beide sleutels; een oudere GitPoulpi negeert de eenheid en leest nog steeds de minuten. |
| `general.fetch_on_tab_open` | `true` | Fetch All als een tabblad opent (overgeslagen als GitPoulpi die repository minder dan 60 s geleden heeft gefetcht). |
| `general.auto_prune` | `true` | Fetches ruimen (prune) ook remote-tracking branches op die niet meer bestaan. |
| `general.auto_prune_tags` | `false` | Een fetch verwijdert ook de lokale tags die de remote heeft verwijderd (alleen tags die erop stonden; een tag die je nooit hebt gepusht, blijft). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Vernieuwing van de lijsten met pull requests / issues en de chip in de werkbalk, 0 tot 120; `0` = alleen op verzoek en bij gebeurtenissen. |
| `general.hosting_refresh_unit` | `"auto"` | De eenheid waarin Instellingen de vernieuwing toont: `minutes`, `hours` of `auto`, gelezen zoals `general.auto_fetch_interval_unit`. Alleen weergave. |
| `general.hosting_post_as` | `""` | De naam waarmee berichten naar een host worden geplaatst als het token van de repository van een bot is (een token van een GitHub App), bijvoorbeeld `my-app[bot]`: getoond in elk schrijfvak en gebruikt om je eigen opmerkingen te herkennen. Leeg: gevraagd aan de host, anders geleerd van het eerste bericht. Een repository kan het overschrijven (Instellingen, SSH en inloggegevens van de repository). |
| `general.keep_submodules_up_to_date` | `false` | Na een checkout of pull `git submodule update --init --recursive` als een submodule niet gelijkloopt. Zie [Submodules](submodules.md). |
| `general.default_branch_name` | `"main"` | Eerste branch van een repository die met Init is gemaakt; leeg laat git beslissen. Hernoemt niets. |
| `general.delete_orig_files_after_merge` | `false` | Verwijder na een merge die slaagt de back-upbestanden `*.orig`. |
| `general.remember_tabs` | `true` | Heropen de tabbladen van de vorige sessie. Uit opent een start binnen een repository alleen die repository, en een start buiten een repository alleen Start. **Herstart.** |
| `general.radar_enabled` | `true` | De [Radar](radar.md): zijn tabblad, palet-opdrachten, `Alt+R` en het ophalen op de achtergrond. Uit verbergt dit alles en er wordt niets voor opgehaald. Geldt meteen; niet getoond in een build zonder de feature `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | Het tabblad waarop een start opent als de map waarin hij begint binnen een repository ligt: `"current-repo"` (het tabblad van die repository, indien nodig geopend), `"last"` (het tabblad dat actief was bij het laatste afsluiten), `"home"` of `"radar"`. Een repository die met `--repo` is opgegeven opent altijd zijn eigen tabblad. `"last"` is Start als `remember_tabs` uit staat of er niets onthouden is; `"radar"` is Start als Radar uit staat of niet in de build zit. Eenmalig gelezen bij het opstarten; vereist een herstart. |
| `general.startup_tab_outside_repo` | `"home"` | Het tabblad waarop een start opent als de map waarin hij begint geen repository is (bijvoorbeeld een gebruikersmap): `"home"`, `"last"` of `"radar"`. `"last"` is Start als `remember_tabs` uit staat of er niets onthouden is; `"radar"` is Start als Radar uit staat of niet in de build zit. Eenmalig gelezen bij het opstarten; vereist een herstart. |
| `general.startup_tab_launcher` | `"last"` | Het tabblad waarop een start opent als GitPoulpi wordt gestart vanaf de snelkoppeling op het bureaublad of in het Startmenu die het zelf maakte (die geeft `--launcher` mee), welke map ook: `"home"`, `"last"` of `"radar"`. Heeft voorrang op de twee instellingen hierboven. `"last"` is Start als `remember_tabs` uit staat of er niets onthouden is; `"radar"` is Start als Radar uit staat of niet in de build zit. Eenmalig gelezen bij het opstarten; vereist een herstart. |
| `general.group_worktree_tabs` | `true` | Houd de tabbladen van de gekoppelde worktrees van een repository direct achter het tabblad van die repository: wanneer een worktree opent, wanneer een worktree-tabblad wordt weggesleept (het springt terug naast zijn groep; het repositorytabblad is de kop van de groep en zijn worktrees volgen erachter) en wanneer de instelling wordt ingeschakeld. Uit laat tabbladen staan waar je ze neerzet. |
| `general.undo_depth` | `100` | Hoeveel bewerkingen Ongedaan maken kan terugdraaien; `0` = onbeperkt. |
| `general.conflict_detection` | `true` | Proactieve conflictdetectie: een label in de Log als het mergen van de huidige branch in de branch waarvan hij is gestart zou conflicteren, op de achtergrond berekend (een merge in het geheugen: er verandert niets in je bestanden, de index of de refs). Een repository kan dit voor zichzelf uitschakelen (`[preferences] conflict_detection`). De basisbranches waarmee wordt vergeleken hebben eigen lijsten: [`[conflict_prevention]`](#conflict_prevention-lijsten-met-basisbranches). Zie [Merge-conflicten oplossen](merge-conflicts.md#conflictpreventie). |
| `general.dev_mode` | afwezig (automatisch) | Voor wie GitPoulpi vanuit een checkout bouwt; nooit getoond in de Instellingen. Afwezig: aan als het draaiende programma in `<checkout>/target/<profile>/` staat (`Cargo.toml` en `crates/gitpoulpi-tui` naast `target`). `true` / `false` dwingt het af. De ontwikkelmodus ontgrendelt alleen `ui.launcher_build`. |
| `general.journal_outside` | `true` | Wat een shell of een AI-agent doet met branches, tags, stashes en HEAD vastleggen als items van de ongedaanmaakgeschiedenis, gemarkeerd als extern, zodat `Ctrl+Z` het kan terugdraaien. Bewerkingen van bestanden worden nooit vastgelegd. Vereist `undo_enabled`. Zie [Ongedaan maken en opnieuw](undo-redo.md#wijzigingen-buiten-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: helemaal geen administratie voor Ongedaan maken (het snelst), de ongedaanmaakgeschiedenis wordt gewist, de knoppen Ongedaan/Opnieuw verdwijnen. Zie [Ongedaan maken en opnieuw](undo-redo.md). |
| `general.active_profile` | `""` | Id van de `[[profiles]]`-identiteit waarmee commits worden gemaakt. Leeg, of een profiel met een lege naam of e-mail: `user.name` / `user.email` van git. |
| `general.external_editor` | `""` | Opdracht voor *Openen in externe editor* van een bestand, bijv. `code -r {file}`. Leeg: VS Code als dat is geïnstalleerd, anders de systeemstandaard. |
| `general.external_diff_tool` | `""` | Opdracht voor *Openen in externe diff-tool*: `{local}` (gecommitte / HEAD-versie), `{remote}` (versie in de werkmap / index). Leeg: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Opdracht voor *Openen in externe merge-tool* op een conflictkaart (een binair, LFS- of te groot bestand): `{base}`, `{local}` (A, huidig), `{remote}` (B, inkomend) en `{merged}` (het bestand waarin de tool het resultaat schrijft; GitPoulpi stagt het als de tool succesvol afsluit). Leeg: `git merge-tool --no-prompt`. |
| `general.file_explorer` | per OS | Opdracht voor *Tonen in map*: `{path}` (het bestand), `{dir}` (zijn map). Standaarden: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

De drie opdrachtsjablonen worden eerst in argumenten gesplitst (`"..."` groepeert een
woord), daarna worden de plaatshouders ingevuld, zodat paden met spaties één argument
blijven. Er komt geen shell aan te pas.

```toml
[[profiles]]                # commit-identiteiten (Instellingen > Profielen)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log en grafiek**

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (een afbeelding waar de terminal die kan tekenen, elders tekens), `image`, `text`. **Herstart.** |
| `ui.image_protocol` | `"auto"` | Hoe afbeeldingen (grafiek, avatars) de terminal bereiken: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = Kitty-protocol in kitty, Ghostty, iTerm2 3.7.7 of nieuwer en een herdr met `kitty_graphics = true`; het iTerm2-protocol in WezTerm en Rio (transparantie doorgemengd), oudere iTerm2 en mintty (ondoorzichtige achtergrond); sixel (de grafiekkolom en avatars) in Windows Terminal, foot, Tabby, Konsole, mlterm, in tmux als de terminal erachter sixel aankan, en in elke terminal waarvan de opstartantwoorden sixel noemen (een xterm gestart met `-ti vt340`, een VS Code-terminal met afbeeldingen aan); tekens in zellij, screen en onbekende terminals. In tmux hebben kitty-afbeeldingen hier `kitty` nodig plus `allow-passthrough on` ([Multiplexers](multiplexers.md)). Waar de pseudoconsole van de terminal afbeeldingen weggooit of verkeerd plaatst (WezTerm 20240203, Rio en mintty op Windows) gebruik je `ui.graph_renderer = "text"` ([Een terminal kiezen](terminals.md#windows)). **Herstart.** |
| `ui.theme` | `"system"` | Het kleurthema: een ingebouwde slug (`gitpoulpi theme list` toont ze allemaal), de slug van een bestand in `<config>/themes/`, of `system` (volgt de lichte of donkere achtergrond van de terminal, bij het opstarten gedetecteerd; de oude waarde `default` betekent hetzelfde). `config set` weigert onbekende namen. Zie [Thema's](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Baanpalet: `theme` (het thema volgen) of een naam van een baanpalet: `gitpoulpi`, `light`, `material`, `serie` en de paletten van de andere thema's. |
| `ui.graph_lane_background` | `"off"` | `off` of `row_tint` (een zwakke band in de baankleur achter de grafiek, plus een balk aan de rand). **App: Rijkleuring van de grafiek omschakelen** (`Ctrl+N Shift+Y`) schakelt het om; **App: Kleurthema van de grafiek wijzigen...** (`Ctrl+N Shift+T`) kiest `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (alleen met `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` en `right` tekenen een kolom; een venster smaller dan 80 kolommen valt terug op `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: of een dialoog dat je hebt verplaatst of geschaald (sleep de titelbalk of de randen, of `Ctrl+N m`) zijn plaats houdt. Eén instelling voor alle dialogen (elk houdt daaronder zijn eigen plaats en grootte): tot GitPoulpi sluit (standaard), op schijf voor de volgende start (elk dialoog in het ene `dialog_geometry.txt` in de gegevensmap) of helemaal niet (elk dialoog opent op zijn standaardplaats en -grootte). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: waar de knoppen langs de balk staan (links, midden, rechts van een rij; boven, midden, onder van een kolom). |
| `ui.toolbar_pr_pinned` | `true` | De pull request-knop zit vast aan het uiteinde van de werkbalk; `false` laat hem met de andere knoppen meestromen. |
| `ui.statusbar_position` | `"bottom"` | `top` (boven de tabbladen van de repository) of `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (de laatste regel van het venster, onder de statusbalk) of `left` (een kolom aan de linkerrand; onder 80 kolommen blijft ze bovenaan). |
| `ui.graph_show_stashes` | `true` | Een rij per stash in de Log. |
| `ui.ref_connector` | `true` | Een dunne lijn van een ref-pil naar zijn commit. |
| `ui.column_resize` | `"line"` | Hoe je een kolomgrens vastpakt in de Log, de Radar en de commitkolom van de diff-, bestands- en blame-weergaven. `line`: elke grens is een tussenruimte van één cel met een klein streepje in de kop; zolang de aanwijzer op de grens staat, in de kop of overal in de tabel, wordt een dunne accentlijn over de volle hoogte getekend en schaalt slepen vanaf elk punt ervan de kolom. `header`: geen tussenruimte en geen streepje; het eerste teken van de kopcel na de grens licht op en slepen vanaf daar schaalt de kolom (alleen in de kop). De toetsen om te schalen en de palet-rijen zijn in beide standen hetzelfde. |
| `ui.ref_tree` | `true` | Groepeer branches en tags op `/` in mappen in de Refs-zijbalk (`y` schakelt het om). |
| `ui.tree_folders_first` | `true` | In de boomweergave van de bestandspanelen (Wijzigingen, details, vergelijken) toont een map eerst zijn submappen en dan zijn bestanden; `false` behoudt de gewone padvolgorde. De platte lijst blijft ongewijzigd (Instellingen > Log & grafiek). |
| `ui.graph_ghost_ref` | `true` | De kolom Ref van de Log toont een vervaagde pil met de naam van de branch (anders de remote branch) waartoe een aangewezen commit behoort, als de commit geen eigen ref heeft en de kolom ruimte heeft; nooit een tag. Een verwijderde branch wordt afgeleid uit het onderwerp van de merge-commit die hem samenvoegde en doorgestreept getekend. Wijs je de pil aan, dan zie je het soort. |
| `ui.reveal_on_ref_click` | `true` | Eén klik op een ref (branch, tag, remote branch, stash) in het Refs-paneel scrolt de Log naar de bijbehorende commit en laat de rij even oplichten; de commit-selectie verplaatst niet. |
| `ui.log_wip_counts_in_message` | `true` | WIP-wijzigingsaantallen in het WIP-bericht zolang de kolom Wijzigingen verborgen is. |
| `ui.log_columns_layout` | `"fit"` | Hoe de Log zijn kolommen indeelt. `fit`: elke zichtbare kolom past precies in het paneel en alleen `message` rekt mee; is het paneel smal, dan wijken hele kolommen. `scroll`: kolommen behouden hun breedte (`message` ook), niets wijkt, en de tabel scrollt zijwaarts als hij breder is dan het paneel. |
| `ui.log_column_widths_scroll` | leeg | Verouderd: de kolombreedtes van de Log gelden nu per repository, in zijn `repo-state.toml` (`[log_columns]`, de tabellen `fit` en `scroll`, naam -> cellen; `scroll` mag `message` bevatten, `fit` nooit). Een repository zonder opgeslagen breedtes gebruikt de verouderde `ui.log_column_widths` / `ui.log_column_widths_scroll` als je die hebt, anders wordt hij bij het eerste openen aan zijn inhoud aangepast. |
| `ui.log_columns_hidden`, `ui.log_columns_order` | leeg | De getoonde Log-kolommen (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`) en hun volgorde, geschreven als je een kolom verbergt of sleept; gedeeld door elke repository. `message` kan niet worden verborgen. `ui.log_column_widths` is verouderd (zie hierboven). |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | leeg | De delen van de [commit-kolom](interface-tour.md#de-commit-kolom) naast een diff, op zichzelf, los van die van de Log: de getoonde delen bij naam (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; leeg is banen, bericht, auteur en datum), hun volgorde van links naar rechts bij dezelfde namen (leeg is de zojuist genoemde volgorde; een weggelaten naam volgt in die volgorde) en de gesleepte breedtes in cellen per deel (een weggelaten deel is automatisch), geschreven als je de delen kiest, verplaatst of hun grootte wijzigt. `message` kan niet worden verborgen. Een sessiebestand dat vóór deze sleutels is geschreven, wordt eenmaal in de sleutels ingelezen. |
| `ui.sidebar_sections_hidden` | leeg | De secties van het linkerpaneel die je hebt verborgen (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), geschreven vanuit het rechtsklikmenu van het paneel of het palet. Minstens één bestaande sectie blijft altijd zichtbaar. Dit is de standaard: elke repository kan eigen secties verbergen (bewaard in het statusbestand van die repository) en deze sleutel geldt voor een repository zonder eigen keuze; *Voor alle repositories gebruiken* schrijft hem. |
| `ui.sidebar_sections_scope` | `"repository"` | Voor wie de secties van het linkerpaneel gelden: `repository` (elke repository houdt een eigen keuze, met `ui.sidebar_sections_hidden` als standaard) of `global` (elke repository toont `ui.sidebar_sections_hidden`, en een wijziging wordt daar geschreven). Wisselen verliest nooit een van beide sets: de keuzes per repository blijven in hun statusbestanden staan zolang `global` aan staat. Instellingen: *Secties van het linkerpaneel*. |
| `ui.language` | `"auto"` | De interfacetaal: `auto` (de systeemlocale, anders Engels), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Live.** `GITPOULPI_LANG` overschrijft het. Een taal die niet in deze build zit, toont Engels. **App: Taal wijzigen...** (`Ctrl+N l`) stelt hem ook in, met een live voorbeeld. De uitvoer van CLI en MCP blijft Engels. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... blijven in elke taal Engels. `true`: het git-vocabulaire van de gekozen taal (Frans "remiser", "étiquette", "valider"...). **Live.** |
| `ui.git_terms_english` | `false` | Alleen ja, ko en zh-CN: toon de Engelse git-woorden (stage, unstage, staged, unstaged...) in plaats van hun transcriptie. Geen effect zolang `ui.translate_git_terms` `true` is. **Live.** |
| `ui.mouse_enabled` | `true` | `false` laat de muis (en tekstselectie) aan de terminal. **Herstart.** |
| `ui.mouse_scroll_lines` | `3` | Log-rijen per tik van het wiel, 1 tot 20. **Herstart.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` vraagt de terminal om het kitty-toetsenbordprotocol (zodat `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` aankomen; alleen Unix, tmux antwoordt nooit: zie [Multiplexers](multiplexers.md#tmux)); `off` vraagt nooit. **Herstart.** |
| `ui.logo_color` | `""` | Kleurstelling van het logo in het venster Over, een slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); leeg of onbekend betekent de standaard van de build (`lavender`). Geschreven als je op het logo klikt of in Over een kleur kiest. |

**Avatars** (standaard uit; als je ze aanzet, wordt een SHA-256-hash van de e-mailadressen
van auteurs naar Gravatar gestuurd en wordt je verbonden GitHub/GitLab-account gevraagd)

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (in de commit-cirkel van de afbeeldingsgrafiek), `author` (in de kolom Auteur), `off`. De oude `ui.show_avatars` (`true`) wordt nog als `author` gelezen zolang `ui.avatars` ontbreekt. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Zoekvolgorde: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Een GitLab-host wordt alleen via een verbonden profiel gevraagd. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (lokaal gegenereerd) of `initials`. Op een terminal zonder afbeeldingsprotocol wordt altijd de initialenbadge gebruikt. |

**Weergave van diff en editor**

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` of `split` (`m` in een diff). |
| `ui.diff_hunks_only` | `true` | `false` toont het hele bestand met de wijzigingen op hun plaats (`H`). |
| `ui.diff_show_whitespace` | `true` | Toon wijzigingen die alleen witruimte betreffen (`s`). |
| `ui.diff_word_wrap` | `false` | Breek lange diff-regels af (`w`; Instellingen > Diff & editor). |
| `ui.minimap` | `false` | Een minimap naast de schuifbalk van de diff en de editor (`M`). |
| `ui.mermaid_detail` | `refs` | Detail van Kopiëren als Mermaid gitGraph van de Log: `structure`, `refs`, `subjects`, `full` (onthouden uit de popup; de standaard van `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Richting van die grafiek: `lr` (van links naar rechts) of `tb` (van boven naar beneden). |
| `ui.mermaid_fence` | `false` | Verpakt het gekopieerde Mermaid-script in een codeblok `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Commit-onderwerpen worden in de Mermaid-id's op zoveel tekens afgekapt (`subjects`, `full`). |
| `ui.text_cursor` | `"terminal"` | Waar de invoegpositie van een actief tekstveld wordt getoond: `terminal` zet de eigen cursor van de terminal daar (een knipperend streepje, zodat de cursorinstellingen van de terminal en het kandidatenvenster van een invoermethode meebewegen; verborgen als geen veld het toetsenbord heeft en bij het afsluiten hersteld naar je eigen cursorstijl), `drawn` tekent de eigen invoegpositie van GitPoulpi en houdt de terminalcursor verborgen. Live. |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` of `16`: met hoeveel kleuren wordt getekend. `auto` is truecolor behalve op Apple Terminal (256), de Linux-console (16) en `*-256color`-terminals zonder `COLORTERM` (256). Live. Zie [Thema's](themes.md). |
| `ui.input_color` | `"theme"` | Achtergrond van tekstvelden: `theme` (die van het thema zelf), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | De `custom`-kleur, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | De AI-commit-composer opent met de bestanden van elk voorstel `expanded` of `collapsed`. |
| `ui.blame_side` | `"right"` | Welke kant van de Blame-weergave de annotatiekolom (avatar, auteur, leeftijd, samenvatting) draagt: `right` (standaard) of `left`. |
| `ui.blame_width` | `46` | Breedte van de annotatiekolom van Blame in cellen (24 tot 90). Sleep haar rand of druk in de weergave op `<` / `>`; de nieuwe breedte wordt hier opgeslagen. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request op GitHub, merge request op GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Verbergt de werkbalkchip van het laatste pull of merge request van de uitgecheckte branch zodra dat gesloten of gemerged is; de gedimde chip "+ PR" (een nieuw verzoek starten) toont dan in plaats daarvan, alsof de branch er geen had. `false`: de chip toont altijd het laatste verzoek, in welke status het ook staat. |

De diff-schakelaars die je in een diff wijzigt, worden hier meteen opgeslagen en gebruikt
door elke diff die daarna opent.

**Datums**: `ui.date_format` (`"YYYY-MM-DD"`, de datumkolom van Blame / Bestandsgeschiedenis),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, de kolom Datum van de Log) en
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, de commit-details, de tooltip van Datum
en *Kopiëren > Datum*). Tokens: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (weekdag, voluit / afgekort) `HH` `mm` `ss`; al het andere wordt
ongewijzigd overgenomen. Maand- en weekdagnamen volgen `ui.language`, en een maand die
na een dagnummer staat, krijgt de genitiefvorm waar de taal die heeft (Russisch,
Oekraïens, Pools, Tsjechisch). Een reeks `D MMMM YYYY` wordt in het Japans, Chinees en
Koreaans met het jaar eerst geschreven (`2024年3月5日`). `ui.date_locale` (`"system"`,
de standaard, of `"english"`) houdt de namen Engels, welke taal ook.

**Agents, terminals en de map van het tabblad**

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | Het venster accepteert `ui show` / `ui propose` van agents helemaal (Instellingen > Agents & MCP > *AI-agents de weergave laten besturen*). Uit bereikt geen enkele agent het. Zie [MCP en agents](mcp-and-agents.md#dingen-aan-de-gebruiker-tonen). |
| `ui.launch` | `"off"` | Of een agent een GitPoulpi-venster mag openen als er geen open is: `off`, een preset (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) of `custom`. |
| `ui.launch_command` | `""` | Het `custom`-sjabloon: `{repo}`, `{session}`, `{gitpoulpi}`. In argumenten gesplitst, geen shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (de tabbalk biedt *Afsluiten & cd hier*) of `auto` (elke afsluiting wijzigt de map van de shell, en een nieuw tabblad voert de standaard *Openen in*-regel uit). Zie [De map van het actieve tabblad volgen](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Verouderd, zie hieronder en [Openen in](open-in.md). Hoe het oude *Terminal hier* een terminal opende: `auto` (de `ui.launch`-preset, anders `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), een preset of `custom`. |
| `ui.terminal_launch_command` | `""` | Het `custom`-sjabloon; `{repo}` is de map. |
| `ui.open_in` | *(afwezig)* | De lijst [Openen in](open-in.md), een array van tabellen `[[ui.open_in]]` met `label` en `command`. Afwezig betekent "nog niet geconfigureerd": de lijst wordt berekend uit de verouderde `ui.terminal_launch*`-sleutels (een niet-`auto`-waarde wordt de eerste regel), de bestandsverkenner van het OS, de multiplexers en terminals die uit de omgeving worden gedetecteerd (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) en de editors op `PATH`. De eerste keer opslaan vanuit Instellingen schrijft de lijst. `open_in = []` is een lege lijst. Niet in te stellen met `config set` (bewerk het bestand). |
| `ui.open_in_default` | `""` | Label van de regel die "Openen in standaard" uitvoert; leeg of onbekend betekent de eerste regel. |
| `ui.launch_terminal` | `"auto"` | De terminal waarin de bureaubladsnelkoppelingen (Instellingen > Starter) en de herstart op Windows GitPoulpi starten. `auto` kiest uit wat is geïnstalleerd (Windows: Windows Terminal, anders WezTerm, Alacritty, de gewone console; Linux: `xdg-terminal-exec`, anders de eerste gevonden terminal; macOS: iTerm2, Ghostty, WezTerm of kitty als die geïnstalleerd is, anders Terminal), `default` is die van het besturingssysteem, of een terminal-id zoals `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Alleen in ontwikkelmodus (zie `general.dev_mode`; alleen dan getoond in Instellingen > Starter, anders genegeerd). `"current"`: snelkoppelingen en de opdrachtalias starten de build die ze heeft gemaakt. `"newest"`: ze starten de meest recente build van `target/dist` en `target/release` van de checkout, bij elke klik bepaald, en het starten van een van beide builds herschrijft ze nooit. |
| `ui.command_alias` | `""` | De naam van de opdracht die GitPoulpi vanuit een terminal start, bijvoorbeeld `gp` (Instellingen > Starter, `gitpoulpi alias create`). Leeg: geen. Alleen letters, cijfers, `-`, `_` en `.`. Het is alleen de naam die de knop **Maken: opdrachtalias** gebruikt; er bestaat niets tot je het maakt. |
| `ui.command_alias_add` | `""` | De naam van de tweede opdracht, bijvoorbeeld `gpa`, die `gitpoulpi --add` uitvoert: ze opent de huidige map als nieuw tabblad in het al draaiende GitPoulpi (Instellingen > Starter, `gitpoulpi alias create gpa --add`). Leeg: geen. Dezelfde naamregels; samen met `ui.command_alias` gemaakt door de knop **Maken**. |
| `ui.relaunch_in_terminal` | `true` | Alleen Windows: als de exe met een dubbelklik in een eigen console start, opent GitPoulpi zich opnieuw in `ui.launch_terminal` en sluit af. Een terminal die de exe rechtstreeks draait (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio, Tabby, Alacritty `-e`) is ook een console van zichzelf, maar geen klassiek consolevenster: hij wordt nooit heropend. Uit: hij blijft in die console. |
| `ui.reuse_window` | `true` | Een klik op een GitPoulpi-snelkoppeling (`gitpoulpi launch`) toont het venster van een GitPoulpi dat al open staat, in plaats van een nieuw te starten. Uit: elke klik start een nieuw venster. |

**`ui.open_in`-opdrachten** worden eerst in argumenten gesplitst (witruimte, `'...'`, `"..."`), daarna worden de variabelen binnen elk argument vervangen, zodat een map met spaties één argument blijft; er komt geen shell aan te pas, tenzij de opdracht er een start (`sh -c '... "$GITPOULPI_CWD"'`). Variabelen: `${cwd}` (de map van het tabblad), `${repo}` (repositoryroot), `${name}` (basisnaam van de map), `${branch}` (leeg bij een losgekoppelde HEAD), `${web_url}` (de webpagina van de remote `origin` als dat een GitHub- of GitLab-project is, anders leeg; een regel die hem gebruikt weigert te draaien als hij leeg is), `${env:NAME}` (leeg als niet ingesteld). De oude `{repo}` betekent nog steeds `${cwd}`. Elke andere `${...}` is een fout die hem noemt. Voorbeeld:

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

Verouderd: `ui.terminal_launch` en `ui.terminal_launch_command` worden nog gelezen (ze vullen de lijst zolang `ui.open_in` ontbreekt: `custom` wordt "Terminal (custom)" met `{repo}` herschreven naar `${cwd}`, een preset-id wordt die preset), maar zijn vervangen door `ui.open_in`.

### Afbeeldingen

| Sleutel | Standaard | Betekenis |
|---|---|---|
| `ui.image_view` | `true` | Toon een binair bestand dat een afbeelding is (PNG, JPEG, GIF, WebP, BMP, ...) als afbeelding in de Diff-weergave en de Bestandsweergave. Uit: de gewone binaire melding. |
| `ui.image_view_max_mib` | `32` | Afbeeldingen groter dan dit (MiB) worden niet gedecodeerd; in plaats daarvan verschijnt een infokaart. |
| `ui.image_view_max_unit` | `"auto"` | De eenheid waarin Instellingen de limiet toont: `mib`, `gib` (1024 MiB) of `auto`. Alleen weergave: de limiet is altijd `image_view_max_mib` MiB, 1 tot 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | Op een terminal zonder afbeeldingsprotocol: `halfblock` (gekleurde halve-blok-tekens) of `card` (alleen infokaart). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom vanaf waar vergroten vierkante pixels toont in plaats van te egaliseren: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | De panelen voor en na delen één zoom en scrollpositie. |
| `ui.image_editors` | *(afwezig)* | De afbeeldingseditors van "Bewerken in", een array van tabellen `[[ui.image_editors]]` met `label` en `command`. Afwezig betekent "nog niet geconfigureerd": de lijst bestaat uit de editors die op deze machine zijn gedetecteerd (programma's op `PATH`, Flatpak-installaties, bekende installatiemappen op Windows, `/Applications` op macOS), daarna de standaardapp van het OS. De eerste keer opslaan vanuit Instellingen schrijft de lijst. `image_editors = []` is een lege lijst. Niet in te stellen met `config set` (bewerk het bestand). |
| `ui.image_editor_default` | `""` | Label van de editor die "Bewerken in standaard" uitvoert; leeg of onbekend betekent de eerste regel. |

**`ui.image_editors`-opdrachten** worden eerst in argumenten gesplitst (witruimte, `'...'`, `"..."`), daarna worden de variabelen binnen elk argument vervangen, zodat een pad met spaties één argument blijft; er komt geen shell aan te pas. Variabelen: `${file}` (het absolute pad van de afbeelding), `${dir}` (zijn map), `${name}` (zijn bestandsnaam), `${repo}` (de repositoryroot, leeg bij een tijdelijke kopie), `${env:NAME}` (leeg als niet ingesteld). Elke andere `${...}` is een fout die hem noemt. Een afbeelding die niet het bestand in de werkmap is (de versie van een commit, de index, een verwijderd bestand) wordt geopend als een **alleen-lezen tijdelijke kopie**, zodat een editor de repository niet kan aanraken. Editors zonder opdrachtregel (Affinity Photo uit de Microsoft Store, sommige Photoshop-installaties) staan niet in de presets: schrijf een eigen regel. Voorbeeld:

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

**Nog niet gebruikt** (geaccepteerd, opgeslagen, maar niets leest ze):
`ui.graph_colorization` (uit Instellingen verwijderd;
bij het laden nog geaccepteerd), `ui.date_word_format` (uit Instellingen verwijderd; bij
het laden nog geaccepteerd). De oude `ui.diff_view_mode`
wordt eenmalig gelezen als `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `editor.tab_size` | `4` | Kolommen per tabstop, 1 tot 8 (editor en diffs). |
| `editor.eol` | `"auto"` | Regeleinde dat de ingebouwde editor schrijft: `auto` (dat van het bestand behouden), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Taalkleuren in de editor. |
| `editor.line_numbers` | `true` | Regelnummers in de editor en de diffs. |
| `editor.follow_selection` | `true` | Zolang de editor open is, opent het selecteren van een ander bestand in het rechterpaneel dat ook in de editor, nadat is gevraagd of niet-opgeslagen wijzigingen moeten worden opgeslagen. Een bestand dat niet bewerkt kan worden, opent in de gebruikelijke weergave. Een bestand van een commit verlaat de editor voor de diff ervan. |

## `[merge]`

De merge-conflicteditor ([Merge-conflicten oplossen](merge-conflicts.md)). Instellingen > Diff & editor toont hem.

| Sleutel | Standaard | Betekenis |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | De breedte, in kolommen, van het gebied van de editor vanaf waar A en B naast elkaar worden getoond (lange regels scrollen zijwaarts: Shift+wiel, Shift+pijlen). Smaller toont hij één kant tegelijk achter tabbladen. `40` is de laagste waarde, `0` betekent altijd naast elkaar (de melding onder 40 kolommen blijft). De editor verbergt de panelen Refs en Status zolang hij open is, dus zijn gebied is meestal het hele venster. |

## `[git]`, `[credentials]`, `[ssh]`

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `git.pull_mode` | `"merge"` | Wat Pull doet (knop, `Ctrl+G p`, `Git: Pull`): `merge` (fast-forward waar mogelijk), `ff_only`, `rebase`. Wordt altijd expliciet aan git doorgegeven. De oude `fetch_all` wordt als `merge` gelezen. |
| `git.push_behavior` | `"direct"` | `direct` pusht naar de standaard-remote; `ask` toont eerst het remote-menu. |
| `git.push_group_preflight` | `true` | Voordat een branch naar meerdere remotes wordt gepusht, eerst elk doel controleren (`ls-remote`, plus een fetch van die remote alleen als hij afwijkt), zodat een probleem op een latere remote wordt gevonden vóór de eerste push. |
| `git.merge_fast_forward` | `"never"` | Wat een merge die je start (branchmenu, slepen en neerzetten, palet, `gitpoulpi merge`) doet als de branch een fast-forward kan zijn: `never` maakt altijd een merge-commit (`--no-ff`), `allowed` doet een fast-forward als dat kan (het standaardgedrag van git, `merge.ff` geldt), `only` doet een fast-forward of weigert. De eigen keuze van een repository (Instellingen > Merge-opties, `merge_fast_forward` in zijn `repo-state.toml`) wint; `--ff`, `--no-ff` en `--ff-only` van de CLI winnen van beide. Direct. |
| `git.default_encoding` | `"utf-8"` | Codering voor tekst die **geen** geldige UTF-8 is (bestandsinhoud, blame, diffs, commit-berichten zonder `encoding`-header): elk WHATWG-label (`windows-1252`, `shift_jis`, ...). Alleen weergave; staging codeert nooit opnieuw. Een repository kan dit overschrijven. |
| `git.lfs_enabled` | `true` | Toon en bestuur Git LFS. Een repository kan dit overschrijven. Zie [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Installeer vóór een pull of push van een repository die LFS gebruikt de LFS-filters / pre-push-hook (`git lfs install --local`, alleen deze repository) als ze ontbreken. Zie [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Niet gebruikt*: `core.autocrlf` / `core.eol` van een repository zijn eigen git-instellingen, te bewerken in Instellingen > Git & codering (of `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | HTTPS-authenticatie voor fetch, pull, push, clone: `credential_helper` (die van git), `prompt` (helpers negeren; git vraagt op een terminal, dus het mislukt in de daemon op de achtergrond), `none` (alleen publieke remotes). Oude `encrypted_file`-waarden worden als `credential_helper` gelezen. |
| `ssh.key_type` | `"ed25519"` | Type dat *Sleutelpaar genereren* maakt: `ed25519`, `rsa` (4096 bits), `ecdsa` (NIST P-256); het bestand is `gitpoulpi_<type>`. SSH zelf gebruikt altijd je ssh-agent en ssh-configuratie. |
| `ssh.private_key_path`, `ssh.public_key_path` | niet ingesteld | Vastgelegd door *Sleutelpaar genereren*. Als `ssh.private_key_path` is ingesteld, draaien fetch, pull, push en clone git met `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` doet hetzelfde (ssh kiest de bijpassende agentsleutel) als het privépad leeg is. Een `GIT_SSH_COMMAND`/`GIT_SSH` in de omgeving of een `core.sshCommand` in de git-configuratie (de pin per repository, `gitpoulpi ssh-command set --key`) wint. Wordt vóór elke overdracht gelezen, dus wijzigingen gelden voor de volgende. |

## `[[integrations]]` (hosting en issuetrackers)

Geschreven door Instellingen > Integraties of `gitpoulpi hosting integrations add`. Elk
type heeft een client (alleen persoonlijke tokens, geen OAuth).

```toml
[[integrations]]
id = "corp"                         # wat --id aanneemt; het token staat onder integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # de API-root wordt eruit afgeleid
token_ref = "integration:corp"      # een verwijzing, nooit het token
name = "Corp GitLab"                # optionele weergavenaam
account = "me@example.com"          # optioneel, zie hieronder
enabled = true                      # false: genegeerd bij het herkennen van hosts
```

Een zelf gehoste server kan gewoon http gebruiken: `url = "http://localhost:3000"` blijft
zoals getypt (het schema en de poort gaan mee in elk verzoek, en een remote op die host
past bij het profiel). Een token dat over gewoon http wordt verstuurd, gaat onversleuteld
over het netwerk; gebruik http dus alleen op een lokaal of vertrouwd netwerk.

`kind` accepteert ook `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` en `gerrit`
(Gitea en Forgejo delen één client; een Forgejo-profiel heeft standaard `https://codeberg.org`). Jira,
Trello, Linear, YouTrack en Redmine zijn alleen issuetrackers; Gerrit is alleen voor review (zijn wijzigingen zijn de
pull requests, het heeft geen issues). Bitbucket Cloud heeft geen issuetracker.

`account` is de niet-geheime identiteit naast het token, leeg en weggelaten voor de
meeste types: het Atlassian-account-e-mailadres voor `jira_cloud` en `bitbucket` (een
leeg Bitbucket-account betekent een workspace- of repositorytoegangstoken, verstuurd
als Bearer), de API-sleutel voor `trello`, de gebruikersnaam voor `gerrit` (het token is zijn HTTP-
wachtwoord, verstuurd als HTTP basic). Het token zelf blijft het enige geheim. Voor
`azure_devops` is de `url` `https://dev.azure.com/<organization>`; een
`https://<organization>.visualstudio.com`-URL wordt als dezelfde organisatie gelezen.
Voor `azure_devops_server` is het de collectieroot, zoals
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (het verbonden account) en `ssh` (de sleutelkeuze van het profiel) worden voor
je geschreven. Het token komt uit `GITPOULPI_TOKEN_<ID>` (id in hoofdletters,
niet-alfanumerieke tekens als `_`), anders het opgeslagen token, anders je credential
helper voor die host als maar één profiel hem gebruikt (alleen GitHub- en GitLab-profielen:
de andere types lezen de credential helper nooit). Details: [Hosting en
issuetrackers](hosting-integration.md). De oude tabel `[[hosting]]` wordt *niet gebruikt*.

Voor `local_markdown` (een map met Markdown-notities als issuebron) is `url` de
map, relatief aan elke repository of absoluut, en er is geen token. De
optionele tabel `[integrations.local]` stelt `statuses` in (de kolomvolgorde van het bord,
standaard `todo`, `doing`, `done`), `done_status` (standaard `done`),
`reference_prefix` (`LOC` toont `LOC-12`) en `me` (wie "aan mij toegewezen" betekent).
Zie [Lokale issues en Obsidian](local-issues.md).

### `[integrations.radar]` (accountbrede lijsten)

Optionele tabel van een codehostprofiel (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; niet Bitbucket Cloud). Afwezig
betekent dat elke sleutel uit staat. Geschreven door `gitpoulpi hosting integrations radar`; gelezen door
`gitpoulpi hosting radar` en de Radar, en bewerkt in Instellingen > Integraties (het Radar-blok van een verbonden profiel).

```toml
[integrations.radar]
review_requests = true    # verzoeken die op mijn review wachten, elke repository
open_requests = false     # elk openstaand verzoek van mijn namespaces
include_drafts = false    # conceptverzoeken in deze lijsten houden
assigned_issues = true    # openstaande issues aan mij toegewezen, elke repository
namespaces = ["acme", "ptm/produits"]  # één item per regel in Instellingen; leeg = geen limiet
namespaces_regex = false  # true: elk item is een reguliere expressie over het volledige repositorypad
review_mode = "ask"       # ask | clone | api
```

| Sleutel | Standaard | Betekenis |
|---|---|---|
| `review_requests` | `false` | Openstaande verzoeken die om jouw review vragen, of de repository nu gekloond is of niet |
| `open_requests` | `false` | Elk openstaand verzoek van je eigen namespace en van elke org of groep waarvan je lid bent (of van `namespaces`), nooit "alles wat de token ziet" |
| `include_drafts` | `false` | Conceptverzoeken blijven buiten deze lijsten tenzij dit aan staat |
| `assigned_issues` | `false` | Openstaande issues aan jou toegewezen, elke repository |
| `namespaces` | `[]` | Welke repositories elke lijst houdt (reviewverzoeken, toegewezen issues, openstaande verzoeken); leeg betekent geen limiet, en `open_requests` doorzoekt dan je eigen namespace plus elk lidmaatschap (GitHub: hoogstens 12 worden doorzocht, GitLab: 10 groepen en 10 eigen projecten, Gitea, Forgejo en Gerrit: 10). Een item komt overeen met het volledige pad van een repository (`groep/sub/project`, Azure DevOps `Project/repo`, Bitbucket Data Center `KEY/repo`) als het eraan gelijk is of een padvoorvoegsel is dat bij een `/` eindigt: `ptm` houdt `ptm/produits/x`, niet `ptmx/y`; hoofdletters tellen niet. Instellingen toont één item per regel; een oudere config kan één door komma's gescheiden tekenreeks bevatten (`"ptm, formation"`), die nog steeds laadt en als lijst wordt teruggeschreven |
| `namespaces_regex` | `false` | Leest elk `namespaces`-item als een reguliere expressie die wordt vergeleken met het volledige repositorypad, zoals je het schrijft (dus `^ptm/(produits\|infra)/` verankert aan het begin; zonder `^` of `$` komt hij overal in het pad overeen; voeg `(?i)` toe om hoofdletters te negeren). `open_requests` doorzoekt dan je lidmaatschappen en de expressies filteren de rijen. Een ongeldige expressie wordt onder het veld in Instellingen getoond en niet opgeslagen; in een met de hand bewerkt bestand laat hij de lijsten leeg met een melding |
| `review_mode` | `"ask"` | Wat het reviewen van een verzoek van een niet-gekloonde repository doet: `ask`, `clone` (kloon het, dan review) of `api` (review via de API van de host) |

De antwoorden worden per profiel vijf minuten bewaard in `radar-account.json` naast
`config.toml`; het bestand is een cache en veilig te verwijderen.

### Het vertrouwensarchief (niet in `config.toml`)

Waarvoor elke integratie en elk AI-profiel met API-sleutel is vertrouwd (zijn adres,
de account-id, een vastgepinde certificaatsleutel, of gewone http was toegestaan) staat
in `trust.toml` in de privégegevensmap die alleen jij kunt lezen (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` op Windows, `~/.local/share/gitpoulpi/private/`
op Linux). Het maakt bewust geen deel uit van `config.toml`: wie de
configuratie kan bewerken, mag het vertrouwen niet kunnen verbreden door dat te bewerken. Wijzig het met
`gitpoulpi hosting integrations trust` en `gitpoulpi ai trust` (zie
[de beveiligingscontrole](hosting-integration.md#de-beveiligingscontrole)); het bestand wordt
door GitPoulpi herschreven, niet met de hand.

## `[mcp]` (wat AI-agents mogen doen)

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `mcp.allow_mutations` | `false` | Uit: MCP-tools die iets wijzigen worden geweigerd; lezen, `ui_*` en voorstellen blijven werken. `true` laat agents de repository wijzigen. |
| `mcp.mutation_allowlist` | `[]` | Niet leeg: alleen deze toolnamen mogen dingen wijzigen (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: weiger de `ui_*`-tools die je venster verplaatsen (niet `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (het venster toont een chip die je bevestigt), `auto` (toegepast tenzij je bezig bent), `off`. `--ui-control` op de opdrachtregel van de server overschrijft het per agent. Geldt ook voor `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Sta `ui_propose_*` / `gitpoulpi ui propose` toe: vooraf ingevulde dialogen die je zelf bevestigt. |
| `mcp.allow_history_purge` | `false` | Alleen in `config.toml` (geen rij in de Instellingen). `true` maakt de tool `rewrite_purge` zichtbaar voor agents (hij heeft nog steeds `allow_mutations` nodig). Uit ziet een agent de tool helemaal niet: een purge verwijdert de oude geschiedenis van een herschrijving, het ongedaan maken ervan en de reflogs voorgoed. De CLI en de TUI worden er niet door beperkt. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` (en zijn alias `gitpoulpi-mcp`) weigeren te starten, met een melding die deze sleutel noemt. Instellingen > Agents & MCP > *MCP-server*. Gelezen wanneer een agent de server start. |
| `mcp.bind` | `"127.0.0.1:8808"` | Het adres waarop `--http` luistert als je er geen opgeeft (`--http ADDR` / `--bind` winnen). Alleen loopback tenzij `--allow-remote`. Leeg of de oude `"stdio"` betekent de standaard. Instellingen > Agents & MCP > *Poort van MCP-server* wijzigt de poort en behoudt de host. |
| `mcp.http_auth` | `true` | `--http` beantwoordt alleen een verzoek met `Authorization: Bearer <token>` (het token van `gitpoulpi mcp token`). `false` haalt de controle weg; de server waarschuwt dan op stderr, en `--allow-remote` wordt geweigerd. Instellingen > Agents & MCP > *HTTP-authenticatie*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (een certificaat dat eenmalig onder de privégegevensmap wordt gemaakt, met zijn SHA-256-vingerafdruk afgedrukt bij de start) of `files` (`tls_cert` en `tls_key`). Met TLS aan beëindigt de server TLS zelf en bestaat er geen gewone listener. Instellingen > Agents & MCP > *TLS*. |
| `mcp.tls_cert` | `""` | PEM-certificaatketen die wordt gebruikt met `tls = "files"`. |
| `mcp.tls_key` | `""` | PEM-privésleutel die wordt gebruikt met `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Je verklaart dat een tunnel of reverse proxy voor het eindpunt de versleuteling verzorgt. Dat kan vanuit de server niet worden gecontroleerd, dus `--allow-remote` zonder TLS vereist dat dit is ingesteld. De schakel tussen de proxy en de server is platte tekst op dat netwerk. Instellingen > Agents & MCP > *Achter een tunnel of reverse proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Alleen in `config.toml`. Uit bieden de MCP-stagetools `allow_sensitive` niet aan, en een stage die een gemarkeerd bestand zou toevoegen (zie `[privacy]`) wordt geweigerd met een bericht dat de agent zegt het jou te vragen. Aan mag een agent `allow_sensitive` meegeven. De CLI en de TUI worden er niet door beperkt: die vragen het jou. |

`allow_mutations`, `mutation_allowlist` en `allow_ui` beperken alleen de MCP-server;
de CLI is je eigen shell en wordt er niet door beperkt. Zie [MCP en
agents](mcp-and-agents.md#rechten).

## `[ai]` (ingebouwde AI, standaard uit)

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `ai.enabled` | `false` | De schakelaar. Uit verschijnt nergens iets van AI en wordt niets verstuurd. |
| `ai.default_profile` | `"default"` | Id van het profiel dat acties gebruiken als ze er geen noemen. |
| `ai.global_instructions` | `""` | Tekst die aan de prompt van elke actie wordt toegevoegd. Mag `${repo}`, `${uilang}`, ... gebruiken (zie [AI > Promptvariabelen](ai.md#promptvariabelen)); hetzelfde geldt voor de instructies per actie en per repository. |
| `ai.actions.<actie>.profile` | `""` | Profiel voor één actie; leeg = de standaard. Acties: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`. Een werkruimte (`workspaces.toml`, `[workspace.ai_instructions]` en `[workspace.ai_overrides.<actie>]`) en een repository kunnen deze instructies aanvullen of vervangen: zie [AI](ai.md#instructies). |
| `ai.actions.<actie>.instructions` | `""` | Tekst die alleen voor die actie wordt toegevoegd. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` of `no`: of de lokale agent van een codereviewconcept voor die run in de repository werkt, boven de `workdir` van het AI-profiel. Een repository kan een eigen instellen (`gitpoulpi ai review-context --repo`). Zie [AI > Reviewcontext](ai.md#reviewcontext). |
| `ai.review.include_claude_md` | `true` | Geef een codereviewconcept de `CLAUDE.md` van de repository (wortel, gelezen op de branch waarop het verzoek mikt; ingekort op 8 KB). |
| `ai.review.include_agents_md` | `true` | Hetzelfde voor `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Nog een bestand voor een codereviewconcept: een pad relatief aan de repositorywortel, een absoluut pad of een link naar een bestand van een codehost (gelezen via het passende integratieprofiel, anders anoniem). Alle bestanden samen worden op 16 KB ingekort. |
| `ai.privacy.redact_secrets` | `true` | Vervang elk geheim dat de scanner in een verzoek herkent (een token, een privésleutelblok, een wachtwoord in een URL, een `password = ...`-waarde die willekeurig lijkt) door `[REDACTED:<regel>]` voordat het weggaat. Instellingen > AI toont een rode waarschuwing zolang het uit staat. |
| `ai.privacy.exclude_sensitive_files` | `true` | Laat de diff van elk bestand dat overeenkomt met `privacy.sensitive_patterns` weg uit wat wordt verstuurd. Een lokale agent die in de repositorymap werkt, kan bestanden nog steeds zelf lezen: zie [AI > Privacy](ai.md#privacy). |
| `ai.limits.conflict_bytes` | `40000` | Maximaal aantal bytes dat één merge-conflict (zijn basis-, huidige en inkomende blokken samen) mag zijn wanneer "Oplossing voorstellen" het verstuurt. Erboven faalt GitPoulpi niet: het waarschuwt en biedt *Toch verzenden* (alleen dit verzoek), *Instelling openen* en *Annuleren*; een conflict wordt nooit afgekapt. De limiet is de eigen beveiliging van GitPoulpi, niet die van het model: verhoog hem als je model genoeg ruimte heeft. Tussen 1000 en 50000000; instelbaar in Instellingen > AI > Limieten. De andere limieten hieronder werken op dezelfde manier en staan allemaal in die groep. |
| `ai.limits.diff_bytes` | `60000` | Maximaal aantal bytes van de diff die een bericht voor een commit, een uitleg, een stash-naam, een pull request-beschrijving, een commitplan of een commitsplitsing mag bevatten, en van de hele diff van een codereview. Erboven wordt niets stilzwijgend afgekapt: in de TUI biedt *Boven de AI-limiet* *Alles verzenden* (alleen dit verzoek, niets afgekapt), *Ingekort verzenden* (het einde wordt weggelaten, zoals voorheen), *Instelling openen* en *Annuleren*; de CLI en MCP kappen af en melden het in `truncated` (`--send-all` verstuurt alles). Tussen 1000 en 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Maximaal aantal bytes van de diff van één bestand in een codereview. Erboven dezelfde vraag en dezelfde twee antwoorden; *Ingekort verzenden* verkort dat bestand met een zichtbare opmerking. Hetzelfde bereik. |
| `ai.limits.review_text_bytes` | `4000` | Maximaal aantal bytes van de beschrijving van het beoordeelde pull request. Erboven kapt *Ingekort verzenden* de beschrijving af met een opmerking `[text cut]`. Hetzelfde bereik. |
| `ai.limits.review_threads_bytes` | `6000` | Maximaal aantal bytes van de lijst met reviewopmerkingen die al bestaan (gegeven zodat het model ze niet herhaalt). Erboven kapt *Ingekort verzenden* de lijst af. Hetzelfde bereik. |
| `ai.limits.review_context_file_bytes` | `8192` | Maximaal aantal bytes van één projectcontextbestand (`CLAUDE.md`, `AGENTS.md`, het extra bestand) dat aan een codereview wordt gegeven. Erboven verkort *Ingekort verzenden* het bestand met een opmerking. Hetzelfde bereik. |
| `ai.limits.review_context_bytes` | `16384` | Maximaal aantal bytes van alle contextbestanden samen. Erboven verkort *Ingekort verzenden* het bestand dat de grens overschrijdt en laat het de bestanden erna weg. Hetzelfde bereik. |
| `ai.limits.hosting_reply_context_bytes` | `49152` | Maximaal aantal bytes van alles wat een opgesteld antwoord (of een issuebeschrijving) krijgt behalve je eigen tekst: het item en zijn gesprek. Erboven vallen de oudste opmerkingen eerst weg en wordt de inkorting gemeld in `truncated`. Zelfde bereik als de andere bytelimieten. |
| `ai.limits.hosting_reply_file_bytes` | `8192` | Maximaal aantal bytes van één repositorybestand dat aan een opgesteld antwoord wordt gegeven. Gebruikt wanneer de AI de repository leest. Zelfde bereik. |
| `ai.limits.hosting_reply_map_bytes` | `6144` | Maximaal aantal bytes van het repositoryoverzicht dat aan een opgesteld antwoord wordt gegeven. Gebruikt wanneer de AI de repository leest. Zelfde bereik. |
| `ai.limits.hosting_reply_hits_per_term` | `20` | Maximaal aantal zoekresultaten dat per zoekterm van een opgesteld antwoord wordt bewaard. Gebruikt wanneer de AI de repository leest. Begrensd tussen 1 en 200. |
| `ai.limits.hosting_reply_terms` | `8` | Maximaal aantal zoektermen dat een opgesteld antwoord opzoekt. Gebruikt wanneer de AI de repository leest. Begrensd tussen 1 en 50. |
| `ai.hosting_reply.base_prompt` | `""` | De basisprompt van de twee schrijfacties (`hosting_reply`, `issue_body`). Leeg gebruikt de ingebouwde; tekst hier vervangt hem, en de instructieniveaus volgen er nog steeds op. Te bewerken in Instellingen > AI > Antwoorden op een host. |
| `ai.hosting_reply.read_repository` | `false` | Of "AI de repository laten lezen" aangevinkt begint. Standaard uit: de repository wordt alleen gelezen als je het voor dat gebruik aanvinkt. Een lokale agent of een API-profiel leest hem dan zoals beschreven in [het hoofdstuk over AI](ai.md#toegang-tot-de-repository). |
| `ai.hosting_reply.reuse_agent_session` | `true` | De sessie van een lokale agent bewaren tussen twee antwoorden als zijn harnas dat ondersteunt. Werkt zodra de AI de repository mag lezen. |
| `ai.hosting_reply.context_ttl_minutes` | `30` | Hoe lang een voor een antwoord opgebouwde repositorycontext wordt hergebruikt. Begrensd tussen 1 en 1440. Werkt zodra de AI de repository mag lezen. |
| `ai.sent_preview.<actie>` | `true` | Eén sleutel per actie (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`, `hosting_reply`, `issue_body`): toon "Wat wordt verstuurd" voordat het verzoek weggaat. "Niet meer tonen voor ..." in het dialoogvenster schrijft `false`; Instellingen > AI > *Toon wat wordt verstuurd vóór...* zet ze weer aan. `rewrite_script` en `rewrite_rules` worden geaccepteerd maar hebben geen effect: de dialoogvensters voor het herschrijven van de geschiedenis tonen altijd hun eigen paneel. `worktree_name` wordt geaccepteerd maar heeft ook geen effect: het stuurt alleen mee wat je in het dialoogvenster Worktree maken typte en de lokale branchnamen. |

De eigen tabel `[preferences.ai_review]` van een repository (in haar `repo-state.toml`, of de vermelding van een niet-gekloonde repository in `remote-reviews.toml`) neemt dezelfde sleutels aan, allemaal optioneel, plus `no_extra_file = true`: geen extra bestand voor die repository, wat `ai.review.extra_file` ook zegt. Een niet ingestelde sleutel, een niet ingestelde `extra_file` en een ontbrekende `no_extra_file` erven de globale waarde; `extra_file` op een bestand zetten wist de schakelaar en andersom.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # leeg = de standaard van het type
base_url = ""           # leeg = de standaard van het type (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # waar de API-sleutel staat; nooit de sleutel zelf
command = ""            # alleen local_agent: het exacte programma ("claude", een alias, een wrapper, een pad)
harness = ""            # alleen local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; leeg = afgeleid uit `command`
args = []               # alleen local_agent, bijv. ["-p", "--model", "haiku"]
run_mode = "headless"   # alleen local_agent: headless (standaard) | terminal (het interactieve scherm van de agent in een virtuele terminal)
terminal_args = []      # alleen local_agent: de argumenten van de terminalmodus (args behoudt de headless set)
workdir = "default"     # alleen local_agent: default | dedicated | repository (default = speciale map in terminalmodus, de map van GitPoulpi in headless modus)
timeout_secs = 0        # alleen local_agent: seconden voordat de agent wordt gestopt; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # alleen local_agent, optioneel: variabelen alleen voor dat proces
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = een van de eigen variabelen van GitPoulpi
```

`harness` kiest de suggesties die Instellingen > AI biedt (headless argumenten,
opties, modelvlag en -lijst, omgevingsvariabelen) los van `command`; beide sleutels
worden uit het bestand weggelaten als ze leeg zijn. Omgevingswaarden worden nooit gelogd
en zijn in Instellingen verborgen als ze op geheimen lijken.

Instellingen > AI biedt presets voor `harness`/`command`/`args`/`env` (de gangbare agent-CLI's, met
hun headless vlaggen, modelvlag en nuttige omgevingsvariabelen) en voor `kind`/`base_url`/`model` (de
gehoste en lokale HTTP-providers); zie [AI > De AI configureren](ai.md#de-ai-configureren).

Een profiel met de naam *Default* wordt gemaakt als er geen is. Oude bestanden met
`[[ai.presets]]` / `[ai.defaults]` worden bij het laden gemigreerd en bewaard in
`[[ai.legacy_presets]]`. Al het andere: [AI-ondersteuning](ai.md).

## `[repos]`, `[session]`

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `repos.clone_start_source` | `"last"` | De bron waarop het dialoogvenster Klonen opent: `last` (standaard, of leeg) = de bron die het toonde toen het voor het laatst werd gesloten, `url` = altijd Klonen met URL, of een profiel-id van een integratie (`id` in de `[[integrations]]`-invoer). Een profiel dat weg, uitgeschakeld of niet verbonden is, valt terug op Klonen met URL. De laatste bron zelf is status, bewaard in `clone-source.toml` in de configuratiemap, niet hier. |
| `repos.clone_root` | `""` | Waar het dialoogvenster Clone begint; leeg = `%USERPROFILE%\source\repos` of `~/src`. |
| `repos.browse_root` | `""` | Waar Bladeren opent; leeg = de laatst gekozen map, daarna home. |
| `repos.show_hidden` | `false` | De mapkiezer toont verborgen mappen (`Alt+H` schakelt om en slaat op). |
| `repos.show_siblings` | `false` | De linkerboom van de mapkiezer toont ook de andere mappen van elk geopend niveau, als een echte hiërarchie (`Alt+S` schakelt om en slaat op). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Waar GitPoulpi bewaart wat het over één repository onthoudt (ref-filters, vouwstatus, de persoonlijke workflowoverschrijving, de hostingcache): `in_git_dir` = `<git dir>/gitpoulpi` (gedeeld door alle worktrees, verschijnt nooit in `git status`), `in_worktree` = `<repo>/.gitpoulpi` (de enige plek waar een team `workflow.toml` kan committen), `user_data` = `<gegevensmap van het besturingssysteem>/repos/<naam>-<id>` (`%APPDATA%\GitPoulpi\data` op Windows, `$XDG_DATA_HOME/gitpoulpi` op Linux, `~/Library/Application Support/GitPoulpi` op macOS). De oude waarde `".gitpoulpi"` betekent nog steeds `in_worktree`. Wijzigen verplaatst niets terug: gegevens uit een oude `<repo>/.gitpoulpi` worden de eerste keer dat ze nodig zijn naar de gekozen plek verplaatst. |

## `[log]` (het applicatielogboek)

Wat het tabblad Applicatie van het activiteitenlogboek toont en wat "Diagnose kopiëren" citeert. Details: [Wanneer er iets mislukt](troubleshooting.md#wanneer-er-iets-mislukt).

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `log.level` | `"info"` | De meest uitgebreide regel die nog wordt vastgelegd: `error`, `warn`, `info`, `debug` of `trace`. Het niveauvak in het activiteitenlogboek wijzigt dit (en slaat het hier op). |
| `log.retention_days` | `7` | Aantal dagen dat logbestanden bewaard blijven, vandaag meegeteld: 1 tot 3650, tien jaar (een grotere of kleinere waarde wordt begrensd). Altijd een aantal dagen, in welke eenheid Instellingen het ook toont. Oudere bestanden worden verwijderd wanneer GitPoulpi start. Het bestand van een dag wordt ook begrensd door `log.max_file_mb`, dus het slechtste geval is ongeveer `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | De eenheid waarin Instellingen `log.retention_days` toont: `days`, `weeks`, `months`, `years` of `auto`. Alleen weergave, het verandert nooit hoe lang bestanden bewaard blijven. Een week is 7 dagen, een maand is 30 dagen (geen kalendermaand, zodat het altijd hetzelfde aantal dagen is) en een jaar is 365 dagen. `auto`, en een config die is geschreven voordat deze sleutel bestond, kiest de grootste eenheid die de dagen gelijk deelt (14 dagen leest als "2 weken", 45 dagen als "45 dagen"). Een eenheid die de dagen niet deelt (`retention_days = 10` met `weeks`) wordt genegeerd en de dagen worden getoond. Instellingen schrijft beide sleutels; een oudere GitPoulpi negeert de eenheid en leest nog steeds de dagen. |
| `log.max_file_mb` | `5` | Het bestand van een dag wordt bij deze grootte geroteerd naar één `.old`-back-up, dus een dag neemt nooit meer dan het dubbele in beslag. |
| `log.max_file_unit` | `"auto"` | De eenheid waarin Instellingen de grootte toont: `mib`, `gib` (1024 MiB) of `auto`. Alleen weergave: de grootte is altijd `max_file_mb` MiB, 1 tot 1024. |
| `log.timings` | `true` | Het tabblad Git-bewerkingen van het activiteitenlogboek toont hoe lang elke bewerking duurde (`Push - 123 ms`: een repository openen, vernieuwen, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). De duur wordt gemeten door het venster, van het verzoek tot het antwoord van de daemon, en omvat dus de IPC-heen-en-terugreis. Uit verbergt de duur en de regels voor geslaagde bewerkingen. Mislukkingen staan altijd in de lijst. |
| `log.extended` | `false` | "Uitgebreid loggen": het tabblad Git-bewerkingen toont ook de achtergrondbewerkingen (het periodieke vernieuwen dat de status opvraagt, auto-fetch) met hun duur. Ze worden altijd op niveau `debug` naar het applicatielogboek geschreven. |
| `log.commit_count` | `"system-git"` | Hoe het `geladen / totaal` in de statusbalk het totaal te weten komt. `"system-git"` voert `git rev-list --count` uit (één proces, normaal het snelst: git leest het commit-graph-bestand). `"libgit2"` telt in het proces met een libgit2-revwalk en start niets: kies dit als elk opdrachtregelproces traag start, meestal doordat antivirussoftware met realtimescans ze allemaal inspecteert; bij 50.000 commits kost het op de achtergrond ongeveer 200 ms in plaats van 12 ms, maar er wordt geen proces gestart. `"off"` telt nooit: de balk toont alleen de geladen commits. Geldt direct. Een telling met padfilter gebruikt altijd system git. |
| `log.use_mailmap` | `true` | Toon elke auteur zoals de `.mailmap` van de repository (en `mailmap.file`) hem noemt in de Log, in blame en in de bestandsgeschiedenis: de daemon vraagt git om `%aN`/`%aE` (de systeembackend) of past dezelfde bestanden zelf toe (de libgit2-backend), dus het antwoord is identiek. De opgeslagen commits veranderen niet en er wordt niets herschreven. Zoeken in de Log op auteur komt overeen met de namen zoals getoond. Geldt direct; de Log leest binnen zijn poll van 2 s opnieuw. |

De bestanden heten `gitpoulpi-YYYY-MM-DD.log` en staan in de statusmap van het besturingssysteem (`~/.local/state/gitpoulpi/logs` op Linux; `<cache>\logs` op Windows en macOS; `<config>/cache/logs` met `GITPOULPI_CONFIG_DIR`). Tokens, wachtwoorden en `Authorization`-headers worden gemaskeerd voordat er iets wordt weggeschreven.

## `[search]` (het zoeken in de werkruimte)

Waarin `gitpoulpi search` (en zijn MCP-tool) zoekt. Details: [Werkruimtes en zoeken in de CLI](cli-reference.md#werkruimtes-en-zoeken).

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (nieuwste eerst, over elke branch, remote branch en tag) waarvan de berichten de zoekindex per repository in het geheugen houdt, ongeveer 100 bytes plus het bericht per stuk. Een grotere geschiedenis wordt daar afgekapt en het antwoord zegt dat het `partial` is. `0` zet het zoeken in commits uit (branches, tags, stashes en worktrees worden nog wel doorzocht). Live gelezen: de volgende zoekopdracht gehoorzaamt een wijziging. |

## `[rewrite]` (geschiedenis herschrijven)

Wat `gitpoulpi rewrite` en de werkbank voor het herschrijven van de geschiedenis beschermen. Details: [Geschiedenis herschrijven](history-rewrite.md).

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branches (namen, of patronen waarin `*` met elke tekst overeenkomt) die een herschrijving met rust laat als ze al zijn gepusht, tenzij de run ze noemt (`--include-protected BRANCH`; `--include-all-protected` voor de push; het selectievakje per branch in de werkbank). Toegevoegd aan de bases die de `workflow.toml` van een repository als `protected` markeert. De push van een herschrijving slaat ze op dezelfde manier over. Dit is de eigen bescherming van GitPoulpi: de branchbescherming van de host bepaalt nog steeds of de force push wordt geaccepteerd. Wordt live gelezen: het volgende voorbeeld houdt rekening met een wijziging. |
| `rewrite.external_scripts` | `["shell"]` | Welke externe scripttalen de Script-regel mag gebruiken: een van `"shell"` (bash of sh; pwsh of powershell voor PowerShell-scripts), `"python"`, `"lua"` (een geïnstalleerde Lua; de ingebouwde Lua staat altijd aan en heeft geen vermelding nodig). **Een extern script draait niet in een sandbox: het werkt met jouw rechten.** Een taal die niet in de lijst staat, is grijs in de werkbank en wordt door de CLI, MCP en daemon geweigerd (`scripts_disabled`). Ook als hij in de lijst staat, draait een extern script dat uit de repository of een gedeeld recept komt pas nadat je het hebt vertrouwd (`gitpoulpi rewrite script trust FILE`, of de vraag in de werkbank); het vertrouwen hangt aan de inhoudshash van het script, dus een gewijzigd script vraagt opnieuw. Zet Python aan door `config.toml` te bewerken (een lijst kan niet met `config set` worden gezet): `[rewrite]` met daarin `external_scripts = ["shell", "python"]`. Wordt live gelezen. |
| `rewrite.script_timeout_ms` | `5000` | Hoe lang een extern script krijgt om één commit te beantwoorden. Bij een timeout wordt de hele processtructuur beëindigd en mislukt de herschrijving (`script_failed`, met de naam van de commit); er is niets verplaatst. `0` geldt als de standaard. Wordt live gelezen. |
| `rewrite.script_timeout_unit` | `"auto"` | De eenheid waarin Instellingen de tijdslimiet toont: `milliseconds`, `seconds` of `auto`. Alleen weergave: de limiet is altijd `script_timeout_ms` milliseconden. Instellingen accepteert 1 tot 600000 (tien minuten); `0` geldt nog steeds als de standaard. Wordt live gelezen. |
| `rewrite.script_total_timeout_s` | `3600` | Hoe lang één doorloop van een extern script in totaal mag duren. `0` geldt als de standaard. Wordt live gelezen. |
| `rewrite.script_total_timeout_unit` | `"auto"` | De eenheid waarin Instellingen de tijdslimiet toont: `seconds`, `minutes`, `hours` of `auto`. Alleen weergave: de limiet is altijd `script_total_timeout_s` seconden. Instellingen accepteert 1 tot 86400 (een dag); `0` geldt nog steeds als de standaard. Wordt live gelezen. |

## `[updates]` (zoeken naar een nieuwere release)

Wat de versiechip in de titelbalk met nieuwere releases doet. Details: [Updates](updates.md).

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` vraagt eenmaal per dag op de achtergrond aan GitHub of er een nieuwere release bestaat en verandert de versiechip in `vX.Y.Z beschikbaar`. `"auto"` downloadt en klaarzet de update ook zelf als deze build uit een releasearchief komt (het vraagt nog steeds voor het herstarten); voor elke andere build gedraagt het zich als `"notify"`. `"off"` doet nooit een verzoek. Geldt direct. |
| `updates.channel` | `"stable"` | `"stable"` kijkt alleen naar volledige releases; `"prerelease"` biedt ook release candidates aan (`-rc.1`). Geldt direct. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | De `owner/name` op github.com waarvan de releases worden gecontroleerd. Zolang die repository privé is, antwoordt GitHub met 404, tenzij er een token voor github.com is opgeslagen (Instellingen > Integraties); zonder token mislukt de controle stilletjes en wordt er één regel naar het applicatielogboek geschreven. |

Een beheerder kan `mode`, `channel` (als `release_channel`) en `repo`, plus enkele andere update-opties, instellen in een beleidsbestand voor de hele machine of in het Windows-register. Een sleutel die het beleid instelt, wint van `config.toml` en staat in de Instellingen als **Ingesteld door je beheerder**. `mode` in het beleid is een plafond: je kunt een lagere waarde kiezen, nooit een hogere. Zie [Distributie voor organisaties](enterprise-distribution.md#het-beleidsbestand-en-het-register).

## `[privacy]` (gevoelige bestanden)

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Padpatronen in gitignore-stijl. Een patroon zonder `/` komt overeen met de bestandsnaam op elke diepte, een met `/` met het hele pad (een voorafgaande `/` verankert het in de root); `*` en `?` blijven binnen één map, `**` gaat over mappen heen; een `!` staat weer toe wat een eerder patroon trof en de laatste treffer wint; hoofdletters worden genegeerd. Gebruikt door de stagebewaker en door `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Een bestand stagen dat overeenkomt met een patroon, of waarvan de inhoud een herkend geheim bevat, vraagt eerst (TUI), wordt geweigerd tenzij `--allow-sensitive` is opgegeven (CLI) of tenzij `mcp.allow_sensitive_staging` de agent `allow_sensitive` laat meegeven (MCP). Tekstbestanden tot 1 MiB elk en 32 MiB in totaal worden gelezen; binaire bestanden worden alleen op naam vergeleken. Zie [Stagen en committen](staging-and-committing.md#gevoelige-bestanden). |

## `[conflict_prevention]` (lijsten met basisbranches)

Welke branches de proactieve conflictdetectie als basis van de huidige branch mag nemen, is een lijst met patronen (zie [Merge-conflicten oplossen](merge-conflicts.md#conflictpreventie)). Lijsten kunnen een naam krijgen en worden gedeeld; Instellingen > Conflictpreventie bewerkt ze.

| Sleutel | Standaard | Waarden en effect |
|---|---|---|
| `conflict_prevention.default_list` | `""` | De naam van de lijst die een repository gebruikt als hij geen eigen lijst heeft. Leeg, of een naam die geen lijst heeft, betekent de ingebouwde `**` (elke branch). Wordt bij elke controle opnieuw gelezen, dus een wijziging geldt voor geopende repositories zonder herstart. |
| `conflict_prevention.lists` | geen | De lijsten met een naam, elk een eigen tabel: `[conflict_prevention.lists.release]` met `patterns = ["origin/release/**", "!origin/release/old"]`. Een lege `patterns` staat elke branch toe. Een lijst bewerken geldt voor elke repository die hem volgt. |

Welke lijst een repository volgt, staat in zijn `repo-state.toml` (hieronder). Voorrang: de eigen patronen van de repository, dan de lijst die hij volgt, dan `default_list`, dan `**`.

## `[keybindings]`

Herbindt de sneltoetsen van de TUI; Instellingen > Toetsenbord & muis bewerkt het.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # standaard Ctrl+G
nav_prefix = "Alt+n"             # standaard Ctrl+N
settings   = ["Ctrl+,", "F2"]    # een lijst = alternatieven
tab_menu   = []                  # niet gekoppeld

[keybindings.git_prefix]         # de toets na de git-prefix
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; een waarde vervangt de standaarden van
die actie, de regel verwijderen herstelt ze. Toetscombinaties: `Ctrl+Shift+N`, `Alt+n`,
`F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Een ongeldige regel blokkeert nooit
iets: de TUI negeert hem, houdt de standaard en meldt dat op de statusbalk.
Contexten, actie-id's, standaarden en de conflictregels: [Sneltoetsen](keybindings.md).

## Instellingen per repository: `.gitpoulpi/repo-state.toml`

Bewaard in de repository; de map negeert zichzelf (een `.gitignore` met `*`), dus hij
verschijnt nooit als wijziging. Geschreven door de TUI en de CLI; je bewerkt hem zelden.

| Sleutel | Wat het is | Instellen met |
|---|---|---|
| `[preferences] encoding` | Overschrijft hier `git.default_encoding` | Instellingen > Git & codering |
| `[preferences] lfs_enabled` | Overschrijft hier `git.lfs_enabled` | Instellingen > Git & codering |
| `[preferences] profile` | De `[[profiles]]`-id die hier commits ondertekent; afwezig betekent `general.active_profile` | de profielkeuzelijst in de werkbalk, **Repository: Profiel wijzigen...** |
| `[preferences] autocrlf`, `eol` | *Nog niet gebruikt* | Instellingen > Git & codering |
| `[preferences] push_after_commit` | De branch na elke commit naar zijn upstream pushen (uit als afwezig) | Instellingen > Repositoryspecifieke instellingen > Commit |
| `[preferences] skip_hooks_by_default` | Hooks overslaan staat in het commitformulier standaard aangevinkt (uit als afwezig) | Instellingen > Repositoryspecifieke instellingen > Commit |
| `[preferences] squash_merge` | Een merge van een lokale branch squasht: stagen, geen merge-commit (uit als afwezig) | Instellingen > Repositoryspecifieke instellingen > Merge-opties |
| `[preferences] template_apply`, `template_strip_comments` | Een leeg commitformulier vooraf invullen met het sjabloon; `#`-commentaarregels verwijderen bij het committen (beide uit als afwezig) | Instellingen > Repositoryspecifieke instellingen > Berichtsjabloon |
| `[preferences] conflict_detection` | Proactieve conflictdetectie voor deze repository; afwezig volgt `general.conflict_detection` | Instellingen > Repositoryspecifieke instellingen > Conflictpreventie |
| `[preferences] conflict_base_branches` | De eigen patronen van de basisbranches van de repository, één per regel (nog gebruikt als **Eigen lijst**) | Instellingen > Repositoryspecifieke instellingen > Conflictpreventie |
| `[preferences] conflict_base_source` | Waar de basisbranches vandaan komen: `default` (de globale standaardlijst), `list` (`conflict_base_list`) of `own` (`conflict_base_branches`). Afwezig: de eigen patronen als die er zijn, anders de standaard | Instellingen > Repositoryspecifieke instellingen > Conflictpreventie |
| `[preferences] conflict_base_list` | De `[conflict_prevention.lists.<naam>]` die deze repository volgt als de bron `list` is; een verwijderde lijst valt terug op de standaard | Instellingen > Repositoryspecifieke instellingen > Conflictpreventie |
| `commit-template.txt` (een bestand naast `repo-state.toml`) | Het commitsjabloon; `commit.template` van git wijst ernaar | Instellingen > Repositoryspecifieke instellingen > Berichtsjabloon |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: de AI-instructies van deze repository | Instellingen > Repositoryspecifieke instellingen > AI |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` voor één actie hier | Instellingen > Repositoryspecifieke instellingen > AI |
| `default_remote` | De remote die Push/Pull/Fetch met één klik gebruiken | remote-menu, `gitpoulpi remote default <name>` |
| `hosting_integration` | Het hier vastgepinde `[[integrations]]`-profiel | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Het providerproject dat de PR- / issuelijsten lezen, als `{ integration, project, filter }`. `filter` is optioneel: een providereigen extra clausule (een Jira-JQL-fragment, een Azure-WIQL-voorwaarde, Trello-lijstnamen) die met AND aan de lijstquery wordt gehangen; GitHub en GitLab negeren hem. | zijbalksecties, `gitpoulpi hosting link` |
| `[preferences] merge_fast_forward` | Overschrijft hier `git.merge_fast_forward`: `never`, `allowed` of `only` | Instellingen > Repositoryspecifieke instellingen > Merge-opties |
| `[preferences] default_push_targets` | De remotes waar een nieuwe branch ook naartoe pusht, in de branch gekopieerd zodra die voor het eerst een upstream krijgt | Instellingen > Repositoryspecifieke instellingen > Git & codering; zie [Meerdere remotes](multiple-remotes.md) |
| `[preferences.ai_review]`, `[preferences.ai_hosting_reply]` | De eigen waarden van deze repository voor de sleutels van `[ai.review]` en `[ai.hosting_reply]`; ontbrekende sleutels erven de globale | Instellingen > Repositoryspecifieke instellingen > AI |
| `hosting_post_as`, `hosting_identity` | De naam waaronder je berichten aan de host in deze repository worden geplaatst (overschrijft `general.hosting_post_as`), en de login die GitPoulpi leert uit het eerste bericht dat met een bottoken is geplaatst als er geen is ingesteld | Instellingen > Repositoryspecifieke instellingen > SSH & inloggegevens; zie [Gesprekken](conversations.md#plaatsen-als) |
| `assignment_blocked` | Niets kan vanuit GitPoulpi worden toegewezen in de issues en pull requests van deze repository | `gitpoulpi hosting assignment --block \| --allow` |
| `board_labels` | De labels die de kolommen van het issuebord zijn bij een host zonder workflow, in bordvolgorde | de knop **Kolommen** van het bord |
| `conversation_seen`, `conversation_events`, `posted_comment_ids` | Wat je het laatst van elk gesprek las (de rest is ongelezen), of systeemgebeurtenissen worden getoond, en de reacties die deze installatie plaatste (die een bot mag bewerken of verwijderen) | geschreven door de gespreksweergave |
| `reviews` | De lokale staat van de review in de app per verzoek: openstaande opmerkingen, bekeken bestanden | geschreven door de reviewweergave |
| `ref_visibility` | Verborgen / gesoleerde refs | Refs-zijbalk, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotes die als geheel uit de grafiek verborgen zijn (elke branch, ook de later opgehaalde) | de ⊙ en het menu van de remote, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Ingeklapte zijbalksecties en branch- en tagmappen. `[folds]` bevat ook `status_folders` (ingeklapte mappen van de bestandsbomen Unstaged / Staged) en `commit_folders` (ingeklapte mappen van de boom met gewijzigde bestanden van een commit, voor elke commit hetzelfde), elk begrensd op 500 paden, de oudste valt eerst weg | de zijbalk, de bestandsbomen |
| `pending_auto_stash` | Een stash die GitPoulpi heeft gemaakt en nog niet heeft teruggezet | geschreven door de daemon; zie [Auto-stash](log-and-history.md#auto-stash) |

## Andere omgevingsvariabelen

| Variabele | Effect |
|---|---|
| `GITPOULPI_CONFIG_DIR` | De configuratiemap (zie hierboven). |
| `GITPOULPI_TOKEN_<ID>` | Een hosting-token voor profiel `<ID>`, boven het opgeslagen token (CI, eenmalig gebruik). |
| `GITPOULPI_GIT` | Het uit te voeren `git`-programma (standaard: `git` op `PATH`; op Windows de echte `mingw64\bin\git.exe` achter de shim `cmd\git.exe` van Git for Windows, wat een proces per aanroep scheelt). |
| `GITPOULPI_LANG=fr` | De interfacetaal, boven `ui.language` (elke `ui.language`-waarde; `qps` is een pseudo-locale voor vertalers). Een onbekende of lege waarde wordt genegeerd. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Helemaal geen afbeeldingen, wat de configuratie ook zegt. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Hints voor de afbeeldingsresolutie (celgrootte in pixels; oversampling, standaard 2, `1` bespaart CPU). |
| `GITPOULPI_NO_BROWSER=1` | Start nooit een webbrowser (de URL wordt nog wel getoond). |
| `GITPOULPI_TIMING=1` | Schrijf aanvraagtijden naar `gitpoulpi-timing.log` in de tijdelijke map (prestatierapporten). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Wacht niet op Enter nadat de TUI bij een fout is afgesloten. |
