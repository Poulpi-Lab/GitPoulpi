**Sekwencja** to nazwana lista poleceń uruchamianych jedno po drugim: na
przykład *Stage wszystkie zmiany*, *AI: Wygeneruj opis commita*, *Commit*, *Push*
pod jednym klawiszem **Backup**. Każdy krok to polecenie
[palety poleceń](keybindings.md); sekwencja ma własny wiersz palety
(`Sekwencja: <nazwa>`), da się ją przypisać do klawisza i edytuje się ją w
**Ustawienia > Sekwencje** albo w `config.toml`.

## Strona Sekwencje

**Ustawienia > Sekwencje** to [strona z listą](settings.md#listy): sekwencje
po lewej, wiersze zaznaczonej po prawej. Każda zmiana jest stosowana i
zapisywana od razu, a paleta i mapa klawiszy podążają za nią bez restartu.

```
 Sekwencje
+- Sekwencje -------------+  Edycja sekwencji 1/2
| Ta instalacja           |  Nazwa        Backup
| > Backup        Alt+B   |  Id           backup
|   Worktree i agent      |  Opis         Stage wszystkiego, ...
| To repozytorium         |  Zakres       Ta instalacja
|   Release               |  Skrót        [ Alt+B ] [ Wyczyść ]
+-------------------------+  Kroki        Stage > AI: ... > Commit > Push
| [+ Dodaj v] [Dup] [Usuń]|              [ Edytuj kroki (4) ] [ Uruchom teraz ]
| [W górę] [W dół]        |
+-------------------------+
```

* **+ Dodaj** oferuje **Pusta sekwencja** i trzy szablony, **Z szablonu >
  Kopia zapasowa**, **Skomponuj i push** oraz **Worktree i agent**. Szablon to
  punkt wyjścia: nic nie jest aktywne, dopóki go nie dodasz, a to, co dodasz, jest
  kopią, którą możesz edytować. Szablony dostają nazwę w języku interfejsu w
  chwili dodania.
* **Duplikuj**, **Usuń** (pyta najpierw), **W górę** / **W dół** albo przeciągnięcie
  zmieniają kolejność listy (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`).
  Kolejność listy to kolejność w `config.toml`.
* Wiersz z problemem jest oznaczony na czerwono i mówi dlaczego (brak kroków,
  krok, którego ta wersja nie zna, pauza `Nigdy`, której brakuje potrzebnej opcji).
  Sekwencję z problemami **można zapisać**, nieznane przyszłe kroki są zachowywane
  bez zmian, ale jej wiersz palety zostaje wyszarzony, dopóki nie zostanie naprawiona.

### Wiersze sekwencji

| Wiersz | Co robi |
|---|---|
| **Nazwa** | To, co pokazuje paleta: `Sekwencja: <nazwa>`. |
| **Id** | Krótka nazwa używana w `config.toml`, w wierszu poleceń i jako klucz skrótu (`a-z`, `0-9`, `_`, `-`). Zmiana nazwy przenosi razem z nią skrót, w tym samym zapisie. |
| **Opis** | Opcjonalny, dla ciebie. |
| **Zakres** | Zob. niżej. |
| **Skrót** | Przycisk pokazujący klawisz. `Enter` lub kliknięcie otwiera to samo przechwytywanie klawisza co **Klawiatura i mysz**: konflikty i ostrzeżenia terminala pokazują się podczas naciskania, a klawisz używany przez inną akcję jest odrzucany tak samo. **Wyczyść** go usuwa. Klawisz jest zapisywany jako `[keybindings.sequence] <id>`. |
| **Kroki** | Kroki po kolei. **Edytuj kroki** je otwiera. |
| **Uruchom teraz** | Uruchamia sekwencję w bieżącym oknie. |

### Zakres

Każda sekwencja ma zakres, zapisywany razem z nią:

| Zakres | Gdzie jest zapisana | Kto ją widzi |
|---|---|---|
| **Ta instalacja** (domyślnie) | `[[sequences]]` w `config.toml`, z tą instalacją jako właścicielem | Tylko ta instalacja GitPoulpi. |
| **Wszystkie instalacje na tym komputerze** | `[[sequences]]` w `config.toml`, bez właściciela | Każda instalacja współdzieląca ten plik. |
| **Tylko to repozytorium** | `.gitpoulpi/sequences.toml` w bieżącym repozytorium | Każdy, kto ma repozytorium. Liczy się dopiero, gdy jej zaufasz (zob. niżej). |

Zmiana zakresu przenosi definicję: wybranie **Tylko to repozytorium** zapisuje
ją w pliku repozytorium i usuwa z `config.toml`, a odwrotnie tak samo. Zakres
repozytorium wymaga otwartego repozytorium.

### Sekwencje repozytorium i zaufanie

Plik `.gitpoulpi/sequences.toml` może być zatwierdzony, więc może pochodzić od
innych ludzi, a sekwencja uruchamia polecenia na twoim komputerze. Dlatego
sekwencje repozytorium **nie są oferowane, dopóki im nie zaufasz**, tak jak
polecenia zatwierdzonego pliku przepływu pracy.

Gdy repozytorium ma taki plik, któremu nikt jeszcze nie zaufał, nad Logiem
pojawia się powiadomienie: *To repozytorium definiuje 2 sekwencje* z przyciskami
**Przejrzyj**, **Zaufaj** i **Ignoruj**. Nigdy niczego nie blokuje.

- **Przejrzyj** wylicza każdą sekwencję z każdym krokiem, jego pauzą i opcjami,
  tylko do odczytu. **Zaufaj** tam ufa temu, co wyliczono; **Zamknij** (albo `Esc`)
  nie ufa niczemu. Jeśli lista jest dłuższa niż okno, Zaufaj nie jest tam
  oferowane (powiększ okno): nigdy nie ufasz temu, czego nie mogłeś przeczytać.
- **Zaufaj** (przycisk albo polecenie palety *Sekwencja: Zaufaj sekwencjom tego
  repozytorium*) ufa plikowi dokładnie w obecnym kształcie.
- **Ignoruj** ukrywa powiadomienie do zmiany pliku.
- Z klawiatury: paleta ma *Sekwencja: Przejrzyj / Zaufaj / Ignoruj sekwencje tego
  repozytorium*.

Zapisywany jest skrót z id, poleceń, pauz i opcji, trzymany w lokalnym stanie
repozytorium (nigdy w samym repozytorium). Jeśli cokolwiek z tego się zmieni, na
przykład po pull, powiadomienie wraca jako *Sekwencje tego repozytorium się
zmieniły*, a sekwencje znikają z palety i spod klawiszy, dopóki nie zaufasz im
ponownie. Zmiana nazwy sekwencji ani edycja jej opisu niczego nie zmienia.
Sekwencja, którą napiszesz sam w Ustawieniach, zostaje zaufana, gdy plik był już
zaufany (albo pusty) przed twoją edycją.

Po zaufaniu sekwencje repozytorium pojawiają się jako `Sekwencja: <nazwa>` w
palecie jak twoje własne i można im nadać skrót w Ustawieniach. Sekwencja o tym
samym id co jedna z twoich to kolizja: wygrywa twoja, a druga jest zgłaszana
jako duplikat.

Z wiersza poleceń: `gitpoulpi sequence list` pokazuje sekwencje pliku z
`trusted` i stanem `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (oraz `untrust`, `--repo PATH` dla innego
repozytorium) jest dla człowieka, który uruchamia je po przeczytaniu `sequence show`.
`sequence run` odmawia uruchomienia niezaufanej sekwencji repozytorium. Agenci AI nie mogą
ufać: nie ma do tego narzędzia MCP.

## Kroki

**Edytuj kroki** zastępuje listę krokami sekwencji (tytuł mówi **Kroki sekwencji
<nazwa>**), a wiersze wierszami zaznaczonego kroku. **Wstecz**
(krzyżyk u góry wierszy) albo `Esc` wraca do sekwencji.

* **+ Dodaj** otwiera **Dodaj krok**: każde polecenie, które sekwencja może
  uruchomić, pod jego nazwą z palety (te od AI z iskrą), filtrowane podczas
  pisania dopasowaniem palety. Krok, którego ta wersja nie może uruchomić, jest
  wyszarzony z powodem. `Enter` albo kliknięcie dodaje go na końcu; `Esc` albo
  krzyżyk zamyka okienko.
* **Duplikuj**, **Usuń**, **W górę** / **W dół** i przeciąganie działają tak jak
  dla sekwencji.
* Polecenie destrukcyjne (usunięcie gałęzi, odrzucenie zmian...) nigdy nie jest
  oferowane: sekwencja nie może go uruchomić.

### Pauza

Każdy krok ma **Pauzę**:

| Pauza | Znaczenie |
|---|---|
| **Domyślnie** | Krok zatrzymuje się tylko wtedy, gdy potrzebuje odpowiedzi, w swoim zwykłym oknie dialogowym (nazwa gałęzi, opis commita, który jest jeszcze pusty...). |
| **Zawsze** | Zatrzymaj przed tym krokiem i zapytaj, czy kontynuować. |
| **Nigdy** | Nigdy się nie zatrzymuj. Opcje kroku muszą dać mu wszystko, czego potrzebuje, więc krok, który pyta o nazwę, wymaga wypełnionej opcji **Nazwa**. |

Krok, który zawsze pyta przed uruchomieniem (force push), nie oferuje
**Nigdy**: wiersz mówi dlaczego. **Nigdy** na kroku opisu od AI jest dozwolone
jako jawny wybór.

### Opcje

Wiersze pod **Pauzą** to opcje kroku: tekst **Nazwa** lub **Opis**, **Gałąź**,
pole wyboru takie jak **Otwórz w nowej karcie** albo lista rozwijana wypełniona z
twojej konfiguracji (**Zdalne**, **Profil AI**, wpis **Otwórz w**). Teksty
przyjmują zmienne `${branch}`, `${date}`, `${time}` i `${n}` (liczba, która
czyni nazwę unikalną). Wyczyszczenie tekstu usuwa opcję.

## Tworzenie i edycja z AI

Z [włączonym AI](ai.md#szkic-sekwencji) (Ustawienia > AI) pojawiają się dwa wpisy:
**Dodaj > Utwórz z AI...** (także wiersz palety `Sekwencja: Utwórz z AI...`, który
otwiera Ustawienia na tej stronie z oknem dialogowym) oraz **Edytuj z AI...** obok
**Uruchom teraz** dla zaznaczonej sekwencji. Żaden nie istnieje, gdy AI jest wyłączone.

```
+- Utwórz sekwencję z AI ----------------------------------- x -+
| Opisz sekwencję, jakiej chcesz                                |
| +-----------------------------------------------------------+ |
| | stage wszystkiego, opis commita napisany przez AI,        | |
| | commit, potem push                                        | |
| +-----------------------------------------------------------+ |
| Enter dodaje wiersz. Ctrl+Enter wysyła. Esc zamyka.           |
| [ ] Wyślij kontekst (nazwy zdalnych, profile AI, etykiety ...)|
|    Wyłączone: wysyłany jest tylko twój opis.                  |
| AI tylko proponuje. Nic nie jest zapisywane przed akceptacją..|
| [ Wyślij ] [ Anuluj ]                                         |
+---------------------------------------------------------------+
```

Piszesz, czego chcesz, własnymi słowami; **Wyślij** (albo `Ctrl+Enter`, `Ctrl+S`)
pyta model, przez ten sam profil, wskaźnik zadań i `Esc` co każda inna akcja AI
(i to samo pytanie „Co zostanie wysłane”). Odpowiedzią jest **propozycja**, którą
czytasz, zanim cokolwiek się stanie:

* nazwa, id, opis i kroki z ich pauzą i opcjami;
* dla **Edytuj z AI** diff względem bieżącej sekwencji: `+` krok dodany,
  `-` krok usunięty, `~` krok, którego pauza lub opcje się zmieniły, `~ Nazwa: „a”
  -> „b”` dla pola;
* **Pominięto**: krok zaproponowany przez model, który nie jest dozwolony
  (nieznany, destrukcyjny, sekwencja wewnątrz sekwencji, niemożliwa pauza), z
  powodem; poprawne kroki zostają;
* **uwagi** modelu oraz **sugerowany skrót**, który jest tylko pokazywany, z
  informacją, czy klawisz jest już zajęty. Nigdy nie jest przypisywany: użyj
  potem przycisku **Skrót**.

```
+- Proponowana zmiana -------------------------------------- x -+
| + dodano   - usunięto   ~ zmieniono                           |
| ~ Nazwa: „Daily” -> „Daily 2”                                 |
| Kroki (3)                                                     |
| + 1. Git: Fetch wszystkich zdalnych                           |
|   2. Git: Stage wszystkie zmiany                              |
| - Git: Commit...                                              |
| ~ 3. Git: Push: domyślne -> Pauza: Zawsze                     |
| Pominięto                                                     |
| krok 4 `reset_hard`: kroki destrukcyjne nie są dozwolone      |
| [ Akceptuj ] [ Edytuj przed zapisem ] [ Odrzuć ] [ Wstecz ]   |
+---------------------------------------------------------------+
```

* **Akceptuj** zapisuje to jednym zapisem (nowa sekwencja albo zastąpienie
  edytowanej, która zachowuje swoje id, zakres i klawisz) i zaznacza ją na liście.
* **Edytuj przed zapisem** robi to samo i otwiera jej kroki, byś mógł poprawić
  wiersze (ta strona zapisuje każdą edycję od razu, więc sekwencja jest już
  zapisana).
* **Odrzuć** niczego nie zapisuje. **Wstecz** wraca do opisu, by wysłać ponownie.
* **Akceptuj** jest niedostępne, gdy nie ma czego akceptować: brak użytecznego
  kroku, problem, który strona by oznaczyła, albo edycja, która niczego nie zmienia.

Nic nie jest zapisywane przed **Akceptuj**, a sekwencja nigdy nie jest stąd
uruchamiana. Zamknięcie okna dialogowego (krzyżyk, `Esc`, kliknięcie poza nim)
pyta „Odrzucić zmiany?”, gdy wpisano opis albo otwarta jest propozycja;
zamknięcie Ustawień też pyta.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<id tej instalacji>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Klucze, których ta wersja nie zna, i kroki, których nie zna, są zachowywane, gdy
strona zapisuje.

## Uruchamianie sekwencji

Sekwencja to nazwana lista kroków, każdy to polecenie palety (stage wszystkiego, opis commita napisany przez AI, commit, push...). Uruchom ją z palety (`Sekwencja: <nazwa>`), klawiszem, który jej nadałeś w Ustawienia > Klawiatura i mysz (`[keybindings.sequence]` w `config.toml`), albo z wiersza poleceń (`gitpoulpi sequence run`).

Gdy działa, pasek nad górą Logu pokazuje `<nazwa> k/n`, krok i na co czeka. Uruchomienie widać też w panelu Zadania jako `Sekwencja <nazwa> k/n: <krok>`. Naraz działa jedna sekwencja; uruchomienie drugiej mówi, która już działa.

**Gdzie zatrzymuje się, by zapytać.** Każdy krok ma ustawienie `pause` (`default`, `always`, `never`):

| Krok | Domyślnie | `pause = "never"` |
|---|---|---|
| Opis commita od AI | opis trafia do pola opisu; pasek mówi „Sprawdź opis, potem Kontynuuj” | idzie dalej od razu |
| Commit | pusty opis zawsze zatrzymuje: pole dostaje klawiaturę, a twój commit (albo Kontynuuj) idzie dalej | niedostępne, gdy nie ma opisu do zatwierdzenia |
| Kompozytor commitów od AI | otwiera się przegląd; Zastosuj idzie dalej, zamknięcie go anuluje | stosuje propozycję bez zmian |
| Utwórz worktree | otwiera się okno z wypełnioną nazwą | tworzy z `options.name` bez okna |
| Push | gdy `git.push_behavior = ask`, otwiera się wybór „Push do...” | wypycha do `options.remote` |
| Force push z lease, stage plików wyglądających na sekrety | zawsze pyta | niedozwolone |

`pause = "always"` na dowolnym kroku pokazuje „Dalej: <krok>” z przyciskami **Kontynuuj** i **Anuluj** przed jego uruchomieniem. Jeśli push zostanie odrzucony, bo gałąź się rozeszła, otwiera się zwykłe okno, a pasek mówi „Push odrzucony: wybierz w oknie dialogowym”; pull, a potem push (albo force push z lease) kontynuuje uruchomienie, zamknięcie okna je anuluje.

**Kontynuuj** to przycisk paska, `Sekwencja: Kontynuuj` w palecie albo klawisz przypisany do `sequence.continue`. **Anuluj** to krzyżyk zamknięcia paska, `Sekwencja: Anuluj` (`sequence.cancel`), panel Zadania albo `Esc`, gdy nic innego go nie chce (otwarte okno zamyka się najpierw, co też anuluje). Anulowanie zachowuje kroki, które już się wykonały, i mówi o tym: „<nazwa> anulowana po kroku k/n (<krok>); wcześniejsze kroki zostają”.

**Gdy krok się nie powiedzie**, uruchomienie zatrzymuje się tam: pasek stanu nazywa sekwencję, krok i błąd, pasek robi się czerwony z przyciskiem **Szczegóły** (wpis Dziennika aktywności, który wylicza opcje kroku) i krzyżykiem zamknięcia, a wcześniejsze kroki zostają wykonane.

**Worktree.** Krok tworzenia worktree z `open_tab = true` przenosi uruchomienie do karty nowego worktree: kolejne kroki (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) działają tam. Jeśli na ekranie jest inna karta, pasek mówi „czekam na kartę <nazwa>” z przyciskiem **Przejdź tam**. Zamknięcie tej karty anuluje uruchomienie. Uruchomienie nie przeżywa zamknięcia GitPoulpi.

### Agent proponujący uruchomienie

Gdy włączone jest *Pozwól agentom AI proponować zmiany do potwierdzenia przeze mnie* (`[mcp]
allow_ui_propose`), agent może poprosić twoje okno o uruchomienie sekwencji
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Pojawia się baner:
*✦ Zaproponowane przez <agent>: uruchom sekwencję <nazwa>* z przyciskami **Uruchom** i **Odrzuć**.
Nic nie rusza, dopóki nie naciśniesz Uruchom (kliknij je albo `Ctrl+N`, potem `Enter`);
Odrzuć, `Esc` (gdy nic innego go nie chce) albo polecenie palety *Agent: Odrzuć
oczekujący widok* go porzuca. Uruchomienie przebiega potem dokładnie tak, jakbyś je
sam zaczął, zatrzymując się tam, gdzie każe sekwencja. Propozycja jest odrzucana, gdy
sekwencja nie istnieje, jest nieprawidłowa albo to sekwencja repozytorium, której nie zaufałeś.

## Cofanie uruchomienia

Każdy krok, który zmienia repozytorium (stage, commit, tag, gałąź, stash, worktree...), zostawia **własny zwykły wpis cofania**, oznaczony uruchomieniem, więc `Ctrl+Z` nadal cofa po jednym kroku, a [Historia cofania](undo-redo.md#historia-cofania) pokazuje na nich `⟳ <nazwa>`. By cofnąć całe uruchomienie, jest **Cofnij całą sekwencję**: przycisk Historii cofania (`x`), wiersz palety `Edycja: Cofnij całą sekwencję` albo `gitpoulpi undo --sequence`. Cofa wpisy uruchomienia od najnowszego, każdy z własnymi kontrolami i pytaniami (commit, który już został wypchnięty, pyta najpierw), zatrzymuje się na pierwszej odmowie, nazywając krok, i zostawia starsze w dzienniku. Opis od AI i push nie zostawiają wpisu: push nigdy nie jest cofany. Dotyczy tylko dziennika bieżącego repozytorium; kroki wykonane w worktree cofa się z tamtego repozytorium. Ponów zostaje per wpis. Szczegóły: [Uruchomienia sekwencji](undo-redo.md#uruchomienia-sekwencji).
