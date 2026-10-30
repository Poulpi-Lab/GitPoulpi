# German (de) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/de.toml` follow this page. Sources: git's own German translation (git-l10n), Microsoft terminology for German (Visual Studio names stage "bereitstellen"), GitHub Desktop, VS Code, JetBrains, and TortoiseGit. Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Formal "Sie", in sentences only (dialogs, hints, errors). Never "du". Most strings avoid addressing the user at all. |
| Buttons, menu rows, palette commands | Infinitive or bare verb phrase, capitalised first word: "Datei öffnen", "Branch erstellen...". Palette: `Kategorie: Aktion` (colon directly after the category, no space before it). |
| Status messages in progress | "X wird ... geladen/gespeichert..." (passive, never imperative "Schreibe"). |
| Reasons for a greyed row ("why_*", "select_*") | Lower case infinitive: "zuerst einen Branch auswählen". |
| Titles and tabs | Nouns, capitalised. Short buttons use abbreviations only when a width budget forces it ("Rückg.", "Wiederh.", "Bearb."). |
| Generic wording | "Vorgang" for operation (not "Operation"), "Datenträger" for disk, "Arbeitsverzeichnis" for working tree, "Verlauf" for history, "Kürzel"/"Tastenbelegung" for shortcuts/keybindings. |
| Capitalisation | German noun rules. Git words used as nouns are capitalised (der Commit, der Stash, der Tag, der Rebase); verb forms are lower case (stagen, mergen, auschecken, pushen). |
| Quotes | Low-high "..." as „text“ (U+201E/U+201C). The plain ASCII apostrophe `'` is required by the catalog test (never `’`). |
| Ellipsis | Three ASCII dots `...` everywhere (menus, buttons, palette, placeholders, in-flight messages), as in English; the real `…` is not used. A command that opens a dialog or asks first has `...` ("Branch löschen...", "Andere Tabs schließen..."); one that runs at once has none. |
| Status lines | English status messages are lower-case clauses. German keeps the orthographic sentence start ("Datei gespeichert", "Keine Dateien im Konflikt"), as git-l10n does; a German first word that is a noun or the start of a sentence is not lower-cased. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Number/date | German writes "5. März 2024" (period after the day), `[date] day_month_year` is `{day}. {month} {year}`. Percent with a space ("50 %"). |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab` stay English: the chord printer in code shows them, so the text must match. German key labels that the app does not print are used as on the keycap: `Entf`, `Pos1`, `Ende`, `Einfg`, `Bild auf`/`Bild ab`, `Leertaste`. Never "Strg" (it would disagree with the footer hints). |
| Plural | `one` iff n = 1. |

## Genders and articles (the ones that bite)

der Commit, der Branch, der Tag, der Stash, der Merge, der Rebase, der Cherry-Pick, der Revert, der Fetch, der Push, der Pull, der Diff, der Hunk, der Index, der Pull Request / Merge Request, der Worktree, der HEAD ("losgelöster HEAD", "der HEAD"); das Remote, das Issue, das Blame (rarely needed), das Submodul, das Repository; die Ref, die Palette, die Konfiguration.

## Glossary

`git` = default vocabulary (English git words, "Translate git terms" off). `local` = translated vocabulary (setting on). Where both columns are the same, the English word is the German word.

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit (noun/verb) | Commit / committen | Commit / committen | git-l10n and every German tool keep it. "nicht committete Änderungen". |
| branch | Branch | Branch | Never "Zweig". |
| merge | Merge / mergen | Zusammenführen | Local follows git-l10n. "Merge-Editor", "Merge-Konflikt" are compounds and stay. |
| rebase | Rebase / rebasen | Rebase | git-l10n keeps "Rebase". |
| cherry-pick | Cherry-Pick | Cherry-Pick | "per Cherry-Pick übernehmen". |
| revert | Revert | Revert | Menu text: "rückgängig machen (Revert)". |
| reset | Reset | Reset | soft/mixed/hard stay English. |
| stash | Stash | Zwischenspeicher | Local verb: "beiseitelegen"; pop: "wiederherstellen"; apply: "anwenden"; drop: "verwerfen" (tokens `{$Pop}`, `{$Apply}`, `{$Stash_drop}`; the toolbar button is "Holen" in local mode, the full verb exceeds the 9-cell budget). |
| stage / unstage / staged / unstaged | stagen / unstagen / staged / unstaged | bereitstellen / Bereitstellung aufheben / bereitgestellt / nicht bereitgestellt | Microsoft (Visual Studio): "Bereitstellen". Index stays "Index". |
| working tree | Arbeitsverzeichnis | Arbeitsverzeichnis | git-l10n. A linked worktree (feature) is "Worktree". |
| HEAD | HEAD | HEAD | "losgelöster HEAD" (detached), as in git-l10n. |
| remote | Remote | Remote | das Remote. "Remote-Branch", "Remote-Tracking-Branch". |
| upstream | Upstream | Upstream | |
| fetch | Fetch / fetchen | Abrufen | Local: "Abruf-URL". Sentences with a plain verb in git mode may say "abrufen" (accepted, lint baseline-free). |
| pull | Pull / pullen | Holen | Local follows no git-l10n term (untranslated there); "Holen" is the TortoiseGit-style pick. |
| push | Push / pushen | Hochladen | "Erzwungenes Hochladen" for force push; "Force-Push (mit Lease)" in git mode. |
| tag | Tag | Tag | "annotierter Tag". |
| ref | Ref | Ref | die Ref. |
| diff | Diff | Unterschiede | Plural noun: no hyphen compounds with the token. Write explicit variants: "Diff-Ansicht" / "Unterschiedsansicht". |
| hunk | Hunk | Hunk | |
| blame | Blame | Blame | |
| conflict | Konflikt | Konflikt | "Datei im Konflikt", "ungelöst/gelöst". |
| ours / theirs / base | Ours / Theirs / Basis | same | Merge editor sides are "Aktuell (A)" and "Eingehend (B)". |
| pull request / merge request | Pull Request / Merge Request | Pullanforderung / Änderungsanfrage | Host dependent. Both feminine in local mode (git forms are masculine: write explicit pairs for articles). "Änderungsanfrage" because the uppercase sidebar title must fit 18 cells. Abbreviation PR / MR. |
| issue | Issue | Issue | das Issue. |
| log / graph | Log / Graph | Log / Graph | "Commit-Graph". |
| command palette | Befehlspalette | Befehlspalette | "Palette" alone is fine in context. |
| settings | Einstellungen | Einstellungen | Menu: "Einstellungen > Allgemein". |
| repository | Repository | Repository | das Repository, genitive "des Repositorys"; "Git-Repository". |
| clone / init | Klonen / Init | Klonen / Init | "Repository initialisieren". |
| worktree | Worktree | Worktree | |
| submodule | Submodul | Submodul | |
| LFS | Git LFS | Git LFS | "LFS-Objekt", "LFS-Zeiger". |
| tab | Tab | Tab | der Tab, "Tab-Leiste". |
| avatar / minimap | Avatar / Minimap | Avatar / Minimap | |
| amend | Amend | Amend | "Vorherigen Commit ändern" as the label. |
| interactive rebase actions | pick, reword, squash, fixup, drop, edit | übernehmen, umformulieren, zusammenfassen, einarbeiten, verwerfen, bearbeiten | Tokens `{$Pick}` etc. |
| undo / redo | Rückgängig / Wiederholen | same | "Rückgängig-Verlauf" (undo history), "Nur diese wiederherstellen" (restore only this), "Bis hierher rückgängig / wiederholen" (up to here), "Reflog", "Verlorene Arbeit" (lost work), "Rückgängig-Sicherung" (undo pin: a hidden ref that keeps a commit alive), "unreferenziert" (dangling). "Vorgang", never "Operation", for an entry of the journal. |
| outside change / merge mainline | außerhalb / Hauptlinie | same | "Änderung außerhalb von GitPoulpi" (outside), "Elternteil {n}" for a merge's parent, "Hauptlinie" for the mainline of a revert. |
| undo history row labels (`[undo_label]`) | verb phrase, verb last: "Branch {name} erstellen", "{path} stagen" / "{path} bereitstellen" | | Same words as the menus; unstage and stash-push/pop are `{ git, local }` pairs ("Bereitstellung von {path} aufheben", "Änderungen beiseitelegen"). Outside entries keep git's own reflog text (`checkout: Wechsel von ...`). |
| WIP | WIP | WIP | |
| theme / design | Design | Design | "Farbschema" for graph palette. |
| tooltip, toast | Tooltip, Benachrichtigung | same | |

## Known gaps

- Local-mode baseline (`tests/git_vocabulary/de.toml`) holds 10 keys that cannot be translated: an English prompt text for the model, placeholder or keymap ids inside the text (`{stage}`, `{{diff.close}}`, `{{merged}}`, `{{git_prefix.stash_pop}}`) and config keys (`general.auto_stash`, pull_mode values). Any new message with a git word needs a `{$term}` token or a `{ git, local }` pair.
- Local mode never prints `stash@{n}`: it says "Zwischenspeicher {n}" / "Nr. {n}".
- Palette command texts are generated names ("Git: Pull (Fast-Forward, wenn möglich)"); category prefixes: Git, Refs, Remote, Repository, Repositories, Merge, Diff, Navigation, Ansicht, Bearbeiten, Editor, Datei, Dateien, Änderungen, Kopieren, KI, App, Integrationen, Hosting, Issues, Log, LFS, Blame, Verlauf, Bild, Tab, Starter, Benachrichtigung, Aktivitätsprotokoll.

## User guide (`docs/user/de/`)

- Palette tables are regenerated from the catalog: the palette section of `keybindings.md` holds one row per English row, with the `[palette]` name of `de.toml` (git vocabulary). Keep the `palette:begin` / `palette:end` markers.
- Mouse chords use the key names the app prints: `Ctrl+Klick`, `Shift+Klick`, `Alt+Klick` (never "Strg" / "Umschalt").
- "Badge" for the focus badges of the navigation prefix; "Datenträger" (not "Platte") for disk; "hervorheben" (not "tönen") for highlight; "Datumsformate" for the date format keys.
- Settings are "Einstellungen" everywhere, including `Einstellungen > Repository-spezifische Einstellungen`.

## Workspaces, Radar, Suche (2026-10)

| Concept | German | Notes |
|---|---|---|
| workspace | Workspace | Never "Arbeitsbereich" (it was mixed in the search popup; unified). "Workspace-Chip", "Workspaces verwalten...". |
| Home tab | Home-Tab | Established; palette category "Start:" stays (`global_home`). Never "Tab Start". |
| Radar | Radar | Feature name, kept. |
| scope / view / show | Umfang / Ansicht / Anzeigen | Radar comboboxes. |
| pin / unpin | anheften / Anheften aufheben | Not "lösen" ("gelöst" reads as "solved"). |
| snooze / unsnooze | zurückstellen / nicht mehr zurückstellen | Heading "Zurückgestellt". |
| fetch all / pull all (workspace) | `{ git = "Fetch All", local = "Alle abrufen" }` / `"Pull All"`, `"Alle holen"` | Explicit pairs, as `fetch_all`: a `{$Fetch}` token inside "Alle ..." does not read in git mode. |
| locate (missing repo) | Suchen... | Row button. |
| missing / on disk | fehlt / Datenträger | "Festplatte" is not used. |
| review requested | Review angefragt | "Review" stays English (GitHub German UI). |
| new tab | Neuer Tab | The `+` and the transient tab. |
| load more | Mehr laden | |
| row actions | bare noun, no article: "Repository öffnen", "Branch auschecken", "Worktree erstellen...", "Im Repository zeigen", "Arbeit am Issue beginnen..." | Follows the English review. Status lines are lower case: "geladene Positionen: {n}", "{item} nicht mehr zurückgestellt". |

## Review, Metadaten, Board, lokale Issues (2026-10)

Sources: GitHub and GitLab German UI, Gerrit (no German UI, terms stay), Obsidian. "Anfrage" is the generic word for a pull/merge request in the TUI (dialogs, status lines, "Anfrage prüfen..."); the CLI help and the user guide pages about hosting keep "Pull-/Merge-Request" / "Request" (masculine) as before. Avoid articles before it so the git/local pair needs no extra variant.

| Concept | German | Notes |
|---|---|---|
| review (noun, view, verb) | Review / prüfen | das Review; palette category "Review:". "Anfrage prüfen...", "Review abschließen" (finish), "Review absenden" (submit), "Review verwerfen" (discard). |
| pending review / pending comment | ausstehendes Review / ausstehender Kommentar | "Ausstehend - Zeile 12" badge. Not "Entwurf" (that is a draft pull request). |
| thread | Thread | der Thread. Resolve = "Auflösen" (GitLab), reopen = "Wieder öffnen", resolved = "gelöst", outdated = "veraltet", fold = "ein- / ausklappen". |
| conversation | Unterhaltung | GitHub German UI. |
| reply | Antworten / "Auf Thread antworten" | |
| verdict | Beurteilung | List "Beurteilung": Kommentar, Genehmigen, Änderungen anfordern. Badges "genehmigt", "Änderungen angefordert", "kommentiert", "verworfen". |
| approve / request changes | Genehmigen / Änderungen anfordern | GitHub German UI. |
| suggestion (AI) | Vorschlag | Accept = "Übernehmen", dismiss = "Ablehnen" (not "Verwerfen": that deletes work), accept all = "Alle übernehmen". Severities: Hinweis, Warnung, Problem. "KI-Review" for the AI block. |
| viewed (file mark) | angesehen | GitHub German UI. |
| patch set | Patch Set | Gerrit term, kept English. |
| vote | Bewertung (Vote) | Gerrit label vote; first mention in help text adds "(Vote)". |
| assignee / reviewer / label / milestone | Zugewiesene / Reviewer / Label / Meilenstein | Nominalised adjective "Zugewiesene" (plural, no gender), "Reviewer" and "Label" stay English as on GitHub and GitLab German. "Zugewiesene bearbeiten...", "Meilenstein festlegen...". |
| encoding | Codierung | Microsoft and VS Code German. "Automatisch erkennen" for Auto-detect; the short button keeps "Auto". |
| board / column / card | Board / Spalte / Karte | "Issue-Board", "Spalten-Labels", "Karte verschieben nach...". Default columns "Offen" / "Geschlossen". |
| local issues / notes folder / vault | Lokale Issues / Notizordner / Vault | "Obsidian-Vault", "Obsidian Base" (feature name, no hyphen). |
| checks | Checks | "Checks bestanden / fehlgeschlagen / laufen". |
| mentioning me | Erwähnt mich | Radar group and filter. |
| minimap, hunk, split, inline | as before | Reuse the Diff view's own words ("Layout: Vereint oder nebeneinander", "Reine Leerraum-Änderungen"). |
| key name Space | Leertaste | In footer hints; `Shift+Links/Rechts` for the board's card move. |

## Branching-Workflows, Flow, Mermaid, Gerrit (2026-10, Runde 5)

Quellen: git-flow und GitLab Flow in deutschen Handbüchern, Git Town, GitHub und GitLab auf Deutsch, Mermaid-Dokumentation. Englische Fachwörter, die Entwickler im Deutschen so sagen, bleiben; Bindestrich-Komposita mit deutschem Grundwort.

| Concept | German | Notes |
|---|---|---|
| workflow | Workflow | der Workflow; "Workflow-Vorlage", "Workflow-Einstellungen" (Einstellungen > Repository-spezifische Einstellungen > Workflow). Palette-Kategorie und Schaltfläche bleiben `Flow`. |
| template / preset | Vorlage | `Git Flow`, `GitHub Flow`, `GitLab Flow`, `Trunk-based` bleiben (Produkt- bzw. Stilnamen); "Custom" = "Benutzerdefiniert" nur im Fließtext der Hilfe. |
| base branch | Basis-Branch | Badge `[Basis]` / `[Basis, geschützt]`; "Basis" allein im Kontext ("Basen"). |
| topic branch / topic type | Topic-Branch / Topic-Typ | "Topic" bleibt englisch (git-flow-Begriff); Feld "Typ". Nie "Themen-Branch". |
| parent (branch) | Eltern-Branch | Feldlabel in schmalen Zeilen kurz "Eltern". Ein Elternteil eines Merge-Commits bleibt "Elternteil {n}". |
| environment branch | Umgebungs-Branch | GitLab Flow; "Deployment-Branch" als Erklärung. |
| start / finish | starten / abschließen | Schaltflächen "Starten", "Abschließen"; Palette "Flow: Branch starten...". "Abschlussziel", "Abschlussmodus". |
| sync | synchronisieren | "Mit Eltern-Branch synchronisieren...", kurzer Button "Synchron." (Breitenbudget). |
| promote | hochstufen | GitLab-Flow-Umgebungs-Branches; nicht "befördern". |
| set parent | Eltern-Branch festlegen | Button kurz "Festlegen". |
| pick up (a host/bot-made branch) | aufgreifen | Nicht "übernehmen": "Übernehmen" ist schon `{$Pick}` und die Annahme eines Vorschlags. |
| hook | Hook | der Hook, "Shell-Befehle"; "Nach dem Start", "Vor dem Abschluss". |
| name pattern / token / slug | Namensmuster / Token / Slug | Platzhalter in `{{...}}` bleiben Latein; "Tracker-Schlüssel" für issue key; "regulärer Ausdruck". |
| issue flow | Issue-Flow | "vom Host gesteuert" für host-driven; "Wartekarte" für waiting card. |
| merge strategy | Merge-Strategie / `{$Merge}`-Strategie | Werte `merge`, `squash`, `rebase`, `Fast-Forward`. |
| Mermaid detail levels | Struktur, Refs, Betreffzeilen, Vollständig | "Betreff" ist git-l10n-Wort für subject; gespeicherte Werte `structure|refs|subjects|full` bleiben. Richtung: "Von links nach rechts" / "Von oben nach unten"; "Den ```mermaid-Block einschließen". |
| Gerrit submit / abandon | einreichen / aufgeben | "Change einreichen..." (nicht "absenden": das ist Review absenden); "Change aufgeben...". |
| compare patch sets | Patch Sets vergleichen | Combobox "Vergleich: Basis" / "Vergleich: Patch Set {n}". |
| copy as Mermaid gitGraph | Als Mermaid gitGraph kopieren... | `gitGraph` bleibt, wie in Mermaid geschrieben. |
