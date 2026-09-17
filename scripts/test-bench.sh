#!/usr/bin/env bash
# The real-server test bench: local Docker containers GitPoulpi's integrations
# are checked against by hand (and by the `#[ignore]` live tests, which also need GITPOULPI_LIVE_TESTS=1: docs/developer/testing.md). Never part
# of the gate. Everything binds to 127.0.0.1 only; data lives in named volumes
# `gitpoulpi-*`; secrets go to a scratch directory, never to the repository.
#
#   scripts/test-bench.sh up [name...]      create what is missing, start what is stopped
#   scripts/test-bench.sh seed [name...]    admin user, token, sample content (idempotent)
#   scripts/test-bench.sh status [name...]  container state, HTTP answer, token file
#   scripts/test-bench.sh down [name...]    stop the containers (volumes are kept)
#
# Names: gitlab gitea forgejo redmine gerrit (default: all). `up` never
# recreates an existing container. `down` never removes anything.
#
# Environment:
#   GITPOULPI_BENCH_DIR       where tokens and the bench password are written
#                             (mode 600; default ~/.cache/gitpoulpi-bench)
#   GITPOULPI_BENCH_PASSWORD  the admin password used by `up` (GitLab root) and
#                             `seed` (bench user); default: generated once and
#                             kept in $GITPOULPI_BENCH_DIR/password
#   GITPOULPI_BENCH_USER      the admin `seed` creates on Gitea and Forgejo (default: poulpi)
#   DOCKER                    the docker command (default: docker)
#
# YouTrack (jetbrains/youtrack, port 8080) is manual: its first start needs a
# browser wizard (a one-time URL printed in the container log), so the script
# does not manage it. See docs/developer/testing.md.
#
# Tokens are never printed. Point GitPoulpi at a bench with a scratch
# environment (HOME, GITPOULPI_CONFIG_DIR, XDG_* all inside a throwaway
# directory) so the real configuration is never touched.

set -euo pipefail

DOCKER=${DOCKER:-docker}
BENCH_DIR=${GITPOULPI_BENCH_DIR:-$HOME/.cache/gitpoulpi-bench}
ALL=(gitlab gitea forgejo redmine gerrit)
BENCH_USER=${GITPOULPI_BENCH_USER:-poulpi}
BENCH_REPO=gitpoulpi-sandbox

die() { echo "test-bench: $*" >&2; exit 1; }
say() { echo "test-bench: $*"; }

mkdir -p "$BENCH_DIR"
chmod 700 "$BENCH_DIR"
umask 077

password() {
  if [ -n "${GITPOULPI_BENCH_PASSWORD:-}" ]; then
    printf '%s' "$GITPOULPI_BENCH_PASSWORD"
    return
  fi
  if [ ! -s "$BENCH_DIR/password" ]; then
    # Letters, digits and one of each class: passes every product's policy.
    { LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 18; echo 'aA1!'; } | tr -d '\n' >"$BENCH_DIR/password"
    chmod 600 "$BENCH_DIR/password"
  fi
  cat "$BENCH_DIR/password"
}

exists() { $DOCKER inspect "$1" >/dev/null 2>&1; }
running() { [ "$($DOCKER inspect -f '{{.State.Running}}' "$1" 2>/dev/null || echo false)" = true ]; }

# ensure <container> <docker run args...>: create when missing, start when stopped.
ensure() {
  local name=$1
  shift
  if exists "$name"; then
    if running "$name"; then
      say "$name is already running"
    else
      say "starting $name"
      $DOCKER start "$name" >/dev/null
    fi
  else
    say "creating $name"
    $DOCKER run -d --name "$name" --restart unless-stopped "$@" >/dev/null
  fi
}

wait_http() { # url seconds [expected-status-regex]
  local url=$1 secs=${2:-180} want=${3:-^[23]} i code
  for ((i = 0; i < secs; i += 3)); do
    code=$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)
    if [[ $code =~ $want ]]; then return 0; fi
    sleep 3
  done
  die "$url did not answer in ${secs}s"
}

# token_file <name> -> path; a token is valid when `check` answers 200.
token_path() { echo "$BENCH_DIR/$1.token"; }
have_token() { # name check-url header-name header-prefix
  local f
  f=$(token_path "$1")
  [ -s "$f" ] || return 1
  [ "$(curl -s -o /dev/null -w '%{http_code}' -H "$3: $4$(cat "$f")" "$2" || true)" = 200 ]
}

# ---------------------------------------------------------------- up

up_gitlab() {
  # Slow to boot (minutes) and 5+ GB. The root password comes from the bench password.
  ensure gitpoulpi-gitlab --hostname localhost --shm-size 256m \
    -p 127.0.0.1:8929:8929 -p 127.0.0.1:2224:22 \
    -e "GITLAB_ROOT_PASSWORD=$(password)" \
    -e "GITLAB_OMNIBUS_CONFIG=external_url 'http://localhost:8929'; gitlab_rails['gitlab_shell_ssh_port']=2224; puma['worker_processes']=0; sidekiq['concurrency']=5; prometheus_monitoring['enable']=false; gitlab_rails['usage_ping_enabled']=false" \
    -v gitpoulpi-gitlab-config:/etc/gitlab -v gitpoulpi-gitlab-logs:/var/log/gitlab \
    -v gitpoulpi-gitlab-data:/var/opt/gitlab \
    gitlab/gitlab-ce:latest
}

up_gitea() {
  ensure gitpoulpi-gitea -p 127.0.0.1:3000:3000 \
    -e GITEA__security__INSTALL_LOCK=true -e GITEA__server__ROOT_URL=http://localhost:3000/ \
    -e GITEA__database__DB_TYPE=sqlite3 -v gitpoulpi-gitea-data:/data gitea/gitea:latest
}

up_forgejo() {
  ensure gitpoulpi-forgejo -p 127.0.0.1:3001:3000 \
    -e FORGEJO__security__INSTALL_LOCK=true -e FORGEJO__server__ROOT_URL=http://localhost:3001/ \
    -e FORGEJO__database__DB_TYPE=sqlite3 -v gitpoulpi-forgejo-data:/data \
    codeberg.org/forgejo/forgejo:11
}

up_redmine() {
  # The image's default database is SQLite (in /usr/src/redmine/sqlite).
  ensure gitpoulpi-redmine -p 127.0.0.1:3002:3000 \
    -v gitpoulpi-redmine-data:/usr/src/redmine/sqlite -v gitpoulpi-redmine-files:/usr/src/redmine/files \
    redmine:latest
}

up_gerrit() {
  # DEVELOPMENT_BECOME_ANY_ACCOUNT authentication (the image default): the
  # first account to log in becomes the administrator. Localhost only.
  ensure gitpoulpi-gerrit -p 127.0.0.1:8081:8080 -p 127.0.0.1:29418:29418 \
    -e CANONICAL_WEB_URL=http://127.0.0.1:8081 \
    -v gitpoulpi-gerrit-etc:/var/gerrit/etc -v gitpoulpi-gerrit-git:/var/gerrit/git \
    -v gitpoulpi-gerrit-db:/var/gerrit/db -v gitpoulpi-gerrit-index:/var/gerrit/index \
    -v gitpoulpi-gerrit-cache:/var/gerrit/cache \
    gerritcodereview/gerrit:latest
}

# ---------------------------------------------------------------- seed

# Gitea and Forgejo share the CLI (`gitea` binary; `forgejo` in Forgejo) and the API.
seed_gitea_like() { # container binary port
  local c=$1 bin=$2 port=$3 base="http://127.0.0.1:$3" pw tok
  running "$c" || die "$c is not running (scripts/test-bench.sh up ${c#gitpoulpi-})"
  wait_http "$base/api/v1/version" 120
  pw=$(password)
  if ! have_token "${c#gitpoulpi-}" "$base/api/v1/user" Authorization "token "; then
    if ! $DOCKER exec -u git "$c" "$bin" admin user list 2>/dev/null | awk 'NR>1{print $2}' | grep -qx "$BENCH_USER"; then
      say "$c: creating the admin user $BENCH_USER"
      $DOCKER exec -u git "$c" "$bin" admin user create --admin --username "$BENCH_USER" \
        --password "$pw" --email "$BENCH_USER@localhost" --must-change-password=false >/dev/null
    fi
    say "$c: minting a token"
    tok=$($DOCKER exec -u git "$c" "$bin" admin user generate-access-token --username "$BENCH_USER" \
      --token-name "gp-bench-$(date +%s)" --scopes all --raw)
    printf '%s' "$tok" >"$(token_path "${c#gitpoulpi-}")"
  fi
  tok=$(cat "$(token_path "${c#gitpoulpi-}")")
  local code
  code=$(curl -s -o /dev/null -w '%{http_code}' -X POST -H "Authorization: token $tok" \
    -H 'Content-Type: application/json' "$base/api/v1/user/repos" \
    -d "{\"name\":\"$BENCH_REPO\",\"auto_init\":true,\"default_branch\":\"main\",\"readme\":\"Default\"}")
  case $code in 201) say "$c: repository $BENCH_USER/$BENCH_REPO created" ;; 409) ;; *) die "$c: creating the repository answered $code" ;; esac
}

seed_gitlab() {
  local base=http://127.0.0.1:8929 tok code
  running gitpoulpi-gitlab || die "gitpoulpi-gitlab is not running"
  wait_http "$base/users/sign_in" 900 '^200$'
  if ! have_token gitlab "$base/api/v4/user" PRIVATE-TOKEN ""; then
    say "gitlab: minting a root token (a minute)"
    tok=$($DOCKER exec gitpoulpi-gitlab gitlab-rails runner "
      u = User.find_by_username('root')
      t = u.personal_access_tokens.create!(name: 'gp-bench-' + Time.now.to_i.to_s, scopes: [:api, :read_user, :read_repository, :write_repository], expires_at: 300.days.from_now)
      puts t.token" | tail -n 1)
    printf '%s' "$tok" >"$(token_path gitlab)"
  fi
  tok=$(cat "$(token_path gitlab)")
  code=$(curl -s -o /dev/null -w '%{http_code}' -X POST -H "PRIVATE-TOKEN: $tok" \
    "$base/api/v4/projects?name=$BENCH_REPO&initialize_with_readme=true")
  case $code in 201) say "gitlab: project root/$BENCH_REPO created" ;; 400) ;; *) die "gitlab: creating the project answered $code" ;; esac
}

# Redmine: default data (trackers, statuses, priorities), the REST API on, an
# admin API key, a project with a version and a few issues.
seed_redmine() {
  local c=gitpoulpi-redmine base=http://127.0.0.1:3002 key
  running "$c" || die "$c is not running (scripts/test-bench.sh up redmine)"
  wait_http "$base/" 240
  if [ "$($DOCKER exec "$c" bin/rails runner 'puts IssueStatus.count' 2>/dev/null | tail -n 1)" = 0 ]; then
    say "redmine: loading the default data"
    $DOCKER exec -e RAILS_ENV=production -e REDMINE_LANG=en "$c" bundle exec rake redmine:load_default_data >/dev/null
  fi
  if ! have_token redmine "$base/users/current.json" X-Redmine-API-Key ""; then
    say "redmine: enabling the REST API and minting the admin key"
    key=$($DOCKER exec -e BENCH_PW="$(password)" "$c" bin/rails runner '
      Setting.rest_api_enabled = "1"
      u = User.find_by_login("admin")
      u.password = u.password_confirmation = ENV["BENCH_PW"]
      u.must_change_passwd = false
      u.save!
      puts u.api_key' | tail -n 1)
    printf '%s' "$key" >"$(token_path redmine)"
  fi
  $DOCKER exec "$c" bin/rails runner '
    admin = User.find_by_login("admin")
    project = Project.find_by_identifier("bench") || Project.create!(
      name: "Bench", identifier: "bench", is_public: false,
      enabled_module_names: %w[issue_tracking])
    project.trackers = Tracker.all
    project.save!
    unless Member.where(project_id: project.id, user_id: admin.id).exists?
      Member.create!(project: project, user: admin, roles: [Role.find_by(name: "Manager") || Role.first])
    end
    version = Version.find_by(project_id: project.id, name: "v0.1") ||
      Version.create!(project: project, name: "v0.1", status: "open", due_date: Date.today + 30)
    if Issue.where(project_id: project.id).count.zero?
      tracker = Tracker.order(:position).first
      [["Crash on empty name", admin, version], ["Add a dark theme", nil, nil], ["Docs are out of date", admin, nil]].each do |subject, assignee, fixed|
        Issue.create!(project: project, tracker: tracker, author: admin, subject: subject,
                      description: "Seeded by scripts/test-bench.sh", assigned_to: assignee, fixed_version: fixed,
                      priority: IssuePriority.default || IssuePriority.first)
      end
    end' >/dev/null
  say "redmine: project bench ready"
}

# Gerrit (3.14, "become any account" authentication): the image creates the
# `admin` account with the initial token `secret`. Seeding mints a token of
# its own through the REST API (HTTP basic is the one scheme GitPoulpi speaks),
# creates the project `gitpoulpi-sandbox` and pushes one change for review.
seed_gerrit() {
  local c=gitpoulpi-gerrit base=http://127.0.0.1:8081 tmp tok user=admin
  running "$c" || die "$c is not running (scripts/test-bench.sh up gerrit)"
  wait_http "$base/config/server/version" 300
  printf '%s' "$user" >"$BENCH_DIR/gerrit.user"
  if [ ! -s "$(token_path gerrit)" ] ||
    [ "$(curl -s -o /dev/null -w '%{http_code}' -u "$user:$(cat "$(token_path gerrit)")" "$base/a/accounts/self" || true)" != 200 ]; then
    say "gerrit: minting a token for $user"
    tok=$(curl -s -u "$user:${GITPOULPI_BENCH_GERRIT_INITIAL:-secret}" -X PUT \
      -H 'Content-Type: application/json' -d '{}' \
      "$base/a/accounts/self/tokens/gp-bench-$(date +%s)" | tail -n +2 |
      python3 -c 'import json,sys; print(json.load(sys.stdin)["token"], end="")') ||
      die "gerrit: the initial token was refused (set GITPOULPI_BENCH_GERRIT_INITIAL)"
    printf '%s' "$tok" >"$(token_path gerrit)"
  fi
  local auth=(-u "$user:$(cat "$(token_path gerrit)")")
  curl -s "${auth[@]}" -X PUT -H 'Content-Type: application/json' \
    -d '{"create_empty_commit":true,"description":"GitPoulpi bench","branches":["master"]}' \
    "$base/a/projects/$BENCH_REPO" -o /dev/null || true
  local open
  open=$(curl -s "${auth[@]}" "$base/a/changes/?q=project:$BENCH_REPO+status:open" | tail -n +2)
  if [ "$open" = "[]" ] || [ -z "$open" ]; then
    say "gerrit: pushing a change for review"
    tmp=$(mktemp -d)
    # shellcheck disable=SC2064
    trap "rm -rf '$tmp'" RETURN
    # Credentials travel as a header, never in the URL (error messages echo URLs).
    local header
    header="Authorization: Basic $(printf '%s:%s' "$user" "$(cat "$(token_path gerrit)")" | base64 -w0)"
    git -c "http.extraHeader=$header" clone -q "$base/a/$BENCH_REPO" "$tmp/repo"
    (
      cd "$tmp/repo"
      git config user.name Bench
      git config user.email bench@localhost
      mkdir -p src
      printf 'fn main() {\n    println!("hello");\n}\n' >src/main.rs
      git add src/main.rs
      git commit -q -m "Add a main" -m "Seeded by scripts/test-bench.sh"
      # Gerrit wants a Change-Id: the commit-msg hook adds it on the amend.
      curl -s -o .git/hooks/commit-msg "$base/tools/hooks/commit-msg"
      chmod +x .git/hooks/commit-msg
      git commit -q --amend --no-edit
      git -c "http.extraHeader=$header" push -q origin HEAD:refs/for/master
    )
  fi
  say "gerrit: project $BENCH_REPO ready"
}

# ---------------------------------------------------------------- status

status_one() {
  local name=$1 c="gitpoulpi-$1" port url state tok
  case $name in
    gitlab) port=8929; url=http://127.0.0.1:8929/users/sign_in ;;
    gitea) port=3000; url=http://127.0.0.1:3000/api/v1/version ;;
    forgejo) port=3001; url=http://127.0.0.1:3001/api/v1/version ;;
    redmine) port=3002; url=http://127.0.0.1:3002/ ;;
    gerrit) port=8081; url=http://127.0.0.1:8081/config/server/version ;;
  esac
  if ! exists "$c"; then
    printf '%-9s %-10s port %-5s\n' "$name" missing "$port"
    return
  fi
  state=$($DOCKER inspect -f '{{.State.Status}}' "$c")
  tok=no
  [ -s "$(token_path "$name")" ] && tok=yes
  printf '%-9s %-10s port %-5s http %-4s token file %s\n' "$name" "$state" "$port" \
    "$(curl -s -o /dev/null -m 5 -w '%{http_code}' "$url" || true)" "$tok"
}

# ---------------------------------------------------------------- main

cmd=${1:-}
[ -n "$cmd" ] || die "usage: $0 up|down|seed|status [gitlab gitea forgejo redmine gerrit]"
shift
names=("$@")
[ ${#names[@]} -gt 0 ] || names=("${ALL[@]}")
for n in "${names[@]}"; do
  case $n in gitlab | gitea | forgejo | redmine | gerrit) ;; *) die "unknown bench '$n' (${ALL[*]})" ;; esac
done

case $cmd in
  up) for n in "${names[@]}"; do "up_$n"; done ;;
  seed)
    for n in "${names[@]}"; do
      case $n in
        gitea) seed_gitea_like gitpoulpi-gitea gitea 3000 ;;
        forgejo) seed_gitea_like gitpoulpi-forgejo forgejo 3001 ;;
        *) "seed_$n" ;;
      esac
    done
    ;;
  status) for n in "${names[@]}"; do status_one "$n"; done ;;
  down)
    for n in "${names[@]}"; do
      if running "gitpoulpi-$n"; then
        say "stopping gitpoulpi-$n"
        $DOCKER stop "gitpoulpi-$n" >/dev/null
      fi
    done
    ;;
  *) die "unknown command '$cmd' (up|down|seed|status)" ;;
esac
