[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · **Türkçe** · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Kurulum

GitPoulpi'nin üç çalıştırılabilir dosyası vardır:

| Çalıştırılabilir dosya | Nedir |
|---|---|
| `gitpoulpi-tui` | Terminal arayüzü. |
| `gitpoulpi` | Komut satırı arayüzü (betikler için `--json`). Alt komut verilmezse arayüzü açar. |
| `gitpoulpi-mcp` | YZ ajanları için MCP sunucusu. Bkz. [MCP ve ajanlar](mcp-and-agents.md). |

Bunların yanında arşiv, kısayolların zaten açık bir GitPoulpi penceresini göstermek için çalıştırdığı küçük bir program olan `gitpoulpi-launch`'ı içerir (`gitpoulpi launch`). Windows'ta konsol penceresi yoktur, bu yüzden tıklama hiç pencere yanıp söndürmez. Diğerleriyle birlikte kurulur, güncellenir ve kaldırılır.

## Nelere ihtiyacınız var

- `PATH` üzerinde bir **sistem `git`**'i. GitPoulpi işlemlerin çoğu için libgit2,
  geri kalanı için sizin `git`'inizi kullanır.
  **2.34 veya daha yeni** bir sürüm gerekir. macOS'ta Apple'ın Command Line Tools ile gelen
  `git` de, Homebrew'un `git`'i de çalışır.
- Fare desteği olan bir **terminal**. kitty, Ghostty, WezTerm ve iTerm2 (ve sixel üzerinden Windows Terminal ile foot) commit
  grafiğini resim olarak çizer. Diğer terminaller (Windows konsolu, tmux ve
  çoğu başkası) karakterlerden oluşan bir grafik gösterir. Bkz.
  [Terminal seçmek](terminals.md).
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
`.sha256` dosyasına göre doğrulayın, açın ve üç çalıştırılabilir dosyayı
`PATH` üzerindeki bir klasöre koyun. Arşivdeki `INSTALL.txt` aynı adımları verir.

## Bir yükleyiciyle

Her sürüm, işletim sistemi başına bir yükleyiciyle de gelir. İkisi de aynı seçimleri
(kısayollar, PATH, `gp` gibi bir komut takma adı) kurulu `gitpoulpi` programının kendisini
çağırarak yapar; bu kod Ayarlar > Genel > Başlatıcı'nın arkasındaki koddur, yani yükleyicinin
kurduğu ile Ayarlar'ın gösterdiği her zaman aynıdır.

### Windows: `gitpoulpi-<sürüm>-x86_64-pc-windows-msvc-setup.exe`

Geçerli kullanıcı için, yönetici istemi olmadan, `%LOCALAPPDATA%\Programs\GitPoulpi`
klasörüne kurar (ilk iletişim kutusu tüm kullanıcılar için kurulumu ve kurulum klasörünü
önerir). Onay kutuları: Başlat menüsü kısayolu, Masaüstü kısayolu, PATH'e ekle. Bir sayfa
**komut takma adını** sorar: varsayılan `gp`, istediğiniz herhangi bir ad, boş bırakırsanız
takma ad yok. PowerShell `gp` adını zaten `Get-ItemProperty` için kullanır; orada başka bir
ad seçin ya da `gitpoulpi` komutunu kullanın.

Yükleyici üç çalıştırılabilir dosyayı ve `.gitpoulpi-dist` işaretini kopyalar; böylece
[Şimdi güncelle](#yerinde-güncelleme) zip'te olduğu gibi çalışır. Kullanıcı başına bir kurulum
yazılabilir olduğundan kendini güncelleyebilir; Program Files içindeki tüm kullanıcılar için bir
kurulumun değiştirilmesi yükseltilmiş yetki ister, bu yüzden onu yeni kurulum programıyla yeniden
kurun. Windows Ayarlar > Uygulamalar'dan kaldırın: bu, `gitpoulpi uninstall` komutunu (takma ad,
PATH girdileri, kısayollar) çalıştırır ve dosyaları, güncelleyicinin bıraktığı `.old`
dosyaları dahil, siler. Ayarlarınız ve depolarınız kalır.

### Linux ve macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

İşletim sisteminiz ve işlemcinize uygun arşivi seçer, `.sha256` dosyasını denetler,
`~/.local/opt/gitpoulpi` içine kurar ve `gitpoulpi` ile `gitpoulpi-mcp` programlarını
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

Deponun kökündeki `build-opti` betikleri bu derlemeyi üç çalıştırılabilir dosya
için çalıştırır ve boyutlarını yazdırır:

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
sistemler) dosyasını yazar. Arşiv üç çalıştırılabilir dosyayı, `INSTALL.txt`,
`README.md`, `LICENSE` ve `CHANGELOG.md` dosyalarını içerir. Yanına bir `.sha256`
dosyası yazılır.

## Güncellemeler

Başlık çubuğu, çalışan sürümü (`v1.0.0`) uygulama adı ile depo yolu arasında gösterir. Ona tıklayın
ya da komut paletinden (`Ctrl+P`) **Yardım: Sürüm Notları** komutunu çalıştırın: değişiklik
günlüğüyle bir Sürüm notları sekmesi açılır; çalışan sürüme kaydırılmış olarak.

GitPoulpi günde bir kez, arka planda, GitHub'a daha yeni bir sürüm olup olmadığını sorar
(`updates.mode`, [Ayarlar > Eşitleme](settings.md#eşitleme)). Varsa çip vurgu renginde
`v0.2.0 kullanılabilir` der ve tıklamak, o sürümün notları en üstte olacak şekilde Sürüm notları
sekmesini açar. Sekmede her birinin bir tuşu ve bir palet komutu olan üç düğme vardır:

- **İndir** (`d`, *Güncellemeler: İndir*) sürüm sayfasını tarayıcınızda açar.
- **Bu sürümü atla** (`s`, *Güncellemeler: Bu Sürümü Atla*) daha yeni bir sürüm çıkana kadar
  bildirimi gizler.
- **Şimdi denetle** (`r`, *Güncellemeler: Şimdi Denetle*) hemen yeniden sorar.

`updates.mode = "off"` hiçbir istek yapmaz. Ağ bağlantısı yokken ya da depo özelken ve saklı bir
github.com token'ı yokken denetim sessizce başarısız olur (uygulama günlüğünde bir satır, arayüzde
hiçbir şey). `GITPOULPI_UPDATE_API` denetimi başka bir API köküne yöneltir (testlerde kullanılır).

### Yerinde güncelleme

Bir sürüm arşivinden kurulan bir derleme kendini değiştirebilir. Arşiv, çalıştırılabilir
dosyaların yanında küçük bir `.gitpoulpi-dist` dosyası taşır; çalışan bir derleme yalnızca bu
dosya çalıştırılabilir dosyasının yanında durduğunda (ve platformunu adlandırdığında)
"güncellenebilir" sayılır. `cargo` ile yapılmış bir derleme, başka yere kopyalanmış özel bir derleme
ya da dosyayı atlamış bir paket kendini asla güncellemez: Sürüm notları sekmesi bunu söyler ve
yalnızca **İndir** sunar.

Güncellenebilir bir derleme için sekme şunları ekler:

- **Şimdi güncelle** (`u`, *Güncellemeler: Şimdi Güncelle*) arşivi ve `.sha256` dosyasını arka planda
  indirir (durum çubuğunun göstergesinde bir iş; yüzdesiyle, her iş gibi iptal edilebilir),
  özeti denetler, arşivi açar ve her yeni dosyayı eskisinin yanına `<ad>.new` olarak koyar.
  Çalışan hiçbir şeye dokunulmaz. `auto` kipinde bir denetim daha yeni bir sürüm bulur bulmaz
  aynısı kendiliğinden olur.
- **Güncellemek için yeniden başlat** (`R`, *Güncellemeler: Güncellemek İçin Yeniden Başlat*;
  indirme hazırlandığında çip de aynısını der) her zaman sizin eyleminizdir, asla otomatik
  değildir. Oturumun bağımsız daemon'ını durdurur (gömülü olan pencereyle birlikte gider),
  dosyaları değiştirir ve yeni sürümü aynı argümanlar ve oturumla başlatır.
- **Önceki sürüme dön** (`v`, *Güncellemeler: Önceki Sürüme Dön*) önceki dosyaları geri koyar
  ve yeniden başlatır. Önceki sürümün dosyaları, yeni sürüm bir dakika çalışana kadar `<ad>.old`
  olarak kalır, sonra silinir; yani düğme o ilk çalıştırma için vardır. git geri alma
  geçmişinin parçası değildir: bir depoyu değil, GitPoulpi'nin kendi dosyalarını değiştirir.

Dosyaların nasıl değiştirildiği: Linux ve macOS'ta her yeni dosya eskisinin üzerine yeniden
adlandırılır (atomik; çalışan bir program çıkana kadar eski kopyasını korur). Windows'ta çalışan
bir `.exe` üzerine yazılamaz, bu yüzden eskisi `<ad>.old` olarak yeniden adlandırılır ve yenisi
yerine taşınır; hâlâ çalışan bir `.old` sonraki bir başlatmada silinir. Değiştirme ya hep ya
hiçtir: başka bir program çalıştırılabilir dosyalardan birini tutuyorsa (örneğin
`gitpoulpi-mcp` çalıştıran bir MCP istemcisi), GitPoulpi dosyayı adlandırır, hiçbir şeyi
değiştirmez ve indirmeyi yeni bir deneme için hazır bırakır.

Reddedilenler, her biri bir iletiyle: `.sha256` dosyası olmayan bir sürüm, eşleşmeyen bir özet,
adı bu platforma ait olmayan bir arşiv dosyası (`gitpoulpi-<sürüm>-<hedef>.<zip|tar.gz>`),
işareti başka bir sürümü ya da platformu adlandıran bir arşiv ve şu sunucuların dışındaki her
indirme bağlantısı: GitHub'ın `github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` ve `github-releases.githubusercontent.com` adresleri
(ilk istek ve her yönlendirme adımı; başka bir yere yönlendirme indirmeyi durdurur). Özel bir
deponun arşivleri bir tarayıcı oturumu ister, bu yüzden indirme, depo herkese açık olana kadar
orada başarısız olur.

## Çalıştığını denetleyin

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` sürümü ve derlemenin yapıldığı commit'i yazdırır. Ardından
[Başlarken](getting-started.md) sayfasını izleyin.

## Kaldırma

`gitpoulpi uninstall` (iki yükleyicinin de çalıştırdığı komut), GitPoulpi'nin oluşturduğu
komut takma adlarını, PATH girdilerini ve kısayolları, ardından bir sürüm kurulumunun program
dosyalarını kaldırır; `.gitpoulpi-dist` işareti olmayan bir derlemede (bir cargo derlemesi)
reddeder ve `--keep-files` program dosyalarını bırakır. MCP ajanlarına sunulmaz. Yükleyici
kullanmadıysanız üç çalıştırılabilir dosyayı silin. GitPoulpi ayrıca şu dosyaları tutar:

| Ne | Nerede |
|---|---|
| Ayarlar (`config.toml`) ve oturumlar | İşletim sisteminizin yapılandırma klasörü. `GITPOULPI_CONFIG_DIR` bunu değiştirir. |
| Depo başına durum | Her deponun içindeki `.gitpoulpi/`. |
| Önbellekler (avatarlar) | İşletim sisteminizin önbellek klasörü. |
| Token'lar ve API anahtarları | Kimlik bilgisi yardımcınız. **Ayarlar > SSH ve kimlik bilgileri > Tüm Kullanıcı Adlarını ve Parolaları Unut** bunları kaldırır. |

Tam yollar için [Yapılandırma başvurusu](config-reference.md) sayfasına bakın.
