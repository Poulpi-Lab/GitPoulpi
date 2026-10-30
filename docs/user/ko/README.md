[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · **한국어** · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# GitPoulpi 사용자 문서

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi 문어" width="128"></p>

GitPoulpi는 클릭할 수 있는 commit 그래프, 명령 팔레트, 호스팅 및 이슈 트래커 연동,
선택 사항인 AI 도움말, 그리고 UI가 하는 모든 일을 하는 CLI와 MCP 서버를 갖춘
터미널용 Git 클라이언트입니다. 이 페이지들은 읽는 순서대로 되어 있습니다:
맨 위부터 시작하고, 필요할 때 참조 페이지로 건너뛰세요.

![GitPoulpi 메인 창](../images/hero.png)

## 여기서 시작

1. **[설치](installation.md)**: 필요한 것, 릴리스 아카이브, 소스에서 빌드하기.
   더 작은 바이너리가 필요하면 **[사용자 지정 빌드](custom-builds.md)**를
   참고하세요.
2. **[시작하기](getting-started.md)**: 실행부터 첫 commit까지, 처음 5분.
3. **[인터페이스 둘러보기](interface-tour.md)**: 창의 모든 영역의 이름을 그림과
   함께 설명합니다.

## 일상적인 사용

4. **[Log 다루기](log-and-history.md)**: 그래프 읽기, 검색, 브랜치, tag, stash,
   worktree, ref 드래그, rebase, cherry-pick, reword와 amend, 배너, pull과 push,
   auto-stash.
5. **[스테이징과 commit](staging-and-committing.md)**: 작업 중인 변경 사항, diff,
   hunk와 한 줄 단위 스테이징, commit 작성, commit composer, blame, 파일 기록,
   내장 편집기.
6. **[그림](image-view.md)**: Diff 보기와 파일 보기의 이미지, 이전과 이후, 확대,
   편집 프로그램.
7. **[merge 충돌 해결](merge-conflicts.md)**: merge 편집기(출력 위에 A와 B를
   나란히), 줄과 덩어리 선택, 바이너리, LFS, 삭제, 심볼릭 링크, 서브모듈 충돌용
   카드, 에이전트가 해결책을 제안하는 방법.
8. **[실행 취소와 다시 실행](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, 취소할 수 있는
   것과 없는 것, 거부하거나 묻는 경우, 얼마나 거슬러 올라가는지, 끄는 방법.
9. **[Git LFS](lfs.md)**: 큰 파일, 추적, 잠금, `git-lfs`가 설치되어 있지 않을 때
   일어나는 일.
10. **[서브모듈](submodules.md)**: 서브모듈 섹션, 업데이트하기, 체크아웃과 pull에
    맞춰 최신으로 유지하기.
11. **[테마](themes.md)**: 내장 테마, 어떤 테마든 어떤 레인 팔레트와도 조합,
    실시간 미리 보기, 직접 만든 테마 파일, `gitpoulpi theme`.
12. **[워크스페이스](workspaces.md)**: 고유한 탭과 선택적 기본 commit 프로필을 가진,
    이름이 붙은 저장소 묶음, 탭 표시줄의 칩, 워크스페이스 탭,
    모두 fetch, 모두 pull, 누락된 저장소 클론.
13. **[검색](search.md)**: 하나의 퍼지 검색기(`Ctrl+Alt+F`)로 워크스페이스 또는
    열린 탭의 모든 저장소의 브랜치, tag, worktree, stash, commit,
    pull/merge request, 이슈를 찾습니다.
14. **[Radar](radar.md)**: 워크스페이스(또는 열린 탭)의 pull/merge request,
    이슈, 진행 중인 작업을 하나의 목록으로: 필터, 고정, 미루기, 그리고 각 행에서
    할 수 있는 일.
    앱에서 그 요청을 읽고 리뷰하기, 댓글 스레드, 대기 중인 리뷰, 승인 / 변경 요청:
    **[pull request 리뷰](review.md)**.
15. **[열기 위치](open-in.md)**: 활성 탭의 폴더를 터미널, 멀티플렉서, 편집기,
    파일 관리자에서 여는 제목 표시줄의 목록.
16. **[활성 탭의 폴더 따라가기](following-the-tab-folder.md)**:
    "종료 후 cd", 그리고 bash, zsh, fish, PowerShell용 `gitpoulpi shell-init`
    래퍼(`gp`).
    GitPoulpi를 가장 잘 보여 주는 터미널: **[터미널 고르기](terminals.md)**.

추가: **[브랜치 워크플로](branching-workflows.md)**. Git Flow, GitHub Flow, GitLab Flow,
Trunk-based 프리셋 또는 직접 정의한 베이스, 브랜치 이름 패턴, 토픽 유형
(`gitpoulpi flow ...`).

**[기록 다시 쓰기](history-rewrite.md)**: 여러 commit에 걸쳐 작성자, 날짜, 메시지,
파일 내용을 다시 쓰는 워크벤치(`Ctrl+G Shift+H`). 미리보기, Lua 스크립트, 한 번의
실행 취소를 지원합니다.

## 키보드와 마우스

17. **[단축키](keybindings.md)**: 영역별 모든 키와 모든 팔레트 명령. 앱에서는 `?`가
    같은 목록을 보여 주고 `Ctrl+P`로 어떤 명령이든 찾을 수 있습니다.
18. **[마우스와 제스처](mouse-and-gestures.md)**: 클릭, 드래그, 선택과 복사,
    스크롤바, 호버 피드백.

## 설정과 연동

19. **[설정](settings.md)**: 설정 화면의 모든 페이지와 옵션, 검색 상자, 프로필,
    단축키를 다시 지정하는 방법.
20. **[AI 지원](ai.md)**: GitPoulpi 자체 AI(기본적으로 꺼짐): `claude -p` 같은
    로컬 에이전트, Anthropic, OpenAI, OpenRouter, Ollama, LM Studio. 지침, commit
    메시지, 설명, pull request 설명, commit composer, 개인정보와 문제 해결.
21. **[호스팅과 이슈 트래커](hosting-integration.md)**: GitHub, GitLab, Gitea와 Forgejo,
    Azure DevOps, Bitbucket, Gerrit, 이슈 트래커:
    토큰과 그 범위, 연결, 저장소 연결,
    pull request와 이슈, commit 메시지의 참조, SSH 키, 알려진 한계. 그리고
    **[로컬 이슈와 Obsidian](local-issues.md)**: Markdown 노트 폴더를 이슈 소스로 쓰기,
    토큰 없음, Obsidian Base 포함.

## 자동화

22. **[CLI 참조](cli-reference.md)**: 모든 `gitpoulpi` 명령, 스크립트용
    `--json`.
23. **[MCP와 에이전트](mcp-and-agents.md)**: 외부 에이전트(Claude Code, Cursor,
    Codex, ...)가 GitPoulpi를 조작하게 하기: 서버 등록, `.mcp.json`, 권한 스위치와
    기본값, 모든 도구, 에이전트가 사용자의 창에 무언가를 보여 주는 방법. 바로 쓸 수
    있는 에이전트 스킬은 [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md)입니다.

## 참조와 도움말

24. **[설정 참조](config-reference.md)**: 모든 `config.toml` 키와 기본값,
    효과, 저장소별 설정, 환경 변수.
25. **[문제 해결과 FAQ](troubleshooting.md)**: 아이콘, 그래프 렌더링, 도착하지
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
