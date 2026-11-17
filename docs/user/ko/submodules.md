[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · **한국어** · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# 서브모듈

서브모듈은 고정된 commit으로 저장소 안에 중첩된 또 다른 저장소입니다. GitPoulpi는 Refs 사이드바에 서브모듈을 나열하고, UI, CLI, 명령 팔레트에서 업데이트하며, 체크아웃한 commit에 맞춰 최신으로 유지할 수 있습니다.

## 서브모듈 섹션

저장소에 `.gitmodules`가 있으면 Refs 사이드바에 **서브모듈** 섹션이 생깁니다(다른 섹션 아래에 있으며, 서브모듈이 없으면 나타나지 않습니다). 서브모듈마다 한 행이며 상태 표시, 경로, 짧은 commit이 보입니다.

| 표시 | 상태 | 의미 |
|---|---|---|
| `✓` | 최신 | 저장소가 기록한 commit으로 체크아웃되어 있음 |
| `↑` | 수정됨 | 다른 commit으로 체크아웃되어 있음(앞서 나갔거나, 업데이트하지 않고 브랜치를 전환함) |
| `○` | 클론되지 않음 | `.gitmodules`에 선언되고 기록되어 있지만 폴더가 비어 있음: 업데이트하면 클론됨 |
| `✗` | 충돌 | 저장소에 해당 서브모듈에 대한 merge 충돌이 있음 |

![행이 세 개인 서브모듈 섹션: vendor/almanac은 클론되지 않음(○), vendor/ephemeris는 최신(✓), vendor/starfield는 수정됨(↑, 기록된 것과 다른 commit)이며 Status 창에도 변경된 파일로 표시됨](../images/submodules.png)

| 하려는 일 | 마우스 | 키보드 |
|---|---|---|
| 서브모듈을 자체 탭으로 열기 | 행 더블 클릭 | 선택 후 `Enter` (팔레트: `Refs: 서브모듈을 탭으로 열기`) |
| 메뉴: **업데이트 (필요 시 클론)**, **초기화**, **새 탭에서 열기** | 행 오른쪽 클릭 | 선택 후 컨텍스트 메뉴 키(기본 `.`) |
| 선택한 서브모듈 업데이트(필요하면 먼저 클론) | 메뉴 | `Ctrl+P`: `Refs: 서브모듈 업데이트 (선택)` |
| 클론하지 않고 `.git/config`에 등록 | 메뉴 | `Ctrl+P`: `Refs: 서브모듈 초기화 (선택)` |
| 모든 서브모듈 업데이트 | | `Ctrl+P`: `Git: 모든 서브모듈 업데이트` |
| 섹션으로 이동 | 헤더 클릭 | `Ctrl+P`: `탐색: Refs > 서브모듈로 포커스` |

![수정된 vendor/starfield 행의 오른쪽 클릭 메뉴: 업데이트 (필요 시 클론), 초기화, 새 탭에서 열기](../images/submodule-menu.png)

업데이트는 서브모듈의 리모트에서 가져올 수 있으므로, 다른 전송과 마찬가지로 **설정 > SSH 및 자격 증명 > HTTPS 자격 증명**을 따릅니다. 클론되지 않은 서브모듈은 탭으로 열 것이 없습니다. 상태 표시줄이 그렇게 알려 주고 업데이트를 안내합니다.

## 서브모듈을 최신으로 유지하기

**설정 > 동기화 > 서브모듈 최신 상태 유지**(`general.keep_submodules_up_to_date`, 기본적으로 꺼짐). 켜면 **체크아웃**이나 **pull**이 성공한 뒤에 `git submodule update --init --recursive`가 실행됩니다. 이 명령은 저장소가 지금 기록한 내용과 서브모듈이 어긋나 있을 때만 실행되므로, 서브모듈이 없는 저장소는 느려지지 않습니다. 그 업데이트가 실패해도 체크아웃이나 pull은 이미 일어난 것이며, 오류 메시지가 그렇게 알려 줍니다.

## 명령줄에서

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

경로를 지정하지 않으면 모든 서브모듈이 대상입니다. 이 세 가지는 MCP 도구이기도 합니다(`submodule_list`는 읽기이고, `init`과 `update`는 변경 작업이며 `[mcp]`의 `allow_mutations`를 따릅니다).

## 참고

- 최상위 수준만 나열됩니다. 서브모듈 자체의 서브모듈은 탭으로 열 때 표시됩니다(또는 `--recursive`로 업데이트).
- 최신 git은 `protocol.file.allow`가 허용하지 않는 한 `file` 전송으로 *로컬 경로* 서브모듈을 클론하는 것을 거부합니다. 이는 URL이 로컬 폴더인 서브모듈에만 해당하며, 그런 서브모듈을 쓴다면 전역 git 설정에 지정하세요.
