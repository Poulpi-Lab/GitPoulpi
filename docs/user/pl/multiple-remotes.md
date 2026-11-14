[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · **Polski** · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Kilka zdalnych dla jednej gałęzi

Wiele repozytoriów mieszka na więcej niż jednym hoście: głównym (`origin`), lustrze na innym hoście, kopii zapasowej. GitPoulpi pozwala gałęzi wypchnąć się do wszystkich jednym kliknięciem i opcjonalnie ściągać z więcej niż jednego, bez zmiany tego, co robi zwykły git. Nic tu nie wymaga konta, a wszystko jest zapisane w `.git/config` samego repozytorium, więc zwykły git i inne narzędzia nadal działają.

## Cele push i źródła pull

- **Upstream** to własne pojęcie gita (`branch.<name>.remote` i `merge`). To *główne* zdalne gałęzi. GitPoulpi nigdy nie zmienia sposobu jego działania.
- **Cel push** to dodatkowe zdalne, do którego gałąź też się wypycha. Przycisk Push wysyła gałąź do upstreamu i do każdego celu push.
- **Źródło pull** to dodatkowe zdalne, z którego pull też bierze commity. Jest wyłączone, dopóki go nie włączysz (pole **Pull też**): scalanie dwóch zdalnych, których nie wybrałeś, nie jest domyślnie niczym, czego ktokolwiek chce.

Cel może używać innej nazwy gałęzi na tym zdalnym (`backup` dostaje `feat-x`).

## Okno Zdalne gałęzi

Kliknij gałąź prawym przyciskiem i wybierz **Zdalne gałęzi <branch>...** (jest też w rozwijanym menu Push, w palecie jako `Git: Zdalne gałęzi...` i po kliknięciu plakietki `⇉3` w wierszu gałęzi w sekcji Refs). Okno wylicza każde zdalne repozytorium:

| Kolumna | Znaczenie |
|---|---|
| Zdalne, Gałąź | Zdalne i nazwa gałęzi tam. **Zmień nazwę na zdalnym...** zmienia tę nazwę dla zaznaczonego wiersza. |
| Push | Zaznacz, by zdalne stało się celem push tej gałęzi. |
| Pull też | Zaznacz, by pull integrował także to zdalne. Upstream jest zawsze ściągany. |
| Stan | Jak gałąź stoi względem niego: zgodne, do przodu, w tyle, nowa, niepowiązane. |

Wiersz oznaczony „zdalne usunięte” dotyczy zdalnego, które gałąź wciąż wymienia, a którego repozytorium już nie ma; **Usuń** o nim zapomina. To samo okno ustawia upstream (lista **Upstream**), a **Usuń upstream** go czyści. **Zastosuj** zapisuje wszystko jako jedną zmianę, którą można cofnąć.

Repozytorium może też nadać każdej nowej gałęzi te same dodatkowe cele: **Ustawienia > Git i kodowanie** (strona dla repozytorium), „Dodatkowe cele push dla nowych gałęzi” (pole wyboru na każde zdalne). Są kopiowane do gałęzi, gdy po raz pierwszy dostaje upstream; istniejących gałęzi nie dotykają.

## Push

- **Przycisk Push** na gałęzi z dodatkowymi celami: wypycha do wszystkich. Gdy każde zdalne po prostu przyjmuje nowe commity, działa od razu, a widok postępu pokazuje, jak każde zdalne zmienia się w ptaszek. Jeśli coś wymaga decyzji (zdalne ma commity, których ty nie masz), otwiera się zamiast tego okno planu.
- **Push ▾** (strzałka obok przycisku; także sam przycisk, gdy **Przycisk Push** w Ustawieniach to **pytaj**):
  - `Push do wszystkich (origin, gitlab, backup)`
  - `Push tylko do <remote>`, po jednym na każde zdalne gałęzi
  - `Push do wybranych zdalnych...`: jest zawsze, dla każdej gałęzi, także z jednym zdalnym
  - `Zdalne gałęzi <branch>...`
- Menu prawego przycisku gałęzi i paleta (`Git: Push do wszystkich celów`, `Git: Push do zdalnego...`, `Git: Push do wybranych zdalnych...`) oferują te same akcje pod tymi samymi nazwami.

**Push do wybranych zdalnych...** otwiera listę każdego zdalnego repozytorium, z zaznaczonym upstreamem i celami gałęzi. Dotyczy tylko tego pusha; zaznacz **Zapamiętaj dla tej gałęzi**, by zapisać wybór jako cele push gałęzi.

### Plan

Przed grupowym pushem GitPoulpi sprawdza każde zdalne (szybkie zapytanie, a pobranie z danego zdalnego tylko wtedy, gdy się różni), więc problem na trzecim zdalnym widać, zanim ruszy pierwszy push. Okno wylicza każde zdalne z informacją, jak gałąź stoi i co się z nim stanie:

| Sytuacja | Domyślna akcja | Dostępne także |
|---|---|---|
| Zdalne nie ma nic z gałęzi albo brakuje mu części commitów | Push | Pomiń |
| Zdalne jest do przodu (nie masz dla niego nic nowego) | Fast-forward z niego | Pomiń |
| Zdalne się różni, bo zrobiłeś rebase lub amend tego, co wypchnąłeś | Wymuś push (lease) | Merge, Rebase, Pomiń |
| Zdalne ma commity kogoś innego | Merge z niego (Rebase, gdy **Przycisk Pull** to rebase) | Rebase, Wymuś push (lease), Pomiń |
| Historia nie ma nic wspólnego z twoją | Pomiń | Wymuś push (lease) |
| Host odrzucił gałąź (chroniona) | Pomiń | brak: wymuszenie też zostałoby odrzucone |

Akcja każdego wiersza to lista, którą możesz zmienić (Left/Right przechodzi po niej, Enter ją otwiera, `s` pomija wiersz). Merge pokazuje, czy dałby konflikt i w których plikach. **Force push** jest zawsze potwierdzany, podaje zdalne i ile commitów zastępuje, zawsze używa lease (nigdy gołego force) i nie można go cofnąć: wynik podaje, który commit został zastąpiony, i polecenie, które go przywraca. **Pomiń problemy** ustawia każdy kłopotliwy wiersz na Pomiń. **Uruchom** (Ctrl+Enter) startuje.

Najpierw, w kolejności listy, działają merge, fast-forward i rebase, więc twoja gałąź dochodzi do stanu końcowego raz; potem pushe wychodzą po jednym zdalnym naraz, upstream pierwszy. Nie ma atomowego „wszystko albo nic” między zdalnymi: każde kończy się własnym wynikiem, a błąd na jednym (sieć, hook) nie zatrzymuje pozostałych. **Ponów nieudane** (albo `Git: Ponów nieudane pushe`) próbuje je jeszcze raz. Zamknięcie widoku postępu nigdy nie zatrzymuje przebiegu; `Git: Pokaż grupowy push` go przywraca.

### Konflikty i Kontynuuj

Jeśli merge lub rebase przebiegu zatrzyma się na konfliktach, okno się zamyka, a przejmują zwykły baner merge i edytor konfliktów. Pasek nad Log głosi `Grupowy push wstrzymany (gitlab): rozwiąż i zakończ, potem Kontynuuj.` i ma przyciski **Kontynuuj** oraz **Przerwij grupę**.

- Rozwiąż konflikty i zakończ merge lub rebase (w GitPoulpi albo zwykłym gitem).
- **Kontynuuj** jest wyszarzone, dopóki tego nie zrobisz, i **nigdy nie jest wciskane za ciebie**: nic nie rusza samo. Naciśnij je (albo uruchom `Git: Kontynuuj grupowy push`), a przebieg znów wypchnie do każdego zdalnego od twojego nowego czubka. Po rebase zdalne, które wciąż trzyma stare commity, wraca do okna planu (teraz wymaga force push), zamiast być po cichu wymuszone.
- **Przerwij grupę** (z potwierdzeniem; `Git: Przerwij grupowy push`) odrzuca merge lub rebase rozpoczęty przez przebieg i zapomina o przebiegu. To, co już wypchnięto, zostaje.

Pasek przeżywa restart GitPoulpi, a CLI widzi ten sam zapisany przebieg.

## Pull

- Przycisk **Pull** ściąga upstream dokładnie jak dotąd, chyba że gałąź ma źródła pull: wtedy ściąga upstream i każde źródło, po kolei, każde w trybie ustawionym przez **Przycisk Pull** w Ustawieniach (merge, tylko fast-forward lub rebase). Źródło, którego nie da się przesunąć fast-forwardem, jest scalane (lub rebasowane) jak upstream; przy samym fast-forward źródło, które potrzebuje więcej, najpierw pyta.
- **Pull ▾** dodaje `Pull ze wszystkich źródeł (origin + gitlab)`, gdy są źródła, oraz `Pull tylko z <remote>` dla każdego zdalnego gałęzi: jednorazowy pull gałęzi z tego zdalnego, niezależnie od pól wyboru.
- Paleta ma `Git: Pull ze wszystkich źródeł` i `Git: Pull z jednego zdalnego...`.
- Pull, który zatrzyma się na konfliktach, zostawia ten sam pasek, opisany jako grupowy pull.

## Ustawienia

Wszystkie cztery są w **Ustawienia > Git i kodowanie**.

| Ustawienie | Domyślnie | Znaczenie |
|---|---|---|
| **Przycisk Push** | od razu | od razu wypycha natychmiast; pytaj najpierw otwiera menu Push. |
| **Przycisk Pull** | merge | Tryb przycisku Pull (merge, tylko fast-forward lub rebase) oraz źródeł pull. |
| **Sprawdź każde zdalne przed pushem** | włączone | Sprawdza każde zdalne przed grupowym pushem (CLI za tym idzie; `--no-preflight` wyłącza to dla jednego wywołania). |
| **Dodatkowe cele push dla nowych gałęzi** (na repozytorium) | brak | Zdalne, do których nowa gałąź też się wypycha. |

## Z wiersza poleceń

```
gitpoulpi branch upstream add gitlab --pull      # cel push, który jest też źródłem pull
gitpoulpi branch upstream list                    # do czego gałąź jest ustawiona
gitpoulpi push --all-targets                      # upstream plus każdy cel
gitpoulpi push --to gitlab --to backup:feat-x     # dokładnie te, tym razem
gitpoulpi pull --all-sources                      # upstream plus każde źródło pull
gitpoulpi remote-group plan                       # co zrobiłby grupowy push, nic się nie zmienia
gitpoulpi remote-group continue                   # po rozwiązaniu i zakończeniu zatrzymania
```

Gdy zdalne wymaga decyzji, polecenie odpowiada planem (kod 3) i niczego nie zmienia; odpowiedz na to przez `--resolve gitlab=merge`. Wszystkie flagi opisuje [dokumentacja CLI](cli-reference.md). Serwer MCP ma te same narzędzia.

## Ze zwykłym gitem

Cele to zwykłe klucze konfiguracji, więc skrypt lub alias może ich używać bez GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # jedna wartość w wierszu: remote lub remote:branch

# push bieżącej gałęzi do jej upstreamu i do każdego celu
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Wartość to `<remote>` (ta sama nazwa gałęzi) lub `<remote>:<branch>`, co rozstrzyga wiersz `d=${t#*:}`. Zwykły alias nie sprawdza najpierw zdalnych, nie proponuje merge i nie umie kontynuować po konflikcie: od tego jest okno.

## Warto wiedzieć

- Dwa zdalne wskazujące ten sam adres są zauważane („gitlab i mirror mają ten sam URL push”); drugie jest domyślnie pomijane.
- Host z chronioną gałęzią odrzuca force push; wiersz to mówi i oferuje tylko Pomiń.
