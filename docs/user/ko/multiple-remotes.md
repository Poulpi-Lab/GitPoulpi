[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · **한국어** · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# 한 브랜치에 여러 리모트

많은 저장소가 둘 이상의 호스트에 있습니다: 메인 호스트(`origin`), 다른 호스트의 미러, 백업 등입니다. GitPoulpi에서는 브랜치를 클릭 한 번으로 이 모두에 push할 수 있고, 원하면 둘 이상의 리모트에서 pull할 수도 있습니다. 순정 git의 동작은 바뀌지 않습니다. 계정이 필요 없으며 모든 것이 저장소 자체의 `.git/config`에 저장되므로 순정 git과 다른 도구도 그대로 동작합니다.

## push 대상과 pull 소스

- **업스트림**은 git 자체의 개념입니다(`branch.<name>.remote`와 `merge`). 브랜치의 *기본* 리모트이며 GitPoulpi는 그 동작을 절대 바꾸지 않습니다.
- **push 대상**은 브랜치가 함께 push하는 추가 리모트입니다. Push 버튼은 브랜치를 업스트림과 모든 push 대상으로 보냅니다.
- **pull 소스**는 pull이 함께 커밋을 가져오는 추가 리모트입니다. 켜지 않으면 꺼져 있습니다(**함께 Pull** 체크박스): 직접 고르지 않은 두 리모트를 병합하는 것은 누구도 기본값으로 원하지 않기 때문입니다.

대상은 해당 리모트에서 다른 브랜치 이름을 쓸 수 있습니다(`backup`에는 `feat-x`).

## 리모트 대화상자

브랜치를 오른쪽 클릭하고 **<branch>의 리모트...**를 고르세요(Push 드롭다운, 팔레트의 `Git: 브랜치의 리모트...`, Refs 섹션에서 브랜치 행의 `⇉3` 배지를 클릭해도 열립니다). 저장소의 모든 리모트가 나열됩니다.

| 열 | 의미 |
|---|---|
| 리모트, 브랜치 | 리모트와 그곳의 브랜치 이름. **리모트 이름 변경...**은 선택한 행의 이름을 바꿉니다. |
| Push | 체크하면 그 리모트가 이 브랜치의 push 대상이 됩니다. |
| 함께 Pull | 체크하면 pull이 그 리모트도 통합합니다. 업스트림은 항상 pull됩니다. |
| 상태 | 브랜치가 그 리모트와 어떤 상태인지: 동기화됨, 앞섬, 뒤처짐, 새로 만듦, 무관함. |

"리모트 없음"으로 표시된 행은 브랜치가 아직 언급하지만 저장소에는 더 이상 없는 리모트입니다. **제거**를 누르면 잊습니다. 같은 대화상자에서 업스트림(**업스트림** 목록)도 설정하며 **업스트림 해제**로 지웁니다. **적용**은 모든 것을 되돌릴 수 있는 하나의 변경으로 씁니다.

저장소는 모든 새 브랜치에 같은 추가 대상을 줄 수도 있습니다: **설정 > Git 및 인코딩**(저장소별 페이지)의 "새 브랜치의 추가 push 대상"(리모트마다 체크박스 하나)입니다. 브랜치가 처음 업스트림을 갖게 될 때 브랜치에 복사되며, 이미 있는 브랜치는 건드리지 않습니다.

## Push

- 추가 대상이 있는 브랜치의 **Push 버튼**: 모두에 push합니다. 모든 리모트가 새 커밋을 그냥 받을 수 있으면 즉시 실행되고, 진행 보기에서 리모트마다 체크 표시로 바뀝니다. 결정이 필요한 것(리모트에 내가 없는 커밋이 있음)이 있으면 대신 계획 대화상자가 열립니다.
- **Push ▾**(버튼 옆의 화살표. 설정에서 **Push 버튼**이 **묻기**이면 버튼 자체도):
  - `모두 Push (origin, gitlab, backup)`
  - `<remote>에만 Push`, 브랜치의 리모트마다 하나씩
  - `선택한 리모트로 Push...`: 리모트가 하나뿐인 브랜치를 포함해 어떤 브랜치에서나 항상 있습니다
  - `<branch>의 리모트...`
- 브랜치의 오른쪽 클릭 메뉴와 팔레트(`Git: 모든 대상으로 Push`, `Git: 리모트로 Push...`, `Git: 선택한 리모트로 Push...`)도 같은 이름으로 같은 동작을 제공합니다.

**선택한 리모트로 Push...**는 저장소의 모든 리모트 목록을 열며, 브랜치의 업스트림과 대상이 체크되어 있습니다. 이번 push에만 적용됩니다. **이 브랜치에 기억**을 체크하면 그 선택이 브랜치의 push 대상으로 저장됩니다.

### 계획

그룹 push 전에 GitPoulpi는 모든 리모트를 살펴봅니다(빠른 조회이며, 해당 리모트와 다를 때만 내려받습니다). 그래서 세 번째 리모트의 문제가 첫 push가 시작되기 전에 드러납니다. 대화상자는 각 리모트와 브랜치의 상태, 그리고 수행할 동작을 나열합니다.

| 상황 | 기본 동작 | 그 밖의 선택지 |
|---|---|---|
| 리모트에 브랜치가 없거나 일부 커밋이 없음 | Push | 건너뛰기 |
| 리모트가 앞섬(리모트에 보낼 새 것이 없음) | 그 리모트에서 fast-forward | 건너뛰기 |
| 이미 push한 것을 rebase하거나 amend해서 리모트와 다름 | 강제 push (lease) | Merge, Rebase, 건너뛰기 |
| 리모트에 다른 사람의 커밋이 있음 | 그 리모트에서 Merge(**Pull 버튼**이 rebase이면 Rebase) | Rebase, 강제 push (lease), 건너뛰기 |
| 기록이 내 것과 공유하는 것이 없음 | 건너뛰기 | 강제 push (lease) |
| 호스트가 브랜치를 거부함(보호됨) | 건너뛰기 | 없음: 강제 push도 거부됩니다 |

각 행의 동작은 바꿀 수 있는 목록입니다(Left/Right로 넘기고, Enter로 열고, `s`로 행을 건너뜁니다). merge는 충돌이 날지, 어느 파일에서 나는지 보여 줍니다. **강제 push**는 항상 확인을 거치고, 리모트와 대체되는 커밋 수를 밝히며, 항상 lease를 쓰고(맨 force는 절대 아님), 되돌릴 수 없습니다: 결과에 어떤 커밋이 대체되었는지와 그것을 되돌리는 명령이 나옵니다. **문제 건너뛰기**는 문제가 있는 모든 행을 건너뛰기로 바꿉니다. **실행**(Ctrl+Enter)으로 시작합니다.

merge, fast-forward, rebase가 목록 순서대로 먼저 실행되어 브랜치가 최종 상태에 한 번에 도달하고, 그다음 push가 리모트를 하나씩, 업스트림부터 나갑니다. 리모트 전체에 걸친 원자적인 "전부 아니면 전무"는 없습니다. 각 리모트가 자기 결과로 끝나며, 한 곳의 실패(네트워크, 훅)가 다른 곳을 멈추지 않습니다. **실패 재시도**(또는 `Git: 실패한 Push 재시도`)로 그것들을 다시 시도합니다. 진행 보기를 닫아도 실행은 멈추지 않으며, `Git: 그룹 Push 보기`로 다시 불러옵니다.

### 충돌과 계속

실행 중의 merge나 rebase가 충돌로 멈추면 대화상자가 닫히고 일반 merge 배너와 충돌 편집기가 이어받습니다. Log 위의 띠에 `그룹 push 일시 중지 (gitlab): 해결하고 마무리한 뒤 계속하세요.`가 표시되며 **계속**과 **그룹 중단** 버튼이 있습니다.

- 충돌을 해결하고 merge나 rebase를 마무리하세요(GitPoulpi나 순정 git으로).
- **계속**은 그것이 끝날 때까지 회색이며, **대신 눌리지 않습니다**: 아무것도 저절로 계속되지 않습니다. 누르면(또는 `Git: 그룹 Push 계속`을 실행하면) 실행이 새 끝 커밋에서 모든 리모트로 다시 push합니다. rebase 후에도 옛 커밋을 가진 리모트는 몰래 강제로 덮어쓰지 않고(이제 강제 push가 필요하므로) 계획 대화상자로 돌아옵니다.
- **그룹 중단**(확인 후. `Git: 그룹 Push 중단`)은 실행이 시작한 merge나 rebase를 버리고 실행을 잊습니다. 이미 끝난 push는 그대로 남습니다.

띠는 GitPoulpi를 다시 시작해도 남으며, CLI도 같은 저장된 실행을 봅니다.

## Pull

- **Pull** 버튼은 브랜치에 pull 소스가 없으면 전과 똑같이 업스트림을 pull합니다. pull 소스가 있으면 업스트림과 모든 소스를 순서대로, 각각 설정의 **Pull 버튼**이 정한 모드(merge, fast-forward만 또는 rebase)로 pull합니다. fast-forward할 수 없는 소스는 업스트림처럼 merge(또는 rebase)합니다. fast-forward만이면 그 이상이 필요한 소스는 먼저 묻습니다.
- **Pull ▾**에는 소스가 있으면 `모든 소스에서 Pull (origin + gitlab)`이, 브랜치의 리모트마다 `<remote>에서만 Pull`이 추가됩니다: 체크박스와 상관없이 그 리모트의 브랜치를 한 번만 pull합니다.
- 팔레트에는 `Git: 모든 소스에서 Pull`과 `Git: 리모트 하나에서 Pull...`이 있습니다.
- 충돌로 멈춘 pull도 같은 띠를 남기며, 그룹 pull로 표시됩니다.

## 설정

네 가지 모두 **설정 > Git 및 인코딩**에 있습니다.

| 설정 | 기본값 | 의미 |
|---|---|---|
| **Push 버튼** | 바로 | 바로는 즉시 push하고, 묻기는 먼저 Push 메뉴를 엽니다. |
| **Pull 버튼** | merge | Pull 버튼의 모드(merge, fast-forward만 또는 rebase)이며 pull 소스에도 적용됩니다. |
| **push 전에 모든 리모트 확인** | 켜짐 | 그룹 push 전에 모든 리모트를 살펴봅니다(CLI도 따르며, `--no-preflight`로 한 번만 끌 수 있습니다). |
| **새 브랜치의 추가 push 대상**(저장소별) | 없음 | 새 브랜치가 함께 push할 리모트. |

## 명령줄에서

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

리모트가 결정을 필요로 하면 명령은 계획과 함께 응답하고(종료 코드 3) 아무것도 바꾸지 않습니다. `--resolve gitlab=merge`로 답하세요. 모든 플래그는 [CLI 참조](cli-reference.md)를 참고하세요. MCP 서버에도 같은 도구가 있습니다.

## 순정 git으로

대상은 평범한 설정 키이므로 스크립트나 alias가 GitPoulpi 없이도 사용할 수 있습니다.

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

값은 `<remote>`(같은 브랜치 이름) 또는 `<remote>:<branch>`이며, `d=${t#*:}` 줄이 이를 가려냅니다. 순정 alias는 리모트를 먼저 살펴보지 않고, merge를 제안하지 않으며, 충돌 후에 계속할 수 없습니다. 그 일을 위해 대화상자가 있습니다.

## 알아 두면 좋은 점

- 같은 주소를 가리키는 두 리모트는 알아챕니다("gitlab과(와) mirror는 같은 URL로 push합니다"). 두 번째는 기본으로 건너뜁니다.
- 보호된 브랜치가 있는 호스트는 강제 push를 거부합니다. 행이 그렇게 알려 주고 건너뛰기만 제안합니다.
