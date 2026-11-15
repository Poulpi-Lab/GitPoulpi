//! Cross references in issue and pull-request text (plan `conversations`, 4c).
//!
//! What a token means depends on the host the text came from: `#7` is an
//! issue on GitHub, a work item on Azure DevOps, a revision-less number on
//! Redmine, nothing on Jira. [`scan`] finds every token the host's grammar
//! gives a meaning to and says where it points. Pure; the TUI paints the
//! spans, the CLI prints them.
//!
//! The repository's own linked sources are matched first, through
//! [`refs::scan_message`], so a Jira key resolves inside a GitHub PR comment
//! and opens in the app. Whatever the host grammar adds on top points at the
//! web page.
//!
//! Never matched: anything in code spans or fenced blocks, inside Markdown
//! link constructs (they already have a target), e-mail addresses, `C#1`,
//! `UTF-8` / `SHA-256` / `ISO-8601`, URL fragments, words that merely look
//! like a hash (`decade`, `1234567`).
//!
//! Offsets are char offsets into the scanned text, `[start, end)`.

use gitpoulpi_config::ApiFamily;

use crate::refs::{self, RefContext, RefKind};
use crate::spec::spec_for;

/// Where the text came from and what the repository links to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XrefContext {
    /// The host (flavor) of the text; `None` = only the repository's links
    /// and plain URLs resolve.
    pub family: Option<ApiFamily>,
    /// `https://host` of that host (instance root; never the API root).
    pub web_base: String,
    /// The project path on that host (`owner/repo`, `group/proj`).
    pub project: String,
    /// The repository's linked ISSUES and PULLS sources.
    pub repo_links: RefContext,
    /// Local notes: names of the Markdown notes (for `[[wikilinks]]`).
    pub local_notes: Vec<String>,
}

/// An item the app can open itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTarget {
    pub family: ApiFamily,
    pub web_base: String,
    pub project: String,
    pub kind: RefKind,
    /// 0 when the item has no number (a note, a Trello card by id).
    pub number: u64,
    /// The canonical key: `#7`, `!4`, `ABC-12`, the note name.
    pub key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKind {
    Label,
    Milestone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XrefTarget {
    /// An item of the repository's linked sources, or a local note.
    InApp(ItemTarget),
    /// A page on the host: open in the browser (after the link confirm).
    Web(String),
    /// `@login`; the profile address when the host has one.
    Mention { login: String, url: Option<String> },
    /// A commit id; whether it exists in the repository is the caller's call.
    Commit { sha: String, url: Option<String> },
    /// A label or milestone shown as a chip, not a link.
    Chip { kind: ChipKind, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xref {
    pub start: usize,
    pub end: usize,
    /// The text as written.
    pub text: String,
    pub target: XrefTarget,
}

const MAX_DIGITS: usize = 9;

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// Every reference in `text`, in order, without overlaps.
pub fn scan(text: &str, ctx: &XrefContext) -> Vec<Xref> {
    let chars: Vec<char> = text.chars().collect();
    let mut s = Scanner::new(&chars, ctx);
    s.run();
    s.finish()
}

struct Scanner<'a> {
    chars: &'a [char],
    /// `chars` with everything no token may match in blanked out.
    masked: Vec<char>,
    ctx: &'a XrefContext,
    taken: Vec<bool>,
    found: Vec<Xref>,
}

impl<'a> Scanner<'a> {
    fn new(chars: &'a [char], ctx: &'a XrefContext) -> Self {
        Self {
            chars,
            masked: chars.to_vec(),
            ctx,
            taken: vec![false; chars.len()],
            found: Vec::new(),
        }
    }

    fn mask(&mut self, a: usize, b: usize) {
        for c in &mut self.masked[a..b.min(self.chars.len())] {
            if *c != '\n' {
                *c = ' ';
            }
        }
    }

    fn add(&mut self, start: usize, end: usize, target: XrefTarget) -> bool {
        if start >= end || self.taken[start..end].iter().any(|t| *t) {
            return false;
        }
        for t in &mut self.taken[start..end] {
            *t = true;
        }
        self.found.push(Xref {
            start,
            end,
            text: self.chars[start..end].iter().collect(),
            target,
        });
        true
    }

    fn finish(mut self) -> Vec<Xref> {
        self.found.sort_by_key(|x| x.start);
        self.found
    }

    fn run(&mut self) {
        for (a, b) in code_ranges(self.chars) {
            self.mask(a, b);
        }
        if self.ctx.family == Some(ApiFamily::LocalMarkdown) || !self.ctx.local_notes.is_empty() {
            self.notes();
        }
        for (a, b) in link_ranges(&self.masked) {
            self.mask(a, b);
        }
        self.urls();
        self.repo_links();
        self.host_tokens();
    }

    // -- local notes -------------------------------------------------------

    fn note_known(&self, name: &str) -> Option<String> {
        if self.ctx.local_notes.is_empty() {
            return Some(name.to_string());
        }
        let want = name.to_lowercase();
        self.ctx
            .local_notes
            .iter()
            .find(|n| {
                let n = n.to_lowercase();
                n == want || n.rsplit('/').next() == Some(want.as_str())
            })
            .cloned()
    }

    fn note_target(&self, name: String) -> XrefTarget {
        XrefTarget::InApp(ItemTarget {
            family: ApiFamily::LocalMarkdown,
            web_base: String::new(),
            project: self.ctx.project.clone(),
            kind: RefKind::Issue,
            number: 0,
            key: name,
        })
    }

    fn notes(&mut self) {
        let c = self.masked.clone();
        let n = c.len();
        let mut i = 0;
        while i < n {
            // [[note]] and [[note|alias]]
            if c[i] == '[' && c.get(i + 1) == Some(&'[') {
                if let Some(close) = find_pair(&c, i + 2, "]]") {
                    let inner: String = c[i + 2..close].iter().collect();
                    if !inner.contains(['[', ']', '\n']) {
                        let name = inner.split('|').next().unwrap_or("");
                        let name = name.split(['#', '^']).next().unwrap_or("").trim();
                        if !name.is_empty() {
                            if let Some(known) = self.note_known(name) {
                                let target = self.note_target(known);
                                if self.add(i, close + 2, target) {
                                    self.mask(i, close + 2);
                                }
                            }
                        }
                        i = close + 2;
                        continue;
                    }
                }
            }
            // [text](relative/path.md#frag)
            if c[i] == '[' && c.get(i + 1) != Some(&'[') {
                if let Some((end, dest)) = md_link_at(&c, i) {
                    let path = dest.split(['#', '?']).next().unwrap_or("");
                    let relative = !path.contains("://")
                        && !path.starts_with(['/', '#'])
                        && !path.contains(':');
                    if relative && path.to_lowercase().ends_with(".md") {
                        let name = path.trim_start_matches("./").trim_end_matches(".md");
                        let name = name.trim_end_matches(".MD");
                        let target = self.note_target(name.to_string());
                        if self.add(i, end, target) {
                            self.mask(i, end);
                        }
                    }
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
    }

    // -- URLs --------------------------------------------------------------

    fn bases(&self) -> Vec<String> {
        let mut v = vec![self.ctx.web_base.clone()];
        if let Some(i) = &self.ctx.repo_links.issues {
            v.push(i.web_base.clone());
        }
        if let Some(p) = &self.ctx.repo_links.pulls {
            v.push(p.web_base.clone());
        }
        v.retain(|b| !b.is_empty());
        v.iter_mut()
            .for_each(|b| *b = b.trim_end_matches('/').to_lowercase());
        v
    }

    fn urls(&mut self) {
        let c = self.masked.clone();
        let n = c.len();
        let bases = self.bases();
        let mut i = 0;
        while i < n {
            if (c[i] == 'h' || c[i] == 'H') && boundary_loose(&c, i) {
                let rest: String = c[i..n.min(i + 8)].iter().collect::<String>().to_lowercase();
                if rest.starts_with("http://") || rest.starts_with("https://") {
                    let mut j = i;
                    while j < n
                        && !c[j].is_whitespace()
                        && !matches!(c[j], '<' | '>' | '"' | '\'' | '`')
                    {
                        j += 1;
                    }
                    let end = i + trim_url_end(&c[i..j]);
                    let url: String = c[i..end].iter().collect();
                    if let Some(target) = self.classify_url(&url, &bases) {
                        self.add(i, end, target);
                    }
                    // Whatever it is, nothing inside a URL is a token.
                    self.mask(i, j);
                    i = j;
                    continue;
                }
            }
            i += 1;
        }
    }

    fn classify_url(&self, url: &str, bases: &[String]) -> Option<XrefTarget> {
        // A Trello card (or any URL the linked sources already know).
        let found = refs::scan_message(url, &self.ctx.repo_links);
        if let [f] = found.as_slice() {
            if f.start == 0 && f.end == url.chars().count() {
                return Some(self.in_app(f));
            }
        }
        let lower = url.to_lowercase();
        if !bases.iter().any(|b| lower.starts_with(b.as_str())) {
            return None;
        }
        let no_frag = url.split(['#', '?']).next().unwrap_or(url);
        let segs: Vec<&str> = no_frag
            .split('/')
            .skip(3)
            .filter(|s| !s.is_empty())
            .collect();
        let (kind, number, key) = item_in_path(&segs)?;
        match kind {
            UrlItem::Commit => Some(XrefTarget::Commit {
                sha: key,
                url: Some(url.to_string()),
            }),
            UrlItem::Issue | UrlItem::Pull => {
                if let (Some(n), UrlItem::Pull, Some(p)) =
                    (number, kind, self.ctx.repo_links.pulls.as_ref())
                {
                    // Compare with the linked source's own address of the PR
                    // (its sigil may also be an issue's, so `refs` can't tell).
                    let own = (spec_for(p.family).pr_web_url)(&p.web_base, &p.project, n);
                    if own.as_deref().is_some_and(|u| same_url(u, url)) {
                        return Some(XrefTarget::InApp(ItemTarget {
                            family: p.family,
                            web_base: p.web_base.clone(),
                            project: p.project.clone(),
                            kind: RefKind::PullRequest,
                            number: n,
                            key: format!("{}{n}", spec_for(p.family).pr_sigil),
                        }));
                    }
                }
                let (sigil, reference) = match (&number, kind) {
                    (Some(n), UrlItem::Issue) => ('#', n.to_string()),
                    (Some(n), UrlItem::Pull) => (self.pull_sigil(), n.to_string()),
                    _ => ('\0', key.clone()),
                };
                let token = if sigil == '\0' {
                    reference
                } else {
                    format!("{sigil}{reference}")
                };
                for f in refs::scan_message(&token, &self.ctx.repo_links) {
                    if f.start == 0
                        && f.end == token.chars().count()
                        && f.url.as_deref().is_some_and(|u| same_url(u, url))
                    {
                        return Some(self.in_app(&f));
                    }
                }
                Some(XrefTarget::Web(url.to_string()))
            }
        }
    }

    fn pull_sigil(&self) -> char {
        self.ctx
            .repo_links
            .pulls
            .as_ref()
            .map(|p| spec_for(p.family).pr_sigil)
            .or_else(|| self.ctx.family.map(|f| spec_for(f).pr_sigil))
            .unwrap_or('#')
    }

    // -- the repository's own links ----------------------------------------

    fn in_app(&self, f: &refs::FoundRef) -> XrefTarget {
        let (family, web_base, project) = match f.kind {
            RefKind::Issue => self
                .ctx
                .repo_links
                .issues
                .as_ref()
                .map(|l| (l.family, l.web_base.clone(), l.project.clone())),
            RefKind::PullRequest => self
                .ctx
                .repo_links
                .pulls
                .as_ref()
                .map(|l| (l.family, l.web_base.clone(), l.project.clone())),
        }
        .unwrap_or((
            self.ctx.family.unwrap_or(ApiFamily::Github),
            self.ctx.web_base.clone(),
            self.ctx.project.clone(),
        ));
        let digits: String = f
            .text
            .chars()
            .rev()
            .take_while(char::is_ascii_digit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        XrefTarget::InApp(ItemTarget {
            family,
            web_base,
            project,
            kind: f.kind,
            number: if f.text.starts_with("http") {
                0
            } else {
                digits.parse().unwrap_or(0)
            },
            key: f.reference.clone(),
        })
    }

    fn repo_links(&mut self) {
        let masked: String = self.masked.iter().collect();
        for f in refs::scan_message(&masked, &self.ctx.repo_links) {
            let target = self.in_app(&f);
            self.add(f.start, f.end, target);
        }
    }

    /// `#N` / `!N` through the linked sources first, else the host's page.
    fn number_target(&self, sigil: char, n: u64) -> Option<XrefTarget> {
        let token = format!("{sigil}{n}");
        for f in refs::scan_message(&token, &self.ctx.repo_links) {
            if f.start == 0 && f.end == token.chars().count() {
                return Some(self.in_app(&f));
            }
        }
        self.web_number(&self.ctx.project, sigil, n)
    }

    fn web_number(&self, project: &str, sigil: char, n: u64) -> Option<XrefTarget> {
        let family = self.ctx.family?;
        let base = self.ctx.web_base.as_str();
        if base.is_empty() || (project.is_empty() && family != ApiFamily::Redmine) {
            return None;
        }
        let spec = spec_for(family);
        let url = if sigil == '!' {
            (spec.pr_web_url)(base, project, n)
        } else {
            (spec.issue_web_url)(base, project, &n.to_string())
        };
        url.map(XrefTarget::Web)
    }

    // -- host grammar --------------------------------------------------------

    fn host_tokens(&mut self) {
        let c = self.masked.clone();
        let n = c.len();
        let mut i = 0;
        while i < n {
            if self.taken[i] {
                i += 1;
                continue;
            }
            let hit = self.token_at(&c, i);
            match hit {
                Some((end, target)) => {
                    if self.add(i, end, target) {
                        i = end;
                    } else {
                        i += 1;
                    }
                }
                None => i += 1,
            }
        }
    }

    fn family(&self) -> Option<ApiFamily> {
        self.ctx.family
    }

    fn token_at(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        let fam = self.family();
        match c[i] {
            '@' => self.mention(c, i),
            '#' | '!' => self.number(c, i),
            '~' | '%' if fam == Some(ApiFamily::Gitlab) => chip(c, i),
            '{' if fam == Some(ApiFamily::Bitbucket) => {
                let end = uuid_braced(c, i)?;
                let login: String = c[i..end].iter().collect();
                Some((end, XrefTarget::Mention { login, url: None }))
            }
            _ => {
                if !boundary_strict(c, i) {
                    return None;
                }
                self.slug_ref(c, i)
                    .or_else(|| self.gh_key(c, i))
                    .or_else(|| self.redmine(c, i))
                    .or_else(|| self.gerrit_change_id(c, i))
                    .or_else(|| self.sha(c, i))
            }
        }
    }

    fn commit_url(&self, project: &str, sha: &str) -> Option<String> {
        let family = self.family()?;
        if self.ctx.web_base.is_empty() || project.is_empty() {
            return None;
        }
        (spec_for(family).commit_url)(&self.ctx.web_base, project, sha)
    }

    fn sha(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        let end = sha_end(c, i)?;
        let sha: String = c[i..end].iter().collect();
        let url = self.commit_url(&self.ctx.project, &sha);
        Some((end, XrefTarget::Commit { sha, url }))
    }

    fn number(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        let sigil = c[i];
        if i > 0 && (is_word(c[i - 1]) || matches!(c[i - 1], '&' | '/' | '\\' | '#' | '!')) {
            return None;
        }
        let (end, n) = digits_at(c, i + 1)?;
        if c.get(end).is_some_and(|x| is_word(*x)) {
            return None;
        }
        let fam = self.family()?;
        let allowed = match sigil {
            '#' => matches!(
                fam,
                ApiFamily::Github
                    | ApiFamily::Gitlab
                    | ApiFamily::Gitea
                    | ApiFamily::Redmine
                    | ApiFamily::AzureDevops
            ),
            _ => matches!(
                fam,
                ApiFamily::Gitlab | ApiFamily::Gitea | ApiFamily::AzureDevops
            ),
        };
        if !allowed {
            return None;
        }
        Some((end, self.number_target(sigil, n)?))
    }

    fn gh_key(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        if self.family() != Some(ApiFamily::Github)
            || c[i] != 'G'
            || c.get(i + 1) != Some(&'H')
            || c.get(i + 2) != Some(&'-')
        {
            return None;
        }
        let (end, n) = digits_at(c, i + 3)?;
        if c.get(end).is_some_and(|x| is_word(*x)) {
            return None;
        }
        Some((end, self.number_target('#', n)?))
    }

    fn redmine(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        if self.family() != Some(ApiFamily::Redmine) {
            return None;
        }
        let word: String = c[i..c.len().min(i + 7)].iter().collect();
        if word == "commit:" {
            let end = sha_end(c, i + 7)?;
            let sha: String = c[i + 7..end].iter().collect();
            return Some((end, XrefTarget::Commit { sha, url: None }));
        }
        if c[i] == 'r' && !self.ctx.web_base.is_empty() {
            let (end, n) = digits_at(c, i + 1)?;
            if c.get(end).is_some_and(|x| is_word(*x)) {
                return None;
            }
            let base = self.ctx.web_base.trim_end_matches('/');
            let url = if self.ctx.project.is_empty() {
                format!("{base}/repository/revisions/{n}")
            } else {
                format!(
                    "{base}/projects/{}/repository/revisions/{n}",
                    self.ctx.project
                )
            };
            return Some((end, XrefTarget::Web(url)));
        }
        None
    }

    fn gerrit_change_id(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        if self.family() != Some(ApiFamily::Gerrit) || c[i] != 'I' || self.ctx.web_base.is_empty() {
            return None;
        }
        let end = i + 41;
        if end > c.len() || !c[i + 1..end].iter().all(|x| is_hex(*x)) {
            return None;
        }
        if c.get(end).is_some_and(|x| is_word(*x)) {
            return None;
        }
        let id: String = c[i..end].iter().collect();
        let base = self.ctx.web_base.trim_end_matches('/');
        Some((end, XrefTarget::Web(format!("{base}/q/{id}"))))
    }

    /// `owner/repo#N`, `group/proj!N`, `proj#N`, `owner/repo@sha`.
    fn slug_ref(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        let fam = self.family()?;
        if !matches!(
            fam,
            ApiFamily::Github | ApiFamily::Gitea | ApiFamily::Gitlab
        ) {
            return None;
        }
        let mut j = i;
        let mut segs = 0;
        loop {
            let start = j;
            while j < c.len() && (c[j].is_ascii_alphanumeric() || matches!(c[j], '_' | '.' | '-')) {
                j += 1;
            }
            if j == start {
                return None;
            }
            segs += 1;
            if c.get(j) == Some(&'/') && c.get(j + 1).is_some_and(|x| x.is_ascii_alphanumeric()) {
                j += 1;
            } else {
                break;
            }
        }
        let slug_end = j;
        let sigil = *c.get(j)?;
        let slug: String = c[i..slug_end].iter().collect();
        if slug.ends_with(['.', '-']) {
            return None;
        }
        let fits = match fam {
            ApiFamily::Gitlab => {
                segs >= 2 || (slug.len() >= 2 && slug.starts_with(|x: char| x.is_ascii_lowercase()))
            }
            _ => segs == 2,
        };
        if !fits {
            return None;
        }
        let project = if segs == 1 {
            // `proj#N`: a sibling project in the same group.
            match self.ctx.project.rsplit_once('/') {
                Some((group, _)) => format!("{group}/{slug}"),
                None => slug.clone(),
            }
        } else {
            slug.clone()
        };
        match sigil {
            '#' | '!' if sigil == '#' || fam == ApiFamily::Gitlab => {
                let (end, n) = digits_at(c, slug_end + 1)?;
                if c.get(end).is_some_and(|x| is_word(*x)) {
                    return None;
                }
                if project == self.ctx.project {
                    return Some((end, self.number_target(sigil, n)?));
                }
                Some((end, self.web_number(&project, sigil, n)?))
            }
            '@' if segs >= 2 => {
                let end = sha_end(c, slug_end + 1)?;
                let sha: String = c[slug_end + 1..end].iter().collect();
                let url = self.commit_url(&project, &sha);
                Some((end, XrefTarget::Commit { sha, url }))
            }
            _ => None,
        }
    }

    fn mention(&self, c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
        let fam = self.family()?;
        if i > 0
            && (is_word(c[i - 1])
                || matches!(
                    c[i - 1],
                    '@' | '.' | '/' | '\\' | '&' | '-' | '_' | '+' | '=' | '%'
                ))
        {
            return None;
        }
        if fam == ApiFamily::AzureDevops {
            if c.get(i + 1) != Some(&'<') {
                return None;
            }
            let end = uuid_plain(c, i + 2)?;
            if c.get(end) != Some(&'>') {
                return None;
            }
            return Some((
                end + 1,
                XrefTarget::Mention {
                    login: "user".to_string(),
                    url: None,
                },
            ));
        }
        if !matches!(
            fam,
            ApiFamily::Github
                | ApiFamily::Gitlab
                | ApiFamily::Gitea
                | ApiFamily::Bitbucket
                | ApiFamily::Jira
                | ApiFamily::Linear
                | ApiFamily::YouTrack
                | ApiFamily::Redmine
                | ApiFamily::Trello
        ) {
            return None;
        }
        if fam == ApiFamily::Bitbucket && c.get(i + 1) == Some(&'{') {
            let end = uuid_braced(c, i + 1)?;
            let login: String = c[i + 1..end].iter().collect();
            return Some((end, XrefTarget::Mention { login, url: None }));
        }
        let mut j = i + 1;
        if !c.get(j).is_some_and(|x| x.is_ascii_alphanumeric()) {
            return None;
        }
        while j < c.len() && (c[j].is_ascii_alphanumeric() || matches!(c[j], '_' | '.' | '-')) {
            j += 1;
        }
        while matches!(c[j - 1], '.' | '-' | '_') {
            j -= 1;
        }
        let mut login: String = c[i + 1..j].iter().collect();
        let base = self.ctx.web_base.trim_end_matches('/');
        let mut url = match fam {
            ApiFamily::Github | ApiFamily::Gitlab | ApiFamily::Gitea | ApiFamily::Trello
                if !base.is_empty() =>
            {
                Some(format!("{base}/{login}"))
            }
            ApiFamily::Bitbucket if !base.is_empty() => Some(format!("{base}/{login}/")),
            _ => None,
        };
        if fam == ApiFamily::Github && c.get(j) == Some(&'/') {
            // `@org/team`
            let mut k = j + 1;
            while k < c.len() && (c[k].is_ascii_alphanumeric() || matches!(c[k], '_' | '-')) {
                k += 1;
            }
            if k > j + 1 {
                let team: String = c[j + 1..k].iter().collect();
                url = (!base.is_empty()).then(|| format!("{base}/orgs/{login}/teams/{team}"));
                login = format!("{login}/{team}");
                j = k;
            }
        }
        if c.get(j).is_some_and(|x| is_word(*x)) {
            return None;
        }
        Some((j, XrefTarget::Mention { login, url }))
    }
}

// ---------------------------------------------------------------------------
// helpers

/// Not glued to a word, an entity or a path.
fn boundary_loose(c: &[char], i: usize) -> bool {
    i == 0 || (!is_word(c[i - 1]) && !matches!(c[i - 1], '&' | '/' | '\\'))
}

/// `boundary_loose` and not part of a dotted, dashed or tagged run.
fn boundary_strict(c: &[char], i: usize) -> bool {
    boundary_loose(c, i)
        && (i == 0 || !matches!(c[i - 1], '.' | '-' | '@' | '#' | '%' | '~' | '+' | '='))
}

fn digits_at(c: &[char], from: usize) -> Option<(usize, u64)> {
    let mut j = from;
    while j < c.len() && c[j].is_ascii_digit() {
        j += 1;
    }
    if j == from || j - from > MAX_DIGITS {
        return None;
    }
    let n = c[from..j].iter().collect::<String>().parse().ok()?;
    Some((j, n))
}

/// End of a 7 to 40 digit commit id at `i`: it needs a digit and a letter
/// (so `decade` and `1234567` are not ids) and may not run into a word.
fn sha_end(c: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    while j < c.len() && is_hex(c[j]) {
        j += 1;
    }
    let len = j - i;
    if !(7..=40).contains(&len) {
        return None;
    }
    let run = &c[i..j];
    if !run.iter().any(char::is_ascii_digit) || !run.iter().any(char::is_ascii_alphabetic) {
        return None;
    }
    match c.get(j) {
        Some(n) if is_word(*n) || *n == '-' => return None,
        Some('.') if c.get(j + 1).is_some_and(|x| x.is_alphanumeric()) => return None,
        _ => {}
    }
    Some(j)
}

fn uuid_plain(c: &[char], i: usize) -> Option<usize> {
    let end = i + 36;
    let s = c.get(i..end)?;
    let ok = s.iter().enumerate().all(|(k, ch)| {
        if matches!(k, 8 | 13 | 18 | 23) {
            *ch == '-'
        } else {
            is_hex(*ch)
        }
    });
    ok.then_some(end)
}

fn uuid_braced(c: &[char], i: usize) -> Option<usize> {
    if c.get(i) != Some(&'{') {
        return None;
    }
    let end = uuid_plain(c, i + 1)?;
    (c.get(end) == Some(&'}')).then_some(end + 1)
}

/// GitLab `~label`, `~"a b"`, `%milestone`.
fn chip(c: &[char], i: usize) -> Option<(usize, XrefTarget)> {
    let sigil = c[i];
    if i > 0 && (is_word(c[i - 1]) || matches!(c[i - 1], '~' | '%' | '/' | '\\')) {
        return None;
    }
    let first = *c.get(i + 1)?;
    let kind = if sigil == '~' {
        if first == '~' {
            return None;
        }
        ChipKind::Label
    } else {
        let numeric = first.is_ascii_digit()
            || (first == 'v' && c.get(i + 2).is_some_and(char::is_ascii_digit));
        if !(numeric || first == '"') {
            return None;
        }
        ChipKind::Milestone
    };
    if first == '"' {
        let close = (i + 2..c.len().min(i + 62)).find(|&k| c[k] == '"' || c[k] == '\n')?;
        if c[close] != '"' || close == i + 2 {
            return None;
        }
        let name: String = c[i + 2..close].iter().collect();
        return Some((close + 1, XrefTarget::Chip { kind, name }));
    }
    if !first.is_ascii_alphanumeric() {
        return None;
    }
    let mut j = i + 1;
    while j < c.len() && (c[j].is_ascii_alphanumeric() || matches!(c[j], '_' | '.' | '-' | ':')) {
        j += 1;
    }
    while matches!(c[j - 1], '.' | '-' | ':') {
        j -= 1;
    }
    if j - (i + 1) < 2 && kind == ChipKind::Label {
        return None;
    }
    let name: String = c[i + 1..j].iter().collect();
    Some((j, XrefTarget::Chip { kind, name }))
}

// -- URLs ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum UrlItem {
    Issue,
    Pull,
    Commit,
}

/// What an address path names: `(kind, number, key)`; the key is the
/// `KEY-N` of a tracker or the commit id.
fn item_in_path(segs: &[&str]) -> Option<(UrlItem, Option<u64>, String)> {
    let num = |s: Option<&&str>| s.and_then(|s| s.parse::<u64>().ok());
    for (k, seg) in segs.iter().enumerate() {
        let next = segs.get(k + 1);
        match *seg {
            "issues" if num(next).is_some() => {
                return Some((UrlItem::Issue, num(next), String::new()))
            }
            "pull" | "pulls" | "merge_requests" | "pull-requests" | "pullrequest"
                if num(next).is_some() =>
            {
                return Some((UrlItem::Pull, num(next), String::new()));
            }
            "edit" if k > 0 && segs[k - 1] == "_workitems" && num(next).is_some() => {
                return Some((UrlItem::Issue, num(next), String::new()));
            }
            "+" if num(next).is_some() => return Some((UrlItem::Pull, num(next), String::new())),
            "browse" | "issue" => {
                if let Some(key) = next.filter(|n| is_issue_key(n)) {
                    return Some((UrlItem::Issue, None, (*key).to_string()));
                }
            }
            "commit" | "commits" => {
                if let Some(sha) = next.filter(|n| {
                    n.len() >= 7 && n.len() <= 40 && n.chars().all(|c| c.is_ascii_hexdigit())
                }) {
                    return Some((UrlItem::Commit, None, (*sha).to_string()));
                }
            }
            _ => {}
        }
    }
    None
}

fn is_issue_key(s: &str) -> bool {
    let Some((key, n)) = s.rsplit_once('-') else {
        return false;
    };
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && !n.is_empty()
        && n.chars().all(|c| c.is_ascii_digit())
}

fn same_url(a: &str, b: &str) -> bool {
    let norm = |u: &str| {
        u.split(['#', '?'])
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_lowercase()
    };
    norm(a) == norm(b)
}

/// Length of the URL in `run` once sentence punctuation is cut off.
fn trim_url_end(run: &[char]) -> usize {
    let mut end = run.len();
    while end > 0 {
        let last = run[end - 1];
        let drop = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '*' => true,
            ')' | ']' | '}' => {
                let (open, close) = match last {
                    ')' => ('(', ')'),
                    ']' => ('[', ']'),
                    _ => ('{', '}'),
                };
                let o = run[..end].iter().filter(|c| **c == open).count();
                let cl = run[..end].iter().filter(|c| **c == close).count();
                cl > o
            }
            _ => false,
        };
        if drop {
            end -= 1;
        } else {
            break;
        }
    }
    end
}

// -- Markdown structure ----------------------------------------------------------

/// Ranges of fenced blocks and inline code spans.
fn code_ranges(c: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut fence: Option<(char, usize, usize)> = None; // (char, len, start)
    let mut line_start = 0;
    while line_start < c.len() {
        let mut line_end = line_start;
        while line_end < c.len() && c[line_end] != '\n' {
            line_end += 1;
        }
        let next_start = (line_end + 1).min(c.len());
        let line = &c[line_start..line_end];
        let indent = line.iter().take_while(|x| **x == ' ').count();
        let body = &line[indent.min(line.len())..];
        let run = |ch: char| body.iter().take_while(|x| **x == ch).count();
        if let Some((fc, flen, start)) = fence {
            if indent <= 3 && run(fc) >= flen && body[run(fc)..].iter().all(|x| x.is_whitespace()) {
                out.push((start, next_start));
                fence = None;
            }
        } else if indent <= 3 && (run('`') >= 3 || run('~') >= 3) {
            let ch = if run('`') >= 3 { '`' } else { '~' };
            // A backtick fence's info string may not hold backticks.
            if ch == '~' || !body[run('`')..].contains(&'`') {
                fence = Some((ch, run(ch), line_start));
            }
        } else {
            inline_code(c, line_start, line_end, &mut out);
        }
        line_start = next_start;
        if next_start == line_end {
            break;
        }
    }
    if let Some((_, _, start)) = fence {
        out.push((start, c.len()));
    }
    out
}

fn inline_code(c: &[char], from: usize, to: usize, out: &mut Vec<(usize, usize)>) {
    let mut i = from;
    while i < to {
        if c[i] != '`' {
            i += 1;
            continue;
        }
        let mut n = 0;
        while i + n < to && c[i + n] == '`' {
            n += 1;
        }
        let mut j = i + n;
        let mut closed = None;
        while j < to {
            if c[j] == '`' {
                let mut m = 0;
                while j + m < to && c[j + m] == '`' {
                    m += 1;
                }
                if m == n {
                    closed = Some(j + m);
                    break;
                }
                j += m;
            } else {
                j += 1;
            }
        }
        match closed {
            Some(end) => {
                out.push((i, end));
                i = end;
            }
            None => i += n,
        }
    }
}

/// The index of `pat` at or after `from`, on the same line.
fn find_pair(c: &[char], from: usize, pat: &str) -> Option<usize> {
    let p: Vec<char> = pat.chars().collect();
    let mut i = from;
    while i + p.len() <= c.len() {
        if c[i] == '\n' {
            return None;
        }
        if c[i..i + p.len()] == p[..] {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// A `[text](dest)` starting at `[` at `i`: `(end, dest)`.
fn md_link_at(c: &[char], i: usize) -> Option<(usize, String)> {
    let mut depth = 0;
    let mut j = i;
    while j < c.len() {
        match c[j] {
            '\n' => return None,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        j += 1;
    }
    if j >= c.len() || c.get(j + 1) != Some(&'(') {
        return None;
    }
    let mut k = j + 2;
    let mut paren = 1;
    while k < c.len() {
        match c[k] {
            '\n' => return None,
            '(' => paren += 1,
            ')' => {
                paren -= 1;
                if paren == 0 {
                    let inner: String = c[j + 2..k].iter().collect();
                    let dest = inner
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(['<', '>']);
                    return Some((k + 1, dest.to_string()));
                }
            }
            _ => {}
        }
        k += 1;
    }
    None
}

/// Ranges of `[text](dest)` and `![alt](src)`.
fn link_ranges(c: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] == '[' {
            if let Some((end, _)) = md_link_at(c, i) {
                let start = if i > 0 && c[i - 1] == '!' { i - 1 } else { i };
                out.push((start, end));
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests;
