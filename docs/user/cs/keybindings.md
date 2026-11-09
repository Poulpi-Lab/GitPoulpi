[English](../keybindings.md) · **Čeština** · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Klávesové zkratky

Tato stránka uvádí **výchozí** klávesy. Většinu z nich můžete přemapovat.
Aplikace vždy ukazuje klávesy, které platí: v paletě příkazů, v nápovědě `?`,
ve stavových řádcích, v kontextových nabídkách a v nápovědách tlačítek. Když klávesu
přesunete, tato místa ukážou novou klávesu. K myši viz
[Myš a gesta](mouse-and-gestures.md).

## Přemapování

Otevřete **Nastavení > Klávesnice a myš**. Stránka je popsána v
[Nastavení](settings.md#klávesnice-a-myš). Změny se použijí okamžitě a uloží se
do tabulky `[keybindings]` v `config.toml`. Tuto tabulku můžete také upravit
ručně:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Syntaxe, pravidla konfliktů a varování jsou v
[Referenční příručce konfigurace](config-reference.md) u `[keybindings]`. Ve zkratce:

- Chyba se ohlásí ve stavovém řádku a výchozí klávesa zůstane.
- Dvě akce na jedné kombinaci ve stejném kontextu se odmítnou.
- Předpona potřebuje `Ctrl` nebo `Alt`.
- Kombinace panelu, která zakrývá globální kombinaci, vyvolá jen varování.

Tabulka níže uvádí každou klávesu, kterou lze přemapovat. Některé klávesy jsou
pevné a přemapovat je nelze: pohyb v seznamech (`j` `k` `h` `l`, šipky,
`PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, číslice (předpona
počtu), klávesy pro úpravu textu a klávesy `Enter` / `Tab` / šipky vyskakovacích
oken. Paleta má `Aplikace: Klávesové zkratky...` a `Zkratky: Obnovit vše...`.

## Výchozí klávesové zkratky

Každá klávesa, kterou můžete přemapovat, s identifikátorem, který použijete v
`[keybindings.<kontext>]`, a její výchozí hodnotou. Tabulka je generována z
aplikace (test ji udržuje aktuální). Klávesy za předponou jsou uvedeny s
předponou (`Ctrl+G s`).

<!-- keymap:begin -->
| Kontext | ID akce | Co dělá | Výchozí klávesy |
|---|---|---|---|
| `global` | `git_prefix` | Předpona akcí Git | `Ctrl+G` |
| `global` | `nav_prefix` | Předpona navigace | `Ctrl+N` |
| `global` | `palette` | Paleta příkazů | `Ctrl+P` |
| `global` | `settings` | Nastavení | `Ctrl+,` |
| `global` | `home` | Domů (správa repozitářů) | `Ctrl+O` |
| `global` | `clone` | Klonovat repozitář | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nová karta | `Alt+T` |
| `global` | `workspace_switch` | Přepnout pracovní prostor | `Alt+W` |
| `global` | `radar` | Otevřít Radar | `Alt+R` |
| `global` | `workspaces` | Otevřít pracovní prostory | `Alt+J` |
| `global` | `tab_menu` | Nabídka karty | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Otevřít v (seznam) | `Alt+O` |
| `global` | `tab_open_in_default` | Otevřít ve výchozí položce | `Ctrl+Alt+O` |
| `global` | `close_tab` | Zavřít kartu | `Ctrl+W` |
| `global` | `next_tab` | Další karta | `Ctrl+PageDown` |
| `global` | `prev_tab` | Předchozí karta | `Ctrl+PageUp` |
| `global` | `help` | Nápověda | `?` |
| `global` | `quit` | Ukončit | `q` |
| `global` | `refresh` | Obnovit | `r` |
| `global` | `refresh_all` | Obnovit (fetch, pull requesty, issues) | `F5` |
| `global` | `find_in_log` | Hledat v grafu commitů | `f` |
| `global` | `workspace_search` | Hledat v pracovním prostoru | `Ctrl+Alt+F` |
| `global` | `select_tab` | Vybrat kartu | `t` |
| `global` | `undo` | Vrátit poslední operaci | `Ctrl+Z` |
| `global` | `redo` | Zopakovat vrácenou operaci | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Otevřít historii vrácení | `Alt+Z` |
| `global` | `commit_column` | Zobrazit nebo skrýt sloupec commitů | `Alt+C` |
| `global` | `undo_offer` | Obnovit větev tam, kde ji nechala blokovaná akce | `Alt+Shift+Z` |
| `global` | `context_menu` | Kontextová nabídka | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Napsat zprávu commitu | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Hned commitnout změny ze stage | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI: Rozdělit změny na commity | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI: Vysvětlit změny | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash se zprávou | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI: Napsat zprávu commitu | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage všechny změny | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Zahodit všechny změny | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage vybraný soubor | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage vybraný soubor | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Otevřít propojený pull request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push všech tagů | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch všech remote | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Nový tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Nová větev | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash změn | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick vybraného commitu | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert vybraného commitu | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Upravit zprávu commitu | `Ctrl+G e` |
| `git_prefix` | `amend` | Upravit vybraný commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Upravit autora vybraného commitu | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) na vybraný commit | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase na vybranou větev | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop stashe | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Smazat vybraný ref nebo drop stashe | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Otevřít nabídku LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Otevřít nabídku Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch všech objektů | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout všech souborů | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull objektů | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push všech objektů | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Vyčistit lokální objekty | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Otevřít nástroj pro přepis historie | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Sbalit nebo obnovit panel Refs | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Sbalit nebo obnovit panel Stav | `Ctrl+N l` |
| `nav_prefix` | `settings` | Otevřít nastavení | `Ctrl+N ,` |
| `nav_prefix` | `about` | Otevřít „O aplikaci“ | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Otevřít Protokol aktivity | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Otevřít globální protokol aktivity | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Zobrazit úlohy na pozadí | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Zobrazit panel aktualizací | `Ctrl+N u` |
| `nav_prefix` | `help` | Přepnout nápovědu | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Vybrat kartu | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Změnit motiv | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Změnit jazyk | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Změnit barevný motiv grafu | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Přepnout zbarvení řádků grafu | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Zvětšit sekci Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Zmenšit sekci Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Zvětšit blok zprávy commitu | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Zmenšit blok zprávy commitu | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Rozšířit panel Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Zúžit panel Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Rozšířit panel Stav | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Zúžit panel Stav | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent: Zobrazit čekající pohled | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent: Zpět (nebo zavřít čekající pohled) | `Ctrl+N Backspace` |
| `refs` | `filter` | Filtrovat refs podle názvu | `/` |
| `refs` | `solo` | Solo pro výběr | `o` |
| `refs` | `clear_solo` | Ukončit solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Přepnout hierarchii větví | `y` |
| `refs` | `show_all` | Zobrazit všechny skryté refs | `Shift+H` |
| `refs` | `reveal` | Ukázat v grafu | `g` |
| `refs` | `mark_drop` | Označit jako zdroj přetažení | `m` |
| `refs` | `drop_marked` | Pustit sem označený ref | `Shift+M` |
| `refs` | `expand_all` | Rozbalit všechny složky | `=` |
| `refs` | `collapse_all` | Sbalit všechny složky | `-` |
| `refs` | `expand_below` | Rozbalit složku a její podsložky | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Sbalit složku a její podsložky | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Soubory: Cesta nebo strom | `m` |
| `log` | `view_all_files` | Soubory: Zobrazit všechny soubory | `a` |
| `log` | `compare_selection` | Porovnat vybrané řádky | `v` |
| `log` | `compare_swap` | Prohodit konce porovnání | `x` |
| `log` | `compare_next` | Přejít na další porovnávaný řádek | `n` |
| `log` | `pick_column` | Vybrat sloupec k přesunu nebo změně šířky | `Shift+C` |
| `log` | `scroll_left` | Posunout sloupce doleva | `Shift+Left` |
| `log` | `scroll_right` | Posunout sloupce doprava | `Shift+Right` |
| `log` | `copy_mermaid` | Zkopírovat commity jako Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Vybrat další sloupec | `Tab` |
| `log_columns` | `prev` | Vybrat předchozí sloupec | `Shift+BackTab` |
| `log_columns` | `move_left` | Přesunout sloupec doleva | `<` |
| `log_columns` | `move_right` | Přesunout sloupec doprava | `>` |
| `log_columns` | `wider` | Rozšířit sloupec | `]` |
| `log_columns` | `narrower` | Zúžit sloupec | `[` |
| `log_columns` | `reset_width` | Obnovit šířky sloupců | `0` |
| `log_columns` | `fit` | Přizpůsobit sloupec obsahu | `=` |
| `log_columns` | `fit_all` | Přizpůsobit všechny sloupce obsahu | `+` |
| `log_columns` | `layout` | Posouvat do stran nebo přizpůsobit panelu | `s` |
| `log_files` | `collapse_all` | Sbalit nebo rozbalit všechny složky | `z` |
| `log_files` | `filter` | Filtrovat změněné soubory | `/` |
| `log_files` | `expand_all` | Rozbalit všechny složky | `=` |
| `log_files` | `collapse_all_folders` | Sbalit všechny složky | `-` |
| `log_files` | `expand_below` | Rozbalit složku a její podsložky | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Sbalit složku a její podsložky | `_`, `Shift+Left` |
| `status` | `compose` | Přejít na zprávu commitu | `c` |
| `status` | `toggle_file_mode` | Soubory: Cesta nebo strom | `m` |
| `status` | `view_all_files` | Soubory: Zobrazit všechny soubory | `a` |
| `status` | `amend` | Přepnout Amend | `Shift+A` |
| `status` | `skip_hooks` | Přepnout Přeskočit hooky | `Shift+H` |
| `status` | `collapse_all` | Sbalit nebo rozbalit všechny složky | `z` |
| `status` | `stage_toggle` | Stage nebo unstage vybraný řádek | `Ctrl+Space` |
| `status` | `open_diff` | Otevřít diff vybraného souboru | `d` |
| `status` | `expand_all` | Rozbalit všechny složky | `=` |
| `status` | `collapse_all_folders` | Sbalit všechny složky | `-` |
| `status` | `expand_below` | Rozbalit složku a její podsložky | `*`, `Shift+Right` |
| `status` | `collapse_below` | Sbalit složku a její podsložky | `_`, `Shift+Left` |
| `home` | `favorite` | Přidat repozitář k oblíbeným nebo odebrat | `f`, `*` |
| `home` | `close_repo` | Zavřít, odebrat nebo odznačit řádek | `x`, `Delete` |
| `home` | `browse` | Vyhledat repozitář | `b` |
| `home` | `clone` | Klonovat repozitář | `c` |
| `home` | `init` | Inicializovat repozitář | `i` |
| `home` | `collapse_all` | Sbalit všechny sekce | `-` |
| `home` | `expand_all` | Rozbalit všechny sekce | `+`, `=` |
| `home` | `search` | Hledat v repozitářích | `/` |
| `home` | `workspace_new` | Nový pracovní prostor | `n` |
| `home` | `workspace_edit` | Upravit pracovní prostor | `e` |
| `home` | `workspace_from_tabs` | Nový prostor z otevřených karet | `a` |
| `home` | `workspace_fetch` | Fetch vše v pracovním prostoru | `Shift+F` |
| `home` | `workspace_pull` | Pull vše v pracovním prostoru | `Shift+P` |
| `home` | `workspace_clone_missing` | Naklonovat chybějící repozitáře | `m` |
| `home` | `workspace_locate` | Najít chybějící repozitář | `l` |
| `refs` | `hosting_filter` | Pull requesty / Issues: Zvolit filtr | `v` |
| `refs` | `hosting_edit` | Pull request: Upravit u hostitele | `e` |
| `refs` | `new_pull_request` | Pull requesty: Vytvořit | `n` |
| `refs` | `add_remote` | Remotes: Přidat remote | `+` |
| `diff` | `close` | Zavřít diff | `q` |
| `diff` | `edit` | Upravit v pracovním adresáři | `e` |
| `diff` | `lfs_raw` | LFS: Karta nebo surový ukazatel | `r` |
| `diff` | `copy` | Kopírovat výběr nebo řádek | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage nebo unstage hunk | `Space` |
| `diff` | `stage_lines` | Stage nebo unstage vybrané řádky | `Enter` |
| `diff` | `discard_hunk` | Zahodit hunk | `x` |
| `diff` | `discard_lines` | Zahodit vybrané řádky | `Shift+X` |
| `diff` | `select_lines` | Začít nebo zrušit výběr řádků | `v` |
| `diff` | `next_hunk` | Další hunk | `n` |
| `diff` | `prev_hunk` | Předchozí hunk | `p` |
| `diff` | `toggle_layout` | Rozvržení: Sjednocené nebo vedle sebe | `m` |
| `diff` | `toggle_hunks_only` | Jen hunky nebo celý soubor | `Shift+H` |
| `diff` | `toggle_minimap` | Přepnout minimapu | `Shift+M` |
| `diff` | `choose_encoding` | Vybrat kódování | `Shift+C` |
| `diff` | `toggle_wrap` | Přepnout zalamování řádků | `w` |
| `diff` | `toggle_whitespace` | Zobrazit nebo skrýt změny jen v bílých znacích | `s` |
| `diff` | `merge_editor` | Otevřít editor merge (soubor v konfliktu) | `Shift+E` |
| `file_insight` | `close` | Zavřít pohled na soubor | `q` |
| `file_insight` | `edit` | Upravit v pracovním adresáři | `e` |
| `file_insight` | `copy` | Kopírovat výběr nebo řádek | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Přejít na poslední řádek | `Shift+G` |
| `file_insight` | `open_commit` | Přejít na commit řádku | `Enter` |
| `history` | `show_in_log` | Zobrazit v Logu | `Enter` |
| `history` | `copy_sha` | Kopírovat SHA commitu | `y` |
| `history` | `copy_summary` | Kopírovat shrnutí | `Shift+Y` |
| `history` | `show_diff` | Zobrazit diff tohoto commitu | `d`, `Alt+1` |
| `history` | `open_file` | Otevřít soubor u tohoto commitu | `o`, `Alt+2` |
| `history` | `blame` | Blame u tohoto commitu | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Porovnat s pracovním stromem | `c` |
| `history` | `restore` | Obnovit tuto verzi | `Shift+R` |
| `history` | `range_up` | Rozšířit výběr nahoru | `Shift+Up` |
| `history` | `range_down` | Rozšířit výběr dolů | `Shift+Down` |
| `history` | `switch_focus` | Přepnout mezi seznamem a diffem | `Tab` |
| `history` | `widen_list` | Rozšířit seznam | `Alt+Right` |
| `history` | `narrow_list` | Zúžit seznam | `Alt+Left` |
| `history` | `reset_split` | Obnovit šířku seznamu | `Alt+0` |
| `commit_column` | `scroll_left` | Posunout sloupec doleva | `Shift+Left` |
| `commit_column` | `scroll_right` | Posunout sloupec doprava | `Shift+Right` |
| `commit_column` | `next_part` | Vybrat další část | `Tab` |
| `commit_column` | `move_left` | Přesunout část doleva | `<` |
| `commit_column` | `move_right` | Přesunout část doprava | `>` |
| `commit_column` | `wider_part` | Rozšířit část | `]` |
| `commit_column` | `narrower_part` | Zúžit část | `[` |
| `commit_column` | `reset_widths` | Obnovit šířky částí | `0` |
| `file_insight` | `blame_card` | Blame: Zobrazit nebo skrýt kartu s detaily | `i` |
| `file_insight` | `blame_next` | Blame: Další hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Předchozí hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Další hunk stejného commitu | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Předchozí hunk stejného commitu | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Přejít na řádek | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Kopírovat SHA commitu | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Zobrazit diff commitu pro soubor | `d` |
| `file_insight` | `blame_parent` | Blame: Blame na rodiči commitu | `b` |
| `file_insight` | `blame_history` | Blame: Otevřít historii souboru | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Rozšířit sloupec anotací | `>` |
| `file_insight` | `blame_narrower` | Blame: Zúžit sloupec anotací | `<` |
| `image` | `zoom_in` | Přiblížit obrázek | `+`, `=` |
| `image` | `zoom_out` | Oddálit obrázek | `-` |
| `image` | `fit` | Přizpůsobit obrázek zobrazení | `0` |
| `image` | `actual_size` | Zobrazit obrázek ve skutečné velikosti | `1` |
| `image` | `flip_side` | Zobrazit druhou stranu (před / po) | `Tab` |
| `image` | `widen_before` | Rozšířit panel Před | `Alt+Right` |
| `image` | `widen_after` | Rozšířit panel Po | `Alt+Left` |
| `image` | `reset_split` | Obnovit rozdělení před / po | `Alt+0` |
| `image` | `edit_in` | Otevřít pomocí... | `Shift+O` |
| `image` | `edit_default` | Otevřít ve výchozí aplikaci | `o` |
| `editor` | `save` | Uložit | `Ctrl+S` |
| `editor` | `undo` | Zpět | `Ctrl+Z` |
| `editor` | `redo` | Znovu | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Hledat | `Ctrl+F` |
| `editor` | `find_next` | Hledat další (Shift: předchozí) | `F3` |
| `editor` | `paste` | Vložit | `Ctrl+V` |
| `editor` | `toggle_minimap` | Přepnout minimapu | `Alt+M` |
| `editor_prompt` | `save` | Neuložené změny: Uložit a zavřít | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Neuložené změny: Zahodit a zavřít | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Neuložené změny: Dál upravovat | `c`, `Shift+C` |
| `merge` | `next_conflict` | Další konflikt | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Předchozí konflikt | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Další soubor v konfliktu | `Alt+PageDown` |
| `merge` | `prev_file` | Předchozí soubor v konfliktu | `Alt+PageUp` |
| `merge` | `next_pane` | Přejít na další panel | `F6` |
| `merge` | `prev_pane` | Přejít na předchozí panel | `Shift+F6` |
| `merge` | `toggle_panes` | Zobrazit nebo skrýt boční panely (Refs a Stav) | `F9` |
| `merge` | `toggle_base` | Zobrazit nebo skrýt panel Základ | `Alt+D` |
| `merge` | `toggle_full_file` | Přepnout diff celého souboru | `Alt+F` |
| `merge` | `undo` | Zpět (výběry a psaní) | `Ctrl+Z` |
| `merge` | `redo` | Znovu (výběry a psaní) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Rozšířit panel Aktuální (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Rozšířit panel Příchozí (B) | `Alt+Left` |
| `merge` | `reset_panes` | Obnovit velikosti panelů | `Alt+0` |
| `merge` | `save` | Uložit bez označení jako vyřešené | `Alt+S` |
| `merge` | `save_resolve` | Uložit a označit jako vyřešené | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Navrhnout řešení (AI) | `Alt+I` |
| `merge` | `unpick` | Zrušit výběr tohoto konfliktu (zpět na nevyřešené) | `Alt+U` |
| `merge` | `reset_all` | Obnovit celý soubor na původní konflikty | `Alt+R` |
| `merge` | `focus_a` | Přejít na panel Aktuální (A) | `Alt+1` |
| `merge` | `focus_b` | Přejít na panel Příchozí (B) | `Alt+2` |
| `merge` | `focus_output` | Přejít na panel Výstup | `Alt+3` |
| `merge` | `focus_base` | Přejít na panel Základ | `Alt+4` |
| `merge_sides` | `toggle_line` | Vybrat nebo zrušit řádek | `Space` |
| `merge_sides` | `toggle_chunk` | Vybrat nebo zrušit celý blok | `Enter` |
| `merge_sides` | `take_a` | Vzít jen Aktuální (A) | `a` |
| `merge_sides` | `take_b` | Vzít jen Příchozí (B) | `b` |
| `merge_sides` | `take_both` | Vzít obojí, A pak B | `c` |
| `merge_sides` | `take_both_reversed` | Vzít obojí, B pak A | `Shift+C` |
| `merge_sides` | `clear` | Nevzít pro tento konflikt nic | `x` |
| `merge_sides` | `unpick` | Zrušit výběr tohoto konfliktu (zpět na nevyřešené) | `u` |
| `merge_sides` | `reset_all` | Obnovit celý soubor na původní konflikty | `Shift+U` |
| `merge_sides` | `swap_order` | Prohodit pořadí obou stran | `s` |
| `merge_sides` | `next_conflict` | Další konflikt | `n` |
| `merge_sides` | `prev_conflict` | Předchozí konflikt | `Shift+N` |
| `merge_sides` | `take_all_a` | Vzít vše z Aktuální (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Vzít vše z Příchozí (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Zobrazit nebo skrýt panel Základ | `d` |
| `merge_sides` | `scroll_left` | Posunout panely doleva | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Posunout panely doprava | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Přepnout diff celého souboru | `f` |
| `merge_sides` | `cycle_height` | Přepnout výšku Výstupu | `z` |
| `merge_card` | `take_a` | Ponechat A (Aktuální) beze změny | `a` |
| `merge_card` | `take_b` | Vzít B (Příchozí) beze změny | `b` |
| `merge_card` | `take_base` | Vzít Základ (společný předek) | `o` |
| `merge_card` | `keep` | Ponechat zbylý soubor | `k` |
| `merge_card` | `delete` | Přijmout smazání | `Shift+D` |
| `merge_card` | `external_tool` | Otevřít v externím nástroji merge | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (zastavit kvůli amend) | `e` |
| `rebase` | `autosquash` | Přepnout autosquash | `a` |
| `rebase` | `start` | Spustit plán | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Obnovit plán | `Ctrl+R` |
| `rebase` | `split` | Rozdělit Commit na řádku | `t` |
| `rebase` | `move_up` | Posunout řádek nahoru | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Posunout řádek dolů | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Vytvořit commity | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Přepnout výběr po řádcích | `v` |
| `split` | `move_up` | Posunout Commit nahoru | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Posunout Commit dolů | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Obnovit rozdělení | `Shift+R` |
| `rewrite` | `preview` | Náhled přepisu | `F5` |
| `rewrite` | `apply` | Provést přepis | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Zrušit náhled | `Ctrl+X` |
| `rewrite` | `add_rule` | Přidat pravidlo | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Zapnout nebo vypnout pravidlo | `Space` |
| `rewrite` | `move_up` | Posunout pravidlo nahoru | `Alt+Up` |
| `rewrite` | `move_down` | Posunout pravidlo dolů | `Alt+Down` |
| `rewrite` | `delete_rule` | Smazat pravidlo | `d`, `Delete` |
| `rewrite` | `restore_rule` | Obnovit smazané pravidlo | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplikovat pravidlo | `Ctrl+D` |
| `rewrite` | `next_tab` | Další karta náhledu | `Alt+Right` |
| `rewrite` | `prev_tab` | Předchozí karta náhledu | `Alt+Left` |
| `rewrite` | `save_shared` | Uložit sadu do repozitáře | `Alt+S` |
| `rewrite` | `save_mine` | Uložit sadu jen pro mě | `Alt+M` |
| `rewrite` | `load` | Načíst sadu | `Alt+L` |
| `rewrite` | `name` | Přejmenovat sadu | `F2` |
| `rewrite` | `protected_all` | Zahrnout všechny chráněné větve | `Alt+A` |
| `rewrite` | `protected_none` | Vynechat všechny chráněné větve | `Alt+U` |
| `rewrite` | `check_tools` | Znovu zkontrolovat nástroje | `F6` |
| `rewrite` | `analyze` | Analyzovat velikosti historie | `F7` |
| `rewrite` | `host_howto` | Vysvětlit ochranu větví u hostitele | `Alt+H` |
| `rewrite` | `ai_rules` | Vytvořit pravidla přepisu pomocí AI | `Alt+I` |
| `reword` | `apply` | Použít novou zprávu | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Upravit historii hlouběji | `Alt+H` |
| `composer` | `commit` | Commit z Composeru | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Vložit odkaz na issue | `Alt+R` |
| `dialog` | `browse` | Vybrat složku | `Ctrl+B` |
| `dialog` | `sources` | Přejít na zdroje klonování | `Alt+S` |
| `dialog` | `submit` | Odeslat formulář pro pull request | `Ctrl+S` |
| `dialog` | `web` | Otevřít webový formulář poskytovatele | `Alt+W` |
| `folder_picker` | `choose` | Zvolit složku | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nová složka | `Alt+N` |
| `folder_picker` | `filter` | Filtrovat položky složky | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Zobrazit nebo skrýt sousední složky | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Zobrazit nebo skrýt skryté složky | `Alt+H` |
| `preferences` | `search` | Hledat v nastavení | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Přidat položku seznamu | `Insert` |
| `preferences` | `list_duplicate` | Duplikovat vybranou položku seznamu | `Ctrl+D` |
| `preferences` | `list_remove` | Odebrat vybranou položku seznamu | `Delete` |
| `preferences` | `list_move_up` | Posunout vybranou položku seznamu nahoru | `Alt+Up` |
| `preferences` | `list_move_down` | Posunout vybranou položku seznamu dolů | `Alt+Down` |
| `preferences` | `prev_tab` | Předchozí karta stránky nastavení | `Alt+Left` |
| `preferences` | `next_tab` | Další karta stránky nastavení | `Alt+Right` |
| `ai_popup` | `copy` | Kopírovat výsledek | `c` |
| `activity_log` | `scroll_left` | Scroll the Activity Log Left | `Left` |
| `activity_log` | `scroll_right` | Scroll the Activity Log Right | `Right` |
| `integrations` | `add` | Přidat profil | `a` |
| `integrations` | `edit` | Upravit vybraný profil | `e` |
| `integrations` | `open_token` | Otevřít stránku tokenu | `o` |
| `integrations` | `remove` | Odebrat vybraný profil | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI: Vygenerovat zprávu stashe | `Ctrl+G` |
<!-- keymap:end -->

### Klávesy, které terminál neumí poslat, a klávesnicový protokol kitty

Starší terminál neumí poslat některé kombinace: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` a `Ctrl+<číslice>`. Nepošle nic, nebo jinou klávesu: například
`Ctrl+Shift+N` dorazí jako `Ctrl+N`, `Ctrl+I` jako `Tab` a `Ctrl+M` jako
`Enter`. Stránka Nastavení před takovou vazbou varuje („tento terminál
neposílá ...“).

Terminály s **klávesnicovým protokolem kitty** tyto kombinace poslat umějí.
Příklady: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 a novější a tmux nebo
herdr, když protokol propouštějí. GitPoulpi protokol používá ve výchozím stavu:

- Při startu se GitPoulpi zeptá terminálu a čeká na odpověď nejvýše asi 100 ms.
  Pokud terminál neodpoví, zůstanou starší klávesy.
- Pokud terminál odpoví, GitPoulpi protokol zapne. Při ukončení, selhání nebo
  pádu jej opět vypne.
- Varování na stránce Nastavení protokol sledují: když je zapnutý, výše uvedené
  kombinace lze poslat.
- **Nastavení > Klávesnice a myš > Klávesnicový protokol Kitty**
  (`ui.kitty_keyboard`) nastavené na `off` nikdy nemění klávesnicový režim
  terminálu. Použijte to, pokud se terminál nebo multiplexer chová špatně.
  Je potřeba restart.

Konzole Windows hlásí modifikátory samy a protokol nepotřebují.

## Paleta příkazů (`Ctrl+P`)

Paleta uvádí **každý** příkaz aplikace, takže dosáhne na vše bez zkratky. Úplný
seznam je [níže](#všechny-příkazy-palety).

| Klávesa | Akce |
|---|---|
| `Ctrl+P` | Otevřít paletu |
| *(psaní)* | Filtrovat seznam. Nejdřív přijde přesná shoda za předponou `Kategorie: `, poté shoda kdekoli v popisku, poté přibližná shoda |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Posunout zvýraznění (viz [Filtrovatelné vyskakovací seznamy](#filtrovatelné-vyskakovací-seznamy)) |
| `Enter`, nebo dvojklik | Spustit zvýrazněný příkaz |
| `Esc` | Zavřít |

`j` a `k` se píší do filtru. Zvýraznění neposouvají.

Každý řádek ukazuje vpravo svou zkratku. Příkaz, který se nyní nehodí (například
není vybrán žádný ref), zůstane v seznamu zašedlý a důvod se zobrazí pod
seznamem. Popisky začínají kategorií, například `Navigace:`, `Refs:`, `Git:`,
`Zobrazení:`, `Diff:` nebo `Aplikace:`. Napište kategorii a uvidíte celou její
skupinu.

### Klávesa pro libovolný příkaz

Každý řádek palety končí tlačítkem klávesy (`[k]`) a `Alt+K` dělá totéž na zvýrazněném řádku: otevře **Nastavení > Klávesnice a myš** filtrované podle zkratky daného příkazu, připravené k přeřazení. Příkaz bez vlastní zkratky ji tu dostane: je uveden v **Příkazy palety**, ve výchozím stavu bez klávesy, a klávesa, kterou mu přiřadíte, spustí příkaz přesně jako jeho výběr v paletě (příkaz, který teď nelze spustit, uvede důvod). Řádek pak ukazuje vaši klávesu.

V `config.toml` jde o tabulku `[keybindings.palette]` pojmenovanou podle příkazu (`toggle_minimap = "Ctrl+Alt+M"`). Klávesa, která koliduje s jinou akcí, je zobrazena červeně a `c` na jejím řádku vypíše všechny akce, se kterými koliduje.

Řádek nápovědy pod seznamem uvádí tuto klávesu (výchozí `Alt+K`). Je to akce `palette.configure_shortcut`, takže ji lze přeřadit jako kteroukoli jinou.

### Všechny příkazy palety

Generováno z aplikace (test to udržuje aktuální). Příkazy AI se objeví jen,
když je AI zapnutá; ostatní jsou zašedlé, s uvedeným důvodem, když se nehodí.
Klávesy jsou výchozí.

<!-- palette:begin -->
| Příkaz | Výchozí klávesa |
|---|---|
| Navigace: Přejít na panel Refs | `Ctrl+N <badge>` |
| Navigace: Přejít na panel Log | `Ctrl+N <badge>` |
| Navigace: Přejít na panel Stav | `Ctrl+N <badge>` |
| Navigace: Přejít na filtr Refs | `Ctrl+N <badge>` |
| Navigace: Přejít na Refs > Lokální | `Ctrl+N <badge>` |
| Navigace: Přejít na Refs > Remote | `Ctrl+N <badge>` |
| Navigace: Přejít na Refs > Worktrees | `Ctrl+N <badge>` |
| Navigace: Přejít na Refs > Submoduly | `Ctrl+N <badge>` |
| Navigace: Přejít na Refs > Stashes | `Ctrl+N <badge>` |
| Navigace: Přejít na Refs > Tags | `Ctrl+N <badge>` |
| Levý panel: Zobrazit/skrýt sekce... |  |
| Levý panel: Obnovit výchozí sekce |  |
| Levý panel: Použít tyto sekce pro všechny repozitáře |  |
| Levý panel: Zobrazit/skrýt lokální větve |  |
| Levý panel: Zobrazit/skrýt remote |  |
| Levý panel: Zobrazit/skrýt worktrees |  |
| Levý panel: Zobrazit/skrýt Stashes |  |
| Levý panel: Zobrazit/skrýt Tags |  |
| Levý panel: Zobrazit/skrýt Pull requesty |  |
| Levý panel: Zobrazit/skrýt issues |  |
| Levý panel: Zobrazit/skrýt submoduly |  |
| Navigace: Přejít na Stav > Unstaged | `Ctrl+N <badge>` |
| Navigace: Přejít na Stav > Staged | `Ctrl+N <badge>` |
| Navigace: Fokus na další panel | `Tab` |
| Navigace: Fokus na předchozí panel | `Shift+Tab` |
| Navigace: Hledat commity... | `f` |
| Navigace: Sbalit/rozbalit panel Refs | `Ctrl+N h` |
| Navigace: Rozšířit panel Refs | `Ctrl+N Right` |
| Navigace: Zúžit panel Refs | `Ctrl+N Left` |
| Navigace: Rozšířit panel Stav | `Ctrl+N Shift+Left` |
| Navigace: Zúžit panel Stav | `Ctrl+N Shift+Right` |
| Navigace: Sbalit/rozbalit panel Stav | `Ctrl+N l` |
| Navigace: Zvětšit sekci Refs | `Ctrl+N +` |
| Navigace: Zmenšit sekci Refs | `Ctrl+N -` |
| Navigace: Obnovit velikosti sekcí Refs |  |
| Navigace: Obnovit rozvržení |  |
| Detaily commitu: Zvětšit zprávu | `Ctrl+N ]` |
| Detaily commitu: Zmenšit zprávu | `Ctrl+N [` |
| Detaily commitu: Obnovit velikost zprávy |  |
| Detaily commitu: Posunout zprávu dolů |  |
| Detaily commitu: Posunout zprávu nahoru |  |
| Soubory: Přepnout řazení souborů commitu (A-Z / Z-A) |  |
| Refs: Checkout vybraného refu | `dvojklik` |
| Refs: Kopírovat název refu |  |
| Refs: Kopírovat úplný ref (refs/heads/...) |  |
| Refs: Kopírovat cílové SHA refu |  |
| Refs: Merge vybrané větve do aktuální |  |
| Refs: Rebase aktuální větve na vybranou | `Ctrl+G r` |
| Refs: Pustit vybraný ref na přepnutou větev... |  |
| Refs: Označit jako zdroj přetažení | `m` |
| Refs: Pustit sem označený ref... | `M` |
| Refs: Přejmenovat vybranou větev... |  |
| Refs: Fast-forward vybrané větve na aktuální |  |
| Refs: Fast-forward aktuální větve na vybranou |  |
| Refs: Vytvořit worktree z vybrané větve nebo tagu... |  |
| Refs: Reflog vybrané větve... |  |
| Refs: Založit pull request z vybrané větve... |  |
| Refs: Otevřít worktree na kartě | `Enter / dvojklik` |
| Refs: Aktualizovat submodul (vybraný) |  |
| Refs: Inicializovat submodul (vybraný) |  |
| Refs: Otevřít submodul na kartě | `Enter / dvojklik` |
| Git: Aktualizovat všechny submoduly |  |
| Git: Vytvořit worktree ve vybraném commitu... |  |
| Git: Vytvořit patch z vybraného commitu... |  |
| Git: Vytvořit patch od vybraného commitu do HEAD... |  |
| Git: Vytvořit patch ze změn pracovního stromu... |  |
| Přepis: Vymazat starou historii... |  |
| Git: Použít patch... |  |
| Přepis: Otevřít nástroj pro přepis historie | `Ctrl+G H` |
| Přepis: Nahradit autora... |  |
| Přepis: Anonymizovat autory... |  |
| Přepis: Použít .mailmap na historii... |  |
| Přepis: Merge identity (.mailmap)... |  |
| Přepis: Posunout data commitů... |  |
| Přepis: Změnit časové pásmo commitů... |  |
| Přepis: Omezit commity na určené hodiny... |  |
| Přepis: Nastavit datum committera podle autora |  |
| Přepis: Hledat a nahradit ve zprávách... |  |
| Přepis: Smazat řádky zpráv... |  |
| Přepis: Přidat nebo odebrat trailery... |  |
| Přepis: Odstranit podpisy... |  |
| Přepis: Opravit kódování zpráv... |  |
| Přepis: Hledat a nahradit v souborech... |  |
| Přepis: Odstranit soubory z historie... |  |
| Přepis: Odstranit velké soubory z historie... |  |
| Přepis: Přesunout nebo přejmenovat cesty... |  |
| Přepis: Převést soubory do LFS... |  |
| Přepis: Vrátit soubory z LFS... |  |
| Přepis: Převést konce řádků... |  |
| Přepis: Odebrat příznak spustitelnosti... |  |
| Přepis: Převést kódování souborů... |  |
| Přepis: Spustit skript... |  |
| ✦ Přepis: Napsat skript pomocí AI... |  |
| ✦ Přepis: Vytvořit pravidla pomocí AI... |  |
| Přepis: Od vybraného commitu po HEAD... |  |
| Přepis: Vybrané commity... |  |
| LFS: Inicializovat pro tento repozitář |  |
| LFS: Sledovat vzor... |  |
| LFS: Přestat sledovat vzor... |  |
| LFS: Fetch objektů |  |
| LFS: Otevřít nabídku | `Ctrl+G l` |
| LFS: Fetch všech objektů | `Ctrl+G F` |
| LFS: Checkout všech souborů | `Ctrl+G K` |
| LFS: Push všech objektů | `Ctrl+G J` |
| LFS: Pull objektů | `Ctrl+G L` |
| LFS: Push objektů |  |
| LFS: Pročistit lokální objekty | `Ctrl+G V` |
| LFS: Zobrazit zámky |  |
| LFS: Otevřít stránku instalace |  |
| LFS: Přepnout surový ukazatel v diffu | `r` |
| LFS: Sledovat vybraný soubor |  |
| LFS: Sledovat příponu vybraného souboru |  |
| LFS: Zamknout vybraný soubor |  |
| LFS: Odemknout vybraný soubor |  |
| LFS: Vynutit odemknutí vybraného souboru |  |
| Refs: Smazat vybranou větev nebo tag... | `Ctrl+G d` |
| Refs: Smazat vybranou větev z jejího remote... |  |
| Refs: Smazat vybranou větev lokálně i na remote... |  |
| Refs: Kopírovat odkaz na vybranou větev na remote |  |
| Git: Kopírovat odkaz na vybraný commit na remote |  |
| Git: Anotovaný tag ve vybraném commitu... |  |
| Refs: Push vybraného tagu do výchozího remote |  |
| Refs: Smazat vybraný tag z jeho remote... |  |
| Git: Fetch vybraného tagu |  |
| Refs: Anotovat vybraný tag... |  |
| Refs: Přepnout skrytí v grafu |  |
| Refs: Ukázat v grafu | `g` |
| Refs: Přepnout solo na vybraném refu | `o` |
| Refs: Ukončit solo (zrušit solo u všech refů) | `O` |
| Refs: Zobrazit všechny skryté refs | `H` |
| Refs: Přepnout skrytí vybrané složky/remote |  |
| Refs: Přepnout solo vybrané složky/remote | `o` |
| Refs: Filtrovat podle názvu... | `/` |
| Refs: Zrušit filtr názvu | `Esc` |
| Refs: Rozbalit všechny složky | `=` |
| Refs: Sbalit všechny složky | `-` |
| Refs: Rozbalit vybranou složku a její podsložky | `*` |
| Refs: Sbalit vybranou složku a její podsložky | `_` |
| Refs: Další sekce | `l` |
| Refs: Předchozí sekce | `h` |
| Refs: Rozbalit/sbalit aktivní sekci | `Enter` |
| Refs: Přepnout hierarchii větví | `y` |
| Git: Stage vybraný soubor | `Ctrl+G s` |
| Git: Unstage vybraný soubor | `Ctrl+G u` |
| Git: Stage všechny změny | `Ctrl+G a` |
| Změny: Zahodit vybraný soubor... |  |
| Změny: Ignorovat vybraný soubor |  |
| Změny: Ignorovat příponu vybraného souboru |  |
| Změny: Považovat vybraný soubor za binární... |  |
| Změny: Považovat vybraný soubor za textový... |  |
| Změny: Považovat příponu vybraného souboru za binární... |  |
| Změny: Považovat příponu vybraného souboru za textovou... |  |
| Změny: Stash vybraného souboru |  |
| Změny: Historie vybraného souboru |  |
| Změny: Blame vybraného souboru |  |
| Změny: Otevřít soubor v externím diffu |  |
| Změny: Otevřít soubor v externím editoru |  |
| Změny: Otevřít soubor ve výchozím programu |  |
| Změny: Zobrazit soubor ve složce |  |
| Změny: Kopírovat cestu souboru |  |
| Změny: Vytvořit patch ze souboru... |  |
| Změny: Upravit vybraný soubor |  |
| Změny: Smazat vybraný soubor... |  |
| Změny: Otevřít nabídku složky |  |
| Změny: Vytvořit soubor |  |
| Změny: Zahodit všechny změny... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Hned commitnout změny ze stage | `Ctrl+G Enter` |
| Git: Upravit zprávu WIP commitu (v Logu) | `c v Logu` |
| Git: Fetch všech remote | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward, pokud lze) |  |
| Git: Pull (jen fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Možnosti pullu... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push do remote... |  |
| Git: Force push (s lease)... |  |
| Git: Nastavit upstream... |  |
| Remote: Přidat remote... |  |
| Remote: Odebrat remote... |  |
| Remote: Upravit remote... |  |
| Remote: Nastavit výchozí remote... |  |
| Remote: Kopírovat název remote |  |
| Remote: Kopírovat odkaz na remote |  |
| Remote: Fetch remote... |  |
| Remote: Akce vybraného remote... | `.` |
| Repozitář: Klíč SSH pro tento repozitář... |  |
| Repozitář: Částečný checkout... |  |
| Flow: Otevřít nabídku | `Ctrl+G W` |
| Flow: Zahájit větev... |  |
| Flow: Nové issue... |  |
| Flow: Převzít větev hostitele... |  |
| Flow: Dokončit aktuální větev... |  |
| Flow: Synchronizovat s rodičem... |  |
| Flow: Povýšit... |  |
| Flow: Nastavit rodiče... |  |
| Flow: Pokračovat v zastavené akci nebo ji přerušit... |  |
| Flow: Přepnout seskupení větví |  |
| Flow: Nastavení workflow... |  |
| Remote: Skrýt remote v grafu... |  |
| Remote: Zobrazit remote v grafu... |  |
| Git: Nová větev na HEAD... | `Ctrl+G b` |
| Git: Nový tag na HEAD... | `Ctrl+G t` |
| Git: Push všech neodeslaných tagů | `Ctrl+G U` |
| Git: Stash všech změn | `Ctrl+G T` |
| Git: Stash všech změn se zprávou... | `Ctrl+G m` |
| Git: Stash pop | `Ctrl+G o` |
| Git: Stash apply |  |
| Git: Stash drop... |  |
| Git: Cherry-pick vybraného commitu na HEAD | `Ctrl+G y` |
| Git: Checkout vybraného commitu (odpojený HEAD) |  |
| Git: Vytvořit větev ve vybraném commitu... |  |
| Git: Vytvořit tag ve vybraném commitu... |  |
| Commit: Kopírovat zprávu |  |
| Commit: Kopírovat jméno autora |  |
| Commit: Kopírovat e-mail autora |  |
| Commit: Kopírovat datum |  |
| Commit: Kopírovat krátké SHA |  |
| Commit: Kopírovat úplné SHA |  |
| Git: Revert vybraného commitu... | `Ctrl+G v` |
| Git: Upravit zprávu commitu... | `Ctrl+G e` |
| Commit: Upravit... | `Ctrl+G E` |
| Commit: Upravit autora... | `Ctrl+G A` |
| Commit: Upravit souhrn... |  |
| Commit: Upravit popis... |  |
| Git: Rozdělit vybraný Commit... |  |
| Git: Interaktivní rebase od vybraného commitu... |  |
| Rebase: Pokračovat |  |
| Rebase: Přeskočit |  |
| Rebase: Přerušit |  |
| git am: Pokračovat |  |
| git am: Přeskočit tento patch |  |
| git am: Přerušit |  |
| Repozitář: Odstranit zastaralý index.lock |  |
| Banner: Pop čekajícího auto-stash |  |
| Banner: Dokončit čekající auto-stash (ponechat výsledek) |  |
| Banner: Zobrazit čekající auto-stash |  |
| Konflikty: Zobrazit předpokládané konflikty |  |
| Banner: Drop čekajícího auto-stash... |  |
| Banner: Začít znovu... |  |
| Git: Reset větve na vybraný commit (soft - ponechá index a pracovní strom) |  |
| Git: Reset větve na vybraný commit (mixed - ponechá pracovní strom) | `Ctrl+G R` |
| Git: Reset větve na vybraný commit (hard - zahodí vše)... |  |
| Git: Vyřešit konflikt pomocí ours (tato větev) |  |
| Git: Vyřešit konflikt pomocí theirs (příchozí) |  |
| Git: Vyřešit konflikt pomocí base (společný předek) |  |
| Konflikty: Otevřít editor merge | `Enter na souboru v konfliktu` |
| Merge: Další konflikt | `F7` |
| Merge: Předchozí konflikt | `Shift+F7` |
| Merge: Další soubor v konfliktu | `Alt+PageDown` |
| Merge: Předchozí soubor v konfliktu | `Alt+PageUp` |
| Merge: Vzít blok z A |  |
| Merge: Vzít blok z B |  |
| Merge: Vzít Aktuální pro tento konflikt | `a` |
| Merge: Vzít Příchozí pro tento konflikt | `b` |
| Merge: Vzít obojí, A pak B | `c` |
| Merge: Vzít obojí, B pak A | `C` |
| Merge: Vzít Základ pro tento konflikt |  |
| Merge: Vymazat tento konflikt | `x` |
| Merge: Prohodit pořadí | `s` |
| Merge: Vzít vše Aktuální | `A` |
| Merge: Vzít vše Příchozí | `B` |
| Merge: Upravit tento konflikt ve Výstupu |  |
| Merge: Zobrazit/skrýt boční panely | `F9` |
| Merge: Posunout panely doleva | `Shift+Left` |
| Merge: Posunout panely doprava | `Shift+Right` |
| Merge: Přepnout panel Základ | `Alt+D` |
| Merge: Přepnout diff celého souboru | `Alt+F` |
| Merge: Zpět | `Ctrl+Z` |
| Merge: Znovu | `Ctrl+Y` |
| Merge: Uložit bez označení jako vyřešené | `Alt+S` |
| Merge: Uložit a označit jako vyřešené | `Ctrl+S` |
| Merge: Obnovit Výstup na výsledek merge | `Alt+R` |
| Merge: Přepnout výšku Výstupu | `z` |
| Merge: Rozšířit panel Aktuální | `Alt+Right` |
| Merge: Rozšířit panel Příchozí | `Alt+Left` |
| Merge: Obnovit velikosti panelů | `Alt+0` |
| Merge: Zrušit výběr tohoto konfliktu | `Alt+U` |
| Merge: Fokus na panel Aktuální (A) | `Alt+1` |
| Merge: Fokus na panel Příchozí (B) | `Alt+2` |
| Merge: Fokus na panel Výstup | `Alt+3` |
| Merge: Fokus na panel Základ | `Alt+4` |
| Merge: Přejít na panel vlevo (stejný konflikt) |  |
| Merge: Přejít na panel vpravo (stejný konflikt) |  |
| Merge: Fokus na další panel | `F6` |
| Merge: Fokus na předchozí panel | `Shift+F6` |
| Merge: Pokračovat ze souboru na disku |  |
| Merge: Začít znovu od merge |  |
| Merge: Zrušit vyřešení a znovu otevřít |  |
| Merge: Načíst soubor znovu |  |
| Merge: Použít aktuální soubor beze změny | `a` |
| Merge: Použít příchozí soubor beze změny | `b` |
| Merge: Použít soubor Základ beze změny | `o` |
| Merge: Ponechat zbylý soubor | `k` |
| Merge: Přijmout smazání | `D` |
| Merge: Otevřít v externím nástroji merge | `m` |
| ✦ AI: Navrhnout řešení konfliktu merge | `Alt+I` |
| Merge: Zavřít editor merge |  |
| Git: Přerušit merge, cherry-pick nebo revert |  |
| Git: Přeskočit cherry-pick nebo revert |  |
| Repozitář: Otevřít nebo přepnout (správa repozitářů)... | `Ctrl+O` |
| Repozitář: Vyhledat repozitář... | `b (karta správy)` |
| Repozitář: Vytvořit nový (Init)... | `i (karta správy)` |
| Repozitáře: Otevřít vybraný | `Enter` |
| Repozitáře: Přidat/odebrat vybraný z oblíbených | `f` |
| Repozitáře: Zavřít nebo odebrat vybraný | `x` |
| Repozitáře: Nabídka vybraného repozitáře... | `.` |
| Repozitáře: Hledat | `/ / Ctrl+F` |
| Repozitáře: Sbalit všechny sekce | `-` |
| Repozitáře: Rozbalit všechny sekce | `+` |
| Repozitáře: Zavřít všechny karty... |  |
| Repozitáře: Odebrat vybraný ze seznamu |  |
| Repozitáře: Odebrat všechny nedávné |  |
| Pracovní prostor: Nový... | `n` |
| Pracovní prostor: Nový z otevřených karet... | `a` |
| Pracovní prostor: Přepnout... | `Alt+W` |
| Pracovní prostor: Upravit... | `e` |
| Pracovní prostor: Zavřít (žádný pracovní prostor) |  |
| Pracovní prostor: Spravovat... | `Alt+J` |
| Pracovní prostor: Zavřít kartu |  |
| Pracovní prostor: Najít chybějící repozitář... | `l` |
| Pracovní prostor: Smazat... |  |
| Pracovní prostor: Fetch vše | `F` |
| Pracovní prostor: Pull vše | `P` |
| Pracovní prostor: Naklonovat chybějící | `m` |
| Pracovní prostor: Přidat kartu do pracovního prostoru |  |
| Pracovní prostor: Odebrat kartu z pracovního prostoru |  |
| Radar: Otevřít | `Alt+R` |
| Radar: Obnovit | `r` |
| Radar: Hledat | `/` |
| Radar: Otevřít v prohlížeči | `o` |
| Radar: Otevřít repozitář | `Enter` |
| Radar: Checkout větve | `c` |
| Radar: Vytvořit worktree... | `t` |
| Radar: Zobrazit v repozitáři | `s` |
| Radar: Začít pracovat na issue... | `n` |
| Radar: Připnout nebo odepnout | `p` |
| Radar: Odložit... | `z` |
| Radar: Zkontrolovat žádost... | `e` |
| Radar: Zkontrolovat přes API |  |
| Radar: Naklonovat a zkontrolovat... |  |
| Radar: Upravit řešitele... |  |
| Radar: Upravit recenzenty... |  |
| Radar: Upravit štítky... |  |
| Radar: Nastavit milník... |  |
| Radar: Vybrat rozsah | `w` |
| Radar: Vybrat zobrazení | `v` |
| Radar: Vybrat filtr | `f` |
| Radar: Zobrazit nenaklonované žádosti | `a` |
| Radar: Zobrazit koncepty | `d` |
| Radar: Zapnout žádosti o kontrolu za celý účet... | `T` |
| Radar: Nabídka řádku... | `.` |
| Radar: Vybrat sloupec | `C` |
| Radar: Rozšířit sloupec | `]` |
| Radar: Zúžit sloupec | `[` |
| Radar: Obnovit šířky sloupců | `0` |
| Radar: Zpět na úvod | `Esc` |
| Radar: Zavřít kartu |  |
| Karta: Nová karta | `Alt+T` |
| Karta: Nová... |  |
| Karta: Nabídka akcí... | `Ctrl+T` |
| Karta: Zavřít ostatní karty... |  |
| Karta: Zavřít karty vpravo... |  |
| Karta: Přidat/odebrat z oblíbených |  |
| Karta: Kopírovat cestu repozitáře |  |
| Karta: Otevřít ve správci souborů |  |
| Karta: Otevřít v terminálu |  |
| Karta: Otevřít v... | `Alt+O` |
| Karta: Otevřít ve výchozí položce | `Ctrl+Alt+O` |
| Repozitář: Změnit profil... |  |
| Nápověda: O aplikaci GitPoulpi | `Ctrl+N a` |
| Nápověda: Licence a poděkování |  |
| Zobrazení: Protokol aktivity | `Ctrl+N A` |
| Zobrazení: Globální protokol aktivity | `Ctrl+N G` |
| Nápověda: Poznámky k vydání |  |
| Aktualizace: Zobrazit aktualizaci | `Ctrl+N u` |
| Aktualizace: Otevřít bezpečnostní upozornění |  |
| Aktualizace: Kopírovat příkaz aktualizace |  |
| Aktualizace: Spustit příkaz aktualizace v terminálu |  |
| Aktualizace: Zkontrolovat aktualizace nyní |  |
| Aktualizace: Stáhnout |  |
| Aktualizace: Aktualizovat nyní |  |
| Aktualizace: Restartovat pro aktualizaci |  |
| Aktualizace: Vrátit se k předchozí verzi |  |
| Aktualizace: Přeskočit tuto verzi |  |
| Poznámky k vydání: Zavřít kartu |  |
| Úlohy: zobrazit úlohy na pozadí | `Ctrl+N J` |
| Úlohy: zrušit vše, co jde zrušit |  |
| Zobrazení: Kopírovat diagnostiku |  |
| Protokol aktivity: zobrazit nebo skrýt úroveň Chyba |  |
| Protokol aktivity: zobrazit nebo skrýt úroveň Varování |  |
| Protokol aktivity: zobrazit nebo skrýt úroveň Info |  |
| Protokol aktivity: zobrazit nebo skrýt úroveň Ladění |  |
| Protokol aktivity: zobrazit nebo skrýt úroveň Trasování |  |
| Protokol aktivity: zobrazit všechny úrovně |  |
| Protokol aktivity: nezobrazovat žádnou úroveň |  |
| Protokol aktivity: Vymazat protokol aplikace |  |
| Protokol aktivity: Zvolit rozsah |  |
| Protokol aktivity: Operace gitu |  |
| Protokol aktivity: Protokol aplikace |  |
| Activity Log: Scroll Left | `Left` |
| Activity Log: Scroll Right | `Right` |
| Protokol aktivity: Kopírovat poslední chybu |  |
| Protokol aktivity: Provést navrženou akci |  |
| Protokol aktivity: Otevřít poslední chybu |  |
| Karta: Ukončit a cd sem |  |
| Repozitář: Klonovat... | `Ctrl+Shift+N` |
| Repozitář: Další karta | `Ctrl+PageDown` |
| Repozitář: Předchozí karta | `Ctrl+PageUp` |
| Karta: Přesunout doleva |  |
| Karta: Přesunout doprava |  |
| Repozitář: Zavřít kartu | `Ctrl+W` |
| Repozitář: Vybrat kartu... | `t` |
| Hledat: Pracovní prostor... | `Ctrl+Alt+F` |
| Úpravy: Zpět | `Ctrl+Z` |
| Úpravy: Znovu | `Ctrl+Y` |
| Úpravy: Zapomenout blokovanou položku vrácení |  |
| Úpravy: Historie vrácení... | `Alt+Z` |
| Úpravy: Reflog... |  |
| Úpravy: Zachránit ztracené commity... |  |
| Úpravy: Obnovit větev tam, kde ji nechala blokovaná akce | `Alt+Shift+Z` |
| Kontrola: Zobrazit přehled |  |
| Kontrola: Zobrazit soubory |  |
| Kontrola: Zobrazit Commity |  |
| Kontrola: Okomentovat řádek... | `c` |
| Kontrola: Okomentovat konverzaci... | `C` |
| Kontrola: Odpovědět ve vlákně... | `r` |
| Kontrola: Vyřešit nebo znovu otevřít vlákno | `x` |
| Kontrola: Sbalit nebo rozbalit vlákno | `z` |
| Kontrola: Upravit čekající komentář... | `e` |
| Kontrola: Smazat čekající komentář | `d` |
| Kontrola: Označit soubor jako zobrazený | `v` |
| Kontrola: Další soubor | `]` |
| Kontrola: Předchozí soubor | `[` |
| Kontrola: Dokončit kontrolu... | `f` |
| Kontrola: Zahodit čekající kontrolu... |  |
| Kontrola: Obnovit | `F5` |
| Kontrola: Otevřít v prohlížeči | `o` |
| Kontrola: Checkout větve | `b` |
| Kontrola: Zavřít | `Esc` |
| Kontrola: Upravit řešitele... | `a` |
| Kontrola: Upravit recenzenty... | `p` |
| Kontrola: Upravit štítky... | `l` |
| Kontrola: Nastavit milník... | `m` |
| Kontrola: Vybrat patch set... | `P` |
| Kontrola: Porovnat patch sety... | `D` |
| Kontrola: Submit změny... | `U` |
| Kontrola: Abandon změny... | `Q` |
| Kontrola: Přepnout sjednocené / vedle sebe | `V` |
| Kontrola: Přepnout celý soubor / jen hunky | `H` |
| Kontrola: Přepnout změny jen v bílých znacích | `S` |
| Kontrola: Přepnout zalamování řádků | `W` |
| Kontrola: Další hunk | `J` |
| Kontrola: Předchozí hunk | `K` |
| Kontrola: Vybrat kódování... | `E` |
| Kontrola: Přepnout minimapu | `M` |
| Kontrola: Posunout diff doleva | `Left` |
| Kontrola: Posunout diff doprava | `Right` |
| Kontrola: Kopírovat vybraný text nebo řádek | `Ctrl+C` |
| Tabule: Přesunout kartu do... | `m` |
| Tabule: Přesunout kartu doleva |  |
| Tabule: Přesunout kartu doprava |  |
| Tabule: Otevřít kartu | `Enter` |
| Tabule: Upravit řešitele... |  |
| Tabule: Upravit štítky... |  |
| Tabule: Nastavit milník... |  |
| Tabule: Nové issue v tomto sloupci | `n` |
| Tabule: Filtrovat podle řešitele... | `a` |
| Tabule: Filtrovat podle štítku... | `l` |
| Tabule: Filtrovat podle milníku... | `M` |
| Tabule: Filtrovat podle textu | `/` |
| Tabule: Vymazat filtry | `x` |
| Tabule: Nastavit štítky sloupců... | `c` |
| Tabule: Obnovit | `r` |
| Tabule: Zavřít | `Esc` |
| Kontrola: ✦ Koncept kontroly... | `s` |
| Kontrola: Přijmout návrh | `y` |
| Kontrola: Upravit a přijmout návrh... | `w` |
| Kontrola: Odmítnout návrh | `n` |
| Kontrola: Přijmout všechny návrhy | `A` |
| Kontrola: Odmítnout všechny návrhy | `N` |
| Kontrola: Použít shrnutí od AI v textu kontroly | `i` |
| Kontrola: ✦ Použít koncept... | `u` |
| Kontrola: Upravit text konceptu ✦... | `t` |
| Kontrola: Zahodit koncept ✦ | `N` |
| Historie vrácení: Vrátit až sem... | `u` |
| Historie vrácení: Zopakovat až sem... | `y` |
| Historie vrácení: Obnovit jen tuto... | `r` |
| Historie vrácení: Obnovit větev tam, kde byla... | `o` |
| Historie vrácení: Obnovit seznam |  |
| Historie vrácení: Zavřít | `Esc` |
| Reflog: Zobrazit v Logu | `s` |
| Reflog: Vytvořit větev tady... | `b` |
| Reflog: Obnovit na tuto pozici... | `r` |
| Reflog: Načíst další | `m` |
| Reflog: Obnovit seznam |  |
| Ztracená práce: Zobrazit v Logu | `s` |
| Ztracená práce: Vytvořit větev tady... | `b` |
| Ztracená práce: Cherry-pick | `c` |
| Ztracená práce: Hledat znovu |  |
| Zobrazení: Obnovit | `r` |
| Zobrazení: Obnovit (fetch, pull requesty, issues) | `F5` |
| Zobrazení: Přepnout nápovědu | `?` |
| Zobrazení: Přepnout sloupec Ref |  |
| Zobrazení: Přepnout sloupec Graf |  |
| Zobrazení: Přepnout sloupec Zpráva |  |
| Zobrazení: Přepnout sloupec Autor |  |
| Zobrazení: Přepnout sloupec Datum |  |
| Zobrazení: Přepnout sloupec Změny |  |
| Zobrazení: Přepnout sloupec SHA |  |
| Zobrazení: Sloupec Logu: Vybrat sloupec k přesunu nebo změně šířky | `C` |
| Zobrazení: Sloupec Logu: Vybrat další sloupec | `Tab` |
| Zobrazení: Sloupec Logu: Vybrat předchozí sloupec | `Shift+BackTab` |
| Zobrazení: Sloupec Logu: Přesunout sloupec doleva | `<` |
| Zobrazení: Sloupec Logu: Přesunout sloupec doprava | `>` |
| Zobrazení: Sloupec Logu: Rozšířit sloupec | `]` |
| Zobrazení: Sloupec Logu: Zúžit sloupec | `[` |
| Zobrazení: Sloupec Logu: Obnovit šířky sloupců | `0` |
| Zobrazení: Sloupec Logu: Přizpůsobit sloupec obsahu | `=` |
| Zobrazení: Sloupec Logu: Přizpůsobit všechny sloupce obsahu | `+` |
| Zobrazení: Log: Posunout sloupce doleva | `Shift+Left` |
| Zobrazení: Log: Posunout sloupce doprava | `Shift+Right` |
| Zobrazení: Log: Přepnout posouvání sloupců do stran | `s` |
| Zobrazení: Log: Vybrat zobrazené sloupce |  |
| Diff: Otevřít diff vybraného souboru | `Enter` |
| Diff: Zavřít zobrazení diffu | `Esc` |
| Diff: Další hunk | `n` |
| Diff: Předchozí hunk | `p` |
| Diff: Stage hunk pod kurzorem | `Space` |
| Diff: Unstage hunk pod kurzorem | `Space` |
| Diff: Vybrat řádky (nebo zrušit výběr) | `v` |
| Diff: Stage vybrané řádky | `Enter` |
| Diff: Unstage vybrané řádky | `Enter` |
| Diff: Zahodit hunk pod kurzorem... | `x` |
| Diff: Zahodit vybrané řádky... | `X` |
| Kopírovat: Vybraný text (nejdřív ho označte tažením) | `Ctrl+C` |
| Kopírovat: Řádek pod kurzorem (nebo vybrané řádky) | `y` |
| Diff: Kopírovat hunk pod kurzorem |  |
| Diff: Rozvržení sjednocené (+/- v jednom sloupci) | `m` |
| Diff: Rozvržení vedle sebe (před a po) | `m` |
| Diff: Přepnout jen hunky (celý soubor / jen změny) | `H` |
| Soubory: Přepnout cesta/strom | `m` |
| Soubory: Přepnout zobrazení všech souborů | `a` |
| Commit: Přepnout Přeskočit hooky | `H` |
| Soubory: Otevřít nabídku řádku nebo výběru | `.` |
| Soubory: Rozbalit všechny složky | `=` |
| Soubory: Sbalit všechny složky | `-` |
| Soubory: Rozbalit vybranou složku a její podsložky | `*` |
| Soubory: Sbalit vybranou složku a její podsložky | `_` |
| Soubory: Vybrat vše |  |
| Soubory: Zrušit výběr |  |
| Soubory: Přepnout výběr aktuálního řádku |  |
| Log: Zkopírovat jako Mermaid gitGraph... | `Y` |
| Log: Porovnat výběr | `v` |
| Log: Prohodit konce porovnání | `x` |
| Log: přejít na další porovnávaný řádek | `n` |
| Diff: Přepnout zalamování řádků | `w` |
| Diff: Vybrat kódování... | `C` |
| Zobrazení: Přepnout minimapu | `M` |
| Zobrazení: Přepnout sloupec commitů | `Alt+C` |
| Zobrazení: Sloupec commitů: skrýt |  |
| Zobrazení: Sloupec commitů: historie souboru |  |
| Zobrazení: Sloupec commitů: všechny commity |  |
| Zobrazení: Přejít na sloupec commitů |  |
| Zobrazení: Rozšířit sloupec commitů |  |
| Zobrazení: Zúžit sloupec commitů |  |
| Zobrazení: Sloupec commitů: posunout doleva | `Shift+Left` |
| Zobrazení: Sloupec commitů: posunout doprava | `Shift+Right` |
| Zobrazení: Sloupec commitů: vybrat další část | `Tab` |
| Zobrazení: Sloupec commitů: přesunout část doleva | `<` |
| Zobrazení: Sloupec commitů: přesunout část doprava | `>` |
| Zobrazení: Sloupec commitů: rozšířit část | `]` |
| Zobrazení: Sloupec commitů: zúžit část | `[` |
| Zobrazení: Sloupec commitů: obnovit šířky částí | `0` |
| Zobrazení: Sloupec commitů: nabídka vybraného řádku | `.` |
| Zobrazení: Sloupec commitů: vybrat sloupce |  |
| Zobrazení: Sloupec commitů: přepnout Ref |  |
| Zobrazení: Sloupec commitů: přepnout Graf |  |
| Zobrazení: Sloupec commitů: přepnout Autor |  |
| Zobrazení: Sloupec commitů: přepnout Datum |  |
| Zobrazení: Sloupec commitů: přepnout Změny |  |
| Zobrazení: Sloupec commitů: přepnout SHA |  |
| Zobrazení: Sloupec commitů: zobrazit vybraný řádek v Logu |  |
| Zobrazení: Sloupec commitů: zobrazit změny souboru ve vybraném řádku |  |
| Zobrazení: Sloupec commitů: zobrazit soubor tak, jak byl ve vybraném řádku |  |
| Zobrazení: Sloupec commitů: zobrazit Blame souboru ve vybraném řádku |  |
| Editor: Přepnout sledování výběru |  |
| Obrázek: Přiblížit | `+` |
| Obrázek: Oddálit | `-` |
| Obrázek: Přizpůsobit zobrazení | `0` |
| Obrázek: Skutečná velikost (1:1) | `1` |
| Obrázek: Přepnout před / po | `Tab` |
| Obrázek: Rozšířit panel Před | `Alt+Right` |
| Obrázek: Rozšířit panel Po | `Alt+Left` |
| Obrázek: Obnovit velikosti panelů | `Alt+0` |
| Obrázek: Upravit v... | `O` |
| Obrázek: Upravit ve výchozím editoru | `o` |
| Historie souboru: Zobrazit v Logu | `Enter` |
| Historie souboru: Kopírovat SHA commitu | `y` |
| Historie souboru: Kopírovat shrnutí | `Y` |
| Historie souboru: Zobrazit diff tohoto commitu | `d` |
| Historie souboru: Otevřít soubor u tohoto commitu | `o` |
| Historie souboru: Blame u tohoto commitu | `b` |
| Historie souboru: Porovnat s pracovním stromem | `c` |
| Historie souboru: Obnovit tuto verzi | `R` |
| Historie souboru: Rozšířit výběr nahoru | `Shift+Up` |
| Historie souboru: Rozšířit výběr dolů | `Shift+Down` |
| Historie souboru: Přepnout mezi seznamem a diffem | `Tab` |
| Historie souboru: Rozšířit seznam | `Alt+Right` |
| Historie souboru: Zúžit seznam | `Alt+Left` |
| Historie souboru: Obnovit šířku seznamu | `Alt+0` |
| Soubor: Otevřít pomocí... | `O` |
| Soubor: Otevřít ve výchozí aplikaci | `o` |
| Diff: Přepnout změny jen v bílých znacích | `s` |
| ✦ AI: Vygenerovat zprávu commitu | `Ctrl+G i` |
| ✦ AI: Vygenerovat zprávu stashe |  |
| ✦ AI: Vložit proměnnou promptu |  |
| ✦ AI: Upravit profil... |  |
| ✦ AI: Otevřít agenta... |  |
| ✦ AI: Vysvětlit vybraný commit | `Ctrl+G w` |
| ✦ AI: Rozdělit vybraný Commit... |  |
| ✦ AI: Vysvětlit rozpracované změny | `Ctrl+G w` |
| ✦ AI: Sestavit commity z rozpracovaných změn... | `Ctrl+G C` |
| Composer: Přesunout soubor do commitu... | `m` |
| Diff: Blame |  |
| Blame: Zobrazit commit v Logu | `Enter` |
| Blame: Kopírovat SHA commitu | `Y` |
| Blame: Zobrazit diff commitu pro soubor | `d` |
| Blame: Blame na rodiči commitu | `b` |
| Blame: Otevřít historii souboru | `H` |
| Blame: Zobrazit nebo skrýt kartu s detaily | `i` |
| Blame: Další hunk | `n` |
| Blame: Předchozí hunk | `p` |
| Blame: Další hunk stejného commitu | `N` |
| Blame: Předchozí hunk stejného commitu | `P` |
| Blame: Přejít na řádek | `g` |
| Blame: Rozšířit sloupec anotací | `>` |
| Blame: Zúžit sloupec anotací | `<` |
| Diff: Historie souboru |  |
| Diff: Pohled na soubor |  |
| Soubor: Upravit v pracovním adresáři | `e` |
| Editor: Změnit kódování |  |
| Editor: Znovu otevřít s kódováním |  |
| Editor: Převést na kódování |  |
| Editor: Převést konce řádků |  |
| Editor: Změnit odsazení |  |
| Aplikace: Nastavení... | `Ctrl+, / Ctrl+N ,` |
| Aplikace: Hledat v nastavení... | `/ v Nastavení` |
| Aplikace: Klávesové zkratky... |  |
| Agent: Zobrazit čekající pohled | `Ctrl+N Enter` |
| Agent: Zavřít čekající pohled |  |
| Agent: Zpět | `Ctrl+N Backspace` |
| Zkratky: Obnovit vše... |  |
| Integrace: Otevřít propojený pull request | `Ctrl+G g` |
| Integrace: Upravit propojený pull request u hostitele | `e (na řádku)` |
| Integrace: Kopírovat URL pro propojený pull request |  |
| Integrace: Obnovit pull requesty a issues |  |
| Integrace: Checkout pull request... |  |
| Integrace: Otevřít odkaz z commitu (#issue / !request)... |  |
| Integrace: Otevřít issue... |  |
| Hosting: Zkontrolovat žádost... |  |
| Hosting: Push ke kontrole... |  |
| Hosting: Upravit řešitele... |  |
| Hosting: Upravit recenzenty... |  |
| Hosting: Upravit štítky... |  |
| Hosting: Nastavit milník... |  |
| Lokální issues: Zavřít issue... |  |
| Lokální issues: Znovu otevřít issue... |  |
| Lokální issues: Otevřít v Obsidianu... |  |
| Issues: Otevřít tabuli |  |
| Lokální issues: Vytvořit Obsidian Base |  |
| Pull requesty: Vytvořit pull request... | `n` |
| Hosting: Nové issue... | `n` |
| Pull requesty: Hledat | `Enter na řádku hledání` |
| Pull requesty: Zvolit filtr (otevřené / merged / všechny)... | `v` |
| Pull requesty: Zvolit poskytovatele... | `Enter na řádku poskytovatele` |
| Pull requesty: Zvolit repozitář... | `Enter na řádku repozitáře` |
| Pull requesty: Odpojit zdroj (žádný) |  |
| Issues: Zvolit poskytovatele... | `Enter na řádku poskytovatele` |
| Issues: Odpojit zdroj (žádný) |  |
| Integrace: Odpojit profil (automaticky) |  |
| Issues: Zvolit repozitář... | `Enter na řádku repozitáře` |
| Issues: Hledat | `Enter na řádku hledání` |
| Issues: Zvolit filtr... | `v` |
| Issues: Upravit dotaz / seznamy... |  |
| Commit: Vložit odkaz na issue... |  |
| Integrace: Použít profil pro tento repozitář... |  |
| Integrace: Spravovat profil tohoto repozitáře |  |
| Integrace: Připojit integraci... |  |
| Nastavení: Možnosti Radaru profilu... |  |
| Integrace: Zkontrolovat zabezpečení |  |
| Aplikace: Ukončit | `q` |
| Aplikace: Motiv... |  |
| Aplikace: Změnit motiv... | `Ctrl+N y` |
| Aplikace: Změnit jazyk... | `Ctrl+N g` |
| Spouštěč: Vytvořit zástupce |  |
| Spouštěč: Vytvořit zástupce na ploše |  |
| Spouštěč: Odebrat zástupce |  |
| Spouštěč: Vytvořit alias příkazu |  |
| Spouštěč: Odebrat alias příkazu |  |
| Spouštěč: Přidat do PATH |  |
| Spouštěč: Odebrat z PATH |  |
| Aplikace: Změnit barevný motiv grafu... | `Ctrl+N Y` |
| Aplikace: Přepnout zbarvení řádků grafu | `Ctrl+N T` |
<!-- palette:end -->

## Pohyb po aplikaci

| Klávesa | Akce |
|---|---|
| `Tab` / `Shift+Tab` | Zaostřit další / předchozí panel |
| `Ctrl+N`, poté odznak | Zaostřit tuto část okna (viz [Předpona navigace](#předpona-navigace-ctrln)) |
| `j` / `k`, `Up` / `Down` | Posunout výběr, v každém panelu |
| `PageUp` / `PageDown`, `Home` / `End` | Posun o stránku; skok na první / poslední řádek |
| číslo, poté `j` / `k` / `Up` / `Down` | Předpona počtu: `4j` posune o čtyři řádky. S `PageUp` / `PageDown` je počet počtem stránek. Stavový řádek ukazuje čekající počet (`4…`). `Esc` nebo jakákoli jiná klávesa jej zruší. Počáteční `0` počet nikdy nezačíná |
| `h` / `l`, `Left` / `Right` | Panel Refs: přepnout sekci. Log: opustit (`h`) nebo vstoupit (`l`) do seznamu změněných souborů commitu |
| `?` | Otevřít nebo zavřít nápovědu (viz [Nápověda v aplikaci](#nápověda-v-aplikaci-)) |
| `r` | Obnovit |
| `F5` | Fetch všech remote, znovu načíst pull requesty a issues a obnovit |
| `q` | Ukončit |
| `Ctrl+Z` | **Zpět**: vrátit poslední operaci. Viz [Zpět a znovu](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Znovu**. `Ctrl+Shift+Z` potřebuje klávesnicový protokol kitty; `Ctrl+Y` funguje vždy |
| `.`, `Menu`, `Shift+F10` | Otevřít kontextovou nabídku vybraného řádku |

`Ctrl+Z` a `Ctrl+Y` nedělají nic, dokud má klávesnici textové pole. Editor má
vlastní vrácení textu.

## Filtrovatelné vyskakovací seznamy

Všechny filtrovatelné vyskakovací seznamy používají stejné klávesy: paleta
příkazů a její výběry (motiv, jazyk, barvy grafu), výběr karty a každý
rozbalovací seznam s vyhledávacím řádkem (volby v Nastavení, formulář pull
requestu, výběr odkazu na issue, seznam **Otevřít v**). Psaním seznam filtrujete.
Tyto klávesy posouvají zvýraznění:

| Klávesa | Akce |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Posun o jeden řádek |
| `Home` / `End` | První / poslední řádek |
| `PageUp` / `PageDown` | Posun o jednu viditelnou stránku |
| `Enter` | Zvolit zvýrazněný řádek |
| `Esc` | Zavřít |

V těchto oknech `Home`, `End` a `Ctrl+K` posouvají seznam, ne kurzor pole
filtru. `Ctrl+A` a `Ctrl+E` stále přesunou kurzor na začátek a na konec. Prosté
`j` a `k` se píší do filtru.

Myší:

- Kolečko seznam **posouvá**. Zvýraznění nepřesouvá.
- Klik řádek **vybere**. Dvojklik jej **zvolí**, jako `Enter`.
- Seznam vyšší než jeho okno ukazuje na pravém okraji posuvník. Můžete jej
  táhnout.
- Krátký rozbalovací seznam bez vyhledávacího řádku (dvě nebo tři možnosti)
  volí už prvním kliknutím.

**Živý náhled.** Výběry motivu, barevného motivu grafu a jazyka použijí
zvýrazněný řádek okamžitě, za oknem. `Enter` jej ponechá. `Esc` nebo klik mimo
okno obnoví předchozí hodnotu.

**Filtr motivů.** Výběr motivu má pod polem filtru řádek: **Vše**, **Tmavé**,
**Světlé** a **Barevné**. `Tab` / `Shift+Tab` přejdou na další / předchozí.
`Alt+1` až `Alt+4` zvolí jeden přímo. Můžete na ně také klikat.

## Nápověda v aplikaci (`?`)

`?`, `Ctrl+N ?` nebo `Zobrazení: Přepnout nápovědu` z palety otevře nápovědu.
Uvádí klávesy a gesta myši každé části aplikace. Otevře se nad čímkoli, včetně
obrazovky Nastavení.

| Klávesa | Akce |
|---|---|
| `?` | Zavřít nápovědu (a vymazat filtr) |
| *(psaní)* | Filtr: ponechá jen sekce, které obsahují text. `j`, `k` a `q` se také píší |
| `Backspace`, `Ctrl+U` | Smazat poslední znak / vymazat filtr |
| `Esc` | Vymazat filtr; je-li prázdný, zavřít nápovědu |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Posun o jeden řádek |
| `PageUp` / `PageDown`, `Home` / `End` | Posun o stránku; skok na začátek / konec |
| `Ctrl+D`, nebo klik na odkaz | Otevřít tuto dokumentaci na GitHubu v prohlížeči |

Kolečko nápovědu posouvá a její posuvník můžete táhnout. Klik mimo nápovědu ji
zavře.

## Předpona navigace (`Ctrl+N`)

`Ctrl+N` čeká na jednu další klávesu. Stavový řádek ukazuje klávesy, které přijímá.
Další klávesa provede akci, nebo zruší.

| Klávesa | Akce |
|---|---|
| `Ctrl+N`, poté odznak | Zaostřit část okna, která ukazuje tento odznak |
| `Ctrl+N h` / `Ctrl+N l` | Sbalit nebo obnovit panel Refs / pravý panel |
| `Ctrl+N +` / `Ctrl+N -` | Zvětšit / zmenšit aktivní sekci Refs o jeden řádek |
| `Ctrl+N ]` / `Ctrl+N [` | Zvětšit / zmenšit blok zprávy commitu v pravém panelu o jeden řádek |
| `Ctrl+N Left` / `Ctrl+N Right` | Posunout dělič mezi panelem Refs a Logem o krok doleva / doprava (sbalený panel Refs se nejdřív vrátí) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Posunout dělič mezi Logem a pravým panelem o krok doleva / doprava |
| `Ctrl+N t` | Zvolit kartu (výběr karty; samotné `t` dělá totéž) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Změnit motiv / jazyk / barevný motiv grafu |
| `Ctrl+N Shift+T` | Zapnout nebo vypnout zbarvení řádků grafu |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Ukázat zobrazení, které agent připravil / vrátit se (viz [MCP a agenti](mcp-and-agents.md)) |
| `Ctrl+N ,` | Otevřít Nastavení |
| `Ctrl+N ?` | Otevřít nebo zavřít nápovědu |
| `Esc`, nebo jakákoli jiná klávesa | Zrušit |

### Odznaky

Dokud předpona čeká, každá část okna, která může převzít fokus, ukazuje
jednoznakový odznak na svém levém horním okraji. Stiskněte tento znak a část se
zaostří. Odznaky se rozdávají pokaždé znovu, v pořadí na obrazovce: nejdřív `1`
až `9`, pak `a`, `b`, `c`... Čtěte tedy odznak na obrazovce: část nemá vždy
stejný odznak.

Části s odznakem: filtr Refs a každá viditelná sekce Refs, Log (nebo zobrazení
Diff), zobrazení Soubor, když je otevřeno, seznamy Unstaged a Staged, pole
souhrnu commit composeru a ovládací prvky seznamu souborů commitu. Na kartě
Domů jsou odznaky vyhledávací pole, čtyři sekce a tlačítka panelu nástrojů (odznak tlačítka ho stiskne). Na kartě Radar jsou to pole filtru,
rozbalovací seznamy Rozsah, Zobrazení a Zobrazit (odznak seznamu ho otevře),
tlačítka Obnovit a Zpět a každý nadpis sekce na obrazovce. Odznak sekce přesune
výběr na její nadpis; `Enter` nebo `Space` ji pak sbalí.

V dialozích Procházet (výběr složky), Klonovat, Init, Vytvořit worktree, Pracovní prostor, Částečný checkout, Flow, Nové issue a pull request jsou odznaky pole, zaškrtávací políčka a tlačítka (odznak tlačítka ho stiskne, odznak pole do něj dá kurzor; v seznamu je jeden odznak na jeho prvním řádku; výběr složky Procházet má jeden odznak pro seznam Rychlý přístup a jeden pro seznam Složky, vedle prvního řádku každého seznamu, a jeho stisk dá klávesnici do toho seznamu, aniž by vybral řádek). V Protokolu aktivity jsou to obě karty, pole Rozsah a Úroveň, filtr, zaškrtávací políčka úrovní, tlačítka a první položka seznamu. Odznak udělá totéž co kliknutí na daný prvek. Ve výběru složky `Ctrl+N` spouští odznaky, takže klávesa Nová složka je `Alt+N`.

V dialogu Upravit (a Upravit zprávu commitu) jsou odznaky souhrn, popis, jméno, e-mail a datum autora, **Upravit historii hlouběji...**, **Zrušit**, **Použít** a křížek zavření; v nástroji pro přepis historie je to pole rozsahu, jeho zaškrtávací políčka a tlačítko refů, název receptu, každé pravidlo, jeho políčko a tlačítka, tlačítko **+ Přidat pravidlo**, pole vybraného pravidla, karty náhledu, filtr, řádky chráněných větví a jejich Zaškrtnout vše / Odškrtnout vše a tlačítka Náhled, Použít, Zrušit, Uložit, Načíst a Vytvořit pravidla pomocí AI (stránky Tajemství, Identity, Velikost a výsledek přidávají vlastní tlačítka a řádky; řádky karty Commity, jako seznam, žádné nemají). Odznak udělá totéž co kliknutí. Nic se neoznačuje, dokud je otevřený rozbalovací seznam, otázka, pole, do kterého se píše, výběr složky, obrazovka čištění, editor skriptu, dialogy AI, diff blobu nebo návod hostitele, ani když dialog Upravit klade otázku „zahodit?“; `Esc` odznaky odstraní, aniž by cokoli zavřel.

V Nastavení jsou odznaky u vyhledávacího pole, seznamu stránek (jeden odznak, na aktuální stránce), podkaret, seznamu a jeho tlačítek, každého pole stránky na obrazovce a křížku zavření. Odznak udělá totéž co kliknutí na daný prvek: textové pole se otevře k úpravě, zaškrtávací políčko se přepne, rozbalovací seznam se otevře, tlačítko se provede. `Ctrl+N` si ponechává vlastní význam, když do pole píšete nebo je otevřený rozbalovací seznam, editor či hledání.

Paleta má stejné akce, například `Navigace: Přejít na Refs > Lokální` nebo
`Navigace: Přejít na Stav > Staged`.

## Repozitáře a karty

| Klávesa | Akce |
|---|---|
| `Ctrl+O`, nebo klik na `⌂` | Přejít na kartu Domů (na kartě Domů: zaostřit vyhledávací pole; s otevřeným Radarem nebo poznámkami k vydání: nejprve je zavřít) |
| `Ctrl+Shift+N` | Otevřít kartu Domů s dialogem Klonovat |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Další / předchozí karta |
| `t` | Zvolit kartu z filtrovatelného seznamu |
| `Ctrl+W` | Zavřít aktuální kartu (nejdřív se zeptá; repozitář se nezmění) |
| `Ctrl+T`, nebo pravé tlačítko na kartě | Nabídka karty (její řádky jsou vypsány v části [Řádek karet](interface-tour.md#řádek-karet)) |
| `Alt+O` / `Ctrl+Alt+O` | Otevřít složku karty: zvolit ze seznamu **Otevřít v** / použít výchozí položku. Viz [Otevřít v](open-in.md) |

### Karta Domů

Karta `⌂` spravuje vaše repozitáře. Má čtyři sekce: **Otevřené repozitáře**,
**Oblíbené**, **Nedávné repozitáře** a **Všechny repozitáře**. Viz
[Prohlídka rozhraní](interface-tour.md#domů).

| Klávesa | Akce |
|---|---|
| `j` / `k`, šipky, `PageUp` / `PageDown`, `Home` / `End` | Pohyb |
| `Enter`, nebo dvojklik | Otevřít repozitář (nebo přepnout na jeho kartu). Na záhlaví sekce: sbalit nebo rozbalit |
| `f` nebo `*`, nebo klik na hvězdičku | Přidat do oblíbených / odebrat z nich |
| `x`, `Delete`, nebo tlačítko řádku | Zavřít jeho kartu (Otevřené), odebrat jej ze seznamu (Nedávné), odebrat hvězdičku (Oblíbené), zapomenout jej (Všechny). Nic na disku se nemaže |
| `.`, pravé tlačítko, nebo `⋮` řádku | Nabídka řádku |
| `/` nebo `Ctrl+F` | Hledat ve všech sekcích podle názvu, větve nebo cesty. `Enter` otevře první shodu; `Esc` vymaže hledání a poté opustí pole |
| `Space`, `Left` / `Right` | Sbalit / rozbalit aktuální sekci |
| `-` / `+` | Sbalit vše / Rozbalit vše |
| `b` / `c` / `i` | Procházet (otevřít složku) / Klonovat / Init |
| `Esc` | Vymazat hledání, nebo se vrátit na kartu aktivního repozitáře |
| `q` | Ukončit |

`Ctrl+P`, `Ctrl+,`, `?` a `Ctrl+N` na kartě Domů fungují. `Ctrl+G`, `Ctrl+W`
a `Ctrl+T` potřebují kartu repozitáře: na kartě Domů jen ukážou zprávu. Příkazy
palety, které potřebují repozitář, jsou zašedlé.

### Výběr složky

**Procházet** (a tlačítko **Procházet...** v dialozích Init, Klonovat a
Vytvořit worktree) otevře výběr složky. Nahoře má pole cesty, vlevo místa a
strom složek a vpravo obsah aktuální složky.

| Klávesa | Akce |
|---|---|
| `Tab` / `Shift+Tab` | Pole cesty, poté levý sloupec, poté obsah |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Pohyb v zaostřeném sloupci. V poli cesty: pohyb mezi doplněními |
| písmeno (obsah) | Přejít na další složku, která začíná tímto písmenem |
| `Enter` / `Right` | Otevřít vybranou složku. V poli cesty: přejít do zapsané složky; je-li už zobrazena, zvolit ji |
| `Backspace` / `Left` / `Alt+Up` | Nadřazená složka (v poli cesty `Backspace` maže znak) |
| `Tab` (pole cesty) | Přijmout jediné doplnění, nebo to zvýrazněné |
| `Alt+N` | Nová složka (`Ctrl+N` spouští odznaky, viz [Odznaky](#odznaky)) |
| `Alt+H` | Zobrazit nebo skrýt skryté složky |
| `Ctrl+F`, nebo `/` mimo pole cesty | Filtrovat položky složky během psaní (bez ohledu na velikost písmen; `..` zůstává; záhlaví počítá „x z n“; `Enter` skočí na první shodu, `Esc` filtr vymaže; při změně složky se vyprázdní). Dostanete se k němu i přes `Tab` nebo kliknutím |
| `Ctrl+S`, `Ctrl+Enter`, nebo potvrzovací tlačítko | Zvolit vybranou podsložku, jinak zobrazenou složku |
| `Esc` | Zavřít doplnění, nebo zrušit |

Když pole cesty ukáže novou cestu, vybere se celá: první klávesa, kterou
napíšete, ji nahradí.

### Dialogy Init a Klonovat

| Klávesa | Akce |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Další / předchozí pole nebo tlačítko |
| `Enter` | Stisknout zaostřené tlačítko, nebo vytvořit / klonovat |
| `Space` | Přepnout zaostřené zaškrtávací pole, nebo stisknout zaostřené tlačítko |
| `Ctrl+B` | Procházet složku |
| `j` / `k`, `g` / `G` | V seznamu zdrojů a seznamu repozitářů dialogu Klonovat: dolů / nahoru, první / poslední (ne v poli hledání) |
| `Esc` | Zavřít. Během klonování klonování pokračuje na pozadí |

V dialogu Klonovat uvádí levý sloupec **Klonovat z URL** a jeden zdroj pro
každý připojený hostingový profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit).
Zdroj zvolíte pomocí `Up` / `Down` nebo kliknutím. Poté pište do vyhledávacího
pole **Repozitář**, pohybujte se v seznamu šipkami, `PageUp` / `PageDown` nebo
kolečkem a řádek zvolte pomocí `Enter` nebo kliknutím. Pole viz
[První kroky](getting-started.md).

## Panel Refs

| Klávesa | Akce |
|---|---|
| `Enter` / `Space` | Rozbalit nebo sbalit aktivní sekci, nebo vybranou složku |
| `h` / `l`, `Left` / `Right` | Přepnout sekci |
| `/` | Filtrovat podle názvu (`Enter` použije, `Esc` zruší) |
| `y` | Zapnout nebo vypnout hierarchické zobrazení větví: složky podle `/` v názvech větví (LOKÁLNÍ a REMOTE) |
| `o` | **Solo** pro vybraný ref, nebo pro každý ref pod vybranou složkou či remote. Solo pro několik refs ukáže jejich sjednocení |
| `Shift+O` | **Ukončit solo**: zrušit každé solo |
| `Shift+H` | **Zobrazit všechny skryté refs**: znovu zobrazit každý skrytý ref |
| `g` | **Ukázat v grafu**: posune Log na commit vybraného refu (nebo stashe) a jeho řádek krátce zvýrazní; výběr se nepohne |
| `m` | **Označit** vybraný ref jako zdroj přetažení (`Esc`, nebo opakované `m` na něm, značku zruší) |
| `Shift+M` | **Pustit označený ref** na vybraný ref: otevře nabídku, kterou otevírá tažení (Merge, Rebase, Push, Fast-forward...) |
| `+` (sekce REMOTE) | Přidat remote |
| `.`, `Menu`, `Shift+F10`, nebo pravé tlačítko | Nabídka řádku: větev, tag, název remote nebo složka |
| `Enter` na řádku WORKTREES | Otevřít tento worktree na vlastní kartě |
| `Ctrl+G d` | Smazat vybranou větev nebo tag (nejdřív se zeptá). Na řádku REMOTE: smazat tuto větev remote |

K tomu, co nabídky dělají, viz [Práce s Logem](log-and-history.md).

### Pull requesty a issues

S odpovídající integrací má panel Refs sekci pull requestů (GitLab: merge
requesty). S připojeným hostingovým profilem má také sekci ISSUES. Viz
[Hostingové služby a issue trackery](hosting-integration.md#5-pull-requesty-a-issues-v-postranním-panelu).

| Klávesa | Akce |
|---|---|
| `Enter` | Otevřít vybraný řádek v prohlížeči. Na řádku `Reconnect`: otevřít Nastavení na tomto profilu |
| `Space` | Rozbalit nebo sbalit sekci |
| `v` | Otevřít filtr sekce. Pull requesty: Otevřené, Merged / zavřené, Všechny. Issues: Otevřené, Přiřazené mně, Zavřené, Všechny |
| `n` | Sekce pull requestů: vytvořit pull request. Sekce ISSUES: nové issue. (Tlačítko `+` v záhlaví) |
| `e` | Sekce pull requestů: upravit vybraný požadavek na hostiteli (tužka na řádku) |
| `Enter` na řádku hledání | Psát do vyhledávacího pole. `Enter`, `Tab` nebo `Esc` jej opustí; `Ctrl+U` je vymaže |
| `Enter` na záhlaví skupiny | Sbalit nebo rozbalit skupinu (*Moje pull requesty*, *Přiřazené mně*, *Čekají na mou kontrolu*, *Všechny pull requesty*) |
| `Enter` na řádku filtru, poskytovatele nebo repozitáře | Otevřít tento rozbalovací seznam. Seznam poskytovatelů končí volbou **Žádný (odpojit tento repozitář)** |
| `.`, nebo pravé tlačítko | Nabídka řádku: otevřít v prohlížeči, kopírovat URL nebo číslo, checkout zdrojové větve (pull requesty); vytvořit větev z issue (issues) |

### Tabule issues

**Tabule** v záhlaví ISSUES (paleta: **Issues: Otevřít tabuli**) otevře tabuli Kanban zdroje issues. Viz
[tabule](hosting-integration.md#5b-tabule-issues) a [lokální issues](local-issues.md#tabule-kanban). Každá klávesa
níže je také příkazem z palety (**Tabule: ...**).

| Klávesa | Akce |
|---|---|
| Šipky | Vybrat kartu (`Up`, `Down`) nebo sousední sloupec (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` skáčou ve sloupci |
| `Shift+Left`, `Shift+Right` | Přesunout vybranou kartu do předchozího nebo dalšího sloupce |
| `m` | **Přesunout kartu do...**: rozbalovací seznam sloupců |
| `Enter` | Otevřít issue (u lokálního issue poznámku; jinak stránku hostitele) |
| `e`, `.` | Upravit řešitele, štítky nebo milník; nabídka karty |
| `n` | Nové issue ve vybraném sloupci |
| `a`, `l`, `M`, `/` | Filtrovat podle řešitele, štítku, milníku, textu. `x` filtry zruší |
| `c` | Štítky, které jsou sloupci tabule GitHub / GitLab |
| `r` | Znovu načíst issues |
| `Esc` | Zavřít tabuli |

Myš: přetažením karty na jiný sloupec ji přesunete, kliknutím kartu vyberete, dvojklikem otevřete, pravým tlačítkem
otevřete nabídku karty, kolečkem posouváte sloupec, `Shift`+kolečko (nebo lišta pod tabulí) posouvá sloupce.

### Flow (workflow větvení)

Když je pro repozitář nastaven workflow (viz [workflow větvení](branching-workflows.md)), má panel nástrojů tlačítko
**Flow** a postranní panel seskupuje LOKÁLNÍ a REMOTE podle typu větve. Každý řádek nabídky je příkazem z palety
(**Flow: ...**; **Flow: Zahájit Feature...** a **Flow: Nové issue: Feature...** se generují pro každý typ tématické větve
workflow). Řádky řízené hostitelem (*Nové issue: <typ>*, *Převzít větev vytvořenou hostitelem*) a jejich dialogy, karta čekání a karta
„objevila se větev“ jsou popsány ve [workflow větvení](branching-workflows.md#toky-řízené-hostitelem-v-aplikaci);
`Enter` spustí zvýrazněné tlačítko (*Vytvořit issue*, *Převzít*), `Esc` přestane čekat nebo zavře.

| Klávesa | Akce |
|---|---|
| `Ctrl+G Shift+W` | Otevřít nabídku Flow (tlačítko na panelu nástrojů) |
| `Enter`, `Tab`, `Shift+Tab` | V dialogu Zahájit / Dokončit / Synchronizovat / Povýšit / Nastavit rodiče: náhled nebo provedení, další pole, předchozí pole |
| `Space` | Zaškrtnout políčko (*Publikovat*, *Spustit hooky*, *Aktualizovat i větve na ní navázané*) |
| `Esc` | Zavřít rozbalovací seznam, vrátit se z náhledu do formuláře, zavřít dialog (ne během běhu) |
| Najetí na větev | Její typ, rodič a o kolik je pozadu za rodičem |

## Panel Log

| Klávesa | Akce |
|---|---|
| `Enter` / `l` | Vstoupit do seznamu změněných souborů vybraného commitu |
| `Enter` / `l` na řádku souboru | Otevřít diff souboru uprostřed |
| `Esc` / `h` | Vrátit se na commity |
| `Right` | (commit s více než jedním refem) Otevřít seznam jeho dalších refs: `Up` / `Down` se pohybují, `Enter` provede checkout, `Esc` zavře |
| `f` | Otevřít lištu hledání (viz [Hledání](#hledání)) |
| `c` | (s necommitnutými změnami) Vybrat řádek WIP a dát kurzor do jeho pole zprávy |
| `Enter` na řádku WIP | Dát kurzor do pole zprávy řádku WIP |
| `.`, `Menu`, `Shift+F10` | Nabídka commitu. `Up` / `Down` se pohybují, `Right` / `l` / `Enter` otevřou podnabídku, `Left` / `h` / `Esc` se vrátí |
| `m` / `a` | Soubory pravého panelu: `Cesta` nebo `Strom` / `Zobrazit všechny soubory` |
| `Shift+C` | Vybrat sloupec Logu k přesunu nebo změně šířky (viz níže) |

S vybraným sloupcem `Tab` / `Shift+Tab` vyberou jiný, `<` / `>` ho přesunou, `]` / `[` ho rozšíří nebo zúží, `0` vrátí všechny šířky na automatické a `Esc` nebo `Enter` výběr ukončí (kontext `log_columns` ve vygenerované tabulce výše).

Pole zprávy řádku WIP je pole souhrnu commit composeru. Dokud má kurzor, píše se
každá klávesa, ale kombinace `Ctrl` stále fungují. `Enter` přejde na popis
composeru. `Esc` pole opustí a text ponechá. Viz
[Stagování a commitování](staging-and-committing.md).

### Výběr několika commitů

| Klávesa nebo gesto | Akce |
|---|---|
| `Space` | Přidat aktuální commit do výběru, nebo jej odebrat |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Rozšířit výběr (Alt pro terminály, které si Shift ponechávají pro sebe) |
| `Ctrl+klik` / `Shift+klik` nebo `Alt+klik` | Přepnout commit / vybrat rozsah |
| `Esc` | Vymazat výběr |
| `Shift+Y` | [Zkopírovat výběr jako Mermaid gitGraph](log-and-history.md#kopírování-commitů-jako-mermaid-gitgraph) (okno s úrovněmi detailu a živým náhledem; `Enter` zkopíruje) |

Při výběru několika commitů klikněte na jeden z nich pravým tlačítkem a zvolte
**Cherry-pick N commitů na `<větev>`...**. Otevře se stejné zobrazení jako u
interaktivního rebase.

### Pojmenování větve nebo tagu

`Ctrl+G b`, `Ctrl+G t`, tlačítka **Větev** a **Tag** na panelu nástrojů a
položky nabídek **Vytvořit větev zde...** / **Vytvořit tag zde...** se zeptají na
název v textovém poli na řádku commitu. `Enter` vytvoří ref. `Esc` zruší.
Neplatný nebo existující název se ukáže červeně a pole zůstane otevřené.

### Úprava zprávy commitu

**Upravit zprávu commitu...** v nabídce commitu, `Ctrl+G e` nebo
`Git: Upravit zprávu commitu...` z palety otevře okno **Úprava zprávy commitu**.
Co mění, viz [Práce s Logem](log-and-history.md).

| Klávesa | Akce |
|---|---|
| `Enter` v souhrnu | Přejít na popis |
| `Enter` v popisu | Nový řádek. `Up` na prvním řádku se vrátí na souhrn |
| `Ctrl+Enter`, `Ctrl+S`, nebo **Aktualizovat zprávu** | Uložit novou zprávu |
| `Tab` / `Shift+Tab` | Přechod mezi souhrnem, popisem a tlačítky |
| `Esc`, nebo **Zrušit** | Zavřít bez změny |

### Úprava commitu

`Ctrl+G Shift+E` (všechna pole) a `Ctrl+G Shift+A` (autor) otevřou dialog Úprava commitu, stejně jako nabídka commitu, řádky `Commit: Upravit...` v paletě a panel podrobností commitu; všechny cesty do něj popisuje [Úprava commitu](log-and-history.md#úprava-commitu-zpráva-autor-data). Obě zkratky lze přiřadit jinak: `git_prefix.amend` a `git_prefix.edit_author`. Klávesy uvnitř dialogu:

| Klávesa | Akce |
|---|---|
| `Tab` / `Shift+Tab` | Přechod mezi poli a tlačítky |
| `Enter` v jednořádkovém poli | Další pole; v posledním poli **Použít** |
| `Up` / `Down` | Předchozí / další pole (v popisu od jeho prvního / posledního řádku) |
| `Ctrl+Enter`, `Ctrl+S`, nebo **Použít** | Použít změněná pole |
| `Esc`, nebo **Zrušit** | Zavřít bez změny |
| `Alt+H`, nebo **Upravit historii hlouběji...** | Zavře dialog a otevře nástroj pro přepis historie od tohoto commitu (jen sestavení s přepisem historie); pokud jste upravili pole, nejdřív se zeptá (`Esc` potvrdí). Lze přiřadit jinak: `reword.deeper` |

### Vytvoření worktree

**Vytvořit worktree z `<ref>`...** (nabídky větve a tagu) a **Vytvořit worktree
zde...** (nabídka commitu) otevřou dialog: složka, volitelná nová větev a
**Otevřít nový worktree na kartě**. Klávesy: `Tab` / `Shift+Tab` se pohybují,
`Enter` vytvoří, `Space` přepne zaškrtávací pole, `Ctrl+B` prochází, `Esc`
zruší.

### Interaktivní rebase

**Interaktivní rebase odsud...** v nabídce commitu, nebo
`Git: Interaktivní rebase od vybraného commitu...`, otevře zobrazení rebase.
Stejné zobrazení používá cherry-pick několika commitů. Viz
[Práce s Logem](log-and-history.md).

| Klávesa | Akce |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Nastavit akci řádku: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup a Edit jen v rebase) |
| `Space` | Otevřít seznam akcí řádku (`Up` / `Down` volí, `Enter` použije, `Esc` seznam zavře) |
| `Up` / `Down` | Posunout výběr |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Posunout vybraný řádek nahoru / dolů |
| `a` | Zapnout nebo vypnout náhled autosquash |
| `Enter` na řádku Reword | Upravit novou zprávu tohoto commitu |
| `Ctrl+S`, `Ctrl+Enter`, nebo **Spustit rebase** / **Spustit cherry-pick** | Provést plán |
| `Ctrl+R`, nebo **Obnovit** | Obnovit plán do stavu při otevření zobrazení |
| `Esc`, nebo **Zrušit** | Zavřít bez změny |

Když se rebase zastaví (na konfliktu nebo kroku Edit), banner nad Logem nabídne
**Pokračovat**, **Přeskočit** a **Přerušit**. Paleta má `Rebase: Pokračovat`,
`Rebase: Přeskočit` a `Rebase: Přerušit`.

## Hledání

`f`, tlačítko **Hledat** na panelu nástrojů nebo `Navigace: Hledat commity...`
otevře lištu hledání nad Logem. Hledá v celé zprávě commitu, jménu a e-mailu
autora, v SHA a v názvech refs. Řádky neskrývá: řádky, které neodpovídají, jsou
ztlumené.

| Klávesa | Akce |
|---|---|
| *(psaní)* | Upravit hledaný text (píše se každá klávesa) |
| `Enter` / `Down`, nebo `↑` na liště | Další shoda |
| `Shift+Enter` / `Up`, nebo `↓` na liště | Předchozí shoda |
| `Esc`, nebo `✕` na liště | Zavřít lištu hledání |

## Zobrazení diffu

Změněný soubor se otevře uprostřed, místo Logu. Viz
[Stagování a commitování](staging-and-committing.md).

| Klávesa | Akce |
|---|---|
| `Enter` / `l`, nebo dvojklik (na řádku souboru) | Otevřít diff souboru |
| `Esc`, `q`, nebo `✕` v titulku | Zavřít diff |
| `j` / `k`, `Up` / `Down` | Posouvat |
| `PageUp` / `PageDown`, `Home` / `End` | Posun o stránku; skok na začátek / konec |
| `h` / `l`, `Left` / `Right` | Posun do stran (ne se zalamováním slov) |
| `n` / `p` | Další / předchozí hunk |
| `m` | Řádkové nebo rozdělené (split) uspořádání |
| `Shift+H` | Jen hunky, nebo celý soubor |
| `w` | Zapnout nebo vypnout zalamování slov |
| `s` | Zobrazit nebo skrýt změny jen bílých znaků |
| `Shift+M` | Zapnout nebo vypnout minimapu |
| `Shift+C` | Vybrat kódování, ve kterém se soubor čte (seznam: nejdřív Automatická detekce) |
| `e` | Upravit soubor ve vestavěném editoru (viz [Editor](#editor)) |
| `r` | (soubor Git LFS) Karta LFS nebo surový ukazatel. Viz [Git LFS](lfs.md) |
| `Shift+E` | (soubor v konfliktu) Otevřít editor merge. Viz [Řešení merge konfliktů](merge-conflicts.md) |
| `y` / `Ctrl+C` | Kopírovat vybraný text. Bez výběru: řádek pod kurzorem, nebo vybrané řádky |
| `.`, nebo pravé tlačítko na hunku | Nabídka hunku |

### Stage a zahození z diffu

Tyto klávesy fungují jen v diffu rozpracovaných změn (otevřeném ze seznamu
Unstaged nebo Staged). Zahození funguje jen v seznamu Unstaged.

| Klávesa | Akce |
|---|---|
| `Space` | Stage (Unstaged) / unstage (Staged) hunku pod kurzorem |
| `v` | Zahájit nebo zrušit výběr řádků; `j` / `k` jej rozšiřují |
| `Enter` | Stage / unstage vybraných řádků, nebo řádku pod kurzorem |
| `x` | Zahodit hunk pod kurzorem (nejdřív se zeptá: `y` / `Enter` potvrdí, `n` / `Esc` zruší) |
| `Shift+X` | Zahodit vybrané řádky, nebo řádek pod kurzorem (nejdřív se zeptá) |

Počítají se jen změněné řádky: kontextové řádky ve výběru se ignorují. Výběr,
který zasahuje do druhého hunku, působí jen na první hunk.

## Zobrazení obrázku

Obrázek v zobrazení diff nebo v zobrazení Soubor. Klávesy jsou v kontextu
`image` tabulky výše. Viz [Obrázky](image-view.md).

## Zobrazení Soubor, Blame a Historie souboru

| Klávesa | Akce |
|---|---|
| `Esc` / `q` | Zavřít (s výběrem textu jej `Esc` nejdřív zruší) |
| `j` / `k`, `Up` / `Down` | Posunout výběr |
| `PageUp` / `PageDown` | Posun o stránku |
| `Home` / `End`, `Shift+G` | První / poslední řádek |
| `h` / `l`, `Left` / `Right` | Posun textu do stran |
| `Enter` | (Blame, Historie souboru) Přejít na commit řádku v Logu |
| `e` | Upravit soubor ve vestavěném editoru |
| `y` / `Ctrl+C` | Kopírovat vybraný text. Bez výběru: vybraný řádek (Blame: jen kód; Historie souboru: souhrn) |

Blame a historie souboru mají vlastní klávesy, uvedené ve vygenerované tabulce výše (kontexty `file_insight` a `history`) a jako `Blame: ...` / `Historie souboru: ...` v paletě; všechny lze předefinovat. Podrobnosti v [Blame](log-and-history.md#blame) a [Historie souboru](log-and-history.md#historie-souboru). V seznamu historie působí samotná písmena (`d` `o` `b` `c` `y`) na vybraný řádek, ale akord s Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) si vždy ponechá svůj globální význam.

## Sloupec commitů

Sloupec vlevo od zobrazení souboru (viz [Sloupec commitů](interface-tour.md#sloupec-commitů)). `Alt+C` ho odkudkoli zobrazí nebo skryje; ostatní klávesy fungují, jakmile má klávesnici (`Ctrl+N` a jeho štítek, nebo `Zobrazení: Přejít na sloupec commitů`).

| Klávesa | Akce |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Vybrat commit |
| `Enter` / `Right` | Pokračovat na seznam souborů vybraného commitu |
| `Esc` | Vrátit klávesnici pohledu |
| `.`, `Menu`, `Shift+F10` | Nabídka vybraného řádku |

Jeho další klávesy (`Shift+Left` / `Shift+Right` pro posun do stran, `Tab`, `]`, `[` a `0` pro šířky částí) jsou ve vygenerované tabulce výše, kontext `commit_column`.

## Editor

**Upravit**, nebo `e` v diffu či v zobrazení Soubor, otevře vestavěný editor na
kopii souboru v pracovním adresáři. Viz
[Stagování a commitování](staging-and-committing.md).

| Klávesa | Akce |
|---|---|
| *(psaní)* | Vložit na kurzor (nahradí výběr) |
| `Enter` | Nový řádek, s odsazením aktuálního řádku |
| `Tab` / `Shift+Tab` | Odsadit / zrušit odsazení (vybraných řádků, je-li jich více) |
| šipky, `Home` / `End` | Pohyb. `Home` přejde na první neprázdný znak, poté na sloupec 0. `Shift` rozšiřuje výběr |
| `Ctrl+Left` / `Ctrl+Right` | Posun o slovo |
| `Ctrl+Home` / `Ctrl+End` | Začátek / konec souboru |
| `PageUp` / `PageDown` | O stránku nahoru / dolů |
| `Ctrl+A` | Vybrat vše |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Kopírovat / vyjmout / vložit |
| `Ctrl+U` / `Ctrl+K` | Smazat po začátek / konec řádku |
| `Ctrl+Z` | Zpět |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Znovu |
| `Ctrl+S` | Uložit |
| `Ctrl+F` / `F3` | Hledat. V poli hledání: `Enter` / `Down` / `F3` další, `Shift+Enter` / `Up` předchozí, `Esc` zavře |
| `Alt+M` | Zapnout nebo vypnout minimapu (**Map**) |
| `Esc` | Zavřít. S neuloženými změnami se zeptá: **Uložit** (`S` / `Enter`), **Zahodit** (`D`), **Zrušit** (`C` / `Esc`) |

`Tab` v editoru odsazuje, takže nepřesouvá fokus. Ostatní kombinace `Ctrl`
(`Ctrl+P`, `Ctrl+N`) stále fungují.

**Kódování**, **konce řádků** a **odsazení** ve stavovém řádku jsou tlačítka: každé otevře rozbalovací
nabídku (šipky nebo kolečko, psaním filtrujte, `Enter` vybere, `Esc` zavře). Paleta (`Ctrl+P`) otevře stejné
nabídky: *Editor: Změnit kódování*, *Editor: Znovu otevřít s kódováním*, *Editor: Převést na kódování*, *Editor: Převést konce řádků*, *Editor: Změnit odsazení*. Podrobnosti v [Stagování a commitování](staging-and-committing.md).

## Akce Gitu (`Ctrl+G`)

`Ctrl+G` čeká na jednu další klávesu:

| Klávesa | Akce |
|---|---|
| `s` / `u` | Stage / unstage vybraného souboru |
| `a` | Stage všech změn |
| `x` | Zahodit všechny změny (nejdřív se zeptá) |
| `c` | Otevřít commit composer |
| `Enter` | Hned commitnout změny ze stage se zprávou z formuláře, odkudkoli (viz [Stagování a commitování](staging-and-committing.md#psaní-commitu)) |
| `m` | Stash všech změn se zprávou (prázdná je povolena). Se zapnutou AI napíše zprávu `Ctrl+G` v tomto promptu |
| `Shift+T` | Stash všech změn (včetně nesledovaných souborů), bez zprávy |
| `o` | Pop stashe: řádek stashe vybraný v Logu, jinak nejnovější stash |
| `p` | Pull, tak jak je nastaveno tlačítko Pull (`git.pull_mode`) |
| `Shift+P` | Push do výchozího remote (při prvním pushi nastaví upstream), nebo se zeptá na remote |
| `Shift+U` | Push všech tagů |
| `f` | Fetch všech remote, s prune |
| `g` | Otevřít pull request větve, na které jste, v prohlížeči |
| `b` / `t` | Nová větev / nový tag na HEAD (viz [Pojmenování větve nebo tagu](#pojmenování-větve-nebo-tagu)) |
| `d` | Smazat vybranou větev nebo tag (nejdřív se zeptá) |
| `e` | Upravit zprávu vybraného commitu |
| `Shift+E` / `Shift+A` | Upravit vybraný commit (všechna pole) / jeho autora (viz [Úprava commitu](#úprava-commitu)) |
| `y` | Cherry-pick vybraného commitu |
| `v` | Revert vybraného commitu |
| `Shift+R` | Reset (mixed) na vybraný commit |
| `r` | Rebase na vybranou větev |
| `Shift+W` | Otevřít nabídku Flow (repozitář s workflow větvení) |
| `Shift+H` | Otevřít nástroj pro přepis historie (jen sestavení s přepisem historie) |
| `l` | Otevřít nabídku LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch all / checkout all / pull / push all / prune. Viz [Git LFS](lfs.md) |
| `i` | ✦ AI: napsat zprávu commitu pro změny ve stage |
| `w` | ✦ AI: vysvětlit vybraný commit, nebo rozpracované změny |
| `Shift+C` | ✦ AI: navrhnout, jak rozdělit rozpracované změny do commitů |

Klávesy AI fungují jen se zapnutou AI (viz [AI asistence](ai.md)). `Esc` ruší
běžící požadavek AI.

## Pravý panel (rozpracované změny a podrobnosti commitu)

S vybraným řádkem WIP ukazuje pravý panel seznamy **Soubory unstaged** a
**Soubory staged** a commit composer. S vybraným commitem ukazuje podrobnosti
commitu a jeho soubory.

| Klávesa | Akce |
|---|---|
| `j` / `k`, `Up` / `Down` | Pohyb. Na konci jednoho seznamu výběr pokračuje do druhého |
| `h` / `l`, `Left` / `Right` | Přejít do seznamu Unstaged / Staged |
| `Enter` | Sbalit nebo rozbalit aktivní seznam |
| `Space` / `Insert` | Přidat aktuální soubor do výběru, nebo ho odebrat |
| `Ctrl+A` | Vybrat všechny soubory seznamu |
| `Ctrl+Space` | Stage / unstage vybraného souboru, nebo sbalit řádek složky |
| `d` | Otevřít diff vybraného souboru |
| `m` | `Cesta` (plochý seznam) nebo `Strom` (složky) |
| `a` | `Zobrazit všechny soubory`: celý strom, ne jen změněné soubory |
| `z` | Sbalit všechny složky, nebo je znovu rozbalit |
| `/` | Filtrovat změněné soubory commitu podle názvu |
| `.`, `Menu`, `Shift+F10` | Nabídka souboru |
| `c` | Zaostřit pole souhrnu commit composeru |
| `Shift+A` | Zapnout nebo vypnout **Amend předchozího commitu** |
| `Ctrl+N ]` / `Ctrl+N [` | Zvětšit / zmenšit blok zprávy commitu |

V commit composeru:

| Klávesa | Akce |
|---|---|
| `Enter` | Souhrn: přejít na popis. Popis: nový řádek |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` to udělá odkudkoli) |
| `Alt+R` | Vložit odkaz na issue |
| `Esc` | Opustit composer |

Nabídka souboru obsahuje: Stage nebo Unstage, Zahodit změny, Ignorovat, Stash
souboru, Historie souboru, Blame souboru, Otevřít v externím nástroji diff,
Otevřít v externím editoru, Otevřít soubor ve výchozím programu, Zobrazit ve
složce, Kopírovat cestu souboru, Vytvořit patch ze změn souboru, Upravit soubor
a Smazat soubor. U souboru v konfliktu začíná položkou **Otevřít v editoru
merge** a třemi řádky **Vyřešit pomocí**. Se zapnutým Git LFS má řádky LFS.
**Zahodit změny...** a **Smazat soubor...** se nejdřív zeptají.

## Tlačítka okna

Každé tlačítko má klávesový protějšek:

| Tlačítko | Klávesa |
|---|---|
| 🐙 vlevo v záhlaví okna (O aplikaci) | `Nápověda: O aplikaci GitPoulpi` v paletě |
| **Otevřít v** v záhlaví okna | `Alt+O`, `Ctrl+Alt+O` |
| `×` vpravo v záhlaví okna (ukončit) | `q` |
| Verze v záhlaví okna (Poznámky k vydání) | `Nápověda: Poznámky k vydání` v paletě |
| `⌕ Globální hledání` v záhlaví okna, vlevo od `⚙` | `Ctrl+Alt+F` |
| `⚙` v záhlaví okna (Nastavení) | `Ctrl+,` |
| Čip pracovního prostoru v řádku karet | `Alt+W` |
| `+` za poslední kartou | `Alt+T` |
| Karta **Radar** | `Alt+R` |
| Karta **Pracovní prostory** | `Alt+J` |
| **Aktivita** ve stavovém řádku | `Ctrl+N Shift+A` |
| Panel nástrojů: **Zpět**, **Znovu** | `Ctrl+Z`, `Ctrl+Y` |
| Panel nástrojů: **Pull** / jeho `▾` | `Ctrl+G p` / `Git: Možnosti pullu...` |
| Panel nástrojů: **Push** / jeho `▾` | `Ctrl+G Shift+P` / `Git: Push do remote...` |
| Panel nástrojů: **Větev**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Panel nástrojů: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Panel nástrojů: **LFS** (když je LFS zapnuté) | `Ctrl+G l` |
| Panel nástrojů: **Flow** (když má repozitář workflow) | `Ctrl+G Shift+W` |
| Panel nástrojů: **Hledat** | `f` |
| Panel nástrojů: propojený pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Panel nástrojů: seznam profilů (se dvěma nebo více profily) | `Repozitář: Změnit profil...` |

Panel nástrojů je popsán v [Prohlídce rozhraní](interface-tour.md#panel-nástrojů).

## Nastavení

`Ctrl+,`, `Ctrl+N ,`, ozubené kolo `⚙` nebo `Aplikace: Nastavení...`
otevře obrazovku Nastavení.

| Klávesa | Akce |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Přechod mezi seznamem stránek a volbami |
| `j` / `k`, `Up` / `Down` | Posunout výběr |
| `Enter` / `Space` | Otevřít stránku, přepnout zaškrtávací pole, upravit pole, otevřít rozbalovací seznam, nebo stisknout tlačítko |
| `/`, `Ctrl+F` | Hledat ve všech volbách |
| `Alt+Left` / `Alt+Right` | Přepnout podkartu (Obecné / seznam) |
| `Esc` / `q` | Zavřít Nastavení. Při úpravě pole `Esc` ruší jen úpravu |
| `?` | Otevřít nápovědu nad obrazovkou Nastavení |

Viz [Nastavení](settings.md).

## Textová pole

Všechna textová pole sdílejí stejné klávesy: filtry, vyhledávací pole, paleta
příkazů, prompty, commit composer, pole řádku WIP, pole dialogů a pole
Nastavení. Dokud má pole klávesnici, tyto klávesy mají přednost před zkratkami
panelů.

| Klávesa | Akce |
|---|---|
| znak | Napsat jej na kurzor (nahradí výběr) |
| `Left` / `Right` | Posun o jeden znak (s výběrem: přejít na jeho začátek / konec) |
| `Ctrl+Left` / `Ctrl+Right` | Posun o slovo |
| `Home` / `End` | Začátek / konec řádku |
| `Shift` + výše uvedený pohyb | Rozšířit nebo zmenšit výběr |
| `Up` / `Down` (jen popis) | Předchozí / další řádek |
| `Ctrl+Home` / `Ctrl+End` (jen popis) | Začátek / konec textu |
| `Ctrl+A` | Vybrat vše |
| `Backspace` / `Delete` | Smazat znak před / za kurzorem, nebo výběr |
| `Ctrl+U` | Smazat od kurzoru po začátek řádku (nebo výběr) |
| `Ctrl+K` | Smazat od kurzoru po konec řádku (nebo výběr) |
| `Ctrl+C` / `Ctrl+X` | Kopírovat / vyjmout výběr (ne v poli hesla) |
| vložení (vlastní vložení terminálu) | Vložit na kurzor, nahradí výběr |

V popisu commitu `Enter` napíše nový řádek a `Ctrl+Enter` commituje. Ne všechny
terminály posílají `Ctrl+Enter` stejně, takže commitují i `Alt+Enter` a
`Ctrl+J`.

Některé konzole posílají `Ctrl+písmeno` jako řídicí znak (`Ctrl+A` jako
`0x01`). GitPoulpi rozumí oběma tvarům. Znaky `AltGr` (`@`, `#`, `€` na
švýcarských nebo francouzských klávesnicích) se píší.

## Tooltipy

Tooltip se objeví, když ukazatel zůstane asi 0,6 s na cíli (buňka Logu,
tlačítko panelu nástrojů, propojený pull request). Stisk klávesy, klik, posun,
změna velikosti, otevřené okno nebo nabídka, nebo opuštění cíle ukazatelem ji
skryje. Poté se vrátí, až když se ukazatel znovu pohne.
