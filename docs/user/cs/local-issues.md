[English](../local-issues.md) · **Čeština** · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Lokální issues a Obsidian

Mějte své issues jako složku poznámek Markdown: bez účtu, bez tokenu, bez sítě. Složka může být uvnitř
repozitáře (`docs/issues/`, commitovaná spolu s kódem) nebo kdekoli na disku, například podsložka
vaultu [Obsidian](https://obsidian.md). GitPoulpi poznámky čte a zapisuje. Obsidian, jakýkoli editor nebo shell je může
upravovat zároveň a seznamy to sledují.

![Lokální issues Markdown v postranním panelu, s nabídkou řádku jedné poznámky: otevřít poznámku, zavřít ji, upravit její štítky nebo milník](../images/local-issues.png)

## Nastavení

1. **Nastavení > Integrace > Issue trackery > Local Markdown > Přidat profil.** Napište složku, nebo stiskněte
   **Procházet...** a vyberte ji. Relativní složka (`docs/issues`) se čte vůči každému repozitáři, se kterým je
   propojena; absolutní (`/home/me/Vault/Projects/Issues`) je všude stejná. Žádný token se nezadává.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. V repozitáři otevřete sekci **ISSUES** postranního panelu a vyberte poskytovatele **Local Markdown - <name>** (složka
   se volí spolu s ním; rozbalovací seznam repozitáře uvádí složku a její přímé podsložky, chcete-li jinou).
   - CLI: `gitpoulpi hosting link issues --id <profile>` (složka profilu je výchozí `--project`).

Nic se nepropojuje samo: jako každý zdroj issues je to dobrovolné pro každý repozitář zvlášť.

## Komentáře k poznámce

`Enter` na řádku poznámky v sekci ISSUES nebo na její kartě otevře její konverzaci: text poznámky a pak komentáře. Komentáře se uchovávají v samotném souboru poznámky, v závěrečné sekci `## Comments`, každý jako záznam `### <autor> - <čas UTC>` (upravený přidá `(edited <čas>)`). Nic nad tímto nadpisem se nikdy nemění, takže Obsidian i shell čtou a upravují tentýž soubor. Můžete psát, upravovat a mazat komentáře lokálního autora (`local.me` profilu, jinak připojeného uživatele, tedy jména, které používá *Přiřazené mně*); nikoho jiného. Pole je obvyklé Markdown textové pole s náhledem a *Píšete jako <jméno> (lokální poznámka)* pojmenuje autora. Nic nejde po síti. Poznámka s nadpisem `## Comments`, který jste napsali ručně, se čte jako taková.

## Poznámky

Jedna poznámka na issue, pojmenovaná `<id>-<slug>.md` (`12-crash-on-empty-name.md`), s blokem frontmatter:

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
Free Markdown. GitPoulpi never rewrites the body.
```

- Nová issues dostávají `max(id) + 1`.
- GitPoulpi zapisuje jen vlastnosti, které mu patří (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Každá jiná vlastnost, tělo, konce řádků a pořadí klíčů se zachovají bajt po bajtu.** Zápisy jsou
  atomické (dočasný soubor, potom přejmenování).
- Čtení je shovívavé: poznámka bez `id` ho převezme z názvu souboru, poznámka bez `status` má `todo`, poznámka
  bez frontmatter převezme název z prvního `# nadpisu`. Soubor, který není issue (`readme.md`), se ignoruje.
- Stav je volný text. **Otevřené** znamená „není stav hotovo“.

### Komentáře v poznámce

Komentář k poznámce se zapisuje přímo do poznámky, do poslední sekce `## Comments` (bez sítě a v Obsidianu se čte bez potíží):

```markdown
(text poznámky)

## Comments

### kai - 2026-11-15T10:00:00Z
První komentář, **libovolný Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
Druhý.
```

- Záznam začíná řádkem, který je přesně `### <autor> - <čas UTC>`, po úpravě s ` (edited <čas UTC>)`; jeho text sahá k dalšímu záznamu. Sekce je poslední nadpis `## Comments` mimo blok kódu a sahá do konce souboru, takže za ni nic nepište.
- Řádek komentáře, který by se četl jako záznam nebo jako nadpis, se uloží s úvodním zpětným lomítkem (`\### ...`, které Markdown zobrazí jako stejný text) a čte se zpět bez něj.
- Odeslání přidá na konec a vše nad tím nechá bajt po bajtu (včetně konců řádků); úprava nebo smazání přepíše jen sekci. Odeslání také posune `updated` poznámky (poznámce bez frontmatter se žádné nepřidá).
- Autorem je lokální autor profilu („já“ z nastavení Lokální issues). Upravovat nebo mazat můžete jen vlastní komentáře; id komentáře je jeho čas a autor a dvě odeslání ve stejné vteřině se rozliší posunutím druhého o vteřinu.
- U poznámky nejsou žádné odpovědi, žádné interní komentáře a žádná historie událostí.

## Co lze dělat

| V sekci ISSUES | Klávesnice / paleta |
|---|---|
| Nové issue (`+`, nebo `n`): název, popis, stav, řešitelé, štítky, milník | `Hosting: Nové issue...` |
| Upravit řešitele / štítky / milník z nabídky řádku. Vyberte z toho, co složka už používá, nebo **napište nový štítek či login** a vezměte řádek `Přidat „...“` | `Hosting: Upravit štítky...` a spol. |
| **Zavřít issue** / **Znovu otevřít issue** (nastaví stav hotovo, nebo první jiný) | `Lokální issues: Zavřít issue...` / `Znovu otevřít issue...` |
| **Otevřít poznámku** (váš výchozí program pro `.md`) | `Integrace: Otevřít issue...` |
| **Otevřít v Obsidianu** (jen když je složka uvnitř vaultu, tedy nad ní je složka `.obsidian/`) | `Lokální issues: Otevřít v Obsidianu...` |
| Filtry: Otevřené / Zavřené / Všechny, „Přiřazené mně“ | rozbalovací seznam filtru |

„Přiřazené mně“ odpovídá nastavení `me` profilu, jinak připojenému jménu, jinak názvu profilu.

Větve a commity fungují jako u GitHubu: „Vytvořit větev z issue“ dá `12-crash-on-empty-name` a `#12` ve zprávě
commitu nebo `12-...` v názvu větve se rozpozná.

Změny provedené mimo GitPoulpi (Obsidian, shell, `git pull`) se ukážou zhruba do vteřiny: názvy souborů složky
a časy změn se kontrolují ve vlákně na pozadí, nikdy ve vlákně vykreslování.

## Konfigurační soubor

V `config.toml`, vedle profilu (každý klíč je volitelný):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # the board's column order
done_status = "done"                            # closes an issue
reference_prefix = "LOC"                         # shown as LOC-12 instead of #12
me = "kai"                                       # who "assigned to me" means
```

Stavy, které poznámky používají, ale nejsou v `statuses`, následují za uvedenými. `statuses` je pořadí sloupců
[tabule](#tabule-kanban) (`gitpoulpi hosting issue options` ho vypíše).

## Tabule Kanban

![Tabule složky poznámek: jeden sloupec na stav, karty s řešiteli, štítky a milníkem](../images/issue-board.png)

Pokud pracujete sami, nebo v týmu, který drží issues vedle kódu, je složka také tabulí Kanban.
**Záhlaví ISSUES > Tabule**, nebo příkaz z palety **Issues: Otevřít tabuli**, ji otevře přes okno; `Esc` ji zavře.

- **Sloupce** jsou stavy: nejdřív ty uvedené v `statuses`, v tomto pořadí, potom každý další stav, který poznámka používá.
  Prázdný sloupec se přesto zobrazí. Každé záhlaví nese počet karet.
- **Karty** ukazují číslo issue, název (dva řádky), řešitele jako iniciály, štítky a milník.
  Když se sloupce nevejdou všechny, tabule se posouvá do stran (lišta pod ní, kolečko se `Shift`, nebo šipky).
- **Přesun karty** tažením na jiný sloupec (sloupec se pod ukazatelem rozsvítí), klávesami
  `Shift+Left` / `Shift+Right`, nebo příkazem **Tabule: Přesunout kartu do...** (`m`), rozbalovacím seznamem sloupců. Karta se přesune
  hned; `status` poznámky se přepíše jako v [sekci o poznámkách](#poznámky), nic jiného v souboru. Když
  zápis selže, karta se vrátí a chyba je na stavovém řádku.
- **Otevření** karty klávesou `Enter` nebo dvojklikem: v okně se otevře [konverzace](conversations.md) poznámky s jejím textem a komentáři (**Otevřít poznámku** v nabídce karty otevře soubor ve vašem výchozím programu pro `.md`). `e` upraví její
  řešitele, štítky nebo milník, `.` nebo pravé tlačítko otevře nabídku karty, `n` založí nové issue ve vybraném
  sloupci (formulář se po zavření vrátí na tabuli).
- **Filtrování** podle řešitele (`a`), štítku (`l`), milníku (`M`) nebo textu (`/`); `x` je zruší. Filtry jsou
  rozbalovací seznamy: psaním zužujete, `Enter` vybere.
- **Živě**: poznámka změněná z Obsidianu nebo shellu přesune svou kartu do vteřiny.

Každé gesto myší má klávesu a příkaz z palety (**Tabule: ...**, uvedené na stránce
[klávesové zkratky](keybindings.md)).

## Obsidian Bases

**Paleta: `Lokální issues: Vytvořit Obsidian Base`** (CLI: `gitpoulpi hosting issue base`) zapíše `issues.base` vedle
poznámek, jednou: pohled *tabulka* a pohled *kanban* seskupený podle `status`, v pořadí `statuses`. Otevřete ho v Obsidianu, abyste
tytéž issues viděli jako tabuli i tam. GitPoulpi nikdy nepřepíše existující soubor `.base` (váš ani předchozí): odmítne
a řekne to. Formát se řídí syntaxí Bases Obsidianu
(<https://obsidian.md/help/bases/syntax>); pohled kanban potřebuje novější Obsidian.

## CLI a MCP

Stejné příkazy jako u každého zdroje issues fungují i na něm: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Čtyři další jsou pro poznámky:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` a `hosting issue base`.
Viz [Referenční příručka CLI](cli-reference.md).

## Omezení

- Nový **milník** nelze v poli výběru napsat (vyberte takový, který složka už používá), ale `milestone:` v poznámce může
  být cokoli a ukáže se, jakmile se uloží.
- Issues jsou jen poznámky přímo ve složce (ne v podsložkách).
- Mazání neexistuje: poznámku odstraňte ve správci souborů nebo v Obsidianu (jeho koš je váš).
