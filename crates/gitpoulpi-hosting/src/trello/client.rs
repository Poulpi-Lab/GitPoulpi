//! The Trello client.

use std::collections::HashMap;
use std::sync::Arc;

use crate::auth::auth_headers;
use crate::client::{
    encode_component, CreatedIssue, CreatedPullRequest, HostingClient, NewIssue, NewPullRequest,
};
use crate::http::{HttpRequest, HttpTransport};
use crate::integration::Integration;
use crate::model::{
    HostedUser, IssueCreateOptions, IssueFilter, IssueSummary, StateFilter, TokenCheck,
};
use crate::project::{HostedProject, HostedSshKey};
use crate::{Error, Result};

use super::map::{self, CARD_FIELDS};

pub(crate) struct TrelloClient {
    integration: Integration,
    token: String,
    transport: Arc<dyn HttpTransport>,
}

impl TrelloClient {
    pub(crate) fn new(
        integration: Integration,
        token: String,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            integration,
            token,
            transport,
        }
    }

    /// The key and token go in this header; no URL ever carries them.
    fn authed(&self, request: HttpRequest) -> HttpRequest {
        let mut request = request.header("Accept", "application/json");
        for (name, value) in auth_headers(&self.integration, &self.token) {
            request = request.header(name, value);
        }
        request
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str, what: &str) -> Result<T> {
        let url = format!("{}{path}", self.integration.api_base());
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(url)))
            .await?;
        response.error_for_status(what)?;
        response.json()
    }

    async fn open_lists(&self, board: &str) -> Result<Vec<map::TrelloList>> {
        self.get_json(
            &format!(
                "/boards/{}/lists?filter=open&fields=name",
                encode_component(board.trim())
            ),
            "reading the Trello lists",
        )
        .await
    }
}

/// `Todo, Doing` to `["todo", "doing"]`.
fn wanted_lists(query: Option<&str>) -> Vec<String> {
    query
        .unwrap_or("")
        .split(',')
        .map(|n| n.trim().to_lowercase())
        .filter(|n| !n.is_empty())
        .collect()
}

#[async_trait::async_trait]
impl HostingClient for TrelloClient {
    fn integration(&self) -> &Integration {
        &self.integration
    }

    fn unsupported(&self, what: &str) -> Error {
        Error::Unsupported(what.to_string())
    }

    async fn list_projects(&self) -> Result<Vec<HostedProject>> {
        let rows: Vec<map::TrelloBoard> = self
            .get_json(
                "/members/me/boards?filter=open&fields=name,shortLink,url,desc,prefs",
                "listing Trello boards",
            )
            .await?;
        let id = &self.integration.id;
        Ok(rows.into_iter().map(|b| map::map_board(id, b)).collect())
    }

    async fn list_ssh_keys(&self) -> Result<Vec<HostedSshKey>> {
        Err(self.unsupported("list_ssh_keys"))
    }

    async fn upload_ssh_key(&self, _title: &str, _public_key: &str) -> Result<HostedSshKey> {
        Err(self.unsupported("upload_ssh_key"))
    }

    async fn create_pull_request(&self, _req: &NewPullRequest) -> Result<CreatedPullRequest> {
        Err(self.unsupported("create_pull_request"))
    }

    async fn current_user(&self) -> Result<HostedUser> {
        let me: map::TrelloMember = self
            .get_json(
                "/members/me?fields=username,fullName,url",
                "checking the Trello token",
            )
            .await?;
        Ok(map::map_user(me))
    }

    async fn check_token(&self) -> Result<TokenCheck> {
        // One call only: the members endpoints have the tightest limit.
        Ok(TokenCheck {
            user: self.current_user().await?,
            // Trello tokens carry no scope list.
            scopes: None,
            missing_scopes: Vec::new(),
            expires_at: None,
            ..Default::default()
        })
    }

    async fn list_issues(&self, board: &str, filter: &IssueFilter) -> Result<Vec<IssueSummary>> {
        let which = match filter.state {
            StateFilter::Open => "open",
            StateFilter::Closed => "closed",
            StateFilter::All => "all",
        };
        let board = encode_component(board.trim());
        let lists = self.open_lists(&board).await?;
        let names: HashMap<String, String> = lists
            .iter()
            .map(|l| (l.id.clone(), l.name.clone()))
            .collect();
        let wanted = wanted_lists(filter.query.as_deref());
        let me = if filter.assigned_to_me {
            Some(self.current_user().await?.id)
        } else {
            None
        };
        let cards: Vec<map::TrelloCard> = self
            .get_json(
                &format!(
                    "/boards/{board}/cards/{which}?fields={CARD_FIELDS}&members=true&limit=1000"
                ),
                "listing Trello cards",
            )
            .await?;
        Ok(cards
            .into_iter()
            .filter(|c| {
                wanted.is_empty()
                    || names
                        .get(&c.id_list)
                        .is_some_and(|n| wanted.contains(&n.to_lowercase()))
            })
            .filter(|c| me.as_ref().map_or(true, |me| c.id_members.contains(me)))
            .map(|c| map::map_card(&names, c))
            .collect())
    }

    async fn create_issue(&self, req: &NewIssue) -> Result<CreatedIssue> {
        let list = match req.list_id.as_deref().filter(|l| !l.is_empty()) {
            Some(l) => l.to_string(),
            None => self
                .issue_create_options(&req.project)
                .await?
                .default_list
                .ok_or_else(|| Error::Api {
                    what: "opening a Trello card".into(),
                    status: 400,
                    detail: "the board has no open list to put the card in".into(),
                })?,
        };
        let mut body = serde_json::json!({ "idList": list, "name": req.title });
        if !req.body.trim().is_empty() {
            body["desc"] = serde_json::json!(req.body);
        }
        let url = format!("{}/cards", self.integration.api_base());
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, &body)))
            .await?;
        response.error_for_status("opening a Trello card")?;
        let created: map::CreatedCard = response.json()?;
        let web_url = created.web_url();
        Ok(CreatedIssue {
            number: created.number(),
            reference: web_url.clone(),
            web_url,
        })
    }

    fn board_source(&self) -> crate::board::BoardSource {
        crate::board::BoardSource::Lists
    }

    /// The board's open lists, left to right.
    async fn issue_statuses(&self, board: &str) -> Result<Vec<String>> {
        Ok(self
            .open_lists(board)
            .await?
            .into_iter()
            .map(|l| l.name)
            .collect())
    }

    /// Moves the card to the list named like the column
    /// (`PUT /cards/{id}?idList=..`).
    async fn move_issue(
        &self,
        board: &str,
        issue: &IssueSummary,
        spec: &crate::board::BoardSpec,
        to: &crate::board::BoardColumn,
    ) -> Result<()> {
        let plan = crate::board::plan_move(spec, issue, to)?;
        let Some(list) = plan.list else {
            return Ok(());
        };
        let lists = self.open_lists(board).await?;
        let target = lists
            .iter()
            .find(|l| l.name.trim().eq_ignore_ascii_case(list.trim()))
            .ok_or_else(|| Error::Api {
                what: "moving a Trello card".into(),
                status: 404,
                detail: format!("the board has no open list named `{list}`"),
            })?;
        let url = format!(
            "{}/cards/{}",
            self.integration.api_base(),
            encode_component(&issue.id)
        );
        let response = self
            .transport
            .send(self.authed(HttpRequest::put_json(
                url,
                &serde_json::json!({ "idList": target.id }),
            )))
            .await?;
        response.error_for_status("moving a Trello card")
    }

    async fn issue_create_options(&self, board: &str) -> Result<IssueCreateOptions> {
        Ok(map::map_options(self.open_lists(board).await?))
    }
}
