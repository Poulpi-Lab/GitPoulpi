[English](../workspaces.md) · **Čeština** · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Pracovní prostory

**Pracovní prostor** je pojmenovaná skupina repozitářů: těch, na kterých
pracujete společně (front end, jeho API, dokumentace). Pamatuje si, které z nich
jsou otevřené jako karty, a může nést **výchozí profil commitů**, takže commity
v těchto repozitářích používají správnou identitu, aniž byste profil přepínali
ručně. Repozitář může být ve více pracovních prostorech a být v žádném je také
v pořádku: „Žádný pracovní prostor“ je dnešní obyčejný seznam karet.

Pracovní prostory jsou uložené v `workspaces.toml` vedle `config.toml` a sdílí
je každá relace, okno, CLI i MCP (`gitpoulpi workspace ...`, viz
[Referenční příručka CLI](cli-reference.md)). Pracovní prostor přidaný, přejmenovaný nebo odebraný z shellu nebo příkazem `gitpoulpi workspace ...`, zatímco je okno
otevřené, se objeví sám zhruba do vteřiny (karta Pracovní prostory, čip na liště karet a jeho seznam),
bez stisku klávesy. Zavření nebo smazání pracovního prostoru se nikdy nedotkne repozitáře na disku. Úpravy pracovních prostorů jsou
nastavení, stejně jako oblíbené a profily: nejsou součástí
[Zpět a znovu](undo-redo.md) a Smazat se nejdřív zeptá.

## Čip pracovního prostoru

Jakmile máte pracovní prostor, na liště karet se hned za `⌂` objeví čip:
barevná tečka a název aktivního pracovního prostoru, nebo „Žádný pracovní
prostor“ (ztlumeně). Na úzké liště se zmenší na tečku a na velmi úzké zmizí.

![Rozbalovací nabídka pracovních prostorů nad lištou karet: Žádný pracovní prostor, Orbit (zaškrtnutý), Side projects, Archive a Spravovat pracovní prostory, s čipem Orbit a jeho profilem „Work - z pracovního prostoru Orbit“ na panelu nástrojů](../images/workspace-switch.png)

- **Klikněte na něj**, stiskněte `Alt+W`, nebo spusťte z palety
  `Pracovní prostor: Přepnout...` a otevře se rozbalovací nabídka: vaše pracovní
  prostory, „Žádný pracovní prostor“ a „Spravovat pracovní prostory...“ (otevře
  kartu Pracovní prostory).
- **Přepnutí vymění karty.** Karty, které jste měli otevřené, se uchovají pro
  pracovní prostor, který opouštíte, a vrátí se karty cílového (poprvé jeho
  repozitáře, které existují). Přepněte zpět a všechno je tam, kde jste to
  nechali. Okno se restartuje jako při přepnutí karty; daemon běží dál.
- **Pracovní prostor se pamatuje**: příští spuštění se vrátí do něj.
- **Zavření poslední karty** pracovního prostoru nechá pracovní prostor aktivní
  a ukáže kartu Domů.
- **`+` (nová karta), když je pracovní prostor aktivní**, přidá repozitář, který
  vyberete, do seznamu karet a na nic se neptá. Chcete-li ho přidat do samotného
  pracovního prostoru, použijte „Přidat do pracovního prostoru“ v nabídce karty
  na pravé tlačítko (nebo `Pracovní prostor: Přidat kartu do pracovního
  prostoru`); „Odebrat z pracovního prostoru“ ho zase vyjme a kartu nechá
  otevřenou.

Dokud pracovní prostor nemáte, čip není: nejdřív jeden vytvořte na kartě Pracovní prostory
nebo v paletě.

## Karta Pracovní prostory

Otevřete ji z nabídky u `+`, klávesou `Alt+J` nebo příkazem `Pracovní prostor: Spravovat...` v paletě; její `x` (nebo `Ctrl+W`) ji zavře a zůstává otevřená i po restartu, dokud ji nezavřete. Každý
pracovní prostor je řádek s barevnou tečkou, názvem, počtem repozitářů, které
uvádí (a kolik jich chybí), výchozím profilem a slovem „aktivní“, když je v něm
okno. `Right`, `Space` nebo klik ho rozbalí na jeho repozitáře.

![Karta Pracovní prostory: Orbit rozbalený na dva repozitáře, Side projects a Archive označený „chybí: 1“](../images/workspaces.png)

| Pro každý pracovní prostor | Klávesa | |
|---|---|---|
| Otevřít | `Enter` | Aktivuje ho a otevře jeho karty. |
| Upravit... | `e` | Formulář níže. |
| Smazat... | `x` | Nejdřív se zeptá. Repozitářů se to nedotkne. |
| Fetch vše | `Shift+F` | Repozitář po repozitáři, průběh ve stavovém řádku, selhání vypsaná na konci. |
| Pull vše | `Shift+P` | Totéž, jen fast-forward: nikdy se nezastaví na merge. |
| Naklonovat chybějící | `m` | Naklonuje každý chybějící repozitář se zaznamenaným remote. |
| Nový pracovní prostor... | `n` | |
| Z otevřených karet... | `a` | Nový pracovní prostor s označenými otevřenými kartami. |

**Chybějící** repozitář (jeho složka je pryč) zůstane v seznamu, označený, a má
**Najít...** (`l` nebo `Enter`: vyberte, kde je nyní), **Klonovat** (když se
jeho URL `origin` zaznamenalo při přidání) a **Odebrat** (`x`). Každý řádek má
také nabídku `⋮` (pravé tlačítko, `.` nebo `Shift+F10`) se stejnými akcemi a
každá akce je v paletě pod `Pracovní prostor:`.

## Formulář Nový / Upravit

![Formulář Úprava pracovního prostoru: název, barva, popis, výchozí profil a seznam repozitářů k zaškrtnutí](../images/workspace-edit.png)

- **Název**: povinný, jedinečný.
- **Barva**: rozbalovací nabídka barev motivu; tečka na čipu a na řádcích.
- **Popis**: volný text.
- **Profil**: rozbalovací nabídka vašich `[[profiles]]`, nebo „Žádný“. Je
  volitelný.
- **✦ Instrukce AI**: jen při zapnuté AI, viz [níže](#instrukce-ai-pracovního-prostoru).
- **Repozitáře**: seznam k zaškrtnutí všech repozitářů, které GitPoulpi zná
  (otevřené karty, oblíbené, nedávné, vlastní pracovního prostoru, včetně
  chybějících). `Space` řádek zaškrtne; **Přidat složku...** (`Ctrl+B`) vybere
  repozitář z disku. URL `origin` každého nového repozitáře se přečte z
  repozitáře a zaznamená (bez jakýchkoli přihlašovacích údajů), což později
  používá Klonovat.

## Instrukce AI pracovního prostoru

Při zapnuté AI (*Nastavení > AI*) má formulář řádek **✦ Instrukce AI** s tlačítkem **Upravit...** (`Enter` na
řádku). Otevře malý editor toho, co se AI říká v **každém repozitáři pracovního prostoru**: v seznamu **Pro** vyberte *Každou akci AI*
nebo jednu akci (zpráva commitu, vysvětlení, zpráva stashe, pull request, composer commitů, řešení konfliktů,
kontrola kódu), v seznamu **Spojení** zvolte *Přidat ke globálním* (odešle se za globální instrukce) nebo
*Nahradit globální* (odešle se místo nich) a napište text. **Použít** zachová celou sadu, **Uložit** ve formuláři
ji zapíše spolu s pracovním prostorem do `workspaces.toml`; **Zrušit** zahodí, co jste napsali. Při vypnuté AI je řádek
skrytý a instrukce pracovního prostoru zůstanou, jak jsou.

Slova pracovního prostoru leží mezi globálními instrukcemi a instrukcemi samotného repozitáře (pořadí a to, co dělá *nahradit*,
viz [Instrukce AI](ai.md#instrukce)). Repozitář, který patří do
více pracovních prostorů, používá aktivní pracovní prostor okna, když ho obsahuje, jinak první, který ano. Ze
skriptu: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` je vypíše jako `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # every AI action in this workspace
text = "Use the Acme glossary."
mode = "add"                       # or "replace"

[workspace.ai_overrides.code_review]   # one action
text = "Flag missing tests."
mode = "add"
```

## Výchozí profil

Commity podepisuje první z těchto možností, která platí:

1. profil, který jste pro daný repozitář zvolili **ručně** (pole profilu na
   panelu nástrojů),
2. profil **aktivního pracovního prostoru**, když je repozitář jeho členem,
3. `general.active_profile`.

Pole profilu na panelu nástrojů říká, odkud pochází: „Work - z pracovního
prostoru Acme“. Pole se zobrazí i tehdy, když máte jediný profil, pokud platí
výchozí profil pracovního prostoru. Pracovní prostor, jehož profil už
neexistuje, se prostě přestane uplatňovat. V CLI se stejně rozhoduje
`commit --workspace <id>` (nebo `GITPOULPI_WORKSPACE`).

## Klávesy

| Klávesa | Akce |
|---|---|
| `Alt+W` | Otevřít nabídku pracovních prostorů |
| `Alt+J` | Otevřít kartu Pracovní prostory |
| `n`, `a` | Nový pracovní prostor, nový z otevřených karet |
| `e` | Upravit pracovní prostor |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch vše, Pull vše, Naklonovat chybějící, Najít |

Všechny jdou přeřadit v Nastavení > Klávesnice a myš; celý seznam je v
[Klávesových zkratkách](keybindings.md).
