[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · **Polski** · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# Wsparcie AI

GitPoulpi nigdy nie wymaga konta, a **AI jest domyślnie wyłączone**. Po
włączeniu AI może pisać opisy commitów i stash, wyjaśniać zmiany, pisać opisy
pull requestów, sugerować rozwiązania konfliktów merge i dzielić zmiany robocze
na kilka commitów. Model jest twój: klucz API, który posiadasz, lokalny serwer
albo już zainstalowany agent CLI (taki jak `claude -p`).

Każdy element sterujący AI pokazuje fioletowe `✦`. Gdy AI jest wyłączone:

- Nic nie jest nigdzie wysyłane.
- Nie widać żadnego elementu AI: ani przycisku `✦`, ani polecenia palety
  `AI:`, ani wiersza menu AI, ani podpowiedzi klawiszy AI. Klawisze AI nic nie
  robią.
- Strona AI repozytorium pokazuje tylko „AI jest wyłączone (Ustawienia > AI)”.
- Serwer MCP nie wylicza narzędzi `ai_*`. Polecenia CLI `gitpoulpi ai ...`
  odmawiają z tym samym komunikatem.

**Włącz AI** działa od razu. Nie trzeba restartować.

Z AI w GitPoulpi można korzystać na dwa sposoby:

| | Wbudowane AI (ta strona) | Zewnętrzny agent |
|---|---|---|
| Co to jest | GitPoulpi prosi model o opis commita, wyjaśnienie, ... | Claude Code, Cursor, Codex... sterują GitPoulpi jak narzędziem |
| Gdzie skonfigurować | Ustawienia > AI (`[ai]`) | konfiguracja MCP agenta, limity w `[mcp]` |
| Przewodnik | niżej | [MCP i agenci](mcp-and-agents.md) |

## Konfiguracja AI

1. `Ctrl+,` > **AI** > zaznacz **Włącz AI**.
2. Profile tworzą listę karty **Profile (połączenia)**, po lewej stronie strony (na wąskim terminalu nad resztą):
   wybierz jeden, aby edytować go po prawej (`Enter` przechodzi do jego wierszy;
   polecenie palety **AI: Edytuj profil...** przenosi do listy). **+ Dodaj**
   tworzy kolejny, **Duplikuj** kopiuje zaznaczony (bez jego klucza), a **Usuń**
   najpierw pyta. Profil o nazwie *Default* zawsze istnieje. Wybierz
   **Dostawca** i wypełnij jego pola:

   | Dostawca | Pola | Wartości domyślne |
   |---|---|---|
   | **Agent lokalny** (bez klucza, bez własnego konta) | **Typ agenta** (jaki to agent), **Polecenie** (dokładny program: w `PATH`, alias lub skrypt opakowujący albo pełna ścieżka), **Argumenty** czyniące go nieinteraktywnym i opcjonalne **zmienne środowiskowe**. Prompt trafia na jego stdin, odpowiedź jest czytana ze stdout. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **Klucz API**, opcjonalnie **Model** i **Bazowy URL** | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **Klucz API**, Model, Bazowy URL | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **Klucz API**, Model, Bazowy URL | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **Zgodny z OpenAI** (Ollama, LM Studio, vLLM, firmowa brama) | **Bazowy URL** (`.../v1`), **Model**, opcjonalny klucz | `http://localhost:11434/v1` (Ollama; LM Studio to `http://localhost:1234/v1`), `llama3` |

   **Ustawienia wstępne** (zawsze można wpisać inną wartość):

   ![Lista Typ agenta profilu agenta lokalnego, wyliczająca Claude Code, OpenAI Codex CLI, Gemini CLI, opencode i inne agenty CLI](../images/settings-ai-presets.png)

   - **Agent lokalny > Typ agenta** to lista popularnych agentów CLI, najczęściej
     używane pierwsze: Claude Code, OpenAI Codex CLI, Gemini CLI, opencode, Cursor
     Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose, Crush, Amp lub
     *Inny (bez podpowiedzi)*. Typ jest **osobny od polecenia**: wybiera tylko to,
     jakie podpowiedzi dostajesz. Wybranie go wstępnie wypełnia **Polecenie**
     (dopóki jest jeszcze domyślnym poprzedniego typu) i **Argumenty** wywołaniem
     headless (`claude -p`, `codex exec -`, `gemini -p ...`, ...). **Polecenie**
     to wolny tekst: wpisz `/opt/bin/work-claude`, skrypt opakowujący albo cel
     aliasu, a typ razem z opcjami, listą modeli i zmiennymi zostaje. Gdy **Typ
     agenta** jest pusty, GitPoulpi wnioskuje go z polecenia.
   - **Opcje argumentów** dodaje lub usuwa opcje pasujące do jednorazowego użycia
     przez GitPoulpi, każdą z jednowierszowym uzasadnieniem: bez narzędzi, bez
     zapisanej sesji, wyjście jako zwykły tekst, mało rozumowania, tryb tylko do
     odczytu, bez próśb o zgodę, bez sprawdzania aktualizacji i statystyk. Lista
     różni się w zależności od agenta, bo agenci się różnią (tylko niektórzy
     potrafią wyłączyć narzędzia z wiersza poleceń; w opencode jest to zmienna
     środowiskowa). **Model** wylicza modele tego agenta i zapisuje właściwą flagę
     (`--model haiku` dla Claude Code, `-m gpt-6-luna` dla Codex,
     `--model provider/model` dla opencode); *(domyślny - bez flagi)* ją usuwa.
     Amp nie ma flagi modelu. `aider` czyta prompt z `/dev/stdin`, więc działa
     tylko w Linuksie i macOS.
   - **Zmienne środowiskowe** są ustawiane tylko dla procesu tego agenta, nigdy
     dla GitPoulpi ani innych profili. **Dodaj zmienną środowiskową** oferuje
     udokumentowane zmienne ważne dla tego agenta, każdą z opisem działania
     (wyłączenie telemetrii i sprawdzania aktualizacji, poziom rozumowania, folder
     konfiguracji, klucz API, bazowy URL lub lokalny host, ...); można też wpisać
     `NAME` lub `NAME=value`. Każda zmienna ma potem własny wiersz **Env NAME**:
     jej wartość jako przycisk (naciśnij go, by wpisać nową; zapisuje się po
     opuszczeniu pola) oraz przycisk **[x]**, który usuwa zmienną. Wartością może być `${env:NAME}`, aby pożyczyć jedną z własnych
     zmiennych GitPoulpi (przydatne dla kluczy API, by nie trafiały do
     `config.toml`). Wartości wyglądające na sekrety (nazwy kończące się na
     `_KEY`, `_TOKEN`, `_SECRET`, ... lub znane prefiksy kluczy) pokazują się jako
     kropki i wpisuje się je od nowa. Wartości nigdy nie trafiają do logów.
   - **Dostawcy HTTP > Ustawienie wstępne dostawcy** wypełnia bazowy URL i model dla OpenAI,
     Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek, Mistral, Groq, Together,
     Fireworks, Cerebras, Perplexity, Moonshot, Qwen (DashScope), Zhipu (GLM)
     oraz lokalnych serwerów Ollama, LM Studio, llama.cpp, vLLM, Jan i LocalAI.
     **Model** to lista modeli tego dostawcy, dozwolony jest wolny tekst.

3. Naciśnij **Testuj połączenie**: *Ostatni test* pokazuje `połączono: model odpowiedział`
   lub `niepowodzenie: <powód>`.
4. Przy kilku profilach wybierz **Profil domyślny** na dole; każda akcja może
   nadal wskazać własny.

![Blok Profile strony AI dla agenta lokalnego: Typ agenta (Claude Code), Polecenie, Argumenty, Model, dwa wiersze Env, Dodaj i Usuń zmienną środowiskową, Testuj połączenie i Profil domyślny](../images/settings-ai-agent.png)

**Klucze API** wpisuje się przez `Enter` w wierszu **Klucz API** i zapisuje w
systemowym menedżerze poświadczeń (Git Credential Manager, libsecret / `pass`,
Keychain) pod `ai:<profile id>`, nigdy w `config.toml`. Bez menedżera poświadczeń GitPoulpi
mówi o tym wprost, zamiast udawać, że go przechowuje. *Ustawienia > SSH i
poświadczenia > Zapomnij wszystkie nazwy użytkowników i hasła* je usuwa.

To samo w `config.toml` (reszta jest w [opisie
konfiguracji](config-reference.md#ai-wbudowane-ai-domyślnie-wyłączone)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # dowolne: alias, skrypt opakowujący, ścieżka bezwzględna
harness = "claude"        # jaki to agent; puste = wnioskowane z `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # tylko dla procesu tego agenta; ${env:NAME} pożycza naszą
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Tryb uruchomienia: headless lub terminal wirtualny

Profil agenta lokalnego uruchamia swojego agenta na jeden z dwóch sposobów,
wybieranych w **Tryb uruchomienia** (widoczny dla Claude Code, OpenAI Codex CLI
i Gemini CLI):

- **Headless (nieinteraktywny)**, domyślny: jednorazowy tryb agenta
  (`claude -p`), prompt na stdin, odpowiedź na stdout.
- **Interaktywny (terminal wirtualny)**: GitPoulpi uruchamia własny ekran agenta
  w ukrytym terminalu, wpisuje żądanie tak jak człowiek, czeka na odpowiedź
  i zamyka agenta. Niektórzy agenci inaczej rozliczają lub ograniczają tryb
  headless i interaktywny; który przypadek dotyczy ciebie i co zasady dostawcy
  mówią o każdym z nich, zależy od dostawcy, więc sprawdź jego zasady.

Każdy tryb ma własne **Argumenty**, **Opcje argumentów** i **Model**:
przełączanie trybów niczego nie gubi, a tryb interaktywny oferuje tylko opcje
sensowne bez flagi headless. Dwa dalsze wiersze dotyczą obu trybów:

- **Folder roboczy**: *Folder dedykowany* to pusty folder należący do GitPoulpi,
  więc agent nie widzi żadnego projektu i raz prosi o zaufanie. *Bieżące
  repozytorium* daje agentowi własny kontekst projektu (na przykład jego
  `CLAUDE.md`), a agent prosi o zaufanie raz na repozytorium. *Domyślny* to
  folder dedykowany w trybie interaktywnym, a w trybie headless folder, w którym
  działa GitPoulpi, jak dotąd.
- **Limit czasu (sekundy)**: ile czasu może zająć agentowi, zanim GitPoulpi się
  podda i go zatrzyma. Puste oznacza 120 sekund w headless i 180 w interaktywnym.

**Na pytania przy pierwszym uruchomieniu odpowiadasz ty.** Agent, który nigdy nie
działał w danym folderze, najpierw o coś pyta (zaufać folderowi, zalogować się,
wybrać motyw). GitPoulpi nigdy nie odpowiada za ciebie: żądanie zatrzymuje się
z komunikatem, a **Otwórz agenta...** (w profilu albo *AI: Otwórz agenta* w
palecie poleceń) otwiera agenta jednorazowo, w widocznym terminalu, w folderze
roboczym, którego użyje. Odpowiedz na pytanie, zamknij agenta i spróbuj ponownie;
odpowiedź zapamiętuje agent.

Odpowiedź jest czytana z transkryptu sesji agenta w przypadku Claude Code (jego
dokładny tekst, usuwany potem) oraz z ekranu między dwiema liniami znaczników,
o które prosi żądanie, w przypadku Codex i Gemini. Naraz działają co najwyżej
dwa uruchomienia w terminalu wirtualnym, a naciśnięcie Esc w trakcie żądania
zatrzymuje agenta.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # zestaw headless, zachowany
run_mode = "terminal"               # headless (domyślnie) | terminal
terminal_args = ["--effort", "low"] # zestaw interaktywny
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 lub brak: 120 headless, 180 terminal
```

## Instrukcje

Tekst, który dodajesz do tego, co dostaje model. Wbudowany prompt każdej akcji
(który ustala format odpowiedzi) zawsze idzie pierwszy i nie można go edytować;
potem cztery poziomy, od najogólniejszego do najbardziej szczegółowego, każdy tylko
gdy niepusty:

1. **Instrukcje globalne** (każda akcja). Znaczniki *Szybkie wstawianie* w edytorze
   wstawiają typowe zdania: *Odpowiadaj w `<języku>`*, *Zwięzłe odpowiedzi*,
   *Tryb rozkazujący*, *Ton techniczny*, *Ton swobodny*.
2. **Instrukcje obszaru roboczego** (każda akcja, każde repozytorium
   [obszaru roboczego](workspaces.md#instrukcje-ai-obszaru-roboczego)): dla wszystkich akcji
   i opcjonalnie dla każdej z osobna. Edytowane w formularzu Edytuj obszaru roboczego.
3. **Instrukcje repozytorium** (każda akcja, tylko to repozytorium), z
   *Połącz z globalnymi*: **Dodaj do globalnych** (po nich) lub **Zastąp
   globalne** (zamiast nich).
4. **Instrukcje dla akcji** (*Generowanie opisów commitów*, *Wyjaśnianie zmian*,
   *Generowanie opisów stashy*, *Pull request*, *Kompozytor commitów*,
   *Rozwiązywanie konfliktów*, *Podział commita*, *Przegląd kodu*), każdy blok z własnymi znacznikami w swoim edytorze (np.
   *Format conventional commits*, *Limit 50 znaków*) i własną listą **Profil**
   (`Domyślny (<name>)` idzie za profilem domyślnym).

`Enter` na wierszu *Własne instrukcje* otwiera wielowierszowy edytor (`Ctrl+S`
zapisuje, `Esc` anuluje). Nad tekstem wiersz *Szybkie wstawianie* zawiera znaczniki
tej sekcji: kliknij jeden, by wstawić jego zdanie w miejscu kursora. Z klawiatury `Tab`
przechodzi do wiersza, `Left`/`Right` wybierają znacznik, `Enter` lub `Space` wstawia go
i wraca do tekstu (`Tab` lub `Esc` wracają bez wstawiania). Znacznik, którego zdanie
jest już w instrukcjach, jest przyciemniony, a ponowne wstawienie nic nie robi.
Tekst to pełny edytor: kliknięcie ustawia kursor, przeciągnięcie lub `Shift`+strzałki
zaznaczają, `Ctrl+A` zaznacza wszystko, `Ctrl+C` / `Ctrl+X` kopiują i wycinają
zaznaczenie, `Ctrl`+strzałki przeskakują po słowach, a `Home` / `End`, `PageUp` /
`PageDown` i kółko myszy przesuwają po długim tekście. Sama strona pokazuje tylko
podgląd każdej instrukcji.

![Strona AI: instrukcje globalne, potem bloki Generowanie opisów commitów i Wyjaśnianie zmian z listą profilu i własnymi instrukcjami](../images/settings-ai.png)

**Dla repozytorium**: *Ustawienia > Ustawienia repozytorium > AI* (przy otwartym
repozytorium) zawiera instrukcje repozytorium oraz, dla każdej akcji, profil
i instrukcje nadpisujące globalne. Puste pole używa ustawienia globalnego
(pokazanego w nim wyszarzonego); **Przywróć globalne** usuwa nadpisanie.
Zapisywane w `.gitpoulpi/repo-state.toml` repozytorium i respektowane także
przez CLI.

![Strona AI repozytorium: Instrukcje repozytorium z listą Połącz z globalnymi, potem profil i instrukcje dla każdej akcji](../images/settings-repo-ai.png)

### Jak poziomy się łączą

Kolejność wysyłania to: wbudowany prompt, **globalne**, **obszaru roboczego**, **repozytorium**,
potem własne instrukcje akcji. Tryb poziomu decyduje, co dzieje się z tymi
powyżej:

- **Dodaj** (domyślnie): tekst poziomu jest dodawany po bardziej ogólnych.
- **Zastąp**: tekst poziomu jest używany *zamiast* każdego bardziej ogólnego poziomu
  (globalnego, a gdy zastępuje repozytorium, także obszaru roboczego). Poziom bez tekstu
  nigdy niczego nie zastępuje.

Własne instrukcje akcji idą tym samym tropem: globalne dla tej
akcji, potem obszaru roboczego dla tej akcji (dodane po nich albo je zastępujące),
a własne nadpisanie akcji w repozytorium wygrywa z oboma. Używany obszar roboczy to aktywny
obszar roboczy okna, gdy zawiera repozytorium, w przeciwnym razie pierwszy obszar roboczy, który je zawiera; w
wierszu poleceń to ten z `GITPOULPI_WORKSPACE`, w przeciwnym razie pierwszy.

Aby zobaczyć dokładnie, co jest wysyłane, poziom po poziomie, uruchom
`gitpoulpi ai prompt --action <action> --repo .` (`base`, `global`, `workspace`,
`repository`, `task` i końcowy `system_prompt`). Nie wywołuje żadnego modelu i
działa przy wyłączonym AI. Instrukcje obszaru roboczego ustawisz ze skryptu poleceniem
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (albo `--clear`).

### Zmienne promptu

Każdy tekst instrukcji (globalnych, obszaru roboczego, repozytorium, akcji) może zawierać zmienne
`${name}`. Są zastępowane swoją wartością tuż przed wysłaniem promptu
(zapisany tekst zachowuje `${...}`). Nazwa spoza tabeli jest wysyłana tak, jak
napisana, a edytor ostrzega o niej; napisz `$${` dla dosłownego `${`.

| Zmienna | Zamienia się na | Przykład |
|---|---|---|
| `${uilang}` | Język interfejsu, pod angielską nazwą | `French` |
| `${repo}` | Nazwa folderu repozytorium | `gitpoulpi` |
| `${branch}` | Bieżąca gałąź | `main` |
| `${ref}` / `${commit}` | Krótki sha commita, którego dotyczy akcja (*Wyjaśnij*), w przeciwnym razie HEAD | `4c0443d` |
| `${commit_subject}` | Temat tego commita | `feat: add search` |
| `${date}` | Data lokalna, ISO | `2026-10-03` |
| `${time}` | Czas lokalny, `HH:MM` | `14:05` |
| `${datetime}` | Data i czas lokalny | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | Domyślne zdalne repozytorium (`origin`, w przeciwnym razie pierwsze) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (inny host) lub `none` | `github` |
| `${files}` | Liczba plików w diff akcji | `3` |
| `${action}` | Wykonywana akcja | `commit_message` |

W edytorze instrukcji: wpisz `${`, a lista podręczna pokaże pasujące zmienne
(`Up`/`Down` wybierają, `Enter` lub `Tab` wstawia, `Esc` ją zamyka); albo użyj
listy **Zmienne** w lewym dolnym rogu (`Ctrl+T` lub *AI: Wstaw zmienną promptu*
w palecie poleceń), która wstawia w miejscu kursora. Każda pozycja ma opis,
pokazywany na liście i w wierszu nad przyciskami, gdy ją wskażesz lub
wyróżnisz.

## Co potrafi

| Akcja | W oknie | CLI |
|---|---|---|
| **Opis commita** | Dodaj zmiany do poczekalni, naciśnij `✦` na prawo od pola Podsumowanie, `Ctrl+G i` albo polecenie palety `AI: Wygeneruj opis commita`. Podsumowanie i opis są wypełniane do edycji; nic nie jest zatwierdzane. | `ai commit-message` (wypisuje), `commit --ai` (zatwierdza) |
| **Wyjaśnianie zmian** | `✦` w prawym górnym rogu panelu zmian roboczych (`Ctrl+G w`: zaznaczony commit, w przeciwnym razie zmiany robocze) albo pozycja menu commita *Wyjaśnij zmiany (AI)*. Okno: strzałki przewijają, `c` kopiuje, `Esc` zamyka. | `ai explain [rev] [--wip]` |
| **Opis stasha** | `Ctrl+G m` otwiera okno stasha; jego przycisk Opis AI (`Ctrl+G` w oknie) pyta model. Polecenie palety `AI: Wygeneruj opis stasha` robi jedno i drugie. | `stash push --ai` |
| **Tytuł i opis pull requesta** | *Wygeneruj tytuł i opis* w formularzu tworzenia pull requesta (commity i diff gałęzi są czytane lokalnie). | |
| **Rozwiązywanie konfliktów merge** | W edytorze konfliktów merge: przycisk paska narzędzi `✦ Zasugeruj`, menu prawego przycisku konfliktu *✦ Zasugeruj rozwiązanie (AI)*, `Alt+I` albo polecenie palety `AI: Zasugeruj rozwiązanie konfliktu merge`. Sugestia zastępuje region *bieżącego konfliktu* w Wyniku jako jedną edycję (oznaczoną jako edytowana, jeden krok cofania); nic nie jest zapisywane ani dodawane do poczekalni, dopóki nie zapiszesz. | `ai resolve-conflict <path> [--conflict N]` (wypisuje) |
| **Przegląd kodu** | W [przeglądzie pull lub merge requesta](review.md#-szkic-przeglądu-ai): przycisk `✦ Szkic przeglądu...`, `s` albo polecenie palety `Przegląd: ✦ Przygotuj szkic przeglądu...` (otwiera opcje przeglądu, `Enter` uruchamia). Model czyta diff, opis i istniejące komentarze żądania (oraz domyślnie pliki `CLAUDE.md` i `AGENTS.md` projektu: zob. [Kontekst przeglądu](#kontekst-przeglądu)) i proponuje komentarze; każdy przyjmujesz, edytujesz albo odrzucasz. Nic nie trafia do hosta, zanim nie wyślesz przeglądu. | `hosting review suggest <n>` (wypisuje JSON, niczego nie publikuje), `ai review-context` (wartości domyślne) |
| **Podział commita** | Menu commita *Podziel commit (AI)...* albo polecenie palety `AI: Podziel zaznaczony commit...`. Edytor podziału otwiera się z podziałem zaproponowanym przez model; zob. niżej. | `ai split-plan <rev>` (wypisuje) |
| **Skrypt przepisania historii** | W [warsztacie przepisywania historii](history-rewrite.md#napisz-skrypt-z-ai): przycisk `✦ Napisz z AI...` edytora skryptu albo polecenie palety `Przepisanie: Napisz skrypt z AI...`. Okno pokazuje dokładnie, co zostanie wysłane; skrypt ląduje w edytorze, oznaczony, i nigdy nie jest uruchamiany ani stosowany za ciebie. | `ai rewrite-script "<instruction>" [--sample N]` (wypisuje) |
| **Reguły przepisania historii** | W [warsztacie przepisywania historii](history-rewrite.md#utwórz-reguły-z-ai): przycisk `✦ Utwórz reguły z AI...`, `Alt+I` albo polecenie palety `Przepisanie: Utwórz reguły z AI...`. Wielowierszowe okno pokazuje dokładnie, co zostanie wysłane; reguły i ustawienia trafiają do warsztatu, a nic nie jest podglądane ani stosowane za ciebie. | `ai rewrite-rules "<description>" [--context]` (wypisuje) |
| **Nazwa worktree** | W [oknie Utwórz worktree](worktrees.md#nazywanie-z-ai): przycisk `✦ Nazwij z AI` lub `Alt+A` otwiera pole, w którym opisujesz funkcję, błąd lub zadanie; model nazywa worktree i jego gałąź zgodnie z nazewnictwem gałęzi w repozytorium, a obie nazwy trafiają do pól okna, do poprawienia. Wysyłany jest tylko twój opis i nazwy lokalnych gałęzi, więc nie ma okna „co zostanie wysłane”. | `ai worktree-name --describe "<task>"` (wypisuje) |
| **Odpowiedź na hoście** i **Opis zgłoszenia** | Przycisk **Pisz z AI** (`Alt+E`) każdego edytora Markdown, który wysyła na hosta: komentarz, odpowiedź lub edycja w rozmowie, komentarz albo podsumowanie przeglądu, opis nowego zgłoszenia. Szkic trafia do pola; nic nie jest publikowane. Zob. [Pomoc w pisaniu na hoście](#pomoc-w-pisaniu-na-hoście). | `ai hosting-reply` (wypisuje szkic) |
| **Kompozytor commitów** | Menu wiersza WIP *Skomponuj commity (AI)...*, `Ctrl+G C`, polecenie palety `AI: Skomponuj commity ze zmian roboczych...` | |

`Esc` anuluje trwające żądanie (proces agenta lokalnego jest zabijany).

### Rozwiązywanie konfliktów merge

Model dostaje ścieżkę i język pliku, bloki bazowy, bieżący (A) i przychodzący (B)
konfliktu z etykietami pokazywanymi przez edytor, do 20 linii już scalonego
tekstu przed nim i po nim oraz twoje instrukcje (globalne, repozytorium
i te z *Rozwiązywanie konfliktów* w Ustawieniach > AI). Odpowiada rozwiązanym
tekstem tego jednego konfliktu w bloku kodu; odpowiedź ze znacznikami konfliktu
albo dodatkowym tekstem zamiast kodu jest odsyłana raz, potem odrzucana. Konflikt powyżej
40 KB (`ai.limits.conflict_bytes`, *Ustawienia > AI > Limity*) jest wstrzymywany za
ostrzeżeniem, które podaje jego rozmiar i limit, z przyciskami *Wyślij mimo to*
(tylko to żądanie, nadal oczyszczone kontrolami prywatności), *Otwórz ustawienie*
i *Anuluj*; limit to własne zabezpieczenie GitPoulpi, więc podnieś go, gdy twój
model ma zapas. Elementy sterujące są ukryte, gdy AI jest
wyłączone, a sugestia nigdy niczego nie zapisuje, nie dodaje do poczekalni ani
nie rozwiązuje: przejrzyj Wynik, edytuj go i zapisz jak zwykle.

![Formularz commita po przycisku z iskrą: podsumowanie i opis napisane przez model, gotowe do edycji](../images/ai-commit-message.png)

![Okno Wyjaśnij zmiany robocze: wyjaśnienie i lista rzeczy do sprawdzenia](../images/ai-explain.png)

### Kompozytor commitów

Model dzieli zmiany robocze na kilka commitów (każdy zmieniony plik w dokładnie
jednym; nieprawidłowy plan jest odsyłany raz, potem odrzucany). Dopóki pliki
mają konflikty, o nic się nie pyta: okno wypisuje je i otwiera widok rozwiązywania
konfliktów, bo kompozytor potrzebuje drzewa bez konfliktów. Plan
przeglądasz, zanim cokolwiek zostanie zatwierdzone:

![Przegląd kompozytora: cztery proponowane commity z plikami oraz przyciski Resetuj, Anuluj i Utwórz 4 commity](../images/composer.png)

| Klawisz | Mysz | Działanie |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | kliknięcie wiersza | porusza się po commitach i plikach |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | strzałki, przeciągnij i upuść | przesuwa commit; przenosi plik do innego commita |
| `Right` / `Left`, `*` | strzałka wiersza | rozwija / zwija commit, wszystkie (`ui.composer_files` ustawia początek) |
| `e` / `Enter` | ołówek | edytuje opis |
| `d` / `Delete` | `drop` / `przywróć` | pomija commit (jego pliki zostają niezatwierdzone) |
| `n` | `+ własny commit` | przenosi plik do osobnego commita |
| `r` | `Resetuj` | wraca do planu modelu |
| `Ctrl+S` / `a` | `Utwórz N commitów` | tworzy je, po kolei |
| `Esc` | `Anuluj` | wychodzi, nic nie zrobiono |

Tworzenie dodaje do poczekalni pliki każdego commita i zatwierdza je po kolei;
drzewo robocze nigdy nie jest ruszane. Plik dodany do poczekalni tylko
częściowo jest zatwierdzany w całości. Jeśli krok się nie powiedzie, proces się
zatrzymuje i podaje, ile commitów utworzono.

### Podział commita

Model czyta zmiany commita, ponumerowane tak samo jak w
[edytorze podziału](log-and-history.md#podział-commita), i proponuje kilka commitów,
każdy z własnym opisem; potrafi grupować hunki i pojedyncze linie. Odpowiedź, która
pomija jakąś zmianę, przypisuje którąś dwa razy albo wskazuje coś, co nie istnieje,
jest odrzucana. Edytor podziału otwiera się w obu przypadkach: z gotową propozycją
albo, gdy model zawiódł, z powodem w notatce, abyś podzielił commit ręcznie.
Nic nie zostanie podzielone, dopóki nie naciśniesz **Podziel** (`Ctrl+S`); jeśli
podział ci nie odpowiada, najpierw przenieś zmiany, zmień opisy commitów albo dodaj
nowy. Profil i instrukcje to te z pozycji *Podział commita* w Ustawieniach > AI.

### Skrypt przepisania historii

Opisujesz, co przepisanie ma zrobić („przenieś każdy commit zrobiony nocą na
następny ranek”, „zamień mój stary e-mail na nowy, tylko przed 2024”), a model
pisze [skrypt Lua](history-rewrite.md#skrypty-lua) dla reguły Skrypt, z nagłówkiem,
`rewrite_commit(c, ctx)` i funkcjami pomocniczymi piaskownicy. Demon kompiluje
odpowiedź; jeśli się nie kompiluje, błąd wraca do modelu jeden raz. To, co zostaje,
widać w edytorze: przeczytaj, przetestuj, potem zrób podgląd. Szczegóły i okno
są w części [Przepisywanie historii](history-rewrite.md#napisz-skrypt-z-ai). Profil i
instrukcje to te z pozycji *Skrypt przepisania historii* w Ustawieniach > AI.

### Reguły przepisania historii

Opisujesz własnymi słowami, jak przepisać historię („zamień mój stary e-mail a@b.org na
c@d.org”, „usuń secrets.txt z całej historii”), a model proponuje zakres, kilka ustawień
i listę [reguł przepisania](history-rewrite.md#utwórz-reguły-z-ai) w słownictwie samego
warsztatu. **Tylko przygotowuje**: odpowiedź wypełnia warsztat, nic nie jest podglądane ani
stosowane, a wpis, którego nie da się użyć, trafia na listę z powodem, podczas gdy poprawne
zostają. Reguła Skrypt nigdy nie powstaje w ten sposób. Opcjonalnie wysyłane są też zakres,
nazwy gałęzi i tagów oraz autorzy najnowszych commitów (pole wyboru, domyślnie wyłączone,
widoczne w panelu „co zostanie wysłane”). Profil i instrukcje to te z pozycji *Reguły
przepisania historii* w Ustawieniach > AI.

### Pomoc w pisaniu na hoście

*Odpowiedź na hoście* przygotowuje komentarz, odpowiedź w wątku, komentarz do linii kodu lub podsumowanie
przeglądu; *Opis zgłoszenia* przygotowuje opis nowego zgłoszenia. **AI tylko pisze szkic.** Tekst trafia do
edytora (albo jest wypisywany przez CLI), abyś sam go przeczytał, poprawił i wysłał. Z tej funkcji nie
prowadzi żadna ścieżka kodu do zapisu na hoście: nie może niczego opublikować, edytować, rozwiązać ani usunąć,
odpowiedź CLI i MCP zawiera `"posted": false`, a lokalny agent działa tylko do odczytu (niżej). Tekst opuszcza
GitPoulpi wyłącznie przez własny przycisk **Wyślij** (`Ctrl+Enter`) lub **Komentarz** edytora.

**W edytorze Markdown.** Gdy AI jest włączone, każdy edytor, którego tekst trafia na hosta (komentarz, odpowiedź
albo edycja w [widoku rozmowy](hosting-integration.md#5c-rozmowy), komentarz do przeglądu, podsumowanie w oknie
Zakończ przegląd, opis nowego zgłoszenia i opis pull requesta), ma po prawej stronie paska trybów przycisk
**Pisz z AI** (`Alt+E`, ten sam klawisz wszędzie). Otwiera on panel pod paskiem:

- pole *Co chcę w tej odpowiedzi*, wpisywane w chwili użycia (`Enter` pisze szkic). Jego tekst zastępczy
  podaje przykład tego rodzaju („zgódź się i poproś o test, który zawodzi”). Lista **Ostatnie** zachowuje
  wcześniejsze instrukcje (kliknij jedną albo użyj `Up` / `Down` w polu; zapisywane w `ai-hints.toml`);
- *Popraw mój szkic*, zaznaczone, gdy pole już zawiera tekst: AI zaczyna od niego zamiast pisać nowy;
- *Pozwól AI czytać repozytorium*, **niezaznaczone za każdym razem**, chyba że włączysz jego wartość domyślną
  (niżej);
- **Szkic**, **Zatrzymaj** (także `Esc`) i **Zamknij** oraz jedna linia na postęp lub błąd.

Zwykłe okno „Co zostanie wysłane” (`[ai.sent_preview] hosting_reply`, `issue_body`,
`pull_request_description`) pokazuje żądanie, zanim wyjdzie. Oprócz tekstu systemowego i użytkownika pokazuje
listę **Kontekst repozytorium**, gdy repozytorium jest czytane: każda część z rozmiarem, suma na tle limitu
(*31.2 KB z 48 KB*), co przycięto, by się zmieściło, co pominięto jako wrażliwe i które części użyto ponownie.
Dla lokalnego agenta mówi, że agent sam czyta repozytorium narzędziami tylko do odczytu, nazywa je, podaje
folder roboczy i to, czy sesja agenta jest nowa, czy kontynuowana, a każde okno kończy się zdaniem *AI pisze
tylko szkic. Nic nie jest publikowane, dopóki sam tego nie wyślesz.* Szkic zastępuje tekst jednym krokiem
cofania, a edytor przełącza się na Podgląd, byś przeczytał go przed wysłaniem; `Ctrl+Z` oddaje twój tekst. Opis
pull requesta jest pisany z commitów i diffu gałęzi i przyjmuje tę samą instrukcję; pole repozytorium jest tam
wyszarzone („nie dla tego tekstu”).

Co dostaje model: tytuł, stan i opis elementu, wątek, na który odpowiadasz, ostatnie komentarze (od najnowszych,
w granicach `ai.limits.hosting_reply_context_bytes`, domyślnie 49152), znaczniki, które przyjmuje host (więc
odpowiedź dla Jiry wraca jako Markdown, który GitPoulpi konwertuje), to, czego chcesz w tej odpowiedzi, a przy
poprawianiu własnego tekstu twój szkic. Wszystko, co napisali inni, jest w żądaniu oznaczone jako dane
niezaufane, więc instrukcja ukryta w komentarzu nie jest wykonywana. Powyżej limitu nic nie jest ucinane po
cichu: dostajesz pytanie, a najstarsze komentarze są pomijane najpierw (wątek, na który odpowiadasz, jest
zachowany do połowy budżetu); CLI zgłasza cięcie w `truncated`, a `--send-all` je znosi.

**Własny prompt bazowy.** Instrukcje, od których zaczynają obie akcje, można edytować: *Ustawienia > AI >
Odpowiedzi na hoście > Prompt bazowy* (`ai.hosting_reply.base_prompt`). Puste używa wbudowanego promptu, który
prosi tylko o tekst, w znacznikach hosta i w języku rozmowy, chyba że twoja wskazówka mówi inaczej. Twoje
instrukcje globalne, obszaru roboczego i repozytorium nadal po nim następują, a tekst innych osób pozostaje
oznaczony jako niezaufany, cokolwiek tam wpiszesz. `ai prompt --action hosting_reply` wypisuje warstwy.

#### Dostęp do repozytorium

**Domyślnie wyłączony i zaznaczany przy każdym użyciu.** Czytanie repozytorium daje znacznie lepsze odpowiedzi
na „czy ta awaria występuje w naszym kodzie?”, ale wysyła więcej, więc to wybór, który robisz za każdym razem:
*Pozwól AI czytać repozytorium* zaczyna niezaznaczone w każdym polu. `ai.hosting_reply.read_repository`
(*Domyślnie pozwalaj AI czytać repozytorium*, wyłączone) sprawia tylko, że pole zaczyna zaznaczone;
`--read-repo` to zaznaczenie w CLI. Wybór jest wyszarzony, z powodem po jego aktywacji, w trzech przypadkach:
repozytorium **nie jest sklonowane** (rozmowa innego repozytorium z Radaru: nie ma czego czytać), wybrany lokalny
agent **nie ma trybu tylko do odczytu**, któremu GitPoulpi ufa, albo tekst **nie jest pisany z repozytorium**
(opis pull requesta). Włącza go wyłącznie twoje własne zaznaczenie.

Co się dzieje potem, zależy od profilu:

| Profil | Co czyta repozytorium | Co jest wysyłane |
|---|---|---|
| **API** (zgodne z OpenAI, Anthropic) | GitPoulpi, przez demona, tylko do odczytu | Kontekst, który buduje i daje modelowi: mapa repozytorium (drzewo najwyższego poziomu i ścieżki wokół dopasowanych plików), pliki wymienione w rozmowie (cały plik albo okno 40 linii wokół linii lub symbolu), trafienia wyszukiwania identyfikatorów i komunikatów błędów z rozmowy, a dla pull requesta jego diff. W granicach `ai.limits.hosting_reply_*` (kontekst 49152, plik 8192, mapa 6144, 20 trafień na termin, 8 terminów), cięty w kolejności diff, pliki, trafienia, mapa i zgłaszany. Pliki wyglądające na wrażliwe są pomijane, a sekrety w pozostałych maskowane (zob. [Prywatność](#prywatność)). |
| **Lokalny agent** (Claude Code, Codex), headless | Sam agent, w folderze repozytorium | Nic nie jest budowane: agent jest uruchamiany w repozytorium z **narzędziami tylko do odczytu** i czyta, czego potrzebuje. Claude Code dostaje `Read`, `Grep` i `Glob`, a `Bash`, `Edit`, `Write`, `NotebookEdit`, `WebFetch` i `WebSearch` są mu odmówione, bez ładowania serwera MCP (własny serwer projektu mógłby publikować); Codex działa w swojej piaskownicy tylko do odczytu. Dane uwierzytelniające hostingu ze środowiska (`GH_TOKEN`, `GITHUB_TOKEN`, `GITLAB_TOKEN`, zmienne Jira, Linear, Azure DevOps i Bitbucket, `GIT_ASKPASS`, `SSH_AUTH_SOCK`) są usuwane z jego środowiska, więc nawet źle skonfigurowany agent nie może niczego opublikować ani wypchnąć. |

Agenta, dla którego GitPoulpi nie ma potwierdzonego trybu tylko do odczytu (oraz żadnego agenta w trybie
uruchomienia w terminalu interaktywnym), nie da się dopuścić: wybór pozostaje wyszarzony z komunikatem *Ten agent
nie ma trybu tylko do odczytu, któremu GitPoulpi ufa.*

**Pamięć podręczna i Odśwież kontekst.** To, co zebrano, jest używane ponownie, aby następna odpowiedź była
szybka i tania:

- Zbudowany kontekst profilu API jest przechowywany dla repozytorium przez
  `ai.hosting_reply.context_ttl_minutes` (*Przechowuj kontekst repozytorium przez (minuty)*, domyślnie 30).
  Kluczem są repozytorium, jego `HEAD`, wybór i limity, a jego części są przechowywane osobno, więc następne
  zgłoszenie tego samego repozytorium odbudowuje tylko te części, których nie dzieli. W oknie pamięci podręczne
  żyją w pamięci (ostatnie 8 kontekstów); CLI trzyma je w plikach w swoim prywatnym folderze stanu, więc drugie
  `ai hosting-reply --read-repo` używa tego, co zbudowało pierwsze.
- Nowy commit w wyewidencjonowanej gałęzi (`HEAD` się przesuwa), zmiana limitów lub upływ czasu zaczynają od
  nowa. Tak samo **Odśwież kontekst**, przycisk w panelu, gdy repozytorium jest czytane, albo
  `--refresh-context` w CLI. Linia pod polem mówi, co zrobił ostatni szkic: *Kontekst zbudowany: 5 części,
  31.2 KB, HEAD a1b2c3d*, *Kontekst z pamięci (4 z 5 części, HEAD a1b2c3d)*, *Cache dostawcy: ponownie 12k
  tokenów*, *Agent czytał repozytorium (HEAD a1b2c3d)*, *Sesja agenta rozpoczęta* lub *Sesja agenta
  kontynuowana*.
- **Cache promptów Anthropic.** Dla profilu Anthropic żądanie jest wysyłane z promptem bazowym i instrukcjami
  jako jednym buforowanym blokiem i kontekstem repozytorium jako drugim, a po nich twoja rozmowa, wskazówka
  i szkic, więc druga odpowiedź na tym samym kontekście jest czytana z pamięci podręcznej dostawcy i kosztuje
  ułamek. Liczniki pamięci podręcznej w odpowiedzi to to, co pokazuje *Cache dostawcy: ponownie N tokenów*.
  Serwery zgodne z OpenAI dostają ten sam stabilny porządek żądania, z którego korzysta ich automatyczne
  buforowanie prefiksów; GitPoulpi nie wysyła niczego dodatkowo.
- **Agent zachowuje sesję.** Przy włączonym **Zachowaj sesję agenta między odpowiedziami**
  (`ai.hosting_reply.reuse_agent_session`, domyślnie włączone) i zaznaczonym *Pozwól AI czytać repozytorium*
  lokalny agent, który to obsługuje (dziś Claude Code), jest uruchamiany z id sesji, a potem wznawiany przez nie,
  więc to, co przeczytał z repozytorium, zostaje w jego własnej historii, a następna odpowiedź startuje
  szybciej. Inni agenci działają jednorazowo. Sesja jest przypisana do repozytorium, `HEAD` i akcji; nowy commit
  lub **Odśwież kontekst** zaczyna nową; przy wyłączonej opcji lub niezaznaczonym polu każda odpowiedź to
  jednorazowe uruchomienie, które niczego nie zachowuje. Sesja żyje w własnym magazynie agenta, GitPoulpi
  zachowuje tylko jej id (w pamięci w oknie, w pliku tylko dla użytkownika w CLI).

```
gitpoulpi ai hosting-reply --conversation conversation.json --hint "say I will look tomorrow"
gitpoulpi ai hosting-reply --conversation conversation.json --reply-to <thread id> --draft "thx will do"
gitpoulpi ai hosting-reply --conversation conversation.json --read-repo --hint "is this our bug?"
gitpoulpi ai hosting-reply --issue-title "Crash on empty config" --hint "steps to reproduce"
```

`--conversation` przyjmuje plik, `-` dla standardowego wejścia albo sam JSON: ten, który wypisuje
`hosting issue comments --json` lub `hosting review conversation --json` (starsza strona z `header` i `entries`
też jest akceptowana), więc działa `gitpoulpi hosting issue comments 7 --json | gitpoulpi ai hosting-reply
--conversation -`; `--item <ref>` sam pobiera rozmowę (`7` lub `#7` to zgłoszenie, `!7` to pull request). Z
`--read-repo` flagi `--pr-base REV --pr-head REV` wkładają do kontekstu diff pull requesta; `context` w
odpowiedzi mówi, które części przeczytano, czy użyto ich ponownie, tokeny pamięci podręcznej dostawcy i sesję
agenta. Narzędzie MCP `ai_hosting_reply` to to samo polecenie i jest tylko do odczytu.

## Dokąd trafia klucz

Klucz API (OpenAI, Anthropic, OpenRouter, serwer zgodny z OpenAI) jest związany z adresem, dla którego go zapisano: z originem base URL profilu, zapisanym w `trust.toml` w prywatnym folderze danych dostępnym tylko dla użytkownika. Jeśli base URL później wskazuje gdzie indziej, żądanie jest odrzucane, zanim cokolwiek zostanie wysłane, a klucz zostaje tam, gdzie jest. Uruchom `gitpoulpi ai trust <profile> --accept-origin <url>` (albo zaakceptuj to w aplikacji), by celowo przenieść klucz na nowy adres.

- Zwykłe `http://` jest dozwolone na tym komputerze (Ollama, LM Studio na `localhost`); gdziekolwiek indziej wymaga `gitpoulpi ai trust <profile> --allow-insecure-http on`.
- Serwer na własnym hostingu, którego certyfikat nie łączy się ze znanym urzędem, jest akceptowany tylko dla odcisku klucza, który przypniesz przez `--accept-certificate <sha256>`; uzgadnianie TLS nadal dowodzi, że serwer ma ten klucz.
- Profil, który miał klucz, zanim ta kontrola istniała, jest zapisywany po cichu, z jego bieżącego base URL, przy pierwszym użyciu.
- Lokalny agent (`claude -p` i podobne) nie ma klucza ani HTTP, więc nie ma czego wiązać.

## Kontekst przeglądu

Szkic przeglądu kodu (`✦` lub `hosting review suggest`) czyta diff żądania, tytuł, opis i istniejące
komentarze. Może też dostać własne konwencje projektu w bloku **Project conventions** umieszczonym przed
diffem, a lokalny agent może pracować wewnątrz repozytorium. Wartości domyślne:

| Ustawienie | Domyślnie | Znaczenie |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: decyduje *Folder roboczy* profilu AI. `yes`: lokalny agent uruchamia się w repozytorium na czas przeglądu, cokolwiek mówi profil. `no`: nie tam (profil ustawiony na *Repozytorium* używa wtedy dedykowanego folderu). Profile HTTP nie mają folderu roboczego, więc dla nich nic się nie zmienia. |
| `include_claude_md` | włączone | Przekaż modelowi plik `CLAUDE.md` repozytorium. |
| `include_agents_md` | włączone | Przekaż modelowi plik `AGENTS.md` repozytorium. |
| `extra_file` | brak | Jeszcze jeden plik: ścieżka względem katalogu głównego repozytorium, ścieżka bezwzględna lub link. |

Są warstwowane jak instrukcje, przy czym wygrywa najbardziej szczegółowa wartość, pole po polu: globalna
tabela `[ai.review]`, potem własne wartości repozytorium (nieustawione pole dziedziczy), potem wybory
jednego przebiegu. Repozytorium, które nie jest sklonowane, trzyma własne wartości pod `host/project` w
`remote-reviews.toml`, obok swoich szkiców przeglądu.

Repozytorium może też **wyłączyć** globalny plik dodatkowy: jego `no_extra_file = true` (pole wyboru **Brak
dodatkowego pliku** w *Ustawieniach repozytorium > AI*, w oknie ✦ Opcje przeglądu oraz flaga `--no-extra-file`
polecenia `ai review-context`) oznacza „brak dodatkowego pliku dla tego repozytorium”, cokolwiek wskazuje
`[ai.review]`. **Puste** pole **Dodatkowy plik** to co innego: dziedziczy plik globalny. Plik repozytorium i
przełącznik wykluczają się (ustawienie jednego czyści drugie), a przełącznik wygrywa, jeśli ręcznie
edytowany plik zawiera oba.
Starsze `extra_file = ""` repozytorium nadal oznacza brak pliku.

Skąd biorą się pliki:

- `CLAUDE.md`, `AGENTS.md` i względny plik dodatkowy są czytane z **katalogu głównego** repozytorium w
  gałęzi, do której zmierza żądanie, nigdy z katalogu roboczego (który może być na innej gałęzi lub mieć
  lokalne zmiany). Z klonem: z commita bazowego żądania; bez niego: z API hosta.
- Ścieżka bezwzględna jest czytana z tego komputera.
- Link do pliku na hoście kodu (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea i Forgejo `.../src/branch/<branch>/<path>` albo link do surowego pliku) jest czytany przez profil
  integracji pasujący do hosta, więc prywatne repozytorium działa z tokenem, który już podałeś. Link, do
  którego nie pasuje żaden profil, oraz każdy inny adres są pobierane anonimowo. Nazwy gałęzi z ukośnikiem
  nie da się odczytać z linku: użyj ścieżki albo linku na gałęzi bez ukośnika.
- Brakujący `CLAUDE.md` lub `AGENTS.md` jest po cichu pomijany; brakujący plik dodatkowy jest zgłaszany
  (`context.problems` w JSON-ie CLI).
- Każdy plik jest obcinany przy **8 KB**, a wszystkie razem przy **16 KB**, z widoczną notką
  `[CLAUDE.md cut: showing ... of ... bytes]` w tym, co czyta model.

Gdy lokalny agent działa w repozytorium, prompt mówi mu, że pliki w checkoutcie mogą nie zgadzać się z
głową żądania i że rozstrzygający jest diff.

W widoku przeglądu `s` otwiera okno **✦ Opcje przeglądu** z wypełnionymi wartościami: repozytorium nad
globalnymi. Zmień je tylko na ten przebieg albo zaznacz *Zapisz jako domyślne dla tego repozytorium*, by je
zachować (zob. [Opcje przeglądu](review.md#opcje-przeglądu)). Wartości domyślne można też edytować w
*Ustawieniach > AI* (blok *Przegląd kodu*, cztery wiersze) i dla repozytorium w *Ustawieniach repozytorium > AI*,
gdzie każdy wiersz może *użyć globalnych*, a przycisk *Przywróć globalne* porzuca własne wartości
repozytorium; wyszukiwarka Ustawień je znajduje (`Dołącz AGENTS.md`, `Dodatkowy plik`). Wszystko, czego model nie
mógł dostać (brakujący plik dodatkowy) lub co obcięto, żeby się zmieściło, widać na karcie szkicu.

Z wiersza poleceń:

```sh
gitpoulpi ai review-context                      # globalne wartości domyślne i to, co dostaje przegląd
gitpoulpi ai review-context --agents-md off      # zapisz globalną wartość domyślną
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # znów podążaj za globalnymi
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` nie wymaga modelu i działa przy wyłączonym AI. `ai prompt --action code_review` wypisuje
efektywny kontekst jako `review_context`. Te same cztery ustawienia to `ai.review.*` w
`config.toml` ([opis](config-reference.md#ai-wbudowane-ai-domyślnie-wyłączone)).

## Limity wejścia

Każde wejście, które GitPoulpi umieszcza w żądaniu, ma limit rozmiaru: ustawienie
w `[ai.limits]` (*Ustawienia > AI > Limity*). Limity to własne zabezpieczenie
GitPoulpi (okno kontekstu modelu różni się zależnie od modelu): podnieś je, gdy
twój model ma zapas, obniż, by żądania były małe.

| Ustawienie (`ai.limits.`) | Domyślnie | Co ogranicza |
|---|---|---|
| `conflict_bytes` | 40000 | Bloki bazowy, bieżący i przychodzący jednego konfliktu merge |
| `diff_bytes` | 60000 | Diff opisu commita, wyjaśnienia, nazwy stasha, opisu pull requesta, planu commitów lub podziału; cały diff przeglądu kodu |
| `review_file_bytes` | 12000 | Diff jednego pliku w przeglądzie kodu |
| `review_text_bytes` | 4000 | Opis przeglądanego pull requesta |
| `review_threads_bytes` | 6000 | Istniejące komentarze przeglądu |
| `review_context_file_bytes` | 8192 | Jeden plik kontekstu projektu w przeglądzie (`CLAUDE.md`, `AGENTS.md`, dodatkowy plik) |
| `review_context_bytes` | 16384 | Wszystkie pliki kontekstu przeglądu razem |
| `hosting_reply_context_bytes` | 49152 | Element i rozmowa podawane do przygotowywanej odpowiedzi |
| `hosting_reply_file_bytes` | 8192 | Jeden plik repozytorium podawany do przygotowywanej odpowiedzi (gdy AI czyta repozytorium) |
| `hosting_reply_map_bytes` | 6144 | Mapa repozytorium podawana do przygotowywanej odpowiedzi |
| `hosting_reply_hits_per_term` | 20 | Trafienia wyszukiwania zachowywane na termin (od 1 do 200) |
| `hosting_reply_terms` | 8 | Wyszukiwane terminy (od 1 do 50) |

**Nic nie jest obcinane po cichu.** Gdy wejście przekracza limit, żądanie czeka,
a okno *Przekroczono limit AI* wypisuje, co go przekracza: która część (diff,
plik, opis, plik kontekstu), jej rozmiar, limit i ustawienie. Przegląd wypisuje
każdą przekraczającą część w jednym oknie. Przyciski, każdy z klawiszem:

- *Wyślij wszystko* (`W`): tylko to żądanie, nic nie jest obcinane. Nadal przechodzi
  przez kontrole prywatności (sekrety zastąpione, pliki wrażliwe pominięte) i, gdy
  jest włączony, panel *Co jest wysyłane*, który pojawia się potem.
- *Wyślij obcięte* (`O`): to, co przekracza limit, jest obcinane, jak zawsze;
  każda linia okna mówi, co zostaje pominięte (koniec diffa, ostatnie hunki,
  pliki, które się nie mieszczą).
- *Otwórz ustawienie* (`U`): anuluje żądanie i otwiera *Ustawienia > AI > Limity*
  na pierwszym wymienionym limicie. Fokus zaczyna się tutaj: Enter nigdy nie wysyła.
- *Anuluj* (`A`, `Esc`, krzyżyk lub kliknięcie poza oknem): niczego nie wysyła.

Wyjątkiem jest konflikt: nigdy nie jest obcinany. *Wyślij mimo to* (`M`) wysyła go
w całości jeden raz.

Nic nie uruchamia żądania AI samo, więc nie ma przebiegu w tle, który mógłby
pytać: każde żądanie to takie, które sam uruchomiłeś. Przebieg, który nie może
zapytać (CLI i MCP), obcina jak dawniej i mówi o tym; zob. niżej.

**W wierszu poleceń** nic nie może odpowiedzieć na okno, więc wejście przekraczające
limit jest obcinane jak dawniej, a odpowiedź mówi, co obcięto:
`truncated: [{part, detail?, bytes, limit, setting}]` (obecne tylko, gdy coś
obcięto; `part` to jedno z `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file`, `context_total`). `--send-all`
(alias `--no-truncate`) wysyła natomiast wszystko w całości: przy `ai` (wszystkie
jego polecenia, także `ai resolve-conflict`, gdzie konflikt powyżej limitu jest
inaczej odrzucany), przy `commit --ai`, `stash push --ai` i `hosting review suggest`.
Narzędzia MCP mają ten sam argument `send_all` i to samo pole `truncated`.

## Prywatność

- Nic nie jest wysyłane, dopóki nie naciśniesz przycisku lub nie uruchomisz
  polecenia. Żadnego AI w tle, żadnej telemetrii, żadnego konta GitPoulpi.
- Co jest wysyłane: prompt (wbudowany, potem twoje instrukcje) i diff, o który
  zapytano; dla pull requesta także tematy commitów gałęzi. Diff powyżej 60 KB
  (`ai.limits.diff_bytes`) nigdy nie jest obcinany po cichu: zob.
  [Limity wejścia](#limity-wejścia).
- Zapytanie o skrypt przepisania historii wysyła twoje polecenie i stały kontrakt
  skryptu (API piaskownicy skryptów). Autor, data i temat do 20 commitów idą razem
  z nim **tylko** po zaznaczeniu *Wyślij próbkę* w oknie (domyślnie wyłączone; CLI
  wymaga `--sample N`). Zawartość plików i wartości sekretów nigdy nie są
  wysyłane.
- Trafia tylko do profilu wybranego dla tej akcji: tego hosta API albo na stdin
  twojego polecenia agenta lokalnego (który robi wtedy to, co robi ten agent).
- Klucze API są używane przez sam proces `gitpoulpi` i nigdy nie docierają do
  demona, innych klientów okna ani klienta MCP.
- Wygenerowany tekst nigdy nie jest sam zatwierdzany, wypychany ani stosowany.

### Co jest usuwane z żądania

Każde żądanie, niezależnie od tego, która akcja je wysyła i który dostawca je odbiera
(hostowane API lub lokalny agent), przechodzi najpierw przez ten sam krok czyszczenia:

- **Sekrety są zastępowane** (`[ai.privacy] redact_secrets`, domyślnie włączone). Token
  ze znanym prefiksem, blok klucza prywatnego, JWT, hasło w URL i wiersz
  `password = value`, którego wartość wygląda na losową, są zastępowane przez
  `[REDACTED:<rule>]`. To nie jest wyczerpujące: znajduje typowe kształty (ten sam
  skaner co `rewrite scan-secrets`).
- **Wrażliwe pliki są pomijane** (`[ai.privacy] exclude_sensitive_files`, domyślnie
  włączone). Diff pliku pasującego do `[privacy] sensitive_patterns` (`.env`
  i `.env.*` z wyjątkiem plików z końcówką `.example`, `.sample` i `.template`, `*.pem`,
  `*.key`, prywatne klucze SSH, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`,
  `*.cer`, `*.der`, `*.kdbx`) nie jest częścią żądania. Kawałek podziału lub lista plików
  kompozytora, która wymienia taki plik, jest ucinana tak samo, więc plan dla niej
  może zostać odrzucony jako niepełny: dodaj do stage lub podziel ten plik sam.

Wyłączenie któregokolwiek jest twoim wyborem, a Ustawienia > AI pokazują pod nim czerwony wiersz,
dopóki jest wyłączone.

**Co jest wysyłane, zanim zostanie wysłane.** Zanim żądanie wyjdzie, TUI pokazuje
okno **Co zostanie wysłane**: instrukcje i żądanie po oczyszczeniu (ten sam
panel co w oknach przepisywania historii, przewijany, `Alt+W` go pokazuje lub ukrywa), ile
sekretów zastąpiono i które pliki pominięto (reguła i zamaskowany
podgląd, taki jak `ghp_****abcd`, nigdy wartość), pole **Nie pokazuj ponownie dla
<action>** oraz **Wyślij** / **Anuluj** (`Esc`, krzyżyk zamknięcia i kliknięcie
poza oknem anulują; nic nie wychodzi, dopóki nie wybierzesz Wyślij). Pole zapisuje
`[ai.sent_preview] <action> = false`; Ustawienia > AI > *Pokazuj, co zostanie wysłane,
przed...* mają jeden przełącznik na akcję i **Pokaż wszystko ponownie**. Dwa okna
przepisywania historii już pokazują swoje żądanie, więc nie mają przełącznika.

**W wierszu poleceń** każda odpowiedź `ai ...` (i `hosting review suggest`)
niesie `privacy: {redactions: N, excluded_files: [...]}`, a `--print-sent`
wypisuje oczyszczony tekst systemowy i użytkownika jako `{sent: [{system, user}], privacy,
dry_run: true}`, bez wywoływania modelu (wymaga `ai.enabled = true`, ale nie klucza
ani sieci).

**Ograniczenie.** Lokalny agent, którego folderem roboczym jest repozytorium
(`workdir = repository` albo *Uruchamiaj agenta w repozytorium* dla przeglądu), może
sam czytać pliki: czyszczenie obejmuje to, co GitPoulpi mu wysyła, a nie to,
co otwiera sam. Użyj folderu roboczego `dedicated`, by trzymać go poza
repozytorium.

## Rozwiązywanie problemów

| Objaw | Rozwiązanie |
|---|---|
| Nigdzie nie ma `✦` | AI jest wyłączone: *Ustawienia > AI > Włącz AI*. |
| `✦` jest szare | Nic nie jest w poczekalni (opis commita). |
| *Testuj połączenie*: nie znaleziono polecenia | **Polecenie** agenta lokalnego nie jest w `PATH`, który widzi GitPoulpi: uruchom GitPoulpi tam, gdzie działa `claude --version`, albo podaj pełną ścieżkę. |
| Agent lokalny się zawiesza lub pokazuje ekran interaktywny | W trybie uruchomienia *Headless* dodaj jego flagę headless (`-p` dla Claude Code) do **Argumentów** albo wybierz jego **Typ agenta** ponownie, by przywrócić wywołanie headless. Ekran interaktywny to właśnie to, do czego służy tryb *Interaktywny (terminal wirtualny)*. |
| "Use "Open the agent"..." | Agent zadał pytanie przy pierwszym uruchomieniu (zaufanie do folderu, logowanie). Użyj **Otwórz agenta...** w profilu, odpowiedz, zamknij agenta i spróbuj ponownie. |
| Żądanie w terminalu wirtualnym przekracza limit czasu | Komunikat kończy się ostatnimi liniami ekranu agenta: widać tam prośbę o zgodę, logowanie lub pytanie o aktualizację. Odpowiedz na nie przez **Otwórz agenta...** albo zwiększ **Limit czasu**. |
| `niepowodzenie:` 401 | Zły klucz albo klucz innego dostawcy: wpisz go ponownie. |
| `niepowodzenie:` 404 | **Model** tam nie istnieje: wyczyść go, aby użyć domyślnego. |
| `niepowodzenie:` błąd połączenia (Ollama, LM Studio) | Serwer nie działa albo **Bazowy URL** ma zły port lub brakuje w nim `/v1`. |
| Klucz pokazuje `(nie ustawiono)` po wpisaniu | Brak menedżera poświadczeń: skonfiguruj go (`git config --global credential.helper ...`) i wpisz klucz ponownie. |
| *Przekroczono limit AI* | Wejście żądania przekracza limit z `[ai.limits]`: wybierz *Wyślij wszystko*, *Wyślij obcięte*, *Otwórz ustawienie* lub *Anuluj* (zob. [Limity wejścia](#limity-wejścia)). |
| „Ten konflikt jest większy niż limit” | Konflikt przekracza `ai.limits.conflict_bytes` (domyślnie 40000). *Wyślij mimo to* wysyła go raz; *Otwórz ustawienie* prowadzi do limitu w Ustawienia > AI > Limity. |
| Zły język lub styl | Dodaj instrukcje; znaczniki Szybkie wstawianie w edytorze robią to szybko. |
| Kompozytor mówi, że plan jest nieprawidłowy | Model pominął lub powtórzył plik: spróbuj ponownie albo użyj mocniejszego modelu. |

## Starsze pliki konfiguracji

Starszy `config.toml` z `[[ai.presets]]` i `[ai.defaults]` nadal się wczytuje:

- Domyślne ustawienie wstępne każdej akcji staje się jej `[ai.actions.<action>]`
  (`system_prompt` staje się `instructions`).
- Każde stare ustawienie wstępne jest zachowane w `[[ai.legacy_presets]]`. Ustawienia > AI
  pokazuje je pod *Stare ustawienia wstępne (sprzed przeprojektowania)*, z *Usuń archiwum*.
- Dawne `[preferences.ai_presets]` repozytorium staje się nadpisaniem
  repozytorium.

Wbudowany prompt zawsze idzie teraz pierwszy. Jeśli stary prompt kopiował
prompt wbudowany, usuń skopiowaną część i zostaw tylko to, co chcesz dodać.
