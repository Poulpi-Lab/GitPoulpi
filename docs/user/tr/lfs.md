[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · **Türkçe** · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS ("Large File Storage") büyük dosyaları (tasarım dosyaları, videolar, veri kümeleri) bir deponun geçmişinin dışında tutar: Git bunun yerine üç satırlık bir *işaretçi* metni saklar; gerçek içerik LFS sunucusunda ve `.git/lfs` altındaki yerel bir önbellekte durur.

GitPoulpi bunların hiçbiri için hesap gerektirmez. LFS, uzak deponuzun zaten kullandığı kimlik bilgilerini kullanır (Git kimlik bilgisi yardımcınız ya da ssh-agent).

## Hangi işlemler `git-lfs` programını gerektirir, hangileri gerektirmez

GitPoulpi, dosya dönüştüren veya taşıyan her şey için (install, track, fetch, pull, push, kilitler) standart `git-lfs` programını çalıştırır.

LFS *durumunu* okumak için program gerekmez: bir deponun LFS kullanıp kullanmadığı, hangi dosyaların işaretçi olduğu, ne kadar büyük oldukları ve gerçek nesnenin yerelde zaten saklı olup olmadığı.

| | `git-lfs` gerekir |
|---|---|
| `gitpoulpi lfs status` (programı başarısız olmak yerine eksik olarak bildirir) | hayır |
| `gitpoulpi lfs files` | hayır |
| install / track / untrack / prune | evet |
| fetch / pull / push | evet |
| kilitler | evet |

## `git-lfs` kurulumu

- **Windows**: Git for Windows ile birlikte gelir; ya da `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Diğer her şey: https://git-lfs.com

3.0 veya daha yeni bir sürüm gerekir. Kurduktan sonra GitPoulpi bunu bir dakika içinde fark eder (ya da `lfs status` ile hemen).

## `git-lfs` eksik olduğunda

GitPoulpi bu yüzden asla çökmez ya da çıkmaz:

- `gitpoulpi lfs status` başarılı olur ve kurulum bağlantısıyla birlikte `git-lfs: not installed` der.
- Programı gerektiren bir işlem, kurulum bağlantısıyla (`https://git-lfs.com`) biten açık bir mesajla ve sıfırdan farklı bir çıkış durumuyla başarısız olur; MCP üzerinden araç bu mesajı döndürür.
- LFS kullanan bir depo yine de açılır; program kurulana kadar büyük dosyaları yalnızca işaretçi olarak görünür.

## LFS dosyalarına bakmak

```
gitpoulpi lfs status            # program, desenler, filtreler, kanca
gitpoulpi lfs files             # indeksteki işaretçi dosyaları; boyut ve yerel / indirilmedi
gitpoulpi lfs files --at v1.0   # ... bir revizyondaki hâli
gitpoulpi show big.bin --at HEAD  # işaretçi metni yerine "Git LFS object, 12.3 MB, sha256:..." bildirir
```

## Bir depoyu hazırlamak: install, track, untrack

```
gitpoulpi lfs install                 # BU depo için filtreler + kancalar (git lfs install --local)
gitpoulpi lfs install --user          # bunun yerine genel git yapılandırmanıza (asla sessizce yapılmaz)
gitpoulpi lfs track '*.psd'           # *.psd dosyalarını LFS üzerinden yönlendir
gitpoulpi lfs track '*.psd' --lockable   # ... ve kilitlenene kadar salt okunur yap
gitpoulpi lfs track '*.bin' --renormalize  # ... ve zaten commit'lenmiş *.bin dosyalarını işaretçi olarak yeniden stage'le
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # yerel depodan eski, zaten push edilmiş nesneleri sil
```

- `install` varsayılan olarak deponun kendi `.git/config` dosyasına ve kancalarına yazar; böylece `--user` vermedikçe genel git yapılandırmanıza asla dokunulmaz.
- `track` ve `untrack` `.gitattributes` dosyasını düzenler ve **stage'lenmemiş** bırakır: kendiniz commit ettiğiniz sıradan bir çalışma ağacı değişikliği olarak görünür. İzlemeden *sonra* eklenen dosyalar, sonraki commit'leriyle işaretçi olarak saklanır.
- Zaten düz içerik olarak **commit edilmiş** dosyalar geçmişte düz kalır. `--renormalize` onları yeniden stage'ler; böylece sonraki commit'iniz onların yerine işaretçileri koyar. Bunları geçmişteki her commit'te de taşımak için geçmişi yeniden yazma kuralı *Dosyaları LFS'ye taşı*'yı kullanın (bkz. [Geçmişi yeniden yazma](history-rewrite.md#dosyaları-lfsye-veya-lfsden-taşımak)); o GitPoulpi'nin kendi kuralıdır, `git lfs migrate` değildir.
- Kabuğunuzun genişletmemesi için deseni tırnak içine alın.
- MCP üzerinden bunlar sıradan değişiklik işlemleridir (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`) ve `config.toml` içindeki `[mcp]` kapılamasına tabidir.

## Nesneleri taşımak: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # nesneleri yerel depoya indir; çalışma ağacına dokunulmaz
gitpoulpi lfs pull  [--remote origin]           # fetch, sonra işaretçi dosyalarını gerçek içerikle değiştir
gitpoulpi lfs push  [--remote origin] [--all]   # geçerli dalın nesnelerini yükle (--all ile her ref'inkini)
```

- Uzak depo, `fetch` ve `push` gibi, deponun varsayılan uzak deposudur.
- Kimlik bilgileri uzak deponuzun olağan olanlarıdır (kimlik bilgisi yardımcısı, ssh-agent). GitPoulpi asla kendine ait bir hesap istemez.
- `pull`, eksikse bu depo için LFS filtrelerini kurar: onlar olmadan git-lfs nesneleri indirir ama işaretçilerin yerine koymayı atlar ve sonraki bir checkout işaretçileri geri koyardı.
- MCP, `lfs push`'ı `push` gibi ele alır: yıkıcı bir araçtır (uzak depoyu değiştirir).
- Pencere, bir aktarımın ilerlemesini (dosyalar ve baytlar) durum çubuğunda gösterir. CLI sonucu bekler.

### İçeriği checkout etmek: `lfs checkout`

```
gitpoulpi lfs checkout [<yol>...]    # çevrimdışı: yerel depodan gerçek içeriği işaretçi dosyalarının üzerine yaz
```

Asla indirmez (o `lfs pull`'dır); yerel depoda eksik olan nesneler işaretçi olarak kalır. `pull` gibi, hiçbiri yapılandırılmamışsa önce deponun LFS filtrelerini kurar. Her LFS işlemi gibi geri alınamaz; bkz. [Geri al ve yinele](undo-redo.md). MCP üzerinden `lfs_checkout`'tur (bir yazma, yıkıcı değil).

### Araç çubuğunun LFS düğmesi ve pull / push

Araç çubuğunda **Pop**'tan sonra bir **`LFS ▾`** düğmesi vardır (`git.lfs_enabled` kapalıysa ve LFS'nin başlatılmadığı, `.gitattributes` dosyasının da onu kullanmadığı bir depoda gizlidir: bkz. [Ayarlar > LFS](settings.md#depoya-özel-lfs)). Bir tıklama açılır bir liste açar: **Fetch: tüm LFS dosyaları** (`lfs fetch --all`), **Checkout: tüm LFS dosyaları** (`lfs checkout`), **Pull: tüm LFS dosyaları** (`lfs pull`), **Push: tüm LFS dosyaları** (`lfs push --all`, her ref'in nesneleri) ve **Yerel LFS deposunu temizle** (`lfs prune`). Her satır aynı zamanda bir palet komutudur (`LFS: Fetch (tüm nesneler)`, `LFS: Checkout (tüm dosyalar)`, `LFS: Nesneleri Pull Et`, `LFS: Push (tüm nesneler)`, `LFS: Yerel Nesneleri Prune Et`). `LFS: Menüyü Aç` açılır listeyi açar. Her birinin bir `Ctrl+G` akoru vardır: menü için `l`, ardından satırlar için `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` ve `Shift+V`. Bunları *Ayarlar > Klavye ve fare* içinde değiştirebilirsiniz. `git-lfs` eksikse her satır kurulum iletişim kutusuna götürür.

![Araç çubuğunun LFS açılır listesi: tüm LFS dosyalarını fetch, checkout, pull ve push etme ve yerel depoyu temizleme](../images/lfs-menu.png)

**Pull ve push, LFS'yi kendileri halleder.** LFS kullanan, `git-lfs` kurulu bir depoda sade bir `pull` önce deponun LFS filtrelerinin kurulu olduğunu denetler ve sade bir `push` `pre-push` kancasını denetler. Biri eksikse GitPoulpi aktarımdan önce `git lfs install --local` çalıştırır (yalnızca bu depo) ve durum çubuğu *LFS filtreleri kuruldu* der. Kanca olmadan bir push hiçbir LFS nesnesi yüklemezdi. `git.lfs_autofix_filters = false` ile kapatın (*Ayarlar > Git ve Kodlama > Pull ve Push öncesinde LFS filtrelerini kur*). Sade bir `push` yine yalnızca push'lanan dalın nesnelerini yükler, asla her ref'inkini değil; prune asla örtük değildir.

## Dosya kilitleri

Bazı dosyalar merge edilemez (bir Photoshop dosyası, bir seviye, ikili bir varlık). Git LFS *kilitleri*, bir kişinin "bunu düzenliyorum" demesini sağlar: kilit açılana kadar başka kimse onu kilitleyemez ya da ona değişiklik push'layamaz.

```
gitpoulpi lfs locks  [--remote origin]              # hangi dosyayı kim tutuyor (sizinkiler işaretli)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Kilitleme **LFS sunucusunun** bir özelliğidir (GitHub, GitLab ve Gitea'da vardır). Uzak deponun kendi kimlik bilgileriyle erişilir; GitPoulpi'nin kendine ait bir hesabı yoktur.
- Kilitlemesi olmayan bir sunucu `lock` komutunu bunu söyleyen bir mesajla başarısız kılar (türü `LfsLockingUnsupported`). Böyle bir sunucuda `locks` hiçbir şey listelemez; git-lfs'nin kendisi de bunu bildirir.
- Bir deseni `--lockable` ile izlemek (`lfs track '*.psd' --lockable`) o dosyaları siz kilitleyene kadar salt okunur yapar.
- Başkasının kilidinin `unlock` işlemi reddedilir; `--force` onu kırar, MCP'nin `lfs_unlock`'ı yıkıcı bir araç sayması bu yüzdendir. `lfs_locks` salt okunurdur.

## Bir LFS deposunu klonlamak

`gitpoulpi clone <url>` (ve TUI'nin Klonla iletişim kutusu) depoyu LFS filtrelerini çalıştırmadan checkout eder, ardından depo LFS kullanıyorsa ve `git-lfs` kuruluysa nesneleri `git lfs pull` ile indirir (hiçbiri yapılandırılmamışsa o depo için filtreleri kurarak) ve bunu kendi aşaması olarak gösterir, *LFS nesneleri indiriliyor*. `--no-lfs` bu adımı atlar ve işaretçileri bırakır.

Bir klon **asla LFS yüzünden başarısız olmaz**: `git-lfs` eksikse ya da bir indirme başarısız olursa klon yine başarılı olur ve büyük dosyalar işaretçi olarak kalır; deponun LFS durumu nedenini söyler ve sorun giderildiğinde `gitpoulpi lfs pull` işi bitirir.

## Terminal arayüzünde

- **`LFS` etiketi.** Git LFS ile saklanan bir dosyanın adından sonra, Durum panelinin Unstaged ve Staged listelerinde ve bir commit'in değişen dosya listesinde küçük bir `LFS` etiketi bulunur.
- **Diff kartı.** Bir LFS dosyasının diff'ini açmak işaretçi metni yerine bir kart gösterir: `Git LFS nesnesi`, `1.2 MB -> 1.4 MB`, eski ve yeni kısa oid'ler ve yeni nesnenin yerelde saklı olup olmadığı ya da `indirilmedi`. Ham işaretçi diff'i için `r`, geri dönmek için yine `r` tuşuna basın. Bir LFS dosyası için yalnızca dosya düzeyinde stage uygulanır (stage'lenecek hunk veya satır yoktur).
- Bir LFS dosyasının **Dosya Görünümü** `Git LFS nesnesi, 12.3 MB, sha256:ab12cd34...` der ve içeriğin nereden alınacağını söyler.
- **Banner.** `git-lfs` eksik veya 3.0'dan eskiyken LFS kullanan bir depo için Durum panelinde bir banner bunu söyler. https://git-lfs.com adresini açmak için ona tıklayın (ya da *LFS: Kurulum Sayfasını Aç* palet girişini kullanın). Başka hiçbir şey değişmez: pencere açılır, diff'ler çalışır, büyük dosyalar yalnızca işaretçi olarak görünür.
- **Dosya menüsü ve palet.** Durum panelinde değişen bir dosyaya sağ tıklayın: düz bir dosya için *LFS ile izle: bu dosya* / *bu uzantı*; kilidini kimin tuttuğuna göre bir LFS dosyası için *Dosyayı kilitle (LFS)*, *Dosyanın kilidini aç (LFS)* veya *Kilidi zorla aç (LFS, başkasının kilidi)*. Kilitli bir dosyanın satırı `siz tarafından kilitli` / `<ad> tarafından kilitli` der. Her şey aynı zamanda bir palet komutudur (`Ctrl+P`, `LFS` yazın): başlatma, bir deseni izleme veya izlemeyi bırakma, nesneleri fetch, pull veya push etme, prune, kilitleri gösterme, kurulum sayfasını açma.
- **Başarısız bir eylem kendini açıklar.** Bir LFS eylemi `git-lfs` eksik olduğu için başarısız olursa bir iletişim kutusu mesajı ve bir *Kurulum sayfasını aç* düğmesini taşır.
- **Ayarlar.** *Ayarlar > LFS* (depo başına) LFS'nin başlatılıp başlatılmadığını gösterir, izlenen desenleri listeler ve **LFS'yi burada kur** düğmesini içerir. *Git ve Kodlama* (depo başına), **Git LFS** anahtarını, salt okunur bir **git-lfs programı** satırını ve bir **Bu depo için LFS'yi başlat** düğmesini içerir.
- **Git LFS** kapalıyken (`git.lfs_enabled = false`, bkz. [yapılandırma başvurusu](config-reference.md)) bunların hepsi gizlenir. Git'in kendi LFS filtreleri yine çalışır.
- LFS durumu, depo açıldıktan kısa süre sonra, her değişiklikten sonra, elle yenilemede, `.gitattributes` değiştiğinde ve LFS kullanan bir depo için her on saniyede bir okunur.

## Bilinen sınırlar

- LFS desenleri yalnızca *iç içe* bir `.gitattributes` dosyasında (kökte değil, `.git/info/attributes` içinde değil) bulunan bir depo, LFS kullandığı yalnızca bir LFS nesne deposuna (`.git/lfs`) sahip olduğunda algılanır, ondan önce değil.
- `git lfs migrate` kullanılmaz. Dosyaları geçmiş boyunca LFS'ye taşımak veya LFS'den çıkarmak, geçmişi yeniden yazmanın *Dosyaları LFS'ye taşı* / *Dosyaları LFS'den geri getir* kurallarıdır: yerel klon üzerinde çalışırlar, yalnızca kökteki `.gitattributes` yazılır ve nesneler sonrasında yine gönderilmelidir (sonuç panelindeki *LFS nesnelerini gönder* veya `lfs push --all`).
