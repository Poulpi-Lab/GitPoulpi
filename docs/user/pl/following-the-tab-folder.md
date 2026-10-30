[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · **Polski** · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Podążanie za folderem aktywnej karty

GitPoulpi może mieć otwartych kilka repozytoriów jako karty. Powłoka, z której go
uruchomiono, zostaje w swoim folderze: program nie może zmienić folderu
powłoki, która go uruchomiła. GitPoulpi oferuje dwa sposoby, by dostać się
do powłoki w folderze aktywnej karty:

- **Zakończ i cd tutaj**: GitPoulpi kończy działanie, a twoja powłoka przechodzi
  do folderu aktywnej karty. Wymaga to małej funkcji powłoki, wypisywanej przez
  `gitpoulpi shell-init`.
- **Otwórz w**: otwiera folder aktywnej karty w terminalu, panelu
  multiplekserze, edytorze lub menedżerze plików. Nie wymaga konfiguracji
  powłoki. Zob. [Otwórz w](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Otwórz w ▾ ─ ⌕ Szukaj wszędzie ⚙ × ┐
│ ⌂  main-repo x  other-repo x               Zakończ i cd tutaj           │
```

- **Otwórz w** jest na pasku tytułu, za ścieżką repozytorium. Jest ukryte, gdy
  okno jest zbyt wąskie.
- **Zakończ i cd tutaj** jest na prawym końcu paska kart. Pokazuje się tylko,
  gdy spełnione są oba warunki:
  - GitPoulpi uruchomiono przez funkcję powłoki (poniżej).
  - Aktywna karta nie jest folderem, w którym uruchomiono GitPoulpi.
- Polecenia są zawsze w palecie (`Ctrl+P`): `Karta: Zakończ i cd tutaj`,
  `Karta: Otwórz w...` (`Alt+O`) i `Karta: Otwórz w domyślnym` (`Ctrl+Alt+O`).
  Bez funkcji powłoki `Karta: Zakończ i cd tutaj` podpowiada, jak ją
  skonfigurować.

Zwykłe zakończenie (`q`) nigdy nie zmienia folderu twojej powłoki. Robi to
tylko **Zakończ i cd tutaj**, chyba że wybierzesz `auto` (poniżej).

## Konfiguracja funkcji powłoki

`gitpoulpi shell-init <shell>` wypisuje funkcję o nazwie `gp`. Opcja `--name`
pozwala wybrać inną nazwę. Funkcja uruchamia `gitpoulpi --cwd-file <temp file> ...`.
Gdy zakończysz przez **Zakończ i cd tutaj**, GitPoulpi zapisuje folder do tego
pliku, a funkcja przechodzi do tego folderu. GitPoulpi niczego nie instaluje
ani nie edytuje: dodajesz jedną linię do pliku startowego swojej powłoki.

| Powłoka | Dodaj do | Wiersz |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | twój `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Potem uruchamiaj GitPoulpi przez `gp` zamiast `gitpoulpi`. Argumenty są
przekazywane dalej (`gp -s work`), a kod wyjścia to kod wyjścia `gitpoulpi`.

**Istniejące `gp` nigdy nie jest zastępowane bez ostrzeżenia.** Jeśli twoja
powłoka ma już polecenie, alias lub funkcję o nazwie `gp`, skrypt wypisuje
ostrzeżenie i niczego nie definiuje. Wybierz inną nazwę, na przykład
`gitpoulpi shell-init bash --name gpl`. W PowerShellu funkcja zastępuje
wbudowany alias `gp` (`Get-ItemProperty`) oraz alias wskazujący na
`gitpoulpi`. Każde inne `gp` zostaje, z ostrzeżeniem.

Każde polecenie `gitpoulpi` przyjmuje `--cwd-file`, ale plik zapisuje tylko
TUI.

## Ustawienie: Podążaj za folderem aktywnej karty

Ustawienia > Otwórz w > **Podążaj za folderem aktywnej karty** (`ui.tab_dir_follow`):

| Wartość | Zachowanie |
|---|---|
| `buttons` (domyślnie) | **Zakończ i cd tutaj** i **Otwórz w**, jak opisano wyżej. |
| `auto` | Bez przycisku **Zakończ i cd tutaj**. **Otwórz w** zostaje. Każde zakończenie przenosi powłokę do folderu aktywnej karty (gdy GitPoulpi uruchomiono przez funkcję powłoki). Domyślny wpis **Otwórz w** uruchamia się przy każdym otwarciu nowej karty. |

W trybie `auto` domyślny wpis uruchamia się tylko dla kart otwieranych ręcznie: z karty Home, z worktree, po klonowaniu lub init. Nie uruchamia się dla
kart przywróconych przy starcie, dla pierwszej karty ani przy przełączeniu na
kartę, która jest już otwarta.

Starsze ustawienia `ui.terminal_launch` i `ui.terminal_launch_command` zostały
przeniesione do listy **Otwórz w**. Zob.
[Otwórz w](open-in.md#starsze-klucze).
