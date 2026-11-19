[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · **Polski** · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Pierwsze kroki

Ta strona prowadzi od świeżej instalacji do pierwszego commita.
[Przewodnik po interfejsie](interface-tour.md) nazywa każdy obszar okna.
[Ustawienia](settings.md) wymieniają każdą opcję.

## 1. Instalacja i uruchomienie

Postępuj według [Instalacji](installation.md). Potem otwórz terminal w
repozytorium Git i uruchom:

```
gitpoulpi
```

GitPoulpi otwiera repozytorium
zawierające bieżący folder. Poza repozytorium otwiera kartę **Home**, na której
można przeglądać, klonować lub inicjować repozytorium (zob.
[krok 7](#7-otwieranie-klonowanie-i-inicjowanie-repozytoriów)).

Ikony wymagają Nerd Fonta albo czcionki z emoji. Jeśli widać prostokąty, zob.
[Rozwiązywanie problemów](troubleshooting.md).

![Główne okno: panel boczny Refs po lewej, graf Log pośrodku oraz zmiany robocze z formularzem commita po prawej](../images/hero.png)

## 2. Trzy klawisze, dzięki którym poznasz resztę

| Klawisz | Otwiera |
|---|---|
| `?` | **Pomoc**: każdy klawisz i gest myszy. Wpisz tekst, aby filtrować. `Ctrl+D` otwiera tę dokumentację w przeglądarce. |
| `Ctrl+P` | **Paletę poleceń**: wpisz, aby znaleźć polecenie, `Enter` je uruchamia. Każdy wiersz pokazuje swój skrót. |
| `Ctrl+,` | **Ustawienia**. Otwiera je także koło zębate na pasku tytułu, na lewo od ×. |

Wszystko można robić z klawiatury albo myszą. Paleta sięga każdego polecenia.
Zob. [Skróty klawiszowe](keybindings.md) i
[Mysz i gesty](mouse-and-gestures.md).

![Paleta poleceń przefiltrowana słowem stage, z poleceniami stage i ich skrótami](../images/palette.png)

## 3. Przeglądanie historii

Środek okna to **Log**: jeden wiersz na commit, od najnowszego. Pokazuje
gałęzie i tagi jako kolorowe plakietki, graf, opis, autora, datę i krótki hash.

- Kliknij commit. Jego szczegóły i zmienione pliki pojawią się po prawej.
- Kliknij zmieniony plik. Jego diff otworzy się pośrodku. `Esc` go zamyka.
- `j` / `k` lub strzałki przesuwają zaznaczenie. `f` przeszukuje commity.
- Kliknij commit prawym przyciskiem, aby zobaczyć jego menu.

![Log z zaznaczonym commitem merge: jego opis, autor i dwa pliki, które dodaje, po prawej](../images/log-graph.png)

## 4. Zmiana i stage

Edytuj plik w swoim edytorze. **Górny wiersz Logu** pokazuje teraz zmiany
robocze: przerywany wiersz z liczbą zmienionych plików. Kliknij go. Prawy panel
pokazuje **Pliki unstaged** i **Pliki staged**.

- Najedź na plik i kliknij jego przycisk **Stage plik**. Albo kliknij
  **Stage wszystkie zmiany**.
- Kliknij nazwę pliku, aby otworzyć jego diff. Tam można zrobić stage jednego
  **hunka** (**Stage hunk**) albo jednej **linii**.

Zob. [Stage i commity](staging-and-committing.md).

![Diff zmodyfikowanego pliku, z przyciskami Stage hunk i Odrzuć hunk nad każdym hunkiem](../images/diff.png)

## 5. Commit

Wpisz podsumowanie i opcjonalnie opis w formularzu commita w prawym dolnym
rogu. Potem kliknij zielony przycisk **Commit zmian w N plikach** albo naciśnij
`Ctrl+Enter` (albo `Ctrl+G Enter`, z dowolnego miejsca). Zaznacz **Popraw poprzedni commit**, aby dodać zmiany do ostatniego
commita zamiast tworzyć nowy.

Jeśli włączono [AI](ai.md), przycisk `✦` obok podsumowania pisze opis na
podstawie zmian w stage. GitPoulpi nigdy nie robi commita, dopóki nie klikniesz
przycisku **Commit zmian w N plikach**.

## 6. Udostępnianie pracy

Pasek narzędzi ma przyciski **Pull** i **Push**, każdy z listą `▾`. Z klawiatury
naciśnij `Ctrl+G`, a potem literę: `Ctrl+G p` robi pull, `Ctrl+G Shift+P` robi
push. Po `Ctrl+G` pasek stanu wymienia litery.

GitPoulpi używa poświadczeń, których już używa twój `git` (menedżer poświadczeń,
ssh-agent). Nie potrzebuje własnego konta. Aby widzieć i tworzyć pull
requesty, połącz raz konto hostingu. Zob.
[Hosting i trackery zgłoszeń](hosting-integration.md).

## 7. Otwieranie, klonowanie i inicjowanie repozytoriów

Kliknij ikonę domu na lewym końcu rzędu kart albo naciśnij `Ctrl+O`. Karta
**Home** wymienia otwarte, ulubione i ostatnie repozytoria, z przyciskami
**Przeglądaj**, **Klonuj** i **Init**. Każde repozytorium otwiera się we własnej
karcie. `Ctrl+W` zamyka bieżącą kartę.

![Karta Home z listą otwartych i ostatnich repozytoriów oraz przyciskami Przeglądaj, Klonuj i Init](../images/home.png)

**Klonuj** pyta o URL i folder albo pozwala wybrać repozytorium z połączonego konta (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; zob. [Klonowanie z połączonego konta](hosting-integration.md#klonowanie-z-połączonego-konta)). **Init** zamienia folder w nowe repozytorium.
W tych trzech oknach (i w oknie wyboru folderu, które otwierają) `Ctrl+N` pokazuje plakietkę na każdym polu i przycisku: naciśnij znak, by ustawić kursor w polu albo nacisnąć przycisk.

| ![Okno Klonuj: miejsce docelowe, URL, nazwa folderu oraz przyciski Anuluj i Klonuj](../images/clone.png) | ![Okno inicjowania repozytorium: folder, nazwa i gałąź początkowa](../images/init.png) |
|---|---|

### Częściowy checkout

Duże repozytorium nie musi trafić na dysk w całości. W oknie Klonuj zaznacz **Częściowy checkout (tylko niektóre katalogi)** i wpisz potrzebne katalogi, oddzielone spacjami lub przecinkami (`src docs/guide`). Checkout obejmuje tylko te katalogi i pliki w katalogu głównym repozytorium; cała historia i tak jest pobierana, więc Log, blame i gałęzie działają jak zwykle. Dopóki pole jest zaznaczone, wymagany jest co najmniej jeden katalog.

![Okno Częściowy checkout z dwoma wpisanymi katalogami i przyciskami Anuluj, Wyłącz i Zastosuj](../images/sparse-checkout.png)

Dowolne repozytorium można przełączyć później: otwórz paletę poleceń (`Ctrl+P`) i uruchom **Repozytorium: Częściowy checkout...** albo użyj **Częściowy checkout...** w menu karty repozytorium. W jednym polu okno wymienia katalogi, które są teraz w checkoucie: dodaj lub usuń nazwy i naciśnij **Zastosuj** (`Enter`), albo **Wyłącz (pełny checkout)**, by przywrócić każdy plik. Dopóki drzewo robocze jest częściowe, pasek stanu pokazuje małą plakietkę `częściowy`. Te same elementy są stroną *Ustawienia > Częściowy checkout* (lista ścieżek, po jednej w wierszu, z przyciskami **Rozpocznij**, **Zakończ** i **Odśwież**).

Uwagi:

- To tryb cone gita (katalogi, nie wzorce); `*`, `..` i ścieżki bezwzględne są odrzucane.
- Zmiana zestawu nie jest zapisywana przez Cofnij: nic nie ginie, a **Wyłącz** przywraca każdy plik. Git może odmówić porzucenia katalogu z niezatwierdzonymi zmianami (błąd pojawia się na pasku stanu); najpierw zatwierdź je lub schowaj.
- To samo ze skryptów i agentów: `gitpoulpi clone <url> --sparse src docs` oraz `gitpoulpi sparse-checkout status | set | add | disable` (zob. [Opis CLI](cli-reference.md)); serwer MCP ma te same narzędzia.

## Co dalej

- [Przewodnik po interfejsie](interface-tour.md): nazwa wszystkiego, co widać.
- [Praca z Logiem](log-and-history.md) i
  [Stage i commity](staging-and-committing.md): codzienna praca.
- [Motywy](themes.md) i [Ustawienia](settings.md).
- [Wsparcie AI](ai.md) oraz [Hosting i trackery zgłoszeń](hosting-integration.md):
  dwie opcjonalne integracje.
- [Opis CLI](cli-reference.md) i [MCP i agenci](mcp-and-agents.md):
  te same operacje ze skryptów i od agentów.
- [Rozwiązywanie problemów](troubleshooting.md).
