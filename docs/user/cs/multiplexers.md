[English](../multiplexers.md) · **Čeština**

# Multiplexery a panely

Multiplexer stojí mezi GitPoulpi a vaším terminálem: buňky si překresluje sám a
předává dál jen ty escape sekvence, kterým rozumí. Takže to, co funguje v samotném
terminálu ([Výběr terminálu](terminals.md)), nemusí fungovat uvnitř tmux, zellij,
screen nebo herdr. Tato stránka říká, co projde, a jakou konfiguraci přidat. Výsledky
pocházejí ze stejného testovacího průchodu z listopadu 2026 jako tabulky terminálů:
tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 a 0.46.0, GNU screen 5.0 (macOS dodává 4.00.03),
herdr 0.9.5, psmux 3.4.0.

GitPoulpi pozná multiplexer podle `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` nebo `tmux*`
a `HERDR_*`. Uvnitř jednoho z nich se obrázky ve výchozím stavu kreslí jako znaky, kromě
míst, kde tabulka níže říká, že GitPoulpi pozná fungující cestu. `GITPOULPI_TERMINAL_IMAGE=off`
vypne všechny obrázky a `ui.image_protocol` vynutí protokol.

## Přehled

| | Obrázky | Tvar ukazatele (OSC 22) | Schránka (OSC 52) | Klávesy |
|---|---|---|---|---|
| **tmux** | sixel, když ho terminál za ním umí (pozná se); obrázky kitty s `allow-passthrough on` a `ui.image_protocol = "kitty"` | s `allow-passthrough on` v kitty a foot; ne v Ghostty | s `set-clipboard on` | žádná klávesnice kitty; `Ctrl+Enter` a `Ctrl+,` nedorazí nikdy |
| **zellij** | znaky | ne | ano | klávesnice kitty funguje |
| **GNU screen** | znaky | ne | ne | starší klávesy |
| **herdr** | obrázky kitty (pozná se s `kitty_graphics = true`) | ne | ano | neměřeno |
| **psmux** (Windows) | znaky (záměrně) | ne (Windows Terminal ho nemá) | funguje systémová schránka | neměřeno |
| **MSYS2 tmux** (Windows) | znaky (záměrně) | ne | funguje systémová schránka | neměřeno; obraz je rozbitý |
| **panely WezTerm**, **panely Windows Terminal**, **rozdělení kitty a Ghostty**, **rozdělené panely iTerm2** | jako samotný terminál | jako terminál | jako terminál | jako terminál |
| **iTerm2 `tmux -CC`** | znaky | ne | ne (iTerm2 to ve výchozím stavu odmítá) | neměřeno |

## tmux

**Obrázky.**

- **Sixel** kreslí samotný tmux, překódovaný pro klienta, který ho umí. GitPoulpi se
  tmux zeptá jednou při startu (`tmux display-message -p '#{client_termfeatures}'`,
  omezeno na 400 ms) a použije sixel, když ho klient uvádí. To platí za foot,
  WezTerm, Rio a Tabby; neplatí za kitty a Ghostty (tmux pak vypisuje text
  „SIXEL IMAGE“). Vyžaduje tmux sestavený se sixel (ten z Archu a Homebrew je;
  ověřte pomocí `tmux -V` a testu sixel). Nic se nenastavuje. Odpověď samotného tmux
  na dotaz na atributy zařízení tvrdí, že sixel umí, ať za ním sedí cokoli, proto se
  GitPoulpi místo toho ptá na vlastnosti klienta.
- **iTerm2 za tmux** sixel hlásí, ale nenastavené pixely maluje neprůhledně, takže by
  se převážně průhledný graf změnil v plný obdélník. GitPoulpi iTerm2 pozná
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, typ klienta) a kreslí znaky.
- **Obrázky kitty** procházejí DCS passthrough tmux (kitty, Ghostty; v surové sondě
  také iTerm2 a Rio). Potřebují `set -g allow-passthrough on` (tmux 3.3a nebo novější)
  **a** `ui.image_protocol = "kitty"` v GitPoulpi, které pak nahrávání a mazání obalí.
  Bez volby tmux se obal zahodí a graf zůstane prázdný.
- Vložené obrázky iTerm2 tmuxem projdou, ale tmux je neumí umístit: GitPoulpi je tam
  nikdy nepoužije.

**Tvar ukazatele.** GitPoulpi obalí `OSC 22` pro passthrough. K oknu se dostane v kitty
a foot s `allow-passthrough on`. Ghostty ho po obalení neukáže (zůstane šipka). WezTerm,
Rio, Alacritty, Tabby a iTerm2 žádné `OSC 22` k zobrazení nemají.

**Schránka.** `set -g set-clipboard on`. GitPoulpi posílá také tvar obalený pro passthrough
(funguje s `allow-passthrough on`). Za kitty, Ghostty, Alacritty a Tabby dorazí; iTerm2 ji
odmítá, dokud není zapnuto jeho oprávnění ke schránce.

**Klávesy.** tmux neodpovídá na dotaz na klávesnici kitty a `extended-keys` nepřinese
`Ctrl+Enter` ani `Ctrl+,`: naměřeno s tmux 3.8 uvnitř kitty, obojí dorazí jako prosté
`Enter` a `,` s `extended-keys` vypnutým, zapnutým i `always`. Písmenné akordy jako
`Ctrl+Shift+Z` dorazí jen s `extended-keys` zapnutým nebo `always` **a**
`extended-keys-format csi-u` (tmux 3.5 nebo novější) **a** aplikací, která o rozšířené
klávesy žádá; GitPoulpi to nedělá nikdy, takže zde nastavení nic nemění. Uvnitř tmux
používejte varianty s `Alt`
([Klávesové zkratky](keybindings.md#klávesy-které-terminál-neumí-poslat-a-klávesnicový-protokol-kitty)).

**Konfigurace** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a nebo novější: tvary ukazatele, vynucené obrázky kitty
```

**tmux uvnitř WSL.** Neměřeno.

## zellij

Obrázky: zellij přijímá protokol kitty, ale ne jeho Unicode placeholdery, a nenese ani
sixel, ani obrázky iTerm2, takže GitPoulpi kreslí znaky (záměrné veto). Tvar ukazatele:
spolkne se, takže ho GitPoulpi neposílá. Schránka: `OSC 52` se předává (funguje za kitty,
Ghostty, Alacritty, Tabby; iTerm2 a Terminal.app ho odmítají nebo ignorují). Klávesy: zellij
odpovídá na dotaz na klávesnici kitty, takže celá klávesnice funguje.

zellij 0.46 při novém startu otevře vyskakovací okno „Welcome to Zellij“: stiskněte `Esc`.
Relace, která skončila, si ponechá jméno a další start téhož jména se zeptá, co dělat.
Žádná konfigurace není potřeba.

## GNU screen

Nic neprojde: obrázky a `OSC 22` se vypíšou jako znaky nebo zahodí, `OSC 52` se ignoruje
a klávesnice je ta starší. GitPoulpi kreslí znaky. screen 4.00.03 (verze, kterou dodává
macOS) navíc kazí barvy a široké glyfy v rozhraní; screen 5 byl testován jen v Linuxu.
Pro použitelnou relaci použijte tmux nebo zellij.

## herdr

herdr vkládá terminál, který znovu vysílá **obrázky kitty**. GitPoulpi je použije, když
`config.toml` herdr uvádí `kitty_graphics = true` (v `[terminal]`, u starších verzí v
`[experimental]`) a terminál kolem herdr je kreslí (kitty, Ghostty, WezTerm): pozná se,
nic se nenastavuje. Pokud konfigurace leží jinde (například pod `HERDR_HOME`, který
GitPoulpi nečte), nastavte `ui.image_protocol = "kitty"`. Přes SSH (`herdr --remote`)
GitPoulpi lokálního klienta nevidí, takže kreslí znaky; viz
[Řešení potíží](troubleshooting.md#graf-je-vykreslen-znaky-ne-jako-obrázek-herdr-tmux-jiné-terminály).
`OSC 22` herdr 0.9.5 nepředává, takže ho GitPoulpi neposílá. `OSC 52` dorazí za kitty,
Ghostty, Alacritty a Tabby. V surové sondě herdr kreslil obrázky kitty v kitty, Ghostty a
iTerm2, i s výchozí konfigurací. Klávesy: neměřeno.

## psmux a MSYS2 tmux (Windows)

- **psmux** (Rust, ConPTY, příkazový jazyk tmux; `winget install psmux`) nastavuje `TMUX`,
  takže GitPoulpi tam záměrně kreslí znaky. Uvnitř Windows Terminal běží normálně a
  kopírování funguje přes systémovou schránku; ukazatel se nemění (Windows Terminal
  `OSC 22` nemá). Jiné hostitelské terminály neměřeny.
- **tmux z MSYS2 nebo Cygwin** také nastavuje `TMUX` (záměrně znaky) a v testu byl obraz
  rozbitý: zastaralé snímky přes sebe v mintty i ve Windows Terminal (s `-u` i bez; bez něj
  se kreslení rámečků vypisovalo jako `qqqq`). Nativní program Windows uvnitř pseudoterminálu
  MSYS2 tmux nepřekresluje správně. To GitPoulpi opravit nemůže: použijte psmux, panely
  Windows Terminal nebo panely WezTerm.
- zellij a screen nemají vydání pro Windows. herdr 0.9.5 byl nainstalován, ale nespuštěn.

## Panely terminálu

Panely, které patří samotnému terminálu, nejsou pro GitPoulpi multiplexery:

- **Panely WezTerm** (`wezterm cli split-pane`, `wezterm connect`, domény SSH): WezTerm
  kreslí všechno, takže obrázky fungují (graf a avatary v panelu) a ukazatel se chová jako
  v samotném WezTerm (tvary fungují v sestavení s upstream PR #8092 pro WezTerm; dělič panelu
  dává ukazatel změny velikosti). V Linuxu ani macOS neměřeno.
- **Panely Windows Terminal** (`wt ; split-pane`): graf a avatary sixel v panelu fungují,
  kopírování funguje, žádné tvary ukazatele.
- **Rozdělení kitty, Ghostty a iTerm2**: nativní, jako terminál. V macOS pro kitty a
  Ghostty neměřeno (potřebují GPU).

## iTerm2 `tmux -CC` (řídicí režim)

iTerm2 otevře pro každé okno tmux nativní okno a drží bránové okno „[tmux bash]“. `TMUX`
je nastaveno, takže GitPoulpi v nativním okně kreslí znaky. Obrázky se k iTerm2 přes řídicí
spojení skutečně dostanou (při testu `OSC 1337` se objevil dialog „Allow Terminal-Initiated
Display?“), ale GitPoulpi je tam nepoužívá. Obsah kitty vynuceného `ui.image_protocol =
"kitty"` se vypíše v bránovém okně, proto nechte `auto`. `OSC 22` a `OSC 52` nedorazí.

## Nastavení, která stojí za to znát

| Nastavení | K čemu |
|---|---|
| `ui.image_protocol` | `kitty` vynutí obrázky kitty přes multiplexer, který je předává (tmux s `allow-passthrough on`, herdr s konfigurací, kterou GitPoulpi nečte); `none` pro znaky. |
| `ui.graph_renderer` | `text` je vždy správně. |
| `ui.kitty_keyboard` | `off`, pokud se multiplexer s dotazem na klávesnici chová špatně. |

Terminál za multiplexerem viz [Výběr terminálu](terminals.md) a
[Řešení potíží](troubleshooting.md), když se graf kreslí znaky.
