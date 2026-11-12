[English](../image-view.md) · **Čeština** · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Obrázky v zobrazení diff a v zobrazení Soubor

Binární soubor, který je obrázkem, se zobrazí jako obrázek, ne s oznámením
„(žádné textové změny - binární obsah...)“. Zobrazení diffu ukazuje **před** a
**po**; zobrazení Soubor ukazuje soubor, jaký je; editor merge ukazuje strany
obrázku v konfliktu. Vše zde je jen pro čtení: prohlížení obrázku nikdy
nemění repozitář, takže není co vracet.

Vypnete to volbou **Nastavení > Obrázky > Zobrazovat náhled obrázků**
(`ui.image_view`): vrátí se prosté oznámení o binárním souboru. Vlastní
sestavení bez funkce `image-view` nemá prohlížeč vůbec a stránka Obrázky si
ponechá jen seznam editorů (viz [Vlastní sestavení](custom-builds.md)).

## Které soubory

PNG a JPEG vždy; GIF, WebP (jejich první snímek), BMP a ICO/CUR, když sestavení
obsahuje `image-view`. TIFF, AVIF, HEIC, PSD a RAW se rozpoznají, ale
nedekódují: informační karta uvede formát, velikost a rozměry. SVG je text a
ponechává textový diff. Obrázek nad **Max. velikost náhledu** (ve výchozím stavu
32 MiB) nebo nad 64 megapixelů místo toho ukáže informační kartu. Soubor
uložený v Git LFS se zobrazí, pokud je jeho objekt v lokálním úložišti; jinak
karta řekne, abyste nejdřív spustili `LFS > Pull`. Průhlednost se kreslí přes
šachovnici.

## Jak se kreslí

V terminálu s protokolem obrázků (Kitty a Ghostty, WezTerm a iTerm2, Rio,
mintty a terminály se sixel Windows Terminal, foot a Tabby) je obrázek skutečný obrázek. Jinde (obyčejná konzole Windows,
zellij, terminál, který GitPoulpi nezná) se kreslí barevnými znaky půlbloků, s poloviční
svislou rozlišovací schopností, ale čitelně všude; **Bez protokolu obrázků** lze
nastavit na *Jen informační karta*. **Protokol obrázků** na stránce **Log a graf**
vynutí jeden z nich.

## Zobrazení diffu

Otevřete diff souboru s obrázkem (commitu, nebo unstaged či staged
poloviny rozpracovaných změn). Záhlaví pojmenuje oba soubory
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), poté následují tlačítka
zvětšení, **Přizpůsobit**, **1:1** a **Upravit v**.

- **Vedle sebe | Řádkově** vpravo od záhlaví, nebo `m`: vedle sebe, nebo
  jeden obrázek najednou. **Tab**, nebo klik na *Před | Po*, ukáže ten druhý v
  řádkovém zobrazení. Rozdělení příliš úzké pro dva panely se nakreslí řádkově,
  aniž by se změnilo vaše nastavení (`ui.diff_layout`).
- **Dělicí čára** rozdělení se přesouvá tažením, klávesami `Alt+Right` /
  `Alt+Left` (panel před si bere / uvolňuje místo) a vrátí se doprostřed
  dvojklikem nebo `Alt+0`. Poměr se pamatuje po dobu relace.
- Nový soubor ukáže jeden panel s popiskem *Nový soubor*, smazaný *Smazaný
  soubor*; oba skryjí přepínač. Nesledovaný obrázek vybraný v rozpracovaných
  změnách se zobrazí jako nový soubor.
- Strana před je rodičovský commit (diff commitu), `HEAD` (staged) nebo index
  (unstaged); strana po je commit, index nebo soubor na disku. Pokud se soubor
  na disku změní, když je diff otevřený (uložili jste jej z editoru, nástroj jej
  přepsal), obrázek jej následuje do zhruba půl sekundy.

## Zobrazení Soubor

Otevřete obrázek pomocí **Soubor** z nabídky souboru nebo z lišty diffu: jeden
panel, soubor v dané revizi (nebo na disku pro rozpracované změny).

## Zvětšení a posouvání

| | |
|---|---|
| `+` / `-` (nebo `=`) | Přiblížit / oddálit, kolem středu panelu |
| `0` | **Přizpůsobit** (výchozí): celý obrázek, který zůstává přizpůsoben i při změně velikosti okna nebo dělicí čáry |
| `1` | Skutečná velikost (1:1) |
| Šipky, `h j k l` | Posun o desetinu panelu; se `Shift` o celý panel; `PageUp`/`PageDown`, `Home`/`End` |
| Kolečko | Svislý posun; `Shift`+kolečko vodorovný; `Ctrl`+kolečko přibližuje kolem ukazatele |
| Tažení | Uchopte obrázek a přesuňte jej; posuvníky po stranách fungují jako všude |

Přiblížené zobrazení při změně velikosti panelu zachová své měřítko i bod ve
středu; Přizpůsobit zůstane Přizpůsobit. V zobrazení diff sdílejí oba panely
jedno zvětšení a jednu pozici posunu (**Synchronizovat přiblížení a posun
před/po**, ve výchozím stavu zapnuto), takže je vidět stejná část obou
obrázků; liší-li se velikostí, zvětšení se drží relativně ke každému. Od
**Pixelové zvětšení od** (ve výchozím stavu 2x) se obrázek zvětšuje jako
čtvercové pixely místo vyhlazení, což vyhovuje ikonám a pixel artu.

## Upravit v

Tlačítko **Upravit v** v záhlaví uvádí editory obrázků nalezené v tomto počítači
(GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... a výchozí aplikace
systému); `o` spustí výchozí a `Shift+O` otevře seznam. Je také v nabídce
pravého tlačítka u obrázku v seznamech souborů a v paletě příkazů
(*Obrázek: Upravit v...*). Seznam a výchozí volba se nastavují v
**Nastavení > Obrázky**, tak jako funguje [Otevřít v](open-in.md).

Úprava souboru na disku (strana rozpracovaných změn) je přímá: editor ukládá na
místě a obrázek se obnoví. Jakákoli jiná verze (z commitu, index, smazaný
soubor) se zapíše do **dočasné kopie jen pro čtení**, s oznámením na stavovém
řádku, takže editor nemůže změnit repozitář.

Aplikace, které systém registruje pro typ obrázku (jeho seznam *Otevřít pomocí*), následují v seznamu po vašich editorech a neopakují program, který tam už je. Ostatní binární soubory mají stejný seznam, viz [Otevřít v](open-in.md).

## V editoru merge

Obrázek v konfliktu ukáže svou kartu s obrázky **A**, **B** a (když existuje)
**Základ** v rámečcích, vedle sebe, každý přizpůsobený svému rámečku a jen pro
čtení (bez zvětšování; obrázky jen pomáhají s volbou). Tlačítka a klávesy se
nemění: **Ponechat A** (`a`), **Vzít B** (`b`), **Vzít základ** (`o`),
**Otevřít v nástroji merge** (`m`). Náhledy potřebují dostatečně široké okno
(asi 70 sloupců) a nějaké volné řádky; v užším okně je karta prostě textová.
Viz [Řešení merge konfliktů](merge-conflicts.md).

## Klávesy a nastavení

Klávesy jsou v [Klávesové zkratky](keybindings.md#zobrazení-obrázku); každou lze
přemapovat a je dostupná z palety (`Ctrl+P`, napište *Obrázek*). Volby jsou na
stránce **Obrázky** v [Nastavení](settings.md#obrázky) a v
[referenční příručce konfigurace](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
