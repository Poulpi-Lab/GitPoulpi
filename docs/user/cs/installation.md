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
`PATH` (ponechte vedle nich `gitpoulpi-updater` a značku `.gitpoulpi-dist`:
značka říká `channel=archive`, takže se sestavení umí samo aktualizovat, viz
[Aktualizace](updates.md)). Stejné kroky uvádí `INSTALL.txt` v archivu.

### Ověření stažení

Každý soubor vydání je podepsán klíčem [minisign](https://jedisct1.github.io/minisign/) projektu a GitPoulpi tento podpis před instalací aktualizace ověřuje sám. U souboru, který jste stáhli ručně, je podpisem `.minisig` vedle něj. Veřejný klíč (id klíče `B461FFCBF6F6A4F2`) je v repozitáři commitnut jako `packaging/minisign.pub`:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Přečtěte si důvěryhodný komentář, který příkaz vypíše: musí být `file:<název souboru> version:<očekávaná verze>`. Podpis, který je platný, ale jmenuje jiný soubor nebo jinou verzi, není podpisem vašeho souboru.

Tam, kde bylo vydání sestaveno s atestacemi původu sestavení (proměnná repozitáře `ATTESTATIONS`), GitHub CLI ověří, kde byl soubor sestaven:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Soubor `.sha256` zachytí jen poškozené stažení. Podpis je to, co ukazuje, že soubor pochází od projektu.

## Instalátorem

Každé vydání obsahuje také instalátor pro každý systém. Oba dělají stejné volby
(zástupce, PATH, alias příkazu jako `gp`) tak, že volají nainstalovaný
`gitpoulpi`, tedy kód za Nastavením > Obecné > Spouštěč, takže to, co instalátor
nastaví, a to, co ukazuje Nastavení, se vždy shoduje.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

První dialog se ptá, pro koho je instalace:

| Volba | Kam | Práva správce | Co to znamená pro aktualizace |
|---|---|---|---|
| **Jen pro mě** (výchozí) | `%LOCALAPPDATA%\Programs\GitPoulpi` | žádná | GitPoulpi nahrazuje vlastní soubory: **Aktualizovat a restartovat** v [panelu aktualizací](updates.md), bez dotazu. |
| **Pro všechny uživatele** | `Program Files\GitPoulpi` | jeden dotaz při instalaci | Každá aktualizace žádá o schválení správcem (dotaz Windows) a relace GitPoulpi ostatních uživatelů na počítači se nejprve zavřou. |

Stejná volba funguje tiše: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` nebo `/ALLUSERS`.

Zaškrtávací pole: zástupce v nabídce Start, zástupce na ploše, přidání do PATH. Jedna stránka se ptá na **alias příkazu**: výchozí je `gp`, můžete zvolit libovolný název, prázdné znamená žádný (`/ALIAS=name` nebo `/ALIAS=` na příkazovém řádku). PowerShell už `gp` používá pro `Get-ItemProperty`, proto tam zvolte jiný název nebo použijte `gitpoulpi`.

Instalátor zkopíruje spustitelné soubory (včetně `gitpoulpi-updater`, malého programu, který při aktualizaci vyměňuje soubory) a značku `.gitpoulpi-dist`. Značka zaznamenává, jak byl GitPoulpi nainstalován (`inno-user` nebo `inno-machine`), což panel aktualizací ukazuje jako **Nainstalováno přes**. Opětovné spuštění instalátoru zachová vaše volby (rozsah, zástupce, alias).

Odinstalace probíhá v Nastavení Windows > Aplikace: spustí `gitpoulpi uninstall` (alias, záznamy v PATH, zástupci) a smaže soubory včetně zbytků `.old` po aktualizaci. Vaše nastavení a repozitáře zůstanou.

### Linux a macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Vybere archiv pro váš systém a procesor, ověří jeho `.sha256`, nainstaluje (kanál `archive`: GitPoulpi se aktualizuje na místě) do
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

## Správcem balíčků

Každý kanál níže zapisuje vedle programů stejnou značku `.gitpoulpi-dist` s vlastním názvem kanálu. GitPoulpi ji čte, aby věděl, kdo vlastní aktualizace: vlastní je správce balíčků, takže GitPoulpi nikdy nevyměňuje soubory za jeho zády. Panel aktualizací (klikněte na verzi v záhlaví okna) pak spustí vlastní příkaz správce balíčků, nebo ho ukáže, abyste ho mohli zkopírovat. Viz [Aktualizace](updates.md).

| Kanál | Instalace | Poznámky |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Instaluje pro všechny uživatele, takže aktualizace žádají o schválení správcem. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Přidejte `--scope user` nebo `--scope machine` pro volbu; rozsah machine žádá o schválení správcem při instalaci i při každé aktualizaci. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` pak `scoop install gitpoulpi` | Pro uživatele, bez práv správce. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Tap je `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | viz [repozitář apt](#apt-debian-a-ubuntu) | Podepsaný repozitář; `.deb` je také přiložen ke každému vydání. |
| **MSI** (Windows, spravované) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Pro správce: všichni uživatelé, žádné aktualizace v aplikaci. Viz [Podniková distribuce](enterprise-distribution.md#msi-s-intune-nebo-group-policy). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (nebo `paru -S gitpoulpi-bin`) | Bez pomocníka: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, pak `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Stáhne archiv vydání místo kompilace. |
| **cargo, ze zdrojových kódů** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Kompiluje; potřebuje toolchain Rustu. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Viz [Běh v Dockeru](docker.md). Stejný obraz je na `ghcr.io/poulpi-lab/gitpoulpi`. |

Názvy výše jsou ty, do kterých proces vydání publikuje (viz [Vydávání](../../developer/releasing.md)): kanál, který ještě není aktivní, odpoví "not found" a archiv vydání nebo instalační program funguje vždy.

Platí vlastní zacházení správce balíčků s PATH: Chocolatey, winget, Scoop a Homebrew dávají programy do PATH samy a žádný z nich nevytváří alias `gp` (spusťte `gitpoulpi alias create gp`, nebo Nastavení > Obecné > Spouštěč).

### apt (Debian a Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Repozitář podporuje `amd64` a `arm64` a je podepsán klíčem, který patří projektu. Programy jdou do `/usr/lib/gitpoulpi` s odkazy v `/usr/bin`. Balíček závisí na `git` a doporučuje `git-lfs`. Bez repozitáře nainstalujte `.deb` vydání pomocí `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi vám pak řekne, abyste si další `.deb` stáhli sami, protože nabízí příkazy `apt` jen tehdy, když najde repozitář v `/etc/apt/sources.list.d/gitpoulpi.*`.

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

Záhlaví okna ukazuje běžící verzi (`v1.0.0`) mezi názvem aplikace a cestou k repozitáři. Když je známo novější vydání (nebo probíhá aktualizace), kliknutí na ni otevře panel aktualizací s verzí, způsobem instalace GitPoulpi a tím, co s novým vydáním dělat; jinak kliknutí otevře kartu Poznámky k vydání. `Ctrl+N` a pak `u`, nebo **Aktualizace: Zobrazit aktualizaci** v paletě příkazů (`Ctrl+P`), panel aktualizací otevře vždy, a **Nápověda: Poznámky k vydání** vždy otevře seznam změn.

Jednou denně na pozadí GitPoulpi zkontroluje místo, kde se zveřejňují jeho vydání (ve výchozím stavu GitHub), zda existuje novější (`updates.mode`, [Nastavení > Synchronizace](settings.md#synchronizace)). Když ano, čip hlásí `v0.2.0 k dispozici` v barvě zvýraznění.

Co panel dělá, závisí na způsobu instalace GitPoulpi: archiv vydání nebo instalační program pro Windows nahradí vlastní soubory, správce balíčků spustí vlastní příkaz, spravovaná instalace nebo instalace v kontejneru říká, jak se k ní aktualizace dostávají. Sestavení vytvořené pomocí `cargo build` se nikdy neaktualizuje. Úplný popis, s návratem, `auto` oproti `notify` a tím, co může správce zamknout, je v [Aktualizacích](updates.md).

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
