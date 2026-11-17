[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · **한국어** · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# 대화

이슈, 작업 항목, 카드 또는 pull / merge request의 대화 보기입니다. 계정과 연결된 저장소가 필요합니다: [호스팅 및 이슈 트래커 연동](hosting-integration.md)을 참고하세요. 이를 여는 목록은 [pull request와 이슈](pull-requests-and-issues.md)에, 코드 리뷰는 [pull 또는 merge request 리뷰](review.md)에 있습니다.

**대화**는 이슈, 작업 항목, 카드, 변경 또는 pull / merge request 하나에 대해 오간 모든 것입니다: 설명, 댓글, 리뷰, 코드의 스레드, 이벤트(레이블, 할당, 상태 변경, push). GitPoulpi가 호스트에서 읽어 서식을 적용해 보여 주며, 창을 떠나지 않고 답할 수 있습니다.

### 여는 방법

| 위치 | 방법 |
|---|---|
| 이슈 행 | `Enter` 또는 더블 클릭. 행 메뉴의 **대화 열기**. 팔레트 *호스팅: 이슈 대화 열기...* (화면에 있는 이슈 목록). 행 메뉴의 **브라우저에서 열기**는 브라우저를 그대로 엽니다. |
| Azure DevOps나 Bitbucket의 PULL REQUESTS 행 | `Enter` 또는 **대화 열기**. GitHub, GitLab, Gitea, Forgejo, Gerrit에서는 행이 [리뷰](review.md)를 열며, 그 개요 페이지가 바로 이 대화입니다(타임라인, 키, 아래의 댓글 상자). |
| [이슈 보드](pull-requests-and-issues.md#이슈-보드)의 카드 | `Enter` 또는 더블 클릭. |
| [Radar](radar.md) | 이슈나 요청 행에서 `i`, 행 메뉴의 **대화 열기**, 또는 *Radar: 이슈 대화 열기*. |
| 다른 대화 안의 참조 | 클릭하거나(또는 `l`을 누르고 선택): 연결된 소스의 `#7`, `!12`, `ABC-12`는 앱 안에서 열리고 뒤로 스택에 쌓입니다. |
| 에이전트 | `gitpoulpi ui show issue <n> --view conversation` (`ui show pr`도 가능). [CLI 참조](cli-reference.md)를 참고하세요. |

**복제하지 않은 저장소.** 복제본이 없는 저장소의 Radar 행도 대화를 엽니다: 이 보기에는 연동 프로필과 프로젝트만 필요하므로 Radar가 자기 창 위에 띄우며, `Esc`로 Radar로 돌아갑니다. 복제가 필요한 것은 빠집니다: commit 참조는 Log 대신 호스트에서 열리고, *AI가 저장소를 읽도록 허용*은 이유와 함께 회색이 됩니다([AI 장](ai.md#호스트에서-쓰기-도움) 참고). 복제된 저장소의 요청에는 리뷰가 있고(`e`), 그 요청에 대해 Radar의 **대화 열기**는 여기서 읽을 대화가 없다고 알립니다. 복제된 저장소의 이슈는 그 저장소의 탭에서 대화를 엽니다.

읽을 대화가 없는 호스트의 이슈(Bitbucket과 Gerrit에는 이슈가 없음)는 이전처럼 브라우저가 열리며, `conversation` 기능이 없는 빌드는 어디서나 그렇게 합니다([커스텀 빌드](custom-builds.md)).

### 보기

전체 창 보기입니다: 헤더(상태, 연 사람과 시각, 레이블, 댓글 수, **이벤트 표시**, **뒤로**, **새로고침**, **브라우저에서 열기**), **게시** 줄, 타임라인, 그 아래의 댓글 상자.

![이슈 #7의 대화: #9로 가는 링크가 있는 설명, 댓글 두 개, 이벤트 표시로 보이는 레이블 이벤트, 쓰기, 미리보기, 분할이 있는 빈 새 댓글 상자](../images/issue-conversation.png)

![merge request !58의 개요 페이지: 설명, 파일과 줄 링크가 있는 미해결 및 해결된 코드 스레드, 댓글, 선택한 스레드의 답글과 인용, 댓글 상자](../images/pr-conversation.png)

- 타임라인은 오래된 것부터 최신 순이며 가장 최근 범위(최대 300개 항목)에서 시작합니다. 맨 위의 **이전 항목 불러오기**(`m`, 팔레트 *대화: 이전 항목 불러오기*)는 그 앞의 범위를 읽으며, 호스트가 알려 주면 남은 개수도 표시합니다.
- 댓글에는 작성자, 날짜, `수정됨` 칩, 봇에는 `봇` 칩, 비공개 메모에는 `내부용`이 표시되고, 선택한 항목이 허용하는 **답글**, **인용**, **편집**, **삭제**, **링크** 버튼이 나옵니다. 코드의 스레드는 `path:line`, `해결됨` 또는 `미해결`, `오래됨`, 그리고 그 댓글들을 보여 줍니다. 리뷰는 그 판정을 보여 줍니다.
- 이벤트는 흐리게 한 줄씩 표시됩니다(*carol이 레이블 lfs를 추가했습니다*). **이벤트 표시**(`v`)가 숨기며, 선택은 저장소별로 유지됩니다. 이벤트 기록이 없는 호스트(YouTrack, Azure DevOps 작업 항목, 로컬 노트)에는 이 확인란이 없습니다.
- 텍스트는 렌더링된 Markdown입니다: 제목, 중첩 목록과 작업 목록, 인용, GitHub 스타일 알림, 강조된 코드, 표, 링크, `[image: alt]`로 표시되는 이미지. 호스트 자체의 마크업으로 온 텍스트는 먼저 변환됩니다([마크업 변환](#마크업-변환) 참고). `mermaid` 코드 블록에는 다이어그램의 소스를 스크롤 가능한 뷰어에 보여 주는 **다이어그램 열기** 버튼이 있고(`Enter`는 복사), 팔레트에는 *대화: 항목의 다이어그램 열기*가 있습니다. `conversation`과 `mermaid`가 모두 있는 빌드가 필요합니다.
- 리뷰의 개요에서 코드 스레드의 `path:line`은 코드로 가는 링크입니다(`G`, *대화: 스레드의 코드 열기*, [리뷰](review.md) 참고). 단독으로 연 대화(이슈, Radar 행)에는 이동할 코드 페이지가 없습니다.
- 보기는 60초마다, 그리고 `F5`(**새로고침**)에서 다시 읽습니다.

**키.** 각 키에는 버튼, 메뉴 행 또는 팔레트 행(*대화: ...*)이 있으며, `Ctrl+N`은 버튼에 배지를 붙입니다.

| 키 | 동작 |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | 이전 / 다음 항목 |
| `PageUp`, `PageDown`, `Home`, `End`, 휠, 스크롤바 | 스크롤 |
| `Tab`, `Shift+Tab` | 타임라인, 댓글 상자, 그다음 헤더 버튼 |
| `r` | 답글: 선택한 항목의 스레드에, 스레드가 없는 호스트에서는 그 텍스트를 인용해서 |
| `Q` | 선택한 항목을 인용해 새 댓글 작성 |
| `c` | 새 댓글 |
| `e`, `d` | 내 댓글 편집, 삭제 (삭제 전에 확인을 묻습니다) |
| `l`, `Enter` | 항목의 링크: `Enter`는 하나뿐이면 그것을 따라가고, 아니면 목록 메뉴를 엽니다 |
| `y`, `o` | 항목의 주소 복사, 브라우저에서 열기 |
| `n` | 다음 안 읽은 항목 |
| `v`, `m`, `F5` | 이벤트 표시/숨기기, 이전 항목 불러오기, 새로고침 |
| `Alt+Left`, `Backspace` | 뒤로 스택을 따라 뒤로 |
| `q`, `Esc` | 닫기 (상자에 텍스트가 있으면 먼저 묻습니다) |

### 쓰기

**새 댓글**(`c`, 또는 `Tab`으로 상자로), **답글**(`r`), **인용**(`Q`), **편집**(`e`)은 하나의 Markdown 입력창을 씁니다: **쓰기**, **미리보기**, **분할**(`Alt+P`는 쓰기와 미리보기를 전환, `Alt+L`은 분할을 켜며 너비 100열의 창이 필요). `Ctrl+Enter`(`Ctrl+S`, `Alt+Enter`도 가능) 또는 **보내기**가 게시하고, **취소**가 상자를 버리며, AI가 켜져 있으면 `Alt+E`가 **AI로 쓰기**를 엽니다([AI 장](ai.md#호스트에서-쓰기-도움)). 미리보기는 타임라인과 같은 렌더러로, 같은 참조와 함께 배치됩니다. 호스트가 Markdown을 받지 않으면 바닥글에 텍스트가 어떤 마크업으로 보내지는지 표시됩니다(*Jira wiki markup(으)로 보냄*, *Textile(으)로 보냄*).

![분할 보기에서 이슈 답글을 쓰는 중: 왼쪽에 Markdown 초안, 오른쪽에 렌더링된 미리보기, 그리고 보내기](../images/issue-reply-composer.png)

- 댓글은 타임라인에 바로 추가되고, 호스트가 응답하면 제자리를 잡습니다. 호스트가 거부하면 그 항목이 사라지고 상자가 내 텍스트와 이유와 함께 다시 열립니다.
- 보내지 않은 텍스트는 항목별로 보관되며(설정 폴더의 `remote-conversations.toml`) 다음에 그 상자를 열 때 그대로 있습니다. 보내거나 버리면 지워집니다. 텍스트가 있는 상자를 닫으면 먼저 묻습니다.
- 스레드(리뷰 스레드, GitLab 토론, Linear 댓글, Bitbucket이나 Azure DevOps 스레드)의 **답글**은 그 스레드에 게시됩니다. 호스트에 평평한 목록만 있으면 **답글**은 선택한 항목을 인용한(`> **작성자**`) 새 댓글을 시작하며, **인용**과 같습니다.
- **내부 메모**(확인란, `Alt+I`): GitLab과 Redmine에서는 댓글이 프로젝트 구성원에게만 보입니다. 그런 메모가 있는 호스트에서, 그리고 새 댓글에서만 표시됩니다.
- 게시는 외부로 나가는 동작입니다: 아무것도 기록되지 않으며 **GitPoulpi에서는 아무것도 되돌릴 수 없습니다**. 연결이 없으면 상자가 남고, **보내기**는 이유와 함께 회색이 되며, 헤더에 저장된 사본이 표시 중이라고 나옵니다.

**편집하거나 삭제할 수 있는 사람.** **편집**과 **삭제**는 내 댓글에만 있습니다. 호스트가 직접 알려 주면 그것이 우선이고(GitHub의 `viewerCanUpdate`와 `viewerCanDelete`, 다른 곳은 `can_edit`), 그다음에 내 로그인을 댓글의 작성자와 대소문자 구분 없이 비교합니다. App 봇의 댓글은 봇 계정에서 온 것이어야 합니다. GitHub App 봇의 로그인을 모르는 동안에는 이 설치가 게시한 댓글(저장소가 그 id를 보관)만 해당됩니다. 이름으로 추측하는 일은 없습니다. 스레드에서 **편집**과 **삭제**는 그 스레드의 내 댓글 중 마지막 것에 작용합니다. 편집이나 삭제를 할 수 없는 호스트([표](#호스트별로-할-수-없는-것) 참고)에서는 버튼이 이유와 함께 회색으로 표시됩니다.

### 게시 이름

헤더 아래 줄이 내 메시지가 어떤 이름으로 게시되는지 알려 주며, 같은 줄이 모든 상자에 있습니다.

| 줄 | 의미 |
|---|---|
| *ann(으)로 게시 (개인 토큰)* | 프로필의 토큰: 내 로그인(`GET /user`). |
| *ci-bot(으)로 게시 (저장소 토큰 명령)* | 저장소의 [토큰 명령](hosting-integration.md#8-github-app-봇으로-동작하기)이 호스팅 API에 쓰입니다. |
| *my-app[bot]\(으)로 게시 (GitHub App 봇)* | 설치 토큰. 이름은 **게시 이름**에서, 없으면 호스트에서 옵니다. |
| *GitHub App 봇으로 게시 (이름을 아직 알 수 없음: 설정에서 게시 이름을 지정하세요)* | 설정된 이름도, 알아낸 이름도 아직 없습니다. |
| *...(으)로 게시 (프로젝트 봇)* | GitLab 프로젝트 또는 그룹 액세스 토큰. |
| *kai(으)로 작성 (로컬 노트)* | Markdown 노트 폴더: 프로필의 `local.me`, 없으면 연결된 사용자. |
| *어떤 이름으로 게시할지 아직 알 수 없습니다. 설정에서 게시 이름을 지정하세요.* | 알아낼 수 있는 것이 없었습니다. |

**게시 이름**은 이름입니다: GitHub App이라면 `my-app[bot]`입니다. 모든 저장소에 적용하려면 **설정 > 연동 > 게시 이름**(`general.hosting_post_as`)에서, 한 저장소에만 적용하려면 **설정 > 저장소별 설정 > SSH 및 자격 증명 > 호스팅**에서 지정하며, 후자가 우선합니다. 비워 두면 GitPoulpi가 GitHub에 묻고(GraphQL `viewer { login }`. 설치 토큰이 답할 수도 있고 못 할 수도 있음), 그래도 모르면 처음 게시한 댓글의 작성자에서 이름을 알아내 저장소와 함께 보관합니다. 개인 토큰에는 필요하지 않습니다. [GitHub App 봇으로 동작하기](hosting-integration.md#8-github-app-봇으로-동작하기)를 참고하세요.

### 상호 참조

대화 안의 참조는 밑줄이 그어져 있고 클릭할 수 있습니다. 저장소에 연결된 ISSUES와 PULL REQUESTS 소스가 먼저 일치하므로, 연결된 Jira 프로젝트의 `ABC-12`는 GitHub 댓글 안에서도 앱에서 열립니다. 호스트 자체의 문법이 더하는 것은 링크 확인 뒤에 그 페이지를 엽니다. commit id는 저장소가 복제되어 있고 id가 7자 이상이면 Log에서, 아니면 호스트에서 열립니다. 코드 스팬, 펜스 블록, Markdown 링크, 이메일 주소, URL 안의 것은 참조가 아니며, `C#1`, `UTF-8`, `SHA-256`, `ISO-8601`도 아닙니다.

| 호스트 | 인식하는 것 |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, commit id |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7`(형제 프로젝트), `@user`, 칩으로 `~label`과 `%milestone`, commit id |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, commit id |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, `{uuid}` 멘션, commit id |
| Gerrit | `Change-Id` 값(`I`와 16진수 40자), commit id |
| Redmine | `#7`, `r123` 리비전, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`. 연결된 소스의 프로젝트나 팀에 대해서만 |
| Trello | 카드 주소(`trello.com/c/...`) |
| 로컬 노트 | `[[note]]`, `[[note\|alias]]`, 폴더의 다른 노트로 가는 상대 `.md` 링크 |

`gitpoulpi hosting markdown --body T --id I`는 앱이 텍스트를 어떻게 처리하는지 참조까지 포함해 출력합니다.

### 마크업 변환

Markdown을 쓰지 않는 호스트는 양방향으로 변환되므로 어디서나 Markdown으로 읽고 씁니다. 변환기가 이해하지 못하는 것은 읽을 수 있는 텍스트로 남습니다.

- **Jira**(Cloud와 Data Center)는 API v2에서 댓글을 wiki 마크업으로 저장합니다. 읽는 것은 Markdown으로 변환되고, 쓰는 것은 wiki 마크업으로 변환되며, 바닥글에 *Jira wiki markup(으)로 보냄*이 표시됩니다.
- **Redmine**은 텍스트를 Textile 또는 Markdown으로 서식화하는데 API가 이를 알려 주지 않습니다: 프로필의 **서버의 텍스트 형식** 행(설정 > 연동, `markup`, 기본값 Textile)이 GitPoulpi에 알려 줍니다. Textile은 읽을 때 Markdown으로, 쓸 때 다시 Textile로 변환됩니다.
- **Azure DevOps Server**는 작업 항목 댓글을 HTML로 반환하고 HTML로 받습니다: 그로부터 변환된 Markdown으로 표시되고 HTML로 기록됩니다. Azure DevOps Services와 pull request 스레드는 Markdown을 받습니다.
- **Gerrit**은 일반 텍스트입니다: Markdown으로 렌더링되지 않으며 입력한 그대로 전송됩니다.
- 모든 가져오기는 먼저 제어 시퀀스와 이스케이프 시퀀스를 제거하므로 댓글이 터미널을 움직일 수 없습니다.

### 저장된 사본, 이전 항목 불러오기, 안 읽음

- 연 대화는 저장됩니다(`<state dir>/conversations/<integration>/<project>/<kind>-<n>.json`, 사용자 전용, 토큰 없음, 파일당 최대 2 MB, 최근 50개 파일 유지). 항목을 열면 바로 그려지고 뒤에서 새로고침됩니다. 호스트에 연결할 수 없으면 저장된 사본이 *오프라인: 3분 전 저장된 사본을 표시 중 (이유)* 아래에 그대로 표시되고 보내기는 회색이 됩니다.
- **안 읽음:** 마지막으로 본 것이 대화별로 기억됩니다(최근 200개 유지). 다른 사람이 쓴 더 새로운 항목에는 강조 막대와 `새 글` 칩이 붙고, `n`이 다음 항목으로 이동합니다. 처음 여는 대화에는 새 것이 없습니다.

### 호스트별로 할 수 없는 것

아래의 모든 `no`는 실제 한계입니다: 컨트롤은 숨겨지거나 이유와 함께 회색이 되고, CLI는 아무것도 보내기 전에 호스트를 밝힌 오류로 답합니다.

| 호스트 | 항목 | 스레드에 답글 | 내 것 편집 및 삭제 | 스레드 해결 | 이벤트 | 내부 메모 | 작성 형식 |
|---|---|---|---|---|---|---|---|
| GitHub | 이슈, pull request | pull request | 둘 다 | pull request | 예 | 아니오 | GitHub Markdown |
| GitLab | 이슈, merge request | 예 (토론) | 둘 다 | merge request | 예 | 예 | GitLab Markdown |
| Gitea | 이슈, pull request | pull request | 둘 다 | pull request | 예 | 아니오 | Markdown |
| Forgejo | 이슈, pull request | pull request | 둘 다 | 아니오 (호스트에 엔드포인트 없음) | 예 | 아니오 | Markdown |
| Azure DevOps | 작업 항목, pull request | pull request | 둘 다 | 아니오 (상태는 표시하지만 변경 안 함) | pull request만 | 아니오 | Markdown (Server 작업 항목은 HTML) |
| Bitbucket (Cloud, Data Center) | pull request만 | 예 | 둘 다 | 아니오 (표시만, 변경 안 함) | 예 | 아니오 | Markdown |
| Gerrit | 변경만 | 예 | 아니오 (둘 다 편집 또는 삭제 불가) | 예 | 예 | 아니오 | 일반 텍스트 |
| Jira | 이슈 | 아니오 (평평한 목록) | 둘 다 | 아니오 | 예 (changelog) | 아니오 (프로젝트 역할 필요) | wiki 마크업 |
| Linear | 이슈 | 예 (댓글에 대한 답글) | 둘 다 | 아니오 | 예 | 아니오 | Markdown |
| YouTrack | 이슈 | 아니오 | 둘 다 | 아니오 | 아니오 | 아니오 | Markdown |
| Redmine | 이슈 | 아니오 | 편집만 (Redmine 5 이상), 삭제는 불가 | 아니오 | 예 (journals) | 예 (비공개 메모) | Markdown 또는 Textile |
| Trello | 카드 | 아니오 | 둘 다 | 아니오 | 예 (이동) | 아니오 | Markdown |
| 로컬 노트 | 노트 | 아니오 | 로컬 작성자의 것 | 아니오 | 아니오 | 아니오 | Markdown |

그 밖에: 대화에서 줄에 새 스레드를 시작할 수 있는 호스트는 없으며(리뷰에서 하세요), 반응은 GitHub에서만 표시됩니다. 호스트가 받는 길이를 넘는 텍스트는 거부됩니다(GitHub 65,536자, GitLab 1,000,000자, Azure DevOps 150,000자, Jira 32,767자, Trello 16,384자): CLI는 먼저 확인하고, 앱에서는 호스트 자체의 메시지가 표시되며 텍스트는 보관됩니다. 로컬 노트는 댓글을 노트 파일 자체의 마지막 `## Comments` 섹션에 `### 작성자 - 시각` 항목 하나씩으로 보관하므로 Obsidian과 셸에서도 보입니다.

명령줄에서는 같은 대화가 `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`, `comment-delete`, 그리고 `hosting review conversation`입니다([CLI 참조](cli-reference.md) 참고). 에이전트에게는 같은 이름의 MCP 도구입니다([MCP와 에이전트](mcp-and-agents.md) 참고).
