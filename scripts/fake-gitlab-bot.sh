#!/usr/bin/env bash
# A fake GitLab bot for host-driven flows (docs/user/branching-workflows.md):
# watches a project's open issues for a label and, for each new one, makes
#
#   <type>/<iid>-<slug>      a branch from the target branch with one commit
#   a draft merge request    "Closes #<iid>", from that branch into the target
#
# <type> comes from a `workflow::<type>` label (default `feature`), <slug> from
# the issue title without its `[type] ` prefix. It is the stand-in for the
# real bots (a GitLab webhook handler, a CI job) on the Docker bench
# (scripts/test-bench.sh): `gitpoulpi flow issue new --wait --pick-up` creates
# an issue, this script makes what a bot would, the flow picks it up.
#
#   scripts/fake-gitlab-bot.sh [--url URL] [--token-file FILE | --token TOKEN]
#       [--project GROUP/PROJECT] [--label LABEL] [--target BRANCH]
#       [--interval SECONDS] [--delay SECONDS] [--once] [--no-request]
#
# Defaults: http://localhost:8929, the bench's gitlab.token (GITPOULPI_BENCH_DIR,
# default ~/.cache/gitpoulpi-bench), project root/gitpoulpi-sandbox, label `bot`,
# target the project's default branch, poll every 2 s, no delay. `--once` handles what is open now and
# exits. Issues already handled (by a branch named <type>/<iid>-*) are skipped,
# so it can be restarted. Needs curl and jq. The token needs the `api` scope.
set -euo pipefail

url=http://localhost:8929
token=
token_file=${GITPOULPI_BENCH_DIR:-$HOME/.cache/gitpoulpi-bench}/gitlab.token
project=root/gitpoulpi-sandbox
label=bot
target=
interval=2
delay=0
once=0
request=1

while [ $# -gt 0 ]; do
  case $1 in
    --url) url=$2; shift 2 ;;
    --token) token=$2; shift 2 ;;
    --token-file) token_file=$2; shift 2 ;;
    --project) project=$2; shift 2 ;;
    --label) label=$2; shift 2 ;;
    --target) target=$2; shift 2 ;;
    --interval) interval=$2; shift 2 ;;
    --delay) delay=$2; shift 2 ;;
    --once) once=1; shift ;;
    --no-request) request=0; shift ;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option $1" >&2; exit 2 ;;
  esac
done
command -v jq >/dev/null || { echo "jq is needed" >&2; exit 1; }
[ -n "$token" ] || token=$(tr -d '\n' < "$token_file")

api() { # method path [curl args...]
  local method=$1 path=$2
  shift 2
  curl -sS -X "$method" -H "PRIVATE-TOKEN: $token" "$@" "$url/api/v4$path"
}

pid=$(printf '%s' "$project" | jq -sRr @uri)
[ -n "$target" ] || target=$(api GET "/projects/$pid" | jq -er .default_branch) \
  || { echo "could not read the project's default branch (--target BRANCH)" >&2; exit 1; }

slug_of() { # title -> slug
  printf '%s' "$1" | sed -E 's/^\[[^]]*\] *//' | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//' | cut -c1-50
}

handle() { # one issue as JSON
  local iid title type slug branch existing
  iid=$(jq -r .iid <<<"$1")
  title=$(jq -r .title <<<"$1")
  type=$(jq -r '[.labels[] | select(startswith("workflow::")) | sub("^workflow::"; "")] | first // "feature"' <<<"$1")
  slug=$(slug_of "$title")
  [ -n "$slug" ] || slug=issue
  branch="$type/$iid-$slug"
  existing=$(api GET "/projects/$pid/repository/branches?search=$(printf '%s' "$type/$iid-" | jq -sRr @uri)" \
    | jq -r --arg p "$type/$iid-" '[.[] | select(.name | startswith($p))] | length')
  if [ "$existing" != 0 ]; then
    return 0
  fi
  [ "$delay" = 0 ] || sleep "$delay"
  # The branch and its one commit, in a single call.
  api POST "/projects/$pid/repository/commits" -H 'Content-Type: application/json' \
    -d "$(jq -n --arg b "$branch" --arg s "$target" --arg m "Start work on #$iid" --arg f "bot/issue-$iid.md" \
          --arg c "Prepared by the bot for issue #$iid: $title" \
          '{branch:$b, start_branch:$s, commit_message:$m, actions:[{action:"create", file_path:$f, content:$c}]}')" \
    | jq -e '.id' >/dev/null || { echo "could not make $branch" >&2; return 1; }
  echo "bot: made $branch for #$iid"
  if [ "$request" = 1 ]; then
    api POST "/projects/$pid/merge_requests" -H 'Content-Type: application/json' \
      -d "$(jq -n --arg b "$branch" --arg t "$target" --arg title "Draft: Resolve \"$title\"" --arg d "Closes #$iid" \
            '{source_branch:$b, target_branch:$t, title:$title, description:$d}')" \
      | jq -r '"bot: opened merge request !" + (.iid|tostring)'
  fi
}

while :; do
  issues=$(api GET "/projects/$pid/issues?state=opened&labels=$(printf '%s' "$label" | jq -sRr @uri)&per_page=100")
  jq -e 'type == "array"' >/dev/null <<<"$issues" || { echo "unexpected answer: $issues" >&2; exit 1; }
  while IFS= read -r issue; do
    [ -n "$issue" ] && handle "$issue"
  done < <(jq -c '.[]' <<<"$issues")
  [ "$once" = 1 ] && break
  sleep "$interval"
done
