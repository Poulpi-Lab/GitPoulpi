[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · **Türkçe** · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Güncellemeler

GitPoulpi daha yeni sürümleri denetler ve nasıl kurulduğuna göre ya kendi dosyalarını değiştirir ya da güncellemeyi onu kuran programa bırakır. Sizin eyleminiz olmadan hiçbir şey değiştirilmez veya yeniden başlatılmaz.

## Çip ve panel

Başlık çubuğu, çalışan sürümü (`v1.0.0`) uygulama adı ile depo yolu arasında gösterir. Daha yeni bir sürüm bilindiğinde çip vurgu renginde `v0.2.0 kullanılabilir` olur.

`Ctrl+N` ardından `u` tuşlarına basın, komut paletinden (`Ctrl+P`) **Güncellemeler: Güncellemeyi Göster** komutunu çalıştırın ya da çip `kullanılabilir` derken (veya bir güncelleme sürerken) ona tıklayın: güncelleme paneli açılır. Daha yeni bir sürüm yokken çipe tıklamak bunun yerine Sürüm notları sekmesini açar. Uygulamanın üstünde bir penceredir: `Esc`, `q`, kapatma çarpısı ya da dışına bir tıklama kapatır. Şunları gösterir:

- çalışan sürümü ve **Kurulum yöntemi**: GitPoulpi'nin bu bilgisayara nasıl geldiği (aşağıdaki her şeyi bu belirler);
- güncellemenin durumunu (denetleniyor, yüzdesiyle indiriliyor, hazır, başarısız);
- bu kurulumun yordamının düğmelerini ve varsa tam komutu;
- yöneticinizin bir ileti ayarlamışsa onun bir satırını.

Her düğmenin bir tuşu (panelin ipucu satırında yazılıdır) ve bir palet komutu vardır, bu yüzden panel fare olmadan da çalışır:

| Düğme | Tuş | Palet komutu |
|---|---|---|
| **Güncelle ve yeniden başlat** (veya **Güncellemeyi çalıştır**) | `u` | Güncellemeler: Şimdi Güncelle |
| **Komutu kopyala** | `c` | Güncellemeler: Güncelleme Komutunu Kopyala |
| **Terminalde aç** | `t` | Güncellemeler: Güncelleme Komutunu Terminalde Çalıştır |
| **Önceki sürüme dön** (`<sürüm> sürümüne dön`) | `v` | Güncellemeler: Önceki Sürüme Dön |
| **Sürüm notları** | `n` | Yardım: Sürüm Notları |
| **Bu sürümü atla** | `s` | Güncellemeler: Bu Sürümü Atla |
| **Bildirimi aç** (yalnızca bir güvenlik bildirimiyle) | `a` | Güncellemeler: Güvenlik Bildirimini Aç |
| **Şimdi denetle** | `r` | Güncellemeler: Şimdi Denetle |
| **Kapat** | `Esc` | |

Bu kurulum için geçerli olmayan bir komut, paletin içinde nedeniyle birlikte soluk gösterilir (örneğin "bu kurulum başka bir yolla güncellenir: bkz. Güncellemeler: Güncellemeyi Göster").

## Kurulum türüne göre ne olur

| Kurulum yöntemi | Neye karşı denetlenir | Panel ne yapar | Geri dönüş |
|---|---|---|---|
| Bir sürüm arşivi veya `install.sh` | GitHub sürümleri | **Güncelle ve yeniden başlat**: GitPoulpi arşivi indirir ve doğrular, kapanır, dosyalarını değiştirir ve aynı terminalde yeniden açılır. | **Önceki sürüme dön** önceki dosyaları geri koyar. |
| Windows yükleyicisi, yalnızca benim için | GitHub sürümleri | Aynısı, bir onay istemeden. Windows'ta yeni bir pencerede yeniden açılır. | **Önceki sürüme dön** geri koyar. |
| Windows yükleyicisi, tüm kullanıcılar | GitHub sürümleri | Aynısı, ama Windows yönetici onayı ister. Bilgisayardaki diğer kullanıcıların GitPoulpi oturumları kapatılır, bu da onların kaydedilmemiş girdilerini kaybettirir. Reddederseniz hiçbir şey değişmez. | **Önceki sürüme dön** aynı istemle geri koyar. |
| Chocolatey | Chocolatey topluluk kaynağı | **Güncellemeyi çalıştır** `choco upgrade gitpoulpi -y` komutunu çalıştırır (Windows yönetici onayı ister). | Panel `choco install gitpoulpi --version <önceki> --allow-downgrade -y` komutunu gösterir. |
| Chocolatey, iç kaynak | Kuruluşunuzun kaynağı | **Güncellemeyi çalıştır** `choco upgrade gitpoulpi -y --source=<kaynak>` komutunu çalıştırır. Paket parametreleri yeniden gönderilmez, bu yüzden dağıtılmış bir yapılandırmaya dokunulmaz. | Kaynaklı aynı `choco install` satırı. |
| winget | GitHub sürümleri | **Güncellemeyi çalıştır** `winget upgrade --id GitPoulpi.GitPoulpi --exact --silent` komutunu sözleşmeler kabul edilmiş olarak çalıştırır. Kurulum makine genelindeyse winget yönetici onayını kendisi ister. | Panel `winget install --id GitPoulpi.GitPoulpi --version <önceki>` komutunu gösterir. |
| Scoop | GitHub sürümleri | **Güncellemeyi çalıştır**, GitPoulpi tamamen kapandıktan sonra `scoop update gitpoulpi` komutunu çalıştırır. | **Önceki sürüme dön** `scoop reset gitpoulpi@<önceki>` komutunu çalıştırır. |
| Homebrew | GitHub sürümleri | **Güncellemeyi çalıştır** bu terminalde `brew upgrade gitpoulpi` komutunu çalıştırır. | Panel `brew install gitpoulpi@<önceki>` komutunu gösterir. |
| apt | GitHub sürümleri | Panel, **Komutu kopyala** ve **Terminalde aç** ile birlikte `sudo apt update && sudo apt install --only-upgrade gitpoulpi` komutunu gösterir. GitPoulpi sizin yerinize asla `sudo` çalıştırmaz. Bu bilgisayarda GitPoulpi apt deposu yoksa bunun yerine yeni `.deb` dosyasını indirmenizi söyler. | Panel `sudo apt install gitpoulpi=<önceki>` komutunu gösterir. |
| Bir MSI paketi | GitHub sürümleri veya yöneticinizin kaynağı | Çalıştıracak bir şey yok: kurulumu yöneticiniz yönetir ve panel onun iletisini gösterir. | Sunulmaz. |
| AUR | GitHub sürümleri | Panel `yay -S gitpoulpi-bin` komutunu gösterir (veya önce hangisini bulursa `paru`; yardımcı yoksa `git clone` ve `makepkg -si` adımlarını). | Sunulmaz. |
| cargo | GitHub sürümleri | `cargo-binstall` kuruluysa panel `cargo binstall gitpoulpi-cli` komutunu, değilse `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<sürüm> gitpoulpi-cli` komutunu gösterir. | Önceki etiketle aynı komut. |
| Bir Docker imajı | GitHub sürümleri | Çalıştıracak bir şey yok: bir imaj değiştirilir. Panel `docker pull poulpilab/gitpoulpi:<sürüm>` ve `:latest` komutlarını gösterir. Bkz. [Docker'da çalıştırma](docker.md). | Panel `docker pull poulpilab/gitpoulpi:<önceki>` komutunu gösterir. |
| Yazamadığınız bir klasördeki sürüm arşivi | GitHub sürümleri | Panel, GitPoulpi'nin orada kendini değiştiremeyeceğini söyler ve **İndir**'e yönlendirir. | Sunulmaz. |
| Bir geliştirme derlemesi (`cargo build`) | yok | Asla güncellenmez. Çip yalnızca sürümü gösterir; panel kurulum kaydı olmadığını söyler. | Sunulmaz. |

"Göster" satırları kendiliğinden hiçbir şey çalıştırmaz. **Terminalde aç**, terminalinizi komut yazılı olarak açar ve Enter'a siz basarsınız.

Bir yönetici denetimi başka yere yöneltmedikçe (iç bir kaynak, bir yansı ya da bir `latest.json`) sürümler her zaman projenin GitHub deposundan gelir, bkz. [Kurumsal dağıtım](enterprise-distribution.md). Sürüm notları, ulaşılabilen hangisiyse sürümden veya GitPoulpi'nin içine gömülü değişiklik günlüğünden gelir.

## Bildir ve otomatik

`updates.mode` ([Ayarlar > Eşitleme](settings.md#eşitleme)):

- **Bildir** (varsayılan): GitPoulpi günde bir kez arka planda denetler ve çipi gösterir. Düğmeye basana kadar hiçbir şey indirilmez.
- **Otomatik**: GitPoulpi'nin kendi dosyalarını değiştirdiği yerlerde (sürüm arşivi, Windows yükleyicisi) bir güncelleme bulur bulmaz onu arka planda indirir ve doğrular. Çip bu durumda **Güncellemek için yeniden başlat** der ve yeniden başlatma hâlâ sizin seçiminizdir. Güncellemenin sahibi bir paket yöneticisiyse **Otomatik**, **Bildir** gibi davranır.
- **Kapalı**: hiçbir istek yapılmaz ve çip yalnızca sürümü gösterir.

**Güncelleme kanalı** ayarı (`updates.channel`) **Kararlı** ile **Ön sürüm** (sürüm adayları da) arasında seçim yapar. **Şimdi denetle** ile başlattığınız bir denetim günde bir kez sınırını yok sayar. Ağ bağlantısı yokken ya da depo özelken ve saklı bir github.com token'ı yokken arka plan denetimi uygulama günlüğünde tek bir satırla sessizce başarısız olur.

## Yeniden başlatma ve sonrasında gördükleriniz

**Güncelle ve yeniden başlat** veya **Güncellemeyi çalıştır** düğmesine basmak oturumunuzu kaydeder ve pencereyi kapatır. `gitpoulpi-updater` adlı ayrı küçük bir program işi yapar; böylece GitPoulpi'nin kendi dosyaları değiştirilirken kullanımda olmaz ve bir paket yöneticisi bekleyen bir üst süreç tarafından engellenmez. Önce GitPoulpi'nin arka plan daemon'larını durdurur (kullanıcınızın diğer pencereleri ve MCP sunucuları da kapanır), güncellemeyi yapar, yeni `gitpoulpi --version` çıktısının beklenen sürüm olduğunu denetler ve GitPoulpi'yi yeniden başlatır: Linux ve macOS'ta aynı terminalde, Windows'ta yeni bir pencerede. Denetim başarısız olursa eski dosyaları yerine koyar.

Sonraki başlatma durum çubuğunda bir satır gösterir:

- `v1.4.2 sürümüne güncellendi`
- `Güncelleme başarısız: <neden>. Önceki sürüm hâlâ kurulu.`
- `Güncelleme iptal edildi: yönetici onayı reddedildi. Hiçbir şey değişmedi.`

Aynı anda yalnızca bir güncelleme çalışır; GitPoulpi'nin veri klasöründeki bir kilit dosyası (yazılabilirse kurulum klasöründeki de) ikincisini dışarıda tutar ve on dakikadan eski bir kilit yok sayılır, bu yüzden çöken bir güncelleme GitPoulpi'yi bir daha kilitlemez. Taze bir kilit varken CLI ve daemon başlamaz.

## Önceki sürüme dönme

GitPoulpi'nin kendi dosyalarını değiştirdiği yerlerde **Önceki sürüme dön** önceki dosyaları geri koyar ve yeniden başlatır. Yeni sürüm başladıktan sonra bir dakika boyunca `<ad>.old` olarak kalırlar, sonra silinirler; yani geri dönüş bir güncellemeden sonraki ilk çalıştırma içindir. Kurulumun sahibi bir paket yöneticisiyse geri dönüş tablodaki komuttur: panel onu gösterir veya çalıştırır. Geri dönmek bir depoyu değil, GitPoulpi'nin kendi dosyalarını değiştirir, bu yüzden git geri alma günlüğünün parçası değildir.

## Nelerin denetlendiği ve nelerin reddedildiği

Bir şey değiştirilmeden önce GitPoulpi arşivi, `.minisig` imzasını ve `.sha256` dosyasını indirir. Önce imzayı, GitPoulpi'nin içine gömülü proje sürüm anahtarına karşı denetler ve imzanın güvenilen yorumu bu arşivi ve bu sürümü adlandırmalıdır. Sonra özeti denetler. Şunları reddeder: `.minisig` dosyası olmayan bir sürüm (sürümlerin imzalanmasından önce yayımlanmış eski bir sürüm güncelleyici üzerinden kurulamaz; buna sabitleme veya ona geri dönme de dahildir), başka bir anahtarın imzası, başka bir dosya veya başka bir sürüm için imza, `.sha256` dosyası olmayan bir sürüm, eşleşmeyen bir özet, adı bu platforma ait olmayan bir arşiv (`gitpoulpi-<sürüm>-<hedef>.<zip|tar.gz>`), işareti başka bir sürümü veya platformu adlandıran bir arşiv ve GitHub'ın sürüm sunucularının (her yönlendirme adımı denetlenir) ya da bir yöneticinin yapılandırdığı yansının dışındaki bir indirme bağlantısı. Değiştirme ya hep ya hiçtir: başka bir program çalıştırılabilir dosyalardan birini tutuyorsa GitPoulpi dosyayı adlandırır, hiçbir şeyi değiştirmez ve indirmeyi yeni bir deneme için bırakır. Windows'ta tüm kullanıcılar için bir güncelleme, `.minisig` ve `.sha256` dosyalarını kendisi indirip ikisini de yeniden denetleyen yükseltilmiş adım tarafından yeniden doğrulanır; böylece araya girip kullanıcının indirme klasörüne konan bir dosya reddedilir. Paket yöneticileri (Chocolatey, winget, Scoop, Homebrew, apt, AUR) kendi paketlerini doğrular: orada GitPoulpi hiçbir şey indirmez.

## Güvenlik bildirimleri

Proje her sürümün yanında `security.json` dosyasını yayımlar: güvenlik bildirimlerinin listesi (bir tanımlayıcı, bir önem derecesi, `>=1.2.0, <1.4.3` gibi bir aralık olarak etkilenen sürümler, düzeltilen sürüm, bir bağlantı ve tek cümlelik bir özet). Yeni sürümü arayan aynı günlük denetim onu da, sürümle aynı yerden getirir: GitHub için deponun sürüm sayfası, bir yansı için `latest.json` klasörü, herkese açık Chocolatey kaynağı için GitHub. Bir bildirim yalnızca `security.json.minisig` proje anahtarına karşı doğrulanırsa kullanılır; imzasız veya kötü imzalı bir dosya asla gösterilmez (Etkinlik Günlüğü'ndeki bir satır onun yok sayıldığını söyler).

Bir bildirim çalışan sürümü adlandırdığında çip kırmızıya döner ve **Güvenlik bildirimi: bu sürüm etkileniyor** der; paket yöneticisiyle yönetilenler dahil her kurulumda. Güncelleme paneli bu durumda her bildirimi önem derecesi, özeti, düzeltilen sürümü ve bağlantısıyla listeler; **Bildirimi aç** (`a`, ya da paletteki **Güncellemeler: Güvenlik Bildirimini Aç**) en ciddi olanın bağlantısını tarayıcıda açar. Kurulumun sahibi bir paket yöneticisiyse panel "Paket yöneticinizle güncelleyin." satırını ekler.

Güncelleme denetimi kapalıyken (`updates.mode = "off"`), `check` değeri `none` olan bir yönetici politikasıyla ve `cargo build` ile yapılmış bir derlemede hiçbir şey getirilmez ve hiçbir bildirim gösterilmez. Bir kuruluşun kendi NuGet kaynağı GitPoulpi'nin GitHub'a ulaşmasına yol açmaz: bildirimleri yalnızca politikanın `download_base` yansısından gelir.

## Yöneticinin kilitlediği ayarlar

Bir yönetici makine genelindeki bir politikada güncelleme kipini, kanalı, sabitlenmiş bir sürümü, nereden denetleneceğini ve GitPoulpi'nin kendini güncelleyip güncelleyemeyeceğini belirleyebilir. Ayarlarda bu satırlar **Yöneticiniz tarafından ayarlandı** der, panel onların iletisini gösterir ve kendini güncelleme kapatıldığında panel, dosya değiştiren düğmeler yerine komutları veya bir not sunar. Okunamayan politika dosyaları güvenli tarafa düşer: yalnızca bildirim, kendini güncelleme yok. Yöneticinin tarafı [Kurumsal dağıtım](enterprise-distribution.md#politika-dosyası-ve-kayıt-defteri) sayfasındadır.

## Ortam

`GITPOULPI_UPDATE_API` denetimi başka bir API köküne yöneltir (testlerde kullanılır).
