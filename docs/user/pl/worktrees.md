[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · **Polski** · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktree

Worktree to druga rozpakowana kopia repozytorium: inny folder, na innej
gałęzi, ze wspólną historią. Sekcja WORKTREES panelu Refs je wylicza; główny
jest oznaczony `(główny)`.

## Pokazywanie sekcji

W repozytorium, które nie ma jeszcze powiązanego worktree, sekcja jest ukryta.
Żeby ją mimo to zobaczyć (pokazuje wtedy „Brak worktree: + tworzy nowe”),
użyj polecenia palety **Panel boczny: Pokaż sekcję worktree**, zaznacz
**WORKTREES** na liście pokaż/ukryj panelu bocznego (prawy przycisk myszy na
nagłówku sekcji) albo uruchom dowolne z poleceń tworzenia poniżej. GitPoulpi
zapamiętuje to dla danego repozytorium (zob. **Sekcje panelu bocznego** w
[Ustawieniach](settings.md)); inne repozytorium bez worktree nadal zaczyna z
ukrytą sekcją.

## Tworzenie worktree

Przycisk **+** w nagłówku WORKTREES (klawiatura: `+` w sekcji; paleta:
**Refs: Utwórz worktree...**) otwiera menu:

- **Z bieżącej gałęzi**: gałąź jest przełączona w głównym folderze, więc okno
  proponuje na niej nową gałąź (`<gałąź>-wt`).
- **Z zaznaczonego commita**: wyszarzone, dopóki w Logu nie zaznaczysz
  commita. Worktree zaczyna tam jako odłączone, chyba że podasz nową gałąź.
- **Wybierz gałąź...**: okno otwiera się na liście **Ref**.

Paleta ma te same trzy (**Refs: Utwórz worktree z bieżącej gałęzi**,
**Git: Utwórz worktree dla zaznaczonego commita...**, **Refs: Utwórz worktree z
wybranej gałęzi**). Starsze pozycje nadal działają: **Utwórz worktree z...** w
menu gałęzi, tagu lub commita.

Okno proponuje wszystko i pozwala to zmienić:

| Pole | Znaczenie |
|---|---|
| **Nazwa** | Od niej bierze nazwę folder. Proponowana z gałęzi lub krótkiego id. |
| **Folder** | Sąsiad repozytorium, `<repozytorium>-<nazwa>`. Podąża za polem Nazwa, dopóki nie wpiszesz czegoś w polu Folder; **Przeglądaj...** wybiera miejsce. |
| **Ref** | Gałąź lub commit, od którego zaczyna worktree (lista; pisz, by szukać). Zmiana proponuje nową Nazwę. |
| **Nowa gałąź** | Puste przełącza tam gałąź z pola Ref (lub odłącza HEAD na commicie). Wymagane, gdy ta gałąź jest już przełączona. |
| **Otwórz nowy worktree jako kartę** | Domyślnie włączone. |

Niepusty folder, niepoprawna nazwa lub gałąź, która już istnieje, są
zgłaszane w osobnym wierszu okna, a **Utwórz** pozostaje wyszarzone.

## Menu wiersza

Kliknij worktree prawym przyciskiem myszy (albo zaznacz go i naciśnij klawisz
menu, `Shift+F10` lub `.`):

- **Otwórz w nowej karcie**: jak dwukrotne kliknięcie lub `Enter`.
- **Otwórz w**: wpisy [Otwórz w](open-in.md), które mają zaznaczone pole
  **Może otwierać worktree** (Ustawienia > Otwórz w; domyślnie każdy wpis jest
  zaznaczony). Działają z folderem worktree jako `${cwd}`.
- **Merge <gałąź> do...**: wybierz gałąź, do której scalić (domyślnie gałąź
  głównego worktree). Zwykły merge działa w worktree, w którym ta gałąź jest
  przełączona, z twoimi opcjami merge; konflikt rozwiązuje się tam zwykłym
  przepływem konfliktów. Gałąź przełączona nigdzie jest odrzucana z podaniem
  powodu. Niedostępne dla odłączonego worktree.
- **Usuń worktree...**: nigdy głównego.

Paleta ma te same akcje na zaznaczonym wierszu (**Refs: Otwórz worktree w...**,
**Refs: Merge gałęzi worktree do...**, **Refs: Usuń worktree...**,
**Refs: Otwórz worktree w karcie**).

## Usuwanie worktree

Czysty worktree dostaje zwykłe potwierdzenie. Jeśli jego gałąź jest już scalona
(z jej upstreamem, gałęzią głównego worktree albo `main` / `master`), pole
**Skasuj też gałąź <b>** jest domyślnie zaznaczone; niescalona gałąź mówi, że
zostaje. **Merge do... i usuń** najpierw scala i usuwa dopiero, gdy merge
zakończył się bez konfliktów.

Przy niezatwierdzonych zmianach (śledzonych lub nieśledzonych) okno je wylicza
(liczba, z przewijaniem) i oferuje:

- **Commit i usuń**: pole opisu w oknie; każda zmiana trafia do poczekalni
  (z poszanowaniem `.gitignore`) i jest zatwierdzana na gałęzi worktree, po
  czym worktree jest usuwane. Odłączony worktree w tym samym oknie podaje
  gałąź dla commita.
- **Stash i usuń**: każda zmiana, także w plikach nieśledzonych, trafia do
  stasha z edytowalnym opisem (domyślnie „WIP w <folder>”), po czym worktree
  jest usuwane. Lista stashy jest wspólna dla wszystkich worktree: stash leży
  potem w STASHES głównego repozytorium, gotowy do pop.
- **Otwórz, by zrobić commit**: otwiera worktree w karcie z aktywnym polem
  opisu commita; nic nie jest usuwane.
- **Usuń mimo to**: odrzuca zmiany po drugim, wyraźnym potwierdzeniu.

## Cofanie

Usunięcia worktree nie można cofnąć: folder znika. To, co zapisano wcześniej,
zostaje: commit jest na swojej gałęzi, stash na liście. Skasowanie gałęzi,
które dodaje **Skasuj też gałąź**, jest zwykłym zapisanym w dzienniku:
**Cofnij** przywraca gałąź (nie folder). Utworzenie worktree też nie należy do
historii.

## Ze skryptu lub od agenta

`gitpoulpi worktree changes <path>` wylicza niezatwierdzone zmiany worktree.
`gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` je usuwa; narzędzia
MCP to `worktree_changes` i `worktree_remove`. Zob. [Opis CLI](cli-reference.md).
