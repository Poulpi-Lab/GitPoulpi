**English** · [Čeština](cs/ai.md) · [Deutsch](de/ai.md) · [Español](es/ai.md) · [Français](fr/ai.md) · [Italiano](it/ai.md) · [日本語](ja/ai.md) · [한국어](ko/ai.md) · [Nederlands](nl/ai.md) · [Polski](pl/ai.md) · [Português (Brasil)](pt-BR/ai.md) · [Português (Portugal)](pt-PT/ai.md) · [Русский](ru/ai.md) · [Svenska](sv/ai.md) · [Türkçe](tr/ai.md) · [Українська](uk/ai.md) · [简体中文](zh-CN/ai.md)

# AI assistance

GitPoulpi never needs an account, and **AI is off by default**. When AI is on,
it can write commit and stash messages, explain changes, write pull request
descriptions, suggest merge conflict resolutions and split your working changes
into several commits. The model is yours: an API key that you own, a local
server, or an agent CLI that is already installed (such as `claude -p`).

Every AI control shows the purple `✦`. While AI is off:

- Nothing is sent anywhere.
- No AI control is shown: no `✦` button, no `AI:` palette command, no AI menu
  row and no AI key hint. The AI keys do nothing.
- The repository AI page shows only "AI is turned off (Settings > AI)".
- The MCP server does not list the `ai_*` tools. The `gitpoulpi ai ...` CLI
  commands refuse with the same message.

**Enable AI** takes effect at once. You do not need to restart.

There are two ways to use AI with GitPoulpi:

| | Built-in AI (this page) | An external agent |
|---|---|---|
| What | GitPoulpi asks a model for a commit message, an explanation, ... | Claude Code, Cursor, Codex... drive GitPoulpi as a tool |
| Set up in | Settings > AI (`[ai]`) | the agent's MCP config, limits in `[mcp]` |
| Guide | below | [MCP and agents](mcp-and-agents.md) |

## Configure the AI

1. `Ctrl+,` > **AI** > check **Enable AI**.
2. The profiles are the list of the **Profiles (connections)** tab, on the
   left of the page (stacked above on a narrow terminal): pick one to edit it on the right (`Enter` moves into its
   rows; the palette's **AI: Edit Profile...** jumps to the list). **+ Add**
   makes another, **Duplicate** copies the selected one (without its key),
   and **Remove** asks first. Pick the
   **Provider** and fill its fields:

   | Provider | Fields | Defaults |
   |---|---|---|
   | **Local agent** (no key, no account of its own) | **Agent type** (which agent it is), **Command** (the exact program: on your `PATH`, an alias or wrapper script, or a full path), **Arguments** that make it non-interactive, and optional **Environment variables**. The prompt goes on its stdin, the answer is read from stdout. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **API key**, optional **Model** and **Base URL** | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **API key**, Model, Base URL | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **API key**, Model, Base URL | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **OpenAI-compatible** (Ollama, LM Studio, vLLM, a company gateway) | **Base URL** (`.../v1`), **Model**, optional key | `http://localhost:11434/v1` (Ollama; LM Studio is `http://localhost:1234/v1`), `llama3` |

   **Presets** (you can always type another value):

   ![The Agent type dropdown of a local agent profile, listing Claude Code, OpenAI Codex CLI, Gemini CLI, opencode and the other agent CLIs](images/settings-ai-presets.png)

   - **Local agent > Agent type** is a dropdown of the common agent CLIs, most
     used first: Claude Code, OpenAI Codex CLI, Gemini CLI, opencode, Cursor
     Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose, Crush, Amp, or
     *Other (no suggestions)*. The type is **separate from the command**: it
     only chooses which suggestions you get. Picking one pre-fills **Command**
     (while it is still the previous type's default) and **Arguments** with the
     headless invocation (`claude -p`, `codex exec -`, `gemini -p ...`, ...).
     **Command** is free text: write `/opt/bin/work-claude`, a wrapper script
     or an alias target and the type, with its options, model list and
     variables, stays. When **Agent type** is empty, GitPoulpi infers it from
     the command.
   - **Argument options** adds or removes options that suit GitPoulpi's one-shot
     use, each with a one-line reason: no tools, no saved session, plain-text
     output, low reasoning effort, read-only mode, no permission prompts, no
     update check or analytics. The list differs per agent because the agents
     differ (only some can switch tools off from the command line; for opencode
     that is an environment variable). **Model** lists that agent's models and
     writes the right flag (`--model haiku` for Claude Code, `-m gpt-6-luna`
     for Codex, `--model provider/model` for opencode); *(default - no flag)*
     removes it. Amp has no model flag. `aider` reads its prompt from
     `/dev/stdin`, so it works on Linux and macOS only.
   - **Environment variables** are set for that agent's process only, never for
     GitPoulpi or other profiles. **Add an environment variable** offers the
     documented variables that matter for this agent, each with what it does
     (disable telemetry and update checks, effort level, config directory,
     API key, base URL or local host, ...); you can also type `NAME` or
     `NAME=value`. Every variable then has its own **Env NAME** row: its value
     as a button (press it to type a new one; it saves when you leave the
     field) and a **[x]** button that deletes the variable. A value may be `${env:NAME}` to borrow one of GitPoulpi's own
     variables (handy for API keys, so they are not written to
     `config.toml`). Values that look like secrets (names ending in `_KEY`,
     `_TOKEN`, `_SECRET`, ... or well-known key prefixes) show as bullets and
     are typed from scratch. Values are never logged.
   - **HTTP providers > Provider preset** fills the base URL and a model for
     OpenAI, Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek, Mistral, Groq,
     Together, Fireworks, Cerebras, Perplexity, Moonshot, Qwen (DashScope), Zhipu
     (GLM) and the local servers Ollama, LM Studio, llama.cpp, vLLM, Jan and
     LocalAI. **Model** is a dropdown of that provider's models, free text
     allowed.

3. Press **Test connection**: *Last test* reads `connected: the model answered`
   or `failed: <reason>`.
4. With several profiles, choose the **Default profile** at the bottom; each
   action can still name its own.

![The Profiles block of the AI page for a local agent: Agent type (Claude Code), Command, Arguments, Model, two Env rows, Add and Remove an environment variable, Test connection and Default profile](images/settings-ai-agent.png)

**API keys** are entered with `Enter` on the **API key** row and stored in your
system credential helper (Git Credential Manager, libsecret / `pass`, Keychain)
under `ai:<profile id>`, never in `config.toml`. Without a helper GitPoulpi says
so instead of pretending to keep it. *Settings > SSH & credentials > Forget all
Usernames and Passwords* removes them.

The same in `config.toml` (everything else is in the [configuration
reference](config-reference.md#ai-built-in-ai-off-by-default)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # free: an alias, a wrapper script, an absolute path
harness = "claude"        # which agent it is; empty = inferred from `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # this agent's process only; ${env:NAME} borrows ours
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Run mode: headless or virtual terminal

A local agent profile runs its agent one of two ways, chosen by **Run mode**
(shown for Claude Code, OpenAI Codex CLI and Gemini CLI):

- **Headless (non-interactive)**, the default: the agent's one-shot mode
  (`claude -p`), prompt on stdin, answer on stdout.
- **Interactive (virtual terminal)**: GitPoulpi starts the agent's own screen
  in a hidden terminal, types the request in as a person would, waits for the
  answer and closes the agent. Some agents bill or limit their headless mode
  differently from their interactive one; which applies to you, and what the
  provider's terms say about either, is up to the provider, so check theirs.

Each mode keeps its own **Arguments**, **Argument options** and **Model**:
switching modes loses nothing, and the interactive mode only offers the
options that make sense without the headless flag. Two more rows apply to both
modes:

- **Working folder**: *Dedicated folder* is an empty folder GitPoulpi owns, so
  the agent sees no project and asks you to trust it once. *Current
  repository* gives the agent the project's own context (its `CLAUDE.md`, for
  example), and it asks for trust once per repository. *Default* is the
  dedicated folder in interactive mode and, in headless mode, wherever
  GitPoulpi runs, as before.
- **Timeout (seconds)**: how long the agent may take before GitPoulpi gives up
  and stops it. Empty means 120 seconds headless and 180 interactive.

**First-run questions are yours to answer.** An agent that has never run in a
folder asks something first (trust this folder, sign in, pick a theme).
GitPoulpi never answers for you: the request stops with a message, and **Open
the agent...** (on the profile, or *AI: Open the Agent* in the command palette)
opens the agent once, in a visible terminal, in the working folder it will use.
Answer the question, close the agent, and try again; the answer is remembered
by the agent.

The answer is read from the agent's session transcript for Claude Code (its
exact text, deleted afterwards) and from the screen between two marker lines
the request asks for, for Codex and Gemini. At most two virtual-terminal runs
happen at once, and pressing Esc during a request stops the agent.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # the headless set, kept
run_mode = "terminal"               # headless (default) | terminal
terminal_args = ["--effort", "low"] # the interactive set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 or absent: 120 headless, 180 terminal
```

## Instructions

Text you add to what the model is told. The built-in prompt of each action
(which fixes the answer's format) always comes first and cannot be edited; then
four levels, from the most general to the most specific, each only when not
empty:

1. **Global instructions** (every action). The editor's *Quick insert* chips
   insert common sentences: *Respond in `<language>`*, *Keep responses concise*,
   *Use imperative mood*, *Technical tone*, *Casual tone*.
2. **Workspace instructions** (every action, every repository of the
   [workspace](workspaces.md#ai-instructions-of-a-workspace)): for all actions,
   and optionally per action. Edited in the workspace's Edit form.
3. **Repository instructions** (every action, this repository only), with
   *Combine with global*: **Add to global** (after them) or **Replace global**
   (instead of them).
4. **Instructions for the action** (*Commit message generation*, *Explain
   changes*, *Stash message generation*, *Pull Request*, *Commit composer*,
   *Conflict resolution*, *Code review*), each
   block with its own chips in its editor (e.g. *Conventional commits format*,
   *50 character limit*) and its own **Profile** dropdown (`Default (<name>)` follows the
   default profile).

`Enter` on a *Custom Instructions* row opens a multi-line editor (`Ctrl+S` saves,
`Esc` cancels). Above the text, a *Quick insert* row holds the chips of that
section: click one to insert its sentence at the caret. With the keyboard, `Tab`
moves to the row, `Left`/`Right` choose a chip, `Enter` or `Space` inserts it and
returns to the text (`Tab` or `Esc` return without inserting). A chip whose
sentence is already in the instructions is dimmed and inserting it again does
nothing. The text is a full editor: click places the caret, a drag or `Shift`+arrows
select, `Ctrl+A` selects all, `Ctrl+C` / `Ctrl+X` copy and cut the selection,
`Ctrl`+arrows jump words, `Home` / `End`, `PageUp` / `PageDown` and the
mouse wheel move through long text. The page itself shows only a preview of
each instruction.

![The AI page: global instructions, then the Commit Message Generation and Explain Changes blocks with a profile dropdown and custom instructions](images/settings-ai.png)

**Per repository**: *Settings > Repo-specific settings > AI* (while a
repository is open) holds the repository instructions and, per action, a profile
and instructions that override the global ones. An empty field uses the global
setting (shown grayed in it); **Reset to global** removes an override. Saved in
the repository's `.gitpoulpi/repo-state.toml`, and honored by the CLI too.

![The repository's AI page: Repository Instructions with the Combine with global dropdown, then a profile and instructions for each action](images/settings-repo-ai.png)

### How the levels combine

The order sent is: built-in prompt, **global**, **workspace**, **repository**,
then the action's own instructions. The mode of a level decides what happens to
the ones above it:

- **Add** (the default): the level's text is added after the more general ones.
- **Replace**: the level's text is used *instead of* every more general level
  (global, and the workspace when the repository replaces). A level with no text
  never replaces anything.

The action's own instructions follow the same idea: the global ones for that
action, then the workspace's for that action (added after them, or replacing
them), and a repository's own override for the action wins over both. The
workspace used is the active workspace of the window when it contains the
repository, otherwise the first workspace that contains it; on the command line
it is the one in `GITPOULPI_WORKSPACE`, otherwise the first.

To see exactly what is sent, level by level, run
`gitpoulpi ai prompt --action <action> --repo .` (`base`, `global`, `workspace`,
`repository`, `task`, and the final `system_prompt`). It calls no model and
works with AI turned off. Set a workspace's instructions from a script with
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (or `--clear`).

### Prompt variables

Every instructions text (global, workspace, repository, per action) may contain
`${name}` variables. They are replaced by their value right before the prompt is
sent (the saved text keeps the `${...}`). A name that is not in the table is sent
as written and the editor warns about it; write `$${` for a literal `${`.

| Variable | Expands to | Example |
|---|---|---|
| `${uilang}` | The UI language, by its English name | `French` |
| `${repo}` | The repository's folder name | `gitpoulpi` |
| `${branch}` | The current branch | `main` |
| `${ref}` / `${commit}` | Short sha of the commit the action is about (*Explain*), else HEAD | `4c0443d` |
| `${commit_subject}` | Subject line of that commit | `feat: add search` |
| `${date}` | Local date, ISO | `2026-10-03` |
| `${time}` | Local time, `HH:MM` | `14:05` |
| `${datetime}` | Local date and time | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | The default remote (`origin`, else the first) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (another host) or `none` | `github` |
| `${files}` | Number of files in the action's diff | `3` |
| `${action}` | The action being run | `commit_message` |

In the instructions editor: type `${` and a popup lists the matching variables
(`Up`/`Down` choose, `Enter` or `Tab` inserts, `Esc` closes it); or use the
**Variables** dropdown at the bottom left (`Ctrl+T`, or *AI: Insert Prompt
Variable* in the command palette), which inserts at the caret. Each entry has a
description, shown in the dropdown and on the line above the buttons while you
hover or highlight it.

## What it can do

| Action | In the window | CLI |
|---|---|---|
| **Commit message** | Stage changes, press the `✦` right of the Summary field, `Ctrl+G i`, or palette `AI: Generate Commit Message`. Summary and description are filled for you to edit; nothing is committed. | `ai commit-message` (prints), `commit --ai` (commits) |
| **Explain changes** | The `✦` at the top right of the working-changes pane (`Ctrl+G w`: the selected commit, else the working changes), or a commit's menu *Explain changes (AI)*. A popup: arrows scroll, `c` copies, `Esc` closes. | `ai explain [rev] [--wip]` |
| **Stash message** | `Ctrl+G m` opens the stash dialog; its AI button (`Ctrl+G` inside the dialog) asks the model. Palette `AI: Generate Stash Message` does both. | `stash push --ai` |
| **Pull request title and description** | *Generate title and description* in the Create Pull Request form (the branch's commits and diff are read locally). | |
| **Merge conflict resolution** | In the merge conflict editor: the `✦ Suggest` toolbar button, a conflict's right-click menu *✦ Suggest resolution (AI)*, `Alt+I`, or palette `AI: Suggest Merge Conflict Resolution`. The suggestion replaces the *current conflict's* region of the Output as one edit (marked edited, one undo step); nothing is saved or staged until you save. | `ai resolve-conflict <path> [--conflict N]` (prints) |
| **Code review** | In the [review of a pull or merge request](review.md#-ai-review-draft): the `✦ Draft review...` button, `s`, or palette `Review: ✦ Draft Review...` (it opens the review options, `Enter` runs). The model reads the request's diff, description and existing comments (and, by default, the project's `CLAUDE.md` and `AGENTS.md`: see [Review context](#review-context)) and proposes comments; you accept, edit or dismiss each one. Nothing reaches the host before you submit the review. | `hosting review suggest <n>` (prints JSON, posts nothing), `ai review-context` (the defaults) |
| **Split a commit** | A commit's menu *Split commit (AI)...*, or palette `AI: Split Selected Commit...`. The split editor opens with the model's grouping filled in; see below. | `ai split-plan <rev>` (prints) |
| **History rewrite script** | In the [history rewrite workbench](history-rewrite.md#write-a-script-with-ai): the `✦ Write with AI...` button of the Script editor, or palette `Rewrite: Write a Script with AI...`. A dialog shows exactly what will be sent; the script lands in the editor, marked, and is never run or applied for you. | `ai rewrite-script "<instruction>" [--sample N]` (prints) |
| **History rewrite rules** | In the [history rewrite workbench](history-rewrite.md#create-rules-with-ai): the `✦ Create Rules with AI...` button, `Alt+I`, or palette `Rewrite: Create Rules with AI...`. A multi-line dialog shows exactly what will be sent; the rules and settings land in the workbench and nothing is previewed or applied for you. | `ai rewrite-rules "<description>" [--context]` (prints) |
| **Worktree name** | In the [Create worktree dialog](worktrees.md#naming-with-ai): the `✦ Name with AI` button or `Alt+A` opens a box where you describe the feature, bug or task; the model names the worktree and its branch after the repository's own branch names, and both land in the dialog's fields for you to edit. Only your description and the local branch names are sent, so it has no "what is sent" dialog. | `ai worktree-name --describe "<task>"` (prints) |
| **Commit composer** | The WIP row's menu *Compose commits (AI)...*, `Ctrl+G C`, palette `AI: Compose Commits From Working Changes...` | |

`Esc` cancels a running request (a local agent's process is killed).

### Merge conflict resolution

The model gets the file's path and language, the conflict's base, current (A) and
incoming (B) blocks with the labels the editor shows, up to 20 lines of
already-merged text before and after it, and your instructions (global,
repository, and the *Conflict Resolution* ones in Settings > AI). It answers with the
resolved text of that one conflict in a code block; an answer with conflict markers
or chatter instead of code is sent back once, then refused. A conflict over
40 KB (`ai.limits.conflict_bytes`, *Settings > AI > Limits*) is held back behind a
warning that gives its size and the limit, with *Send anyway* (this request only,
still cleaned by the privacy checks), *Open setting* and *Cancel*; the limit is
GitPoulpi's own guard, so raise it when your model has the room. The controls are hidden while AI is off, and a suggestion never
saves, stages or resolves anything: review the Output, edit it, and save as usual.

![The commit form after the sparkle button: a summary and description written by the model, ready to edit](images/ai-commit-message.png)

![The Explain working changes popup: an explanation and a list of things to double-check](images/ai-explain.png)

### The commit composer

The model splits the working changes into several commits (every changed file in
exactly one; an invalid plan is sent back once, then refused). While files still
have conflicts nothing is asked: a dialog lists them and opens the conflict
resolution view, since the composer needs a tree without conflicts. You review the
plan before anything is committed:

![The composer review: four proposed commits with their files, and the Reset, Cancel and Create 4 commits buttons](images/composer.png)

| Key | Mouse | Does |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | click a row | move over commits and files |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | the arrows, drag and drop | move a commit; move a file to another commit |
| `Right` / `Left`, `*` | the row's arrow | expand / collapse a commit, all of them (`ui.composer_files` sets the start) |
| `e` / `Enter` | the pencil | edit the message |
| `d` / `Delete` | `drop` / `restore` | leave a commit out (its files stay uncommitted) |
| `n` | `+ own commit` | move a file into a commit of its own |
| `r` | `Reset` | back to the model's plan |
| `Ctrl+S` / `a` | `Create N commits` | create them, in order |
| `Esc` | `Cancel` | leave, nothing done |

Creating stages each commit's files and commits them in turn; the working tree is
never touched. A file staged only in part is committed whole. If a step fails it
stops and says how many commits were made.

### Splitting a commit

The model reads the commit's changes, numbered as the [split editor](log-and-history.md#splitting-a-commit)
numbers them, and proposes several commits with a message each; it can group hunks and
single lines. An answer that leaves a change out, assigns one twice or names something
that does not exist is refused. Either way the split editor opens: with the proposal
filled in, or, when the model failed, with the reason in a note so you can split by hand.
Nothing is split until you press **Split** (`Ctrl+S`); move changes, rename commits or add
one first if the grouping is not right. The profile and instructions are the
*Split a commit* ones in Settings > AI.

### A history rewrite script

You describe what the rewrite must do ("move every commit made at night to the next
morning", "replace my old email by the new one, only before 2024") and the model
writes a [Lua script](history-rewrite.md#lua-scripts) for the Script rule, with the header,
`rewrite_commit(c, ctx)` and the helpers of the sandbox. The daemon compiles the answer; when it
does not compile, the error goes back to the model once. What is left is shown in the
editor: read it, test it, then preview. Details and the dialog are in
[History rewrite](history-rewrite.md#write-a-script-with-ai). The profile and instructions are the
*History rewrite script* ones in Settings > AI.

### History rewrite rules

You describe in your own words how the history must be rewritten ("replace my old email a@b.org with
c@d.org", "remove secrets.txt from all history") and the model proposes a scope, some settings and
a list of [rewrite rules](history-rewrite.md#create-rules-with-ai) in the workbench's own
vocabulary. **It only prepares**: the answer fills the workbench, nothing is previewed or applied,
and an entry that cannot be used is listed with the reason while the valid ones are kept. A Script
rule is never created this way. Optionally the scope, the branch and tag names and the authors of
the newest commits go along (a box, off by default, shown in the "what is sent" pane). The profile
and instructions are the *History rewrite rules* ones in Settings > AI.

## Where the key goes

An API key (OpenAI, Anthropic, OpenRouter, an OpenAI-compatible server) is bound to the address it was stored for: the origin of the profile's base URL, recorded in `trust.toml` in the user-only private data folder. If the base URL later points somewhere else, the request is refused before anything is sent and the key stays where it is. Run `gitpoulpi ai trust <profile> --accept-origin <url>` (or accept it in the app) to move the key to the new address on purpose.

- Plain `http://` is allowed on this machine (Ollama, LM Studio on `localhost`); anywhere else it needs `gitpoulpi ai trust <profile> --allow-insecure-http on`.
- A self-hosted server whose certificate does not chain to a known authority is accepted only for the key fingerprint you pin with `--accept-certificate <sha256>`; the TLS handshake still proves the server holds that key.
- A profile that already had a key before this check existed is recorded silently, from its current base URL, the first time it is used.
- A local agent (`claude -p` and the like) has no key and no HTTP, so there is nothing to bind.


## Review context

A code review draft (`✦`, or `hosting review suggest`) reads the request's diff, title, description and
existing comments. It can also be given the project's own conventions, in a **Project conventions**
block placed before the diff, and a local agent can work inside the repository. The defaults are:

| Setting | Default | Meaning |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: the AI profile's *Working folder* decides. `yes`: a local agent runs in the repository for the review, whatever the profile says. `no`: not there (a profile set to *Repository* uses the dedicated folder instead). HTTP profiles have no working folder, so it changes nothing for them. |
| `include_claude_md` | on | Give the model the repository's `CLAUDE.md`. |
| `include_agents_md` | on | Give the model the repository's `AGENTS.md`. |
| `extra_file` | none | One more file: a path relative to the repository root, an absolute path, or a link. |

They are layered like the instructions, the most specific winning field by field: the global `[ai.review]`
table, then the repository's own values (a field left unset inherits), then the choices of one run. A
repository that is not cloned keeps its own values by `host/project` in `remote-reviews.toml`, next to its
review drafts.

A repository can also turn the global extra file **off**: its `no_extra_file = true` (the **No extra file**
checkbox in *Repo-specific settings > AI*, in the ✦ Review options dialog and the `--no-extra-file` flag of
`ai review-context`) means "no extra file for this repository", whatever `[ai.review]` names. An **empty**
**Extra file** field is not the same thing: it inherits the global file. A file of the repository and the
switch exclude each other (setting one clears the other), and the switch wins if a hand-edited file holds both.
An older `extra_file = ""` of a repository still means none.

Where the files come from:

- `CLAUDE.md`, `AGENTS.md` and a relative extra file are read from the repository's **root** at the
  branch the request targets, never from the working tree (which may be on another branch or have local
  edits). With a clone, from the request's base commit; without one, from the host's API.
- An absolute path is read from this machine.
- A link to a file of a code host (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea and Forgejo `.../src/branch/<branch>/<path>`, or a raw link) is read through the integration
  profile whose host matches, so a private repository works with the token you already gave. A link no
  profile matches, or any other address, is fetched anonymously. A branch name with a slash cannot be
  read from a link: use a path, or a link on a branch without one.
- A missing `CLAUDE.md` or `AGENTS.md` is skipped silently; a missing extra file is reported
  (`context.problems` in the CLI's JSON).
- Each file is cut at **8 KB** and the files together at **16 KB**, with a visible
  `[CLAUDE.md cut: showing ... of ... bytes]` note in what the model reads.

When a local agent runs in the repository, the prompt tells it the checked-out files may not match the
request's head and that the diff is authoritative.

In the review view, `s` opens the **✦ Review options** dialog with these values filled in: the repository's over
the global ones. Change them for this run only, or tick *Save as this repository's defaults* to keep them (see
[Review options](review.md#review-options)). The defaults can also be edited in *Settings > AI* (the *Code review*
block, four rows) and per repository in *Repo-specific settings > AI*, where each row can *use the global value*
and a *Reset to global* button drops the repository's own values; the Settings search finds them (`Include
AGENTS.md`, `Extra file`). Anything the model could not be given (a missing extra file) or that was cut to fit is
shown on the draft card.

From the command line:

```sh
gitpoulpi ai review-context                      # the global defaults and what a review gets
gitpoulpi ai review-context --agents-md off      # save a global default
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # follow the global defaults again
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` needs no model and works with AI turned off. `ai prompt --action code_review` prints the
effective context as `review_context`. The same four settings are `ai.review.*` in
`config.toml` ([reference](config-reference.md#ai-built-in-ai-off-by-default)).

## Input limits

Every input GitPoulpi puts in a request has a size limit, a setting under
`[ai.limits]` (*Settings > AI > Limits*). The limits are GitPoulpi's own guard
(a model's context window differs per model): raise them when your model has the
room, lower them to keep requests small.

| Setting (`ai.limits.`) | Default | What it caps |
|---|---|---|
| `conflict_bytes` | 40000 | The base, current and incoming blocks of one merge conflict |
| `diff_bytes` | 60000 | The diff of a message for a commit, an explanation, a stash name, a pull request description, a commit plan or a split; the whole diff of a code review |
| `review_file_bytes` | 12000 | One file's diff in a code review |
| `review_text_bytes` | 4000 | The description of the reviewed pull request |
| `review_threads_bytes` | 6000 | The review comments that already exist |
| `review_context_file_bytes` | 8192 | One project context file of a review (`CLAUDE.md`, `AGENTS.md`, the extra file) |
| `review_context_bytes` | 16384 | All the context files of a review together |

**Nothing is cut silently.** When an input is over its limit the request waits
and *Over the AI limit* lists what is over: which part (the diff, a file, the
description, a context file), its size, the limit and the setting. A review lists
every part that is over in one dialog. The buttons, each with its key (the keys shown here are the English ones; other languages pick their own letters, printed on each button):

- *Send all* (`A`): this request only, nothing is cut. It still goes through the
  privacy checks (secrets replaced, sensitive files left out) and, when it is on,
  the *What is sent* pane, which comes next.
- *Send truncated* (`T`): what is over its limit is cut, as it always was; each
  line of the dialog says what is left out (the end of the diff, the last
  hunks, the files that do not fit).
- *Open setting* (`O`): cancels the request and opens *Settings > AI > Limits*
  on the first limit listed. Focus starts here: Enter never sends.
- *Cancel* (`C`, `Esc`, the cross or a click outside): sends nothing.

A conflict is the exception: it is never cut. *Send anyway* sends it whole once.

Nothing starts an AI request by itself, so there is no background run that
could ask: every request is one you started. A run that cannot ask (the CLI and
MCP) truncates like before and says so; see below.

**On the command line**, nothing can answer a dialog, so an input over its limit
is truncated as before and the answer says what was cut:
`truncated: [{part, detail?, bytes, limit, setting}]` (present only when something
was cut; `part` is one of `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file`, `context_total`). `--send-all`
(alias `--no-truncate`) sends everything whole instead: on `ai` (all its
commands, including `ai resolve-conflict`, where a conflict over its limit is
otherwise refused), on `commit --ai`, on `stash push --ai` and on
`hosting review suggest`. The MCP tools have the same `send_all` argument and the
same `truncated` field.

## Privacy

- Nothing is sent until you press a button or run a command. No background AI,
  no telemetry, no GitPoulpi account.
- What is sent: the prompt (built-in, then your instructions) and the diff you
  asked about; for a pull request, also the branch's commit subjects. A diff
  over 60 KB (`ai.limits.diff_bytes`) is never cut silently: see
  [Input limits](#input-limits).
- A history rewrite script request sends your instruction and the fixed script contract
  (the API of the script sandbox). Author, date and subject of up to 20 commits go
  along **only** if you tick *Send a sample* in the dialog (off by default; the CLI needs
  `--sample N`). File contents and secret values are never sent.
- It goes only to the profile chosen for that action: that API host, or the
  stdin of your local agent command (which then does whatever that agent does).
- API keys are used by the `gitpoulpi` process itself and never reach the daemon,
  the window's other clients, or an MCP client.
- Generated text is never committed, pushed or applied on its own.

### What is cleaned out of a request

Every request, whichever action sends it and whichever provider receives it (a
hosted API or a local agent), goes through the same cleaning step first:

- **Secrets are redacted** (`[ai.privacy] redact_secrets`, on by default). A token
  with a known prefix, a private key block, a JWT, a password in a URL and a
  `password = value` line whose value looks random are replaced by
  `[REDACTED:<rule>]`. It is not exhaustive: it finds the common shapes (the same
  scanner as `rewrite scan-secrets`).
- **Sensitive files are left out** (`[ai.privacy] exclude_sensitive_files`, on by
  default). The diff of a file matching `[privacy] sensitive_patterns` (`.env`
  and `.env.*` except the `.example`, `.sample` and `.template` files, `*.pem`,
  `*.key`, SSH private keys, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`,
  `*.cer`, `*.der`, `*.kdbx`) is not part of the request. A split piece or a
  composer file list that names such a file is cut the same way, so a plan for it
  may be refused as incomplete: stage or split that file yourself.

Turning either off is a choice, and Settings > AI shows a red line under it for as
long as it is off.

**What is sent, before it is sent.** Before a request leaves, the TUI shows the
**What is sent** dialog: the instructions and the request after cleaning (the same
pane as the history rewrite dialogs, scrollable, `Alt+W` shows or hides it), how
many secrets were replaced and which files were left out (a rule and a masked
preview such as `ghp_****abcd`, never a value), a **Don't show again for
<action>** box, and **Send** / **Cancel** (`Esc`, the close cross and a click
outside cancel; nothing leaves until Send). The box writes
`[ai.sent_preview] <action> = false`; Settings > AI > *Show what is sent
before...* has one switch per action and **Show all again**. The two history
rewrite dialogs already show their request, so they have no switch.

**On the command line**, every `ai ...` answer (and `hosting review suggest`)
carries `privacy: {redactions: N, excluded_files: [...]}`, and `--print-sent`
prints the cleaned system and user text, as `{sent: [{system, user}], privacy,
dry_run: true}`, without calling the model (it needs `ai.enabled = true`, no key
and no network).

**A limit.** A local agent whose working folder is the repository
(`workdir = repository`, or *Run the agent in the repository* for a review) can
read the files itself: the cleaning covers what GitPoulpi sends it, not what it
opens on its own. Use the `dedicated` working folder to keep it out of the
repository.

## Troubleshooting

| Symptom | Fix |
|---|---|
| No `✦` anywhere | AI is off: *Settings > AI > Enable AI*. |
| The `✦` is gray | Nothing is staged (commit message). |
| *Test connection*: command not found | The local agent's **Command** is not on the `PATH` GitPoulpi sees: start GitPoulpi where `claude --version` works, or give the full path. |
| A local agent hangs or shows an interactive screen | In *Headless* run mode, add its headless flag (`-p` for Claude Code) to **Arguments**, or pick its **Agent type** again to restore the headless invocation. The interactive screen is what *Interactive (virtual terminal)* run mode is for. |
| "Use "Open the agent"..." | The agent asked a first-run question (trust the folder, sign in). Use **Open the agent...** on the profile, answer it, close the agent, and try again. |
| A virtual-terminal request times out | The message ends with the last lines of the agent's screen: a permission prompt, a login or an update question shows there. Answer it with **Open the agent...**, or raise the **Timeout**. |
| `failed:` 401 | Wrong key, or a key for another provider: re-enter it. |
| `failed:` 404 | The **Model** does not exist there: clear it to use the default. |
| `failed:` connection error (Ollama, LM Studio) | The server is not running, or the **Base URL** has the wrong port or lacks `/v1`. |
| The key shows `(not set)` after typing it | No credential helper: configure one (`git config --global credential.helper ...`) and enter it again. |
| *Over the AI limit* | An input of the request is over a limit of `[ai.limits]`: choose *Send all*, *Send truncated*, *Open setting* or *Cancel* (see [Input limits](#input-limits)). |
| "Conflict too large to send" | The conflict is over `ai.limits.conflict_bytes` (40000 by default). *Send anyway* sends it once; *Open setting* goes to the limit in Settings > AI > Limits. |
| Wrong language or style | Add instructions; the editor's Quick insert chips make it quick. |
| The composer says the plan is invalid | The model skipped or repeated a file: try again or use a stronger model. |

## Older configuration files

An older `config.toml` with `[[ai.presets]]` and `[ai.defaults]` still loads:

- The default preset of each action becomes its `[ai.actions.<action>]`
  (`system_prompt` becomes `instructions`).
- Every old preset is kept in `[[ai.legacy_presets]]`. Settings > AI shows them
  under *Old presets*, with *Delete archive*.
- The old `[preferences.ai_presets]` of a repository becomes a repository
  override.

The built-in prompt now always comes first. If an old prompt copied a built-in
prompt, remove the copied part and keep only what you want to add.
