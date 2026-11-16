//! The AI profile presets: ONE table, easy to update.
//!
//! Re-researched 2026-10-05 (see the AI harness reference). Binary names and headless flags of the agent
//! CLIs were checked against each tool's own `--help` where it was
//! installed (claude, codex, gemini, opencode, pi, cursor-agent, crush,
//! copilot) and against its documentation otherwise (aider, qwen, goose,
//! amp). Model names move fast: every model field stays free text, these
//! are only the suggestions. HTTP providers speak OpenAI's chat-completions
//! dialect (Anthropic its own); base URLs and model ids are the vendors'
//! documented ones.
//!
//! The local-agent client writes the prompt on **stdin** and reads stdout
//! (`agent.rs`), so every harness entry below is the invocation that reads
//! its prompt from stdin; where the CLI insists on a prompt argument, a
//! short pointer sentence is given and the real prompt still comes on
//! stdin.
//!
//! Binary and model names are data, never translated.

use gitpoulpi_config::{AgentRunMode, AiProfile, AiProviderKind};

/// Why an optional argument is worth turning on, in the words the
/// settings page translates (one sentence per purpose, shared by every
/// harness that has it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgPurpose {
    /// Switch every tool off: the agent only writes text.
    NoTools,
    /// Do not save the conversation to disk / history.
    NoSession,
    /// Print plain text (no JSON, no event stream).
    PlainText,
    /// Cap the run to a single model turn.
    OneTurn,
    /// Spend little reasoning on it: faster and cheaper.
    LowEffort,
    /// No color codes or progress noise on the output.
    Quiet,
    /// Read-only / plan mode: nothing can be edited or run.
    ReadOnly,
    /// Never stop to ask for a permission or a question.
    NoPrompts,
    /// Do not require (or touch) a git repository.
    NoGitRepo,
    /// Wait for the whole answer, no streaming.
    NoStream,
    /// Draw in the normal screen, so the whole answer stays readable.
    InlineScreen,
    /// Never commit or edit files on its own.
    NoEdits,
    /// Do not load the project's own rules, memory or extensions.
    NoProjectContext,
    /// Do not check for or install a newer version.
    NoUpdate,
    /// Turn off analytics and usage statistics.
    NoTelemetry,
    /// No network calls at startup (catalog refreshes, version checks).
    Offline,
}

/// Why an environment variable is worth setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvPurpose {
    /// The provider's API key (prefer `${env:NAME}` over typing it).
    ApiKey,
    /// Where the agent keeps its configuration and credentials: a second
    /// account or a clean, isolated profile.
    ConfigDir,
    /// Which model is used when the arguments name none.
    DefaultModel,
    /// How much the model thinks / reasons.
    Effort,
    /// Upper bound on the thinking tokens.
    ThinkingBudget,
    /// Upper bound on the answer's length.
    MaxOutput,
    /// Turn off analytics and crash reports.
    NoTelemetry,
    /// Turn off every optional network call (updates, telemetry, ...).
    NoTraffic,
    /// Do not check for or install a newer version.
    NoUpdate,
    /// The API endpoint, for a proxy or a gateway.
    BaseUrl,
    /// Where the local model server (Ollama, LM Studio, ...) listens.
    Host,
    /// An HTTP(S) proxy.
    Proxy,
    /// Never emit color codes.
    NoColor,
    /// No network calls at startup (catalog refreshes, version checks).
    Offline,
    /// Permissions, sandbox or approvals: how much the agent may do.
    Permissions,
    /// The agent's own request timeout.
    Timeout,
}

/// An optional addition to a harness's arguments (only ones that make
/// sense for a one-shot prompt).
#[derive(Debug, Clone, Copy)]
pub struct ArgOption {
    /// The arguments it adds, as typed: the label is these joined.
    pub args: &'static [&'static str],
    pub purpose: ArgPurpose,
}

impl ArgOption {
    pub fn label(&self) -> String {
        join(self.args)
    }
}

/// An environment variable a harness documents and GitPoulpi's usage
/// benefits from.
#[derive(Debug, Clone, Copy)]
pub struct EnvVar {
    pub name: &'static str,
    pub purpose: EnvPurpose,
    /// The value that suits GitPoulpi (`1`), empty when it is the user's.
    pub value: &'static str,
}

/// A local agent CLI.
#[derive(Debug, Clone, Copy)]
pub struct Harness {
    /// The executable name (`claude`); also the harness's id, what
    /// `AiProfile::harness` stores.
    pub command: &'static str,
    /// The product's name (`Claude Code`).
    pub name: &'static str,
    /// The headless / non-interactive invocation, prompt on stdin.
    pub headless_args: &'static [&'static str],
    pub options: &'static [ArgOption],
    /// The flags that choose a model, the first being what is written.
    pub model_flags: &'static [&'static str],
    /// Suggested model values.
    pub models: &'static [&'static str],
    /// Documented environment variables worth setting here.
    pub env: &'static [EnvVar],
}

const fn opt(purpose: ArgPurpose, args: &'static [&'static str]) -> ArgOption {
    ArgOption { args, purpose }
}

const fn env(name: &'static str, purpose: EnvPurpose, value: &'static str) -> EnvVar {
    EnvVar {
        name,
        purpose,
        value,
    }
}

const POINTER: &str = "Answer the request given on stdin.";

/// Most used first.
///
/// Every entry was checked on 2026-10-05 against the vendor's own docs
/// and, where installed, `--help` of the real binary (see
/// the AI harness reference, which also lists what could
/// not be confirmed). GitPoulpi's use is one-shot: no tools, no saved
/// session, plain text on stdout, a cheap model with little reasoning, so
/// the options and variables below are the ones that serve that.
pub const HARNESSES: &[Harness] = &[
    // https://code.claude.com/docs/en/{cli-reference,headless,env-vars,model-config}
    Harness {
        command: "claude",
        name: "Claude Code",
        headless_args: &["-p"],
        options: &[
            opt(ArgPurpose::NoTools, &["--tools", ""]),
            opt(ArgPurpose::NoSession, &["--no-session-persistence"]),
            opt(ArgPurpose::LowEffort, &["--effort", "low"]),
            opt(ArgPurpose::PlainText, &["--output-format", "text"]),
            opt(ArgPurpose::NoPrompts, &["--permission-mode", "dontAsk"]),
            opt(ArgPurpose::NoProjectContext, &["--safe-mode"]),
        ],
        model_flags: &["--model"],
        models: &[
            "haiku",
            "sonnet",
            "opus",
            "fable",
            "claude-haiku-4-5",
            "claude-sonnet-5-5",
            "claude-opus-5-5",
        ],
        env: &[
            env(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
                EnvPurpose::NoTraffic,
                "1",
            ),
            env("DISABLE_TELEMETRY", EnvPurpose::NoTelemetry, "1"),
            env("CLAUDE_CODE_EFFORT_LEVEL", EnvPurpose::Effort, "low"),
            env("MAX_THINKING_TOKENS", EnvPurpose::ThinkingBudget, ""),
            env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", EnvPurpose::MaxOutput, ""),
            env("ANTHROPIC_MODEL", EnvPurpose::DefaultModel, "haiku"),
            env("CLAUDE_CONFIG_DIR", EnvPurpose::ConfigDir, ""),
            env(
                "ANTHROPIC_API_KEY",
                EnvPurpose::ApiKey,
                "${env:ANTHROPIC_API_KEY}",
            ),
            env("ANTHROPIC_BASE_URL", EnvPurpose::BaseUrl, ""),
            env("API_TIMEOUT_MS", EnvPurpose::Timeout, ""),
            env("NO_COLOR", EnvPurpose::NoColor, "1"),
        ],
    },
    // https://learn.chatgpt.com/docs/{non-interactive-mode,developer-commands,config-file/config-reference,models}
    Harness {
        command: "codex",
        name: "OpenAI Codex CLI",
        headless_args: &["exec", "-"],
        options: &[
            opt(ArgPurpose::ReadOnly, &["--sandbox", "read-only"]),
            opt(ArgPurpose::NoSession, &["--ephemeral"]),
            opt(ArgPurpose::NoGitRepo, &["--skip-git-repo-check"]),
            opt(
                ArgPurpose::LowEffort,
                &["-c", "model_reasoning_effort=\"low\""],
            ),
            opt(ArgPurpose::Quiet, &["--color", "never"]),
            opt(ArgPurpose::NoPrompts, &["-c", "approval_policy=\"never\""]),
            opt(
                ArgPurpose::NoUpdate,
                &["-c", "check_for_update_on_startup=false"],
            ),
            opt(ArgPurpose::NoTelemetry, &["-c", "analytics.enabled=false"]),
        ],
        model_flags: &["-m", "--model"],
        models: &["gpt-6-luna", "gpt-6.1-sol", "gpt-6-astra"],
        env: &[
            env("CODEX_HOME", EnvPurpose::ConfigDir, ""),
            env("CODEX_API_KEY", EnvPurpose::ApiKey, "${env:CODEX_API_KEY}"),
        ],
    },
    // https://github.com/google-gemini/gemini-cli/blob/main/docs/{cli/cli-reference.md,cli/headless.md,reference/configuration.md}
    Harness {
        command: "gemini",
        name: "Gemini CLI",
        headless_args: &["-p", POINTER],
        options: &[
            opt(ArgPurpose::PlainText, &["--output-format", "text"]),
            opt(ArgPurpose::ReadOnly, &["--approval-mode", "plan"]),
            opt(ArgPurpose::NoPrompts, &["--skip-trust"]),
        ],
        model_flags: &["-m", "--model"],
        models: &[
            "flash",
            "flash-lite",
            "pro",
            "gemini-3-flash-preview",
            "gemini-3.8-flash",
            "gemini-2.5-flash",
            "gemini-2.5-pro",
        ],
        env: &[
            env("GEMINI_MODEL", EnvPurpose::DefaultModel, "flash"),
            env(
                "GEMINI_API_KEY",
                EnvPurpose::ApiKey,
                "${env:GEMINI_API_KEY}",
            ),
            env("GEMINI_CLI_HOME", EnvPurpose::ConfigDir, ""),
            env(
                "GEMINI_CLI_TRUST_WORKSPACE",
                EnvPurpose::Permissions,
                "true",
            ),
            env("GOOGLE_GEMINI_BASE_URL", EnvPurpose::BaseUrl, ""),
            env("NO_COLOR", EnvPurpose::NoColor, "1"),
        ],
    },
    // https://opencode.ai/docs/{cli,config,permissions}; OPENCODE_* flags: packages/core/src/flag
    Harness {
        command: "opencode",
        name: "opencode",
        headless_args: &["run", POINTER],
        options: &[
            opt(ArgPurpose::LowEffort, &["--variant", "minimal"]),
            opt(ArgPurpose::NoProjectContext, &["--pure"]),
            opt(ArgPurpose::PlainText, &["--format", "default"]),
        ],
        model_flags: &["--model", "-m"],
        models: &[
            "anthropic/claude-haiku-4-5",
            "anthropic/claude-sonnet-5-5",
            "openai/gpt-5.4-mini",
            "google/gemini-3.8-flash",
        ],
        env: &[
            // No tools flag exists: deny every permission instead.
            env(
                "OPENCODE_PERMISSION",
                EnvPurpose::Permissions,
                "{\"*\":\"deny\"}",
            ),
            env("OPENCODE_DISABLE_AUTOUPDATE", EnvPurpose::NoUpdate, "1"),
            env("OPENCODE_DISABLE_MODELS_FETCH", EnvPurpose::Offline, "1"),
            env("OPENCODE_CONFIG_DIR", EnvPurpose::ConfigDir, ""),
        ],
    },
    // https://cursor.com/docs/cli/{headless,reference/parameters}; stdin: not documented
    Harness {
        command: "cursor-agent",
        name: "Cursor Agent",
        headless_args: &["-p"],
        options: &[
            opt(ArgPurpose::ReadOnly, &["--mode", "ask"]),
            opt(ArgPurpose::PlainText, &["--output-format", "text"]),
            opt(ArgPurpose::NoPrompts, &["--trust"]),
        ],
        model_flags: &["--model"],
        models: &["auto", "gpt-5", "sonnet-4-thinking"],
        env: &[
            env(
                "CURSOR_API_KEY",
                EnvPurpose::ApiKey,
                "${env:CURSOR_API_KEY}",
            ),
            env("CURSOR_API_ENDPOINT", EnvPurpose::BaseUrl, ""),
        ],
    },
    // docs.github.com copilot-cli-command-reference, `copilot help environment`.
    // The prompt comes on stdin ONLY: with -p, "piped input is ignored".
    Harness {
        command: "copilot",
        name: "GitHub Copilot CLI",
        headless_args: &["--silent"],
        options: &[
            opt(ArgPurpose::LowEffort, &["--reasoning-effort", "low"]),
            opt(ArgPurpose::NoPrompts, &["--no-ask-user"]),
            opt(ArgPurpose::NoUpdate, &["--no-auto-update"]),
            opt(ArgPurpose::Quiet, &["--no-color"]),
            opt(ArgPurpose::NoProjectContext, &["--no-custom-instructions"]),
            opt(ArgPurpose::PlainText, &["--output-format", "text"]),
        ],
        model_flags: &["--model"],
        models: &["auto", "gpt-5.4", "claude-sonnet-5-5"],
        env: &[
            env("COPILOT_MODEL", EnvPurpose::DefaultModel, "auto"),
            env("COPILOT_AUTO_UPDATE", EnvPurpose::NoUpdate, "false"),
            env("COPILOT_HOME", EnvPurpose::ConfigDir, ""),
            env(
                "COPILOT_GITHUB_TOKEN",
                EnvPurpose::ApiKey,
                "${env:GH_TOKEN}",
            ),
            env("NO_COLOR", EnvPurpose::NoColor, "1"),
            env("HTTPS_PROXY", EnvPurpose::Proxy, ""),
        ],
    },
    // https://github.com/earendil-works/pi packages/coding-agent/docs/{cli,environment-variables}.md
    Harness {
        command: "pi",
        name: "Pi",
        headless_args: &["-p"],
        options: &[
            opt(ArgPurpose::NoTools, &["--no-tools"]),
            opt(ArgPurpose::NoSession, &["--no-session"]),
            opt(ArgPurpose::LowEffort, &["--thinking", "low"]),
            opt(ArgPurpose::NoProjectContext, &["--no-context-files"]),
            opt(ArgPurpose::Offline, &["--offline"]),
        ],
        model_flags: &["--model"],
        models: &[
            "anthropic/claude-haiku-4-5",
            "anthropic/claude-sonnet-5-5",
            "openai/gpt-5.4-mini",
            "google/gemini-3.8-flash",
        ],
        env: &[
            env("PI_OFFLINE", EnvPurpose::Offline, "1"),
            env("PI_SKIP_VERSION_CHECK", EnvPurpose::NoUpdate, "1"),
            env("PI_TELEMETRY", EnvPurpose::NoTelemetry, "0"),
            env("PI_CODING_AGENT_DIR", EnvPurpose::ConfigDir, ""),
            env(
                "ANTHROPIC_API_KEY",
                EnvPurpose::ApiKey,
                "${env:ANTHROPIC_API_KEY}",
            ),
            env(
                "OPENAI_API_KEY",
                EnvPurpose::ApiKey,
                "${env:OPENAI_API_KEY}",
            ),
            env("HTTPS_PROXY", EnvPurpose::Proxy, ""),
        ],
    },
    // https://qwenlm.github.io/qwen-code-docs/en/users/{features/headless,configuration/settings}/
    Harness {
        command: "qwen",
        name: "Qwen Code",
        headless_args: &["-p", POINTER],
        options: &[
            opt(ArgPurpose::PlainText, &["--output-format", "text"]),
            opt(ArgPurpose::ReadOnly, &["--approval-mode", "plan"]),
            opt(ArgPurpose::OneTurn, &["--max-session-turns", "1"]),
            opt(ArgPurpose::LowEffort, &["--effort", "low"]),
            opt(ArgPurpose::NoProjectContext, &["--safe-mode"]),
            opt(ArgPurpose::NoTelemetry, &["--disable-telemetry"]),
        ],
        model_flags: &["-m", "--model"],
        models: &[
            "qwen3-coder-flash",
            "qwen-turbo",
            "qwen-plus",
            "qwen3-coder-plus",
        ],
        env: &[
            env("QWEN_CODE_DISABLE_TELEMETRY", EnvPurpose::NoTelemetry, "1"),
            env("QWEN_CODE_MAX_OUTPUT_TOKENS", EnvPurpose::MaxOutput, ""),
            env("QWEN_CODE_API_TIMEOUT_MS", EnvPurpose::Timeout, ""),
            env(
                "OPENAI_API_KEY",
                EnvPurpose::ApiKey,
                "${env:OPENAI_API_KEY}",
            ),
            env("NO_COLOR", EnvPurpose::NoColor, "1"),
        ],
    },
    // https://aider.chat/docs/{config/options,scripting,config/dotenv,more/analytics}.html
    Harness {
        // Reads its message from a file: /dev/stdin works on Linux and
        // macOS only.
        command: "aider",
        name: "Aider",
        headless_args: &["--message-file", "/dev/stdin", "--yes-always", "--no-git"],
        options: &[
            opt(ArgPurpose::NoEdits, &["--dry-run"]),
            opt(ArgPurpose::NoStream, &["--no-stream"]),
            opt(ArgPurpose::Quiet, &["--no-pretty"]),
            opt(ArgPurpose::LowEffort, &["--reasoning-effort", "low"]),
            opt(ArgPurpose::NoUpdate, &["--no-check-update"]),
            opt(ArgPurpose::NoTelemetry, &["--no-analytics"]),
        ],
        model_flags: &["--model"],
        models: &["haiku", "sonnet", "opus", "4o", "gemini", "deepseek"],
        env: &[
            env("AIDER_MODEL", EnvPurpose::DefaultModel, ""),
            env("AIDER_CHECK_UPDATE", EnvPurpose::NoUpdate, "false"),
            env("AIDER_OPENAI_API_BASE", EnvPurpose::BaseUrl, ""),
            env(
                "ANTHROPIC_API_KEY",
                EnvPurpose::ApiKey,
                "${env:ANTHROPIC_API_KEY}",
            ),
            env(
                "OPENAI_API_KEY",
                EnvPurpose::ApiKey,
                "${env:OPENAI_API_KEY}",
            ),
        ],
    },
    // https://goose-docs.ai/docs/guides/{goose-cli-commands,environment-variables}
    Harness {
        command: "goose",
        name: "Goose",
        headless_args: &["run", "-i", "-", "--no-session"],
        options: &[
            opt(ArgPurpose::Quiet, &["--quiet"]),
            opt(ArgPurpose::PlainText, &["--output-format", "text"]),
            opt(ArgPurpose::OneTurn, &["--max-turns", "1"]),
        ],
        model_flags: &["--model"],
        models: &[
            "claude-haiku-4-5",
            "claude-sonnet-5-5",
            "gpt-5.4-mini",
            "gemini-3.8-flash",
        ],
        env: &[
            env("GOOSE_PROVIDER", EnvPurpose::DefaultModel, ""),
            env("GOOSE_MODEL", EnvPurpose::DefaultModel, ""),
            env("GOOSE_TELEMETRY_ENABLED", EnvPurpose::NoTelemetry, "false"),
            env("GOOSE_PATH_ROOT", EnvPurpose::ConfigDir, ""),
            env("OLLAMA_HOST", EnvPurpose::Host, "localhost:11434"),
            env(
                "ANTHROPIC_API_KEY",
                EnvPurpose::ApiKey,
                "${env:ANTHROPIC_API_KEY}",
            ),
            env(
                "OPENAI_API_KEY",
                EnvPurpose::ApiKey,
                "${env:OPENAI_API_KEY}",
            ),
            env("HTTPS_PROXY", EnvPurpose::Proxy, ""),
        ],
    },
    // https://github.com/charmbracelet/crush README.md, `crush run --help`
    Harness {
        command: "crush",
        name: "Crush",
        headless_args: &["run", "--quiet"],
        options: &[opt(ArgPurpose::LowEffort, &["--reasoning-effort", "low"])],
        model_flags: &["--model", "-m"],
        models: &[
            "anthropic/claude-haiku-4-5",
            "anthropic/claude-sonnet-5-5",
            "openai/gpt-5.4-mini",
            "google/gemini-3.8-flash",
        ],
        env: &[
            env("CRUSH_DISABLE_METRICS", EnvPurpose::NoTelemetry, "1"),
            env("DO_NOT_TRACK", EnvPurpose::NoTelemetry, "1"),
            env(
                "CRUSH_DISABLE_PROVIDER_AUTO_UPDATE",
                EnvPurpose::NoUpdate,
                "1",
            ),
            env("CRUSH_GLOBAL_CONFIG", EnvPurpose::ConfigDir, ""),
            env("CRUSH_GLOBAL_DATA", EnvPurpose::ConfigDir, ""),
            env(
                "ANTHROPIC_API_KEY",
                EnvPurpose::ApiKey,
                "${env:ANTHROPIC_API_KEY}",
            ),
            env(
                "OPENAI_API_KEY",
                EnvPurpose::ApiKey,
                "${env:OPENAI_API_KEY}",
            ),
        ],
    },
    // https://ampcode.com/docs/cli/execute-mode: a full agent with an
    // account, threads are always saved, no model or tool flags.
    Harness {
        command: "amp",
        name: "Amp",
        headless_args: &["-x"],
        options: &[opt(ArgPurpose::LowEffort, &["--fast"])],
        model_flags: &[],
        models: &[],
        env: &[env("AMP_API_KEY", EnvPurpose::ApiKey, "${env:AMP_API_KEY}")],
    },
];

/// Where the answer of a terminal-mode run is read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalExtract {
    /// The agent's own session transcript (exact text, no screen scraping).
    ClaudeTranscript,
    /// The screen, between the sentinel markers the prompt asks for.
    Screen,
}

/// How to drive one agent's interactive screen in a virtual terminal.
///
/// The marker strings were found by reading the installed packages (not
/// by running the agents), with the versions in
/// the AI harness reference; they are matched as plain
/// substrings of the screen text.
#[derive(Debug, Clone, Copy)]
pub struct TerminalSpec {
    /// The arguments of a fresh profile in terminal mode.
    pub args: &'static [&'static str],
    /// Optional extras valid without the headless flag.
    pub options: &'static [ArgOption],
    /// Any of these on screen: the prompt box is up, input is accepted.
    pub ready: &'static [&'static str],
    /// Any of these on screen: the agent is working on a turn.
    pub busy: &'static [&'static str],
    /// Any of these on screen: a first-run dialog (folder trust, login,
    /// theme...) the user must answer once, by hand.
    pub blockers: &'static [&'static str],
    /// Typed to leave the agent (then Enter).
    pub exit_input: &'static str,
    pub extract: TerminalExtract,
}

const CLAUDE_TERMINAL: TerminalSpec = TerminalSpec {
    args: &[],
    options: &[
        opt(ArgPurpose::NoTools, &["--tools", ""]),
        opt(ArgPurpose::LowEffort, &["--effort", "low"]),
        opt(ArgPurpose::NoPrompts, &["--permission-mode", "dontAsk"]),
        opt(ArgPurpose::NoProjectContext, &["--safe-mode"]),
    ],
    ready: &["? for shortcuts"],
    busy: &["esc to interrupt"],
    blockers: &[
        "Yes, I trust this folder",
        "Quick safety check",
        "Select login method",
        "Choose the text style",
        "Please run /login",
    ],
    exit_input: "/exit",
    extract: TerminalExtract::ClaudeTranscript,
};

const CODEX_TERMINAL: TerminalSpec = TerminalSpec {
    args: &["--no-alt-screen"],
    options: &[
        opt(ArgPurpose::InlineScreen, &["--no-alt-screen"]),
        opt(ArgPurpose::ReadOnly, &["--sandbox", "read-only"]),
        opt(
            ArgPurpose::LowEffort,
            &["-c", "model_reasoning_effort=\"low\""],
        ),
        opt(ArgPurpose::NoPrompts, &["-c", "approval_policy=\"never\""]),
        opt(
            ArgPurpose::NoUpdate,
            &["-c", "check_for_update_on_startup=false"],
        ),
        opt(ArgPurpose::NoTelemetry, &["-c", "analytics.enabled=false"]),
    ],
    ready: &["Ask Codex to do anything"],
    busy: &["to interrupt)"],
    blockers: &[
        "Trust this folder?",
        "Sign in with ChatGPT",
        "Choose how you want to use Codex",
    ],
    exit_input: "/quit",
    extract: TerminalExtract::Screen,
};

const GEMINI_TERMINAL: TerminalSpec = TerminalSpec {
    args: &[],
    options: &[
        opt(ArgPurpose::ReadOnly, &["--approval-mode", "plan"]),
        opt(ArgPurpose::NoPrompts, &["--skip-trust"]),
    ],
    ready: &["Type your message or @path/to/file"],
    busy: &["esc to cancel"],
    blockers: &[
        "Do you trust the files in this folder?",
        "How would you like to authenticate",
        "Select Theme",
        "Waiting for authentication",
    ],
    exit_input: "/quit",
    extract: TerminalExtract::Screen,
};

/// The harness a command names (`claude`, `/usr/bin/claude`, `claude.cmd`).
pub fn harness_for(command: &str) -> Option<&'static Harness> {
    let base = command.trim().rsplit(['/', '\\']).next().unwrap_or("");
    let stem = base
        .rsplit_once('.')
        .filter(|(_, ext)| matches!(*ext, "exe" | "cmd" | "bat"))
        .map_or(base, |(s, _)| s);
    HARNESSES.iter().find(|h| h.command == stem)
}

/// The harness with id `id` (`claude`): what `AiProfile::harness` holds.
pub fn harness_named(id: &str) -> Option<&'static Harness> {
    HARNESSES.iter().find(|h| h.command == id.trim())
}

/// The harness type of a local-agent profile: the one it stores, else the
/// one its command names. A stored `other` (or any unknown id) means "no
/// known type": the command is then a free-form program with no
/// suggestions.
pub fn harness_of(profile: &AiProfile) -> Option<&'static Harness> {
    if profile.harness.trim().is_empty() {
        harness_for(&profile.command)
    } else {
        harness_named(&profile.harness)
    }
}

/// The `harness` value meaning "none of the known agents".
pub const OTHER_HARNESS: &str = "other";

/// Whether the value of environment variable `name` should be hidden in
/// the UI: a key, token, secret or password, unless it is a `${env:..}`
/// reference (which holds no secret itself).
pub fn env_value_is_secret(name: &str, value: &str) -> bool {
    if value.contains("${env:") {
        return false;
    }
    let n = name.to_ascii_uppercase();
    let by_name = ["_KEY", "_TOKEN", "_SECRET", "_PASSWORD", "_CREDENTIALS"]
        .iter()
        .any(|suffix| n.ends_with(suffix))
        || n.contains("API_KEY")
        || n.contains("AUTH_TOKEN")
        || n.contains("SECRET");
    let by_value = [
        "sk-",
        "ghp_",
        "gho_",
        "github_pat_",
        "xoxb-",
        "AIza",
        "glpat-",
    ]
    .iter()
    .any(|p| value.starts_with(p));
    by_name || by_value
}

fn join(args: &[&str]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() || a.chars().any(char::is_whitespace) {
                format!("\"{a}\"")
            } else {
                (*a).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_string()).collect()
}

impl Harness {
    /// The arguments a fresh profile of this harness gets.
    pub fn default_args(&self) -> Vec<String> {
        owned(self.headless_args)
    }

    /// How to drive this agent's interactive screen, when it can be.
    pub fn terminal(&self) -> Option<&'static TerminalSpec> {
        match self.command {
            "claude" => Some(&CLAUDE_TERMINAL),
            "codex" => Some(&CODEX_TERMINAL),
            "gemini" => Some(&GEMINI_TERMINAL),
            _ => None,
        }
    }

    /// The default arguments of `mode`.
    pub fn default_args_for(&self, mode: AgentRunMode) -> Vec<String> {
        match mode {
            AgentRunMode::Headless => self.default_args(),
            AgentRunMode::Terminal => self.terminal().map(|t| owned(t.args)).unwrap_or_default(),
        }
    }

    /// The optional extras valid in `mode`.
    pub fn options_for(&self, mode: AgentRunMode) -> &'static [ArgOption] {
        match mode {
            AgentRunMode::Headless => self.options,
            AgentRunMode::Terminal => self.terminal().map_or(&[], |t| t.options),
        }
    }

    /// The model named in `args`, if any (`--model x` or `--model=x`).
    pub fn model_in(&self, args: &[String]) -> Option<String> {
        let mut i = 0;
        while i < args.len() {
            for flag in self.model_flags {
                if args[i] == *flag {
                    return args.get(i + 1).cloned();
                }
                if let Some(v) = args[i].strip_prefix(&format!("{flag}=")) {
                    return Some(v.to_string());
                }
            }
            i += 1;
        }
        None
    }

    /// `args` with the model flag set to `model`, or removed for `None`.
    pub fn with_model(&self, args: &[String], model: Option<&str>) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if self.model_flags.iter().any(|f| args[i] == *f) {
                i += 2;
                continue;
            }
            if self
                .model_flags
                .iter()
                .any(|f| args[i].starts_with(&format!("{f}=")))
            {
                i += 1;
                continue;
            }
            out.push(args[i].clone());
            i += 1;
        }
        if let (Some(model), Some(flag)) = (model, self.model_flags.first()) {
            if !model.trim().is_empty() {
                out.push((*flag).to_string());
                out.push(model.trim().to_string());
            }
        }
        out
    }

    /// Whether `option`'s arguments appear, contiguously, in `args`.
    pub fn option_active(option: &ArgOption, args: &[String]) -> bool {
        find_run(args, option.args).is_some()
    }

    /// `args` with `option` added, or removed when it was already there.
    pub fn toggle_option(option: &ArgOption, args: &[String]) -> Vec<String> {
        let mut out = args.to_vec();
        match find_run(args, option.args) {
            Some(at) => {
                out.drain(at..at + option.args.len());
            }
            None => out.extend(owned(option.args)),
        }
        out
    }
}

// -- repository reading: read-only tools and sessions ---------------------------

/// How an agent CLI is run so it can read the repository and nothing else
/// (the "Let the AI read the repository" choice for a local agent).
///
/// Only harnesses whose read-only mode is confirmed have one; for every
/// other the choice stays off, with the reason in its tooltip. Kept apart
/// from [`ArgPurpose`]: these flags are never a settings toggle, GitPoulpi
/// adds them for the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOnlyTools {
    /// Flags (and their values) this mode replaces wherever they stand.
    pub replaces: &'static [&'static str],
    /// Flags that are alone (no value) and replaced too.
    pub replaces_bare: &'static [&'static str],
    /// What the mode adds.
    pub adds: &'static [&'static str],
    /// The tools the agent keeps, for "What is sent".
    pub tools: &'static str,
}

/// Claude Code: the three reading tools, every writing or reaching one
/// refused, and no MCP server (a project's GitHub server could post).
const CLAUDE_READ_ONLY: ReadOnlyTools = ReadOnlyTools {
    replaces: &[
        "--tools",
        "--allowedTools",
        "--disallowedTools",
        "--mcp-config",
    ],
    replaces_bare: &["--strict-mcp-config"],
    adds: &[
        "--tools",
        "Read,Grep,Glob",
        "--allowedTools",
        "Read,Grep,Glob",
        "--disallowedTools",
        "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch",
        "--strict-mcp-config",
        "--mcp-config",
        "{\"mcpServers\":{}}",
    ],
    tools: "Read, Grep, Glob",
};

/// Codex: its read-only sandbox (files readable, nothing writable, no network).
const CODEX_READ_ONLY: ReadOnlyTools = ReadOnlyTools {
    replaces: &["--sandbox", "-s"],
    replaces_bare: &[],
    adds: &["--sandbox", "read-only"],
    tools: "read-only sandbox",
};

/// Whether an agent keeps its session between two runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionUse {
    /// The first run of a session the caller named.
    Start(String),
    /// A later run of that session.
    Resume(String),
}

impl Harness {
    /// The read-only mode of this agent, when it is a confirmed one.
    pub fn read_only_tools(&self) -> Option<ReadOnlyTools> {
        match self.command {
            "claude" => Some(CLAUDE_READ_ONLY),
            "codex" => Some(CODEX_READ_ONLY),
            _ => None,
        }
    }

    /// `args` set up for reading the repository: the flags of
    /// [`Harness::read_only_tools`], replacing any the user had of the same
    /// kind. `None`: this agent has no read-only mode GitPoulpi trusts.
    pub fn with_read_only_tools(&self, args: &[String]) -> Option<Vec<String>> {
        let mode = self.read_only_tools()?;
        let mut out = strip_flags(args, mode.replaces, mode.replaces_bare);
        out.extend(owned(mode.adds));
        Some(out)
    }

    /// Whether this agent can keep a session the caller names.
    pub fn supports_sessions(&self) -> bool {
        self.command == "claude"
    }

    /// `args` for the first or a later run of a named session (Claude:
    /// `--session-id <uuid>`, then `--resume <uuid>`, in place of
    /// `--no-session-persistence`). `None`: this agent's sessions are not
    /// supported here (a run is then one-shot, as before).
    pub fn with_session(&self, args: &[String], session: &SessionUse) -> Option<Vec<String>> {
        if !self.supports_sessions() {
            return None;
        }
        let mut out = strip_flags(
            args,
            &["--session-id", "--resume", "-r"],
            &["--no-session-persistence", "--continue", "-c"],
        );
        match session {
            SessionUse::Start(id) => out.extend([String::from("--session-id"), id.clone()]),
            SessionUse::Resume(id) => out.extend([String::from("--resume"), id.clone()]),
        }
        Some(out)
    }
}

/// `args` without the `valued` flags (and the value after each) and the
/// `bare` ones.
fn strip_flags(args: &[String], valued: &[&str], bare: &[&str]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    let mut skip_value = false;
    for arg in args {
        if skip_value {
            skip_value = false;
            continue;
        }
        if valued.contains(&arg.as_str()) {
            skip_value = true;
            continue;
        }
        if valued
            .iter()
            .any(|f| f.starts_with("--") && arg.starts_with(&format!("{f}=")))
            || bare.contains(&arg.as_str())
        {
            continue;
        }
        out.push(arg.clone());
    }
    out
}

fn find_run(args: &[String], run: &[&str]) -> Option<usize> {
    if run.is_empty() || args.len() < run.len() {
        return None;
    }
    (0..=args.len() - run.len()).find(|&at| {
        args[at..at + run.len()]
            .iter()
            .zip(run)
            .all(|(a, b)| a == b)
    })
}

/// An HTTP provider.
#[derive(Debug, Clone, Copy)]
pub struct ProviderPreset {
    pub name: &'static str,
    /// Which client dialect it speaks.
    pub kind: AiProviderKind,
    pub base_url: &'static str,
    /// Suggested model ids; the first is the default.
    pub models: &'static [&'static str],
    /// Runs on this machine: no key needed.
    pub local: bool,
}

const COMPAT: AiProviderKind = AiProviderKind::OpenAiCompatible;

/// Hosted providers by popularity, then the local servers.
pub const PROVIDERS: &[ProviderPreset] = &[
    ProviderPreset {
        name: "OpenAI",
        kind: AiProviderKind::OpenAi,
        base_url: "https://api.openai.com/v1",
        models: &["gpt-5.4-mini", "gpt-5.4", "gpt-5.5", "gpt-5.4-nano"],
        local: false,
    },
    ProviderPreset {
        name: "Anthropic",
        kind: AiProviderKind::Anthropic,
        base_url: "https://api.anthropic.com/v1",
        models: &["claude-sonnet-5-5", "claude-opus-5-5", "claude-haiku-4-5"],
        local: false,
    },
    ProviderPreset {
        name: "OpenRouter",
        kind: AiProviderKind::OpenRouter,
        base_url: "https://openrouter.ai/api/v1",
        models: &[
            "openai/gpt-5.4-mini",
            "anthropic/claude-sonnet-5-5",
            "google/gemini-3.8-flash",
            "deepseek/deepseek-v4-flash",
        ],
        local: false,
    },
    ProviderPreset {
        name: "Google Gemini",
        kind: COMPAT,
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        models: &["gemini-3.8-flash", "gemini-2.5-pro", "gemini-2.5-flash"],
        local: false,
    },
    ProviderPreset {
        name: "xAI (Grok)",
        kind: COMPAT,
        base_url: "https://api.x.ai/v1",
        models: &["grok-4.7", "grok-4.6", "grok-4.3"],
        local: false,
    },
    ProviderPreset {
        name: "DeepSeek",
        kind: COMPAT,
        base_url: "https://api.deepseek.com/v1",
        models: &["deepseek-v4-flash", "deepseek-v4-pro"],
        local: false,
    },
    ProviderPreset {
        name: "Mistral",
        kind: COMPAT,
        base_url: "https://api.mistral.ai/v1",
        models: &[
            "mistral-small-latest",
            "mistral-medium-latest",
            "mistral-large-latest",
            "codestral-latest",
        ],
        local: false,
    },
    ProviderPreset {
        name: "Groq",
        kind: COMPAT,
        base_url: "https://api.groq.com/openai/v1",
        models: &[
            "openai/gpt-oss-120b",
            "openai/gpt-oss-20b",
            "llama-3.3-70b-versatile",
        ],
        local: false,
    },
    ProviderPreset {
        name: "Together AI",
        kind: COMPAT,
        base_url: "https://api.together.xyz/v1",
        models: &[
            "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            "deepseek-ai/DeepSeek-V3.1",
            "Qwen/Qwen3-235B-A22B-Instruct-2507-tput",
        ],
        local: false,
    },
    ProviderPreset {
        name: "Fireworks AI",
        kind: COMPAT,
        base_url: "https://api.fireworks.ai/inference/v1",
        models: &[
            "accounts/fireworks/models/gpt-oss-120b",
            "accounts/fireworks/models/deepseek-v3p1",
            "accounts/fireworks/models/qwen3-235b-a22b",
        ],
        local: false,
    },
    ProviderPreset {
        name: "Cerebras",
        kind: COMPAT,
        base_url: "https://api.cerebras.ai/v1",
        models: &["gpt-oss-120b", "llama-3.3-70b", "qwen-3-32b"],
        local: false,
    },
    ProviderPreset {
        name: "Perplexity",
        kind: COMPAT,
        base_url: "https://api.perplexity.ai",
        models: &["sonar", "sonar-pro", "sonar-reasoning-pro"],
        local: false,
    },
    ProviderPreset {
        name: "Moonshot (Kimi)",
        kind: COMPAT,
        base_url: "https://api.moonshot.ai/v1",
        models: &["kimi-k2-0905-preview", "kimi-k2-turbo-preview"],
        local: false,
    },
    ProviderPreset {
        name: "Qwen (DashScope)",
        kind: COMPAT,
        base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
        models: &["qwen-plus", "qwen3-max", "qwen-flash", "qwen3-coder-plus"],
        local: false,
    },
    ProviderPreset {
        name: "Zhipu (GLM)",
        kind: COMPAT,
        base_url: "https://api.z.ai/api/paas/v4",
        models: &["glm-4.6", "glm-4.5-air"],
        local: false,
    },
    ProviderPreset {
        name: "Ollama",
        kind: COMPAT,
        base_url: "http://localhost:11434/v1",
        models: &["llama3.2", "qwen3", "gpt-oss:20b", "mistral"],
        local: true,
    },
    ProviderPreset {
        name: "LM Studio",
        kind: COMPAT,
        base_url: "http://localhost:1234/v1",
        models: &["local-model"],
        local: true,
    },
    ProviderPreset {
        name: "llama.cpp server",
        kind: COMPAT,
        base_url: "http://localhost:8080/v1",
        models: &["local-model"],
        local: true,
    },
    ProviderPreset {
        name: "vLLM",
        kind: COMPAT,
        base_url: "http://localhost:8000/v1",
        models: &["local-model"],
        local: true,
    },
    ProviderPreset {
        name: "Jan",
        kind: COMPAT,
        base_url: "http://localhost:1337/v1",
        models: &["local-model"],
        local: true,
    },
    ProviderPreset {
        name: "LocalAI",
        kind: COMPAT,
        base_url: "http://localhost:8081/v1",
        models: &["local-model"],
        local: true,
    },
];

fn norm(url: &str) -> &str {
    url.trim().trim_end_matches('/')
}

/// The preset a profile matches: native kinds by kind (an empty base URL
/// is their default), compatible ones by base URL.
pub fn provider_for(profile: &AiProfile) -> Option<&'static ProviderPreset> {
    if profile.kind == AiProviderKind::LocalAgent {
        return None;
    }
    let url = norm(profile.effective_base_url());
    PROVIDERS
        .iter()
        .find(|p| p.kind == profile.kind && norm(p.base_url) == url)
}

/// The preset named `name`.
pub fn provider_named(name: &str) -> Option<&'static ProviderPreset> {
    PROVIDERS.iter().find(|p| p.name == name)
}

impl ProviderPreset {
    /// Applies the preset to `profile`: kind, base URL (empty when it is
    /// the kind's default), the first model, and the key slot.
    pub fn apply(&self, profile: &mut AiProfile) {
        profile.kind = self.kind;
        profile.base_url = if norm(self.base_url) == self.kind.default_base_url() {
            String::new()
        } else {
            self.base_url.to_string()
        };
        profile.model = self.models.first().copied().unwrap_or("").to_string();
        profile.command.clear();
        profile.args.clear();
        // A local server needs no key; the others keep one slot.
        profile.token_ref = if self.local {
            String::new()
        } else {
            AiProfile::token_ref_for(&profile.id)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        owned(a)
    }

    #[test]
    fn every_harness_has_a_headless_invocation_and_unique_command() {
        let mut seen = std::collections::HashSet::new();
        for h in HARNESSES {
            assert!(seen.insert(h.command), "{} twice", h.command);
            assert!(!h.headless_args.is_empty(), "{}", h.command);
            // Only Amp has no model flag (it picks a mode, not a model).
            assert_eq!(
                h.model_flags.is_empty(),
                h.models.is_empty(),
                "{}",
                h.command
            );
            let mut names = std::collections::HashSet::new();
            for e in h.env {
                assert!(names.insert(e.name), "{} has {} twice", h.command, e.name);
            }
            // The headless invocation is never also an optional extra.
            for o in h.options {
                assert!(
                    !Harness::option_active(o, &h.default_args()),
                    "{}",
                    h.command
                );
            }
        }
        assert_eq!(HARNESSES[0].command, "claude");
        assert_eq!(HARNESSES[0].headless_args, &["-p"]);
        assert_eq!(HARNESSES[1].headless_args, &["exec", "-"]);
    }

    #[test]
    fn the_model_flag_is_written_replaced_and_removed() {
        let claude = harness_for("claude").unwrap();
        let a = claude.with_model(&args(&["-p"]), Some("haiku"));
        assert_eq!(a, args(&["-p", "--model", "haiku"]));
        assert_eq!(claude.model_in(&a).as_deref(), Some("haiku"));
        let b = claude.with_model(&a, Some("opus"));
        assert_eq!(b, args(&["-p", "--model", "opus"]));
        assert_eq!(claude.with_model(&b, None), args(&["-p"]));
        let codex = harness_for("/usr/bin/codex").unwrap();
        let c = codex.with_model(&args(&["exec", "-", "-m", "gpt-5.4"]), Some("gpt-5.5"));
        assert_eq!(c, args(&["exec", "-", "-m", "gpt-5.5"]));
        assert_eq!(
            codex
                .model_in(&args(&["exec", "--model=gpt-5.4"]))
                .as_deref(),
            Some("gpt-5.4")
        );
        assert!(harness_for("claude.cmd").is_some());
        assert!(harness_for("nope").is_none());
    }

    #[test]
    fn options_toggle_on_and_off() {
        let claude = harness_for("claude").unwrap();
        let o = &claude.options[0]; // --tools ""
        let on = Harness::toggle_option(o, &args(&["-p"]));
        assert_eq!(on, args(&["-p", "--tools", ""]));
        assert!(Harness::option_active(o, &on));
        assert_eq!(Harness::toggle_option(o, &on), args(&["-p"]));
        assert_eq!(o.label(), "--tools \"\"");
    }

    #[test]
    fn the_type_is_stored_or_inferred_and_the_command_stays_free() {
        let mut p = AiProfile::new("x", "X", AiProviderKind::LocalAgent);
        // New profiles store their type.
        assert_eq!(harness_of(&p).map(|h| h.command), Some("claude"));
        // An alias keeps the stored type.
        p.command = "/opt/bin/work-claude".into();
        assert_eq!(harness_of(&p).map(|h| h.command), Some("claude"));
        // A profile written before `harness` existed is inferred.
        p.harness.clear();
        p.command = "codex".into();
        assert_eq!(harness_of(&p).map(|h| h.command), Some("codex"));
        p.command = "my-alias".into();
        assert!(harness_of(&p).is_none());
        // An explicit "other" turns the suggestions off even for `claude`.
        p.command = "claude".into();
        p.harness = OTHER_HARNESS.into();
        assert!(harness_of(&p).is_none());
    }

    #[test]
    fn secret_looking_env_values_are_recognized() {
        assert!(env_value_is_secret("ANTHROPIC_API_KEY", "abc"));
        assert!(env_value_is_secret("COPILOT_GITHUB_TOKEN", "abc"));
        assert!(env_value_is_secret("SOMETHING", "sk-ant-123"));
        assert!(!env_value_is_secret(
            "ANTHROPIC_API_KEY",
            "${env:ANTHROPIC_API_KEY}"
        ));
        assert!(!env_value_is_secret(
            "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
            "2000"
        ));
        assert!(!env_value_is_secret("NO_COLOR", "1"));
    }

    #[test]
    fn provider_presets_apply_and_are_recognized_again() {
        let mut p = AiProfile::new("x", "X", AiProviderKind::OpenAi);
        let groq = provider_named("Groq").unwrap();
        groq.apply(&mut p);
        assert_eq!(p.kind, AiProviderKind::OpenAiCompatible);
        assert_eq!(p.base_url, "https://api.groq.com/openai/v1");
        assert_eq!(p.model, "openai/gpt-oss-120b");
        assert_eq!(provider_for(&p).map(|p| p.name), Some("Groq"));
        provider_named("OpenAI").unwrap().apply(&mut p);
        assert_eq!(p.base_url, "");
        assert_eq!(provider_for(&p).map(|p| p.name), Some("OpenAI"));
        let ollama = provider_named("Ollama").unwrap();
        ollama.apply(&mut p);
        assert!(p.token_ref.is_empty());
        let names: std::collections::HashSet<_> = PROVIDERS.iter().map(|p| p.name).collect();
        assert_eq!(names.len(), PROVIDERS.len());
    }

    /// Codex could resume (`codex exec resume <id> -`), but it picks the
    /// session id itself and only prints it among its `--json` events, and
    /// `--last` would pick up the user's own sessions. So GitPoulpi does not
    /// resume it: a Codex run stays one-shot (`--ephemeral`) and the earlier
    /// turns of an exchange are given back in the prompt instead.
    #[test]
    fn codex_stays_one_shot_so_the_history_is_given_back() {
        let codex = harness_for("codex").unwrap();
        assert!(!codex.supports_sessions());
        // The one-shot setting (`--ephemeral`) is an option the user keeps on.
        let run = args(&["exec", "-", "--ephemeral"]);
        assert_eq!(run[..2], args(&["exec", "-"]));
        for session in [
            SessionUse::Start("x".into()),
            SessionUse::Resume("x".into()),
        ] {
            assert_eq!(codex.with_session(&run, &session), None);
        }
        let read_only = codex.with_read_only_tools(&run).unwrap();
        assert!(
            read_only.contains(&"--ephemeral".to_string()),
            "{read_only:?}"
        );
        assert!(!read_only.iter().any(|a| a == "resume" || a == "--last"));
        // Claude is the one that keeps a session.
        assert!(harness_for("claude").unwrap().supports_sessions());
        for other in ["gemini", "opencode", "pi", "cursor-agent", "crush"] {
            if let Some(h) = harness_for(other) {
                assert!(!h.supports_sessions(), "{other}");
            }
        }
    }

    #[test]
    fn terminal_options_never_carry_headless_only_flags() {
        for h in HARNESSES {
            let Some(t) = h.terminal() else { continue };
            assert!(!t.ready.is_empty() && !t.busy.is_empty() && !t.blockers.is_empty());
            assert!(!t.exit_input.is_empty(), "{}", h.command);
            for o in t.options {
                for flag in [
                    "-p",
                    "exec",
                    "--output-format",
                    "--no-session-persistence",
                    "--ephemeral",
                    "--skip-git-repo-check",
                    "--color",
                ] {
                    assert!(!o.args.contains(&flag), "{}: {flag}", h.command);
                }
            }
            // The model flags work on the terminal defaults.
            let defaults = h.default_args_for(AgentRunMode::Terminal);
            let withm = h.with_model(&defaults, Some("m"));
            assert_eq!(h.model_in(&withm).as_deref(), Some("m"));
            // No blocker is also a ready marker (a blocker on the normal
            // screen would fail every run).
            for b in t.blockers {
                assert!(
                    !t.ready.iter().any(|r| r.contains(b) || b.contains(r)),
                    "{b}"
                );
            }
        }
        assert!(harness_for("claude").unwrap().terminal().is_some());
        assert!(harness_for("aider").unwrap().terminal().is_none());
        assert_eq!(
            harness_for("codex")
                .unwrap()
                .default_args_for(AgentRunMode::Terminal),
            args(&["--no-alt-screen"])
        );
    }
}
