[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · **Polski** · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submoduły

Submoduł to inne repozytorium zagnieżdżone w twoim repozytorium, na ustalonym commicie. GitPoulpi wymienia je w panelu bocznym Refs, aktualizuje z interfejsu, z CLI lub z palety poleceń i potrafi utrzymywać je w zgodzie z commitem, na który robisz checkout.

## Sekcja SUBMODUŁY

Gdy repozytorium ma `.gitmodules`, panel boczny Refs dostaje sekcję **SUBMODUŁY** (poniżej pozostałych; gdy submodułów nie ma, jej nie ma). Jeden wiersz na submoduł: znacznik stanu, jego ścieżka i krótki commit.

| Znacznik | Stan | Znaczenie |
|---|---|---|
| `✓` | aktualny | checkout jest na commicie zapisanym w repozytorium |
| `↑` | zmodyfikowany | checkout jest na innym commicie (submoduł poszedł dalej albo zmieniono gałąź bez jego aktualizacji) |
| `○` | niesklonowany | zadeklarowany w `.gitmodules` i zapisany, ale jego folder jest pusty: zaktualizuj go, aby go sklonować |
| `✗` | w konflikcie | repozytorium ma konflikty merge na submodule |

![Sekcja SUBMODUŁY z trzema wierszami: vendor/almanac niesklonowany (○), vendor/ephemeris aktualny (✓) i vendor/starfield zmodyfikowany (↑, na innym commicie niż zapisany), co widać też jako zmieniony plik w panelu Status](../images/submodules.png)

| Czynność | Mysz | Klawiatura |
|---|---|---|
| Otworzyć submoduł we własnej karcie | dwukrotne kliknięcie wiersza | zaznacz go, `Enter` (paleta: `Refs: Otwórz submoduł w karcie`) |
| Menu: **Aktualizuj (sklonuj w razie potrzeby)**, **Zainicjuj**, **Otwórz w nowej karcie** | kliknięcie wiersza prawym przyciskiem | zaznacz go, klawisz menu kontekstowego (domyślnie `.`) |
| Zaktualizować zaznaczony submoduł (w razie potrzeby najpierw go klonuje) | menu | `Ctrl+P`: `Refs: Aktualizuj submoduł (zaznaczony)` |
| Zarejestrować go w `.git/config` bez klonowania | menu | `Ctrl+P`: `Refs: Inicjuj submoduł (zaznaczony)` |
| Zaktualizować każdy submoduł | | `Ctrl+P`: `Git: Aktualizuj wszystkie submoduły` |
| Przejść do sekcji | kliknięcie jej nagłówka | `Ctrl+P`: `Nawigacja: Przejdź do Refs > Submodules` |

![Menu prawego przycisku zmodyfikowanego wiersza vendor/starfield: Aktualizuj (sklonuj w razie potrzeby), Zainicjuj i Otwórz w nowej karcie](../images/submodule-menu.png)

Aktualizacja może pobierać dane ze zdalnego repozytorium submodułu, więc podlega ustawieniu **Ustawienia > SSH i poświadczenia > Poświadczenia HTTPS** jak każdy inny transfer. Niesklonowany submoduł nie ma czego otworzyć w karcie; pasek stanu to mówi i wskazuje aktualizację.

## Utrzymywanie submodułów w aktualności

**Ustawienia > Synchronizacja > Aktualizuj submoduły** (`general.keep_submodules_up_to_date`, domyślnie wyłączone). Gdy jest włączone, po udanym **checkoucie** lub **pullu** wykonywane jest `git submodule update --init --recursive`. Dzieje się to tylko wtedy, gdy submoduł rozjechał się z tym, co repozytorium teraz zapisuje, więc repozytorium bez submodułów nie zwalnia. Jeśli ta aktualizacja się nie powiedzie, checkout lub pull i tak się odbył, a błąd to mówi.

## Z wiersza poleceń

```
gitpoulpi submodule list                       # ścieżka, stan, URL (--json: pełna lista)
gitpoulpi submodule init [<path>...]           # rejestruje, nie klonuje
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Brak ścieżek oznacza każdy submoduł. Te same trzy polecenia są narzędziami MCP (`submodule_list` to odczyt; `init` i `update` to mutacje i podlegają `allow_mutations` z `[mcp]`).

## Uwagi

- Wymieniany jest tylko najwyższy poziom; submoduły samego submodułu widać po otwarciu go w karcie (albo użyj `--recursive`, aby je zaktualizować).
- Nowszy git odmawia sklonowania submodułu ze *ścieżką lokalną* przez transport `file`, chyba że pozwala na to `protocol.file.allow`. Dotyczy to tylko submodułów, których URL jest folderem lokalnym; ustaw to w globalnej konfiguracji git, jeśli ich używasz.
