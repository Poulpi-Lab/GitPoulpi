[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · **한국어** · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# 로컬 이슈와 Obsidian

이슈를 Markdown 노트의 폴더로 보관합니다: 계정도, 토큰도, 네트워크도 필요 없습니다. 폴더는 저장소 안(코드와 함께 커밋되는
`docs/issues/`)에 있어도 되고 디스크의 어디에 있어도 되며, 예를 들어 [Obsidian](https://obsidian.md) 볼트의 하위 폴더일 수 있습니다.
GitPoulpi가 노트를 읽고 씁니다. Obsidian, 아무 편집기, 셸이 동시에 노트를 편집할 수 있으며 목록은 그 변경을 따라갑니다.

![사이드바의 로컬 Markdown 이슈와 노트 하나의 행 메뉴: 노트 열기, 닫기, 레이블 또는 마일스톤 편집](../images/local-issues.png)

## 설정

1. **설정 > 연동 > 이슈 트래커 > Local Markdown > 프로필 추가.** 폴더를 입력하거나 **찾아보기...**를 눌러 선택합니다. 상대 폴더(`docs/issues`)는
   연결된 각 저장소를 기준으로 읽고, 절대 폴더(`/home/me/Vault/Projects/Issues`)는 어디서나 같습니다. 입력할 토큰은 없습니다.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. 저장소에서 사이드바의 **이슈** 섹션을 열고 제공자 **Local Markdown - <이름>**을 고릅니다(폴더는 함께 선택되며, 저장소 드롭다운은 다른 폴더를 원할 때
   쓸 수 있도록 그 폴더와 바로 아래 하위 폴더를 나열합니다).
   - CLI: `gitpoulpi hosting link issues --id <profile>`(프로필의 폴더가 기본 `--project`입니다).

저절로 연결되는 것은 없습니다: 다른 모든 이슈 소스처럼 저장소별로 직접 선택해야 합니다.

## 노트

이슈마다 노트 하나이며, 이름은 `<id>-<slug>.md`(`12-crash-on-empty-name.md`)이고 frontmatter 블록이 있습니다:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- 새 이슈는 `max(id) + 1`을 받습니다.
- GitPoulpi는 자신이 소유한 속성(`title`, `status`, `assignees`, `labels`, `milestone`, `updated`)만 씁니다.
  **그 밖의 모든 속성, 본문, 줄 끝, 키의 순서는 바이트 단위로 그대로 유지됩니다.** 쓰기는 원자적입니다(임시 파일을 만든 뒤 이름 변경).
- 읽기는 너그럽습니다: `id`가 없는 노트는 파일 이름에서 가져오고, `status`가 없는 노트는 `todo`이며, frontmatter가 없는 노트는 첫 `# 제목`에서
  제목을 가져옵니다. 이슈가 아닌 파일(`readme.md`)은 무시됩니다.
- 상태는 자유 텍스트입니다. **열림**은 "완료 상태가 아님"을 뜻합니다.

## 할 수 있는 일

| 이슈 섹션에서 | 키보드 / 팔레트 |
|---|---|
| 새 이슈(`+` 또는 `n`): 제목, 설명, 상태, 담당자, 레이블, 마일스톤 | `호스팅: 새 이슈...` |
| 행 메뉴에서 담당자 / 레이블 / 마일스톤 편집. 폴더가 이미 쓰는 것에서 고르거나, **새 레이블이나 로그인을 입력**하고 `"..." 추가` 행을 선택 | `호스팅: 레이블 편집...` 등 |
| **이슈 닫기** / **이슈 다시 열기**(완료 상태, 또는 그 밖의 첫 번째 상태로 설정) | `로컬 이슈: 이슈 닫기...` / `이슈 다시 열기...` |
| **노트 열기**(`.md`의 기본 프로그램) | `연동: 이슈 열기...` |
| **Obsidian에서 열기**(폴더가 볼트 안에 있을 때만, 즉 위에 `.obsidian/` 폴더가 있을 때) | `로컬 이슈: Obsidian에서 열기...` |
| 필터: 열림 / 닫힘 / 전체, "나에게 할당됨" | 필터 드롭다운 |

"나에게 할당됨"은 프로필의 `me` 설정과 일치하며, 없으면 연결된 이름, 그것도 없으면 프로필 이름과 일치합니다.

브랜치와 커밋은 GitHub와 같게 동작합니다: "이슈에서 브랜치 만들기"는 `12-crash-on-empty-name`을 만들고, 커밋 메시지의 `#12` 또는 브랜치 이름의
`12-...`는 인식됩니다.

GitPoulpi 밖(Obsidian, 셸, `git pull`)에서 한 변경은 약 1초 안에 나타납니다: 폴더의 파일 이름과 수정 시각은 백그라운드 스레드에서 확인하며, 렌더링
스레드에서는 절대 확인하지 않습니다.

## 설정 파일

`config.toml`의 프로필 옆에 씁니다(모든 키는 선택 사항입니다):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # the board's column order
done_status = "done"                            # closes an issue
reference_prefix = "LOC"                         # shown as LOC-12 instead of #12
me = "kai"                                       # who "assigned to me" means
```

노트가 쓰지만 `statuses`에 없는 상태는 나열된 상태 뒤에 따라옵니다. `statuses`는 [보드](#칸반-보드)의 열 순서입니다
(`gitpoulpi hosting issue options`가 출력합니다).

## 칸반 보드

![노트 폴더의 보드: 상태마다 한 열, 담당자와 레이블, 마일스톤이 있는 카드](../images/issue-board.png)

혼자 일하거나 이슈를 코드 옆에 두는 팀이라면 폴더는 칸반 보드이기도 합니다. **이슈 머리글 > 보드** 또는 팔레트의 **이슈: 보드 열기**가
창 위에 보드를 엽니다. `Esc`로 닫습니다.

- **열**은 상태입니다: `statuses`에 나열된 것이 그 순서대로 먼저, 그다음 노트가 쓰는 다른 상태. 빈 열도 표시됩니다. 각 머리글에 카드 수가 붙습니다.
- **카드**에는 이슈 번호, 제목(두 줄), 담당자의 머리글자, 레이블, 마일스톤이 표시됩니다. 열이 모두 들어가지 않으면 보드는 옆으로 스크롤합니다
  (아래의 막대, `Shift`와 휠, 또는 화살표).
- **카드 이동**은 다른 열로 드래그하거나(포인터 아래에서 열이 밝아집니다), `Shift+Left` / `Shift+Right`, 또는 열의 드롭다운인
  **보드: 카드 이동...**(`m`)으로 합니다. 카드는 바로 움직이며, 노트의 `status`는 [노트 섹션](#노트)에서처럼 다시 쓰이고 파일의 다른 것은 바뀌지 않습니다.
  쓰기에 실패하면 카드는 제자리로 돌아가고 상태 표시줄에 오류가 표시됩니다.
- 카드를 **열려면** `Enter` 또는 더블 클릭합니다(노트가 `.md`의 기본 프로그램에서 열립니다). `e`는 담당자, 레이블, 마일스톤을 편집하고, `.` 또는
  오른쪽 클릭은 카드 메뉴를 열며, `n`은 선택한 열에 새 이슈를 시작합니다(양식이 닫히면 보드로 돌아옵니다).
- **필터**는 담당자(`a`), 레이블(`l`), 마일스톤(`M`), 텍스트(`/`)로 하며 `x`로 지웁니다. 필터는 드롭다운입니다: 입력하여 좁히고 `Enter`로 고릅니다.
- **실시간**: Obsidian이나 셸에서 바꾼 노트는 1초 안에 카드가 움직입니다.

모든 마우스 동작에는 키보드 키와 팔레트 명령(**보드: ...**, [키 바인딩](keybindings.md) 페이지에 나열)이 있습니다.

## Obsidian Bases

**팔레트: `로컬 이슈: Obsidian Base 만들기`**(CLI: `gitpoulpi hosting issue base`)는 노트 옆에 `issues.base`를 한 번 씁니다: *table* 보기와
`status`로 묶은 *kanban* 보기이며, `statuses`의 순서를 따릅니다. Obsidian에서 열면 같은 이슈를 그곳의 보드로 볼 수 있습니다. GitPoulpi는 기존 `.base`
파일(내 것이든 이전에 만든 것이든)을 절대 편집하지 않으며, 거부하고 그렇게 알립니다. 형식은 Obsidian의 Bases 구문
(<https://obsidian.md/help/bases/syntax>)을 따르며, kanban 보기에는 최신 Obsidian이 필요합니다.

## CLI와 MCP

다른 모든 이슈 소스와 같은 명령이 여기서도 동작합니다: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. 노트 전용 명령이 네 개 더 있습니다:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>`, `hosting issue base`.
[CLI 참조](cli-reference.md)를 참고하세요.

## 제한

- 새 **마일스톤**은 선택기에서 입력할 수 없습니다(폴더가 이미 쓰는 것을 고르세요). 하지만 노트의 `milestone:`은 무엇이든 될 수 있으며 저장하자마자
  나타납니다.
- 폴더 바로 아래의 노트만 이슈입니다(하위 폴더는 아님).
- 삭제 기능은 없습니다: 파일 관리자나 Obsidian에서 노트를 제거하세요(그 휴지통은 내 것입니다).
