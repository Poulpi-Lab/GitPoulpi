[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · **Polski** · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Rozwiązywanie problemów i FAQ

Zacznij od krótkich odpowiedzi poniżej; dłuższe sekcje dotyczą Git LFS i
renderowania grafu. Jeśli nic nie pomoże, zgłoś problem w repozytorium GitHub, podając
system operacyjny, terminal, wynik `gitpoulpi --version` i to, co zrobiono.

## Gdy coś się nie powiedzie

Nieudana operacja nie pokazuje ani okienka, ani ramki: mówi o tym **pasek stanu**, na czerwono, z tytułem i powodem (*błąd: Fetch nie powiódł się: Nie udało się połączyć ze zdalnym repozytorium.*), i zostaje tam, dopóki inny komunikat go nie zastąpi. Przycisk Aktywność na jego prawym końcu liczy błędy, których jeszcze nie obejrzałeś. Kliknij czerwony komunikat albo uruchom polecenie palety `Dziennik aktywności: Otwórz ostatni błąd`, by otworzyć Dziennik aktywności na tym wpisie. Tak działa każde niepowodzenie wywołane przez ciebie, nie tylko operacje git:

- **Operacje git**: fetch, push, pull, checkout, merge, rebase, commit, stash...
- **Klonowanie** (i tworzenie nowego repozytorium): „Klonowanie nie powiodło się”.
- **Wywołanie GitHuba lub GitLaba**: listy repozytoriów w selektorach i w oknie Klonuj, tworzenie pull requesta lub zgłoszenia, sprawdzanie tokenu lub klucza SSH na stronie Integracje: „Żądanie do usługi hostingowej nie powiodło się”.
- **Program zewnętrzny**: edytor, narzędzie diff lub merge, menedżer plików, przeglądarka, terminal lub program uruchamiający, edytor obrazów, przyciski *Testuj* w Ustawieniach: „Program zewnętrzny nie zadziałał”.
- **Żądanie AI** (opis commita, wyjaśnienie, propozycja rozwiązania konfliktu...): „Żądanie AI nie powiodło się”.
- **Plik, którego nie udało się zapisać**: Zapisz we wbudowanym edytorze, edytor merge, ustawienie, którego nie udało się zapisać: „Zapis nie powiódł się”.

Czerwony komunikat jest na pasku stanu w każdym widoku: na karcie repozytorium, na Start, w Radarze, w Obszarach roboczych, w Ustawieniach (w dolnym wierszu ekranu), pod Dziennikiem aktywności lub oknem. Tam, gdzie pasek stanu jest zasłonięty (Start, Radar, Obszary robocze, Ustawienia), komunikat jest wypisany nad jego dolnym wierszem, dopóki nie obejrzysz błędu. Podąża za tobą z karty na kartę. Operacja, która jeszcze trwa, gdy przełączasz karty, nie jest śledzona: otwórz Dziennik aktywności, by zobaczyć, jak się skończyła.

Listy sekcji PULL REQUESTS i ZGŁOSZENIA, których nie da się odświeżyć, zachowują zamiast tego własny wiersz w sekcji (odświeżają się same, komunikat co kilka sekund byłby tylko szumem). Okno, które nadal jest otwarte (Klonuj, Utwórz pull request...), też zachowuje własny jednowierszowy komunikat, więc można poprawić pole i spróbować ponownie.

Wpis w Dzienniku aktywności zawiera wszystko, co pokazywało dawne powiadomienie:

- **Tytuł i powód.** „Fetch nie powiódł się”, potem jedno zdanie mówiące zwykłymi słowami dlaczego, potem co zrobić dalej. Rozpoznawane są: zdalne repozytorium ma commity, których nie masz (*najpierw pull*), odrzucone poświadczenia (*sprawdź Ustawienia > Integracje lub swój klucz SSH*), zatrzymanie na konfliktach, lokalne zmiany, które zostałyby nadpisane (*zrób commit, stash lub odrzuć*), zdalne repozytorium nieosiągalne oraz inny proces git trzymający blokadę repozytorium. Cokolwiek innego pokazuje pierwszy wiersz od samego gita. Klonowanie lub wywołanie hosta, które kończy się tak samo (odrzucone poświadczenia, host nieosiągalny), jest opisane tak samo; inne niepowodzenia dostają jedno zdanie o tym, co się nie udało („Nie udało się sklonować repozytorium.”, „Nie udało się uruchomić programu.”).
- **Szczegóły**, widoczne, gdy wpis jest otwarty (`Enter` albo kliknięcie, które go otworzyło): operacja, uruchomione polecenie git, jego kod wyjścia i ostatnie wiersze tego, co wypisało (dla wbudowanej biblioteki git jej klasa błędu i kod). Dane logowania, tokeny i nagłówki `Authorization` są maskowane, zanim cokolwiek zostanie pokazane, zapisane w dzienniku lub skopiowane.
- **Kopiuj** (przycisk albo `c`) umieszcza w schowku blok tekstu przygotowany do wklejenia w rozmowie z kimś, lub z AI, kto potrafi go wyjaśnić: wersja GitPoulpi, system, terminal, backend, nieudana operacja z jej szczegółami i wersja gita.
- **Przycisk następnego kroku**, gdy jest, obok Kopiuj (albo `r`): *Najpierw Pull* (push, który zdalne repozytorium by odrzuciło), *Stash i ponów* (lokalne zmiany przeszkadzają w checkoucie, merge, rebase lub pull), *Ponów* (zdalne repozytorium było nieosiągalne), *Otwórz Integracje* (poświadczenia). Wykonuje krok i zamyka okno. Dla niepowodzenia w innym repozytorium niż karta na wierzchu oferowane jest tylko *Otwórz Integracje*: ponowienie uruchomiłoby się w złym repozytorium.
- Bez myszy paleta poleceń ma `Dziennik aktywności: Otwórz ostatni błąd`, `Dziennik aktywności: Kopiuj ostatni błąd` i `Dziennik aktywności: Wykonaj sugerowaną akcję` (przycisk następnego kroku we wpisie, dla błędu, który pokazuje pasek stanu).

Push odrzucony przez zdalne repozytorium, bo to poszło naprzód, zachowuje własne okno (najpierw pull, force push z lease, anuluj); fakty i tak są w Dzienniku aktywności.

### Dziennik aktywności

Przycisk **Aktywność** na prawym końcu paska stanu (liczba na nim zlicza błędy, których jeszcze nie obejrzano), `Ctrl+N`, potem `Shift+A` oraz `Widok: Dziennik aktywności` w palecie otwierają okno. `Ctrl+N` pokazuje plakietkę na każdej karcie, polu, polu wyboru i przycisku okna (oraz na pierwszym wpisie listy); naciśnięcie znaku robi to samo co kliknięcie. Dwie karty, **Operacje git** i **Aplikacja**, przełączane przez `Tab`, kliknięcie, `g` (Operacje git) lub `p` (Aplikacja), oraz pole **Zakres** pod nimi, które dotyczy obu (zob. niżej):

- **Aplikacja**: własne wiersze dziennika programu, okna i demona w tle, od najnowszego. Pole **Poziom** (lista rozwijana: Błąd, Ostrzeżenie, Info, Debugowanie, Śledzenie; `l` ją otwiera) mówi, ile pokazać; żądanie większej szczegółowości, niż jest zapisywane, podnosi też poziom zapisu (i zapisuje go jako `log.level`). Pod nim **Pokaż** ma jedno pole wyboru na poziom (Błąd, Ostrzeżenie, Info, Debugowanie, Śledzenie, każde z liczbą wierszy, które zawiera), by ukryć lub pokazać wiersze tego poziomu, na przykład by zobaczyć tylko błędy, a **Wszystko** / **Nic** zaznaczają lub odznaczają wszystkie pola. Klawisze `1` do `5` przełączają poziomy w tej kolejności, `a` to Wszystko, `n` to Nic; paleta ma `Dziennik aktywności: pokaż lub ukryj poziom ...`, `Dziennik aktywności: pokaż wszystkie poziomy` i `Dziennik aktywności: nie pokazuj żadnego poziomu`. Poziomy bardziej szczegółowe niż wybór w polu Poziom są wyszarzone (nic się dla nich nie zapisuje; najedź na jeden, by zobaczyć dlaczego). Znaczniki są pamiętane do zamknięcia programu i ma je tylko karta Aplikacja. **Filtr** (`/`) zostawia wiersze zawierające Twój tekst. **Kopiuj wszystko (przefiltrowane)** kopiuje to, co widać (znaczniki i filtr działają razem); **Otwórz folder dzienników** otwiera folder w menedżerze plików. Długi wiersz nigdy nie jest ucinany: zob. „Długie wiersze” niżej.

  ![Dziennik aktywności, karta Aplikacja: pole Poziom, pola Pokaż dla poszczególnych poziomów z przyciskami Wszystko i Nic, filtr i wiersze dziennika](../images/activity-log.png)

**Wyczyść dziennik** (`x` lub polecenie palety `Dziennik aktywności: Wyczyść dziennik aplikacji`) jest na karcie **Aplikacja**: najpierw pyta, potem opróżnia pliki dziennika ze wszystkich przechowywanych dni i listę na ekranie. Nie można tego cofnąć i nie trafia do historii cofania.

- **Operacje git** (karta, na której otwiera się okno): operacje, od najnowszej, każda z godziną uruchomienia i czasem trwania (`Push - 123 ms`): otwarcie repozytorium, wczytanie grafu commitów, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Niepowodzenia, komunikaty stanu, nieudane klonowanie i Fetch all obszaru roboczego też tu są; `Enter` lub kliknięcie otwiera wpis (jego powód, następny krok, polecenie, kod wyjścia i wyjście), `c` lub jego przycisk Kopiuj go kopiuje, `r` lub jego przycisk wykonuje sugerowany krok. `Tab` (lub `p`) przełącza na kartę Aplikacja; `Left`/`Right` przewijają długie wiersze na boki.

  ![Dziennik aktywności, karta Operacje git: każda operacja z godziną i czasem trwania, nieudany fetch na czerwono](../images/activity-log-repository.png)

- **Która akcja wywołała błąd.** Każde niepowodzenie mówi, co je wywołało: we wpisie, w dzienniku Aplikacja i w skopiowanym raporcie. Operacja jest nazwana tak, jak nazwałby ją człowiek (`Remove remote`, `Fetch all remotes`, `Hosting request: create pull request`, `Token command`), z jej głównymi argumentami (`remote=cm6`, gałąź, ścieżka; nigdy treść komunikatu, a adresy URL i tokeny są maskowane jak wszędzie), potem repozytorium, kto o nią poprosił (`TUI`, `CLI` lub `MCP`) i identyfikator operacji, czyli numer nadany żądaniu przez demona (oraz numer zadania dla fetch, push lub pull). Na karcie Operacje git nagłówek wpisu wygląda tak: `Remove remote: niepowodzenie (remote=cm6) - ... [CLI #12]`, a jego szczegóły zaczynają się od `Operacja:`, `Repozytorium:`, `Zażądał:` i `Identyfikator operacji:`; operacja bez własnego tytułu dostaje tytuł od tego, czym była (`Remove remote: niepowodzenie`). Na karcie Aplikacja wiersz niepowodzenia zaczyna się tymi samymi słowami i znacznikiem: `Remove remote failed (remote=cm6) [op #12, CLI, repo name]: backend error: no such remote: cm6`. Taki wiersz otwiera swoją operację: kliknij go albo zaznacz i naciśnij `Enter` (polecenie palety `Dziennik aktywności: Pokaż operację tego wiersza`). `]` i `[` zaznaczają następny i poprzedni wiersz niepowodzenia (`Dziennik aktywności: Następna nieudana operacja` / `Dziennik aktywności: Poprzednia nieudana operacja`); przy zaznaczonym wierszu strzałki przesuwają zaznaczenie zamiast przewijać. Otwierany wpis jest własnym wpisem okna, gdy operację wykonało okno; dla niepowodzenia CLI lub agenta w tej samej sesji okno tworzy wpis z wiersza. Skopiowany raport błędu ma wiersze `operation:`, `arguments:`, `repository:`, `requested by:` i `operation id:`, więc można go wkleić samodzielnie. W skrypcie błąd `--json` nieudanego polecenia się nie zmienia; odpowiedź demona na nieudane żądanie niesie te same fakty w dodanym polu `context`.

- **Długie wiersze** (obie karty): nagłówek lub wiersz szczegółów szerszy niż okno nigdy nie jest ucinany. Pod listą pojawia się poziomy pasek przewijania; przewijaj na boki przez `Left` / `Right` (do zmiany jako `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + kółko lub poziome kółko, polecenia palety `Dziennik aktywności: Przewiń w lewo` / `Dziennik aktywności: Przewiń w prawo` albo pasek (przeciągnij suwak albo naciśnij tor, by suwak znalazł się pod wskaźnikiem). `Home` wraca na początek, a przełączenie karty zaczyna drugą kartę przy lewej krawędzi. Zawinięty tekst (podsumowanie, sugerowana akcja) pozostaje zawinięty.

- **Zakres** (lista rozwijana pod kartami, otwiera ją `s` albo polecenie palety `Dziennik aktywności: Wybierz zakres`): czego dotyczą obie karty. Wiersze to **To repozytorium (nazwa)** (repozytorium karty, z której otworzono dziennik; nie jest oferowane ze Startu, który go nie ma), **Wszystkie repozytoria**, każdy **obszar roboczy** i każde repozytorium, które zna okno. Z karty repozytorium dziennik otwiera się na *Tym repozytorium*, ze Startu (przycisk Aktywność, `Ctrl+N` a potem `Shift+A`) na *Wszystkich repozytoriach*; `Ctrl+N` a potem `Shift+G` i polecenie palety `Widok: Globalny dziennik aktywności` zawsze otwierają się na *Wszystkich repozytoriach*, tak samo jak kliknięcie czerwonego błędu, którego repozytorium nie jest kartą na wierzchu (na przykład nieudane klonowanie). Twój ostatni wybór jest pamiętany przez sesję, osobno dla tych dwóch sposobów otwarcia. Na karcie Aplikacja wiersz należy do repozytorium, gdy program zapisał go w jego sprawie (nieudany push, czas operacji); w *Wszystkich repozytoriach* każdy taki wiersz mówi, czyj jest (`[nazwa]`). Wiersz bez żadnego repozytorium (start, ustawienia, sam demon) i wpis, który do żadnego nie należy (nieudany zapis), są **w całej aplikacji**: widać je w każdym zakresie, a gdy zakres jest węższy niż *Wszystkie repozytoria*, są oznaczone `[w całej aplikacji]`, więc filtr nigdy nie ukrywa niepowodzenia. Paleta ma też `Dziennik aktywności: Operacje git` i `Dziennik aktywności: Dziennik aplikacji` dla kart.

- **Kopiuj diagnostykę** (przycisk na obu kartach, `d`, zawsze cały dziennik, niezależnie od zakresu; oraz `Widok: Kopiuj diagnostykę` w palecie) kopiuje wersję, system, terminal i backend, ostatnie niepowodzenie ze szczegółami i ostatnie 200 wierszy dziennika na wybranym poziomie: blok do przekazania AI, gdy coś jest nie tak.

Czasy mierzy okno (od żądania do odpowiedzi demona, wraz z drogą tam i z powrotem), więc tego, co robi CLI lub MCP, nie mierzy się. `log.timings` (domyślnie włączone) wyłącza czasy, `log.extended` (domyślnie wyłączone) wypisuje też pracę w tle, okresowe odświeżanie i auto-fetch; oba są w Ustawienia > Ogólne obok pozostałych ustawień dziennika, a każda operacja i tak trafia do dziennika aplikacji (`info` dla tego, o co poproszono, `debug` dla tła).

Pliki dziennika nazywają się `gitpoulpi-YYYY-MM-DD.log`, są domyślnie przechowywane przez tydzień i najwyżej dziesięć lat (`log.retention_days`, od 1 do 3650), czyszczone przy uruchomieniu GitPoulpi i rotowane przy `log.max_file_mb` (5 MiB). Gdzie leżą: [Opis konfiguracji](config-reference.md#log-dziennik-aplikacji). Ze skryptu błąd `--json` nieudanego polecenia niesie te same fakty pod `detail`; zob. [Opis CLI](cli-reference.md).

Kliknięcie czerwonego komunikatu otwiera dziennik na *Wszystkich repozytoriach* z tym wpisem. Lista Operacje git dotyczy sesji; karta Aplikacja zachowuje dziennik programu między uruchomieniami.

**Zadania w tle.** Fetch, push, pull, klonowanie lub transfer LFS, Fetch all / Pull all / Clone missing obszaru roboczego, żądanie do AI oraz wysyłany pull request lub zgłoszenie są zadaniami: prawy kraniec paska stanu pokazuje trwające zadanie z przyciskiem `[x]`, który je anuluje, a kliknięcie etykiety (lub `Ctrl+N` a potem `Shift+J`, polecenie palety `Zadania: pokaż zadania w tle`) otwiera panel z przyciskiem Anuluj przy każdym zadaniu (także `c` lub `Del`). Klonowanie działa w tle; żądanie do AI i przejście po obszarze roboczym przetrwają zmianę karty i są stosowane, gdy wrócisz do ich repozytorium. Zadanie po punkcie, w którym zatrzymanie nie jest już bezpieczne, jest wyszarzone z powodem; anulowane zadanie zostawia repozytorium bez zmian.

## Krótkie odpowiedzi

**Pasek narzędzi i panel boczny pokazują kwadraty lub znaki zapytania zamiast ikon.**
Użyj w terminalu czcionki Nerd Font albo dowolnej czcionki z obsługą emoji. Graf i
tekst działają także bez nich.

**Graf commitów składa się ze znaków, a nie z obrazków.** To automatyczny
tryb zastępczy: obrazki wymagają terminala obsługującego protokół grafiki kitty (kitty,
Ghostty, herdr z włączoną grafiką kitty) albo protokół iTerm2 (WezTerm, Rio,
mintty) albo sixel (Windows Terminal, foot, Tabby, Konsole oraz tmux, gdy terminal za nim obsługuje sixel). Konsola Windows, zellij i screen dostają znaki. Jak wymusić jedno lub drugie, opisuje
sekcja o grafie poniżej, a także [Wybór terminala](terminals.md) i [Multipleksery](multiplexers.md).

**Mysz nic nie robi albo zaznaczanie tekstu myszą nie działa.**
GitPoulpi przechwytuje mysz, żeby kliknięcia do niego docierały. Aby zaznaczać tekst
przez sam terminal, przytrzymaj `Shift` podczas przeciągania (w większości terminali). Można też
zaznaczać wewnątrz diffa (przeciągnij, potem `y`). Aby wyłączyć przechwytywanie myszy, użyj
*Ustawienia > Klawiatura i mysz > Mysz włączona* (wymaga restartu). Klawiatura
dociera do każdego polecenia.

**Skrót nie działa.** Twój terminal, tmux lub skrypt AutoHotkey może
przejąć klawisz, zanim zobaczy go GitPoulpi (`Ctrl+,` często dociera jako zwykły `,` w
terminalach Linuksa i macOS; `Ctrl+Shift+N` wygląda jak `Ctrl+N`). Użyj
alternatyw (`Ctrl+N`, potem `,` otwiera Ustawienia; `Ctrl+P` dociera do każdego
polecenia) lub przypisz akord na nowo: zob. [Ustawienia](settings.md#klawiatura-i-mysz).

**`Ctrl+Tab` nie przełącza kart.** kitty, WezTerm, Ghostty i Windows Terminal
zachowują `Ctrl+Tab` i `Ctrl+Shift+Tab` dla własnych kart, więc ten klawisz
nigdy nie dociera do GitPoulpi. `Ctrl+PageDown` / `Ctrl+PageUp` działają zawsze.
Aby oddać tę parę GitPoulpi, zwolnij ją w konfiguracji terminala, a potem
uruchom terminal ponownie (albo przeładuj jego konfigurację):

- kitty, w `kitty.conf` (`map` bez akcji usuwa domyślne przypisanie; z
  `--when-focus-on` i kryterium okna, takim jak `title:...`, kitty może je
  usunąć tylko wtedy, gdy fokus ma pasujące okno, zob. dokumentację mapowania
  kitty):

  ```
  map ctrl+tab
  map ctrl+shift+tab
  ```

- WezTerm, w `wezterm.lua`:

  ```lua
  config.keys = {
    { key = 'Tab', mods = 'CTRL', action = wezterm.action.DisableDefaultAssignment },
    { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.DisableDefaultAssignment },
  }
  ```

- Ghostty, w jego pliku konfiguracyjnym:

  ```
  keybind = ctrl+tab=unbind
  keybind = ctrl+shift+tab=unbind
  ```

- Windows Terminal, w `settings.json` (`"id": null` usuwa przypisanie domyślne):

  ```json
  "keybindings": [
    { "id": null, "keys": "ctrl+tab" },
    { "id": null, "keys": "ctrl+shift+tab" }
  ]
  ```

  (Starsze pliki ustawień wymieniają `nextTab` / `prevTab` w `actions` z tymi
  klawiszami: usuń tam te klawisze.)

Terminal traci wtedy własne przełączanie kart przez `Ctrl+Tab`. Terminal, który
potrafi wysłać ten klawisz tylko przez protokół klawiatury kitty, wymaga jego
włączenia (domyślnie jest włączony, zob. [Skróty klawiszowe](keybindings.md#klawisze-których-terminal-nie-może-wysłać-i-protokół-klawiatury-kitty)).

**Jak znaleźć zapomniany skrót.** Naciśnij `Ctrl+P` i wpisz, czego chcesz; każdy
wynik pokazuje swój skrót. `?` wypisuje wszystko.

**Opcja jest wyszarzona w Ustawieniach.** Zależy od innej opcji. Wiersz
mówi, od której (na przykład *Intensywność podbarwienia* wymaga, by *Tło ścieżek grafu*
było ustawione na *Podbarwienie wiersza*).

**„Auto-stash oczekuje” nad Logiem.** GitPoulpi zrobił stash twoich zmian, by wykonać
operację, i nie mógł ich przywrócić. Użyj przycisków banera: **Pop teraz**, **Pokaż
stash**, **Zacznij od nowa** (cofa operację i przywraca twoje zmiany) lub **Drop**; zob. [Auto-stash](log-and-history.md#auto-stash).

**Agent zgłasza `no_viewer` lub `deferred`.** `no_viewer`: nie jest otwarte żadne okno GitPoulpi
dla tego repozytorium (albo *Pozwól agentom AI sterować widokiem* jest wyłączone).
`deferred`: okno pokazuje znacznik; `Ctrl+N Enter` go akceptuje. Zob. [MCP i
agenci](mcp-and-agents.md#pokazywanie-rzeczy-użytkownikowi).

**Po otwarciu GitPoulpi widać puste okno albo `log error: backend error`.**
Sprawdź, czy uruchomiono go wewnątrz repozytorium Git (w przeciwnym razie trafisz na kartę
Home: użyj **Przeglądaj**, **Klonuj** lub **Init**). Potem uruchom `gitpoulpi daemon status`;
`gitpoulpi daemon restart` restartuje proces w tle dla tego
repozytorium. Naciśnij `r`, by odświeżyć.

**Dwa okna na tym samym repozytorium zachowują się dziwnie.** Celowo dzielą jednego demona
na repozytorium, więc zmiana w jednym widać w drugim. Jeśli demon
wydaje się zawieszony, `gitpoulpi daemon stop --all` zatrzymuje demony wszystkich sesji; następne
okno uruchomi nowego.

**Commit, pull lub push prosi o hasło albo nie udaje się uwierzytelnić.**
GitPoulpi używa do sieci własnego menedżera poświadczeń i ssh-agenta twojego `git`.
Upewnij się najpierw, że `git fetch` działa w zwykłym terminalu. Tokeny GitHub/GitLab
dla pull requestów są osobne i opcjonalne: zob. [Hosting i trackery
zgłoszeń](hosting-integration.md).

**Fetch, pull lub push kończy się błędem „Skonfigurowany pomocnik poświadczeń zawiódł”, „Klucz hosta SSH zdalnego repozytorium nie jest jeszcze znany” albo odrzuconym logowaniem, a jego wpis w Dzienniku aktywności ma przycisk „SSH i poświadczenia...”.**
Błąd wynika z tego, jak to repozytorium się loguje. Przycisk otwiera
Ustawienia > SSH i poświadczenia, gdzie wybierasz inne źródło dla tego
repozytorium: własnych pomocników gita z ignorowanymi pomocnikami repozytorium,
konto integracji, plik klucza (z plikiem znanych hostów) albo własnego
pomocnika lub polecenie tokenu; **Testuj połączenie** sprawdza potem każde
zdalne repozytorium. Gdy pomocnik nie mógł się uruchomić, `git config
--show-origin --get-all credential.helper` pokazuje, gdzie jest ustawiony.

**„GitPoulpi nie wysłał tokenu konta do ...”.** Zdalne repozytorium wskazuje
serwer, który nie jest adresem, z którym związane jest wybrane konto (często
przepisanie `url.<x>.insteadOf` we własnej konfiguracji repozytorium), więc
token został wstrzymany. Napraw zdalne repozytorium lub wybierz inne źródło na
stronie.

**„Polecenie tokenu tego repozytorium nie powiodło się”, „... zmienił się od chwili, gdy potwierdzono”, „Logowania tego repozytorium nie można użyć”.**
Wyborów ze strony nie da się wykonać: polecenie tokenu nie wypisało tokenu
(jego ostatnie wiersze błędu są cytowane, z usuniętymi sekretami), skrypt,
który uruchamia, zmienił się od potwierdzenia (potwierdź polecenie ponownie na
stronie) albo wybór wskazuje plik klucza lub konto, których już nie ma. Każdy
z tych komunikatów ma przycisk **SSH i poświadczenia...**.

**OpenSSH mówi „bad permissions” dla pliku klucza.** Inne konta mogą czytać
klucz. W Ustawienia > SSH i poświadczenia wiersz **Plik klucza prywatnego**
pokazuje ostrzeżenie z przyciskiem **Ogranicz do mojego konta**.

**„staging...” zostaje na pasku stanu.** Odświeżanie w tle nadal trwa; znika,
gdy lista plików się zaktualizuje. `r` wymusza odświeżenie.

**Bardzo duże repozytorium działa wolno.** Zmniejsz *Początkowe commity w grafie* na
stronie Ustawienia > Log i graf (Log wczytuje więcej podczas przewijania, gdy *Leniwe wczytywanie commitów w grafie* jest włączone); ukryj
kolumnę *Zmiany* w Logu i wyłącz awatary; wyłącz *Cofanie włączone*, jeśli
nie potrzebujesz cofania (wtedy stage i commity nie wymagają żadnej księgowości).
`GITPOULPI_TIMING=1` zapisuje czasy poszczególnych żądań
do `gitpoulpi-timing.log` w folderze tymczasowym; warto dołączyć go do zgłoszenia
o wydajności.

**Brakuje przycisków AI albo są wyszarzone.** AI jest domyślnie wyłączone; zob. [Wsparcie
AI](ai.md#rozwiązywanie-problemów).

**GitPoulpi zamknął się sam.** Każde zakończenie jest zapisywane: poszukaj
`gitpoulpi-tui-exit.log` (oraz `gitpoulpi-tui-panic.txt` po awarii) w
katalogu konfiguracji (`gitpoulpi config path` wypisuje plik ustawień, który tam leży), i dołącz go do zgłoszenia.

**Gdzie są ustawienia?** `gitpoulpi config path`. Stan dla repozytorium jest w
`.gitpoulpi/repo-state.toml` wewnątrz repozytorium.

**Czy GitPoulpi coś gdzieś wysyła?** Tylko to, o co prosisz: operacje git
do twoich zdalnych repozytoriów, żądania hostingu po podłączeniu konta, żądania AI
po naciśnięciu przycisku AI oraz, jeśli włączysz awatary, skrót adresów e-mail
autorów do Gravatara. Nie ma telemetrii. Zob. [Wsparcie
AI](ai.md#prywatność).

## Każdy zapis kończy się błędem: „Unable to create index.lock”

Polecenie git, które przerwano (awaria, zabity terminal), może zostawić
`.git/index.lock`, a wtedy każda zmiana w repozytorium kończy się błędem. Gdy
plik ma więcej niż 10 sekund, GitPoulpi pokazuje pasek nad Logiem:
**Usuń index.lock** (paleta: *Repozytorium: Usuń nieaktualny index.lock*; CLI:
`gitpoulpi index-lock remove`). Najpierw pyta i usuwa tylko ten plik.
GitPoulpi nie potrafi niezawodnie stwierdzić, czy inny program git jeszcze działa,
więc przed potwierdzeniem upewnij się, że żaden nie działa (IDE, inny terminal). W powiązanym
worktree blokada jest własna dla worktree, w `.git/worktrees/<name>`.

## „git am” się zatrzymał, a Rebase: Przerwij nic nie robi

`am` (stosowanie patchy ze skrzynki pocztowej) nie jest rebase. Zatrzymany `git am`
ma własny pasek, **Stosowanie patchy (git am)**, z przyciskami Rozwiąż konflikty,
Kontynuuj, Pomiń i Przerwij (paleta: *git am: Kontynuuj / Pomiń ten patch /
Przerwij*; CLI: `conflicts am-continue|am-skip|am-abort`). Przerwij pyta najpierw,
odrzuca dotychczas zastosowane patche i przywraca gałąź do poprzedniego stanu. Tych operacji
nie można cofnąć, tak jak przycisków rebase.

## Cofnij jest wyszarzone i mówi „Nie można cofnąć: ...” (zablokowany wpis)

Najnowszy wpis dziennika utknął: repozytorium zmieniło się poza GitPoulpi
albo wpis został przerwany. Najedź na Cofnij, by poznać powód; jeśli utknął na
stałe, podpowiedź wskazuje **Edycja: Zapomnij zablokowany wpis cofania** (paleta) lub
`gitpoulpi undo-forget`. Najpierw pyta i tylko usuwa wpis z
dziennika; repozytorium nie jest ruszane. Zob. [Cofnij i ponów](undo-redo.md).

## Duże pliki wyglądają jak trzy wiersze tekstu („pliki LFS widać jako wskaźniki”)

Plik, który powinien być obrazem, modelem lub dużym plikiem binarnym, zawiera `version https://git-lfs.github.com/spec/v1` i wiersz `oid sha256:...`: to **wskaźnik** Git LFS, a prawdziwa zawartość nie została pobrana (albo filtry LFS nie są zainstalowane).

- Spójrz na panel Status: repozytorium używające LFS, gdy brakuje `git-lfs`, pokazuje bursztynowy baner. Zainstaluj program (https://git-lfs.com; Windows: dołączony do Git for Windows lub `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) i otwórz ponownie lub naciśnij `r`.
- `gitpoulpi lfs status` mówi, czy program jest zainstalowany, czy filtry i hook są skonfigurowane i które wzorce są śledzone; `gitpoulpi lfs files` wypisuje pliki wskaźników i które z nich są `local`.
- Przy zainstalowanym `git-lfs` uruchom w palecie **`LFS: Pobierz obiekty (pull)`** (lub `gitpoulpi lfs pull`): pobiera obiekty i zastępuje wskaźniki. Najpierw instaluje filtry LFS dla tego repozytorium, jeśli ich nie ma.
- Jeśli pobieranie się nie powiedzie, komunikat podaje powód (poświadczenia do zdalnego repozytorium, serwer, który już nie ma obiektu). GitPoulpi nie potrzebuje własnego konta: LFS używa zwykłych poświadczeń twojego zdalnego repozytorium (menedżer poświadczeń lub ssh-agent).
- Klonowanie repozytorium LFS nigdy nie kończy się błędem z powodu LFS; jeśli krok pobierania nie mógł się wykonać, repozytorium zostaje sklonowane, ale ze wskaźnikami, a `LFS: Pobierz obiekty (pull)` dokończy go później. `gitpoulpi clone --no-lfs` pomija pobieranie celowo.
- Całe LFS można wyłączyć w Ustawienia > Git i kodowanie > Git LFS (albo `[git] lfs_enabled = false`).

## „git-lfs not found” / „Git LFS is not installed”

Komunikat `Git LFS is not installed. This repository stores large files with Git LFS; ...` (albo okno dialogowe z przyciskiem *Otwórz stronę instalacji*, albo własne `git-lfs: command not found` / `'lfs' is not a git command` z gita) oznacza, że repozytorium jest skonfigurowane pod LFS - jego `.gitattributes`, filtry w `.git/config` lub hook `pre-push` wywołują `git-lfs` - a programu nie ma w `PATH` widzianym przez GitPoulpi. Zainstaluj go (link wyżej). W Windows program jest dołączony do Git for Windows; jeśli `git lfs version` działa w terminalu, a GitPoulpi nadal mówi, że go brakuje, uruchom GitPoulpi z terminala, który widzi ten sam `PATH`. Aby ukryć LFS przed GitPoulpi, zostawiając własne zachowanie gita w spokoju, wyłącz `git.lfs_enabled`.

## Graf jest rysowany znakami, a nie jako obrazek (herdr, tmux, inne terminale)

GitPoulpi rysuje kolumnę Graf i awatary autorów jako piksele tylko tam, gdzie ma pewność, że terminal je pokaże: obrazy inline iTerm2 w samym WezTerm, Rio, mintty i iTerm2 starszym niż 3.7.7; protokół grafiki Kitty w kitty, Ghostty, iTerm2 3.7.7 i nowszym oraz **herdr, gdy własna konfiguracja herdr ustawia `kitty_graphics = true`** (`[terminal]`, a w starszym herdr `[experimental]`), a herdr działa wewnątrz WezTerm, kitty lub Ghostty. Obrazy sixel są używane w Windows Terminal, foot, Tabby, Konsole i mlterm (oraz w tmux, gdy terminal za nim obsługuje sixel). Wszędzie indziej rysuje znaki, co jest zawsze poprawne.

- W herdr z włączoną grafiką Kitty, ale z konfiguracją, której GitPoulpi nie znajduje (lub opartą na domyślnej herdr), ustaw `ui.image_protocol = "kitty"` (Ustawienia > Log i graf > Protokół obrazów) i uruchom ponownie.
- W terminalu, który pokazuje kwadraty lub śmieci tam, gdzie powinien być graf, ustaw `ui.image_protocol = "none"` lub `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` wyłącza obrazki niezależnie od konfiguracji.

**herdr zdalnie (`herdr --remote <host>` lub `ssh`, potem `herdr`): domyślnie jest graf tekstowy.** Przez SSH GitPoulpi działa na zdalnym hoście i może czytać tylko środowisko zdalnego hosta oraz jego `config.toml` herdr. To, czy pojawi się obrazek Kitty, decyduje *lokalny* klient herdr (jego własne ustawienie `kitty_graphics` i własny terminal), którego GitPoulpi nie widzi: gdy klient nie rysuje obrazków, komórki pozostają puste. Dlatego GitPoulpi rysuje graf tekstowy, ilekroć wykryje herdr przez SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Jeśli twój lokalny herdr ma `kitty_graphics = true` i terminal rysujący obrazy Kitty (kitty, Ghostty), włącz to ustawieniem `ui.graph_renderer = "image"` (lub `ui.image_protocol = "kitty"`) na zdalnym hoście. Obrazki są wysyłane inline (`t=d`), co działa przez dowolny strumień bajtów. Jeśli kolumna jest pusta, ustaw `ui.graph_renderer = "text"`.

**Jaśniejszy prostokąt za ścieżkami.** Obrazek jest przezroczysty (PNG RGBA z prostym kanałem alfa) wszędzie tam, gdzie nic nie narysowano, więc przebijają przez niego tło terminala i pas zaznaczenia. Terminale z protokołem Kitty, **WezTerm** i **Rio** składają ten kanał alfa; GitPoulpi rezerwuje wtedy zwykłe puste komórki pod obrazkiem i wysyła go po nich. Każdy inny terminal z protokołem iTerm2 (starszy iTerm2, mintty) nie jest znany z mieszania alfa PNG, więc tam obrazek niesie nieprzezroczyste tło w kolorze motywu, które może być widoczne jako prostokąt w terminalu o innym tle; `ui.graph_renderer = "text"` je usuwa. Przy `ui.graph_lane_background = "off"` na ścieżkach przezroczystych nic nie jest malowane za ścieżkami.

**Windows: kolumna grafu jest pusta, w złych wierszach albo ekran się rwie (WezTerm 20240203, Rio, mintty).** Pseudokonsola za tymi terminalami gubi lub przesuwa sekwencje obrazków ([Wybór terminala](terminals.md#windows)). Ustaw `ui.graph_renderer = "text"` albo użyj Windows Terminal.

W tmux, zellij, screen i herdr zob. [Multipleksery](multiplexers.md): co przechodzi i jaką konfigurację dodać.

Zob. `ui.graph_renderer` i `ui.image_protocol` w [Opisie konfiguracji](config-reference.md).
