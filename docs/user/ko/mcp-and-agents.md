[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · **한국어** · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP와 에이전트

AI 에이전트(Claude Code, Cursor, Codex 등)는 **MCP 서버** 또는 **CLI**로 GitPoulpi를
조작할 수 있습니다. 둘 다 창과 같은 백그라운드 데몬을 거치므로, 에이전트가 하는 일이
열려 있는 GitPoulpi에 실시간으로 나타나고, 두 방식 모두 각 명령에 대해 정확히 같은
코드를 실행합니다.

이 페이지는 *외부* 에이전트에 관한 것입니다. GitPoulpi 자체의 AI 버튼(commit 메시지,
설명, commit 작성기)은 따로 설정합니다. [AI 지원](ai.md)을 참고하세요.

## 빠른 설정

1. `gitpoulpi`와 `gitpoulpi-mcp`를 `PATH`에 둡니다([설치](installation.md)).
2. 에이전트에 서버를 등록합니다. 서버는 stdio로 통신하므로 따로 시작할 것도, 포트도
   없습니다.
3. 필요하면 `config.toml`의 `[mcp]` 테이블(아래)에서 에이전트가 할 수 있는 일을
   선택합니다. 기본값은 에이전트가 저장소를 읽게 하되(`allow_mutations = true`로 설정하기
   전에는 변경이 거부됨), 무언가를 보여 주는 것은 내가 확인한 뒤에만 허용하고, 대화상자를
   미리 채우는 일은 허용하지 않습니다.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project`는 대신 저장소의 `.mcp.json`에 기록하므로, 프로젝트를 여는 모든 사람이
받게 됩니다. 같은 파일을 직접 작성하고 선택 사항을 더한 예:

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off`는 이 에이전트에 한해 `[mcp] ui_control`을 덮어씁니다.
- `GITPOULPI_CONFIG_DIR`은 에이전트에게 별도의 `config.toml`을 줍니다(자체 `[mcp]` 제한이
  있고 내 설정에는 접근하지 못함). 생략하면 내 설정을 공유합니다.
- `gitpoulpi --repo <path> mcp serve --stdio`는 서버를 저장소 하나에 고정합니다. 그렇지
  않으면 도구는 서버의 작업 디렉터리 주변 저장소나 호출의 `repo` 인자를 대상으로
  동작합니다.

**다른 클라이언트**: stdio 서버를 실행하는 모든 MCP 클라이언트.

| 클라이언트 | 위치 | 내용 |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | 클라이언트의 MCP JSON 파일 | 위의 `mcpServers` 블록 (VS Code는 키 이름이 `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| 그 밖의 모든 것 | 해당 문서 | 명령 `gitpoulpi-mcp` (독립 실행 바이너리, 같은 서버, 인자 불필요) |

**HTTP로** (URL에 연결하는 클라이언트):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

상태 없는 JSON, 같은 도구, 같은 `[mcp]` 게이트입니다. 모든 요청에 bearer 토큰이
필요합니다(`[mcp] http_auth`, 기본값 켜짐). `gitpoulpi mcp token --show`로 출력하고
(`--copy`, `--rotate`도 있으며, 토큰은 비공개 데이터 폴더 아래 사용자 전용 파일에
있습니다) 클라이언트에 헤더로 전달하세요.

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

토큰이 없거나 틀리면 `WWW-Authenticate: Bearer`와 함께 `401`을 받습니다. 교체한 토큰은
서버를 재시작하지 않아도 즉시 적용됩니다. `http_auth`를 끄는 것은 모든 로컬 계정을
신뢰하는 루프백 엔드포인트를 위한 것이며, 이때 서버는 stderr에 경고합니다.

### 다른 컴퓨터에서 엔드포인트에 접근하기

가장 안전한 구성은 원격 수신기가 아예 필요 없습니다. 루프백 주소를 유지하고 에이전트를
실행하는 컴퓨터에서 `ssh -L 8808:127.0.0.1:8808 host`로 SSH를 통해 포트를 전달하세요.
그러면 `--allow-remote`는 관여하지 않습니다.

`--allow-remote`(루프백이 아닌 주소)는 인증이 켜져 있고 토큰이 있으며 **그리고** 트래픽이
암호화되어 있지 않으면 거부됩니다. 거부 메시지는 무엇이 빠졌는지 알려 줍니다.
암호화하는 방법은 두 가지입니다.

- **서버의 TLS**(`[mcp] tls = "self_signed"` 또는 `"files"`). 서버가 직접 TLS를
  종료합니다. 키를 가지고 핸드셰이크를 수행하며, 완료된 핸드셰이크만 HTTP 계층에
  도달합니다. 평문 HTTP 클라이언트는 핸드셰이크에 실패하며 그 옆에 평문 수신기는
  존재하지 않습니다. 따라서 암호화는 가정이 아니라 서버가 모든 연결에서 확립하는
  사실입니다. `self_signed`에서는 처음 시작할 때 비공개 데이터 폴더 아래에 키 쌍을
  생성하고 SHA-256 지문을 출력합니다(설정에도 표시). 클라이언트는 그 지문이나
  인증서 파일을 고정하여 서버를 신뢰합니다. `files`에서는 `tls_cert`와 `tls_key`를
  PEM 파일로 지정하며, 클라이언트는 CA로 인증서를 검증합니다. 클라이언트는 bearer
  토큰으로 인증하며 클라이언트 인증서는 사용하지 않습니다.
- **터널 또는 리버스 프록시**(`[mcp] behind_proxy = true`). 터널 에이전트나 프록시는
  다른 로컬 또는 LAN 클라이언트처럼 연결하며 그 헤더는 위조될 수 있으므로, 서버는
  안에서 그것이 있는지 알 수 없습니다. 그래서 감지가 아니라 명시적인 확인입니다(설정에서
  빨간 경고와 함께 표시). 가능하면 같은 컴퓨터에서 터널을 실행하고 루프백 주소를
  유지하세요. 프록시 뒤에서 루프백이 아닌 주소를 쓰면 프록시와 서버 사이 구간은 해당
  네트워크에서 평문입니다.

**MCP 없이** 셸이 있는 에이전트는 CLI를 사용합니다. `gitpoulpi --json <command>`
([CLI 참조](cli-reference.md) 참고). 에이전트에게 준비된 스킬
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md)를 주세요. 원시 `git`
대신 GitPoulpi를 언제, 어떻게 쓸지 알려 줍니다.

## 권한

| 스위치 | 기본값 | 효과 |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp`가 시작을 거부합니다(설정 > 에이전트 및 MCP > *MCP 서버*). 서버가 시작될 때 읽으므로 실행 중인 서버는 기존 설정을 유지합니다. |
| `[mcp] bind` | `"127.0.0.1:8808"` | 주소 없이 `--http`를 쓸 때의 주소 (`--allow-remote`가 없으면 루프백만. `--allow-remote`에는 인증과 암호화가 필요합니다. 위 참고). 설정 > 에이전트 및 MCP > *MCP 서버 포트*가 포트를 편집하고 호스트는 유지합니다. |
| `[mcp] http_auth` | `true` | `--http`는 `Authorization: Bearer <token>`(`gitpoulpi mcp token`)이 필요합니다. 설정 > 에이전트 및 MCP > *HTTP 인증*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` 또는 `files`(PEM 경로): 서버가 직접 TLS를 종료합니다. 설정 > 에이전트 및 MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | 터널이나 리버스 프록시가 트래픽을 암호화한다고 사용자가 보증하며, 이로써 `--allow-remote`를 TLS 없이 실행할 수 있습니다. |
| `[mcp] allow_mutations` | `false` | 꺼짐: 무언가를 바꾸는 모든 도구가 거부됩니다. 읽기, `ui_*`, 제안은 계속 동작합니다. `true`이면 에이전트가 commit, push, reset 등을 할 수 있습니다. |
| `[mcp] mutation_allowlist` | `[]` | 비어 있지 않으면: 이 도구만 변경할 수 있습니다. 예: `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: 에이전트가 내 창을 움직일 수 없습니다(`ui_state` / `ui_viewers`는 계속 응답). |
| `[mcp] ui_control` | `"ask"` | `ask`: 내가 확인하는 칩이 창에 나타남. `auto`: 내가 바쁘지 않으면 적용. `off`: 거부. 에이전트별: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | 에이전트가 내가 확인하는, 미리 채워진 대화상자를 열 수 있게 합니다 (설정 > 에이전트 및 MCP > *AI 에이전트가 변경 사항을 제안하고 내가 확인하도록 허용*). |
| `[mcp] allow_sensitive_staging` | `false` | `config.toml`에서만 설정합니다. 꺼져 있으면 `stage` 도구는 `allow_sensitive`를 제공하지 않으며, 민감한 파일(개인 키, `.env`, 인식된 토큰)을 추가하게 될 스테이징은 `{error: "sensitive_files", files}`와 함께 거부됩니다. 에이전트는 사용자에게 물어봐야 합니다. `true`이면 `allow_sensitive`를 전달할 수 있습니다. [스테이징과 commit](staging-and-committing.md#민감한-파일)을 참고하세요. |
| `[mcp] allow_history_purge` | `false` | `config.toml`에서만 설정합니다. `true`이면 기록 다시 쓰기의 이전 기록(과 그 실행 취소)을 완전히 삭제하는 도구 `rewrite_purge`가 나열됩니다. 꺼져 있으면 에이전트에게 보이지 않습니다. `allow_mutations`도 여전히 필요합니다. |
| `[ui] allow_agent_view_control` | `true` | 창의 마스터 스위치 (설정 > 에이전트 및 MCP > *AI 에이전트의 화면 제어 허용*). 꺼짐: 어떤 에이전트도 내 창에 닿을 수 없습니다. |
| `[ui] launch` | `"off"` | 열린 창이 없을 때 에이전트가 GitPoulpi 창을 열 수 있는지 여부 ([아래](#열린-창이-없거나-다른-세션에-있을-때) 참고). |

실제로는 다음과 같습니다.

- **기본 상태에서 에이전트는 읽기 전용입니다.** 읽을 수 있고, 내 창에 무언가를 보여 줄
  수 있으며(`ui_*`), 허용하면 내가 확인할 변경을 제안할 수 있습니다. 저장소를 바꾸게
  하려면(commit, push, reset 등) `allow_mutations = true`로 설정하세요. 좁은 권한만
  주려면 `true`로 두고 `mutation_allowlist`를 함께 사용하세요. 모든 도구에는 MCP
  어노테이션(`readOnlyHint`, `destructiveHint`, `openWorldHint`)이 붙어 있으므로, Claude
  Code 같은 클라이언트는 위험한 것에 대해 여전히 먼저 묻습니다.
- **충돌**: `conflicts_check`(merge 전의 예측), `conflicts_list`, `conflicts_show`, `conflicts_state`는 읽기입니다.
  `conflicts_resolve`(유지하지 않은 쪽을 버리며, `--delete`는 파일을 삭제함)와
  `conflicts_abort`(merge/rebase/cherry-pick 전체를 버림)는 파괴적이고,
  `conflicts_unresolve`는 일반 쓰기입니다. 거부된 호출(`stale`, `not_conflicted`,
  `pick_incomplete`...)은 실패하는 대신 종료 코드 2와 함께 `{"refused": {...}}`로
  응답합니다.
- **멈춘 상태**: `conflicts_am_continue`는 쓰기이고, `conflicts_am_skip`,
  `conflicts_am_abort`, `index_lock_remove`는 파괴적이며, `index_lock_show`는 읽기입니다.
- **기록 다시 쓰기**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze`(크기 분석), `rewrite_scan_secrets`(가린 미리보기가 있는 발견 항목, 값은 절대 없음),
  `rewrite_script_check`, `rewrite_script_test`, `rewrite_discard`는 읽기입니다
  (미리보기는 아무것도 바꾸지 않고 계획만 데몬과 git 디렉터리에 보관하므로
  `rewrite_apply`는 재시작 뒤에도 동작합니다). `rewrite_apply`와 `rewrite_run`은
  파괴적입니다. 많은 ref를 다시 쓴 commit으로 실행 취소 항목 하나로 옮기며
  (`undo`로 되돌림) push는 하지 않습니다. 레시피는 `recipe_json`(또는 경로인
  `recipe`)으로, 비밀 값은 `secrets_file`로 넘기고 인라인으로 넣지 마세요.
  셸, Python, 외부 Lua 스크립트(`script_language`)는 샌드박스가 없으며
  **에이전트가 신뢰할 수 없습니다**. `rewrite_script_trust` 도구도 `trust` 인자도
  없으므로, 이런 스크립트는 사람이 그 텍스트에 `gitpoulpi rewrite script trust FILE`을
  실행한 뒤에만 실행됩니다 (스크립트가 바뀌면 다시 거부되며 `script_not_trusted`와
  `refused.script`의 해시가 나옴). 또한 `rewrite.external_scripts`가 그 언어를 켠
  경우에만 실행됩니다 (`scripts_disabled`).
  거부되면 `{"refused": {"reason", "message", "detail"}}`로 답하고, 계획이 이미
  push된 ref를 옮기면 호출이 `yes`를 설정할 때까지
  `{"needs_confirmation": ...}`(종료 코드 3)로 답합니다. 먼저 사용자에게
  보여 주세요.
- **에이전트가 절대 닿을 수 없는 것**: 토큰 저장(`hosting login`), SSH 키 생성과 업로드,
  `config set`(자신의 제한을 다시 쓸 수 있음), 데몬 수명 주기. 토큰은 에이전트에게 절대
  표시되지 않습니다.
- **계정 전체 목록**: `hosting_radar`는 읽기입니다(프로필의 모든 저장소에 걸쳐 내 리뷰를 기다리는 요청, 열린 요청, 할당된 이슈를 `refresh`가 없으면 5분 캐시에서 가져옴). `hosting_integrations_radar`는 프로필의 `[integrations.radar]` 테이블을 바꾸므로 `allow_mutations`가 제어합니다(설정만 출력하는 옵션 없는 호출도 마찬가지).
- **리뷰 컨텍스트**: `ai_review_context`는 코드 리뷰 초안에 diff 외에 무엇이 전달되는지(CLAUDE.md, AGENTS.md, 추가 파일, 로컬 에이전트가 실행되는 곳)를 보여 줍니다. 설정 인자를 주면 기본값(전역, 또는 `repo`나 `project`의 고유 값)을 저장하므로, 설정을 바꾸는 다른 도구처럼 `allow_mutations`가 제어하고, 인자가 없으면 출력만 합니다. 모든 `ai_*` 도구처럼 `ai.enabled`가 켜져 있을 때만 목록에 나타납니다. `hosting_review_suggest`는 같은 인자를 받지만 한 번의 실행에만 적용됩니다.
- **`[mcp]` 게이트는 MCP 서버에 적용됩니다.** CLI는 내 자신의 셸이므로
  `allow_mutations`의 제약을 받지 않습니다. `gitpoulpi ui ...`에는 `ui_control`과
  `allow_ui_propose`만 함께 적용됩니다.
- 거부된 호출은 일반적인 도구 오류(`isError: true`, 이유는 `structuredContent.error`)이며,
  조용히 아무 일도 하지 않는 일은 없습니다.

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_sensitive_staging = false  # true: the agent may stage files the staging guard flags
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## 도구

CLI 명령마다 도구가 하나씩 있으며, 이름은 경로를 `_`로 이은 것입니다(`branch create`는
`branch_create`). 각 도구는 명령의 인자에 더해 `repo`(저장소. 기본값은 서버의 저장소)와
`session`을 받습니다. 성공한 호출의 `structuredContent`는 `gitpoulpi --json <command>`가
출력하는 것과 정확히 같습니다. **종류**: *read*는 아무것도 바꾸지 않습니다. *write*와
*destructive*는 변경 작업입니다(게이트 적용). *network*는 리모트나 호스트와 통신합니다.
*view*는 내 창만 움직입니다. *propose*는 내가 확인하는 대화상자를 엽니다.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_list` | `gitpoulpi worktree list` | read |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### 브랜치 워크플로 동작은 계획 후 실행입니다

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote`, `flow_set_parent`
([브랜치 워크플로](branching-workflows.md#동작))는 단순 호출로는 절대 실행되지 않습니다.
`dry_run: true`는 계획(모든 git 단계와 명령, 경고, 실행 취소가 되돌릴 수 없는 것,
차단 요인)을 일반 결과로 돌려주고, `yes: true`는 그것을 실행하며,
둘 다 없으면 호출이 거부되고 계획이 함께 전달됩니다. 워크플로 파일의 `[hooks]`는
팀이 공유하는 파일의 셸 명령이므로 명시적인 `run_hooks: true`가 있을 때만 실행됩니다.
계획이 훅을 나열하므로(`hooks`, `hooks_run`) 사용자에게 정확한 명령을 먼저 보여 줄 수 있습니다.
충돌로 멈춘 merge는 `status: "conflicted"`(오류 결과, 종료 코드 2)로 응답합니다.
충돌 도구로 파일을 해결한 뒤 같은 도구를 `continue_: true` 또는 `abort: true`로 호출하세요.
merge request 열기(`merge-request` 모드의 `finish`)는 실행 취소할 수 없으며 계획에 그렇게 표시됩니다.

### 에이전트가 일반 `git`으로 한 일도 실행 취소 기록에 남습니다

에이전트가 이 도구를 거치지 않고 자신의 `git`으로 브랜치와 워크트리를 만들거나 commit, reset, stash drop을 해도 그 일은 보입니다. 데몬이 저장소의 ref를 마지막으로 본 것과 비교하여 각 변경을 `undo --list` 항목으로 기록하며, 여기에 `"source": "outside"`, git이 남긴 이유(`outside.message`), git이 기록한 committer가 붙습니다. `undo`, `redo`, `undo_restore` 도구는 이 항목도 다른 항목처럼 다루므로 에이전트(또는 `Ctrl+Z`를 누른 사용자)가 삭제된 브랜치를 되살릴 수 있습니다. 파일 편집은 기록하지 않습니다. [실행 취소와 다시 실행](undo-redo.md#gitpoulpi-밖에서-한-변경)을 참고하세요.

## 사용자에게 보여 주기

에이전트는 설명하는 대신 무언가를 가리킬 수 있습니다. `ui_show_commit`, `ui_show_diff`
(줄 위치의 파일, 인라인 또는 분할), `ui_show_file`(줄 위치의 내용, blame 또는 history),
`ui_show_ref`, `ui_show_pr`, `ui_show_issue`, `ui_show_conflict`(충돌한 파일의 merge
편집기를 충돌 위치에서), `ui_show_wip`(`composer`는 아무것도 입력하지 않고 커서를 commit
요약에 놓음), `ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette`(입력한
쿼리만 보여 주고 실행하지 않음), `ui_show_keys`, `ui_show_settings`(페이지와 설정을
보여 줄 뿐 편집하지 않음), `ui_highlight`(열린 diff나 파일의 줄에 색을 입힘),
`ui_notify`(토스트), `ui_back`(에이전트의 마지막 보기 변경을 되돌림)가 있습니다.
`ui_state`는 창이 표시하는 것을 읽습니다(`text: true`는 보이는 텍스트를 추가).
`ui_viewers`는 창을 나열합니다.

이 도구들은 **저장소를 절대 바꾸지 않으며** 입력, 클릭, 설정 편집을 할 수 없습니다.
리비전과 경로는 창에 닿기 전에 검증됩니다.

**사용자가 보고 결정하는 것**:

- `ask`(기본값)일 때, 또는 입력 중이거나, 대화상자나 설정이 열려 있거나, 드래그 중이거나,
  최근 1.5초 안에 키를 눌렀을 때: 에이전트는 즉시 `deferred`를 받고 상태 표시줄에 *"claude-code가
  ...을(를) 표시하려 함"* 칩이 나타납니다. `Ctrl+N Enter` 또는 클릭하면 표시하고,
  `Esc` 또는 `Ctrl+N Backspace`는 닫습니다. 60초 후 만료됩니다.
- 적용되면: 보기가 이동하고, 4초 동안 토스트가 누가 움직였는지 알려 주며, 다음 키를
  누를 때까지 탭에 보라색 `✦`가 붙습니다. `Ctrl+N Backspace`(팔레트 `에이전트: 뒤로`)로
  되돌아갑니다. 창은 보기 열 개를 기억합니다.
- 숨겨진 것을 드러내거나 체크아웃하는 일은 절대 없습니다. 솔로 필터가 숨기는 commit은
  `partial`(`hidden_by_filter`)로 응답하고, 제공자에 연결되지 않은 PR이나 이슈 목록은
  `hosting_not_linked`로 응답합니다.

**응답**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`이며, 적용되었을 때는 창의 `state`가
함께 옵니다. CLI 종료 코드: applied/partial/proposed는 0, deferred는 3, 그 밖은 1입니다.
호출은 속도가 제한됩니다(5회까지 몰아서 가능, 그 뒤로는 초당 1회).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### 변경 제안하기 (`ui_propose_*`)

`allow_ui_propose = true`이면 에이전트가 내 대화상자 중 하나를 **미리 채워서** 열 수
있으며, *✦ <agent>의 제안*으로 표시됩니다. 그 대화상자 자체의 확인을 누르기 전에는
아무 일도 일어나지 않습니다. 먼저 편집하거나 취소할 수 있습니다.

| 도구 | 여는 것 | 확인하는 방법 |
|---|---|---|
| `ui_propose_reword` (rev, message) | Reword 팝업 | *메시지 업데이트* (push된 commit은 force push 여부를 여전히 물음) |
| `ui_propose_commit_message` (summary, body) | 채워진 commit 양식. commit은 절대 하지 않음 | commit 버튼 |
| `ui_propose_branch` (name, start point) | 인라인 새 브랜치 상자 | `Enter` |
| `ui_propose_tag` (name, target, message) | 인라인 새 Tag 상자 (메시지가 있으면 주석 tag) | `Enter` |
| `ui_propose_split` (rev, plan file) | commit 위에 열리는 분할 편집기. 계획이 미리 채워져 있음 | *분할* (`Ctrl+S`). `Esc`는 거부 |
| `ui_propose_rewrite` (recipe file) | 레시피의 규칙이 들어 있는 기록 다시 쓰기 워크벤치. 아무것도 미리 보거나 적용되지 않음 | 내가 직접 *미리보기*한 뒤 *적용*(`Ctrl+Enter`). `Esc`는 거부. 스크립트가 있는 레시피는 거부되고 비밀 값은 제거됨 |
| `ui_propose_resolution` (path, pick or content file) | 충돌한 파일의 merge 편집기. 선택(또는 전체 텍스트)이 이미 출력에 들어 있음 | *저장 및 해결됨으로 표시* (`Ctrl+S`). 닫으면 거부 |

한 번에 제안 하나만 가능합니다. commit 메시지 제안은 작성기에 내 초안이 있는 동안 거부되고,
해결 제안은 merge 편집기에 내 저장되지 않은 작업이 있거나 에이전트가 읽은 뒤 파일의
스테이지가 바뀐 경우 거부됩니다. 확인하기 전에 내가 바꾼 해결은 `accepted_edited`로
보고됩니다. 응답에는 `proposal` id가 있으며, 내가 결정한 내용은 `ui_state`의
`proposals`에 있습니다(`offered`, `open`, `accepted`, `accepted_edited`, `rejected`,
`expired`, `failed`).

### 열린 창이 없거나 다른 세션에 있을 때

- 창은 시작된 세션에 속합니다. 에이전트의 세션에 창이 없으면, `ui show`는 그 저장소를 연
  다른 세션의 창 **하나**를 찾아 사용합니다(응답의 `routed_session`). 여러 개가 일치하면
  이를 나열한 `no_viewer`가 오며, `--session`을 지정하세요.
- **에이전트를 위해 창 열기**는 기본적으로 꺼져 있습니다. `[ui] launch`(설정 > 에이전트 및
  MCP > *에이전트용 창 열기*)를 터미널이나 멀티플렉서 프리셋 또는 `custom` 명령으로
  설정하세요. 그러면 `launch: true`인 호출(CLI `gitpoulpi ui --launch show ...`)이
  창을 열고 최대 10초 기다립니다. `ui_control = "auto"`이면 플래그 없이도 열리고, `ask`이면
  에이전트가 `no_viewer`를 받아 사용자에게 알려야 합니다.

| 프리셋 | 명령 |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (기본 터미널) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (새 창), `zellij` (새 pane), `cmux` (새 워크스페이스) | 실행 중인 멀티플렉서 안에서, 저장소 폴더에서 |
| `custom` | `[ui] launch_command`. 예: `my-term --cd {repo} {gitpoulpi} --session {session}`. 인자로 분리되며 셸은 사용하지 않음 |

프리셋이 사용 중인 프로그램 버전에서 동작하지 않으면 `custom`을 사용하세요.

## 오류

- 잘못된 호출(알 수 없는 도구나 인자, 필수 인자 누락)은 JSON-RPC `-32602 invalid_params`
  오류입니다.
- 거부, 또는 git이나 데몬 오류(잘못된 리비전, 충돌 등)는 도구 오류입니다.
  `isError: true`이며 메시지는 `content`와 `structuredContent.error`에 있습니다.
