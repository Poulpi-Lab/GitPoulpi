[English](../submodules.md) · **Čeština** · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submoduly

Submodul je jiný repozitář vnořený do vašeho na pevném commitu. GitPoulpi je vypisuje v postranním panelu Refs, aktualizuje je z rozhraní, z CLI nebo z palety příkazů a umí je udržovat v souladu s commitem, který checkoutujete.

## Sekce SUBMODULY

Když repozitář obsahuje `.gitmodules`, získá postranní panel Refs sekci **SUBMODULY** (pod ostatními; není-li žádný submodul, chybí). Jeden řádek na submodul: značka stavu, jeho cesta a krátký commit.

| Značka | Stav | Význam |
|---|---|---|
| `✓` | aktuální | checkoutnutý na commitu, který repozitář zaznamenává |
| `↑` | změněný | checkoutnutý na jiném commitu (posunul se, nebo jste přepnuli větve a neaktualizovali jej) |
| `○` | nenaklonovaný | deklarovaný v `.gitmodules` a zaznamenaný, ale jeho složka je prázdná: aktualizací jej naklonujete |
| `✗` | v konfliktu | repozitář má merge konflikty na submodulu |

| Akce | Myš | Klávesnice |
|---|---|---|
| Otevřít submodul jako samostatnou kartu | dvojklik na řádek | vybrat jej, `Enter` (paleta: `Refs: Otevřít submodul na kartě`) |
| Nabídka: **Aktualizovat (v případě potřeby naklonovat)**, **Inicializovat**, **Otevřít na nové kartě** | pravé tlačítko na řádek | vybrat jej, klávesa kontextové nabídky (výchozí `.`) |
| Aktualizovat vybraný submodul (v případě potřeby jej nejdřív naklonuje) | nabídka | `Ctrl+P`: `Refs: Aktualizovat submodul (vybraný)` |
| Zaregistrovat jej v `.git/config` bez klonování | nabídka | `Ctrl+P`: `Refs: Inicializovat submodul (vybraný)` |
| Aktualizovat každý submodul | | `Ctrl+P`: `Git: Aktualizovat všechny submoduly` |
| Přeskočit na sekci | klik na její záhlaví | `Ctrl+P`: `Navigace: Přejít na Refs > Submoduly` |

Aktualizace může provést fetch ze vzdáleného repozitáře submodulu, takže se řídí volbou **Nastavení > SSH a přihlášení > Přihlašovací údaje HTTPS** jako každý jiný přenos. Nenaklonovaný submodul nemá co otevřít jako kartu; stavový řádek to oznámí a odkáže na Aktualizovat.

## Udržování submodulů aktuálních

**Nastavení > Synchronizace > Udržovat submoduly aktuální** (`general.keep_submodules_up_to_date`, ve výchozím stavu vypnuto). Je-li zapnuto, po úspěšném **checkoutu** nebo **pullu** následuje `git submodule update --init --recursive`. Spustí se jen tehdy, když je submodul mimo soulad s tím, co repozitář nyní zaznamenává, takže repozitář bez submodulů to nezpomalí. Pokud tato aktualizace selže, checkout nebo pull přesto proběhl a chyba to uvede.

## Z příkazového řádku

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Bez cest se myslí každý submodul. Stejné tři příkazy jsou nástroje MCP (`submodule_list` je čtení; `init` a `update` jsou mutace a řídí se volbou `allow_mutations` v `[mcp]`).

## Poznámky

- Vypisuje se jen nejvyšší úroveň; submoduly submodulu se ukážou, když jej otevřete jako kartu (nebo je aktualizujte pomocí `--recursive`).
- Novější git odmítá klonovat submodul s *lokální cestou* přes transport `file`, pokud to `protocol.file.allow` nepovolí. Týká se to jen submodulů, jejichž URL je lokální složka; pokud je používáte, nastavte to v globální konfiguraci gitu.
