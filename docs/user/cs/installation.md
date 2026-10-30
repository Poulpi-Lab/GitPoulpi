[English](../installation.md) · **Čeština** · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Instalace

GitPoulpi má tři spustitelné soubory:

| Spustitelný soubor | Co to je |
|---|---|
| `gitpoulpi-tui` | Terminálové rozhraní. |
| `gitpoulpi` | Rozhraní příkazového řádku (`--json` pro skripty). Bez podpříkazu otevře rozhraní. |
| `gitpoulpi-mcp` | Server MCP pro AI agenty. Viz [MCP a agenti](mcp-and-agents.md). |

Vedle nich archiv obsahuje `gitpoulpi-launch`, drobný program, který zástupci spouštějí, aby zobrazili už otevřené okno GitPoulpi (`gitpoulpi launch`). Ve Windows nemá okno konzoly, takže kliknutí žádné nezabliká. Instaluje se, aktualizuje a odebírá spolu s ostatními.

## Co potřebujete

- **Systémový `git`** v `PATH`. GitPoulpi používá libgit2 pro většinu operací
  a váš `git` pro zbytek.
  Je potřeba verze **2.34 nebo novější**. Na macOS funguje `git` z Apple Command Line Tools i ten z Homebrew.
- **Terminál** s podporou myši. kitty, Ghostty, WezTerm a iTerm2 (a také Windows Terminal a foot přes sixel) kreslí graf
  commitů jako obrázky. Ostatní terminály (konzole Windows, tmux a
  většina dalších) zobrazí znakový graf. Viz
  [Výběr terminálu](terminals.md).
- **Nerd Font** nebo písmo s emoji, pro ikony na panelu nástrojů a v
  postranním panelu.
- Volitelně: **`git-lfs`** 3.0 nebo novější, pro repozitáře s Git LFS. Bez něj
  se soubory LFS zobrazí jako malé soubory s ukazateli a GitPoulpi vám řekne
  proč. Jak jej nainstalovat, viz [Git LFS](lfs.md).
- Volitelně: **credential helper** (Git Credential Manager,
  libsecret, `pass`, Keychain), k uložení tokenů hostingových služeb a API
  klíčů AI. Viz [Hostingové služby a issue trackery](hosting-integration.md).

## Z archivu vydání

Archiv vydání je `gitpoulpi-<version>-<target>.zip` (Windows) nebo
`.tar.gz` (Linux, macOS) se souborem `.sha256` vedle něj. Ověřte archiv podle
souboru `.sha256`, rozbalte jej a dejte tři spustitelné soubory do složky v
`PATH`. Stejné kroky uvádí `INSTALL.txt` v archivu.

## Instalátorem

Každé vydání obsahuje také instalátor pro každý systém. Oba dělají stejné volby
(zástupce, PATH, alias příkazu jako `gp`) tak, že volají nainstalovaný
`gitpoulpi`, tedy kód za Nastavením > Obecné > Spouštěč, takže to, co instalátor
nastaví, a to, co ukazuje Nastavení, se vždy shoduje.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Instaluje se pro aktuálního uživatele, bez výzvy správce, do
`%LOCALAPPDATA%\Programs\GitPoulpi` (první dialog nabídne instalaci pro všechny
uživatele a volbu složky). Zaškrtávací pole: zástupce v nabídce Start, zástupce na
ploše, přidání do PATH. Jedna stránka se ptá na **alias příkazu**: výchozí je `gp`,
můžete zvolit libovolný název, prázdné znamená žádný. PowerShell už `gp` používá pro
`Get-ItemProperty`, proto tam zvolte jiný název nebo použijte `gitpoulpi`.

Instalátor zkopíruje tři spustitelné soubory a značku `.gitpoulpi-dist`, takže
[Aktualizovat nyní](#aktualizace-na-místě) funguje stejně jako u archivu zip.
Instalace pro jednoho uživatele je zapisovatelná, takže se umí aktualizovat sama;
instalace pro všechny uživatele v Program Files vyžaduje ke svému nahrazení
zvýšená oprávnění, proto ji přeinstalujte novým instalátorem.
Odinstalace probíhá v Nastavení Windows > Aplikace: spustí `gitpoulpi uninstall`
(alias, záznamy v PATH, zástupci) a smaže soubory včetně zbytků `.old` po aktualizaci.
Vaše nastavení a repozitáře zůstanou.

### Linux a macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Vybere archiv pro váš systém a procesor, ověří jeho `.sha256`, nainstaluje do
`~/.local/opt/gitpoulpi` a odkazy `gitpoulpi` a `gitpoulpi-mcp` zapíše do
`~/.local/bin`. Nepotřebuje root a nic mimo váš domovský adresář se nemění.
Bez `--yes` se zeptá na alias (výchozí `gp`), PATH a zástupce.

| Přepínač | Účinek |
|---|---|
| `--prefix DIR` | Nainstaluje soubory programu do `DIR` (absolutní cesta). |
| `--version X.Y.Z` | Nainstaluje toto vydání místo nejnovějšího. |
| `--archive FILE` | Použije místní archiv; vedle něj musí ležet `FILE.sha256` (`--no-verify` to přeskočí). Nepoužije se síť. |
| `--alias NAME` / `--no-alias` | `NAME` bude spouštět GitPoulpi (`gitpoulpi alias create`), nebo žádný alias. |
| `--no-path` | Nepřidá GitPoulpi do PATH nových shellů (jinak `gitpoulpi path add`). |
| `--desktop` | Vytvoří také zástupce v nabídce aplikací a na ploše. |
| `--yes` | Nikdy se neptá: platí přepínače a výchozí hodnoty (alias `gp`, PATH ano, zástupci ne). |
| `--uninstall` | Spustí `gitpoulpi uninstall`, potom odstraní odkazy a složku. |

Vyhledání nejnovějšího vydání vyžaduje veřejný repozitář; u soukromého stáhněte
archiv v prohlížeči a použijte `--archive`.

## Ze zdrojových kódů

Potřebujete aktuální stabilní toolchain Rustu (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Spustitelné soubory jsou v `target/release/`. Profil `release` se sestaví
rychle. Pro nejmenší a nejrychlejší spustitelné soubory použijte profil `dist`.
Sestavení trvá déle:

```
cargo build --profile dist
```

Skripty `build-opti` v kořeni repozitáře toto sestavení spustí pro všechny tři
spustitelné soubory a vypíšou jejich velikosti:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

Chcete-li sestavit jen jazyky a funkce, které používáte, viz
[Vlastní sestavení](custom-builds.md).

## Vlastní archiv vydání

```
cargo run -p xtask -- dist
```

Tím se sestaví profil `dist` a zapíše se
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) nebo `.tar.gz` (ostatní
systémy). Archiv obsahuje tři spustitelné soubory, `INSTALL.txt`,
`README.md`, `LICENSE` a `CHANGELOG.md`. Vedle něj se zapíše soubor `.sha256`.

## Aktualizace

Záhlaví okna ukazuje běžící verzi (`v1.0.0`) mezi názvem aplikace a cestou k repozitáři. Kliknutím na ni,
nebo příkazem **Nápověda: Poznámky k vydání** z palety (`Ctrl+P`), se otevře karta Poznámky k vydání
se seznamem změn, posunutým na běžící verzi.

Jednou denně GitPoulpi na pozadí zjistí na GitHubu, zda existuje novější vydání
(`updates.mode`, [Nastavení > Synchronizace](settings.md#synchronizace)).
Pokud ano, čip ukáže `v0.2.0 k dispozici` ve zvýrazňovací barvě a kliknutí na něj otevře
kartu Poznámky k vydání s poznámkami k tomuto vydání nahoře. Karta má tři tlačítka,
každé s klávesou a příkazem palety:

- **Stáhnout** (`d`, *Aktualizace: Stáhnout*) otevře stránku vydání v prohlížeči.
- **Přeskočit tuto verzi** (`s`, *Aktualizace: Přeskočit tuto verzi*) skryje oznámení,
  dokud se neobjeví novější vydání.
- **Zkontrolovat nyní** (`r`, *Aktualizace: Zkontrolovat aktualizace nyní*) se zeptá ihned znovu.

`updates.mode = "off"` neodešle žádný požadavek. Bez připojení k síti, nebo dokud je
repozitář soukromý a není uložen token pro github.com, kontrola tiše selže (jeden řádek
v protokolu aplikace, nic v rozhraní).
`GITPOULPI_UPDATE_API` nasměruje kontrolu na jiný kořen API (používají ho testy).

### Aktualizace na místě

Sestavení nainstalované z archivu vydání se umí nahradit samo. Archiv obsahuje
malý soubor `.gitpoulpi-dist` vedle spustitelných souborů; běžící sestavení je
„aktualizovatelné“ jen tehdy, když tento soubor leží vedle jeho spustitelného souboru (a uvádí
jeho platformu). Sestavení vytvořené přes `cargo`, vlastní sestavení zkopírované jinam
nebo balíček, který soubor vynechal, se nikdy samo neaktualizuje: jeho karta Poznámky k vydání
to řekne a nabídne jen **Stáhnout**.

U aktualizovatelného sestavení karta přidává:

- **Aktualizovat nyní** (`u`, *Aktualizace: Aktualizovat nyní*) stáhne archiv a jeho
  `.sha256` na pozadí (úloha v indikátoru stavového řádku, s procenty, jde zrušit
  jako každá úloha), ověří hash, rozbalí archiv
  a každý nový soubor položí vedle starého jako `<name>.new`. Nic běžícího se nedotkne.
  V režimu `auto` se totéž stane samo, jakmile kontrola najde novější vydání.
- **Restartovat pro aktualizaci** (`R`, *Aktualizace: Restartovat pro aktualizaci*; totéž říká čip,
  jakmile je stažení připraveno) je vždy vaše akce, nikdy automatická. Zastaví samostatného
  démona relace (vestavěný zaniká s oknem), vymění soubory a spustí novou verzi se stejnými
  argumenty a relací.
- **Vrátit se k předchozí verzi** (`v`, *Aktualizace: Vrátit se k předchozí verzi*) vrátí předchozí
  soubory a restartuje. Soubory předchozí verze zůstanou jako
  `<name>.old`, dokud nová verze neběží minutu, pak se smažou, takže tlačítko je tu
  pro první spuštění. Není součástí historie vrácení (Zpět): mění vlastní soubory GitPoulpi,
  ne repozitář.

Jak se soubory nahrazují: v Linuxu a macOS se každý nový soubor přejmenuje přes
starý (atomicky; běžící program si drží starou kopii, dokud neskončí). Ve
Windows se běžící `.exe` přepsat nedá, proto se starý přejmenuje na
`<name>.old` a nový se přesune na jeho místo; `.old`, který ještě běží, smaže
některé pozdější spuštění. Výměna je buď celá, nebo žádná: pokud některý ze
spustitelných souborů drží jiný program (třeba klient MCP spouštějící `gitpoulpi-mcp`), GitPoulpi pojmenuje
soubor, nic nezmění a stažení nechá připravené k dalšímu pokusu.

Co se odmítne, vždy se zprávou: vydání bez souboru `.sha256`, hash,
který nesouhlasí, soubor archivu, jehož název není ten pro tuto platformu
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), archiv, jehož značka uvádí
jinou verzi nebo platformu, a každý odkaz ke stažení mimo tyto servery: `github.com` GitHubu, `api.github.com`,
`objects.githubusercontent.com`, `release-assets.githubusercontent.com` a `github-releases.githubusercontent.com`
(první požadavek i každé přesměrování; přesměrování kamkoli jinam stažení zastaví).
Archivy soukromého repozitáře vyžadují relaci v prohlížeči, takže stažení
tam selže, dokud se repozitář nezveřejní.

## Ověření, že vše funguje

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` vypíše verzi a commit, ze kterého bylo sestavení pořízeno. Poté
pokračujte podle [První kroky](getting-started.md).

## Odinstalace

`gitpoulpi uninstall` (to spouští oba instalátory) odebere aliasy příkazu,
záznamy v PATH a zástupce, které GitPoulpi vytvořil, a potom soubory programu
instalace z vydání; odmítne sestavení bez značky `.gitpoulpi-dist`
(sestavení přes cargo) a `--keep-files` soubory programu ponechá. Agentům MCP
není k dispozici. Bez instalátoru smažte tři spustitelné soubory.
GitPoulpi uchovává také tyto soubory:

| Co | Kde |
|---|---|
| Nastavení (`config.toml`) a relace | Konfigurační složka vašeho OS. Změní ji `GITPOULPI_CONFIG_DIR`. |
| Stav jednotlivých repozitářů | `.gitpoulpi/` uvnitř každého repozitáře. |
| Cache (avatary) | Složka cache vašeho OS. |
| Tokeny a API klíče | Váš credential helper. **Nastavení > SSH a přihlášení > Zapomenout všechna jména a hesla** je odstraní. |

Přesné cesty viz [Referenční příručka konfigurace](config-reference.md).
