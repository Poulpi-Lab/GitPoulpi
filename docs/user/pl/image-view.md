[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · **Polski** · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Obrazy w widoku Diff i widoku pliku

Plik binarny będący obrazem jest pokazywany jako obraz, a nie jako informacja
„(brak zmian tekstowych - zawartość binarna...)”. Widok Diff pokazuje **przed**
i **po**; widok pliku pokazuje plik takim, jaki jest; edytor merge pokazuje
strony obrazu w konflikcie. Wszystko tutaj jest tylko do odczytu: oglądanie
obrazu nigdy nie zmienia repozytorium, więc nie ma czego cofać.

Wyłączysz to przez **Ustawienia > Obrazy > Podgląd obrazów** (`ui.image_view`):
wraca zwykła informacja o pliku binarnym. Własna kompilacja bez funkcji
`image-view` nie ma w ogóle przeglądarki, a strona Obrazy zachowuje tylko listę
edytorów (zob. [Własne kompilacje](custom-builds.md)).

## Które pliki

PNG i JPEG zawsze; GIF, WebP (ich pierwsza klatka), BMP i ICO/CUR, gdy kompilacja
ma `image-view`. TIFF, AVIF, HEIC, PSD i RAW są rozpoznawane, ale nie
dekodowane: karta informacyjna podaje format, rozmiar i wymiary. SVG to tekst
i zachowuje diff tekstowy. Obraz większy niż **Maks. rozmiar podglądu (MiB)**
(domyślnie 32 MiB) lub większy niż 64 megapiksele pokazuje zamiast tego kartę
informacyjną. Plik przechowywany w Git LFS jest pokazywany, gdy jego obiekt
jest w lokalnym magazynie; w przeciwnym razie karta każe najpierw zrobić
`LFS > Pull`. Przezroczystość jest rysowana na szachownicy.

## Jak jest rysowany

W terminalu z protokołem obrazów (Kitty i Ghostty, WezTerm i iTerm2, Rio,
mintty oraz terminale sixel: Windows Terminal, foot i Tabby) obraz jest prawdziwym obrazem. Gdzie indziej (zwykła konsola Windows,
zellij, terminal nieznany GitPoulpi) jest rysowany kolorowymi znakami
półbloków: z połową rozdzielczości w pionie, ale czytelnie wszędzie; **Bez
protokołu obrazów** można ustawić na *Tylko karta informacyjna*. **Protokół
obrazów** na stronie **Wygląd** wymusza jeden z nich.

## Widok Diff

Otwórz diff pliku obrazu (commita albo połowy unstaged lub staged
zmian roboczych). Nagłówek podaje oba pliki (`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`),
potem przyciski powiększenia, **Dopasuj**, **1:1** i **Edytuj w**.

- **Obok siebie | W linii** po prawej stronie nagłówka lub `m`: obok siebie albo
  jeden obraz naraz. **Tab** lub kliknięcie *Przed | Po* pokazuje w widoku
  w linii drugi obraz. Podział zbyt wąski na dwa panele jest rysowany w linii
  bez zmiany twojego ustawienia (`ui.diff_layout`).
- **Dzielnik** podziału przesuwa się przeciąganiem, `Alt+Right` /
  `Alt+Left` (panel przed zabiera / oddaje miejsce), a wraca na środek
  dwukrotnym kliknięciem lub `Alt+0`. Proporcja jest pamiętana w sesji.
- Nowy plik pokazuje jeden panel z podpisem *Nowy plik*, usunięty *Usunięty
  plik*; oba ukrywają przełącznik. Nieśledzony obraz zaznaczony w zmianach
  roboczych jest podglądany jak nowy plik.
- Stroną przed jest commit rodzica (diff commita), `HEAD` (staged) lub indeks
  (unstaged); stroną po jest commit, indeks lub plik na dysku. Jeśli plik na
  dysku zmieni się przy otwartym diffie (plik zapisano z edytora lub przepisało go
  narzędzie), obraz podąży za nim w ciągu około pół sekundy.

## Widok pliku

Otwórz obraz przez **Widok pliku** z menu pliku lub paska diffa: jeden panel,
plik w danej rewizji (albo na dysku dla zmian roboczych).

## Powiększenie i przewijanie

| | |
|---|---|
| `+` / `-` (lub `=`) | Powiększ / pomniejsz, wokół środka panelu |
| `0` | **Dopasuj** (domyślnie): cały obraz; pozostaje dopasowany po zmianie rozmiaru okna lub dzielnika |
| `1` | Rzeczywisty rozmiar (1:1) |
| Strzałki, `h j k l` | Przewiń o jedną dziesiątą panelu; z `Shift` o cały panel; `PageUp`/`PageDown`, `Home`/`End` |
| Kółko | Przewijanie w pionie; `Shift`+kółko w poziomie; `Ctrl`+kółko powiększa wokół wskaźnika |
| Przeciąganie | Chwyć obraz i przesuń go; paski przewijania po bokach działają jak wszędzie |

Powiększony widok zachowuje skalę i punkt w swoim środku, gdy zmieni się rozmiar
panelu; Dopasuj pozostaje Dopasuj. W widoku Diff oba panele mają jedno
powiększenie i jedną pozycję przewijania (**Synchronizuj powiększenie
i przewijanie przed/po**, domyślnie włączone), więc widać ten sam fragment
obu obrazów; gdy różnią się rozmiarem, powiększenie jest zachowywane względem
każdego. Od **Powiększ pikselowo od** (domyślnie 2x) obraz jest powiększany jako
kwadratowe piksele zamiast wygładzany, co pasuje do ikon i pixel artu.

## Edytuj w

Przycisk **Edytuj w** w nagłówku wymienia edytory obrazów znalezione na tym
komputerze (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... oraz
domyślną aplikację systemu); `o` uruchamia domyślny, a `Shift+O` otwiera listę.
Jest też w menu prawego przycisku obrazu na listach plików oraz w palecie
poleceń (*Obraz: Edytuj w...*). Listę i domyślny ustawia się w
**Ustawienia > Obrazy**, tak jak działa [Otwórz w](open-in.md).

Edycja pliku na dysku (strona zmian roboczych) jest bezpośrednia: edytor zapisuje
na miejscu, a obraz się odświeża. Każda inna wersja (commita, indeksu, usuniętego
pliku) jest zapisywana do **kopii tymczasowej tylko do odczytu**, z informacją na
pasku stanu, więc edytor nie może zmienić repozytorium.

Aplikacje zarejestrowane w systemie dla typu obrazu (jego lista *Otwórz za pomocą*) występują na liście po edytorach i nie powtarzają programu, który już tam jest. Pozostałe pliki binarne mają tę samą listę, zob. [Otwórz w](open-in.md).

## W edytorze merge

Obraz w konflikcie pokazuje swoją kartę z obrazami **A**, **B** i (gdy jest)
**Baza** w polach, obok siebie, każdy dopasowany do swojego pola i tylko do
odczytu (bez powiększania; obrazy mają tylko pomóc w wyborze). Przyciski i
klawisze są bez zmian: **Zachowaj A** (`a`), **Użyj B** (`b`), **Użyj Bazy**
(`o`), **Otwórz w narzędziu merge** (`m`). Podglądy wymagają dość szerokiego
okna (około 70 kolumn) i trochę wolnych wierszy; w węższym karta jest zwykłą,
tekstową. Zob. [Rozwiązywanie konfliktów merge](merge-conflicts.md).

## Klawisze i ustawienia

Klawisze są w [Skrótach klawiszowych](keybindings.md#widok-obrazu); każdy da się
przypisać na nowo i każdy jest dostępny z palety (`Ctrl+P`, wpisz *Obraz*).
Opcje są na stronie **Obrazy** w [Ustawieniach](settings.md#obrazy) oraz w
[opisie konfiguracji](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
