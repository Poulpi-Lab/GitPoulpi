[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · **Türkçe** · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# Radar

Radar, Ana sekmenin hemen ardından gelen kendi sekmesidir: **birkaç depo** boyunca
sizi bekleyenlerin tek listesi: yazdığınız pull (veya merge) request'ler, sizden
inceleme isteyenler, size atanan veya sizin açtığınız issue'lar ve yarım işinizi
bıraktığınız depolar. Altındaki hiçbir şey tıkı veya tuşu çalamasın diye, Radar
görünümü üstte olan Ana penceredir.

Bir kod barındırıcısı profili gerektirir (bkz. [Barındırma hizmetleri ve issue
takipçileri](hosting-integration.md); tek bir deponun listeleri [Pull request'ler ve issue'lar](pull-requests-and-issues.md) sayfasındadır ve bir satırın tartışması bir [konuşma](conversations.md) olarak açılır) ve ona bağlı depolar ya da hesap geneli listeleri açık bir profil (aşağıda
"Kapsamın ötesi" bölümüne bakın). Bağlı olmayan bir
depoya hiç sorulmaz; onun yerine bir **Bağla...** satırı gösterir (aşağıda
"Söyleyecek bir şeyi olan depolar" bölümüne bakın).

![İki depo üzerinde Radar: sabitlenmiş bir request, ardından İncelememi bekleyenler, Benim pull request'lerim, Bana atananlar, issue'lar ve süren işler](../images/radar.png)

Bir pull veya merge request satırındaki **İncele...** (veya `e`), depoyu isteğin uygulama içi incelemesiyle açar
([Bir pull veya merge request'i incelemek](review.md)). Klonladığınız ama barındırıcıya hiç bağlamadığınız bir deponun request'i için, klon önce request'in projesine bağlanır (zaten kendi pull request bağlantısı yoksa), böylece inceleme kendi sekmesinde başlar.

### Eylemler sütunu

Bir request veya issue satırının sağında, küçük düğmeler seçili satırda bir tuşun yaptığını yapar (bir tık satırı seçer ve
eylemi çalıştırır; adı ve tuşu için işaretçiyi bir düğmenin üzerinde bekletin):

```
Geniş:   #58  Round invoice totals  feature/login → main   alice   2d   İncele  [globe]  [talk]  ⊕  ×  ⇣  ⋮
Dar:     #58  Round invoice totals  feature/login → main                 [rev]  [globe]  [talk]  ⋮
Issue:   #12  Crash on empty list                         bob     5h   Başla  [globe]  [talk]  ×  ⋮
```

| Düğme | Tuş | Palet |
|---|---|---|
| **İncele** (incelemenizi bekleyen bir request; klonlanmamışsa nasıl yapılacağını sorar) | `e` | `Radar: İsteği İncele...` |
| **Başla** (klonlanmış bir issue) | `n` | `Radar: Issue Üzerinde Çalışmaya Başla...` |
| küre: sayfayı barındırıcıda aç | `o` | `Radar: Tarayıcıda Aç` |
| satırlar simgesi: konuşmayı aç | `i` | `Radar: Issue Konuşmasını Aç` |
| `»` **Hazır olarak işaretle** (bir taslak request) | `y` | `Radar: Hazır Olarak İşaretle` |
| `⊕` **Merge** (bir request) | `m` | `Radar: İsteği Merge Et...` |
| `×` **Kapat** (bir request veya bir issue) | `x` | `Radar: Kapat...` |
| `⇣` **Depoyu klonla...** (deposu klonlanmamış bir satır) | `g` | `Radar: Depoyu Klonla...` |

Eylemi olmayan bir satır hücresini boş bırakır, böylece her düğme her satırda aynı sütunda kalır. Pencere
dar olduğunda İncele ve Başla sözcükleri, başka herhangi bir sütun kaybolmadan önce simgeye dönüşür; Hazır, Merge, Kapat ve Klonla
düğmeleri başlıktan önce sütundan çıkar, ama satır menüsünde (`⋮`, sağ tık veya `Shift+F10`) kalır; orada
çalışamayan bir eylem nedeniyle birlikte gri görünür. Sütun otomatiktir (boyutlandırılabilir sütunlardan
biri değildir). Klonlanmamış bir request'e çift tık ya da `Enter`, **İncele** gibi nasıl inceleneceğini sorar.

### Bir request'in dalları

Bir request satırı başlığından sonra `kaynak → hedef` gösterir (kaynak dal bir fork'taysa `alice:feature/login → main`).
Yer kısa olduğunda önce hedef kesilir (üç nokta), başlık yerin çoğunu korur ve dar bir pencerede
dallar gösterilmez; Eylemler sütunu asla dışarı itilmez. Kesilmiş bir metnin tamamı için işaretçiyi üzerinde bekletin. Dalları
adlandırmayan bir barındırıcı orada hiçbir şey göstermez. Bir issue satırı, deposunu bir request satırıyla aynı hapta adlandırır
(klonun klasör adı veya klonlanmamışsa `owner/name`).

### Barındırıcıda işlem yapmak

**Merge**, **Kapat** ve **Hazır olarak işaretle** GitHub, GitLab, Gitea ve Forgejo üzerinde çalışır (diğer barındırıcılar bunları
"... desteklemiyor" ile gri yapar; issue'lar aynı barındırıcılarda kapatılabilir).

- **Merge** önce sorar: kutu request'i, projesini ve dallarını adlandırır ve yöntemi sunar (bir merge commit, squash ve
  barındırıcıda varsa rebase; GitLab merge eder veya squash yapar). Bir taslak, hedefiyle çakışan bir request veya başarısız
  denetimler için nedeniyle reddedilir; barındırıcının bir reddi (izin yok, dalın bir kuralı) durum satırında gösterilir.
- **Kapat** önce sorar ve merge etmeden kapatır.
- **Hazır olarak işaretle** bir taslağı taslaktan çıkarır (GitLab ve Gitea: `Draft:` / `WIP:` işareti başlıktan kalkar). Hiçbir
  şey sormaz: barındırıcıda geri alınabilir.
- **Depoyu klonla...** satırın projesinde olağan Klonla iletişim kutusunu açar (URL doldurulur, klasör seçici olağan olandır). Klon
  oluştuktan sonra Radar, uzak depoları bir sonraki okuyuşunda satırı onun üzerine yerleştirir (yeniden açın veya Yenile) ve Klonla
  bundan sonra klonun nerede olduğunu söyler.

Merge ve Kapat barındırıcıyı değiştirir ve **GitPoulpi'den geri alınamaz** (kutu bunu söyler); kapatılmış bir request'i barındırıcıda yeniden açın.
Başarılı bir çağrıdan sonra listeler yeniden istenir, böylece merge edilmiş veya kapatılmış bir satır kaybolur ve hazır olan `draft` hapını kaybeder.

## Açmak

- **`Alt+R`** (yeniden atanabilir: `global.radar`), her yerden.
- Palet: `Radar: Aç`.
- Sekme çubuğunda Ana'nın hemen arkasındaki **Radar** sekmesi.
- Sekme kapalıyken **`+`** menüsünün **Radar** satırı.

Bir depo sekmesinden bu sizi Radar açık olarak Ana pencereye götürür; `Esc`
veya **Geri** depo listesine döner ve `⌂` de aynısını yapar.

## Sekmeyi kapatmak veya Radar'ı kapatmak

Sekme, bir depo sekmesi gibi kapanır: **x**'i, orta tıklama, gösterilirken `Ctrl+W` veya `Radar: Sekmeyi Kapat` palet komutu ile. Kapatılamayan tek sekme Ana'dır. Kapatılan sekme, o oturum için yeniden başlatmalarda da kapalı kalır; Radar'ı yeniden açana kadar (`Alt+R`, `Radar: Aç` veya `+` menüsü). Sonra Ana'nın hemen arkasına geri döner. Kapatmak hiçbir ayarı değiştirmez.

Hiç kullanmıyorsanız **Ayarlar > Genel > Radar'ı etkinleştir** (`general.radar_enabled`, bkz. [yapılandırma başvurusu](config-reference.md)) ile kapatın. Kapalıyken sekme, palet komutları ve `Alt+R` ortadan kalkar ve GitPoulpi onun için hiçbir şey getirmez. Yeniden başlatmadan, hemen uygulanır.

## Neleri kapsar: kapsam

**Kapsam** açılır listesi hangi depolara sorulacağını seçer:

- ada göre bir **çalışma alanı** (depoları, sekmesi açık olmayanlar dahil);
- **Açık sekmeler**: bu oturumun sekme olarak açık depoları.
- **Varsayılan klonlama klasörü** ve **Varsayılan gözatma klasörü**: [Ayarlar](config-reference.md)'da belirlenen klasörde bulunan her depo (`repos.clone_root`, `repos.browse_root`), sekmesi olsun ya da olmasın.

Burada seçtiğiniz son kapsamda açılır (`radar.toml` içinde tutulur), o hâlâ var olduğu sürece; ilk seçiminizden
önce ya da o çalışma alanı silindiyse oturumun etkin çalışma alanında, o da yoksa açık sekmelerde açılır. Seçim
yalnızca bu görünüm içindir: pencerenin etkin çalışma alanını değiştirmez. İki deponun klasör adı aynıysa, ayırt
edebilmeniz için çipe üst klasör eklenir.

### İki klasör kapsamı

Bir klasör kapsamı depoları **iki düzey derinlikte** arar: doğrudan içindeki klasörleri ve kendileri depo olmayan
klasörlerin içindekileri (`~/src/work/app`). Gizli klasörler atlanır, bir deponun içine asla girilmez ve en fazla
200 depo listelenir. Arama arka planda çalışır, asla çizimi yapan iş parçacığında değil ve sonucu siz
yenileyene kadar tutulur: Yenile düğmesi, `r`, `F5` (pencerenin yenileme tuşu, yeniden atanabilir) veya `Radar: Yenile` klasörü yeniden arar (o zamandan beri
klonlanmış bir depo görünür).

Barındırıcı erişimi depo başına isteğe bağlı kalır: klasördeki bir barındırıcıya **bağlı olmayan** depoya hiçbir
zaman sorulmaz. Yalnızca devam eden iş satırını verir, Depolar altında **Bağla...** olan bir `bağlı değil` satırı
taşır (o deponun pull request bölümünü açar; orada sağlayıcıyı ve projeyi seçersiniz) ve durum çubuğu onları sayar
(`3 depo bağlı değil (yalnızca devam eden işler)`).

Belirlenmemiş bir klasör listede soluk görünür (`Varsayılan gözatma klasörü (Ayarlar'dan belirleyin)`); onu seçmek
Ayarlar satırını açar (Ayarlar > Genel) ve kapsamı olduğu gibi bırakır. Klonlama klasörü, o klasör var olduğu
sürece işletim sistemine göre varsayılanıyla (Klonla penceresinin başladığı klasör) belirlenmiş sayılır; gözatma
klasörünün varsayılanı yoktur. Kapsam listesi her açıldığında ayarlardan okunur, bu yüzden orada yapılan bir
değişiklik hemen uygulanır.

## Liste

| Grup | Neyi tutar |
|---|---|
| Sabitlenenler | Sabitlediğiniz her şey, türü ne olursa olsun, sabitleme sırasıyla |
| İncelememi bekleyenler | Sizden inceleme isteyen (ve sizin yazmadığınız) request'ler |
| Benim request'lerim | Sizin yazdığınız request'ler; taslak için çip `taslak` der |
| Bana atananlar | Size atanan request'ler |
| Benden bahsedenler | Sizden bahseden ve yukarıdaki bir grupta olmayan açık request'ler ve issue'lar (size atanmayan ve sizin açmadığınız bir issue burada yalnızca sizden bahsediyorsa görünür) |
| Diğer request'ler | Kalan açık request'ler |
| Bana atanan issue'lar | Size atanan açık issue'lar |
| Açtığım issue'lar | Sizin açtığınız açık issue'lar |
| Süren işler | Kapsamdaki, commit'lenmemiş değişikliği veya push edilmemiş commit'i olan depolar |
| Ertelenenler | Bir süreliğine gizlediğiniz öğeler; siz açana kadar katlı |
| Depolar | Söyleyecek bir şeyi olan depolar (çevrimdışı, yenilenecek token, bağlı değil) |

"Pull request" ve "merge request" sözcükleri olağan terminoloji ayarını ve
barındırıcıyı izler.

Her satır öğenin türünü (`⇄` request, `●` issue, `✎` süren iş), depo çipini
(çalışma alanının renginde), numarayı, başlığı, yazarı, yaşı ve yer varsa
etiketleri gösterir. Süren iş satırı dalı ve içinde bekleyenleri gösterir: `✎`
değişen dosyalar, `+` yeni dosyalar, `↑` push edilecek commit'ler, `↓` pull
edilecek commit'ler.

Sayılar her başlıkta bulunur. Bir grubun içinde en son güncellenen öğe önce gelir.

Filtrenin altındaki durum çubuğu, barındırıcılar bir andan uzun sürede yanıt
verdiğinde bir döner simge gösterir. Önbellekteki listeler (kenar çubuğunun
kullandığı önbellek) hemen görünür ve yanıtlar geldikçe değiştirilir.

### Sütunlar

Depo çipi, `#` (numara ve onay işaretleri), yazar ve yaş **boyutlandırılabilir sütunlardır**. Otomatikte bırakılırsa depo sütunu ekrandaki en uzun ad
kadar geniştir (en çok 40 hücre ve pencerenin en çok üçte biri), böylece `owner/name` ve bir hesap listesinin
`group/sub/project` adı normal bir terminalde kesilmez. Başlık kalanı alır. Sığmayan bir ad üç noktayla biter
(`group/sub...`) ve:

- Kesilmiş bir depo, başlık, yazar veya etiket listesinin **üzerinde bekleyin**, bir ipucu metnin tamamını gösterir.
  Satırı seçin, filtrenin altındaki durum çubuğu da tam projeyi ve başlığı söyler; klavye de aynısını görür.
- Sütunu boyutlandırmak için **ayırıcıyı sürükleyin** (depo adından, `#`'den sonraki hücre ya da yazardan veya yaştan hemen önceki);
  sütunu yeniden sığdırmak için ona **çift tıklayın**. Listenin bir **başlık satırı** vardır (depo, `#`,
  başlık, yazar, yaş) ve ayırıcılar Log'unkiler gibi `ui.column_resize` ayarını izler: `line` ile (varsayılan)
  başlık satırında ayırıcıda soluk bir `│` durur; imleç ayırıcının üzerindeyken (başlıkta ya da listenin herhangi bir satırında),
  sütun tuşlarla seçiliyken ya da sürüklerken tüm liste boyunca ince bir vurgu çizgisi çizilir; boyunca herhangi bir yerden
  sürüklemek yeniden boyutlandırır. Bıraktığınızda, imleç hâlâ ayırıcının üzerinde değilse çizgi kaybolur (aksi hâlde imleçle birlikte
  gider): fareyle bir sürükleme asla bir sütunu seçili bırakmaz. `header` ile işaret ya da çizgi yoktur: başlık satırında ayırıcıdan
  sonraki ilk hücre aydınlanır ve sürüklemenin başlayabileceği tek yer odur. Destekleyen terminallerde imleç bir ayırıcının üzerinde
  yeniden boyutlandırma imlecine dönüşür. Durum çipinin sabit bir genişliği vardır, yaşın da (siz boyutlandırana kadar gösterilen en geniş yaş;
  dar bir sütunun kestiği bir yaşın tam tarihi ipucunda görünür); böylece yazar her satırda aynı hücrede başlar.
- Ya da tuşları kullanın: **`C`** bir sütun seçer (yine `C` ya da `Tab` sonrakini), **`]`** onu iki hücre genişletir,
  **`[`** daraltır, **`0`** her sütunu otomatiğe döndürür, `Esc` çıkar. Palette de aynısı var: `Radar: Sütun Seç`,
  `Radar: Sütunu Genişlet`, `Radar: Sütunu Daralt`, `Radar: Sütun Genişliklerini Sıfırla`.

Genişlikler `radar.toml` içinde (`[columns]`, `repo`, `number`, `author` ve `age`, hücre olarak) saklanır ve yeniden başlatmada
korunur. Log'un sütunlarıyla aynı boyutlandırma hareketleridir.

### Denetimler ve çakışmalar

Barındırıcı bir şey söylediğinde bir request satırı, numarasından sonra iki küçük işaret taşır:

| İşaret | Anlamı |
|---|---|
| `✓` | head commit'inin denetimleri geçti |
| `✗` | bir denetim başarısız oldu |
| `●` | denetimler kuyrukta veya çalışıyor |
| `!` (yanında) | request hedef dalıyla çakışıyor |

İşaretlerin üzerinde durun; bir ipucu bunu sözcüklerle söyler ("Denetimler başarısız", "main ile çakışıyor");
satırı seçin, filtrenin altındaki durum çubuğu aynısını söyler, böylece klavye de görür. İşareti olmayan bir
request, barındırıcının hakkında bir şey söylemediği bir request'tir (CI yapılandırılmamış veya sorulmamış).

- **GitLab**, boru hattı durumunu ve çakışma bayrağını merge request listesiyle birlikte gönderir: her satırın
  işaretleri vardır.
- **GitHub** göndermez: listede ikisi de yoktur. GitPoulpi bunları request başına sorar (birleştirme durumu için
  request'in kendisi, head commit'in birleşik durumu ve kontrol çalıştırmaları: üç çağrı), depo başına en çok **15 açık
  request** için, sizinkiler önce. GitHub bu çağrılardan birini reddederse (bir hız sınırı), geri kalanlar sonraki
  yenilemeye kadar işaretsiz kalır. Yanıtlar barındırma önbelleğiyle birlikte kaydedilir.

- **Gitea ve Forgejo** GitHub gibi okunur: liste çakışma bayrağını (`mergeable`) taşır, commit durumu request
  başına sorulur (depo başına en çok 15, sizinkiler önce) ve issue'lar aynı gruplardır. Bahsetmeler sunucunun
  aramasını kullanır (`mentioned_by`).

### Bahsetmeler

**Benden bahsedenler** ve **Göster: Benden bahsedenler** filtresi, barındırıcının bahsetmeler hakkında bildiklerini
kullanır. GitHub'a aramasıyla (`mentions:`), GitLab'a yapılacaklar listenizle sorulur: yalnızca orada hâlâ **bekleyen**
bahsetmeler sayılır, yani bitti olarak işaretlediğiniz bir yapılacak gruptan çıkar.

Kapalı ve merge edilmiş request'ler ile kapalı issue'lar listelenmez.

## Görünümler ve filtreler

Üç açılır liste ve bir metin alanı listeyi daraltır. Her seçim bir açılır listedir:
tıklayın veya tuşunu kullanın.

- **Görünüm** (`v`): Tümü, request'ler, Issue'lar, WIP (süren işler), Ertelenenler.
- **Göster** (`f`): Herkes, Benimkiler (yazdığınız request'ler, açtığınız
  issue'lar), Bana atananlar, İnceleme istenenler, Benden bahsedenler.
- **Kapsam** (`w`): yukarıya bakın.
- **Filtre metni** (`/` veya `Ctrl+F`): depoda, numarada, başlıkta, yazarda, dalda
  ve etiketlerde bulanık eşleşme. `Enter` tuşları listeye geri verir ve metni
  korur, `Esc` onu temizler.

`Ctrl+N` süzgecin, her açılır listenin, Yenile ve Geri düğmelerinin ve ekrandaki her bölüm başlığının üzerinde bir rozet gösterir. Bir rozetin karakterine basın: süzgeç klavyeyi alır, açılır liste açılır, düğme çalışır ve başlık seçimi alır (ardından `Enter` veya `Space` onu daraltır). `Esc` iptal eder. Bkz. [Rozetler](keybindings.md#rozetler).

## Eylemler

Bir satır seçin ve menüsünü (sağ tık, satır sonundaki `⋮`, `.` veya `Menu`), bir
tuşu ya da paleti (`Radar: ...`) kullanın. Bir fare eyleminin her zaman bir
tuşu ve bir palet karşılığı vardır.

| Eylem | Tuş | Palet |
|---|---|---|
| Depoyu aç (sekmesi açılır veya ona geçilir; klonlanmamış bir request nasıl inceleneceğini sorar) | `Enter` / çift tık | `Radar: Depoyu Aç` |
| İncele... (bir request; klonlanmamış bir request nasıl yapılacağını sorar) | `e` | `Radar: İsteği İncele...` |
| Bir request'i sunucunun API'si üzerinden incele (klon yok) | sorudaki `a` | `Radar: API Üzerinden İncele` |
| Depoyu klonla, sonra request'i incele | sorudaki `c` | `Radar: Klonla ve İncele...` |
| Tarayıcıda aç | `o` | `Radar: Tarayıcıda Aç` |
| Dalı checkout et (bir request) | `c` | `Radar: Dalı Checkout Et` |
| Worktree oluştur... (bir request) | `t` | `Radar: Worktree Oluştur...` |
| Depoda göster (listesinde) | `s` | `Radar: Depoda Göster` |
| Issue üzerinde çalışmaya başla... (bir issue) | `n` | `Radar: Issue Üzerinde Çalışmaya Başla...` |
| En üste sabitle veya Sabitlemeyi kaldır | `p` / `★`'a tık | `Radar: Sabitle veya Sabitlemeyi Kaldır` |
| Atananları... / İnceleyenleri... / Etiketleri düzenle..., Kilometre taşı ayarla... (GitHub, GitLab) | satır menüsü | `Radar: Atananları Düzenle...`, `Radar: İnceleyenleri Düzenle...`, `Radar: Etiketleri Düzenle...`, `Radar: Kilometre Taşı Ayarla...` |
| Ertele... | `z` | `Radar: Ertele...` |
| Ertelemeyi kaldır | `u` | |
| Klonlanmamış depoların request'lerini göster veya gizle | `a` | `Radar: Klonlanmamış İstekleri Göster` |
| Hesap geneli satırların taslaklarını göster veya gizle | `d` | `Radar: Taslakları Göster` |
| Bir barındırıcının hesap geneli inceleme isteklerini etkinleştir | `T` / satırındaki ipucunda `Enter` / **Etkinleştir** | `Radar: Hesap Geneli İnceleme İsteklerini Etkinleştir...` |
| Yenile | `r`, `F5` | `Radar: Yenile` |
| Satır menüsü | `.` | `Radar: Satır Menüsü...` |
| Ana sekmeye dön | `Esc` | `Radar: Ana Sayfaya Dön` |

Deponun kendi penceresini gerektiren eylemler (checkout, worktree, göster,
çalışmaya başla, bağla, üst veri seçicileri) önce onun sekmesine geçer ve yüklendikten sonra orada
çalışır:

- **Checkout** kenar çubuğunun kendi checkout'unu kullanır: yerelde biliniyorsa
  dalı, değilse request'in head'ini yerel bir dala fetch ederek (örneğin bir
  fork'unkini).
- **Worktree oluştur...**, Worktree oluştur iletişim kutusunu request'in dalında
  açar.
- **Depoda göster** öğeyi PULL REQUEST'LER veya ISSUE'LAR listesinde seçer.
- **Bir issue üzerinde çalışmaya başla**, adı issue'dan üretilen (`12-fix-the-thing`)
  Yeni dal istemini açar.

### Sabitleme ve erteleme

Bir **sabitleme** öğeyi en üstte tutar. Bir **erteleme** onu **1 saat**, **4
saat**, **Yarına kadar** (ertesi sabah 08:00, yerel saatinizle; şu an henüz 08:00 olmasa bile) veya **Değişene kadar** (barındırıcı öğede bir
güncelleme bildirir; süren iş satırında ise sayaçları oynar) gizler. İkisi de
yapılandırma klasörünüzdeki `radar.toml` içinde, barındırıcı, depo, tür ve
numaraya göre tutulur ve makinedeki tüm oturumlar tarafından paylaşılır.

### Söyleyecek bir şeyi olan depolar

Sorunu olan bir depo listenin sonunda kendi satırını alır ve önbellekteki öğeleri
ekranda kalır:

- **çevrimdışı, 5 dakika önceki veri**: barındırıcıya ulaşılamadı;
- **Yeniden bağlan**: token eksik veya reddedildi; düğme Ayarlar > Entegrasyonlar'ı
  açar;
- **bu turda duraklatıldı**: barındırıcı yavaşlamayı söyledi (veya token'ı reddetti),
  bu yüzden GitPoulpi bu turun diğer depoları için ona yeniden sormadı;
- **bağlı değil, GitHub öneriliyor**: deponun uzağı bir profille eşleşiyor, ama
  barındırma depo başına isteğe bağlıdır, bu yüzden siz bağlayana kadar hiçbir şey
  sorulmaz. **Bağla...** sizi deponun PULL REQUEST'LER bölümüne götürür; orada
  sağlayıcıyı ve projeyi seçersiniz.

Aynı anda en fazla dört depoya sorulur ve asla ekranı çizen iş parçacığında
değil.

## Kapsamın ötesi: hesap geneli satırlar

Kapsam, klonladığınız depoları kapsar. Hiç klonlamadığınız bir depoda incelemenizi bekleyen bir request orada
görünmez. **Hesap geneli satırlar** barındırıcıya doğrudan, profil başına bir kez sorar ve kapsam ne olursa olsun
listeye katılır: bir klasör hakkında değil, sizin hakkınızdadır.

Varsayılan olarak **kapalıdırlar** (bir barındırıcının araması API kotasına mal olur). Bağlı bir kod barındırıcısının
listelerinden hiçbiri açık değilken, listenin en üstünde onun için bir satır görünür:

`GitHub: incelemenizi bekleyen istekleri, klonlamadığınız depolardakiler dahil listele  [Etkinleştir]`

**Etkinleştir**'e tıklayın, o satırda `Enter`'a basın, `T`'ye basın ya da `Radar: Hesap Geneli İnceleme İsteklerini
Etkinleştir...` komutunu çalıştırın: `config.toml` içinde o profil için `integrations.radar.review_requests = true`
yazar ve hemen barındırıcıya sorar. Diğer listeler (ad alanlarınızın her açık request'i, size atanan issue'lar,
taslaklar, ad alanları) `gitpoulpi hosting integrations radar` ile ya da profilin ayarlarında belirlenir; bkz.
[yapılandırma başvurusu](config-reference.md) ve [CLI başvurusu](cli-reference.md). GitHub, GitLab, Gitea,
Forgejo, Azure DevOps, Bitbucket Data Center ve Gerrit yanıt verir; Bitbucket Cloud'un hesap geneli uç noktası
yoktur, bu yüzden request'leri yalnızca kapsamın bağlı depoları için görünür.

Profilin **Ad alanları** tüm listelerini bazı depolarla sınırlar (Ayarlar > Entegrasyonlar > Radar: satır başına bir giriş olan bir metin alanı ve bir **Düzenli ifadeler** onay kutusu). Bir satır, deponun tam yoluyla eşitse veya `/` ile biten bir yol öneğiyse eşleşir: `orbit`, `orbit/products/x`'i tutar, `orbitx/y` ile `training/y`'yi dışarıda bırakır. Onay kutusu açıkken her satır, tam yola yazdığınız haliyle uygulanan bir düzenli ifadedir (`^orbit/(products|infra)/`); geçerli olmayan bir satır alanın altında gösterilir ve kaydedilmez. Boş, sınır yok demektir.

Satırların nasıl yerleştirildiği:

- Kapsamın zaten listelediği bir satır (aynı barındırıcı, proje, tür ve numara) bir kez görünür: **kapsamın
  satırı kazanır**.
- Projesini bir yerde klonladığınız bir satır (son depolar, çalışma alanları, klasör kapsamlarının
  bulduğu klasörler; uzak URL'ye göre eşleşir) o klonu taşır: **Enter**, checkout, worktree ve diğerleri onu
  sıradan bir satır gibi açar.
- Diğer her satır **klonlanmamıştır**: depo çipi soluktur ve projeyi (`acme/app`) gösterir, seçildiğinde durum
  çubuğu `Klonlanmamış` der. **Tarayıcıda aç** çalışır, **İncele** nasıl inceleneceğini sorar (aşağıda) ve checkout,
  worktree, depoda göster ve çalışmaya başla `önce depoyu klonlayın` der.

### Bir isteği incelemek

Kapsamın kendi depolarındaki bir request'te **İncele** (`e`), o deponun sekmesini içinde incelemeyle birlikte açar.
Hesap geneli bir satırın request'inde yol, bilinen bir klon olup olmadığına ve profilin **Klonlanmamış bir depodan
gelen istek** ayarına (Ayarlar > Entegrasyonlar, [yapılandırma başvurusundaki](config-reference.md) `review_mode`)
bağlıdır:

| Ayar | Bilinen bir klon var | Bilinen klon yok |
|---|---|---|
| Sor (varsayılan) | sorar: **Yerel klonu aç** (`l`) veya **API üzerinden incele** (`a`) | sorar: **Klonla, sonra incele** (`c`) veya **API üzerinden incele** (`a`) |
| Klonla | klonu açar | Klonla iletişim kutusunu açar |
| API üzerinden incele | API üzerinden inceler | API üzerinden inceler |

Soru küçük bir iletişim kutusudur: harfler seçer, **Enter** vurgulanan satırı seçer, `Esc`, **İptal**, `✕` ve
dışına tıklamak kapatır, **GitHub için hatırla** (`r`) ise profilin ayarını yazar; böylece soru bir daha
sorulmaz (Ayarlar satırı onu geri değiştirir). `Ctrl+N` denetimlerini rozetler.

- **API üzerinden incele**, incelemeyi Ana Sayfa penceresinde, Radar'ın üzerinde açar; kod barındırıcıdan alınır
  (bkz. [Klon olmadan inceleme](review.md#klon-olmadan-inceleme)). `Esc` veya `✕` Radar'ı geri verir; paletin
  *İnceleme: ...* satırları onun üzerinde çalışır.
- **Klonla, sonra incele**, Klonla iletişim kutusunu satırın geldiği bağlı hesapta açar (aynı depo listesi, sahibe
  göre gruplanmış); request'in deposu seçili ve varsayılan klonlama klasörü doldurulmuş olur; ikisinden birini
  değiştirin, sonra o hesaptan yapılan her klonla aynı oturum açmayla klonlayın. Depo hesabın listesinde değilse
  (üyesi değilsiniz), liste açık kalır, arama onun adıyla yapılmıştır ve bir not bunu söyler; depoları listeleyemeyen
  bir barındırıcıda iletişim kutusu adres doldurulmuş olarak **URL ile klonla** üzerinde açılır. Klon tamamlandığında yeni depo barındırıcıya bağlanır
  (PULL REQUEST'LER kaynağı ve barındırıcının issue'ları varsa ve hiçbiri ayarlı değilse ISSUE'LAR kaynağı),
  sekmesi açılır ve o request'in incelemesi içinde açık olur. İletişim kutusunu iptal etmek ya da başarısız bir klon
  hiçbir şeyi bağlamaz.
- Azure DevOps ve Bitbucket request'lerinin uygulama içi incelemesi yoktur: **İncele** request'in sayfasını
  tarayıcıda açar ve bunu durum çubuğunda söyler.

Paletin *Radar: API Üzerinden İncele* ve *Radar: Klonla ve İncele...* satırları soruyu atlar.

Filtrenin yanındaki iki onay kutusu (bir hesap geneli liste sorulmaya başlayınca görünürler), `radar.toml` içinde tutulur:

| Onay kutusu | Tuş | Varsayılan | Etki |
|---|---|---|---|
| `[x] Klonlanmamış` | `a` | açık | Bilinen klonu olmayan depoların satırlarını listeler. Kapalıyken yalnızca bilinen klonların satırları kalır. |
| `[ ] Taslaklar` | `d` | kapalı | Hesap geneli satırların taslak request'lerini listeler. Kapsamın kendi depolarının taslakları her zaman listelenir. |

İkisi de `Ctrl+N` rozetlerinde ve paletteydir (`Radar: Klonlanmamış İstekleri Göster`, `Radar: Taslakları Göster`).

Bir profilin yanıtı `radar-account.json` içinde **5 dakika** tutulur: Radar'ı bu süre içinde yeniden açmak,
barındırıcıya sormadan satırları gösterir. **Yenile** (`r` veya `F5`) her zaman sorar: son yanıtın yaşı ne olursa olsun her depoyu ve her hesap geneli listeyi. Yavaşlamayı söyleyen ya da token'ı
reddeden bir barındırıcıya aynı turda yeniden sorulmaz. Profil ve tür başına en fazla 300 satır tutulur;
**Depolar** altındaki bir satır kaç tane daha olduğunu ya da bir profilin neden okunamadığını söyler
(**Yeniden bağlan** Ayarlar > Entegrasyonlar'ı açar).

## Sınırlar

- Hesap geneli satırlar GitHub, GitLab (kendi barındırdığınız dahil), Gitea, Forgejo, Azure DevOps, Bitbucket Data
  Center ve Gerrit profillerinden gelir (Bitbucket Cloud değil). Size atanan issue'lar ilk beşinden gelir;
  Bitbucket ve Gerrit'te issue yoktur.
- Depo başına listeler GitHub, GitLab (kendi barındırdığınız dahil), Gitea, Forgejo, Azure DevOps, Bitbucket ve
  Gerrit profillerini kullanır; denetimler, çakışmalar ve bahsetmeler yalnızca ilk dördü için okunur.
- GitLab bahsetmeleri yalnızca bekleyen yapılacaklardan gelir; GitHub, Gitea ve Forgejo denetimleri depo başına 15 request için sorulur.
