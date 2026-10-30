[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · **Polski** · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Obszary robocze

**Obszar roboczy** to nazwana grupa repozytoriów: tych, nad którymi pracujesz
razem (front end, jego API, dokumentacja). Pamięta, które z nich są otwarte jako
karty, i może mieć **domyślny profil commitów**, dzięki czemu commity w tych
repozytoriach używają właściwej tożsamości bez ręcznego przełączania.
Repozytorium może należeć do kilku obszarów roboczych, a brak obszaru jest w porządku:
„Brak obszaru” to zwykła lista kart, jaką znasz.

Obszary robocze są zapisane w `workspaces.toml`, obok `config.toml`, wspólnym dla
każdej sesji, okna, CLI i MCP (`gitpoulpi workspace ...`, zob. [Opis CLI](cli-reference.md)). Obszar roboczy dodany, przemianowany lub usunięty z powłoki albo przez `gitpoulpi workspace ...`, gdy okno jest
otwarte, pojawia się sam w ciągu około sekundy (karta Obszary robocze, plakietka na pasku kart i jej lista), bez
naciskania klawisza. Zamknięcie lub usunięcie obszaru roboczego nigdy nie rusza repozytorium na dysku.
Zmiany obszarów roboczych to ustawienia, jak ulubione i profile: nie są częścią
[Cofnij i ponów](undo-redo.md), a Usuń najpierw pyta.

## Plakietka obszaru roboczego

Gdy masz już jakiś obszar roboczy, na pasku kart tuż za `⌂` pojawia się plakietka:
kolorowa kropka i nazwa aktywnego obszaru albo „Brak obszaru” (przygaszone). Na
wąskim pasku kurczy się do kropki, a na bardzo wąskim znika.

![Lista rozwijana obszarów roboczych nad paskiem kart: Brak obszaru, Orbit (zaznaczony), Side projects, Archive i Zarządzaj obszarami roboczymi, z plakietką Orbit i jej profilem „Work - z obszaru Orbit” na pasku narzędzi](../images/workspace-switch.png)

- **Kliknij ją**, naciśnij `Alt+W` albo uruchom z palety `Obszar roboczy: Przełącz...`,
  by otworzyć listę: twoje obszary robocze, „Brak obszaru” i „Zarządzaj obszarami
  roboczymi...” (otwiera kartę Obszary robocze).
- **Przełączenie podmienia karty.** Karty, które miałeś otwarte, są zachowane
  dla obszaru, który opuszczasz, a wracają karty docelowego (za pierwszym razem
  jego repozytoria, które istnieją). Wróć, a wszystko jest tam, gdzie to
  zostawiłeś. Okno restartuje się jak przy przełączeniu karty; demon zostaje.
- **Obszar roboczy jest zapamiętywany**: następne uruchomienie wraca do niego.
- **Zamknięcie ostatniej karty** obszaru zostawia go aktywnym i pokazuje kartę Home.
- **`+` (nowa karta) przy aktywnym obszarze** dodaje wybrane repozytorium do listy
  kart i o nic nie pyta. Aby dodać je do samego obszaru, użyj „Dodaj do obszaru
  roboczego” w menu karty pod prawym przyciskiem (albo `Obszar roboczy: Dodaj kartę
  do obszaru roboczego`); „Usuń z obszaru roboczego” wyjmuje je z powrotem i zostawia
  kartę otwartą.

Dopóki nie ma żadnego obszaru roboczego, nie ma plakietki: najpierw utwórz go
w karcie Obszary robocze albo w palecie.

## Karta Obszary robocze

Otwórz ją z menu przy `+`, klawiszem `Alt+J` albo poleceniem `Obszar roboczy: Zarządzaj...` w palecie; jej `x` (lub `Ctrl+W`) ją zamyka, a po ponownym uruchomieniu pozostaje otwarta, dopóki jej nie zamkniesz. Każdy obszar roboczy to wiersz
z kolorową kropką, nazwą, liczbą repozytoriów, które wymienia (i ilu brakuje),
domyślnym profilem oraz napisem „aktywny”, gdy okno jest w nim. `Right`, `Space`
albo kliknięcie rozwija go do jego repozytoriów.

![Karta Obszary robocze: Orbit rozwinięty do dwóch repozytoriów, Side projects i Archive z oznaczeniem „brakuje: 1”](../images/workspaces.png)

| Dla obszaru | Klawisz | |
|---|---|---|
| Otwórz | `Enter` | Czyni go aktywnym i otwiera jego karty. |
| Edytuj... | `e` | Formularz poniżej. |
| Usuń... | `x` | Najpierw pyta. Repozytoria nie są ruszane. |
| Fetch wszystkich | `Shift+F` | Jedno repozytorium po drugim, postęp na pasku stanu, błędy wypisane na końcu. |
| Pull wszystkich | `Shift+P` | To samo, tylko fast-forward: nigdy nie zatrzymuje się na merge. |
| Klonuj brakujące | `m` | Klonuje każde brakujące repozytorium, które ma zapisane zdalne. |
| Nowy obszar roboczy... | `n` | |
| Z otwartych kart... | `a` | Nowy obszar roboczy z zaznaczonymi otwartymi kartami. |

**Brakujące** repozytorium (jego folderu nie ma) zostaje na liście, oznaczone,
z **Wskaż...** (`l` albo `Enter`: wskaż, gdzie jest teraz), **Klonuj** (gdy jego
URL `origin` został zapisany przy dodaniu) i **Usuń** (`x`). Każdy wiersz ma też
menu `⋮` (prawy przycisk, `.` albo `Shift+F10`) z tymi samymi akcjami, a każda
akcja jest w palecie pod `Obszar roboczy:`.

## Formularz Nowy / Edytuj

![Formularz edycji obszaru roboczego: nazwa, kolor, opis, domyślny profil i lista kontrolna repozytoriów](../images/workspace-edit.png)

- **Nazwa**: wymagana, unikalna.
- **Kolor**: lista rozwijana kolorów motywu; kropka na plakietce i w wierszach.
- **Opis**: dowolny tekst.
- **Profil**: lista rozwijana twoich `[[profiles]]` albo „Brak”. Jest opcjonalny.
- **✦ Instrukcje AI**: tylko przy włączonym AI, zob. [niżej](#instrukcje-ai-obszaru-roboczego).
- **Repozytoria**: lista kontrolna każdego repozytorium, które zna GitPoulpi (otwarte
  karty, ulubione, ostatnie, własne obszaru, także brakujące). `Space` zaznacza
  wiersz; **Dodaj folder...** (`Ctrl+B`) wybiera repozytorium z dysku. URL `origin`
  każdego nowego repozytorium jest odczytywany z repozytorium i zapisywany (bez
  żadnych poświadczeń); to z niego później korzysta Klonuj.

## Instrukcje AI obszaru roboczego

Przy włączonym AI (*Ustawienia > AI*) formularz ma wiersz **✦ Instrukcje AI** z przyciskiem **Edytuj...** (`Enter` na
wierszu). Otwiera mały edytor tego, co AI słyszy w **każdym repozytorium obszaru roboczego**: wybierz *Każda akcja AI*
albo jedną akcję (opis commita, wyjaśnianie zmian, opis stasha, pull request, kompozytor commitów, rozwiązywanie konfliktów,
przegląd kodu) na liście **Dla**, wybierz **Łączenie** - *Dodaj do globalnych* (wysyłane po instrukcjach globalnych) albo
*Zastąp globalne* (wysyłane zamiast nich) - i napisz tekst. **Zastosuj** zachowuje cały zestaw, **Zapisz** w formularzu
zapisuje go w `workspaces.toml` razem z obszarem roboczym; **Anuluj** porzuca to, co wpisałeś. Przy wyłączonym AI wiersz jest
ukryty, a instrukcje obszaru roboczego zostają, jakie były.

Słowa obszaru roboczego leżą między instrukcjami globalnymi a własnymi instrukcjami repozytorium (zob.
[Instrukcje AI](ai.md#instrukcje), gdzie jest kolejność i to, co robi *zastąp*). Repozytorium należące do
kilku obszarów roboczych używa aktywnego obszaru roboczego okna, gdy go zawiera, a w przeciwnym razie pierwszego, który go zawiera. Ze
skryptu: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` wypisuje je jako `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # każda akcja AI w tym obszarze roboczym
text = "Use the Acme glossary."
mode = "add"                       # albo "replace"

[workspace.ai_overrides.code_review]   # jedna akcja
text = "Flag missing tests."
mode = "add"
```

## Domyślny profil

Commity podpisuje pierwszy z tych, który ma zastosowanie:

1. profil wybrany **ręcznie** dla tego repozytorium (pole profilu na pasku narzędzi),
2. profil **aktywnego obszaru roboczego**, gdy repozytorium jest jego członkiem,
3. `general.active_profile`.

Pole profilu na pasku narzędzi mówi, skąd on pochodzi: „Work - z obszaru Acme”.
Pole pokazuje się też wtedy, gdy masz jeden profil, o ile obowiązuje domyślny
profil obszaru. Obszar roboczy, którego profilu już nie ma, po prostu przestaje
go stosować. W CLI `commit --workspace <id>` (albo `GITPOULPI_WORKSPACE`) rozstrzyga
tak samo.

## Klawisze

| Klawisz | Działanie |
|---|---|
| `Alt+W` | Otwiera listę obszarów roboczych |
| `Alt+J` | Otwórz kartę Obszary robocze |
| `n`, `a` | Nowy obszar roboczy, nowy z otwartych kart |
| `e` | Edytuj obszar roboczy |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch wszystkich, Pull wszystkich, Klonuj brakujące, Wskaż |

Wszystkie można przypisać inaczej w Ustawienia > Klawiatura i mysz; pełna lista jest
w [Skróty klawiszowe](keybindings.md).
