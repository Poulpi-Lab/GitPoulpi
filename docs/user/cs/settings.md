[English](../settings.md) · **Čeština** · [Deutsch](../de/settings.md) · [Español](../es/settings.md) · [Français](../fr/settings.md) · [Italiano](../it/settings.md) · [日本語](../ja/settings.md) · [한국어](../ko/settings.md) · [Nederlands](../nl/settings.md) · [Polski](../pl/settings.md) · [Português (Brasil)](../pt-BR/settings.md) · [Português (Portugal)](../pt-PT/settings.md) · [Русский](../ru/settings.md) · [Svenska](../sv/settings.md) · [Türkçe](../tr/settings.md) · [Українська](../uk/settings.md) · [简体中文](../zh-CN/settings.md)

# Nastavení

Nastavení otevřete pomocí `Ctrl+,`, `Ctrl+N` a poté `,`, ozubeného kola v záhlaví
okna (vlevo od `×`) nebo `Ctrl+P` > *Nastavení*. Obrazovka Nastavení po dobu
otevření nahrazuje okno. Zavřete ji tlačítkem `✕` vpravo nahoře, klávesou `Esc`
nebo `q`. Zeptá se jen tehdy, když by zavření něco ztratilo (viz
[Zavření Nastavení](#zavření-nastavení)).

Změny se ukládají do `config.toml` okamžitě. [Referenční příručka
konfigurace](config-reference.md) popisuje každý klíč, jeho výchozí hodnotu a
proměnnou prostředí. Tato stránka popisuje obrazovku Nastavení.

![Obrazovka Nastavení: vyhledávací pole a seznam stránek vlevo, stránka Obecné vpravo](../images/settings.png)

## Práce s obrazovkou Nastavení

| Chcete | Postup |
|---|---|
| Přepnout mezi seznamem stránek a volbami | `Tab` / `Shift+Tab`, nebo klikněte. |
| Pohybovat se | `j` / `k` nebo šipky. |
| Změnit volbu | `Enter` nebo `Space`, nebo klikněte na její hodnotu. Viz níže. |
| Zavřít | Klikněte na `✕`, nebo stiskněte `Esc` či `q`. Dokud je otevřené pole, rozbalovací seznam nebo potvrzení, `Esc` zavře jen to. |

Co `Enter` nebo `Space` udělá, závisí na řádku:

- Zaškrtávací políčko se přepne.
- Rozbalovací seznam se otevře. Šipkami vyberete, `Enter` potvrdí.
- Textové nebo číselné pole se začne upravovat. Fungují šipky, `Home`, `End`,
  `Backspace` a `Delete` a kliknutím se umístí kurzor. Hodnotu uloží `Enter`,
  `Tab`, `Up`, `Down` nebo kliknutí jinam. `Esc` úpravu zruší. Odmítnutá hodnota
  zůstane v poli červeně a stavový řádek uvede důvod.
- Řádek s tlačítky (**Vytvořit**, **Odebrat**, **Vyzkoušet**...) ukazuje jen tlačítka,
  která jsou teď použitelná. `Left` / `Right` vybírají, `Enter` stiskne tlačítko, nebo na
  něj klikněte. Tlačítko, které něco maže, se nejdřív zeptá.

Na každé stránce začínají hodnoty v jednom sloupci, takže popisky jsou zarovnané vlevo
a pole vpravo. Text nebo číslo je tónovaný pruh, výběr je pruh zakončený `▾`,
zaškrtávací políčko je `[x]`. Dlouhý popisek se zalomí místo oříznutí a v úzkém okně
se hodnota přesune pod svůj popisek.

Zašedlý řádek uvádí, proč je neaktivní. Například **Síla podbarvení řádku** je
zašedlá, dokud je **Pozadí drah grafu** nastaveno na **Vypnuto**.

### Seznamy

Stránky, které upravují několik položek stejného druhu (Profily, AI, Workflow,
Otevřít v, Obrázky), je ukazují jako **seznam vlevo a pole vybrané položky vpravo**.
V úzkém okně je seznam nad poli. Seznam má všude stejnou podobu:

- Značka řádku říká jeho stav: `●` aktivní profil, `★` výchozí položka, `⚠`
  nedokončená položka (profil bez e-mailu), s důvodem na místě, kde by byly podrobnosti.
- Pod seznamem: **+ Přidat**, **Duplikovat**, **Odebrat**, `↑` a `↓`. Tlačítko, které
  teď nejde použít (`↑` na prvním řádku), je ztlumené. **+ Přidat ▾** otevře
  rozbalovací seznam, kde stránka nabízí předvolby nebo druhy.
- **Odebrat** se nejdřív zeptá, kromě položky, kterou jste před chvílí přidali a
  neupravili.
- Řádek přesunete tažením; proužek ukazuje, kam dopadne, a `Esc` tažení zruší.
  Dvojklik přejde na pole položky.

| Klávesa | Co dělá |
|---|---|
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Vybere položku. |
| `Insert` | Přidá položku (je-li rozbalovací seznam, otevře ho). |
| `Ctrl+D` | Duplikuje položku. |
| `Delete` | Odebere položku po potvrzení. |
| `Alt+Up` / `Alt+Down` | Posune položku nahoru nebo dolů. |
| `Enter` nebo `Right` | Přejde na pole položky. |
| `Left` | Zpět na seznam stránek. |
| `Alt+Left` / `Alt+Right` | Přepíná mezi kartou Obecné a kartou seznamu (kdekoli na stránce). |

Každá z těchto stránek má pod názvem dvě **podkarty**, aby se nastavení stránky nikdy nepletlo s vlastností vybrané položky: **Obecné**, vlastní nastavení stránky (aktivní profil, výchozí položka, pokyny, které dostává každá akce AI, šablona pracovního postupu...), a kartu pojmenovanou podle položek (**Profily**, **Profily (připojení)**, **Položky**, **Editory**, **Pravidla**), která obsahuje seznam a pole vybrané položky. Stránka se otevře na kartě **Obecné**; nastavení si do ukončení aplikace pamatují kartu, na které jste každou stránku opustili, a také pozici posunu a vybraný řádek každé stránky (a karty). Klikněte na kartu nebo stiskněte `Alt+Left` / `Alt+Right`. Hledání, řádky nastavení v paletě příkazů a odkazy „nastavte v nastavení“ otevřou kartu, na které nastavení leží.
[Klávesnice a myš](#klávesnice-a-myš) má stejné dvě karty (**Obecné**, **Klávesové zkratky**).

Klávesy lze přemapovat v **Klávesnice a myš** (kontext *preferences*, viz
[Klávesové zkratky](keybindings.md)).

### Víceřádkové editory

Nastavení, která obsahují více řádků (například cesty částečného checkoutu, popis šablony commitu, základní větve předcházení konfliktům a hooky workflow), se otevírají v malém editoru klávesou `Enter` nebo kliknutím; `Ctrl+S` text ponechá a `Esc` ho zahodí. Myš tam funguje jako v každém textovém poli:

- Kliknutí umístí kurzor tam, kam jste klikli.
- Tažením se vybírá text; kolečko posouvá.
- Dvojklik vybere slovo pod ukazatelem.
- `Shift` + kliknutí rozšíří výběr od kurzoru k ukazateli.

Klávesnice nabízí totéž: `Shift` se šipkami vybírá, `Ctrl` se šipkami skáče po slovech, `Ctrl+A` vybere vše, `Ctrl+C` a `Ctrl+X` kopírují a vyjímají.

### Zavření Nastavení

`✕`, `Esc` a `q` dělají totéž. Nastavení použije každou změnu hned, takže se většinou
prostě zavře. Zeptá se **Zavřít Nastavení?** jen tehdy, když by se něco ztratilo:

- stránka **Workflow** má neuložené změny;
- šablona zprávy commitu má neuložené změny;
- profil nemá e-mail autora (commity, které vytvoří, nelze přiřadit vám).

Okno nabízí **Přejít tam** (otevře danou stránku), **Zahodit a zavřít** nebo **Přesto
zavřít** a **Zrušit**. Malé editory uvnitř Nastavení (text instrukcí AI, hook workflow,
cesty částečného checkoutu) mají vlastní `✕` a zeptají se **Zahodit změny?** jen tehdy,
když byl jejich text upraven.

### Vyskakovací okna nad Nastavením

Titulní řádek ukazuje chobotnici a název aplikace jako hlavní okno. Kliknutím na
chobotnici otevřete O aplikaci.

Několik oken platných pro celou aplikaci se otevírá nad Nastavením a vrací ho tak,
jak bylo (stránka, poloha posunu a neuložené změny Workflow nebo šablon zůstanou):

- paleta příkazů (`Ctrl+P`),
- výběr motivu (`Ctrl+N t`, **Aplikace: Změnit motiv...**), který ukazuje náhled každého
  motivu přímo na Nastavení; `Esc` vrátí předchozí motiv,
- výběry jazyka a barev grafu (`Ctrl+N` a pak stejná písmena jako v hlavním okně),
- hledání v pracovním prostoru (`Ctrl+Alt+F`),
- O aplikaci (`Ctrl+N a`, chobotnice nebo **Nápověda: O aplikaci GitPoulpi**) a okno nápovědy (`?`).

Každé zavřete pomocí `Esc`, vlastního `✕` nebo kliknutím mimo něj. Příkazy, které
působí na to, co Nastavení zakrývá (karta, větev, diff...), jsou v paletě uvedeny, ale
zašedlé s textem „nejdřív zavřete Nastavení“, a akordy `Ctrl+N` pro ně ukazují
tento řádek na spodním okraji Nastavení: zavřete Nastavení a pak je spusťte.

`Ctrl+N` zobrazí odznak u každého ovládacího prvku stránky: vyhledávacího pole, seznamu stránek (jeden odznak, na aktuální stránce), podkaret, seznamu a jeho tlačítek, každého pole na obrazovce a křížku zavření. Stiskněte znak odznaku a stane se totéž jako při kliknutí na daný prvek: textové pole se otevře k úpravě, zaškrtávací políčko se přepne, rozbalovací seznam se otevře, tlačítko se provede. `Esc` zruší. Pole posunuté mimo obrazovku odznak nemá. Když do pole píšete nebo je otevřený rozbalovací seznam, editor či hledání, `Ctrl+N` patří jim. Viz [Odznaky](keybindings.md#odznaky).

### Vyhledávací pole

Pole nad seznamem stránek prohledává název a popis každé volby na každé stránce.
Klikněte do něj, nebo stiskněte `/` či `Ctrl+F`, a pište. Rozbalí se seznam
shod. Každá shoda zobrazuje volbu a její stránku. `Up` / `Down` vybírají,
`Enter` přejde na volbu, `Esc` seznam zavře.

![Hledání „diff“ v Nastavení: odpovídající volby ze stránek Log a graf, Diff a editor a Git a kódování](../images/settings-search.png)

## Nastavení z palety příkazů

Většinu nastavení lze změnit bez otevření této obrazovky: napište `Nastavení:` nebo `Settings:` do palety příkazů (`Ctrl+P`). Řádky se vytvářejí z polí samotných stránek, takže nové nastavení se objeví na obou místech.

- Zaškrtávací políčko je **Nastavení: Přepnout ...**. Přepne se hned a řádek ukazuje `[x]` nebo `[ ]`.
- Rozbalovací seznam je **Nastavení: ...**. `Enter` zúží paletu na jeho hodnoty, aktuální je označena `aktuální`; `Enter` na hodnotě ji vybere.
- Číslo nebo text otevře políčko s aktuální hodnotou. Hodnota, kterou by stránka odmítla (není číslo, mimo rozsah), je odmítnuta stejně a políčko zůstane otevřené.
- Nastavení, které potřebuje svou stránku (seznam, kombinace s možnostmi zjištěnými za běhu), otevře Nastavení na ní.

![Paleta příkazů filtrovaná na Settings: s jedním řádkem pro každé nastavení a jeho aktuální hodnotou vpravo](../images/palette-settings.png)

Hodnota se uloží do `config.toml` a použije se přesně jako na stránce. Anglická slova klíče nastavení (`remember tabs`) najdou řádek v jakémkoli jazyce.

## Stránky

Seznam stránek má dvě skupiny:

- **Nastavení**: globální nastavení.
- **Nastavení repozitáře**: nastavení repozitáře aktivní karty. Zobrazují se
  pouze při otevřeném repozitáři.

### Obecné

![Rozbalovací seznam Jazyk na stránce Obecné: Auto a sedmnáct jazyků s vyhledávacím polem](../images/settings-language.png)

| Volba | Co dělá |
|---|---|
| **Jazyk** | Jazyk rozhraní: **Auto** (jazyk systému, jinak angličtina), English, Français, Español, Português (Brasil), Português (Portugal), Italiano, Deutsch, 简体中文, 日本語, 한국어, Русский, Українська, Polski, Čeština, Nederlands, Svenska, Türkçe. Projeví se okamžitě, bez restartu. Řídí se jím i názvy měsíců a data v Logu. Volbu přebíjí proměnná prostředí `GITPOULPI_LANG` (například `GITPOULPI_LANG=fr`). Výstup CLI a MCP zůstává anglicky. |
| **Překládat výrazy gitu** | Ve výchozím stavu vypnuto: commit, push, pull, stash, tag a další výrazy gitu zůstávají ve všech jazycích anglicky. Zapnuto: použije se vlastní slovník gitu daného jazyka (francouzsky: *Tirer*, *Pousser*, *Remiser*, *Étiquette*). Projeví se okamžitě. |
| **Anglická slova git** | Pouze pro japonštinu, korejštinu a čínštinu: zobrazí *stage*, *unstage*, *staged* jako anglická slova místo přepisu. Nemá účinek, dokud je zapnuto **Překládat výrazy gitu**. |
| **Pamatovat si karty** | Při spuštění znovu otevře karty z poslední relace. Když je vypnuto, spuštění uvnitř repozitáře otevře jen tento repozitář a spuštění mimo repozitář začne na kartě Domů bez karty repozitáře. Vyžaduje restart. |
| **Povolit Radar** | Ve výchozím stavu zapnuto (`general.radar_enabled`). Vypnuto skryje kartu Radaru, jeho příkazy v paletě a `Alt+R`; na pozadí se pro něj nic nestahuje. Projeví se hned. Viz [Radar](radar.md#zavření-karty-nebo-vypnutí-radaru). V sestavení bez Radaru se neukazuje. |
| **Karta při spuštění v repozitáři** | Karta, na které se aplikace otevře, když složka, v níž se spouští, leží uvnitř repozitáře (`general.startup_tab_in_repo`): **Aktuální repozitář** (výchozí, karta tohoto repozitáře, v případě potřeby otevřená), **Poslední karta** (karta aktivní při posledním ukončení), **Domů**, **Radar**. Repozitář zadaný přes `--repo` vždy otevře vlastní kartu. **Poslední karta** je Domů, když je vypnuto **Pamatovat si karty** nebo si nic nepamatuje; **Radar** je Domů, když je Radar vypnutý nebo není v sestavení. Čte se jednou při startu; vyžaduje restart. |
| **Karta při spuštění mimo repozitář** | Karta, na které se aplikace otevře, když složka, v níž se spouští, není repozitář (například uživatelská složka) (`general.startup_tab_outside_repo`): **Domů** (výchozí), **Poslední karta**, **Radar**. **Poslední karta** je Domů, když je vypnuto **Pamatovat si karty** nebo si nic nepamatuje; **Radar** je Domů, když je Radar vypnutý nebo není v sestavení. Čte se jednou při startu; vyžaduje restart. |
| **Karta při spuštění ze spouštěče** | Karta, na které se aplikace otevře, když ji spustí zástupce na ploše nebo v nabídce Start, který GitPoulpi vytvořil (předává `--launcher`), ať je složka jakákoli (`general.startup_tab_launcher`): **Poslední karta** (výchozí), **Domů**, **Radar**. Má přednost před dvěma nastaveními výše. **Poslední karta** je Domů, když je vypnuto **Pamatovat si karty** nebo si nic nepamatuje; **Radar** je Domů, když je Radar vypnutý nebo není v sestavení. Zástupci vytvoření starší verzí dostanou přepínač při dalším spuštění. Čte se jednou při startu; vyžaduje restart. Ve Windows se při dalším spuštění přepíše také kopie připnutá na hlavní panel a spuštění bez okolního shellu (dvojklik, samotný připnutý exe) se počítá jako spuštění ze spouštěče. |
| **Seskupovat karty worktree** | Ve výchozím stavu zapnuto (`general.group_worktree_tabs`). Karty propojených worktrees repozitáře stojí hned za kartou repozitáře a odtažená karta worktree se k ní vrátí. Vypnuto nechá karty, kde je umístíte. Zapnutí otevřené karty znovu seskupí. |
| **Výchozí složka pro klonování** | Odkud začíná dialog Klonovat. Prázdné: `%USERPROFILE%\source\repos` ve Windows, jinde `~/src`. |
| **Dialog Klonovat se otevře na** | Zdroj, na kterém se dialog Klonovat otevře (`repos.clone_start_source`): **Naposledy použitý** (výchozí, zdroj, který zobrazoval při posledním zavření, naklonováním nebo zrušením), **Klonovat z URL**, nebo jedna z vašich připojených integrací. Zdroj, který už není dostupný, se vrátí na Klonovat z URL. Klonování zahájené z Radaru si ponechá vlastní zdroj. |
| **Výchozí složka pro procházení** | Kde se otevře Procházet. Prázdné: poslední použitá složka, jinak domovská složka. |
| **Umístění dat repozitáře** | Kde GitPoulpi drží svá data jednotlivých repozitářů (`session.per_repo_state_dir`): **Uvnitř .git** (výchozí: v pracovním stromu se nic neobjeví, všechny worktree ho sdílejí), **V pracovním stromu** (složka `.gitpoulpi`; potřebná, chcete-li commitnout týmový `workflow.toml`) nebo **V mé složce s uživatelskými daty** (mimo repozitář). Složka `.gitpoulpi` ponechaná starší verzí se při první potřebě přesune na nové místo a po vyprázdnění se odstraní; soubor sledovaný gitem (commitnutý `workflow.toml`) se nikdy nepřesune ani nesmaže a zůstane čitelný. |
| **Výchozí název větve** | První větev repozitáře, který vytvoříte pomocí Init. Prázdné: rozhodne git. Nepřejmenovává žádnou existující větev. |
| **Úroveň protokolu**, **Uchovávat soubory protokolu**, **Maximální velikost souboru protokolu** | Protokol aplikace za Protokolem aktivity: nejpodrobnější zaznamenávaná úroveň (`log.level`), jak dlouho se soubory uchovávají (celé číslo a jednotka, dny, týdny, měsíce nebo roky, na jednom řádku: kliknutím na číslo ho napíšete, Nahoru a Dolů ho mění po jedné, Tab přejde na jednotku a Enter otevře její seznam; nejvýše 1 až 10 let, výchozí 1 týden, `log.retention_days` a `log.retention_unit`) a velikost, při které se soubor jednoho dne přesouvá (číslo a MiB nebo GiB, 1 MiB až 1 GiB, `log.max_file_mb` a `log.max_file_unit`). |
| **Zobrazit dobu trvání operací**, **Rozšířené protokolování** | Karta Operace gitu v Protokolu aktivity: doba trvání každé operace (`log.timings`, ve výchozím stavu zapnuto) a také práce na pozadí, obnovení a auto-fetch (`log.extended`, ve výchozím stavu vypnuto). Viz [Když něco selže](troubleshooting.md#protokol-aktivity). |
| **Zpět povoleno**, **Hloubka historie Zpět**, **Zaznamenávat změny provedené mimo GitPoulpi** | Skupina **Historie vrácení**. Zapíná a vypíná Zpět a nastavuje, kolik operací uchová (výchozí 100, `0` znamená neomezeně). Vypnutí Zpět po potvrzení smaže historii vracení. Poslední volba zaznamenává i to, co shell nebo agent udělá s větvemi, tagy, stashi a HEADem. Viz [Zpět a znovu](undo-redo.md). |

Příkaz z palety **Aplikace: Změnit jazyk...** (`Ctrl+N l`) otevře stejný seznam
jazyků. Zvýrazněný jazyk se zde ukazuje živě; `Enter` ho ponechá a `Esc`
obnoví předchozí.

### Spouštěč

Stránka **Spouštěč** (`gitpoulpi ui show settings launcher`) vytvoří zástupce, který spouští
GitPoulpi ze spouštěče systému ve zvoleném terminálu.

| Řádek | Co dělá |
|---|---|
| **Terminál pro zástupce** | `ui.launch_terminal`: **Automaticky**, **Výchozí v systému** nebo jeden z terminálů nalezených na tomto počítači (stejné zjišťování jako u **Otevřít v**). Automaticky: ve Windows Windows Terminal (jinak WezTerm, Alacritty, běžná konzole), v Linuxu `xdg-terminal-exec` (Omarchy; jinak první nalezený terminál), v macOS Terminal, pokud není nainstalován iTerm2, Ghostty, WezTerm nebo kitty. |
| **Po dvojkliku znovu otevřít v tomto terminálu** | Jen Windows (`ui.relaunch_in_terminal`). Dvojklik na exe otevře výchozí konzoli, která špatně kreslí obrázky: GitPoulpi se znovu otevře ve zvoleném terminálu a první okno se zavře. Neprovádí se, když byl spuštěn z terminálu. |
| **Při kliknutí zobrazit otevřené okno** | `ui.reuse_window` (ve výchozím stavu zapnuto). Kliknutí na zástupce GitPoulpi (`gitpoulpi launch`) zobrazí okno už otevřeného GitPoulpi místo spuštění dalšího. Nový terminál se spustí, jen když žádný není otevřený, nastavení je vypnuté nebo okno nemůže přejít do popředí. Do popředí se dostane ve Windows, ve WezTermu a v Hyprlandu; na jiných linuxových desktopech kliknutí stále spustí nové okno. V macOS se nepoužívá (Dock to už dělá). Podržením **Shiftu** při kliknutí (Windows) nebo akcí spouštěče **New Window** (Linux) otevřete další okno záměrně; ze skriptu to udělá `gitpoulpi launch --new-window`. |
| **Alias příkazu** | `ui.command_alias`: rozbalovací seznam s volbami **Žádný** a `gp`, nebo napište libovolný název (písmena, číslice, `-`, `_`, `.`; bez mezer a oddělovačů cesty). Tlačítko **Vytvořit** na řádku zařídí, že tento název spouští GitPoulpi (jakmile alias existuje, změní se na **Odebrat** a odebere všechny aliasy, které GitPoulpi vytvořil). Pod řádkem se zobrazí upozornění, když název už na vaší PATH něco spouští, a ve Windows i když PowerShell má vestavěný alias téhož jména (`gp` je tam `Get-ItemProperty` a vyhrává nad jakýmkoli souborem). Alias nebo funkci shellu odsud nevidíte: oh-my-zsh definuje `gp` samo a vyhrává nad naším. |
| **Alias nové karty** | `ui.command_alias_add`: druhý příkaz, například `gpa`, který otevře aktuální složku jako novou kartu již spuštěného GitPoulpi (`gitpoulpi --add`), místo aby spustil další; když žádné neběží, spustí GitPoulpi jako obvykle, a když jich běží několik, zeptá se, ke kterému se připojit. Rozbalovací seznam s volbami **Žádný**, `gpa` a název aliasu s příponou `a`, nebo napište libovolný název (stejná pravidla jako výše). Jedno tlačítko **Vytvořit** vytvoří oba příkazy, **Odebrat** odebere všechny příkazy, které GitPoulpi vytvořil; každý má vlastní řádek stavu. V Linuxu a macOS je to dvouřádkový skript, ve Windows `.cmd`. Viz [Referenční příručka CLI](cli-reference.md#přidání-složky-do-otevřeného-okna). |
| **Přidat GitPoulpi do PATH** | Přidá na vaši PATH pro nové terminály složku, odkud GitPoulpi běží, a složku s aliasem. Tlačítko řádku se jmenuje **Přidat** nebo **Odebrat** a **Odebrat** odebere přesně to, co bylo přidáno. Složky, které na PATH už jsou, se nechají být (řádek pak říká **už v PATH**). |
| **Linux: Spouštěč aplikací** | Zapíše `~/.local/share/applications/gitpoulpi.desktop` (respektuje `XDG_DATA_HOME`) a ikonu do `~/.local/share/icons/hicolor/256x256/apps`. V Omarchy je položka stejná jako ta, kterou vytváří `omarchy-tui-install` (`xdg-terminal-exec --app-id=TUI.tile -e <exe>`); pro ostatní terminály odpovídající příkaz (`kitty`, `wezterm start --`, `alacritty -e`, `ghostty -e`, `foot`, ...). |
| **Linux: Plocha** | Stejná položka ve složce Plocha, pokud ji máte. |
| **Windows: Nabídka Start** | `GitPoulpi.lnk` v `%APPDATA%\Microsoft\Windows\Start Menu\Programs`. Kopie připnutá na hlavní panel sama sleduje změny ikony a barvy; pokud její ikona zůstane stará, odepněte ji a připněte znovu (pravé tlačítko na položce v nabídce Start, potom **Připnout na hlavní panel**). |
| **Windows: Plocha** | `GitPoulpi.lnk` na vaší ploše. |
| **macOS: Složka Aplikace** | `~/Applications/GitPoulpi.app`, malý balíček, který otevře terminál se spuštěným GitPoulpi. |

Každý řádek ukazuje svůj stav: **nevytvořeno**, **vytvořeno**, **zastaralé, aktualizuje se...** (exe se
přesunulo nebo je zvolen jiný terminál; GitPoulpi ho do sekundy samo přepíše, takže to uvidíte jen zřídka) nebo „je tam soubor,
který nevytvořil GitPoulpi“ (soubor toho jména existuje a GitPoulpi ho nevytvořil, takže se
ho nikdy nedotkne). **Odebrat** maže jen soubory, které vytvořil GitPoulpi (nesou značku
`X-GitPoulpi=1`). Ikonou je chobotnice v barvě vašeho loga (okno O aplikaci, klávesa `c`); na Linuxu má každá
barva vlastní název ikony, takže spouštěč, který ikony ukládá do mezipaměti, ukáže novou barvu. Když je zástupce jednou vytvořen, udržuje se sám aktuální: změníte-li barvu, nebo se přesune terminál či exe, GitPoulpi přepíše zástupce, které vytvořil (nikdy takového, který nevytvořil, a nikdy žádného nevytváří), při spuštění i za běhu. Tlačítko Aktualizovat neexistuje; je jen **Vytvořit** (pro zástupce, který ještě neexistuje) a **Odebrat**.

**Seskupování na hlavním panelu a v docku.** GitPoulpi běží v okně terminálu, takže hlavní panel ho obvykle ukazuje pod ikonou terminálu, bez tečky „běží“ pod připnutou ikonou GitPoulpi. Windows: zástupci nesou AppUserModelID `GitPoulpi.GitPoulpi` a GitPoulpi spuštěný z některého z nich dá oknu svého terminálu stejné ID a svou ikonu (okno najde tak, že mu na okamžik nastaví jedinečný titulek a pak ho obnoví), takže okno leží pod připnutou ikonou GitPoulpi. Okno terminálu s více kartami se přesouvá jako celek: všechny jeho karty se seskupí pod ikonou GitPoulpi, dokud v něm GitPoulpi běží. Dříve vytvořené zástupce a připnutí na hlavním panelu přepíše stejná synchronizace; pokud připnutí stále ukazuje ikonu terminálu, odepněte ho a připněte znovu. Linux: kitty, foot, Alacritty, WezTerm a Ghostty se spouštějí s třídou okna `gitpoulpi` a záznam `.desktop` ji uvádí (`StartupWMClass`), takže dock seskupí okno pod ikonu spouštěče; ostatní terminály si nechávají vlastní ikonu. macOS: není co dělat, Dock seskupuje podle aplikace.

![Nastavení > Spouštěč: terminál pro zástupce a řádek spouštěče aplikací](../images/settings-launcher.png)

#### Alias příkazu a PATH podle systému

| Systém | Alias | PATH |
|---|---|---|
| Linux, macOS | Symbolický odkaz `~/.local/bin/<name>` na `gitpoulpi-tui`. Odkaz nemůže nést značku, takže se počítá za náš, když jeho cíl je soubor jménem `gitpoulpi-tui`; běžný soubor nebo odkaz na cokoli jiného **není náš** a nikdy se nenahradí ani neodebere. | Blok mezi `# >>> GitPoulpi PATH (X-GitPoulpi=1) >>>` a `# <<< GitPoulpi PATH <<<` v `~/.bashrc` a `~/.zshrc` (kterékoli existují; `~/.zshrc` v macOS, jinak `~/.bashrc`, když neexistuje ani jeden) a `~/.config/fish/conf.d/gitpoulpi-path.fish`, je-li fish nastaven. Odebrání smaže blok (nebo soubor fish) a nic víc. |
| Windows | `%LOCALAPPDATA%\Programs\GitPoulpi\bin\<name>.cmd`, krátký zástupný skript s řádkem `REM X-GitPoulpi=1` (nastavuje také `GITPOULPI_CMDCMDLINE`, podle čeho se pozná spuštění z adresního řádku). | Uživatelská PATH v `HKCU\Environment`, zapsaná přes PowerShell, potom oznámení `WM_SETTINGCHANGE`, aby ji nové programy viděly. Hodnota `GitPoulpiPathAdded` eviduje, co se přidalo, a Odebrat vyjme jen to. |

Změnu PATH vidí nové terminály, ne ty, které už jsou otevřené. Žádná z obou akcí se nezapisuje do historie vrácení: Zpět pro ně není, cestou zpět je **Odebrat**.

Paleta nabízí stejné akce (**Spouštěč: Vytvořit zástupce**, **Spouštěč: Vytvořit zástupce na ploše**,
**Spouštěč: Odebrat zástupce**, **Spouštěč: Vytvořit alias příkazu**, **Spouštěč: Odebrat alias příkazu**, **Spouštěč: Přidat do PATH**, **Spouštěč: Odebrat z PATH**) a `gitpoulpi desktop shortcut
status|install|remove [--where launcher|desktop|startmenu]` totéž udělá ze skriptu.

#### Psaní do adresního řádku Průzkumníka souborů (Windows)

Napište alias (`gp`) nebo `gitpoulpi` do adresního řádku okna Průzkumníka souborů a stiskněte Enter: GitPoulpi se otevře v terminálu zvoleném v **Terminál pro zástupce**, ve složce, kterou okno zobrazuje, bez otevření **Poslední karta**. Je-li zapnuto **Při kliknutí zobrazit otevřené okno** a GitPoulpi už běží, stane se z této složky místo toho nová karta onoho okna.

- **`gp` (alias příkazu).** Jakmile alias existuje, jeho vytvoření také zaregistruje `gp.exe` v `HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths` s odkazem na `gitpoulpi-launch.exe` a Odebrat tento klíč smaže (jen když je náš). Když je složka s aliasem v PATH (**Přidat GitPoulpi do PATH**), Windows spustí nejdřív zástupný skript `.cmd` a před zobrazením GitPoulpi může na okamžik blikat okno konzole. Skript zapsaný starší verzí se přepíše automaticky, když Nastavení alias čtou.
- **`gitpoulpi`.** Funguje s nainstalovanou **Nabídka správce souborů**: Nainstalovat také zaregistruje `gitpoulpi.exe` v App Paths. Bez ní se `gitpoulpi` najde jen tehdy, když je složka GitPoulpi v PATH, a konzole pak může blikat jako výše.

Průzkumník souborů si ponechává PATH, který měl při vašem přihlášení do Windows: složka přidaná do PATH později se v něm projeví až po odhlášení a opětovném přihlášení. Do té doby se najdou jen položky App Paths a název, který Windows nenajdou, se otevře jako webová adresa ve vašem prohlížeči.

Linux a macOS nic podobného nemají: adresní řádky jejich správců souborů programy nespouštějí. Použijte položku **Nabídka správce souborů** nebo alias příkazu v terminálu.

#### Nabídka správce souborů (Otevřít v GitPoulpi)

Pod zástupci přidávají řádky **Nabídka správce souborů** položku **Otevřít v GitPoulpi** do nabídky složek po kliknutí pravým tlačítkem. Každý správce souborů má jeden stavový řádek (v Linuxu jeden pro každého zde nalezeného správce a pro každého, který položku ještě drží; řádek **Nalezeno** vypíše, co se našlo), pak tlačítka **Nainstalovat** a **Odebrat**. **Nainstalovat** zapíše položku pro každého nalezeného správce souborů; **Odebrat** se nejdřív zeptá a pak vyjme přesně to, co Nainstalovat zapsalo, ať je ten správce souborů ještě nainstalován, nebo ne.

Položka spouští `gitpoulpi launch --folder <složka>` (ve Windows `gitpoulpi-launch.exe --folder` bez konzole) a až při kliknutí ten příkaz rozhodne: je-li zapnuto **Při kliknutí zobrazit otevřené okno** a okno GitPoulpi běží, stane se složka uvnitř repozitáře novou kartou naposledy použitého okna, které se přesune do popředí; jinak GitPoulpi ve složce spustí terminál zvolený v **Terminál pro zástupce**. Změna terminálu proto položku nikdy nezastará; přesunutý exe nebo nová barva loga ano a GitPoulpi své položky přepíše automaticky (nikdy žádnou nevytvoří), stejně jako zástupce. Položka potřebuje pomocný spouštěč vedle GitPoulpi (`gitpoulpi`, ve Windows `gitpoulpi-launch.exe`); bez něj to řádek řekne a tlačítka zůstanou vypnutá.

| Systém | Co Nainstalovat zapíše |
|---|---|
| Windows | Klíče pro jednoho uživatele, bez práv správce: `HKCU\Software\Classes\Directory\shell\GitPoulpi`, `Directory\Background\shell\GitPoulpi` a `Drive\shell\GitPoulpi`, každý s podklíčem `command`, takže se položka ukáže na složce, na pozadí otevřené složky i na jednotce. Každý klíč nese hodnotu `X-GitPoulpi`; klíč bez ní **není náš** a nikdy se ho nedotkne. Také zaregistruje `gitpoulpi.exe` v `HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths` s odkazem na `gitpoulpi-launch.exe`. Ve Windows 11 je položka jen v klasické nabídce: **Zobrazit další možnosti**, Shift+F10 nebo Shift+pravé tlačítko (nová nabídka nejvyšší úrovně uvádí jen zabalené aplikace). Odebrat smaže naše klíče a potom nadřazené klíče, které vytvořilo Nainstalovat, jen když zůstaly prázdné. |
| Linux | Jen pro správce souborů nalezené v PATH. GNOME Files a Caja: skript pojmenovaný podle položky v `~/.local/share/nautilus/scripts` nebo `~/.config/caja/scripts` (v podnabídce **Skripty**). Dolphin: `~/.local/share/kio/servicemenus/gitpoulpi-open.desktop`, spustitelný (KDE Frameworks 6 uživatelskou servisní nabídku, která není spustitelná, ignoruje; KDE Frameworks 5 před 5.85 se nepodporuje). Nemo: dvě akce v `~/.local/share/nemo/actions` (vybraná složka, pozadí). Thunar: jedna akce sloučená do `~/.config/Thunar/uca.xml`; vaše vlastní akce zůstanou a Odebrat vrátí soubor bajt po bajtu (nebo ho smaže, pokud ho vytvořilo Nainstalovat). PCManFM a PCManFM-Qt: `~/.local/share/file-manager/actions/gitpoulpi-open.desktop`. Ikona jde do `~/.local/share/icons/hicolor/256x256/apps/gitpoulpi-folder.png` a odchází s poslední položkou. `XDG_DATA_HOME` a `XDG_CONFIG_HOME` se respektují. Položka se ukáže na vybrané složce a na pozadí složky tam, kde to správce souborů nabízí. |
| macOS | Rychlá akce `~/Library/Services/Open in GitPoulpi.workflow`: klikněte pravým tlačítkem na složku ve Finderu, potom **Rychlé akce**. Jen vybrané složky (Finder nemá nabídku pro pozadí složky). Při prvním kliknutí se macOS zeptá, zda smí služba workflow ovládat váš terminál (výzva Automatizace, například „WorkflowServiceRunner.xpc chce získat přístup k ovládání ...“): povolte to. Pokud položka chybí, zapněte ji v Nastavení systému > Klávesnice > Klávesové zkratky > Služby. |

Každý soubor a klíč, který GitPoulpi zapíše, nese značku `X-GitPoulpi=1` (ve skriptu řádek `# X-GitPoulpi=1`) a Odebrat maže jen označené. Tyto akce mění váš systém, ne repozitář: Zpět je nepokrývá, cestou zpět je **Odebrat**. Adresní řádky správců souborů v Linuxu a macOS programy nespouštějí, takže tam není co psát; terminál má alias příkazu.

Paleta má **Spouštěč: Nainstalovat nabídku Otevřít v GitPoulpi** a **Spouštěč: Odebrat nabídku Otevřít v GitPoulpi** a `gitpoulpi desktop folder-menu status|install|remove` totéž udělá ze skriptu.

### Synchronizace

Jak GitPoulpi udržuje repozitář v souladu s remote repozitáři a co si mezitím
počne s vašimi místními změnami.

![Stránka Synchronizace: interval automatického načítání jako číslo a jednotka (minuty) na jednom řádku, pod ním další volby](../images/settings-quantity.png)

| Volba | Co dělá |
|---|---|
| **Interval Auto-Fetch** | V tomto intervalu načítá všechny remote na pozadí, číslo a jednotka (minuty nebo hodiny) na jednom řádku. Výchozí 5 minut, maximum 60 minut (1 hodina), `0` funkci vypne. `general.auto_fetch_interval_minutes` zůstává v minutách; `general.auto_fetch_interval_unit` si jen pamatuje, jak se zobrazuje. Vyžaduje restart. |
| **Fetch při otevření karty** | Při otevření karty repozitáře spustí na pozadí Fetch všech remote. Ve výchozím stavu zapnuto. Repozitář načtený během posledních 60 sekund se znovu nenačítá. |
| **Auto-Prune** | Fetch také odstraní remote-tracking větve, které na remote už neexistují. Ve výchozím stavu zapnuto, takže větev smazaná na serveru (například na GitLabu) zmizí ze sekce REMOTE po dalším fetchi. Nelze vrátit zpět: odstraněná větev se vrátí při dalším fetchi, pokud se na remote znovu objeví. |
| **Automaticky mazat tagy** | Fetch také smaže lokální tagy, které na remote smazali. Týká se jen tagů, které na remote byly: tag, který jste nikdy neodeslali (push), zůstane. Ve výchozím stavu vypnuto, stejně jako v gitu. Vyžaduje git 2.41 nebo novější. CLI: `gitpoulpi fetch --prune-tags`; konfigurační klíč `general.auto_prune_tags`. Viz [Větve, tagy a postranní panel Refs](log-and-history.md#větve-tagy-a-postranní-panel-refs). |
| **Udržovat submoduly aktuální** | Po checkoutu nebo pull aktualizuje submoduly na commity, které repozitář eviduje. Viz [Submoduly](submodules.md). |
| **Po merge mazat soubory „.orig“** | Po dokončení merge smaže záložní soubory `*.orig`. Ve výchozím stavu vypnuto. |
| **Auto-stash před checkoutem, pull, merge nebo rebase** | Co dělat, když místní změny blokují některou z těchto operací: **zeptat se** (dialog nabídne uložit změny do stash, provést operaci a stash obnovit), **vždy** (provede se bez ptaní) nebo **nikdy** (odmítne se). |
| **Zahrnout nesledované soubory do auto-stash** | Automatický stash bere i nesledované soubory. |

Též na této stránce, ve skupině **Aktualizace**:

| Volba | Co dělá |
|---|---|
| **Kontrola aktualizací** | Rozbalovací seznam (`updates.mode`): **Oznamovat** (výchozí) jednou denně na pozadí zkontroluje a v záhlaví okna ukáže `vX.Y.Z k dispozici`; **Auto** navíc stáhne aktualizaci na pozadí, byl-li GitPoulpi nainstalován z archivu vydání (kdy restartovat, volíte vy); **Vypnuto** neodešle nikdy žádný požadavek. Platí okamžitě. Viz [Aktualizace](updates.md). |
| **Kanál aktualizací** | Rozbalovací seznam (`updates.channel`): **Stabilní** (výchozí) nebo **Předběžný** (i kandidáti na vydání). Platí okamžitě. |
| **Nainstalováno přes** | Jen pro čtení. Jak se GitPoulpi dostal do tohoto počítače (archiv vydání, instalační program pro tohoto uživatele nebo všechny uživatele, Chocolatey, winget, Scoop, Homebrew, apt, balíček MSI, AUR, obraz Dockeru, cargo nebo vývojové sestavení). Určuje, jak se aktualizace použijí, viz [Aktualizace](updates.md). |

Řádek, který nastavil váš správce v politice aktualizací platné pro celý počítač, ukazuje **Nastaveno vaším správcem** a nelze ho tady změnit.

Též na této stránce, ve skupině **Přepis historie**, časové limity externích skriptů (viz [Přepis historie](history-rewrite.md)):

| Volba | Co dělá |
|---|---|
| **Časový limit odpovědi skriptu** | Jak dlouho má externí skript na odpověď u jednoho commitu, číslo a jednotka (milisekundy nebo sekundy) na jednom řádku. Výchozí 5 sekund, od 1 milisekundy do 10 minut (`rewrite.script_timeout_ms`, `rewrite.script_timeout_unit`). Při překročení se ukončí celý proces a přepis selže, aniž by se cokoli přesunulo. |
| **Časový limit skriptu na průchod** | Jak dlouho smí trvat celý jeden průchod externího skriptu, číslo a jednotka (sekundy, minuty nebo hodiny). Výchozí 1 hodina, nejvýše 24 hodin (`rewrite.script_total_timeout_s`, `rewrite.script_total_timeout_unit`). |

### Profily

![Stránka Profily](../images/settings-profiles.png)

Profily identity pro commity: jméno a e-mail použité pro commity vytvořené v
GitPoulpi. Karta **Obecné** obsahuje rozbalovací seznam **Aktivní profil**; karta
**Profily** je [seznam](#seznamy) (přidání, duplikování, odebrání, změna pořadí)
s poli vybraného profilu.
Každý profil má **ID profilu**, **Název profilu**, **Jméno autora** a
**E-mail autora**. Profil bez e-mailu má v seznamu `⚠`. Bez aktivního profilu, nebo s
profilem s prázdným jménem či e-mailem, platí hodnoty `user.name` a
`user.email` samotného repozitáře.

Při dvou a více profilech zobrazuje panel nástrojů na pravém konci výběr profilu.
Nastavuje profil pouze pro aktivní repozitář. Po výběru dialog nabídne zapsat
identitu také do git konfigurace tohoto repozitáře. Příkaz z palety je
**Repozitář: Změnit profil...**.

Tyto profily nejsou profily hostitelů ze stránky Integrace ani profily AI ze
stránky AI.

### SSH a přihlášení

![Stránka SSH a přihlášení](../images/settings-ssh.png)

SSH používá systémové `ssh`, takže váš ssh-agent i konfigurace ssh dál
fungují. GitPoulpi nikdy nečte obsah klíčů.

| Volba | Co dělá |
|---|---|
| **Cesta k privátnímu klíči** | Je-li nastavena, fetch, pull, push a clone spouštějí git s `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`, takže ssh nabízí pouze tento klíč. Prázdné: platí agent a vaše konfigurace ssh beze změny. |
| **Cesta k veřejnému klíči** | Používá se, jen když je cesta k privátnímu klíči prázdná: ssh vybere odpovídající klíč držený ssh-agentem (nebo hardwarový klíč). |
| **Typ klíče** | Typ klíče, který vytvoří **Vygenerovat pár klíčů**: `ed25519`, `rsa` (4096 bitů) nebo `ecdsa` (NIST P-256). |
| **Přihlašovací údaje HTTPS** | Jak se fetch, pull, push a clone autentizují přes HTTPS: **credential_helper** (vlastní helper gitu, výchozí), **prompt** (helpery se ignorují; git se zeptá v terminálu) nebo **none** (bez přihlašovacích údajů, jen veřejné remote). |
| **Zapomenout všechna jména a hesla** | Odstraní všechny tokeny, které GitPoulpi uložil do vašeho credential helperu. Nejdřív se zeptá na potvrzení. |
| **Vygenerovat pár klíčů** | Vytvoří pár klíčů zvoleného typu (**Typ klíče**) v `~/.ssh/gitpoulpi_<type>` bez hesla a zapíše jeho dvě cesty. Nikdy nepřepíše existující soubor. |

Cesty ke klíčům se uplatní při dalším přenosu. Předpona `~/` znamená vaši
domovskou složku.

Nastavení provedené jinde vždy vyhraje nad těmito cestami. Pořadí od nejvyšší
priority:

0. Klíč zvolený pro jeden repozitář na jeho vlastní stránce
   [SSH a přihlášení](#nastavení-repozitáře-ssh-a-přihlášení) (skupina pro repozitář).
1. `GIT_SSH_COMMAND` nebo `GIT_SSH` v prostředí, ve kterém GitPoulpi běží.
2. `core.sshCommand` v git konfiguraci repozitáře nebo ve vaší globální git
   konfiguraci. Položka *Klíč SSH...* v nabídce karty starších verzí ji zapisovala pro jeden
   repozitář; SSH a přihlášení nabídne její přesun do GitPoulpi.
3. Dvě cesty ke klíčům z této stránky.
4. Vlastní výchozí chování ssh a agent.

Klíč může patřit i k účtu hostitele; viz
[Hostingové služby a issue trackery](hosting-integration.md#klíč-ssh-pro-profil).

### Integrace

![Stránka Integrace: poskytovatelé vlevo a tlačítko Připojit pro GitHub](../images/settings-integrations.png)

Připojuje vaše účty. Skupina **Hosting** obsahuje GitHub, GitHub Enterprise,
GitLab, GitLab Self-Managed, Azure DevOps, Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea, Forgejo a Gerrit. Skupina **Issue trackery** obsahuje Jira Cloud, Jira Data Center, Trello, Linear, YouTrack, Redmine a Local Markdown. Každý poskytovatel má **Připojit**, pole pro token, oprávnění,
která token potřebuje, a odkaz na stránku tokenů hostitele. Tato stránka má
vlastního průvodce: [Hostingové služby a issue trackery](hosting-integration.md).

Nad poskytovateli jsou dvě volby. Dostanete se k nim stisknutím `Up` na prvním
poskytovateli.

| Volba | Co dělá |
|---|---|
| **Obnovení: pull requesty a issues** | Jak často se obnovují seznamy v postranním panelu a čip na panelu nástrojů, číslo a jednotka (minuty nebo hodiny). Výchozí 5 minut, nejvýše 2 hodiny, `0` časovač vypne (`general.hosting_refresh_minutes`, `general.hosting_refresh_unit`). |
| **Odesílat jako** | Jméno, pod kterým se zprávy odesílají hostiteli, když je token repozitáře botův (token GitHub App), například `my-app[bot]`; slouží také k rozpoznání vašich vlastních komentářů. Prázdné: zeptá se hostitele, jinak se naučí z první odeslané zprávy. Repozitář ho může přepsat v Nastavení > Nastavení repozitáře > SSH a přihlášení > Hosting (`general.hosting_post_as`). |
| **Terminologie žádosti o kontrolu kódu** | Slovo používané pro hostované review: **Auto (podle hostitele)**, **Pull Request** nebo **Merge Request**. |
| **Skrýt čip pull requestu, když je uzavřen nebo sloučen** | Ve výchozím stavu zapnuto (`ui.hide_finished_pr_chip`; na GitLabu se jmenuje „merge request“). Čip na panelu nástrojů ukazuje nejnovější žádost přepnuté větve; jakmile je tato žádost uzavřena nebo sloučena, skryje se a místo něj se ukáže tlumený čip **+ PR** (založit novou žádost), jako by větev žádnou neměla. Vypnuto: čip ukazuje nejnovější žádost vždy, ať je v jakémkoli stavu. Platí okamžitě. |

### AI

![Stránka AI: Enable AI, globální instrukce a bloky pro jednotlivé akce](../images/settings-ai.png)

Na kartě **Obecné**: přepínač, který AI zapíná, výchozí profil, instrukce, které
dostává každá akce, a profil a instrukce pro každou akci. Karta **Profily (připojení)**
je [seznam](#seznamy) profilů AI s vloženou tabulkou proměnných prostředí profilu. Blok *Kontrola kódu*
na kartě Obecné obsahuje také výchozí hodnoty [kontextu kontroly](ai.md#kontext-kontroly): **Spustit agenta
v repozitáři** (*Podle profilu*, *Ano*, *Ne*), **Zahrnout CLAUDE.md**, **Zahrnout AGENTS.md** a **Další soubor**
(cesta nebo odkaz; prázdné nepřidá žádný, s tlačítkem **Procházet...** pod ním, které otevře výběr souboru; `Enter` na tlačítku je jeho klávesová podoba). Tato stránka má vlastního průvodce: [Asistence AI](ai.md).

Blok **Odpovědi na hostiteli** drží to, co navrhuje komentáře, odpovědi a popisy issue: jeho profil a instrukce, **Základní prompt** (prázdné použije vestavěný) a skupinu **Přístup k repozitáři**: **Nechat AI číst repozitář ve výchozím stavu** (vypnuto: volba v každém textovém poli začíná nezaškrtnutá), **Zachovat relaci agenta mezi odpověďmi** a **Uchovat kontext repozitáře po dobu (minuty)**. Bajtové limity **Největší konverzace k odeslání pro odpověď**, **Největší soubor repozitáře pro odpověď**, **Největší mapa repozitáře pro odpověď**, **Nejvíc nálezů na hledaný výraz pro odpověď** a **Nejvíc hledaných výrazů pro odpověď** jsou u ostatních limitů vstupu. Viz [Pomoc s psaním na hostiteli](ai.md#pomoc-s-psaním-na-hostiteli).

Pod bloky jednotlivých akcí je skupina **Soukromí**:

| Volba | Co dělá |
|---|---|
| **Před odesláním skrýt tajné hodnoty** | Zapnuto (výchozí): tokeny, soukromé klíče a hesla nalezené v požadavku se před odesláním nahradí `[REDACTED:<rule>]`. Vypnuto: červený řádek říká, že se tajné hodnoty odesílají beze změny. |
| **Vynechat citlivé soubory** | Zapnuto (výchozí): změny souborů `.env`, soukromých klíčů a certifikátů (`[privacy] sensitive_patterns`) nejsou nikdy součástí požadavku. Vypnuto: červený řádek říká, že se odesílají, když jsou součástí požadavku. |
| **Zobrazit, co se odesílá, před...** | Jeden přepínač na akci (zpráva commitu, vysvětlení, zpráva stashe, popis pull requestu, composer commitů, řešení konfliktů, kontrola kódu, rozdělení commitu): ukáže požadavek v dialogu **Co se odešle**, než odejde. Pole *Příště nezobrazovat* v dialogu jeden vypne. Akce přepisu historie ukazují vlastní dialog. |
| **Zobrazit vše znovu** | Objeví se, dokud je některý přepínač vypnutý; zapne všechny zpět. |

Viz [AI > Soukromí](ai.md#soukromí).

### Agenti a MCP

Co smějí AI agenti dělat v tomto okně, zda smí běžet server MCP a jak je chráněn
jeho endpoint HTTP. Řádky MCP se zobrazují jen v sestaveních, která server
obsahují; viz [MCP a agenti](mcp-and-agents.md).

| Volba | Co dělá |
|---|---|
| **Server MCP** | Zapnuto (výchozí): agent může spustit server (`gitpoulpi mcp serve`). Vypnuto: odmítne startovat. Ukládá se jako `mcp.enabled`. Změna se projeví při dalším spuštění serveru agentem; již běžící server si ponechá svá nastavení. |
| **Port serveru MCP** | Port, na kterém `--http` naslouchá, není-li zadána adresa, 1 až 65535 (výchozí 8808). Ukládá se do portu `mcp.bind`; jeho hostitelská část zůstává (`127.0.0.1`, je-li prázdná). Projeví se při dalším spuštění serveru agentem. |
| **Povolit AI agentům ovládat zobrazení** | Umožní agentům ukázat vám v tomto okně commit, soubor nebo oznámení (`ui show`). Vypnuto: žádný agent okno neovlivní. |
| **Nechat AI agenty navrhovat změny k mému potvrzení** | Umožní agentům otevírat předvyplněné dialogy Reword, commit, větev nebo tag, které potvrdíte vy. Ve výchozím stavu vypnuto. Viz [MCP a agenti](mcp-and-agents.md#oprávnění). |
| **Otevřít okno pro agenty**, **Příkaz ke spuštění (custom)** | Zda smí agent otevřít okno GitPoulpi, když žádné není otevřené, a pomocí kterého terminálu nebo multiplexeru. Ve výchozím stavu vypnuto. |
| **Ověřování HTTP** | Zapnuto (výchozí): endpoint HTTP serveru MCP odpovídá jen na požadavek s přístupovým tokenem. Vypnuto ukáže červené varování: jakýkoli program, který se k endpointu dostane, může spustit každý nástroj povolený v nastavení MCP, a `--allow-remote` se odmítne. Ukládá se jako `mcp.http_auth`. |
| **Přístupový token** | Token, maskovaný. Maskovaná hodnota je tlačítko: `Enter` (nebo klik) ji ukáže celou na řádku pod ní a znovu ji skryje. **Kopírovat** ji vloží do schránky bez zobrazení ve stavovém řádku, **Obnovit** ji po potvrzení nahradí (klienti se starým tokenem jsou okamžitě odmítnuti, restart serveru není potřeba), **Vytvořit** udělá první. `Left` / `Right` se pohybují mezi tlačítky a `Enter` jedno stiskne. Stejný token je `gitpoulpi mcp token`. |
| **TLS** | **Vypnuto** (výchozí), **Self-signed** (server jednou vytvoří certifikát a ukáže jeho otisk) nebo **Soubory** (váš certifikát a klíč). Server ukončuje TLS sám. Ukládá se jako `mcp.tls`; platí od dalšího spuštění serveru. |
| **Cesta k certifikátu**, **Cesta ke klíči** | Soubory PEM používané se **Soubory** (`mcp.tls_cert`, `mcp.tls_key`). |
| **Otisk certifikátu** | Jen pro čtení: otisk SHA-256 certifikátu, který endpoint předkládá, aby ho klienti mohli ověřit. |
| **Za tunelem nebo reverzní proxy** | Vaše prohlášení, že provoz šifruje tunel nebo proxy před endpointem, což umožní běh `--allow-remote` bez TLS. Server to nemůže ověřit, takže při zapnutí ukazuje červené varování. Ukládá se jako `mcp.behind_proxy`. Viz [MCP a agenti](mcp-and-agents.md#přístup-k-endpointu-z-jiného-počítače). |

### Vzhled

![Stránka Vzhled: motiv, avatary, barva vstupů, barevná hloubka a barvy grafu (formáty data následují níže)](../images/settings-ui.png)

![Rozbalený seznam Motiv na stránce Vzhled se zvýrazněným motivem Light: celá obrazovka Nastavení se už v tomto motivu překreslila](../images/settings-appearance-theme.png)

| Volba | Co dělá |
|---|---|
| **Motiv** | **Systém (podle terminálu)**, vestavěný motiv nebo některý z vašich vlastních souborů motivů. Zvýrazněný motiv se ukazuje živě: `Enter` ho ponechá, `Esc` obnoví předchozí. Pás **Barvy** pod řádkem ukazuje aktuálně platné barvy. Viz [Motivy](themes.md). |
| **Avatary autorů** | **vypnuto** (výchozí, bez sítě), **graf** (v tečce commitu) nebo **autor** (ve sloupci Autor). Když je zapnuto, hash e-mailu každého autora se odesílá službě Gravatar a vašemu poskytovateli hostingu. |
| **Zdroje avatarů**, **Náhradní avatar** | Kde se obrázky hledají a v jakém pořadí. Pro autory bez obrázku: **identicon** (vygenerovaný vzor) nebo **iniciály**. |
| **Barva vstupů**, **Vlastní barva vstupů** | Pozadí textových polí: **Podle motivu** (výchozí), pevná tmavá barva, nebo vaše vlastní `#rrggbb`. |
| **Textový kurzor** | `ui.text_cursor`: **terminal** (výchozí) umístí vlastní kurzor terminálu na místo vkládání v aktivním textovém poli, jako svislou čárku (platí nastavení kurzoru vašeho terminálu a okno s návrhy metody zadávání se otevře u kurzoru); **drawn** vykreslí vlastní kurzor GitPoulpi a kurzor terminálu nechá skrytý. Platí okamžitě. Paleta: `Nastavení: Textový kurzor...`. |
| **Barevná hloubka** | **auto** (výchozí), **truecolor**, **256** nebo **16**. Viz [Motivy](themes.md). |
| **Barevný motiv grafu** | **Podle motivu** (výchozí: vlastní barvy drah motivu) nebo libovolná paleta drah. Každá paleta ukazuje několik svých barev. Použije se po zavření Nastavení. Příkaz z palety **Aplikace: Změnit barevný motiv grafu...** (`Ctrl+N Shift+T`) ukazuje každou paletu živě. |
| **Pozadí drah grafu** | **Vypnuto** (průhledné) nebo **Podbarvení řádku**: slabý pruh barvy dráhy za sloupcem grafu s tenkou lištou na okraji. |
| **Síla podbarvení řádku** | Jak vybledlý je pruh podbarvení: **Silné**, **Střední** (výchozí) nebo **Jemné**. Okrajová lišta si ponechá plnou barvu. |
| **Umístění panelu nástrojů** | `ui.toolbar_position`: Kde je panel nástrojů: Nahoře, Dole (výchozí), Vlevo nebo Vpravo. Vlevo a Vpravo z něj udělají sloupec vedle panelů; v okně užším než 80 sloupců se kreslí dole. Projeví se hned. |
| **Pamatovat si pozici a velikost dialogů** | `ui.dialog_geometry`: **Pro tuto relaci** (výchozí), **Uložit na disk** nebo **Nikdy**: zda dialog, který jste přesunuli nebo jemuž jste změnili velikost, drží své místo. Jedno nastavení pro všechny dialogy; každý dialog si pamatuje své vlastní místo a velikost. Projeví se hned. |
| **Zarovnání tlačítek panelu nástrojů** | `ui.toolbar_align`: Kde leží tlačítka podél panelu: Začátek (výchozí), Střed nebo Konec. Začátek je levá strana řádku a horní část sloupce; konec je pravá strana a spodek. Projeví se hned. |
| **Připnout tlačítko pull request na konec** | `ui.toolbar_pr_pinned`: Zapnuto (výchozí): tlačítko pull request leží na samém konci panelu nástrojů (vpravo, ve sloupci dole) při jakémkoli zarovnání. Vypnuto: plyne spolu s ostatními tlačítky a zarovnává se s nimi. Projeví se hned. |
| **Umístění stavového řádku** | `ui.statusbar_position`: Kde je stavový řádek (větev, počty, zprávy, nápověda kláves): Nahoře, nad kartami repozitářů, nebo Dole (výchozí). Karta Domů si ponechává vlastní stavový řádek dole. Projeví se hned. |
| **Umístění lišty karet** | `ui.tabs_position`: Kde jsou karty repozitářů: Nahoře (výchozí), Dole nebo Vlevo, seznam s jednou kartou na řádek u levého okraje okna. V okně užším než 80 sloupců se kreslí nahoře. Vedle panelu nástrojů jako sloupce zůstávají karty nejvíc vně. Projeví se hned. |
| **Národní prostředí data/času** | **systémový** (výchozí): názvy měsíců a dnů v jazyce rozhraní. **english**: vždy anglické názvy. |
| **Formát data**, **Formát data a času**, **Podrobný formát data** | Vzory data pro sloupec s datem v Historii souboru a Blame, pro sloupec Datum v Logu a pro podrobnosti commitu. Každý ukazuje živý náhled. Tokeny viz [Referenční příručka konfigurace](config-reference.md#ui). |

![Rozbalený seznam Barevný motiv grafu: Podle motivu, poté každá paleta drah s několika svými barvami](../images/settings-log-graph-lanes.png)

### Klávesnice a myš

![Stránka Klávesnice a myš: každá akce podle kontextu s jejími klávesami, vyhledávací pole a filtry](../images/settings-keyboard.png)

**Nastavení > Klávesnice a myš** má dvě karty (klikněte, nebo `Alt+Left` / `Alt+Right`).
**Obecné** obsahuje klávesnicový protokol a nastavení myši; **Klávesové zkratky**
uvádí každou zkratku podle kontextu s vyhledávacím polem (`/`) a filtry. Na řádku:

| Klávesa | Akce |
|---|---|
| `Enter` nebo dvojklik | Zachytí novou klávesu. Konflikty a upozornění terminálu se zobrazují během stisku. |
| `a` | Přidá alternativní klávesu. |
| `u` | Zruší vazbu akce. |
| `r` | Vrátí řádek na výchozí hodnotu. |
| `R` | Vrátí všechny zkratky na výchozí hodnoty po potvrzení. |
| `c` nebo klik na `!` řádku | Zobrazí akci se všemi akcemi, se kterými její klávesa koliduje; `Esc` znovu zobrazí vše. Klávesa kolidující s jinou akcí je červená, varování jen ke klávese (terminál ji neumí poslat) oranžové. |

Použijte ji k vyřešení kolizí s tmux, AutoHotkey, herdr nebo vaším terminálem.
Stejná nastavení jsou v tabulce `[keybindings]` souboru `config.toml`; viz
[referenční příručka konfigurace](config-reference.md#keybindings) a
[Klávesové zkratky](keybindings.md).

![Stránka Klávesnice a myš po stisku c na řádku Quit: klávesa q je červená u Quit i u Stage or Unstage the Selected Row, dvou akcí, které se střetávají](../images/settings-keyboard-conflicts.png)

Tři volby jsou na kartě **Obecné**.

| Volba | Co dělá |
|---|---|
| **Klávesnicový protokol Kitty** | **auto** požádá terminál o klávesnicový protokol kitty, takže dorazí `Ctrl+,`, `Ctrl+Enter` a `Ctrl+Shift+N`. **off** nepožádá nikdy; použijte, pokud se terminál chová nesprávně. Vyžaduje restart. |
| **Myš zapnuta** | Vypnuto ponechá myš a výběr textu vašemu terminálu. Vyžaduje restart. |
| **Řádků na kolečko myši** | O kolik řádků se v Logu posune jeden krok kolečka. Vyžaduje restart. |

### Log a graf

Jak se vykresluje graf commitů a kolik commitů Log načítá. Barvy grafu jsou na
stránce **Vzhled**.

| Volba | Co dělá |
|---|---|
| **Vykreslování grafu** | **auto** kreslí obrázek tam, kde to terminál umí, jinak znaky. **obrázek** vynutí obrázek, **text** vynutí znaky. Vyžaduje restart. |
| **Protokol obrázků** | Jak se obrázky dostanou do terminálu: **auto**, **kitty**, **iterm2**, **sixel** nebo **none**. Vyžaduje restart. |
| **Zobrazit stashe v grafu** | Řádek pro každý stash v Logu. Vypnuto: stashe se zobrazují jen v postranním panelu. |
| **Zobrazit zástupnou větev u najetého commitu** | Ve výchozím stavu zapnuto (`ui.graph_ghost_ref`). Ve sloupci Ref najetý commit, který nemá vlastní ref, ukáže vybledlou pilulku s názvem větve, do které patří (jinak vzdálené větve), odvozeným z historie, je-li ve sloupci místo. Větev smazaná po sloučení se přesto zobrazí přeškrtnutě pod názvem z předmětu slučovacího commitu (`Merge branch 'x'`, `Merge pull request #N from owner/x`, ...); squash merge, rebase a fast-forward takovou stopu nezanechají. Tagy se jako zástupné nezobrazují nikdy. Najetí na pilulku ukáže její druh. Kliknutí nedělá nic zvláštního; větev vytvoří příkaz palety `Git: Vytvořit větev ve vybraném commitu...`. |
| **Spojnice ref–commit** | Tenká čára od popisku refu k tečce jeho commitu. |
| **Rozložení sloupců** | `ui.log_columns_layout`, platí hned pro Log. **Přizpůsobit panelu** (výchozí): každý sloupec se do panelu vejde přesně a pružná je jen Zpráva. **Posouvat do stran**: sloupce si drží šířky, i Zpráva, a tabulka se posouvá do stran, když je širší než panel (`Shift+Left` / `Shift+Right`, vodorovné kolečko, lišta na spodním okraji). Stejný přepínač je ve výběru sloupců Logu a v paletě. |
| **Změna velikosti sloupců** | `ui.column_resize`, platí hned pro Log, Radar a sloupec commitů zobrazení diff, Soubor a blame. **line** (výchozí): každá hranice sloupce je jednobuňková mezera s malou značkou v záhlaví, dokud je ukazatel na hranici, kreslí se přes celou tabulku tenká akcentní čára a tažení kdekoli podél ní mění šířku. **header**: bez značky, rozsvítí se první znak buňky záhlaví za hranicí a jen odtud se dá táhnout. Klávesy pro změnu šířky (`Shift+C` v Logu, `C` v Radaru, `Tab` ve sloupci commitů) dělají v obou režimech totéž. |
| **Zobrazit počty změn WIP ve zprávě, když je sloupec Změny skrytý** | Zobrazí počty změn (`✎ 3  + 2  − 1`) ve zprávě řádku WIP. Když je sloupec Změny zobrazen, počty jsou v něm. |
| **Strom složek větví** | V postranním panelu seskupí větve a tagy podle `/` do složek. Přepíná se klávesou `y` v panelu Refs. |
| **Sekce levého panelu** | Čí volba určuje, které sekce levého panelu jsou zobrazené a skryté: **podle repozitáře** (výchozí; každý repozitář má vlastní sadu, globální je výchozí bod) nebo **globální** (jedna sada pro všechny repozitáře). Přepínání tam a zpět zachová obě sady. Viz [myš a gesta](mouse-and-gestures.md). (`ui.sidebar_sections_scope`) |
| **Kliknutí na ref ho ukáže v grafu** | Jedno kliknutí na ref v panelu Refs posune Log tam, kam ref ukazuje, a řádek krátce zvýrazní (výběr commitu a pravý panel zůstanou, jak jsou). Vypnuto: kliknutí jen vybere řádek postranního panelu. **Ukázat v grafu** funguje v obou případech. |
| **Návrhy Composeru** | Zda se AI composer commitů otevře se soubory každého návrhu **rozbalenými** nebo **sbalenými**. |
| **Počáteční commity v grafu** | Kolik commitů Log načte při otevření repozitáře. Výchozí 5000, minimum 500. Vyšší hodnoty zpomalují velké repozitáře. |
| **Líné načítání commitů v grafu** | Zapnuto: při posunu k nejstaršímu načtenému commitu se na pozadí načtou další. Vypnuto: Log končí u počátečních commitů. |
| **Commitů na líné načtení** | Kolik commitů načte jedno načtení na pozadí. |
| **Maximum commitů v grafu** | Líné načítání se u tohoto počtu zastaví. `0` znamená bez omezení. Každý commit zabere přibližně 300 bajtů paměti. |
| **Celkem commitů zjišťovat pomocí** | Jak stavový řádek zjistí celkový počet v údaji `načteno / celkem` (`log.commit_count`, rozbalovací seznam, platí okamžitě): **Systémový git** spustí jeden proces `git rev-list --count` (výchozí, obvykle nejrychlejší); **libgit2** počítá uvnitř GitPoulpi a nespouští žádný proces, pro počítače, kde antivirový program kontroluje každý příkaz a každé volání gitu zpomaluje; **Vypnuto** nepočítá nikdy a ukazuje jen načtené commity. |
| **Zobrazovat autory podle .mailmap** | Log, blame i historie souboru ukazují každého autora tak, jak ho pojmenovává `.mailmap` repozitáře (`log.use_mailmap`, přepínač, ve výchozím stavu zapnuto, platí okamžitě). Uložené commity se nikdy nemění. Vypnuto ukáže jména tak, jak byla commitnuta. Viz `history-rewrite.md`, *Karta Identity*. |

### Diff a editor

![Stránka Diff a editor](../images/settings-editor.png)

| Volba | Co dělá |
|---|---|
| **Velikost tabulátoru** | Sloupců na jednu zarážku tabulátoru, 1 až 8 (výchozí 4). |
| **Konec řádku** | Co editor zapíše při uložení: **Auto (podle souboru)**, **LF** nebo **CRLF**. |
| **Zvýrazňování syntaxe** | Barvy jazyka v editoru. |
| **Čísla řádků** | Sloupec s čísly řádků v editoru a v diffu. |
| **Sledovat výběr v editoru** | Dokud je editor otevřený, výběr jiného souboru v pravém panelu jej otevře i v editoru, a to po dotazu, zda uložit neuložené změny. Soubor, který nelze upravovat, se otevře v obvyklém zobrazení. Jen pro pracovní změny: soubor vybraný z commitu opustí editor a otevře se v diffu toho commitu. Ve výchozím stavu zapnuto (`editor.follow_selection`). |
| **Zalamování řádků** | Zalamuje dlouhé řádky diffu. |
| **Výchozí rozvržení**, **Jen hunky**, **Zobrazit změny jen v bílých znacích**, **Minimapa** | Počáteční stav zobrazení diffu. Lišta zobrazení diffu je mění a ukládá. |
| **Anotace blame**, **Šířka anotací blame** | Na které straně pohledu Blame je sloupec anotací (avatar, autor, stáří, shrnutí): **right** (výchozí) nebo **left**, a jeho šířka v buňkách (24 až 90). Tažení za okraj sloupce v pohledu Blame nebo klávesy `>` / `<` uloží šířku sem (`ui.blame_side`, `ui.blame_width`). Viz [Blame](log-and-history.md#blame). |
| **Editor merge: vedle sebe od (sloupců)** | Od této šířky zobrazí editor merge A a B vedle sebe. Výchozí 100, minimum 40, `0` znamená vždy. Pod touto šířkou jsou A a B karty. Viz [Řešení konfliktů merge](merge-conflicts.md). |
| **Externí editor** | Příkaz pro položku souboru *Otevřít v externím editoru*. `{file}` je cesta. Prázdné: VS Code, je-li nainstalován, jinak výchozí program systému. |
| **Externí nástroj diff** | Příkaz pro *Otevřít v externím nástroji diff*. `{local}` a `{remote}` jsou obě verze. Prázdné: `git difftool`. |
| **Externí nástroj merge** | Příkaz pro *Otevřít v externím nástroji merge* na kartě konfliktu. `{base}`, `{local}`, `{remote}` a `{merged}` jsou soubory. Prázdné: `git mergetool`. |
| **Správce souborů** | Příkaz pro *Zobrazit ve složce*. `{path}` je soubor, `{dir}` jeho složka. |

### Otevřít v

![Stránka Otevřít v, karta Položky: seznam položek, upravovaná položka s popiskem a příkazem a tlačítko + Přidat](../images/settings-open-in.png)

Upravuje seznam tlačítka **Otevřít v** v záhlaví. Karta **Obecné** obsahuje
tlačítka složky **Konec a cd sem**, **Výchozí položku** a **Obnovit zjištěné výchozí
hodnoty**. Karta **Položky** je [seznam](#seznamy) (každá položka má popisek a
příkaz): **+ Přidat ▾** nabízí předvolby a **Vlastní** a výchozí položka je
označena `★`. Pole vybrané položky zahrnují **Může otevřít worktree** (ve výchozím
stavu zaškrtnuto: položka se nabízí v nabídce Otevřít v u řádku worktree, viz
[Worktrees](worktrees.md)) a tlačítko **Vyzkoušet**. Viz [Otevřít v](open-in.md).

První řádek, **Sledovat složku aktivní karty**, má dvě hodnoty:

- **tlačítka** (výchozí): řada karet zobrazuje **Konec a cd sem**, když aktivní
  karta není složkou, ve které byl GitPoulpi spuštěn.
- **auto**: žádné tlačítko. Každé ukončení přepne složku vašeho shellu na
  aktivní kartu a otevření nové karty spustí výchozí položku Otevřít v.

Viz [Sledování složky aktivní karty](following-the-tab-folder.md).

### Obrázky

Jak se zobrazují soubory s obrázky a který program používá **Upravit v**. Viz
[Obrázky](image-view.md). Karta **Obecné** obsahuje níže uvedené volby prohlížeče,
**Výchozí položku** a **Obnovit zjištěné výchozí hodnoty**; karta **Editory** je
seznam editorů obrázků. Sestavení bez náhledu obrázků nemá volby prohlížeče.

| Volba | Co dělá |
|---|---|
| **Zobrazovat náhled obrázků** | Zobrazí soubor s obrázkem jako obrázek v zobrazení Diff a v Zobrazení souboru. Vypnuto: prosté upozornění na binární soubor. Ve výchozím stavu zapnuto. |
| **Bez protokolu obrázků** | V terminálu bez protokolu pro vložené obrázky: **Barvy půlbloků** (výchozí) nebo **Jen informační karta**. |
| **Max. velikost náhledu** | Větší obrázky se nedekódují; místo nich se zobrazí informační karta. Číslo a MiB nebo GiB, výchozí 32 MiB, nejvýše 1 GiB (`ui.image_view_max_mib`, `ui.image_view_max_unit`). |
| **Pixelové zvětšení od** | `1x`, `2x` (výchozí) nebo `never`: od tohoto přiblížení obrázek ukazuje čtvercové pixely místo vyhlazeného obrázku. |
| **Synchronizovat přiblížení a posun před/po** | Oba panely obrazového diffu sdílejí jedno přiblížení a jednu polohu posunu. Ve výchozím stavu zapnuto. |

Karta **Editory** je [seznam](#seznamy) jako [Otevřít v](open-in.md):
u každého editoru **Popisek**, **Příkaz**, čipy proměnných
`${file}` `${dir}` `${name}` `${repo}` a **Vyzkoušet na ukázkovém obrázku**;
**+ Přidat ▾** nabízí předvolby a **Vlastní** a **Obnovit zjištěné výchozí hodnoty**
seznam vrátí. Dokud ho neupravíte, obsahuje seznam
editory nainstalované v tomto počítači a výchozí aplikaci systému. Každá změna se
projeví okamžitě.

### Prevence konfliktů

Globální část proaktivní detekce konfliktů (`gitpoulpi ui show settings conflict-prevention-global`); vlastní stránka repozitáře je
[Nastavení repozitáře: Prevence konfliktů](#nastavení-repozitáře-prevence-konfliktů). Příkaz z palety **Konflikty: Upravit seznamy základních větví...** otevře tuto stránku.

| Volba | Co dělá |
|---|---|
| **Zapnout proaktivní detekci konfliktů** | Ve výchozím stavu zapnuto (`general.conflict_detection`). Upozorní v Logu, když by merge aktuální větve do větve, ze které vznikla, skončil konfliktem, ještě než to zkusíte. Repozitář ji může pro sebe vypnout na své vlastní stránce **Prevence konfliktů**. Viz [Řešení konfliktů při slučování](merge-conflicts.md#prevence-konfliktů). Projeví se po zavření Nastavení. |
| **Výchozí základní větve** | Pod přepínačem. Seznam vzorů základních větví, který repozitáře používají, když nemají vlastní: **Vestavěný (`**`)**, tedy každá větev, nebo jeden z vašich pojmenovaných seznamů (`conflict_prevention.default_list`). Platí pro otevřené repozitáře po zavření Nastavení. **Obnovit vestavěný výchozí** vrátí `**`. |
| **Seznamy základních větví** | Také na této stránce: vaše pojmenované seznamy vzorů základních větví (`[conflict_prevention.lists.<name>]`), každý jako blok s **Názvem** (přejmenováním seznam zůstane výchozím, pokud jím byl; repozitář, který sledoval starý název, používá výchozí seznam, dokud znovu nevyberete přejmenovaný seznam), **Vzory** (jeden na řádek, `Enter` nebo kliknutí otevře editor, `Ctrl+S` ho ponechá), **Nastavit jako výchozí** a **Smazat seznam** (nejprve se zeptá a řekne, že repozitáře, které ho sledují, budou místo něj používat výchozí seznam). Zadejte název do **Nový seznam** a stiskněte `Enter`; seznam začíná vzorem `**`. Úprava seznamu změní každý repozitář, který ho sleduje. |

### Nastavení repozitáře: Git a kódování

![Stránka Git a kódování pro tento repozitář](../images/settings-git.png)

| Volba | Co dělá |
|---|---|
| **Backend gitu** | `git2` (libgit2, výchozí) nebo `system` (váš nainstalovaný `git`). Globální. Platí pro daemony spuštěné po změně. |
| **Tlačítko Pull** | Co dělá tlačítko Pull: merge (fast-forward, je-li to možné), pouze fast-forward, nebo rebase. Globální. Položky `▾` ho nemění. |
| **Tlačítko Push** | **přímo** pushuje hned; **zeptat se** nejprve zobrazí nabídku remote. Globální. |
| **Zkontrolovat všechny remote před push** | Ve výchozím stavu zapnuto (`git.push_group_preflight`). Před pushem větve, která má další cíle push, se nejdřív podívá na všechny remote (rychlý dotaz a stažení z toho remote jen tehdy, když se liší), takže problém na třetím remote se ukáže dřív, než začne první push. Vypnuto: pushne na jednotlivé remote postupně a hlásí, co se stalo. Globální. |
| **Merge fast-forward** | Co udělá merge, který spustíte, když lze větev jednoduše posunout fast-forwardem: **never** (výchozí) vždy vytvoří merge commit, **allowed** provede fast-forward, když lze (výchozí chování gitu), **only** provede fast-forward, nebo odmítne. Globální (`git.merge_fast_forward`); repozitář to přepíše na své stránce [Možnosti merge](#nastavení-repozitáře-možnosti-merge). |
| **Výchozí kódování** | Kódování použité k zobrazení textu, který **není platné UTF-8**: obsah souborů, blame, diffy a zprávy commitů. Stránka má globální řádek a řádek pro tento repozitář; řádek repozitáře má přednost. Platné UTF-8 se nikdy nedekóduje znovu a staging nic nepřekóduje. |
| **core.autocrlf**, **core.eol** | Vlastní git nastavení tohoto repozitáře. Řádek zapíše `core.autocrlf` nebo `core.eol` do git konfigurace repozitáře. **(zděděno)** odstraní hodnotu repozitáře, takže platí vaše globální git konfigurace. `Ctrl+Z` obnoví předchozí hodnotu. Ekvivalent v CLI je `gitpoulpi repo-config`. |
| **Další cíle push pro nové větve** | Jen tento repozitář: pod tímto nadpisem jedno zaškrtávací políčko pro každý remote repozitáře. Zaškrtnutý remote se stane dalším cílem push každé nové větve, kromě jejího upstreamu, ve chvíli, kdy větev poprvé dostane upstream (první push, který ho nastaví, nebo větev vytvořená ze vzdálené větve). Volba se zkopíruje do vlastních klíčů `gitpoulpi-push-to` větve, takže už existující větve se nemění: upravte je pomocí odznaku **Remote** v jejich řádku v Refs (nebo `u` na řádku). Ukládá se do dat repozitáře (`default_push_targets` v `repo-state.toml`). |
| **Git LFS** | Zobrazí Git LFS v rozhraní. Viz [Git LFS](lfs.md). |
| **Instalovat filtry LFS před pull a push** | Před pull nebo push spustí `git lfs install --local`, pokud repozitář používá LFS a jeho filtry chybí. |
| **Program git-lfs** | Zjištěný program `git-lfs` (pouze ke čtení). |
| **Inicializovat LFS pro tento repozitář** | Spustí `git lfs install --local` hned. |

### Nastavení repozitáře: Částečný checkout

Stejné ovládací prvky jako dialog Částečný checkout (`Repozitář: Částečný checkout...`), jako stránka nastavení tohoto
repozitáře (`gitpoulpi ui show settings sparse-checkout`).

| Volba | Co dělá |
|---|---|
| **Stav** | Zda je tento pracovní strom částečným checkoutem. |
| **Adresáře k ponechání** | Jeden adresář na řádek, relativně ke kořeni repozitáře. `Enter` (nebo kliknutí) otevře víceřádkový editor: `Ctrl+S` text ponechá, `Esc` ho zahodí. Dokud je částečný checkout zapnutý, pole ukazuje aktuální adresáře. Režim cone v gitu zná jen adresáře (žádné vzory ani jednotlivé soubory), soubory v kořeni se zapisují vždy a prázdný seznam se odmítne. |
| **Zahájit částečný checkout** | Ponechá jen uvedené adresáře. Jakmile je částečný checkout zapnutý, totéž tlačítko se jmenuje **Použít seznam adresářů** a použije upravený seznam: adresář, který ze seznamu zmizel, se z pracovního adresáře odstraní. Zašedlé, dokud je seznam prázdný. |
| **Ukončit částečný checkout** | Zpět k úplnému checkoutu: všechny soubory se vrátí (po potvrzení). Zašedlé, dokud je částečný checkout vypnutý. |
| **Obnovit částečný checkout** | Znovu použije aktuální seznam (například když skript nebo jiný nástroj zapsal soubory mimo něj). Pošle aktuální adresáře, ne neodeslaný text pole. Zašedlé, dokud je částečný checkout vypnutý. |

Tlačítka běží jako běžné akce na pozadí (stránka při běhu ukazuje „používá se částečný checkout...“). Stejně jako dialog nejsou
na zásobníku Zpět: nic se neztratí a **Ukončit** vrátí všechny soubory.

### Nastavení repozitáře: LFS

Stav LFS tohoto repozitáře (`gitpoulpi ui show settings lfs`; není v sestavení bez LFS).

- **Neinicializováno:** „Git LFS není v `<repozitář>` zprovozněno“ a tlačítko **Zprovoznit LFS zde**,
  které spustí `git lfs install --local` (filtry a hooky tohoto repozitáře, nikdy vaše globální konfigurace; vyžaduje
  `git-lfs`). Když `.gitattributes` repozitáře už vede soubory přes LFS, ale chybí jeho hooky, stránka to řekne.
- **Inicializováno:** „Git LFS je v `<repozitář>` zprovozněno“ a vzory sledované v `.gitattributes`
  (`filter=lfs`).

LFS se počítá jako **inicializované**, když má repozitář vlastní hooky LFS (hook `pre-push` nebo `post-checkout`, který
zapíše `git lfs install --local`). Samotné filtry se nepočítají: `git lfs install` na úrovni uživatele je nastaví pro každý
repozitář, takže by každý vypadal jako inicializovaný.

Tlačítko **`LFS ▾`** na panelu nástrojů se řídí stejným stavem: zobrazí se jen v repozitáři, kde je LFS inicializované
nebo kde ho `.gitattributes` už používá, a v běžném repozitáři zůstane skryté (a dokud nedoběhlo první čtení LFS
repozitáře, takže se může objevit chvíli po jeho otevření). Inicializace z této stránky ho zobrazí. Příkazy palety `LFS:`
zůstávají dostupné všude, včetně **LFS: Inicializovat pro tento repozitář**.

### Nastavení repozitáře: SSH a přihlášení

`gitpoulpi ui show settings ssh-credentials`. Jak se přihlašuje *tento repozitář*; volba se uchovává v soukromé složce
GitPoulpi: nic se nezapisuje do repozitáře ani do konfigurace gitu a nelze ji vrátit zpět (jako každé nastavení).
Používají ji jen fetch, pull, push, submoduly a LFS tohoto repozitáře. Jejím dvojčetem v CLI je `gitpoulpi repo-auth`
(viz [Reference CLI](cli-reference.md)). Stránka je dostupná i z palety příkazů:
**Repozitář: SSH a přihlášení...**. Nic se neodešle, dokud nestisknete **Uložit**; stránka se ptá daemona, takže nikdy
neblokuje.

| Řádek | Co dělá |
|---|---|
| **HTTPS: Přihlásit pomocí** | **Systém** (vlastní pomocníci gitu, výchozí), **Účet GitPoulpi** (token integrace; účty, jejichž hostitel je jedním z remote tohoto repozitáře, jsou první, a řádek pod tím říká, co našla poslední kontrola tokenu: chybějící a nadbytečná oprávnění), **Credential helper** nebo **Příkaz pro token**. |
| **Uživatelské jméno gitu** | Odesílá se jako uživatelské jméno pro remote přes HTTPS. Se **Systémem** umožní správci přihlašovacích údajů vybrat jeden z více uložených účtů. |
| **Ignorovat credential helpery nastavené tímto repozitářem** | Jen **Systém**. Pomocníci, které si repozitář nastavuje sám (jeho `.git/config`, soubory, které zahrnuje), jsou pod tím vypsáni se souborem, z něhož pocházejí; systémoví a globální pomocníci dál fungují. |
| **Pomocník**, **Příkaz**, **Hostitelé**, **Mezipaměť (minuty)** | Příkaz, který repozitář spouští k přihlášení. *Helper* je credential helper gitu; *příkaz pro token* vytiskne token (seznam argumentů se dělí podle mezer, argument s mezerami patří do uvozovek, a zobrazí se pod **Spustí se jako**; žádný shell ho nespouští, běží ve složce repozitáře). **Hostitelé** ho omezí na tyto hostitele (prázdné: hostitel hlavního remote); **Mezipaměť** říká, jak dlouho se token znovu používá (výchozí 50 minut). **Použít současného pomocníka tohoto repozitáře** zkopíruje do pole pomocníka, kterého si repozitář nastavuje. |
| **Použít pro API hostitele (issues, pull requesty)** | S příkazem tokenu: tentýž příkaz dává i token pro volání issues a pull requestů tohoto repozitáře místo vašeho osobního tokenu (instalační token GitHub App, takže komentáře vystupují jako App). Zapnutí i vypnutí vyžaduje potvrzení. Viz „Vystupování jako bot GitHub App“ v průvodci [integracemi s hostiteli](hosting-integration.md). |
| **Přiřazování** | **Povoleno** (výchozí) nebo **Zablokováno**, uloží se hned. Zablokováno: v issues a pull requestech tohoto repozitáře nelze nikoho přiřadit z aplikace, z CLI ani agentem; stávající řešitelé zůstávají vidět. |
| **SSH: Přihlásit pomocí** | **Systém** (vlastní nastavení ssh, agent a výchozí klíče), **Soubor klíče** nebo **Klíč účtu** (klíč integrace). |
| **Soubor soukromého klíče** | Vyberte jeden z klíčů nalezených v `~/.ssh` nebo zadejte cestu. Nabízí se jen tento klíč, každému remote SSH repozitáře, včetně submodulů. Když soubor klíče mohou číst jiné účty (bit oprávnění pro skupinu nebo ostatní v Linuxu a macOS, záznam pro Everyone, Users nebo Authenticated Users ve Windows), varování řekne, že OpenSSH takový klíč odmítne („bad permissions“), a **Omezit na můj účet** to opraví (`chmod 600`, nebo ve Windows DACL jen pro váš účet a SYSTEM). |
| **Soubor známých hostitelů**, **Přísná kontrola klíče hostitele** | Volitelný soubor důvěryhodných klíčů hostitelů a odmítnutí hostitele, který v něm není. |
| **Program ssh** | Program, který se spustí (jen pro čtení). |
| **Připnuto v konfiguraci gitu** | Jen když si tento repozitář nastavuje vlastní `core.sshCommand` (staré připnutí „Klíč SSH pro tento repozitář“, které GitPoulpi dříve zapisoval). Příkaz se zobrazí jen pro čtení. **Přesunout do GitPoulpi** uloží rovnocennou volbu **Soubor klíče** a pak odstraní připnutí z konfigurace gitu; nabízí se, když je příkaz obyčejné připnutí `ssh -i <klíč>`, a jen dokud stránka nemá neuložené úpravy a její volba SSH je stále **Systém**. Jiný příkaz se zobrazí s poznámkou: upravte ho pomocí `git config`. Přesun nelze vrátit zpět, jako každé nastavení tady, a odstranění připnutí také není na zásobníku Zpět (nic se neztratí: klíč je v profilu). |
| **Remote** | Jen pro čtení: každý remote s protokolem a identitou, kterou použije. Nastavení, která profil přepisuje (`core.sshCommand`, `credential.helper`), jsou vypsána se souborem a profil, který nelze použít, řekne proč. |

Tlačítka (každé má klávesu: `Tab`/`Left`/`Right` mezi nimi, `Enter` stiskne):

- **Otestovat připojení** zkontroluje každý remote s uloženými volbami, nejdřív fetch a pak push (žádná data se
  nepřenášejí), a otevře panel s výsledky: u každého remote v pořádku nebo selhalo, identita, která odpověděla (klíč,
  který server přijal, nebo účet, jehož token byl přijat), doba trvání a srozumitelné vysvětlení selhání. Běží na
  pozadí, po chvíli se spinnerem. Panel se posouvá pomocí `Up`/`Down`, `PgUp`/`PgDn`, `Home`/`End`, kolečka a posuvníku
  a zavře se klávesou `Esc` nebo křížkem. Dvojčetem v paletě je **Repozitář: Otestovat připojení**.
- **Uložit** odešle volby. **Vrátit zpět** zahodí neuložené. Zavření Nastavení s neuloženými volbami se nejdřív zeptá.
- **Výchozí nastavení** se zeptá a pak zapomene profil: pomocníci gitu pro HTTPS a vlastní nastavení ssh pro SSH.
  Dvojčetem v paletě je **Repozitář: Výchozí přihlašovací údaje**, které se také zeptá.

Uložení **Credential helperu** nebo **Příkazu pro token** se nejdřív zeptá **Povolit tento příkaz?**: příkaz se ukáže
slovo od slova (řídicí znaky jsou vypsány, dlouhé řádky se zalomí), s jeho SHA-256 a dvěma odpověďmi, `1` **Uložit a
povolit** a `2` **Zrušit**. Žádná odpověď není předvybraná, dokud nestisknete šipku nebo číslici, takže náhodné `Enter`
nic neudělá; `Esc`, **Zrušit** i křížek panelu zrušují. `PgUp`/`PgDn`/`Home`/`End` a posuvník posouvají dlouhý příkaz.
Je-li příkaz už uložen a beze změny, uložení druhé poloviny stránky se znovu nezeptá.

Profil, který nelze použít (chybějící soubor klíče, zdroj, který tato verze zatím nespouští), způsobí, že přenos selže
s uvedením důvodu, místo aby se vrátil k vlastní identitě počítače.

Když profil není celý výchozí, nabídka **Push** končí řádkem, který říká, jako kdo se push přihlásí (**Přihlášení
jako: github-corp (token)** nebo **Klíč SSH: work_ed25519**); otevře tuto stránku. Stejný řádek se přidá k záznamům
Protokolu aktivity pro fetch, pull a push. Selhání způsobené volbami této stránky (credential helper, který nešel
spustit, dosud neznámý hostitelský klíč SSH, odmítnuté přihlášení, příkaz pro token, který selhal, token, který se
neodeslal, protože server není adresou účtu) ukáže na svém záznamu v Protokolu aktivity tlačítko **SSH a přihlášení...**
(`r` na záznamu), vedle **Otevřít Integrace**, může-li být zdrojem účet integrace. Řádek **SSH a přihlášení...** v nabídce
karty stránku otevře také; nahrazuje starou nabídku připnutí **Klíč SSH...** i její položku v paletě.

### Nastavení repozitáře: Commit

Jak se chová commit vytvořený v tomto repozitáři (`gitpoulpi ui show settings commit`). Oba přepínače jsou ve výchozím
stavu **vypnuté** a ukládají se jen pro tento repozitář (`[preferences]` v
[`repo-state.toml`](config-reference.md#nastavení-pro-jednotlivé-repozitáře-gitpoulpirepo-statetoml)).

| Nastavení | Co dělá |
|---|---|
| **Push při každém commitu** | Po úspěšném commitu pošle přepnutou větev do jejího upstreamu jako obvyklý push (sledovaný přenos s textem „vytváření commitu, poté push...“). Větev bez upstreamu se commitne, ale neodešle, a oznámení to řekne: první push nové větve zůstává věcí tlačítka **Push**, které upstream nastaví. Odmítnutý push otevře obvyklou kartu odmítnutého pushe; commit zůstane. |
| **Ve výchozím stavu obcházet git hooky při každém commitu** | Zaškrtávací pole **Přeskočit hooky** ve formuláři commitu (vedle **Amend předchozího commitu**; `Shift+H` v panelu Stav nebo **Commit: Přepnout Přeskočit hooky** v paletě) začíná zaškrtnuté, takže hooky `pre-commit` a `commit-msg` neběží (`git commit --no-verify`). Zůstává to volbou pro každý commit: po commitu se vrátí na tento výchozí stav. |

Bez výchozí hodnoty běží hooky repozitáře při každém commitu, ať GitPoulpi používá jakýkoli backend (repozitář
s commit hooky se commituje přes systémový `git`, protože libgit2 žádné nespouští). Hook, který selže, ukáže svůj
výstup ve svém záznamu v Protokolu aktivity (stavový řádek řekne, že commit selhal) a nic se nevytvoří.

### Nastavení repozitáře: Možnosti merge

Dvě nastavení. **Fast-forward** říká, co merge udělá, když lze větev jednoduše posunout fast-forwardem:

| Volba | Merge, který spustíte... |
|---|---|
| **(inherited)** | sleduje globální nastavení **Merge fast-forward** na stránce Git (výchozí **never**). |
| **never** | vždy vytvoří merge commit (`git merge --no-ff`), i když je větev jen před cílem. První rodič commitu je starý konec větve, druhý sloučená větev a zpráva je od gitu (`Merge branch 'x' into y`). |
| **allowed** | provede fast-forward, když lze, jinak vytvoří merge commit (výchozí chování gitu). |
| **only** | provede fast-forward a odmítne s *cannot fast-forward*, když se větve rozešly (`git merge --ff-only`). |

Platí pro každý merge, který spustíte: nabídka větve, přetažení, `Merge` v paletě a `gitpoulpi merge` (jehož přepínače
`--ff`, `--no-ff` a `--ff-only` ho pro jeden merge přepíšou). Nabídky říkají, co merge udělá:
*Merge x do y (merge commit)*, *(fast-forward)* nebo *(jen fast-forward)*, a samostatná položka
*Fast-forward y na x* (a její příkaz v paletě) zůstává na dosah jednoho gesta při každé volbě. Merge commit je
běžný záznam Zpět, stejně jako fast-forward. Akce Flow volí vlastní strategii merge a toto nastavení ignorují.

Vaše vlastní volba má přednost před gitem: merge spuštěný GitPoulpi pod **never** nebo **only** předává přepínač
výslovně, takže `merge.ff` v git konfiguraci repozitáře se přepíše. Jen pod **allowed** platí `merge.ff` gitu dál, a když
ho repozitář nastavuje, stránka ho ukáže pod řádkem. (Pro tento řádek se čte jen vlastní `.git/config` repozitáře, ne vaše globální git konfigurace.)

**Squash** (ve výchozím stavu vypnuto). Je-li zapnutý, merge lokální větve (nabídka větve, přetažení,
`Merge` v paletě) přidá změny větve do indexu bez merge commitu (`git merge --squash`): commit je ponechán na vás.
Formulář commitu se otevře se zprávou squash od gitu (`SQUASH_MSG`, bez řádků komentářů), jednou na každý
squash. Potom nic neprobíhá (žádný `MERGE_HEAD`), takže **Přerušit** neplatí: pro návrat zahoďte změny
v indexu. Squash, který se zastaví na konfliktech, je nechá v panelu Stav jako každý merge v konfliktu.
Změny v indexu nejsou záznamem Zpět (žádný ref se nehnul, nemá se co obnovit); `gitpoulpi merge <branch> --squash`
v CLI dělá totéž bez ohledu na nastavení. Squash má přednost před **Fast-forward**.

### Nastavení repozitáře: Šablona zprávy

Zpráva, kterou formulář commitu začíná (`gitpoulpi ui show settings commit-template`).

- **Předvyplnit prázdný panel commitu šablonou**: *prázdný* formulář commitu se předvyplní šablonou, jednou na commit
  (ruční vymazání se respektuje do dalšího commitu).
- **Při vytváření commitu vynechat řádky komentářů**: při vytvoření commitu se ze zprávy odstraní řádky začínající `#` (nebo
  `core.commentChar` repozitáře), jako by to udělal `git commit --cleanup=strip`. GitPoulpi commituje s `-m`, které git
  nečistí, proto to dělá samo. S vypnutým přepínačem řádek `#` v commitu zůstane.
- **Souhrn** (popisek počítá zbývajících 72 sloupců a kolik jich je navíc) a víceřádkový **Popis**
  (`Enter` nebo kliknutí otevře editor, `Ctrl+S` ho ponechá).
- **Uložit šablonu** zapíše šablonu do `commit-template.txt` v datové složce repozitáře (viz
  `session.per_repo_state_dir`) a nastaví `commit.template` tohoto repozitáře na tento soubor, takže ji používá i
  `git commit` na příkazovém řádku a další nástroje. Pokud `commit.template` už ukazuje na jiný soubor, stránka ukáže
  cestu k němu, upraví *tento soubor* a nastavení nechá být.

### Nastavení repozitáře: Prevence konfliktů

`gitpoulpi ui show settings conflict-prevention`. Stránka proaktivní detekce konfliktů tohoto repozitáře
(viz [Řešení konfliktů při slučování](merge-conflicts.md#prevence-konfliktů)):

- **Zapnout proaktivní detekci konfliktů** (u cílové větve): ve výchozím stavu zapnuto. Zrušení zaškrtnutí vypne detekci
  jen pro tento repozitář; globální přepínač je **Zapnout proaktivní detekci konfliktů** na globální stránce **Prevence konfliktů**.
  Oba musí být zapnuté.
- **Základní větve z**: odkud se berou základní větve tohoto repozitáře. **Výchozí** sleduje globální výchozí
  (vestavěné `**`, nebo seznam zvolený pod **Výchozí základní větve** v Nastavení > Prevence konfliktů); **Seznam: <název>**
  sleduje jeden z vašich pojmenovaných seznamů, takže úprava seznamu změní i tento repozitář; **Vlastní seznam** je
  seznam jen tohoto repozitáře (jeho volba začne kopií vzorů, které právě platí). Soubor repozitáře zapsaný před
  existencí této volby a obsahující vlastní vzory se čte jako **Vlastní seznam**.
- **Základní větve** (jeden vzor na řádek): u **Vlastního seznamu** `Enter` nebo kliknutí otevře editor a `Ctrl+S` ho
  ponechá; u **Výchozí** nebo **Seznam** se platné vzory zobrazí jen ke čtení. Tabulka vzorů pod ním je přehledem:

  | Vzor | Odpovídá |
  |---|---|
  | `origin/main` | jen tato větev |
  | `origin/**` | každá větev vzdáleného repozitáře `origin` |
  | `origin/feature/**` | větve z `origin`, které začínají na `feature/` |
  | `!origin/abc` | tuto větev vylučuje |

  Lokální větev je její holý název (`main`), vzdálená sledovací větev má před sebou svůj remote (`origin/main`). `*` a `?`
  zůstávají v jednom segmentu cesty, `**` je překračuje. Prázdné řádky a komentáře `#` se ignorují; seznam jen
  s vyloučeními znamená „každá větev kromě těchto“. Volba se ukládá do `repo-state.toml` a znovu se čte po zavření
  Nastavení.
- **Uložit jako seznam...**: zadejte název a stiskněte `Enter`; platné vzory se stanou pojmenovaným seznamem (týmž,
  který upravuje Nastavení > Prevence konfliktů) a tento repozitář ho sleduje.
- **Obnovit výchozí** (nejprve se zeptá): zahodí volbu, seznam a vlastní vzory tohoto repozitáře; znovu sleduje výchozí.
- **Použít základní větve workflow** (zobrazí se, když je pro repozitář nastaven [větvicí workflow](branching-workflows.md)
  se základními větvemi): naplní **Vlastní seznam** těmito základními větvemi, každou jako lokální větev i jako větev
  libovolného remote (`main` a `*/main`). Zkopíruje je jednou; pozdější změny workflow nesleduje.

### Nastavení repozitáře: AI

![Stránka AI repozitáře: instrukce repozitáře s volbami Přidat ke globálním nebo Nahradit globální a profil a instrukce pro každou akci](../images/settings-repo-ai.png)

Profil AI tohoto repozitáře a instrukce pro každou akci a *Instrukce
repozitáře*, které platí pro každou akci v tomto repozitáři. Prázdná pole
používají globální nastavení AI. Viz [Asistence AI](ai.md#instrukce). Její blok *Kontrola kódu* opakuje
čtyři řádky kontextu kontroly: každá volba začíná na *Použít globální (hodnota)* s globální hodnotou a prázdný
**Další soubor** ukazuje globální šedě (a má stejné tlačítko **Procházet...**; výběr začíná v kořeni repozitáře a soubor uvnitř repozitáře se uloží relativně k němu); přepínač **Žádný další soubor** vypne globální další soubor pro tento repozitář, zatímco prázdné pole ho dál následuje; **Obnovit globální** (zobrazí se, jakmile má repozitář vlastní hodnoty)
vrátí všechny zpět. Ukládají se do `repo-state.toml` tohoto repozitáře.

### Nastavení repozitáře: Workflow

`gitpoulpi ui show settings workflow`. Branching workflow tohoto repozitáře (šablona, detekce větví, základní větev) má vlastního průvodce: [Branching workflows](branching-workflows.md#nastavení--workflow). Karta **Obecné** obsahuje, co není pravidlo (šablona, zjišťování, hooky, tok issue, kam se ukládá); karta **Pravidla** je [seznam](#seznamy) rozdělený do skupin (**Základní větev**, **Typ tématické větve**, **Pravidlo dokončení**); **+ Přidat ▾** vybírá druh a položky se přesouvají v rámci své skupiny. Na rozdíl od ostatních stránek drží koncept (lišta je na obou kartách): lišta připnutá dole na stránce ukazuje **● Neuloženo** (nebo **Beze změn**) s tlačítky **Uložit do sdíleného souboru**, **Jen pro mě** a **Zahodit změny** a zavření Nastavení s neuloženým konceptem se nejdřív zeptá.

## Soubory, proměnné prostředí a CLI

- Soubor s nastavením je `config.toml` ve složce konfigurace vašeho OS.
  `gitpoulpi config path` vypíše jeho umístění.
- Stav jednotlivých repozitářů je v `.gitpoulpi/repo-state.toml` uvnitř
  repozitáře.
- `gitpoulpi config get <key>` a `gitpoulpi config set <key> <value>` čtou a
  zapisují každé nastavení.
- Proměnné prostředí a přepínače příkazové řádky přebíjejí soubor.

Podrobnosti uvádí [referenční příručka konfigurace](config-reference.md).
