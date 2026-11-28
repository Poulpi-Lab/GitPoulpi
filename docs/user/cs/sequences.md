[English](../sequences.md) · **Čeština**

# Sekvence

**Sekvence** je pojmenovaný seznam příkazů, které se spouštějí jeden po druhém:
například *Stage všech změn*, *AI: Vygenerovat zprávu commitu*, *Commit*, *Push*
jako jedna klávesa **Záloha**. Každý krok je příkaz
[palety příkazů](keybindings.md); sekvence je vlastní řádek palety
(`Sekvence: <název>`), lze jí přiřadit klávesu a upravuje se v
**Nastavení > Sekvence** nebo v `config.toml`.

## Stránka Sekvence

**Nastavení > Sekvence** je [stránka se seznamem](settings.md#seznamy): vlevo sekvence,
vpravo řádky vybrané sekvence. Každá změna se použije a zapíše hned a paleta
i mapování kláves ji následují bez restartu.

```
 Sekvence
+- Sekvence --------------+  Úprava sekvence 1/2
| Tato instalace          |  Název        Záloha
| > Záloha        Alt+B   |  Id           backup
|   Worktree a agent      |  Popis        Stage všeho, ...
| Tento repozitář         |  Rozsah       Tato instalace
|   Vydání                |  Zkratka      [ Alt+B ] [ Vymazat ]
+-------------------------+  Kroky        Stage all > AI: ... > Commit > Push
| [+ Přidat v] [Dup] [Odebrat]|          [ Upravit kroky (4) ] [ Spustit hned ]
| [Nahoru] [Dolů]         |
+-------------------------+
```

* **+ Přidat** nabízí **Prázdná sekvence** a tři předvolby, **Z předvolby >
  Záloha**, **Sestavit a push** a **Worktree a agent**. Předvolba je výchozí
  bod: nic není aktivní, dokud ji nepřidáte, a to, co přidáte, je kopie,
  kterou můžete upravit. Předvolby dostanou název v jazyce rozhraní, když se
  přidají.
* **Duplikovat**, **Odebrat** (nejdřív se zeptá), **Nahoru** / **Dolů** nebo tažení
  mění pořadí seznamu (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). Pořadí
  seznamu je pořadí v `config.toml`.
* Řádek s problémem je označen červeně a říká proč (žádné kroky, krok, který tato
  verze nezná, pauza `Nikdy`, které chybí potřebná volba).
  Sekvenci s problémy **lze uložit**, neznámé budoucí kroky se ponechají tak, jak
  jsou, ale její řádek palety zůstane zašedlý, dokud se neopraví.

### Řádky sekvence

| Řádek | Co dělá |
|---|---|
| **Název** | Co ukazuje paleta: `Sekvence: <název>`. |
| **Id** | Krátký název používaný v `config.toml`, na příkazové řádce a jako klíč zkratky (`a-z`, `0-9`, `_`, `-`). Přejmenováním se přesune i zkratka, v témže zápisu. |
| **Popis** | Volitelný, pro vás. |
| **Rozsah** | Viz níže. |
| **Zkratka** | Tlačítko s klávesou. `Enter` nebo klik otevře totéž zachycení klávesy jako **Klávesnice a myš**: konflikty a varování terminálu se ukazují při stisku a klávesa, kterou používá jiná akce, se odmítne stejně. **Vymazat** ji odebere. Klávesa se uloží jako `[keybindings.sequence] <id>`. |
| **Kroky** | Kroky popořadě. **Upravit kroky** je otevře. |
| **Spustit hned** | Spustí sekvenci v aktuálním okně. |

### Rozsah

Každá sekvence má rozsah, uložený s ní:

| Rozsah | Kam se ukládá | Kdo ji vidí |
|---|---|---|
| **Tato instalace** (výchozí) | `[[sequences]]` v `config.toml`, s touto instalací jako vlastníkem | Jen tato instalace GitPoulpi. |
| **Všechny instalace na tomto počítači** | `[[sequences]]` v `config.toml`, bez vlastníka | Každá instalace, která sdílí soubor. |
| **Jen tento repozitář** | `.gitpoulpi/sequences.toml` v aktuálním repozitáři | Každý, kdo repozitář má. Počítá se až poté, co jí začnete důvěřovat (viz níže). |

Změna rozsahu přesune definici: volba **Jen tento repozitář** ji zapíše do souboru
repozitáře a odebere z `config.toml`, a naopak. Rozsah repozitáře vyžaduje
otevřený repozitář.

### Sekvence repozitáře a důvěra

Soubor `.gitpoulpi/sequences.toml` lze commitnout, takže může pocházet od jiných
lidí, a sekvence spouští příkazy na vašem počítači. Sekvence repozitáře se proto
**nenabízejí, dokud jim nezačnete důvěřovat**, stejně jako příkazy commitnutého souboru
workflow.

Když repozitář má takový soubor, kterému zatím nikdo nedůvěřoval, objeví se nad Logem
oznámení: *Tento repozitář definuje 2 sekvence* s tlačítky **Zkontrolovat**, **Důvěřovat** a
**Ignorovat**. Nikdy nic neblokuje.

- **Zkontrolovat** vypíše každou sekvenci s každým krokem, jeho pauzou a volbami,
  jen pro čtení. **Důvěřovat** tam označí za důvěryhodné to, co bylo vypsáno; **Zavřít**
  (nebo `Esc`) nedůvěřuje ničemu. Je-li seznam delší než okno, Důvěřovat se tam
  nenabízí (zvětšete okno): nikdy nedůvěřujete tomu, co jste nemohli přečíst.
- **Důvěřovat** (tlačítko, nebo příkaz z palety *Sekvence: Důvěřovat sekvencím tohoto
  repozitáře*) označí soubor za důvěryhodný přesně tak, jak je teď.
- **Ignorovat** skryje oznámení, dokud se soubor nezmění.
- Z klávesnice: paleta má *Sekvence: Zkontrolovat / Důvěřovat / Ignorovat sekvence tohoto
  repozitáře*.

Zaznamená se otisk id, příkazů, pauz a voleb, uložený v lokálním stavu
repozitáře (nikdy v repozitáři). Pokud se cokoli z toho změní, například po pullu,
oznámení se vrátí jako *Sekvence tohoto repozitáře se změnily* a sekvence z palety
a z kláves zmizí, dokud jim znovu nezačnete důvěřovat. Přejmenování sekvence nebo
úprava jejího popisu nic nemění. Sekvence, kterou napíšete sami v Nastavení, zůstane
důvěryhodná, když byl soubor před vaší úpravou už důvěryhodný (nebo prázdný).

Po získání důvěry se sekvence repozitáře objeví v paletě jako `Sekvence: <název>`
stejně jako vaše vlastní a lze jim v Nastavení přiřadit zkratku. Sekvence se stejným
id jako vaše je střet: vaše vyhraje a ta druhá se nahlásí jako duplicita.

Z příkazové řádky: `gitpoulpi sequence list` ukáže sekvence souboru
s `trusted` a stavem `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (a `untrust`, `--repo PATH` pro jiný
repozitář) má spustit člověk po přečtení `sequence show`.
`sequence run` nedůvěryhodnou sekvenci repozitáře odmítne. AI agenti nemohou
důvěřovat: pro to neexistuje nástroj MCP.

## Kroky

**Upravit kroky** nahradí seznam kroky sekvence (titulek
říká **Kroky sekvence <název>**) a řádky řádky vybraného kroku. **Zpět**
(křížek nad řádky) nebo `Esc` vrátí na sekvence.

* **+ Přidat** otevře **Přidat krok**: každý příkaz, který sekvence smí spustit, pod
  svým názvem z palety (ty s AI s jiskrou), filtrovaný při psaní stejným
  porovnáváním jako paleta. Krok, který tato verze nemůže spustit, je zašedlý
  s důvodem. `Enter` nebo klik ho přidá na konec; `Esc` nebo křížek
  okno zavře.
* **Duplikovat**, **Odebrat**, **Nahoru** / **Dolů** a tažení fungují jako u
  sekvencí.
* Destruktivní příkaz (smazat větev, zahodit změny...) se nikdy
  nenabízí: sekvence žádný spustit nemůže.

### Pauza

Každý krok má **Pauzu**:

| Pauza | Význam |
|---|---|
| **Výchozí** | Krok se zastaví, jen když potřebuje odpověď, na svém obvyklém dialogu (název větve, zpráva commitu, která je ještě prázdná...). |
| **Vždy** | Zastavit před tímto krokem a zeptat se, zda pokračovat. |
| **Nikdy** | Nikdy se nezastavit. Volby kroku mu musí dát vše potřebné, takže krok, který se ptá na název, potřebuje vyplněnou volbu **Název**. |

Krok, který se před spuštěním vždy zeptá (force push), nenabízí
**Nikdy**: řádek říká proč. **Nikdy** u kroku zprávy od AI je povoleno
jako výslovná volba.

### Volby

Řádky pod **Pauzou** jsou volby kroku: textový **Název** nebo **Zpráva**,
**Větev**, políčko jako **Otevřít v nové kartě**, nebo rozbalovací seznam
naplněný z vaší konfigurace (**Remote**, **Profil AI**, položka **Otevřít
v**). Texty přijímají proměnné `${branch}`, `${date}`, `${time}` a
`${n}` (číslo, které název zajistí jedinečný). Vymazání textu volbu odebere.

## Vytvoření a úprava pomocí AI

Se [zapnutou AI](ai.md#návrh-sekvence) (Nastavení > AI) se objeví dvě položky:
**Přidat > Vytvořit pomocí AI...** (také řádek palety `Sekvence: Vytvořit pomocí AI...`,
který otevře Nastavení na této stránce s dialogem) a **Upravit pomocí AI...** vedle
**Spustit hned** u vybrané sekvence. Při vypnuté AI neexistuje ani jedna.

```
+- Vytvořit sekvenci pomocí AI ----------------------------- x -+
| Popište sekvenci, kterou chcete                               |
| +-----------------------------------------------------------+ |
| | stage všeho, napsat zprávu commitu pomocí AI,             | |
| | commit a pak push                                         | |
| +-----------------------------------------------------------+ |
| Enter přidá řádek. Ctrl+Enter odešle. Esc zavře.              |
| [ ] Poslat kontext (názvy remote, profily AI, popisky Otevřít v) |
|    Vypnuto: odešle se jen váš popis.                          |
| AI jen navrhuje. Před přijetím se nic neuloží...              |
| [ Odeslat ] [ Zrušit ]                                        |
+---------------------------------------------------------------+
```

Napíšete, co chcete, vlastními slovy; **Odeslat** (nebo `Ctrl+Enter`, `Ctrl+S`)
zeptá se modelu, přes týž profil, ukazatel úloh a `Esc` jako každá
jiná akce AI (a stejnou otázku „Co se odešle“). Odpověď je
**návrh**, který si přečtete, než se cokoli stane:

* název, id, popis a kroky s jejich pauzou a volbami;
* u **Upravit pomocí AI** rozdíl proti současné sekvenci: `+` přidaný krok,
  `-` odebraný krok, `~` krok se změněnou pauzou nebo volbami, `~ Název: „a“
  -> „b“` u pole;
* **Vynecháno**: krok, který model navrhl a který není povolený (neznámý,
  destruktivní, sekvence uvnitř sekvence, nemožná pauza), s
  důvodem; platné kroky zůstanou;
* **poznámky** modelu a **navržená zkratka**, která se jen ukáže, s
  informací, zda je klávesa už obsazená. Nikdy se nepřiřadí: použijte potom
  tlačítko **Zkratka**.

```
+- Navržená změna ------------------------------------------ x -+
| + přidáno   - odebráno   ~ změněno                            |
| ~ Název: „Denní“ -> „Denní 2“                                 |
| Kroky (3)                                                     |
| + 1. Git: Fetch All Remotes                                   |
|   2. Git: Stage All Changes                                   |
| - Git: Commit...                                              |
| ~ 3. Git: Push: výchozí -> Pauza: Vždy                        |
| Vynecháno                                                     |
| krok 4 `reset_hard`: destruktivní kroky nejsou povoleny       |
| [ Přijmout ] [ Upravit před uložením ] [ Zamítnout ] [ Zpět ] |
+---------------------------------------------------------------+
```

* **Přijmout** zapíše návrh jedním uložením (nová sekvence, nebo náhrada
  upravované, která si ponechá své id, rozsah a klávesu) a vybere ji v seznamu.
* **Upravit před uložením** udělá totéž a otevře její kroky, abyste mohli upravit
  řádky (tato stránka ukládá každou úpravu hned, takže sekvence je už
  uložená).
* **Zamítnout** nezapíše nic. **Zpět** vrátí k popisu, abyste ho poslali znovu.
* **Přijmout** je nedostupné, když není co přijmout: žádný použitelný krok,
  problém, který by stránka označila, nebo úprava, která nic nemění.

Před **Přijmout** se nic nezapíše a sekvence se odsud nikdy nespustí.
Zavření dialogu (křížek, `Esc`, klik mimo) se zeptá „Zahodit změny?“,
dokud je napsaný popis nebo otevřený návrh; zavření Nastavení se zeptá také.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<this installation's id>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Klíče, které tato verze nezná, a kroky, které nezná, se při uložení stránky
ponechají.

## Spuštění sekvence

Sekvence je pojmenovaný seznam kroků, každý je příkaz palety (stage všeho, napsat zprávu commitu pomocí AI, commit, push...). Spusťte ji z palety (`Sekvence: <název>`), klávesou, kterou jste jí dali v Nastavení > Klávesnice a myš (`[keybindings.sequence]` v `tui.toml`), nebo z příkazové řádky (`gitpoulpi sequence run`).

Při běhu ukazuje pás nad horním okrajem Logu `<název> k/n`, krok a na co čeká. Běh je vidět i v panelu Jobs jako `Sekvence <název> k/n: <krok>`. Najednou běží jedna sekvence; spuštění druhé řekne, která běží.

**Kde se zastaví, aby se zeptala.** Každý krok má nastavení `pause` (`default`, `always`, `never`):

| Krok | Výchozí | `pause = "never"` |
|---|---|---|
| Zpráva commitu od AI | zpráva se objeví v poli zprávy; pás říká „Zkontrolujte zprávu a pak Pokračovat“ | pokračuje hned |
| Commit | prázdná zpráva se vždy zastaví: pole má klávesnici a váš commit (nebo Pokračovat) pokračuje | nedostupné, když není zpráva k commitu |
| Sestavení commitů pomocí AI | otevře se kontrola; Použít pokračuje, zavření ji zruší | použije návrh tak, jak je |
| Vytvořit worktree | dialog se otevře s vyplněným názvem | vytvoří ho z `options.name` bez dialogu |
| Push | při `git.push_behavior = ask` se otevře výběr „Push do...“ | pushne na `options.remote` |
| Force push s lease, stage souborů, které vypadají jako tajné | vždy se zeptá | nepovoleno |

`pause = "always"` na libovolném kroku ukáže před jeho spuštěním „Další: <krok>“ s tlačítky **Pokračovat** a **Zrušit**. Pokud je push odmítnut, protože se větev rozešla, otevře se obvyklý dialog a pás říká „Push byl odmítnut: vyberte v dialogu“; pull a pak push (nebo force push s lease) běh protáhne, zavření dialogu ho zruší.

**Pokračovat** je tlačítko pásu, `Sekvence: Pokračovat` v paletě, nebo klávesa přiřazená `sequence.continue`. **Zrušit** je zavírací křížek pásu, `Sekvence: Zrušit` (`sequence.cancel`), panel Jobs, nebo `Esc`, když ho nic jiného nechce (otevřený dialog se zavře první, což také zruší). Zrušení ponechá kroky, které už proběhly, a řekne to: „<název> zrušena po kroku k/n (<krok>); dřívější kroky zůstávají“.

**Když krok selže**, běh se tam zastaví: stavový řádek pojmenuje sekvenci, krok a chybu, pás zčervená s tlačítkem **Detaily** (záznam Logu aktivity, který vypíše volby kroku) a zavíracím křížkem a dřívější kroky zůstanou provedeny.

**Worktrees.** Krok vytvoření worktree s `open_tab = true` přesune běh do karty nového worktree: další kroky (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) jednají tam. Pokud je na obrazovce jiná karta, pás říká „čeká se na kartu <název>“ s tlačítkem **Přejít tam**. Zavření té karty běh zruší. Běh nepřežije ukončení GitPoulpi.

### Agent navrhující spuštění

Se zapnutým *Nechat AI agenty navrhovat změny, které potvrdím* (`[mcp]
allow_ui_propose`) může agent požádat vaše okno o spuštění sekvence
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Objeví se banner:
*✦ Navrhuje <agent>: spustit sekvenci <název>* s tlačítky **Spustit** a **Zamítnout**.
Nic se nespustí, dokud nestisknete Spustit (klik, nebo `Ctrl+N` a pak `Enter`);
Zamítnout, `Esc` (když ho nic jiného nechce) nebo příkaz z palety *Agent: Zobrazit
čekající pohled* ho zahodí. Běh pak probíhá přesně, jako byste ho spustili vy,
a zastavuje se tam, kde to sekvence říká. Návrh se odmítne, když sekvence
neexistuje, je neplatná, nebo je to sekvence repozitáře, které jste nedůvěřovali.

## Vrácení běhu

Každý krok, který mění repozitář (stage, commit, tag, větev, stash, worktree...), zanechá **svůj vlastní obyčejný záznam Zpět**, označený během, takže `Ctrl+Z` dál vrací po jednom kroku a [Historie vrácení](undo-redo.md#historie-vrácení) na nich ukazuje `⟳ <název>`. Pro vrácení celého běhu je **Vrátit celou sekvenci**: tlačítko Historie vrácení (`s`), řádek palety `Úpravy: Vrátit celou sekvenci`, nebo `gitpoulpi undo --sequence`. Vrací záznamy běhu od nejnovějšího, každý s vlastními kontrolami a otázkami (commit, který už byl pushnut, se zeptá předem), zastaví se u prvního odmítnutí s pojmenováním kroku a starší nechá v žurnálu. Zpráva od AI a push žádný záznam nezanechají: push se nikdy nevrací. Dotkne se jen žurnálu aktuálního repozitáře; kroky provedené ve worktree se vracejí z onoho repozitáře. Znovu zůstává po záznamech. Podrobnosti: [Běhy sekvencí](undo-redo.md#běhy-sekvencí).
