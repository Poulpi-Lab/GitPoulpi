# ko terminology and style (crates/gitpoulpi-i18n/locales/ko.toml)

Sources: git's own Korean translation (git-l10n po/ko), Microsoft Korean terminology and style guide, usage in GitHub Desktop / VS Code / JetBrains / TortoiseGit / SourceTree / GitLab. Follow this table for any new key.

## Two vocabularies

Default ("git" variant): git words stay English, or the transliteration Korean developers say (스테이징). "Translate git terms" on ("local" variant): the Korean word. Every git word is a `{$term}` token, never inline text, so both modes (and the "git words in English" option) work. Where a concept has no token, write a pair `{ git = "..", local = ".." }` or, in a plain string, nothing: a pair whose two sides only differ by tokens is redundant, write one tokenised string.

| Concept | git variant | local variant | Notes |
|---|---|---|---|
| commit (n./v.) | commit | 커밋 | |
| push / pull / fetch | push / pull / fetch | 푸시 / 풀 / 페치 | transliterations; "가져오기" is not used (it means import) |
| force push | force push | 강제 푸시 | no token, pair |
| merge | merge | 병합 | |
| rebase | rebase | 리베이스 | |
| cherry-pick | cherry-pick | 체리픽 | |
| stash | stash | 스태시 | |
| stash pop / apply / drop | Pop / Apply / Drop | 꺼내기 / 적용 / 삭제 | pop = apply and remove; tokens `{$Pop}`, `{$Apply}`, `{$Stash_drop}`; autostash banner "Pop now" / 지금 꺼내기 |
| tag | tag | 태그 | |
| diff | diff | 차이 | `{$diff}` / `{$Diff}` ("Diff 보기" / "차이 보기") |
| stage / unstage / staged / unstaged | 스테이징 / 스테이징 해제 / 스테이징된 / 스테이징되지 않은 | same | transliteration accepted in git mode; the pane titles are 스테이징 안 된 파일 / 스테이징된 파일 |
| pull request / merge request | pull request / merge request | 풀 리퀘스트 / 머지 리퀘스트 | follows the host; abbreviation PR / MR; fixed (non-token) names use a pair |
| pick / reword / squash / fixup / drop / edit | same | 픽 / 메시지 수정 / 스쿼시 / 픽스업 / 삭제 / 편집 | rebase todo actions |
| checkout, revert, reset | 체크아웃, 되돌리기, 리셋 | same | no token, never English inline (`Reset --hard`, `soft/mixed/hard` stay as command names) |
| worktree | worktree | 워크트리 | pair; "작업 트리" is the working tree (the files) |
| fast-forward, upstream | fast-forward, 업스트림 | same | fast-forward is kept English |
| conflict | 충돌 | same | never 콘플릭트 |
| blame | Blame | Blame | kept English (GitHub, VS Code ko) |
| hunk | hunk / Hunk | same | kept English (as GitHub does) |
| clone | 클론 | same | never 복제 |
| HEAD, SHA, LFS, ours/theirs/base | unchanged | unchanged | Base also stays in the merge editor tabs |
| repository, branch, remote, ref | 저장소, 브랜치, 리모트, ref | same | |
| settings / palette | 설정 / 명령 팔레트 | same | English says "Settings" everywhere; never 환경설정 |
| issue, avatar, minimap | 이슈, 아바타, 미니맵 | same | |
| Log (the commit graph pane) | Log | same | the application log is 활동 로그 / 애플리케이션 로그 |
| merge-editor panes | 현재 (A), 수신 (B), Base, 출력 | same | not Output / Incoming in prose |
| composer fields | 요약, 설명 | same | not Summary / Description |
| diff layout | 인라인, 분할 | same | not Inline / Split in prose |
| undo / redo | 실행 취소 / 다시 실행 | same | never 실행취소 or a bare 다시 |
| Undo history / Restore only this / Lost work | 실행 취소 기록 / 이것만 복원 / 잃어버린 작업 | same | the undo safety net; Reflog stays `Reflog` (a git word, like Blame); an unreachable commit is 고아 in the list, `dangling` in prose |
| Palette twins of the three undo views | `실행 취소 기록:` / `Reflog:` / `잃어버린 작업:` | same | category + action; greyed-reason keys (`why_*`) use `~하세요`; Lost work refresh is `다시 검사` (scan again) |
| SHA | SHA | same | always uppercase, never "ID" or lowercase `sha` (the English source says SHA) |
| discard / delete / remove | 버리기 / 삭제 / 제거 | same | discard = throw away changes; remove = take out of a list |

## Style

- Address: no pronoun. Statements and status messages in 합쇼체 past (`삭제했습니다`), instructions `~하세요`, confirmations always `~하시겠습니까?` (never `~할까요?`). Buttons, menu rows and palette entries are noun or `~하기` phrases (`브랜치 만들기`), no sentence ending.
- Particles (josa) after a `{$term}` follow the pronunciation of both modes: vowel-final stash/push/fetch/rebase/tag/diff/pr/unstage/squash take 를 가 는 와 로; consonant-final commit/cherry-pick/stage/staged take 을 이 은 과 으로; pull is ㄹ-final (을 이 은 과 로). merge, drop, edit and the PR/MR abbreviations are ambiguous between modes: write the dual form `을(를)` `이(가)` or restructure (`{$Merge} 작업이`).
- Confirmations are `~하시겠습니까?` everywhere, also in the undo history dialogs (never `~할까요?`).
- Punctuation: ASCII `: , . ? !`, a space after `:`; palette entries keep the English shape `카테고리: 동작` (Git, Refs, 탐색, 작업 변경, 기록, 보기...). `...` exactly as in the English source: three ASCII dots in the catalog and the guide, never the single `…` glyph. A menu row or button that opens a dialog ends in `...` (`삭제...`, `파일 삭제...`). Quotes are straight `"..."` (never ‘ ’ or « »), the apostrophe is the ASCII `'` the catalog test requires. No full-width punctuation.
- Spelling: `미리 보기`, `새로 고침`, `줄바꿈`, `디렉터리` (Microsoft); `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Space`, `Up/Down` stay Latin in hints.
- Time: `2026년 10월 12일`, `10월`, `월요일`, relative `5분 전` / `3개월 전`.
- Width: CJK is 2 cells; buttons and tabs at most 4 Hangul (`실행 취소` and `다시 실행` are 9 cells, the toolbar limit).
- Lint limits: placeholders named `{commit}`, `{tag}`, refs like `stash@{id}`, code identifiers (`git.pull_mode`, `general.auto_stash`, `diff.tool`, `{{merged}}`) and AI prompt chips are flagged by the local-mode lint and stay in the baseline `tests/git_vocabulary/ko.toml`; the self-referential `settings.translate_git_terms_help` does too.

## User guide (docs/user/ko/)

Prose register: statements in 합쇼체 (`~합니다`), instructions `~하십시오` (formal, one register for the whole guide; the catalog keeps `~하세요` for UI hints). No pronoun for "you" (never 여러분); `사용자의` only where the owner of a thing must be named.

| Concept | Word | Notes |
|---|---|---|
| history (file / graph) | 기록 | `파일 기록`, `기록: ...` palette group; never 히스토리 |
| toolbar / tab bar / footer / status line | 도구 모음 / 탭 표시줄 / 푸터 / 상태 줄 | the footer is the bottom edge, the status line is its message; never 툴바, 탭 줄 |
| form / dialog / checkbox / label | 양식 / 대화상자 / 체크박스 / 레이블 | not 폼, 대화 상자, 확인란, 라벨 |
| scrollbar thumb | 썸 | not 손잡이 |
| highlight (selection) | 강조 표시 | not 하이라이트 |
| hunk | hunk | not 헝크 |
| crate | 크레이트 | as in the About window |
| reference pages | 설정 참조, CLI 참조 | not 레퍼런스; config.toml page is `설정 참조` |
| Settings groups | 설정 / 저장소별 설정 | never 환경설정 |
| spelling | 미리 보기, 새로 고침, 줄바꿈 | |
| CLI flags / env | `--logo-colors`, `--icon-color`, `GITPOULPI_LOGO_COLORS`, `GITPOULPI_ICON_COLOR` | American spelling, as the English guide |
| `pull request` plural | `{$prs}` labels keep the plural (`pull requests 및 이슈 새로 고침(분)`); palette group is `Pull request:`; `내 pull request` | |

The keymap and palette tables of `keybindings.md` are generated from this catalog (temporary local patch of the two drift tests: `label.get_in(Lang::Ko)` and `with_lang(Lang::Ko, ..)`, header row kept translated); labels quoted in the prose are checked against `ko.toml` (git variant).

## Workspaces, search, Radar (added in the l10n3 review)

| Concept | Word | Notes |
|---|---|---|
| workspace | 워크스페이스 | one term everywhere (catalog and guide); never 작업 공간 |
| missing (repository whose folder is gone) | 누락 / 누락된 저장소 | not 없음; "missing on disk" stays 디스크에 없음 |
| clone (also "Clone missing") | 클론 | never 복제 |
| Fetch all / Pull all | git `{$Fetch} All` / local `모두 {$Fetch}` | menu rows, palette, hints are pairs; progress lines use `{name}의 모든 저장소 {$fetch} 중:` so no mixed word order |
| worktree in new strings | pair `worktree` / `워크트리` | no token exists; the Radar key hint is a pair too |
| snooze / pin | 미루기 / 고정 | views and headings use 미룬 항목; messages are impersonal (`미뤘습니다`, `고정했습니다`) |
| Radar | Radar | feature name, kept |
| tab bar | 탭 표시줄 | help strings too, never 탭 막대 |
| status messages | 합쇼체 past, impersonal | `만들었습니다`, `전환했습니다`; confirmations `~하시겠습니까?` |

## Review, metadata, board, local issues (round 4, 2026-10)

| Concept | Decision | Notes |
|---|---|---|
| review (the whole), code review | 리뷰, 코드 리뷰 (as in the Radar's "내 리뷰 대기 중") | palette category `리뷰:`; verbs `리뷰 완료` (Finish review), `리뷰 제출` (Submit review), `리뷰 폐기...` (Discard review) |
| request (pull / merge request / Gerrit change, host not the point) | 요청 | `{$pr}` where a list or dialog is host-specific; Gerrit's own noun 변경 only in Gerrit text |
| thread | 스레드 | never 대화 (that is the Overview's conversation, `대화`, for the comments that are not on code) |
| comment (n.) | 댓글 | GitHub Korean; the verdict "Comment" is `댓글` too |
| resolve / reopen (a thread) | 해결 / 다시 열기; badges `해결됨`, `오래됨` | |
| pending comment / pending review | 대기 중인 댓글 / 대기 중인 리뷰; counter `대기 중인 댓글 {n}개`; badge `대기 중 - 줄 12` | not 보류 or 임시 저장; 미루기 stays the Radar's snooze |
| verdict, Approve, Request changes | 판정: `승인`, `변경 요청`, `댓글`; badges `승인함`, `변경 요청함`, `댓글 남김`, `기각됨` | GitHub Korean wording |
| suggestion (AI) | 제안: `수락` / `편집` / `무시`, `모두 수락` / `모두 무시`, `AI 요약` | severities `정보`, `경고`, `문제` |
| patch set, vote (Gerrit) | 패치 세트, 투표 (labels `Code-Review`, `Verified` stay Latin) | `리뷰용 push` for push for review (`refs/for/` stays Latin); `abandon` stays English |
| checks, conflicts | `검사 통과` / `검사 실패` / `검사 실행 중`, `충돌`, `충돌 없음` | GitHub Korean says 검사 for checks |
| assignee / reviewer / label / milestone | 담당자 / 리뷰어 / 레이블 / 마일스톤 | 레이블 as in the user guide; buttons use the noun alone, palette `담당자 편집...`, `마일스톤 설정...` |
| encoding | 인코딩; `자동 감지` for Auto-detect, `자동` on the bar | encoding names (UTF-8, Windows-1252...) stay Latin |
| board / column / card | 보드 / 열 / 카드; `카드 이동`, `열 레이블` | Open / Closed columns `열림` / `닫힘`, as the state filters |
| local issues | 로컬 이슈; 노트 폴더, 노트, Obsidian 볼트 | `Obsidian Base`, `Local Markdown` stay Latin; the sidebar section is `이슈`, never ISSUES, in catalog text |
| AI instructions of a workspace | `AI 지침`; `결합 방식`: `전역에 추가` / `전역 대체`; `{n}개 세트` | one noun 지침 for instructions, as the Settings page |
| fetching in new strings | `{$fetch}하는 중`, `{$fetch}하지 않았습니다` | token plus a native verb ending so both vocabularies read well; never 가져오기 (import) |
| status lines of the new views | 합쇼체 past, impersonal, no final period (`삭제했습니다`); greyed-reason `why_*` keys are fragments without a verb ending or use `~하세요` | notes ending in a sentence keep their period |
| supported-host lists | GitHub, GitLab, Gitea, Forgejo (then Gerrit) | |
| width | the diff bar's `인코딩` (6 cells) and `자동` fit the 4-Hangul button budget | `WS` stays Latin (two letters) |

## Branching workflows, Mermaid, Gerrit actions (round 5, 2026-10)

| Concept | Decision | Notes |
|---|---|---|
| workflow | 워크플로 | not 워크플로우; the toolbar button, menu and palette category stay `Flow` (feature name, 4 cells) |
| base branch / topic branch / topic type | 베이스 브랜치 / 토픽 브랜치 / 토픽 유형 | badge `[베이스]`, `[베이스, 보호됨]`; sidebar folder for unclaimed branches `기타` |
| parent / name pattern / environment branch | 부모 / 이름 패턴 / 환경 브랜치 | `부모 설정` for Set parent; stacked branches `그 위에 쌓인 브랜치` |
| start / finish / sync / promote | 시작 / 완료 / 동기화 / 승격 | `완료` is also Finish review; dialog buttons are the noun alone, `부모 설정` is 4 Hangul + space |
| hook | 훅 | `시작 후`, `완료 전`; warning says the hooks run shell commands |
| pick up (take over a host-made branch) | 이어받기 (`이어받을 대상` for the field) | never 가져오기, which means import: the Settings `Import it` button is `가져오기` on purpose |
| host-driven issue flow | 이슈 흐름 (호스트 주도) | "Branch made by" is `브랜치 생성 주체`; modes `사용자`, `호스트`, `봇` |
| Mermaid gitGraph popup | `Mermaid gitGraph로 복사`; Detail `상세도` (rows `구조`, `Refs`, `제목`, `전체`), Direction `방향`, `미리 보기`, `복사` | the English rows Structure/Refs/Subjects/Full; CLI values (structure, refs, subjects, full) stay Latin; guide heading `commit을 Mermaid gitGraph로 복사하기` (the guide writes commit in English) |
| Gerrit Submit / Abandon (change) | `Submit` / `Abandon` kept English (buttons, prompts, palette `리뷰: 변경 Submit...`) | Gerrit's own action names; `제출` would clash with `리뷰 제출`; sentences take them as a verb stem (`Submit하시겠습니까?`) |
| compare patch sets | `비교 대상`, `비교: 베이스`, `비교: 패치 세트 {n}` | status line `패치 세트 {n}부터의 차이를 읽는 중...` avoids a particle after a number |
| Detect / Use detected | `브랜치에서 감지` / `감지한 워크플로 사용` | scores `점수 {score}/100` |
| Save to shared / Save for me only | `.gitpoulpi/workflow.toml에 저장 (공유)` / `나에게만 저장` | |
| particles after placeholders | restructure instead of guessing | `부모 설정: {branch}`, `이슈를 만들었습니다: {reference}`, `{to} (출처: {from})`, `{parent}보다 ...` |
