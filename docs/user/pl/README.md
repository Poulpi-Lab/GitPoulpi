[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · **Polski** · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Dokumentacja użytkownika GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="Ośmiornica GitPoulpi" width="128"></p>

GitPoulpi to klient Git dla terminala z klikalnym grafem commitów, paletą
poleceń, integracją z hostingami i trackerami zgłoszeń, opcjonalną pomocą AI
oraz CLI i serwerem MCP, które robią wszystko to, co interfejs. Strony są
ułożone w kolejności czytania: zacznij od góry, a do stron referencyjnych
przechodź wtedy, gdy będą potrzebne.

![Główne okno GitPoulpi](../images/hero.png)

## Zacznij tutaj

1. **[Instalacja](installation.md)**: czego potrzeba, archiwa wydań,
   menedżery pakietów, budowanie ze źródeł. Mniejszy plik binarny opisują
   **[Własne kompilacje](custom-builds.md)**. Jak GitPoulpi sam dba o aktualność
   (znaczek wersji, panel aktualizacji, powrót do poprzedniej wersji):
   **[Aktualizacje](updates.md)**. Uruchamianie w kontenerze:
   **[Uruchamianie w Dockerze](docker.md)**. Dla administratorów: wewnętrzne feedy,
   lustra i polityka aktualizacji: **[Dystrybucja w przedsiębiorstwie](enterprise-distribution.md)**.
2. **[Pierwsze kroki](getting-started.md)**: pierwsze pięć minut, od
   uruchomienia do pierwszego commita.
3. **[Przewodnik po interfejsie](interface-tour.md)**: nazwa każdego obszaru
   okna, z obrazkami.

## Codzienna praca

4. **[Praca z Logiem](log-and-history.md)**: czytanie grafu, wyszukiwanie,
   gałęzie, tagi, stashe, worktree, przeciąganie refów, rebase, cherry-pick,
   reword i poprawianie commitów, banery, pull i push, auto-stash.
5. **[Stage i commity](staging-and-committing.md)**: zmiany robocze, diffy,
   stage hunków i pojedynczych linii, pisanie commitów, kompozytor commitów,
   blame, historia pliku i wbudowany edytor.
6. **[Obrazy](image-view.md)**: obrazy w widoku Diff i widoku pliku, przed
   i po, powiększenie, Edytuj w.
7. **[Rozwiązywanie konfliktów merge](merge-conflicts.md)**: edytor merge (A i B
   obok siebie nad Wynikiem), wybieranie linii i fragmentów, karty dla
   konfliktów plików binarnych, LFS, usuniętych, dowiązań symbolicznych
   i submodułów oraz sposób, w jaki agent proponuje rozwiązanie.
8. **[Cofnij i ponów](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, co można,
   a czego nie można cofnąć, kiedy odmawia lub pyta, jak daleko sięga
   i jak je wyłączyć.
9. **[Git LFS](lfs.md)**: duże pliki, śledzenie, blokady i co się dzieje bez
   zainstalowanego `git-lfs`.
10. **[Submoduły](submodules.md)**: sekcja SUBMODUŁY, ich aktualizowanie
    i utrzymywanie w zgodzie z checkoutami i pullami.
11. **[Motywy](themes.md)**: wbudowane motywy, dowolny motyw z dowolną paletą
    linii, podgląd na żywo, własne pliki motywów, `gitpoulpi theme`.
12. **[Obszary robocze](workspaces.md)**: nazwane grupy repozytoriów z własnymi
    kartami i opcjonalnym domyślnym profilem commitów, plakietka na pasku kart,
    karta Obszary robocze, Fetch wszystkich, Pull wszystkich,
    Klonuj brakujące.
13. **[Wyszukiwanie](search.md)**: jedna przeszukiwarka (`Ctrl+Alt+F`) gałęzi,
    tagów, worktree, stashy, commitów, pull/merge requestów i zgłoszeń każdego
    repozytorium obszaru roboczego lub otwartych kart.
14. **[Radar](radar.md)**: pull/merge requesty, zgłoszenia i praca
    w toku z obszaru roboczego (albo z otwartych kart) na jednej liście: filtry,
    przypięcia, odłożenia i to, co może każdy wiersz.
    Czytanie i przegląd jednego z tych requestów w aplikacji, z wątkami
    komentarzy, oczekującym przeglądem i Zaakceptuj / Zażądaj zmian:
    **[Przegląd pull lub merge requesta](review.md)**.
15. **[Otwórz w](open-in.md)**: lista na pasku tytułu, która otwiera folder
    aktywnej karty w terminalu, multiplekserze, edytorze lub menedżerze
    plików.
16. **[Podążanie za folderem aktywnej karty](following-the-tab-folder.md)**:
    „Zakończ i cd tutaj” oraz wrapper `gitpoulpi shell-init` (`gp`) dla bash,
    zsh, fish i PowerShell.
    Który terminal pokazuje GitPoulpi najlepiej: **[Wybór terminala](terminals.md)**.
    Co przechodzi przez tmux, zellij, screen, herdr i panele terminala:
    **[Multipleksery](multiplexers.md)**.

Także: **[Kilka zdalnych dla jednej gałęzi](multiple-remotes.md)**: cele push i źródła pull,
okno Zdalne gałęzi, Push do wszystkich lub do wybranych zdalnych, rozstrzyganie zdalnego,
które się przesunęło, Kontynuuj po konflikcie, CLI i alias zwykłego gita.

**[Przepływy pracy z gałęziami](branching-workflows.md)**, gotowe szablony
Git Flow, GitHub Flow, GitLab Flow i Trunk-based albo własne bazy, wzorce nazw
gałęzi i typy gałęzi tematycznych (`gitpoulpi flow ...`).

**[Przepisywanie historii](history-rewrite.md)**: warsztat (`Ctrl+G Shift+H`), który
przepisuje autorów, daty, komunikaty i zawartość plików w wielu commitach, z podglądem,
skryptami Lua i jednym cofnięciem.

## Klawiatura i mysz

17. **[Skróty klawiszowe](keybindings.md)**: każdy klawisz, według obszaru,
    oraz każde polecenie palety. W aplikacji `?` pokazuje tę samą listę,
    a `Ctrl+P` znajduje dowolne polecenie.
18. **[Mysz i gesty](mouse-and-gestures.md)**: klikanie, przeciąganie,
    zaznaczanie i kopiowanie, paski przewijania i reakcje na najechanie.

## Ustawienia i integracje

19. **[Ustawienia](settings.md)**: każda strona i opcja ekranu Ustawień,
    pole wyszukiwania, profile i zmiana przypisań skrótów.
20. **[Wsparcie AI](ai.md)**: własne AI GitPoulpi (domyślnie wyłączone):
    lokalny agent taki jak `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama
    lub LM Studio; instrukcje; opisy commitów, wyjaśnienia, opisy pull
    requestów i kompozytor commitów; prywatność i rozwiązywanie problemów.
21. **[Hosting i trackery zgłoszeń](hosting-integration.md)**: GitHub, GitLab, Gitea i Forgejo,
    Azure DevOps, Bitbucket, Gerrit oraz trackery zgłoszeń:
    tokeny i ich zakresy, łączenie, powiązanie repozytorium, pull requesty
    i zgłoszenia, odwołania w opisach commitów, klucze SSH, znane ograniczenia.
    Także **[Zgłoszenia lokalne i Obsidian](local-issues.md)**: folder notatek
    Markdown jako źródło zgłoszeń, bez tokenu, z Obsidian Base.

## Automatyzacja

22. **[Opis CLI](cli-reference.md)**: każde polecenie `gitpoulpi`, z `--json`
    dla skryptów.
23. **[MCP i agenci](mcp-and-agents.md)**: pozwól zewnętrznemu agentowi (Claude
    Code, Cursor, Codex, ...) sterować GitPoulpi: rejestracja serwera,
    `.mcp.json`, przełączniki uprawnień i ich wartości domyślne, każde
    narzędzie oraz sposób, w jaki agent coś pokazuje w oknie.
    Gotowy skill agenta to
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Dokumentacja referencyjna i pomoc

24. **[Opis konfiguracji](config-reference.md)**: każdy klucz `config.toml`,
    jego wartość domyślna i działanie, ustawienia poszczególnych repozytoriów,
    zmienne środowiskowe.
25. **[Rozwiązywanie problemów i FAQ](troubleshooting.md)**: ikony,
    renderowanie grafu, klawisze, które nie docierają, zawieszone demony,
    szybkość, wskaźniki LFS.

## Zrzuty ekranu

Każdy obrazek na tych stronach to prawdziwy zrzut z wersji release, na małym
repozytorium demonstracyjnym ze zmyślonymi autorami. Obrazki są w
[`images/`](../images/). `scripts/docs-screenshots.sh` buduje repozytoria
demonstracyjne i robi je od nowa. Powstały w terminalu `kitty`, który rysuje
graf commitów i awatary autorów jako obrazy. Terminale bez protokołu
graficznego pokazują zamiast tego graf znakowy.

## Dla współtwórców

[CLAUDE.md](../../../CLAUDE.md), [przegląd architektury](../../architecture/overview.md)
i [plan rozwoju](../../roadmap.md) opisują projekt i to, co jest planowane;
[CHANGELOG.md](../../../CHANGELOG.md) wymienia zmiany w każdym wydaniu.

---

GitKraken jest znakiem towarowym swojego właściciela. GitPoulpi jest niezależnym projektem, niepowiązanym z tym właścicielem, niezatwierdzonym i niesponsorowanym przez niego; nazwa jest wymieniona wyłącznie w celu opisania zgodności i inspiracji.
