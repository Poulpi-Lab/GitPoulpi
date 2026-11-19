[English](../getting-started.md) · **Čeština** · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# První kroky

Tato stránka vás provede od čerstvé instalace k prvnímu commitu.
[Prohlídka rozhraní](interface-tour.md) pojmenovává každou oblast okna.
[Nastavení](settings.md) uvádí každou volbu.

## 1. Instalace a spuštění

Postupujte podle [Instalace](installation.md). Poté otevřete terminál v
repozitáři Gitu a spusťte:

```
gitpoulpi
```

GitPoulpi otevře repozitář, který
obsahuje aktuální složku. Mimo repozitář otevře kartu **Domů**, kde můžete
procházet, klonovat nebo inicializovat repozitář (viz
[krok 7](#7-otevření-klonování-a-inicializace-repozitářů)).

Ikony potřebují Nerd Font nebo písmo s emoji. Pokud vidíte čtverečky, viz
[Řešení potíží](troubleshooting.md).

![Hlavní okno: vlevo postranní panel Refs, uprostřed graf Logu a vpravo rozpracované změny s formulářem commitu](../images/hero.png)

## 2. Tři klávesy, které vás naučí vše ostatní

| Klávesa | Otevře |
|---|---|
| `?` | **Nápovědu**: každou klávesu a každé gesto myši. Psaním ji filtrujete. `Ctrl+D` otevře tuto dokumentaci ve vašem prohlížeči. |
| `Ctrl+P` | **Paletu příkazů**: psaním najdete příkaz, `Enter` jej spustí. Každý řádek ukazuje svou zkratku. |
| `Ctrl+,` | **Nastavení**. Otevře je také ozubené kolo v záhlaví okna vlevo od ×. |

Vše lze dělat klávesnicí i myší. Paleta dosáhne na každý příkaz. Viz
[Klávesové zkratky](keybindings.md) a [Myš a gesta](mouse-and-gestures.md).

![Paleta příkazů filtrovaná na slovo stage, se seznamem příkazů stage a jejich zkratkami](../images/palette.png)

## 3. Prohlédněte si historii

Střed okna je **Log**: jeden řádek na commit, od nejnovějšího. Ukazuje větve a
tagy jako barevné štítky, graf, zprávu, autora, datum a krátký hash.

- Klikněte na commit. Jeho podrobnosti a změněné soubory se zobrazí vpravo.
- Klikněte na změněný soubor. Jeho diff se otevře uprostřed. `Esc` jej zavře.
- `j` / `k` nebo šipky posouvají výběr. `f` hledá v commitech.
- Pravým tlačítkem na commit otevřete jeho nabídku.

![Log s vybraným merge commitem: vpravo jeho zpráva, autor a dva soubory, které přidává](../images/log-graph.png)

## 4. Proveďte změnu a přidejte ji do stage

Upravte soubor ve svém editoru. **Horní řádek Logu** nyní ukazuje rozpracované
změny: čárkovaný řádek s počtem změněných souborů. Klikněte na něj. Pravý panel
ukáže **Soubory unstaged** a **Soubory staged**.

- Najeďte na soubor a klikněte na jeho tlačítko **Stage soubor**. Nebo klikněte
  na **Stage všechny změny**.
- Kliknutím na název souboru otevřete jeho diff. Tam můžete přidat do stage
  jeden **hunk** (**Stage hunk**) nebo jeden **řádek**.

Viz [Stagování a commitování](staging-and-committing.md).

![Diff změněného souboru s tlačítky Stage hunk a Zahodit hunk nad každým hunkem](../images/diff.png)

## 5. Commit

Napište souhrn a případně popis do formuláře commitu vpravo dole. Poté
klikněte na zelené tlačítko **Commit změn v N souborech** nebo stiskněte
`Ctrl+Enter` (nebo odkudkoli `Ctrl+G Enter`). Zaškrtněte **Amend předchozího commitu**, chcete-li své změny
přidat k poslednímu commitu.

Pokud jste zapnuli [AI](ai.md), tlačítko `✦` vedle souhrnu napíše zprávu ze
změn ve stage. GitPoulpi nikdy nic necommitne, dokud nekliknete na **Commit
změn v N souborech**.

## 6. Sdílejte svou práci

Panel nástrojů má tlačítka **Pull** a **Push**, každé se seznamem `▾`. Z
klávesnice stiskněte `Ctrl+G` a poté písmeno: `Ctrl+G p` provede pull,
`Ctrl+G Shift+P` push. Po `Ctrl+G` vypíše stavový řádek písmena.

GitPoulpi používá ověřování, které už používá váš `git` (credential helper,
ssh-agent). Nepotřebuje žádný vlastní účet. Chcete-li
zobrazovat a vytvářet pull requesty, jednou připojte účet hostingové služby.
Viz [Hostingové služby a issue trackery](hosting-integration.md).

## 7. Otevření, klonování a inicializace repozitářů

Klikněte na ikonu domku na levém konci řádku karet, nebo stiskněte `Ctrl+O`.
Karta **Domů** uvádí vaše otevřené, oblíbené a nedávné repozitáře s tlačítky
**Procházet**, **Klonovat** a **Init**. Každý repozitář se otevře ve vlastní
kartě. `Ctrl+W` zavře aktuální kartu.

![Karta Domů se seznamem otevřených a nedávných repozitářů a tlačítky Procházet, Klonovat a Init](../images/home.png)

**Klonovat** se zeptá na URL a složku, nebo vám nechá vybrat repozitář z připojeného účtu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; viz [Klonování z připojeného účtu](hosting-integration.md#klonování-z-připojeného-účtu)). **Init** změní složku na nový
repozitář.
Ve všech třech dialozích (i ve výběru složky, který otevírají) `Ctrl+N` ukáže odznak na každém poli a tlačítku: stiskněte znak a kurzor skočí do pole, nebo se stiskne tlačítko.

| ![Dialog Klonovat repozitář: kam klonovat, URL, název složky a tlačítka Zrušit a Naklonovat](../images/clone.png) | ![Dialog Inicializovat repozitář: složka, název a počáteční větev](../images/init.png) |
|---|---|

### Částečný checkout

Velký repozitář nemusí skončit na disku celý. V dialogu Klonovat zaškrtněte **Částečný checkout (jen některé adresáře)** a napište adresáře, které chcete, oddělené mezerami nebo čárkami (`src docs/guide`). Checkout se provede jen pro tyto adresáře a soubory v kořeni repozitáře; celá historie se přesto stáhne, takže Log, blame a větve fungují jako obvykle. Dokud je políčko zaškrtnuté, je potřeba alespoň jeden adresář.

![Dialog Částečný checkout se dvěma zadanými adresáři a tlačítky Zrušit, Vypnout a Použít](../images/sparse-checkout.png)

Přepnout lze později libovolný repozitář: otevřete paletu příkazů (`Ctrl+P`) a spusťte **Repozitář: Částečný checkout...**, nebo použijte **Částečný checkout...** v nabídce karty repozitáře. Dialog vypíše adresáře, pro které je právě proveden checkout, v jednom poli: přidejte nebo odeberte názvy a stiskněte **Použít** (`Enter`), nebo **Vypnout (úplný checkout)**, aby se vrátil každý soubor. Dokud je pracovní strom částečný, ukazuje stavový řádek malý štítek `částečný`. Stejné ovládací prvky jsou stránkou *Nastavení > Částečný checkout* (seznam cest, jedna na řádek, s tlačítky **Zahájit**, **Ukončit** a **Obnovit**).

Poznámky:

- Je to režim cone gitu (adresáře, ne vzory); `*`, `..` a absolutní cesty se odmítají.
- Změnu sady Zpět nezaznamenává: nic se neztratí a **Vypnout** obnoví každý soubor. Git může odmítnout vyřadit adresář s necommitnutými změnami (chyba se ukáže na stavovém řádku); nejdřív je commitněte nebo stashněte.
- Totéž ze skriptů a od agentů: `gitpoulpi clone <url> --sparse src docs` a `gitpoulpi sparse-checkout status | set | add | disable` (viz [Referenční příručka CLI](cli-reference.md)); server MCP má stejné nástroje.

## Kam dál

- [Prohlídka rozhraní](interface-tour.md): název všeho, co vidíte.
- [Práce s Logem](log-and-history.md) a
  [Stagování a commitování](staging-and-committing.md): každodenní práce.
- [Motivy](themes.md) a [Nastavení](settings.md).
- [AI asistence](ai.md) a [Hostingové služby a issue trackery](hosting-integration.md):
  dvě volitelné integrace.
- [Referenční příručka CLI](cli-reference.md) a [MCP a agenti](mcp-and-agents.md):
  stejné operace ze skriptů a od agentů.
- [Řešení potíží](troubleshooting.md).
