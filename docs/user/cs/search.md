[English](../search.md) · **Čeština** · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Hledání

`Ctrl+Alt+F` (nebo `Ctrl+P` a pak `Hledat: Pracovní prostor...`) otevře fuzzy
vyhledávač přes **všechny repozitáře, ve kterých pracujete**: repozitáře
aktivního [pracovního prostoru](workspaces.md), když nějaký je, jinak otevřené
karty. Otevře ho i tlačítko **Globální hledání** (lupa s popiskem v záhlaví okna, těsně vlevo od
ozubeného kola). Najde:

- **repozitáře** (podle názvu, pak podle cesty),
- **větve** (lokální i vzdálené sledovací) a **tagy**,
- **worktrees** (podle složky, pak podle cesty a větve),
- **stashe** (podle zprávy),
- **commity** (podle předmětu; tělo commitu se prohledává, jen když se
  předmět neshoduje a napsali jste tři znaky nebo víc),
- **pull nebo merge requesty** a **issues** (podle názvu, `#čísla`, autora a
  větve).

![Okno hledání nad dvěma otevřenými repozitáři: větve, stash a commit odpovídající „main“, každý s repozitářem vpravo](../images/workspace-search.png)

Výsledky jsou seskupené podle druhu a uvnitř skupiny jde nejlepší shoda první.
Každý řádek má ikonu druhu, zvýrazněné shodující se znaky, ztlumený detail
(upstream větve, autor a větev žádosti, pozice stashe) a vpravo název
repozitáře, takže dva repozitáře s větví `main` nikdy nesplynou. V titulku se
objeví spinner, když repozitář ještě indexuje své commity (první hledání ve
velkém repozitáři může chvíli trvat); ostatní druhy jsou tu hned a commity se
přidají, jakmile jsou hotové.

## Přechod na výsledek

| Klávesa | Co dělá |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `PageUp` / `PageDown`, `Home` / `End` | Pohyb po výsledcích (nadpisy skupin se přeskakují). |
| `Enter` | Přejde na výsledek: karta repozitáře se otevře (nebo se na ni přepne) a výsledek se ukáže. |
| `Esc` | Zavře hledání. |
| `Alt+1` .. `Alt+8` | Zapne nebo vypne druh (čipy nad seznamem, v jejich pořadí). `Alt+0` zapne všechny. |

Co „přejít“ znamená:

| Výsledek | Enter |
|---|---|
| Větev nebo tag | Vybere ho v panelu Refs a jeho commit v Logu. |
| Commit | Vybere ho v Logu; když je starší než to, co je načteno, nejdřív načte víc Logu. |
| Stash | Vybere jeho řádek v sekci STASHES. |
| Worktree | Otevře ho jako kartu. |
| Pull/merge request, issue | Vybere jeho řádek v postranním panelu (na kartě toho repozitáře). |
| Repozitář | Přepne na jeho kartu (a otevře ji, když není otevřená). |

`Ctrl+N Backspace` (nebo `Zpět` v paletě) vrátí tam, kde jste byli, jako každou
změnu pohledu, kterou za vás udělá agent. Přechodem kamkoli se nic nepřepne,
nesloučí ani nezmění.

Myší: klik na řádek ho vybere, dvojklik na něj přejde, klik na čip druh zapne
nebo vypne, kolečko posouvá a klik mimo okno ho zavře.

## Zúžení předponou

Začněte dotaz předponou a hledá se jen jeden druh, ať čipy říkají cokoli:

| Předpona | Druh |
|---|---|
| `r:` | repozitáře |
| `b:` | větve |
| `t:` | tagy |
| `w:` | worktrees |
| `s:` | stashe |
| `c:` | commity |
| `pr:` (nebo `mr:`, `p:`) | pull nebo merge requesty |
| `i:` | issues |
| `#` | pull/merge requesty a issues (`#12` najde číslo 12) |

`fix: login` je obyčejný dotaz, ne předpona: počítá se jen jediné písmeno (nebo
`pr:` / `mr:`) následované dvojtečkou, nebo úvodní `#`.

## Jak se shoduje

Stejné tři úrovně jako paleta příkazů: váš text doslova, pak doslova kdekoli a
pak jeho znaky ve stejném pořadí, ale od sebe. Dřívější shoda a kratší text
vyhrávají. Velká a malá písmena jsou totéž.

## Pull requesty a issues

Pocházejí z toho, co GitPoulpi už pro každý repozitář načetl (seznamy za
sekcemi PULL REQUESTS a ISSUES v postranním panelu), uložené v malém souboru
cache ve složce `.gitpoulpi/` repozitáře. Hledání **nikdy nevolá GitHub ani
GitLab**: repozitář, jehož žádosti se v žádném okně nikdy nenačetly, nemá co
najít, a to, co vidíte, je tak čerstvé jako poslední obnovení postranního
panelu. Bez propojeného účtu není co najít a není co nastavovat.

## Z příkazové řádky

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` dělá totéž
a vypisuje JSON (viz [Referenční příručka CLI](cli-reference.md#pracovní-prostory-a-hledání));
MCP server má nástroj `search`, takže agent najde to, co vy.
`search.commit_limit` v [Nastavení](config-reference.md#search-hledání-v-pracovním-prostoru)
omezuje, kolik commitů každého repozitáře se indexuje (`0` vypne hledání commitů).

## Změna klávesy

`Hledat: Pracovní prostor...` je příkaz palety a klávesa je akce
`global.workspace_search`: přeřaďte ji v Nastavení > Klávesnice a myš, nebo v
tabulce `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
