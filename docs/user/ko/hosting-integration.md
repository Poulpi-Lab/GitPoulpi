[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · **한국어** · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# 호스팅 및 이슈 트래커 연동

GitPoulpi는 다음 제품과 연결됩니다.

- **호스팅** (pull request, 이슈, 내 계정에서 clone, SSH 키):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea, Forgejo
  (Codeberg 포함). **Gerrit Code Review**는 리뷰 전용 호스팅입니다: 그 변경이 곧 pull request이며
  이슈는 없습니다.
- **이슈 트래커** (이슈만): Jira Cloud, Jira Data Center, Trello, Linear,
  YouTrack (Cloud와 Server), Redmine.
- **로컬 Markdown 노트** (이슈만, 토큰 없음, 네트워크 없음): Markdown 노트가 든 폴더, 예를 들어 Obsidian 볼트의 폴더입니다. [로컬 이슈와 Obsidian](local-issues.md)을 참고하세요.

모든 연동은 **개인 토큰**을 사용합니다. GitPoulpi 계정도, OAuth 로그인도
없습니다. 연동은 선택 사항이며 **저장소별로 직접 켭니다**. 저장소를 연결하기
전에는 아무것도 가져오지 않습니다. 토큰은 자격 증명 도우미에 저장되며
`config.toml`에는 절대 기록되지 않습니다. 일반 `git` 작업(fetch, pull, push)에는
연동이 필요 없습니다. 이미 `git`에 설정된 자격 증명을 그대로 사용합니다.

## 1. 호스트에서 토큰 만들기

연결 양식에 *<host>에서 토큰 생성* 링크가 있습니다. 이 링크는 알맞은 범위(scope)가
미리 선택된 호스트의 토큰 페이지를 엽니다. 직접 만들려면 다음을 따르세요.

| 호스트 | 위치 | 범위 |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (저장소, pull request, 이슈), `read:user` (본인 확인), `admin:public_key` (SSH 키를 추가하거나 제거할 때만) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. 토큰은 "All accessible organizations"가 아니라 **해당 조직용으로** 만드세요. 전역 토큰은 2026-12-01부터 작동하지 않습니다. |
| Azure DevOps Server | 서버의 같은 페이지 (`https://<server>/<collection>/_usersSettings/tokens`) | 같은 두 범위 |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. SSH 키를 쓰려면 `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`도 필요합니다. |
| Bitbucket Data Center | 프로필 사진 > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, `<host>/user/settings/applications` | `user` 읽기, `repository`, `issue`, `user` 읽기와 쓰기(마지막은 SSH 키용). Gitea는 사후에 범위를 보여 주지 않으므로 GitPoulpi가 빠진 범위를 경고할 수 없습니다: 거부된 호출이 그렇게 알려 줍니다. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. 키는 본인 권한으로 동작합니다. |
| YouTrack (Cloud와 Server) | 프로필 > Account Security > New token, `<host>/users/me?tab=account-security` | `YouTrack` 범위. 영구 토큰은 본인 권한으로 동작합니다. |
| Gerrit | Settings > HTTP Credentials > Generate new token (최신 서버. 이전 서버: Generate new password), `<host>/settings/#HTTPCredentials` | 없음. 토큰은 **HTTP 비밀번호**이며 본인 권한으로 동작하고, 사용자 이름을 함께 입력합니다. |
| Redmine | My account > API access key > Show, `<host>/my/account`. 관리자가 **Administration > Settings > API > Enable REST web service**를 켜 두어야 합니다. | 없음. 키는 본인 권한으로 동작합니다. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (범위 없는 기존 방식) | 없음. 토큰은 본인 권한으로 동작합니다. |
| Jira Data Center | Profile > Personal Access Tokens | 없음. 토큰은 본인 권한으로 동작합니다. |
| Trello | 먼저 `trello.com/apps/admin`에서 **API 키**를 받습니다 (Power-Up을 만든 뒤 API 키 생성). 그다음 `trello.com/1/authorize?...&key=<your key>`에서 **토큰**을 받습니다. | `read`, `write` |

토큰에는 "GitPoulpi" 같은 이름과 만료일을 지정하세요. 토큰을 복사해 두세요.
호스트는 토큰을 한 번만 보여 줍니다.

## 2. 연결하기

1. `Ctrl+,`를 눌러 **연동**을 선택합니다.
2. 왼쪽에서 제품을 선택합니다. 제품은 **호스팅**과 **이슈 트래커** 두 그룹으로
   나뉩니다.
3. **연결**을 누릅니다. 같은 제품의 계정을 하나 더 추가하려면 **+ 프로필 추가**를
   사용하세요.

   ![연동 페이지: 왼쪽의 제품 목록과 GitHub용 연결 버튼](../images/settings-integrations.png)

4. 양식을 채우고 토큰을 붙여넣습니다. 토큰은 입력하는 동안 그대로 표시되므로
   주변에 보는 사람이 없는지 확인하세요. 양식에는 필요한 범위가 나열됩니다.

   ![연결 양식: 개인 액세스 토큰 입력란, 범위 repo, read:user, admin:public_key, github.com에서 토큰을 생성하는 링크, 연결 및 취소 버튼](../images/settings-connect.png)

5. **연결**을 누릅니다. GitPoulpi가 백그라운드에서 호스트에 토큰을 확인합니다.
   그러면 페이지에 이름과 로그인이 적힌 카드, 녹색의 *연결됨*, SSH 키 섹션이
   표시됩니다. 필요한 범위 중 일부가 없는 토큰도 연결은 되며, 빠진 범위는 빨간색으로
   나열됩니다.

카드에서 **편집**은 양식을 다시 엽니다. **연결 해제**는 토큰을 삭제하고 프로필은
남깁니다. **제거**는 확인 후 둘 다 삭제합니다.

연동 페이지의 키보드: `Tab` / `Shift+Tab`은 제품 목록, 프로필 줄, 양식 사이를
이동합니다. `a`는 프로필 추가, `e`는 편집, `d` 또는 `Delete`는 제거, `o`는 토큰
페이지 열기, `Esc`는 한 단계 뒤로 갑니다.

### 각 양식에서 요구하는 것

*토큰 생성* 링크는 호스트 또는 조직을 입력한 뒤에만 작동합니다.

- **GitHub, GitLab:** 토큰만 필요합니다. **GitHub Enterprise**와
  **GitLab Self-Managed**는 **호스트 도메인**도 요구합니다
  (`gitlab.example.com`, `http://`를 입력하지 않으면 https). 점이 없는 이름(Docker 서비스나 LAN 컴퓨터)에는
  포트나 `http://`가 필요합니다: `gitea:3000`, `http://nas`. 자체 호스팅 서버는 평문 http일 수 있습니다. 예를 들어
  `http://localhost:8080`의 로컬 컨테이너입니다. 평문 http로 보낸 토큰은 암호화되지 않은 채
  전송되므로, http는 로컬 또는 신뢰할 수 있는 네트워크에서만 사용하세요.
- **Azure DevOps:** **조직**, 즉 `dev.azure.com/<organization>`의 이름입니다.
  `https://<org>.visualstudio.com` 주소도 사용할 수 있습니다. 조직이 둘이면
  프로필도 둘이 필요합니다. 저장소는 리모트에 적힌 조직의 프로필을 사용합니다.
  **Azure DevOps Server**는 **컬렉션 URL**을 요구합니다. 예:
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi는 Azure DevOps에 시스템 자격 증명
  도우미를 사용하지 않습니다. Git Credential Manager가 거기에 저장하는 것은
  개인 액세스 토큰이 아니라 로그인 토큰이기 때문입니다.
- **Bitbucket:** **Atlassian 계정 이메일**과 API 토큰입니다. 이메일이 비어
  있으면 토큰이 bearer 토큰으로 전송됩니다. 워크스페이스 또는 저장소 액세스
  토큰에 이 방식을 사용하세요. GitPoulpi는 토큰에서 워크스페이스를 찾습니다.
  **Bitbucket Data Center:** **서버 URL**과 HTTP 액세스 토큰입니다.
- **Gitea, Forgejo:** **호스트 도메인**과 토큰입니다. **Forgejo**는
  `https://codeberg.org`로 시작하며, 자체 호스팅 서버에서는 바꾸세요. Gitea에는 공개 기본값이
  없습니다. 로컬 서버에는 `http://localhost:3000` 주소가 동작합니다. 두 종류는 클라이언트 하나를
  공유하며(Forgejo는 같은 `/api/v1`을 가진 Gitea의 fork입니다), 토큰은 `Authorization: token <token>`으로
  전송됩니다. 시스템 자격 증명 도우미는 이 둘에 쓰이지 않습니다.
- **Jira Cloud:** **사이트 URL**(`acme.atlassian.net`), **Atlassian 계정 이메일**,
  기존 방식의 API 토큰입니다. 범위가 지정된 Jira 토큰은 지원하지 않습니다.
  **Jira Data Center:** **서버 URL**과 개인 액세스 토큰입니다.
- **Trello:** **API 키**와 **토큰**입니다. API 키는 공개 값이므로 `config.toml`에
  (`account`로) 보관됩니다. 토큰은 비밀 값이라 자격 증명 도우미에 들어갑니다.
  양식에는 링크가 둘 있습니다. *Trello에서 API 키 받기*와, 키를 입력한 뒤에 나타나는
  *토큰 생성*입니다. 키와 토큰은 URL이 아니라 헤더로 전송됩니다.
- **Linear:** **API 키**뿐입니다: Linear에는 공개 호스트가 하나뿐이므로 입력할 주소가
  없습니다. 키는 `Authorization` 헤더에 그대로 전송됩니다(Linear은 개인 키에 `Bearer`를
  쓰지 않습니다). commit 메시지의 `ENG-12` 참조가 올바른 페이지를 열게 하려면 프로필 URL을
  워크스페이스와 함께 저장하세요(`https://linear.app/acme`, CLI에서는 `--url`). 이슈 행
  자체에는 항상 정확한 주소가 들어 있습니다.
- **YouTrack:** **사이트 URL**(Cloud는 `acme.youtrack.cloud`, 그 밖에는 직접 운영하는 서버이며
  컨텍스트 경로가 있으면 함께, 예: `host/youtrack`)과 영구 토큰이며, 토큰은
  `Authorization: Bearer`로 전송됩니다. 같은 클라이언트가 Cloud와 Server를 모두 처리합니다(`<url>/api`).
- **Gerrit:** **서버 URL**(컨텍스트 경로가 있으면 함께), **사용자 이름**, 그리고 토큰으로 쓰는 HTTP
  비밀번호입니다. 이들은 HTTP basic 인증으로 전송되며, GitPoulpi는 서버의 `/a/` REST API와
  통신하고 각 응답이 시작하는 `)]}'` 줄은 무시합니다. 사용자 이름은 비밀이 아니므로 `config.toml`에
  (`account`로) 보관되고, 비밀번호는 자격 증명 도우미로 들어갑니다. 저장소는 PULL REQUESTS에서 Gerrit
  *프로젝트*에 연결되며(그 변경이 거기에 나열됩니다), Gerrit에는 이슈가 없고 변경은 push로 만들어집니다
  ([리뷰](review.md#gerrit)의 *리뷰용 push* 참고).
- **Redmine:** Redmine의 **사이트 URL**(하위 경로가 있으면 함께, 예: `host/redmine`)과 API 키이며,
  키는 `X-Redmine-API-Key`로 전송됩니다. Redmine이 403이나 HTML 페이지로 답하면 REST 웹 서비스가
  꺼져 있는 것입니다(관리자에게 요청하거나 **Administration > Settings > API**에서 켜세요).

Jira, Trello, Linear, YouTrack, Redmine은 이슈 트래커일 뿐입니다. pull request도, clone할 저장소도,
SSH 키도 없습니다. Bitbucket에는 이슈가 없습니다(Atlassian이 Bitbucket Issues를
종료했습니다). Bitbucket 저장소의 이슈 섹션은 대신 Jira 프로젝트에 연결하세요.

## 3. 토큰이 저장되는 곳

토큰은 시스템 **자격 증명 도우미**(Windows는 Git Credential Manager, Linux는
libsecret 또는 `pass`, macOS는 Keychain)에 저장됩니다. 실제 호스트의 자격 증명과
혼동될 수 없는 전용 항목을 사용합니다. 토큰은 `config.toml`에 기록되지 않고
출력되지도 않습니다.

자격 증명 도우미가 설정되어 있지 않거나 토큰을 거부하면, GitPoulpi는 토큰을
**이번 세션 동안만 메모리에** 보관하고 그 사실을 알려 줍니다. 재시작한 뒤에는
다시 연결하세요.

GitPoulpi는 프로필의 토큰을 다음 순서로 찾습니다.

1. 환경 변수 `GITPOULPI_TOKEN_<PROFILE ID>`. CI처럼 토큰을 저장하지 않고
   전달할 때 사용합니다.
2. 프로필에 저장된 토큰(`token_ref`).
3. GitHub와 GitLab 프로필에 한해: 같은 호스트를 쓰는 다른 활성 프로필이 없을 때,
   그 호스트의 시스템 자격 증명 도우미.

각 프로필은 `config.toml`의 `[[integrations]]` 항목입니다.
[설정 참조](config-reference.md#integrations-호스팅과-이슈-트래커)를 참고하세요.

## 보안 확인

저장된 토큰은 그것이 전송되는 주소만큼만 안전합니다. 그래서 GitPoulpi는 각 연동에 대해 무엇에 연결되었는지 기억하고, 그것이 바뀌면 멈춥니다.

- **주소.** 연동을 처음 사용할 때(또는 연결할 때) 그 주소가 `config.toml`이 아니라 사용자만 접근할 수 있는 비공개 데이터 폴더(`<data>/private/`)의 `trust.toml`에 기록됩니다. 설정을 편집해도 신뢰하는 범위를 넓힐 수 없습니다. 설정의 주소가 기록된 주소와 다르면 그곳으로는 아무것도 전송되지 않으며 토큰도 읽지 않습니다. 같은 주소의 표기(`HTTPS://Git.Example.com:443/`)는 같은 주소로 보며, 비슷하게 생긴 호스트는 다른 주소입니다.
- **계정.** 토큰을 연결한 뒤, 그리고 대략 하루에 한 번(그리고 `GITPOULPI_TOKEN_*`나 자격 증명 도우미에서 오는 토큰은 실행마다 한 번), GitPoulpi가 호스트에 토큰의 주인을 물어 로그인이 아닌 계정 **id**를 비교합니다. 이제 다른 사람의 것이 된 토큰은 차단됩니다.
- **인증서.** 인증서가 알려진 기관으로 이어지지 않는 서버(사설 CA, 자체 서명 서버)는 사용자가 수락한 정확한 키에 대해서만 수락되며, TLS 핸드셰이크는 서버가 그 키를 가지고 있음을 여전히 증명합니다. 엄격한 고정을 켜면 인증서가 공인 루트로 이어지더라도 키가 일치해야 합니다. 키를 교체할 때마다 다시 묻는 것이 그 대가이므로 선택 사항입니다. 공개 호스트(github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net` 등)는 인증서 기록을 두지 않습니다.
- **일반 http.** 루프백이 아닌 `http://` 주소는 해당 연동에 대해 허용하지 않는 한 거부됩니다.

모든 요청은 연동 자신의 주소(공개 호스트의 경우 `api.github.com` 같은 API 호스트도)로 제한됩니다. 그 밖의 곳으로 가는 요청은 전송되기 전에 거부되고, 다른 주소로의 리디렉션은 따라가지 않습니다.

**이 검사가 생기기 전에 연결한 연동**은 업그레이드 후 처음 사용할 때 당시의 설정을 기준으로 조용히 기록됩니다. 현재의 신뢰 수준이 유지되며 아무것도 묻지 않습니다.

### 명령줄에서

멈춘 명령은 종료 코드 2로 실패하며 무엇이 바뀌었는지 출력합니다([CLI 참조](cli-reference.md) 참고). `gitpoulpi hosting integrations trust <id>`는 신뢰하는 항목을 보여 줍니다. `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off`, `--allow-insecure-http on|off`는 각각 정확한 값을 지정하여 한 가지씩 수락합니다. 새 주소를 수락한 뒤에는 토큰을 다시 입력하세요. 연결 해제(`hosting logout`)나 연동 제거는 토큰, 신뢰 기록, 그 연동에서 캐시한 것(클론 대화상자의 저장된 저장소 목록, Radar 계정 목록, 해당 호스트의 원격 리뷰 초안)을 삭제합니다.

에이전트(MCP)는 아무것도 수락할 수 없습니다. 보안 확인에는 사용자가 필요하다는 안내를 받습니다.

### 터미널 UI에서

멈춘 요청은 하던 작업 위에 **연동 보안 확인** 대화상자를 엽니다. 무엇이 바뀌었는지(이전 주소와 새 주소, 이전 계정과 새 계정과 그 id, 또는 이전 인증서와 새 인증서를 4자리씩 묶은 SHA-256 지문, 주체, 발급자, 유효 기간이 담긴 작은 표로)와 토큰을 보내지 않았다는 것을 알려 줍니다. 답은 번호가 붙어 있습니다.

1. **연결 해제**는 저장된 토큰, 신뢰 기록, 연동에서 캐시한 것을 삭제합니다.
2. 새 주소, 계정 또는 인증서를 **신뢰**합니다(암호화되지 않은 주소는 **일반 http 허용**).
3. **취소**는 아무것도 바꾸지 않습니다.

화살표 키나 숫자를 누르기 전에는 어떤 답도 강조되지 않으므로 실수로 누른 Enter가 아무것도 수락하지 않습니다. Esc와 닫기 표시는 취소하며, 대화상자 바깥을 클릭해도 아무 일도 일어나지 않습니다. 긴 본문은 휠, Page Up/Page Down 또는 스크롤바로 스크롤합니다. 여러 확인은 대기열에 들어가(연동과 변경 종류마다 하나) 차례로 묻습니다. 취소한 확인은 바뀌지 않는 동안 다시 뜨지 않으며, 명령 팔레트의 **연동: 보안 확인 검토**가 다시 불러옵니다.

새 **주소**를 신뢰하면 연동 양식이 토큰 필드에 포커스를 둔 채 "토큰을 다시 입력하는 것을 권장합니다"라는 힌트와 함께 열립니다. 저장된 토큰은 그곳으로 전송된 적이 없습니다. 양식에서 의도적으로 바꾼 주소는 그에 맞게 입력한 토큰과 함께 수락되며, 토큰이 없으면 양식이 토큰을 요구합니다.

양식에는 스위치도 두 개 있습니다. **엄격한 인증서 고정**(https 주소용)은 연결할 때 서버가 제시하는 인증서 키를 고정합니다. **일반 http 허용 (암호화 안 됨)**은 이 컴퓨터가 아닌 `http://` 주소에만 제공되며, 토큰이 평문으로 전송된다는 빨간 경고가 함께 표시됩니다. 연결된 카드에는 고정된 지문이 표시됩니다.

### 토큰이 할 수 있는 일과 만료 시점

연결하면 호스트에 토큰이 무엇을 할 수 있는지도 묻습니다. GitHub(클래식 토큰), GitLab, Bitbucket(스코프를 보내는 경우)은 스코프를 알려 줍니다. 카드에는 GitPoulpi가 사용하는 것 이상의 스코프를 나열한 노란색 "이 토큰에는 GitPoulpi에 필요한 것보다 많은 권한이 있습니다" 줄이 표시되며, 민감한 스코프(`delete_repo`, `admin:org`, `sudo` 등)는 빨간색으로, 만료일이 있는 세분화된 토큰을 쓰라는 권고가 함께 표시됩니다. 클래식 GitHub 토큰(`ghp_...`)은 스코프가 정확하더라도 그 권고를 받습니다. 세분화된 GitHub 토큰은 권한을 물어볼 수 없으므로 만료일만 표시됩니다. 호스트가 알려 주면 카드에 만료일이 표시됩니다. 만료일이 없는 토큰에는 알림이 없으며, 다른 제품은 스코프를 알려 주지 않고 경고도 띄우지 않습니다. 같은 필드가 `hosting integrations login`과 `test`의 JSON에 있습니다(`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). 연결할 때 읽으므로 카드는 연결한 세션에서만 이를 표시합니다.

## 4. 저장소 연결하기

연결은 명시적이며 저장소별입니다. 프로필이 연결되어 있으면 Refs 사이드바에
TAGS 다음으로 **PULL REQUESTS** 섹션과 **이슈** 섹션이 표시됩니다. 각 섹션에는
드롭다운이 둘 있습니다. 제공자와 프로젝트입니다.

- 제공자 드롭다운은 저장소의 리모트(`origin` 우선)와 일치하는 계정을
  `owner/name · origin · GitHub` 같은 행으로 **제안**합니다. 하나를 고르면 제공자와
  프로젝트가 함께 설정됩니다.
- 자동으로 연결되는 것은 없습니다. Jira, Trello, Linear, YouTrack, Redmine은 이름이 붙은 리모트가 없으므로
  제안되지 않습니다. 직접 선택하세요.
- 드롭다운 맨 위의 **없음 (이 저장소 연결 해제)**는 해당 섹션의 연결을 끊습니다.
  이후 저절로 다시 연결되지 않습니다.
- 연결 정보는 저장소의 `.gitpoulpi/repo-state.toml` 파일에 저장됩니다.

두 섹션은 **따로** 연결됩니다. 예를 들어 pull request는 Azure DevOps에서,
이슈는 Jira에서 가져올 수 있습니다.

| 제공자 | PULL REQUESTS 프로젝트 | 이슈 프로젝트 |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | 같음 |
| Azure DevOps | `Project/Repo` (조직은 프로필에서 가져옴) | `Project` (작업 항목) |
| Gitea, Forgejo | `owner/name` | 같음 |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | 없음: Jira를 연결 |
| Jira | 없음 | 프로젝트 키. 예: `ABC` |
| Trello | 없음 | 보드 |
| Linear | 없음 | 팀 키. 예: `ENG` (팀이 나열됨) |
| YouTrack | 없음 | 프로젝트 약칭. 예: `ABC` |
| Gerrit | Gerrit 프로젝트 이름. 예: `team/tool` | 없음 |
| Redmine | 없음 | 프로젝트 식별자. 예: `bench` (프로젝트 주소의 마지막 부분) |

이슈 프로젝트 목록은 검색할 수 있습니다. 목록에 없는 프로젝트를 직접 입력할 수도
있습니다.

**추가 쿼리.** 이슈 소스에는 프로젝트 드롭다운 아래에 선택 사항인 필터 행이
있습니다. 이 필터는 프로젝트와 AND로 결합됩니다.

- Jira: **JQL** 조각(`component = UI AND labels = backend`).
- Azure DevOps: **WIQL** 조건(`[System.AssignedTo] = @Me`).
- YouTrack: **YouTrack 쿼리**(`#Unresolved Type: Bug`, `tag: ui or tag: ux`). 괄호로 감싸지므로
  `or`가 프로젝트를 넓힐 수 없습니다. 직접 쓴 `sort by:`는 기본 `sort by: updated desc`를 대체합니다.
- Redmine: 없음(섹션의 상태와 "나에게 할당됨" 필터로 충분합니다).
- Linear: 없음(섹션의 상태와 "나에게 할당됨" 필터로 충분합니다).
- Trello: 표시할 **리스트**를 체크하는 드롭다운. 아무것도 체크하지 않으면 열려 있는
  모든 리스트를 표시합니다.
- GitHub와 GitLab은 이 필터를 무시합니다.

`Enter`는 입력한 쿼리를 유지하고, `Esc`는 버립니다. 팔레트:
*이슈: 쿼리 / 목록 편집...*.

CLI에서는 `gitpoulpi hosting link pulls --id <profile>`(또는 `issues`), 연결 해제는
`--clear`, 추가 쿼리는 `--filter`를 사용합니다.
[CLI 참조](cli-reference.md#명령)를 참고하세요.

### 연결된 계정에서 클론하기

연결된 프로필(GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit)은 **클론** 대화상자의 소스이기도 합니다(Home 탭의 **클론** 버튼, 또는 그곳에서 `c`). 각 프로필은 **URL로 클론** 아래에 독립된 소스로 나열됩니다(`GitHub · name`, `GitLab · name`). GitHub와 GitLab은 절대 섞이지 않습니다. 하나를 고르면 그 저장소들이 소유자별로 묶여 불러와집니다. 먼저 내 계정, 그다음 조직이나 그룹마다 섹션 하나(GitLab 하위 그룹은 `quarry/tools`)이며, 각 섹션은 개수가 적힌 머리글 아래에 있습니다. 자물쇠는 비공개 저장소를 표시합니다.

![연결된 GitHub 계정의 클론 대화상자: 소유자별 섹션과 비공개 저장소의 자물쇠](../images/clone-hosted.png)

검색창은 모든 섹션에 걸쳐 걸러 냅니다(이름, 전체 경로, 설명). 일치하는 항목이 없는 섹션은 사라집니다. 행에서 `Enter`를 누르면 그 저장소의 HTTPS URL과 폴더 이름이 클론 입력란에 채워집니다.

![검색으로 목록을 조직 하나로 좁힌 클론 대화상자](../images/clone-hosted-search.png)

이전에 연 적이 있는 프로필의 목록은 저장된 사본으로 바로 표시되고, 그동안 호스트에 다시 묻습니다(레이블 옆의 *새로 고치는 중...*). 응답이 도착하면 새 저장소가 나타나고 사라진 저장소는 없어지며, 강조 표시된 저장소와 선택한 저장소는 그대로 유지됩니다. 새로 고침에 실패하면 저장된 목록이 메모와 함께 남고, 이유는 활동 로그에 있습니다.

호스트가 토큰을 거부하면 대화상자는 목록 대신 이유와 함께 *저장소 목록을 가져오지 못했습니다*를 표시합니다(저장된 사본은 삭제됩니다). 다른 소스는 계속 동작합니다.

## 5. 사이드바의 pull request와 이슈

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

### 행에서 사용하는 동작

- `Enter` 또는 더블 클릭은 행을 엽니다: pull request는 [리뷰](review.md)로(GitHub, GitLab, Gitea, Forgejo, Gerrit), 이슈와 Azure DevOps나 Bitbucket의 요청은 창 안의 [대화](#5c-대화)로, 그 밖에는 브라우저에서 엽니다. 행 메뉴의 **브라우저에서 열기**는 항상 브라우저를 엽니다.
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

## 5b. 이슈 보드

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

## 5c. 대화

**대화**는 이슈, 작업 항목, 카드, 변경 또는 pull / merge request 하나에 대해 오간 모든 것입니다: 설명, 댓글, 리뷰, 코드의 스레드, 이벤트(레이블, 할당, 상태 변경, push). GitPoulpi가 호스트에서 읽어 서식을 적용해 보여 주며, 창을 떠나지 않고 답할 수 있습니다.

### 여는 방법

| 위치 | 방법 |
|---|---|
| 이슈 행 | `Enter` 또는 더블 클릭. 행 메뉴의 **대화 열기**. 팔레트 *호스팅: 이슈 대화 열기...* (화면에 있는 이슈 목록). 행 메뉴의 **브라우저에서 열기**는 브라우저를 그대로 엽니다. |
| Azure DevOps나 Bitbucket의 PULL REQUESTS 행 | `Enter` 또는 **대화 열기**. GitHub, GitLab, Gitea, Forgejo, Gerrit에서는 행이 [리뷰](review.md)를 열며, 그 개요 페이지가 요청의 대화입니다. |
| [이슈 보드](#5b-이슈-보드)의 카드 | `Enter` 또는 더블 클릭. |
| [Radar](radar.md) | 이슈나 요청 행에서 `i`, 행 메뉴의 **대화 열기**, 또는 *Radar: 이슈 대화 열기*. |
| 다른 대화 안의 참조 | 클릭하거나(또는 `l`을 누르고 선택): 연결된 소스의 `#7`, `!12`, `ABC-12`는 앱 안에서 열리고 뒤로 스택에 쌓입니다. |
| 에이전트 | `gitpoulpi ui show issue <n> --view conversation` (`ui show pr`도 가능). [CLI 참조](cli-reference.md)를 참고하세요. |

**복제하지 않은 저장소.** 복제본이 없는 저장소의 Radar 행도 대화를 엽니다: 이 보기에는 연동 프로필과 프로젝트만 필요하므로 Radar가 자기 창 위에 띄우며, `Esc`로 Radar로 돌아갑니다. 복제가 필요한 것은 빠집니다: commit 참조는 Log 대신 호스트에서 열리고, *AI가 저장소를 읽도록 허용*은 이유와 함께 회색이 됩니다([AI 장](ai.md#호스트에서-쓰기-도움) 참고). 복제된 저장소의 요청에는 리뷰가 있고(`e`), 그 요청에 대해 Radar의 **대화 열기**는 여기서 읽을 대화가 없다고 알립니다. 복제된 저장소의 이슈는 그 저장소의 탭에서 대화를 엽니다.

읽을 대화가 없는 호스트의 이슈(Bitbucket과 Gerrit에는 이슈가 없음)는 이전처럼 브라우저가 열리며, `conversation` 기능이 없는 빌드는 어디서나 그렇게 합니다([커스텀 빌드](custom-builds.md)).

### 보기

전체 창 보기입니다: 헤더(상태, 연 사람과 시각, 레이블, 댓글 수, **이벤트 표시**, **뒤로**, **새로고침**, **브라우저에서 열기**), **게시** 줄, 타임라인, 그 아래의 댓글 상자.

- 타임라인은 오래된 것부터 최신 순이며 가장 최근 범위(최대 300개 항목)에서 시작합니다. 맨 위의 **이전 항목 불러오기**(`m`, 팔레트 *대화: 이전 항목 불러오기*)는 그 앞의 범위를 읽으며, 호스트가 알려 주면 남은 개수도 표시합니다.
- 댓글에는 작성자, 날짜, `수정됨` 칩, 봇에는 `봇` 칩, 비공개 메모에는 `내부용`이 표시되고, 선택한 항목이 허용하는 **답글**, **인용**, **편집**, **삭제**, **링크** 버튼이 나옵니다. 코드의 스레드는 `path:line`, `해결됨` 또는 `미해결`, `오래됨`, 그리고 그 댓글들을 보여 줍니다. 리뷰는 그 판정을 보여 줍니다.
- 이벤트는 흐리게 한 줄씩 표시됩니다(*carol이 레이블 lfs를 추가했습니다*). **이벤트 표시**(`v`)가 숨기며, 선택은 저장소별로 유지됩니다. 이벤트 기록이 없는 호스트(YouTrack, Azure DevOps 작업 항목, 로컬 노트)에는 이 확인란이 없습니다.
- 텍스트는 렌더링된 Markdown입니다: 제목, 중첩 목록과 작업 목록, 인용, GitHub 스타일 알림, 강조된 코드, 표, 링크, `[image: alt]`로 표시되는 이미지. 호스트 자체의 마크업으로 온 텍스트는 먼저 변환됩니다([마크업 변환](#마크업-변환) 참고). `mermaid` 코드 블록에는 다이어그램의 소스를 스크롤 가능한 뷰어에 보여 주는 **다이어그램 열기** 버튼이 있고(`Enter`는 복사), 팔레트에는 *대화: 항목의 다이어그램 열기*가 있습니다. `conversation`과 `mermaid`가 모두 있는 빌드가 필요합니다.
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
| *ci-bot(으)로 게시 (저장소 토큰 명령)* | 저장소의 [토큰 명령](#13d-github-app-봇으로-동작하기)이 호스팅 API에 쓰입니다. |
| *my-app[bot](으)로 게시 (GitHub App 봇)* | 설치 토큰. 이름은 **게시 이름**에서, 없으면 호스트에서 옵니다. |
| *GitHub App 봇으로 게시 (이름을 아직 알 수 없음: 설정에서 게시 이름을 지정하세요)* | 설정된 이름도, 알아낸 이름도 아직 없습니다. |
| *...(으)로 게시 (프로젝트 봇)* | GitLab 프로젝트 또는 그룹 액세스 토큰. |
| *kai(으)로 작성 (로컬 노트)* | Markdown 노트 폴더: 프로필의 `local.me`, 없으면 연결된 사용자. |
| *어떤 이름으로 게시할지 아직 알 수 없습니다. 설정에서 게시 이름을 지정하세요.* | 알아낼 수 있는 것이 없었습니다. |

**게시 이름**은 이름입니다: GitHub App이라면 `my-app[bot]`입니다. 모든 저장소에 적용하려면 **설정 > 연동 > 게시 이름**(`general.hosting_post_as`)에서, 한 저장소에만 적용하려면 **설정 > 저장소별 설정 > SSH 및 자격 증명 > 호스팅**에서 지정하며, 후자가 우선합니다. 비워 두면 GitPoulpi가 GitHub에 묻고(GraphQL `viewer { login }`. 설치 토큰이 답할 수도 있고 못 할 수도 있음), 그래도 모르면 처음 게시한 댓글의 작성자에서 이름을 알아내 저장소와 함께 보관합니다. 개인 토큰에는 필요하지 않습니다. [GitHub App 봇으로 동작하기](#13d-github-app-봇으로-동작하기)를 참고하세요.

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

## 6. pull request 만들기

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

## 7. 이슈 만들기

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

## 8. 호스트의 웹 페이지에서 만들기

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

## 9. 도구 모음 칩

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

## 10. 목록이 새로 고침되는 때

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

## 11. 용어

GitHub, Azure DevOps, Bitbucket은 "pull request"라고 하고 GitLab은 "merge
request"라고 합니다. *리뷰 요청 용어*(`ui.pr_terminology`, 설정 > 연동)는
모든 레이블, 메뉴, 팔레트 명령, 메시지에 쓸 단어를 정합니다.

- **자동 (호스트별)**: 저장소에 연결된 제공자를 따릅니다(아무것도 연결되지 않은
  동안은 "pull request").
- **Pull request** 또는 **Merge request**: 항상 그 단어를 씁니다.

변경은 즉시 적용됩니다. CLI와 MCP는 항상 "pull/merge request"라고 합니다. 이
페이지는 "pull request"라고 씁니다.

## 12. commit 메시지와 브랜치 이름의 참조

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

**대화에서.** 이슈나 요청의 텍스트는 그 호스트의 문법으로 읽으며, 이는 위의 표보다 넓습니다: `owner/repo#7`, `!7`, `GH-7`, `@login`, commit id와 [상호 참조](#상호-참조)에 나열된 다른 형식. 저장소에 연결된 소스가 먼저 일치하므로 GitHub 댓글 안의 Jira 키도 앱에서 그 이슈를 엽니다. `gitpoulpi hosting markdown --body TEXT --id I`는 앱이 텍스트에서 찾아내는 것을 보여 줍니다.

### 이슈 참조 삽입

commit 작성기에서 요약 오른쪽 끝의 `#▾` 버튼은 연결된 이슈의 필터 가능한 목록을
엽니다. 현재 브랜치 이름이 가리키는 이슈가 *(브랜치에서)* 표시와 함께 맨 위에
옵니다. `Enter` 또는 클릭하면 그 참조(`ABC-123`, `#12`, `AB#12`)가 커서 위치에
삽입됩니다. 키보드: 작성기에서 `Alt+R`. 팔레트: *Commit: 이슈 참조 삽입...*.
연결된 이슈 목록이 없으면 버튼이 회색으로 비활성화되고 *먼저 사이드바에서 이슈 목록을
연결하세요*라고 표시됩니다. GitPoulpi는 참조를 스스로 추가하지 않습니다.

## 13. SSH 키

### 프로필별 SSH 키

연결된 프로필의 카드 아래에는 **SSH 키** 섹션이 있습니다. 키가 없으면 다음 중에서
선택합니다.

- **기본 키**: ssh-agent 또는 `~/.ssh`. 바뀌는 것은 없습니다.
- **새 키 생성**: 선택 사항인 제목(기본값 `GitPoulpi <host> <date>`).
  **공개 키를 <host>에 추가**를 체크하면 호스트에 아직 없는 경우 GitPoulpi가
  업로드합니다. 키는 암호 문구 없이 `~/.ssh/gitpoulpi_<profile id>_ed25519`와
  `.pub`로 기록됩니다. `ssh-keygen`은 필요 없습니다. 기존 파일을 덮어쓰는 일은
  없습니다.
- **사용자 지정 키**: 경로를 입력하거나 개인 키 파일을 **찾아보기**로 고른 뒤
  **이 키 사용**을 누릅니다.

키가 연결되면 섹션에 `title (SHA256:...)`이 표시되며 다음이 함께 나옵니다.

- **제거**: 키를 분리하고, GitPoulpi가 추가한 것이면 호스트에서도 삭제합니다.
- **삭제**: 위와 같고 키 파일도 삭제합니다. 두 경로를 나열한 확인을 거칩니다.

### 저장소 하나를 위한 SSH 키

저장소 하나가 특정 키를 쓰도록 하려면 탭 메뉴(`Ctrl+T` 또는 탭 오른쪽 클릭)를 열어
**SSH 및 자격 증명...**을 선택하거나, 팔레트 명령 *저장소: SSH 및 자격 증명...*을
실행한 다음, SSH 아래에서 **키 파일**(또는 연동 프로필의 키를 쓰려면 **계정 키**)을
고르세요. 이 선택은 GitPoulpi의 비공개 폴더에 보관되며 저장소나 git 구성에는 아무것도
기록되지 않습니다. [설정](settings.md#저장소별-ssh-및-자격-증명)을 참고하세요.

예전의 "이 저장소의 SSH 키" 고정에서 남은 자체 `core.sshCommand`가 있는 저장소는
그 페이지에 읽기 전용으로 표시되며 **GitPoulpi로 옮기기**가 제공됩니다.

CLI와 MCP: `gitpoulpi repo-auth`(저장소별 선택), 그리고 `core.sshCommand` 고정
자체에는 `gitpoulpi ssh-command show | set --key <file> | clear`.

설정 > SSH 및 자격 증명의 전역 키(*개인 키 경로* 또는 *키 쌍 생성*)는 자체 선택이 없는
모든 저장소에 적용됩니다. 저장소의 키 파일이나 고정이 우선합니다.

## 13b. 기록 다시 쓰기를 위한 브랜치 보호

기록을 다시 쓸 때([워크벤치](history-rewrite.md)) GitPoulpi는 호스트에 어떤 브랜치를 보호하는지 물어, 그 브랜치를
기본적으로 다시 쓰기에서 제외합니다. **연결한** 저장소(사이드바의 pull request나 이슈 섹션, 또는 고정한 프로필)에만
묻고, 그 프로필의 토큰을 쓰며, 토큰이 읽지 못하면 그냥 "알 수 없음"이 되고 아무것도 실패하지 않습니다. 공급자별로
무엇을 답하는지와 토큰에 필요한 권한은 다음과 같습니다.

| 호스트 | 읽는 것 | 토큰에 필요한 것 |
|---|---|---|
| GitHub (와 Enterprise Server) | 브랜치의 `protected` 플래그와 활성 브랜치 룰셋(`~DEFAULT_BRANCH`, `~ALL`, `refs/heads/...` 패턴) | `repo`(공개 저장소는 `public_repo`)가 있는 클래식 토큰, 또는 Metadata(읽기)가 있는 세분화된 토큰. 토큰이 읽지 못하는 룰셋은 그렇다고 알려 주며 클래식 규칙은 그대로 적용됩니다 |
| GitLab (gitlab.com과 자체 관리형) | `protected_branches`(이름과 `*` 패턴) | `read_api`, 그리고 저장소 설정을 볼 수 있는 역할 |
| Azure DevOps (Services와 Server) | 저장소의 활성화된 차단 브랜치 정책(`Exact`, `Prefix`, `DefaultBranch` 범위) | Code(읽기). "force push" 권한은 보안 설정이라 읽지 않습니다 |
| Bitbucket Cloud | push를 막는 브랜치 제한(`push`, `force`, `delete`) | `repository:admin`. 브랜칭 모델의 제한은 해석하지 않습니다 |
| Bitbucket Data Center | force push를 막는 브랜치 권한(`fast-forward-only`, `pull-request-only`, `read-only`) | 저장소 관리자 |

보호 설정 페이지(그리고 기본 브랜치와 저장소 삭제 페이지)는 저장소의 리모트 URL로 만들어지며, GitHub, GitLab, Azure
DevOps, Bitbucket에서는 저장소가 연결되어 있지 않아도 만들어져, 호스트가 push를 거부할 때 세 가지 해결책과 함께
표시됩니다.

## 13c. Radar의 계정 전체 목록

Radar는 보통 클론한 저장소를 지켜봅니다. 프로필은 클론 여부와 관계없이 호스트 전체에서 나와 관련된 것을 나열할 수도 있으며, `[integrations.radar]` 테이블의 세 스위치로 제어합니다(모두 기본값 꺼짐. [설정 참조](config-reference.md#integrationsradar-계정-전체-목록) 참고).

- **리뷰 요청**: 내 리뷰를 요청한 열린 요청.
- **열린 요청**: 내 네임스페이스와 내가 속한 각 org 또는 그룹, 또는 지정한 네임스페이스의 열린 요청 전부. 호스트 전체는 절대 아닙니다.
  프로필의 **네임스페이스**(설정 > 연동 > Radar에서 한 줄에 하나, 필요하면 정규 표현식)는 세 목록 모두를 제한합니다. 한 줄은 저장소의 전체 경로와 같거나 `/`에서 끝나는 경로 접두사이면 일치하며(`ptm`은 `ptm/produits/x`를 유지하고 `ptmx/y`는 유지하지 않음), 호스트가 돌려준 행에 적용되고 호스트의 검색이 허용하는 곳에서는 서버 쪽에서도 적용됩니다(GitHub의 `user:` / `repo:` 한정자).
- **할당된 이슈**: 나에게 할당된 열린 이슈.

GitHub, GitLab(self-managed 포함), Gitea, Forgejo, Azure DevOps(Services와 Server), Bitbucket Data Center, Gerrit이 응답합니다. 각 호스트에 묻는 방식:

| 호스트 | 요청 | 이슈 |
|---|---|---|
| Azure DevOps | 조직 전체의 활성 pull request(내 리뷰를 기다리는 것은 `searchCriteria.reviewerId`. 이미 투표한 요청은 제외). 서버가 조직 수준 경로를 거부하면 처음 20개 프로젝트를 하나씩 묻습니다. `namespaces`는 프로젝트 이름 | 나에게 할당된 Done, Closed, Removed가 아닌 작업 항목 |
| Bitbucket Data Center | 내 대시보드: 리뷰 요청은 아직 승인하지 않은 `REVIEWER` 역할, 열린 요청은 `AUTHOR`와 `PARTICIPANT`. `namespaces`는 프로젝트 키 | 없음(Data Center에는 이슈 트래커가 없으며 메모가 그렇게 알려 줌) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`. 열린 요청은 네임스페이스(프로젝트 경로 접두사)마다 `is:open projects:<prefix>`, 지정하지 않으면 내가 소유하거나 리뷰한 변경. work-in-progress 변경은 초안 | 없음 |

Bitbucket Cloud에는 계정 전체 행이 없습니다. 저장소 전반의 요청을 나열하는 엔드포인트가 없고 저장소마다 묻기에는 호출이 너무 많기 때문입니다. 클론한 저장소는 각자의 목록을 유지하며 `gitpoulpi hosting radar --id <bitbucket>`도 그렇게 알려 줍니다. Gerrit이 반환 한도보다 많은 변경을 가지고 있으면 목록이 잘렸다는 메모가 붙습니다.

초안은 유지하도록 설정하지 않으면 제외됩니다. 각 목록에는 한도가 있어 종류당 300행이며 더 있었던 개수를 알리는 메모가 붙고, GitHub는 검색 API가 분당 30회만 허용하므로 한 번에 몇 번의 검색 호출만 합니다. 토큰이 읽을 수 없는 네임스페이스는 메모가 되고 나머지는 계속 로드되며, 거부된 토큰이나 속도 제한은 해당 프로필만 실패시킵니다. 응답은 5분간 재사용됩니다(설정 폴더의 `radar-account.json`).

설정 > 연동에서 연결된 코드 호스트 프로필에는 SSH 키 아래에 **Radar** 블록이 있습니다. 목록마다 확인란 하나(*내 리뷰를 기다리는 요청*, *열려 있는 모든 요청*, *초안 포함*, *나에게 할당된 이슈*, 마지막은 이슈가 없는 호스트에서는 없음), *네임스페이스* 필드(한 줄에 항목 하나. 이름은 해당 그룹 또는 소유자와 그 아래 모든 것과 일치하며, *정규 표현식*을 체크하면 한 줄에 정규 표현식 하나. 비어 있으면 내 것과 소속된 모든 것), 그리고 선택 항목 *클론하지 않은 저장소의 요청*(*묻기*, *클론하기*, *API로 리뷰*)이 있습니다. 모든 변경은 바꾸는 즉시 저장됩니다. 계정 전체 쿼리가 없는 호스트(Bitbucket Cloud)에는 대신 그렇게 알리는 한 줄이 표시됩니다. 명령 팔레트에서는 *설정: 프로필의 Radar 옵션...*으로 이 블록에 갈 수 있고, 설정 검색은 각 컨트롤을 라벨로 찾습니다.

명령줄에서: `gitpoulpi hosting integrations radar <id> --review-requests on`으로 목록을 켜고, `gitpoulpi hosting radar --json`으로 목록을 출력합니다(`--refresh`는 캐시를 건너뜀). AI 에이전트도 MCP로 같은 두 명령(`hosting_radar`, `hosting_integrations_radar`)을 사용할 수 있습니다.

## 13d. GitHub App 봇으로 동작하기

사람의 이름이 보이면 안 되는 프로젝트는 호스트에서 모든 것을 **GitHub App**으로 할 수 있습니다: 그러면 모든 댓글, 리뷰, 닫기나 다시 열기, 레이블 변경이 `<app-slug>[bot]`으로 표시됩니다. 한 번에 저장소 하나에 적용되며, 다른 저장소는 내 토큰을 그대로 씁니다.

1. 조직용 GitHub App을 만들고 **Issues**와 **Pull requests** 권한을 *Read & write*로 설정한 뒤(pull request를 나열하고 리뷰할 수 있도록 *Contents: read*도), 저장소에 설치합니다.
2. 표준 출력에 **설치 토큰**을 출력하는 작은 스크립트를 씁니다 (App의 개인 키로 수명이 짧은 JWT에 서명하고, 이를 한 시간 동안 유효한 토큰으로 교환). GitPoulpi는 키를 보지 않습니다.
3. **설정 > 저장소별 설정 > SSH 및 자격 증명**에서 HTTPS 아래의 **토큰 명령**을 고르고, 스크립트를 **명령**에, 호스트(`github.com`)를 **호스트**에 입력한 뒤 **호스팅 API에 사용 (이슈, pull requests)**를 체크합니다. 저장하고 명령을 확인합니다(항상 사람이 SHA-256과 함께 확인). 명령줄에서는: `gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`.

그 저장소에서 달라지는 것:

- 이슈와 pull request 호출(목록, 만들기, 댓글, 리뷰, merge, 닫기, 레이블)이 명령의 토큰을 쓰며, git 자체 로그인과 같은 30초 제한, 메모리 캐시, 가리기가 적용됩니다. 토큰이 만료되어 호스트가 401로 답하면 명령을 한 번 다시 실행합니다. 명령을 실행할 수 없으면 호출은 거부됩니다: 내 개인 토큰은 절대 대체 수단이 아닙니다.
- "나"가 없습니다: 설치 토큰은 자기가 누구인지 물을 수 없습니다. 섹션 헤더가 프로필 이름에 **(앱 봇)**을 붙이고, "나"를 기준으로 한 필터(*나에게 할당됨*, *내 것*, *내 리뷰를 기다리는 것*)는 일치하는 것이 없습니다. 오류로 표시되는 것은 없습니다.
- **게시 이름.** GitPoulpi가 게시하는 모든 메시지는 게시되는 이름과 함께 표시됩니다([대화](#게시-이름) 헤더 아래의 *my-app[bot](으)로 게시 (GitHub App 봇)*과 그 댓글 상자, CLI 응답의 `posted_as`). 개인 토큰이면 내 로그인(`GET /user`)입니다. App 토큰이면 **게시 이름**의 이름, 예를 들어 `my-app[bot]`입니다: 모든 저장소에 적용하려면 **설정 > 연동**(`general.hosting_post_as`, `gitpoulpi config set general.hosting_post_as my-app[bot]`도 가능)에서, 한 저장소에만 적용하려면 **설정 > 저장소별 설정 > SSH 및 자격 증명 > 호스팅**에서 지정하며 후자가 우선합니다. 비워 두면 GitPoulpi가 GitHub에 묻고(GraphQL `viewer { login }`), 아니면 처음 게시한 댓글의 작성자에서 이름을 알아내 저장소와 함께 보관합니다. 그 이름은 내 댓글을 알아보는 방법이기도 하므로, 편집과 삭제는 그 댓글에만 제공됩니다 (App의 댓글은 봇 계정에서 온 것이어야 함). 이름을 알기 전에는 이 설치가 게시한 댓글만 바꿀 수 있습니다.
- App에는 할당할 수 없습니다. 이 설정을 아래의 할당 차단과 함께 쓰세요.
- **호스팅 API에 사용**을 켜거나 끄려면 새 명령처럼 사람이 해야 합니다.

GitHub 프로필만 저장소의 토큰을 이렇게 쓰며, 다른 호스트는 프로필 자체의 토큰을 유지합니다. (GitLab 프로젝트 또는 그룹 액세스 토큰도 봇입니다: 프로필의 토큰이며 *프로젝트 봇*으로 표시되고 설정할 것이 없습니다.)

### 저장소에서 할당 차단하기

**설정 > 저장소별 설정 > SSH 및 자격 증명**에 **할당** 행이 있습니다(기본값 **허용**, 또는 **차단**. 바로 저장됨). 차단하면 누가 요청하든 그 저장소의 이슈와 pull request에 할당할 수 없습니다.

- 새 이슈와 새 pull request 폼에서 담당자 선택기가 빠지고(이유를 알리는 메모가 붙음), 팔레트의 *호스팅: 담당자 편집...*이 이유와 함께 회색이 되며, 행 메뉴는 열지 않고 그렇게 알립니다.
- 모든 클라이언트가 아무것도 보내기 전에 할당을 거부합니다. 이슈나 pull request를 만들면서 지정한 담당자도 포함입니다: CLI는 `{"refused": {"reason": "assignment_blocked", ...}}`와 종료 코드 2로 답하고, MCP의 에이전트도 같은 거부를 받습니다.
- 이미 있는 담당자는 계속 표시됩니다(읽기 전용).

`gitpoulpi hosting assignment`는 설정을 보여 주고, `--block`이 켜며 `--allow`가 다시 끕니다. 다시 허용하는 것은 사람만 할 수 있습니다(MCP 서버는 할 수 없음).

## 14. 명령줄에서

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

모든 명령은 [CLI 참조](cli-reference.md#명령)를 참고하세요. 토큰은 인자로
받지 않으며(`hosting login`은 OS 자격 증명 저장소가 보관하면 `stored: "os"`를, 저장소가 거부해 토큰이 이번 세션 동안만 메모리에 있으면 `stored: "session"`과 `store_error`를 출력합니다) MCP 서버는 `hosting login`을 제공하지 않으므로, 토큰이 에이전트를
거치는 일은 없습니다.

## 문제 해결

| 증상 | 해결 |
|---|---|
| *연결*이 인증 오류로 실패함 | 토큰이 잘못되었거나, 만료되었거나, 폐기되었습니다. 새로 만드세요. |
| 범위가 빨간색으로 나열됨 | 토큰에 그 범위가 없습니다. 연결은 되지만 해당 기능은 작동하지 않습니다. 호스트에서 범위를 추가한 뒤 **편집**을 눌러 새 토큰을 붙여넣으세요. |
| 사이드바에 PULL REQUESTS 섹션이 없음 | 연결된 활성 프로필이 없습니다. 하나를 연결한 뒤 섹션의 드롭다운에서 저장소를 연결하세요. |
| 목록에 *다시 연결* 행이 있음 | 토큰을 잃었거나(자격 증명 도우미 없이 재시작) 만료되었습니다. 행을 클릭해 프로필을 여세요. |
| 자체 호스팅 인스턴스가 실패함 | **호스트 도메인**에 경로가 없는지, 그리고 컴퓨터가 인스턴스의 인증서를 신뢰하는지 확인하세요. |

## 알려진 제한

- **fork의 pull request:** Azure DevOps와 Bitbucket은 fork에서 온 요청을 체크아웃할
  수 없습니다([행에서 사용하는 동작](#행에서-사용하는-동작) 참고). Azure DevOps는
  fork에서 pull request를 만들 수 없습니다.
- **대화:** 가장 최근 300개 항목이 먼저 불러와집니다(나머지는 **이전 항목 불러오기**). GitPoulpi에서 게시한 댓글, 편집, 삭제는 외부로 나가는 동작이라 GitPoulpi에서 되돌릴 수 없습니다. 호스트별로 할 수 없는 것은 [표](#호스트별로-할-수-없는-것)에 있습니다. Gerrit에서는 메시지나 댓글을 편집하거나 삭제할 수 없고 Gerrit 텍스트는 일반 텍스트입니다.
- **Trello**는 보드의 처음 1000장까지만 표시합니다. 리스트 필터로 범위를
  좁히세요. **Jira**, **Linear**, **YouTrack**은 새로 고침마다 최대 500개의 이슈를
  나열합니다. JQL 조각이나 YouTrack 쿼리로 범위를 좁히세요.
- **Linear**는 이슈당 담당자 한 명을 받습니다: 선택기에서 여러 명을 고르면 첫 번째만 전송됩니다.
  그 "마일스톤"은 팀의 **사이클**입니다(프로젝트 마일스톤은 쓰지 않음). **YouTrack**의 것은
  `Fix versions` 필드이며, 버전에는 번호가 없으므로 선택기는 릴리스되지 않은 것을 프로젝트 안의
  위치로 번호를 매깁니다. 두 트래커 모두 pull request가 없습니다.
- **Redmine**은 이슈당 담당자 한 명(선택기의 첫 번째)을 받고, 레이블이 없으며, 새로 고침마다 최대
  500개의 이슈를 나열합니다. 담당자는 프로젝트의 구성원(사람과 그룹)이며 표시 이름으로 구분됩니다.
  칸반 보드의 열은 이슈 상태이고 카드를 옮기면 상태가 바뀝니다. Redmine의 기본 워크플로 규칙이 적용됩니다
  (그 전환을 할 수 없는 역할은 거부를 받고 카드는 제자리로 돌아갑니다). "Resolved"는 Redmine 기본 데이터에서
  닫힘 상태가 아니며, 닫힘 열은 "Closed"와 "Rejected" 상태입니다.
- **Bitbucket Data Center**에는 새 pull request, 브랜치, commit에 대한 브라우저 링크가
  없습니다. pull request 페이지는 정상적으로 열립니다.
- **Azure DevOps**는 작성자 아바타를 표시하지 않습니다. "내 것"과 "리뷰 요청됨" 필터는
  개인 액세스 토큰을 거부할 수 있는 엔드포인트가 필요합니다. 이 경우 프로필은 사용자
  이름 없이 연결되며 이 두 필터는 사용할 수 없습니다. 잘못된 토큰은 오류 대신
  로그인 페이지를 받으며, GitPoulpi는 이를 거부된 토큰으로 보고합니다.
- **Gitea와 Forgejo:** 초안은 제목이 `WIP:`로 시작하는 요청입니다(서버에는 생성 시 초안 플래그가
  없음). 요청은 이미 리뷰한 모든 사람을 리뷰어로 나열하며, 리뷰 요청을 철회해도 대기 중인 것만 제거됩니다.
  fork의 로컬 브랜치에서 온 pull request는 GitHub처럼 `refs/pull/<n>/head`로 체크아웃합니다. 언급은 서버의
  `mentioned_by` 필터로 검색합니다. 서버가 토큰 범위를 보여 주지 않으므로 빠진 범위는 호출이 거부될 때만
  발견됩니다.
- **이슈 보드**는 모든 이슈 소스에서 카드를 옮깁니다. Jira에서는 이동이 그 열로 이어지는 워크플로 전환을 실행하며,
  필드(예: 해결 상태)를 요구하는 전환은 Jira가 거부하고 카드는 그 메시지와 함께 돌아갑니다. Azure DevOps에서는 대상 state가 없는
  작업 항목 유형의 카드가 이유와 함께 돌아갑니다.
- **Bitbucket:** 앱 비밀번호는 더 이상 존재하지 않습니다. API 토큰을 사용하세요.
  저장소는 워크스페이스별로 나열됩니다.
- **Jira Cloud:** 기존 방식의 API 토큰만 작동합니다(범위 지정 토큰도, OAuth도
  없음). 새 Jira 이슈의 설명은 일반 텍스트입니다.
- pull request, 작업 항목, 이슈, 카드를 만드는 일은 호스트에서 이루어지며
  GitPoulpi에서 **실행 취소할 수 없습니다**. 요청 체크아웃은 정상적으로 실행 취소할 수
  있습니다.
