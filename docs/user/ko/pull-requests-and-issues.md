[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · **한국어** · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# pull request와 이슈

사이드바, 보드, 도구 모음에서 pull / merge request와 이슈를 읽고, 만들고, 관리합니다. 먼저 계정을 연결하고 저장소를 연결하세요: [호스팅 및 이슈 트래커 연동](hosting-integration.md)을 참고하세요. 이슈나 요청 하나에 대한 모든 것(댓글, 리뷰, 스레드)은 [대화](conversations.md)에, 요청을 읽고 리뷰하는 방법은 [pull 또는 merge request 리뷰](review.md)에 있습니다.

## 사이드바의 pull request와 이슈

### PULL REQUESTS

섹션 헤더에는 개수와 `+`(pull request 만들기), 제공자 및 프로젝트 드롭다운, 검색
상자, 상태 드롭다운(*열림*, *merge됨 / 닫힘*, *전체*)이 있습니다. 그 아래에 네
그룹이 개수와 함께 표시됩니다.

- **내 pull request**: 내가 작성자인 것.
- **나에게 할당됨**.
- **내 리뷰 대기 중**: 내가 요청받은 리뷰어인 것.
- **모든 pull request**: 처음에는 접혀 있습니다.

검색 상자는 모든 그룹을 필터링합니다(제목, 번호, 작성자, 브랜치). 상태 드롭다운을
바꿀 때마다 호스트에 다시 요청합니다.

행은 상태 글리프로 시작합니다. `●` 열림(녹색), `○` 초안, `◆` merge됨(보라색),
`✕` 닫힘(빨간색). 체크아웃된 브랜치의 요청은 굵게 표시됩니다. 행은
`#12  제목`(GitHub, Gitea, Forgejo, Bitbucket) 또는 `!53  제목`(GitLab, Azure DevOps) 형식입니다.

### 이슈

섹션 제목에는 제공자 이름이 붙습니다. `GITHUB 이슈`, `JIRA 이슈` 등입니다.
Azure DevOps는 `... 작업 항목`, Trello는 `... 카드`로 표시됩니다. 드롭다운 아래에는
검색 상자, 상태 드롭다운(*열림*, *나에게 할당됨*, *닫힘*, *전체*), 그리고 이슈가
있습니다. Jira 행은 `ABC-12  제목` 형식이며, 사이드바가 충분히 넓으면 유형과 상태가
흐리게 표시됩니다.

![사이드바의 ISSUES 섹션: 이슈 #7의 오른쪽 클릭 메뉴가 열려 있고 맨 위에 대화 열기가 있음](../images/issues-sidebar.png)

### 행에서 사용하는 동작

- `Enter` 또는 더블 클릭은 행을 엽니다: pull request는 [리뷰](review.md)로(GitHub, GitLab, Gitea, Forgejo, Gerrit), 이슈와 Azure DevOps나 Bitbucket의 요청은 창 안의 [대화](conversations.md#대화)로, 그 밖에는 브라우저에서 엽니다. 행 메뉴의 **브라우저에서 열기**는 항상 브라우저를 엽니다.
- 오른쪽 클릭 또는 `.`는 행 메뉴를 엽니다.
  - **대화 열기**(이슈, 그리고 리뷰 보기가 없는 요청), **브라우저에서 열기**, **URL 복사**, **번호 복사**.
  - pull request: **<provider>에서 편집**, **소스 브랜치 체크아웃**,
    **소스 브랜치 commit 보기**. fork에서 온 요청이면 체크아웃은
    **Pull request 체크아웃 (fork)**로 표시되고, **보기**는 회색으로 비활성화됩니다.
  - 이슈: **이슈에서 브랜치 만들기**. `12-fix-the-thing`(Jira는
    `ABC-123-fix-the-thing`) 같은 이름이 채워진 새 브랜치 프롬프트가 열립니다.
  - GitHub, GitLab, Gitea, Forgejo에서는 pull request 행에 **리뷰...**도 있습니다(행에서
    **Enter**도 같음): 앱 내 리뷰이며 [pull request 리뷰](review.md)를 참고하세요.
  - GitHub, GitLab, Gitea, Forgejo에서는 두 종류의 행 모두에 **담당자 편집...**,
    **레이블 편집...**, **마일스톤 설정...**이 있습니다(pull request에는 **리뷰어 편집...**도).
    [담당자, 리뷰어, 레이블, 마일스톤](#담당자-리뷰어-레이블-마일스톤)을 참고하세요.
- **<provider>에서 편집**에는 다른 진입 경로가 셋 더 있습니다. 마우스를 올리거나
  선택한 pull request 행 오른쪽 끝의 연필 `✎`, 선택한 행에서 `e`, 그리고 팔레트
  명령 *연동: 연결된 Pull request를 호스트에서 편집*(체크아웃된 브랜치의 요청용)입니다.
- 소스 브랜치가 아직 저장소에 없으면 GitPoulpi가 그렇게 알려 줍니다. 먼저
  fetch하세요(`Ctrl+G f`).

**pull request 체크아웃.** GitPoulpi는 요청을 이를 추적하는 로컬 브랜치
`pr/<n>`(GitLab은 `mr/<n>`)으로 fetch하고 그 브랜치로 전환합니다. GitHub, GitLab,
Gitea, Forgejo에서는 fork에서 온 요청에도 작동합니다(GitPoulpi가 호스트의 특수 head ref를
fetch합니다). Azure DevOps와 Bitbucket은 요청의 소스 브랜치를 fetch합니다. 이
둘에서 fork의 요청은 거부됩니다. fork를 리모트로 추가하고 그 브랜치를 직접
체크아웃하세요. 실행 취소하면 이전 브랜치로 돌아갑니다. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**명령줄(과 에이전트)에서 pull request 리뷰.** GitHub(Enterprise 포함), GitLab(자체 호스팅 포함), Gitea, Forgejo에서는
GitPoulpi를 떠나지 않고 요청을 읽고 리뷰할 수 있습니다: `hosting review show <n>`(파일, 병합 상태, 검사),
`hosting review threads <n>`(리뷰 스레드와 대화), `hosting review comment`, `hosting review reply`, `hosting review resolve`,
그리고 `hosting review submit <n> --event approve|request-changes|comment --comment path:line[:side]:text`.
diff는 호스트가 아니라 내 저장소에서 가져옵니다: `hosting review fetch <n>`은 요청의 head를 `refs/gitpoulpi/pr/<n>`으로,
대상 브랜치를 `refs/gitpoulpi/pr/<n>-base`로 fetch합니다(둘 다 브랜치가 아니며 아무것도 체크아웃하지 않습니다).
줄 번호는 그 diff의 것입니다: `new`는 head의 줄, `old`는 merge base의 줄입니다. GitLab에는 "변경 요청" 리뷰 상태가 없으므로
거기서는 거부되며 아무것도 전송되지 않습니다. Gitea와 Forgejo는 한 줄에만 댓글을 받습니다(범위 없음). 스레드는 파일의 한 줄에 달린 댓글들입니다.
Gitea는 스레드를 해결할 수 있지만 Forgejo 11은 할 수 없습니다(호스트에 그 엔드포인트가 없으며, 호출이 그렇게 알려 줍니다). 답글 엔드포인트가
없는 서버에서의 답글은 같은 줄에 댓글 하나짜리 리뷰로 게시됩니다. Gitea는 자신의 요청을 승인하거나 거부하는 것을 거부합니다.
클론 없이: `review show`, `threads`, `comment`, `reply`, `resolve`, `submit`, `suggest`에 `--project owner/name`(GitLab: `group/project`)을 붙이면 현재 저장소가 아닌 프로젝트를 리뷰할 수 있고, `hosting review diff <n> --project P`는 호스트 API에서 diff(파일별 hunk)를 읽으며, `hosting file <path> --project P [--ref R]`는 특정 리비전의 파일을 읽습니다(GitHub, GitLab, Gitea, Forgejo, Gerrit. Gerrit에서는 패치 세트의 diff를 파일별로 읽고 파일은 브랜치에서, commit id이면 해당 commit에서 가져옵니다). diff에 호스트가 보여 주지 않는 파일(너무 큼)은 `omitted`로 표시됩니다.
MCP 도구는 같은 명령입니다(`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`, `hosting_file`...).

### 담당자, 리뷰어, 레이블, 마일스톤

GitHub(Enterprise 포함), GitLab(자체 호스팅 포함), Gitea, Forgejo에서는 pull request나 이슈의 담당자, 리뷰어, 레이블,
마일스톤을 GitPoulpi를 떠나지 않고 바꿀 수 있습니다. 다른 제공자는 이 중 아무것도 표시하지 않습니다.

- **행 메뉴**: **담당자 편집...**, **리뷰어 편집...**(pull request만), **레이블 편집...**, **마일스톤 설정...**.
- **팔레트**: *호스팅: 담당자 편집...*, *호스팅: 리뷰어 편집...*, *호스팅: 레이블 편집...*, *호스팅: 마일스톤 설정...*은 각각 먼저
  사이드바의 요청과 이슈를 나열해 고르게 합니다. [Radar](radar.md)에서는 같은 네 가지가 선택한 행에 적용됩니다.
- 선택기는 즉시 열리고 호스트에서 프로젝트의 사람이나 레이블(그리고 항목의 현재 값)을 불러옵니다. 약 150ms 뒤에 스피너 행이 표시됩니다.
  입력하여 필터링(퍼지 일치)하고, `Up` / `Down`으로 이동하며, `Space`는 체크 또는 체크 해제, `Enter`는 적용, `Esc`는 취소, `Tab`은 버튼으로 이동합니다.
  행을 클릭하면 체크되고 휠로 스크롤합니다. 사람은 머리글자 배지가, 레이블은 색이 표시됩니다.
- 마일스톤은 드롭다운입니다(**마일스톤 없음**으로 지울 수 있음).
- **적용**하면 호스트가 업데이트되고, 목록과 Radar는 곧바로 시작되는 다음 새로 고침 뒤에 변경을 표시합니다.

생성 양식(pull request, 이슈)에는 같은 선택이 버튼 줄로 있습니다: `담당자: 없음`, `리뷰어: 없음`, `레이블: 없음`, `마일스톤: 없음`.
GitHub, GitLab, Gitea, Forgejo에서는 레이블 버튼이 이슈 양식의 레이블 입력란을 대체합니다. 호스트가 새 pull request는 받았지만 일부 레이블이나 사람을
거부하면 상태 표시줄이 그렇게 알려 줍니다.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`, `hosting issue edit <n>`, 그리고 `hosting pr`와
`hosting issue create`의 `--assignee`, `--reviewer`, `--label`, `--milestone`. [CLI 참조](cli-reference.md)를 참고하세요.

### 섹션의 키

| 키 | 동작 |
|---|---|
| `Enter` / 더블 클릭 | 호스트에 대화가 있으면 GitPoulpi에서 대화 열기 (GitHub, GitLab, Gitea, Gerrit의 pull request는 리뷰가 열림), 없으면 브라우저 |
| `o` | 행을 브라우저에서 열기 |
| `.` / 오른쪽 클릭 | 행 메뉴 |
| `e` | 선택한 pull request를 호스트에서 편집 |
| `n` | 새 pull request (PULL REQUESTS) 또는 새 이슈 (이슈) |
| `v` | 상태 드롭다운 열기 |
| `r` | 새로 고침 |
| 헤더에서 `Enter` / `Space` | 섹션 접기 또는 펼치기 |

### 다시 연결 및 오프라인

토큰이 없거나, 거부되었거나, 범위가 부족하면 섹션의 첫 행에
`다시 연결 (토큰 거부됨)`이 표시됩니다. 클릭하거나 `Enter`를 누르면 설정 >
연동에서 해당 프로필이 열립니다.

호스트에 연결할 수 없으면 마지막 목록이 화면에 남고 헤더에 그 경과 시간이
표시됩니다(`· 5분 전 데이터`). 마지막 응답은 디스크(`.gitpoulpi/hosting-cache.json`)에도
캐시되므로, 네트워크 응답이 오기 전에도 목록이 표시됩니다.

## 이슈 보드

이슈 헤더의 **보드** 버튼(팔레트: **이슈: 보드 열기**)은 연결된 소스의 이슈를 창 위의 칸반 보드로 보여 줍니다. 열은 소스에 따라 다릅니다:

| 소스 | 열 | 카드 이동 |
|---|---|---|
| 로컬 Markdown 노트 | 상태(프로필의 `statuses`, 그다음 사용 중인 다른 상태) | 노트의 `status`를 다시 씀 |
| GitHub, GitLab, Gitea, Forgejo | **열림**, 그다음 고른 레이블, 그다음 **닫힘** | 상태 레이블을 교체함. **닫힘**으로 옮기면 이슈를 닫고, 다시 열로 옮기면 다시 엶 |
| Trello | 보드의 열려 있는 리스트, 왼쪽에서 오른쪽으로 | 카드를 그 리스트로 옮김 |
| Redmine | 이슈 상태 | 이슈의 상태를 바꿈(Redmine의 워크플로 규칙이 적용됨) |
| Jira(Cloud와 Data Center) | 프로젝트의 상태, 해야 할 일이 먼저, 완료가 마지막 | 그 열로 이어지는 워크플로 전환을 실행함(`GET` 다음 `POST .../transitions`). 그런 전환이 없으면 카드가 돌아가고 상태 표시줄에 `no transition from X to Y`가 표시됨 |
| Azure DevOps(Services와 Server) | 프로젝트 작업 항목 유형의 state를 state 범주별로 | `System.State`를 설정함. 작업 항목의 유형에 그 열의 state가 없으면 카드가 이유와 함께 돌아감(`a Bug has no state Done`) |

GitHub 또는 GitLab 프로젝트에서는 **열**(또는 `c`, 또는 **보드: 열 레이블 설정...**)이 열로 쓸 레이블을 순서대로 묻습니다. 예:
`status::todo, status::doing, status::review`. 프로젝트의 레이블 중 상태처럼 보이는 것을 제안합니다. 선택은 저장소별로 저장됩니다.
레이블이 없으면 보드는 열림 / 닫힘입니다. 카드를 옮기면 나열한 상태 레이블만 건드리며 다른 레이블은 그대로입니다. 호스트 보드는 열린 이슈와 최근에 닫힌 이슈 60개를 나열합니다.

Jira 전환이 필수 필드(예: 해결 상태)를 요구하면 Jira가 거부하고 카드는 Jira의 메시지와 함께 돌아갑니다. 그런 이동은 사이트에서 하세요.

같은 보드를 스크립트와 에이전트도 쓸 수 있습니다. `gitpoulpi hosting board`는 열과 카드를 출력하고,
`gitpoulpi hosting board move <card> <column>`은 카드를 옮깁니다([CLI 참조](cli-reference.md) 참고).

카드에는 번호, 제목, 담당자 머리글자, 레이블, 마일스톤이 표시됩니다. 다른 열로 드래그하거나, `Shift+Left` / `Shift+Right`, 또는 **보드: 카드 이동...**(`m`)을 쓰세요.
카드는 바로 움직이며, 약 150ms 뒤에 스피너가 호스트가 아직 작업 중임을 알려 주고, 호스트가 거부하면 카드가 돌아가고 상태 표시줄이 이유를 알려 줍니다.
`Enter`는 이슈의 대화를 엽니다(읽을 대화가 없는 호스트에서는 호스트의 페이지). `e`는 담당자, 레이블, 마일스톤을 편집하고, `n`은 선택한 열에 새 이슈를 시작하며, `a`, `l`, `M`, `/`는
담당자, 레이블, 마일스톤, 텍스트로 필터링하고 `x`는 필터를 지우며, `r`은 호스트에서 다시 읽습니다. 보드는 사이드바의 목록이 새로 고쳐질 때 다시 읽습니다.
Radar에는 보드가 없습니다.

## pull request 만들기

PULL REQUESTS 헤더의 `+`, 섹션에서 `n`, 또는 팔레트 명령
*Pull request: Pull request 만들기...*로 양식을 엽니다. 브랜치 메뉴에도
**<branch>에서 pull request 시작...**이 있습니다.

양식은 다음으로 구성됩니다.

- pull request를 지원하는 연결된 프로필마다 탭이 하나씩 있습니다(탭 줄에서
  `Left` / `Right`).
- **원본 저장소 / 브랜치**와 **대상 저장소 / 브랜치** 콤보박스. 긴 목록은 검색할 수
  있습니다. 목록에 없는 브랜치나 저장소를 직접 입력할 수도 있습니다.
- **제목**: 팁 commit의 제목으로 채워집니다.
- **설명**: 여러 줄(`Enter`는 줄 바꿈, `Tab`은 입력란을 벗어남).
- **담당자**, **리뷰어**, **레이블**, **마일스톤** 버튼(GitHub, GitLab, Gitea, Forgejo): 선택기를 엽니다.
- **초안으로 제출**.
- [AI](ai.md)를 켜면: **제목과 설명 생성**.

양식은 체크아웃된 브랜치의 프로필, 저장소, 브랜치(`feature -> main`)로 열립니다.
원본 저장소가 다르면 fork에서 보내는 요청이 됩니다.

**Pull request 만들기**(또는 `Ctrl+S`)는 호스트에 요청을 만듭니다. 결과는 브라우저에서
열리고 상태 표시줄에 이름이 표시됩니다.

## 이슈 만들기

이슈 섹션에 제공자와 프로젝트가 **모두** 있으면 헤더에 `+`(*+ 새 이슈*)가
나타납니다. 섹션에서 `n`을 누르거나 팔레트 명령 *호스팅: 새 이슈...*를 써도 됩니다.

양식에는 **제목**(필수), **설명**(여러 줄), **레이블**(쉼표로 구분, 예: `bug, ui`. GitHub, GitLab, Gitea, Forgejo에서는
레이블, 담당자, 마일스톤 버튼이 대신 선택기를 엽니다)이 있습니다. Jira와 Azure DevOps는 **유형** 드롭다운이 추가되고, Trello는 **리스트**
드롭다운이 추가됩니다. Linear와 YouTrack은 **상태** 드롭다운(이슈가 시작하는 워크플로 상태 또는 `State` 값)이 추가됩니다. Redmine은 프로젝트의 트래커(Bug, Feature,
Support...)를 담은 **유형** 드롭다운이 추가되며, 레이블이 없으므로 레이블 버튼도 없고 명령줄에서 준 레이블은 거부됩니다. 양식은 열릴 때 호스트에서 이 드롭다운의 항목을 채웁니다.

**이슈 만들기**(또는 `Ctrl+S`)는 호스트에 이슈를 만듭니다. `Esc` 또는 **취소**는
양식을 닫습니다. 호스트가 거부하면(토큰 거부, 범위 부족, 알 수 없는 프로젝트,
알 수 없는 레이블) 양식에 이유가 표시되고 입력한 내용은 유지됩니다. 성공하면
상태 표시줄에 새 이슈가 표시되고 이슈 섹션이 새로 고침됩니다.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## 호스트의 웹 페이지에서 만들기

새 이슈 양식과 Pull request 만들기 양식의 오른쪽 위에는
**<provider>에서 만들기** 버튼이 있습니다. 키는 `Alt+W`입니다. 지금까지 입력한
내용으로 채워진 호스트의 웹 양식을 브라우저에서 엽니다. 대화상자는 그대로
열려 있습니다. 토큰이 필요 없고 API 호출도 하지 않으므로 API 오류가 난 뒤에도
작동합니다.

- GitHub에는 *초안*에 해당하는 URL 매개변수가 없으므로 초안 체크는 전달되지
  않습니다. GitLab은 제목에 `Draft: ` 접두사가 붙습니다.
- GitLab은 레이블을 id로만 받으므로, 레이블은 설명 끝에 `/label ~"x"` 퀵 액션으로
  추가됩니다.
- 매우 긴 설명은 URL이 6000자를 넘지 않도록 잘리며 `[...truncated: paste the rest]`
  메모가 붙습니다.
- Bitbucket Data Center에는 웹 양식 주소가 없으므로
  **<provider>에서 만들기** 버튼이 없습니다.

CLI: `gitpoulpi hosting issue create ... --web`과 `gitpoulpi hosting pr --web`은
같은 URL을 출력합니다.

## 도구 모음 칩

도구 모음 끝(기본값은 오른쪽)의 칩은 체크아웃된 브랜치의 요청을 보여 줍니다. `GH #12`(GitHub),
`GL !53`(GitLab), `AZ !12`(Azure DevOps), `BB #12`(Bitbucket)입니다. 색은 상태를
나타냅니다. 녹색은 열림, 회색은 초안, 보라색은 merge됨, 빨간색은 닫힘입니다.

- 마우스를 올리면 제목, 상태, `source → target`이 표시됩니다.
- 클릭하거나 `Ctrl+G g`를 누르면 브라우저에서 요청이 열립니다.
- 오른쪽 클릭하면 **브라우저에서 열기**, **URL 복사**, **새로 고침**,
  **프로필 전환**이 나옵니다.

push했지만 요청이 없는 브랜치에는 흐린 `+ PR` 칩이 표시됩니다. 클릭하면 pull
request를 시작합니다. 일치하는 프로필이 없거나, 브랜치를 push하지 않았거나, 호스트에
연결할 수 없으면 칩이 없습니다. 창이 좁으면 도구 모음 버튼 공간을 확보하기 위해
칩이 숨겨집니다.

## 목록이 새로 고침되는 때

목록은 다음 경우에 새로 고침됩니다.

- 저장소를 열 때
- 체크아웃된 브랜치가 바뀔 때
- push 또는 fetch 후
- *pull request 및 이슈 새로 고침(분)*마다
  (`general.hosting_refresh_minutes`, 기본값 5, `0`이면 타이머 끔.
  설정 > 연동)
- 설정을 닫을 때
- `r`을 누르거나, *연동: Pull request 및 이슈 새로 고침*을 실행하거나, 필터를
  바꿀 때

요청이 창을 멈추게 하는 일은 없습니다.

호스트는 호출 횟수를 제한합니다. GitPoulpi는 마지막 응답의 `ETag`를 다시 보내므로
바뀌지 않은 목록은 비용이 거의 들지 않습니다(GitHub에서는 전혀 들지 않습니다).
호스트가 남은 호출 횟수가 적다고 알리면 상태 표시줄에 표시됩니다. 남은 횟수가
없으면 한도가 초기화될 때까지 GitPoulpi는 그 호스트에 요청하지 않으며 목록은
그대로 유지됩니다.

## 용어

GitHub, Azure DevOps, Bitbucket은 "pull request"라고 하고 GitLab은 "merge
request"라고 합니다. *리뷰 요청 용어*(`ui.pr_terminology`, 설정 > 연동)는
모든 레이블, 메뉴, 팔레트 명령, 메시지에 쓸 단어를 정합니다.

- **자동 (호스트별)**: 저장소에 연결된 제공자를 따릅니다(아무것도 연결되지 않은
  동안은 "pull request").
- **Pull request** 또는 **Merge request**: 항상 그 단어를 씁니다.

변경은 즉시 적용됩니다. CLI와 MCP는 항상 "pull/merge request"라고 합니다. 이
페이지는 "pull request"라고 씁니다.

## commit 메시지와 브랜치 이름의 참조

저장소의 이슈나 pull request가 연결되어 있으면 Commit details의 commit 메시지에 있는
참조에 밑줄이 표시됩니다. 클릭하면 연결된 페이지가 열립니다. 키보드로는 팔레트 명령
*연동: commit 참조 열기 (#issue / !request)...*를 사용합니다(메시지에 참조가 여럿이면
메뉴가 나옵니다). 아무것도 연결되지 않은 저장소에는 링크가 없습니다.

무엇이 참조로 인정되는지는 이슈 섹션의 제공자에 따라 다릅니다.

| 이슈 연결 대상 | 메시지 | 브랜치 이름 |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` 및 `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, `ABC`가 연결된 프로젝트의 키일 때만. `UTF-8`, `SHA-256`, `ISO-8601` 같은 텍스트는 링크가 되지 않습니다. | 이름 어디에든 있는 `ABC-123`(`feature/ABC-123-x`), 대문자 |
| Linear | `ENG-123`, `ENG`가 연결된 팀의 키(또는 프로필의 팀 목록에 있는 팀의 키)일 때만. `UTF-8`은 링크가 되지 않습니다. | 이슈에서 브랜치를 만들 때의 Linear 자체 `branchName`. 이름 어디에든 있는 `ENG-123`도 다시 읽힙니다 |
| YouTrack | `ABC-123`, `ABC`가 연결된 프로젝트의 약칭(또는 프로필의 프로젝트 목록에 있는 프로젝트의 약칭)일 때만 | 이름 어디에든 있는 `ABC-123` |
| Redmine | `#123`, Redmine이 연결된 이슈 소스일 때만(pull request 소스는 자기 기호를 유지함) | `123-fix-login` |
| Trello | `https://trello.com/c/<shortLink>` 주소. `#12`는 캐시된 카드 중 번호가 12인 카드가 있을 때만 | `12-fix-login` |

| PULL REQUESTS 연결 대상 | 메시지 |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

두 목록이 모두 `#N`과 일치하면 이슈가 우선합니다. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**대화에서.** 이슈나 요청의 텍스트는 그 호스트의 문법으로 읽으며, 이는 위의 표보다 넓습니다: `owner/repo#7`, `!7`, `GH-7`, `@login`, commit id와 [상호 참조](conversations.md#상호-참조)에 나열된 다른 형식. 저장소에 연결된 소스가 먼저 일치하므로 GitHub 댓글 안의 Jira 키도 앱에서 그 이슈를 엽니다. `gitpoulpi hosting markdown --body TEXT --id I`는 앱이 텍스트에서 찾아내는 것을 보여 줍니다.

### 이슈 참조 삽입

commit 작성기에서 요약 오른쪽 끝의 `#▾` 버튼은 연결된 이슈의 필터 가능한 목록을
엽니다. 현재 브랜치 이름이 가리키는 이슈가 *(브랜치에서)* 표시와 함께 맨 위에
옵니다. `Enter` 또는 클릭하면 그 참조(`ABC-123`, `#12`, `AB#12`)가 커서 위치에
삽입됩니다. 키보드: 작성기에서 `Alt+R`. 팔레트: *Commit: 이슈 참조 삽입...*.
연결된 이슈 목록이 없으면 버튼이 회색으로 비활성화되고 *먼저 사이드바에서 이슈 목록을
연결하세요*라고 표시됩니다. GitPoulpi는 참조를 스스로 추가하지 않습니다.
