//! The GraphQL documents. Plain strings: the variables travel apart, so
//! nothing the user typed is ever spliced into a query.

pub(super) const VIEWER: &str = "query Viewer { viewer { id name displayName email url } }";

pub(super) const TEAMS: &str = "query Teams($after: String) { \
    organization { urlKey } \
    teams(first: 100, after: $after) { \
        nodes { id key name description } \
        pageInfo { hasNextPage endCursor } } }";

pub(super) const TEAM_BY_KEY: &str = "query Team($key: String!) { \
    teams(filter: { key: { eq: $key } }, first: 1) { nodes { id key name } } }";

/// What a list row needs: see `map::IssueRow`.
pub(super) const ISSUES: &str = "query Issues($filter: IssueFilter, $after: String) { \
    issues(filter: $filter, first: 100, after: $after, orderBy: updatedAt) { \
        nodes { id identifier number title url branchName updatedAt priorityLabel \
            state { name type } creator { name displayName } \
            assignee { name displayName } labels { nodes { name } } } \
        pageInfo { hasNextPage endCursor } } }";

pub(super) const MEMBERS: &str = "query Members($id: String!, $after: String) { \
    team(id: $id) { members(first: 100, after: $after) { \
        nodes { id name displayName email active } \
        pageInfo { hasNextPage endCursor } } } }";

pub(super) const STATES: &str = "query States($id: String!, $after: String) { \
    team(id: $id) { defaultIssueState { id } \
        states(first: 100, after: $after) { \
            nodes { id name type position } \
            pageInfo { hasNextPage endCursor } } } }";

pub(super) const LABELS: &str = "query Labels($id: ID!, $after: String) { \
    issueLabels(filter: { or: [ { team: { id: { eq: $id } } }, { team: { null: true } } ] }, \
        first: 100, after: $after) { \
        nodes { id name color description } \
        pageInfo { hasNextPage endCursor } } }";

pub(super) const CYCLES: &str = "query Cycles($id: String!, $after: String) { \
    team(id: $id) { cycles(filter: { isCompleted: { eq: false } }, first: 50, after: $after) { \
        nodes { id number name startsAt endsAt isActive } \
        pageInfo { hasNextPage endCursor } } } }";

pub(super) const CYCLE_BY_NUMBER: &str = "query Cycle($id: ID!, $number: Float!) { \
    cycles(filter: { team: { id: { eq: $id } }, number: { eq: $number } }, first: 1) { \
        nodes { id number name startsAt endsAt isActive } } }";

pub(super) const ISSUE_META: &str = "query IssueMeta($filter: IssueFilter) { \
    issues(filter: $filter, first: 1) { nodes { id \
        assignee { name displayName } labels { nodes { name } } \
        cycle { id number name startsAt endsAt isActive } } } }";

pub(super) const ISSUE_ID: &str = "query IssueId($filter: IssueFilter) { \
    issues(filter: $filter, first: 1) { nodes { id } } }";

pub(super) const ISSUE_CREATE: &str = "mutation IssueCreate($input: IssueCreateInput!) { \
    issueCreate(input: $input) { success issue { id identifier number url } } }";

pub(super) const ISSUE_UPDATE: &str =
    "mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) { \
    issueUpdate(id: $id, input: $input) { success } }";
