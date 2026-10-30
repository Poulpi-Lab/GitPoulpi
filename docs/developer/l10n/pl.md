# Polish (pl) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/pl.toml` follow this page. Sources: git's own Polish translation (git-l10n `po/pl`), Microsoft terminology and style guide for Polish (also Azure DevOps / TFS and Visual Studio Polish UI), GitHub Desktop, VS Code, JetBrains and TortoiseGit in Polish, and what Polish developers say aloud (the English git word, inflected). Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Impersonal wherever possible; where the reader is addressed, 2nd person singular imperative ("Wybierz", "Sprawdź") and lower case possessives ("twoje zmiany", "swój klucz"). Never "Pan/Pani", never capitalised "Ty/Twój" (except at the start of a sentence). Never first person singular ("Pracuję", "pokazuję"): use the verbal noun or a passive ("Praca w toku", "pokazano ..."). |
| Buttons, menu rows, palette commands | Imperative, capitalised first word: "Utwórz gałąź...", "Otwórz w przeglądarce". Progress: verbal noun ("Wczytywanie...", "Stosowanie schowka..."). Results: past impersonal ("utworzono {path}", "skopiowano {what}"). |
| Palette and keymap | `Kategoria: Akcja`, colon directly after the category (no space before it), capitalised action. Categories are translated (Nawigacja, Plik, Widok, Repozytorium, Aplikacja, Zdalne) except interface part names (`Git`, `LFS`, `Refs`, `Log`, `Blame`, `Status`...). In local mode the categories `Commit`, `Diff`, `Merge` become `Zatwierdzenie`, `Różnice`, `Scalanie`. |
| Titles, tabs, columns | Sentence case ("Dziennik aktywności", "Szczegóły commita"), not English Title Case. Sidebar section headings stay upper case English names (`LOCAL`, `REMOTE`, `ISSUES`) except those that are `{$TERM}` tokens. |
| Reasons for a greyed row (`why_*`, `select_*`) | Lower case, imperative: "najpierw zaznacz gałąź w panelu Refs". |
| Quotes | Polish low-high „tekst” (U+201E / U+201D) for UI names and quoted values. The plain ASCII apostrophe `'` is required by the catalog test (also in "crate'y", "blame'ować"); never `’`. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Ellipsis | Three ASCII dots `...` everywhere English has them (dialog-opening commands, placeholders, in-flight messages); the real `…` is no longer used in the catalog, and the guide quotes follow. |
| Status-line and in-flight messages | As in English: a short lower case clause, no final period ("zapisano {path}", "nie można otworzyć {path}: {why}", "zapisywanie {path}..."); acronyms and proper nouns keep their capitals ("AI nie mogło ..."). |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Up/Down`, `PgUp/PgDn`, `Home/End` stay Latin (the chord printer shows them). Words without a printed form: "Spacja", "w górę/w dół" only in prose. |
| Mouse | "kliknięcie", "podwójne kliknięcie", "prawy przycisk myszy" in prose; "Shift+klik", "Ctrl+klik" as compounds; "dwuklik" only in the narrow key columns (`*_keys`, palette `shortcut_*`). |
| Dates and times | `{day} {month} {year}` with the month in the genitive from `[month_of]` ("5 marca 2024", no "r."), standalone months and weekdays lower case ("marzec 2024", "wtorek"), three-letter month abbreviations ("sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru"), "przed chwilą", "5 minut temu". |
| Plurals | CLDR pl: `one` (1), `few` (2-4, 22-24...), `many` (0, 5-21, 25-31...); `other` is for fractions only, but the catalog test wants it: write it as the genitive singular ("5 pliku"). Where English is a plain string with `{n}`, keep it grammatical for any n by rewording ("Cherry-pick {n} commitów na"). |
| Generic wording | "Ustawienia" (never "Preferencje"), "karta" (tab), "okno" (window/dialog), "panel" (pane), "widok" (view), "ścieżka" (path), "poświadczenia" (credentials; never "dane logowania"), "ustawienie wstępne" (preset), "baner", "paleta poleceń", "dostawca" (provider), "zgłoszenie" (issue), "etykieta" (label), "AI". "linia" is a line of file text, "wiersz" is a row of a list or table; end of line is "koniec linii". "Drzewo robocze" for working tree (never "katalog roboczy"), "katalog główny repozytorium" for the repository root. |

## Genders and declension (the ones that bite)

git vocabulary words are declined like Polish nouns: commit (m.) - commita, commitowi, commit, commitem, o commicie, commity, commitów; tag (m.) - tagu, tagi, tagów; stash (m.) - stasha, stashe, stashy; merge/rebase/cherry-pick/revert/reset/fetch/push/pull stay indeclinable verbs or nouns ("po merge", "przed push", "zrób fetch"); pull request - pull requestu, pull requesty, pull requestów. The local vocabulary is neuter for the commit ("zatwierdzenie", "zaznaczone zatwierdzenie", plural "zatwierdzenia", genitive plural "zatwierdzeń") and masculine for stash/tag ("schowek", "znacznik"). Adjectives and participles must agree: "każde zatwierdzenie jest pominięte" but "każdy commit jest pominięty".

## Glossary

`git` = default vocabulary (English git words, "Translate git terms" off). `local` = translated vocabulary (setting on), following git-l10n where it translates. Where both columns are the same, the English word is the Polish word.

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit (noun) | commit (m.) | zatwierdzenie (n.) | TortoiseGit "Zatwierdź"; verb in prose: "zatwierdzić" / "zrobić commit". |
| branch | gałąź | gałąź | git-l10n. Never "oddział". |
| merge | merge | scalenie / scalanie | verb "scal"; "edytor scalania", "narzędzie scalania". Compound: "merge commit" / "zatwierdzenie scalające". |
| rebase, cherry-pick, revert, reset (soft/mixed/hard), amend, autosquash, upstream, checkout, hunk, blame, ref, SHA, LFS, worktree, WIP, solo, lease, prune, force | same | same | git-l10n does not translate them either; "popraw" for the Amend checkbox, "wymuś wypchnięcie" for force push in local mode. |
| push | push | wypchnij / wypchnięcie | "force push" = "wymuszone wypchnięcie". |
| pull | pull | ściągnij / ściągnięcie | Button "Ściągnij"; never mixed with fetch. |
| fetch | fetch | pobierz / pobranie | "Pobierz wszystkie" for Fetch All. |
| stash | stash (m.) | schowek (m.) | pop = "przywróć", apply = "zastosuj", drop = "porzuć", verb "schowaj"; auto-stash = "automatyczny schowek". "Schowek" also means the system clipboard (Microsoft); the clipboard strings stay unambiguous by context ("skopiowano ... do schowka"). |
| tag | tag (m.) | znacznik (m.) | git-l10n. "opisany tag" = annotated. |
| diff | diff (m.) | różnice (pl.) | "widok różnic", "narzędzie porównywania" (external diff tool); the palette category `Diff:` becomes `Różnice:`. |
| stage / unstage / staged / unstaged | stage, unstage, staged, unstaged | dodaj do poczekalni, usuń z poczekalni, w poczekalni, poza poczekalnią | git-l10n "poczekalnia". Index (the same thing in prose) is "indeks". |
| pull request / merge request | pull request / merge request (pull requesty) | żądanie scalenia (both hosts) | GitLab Polish; Microsoft says "żądanie ściągnięcia" for pull requests, but "ŻĄDANIA ŚCIĄGNIĘCIA" does not fit the 18-cell sidebar title, so local mode uses one term for both hosts. PR / MR abbreviations stay Latin; the "Pull Request" / "Merge Request" setting choices stay English (indistinguishable otherwise). |
| remote, upstream | zdalne repozytorium / zdalne, upstream | same | "gałąź zdalna", "domyślne zdalne". |
| HEAD | HEAD | HEAD | "odłączony HEAD". |
| working tree | drzewo robocze | same | git-l10n. |
| conflict, resolve | konflikt, rozwiąż | same | "Bieżący" (A) / "Przychodzący" (B) / "Baza" / "Wynik" in the merge editor. |
| repository, submodule, settings, palette | repozytorium, submoduł, Ustawienia, paleta poleceń | same | |
| issue, label, profile, integration, token | zgłoszenie, etykieta, profil, integracja, token | same | |
| undo history, reflog, lost work, "restore only this" | Historia cofania, Reflog, Utracona praca, "Przywróć tylko tę" | same | "Cofnij do tego miejsca" / "Ponów do tego miejsca" for the up-to-here actions; "pin cofania" for the hidden ref that keeps an entry alive; "osierocony" for dangling; "reflog" stays Latin and indeclinable. The commit hash is "SHA" (never "ID"). |
| LFS local store | magazyn LFS | same | The English changed "repository" to "store" (`lfs prune`); "repozytorium" stays for the git repository only. |
| workspace, Radar, pin, snooze | obszar roboczy, Radar, przypnij / odepnij, odłóż | same | One word everywhere: "obszar roboczy" (never "przestrzeń robocza"); chip "Brak obszaru" (short). "Radar" is the feature name, never translated, declined as "w Radarze". Fetch/Pull all over a workspace are `{ git, local }` pairs ("Fetch wszystkich" / "Pobierz wszystkie", "Pull wszystkich" / "Ściągnij wszystkie"), not `{$Fetch} wszystko`. Search kinds: "Worktree" (indeclinable), "Zgłoszenia" (never "Issues"). Radar "Checkout gałęzi" as the rest of the catalog; "Do pierwszej zmiany" for "Until it changes". |
| Home / Log / Refs / Status | Home / Log / Refs / Status | same | pane and view names stay Latin. |

## Known residue (lint baseline `tests/git_vocabulary/pl.toml`)

Inherent, cannot be written without the English stem: placeholder or key names that are code (`{commit}`, `{stage}`, `{diff.edit}`, `{{diff.merge_editor}}`, `{{git_prefix.stash_pop}}`, `git.pull_mode`, `git.push_behavior`, `diff.tool`, `merge.tool`, `general.auto_stash`, `ui.graph_show_stashes`), the `stash@{n}` ref name, `<tag>` placeholders, the English prompt chips sent to the model (`ai_page.chip_*`), the "Pull Request" / "Merge Request" setting choices and their help, and the help of the "Translate git terms" setting itself. The baseline may only shrink.

## User guide wording (`docs/user/pl/`)

Choices made by the user-guide review (2026-10) where the catalog is silent or inconsistent. The guide quotes catalog labels exactly; this table covers the prose around them.

| Concept | Polish in the guide | Notes |
|---|---|---|
| checked-out branch | "bieżąca gałąź" | Never "wyewidencjonowana" or "wypożyczona"; "checkout" stays a Latin noun/verb ("zrób checkout"). |
| remote (the repository) | "zdalne repozytorium", "zdalne" | Not "remote'y" (fine only as part of a code name such as `remote/branch`). |
| credential helper | "menedżer poświadczeń" | As in the catalog; "(credential helper)" once in the installation page. |
| status bar / footer / status line | "pasek stanu" / "stopka" / "wiersz stanu" | Three different things in the English UI; "linia stanu" is not used. |
| lane (graph) | "ścieżka" | As in the catalog's `graph_lane_background`; "tor" only for a scrollbar track. |
| pill (ref label), badge | "plakietka" (pill), "plakietka klawisza" (the `Ctrl+N` badge) | |
| card (Compare pane) | "kafelek" | "karta" is the tab; conflict and failure cards say "karta"/"powiadomienie". |
| chip (a button-like marker) | "znacznik" | "próbka koloru" for a theme color chip. |
| built-in theme | "motyw wbudowany" | "ustawienie wstępne" (preset) is kept for Open in / AI presets, as in the catalog. |
| commit message | "opis" | "komunikat" is a system message (status line, error). |
| double-click (verb) | "kliknij dwukrotnie" | Noun in prose: "podwójne kliknięcie"; "dwuklik" only in narrow key columns. |
| right-click | "kliknij prawym przyciskiem" | Not "prawy klik". |
| line of a file / row of a list | "linia" / "wiersz" | Blame and diff lines are "linie". |
| "Pull requests" sidebar section | `PULL REQUESTS` | Upper case English as the app draws it in git vocabulary; never "PULL REQUESTY". |
| page names in links | "Opis CLI", "Opis konfiguracji", "Wsparcie AI", "Cofnij i ponów", "Stage i commity", "Praca z Logiem" | Keep the link text equal to the page's H1. |
| "Zob." | "zob. [Strona](...)" | Not "zobacz" before a link. |

## Undo history labels (`[undo_label]`, outside changes)

Labels are imperative rows built from the catalog's own words: "Odrzuć", "Przywróć", "Utwórz gałąź", "Usuń", "Zastosuj patch"; stage family and stash verbs are `{ git, local }` pairs ("Stage linii z {path}" / "Dodaj linie z {path} do poczekalni", "Stash zmian" / "Schowaj zmiany"), merge is "Merge" / "Scal". `commit` / `reword` / `fetch` / pop / apply / drop use the `{$term}` tokens. Plurals (`compose`, `cherry_pick`, `n_files`) use one/few/many with the genitive plural after 5 ("5 commitów" / "5 zatwierdzeń"). An outside change is "zmiana zewnętrzna" (tag `zewnętrzna`, setting "Zapisuj zmiany wykonane poza GitPoulpi"); the git reflog imitations (`outside_checkout`, `outside_reset`) are translated ("przejście z A do B"), while `(amend)` stays Latin. "tip" of a branch is "wierzchołek". The merge revert dropdown calls the kept parent "linia główna" (mainline).

## Review, metadata, boards and local issues (2026-10, round 4)

New vocabulary of the in-app code review, the metadata pickers, the Kanban board, the new providers and local Markdown issues. The English `request` is "żądanie" in prose and catalog; the rows that name a host's own word keep the `{$pr}` family. Palette categories: `Przegląd:` (review), `Tablica:` (board), `Zgłoszenia lokalne:` (local issues), `Hosting:` and `Radar:` unchanged.

| Concept | Polish | Notes |
|---|---|---|
| review (the activity, the view, the pending state) | przegląd ; verb "przejrzyj" | `Przegląd: Zakończ przegląd...`, "Przegląd kodu" (AI action). Never "recenzja" for the whole; "Poproszono o przegląd" already in the Radar. |
| reviewer / assignee | recenzent / osoba przypisana, "przypisani" in lists | GitLab Polish "Recenzenci". Pickers: "Edytuj recenzentów...", "Edytuj przypisanych..."; prose "osoby przypisane". |
| thread / conversation | wątek / dyskusja | "Odpowiedz", "Rozwiąż", "Otwórz ponownie"; badges "rozwiązany", "nieaktualny" (outdated). "dyskusja" = the non-code comments only. |
| pending comment / pending review | oczekujący komentarz / oczekujący przegląd | Never "szkic" (draft PR, Gerrit draft). Plural: one "oczekujący komentarz", few "oczekujące komentarze", many "oczekujących komentarzy". |
| Finish / Submit / Discard review | Zakończ przegląd / Wyślij przegląd / Odrzuć przegląd... | "Odrzuć" is the catalog's discard word, also for dismissing an AI suggestion. |
| verdict, Comment, Approve, Request changes | werdykt, Komentarz, Zaakceptuj, Zażądaj zmian | "Zatwierdź" is avoided: it is the local-mode word for commit. Badges: "zaakceptowano", "zażądano zmian", "skomentowano", "uchylono" (dismissed). |
| suggestion (AI), accept | sugestia, "Przyjmij" | "Przyjmij" / "Odrzuć", "Przyjmij wszystkie" / "Odrzuć wszystkie", "Użyj podsumowania AI". |
| viewed (file) | przejrzany | "Oznacz plik jako przejrzany"; "Pliki ({n}) - przejrzano: {viewed}". |
| patch set, vote | patch set (indeclinable-ish: "patch setu"), głos | Gerrit's own words stay Latin like the host; "głos" only in prose. |
| overview tab | Podsumowanie | Not "Przegląd", which is the review itself. |
| board, column, card | tablica, kolumna, karta | "karta" as the existing Trello `KARTY`; the tab is also "karta", the context separates them. "Zgłoszenia: Otwórz tablicę". |
| milestone | kamień milowy (gen. "kamienia milowego", pl. "kamienie milowe") | GitHub and GitLab Polish. "Ustaw kamień milowy...", "Brak kamienia milowego". |
| label | etykieta | as before; Gerrit's vote label stays "label" in the CLI help. |
| encoding | kodowanie | "Wykryj automatycznie" for Auto-detect; button "Kodowanie", value "Auto". |
| checks (CI), conflicts | kontrole, konflikty | "Kontrole zaliczone / nie przeszły / w toku"; check runs stay English. |
| mentioning me | wspominają o mnie | Radar group and filter. |
| local issues, notes folder, vault | zgłoszenia lokalne, folder notatek, skarbiec | Obsidian's Polish word is "skarbiec"; "Obsidian Base" stays Latin (file type). |
| head ref, merge base | head ref, baza scalenia | refs stay git words. |

Style notes of this round: status lines and `why_*` reasons in lower case, no final period; sentences shown in a view or dialog (`too_small`, `retry_hint`, notes) keep their capital and period; footer-hint key names stay Latin (`Enter`, `Esc`, `Shift+Left/Right`) except "Spacja"; AI chip texts (`chip_*_text`) are sent to the model and stay English. The `{$commits}` token is written lower case in mid-sentence rows ("Przegląd: Pokaż {$commits}").

## Branching workflows, Mermaid and Gerrit (2026-10, round 5)

Vocabulary of the `Flow` menu, the Workflow settings page, the host-driven issue flows, Copy as Mermaid gitGraph and the Gerrit compare / submit / abandon actions. The palette category is `Flow:` (an interface part name, stays Latin like `Git`, `Log`); `Przegląd:` for the Gerrit rows. Type labels (Feature, Hotfix) are user data and are never translated.

| Concept | Polish | Notes |
|---|---|---|
| branching workflow | przepływ pracy (z gałęziami) | Microsoft "workflow" = "przepływ pracy"; the toolbar button and category stay `Flow`. The settings page and menu say "Ustawienia przepływu pracy". |
| base branch | gałąź bazowa; `[baza]`, `[baza, chroniona]` badges | "trunk" stays Latin ("gałąź główna (trunk)"). |
| topic branch / topic type | gałąź tematyczna / typ gałęzi | Headings and rows say "Typy gałęzi" (short); "gałąź tematyczna" where the concept is named. |
| parent (of a branch) | gałąź nadrzędna; "Nadrzędna" in narrow labels | Never "rodzic" except in "rodzic commita" prose. "Ustaw gałąź nadrzędną...", "Synchronizuj z gałęzią nadrzędną...". |
| environment branch | gałąź środowiskowa | GitLab Flow. |
| start / finish / sync / promote | Rozpocznij / Zakończ / Synchronizuj / Promuj | Buttons and menu rows; "Start" is never used as a verb label. "Zakończ" here is the flow action, not the review's "Zakończ przegląd". |
| set parent | Ustaw gałąź nadrzędną (button: "Ustaw nadrzędną") | |
| hook | hook (plural "hooki") | git-l10n keeps it Latin. "Uruchom hooki". |
| pick up (a host-made branch) | przejmij / przejmowanie | "Przejmij gałąź utworzoną przez hosta...". Not "podnieś". |
| issue flow (host-driven) | przepływ ze zgłoszeń (sterowany przez host) | "host" (inflected "hosta", "hoście") as in the hosting guide; "bot" stays. |
| slug, token (name pattern) | slug, token | Latin, as in the CLI; "wzorzec nazwy" for the name pattern; the `{...}` names are code. |
| finish by / finish target | zakończ przez / cel zakończenia | "Merge lokalnie" / "Scalenie lokalnie" (token pair), "Otwórz {$pr}". |
| detect / import / template | wykryj / zaimportuj / szablon | Template names (Git Flow, GitHub Flow, GitLab Flow, Trunk-based, Custom) stay as the app draws them. |
| Mermaid detail levels | Struktura / Refs / Opisy / Pełne | Stored words `structure`, `refs`, `subjects`, `full` stay lowercase English in config and CLI. "Szczegóły" is the combobox label. |
| Mermaid direction | Kierunek: "Od lewej do prawej" / "Od góry do dołu" | The config key stays `orientation`. |
| Gerrit submit / abandon | Wprowadź / Porzuć (zmianę) | Not "Zatwierdź" (local-mode commit word) and not "Wyślij" (review submit). |
| compare patch sets | Porównaj patch sety; "Porównanie: baza" | |

Patterns of this round: a message that holds a git word and must inflect is a `{ git, local }` pair ("na merge commit" / "na zatwierdzeniu scalającym"), as the lint demands the English stem in git mode; where the sentence can use the nominative the `{$term}` token is kept. Status lines stay lower case without a final period ("utworzono zgłoszenie {reference}"); sentences shown in a card keep capital and period.
