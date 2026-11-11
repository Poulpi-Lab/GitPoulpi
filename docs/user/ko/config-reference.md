[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · **한국어** · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# 설정 참조

`config.toml`의 모든 키와 기본값, 그리고 그 역할입니다. 파일을 직접 편집해야 하는 일은
드뭅니다. [설정 화면](settings.md)(`Ctrl+,`)이 값을 바꿀 때마다 파일에 기록하며,
`gitpoulpi config set`도 마찬가지입니다.

## 위치와 값이 정해지는 방식

| | |
|---|---|
| 파일 | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path`가 경로를 출력합니다. |
| 다른 폴더 | `GITPOULPI_CONFIG_DIR=<dir>`은 설정 폴더 전체(파일, `sessions/`, 데몬 포트 파일)를 옮깁니다. 포터블 설치, 테스트, 또는 에이전트를 자신의 설정과 분리해 두는 용도입니다. |
| 우선순위 | 내장 기본값, 그다음 `config.toml`, 그다음 `GITPOULPI_*` 변수, 그다음 명령줄 플래그입니다. 가장 뒤의 것이 이깁니다. 잘못된 값(파일의 철자가 틀린 선택지, 잘못된 변수)은 조용히 무시되지 않고 항상 오류가 됩니다. |
| 변수 | 키를 대문자로 바꾸고 점을 밑줄로 바꿉니다. `general.auto_prune`은 `GITPOULPI_GENERAL_AUTO_PRUNE`입니다. GitPoulpi가 실제로 읽는 키에만 해당합니다(아래에서 *사용하지 않음*으로 표시된 키나 `ui.log_columns_hidden` 같은 목록은 제외). |
| 플래그 | `--backend git2\|system`은 그 명령이 시작하는 데몬에 한해 `general.default_backend`를 재정의합니다. |
| 토큰과 키 | 이 파일에는 절대 저장하지 않습니다. 호스팅 토큰과 AI API 키는 시스템 자격 증명 도우미(Git Credential Manager, libsecret / `pass`, Keychain)로 가며, 파일에는 참조(`token_ref`)만 남습니다. 도우미가 없으면 토큰 저장이 메시지와 함께 실패하고 토큰은 세션 동안만 유지됩니다. |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set`은 파일을 간결하게 유지하고(누군가 설정한 키만 둠), 요청받지 않은 키는 그대로
두지만 주석은 지웁니다. 목록과 전체 표는 거부하며, `keybindings.*`는 예외입니다(여러 키 조합을
공백으로 구분하고, 비우면 바인딩 해제). `[[profiles]]`, `[[integrations]]`, `[[ai.profiles]]`는
설정 화면에서 또는 직접 편집하세요.

표에서 **다시 시작**은 값을 시작할 때 읽는다는 뜻입니다. 나머지는 즉시 적용됩니다(또는
다음 새로 고침, fetch, 설정 닫기 때 적용됩니다).

## `[general]`

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2`(libgit2. 스테이징, merge, rebase, stash, 네트워크는 `git` 프로그램으로 대체) 또는 `system`(설치된 `git`만 사용). 실행 중인 데몬은 자신의 백엔드를 유지하므로 **재시작**해야 합니다(`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | 로컬 변경 사항이 체크아웃, pull, merge, rebase를 막을 때: `ask`(TUI에서는 대화상자. CLI와 MCP는 실패하며 `--autostash`를 안내), `always`(stash하고 실행한 뒤 pop), `never`(거부). [자동 stash](log-and-history.md#자동-stash)를 참고하세요. |
| `general.include_untracked_in_autostash` | `false` | 자동 stash가 추적되지 않는 파일도 포함합니다. |
| `general.initial_commits_in_graph` | `5000` | Log가 처음 불러오는 commit 수(최소 500). 값이 클수록 큰 저장소에서 메모리와 그리기 시간이 더 듭니다. |
| `general.lazy_load_commits_in_graph` | `true` | 끝 근처로 스크롤하면 더 오래된 commit을 불러옵니다. 끄면 Log가 초기 개수에서 멈춥니다. |
| `general.commits_per_lazy_page` | `1000` | 백그라운드 페이지당 commit 수(최소 50). |
| `general.max_commits_in_graph` | `100000` | 지연 로딩이 멈추는 지점(commit당 약 300바이트). `0` = 제한 없음. |
| `general.auto_fetch_interval_minutes` | `5` | N분마다 백그라운드 Fetch All, 0~60. `0` = 끔. **다시 시작.** |
| `general.auto_fetch_interval_unit` | `"auto"` | 설정이 간격을 표시하는 단위: `minutes`, `hours`, `auto`. 표시 전용이며 간격은 항상 `auto_fetch_interval_minutes`분입니다. `auto`(와 이 키가 없는 설정)는 분을 나누어떨어지게 하는 가장 큰 단위를 고릅니다 (60은 "1시간", 45는 "45분"). 맞지 않는 단위는 무시됩니다. 설정은 두 키를 모두 쓰며, 이전 GitPoulpi는 단위를 무시하고 분만 읽습니다. |
| `general.fetch_on_tab_open` | `true` | 탭이 열릴 때 Fetch All(GitPoulpi가 60초 이내에 그 저장소를 fetch했다면 건너뜀). |
| `general.auto_prune` | `false` | fetch가 사라진 remote-tracking 브랜치도 정리(prune)합니다. |
| `general.auto_prune_tags` | `false` | fetch가 리모트에서 삭제된 로컬 tag도 삭제합니다 (리모트에 있었던 tag만 대상이며, push한 적 없는 tag는 유지). Git 2.41 이상. |
| `general.hosting_refresh_minutes` | `5` | pull request / 이슈 목록과 도구 모음 칩의 새로 고침 주기, 0~120. `0` = 요청 시와 이벤트 때만. |
| `general.hosting_refresh_unit` | `"auto"` | 설정이 새로 고침 주기를 표시하는 단위: `minutes`, `hours`, `auto`이며 `general.auto_fetch_interval_unit`처럼 읽습니다. 표시 전용입니다. |
| `general.keep_submodules_up_to_date` | `false` | 체크아웃이나 pull 후 서브모듈이 어긋나 있으면 `git submodule update --init --recursive`를 실행합니다. [서브모듈](submodules.md)을 참고하세요. |
| `general.default_branch_name` | `"main"` | Init으로 만든 저장소의 첫 브랜치. 비워 두면 git이 정합니다. 기존 브랜치의 이름은 바꾸지 않습니다. |
| `general.delete_orig_files_after_merge` | `false` | merge가 완료된 후 `*.orig` 백업 파일을 삭제합니다. |
| `general.remember_tabs` | `true` | 지난 세션의 탭을 다시 엽니다. 끄면 시작할 때 해당 저장소만 엽니다(저장소 밖에서는 저장소 탭 없이 홈만). **다시 시작.** |
| `general.radar_enabled` | `true` | [Radar](radar.md): 탭, 팔레트 명령, `Alt+R`, 백그라운드 가져오기. 끄면 모두 숨겨지고 Radar를 위해 아무것도 가져오지 않습니다. 즉시 적용되며 `radar` 기능이 없는 빌드에는 표시되지 않습니다. |
| `general.startup_tab_in_repo` | `"current-repo"` | 시작한 폴더가 저장소 안에 있을 때 처음 여는 탭: `"current-repo"` (그 저장소의 탭, 없으면 새로 엽니다), `"last"` (마지막 종료 때 활성이던 탭), `"home"` 또는 `"radar"`. `--repo`로 지정한 저장소는 항상 자기 탭을 엽니다. `"last"`: `remember_tabs`가 꺼져 있거나 기억된 탭이 없으면 홈으로 열립니다. `"radar"`: Radar가 꺼져 있거나 빌드에 없으면 홈으로 열립니다. 시작할 때 한 번 읽으며, 적용하려면 재시작이 필요합니다. |
| `general.startup_tab_outside_repo` | `"home"` | 시작한 폴더가 저장소가 아닐 때(예: 사용자 폴더) 처음 여는 탭: `"home"`, `"last"` 또는 `"radar"`. `"last"`: `remember_tabs`가 꺼져 있거나 기억된 탭이 없으면 홈으로 열립니다. `"radar"`: Radar가 꺼져 있거나 빌드에 없으면 홈으로 열립니다. 시작할 때 한 번 읽으며, 적용하려면 재시작이 필요합니다. |
| `general.startup_tab_launcher` | `"last"` | GitPoulpi가 만든 바탕 화면 또는 시작 메뉴 바로 가기(`--launcher`를 전달합니다)로 시작할 때, 폴더와 관계없이 처음 여는 탭: `"home"`, `"last"` 또는 `"radar"`. 위의 두 설정보다 우선합니다. `"last"`: `remember_tabs`가 꺼져 있거나 기억된 탭이 없으면 홈으로 열립니다. `"radar"`: Radar가 꺼져 있거나 빌드에 없으면 홈으로 열립니다. 시작할 때 한 번 읽으며, 적용하려면 재시작이 필요합니다. |
| `general.group_worktree_tabs` | `true` | 저장소에 연결된 worktree의 탭을 그 저장소 탭 바로 뒤에 둡니다. worktree를 열 때, worktree 탭을 멀리 끌어 놓았을 때(그룹 옆으로 다시 붙습니다. 저장소 탭이 그룹의 머리이고 worktree가 그 뒤를 따릅니다), 이 설정을 켤 때 적용됩니다. 끄면 탭이 놓은 자리에 그대로 남습니다. |
| `general.undo_depth` | `100` | 실행 취소가 되돌릴 수 있는 작업 수. `0` = 무제한. |
| `general.conflict_detection` | `true` | 사전 충돌 감지: 현재 브랜치를 갈라져 나온 브랜치로 merge하면 충돌이 생길 때 Log에 배지를 표시합니다. 백그라운드에서 계산하며(메모리 안의 merge이므로 파일, 인덱스, ref는 바뀌지 않습니다) 저장소마다 끌 수 있습니다(`[preferences] conflict_detection`). 비교하는 기준 브랜치에는 자체 목록이 있습니다: [`[conflict_prevention]`](#conflict_prevention-기준-브랜치-목록). [merge 충돌 해결](merge-conflicts.md#충돌-방지)을 참조하세요. |
| `general.dev_mode` | 없음(자동) | 체크아웃에서 GitPoulpi를 빌드하는 사람을 위한 것이며 설정에는 표시되지 않습니다. 없으면 실행 중인 exe가 `<checkout>/target/<profile>/`에 있을 때 켜집니다(`target` 옆에 `Cargo.toml`과 `crates/gitpoulpi-tui`가 있음). `true` / `false`로 강제할 수 있습니다. 개발 모드는 `ui.launcher_build`만 사용할 수 있게 합니다. |
| `general.journal_outside` | `true` | 셸이나 AI 에이전트가 브랜치, tag, stash, HEAD에 한 작업을 실행 취소 기록의 항목으로 기록하고 외부 작업으로 표시하여 `Ctrl+Z`로 되돌릴 수 있게 합니다. 파일 편집은 기록하지 않습니다. `undo_enabled`가 필요합니다. [실행 취소와 다시 실행](undo-redo.md#gitpoulpi-밖에서-한-변경)을 참고하세요. |
| `general.undo_enabled` | `true` | `false`: 실행 취소 기록을 아예 하지 않고(가장 빠름), 실행 취소 기록이 지워지며, 실행 취소/다시 버튼이 사라집니다. [실행 취소와 다시 실행](undo-redo.md)을 참고하세요. |
| `general.active_profile` | `""` | commit을 만들 때 사용하는 `[[profiles]]` 신원의 ID. 비어 있거나 이름이나 이메일이 빈 프로필이면 git의 `user.name` / `user.email`을 사용합니다. |
| `general.external_editor` | `""` | 파일의 *외부 편집기로 열기*에 쓰는 명령. 예: `code -r {file}`. 비어 있으면 설치된 경우 VS Code, 아니면 시스템 기본값. |
| `general.external_diff_tool` | `""` | *외부 diff 도구로 열기*에 쓰는 명령: `{local}`(commit된 / HEAD 버전), `{remote}`(작업 트리 / 인덱스 버전). 비어 있으면 `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | 충돌 카드(바이너리, LFS 또는 너무 큰 파일)의 *외부 merge 도구로 열기*에 쓰는 명령: `{base}`, `{local}`(A, 현재), `{remote}`(B, 수신), `{merged}`(도구가 결과를 기록하는 파일. 도구가 성공적으로 종료하면 GitPoulpi가 스테이징합니다). 비어 있으면 `git mergetool --no-prompt`. |
| `general.file_explorer` | OS별 | *폴더에서 보기*에 쓰는 명령: `{path}`(파일), `{dir}`(그 폴더). 기본값: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

세 명령 템플릿은 먼저 인수로 분할되고(`"..."`가 한 단어로 묶음), 그다음 자리 표시자가
채워지므로 공백이 있는 경로도 하나의 인수로 유지됩니다. 셸은 사용되지 않습니다.

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log와 그래프**

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto`(터미널이 그림을 그릴 수 있으면 그림, 아니면 문자), `image`, `text`. **다시 시작.** |
| `ui.image_protocol` | `"auto"` | 그림(그래프, 아바타)이 터미널에 전달되는 방식: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = kitty, Ghostty, 그리고 `kitty_graphics = true`인 herdr에서는 Kitty 프로토콜. WezTerm, iTerm2, Rio, mintty에서는 iTerm2. Windows Terminal, foot, Konsole, mlterm에서는 sixel(아바타만). tmux, zellij, screen, 알 수 없는 터미널에서는 none. **다시 시작.** |
| `ui.theme` | `"system"` | 색상 테마: 내장 슬러그(`gitpoulpi theme list`가 전체를 나열), `<config>/themes/`에 있는 파일의 슬러그, 또는 `system`(시작할 때 감지한 터미널의 밝거나 어두운 배경을 따름. 예전 값 `default`도 같은 뜻). `config set`은 알 수 없는 이름을 거부합니다. [테마](themes.md)를 참고하세요. |
| `ui.graph_color_theme` | `"theme"` | 레인 팔레트: `theme`(테마를 따름) 또는 임의의 레인 팔레트 이름: `gitpoulpi`, `light`, `material`, `serie`, 그리고 다른 테마의 팔레트. |
| `ui.graph_lane_background` | `"off"` | `off` 또는 `row_tint`(그래프 뒤의 옅은 레인 색 띠와 그 가장자리의 막대). **앱: 그래프 행 색조 전환**(`Ctrl+N Shift+T`)이 이를 전환하고, **앱: 그래프 색상 테마 변경...**(`Ctrl+N Shift+Y`)이 `ui.graph_color_theme`을 고릅니다. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle`(`row_tint`일 때만). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left`와 `right`는 세로 열로 그리며, 80열 미만의 창에서는 `bottom`으로 되돌아갑니다. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: 옮기거나 크기를 바꾼 대화상자(제목 표시줄이나 테두리를 끌거나 `Ctrl+N m`)가 자리를 유지할지 정합니다. GitPoulpi를 닫을 때까지(기본값), 다음 실행에도(데이터 폴더의 `dialog_geometry.txt`에 디스크 저장), 또는 유지하지 않음. |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: 도구 모음에서 버튼이 놓이는 위치(행의 왼쪽·가운데·오른쪽, 열의 위·가운데·아래). |
| `ui.toolbar_pr_pinned` | `true` | 풀 리퀘스트 버튼을 도구 모음의 맨 끝에 고정합니다. `false`이면 다른 버튼과 함께 배치됩니다. |
| `ui.statusbar_position` | `"bottom"` | `top`(저장소 탭 위) 또는 `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom`(창의 마지막 줄, 상태 표시줄 아래) 또는 `left`(왼쪽 가장자리의 열, 80열 미만에서는 위에 유지). |
| `ui.graph_show_stashes` | `true` | Log에 stash마다 한 행을 표시합니다. |
| `ui.ref_connector` | `true` | ref 배지에서 해당 commit까지 가는 가는 선. |
| `ui.column_resize` | `"line"` | Log, Radar, diff·파일·blame 보기의 commit 열에서 열 경계를 잡는 방식. `line`: 각 경계는 한 칸 폭의 틈이고 머리글에 작은 눈금이 있으며, 포인터가 경계 위(머리글 또는 표의 어디든)에 있는 동안 전체 높이에 걸쳐 가는 강조선이 그려지고 그 선의 어느 지점에서든 드래그하면 크기가 바뀝니다. `header`: 틈도 눈금도 없고, 경계 뒤 머리글 셀의 첫 글자가 밝아지며 거기서 드래그하면 크기가 바뀝니다(머리글에서만). 키보드 크기 조절 키와 팔레트 항목은 두 모드에서 같습니다. |
| `ui.ref_tree` | `true` | Refs 사이드바에서 브랜치와 tag를 `/` 기준으로 폴더로 묶습니다(`y`로 전환). |
| `ui.graph_ghost_ref` | `true` | Log의 Ref 열은 자체 ref가 없는 마우스를 올린 commit에, 열에 공간이 있으면, 소속 브랜치(없으면 원격 브랜치)의 이름을 흐린 알약 모양으로 표시하며, tag는 절대 표시하지 않습니다. 삭제된 브랜치는 그것을 병합한 merge commit의 제목에서 추론해 취소선으로 그립니다. 알약에 마우스를 올리면 종류가 표시됩니다. |
| `ui.reveal_on_ref_click` | `true` | Refs 창에서 ref(브랜치, tag, 리모트 브랜치, stash)를 한 번 클릭하면 Log가 해당 commit으로 스크롤하고 행이 깜박입니다. commit 선택은 움직이지 않습니다. |
| `ui.log_wip_counts_in_message` | `true` | 변경 열이 숨겨져 있을 때 WIP 메시지에 WIP 변경 수를 표시합니다. |
| `ui.log_columns_layout` | `"fit"` | Log가 열을 배치하는 방식. `fit`: 보이는 모든 열이 창에 정확히 맞춰지고 `message`만 늘어나며, 창이 좁으면 열이 통째로 양보합니다. `scroll`: 열이 너비를 유지하고(`message`도), 양보하는 열이 없으며, 표가 창보다 넓으면 옆으로 스크롤됩니다. |
| `ui.log_column_widths_scroll` | 비어 있음 | `scroll` 배치의 열 너비, 이름 -> 칸 수(`message` 허용. `fit` 배치의 것으로 `message`를 담지 않는 `ui.log_column_widths`와 다릅니다). 배치가 `scroll`일 때 열을 드래그하거나 크기를 바꾸거나 내용에 맞출 때 기록됩니다. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | 비어 있음 | Log 열(`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`). 열을 숨기거나 끌거나 크기를 바꿀 때 기록됩니다. `message`는 숨길 수 없습니다. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | 비어 있음 | diff 옆 [commit 열](interface-tour.md#commit-열)의 부분을 Log와는 별도로 저장합니다. 표시할 부분의 이름(`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`. 비어 있으면 레인, 메시지, 작성자, 날짜), 왼쪽에서 오른쪽으로의 순서(같은 이름으로 쓰며, 비어 있으면 방금 나열한 순서이고 빠진 이름은 그 순서대로 뒤에 붙습니다), 부분별로 끌어서 정한 너비(셀 단위, 빠진 부분은 자동)이며, 부분을 고르거나 옮기거나 크기를 바꿀 때 기록됩니다. `message`는 숨길 수 없습니다. 이 키가 생기기 전에 쓰인 세션 파일은 한 번 읽어 이 키로 옮겨집니다. |
| `ui.sidebar_sections_hidden` | 비어 있음 | 숨긴 왼쪽 창 섹션(`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`). 창의 오른쪽 클릭 메뉴나 팔레트에서 기록됩니다. 존재하는 섹션 중 적어도 하나는 항상 표시됩니다. 이 값은 기본값입니다. 저장소마다 자체 섹션을 숨길 수 있고(저장소의 상태 파일에 저장), 이 키는 자체 선택이 없는 저장소에 적용됩니다. *모든 저장소에 사용*이 이 키를 씁니다. |
| `ui.sidebar_sections_scope` | `"repository"` | 왼쪽 창 섹션을 누구의 선택으로 적용할지 정합니다. `repository`(저장소마다 자체 선택을 가지며 `ui.sidebar_sections_hidden`이 기본값)이거나 `global`(모든 저장소가 `ui.sidebar_sections_hidden`을 표시하며 변경도 거기에 기록됨)입니다. 전환해도 어느 쪽 선택도 사라지지 않습니다. `global`이 켜져 있는 동안에도 저장소별 선택은 상태 파일에 남습니다. 설정: *왼쪽 창 섹션*. |
| `ui.language` | `"auto"` | 인터페이스 언어: `auto`(시스템 로캘, 없으면 영어), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **실시간.** `GITPOULPI_LANG`가 이를 재정의합니다. 이 빌드에 없는 언어는 영어로 표시됩니다. **앱: 언어 변경...**(`Ctrl+N g`)도 실시간 미리 보기와 함께 이를 설정합니다. CLI와 MCP 출력은 영어로 유지됩니다. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... 가 모든 언어에서 영어로 유지됩니다. `true`: 선택한 언어의 git 어휘(프랑스어의 "remiser", "étiquette", "valider"...). **실시간.** |
| `ui.git_terms_english` | `false` | ja, ko, zh-CN에만 해당: 음역 대신 영어 git 단어(stage, unstage, staged, unstaged...)를 표시합니다. `ui.translate_git_terms`가 `true`인 동안에는 효과가 없습니다. **실시간.** |
| `ui.mouse_enabled` | `true` | `false`이면 마우스(및 텍스트 선택)를 터미널에 맡깁니다. **다시 시작.** |
| `ui.mouse_scroll_lines` | `3` | 휠 한 칸당 Log 행 수, 1~20. **다시 시작.** |
| `ui.kitty_keyboard` | `"auto"` | `auto`는 터미널에 kitty 키보드 프로토콜을 요청합니다(그러면 `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`이 도착함). `off`는 요청하지 않습니다. **다시 시작.** |
| `ui.logo_color` | `""` | 정보 창의 로고 색 구성, 슬러그(`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`). 비어 있거나 알 수 없는 값이면 빌드의 기본값(`lavender`). 정보에서 로고를 클릭하거나 색을 고르면 기록됩니다. |

**아바타**(기본적으로 꺼짐. 켜면 작성자 이메일의 SHA-256 해시를 Gravatar에 보내고 연결된
GitHub/GitLab 계정에도 질의합니다)

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `ui.avatars` | `"off"` | `graph`(그림 그래프의 commit 원 안), `author`(작성자 열 안), `off`. 예전 `ui.show_avatars`(`true`)는 `ui.avatars`가 없는 동안 `author`로 읽힙니다. |
| `ui.avatar_sources` | `"gravatar_hosting"` | 조회 순서: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. GitLab 호스트는 연결된 프로필을 통해서만 질의합니다. |
| `ui.avatar_fallback` | `"identicon"` | `identicon`(로컬에서 생성) 또는 `initials`. 그림 프로토콜이 없는 터미널에서는 항상 이니셜 배지를 씁니다. |

**Diff와 편집기 표시**

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` 또는 `split`(diff에서 `m`). |
| `ui.diff_hunks_only` | `true` | `false`이면 변경 사항이 제자리에 표시된 전체 파일을 보여 줍니다(`H`). |
| `ui.diff_show_whitespace` | `true` | 공백만 바뀐 변경 사항을 표시합니다(`s`). |
| `ui.diff_word_wrap` | `false` | 긴 diff 줄을 줄 바꿈합니다(`w`. 설정 > Diff 및 편집기). |
| `ui.minimap` | `false` | diff와 편집기의 스크롤바 옆에 미니맵을 표시합니다(`M`). |
| `ui.mermaid_detail` | `refs` | Log의 Mermaid gitGraph로 복사의 상세 수준: `structure`, `refs`, `subjects`, `full`(팝업에서 기억되며 `log --format mermaid`의 기본값). |
| `ui.mermaid_orientation` | `lr` | 그 그래프의 방향: `lr`(왼쪽에서 오른쪽) 또는 `tb`(위에서 아래). |
| `ui.mermaid_fence` | `false` | 복사한 Mermaid 스크립트를 `mermaid` 코드 펜스로 감쌉니다. |
| `ui.mermaid_subject_width` | `40` | Mermaid id에서 커밋 제목을 이 글자 수로 자릅니다(`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256`, `16`: 그리는 데 쓸 색 수. `auto`는 Apple Terminal(256), Linux 콘솔(16), `COLORTERM`이 없는 `*-256color` 터미널(256)을 제외하면 truecolor입니다. 실시간. [테마](themes.md)를 참고하세요. |
| `ui.input_color` | `"theme"` | 텍스트 필드 배경: `theme`(테마 자체의 것), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | `custom` 색, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | AI commit 작성기가 각 제안의 파일을 `expanded` 또는 `collapsed` 상태로 엽니다. |
| `ui.blame_side` | `"right"` | Blame 보기에서 주석 열(아바타, 작성자, 경과 시간, 요약)이 놓이는 쪽: `right`(기본값) 또는 `left`. |
| `ui.blame_width` | `46` | Blame 주석 열의 너비(셀 단위, 24에서 90). 경계를 드래그하거나 보기에서 `<` / `>`를 누르세요. 새 너비는 여기에 저장됩니다. |
| `ui.pr_terminology` | `"auto"` | `auto`(GitHub에서는 pull request, GitLab에서는 merge request), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | 체크아웃한 브랜치의 최신 pull request 또는 merge request가 닫히거나 병합되면 도구 모음의 해당 칩을 숨기고, 대신 흐리게 표시된 "+ PR" 칩(새로 만들기)이 브랜치에 요청이 없는 것처럼 표시됩니다. `false`: 칩은 상태와 관계없이 항상 최신 요청을 보여 줍니다. |

diff 안에서 바꾼 전환 설정은 즉시 여기에 저장되며 이후 열리는 모든 diff에 사용됩니다.

**날짜**: `ui.date_format`(`"YYYY-MM-DD"`, Blame / 파일 기록 날짜 열), `ui.date_time_format`
(`"YYYY-MM-DD HH:mm"`, Log의 날짜 열), `ui.date_verbose_format`(`"D MMMM YYYY HH:mm"`,
commit 상세, 날짜 툴팁, *복사 > 날짜*). 토큰: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd`(요일, 전체 / 약어) `HH` `mm` `ss`. 그 외는 그대로 복사됩니다. 월과 요일 이름은
`ui.language`를 따르며, 일 번호 뒤에 쓰는 월은 해당 언어에 속격형이 있으면 속격형입니다
(러시아어, 우크라이나어, 폴란드어, 체코어). `D MMMM YYYY` 형식은 일본어, 중국어, 한국어에서
연도가 앞에 오도록 기록됩니다(`2024年3月5日`). `ui.date_locale`(`"system"`이 기본값, 또는
`"english"`)은 언어와 관계없이 이름을 영어로 유지합니다.

**에이전트, 터미널, 탭 폴더**

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | 창이 에이전트의 `ui show` / `ui propose`를 아예 받아들입니다(설정 > 에이전트 및 MCP > *AI 에이전트의 화면 제어 허용*). 끄면 어떤 에이전트도 닿지 못합니다. [MCP와 에이전트](mcp-and-agents.md#사용자에게-보여-주기)를 참고하세요. |
| `ui.launch` | `"off"` | 열린 창이 없을 때 에이전트가 GitPoulpi 창을 열 수 있는지 여부: `off`, 프리셋(`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) 또는 `custom`. |
| `ui.launch_command` | `""` | `custom` 템플릿: `{repo}`, `{session}`, `{gitpoulpi}`. 인수로 분할되며 셸은 사용되지 않습니다. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons`(탭 줄이 *종료 후 cd*를 제공) 또는 `auto`(종료할 때마다 셸의 폴더가 바뀌고, 새 탭은 기본 *열기 위치* 항목을 실행). [활성 탭의 폴더 따라가기](following-the-tab-folder.md)를 참고하세요. |
| `ui.terminal_launch` | `"auto"` | 레거시. 아래와 [열기 위치](open-in.md)를 참고하세요. 예전 *여기서 터미널*이 터미널을 여는 방식: `auto`(`ui.launch` 프리셋, 없으면 `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), 프리셋 또는 `custom`. |
| `ui.terminal_launch_command` | `""` | `custom` 템플릿. `{repo}`는 폴더입니다. |
| `ui.open_in` | *(없음)* | [열기 위치](open-in.md) 목록. `label`과 `command`를 가진 테이블 배열 `[[ui.open_in]]`입니다. 없으면 "아직 구성되지 않음"을 뜻하며, 목록은 레거시 `ui.terminal_launch*` 키(`auto`가 아닌 값은 첫 항목이 됨), OS 파일 탐색기, 환경에서 감지한 멀티플렉서와 터미널(herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...), `PATH`에서 찾은 편집기로부터 계산됩니다. 설정에서 처음 저장할 때 목록이 기록됩니다. `open_in = []`는 빈 목록입니다. `config set`으로는 설정할 수 없습니다(파일을 편집). |
| `ui.open_in_default` | `""` | "기본 위치에서 열기"가 실행하는 항목의 레이블. 비어 있거나 알 수 없는 값이면 첫 항목. |
| `ui.launch_terminal` | `"auto"` | 데스크톱 바로 가기(설정 > 런처)와 Windows의 다시 열기가 GitPoulpi를 시작하는 터미널입니다. `auto`는 설치된 것 중에서 고릅니다(Windows: Windows Terminal, 없으면 WezTerm, Alacritty, 일반 콘솔. Linux: `xdg-terminal-exec`, 없으면 처음 찾은 터미널. macOS: iTerm2, Ghostty, WezTerm, kitty가 설치되어 있으면 그것, 없으면 Terminal). `default`는 OS 자체의 기본값이며, `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal` 같은 터미널 ID를 지정할 수도 있습니다. |
| `ui.launcher_build` | `"current"` | 개발 모드 전용(`general.dev_mode` 참고. 그때만 설정 > 런처에 표시되며 그 밖에는 무시됨). `"current"`: 바로 가기와 명령 별칭이 자신을 만든 빌드를 시작합니다. `"newest"`: 체크아웃의 `target/dist`와 `target/release` 중 가장 최근에 빌드된 것을 클릭할 때마다 판단해 시작하며, 어느 빌드를 시작해도 바로 가기를 다시 쓰지 않습니다. |
| `ui.command_alias` | `""` | 터미널에서 GitPoulpi를 시작하는 명령의 이름, 예를 들어 `gp`(설정 > 런처, `gitpoulpi alias create`). 비어 있으면 없음입니다. 영문자, 숫자, `-`, `_`, `.`만 쓸 수 있습니다. **만들기: 명령 별칭** 버튼이 쓰는 이름일 뿐이며 만들기 전에는 아무것도 생기지 않습니다. |
| `ui.command_alias_add` | `""` | `gitpoulpi --add`를 실행하는 두 번째 명령의 이름, 예를 들어 `gpa`. 현재 폴더를 이미 실행 중인 GitPoulpi의 새 탭으로 엽니다(설정 > 런처, `gitpoulpi alias create gpa --add`). 비어 있으면 없음입니다. 이름 규칙은 같으며, **만들기** 버튼이 `ui.command_alias`와 함께 만듭니다. |
| `ui.relaunch_in_terminal` | `true` | Windows 전용: exe가 더블 클릭으로 자체 콘솔에서 시작되면 GitPoulpi가 `ui.launch_terminal`에서 다시 열리고 종료합니다. 끄면 그 콘솔에 그대로 있습니다. |
| `ui.reuse_window` | `true` | GitPoulpi 바로 가기(`gitpoulpi launch`)를 클릭하면 새로 시작하지 않고 이미 열려 있는 GitPoulpi의 창을 보여 줍니다. 끄면 클릭할 때마다 새 창을 시작합니다. |

**`ui.open_in` 명령**은 먼저 인수로 분할되고(공백, `'...'`, `"..."`), 그다음 각 인수 안에서 변수가 치환되므로 공백이 있는 폴더도 하나의 인수로 유지됩니다. 명령이 셸을 시작하지 않는 한(`sh -c '... "$GITPOULPI_CWD"'`) 셸은 사용되지 않습니다. 변수: `${cwd}`(탭의 폴더), `${repo}`(저장소 루트), `${name}`(폴더의 기본 이름), `${branch}`(detached 상태에서는 비어 있음), `${web_url}`(`origin` 리모트가 GitHub 또는 GitLab 프로젝트이면 그 웹 페이지, 아니면 비어 있음. 이를 사용하는 항목은 비어 있으면 실행을 거부함), `${env:NAME}`(설정되지 않았으면 비어 있음). 예전 `{repo}`는 여전히 `${cwd}`를 뜻합니다. 그 밖의 `${...}`는 그 이름을 알려 주는 오류입니다. 예시:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

레거시: `ui.terminal_launch`와 `ui.terminal_launch_command`는 여전히 읽히지만(`ui.open_in`이 없는 동안 목록의 초기값이 됩니다. `custom`은 `{repo}`가 `${cwd}`로 바뀐 "Terminal (custom)"이 되고, 프리셋 ID는 그 프리셋이 됩니다) `ui.open_in`으로 대체되었습니다.

### 이미지

| 키 | 기본값 | 의미 |
|---|---|---|
| `ui.image_view` | `true` | 이미지(PNG, JPEG, GIF, WebP, BMP, ...)인 바이너리 파일을 Diff 보기와 파일 보기에서 그림으로 표시합니다. 끄면 일반 바이너리 안내가 표시됩니다. |
| `ui.image_view_max_mib` | `32` | 이보다 큰 이미지(MiB)는 디코딩하지 않으며 대신 정보 카드가 표시됩니다. |
| `ui.image_view_max_unit` | `"auto"` | 설정이 한도를 표시하는 단위: `mib`, `gib`(1024 MiB), `auto`. 표시 전용이며 한도는 항상 `image_view_max_mib` MiB(1에서 1024)입니다. |
| `ui.image_view_text_fallback` | `"halfblock"` | 이미지 프로토콜이 없는 터미널에서: `halfblock`(색이 있는 반블록 문자) 또는 `card`(정보 카드만). |
| `ui.image_view_pixelated_from` | `"2x"` | 확대할 때 부드럽게 처리하는 대신 정사각형 픽셀을 보여 주기 시작하는 배율: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | 이전과 이후 창이 하나의 확대/축소와 스크롤 위치를 공유합니다. |
| `ui.image_editors` | *(없음)* | "편집 프로그램" 이미지 편집기. `label`과 `command`를 가진 테이블 배열 `[[ui.image_editors]]`입니다. 없으면 "아직 구성되지 않음"을 뜻하며, 목록은 이 컴퓨터에서 감지한 편집기(`PATH`의 프로그램, Flatpak 설치, Windows의 알려진 설치 폴더, macOS의 `/Applications`)와 그다음 OS 기본 앱입니다. 설정에서 처음 저장할 때 목록이 기록됩니다. `image_editors = []`는 빈 목록입니다. `config set`으로는 설정할 수 없습니다(파일을 편집). |
| `ui.image_editor_default` | `""` | "기본 편집 프로그램"이 실행하는 편집기의 레이블. 비어 있거나 알 수 없는 값이면 첫 항목. |

**`ui.image_editors` 명령**은 먼저 인수로 분할되고(공백, `'...'`, `"..."`), 그다음 각 인수 안에서 변수가 치환되므로 공백이 있는 경로도 하나의 인수로 유지됩니다. 셸은 사용되지 않습니다. 변수: `${file}`(이미지의 절대 경로), `${dir}`(그 폴더), `${name}`(파일 이름), `${repo}`(저장소 루트. 임시 사본에서는 비어 있음), `${env:NAME}`(설정되지 않았으면 비어 있음). 그 밖의 `${...}`는 그 이름을 알려 주는 오류입니다. 작업 트리 파일이 아닌 이미지(commit의 버전, 인덱스, 삭제된 파일)는 **읽기 전용 임시 사본**으로 열리므로 편집기가 저장소를 건드릴 수 없습니다. 명령줄이 없는 편집기(Microsoft Store의 Affinity Photo, 일부 Photoshop 설치)는 프리셋에 없으므로 사용자 지정 항목을 작성하세요. 예시:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**아직 사용하지 않음**(허용되고 저장되지만 아무것도 읽지 않음):
`ui.graph_colorization`(설정에서 제거됨. 로드할 때는 여전히
허용됨), `ui.date_word_format`(설정에서 제거됨. 로드할 때는 여전히 허용됨). 예전 `ui.diff_view_mode`는
`diff_layout` / `diff_hunks_only`로 한 번 읽힙니다.

## `[editor]`

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `editor.tab_size` | `4` | 탭 정지당 열 수, 1~8(편집기와 diff). |
| `editor.eol` | `"auto"` | 내장 편집기가 기록하는 줄 끝: `auto`(파일의 것을 유지), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | 편집기의 언어별 색상. |
| `editor.line_numbers` | `true` | 편집기와 diff의 줄 번호. |
| `editor.follow_selection` | `true` | 편집기가 열려 있는 동안 오른쪽 창에서 다른 파일을 선택하면 그 파일도 편집기에서 열리며, 저장하지 않은 변경 사항이 있으면 먼저 저장할지 묻습니다. 편집할 수 없는 파일은 기본 보기로 열립니다. commit의 파일을 고르면 편집기를 떠나 그 diff가 열립니다. |

## `[merge]`

merge 충돌 편집기([merge 충돌 해결](merge-conflicts.md))입니다. 설정 > Diff 및 편집기에 표시됩니다.

| 키 | 기본값 | 의미 |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | A와 B를 나란히 표시하기 시작하는 편집기 영역의 너비(열 단위). 긴 줄은 옆으로 스크롤됩니다(Shift+휠, Shift+화살표). 더 좁으면 탭 뒤에 한 번에 한쪽만 표시합니다. `40`이 최솟값이며 `0`은 항상 나란히 표시를 뜻합니다(40열 미만에서의 안내는 유지됨). 편집기는 열려 있는 동안 Refs와 Status 패널을 숨기므로 영역은 보통 창 전체입니다. |

## `[git]`, `[credentials]`, `[ssh]`

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `git.pull_mode` | `"merge"` | Pull(버튼, `Ctrl+G p`, `Git: Pull`)이 하는 일: `merge`(가능하면 fast-forward), `ff_only`, `rebase`. 항상 git에 명시적으로 전달됩니다. 예전 `fetch_all`은 `merge`로 읽힙니다. |
| `git.push_behavior` | `"direct"` | `direct`는 기본 리모트로 push합니다. `ask`는 먼저 리모트 메뉴를 표시합니다. |
| `git.merge_fast_forward` | `"never"` | 직접 시작하는 merge(브랜치 메뉴, 드래그 앤 드롭, 팔레트, `gitpoulpi merge`)에서 브랜치를 fast-forward할 수 있을 때의 동작: `never`는 항상 merge commit을 만들고(`--no-ff`), `allowed`는 가능하면 fast-forward하며(git 기본값, `merge.ff`가 적용됨), `only`는 fast-forward하거나 거부합니다. 저장소 자체의 선택(설정 > Merge 옵션, 그 `repo-state.toml`의 `merge_fast_forward`)이 우선하며, CLI의 `--ff`, `--no-ff`, `--ff-only`는 둘 다보다 우선합니다. 즉시 적용됩니다. |
| `git.default_encoding` | `"utf-8"` | 유효한 UTF-8이 **아닌** 텍스트(파일 내용, blame, diff, `encoding` 헤더가 없는 commit 메시지)의 인코딩: 모든 WHATWG 레이블(`windows-1252`, `shift_jis`, ...). 표시 전용이며 스테이징은 다시 인코딩하지 않습니다. 저장소가 재정의할 수 있습니다. |
| `git.lfs_enabled` | `true` | Git LFS를 표시하고 제어합니다. 저장소가 재정의할 수 있습니다. [Git LFS](lfs.md)를 참고하세요. |
| `git.lfs_autofix_filters` | `true` | LFS를 사용하는 저장소를 pull 또는 push하기 전에, 누락된 LFS 필터 / pre-push 훅을 설치합니다(`git lfs install --local`, 이 저장소에만 해당). [Git LFS](lfs.md)를 참고하세요. |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *사용하지 않음*: 저장소의 `core.autocrlf` / `core.eol`은 git 자체 설정이며 설정 > Git 및 인코딩(또는 `gitpoulpi repo-config`)에서 편집합니다. |
| `credentials.https_mode` | `"credential_helper"` | fetch, pull, push, clone의 HTTPS 인증: `credential_helper`(git의 것), `prompt`(도우미를 무시. git이 터미널에서 묻기 때문에 백그라운드 데몬에서는 실패함), `none`(공개 리모트만). 예전 `encrypted_file` 값은 `credential_helper`로 읽힙니다. |
| `ssh.key_type` | `"ed25519"` | *키 쌍 생성*이 만드는 유형: `ed25519`, `rsa`(4096비트), `ecdsa`(NIST P-256). 파일은 `gitpoulpi_<type>`입니다. SSH 자체는 항상 ssh-agent와 ssh 설정을 사용합니다. |
| `ssh.private_key_path`, `ssh.public_key_path` | 설정 안 됨 | *키 쌍 생성*이 기록합니다. `ssh.private_key_path`가 설정되어 있으면 fetch, pull, push, clone이 `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`로 git을 실행합니다. 개인 키 경로가 비어 있으면 `ssh.public_key_path`가 같은 일을 합니다(ssh가 일치하는 agent 키를 고름). 환경의 `GIT_SSH_COMMAND`/`GIT_SSH` 또는 git 설정의 `core.sshCommand`(저장소별 고정, `gitpoulpi ssh-command set --key`)가 우선합니다. 전송 직전마다 읽으므로 변경 사항은 다음 전송에 적용됩니다. |

## `[[integrations]]` (호스팅과 이슈 트래커)

설정 > 연동 또는 `gitpoulpi hosting integrations add`가 기록합니다. 모든 종류에 클라이언트가
있습니다(개인 토큰만 지원하며 OAuth는 없음).

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

자체 호스팅 서버는 평문 http일 수 있습니다. `url = "http://localhost:3000"`은 입력한 그대로
유지됩니다(스킴과 포트가 모든 요청에 쓰이며, 그 호스트의 리모트가 이 프로필과 일치합니다). 평문 http로
보낸 토큰은 암호화되지 않은 채 전송되므로, http는 로컬 또는 신뢰할 수 있는 네트워크에서만 사용하세요.

`kind`는 `azure_devops`, `azure_devops_server`, `bitbucket`, `bitbucket_data_center`,
`jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine`, `gerrit`도 허용합니다
(Gitea와 Forgejo는 클라이언트 하나를 공유하며, Forgejo 프로필의 기본값은 `https://codeberg.org`입니다). Jira, Trello,
Linear, YouTrack, Redmine은 이슈 트래커 전용이고, Gerrit은 리뷰 전용입니다(변경이 곧 pull request이며 이슈는 없습니다).
Bitbucket Cloud에는 이슈 트래커가 없습니다.

`account`는 토큰 옆에 두는 비밀이 아닌 신원 정보로, 대부분의 종류에서는 비어 있으며 생략됩니다.
`jira_cloud`와 `bitbucket`에서는 Atlassian 계정 이메일(Bitbucket 계정이 비어 있으면 워크스페이스
또는 저장소 액세스 토큰을 뜻하며 Bearer로 전송됨), `trello`에서는 API 키, `gerrit`에서는 사용자 이름(토큰은 그 HTTP 비밀번호이며 HTTP basic으로 전송됨)입니다. 비밀은 여전히
토큰 하나뿐입니다. `azure_devops`의 `url`은 `https://dev.azure.com/<organization>`입니다.
`https://<organization>.visualstudio.com` URL은 같은 조직으로 읽힙니다.
`azure_devops_server`에서는 `https://tfs.example.com/tfs/DefaultCollection` 같은 컬렉션
루트입니다.

`user`(연결된 계정)와 `ssh`(프로필의 키 선택)는 자동으로 기록됩니다. 토큰은
`GITPOULPI_TOKEN_<ID>`(ID는 대문자, 영숫자가 아닌 문자는 `_`)에서, 없으면 저장된 토큰에서,
없으면 하나의 프로필만 그 호스트를 사용할 때 해당 호스트의 자격 증명 도우미에서 가져옵니다
(GitHub와 GitLab 프로필만 해당. 다른 종류는 자격 증명 도우미를 읽지 않음). 자세한 내용:
[호스팅과 이슈 트래커 연동](hosting-integration.md). 예전 `[[hosting]]` 테이블은 *사용하지
않습니다*.

`local_markdown`(Markdown 노트 폴더를 이슈 소스로 사용)에서 `url`은 각 저장소 기준 상대 경로 또는 절대 경로의 폴더이며 토큰은 없습니다.
선택적인 `[integrations.local]` 테이블은 `statuses`(보드의 열 순서, 기본값 `todo`, `doing`, `done`), `done_status`(기본값 `done`),
`reference_prefix`(`LOC`이면 `LOC-12`로 표시), `me`("나에게 할당됨"이 뜻하는 사람)를 정합니다.
[로컬 이슈와 Obsidian](local-issues.md)을 참조하세요.

### `[integrations.radar]` (계정 전체 목록)

코드 호스트 프로필(GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; Bitbucket Cloud는 제외)의 선택적 테이블입니다. 없으면 모든 키가 꺼진 것과 같습니다. `gitpoulpi hosting integrations radar`가 쓰고, `gitpoulpi hosting radar`와 Radar가 읽으며, 설정 > 연동(연결된 프로필의 Radar 블록)에서 편집합니다.

```toml
[integrations.radar]
review_requests = true    # requests waiting for my review, every repository
open_requests = false     # every open request of my namespaces
include_drafts = false    # keep draft requests in these lists
assigned_issues = true    # open issues assigned to me, every repository
namespaces = ["acme"]     # orgs / groups to search for open requests
review_mode = "ask"       # ask | clone | api
```

| 키 | 기본값 | 의미 |
|---|---|---|
| `review_requests` | `false` | 저장소의 클론 여부와 관계없이 내 리뷰를 요청한 열린 요청 |
| `open_requests` | `false` | 내 네임스페이스와 내가 속한 각 org 또는 그룹(또는 `namespaces`)의 열린 요청 전부. "토큰이 볼 수 있는 모든 것"은 아닙니다 |
| `include_drafts` | `false` | 이 값을 켜지 않으면 초안 요청은 이 목록에서 제외됩니다 |
| `assigned_issues` | `false` | 나에게 할당된 열린 이슈, 모든 저장소 |
| `namespaces` | `[]` | `open_requests`를 검색할 org / 그룹. 비어 있으면 내 네임스페이스와 소속된 모든 곳(GitHub: 최대 12개 검색, GitLab: 그룹 10개와 소유 프로젝트 10개, Gitea, Forgejo, Gerrit: 10개. Azure DevOps는 프로젝트 이름, Bitbucket Data Center는 프로젝트 키, Gerrit은 프로젝트 경로 접두사를 받습니다) |
| `review_mode` | `"ask"` | 클론되지 않은 저장소의 요청을 리뷰할 때의 동작: `ask`, `clone`(클론한 뒤 리뷰) 또는 `api`(호스트 API로 리뷰) |

응답은 프로필별로 `config.toml` 옆의 `radar-account.json`에 5분간 캐시됩니다. 이 파일은 캐시이므로 삭제해도 안전합니다.

### 신뢰 저장소(`config.toml`에 없음)

각 연동과 API 키 AI 프로필이 무엇에 대해 신뢰받았는지(주소, 계정 id, 고정된 인증서 키, 일반 http 허용 여부)는 사용자만 접근할 수 있는 비공개 데이터 폴더의 `trust.toml`(`<data>/private/trust.toml`: Windows는 `%APPDATA%\GitPoulpi\private\`, Linux는 `~/.local/share/gitpoulpi/private/`)에 있습니다. 설정을 편집할 수 있는 사람이 편집만으로 신뢰를 넓힐 수 없어야 하므로 의도적으로 `config.toml`에 포함하지 않습니다. `gitpoulpi hosting integrations trust`와 `gitpoulpi ai trust`로 변경하세요([보안 확인](hosting-integration.md#보안-확인) 참고). 이 파일은 직접 편집하는 것이 아니라 GitPoulpi가 다시 씁니다.

## `[mcp]` (AI 에이전트가 할 수 있는 일)

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `mcp.allow_mutations` | `false` | 꺼짐: 무언가를 바꾸는 MCP 도구가 거부됩니다. 읽기, `ui_*`, 제안은 계속 동작합니다. `true`이면 에이전트가 저장소를 바꿀 수 있습니다. |
| `mcp.mutation_allowlist` | `[]` | 비어 있지 않으면 이 도구 이름만 변경할 수 있습니다(`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: 창을 움직이는 `ui_*` 도구를 거부합니다(`ui_state` / `ui_viewers`는 제외). |
| `mcp.ui_control` | `"ask"` | `ask`(창에 확인용 칩이 표시됨), `auto`(바쁘지 않으면 적용), `off`. 서버 명령의 `--ui-control`이 에이전트별로 이를 재정의합니다. `gitpoulpi ui show`에도 적용됩니다. |
| `mcp.allow_ui_propose` | `false` | `ui_propose_*` / `gitpoulpi ui propose`를 허용합니다: 미리 채워진 대화상자를 사용자가 직접 확인합니다. |
| `mcp.allow_history_purge` | `false` | `config.toml`에서만 설정합니다(설정 화면에는 행이 없음). `true`이면 에이전트에게 도구 `rewrite_purge`가 나열됩니다(`allow_mutations`도 여전히 필요). 꺼져 있으면 에이전트에게 도구가 아예 보이지 않습니다. 영구 삭제는 기록 다시 쓰기의 이전 기록과 그 실행 취소, reflog를 완전히 지우기 때문입니다. CLI와 TUI는 이 설정의 제한을 받지 않습니다. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve`와 `gitpoulpi-mcp`가 이 키를 알려 주는 메시지와 함께 시작을 거부합니다. 설정 > 에이전트 및 MCP > *MCP 서버*. 에이전트가 서버를 시작할 때 읽습니다. |
| `mcp.bind` | `"127.0.0.1:8808"` | 주소를 지정하지 않았을 때 `--http`가 수신하는 주소(`--http ADDR` / `--bind`가 우선). `--allow-remote`가 없으면 루프백만 허용됩니다. 비어 있거나 예전 `"stdio"`는 기본값을 뜻합니다. 설정 > 에이전트 및 MCP > *MCP 서버 포트*가 포트를 편집하고 호스트는 유지합니다. |
| `mcp.http_auth` | `true` | `--http`는 `Authorization: Bearer <token>`(`gitpoulpi mcp token`의 토큰)이 있는 요청에만 응답합니다. `false`는 이 검사를 없애며, 이때 서버는 stderr에 경고하고 `--allow-remote`는 거부됩니다. 설정 > 에이전트 및 MCP > *HTTP 인증*. |
| `mcp.tls` | `"off"` | `off`, `self_signed`(비공개 데이터 폴더에 한 번 생성되는 인증서이며 시작할 때 SHA-256 지문을 출력), 또는 `files`(`tls_cert`와 `tls_key`). TLS를 켜면 서버가 직접 TLS를 종료하며 평문 수신기는 존재하지 않습니다. 설정 > 에이전트 및 MCP > *TLS*. |
| `mcp.tls_cert` | `""` | `tls = "files"`일 때 사용하는 PEM 인증서 체인. |
| `mcp.tls_key` | `""` | `tls = "files"`일 때 사용하는 PEM 개인 키. |
| `mcp.behind_proxy` | `false` | 엔드포인트 앞의 터널이나 리버스 프록시가 암호화를 맡는다고 사용자가 밝힙니다. 서버 안에서는 확인할 수 없으므로 TLS 없는 `--allow-remote`에는 이 값이 필요합니다. 프록시와 서버 사이 구간은 해당 네트워크에서 평문입니다. 설정 > 에이전트 및 MCP > *터널 또는 리버스 프록시 뒤*. |
| `mcp.allow_sensitive_staging` | `false` | `config.toml`에만 있습니다. 꺼져 있으면 MCP 스테이징 도구는 `allow_sensitive`를 제공하지 않으며, 표시된 파일(`[privacy]` 참고)을 추가하게 될 스테이징은 에이전트에게 사용자에게 물어보라는 메시지와 함께 거부됩니다. 켜져 있으면 에이전트가 `allow_sensitive`를 전달할 수 있습니다. CLI와 TUI는 이 값의 제약을 받지 않으며 사용자에게 묻습니다. |

`allow_mutations`, `mutation_allowlist`, `allow_ui`는 MCP 서버에만 적용됩니다. CLI는 사용자
자신의 셸이므로 이 설정의 제한을 받지 않습니다. [MCP와 에이전트](mcp-and-agents.md#권한)를
참고하세요.

## `[ai]` (내장 AI, 기본적으로 꺼짐)

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `ai.enabled` | `false` | 스위치입니다. 꺼져 있으면 AI 관련 항목이 어디에도 표시되지 않고 아무것도 전송되지 않습니다. |
| `ai.default_profile` | `"default"` | 동작이 프로필을 지정하지 않을 때 사용하는 프로필의 ID. |
| `ai.global_instructions` | `""` | 모든 동작의 프롬프트에 추가되는 텍스트. `${repo}`, `${uilang}`, ... 를 사용할 수 있습니다([AI > 프롬프트 변수](ai.md#프롬프트-변수) 참고). 동작별 지침과 저장소 지침도 같습니다. |
| `ai.actions.<action>.profile` | `""` | 한 동작의 프로필. 비어 있으면 기본값. 동작: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. 워크스페이스(`workspaces.toml`, `[workspace.ai_instructions]`와 `[workspace.ai_overrides.<action>]`)와 저장소는 이 지침에 추가하거나 이를 대체할 수 있습니다: [AI](ai.md#지침)를 참조하세요. |
| `ai.actions.<action>.instructions` | `""` | 해당 동작에만 추가되는 텍스트. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` 또는 `no`: 코드 리뷰 초안의 로컬 에이전트가 해당 실행에서 AI 프로필의 `workdir`을 무시하고 저장소 안에서 작업할지 여부. 저장소가 고유 값을 가질 수 있습니다(`gitpoulpi ai review-context --repo`). [AI > 리뷰 컨텍스트](ai.md#리뷰-컨텍스트)를 참고하세요. |
| `ai.review.include_claude_md` | `true` | 코드 리뷰 초안에 저장소의 `CLAUDE.md`를 전달합니다(루트, 요청이 대상으로 하는 브랜치에서 읽음, 8 KB에서 잘림). |
| `ai.review.include_agents_md` | `true` | `AGENTS.md`도 같습니다. |
| `ai.review.extra_file` | `""` | 코드 리뷰 초안에 쓸 추가 파일 하나: 저장소 루트 기준 경로, 절대 경로 또는 코드 호스트 파일의 링크(일치하는 연동 프로필로 읽고, 없으면 익명). 모든 파일을 합쳐 16 KB에서 잘립니다. |
| `ai.limits.conflict_bytes` | `40000` | "해결 방법 제안"이 보낼 때 merge 충돌 하나(base, 현재, 들어오는 블록 합계)가 가질 수 있는 최대 바이트 수. 이를 넘으면 GitPoulpi는 실패하지 않고 경고하며 *그래도 보내기*(이번 요청에만), *설정 열기*, *취소*를 제공합니다. 충돌은 절대 잘리지 않습니다. 이 한도는 모델이 아닌 GitPoulpi 자체의 안전장치이므로 모델에 여유가 있으면 늘리세요. 1000~50000000 사이로 유지되며 설정 > AI > 한도에서 바꿀 수 있습니다. 아래의 다른 한도도 같은 방식으로 동작하며 모두 그 그룹에 있습니다. |
| `ai.limits.diff_bytes` | `60000` | commit 메시지, 설명, stash 이름, pull request 설명, commit 계획, commit 분할 요청이 담을 수 있는 diff의 최대 바이트 수이자 코드 리뷰 전체 diff의 최대 바이트 수. 이를 넘어도 조용히 잘리지 않습니다. TUI에서는 *AI 한도 초과*가 *모두 보내기*(이번 요청에만, 자르지 않음), *잘라서 보내기*(이전처럼 끝부분 제외), *설정 열기*, *취소*를 제공하고, CLI와 MCP는 잘라서 `truncated`에 알립니다(`--send-all`은 전부 보냄). 1000~50000000 사이로 유지됩니다. |
| `ai.limits.review_file_bytes` | `12000` | 코드 리뷰에서 파일 하나의 diff의 최대 바이트 수. 이를 넘으면 같은 질문과 같은 두 가지 답이 나오며, *잘라서 보내기*는 그 파일을 눈에 보이는 메모와 함께 줄입니다. 범위는 같습니다. |
| `ai.limits.review_text_bytes` | `4000` | 리뷰 대상 pull request 설명의 최대 바이트 수. 이를 넘으면 *잘라서 보내기*가 설명을 `[text cut]` 메모와 함께 자릅니다. 범위는 같습니다. |
| `ai.limits.review_threads_bytes` | `6000` | 이미 있는 리뷰 댓글 목록(모델이 반복하지 않도록 전달됨)의 최대 바이트 수. 이를 넘으면 *잘라서 보내기*가 목록을 자릅니다. 범위는 같습니다. |
| `ai.limits.review_context_file_bytes` | `8192` | 코드 리뷰에 전달되는 프로젝트 컨텍스트 파일 하나(`CLAUDE.md`, `AGENTS.md`, 추가 파일)의 최대 바이트 수. 이를 넘으면 *잘라서 보내기*가 파일을 메모와 함께 줄입니다. 범위는 같습니다. |
| `ai.limits.review_context_bytes` | `16384` | 모든 컨텍스트 파일 합계의 최대 바이트 수. 이를 넘으면 *잘라서 보내기*가 한도를 넘는 파일을 줄이고 그 뒤의 파일은 제외합니다. 범위는 같습니다. |
| `ai.privacy.redact_secrets` | `true` | 스캐너가 인식하는 모든 비밀 정보(토큰, 개인 키 블록, URL 안의 비밀번호, 값이 무작위로 보이는 `password = ...`)를 요청이 나가기 전에 `[REDACTED:<rule>]`로 대체합니다. 꺼져 있는 동안 설정 > AI에 빨간 경고가 표시됩니다. |
| `ai.privacy.exclude_sensitive_files` | `true` | `privacy.sensitive_patterns`에 일치하는 모든 파일의 diff를 전송 내용에서 제외합니다. 저장소 폴더에서 작업하는 로컬 에이전트는 여전히 파일을 직접 읽을 수 있습니다: [AI > 개인정보](ai.md#개인정보)를 참고하세요. |
| `ai.sent_preview.<action>` | `true` | 동작마다 키 하나(`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): 요청이 나가기 전에 "보내는 내용"을 표시합니다. 대화상자의 "다시 표시하지 않기"는 `false`를 기록하고, 설정 > AI > *보내기 전에 전송 내용 표시...*가 다시 켭니다. |

저장소 고유의 `[preferences.ai_review]` 테이블(`repo-state.toml`에 있으며, 클론되지 않은 저장소의 항목은 `remote-reviews.toml`에 있음)은 같은 키를 모두 선택 사항으로 받고, 추가로 `no_extra_file = true`를 받습니다. 이는 `ai.review.extra_file`이 무엇이든 그 저장소에는 추가 파일이 없다는 뜻입니다. 설정하지 않은 키, 설정하지 않은 `extra_file`, 없는 `no_extra_file`은 전역 값을 상속하며, `extra_file`을 파일로 설정하면 스위치가 지워지고 그 반대도 마찬가지입니다.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness`는 `command`와 별개로 설정 > AI가 제안하는 항목(헤드리스 인수, 옵션, 모델 플래그와
목록, 환경 변수)을 고릅니다. 두 키 모두 비어 있으면 파일에서 생략됩니다. 환경 값은 로그에
기록되지 않으며, 비밀처럼 보이면 설정에서 가려집니다.

설정 > AI는 `harness`/`command`/`args`/`env`(헤드리스 플래그, 모델 플래그, 유용한 환경 변수를 갖춘
일반적인 에이전트 CLI)와 `kind`/`base_url`/`model`(호스팅 및 로컬 HTTP 제공자)의 프리셋을
제공합니다. [AI > 구성](ai.md#ai-구성)을 참고하세요.

프로필이 없으면 *Default*라는 프로필이 만들어집니다. 예전 `[[ai.presets]]` /
`[ai.defaults]` 파일은 로드할 때 마이그레이션되며 `[[ai.legacy_presets]]`에 보관됩니다.
나머지는 [AI 지원](ai.md)을 참고하세요.

## `[repos]`, `[session]`

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `repos.clone_root` | `""` | 클론 대화상자가 시작하는 위치. 비어 있으면 `%USERPROFILE%\source\repos` 또는 `~/src`. |
| `repos.clone_start_source` | `"last"` | 클론 대화상자가 처음 여는 소스: `last`(기본값, 비어 있어도 같음) = 대화상자를 마지막으로 닫았을 때 보이던 소스, `url` = 항상 URL로 클론, 또는 연동 프로필 id(`[[integrations]]` 항목의 `id`). 사라졌거나 비활성이거나 연결되지 않은 프로필은 URL로 클론으로 돌아갑니다. 마지막 소스 자체는 상태이며 여기가 아니라 설정 폴더의 `clone-source.toml`에 저장됩니다. |
| `repos.browse_root` | `""` | 찾아보기가 여는 위치. 비어 있으면 마지막으로 선택한 폴더, 그다음 홈. |
| `repos.show_hidden` | `false` | 폴더 선택기가 숨김 폴더를 표시합니다(`Alt+H`로 전환하고 저장). |
| `repos.show_siblings` | `false` | 폴더 선택기의 왼쪽 트리에 열린 모든 단계의 다른 폴더도 실제 계층으로 표시합니다(`Alt+S`로 전환하고 저장). |
| `session.per_repo_state_dir` | `"in_git_dir"` | GitPoulpi가 저장소 하나에 대해 기억하는 것(ref 필터, 접기 상태, 개인 워크플로 재정의, 호스팅 캐시)을 두는 곳. `in_git_dir` = `<git dir>/gitpoulpi`(모든 worktree가 공유하며 `git status`에 나타나지 않음), `in_worktree` = `<repo>/.gitpoulpi`(팀이 `workflow.toml`을 commit할 수 있는 유일한 곳), `user_data` = `<OS 데이터 폴더>/repos/<name>-<id>`(Windows는 `%APPDATA%\GitPoulpi\data`, Linux는 `$XDG_DATA_HOME/gitpoulpi`, macOS는 `~/Library/Application Support/GitPoulpi`). 예전 값 `".gitpoulpi"`는 계속 `in_worktree`를 뜻합니다. 바꿔도 되돌려 옮기지 않습니다. 예전 `<repo>/.gitpoulpi`에서 찾은 데이터는 처음 필요할 때 선택한 위치로 옮겨집니다. |

## `[log]`(애플리케이션 로그)

활동 로그의 "애플리케이션" 탭이 보여 주는 내용과 "진단 정보 복사"가 인용하는 내용입니다. 자세한 내용: [무언가 실패했을 때](troubleshooting.md#무언가-실패했을-때).

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `log.level` | `"info"` | 기록되는 가장 상세한 줄: `error`, `warn`, `info`, `debug`, `trace`. 활동 로그의 수준 상자가 이 값을 바꿉니다(여기에도 저장됩니다). |
| `log.retention_days` | `7` | 보관할 로그 파일의 일수(오늘 포함): 1에서 3650(10년)까지(범위를 벗어나는 값은 맞춰집니다). 설정이 어떤 단위로 보여 주든 항상 일수입니다. 더 오래된 파일은 GitPoulpi가 시작될 때 삭제됩니다. 하루치 파일도 `log.max_file_mb`로 제한되므로 최악의 경우는 약 `2 x max_file_mb x retention_days`입니다. |
| `log.retention_unit` | `"auto"` | 설정이 `log.retention_days`를 표시하는 단위: `days`, `weeks`, `months`, `years`, `auto`. 표시 전용이며 파일을 보관하는 기간은 바꾸지 않습니다. 1주는 7일, 1개월은 30일(달력상의 한 달이 아니므로 언제 적용해도 같은 일수), 1년은 365일입니다. `auto`와 이 키가 생기기 전에 쓴 설정은 일수를 나누어떨어지게 하는 가장 큰 단위를 고릅니다 (14일은 "2주", 45일은 "45일"). 일수를 나누지 못하는 단위(`retention_days = 10`에 `weeks`)는 무시되고 일수가 표시됩니다. 설정은 두 키를 모두 쓰며, 이전 GitPoulpi는 단위를 무시하고 일수만 읽습니다. |
| `log.max_file_mb` | `5` | 하루치 파일이 이 크기가 되면 `.old` 백업 하나로 회전합니다. 그래서 하루치가 이 크기의 두 배를 넘지 않습니다. |
| `log.max_file_unit` | `"auto"` | 설정이 크기를 표시하는 단위: `mib`, `gib`(1024 MiB), `auto`. 표시 전용이며 크기는 항상 `max_file_mb` MiB(1에서 1024)입니다. |
| `log.timings` | `true` | 활동 로그의 "Git 작업" 탭에 각 작업이 걸린 시간을 표시합니다(`Push - 123 ms`: 저장소 열기, 새로 고침, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). 시간은 창이 측정하며 요청부터 데몬의 응답까지이므로 IPC 왕복을 포함합니다. 끄면 시간과 성공한 작업의 항목을 숨깁니다. 실패는 항상 표시됩니다. |
| `log.extended` | `false` | "확장 로깅": "Git 작업" 탭에 백그라운드 작업(상태를 폴링하는 주기적 새로 고침, 자동 fetch)도 소요 시간과 함께 표시합니다. 이 작업들은 항상 애플리케이션 로그에 `debug` 수준으로 기록됩니다. |
| `log.commit_count` | `"system-git"` | 상태 표시줄의 `불러온 수 / 전체`가 전체 수를 구하는 방법. `"system-git"`은 `git rev-list --count`를 실행합니다(프로세스 하나, 보통의 컴퓨터에서 가장 빠름. git이 commit-graph 파일을 읽습니다). `"libgit2"`는 libgit2 revwalk로 프로세스 안에서 세며 프로세스를 시작하지 않습니다. 실시간 검사를 하는 바이러스 백신 소프트웨어 때문에 모든 명령줄 프로세스의 시작이 느릴 때 고르세요. commit 50,000개에서 12 ms 대신 백그라운드에서 약 200 ms가 걸리지만 프로세스는 시작되지 않습니다. `"off"`는 세지 않으며 바에는 불러온 commit만 표시됩니다. 즉시 적용됩니다. 경로로 걸러서 세는 경우는 항상 시스템 git을 씁니다. |
| `log.use_mailmap` | `true` | Log, blame, 파일 기록에서 각 작성자를 저장소의 `.mailmap`(과 `mailmap.file`)에 적힌 이름으로 표시합니다. 데몬이 git에 `%aN`/`%aE`를 요청하거나(시스템 백엔드) 같은 파일을 직접 적용하므로(libgit2 백엔드) 답은 같습니다. 저장된 commit은 바뀌지 않고 아무것도 다시 쓰이지 않습니다. 작성자로 Log를 검색하면 표시되는 이름에 일치합니다. 즉시 적용되며, Log는 2초 폴링 안에 다시 읽습니다. |

파일은 OS 상태 폴더의 `gitpoulpi-YYYY-MM-DD.log`입니다(Linux는 `~/.local/state/gitpoulpi/logs`, Windows와 macOS는 `<cache>\logs`, `GITPOULPI_CONFIG_DIR`을 쓰면 `<config>/cache/logs`). 토큰, 비밀번호, `Authorization` 헤더는 무엇이든 기록되기 전에 가려집니다.

## `[search]` (워크스페이스 검색)

`gitpoulpi search`(와 그 MCP 도구)가 살펴보는 범위입니다. 자세한 내용: [CLI의 워크스페이스와 검색](cli-reference.md#워크스페이스와-검색).

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `search.commit_limit` | `200000` | 검색 인덱스가 저장소마다 메모리에 보관하는 commit(최신순, 모든 브랜치, 리모트 브랜치, tag 기준)의 메시지 수이며, 각각 약 100바이트에 메시지 길이를 더한 만큼입니다. 더 긴 기록은 거기서 잘리고 응답에 `partial`이라고 표시됩니다. `0`이면 commit 검색을 끕니다(브랜치, tag, stash, worktree는 계속 검색됩니다). 실시간으로 읽습니다: 값을 바꾸면 다음 검색에 적용됩니다. |

## `[rewrite]` (기록 다시 쓰기)

`gitpoulpi rewrite`와 기록 다시 쓰기 워크벤치가 보호하는 것입니다. 자세한 내용: [기록 다시 쓰기](history-rewrite.md).

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | 이미 push된 경우 다시 쓰기가 건드리지 않는 브랜치(이름 또는 `*`가 아무 텍스트와 일치하는 패턴). 실행에서 직접 지정하면(`--include-protected BRANCH`, push에는 `--include-all-protected`, 워크벤치에서는 브랜치별 체크박스) 포함됩니다. 저장소의 `workflow.toml`이 `protected`로 표시한 기준 브랜치에 더해집니다. 다시 쓰기의 push도 같은 방식으로 건너뜁니다. 이것은 GitPoulpi 자체의 보호 장치이며, force push를 받아들일지는 여전히 호스트의 브랜치 보호가 정합니다. 즉시 반영되며, 다음 미리보기에 적용됩니다. |
| `rewrite.external_scripts` | `["shell"]` | 스크립트 규칙이 쓸 수 있는 외부 스크립트 언어: `"shell"`(bash 또는 sh. PowerShell 스크립트는 pwsh 또는 powershell), `"python"`, `"lua"`(설치된 Lua. 내장 Lua는 항상 켜져 있으며 항목이 필요 없음) 중 원하는 것. **외부 스크립트는 샌드박스가 없습니다. 내 권한으로 실행됩니다.** 목록에 없는 언어는 워크벤치에서 회색이 되고 CLI, MCP, 데몬이 거부합니다(`scripts_disabled`). 목록에 있어도 저장소나 공유 레시피에서 온 외부 스크립트는 신뢰한 뒤에만 실행됩니다(`gitpoulpi rewrite script trust FILE` 또는 워크벤치의 질문). 신뢰는 스크립트의 내용 해시를 기준으로 하므로 스크립트가 바뀌면 다시 묻습니다. Python을 켜려면 `config.toml`을 직접 편집합니다(목록은 `config set`으로 설정할 수 없음): `[rewrite]` 아래에 `external_scripts = ["shell", "python"]`. 즉시 반영됩니다. |
| `rewrite.script_timeout_ms` | `5000` | 외부 스크립트가 commit 하나에 답하는 데 주어지는 시간. 초과하면 프로세스 트리 전체가 종료되고 다시 쓰기가 실패합니다(`script_failed`, commit 이름 포함). 아무것도 옮겨지지 않은 상태입니다. `0`은 기본값으로 읽습니다. 즉시 반영됩니다. |
| `rewrite.script_timeout_unit` | `"auto"` | 설정이 제한 시간을 표시하는 단위: `milliseconds`, `seconds`, `auto`. 표시 전용이며 제한은 항상 `script_timeout_ms`밀리초입니다. 설정은 1에서 600000(10분)을 받으며 `0`은 여전히 기본값으로 읽습니다. 즉시 반영됩니다. |
| `rewrite.script_total_timeout_s` | `3600` | 외부 스크립트의 한 번의 실행 전체에 걸릴 수 있는 시간. `0`은 기본값으로 읽습니다. 즉시 반영됩니다. |
| `rewrite.script_total_timeout_unit` | `"auto"` | 설정이 제한 시간을 표시하는 단위: `seconds`, `minutes`, `hours`, `auto`. 표시 전용이며 제한은 항상 `script_total_timeout_s`초입니다. 설정은 1에서 86400(하루)을 받으며 `0`은 여전히 기본값으로 읽습니다. 즉시 반영됩니다. |

## `[updates]` (새 릴리스 확인)

제목 표시줄의 버전 칩이 새 릴리스에 대해 하는 일입니다. 자세한 내용: [업데이트](updates.md).

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"`는 하루에 한 번 백그라운드에서 GitHub에 새 릴리스가 있는지 물어 버전 칩을 `vX.Y.Z available`로 바꿉니다. `"auto"`는 이 빌드가 릴리스 아카이브에서 왔다면 업데이트를 스스로 내려받아 준비해 두기도 하며(재시작하기 전에는 계속 묻습니다), 다른 빌드에서는 `"notify"`처럼 동작합니다. `"off"`는 요청을 전혀 보내지 않습니다. 즉시 적용됩니다. |
| `updates.channel` | `"stable"` | `"stable"`은 정식 릴리스만 대상으로 하고, `"prerelease"`는 릴리스 후보(`-rc.1`)도 알립니다. 즉시 적용됩니다. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | 릴리스를 확인할 github.com의 `owner/name`. 그 저장소가 비공개인 동안에는 github.com용 토큰이 저장되어 있지 않으면(설정 > 연동) GitHub가 404로 답하며, 토큰이 없으면 확인이 조용히 실패하고 애플리케이션 로그에 한 줄만 기록됩니다. |

관리자는 컴퓨터 전체 정책 파일이나 Windows 레지스트리에서 `mode`, `channel`(`release_channel`로), `repo`와 몇 가지 다른 업데이트 옵션을 설정할 수 있습니다. 정책이 설정한 키는 `config.toml`보다 우선하며 설정에서 **관리자가 설정함**으로 표시됩니다. 정책의 `mode`는 상한입니다: 더 낮은 값은 고를 수 있지만 더 높은 값은 고를 수 없습니다. [엔터프라이즈 배포](enterprise-distribution.md#정책-파일과-레지스트리)를 참고하세요.

## `[privacy]` (민감한 파일)

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | gitignore 방식의 경로 패턴. `/`가 없는 패턴은 모든 깊이의 파일 이름에 일치하고, `/`가 있는 패턴은 전체 경로에 일치합니다(앞의 `/`는 루트에 고정). `*`와 `?`는 한 폴더 안에 머물고 `**`는 폴더를 넘나듭니다. `!`는 앞선 패턴이 일치시킨 것을 다시 허용하며 마지막으로 일치한 것이 적용됩니다. 대소문자는 무시합니다. 스테이징 가드와 `ai.privacy.exclude_sensitive_files`가 사용합니다. |
| `privacy.staging_guard` | `true` | 패턴에 일치하거나 내용에 인식된 비밀 정보가 있는 파일을 스테이징하면 먼저 묻고(TUI), `--allow-sensitive`가 없으면 거부되며(CLI), `mcp.allow_sensitive_staging`이 에이전트의 `allow_sensitive` 전달을 허용하지 않으면 거부됩니다(MCP). 텍스트 파일은 각각 최대 1 MiB, 전체 32 MiB까지 읽으며, 바이너리 파일은 이름으로만 일치시킵니다. [스테이징과 commit](staging-and-committing.md#민감한-파일)을 참고하세요. |

## `[conflict_prevention]` (기준 브랜치 목록)

사전 충돌 감지가 현재 브랜치의 기준으로 삼을 수 있는 브랜치는 패턴 목록으로 정합니다([merge 충돌 해결](merge-conflicts.md#충돌-방지) 참조). 목록에는 이름을 붙여 공유할 수 있으며, 설정 > 충돌 방지에서 편집합니다.

| 키 | 기본값 | 값과 효과 |
|---|---|---|
| `conflict_prevention.default_list` | `""` | 자체 목록이 없는 저장소가 사용하는 목록의 이름. 비어 있거나 어느 목록에도 없는 이름이면 기본 제공 `**`(모든 브랜치)입니다. 검사할 때마다 다시 읽으므로 변경은 다시 시작하지 않아도 열려 있는 저장소에 적용됩니다. |
| `conflict_prevention.lists` | 없음 | 이름 붙인 목록, 목록마다 테이블 하나: `[conflict_prevention.lists.release]`와 `patterns = ["origin/release/**", "!origin/release/old"]`. `patterns`가 비어 있으면 모든 브랜치를 허용합니다. 목록을 편집하면 그 목록을 따르는 모든 저장소에 적용됩니다. |

저장소가 어느 목록을 따르는지는 그 `repo-state.toml`(아래)에 있습니다. 우선순위: 저장소 자체의 패턴, 그다음 따르는 이름 붙인 목록, 그다음 `default_list`, 그다음 `**`.



TUI의 단축키를 다시 할당합니다. 설정 > 키보드 및 마우스에서 편집합니다.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`. 값은 그 동작의 기본값을 대체하며, 줄을
제거하면 기본값이 복원됩니다. 키 조합: `Ctrl+Shift+N`, `Alt+n`, `F2`, `PageDown`, `Space`,
`,`, `P`(= `Shift+p`). 잘못된 항목은 아무것도 막지 않습니다. TUI가 이를 무시하고 기본값을
유지하며 상태 표시줄에 알려 줍니다. 컨텍스트, 동작 ID, 기본값, 충돌 규칙은
[키보드 단축키](keybindings.md)에 있습니다.

## 저장소별 설정: `.gitpoulpi/repo-state.toml`

저장소 안에 보관되며, 폴더가 스스로를 무시하므로(`*`만 있는 `.gitignore`) 변경 사항으로
나타나지 않습니다. TUI와 CLI가 기록하며 직접 편집할 일은 드뭅니다.

| 키 | 설명 | 설정 방법 |
|---|---|---|
| `[preferences] encoding` | 여기서 `git.default_encoding`을 재정의 | 설정 > Git 및 인코딩 |
| `[preferences] lfs_enabled` | 여기서 `git.lfs_enabled`를 재정의 | 설정 > Git 및 인코딩 |
| `[preferences] profile` | 여기서 commit에 서명하는 `[[profiles]]` ID. 없으면 `general.active_profile` | 도구 모음의 프로필 콤보박스, **저장소: 프로필 변경...** |
| `[preferences] autocrlf`, `eol` | *아직 사용하지 않음* | 설정 > Git 및 인코딩 |
| `[preferences] push_after_commit` | 각 commit 뒤에 브랜치를 업스트림으로 push (없으면 꺼짐) | 설정 > 저장소별 설정 > Commit |
| `[preferences] skip_hooks_by_default` | commit 양식의 훅 건너뛰기가 선택된 채로 시작 (없으면 꺼짐) | 설정 > 저장소별 설정 > Commit |
| `[preferences] squash_merge` | 로컬 브랜치 merge를 squash: 스테이징만 하고 merge commit을 만들지 않음 (없으면 꺼짐) | 설정 > 저장소별 설정 > Merge 옵션 |
| `[preferences] template_apply`, `template_strip_comments` | 비어 있는 commit 양식을 템플릿으로 미리 채움, commit할 때 `#` 주석 줄 제거 (둘 다 없으면 꺼짐) | 설정 > 저장소별 설정 > 메시지 템플릿 |
| `[preferences] conflict_detection` | 이 저장소의 사전 충돌 감지. 없으면 `general.conflict_detection`을 따름 | 설정 > 저장소별 설정 > 충돌 방지 |
| `[preferences] conflict_base_branches` | 이 저장소 자체의 기준 브랜치 패턴, 한 줄에 하나 (**자체 목록**으로 계속 사용됨) | 설정 > 저장소별 설정 > 충돌 방지 |
| `[preferences] conflict_base_source` | 기준 브랜치의 출처: `default`(전역 기본 목록), `list`(`conflict_base_list`) 또는 `own`(`conflict_base_branches`). 없으면 자체 패턴이 있을 때는 그것을, 아니면 기본값을 사용 | 설정 > 저장소별 설정 > 충돌 방지 |
| `[preferences] conflict_base_list` | 출처가 `list`일 때 이 저장소가 따르는 `[conflict_prevention.lists.<name>]`. 삭제된 목록은 기본값으로 되돌아감 | 설정 > 저장소별 설정 > 충돌 방지 |
| `commit-template.txt` (`repo-state.toml` 옆의 파일) | commit 템플릿. git의 `commit.template`이 이 파일을 가리킴 | 설정 > 저장소별 설정 > 메시지 템플릿 |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: 이 저장소의 AI 지침 | 설정 > 저장소별 설정 > AI |
| `[preferences.ai_overrides.<action>]` | 여기서 한 동작의 `profile`, `instructions` | 설정 > 저장소별 설정 > AI |
| `default_remote` | 원클릭 Push/Pull/Fetch가 사용하는 리모트 | 리모트 메뉴, `gitpoulpi remote default <name>` |
| `hosting_integration` | 여기에 고정된 `[[integrations]]` 프로필 | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | PR / 이슈 목록이 읽는 제공자 프로젝트, `{ integration, project, filter }` 형태. `filter`는 선택 사항입니다: 목록 쿼리에 AND로 붙는 제공자 고유의 추가 절(Jira JQL 조각, Azure WIQL 조건, Trello 목록 이름). GitHub와 GitLab은 무시합니다. | 사이드바 섹션, `gitpoulpi hosting link` |
| `ref_visibility` | 숨기거나 솔로로 지정한 ref | Refs 사이드바, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | 그래프에서 통째로 숨긴 리모트(모든 브랜치, 나중에 fetch하는 것 포함) | 리모트의 눈 아이콘과 메뉴, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | 접힌 사이드바 섹션과 브랜치 및 tag 폴더. `[folds]`에는 `status_folders`(스테이징 안 된 / 스테이징된 파일 트리의 접힌 폴더)와 `commit_folders`(commit의 변경된 파일 트리의 접힌 폴더. 모든 commit에 동일)도 있으며, 각각 500개 경로로 제한되고 오래된 것부터 삭제됩니다 | 사이드바, 파일 트리 |
| `pending_auto_stash` | GitPoulpi가 만들었으나 아직 되돌리지 않은 stash | 데몬이 기록. [자동 stash](log-and-history.md#자동-stash) 참고 |

## 기타 환경 변수

| 변수 | 효과 |
|---|---|
| `GITPOULPI_CONFIG_DIR` | 설정 폴더(위 참고). |
| `GITPOULPI_TOKEN_<ID>` | 저장된 토큰보다 우선하는, 프로필 `<ID>`의 호스팅 토큰(CI, 일회성 사용). |
| `GITPOULPI_GIT` | 실행할 `git` 실행 파일(기본값: `PATH`의 `git`. Windows에서는 Git for Windows의 `cmd\git.exe` 심(shim) 뒤의 실제 `mingw64\bin\git.exe`로, 호출당 프로세스 하나를 아낍니다). |
| `GITPOULPI_LANG=fr` | `ui.language`보다 우선하는 인터페이스 언어(모든 `ui.language` 값. `qps`는 번역가용 의사 로캘). 알 수 없거나 빈 값은 무시됩니다. |
| `GITPOULPI_TERMINAL_IMAGE=off` | 설정과 관계없이 그림을 전혀 사용하지 않습니다. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | 그림 해상도 힌트(셀 크기(픽셀), 오버샘플링. 기본값 2이며 `1`은 CPU를 아낌). |
| `GITPOULPI_NO_BROWSER=1` | 웹 브라우저를 절대 실행하지 않습니다(URL은 여전히 표시됨). |
| `GITPOULPI_TIMING=1` | 요청 소요 시간을 임시 폴더의 `gitpoulpi-timing.log`에 기록합니다(성능 보고용). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | TUI가 오류로 종료한 뒤 Enter를 기다리지 않습니다. |
