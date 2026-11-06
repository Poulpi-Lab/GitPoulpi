[English](../enterprise-distribution.md) · **Čeština** · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Podniková distribuce

Pro správce, kteří nasazují GitPoulpi na mnoho počítačů z vlastních serverů: interní zdroje balíčků, zrcadla, politika aktualizací platná pro celý počítač a výchozí konfigurace. Nejprve si přečtěte [Instalaci](installation.md) a [Aktualizace](updates.md).

Všechno tady se opírá o dvě skutečnosti:

- Každá instalace zaznamenává **jak byla nainstalována** do značky `.gitpoulpi-dist` vedle programů (`channel=...`). GitPoulpi se aktualizuje jen tak, jak jeho kanál dovoluje: nikdy nenahrazuje soubory, které vlastní správce balíčků, a MSI se nikdy nedotkne.
- **Politika** (soubor a ve Windows registr) je vaše. Vlastní `config.toml` uživatele ji nikdy nemůže uvolnit.

## Chocolatey se soukromým zdrojem

Veřejný balíček stahuje instalační program z GitHubu. Pro uzavřenou síť ho internalizujte:

1. Získejte balíček (`choco download gitpoulpi --internalize` s licencovaným nástrojem Chocolatey, nebo stáhněte `.nupkg` a rozbalte ho).
2. V `tools\chocolateyinstall.ps1` nasměrujte `url64bit` na vlastní kopii `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` a ponechte `checksum64` rovné prvnímu poli souboru `.sha256` vydání (nebo kontrolnímu součtu vaší kopie).
3. `choco pack`, pak nahrajte `.nupkg` do svého zdroje (Nexus, Artifactory, ProGet, sdílená složka).

Nainstalujte ho s parametry balíčku:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parametr | Účinek |
|---|---|
| `/Channel:choco-internal` | Zapíše `channel=choco-internal` do značky, takže GitPoulpi ví, že zdroj je váš a ne komunitní zdroj Chocolatey. |
| `/Feed:<url>` | Zapíše adresu zdroje (`feed=` ve značce). GitPoulpi v něm hledá nové verze a spouští `choco upgrade gitpoulpi -y --source=<feed>`. |

Bez parametrů je kanál `choco` a kontrola jde na komunitní zdroj Chocolatey. Balíček se instaluje pro všechny uživatele do Program Files, bez aliasu `gp`.

Aktualizace GitPoulpi spouští `choco upgrade` bez `--params`, aby se nic z toho, co jste nasadili, nedotklo. Chocolatey si proto musí pamatovat parametry balíčku z první instalace (`useRememberedArgumentsForUpgrades`, ve výchozím stavu zapnuto od Chocolatey 2.0; ve starších verzích `choco feature enable -n useRememberedArgumentsForUpgrades`). Jinak by upgrade zapsal prostý kanál `choco`. `check_url` v politice (níže) udrží kontrolu i upgrade nasměrované na váš zdroj bez ohledu na to, co říká značka, a má přednost před zdrojem ve značce.

**Nic v balíčku nezapisuje `policy.toml` ani `config.toml`.** Co jste nasadili, zůstává, při instalaci i při každém upgradu. Viz [Nasazení výchozího config.toml](#nasazení-výchozího-configtoml).

## winget s interním zdrojem

winget aktualizuje GitPoulpi příkazem `winget upgrade --id GitPoulpi.GitPoulpi --exact` ze zdroje, ze kterého balíček přišel. Pro interní zdroj zprovozněte zdroj REST Windows Package Manager (libovolná implementace REST API wingetu), publikujte tři manifesty vydání (`GitPoulpi.GitPoulpi.yaml`, `...installer.yaml`, `...locale.en-US.yaml`) s `InstallerUrl` ukazujícím na vaši kopii instalačního programu a odpovídajícím `InstallerSha256`, pak:

```
winget source add --name corp --arg https://winget.example/api --type Microsoft.Rest
winget install --id GitPoulpi.GitPoulpi --source corp
```

Ponechte přepínače instalačního programu z manifestu: `/CHANNEL=winget` způsobí, že značka říká `winget`, a `/ALIAS=` vynechá alias `gp`. winget GitPoulpi neřekne, co je nové, takže kontrola se na verzi stále ptá GitHubu. V síti bez GitHubu ji nasměrujte na vlastní `latest.json` politikou: `check = "manifest"` (viz [latest.json](#latestjson-a-download_base-zrcadlo-vydání)).

## Scoop se soukromým bucketem

Vložte `gitpoulpi.json` (manifest z veřejného bucketu) do `bucket/` repozitáře Git, který hostujete, změňte `url` na vlastní kopii zip archivu vydání a nastavte `hash` na jeho sha256 (`autoupdate` může zůstat, pokud vaše kopie sleduje názvy vydání). Pak:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

`post_install` manifestu přepíše značku na `channel=scoop`; GitPoulpi pak sám spouští `scoop update gitpoulpi` a nikdy nevyměňuje soubory za zády Scoopu. Scoop instaluje pro uživatele a nepotřebuje práva správce.

## apt se zrcadlem

Repozitář je prostý adresářový strom (`dists/`, `pool/`, `gitpoulpi-archive-keyring.gpg` a `.asc` podpisového klíče a ukázkový `gitpoulpi.sources`) servírovaný přes HTTPS. Zrcadlete ho pomocí `wget --mirror`, `rsync` nebo svého správce artefaktů, servírujte ho interně a na každý počítač nainstalujte klíč a zdroj:

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`:

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

Ponechte název souboru `gitpoulpi.sources` (nebo `gitpoulpi.list`): GitPoulpi ukazuje příkazy `apt` jen tehdy, když najde `/etc/apt/sources.list.d/gitpoulpi.*`. Použijte klíč projektu: zrcadlo zůstane jím podepsané a `Signed-By` připne právě tento klíč jen pro tento repozitář. Repozitář uchovává posledních pět verzí každého balíčku.

**Offline `.deb`:** každé vydání přikládá `gitpoulpi_<version>_<arch>.deb`. Zkopírujte ho a `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. Značka říká `channel=apt`; bez souboru zdroje GitPoulpi uživateli řekne, aby si místo příkazu `apt` stáhl další `.deb`.

## MSI s Intune nebo Group Policy

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` instaluje pro všechny uživatele a je určen ke správě: panel GitPoulpi jen řekne "spravuje váš správce" (s vaší `message`), instalaci MSI nikdy neaktualizuje. Upgradujte nasazením nového MSI; je to hlavní upgrade, takže stará verze se nahradí.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Vlastnost | Účinek |
|---|---|
| `INSTALLFOLDER` | Složka instalace (výchozí `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (výchozí) přidá složku do PATH počítače, `0` ne. |
| `UPDATES_MODE` | `mode` politiky: `off`, `notify` nebo `auto`. |
| `UPDATES_PIN` | `pin` politiky, například `0.2` nebo `0.2.4`. |
| `UPDATES_CHECK` | `check` politiky: `github`, `nuget`, `manifest` nebo `none`. |
| `UPDATES_CHECK_URL` | `check_url` politiky. |
| `UPDATES_MESSAGE` | `message` politiky, zobrazená v panelu aktualizací. |

Vlastnost zapíše svou hodnotu do `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` jen tehdy, když ji nastavíte, takže nenastavená nechává registr (a soubor politiky) na pokoji. **Intune:** nasaďte MSI jako line-of-business nebo aplikaci Win32 s příkazem výše. **Group Policy:** instalace softwaru nemůže předávat vlastnosti, takže použijte transformaci (`.mst`) nebo nastavte hodnoty registru pomocí Group Policy Preferences, viz níže.

## Homebrew s interním tapem

Zkopírujte `gitpoulpi.rb` z veřejného tapu do `Formula/` repozitáře Git, který hostujete, změňte řádky `url` na vlastní kopie archivů vydání a aktualizujte každé `sha256` (první pole souboru `.sha256` vedle každého archivu). Pak:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Formule nainstaluje značku s `channel=homebrew`: GitPoulpi sám spouští `brew upgrade gitpoulpi`, který čte váš tap.

## Zrcadlo registru Dockeru

Název obrazu je `poulpilab/gitpoulpi` (a `ghcr.io/poulpi-lab/gitpoulpi`). Panel vždy ukazuje tento název, takže zrcadlo musí být průhledné.

- **Pull-through cache:** spusťte registr jako proxy Docker Hubu (obraz `registry` s `proxy.remoteurl: https://registry-1.docker.io`) a nastavte `"registry-mirrors": ["https://registry.example"]` v `daemon.json` daemona Dockeru. `docker pull poulpilab/gitpoulpi` pak půjde přes vaši cache.
- **Kopie:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, pak spusťte tento název. Panel stále ukazuje veřejný, což je v pořádku jako odkaz na verzi.

Jak ho spustit, viz [Běh v Dockeru](docker.md).

## Soubor politiky a registr

Politika určuje, jak se aktualizace na počítači chovají, a uživatelé ji nemohou měnit. Čte se z:

| OS | Soubor politiky |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, také | klíč registru `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, jedna hodnota `REG_SZ` na každý klíč níže, se stejnými názvy (pro Group Policy, Intune a vlastnosti MSI) |

Soubor má jednu tabulku:

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
```

| Klíč | Hodnoty | Účinek |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Strop: `off` < `notify` < `auto`. Uživatelé mohou zvolit nižší hodnotu, nikdy vyšší. `off` skryje kontrolu a neodešle žádný požadavek. |
| `release_channel` | `stable`, `prerelease` | Vynutí kanál aktualizací. |
| `pin` | `1.4` nebo `1.4.2` | Nejvyšší nabízená verze: `1.4` povolí libovolnou `1.4.x`, `1.4.2` povolí nejvýše `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Kde se zjišťuje "jaké je nejnovější vydání". Výchozí podle instalace: GitHub, u kanálů Chocolatey zdroj Chocolatey. `none` nikdy nekontroluje. |
| `check_url` | URL | S `nuget`: zdroj. S `manifest`: adresa `latest.json`. S `github`: základ API GitHubu, pro GitHub Enterprise Server (přidejte `repo`). |
| `repo` | `owner/name` | Repozitář kontrolovaný s `check = "github"`. |
| `download_base` | URL | Zrcadlo souborů vydání (stejné názvy souborů) pro kanály archivů a instalačních programů. Stahování z tohoto hostitele je povoleno navíc k hostitelům GitHubu. |
| `allow_self_update` | `true`, `false` | `false` změní každou akci "nahradit soubory GitPoulpi" a "spustit správce balíčků" na poznámku nebo příkaz ke zkopírování. Výchozí `true`. |
| `message` | text | Zobrazí se v panelu aktualizací ("Od vašeho správce: ..."). |

**Přednost**, od nejsilnější: registr, soubor politiky, `[updates]` v uživatelově `config.toml`, `feed=` ve značce, vestavěné výchozí hodnoty. Klíč, který politika nastaví, je v Nastavení **Nastaveno vaším správcem** a nelze ho tam upravit.

**Selhání na bezpečnou stranu:** soubor politiky nebo hodnota registru, které nelze přečíst (syntaktická chyba TOML, `mode = "sometimes"`, `pin`, který není verze), se berou jako `mode = notify` a `allow_self_update = false` a jeden řádek jde do protokolu aplikace. Překlep tak nikdy nenechá flotilu aktualizovat se samu. Neznámý klíč se ignoruje s varováním. Ve Windows krok s právy správce čte jen politiku počítače, nikdy konfiguraci uživatele.

### Příklady

Vypnutí aktualizací (zamčený obraz, síť odpojená od internetu):

```toml
[updates]
mode = "off"
```

Připnutí flotily na řadu 1.4, jen oznamování a uvedení, koho se zeptat:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Kontrola interního zdroje Chocolatey (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Stejný druh politiky ve Windows přes registr (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json a download_base: zrcadlo vydání

Zkopírujte soubory vydání (archivy, instalační program, soubory `.sha256`, `latest.json`, **každý soubor podpisu `.minisig`** a `security.json`) do jedné složky interního webového serveru a ponechte názvy. `latest.json` je přiložen ke každému vydání:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Pak je politika:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Kanály archivů a instalačních programů kontrolují manifest a stahují z `download_base`, se stejným ověřováním jako z GitHubu. Zrcadlo musí jen kopírovat soubory; nepotřebuje žádné API.

Podpisy putují se soubory, takže je zrcadlo nemůže oslabit:

- `latest.json` je důvěryhodný jen tehdy, když vedle něj leží `latest.json.minisig` a ověří se proti klíči projektu zabudovanému v GitPoulpi. Manifest bez podpisu, nebo upravený po podepsání, se odmítne (hashe v něm jsou to, proti čemu se stahování kontroluje).
- Každý archiv se stahuje spolu s `<archive>.minisig` ze stejného hostitele a instaluje se jen tehdy, když se tento podpis ověří a jmenuje tento soubor a tuto verzi. Zrcadlo, které archiv zkopírovalo, ale ne jeho `.minisig`, způsobí selhání aktualizace s "no signature" a nic se nezmění.
- `security.json` (bezpečnostní upozornění, viz [Aktualizace](updates.md#bezpečnostní-upozornění)) a `security.json.minisig` se hledají vedle manifestu. Zrcadlo, které je nezkopíruje, prostě žádná upozornění nezobrazí.

Kopírujte soubory beze změny: podpis pokrývá přesné bajty. Bezpečnostní upozornění se pro poslední vydání zveřejňují znovu bez nového sestavení, takže `latest.json`, `security.json` a jejich soubory `.minisig` obnovte ze stránky vydání pokaždé, když obnovujete zrcadlo.

## Nasazení výchozího config.toml

GitPoulpi nemá `config.toml` platný pro celý počítač: nastavení žijí ve složce každého uživatele (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Chcete-li novým uživatelům dát své výchozí hodnoty, zkopírujte řídký `config.toml` (jen klíče, které chcete, viz [Reference konfigurace](config-reference.md)) do této složky, když ho uživatel nemá: skript při přihlášení nebo Intune ve Windows, `/etc/skel/.config/gitpoulpi/` v Linuxu, skript prvního přihlášení v macOS. Nic v balíčcích GitPoulpi nikdy konfiguraci nepřepíše, takže je vaše ke správě. Nedávejte tam pravidla aktualizací, když musí platit: použijte politiku, kterou uživatel nemůže přepsat. Přenosnou konfiguraci nebo konfiguraci pro jednotlivou úlohu lze také zvolit pomocí `GITPOULPI_CONFIG_DIR`.
