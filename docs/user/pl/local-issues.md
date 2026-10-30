[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · **Polski** · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Zgłoszenia lokalne i Obsidian

Trzymaj swoje zgłoszenia jako folder notatek Markdown: bez konta, bez tokenu, bez sieci. Folder może leżeć w
repozytorium (`docs/issues/`, zatwierdzany razem z kodem) albo gdziekolwiek na dysku, na przykład w podfolderze
skarbca [Obsidian](https://obsidian.md). GitPoulpi czyta i zapisuje notatki. Obsidian, dowolny edytor lub powłoka mogą je edytować
w tym samym czasie, a listy za tym podążają.

![Zgłoszenia lokalne Markdown na pasku bocznym, z menu wiersza jednej notatki: otwórz notatkę, zamknij ją, edytuj jej etykiety lub kamień milowy](../images/local-issues.png)

## Konfiguracja

1. **Ustawienia > Integracje > Trackery zgłoszeń > Local Markdown > Dodaj profil.** Wpisz folder albo naciśnij
   **Przeglądaj...** i go wybierz. Folder względny (`docs/issues`) jest czytany względem każdego repozytorium, z którym jest powiązany; folder
   bezwzględny (`/home/me/Vault/Projects/Issues`) jest wszędzie ten sam. Nie ma tokenu do wpisania.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. W repozytorium otwórz sekcję **ZGŁOSZENIA** paska bocznego i wybierz dostawcę **Local Markdown - <nazwa>** (folder
   wybiera się razem z nim; lista repozytoriów zawiera ten folder i jego bezpośrednie podfoldery, gdybyś chciał innego).
   - CLI: `gitpoulpi hosting link issues --id <profile>` (folder profilu jest domyślnym `--project`).

Nic nie jest powiązane samo: jak każde źródło zgłoszeń, jest włączane osobno dla każdego repozytorium.

## Notatki

Jedna notatka na zgłoszenie, nazwana `<id>-<slug>.md` (`12-crash-on-empty-name.md`), z blokiem frontmatter:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Dowolny Markdown. GitPoulpi nigdy nie przepisuje treści.
```

- Nowe zgłoszenia dostają `max(id) + 1`.
- GitPoulpi zapisuje tylko właściwości, które posiada (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Każda inna właściwość, treść, zakończenia linii i kolejność kluczy są zachowywane bajt w bajt.** Zapisy są
  atomowe (plik tymczasowy, potem zmiana nazwy).
- Odczyt jest wyrozumiały: notatka bez `id` bierze go z nazwy pliku, notatka bez `status` ma `todo`, notatka
  bez frontmatter bierze tytuł z pierwszego `# nagłówka`. Plik, który nie jest zgłoszeniem (`readme.md`), jest pomijany.
- Status to dowolny tekst. **Otwarte** znaczy „nie jest statusem zakończenia”.

## Co możesz zrobić

| W sekcji ZGŁOSZENIA | Klawiatura / paleta |
|---|---|
| Nowe zgłoszenie (`+` albo `n`): tytuł, opis, status, przypisani, etykiety, kamień milowy | `Hosting: Nowe zgłoszenie...` |
| Edytuj przypisanych / etykiety / kamień milowy z menu wiersza. Wybierz z tego, czego folder już używa, albo **wpisz nową etykietę lub login** i weź wiersz `Dodaj „...”` | `Hosting: Edytuj etykiety...` i pozostałe |
| **Zamknij zgłoszenie** / **Otwórz zgłoszenie ponownie** (ustawia status zakończenia albo pierwszy inny) | `Zgłoszenia lokalne: Zamknij zgłoszenie...` / `Otwórz zgłoszenie ponownie...` |
| **Otwórz notatkę** (twój domyślny program dla `.md`) | `Integracje: Otwórz zgłoszenie...` |
| **Otwórz w Obsidian** (tylko gdy folder leży w skarbcu, czyli powyżej jest folder `.obsidian/`) | `Zgłoszenia lokalne: Otwórz w Obsidian...` |
| Filtry: Otwarte / Zamknięte / Wszystkie, „Przypisane do mnie” | lista filtrów |

„Przypisane do mnie” odpowiada ustawieniu `me` profilu, a w jego braku nazwie połączonego konta, a w jej braku nazwie profilu.

Gałęzie i commity działają jak w GitHub: „Utwórz gałąź ze zgłoszenia” daje `12-crash-on-empty-name`, a `#12` w opisie
commita lub `12-...` w nazwie gałęzi jest rozpoznawane.

Zmiany zrobione poza GitPoulpi (Obsidian, powłoka, `git pull`) pojawiają się w ciągu około sekundy: nazwy plików folderu
i czasy modyfikacji są sprawdzane w wątku w tle, nigdy w wątku rysowania.

## Plik konfiguracji

W `config.toml`, obok profilu (każdy klucz jest opcjonalny):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # kolejność kolumn tablicy
done_status = "done"                            # zamyka zgłoszenie
reference_prefix = "LOC"                         # pokazywane jako LOC-12 zamiast #12
me = "kai"                                       # kogo oznacza „przypisane do mnie”
```

Statusy używane przez notatki, ale niewymienione w `statuses`, idą za wymienionymi. `statuses` to kolejność kolumn
[tablicy](#tablica-kanban) (`gitpoulpi hosting issue options` ją wypisuje).

## Tablica Kanban

![Tablica folderu notatek: jedna kolumna na status, karty z przypisanymi, etykietami i kamieniem milowym](../images/issue-board.png)

Jeśli pracujesz sam albo w zespole, który trzyma zgłoszenia obok kodu, folder jest też tablicą Kanban.
**Nagłówek ZGŁOSZENIA > Tablica** albo wiersz palety **Zgłoszenia: Otwórz tablicę** otwiera ją nad oknem; `Esc` ją zamyka.

- **Kolumny** to statusy: najpierw te wymienione w `statuses`, w tej kolejności, potem każdy inny status użyty przez notatkę.
  Pusta kolumna jest nadal pokazana. Każdy nagłówek niesie liczbę kart.
- **Karty** pokazują numer zgłoszenia, tytuł (dwie linie), przypisanych jako inicjały, etykiety i kamień milowy.
  Gdy kolumny nie mieszczą się wszystkie, tablica przewija się w bok (pasek pod nią, kółko z `Shift` lub strzałki).
- **Przenieś kartę** przeciągając ją na inną kolumnę (kolumna podświetla się pod wskaźnikiem), klawiszami
  `Shift+Left` / `Shift+Right` albo wierszem **Tablica: Przenieś kartę do...** (`m`), listą kolumn. Karta przenosi się
  od razu; `status` notatki jest przepisywany jak w [sekcji o notatkach](#notatki), nic więcej w pliku. Jeśli zapis
  się nie uda, karta wraca, a błąd jest na pasku stanu.
- **Otwórz** kartę klawiszem `Enter` lub podwójnym kliknięciem (notatka otwiera się w twoim domyślnym programie dla `.md`). `e` edytuje jej
  przypisanych, etykiety lub kamień milowy, `.` albo prawy przycisk myszy otwiera menu karty, `n` zaczyna nowe zgłoszenie w zaznaczonej
  kolumnie (formularz wraca do tablicy po zamknięciu).
- **Filtruj** według przypisanego (`a`), etykiety (`l`), kamienia milowego (`M`) lub tekstu (`/`); `x` je czyści. Filtry to
  listy rozwijane: wpisz, aby zawęzić, `Enter` wybiera.
- **Na żywo**: notatka zmieniona z Obsidian lub powłoki przenosi swoją kartę w ciągu sekundy.

Każdy gest myszą ma klawisz i polecenie palety (**Tablica: ...**, wypisane na stronie
[klawiszy](keybindings.md)).

## Obsidian Bases

**Paleta: `Zgłoszenia lokalne: Utwórz Obsidian Base`** (CLI: `gitpoulpi hosting issue base`) zapisuje `issues.base` obok
notatek, jeden raz: widok *tabeli* i widok *kanban* pogrupowany według `status`, w kolejności `statuses`. Otwórz go w Obsidian, aby
zobaczyć tam te same zgłoszenia jako tablicę. GitPoulpi nigdy nie edytuje istniejącego pliku `.base` (twojego ani poprzedniego): odmawia
i o tym mówi. Format podąża za składnią Bases Obsidian
(<https://obsidian.md/help/bases/syntax>); widok kanban wymaga nowszego Obsidian.

## CLI i MCP

Te same polecenia co dla każdego źródła zgłoszeń działają na nim: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Cztery kolejne są dla notatek:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` i `hosting issue base`.
Zob. [Opis CLI](cli-reference.md).

## Ograniczenia

- Nowego **kamienia milowego** nie można wpisać w selektorze (wybierz taki, którego folder już używa), ale `milestone:` w notatce może
  być dowolne i pojawia się, gdy tylko zostanie zapisane.
- Zgłoszeniami są tylko notatki leżące bezpośrednio w folderze (nie w podfolderach).
- Nie ma usuwania: usuń notatkę w menedżerze plików albo w Obsidian (jego kosz jest twój).
