# Turkish (tr) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/tr.toml` follow this page. Sources: git's own Turkish translation (git-l10n `po/tr`), Microsoft terminology and style guide for Turkish (also Azure DevOps and Visual Studio Turkish UI), GitHub Desktop, VS Code, JetBrains and TortoiseGit in Turkish, and what Turkish developers say aloud (the English git word, with a Turkish suffix). Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Instructions to the reader use the polite plural imperative ("Seçin", "girin", "denetleyin"), never the familiar "sen" form and never "siz" written out. Possessives are neutral ("değişiklikleriniz" where the user's own work is meant, otherwise no possessive). |
| Buttons, menu rows | Bare 2nd person singular imperative, as in Windows/Office Turkish: "Kaydet", "Aç", "Oluştur...", "Dalı sil". Progress: present continuous passive with an ellipsis ("Kaydediliyor...", "{name} birleştiriliyor..."). Results: past passive ("kaydedildi", "silindi"). Status and error text starts lower case, no final full stop (also the progress lines of the footer: "stage'leniyor...", "commit ediliyor..."), except when it starts with an acronym ("YZ", "LFS"). |
| Palette and keymap names | `Kategori: Eylem`, colon directly after the category (no space before it), Title Case actions ("Seçili Dosyayı Sakla"). Categories are translated (Gezinme, Dosya, Görünüm, Depo, Uygulama, Ajan) except interface part names (`Git`, `LFS`, `Refs`, `Log`, `Blame`, `Remote`, `Banner`, `Composer`). In local mode `Merge:` and `Diff:` become `Birleştirme:` and `Fark:`. |
| Titles, tabs, columns | Sentence case for sentences, Title Case for dialog and section titles ("Etkinlik Günlüğü", "Depo Yönetimi"). Sidebar section headings are upper case; Turkish upper case uses `İ`/`I` correctly (the build handles it for `{$TERM}` tokens). They are named as the sidebar shows them (YEREL, UZAK, ALT MODÜLLER, ISSUE'LAR), so help text must use the same words. |
| Suffixes on foreign words | Foreign words (git words, GitPoulpi, SHA, ref, crate) take an ASCII apostrophe and a suffix chosen by how the word is pronounced: commit'in, commit'e, commit'te, ref'i, HEAD'in, SHA'sı, GitPoulpi'nin, stash'i, tag'e. When the word may be a `{$term}` token (it changes between git and local vocabulary) do not hang a suffix on it: reword ("{$Commit} mesajı") or write a `{ git, local }` pair. |
| Quotes | Typographic double quotes “...” for UI names and values, or plain `"..."` inside code-like text. The plain ASCII apostrophe `'` is required by the catalog test; never `’`. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Ellipsis | Three ASCII dots `...` everywhere, as in the English source (the real `…` is no longer used): on commands that open a dialog, and on progress text ("kaydediliyor..."). |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Home/End`, `PgUp/PgDn` stay Latin (Turkish keyboards print them). In prose "Space" and "Boşluk" both appear for the space bar; arrows are "Yukarı/Aşağı/Sol/Sağ". |
| Mouse | "tık" (click, short form), "çift tık", "sağ tık"; in full sentences "tıklayın". "Shift+tık", "Ctrl+tık" as compounds. |
| Dates and times | `{day} {month} {year}` ("5 Mart 2024"), month names capitalised, weekdays capitalised ("Salı"), three-letter month abbreviations ("Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara"), relative times "az önce", "5 dakika önce" (no plural form: Turkish does not pluralise after a numeral). |
| SHA | "SHA değeri" ("{$commit} SHA değerini kopyala"), never "kimlik" for the commit id; a suffix on a `{$commit}` token is avoided. |
| Undo history | Nouns for the action in "{op} bitti" and "{name} dalını {op} eyleminin bıraktığı yere": "atma", "dosya silme", "dal silme", "uzak depoyu kaldırma". Tabs and titles: "Geri alma geçmişi", "Reflog", "Kaybolan çalışma"; buttons "Yalnızca bunu geri yükle", "Buraya kadar geri al". |
| Undo history labels | `[undo_label]` rows are short imperative phrases, `Fiil: nesne` (colon) when the object is a free name or message ("Commit: {subject}", "Dal oluştur: {name}", "Sil: {what}", "At: {what}") and an object-first phrase when a path takes a case suffix ("{path} dosyasını {$stage}", "{path} klasörünü {$stage}"). Git words stay as in the rest of the catalog (Checkout, Amend, Revert, Reset, hunk, fast-forward, upstream). Plural labels keep the noun singular after the number, with `{$commit}` (never `{$commits}`) in both forms. |
| Outside changes | "dışarıda" (tag), "dış değişiklik" (outside change), "GitPoulpi dışında yapılan değişiklikler"; the git-reason fragments (`checkout: ... konumundan ... konumuna geçiliyor`) are translated, the leading git verb stays. |
| Merge parent | "ebeveyn" (git-l10n), "ana hat" for mainline: "Hangi ebeveyn ana hat (kalacak taraf)?". |
| Plurals | CLDR tr: `one` iff n = 1, otherwise `other`; the noun stays singular after a number ("3 dosya", "2 commit"). |
| Generic wording | "Ayarlar" for Settings, never "Tercihler" (the English source says Settings everywhere); "sekme" (tab), "pencere" (window), "bölme" (pane in editors) and "panel" (pane of the main screen), "kenar çubuğu" (sidebar), "komut paleti", "barındırıcı" (host), "sağlayıcı" (provider), "token" (token), "kimlik bilgileri" (credentials), "çalışma ağacı" (working tree; never "çalışma dizini"), "klasör" (directory, never "dizin"), "indeks" (the git index), "mesaj" (message; not "ileti"), "issue" (kept English), "YZ" (AI), "ajan" (agent), "ön ayar" (preset). |

## Glossary

`git` = default vocabulary (English git words, "Translate git terms" off). `local` = translated vocabulary (setting on). Where both columns are the same, the English word is also the Turkish developer word (git-l10n does not translate it consistently either).

| Concept | git | local | Notes / source |
|---|---|---|---|
| commit | commit | commit | Turkish developers say "commit" in both; git-l10n has "işleme" but it is not used by GitHub Desktop, VS Code or TortoiseGit in Turkish. Kept in both modes, so no suffix problems. The verb is "commit et" / "commit yap". |
| branch | dal | dal | git-l10n. Never "şube". |
| merge | merge | birleştir / birleştirme | Microsoft, git-l10n. "Birleştirme düzenleyicisi", "harici birleştirme aracı"; verb in git mode "merge et". Palette category `Merge:` becomes `Birleştirme:`. |
| rebase, cherry-pick, revert, reset (soft/mixed/hard), amend, autosquash, checkout, hunk, blame, ref, SHA, LFS, worktree, WIP, solo, lease, prune, upstream | same | same | git-l10n does not translate them; Turkish developers use them as is ("rebase et", "checkout yap"). |
| push | push | gönder / gönderme | Microsoft "Gönder". "force push" = "zorla gönder". |
| pull | pull | çek / çekme | Microsoft "Çek". Never mixed with fetch. |
| fetch | fetch | getir / getirme | Microsoft "Getir". "Tümünü Getir" for Fetch All. |
| stash | stash | saklama (noun), sakla (verb) | Plural "saklamalar"; pop = "geri yükle ve kaldır" (the toolbar keeps the label "Pop"), apply = "uygula", drop = "at". `stash@{n}` is the git ref name and stays. |
| tag | tag | etiket | git-l10n. "Açıklamalı etiket" = annotated. |
| diff | diff | fark | Microsoft. "Fark görünümü"; palette category `Diff:` becomes `Fark:`. |
| stage / unstage / staged / unstaged | stage'le, unstage'le, staged, unstaged | hazırla, hazırlıktan çıkar, hazırlanmış, hazırlanmamış | "Hazırlık" (staging area) as in git-l10n "hazırlık alanı". The index itself is "indeks". |
| pull request / merge request | Pull Request / Merge Request | Çekme İsteği / Birleştirme İsteği | Microsoft "çekme isteği"; GitLab Turkish "birleştirme isteği". Plural in the sidebar title local mode: "ÇEKME İSTEKLERİ" fits 18 cells, "BİRLEŞTİRMELER" is used for merge requests because "BİRLEŞTİRME İSTEKLERİ" does not. Abbreviations PR / MR stay Latin. |
| remote, upstream | uzak depo, upstream | uzak depo, upstream | git-l10n "uzak". "Uzak dal" = remote branch. |
| HEAD | HEAD | HEAD | "ayrık HEAD" for detached. |
| working tree | çalışma ağacı | çalışma ağacı | git-l10n. |
| conflict, resolve | çakışma, çöz | çakışma, çöz | Merge editor sides: "Mevcut" (A, ours), "Gelen" (B, theirs), "Temel" (base), "Çıktı" (output). |
| repository, submodule, settings, palette | depo, alt modül, Ayarlar, komut paleti | same | |
| issue, label, profile, integration, token | issue, etiket (label), profil, entegrasyon, token | same | "Etiket" is also the local word for git tag; the context (issue form vs Refs) separates them. |
| workspace, Radar, pin, snooze | çalışma alanı, Radar, sabitle, ertele | same | "Çalışma alanı" everywhere (never "iş alanı" or "çalışma ortamı"); in palette rows "Çalışma Alanı:" is Title Case after the category. Radar stays English (product name; "Radar'ı" takes a plain suffix). "Sabitle / Sabitlemeyi kaldır" (pin), "Ertele / Ertelemeyi kaldır" (snooze); "Süren işler" for work in progress, "WIP" for the view chip. "Eksik" = missing repo, "Bul..." = Locate, "Eksikleri klonla" = Clone missing. Issues are "issue" everywhere, also in Radar ("Issue'lar", "Bana atanan issue'lar"), never "sorun" (the word for problems). |
| fetch all / pull all (workspace) | tümünü fetch et, tümünü pull et | tümünü getir, tümünü çek | Written as `{ git, local }` pairs (a verb with a `{$fetch}` token would read "tümünü Fetch" in git mode). Progress and result lines use the passive ("fetch ediliyor" / "getiriliyor", "fetch edildi" / "getirildi"). |
| Home / Log / Refs / Status | Ana Sayfa (Home tab), Log, Refs, Durum | same | Pane and view names that are interface labels. |
| review (code review) | İnceleme (noun, palette category `İnceleme:`), "incelemek" (verb), "Kod incelemesi" (the AI action) | same | Round 4. "Review" is not kept English: GitHub and Azure DevOps Turkish say "inceleme". "İncele..." is the row-menu entry, "İncelemeyi bitir" Finish review, "İncelemeyi gönder" Submit review, "İncelemeyi at..." Discard review. |
| request (the pull/merge request seen as a review object) | istek | istek | "Bir istek veya issue seçin"; the full name stays "Pull Request" / "Merge Request" via the term token. |
| thread | yorum dizisi | yorum dizisi | "Yorum dizisini yanıtla / çöz / daralt". Conversation = "konuşma" (GitHub Turkish). Never "konu" (topic). |
| pending comment | bekleyen yorum | bekleyen yorum | "Bekleyen - satır 12", "bekleyen inceleme". Posted at once = "hemen gönderilir". |
| resolve / reopen (a thread) | çöz / yeniden aç | çöz / yeniden aç | Same verb as for merge conflicts; the context (thread) separates them. Badges: "çözüldü", "eskimiş" (outdated). |
| verdict, approve, request changes, comment | karar, Onayla, Değişiklik iste, Yorum | same | Submitted-review badges: "onayladı", "değişiklik istedi", "yorum yaptı", "yoksayıldı" (dismissed). |
| viewed (file mark) | görüldü | görüldü | "Dosyayı Görüldü Olarak İşaretle". |
| suggestion (AI) | öneri | öneri | "Öneriyi Kabul Et / Yoksay / Düzenle ve Kabul Et", "Tümünü kabul et". Severities: bilgi, uyarı, sorun. "YZ özeti" = AI summary. |
| patch set, vote (Gerrit) | yama kümesi, oy | yama kümesi, oy | "Yama kümesi 3 (en son)"; labels `Code-Review` and `Verified` stay Latin. |
| assignee, reviewer, label, milestone | atanan, inceleyen, etiket, kilometre taşı | same | GitHub Turkish wording ("Atananlar", "İnceleyenler", "Etiketler", "Kilometre taşı"). "Atananları düzenle...", "Kilometre taşı ayarla...". |
| mentions (Radar) | "Benden bahsedenler" | same | Group and filter: items that mention me. |
| encoding, Auto-detect | kodlama, Otomatik algıla | same | Button value "Otomatik"; encoding names (UTF-8, Windows-1252...) stay Latin. |
| board, column, card | pano, sütun, kart | same | Trello, Jira and Azure DevOps Turkish call a board "pano". "Issue panosu", "Pano: Kartı Şuraya Taşı...", button "Pano". |
| notes folder, vault | notlar klasörü, kasa | same | "Obsidian kasası" (vault), "Obsidian Base" stays English (file type). Local issues = "yerel issue'lar", palette `Yerel Issue'lar:`. |
| instructions (AI) | talimat | talimat | Already used by the catalog ("Genel talimatlar", "Depo talimatları"); workspace level = "Çalışma alanı talimatları". Not "yönerge". |
| Gitea / Forgejo / Linear / YouTrack / Redmine / Gerrit | brand names | same | Never translated or suffixed with a token; "Gerrit'te" takes a plain apostrophe suffix. |
| workflow, Flow | iş akışı, Flow | same | Round 5. "İş akışı" (Microsoft, Azure DevOps) for the branching workflow; the toolbar button and the palette category stay `Flow` ("Flow: Menüyü Aç"), like the other interface part names. Settings page "İş akışı"; path "Ayarlar > Depoya Özel Ayarlar > İş akışı". Templates (Git Flow, GitHub Flow, GitLab Flow, Trunk-based, Custom/Özel) keep their names. |
| base branch, topic branch / type, parent, environment branch | temel dal, konu dalı / konu türü, üst dal, ortam dalı | same | "Temel dal" (long-lived; badge `[temel]`, `[temel, korumalı]`), "konu dalı" as in git documentation, "konu türü" for the kind. The branch parent is "üst dal" (not "ebeveyn", which stays the merge parent of git-l10n). Trunk = "ana hat". |
| start, finish, sync, promote, set parent | başlat, bitir, eşitle, yükselt, üst dalı ayarla | same | Buttons and menu rows are bare imperatives ("{label} başlat...", "Geçerli dalı bitir...", "Üst dalla eşitle...", "Yükselt..."). Status lines are lowercase ("bitti: ...", "iptal edildi"). Continue / Abort = "Devam" / "İptal". |
| pick up (host-made branch) | devral | devral | "Dalı devral", "Barındırıcının oluşturduğu dalı devral...", status "{name} devralındı". Not "al" or "benimse". |
| hook, slug, token (name pattern) | hook, slug, token | same | Kept English with an apostrophe suffix ("Hook'lar", "Token'lar"); "ad deseni" for name pattern, "düzenli ifade" for regular expression. |
| issue flow (host-driven) | issue akışı, barındırıcı tarafından yürütülen | same | "Dalı oluşturan: siz / barındırıcı / bir bot". Waiting card: "Beklemeyi bırak" (Stop waiting), "Sonra" (Later). |
| Mermaid gitGraph popup | Mermaid gitGraph | same | Dialog title "Mermaid gitGraph Olarak Kopyala"; detail levels "Yapı", "Refs" (interface part name), "Başlıklar" (commit subjects, as in the commit composer), "Tam"; "Yön" with "Soldan sağa" / "Yukarıdan aşağıya". The stored values `structure`, `refs`, `subjects`, `full` stay lowercase Latin in docs and CLI help. |
| Gerrit submit / abandon | teslim et, vazgeç / terk et | same | "Teslim et" for Submit (kept apart from "gönder" = push and "İncelemeyi gönder"); button "Vazgeç", confirmation "terk edilsin mi" and status "terk edildi" for Abandon. Palette: "İnceleme: Değişikliği Teslim Et...", "İnceleme: Değişiklikten Vazgeç...". |

## Known residue (lint baseline `tests/git_vocabulary/tr.toml`)

Inherent, cannot be written without the English stem: placeholder or key names that are code (`{tag}`, `{stage}`, `{{merged}}`, `{{diff.merge_editor}}`, `{{git_prefix.stash_pop}}`, `git.pull_mode`, `git.push_behavior`, `general.auto_stash`, `ui.graph_show_stashes`), the `stash@{n}` ref name, the English prompt chip sent to the model (`ai_page.chip_task_context_text`), the "Pull Request" / "Merge Request" setting choices and the help that names them, config values in `pull_mode_help` (`merge`, `rebase`), and the help of the "Translate git terms" setting itself. The baseline may only shrink.

## User guide (`docs/user/tr/`)

Decisions of the guide review (2026-10); the catalog wording always wins for a quoted label.

| Topic | Decision |
|---|---|
| Address | Polite plural in sentences ("tıklayın", "yazın", "basın"); bare imperative only in table action cells and menu lists, as in the UI. Never "sen" or "-in/-ın" singular forms. |
| Panes | Main-screen panes are "panel" ("sağ panel", "orta panel", "Durum paneli"); "bölme" only for editor-internal panes (merge editor A/B/Base, image view) and terminal multiplexers. |
| Banner | The strip above the Log is "banner" (as the palette prefix `Banner:`); never "şerit" for it ("şerit" = graph lane). |
| Home tab | "Ana sekme" (catalog: "Ana sekme (Depo Yönetimi)"), never "Ana sayfa sekmesi". |
| Composer | "commit composer" / "composer" stays English, as in the catalog. |
| Filter | "filtre" (never "süzgeç"); "token" (never "jeton" or "belirteç"); "mesaj" (never "ileti"); "klasör" and "indeks" (never "dizin"). |
| Clicks | "tık" compounds in key chords ("Ctrl+tık", "Shift+tık", "Alt+tık"); "tıklayın" in sentences. |
| Suffix after code | ASCII apostrophe after an inline code span: `` `default`'tur ``, `` `dark`'a ``. |
| Code blocks | Comments inside shell and TOML examples are translated; commands, keys and values are not. The generated blocks (keymap, palette, MCP tools) are produced by the app and are not edited by hand. |
| Link texts | Equal to the target page's title: "Log ile çalışmak", "Stage'leme ve commit'leme", "Merge çakışmalarını çözmek", "Etkin sekmenin klasörünü izlemek", "YZ desteği", "Barındırma hizmetleri ve issue takipçileri". |
