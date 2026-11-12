[English](../terminals.md) · **Čeština** · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Výběr terminálu

GitPoulpi běží v každém terminálu s podporou myši, ale nejlépe vypadá a funguje v
takovém, který kreslí obrázky, umí true color a posílá celou klávesnici. Tato stránka
říká, co který terminál v kterém OS nabízí a jaké nastavení změnit.

**Jak ji číst.** Tabulky vycházejí z testovacího průchodu v **listopadu 2026**: každý
terminál byl ovládán surovými escape sekvencemi a tam, kde ho mohl spustit, i samotným
GitPoulpi, v Linuxu, Windows 11 a macOS 15. Verze jsou uvedeny
[níže](#testované-verze). Každá buňka je jedna z možností:

- **funguje**;
- **částečně** (a co chybí);
- **nefunguje** (a proč: *terminál*, *vrstva OS* nebo *GitPoulpi*);
- **n/a** (terminál v tom OS neexistuje);
- **neměřeno** (a proč).

Multiplexery (tmux, zellij, screen, herdr, psmux, panely) mají vlastní stránku:
[Multiplexery](multiplexers.md).

## Nejlepší volby

| OS | Nejlepší volby v pořadí | Proč |
|---|---|---|
| **Windows** | **Windows Terminal**, poté **WezTerm** a **Rio** | Windows Terminal kreslí graf a avatary jako obrázky sixel, ukotvené správně, a je výchozí ve spouštěči. WezTerm a Rio je kreslí jako obrázky iTerm2, ukotvené správně od opravy ConPTY z listopadu 2026 (viz [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty a Ghostty dávají obrázky, tvary ukazatele i celou klávesnici. foot kreslí obrázky sixel a má celou klávesnici i tvary ukazatele. |
| **macOS** | **iTerm2**, poté **kitty** a **Ghostty** | iTerm2 3.7.7 kreslí obrázky přes protokol kitty. Terminal.app funguje, ale nemá obrázky ani true color. kitty a Ghostty se chovají jako v Linuxu (v macOS neměřeno). |

V libovolném OS **screen** a **zellij** skrývají před GitPoulpi protokol obrázků
terminálu, takže kreslí znaky; tmux sixel propouští. Viz
[Multiplexery](multiplexers.md).

## Co GitPoulpi používá

- **Obrázky** (sloupec grafu, avatary autorů, obrázkový diff a zobrazení Soubor):
  **grafický protokol kitty** s Unicode placeholdery, **vložené obrázky iTerm2**
  (`OSC 1337`), nebo **sixel**. Detekce je „zakázáno, dokud není prokázáno“, vychází z
  `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` a proměnných multiplexerů. Když prostředí nikoho
  nejmenuje (xterm spuštěný s `-ti vt340`, terminál VS Code se zapnutými obrázky, jakýkoli
  terminál dosažený přes SSH, kam se `TERM_PROGRAM` nepředává), rozhodují odpovědi
  samotného terminálu: GitPoulpi se při startu zeptá jednou (viz „Dotaz při startu“ níže)
  a použije jméno terminálu z XTVERSION (kitty, Ghostty, WezTerm, iTerm2, mintty, foot)
  nebo sixel v jeho atributech zařízení. Uvnitř multiplexeru se odpovědi ignorují, protože
  tmux tvrdí, že sixel umí, ať za ním sedí cokoli.
- **Který protokol kde.** kitty, Ghostty a iTerm2 3.7.7 nebo novější: protokol kitty.
  WezTerm a Rio: protokol iTerm2 s mísením průhlednosti PNG. mintty a iTerm2 starší než
  3.7.7: protokol iTerm2 s neprůhledným pozadím. Windows Terminal, foot, Tabby, Konsole
  a mlterm: **sixel**. Vše ostatní: znaky a avatar se změní na odznak s iniciálami.
  `ui.image_protocol` vynutí jeden z nich.
- **iTerm2 3.7.7.** Každý vložený obrázek `OSC 1337` otevře modální dialog „Allow
  Terminal-Initiated Display?“, který blokuje okno, dokud nezaškrtnete „Remember my
  choice“. Stejné obrázky poslané přes protokol kitty, včetně Unicode placeholderů, se
  kreslí bez dotazu, proto tam GitPoulpi používá protokol kitty.
  `ui.image_protocol = "iterm2"` dál dává vložené obrázky.
- **Sixel** se kreslí pro **sloupec grafu** (dráhy, uzly a s `ui.avatars = "graph"` i
  avatary uvnitř kroužků commitů) a jinde pro **avatary** (Log, detail commitu, Historie
  souboru, Blame, zobrazení obrázku). Sixel nemá alfa kanál, id ani změnu měřítka, proto
  má obrázek přesně `sloupce x šířka buňky` krát `řádky x výška buňky` pixelů, kreslí se
  jen tam, kde něco je, přes prázdné buňky se zachovaným pozadím; překóduje se jen při
  změně drah, pozice posunu nebo přiblížení. Sixel nelze smazat: GitPoulpi ho odstraní
  zapsáním textu přes jeho buňky. Sixel potřebuje skutečnou velikost buňky v pixelech:
  Windows Terminal, Tabby a mintty odpovídají na `CSI 16 t` jednou při startu a po změně
  přiblížení (při mlčení nikdy neblokuje); jinde se bere z velikosti okna terminálu nebo
  z `GITPOULPI_TERMINAL_CELL_PX=10x20`. Terminál, který sixel kreslí, ale nepozná se,
  potřebuje `ui.image_protocol = "sixel"`. Windows Terminal před 1.22 sixel nekreslí;
  nastavte tam `ui.image_protocol = "none"`.
- **True color**: motivy jsou RGB. `ui.color_depth = "auto"` sníží na 256 barev při
  `TERM` typu `*-256color` bez `COLORTERM=truecolor` (a v Terminal.app, který true color
  nemá) a na 16 barev v konzoli Linuxu.
- **Myš**: hlášení myši SGR s tlačítky, tažením a pohybem (zpětná vazba při najetí,
  drag and drop, pravé tlačítko, dvojklik).
- **Dotaz při startu.** Čtyři otázky odejdou jedním zápisem: příznaky klávesnice kitty
  (`CSI ? u`), barva pozadí (`OSC 11`, používá se jen pro `ui.theme = system`), jméno
  terminálu (XTVERSION, `CSI > 0 q`) a jeho atributy zařízení (`CSI c`, poslední: odpovídá
  na ně každý terminál, takže uzavírá čtení). Čtení je jedna smyčka `poll` s limitem 100 ms,
  dokončená dřív, než se spustí čtečka klávesnice. Terminál, který neodpoví nic, stojí
  jednou 100 ms a vše zůstane, jak říká prostředí. Jen Unix: ve Windows hlásí klávesy samotná
  konzole a rozhoduje prostředí. Odpovědi jsou ve zprávě o prostředí v Protokolu aktivity
  (řádek `start-up query:`).
- **Klávesnice**: **klávesnicový protokol kitty** (jen příznak disambiguate) se vyžádá při
  startu. Díky němu dorazí `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` a `Ctrl+Shift+Z`. Bez něj
  zůstanou starší klávesy (`Ctrl+Y` provede znovu, varianty s `Alt` nahrazují některé
  akordy). Viz
  [Klávesové zkratky](keybindings.md#klávesy-které-terminál-neumí-poslat-a-klávesnicový-protokol-kitty).
- **Schránka**: každá akce kopírování se řídí jednou politikou. Lokálně **systémová
  schránka** počítače, na kterém GitPoulpi běží, plus `OSC 52` terminálu, když je známo,
  že ho respektuje (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` je také záložní cesta, když systémová schránka selže. **Přes SSH
  nebo bez displeje** (konzole Linuxu, kontejner) se posílá jen `OSC 52` a uvnitř tmux se
  posílá obalený pro passthrough i holý. Stavový řádek řekne „zkopírováno“ jen když text
  převzala systémová schránka; to, co šlo terminálu, říká „odesláno do schránky terminálu
  (OSC 52, nepotvrzeno)“, protože nic nepotvrzuje, že terminál poslechl. Zpráva o prostředí
  v Protokolu aktivity jmenuje cestu na řádku `clipboard:`.
- **Tvar ukazatele myši** (ukazatel změny velikosti nad děliči, ruka nad klikatelnými
  položkami, I-beam nad textovými poli): `OSC 22`, bonus tam, kde ho terminál implementuje.
  GitPoulpi ho posílá vždy (kromě konzole Linuxu a za screen, zellij a herdr, které ho
  spolknou nebo vypíšou; za tmux se obalí pro passthrough). Pod `TERM_PROGRAM=mintty` posílá
  vlastní názvy kurzorů mintty (`hand`, `ibeam`, `sizewe`, ...), jediné, které mintty čte.
  **Rio `OSC 22` neimplementuje v žádném OS**: jeho zdroj sekvenci parsuje a zahazuje, takže
  to žádné nastavení na žádné straně nezmění.
- **Kurzor textu**: s `ui.text_cursor = "terminal"` (výchozí) se vlastní kurzor terminálu
  umístí na místo vkládání v aktivním textovém poli, jako blikající svislá čárka
  (`DECSCUSR`), a jinde se skryje; při ukončení se vrátí váš vlastní styl kurzoru. `drawn`
  místo toho vykreslí vlastní kurzor GitPoulpi.
- **Události fokusu** jsou zapnuty (tooltipy se zavřou, když okno ztratí fokus).
  **Bracketed paste** se vyžádá při startu a uvolní při každém odchodu (Unix), takže
  víceřádkové vložení dorazí jako jedno vložení: jeho zalomení řádků nikdy neodešlou pole a
  jeho písmena nikdy nespustí zkratky. Ve Windows konzole závorky vložení nemá; co místo toho
  GitPoulpi dělá, viz [Windows](#windows). GitPoulpi si nevyžádá hypertextové odkazy OSC 8
  ani undercurl. Jen na cestě s obrázky se každý snímek obalí synchronizovanou aktualizací
  (režim 2026, terminály, které ho neznají, ho ignorují), aby obrázek neblikl přes
  vyskakovací okno.
- **Emoji a symboly**: GitPoulpi používá jen glyfy s výchozí emoji prezentací nebo
  jednoznakové tvary z BMP, protože terminály a písma se v selektorech variant
  neshodují.

## Tabulka schopností

Řádky jsou terminály, sloupce OS. „GitPoulpi“ v buňce znamená, že tam běželo samotné
GitPoulpi; „sonda“ znamená, že běžely jen surové escape sekvence. Buňky **n/a**: kitty nemá
nativní sestavení pro Windows (vydání dodávají Linux a macOS); Ghostty nemá sestavení pro
Windows (multiplatformní je jen jeho knihovna libghostty); foot je terminál pro Wayland (jen
Linux); xterm je klient X11 (Linux a macOS přes XQuartz); iTerm2 a Terminal.app jsou jen pro
macOS; Windows Terminal, conhost a mintty jsou jen pro Windows (mintty také pod Cygwin a
MSYS2).

**V macOS neměřeno: kitty, Ghostty, WezTerm, Rio a Warp.** Potřebují GPU (Metal nebo
OpenGL 3.3). Jejich výsledky z Linuxu jsou referencí toho, co dělá tentýž program; než se na
buňku pro macOS spolehnete, zopakujte měření na Macu s GPU.

### Obrázky: který protokol GitPoulpi vybere a zda se zobrazí

| Terminál | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funguje**: protokol kitty (GitPoulpi) | n/a | neměřeno (potřebuje GPU) |
| **Ghostty** | **funguje**: protokol kitty (GitPoulpi) | n/a | neměřeno (potřebuje GPU) |
| **WezTerm** | **funguje**: protokol iTerm2 s průhledností (GitPoulpi). Placeholdery se kreslí stlačené, proto se protokol kitty nepoužívá | **funguje**: protokol iTerm2 s průhledností, graf a avatary na svých řádcích (oficiální 20240203 i sestavení s upstream PR #8092). Oficiální 20240203 dával obrázky na špatné řádky, dokud GitPoulpi kolem nich neukázalo kurzor, viz „Limit ConPTY“. | neměřeno (potřebuje GPU) |
| **foot** | **funguje**: sixel s průhledností (GitPoulpi) | n/a | n/a |
| **Rio** | **funguje**: protokol iTerm2 s průhledností (GitPoulpi) | **funguje**: protokol iTerm2 s průhledností, přesně na buňky při každé vyzkoušené velikosti písma (0.5.28; GitPoulpi). Sixel ani kitty nikdy nedorazí (ConPTY je zahazuje), takže není z čeho vybírat | neměřeno (potřebuje GPU) |
| **Alacritty** | text: **funguje** (terminál nemá podporu obrázků) | text: **částečně** (žádné obrázky; obrázkový diff z bloků) | text: **funguje** |
| **Tabby** | **funguje**: sixel s průhledností (sonda; GitPoulpi v Tabby na Linuxu nespuštěno) | **funguje**: sixel, zarovnaný (GitPoulpi) | **funguje**: sixel, průhlednost zachována (GitPoulpi) |
| **Warp** | neměřeno (potřebuje XWayland a špatně dekóduje syntetické stisky kláves, takže nešlo nic napsat) | **nefunguje** (terminál: žádné obrázky sixel, iTerm2 ani kitty); GitPoulpi neměřeno (nešlo ho spustit ze vstupního pole Warp) | neměřeno (potřebuje GPU) |
| **xterm** | sonda: sixel **funguje** s `-ti vt340`; GitPoulpi ho vybere z atributů zařízení (dotaz při startu přišel až po měření), nebo nastavte `ui.image_protocol = "sixel"` | n/a | text: **funguje**; sixel se kreslí (sonda) a potřebuje `ui.image_protocol = "sixel"` |
| **iTerm2** | n/a | n/a | **funguje**: protokol kitty od 3.7.7, bez dialogu (GitPoulpi). Starší verze dostanou vložené obrázky s neprůhledným pozadím. Sixel se kreslí, ale nenastavené pixely maluje neprůhledně, takže `ui.image_protocol = "sixel"` dá plný obdélník |
| **Terminal.app** | n/a | n/a | text: **funguje** (žádné obrázky) |
| **Windows Terminal** | n/a | **funguje**: sixel (GitPoulpi, 1.25) | n/a |
| **conhost** | n/a | text: **částečně** (znakový graf, obrázkový diff z půlbloků) | n/a |
| **mintty** | n/a | **nefunguje** (terminál za ConPTY: nekreslí se žádný obrázek, ani prostý `OSC 1337` umístěný se zobrazeným kurzorem; sloupec grafu zůstane prázdný; text a paleta jsou v pořádku) | n/a |

### Tvar ukazatele myši (OSC 22)

| Terminál | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funguje** | n/a | neměřeno (potřebuje GPU) |
| **Ghostty** | **funguje** | n/a | neměřeno (potřebuje GPU) |
| **WezTerm** | **nefunguje** (terminál: v testovaném sestavení neimplementováno) | oficiální: **nefunguje** (terminál). Sestavení s upstream PR #8092 pro WezTerm, který tvary ukazatele přidává: **funguje** (ruka, I-beam, šipky změny velikosti) | neměřeno (potřebuje GPU) |
| **foot** | **funguje** | n/a | n/a |
| **Rio** | **nefunguje** (terminál: parsuje a zahazuje) | **nefunguje** (terminál, ze stejného důvodu; ne ConPTY) | **nefunguje** (terminál, stejný zdroj; spuštěno jen v Linuxu a Windows) |
| **Alacritty** | **nefunguje** (terminál) | **nefunguje** (terminál) | **nefunguje** (terminál) |
| **Tabby** | **nefunguje** (terminál: xterm.js) | **nefunguje** (terminál) | **nefunguje** (terminál) |
| **Warp** | neměřeno | **nefunguje** (terminál) | neměřeno |
| **xterm** | **nefunguje** (terminál: neznámé názvy CSS) | n/a | neměřeno (ukazatel byl mimo okno) |
| **iTerm2** | n/a | n/a | **nefunguje** (terminál) |
| **Terminal.app** | n/a | n/a | **nefunguje** (terminál) |
| **Windows Terminal** | n/a | **nefunguje** (terminál) | n/a |
| **conhost** | n/a | **nefunguje** (terminál) | n/a |
| **mintty** | n/a | **funguje** s vlastními názvy mintty; GitPoulpi je posílá | n/a |

Kde buňka říká „nefunguje“, dělič nebo tlačítko pod myší se přesto rozsvítí.

### Klávesnicový protokol

GitPoulpi žádá o klávesnicový protokol kitty v Linuxu a macOS. Ve Windows nežádá: modifikátory
hlásí samotná konzole a akordy níže omezuje to, co si terminál ponechává pro sebe.

| Terminál | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funguje** | n/a | neměřeno (potřebuje GPU) |
| **Ghostty** | **funguje** | n/a | neměřeno (potřebuje GPU) |
| **WezTerm** | **částečně**: žádná odpověď, dokud není v `wezterm.lua` `enable_kitty_keyboard = true` (výchozí `false`) | **částečně**: WezTerm si ponechává `Ctrl+Shift+N` (otevírá své okno); `Ctrl+,` nedorazí (oficiální sestavení). Sestavení s PR #8092 si nechává ani jeden akord, ale `Ctrl+,` Nastavení stále neotevře (žádná odpověď klávesnice kitty přes ConPTY) | neměřeno (potřebuje GPU) |
| **foot** | **funguje** | n/a | n/a |
| **Rio** | **funguje** (ve výchozím stavu zapnuto) | **částečně**: Rio si ponechává `Ctrl+Shift+N` (otevírá své okno) | neměřeno (potřebuje GPU) |
| **Alacritty** | **funguje** (0.13 a novější) | **částečně**: `Ctrl+Shift+N` dorazí jako `Ctrl+N`, `Ctrl+,` nedorazí | **funguje** |
| **Tabby** | **nefunguje** (terminál: žádná odpověď; `Ctrl+Shift+N` a `Ctrl+,` si ponechává) | **částečně**: Tabby si ponechává `Ctrl+Shift+N` a `Ctrl+,` | **nefunguje** (terminál: žádná odpověď); některé akordy nedorazily |
| **Warp** | neměřeno | neměřeno | neměřeno (potřebuje GPU) |
| **xterm** | **nefunguje** (terminál) | n/a | **nefunguje** (terminál) |
| **iTerm2** | n/a | n/a | **funguje** (3.7.7; pokud ne, zkontrolujte *Profiles > Keys*) |
| **Terminal.app** | n/a | n/a | **nefunguje** (terminál: žádná odpověď; starší klávesy) |
| **Windows Terminal** | n/a | **částečně**: WT si ponechává `Ctrl+Shift+N` (nové okno) a `Ctrl+,` (své nastavení) | n/a |
| **conhost** | n/a | neměřeno | n/a |
| **mintty** | n/a | **částečně**: ani jeden akord neměl viditelný účinek | n/a |

Kde akord nedorazí, použijte varianty s `Alt`
([Klávesové zkratky](keybindings.md#klávesy-které-terminál-neumí-poslat-a-klávesnicový-protokol-kitty)),
nebo akord předefinujte v *Nastavení > Klávesnice a myš*.

### Schránka (akce kopírování)

Kopírování vždy používá systémovou schránku, když je k dispozici lokální displej, takže
tyto buňky popisují cestu `OSC 52` terminálu, která má smysl přes SSH, v kontejnerech a jako
záloha. Každý test kopírování GitPoulpi, který používal systémovou schránku, fungoval.

| Terminál | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funguje** | n/a | neměřeno (potřebuje GPU) |
| **Ghostty** | **funguje** (viděno za tmux, zellij, herdr); holý: neměřeno (zápis do schránky potřebuje fokus klávesnice, který mu test nedal) | n/a | neměřeno (potřebuje GPU) |
| **WezTerm** | **funguje** | **funguje** | neměřeno (potřebuje GPU) |
| **foot** | neměřeno (zápis potřebuje fokus klávesnice, který mu test nedal) | n/a | n/a |
| **Rio** | neměřeno (ze stejného důvodu) | **funguje** | neměřeno (potřebuje GPU) |
| **Alacritty** | neměřeno (ze stejného důvodu) | **nefunguje** (schránka beze změny s výchozí konfigurací; nezkoumáno) | **funguje** |
| **Tabby** | **funguje** | **funguje** | **funguje** |
| **Warp** | neměřeno | **nefunguje** (schránka beze změny) | neměřeno (potřebuje GPU) |
| **xterm** | neměřeno | n/a | **nefunguje** (schránka beze změny) |
| **iTerm2** | n/a | n/a | **nefunguje**, dokud není zapnuto *Settings > General > Selection > Applications in terminal may access clipboard* |
| **Terminal.app** | n/a | n/a | **nefunguje** (terminál) |
| **Windows Terminal** | n/a | **funguje** (když má fokus) | n/a |
| **conhost** | n/a | neměřeno (žádná sonda nebyla spuštěna) | n/a |
| **mintty** | n/a | **nefunguje** (schránka beze změny s výchozím nastavením) | n/a |

### Vložení

V Linuxu a macOS GitPoulpi žádá o bracketed paste, takže víceřádkové vložení je jedno
vložení. To nebylo spuštěno terminál po terminálu (**neměřeno** po terminálech). Ve Windows
konzole závorky vložení nemá, takže terminál, který se nejdřív nezeptá, doručí text jako
stisky kláves a každé zalomení řádku by byl `Enter`. GitPoulpi proto bere stisky kláves,
které po sobě následují do 30 ms (vnitřní `Enter` nebo osm a více znaků), jako jedno
vložení. Držené klávesy a „abc, `Enter`“ napsané ručně vložením nejsou.

| Terminál | Windows (dvouřádkové vložení do výzvy pro novou větev) |
|---|---|
| **Windows Terminal** | **funguje**: WT se nejdřív zeptá „vložit více řádků?“ (jeho vlastní varování), pak se žádná větev nevytvoří |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **funguje** (žádná větev se nevytvoří) |
| **Warp** | neměřeno (GitPoulpi se v něm nepodařilo spustit) |
| **conhost** | neměřeno (`Ctrl+V` a `Shift+Insert` poslané testem do klasické konzole nevložily) |

### Šířka emoji a ikon

| Terminál | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **funguje** (široké buňky zarovnané; kitty kreslí ozubené kolo U+2699 prázdné, známý problém písma) | WezTerm, Rio, Tabby, Windows Terminal: **funguje**. Alacritty: **částečně** (emoji jako jednobarevné obrysy) | Alacritty, Tabby: **funguje**. Ostatní: neměřeno (potřebují GPU) |
| **Tabby** | **funguje** | **funguje** | **funguje** |
| **Warp** | neměřeno | **funguje** | neměřeno |
| **xterm** | **částečně** (čtverečky: písmo) | n/a | **částečně** (čtverečky: písmo) |
| **iTerm2**, **Terminal.app** | n/a | n/a | **funguje** |
| **conhost** | n/a | **částečně**: barevná emoji se kreslí, široké buňky zarovnané | n/a |
| **mintty** | n/a | **funguje** | n/a |

### Testované verze

kitty 0.49.2 (macOS 0.50.0 se nespustilo), Ghostty 1.4.0, WezTerm 20260716 v Linuxu a
20240203 (poslední označené vydání) ve Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) a
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) a 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Terminály se rychle mění: novější verze se může lišit.

## Po jednotlivých terminálech

### Windows

- **Windows Terminal.** Výchozí ve spouštěči a nejlepší podpora obrázků ve Windows:
  graf a avatary sixel, OSC 52, dobrá emoji a odpovídá na dotaz na velikost buňky. Nerd Font
  zvolte v *Settings > Profiles > Defaults > Appearance > Font face*. Podržte `Shift` pro
  výběr textu, když myš zachytává aplikace. Ponechává si `Ctrl+Shift+N` a `Ctrl+,`: použijte
  varianty s `Alt` nebo akordy předefinujte. Tvar ukazatele se nemění.
- **WezTerm.** V `wezterm.lua` nastavte `config.enable_kitty_keyboard = true` (sestavení pro
  Linux a macOS) a písmo se záložními emoji, například
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Obrázky používají protokol iTerm2 a nic se nastavovat nemusí: poslední označené vydání,
  20240203-110809 (únor 2024), je ukotví na správné řádky (znovu ověřeno v listopadu 2026
  s aktuálním GitPoulpi; před opravou ConPTY popsanou níže ne). Upstream PR #8092 pro WezTerm
  přidává tvary ukazatele.
- **Rio** (MSI, vyžaduje UAC). Obrázky (graf, avatary, obrázkový diff) používají protokol
  iTerm2 s průhledností a dopadnou přesně na své buňky: změřeno na 0.5.28 s výchozím písmem
  a při 18 a 22,5 px (buňky 8x16, 11x21 a 13x26 pixelů, vždy celé pixely), obrázek 20x10
  buněk i 4x27 buněk vyšel přesně na buňky. Nic se nastavovat nemusí. Pseudokonzole
  spolkne dotaz na velikost buňky, takže GitPoulpi předpokládá, že buňky Rio jsou dvakrát
  vyšší než široké (platí u všech tří velikostí); s písmem jiných proporcí nastavte
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (například `10x24`), aby značky commitů byly kulaté. Rio
  nemá tvary ukazatele v žádném OS.
- **Tabby.** Graf a avatary sixel, kopírování i vložení fungují. Tabby si `Ctrl+Shift+N` a
  `Ctrl+,` ponechává pro sebe.
- **Alacritty.** Potřebuje OpenGL 3.3. Znakový graf; kopírování přes `OSC 52` nebylo s
  výchozí konfigurací zapnuto.
- **mintty** (Git Bash, MSYS2, Cygwin). Tvary ukazatele fungují (vlastní názvy mintty).
  Obrázky se nezobrazují (sloupec grafu zůstane prázdný; mintty 3.8.3 za ConPTY nekreslí
  žádný obrázek `OSC 1337`, ani když ho umístí prostá sonda) a dotaz na velikost buňky tam
  neodpovídá: nastavte `ui.graph_renderer = "text"`.
- **Warp.** S GitPoulpi neměřeno: nešlo ho spustit ze vstupního pole Warp. Terminál nemá
  obrázky sixel, iTerm2 ani kitty.
- **conhost, cmd, okno PowerShellu.** Znakový graf, obrázkový diff z půlbloků. GitPoulpi
  spuštěné z takového okna se ve výchozím stavu znovu otevře ve vámi zvoleném terminálu;
  viz „Pravidlo znovuspuštění“.

**Limit ConPTY.** Windows spouští program každého terminálu za pseudokonzolí (ConPTY).
Windows Terminal a sestavení WezTerm, která nesou novou, sekvence obrázků propouštějí; ta
uvnitř Windows (používají ji Rio a mintty) a ta dodávaná s WezTerm 20240203 ne. Co se
ztrácí: sekvence DCS (sixel) a APC (grafika kitty) se zahazují, takže dorazí jen obrázky
iTerm2 (`OSC 1337`), proto je Rio a WezTerm používají; `CSI 16 t` (velikost buňky) se
odpovídá jen ve Windows Terminal a Tabby. Co bylo opraveno: `OSC 1337` projde ConPTY hned a
dopadne na kurzor, který ConPTY naposledy vykreslil, a ConPTY kurzor kreslí jen dokud je
viditelný. GitPoulpi kurzor drží skrytý, takže jeho obrázky dopadaly na zastaralé řádky
(graf přes zápatí, roztržený obraz po posunu). Nyní ve Windows kolem každého zapsaného
obrázku kurzor ukáže, uvnitř téhož zápisu, a znovu skryje; Rio a WezTerm 20240203 každý
obrázek ukotví na jeho buňku (`pty_conpty_picture_anchor` čte pohyb kurzoru za ConPTY
zpět). Mimo Windows ani u sixel se nic nemění.

**Vložení ve Windows.** Viz tabulka „Vložení“ výše: GitPoulpi slévá nárazy stisků kláves
do jednoho vložení, takže víceřádkové vložení v testovaných terminálech nikdy neodešle
výzvu.

**Pravidlo znovuspuštění.** Dvojklik na exe ho spustí ve vlastním okně klasické konzole
(conhost). Když `ui.launch_terminal` jmenuje jiný terminál (výchozí: Windows Terminal),
GitPoulpi se tam znovu otevře a první okno se zavře (`ui.relaunch_in_terminal`). Dělá to
**jen** tehdy, když je samo v klasickém okně konzole. Terminál, který exe spouští přímo
(`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty `-e`, Tabby `run`), je tam, kde chcete být,
a nic se znovu nespouští; z shellu není samo, takže se také nic neděje. Vypněte pomocí
`ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Vše funguje bez nastavení. `Shift+klik` patří kitty, proto rozšiřte výběr
  pomocí `Alt+klik` ([Myš](mouse-and-gestures.md)).
- **Ghostty.** Vše funguje bez nastavení. Pokud se ikony ukazují jako díry, přidejte do
  `font-family` emoji nebo Nerd Font (klíč zopakujte pro přidání záložních).
- **WezTerm.** Obrázky přes protokol iTerm2, průhlednost se míchá. Pro celé akordy nastavte
  `enable_kitty_keyboard = true`. Žádné tvary ukazatele.
- **foot.** Graf a avatary sixel, tvary ukazatele, celá klávesnice, rychlý. V `foot.ini`
  nastavte `font=JetBrainsMono Nerd Font:size=11`. Kopírování přes `OSC 52` řídí
  `security.osc52`.
- **Rio.** Obrázky přes protokol iTerm2 s průhledností, celá klávesnice. Žádné tvary
  ukazatele (Rio `OSC 22` zahazuje).
- **Alacritty.** Znakový graf, klávesnice kitty od 0.13. Žádná podpora obrázků.
- **Tabby.** Obrázky sixel; žádná klávesnice kitty, žádné tvary ukazatele (Electron,
  xterm.js). Ponechává si `Ctrl+Shift+N` a `Ctrl+,`.
- **xterm.** Spusťte jako `xterm -u8 -ti vt340` pro sixel (UTF-8 je potřeba pro kreslení
  rámečků). Čtverečky místo emoji: písmo.
- **Warp.** Neměřeno: potřebuje XWayland a v testu špatně dekódoval syntetické stisky kláves.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Nebyly součástí průchodu z listopadu 2026.
  Fungují se znaky a staršími klávesami: použijte varianty s `Alt` a `Ctrl+Y` pro znovu.
  VTE `OSC 52` ignoruje, takže kopírování přes SSH se do schránky nedostane. Metoda zadávání
  (IBus, Fcitx) může zachytit přepínač stage `Ctrl+Space`; předefinujte ho v *Nastavení >
  Klávesnice a myš*. Konsole se pro sixel pozná podle vlastní dokumentace.
- **Wayland, Hyprland, Omarchy.** Nápověda ukazatele (`OSC 22`) funguje v kitty, Ghostty a
  foot. Položka spouštěče Omarchy jde přes `xdg-terminal-exec`.

### macOS

- **iTerm2.** 3.7.7 a novější kreslí obrázky přes protokol kitty, bez dotazu. Nastavení,
  která pomáhají:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    potřeba pro kopírování přes `OSC 52` (jinak iTerm2 ukáže „the terminal attempted to
    access the clipboard but it was denied“).
  - *Profiles > Keys > Left Option key: Esc+*: potřeba pro akordy s `Alt`.
  - Pokud nastavíte `ui.image_protocol = "iterm2"` (nebo použijete verzi starší než 3.7.7),
    zaškrtněte „Remember my choice“ v dialogu „Allow Terminal-Initiated Display?“. Obrázky
    pak mají neprůhledné pozadí (viditelný obdélník, pokud se vaše pozadí od motivu liší;
    `ui.graph_renderer = "text"` ho odstraní).
  - Klávesnicový protokol kitty je ve výchozím stavu zapnut (3.6.9 a novější).
- **Terminal.app.** Jen 256 barev: GitPoulpi si samo zvolí `ui.color_depth = 256` a motivy
  vypadají plošeji. Žádné obrázky, žádná klávesnice kitty, žádná schránka přes `OSC 52`.
  Pro akordy s `Alt` zaškrtněte *Use Option as Meta key*.
- **Alacritty, Tabby.** Alacritty: znakový graf, klávesnice kitty, kopírování přes `OSC 52`;
  pro akordy s `Alt` nastavte `option_as_alt = "Both"`. Tabby: graf a avatary sixel.
- **Option jako Meta.** Option skládá znaky (Option+B napíše znak integrálu), dokud ho
  terminál nepřemění na Meta: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. Akordy s `Alt` v GitPoulpi to potřebují a nikde není
  ve výchozím stavu zapnuto.
- **kitty, Ghostty, WezTerm, Rio, Warp**: v macOS neměřeno (potřebují GPU). Očekávejte
  chování z Linuxu.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel potřebuje `ui.image_protocol = "sixel"`.

### VS Code a Warp

Terminál VS Code (`TERM_PROGRAM=vscode`) nebyl součástí průchodu. Pozná se podle atributů
zařízení, když je zapnuto `terminal.integrated.enableImages`, jinak GitPoulpi kreslí znaky;
`ui.image_protocol = "sixel"` obrázky vynutí. Warp: viz poznámky podle OS výše.

## Multiplexery

tmux, zellij, screen, herdr, psmux a vlastní panely terminálů stojí mezi GitPoulpi a
terminálem a mění, co projde. Ve zkratce: tmux propouští sixel a s
`allow-passthrough on` i tvary ukazatele a obrázky kitty; zellij a screen nepropouštějí
obrázky ani tvary ukazatele; herdr propouští obrázky kitty. Všechny podrobnosti a
konfigurace k přidání jsou v [Multiplexery](multiplexers.md).

## Co chybí a čím se nahrazuje

| Chybí | Co dostanete místo toho |
|---|---|
| Protokol obrázků | Znakový graf. Avatar se změní na barevný odznak s iniciálou (jedno písmeno) bez ohledu na `ui.avatar_fallback`: identicon vmáčknutý do textových buněk je nerozpoznatelný. Obrázkový diff a zobrazení Soubor použijí barevné půlbloky nebo informační kartu. |
| Truecolor | Barvy se převedou na paletu 256 nebo 16 barev (`ui.color_depth`). |
| Klávesnicový protokol kitty | Starší klávesy; některé akordy dorazí jako jiná klávesa. Nastavení varuje a vy akordy předefinujete nebo použijete varianty s `Alt`. |
| OSC 52 | Lokálně se nic nemění (použije se systémová schránka). Přes SSH nebo bez displeje akce kopírování nedají do schránky nic: vyberte myší (držte `Shift`) a použijte vlastní kopírování terminálu. |
| OSC 22 | Ukazatel se nemění; dělič nebo tlačítko pod myší se přesto rozsvítí. |
| Písmo s emoji | Čtverečky nebo díry místo ikon; graf a text přesto fungují. Nainstalujte Nerd Font nebo písmo s emoji a uveďte ho jako záložní. |

## Nastavení, která stojí za to znát

| Nastavení | K čemu |
|---|---|
| `ui.image_protocol` | `auto` (výchozí), `kitty`, `iterm2`, `sixel`, `none`. Nastavte `none`, pokud na místě obrázků vidíte změť znaků. Restart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` je vždy správně (použijte tam, kde platí limit ConPTY). Restart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` nebo `off`, pokud se terminál nebo multiplexer chová špatně. Restart. |
| `ui.text_cursor` | `terminal` (výchozí) nebo `drawn`. |
| `ui.mouse_enabled` | Vypnuto nechá myš terminálu. Restart. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | Terminál, který otevírají zástupci OS, a zda ho dvojklik znovu otevře v GitPoulpi ([Nastavení](settings.md)). |

Viz také [Multiplexery](multiplexers.md), [Obrázky](image-view.md),
[Klávesové zkratky](keybindings.md) a [Řešení potíží](troubleshooting.md).

## Zdroje

Dokumentace ověřena v říjnu 2026, chování testováno v listopadu 2026. Primární zdroje, pokud není uvedeno jinak.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [tvary ukazatele](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [stažení](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sekvence](https://wezterm.org/escape-sequences.html), [pull request k OSC 22](https://github.com/wezterm/wezterm/pull/6292), [vydání](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [vydání](https://github.com/alacritty/alacritty/releases)
- Konsole: [merge request k OSC 52](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [merge request ke klávesnici kitty](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [issue k OSC 52](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [issue ke klávesnici kitty](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [stažení](https://iterm2.com/downloads.html); klávesnice kitty z [matice terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (sekundární)
- Windows Terminal: [oznámení 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (sekundární), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [podpora escape sekvencí](https://rioterm.com/docs/escape-sequence-support) (bez řádku `OSC 22`), [funkce](https://rioterm.com/docs/features), [klávesnice kitty](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [aplikace na celou obrazovku](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 z [rozboru bezpečnostního upozornění Warp k OSC 52](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (sekundární)
- VS Code: [pokročilý terminál](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [řídicí sekvence](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tipy](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (bez truecolor): [diskuse Apple](https://discussions.apple.com/thread/255315820) (sekundární)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Srovnávací weby (sekundární, jen k ověření): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
