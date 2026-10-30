# Czech (cs) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/cs.toml` follow this page. Sources: git's own Czech translation (git-l10n `po/cs`), Microsoft terminology and style guide for Czech, TortoiseGit / SourceTree / JetBrains / VS Code in Czech, and what Czech developers say aloud (the English git word, declined: "po pushi", "commitu", "fetchnout"). Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Formal (vykání), plural/polite imperative in prose: "Zadejte", "Vyberte", "zkontrolujte". Never "ty" forms, never first person ("Načítám", "vracím"). |
| Buttons, menu rows, palette commands | Infinitive, capitalised first word: "Uložit", "Vytvořit větev...", "Otevřít v prohlížeči". |
| Progress | Imperfective verbal noun, no subject: "Načítání...", "odesílání {name} do {remote}...", "slučování {name}..." (capitalised for toasts/titles, lower case in status lines). |
| Results | Past passive participle: "uloženo {path}", "smazána větev {name}", "zkopírováno: ...". |
| Reasons for a greyed row (`why_*`, `select_*`) | Lower case, imperative: "nejdřív vyberte větev v panelu Refs". |
| Palette and keymap | `Kategorie: Akce`, colon directly after the category (no space before it). Categories are translated (Soubory, Zobrazení, Repozitář, Aplikace, Karta, Navigace, Změny, Integrace) except interface part names (`Git`, `LFS`, `Refs`, `Log`, `Blame`, `Remote`, `AI`, `Editor`). In local mode `Diff:` becomes `Rozdíly:` and `Merge:` becomes `Sloučení:`. |
| Capitalisation | Sentence case for labels, tabs, titles, columns ("Detaily commitu", "Protokol aktivity"); sidebar headings upper case (`LOKÁLNÍ`, `REMOTE`, `WORKTREES`); pane names stay capitalised mid-sentence ("v panelu Stav", "ve Výstupu", "panel Základ"). |
| Settings | "Nastavení" everywhere (never "Předvolby"); "předvolba" only means a preset. |
| Quotes | Czech low-high „text“ (U+201E / U+201D) for UI names and quoted values. The plain ASCII apostrophe `'` is required by the catalog test; never `’`. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Ellipsis | Same character as English (`...` or `…`) on the same strings. |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Space`, `Up/Down`, `PgUp/PgDn`, `Home/End` stay Latin (the chord printer shows them); never "Mezerník" in help text. |
| Mouse | "klik", "kliknutí", "dvojklik", "pravé tlačítko", "tažení", "najetí" (never "klepnutí"); compounds "Shift+klik", "Ctrl+klik". |
| Dates and times | `{day}. {month} {year}` with the genitive month from `[month_of]` ("5. března 2024"), standalone months and weekdays lower case ("březen 2024", "úterý"), abbreviations "led, úno, bře, dub, kvě, čvn, čvc, srp, zář, říj, lis, pro" and "po, út, st, čt, pá, so, ne"; relative "právě teď", "před 5 minutami" (instrumental: "před 1 minutou", "před 2 dny", "před 3 lety"). |
| Plurals | CLDR cs: `one` (1), `few` (2-4), `other` (0, 5+ and fractions: genitive plural: "5 souborů", "0 commitů"). Verb and participle agree: "Vybrán 1 commit" / "Vybrány 3 commity" / "Vybráno 5 commitů". |
| Generic wording | "karta" (tab), "panel" (pane), "okno" (window), "dialog", "soubor", "složka" (folder), "pracovní strom" (working tree, git-l10n; the English label "Working Directory" is "pracovní adresář"), "kontrola" (review), "poskytovatel" (provider), "přihlašovací údaje" (credentials), "hostitel" (host), "schránka" (clipboard), "daemon", "pohled na soubor" (file view), "kořen repozitáře". |

## Genders and declension (the ones that bite)

git vocabulary words are declined like Czech nouns: commit (m. inanimate) - commitu, commitem, commity, commitů; tag (m.) - tagu, tagy, tagů; stash (m.) - stashe, stashů; pull request (m.) - pull requestu, pull requesty, pull requestů; hunk (m.) - hunku, hunky, hunků. Fetch / push / pull / merge / rebase / checkout / revert / reset are indeclinable or take the noun-style suffix ("po pushi", "bez fetche", "rebase na", "mergovat"). Messages that glue a suffix to a `{$term}` token ("{$commit}u", "{$diff}em", "{$tag}u") work in local mode because commit and diff keep the same nominative stem ("rozdílem", "rozdílu"); where the local word changes gender or ending, write a per-message `{ git = .., local = .. }`: tag (m.) / značka (f.) - "Vytvořit tag" / "Vytvořit značku", "Nový tag" / "Nová značka", "smazán tag" / "smazána značka"; pull request (m.) / žádost o sloučení (f.) - "Nový pull request" / "Nová žádost o sloučení", "propojený" / "propojenou", "žádný" / "žádná", genitive "pull requestu" / "žádosti o sloučení".

## Glossary

`git` = default vocabulary (English git words, "Translate git terms" off). `local` = translated vocabulary (setting on), following git-l10n where it translates. Where both columns are the same, the English word is the Czech word.

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit (noun) | commit | commit | Czech developers say "commit"; git-l10n also keeps it. Verb in prose: "commitnout", "necommitnuté změny". Merge commit: "merge commit" / "slučovací commit". |
| branch | větev | větev | git-l10n. "vzdálená větev", "lokální větev". |
| merge | merge | sloučit / sloučení | Verb "Sloučit", noun "sloučení", "editor sloučení" (git: "editor merge"), "nástroj pro slučování", "konflikt při sloučení". |
| rebase, cherry-pick, revert, reset (soft/mixed/hard), amend, autosquash, upstream, checkout, hunk, blame, ref, SHA, LFS, worktree, WIP, fast-forward, squash, fixup, lease, prune, force | same | same | git-l10n does not translate them either (hunk is "úsek" in git-l10n, but Czech UIs use "hunk"). |
| push | push | odeslat / odeslání | TortoiseGit "Odeslat". "force push" = "vynucené odeslání". |
| pull | pull | stáhnout / stažení | TortoiseGit "Stáhnout". |
| fetch | fetch | načíst / načtení | TortoiseGit "Načíst"; progress "načítání". |
| stash | stash (m.) | odložit / odložené změny | verb "odložit", noun "odložené změny", auto-stash "automatické odložení", pop = "obnovit", apply = "použít", drop = "zahodit". The literal ref `stash@{n}` stays. |
| tag | tag (m.) | značka (f.) | git-l10n. Never "štítek": that is the issue "label" (`štítky`). Annotated: "anotovaný tag" / "anotovaná značka". |
| diff | diff (m.) | rozdíl (m.), pl. rozdíly | "zobrazení diffu" / "zobrazení rozdílů"; external diff tool: "nástroj pro porovnání". |
| stage / unstage / staged / unstaged | stage, unstage, staged, unstaged | přidat do indexu, odebrat z indexu, v indexu, mimo index | git-l10n "index". "Staging area" in prose is "index". |
| pull request / merge request | pull request / merge request | žádost o sloučení (both hosts) | GitLab Czech says "žádost o sloučení"; Azure DevOps says "žádost o přijetí změn", but the sidebar title must fit 18 cells, so local mode uses one phrase for both hosts. PR / MR abbreviations stay Latin; the "Pull Request" / "Merge Request" setting choices stay English (indistinguishable otherwise). |
| remote, upstream | remote, upstream | same | "výchozí remote", "vzdálená větev"; "vzdálený repozitář" in long prose. |
| HEAD | HEAD | HEAD | "odpojený HEAD". |
| conflict, resolve | konflikt, vyřešit | same | merge editor panes: "Aktuální" (A) / "Příchozí" (B) / "Základ" / "Výstup" (VS Code Czech). |
| repository, submodule, settings, palette | repozitář, submodul, Nastavení, paleta příkazů | same | |
| issue, label, profile, integration, token | issue, štítek, profil, integrace, token | same | "issue" stays English (GitLab Czech "problém" is rarely used by developers); issue label = "štítek". |
| undo / redo | Zpět / Znovu | same | "Zpět" also means "back"; context tells. |
| Home / Log / Refs / Status | Domů / Log / Refs / Stav | same | tab "Domů"; pane names Log and Refs stay Latin; Status pane is "Stav". |

## Known residue (lint baseline `tests/git_vocabulary/cs.toml`)

Inherent, cannot be written without the English stem: the literal ref name `stash@{n}` (auto-stash banners, pop/apply/drop messages), placeholder or key names that are code (`{tag}`, `{stage}`, `{{diff.edit}}`, `{{diff.close}}`, `{{diff.merge_editor}}`, `{{merged}}`, `{{git_prefix.stash_pop}}`, `git.pull_mode`, `git.push_behavior`, `diff.tool`, `merge.tool`, `general.auto_stash`, `ui.graph_show_stashes`), the English prompt chips sent to the model (`ai_page.chip_task_context_text`), the "Pull Request" / "Merge Request" setting choices and their help, and the help of the "Translate git terms" setting itself. The baseline may only shrink.

## User guide (docs/user/cs)

Decisions of the user-guide review (2026-10), on top of the glossary above. The guide quotes the catalog's git-vocabulary variant exactly ("Nastavení repozitáře", "Konec a cd sem", "Commit změn v N souborech", "Obnovení pull requesty a Issues (minuty)"), keys stay Latin, and the keymap and palette tables of `keybindings.md` come from the catalog.

| Topic | Decision |
|---|---|
| Ref | "ref" is masculine and declined like a noun: "vybraný ref", "každý skrytý ref", "commit s více než jedním refem", "nabídka refu". |
| Daemon, session | "daemon" (declined: daemona, daemony; never "démon"), "relace" (a session; `session:` in the title bar stays literal). |
| Footer, status bar, tooltip | "zápatí" (the bottom line, as in the catalog), "stavový řádek" (status line / bar), "tooltip" (never "nápověda" for it: that is the Help overlay). |
| Credential helper | "credential helper" (declined: "credential helperu"), as in the catalog; not "pomocník pro přihlašovací údaje". |
| Grayed | "zašedlý" for a disabled row or button, "šedý" only for a color (`gray`, "šedá" draft). |
| Tag (LFS) | The small `LFS` marker next to a file is the "tag `LFS`" (catalog), never "štítek": "štítek" is the issue label. |
| Page titles | "Stagování a commitování" (staging-and-committing), "Hostingové služby a issue trackery" (hosting-integration), "Referenční příručka CLI" / "Referenční příručka konfigurace"; running text may say "referenční příručka konfigurace" in the right case. |
| Badge, prefix | "odznak" (the `Ctrl+N` badge), "předpona" (`Ctrl+G` / `Ctrl+N`). |
| Numbers | Decimal comma in prose and tables ("32,0 MiB"); sample program output keeps the dot. |

## Delta review (2026-10, after 4c8a5709)

| Topic | Decision |
|---|---|
| Ellipsis | English moved to ASCII `...` everywhere, so the catalog and the guide use `...` too (also on the rows English newly ends with `...`: "Zavřít ostatní karty...", "Zahodit změny...", "Smazat soubor..."). |
| Status lines | Lower case first letter, like English now ("nelze uložit {path}", "otevírání {path} v editoru merge...", "staging..."); acronyms (`AI`, `LFS`) and placeholders keep their case. Toasts, titles and labels stay capitalised. |
| Undo history | "Historie vrácení" (panel), "Vrátit až sem" / "Zopakovat až sem", "Obnovit jen tuto" (Restore only this), "Ztracená práce" (Lost work), "Reflog" (stays Latin). No first person in progress text ("načítání...", "probíhá...", "hledání ztracené práce..."). |
| Uncommitted changes | "necommitnuté změny" (not "místní změny") in dialogs about local edits. |
| Checkout | English "Check out" (verb) stays the indeclinable "Checkout" in Czech labels. |
| Commit ID | "SHA commitu" (English "Commit SHA"), not "ID". |
| LFS store | "lokální úložiště LFS" (English "local LFS store"). |

## Outside changes, merge revert and undo labels (2026-10)

| Topic | Decision |
|---|---|
| Outside change | "externí" for the short tag, "provedeno mimo GitPoulpi" in sentences; status lines "zaznamenána externí změna - {key} ji vrátí". |
| Undo labels (`[undo_label]`) | Infinitive, the catalog's own action words (Checkout, Zahodit, Smazat, Vytvořit větev, Přejmenovat, Fast-forward, Reset ({mode}) na ...); git terms through `{$Stage}`, `{$Merge}`... tokens; per-message `{ git, local }` where the noun changes ("Pop stash" / "Obnovit odložené změny", "Vytvořit tag" / "Vytvořit značku"). Plurals with a counted `commit`: `one` "commit", `few` `{$commits}`, `other` `{$commit}ů` (genitive plural). Files: "1 soubor", "3 soubory", "5 souborů". |
| Merge revert | "rodič" (parent), "hlavní linie" (mainline); title "Revert merge commitu" / "Revert slučovacího commitu". |
| Reflog-style fragments | `outside_checkout` and `outside_reset` keep git's lower-case reason shape in Czech ("checkout: přesun z a na b"). |

## Workspaces, search and Radar (2026-10)

| Topic | Decision |
|---|---|
| Workspace | "pracovní prostor" (both modes; never "workspace" untranslated, never "pracovní plocha"). The tab-bar chip is "čip" (as in the catalog's "čip propojeného PR"), never "štítek" (issue label). |
| Issue in Radar | "issue" stays English everywhere (headings "Issues přiřazené mně", "Issues vytvořené mnou"); never "úkol". |
| Snooze | "Odložit..." / "Odložené" / "Přestat odkládat", as Gmail and Outlook Czech. Not to be confused with the stash verb "odložit" (different context). |
| Review requested | "Vyžádaná kontrola", state chip "kontrola" (glossary: "kontrola" = review; matches `group_awaiting`). |
| Check out branch | Radar "Checkout větve" (indeclinable Checkout, see the delta review). |
| Token + word | `{$Fetch} vše` / `{$fetch} vše` works in both modes ("Fetch vše" / "Načíst vše"); the infinitive takes an accusative object ("{$fetch} / {$pull} všechny repozitáře"). A noun phrase after `jen` takes the token in the nominative ("jen {$pr} má větev"). |
| Quoting buttons in prose | Button and field names inside sentences are quoted „takto“ so the declension of the sentence does not clash with the nominative label. |
| Status lines | Lower case, past participle first or participle last ("vytvořen pracovní prostor {name}", "do pracovního prostoru {workspace} přidáno: {repo}"), so the gender of `{repo}` never matters. |

## Code review, metadata, new providers, local issues and board (2026-10, round 4)

| Topic | Decision |
|---|---|
| Review (noun) | "kontrola" (as in "Vyžádaná kontrola"), palette category "Kontrola:", verb "zkontrolovat" ("Zkontrolovat...", "Hosting: Zkontrolovat žádost..."). "Finish review" is "Dokončit kontrolu", "Submit review" "Odeslat kontrolu", "Discard review" "Zahodit kontrolu". |
| Request | "žádost" (a pull / merge request seen in the review view; the vocabulary token `{$pr}` is used where the catalog already did). |
| Pending comment / review | "čekající komentář" (one, "{n} čekající komentáře", "{n} čekajících komentářů"), "čekající kontrola". |
| Thread | "vlákno" (Resolve "Vyřešit", Reopen "Znovu otevřít", fold "sbalit / rozbalit", outdated "zastaralé"). |
| Verdict | "rozhodnutí" (Comment "Komentář", Approve "Schválit", Request changes "Požádat o změny"); badges "schváleno", "požadovány změny", "okomentováno", "zamítnuto". |
| Suggestion (AI) | "návrh" (Accept "Přijmout", Dismiss "Odmítnout", Edit "Upravit"); severities "info", "varování", "problém". Prompt chips sent to the model stay English. |
| Patch set, vote | "patch set" (declined: "patch setu"), "hlas" (vote); Gerrit label names (`Code-Review`, `Verified`) stay. |
| Assignee, reviewer, label, milestone | "řešitel" (assignees: "Řešitelé"), "recenzent" ("Recenzenti"), "štítek" (as before), "milník". Picker footer keys stay Latin (`Space`, `Enter`, `Esc`). |
| Encoding | "kódování" (button "Kódování", value "Auto", first list row "Automatická detekce"); palette "Diff: Vybrat kódování..." (local "Rozdíly:"). |
| Checks, conflicts | "kontroly" (CI checks: "Kontroly prošly / selhaly / běží"), "konflikt s {branch}", "bez konfliktů". |
| Mentioning me | "Zmiňující mě". |
| Board, column, card | "tabule" (Kanban board; "Tabule issues", palette category "Tabule:"), "sloupec", "karta" (also the word for a tab: context tells, the board card is always next to "sloupec"). Column labels "štítky sloupců". |
| Notes folder, vault | "složka poznámek", "vault Obsidian" (declined: "vaultu Obsidian"); "Obsidian Base" stays; the provider name "Local Markdown" is shown by the UI in English and stays so in the guide. |
| Push for review | "{$Push} ke kontrole" (git "Push ke kontrole...", local "Odeslat ke kontrole..."). |
| Status lines | Lower case participle first ("{item}: řešitelé aktualizováni", "kontrola odeslána s {n} komentářem"); plural pending counts as "(čekajících: {n})" to dodge agreement. |
| Plural nouns after a number | locative plural after "v": "v {n} souboru" (one), "v {n} souborech" (few, other). |

## Branching workflows, Mermaid, Gerrit actions (2026-10, round 5)

| Topic | Decision |
|---|---|
| Workflow | "workflow" stays English and is declined like a noun (masculine, inanimate: "workflow", "nastavení workflow", "bez workflow"); also the sidebar page title "Workflow" and the toolbar button / palette category "Flow" (a product feature name, like Radar). Never "pracovní postup" (Jira-ism). |
| Base branch, topic branch, type | "základní větev" (badge `[základ]`, `[základ, chráněná]`), "tématická větev" (git-flow's topic branch), "typ tématické větve" (section "Typy tématických větví"), "Popisek" (label of a type), "ID" (the stable short name), "Vzor názvu" (name pattern), "zástupný symbol" in prose, "token" in the UI. |
| Parent | "rodič" (labels, tooltips: "Rodič: develop", "Nastavit rodiče...", "Synchronizovat s rodičem..."), the same word as the merge-revert parent. |
| Start / finish / sync / promote | "Zahájit" (never the loanword "Startovat"), "Dokončit", "Synchronizovat", "Povýšit" (GitLab Flow environment promotion); noun forms "zahájení", "dokončení", "synchronizace", "povýšení". "Set parent" is "Nastavit rodiče". Continue / Abort in a stopped action: "Pokračovat" / "Přerušit" ("Zrušit" is only for cancelling a dialog). |
| Environment branch | "větev prostředí". |
| Finish mode / target | "Režim dokončení", "Cíle dokončení"; values "{$Merge} lokálně" (local merge), "Otevřít žádost na hostiteli", "Pokaždé se zeptat". Merge strategy label is a per-vocabulary message ("Strategie merge" / "Strategie sloučení"): a bare `{$merge}` token would give the infinitive "Strategie sloučit". |
| Hook | "hook" (declined: "hooky", "hooků", "po selhání hooku"); git-l10n keeps it. |
| Pick up / take over | "Převzít" (a branch the host or a bot made), progress "přebírání větve...", status "převzato: {name}". |
| Host, bot | "hostitel" (the hosting service), "bot" (declined: "botem"). "Branch made by" is "Větev vytváří" with nominative values ("hostitel, hned po vytvoření issue", "vy (bez automatizace hostitele)"). |
| Issue flow | "Tok issue (řízený hostitelem)" for the host-driven issue flow; "issue" stays English and indeclinable-looking ("nové issue", "pro issue #12"). A request on the host in these dialogs is "žádost" (`!5`). |
| Ahead / behind | "Napřed o {n} commitů" / "Pozadu za {parent} o {n} commitů" (tooltips); "Aktuální vůči rodiči" for up to date. |
| Mermaid gitGraph | Titles keep "Mermaid gitGraph" and "Log" Latin ("Zkopírovat jako Mermaid gitGraph", "v Logu"). Detail levels "Struktura", "Refs", "Předměty", "Úplné"; "Směr" (Direction) with "Zleva doprava" / "Shora dolů"; "Detail"; the fence checkbox "Zahrnout blok ```mermaid". Merge commits at `structure` are "merge {$commits}" (git) / "slučovací {$commits}" (local). |
| Gerrit submit / abandon | Kept as Gerrit's own words "Submit" and "Abandon" (button, palette "Kontrola: Submit změny...", "Kontrola: Abandon změny..."; verbs "submitnout", "abandonovat"): "Odeslat" would clash with push, and Gerrit's UI is English. Status lines "{id}: submit proveden". Compare patch sets: "Porovnat s", "Porovnat: základ", "Porovnat: patch set {n}". |
| Template names | `Git Flow`, `GitHub Flow`, `GitLab Flow`, `Trunk-based`, `Custom` stay as the preset names (they are code identifiers shown in the UI). |
| Undo wording | "Vrácení neobnoví:" (Undo will not take back); undo label "Flow {rest}" stays Latin like the other action words. |
