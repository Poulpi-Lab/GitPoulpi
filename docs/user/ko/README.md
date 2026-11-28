[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · **한국어** · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# GitPoulpi 사용자 문서

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi 문어" width="128"></p>

GitPoulpi는 클릭할 수 있는 commit 그래프, 명령 팔레트, 호스팅 및 이슈 트래커 연동,
선택 사항인 AI 도움말, 그리고 UI가 하는 모든 일을 하는 CLI와 MCP 서버를 갖춘
터미널용 Git 클라이언트입니다. 이 페이지들은 읽는 순서대로 되어 있습니다:
맨 위부터 시작하고, 필요할 때 참조 페이지로 건너뛰세요.

![GitPoulpi 메인 창](../images/hero.png)

## 여기서 시작

1. **[설치](installation.md)**: 필요한 것, 릴리스 아카이브, 패키지 관리자,
   소스에서 빌드하기. 더 작은 바이너리가 필요하면
   **[사용자 지정 빌드](custom-builds.md)**를 참고하세요. GitPoulpi가 스스로
   최신 상태를 유지하는 방법(버전 칩, 업데이트 패널, 되돌리기):
   **[업데이트](updates.md)**. 컨테이너에서 실행하려면:
   **[Docker에서 실행하기](docker.md)**. 관리자를 위한 사내 피드, 미러,
   업데이트 정책: **[엔터프라이즈 배포](enterprise-distribution.md)**.
2. **[시작하기](getting-started.md)**: 실행부터 첫 commit까지, 처음 5분.
3. **[인터페이스 둘러보기](interface-tour.md)**: 창의 모든 영역의 이름을 그림과
   함께 설명합니다.

## 기능 찾기

| 하고 싶은 일 | 페이지 | 앱에서 여는 방법 |
|---|---|---|
| 브랜치 두 개에서 동시에 작업하기 | [워크트리](worktrees.md) | Refs 사이드바의 **워크트리** 섹션(`+`로 만들기. 하나가 생기기 전에는 숨겨져 있음: 팔레트 **왼쪽 창: 워크트리 섹션 표시**), 팔레트 **Refs: 워크트리 만들기...** |
| 중첩된 저장소 다루기 | [서브모듈](submodules.md) | Refs 사이드바의 **서브모듈** 섹션, 팔레트 **Git: 모든 서브모듈 업데이트**, **Refs: 서브모듈 업데이트 (선택)** |
| merge 충돌 해결하기 | [merge 충돌 해결](merge-conflicts.md) | Status 창의 `!` 행에서 `Enter`, 배너의 **충돌 해결...** 버튼, 또는 Diff 보기에서 `Shift+E`, 팔레트 **충돌: merge 편집기 열기** |
| pull 또는 merge request 리뷰하기 | [pull 또는 merge request 리뷰](review.md) | **PULL REQUESTS** 섹션의 행에서 `Enter`, Radar 행에서 `e`, 워크스페이스 검색에서 `Alt+R`, 팔레트 **호스팅: 요청 리뷰...** |
| pull request와 이슈를 둘러보고 만들고 관리하기 | [pull request와 이슈](pull-requests-and-issues.md) | Refs 사이드바의 **PULL REQUESTS**와 **이슈** 섹션(`n`으로 만들기), 팔레트 **Pull request: Pull request 만들기...**, **호스팅: 새 이슈...**. 먼저 계정을 연결하세요: [호스팅과 이슈 트래커](hosting-integration.md) |
| 이슈나 요청의 토론을 읽고 답하기 | [대화](conversations.md) | 이슈 행, 보드 카드, Radar 행에서 `Enter`, 팔레트 **호스팅: 이슈 대화 열기...** |
| 내 이슈를 보드로 보기 | [이슈 보드](pull-requests-and-issues.md#이슈-보드) | 이슈 머리글의 **보드** 버튼, 팔레트 **이슈: 보드 열기** |
| 토큰 없이 이슈를 Markdown 노트로 관리하기 | [로컬 이슈와 Obsidian](local-issues.md) | 이슈 섹션, 제공자 `Local Markdown - <이름>`, 팔레트 **로컬 이슈: Obsidian에서 열기...** |
| 여러 commit에 걸쳐 작성자, 날짜, 메시지, 파일 다시 쓰기 | [기록 다시 쓰기](history-rewrite.md) | `Ctrl+G Shift+H`, 팔레트 **다시 쓰기: 기록 워크벤치** |
| 작업 실행 취소(또는 다시 실행) | [실행 취소와 다시 실행](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`, `Alt+Z`로 실행 취소 기록 열기, 팔레트 **편집: 실행 취소** |
| 키 하나로 여러 동작 실행(스테이지, AI 메시지, 커밋, 푸시...) | [시퀀스](sequences.md) | 설정 > 시퀀스; 팔레트 **시퀀스: <이름>** |
| Git LFS로 큰 파일 추적하기 | [Git LFS](lfs.md) | 팔레트 **LFS: 패턴 추적...**, **LFS: 이 저장소에 초기화** |
| 저장소 전체에서 브랜치, commit, pull request, 이슈 찾기 | [검색](search.md) | `Ctrl+Alt+F` 또는 제목 표시줄의 전체 검색 버튼, `f`는 현재 저장소의 commit 그래프를 검색 |
| 나를 기다리는 모든 것을 한 목록으로 보기 | [Radar](radar.md) | `Alt+R`, 팔레트 **Radar: 열기** |
| 저장소를 묶어 함께 fetch하거나 pull하기 | [워크스페이스](workspaces.md) | `Alt+W`로 전환, `Alt+J`로 워크스페이스 탭 열기, 팔레트 **워크스페이스: 관리...** |
| AI 도움 받기(commit 메시지, 설명, 서술) | [AI 지원](ai.md) | `Ctrl+,` > **AI** > **AI 사용**, 이어서 `Ctrl+G i`로 commit 메시지 작성, 팔레트 **AI: ...** |
| 명령이나 키 찾기 | [단축키](keybindings.md) | 앱에서 `?`가 키 목록을 보여 주고 `Ctrl+P`로 어떤 명령이든 찾을 수 있음 |

## 일상적인 사용

4. **[Log 다루기](log-and-history.md)**: 그래프 읽기, 검색, 브랜치, tag, stash,
   ref 드래그, rebase, cherry-pick, reword와 amend, 배너, pull과 push,
   auto-stash.
5. **[스테이징과 commit](staging-and-committing.md)**: 작업 중인 변경 사항, diff,
   hunk와 한 줄 단위 스테이징, commit 작성, commit composer, blame, 파일 기록,
   내장 편집기.
6. **[그림](image-view.md)**: Diff 보기와 파일 보기의 이미지, 이전과 이후, 확대,
   편집 프로그램.
7. **[merge 충돌 해결](merge-conflicts.md)**: merge 편집기(출력 위에 A와 B를
   나란히), 줄과 덩어리 선택, 바이너리, LFS, 삭제, 심볼릭 링크, 서브모듈 충돌용
   카드, 에이전트가 해결책을 제안하는 방법.
8. **[워크트리](worktrees.md)**: 다른 브랜치의 두 번째 체크아웃 폴더:
   워크트리 섹션, 연결된 워크트리 만들기, 열기, merge, 제거.
9. **[서브모듈](submodules.md)**: 서브모듈 섹션, 업데이트하기, 체크아웃과 pull에
   맞춰 최신으로 유지하기.
10. **[실행 취소와 다시 실행](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, 취소할 수 있는
    것과 없는 것, 거부하거나 묻는 경우, 얼마나 거슬러 올라가는지, 끄는 방법.
11. **[Git LFS](lfs.md)**: 큰 파일, 추적, 잠금, `git-lfs`가 설치되어 있지 않을 때
    일어나는 일.
12. **[테마](themes.md)**: 내장 테마, 어떤 테마든 어떤 레인 팔레트와도 조합,
    실시간 미리 보기, 직접 만든 테마 파일, `gitpoulpi theme`.
13. **[워크스페이스](workspaces.md)**: 고유한 탭과 선택적 기본 commit 프로필을 가진,
    이름이 붙은 저장소 묶음, 탭 표시줄의 칩, 워크스페이스 탭,
    모두 fetch, 모두 pull, 누락된 저장소 클론.
14. **[검색](search.md)**: 하나의 퍼지 검색기(`Ctrl+Alt+F`)로 워크스페이스 또는
    열린 탭의 모든 저장소의 브랜치, tag, worktree, stash, commit,
    pull/merge request, 이슈를 찾습니다.
15. **[Radar](radar.md)**: 워크스페이스(또는 열린 탭)의 pull/merge request,
    이슈, 진행 중인 작업을 하나의 목록으로: 필터, 고정, 미루기, 그리고 각 행에서
    할 수 있는 일.
16. **[pull request와 이슈](pull-requests-and-issues.md)**: 사이드바의 PULL
    REQUESTS와 이슈 섹션, 이슈 보드, pull request나 이슈 만들기, 도구 모음 칩,
    commit 메시지의 참조.
17. **[대화](conversations.md)**: 이슈나 요청의 토론을 창 안에서 읽고 답하기,
    게시 이름, 호스트별로 할 수 없는 것.
18. **[pull request 리뷰](review.md)**: 댓글 스레드, 대기 중인 리뷰, 승인 / 변경 요청이
    있는 리뷰 보기.
19. **[열기 위치](open-in.md)**: 활성 탭의 폴더를 터미널, 멀티플렉서, 편집기,
    파일 관리자에서 여는 제목 표시줄의 목록.
20. **[활성 탭의 폴더 따라가기](following-the-tab-folder.md)**:
    "종료 후 cd", 그리고 bash, zsh, fish, PowerShell용 `gitpoulpi shell-init`
    래퍼(`gp`).
    GitPoulpi를 가장 잘 보여 주는 터미널: **[터미널 고르기](terminals.md)**.
    tmux, zellij, screen, herdr, 터미널 창을 통과하는 것: **[멀티플렉서](multiplexers.md)**.

추가: **[한 브랜치에 여러 리모트](multiple-remotes.md)**: push 대상과 pull 소스, 리모트 대화상자, 모두 또는 선택한 리모트로 Push, 움직인 리모트 해결, 충돌 뒤 계속, CLI와 순정 git alias.

**[브랜치 워크플로](branching-workflows.md)**. Git Flow, GitHub Flow, GitLab Flow,
Trunk-based 프리셋 또는 직접 정의한 베이스, 브랜치 이름 패턴, 토픽 유형
(`gitpoulpi flow ...`).

**[기록 다시 쓰기](history-rewrite.md)**: 여러 commit에 걸쳐 작성자, 날짜, 메시지,
파일 내용을 다시 쓰는 워크벤치(`Ctrl+G Shift+H`). 미리보기, Lua 스크립트, 한 번의
실행 취소를 지원합니다.

## 키보드와 마우스

21. **[단축키](keybindings.md)**: 영역별 모든 키와 모든 팔레트 명령. 앱에서는 `?`가
    같은 목록을 보여 주고 `Ctrl+P`로 어떤 명령이든 찾을 수 있습니다.
22. **[마우스와 제스처](mouse-and-gestures.md)**: 클릭, 드래그, 선택과 복사,
    스크롤바, 호버 피드백.

## 설정과 연동

23. **[설정](settings.md)**: 설정 화면의 모든 페이지와 옵션, 검색 상자, 프로필,
    단축키를 다시 지정하는 방법.
24. **[AI 지원](ai.md)**: GitPoulpi 자체 AI(기본적으로 꺼짐): `claude -p` 같은
    로컬 에이전트, Anthropic, OpenAI, OpenRouter, Ollama, LM Studio. 지침, commit
    메시지, 설명, pull request 설명, commit composer, 개인정보와 문제 해결.
25. **[호스팅과 이슈 트래커](hosting-integration.md)**: GitHub, GitLab, Gitea와 Forgejo,
    Azure DevOps, Bitbucket, Gerrit, 이슈 트래커:
    토큰과 그 범위, 연결, 저장소 연결, SSH 키, 알려진 한계. 그리고
    **[로컬 이슈와 Obsidian](local-issues.md)**: Markdown 노트 폴더를 이슈 소스로 쓰기,
    토큰 없음, Obsidian Base 포함.

## 자동화

26. **[CLI 참조](cli-reference.md)**: 모든 `gitpoulpi` 명령, 스크립트용
    `--json`.
27. **[MCP와 에이전트](mcp-and-agents.md)**: 외부 에이전트(Claude Code, Cursor,
    Codex, ...)가 GitPoulpi를 조작하게 하기: 서버 등록, `.mcp.json`, 권한 스위치와
    기본값, 모든 도구, 에이전트가 사용자의 창에 무언가를 보여 주는 방법. 바로 쓸 수
    있는 에이전트 스킬은 [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md)입니다.

## 참조와 도움말

28. **[설정 참조](config-reference.md)**: 모든 `config.toml`과 `tui.toml` 키와 기본값,
    효과, 저장소별 설정, 환경 변수.
29. **[문제 해결과 FAQ](troubleshooting.md)**: 아이콘, 그래프 렌더링, 도착하지
    않는 키, 멈춘 데몬, 속도, LFS 포인터.

## 스크린샷

이 페이지들의 모든 그림은 가상의 작성자가 있는 작은 데모 저장소에서 릴리스 빌드를
실제로 캡처한 것입니다. 그림은 [`images/`](../images/)에 있습니다.
`scripts/docs-screenshots.sh`가 데모 저장소를 만들고 다시 캡처합니다.
그림은 commit 그래프와 작성자 아바타를 그림으로 그리는 `kitty` 터미널에서
찍었습니다. 그래픽 프로토콜이 없는 터미널에서는 대신 문자 그래프가 표시됩니다.

## 기여자를 위해

[CLAUDE.md](../../../CLAUDE.md), [아키텍처 개요](../../architecture/overview.md),
[로드맵](../../roadmap.md)이 설계와 계획된 내용을 설명하고,
[CHANGELOG.md](../../../CHANGELOG.md)에 릴리스별 변경 내용이 있습니다.

---

GitKraken은 해당 소유자의 상표입니다. GitPoulpi는 독립 프로젝트이며 해당 소유자와 제휴, 승인 또는 후원 관계가 없습니다. 이 이름은 호환성과 영감을 설명하기 위해서만 언급됩니다.

그 외 GitPoulpi와 그 문서에서 언급되는 모든 제품 및 서비스 이름은 해당 소유자의 상표입니다. 예를 들어 코드 호스팅 서비스(GitHub, GitLab, Bitbucket 등), 이슈 트래커(Jira, Trello, Linear 등), GitPoulpi가 열 수 있는 편집기와 도구(Visual Studio Code, JetBrains IDE, lazygit 등)가 있습니다. 이 이름들은 GitPoulpi가 함께 동작하는 대상을 가리키기 위해서만 사용되며, GitPoulpi는 이들 중 어느 것과도 제휴, 승인 또는 후원 관계가 없습니다.
