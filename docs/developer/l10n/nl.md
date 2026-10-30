# Dutch (nl) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/nl.toml` follow this page. Sources: git's own Dutch translation (git-l10n), Microsoft terminology and style guide for Dutch, GitHub Desktop, VS Code, JetBrains, TortoiseGit, SourceTree. Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Informal "je/jouw", never "u" (as Microsoft, Apple, GitHub and VS Code do in Dutch). Most strings avoid addressing the user. |
| Buttons, menu rows, palette commands | Infinitive, capitalised first word: "Bestand openen", "Branch maken...". Palette: `Categorie: Actie` (colon directly after the category, no space before it, action capitalised). |
| Progress messages | Infinitive with trailing dots: "{name} bijwerken...", "Samenvoegen...". |
| Reasons for a greyed row ("why_*", "select_*") | Lower-case imperative: "selecteer eerst een branch". Errors: "Kan {path} niet openen" (present tense, not "Kon"). |
| Capitalisation | Sentence case, never English Title Case. Only the first word is capitalised, so a term token is `{$commit}` mid-sentence and `{$Commit}` only at the start. Never `{$^pr}` (it capitalises every word). |
| Titles and tabs | Nouns, short; abbreviations only when a width budget forces it ("Afbr.", "Versch."). |
| Quotes | Straight ASCII `"..."` and `'`; never curly quotes. |
| Ellipsis | `...` or `…` exactly where English uses it (dialog-opening commands). |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Dates | `{day} {month} {year}`: "5 maart 2024"; months and weekdays lower case ("maart"), abbreviations "mrt", "ma di wo do vr za zo". Relative: "3 uur geleden", "zojuist". |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Backspace`, `Delete` stay as printed by the chord printer; arrows are "omhoog/omlaag/links/rechts", Space is "Spatie". |
| Plural | `one` iff n = 1. |
| Command | "opdracht" everywhere (Microsoft term, "Opdrachtenpalet"); never "commando". |
| Toggle | "omschakelen" for a toggle command ("Minimap omschakelen"); "wisselen" only for switching between things ("Naar tabblad wisselen"). |
| Refresh | "vernieuwen" (never "verversen"). |
| Collapse/expand | "inklappen/uitklappen" (never "invouwen"). |
| Merge editor: take A/B | "overnemen" ("Huidig overnemen"), not "nemen". |
| Working changes | "lokale wijzigingen" (never "werkwijzigingen"); the folder is "werkmap". |
| Compounds | Dutch compounds are closed ("verschilweergave", "bergingsbericht", "commitbericht"); English git words take a hyphen ("commit-bericht", "merge-editor"). When the local word differs, write a `{ git, local }` pair instead of `{$term}-xxx`. |

## Genders and verb forms (the ones that bite)

Common gender (de): commit, branch, push, pull, rebase, cherry-pick, ref, remote, worktree, repository, submodule, HEAD, stash (git), pull request / merge request, berging, samenvoeging, samenvoegaanvraag, werkmap, index. Neuter (het): label, verschil, issue. The Log pane is "de Log".

Local `tag` = "label" and `diff` = "verschil" are neuter: adjectives lose the -e ("nieuw label", "geannoteerd label", "elk label", "dit verschil", "het verschil"), so messages that use them with an adjective or article are written as `{ git, local }` pairs. Local `merge` = "samenvoeging" and `fetch` = "ophalen" are a noun and a verb: use the term where a noun is needed ("de lopende samenvoeging") and a pair where a verb is needed ("{name} samenvoegen in {head}", "Alle remotes ophalen"). The merge-request term is the common-gender "samenvoegaanvraag" (not "samenvoegverzoek", which is neuter) so that "de {$pr}" and "nieuwe {$pr}" read right for both hosts.

## Glossary

`git` = default vocabulary (English git words, "Translate git terms" off). `local` = translated vocabulary (setting on). Where both columns are the same, the English word is the Dutch developer word.

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit | commit / committen | commit / committen | git-l10n and every Dutch tool keep it. "niet-gecommitte wijzigingen". |
| branch | branch | branch | Never "tak". |
| merge | merge / mergen | samenvoeging / samenvoegen | Local follows git-l10n. "samenvoegeditor", "samenvoegconflict" (local) vs "merge-editor", "merge-conflict" (git). |
| rebase | rebase / rebasen | rebase | git-l10n keeps "rebase". |
| cherry-pick | cherry-pick / cherry-picken | cherry-pick | |
| revert / reset | revert / reset | revert / reset | soft, mixed, hard stay English. |
| stash | stash / stashen | berging / in berging zetten | Local: pop = terugzetten, apply = toepassen, drop = verwijderen. Local never prints `stash@{n}`: it says "berging {id}" / "nr. {id}". |
| stage / unstage / staged / unstaged | stage / stagen / staged | same | Dutch git tools keep the English verb ("stagen", "unstagen"). |
| working tree | werkmap | werkmap | git-l10n. A linked worktree (feature) is "worktree". |
| HEAD | HEAD | HEAD | "losgekoppelde HEAD" (detached). |
| remote | remote | remote | de remote. "remote branch", "standaardremote". |
| upstream | upstream | upstream | |
| fetch | fetch / fetchen | ophalen | Noun in local mode: "ophaalactie", "ophaal-URL". |
| pull / push | pull / push | pull / push | git-l10n leaves both untranslated. |
| tag | tag | label | het label. Annotated: "geannoteerd label". Sidebar title TAGS / LABELS. |
| ref | ref | ref | |
| diff | diff | verschil | het verschil; "verschilweergave", "verschiltool". |
| hunk | hunk | hunk | |
| blame | blame | blame | "niets om te blamen". |
| conflict | conflict | conflict | "bestand met conflict", "opgelost/onopgelost". |
| ours / theirs / base | ours / theirs / basis | same | Merge editor sides are "Huidig (A)" and "Inkomend (B)". |
| pull request / merge request | pull request / merge request | pull request / samenvoegaanvraag | Host dependent. Abbreviation PR / MR. Sidebar title SAMENVOEGAANVRAGEN (18 cells, the budget). |
| issue | issue | issue | het issue. |
| log / graph | Log / grafiek | Log / grafiek | "de Log", "commitgrafiek". |
| command palette | opdrachtenpalet | opdrachtenpalet | "palet" alone in context. |
| settings | Instellingen | same | English says "Settings" everywhere; never "Voorkeuren". Stash verbs: terms `pop` terugzetten, `apply` toepassen, `drop` verwijderen; the toolbar Pop button is "Herstel" in local mode (9-cell budget). |
| repository | repository | repository | de repository; "Git-repository", "repositorymap". |
| clone / init | klonen / init | klonen / init | |
| worktree | worktree | worktree | |
| submodule | submodule | submodule | |
| LFS | Git LFS | Git LFS | "LFS-object", "pointer". |
| tab | tabblad | tabblad | "tabbladmenu". |
| avatar / minimap | avatar / minimap | avatar / minimap | |
| amend | amend / amenden | amend / amenden | "Vorige commit amenden". |
| interactive rebase actions | pick, reword, squash, fixup, drop, edit | kiezen, herformuleren, samenpersen, corrigeren, weglaten, bewerken | Tokens `{$Pick}` etc. |
| undo / redo | Ongedaan maken / Opnieuw | same | Short toolbar labels "Ongedaan" / "Opnieuw"; palette "Opnieuw uitvoeren". |
| discard | verwerpen | verwerpen | |
| WIP | WIP | WIP | |
| output (merge editor) | uitvoer | uitvoer | Never "Resultaat" for the pane. |

## Known gaps

- Local-mode baseline (`tests/git_vocabulary/nl.toml`) holds 16 keys that cannot be translated: an English prompt text for the model (`ai_page.chip_task_context_text`), keymap ids and config keys in the text (`{{diff.edit}}`, `{{diff.close}}`, `{{diff.merge_editor}}`, `{{git_prefix.stash_pop}}`, `git.pull_mode`, `general.auto_stash`, `diff.tool`, `merge.tool`, `{{merged}}`), placeholders named `{tag}` (`input.annotating_tag`, `input.creating_annotated_tag_at`, `remotes.pushing_tag`), and the sentence that lists the English git words themselves (`settings.translate_git_terms_help`). Any new message with a git word needs a `{$term}` token or a `{ git, local }` pair.
- Palette command texts are generated names; their category prefixes: Git, Refs, Remote, Repository, Repositories, Samenvoegen (local) / Merge (git), Verschil (local) / Diff (git), Navigatie, Weergave, Bewerken, Editor, Bestand, Bestanden, Lokale wijzigingen, Kopiëren, AI, App, Integraties, Hosting, Issues, Log, LFS, Blame, Geschiedenis, Afbeelding, Tabblad, Starter, Melding, Activiteitenlogboek.

## User guide (docs/user/nl)

Conventions of the translated guide, on top of the tables above:

- The guide documents the default git vocabulary ("Translate git terms" off): UI labels are quoted exactly as the `git` variant of the catalog prints them ("Ongedaan", "Opnieuw", "Pop", "Nu poppen", "Rebase starten", "Commit-bericht bewerken..."). The toolbar Undo button is "Ongedaan", never "Herstel" (that is the local-mode Pop).
- Compounds with an English git word take a hyphen as the catalog does: commit-bericht, commit-details, commit-composer, commit-grafiek, merge-editor, merge-conflict, merge-tool, stash-bericht.
- Names: tabbalk (tab row), toetscombinatie (chord; the pair "Ctrl+G dan een toets" is a toetsenreeks), sneltoets (shortcut), schuifbalk (scrollbar), keuzelijst (dropdown/combo), pictogram (icon), opdracht (command, also in "palet-opdracht" -> "opdracht van het palet"), toggle = omschakelen, vernieuwen (refresh).
- Key names are written as the chord printer prints them (`Up`, `Down`, `Left`, `Right`, `Space`, `PageUp`), also in prose.
- Pane names: "paneel Status", "paneel Refs", "de Log", "Uitvoer" (merge editor), "Basis", "Heel bestand".
- Generated tables (keymap, palette, MCP tools) are regenerated from this catalog; their column headers are translated by hand.

## Delta review (2026-10)

Decisions taken when the catalog and guide caught up with the English edits:

- Ellipses are ASCII `...` everywhere (catalog and guide), exactly where English has them; rows that open a dialog (delete, discard, close tabs, browse) end in `...`.
- Status-line messages (merge editor, "link.act_*", profile bar, compare, stuck states) start in lower case, as the English ones do, unless they begin with a placeholder, an abbreviation or a product name. Buttons and menu rows keep the capital.
- "SHA" in capitals (never "sha" or "ID") for the commit hash: "SHA van de commit kopiëren".
- Local LFS storage is "lokale LFS-opslag" (prune: "Lokale LFS-opslag opruimen"), not "repository".
- The palette entry for lost commits uses the lower-case term token: "Verloren {$commits} terughalen...".
- The new guide page `terminals.md` is "Een terminal kiezen"; terminal and protocol names (kitty, WezTerm, OSC 52, sixel) stay as written upstream, "(niet geverifieerd)" renders "(unverified)", and `ui.*` keys and shell snippets stay English.
- Undo vocabulary: "Geschiedenis van ongedaan maken" (panel), "Ongedaan maken tot hier", "Alleen dit herstellen", "Opnieuw tot hier", "Reflog", "Verloren werk".
- Palette twins of the Undo history, Reflog and Lost work row actions use the categories "Geschiedenis:" (the Undo history panel, as its tab is called), "Reflog:" and "Verloren werk:"; "Show in Log" is "Tonen in het Log", "Scan Again" is "Opnieuw zoeken". Sixel is written "sixel" (lower case) like the other protocol names.

## Workspaces, search and Radar (2026-10)

- workspace = "werkruimte" (de), "werkruimtes", chip "Geen werkruimte"; "Werkruimte: Actie" is the palette category.
- "Alles fetchen" / "Alles pullen" (git) and "Alles ophalen" (local) are written as `{ git, local }` pairs or plain text, never `Alles {$Fetch}` (that prints "Alles Fetch"). The verbs "fetchen" and "pullen" follow the glossary.
- Radar = "Radar" (product name, de Radar); scope = "Bereik", view = "Weergave", pin = "vastpinnen" / "losmaken", snooze = "uitstellen" / "Niet meer uitstellen", work in progress = "werk in uitvoering" (chip "WIP"), missing repository = "ontbrekende repository".
- Radar row actions are bare: "Repository openen", "Branch uitchecken", "Worktree maken...", "Tonen in de repository", "Werk aan issue starten...". Reflog status: "geladen posities: {n}", button "Meer laden".
- Search prefixes (b: t: c: s: w: r: pr: i: #) stay as typed; the guide calls them "voorvoegsels".

## Review, metadata, providers, local issues and board (round 4, 2026-10)

Terms chosen for the features added since round 3. Source decisions: `en.md` (request, thread, pending comment, suggestion, verdict, patch set, vote, board/column/card, notes folder).

| Concept | Dutch | Notes |
|---|---|---|
| review (noun, verb) | review / reviewen | Dutch developers keep the English word ("Review afronden", "Verzoek reviewen..."). Compounds closed: reviewtekst, codereview, AI-review, concept-review. |
| request (a PR/MR as a generic) | verzoek (het) | "Het verzoek wordt nog geladen". "aanvraag" stays only in the Radar guide's group names and in "samenvoegaanvraag" (local MR term). |
| pending comment / pending review | conceptopmerking / concept-review | Badge "Concept - regel 12", counter "3 conceptopmerkingen". Never "in behandeling". |
| comment (on code or conversation) | opmerking | "Opmerking bij regel plaatsen...". Verb "plaatsen" for the immediate post, "toevoegen aan review" for the pending one. |
| thread | discussie (de) | "Op discussie reageren...", "Discussie oplossen of heropenen". |
| conversation | gesprek (het) | The comments that are not on code. |
| resolve / reopen / resolved / outdated | oplossen / heropenen / opgelost / verouderd | |
| reply | reageren (verb), reactie (noun) | Button "Reageren". |
| approve / request changes / comment (verdict) | Goedkeuren / Wijzigingen aanvragen / Opmerking | Field "Oordeel"; badges "goedgekeurd", "wijzigingen aangevraagd", "opmerking geplaatst", "ingetrokken" (dismissed). |
| Finish review / Submit review / Discard review | Review afronden / Review indienen / Review verwerpen... | |
| viewed (file mark) | bekeken | |
| suggestion (AI) | suggestie; suggest = suggereren | "Review suggereren", Accepteren / Bewerken / Negeren (dismiss); severity info / waarschuwing / probleem. Not "voorstel" (that is the existing short toolbar label "Voorstel" for the conflict resolution). |
| patch set | patchset | Gerrit term, kept. |
| vote | stem | Gerrit labels `Code-Review`, `Verified` stay as printed. |
| assignee / reviewer / label / milestone | toegewezene (pl. toegewezenen) / reviewer / label / mijlpaal | Chip "Toegewezen" (width). GitHub's Dutch UI says "mijlpaal". |
| encoding | codering | Button "Codering", value "Auto"; list entry "Automatisch detecteren". |
| board / column / card | bord / kolom / kaart | "Issuebord", palette category "Bord:"; the button on the ISSUES header is "Bord". |
| notes folder / vault | notitiemap / vault | "Obsidian-vault", "Obsidian Base" stays. |
| local issue | lokaal issue | Palette category "Lokale issues:". |
| checks (CI) | checks | "checks geslaagd / mislukt / bezig". |
| head ref | head-ref | |
| push for review (Gerrit) | push voor review | `{$Push}` token, never a bare "pushen". |

Quotes and keys: `Shift+Left/Right` stays as the chord printer prints it; `Space` keycap is "Spatie" in the picker hints. Model prompts (`chip_*_text`) stay English.

## Branching workflows, Mermaid, Gerrit and board follow-ups (round 5, 2026-10)

Vocabulary of the `[flow]` and `[settings_workflow]` catalog sections, the `[mermaid]` popup, the CLI `flow` help and `docs/user/nl/branching-workflows.md`.

| Concept | Dutch | Notes |
|---|---|---|
| workflow | workflow (de) | The English loanword; "Workflowsjabloon", "Workflowinstellingen...", closed compounds. The undo label is `Workflow {rest}` (the label lint wants words different from English, so not `Flow {rest}`). |
| Flow (toolbar button, palette category) | Flow | Proper name like Log: `Flow: Menu openen`, "het Flow-menu". |
| base branch | basisbranch | Badges `[basis]`, `[basis, beschermd]`; "trunk" for the trunk; environment branch = "omgevingsbranch". |
| topic type / topic branch | branchtype / onderwerpbranch | Settings say "Branchtypen"; "onderwerpbranch" only in prose; the file key `topic` stays. |
| parent (branch) | bovenliggende branch | Short "bovenliggende" in field labels; never "ouder" for branches. |
| start / finish / sync / promote / set parent | starten / afronden / synchroniseren / promoveren / bovenliggende branch instellen | "afronden" matches "Review afronden". Buttons "Starten", "Afronden", "Synchroniseren", "Promoveren", "Instellen". |
| abort / continue (a stopped action) | Afbreken / Doorgaan | Palette "voortzetten of afbreken". |
| pick up (take over a host-made branch) | oppakken | Not "overnemen" (that is the merge editor's take A/B). |
| hook | hook (de), plural hooks | Git-l10n says "haak"; developers say hook. |
| name pattern, token, slug | naampatroon, token, slug | `{slug}` etc. stay Latin. |
| stacked branches | erop gestapelde branches | |
| issue flow (host-driven) | issueflow (door de host aangestuurd) | |
| request (generic PR/MR in flow texts) | verzoek | As in the review round. |
| merge strategy / tag on merge | Merge-strategie / Tag bij merge (local: Samenvoegstrategie / Label bij samenvoeging) | `{ git, local }` pairs because the local tag is neuter. Finish "Mergen in" / "Samenvoegen in". |
| "the next {$fetch}" | de volgende fetch / ophaalactie | A pair: the local fetch is a verb. |
| gitGraph detail levels | Structuur, Refs, Onderwerpen, Volledig | Stored words `structure`, `refs`, `subjects`, `full` stay lower case in config and CLI. |
| Mermaid `Direction` | Richting | "Van links naar rechts" / "Van boven naar beneden". |
| Gerrit change | wijziging | Submit = indienen, abandon = opgeven ("Wijziging indienen...", "Wijziging opgeven..."); never "laten vallen". Compare = "Patchsets vergelijken...", base = "basis". |
| Kanban moves (Jira, Azure) | overgang / staat | "workflowovergang" (Jira), "staat" (Azure DevOps state). |
