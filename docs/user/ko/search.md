[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · **한국어** · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# 검색

`Ctrl+Alt+F`(또는 `Ctrl+P`를 누른 뒤 `검색: 워크스페이스...`)는 **작업 중인 모든 저장소**를
대상으로 하는 퍼지 검색기를 엽니다. 활성 [워크스페이스](workspaces.md)가 있으면 그
저장소들이, 없으면 열린 탭이 대상입니다. **검색** 버튼(제목 표시줄에서 톱니바퀴 바로 왼쪽의
돋보기와 레이블)으로도 열립니다. 다음을 찾습니다.

- **저장소**(이름, 그다음 경로),
- **브랜치**(로컬과 리모트 추적)와 **tag**,
- **worktree**(폴더, 그다음 경로와 브랜치),
- **stash**(메시지),
- **commit**(제목. commit 본문은 제목이 일치하지 않고 세 글자 이상 입력했을 때만
  검색합니다),
- **pull 또는 merge request**와 **이슈**(제목, `#번호`, 작성자, 브랜치).

![열린 저장소 두 개 위의 검색 팝업: "main"과 일치하는 브랜치, stash, commit과 오른쪽의 각 저장소](../images/workspace-search.png)

결과는 종류별로 묶이고 각 묶음 안에서는 가장 잘 맞는 것이 먼저 옵니다. 모든 행에는
종류의 아이콘, 강조 표시된 일치 문자, 흐린 세부 정보(브랜치의 upstream, 요청의 작성자와
브랜치, stash의 위치), 그리고 오른쪽의 저장소 이름이 있어서 `main` 브랜치가 있는 두
저장소를 혼동할 일이 없습니다. 저장소가 아직 commit을 인덱싱하는 동안에는 제목에
스피너가 나타납니다(큰 저장소에서의 첫 검색은 잠시 걸릴 수 있습니다). 다른 종류는
즉시 나타나고 commit은 준비되는 대로 합류합니다.

## 이동하기

| 키 | 동작 |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | 결과 사이를 이동합니다(묶음 제목은 건너뜁니다). |
| `Enter` | 결과로 이동합니다. 저장소의 탭이 열리고(또는 전환되고) 결과가 표시됩니다. |
| `Esc` | 검색을 닫습니다. |
| `Alt+1` .. `Alt+8` | 종류를 켜거나 끕니다(목록 위의 칩, 그 순서대로). `Alt+0`은 모두 켭니다. |

"이동"의 의미는 다음과 같습니다.

| 결과 | Enter |
|---|---|
| 브랜치 또는 tag | Refs 창에서 선택하고 Log에서 그 commit을 선택합니다. |
| Commit | Log에서 선택합니다. 불러온 것보다 오래된 commit이면 먼저 Log를 더 불러옵니다. |
| Stash | STASHES 섹션에서 그 행을 선택합니다. |
| Worktree | 탭으로 엽니다. |
| Pull/merge request, 이슈 | (그 저장소의 탭에서) 사이드바의 그 행을 선택합니다. |
| 저장소 | 그 탭으로 전환합니다(열려 있지 않으면 엽니다). |

`Ctrl+N Backspace`(또는 팔레트의 `뒤로`)는 에이전트가 대신 바꿔 준 보기처럼 이전에 있던
곳으로 돌아갑니다. 어딘가로 이동한다고 해서 체크아웃, merge 등 변경되는 것은 없습니다.

마우스로는 행을 클릭하면 선택되고, 더블 클릭하면 이동하며, 칩을 클릭하면 그 종류가
켜지거나 꺼지고, 휠로 스크롤하며, 바깥을 클릭하면 팝업이 닫힙니다.

## 접두사로 좁히기

쿼리를 접두사로 시작하면 칩과 관계없이 한 종류만 검색합니다.

| 접두사 | 종류 |
|---|---|
| `r:` | 저장소 |
| `b:` | 브랜치 |
| `t:` | tag |
| `w:` | worktree |
| `s:` | stash |
| `c:` | commit |
| `pr:`(또는 `mr:`, `p:`) | pull 또는 merge request |
| `i:` | 이슈 |
| `#` | pull/merge request와 이슈(`#12`는 12번을 찾습니다) |

`fix: login`은 접두사가 아니라 평범한 쿼리입니다. 한 글자(또는 `pr:` / `mr:`) 뒤에 콜론이
오거나 맨 앞에 `#`이 있는 경우만 접두사로 칩니다.

## 일치 방식

명령 팔레트와 같은 세 단계입니다. 입력한 텍스트 그대로, 그다음 어디든 그대로 포함,
그다음 문자가 순서대로이지만 떨어져 있는 경우. 앞에 있는 일치와 짧은 텍스트가
우선합니다. 대문자와 소문자는 같게 취급합니다.

## Pull request와 이슈

GitPoulpi가 저장소마다 이미 불러온 것(사이드바의 PULL REQUESTS와 ISSUES 섹션 뒤의
목록)에서 가져오며, 저장소의 `.gitpoulpi/` 폴더에 있는 작은 캐시 파일에 보관됩니다.
검색은 **GitHub나 GitLab을 절대 호출하지 않습니다**. 창에서 요청을 한 번도 불러오지
않은 저장소에는 찾을 것이 없고, 보이는 내용은 사이드바의 마지막 새로 고침만큼만
최신입니다. 연결된 계정이 없으면 찾을 것도, 설정할 것도 없습니다.

## 명령줄에서

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]`은 같은 일을 하고
JSON을 출력합니다([CLI 참조](cli-reference.md#워크스페이스와-검색) 참고). MCP 서버에는
`search` 도구가 있어서 에이전트도 사용자가 찾는 것을 찾을 수 있습니다.
[설정](config-reference.md#search-워크스페이스-검색)의 `search.commit_limit`은 저장소마다
인덱싱하는 commit 수를 제한합니다(`0`이면 commit 검색을 끕니다).

## 키 바꾸기

`검색: 워크스페이스...`는 팔레트 명령이고 키는 `global.workspace_search` 동작입니다.
설정 > 키보드 및 마우스에서, 또는 `[keybindings.global]` 표(`workspace_search = "Ctrl+Alt+F"`)에서
다시 할당하세요.
