# Swedish (sv) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/sv.toml` follow this page. Sources: git's own Swedish translation (git-l10n, read from the installed `git.mo`), Microsoft terminology and style guide for Swedish, VS Code, JetBrains, TortoiseGit, and SourceTree. Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Informal "du/din", never "ni" (Microsoft, Apple and GitHub use "du"). Most strings avoid addressing the user. |
| Buttons, menu rows, palette commands | Bare imperative, capitalised first word: "Öppna fil", "Skapa gren...". Palette: `Kategori: Åtgärd` (colon directly after the category, no space before it, action capitalised). |
| Status lines | Lower-case first letter when English's is lower case ("sparar {path}...", "kunde inte öppna {path}"); proper nouns (AI, LFS, WIP) keep their capital. |
| Progress messages | Present tense: "Hämtar {remote}...", "Sparar...". Done messages: past tense ("tog bort grenen {name}"). |
| Reasons for a greyed row ("why_*", "select_*") | Lower-case imperative: "välj först en gren". Errors: "Kunde inte öppna {path}". |
| Capitalisation | Sentence case, never English Title Case. A term token is `{$commit}` mid-sentence and `{$Commit}` only at the start of the string or after `: `. Never `{$^pr}` mid-sentence (the normaliser turns it into `{$pr}`, at the start `{$Pr}`). Feature names stay capitalised (Log, Refs, Status, Blame). |
| Quotes | Prose uses the Swedish quote ”...” (U+201D on both sides); the plain ASCII apostrophe `'` is required by the catalog test. |
| Ellipsis | ASCII `...` exactly where English uses it (dialog-opening commands); never the single character. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Dates | `{day} {month} {year}`: "5 mars 2024"; months and weekdays lower case; month abbreviations stay three letters for the column width ("mar", "jun", "jul"; SAOL would write "mars", "juni", "juli"); weekdays "mån tis ons tors fre lör sön". Relative: "nyss", "för 3 timmar sedan". Percent with a space ("50 %"). |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab` stay as the chord printer shows them (never "Skift", "Retur"); other keys by Swedish names: "Mellanslag", "Upp/Ned", "Höger/Vänster". |
| Plural | `one` iff n = 1. |
| Edit | "Redigera" for edit (button, menu, palette); "ändra" only for change/modify in running text ("ändra storlek"). |
| Working changes | "Lokala ändringar" (palette category and title), "oincheckade ändringar" for uncommitted. |
| Unbound | "obunden" / "avbinda" (not "olöst", which means unresolved). |
| Compounds | Swedish compounds are closed: "incheckningsmeddelande", "sammanslagningskonflikt". English git words take a hyphen ("commit-meddelande", "merge-konflikt"). A token never sits inside a compound in local mode: write a `{ git, local }` pair ("Commit-meddelande" / "Incheckningsmeddelande"). |

## Genders (the ones that bite)

Common gender (en): commit/incheckning, gren, tagg/tag, stash, sammanslagning, diff, hunk, ref, remote, worktree, submodul, fil, flik, ruta, konflikt, pull request. Neuter (ett): repository ("repositoryt"), ärende (issue), index ("indexet"). Definite forms of the git nouns are not available through a token ("commiten" / "incheckningen"): write a `{ git, local }` pair. Verbs need pairs too, because the local term is a noun: "Slå ihop {name} i {head}" (local) vs "Merge {name} i {head}" (git), "Hämta" (local) vs "Fetch" (git).

## Glossary

`git` = default vocabulary (English git words, "Översätt git-termer" off). `local` = translated vocabulary (setting on). Where both columns are the same, the English word is the Swedish developer word and git-l10n keeps it too.

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit | commit / commita | incheckning / checka in | git-l10n. "oincheckade ändringar", "incheckningsgrafen". |
| branch | gren | gren | git-l10n, VS Code. Never "branch" in running text. |
| merge | merge / merga (also "slå ihop") | sammanslagning / slå ihop | git-l10n. Merge editor: "Merge-redigerare" / "Konfliktredigerare"; "Sammanslagning:" palette category. |
| rebase | rebase / göra rebase | rebase | git-l10n has "ombasera"; not used (nobody says it). |
| cherry-pick, revert, reset | same | same | soft, mixed, hard stay English. Verb: "Revertera". |
| stash | stash / stasha | stash | git-l10n keeps "stash" (”stash”-post). pop = "poppa", apply = "tillämpa", drop = "droppa". |
| stage / unstage / staged / unstaged | stage / stagea / staged / unstaged | same | git-l10n has "köa/köad"; Swedish developers say "stagea". |
| working tree | arbetskatalog | arbetskatalog | git-l10n. A linked worktree (feature) is "worktree". Never "arbetskopia". |
| HEAD | HEAD | HEAD | "frånkopplad HEAD" (detached), as in git-l10n. |
| remote | remote | remote | en remote. git-l10n "fjärr(arkiv)" not used. "remote-gren", "standard-remote". |
| upstream | upstream | upstream | |
| fetch | fetch | hämtning / hämta | git-l10n. Noun in tokens, verb in pairs. "Hämta alla". |
| pull / push | pull / push (pulla, pusha) | same | git-l10n has no pull term; "sända" for push is not used. |
| tag | tag | tagg | git-l10n. Annotated: "annoterad tagg". Sidebar title TAGS / TAGGAR. |
| ref | ref | ref | |
| diff | diff | diff | git-l10n keeps "diff". Never "skillnader". |
| hunk | hunk | hunk | git-l10n has "stycke"; not used. |
| blame | blame | blame | |
| conflict | konflikt | konflikt | "fil med konflikt", "olöst/löst". |
| ours / theirs / base | ours / theirs / Bas | same | Merge editor sides: "Nuvarande (A)", "Inkommande (B)", pane "Resultat" (output). |
| pull request / merge request | pull request / merge request | same | Host dependent, never translated: a translated "merge request" does not fit the 18-cell sidebar title. Abbreviation PR / MR. |
| issue | ärende | ärende | Jira/Azure term. |
| log / graph | Log / graf | Log / graf | "Log" is the pane name; "grafen". |
| command palette | kommandopalett | kommandopalett | "paletten" in context. |
| settings | Inställningar | Inställningar | English source mixes Settings and Preferences; both are "Inställningar". |
| repository | repository | repository | ett repository. git-l10n "arkiv" and Microsoft "lagringsplats" are not used; "repo" in short strings. |
| clone / init | klona / init | klona / init | |
| worktree / submodule | worktree / submodul | worktree / submodul | |
| LFS | Git LFS | Git LFS | "LFS-objekt", "pekare". |
| tab | flik | flik | |
| avatar / minimap | avatar / minikarta | avatar / minikarta | |
| amend | amend | Ändra | "Ändra" is the verb in commands (menu, palette, keymap: "Ändra...", "Ändra författare..."); "amend" stays in running text. |
| interactive rebase actions | pick, reword, squash, fixup, drop, edit | plocka, skriv om, slå ihop, rätta, ta bort, redigera | Tokens `{$Pick}` etc. |
| undo / redo | Ångra / Gör om | same | |
| discard | förkasta | förkasta | Never "kasta". |
| undo history terms | Ångrahistorik, Reflog, Förlorat arbete | same | "Återställ bara den här" (restore only this), "Ångra hit" (undo up to here), "Rädda förlorade commits" (recover lost commits), "dinglande" (dangling), drop = "droppa" also in undo texts. |
| WIP | WIP | WIP | |

## Known gaps

- Local-mode baseline (`tests/git_vocabulary/sv.toml`) is not empty for reasons that cannot be fixed in the catalog: (1) the local tag word "tagg" starts with the stem "tag" (git-l10n's word, so the lint reads it as English); (2) `pull request` / `merge request` are deliberately untranslated and "merge request" holds the stem "merg"; (3) placeholders named `{commit}` / `{tag}`; (4) AI prompt texts for the model; (5) keymap ids and config keys in prose (`{{merged}}`, `merge.tool`, `git.pull_mode`); (6) the sentence that lists the English git words (`settings.translate_git_terms_help`). Any new message with a git word needs a `{$term}` token or a `{ git, local }` pair.
- Palette command texts are generated names; their category prefixes: Git, Refs, Remote, Repository, Repositories, Sammanslagning (local) / Merge (git), Diff, Navigering, Vy, Redigera, Redigerare, Fil, Filer, Lokala ändringar, Kopiera, AI, App, Integrationer, Hosting, Ärenden, Log, LFS, Blame, Historik, Bild, Flik, Startare, Avisering, Aktivitetslogg.

## User guide (`docs/user/sv/`)

Decisions of the user-guide review (2026-10). Labels quoted in the pages are the catalog's git-vocabulary texts, character for character.

| Topic | Decision |
|---|---|
| Address | "du" (never "ni"); imperative for steps ("klicka", "tryck"). |
| Working changes | "lokala ändringar" in prose, "Lokala ändringar" as the heading, never "arbetsändringar". The working tree is "arbetskatalogen" (never "arbetskopia", "arbetsträdet"). |
| Bars | The bottom line of the window is "sidfoten" (footer) or, for the status messages and badges, "statusraden"; the editor's own bar is "statusfältet". |
| Pills, banners | "pill" / "pills" (as the catalog's "ref-pill"), never "piller"; "banner" / "bannrar". |
| Hunk | Plural "hunks" (catalog). Verb forms: "stagea / stagear / stageade / stageat", "unstagea". |
| Panes | "ruta" for the Refs / Log / right-hand areas ("Status-rutan"), "panel" only in "högerpanelen", "sidopanelen". Merge editor output is "Resultat". |
| Hosting | Page and links: "Värdtjänster och ärendehantering"; the settings group is "Hosting" (catalog). |
| Keys page | "Kortkommandon"; "tangentföljd" for a chord ("Ctrl+G och sedan s"), "märke" for the `Ctrl+N` badge. Key names `Shift`, `Enter`, never "Skift", "Retur". |
| Mutations | "verktyg som ändrar något" in prose; the config keys keep `allow_mutations`, `mutation_allowlist`. |
| Code blocks | Comments inside shell and TOML examples are translated; commands, keys, values and JSON field names are not. The generated blocks (keymap, palette) are regenerated from the catalog, git variant; the key column stays English as in every language. MCP tool table: names stay English, header and "Varför" column are Swedish. |
| Decimals | Comma in prose and tables ("32,0"). |
| Window names | "Om-fönstret", "Inställningsskärmen" (compound, lower case in the middle of a sentence). |

Catalog points the review could not change (reported): `settings_general.tab_dir_follow` reads "Följ aktiv flikes mapp" (should be "Följ aktiv flikens mapp"); the docs quote it as it is.

## Undo labels and outside changes (2026-10)

Settled terms: "utanför" (tag for an operation made outside GitPoulpi; "ändring utanför" in prose), "Spela in ändringar gjorda utanför GitPoulpi" (setting), "huvudlinje" (mainline of a merge revert; "förälder N" for the parent), "spets" (tip), "I dag" / "I går" (day headings). `[undo_label]` rows are imperative and reuse the catalog's verbs: commit = "Commita" / "Checka in", merge = "Merge" / "Slå ihop", revert = "Revert" / "Revertera", fetch = "Fetch" / "Hämta", stash verbs through the `{$Pop}`, `{$Apply}`, `{$Stash_drop}` tokens, stage/unstage through `{$Stage}`/`{$Unstage}`. Reflog-style texts of outside entries (`outside_commit`, `outside_reset`) keep git's own shape ("commit: ...", "reset: flyttar till ..."). Help texts of `reflog`, `undo-restore`, `lost-commits` are translated; the help file uses "arbetskatalog" throughout.

## Workspaces, search, Radar (2026-10)

| Concept | Decision |
|---|---|
| workspace | "arbetsyta" (pl. "arbetsytor") everywhere; palette category "Arbetsyta:", prose compounds "arbetsytesida", "arbetsyte-chip". Never for the git working tree (that stays "arbetskatalog"). |
| Plural of repository | "repositories" (never "repository" as a plural: "alla repositories i arbetsytan"). |
| Radar | Feature name, stays "Radar"; palette category "Radar:". scope = "Omfång", view = "Vy", show = "Visa". |
| pin / snooze | "fäst" / "lossa"; "snooza", "Sluta snooza". Status lines in past tense: "fäste {item}", "slutade snooza {item}". |
| Radar actions | Bare imperatives, no definite forms: "Öppna repository", "Checka ut gren", "Skapa worktree...", "Visa i repository", "Börja arbeta med ärende...". |
| Fetch all | `{ git, local }` pair: "Fetch alla" / "Hämta alla" (never `{$Fetch} alla`: the local noun "hämtning" reads wrong). `{$Pull} alla` is fine, pull has no local term. |
| Workspace done messages | Past tense verb first: "skapade arbetsytan {name}", "lade till {repo} i arbetsytan {workspace}". |
| New tab, Reflog | "Ny flik"; "Ladda fler" (Load more), status "inlästa positioner: {n}". |
| pull/merge request kinds | Search kind: `{ git = "{$Prs}", local = "Ändringsbegäranden" }`; "begäran/begäranden" in prose when the host term is not wanted. |

## Review, metadata, board, local issues (round 4, 2026-10)

| Concept | Decision |
|---|---|
| review | "granskning" (verb "granska"); palette category "Granskning:". Code review: "Kodgranskning". Never "review" in running text. The in-app view is "granskningsvyn". |
| request (the host-neutral word for pull/merge request/change) | "begäran" (pl. "begäranden"), as in the round-3 search kind; `{$pr}` where the host word is wanted. |
| pending comment / pending review | "väntande kommentar" / "väntande granskning" (not "preliminär", "utkast": a draft is the Gerrit word and stays out of the UI). |
| thread | "tråd" (pl. "trådar"); "konversation" for the non-code comments of a request. |
| resolve / reopen | "lös" / "öppna igen"; states "löst", "föråldrad" (outdated). Never "avsluta". |
| Finish review / Submit review / Discard review | "Slutför granskning" (opens the form) / "Skicka granskning" / "Förkasta granskning". |
| verdict | "omdöme" (Approve = "Godkänn", Request changes = "Begär ändringar", Comment = "Kommentera"); badges "godkänd", "begärde ändringar", "kommenterade", "avfärdad". |
| suggestion (AI) | "förslag"; Accept = "Godta", Dismiss = "Avfärda" ("avfärda" never "ignorera"). AI review block: "AI-granskning". |
| patch set / vote | "patch set" (Gerrit's word, kept, pl. "patch sets"); "röst" (Code-Review votes). |
| viewed (file) | "visad". |
| assignee / reviewer / label / milestone | "tilldelad" (pl. "tilldelade", the GitHub Swedish term) / "granskare" / "etikett" / "milstolpe". Menu: "Redigera tilldelade...", "Redigera granskare...", "Redigera etiketter...", "Ange milstolpe...". |
| Apply (picker button) | "Verkställ" (Microsoft term for Apply). |
| encoding | "teckenkodning"; the auto choice is "Autoavkänning" (button "Auto"). |
| board / column / card | "tavla" (palette category "Tavla:"; "Ärendetavla" in titles) / "kolumn" / "kort". Move = "Flytta", "Kolumnetiketter" for the label-columns dialog. Key names "Shift+Vänster/Höger". |
| notes folder / vault | "anteckningsmapp" / "valv" ("Obsidian-valv"; `.obsidian` stays). Palette category "Lokala ärenden:". "Obsidian Base" stays English (feature name). |
| open / closed columns | "Öppna" / "Stängda" (plural, matches the cards they hold). |
| Mentioning me | "Nämner mig". |
| checks | "kontroller" ("Kontroller godkända / misslyckades / pågår"). |

Text sent to the model (`review_ai.chip_*_text`) stays English, as in the other languages.

## Branching workflows, Mermaid, Gerrit follow-ups (round 5, 2026-10)

| Concept | Decision |
|---|---|
| workflow | "arbetsflöde" (palette and Settings page "Arbetsflöde"; "grenarbetsflöde" for the whole feature and the page title "Grenarbetsflöden"). The toolbar button and palette category stay "Flow". |
| base branch | "basgren" (badge `[bas]`, `[bas, skyddad]`); "bas" alone in short labels. |
| topic type / topic branch | "grentyp" (the kind) / "ämnesgren" (one branch of a type); "Hittade grentyper" in detection. Never "topic" in running text. |
| parent / parent branch | "förälder" (label), "föräldergren" in running text; "Ange förälder..." for set parent. Same word as git-l10n's commit parent. |
| environment branch | "miljögren". |
| start / finish / sync / promote | "Starta" / "Slutför" / "Synka" / "Promota". "Slutför" is also the word for finishing a review, which is fine: both are completing an action. The noun "slutmål" is a finish target. |
| pick up | "plocka upp" (take over a branch the host or a bot made); "Ta över" only as the form label of the target. |
| hook | "hook" (Swedish developers say it; git-l10n's "krok" is not used). |
| issue flow | "ärendeflöde"; "hostdriven" for host-driven; "hosten" (definite, common gender) for the host; "bot" stays. |
| Mermaid detail levels | Structure / Refs / Subjects / Full = "Struktur" / "Refs" / "Ämnesrader" / "Allt"; the stored words `structure`, `refs`, `subjects`, `full` stay as config and CLI values. "Detaljnivå" for Detail, "Riktning" for Direction (Vänster till höger, Uppifrån och ned). A commit subject is "ämnesrad". |
| Gerrit submit / abandon | "Skicka in" / "Överge"; a Gerrit change is "ändring" (never "ändringsbegäran"). Compare patch sets = "Jämför patch sets". |
| detect / use detected | "Identifiera från grenarna" / "Använd det identifierade arbetsflödet". |
| Save for me only | "Spara bara för mig". |
| slug | "slug" stays (it is a token name in the patterns). |
| Flow dropdown | "rullgardin" for the toolbar's Flow dropdown in prose; "kombinationsruta" for combobox. |
| Reasons and statuses | Lower-case fragments for palette reasons and status lines ("klart: {what}", "avbrutet", "skapar ärendet..."); errors and form notes are sentences ("Ange en titel"). |
