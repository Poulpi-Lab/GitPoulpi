# English (en) UI style guide

The source catalog `crates/gitpoulpi-i18n/locales/en.toml` follows this page. It was edited as a whole in 2026-10 (clarity, one term per concept, one capitalization rule per kind of text). Sources: Microsoft Writing Style Guide (sentence-style capitalization, plain verbs), Apple HIG wording, git's own documentation vocabulary, GitHub / GitLab / VS Code UI conventions. Mechanics (tokens, plurals, lints) are in `../i18n.md`; other languages have their own page next to this one.

Keep meaning stable when you fix wording: a case, punctuation or word-order change needs no translation change. A change of meaning (a fact added, removed or corrected) means every language file must follow: list the keys in the commit message.

## Capitalization: one rule per kind of text

| Kind of text | Rule | Examples |
|---|---|---|
| Palette command names | `Category: Action`, Title Case after the colon, `...` when the command does not run at once | `Git: Create Patch from Selected Commit...`, `Refs: Check Out Selected Ref` |
| Keyboard-shortcut labels (`keymap_action`) | Title Case, same words as the palette where both exist | `Stage or Unstage the Selected Row` |
| Buttons, menu rows, context-menu rows, dialog titles, Settings labels and headings, tab and column labels, tooltips | Sentence case (first word and proper names only) | `Create branch here...`, `Default branch name`, `Reset all`, `Save & mark resolved` |
| Named views and panes | Capitalized as proper names wherever they appear: Log, Graph (the column), Refs, Status, Output, Base, Home, Settings, File View, File History, Blame, Activity Log, Repository Management; "merge editor" stays lowercase (a feature, not a pane) | `Log pane`, `the Refs pane` |
| Sidebar section headers | UPPER CASE in the catalog (budget 18 cells) | `LOCAL`, `PULL REQUESTS` |
| Status-line, footer and error messages | git's voice: a short lowercase clause, no final period; proper nouns and acronyms keep their capitals (`AI`, `HEAD`, `LFS`, `Git LFS`, `SSH`) | `pulled: fast-forwarded to abc1234`, `could not open {path}: {why}`, `no remote branch known for {name}` |
| In-flight messages | lowercase gerund ending in `...` | `staging all changes...`, `fetching {remote}...` |
| Dialog text, confirmations, pane notices, banners, help prose | Full sentences with a capital and a final period (a one-line question or banner may omit the period); a confirmation names the consequence | `Discard 3 files? This cannot be undone.` |
| Help-overlay rows (`help_text`) | lowercase fragment describing the key, no final period; a row that starts with a command or a key name keeps that capital | `go to the row's commit in the Log` |
| Footer hints, "why" reasons of a grayed row, placeholders | lowercase fragments | `select a file in the Status pane first` |
| Settings values that are raw schema words (`off`, `ask`, `direct`) | lowercase, exactly as stored in `config.toml` | `ask`, `credential_helper` |
| Names of choices shown as options | Capitalized as a menu row | `Strong`, `Light`, `Pull request` |

Exceptions are deliberate and few: `{$^pr}` / `{$^prs}` tokens produce Title Case ("Create Pull Request"), so the dialog title that uses them keeps it; seed messages in `[common]` are used by the catalog tests.

## Voice and wording

- Buttons and menu rows are verb phrases in the imperative: `Create branch here...`, `Discard changes...`. No exclamation marks, no "please", no "the" in front of a bare verb.
- Messages are active and say what happened: `could not save {path}: {why}`, not `Saving of {path} has failed`. Say what to do next when there is a next step.
- Address the user as "you" only in sentences (dialogs, help prose). Most strings do not address anyone.
- One idea per sentence. Prefer plain words: "use", "start", "turn on", "check" (a checkbox), "select" (a row), "choose" (from a list), "press" (a key), "click" (a mouse button).
- Never `—` (em dash) and never `’`: use `" - "` and the plain ASCII apostrophe `'`. The typographic marks that exist are `...` (see below), the middle dot `·` as a field separator, `×`, arrows and the AI star, as glyphs.
- American spelling (color, canceled, gray, organization, behavior). "e-mail" is "email". Numbers are digits.
- Keys are written as the chord printer prints them: `Ctrl+Enter`, `Shift+Tab`, `Esc`, `Enter`, `Space`. Keymap keys inside a message use `{{ctx.action}}` so a rebinding shows.

## Ellipsis

Three ASCII dots `...` everywhere, in menus, buttons, palette names, placeholders and in-flight messages. The real `…` is not used in English.

- A command gets `...` when it does not run at once: it opens a dialog, a picker, a prompt or asks for confirmation (`Rename {name}...`, `Browse...`, `Delete {name}...`).
- A command that runs immediately has none (`Copy path`, `Stage file`, `Fetch All`).
- A submenu parent has none (`Copy`, `Reset`); the `...` goes on the row that opens the dialog.

## Terms (one word per concept)

| Concept | Write | Never |
|---|---|---|
| verb "check out" | `check out` (verb), `checkout` (noun, in "Sparse checkout", "Checkout failed") | `Checkout` as a verb |
| `cherry-pick` | `cherry-pick` (git's spelling). Five keys still say "Cherry pick" (`menu_log.cherry_pick_many`, `rebase_ui.*cherry_pick*`, `header_cherry_picking`) until the language files follow: the git-vocabulary lint ties the stem | |
| commit hash | `SHA` (uppercase) | `sha`, `commit ID` |
| uncommitted work | `working changes`, the `WIP` row in the Log | `dirty tree` in prose |
| the checked-out tree | `working tree`; the feature name "Edit in Working Directory" is an established label | mixing both in one sentence |
| folder vs directory | `folder` in the UI; `directory` only where git uses it (sparse checkout, `.git` paths) | |
| PR / MR | the `{$pr}` family; the host chooses | hard-coded "pull request" |
| profile | `profile` (author identity); an AI connection is also a "profile" but the page says "AI profile" when both are near | |
| LFS | `Git LFS` for the product, `LFS` for the feature and in compound names; `git-lfs` only for the program | |
| remote-tracking branch | `remote-tracking branch`; "remote branch" is fine in a menu row | |
| shortcut vs key vs keybinding | "shortcut" in prose, "key" for one key, "keybindings" for the page and `config.toml` table | |
| cancel vs close | Cancel abandons a dialog or an operation, Close dismisses a view or tab | |
| setting paths | `Settings > General > Language`, `Settings > Integrations` (spaces around `>`) | commas or "->" |
| "turn on / off" | for settings and features | "enable / disable" in sentences (the word "Disable" stays on the sparse-checkout button) |
| "check" a checkbox | `check` / `uncheck` | `tick` (British) |
| workspace | `workspace` (lowercase in sentences, "Workspace:" palette prefix); the tab-bar control is the "workspace chip"; "No workspace" is the empty choice | "project", "group" |
| work in progress | `Work in progress` as a Radar group heading and in prose; `WIP` only for the Log's working-changes row and the short Radar View choice | "WIP" in sentences, "in-progress work" |
| pin / snooze | `Pin to top` / `Unpin`, `Snooze...` / `Unsnooze` (hide an item for a while); status messages `pinned {item}`, `snoozed {item}: {how}`, `unsnoozed {item}` | "favorite" for pins, "mute", "dismiss" |
| pagination | `Load more` (button, sentence case) fetches the next page; its status line is `positions loaded: {n}` | "Show more", "Next page" |
| New tab | `New tab` for the `+` and its menu row (sentence case), `New Tab` for the keymap label and `Tab: New Tab` palette name | |
| request (review UI) | `request` for a pull request, a merge request or a Gerrit change when the host is not the point (review view, board, pickers); the `{$pr}` family where a list or dialog is host-specific; `item` only for a request or an issue together | "PR" in prose, "change" outside Gerrit text |
| review vocabulary | `review` (the whole), `thread` (a conversation on a line), `pending comment` (unsent, in your pending review), `Finish review` (button), `Submit review` (button inside that dialog), `verdict` (Comment / Approve / Request changes), `Reply`, `Resolve` / `Reopen`; `mark as viewed` for the file box | "draft comment", "tick"; `conversation` is only the non-code comments of the Overview, never a thread |
| AI review | `suggestion` (never a comment until accepted), `Accept` / `Edit` / `Dismiss`, `Accept all` / `Dismiss all`, `AI summary`; the ✦ star marks AI actions | "proposal", "ghost comment" in UI text |
| Gerrit | `patch set` (two words), `vote` on a label (`Code-Review`, `Verified`), `change` only as Gerrit's own noun, `push for review` | "patchset", "revision" |
| board | `board` with `column` and `card`; a card is one issue; `Move card`, `Column labels` | "kanban" except in the page title and "Obsidian Base"; "lane", "ticket" |
| metadata | `assignees`, `reviewers`, `labels`, `milestone` (lowercase in sentences, sentence case on buttons); `Edit ...` for the people and label lists, `Set milestone...` for the single choice | "owners", "tags", "assign to" |
| local issues | `local issues` for the feature, `notes folder` for the setting, `note` for one .md file, `vault` for Obsidian's | "tracker" for it |
| status lines of the new views | lowercase clause, no final period (`moved #12 to Done`, `suggestion dismissed`); pane notices and dialog notes are full sentences; why-reasons of a grayed row are lowercase fragments with no period | capitalized statuses |
| supported-host lists | name them in this order: GitHub, GitLab, Gitea, Forgejo (then Gerrit where it applies) | "GitHub and GitLab" alone, now that Gitea and Forgejo take the same features |
| branching workflow | `workflow` (the whole definition), `base branch` (long-lived; `[base]` badge), `topic type` (the kind: Feature, Hotfix) and `topic branch` (one branch of a type), `parent` (the branch it starts from and finishes into), `environment branch` (GitLab Flow), `hook` (a shell command run around an action), `issue flow` (host-driven: an issue makes the branch), `Flow` (the toolbar button and the `Flow:` palette category). Actions: `start`, `finish`, `sync` (bring the parent in), `promote` (into an environment branch), `set parent`, `pick up` (take over a branch the host or a bot made). Templates: `Git Flow`, `GitHub Flow`, `GitLab Flow`, `Trunk-based`, `Custom` | "branch type" for the concept, "long-lived branch", "update from parent", "adopt", "take over" for pick up, "git-flow" for the model (it is only the CLI value of `template`) |
| Mermaid popup | `Copy as Mermaid gitGraph`; detail levels `Structure`, `Refs`, `Subjects`, `Full` (the stored words `structure`, `refs`, `subjects`, `full` stay lowercase in config and CLI); `Direction`: `Left to right` / `Top to bottom` | "orientation" in UI text (it is only the config key), "verbosity" |
| Flow dialogs: progress, reasons, statuses | spinners are lowercase gerunds (`working out the steps...`); palette why-reasons are lowercase fragments with no period; status lines are lowercase clauses (`issue #12 created`, `done: finish feature/x`); form errors and dialog notes are sentences. A new issue is `New {label} issue...` in menus and `Flow: New {label} Issue...` in the palette | capitalized statuses, "New issue: {label}" |
| entering a value | `Enter a title` in a form error; `Type` only as a field label (the Start form's `Type` is the topic type) | "Type a title" |
| row actions that take a noun | bare noun, no article: `Open repository`, `Check out branch`, `Create worktree...`, `Show in repository` | "Open the repository", "Create a worktree" |

## Key names in the catalog

Section names are the TUI area (`menu_ref`, `status_pane`, `help_text`). Key names are lowercase snake_case. Prefixes carry the kind:

- `*_help` the long explanation shown under or beside a setting; `*_tip`, `t_*`, `tooltip_*` hover tooltips; `hint_*` footer hints; `why_*`, `reason_*` the lowercase reason a row is grayed out; `msg_*`, `err_*` status-line or error messages; `act_*` in-flight labels; `title_*`, `heading_*` titles; `btn_*`, `b_*` buttons; `noun_*`, `what_*` the thing named in "copied <what>"; `palette_*` a palette command owned by a feature section; `ctl_*` small controls; `fa_*`, `fm_*`, `fo_*` file, menu and folder rows.
- A key that is built from the English text (`global_7`, `refs_pane_1`) is a positional key in a help table; new rows get the next number.
- Plural messages use `{ one = .., other = .. }` and always carry `{n}`.

Every key gets a `# context:` comment when the English word is ambiguous (verb or noun, which host, which pane). Dev notes ("slice I11", "not built yet") do not belong in values or comments.

## Checklist for a new or edited string

1. Which row of the capitalization table is it? Apply that rule.
2. Is there already a string for the concept? Reuse its words.
3. Does the command run at once? Decide on `...`.
4. No `—`, no `’`, no `…`, American spelling.
5. Placeholders and `{$term}` tokens unchanged; widths unchanged (toolbar and Settings labels stay within 9 cells, buttons within their budget).
6. If the meaning changed, list the key for the language agents.
7. `cargo test -p gitpoulpi-i18n` and `cargo test -p gitpoulpi-tui i18n_layout`.
