use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum RepoAuthAction {
    /// Show which identity this repository authenticates with: the stored
    /// profile, the identity each remote will use, the settings the profile
    /// overrides and any warning. Never prints a secret; a helper or token
    /// command is printed only to a person (an agent gets `{set, sha256}`).
    Show,
    /// Choose the identity this repository authenticates with, per protocol:
    /// `--https` for HTTPS remotes, `--ssh` for SSH remotes. The profile is
    /// kept in GitPoulpi's private directory, never in the repository.
    /// Only a person can set it: an agent is refused (exit code 2).
    Set {
        /// HTTPS identity: `system` (git's own helpers), `integration:<id>`
        /// (a GitPoulpi account's token), `helper` (one credential helper,
        /// with `--helper`) or `token-command` (a command that prints a
        /// token, with `--command` and `--hosts`).
        #[arg(long, value_name = "SOURCE")]
        https: Option<String>,
        /// A user name: with `system`, `credential.username` (lets a
        /// credential manager pick one of several accounts); with
        /// `integration:<id>`, the user name sent with the token.
        #[arg(long)]
        username: Option<String>,
        /// With `--https system`: drop the credential helpers this repository
        /// sets itself and keep the system and global ones.
        #[arg(long)]
        ignore_repo_helpers: bool,
        /// With `--https helper`: the credential helper command. A person
        /// confirms it, with its hash (or passes `--yes`).
        #[arg(long, value_name = "CMD")]
        helper: Option<String>,
        /// With `--https token-command`: the command and its arguments (the
        /// first word is the program). Write `--command` last: every word
        /// after it, dashes included (`sh -c ...`), belongs to the command.
        #[arg(long, num_args = 1.., allow_hyphen_values = true, value_name = "ARG")]
        command: Vec<String>,
        /// With `helper` or `token-command`: the hosts it may serve
        /// (comma-separated or repeated).
        #[arg(long, value_delimiter = ',', value_name = "HOST")]
        hosts: Vec<String>,
        /// With `token-command`: how long its token is reused, in minutes.
        #[arg(long, value_name = "N")]
        cache_minutes: Option<u32>,
        /// SSH identity: `system` (ssh's own configuration and agent),
        /// `key:<path>` (a private key file, offered alone) or
        /// `integration-key:<id>` (the key of a GitPoulpi account).
        #[arg(long, value_name = "SOURCE")]
        ssh: Option<String>,
        /// With `--ssh key:` or `integration-key:`: the `known_hosts` file.
        #[arg(long, value_name = "FILE")]
        known_hosts: Option<String>,
        /// With `--ssh key:` or `integration-key:`: refuse unknown host keys
        /// (`StrictHostKeyChecking=yes`).
        #[arg(long)]
        strict: bool,
        /// Confirm a helper or token command without the interactive question
        /// (the command and its hash are still printed).
        #[arg(long)]
        yes: bool,
    },
    /// Forget the profile: the repository goes back to the machine's own
    /// identity. Only a person can: an agent is refused.
    Clear,
    /// Test the connection of each remote (or one) with the profile applied,
    /// without changing anything: lists the remote's branches, or with
    /// `--push` asks whether a push would be accepted. Answers the report of
    /// each remote plus the identity it used. Reaches the network.
    Test {
        /// Only this remote (default: every remote).
        #[arg(long)]
        remote: Option<String>,
        /// Also check that a push would be accepted.
        #[arg(long)]
        push: bool,
    },
}
