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

## Znajdź funkcję

| Chcę... | Strona | Jak do niej dojść w aplikacji |
|---|---|---|
| Pracować na dwóch gałęziach naraz | [Worktree](worktrees.md) | Sekcja **WORKTREES** panelu Refs (`+` tworzy worktree; ukryta, dopóki żaden nie istnieje: paleta **Panel boczny: Pokaż sekcję worktree**); paleta **Refs: Utwórz worktree...** |
| Pracować z zagnieżdżonymi repozytoriami | [Submoduły](submodules.md) | Sekcja **SUBMODUŁY** panelu Refs; paleta **Git: Aktualizuj wszystkie submoduły**, **Refs: Aktualizuj submoduł (zaznaczony)** |
| Rozwiązać konflikt merge | [Rozwiązywanie konfliktów merge](merge-conflicts.md) | `Enter` na wierszu `!` panelu Status, przycisk banera **Rozwiąż konflikty...** albo `Shift+E` w widoku Diff; paleta **Konflikty: Otwórz edytor merge** |
| Zrecenzować pull lub merge request | [Przegląd pull lub merge requesta](review.md) | `Enter` na wierszu sekcji **PULL REQUESTS**, `e` na wierszu Radaru, `Alt+R` w wyszukiwaniu obszaru roboczego; paleta **Hosting: Review Request...** |
| Przeglądać, tworzyć i obsługiwać pull requesty i zgłoszenia | [Pull requesty i zgłoszenia](pull-requests-and-issues.md) | Sekcje **PULL REQUESTS** i **ZGŁOSZENIA** panelu Refs (`n` tworzy nowy element); paleta **Pull requesty: Utwórz pull request...**, **Hosting: Nowe zgłoszenie...**. Najpierw połącz konto: [Hosting i trackery zgłoszeń](hosting-integration.md) |
| Czytać i odpowiadać w dyskusji zgłoszenia lub żądania | [Rozmowy](conversations.md) | `Enter` na wierszu zgłoszenia, karcie tablicy lub wierszu Radaru; paleta **Hosting: Otwórz rozmowę zgłoszenia...** |
| Zobaczyć swoje zgłoszenia jako tablicę | [Tablica zgłoszeń](pull-requests-and-issues.md#tablica-zgłoszeń) | Przycisk **Tablica** w nagłówku ZGŁOSZENIA; paleta **Zgłoszenia: Otwórz tablicę** |
| Trzymać zgłoszenia jako notatki Markdown, bez tokenu | [Zgłoszenia lokalne i Obsidian](local-issues.md) | Sekcja ZGŁOSZENIA, dostawca `Local Markdown - <nazwa>`; paleta **Zgłoszenia lokalne: Otwórz w Obsidian...** |
| Przepisać autorów, daty, komunikaty lub pliki w wielu commitach | [Przepisywanie historii](history-rewrite.md) | `Ctrl+G Shift+H`; paleta **Przepisanie: Warsztat historii** |
| Cofnąć (lub ponowić) operację | [Cofnij i ponów](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` otwiera historię cofania; paleta **Edycja: Cofnij** |
| Uruchomić kilka akcji jednym klawiszem (stage, wiadomość AI, commit, push...) | [Sekwencje](sequences.md) | Ustawienia > Sekwencje; paleta **Sekwencja: <nazwa>** |
| Śledzić duże pliki przez Git LFS | [Git LFS](lfs.md) | Paleta **LFS: Śledź wzorzec...**, **LFS: Zainicjuj dla tego repozytorium** |
| Znaleźć gałąź, commit, pull request lub zgłoszenie w wielu repozytoriach | [Wyszukiwanie](search.md) | `Ctrl+Alt+F` albo przycisk Szukaj wszędzie na pasku tytułu; `f` przeszukuje graf commitów bieżącego repozytorium |
| Zobaczyć wszystko, co na mnie czeka, w jednej liście | [Radar](radar.md) | `Alt+R`; paleta **Radar: Otwórz** |
| Pogrupować repozytoria i pobierać je lub aktualizować razem | [Obszary robocze](workspaces.md) | `Alt+W` przełącza, `Alt+J` otwiera kartę Obszary robocze; paleta **Obszar roboczy: Zarządzaj...** |
| Dostać pomoc AI (komunikaty commitów, wyjaśnienia, opisy) | [Wsparcie AI](ai.md) | `Ctrl+,` > **AI** > **Włącz AI**, potem `Ctrl+G i` pisze komunikat commita; paleta **AI: ...** |
| Znaleźć dowolne polecenie lub klawisz | [Skróty klawiszowe](keybindings.md) | `?` w aplikacji wyświetla klawisze, `Ctrl+P` znajduje dowolne polecenie |

## Codzienna praca

4. **[Praca z Logiem](log-and-history.md)**: czytanie grafu, wyszukiwanie,
   gałęzie, tagi, stashe, przeciąganie refów, rebase, cherry-pick,
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
8. **[Worktree](worktrees.md)**: drugi wyewidencjonowany folder na innej gałęzi:
   sekcja WORKTREES, tworzenie, otwieranie, scalanie i usuwanie powiązanych
   worktree.
9. **[Submoduły](submodules.md)**: sekcja SUBMODUŁY, ich aktualizowanie
   i utrzymywanie w zgodzie z checkoutami i pullami.
10. **[Cofnij i ponów](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, co można,
    a czego nie można cofnąć, kiedy odmawia lub pyta, jak daleko sięga
    i jak je wyłączyć.
11. **[Git LFS](lfs.md)**: duże pliki, śledzenie, blokady i co się dzieje bez
    zainstalowanego `git-lfs`.
12. **[Motywy](themes.md)**: wbudowane motywy, dowolny motyw z dowolną paletą
    linii, podgląd na żywo, własne pliki motywów, `gitpoulpi theme`.
13. **[Obszary robocze](workspaces.md)**: nazwane grupy repozytoriów z własnymi
    kartami i opcjonalnym domyślnym profilem commitów, plakietka na pasku kart,
    karta Obszary robocze, Fetch wszystkich, Pull wszystkich,
    Klonuj brakujące.
14. **[Wyszukiwanie](search.md)**: jedna przeszukiwarka (`Ctrl+Alt+F`) gałęzi,
    tagów, worktree, stashy, commitów, pull/merge requestów i zgłoszeń każdego
    repozytorium obszaru roboczego lub otwartych kart.
15. **[Radar](radar.md)**: pull/merge requesty, zgłoszenia i praca
    w toku z obszaru roboczego (albo z otwartych kart) na jednej liście: filtry,
    przypięcia, odłożenia i to, co może każdy wiersz.
16. **[Pull requesty i zgłoszenia](pull-requests-and-issues.md)**: sekcje
    PULL REQUESTS i ZGŁOSZENIA panelu bocznego, tablica zgłoszeń, tworzenie
    pull requesta lub zgłoszenia, znacznik na pasku narzędzi, odwołania
    w opisach commitów.
17. **[Rozmowy](conversations.md)**: czytanie i odpowiadanie w dyskusji
    zgłoszenia lub żądania w oknie, publikowanie jako oraz to, czego nie
    potrafi każdy host.
18. **[Przegląd pull lub merge requesta](review.md)**: widok przeglądu z wątkami
    komentarzy, oczekującym przeglądem i Zaakceptuj / Zażądaj zmian.
19. **[Otwórz w](open-in.md)**: lista na pasku tytułu, która otwiera folder
    aktywnej karty w terminalu, multiplekserze, edytorze lub menedżerze
    plików.
20. **[Podążanie za folderem aktywnej karty](following-the-tab-folder.md)**:
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

21. **[Skróty klawiszowe](keybindings.md)**: każdy klawisz, według obszaru,
    oraz każde polecenie palety. W aplikacji `?` pokazuje tę samą listę,
    a `Ctrl+P` znajduje dowolne polecenie.
22. **[Mysz i gesty](mouse-and-gestures.md)**: klikanie, przeciąganie,
    zaznaczanie i kopiowanie, paski przewijania i reakcje na najechanie.

## Ustawienia i integracje

23. **[Ustawienia](settings.md)**: każda strona i opcja ekranu Ustawień,
    pole wyszukiwania, profile i zmiana przypisań skrótów.
24. **[Wsparcie AI](ai.md)**: własne AI GitPoulpi (domyślnie wyłączone):
    lokalny agent taki jak `claude -p`, Anthropic, OpenAI, OpenRouter, Ollama
    lub LM Studio; instrukcje; opisy commitów, wyjaśnienia, opisy pull
    requestów i kompozytor commitów; prywatność i rozwiązywanie problemów.
25. **[Hosting i trackery zgłoszeń](hosting-integration.md)**: GitHub, GitLab, Gitea i Forgejo,
    Azure DevOps, Bitbucket, Gerrit oraz trackery zgłoszeń:
    tokeny i ich zakresy, łączenie, powiązanie repozytorium, klucze SSH,
    znane ograniczenia.
    Także **[Zgłoszenia lokalne i Obsidian](local-issues.md)**: folder notatek
    Markdown jako źródło zgłoszeń, bez tokenu, z Obsidian Base.

## Automatyzacja

26. **[Opis CLI](cli-reference.md)**: każde polecenie `gitpoulpi`, z `--json`
    dla skryptów.
27. **[MCP i agenci](mcp-and-agents.md)**: pozwól zewnętrznemu agentowi (Claude
    Code, Cursor, Codex, ...) sterować GitPoulpi: rejestracja serwera,
    `.mcp.json`, przełączniki uprawnień i ich wartości domyślne, każde
    narzędzie oraz sposób, w jaki agent coś pokazuje w oknie.
    Gotowy skill agenta to
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Dokumentacja referencyjna i pomoc

28. **[Opis konfiguracji](config-reference.md)**: każdy klucz `config.toml`,
    jego wartość domyślna i działanie, ustawienia poszczególnych repozytoriów,
    zmienne środowiskowe.
29. **[Rozwiązywanie problemów i FAQ](troubleshooting.md)**: ikony,
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

Wszystkie pozostałe nazwy produktów i usług wymienione w GitPoulpi i jego dokumentacji są znakami towarowymi swoich właścicieli, na przykład hostingi kodu (GitHub, GitLab, Bitbucket itd.), systemy śledzenia zgłoszeń (Jira, Trello, Linear itd.) oraz edytory i narzędzia, które GitPoulpi potrafi otworzyć (Visual Studio Code, środowiska IDE JetBrains, lazygit itp.). Służą wyłącznie do nazwania tego, z czym GitPoulpi współpracuje; GitPoulpi nie jest powiązany z żadnym z nich, przez żaden z nich zatwierdzony ani sponsorowany.
