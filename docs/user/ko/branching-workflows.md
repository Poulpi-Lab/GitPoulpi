[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · **한국어** · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# 브랜치 워크플로

브랜치 워크플로는 팀이 브랜치를 어떻게 쓰는지 GitPoulpi에 알려 줍니다. 어떤 브랜치가 영구적으로 살아 있는지
(`main`, `develop`, `production`), 어떤 브랜치가 수명이 짧은 토픽 브랜치인지
(`feature/...`, `hotfix/...`), 이름을 어떻게 짓는지, 각 브랜치가 어디에서 시작해서 어디로 끝나는지를 정의합니다.
Git Flow를 사용자 정의할 수 있게 만든 것입니다: 고정 이름, 접두사와 접미사, 또는 자리 표시자가 있는 패턴을 쓸 수 있으므로,
봇이 이슈에서 만든 브랜치도 직접 만든 브랜치처럼 인식됩니다.

이 페이지는 모델, 파일, 프리셋, 감지와 가져오기, 읽기 전용 명령, 브랜치를 시작, 완료, 동기화,
승격하는 동작(CLI, MCP, 앱), 그리고 앱의 화면인 설정 편집기, 묶인 사이드바, `Flow` 메뉴와 그 대화상자를 다룹니다.

## 개념

- **베이스 브랜치**는 오래 유지됩니다. 베이스에는 **부모**가 있을 수 있습니다(`develop`은 `main`으로 들어감). 트렁크에는 없습니다.
  베이스는 후보 이름을 여럿 나열할 수 있습니다(`main` 또는 `master`, `develop` 또는 `dev`). 존재하는 쪽이 쓰입니다.
- **토픽 유형**은 수명이 짧은 브랜치의 한 종류입니다(feature, bugfix, release, hotfix, support...).
  **이름 패턴**, **부모**(시작하는 곳이자 완료되어 들어가는 곳), 완료하는 방법이 있습니다.
- 모든 브랜치는 **분류**됩니다: 베이스(이름으로), 어떤 유형의 토픽(패턴으로), 또는 미분류입니다.
- **브랜치의 부모**는 다음 순서로 정해집니다: GitPoulpi가 브랜치를 만들 때 기록한 것(`git config branch.<name>.gitpoulpi-parent`),
  없으면 있을 때 Git Town의 `git-town-branch.<name>.parent`, 없으면 그 유형의 부모입니다.

## 저장되는 위치

| 무엇 | 어디 | 커밋 여부 |
|---|---|---|
| 공유 워크플로 | 저장소의 `.gitpoulpi/workflow.toml` | 예, 직접 커밋합니다. GitPoulpi는 요청할 때만 씁니다 |
| 내 조정 사항 | `.gitpoulpi/repo-state.toml`의 `[workflow]` | 아니요(그 파일은 무시됨) |

상태 디렉터리는 스스로를 무시하지만(`.gitpoulpi/.gitignore`), 커밋하도록 만들어진 `workflow.toml`은 예외입니다.
실제 적용되는 워크플로는 공유 파일 위에 내 재정의를 얹은 것입니다: 같은 이름의 베이스나 같은 id의 토픽은 교체되고,
새 것은 추가되며, `disable_bases`와 `disable_topics`는 공유된 것을 나에게서만 제거합니다.

기존 파일 위에 저장하면 이 버전이 모르는 키(더 새로운 GitPoulpi의 것, 팀원의 것)는 유지되지만, 주석과 키 순서는 유지되지 않습니다.

## 파일

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informative)

[[base]]
name = "main"
names = ["master"]             # other spellings of the same branch
protected = true               # never deleted, direct commits are suspect

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # an environment branch (GitLab Flow): promoted downstream

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # more names that also match; never used to create
# regex = "^feat-(?P<slug>.+)$"  # also matches; named groups become fields
parent = "develop"             # where it starts, and the default finish target
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # opt-in shell commands, always shown before they run
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # host-driven flows, see below
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode`, `hooks`는 아래의 [동작](#동작)을,
`issue_flow`는 [호스트 주도 흐름](#호스트-주도-흐름)을 제어합니다.

### 이름 패턴

`{자리 표시자}`가 들어간 텍스트입니다.

| 자리 표시자 | 일치하는 것 | 만들어지는 방식 |
|---|---|---|
| `{slug}` | 자유 텍스트. 마지막에 오면 `/`를 포함할 수 있음 | 텍스트를 소문자로, 악센트를 제거하고, 단어 사이는 `-`로, 최대 50자 |
| `{issue}` | 숫자(`123`) | `123` 또는 `#123` |
| `{issue_key}` | 트래커 키(`PROJ-123`) | 입력한 그대로 |
| `{user}` | `/`가 없는 이름 | 슬러그로 변환 |
| `{date}` | `2026-10-16` 또는 `20261016` | 입력한 그대로 |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | 입력한 그대로 |
| `{n}` | 숫자 | 입력한 그대로 |

그 밖의 것은 모두 문자 그대로이므로 접두사와 접미사는 그냥 텍스트입니다:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. 각 자리 표시자는 한 번만 나올 수 있고,
자리 표시자 두 개 사이에는 텍스트가 있어야 합니다. 패턴은 유효한 브랜치 이름을 만들어야 합니다(`git check-ref-format`의 규칙):
공백, `~ ^ : ? * [ \`, `..`이 없어야 하고, `.`으로 시작하거나 `.lock`으로 끝나는 구성 요소가 없어야 합니다.

`regex`는 일치 확인에만 쓰입니다(양 끝이 고정됨). 어떤 패턴으로도 표현할 수 없는 이름에 쓰세요.
이름 있는 그룹은 브랜치의 필드가 됩니다.

### 어떤 토픽이 이기는가

1. 베이스처럼 이름 붙은 브랜치(그 이름 중 어느 것이든)는 그 베이스입니다.
2. 그렇지 않으면 일치하는 모든 토픽의 모든 패턴이 후보입니다: `pattern`, 그다음 `patterns`, 그다음 `regex` 순으로,
   토픽은 파일 순서대로입니다. 고정하는 문자 수가 가장 많은 쪽이 이깁니다(`release/{version}`이 `{user}/{slug}`를 이기고,
   그것이 단독 `{slug}`를 이김). regex는 문자 그대로인 접두사를 셉니다. 동점이면 파일에서 앞에 있는 것이 이깁니다.
3. 그 밖에는 미분류입니다.

잘못된 패턴이나 regex는 경고로 보고되고 건너뜁니다. 나머지는 계속 동작합니다.

## 템플릿

| 템플릿 | 베이스 | 토픽 |
|---|---|---|
| `git-flow` | `main` 또는 `master`, `develop` 또는 `dev`(부모는 main) | `feature/`와 `bugfix/`(develop에서), `release/`(develop에서, main과 develop으로, tag 붙임), `hotfix/`(main에서, main과 develop으로, tag 붙임), `support/`(main에서). 복수형 표기(`features/`, `releases/`...)도 일치함 |
| `github-flow` | `main` | `feature/` 또는 아무 이름, main에서, main으로의 merge request, squash, 이후 삭제 |
| `gitlab-flow` | `main`, `pre-production`, `production`(환경 브랜치이며 main, pre-production, production 순으로 연결됨) | `feature/` 또는 아무 이름, main에서. `release/{version}`은 cherry-pick 대상 |
| `trunk` | `main` | main에서 시작하는 수명이 짧은 `{user}/{slug}`, rebase |
| `custom` | 없음 | 없음 |

환경 브랜치는 `preprod`, `staging`, `prod`로도 일치합니다. 템플릿은 출발점입니다:
`flow config init`으로 출력해서 파일을 편집하세요.

## 감지와 가져오기

`gitpoulpi flow detect`는 저장소에 있는 브랜치에 대해 템플릿의 순위를 100점 만점의 점수와, 발견한 실제 이름
(`main` 대신 `master`, `develop` 대신 `dev`, `feature/` 대신 `features/`)과 함께 보여 줍니다. 베이스는 60점,
템플릿의 고유한 패턴이 인식하는 나머지 브랜치의 비율은 40점이며, 다른 템플릿의 오래 유지되는 브랜치
(GitHub Flow 후보에 `develop`이 있는 경우)는 25점을 깎습니다.

`gitpoulpi flow import --from git-flow`는 저장소 자체의 `.git/config`에서 고전적인 git-flow 설정
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`)을 읽고, `--from git-town`은
`git-town.main-branch`와 `git-town.perennial-branches`를 읽습니다(각 브랜치의 부모인
`git-town-branch.<name>.parent`는 기록된 부모로 인식됨). 조용히 적용되는 것은 없습니다:
명령은 TOML을 출력하고, `--write`는 `.gitpoulpi/workflow.toml`이 없을 때만 만듭니다.

## 명령

모두 `--json`을 받으며, MCP 서버는 이를 도구로 제공합니다(`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show`는 읽기 전용이고, 동작은 아래에 설명합니다).

| 명령 | 하는 일 |
|---|---|
| `gitpoulpi flow status` | 모든 로컬 및 리모트 브랜치: 종류(베이스, 토픽, 미분류), 토픽 유형과 필드, 부모와 그 출처(`recorded`, `git_town`, `type`, `base`), 로컬 브랜치는 부모보다 몇 commit `ahead`이고 `behind`인지. 정의에서 발견된 경고, 순위가 매겨진 감지 결과, 가져올 수 있는 다른 도구의 설정도 함께 보여 줍니다. |
| `gitpoulpi flow templates` | 위의 프리셋과 그 전체 정의. 저장소가 필요 없습니다. |
| `gitpoulpi flow detect` | 발견한 베이스와 토픽이 있는 순위별 템플릿. |
| `gitpoulpi flow config [--show-effective]` | 공유 파일(경로, 존재 여부, 파싱된 워크플로, 알 수 없는 키). `--show-effective`를 주면 브랜치를 분류하는 데 쓰는 워크플로. `flow config show`도 같습니다. |
| `gitpoulpi flow config init --template <t> [--write]` | 템플릿의 TOML을 출력합니다. `--write`는 `.gitpoulpi/workflow.toml`을 만들며 기존 파일은 덮어쓰지 않습니다. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | 위에 설명한 대로입니다. |

앞서/뒤처진 개수는 호출당 로컬 브랜치 최대 300개에 대해서만 계산합니다.

읽을 수 없는 워크플로 파일(잘못된 TOML, 타입이 틀린 값)은 파일 이름을 밝히는 오류이며, 조용히 무시되지 않습니다.

## 동작

`start`, `finish`, `sync`, `promote`, `set-parent`는 저장소에 작용합니다. 각각 먼저 **계획**합니다:
수행할 모든 git 단계를 각각에 해당하는 git 명령과 함께, 경고, 되돌릴 수 없는 것, 막는 요인을 나열합니다.
확인하기 전에는 아무것도 바뀌지 않습니다. 실행 한 번은 **실행 취소 기록의 항목 하나**입니다
(`gitpoulpi undo`가 브랜치, tag, 삭제된 브랜치, 기록된 부모, 체크아웃을 함께 되돌립니다).

| 명령 | 하는 일 |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | 유형의 패턴으로 브랜치 이름을 정하고(`flow start feature "Login form"`은 `feature/login-form`을, `flow start release version=1.2.0`은 `release/1.2.0`을 만듭니다. 이름 없는 단어는 `slug`이며, `user`와 `date`의 기본값은 사용자와 오늘입니다), 유형의 부모 또는 `--from`(브랜치, 리모트 브랜치, tag)에서 만들고, 체크아웃하고 부모를 기록합니다. `--publish`는 `-u`로 push합니다(`undo`로 되돌려지지 않음). |
| `gitpoulpi flow finish [<branch>]` | 유형의 `finish` 대상마다 순서대로: 대상을 체크아웃하고 대상의 전략으로 브랜치를 merge합니다(`merge`: 항상 merge commit. `squash`: `--title` 또는 브랜치 이름에서 따온 제목의 commit 하나. `rebase`: 브랜치를 대상 위로 rebase한 뒤 fast-forward. `fast-forward`). 대상에 tag가 있으면 만들고(`tag = "v{version}"`, annotated), 유형이 지정하면 부모를 체크아웃한 뒤 로컬 브랜치를 삭제합니다(`--delete` / `--keep`이 덮어씀. `--delete-remote`는 리모트에서도 삭제하며 되돌릴 수 없음). `finish_mode = "merge-request"`(또는 `--mode merge-request`)이면 브랜치를 push하고 부모로 향하는 pull 또는 merge request를 열기만 하며, `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone`, `--integration`을 받습니다(되돌릴 수 없으며 로컬에서는 아무것도 merge하거나 삭제하지 않음). `--mode local-merge`는 `merge-request` 유형을 덮어쓰고, `ask`로 지정된 유형은 둘 중 하나가 필요합니다. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | 부모의 commit을 브랜치로 가져옵니다: merge이거나, 브랜치의 전략이 `rebase`(또는 `--rebase`)이면 rebase입니다. `--stack`을 주면 부모가 이 브랜치인 브랜치(`set-parent`로 기록됨)도 재귀적으로 순서대로 합니다. 원래 있던 브랜치로 돌아옵니다. |
| `gitpoulpi flow promote <env-branch>` | GitLab Flow 환경 브랜치: 그 브랜치의 상위에 있는 브랜치(`main`을 `pre-production`으로, `pre-production`을 `production`으로)를 한 단계 merge하고 돌아옵니다. 가능하면 fast-forward합니다. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | `branch.<name>.gitpoulpi-parent`를 쓰거나(또는 제거합니다). 순환은 거부합니다. 되돌릴 수 있습니다. |

다른 곳에서 시작한 브랜치는 그곳으로 완료됩니다: 기록된 부모(`--from` 또는 `set-parent`로 지정)가 유형의 부모와
다르면 첫 번째 완료 대상은 기록된 부모입니다.

### 확인, 계획, 스크립트

| 플래그 | 의미 |
|---|---|
| `--dry-run` | 계획을 JSON(`{"plan": ...}`)으로 출력하고 멈춥니다. 막는 요인이 있으면 종료 코드 1. |
| `--yes` | 묻지 않고 실행합니다. |
| 둘 다 없음 | 터미널에서는 계획을 보여 주고 묻습니다. 그 외(스크립트, `--json`, 에이전트)에는 계획이 `{"needs_confirmation": {"plan": ...}}`로 종료 코드 3과 함께 돌아오고 아무것도 바뀌지 않습니다. `--publish`나 훅이 없는 `start`, `set-parent`, `--abort`는 확인이 필요 없습니다. |
| `--run-hooks` / `--no-hooks` | [훅](#훅)을 참고하세요. |
| `--autostash` | 체크아웃이나 merge를 가로막는 로컬 변경을 stash해 두었다가 이후에 되돌려 놓습니다. `general.auto_stash` 설정과 상관없이 적용됩니다. |

막힌 계획은 이유를 밝힙니다: 토픽이 필요한 곳에 베이스 브랜치가 있음, 이미 있는 이름, 빠진 필드(`release/{version}`에는 `version=...`이 필요함),
이미 있는 tag, 리모트보다 뒤처진 대상 브랜치(`--allow-behind`는 그래도 진행), 이미 진행 중인 merge 또는 rebase,
설정되지 않은 워크플로.

**auto-stash** 설정(`general.auto_stash`)이 있으면 체크아웃이나 merge를 가로막는 작업 변경은 일반 체크아웃과 똑같이 동작 앞뒤로 stash되고 복원됩니다.
`never`이면 동작은 아무것도 바꾸기 전에 멈춥니다.

### 충돌

충돌로 멈춘 merge나 rebase는 일반 충돌 상태(충돌 배너, `gitpoulpi conflicts ...`)를 남기고 흐름은 기억됩니다. 종료 코드 2,
`status: "conflicted"`, 경로는 `conflicts`에 담깁니다. 파일을 해결하고 해결됨으로 표시한 뒤:

- `gitpoulpi flow finish --continue`는 merge를 commit하고(또는 rebase를 이어 가거나, squash를 그 메시지와 함께 commit하고) 남은 단계를 실행합니다.
- `gitpoulpi flow finish --abort`는 merge나 rebase를 abort하고, 흐름이 시작된 브랜치로 돌아가며 나머지는 잊습니다. 이미 완료된 단계
  (이중 완료의 첫 대상과 그 tag)는 남으며, `undo`가 되돌립니다.

실패한 훅으로 멈춘 흐름도 같은 방식으로 재개합니다(`--continue`가 그 단계를 다시 실행합니다). 충돌이 난 실행과 그 이어진 실행은 실행 취소 항목 두 개입니다.

### Merge request

daemon이 브랜치를 push합니다. 요청을 여는 데에는 호스팅 토큰이 필요하므로 CLI가 `gitpoulpi hosting pr`와 같은 코드로 처리합니다(리모트와 일치하는 연동,
없으면 호스트의 미리 채워진 웹 페이지를 URL로 출력). 결과에는 `merge_request`(요청한 것)와 `opened`(호스트가 답한 것)가 담기고,
브랜치는 push되었지만 요청을 열 수 없었으면 `open_error`가 담깁니다.

### 훅

워크플로 파일의 `[hooks]`는 셸 명령(`after_start`, `before_finish`)을 나열합니다. 팀이 공유하는 파일에서 오므로 **요청하지 않으면 절대 실행되지 않습니다**:
`--run-hooks`, 또는 나열한 뒤 터미널이 묻는 질문에 예라고 답하는 경우입니다. 계획은 항상 훅을 나열하며,
실행되지 않을 때는 `hooks_run: false`입니다. 훅은 저장소를 작업 디렉터리로 하는 셸에서 실행되며
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH`가 설정됩니다. 정확한 명령, 종료 코드, 전체 출력이 결과에 담깁니다.
0이 아닌 코드로 끝나는 훅은 그 자리에서 흐름을 멈춥니다(`before_finish` 훅은 아무것도 바뀌기 전에 실행되므로 단순한 오류이고,
`after_start` 훅 이후에는 브랜치가 존재하므로 흐름을 계속하거나 abort할 수 있습니다). `--no-hooks`는 묻지 않고 건너뜁니다.
훅은 무엇이든 할 수 있으므로 `undo`는 git 변경만 되돌립니다.

### 에이전트용(MCP)

도구는 `flow_start`, `flow_finish`, `flow_sync`, `flow_promote`, `flow_set_parent`입니다(finish, sync, promote는
destructive로 표시되므로 `allow_mutations`가 제어합니다). 계획 후 실행 방식입니다: `dry_run: true`로 호출해 계획을(일반 결과로) 받고,
`yes: true`로 다시 호출해 실행합니다. 둘 다 없으면 호출은 거부되고 계획이 함께 전달되므로 실수로 실행되는 일이 없습니다.
훅은 명시적인 `run_hooks: true`가 있을 때만 실행됩니다. 계획이 훅을 나열하므로(`hooks`, `hooks_run`) 에이전트가
사용자에게 명령을 먼저 보여 줄 수 있습니다. 충돌 뒤에는 `continue_: true`(플래그는 `--continue`) 또는 `abort: true`를 씁니다.

### 실행 취소

| 동작 | `undo`가 하는 일 |
|---|---|
| start | 브랜치를 삭제하고, 부모를 잊고, 이전 브랜치를 다시 체크아웃합니다. |
| finish | 옮긴 모든 브랜치, tag, 삭제된 브랜치, 체크아웃을 한 번에 복원합니다. push된 브랜치, 리모트 삭제, 열린 merge request는 남습니다(계획이 그렇게 알림). |
| sync | 업데이트된 모든 브랜치를 되돌려 놓으며, 체크아웃된 브랜치는 파일도 포함합니다. |
| promote | 환경 브랜치를 되돌려 놓습니다. |
| set-parent | 이전에 기록된 부모를 복원합니다. |

## 호스트 주도 흐름

이슈가 작업의 시작인 팀을 위한 것입니다: 이슈는 템플릿으로 작성되고, 호스트(GitLab, GitHub, Gitea, Forgejo)나 봇이
브랜치와 merge request를 만듭니다. GitPoulpi는 이슈를 만들고, 호스트가 만드는 것을 기다렸다가 이어받습니다:
리모트 브랜치를 추적하는 로컬 브랜치를 만들고, 그 부모(merge request의 대상)를 기록하고, 체크아웃합니다.
이슈 섹션이 트래커에 연결되어 있어야 하며(`gitpoulpi hosting link issues`), PULL REQUESTS 연결은
브랜치와 요청이 있는 곳을 알려 줍니다(연결되지 않았으면 이슈 프로젝트).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # a bot often triggers on a label
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # a file of the repository
branch_from_issue = "{type}/{issue}-{slug}"   # what the host or bot names the branch
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # how long `wait` polls
merge_request = true                 # host-creates-branch: also a draft request
assignees = []
```

`branch_from_issue`의 `{type}`은 토픽 유형의 id이고, 나머지는 일반적인 [이름 패턴](#이름-패턴)
토큰(`{issue}`, `{slug}`, `{issue_key}`...)입니다. 브랜치가 그 패턴과 일치하고 `{issue}`가 12일 때 그 브랜치는 이슈 12에 속합니다
(`feature/12-login`은 속하고 `feature/120-x`는 속하지 않음).

| 모드 | 이슈를 만든 뒤 |
|---|---|
| `local` | 호스트에는 아무것도 없습니다. `--pick-up`을 주면 이슈 번호와 제목으로 그 유형의 `flow start`를 합니다(다른 start처럼 되돌릴 수 있음). |
| `host-creates-branch` | GitPoulpi가 유형의 부모에서 **호스트에** 브랜치를 만들고(GitLab: 브랜치 API. GitHub: `createLinkedBranch`이므로 이슈의 Development 아래에 표시됨. Gitea/Forgejo: 브랜치 API), `Closes #12`라고 쓴 **초안** 요청을 만듭니다. 빈 요청을 거부하는 호스트(GitHub: 아직 commit이 없음)에서는 브랜치를 남기고 경고로 알립니다. |
| `bot` | 아무것도 하지 않습니다: 봇(웹훅 처리기, CI 작업, GitHub App)이 브랜치와 요청을 만듭니다. `--wait`는 `wait_for`가 충족될 때까지 폴링합니다. |

`wait`가 모든 제공자에 대해 보는 것: 호스트 자체의 링크(GitLab `issues/:iid/related_merge_requests`와 `closed_by`. GitHub 연결된 브랜치와
타임라인의 `connected` / `cross-referenced` pull request), 이름이 `branch_from_issue`와 일치하는 브랜치, 소스 브랜치가 일치하는 열린 요청입니다.
점점 늘어나는 간격(1, 2, 3, 5, 8초, 그다음은 10초마다)으로 `wait_for`가 충족되거나 `timeout_secs`가 끝날 때까지 폴링하며,
응답 뒤에 느리거나 잠시 닿지 않는 호스트는 다시 시도합니다. 호스트가 제공하지 않는 소스(Gitea에는 이슈 링크가 없음)는 건너뛰고,
모든 소스가 실패하면 첫 오류가 답입니다.

| 명령 | 하는 일 |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | 템플릿(`issue_title`, `issue_labels`, 설명 파일, 그것 대신 `--body`)으로 이슈를 만들고 모드에 따라 계속합니다. `--dry-run`은 만들어질 것을(`{"plan": ...}`) 출력하고 아무것도 만들지 않습니다. `--pick-up`은 `bot` 모드에서 먼저 기다립니다. 종료 코드 4: 기다린 시간이 다 됨(이슈는 존재하며 `flow issue wait`가 계속 기다립니다). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | 호스트가 이슈에 대해 만든 것을 폴링하며, 터미널에서는 진행 상황을 stderr에 표시합니다. Ctrl+C로 취소합니다. `--once`는 한 번만 봅니다. 종료 코드 4: 아직 없음. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | fetch하고, `origin/<branch>`를 추적하는 로컬 브랜치를 만들고, 부모를 기록하고, 체크아웃합니다. `#12`는 이슈의 브랜치나 요청을 호스트에서 찾고, `!5`는 요청이며, 그 밖은 브랜치 이름입니다. 부모는 요청의 대상(`--parent`가 덮어씀), 없으면 유형의 부모입니다. `--yes`가 필요 없습니다: `start`처럼 로컬이며 되돌릴 수 있습니다. `--dry-run`도 fetch는 합니다(리모트 추적 ref만 움직이며 계획이 브랜치를 봐야 하기 때문). |

`pick-up`은 실행 취소 기록의 항목 하나입니다: `gitpoulpi undo`는 브랜치를 삭제하고, 부모를 잊고, 이전 브랜치를 다시 체크아웃합니다
(fetch는 모든 fetch처럼 남습니다). 다른 동작처럼 계획됩니다: `--dry-run`은 그 계획(`create_branch`,
`set_upstream`, `checkout`, `record_parent`)을 출력하고 일반적인 차단 요인이 적용됩니다(작업 변경은
auto-stash 설정을 따름). 요청은
`{"action": "pick_up", "branch": ..., "parent": ...}`입니다.

흐름이 만든 이슈는 **감시**됩니다(`.gitpoulpi/repo-state.toml`의 `watched_issues`, 커밋되지 않음):
fetch할 때마다 앱이 리모트 브랜치를 `branch_from_issue`와 대조하여 감시 중인 이슈의 브랜치가 나타나면 알려 줍니다.
이어받으면 정리됩니다.

### 에이전트용(호스트 주도)

도구는 `flow_issue_new`, `flow_issue_wait`, `flow_pick_up`입니다(write, network.
`allow_mutations`가 제어합니다). `dry_run: true`인 `flow_issue_new`는 계획을 돌려줍니다.
`flow_issue_wait`는 최대 `timeout`초 동안 멈추므로 에이전트에는 짧은 값을 주고
다시 호출하세요(`once: true`는 바로 답합니다). 둘 다 `found`(`branches`, `requests`)와 `timed_out`이 있는 JSON을 돌려줍니다.

### 앱에서의 호스트 주도 흐름

워크플로가 있고 트래커가 연결되어 있으면 Flow 메뉴에 *새 <유형> 이슈...*(유형마다 하나.
팔레트 `Flow: 새 <유형> 이슈...`, 그리고 `Flow: 새 이슈...`)와 *호스트가 만든 브랜치 이어받기...*
(팔레트 `Flow: 호스트 브랜치 이어받기...`)가 생깁니다.

- **새 이슈**는 유형(드롭다운)과 제목을 묻고, 설명이 어디서 오는지와 누가 브랜치를 만드는지 보여 줍니다.
  *브랜치가 생기면 이어받기*는 기본적으로 켜져 있습니다. *이슈 만들기*는 이슈를 만든 다음 `bot` 모드에서는 **대기
  카드**를 보여 줍니다: 스피너, 기다리는 대상, 경과 시간과 포기하는 시간, 그리고
  **대기 중단**(`Esc`). 중단해도 이슈는 사라지지 않으며 계속 감시됩니다. 브랜치가
  생기면 이어받고 상태 표시줄이 그렇게 알려 줍니다.
- **브랜치가 생겼습니다**는 fetch로 감시 중인 이슈의 브랜치가 들어왔을 때(다른 대화상자가 열려 있지 않다면) 열리는 카드입니다:
  *이어받기*는 그것을 가져오고, *나중에*는 닫습니다(앱을 다시 시작하기 전까지 다시 제안하지 않음).
- **이어받기**는 `#12`, `!5` 또는 브랜치 이름을 받아 이어받기 전에 찾은 것을 보여 줍니다.

모든 것은 백그라운드에서 실행되며, 150ms보다 오래 걸리는 호출은 스피너를 표시합니다.

### 벤치에서 시험하기

`scripts/fake-gitlab-bot.sh`는 Docker GitLab(`scripts/test-bench.sh`)용 대역 봇입니다: 프로젝트에서 레이블이 붙은 이슈를 감시하다가
`<type>/<iid>-<slug>`(commit 하나 포함)와 이슈를 닫는 초안 요청을 만듭니다.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>`는 같은 것을 오프라인으로 제공합니다
(실제 bare 저장소 위의 GitLab 형태 API와 봇).

## 앱에서

![Git Flow 워크플로가 적용된 사이드바: main과 develop은 베이스 브랜치로 표시되고, feature 브랜치는 Feature 폴더에, 나머지는 기타 아래에 있음](../images/workflow-sidebar.png)

![Feature 시작: 유형, 슬러그, 시작 지점, 만들어질 브랜치 이름](../images/workflow-start.png)

![설정 > 저장소별 설정 > 워크플로: 템플릿, 감지, 베이스 브랜치 편집기](../images/workflow-settings.png)

아래의 모든 것은 워크플로가 있는 저장소(`.gitpoulpi/workflow.toml` 또는 내 재정의)에서만 나타납니다.
없으면 아무것도 바뀌지 않습니다: 사이드바는 평범한 트리이고 `Flow` 버튼도 없습니다.

### 설정 > 워크플로

*설정 > 저장소별 설정 > 워크플로*는 정의를 편집합니다. 초안으로 작업하며 저장 버튼을 누르기 전에는
아무것도 저장하지 않습니다.

- **템플릿**은 프리셋 드롭다운입니다. 하나를 고르면 아래의 베이스와
  토픽 유형이 프리셋으로 교체됩니다(훅과 이슈 흐름은 유지됨).
- **브랜치에서 감지**는 이 저장소의 브랜치 이름을 읽고 가장 가까운
  템플릿을 점수와 함께, 찾은 베이스 이름(`main` 대신 `master`),
  각 토픽 유형이 인식하는 브랜치 수와 예시, 모든 템플릿의 점수와 함께 보여 줍니다.
  **감지한 워크플로 사용**은 그것을 초안에 불러옵니다.
- 저장소에 다른 도구의 설정(Git Flow의 `gitflow.*`,
  Git Town의 `git-town.*`)이 있으면 **가져오기**가 제공되며 가져온
  정의를 초안에 불러옵니다. 누르기 전에는 아무것도 적용되지 않습니다.
- **베이스 브랜치**, **토픽 유형**과 그 **완료 규칙**은 **규칙** 탭에서 그룹으로 나뉜
  [목록](settings.md#목록)이며, **일반** 탭에 나머지(템플릿, 감지, 훅, 이슈 흐름)가 있습니다.
  항목을 고르면 오른쪽에서 편집합니다(이름과 다른 이름, 부모, 보호됨, merge 전략, merge 시 tag, 환경
  브랜치. 레이블, 이름 패턴, 다른 패턴, 정규식, 부모, 완료
  모드, 완료 후 삭제). **+ 추가 ▾**로 추가할 종류를 고르며(완료 규칙은 선택한 토픽 유형에 속합니다),
  추가, 복제, 제거, 드래그는 다른 모든 설정 목록과 같고 항목은 자기 그룹 안에서 이동합니다.
  **이름 패턴** 행은 토큰(`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`)을 나열합니다. 그 아래의 실시간 미리 보기는 패턴이 일치하는 기존 브랜치와
  만들어질 이름을 보여 주며, 파싱되지 않는 패턴은 필드 바로 아래에
  빨간색으로 이유를 알려 줍니다(입력한 것은 사라지지 않음).
- **훅**은 한 줄에 하나씩인 셸 명령(`시작 후`, `완료 전`)이며
  여러 줄 편집기에서 편집합니다(`Ctrl+S` 저장, `Esc` 취소). 내 컴퓨터에서
  명령을 실행하는 것이므로 페이지가 그렇게 알리고, 대화상자도 실행 전에 그 명령을
  보여 줍니다.
- **이슈 흐름**은 [호스트 주도 흐름](#호스트-주도-흐름)이 쓰는 필드
  (모드, 이슈 제목과 레이블, 브랜치 패턴, 기다릴 대상)를 담으며 워크플로 파일에 저장됩니다.
- **문제**는 정의가 잘못된 곳(존재하지 않는 부모, 깨진 패턴)을 나열합니다.
- 페이지 맨 아래에 고정된 막대가 **● 저장 안 됨**(또는 **변경 사항 없음**)을 표시합니다.
  **공유 파일에 저장**은 초안을 `.gitpoulpi/workflow.toml`에 쓰며(커밋하세요),
  이 버전이 모르는 키는 유지하고 내 재정의를 거기에 합칩니다. **나에게만**은 공유 파일과의 차이만
  내 저장소 상태(`.gitpoulpi/repo-state.toml`, 커밋되지 않음)에 보관합니다. **변경 사항 버리기**는
  저장된 것을 다시 읽습니다. 저장하지 않은 초안이 있을 때 설정을 닫으면 먼저 묻습니다.

### 사이드바

LOCAL과 REMOTE는 워크플로에 따라 묶입니다: 베이스 브랜치가 먼저 나오며 각각 `[베이스]` 배지가 붙고(보호되면 `[베이스, 보호됨]`),
그다음 토픽 유형마다 레이블 이름의 폴더가 브랜치 수와 함께 나오고(`Feature (3)`),
어떤 유형도 가져가지 않은 브랜치는 `기타` 아래에 나옵니다. 폴더는 다른 폴더처럼 접히며,
평범한 트리는 *Flow > 브랜치를 유형별로 묶기*(Flow 메뉴 행) 또는
팔레트 명령 `Flow: 브랜치 묶기 전환`으로 돌아옵니다. 브랜치에 마우스를 올리면 그 유형,
부모, 부모보다 얼마나 뒤처졌는지(그리고 앞섰는지)를 보여 줍니다.

### Flow 메뉴와 그 대화상자

도구 모음의 **Flow** 버튼(`Ctrl+G Shift+W`, 팔레트 `Flow: 메뉴 열기`)은 모든 토픽 유형의
*<유형> 시작...*, *현재 브랜치 완료...*,
*부모와 동기화...*, *승격...*(GitLab Flow 환경 브랜치), *부모
설정...*, 묶기 전환, *워크플로 설정...*을 나열합니다. 팔레트에도
같은 명령이 있습니다(`Flow: Feature 시작...`은 유형마다 생성됨).

- **시작**은 유형의 이름 패턴으로 만든 양식으로, 토큰마다 필드가 있습니다. `{issue}`
  와 `{issue_key}`에는 연결된 트래커의 이슈 드롭다운이 딸려 있고, 이것은 이슈 제목으로 `{slug}`도
  채웁니다. `{version}`은 가장 높은 release tag 다음 버전을 제안하고, `{user}`는 활성 프로필이며, `{date}`는 오늘입니다.
  한 줄이 패턴이 만드는 브랜치 이름을(또는 만들 수 없는 이유를: 잘못된 ref, 이미 있는
  이름) 보여 줍니다. 시작 지점은 드롭다운이며(유형의 부모가 먼저), *게시*는
  새 브랜치를 push합니다.
- **완료**, **동기화**, **승격**, **부모 설정**은 필요한 것을 묻고,
  daemon이 계산한 **미리 보기**(`gitpoulpi flow ... --dry-run`이
  출력하는 것과 같은 계획)를 보여 줍니다: 모든 단계를 순서대로, 경고(리모트보다 뒤처진 대상, 이 브랜치 위에 쌓인 브랜치),
  그리고 실행 버튼을 끄고 이유를 알려 주는 **차단 요인**(존재하지 않는 대상, 리모트보다
  뒤처진 대상)입니다. 되돌릴 수 없는 단계(push, 리모트 브랜치 삭제, merge request)는 `!`로 표시되고 색이 입혀지며,
  "실행 취소로 되돌릴 수 없는 것: ..." 줄이 붙습니다. 실행될 훅은 경고 아래에
  나열되고 *훅 실행* 체크박스(기본값은 꺼짐)가 있습니다. 유형의 완료
  모드가 `ask`이면 완료는 먼저 *완료 방식*을 묻습니다: 로컬에서 merge하거나, pull 또는
  merge request를 엽니다.
- 그다음 동작은 단계마다 표시와 함께 실행되고(150ms 뒤 스피너) 대화상자가
  각 단계가 어떻게 되었는지 보여 줍니다. 동작 전체가 **실행 취소 항목 하나**입니다: `Ctrl+Z`가
  시작, 완료, 동기화, 승격을 한꺼번에 되돌립니다(실행 취소 기록에는
  `Flow finish feature/login`으로 표시). push, 리모트 삭제, merge request는 미리 보기가 알린 대로 남습니다.
- **충돌**은 동작을 멈추고 **계속**과 **중단**이 나옵니다. 대화상자를 닫고 일반 충돌 도구로 충돌을
  해결한 다음, 팔레트 명령 `Flow: 중단된 동작 계속 또는 중단...`(동작이 멈춘 동안은 Flow
  메뉴의 행이기도 함)으로 다시 열 수 있습니다. 계속은 아직 충돌 중인 파일이 있으면 거부하고
  두 버튼을 그대로 둡니다. 중단은 동작이 시작된 브랜치로 돌아가며, 이미
  끝난 단계는 남습니다(실행 취소가 되돌림).
- **merge request**로 완료하면 브랜치를 push한 다음 일반적인 *Pull Request 만들기*
  양식(GitLab에서는 *Merge Request 만들기*)을 미리 채워서 엽니다: 대상은 브랜치의 부모이고, 제목은
  브랜치(와 이슈)에서 옵니다. 메타데이터 버튼(담당자, 리뷰어, 레이블, 마일스톤)은
  다른 pull request에서처럼 동작합니다. 리모트에 답하는 연동이 없으면
  대신 비교 URL이 브라우저에서 열립니다(그리고 상태 표시줄에 표시됩니다).
- 단계와 차단 요인의 텍스트는 CLI처럼 영어이고, 대화상자 자체의 텍스트는
  번역됩니다. 변경이 있는 작업 트리는 `auto-stash` 설정을 따릅니다.

이 모든 것은 키보드와 팔레트로 접근할 수 있으며, 모든
드롭다운은 콤보박스입니다(Enter로 열고, 입력으로 필터링하고, Enter로 고름).

호스트 주도 행(*새 <유형> 이슈...*, *호스트가 만든 브랜치 이어받기...*)은
위의 [앱에서의 호스트 주도 흐름](#앱에서의-호스트-주도-흐름)에 설명되어 있습니다.
