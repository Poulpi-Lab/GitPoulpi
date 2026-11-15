[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · **한국어** · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# 키보드 단축키

이 페이지는 **기본** 키를 나열합니다. 대부분의 키는 다시 할당할 수 있습니다. 앱은 현재
적용 중인 키를 항상 보여 줍니다. 명령 팔레트, `?` 도움말, 상태 표시줄, 컨텍스트 메뉴, 버튼
툴팁에서 확인할 수 있습니다. 키를 옮기면 이 모든 곳에 새 키가 표시됩니다. 마우스에 대해서는
[마우스와 제스처](mouse-and-gestures.md)를 참고하세요.

## 키 다시 할당

**설정 > 키보드 및 마우스**를 엽니다. 이 페이지는 [설정](settings.md#키보드-및-마우스)에
설명되어 있습니다. 변경 사항은 즉시 적용되며 `config.toml`의 `[keybindings]` 표에 저장됩니다.
이 표를 직접 편집할 수도 있습니다.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

문법, 충돌 규칙, 경고는 [설정 참조](config-reference.md)의 `[keybindings]` 항목에 있습니다.
요약하면 다음과 같습니다.

- 실수는 상태 표시줄에 보고되며 기본 키가 그대로 유지됩니다.
- 같은 컨텍스트에서 한 키 조합에 두 동작을 지정하면 거부됩니다.
- 접두 키에는 `Ctrl` 또는 `Alt`가 필요합니다.
- 전역 키 조합을 가리는 창 키 조합은 경고만 표시합니다.

아래 표는 다시 할당할 수 있는 모든 키를 나열합니다. 일부 키는 고정되어 있어 다시 할당할 수
없습니다. 목록 이동(`j` `k` `h` `l`, 화살표, `PageUp` / `PageDown`, `Home` / `End`),
`Enter`, `Esc`, `Tab`, 숫자(반복 횟수 접두), 텍스트 편집 키, 팝업의
`Enter` / `Tab` / 화살표 키가 그렇습니다. 팔레트에는 `앱: 단축키...`와
`단축키: 모두 초기화...`가 있습니다.

## 기본 키 바인딩

다시 할당할 수 있는 모든 키를 `[keybindings.<context>]`에서 사용할 ID와 기본값과 함께
나열합니다. 이 표는 앱에서 생성되며(테스트가 최신 상태를 유지합니다), 접두 키 뒤의 키는
접두 키와 함께 표시됩니다(`Ctrl+G s`).

<!-- keymap:begin -->
| 컨텍스트 | 동작 ID | 기능 | 기본 키 |
|---|---|---|---|
| `global` | `git_prefix` | Git 동작 접두 키 | `Ctrl+G` |
| `global` | `nav_prefix` | 탐색 접두 키 | `Ctrl+N` |
| `global` | `palette` | 명령 팔레트 | `Ctrl+P` |
| `global` | `settings` | 설정 | `Ctrl+,` |
| `global` | `home` | 홈 (저장소 관리) | `Ctrl+O` |
| `global` | `clone` | 저장소 클론 | `Ctrl+Shift+N` |
| `global` | `new_tab` | 새 탭 | `Alt+T` |
| `global` | `workspace_switch` | 워크스페이스 전환 | `Alt+W` |
| `global` | `radar` | Radar 열기 | `Alt+R` |
| `global` | `workspaces` | 워크스페이스 열기 | `Alt+J` |
| `global` | `tab_menu` | 탭 메뉴 | `Ctrl+T` |
| `global` | `tab_open_in_popup` | 열기 위치 (목록) | `Alt+O` |
| `global` | `tab_open_in_default` | 기본 위치에서 열기 | `Ctrl+Alt+O` |
| `global` | `close_tab` | 탭 닫기 | `Ctrl+W` |
| `global` | `next_tab` | 다음 탭 | `Ctrl+PageDown` |
| `global` | `prev_tab` | 이전 탭 | `Ctrl+PageUp` |
| `global` | `help` | 도움말 | `?` |
| `global` | `quit` | 종료 | `q` |
| `global` | `refresh` | 새로 고침 | `r` |
| `global` | `refresh_all` | 새로 고침 (fetch, pull request, 이슈) | `F5` |
| `global` | `find_in_log` | Commit 그래프 검색 | `f` |
| `global` | `workspace_search` | 워크스페이스 검색 | `Ctrl+Alt+F` |
| `global` | `select_tab` | 탭 선택 | `t` |
| `global` | `undo` | 마지막 작업 실행 취소 | `Ctrl+Z` |
| `global` | `redo` | 취소한 작업 다시 실행 | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | 실행 취소 기록 열기 | `Alt+Z` |
| `global` | `commit_column` | Commit 열 표시 또는 숨기기 | `Alt+C` |
| `global` | `undo_offer` | 브랜치를 차단된 작업이 남긴 위치로 복원 | `Alt+Shift+Z` |
| `global` | `context_menu` | 컨텍스트 메뉴 | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Commit 메시지 작성 | `Ctrl+G c` |
| `git_prefix` | `commit_now` | 스테이징된 변경 사항 지금 Commit | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | AI: 변경 사항을 commit으로 나누기 | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | AI: 변경 사항 설명 | `Ctrl+G w` |
| `git_prefix` | `stash_message` | 메시지와 함께 Stash | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | AI: Commit 메시지 작성 | `Ctrl+G i` |
| `git_prefix` | `stage_all` | 모든 변경 사항 스테이징 | `Ctrl+G a` |
| `git_prefix` | `discard_all` | 모든 변경 사항 버리기 | `Ctrl+G x` |
| `git_prefix` | `stage_file` | 선택한 파일 스테이징 | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | 선택한 파일 스테이징 해제 | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | 연결된 Pull request 열기 | `Ctrl+G g` |
| `git_prefix` | `push_tags` | 모든 Tag Push | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | 모든 리모트 Fetch | `Ctrl+G f` |
| `git_prefix` | `new_tag` | 새 Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | 새 브랜치 | `Ctrl+G b` |
| `git_prefix` | `stash_push` | 변경 사항 Stash | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | 선택한 Commit Cherry-pick | `Ctrl+G y` |
| `git_prefix` | `revert` | 선택한 Commit 되돌리기 | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Commit 메시지 편집 | `Ctrl+G e` |
| `git_prefix` | `amend` | 선택한 Commit 수정 | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | 선택한 Commit의 작성자 수정 | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | 선택한 Commit으로 리셋 (Mixed) | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | 선택한 브랜치 위로 Rebase | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Stash Pop | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | 선택한 ref 삭제 또는 Stash Drop | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | LFS 메뉴 열기 | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Flow 메뉴 열기 | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: 모든 객체 Fetch | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: 모든 파일 체크아웃 | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: 객체 Pull | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: 모든 객체 Push | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: 로컬 객체 정리 | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | 기록 다시 쓰기 워크벤치 열기 | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Refs 창 접기 또는 복원 | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Status 창 접기 또는 복원 | `Ctrl+N l` |
| `nav_prefix` | `settings` | 설정 열기 | `Ctrl+N ,` |
| `nav_prefix` | `about` | 정보 열기 | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | 활동 로그 열기 | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | 전체 활동 로그 열기 | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | 백그라운드 작업 표시 | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | 업데이트 패널 보기 | `Ctrl+N u` |
| `nav_prefix` | `help` | 도움말 전환 | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | 탭 선택 | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | 테마 변경 | `Ctrl+N y` |
| `nav_prefix` | `change_language` | 언어 변경 | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | 그래프 색상 테마 변경 | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | 그래프 행 색조 전환 | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Refs 섹션 키우기 | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Refs 섹션 줄이기 | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Commit 메시지 영역 키우기 | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Commit 메시지 영역 줄이기 | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Refs 창 넓히기 | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Refs 창 좁히기 | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Status 창 넓히기 | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Status 창 좁히기 | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | 에이전트: 대기 중인 보기 표시 | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | 에이전트: 뒤로 (또는 대기 중인 보기 닫기) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | 열린 대화상자 정리(이동, 크기 조절) | `Ctrl+N m` |
| `refs` | `filter` | 이름으로 Refs 필터 | `/` |
| `refs` | `solo` | 선택 항목만 보기 | `o` |
| `refs` | `clear_solo` | 솔로 해제 | `Shift+O` |
| `refs` | `toggle_hierarchy` | 브랜치 계층 보기 전환 | `y` |
| `refs` | `show_all` | 숨긴 Ref 모두 표시 | `Shift+H` |
| `refs` | `reveal` | 그래프에서 보기 | `g` |
| `refs` | `mark_drop` | 드롭 소스로 표시 | `m` |
| `refs` | `drop_marked` | 표시한 ref를 여기에 놓기 | `Shift+M` |
| `refs` | `expand_all` | 모든 폴더 펼치기 | `=` |
| `refs` | `collapse_all` | 모든 폴더 접기 | `-` |
| `refs` | `expand_below` | 폴더와 하위 폴더 펼치기 | `*`, `Shift+Right` |
| `refs` | `collapse_below` | 폴더와 하위 폴더 접기 | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | 파일: 경로 또는 트리 | `m` |
| `log` | `view_all_files` | 파일: 모든 파일 보기 | `a` |
| `log` | `compare_selection` | 선택한 행 비교 | `v` |
| `log` | `compare_swap` | 비교 양쪽 바꾸기 | `x` |
| `log` | `compare_next` | 다음 비교 행으로 이동 | `n` |
| `log` | `pick_column` | 이동하거나 크기를 바꿀 열 고르기 | `Shift+C` |
| `log` | `scroll_left` | 열을 왼쪽으로 스크롤 | `Shift+Left` |
| `log` | `scroll_right` | 열을 오른쪽으로 스크롤 | `Shift+Right` |
| `log` | `copy_mermaid` | Mermaid gitGraph로 Commits 복사 | `Shift+Y` |
| `log_columns` | `next` | 다음 열 고르기 | `Tab` |
| `log_columns` | `prev` | 이전 열 고르기 | `Shift+BackTab` |
| `log_columns` | `move_left` | 열을 왼쪽으로 이동 | `<` |
| `log_columns` | `move_right` | 열을 오른쪽으로 이동 | `>` |
| `log_columns` | `wider` | 열 넓히기 | `]` |
| `log_columns` | `narrower` | 열 좁히기 | `[` |
| `log_columns` | `reset_width` | 열 너비 초기화 | `0` |
| `log_columns` | `fit` | 열을 내용에 맞추기 | `=` |
| `log_columns` | `fit_all` | 모든 열을 내용에 맞추기 | `+` |
| `log_columns` | `layout` | 옆으로 스크롤 / 창에 맞춤 | `s` |
| `log_files` | `collapse_all` | 모든 폴더 접기 또는 펼치기 | `z` |
| `log_files` | `filter` | 변경된 파일 필터 | `/` |
| `log_files` | `expand_all` | 모든 폴더 펼치기 | `=` |
| `log_files` | `collapse_all_folders` | 모든 폴더 접기 | `-` |
| `log_files` | `expand_below` | 폴더와 하위 폴더 펼치기 | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | 폴더와 하위 폴더 접기 | `_`, `Shift+Left` |
| `status` | `compose` | Commit 메시지로 포커스 | `c` |
| `status` | `toggle_file_mode` | 파일: 경로 또는 트리 | `m` |
| `status` | `view_all_files` | 파일: 모든 파일 보기 | `a` |
| `status` | `amend` | Amend 전환 | `Shift+A` |
| `status` | `skip_hooks` | 훅 건너뛰기 전환 | `Shift+H` |
| `status` | `collapse_all` | 모든 폴더 접기 또는 펼치기 | `z` |
| `status` | `stage_toggle` | 선택한 행 스테이징 또는 해제 | `Ctrl+Space` |
| `status` | `open_diff` | 선택한 파일의 Diff 열기 | `d` |
| `status` | `expand_all` | 모든 폴더 펼치기 | `=` |
| `status` | `collapse_all_folders` | 모든 폴더 접기 | `-` |
| `status` | `expand_below` | 폴더와 하위 폴더 펼치기 | `*`, `Shift+Right` |
| `status` | `collapse_below` | 폴더와 하위 폴더 접기 | `_`, `Shift+Left` |
| `home` | `favorite` | 저장소 즐겨찾기 추가 또는 해제 | `f`, `*` |
| `home` | `close_repo` | 행 닫기, 제거 또는 즐겨찾기 해제 | `x`, `Delete` |
| `home` | `browse` | 저장소 찾아보기 | `b` |
| `home` | `clone` | 저장소 클론 | `c` |
| `home` | `init` | 저장소 초기화 | `i` |
| `home` | `collapse_all` | 모든 섹션 접기 | `-` |
| `home` | `expand_all` | 모든 섹션 펼치기 | `+`, `=` |
| `home` | `search` | 저장소 검색 | `/` |
| `home` | `workspace_new` | 새 워크스페이스 | `n` |
| `home` | `workspace_edit` | 워크스페이스 편집 | `e` |
| `home` | `workspace_from_tabs` | 열린 탭에서 새 워크스페이스 | `a` |
| `home` | `workspace_fetch` | 워크스페이스 Fetch All | `Shift+F` |
| `home` | `workspace_pull` | 워크스페이스 Pull All | `Shift+P` |
| `home` | `workspace_clone_missing` | 누락된 저장소 클론 | `m` |
| `home` | `workspace_locate` | 누락된 저장소 위치 지정 | `l` |
| `refs` | `hosting_filter` | Pull request / 이슈: 필터 선택 | `v` |
| `refs` | `hosting_edit` | Pull request: 호스트에서 편집 | `e` |
| `refs` | `new_pull_request` | Pull request: 새로 만들기 | `n` |
| `refs` | `add_remote` | 리모트: 리모트 추가 | `+` |
| `refs` | `remotes` | 브랜치: 리모트 편집 | `u` |
| `diff` | `close` | Diff 닫기 | `q` |
| `diff` | `edit` | 작업 트리에서 편집 | `e` |
| `diff` | `lfs_raw` | LFS: 카드 또는 원본 포인터 | `r` |
| `diff` | `copy` | 선택 영역 또는 줄 복사 | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Hunk 스테이징 또는 해제 | `Space` |
| `diff` | `stage_lines` | 선택한 줄 스테이징 또는 해제 | `Enter` |
| `diff` | `discard_hunk` | Hunk 버리기 | `x` |
| `diff` | `discard_lines` | 선택한 줄 버리기 | `Shift+X` |
| `diff` | `select_lines` | 줄 선택 시작 또는 해제 | `v` |
| `diff` | `next_hunk` | 다음 Hunk | `n` |
| `diff` | `prev_hunk` | 이전 Hunk | `p` |
| `diff` | `toggle_layout` | 레이아웃: 통합 또는 나란히 | `m` |
| `diff` | `toggle_hunks_only` | 변경 부분만 또는 전체 파일 | `Shift+H` |
| `diff` | `toggle_minimap` | 미니맵 전환 | `Shift+M` |
| `diff` | `choose_encoding` | 인코딩 선택 | `Shift+C` |
| `diff` | `toggle_wrap` | 자동 줄바꿈 전환 | `w` |
| `diff` | `toggle_whitespace` | 공백만 바뀐 변경 표시 또는 숨기기 | `s` |
| `diff` | `merge_editor` | merge 편집기 열기 (충돌한 파일) | `Shift+E` |
| `file_insight` | `close` | 파일 보기 닫기 | `q` |
| `file_insight` | `edit` | 작업 트리에서 편집 | `e` |
| `file_insight` | `copy` | 선택 영역 또는 행 복사 | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | 마지막 행으로 이동 | `Shift+G` |
| `file_insight` | `open_commit` | 행의 Commit으로 이동 | `Enter` |
| `history` | `show_in_log` | Log에서 보기 | `Enter` |
| `history` | `copy_sha` | commit SHA 복사 | `y` |
| `history` | `copy_summary` | 요약 복사 | `Shift+Y` |
| `history` | `show_diff` | 이 commit의 diff 표시 | `d`, `Alt+1` |
| `history` | `open_file` | 이 commit 시점의 파일 열기 | `o`, `Alt+2` |
| `history` | `blame` | 이 commit에서 Blame | `b`, `Alt+3` |
| `history` | `compare_working_tree` | 작업 트리와 비교 | `c` |
| `history` | `restore` | 이 버전 복원 | `Shift+R` |
| `history` | `range_up` | 선택을 위로 확장 | `Shift+Up` |
| `history` | `range_down` | 선택을 아래로 확장 | `Shift+Down` |
| `history` | `switch_focus` | 목록과 diff 사이 전환 | `Tab` |
| `history` | `widen_list` | 목록 넓히기 | `Alt+Right` |
| `history` | `narrow_list` | 목록 좁히기 | `Alt+Left` |
| `history` | `reset_split` | 목록 너비 초기화 | `Alt+0` |
| `commit_column` | `scroll_left` | 열을 왼쪽으로 스크롤 | `Shift+Left` |
| `commit_column` | `scroll_right` | 열을 오른쪽으로 스크롤 | `Shift+Right` |
| `commit_column` | `next_part` | 다음 부분 선택 | `Tab` |
| `commit_column` | `move_left` | 부분을 왼쪽으로 이동 | `<` |
| `commit_column` | `move_right` | 부분을 오른쪽으로 이동 | `>` |
| `commit_column` | `wider_part` | 부분 넓히기 | `]` |
| `commit_column` | `narrower_part` | 부분 좁히기 | `[` |
| `commit_column` | `reset_widths` | 부분 너비 초기화 | `0` |
| `commit_column` | `fit` | 열을 내용에 맞추기 | `=` |
| `commit_column` | `fit_all` | 모든 열을 내용에 맞추기 | `+` |
| `file_insight` | `blame_card` | Blame: 상세 카드 표시 또는 숨기기 | `i` |
| `file_insight` | `blame_next` | Blame: 다음 Hunk | `n` |
| `file_insight` | `blame_prev` | Blame: 이전 Hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: 같은 commit의 다음 Hunk | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: 같은 commit의 이전 Hunk | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: 행으로 이동 | `g` |
| `file_insight` | `blame_copy_sha` | Blame: commit SHA 복사 | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: 이 commit의 이 파일 diff 표시 | `d` |
| `file_insight` | `blame_parent` | Blame: 이 commit의 부모에서 Blame | `b` |
| `file_insight` | `blame_history` | Blame: 파일 기록 열기 | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: 주석 열 넓히기 | `>` |
| `file_insight` | `blame_narrower` | Blame: 주석 열 좁히기 | `<` |
| `image` | `zoom_in` | 이미지 확대 | `+`, `=` |
| `image` | `zoom_out` | 이미지 축소 | `-` |
| `image` | `fit` | 이미지를 보기에 맞춤 | `0` |
| `image` | `actual_size` | 이미지를 실제 크기로 표시 | `1` |
| `image` | `flip_side` | 반대쪽 보기 (이전 / 이후) | `Tab` |
| `image` | `widen_before` | 이전 창 넓히기 | `Alt+Right` |
| `image` | `widen_after` | 이후 창 넓히기 | `Alt+Left` |
| `image` | `reset_split` | 이전 / 이후 분할 초기화 | `Alt+0` |
| `image` | `edit_in` | 연결 프로그램... | `Shift+O` |
| `image` | `edit_default` | 기본 앱으로 열기 | `o` |
| `editor` | `save` | 저장 | `Ctrl+S` |
| `editor` | `undo` | 실행 취소 | `Ctrl+Z` |
| `editor` | `redo` | 다시 실행 | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | 찾기 | `Ctrl+F` |
| `editor` | `find_next` | 다음 찾기 (Shift: 이전) | `F3` |
| `editor` | `paste` | 붙여넣기 | `Ctrl+V` |
| `editor` | `toggle_minimap` | 미니맵 전환 | `Alt+M` |
| `editor_prompt` | `save` | 저장하지 않은 변경 사항: 저장 후 닫기 | `s`, `Shift+S` |
| `editor_prompt` | `discard` | 저장하지 않은 변경 사항: 버리고 닫기 | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | 저장하지 않은 변경 사항: 계속 편집 | `c`, `Shift+C` |
| `merge` | `next_conflict` | 다음 충돌 | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | 이전 충돌 | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | 다음 충돌 파일 | `Alt+PageDown` |
| `merge` | `prev_file` | 이전 충돌 파일 | `Alt+PageUp` |
| `merge` | `next_pane` | 다음 창으로 포커스 | `F6` |
| `merge` | `prev_pane` | 이전 창으로 포커스 | `Shift+F6` |
| `merge` | `toggle_panes` | 사이드 패널 표시 또는 숨기기 (Refs 및 Status) | `F9` |
| `merge` | `toggle_base` | Base 창 표시 또는 숨기기 | `Alt+D` |
| `merge` | `toggle_full_file` | 전체 파일 diff 전환 | `Alt+F` |
| `merge` | `undo` | 실행 취소 (선택 및 입력) | `Ctrl+Z` |
| `merge` | `redo` | 다시 실행 (선택 및 입력) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | 현재 (A) 창 넓히기 | `Alt+Right` |
| `merge` | `widen_incoming` | 수신 (B) 창 넓히기 | `Alt+Left` |
| `merge` | `reset_panes` | 창 크기 초기화 | `Alt+0` |
| `merge` | `save` | 해결됨 표시 없이 저장 | `Alt+S` |
| `merge` | `save_resolve` | 저장 및 해결됨으로 표시 | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ 해결 방법 제안 (AI) | `Alt+I` |
| `merge` | `unpick` | 이 충돌의 선택 해제 (미해결로 되돌림) | `Alt+U` |
| `merge` | `reset_all` | 파일 전체를 원래 충돌로 초기화 | `Alt+R` |
| `merge` | `focus_a` | 현재 (A) 창에 포커스 | `Alt+1` |
| `merge` | `focus_b` | 수신 (B) 창에 포커스 | `Alt+2` |
| `merge` | `focus_output` | 출력 창에 포커스 | `Alt+3` |
| `merge` | `focus_base` | Base 창에 포커스 | `Alt+4` |
| `merge_sides` | `toggle_line` | 줄 선택 또는 해제 | `Space` |
| `merge_sides` | `toggle_chunk` | 덩어리 전체 선택 또는 해제 | `Enter` |
| `merge_sides` | `take_a` | 현재 (A)만 선택 | `a` |
| `merge_sides` | `take_b` | 수신 (B)만 선택 | `b` |
| `merge_sides` | `take_both` | 둘 다 선택, A 다음 B | `c` |
| `merge_sides` | `take_both_reversed` | 둘 다 선택, B 다음 A | `Shift+C` |
| `merge_sides` | `clear` | 이 충돌에서 아무것도 선택 안 함 | `x` |
| `merge_sides` | `unpick` | 이 충돌의 선택 해제 (미해결로 되돌림) | `u` |
| `merge_sides` | `reset_all` | 파일 전체를 원래 충돌로 초기화 | `Shift+U` |
| `merge_sides` | `swap_order` | 두 쪽의 순서 바꾸기 | `s` |
| `merge_sides` | `next_conflict` | 다음 충돌 | `n` |
| `merge_sides` | `prev_conflict` | 이전 충돌 | `Shift+N` |
| `merge_sides` | `take_all_a` | 현재 (A) 전체 선택 | `Shift+A` |
| `merge_sides` | `take_all_b` | 수신 (B) 전체 선택 | `Shift+B` |
| `merge_sides` | `toggle_base` | Base 창 표시 또는 숨기기 | `d` |
| `merge_sides` | `scroll_left` | 창 왼쪽으로 스크롤 | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | 창 오른쪽으로 스크롤 | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | 전체 파일 diff 전환 | `f` |
| `merge_sides` | `cycle_height` | 출력 높이 순환 | `z` |
| `merge_card` | `take_a` | A (현재) 그대로 유지 | `a` |
| `merge_card` | `take_b` | B (수신) 그대로 선택 | `b` |
| `merge_card` | `take_base` | Base (공통 조상) 선택 | `o` |
| `merge_card` | `keep` | 남은 파일 유지 | `k` |
| `merge_card` | `delete` | 삭제 수락 | `Shift+D` |
| `merge_card` | `external_tool` | 외부 merge 도구로 열기 | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (Amend를 위해 중지) | `e` |
| `rebase` | `autosquash` | Autosquash 전환 | `a` |
| `rebase` | `start` | 계획 시작 | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | 계획 초기화 | `Ctrl+R` |
| `rebase` | `split` | 행의 commit 분할 | `t` |
| `rebase` | `move_up` | 행 위로 이동 | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | 행 아래로 이동 | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Commit 만들기 | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | 줄 선택 전환 | `v` |
| `split` | `move_up` | commit 위로 이동 | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | commit 아래로 이동 | `Alt+Down`, `Shift+J` |
| `split` | `reset` | 분할 초기화 | `Shift+R` |
| `rewrite` | `preview` | 다시 쓰기 미리보기 | `F5` |
| `rewrite` | `apply` | 다시 쓰기 적용 | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | 미리보기 취소 | `Ctrl+X` |
| `rewrite` | `add_rule` | 규칙 추가 | `a`, `Insert` |
| `rewrite` | `toggle_rule` | 규칙 켜기/끄기 | `Space` |
| `rewrite` | `move_up` | 규칙 위로 이동 | `Alt+Up` |
| `rewrite` | `move_down` | 규칙 아래로 이동 | `Alt+Down` |
| `rewrite` | `delete_rule` | 규칙 삭제 | `d`, `Delete` |
| `rewrite` | `restore_rule` | 삭제한 규칙 복원 | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | 규칙 복제 | `Ctrl+D` |
| `rewrite` | `next_tab` | 다음 미리보기 탭 | `Alt+Right` |
| `rewrite` | `prev_tab` | 이전 미리보기 탭 | `Alt+Left` |
| `rewrite` | `save_shared` | 레시피를 저장소에 저장 | `Alt+S` |
| `rewrite` | `save_mine` | 레시피를 나만 쓰도록 저장 | `Alt+M` |
| `rewrite` | `load` | 레시피 불러오기 | `Alt+L` |
| `rewrite` | `name` | 레시피 이름 변경 | `F2` |
| `rewrite` | `protected_all` | 보호된 브랜치 모두 포함 | `Alt+A` |
| `rewrite` | `protected_none` | 보호된 브랜치 모두 제외 | `Alt+U` |
| `rewrite` | `check_tools` | 도구 다시 확인 | `F6` |
| `rewrite` | `analyze` | 기록 크기 분석 | `F7` |
| `rewrite` | `host_howto` | 호스트의 브랜치 보호를 통과하는 방법 설명 | `Alt+H` |
| `rewrite` | `ai_rules` | AI로 다시 쓰기 규칙 만들기 | `Alt+I` |
| `reword` | `apply` | 새 메시지 적용 | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | 기록을 더 깊이 수정 | `Alt+H` |
| `composer` | `commit` | 작성기에서 Commit | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | 이슈 참조 삽입 | `Alt+R` |
| `md_editor` | `preview` | Markdown Editor: Toggle Write and Preview | `Alt+P` |
| `md_editor` | `split` | Markdown Editor: Toggle the Split View | `Alt+L` |
| `md_editor` | `submit` | Markdown Editor: Send the Text | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Markdown Editor: Write with AI | `Alt+I` |
| `dialog` | `browse` | 폴더 찾아보기 | `Ctrl+B` |
| `dialog` | `sources` | 클론 소스로 이동 | `Alt+S` |
| `dialog` | `submit` | Pull request 양식 제출 | `Ctrl+S` |
| `dialog` | `web` | 제공자의 웹 양식 열기 | `Alt+W` |
| `folder_picker` | `choose` | 폴더 선택 | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | 새 폴더 | `Alt+N` |
| `folder_picker` | `filter` | 폴더 항목 필터링 | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | 형제 폴더 표시 또는 숨기기 | `Alt+S` |
| `folder_picker` | `toggle_hidden` | 숨김 폴더 표시 또는 숨기기 | `Alt+H` |
| `preferences` | `search` | 설정 검색 | `Ctrl+F`, `/` |
| `preferences` | `list_add` | 목록 항목 추가 | `Insert` |
| `preferences` | `list_duplicate` | 선택한 목록 항목 복제 | `Ctrl+D` |
| `preferences` | `list_remove` | 선택한 목록 항목 제거 | `Delete` |
| `preferences` | `list_move_up` | 선택한 목록 항목 위로 이동 | `Alt+Up` |
| `preferences` | `list_move_down` | 선택한 목록 항목 아래로 이동 | `Alt+Down` |
| `preferences` | `prev_tab` | 설정 페이지의 이전 탭 | `Alt+Left` |
| `preferences` | `next_tab` | 설정 페이지의 다음 탭 | `Alt+Right` |
| `ai_popup` | `copy` | 결과 복사 | `c` |
| `activity_log` | `scroll_left` | 활동 로그 왼쪽으로 스크롤 | `Left` |
| `activity_log` | `scroll_right` | 활동 로그 오른쪽으로 스크롤 | `Right` |
| `integrations` | `add` | 프로필 추가 | `a` |
| `integrations` | `edit` | 선택한 프로필 편집 | `e` |
| `integrations` | `open_token` | 토큰 페이지 열기 | `o` |
| `integrations` | `remove` | 선택한 프로필 제거 | `d`, `Delete` |
| `prompt` | `ai_stash_message` | AI: Stash 메시지 생성 | `Ctrl+G` |
<!-- keymap:end -->

### 터미널이 보낼 수 없는 키와 kitty 키보드 프로토콜

레거시 터미널은 일부 키 조합을 보낼 수 없습니다. `Ctrl+,`, `Ctrl+Shift+N`, `Ctrl+Enter`,
`Ctrl+<숫자>`가 그렇습니다. 아무것도 보내지 않거나 다른 키를 보냅니다. 예를 들어
`Ctrl+Shift+N`은 `Ctrl+N`으로, `Ctrl+I`는 `Tab`으로, `Ctrl+M`은 `Enter`로 도착합니다. 설정
페이지는 이런 바인딩에 대해 경고합니다("this terminal does not send ...").

**kitty 키보드 프로토콜**을 지원하는 터미널은 이 키 조합을 보낼 수 있습니다. 예: kitty,
WezTerm, Ghostty, foot, Alacritty 0.13 이상, 그리고 zellij(tmux는 응답하지 않음:
[멀티플렉서](multiplexers.md#tmux) 참고). GitPoulpi는 기본적으로 이 프로토콜을 사용합니다.

- 시작할 때 GitPoulpi는 터미널에 질의하고 응답을 최대 약 100 ms 기다립니다. 터미널이
  응답하지 않으면 레거시 키가 그대로 유지됩니다.
- 터미널이 응답하면 GitPoulpi는 프로토콜을 켭니다. 종료하거나 실패하거나 충돌할 때
  다시 끕니다.
- 설정 페이지의 경고는 프로토콜을 따릅니다. 프로토콜이 켜져 있으면 위의 키 조합을 보낼 수
  있습니다.
- **설정 > 키보드 및 마우스 > Kitty 키보드 프로토콜**(`ui.kitty_keyboard`)을 `off`로 설정하면
  터미널의 키보드 모드를 전혀 바꾸지 않습니다. 터미널이나 멀티플렉서가 오작동할 때
  사용하세요. 다시 시작해야 합니다.

Windows 콘솔은 수정 키를 직접 보고하므로 프로토콜이 필요하지 않습니다.

## 명령 팔레트 (`Ctrl+P`)

팔레트는 앱의 **모든** 명령을 나열하므로 단축키 없이도 모든 기능에 닿을 수 있습니다. 전체
목록은 [아래](#모든-팔레트-명령)에 있습니다.

| 키 | 동작 |
|---|---|
| `Ctrl+P` | 팔레트 열기 |
| *(입력)* | 목록 필터링. `Category: ` 접두 뒤의 정확히 일치하는 항목이 먼저, 그다음 레이블 어디서든 일치하는 항목, 그다음 퍼지 일치 순입니다 |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | 강조 표시 이동([필터 가능한 팝업 목록](#필터-가능한-팝업-목록) 참고) |
| `Enter` 또는 더블 클릭 | 강조 표시된 명령 실행 |
| `Esc` | 닫기 |

`j`와 `k`는 필터에 입력됩니다. 강조 표시를 움직이지 않습니다.

각 행은 오른쪽에 단축키를 보여 줍니다. 지금 적용할 수 없는 명령(예: 선택된 ref가 없음)은
목록에 남아 회색으로 표시되며, 이유는 목록 아래에 나타납니다. 레이블은 `탐색:`,
`Refs:`, `Git:`, `보기:`, `Diff:`, `앱:` 같은 카테고리로 시작합니다. 카테고리를 입력하면
그 그룹 전체를 볼 수 있습니다.

### 모든 명령에 키 지정

팔레트의 각 행 끝에는 키 버튼(`[k]`)이 있고, 강조된 행에서 `Alt+K`를 눌러도 같습니다. 해당 명령의 단축키로 필터링된 **설정 > 키보드 및 마우스**가 열려 바로 다시 지정할 수 있습니다. 전용 단축키가 없는 명령도 여기서 하나를 받습니다. **팔레트 명령** 아래에 나열되고 기본값은 지정 없음이며, 지정한 키는 팔레트에서 선택하는 것과 똑같이 명령을 실행합니다(지금 실행할 수 없는 명령은 이유를 알려 줍니다). 이후 그 행에 지정한 키가 표시됩니다.

`config.toml`에서는 `[keybindings.palette]` 테이블이며 명령 이름으로 지정합니다(`toggle_minimap = "Ctrl+Alt+M"`). 다른 동작과 충돌하는 키는 빨간색으로 표시되고, 그 행에서 `c`를 누르면 충돌하는 모든 동작이 나열됩니다.

목록 아래 도움말 줄에 이 키(기본값 `Alt+K`)가 표시됩니다. 이 키는 `palette.configure_shortcut` 동작이므로 다른 동작처럼 다시 지정할 수 있습니다.

팔레트와 선택기(테마, 언어, 탭, 작업 공간 검색)는 대화상자처럼 이동하고 크기를 바꿀 수 있습니다. 위쪽 테두리를 드래그하거나, (모든 대화상자에서처럼) `Ctrl+N` 다음 `m`을 누르거나, `Alt+G`를 누르세요. `Alt+G`는 동작 `palette.arrange`("대화상자: 맨 앞 대화상자 이동 또는 크기 조절")이며, 모든 대화상자와 팝업 위에서 맨 앞의 것에 정리 모드를 시작하는 추가 방법입니다(팔레트의 `대화상자: 이동 또는 크기 조절`, `대화상자: 위치와 크기 초기화` 행은 팔레트가 열려 있을 때 팔레트 자체에 적용됩니다). 화살표는 이동, `Shift`+화살표는 크기 조절, `0`은 초기화이며, `Enter` 또는 `Esc`는 모드를 종료하고 팝업은 열린 채로 남습니다. 위치를 유지할지는 `ui.dialog_geometry`가 정합니다.

`Ctrl+N`은 여기서도 탐색 접두 키입니다. 필터 상자, 목록의 첫 행, 닫기 십자에 배지를 표시하고, 뒤따르는 키가 해당 컨트롤을 누르거나 `m`으로 팝업을 정리합니다. 더 이상 강조 표시를 아래로 옮기지 않으며 `Ctrl+P`도 위로 옮기지 않습니다. 필터 필드가 있는 목록은 화살표, `PageUp` / `PageDown`, `Home` / `End`만 움직입니다.

### 모든 팔레트 명령

앱에서 생성되었습니다(테스트가 최신 상태를 유지합니다). AI 명령은 AI가 켜져 있을 때만
나타나며, 나머지는 적용할 수 없을 때 이유와 함께 회색으로 표시됩니다. 키는 기본값입니다.

<!-- palette:begin -->
| 명령 | 기본 키 |
|---|---|
| 탐색: Refs 창으로 포커스 | `Ctrl+N <badge>` |
| 탐색: Log 창으로 포커스 | `Ctrl+N <badge>` |
| 탐색: Status 창으로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs 필터로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs > 로컬로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs > 리모트로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs > 워크트리로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs > 서브모듈로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs > Stashes로 포커스 | `Ctrl+N <badge>` |
| 탐색: Refs > Tags로 포커스 | `Ctrl+N <badge>` |
| 왼쪽 창: 섹션 표시/숨기기... |  |
| 왼쪽 창: 섹션을 기본값으로 초기화 |  |
| 왼쪽 창: 이 섹션 설정을 모든 저장소에 사용 |  |
| 왼쪽 창: 로컬 브랜치 표시/숨기기 |  |
| 왼쪽 창: 리모트 표시/숨기기 |  |
| 왼쪽 창: Worktree 표시/숨기기 |  |
| 왼쪽 창: 워크트리 섹션 표시 |  |
| 왼쪽 창: Stashes 표시/숨기기 |  |
| 왼쪽 창: Tags 표시/숨기기 |  |
| 왼쪽 창: Pull requests 표시/숨기기 |  |
| 왼쪽 창: 이슈 표시/숨기기 |  |
| 왼쪽 창: 서브모듈 표시/숨기기 |  |
| 탐색: Status > 스테이징 안 됨으로 포커스 | `Ctrl+N <badge>` |
| 탐색: Status > 스테이징된으로 포커스 | `Ctrl+N <badge>` |
| 탐색: 다음 창으로 포커스 이동 | `Tab` |
| 탐색: 이전 창으로 포커스 이동 | `Shift+Tab` |
| 탐색: Commit 검색... | `f` |
| 탐색: Refs 창 접기 전환 | `Ctrl+N h` |
| 탐색: Refs 창 넓히기 | `Ctrl+N Right` |
| 탐색: Refs 창 좁히기 | `Ctrl+N Left` |
| 탐색: Status 창 넓히기 | `Ctrl+N Shift+Left` |
| 탐색: Status 창 좁히기 | `Ctrl+N Shift+Right` |
| 탐색: Status 창 접기 전환 | `Ctrl+N l` |
| 탐색: Refs 섹션 키우기 | `Ctrl+N +` |
| 탐색: Refs 섹션 줄이기 | `Ctrl+N -` |
| 탐색: Refs 섹션 크기 초기화 |  |
| 탐색: 레이아웃 초기화 |  |
| Commit 상세: 메시지 키우기 | `Ctrl+N ]` |
| Commit 상세: 메시지 줄이기 | `Ctrl+N [` |
| Commit 상세: 메시지 크기 초기화 |  |
| Commit 상세: 메시지 아래로 스크롤 |  |
| Commit 상세: 메시지 위로 스크롤 |  |
| Commit 상세: 메시지의 링크 열기... |  |
| 파일: Commit 파일 정렬 전환 (A-Z / Z-A) |  |
| Refs: 선택한 Ref 체크아웃 | `더블 클릭` |
| Refs: Ref 이름 복사 |  |
| Refs: 전체 Ref 복사 (refs/heads/...) |  |
| Refs: Ref 대상 SHA 복사 |  |
| Refs: 선택한 브랜치를 현재 브랜치에 Merge |  |
| Refs: 현재 브랜치를 선택한 브랜치 위로 Rebase | `Ctrl+G r` |
| Refs: 선택한 Ref를 체크아웃한 브랜치에 놓기... |  |
| Refs: 드롭 소스로 표시 | `m` |
| Refs: 표시한 ref를 여기에 놓기... | `M` |
| Refs: 선택한 브랜치 이름 변경... |  |
| Refs: 선택한 브랜치를 현재 위치로 Fast-Forward |  |
| Refs: 현재 브랜치를 선택한 위치로 Fast-Forward |  |
| Refs: 선택한 브랜치 또는 Tag에서 워크트리 만들기... |  |
| Refs: 선택한 브랜치의 Reflog... |  |
| Refs: 선택한 브랜치에서 Pull request 시작... |  |
| Refs: 워크트리를 탭으로 열기 | `Enter / 더블 클릭` |
| Refs: 워크트리를 열 위치... |  |
| Refs: 워크트리 브랜치를 다른 브랜치로 merge... |  |
| Refs: 워크트리 제거... |  |
| Refs: 워크트리 만들기... | `+` |
| Refs: 현재 브랜치에서 워크트리 만들기 |  |
| Refs: 브랜치를 골라 워크트리 만들기 |  |
| Refs: 서브모듈 업데이트 (선택) |  |
| Refs: 서브모듈 초기화 (선택) |  |
| Refs: 서브모듈을 탭으로 열기 | `Enter / 더블 클릭` |
| Git: 모든 서브모듈 업데이트 |  |
| Git: 선택한 Commit에 워크트리 만들기... |  |
| Git: 선택한 Commit에서 패치 만들기... |  |
| Git: 선택한 Commit부터 HEAD까지 패치 만들기... |  |
| Git: 작업 트리 변경 사항에서 패치 만들기... |  |
| 다시 쓰기: 이전 기록 영구 삭제... |  |
| Git: 패치 적용... |  |
| 다시 쓰기: 기록 워크벤치 | `Ctrl+G H` |
| 다시 쓰기: 작성자 바꾸기... |  |
| 다시 쓰기: 작성자 익명화... |  |
| 다시 쓰기: 기록에 .mailmap 적용... |  |
| 다시 쓰기: 작성자 정보 Merge (.mailmap)... |  |
| 다시 쓰기: Commit 날짜 이동... |  |
| 다시 쓰기: Commit 시간대 변경... |  |
| 다시 쓰기: Commit 시각을 허용 시간 안으로... |  |
| 다시 쓰기: 커미터 날짜를 작성자 날짜로 설정 |  |
| 다시 쓰기: 메시지 찾아 바꾸기... |  |
| 다시 쓰기: 메시지 줄 삭제... |  |
| 다시 쓰기: 트레일러 추가/제거... |  |
| 다시 쓰기: 서명 제거... |  |
| 다시 쓰기: 메시지 인코딩 수정... |  |
| 다시 쓰기: 파일 내용 찾아 바꾸기... |  |
| 다시 쓰기: 기록에서 파일 제거... |  |
| 다시 쓰기: 기록에서 큰 파일 제거... |  |
| 다시 쓰기: 경로 이동/이름 변경... |  |
| 다시 쓰기: 파일을 LFS로 마이그레이션... |  |
| 다시 쓰기: LFS에서 파일 되돌리기... |  |
| 다시 쓰기: 줄 끝 변환... |  |
| 다시 쓰기: 실행 비트 제거... |  |
| 다시 쓰기: 파일 인코딩 변환... |  |
| 다시 쓰기: 스크립트 실행... |  |
| ✦ 다시 쓰기: AI로 스크립트 작성... |  |
| ✦ 다시 쓰기: AI로 규칙 만들기... |  |
| 다시 쓰기: 선택한 Commit부터 HEAD까지... |  |
| 다시 쓰기: 선택한 Commits... |  |
| LFS: 이 저장소에 초기화 |  |
| LFS: 패턴 추적... |  |
| LFS: 패턴 추적 해제... |  |
| LFS: 객체 Fetch |  |
| LFS: 메뉴 열기 | `Ctrl+G l` |
| LFS: 모든 객체 Fetch | `Ctrl+G F` |
| LFS: 모든 파일 체크아웃 | `Ctrl+G K` |
| LFS: 모든 객체 Push | `Ctrl+G J` |
| LFS: 객체 Pull | `Ctrl+G L` |
| LFS: 객체 Push |  |
| LFS: 로컬 객체 정리 | `Ctrl+G V` |
| LFS: 잠금 표시 |  |
| LFS: 설치 페이지 열기 |  |
| LFS: diff의 원본 포인터 전환 | `r` |
| LFS: 선택한 파일 추적 |  |
| LFS: 선택한 파일의 확장자 추적 |  |
| LFS: 선택한 파일 잠금 |  |
| LFS: 선택한 파일 잠금 해제 |  |
| LFS: 선택한 파일 강제 잠금 해제 |  |
| Refs: 선택한 브랜치 또는 Tag 삭제... | `Ctrl+G d` |
| Refs: 선택한 브랜치를 리모트에서 삭제... |  |
| Refs: 선택한 브랜치를 로컬과 리모트에서 삭제... |  |
| Refs: 리모트의 선택한 브랜치 링크 복사 |  |
| Git: 리모트의 선택한 Commit 링크 복사 |  |
| Git: 선택한 Commit에 주석 Tag... |  |
| Refs: 선택한 Tag를 기본 리모트로 Push |  |
| Refs: 선택한 Tag를 리모트에서 삭제... |  |
| Git: 선택한 Tag Fetch |  |
| Refs: 선택한 Tag 주석 달기... |  |
| Refs: 그래프에서 숨기기 전환 |  |
| Refs: 그래프에서 보기 | `g` |
| Refs: 선택한 Ref 솔로 전환 | `o` |
| Refs: 솔로 해제 (모든 솔로 Ref 해제) | `O` |
| Refs: 숨긴 Ref 모두 표시 | `H` |
| Refs: 선택한 폴더/리모트 숨기기 전환 |  |
| Refs: 선택한 폴더/리모트 솔로 전환 | `o` |
| Refs: 이름으로 필터... | `/` |
| Refs: 이름 필터 지우기 | `Esc` |
| Refs: 모든 폴더 펼치기 | `=` |
| Refs: 모든 폴더 접기 | `-` |
| Refs: 선택한 폴더와 하위 폴더 펼치기 | `*` |
| Refs: 선택한 폴더와 하위 폴더 접기 | `_` |
| Refs: 다음 섹션 | `l` |
| Refs: 이전 섹션 | `h` |
| Refs: 활성 섹션 펼치기 또는 접기 | `Enter` |
| Refs: 브랜치 계층 보기 전환 | `y` |
| Git: 선택한 파일 스테이징 | `Ctrl+G s` |
| Git: 선택한 파일 스테이징 해제 | `Ctrl+G u` |
| Git: 모든 변경 사항 스테이징 | `Ctrl+G a` |
| 작업 변경: 선택한 파일 버리기... |  |
| 작업 변경: 선택한 파일 무시 |  |
| 작업 변경: 선택한 파일의 확장자 무시 |  |
| 작업 변경: 선택한 파일을 바이너리로 처리... |  |
| 작업 변경: 선택한 파일을 텍스트로 처리... |  |
| 작업 변경: 선택한 파일의 확장자를 바이너리로 처리... |  |
| 작업 변경: 선택한 파일의 확장자를 텍스트로 처리... |  |
| 작업 변경: 선택한 파일 Stash |  |
| 작업 변경: 선택한 파일 기록 |  |
| 작업 변경: 선택한 파일 Blame |  |
| 작업 변경: 선택한 파일을 외부 diff 도구로 열기 |  |
| 작업 변경: 선택한 파일을 외부 편집기로 열기 |  |
| 작업 변경: 선택한 파일을 기본 프로그램으로 열기 |  |
| 작업 변경: 선택한 파일을 폴더에서 보기 |  |
| 작업 변경: 선택한 파일 경로 복사 |  |
| 작업 변경: 선택한 파일에서 패치 만들기... |  |
| 작업 변경: 선택한 파일 편집 |  |
| 작업 변경: 선택한 파일 삭제... |  |
| 작업 변경: 선택한 폴더 메뉴 열기 |  |
| 작업 변경: 파일 만들기 |  |
| 작업 변경: 모든 변경 사항 버리기... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: 스테이징된 변경 사항 지금 Commit | `Ctrl+G Enter` |
| Git: WIP Commit 메시지 편집 (Log에서) | `Log에서 c` |
| Git: 모든 리모트 Fetch | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (가능하면 fast-forward) |  |
| Git: Pull (fast-forward만) |  |
| Git: Pull (rebase) |  |
| Git: Pull 옵션... |  |
| Git: Push | `Ctrl+G P` |
| Git: 리모트로 Push... |  |
| Git: 선택한 리모트로 Push... |  |
| Git: 모든 대상으로 Push |  |
| Git: 모든 소스에서 Pull |  |
| Git: 리모트 하나에서 Pull... |  |
| Git: 그룹 Push 보기 |  |
| Git: 그룹 Push 계속 |  |
| Git: 그룹 Push 중단 |  |
| Git: 실패한 Push 재시도 |  |
| Git: 강제 Push (lease 사용)... |  |
| Git: 브랜치의 리모트... |  |
| 리모트: 리모트 추가... |  |
| 리모트: 리모트 제거... |  |
| 리모트: 리모트 편집... |  |
| 리모트: 기본 리모트 설정... |  |
| 리모트: 리모트 이름 복사 |  |
| 리모트: 리모트 링크 복사 |  |
| 리모트: 리모트 Fetch... |  |
| 리모트: 선택한 리모트의 동작... | `.` |
| 저장소: 스파스 체크아웃... |  |
| 저장소: SSH 및 자격 증명... |  |
| 저장소: 연결 테스트 |  |
| 저장소: 기본 자격 증명 사용 |  |
| Flow: 메뉴 열기 | `Ctrl+G W` |
| Flow: 브랜치 시작... |  |
| Flow: 새 이슈... |  |
| Flow: 호스트 브랜치 이어받기... |  |
| Flow: 현재 브랜치 완료... |  |
| Flow: 부모와 동기화... |  |
| Flow: 승격... |  |
| Flow: 부모 설정... |  |
| Flow: 중단된 동작 계속 또는 중단... |  |
| Flow: 브랜치 묶기 전환 |  |
| Flow: 워크플로 설정... |  |
| 리모트: 그래프에서 리모트 숨기기... |  |
| 리모트: 그래프에 리모트 표시... |  |
| Git: HEAD에 새 브랜치... | `Ctrl+G b` |
| Git: HEAD에 새 Tag... | `Ctrl+G t` |
| Git: push하지 않은 모든 Tag Push | `Ctrl+G U` |
| Git: 모든 변경 사항 Stash | `Ctrl+G T` |
| Git: 메시지와 함께 모든 변경 사항 Stash... | `Ctrl+G m` |
| Git: Stash Pop | `Ctrl+G o` |
| Git: Stash Apply |  |
| Git: Stash Drop... |  |
| Git: 선택한 Commit을 HEAD에 Cherry-pick | `Ctrl+G y` |
| Git: 선택한 Commit 체크아웃 (분리된 HEAD) |  |
| Git: 선택한 Commit에 브랜치 만들기... |  |
| Git: 선택한 Commit에 Tag 만들기... |  |
| Commit: 메시지 복사 |  |
| Commit: 작성자 이름 복사 |  |
| Commit: 작성자 이메일 복사 |  |
| Commit: 날짜 복사 |  |
| Commit: 짧은 SHA 복사 |  |
| Commit: 전체 SHA 복사 |  |
| Git: 선택한 Commit 되돌리기... | `Ctrl+G v` |
| Git: Commit 메시지 편집... | `Ctrl+G e` |
| Commit: 수정... | `Ctrl+G E` |
| Commit: 작성자 수정... | `Ctrl+G A` |
| Commit: 요약 수정... |  |
| Commit: 설명 수정... |  |
| Git: 선택한 Commit 분할... |  |
| Git: 선택한 Commit부터 대화형 Rebase... |  |
| Rebase: 계속 |  |
| Rebase: 건너뛰기 |  |
| Rebase: 중단 |  |
| git am: 계속 |  |
| git am: 이 패치 건너뛰기 |  |
| git am: 중단 |  |
| 저장소: 오래된 index.lock 삭제 |  |
| 배너: 대기 중인 자동 stash Pop |  |
| 배너: 대기 중인 자동 stash 완료 (결과 유지) |  |
| 배너: 대기 중인 자동 stash 보기 |  |
| 충돌: 예상 충돌 표시 |  |
| 배너: 대기 중인 자동 stash Drop... |  |
| 배너: 처음부터 다시... |  |
| Git: 브랜치를 선택한 Commit으로 리셋 (soft - 인덱스와 작업 트리 유지) |  |
| Git: 브랜치를 선택한 Commit으로 리셋 (mixed - 작업 트리 유지) | `Ctrl+G R` |
| Git: 브랜치를 선택한 Commit으로 리셋 (hard - 모두 버림)... |  |
| Git: Ours로 충돌 해결 (이 브랜치) |  |
| Git: Theirs로 충돌 해결 (수신) |  |
| Git: Base로 충돌 해결 (공통 조상) |  |
| 충돌: merge 편집기 열기 | `충돌한 파일에서 Enter` |
| merge: 다음 충돌 | `F7` |
| merge: 이전 충돌 | `Shift+F7` |
| merge: 다음 충돌 파일 | `Alt+PageDown` |
| merge: 이전 충돌 파일 | `Alt+PageUp` |
| merge: A에서 덩어리 선택 |  |
| merge: B에서 덩어리 선택 |  |
| merge: 이 충돌에서 현재 선택 | `a` |
| merge: 이 충돌에서 수신 선택 | `b` |
| merge: 둘 다 선택, A 다음 B | `c` |
| merge: 둘 다 선택, B 다음 A | `C` |
| merge: 이 충돌에서 Base 선택 |  |
| merge: 이 충돌 선택 지우기 | `x` |
| merge: 순서 바꾸기 | `s` |
| merge: 현재 전체 선택 | `A` |
| merge: 수신 전체 선택 | `B` |
| merge: 이 충돌을 출력에서 편집 |  |
| merge: 사이드 패널 표시 또는 숨기기 | `F9` |
| merge: 창 왼쪽으로 스크롤 | `Shift+Left` |
| merge: 창 오른쪽으로 스크롤 | `Shift+Right` |
| merge: Base 창 전환 | `Alt+D` |
| Merge: 전체 파일 diff 전환 | `Alt+F` |
| merge: 실행 취소 | `Ctrl+Z` |
| merge: 다시 실행 | `Ctrl+Y` |
| merge: 해결됨 표시 없이 저장 | `Alt+S` |
| merge: 저장 및 해결됨으로 표시 | `Ctrl+S` |
| merge: 출력을 merge 결과로 초기화 | `Alt+R` |
| merge: 출력 높이 순환 | `z` |
| merge: 현재 창 넓히기 | `Alt+Right` |
| merge: 수신 창 넓히기 | `Alt+Left` |
| merge: 창 크기 초기화 | `Alt+0` |
| merge: 이 충돌의 선택 해제 | `Alt+U` |
| merge: 현재 (A) 창에 포커스 | `Alt+1` |
| merge: 수신 (B) 창에 포커스 | `Alt+2` |
| merge: 출력 창에 포커스 | `Alt+3` |
| merge: Base 창에 포커스 | `Alt+4` |
| merge: 왼쪽 창으로 이동 (같은 충돌) |  |
| merge: 오른쪽 창으로 이동 (같은 충돌) |  |
| merge: 다음 창으로 포커스 | `F6` |
| merge: 이전 창으로 포커스 | `Shift+F6` |
| merge: 디스크의 파일로 계속 |  |
| merge: merge 결과부터 다시 시작 |  |
| merge: 해결 취소 후 다시 열기 |  |
| merge: 파일 새로 고침 |  |
| merge: 현재 파일을 그대로 사용 | `a` |
| merge: 수신 파일을 그대로 사용 | `b` |
| merge: Base 파일을 그대로 사용 | `o` |
| merge: 남은 파일 유지 | `k` |
| merge: 삭제 수락 | `D` |
| merge: 외부 merge 도구로 열기 | `m` |
| ✦ AI: merge 충돌 해결 방법 제안 | `Alt+I` |
| merge: merge 편집기 닫기 |  |
| Git: Merge, Cherry-pick 또는 되돌리기 중단 |  |
| Git: Cherry-pick 또는 되돌리기 건너뛰기 |  |
| 저장소: 열기 또는 전환 (저장소 관리)... | `Ctrl+O` |
| 저장소: 저장소 찾아보기... | `b (관리 탭)` |
| 저장소: 새로 만들기 (초기화)... | `i (관리 탭)` |
| 저장소 목록: 선택 항목 열기 | `Enter` |
| 저장소 목록: 선택 항목 즐겨찾기 추가 또는 제거 | `f` |
| 저장소 목록: 선택 항목 닫기 또는 제거 | `x` |
| 저장소 목록: 선택한 저장소 메뉴... | `.` |
| 저장소 목록: 검색 | `/ / Ctrl+F` |
| 저장소 목록: 모든 섹션 접기 | `-` |
| 저장소 목록: 모든 섹션 펼치기 | `+` |
| 저장소 목록: 모든 탭 닫기... |  |
| 저장소 목록: 선택 항목을 목록에서 제거 |  |
| 저장소 목록: 최근 항목 모두 제거 |  |
| 워크스페이스: 새로 만들기... | `n` |
| 워크스페이스: 열린 탭에서 새로 만들기... | `a` |
| 워크스페이스: 전환... | `Alt+W` |
| 워크스페이스: 편집... | `e` |
| 워크스페이스: 닫기 (워크스페이스 없음) |  |
| 워크스페이스: 관리... | `Alt+J` |
| 워크스페이스: 탭 닫기 |  |
| 워크스페이스: 누락된 저장소 위치 지정... | `l` |
| 워크스페이스: 삭제... |  |
| 워크스페이스: Fetch All | `F` |
| 워크스페이스: Pull All | `P` |
| 워크스페이스: 누락된 저장소 클론 | `m` |
| 워크스페이스: 탭을 워크스페이스에 추가 |  |
| 워크스페이스: 탭을 워크스페이스에서 제거 |  |
| Radar: 열기 | `Alt+R` |
| Radar: 새로 고침 | `F5` |
| Radar: 검색 | `/` |
| Radar: 브라우저에서 열기 | `o` |
| Radar: 저장소 열기 | `Enter` |
| Radar: 브랜치 체크아웃 | `c` |
| Radar: worktree 만들기... | `t` |
| Radar: 저장소에서 표시 | `s` |
| Radar: 이슈 작업 시작... | `n` |
| Radar: 고정 또는 해제 | `p` |
| Radar: 미루기... | `z` |
| Radar: 요청 리뷰... | `e` |
| Radar: Open Issue Conversation | `i` |
| Radar: API로 리뷰 |  |
| Radar: 클론 후 리뷰... |  |
| Radar: 담당자 편집... |  |
| Radar: 리뷰어 편집... |  |
| Radar: 레이블 편집... |  |
| Radar: 마일스톤 설정... |  |
| Radar: 범위 선택 | `w` |
| Radar: 보기 선택 | `v` |
| Radar: 필터 선택 | `f` |
| Radar: 클론하지 않은 요청 표시 | `a` |
| Radar: 초안 표시 | `d` |
| Radar: 계정 전체 리뷰 요청 켜기... | `T` |
| Radar: 행 메뉴... | `.` |
| Radar: 열 선택 | `C` |
| Radar: 열 넓히기 | `]` |
| Radar: 열 좁히기 | `[` |
| Radar: 열 너비 초기화 | `0` |
| Radar: 열을 내용에 맞추기 | `=` |
| Radar: 모든 열을 내용에 맞추기 | `+` |
| Radar: 홈으로 돌아가기 | `Esc` |
| Radar: 탭 닫기 |  |
| 탭: 새 탭 | `Alt+T` |
| 탭: 새로 만들기... |  |
| 탭: 동작 메뉴... | `Ctrl+T` |
| 탭: 다른 탭 닫기... |  |
| 탭: 오른쪽 탭 닫기... |  |
| 탭: 즐겨찾기 추가 또는 제거 |  |
| 탭: 저장소 경로 복사 |  |
| 탭: 파일 관리자에서 열기 |  |
| 탭: 터미널에서 열기 |  |
| 탭: 열기 위치... | `Alt+O` |
| 탭: 기본 위치에서 열기 | `Ctrl+Alt+O` |
| 저장소: 프로필 변경... |  |
| 도움말: GitPoulpi 정보 | `Ctrl+N a` |
| 도움말: 라이선스 및 크레딧 |  |
| 보기: 활동 로그 | `Ctrl+N A` |
| 보기: 전체 활동 로그 | `Ctrl+N G` |
| 도움말: 릴리스 노트 |  |
| 업데이트: 업데이트 패널 보기 | `Ctrl+N u` |
| 업데이트: 보안 공지 열기 |  |
| 업데이트: 업데이트 명령 복사 |  |
| 업데이트: 터미널에서 업데이트 명령 실행 |  |
| 업데이트: 지금 업데이트 확인 |  |
| 업데이트: 다운로드 |  |
| 업데이트: 지금 업데이트 |  |
| 업데이트: 재시작하여 업데이트 |  |
| 업데이트: 이전 버전으로 되돌리기 |  |
| 업데이트: 이 버전 건너뛰기 |  |
| 릴리스 노트: 탭 닫기 |  |
| 작업: 백그라운드 작업 표시 | `Ctrl+N J` |
| 작업: 취소할 수 있는 것을 모두 취소 |  |
| 보기: 진단 정보 복사 |  |
| 활동 로그: 오류 수준 표시/숨기기 |  |
| 활동 로그: 경고 수준 표시/숨기기 |  |
| 활동 로그: 정보 수준 표시/숨기기 |  |
| 활동 로그: 디버그 수준 표시/숨기기 |  |
| 활동 로그: 트레이스 수준 표시/숨기기 |  |
| 활동 로그: 모든 수준 표시 |  |
| 활동 로그: 수준 표시 안 함 |  |
| 활동 로그: 애플리케이션 로그 지우기 |  |
| 활동 로그: 범위 선택 |  |
| 활동 로그: Git 작업 |  |
| 활동 로그: 애플리케이션 로그 |  |
| 활동 로그: 이 줄의 작업 표시 |  |
| 활동 로그: 다음 실패한 작업 |  |
| 활동 로그: 이전 실패한 작업 |  |
| 활동 로그: 왼쪽으로 스크롤 | `Left` |
| 활동 로그: 오른쪽으로 스크롤 | `Right` |
| 활동 로그: 마지막 오류 복사 |  |
| 활동 로그: 제안된 작업 실행 |  |
| 활동 로그: 마지막 오류 열기 |  |
| 탭: 종료 후 여기로 cd |  |
| 저장소: 클론... | `Ctrl+Shift+N` |
| 저장소: 다음 탭 | `Ctrl+PageDown` |
| 저장소: 이전 탭 | `Ctrl+PageUp` |
| 탭: 왼쪽으로 이동 |  |
| 탭: 오른쪽으로 이동 |  |
| 저장소: 탭 닫기 | `Ctrl+W` |
| 저장소: 탭 선택... | `t` |
| 검색: 워크스페이스... | `Ctrl+Alt+F` |
| 편집: 실행 취소 | `Ctrl+Z` |
| 편집: 다시 실행 | `Ctrl+Y` |
| 편집: 막힌 실행 취소 항목 잊기 |  |
| 편집: 실행 취소 기록... | `Alt+Z` |
| 편집: Reflog... |  |
| 편집: 잃어버린 Commits 복구... |  |
| 편집: 브랜치를 차단된 작업이 남긴 위치로 복원 | `Alt+Shift+Z` |
| 리뷰: 개요 표시 |  |
| 리뷰: 파일 표시 |  |
| 리뷰: Commits 표시 |  |
| 리뷰: 줄에 댓글 달기... | `c` |
| 리뷰: 대화에 댓글 달기... | `C` |
| 리뷰: 스레드에 답글... | `r` |
| 리뷰: 스레드 해결 또는 다시 열기 | `x` |
| 리뷰: 스레드 접기 또는 펼치기 | `z` |
| 리뷰: 대기 중인 댓글 편집... | `e` |
| 리뷰: 대기 중인 댓글 삭제 | `d` |
| 리뷰: 파일을 확인함으로 표시 | `v` |
| 리뷰: 다음 파일 | `]` |
| 리뷰: 이전 파일 | `[` |
| 리뷰: 리뷰 완료... | `f` |
| 리뷰: 대기 중인 리뷰 폐기... |  |
| 리뷰: 새로 고침 | `F5` |
| 리뷰: 브라우저에서 열기 | `o` |
| 리뷰: 브랜치 체크아웃 | `b` |
| 리뷰: 닫기 | `Esc` |
| 리뷰: 담당자 편집... | `a` |
| 리뷰: 리뷰어 편집... | `p` |
| 리뷰: 레이블 편집... | `l` |
| 리뷰: 마일스톤 설정... | `m` |
| 리뷰: 패치 세트 선택... | `P` |
| 리뷰: 패치 세트 비교... | `D` |
| 리뷰: 변경 Submit... | `U` |
| 리뷰: 변경 Abandon... | `Q` |
| 리뷰: 인라인 / 분할 레이아웃 전환 | `V` |
| 리뷰: 파일 전체 / 변경 부분만 전환 | `H` |
| 리뷰: 공백만 바뀐 변경 전환 | `S` |
| 리뷰: 자동 줄바꿈 전환 | `W` |
| 리뷰: 다음 hunk | `J` |
| 리뷰: 이전 hunk | `K` |
| 리뷰: 인코딩 선택... | `E` |
| 리뷰: 미니맵 전환 | `M` |
| 리뷰: Diff 왼쪽으로 스크롤 | `Left` |
| 리뷰: Diff 오른쪽으로 스크롤 | `Right` |
| 리뷰: 선택한 텍스트 또는 줄 복사 | `Ctrl+C` |
| 보드: 카드 이동... | `m` |
| 보드: 카드를 왼쪽으로 이동 |  |
| 보드: 카드를 오른쪽으로 이동 |  |
| 보드: 카드 열기 | `Enter` |
| 보드: 담당자 편집... |  |
| 보드: 레이블 편집... |  |
| 보드: 마일스톤 설정... |  |
| 보드: 이 열에 새 이슈 | `n` |
| 보드: 담당자로 필터링... | `a` |
| 보드: 레이블로 필터링... | `l` |
| 보드: 마일스톤으로 필터링... | `M` |
| 보드: 텍스트로 필터링 | `/` |
| 보드: 필터 지우기 | `x` |
| 보드: 열 레이블 설정... | `c` |
| 보드: 새로 고침 | `r` |
| 보드: 닫기 | `Esc` |
| Conversation: Back |  |
| Conversation: Refresh | `F5` |
| Conversation: Load Older Entries | `m` |
| Conversation: Previous Entry | `Up` |
| Conversation: Next Entry | `Down` |
| Conversation: Next Unread Entry | `n` |
| Conversation: Reply to Entry | `r` |
| Conversation: New Comment | `c` |
| Conversation: Links of Entry... | `l` |
| Conversation: Copy Link of Entry | `y` |
| Conversation: Open in Browser | `o` |
| Conversation: Edit Comment | `e` |
| Conversation: Delete Comment | `d` |
| Conversation: Show or Hide Events | `v` |
| Conversation: Toggle Write and Preview |  |
| Conversation: Toggle the Split View |  |
| Conversation: Send the Comment |  |
| Conversation: Close | `Esc` |
| Hosting: Open Issue Conversation... |  |
| 리뷰: ✦ 리뷰 초안 작성... | `s` |
| 리뷰: 제안 수락 | `y` |
| 리뷰: 제안 편집 후 수락... | `w` |
| 리뷰: 제안 무시 | `n` |
| 리뷰: 모든 제안 수락 | `A` |
| 리뷰: 모든 제안 무시 | `N` |
| 리뷰: 리뷰 텍스트에 AI 요약 사용 | `i` |
| 리뷰: ✦ 초안 사용... | `u` |
| 리뷰: ✦ 초안 텍스트 편집... | `t` |
| 리뷰: ✦ 초안 폐기 | `N` |
| 실행 취소 기록: 여기까지 실행 취소... | `u` |
| 실행 취소 기록: 여기까지 다시 실행... | `y` |
| 실행 취소 기록: 이것만 복원... | `r` |
| 실행 취소 기록: 브랜치를 원래 위치로 복원... | `o` |
| 실행 취소 기록: 새로 고침 |  |
| 실행 취소 기록: 닫기 | `Esc` |
| Reflog: Log에서 보기 | `l` |
| Reflog: 여기에 브랜치 만들기... | `b` |
| Reflog: 여기에 tag 만들기... | `t` |
| Reflog: 변경 사항을 작업 트리에 적용 | `w` |
| Reflog: 변경 사항을 stash(으)로 저장 | `s` |
| Reflog: 이 위치로 복원... | `r` |
| Reflog: 더 불러오기 | `m` |
| Reflog: 새로 고침 |  |
| 잃어버린 작업: 여기에 브랜치 만들기... | `b` |
| 잃어버린 작업: 여기에 tag 만들기... | `t` |
| 잃어버린 작업: Cherry-pick | `c` |
| 잃어버린 작업: 변경 사항을 작업 트리에 적용 | `w` |
| 잃어버린 작업: 변경 사항을 stash(으)로 저장 | `s` |
| 잃어버린 작업: 다시 검사 |  |
| Reflog / 잃어버린 작업: 다음 창 | `Tab` |
| Reflog / 잃어버린 작업: 경로 또는 트리 파일 목록 | `v` |
| Reflog / 잃어버린 작업: 창 크기 초기화 |  |
| 대화상자: 이동 또는 크기 조절 | `Ctrl+N m` |
| 대화상자: 위치와 크기 초기화 |  |
| 보기: 새로 고침 | `r` |
| 보기: 새로 고침 (fetch, pull request, 이슈) | `F5` |
| 보기: 도움말 오버레이 전환 | `?` |
| 보기: Ref 열 전환 |  |
| 보기: 그래프 열 전환 |  |
| 보기: 메시지 열 전환 |  |
| 보기: 작성자 열 전환 |  |
| 보기: 날짜 열 전환 |  |
| 보기: 변경 열 전환 |  |
| 보기: SHA 열 전환 |  |
| 보기: Log 열: 이동하거나 크기를 바꿀 열 고르기 | `C` |
| 보기: Log 열: 다음 열 고르기 | `Tab` |
| 보기: Log 열: 이전 열 고르기 | `Shift+BackTab` |
| 보기: Log 열: 열을 왼쪽으로 이동 | `<` |
| 보기: Log 열: 열을 오른쪽으로 이동 | `>` |
| 보기: Log 열: 열 넓히기 | `]` |
| 보기: Log 열: 열 좁히기 | `[` |
| 보기: Log 열: 열 너비 초기화 | `0` |
| 보기: Log 열: 열을 내용에 맞추기 | `=` |
| 보기: Log 열: 모든 열을 내용에 맞추기 | `+` |
| 보기: Log: 열을 왼쪽으로 스크롤 | `Shift+Left` |
| 보기: Log: 열을 오른쪽으로 스크롤 | `Shift+Right` |
| 보기: Log: 열 옆으로 스크롤 전환 | `s` |
| 보기: Log: 표시할 열 선택 |  |
| Diff: 선택한 파일의 Diff 열기 | `Enter` |
| Diff: Diff 보기 닫기 | `Esc` |
| Diff: 다음 Hunk | `n` |
| Diff: 이전 Hunk | `p` |
| Diff: 커서 위치의 Hunk 스테이징 | `Space` |
| Diff: 커서 위치의 Hunk 스테이징 해제 | `Space` |
| Diff: 줄 선택 (또는 선택 해제) | `v` |
| Diff: 선택한 줄 스테이징 | `Enter` |
| Diff: 선택한 줄 스테이징 해제 | `Enter` |
| Diff: 커서 위치의 Hunk 버리기... | `x` |
| Diff: 선택한 줄 버리기... | `X` |
| 복사: 선택한 텍스트 (먼저 텍스트를 드래그) | `Ctrl+C` |
| 복사: 커서 위치의 줄 (또는 선택한 줄) | `y` |
| Diff: 커서 위치의 Hunk 복사 |  |
| Diff: 인라인 레이아웃 (한 열에 +/-) | `m` |
| Diff: 분할 레이아웃 (변경 전후) | `m` |
| Diff: 변경 부분만 전환 (전체 파일 / 변경만) | `H` |
| 파일: 경로/트리 전환 | `m` |
| 파일: 모든 파일 보기 전환 | `a` |
| Commit: 훅 건너뛰기 전환 | `H` |
| 파일: 행 또는 선택 항목의 메뉴 열기 | `.` |
| 파일: 모든 폴더 펼치기 | `=` |
| 파일: 모든 폴더 접기 | `-` |
| 파일: 선택한 폴더와 하위 폴더 펼치기 | `*` |
| 파일: 선택한 폴더와 하위 폴더 접기 | `_` |
| 파일: 모두 선택 |  |
| 파일: 선택 해제 |  |
| 파일: 현재 행 선택 전환 |  |
| Log: Mermaid gitGraph로 복사... | `Y` |
| Log: 선택 항목 비교 | `v` |
| Log: 비교 양쪽 바꾸기 | `x` |
| Log: 다음 비교 행으로 이동 | `n` |
| Diff: 자동 줄바꿈 전환 | `w` |
| Diff: 인코딩 선택... | `C` |
| 보기: 미니맵 전환 | `M` |
| 보기: Commit 열 전환 | `Alt+C` |
| 보기: Commit 열: 숨기기 |  |
| 보기: Commit 열: 파일 기록 |  |
| 보기: Commit 열: 모든 Commits |  |
| 보기: Commit 열에 포커스 |  |
| 보기: Commit 열 넓히기 |  |
| 보기: Commit 열 좁히기 |  |
| 보기: Commit 열: 왼쪽으로 스크롤 | `Shift+Left` |
| 보기: Commit 열: 오른쪽으로 스크롤 | `Shift+Right` |
| 보기: Commit 열: 다음 부분 선택 | `Tab` |
| 보기: Commit 열: 부분을 왼쪽으로 이동 | `<` |
| 보기: Commit 열: 부분을 오른쪽으로 이동 | `>` |
| 보기: Commit 열: 부분 넓히기 | `]` |
| 보기: Commit 열: 부분 좁히기 | `[` |
| 보기: Commit 열: 부분 너비 초기화 | `0` |
| 보기: Commit 열: 열을 내용에 맞추기 | `=` |
| 보기: Commit 열: 모든 열을 내용에 맞추기 | `+` |
| 보기: Commit 열: 선택한 행의 메뉴 | `.` |
| 보기: Commit 열: 표시할 열 선택 |  |
| 보기: Commit 열: Ref 열 표시 전환 |  |
| 보기: Commit 열: 그래프 열 표시 전환 |  |
| 보기: Commit 열: 작성자 열 표시 전환 |  |
| 보기: Commit 열: 날짜 열 표시 전환 |  |
| 보기: Commit 열: 변경 열 표시 전환 |  |
| 보기: Commit 열: SHA 열 표시 전환 |  |
| 보기: Commit 열: 선택한 행을 Log에서 보기 |  |
| 보기: Commit 열: 선택한 행의 파일 변경 내용 보기 |  |
| 보기: Commit 열: 선택한 행 시점의 파일 보기 |  |
| 보기: Commit 열: 선택한 행에서 파일 Blame |  |
| 편집기: 선택 따라가기 전환 |  |
| 이미지: 확대 | `+` |
| 이미지: 축소 | `-` |
| 이미지: 보기에 맞춤 | `0` |
| 이미지: 실제 크기 (1:1) | `1` |
| 이미지: 이전 / 이후 전환 | `Tab` |
| 이미지: 이전 창 넓히기 | `Alt+Right` |
| 이미지: 이후 창 넓히기 | `Alt+Left` |
| 이미지: 창 크기 초기화 | `Alt+0` |
| 이미지: 편집 프로그램... | `O` |
| 이미지: 기본 편집기로 편집 | `o` |
| 파일 기록: Log에서 보기 | `Enter` |
| 파일 기록: commit SHA 복사 | `y` |
| 파일 기록: 요약 복사 | `Y` |
| 파일 기록: 이 commit의 diff 표시 | `d` |
| 파일 기록: 이 commit 시점의 파일 열기 | `o` |
| 파일 기록: 이 commit에서 Blame | `b` |
| 파일 기록: 작업 트리와 비교 | `c` |
| 파일 기록: 이 버전 복원 | `R` |
| 파일 기록: 선택을 위로 확장 | `Shift+Up` |
| 파일 기록: 선택을 아래로 확장 | `Shift+Down` |
| 파일 기록: 목록과 diff 사이 전환 | `Tab` |
| 파일 기록: 목록 넓히기 | `Alt+Right` |
| 파일 기록: 목록 좁히기 | `Alt+Left` |
| 파일 기록: 목록 너비 초기화 | `Alt+0` |
| 파일: 연결 프로그램... | `O` |
| 파일: 기본 앱으로 열기 | `o` |
| Diff: 공백만 바뀐 변경 전환 | `s` |
| ✦ AI: Commit 메시지 생성 | `Ctrl+G i` |
| ✦ AI: Stash 메시지 생성 |  |
| ✦ AI: 프롬프트 변수 삽입 |  |
| ✦ AI: 프로필 편집... |  |
| ✦ AI: 에이전트 열기... |  |
| ✦ AI: 선택한 Commit 설명 | `Ctrl+G w` |
| ✦ AI: 선택한 Commit 분할... |  |
| ✦ AI: 작업 변경 사항 설명 | `Ctrl+G w` |
| ✦ AI: 작업 변경 사항으로 Commit 구성... | `Ctrl+G C` |
| 작성기: 파일을 Commit으로 이동... | `m` |
| Diff: Blame |  |
| Blame: Log에서 commit 표시 | `Enter` |
| Blame: commit SHA 복사 | `Y` |
| Blame: 이 commit의 이 파일 diff 표시 | `d` |
| Blame: 이 commit의 부모에서 Blame | `b` |
| Blame: 파일 기록 열기 | `H` |
| Blame: 상세 카드 표시 또는 숨기기 | `i` |
| Blame: 다음 Hunk | `n` |
| Blame: 이전 Hunk | `p` |
| Blame: 같은 commit의 다음 Hunk | `N` |
| Blame: 같은 commit의 이전 Hunk | `P` |
| Blame: 행으로 이동 | `g` |
| Blame: 주석 열 넓히기 | `>` |
| Blame: 주석 열 좁히기 | `<` |
| Diff: 파일 기록 |  |
| Diff: 파일 보기 |  |
| 파일: 작업 트리에서 편집 | `e` |
| 편집기: 인코딩 변경 |  |
| 편집기: 인코딩으로 다시 열기 |  |
| 편집기: 인코딩으로 변환 |  |
| 편집기: 줄바꿈 변환 |  |
| 편집기: 들여쓰기 변경 |  |
| 앱: 설정... | `Ctrl+, / Ctrl+N ,` |
| 앱: 설정 검색... | `설정에서 /` |
| 앱: 단축키... |  |
| 에이전트: 대기 중인 보기 표시 | `Ctrl+N Enter` |
| 에이전트: 대기 중인 보기 닫기 |  |
| 에이전트: 뒤로 | `Ctrl+N Backspace` |
| 단축키: 모두 초기화... |  |
| 연동: 연결된 Pull request 열기 | `Ctrl+G g` |
| 연동: 연결된 Pull request를 호스트에서 편집 | `e (행에서)` |
| 연동: 연결된 Pull request URL 복사 |  |
| 연동: Pull request 및 이슈 새로 고침 |  |
| 연동: Pull request 체크아웃... |  |
| 연동: commit 참조 열기 (#issue / !request)... |  |
| 연동: 이슈 열기... |  |
| 호스팅: 요청 리뷰... |  |
| 호스팅: 리뷰용 Push... |  |
| 호스팅: 담당자 편집... |  |
| 호스팅: 리뷰어 편집... |  |
| 호스팅: 레이블 편집... |  |
| 호스팅: 마일스톤 설정... |  |
| 로컬 이슈: 이슈 닫기... |  |
| 로컬 이슈: 이슈 다시 열기... |  |
| 로컬 이슈: Obsidian에서 열기... |  |
| 이슈: 보드 열기 |  |
| 로컬 이슈: Obsidian Base 만들기 |  |
| Pull request: Pull request 만들기... | `n` |
| 호스팅: 새 이슈... | `n` |
| Pull request: 검색 | `검색 행에서 Enter` |
| Pull request: 필터 선택 (열림 / merge됨 / 전체)... | `v` |
| Pull request: 제공자 선택... | `제공자 행에서 Enter` |
| Pull request: 저장소 선택... | `저장소 행에서 Enter` |
| Pull request: 소스 연결 해제 (없음) |  |
| 이슈: 제공자 선택... | `제공자 행에서 Enter` |
| 이슈: 소스 연결 해제 (없음) |  |
| 연동: 프로필 연결 해제 (자동) |  |
| 이슈: 저장소 선택... | `저장소 행에서 Enter` |
| 이슈: 검색 | `검색 행에서 Enter` |
| 이슈: 필터 선택... | `v` |
| 이슈: 쿼리 / 목록 편집... |  |
| Commit: 이슈 참조 삽입... |  |
| 연동: 이 저장소에 프로필 사용... |  |
| 연동: 이 저장소의 프로필 관리 |  |
| 연동: 연동 연결... |  |
| 설정: 프로필의 Radar 옵션... |  |
| 연동: 보안 확인 검토 |  |
| 앱: 종료 | `q` |
| 앱: 테마... |  |
| 충돌: 기준 브랜치 목록 편집... |  |
| 충돌: 이 저장소의 기준 브랜치... |  |
| 앱: 테마 변경... | `Ctrl+N y` |
| 앱: 언어 변경... | `Ctrl+N g` |
| 런처: 바로 가기 만들기 |  |
| 런처: 바탕 화면 바로 가기 만들기 |  |
| 런처: 바로 가기 제거 |  |
| 런처: 명령 별칭 만들기 |  |
| 런처: 명령 별칭 제거 |  |
| 런처: PATH에 추가 |  |
| 런처: PATH에서 제거 |  |
| 런처: 'GitPoulpi에서 열기' 메뉴 설치 |  |
| 런처: 'GitPoulpi에서 열기' 메뉴 제거 |  |
| 앱: 그래프 색상 테마 변경... | `Ctrl+N Y` |
| 앱: 그래프 행 색조 전환 | `Ctrl+N T` |
<!-- palette:end -->

## 이동

| 키 | 동작 |
|---|---|
| `Tab` / `Shift+Tab` | 다음 / 이전 창으로 포커스 이동 |
| `Ctrl+N`, 그다음 배지 | 창의 해당 부분으로 포커스 이동하거나 앞에 있는 대화상자나 팝업의 해당 컨트롤 누르기([탐색 접두 키](#탐색-접두-키-ctrln) 참고) |
| `j` / `k`, `Up` / `Down` | 모든 창에서 선택 이동 |
| `PageUp` / `PageDown`, `Home` / `End` | 한 페이지 이동. 첫 / 마지막 행으로 이동 |
| 숫자, 그다음 `j` / `k` / `Up` / `Down` | 반복 횟수 접두: `4j`는 네 행을 이동합니다. `PageUp` / `PageDown`과 함께 쓰면 횟수는 페이지 수입니다. 상태 표시줄에 대기 중인 횟수(`4...`)가 표시됩니다. `Esc`나 다른 키를 누르면 지워집니다. 맨 앞의 `0`은 횟수를 시작하지 않습니다 |
| `h` / `l`, `Left` / `Right` | Refs 창: 섹션 전환. Log: commit의 변경된 파일 목록에서 나가기(`h`) 또는 들어가기(`l`) |
| `?` | 도움말 열기 또는 닫기([앱 내 도움말](#앱-내-도움말-) 참고) |
| `r` | 새로 고침 |
| `F5` | 모든 리모트를 fetch하고, pull request와 이슈를 다시 불러오고, 새로 고침 |
| `q` | 종료 |
| `Ctrl+Z` | 마지막 작업 **실행 취소**. [실행 취소와 다시 실행](undo-redo.md) 참고 |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **다시 실행**. `Ctrl+Shift+Z`는 kitty 키보드 프로토콜이 필요하며, `Ctrl+Y`는 항상 동작합니다 |
| `.`, `Menu`, `Shift+F10` | 선택한 행의 컨텍스트 메뉴 열기 |

### 텍스트 필드의 Ctrl+Z / Ctrl+Y

텍스트 필드가 키보드를 사용 중일 때 `Ctrl+Z`와 `Ctrl+Y`(`Ctrl+Shift+Z`도)는 저장소가 아니라
그 필드의 텍스트에 작용합니다. 키보드를 가진 필드가 항상 우선하므로 commit 메시지에서
`Ctrl+Z`를 눌러도 commit이 취소되지 않습니다. 그 밖의 모든 곳에서는 위의 전역 의미(실행 취소 /
다시 실행 기록)를 유지합니다. 이는 모든 텍스트 필드와 여러 줄 편집기에 적용됩니다: commit 메시지,
설정의 필드와 텍스트 편집기, pull request와 이슈 양식, AI 대화상자, 모든 필터와 검색 상자.

- **입력**은 한 단어씩 취소됩니다(글자 연속, 공백 연속, 문장 부호 연속). 1초 넘게 멈추면
  새 단계가 시작됩니다. 연속된 Backspace는 한 단계입니다. 붙여넣기, 잘라내기, `Ctrl+U` /
  `Ctrl+K`, 삽입한 스니펫, 수락한 자동 완성은 각각 한 단계입니다.
- 실행 취소는 텍스트, 캐럿, 선택 영역을 되돌립니다. 실행 취소 뒤에 새로 편집하면 다시 실행할 수
  있던 것은 사라집니다.
- 각 필드는 한도가 있는 기록(최근 200단계, 텍스트 약 1MB)을 유지하며, 필드가 밖에서 교체되거나
  (선택기에서 고른 폴더, 다시 불러온 양식) 다른 필드가 키보드를 가져가면 기록을 잊습니다.
- 취소할 것이 없으면 `Ctrl+Z`는 아무 동작도 하지 않습니다. 필드가 키보드를 가진 동안에는
  저장소 실행 취소로 넘어가지 않습니다.
- AI 설정의 **지침 편집기**에는 저장과 취소 옆에 **실행 취소**와 **다시 실행** 버튼도 있으며,
  이동할 단계가 없으면 비활성입니다.
- 코드 편집기와 merge 편집기는 자체 실행 취소(`편집: 실행 취소`, `merge: 실행 취소`)를 유지하며,
  선택도 포함합니다.

## 필터 가능한 팝업 목록

필터 가능한 모든 팝업 목록은 같은 키를 사용합니다. 명령 팔레트와 그 선택기(테마, 언어, 그래프
색상), 탭 선택기, 검색 줄이 있는 모든 드롭다운(설정 선택 항목, pull request 양식, 이슈 참조
선택기, **열기 위치** 목록)이 해당합니다. 입력하면 목록이 필터링됩니다. 다음 키로 강조 표시를
움직입니다.

| 키 | 동작 |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | 한 행 위 / 아래로 이동 |
| `Home` / `End` | 첫 / 마지막 행 |
| `PageUp` / `PageDown` | 보이는 한 페이지 이동 |
| `Enter` | 강조 표시된 행 선택 |
| `Esc` | 닫기 |

이 팝업에서 `Home`, `End`는 필터 필드의 캐럿이 아니라 목록을 움직입니다. `Ctrl+A`와
`Ctrl+E`는 여전히 캐럿을 처음과 끝으로 옮깁니다. `Ctrl+K` / `Ctrl+J`도 목록을 움직이므로
`Ctrl+K`는 더 이상 필터를 끝까지 지우지 않습니다(`Ctrl+U`는 여전히 필터를 비우며, 목록 밖의 일반
텍스트 필드에서는 `Ctrl+K`가 계속 줄의 끝까지 지웁니다). `Ctrl+J` / `Ctrl+K`는 텍스트 필드가
있든 없든 다른 모든 목록도 움직입니다(`Ctrl+N`은 탐색 접두 키, `Ctrl+P`는 명령 팔레트이므로 목록
키가 아닙니다). 일반 `j`와 `k`는 필터에 입력되며, 텍스트 필드가 없는 목록에서는 `j` / `k`가
아래 / 위로 이동하는 추가 방법입니다.

마우스를 사용하면 다음과 같습니다.

- 휠은 목록을 **스크롤**합니다. 강조 표시는 움직이지 않습니다.
- 클릭은 행을 **선택**합니다. 더블 클릭은 `Enter`처럼 행을 **선택 확정**합니다.
- 팝업보다 긴 목록은 오른쪽 가장자리에 스크롤바를 표시합니다. 드래그할 수 있습니다.
- 검색 줄이 없는 짧은 드롭다운(선택지가 두세 개)은 첫 클릭으로 선택됩니다.

**실시간 미리 보기.** 테마, 그래프 색상 테마, 언어 선택기는 강조 표시된 행을 팝업 뒤에 즉시
적용합니다. `Enter`는 그것을 유지합니다. `Esc`나 팝업 바깥 클릭은 이전 값으로 되돌립니다.

**테마 필터.** 테마 선택기에는 필터 필드 아래에 **전체**, **다크**, **라이트**, **컬러**
행이 있습니다. `Tab` / `Shift+Tab`은 다음 / 이전 항목으로 이동합니다. `Alt+1`부터 `Alt+4`까지는
직접 선택합니다. 클릭해도 됩니다.

## 앱 내 도움말 (`?`)

`?`, `Ctrl+N ?` 또는 팔레트의 `보기: 도움말 오버레이 전환`으로 도움말을 엽니다. 앱의 모든
부분의 키와 마우스 제스처를 나열합니다. 설정 화면을 포함해 어떤 화면 위에서도 열립니다.

| 키 | 동작 |
|---|---|
| `?` | 도움말 닫기(필터도 지움) |
| *(입력)* | 필터: 텍스트가 포함된 섹션만 유지합니다. `j`, `k`, `q`도 입력됩니다 |
| `Backspace`, `Ctrl+U` | 마지막 문자 삭제 / 필터 지우기 |
| `Esc` | 필터 지우기. 비어 있으면 도움말 닫기 |
| `Up` / `Down` | 한 줄 스크롤 |
| `PageUp` / `PageDown`, `Home` / `End` | 한 페이지 스크롤. 맨 위 / 맨 아래로 이동 |
| `Ctrl+D` 또는 링크 클릭 | 브라우저에서 GitHub의 이 문서 열기 |

휠로 도움말을 스크롤할 수 있고 스크롤바를 드래그할 수도 있습니다. 도움말 바깥을 클릭하면
닫힙니다.

## 탐색 접두 키 (`Ctrl+N`)

`Ctrl+N`은 키를 하나 더 기다립니다. 상태 표시줄에 받아들이는 키가 표시됩니다. 다음 키는 동작을
실행하거나 취소합니다.

| 키 | 동작 |
|---|---|
| `Ctrl+N`, 그다음 배지 | 이 배지가 표시된 창 부분으로 포커스 이동 |
| `Ctrl+N h` / `Ctrl+N l` | Refs 창 / 오른쪽 창 접기 또는 복원 |
| `Ctrl+N Left` / `Ctrl+N Right` | Refs 창과 Log 사이의 구분선을 한 단계 왼쪽 / 오른쪽으로 옮기기(접힌 Refs 창이 먼저 복원됨) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Log와 오른쪽 창 사이의 구분선을 한 단계 왼쪽 / 오른쪽으로 옮기기 |
| `Ctrl+N +` / `Ctrl+N -` | 활성 Refs 섹션을 한 행 늘리기 / 줄이기 |
| `Ctrl+N ]` / `Ctrl+N [` | 오른쪽 창의 commit 메시지 블록을 한 행 늘리기 / 줄이기 |
| `Ctrl+N t` | 탭 선택(탭 선택기. `t`만 눌러도 같음) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | 테마 / 언어 / 그래프 색상 테마 변경 |
| `Ctrl+N Shift+T` | 그래프 행 색조 켜기 또는 끄기 |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | 에이전트가 준비한 보기 표시 / 뒤로 가기([MCP와 에이전트](mcp-and-agents.md) 참고) |
| `Ctrl+N ,` | 설정 열기 |
| `Ctrl+N ?` | 도움말 열기 또는 닫기 |
| `Esc` 또는 다른 아무 키 | 취소 |

### 배지

접두 키가 대기하는 동안 포커스를 받을 수 있는 창의 각 부분은 왼쪽 위 테두리에 한 글자짜리
배지를 표시합니다. 해당 글자를 누르면 그곳으로 포커스가 이동합니다. 배지는 매번 화면 순서대로
다시 배정됩니다. `1`부터 `9`까지 먼저, 그다음 `a`, `b`, `c`... 순입니다. 따라서 화면에 보이는
배지를 읽으세요. 같은 부분이 항상 같은 배지를 받는 것은 아닙니다.

배지가 있는 부분은 다음과 같습니다. Refs 필터와 보이는 각 Refs 섹션, Log(또는 Diff 보기),
열려 있을 때의 파일 보기, 스테이징 안 된 목록과 스테이징된 목록, commit 작성기의 요약 필드,
commit 파일 목록의 컨트롤. 홈 탭에서는 검색 필드, 네 섹션, 도구 모음 버튼(버튼의 배지는 그 버튼을 누릅니다)이 배지를 받습니다.
Radar 탭에서는 필터 필드, **범위**·**보기**·**표시** 콤보박스(콤보박스의 배지는 목록을 엽니다), **새로 고침**과 **뒤로** 버튼,
화면의 각 섹션 제목이 배지를 받습니다. 섹션의 배지는 선택을 그 제목으로 옮기며, 이어서 `Enter`나 `Space`를 누르면 접힙니다.

설정에서 배지는 검색 상자, 페이지 목록(현재 페이지에 하나), 하위 탭, 목록과 그 버튼, 화면에 있는 페이지의 각 필드, 닫기 `✕`에 붙습니다. 배지는 해당 컨트롤을 클릭한 것과 같은 일을 합니다. 텍스트 필드는 편집 상태로 열리고, 체크박스는 전환되고, 드롭다운은 열리고, 버튼은 실행됩니다. 필드에 입력하는 동안이나 드롭다운, 편집기, 검색이 열려 있는 동안에는 `Ctrl+N`이 원래 의미를 유지합니다.

찾아보기 폴더 선택기와 클론, 초기화, worktree 만들기, 워크스페이스, 스파스 체크아웃, Flow, 새 이슈, 풀 리퀘스트 대화상자에서 배지는 필드, 체크박스, 버튼에 붙습니다(버튼의 배지는 그 버튼을 누르고, 필드의 배지는 그 안에 커서를 놓으며, 목록에서는 첫 행에 배지 하나가 붙습니다. 찾아보기 폴더 선택기에는 빠른 액세스 목록용 배지 하나와 폴더 목록용 배지 하나가 각 목록의 첫 행 옆에 있으며, 배지를 누르면 행을 고르지 않고 키보드가 그 목록으로 이동합니다). 활동 로그에서는 두 탭, 범위와 수준 상자, 필터, 수준 체크박스, 버튼, 목록의 첫 항목에 붙습니다. 배지는 해당 컨트롤을 클릭한 것과 같은 일을 합니다. 폴더 선택기에서는 `Ctrl+N`이 배지를 시작하므로 새 폴더 키는 `Alt+N`입니다.

수정(및 commit 메시지 편집) 대화상자에서 배지는 요약, 설명, 작성자 이름·이메일·날짜, **기록을 더 깊이 수정...**, **취소**, **적용**, 닫기 표시에 붙습니다. 기록 다시 쓰기 워크벤치에서는 범위 상자와 그 체크박스 및 ref 버튼, 레시피 이름, 각 규칙과 그 체크박스 및 버튼, **+ 규칙 추가** 버튼, 선택한 규칙의 필드, 미리보기 탭, 필터 상자, 보호된 브랜치 행과 그 모두 선택 / 모두 해제, 그리고 미리보기, 적용, 취소, 저장, 불러오기, AI로 규칙 만들기 버튼에 붙습니다(비밀 값, 신원, 크기, 결과 페이지는 자체 버튼과 행을 더하며, Commits 탭의 행은 목록이므로 배지가 없습니다). 배지는 클릭과 같은 일을 합니다. 드롭다운, 질문, 입력 중인 필드, 폴더 선택기, 영구 삭제 화면, 스크립트 편집기, AI 대화상자, blob diff, 호스트 안내가 열려 있는 동안, 그리고 수정 대화상자가 "버릴까요?"를 묻는 동안에는 아무것도 배지가 붙지 않습니다. `Esc`는 아무것도 닫지 않고 배지를 없앱니다.

팔레트에도 같은 동작이 있습니다. 예를 들어 `탐색: Refs > 로컬로 포커스`나
`탐색: Status > 스테이징된으로 포커스`가 있습니다.

## 저장소와 탭

| 키 | 동작 |
|---|---|
| `Ctrl+O` 또는 `⌂` 클릭 | 홈 탭으로 이동(홈 탭에서는 검색 필드로 포커스, Radar나 릴리스 노트가 열려 있으면 먼저 그것을 닫음) |
| `Ctrl+Shift+N` | 클론 대화상자와 함께 홈 탭 열기 |
| `Ctrl+PageDown` / `Ctrl+PageUp` | 다음 / 이전 탭 |
| `t` | 필터 가능한 목록에서 탭 선택 |
| `Ctrl+W` | 현재 탭 닫기(먼저 확인하며 저장소는 변경되지 않음) |
| `Ctrl+T` 또는 탭 오른쪽 클릭 | 탭 메뉴(행 목록은 [탭 표시줄](interface-tour.md#탭-표시줄)에 있음) |
| `Alt+O` / `Ctrl+Alt+O` | 탭의 폴더 열기: **열기 위치** 목록에서 선택 / 기본 항목 사용. [열기 위치](open-in.md) 참고 |

### 홈 탭

`⌂` 탭은 저장소를 관리합니다. 네 섹션이 있습니다. **열린 저장소**, **즐겨찾기**, **최근
저장소**, **모든 저장소**입니다. [인터페이스 둘러보기](interface-tour.md#홈)를 참고하세요.

| 키 | 동작 |
|---|---|
| `j` / `k`, 화살표, `PageUp` / `PageDown`, `Home` / `End` | 이동 |
| `Enter` 또는 더블 클릭 | 저장소 열기(또는 해당 탭으로 전환). 섹션 헤더에서는 접기 또는 펼치기 |
| `f` 또는 `*`, 또는 별 클릭 | 즐겨찾기에 추가 / 제거 |
| `x`, `Delete` 또는 행의 버튼 | 탭 닫기(열림), 목록에서 제거(최근), 별 제거(즐겨찾기), 잊기(모두). 디스크의 어떤 것도 삭제되지 않습니다 |
| `.`, 오른쪽 클릭 또는 행의 `⋮` | 행 메뉴 |
| `/` 또는 `Ctrl+F` | 이름, 브랜치 또는 경로로 모든 섹션 검색. `Enter`는 첫 번째 일치 항목을 열고, `Esc`는 검색을 지운 다음 필드에서 나갑니다 |
| `Space`, `Left` / `Right` | 현재 섹션 접기 / 펼치기 |
| `-` / `+` | 모두 접기 / 모두 펼치기 |
| `b` / `c` / `i` | 찾아보기(폴더 열기) / 클론 / 초기화 |
| `Esc` | 검색 지우기, 또는 활성 저장소 탭으로 돌아가기 |
| `q` | 종료 |

`Ctrl+P`, `Ctrl+,`, `?`, `Ctrl+N`은 홈 탭에서 동작합니다. `Ctrl+G`, `Ctrl+W`, `Ctrl+T`는
저장소 탭이 필요합니다. 홈 탭에서는 메시지만 표시합니다. 저장소가 필요한 팔레트 명령은
회색으로 표시됩니다.

### 폴더 선택기

**찾아보기**(그리고 초기화, 클론, worktree 만들기 대화상자의 **찾아보기...** 버튼)는 폴더
선택기를 엽니다. 위쪽에 경로 필드가 있고, 왼쪽에 위치와 폴더 트리, 오른쪽에 현재 폴더의
내용이 있습니다.

| 키 | 동작 |
|---|---|
| `Tab` / `Shift+Tab` | 경로 필드, 그다음 왼쪽 열, 그다음 내용 |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | 포커스된 열에서 이동. 경로 필드에서는 자동 완성 항목 사이를 이동 |
| 글자(내용) | 이 글자로 시작하는 다음 폴더로 이동 |
| `Enter` / `Right` | 선택한 폴더 열기. 경로 필드에서는 입력한 폴더로 이동하며, 이미 표시된 폴더라면 선택 |
| `Backspace` / `Left` / `Alt+Up` | 상위 폴더(경로 필드에서 `Backspace`는 문자를 삭제) |
| `Tab` (경로 필드) | 유일한 완성 항목 또는 강조 표시된 항목 수락 |
| `Alt+N` | 새 폴더(`Ctrl+N`은 배지를 시작합니다. [배지](#배지) 참고) |
| `Alt+H` | 숨김 폴더 표시 또는 숨기기 |
| `Ctrl+F` 또는 경로 필드 밖에서 `/` | 입력하는 대로 폴더 항목을 필터링(대소문자 구분 없음. `..`은 남음. 머리글에 "n개 중 x개" 표시. `Enter`는 첫 일치 항목으로 이동, `Esc`는 지움. 폴더를 바꾸면 비워짐). `Tab`이나 클릭으로도 이동 가능 |
| `Ctrl+S`, `Ctrl+Enter` 또는 확인 버튼 | 선택한 하위 폴더, 없으면 표시된 폴더 선택 |
| `Esc` | 자동 완성 닫기 또는 취소 |

경로 필드가 새 경로를 표시하면 경로 전체가 선택됩니다. 처음 입력하는 키가 그것을 대체합니다.

### 초기화 및 클론 대화상자

| 키 | 동작 |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | 다음 / 이전 필드 또는 버튼 |
| `Enter` | 포커스된 버튼 누르기, 또는 만들기 / 클론 |
| `Space` | 포커스된 체크박스 전환, 또는 포커스된 버튼 누르기 |
| `Ctrl+B` | 폴더 찾아보기 |
| `j` / `k`, `g` / `G` | 클론 대화상자의 소스 목록과 저장소 목록에서: 아래 / 위, 처음 / 마지막(검색 필드에서는 아님) |
| `Esc` | 닫기. 클론 중에는 클론이 백그라운드에서 계속됩니다 |

클론 대화상자의 왼쪽 열에는 **URL로 클론**과 연결된 호스팅 프로필마다 하나씩의 소스(GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit)가 나열됩니다. `Up` / `Down`이나 클릭으로 소스를 선택합니다.
그다음 **저장소** 검색 필드에 입력하고, 화살표, `PageUp` / `PageDown` 또는 휠로 목록을
이동한 뒤, `Enter`나 클릭으로 행을 선택합니다. 필드에 대해서는 [시작하기](getting-started.md)를
참고하세요.

## Refs 창

| 키 | 동작 |
|---|---|
| `Enter` / `Space` | 활성 섹션 또는 선택한 폴더 펼치기 또는 접기 |
| `h` / `l`, `Left` / `Right` | 섹션 전환 |
| `/` | 이름으로 필터(`Enter`로 적용, `Esc`로 취소) |
| `y` | 브랜치 계층 보기 켜기 또는 끄기: 브랜치 이름의 `/`에서 만든 폴더(로컬과 리모트) |
| `o` | 선택한 ref, 또는 선택한 폴더나 리모트 아래의 모든 ref를 **솔로**로 지정. 여러 ref를 솔로로 지정하면 그 합집합을 볼 수 있습니다 |
| `Shift+O` | **솔로 해제**: 모든 솔로 지우기 |
| `Shift+H` | **모두 표시**: 숨긴 모든 ref를 다시 표시 |
| `g` | **그래프에서 보기**: Log를 선택한 ref(또는 stash)의 commit으로 스크롤하고 해당 행을 깜박임. 선택은 움직이지 않습니다 |
| `m` | 선택한 ref를 드롭 소스로 **표시**(`Esc`, 또는 그 ref에서 `m`을 다시 누르면 표시 해제) |
| `Shift+M` | 표시한 ref를 선택한 ref에 **놓기**: 드래그 앤 드롭이 여는 메뉴(Merge, Rebase, Push, Fast-forward...)가 열림 |
| `+` (리모트 섹션) | 리모트 추가 |
| `.`, `Menu`, `Shift+F10` 또는 오른쪽 클릭 | 행 메뉴: 브랜치, tag, 리모트 이름 또는 폴더 |
| 워크트리 행에서 `Enter` | 해당 worktree를 자체 탭에서 열기 |
| `Ctrl+G d` | 선택한 브랜치 또는 tag 삭제(먼저 확인). 리모트 행에서는 해당 리모트 브랜치 삭제 |

메뉴가 하는 일은 [Log 다루기](log-and-history.md)를 참고하세요.

### Pull request와 이슈

해당 연동이 있으면 Refs 창에 PULL REQUESTS 섹션이 생깁니다(GitLab: merge requests). 연결된
호스팅 프로필이 있으면 이슈 섹션도 생깁니다. [호스팅과 이슈 트래커](hosting-integration.md#5-사이드바의-pull-request와-이슈)를
참고하세요.

| 키 | 동작 |
|---|---|
| `Enter` | 선택한 행을 브라우저에서 열기. `다시 연결` 행에서는 해당 프로필의 설정 열기 |
| `Space` | 섹션 펼치기 또는 접기 |
| `v` | 섹션의 필터 열기. Pull requests: 열림, merge됨 / 닫힘, 전체. 이슈: 열림, 나에게 할당됨, 닫힘, 전체 |
| `n` | PULL REQUESTS 섹션: pull request 만들기. 이슈 섹션: 새 이슈. (헤더의 `+` 버튼) |
| `e` | PULL REQUESTS 섹션: 선택한 요청을 호스트에서 편집(행의 연필) |
| 검색 행에서 `Enter` | 검색 필드에 입력. `Enter`, `Tab`, `Esc`로 나가며, `Ctrl+U`는 지웁니다 |
| 그룹 헤더에서 `Enter` | 그룹 접기 또는 펼치기(*내 pull request*, *나에게 할당됨*, *내 리뷰 대기 중*, *모든 pull request*) |
| 필터, 제공자 또는 저장소 행에서 `Enter` | 해당 드롭다운 열기. 제공자 목록의 끝에는 **없음 (이 저장소 연결 해제)**가 있습니다 |
| `.` 또는 오른쪽 클릭 | 행 메뉴: 브라우저에서 열기, URL 또는 번호 복사, 소스 브랜치 체크아웃(pull requests). 이슈에서 브랜치 만들기(이슈) |

### 이슈 보드

이슈 헤더의 **보드**(팔레트: **이슈: 보드 열기**)는 이슈 소스의 칸반 보드를 엽니다.
[보드](hosting-integration.md#5b-이슈-보드)와 [로컬 이슈](local-issues.md#칸반-보드)를 참고하세요. 아래의 모든 키는 팔레트 명령(**보드: ...**)이기도 합니다.

| 키 | 동작 |
|---|---|
| 화살표 | 카드 선택(`Up`, `Down`) 또는 이웃 열 선택(`Left`, `Right`). `Home`, `End`, `PageUp`, `PageDown`은 열 안에서 이동 |
| `Shift+Left`, `Shift+Right` | 선택한 카드를 이전 또는 다음 열로 이동 |
| `m` | **보드: 카드 이동...**: 열의 드롭다운 |
| `Enter` | 이슈 열기(로컬 이슈는 노트, 그 밖에는 호스트의 페이지) |
| `e`, `.` | 담당자, 레이블, 마일스톤 편집. 카드 메뉴 |
| `n` | 선택한 열에 새 이슈 |
| `a`, `l`, `M`, `/` | 담당자, 레이블, 마일스톤, 텍스트로 필터링. `x`는 필터를 지움 |
| `c` | GitHub / GitLab 보드의 열이 되는 레이블 |
| `r` | 이슈를 다시 읽기 |
| `Esc` | 보드 닫기 |

마우스: 카드를 다른 열로 드래그하면 이동하고, 카드를 클릭하면 선택, 더블 클릭하면 열기, 오른쪽 클릭하면 카드 메뉴, 휠은 열을 스크롤하며, `Shift`+휠(또는 보드 아래의 막대)은
열들을 스크롤합니다.

### Flow (브랜치 워크플로)

저장소에 워크플로가 설정되어 있으면([브랜치 워크플로](branching-workflows.md) 참고) 도구 모음에 **Flow** 버튼이 있고, 사이드바는 LOCAL과 REMOTE를
브랜치 유형별로 묶습니다. 메뉴의 모든 행은 팔레트 명령(**Flow: ...**. **Flow: Feature 시작...**과 **Flow: 새 Feature 이슈...**는 워크플로의
토픽 유형마다 생성됨)이기도 합니다. 호스트 주도 행(*새 <유형> 이슈*, *호스트가 만든 브랜치 이어받기*)과 그 대화상자, 대기 카드,
"브랜치가 생겼습니다" 카드는 [브랜치 워크플로](branching-workflows.md#앱에서의-호스트-주도-흐름)에 설명되어 있습니다.
`Enter`는 포커스된 버튼(*이슈 만들기*, *이어받기*)을 실행하고, `Esc`는 대기를 중단하거나 닫습니다.

| 키 | 동작 |
|---|---|
| `Ctrl+G Shift+W` | Flow 메뉴 열기(도구 모음 버튼) |
| `Enter`, `Tab`, `Shift+Tab` | 시작 / 완료 / 동기화 / 승격 / 부모 설정 대화상자에서: 미리 보기 또는 실행, 다음 필드, 이전 필드 |
| `Space` | 체크박스 선택(*게시*, *훅 실행*, *그 위에 쌓인 브랜치도 함께 업데이트*) |
| `Esc` | 드롭다운 닫기, 미리 보기에서 양식으로 돌아가기, 대화상자 닫기(실행 중에는 안 됨) |
| 브랜치에 마우스 올리기 | 유형, 부모, 부모보다 얼마나 뒤처졌는지 |

## Log 창

| 키 | 동작 |
|---|---|
| `Enter` / `l` | 선택한 commit의 변경된 파일 목록으로 들어가기 |
| 파일 행에서 `Enter` / `l` | 가운데에 파일의 diff 열기 |
| `Esc` / `h` | commit 목록으로 돌아가기 |
| `Right` | (ref가 둘 이상인 commit) 다른 ref 목록 열기: `Up` / `Down`으로 이동, `Enter`로 체크아웃, `Esc`로 닫기 |
| `f` | 검색 바 열기([검색](#검색) 참고). 이 키는 `/`였던 적이 없으므로 Log의 검색은 그대로입니다 |
| `/` | Refs 필터 필드에 포커스(Refs 창에 포커스가 있을 때도 같은 키). 텍스트 필드에 포커스가 있으면 문자로 입력됩니다 |
| `c` | (commit되지 않은 변경 사항이 있을 때) WIP 행을 선택하고 메시지 필드에 캐럿 두기 |
| WIP 행에서 `Enter` | WIP 행의 메시지 필드에 캐럿 두기 |
| `.`, `Menu`, `Shift+F10` | commit 메뉴. `Up` / `Down`으로 이동, `Right` / `l` / `Enter`로 하위 메뉴 열기, `Left` / `h` / `Esc`로 뒤로 가기 |
| `m` / `a` | 오른쪽 창의 파일: `경로` 또는 `트리` / `모든 파일 보기` |
| `Shift+C` | 이동하거나 크기를 바꿀 Log 열 고르기(아래 참고) |

열을 고른 상태에서는 `Tab` / `Shift+Tab`으로 다른 열을 고르고, `<` / `>`로 이동하고, `]` / `[`로 넓히거나 좁히고, `0`으로 모든 너비를 자동으로 되돌리고, `Esc`나 `Enter`로 나갑니다(위 생성된 표의 `log_columns` 컨텍스트).

WIP 행의 메시지 필드는 commit 작성기의 요약입니다. 캐럿이 있는 동안에는 모든 키가 입력되지만
`Ctrl` 키 조합은 여전히 동작합니다. `Enter`는 작성기의 설명으로 이동합니다. `Esc`는 필드를
벗어나며 텍스트를 유지합니다. [스테이징과 commit](staging-and-committing.md)을 참고하세요.

### commit 여러 개 선택하기

| 키 또는 제스처 | 동작 |
|---|---|
| `Space` | 현재 commit을 선택에 추가하거나 제거 |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | 선택 확장 (Shift를 터미널이 쓰는 경우 Alt) |
| `Ctrl+클릭` / `Shift+클릭` 또는 `Alt+클릭` | commit 전환 / 범위 선택 |
| `Esc` | 선택 지우기 |
| `Shift+Y` | [선택 항목을 Mermaid gitGraph로 복사](log-and-history.md#commit을-mermaid-gitgraph로-복사하기)(상세 수준과 실시간 미리 보기가 있는 팝업. `Enter`로 복사) |

commit을 여러 개 선택한 상태에서 그중 하나를 오른쪽 클릭하고 **commit N개 cherry-pick →
`<branch>`...**를 선택합니다. 대화형 rebase와 같은 보기가 열립니다.

### 브랜치 또는 tag 이름 짓기

`Ctrl+G b`, `Ctrl+G t`, 도구 모음의 **브랜치**와 **Tag** 버튼, 메뉴의 **여기에 브랜치
만들기...** / **여기에 tag 만들기...**는 commit 행의 텍스트 상자에서 이름을 묻습니다.
`Enter`로 ref를 만들고 `Esc`로 취소합니다. 잘못되었거나 이미 있는 이름은 빨간색으로
표시되며 상자는 열린 채로 남습니다.

그 행이 화면에 없으면(diff가 열려 있거나, Log만 크게 보거나, Ref 열이 숨겨진 경우) 대신 닫기 표시(x)와
**취소**, **만들기** 버튼이 있는 작은 틀 상자에 이름을 입력합니다. 브랜치 이름 변경은 Refs 창의 해당 행
위에서, 또는 Log 행의 Ref 칸에서 입력하고, 파일 만들기는 작업 변경 사항 목록의 폴더 아래에서 입력합니다.
어디서든 키는 같습니다: `Enter`로 확인하고 `Esc`로 취소합니다.

상태 표시줄에는 아무것도 입력하지 않습니다. 더 넓은 공간이 필요한 입력은 실제 버튼이 있는 대화상자입니다:
stash 메시지, 주석 tag의 메시지(`Ctrl+Enter` 또는 `Ctrl+S`로 적용, `Enter`로 줄 추가), 업스트림 설정(검색할 수 있는
리모트 브랜치 목록과 해제 버튼), Git LFS 추적 및 추적 중단 패턴입니다. 팔레트에서 고른 숫자 또는 텍스트 설정은
그 행에서 바로 편집할 수 있도록 설정을 열고, Refs 필터(`/`)는 Refs 창 위쪽의 상자에 입력합니다.

### commit 메시지 편집

commit 메뉴의 **commit 메시지 편집...**, `Ctrl+G e` 또는 팔레트의 `Git: Commit 메시지 편집...`은
**Commit 메시지 수정** 팝업을 엽니다. 무엇이 바뀌는지는 [Log 다루기](log-and-history.md)를
참고하세요.

| 키 | 동작 |
|---|---|
| 요약에서 `Enter` | 설명으로 이동 |
| 설명에서 `Enter` | 새 줄. 첫 줄에서 `Up`을 누르면 요약으로 돌아갑니다 |
| `Ctrl+Enter`, `Ctrl+S` 또는 **메시지 업데이트** | 새 메시지 저장 |
| `Tab` / `Shift+Tab` | 요약, 설명, 버튼 사이 이동 |
| `Esc` 또는 **취소** | 변경 없이 닫기 |

### commit 수정

`Ctrl+G Shift+E`(모든 칸)와 `Ctrl+G Shift+A`(작성자)는 수정 대화상자를 엽니다. commit 메뉴, 팔레트의 `Commit: 수정...` 행, commit 상세 패널에서도 열 수 있으며, 여는 모든 방법은 [commit 수정](log-and-history.md#commit-수정메시지-작성자-날짜)에 설명되어 있습니다. 두 단축키 모두 다시 지정할 수 있습니다: `git_prefix.amend`, `git_prefix.edit_author`. 대화상자 안의 키:

| 키 | 동작 |
|---|---|
| `Tab` / `Shift+Tab` | 칸과 버튼 사이 이동 |
| 한 줄 칸에서 `Enter` | 다음 칸. 마지막 칸 다음에는 **적용** |
| `Up` / `Down` | 이전 / 다음 칸 (설명에서는 첫 줄 / 마지막 줄에서) |
| `Ctrl+Enter`, `Ctrl+S` 또는 **적용** | 바꾼 칸 적용 |
| `Esc` 또는 **취소** | 변경 없이 닫기 |
| `Alt+H` 또는 **기록을 더 깊이 수정...** | 대화상자를 닫고 이 commit부터 기록 다시 쓰기 워크벤치를 엽니다(기록 다시 쓰기가 포함된 빌드에서만). 칸을 편집했다면 먼저 묻습니다(`Esc`로 확인). 다시 지정 가능: `reword.deeper` |

### worktree 만들기

**`<ref>`에서 worktree 만들기...**(브랜치 및 tag 메뉴)와 **여기에 worktree 만들기...**(commit
메뉴)는 대화상자를 엽니다. 폴더, 선택 사항인 새 브랜치, **새 worktree를 탭으로 열기**가
있습니다. 키: `Tab` / `Shift+Tab`으로 이동, `Enter`로 만들기, `Space`로 체크박스 전환,
`Ctrl+B`로 찾아보기, `Esc`로 취소합니다.

### 대화형 rebase

commit 메뉴의 **여기서부터 대화형 rebase...** 또는
`Git: 선택한 Commit부터 대화형 Rebase...`는 rebase 보기를 엽니다. commit 여러 개의
cherry-pick도 같은 보기를 사용합니다. [Log 다루기](log-and-history.md)를 참고하세요.

| 키 | 동작 |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | 행의 동작 지정: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup과 Edit은 rebase에서만) |
| `Space` | 행의 동작 목록 열기(`Up` / `Down`으로 선택, `Enter`로 적용, `Esc`로 목록 닫기) |
| `Up` / `Down` | 선택 이동 |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | 선택한 행을 위로 / 아래로 이동 |
| `a` | Autosquash 미리 보기 켜기 또는 끄기 |
| Reword 행에서 `Enter` | 해당 commit의 새 메시지 편집 |
| `Ctrl+S`, `Ctrl+Enter` 또는 **Rebase 시작** / **Cherry-pick 시작** | 계획 실행 |
| `Ctrl+R` 또는 **초기화** | 보기를 열었을 때의 계획으로 복원 |
| `Esc` 또는 **취소** | 변경 없이 닫기 |

rebase가 멈추면(충돌 또는 Edit 단계) Log 위의 배너가 **계속**, **건너뛰기**, **중단**을
제공합니다. 팔레트에는 `Rebase: 계속`, `Rebase: 건너뛰기`, `Rebase: 중단`이 있습니다.

## 검색

`f`, 도구 모음의 **검색** 버튼 또는 `탐색: Commit 검색...`은 Log 위에 검색 바를 엽니다.
전체 commit 메시지, 작성자 이름과 이메일, SHA, ref 이름을 검색합니다. 행을 숨기지
않습니다. 일치하지 않는 행은 흐리게 표시됩니다.

| 키 | 동작 |
|---|---|
| *(입력)* | 검색 텍스트 편집(모든 키가 입력됨) |
| `Enter` / `Down` 또는 바의 `↑` | 다음 일치 항목 |
| `Shift+Enter` / `Up` 또는 바의 `↓` | 이전 일치 항목 |
| `Esc` 또는 바의 `✕` | 검색 바 닫기 |

## Diff 보기

변경된 파일은 Log 대신 가운데에 열립니다. [스테이징과 commit](staging-and-committing.md)을
참고하세요.

| 키 | 동작 |
|---|---|
| `Enter` / `l` 또는 더블 클릭(파일 행에서) | 파일의 diff 열기 |
| `Esc`, `q` 또는 제목의 `✕` | diff 닫기 |
| `j` / `k`, `Up` / `Down` | 스크롤 |
| `PageUp` / `PageDown`, `Home` / `End` | 한 페이지 스크롤. 처음 / 끝으로 이동 |
| `h` / `l`, `Left` / `Right` | 옆으로 스크롤(자동 줄바꿈 시에는 해당 없음) |
| `n` / `p` | 다음 / 이전 hunk |
| `m` | 인라인 또는 분할 레이아웃 |
| `Shift+H` | hunk만 보기 또는 전체 파일 |
| `w` | 자동 줄바꿈 켜기 또는 끄기 |
| `s` | 공백만 바뀐 변경 사항 표시 또는 숨기기 |
| `Shift+M` | 미니맵 켜기 또는 끄기 |
| `Shift+C` | 파일을 읽는 인코딩 선택(목록: 먼저 자동 감지) |
| `e` | 내장 편집기에서 파일 편집([편집기](#편집기) 참고) |
| `r` | (Git LFS 파일) LFS 카드 또는 원시 포인터. [Git LFS](lfs.md) 참고 |
| `Shift+E` | (충돌한 파일) merge 편집기 열기. [merge 충돌 해결](merge-conflicts.md) 참고 |
| `y` / `Ctrl+C` | 선택한 텍스트 복사. 선택이 없으면 커서 아래의 줄 또는 선택한 줄 |
| `.` 또는 hunk 오른쪽 클릭 | hunk 메뉴 |

### diff에서 스테이징하고 버리기

이 키들은 작업 변경 사항의 diff에서만 동작합니다(스테이징 안 된 목록이나 스테이징된
목록에서 열린 경우). 버리기는 스테이징 안 된 목록에서만 동작합니다.

| 키 | 동작 |
|---|---|
| `Space` | 커서 아래의 hunk 스테이징(스테이징 안 됨) / 스테이징 해제(스테이징됨) |
| `v` | 줄 선택 시작 또는 해제. `j` / `k`로 확장 |
| `Enter` | 선택한 줄, 또는 커서 아래의 줄 스테이징 / 스테이징 해제 |
| `x` | 커서 아래의 hunk 버리기(먼저 확인: `y` / `Enter`로 확정, `n` / `Esc`로 취소) |
| `Shift+X` | 선택한 줄, 또는 커서 아래의 줄 버리기(먼저 확인) |

변경된 줄만 해당합니다. 선택에 포함된 문맥 줄은 무시됩니다. 두 번째 hunk에 걸친 선택은 첫 번째
hunk에만 적용됩니다.

## 이미지 보기

Diff 보기나 파일 보기의 그림입니다. 키는 위 표의 `image` 컨텍스트에 있습니다.
[그림](image-view.md)을 참고하세요.

## 파일 보기, Blame, 파일 기록

| 키 | 동작 |
|---|---|
| `Esc` / `q` | 닫기(텍스트 선택이 있으면 `Esc`가 먼저 선택을 지움) |
| `j` / `k`, `Up` / `Down` | 선택 이동 |
| `PageUp` / `PageDown` | 한 페이지 스크롤 |
| `Home` / `End`, `Shift+G` | 첫 / 마지막 줄 |
| `h` / `l`, `Left` / `Right` | 텍스트를 옆으로 스크롤 |
| `Enter` | (Blame, 파일 기록) Log에서 해당 행의 commit으로 이동 |
| `e` | 내장 편집기에서 파일 편집 |
| `y` / `Ctrl+C` | 선택한 텍스트 복사. 선택이 없으면 선택한 행(Blame: 코드만, 파일 기록: 요약) |

Blame과 파일 기록에는 전용 키가 있으며, 위의 생성된 표(컨텍스트 `file_insight`와 `history`)와 팔레트의 `Blame: ...` / `파일 기록: ...`에 나열됩니다. 모두 다시 지정할 수 있습니다. 자세한 내용은 [Blame](log-and-history.md#blame)과 [파일 기록](log-and-history.md#파일-기록)을 참고하세요. 기록 목록에서는 단독 문자(`d` `o` `b` `c` `y`)가 선택한 행에 작용하지만, Ctrl 조합(`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`)은 항상 전역 의미를 유지합니다.

## Commit 열

파일 보기 왼쪽의 열입니다([Commit 열](interface-tour.md#commit-열) 참고). `Alt+C`는 어디에서든 열을 표시하거나 숨깁니다. 다른 키는 열에 키보드가 있을 때 동작합니다(`Ctrl+N` 다음 열의 배지, 또는 `보기: Commit 열에 포커스`).

| 키 | 동작 |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | commit 고르기 |
| `Enter` / `Right` | 고른 commit의 파일 목록으로 넘어가기 |
| `Esc` | 키보드를 보기에 돌려주기 |
| `.`, `Menu`, `Shift+F10` | 고른 행의 메뉴 |

그 밖의 키(옆으로 스크롤하는 `Shift+Left` / `Shift+Right`, 부분 크기를 조절하는 `Tab`, `]`, `[`, `0`)는 위의 생성된 표의 컨텍스트 `commit_column`에 있습니다.

## 편집기

diff나 파일 보기에서 **편집** 또는 `e`를 누르면 파일의 작업 디렉터리 사본에서 내장 편집기가
열립니다. [스테이징과 commit](staging-and-committing.md)을 참고하세요.

| 키 | 동작 |
|---|---|
| *(입력)* | 캐럿 위치에 삽입(선택 영역을 대체) |
| `Enter` | 현재 줄의 들여쓰기를 유지하며 새 줄 |
| `Tab` / `Shift+Tab` | 들여쓰기 / 내어쓰기(여러 줄이 선택되었으면 그 줄들) |
| 화살표, `Home` / `End` | 이동. `Home`은 공백이 아닌 첫 문자로, 그다음 0열로 이동합니다. `Shift`는 선택을 확장합니다 |
| `Ctrl+Left` / `Ctrl+Right` | 한 단어 이동 |
| `Ctrl+Home` / `Ctrl+End` | 파일의 처음 / 끝 |
| `PageUp` / `PageDown` | 한 페이지 위 / 아래 |
| `Ctrl+A` | 모두 선택 |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | 복사 / 잘라내기 / 붙여넣기 |
| `Ctrl+U` / `Ctrl+K` | 줄의 처음까지 / 끝까지 삭제 |
| `Ctrl+Z` | 실행 취소 |
| `Ctrl+Y` / `Ctrl+Shift+Z` | 다시 실행 |
| `Ctrl+S` | 저장 |
| `Ctrl+F` / `F3` | 찾기. 찾기 필드에서: `Enter` / `Down` / `F3`은 다음, `Shift+Enter` / `Up`은 이전, `Esc`는 닫기 |
| `Alt+M` | 미니맵 켜기 또는 끄기 (**맵**) |
| `Esc` | 닫기. 저장하지 않은 변경 사항이 있으면 묻습니다: **저장**(`S` / `Enter`), **버리기**(`D`), **취소**(`C` / `Esc`) |

편집기에서 `Tab`은 들여쓰기를 하므로 포커스를 옮기지 않습니다. 다른 `Ctrl` 키 조합
(`Ctrl+P`, `Ctrl+N`)은 여전히 동작합니다.

상태 표시줄의 **인코딩**, **줄바꿈**, **들여쓰기**는 버튼입니다. 각각 드롭다운을 엽니다(화살표 또는 휠,
입력하여 필터링, `Enter`로 선택, `Esc`로 닫기). 팔레트(`Ctrl+P`)에서도 같은 메뉴를 열 수 있습니다: *편집기: 인코딩 변경*, *편집기: 인코딩으로 다시 열기*, *편집기: 인코딩으로 변환*, *편집기: 줄바꿈 변환*, *편집기: 들여쓰기 변경*.
자세한 내용은 [스테이징과 commit](staging-and-committing.md)을 참고하세요.

## Git 동작 (`Ctrl+G`)

`Ctrl+G`는 키를 하나 더 기다립니다.

| 키 | 동작 |
|---|---|
| `s` / `u` | 선택한 파일 스테이징 / 스테이징 해제 |
| `a` | 모든 변경 사항 스테이징 |
| `x` | 모든 변경 사항 버리기(먼저 확인) |
| `c` | commit 작성기 열기 |
| `Enter` | 어디에서든 양식의 메시지로 스테이징된 변경 사항을 바로 commit([스테이징과 commit](staging-and-committing.md#commit-작성) 참고) |
| `m` | 메시지와 함께 모든 변경 사항 stash. 작은 대화상자에 입력하며 비워 두어도 됩니다. AI가 켜져 있으면 대화상자의 **AI 메시지** 버튼(대화상자 안에서 `Ctrl+G`)이 메시지를 작성합니다 |
| `Shift+T` | 메시지 없이 모든 변경 사항 stash(추적되지 않는 파일 포함) |
| `o` | stash pop: Log에서 선택한 stash 행, 없으면 가장 최근 stash |
| `p` | Pull 버튼의 설정대로 pull(`git.pull_mode`) |
| `Shift+P` | 기본 리모트로 push(첫 push에서는 업스트림 설정). 또는 리모트를 묻기 |
| `Shift+U` | 모든 tag push |
| `f` | prune과 함께 모든 리모트 fetch |
| `g` | 체크아웃한 브랜치의 pull request를 브라우저에서 열기 |
| `b` / `t` | HEAD에 새 브랜치 / 새 tag([브랜치 또는 tag 이름 짓기](#브랜치-또는-tag-이름-짓기) 참고) |
| `d` | 선택한 브랜치 또는 tag 삭제(먼저 확인) |
| `e` | 선택한 commit의 메시지 편집 |
| `Shift+E` / `Shift+A` | 선택한 commit 수정(모든 칸) / 작성자 수정([commit 수정](#commit-수정) 참고) |
| `y` | 선택한 commit cherry-pick |
| `v` | 선택한 commit revert |
| `Shift+R` | 선택한 commit으로 reset(mixed) |
| `r` | 선택한 브랜치 위로 rebase |
| `Shift+W` | Flow 메뉴 열기(브랜치 워크플로가 있는 저장소) |
| `Shift+H` | 기록 다시 쓰기 워크벤치 열기(기록 다시 쓰기가 포함된 빌드에서만) |
| `l` | LFS 메뉴 열기. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch all / checkout all / pull / push all / prune. [Git LFS](lfs.md) 참고 |
| `i` | ✦ AI: 스테이징된 변경 사항의 commit 메시지 작성 |
| `w` | ✦ AI: 선택한 commit 또는 작업 변경 사항 설명 |
| `Shift+C` | ✦ AI: 작업 변경 사항을 commit으로 나누는 방법 제안 |

AI 키는 AI가 켜져 있을 때만 동작합니다([AI 지원](ai.md) 참고). `Esc`는 실행 중인 AI 요청을
취소합니다.

## 오른쪽 창 (작업 변경 사항과 commit 상세)

WIP 행을 선택하면 오른쪽 창에 **스테이징 안 된 파일**과 **스테이징된 파일** 목록 및 commit
작성기가 표시됩니다. commit을 선택하면 commit 상세와 그 파일이 표시됩니다.

| 키 | 동작 |
|---|---|
| `j` / `k`, `Up` / `Down` | 이동. 한 목록의 끝에서는 선택이 다른 목록으로 이어집니다 |
| `h` / `l`, `Left` / `Right` | 스테이징 안 된 목록 / 스테이징된 목록으로 이동 |
| `Enter` | 활성 목록 접기 또는 펼치기 |
| `Space` / `Insert` | 현재 파일을 선택에 추가하거나 제거 |
| `Ctrl+A` | 목록의 모든 파일 선택 |
| `Ctrl+Space` | 선택한 파일 스테이징 / 스테이징 해제, 또는 폴더 행 접기 |
| `d` | 선택한 파일의 diff 열기 |
| `m` | `경로`(평면 목록) 또는 `트리`(폴더) |
| `a` | `모든 파일 보기`: 변경된 파일만이 아니라 전체 트리 |
| `z` | 모든 폴더를 접거나 다시 펼치기 |
| `/` | commit의 변경된 파일을 이름으로 필터 |
| `.`, `Menu`, `Shift+F10` | 파일 메뉴 |
| `c` | commit 작성기의 요약 필드로 포커스 |
| `Shift+A` | **이전 commit 수정 (Amend)** 켜기 또는 끄기 |
| `Ctrl+N ]` / `Ctrl+N [` | commit 메시지 블록 늘리기 / 줄이기 |

commit 작성기에서는 다음과 같습니다.

| 키 | 동작 |
|---|---|
| `Enter` | 요약: 설명으로 이동. 설명: 새 줄 |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit(`Ctrl+G Enter`는 어디에서든 같은 일을 함) |
| `Alt+R` | 이슈 참조 삽입 |
| `Esc` | 작성기에서 나가기 |

파일 메뉴에는 다음이 있습니다. 스테이징 또는 스테이징 해제, 변경 사항 버리기, 무시, 파일
Stash, 파일 기록, 파일 Blame, 외부 diff 도구로 열기, 외부 편집기로 열기, 기본 프로그램으로
파일 열기, 폴더에서 보기, 파일 경로 복사, 파일 변경 사항에서 패치 만들기, 파일 편집, 파일
삭제. 충돌한 파일에서는 **merge 편집기에서 열기**와 세 개의 **해결** 행으로 시작합니다. Git
LFS가 켜져 있으면 LFS 행이 있습니다. **변경 사항 버리기...**와 **파일 삭제...**는 먼저 확인합니다.

## 창 버튼

모든 버튼에는 키보드 대응 키가 있습니다.

| 버튼 | 키 |
|---|---|
| 제목 표시줄 왼쪽의 🐙 (정보) | 팔레트의 `도움말: GitPoulpi 정보` |
| 제목 표시줄의 **열기 위치** | `Alt+O`, `Ctrl+Alt+O` |
| 제목 표시줄 오른쪽의 `×` (종료) | `q` |
| 제목 표시줄의 버전 (릴리스 노트) | 팔레트의 `도움말: 릴리스 노트` |
| 제목 표시줄에서 `⚙` 왼쪽의 `⌕ 전체 검색` | `Ctrl+Alt+F` |
| 제목 표시줄의 `⚙` (설정) | `Ctrl+,` |
| 탭 표시줄의 워크스페이스 칩 | `Alt+W` |
| 마지막 탭 뒤의 `+` | `Alt+T` |
| **Radar** 탭 | `Alt+R` |
| **워크스페이스** 탭 | `Alt+J` |
| 상태 표시줄의 **활동** | `Ctrl+N Shift+A` |
| 도구 모음: **실행 취소**, **다시 실행** | `Ctrl+Z`, `Ctrl+Y` |
| 도구 모음: **Pull** / 그 `▾` | `Ctrl+G p` / `Git: Pull 옵션...` |
| 도구 모음: **Push** / 그 `▾` | `Ctrl+G Shift+P` / `Git: 리모트로 Push...` |
| 도구 모음: **브랜치**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| 도구 모음: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| 도구 모음: **LFS** (LFS가 켜져 있을 때) | `Ctrl+G l` |
| 도구 모음: **Flow** (저장소에 워크플로가 있을 때) | `Ctrl+G Shift+W` |
| 도구 모음: **검색** | `f` |
| 도구 모음: 연결된 pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| 도구 모음: 프로필 목록 (프로필이 둘 이상일 때) | `저장소: 프로필 변경...` |

도구 모음은 [인터페이스 둘러보기](interface-tour.md#도구-모음)에 설명되어 있습니다.

## 설정

`Ctrl+,`, `Ctrl+N ,`, `⚙` 톱니바퀴 또는 `앱: 설정...`은 설정 화면을 엽니다.

| 키 | 동작 |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | 페이지 목록과 옵션 사이 이동 |
| `j` / `k`, `Up` / `Down` | 선택 이동 |
| `Enter` / `Space` | 페이지 열기, 체크박스 전환, 필드 편집, 드롭다운 열기 또는 버튼 누르기 |
| `/`, `Ctrl+F` | 모든 옵션 검색 |
| `Alt+Left` / `Alt+Right` | 하위 탭 전환(일반 / 목록) |
| `Esc` / `q` | 설정 닫기. 필드를 편집 중일 때 `Esc`는 편집만 취소합니다 |
| `?` | 설정 화면 위에 도움말 열기 |

[설정](settings.md)을 참고하세요.

## 텍스트 필드

모든 텍스트 필드는 같은 키를 공유합니다. 필터, 검색 필드, 명령 팔레트, 프롬프트, commit
작성기, WIP 행의 필드, 대화상자 필드, 설정 필드가 그렇습니다. 필드가 키보드를 사용하는
동안에는 이 키들이 창 단축키보다 우선합니다.

| 키 | 동작 |
|---|---|
| 문자 | 캐럿 위치에 입력(선택 영역을 대체) |
| `Left` / `Right` | 한 문자 이동(선택이 있으면 선택의 처음 / 끝으로 이동) |
| `Ctrl+Left` / `Ctrl+Right` | 한 단어 이동 |
| `Home` / `End` | 줄의 처음 / 끝 |
| `Shift` + 위의 이동 키 | 선택 확장 또는 축소 |
| `Up` / `Down` (설명에서만) | 이전 / 다음 줄 |
| `Ctrl+Home` / `Ctrl+End` (설명에서만) | 텍스트의 처음 / 끝 |
| `Ctrl+A` | 모두 선택 |
| `Backspace` / `Delete` | 캐럿 앞 / 뒤의 문자 또는 선택 영역 삭제 |
| `Ctrl+U` | 캐럿에서 줄의 처음까지 삭제(또는 선택 영역 삭제) |
| `Ctrl+K` | 캐럿에서 줄의 끝까지 삭제(또는 선택 영역 삭제) |
| `Ctrl+C` / `Ctrl+X` | 선택 영역 복사 / 잘라내기(비밀번호 필드에서는 불가) |
| 붙여넣기 (터미널 자체의 붙여넣기) | 캐럿 위치에 삽입하며 선택 영역을 대체 |

commit 설명에서 `Enter`는 새 줄을 입력하고 `Ctrl+Enter`는 commit합니다. 터미널마다
`Ctrl+Enter`를 보내는 방식이 다르므로 `Alt+Enter`와 `Ctrl+J`도 commit합니다.

일부 콘솔은 `Ctrl+글자`를 제어 문자로 보냅니다(`Ctrl+A`는 `0x01`). GitPoulpi는 두 형식을 모두
이해합니다. `AltGr` 문자(스위스나 프랑스 키보드의 `@`, `#`, `€`)는 입력됩니다.

## 툴팁

포인터가 대상(Log 셀, 도구 모음 버튼, 연결된 pull request) 위에 약 0.6초 머물면 툴팁이
나타납니다. 키 입력, 클릭, 스크롤, 크기 조정, 열린 팝업이나 메뉴, 또는 포인터가 대상을
벗어나면 툴팁이 숨겨집니다. 그 후에는 포인터가 다시 움직일 때만 돌아옵니다.
