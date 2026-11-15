use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// -- the mock GitLab -----------------------------------------------------------

pub use xtask::mock_host::http::Request as Seen;

pub struct Mock {
    pub port: u16,
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

impl Mock {
    pub fn requests(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }

    /// Whether some request matched.
    pub fn saw(&self, method: &str, needle: &str) -> bool {
        self.requests()
            .iter()
            .any(|r| r.method == method && r.target.contains(needle))
    }
}

pub const MR_ROWS: &str = r#"[
  {"iid":53,"title":"Add the parser","state":"opened","source_branch":"feature","target_branch":"main",
   "source_project_id":1,"target_project_id":1,"author":{"username":"kai"},
   "web_url":"http://127.0.0.1/g/p/-/merge_requests/53","updated_at":"2026-09-29T10:00:00Z",
   "assignees":[],"reviewers":[],"has_conflicts":true,"head_pipeline":{"status":"failed"}},
  {"iid":54,"title":"Docs for the parser","state":"opened","source_branch":"docs","target_branch":"main",
   "source_project_id":1,"target_project_id":1,"author":{"username":"zed"},
   "web_url":"http://127.0.0.1/g/p/-/merge_requests/54","updated_at":"2026-09-28T10:00:00Z",
   "assignees":[{"username":"kai"}],"reviewers":[],"has_conflicts":false,"head_pipeline":{"status":"success"}}
]"#;

/// A request and an issue of someone else that mention kai: what the to-do
/// list (`action=mentioned`) answers for the Radar's "Mentioning me".
pub const MENTION_MR: &str = r#"{"iid":55,"title":"Please weigh in on the parser","state":"opened",
   "source_branch":"weigh","target_branch":"main","source_project_id":1,"target_project_id":1,
   "author":{"username":"zed"},"web_url":"http://127.0.0.1/g/p/-/merge_requests/55",
   "updated_at":"2026-09-27T10:00:00Z","assignees":[],"reviewers":[]}"#;

pub const MENTION_ISSUE: &str = r#"{"iid":14,"title":"Parser question for kai","state":"opened",
   "web_url":"http://127.0.0.1/g/p/-/issues/14","updated_at":"2026-09-27T10:00:00Z",
   "author":{"username":"zed"},"assignees":[],"labels":[]}"#;

/// The to-do answer for a `type=` of `MergeRequest` or `Issue`, for both
/// fixture projects.
pub fn todos(target: &str) -> String {
    let (kind, item) = if target.contains("type=MergeRequest") {
        ("MergeRequest", MENTION_MR)
    } else if target.contains("type=Issue") {
        ("Issue", MENTION_ISSUE)
    } else {
        return "[]".into();
    };
    let one = |project: &str| {
        format!(
            r#"{{"action":"mentioned","target_type":"{kind}","project":{{"path_with_namespace":"{project}"}},"target":{item}}}"#
        )
    };
    format!("[{},{}]", one("g/p"), one("team/tool"))
}

pub const ISSUE_ROWS: &str = r#"[
  {"iid":12,"title":"Crash on startup","state":"opened","web_url":"http://127.0.0.1/g/p/-/issues/12",
   "updated_at":"2026-09-29T10:00:00Z","author":{"username":"kai"},"assignees":[],"labels":[]}
]"#;

pub const JIRA_SEARCH: &str = r#"{"total":1,"issues":[
  {"id":"10001","key":"ABC-12","fields":{"summary":"Crash on startup",
   "status":{"name":"To Do","statusCategory":{"key":"new"}},"issuetype":{"name":"Bug"},
   "assignee":null,"reporter":{"name":"kai","displayName":"Kai"},"labels":[],
   "updated":"2026-09-29T10:00:00.000+0000"}}]}"#;

pub const JIRA_PROJECTS: &str = r#"[{"key":"ABC","name":"Alphabet"}]"#;

/// Extra generated projects (`many/repoNNN`) a mock lists after the two
/// fixed ones, by mock port: enough to overflow the dropdown and to need
/// a second API page.
pub static EXTRA_PROJECTS: Mutex<Vec<(u16, usize)>> = Mutex::new(Vec::new());

/// A delay in milliseconds a mock adds before answering the projects
/// listing, by mock port: a slow host.
/// Mock ports whose branch `feature` request (!53) answers as merged.
pub static MERGED_HEAD_PORTS: Mutex<Vec<u16>> = Mutex::new(Vec::new());
pub static SLOW_PROJECTS_MS: Mutex<Vec<(u16, u64)>> = Mutex::new(Vec::new());

/// A status every POST to a mock answers instead of success, by mock port:
/// a host that refuses the token (401) or breaks (500) on create.
pub static POST_FAILS: Mutex<Vec<(u16, u16)>> = Mutex::new(Vec::new());

pub fn project_json(port: u16, id: usize, path: &str, visibility: &str) -> String {
    let name = path.rsplit('/').next().unwrap();
    format!(
        r#"{{"id":{id},"name":"{name}","path_with_namespace":"{path}","description":"d",
    "default_branch":"main","ssh_url_to_repo":"git@127.0.0.1:{path}.git",
    "http_url_to_repo":"http://127.0.0.1:{port}/{path}.git","web_url":"http://127.0.0.1:{port}/{path}",
    "visibility":"{visibility}"}}"#
    )
}

/// The projects listing, `per_page=100` pages of it.
pub fn projects_json(port: u16, target: &str) -> String {
    let extra = EXTRA_PROJECTS
        .lock()
        .unwrap()
        .iter()
        .find(|(p, _)| *p == port)
        .map_or(0, |(_, n)| *n);
    let mut all = vec![
        project_json(port, 1, "g/p", "private"),
        project_json(port, 2, "team/tool", "public"),
    ];
    all.extend(
        (0..extra).map(|i| project_json(port, 10 + i, &format!("many/repo{i:03}"), "public")),
    );
    let page: usize = target
        .split(['?', '&'])
        .find_map(|kv| kv.strip_prefix("page=")?.parse().ok())
        .unwrap_or(1);
    let slice: Vec<String> = all.into_iter().skip((page - 1) * 100).take(100).collect();
    format!("[{}]", slice.join(","))
}

/// The routes of the conversation: `#12` (a description with a bold word, a
/// reference to `#7`, a web address; one comment and one label event) and `#7`.
fn conversation_route(method: &str, path: &str) -> Option<(u16, String)> {
    if method != "GET" {
        return None;
    }
    let body = match path {
        "/api/v4/projects/g%2Fp/issues/12" => {
            r#"{"id":112,"iid":12,"title":"Crash on startup",
            "state":"opened","description":"Opening a repo **freezes** when files are locked.\n\nSame as #7, see https://example.org/docs and more.\n\n- [x] reproduced on Linux",
            "author":{"username":"ann","name":"Ann Archer"},"created_at":"2026-11-12T09:14:00Z",
            "updated_at":"2026-11-12T09:14:00Z","web_url":"http://127.0.0.1/g/p/-/issues/12","labels":["bug","lfs"]}"#
        }
        "/api/v4/projects/g%2Fp/issues/12/discussions" => {
            r#"[
            {"id":"dn1","individual_note":true,"notes":[{"id":201,"body":"Seen on Linux too, really.",
              "author":{"username":"bo","name":"Bo Brandt"},"created_at":"2026-11-12T10:30:00Z",
              "updated_at":"2026-11-12T10:30:00Z","system":false}]},
            {"id":"dn2","individual_note":true,"notes":[{"id":202,"body":"added ~lfs label",
              "author":{"username":"bo","name":"Bo Brandt"},"created_at":"2026-11-12T10:31:00Z","system":true}]}]"#
        }
        "/api/v4/projects/g%2Fp/issues/7" => {
            r#"{"id":107,"iid":7,"title":"Locked files",
            "state":"opened","description":"Locks are not released.",
            "author":{"username":"kai","name":"Kai"},"created_at":"2026-11-01T09:00:00Z",
            "web_url":"http://127.0.0.1/g/p/-/issues/7","labels":[]}"#
        }
        "/api/v4/projects/g%2Fp/issues/7/discussions" => "[]",
        _ => return None,
    };
    Some((200, body.to_string()))
}

/// The writes to issue `#12`'s conversation: a note, an answer in a
/// discussion, an edit and a delete, as a GitLab takes them. The answer
/// carries the posted text and `kai` as the author, like the real host.
fn conversation_write(request: &Seen) -> Option<(u16, String)> {
    let path = request.target.split('?').next()?;
    let rest = path.strip_prefix("/api/v4/projects/g%2Fp/issues/12/")?;
    let text = serde_json::from_str::<serde_json::Value>(&request.body)
        .ok()
        .and_then(|v| v["body"].as_str().map(str::to_string))
        .unwrap_or_default();
    let note = |id: u64| {
        serde_json::json!({"id": id, "body": text, "system": false,
            "created_at": "2026-11-15T10:00:00Z", "updated_at": "2026-11-15T10:00:00Z",
            "author": {"username": "kai", "name": "Kai"}})
        .to_string()
    };
    match (request.method.as_str(), rest) {
        ("POST", "notes") => Some((201, note(301))),
        ("POST", e) if e.starts_with("discussions/") && e.ends_with("/notes") => {
            Some((201, note(302)))
        }
        ("PUT", e) if e.starts_with("notes/") => {
            Some((200, note(e.trim_start_matches("notes/").parse().ok()?)))
        }
        ("DELETE", e) if e.starts_with("notes/") => Some((204, String::new())),
        _ => None,
    }
}

pub fn route(method: &str, target: &str, port: u16) -> (u16, String) {
    let path = target.split('?').next().unwrap_or(target);
    if let Some(answer) = providers_route(method, path) {
        return answer;
    }
    if let Some(answer) = conversation_route(method, path) {
        return answer;
    }
    // An OpenAI-compatible model on the same loopback server (the AI tests'
    // HTTP route): it answers one canned draft and, like any model, changes
    // nothing on the host.
    if method == "POST" && path == "/v1/chat/completions" {
        return (
            200,
            r#"{"choices":[{"message":{"content":"Please attach the stack trace of the crash."}}]}"#
                .into(),
        );
    }
    match (method, path) {
        ("GET", "/api/v4/user") => (
            200,
            r#"{"id":9,"username":"kai","name":"Kai","web_url":"http://127.0.0.1/kai"}"#.into(),
        ),
        ("GET", "/api/v4/todos") => (200, todos(target)),
        ("GET", p) if p.ends_with("/merge_requests") => {
            let merged = MERGED_HEAD_PORTS.lock().unwrap().contains(&port);
            let rows = if merged {
                // The branch lookup (`source_branch=`) answers only that
                // branch's request; the section's list answers both.
                let mut rows: serde_json::Value = serde_json::from_str(MR_ROWS).unwrap();
                rows[0]["state"] = "merged".into();
                if target.contains("source_branch=") {
                    rows = serde_json::Value::Array(vec![rows[0].clone()]);
                }
                rows.to_string()
            } else {
                MR_ROWS.into()
            };
            (200, rows)
        }
        ("POST", p) if p.ends_with("/merge_requests") => (
            201,
            r#"{"iid":7,"web_url":"http://127.0.0.1/g/p/-/merge_requests/7"}"#.into(),
        ),
        ("GET", p) if p.ends_with("/issues") => (200, ISSUE_ROWS.into()),
        // Slice M: what the metadata pickers offer.
        ("GET", p) if p.ends_with("/members/all") => (
            200,
            r#"[{"id":9,"username":"kai","name":"Kai"},{"id":12,"username":"ann","name":"Ann Archer"}]"#
                .into(),
        ),
        ("GET", p) if p.ends_with("/labels") => (
            200,
            r##"[{"name":"bug","color":"#d9534f","description":"Broken"},{"name":"ui","color":"#428bca"}]"##
                .into(),
        ),
        ("GET", p) if p.ends_with("/milestones") => (
            200,
            r#"[{"id":7,"iid":1,"title":"v1.0","due_date":"2026-12-01","state":"active"}]"#.into(),
        ),
        ("POST", p) if p.ends_with("/issues") => (
            201,
            r#"{"iid":13,"web_url":"http://127.0.0.1/g/p/-/issues/13"}"#.into(),
        ),
        ("GET", "/api/v4/projects") => (200, projects_json(port, target)),
        // The Jira Data Center side (`jira_data_center`, REST v2).
        ("GET", "/rest/api/2/search") => (200, JIRA_SEARCH.into()),
        ("GET", "/rest/api/2/project") => (200, JIRA_PROJECTS.into()),
        // Jira Cloud (REST v3): the Integrations page's token check.
        ("GET", "/rest/api/3/myself") => (
            200,
            r#"{"accountId":"5b10","displayName":"Kai","emailAddress":"me@x"}"#.into(),
        ),
        // Gitea / Forgejo (`/api/v1`): the Integrations page's token check.
        ("GET", "/api/v1/user") => (
            200,
            r#"{"id":1,"login":"kai","full_name":"Kai"}"#.into(),
        ),
        ("GET", "/rest/api/2/myself") => (
            200,
            r#"{"name":"kai","key":"kai","displayName":"Kai"}"#.into(),
        ),
        _ => (404, r#"{"message":"404 Not Found"}"#.into()),
    }
}

// -- Azure DevOps Server, Jira Data Center issue types, Trello -------------------

/// The Azure pull request (`P/R`, id 12) of the `feature` branch.
pub const AZURE_PULLS: &str = r#"{"value":[
  {"pullRequestId":12,"title":"Add the feature","status":"active","isDraft":false,
   "sourceRefName":"refs/heads/feature","targetRefName":"refs/heads/main",
   "createdBy":{"id":"u1","displayName":"kai","uniqueName":"kai"},
   "creationDate":"2026-09-29T10:00:00Z","reviewers":[],
   "repository":{"id":"rid","name":"R","project":{"name":"P"}}}]}"#;

pub const AZURE_REPOS: &str = r#"{"value":[
  {"id":"rid","name":"R","project":{"name":"P"},"remoteUrl":"http://127.0.0.1/DefaultCollection/P/_git/R",
   "webUrl":"http://127.0.0.1/DefaultCollection/P/_git/R","defaultBranch":"refs/heads/main","isDisabled":false}]}"#;

pub const JIRA_TYPES: &str =
    r#"{"values":[{"id":"10002","name":"Task"},{"id":"10004","name":"Bug"}],"total":2}"#;

pub const TRELLO_BOARDS: &str = r#"[{"id":"b1","name":"Roadmap","shortLink":"AbCd1234",
  "url":"http://127.0.0.1/b/AbCd1234/roadmap","desc":"","prefs":{"permissionLevel":"private"}}]"#;

pub const TRELLO_LISTS: &str = r#"[{"id":"l1","name":"Todo"},{"id":"l2","name":"Doing"}]"#;

pub const TRELLO_CARDS: &str = r#"[
  {"id":"c1","name":"Write the docs","idShort":7,"shortUrl":"http://127.0.0.1/c/xyz7","url":"http://127.0.0.1/c/xyz7/write",
   "closed":false,"idList":"l1","idMembers":[],"members":[],"labels":[],"dateLastActivity":"2026-09-29T10:00:00.000Z"}]"#;

pub const LINEAR_ISSUES: &str = r#"{"data":{"issues":{"nodes":[
  {"id":"u1","identifier":"ENG-12","number":12,"title":"Crash on startup","url":"http://127.0.0.1/acme/issue/ENG-12/crash",
   "branchName":"kai/eng-12-crash-on-startup","updatedAt":"2026-09-29T10:00:00.000Z","priorityLabel":"High",
   "state":{"name":"In Progress","type":"started"},"creator":{"name":"Kai","displayName":"kai"},
   "assignee":null,"labels":{"nodes":[]}}],
  "pageInfo":{"hasNextPage":false,"endCursor":null}}}}"#;

/// The one GraphQL endpoint of the Linear side, told apart by the document.
pub fn linear_route(body: &str) -> (u16, String) {
    let answer = if body.contains("query Viewer") {
        r#"{"data":{"viewer":{"id":"v1","name":"Kai","displayName":"kai","email":"s@x"}}}"#
    } else if body.contains("query Teams") {
        r#"{"data":{"organization":{"urlKey":"acme"},"teams":{"nodes":[{"id":"t1","key":"ENG","name":"Engineering"}],"pageInfo":{"hasNextPage":false}}}}"#
    } else if body.contains("query Issues") {
        LINEAR_ISSUES
    } else {
        return (404, r#"{"errors":[{"message":"unknown document"}]}"#.into());
    };
    (200, answer.to_string())
}

pub const YOUTRACK_ISSUES: &str = r#"[
  {"id":"2-12","idReadable":"ABC-12","summary":"Crash on startup","resolved":null,"updated":1790000000000,
   "reporter":{"login":"kai"},"tags":[],
   "customFields":[{"name":"State","value":{"name":"Open"}},{"name":"Type","value":{"name":"Bug"}}]}]"#;

pub const REDMINE_ISSUES: &str = r#"{"issues":[
  {"id":12,"project":{"id":1,"name":"Bench"},"tracker":{"id":1,"name":"Bug"},
   "status":{"id":1,"name":"New","is_closed":false},"author":{"id":1,"name":"Kai"},
   "assigned_to":{"id":1,"name":"Kai"},"subject":"Crash on startup",
   "updated_on":"2026-09-29T10:00:00Z","closed_on":null}],"total_count":1}"#;

/// The routes of the providers beyond GitLab (`None`: not one of theirs).
fn providers_route(method: &str, path: &str) -> Option<(u16, String)> {
    let answer = match (method, path) {
        ("GET", "/DefaultCollection/_apis/git/repositories") => AZURE_REPOS.to_string(),
        ("GET", "/DefaultCollection/P/_apis/git/repositories/R") => {
            r#"{"id":"rid","name":"R","project":{"name":"P"}}"#.to_string()
        }
        ("GET", "/DefaultCollection/P/_apis/git/repositories/rid/pullrequests") => {
            AZURE_PULLS.to_string()
        }
        ("GET", "/rest/api/2/issue/createmeta/ABC/issuetypes") => JIRA_TYPES.to_string(),
        ("POST", "/rest/api/2/issue") => {
            return Some((201, r#"{"id":"10010","key":"ABC-13"}"#.into()))
        }
        ("GET", "/api/users/me") => r#"{"id":"1-1","login":"kai","fullName":"Kai"}"#.to_string(),
        ("GET", "/api/admin/projects") => {
            r#"[{"id":"0-1","shortName":"ABC","name":"Alphabet"}]"#.to_string()
        }
        ("GET", "/api/issues") => YOUTRACK_ISSUES.to_string(),
        ("GET", "/issues.json") => REDMINE_ISSUES.to_string(),
        ("GET", "/1/members/me/boards") => TRELLO_BOARDS.to_string(),
        ("GET", "/1/boards/AbCd1234/lists") => TRELLO_LISTS.to_string(),
        ("GET", "/1/boards/AbCd1234/cards/open") => TRELLO_CARDS.to_string(),
        ("POST", "/1/cards") => {
            return Some((
                200,
                r#"{"idShort":8,"shortUrl":"http://127.0.0.1/c/new8"}"#.into(),
            ))
        }
        _ => return None,
    };
    Some((200, answer))
}

pub fn start_mock() -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
    let port = listener.local_addr().unwrap().port();
    // The HTTP loop is `xtask::mock_host::http`'s, shared with the fake
    // hosting service; only the routes are this file's.
    let log = xtask::mock_host::http::serve(listener, move |request| {
        if request.target.starts_with("/api/v4/projects") {
            let ms = SLOW_PROJECTS_MS
                .lock()
                .unwrap()
                .iter()
                .find(|(p, _)| *p == port)
                .map(|(_, ms)| *ms);
            if let Some(ms) = ms {
                std::thread::sleep(Duration::from_millis(ms));
            }
        }
        let forced = if request.method == "POST" {
            POST_FAILS
                .lock()
                .unwrap()
                .iter()
                .find(|(p, _)| *p == port)
                .map(|(_, status)| *status)
        } else {
            None
        };
        if request.method == "POST" && request.target == "/graphql" {
            return linear_route(&request.body);
        }
        if forced.is_none() {
            if let Some(answer) = conversation_write(request) {
                return answer;
            }
        }
        match forced {
            Some(401) => (401, r#"{"message":"401 Unauthorized"}"#.to_string()),
            Some(status) => (status, r#"{"message":"boom from the mock"}"#.to_string()),
            None => route(&request.method, &request.target, port),
        }
    });
    let seen = log;
    Mock { port, seen }
}
