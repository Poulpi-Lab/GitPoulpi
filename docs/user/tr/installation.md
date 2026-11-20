[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · **Türkçe** · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Kurulum

GitPoulpi tek bir programdır: `gitpoulpi`. Alt komutsuz çalıştırıldığında terminal arayüzünü açar; bir alt komutla komut satırı arayüzüdür (betikler için `--json`); `gitpoulpi mcp serve` YZ ajanları için MCP sunucusudur (bkz. [MCP ve ajanlar](mcp-and-agents.md)). Yanında arşiv, bir kurulumu güncelleyen ayrı küçük program `gitpoulpi-updater`'ı içerir (bkz. [Güncellemeler](updates.md)).

Uyumluluk için arşiv ayrıca yalnızca `gitpoulpi` programını başlatan üç küçük aktarıcı içerir: `gitpoulpi-tui` (arayüzü açar; sonraki bir sürümde kaldırılacaktır), `gitpoulpi-mcp` (MCP sunucusunu başlatır; tüm 1.x serisi boyunca kalır, böylece mevcut ajan yapılandırmaları çalışmaya devam eder) ve `gitpoulpi-launch` (kısayolların zaten açık bir GitPoulpi penceresini göstermek için çalıştırdığı program, `gitpoulpi launch`; Windows'ta konsol penceresi yoktur, bu yüzden tıklama hiç pencere yanıp söndürmez). Hepsi birlikte kurulur, güncellenir ve kaldırılır.

## Nelere ihtiyacınız var

- `PATH` üzerinde bir **sistem `git`**'i. GitPoulpi işlemlerin çoğu için libgit2,
  geri kalanı için sizin `git`'inizi kullanır.
  **2.34 veya daha yeni** bir sürüm gerekir. macOS'ta Apple'ın Command Line Tools ile gelen
  `git` de, Homebrew'un `git`'i de çalışır.
- Fare desteği olan bir **terminal**. kitty, Ghostty, WezTerm, Rio, iTerm2 (ve sixel üzerinden Windows Terminal, foot ile Tabby) commit
  grafiğini resim olarak çizer. Diğer terminaller (Windows konsolu, zellij, screen ve
  çoğu başkası) karakterlerden oluşan bir grafik gösterir. Bkz.
  [Terminal seçmek](terminals.md) ve [Çoklayıcılar](multiplexers.md).
- Araç çubuğundaki ve kenar çubuğundaki simgeler için bir **Nerd Font** ya da
  emoji içeren bir yazı tipi.
- İsteğe bağlı: Git LFS depoları için 3.0 veya daha yeni bir **`git-lfs`**. Onsuz
  LFS dosyaları küçük işaretçi dosyaları olarak görünür ve GitPoulpi nedenini
  söyler. Nasıl kurulacağı için [Git LFS](lfs.md) sayfasına bakın.
- İsteğe bağlı: barındırma token'larını ve YZ API anahtarlarını saklamak için bir
  **kimlik bilgisi yardımcısı** (Git Credential Manager, libsecret, `pass`,
  Keychain). Bkz. [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md).

## Bir sürüm arşivinden

Bir sürüm arşivi `gitpoulpi-<sürüm>-<hedef>.zip` (Windows) veya `.tar.gz`
(Linux, macOS) dosyasıdır ve yanında bir `.sha256` dosyası bulunur. Arşivi
`.sha256` dosyasına göre doğrulayın, açın ve çalıştırılabilir dosyalarını
(`gitpoulpi`, `gitpoulpi-updater` ve aktarıcılar `gitpoulpi-tui`, `gitpoulpi-mcp` ile `gitpoulpi-launch`)
`PATH` üzerindeki bir klasöre koyun (`gitpoulpi-updater` ve `.gitpoulpi-dist` işaretini
onların yanında tutun: işaret `channel=archive` der, böylece derleme kendini güncelleyebilir,
bkz. [Güncellemeler](updates.md)). Arşivdeki `INSTALL.txt` aynı adımları verir.

### Bir indirmeyi doğrulayın

Her sürüm dosyası projenin [minisign](https://jedisct1.github.io/minisign/) anahtarıyla imzalanır ve GitPoulpi bir güncellemeyi kurmadan önce bu imzayı kendisi denetler. Elle indirdiğiniz bir dosya için yanındaki `.minisig` imzadır. Açık anahtar (anahtar kimliği `B461FFCBF6F6A4F2`) depoda `packaging/minisign.pub` olarak commit'lidir:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Komutun yazdırdığı güvenilen yorumu okuyun: `file:<dosyanın adı> version:<beklediğiniz sürüm>` olmalıdır. Geçerli ama başka bir dosyayı ya da başka bir sürümü adlandıran bir imza, sizin dosyanız için bir imza değildir.

Sürüm, yapı kaynağı kanıtlarıyla (build provenance attestation) derlenmişse (deponun `ATTESTATIONS` değişkeni), GitHub CLI bir dosyanın nerede derlendiğini denetler:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

`.sha256` dosyası yalnızca bozuk bir indirmeyi yakalar. Dosyanın projeye ait olduğunu gösteren şey imzadır.

## Bir yükleyiciyle

Her sürüm, işletim sistemi başına bir yükleyiciyle de gelir. İkisi de aynı seçimleri
(kısayollar, PATH, `gp` gibi bir komut takma adı) kurulu `gitpoulpi` programının kendisini
çağırarak yapar; bu kod Ayarlar > Başlatıcı'nın arkasındaki koddur, yani yükleyicinin
kurduğu ile Ayarlar'ın gösterdiği her zaman aynıdır.

### Windows: `gitpoulpi-<sürüm>-x86_64-pc-windows-msvc-setup.exe`

İlk iletişim kutusu kurulumun kimin için olduğunu sorar:

| Seçim | Nereye | Yönetici hakları | Güncellemeler için anlamı |
|---|---|---|---|
| **Yalnızca benim için** (varsayılan) | `%LOCALAPPDATA%\Programs\GitPoulpi` | yok | GitPoulpi kendi dosyalarını değiştirir: [güncelleme panelinde](updates.md) **Güncelle ve yeniden başlat**, onay istemi yok. |
| **Tüm kullanıcılar için** | `Program Files\GitPoulpi` | kurulumda bir onay istemi | Her güncelleme yönetici onayı ister (bir Windows istemi) ve bilgisayardaki diğer kullanıcıların GitPoulpi oturumları önce kapatılır. |

Aynı seçim sessizce de çalışır: `gitpoulpi-<sürüm>-...-setup.exe /CURRENTUSER` veya `/ALLUSERS`.

Onay kutuları: Başlat menüsü kısayolu, Masaüstü kısayolu, PATH'e ekle. Bir sayfa **komut takma adını** sorar: varsayılan `gp`, istediğiniz herhangi bir ad, boş bırakırsanız takma ad yok (komut satırında `/ALIAS=ad` veya `/ALIAS=`). PowerShell `gp` adını zaten `Get-ItemProperty` için kullanır; orada başka bir ad seçin ya da `gitpoulpi` komutunu kullanın.

Yükleyici çalıştırılabilir dosyaları (güncelleme sırasında dosyaları değiştiren küçük program olan `gitpoulpi-updater` dahil) ve `.gitpoulpi-dist` işaretini kopyalar. İşaret GitPoulpi'nin nasıl kurulduğunu kaydeder (`inno-user` veya `inno-machine`); güncelleme paneli bunu **Kurulum yöntemi** olarak gösterir. Kurulum programını yeniden çalıştırmak seçimlerinizi (kapsam, kısayollar, takma ad) korur.

Windows Ayarlar > Uygulamalar'dan kaldırın: bu, `gitpoulpi uninstall` komutunu (takma ad, PATH girdileri, kısayollar) çalıştırır ve dosyaları, güncelleyicinin bıraktığı `.old` dosyaları dahil, siler. Ayarlarınız ve depolarınız kalır.

### Linux ve macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

İşletim sisteminiz ve işlemcinize uygun arşivi seçer, `.sha256` dosyasını denetler,
`~/.local/opt/gitpoulpi` içine kurar (kanal `archive`: GitPoulpi kendini yerinde günceller) ve `gitpoulpi` ile `gitpoulpi-mcp` (aktarıcı) programlarını
`~/.local/bin` içine bağlar. Root gerekmez ve ana klasörünüzün dışındaki hiçbir şeye
dokunulmaz. `--yes` olmadan takma adı (varsayılan `gp`), PATH'i ve kısayolları sorar.

| Bayrak | Etki |
|---|---|
| `--prefix DIR` | Program dosyalarını `DIR` içine kurar (mutlak yol). |
| `--version X.Y.Z` | En son sürüm yerine bu sürümü kurar. |
| `--archive FILE` | Yerel bir arşiv kullanır; `FILE.sha256` yanında durmalıdır (`--no-verify` bunu atlar). Ağ kullanılmaz. |
| `--alias NAME` / `--no-alias` | `NAME` adını GitPoulpi'yi başlatan bir komut yapar (`gitpoulpi alias create`) ya da takma ad yapmaz. |
| `--no-path` | GitPoulpi'yi yeni kabukların PATH'ine eklemez (aksi halde `gitpoulpi path add` çalışır). |
| `--desktop` | Uygulama menüsü ve Masaüstü kısayollarını da oluşturur. |
| `--yes` | Hiçbir şey sormaz: bayraklar ve varsayılanlar geçerlidir (takma ad `gp`, PATH evet, kısayollar hayır). |
| `--uninstall` | `gitpoulpi uninstall` komutunu çalıştırır, sonra bağlantıları ve klasörü kaldırır. |

En son sürüm sorgusu için deponun herkese açık olması gerekir; özel bir depo için arşivi
tarayıcıda indirin ve `--archive` kullanın.

## Bir paket yöneticisiyle

Aşağıdaki her kanal, programların yanına kendi kanal adıyla aynı `.gitpoulpi-dist` işaretini yazar. GitPoulpi güncellemelerin kimde olduğunu bilmek için onu okur: paket yöneticisinde; bu yüzden GitPoulpi onun arkasından dosyaları asla değiştirmez. Güncelleme paneli (başlık çubuğundaki sürüme tıklayın) bu durumda paket yöneticisinin kendi komutunu çalıştırır ya da kopyalayabilmeniz için gösterir. Bkz. [Güncellemeler](updates.md).

| Kanal | Kurulum | Notlar |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Tüm kullanıcılar için kurar, bu yüzden güncellemeler yönetici onayı ister. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` ardından `scoop install gitpoulpi` | Kullanıcı başına, yönetici hakkı yok. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Tap `Poulpi-Lab/homebrew-gitpoulpi` adresindedir. |
| **apt** (Debian, Ubuntu) | bkz. [apt deposu](#apt-debian-ve-ubuntu) | İmzalı depo; her sürüme bir `.deb` de eklenir. |
| **MSI** (Windows, yönetilen) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Yöneticiler için: tüm kullanıcılar, uygulama içi güncelleme yok. Bkz. [Kurumsal dağıtım](enterprise-distribution.md#msi-intune-veya-grup-politikası-ile). |
| **cargo** | `cargo binstall gitpoulpi-cli` | Derlemek yerine sürüm arşivini indirir. |
| **cargo, crates.io'dan** | `cargo install --locked gitpoulpi-cli` | Derler; bir Rust araç zinciri gerekir. Alt komutsuz çalıştırıldığında terminal arayüzünü açan ve MCP'yi de sunan (`gitpoulpi mcp`) `gitpoulpi` programını kurar. |
| **cargo, kaynaktan** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Derler; bir Rust araç zinciri gerekir. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Bkz. [Docker'da çalıştırma](docker.md). |

Yukarıdaki adlar, sürüm sürecinin yayımladığı adlardır (bkz. [Sürüm yayımlama](../../developer/releasing.md)): henüz canlı olmayan bir kanal "bulunamadı" yanıtı verir ve sürüm arşivi veya yükleyici her zaman çalışır.

Paket yöneticisinin kendi PATH işleyişi geçerlidir: Chocolatey, Scoop ve Homebrew programları PATH'e kendileri koyar ve hiçbiri `gp` takma adını oluşturmaz (`gitpoulpi alias create gp` çalıştırın ya da Ayarlar > Başlatıcı).

### apt (Debian ve Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Depo `amd64` ve `arm64` mimarilerini destekler ve projeye ait bir anahtarla imzalıdır. Programlar `/usr/lib/gitpoulpi` içine gider ve `/usr/bin` içinde bağlantılar bulunur. Paket `git` paketine bağlıdır ve `git-lfs` paketini önerir. Depo olmadan, bir sürümün `.deb` dosyasını `sudo apt install ./gitpoulpi_<version>_<arch>.deb` ile kurun: GitPoulpi bu durumda bir sonraki `.deb` dosyasını kendiniz indirmenizi söyler; çünkü `apt` komutlarını yalnızca depoyu `/etc/apt/sources.list.d/gitpoulpi.*` içinde bulduğunda sunar.

## Kaynaktan

Güncel bir kararlı Rust araç zincirine ihtiyacınız var (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Çalıştırılabilir dosyalar `target/release/` içindedir. `release` profili hızlı
derlenir. En küçük ve en hızlı çalıştırılabilir dosyalar için `dist` profilini
kullanın. Derlemesi daha uzun sürer:

```
cargo build --profile dist
```

Deponun kökündeki `build-opti` betikleri bu derlemeyi çalıştırır ve çalıştırılabilir
dosyaların boyutlarını yazdırır:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd veya PowerShell)
```

Yalnızca kullandığınız dilleri ve özellikleri derlemek için
[Özel derlemeler](custom-builds.md) sayfasına bakın.

## Kendi sürüm arşivinizi oluşturun

```
cargo run -p xtask -- dist
```

Bu komut `dist` profilini derler ve
`target/dist/gitpoulpi-<sürüm>-<hedef>.zip` (Windows) veya `.tar.gz` (diğer
sistemler) dosyasını yazar. Arşiv çalıştırılabilir dosyaları, `INSTALL.txt`,
`README.md`, `LICENSE` ve `CHANGELOG.md` dosyalarını içerir. Yanına bir `.sha256`
dosyası yazılır.

## Güncellemeler

Başlık çubuğu, çalışan sürümü (`v1.0.0`) uygulama adı ile depo yolu arasında gösterir. Daha yeni bir sürüm bilindiğinde (ya da bir güncelleme sürerken) ona tıklamak güncelleme panelini açar: sürümle, GitPoulpi'nin nasıl kurulduğuyla ve yeni sürüm için ne yapılacağıyla; aksi halde tıklamak Sürüm notları sekmesini açar. `Ctrl+N` ardından `u` tuşları ya da komut paletindeki (`Ctrl+P`) **Güncellemeler: Güncellemeyi Göster** komutu güncelleme panelini her zaman açar, **Yardım: Sürüm Notları** ise değişiklik günlüğünü her zaman açar.

GitPoulpi günde bir kez, arka planda, sürümlerinin yayımlandığı yerde (varsayılan olarak GitHub) daha yenisi olup olmadığını denetler (`updates.mode`, [Ayarlar > Eşitleme](settings.md#eşitleme)). Varsa çip vurgu renginde `v0.2.0 kullanılabilir` der.

Panelin ne yaptığı GitPoulpi'nin nasıl kurulduğuna bağlıdır: bir sürüm arşivi veya Windows yükleyicisi kendi dosyalarını değiştirir, bir paket yöneticisi kendi komutunu çalıştırır, yönetilen veya kapsayıcı bir kurulum güncellemelerin ona nasıl ulaştığını söyler. `cargo build` ile yapılmış bir derleme asla güncellenmez. Önceki sürüme dönme, `auto` ile `notify` farkı ve bir yöneticinin neleri kilitleyebileceğiyle tam anlatım [Güncellemeler](updates.md) sayfasındadır.

## Çalıştığını denetleyin

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi
```

`--version` sürümü ve derlemenin yapıldığı commit'i yazdırır. Ardından
[Başlarken](getting-started.md) sayfasını izleyin.

## Yan yana birkaç kurulum

Bir paket, bir geliştirme çalışma kopyası, bir cargo derlemesi ve açılmış bir arşiv aynı anda kurulu olabilir. Her biri ayrı bir kurulumdur: kendi başlatıcı adı (`GitPoulpi (Chocolatey)`, `GitPoulpi (dev)`, ...), ahtapot rengi, komut takma adları, "Şurada aç" menü girdisi ve daemon'ları vardır; böylece iki sürüm paralel çalışır ve birbirinin pencerelerini asla devralmaz. Ayarlar, depolar ve token'lar ortaktır. Siz Ayarlar > Başlatıcı bölümünde **Birincil kurulum ol** ile seçene kadar düz `GitPoulpi` başlatıcı adı kimsenin değildir. [Ayarlar](settings.md#başlatıcı) sayfası diğer kurulumları listeler ve kaybolmuş olan için bir **Unut** düğmesi sunar.

## Kaldırma

`gitpoulpi uninstall`, komut takma adlarını (Windows'ta App Paths anahtarlarıyla birlikte),
PATH girdilerini, kısayolları ve dosya yöneticisi menü girdilerini **yalnızca bu kurulumun**
kaldırır: diğerlerininkiler kalır; `PATH` de onların klasörlerini ve, hâlâ içinde takma adı
olan bir kurulum varsa, ortak takma ad klasörünü korur. Görev çubuğu ve Dock sabitlemeleri
kalır (Windows'ta dışarıdan sabitlemeyi kaldırmanın yolu yoktur) ve ayarlarınız bu kurulumun
girdisini korur; böylece aynı klasöre yeniden kurmak görünümünü geri getirir. Ardından bir
sürüm kurulumunun program dosyalarını siler; bu kısım `.gitpoulpi-dist` işareti olmayan bir
derlemede (bir cargo derlemesi) reddeder. `--keep-files` ile program dosyalarını bırakır ve
bir geliştirme çalışma kopyası dahil her derlemede çalışır. MCP ajanlarına sunulmaz.

| Nasıl kurdunuz | Başlatıcıları ve takma adları ne kaldırır |
|---|---|
| Windows yükleyicisi (kullanıcı veya tüm kullanıcılar), winget | Kaldırıcı `gitpoulpi uninstall --keep-files` komutunu çalıştırır, sonra dosyaları siler. |
| Chocolatey | Otomatik kaldırıcısı, yükleyicinin kaldırıcısını çalıştırır. |
| Scoop | `scoop uninstall gitpoulpi` önce `gitpoulpi uninstall --keep-files` komutunu çalıştırır. |
| MSI | Kaldırma, onu kurulumu yapan kullanıcı olarak çalıştırır. Başka bir hesap kaldırırsa kullanıcının başlatıcıları kalır ve diğer kurulumların Ayarlar sayfasında yetim olarak görünür (**Unut**). |
| apt, AUR, Homebrew, cargo, bir geliştirme çalışma kopyası | Sizin için hiçbir kanca çalışmaz: paketi kaldırmadan önce `gitpoulpi uninstall --keep-files` komutunu çalıştırın. |
| Arşiv, `install.sh --uninstall` | `gitpoulpi uninstall` veya `install.sh --uninstall` komutunu çalıştırın, sonra klasörü silin. |

Yükleyici kullanmadıysanız çalıştırılabilir dosyaları silin. GitPoulpi ayrıca şu dosyaları tutar:

| Ne | Nerede |
|---|---|
| Ayarlar (`config.toml`) ve oturumlar | İşletim sisteminizin yapılandırma klasörü. `GITPOULPI_CONFIG_DIR` bunu değiştirir. |
| Depo başına durum | Varsayılan olarak deponun `.git` klasörünün içindeki `gitpoulpi/` (`session.per_repo_state_dir` bunu çalışma ağacındaki `.gitpoulpi/` klasörüne ya da kullanıcı veri klasörüne taşıyabilir). |
| Önbellekler (avatarlar) | İşletim sisteminizin önbellek klasörü. |
| Token'lar ve API anahtarları | Kimlik bilgisi yardımcınız. **Ayarlar > SSH ve kimlik bilgileri > Tüm Kullanıcı Adlarını ve Parolaları Unut** bunları kaldırır. |

Tam yollar için [Yapılandırma başvurusu](config-reference.md) sayfasına bakın.
