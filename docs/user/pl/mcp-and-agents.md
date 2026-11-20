[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · **Polski** · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP i agenci

Agent AI (Claude Code, Cursor, Codex, ...) może sterować GitPoulpi przez swój
**serwer MCP** lub swoje **CLI**. Oba przechodzą przez tego samego demona
w tle co okno, więc to, co robi agent, od razu widać w otwartym GitPoulpi,
a oba uruchamiają dokładnie ten sam kod dla każdego polecenia.

Ta strona dotyczy agentów *zewnętrznych*. Własne przyciski AI w GitPoulpi (opisy
commitów, wyjaśnienia, kompozytor commitów) konfiguruje się osobno: zob.
[Wsparcie AI](ai.md).

## Szybka konfiguracja

1. Umieść `gitpoulpi` w `PATH` ([Instalacja](installation.md)). Serwerem jest `gitpoulpi mcp serve`; przekaźnik `gitpoulpi-mcp` z archiwum nadal działa jako alias, więc istniejąca konfiguracja agenta, która go wskazuje, dalej działa.
2. Zarejestruj serwer w swoim agencie. Mówi przez stdio: nic nie trzeba
   uruchamiać, żadnego portu.
3. Opcjonalnie wybierz w tabeli `[mcp]` pliku `config.toml`, co agent może
   robić (niżej). Domyślnie może czytać repozytorium (zmiany są odrzucane, dopóki nie
   ustawisz `allow_mutations = true`), pokazywać ci rzeczy dopiero po twoim
   potwierdzeniu i nigdy nie wypełnia wstępnie okien dialogowych.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` zapisuje to zamiast tego w pliku `.mcp.json` repozytorium,
więc każdy, kto otworzy projekt, to dostaje. Ten sam plik ręcznie, z opcjonalnymi
dodatkami:

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` nadpisuje `[mcp] ui_control` tylko dla tego agenta.
- `GITPOULPI_CONFIG_DIR` daje agentowi własny `config.toml` (własne limity
  `[mcp]`, bez dostępu do twoich ustawień). Pomiń go, aby współdzielić swój.
- `gitpoulpi --repo <path> mcp serve --stdio` przypina serwer do jednego
  repozytorium; w przeciwnym razie narzędzie działa na repozytorium wokół
  katalogu roboczego serwera albo na argumencie `repo` wywołania.

**Inni klienci**: każdy klient MCP, który uruchamia serwer stdio.

| Klient | Gdzie | Co |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | plik JSON MCP klienta | blok `mcpServers` powyżej (VS Code nazywa klucz `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Cokolwiek innego | jego dokumentacja | polecenie `gitpoulpi mcp serve` (domyślnie stdio; `gitpoulpi-mcp` to ten sam serwer dla konfiguracji, które już go używają) |

**Przez HTTP** (klienci łączący się z URL-em):

```
gitpoulpi mcp serve --http                   # [mcp] bind, domyślnie http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000    # lub: --bind 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Bezstanowy JSON, te same narzędzia, te same bramki `[mcp]`. Każde żądanie wymaga
tokenu bearer (`[mcp] http_auth`, domyślnie włączone): wypisz go przez `gitpoulpi mcp token
--show` (albo `--copy`, `--rotate`; token leży w pliku dostępnym tylko dla użytkownika w
prywatnym folderze danych) i przekaż klientowi jako nagłówek:

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

Brakujący lub błędny token dostaje `401` z `WWW-Authenticate: Bearer`. Wymieniony
token obowiązuje od razu, bez restartu serwera. Wyłączenie `http_auth` jest dla
punktu końcowego na pętli zwrotnej, gdy ufasz każdemu lokalnemu kontu; serwer ostrzega wtedy na
stderr.

### Dostęp do punktu końcowego z innego komputera

Najbezpieczniejsza konfiguracja w ogóle nie potrzebuje zdalnego nasłuchu: zostań przy adresie pętli zwrotnej i
przekieruj port przez SSH z komputera, na którym działa agent, `ssh -L
8808:127.0.0.1:8808 host`. Wtedy `--allow-remote` nie jest potrzebne.

`--allow-remote` (adres inny niż pętla zwrotna) jest odrzucane, chyba że uwierzytelnianie jest włączone
i istnieje token **oraz** ruch jest szyfrowany. Odmowa mówi, czego brakuje. Są dwa sposoby
na szyfrowanie:

- **TLS w serwerze** (`[mcp] tls = "self_signed"` lub `"files"`). Serwer
  sam kończy TLS: trzyma klucz, przeprowadza uzgadnianie i tylko zakończone
  uzgadnianie dociera do warstwy HTTP; klient ze zwykłym HTTP nie przechodzi uzgadniania, a obok
  nie istnieje żaden zwykły nasłuch. Szyfrowanie jest więc faktem, który serwer
  ustala przy każdym połączeniu, a nie założeniem. Przy `self_signed` pierwszy
  start generuje parę w prywatnym folderze danych i wypisuje jej
  odcisk SHA-256 (jest też w Ustawieniach); klient ufa serwerowi, przypinając
  ten odcisk lub plik certyfikatu. Przy `files` ustaw `tls_cert` i
  `tls_key` na swoje pliki PEM, a klienci weryfikują certyfikat przez jego urząd.
  Klienci uwierzytelniają się tokenem bearer; certyfikaty klientów nie są używane.
- **Tunel lub zwrotne proxy** (`[mcp] behind_proxy = true`). Agent tunelu lub
  proxy łączy się jak każdy lokalny klient lub klient z sieci LAN, a jego nagłówki można sfałszować, więc
  serwer nie może poznać od środka, czy jakiś jest. Dlatego jest to
  jawne oświadczenie (pokazywane z czerwonym ostrzeżeniem w Ustawieniach), a nie
  wykrywanie. Uruchom tunel na tym samym komputerze i zostań przy adresie pętli zwrotnej, gdy
  możesz. Przy adresie innym niż pętla zwrotna za proxy odcinek między proxy
  a serwerem jest w tej sieci zwykłym tekstem.

**Bez MCP** agent z powłoką używa CLI: `gitpoulpi --json <command>` (zob.
[Opis CLI](cli-reference.md)). Daj mu gotowy skill
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), który mówi,
kiedy i jak używać GitPoulpi zamiast surowego `git`.

## Uprawnienia

| Przełącznik | Domyślnie | Skutek |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` (i jego alias `gitpoulpi-mcp`) odmawia uruchomienia (Ustawienia > Agenci i MCP > *Serwer MCP*). Czytane przy starcie serwera: działający serwer zachowuje swoje ustawienia. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Adres, którego używa `--http`, gdy nie podano żadnego (tylko pętla zwrotna, chyba że `--allow-remote`, które wymaga uwierzytelniania i szyfrowania, zob. wyżej). Ustawienia > Agenci i MCP > *Port serwera MCP* edytuje jego port i zachowuje host. |
| `[mcp] http_auth` | `true` | `--http` wymaga `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Ustawienia > Agenci i MCP > *Uwierzytelnianie HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` lub `files` (twoje ścieżki PEM): serwer sam kończy TLS. Ustawienia > Agenci i MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Ręczysz, że tunel lub zwrotne proxy szyfruje ruch, co pozwala `--allow-remote` działać bez TLS. |
| `[mcp] allow_mutations` | `false` | Wyłączone: każde narzędzie, które coś zmienia, jest odrzucane; odczyty, `ui_*` i propozycje nadal działają. `true` pozwala agentowi robić commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Niepusta: tylko te narzędzia mogą coś zmieniać, np. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: agent nie może poruszać twoim oknem (`ui_state` / `ui_viewers` nadal odpowiadają). |
| `[mcp] ui_control` | `"ask"` | `ask`: znacznik w twoim oknie, który potwierdzasz; `auto`: stosowane, chyba że jesteś zajęty; `off`: odrzucane. Dla jednego agenta: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Pozwala agentowi otwierać wstępnie wypełnione okna dialogowe, które potwierdzasz (Ustawienia > Agenci i MCP > *Pozwól agentom AI proponować zmiany do mojego potwierdzenia*). |
| `[mcp] allow_sensitive_staging` | `false` | Tylko w `config.toml`. Wyłączone: narzędzie `stage` nie oferuje `allow_sensitive`, a dodanie, które objęłoby wrażliwy plik (klucz prywatny, `.env`, rozpoznany token), jest odrzucane z `{error: "sensitive_files", files}`: agent ma zapytać ciebie. `true` pozwala mu przekazać `allow_sensitive`. Zob. [Stage i commity](staging-and-committing.md#wrażliwe-pliki). |
| `[mcp] allow_history_purge` | `false` | Tylko w `config.toml`. `true` udostępnia narzędzie `rewrite_purge`, które usuwa na stałe starą historię przepisania (i jego cofnięcie). Wyłączone: agent go nie widzi. Nadal wymaga `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Główny wyłącznik twojego okna (Ustawienia > Agenci i MCP > *Pozwól agentom AI sterować widokiem*). Wyłączony: żaden agent nie dosięgnie twojego okna. |
| `[ui] launch` | `"off"` | Czy agent może otworzyć okno GitPoulpi, gdy żadne nie jest otwarte (zob. [niżej](#gdy-nie-ma-otwartego-okna-albo-jest-w-innej-sesji)). |

Co to oznacza w praktyce:

- **Od razu po instalacji agent jest tylko do odczytu**: może czytać, pokazywać
  ci rzeczy w twoim oknie (`ui_*`) i, jeśli na to pozwolisz, proponować zmiany,
  które potwierdzasz. Aby pozwolić mu zmieniać repozytorium (commit, push,
  reset...), ustaw `allow_mutations = true`; dla wąskiego uprawnienia zostaw
  `true` z `mutation_allowlist`. Każde narzędzie ma adnotacje MCP
  (`readOnlyHint`, `destructiveHint`, `openWorldHint`), więc klient taki jak
  Claude Code nadal pyta cię przed ryzykownymi.
- **Konflikty**: `conflicts_check` (przewidywanie, przed jakimkolwiek merge), `conflicts_list`, `conflicts_show` i `conflicts_state` to
  odczyty; `conflicts_resolve` (odrzuca stronę, której nie zachowano, `--delete`
  usuwa plik) i `conflicts_abort` (porzuca cały merge/rebase/cherry-pick) są
  destrukcyjne; `conflicts_unresolve` to zwykły zapis. Odrzucone wywołanie
  (`stale`, `not_conflicted`, `pick_incomplete`...) odpowiada `{"refused": {...}}`
  z kodem wyjścia 2 zamiast kończyć się błędem.
- **Rozmowy zgłoszeń i pull requestów**: `hosting_issue_comments` i
  `hosting_review_conversation` czytają całą oś czasu (odczyty, świat otwarty:
  wywołują hosta); `hosting_markdown` oczyszcza tekst i znajduje jego odwołania,
  niczego nie wywołuje (świat zamknięty). Publikowanie (`hosting_issue_comment`,
  `hosting_review_comment`, `hosting_review_reply`) i edycja
  (`hosting_*_comment_edit`) to zwykłe mutacje oznaczone jako świat otwarty: idą
  do hosta od razu, jako tożsamość tokenu (`posted_as` w odpowiedzi; token bota
  publikuje jako bot), więc to `allow_mutations` i `mutation_allowlist`
  decydują, czy agent może. Usunięcie komentarza (`hosting_*_comment_delete`) jest
  destrukcyjne. Po publikacji, edycji lub usunięciu demon jest powiadamiany, a otwarte
  okno repozytorium odświeża ten element. `ui_show_pr` / `ui_show_issue`
  przyjmują `--view conversation`, aby otworzyć rozmowę zamiast wiersza.
- **Zablokowane stany**: `conflicts_am_continue` to zapis, `conflicts_am_skip`,
  `conflicts_am_abort` i `index_lock_remove` są destrukcyjne, `index_lock_show`
  to odczyt.
- **Przepisywanie historii**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (analiza rozmiaru), `rewrite_scan_secrets` (znaleziska z zamaskowanym podglądem, nigdy z wartością),
  `rewrite_script_check`, `rewrite_script_test` i `rewrite_discard` to odczyty
  (podgląd niczego nie zmienia; tylko trzyma plan w demonie i na dysku w katalogu
  git, więc `rewrite_apply` działa też po restarcie). `rewrite_apply` i
  `rewrite_run` są destrukcyjne: przesuwają wiele refów na przepisane commity
  jako jeden wpis cofania (`undo` je przywraca) i nigdy nie wypychają. Przepis
  podaj jako `recipe_json` (albo `recipe`, ścieżka), a sekrety jako
  `secrets_file`, nigdy bezpośrednio.
  Skrypty powłoki, Pythona i zewnętrznego Lua (`script_language`) nie działają
  w piaskownicy, a **agent nie może im zaufać**: nie ma narzędzia
  `rewrite_script_trust` ani argumentu `trust`, więc taki skrypt działa dopiero
  po tym, jak człowiek uruchomił `gitpoulpi rewrite script trust FILE` na tym
  dokładnym tekście (zmieniony skrypt jest znowu odrzucany, `script_not_trusted`
  z hashem w `refused.script`), i tylko gdy `rewrite.external_scripts` włącza jego
  język (`scripts_disabled`).
  Odmowa odpowiada
  `{"refused": {"reason", "message", "detail"}}`, a plan, który przesuwa
  wypchnięte refy, odpowiada `{"needs_confirmation": ...}` (kod wyjścia 3),
  dopóki wywołanie nie ustawi `yes` - najpierw pokaż to użytkownikowi.
- **Nigdy nieosiągalne dla agenta**: zapisanie tokenu (`hosting login`),
  tworzenie i wysyłanie klucza SSH, `config set` (mógłby przepisać własne
  limity) oraz cykl życia demona. Token nigdy nie jest pokazywany agentowi.
- **Listy obejmujące całe konto**: `hosting_radar` to odczyt (żądania czekające na twój
  przegląd, otwarte żądania, przypisane zgłoszenia we wszystkich repozytoriach profilu,
  z pięciominutowej pamięci podręcznej, chyba że podano `refresh`); `hosting_integrations_radar`
  zmienia tabelę `[integrations.radar]` profilu, więc bramkuje je `allow_mutations`
  (także gdy wywołano je bez opcji, co tylko wypisuje ustawienia).
- **Kontekst przeglądu**: `ai_review_context` pokazuje, co szkic przeglądu kodu dostaje
  poza diffem (CLAUDE.md, AGENTS.md, plik dodatkowy, miejsce pracy lokalnego agenta). Z
  argumentem ustawienia zapisuje wartość domyślną (globalną albo własną `repo` lub `project`),
  więc bramkuje go `allow_mutations`, jak inne narzędzia zmieniające ustawienie; bez niego
  tylko wypisuje. Jak każde narzędzie `ai_*` jest wymieniane tylko przy włączonym `ai.enabled`.
  `hosting_review_suggest` przyjmuje te same argumenty, ale tylko na jeden przebieg.
- **Bramki `[mcp]` dotyczą serwera MCP.** CLI to twoja własna powłoka i nie
  podlega `allow_mutations`; do `gitpoulpi ui ...` stosują się tylko
  `ui_control` i `allow_ui_propose`.
- Odrzucone wywołanie to zwykły błąd narzędzia (`isError: true`, powód
  w `structuredContent.error`), nigdy ciche nic-nie-robienie.

```toml
[mcp]
allow_mutations = false        # true: agent może zmieniać repozytorium
mutation_allowlist = []        # np. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" pozwala agentowi zmieniać widok bez pytania
allow_ui_propose = false       # true: agent może wstępnie wypełniać okna Reword / Split / commit / gałąź / tag
allow_sensitive_staging = false  # true: agent może dodawać do stage pliki oznaczone przez strażnika
allow_history_purge = false    # true: agent widzi rewrite_purge (usuwa starą historię na stałe)
```

## Narzędzia

Jedno narzędzie na polecenie CLI, nazwane od jego ścieżki z `_` (`branch create`
to `branch_create`). Każde przyjmuje argumenty polecenia oraz `repo`
(repozytorium; domyślnie: serwera) i `session`. `structuredContent` udanego
wywołania to dokładnie to, co drukuje `gitpoulpi --json <command>`. **Rodzaj**:
*odczyt* niczego nie zmienia; *zapis* i *destrukcyjne* to mutacje (objęte
bramką); *sieć* rozmawia ze zdalnym repozytorium lub hostem; *widok* porusza
tylko twoim oknem; *propozycja* otwiera okno dialogowe, które potwierdzasz.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_hosting_reply` | `gitpoulpi ai hosting-reply` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `ai_worktree_name` | `gitpoulpi ai worktree-name` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_assignment` | `gitpoulpi hosting assignment` | write |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_comment` | `gitpoulpi hosting issue comment` | write, network |
| `hosting_issue_comment_delete` | `gitpoulpi hosting issue comment-delete` | destructive, network |
| `hosting_issue_comment_edit` | `gitpoulpi hosting issue comment-edit` | write, network |
| `hosting_issue_comments` | `gitpoulpi hosting issue comments` | read, network |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_markdown` | `gitpoulpi hosting markdown` | read |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_comment_delete` | `gitpoulpi hosting review comment-delete` | destructive, network |
| `hosting_review_comment_edit` | `gitpoulpi hosting review comment-edit` | write, network |
| `hosting_review_conversation` | `gitpoulpi hosting review conversation` | read, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi installation show` | describes this machine's GitPoulpi installations (folders, environments); local, like desktop and alias |
| `gitpoulpi installation set` | changes this installation's own names, colour, aliases and environment in the user's config |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
<!-- mcp-tools:end -->

### Akcje przepływu pracy z gałęziami działają w trybie plan, potem wykonanie

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` i `flow_set_parent`
([Przepływy pracy z gałęziami](branching-workflows.md#akcje)) nigdy nie ruszają
przy gołym wywołaniu. `dry_run: true` zwraca plan (każdy krok git i polecenie,
ostrzeżenia, to, czego Cofnij nie przywróci, blokady) jako zwykły wynik;
`yes: true` go wykonuje; bez żadnego z nich wywołanie jest odrzucane i niesie
plan. `[hooks]` pliku przepływu pracy to polecenia powłoki z pliku
współdzielonego przez zespół, więc działają tylko z jawnym `run_hooks: true`:
plan je wymienia (`hooks`, `hooks_run`), aby można było najpierw pokazać
użytkownikowi dokładne polecenia. Merge zatrzymany na konfliktach odpowiada
`status: "conflicted"` (wynik błędu, kod wyjścia 2); gdy narzędzia konfliktów
rozwiążą pliki, wywołaj to samo narzędzie z `continue_: true` lub `abort: true`.
Otwarcie merge requestu (`finish` w trybie `merge-request`) nie jest do
cofnięcia i plan o tym mówi.

### To, co agent robi zwykłym `git`, też trafia do historii cofania

Agent, który tworzy gałęzie i worktree, robi commity, reset albo porzuca stashe
własnym `git` (nie przez te narzędzia), nadal jest widoczny: daemon porównuje
refy repozytorium z tym, co widział ostatnio, i zapisuje każdą zmianę jako wpis
`undo --list` z `"source": "outside"`, powodem podanym przez gita
(`outside.message`) i autorem zapisanym przez gita. Narzędzia `undo`, `redo`
i `undo_restore` traktują te wpisy jak każde inne, więc agent (albo
użytkownik, przez `Ctrl+Z`) może przywrócić usuniętą gałąź. Zmiany plików nie
są nigdy przechwytywane. Zob. [Cofnij i ponów](undo-redo.md#zmiany-wykonane-poza-gitpoulpi).

## Pokazywanie rzeczy użytkownikowi

Agent może wskazać coś zamiast to opisywać: `ui_show_commit`, `ui_show_diff`
(plik w linii, inline lub split), `ui_show_file` (zawartość, blame lub historia
w linii), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`, `ui_show_conflict`
(edytor merge na pliku z konfliktem, przy konflikcie), `ui_show_wip` (`composer`
ustawia kursor w podsumowaniu commita, niczego nie wpisując), `ui_show_tab`,
`ui_show_home`, `ui_show_search`, `ui_show_palette` (wpisane zapytanie, nigdy
niewykonane), `ui_show_keys`, `ui_show_settings` (strona i ustawienie, nigdy
niezmieniane), `ui_highlight` (podświetla linie otwartego diffa lub pliku),
`ui_notify` (powiadomienie) i `ui_back` (cofa ostatnią zmianę widoku przez agenta).
`ui_state` odczytuje, co pokazuje okno (`text: true` dodaje widoczny tekst);
`ui_viewers` wymienia okna.

Te narzędzia **nigdy nie zmieniają repozytorium** i nie mogą pisać, klikać ani
edytować ustawień. Rewizje i ścieżki są sprawdzane, zanim cokolwiek dotrze do
okna.

**Co użytkownik widzi i o czym decyduje**:

- `ask` (domyślnie), albo gdy piszesz, otwarte jest okno dialogowe lub
  Ustawienia, przeciągasz albo w ciągu ostatnich 1,5 s naciśnięto klawisz:
  agent od razu dostaje `deferred`, a twój pasek stanu pokazuje znacznik *„claude-code
  chce pokazać ...”*. `Ctrl+N Enter` lub kliknięcie to pokazuje; `Esc` lub
  `Ctrl+N Backspace` to odrzuca. Wygasa po 60 s.
- Zastosowane: widok się przesuwa, 4-sekundowe powiadomienie mówi, kto go przesunął,
  a karta dostaje fioletowe `✦` do następnego klawisza. `Ctrl+N Backspace`
  (paleta `Agent: Wstecz`) wraca; okno pamięta dziesięć widoków.
- Nic nigdy nie jest odkrywane ani nie jest robiony checkout: commit ukryty przez twój
  filtr solo odpowiada `partial` (`hidden_by_filter`); lista PR lub zgłoszeń
  niepowiązana z dostawcą odpowiada `hosting_not_linked`.

**Odpowiedzi**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, ze `state` okna, gdy
zastosowano. Kody wyjścia CLI: 0 applied/partial/proposed, 3 deferred, 1 w
pozostałych przypadkach. Wywołania są ograniczane tempem (seria 5, potem jedno
na sekundę).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Proponowanie zmiany (`ui_propose_*`)

Gdy `allow_ui_propose = true`, agent może otworzyć jedno z twoich okien
dialogowych **wstępnie wypełnione**, oznaczone *✦ Propozycja od
<agent>*. Nic się nie dzieje, dopóki nie naciśniesz własnego przycisku
potwierdzenia tego okna; możesz najpierw edytować albo anulować.

| Narzędzie | Otwiera | Potwierdzasz przez |
|---|---|---|
| `ui_propose_reword` (rev, message) | okno Reword | *Zaktualizuj opis* (commit już wypchnięty nadal pyta o force-push) |
| `ui_propose_commit_message` (summary, body) | formularz commita, wypełniony; nigdy nie robi commita | przycisk Commit |
| `ui_propose_branch` (name, start point) | wbudowane pole Nowa gałąź | `Enter` |
| `ui_propose_tag` (name, target, message) | wbudowane pole Nowy tag (z adnotacją, gdy jest opis) | `Enter` |
| `ui_propose_rewrite` (recipe file) | warsztat przepisywania historii z regułami przepisu; niczego nie podgląda ani nie stosuje | twój własny *Podgląd*, potem *Zastosuj* (`Ctrl+Enter`); `Esc` odrzuca. Przepis ze skryptem jest odrzucany, wartości tajne są usuwane |
| `ui_propose_split` (rev, plan file) | edytor podziału nad commitem, wypełniony z planu | *Podziel* (`Ctrl+S`); `Esc` odrzuca |
| `ui_propose_resolution` (path, pick or content file) | edytor merge na pliku z konfliktem, z wyborami (lub całym tekstem) już w Wyniku | *Zapisz i oznacz jako rozwiązany* (`Ctrl+S`); zamknięcie go odrzuca |

Jedna propozycja naraz; propozycja opisu commita jest odrzucana, gdy masz własny
szkic w kompozytorze, a propozycja rozwiązania, gdy edytor merge trzyma twoją
niezapisaną pracę albo gdy stan pliku w poczekalni zmienił się od odczytu przez agenta.
Rozwiązanie zmienione przed potwierdzeniem jest zgłaszane jako
`accepted_edited`. Odpowiedź ma id `proposal`; to, co zdecydowano, jest
w `proposals` w `ui_state` (`offered`, `open`, `accepted`, `accepted_edited`,
`rejected`, `expired`, `failed`).

### Gdy nie ma otwartego okna albo jest w innej sesji

- Okno należy do sesji, w której je uruchomiono. Jeśli sesja agenta jej nie ma,
  `ui show` szuka **jednego** okna w innej sesji, które ma otwarte to
  repozytorium, i go używa (`routed_session` w odpowiedzi). Pasuje kilka:
  `no_viewer` z ich nazwami; podaj `--session`.
- **Otwieranie okna dla agenta** jest domyślnie wyłączone. Ustaw `[ui] launch`
  (Ustawienia > Agenci i MCP > *Otwórz okno dla agentów*) na ustawienie wstępne terminala
  lub multipleksera albo polecenie `custom`; wtedy wywołanie z `launch: true`
  (CLI `gitpoulpi ui --launch show ...`) otwiera okno i czeka do 10 s. Przy
  `ui_control = "auto"` otwiera się też bez tej flagi; przy `ask` agent dostaje
  `no_viewer` i powinien ci o tym powiedzieć.

| Ustawienie wstępne | Polecenie |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (twój domyślny terminal) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (new window), `zellij` (new pane), `cmux` (new workspace) | wewnątrz działającego multipleksera, w folderze repozytorium |
| `custom` | `[ui] launch_command`, np. `my-term --cd {repo} {gitpoulpi} --session {session}`; dzielone na argumenty, bez powłoki |

Jeśli ustawienie wstępne nie działa z twoją wersją programu, użyj `custom`.

## Błędy

- Źle sformułowane wywołanie (nieznane narzędzie lub argument, brak wymaganego)
  to błąd JSON-RPC `-32602 invalid_params`.
- Odmowa albo błąd git lub demona (zła rewizja, konflikt, ...) to błąd
  narzędzia: `isError: true` z komunikatem w `content` i w
  `structuredContent.error`.
