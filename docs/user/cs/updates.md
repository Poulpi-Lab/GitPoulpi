[English](../updates.md) · **Čeština** · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Aktualizace

GitPoulpi kontroluje, zda existují novější vydání, a podle způsobu instalace buď nahradí vlastní soubory, nebo předá aktualizaci programu, který ho nainstaloval. Nic se nikdy nenahradí ani nerestartuje bez vaší akce.

## Čip a panel

Záhlaví okna ukazuje běžící verzi (`v1.0.0`) mezi názvem aplikace a cestou k repozitáři. Když je známo novější vydání, čip se změní na `v0.2.0 k dispozici` v barvě zvýraznění.

Stiskněte `Ctrl+N` a pak `u`, spusťte **Aktualizace: Zobrazit aktualizaci** z palety příkazů (`Ctrl+P`), nebo klikněte na čip, dokud říká `k dispozici` (nebo dokud probíhá aktualizace): otevře se panel aktualizací. Když žádné novější vydání neexistuje, kliknutí na čip místo toho otevře kartu Poznámky k vydání. Je to okno nad aplikací: zavře ho `Esc`, `q`, zavírací křížek nebo klik mimo něj. Ukazuje:

- běžící verzi a **Nainstalováno přes**: jak se GitPoulpi dostal do tohoto počítače (rozhoduje o všem níže);
- v jakém stavu aktualizace je (kontrola, stahování s procenty, připravena, selhala);
- tlačítka postupu pro tuto instalaci a přesný příkaz, pokud existuje;
- řádek od vašeho správce, pokud nějaký nastavil.

Každé tlačítko má klávesu (vypsanou v nápovědném řádku panelu) a příkaz v paletě, takže panel funguje i bez myši:

| Tlačítko | Klávesa | Příkaz v paletě |
|---|---|---|
| **Aktualizovat a restartovat** (nebo **Spustit aktualizaci**) | `u` | Aktualizace: Aktualizovat nyní |
| **Kopírovat příkaz** | `c` | Aktualizace: Kopírovat příkaz aktualizace |
| **Otevřít v terminálu** | `t` | Aktualizace: Spustit příkaz aktualizace v terminálu |
| **Vrátit se** | `v` | Aktualizace: Vrátit se k předchozí verzi |
| **Poznámky k vydání** | `n` | Nápověda: Poznámky k vydání |
| **Přeskočit tuto verzi** | `s` | Aktualizace: Přeskočit tuto verzi |
| **Otevřít upozornění** (jen s bezpečnostním upozorněním) | `a` | Aktualizace: Otevřít bezpečnostní upozornění |
| **Zkontrolovat nyní** | `r` | Aktualizace: Zkontrolovat aktualizace nyní |
| **Zavřít** | `Esc` | |

Příkaz, který pro tuto instalaci neplatí, je v paletě zašedlý s uvedením důvodu (například "tato instalace se aktualizuje jinak: viz Aktualizace: Zobrazit aktualizaci").

## Co se děje podle instalace

| Nainstalováno přes | Kontrola proti | Co dělá panel | Návrat |
|---|---|---|---|
| Archiv vydání, nebo `install.sh` | Vydání na GitHubu | **Aktualizovat a restartovat**: GitPoulpi stáhne a ověří archiv, zavře se, nahradí své soubory a znovu se otevře ve stejném terminálu. | **Vrátit se** vymění předchozí soubory zpět. |
| Instalační program pro Windows, jen pro mě | Vydání na GitHubu | Totéž, bez dotazu. Ve Windows se znovu otevře v novém okně. | **Vrátit se** vymění zpět. |
| Instalační program pro Windows, všichni uživatelé | Vydání na GitHubu | Totéž, ale Windows požádá o schválení správcem. Relace GitPoulpi ostatních uživatelů na počítači se zavřou, čímž se ztratí jejich nedokončený vstup. Pokud odmítnete, nic se nezmění. | **Vrátit se** vymění zpět, se stejným dotazem. |
| Chocolatey | Komunitní zdroj Chocolatey | **Spustit aktualizaci** spustí `choco upgrade gitpoulpi -y` (Windows požádá o schválení správcem). | Panel ukáže `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, interní zdroj | Zdroj vaší organizace | **Spustit aktualizaci** spustí `choco upgrade gitpoulpi -y --source=<feed>`. Parametry balíčku se neposílají znovu, takže nasazená konfigurace zůstane nedotčená. | Stejný řádek `choco install` se zdrojem. |
| winget | Vydání na GitHubu | **Spustit aktualizaci** spustí `winget upgrade --id GitPoulpi.GitPoulpi --exact --silent` s přijatými podmínkami. winget si o schválení správcem řekne sám, když je instalace pro celý počítač. | Panel ukáže `winget install --id GitPoulpi.GitPoulpi --version <previous>`. |
| Scoop | Vydání na GitHubu | **Spustit aktualizaci** spustí `scoop update gitpoulpi` poté, co se GitPoulpi úplně zavře. | **Vrátit se** spustí `scoop reset gitpoulpi@<previous>`. |
| Homebrew | Vydání na GitHubu | **Spustit aktualizaci** spustí `brew upgrade gitpoulpi` v tomto terminálu. | Panel ukáže `brew install gitpoulpi@<previous>`. |
| apt | Vydání na GitHubu | Panel ukáže `sudo apt update && sudo apt install --only-upgrade gitpoulpi` s tlačítky **Kopírovat příkaz** a **Otevřít v terminálu**. GitPoulpi za vás nikdy nespustí `sudo`. Bez repozitáře apt GitPoulpi na tomto počítači řekne, abyste místo toho stáhli nový `.deb`. | Panel ukáže `sudo apt install gitpoulpi=<previous>`. |
| Balíček MSI | Vydání na GitHubu, nebo zdroj vašeho správce | Není co spouštět: instalaci spravuje váš správce, jehož zprávu panel ukáže. | Nenabízí se. |
| AUR | Vydání na GitHubu | Panel ukáže `yay -S gitpoulpi-bin` (nebo `paru`, podle toho, co najde dřív; bez pomocníka kroky `git clone` a `makepkg -si`). | Nenabízí se. |
| cargo | Vydání na GitHubu | Panel ukáže `cargo binstall gitpoulpi-cli`, když je nainstalováno `cargo-binstall`, jinak `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | Stejný příkaz s předchozím tagem. |
| Obraz Dockeru | Vydání na GitHubu | Není co spouštět: obraz se nahrazuje. Panel ukáže `docker pull poulpilab/gitpoulpi:<version>` a `:latest`. Viz [Běh v Dockeru](docker.md). | Panel ukáže `docker pull poulpilab/gitpoulpi:<previous>`. |
| Archiv vydání ve složce, do které nemůžete zapisovat | Vydání na GitHubu | Panel řekne, že se tam GitPoulpi nemůže nahradit, a odkáže na **Stáhnout**. | Nenabízí se. |
| Vývojové sestavení (`cargo build`) | žádná | Nikdy se neaktualizuje. Čip ukazuje verzi a nic dalšího; panel řekne, že neexistuje záznam o instalaci. | Nenabízí se. |

Řádky, které jen ukazují příkaz, sami nikdy nic nespustí. **Otevřít v terminálu** otevře váš terminál s napsaným příkazem a vy stisknete Enter.

Vydání vždy pocházejí z repozitáře projektu na GitHubu, pokud správce nasměruje kontrolu jinam (interní zdroj, zrcadlo nebo `latest.json`), viz [Podniková distribuce](enterprise-distribution.md). Poznámky k vydání pocházejí z vydání nebo z changelogu zabudovaného v GitPoulpi, podle toho, co je dosažitelné.

## Oznamovat a automaticky

`updates.mode` ([Nastavení > Synchronizace](settings.md#synchronizace)):

- **Oznamovat** (výchozí): jednou denně GitPoulpi na pozadí zkontroluje novinky a ukáže čip. Nic se nestahuje, dokud nestisknete tlačítko.
- **Automaticky**: tam, kde GitPoulpi nahrazuje vlastní soubory (archiv vydání, instalační program pro Windows), také aktualizaci stáhne a ověří na pozadí, jakmile ji najde. Čip pak hlásí **Restartovat pro aktualizaci** a restart je stále na vás. Tam, kde aktualizaci vlastní správce balíčků, se **Automaticky** chová jako **Oznamovat**.
- **Vypnuto**: nikdy se neodešle žádný požadavek a čip ukazuje jen verzi.

Nastavení **Kanál aktualizací** (`updates.channel`) volí mezi **Stabilní** a **Předběžný** (i kandidáti na vydání). Kontrola, kterou spustíte tlačítkem **Zkontrolovat nyní**, ignoruje limit jednou denně. Bez připojení k síti, nebo dokud je repozitář soukromý a není uložen žádný token github.com, kontrola na pozadí tiše selže s jedním řádkem v protokolu aplikace.

## Restart a co uvidíte potom

Stisknutí **Aktualizovat a restartovat** nebo **Spustit aktualizaci** uloží vaši relaci a zavře okno. Práci dělá samostatný malý program `gitpoulpi-updater`, takže soubory GitPoulpi nejsou při nahrazování používány a správce balíčků neblokuje čekající rodič. Nejprve zastaví démony GitPoulpi na pozadí (zavřou se i ostatní okna a servery MCP vašeho uživatele), provede aktualizaci, ověří, že nový `gitpoulpi --version` vypíše očekávanou verzi, a znovu spustí GitPoulpi: v Linuxu a macOS ve stejném terminálu, ve Windows v novém okně. Pokud ověření selže, vrátí staré soubory zpět.

Další spuštění ukáže jeden řádek ve stavovém řádku:

- `Aktualizováno na v1.4.2`
- `Aktualizace selhala: <důvod>. Předchozí verze je stále nainstalována.`
- `Aktualizace zrušena: schválení správcem bylo zamítnuto. Nic se nezměnilo.`

Najednou běží jen jedna aktualizace; zamykací soubor ve složce dat GitPoulpi (a jeden ve složce instalace, pokud je zapisovatelná) brání druhé, a zámek starší než deset minut se ignoruje, takže spadlá aktualizace nikdy GitPoulpi trvale nezamkne. CLI a daemon se nespustí, dokud existuje čerstvý zámek.

## Návrat

Tam, kde GitPoulpi nahrazuje vlastní soubory, **Vrátit se** vymění předchozí soubory zpět a restartuje. Zůstanou jako `<name>.old` ještě minutu po spuštění nové verze, pak se smažou, takže návrat je určen pro první spuštění po aktualizaci. Tam, kde instalaci vlastní správce balíčků, je návrat příkaz z tabulky: panel ho ukáže nebo spustí. Návrat mění vlastní soubory GitPoulpi, ne repozitář, takže není součástí žurnálu zpět gitu.

## Co se kontroluje a odmítá

Před nahrazením čehokoli GitPoulpi stáhne archiv, jeho podpis `.minisig` a jeho `.sha256`. Nejprve ověří podpis proti klíči vydání projektu zabudovanému v GitPoulpi a důvěryhodný komentář podpisu musí jmenovat tento archiv a tuto verzi. Pak ověří hash. Odmítne: vydání bez `.minisig` (starší vydání zveřejněné před podepisováním vydání nelze nainstalovat přes aktualizátor, což zahrnuje i připnutí na ně nebo návrat k němu), podpis jiným klíčem, podpis jiného souboru nebo jiné verze, vydání bez `.sha256`, hash, který nesedí, archiv, jehož název není ten pro tuto platformu (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), archiv, jehož značka jmenuje jinou verzi nebo platformu, a odkaz ke stažení mimo hostitele vydání GitHubu (kontroluje se každý přesměrovací krok) nebo mimo zrcadlo nakonfigurované správcem. Výměna je vše nebo nic: pokud některý ze spustitelných souborů drží jiný program, GitPoulpi soubor pojmenuje, nic nezmění a stažené ponechá pro další pokus. Ve Windows se aktualizace pro všechny uživatele znovu ověřuje povýšeným krokem, který si `.minisig` a `.sha256` stáhne sám a obojí zkontroluje znovu, takže soubor vyměněný mezitím ve složce stahování uživatele se odmítne. Správci balíčků (Chocolatey, winget, Scoop, Homebrew, apt, AUR) ověřují své balíčky sami: GitPoulpi tam nestahuje nic.

## Bezpečnostní upozornění

Vedle každého vydání projekt zveřejňuje `security.json`: seznam bezpečnostních upozornění (identifikátor, závažnost, postižené verze jako rozsah například `>=1.2.0, <1.4.3`, opravená verze, odkaz a shrnutí v jedné větě). Stejná denní kontrola, která hledá novou verzi, ho také stáhne, ze stejného místa jako vydání: ze stránky vydání repozitáře pro GitHub, ze složky `latest.json` pro zrcadlo, z GitHubu pro veřejný zdroj Chocolatey. Upozornění se použije jen tehdy, když se `security.json.minisig` ověří proti klíči projektu; nepodepsaný nebo špatně podepsaný soubor se nikdy nezobrazí (jeden řádek v Protokolu aktivity řekne, že byl ignorován).

Když upozornění jmenuje běžící verzi, čip zčervená a hlásí **Bezpečnostní upozornění: tato verze je postižena**, při každé instalaci, včetně těch spravovaných správcem balíčků. Panel aktualizací pak vypíše každé upozornění se závažností, shrnutím, opravenou verzí a odkazem; **Otevřít upozornění** (`a`, nebo **Aktualizace: Otevřít bezpečnostní upozornění** v paletě) otevře v prohlížeči odkaz nejzávažnějšího z nich. Tam, kde instalaci vlastní správce balíčků, panel přidá "Aktualizujte pomocí správce balíčků."

S vypnutou kontrolou aktualizací (`updates.mode = "off"`), s politikou správce, jejíž `check` je `none`, a u sestavení vytvořeného pomocí `cargo build` se nic nestahuje a žádné upozornění se nezobrazí. Vlastní zdroj NuGet organizace nezpůsobí, že GitPoulpi kontaktuje GitHub: jeho upozornění pocházejí jen ze zrcadla `download_base` v politice.

## Nastavení zamknutá správcem

Správce může v politice platné pro celý počítač nastavit režim aktualizací, kanál, připnutou verzi, kde kontrolovat a zda se GitPoulpi smí vůbec sám aktualizovat. V Nastavení tyto řádky ukazují **Nastaveno vaším správcem**, panel ukazuje jeho zprávu, a když je samoaktualizace vypnutá, panel místo tlačítek nahrazujících soubory nabídne příkazy nebo poznámku. Nečitelné soubory politiky selhávají na bezpečnou stranu: jen oznamování, žádná samoaktualizace. Strana správce je v [Podnikové distribuci](enterprise-distribution.md#soubor-politiky-a-registr).

## Prostředí

`GITPOULPI_UPDATE_API` nasměruje kontrolu na jiný kořen API (používá ho testy).
