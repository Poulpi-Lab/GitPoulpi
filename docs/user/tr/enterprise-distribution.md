[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · **Türkçe** · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Kurumsal dağıtım

GitPoulpi'yi kendi sunucularından birçok bilgisayara dağıtan yöneticiler için: iç paket kaynakları, yansılar, makine genelinde bir güncelleme politikası ve varsayılan bir yapılandırma. Önce [Kurulum](installation.md) ve [Güncellemeler](updates.md) sayfalarını okuyun.

Buradaki her şeyi iki olgu belirler:

- Her kurulum, programların yanındaki `.gitpoulpi-dist` işaretinde **nasıl kurulduğunu** kaydeder (`channel=...`). GitPoulpi kendini yalnızca o kanalın izin verdiği şekilde günceller: bir paket yöneticisinin sahip olduğu dosyaları asla değiştirmez ve bir MSI'a asla dokunmaz.
- **Politika** (bir dosya ve Windows'ta kayıt defteri) sizindir. Bir kullanıcının kendi `config.toml` dosyası onu asla gevşetemez.

## Chocolatey, özel bir kaynakla

Herkese açık paket yükleyiciyi GitHub'dan indirir. Kapalı bir ağ için onu içselleştirin:

1. Paketi alın (lisanslı Chocolatey araçlarıyla `choco download gitpoulpi --internalize`, ya da `.nupkg` dosyasını indirip açın).
2. `tools\chocolateyinstall.ps1` içinde `url64bit` değerini `gitpoulpi-<sürüm>-x86_64-pc-windows-msvc-setup.exe` dosyasının kendi kopyanıza yöneltin ve `checksum64` değerini sürümün `.sha256` dosyasının ilk alanına (veya kopyanızın sağlama toplamına) eşit tutun.
3. `choco pack`, ardından `.nupkg` dosyasını kaynağınıza gönderin (Nexus, Artifactory, ProGet, bir dosya paylaşımı).

Paket parametreleriyle kurun:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parametre | Etki |
|---|---|
| `/Channel:choco-internal` | İşarete `channel=choco-internal` yazar; böylece GitPoulpi kaynağın Chocolatey topluluk kaynağı değil, sizin kaynağınız olduğunu bilir. |
| `/Feed:<url>` | Kaynak adresini kaydeder (işarette `feed=`). GitPoulpi yeni sürümler için onu denetler ve `choco upgrade gitpoulpi -y --source=<kaynak>` komutunu çalıştırır. |

Parametresiz kanal `choco`'dur ve denetim Chocolatey topluluk kaynağına gider. Paket tüm kullanıcılar için Program Files içine kurulur, `gp` takma adı olmadan.

GitPoulpi'nin güncellemesi `choco upgrade` komutunu `--params` olmadan çalıştırır, böylece dağıttığınız hiçbir şeye dokunulmaz. Bu yüzden Chocolatey ilk kurulumun paket parametrelerini hatırlamalıdır (`useRememberedArgumentsForUpgrades`, Chocolatey 2.0'dan beri varsayılan olarak açık; eski sürümlerde `choco feature enable -n useRememberedArgumentsForUpgrades`). Aksi halde yükseltme düz `choco` kanalını kaydederdi. Bir politika `check_url` değeri (aşağıda), işaret ne derse desin denetimi ve yükseltmeyi kaynağınıza yöneltir ve işaretin kaynağından önceliklidir.

**Paketteki hiçbir şey `policy.toml` veya `config.toml` yazmaz.** Dağıttığınız her şey, kurulumda ve her yükseltmede kalır. Bkz. [Varsayılan bir config.toml dağıtma](#varsayılan-bir-configtoml-dağıtma).

## Scoop, özel bir bucket ile

`gitpoulpi.json` dosyasını (herkese açık bucket'taki bildirim) barındırdığınız bir Git deposunun `bucket/` klasörüne koyun, `url` değerini sürüm zip dosyasının kopyanıza çevirin ve `hash` değerini onun sha256 değerine ayarlayın (kopyanız sürüm adlarını izliyorsa `autoupdate` kalabilir). Sonra:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

Bildirimin `post_install` adımı işareti `channel=scoop` ile yeniden yazar; GitPoulpi sonra `scoop update gitpoulpi` komutunu kendisi çalıştırır ve Scoop'un arkasından dosyaları asla değiştirmez. Scoop kullanıcı başına kurar ve yönetici hakkı gerektirmez.

## apt, bir yansıyla

Depo, HTTPS üzerinden sunulan düz bir dizin ağacıdır (`dists/`, `pool/`, imzalama anahtarının `gitpoulpi-archive-keyring.gpg` ve `.asc` dosyaları ve örnek `gitpoulpi.sources`). Onu `wget --mirror`, `rsync` veya yapıt yöneticinizle yansıtın, iç ağda sunun ve anahtarı ile kaynağı her bilgisayara kurun:

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

Dosya adını `gitpoulpi.sources` (veya `gitpoulpi.list`) olarak koruyun: GitPoulpi `apt` komutlarını yalnızca `/etc/apt/sources.list.d/gitpoulpi.*` dosyasını bulduğunda gösterir. Projenin anahtarını kullanın: yansı onunla imzalı kalır ve `Signed-By` yalnızca bu depo için o tek anahtarı sabitler. Depo her paketin son beş sürümünü tutar.

**Çevrimdışı `.deb`:** her sürüm `gitpoulpi_<sürüm>_<mimari>.deb` dosyasını ekler. Onu kopyalayın ve `sudo apt install ./gitpoulpi_<sürüm>_<mimari>.deb` çalıştırın. İşaret `channel=apt` der; kaynak dosyası yoksa GitPoulpi bir `apt` komutu göstermek yerine kullanıcıya sonraki `.deb` dosyasını indirmesini söyler.

## MSI, Intune veya Grup Politikası ile

`gitpoulpi-<sürüm>-x86_64-pc-windows-msvc.msi` tüm kullanıcılar için kurulur ve yönetilmek üzere tasarlanmıştır: GitPoulpi'nin paneli yalnızca "yöneticiniz tarafından yönetiliyor" der (sizin `message` değerinizle), bir MSI kurulumunu asla güncellemez. Yeni MSI'ı dağıtarak yükseltin; bu bir ana yükseltmedir, bu yüzden eski sürüm değiştirilir.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Özellik | Etki |
|---|---|
| `INSTALLFOLDER` | Kurulum klasörü (varsayılan `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (varsayılan) klasörü makine PATH'ine ekler, `0` eklemez. |
| `UPDATES_MODE` | Politika `mode`: `off`, `notify` veya `auto`. |
| `UPDATES_PIN` | Politika `pin`, örneğin `0.2` veya `0.2.4`. |
| `UPDATES_CHECK` | Politika `check`: `github`, `nuget`, `manifest` veya `none`. |
| `UPDATES_CHECK_URL` | Politika `check_url`. |
| `UPDATES_MESSAGE` | Güncelleme panelinde gösterilen politika `message`. |

Bir özellik, değerini `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` anahtarına yalnızca siz ayarladığınızda yazar; yani ayarlanmamış olan kayıt defterine (ve bir politika dosyasına) dokunmaz. **Intune:** MSI'ı yukarıdaki komutla bir iş kolu uygulaması veya Win32 uygulaması olarak dağıtın. **Grup Politikası:** yazılım kurulumu özellik geçiremez; bu yüzden ya bir dönüşüm (`.mst`) kullanın ya da kayıt defteri değerlerini Grup Politikası Tercihleri ile ayarlayın, aşağıya bakın.

## Homebrew, iç bir tap ile

`gitpoulpi.rb` dosyasını herkese açık tap'ten barındırdığınız bir Git deposunun `Formula/` klasörüne kopyalayın, `url` satırlarını sürüm arşivlerinin kopyalarınıza çevirin ve her `sha256` değerini güncelleyin (her arşivin yanındaki `.sha256` dosyasının ilk alanı). Sonra:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Formül işareti `channel=homebrew` ile kurar: GitPoulpi `brew upgrade gitpoulpi` komutunu kendisi çalıştırır ve bu komut sizin tap'inizi okur.

## Docker registry yansısı

İmaj adı `poulpilab/gitpoulpi` şeklindedir. Panel her zaman bu adı gösterir, bu yüzden bir yansının şeffaf olması gerekir.

- **Pull-through önbellek:** bir registry'yi Docker Hub'ın vekili olarak çalıştırın (`registry` imajı, `proxy.remoteurl: https://registry-1.docker.io` ile) ve Docker daemon'ının `daemon.json` dosyasında `"registry-mirrors": ["https://registry.example"]` ayarlayın. `docker pull poulpilab/gitpoulpi` bu durumda önbelleğinizden geçer.
- **Kopya:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, sonra o adı çalıştırın. Panel hâlâ herkese açık adı gösterir; bu, sürüme bir işaret olarak yeterlidir.

Nasıl çalıştırılacağı için bkz. [Docker'da çalıştırma](docker.md).

## Politika dosyası ve kayıt defteri

Politika, bir bilgisayarda güncellemelerin nasıl davrandığını belirler ve kullanıcılar tarafından değiştirilemez. Şuradan okunur:

| İşletim sistemi | Politika dosyası |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, ayrıca | `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` kayıt defteri anahtarı, aşağıdaki her anahtar için aynı adlarla bir `REG_SZ` değeri (Grup Politikası, Intune ve MSI özellikleri için) |

Dosyada tek bir tablo vardır:

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

| Anahtar | Değerler | Etki |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Bir üst sınır: `off` < `notify` < `auto`. Kullanıcılar daha düşük bir değer seçebilir, asla daha yüksek bir değer değil. `off` denetimi gizler ve hiçbir istek yapmaz. |
| `release_channel` | `stable`, `prerelease` | Güncelleme kanalını zorlar. |
| `pin` | `1.4` veya `1.4.2` | Sunulan en yüksek sürüm: `1.4` her `1.4.x` sürümüne izin verir, `1.4.2` en fazla `1.4.2` sürümüne izin verir. |
| `check` | `github`, `nuget`, `manifest`, `none` | "En son sürüm nedir" sorusunun sorulduğu yer. Kurulum başına varsayılan: GitHub, Chocolatey kanalları için Chocolatey kaynağı. `none` asla denetlemez. |
| `check_url` | URL | `nuget` ile: kaynak. `manifest` ile: `latest.json` adresi. `github` ile: GitHub Enterprise Server için GitHub API tabanı (`repo` ekleyin). |
| `repo` | `owner/name` | `check = "github"` ile denetlenen depo. |
| `download_base` | URL | Arşiv ve yükleyici kanalları için sürüm dosyalarının bir yansısı (aynı dosya adları). Bu sunucudan indirmelere GitHub'ınkine ek olarak izin verilir. |
| `allow_self_update` | `true`, `false` | `false`, "GitPoulpi'nin dosyalarını değiştir" ve "paket yöneticisini çalıştır" eylemlerinin her birini bir nota veya kopyalanacak bir komuta çevirir. Varsayılan `true`. |
| `message` | metin | Güncelleme panelinde gösterilir ("Yöneticinizden: ..."). |

**Öncelik**, en güçlüden başlayarak: kayıt defteri, politika dosyası, kullanıcının `config.toml` dosyasındaki `[updates]`, işaretin `feed=` değeri, yerleşik varsayılanlar. Politikanın ayarladığı bir anahtar Ayarlarda **Yöneticiniz tarafından ayarlandı** der ve orada düzenlenemez.

**Güvenli tarafa düşme:** okunamayan bir politika dosyası veya kayıt defteri değeri (bir TOML söz dizimi hatası, `mode = "sometimes"`, sürüm olmayan bir `pin`) `mode = notify` ve `allow_self_update = false` sayılır ve uygulama günlüğüne bir satır yazılır. Bir yazım hatası bir filoyu asla kendini güncellerken bırakmaz. Bilinmeyen bir anahtar bir uyarıyla yok sayılır. Windows'ta yönetici haklarını tutan adım yalnızca makine politikasını okur, kullanıcının yapılandırmasını asla.

### Örnekler

Güncellemeleri kapatın (kilitli bir imaj, hava boşluklu bir ağ):

```toml
[updates]
mode = "off"
```

Filoyu 1.4 çizgisine sabitleyin, yalnızca bildirin ve kime soracaklarını söyleyin:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

İç bir Chocolatey (NuGet) kaynağını denetleyin:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Windows'ta aynı türden bir politika kayıt defteri üzerinden (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json ve download_base: sürümün bir yansısı

Bir sürümün dosyalarını (arşivler, kurulum programı, `.sha256` dosyaları, `latest.json`, **her `.minisig` imza dosyası** ve `security.json`) adlarını koruyarak bir iç web sunucusunun tek bir klasörüne kopyalayın. `latest.json` her sürüme eklidir:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Bu durumda politika şudur:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Arşiv ve yükleyici kanalları bildirimi denetler ve `download_base` adresinden, GitHub'dakiyle aynı doğrulamayla indirir. Bir yansının yalnızca dosyaları kopyalaması gerekir; API'ye ihtiyacı yoktur.

İmzalar dosyalarla birlikte yolculuk eder, bu yüzden bir yansı onları zayıflatamaz:

- `latest.json` yalnızca `latest.json.minisig` yanında duruyorsa ve GitPoulpi'nin içine gömülü proje anahtarına karşı doğrulanıyorsa güvenilir sayılır. İmzası olmayan ya da imzalandıktan sonra düzenlenen bir bildirim reddedilir (indirmelerin denetlendiği özetler onun içindedir).
- Her arşiv, aynı sunucudan `<arşiv>.minisig` ile birlikte indirilir ve yalnızca bu imza doğrulanır ve bu dosyayı ve bu sürümü adlandırırsa kurulur. Arşivi kopyalayıp `.minisig` dosyasını kopyalamayan bir yansı, güncellemenin "imza yok" ile başarısız olmasına yol açar ve hiçbir şey değişmez.
- `security.json` (güvenlik bildirimleri, bkz. [Güncellemeler](updates.md#güvenlik-bildirimleri)) ve `security.json.minisig` bildirimin yanında aranır. Bunları kopyalamayan bir yansı basitçe hiç bildirim göstermez.

Dosyaları değiştirmeden kopyalayın: bir imza tam baytları kapsar. Güvenlik bildirimleri yeni bir derleme olmadan en son sürüm için yeniden yayımlanır, bu yüzden yansıyı her yenilediğinizde `latest.json`, `security.json` ve onların `.minisig` dosyalarını sürüm sayfasından yenileyin.

## Varsayılan bir config.toml dağıtma

GitPoulpi'nin makine genelinde bir `config.toml` dosyası yoktur: ayarlar her kullanıcının klasöründe durur (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Yeni kullanıcılara varsayılanlarınızı vermek için, kullanıcının hiç yoksa, seyrek bir `config.toml` dosyasını (yalnızca istediğiniz anahtarlar, bkz. [Yapılandırma başvurusu](config-reference.md)) o klasöre kopyalayın: Windows'ta bir oturum açma veya Intune betiği, Linux'ta `/etc/skel/.config/gitpoulpi/`, macOS'ta bir ilk oturum betiği. GitPoulpi'nin paketlerinden hiçbiri bir yapılandırmanın üzerine asla yazmaz, bu yüzden onu yönetmek size kalır. Geçerli kalması gereken güncelleme kurallarını oraya koymayın: bir kullanıcının geçersiz kılamayacağı politikayı kullanın. Taşınabilir ya da iş başına bir yapılandırma `GITPOULPI_CONFIG_DIR` ile de seçilebilir.
