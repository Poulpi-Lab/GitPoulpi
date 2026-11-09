[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · **Svenska** · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Kortkommandon

Den här sidan listar **standardtangenterna**. De flesta kan du binda om. Appen
visar alltid de tangenter som gäller: i kommandopaletten, i `?`-hjälpen, på
statusraderna, i snabbmenyerna och i knapparnas tooltips. När du flyttar en
tangent visar alla dessa platser den nya tangenten. För musen, se
[Mus och gester](mouse-and-gestures.md).

## Binda om

Öppna **Inställningar > Tangentbord & mus**. Sidan beskrivs i
[Inställningar](settings.md#tangentbord--mus). Ändringar gäller direkt och
sparas i tabellen `[keybindings]` i `config.toml`. Du kan också redigera den
tabellen för hand:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux och GNU screen använder ofta Ctrl+B och Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Syntaxen, konfliktreglerna och varningarna finns i
[konfigurationsreferensen](config-reference.md) under `[keybindings]`. I korthet:

- Ett fel rapporteras på statusraden, och standardtangenten finns kvar.
- Två åtgärder på samma kombination i samma kontext nekas.
- Ett prefix kräver `Ctrl` eller `Alt`.
- En tangent i en ruta som skuggar en global tangentkombination ger bara en varning.

Tabellen nedan listar varje tangent du kan binda om. Vissa tangenter är fasta
och kan inte bindas om: liströrelser (`j` `k` `h` `l`, piltangenterna,
`PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, siffrorna
(räknarprefixet), textredigeringstangenterna och tangenterna `Enter` / `Tab` /
pilar i popuper. Paletten har `App: Tangentbindningar...` och
`Tangentbindningar: Återställ alla...`.

## Standardtangenter

Varje tangent du kan binda om, med id:t som används i
`[keybindings.<context>]` och dess standardvärde. Tabellen genereras från appen
(ett test håller den aktuell). Tangenter efter ett prefix visas med prefixet
(`Ctrl+G s`).

<!-- keymap:begin -->
| Kontext | Åtgärds-id | Vad den gör | Standardtangenter |
|---|---|---|---|
| `global` | `git_prefix` | Prefix för Git-åtgärder | `Ctrl+G` |
| `global` | `nav_prefix` | Navigeringsprefix | `Ctrl+N` |
| `global` | `palette` | Kommandopalett | `Ctrl+P` |
| `global` | `settings` | Inställningar | `Ctrl+,` |
| `global` | `home` | Hem (repositoryhantering) | `Ctrl+O` |
| `global` | `clone` | Klona repository | `Ctrl+Shift+N` |
| `global` | `new_tab` | Ny flik | `Alt+T` |
| `global` | `workspace_switch` | Byt arbetsyta | `Alt+W` |
| `global` | `radar` | Öppna Radar | `Alt+R` |
| `global` | `workspaces` | Öppna arbetsytor | `Alt+J` |
| `global` | `tab_menu` | Flikmeny | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Öppna i (lista) | `Alt+O` |
| `global` | `tab_open_in_default` | Öppna i standard | `Ctrl+Alt+O` |
| `global` | `close_tab` | Stäng flik | `Ctrl+W` |
| `global` | `next_tab` | Nästa flik | `Ctrl+PageDown` |
| `global` | `prev_tab` | Föregående flik | `Ctrl+PageUp` |
| `global` | `help` | Hjälp | `?` |
| `global` | `quit` | Avsluta | `q` |
| `global` | `refresh` | Uppdatera | `r` |
| `global` | `refresh_all` | Uppdatera (fetch, pull requests, ärenden) | `F5` |
| `global` | `find_in_log` | Sök i commit-grafen | `f` |
| `global` | `workspace_search` | Sök i arbetsytan | `Ctrl+Alt+F` |
| `global` | `select_tab` | Välj flik | `t` |
| `global` | `undo` | Ångra senaste operationen | `Ctrl+Z` |
| `global` | `redo` | Gör om den ångrade operationen | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Öppna ångrahistoriken | `Alt+Z` |
| `global` | `commit_column` | Visa eller dölj commit-kolumnen | `Alt+C` |
| `global` | `undo_offer` | Återställ grenen till där den blockerade åtgärden lämnade den | `Alt+Shift+Z` |
| `global` | `context_menu` | Snabbmeny | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Skriv commit-meddelande | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Commita stagade ändringar nu | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI: Dela upp ändringar i commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI: Förklara ändringar | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash med meddelande | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI: Skriv commit-meddelande | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage alla ändringar | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Förkasta alla ändringar | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage vald fil | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage vald fil | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Öppna länkad pull request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push alla tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch alla remotes | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Ny tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Ny gren | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash ändringar | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick vald commit | `Ctrl+G y` |
| `git_prefix` | `revert` | Revertera vald commit | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Redigera commit-meddelande | `Ctrl+G e` |
| `git_prefix` | `amend` | Ändra vald commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Ändra författare för vald commit | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) till vald commit | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase på vald gren | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Poppa stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Ta bort vald ref eller drop stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Öppna LFS-menyn | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Öppna Flow-menyn | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch (alla objekt) | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checka ut alla filer | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull (objekt) | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push (alla objekt) | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Rensa lokala objekt | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Öppna omskrivningsvyn | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Fäll ihop eller återställ Refs-rutan | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Fäll ihop eller återställ Status-rutan | `Ctrl+N l` |
| `nav_prefix` | `settings` | Öppna inställningar | `Ctrl+N ,` |
| `nav_prefix` | `about` | Öppna Om GitPoulpi | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Öppna Aktivitetsloggen | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Öppna den globala aktivitetsloggen | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Visa bakgrundsjobben | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Visa uppdateringspanelen | `Ctrl+N u` |
| `nav_prefix` | `help` | Växla hjälp | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Välj flik | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Byt tema | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Byt språk | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Byt färgtema för grafen | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Växla radfärgning i grafen | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Förstora Refs-avsnitt | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Förminska Refs-avsnitt | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Förstora commit-meddelandets block | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Förminska commit-meddelandets block | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Bredda Refs-rutan | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Smalna av Refs-rutan | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Bredda Status-rutan | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Smalna av Status-rutan | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent: visa den väntande vyn | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent: tillbaka (eller avfärda den väntande vyn) | `Ctrl+N Backspace` |
| `refs` | `filter` | Filtrera refs efter namn | `/` |
| `refs` | `solo` | Solo för markeringen | `o` |
| `refs` | `clear_solo` | Sluta solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Växla grenträd | `y` |
| `refs` | `show_all` | Visa alla dolda refs | `Shift+H` |
| `refs` | `reveal` | Visa i grafen | `g` |
| `refs` | `mark_drop` | Markera som släppkälla | `m` |
| `refs` | `drop_marked` | Släpp markerad ref här | `Shift+M` |
| `refs` | `expand_all` | Fäll ut alla mappar | `=` |
| `refs` | `collapse_all` | Fäll ihop alla mappar | `-` |
| `refs` | `expand_below` | Fäll ut mappen och dess undermappar | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Fäll ihop mappen och dess undermappar | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Filer: sökväg eller träd | `m` |
| `log` | `view_all_files` | Filer: visa alla filer | `a` |
| `log` | `compare_selection` | Jämför de markerade raderna | `v` |
| `log` | `compare_swap` | Byt plats på jämförelsens ändar | `x` |
| `log` | `compare_next` | Gå till nästa jämförda rad | `n` |
| `log` | `pick_column` | Välj en kolumn att flytta eller ändra bredd på | `Shift+C` |
| `log` | `scroll_left` | Rulla kolumnerna åt vänster | `Shift+Left` |
| `log` | `scroll_right` | Rulla kolumnerna åt höger | `Shift+Right` |
| `log` | `copy_mermaid` | Kopiera commits som Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Välj nästa kolumn | `Tab` |
| `log_columns` | `prev` | Välj föregående kolumn | `Shift+BackTab` |
| `log_columns` | `move_left` | Flytta kolumnen åt vänster | `<` |
| `log_columns` | `move_right` | Flytta kolumnen åt höger | `>` |
| `log_columns` | `wider` | Bredda kolumnen | `]` |
| `log_columns` | `narrower` | Smalna av kolumnen | `[` |
| `log_columns` | `reset_width` | Återställ kolumnernas bredder | `0` |
| `log_columns` | `fit` | Anpassa kolumnen efter innehållet | `=` |
| `log_columns` | `fit_all` | Anpassa alla kolumner efter innehållet | `+` |
| `log_columns` | `layout` | Rulla i sidled eller anpassa efter rutan | `s` |
| `log_files` | `collapse_all` | Fäll ihop eller ut alla mappar | `z` |
| `log_files` | `filter` | Filtrera de ändrade filerna | `/` |
| `log_files` | `expand_all` | Fäll ut alla mappar | `=` |
| `log_files` | `collapse_all_folders` | Fäll ihop alla mappar | `-` |
| `log_files` | `expand_below` | Fäll ut mappen och dess undermappar | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Fäll ihop mappen och dess undermappar | `_`, `Shift+Left` |
| `status` | `compose` | Fokusera commit-meddelandet | `c` |
| `status` | `toggle_file_mode` | Filer: sökväg eller träd | `m` |
| `status` | `view_all_files` | Filer: visa alla filer | `a` |
| `status` | `amend` | Växla amend | `Shift+A` |
| `status` | `skip_hooks` | Växla att hoppa över hooks | `Shift+H` |
| `status` | `collapse_all` | Fäll ihop eller ut alla mappar | `z` |
| `status` | `stage_toggle` | Stage eller unstage vald rad | `Ctrl+Space` |
| `status` | `open_diff` | Öppna vald fils diff | `d` |
| `status` | `expand_all` | Fäll ut alla mappar | `=` |
| `status` | `collapse_all_folders` | Fäll ihop alla mappar | `-` |
| `status` | `expand_below` | Fäll ut mappen och dess undermappar | `*`, `Shift+Right` |
| `status` | `collapse_below` | Fäll ihop mappen och dess undermappar | `_`, `Shift+Left` |
| `home` | `favorite` | Stjärnmarkera repositoryt | `f`, `*` |
| `home` | `close_repo` | Stäng, ta bort eller ta bort stjärnan på raden | `x`, `Delete` |
| `home` | `browse` | Bläddra efter ett repository | `b` |
| `home` | `clone` | Klona ett repository | `c` |
| `home` | `init` | Initiera ett repository | `i` |
| `home` | `collapse_all` | Fäll ihop alla sektioner | `-` |
| `home` | `expand_all` | Fäll ut alla sektioner | `+`, `=` |
| `home` | `search` | Sök i repositoryna | `/` |
| `home` | `workspace_new` | Ny arbetsyta | `n` |
| `home` | `workspace_edit` | Redigera arbetsytan | `e` |
| `home` | `workspace_from_tabs` | Ny arbetsyta från öppna flikar | `a` |
| `home` | `workspace_fetch` | Fetch alla i arbetsytan | `Shift+F` |
| `home` | `workspace_pull` | Pull alla i arbetsytan | `Shift+P` |
| `home` | `workspace_clone_missing` | Klona de saknade repositories | `m` |
| `home` | `workspace_locate` | Leta upp det saknade repositoryt | `l` |
| `refs` | `hosting_filter` | Pull requests / ärenden: välj filter | `v` |
| `refs` | `hosting_edit` | Pull request: redigera på värden | `e` |
| `refs` | `new_pull_request` | Pull requests: skapa en | `n` |
| `refs` | `add_remote` | Remotes: lägg till en remote | `+` |
| `diff` | `close` | Stäng diff | `q` |
| `diff` | `edit` | Redigera i arbetskatalogen | `e` |
| `diff` | `lfs_raw` | LFS: kort eller rå pekare | `r` |
| `diff` | `copy` | Kopiera markeringen eller raden | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage eller unstage hunken | `Space` |
| `diff` | `stage_lines` | Stage eller unstage valda rader | `Enter` |
| `diff` | `discard_hunk` | Förkasta hunken | `x` |
| `diff` | `discard_lines` | Förkasta valda rader | `Shift+X` |
| `diff` | `select_lines` | Starta eller släpp en radmarkering | `v` |
| `diff` | `next_hunk` | Nästa hunk | `n` |
| `diff` | `prev_hunk` | Föregående hunk | `p` |
| `diff` | `toggle_layout` | Layout: enhetlig eller sida vid sida | `m` |
| `diff` | `toggle_hunks_only` | Bara hunks eller hela filen | `Shift+H` |
| `diff` | `toggle_minimap` | Växla minikartan | `Shift+M` |
| `diff` | `choose_encoding` | Välj teckenkodning | `Shift+C` |
| `diff` | `toggle_wrap` | Växla radbrytning | `w` |
| `diff` | `toggle_whitespace` | Visa eller dölj ändringar som bara rör blanksteg | `s` |
| `diff` | `merge_editor` | Öppna merge-redigeraren (fil med konflikt) | `Shift+E` |
| `file_insight` | `close` | Stäng filvyn | `q` |
| `file_insight` | `edit` | Redigera i arbetskatalogen | `e` |
| `file_insight` | `copy` | Kopiera markeringen eller raden | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Hoppa till sista raden | `Shift+G` |
| `file_insight` | `open_commit` | Gå till radens commit | `Enter` |
| `history` | `show_in_log` | Visa i Log | `Enter` |
| `history` | `copy_sha` | Kopiera SHA för commit | `y` |
| `history` | `copy_summary` | Kopiera sammanfattningen | `Shift+Y` |
| `history` | `show_diff` | Visa diffen för denna commit | `d`, `Alt+1` |
| `history` | `open_file` | Öppna filen vid denna commit | `o`, `Alt+2` |
| `history` | `blame` | Blame vid denna commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Jämför med arbetskatalogen | `c` |
| `history` | `restore` | Återställ den här versionen | `Shift+R` |
| `history` | `range_up` | Utvidga markeringen uppåt | `Shift+Up` |
| `history` | `range_down` | Utvidga markeringen nedåt | `Shift+Down` |
| `history` | `switch_focus` | Växla mellan listan och diffen | `Tab` |
| `history` | `widen_list` | Bredda listan | `Alt+Right` |
| `history` | `narrow_list` | Smalna av listan | `Alt+Left` |
| `history` | `reset_split` | Återställ listans bredd | `Alt+0` |
| `commit_column` | `scroll_left` | Rulla kolumnen åt vänster | `Shift+Left` |
| `commit_column` | `scroll_right` | Rulla kolumnen åt höger | `Shift+Right` |
| `commit_column` | `next_part` | Välj nästa del | `Tab` |
| `commit_column` | `move_left` | Flytta delen åt vänster | `<` |
| `commit_column` | `move_right` | Flytta delen åt höger | `>` |
| `commit_column` | `wider_part` | Bredda delen | `]` |
| `commit_column` | `narrower_part` | Smalna av delen | `[` |
| `commit_column` | `reset_widths` | Återställ delarnas bredder | `0` |
| `file_insight` | `blame_card` | Blame: Visa eller dölj detaljkortet | `i` |
| `file_insight` | `blame_next` | Blame: Nästa hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Föregående hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Nästa hunk från samma commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Föregående hunk från samma commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Gå till rad | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Kopiera commit-SHA | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Diff för commit i filen | `d` |
| `file_insight` | `blame_parent` | Blame: Blame på commitens förälder | `b` |
| `file_insight` | `blame_history` | Blame: Öppna filhistoriken | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Bredda annoteringskolumnen | `>` |
| `file_insight` | `blame_narrower` | Blame: Smalna av annoteringskolumnen | `<` |
| `image` | `zoom_in` | Zooma in bilden | `+`, `=` |
| `image` | `zoom_out` | Zooma ut bilden | `-` |
| `image` | `fit` | Anpassa bilden till vyn | `0` |
| `image` | `actual_size` | Visa bilden i verklig storlek | `1` |
| `image` | `flip_side` | Visa andra sidan (före / efter) | `Tab` |
| `image` | `widen_before` | Bredda Före-panelen | `Alt+Right` |
| `image` | `widen_after` | Bredda Efter-panelen | `Alt+Left` |
| `image` | `reset_split` | Återställ före/efter-delningen | `Alt+0` |
| `image` | `edit_in` | Öppna med... | `Shift+O` |
| `image` | `edit_default` | Öppna i standardappen | `o` |
| `editor` | `save` | Spara | `Ctrl+S` |
| `editor` | `undo` | Ångra | `Ctrl+Z` |
| `editor` | `redo` | Gör om | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Sök | `Ctrl+F` |
| `editor` | `find_next` | Sök nästa (Shift: föregående) | `F3` |
| `editor` | `paste` | Klistra in | `Ctrl+V` |
| `editor` | `toggle_minimap` | Växla minikartan | `Alt+M` |
| `editor_prompt` | `save` | Osparade ändringar: spara och stäng | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Osparade ändringar: förkasta och stäng | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Osparade ändringar: fortsätt redigera | `c`, `Shift+C` |
| `merge` | `next_conflict` | Nästa konflikt | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Föregående konflikt | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Nästa fil med konflikt | `Alt+PageDown` |
| `merge` | `prev_file` | Föregående fil med konflikt | `Alt+PageUp` |
| `merge` | `next_pane` | Fokusera nästa ruta | `F6` |
| `merge` | `prev_pane` | Fokusera föregående ruta | `Shift+F6` |
| `merge` | `toggle_panes` | Visa eller dölj sidopanelerna (Refs och Status) | `F9` |
| `merge` | `toggle_base` | Visa eller dölj Bas-rutan | `Alt+D` |
| `merge` | `toggle_full_file` | Växla diff av hela filen | `Alt+F` |
| `merge` | `undo` | Ångra (val och inmatning) | `Ctrl+Z` |
| `merge` | `redo` | Gör om (val och inmatning) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Bredda rutan Nuvarande (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Bredda rutan Inkommande (B) | `Alt+Left` |
| `merge` | `reset_panes` | Återställ rutstorlekarna | `Alt+0` |
| `merge` | `save` | Spara utan att markera löst | `Alt+S` |
| `merge` | `save_resolve` | Spara och markera löst | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Föreslå en lösning (AI) | `Alt+I` |
| `merge` | `unpick` | Ångra valet för den här konflikten (tillbaka till olöst) | `Alt+U` |
| `merge` | `reset_all` | Återställ hela filen till de ursprungliga konflikterna | `Alt+R` |
| `merge` | `focus_a` | Fokusera rutan Nuvarande (A) | `Alt+1` |
| `merge` | `focus_b` | Fokusera rutan Inkommande (B) | `Alt+2` |
| `merge` | `focus_output` | Fokusera Resultat-rutan | `Alt+3` |
| `merge` | `focus_base` | Fokusera Bas-rutan | `Alt+4` |
| `merge_sides` | `toggle_line` | Välj eller släpp raden | `Space` |
| `merge_sides` | `toggle_chunk` | Välj eller släpp hela blocket | `Enter` |
| `merge_sides` | `take_a` | Ta bara Nuvarande (A) | `a` |
| `merge_sides` | `take_b` | Ta bara Inkommande (B) | `b` |
| `merge_sides` | `take_both` | Ta båda, A sedan B | `c` |
| `merge_sides` | `take_both_reversed` | Ta båda, B sedan A | `Shift+C` |
| `merge_sides` | `clear` | Ta ingenting för den här konflikten | `x` |
| `merge_sides` | `unpick` | Ångra valet för den här konflikten (tillbaka till olöst) | `u` |
| `merge_sides` | `reset_all` | Återställ hela filen till de ursprungliga konflikterna | `Shift+U` |
| `merge_sides` | `swap_order` | Byt ordning på de två sidorna | `s` |
| `merge_sides` | `next_conflict` | Nästa konflikt | `n` |
| `merge_sides` | `prev_conflict` | Föregående konflikt | `Shift+N` |
| `merge_sides` | `take_all_a` | Ta allt från Nuvarande (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Ta allt från Inkommande (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Visa eller dölj Bas-rutan | `d` |
| `merge_sides` | `scroll_left` | Rulla rutorna åt vänster | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Rulla rutorna åt höger | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Växla diff av hela filen | `f` |
| `merge_sides` | `cycle_height` | Växla höjd på Resultat | `z` |
| `merge_card` | `take_a` | Behåll A (nuvarande) som den är | `a` |
| `merge_card` | `take_b` | Ta B (inkommande) som den är | `b` |
| `merge_card` | `take_base` | Ta Bas (gemensam förfader) | `o` |
| `merge_card` | `keep` | Behåll filen som finns kvar | `k` |
| `merge_card` | `delete` | Godta borttagningen | `Shift+D` |
| `merge_card` | `external_tool` | Öppna i externt merge-verktyg | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (stanna för att ändra) | `e` |
| `rebase` | `autosquash` | Växla autosquash | `a` |
| `rebase` | `start` | Starta planen | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Återställ planen | `Ctrl+R` |
| `rebase` | `split` | Dela upp radens commit | `t` |
| `rebase` | `move_up` | Flytta raden upp | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Flytta raden ned | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Skapa commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Växla radmarkering | `v` |
| `split` | `move_up` | Flytta commit upp | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Flytta commit ned | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Återställ uppdelningen | `Shift+R` |
| `rewrite` | `preview` | Förhandsgranska omskrivningen | `F5` |
| `rewrite` | `apply` | Tillämpa omskrivningen | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Avbryt förhandsgranskningen | `Ctrl+X` |
| `rewrite` | `add_rule` | Lägg till en regel | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Slå på eller av regeln | `Space` |
| `rewrite` | `move_up` | Flytta regeln uppåt | `Alt+Up` |
| `rewrite` | `move_down` | Flytta regeln nedåt | `Alt+Down` |
| `rewrite` | `delete_rule` | Ta bort regeln | `d`, `Delete` |
| `rewrite` | `restore_rule` | Återställ den borttagna regeln | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplicera regeln | `Ctrl+D` |
| `rewrite` | `next_tab` | Nästa förhandsgranskningsflik | `Alt+Right` |
| `rewrite` | `prev_tab` | Föregående förhandsgranskningsflik | `Alt+Left` |
| `rewrite` | `save_shared` | Spara receptet i repositoryt | `Alt+S` |
| `rewrite` | `save_mine` | Spara receptet bara för mig | `Alt+M` |
| `rewrite` | `load` | Läs in ett recept | `Alt+L` |
| `rewrite` | `name` | Byt namn på receptet | `F2` |
| `rewrite` | `protected_all` | Ta med alla skyddade grenar | `Alt+A` |
| `rewrite` | `protected_none` | Utelämna alla skyddade grenar | `Alt+U` |
| `rewrite` | `check_tools` | Kontrollera verktygen igen | `F6` |
| `rewrite` | `analyze` | Analysera historikens storlek | `F7` |
| `rewrite` | `host_howto` | Förklara värdens grenskydd | `Alt+H` |
| `rewrite` | `ai_rules` | Skapa omskrivningsregler med AI | `Alt+I` |
| `reword` | `apply` | Använd det nya meddelandet | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Ändra historiken djupare | `Alt+H` |
| `composer` | `commit` | Commit från composern | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Infoga en ärendereferens | `Alt+R` |
| `dialog` | `browse` | Bläddra efter en mapp | `Ctrl+B` |
| `dialog` | `sources` | Gå till klonkällorna | `Alt+S` |
| `dialog` | `submit` | Skicka formuläret för pull request | `Ctrl+S` |
| `dialog` | `web` | Öppna leverantörens webbformulär | `Alt+W` |
| `folder_picker` | `choose` | Välj mappen | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Ny mapp | `Alt+N` |
| `folder_picker` | `filter` | Filtrera mappens poster | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Visa eller dölj syskonmappar | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Visa eller dölj dolda mappar | `Alt+H` |
| `preferences` | `search` | Sök i inställningarna | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Lägg till ett listobjekt | `Insert` |
| `preferences` | `list_duplicate` | Duplicera det valda listobjektet | `Ctrl+D` |
| `preferences` | `list_remove` | Ta bort det valda listobjektet | `Delete` |
| `preferences` | `list_move_up` | Flytta det valda listobjektet upp | `Alt+Up` |
| `preferences` | `list_move_down` | Flytta det valda listobjektet ned | `Alt+Down` |
| `preferences` | `prev_tab` | Föregående flik på inställningssidan | `Alt+Left` |
| `preferences` | `next_tab` | Nästa flik på inställningssidan | `Alt+Right` |
| `ai_popup` | `copy` | Kopiera resultatet | `c` |
| `activity_log` | `scroll_left` | Rulla Aktivitetsloggen åt vänster | `Left` |
| `activity_log` | `scroll_right` | Rulla Aktivitetsloggen åt höger | `Right` |
| `integrations` | `add` | Lägg till en profil | `a` |
| `integrations` | `edit` | Redigera vald profil | `e` |
| `integrations` | `open_token` | Öppna tokensidan | `o` |
| `integrations` | `remove` | Ta bort vald profil | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI: Generera stash-meddelande | `Ctrl+G` |
<!-- keymap:end -->

### Tangenter som terminalen inte kan skicka, och kitty-tangentbordsprotokollet

En äldre terminal kan inte skicka vissa kombinationer: `Ctrl+,`,
`Ctrl+Shift+N`, `Ctrl+Enter` och `Ctrl+<siffra>`. Den skickar ingenting, eller
en annan tangent: till exempel kommer `Ctrl+Shift+N` fram som `Ctrl+N`,
`Ctrl+I` som `Tab` och `Ctrl+M` som `Enter`. Sidan Inställningar varnar för en
sådan bindning (”den här terminalen skickar inte ...”).

Terminaler med **kitty-tangentbordsprotokollet** kan skicka dessa kombinationer.
Exempel: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 och senare, samt tmux
eller herdr när de släpper igenom protokollet. GitPoulpi använder protokollet
som standard:

- Vid start frågar GitPoulpi terminalen och väntar högst cirka 100 ms på svaret.
  Om terminalen inte svarar finns de äldre tangenterna kvar.
- Om terminalen svarar slår GitPoulpi på protokollet. Det slås av igen när
  programmet avslutas, misslyckas eller kraschar.
- Varningarna på sidan Inställningar följer protokollet: när det är på går
  kombinationerna ovan att skicka.
- **Inställningar > Tangentbord & mus > Kitty-tangentbordsprotokoll**
  (`ui.kitty_keyboard`) satt till `off` ändrar aldrig terminalens
  tangentbordsläge. Använd det om en terminal eller en multiplexer beter sig
  fel. En omstart behövs.

Windows-konsoler rapporterar modifierartangenterna själva och behöver inte
protokollet.

## Kommandopalett (`Ctrl+P`)

Paletten listar **varje** kommando i appen, så den når allt utan en genväg.
Hela listan finns [nedan](#varje-palettkommando).

| Tangent | Åtgärd |
|---|---|
| `Ctrl+P` | Öppna paletten |
| *(skriv)* | Filtrera listan. En exakt träff efter prefixet `Kategori: ` kommer först, sedan en träff var som helst i etiketten, sedan en suddig träff |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Flytta markeringen (se [Filtrerbara popuplistor](#filtrerbara-popuplistor)) |
| `Enter`, eller ett dubbelklick | Kör det markerade kommandot |
| `Esc` | Stäng |

`j` och `k` skrivs in i filtret. De flyttar inte markeringen.

Varje rad visar sin genväg till höger. Ett kommando som inte gäller just nu
(till exempel ingen ref vald) finns kvar i listan, nedtonat, och orsaken visas
under listan. Etiketterna börjar med en kategori, som `Navigering:`, `Refs:`,
`Git:`, `Vy:`, `Diff:` eller `App:`. Skriv en kategori för att se hela gruppen.

### En tangent för vilket kommando som helst

Varje rad i paletten slutar med en tangentknapp (`[k]`), och `Alt+K` gör detsamma på den markerade raden: den öppnar **Inställningar > Tangentbord & mus** filtrerat på kommandots genväg, redo att ändras. Ett kommando utan egen genväg får en här: det står under **Palettkommandon**, utan tangent som standard, och en tangent du ger det kör kommandot precis som när du väljer det i paletten (ett kommando som inte kan köras nu anger varför). Raden visar sedan din tangent.

I `config.toml` är det tabellen `[keybindings.palette]`, uppkallad efter kommandot (`toggle_minimap = "Ctrl+Alt+M"`). En tangent som kolliderar med en annan åtgärd visas i rött, och `c` på dess rad listar alla åtgärder den kolliderar med.

Hjälpraden under listan anger den här tangenten (`Alt+K` som standard). Det är åtgärden `palette.configure_shortcut`, så den kan ändras som alla andra.

### Varje palettkommando

Genereras från appen (ett test håller den aktuell). AI-kommandon visas bara
när AI är på; de andra är nedtonade, med orsaken, när de inte gäller.
Tangenterna är standardvärdena.

<!-- palette:begin -->
| Kommando | Standardtangent |
|---|---|
| Navigering: Fokusera Refs-rutan | `Ctrl+N <badge>` |
| Navigering: Fokusera Log-rutan | `Ctrl+N <badge>` |
| Navigering: Fokusera Status-rutan | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs-filtret | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs > Lokala | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs > Remote | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs > Worktrees | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs > Submoduler | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs > Stashes | `Ctrl+N <badge>` |
| Navigering: Fokusera Refs > Tags | `Ctrl+N <badge>` |
| Sidopanel: Visa/dölj avsnitt... |  |
| Sidopanel: Återställ avsnitten till standard |  |
| Sidopanelen: Använd dessa avsnitt för alla repositories |  |
| Sidopanel: Visa/dölj lokala grenar |  |
| Sidopanel: Visa/dölj remotes |  |
| Sidopanel: Visa/dölj worktrees |  |
| Sidopanel: Visa/dölj stashes |  |
| Sidopanel: Visa/dölj tags |  |
| Sidopanel: Visa/dölj pull requests |  |
| Sidopanel: Visa/dölj ärenden |  |
| Sidopanel: Visa/dölj submoduler |  |
| Navigering: Fokusera Status > Unstaged | `Ctrl+N <badge>` |
| Navigering: Fokusera Status > Staged | `Ctrl+N <badge>` |
| Navigering: Flytta fokus till nästa ruta | `Tab` |
| Navigering: Flytta fokus till föregående ruta | `Shift+Tab` |
| Navigering: Sök commits... | `f` |
| Navigering: Fäll ihop/ut Refs-rutan | `Ctrl+N h` |
| Navigering: Bredda Refs-rutan | `Ctrl+N Right` |
| Navigering: Smalna av Refs-rutan | `Ctrl+N Left` |
| Navigering: Bredda Status-rutan | `Ctrl+N Shift+Left` |
| Navigering: Smalna av Status-rutan | `Ctrl+N Shift+Right` |
| Navigering: Fäll ihop/ut Status-rutan | `Ctrl+N l` |
| Navigering: Förstora Refs-avsnitt | `Ctrl+N +` |
| Navigering: Förminska Refs-avsnitt | `Ctrl+N -` |
| Navigering: Återställ Refs-avsnittens storlek |  |
| Navigering: Återställ layouten |  |
| Commit-detaljer: Förstora meddelandet | `Ctrl+N ]` |
| Commit-detaljer: Förminska meddelandet | `Ctrl+N [` |
| Commit-detaljer: Återställ meddelandets storlek |  |
| Commit-detaljer: Rulla meddelandet ned |  |
| Commit-detaljer: Rulla meddelandet upp |  |
| Filer: Växla sortering av commit-filer (A-Ö / Ö-A) |  |
| Refs: Checka ut vald ref | `dubbelklick` |
| Refs: Kopiera refnamn |  |
| Refs: Kopiera fullständig ref (refs/heads/...) |  |
| Refs: Kopiera refens mål-SHA |  |
| Refs: Merge vald gren i aktuell |  |
| Refs: Rebase aktuell gren på vald | `Ctrl+G r` |
| Refs: Släpp vald ref på utcheckad gren... |  |
| Refs: Markera som släppkälla | `m` |
| Refs: Släpp markerad ref här... | `M` |
| Refs: Byt namn på vald gren... |  |
| Refs: Fast-forward vald gren till aktuell |  |
| Refs: Fast-forward aktuell gren till vald |  |
| Refs: Skapa worktree från vald gren eller tag... |  |
| Refs: Reflog för vald gren... |  |
| Refs: Starta en pull request från vald gren... |  |
| Refs: Öppna worktree i flik | `Enter / dubbelklick` |
| Refs: Uppdatera submodul (vald) |  |
| Refs: Initiera submodul (vald) |  |
| Refs: Öppna submodul i flik | `Enter / dubbelklick` |
| Git: Uppdatera alla submoduler |  |
| Git: Skapa worktree vid vald commit... |  |
| Git: Skapa patch från vald commit... |  |
| Git: Skapa patch från vald commit till HEAD... |  |
| Git: Skapa patch från ändringar i arbetskatalogen... |  |
| Omskrivning: Rensa gammal historik... |  |
| Git: Tillämpa patch... |  |
| Omskrivning: Öppna omskrivningsvyn | `Ctrl+G H` |
| Omskrivning: Byt ut en författare... |  |
| Omskrivning: Anonymisera författare... |  |
| Omskrivning: Tillämpa .mailmap på historiken... |  |
| Omskrivning: Merge av identiteter (.mailmap)... |  |
| Omskrivning: Förskjut datum för commits... |  |
| Omskrivning: Byt tidszon för commits... |  |
| Omskrivning: Begränsa commits till vissa timmar... |  |
| Omskrivning: Sätt committerdatum till författardatum |  |
| Omskrivning: Sök och ersätt i meddelanden... |  |
| Omskrivning: Ta bort meddelanderader... |  |
| Omskrivning: Lägg till eller ta bort trailers... |  |
| Omskrivning: Ta bort signaturer... |  |
| Omskrivning: Rätta meddelandenas teckenkodning... |  |
| Omskrivning: Sök och ersätt i filer... |  |
| Omskrivning: Ta bort filer ur historiken... |  |
| Omskrivning: Ta bort stora filer ur historiken... |  |
| Omskrivning: Flytta eller byt namn på sökvägar... |  |
| Omskrivning: Migrera filer till LFS... |  |
| Omskrivning: Ta tillbaka filer från LFS... |  |
| Omskrivning: Konvertera radslut... |  |
| Omskrivning: Ta bort körbehörigheten... |  |
| Omskrivning: Konvertera filernas teckenkodning... |  |
| Omskrivning: Kör ett skript... |  |
| ✦ Omskrivning: Skriv ett skript med AI... |  |
| ✦ Omskrivning: Skapa regler med AI... |  |
| Omskrivning: Från vald commit till HEAD... |  |
| Omskrivning: Valda commits... |  |
| LFS: Initiera för det här repositoryt |  |
| LFS: Spåra mönster... |  |
| LFS: Sluta spåra mönster... |  |
| LFS: Fetch objekt |  |
| LFS: Öppna menyn | `Ctrl+G l` |
| LFS: Fetch (alla objekt) | `Ctrl+G F` |
| LFS: Checka ut alla filer | `Ctrl+G K` |
| LFS: Push (alla objekt) | `Ctrl+G J` |
| LFS: Pulla objekt | `Ctrl+G L` |
| LFS: Pusha objekt |  |
| LFS: Rensa lokala objekt | `Ctrl+G V` |
| LFS: Visa lås |  |
| LFS: Öppna installationssidan |  |
| LFS: Växla rå pekare i diff | `r` |
| LFS: Spåra vald fil |  |
| LFS: Spåra vald fils filändelse |  |
| LFS: Lås vald fil |  |
| LFS: Lås upp vald fil |  |
| LFS: Tvinga upplåsning av vald fil |  |
| Refs: Ta bort vald gren eller tag... | `Ctrl+G d` |
| Refs: Ta bort vald gren från dess remote... |  |
| Refs: Ta bort vald gren lokalt och på dess remote... |  |
| Refs: Kopiera länk till vald gren på remote |  |
| Git: Kopiera länk till vald commit på remote |  |
| Git: Annoterad tag vid vald commit... |  |
| Refs: Pusha vald tag till standard-remote |  |
| Refs: Ta bort vald tag från dess remote... |  |
| Git: Fetch vald tag |  |
| Refs: Annotera vald tag... |  |
| Refs: Växla dölj i grafen |  |
| Refs: Visa i grafen | `g` |
| Refs: Växla solo för vald ref | `o` |
| Refs: Sluta solo (rensa alla solo-refs) | `O` |
| Refs: Visa alla dolda refs | `H` |
| Refs: Växla dölj för vald mapp/remote |  |
| Refs: Växla solo för vald mapp/remote | `o` |
| Refs: Filtrera efter namn... | `/` |
| Refs: Rensa namnfilter | `Esc` |
| Refs: Fäll ut alla mappar | `=` |
| Refs: Fäll ihop alla mappar | `-` |
| Refs: Fäll ut vald mapp och dess undermappar | `*` |
| Refs: Fäll ihop vald mapp och dess undermappar | `_` |
| Refs: Nästa avsnitt | `l` |
| Refs: Föregående avsnitt | `h` |
| Refs: Fäll ut/ihop aktivt avsnitt | `Enter` |
| Refs: Växla grenträd | `y` |
| Git: Stage vald fil | `Ctrl+G s` |
| Git: Unstage vald fil | `Ctrl+G u` |
| Git: Stage alla ändringar | `Ctrl+G a` |
| Lokala ändringar: Förkasta vald fil... |  |
| Lokala ändringar: Ignorera vald fil |  |
| Lokala ändringar: Ignorera vald fils filändelse |  |
| Lokala ändringar: Behandla vald fil som binär... |  |
| Lokala ändringar: Behandla vald fil som text... |  |
| Lokala ändringar: Behandla vald fils filändelse som binär... |  |
| Lokala ändringar: Behandla vald fils filändelse som text... |  |
| Lokala ändringar: Stash vald fil |  |
| Lokala ändringar: Vald fils historik |  |
| Lokala ändringar: Vald fils blame |  |
| Lokala ändringar: Öppna vald fil i externt diff-verktyg |  |
| Lokala ändringar: Öppna vald fil i extern redigerare |  |
| Lokala ändringar: Öppna vald fil i standardprogram |  |
| Lokala ändringar: Visa vald fil i mapp |  |
| Lokala ändringar: Kopiera vald fils sökväg |  |
| Lokala ändringar: Skapa patch från vald fil... |  |
| Lokala ändringar: Redigera vald fil |  |
| Lokala ändringar: Ta bort vald fil... |  |
| Lokala ändringar: Öppna vald mapps meny |  |
| Lokala ändringar: Skapa fil |  |
| Lokala ändringar: Förkasta alla ändringar... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Commita stagade ändringar nu | `Ctrl+G Enter` |
| Git: Redigera WIP-commit-meddelande (i Log) | `c i Log` |
| Git: Fetch alla remotes | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward om möjligt) |  |
| Git: Pull (bara fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Pull-alternativ... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push till remote... |  |
| Git: Force push (med lease)... |  |
| Git: Ange upstream... |  |
| Remote: Lägg till remote... |  |
| Remote: Ta bort remote... |  |
| Remote: Redigera remote... |  |
| Remote: Ange standard-remote... |  |
| Remote: Kopiera remotens namn |  |
| Remote: Kopiera länk till remote |  |
| Remote: Fetch remote... |  |
| Remote: Åtgärder för vald remote... | `.` |
| Repository: SSH-nyckel för det här repositoryt... |  |
| Repository: Delvis utcheckning... |  |
| Repository: SSH & Credentials... |  |
| Repository: Test Connection |  |
| Repository: Use Default Credentials |  |
| Flow: Öppna menyn | `Ctrl+G W` |
| Flow: Starta gren... |  |
| Flow: Nytt ärende... |  |
| Flow: Plocka upp gren från hosten... |  |
| Flow: Slutför aktuell gren... |  |
| Flow: Synka med föräldern... |  |
| Flow: Promota... |  |
| Flow: Ange förälder... |  |
| Flow: Fortsätt eller avbryt stoppad åtgärd... |  |
| Flow: Växla grupperade grenar |  |
| Flow: Arbetsflödesinställningar... |  |
| Remote: Dölj remote i grafen... |  |
| Remote: Visa remote i grafen... |  |
| Git: Ny gren vid HEAD... | `Ctrl+G b` |
| Git: Ny tag vid HEAD... | `Ctrl+G t` |
| Git: Pusha alla opushade taggar | `Ctrl+G U` |
| Git: Stash alla ändringar | `Ctrl+G T` |
| Git: Stash alla ändringar med meddelande... | `Ctrl+G m` |
| Git: Stash pop | `Ctrl+G o` |
| Git: Stash apply |  |
| Git: Stash drop... |  |
| Git: Cherry-pick vald commit på HEAD | `Ctrl+G y` |
| Git: Checka ut vald commit (frånkopplad HEAD) |  |
| Git: Skapa gren vid vald commit... |  |
| Git: Skapa tag vid vald commit... |  |
| Commit: Kopiera meddelande |  |
| Commit: Kopiera författarens namn |  |
| Commit: Kopiera författarens e-post |  |
| Commit: Kopiera datum |  |
| Commit: Kopiera kort SHA |  |
| Commit: Kopiera fullständig SHA |  |
| Git: Revertera vald commit... | `Ctrl+G v` |
| Git: Redigera commit-meddelande... | `Ctrl+G e` |
| Commit: Ändra... | `Ctrl+G E` |
| Commit: Ändra författare... | `Ctrl+G A` |
| Commit: Ändra sammanfattning... |  |
| Commit: Ändra beskrivning... |  |
| Git: Dela upp vald commit... |  |
| Git: Interaktiv rebase från vald commit... |  |
| Rebase: Fortsätt |  |
| Rebase: Hoppa över |  |
| Rebase: Avbryt |  |
| git am: Fortsätt |  |
| git am: Hoppa över den här patchen |  |
| git am: Avbryt |  |
| Repository: Ta bort föråldrad index.lock |  |
| Banner: Pop väntande auto-stash |  |
| Banner: Slutför väntande auto-stash (behåll resultatet) |  |
| Banner: Visa väntande auto-stash |  |
| Konflikter: Visa förutsedda konflikter |  |
| Banner: Drop väntande auto-stash... |  |
| Banner: Börja om... |  |
| Git: Reset gren till vald commit (soft - behåll index och arbetskatalog) |  |
| Git: Reset gren till vald commit (mixed - behåll arbetskatalog) | `Ctrl+G R` |
| Git: Reset gren till vald commit (hard - förkasta allt)... |  |
| Git: Lös konflikt med Ours (den här grenen) |  |
| Git: Lös konflikt med Theirs (inkommande) |  |
| Git: Lös konflikt med Bas (gemensam förfader) |  |
| Konflikter: Öppna merge-redigeraren | `Enter på en fil med konflikt` |
| Merge: Nästa konflikt | `F7` |
| Merge: Föregående konflikt | `Shift+F7` |
| Merge: Nästa fil med konflikt | `Alt+PageDown` |
| Merge: Föregående fil med konflikt | `Alt+PageUp` |
| Merge: Ta block från A |  |
| Merge: Ta block från B |  |
| Merge: Ta Nuvarande för den här konflikten | `a` |
| Merge: Ta Inkommande för den här konflikten | `b` |
| Merge: Ta båda, A sedan B | `c` |
| Merge: Ta båda, B sedan A | `C` |
| Merge: Ta Bas för den här konflikten |  |
| Merge: Töm den här konflikten | `x` |
| Merge: Byt ordning | `s` |
| Merge: Ta alla Nuvarande | `A` |
| Merge: Ta alla Inkommande | `B` |
| Merge: Redigera konflikten i Resultat |  |
| Merge: Visa/dölj sidopanelerna | `F9` |
| Merge: Rulla rutorna åt vänster | `Shift+Left` |
| Merge: Rulla rutorna åt höger | `Shift+Right` |
| Merge: Växla Bas-rutan | `Alt+D` |
| Merge: Växla diff av hela filen | `Alt+F` |
| Merge: Ångra | `Ctrl+Z` |
| Merge: Gör om | `Ctrl+Y` |
| Merge: Spara utan att markera löst | `Alt+S` |
| Merge: Spara och markera löst | `Ctrl+S` |
| Merge: Återställ Resultat till sammanslagningen | `Alt+R` |
| Merge: Växla Resultats höjd | `z` |
| Merge: Bredda Nuvarande-rutan | `Alt+Right` |
| Merge: Bredda Inkommande-rutan | `Alt+Left` |
| Merge: Återställ rutstorlekar | `Alt+0` |
| Merge: Ångra valet för konflikten | `Alt+U` |
| Merge: Fokusera Nuvarande (A) | `Alt+1` |
| Merge: Fokusera Inkommande (B) | `Alt+2` |
| Merge: Fokusera Resultat | `Alt+3` |
| Merge: Fokusera Bas | `Alt+4` |
| Merge: Gå till rutan till vänster (samma konflikt) |  |
| Merge: Gå till rutan till höger (samma konflikt) |  |
| Merge: Fokusera nästa ruta | `F6` |
| Merge: Fokusera föregående ruta | `Shift+F6` |
| Merge: Fortsätt från filen på disk |  |
| Merge: Börja om från sammanslagningen |  |
| Merge: Ångra lösning och öppna igen |  |
| Merge: Läs om filen |  |
| Merge: Använd Nuvarande fil som den är | `a` |
| Merge: Använd Inkommande fil som den är | `b` |
| Merge: Använd Bas-filen som den är | `o` |
| Merge: Behåll filen som finns kvar | `k` |
| Merge: Godta borttagningen | `D` |
| Merge: Öppna i externt merge-verktyg | `m` |
| ✦ AI: Föreslå lösning på merge-konflikt | `Alt+I` |
| Merge: Stäng merge-redigeraren |  |
| Git: Avbryt merge, cherry-pick eller revert |  |
| Git: Hoppa över cherry-pick eller revert |  |
| Repository: Öppna eller växla (repositoryhantering)... | `Ctrl+O` |
| Repository: Bläddra efter ett repository... | `b (hanteringsflik)` |
| Repository: Skapa nytt (Init)... | `i (hanteringsflik)` |
| Repositories: Öppna valt | `Enter` |
| Repositories: Lägg till/ta bort valt i favoriter | `f` |
| Repositories: Stäng eller ta bort valt | `x` |
| Repositories: Valt repositorys meny... | `.` |
| Repositories: Sök | `/ / Ctrl+F` |
| Repositories: Fäll ihop alla sektioner | `-` |
| Repositories: Fäll ut alla sektioner | `+` |
| Repositories: Stäng alla flikar... |  |
| Repositories: Ta bort valt från listan |  |
| Repositories: Ta bort alla senaste |  |
| Arbetsyta: Ny... | `n` |
| Arbetsyta: Ny från öppna flikar... | `a` |
| Arbetsyta: Byt... | `Alt+W` |
| Arbetsyta: Redigera... | `e` |
| Arbetsyta: Stäng (ingen arbetsyta) |  |
| Arbetsyta: Hantera... | `Alt+J` |
| Arbetsyta: Stäng fliken |  |
| Arbetsyta: Leta upp det saknade repositoryt... | `l` |
| Arbetsyta: Ta bort... |  |
| Arbetsyta: Fetch alla | `F` |
| Arbetsyta: Pull alla | `P` |
| Arbetsyta: Klona de som saknas | `m` |
| Arbetsyta: Lägg till fliken i arbetsytan |  |
| Arbetsyta: Ta bort fliken från arbetsytan |  |
| Radar: Öppna | `Alt+R` |
| Radar: Uppdatera | `r` |
| Radar: Sök | `/` |
| Radar: Öppna i webbläsaren | `o` |
| Radar: Öppna repository | `Enter` |
| Radar: Checka ut gren | `c` |
| Radar: Skapa worktree... | `t` |
| Radar: Visa i repository | `s` |
| Radar: Börja arbeta med ett ärende... | `n` |
| Radar: Fäst eller lossa | `p` |
| Radar: Snooza... | `z` |
| Radar: Granska begäran... | `e` |
| Radar: Granska via API:et |  |
| Radar: Klona och granska... |  |
| Radar: Redigera tilldelade... |  |
| Radar: Redigera granskare... |  |
| Radar: Redigera etiketter... |  |
| Radar: Ange milstolpe... |  |
| Radar: Välj omfång | `w` |
| Radar: Välj vy | `v` |
| Radar: Välj filter | `f` |
| Radar: Visa ej klonade begäranden | `a` |
| Radar: Visa utkast | `d` |
| Radar: Slå på kontoövergripande granskningsbegäranden... | `T` |
| Radar: Radmeny... | `.` |
| Radar: Välj kolumn | `C` |
| Radar: Bredda kolumnen | `]` |
| Radar: Smalna av kolumnen | `[` |
| Radar: Återställ kolumnbredder | `0` |
| Radar: Tillbaka till Hem | `Esc` |
| Radar: Stäng fliken |  |
| Flik: Ny flik | `Alt+T` |
| Flik: Ny... |  |
| Flik: Åtgärdsmeny... | `Ctrl+T` |
| Flik: Stäng övriga flikar... |  |
| Flik: Stäng flikar till höger... |  |
| Flik: Lägg till/ta bort i favoriter |  |
| Flik: Kopiera repositorysökväg |  |
| Flik: Öppna i filhanteraren |  |
| Flik: Öppna i terminal |  |
| Flik: Öppna i... | `Alt+O` |
| Flik: Öppna i standard | `Ctrl+Alt+O` |
| Repository: Byt profil... |  |
| Hjälp: Om GitPoulpi | `Ctrl+N a` |
| Hjälp: Licenser och erkännanden |  |
| Visa: Aktivitetslogg | `Ctrl+N A` |
| Visa: Global aktivitetslogg | `Ctrl+N G` |
| Hjälp: Versionsanteckningar |  |
| Uppdateringar: Visa uppdatering | `Ctrl+N u` |
| Uppdateringar: Öppna säkerhetsmeddelande |  |
| Uppdateringar: Kopiera uppdateringskommando |  |
| Uppdateringar: Kör uppdateringskommando i en terminal |  |
| Uppdateringar: Sök efter uppdateringar nu |  |
| Uppdateringar: Ladda ned |  |
| Uppdateringar: Uppdatera nu |  |
| Uppdateringar: Starta om för att uppdatera |  |
| Uppdateringar: Gå tillbaka till föregående version |  |
| Uppdateringar: Hoppa över den här versionen |  |
| Versionsanteckningar: Stäng fliken |  |
| Jobb: visa bakgrundsjobben | `Ctrl+N J` |
| Jobb: avbryt allt som går att avbryta |  |
| Visa: Kopiera diagnostik |  |
| Aktivitetslogg: visa eller dölj nivå Fel |  |
| Aktivitetslogg: visa eller dölj nivå Varning |  |
| Aktivitetslogg: visa eller dölj nivå Info |  |
| Aktivitetslogg: visa eller dölj nivå Felsökning |  |
| Aktivitetslogg: visa eller dölj nivå Spårning |  |
| Aktivitetslogg: visa alla nivåer |  |
| Aktivitetslogg: visa ingen nivå |  |
| Aktivitetslogg: Rensa programloggen |  |
| Aktivitetslogg: Välj omfång |  |
| Aktivitetslogg: Git-åtgärder |  |
| Aktivitetslogg: Programlogg |  |
| Aktivitetslogg: Rulla åt vänster | `Left` |
| Aktivitetslogg: Rulla åt höger | `Right` |
| Aktivitetslogg: Kopiera det senaste felet |  |
| Aktivitetslogg: Kör den föreslagna åtgärden |  |
| Aktivitetslogg: Öppna det senaste felet |  |
| Flik: Avsluta och cd hit |  |
| Repository: Klona... | `Ctrl+Shift+N` |
| Repository: Nästa flik | `Ctrl+PageDown` |
| Repository: Föregående flik | `Ctrl+PageUp` |
| Flik: Flytta åt vänster |  |
| Flik: Flytta åt höger |  |
| Repository: Stäng flik | `Ctrl+W` |
| Repository: Välj flik... | `t` |
| Sök: Arbetsyta... | `Ctrl+Alt+F` |
| Redigera: Ångra | `Ctrl+Z` |
| Redigera: Gör om | `Ctrl+Y` |
| Redigera: Glöm blockerad ångrapost |  |
| Redigera: Ångrahistorik... | `Alt+Z` |
| Redigera: Reflog... |  |
| Redigera: Rädda förlorade Commits... |  |
| Redigera: Återställ grenen till där den blockerade åtgärden lämnade den | `Alt+Shift+Z` |
| Granskning: Visa översikt |  |
| Granskning: Visa filer |  |
| Granskning: Visa Commits |  |
| Granskning: Kommentera rad... | `c` |
| Granskning: Kommentera konversationen... | `C` |
| Granskning: Svara i tråd... | `r` |
| Granskning: Lös eller öppna tråd igen | `x` |
| Granskning: Fäll ihop eller ut tråd | `z` |
| Granskning: Redigera väntande kommentar... | `e` |
| Granskning: Ta bort väntande kommentar | `d` |
| Granskning: Markera fil som visad | `v` |
| Granskning: Nästa fil | `]` |
| Granskning: Föregående fil | `[` |
| Granskning: Slutför granskning... | `f` |
| Granskning: Förkasta väntande granskning... |  |
| Granskning: Uppdatera | `F5` |
| Granskning: Öppna i webbläsaren | `o` |
| Granskning: Checka ut gren | `b` |
| Granskning: Stäng | `Esc` |
| Granskning: Redigera tilldelade... | `a` |
| Granskning: Redigera granskare... | `p` |
| Granskning: Redigera etiketter... | `l` |
| Granskning: Ange milstolpe... | `m` |
| Granskning: Välj patch set... | `P` |
| Granskning: Jämför patch sets... | `D` |
| Granskning: Skicka in ändring... | `U` |
| Granskning: Överge ändring... | `Q` |
| Granskning: Växla mellan inline och delad vy | `V` |
| Granskning: Växla mellan hela filen och bara hunks | `H` |
| Granskning: Växla ändringar med bara blanksteg | `S` |
| Granskning: Växla radbrytning | `W` |
| Granskning: Nästa hunk | `J` |
| Granskning: Föregående hunk | `K` |
| Granskning: Välj teckenkodning... | `E` |
| Granskning: Växla minikarta | `M` |
| Granskning: Rulla diff åt vänster | `Left` |
| Granskning: Rulla diff åt höger | `Right` |
| Granskning: Kopiera markerad text eller rad | `Ctrl+C` |
| Tavla: Flytta kort till... | `m` |
| Tavla: Flytta kort åt vänster |  |
| Tavla: Flytta kort åt höger |  |
| Tavla: Öppna kort | `Enter` |
| Tavla: Redigera tilldelade... |  |
| Tavla: Redigera etiketter... |  |
| Tavla: Ange milstolpe... |  |
| Tavla: Nytt ärende i den här kolumnen | `n` |
| Tavla: Filtrera på tilldelad... | `a` |
| Tavla: Filtrera på etikett... | `l` |
| Tavla: Filtrera på milstolpe... | `M` |
| Tavla: Filtrera på text | `/` |
| Tavla: Rensa filter | `x` |
| Tavla: Ange kolumnetiketter... | `c` |
| Tavla: Uppdatera | `r` |
| Tavla: Stäng | `Esc` |
| Granskning: ✦ Granskningsutkast... | `s` |
| Granskning: Godta förslag | `y` |
| Granskning: Redigera och godta förslag... | `w` |
| Granskning: Avfärda förslag | `n` |
| Granskning: Godta alla förslag | `A` |
| Granskning: Avfärda alla förslag | `N` |
| Granskning: Använd AI-sammanfattning i granskningstexten | `i` |
| Granskning: ✦ Använd utkastet... | `u` |
| Granskning: Redigera ✦-utkastets text... | `t` |
| Granskning: Kassera ✦-utkastet | `N` |
| Ångrahistorik: Ångra hit... | `u` |
| Ångrahistorik: Gör om hit... | `y` |
| Ångrahistorik: Återställ bara den här... | `r` |
| Ångrahistorik: Återställ grenen till där den var... | `o` |
| Ångrahistorik: Uppdatera |  |
| Ångrahistorik: Stäng | `Esc` |
| Reflog: Visa i Log | `s` |
| Reflog: Skapa gren här... | `b` |
| Reflog: Återställ till den här positionen... | `r` |
| Reflog: Ladda fler | `m` |
| Reflog: Uppdatera |  |
| Förlorat arbete: Visa i Log | `s` |
| Förlorat arbete: Skapa gren här... | `b` |
| Förlorat arbete: Cherry-pick | `c` |
| Förlorat arbete: Sök igen |  |
| Vy: Uppdatera | `r` |
| Vy: Uppdatera (fetch, pull requests, ärenden) | `F5` |
| Vy: Växla hjälpöverlägg | `?` |
| Vy: Växla Ref-kolumn |  |
| Vy: Växla Graf-kolumn |  |
| Vy: Växla Meddelande-kolumn |  |
| Vy: Växla Författare-kolumn |  |
| Vy: Växla Datum-kolumn |  |
| Vy: Växla Ändringar-kolumn |  |
| Vy: Växla SHA-kolumn |  |
| Vy: Log-kolumn: välj en kolumn att flytta eller ändra bredd på | `C` |
| Vy: Log-kolumn: välj nästa kolumn | `Tab` |
| Vy: Log-kolumn: välj föregående kolumn | `Shift+BackTab` |
| Vy: Log-kolumn: flytta kolumnen åt vänster | `<` |
| Vy: Log-kolumn: flytta kolumnen åt höger | `>` |
| Vy: Log-kolumn: bredda kolumnen | `]` |
| Vy: Log-kolumn: smalna av kolumnen | `[` |
| Vy: Log-kolumn: återställ kolumnernas bredder | `0` |
| Vy: Log-kolumn: anpassa kolumnen efter innehållet | `=` |
| Vy: Log-kolumn: anpassa alla kolumner efter innehållet | `+` |
| Vy: Log: rulla kolumnerna åt vänster | `Shift+Left` |
| Vy: Log: rulla kolumnerna åt höger | `Shift+Right` |
| Vy: Log: växla rullning i sidled för kolumnerna | `s` |
| Vy: Log: välj vilka kolumner som visas |  |
| Diff: Öppna diff för vald fil | `Enter` |
| Diff: Stäng diff-vyn | `Esc` |
| Diff: Nästa hunk | `n` |
| Diff: Föregående hunk | `p` |
| Diff: Stage hunk under markören | `Space` |
| Diff: Unstage hunk under markören | `Space` |
| Diff: Markera rader (eller rensa markeringen) | `v` |
| Diff: Stage valda rader | `Enter` |
| Diff: Unstage valda rader | `Enter` |
| Diff: Förkasta hunk under markören... | `x` |
| Diff: Förkasta valda rader... | `X` |
| Kopiera: Markerad text (dra över texten först) | `Ctrl+C` |
| Kopiera: Raden under markören (eller valda rader) | `y` |
| Diff: Kopiera hunk under markören |  |
| Diff: Layout Inline (+/- i en kolumn) | `m` |
| Diff: Layout Split (före och efter) | `m` |
| Diff: Växla bara hunks (hela filen / bara ändringarna) | `H` |
| Filer: Växla sökväg/träd | `m` |
| Filer: Växla visa alla filer | `a` |
| Commit: Växla att hoppa över hooks | `H` |
| Filer: Öppna menyn för raden eller markeringen | `.` |
| Filer: Fäll ut alla mappar | `=` |
| Filer: Fäll ihop alla mappar | `-` |
| Filer: Fäll ut vald mapp och dess undermappar | `*` |
| Filer: Fäll ihop vald mapp och dess undermappar | `_` |
| Filer: Markera allt |  |
| Filer: Rensa markeringen |  |
| Filer: Växla markeringen av aktuell rad |  |
| Log: Kopiera som Mermaid gitGraph... | `Y` |
| Log: Jämför markeringen | `v` |
| Log: Byt plats på jämförelsens ändar | `x` |
| Log: gå till nästa jämförda rad | `n` |
| Diff: Växla radbrytning | `w` |
| Diff: Välj teckenkodning... | `C` |
| Vy: Växla minikarta | `M` |
| Vy: Växla commit-kolumnen | `Alt+C` |
| Vy: Commit-kolumnen: dölj |  |
| Vy: Commit-kolumnen: filens historik |  |
| Vy: Commit-kolumnen: alla commits |  |
| Vy: Fokusera commit-kolumnen |  |
| Vy: Bredda commit-kolumnen |  |
| Vy: Smalna av commit-kolumnen |  |
| Vy: Commit-kolumnen: rulla åt vänster | `Shift+Left` |
| Vy: Commit-kolumnen: rulla åt höger | `Shift+Right` |
| Vy: Commit-kolumnen: välj nästa del | `Tab` |
| Vy: Commit-kolumnen: flytta delen åt vänster | `<` |
| Vy: Commit-kolumnen: flytta delen åt höger | `>` |
| Vy: Commit-kolumnen: bredda delen | `]` |
| Vy: Commit-kolumnen: smalna av delen | `[` |
| Vy: Commit-kolumnen: återställ delarnas bredder | `0` |
| Vy: Commit-kolumnen: meny för den valda raden | `.` |
| Vy: Commit-kolumnen: välj kolumner |  |
| Vy: Commit-kolumnen: växla Ref |  |
| Vy: Commit-kolumnen: växla Graf |  |
| Vy: Commit-kolumnen: växla Författare |  |
| Vy: Commit-kolumnen: växla Datum |  |
| Vy: Commit-kolumnen: växla Ändringar |  |
| Vy: Commit-kolumnen: växla SHA |  |
| Vy: Commit-kolumnen: visa den valda raden i Log |  |
| Vy: Commit-kolumnen: visa filens ändringar vid den valda raden |  |
| Vy: Commit-kolumnen: visa filen som den var vid den valda raden |  |
| Vy: Commit-kolumnen: visa Blame för filen vid den valda raden |  |
| Redigerare: Växla följ markeringen |  |
| Bild: Zooma in | `+` |
| Bild: Zooma ut | `-` |
| Bild: Anpassa till vyn | `0` |
| Bild: Verklig storlek (1:1) | `1` |
| Bild: Växla före / efter | `Tab` |
| Bild: Bredda Före-panelen | `Alt+Right` |
| Bild: Bredda Efter-panelen | `Alt+Left` |
| Bild: Återställ panelstorlekar | `Alt+0` |
| Bild: Redigera i... | `O` |
| Bild: Redigera i standardredigeraren | `o` |
| Filhistorik: Visa i Log | `Enter` |
| Filhistorik: Kopiera SHA för commit | `y` |
| Filhistorik: Kopiera sammanfattningen | `Y` |
| Filhistorik: Visa diffen för denna commit | `d` |
| Filhistorik: Öppna filen vid denna commit | `o` |
| Filhistorik: Blame vid denna commit | `b` |
| Filhistorik: Jämför med arbetskatalogen | `c` |
| Filhistorik: Återställ den här versionen | `R` |
| Filhistorik: Utvidga markeringen uppåt | `Shift+Up` |
| Filhistorik: Utvidga markeringen nedåt | `Shift+Down` |
| Filhistorik: Växla mellan listan och diffen | `Tab` |
| Filhistorik: Bredda listan | `Alt+Right` |
| Filhistorik: Smalna av listan | `Alt+Left` |
| Filhistorik: Återställ listans bredd | `Alt+0` |
| Fil: Öppna med... | `O` |
| Fil: Öppna i standardappen | `o` |
| Diff: Växla ändringar som bara rör blanksteg | `s` |
| ✦ AI: Generera commit-meddelande | `Ctrl+G i` |
| ✦ AI: Generera stash-meddelande |  |
| ✦ AI: Infoga promptvariabel |  |
| ✦ AI: Redigera profil... |  |
| ✦ AI: Öppna agenten... |  |
| ✦ AI: Förklara vald commit | `Ctrl+G w` |
| ✦ AI: Dela upp vald commit... |  |
| ✦ AI: Förklara ändringar i arbetskatalogen | `Ctrl+G w` |
| ✦ AI: Komponera commits från ändringar... | `Ctrl+G C` |
| Composer: Flytta fil till commit... | `m` |
| Diff: Blame |  |
| Blame: Visa commit i Log | `Enter` |
| Blame: Kopiera commit-SHA | `Y` |
| Blame: Diff för commit i filen | `d` |
| Blame: Blame på commitens förälder | `b` |
| Blame: Öppna filhistoriken | `H` |
| Blame: Visa eller dölj detaljkortet | `i` |
| Blame: Nästa hunk | `n` |
| Blame: Föregående hunk | `p` |
| Blame: Nästa hunk från samma commit | `N` |
| Blame: Föregående hunk från samma commit | `P` |
| Blame: Gå till rad | `g` |
| Blame: Bredda annoteringskolumnen | `>` |
| Blame: Smalna av annoteringskolumnen | `<` |
| Diff: Filhistorik |  |
| Diff: Filvy |  |
| Fil: Redigera i arbetskatalogen | `e` |
| Redigerare: Ändra teckenkodning |  |
| Redigerare: Öppna igen med teckenkodning |  |
| Redigerare: Konvertera till teckenkodning |  |
| Redigerare: Konvertera radbrytningar |  |
| Redigerare: Ändra indrag |  |
| App: Inställningar... | `Ctrl+, / Ctrl+N ,` |
| App: Sök i inställningar... | `/ i Inställningar` |
| App: Tangentbindningar... |  |
| Agent: Visa väntande vy | `Ctrl+N Enter` |
| Agent: Avfärda väntande vy |  |
| Agent: Tillbaka | `Ctrl+N Backspace` |
| Tangentbindningar: Återställ alla... |  |
| Integrationer: Öppna länkad pull request | `Ctrl+G g` |
| Integrationer: Redigera länkad pull request på värden | `e (på en rad)` |
| Integrationer: Kopiera länkad pull request-URL |  |
| Integrationer: Uppdatera pull requests och ärenden |  |
| Integrationer: Checka ut pull request... |  |
| Integrationer: Öppna commit-referens (#ärende / !request)... |  |
| Integrationer: Öppna ärende... |  |
| Hosting: Granska begäran... |  |
| Hosting: Push för granskning... |  |
| Hosting: Redigera tilldelade... |  |
| Hosting: Redigera granskare... |  |
| Hosting: Redigera etiketter... |  |
| Hosting: Ange milstolpe... |  |
| Lokala ärenden: Stäng ärende... |  |
| Lokala ärenden: Öppna ärende igen... |  |
| Lokala ärenden: Öppna i Obsidian... |  |
| Ärenden: Öppna tavlan |  |
| Lokala ärenden: Skapa Obsidian Base |  |
| Pull requests: Skapa pull request... | `n` |
| Hosting: Nytt ärende... | `n` |
| Pull requests: Sök | `Enter på sökraden` |
| Pull requests: Välj filter (Öppna / Mergade / Alla)... | `v` |
| Pull requests: Välj leverantör... | `Enter på leverantörsraden` |
| Pull requests: Välj repository... | `Enter på repositoryraden` |
| Pull requests: Koppla bort källa (ingen) |  |
| Ärenden: Välj leverantör... | `Enter på leverantörsraden` |
| Ärenden: Koppla bort källa (ingen) |  |
| Integrationer: Koppla bort profil (automatisk) |  |
| Ärenden: Välj repository... | `Enter på repositoryraden` |
| Ärenden: Sök | `Enter på sökraden` |
| Ärenden: Välj filter... | `v` |
| Ärenden: Redigera fråga / listor... |  |
| Commit: Infoga ärendereferens... |  |
| Integrationer: Använd profil för det här repositoryt... |  |
| Integrationer: Hantera det här repositoryts profil |  |
| Integrationer: Anslut integration... |  |
| Inställningar: Radar-alternativ för en profil... |  |
| Integrationer: Granska säkerhetskontroll |  |
| App: Avsluta | `q` |
| App: Tema... |  |
| App: Byt tema... | `Ctrl+N y` |
| App: Byt språk... | `Ctrl+N g` |
| Startare: Skapa genväg |  |
| Startare: Skapa skrivbordsgenväg |  |
| Startare: Ta bort genvägar |  |
| Startare: Skapa kommandoalias |  |
| Startare: Ta bort kommandoalias |  |
| Startare: Lägg till i PATH |  |
| Startare: Ta bort från PATH |  |
| App: Byt färgtema för grafen... | `Ctrl+N Y` |
| App: Växla radfärgning i grafen | `Ctrl+N T` |
<!-- palette:end -->

## Förflyttning

| Tangent | Åtgärd |
|---|---|
| `Tab` / `Shift+Tab` | Fokusera nästa / föregående ruta |
| `Ctrl+N`, sedan ett märke | Fokusera den delen av fönstret (se [Navigeringsprefix](#navigeringsprefix-ctrln)) |
| `j` / `k`, `Up` / `Down` | Flytta markeringen, i varje ruta |
| `PageUp` / `PageDown`, `Home` / `End` | Flytta en sida; gå till första / sista raden |
| ett tal, sedan `j` / `k` / `Up` / `Down` | Räknarprefix: `4j` flyttar fyra rader. Med `PageUp` / `PageDown` är räknaren ett antal sidor. Statusraden visar den väntande räknaren (`4...`). `Esc` eller vilken annan tangent som helst rensar den. En inledande `0` startar aldrig en räknare |
| `h` / `l`, `Left` / `Right` | Refs-rutan: byt avsnitt. Log: lämna (`h`) eller gå in i (`l`) en commits lista över ändrade filer |
| `?` | Öppna eller stäng hjälpen (se [Hjälp i appen](#hjälp-i-appen-)) |
| `r` | Uppdatera |
| `F5` | Fetcha varje remote, ladda om pull requests och ärenden och uppdatera |
| `q` | Avsluta |
| `Ctrl+Z` | **Ångra** den senaste operationen. Se [Ångra och gör om](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Gör om**. `Ctrl+Shift+Z` kräver kitty-tangentbordsprotokollet; `Ctrl+Y` fungerar alltid |
| `.`, `Menu`, `Shift+F10` | Öppna snabbmenyn för den markerade raden |

`Ctrl+Z` och `Ctrl+Y` gör ingenting medan ett textfält har tangentbordet.
Redigeraren har sin egen textångra.

## Filtrerbara popuplistor

Alla filtrerbara popuplistor använder samma tangenter: kommandopaletten och
dess väljare (tema, språk, grafens färger), flikväljaren och varje rullgardinslista
med en sökrad (val i Inställningar, formuläret för pull request, väljaren för
ärendereferenser, listan **Öppna i**). Skriv för att filtrera listan. Dessa
tangenter flyttar markeringen:

| Tangent | Åtgärd |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Flytta en rad |
| `Home` / `End` | Första / sista raden |
| `PageUp` / `PageDown` | Flytta en synlig sida |
| `Enter` | Välj den markerade raden |
| `Esc` | Stäng |

I dessa popuper flyttar `Home`, `End` och `Ctrl+K` listan, inte markören i
filterfältet. `Ctrl+A` och `Ctrl+E` flyttar fortfarande markören till början och
slutet. Vanliga `j` och `k` skrivs in i filtret.

Med musen:

- Hjulet **rullar** listan. Det flyttar inte markeringen.
- Ett klick **markerar** en rad. Ett dubbelklick **väljer** den, som `Enter`.
- En lista som är högre än sin popup visar en rullningslist på högra kanten.
  Du kan dra i den.
- En kort rullgardinslista utan sökrad (två eller tre val) väljer vid det
  första klicket.

**Liveförhandsvisning.** Väljarna för tema, färgtema för grafen och språk
tillämpar den markerade raden direkt, bakom popupen. `Enter` behåller den.
`Esc`, eller ett klick utanför popupen, återställer det tidigare värdet.

**Temafilter.** Temaväljaren har en rad under filterfältet: **Alla**,
**Mörka**, **Ljusa** och **Färgade**. `Tab` / `Shift+Tab` går till nästa /
föregående. `Alt+1` till `Alt+4` väljer ett direkt. Du kan också klicka på dem.

## Hjälp i appen (`?`)

`?`, `Ctrl+N ?` eller palettens `Vy: Växla hjälpöverlägg` öppnar hjälpen.
Den listar tangenterna och musgesterna för varje del av appen. Den öppnas över
allt, även skärmen Inställningar.

| Tangent | Åtgärd |
|---|---|
| `?` | Stäng hjälpen (och rensa filtret) |
| *(skriv)* | Filter: behåll bara de avsnitt som innehåller texten. `j`, `k` och `q` skrivs också in |
| `Backspace`, `Ctrl+U` | Radera sista tecknet / rensa filtret |
| `Esc` | Rensa filtret; om det är tomt, stäng hjälpen |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Rulla en rad |
| `PageUp` / `PageDown`, `Home` / `End` | Rulla en sida; gå till början / slutet |
| `Ctrl+D`, eller ett klick på länken | Öppna den här dokumentationen på GitHub i webbläsaren |

Hjulet rullar hjälpen, och du kan dra i dess rullningslist. Ett klick utanför
hjälpen stänger den.

## Navigeringsprefix (`Ctrl+N`)

`Ctrl+N` väntar på ytterligare en tangent. Statusraden visar vilka tangenter den
tar emot. Nästa tangent kör en åtgärd, eller avbryter.

| Tangent | Åtgärd |
|---|---|
| `Ctrl+N` sedan ett märke | Fokusera den del av fönstret som visar det märket |
| `Ctrl+N h` / `Ctrl+N l` | Fäll ihop eller återställ Refs-rutan / högerpanelen |
| `Ctrl+N Left` / `Ctrl+N Right` | Flytta avdelaren mellan Refs-rutan och Log ett steg åt vänster / höger (en hopfälld Refs-ruta kommer tillbaka först) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Flytta avdelaren mellan Log och högra panelen ett steg åt vänster / höger |
| `Ctrl+N +` / `Ctrl+N -` | Förstora / förminska det aktiva Refs-avsnittet med en rad |
| `Ctrl+N ]` / `Ctrl+N [` | Förstora / förminska commit-meddelandets block i högerpanelen med en rad |
| `Ctrl+N t` | Välj en flik (flikväljaren; bara `t` gör detsamma) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Byt tema / språk / färgtema för grafen |
| `Ctrl+N Shift+T` | Slå på eller av radfärgningen i grafen |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Visa vyn en agent förberedde / gå tillbaka (se [MCP och agenter](mcp-and-agents.md)) |
| `Ctrl+N ,` | Öppna Inställningar |
| `Ctrl+N ?` | Öppna eller stäng hjälpen |
| `Esc`, eller vilken annan tangent som helst | Avbryt |

### Märken

Medan prefixet väntar visar varje del av fönstret som kan ta fokus ett märke
på ett tecken på sin övre vänstra ram. Tryck på det tecknet för att fokusera
delen. Märkena delas ut på nytt varje gång, i skärmordning: först `1` till `9`,
sedan `a`, `b`, `c`... Läs alltså märket på skärmen: en del får inte alltid
samma märke.

Delar med märke: Refs-filtret och varje synligt Refs-avsnitt, Log (eller
Diff-vyn), Filvyn när den är öppen, listorna Unstaged och Staged, sammanfattningsfältet
i commit-composern och kontrollerna i commitens fillista. På fliken Hem är
märkena sökfältet, de fyra sektionerna och knapparna i verktygsfältet (en knapps märke trycker på den). På fliken Radar är det filterfältet,
rullgardinslistorna Omfång, Vy och Visa (en rullgardinslistas märke öppnar den),
knapparna Uppdatera och Tillbaka samt varje sektionsrubrik på skärmen. En sektions
märke flyttar markeringen till rubriken; `Enter` eller `Space` viker sedan ihop den.

I mappväljaren Bläddra och i dialogerna Klona, Init, Skapa worktree, Arbetsyta, Delvis utcheckning, Flow, Nytt ärende och Skapa pull request är märkena fälten, kryssrutorna och knapparna (en knapps märke trycker på den, ett fälts märke sätter markören i det; i en lista sitter ett märke på dess första rad; mappväljaren Bläddra har ett märke för listan Snabbåtkomst och ett för listan Mappar, bredvid varje listas första rad, och att trycka på ett sätter tangentbordet i den listan utan att välja en rad). I Aktivitetsloggen är de de två flikarna, rutorna Omfång och Nivå, filtret, nivåkryssrutorna, knapparna och listans första post. Ett märke gör vad ett klick på kontrollen gör. I mappväljaren startar `Ctrl+N` märkena, så dess tangent för Ny mapp är `Alt+N`.

I dialogen Ändra (och Redigera commit-meddelande) är märkena sammanfattningen, beskrivningen, författarens namn, e-post och datum, **Ändra historiken djupare...**, **Avbryt**, **Tillämpa** och stängningskorset; i omskrivningsvyn är de omfångsrutan, dess kryssrutor och refs-knapp, receptets namn, varje regel, dess kryssruta och knappar, knappen **+ Lägg till regel**, fälten i den valda regeln, förhandsgranskningsflikarna, filterrutan, raderna för skyddade grenar med Markera alla / Avmarkera alla, och knapparna Förhandsgranska, Tillämpa, Avbryt, Spara, Läs in och Skapa regler med AI (sidorna Hemligheter, Identiteter, Storlek och resultat lägger till egna knappar och rader; fliken Commits rader, som är en lista, har inga). Ett märke gör vad ett klick gör. Inget märks medan en rullgardinslista, en fråga, ett fält som skrivs i, mappväljaren, rensningsskärmen, skriptredigeraren, AI-dialogerna, blob-diffen eller värdguiden är öppen, och inte heller medan dialogen Ändra frågar "förkasta?"; `Esc` tar bort märkena utan att stänga något.

I Inställningar sitter märkena på sökrutan, sidlistan (ett enda märke, på den aktuella sidan), underflikarna, listan och dess knappar, varje fält på sidan som syns på skärmen och stängningskorset. Ett märke gör vad ett klick på kontrollen gör: ett textfält öppnas för redigering, en kryssruta kryssas, en rullgardinslista öppnas, en knapp körs. `Ctrl+N` behåller sin egen betydelse medan du skriver i ett fält, eller en rullgardinslista, en redigerare eller sökningen är öppen.

Paletten har samma åtgärder, till exempel
`Navigering: Fokusera Refs > Lokala` eller `Navigering: Fokusera Status > Staged`.

## Repositories och flikar

| Tangent | Åtgärd |
|---|---|
| `Ctrl+O`, eller ett klick på `⌂` | Gå till fliken Hem (på fliken Hem: fokusera sökfältet; med Radarn eller versionsanteckningarna öppna: stäng dem först) |
| `Ctrl+Shift+N` | Öppna fliken Hem med dialogen Klona |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Nästa / föregående flik |
| `t` | Välj en flik i en filtrerbar lista |
| `Ctrl+W` | Stäng aktuell flik (frågar först; repositoryt ändras inte) |
| `Ctrl+T`, eller ett högerklick på en flik | Flikmenyn (dess rader listas i [Flikraden](interface-tour.md#flikraden)) |
| `Alt+O` / `Ctrl+Alt+O` | Öppna flikens mapp: välj i listan **Öppna i** / använd standardposten. Se [Öppna i](open-in.md) |

### Fliken Hem

Fliken `⌂` hanterar dina repositories. Den har fyra sektioner: **Öppna
repositories**, **Favoriter**, **Senaste repositories** och **Alla
repositories**. Se [Rundtur i gränssnittet](interface-tour.md#hem).

| Tangent | Åtgärd |
|---|---|
| `j` / `k`, piltangenter, `PageUp` / `PageDown`, `Home` / `End` | Flytta |
| `Enter`, eller ett dubbelklick | Öppna repositoryt (eller växla till dess flik). På en sektionsrubrik: fäll ihop eller ut den |
| `f` eller `*`, eller ett klick på stjärnan | Lägg till i / ta bort från favoriter |
| `x`, `Delete`, eller radens knapp | Stäng dess flik (Öppna), ta bort det från listan (Senaste), ta bort stjärnan (Favoriter), glöm det (Alla). Inget raderas på disk |
| `.`, ett högerklick, eller radens `⋮` | Radens meny |
| `/` eller `Ctrl+F` | Sök i alla sektioner efter namn, gren eller sökväg. `Enter` öppnar första träffen; `Esc` rensar sökningen och lämnar sedan fältet |
| `Space`, `Left` / `Right` | Fäll ihop / ut aktuell sektion |
| `-` / `+` | Fäll ihop alla / fäll ut alla |
| `b` / `c` / `i` | Bläddra (öppna en mapp) / Klona / Init |
| `Esc` | Rensa sökningen, eller gå tillbaka till den aktiva repositoryfliken |
| `q` | Avsluta |

`Ctrl+P`, `Ctrl+,`, `?` och `Ctrl+N` fungerar på fliken Hem. `Ctrl+G`, `Ctrl+W`
och `Ctrl+T` kräver en repositoryflik: på fliken Hem visar de bara ett
meddelande. Palettkommandon som kräver ett repository är nedtonade.

### Mappväljaren

**Bläddra** (och knappen **Bläddra...** i dialogerna Init, Klona och Skapa
worktree) öppnar mappväljaren. Den har ett sökvägsfält högst upp, platserna
och mappträdet till vänster, och innehållet i aktuell mapp till höger.

| Tangent | Åtgärd |
|---|---|
| `Tab` / `Shift+Tab` | Sökvägsfältet, sedan vänstra kolumnen, sedan innehållet |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Flytta i den fokuserade kolumnen. I sökvägsfältet: flytta bland kompletteringarna |
| en bokstav (innehållet) | Gå till nästa mapp som börjar med den bokstaven |
| `Enter` / `Right` | Öppna den markerade mappen. I sökvägsfältet: gå till den skrivna mappen; är den redan visad, välj den |
| `Backspace` / `Left` / `Alt+Up` | Föräldramapp (i sökvägsfältet raderar `Backspace` ett tecken) |
| `Tab` (sökvägsfältet) | Godta den enda kompletteringen, eller den markerade |
| `Alt+N` | Ny mapp (`Ctrl+N` startar märkena, se [Märken](#märken)) |
| `Alt+H` | Visa eller dölj dolda mappar |
| `Ctrl+F`, eller `/` utanför sökvägsfältet | Filtrera mappens poster medan du skriver (skiftlägesokänsligt; `..` blir kvar; rubriken räknar ”x av n”; `Enter` hoppar till första träffen, `Esc` rensar; filtret töms när du byter mapp). Nås också med `Tab` eller ett klick |
| `Ctrl+S`, `Ctrl+Enter`, eller bekräftelseknappen | Välj den markerade undermappen, annars den visade mappen |
| `Esc` | Stäng kompletteringarna, eller avbryt |

När sökvägsfältet visar en ny sökväg är hela sökvägen markerad: den första
tangent du skriver ersätter den.

### Dialogerna Init och Klona

| Tangent | Åtgärd |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Nästa / föregående fält eller knapp |
| `Enter` | Tryck på den fokuserade knappen, eller skapa / klona |
| `Space` | Växla den fokuserade kryssrutan, eller tryck på den fokuserade knappen |
| `Ctrl+B` | Bläddra efter mappen |
| `j` / `k`, `g` / `G` | I klondialogens lista över källor och lista över repositories: ned / upp, första / sista (inte i sökfältet) |
| `Esc` | Stäng. Under en kloning fortsätter kloningen i bakgrunden |

I dialogen Klona listar vänstra kolumnen **Klona med URL** och en källa per
ansluten värdprofil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Välj en källa med
`Up` / `Down` eller ett klick. Skriv sedan i sökfältet **Repository**, flytta i
listan med piltangenterna, `PageUp` / `PageDown` eller hjulet, och välj en rad
med `Enter` eller ett klick. Se [Komma igång](getting-started.md) för fälten.

## Refs-rutan

| Tangent | Åtgärd |
|---|---|
| `Enter` / `Space` | Fäll ut eller ihop det aktiva avsnittet, eller den markerade mappen |
| `h` / `l`, `Left` / `Right` | Byt avsnitt |
| `/` | Filtrera efter namn (`Enter` tillämpar, `Esc` avbryter) |
| `y` | Slå på eller av grenhierarkivyn: mappar från `/` i grennamn (LOKALA och REMOTE) |
| `o` | **Solo** för den markerade refen, eller för varje ref under den markerade mappen eller remoten. Solo för flera refs för att se deras union |
| `Shift+O` | **Sluta solo**: rensa varje solo |
| `Shift+H` | **Visa alla**: visa varje dold ref igen |
| `g` | **Visa i grafen**: rulla Log till den markerade refens (eller stashens) commit och blinka till på dess rad; markeringen flyttas inte |
| `m` | **Markera** den valda refen som släppkälla (`Esc`, eller `m` på den igen, tar bort markeringen) |
| `Shift+M` | **Släpp den markerade refen** på den valda refen: öppnar menyn som en dragning öppnar (Merge, Rebase, Push, Fast-forward...) |
| `+` (avsnittet REMOTE) | Lägg till en remote |
| `.`, `Menu`, `Shift+F10`, eller ett högerklick | Radens meny: en gren, en tag, en remotes namn eller en mapp |
| `Enter` på en rad under WORKTREES | Öppna den worktreen i en egen flik |
| `Ctrl+G d` | Ta bort den markerade grenen eller taggen (frågar först). På en REMOTE-rad: ta bort den remote-grenen |

För vad menyerna gör, se [Arbeta med Log](log-and-history.md).

### Pull requests och ärenden

Med en matchande integration har Refs-rutan ett avsnitt för pull requests
(GitLab: merge requests). Med en ansluten värdprofil har den också ett avsnitt
ÄRENDEN. Se [Värdtjänster och ärendehantering](hosting-integration.md#5-pull-requests-och-ärenden-i-sidopanelen).

| Tangent | Åtgärd |
|---|---|
| `Enter` | Öppna den markerade raden i webbläsaren. På en `Anslut igen`-rad: öppna Inställningar på den profilen |
| `Space` | Fäll ut eller ihop avsnittet |
| `v` | Öppna avsnittets filter. Pull requests: Öppna, Mergade / stängda, Alla. Ärenden: Öppna, Tilldelade mig, Stängda, Alla |
| `n` | Avsnittet pull requests: skapa en pull request. Avsnittet ÄRENDEN: nytt ärende. (Rubrikens `+`-knapp) |
| `e` | Avsnittet pull requests: ändra den markerade begäran på värden (radens penna) |
| `Enter` på sökraden | Skriv i sökfältet. `Enter`, `Tab` eller `Esc` lämnar det; `Ctrl+U` rensar det |
| `Enter` på en grupprubrik | Fäll ihop eller ut gruppen (*Mina pull requests*, *Tilldelade mig*, *Väntar på min granskning*, *Alla pull requests*) |
| `Enter` på filter-, leverantörs- eller repositoryraden | Öppna den rullgardinslistan. Leverantörslistan slutar med **Ingen (koppla bort det här repositoryt)** |
| `.`, eller ett högerklick | Radens meny: öppna i webbläsaren, kopiera URL eller nummer, checka ut källgrenen (pull requests); skapa en gren från ärendet (ärenden) |

### Ärendetavlan

**Tavla** i ÄRENDEN-huvudet (palett: **Ärenden: Öppna tavlan**) öppnar Kanban-tavlan för ärendekällan. Se
[tavlan](hosting-integration.md#5b-ärendetavlan) och [lokala ärenden](local-issues.md#kanban-tavlan). Varje tangent
nedan är också ett palettkommando (**Tavla: ...**).

| Tangent | Åtgärd |
|---|---|
| Piltangenter | Markera ett kort (`Up`, `Down`) eller grannkolumnen (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` hoppar i kolumnen |
| `Shift+Left`, `Shift+Right` | Flytta det markerade kortet till föregående eller nästa kolumn |
| `m` | **Flytta kort till...**: en lista med kolumnerna |
| `Enter` | Öppna ärendet (anteckningen för ett lokalt ärende; annars värdens sida) |
| `e`, `.` | Redigera tilldelade, etiketter eller milstolpe; kortmenyn |
| `n` | Nytt ärende i den markerade kolumnen |
| `a`, `l`, `M`, `/` | Filtrera på tilldelad, etikett, milstolpe, text. `x` rensar filtren |
| `c` | Etiketterna som är kolumner i en GitHub- / GitLab-tavla |
| `r` | Läs ärendena igen |
| `Esc` | Stäng tavlan |

Mus: dra ett kort till en annan kolumn för att flytta det, klicka på ett kort för att markera det, dubbelklicka för att öppna det, högerklicka för
kortmenyn, hjulet rullar en kolumn, `Shift`+hjul (eller listen under tavlan) rullar kolumnerna.

### Flow (grenarbetsflöden)

Med ett arbetsflöde konfigurerat för repositoryt (se [grenarbetsflöden](branching-workflows.md)) har verktygsraden en
**Flow**-knapp, och sidopanelen grupperar LOCAL och REMOTE efter grentyp. Varje rad i menyn är ett palettkommando
(**Flow: ...**; **Flow: Starta Feature...** och **Flow: Nytt Feature-ärende...** genereras för varje ämnestyp i
arbetsflödet). De hostdrivna raderna (*Nytt <Typ>-ärende*, *Plocka upp en gren som hosten skapat*) och deras dialoger, väntekortet och
kortet ”en gren dök upp” beskrivs i [grenarbetsflöden](branching-workflows.md#hostdrivna-flöden-i-appen);
`Enter` kör den fokuserade knappen (*Skapa ärende*, *Plocka upp*), `Esc` slutar vänta eller stänger.

| Tangent | Åtgärd |
|---|---|
| `Ctrl+G Shift+W` | Öppna Flow-menyn (verktygsradens knapp) |
| `Enter`, `Tab`, `Shift+Tab` | I dialogen Starta / Slutför / Synka / Promota / Ange förälder: förhandsvisa eller kör, nästa fält, föregående fält |
| `Space` | Kryssa en kryssruta (*Publicera*, *Kör hooks*, *Uppdatera även grenarna som är staplade på den*) |
| `Esc` | Stäng en rullgardin, gå tillbaka från en förhandsvisning till formuläret, stäng dialogen (inte medan en körning pågår) |
| Håll muspekaren över en gren | Dess typ, förälder och hur långt efter föräldern den ligger |

## Log-rutan

| Tangent | Åtgärd |
|---|---|
| `Enter` / `l` | Gå in i den markerade commitens lista över ändrade filer |
| `Enter` / `l` på en filrad | Öppna filens diff i mitten |
| `Esc` / `h` | Gå tillbaka till commits |
| `Right` | (commit med mer än en ref) Öppna listan med dess andra refs: `Up` / `Down` flyttar, `Enter` checkar ut, `Esc` stänger |
| `f` | Öppna sökfältet (se [Sök](#sök)) |
| `c` | (med ocommittade ändringar) Markera WIP-raden och placera markören i dess meddelandefält |
| `Enter` på WIP-raden | Placera markören i WIP-radens meddelandefält |
| `.`, `Menu`, `Shift+F10` | Commitmenyn. `Up` / `Down` flyttar, `Right` / `l` / `Enter` öppnar en undermeny, `Left` / `h` / `Esc` går tillbaka |
| `m` / `a` | Filer i högerpanelen: `Sökväg` eller `Träd` / `Visa alla filer` |
| `Shift+C` | Välj en Log-kolumn att flytta eller ändra bredd på (se nedan) |

Med en kolumn vald väljer `Tab` / `Shift+Tab` en annan, `<` / `>` flyttar den, `]` / `[` gör den bredare eller smalare, `0` ger alla bredder automatiskt värde igen och `Esc` eller `Enter` avslutar (kontexten `log_columns` i den genererade tabellen ovan).

WIP-radens meddelandefält är commit-formulärets sammanfattning. Medan det har markören
skrivs varje tangent in, men `Ctrl`-kombinationer fungerar fortfarande. `Enter`
går vidare till beskrivningen. `Esc` lämnar fältet och behåller
texten. Se [Stage och commit](staging-and-committing.md).

### Markera flera commits

| Tangent eller gest | Åtgärd |
|---|---|
| `Space` | Lägg till aktuell commit i markeringen, eller ta bort den |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Utöka markeringen (Alt för terminaler som behåller Shift åt sig själva) |
| `Ctrl+klick` / `Shift+klick` eller `Alt+klick` | Växla en commit / markera ett intervall |
| `Esc` | Rensa markeringen |
| `Shift+Y` | [Kopiera markeringen som en Mermaid gitGraph](log-and-history.md#kopiera-commits-som-en-mermaid-gitgraph) (en popup med detaljnivåer och förhandsvisning live; `Enter` kopierar) |

Med flera commits markerade, högerklicka på en av dem och välj **Cherry-pick N
commits på `<gren>`...**. Det öppnar samma vy som en interaktiv rebase.

### Ge en gren eller tag ett namn

`Ctrl+G b`, `Ctrl+G t`, verktygsfältets knappar **Gren** och **Tag** och menyernas
**Skapa gren här...** / **Skapa tag här...** ber om namnet i en textruta på
commitens rad. `Enter` skapar refen. `Esc` avbryter. Ett ogiltigt eller redan
befintligt namn visas i rött, och rutan förblir öppen.

### Redigera ett commit-meddelande

**Redigera commit-meddelande...** i commitmenyn, `Ctrl+G e` eller palettens
`Git: Redigera commit-meddelande...` öppnar popupen **Redigera commit-meddelande**.
Se [Arbeta med Log](log-and-history.md) för vad den ändrar.

| Tangent | Åtgärd |
|---|---|
| `Enter` i sammanfattningen | Gå till beskrivningen |
| `Enter` i beskrivningen | Ny rad. `Up` på första raden går tillbaka till sammanfattningen |
| `Ctrl+Enter`, `Ctrl+S`, eller **Uppdatera meddelande** | Spara det nya meddelandet |
| `Tab` / `Shift+Tab` | Flytta mellan sammanfattning, beskrivning och knapparna |
| `Esc`, eller **Avbryt** | Stäng utan ändring |

### Ändra en commit

`Ctrl+G Shift+E` (alla fält) och `Ctrl+G Shift+A` (författaren) öppnar dialogen Ändra commit, och det gör även commitmenyn, palettens rader `Commit: Ändra...` och commitens detaljruta; alla vägar dit beskrivs i [Ändra en commit](log-and-history.md#ändra-en-commit-meddelande-författare-datum). Båda tangentföljderna kan bindas om: `git_prefix.amend` och `git_prefix.edit_author`. Tangenter i dialogen:

| Tangent | Åtgärd |
|---|---|
| `Tab` / `Shift+Tab` | Flytta mellan fälten och knapparna |
| `Enter` i ett enradsfält | Nästa fält; efter det sista, **Tillämpa** |
| `Up` / `Down` | Föregående / nästa fält (i beskrivningen från dess första / sista rad) |
| `Ctrl+Enter`, `Ctrl+S`, eller **Tillämpa** | Tillämpa de ändrade fälten |
| `Esc`, eller **Avbryt** | Stäng utan ändring |
| `Alt+H`, eller **Ändra historiken djupare...** | Stäng dialogen och öppna omskrivningsvyn från den här commiten (bara i byggen med historikomskrivning); frågar först om du har ändrat ett fält (`Esc` bekräftar). Kan bindas om: `reword.deeper` |

### Skapa worktree

**Skapa worktree från `<ref>`...** (menyerna för gren och tag) och **Skapa
worktree här...** (commitmenyn) öppnar en dialog: mappen, en valfri ny gren och
**Öppna den nya worktree som flik**. Tangenter: `Tab` / `Shift+Tab` flyttar,
`Enter` skapar, `Space` växlar kryssrutan, `Ctrl+B` bläddrar, `Esc` avbryter.

### Interaktiv rebase

**Interaktiv rebase härifrån...** i commitmenyn, eller
`Git: Interaktiv rebase från vald commit...`, öppnar rebase-vyn. En
cherry-pick av flera commits använder samma vy. Se
[Arbeta med Log](log-and-history.md).

| Tangent | Åtgärd |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Ställ in radens åtgärd: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup och Edit bara i en rebase) |
| `Space` | Öppna radens åtgärdslista (`Up` / `Down` väljer, `Enter` tillämpar, `Esc` stänger listan) |
| `Up` / `Down` | Flytta markeringen |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Flytta den markerade raden upp / ned |
| `a` | Slå på eller av förhandsvisningen av autosquash |
| `Enter` på en Reword-rad | Redigera den commitens nya meddelande |
| `Ctrl+S`, `Ctrl+Enter`, eller **Starta rebase** / **Starta cherry-pick** | Kör planen |
| `Ctrl+R`, eller **Återställ** | Återställ planen som den var när vyn öppnades |
| `Esc`, eller **Avbryt** | Stäng utan ändring |

När en rebase stannar (vid en konflikt eller ett Edit-steg) erbjuder en banner
ovanför Log **Fortsätt**, **Hoppa över** och **Avbryt**. Paletten har
`Rebase: Fortsätt`, `Rebase: Hoppa över` och `Rebase: Avbryt`.

## Sök

`f`, verktygsfältets knapp **Sök** eller `Navigering: Sök commits...` öppnar
sökfältet över Log. Det söker i hela commit-meddelandet, författarens namn och
e-post, SHA och refnamnen. Det döljer inga rader: rader som inte matchar tonas
ned.

| Tangent | Åtgärd |
|---|---|
| *(skriv)* | Redigera söktexten (varje tangent skrivs in) |
| `Enter` / `Down`, eller fältets `↑` | Nästa träff |
| `Shift+Enter` / `Up`, eller fältets `↓` | Föregående träff |
| `Esc`, eller fältets `✕` | Stäng sökfältet |

## Diff-vy

En ändrad fil öppnas i mitten, i stället för Log. Se
[Stage och commit](staging-and-committing.md).

| Tangent | Åtgärd |
|---|---|
| `Enter` / `l`, eller ett dubbelklick (på en filrad) | Öppna filens diff |
| `Esc`, `q`, eller `✕` i titeln | Stäng diffen |
| `j` / `k`, `Up` / `Down` | Rulla |
| `PageUp` / `PageDown`, `Home` / `End` | Rulla en sida; gå till början / slutet |
| `h` / `l`, `Left` / `Right` | Rulla i sidled (inte med radbrytning) |
| `n` / `p` | Nästa / föregående hunk |
| `m` | Inline- eller split-layout |
| `Shift+H` | Bara hunks eller hela filen |
| `w` | Slå på eller av radbrytning |
| `s` | Visa eller dölj ändringar som bara rör blanksteg |
| `Shift+M` | Slå på eller av minikartan |
| `Shift+C` | Välj teckenkodningen filen läses i (en lista: Autoavkänning först) |
| `e` | Redigera filen i den inbyggda redigeraren (se [Redigerare](#redigerare)) |
| `r` | (Git LFS-fil) LFS-kortet eller den råa pekaren. Se [Git LFS](lfs.md) |
| `Shift+E` | (fil med konflikt) Öppna merge-redigeraren. Se [Lösa merge-konflikter](merge-conflicts.md) |
| `y` / `Ctrl+C` | Kopiera den markerade texten. Utan markering: raden under markören, eller de markerade raderna |
| `.`, eller ett högerklick på en hunk | Hunkmenyn |

### Stagea och förkasta från diffen

Dessa tangenter fungerar bara i en diff av ändringarna i arbetskatalogen (öppnad
från listan Unstaged eller Staged). Att förkasta fungerar bara i listan
Unstaged.

| Tangent | Åtgärd |
|---|---|
| `Space` | Stagea (Unstaged) / unstagea (Staged) hunken under markören |
| `v` | Starta eller släpp en radmarkering; `j` / `k` utökar den |
| `Enter` | Stagea / unstagea de markerade raderna, eller raden under markören |
| `x` | Förkasta hunken under markören (frågar först: `y` / `Enter` bekräftar, `n` / `Esc` avbryter) |
| `Shift+X` | Förkasta de markerade raderna, eller raden under markören (frågar först) |

Bara ändrade rader räknas: kontextrader i en markering ignoreras. En markering
som når in i en andra hunk verkar bara på den första hunken.

## Bildvy

En bild i Diff-vyn eller Filvyn. Tangenterna finns i kontexten `image` i
tabellen ovan. Se [Bilder](image-view.md).

## Filvy, Blame och Filhistorik

| Tangent | Åtgärd |
|---|---|
| `Esc` / `q` | Stäng (med en textmarkering rensar `Esc` den först) |
| `j` / `k`, `Up` / `Down` | Flytta markeringen |
| `PageUp` / `PageDown` | Rulla en sida |
| `Home` / `End`, `Shift+G` | Första / sista raden |
| `h` / `l`, `Left` / `Right` | Rulla texten i sidled |
| `Enter` | (Blame, Filhistorik) Gå till radens commit i Log |
| `e` | Redigera filen i den inbyggda redigeraren |
| `y` / `Ctrl+C` | Kopiera den markerade texten. Utan markering: den markerade raden (Blame: bara koden; Filhistorik: sammanfattningen) |

Blame och filhistoriken har egna tangenter, listade i den genererade tabellen ovan (kontexterna `file_insight` och `history`) och som `Blame: ...` / `Filhistorik: ...` i paletten; alla kan omdefinieras. Detaljer i [Blame](log-and-history.md#blame) och [Filhistorik](log-and-history.md#filhistorik). I historiklistan verkar de enstaka bokstäverna (`d` `o` `b` `c` `y`) på den valda raden, men ett Ctrl-ackord (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) behåller alltid sin globala betydelse.

## Commit-kolumnen

Kolumnen till vänster om en fils vy (se [Commit-kolumnen](interface-tour.md#commit-kolumnen)). `Alt+C` visar eller döljer den varifrån som helst; de andra tangenterna fungerar när den har tangentbordet (`Ctrl+N` och sedan dess märke, eller `Vy: Fokusera commit-kolumnen`).

| Tangent | Åtgärd |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Välj en commit |
| `Enter` / `Right` | Gå vidare till den valda commitens fillista |
| `Esc` | Ge tillbaka tangentbordet till vyn |
| `.`, `Menu`, `Shift+F10` | Den valda radens meny |

Dess övriga tangenter (`Shift+Left` / `Shift+Right` för att rulla i sidled, `Tab`, `]`, `[` och `0` för delarnas storlek) finns i den genererade tabellen ovan, kontexten `commit_column`.

## Redigerare

**Redigera**, eller `e` i en diff eller en Filvy, öppnar den inbyggda
redigeraren på kopian av filen i arbetskatalogen. Se
[Stage och commit](staging-and-committing.md).

| Tangent | Åtgärd |
|---|---|
| *(skriv)* | Infoga vid markören (ersätter markeringen) |
| `Enter` | Ny rad, med nuvarande radens indrag |
| `Tab` / `Shift+Tab` | Indrag / minska indrag (de markerade raderna, om flera) |
| piltangenter, `Home` / `End` | Flytta. `Home` går till första tecknet som inte är blanksteg, sedan till kolumn 0. `Shift` utökar markeringen |
| `Ctrl+Left` / `Ctrl+Right` | Flytta ett ord |
| `Ctrl+Home` / `Ctrl+End` | Början / slutet av filen |
| `PageUp` / `PageDown` | En sida upp / ned |
| `Ctrl+A` | Markera allt |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Kopiera / klipp ut / klistra in |
| `Ctrl+U` / `Ctrl+K` | Radera till radens början / slut |
| `Ctrl+Z` | Ångra |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Gör om |
| `Ctrl+S` | Spara |
| `Ctrl+F` / `F3` | Sök. I sökfältet: `Enter` / `Down` / `F3` nästa, `Shift+Enter` / `Up` föregående, `Esc` stänger |
| `Alt+M` | Slå på eller av minikartan (**Karta**) |
| `Esc` | Stäng. Med osparade ändringar frågar den: **Spara** (`S` / `Enter`), **Förkasta** (`D`), **Avbryt** (`C` / `Esc`) |

`Tab` ger indrag i redigeraren, så den flyttar inte fokus. Andra
`Ctrl`-kombinationer (`Ctrl+P`, `Ctrl+N`) fungerar fortfarande.

**Teckenkodning**, **radslut** och **indrag** i statusraden är knappar: varje öppnar en rullgardinsmeny
(piltangenter eller hjul, skriv för att filtrera, `Enter` väljer, `Esc` stänger). Paletten (`Ctrl+P`) öppnar
samma menyer: *Redigerare: Ändra teckenkodning*, *Redigerare: Öppna igen med teckenkodning*, *Redigerare: Konvertera till teckenkodning*, *Redigerare: Konvertera radbrytningar*, *Redigerare: Ändra indrag*. Mer i [Stage och commit](staging-and-committing.md).

## Git-åtgärder (`Ctrl+G`)

`Ctrl+G` väntar på ytterligare en tangent:

| Tangent | Åtgärd |
|---|---|
| `s` / `u` | Stagea / unstagea den markerade filen |
| `a` | Stagea alla ändringar |
| `x` | Förkasta alla ändringar (frågar först) |
| `c` | Öppna commit-composern |
| `Enter` | Commita de stagade ändringarna nu, med formulärets meddelande, varifrån som helst (se [Stage och commit](staging-and-committing.md#skriva-commiten)) |
| `m` | Stasha alla ändringar med ett meddelande (tomt är tillåtet). Med AI på skriver `Ctrl+G` i den prompten meddelandet |
| `Shift+T` | Stasha alla ändringar (ospårade filer inkluderade), utan meddelande |
| `o` | Pop en stash: stash-raden som är markerad i Log, annars den senaste stashen |
| `p` | Pull, så som knappen Pull är inställd (`git.pull_mode`) |
| `Shift+P` | Pusha till standard-remoten (anger upstream vid första push), eller fråga efter remoten |
| `Shift+U` | Pusha alla tags |
| `f` | Fetcha varje remote, med prune |
| `g` | Öppna pull requesten för den utcheckade grenen i webbläsaren |
| `b` / `t` | Ny gren / ny tag vid HEAD (se [Ge en gren eller tag ett namn](#ge-en-gren-eller-tag-ett-namn)) |
| `d` | Ta bort den markerade grenen eller taggen (frågar först) |
| `e` | Ändra den markerade commitens meddelande |
| `Shift+E` / `Shift+A` | Ändra den markerade commiten (alla fält) / dess författare (se [Ändra en commit](#ändra-en-commit)) |
| `y` | Cherry-pick den markerade commiten |
| `v` | Revert den markerade commiten |
| `Shift+R` | Reset (mixed) till den markerade commiten |
| `r` | Rebase på den markerade grenen |
| `Shift+W` | Öppna Flow-menyn (ett repository med ett grenarbetsflöde) |
| `Shift+H` | Öppna omskrivningsvyn (bara i byggen med historikomskrivning) |
| `l` | Öppna LFS-menyn. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch alla / checka ut alla / pull / push alla / rensa. Se [Git LFS](lfs.md) |
| `i` | ✦ AI: skriv commit-meddelandet för de stageade ändringarna |
| `w` | ✦ AI: förklara den markerade commiten, eller ändringarna i arbetskatalogen |
| `Shift+C` | ✦ AI: föreslå hur ändringarna i arbetskatalogen delas upp i commits |

AI-tangenterna fungerar bara när AI är på (se [AI-hjälp](ai.md)). `Esc` avbryter
en pågående AI-förfrågan.

## Högerpanelen (ändringar i arbetskatalogen och commit-detaljer)

När WIP-raden är markerad visar högerpanelen listorna **Unstaged** och
**Staged** och commit-composern. När en commit är markerad visar den
commit-detaljerna och dess filer.

| Tangent | Åtgärd |
|---|---|
| `j` / `k`, `Up` / `Down` | Flytta. Vid slutet av en lista fortsätter markeringen in i den andra |
| `h` / `l`, `Left` / `Right` | Gå till listan Unstaged / Staged |
| `Enter` | Fäll ihop eller ut den aktiva listan |
| `Space` / `Insert` | Lägga till den aktuella filen i markeringen, eller ta bort den |
| `Ctrl+A` | Markera alla filer i listan |
| `Ctrl+Space` | Stagea / unstagea den markerade filen, eller fäll en mapprad |
| `d` | Öppna den markerade filens diff |
| `m` | `Sökväg` (platt lista) eller `Träd` (mappar) |
| `a` | `Visa alla filer`: hela trädet, inte bara de ändrade filerna |
| `z` | Fäll ihop alla mappar, eller fäll ut dem igen |
| `/` | Filtrera en commits ändrade filer efter namn |
| `.`, `Menu`, `Shift+F10` | Filens meny |
| `c` | Fokusera commit-formulärets sammanfattningsfält |
| `Shift+A` | Slå på eller av **Amend föregående commit** |
| `Ctrl+N ]` / `Ctrl+N [` | Förstora / förminska commit-meddelandets block |

I commit-composern:

| Tangent | Åtgärd |
|---|---|
| `Enter` | Sammanfattning: gå till beskrivningen. Beskrivning: ny rad |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` gör det varifrån som helst) |
| `Alt+R` | Infoga en ärendereferens |
| `Esc` | Lämna composern |

Filmenyn innehåller: Stage eller Unstage, Förkasta ändringar, Ignorera, Stash
fil, Filhistorik, Fil-blame, Öppna i externt diff-verktyg, Öppna i extern
redigerare, Öppna filen i standardprogram, Visa i mapp, Kopiera filsökväg,
Skapa patch från filändringar, Redigera fil och Ta bort fil. På en fil med
konflikt börjar den med **Öppna i merge-redigeraren** och de tre raderna **Lös
med**. Med Git LFS på har den LFS-rader. **Förkasta ändringar** och **Ta bort
fil** frågar först.

## Fönsterknappar

Varje knapp har en tangentbordsmotsvarighet:

| Knapp | Tangent |
|---|---|
| 🐙 till vänster i titelraden (Om) | `Hjälp: Om GitPoulpi` i paletten |
| **Öppna i** i titelraden | `Alt+O`, `Ctrl+Alt+O` |
| `×` till höger i titelraden (avsluta) | `q` |
| Versionen i titelraden (Versionsanteckningar) | `Hjälp: Versionsanteckningar` i paletten |
| `⌕ Global sökning` i titelraden, till vänster om `⚙` | `Ctrl+Alt+F` |
| `⚙` i titelraden (Inställningar) | `Ctrl+,` |
| Arbetsytechipet i flikraden | `Alt+W` |
| `+` efter den sista fliken | `Alt+T` |
| Fliken **Radar** | `Alt+R` |
| Fliken **Arbetsytor** | `Alt+J` |
| **Aktivitet** på statusraden | `Ctrl+N Shift+A` |
| Verktygsfält: **Ångra**, **Gör om** | `Ctrl+Z`, `Ctrl+Y` |
| Verktygsfält: **Pull** / dess `▾` | `Ctrl+G p` / `Git: Pull-alternativ...` |
| Verktygsfält: **Push** / dess `▾` | `Ctrl+G Shift+P` / `Git: Push till remote...` |
| Verktygsfält: **Gren**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Verktygsfält: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Verktygsfält: **LFS** (när LFS är på) | `Ctrl+G l` |
| Verktygsfält: **Flow** (när repositoryt har ett arbetsflöde) | `Ctrl+G Shift+W` |
| Verktygsfält: **Sök** | `f` |
| Verktygsfält: den länkade pull requesten (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Verktygsfält: profillistan (med två eller fler profiler) | `Repository: Byt profil...` |

Verktygsfältet beskrivs i [Rundtur i gränssnittet](interface-tour.md#verktygsfält).

## Inställningar

`Ctrl+,`, `Ctrl+N ,`, kugghjulet `⚙` eller `App: Inställningar...` öppnar
skärmen Inställningar.

| Tangent | Åtgärd |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Flytta mellan sidlistan och alternativen |
| `j` / `k`, `Up` / `Down` | Flytta markeringen |
| `Enter` / `Space` | Öppna en sida, växla en kryssruta, redigera ett fält, öppna en rullgardinslista eller trycka på en knapp |
| `/`, `Ctrl+F` | Sök i alla alternativ |
| `Alt+Left` / `Alt+Right` | Byt underflik (Allmänt / listan) |
| `Esc` / `q` | Stäng Inställningar. Medan ett fält redigeras avbryter `Esc` bara redigeringen |
| `?` | Öppna hjälpen över skärmen Inställningar |

Se [Inställningar](settings.md).

## Textfält

Alla textfält delar samma tangenter: filter, sökfält, kommandopaletten,
prompter, commit-composern, WIP-radens fält, dialogfält och fält i
Inställningar. Medan ett fält har tangentbordet går dessa tangenter före
rutornas genvägar.

| Tangent | Åtgärd |
|---|---|
| ett tecken | Skriv det vid markören (ersätter markeringen) |
| `Left` / `Right` | Flytta ett tecken (med en markering: gå till dess början / slut) |
| `Ctrl+Left` / `Ctrl+Right` | Flytta ett ord |
| `Home` / `End` | Början / slutet av raden |
| `Shift` + en rörelse ovan | Utöka eller minska markeringen |
| `Up` / `Down` (bara beskrivningen) | Föregående / nästa rad |
| `Ctrl+Home` / `Ctrl+End` (bara beskrivningen) | Början / slutet av texten |
| `Ctrl+A` | Markera allt |
| `Backspace` / `Delete` | Radera tecknet före / efter markören, eller markeringen |
| `Ctrl+U` | Radera från markören till radens början (eller markeringen) |
| `Ctrl+K` | Radera från markören till radens slut (eller markeringen) |
| `Ctrl+C` / `Ctrl+X` | Kopiera / klipp ut markeringen (inte i ett lösenordsfält) |
| klistra in (terminalens egen inklistring) | Infoga vid markören och ersätt markeringen |

I commit-description skriver `Enter` en ny rad och `Ctrl+Enter` commitar.
Alla terminaler skickar inte `Ctrl+Enter` på samma sätt, så `Alt+Enter` och
`Ctrl+J` commitar också.

Vissa konsoler skickar `Ctrl+bokstav` som ett kontrolltecken (`Ctrl+A` som
`0x01`). GitPoulpi förstår båda formerna. `AltGr`-tecken (`@`, `#`, `€` på
schweiziska eller franska tangentbord) skrivs in.

## Tooltips

En tooltip visas när pekaren stannar ungefär 0,6 s på ett mål (en Log-cell,
en verktygsfältsknapp, den länkade pull requesten). Ett tangenttryck, ett klick,
en rullning, en storleksändring, en öppen popup eller meny, eller att pekaren
lämnar målet döljer den. Efter det kommer den tillbaka först när pekaren rör
sig igen.
