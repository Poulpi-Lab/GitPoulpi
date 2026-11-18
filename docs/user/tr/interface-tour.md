[English](../interface-tour.md) · [Čeština](../cs/interface-tour.md) · [Deutsch](../de/interface-tour.md) · [Español](../es/interface-tour.md) · [Français](../fr/interface-tour.md) · [Italiano](../it/interface-tour.md) · [日本語](../ja/interface-tour.md) · [한국어](../ko/interface-tour.md) · [Nederlands](../nl/interface-tour.md) · [Polski](../pl/interface-tour.md) · [Português (Brasil)](../pt-BR/interface-tour.md) · [Português (Portugal)](../pt-PT/interface-tour.md) · [Русский](../ru/interface-tour.md) · [Svenska](../sv/interface-tour.md) · **Türkçe** · [Українська](../uk/interface-tour.md) · [简体中文](../zh-CN/interface-tour.md)

# Arayüz turu

Bu sayfa GitPoulpi penceresinin her alanını yukarıdan aşağıya adlandırır. Diğer
sayfalar da aynı adları kullanır. Görseller, uydurma yazarlara sahip küçük bir
demo deposu kullanır.

![Ana pencere ve alanları: sekme çubuğu, araç çubuğu, Refs kenar çubuğu, Log, çalışma değişiklikleri paneli, commit formu ve durum çubuğu](../images/hero.png)

## Çerçeve

### Başlık çubuğu

Pencerenin üst kenarında
<img src="../../../assets/logo/gitpoulpi.png" alt="GitPoulpi simgesi" height="16"> `GitPoulpi vX.Y.Z -- <depo yolu>` yazar (oturum adı durum çubuğundadır, varsayılan olarak en altta).

- **<img src="../../../assets/logo/gitpoulpi.png" alt="GitPoulpi simgesi" height="16">**, **GitPoulpi Hakkında** penceresini açar: sürüm, derleme özellikleri,
  lisans, bağlantılar, GitPoulpi'nin bağımsız bir proje olduğunu söyleyen kısa bir
  not, üçüncü taraf crate'ler ve lisanslar ile emeği geçenler. Görüntü çizen bir terminalde
  ahtapot piksel logodur ve logo renginizdedir (başka yerde emoji olarak kalır).
  Palet: `Yardım: GitPoulpi Hakkında`; tuşlar: `Ctrl+N` ardından `A`. Hakkında
  penceresinde ahtapotun gözbebekleri fare imlecini izler. Logoya tıklayın (veya
  `Space` tuşuna basın); logo bir sonraki renge döner. Renklerin listesi için
  logoya sağ tıklayın (veya `c` tuşuna basın). GitPoulpi seçiminizi
  `ui.logo_color` içine kaydeder. Bağlantılar bir tıklamayla ya da erişim
  tuşlarıyla açılır (`s` kaynak kodu, `d` belgeler, `i` lisans); `l` ise üçüncü
  taraf crate'leri sürümleri ve lisanslarıyla listeler (kaydırılabilir bir
  liste; `Esc` veya `l` ya da başlığına tıklamak geri döner). Listedeki her
  crate, kendi sayfasına bir bağlantı taşır (deposu, yoksa crates.io sayfası):
  tarayıcıda açmak için satıra tıklayın ya da `↑` `↓` ile seçip `Enter` tuşuna
  basın. `t` (veya `Yardım: Lisanslar ve Emeği Geçenler` palet komutu) programa gömülü her şeyin lisans bildirimlerini ve emeği geçenleri kaydırılabilir bir metin olarak gösterir (`Up` / `Down`, `PgUp` / `PgDn`, `Home` / `End`; geri dönmek için `Esc` veya `t`). Bir betikte aynı metni `gitpoulpi licenses` yazdırır.
- **Sürüm** (`v1.0.0`; daha yeni bir sürüm varsa `vX.Y.Z kullanılabilir`) ad ile
  yol arasında durur: daha yeni bir sürüm bilindiğinde (ya da bir güncelleme
  sürerken) tıklamak güncelleme panelini, aksi halde **Sürüm notları** sekmesini
  açar, bkz. [Güncellemeler](updates.md).
- Yolun hemen ardından gelen **Şurada aç ▾**, sekmenin klasörünü bir terminalde,
  bir düzenleyicide, bir multiplexer'da veya dosya yöneticisinde açar
  (`Alt+O`). Bkz. [Şurada aç](open-in.md).
- Dişlinin hemen solundaki **⌕ Genel arama** (bir büyüteç ve etiketi) [genel aramayı](search.md) açar (`Ctrl+Alt+F`). Başlık çubuğu çok darken ilk küçülen odur: önce etiket kaybolur (büyüteç kalır), sonra düğme.
- **×** simgesinin hemen solundaki **dişli** (`⚙`) **Ayarlar**'ı açar (`Ctrl+,`).
- Sağ uçtaki **×** çıkar, `q` gibi.

![Ana pencerenin üzerindeki Hakkında penceresi: piksel ahtapot, sürüm, lisans, derleme özellikleri, bağlantılar ve crate sayısı](../images/about.png)

![Hakkında penceresindeki üçüncü taraf crate listesi: her satırda ad, sürüm, crate sayfasına bağlantı ve lisans](../images/about-crates.png)

### Durum çubuğu

Durum çubuğu varsayılan olarak pencerenin son satırıdır (**Ayarlar > Görünüm > Durum çubuğu konumu** onu depo sekmelerinin üstüne, en üste taşır). Geçerli dalı, yüklenen
commit'leri (daha eski geçmişin yüklenmesi sürerken `500 / 48213 Commit'ler`, toplam
sayılırken toplamın yerinde `…`; tüm geçmiş yüklenince yalnızca `2500 Commit'ler`),
`dirty` veya `clean`, oturum adını, seyrek (sparse) bir checkout'ta bir `kısmi`
rozetini ve en işe yarar tuşları gösterir: `?` yardım, `Ctrl+G` Git eylemleri,
`Ctrl+N` gezinme, `Ctrl+P` komutlar, `Shift+F10` menü, `q` çıkış. Dar bir
pencerede en az işe yarayan tuşlar bırakılır. Durum çubuğu durum mesajlarını ve
ilerlemeyi de gösterir. Gösterdiği tuşlar her zaman geçerli olan tuşlardır;
değiştirdikleriniz de buna dahildir (bkz.
[Ayarlar](settings.md#klavye-ve-fare)). Toplamın nasıl sayılacağı bir ayardır
(**Toplam Commit sayısını şununla hesapla**, bkz. [Ayarlar](settings.md#log-ve-grafik)).

Sağ ucundaki **Etkinlik** düğmesi Etkinlik Günlüğü'nü açar: bir Git
işlemleri sekmesi (işlemler, ne kadar sürdükleri, mesajlar ve hatalarla), bir
Uygulama sekmesi (düzey kutusu ve **Günlüğü temizle** düğmesi olan uygulama
günlüğü) ve ikisini bu depoya, bir çalışma alanına ya da tüm depolara daraltan
bir Kapsam kutusu. Düğmenin üzerindeki sayı, henüz bakmadığınız hataları sayar.
Solunda, iş göstergesi çalışan bir arka plan işini ve onu iptal eden bir `[x]`
gösterir (iş listesinde **Hepsini iptal et** vardır, palette de *Görevler: İptal
Edilebilen Her Şeyi İptal Et*; güvenle durdurulamayan bir iş nedenini söyler). **Ana** sekmede de aynı durum çubuğu vardır; Etkinlik düğmesi sağ alttadır
(Tüm depolar kapsamında açılır). Bir işlem başarısız olduğunda durum çubuğu kırmızıyla nedenini söyler; Etkinlik
Günlüğü'ndeki girdisi bir sonraki adımı, ayrıntıları ve bir YZ için Kopyala
seçeneğini sunar. İkisi de
[Bir şey başarısız olduğunda](troubleshooting.md#bir-şey-başarısız-olduğunda) bölümünde anlatılır.

### Katmanlar

Komut paleti, yardım, bağlam menüleri, iletişim kutuları, açılır pencereler ve
Ayarlar pencerenin üstünde açılır. Her birinin kenarlığının sağ üstünde bir `✕` vardır; `Esc` veya `✕`'e tıklamak kapatır, bir iletişim kutusunun, açılır pencerenin ya da panelin dışına tıklamak da onu kapatır (bkz. [Pencereleri ve iletişim kutularını kapatma](mouse-and-gestures.md#pencereleri-ve-iletişim-kutularını-kapatma)).

### Pencere küçükken

Pencere daraldıkça hiçbir şey sessizce kaybolmaz. Bir düğme önce uzun etiketini yitirir (`...` ile kırpılır), sonra
ikinci bir satıra sarılır ya da satırının `⋯` menüsüne girer; tuşu ve palet girişi her zaman çalışmaya devam eder.
Pencereye sığmayan bir diyalog, hiçbir şey çizmemek yerine başlığı, `✕` ve **Pencere çok küçük: büyütün** satırı olan
küçük bir çerçeve çizer; `Esc` ve `✕` onu kapatır, pencereyi büyütünce diyalog bütün olarak geri gelir.

## Sekme satırı

**Ana** sekme (ev simgesi) sol uçtadır; ardından [çalışma alanı rozeti](workspaces.md)
(bir çalışma alanınız olunca), sabitlenmiş **Radar** sekmesi (her sekme gibi kapanır;
bkz. [Radar](radar.md)) ve açtıysanız **Çalışma alanları** sekmesi (`Alt+J`; her sekme
gibi kapanır) gelir. Sonra açık her depo için bir sekme gelir. Bir sekmeyi kapatan `x`
yalnızca etkin sekmede ve imlecin altındaki sekmede görünür. Sekme satırında yalnızca
sekmeler vardır ([genel arama](search.md) düğmesi başlık çubuğundadır).

Depo sekmeleri sığmadığında satır küçülmek yerine kayar: sekmeler iki ok düğmesi
arasında durur, solda `‹` ve sağda `›`. Bir yanında gizli sekme yoksa o yöndeki ok
soluk görünür. O yanda bir sekme daha göstermek için oka tıklayın (kaydırmayı
sürdürmek için basılı tutun) ya da satırın üzerinde tekerleği kullanın (yukarı veya
sola, önceki sekmeleri gösterir). Geçerli sekme değiştiğinde (klavyeden, sekme
seçiciden veya paletten sekme değiştirdiğinizde, bir sekme açtığınızda veya
kapattığınızda ya da pencereyi yeniden boyutlandırdığınızda) satır, onu bütünüyle
göstermeye yetecek kadar kayar. Yer varsa sağ okun ardındaki bir `…` düğmesi sekme
seçiciyi açar. Bir sekmeyi oka sürüklemek, orada tuttuğunuz sürece satırı kaydırır;
böylece onu ekrandaki sekmelerin ötesine bırakabilirsiniz. `Ctrl+N`, etkin her oka bir
rozet koyar.

**Ayarlar > Görünüm > Sekme çubuğu konumu** bu satırı pencerenin altına (durum çubuğunun altına) taşır ya da sol kenarda bir sütuna çevirir: satır başına bir sekme, önce Ana Sayfa, sonra çalışma alanı rozeti, Radar, Çalışma alanları sekmesi, sekmeler (uzun bir ad `…` ile biter) ve `+`. Sekmeler satırlardan çoksa, bir `…` satırı yukarıda (`▲`) ve aşağıda (`▼`) kaç sekmenin gizli olduğunu gösterir ve sekme listesini açar; tekerlek sütunu kaydırır. Sütun biçimindeki bir araç çubuğunun yanında sekmeler en dışta kalır. 80 sütundan dar bir pencere sekmeleri üstte tutar. Tıklamalar, kapatma çarpısı, orta tıklama, sağ tık menüsü, sıralamak için sürükleme ve klavye her konumda aynı çalışır.

- Son sekmenin hemen ardındaki **+** küçük bir menü açar: **Yeni sekme** (`Alt+T`, palette
  **Sekme: Yeni Sekme**) ve sekmeleri kapalıyken o sekmeleri açan **Radar** ile
  **Çalışma alanları** (`Alt+J`); paletteki **Sekme: Yeni...** aynı menüyü
  açar. Bir **Yeni sekme**, arama alanı hazır Ana görünümdür, yani yazmak
  depolarınızı hemen filtreler. Birini seçin (veya Gözat, Klonla, Init) ve sekmesi
  çubuğun sonunda belirir; zaten açık olan bir depoya yalnızca geçilir. Boş
  aramada `Esc`, `Ctrl+W`, `x`'i veya orta tık Yeni sekmeyi kapatır ve içinde
  olduğunuz sekmeye döner. Oturumunuza asla kaydedilmez.

  ![Bir Yeni sekme: arama alanı hazır Ana görünüm, açık sekmelerin üzerinde](../images/new-tab.png)
- Sekme menüsü için bir sekmeye sağ tıklayın veya `Ctrl+T` tuşuna basın:
  **Yeni sekme**, **Sekmeyi kapat**, **Diğer sekmeleri kapat...**, **Sağdaki sekmeleri kapat...**,
  **Favorilere ekle** (veya **Favorilerden kaldır**), **Çalışma alanına ekle** (veya
  **Çalışma alanından çıkar**, çalışma alanlarınız varken), **Yolu kopyala**,
  **Şurada aç ›**, **SSH ve kimlik bilgileri...** ve **Kısmi checkout...**.
- `Ctrl+PageUp` ve `Ctrl+PageDown` sekmeler arasında geçiş yapar; terminaliniz
  ilettiği sürece `Ctrl+Shift+Tab` ve `Ctrl+Tab` de öyle (kitty klavye
  protokolü; bazı terminaller ve çoklayıcılar bunları kendilerine ayırır). `Ctrl+W`
  geçerli sekmeyi kapatır. Bir sekmeye orta tık da onu kapatır (aynı soruyla).
- `t` (veya `Ctrl+N Tab`) sekme seçiciyi açar; açık sekmelerin aranabilir bir
  listesidir.
- **Sekmeleri yeniden sıralama:** bir depo sekmesine basıp yana sürükleyin (sekmenin bir gölgesi
  işaretçiyi izler ve bir çubuk nereye ineceğini gösterir; `Esc` iptal eder), ya da sekme
  seçicinin bir satırını sürükleyin, seçicide vurgulanan satırda `Alt+Yukarı` / `Alt+Aşağı`
  tuşlarına basın veya paletten **Sekme: Sola Taşı** / **Sekme: Sağa Taşı** komutunu çalıştırın.
  Sıra oturumla birlikte kaydedilir. Hareketsiz düz bir tıklama yine sekmeyi seçer. Ev simgesi,
  çalışma alanı rozeti ve `+` asla yer değiştirmez.
- GitPoulpi'yi `gp` kabuk sarmalayıcısıyla başlattıysanız ve sekme, başladığınız
  klasör değilse, sekmelerin sağında **Çık ve cd yap** görünür. Bkz.
  [Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md).
- Ev simgesinin hemen ardındaki **çalışma alanı rozeti** (renkli bir nokta ve bir
  ad, ya da "Çalışma alanı yok") [çalışma alanını](workspaces.md) değiştirir:
  tıklayın veya `Alt+W` tuşuna basın. Bir çalışma alanınız olunca görünür.

![Açık depoları listeleyen sekme seçici](../images/tab-picker.png)

**Sekmeleri hatırla** açıkken (varsayılan), sekmeleriniz bir sonraki açılışta
geri gelir. Bir worktree kendi sekmesinde, deposunun sekmesinin hemen arkasında
açılır (Ayarlar > Genel içindeki **Worktree sekmelerini birlikte grupla**, varsayılan
olarak açık). Worktree kaldırıldığında bu sekme kapanır.

### Ana sekme

Ana sekme depo yöneticisidir. Dört bölümü vardır: **AÇIK DEPOLAR**,
**FAVORİLER**, **SON DEPOLAR** ve **TÜM DEPOLAR**. Her satır dalı, önde ve
geride olan commit sayılarını ve değişiklik sayısını, sonra yer varsa klasörün
yolunu gösterir: `~` ile kısaltılır, dar olduğunda ortasından kesilir ve adı,
sahibi veya dalı asla dışarı itmez. Her satırda bir yıldız (favori), **Aç** veya **Geç**, **Kapat** veya **Kaldır** ve bir `⋮` menüsü
vardır.

En üstte: **Gözat**, **Klonla**, **Init**, bir arama kutusu, **Tümünü daralt**
ve **Tümünü genişlet**. Artık var olmayan bir klasör *Son* ve *Tüm*
bölümlerinden kendiliğinden çıkar. Eksik bir favori kalır ve yanında bir
**Kaldır** düğmesi görünür. **Kaldır** diskte hiçbir şeyi silmez.

Tuşlar: `Enter` aç, `f` favori, `x` kapat veya kaldır, `y` yolu kopyala, `o` varsayılan araçta aç (`⋮` menüsünün **Şurada aç** seçeneği başka birini seçer), `.` menü, `/` ara,
`Space` bir bölümü katla, `Esc` sekmenize dön.
`Ctrl+N` arama alanında, her bölümde ve her düğmede bir rozet gösterir: odaklanmak veya basmak için rozetin karakterine basın (bkz. [Rozetler](keybindings.md#rozetler)).

Tek bir arama alanı altındaki her şeyi, Favoriler dahil, süzer: bir depo, adı, dalı ya da yolu metni içeriyorsa kalır. Çalışma alanlarının kendi sekmesi vardır ve Ana sekmenin bir sayfası değildir: `+` menüsünden, `Alt+J` ile ya da komut paletinden açın, `x` ile kapatın: bkz. [Çalışma alanları](workspaces.md).

**Klonla**, bağlı her profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) için erişebildiğiniz depoları,
sahibe göre bir bölüm olacak şekilde listeler: önce sizinkiler, sonra her
organizasyon veya grup için bir bölüm (alfabetik), her biri bir başlığın altında.
Kilit simgesi özel bir depoyu işaretler. Arama kutusu tüm bölümlerde süzer;
eşleşmesi olmayan bölüm kaybolur. `Up` ve `Down` başlıkları atlar; `Home`,
`End`, `PgUp` ve `PgDn` listede çalışır.

![Ana sekme: açık, favori ve son depolar ile Gözat, Klonla ve Init düğmeleri](../images/home.png)

## Araç çubuğu

Varsayılan olarak bölmelerin altındaki, durum çubuğunun üstündeki satırdır. **Ayarlar > Görünüm > Araç çubuğu konumu** onu üste (sekmelerin altına), alta ya da bölmelerin soluna veya sağına bir sütun olarak koyar (80 sütundan dar bir pencere sütunu altta bir satır olarak çizer). Düğmeler satırın ya da sütunun başında durur; **Ayarlar > Görünüm > Araç çubuğu düğmelerinin hizası** onları ortalar veya sona taşır. Her düğmenin temanın belirlediği kendi simge rengi vardır (bkz. [Temalar](themes.md)). Hepsine yer yoksa en az önemli düğmeler bir `⋯` arkasına geçer. Sütunda her düğme bir satır tutar, `▾` satırın sonundadır ve menüsü sütunun yanında açılır; alttaki araç çubuğunun menüsü yukarı açılır.

| Düğme | Ne yapar |
|---|---|
| **Geri al**, **Yinele** | Son işlemi geri alır veya yeniden uygular (`Ctrl+Z`, `Ctrl+Y`). Geri Al kapalıyken gizlidir. Bkz. [Geri al ve yinele](undo-redo.md). |
| **Pull** `▾` | **Ayarlar > Git ve Kodlama > Pull düğmesi** ayarının dediği gibi pull yapar. `▾` listesinde **Fetch All**, **Pull (mümkünse fast-forward)**, **Pull (yalnızca fast-forward)** ve **Pull (rebase)** vardır. Listeden seçilen öğe bir kez çalışır ve düğmeyi değiştirmez. |
| **Push** `▾` | Checkout yapılmış dalı push eder. `▾` listesi, seçtiğiniz uzak depoya bir kez push eder. |
| **Dal** | Checkout yapılmış commit'te bir dal oluşturur. |
| **Tag** | Checkout yapılmış commit'te bir tag oluşturur. |
| **Stash**, **Pop** | Tüm değişiklikleri stash'ler. En son stash'i pop eder (uygular ve siler). |
| **LFS** `▾` | Git LFS eylemleri: fetch, checkout, pull, push, prune (`Ctrl+G l`). Git LFS kapalıyken gizlidir. Bkz. [Git LFS](lfs.md). |
| **Flow** `▾` | Dal iş akışı menüsü (başlat, bitir, eşitle, yükselt; `Ctrl+G Shift+W`). Yalnızca bir iş akışı olan depoda. Bkz. [Dal iş akışları](branching-workflows.md). |
| **Ara** | Log'un üstünde commit arama çubuğunu açar (`f`). |

Uçta (satırda sağ, sütunda alt), depo bir barındırma hizmetine bağlıysa, checkout yapılmış dalın pull request'ini gösteren bir **çip** (örneğin `GH #12` veya `GL !53`) ya da soluk bir `+ PR` görünür. **Ayarlar > Görünüm > Pull Request düğmesini sona sabitle** (varsayılan olarak açık) onu orada tutar; kapalıyken diğer düğmelerle birlikte akar ve onların hizasını izler. Dar bir satırda çip önce küçülür (`● #12`, sonra yalnızca durum simgesi) ve ancak ondan sonra `⋯` menüsüne geçer. Bkz. [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md). Sütunda profil açılır kutusu son satırı alır.

İki veya daha fazla [profiliniz](settings.md#profiller) varsa sağ uçtaki
**profil açılır kutusu** bu depo için commit kimliğini seçer. İlk satır,
*Varsayılan*, **Ayarlar > Profiller**'i izler. Bir profil seçtikten sonra
GitPoulpi, adını ve e-postasını deponun git yapılandırmasına da (`user.name`,
`user.email`) yazıp yazmayacağınızı sorar. **Git yapılandırmasına yaz**
bunları yazar. `Esc` seçimi yalnızca GitPoulpi içinde tutar. Palet:
`Depo: Profili değiştir...`.

Her düğmenin bir palet komutu ve bir kısayolu vardır.

![Araç çubuğunun altında açık Pull listesi: Fetch All ve pull yapmanın üç yolu](../images/pull-push-tooltip.png)

## Bannerlar

Log'un üstündeki bir şerit, deponun özel bir durumda olduğunu söyler. İlerlemek
için düğmeleri vardır. Her düğme aynı zamanda bir palet komutudur.

| Banner | Düğmeler |
|---|---|
| Bir merge, cherry-pick veya revert çakışmalarda durdu | **Çakışmaları çöz...**, **Commit** veya **Devam et**, **İptal et** |
| Süren bir rebase | **Çakışmaları çöz...**, **Devam et**, **Atla**, **İptal et** |
| **Yamalar uygulanıyor (git am)** | **Çakışmaları çöz**, **Devam et**, **Atla**, **İptal et** |
| Kalıntı bir `index.lock` her değişikliği engelliyor | **index.lock dosyasını sil** (önce sorar) |
| Bir otomatik stash geri konmadı | **Şimdi pop et**, **Stash göster**, **Baştan başla**, **At** (seçenekler duruma bağlıdır) |

Bkz. [Merge çakışmalarını çözmek](merge-conflicts.md) ve
[Sorun giderme ve SSS](troubleshooting.md).

## Refs kenar çubuğu (sol)

Bir commit'e ad veren her şeyin ağacı. Her bölümün başlığında bir sayı, katlamak
için bir ok ve kendi kaydırma konumu vardır. İki bölümün arasındaki kenarlığı
sürükleyerek boyutlarını değiştirin.

- En üstteki **Filtre** kutusu (`/`): her bölümü ada göre filtreler.
- **YEREL**: dallarınız, `/` ile bölünen klasörler içinde (`feature/`, `fix/`).
  Bir onay işareti checkout yapılmış dalı gösterir. Klasörleri Ayarlar'dan
  kapatabilirsiniz.
- **UZAK**: her uzak depo ve altındaki dalları.
- **WORKTREE'LER**: varsa bağlı worktree'ler (veya bölümü gösterdiyseniz).
  Başlığındaki **+** bir tane oluşturur; bir worktree'ye çift tıklayarak kendi
  sekmesinde açın, sağ tıklayarak Şurada aç, Merge ve Kaldır seçeneklerine
  ulaşın. Bkz. [Worktree'ler](worktrees.md).
- **STASH'LER**: stash'leriniz. Birine sağ tıklayarak **Pop**, **Apply** ve
  **Drop** seçeneklerine ulaşın.
- **TAG'LER**: tag'leriniz; dallar gibi `/` ile bölünen klasörler içinde (`releases/`, `backups/`). Bir ok, tag'in uzak depoda da olduğunu gösterir.
- **PULL REQUEST'LER** ve **ISSUE'LAR**: bir barındırma hesabı bağlayıp depoyu
  ilişkilendirdikten sonra.
- **ALT MODÜLLER**: varsa alt modüller ve durumları. Bkz.
  [Alt modüller](submodules.md).

**Bölümleri gösterme veya gizleme:** bir bölüm başlığına ya da panelin boş kısmına sağ tıklayın; bölümlerin işaret listesi açılır (görünen son bölüm kalır). Seçim her depo için ayrı tutulur ve yeniden başlatmalarda korunur; bkz. [Sol panel bölümleri](mouse-and-gestures.md#sol-panelin-bölümleri).

Düğmelerini görmek için bir satırın üzerine gelin: ref'i Log'da gizleme veya
gösterme ve **Solo** (yalnızca o ref'in commit'lerini gösterir). Checkout etmek
için bir yerel dala veya bir tag'e çift tıklayın. Menüsü için bir satıra sağ
tıklayın. Kenar çubuğu başlığındaki `‹` kenar çubuğunu daraltır (`Ctrl+N Shift+H`).

İki yan bölme (Refs kenar çubuğu: `Ctrl+N Shift+H`; sağ bölme: `Ctrl+N Shift+L`) Log'a aittir: bir diff, dosya görünümü veya düzenleyici açıkken birini göstermek ya da daraltmak yalnızca o görünüm için geçerlidir. Bir diff veya dosya görünümü, içeriğe yer açmak için Refs kenar çubuğunu kendiliğinden de daraltır. Log'a dönünce iki bölme de orada bıraktığınız gibi durur ve bir sonraki açılış için kaydedilen durum odur.

## Log (orta)

![Log: sütun başlıklarından oluşan bir satır, en üstte çalışma değişiklikleri satırı, bir stash satırı, grafiğin solunda ref hapları](../images/log-graph.png)

Her commit için bir satır, en yenisi en üstte. Sütunları göstermek veya
gizlemek için başlığa sağ tıklayın. Bir sütunu yeniden boyutlandırmak için iki
başlık arasındaki kenarlığı sürükleyin. Sütunu taşımak için bir başlığı
sürükleyin. Genişlikler depoya aittir: onunla birlikte kaydedilir ve yeniden
açtığınızda geri gelir; ilk kez açılan bir depoda her sütun Log'un gösterdiğine
göre sığdırılır (`Tüm Sütunları İçeriğe Sığdır` bunu yeniden yapar, `Sütun
genişliklerini sıfırla` otomatik genişliklere döner ve onlar kalır).

Panelin başlığı yoktur: sütun başlıkları ne olduğunu söyler. Ref'i olmayan bir commit'te, üzerine gelinen satır ait olduğu dalın adını taşıyan soluk bir hap gösterir (Ayarlar > Log ve grafik içindeki **Üzerine gelinen Commit için soluk bir dal/Tag göster** ile kapatılır).

| Sütun | Gösterdiği |
|---|---|
| **Ref** | Commit üzerindeki dal ve tag'ler için renkli haplar: yerel için bir dizüstü simgesi, uzak için bir küre, tag'ler için bir etiket simgesi, oraya daha fazla ref işaret ettiğinde `+N`. Tag'ler bulundukları yerde çizilir, asla hayalet olarak değil. Ref'i olmayan bir commit'te, üzerine gelinen satır ait olduğu dalın adını taşıyan soluk bir hap gösterir; o dal silinmişse, onu merge eden merge commit'inden okunan adı taşıyan üstü çizili bir hap gösterilir (Ayarlar > Log ve grafik içindeki **Üzerine gelinen Commit için soluk bir dal göster** ile kapatın). |
| **Grafik** | Dal şeritleri, merge'ler ve dal noktaları. kitty, Ghostty, WezTerm, Rio ve iTerm2 (ve sixel üzerinden Windows Terminal, foot ile Tabby)'de resim olarak, diğer terminallerde karakterlerle çizilir. Şerit renkleri temadan veya seçtiğiniz bir paletten gelir (bkz. [Temalar](themes.md)). En az genişlikte (tek commit noktası) sütun kompakttır: her commit tek bir şeritte, dal veya merge çizgisi olmadan. |
| **Mesaj** | Commit özeti. |
| **Yazar** | Yazar, isteğe bağlı bir avatarla. |
| **Tarih** | Yazar tarihi, seçtiğiniz biçimde. |
| **Değişim** | İsteğe bağlı: değişen dosyalar ve `+eklenen -silinen` satırlar. |
| **SHA** | Kısa hash. |

![Log başlığının sütun menüsü: Ref, Grafik, Mesaj, Yazar, Tarih, Değişim ve SHA onay kutuları](../images/log-columns-menu.png)

Özel satırlar:

- En üstteki **çalışma değişiklikleri satırı** (WIP satırı): kesik çizgili bir
  daire, `// WIP` veya taslak mesajınız ve değişen ile eklenen dosya sayıları.
  Sağ panelde stage'lemek ve commit etmek için üzerine tıklayın.
- **Stash satırları**: bir şeritte küçük bir kare, stash'in yapıldığı commit'in
  yanında.
- Arama sırasında eşleşmeyen satırlar soluk görünür.

Tam metni içeren bir araç ipucu için bir hücrenin üzerine gelin. [Commit
menüsü](log-and-history.md#commit-menüsü) için bir satıra sağ tıklayın.

![Bir commit'in sağ tık menüsü: cherry-pick, revert, reset, mesajı düzenle, açıkla, rebase, dal, tag, checkout, worktree, yama ve kopyalama eylemleri](../images/context-menu.png)

## Sağ panel

Sağ panel, Log'daki seçimi izler.

**Çalışma değişiklikleri** (WIP satırı seçiliyken):

- `main üzerinde 4 dosya değişikliği` gibi bir başlık. Çakışmalı bir merge
  sırasında çakışan dosya sayısını da ekler. Soldaki kırmızı `✗` tüm
  değişiklikleri atar (önce sorar). YZ açıkken sağdaki `✦` değişiklikleri
  açıklar.
- **Yol** veya **Ağaç** (düz bir liste veya klasörler, `m`), **Tümünü Daralt** ve
  **Tüm dosyaları göster** (`a`).
- **Unstaged Dosyalar** ve **Staged Dosyalar**; her birinin bir sayısı ve bir
  **Tüm Değişiklikleri Stage'le** veya **Tüm Değişiklikleri Unstage'le** düğmesi
  vardır. Satırın **Dosyayı Stage'le** veya **Dosyayı Unstage'le** düğmesini
  görmek için üzerine gelin.
- Altta **commit formu**: **Önceki commit'i amend et**, **YZ ile commit'ler
  oluştur** (YZ açıkken), karakter sayacı ve `✦` düğmesi olan **Özet**,
  **Açıklama** ve **N dosyadaki değişiklikleri commit et** düğmesi.

**Commit ayrıntıları** (bir commit seçiliyken), yukarıdan aşağıya: tam hash,
avatarlı ve tarihli yazar, üst commit'in hash'i ve değişiklik sayıları, ardından
mesaj (özet ve açıklama; yeniden boyutlandırmak için alt kenarlığını sürükleyin).
Alttaki **Değişen dosyalar** paneli kendi denetimleriyle başlar: sıralama düğmesi,
**Yol** / **Ağaç**, **Tüm dosyaları göster** ve bir **Dosyaları filtrele** kutusu;
bunlar listenin üstünde sabit durur ve yalnızca liste kayar. Diff'ini açmak için bir
dosyaya tıklayın. Kısa bir pencerede önce mesaj (bir satıra kadar), sonra liste yer
bırakır. Çalışma değişiklikleri görünümü **Yol** / **Ağaç** / **Tüm dosyaları göster**
satırını dosya listelerinin en üstünde, Unstaged ve Staged listelerinin üstünde tutar.

**Mesajdaki bağlantılar.** Mesaja yazılmış bir web adresi (`https://...` veya bir
markdown `[metin](https://...)` bağlantısının hedefi) altı çizili görünür, el
imleci gösterir ve tıklandığında, hiçbir şey açılmadan önce adresin tamamıyla
birlikte **Bu bağlantı açılsın mı?** diye sorar: mesaj metni güvenilir değildir.
**Aç** (`A`) tarayıcıyı başlatır, **İptal** (`P`, `Esc`, çarpı) hiçbir şey yapmaz.
Yalnızca `http` ve `https` adresleri sunulur. Klavye karşılığı, mesajın ilk adresini
soran ve diğerlerini **Sonraki** (`S`) ile dolaşan `Commit Ayrıntıları: Mesajdaki
Bağlantıyı Aç...` palet komutudur. Bazı terminaller imlecin altındaki bir adrese kendi
altçizgisini çizer ve değiştirici tuşlu bir tıklamada (çoğunlukla Shift+tık) onu
kendileri açar; bu terminalin özelliğidir, GitPoulpi'nin değil.

Her dosya listesinin dosyalar, klasörler ve boş kısım için menüleri vardır ve
birden çok dosya aynı anda seçilebilir (**Ctrl+tık**, **Shift+tık**); bkz.
[Stage'leme ve commit'leme](staging-and-committing.md).

## Bir dosya açıkken orta panel

Değişen bir dosyaya tıklayın; görünümü Log'un yerini alır. `Esc` Log'u geri
getirir. Üst kenar boyunca bir düğme çubuğu uzanır:

![Diff olarak açık değiştirilmiş bir dosya, üstünde görünüm çubuğuyla: Dosya Görünümü, Blame, Geçmiş │ Tümü seçicisi, Düzenle, hunk gezinmesi, Satır, Bölme, Hunk, boşluk, Kaydır ve Harita](../images/diff.png)

| Düğme | Ne yapar |
|---|---|
| **Dosya** (Dosya Görünümü), **Diff** (Diff Görünümü), **Blame** | Panelin dosya için neyi göstereceğini seçer. |
| **Geçmiş │ Tümü** | Görünümün yanındaki [commit sütununun](#commit-sütunu) listesini seçer ve sütunu gösterir. |
| **Düzenle** | Yerleşik düzenleyiciyi çalışma kopyasında açar. |
| `↑` `↓` | Önceki veya sonraki hunk. |
| **Satır**, **Bölme** | Tek sütun (satır içi) veya yan yana. |
| **Hunk** | Yalnızca değişen hunk'lar veya tüm dosya. |
| **WS** | Yalnızca boşluk değişikliklerini göster veya gizle. |
| **Kaydır** | Uzun satırları kaydırır. |
| **Harita** | Kaydırma çubuğunun yanında tüm dosyanın bir mini haritası. |
| **Kodlama** | Dosyanın okunduğu kodlama (önce **Otomatik**). |

Çalışma ağacının bir diff'inde her hunk'ın **Hunk'ı Stage'le** ve **Hunk'ı At**
düğmeleri vardır. Satır seçmek için `v` tuşuna basın, ardından yalnızca o
satırları stage'leyin veya atın.

![Bir dosyanın Blame'i: sözdizimi renkli kod ve sağda her hunk için avatar, yazar, yaş ve özet içeren bir ek açıklama sütunu, ayrıca yazar ve yaş şeritleri](../images/blame.png)

![Dosya geçmişi: solda dosyanın commit'leri, iki satır hâlinde, sağda seçili commit'in diff'i](../images/file-history.png)

Her iki görünüm de [Blame](log-and-history.md#blame) ve [Dosya geçmişi](log-and-history.md#dosya-geçmişi) bölümlerinde anlatılır.

Yerleşik **düzenleyicide** sözdizimi renkleri, satır numaraları, geri alma ve
bulma vardır; durum çubuğu satırı, sütunu, satır sonunu, girintiyi ve dili
gösterir.

![Sözdizimi renklendirmesi ve satır numaralarıyla bir Rust dosyasını gösteren yerleşik düzenleyici](../images/editor.png)

Bir resim dosyası resim olarak gösterilir: diff'te öncesi ve sonrası, Dosya
görünümünde dosyanın kendisi. Bkz. [Görseller](image-view.md).

Resim olmayan bir ikili dosya (PDF, belge, arşiv), sistem türü için uygulamalar
biliyorsa bir **Şununla aç** düğmesi gösterir: bkz. [Şurada aç](open-in.md).

## Commit sütunu

Bir commit'in dosyası diff olarak, Dosya Görünümü'nde veya Blame'de açıkken, solunda tek bir sütun durabilir (çalışma kopyasını düzenleyen yerleşik düzenleyicinin yanında asla durmaz: düzenleyici açılınca sütun gider, düzenleyici diff'e doğru kapanınca geri gelir). Üç durumu vardır:

- **Geçmiş**: yalnızca o dosyanın commit'leri, iki satırda bir ([Dosya geçmişi](log-and-history.md#dosya-geçmişi)). Sağ taraf seçili commit'teki dosya sürümünü gösterir.
- **Tümü**: Log'daki commit'lerin grafiğiyle birlikte sıkıştırılmış bir listesi, aşağıda anlatılıyor.
- **Gizli**: görünümün solundaki ince bir `›` şeridi sütunu geri getirir (ipucu tuşu söyler).

Görünümün düğme çubuğu her zaman sütunun üstünde tüm genişlik boyunca uzanır; bu yüzden sütun açılınca, kapanınca ya da liste değişince hiçbir düğme yer değiştirmez. Çubuktaki **`Geçmiş │ Tümü`** seçicisi (tek bir denetimin iki parçası, genişliği hep aynı) listeyi seçer: bir parçaya tıklamak o listeyi gösterir, sütun gizliyse onu açar; sütun görünürken zaten seçili olan parçaya tıklamak sütunu gizler. Ekrandaki listenin parçası vurgulanır; sütun gizliyken geri geleceği listenin parçasının altı çizilidir ve liste korunur. İpucu tıklamayı, seçili parçaya ikinci tıklamayı ve `Alt+C`'yi söyler. Dosya geçmişi her zamanki yerlerden de açılır (dosya menüleri, paletteki `Dosya Geçmişi: ...`).

![Bir commit'in dosya diff'inin yanında Tümü durumundaki commit sütunu; başlığında Ref, Grafik, Mesaj, Yazar ve Tarih var, Ref bölümünde dal ve tag etiketleri görünüyor](../images/commit-column-all.png)

- **Göster veya gizle**: `Alt+C` (palette `Görünüm: Commit sütununu aç/kapat`), seçicinin seçili parçasına tıklama, sütundaki ✕ ya da `›` şeridi. Palette `Görünüm: Commit sütunu: gizle`, `...: dosya geçmişi` ve `...: tüm Commit'ler` bir durum seçer. Seçim, o andaki listesiyle birlikte sütunun diğer ayarlarıyla hatırlanır. Seçim yoksa Tümü listesi için terminalin genişliği karar verir: 140 sütun genişliğinden itibaren görünür. Yanındaki görünüme 30 sütundan az kalacaksa sütun çizilmez; Tümü grafiği de çalışma ağacının diff'i veya bir karşılaştırma için çizilmez (dosya geçmişi çizilir: commit gerektirmez).
- **Genişlik**: sağ kenarını sürükleyin (aydınlık kenar işaretçiyi izler) ya da `Görünüm: Commit sütununu genişlet` ve `Görünüm: Commit sütununu daralt` kullanın. Merkez sütunun en fazla %70'ini alabilir (kalanı görünümün kalır, görünüm asla 30 sütunun altına bırakılmaz). Diğer boyutlarla birlikte hatırlanır.
- **Commit seçmek**: üzerine tıklayın ya da sütuna klavyeyi verin (`Ctrl+N` ve rozeti, veya `Görünüm: Commit sütununa odaklan`) ve `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` kullanın. `Enter` veya `Right` commit'in dosya listesine geçer, `Esc` klavyeyi görünüme geri verir. Sağ bölme seçimi izler: commit'in ayrıntıları ve değişen dosyaları; böylece onun başka bir dosyasını açabilirsiniz.
- **Görünüm ne yapar**: açık dosya açık kalır. Seçilen commit onu değiştirdiyse görünüm o commit'teki diff'ini gösterir. Değiştirmediyse dosyayı o zamanki haliyle, başlıkta "bu commit'te değişmedi" ile gösterir. Commit'te böyle bir dosya yoksa görünüm bunu söyler ve kullanılabilir kalır. Blame, Blame olarak kalır; seçilen commit'in durumuna göre.

Tümü listesi bir başlığın (Log'un kendi sütun adları) altında durur. Satır başına bir commit: grafik şeritleri, başlık (mesaj), yazarın adı ve kısa bir tarih; açarsanız bir de dal ve tag etiketleri. Bu Log'un kendi listesidir; aynı commit'ler aynı sırada gelir ve kaydırdıkça eskileri yükler. Şeritler Log'un Grafik sütunu gibi çizilir: resim gösteren bir terminalde (bkz. [Terminal seçmek](terminals.md)) birbirine bağlı şeritler ve yuvarlak noktalar olarak, Grafik'te avatar ayarı açıksa dairelerin içinde yazarların resimleriyle; diğer terminallerde karakterlerle. Varsayılan olarak şeritler gereken yeri alır ve başlık her zaman birkaç kelimelik yerini korur: dar bir sütunda şeritler kesilmeden önce yazar, ardından tarih yer bırakır. En az genişliğe, tek commit noktasına küçültülünce Grafik bölümü Log'daki gibi kompakt olur: her commit tek bir şeritte, dal veya merge çizgisi olmadan. Bölmenin kestiği bir bölüm (yana kaydırılmış satır ya da odadan geniş bir Grafik bölümü) resimlerini ve avatarlarını korur, kenarda kırpılır. Tekerlek imlecin altındaki listeyi kaydırır.

- **Üzerine gelme ve menü**: bir satırın üzerinde durmak, işaretçinin altındaki kısım için Log'un ipucunu gösterir (başlıkta mesajın tamamı, yazarın adı ve e-postası, tam tarih). Bir satıra sağ tıklamak (ya da sütunda klavye varken menü tuşu, `Shift+F10`; palette `Görünüm: Commit sütunu: seçili satırın menüsü`) menüsünü açar: *Log'da göster*, dosyanın o commit'teki diff'i, dosyanın o commit'teki hali, dosyanın o commit'teki blame'i, ardından o commit için Log'un kendi menüsü (kopyala, cherry-pick, dal ve bir çizginin altında revert gibi bir şeyleri yeniden yazan satırlar). İlk dört satırın her birinin bir palet girdisi de vardır (`Görünüm: Commit sütunu: ... göster`). Geçmiş listesinde satırlar Dosya geçmişinin kendi menüsünü ve aynı ipuçlarını taşır.
- **Bölümleri boyutlandırma ve yana kaydırma**: soldaki bölümü boyutlandırmak için başlıktaki bir sınırı sürükleyin: şeritler, mesaj, yazar, tarih, ref'ler, değişim veya SHA. Son bölümün de bir sınırı vardır: hemen ardındaki hücre (satır yeri doldurduğunda sütunun kendi kenarlığı), aynı şekilde aydınlanır ve o son bölümü boyutlandırır. Log'da olduğu gibi (`ui.column_resize`) sınır, varsayılan olarak iki bölüm arasındaki ayırıcı hücredir; Grafik'in satır tonu orada biter (ton içinden geçer, bu yüzden onunla sonraki bölüm arasında düz bir hücre kalmaz): başlıkta içinde bir işaret durur ve imleç üzerindeyken ya da onu sürüklerken sütun boyunca ince bir çizgi çizilir; çizginin herhangi bir noktasından sürüklemek yeniden boyutlandırır; `header` ile sınır, sonraki adın ilk harfidir ve işaretçi üzerindeyken aydınlanır (çizgi çizilmez, sürüklemenin başlayabileceği tek yer başlıktır). Bir şeyi boyutlandırana kadar mesaj diğerlerinin bıraktığı yeri alır; ilk sürükleme her bölümü, mesaj dahil, sahip olduğu genişliğe sabitler; böylece bir bölümü boyutlandırmak ondan sonraki bölümleri kaydırır, başka hiçbir genişliği değiştirmez. Boyutlandırdığınız bir bölüm korunur, yer açmak için asla küçültülmez: satır sütundan genişse Shift+tekerlek, yatay tekerlek ya da sütunda klavye varken `Shift+Left` / `Shift+Right` ile yana kayar; alt kenarındaki bir kaydırma çubuğu nerede olduğunuzu gösterir. Klavyeden: `Tab` bölümü seçer (gösterilenler arasında), `]` genişletir, `[` daraltır, `0` tüm bölümleri otomatik genişliğe döndürür (palette `Görünüm: Commit sütunu: sonraki bölümü seç`, `... bölümü genişlet`, `... bölümü daralt`, `... bölüm genişliklerini sıfırla`, `... sola kaydır` ve `... sağa kaydır` vardır). Genişlikler hatırlanır. Şeritler bir kaydırmayla kesildiğinde, resim gösteren bir terminalde bile karakterlerle çizilir.
- **Bölümleri seçme**: başlığa sağ tıklamak, bölüm başına bir onay kutusu olan bir menü açar; Log'un sunduğu her sütun için: Ref, Grafik, Mesaj, Yazar, Tarih, Değişim ve SHA. Mesaj her zaman gösterilir. **Ref**, **Değişim** ve **SHA** varsayılan olarak kapalıdır; gösterilince Ref commit'in dal ve tag etiketlerini, Değişim dosyalarını ve `+eklenen -silinen` satırlarını, SHA kısa hash'ini taşır, hepsi Log'un kendi sütunları gibi. Ref gibi Değişim ve SHA de yer açmak için asla küçültülmez: satır bunun yerine yana kayar. Klavyeden `Görünüm: Commit sütunu: sütunları seç` aynı menüyü açar (oklar ve `Enter` satır seçer); `Görünüm: Commit sütunu: Ref sütununu aç/kapat`, `... Grafik sütununu aç/kapat`, `... Yazar sütununu aç/kapat`, `... Tarih sütununu aç/kapat`, `... Değişim sütununu aç/kapat` ve `... SHA sütununu aç/kapat` bir bölümü hemen çevirir. Gösterilen bölümler, sıraları ve genişlikleri yalnızca sütuna aittir: `config.toml` içine yazılır (`ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths`); Log'un kendi sütunlarından ayrıdır, onlar bunlarla asla değişmez, tersi de geçerlidir.
- **Bölümleri taşıma**: bir bölümü oraya taşımak için başlıktaki bir adı başka bir adın üzerine sürükleyin, Log'da olduğu gibi (satır işaretçiyi izler; `ui.column_resize = "header"` ile bir adın ilk harfi boyutlandırma sınırıdır, bu yüzden adın geri kalanından basın). Her bölüm her yere gidebilir, şeritler ve Ref etiketleri dahil. Klavyeden `Tab` bölümü seçer, `<` / `>` onu gösterilenler arasında bir yer sola veya sağa taşır (palette `Görünüm: Commit sütunu: bölümü sola taşı` ve `... bölümü sağa taşı` vardır). Sıra hatırlanır.

  ![Commit sütunu başlığının sağ tık menüsü: Ref, Grafik, Mesaj (her zaman gösterilir), Yazar, Tarih, Değişim ve SHA için birer onay kutusu](../images/commit-column-menu.png)

Fare hareketleri [Fare ve hareketler](mouse-and-gestures.md#commit-sütunu-tümü-listesi) sayfasında da listelenir.

## Boyutlar hatırlanır

GitPoulpi sürüklediğiniz her boyutu sekme geçişleri ve yeniden başlatmalar
boyunca saklar. Boyutlar her depo için aynıdır. Bu; Refs kenar çubuğu, Log ve
sağ panel arasındaki ayırıcıları, her Refs bölümünün yüksekliğini, commit
mesajı bloğunu, Log sütun genişliklerini, merge düzenleyicisi bölmelerini ve
klasör seçici ayırıcısını kapsar. Boyutlar pencerenin parçaları olarak
saklanır; bu yüzden geniş bir terminalde yapılan yerleşim dar birine de uyar.

Tüm boyutları sıfırlamak için `Gezinme: Yerleşimi Sıfırla` palet komutunu
çalıştırın. Bazı boyutların kendi sıfırlaması da vardır: `Gezinme: Refs Bölüm
Boyutlarını Sıfırla`, `Commit Ayrıntıları: Mesaj Boyutunu Sıfırla` ve merge
düzenleyicisinde bir ayırıcıya çift tıklama veya `Alt+0`.

## Katmanlar ve özel ekranlar

- **Komut paleti** (`Ctrl+P`): her komutun üstünde bir filtre kutusu. Her satır
  kısayolunu gösterir. Şu an çalışamayan bir komut grileşir ve nedeni altta
  gösterilir. Komutların `Git:`, `Refs:`, `Gezinme:`, `Diff:`, `Çalışma
  Değişiklikleri:`, `Commit:`, `Görünüm:`, `Depo:`, `Sekme:`, `LFS:`, `Pull
  Request'ler:`, `Issue'lar:` veya `YZ:` gibi bir grup öneki vardır. Grubunu
  görmek için bir önek yazın.
- **Seçiciler**: bazı palet komutları ikinci bir liste açar; örneğin
  `Uygulama: Temayı Değiştir...`, `Uygulama: Dili Değiştir...` ve
  `Uygulama: Grafik Renk Temasını Değiştir...`. Listede ilerledikçe pencere her
  seçeneği canlı gösterir. `Enter` seçimi tutar. `Esc` eski seçimi geri koyar.
  Tema listesinde **Tümü**, **Koyu**, **Açık** ve **Renkli** filtreleri vardır.
  Bkz. [Temalar](themes.md).
- **Yardım** (`?`): sekmeler. **Tuşlar** her tuşu ve hareketi listeler; diğer
  sekmeler (Başlangıç, Log, Değişim, Refs, Barındırma, Ayarlar, YZ ve CLI)
  eylemlere ve her birine nasıl ulaşılacağına dair kısa kılavuzlardır. Sekmeyi
  `Tab` / `Shift+Tab`, `Left` / `Right`, `Alt+1`..`Alt+8` veya bir tıklamayla
  değiştirin. Geçerli sekmeyi süzmek için yazın. `Ctrl+D` ya da alttaki
  bağlantıya tıklamak, bulunduğunuz sekmenin belge sayfasını açar.

  ![Yardımın Barındırma sekmesi: her eyleme nasıl ulaşılacağını anlatan kısa bir kılavuz](../images/help-hosting.png)

  ![Yardım katmanı: Ctrl+P, Ctrl+N, Ctrl+G, ? ve hareket tuşlarını listeleyen Genel bölümü, ardından Fare temelleri bölümü](../images/help.png)
- **Arama çubuğu** (`f`): Log'un sağ üstünde. `n / N` eşleşmeyi gösterir.
- **Bağlam menüleri**: sağ tıklayın veya seçili satırda `.` tuşuna basın.
  `›` olan bir satır alt menü açar.
- **İletişim kutuları**: onaylar, bir dal veya tag adı, Klonla, Init, Pull
  Request Oluştur, reddedilen bir push'tan sonraki seçenekler ve diğerleri.
  `Enter` kabul eder, `Esc` iptal eder.
- **Etkileşimli rebase** ve **commit composer**, Log'un yerine bir inceleme
  ekranı getirir. Bkz.
  [Log ile çalışmak](log-and-history.md#etkileşimli-rebase) ve
  [Stage'leme ve commit'leme](staging-and-committing.md#commit-composer-değişikliklerinizi-commitlere-bölün).

  ![Önerilen dört commit'i, her birinin dosyalarıyla birlikte inceleyen commit composer; Sıfırla, İptal ve 4 commit oluştur düğmeleri](../images/composer.png)

- **Ayarlar** (`Ctrl+,` veya başlık çubuğundaki dişli): solda bir sayfa listesi, sağda sayfanın
  seçenekleri. Bkz. [Ayarlar](settings.md).

  ![Ayarlar: solda üstünde arama kutusu olan sayfa listesi, sağda Genel sayfası](../images/settings.png)

- Uzun yanıtlar için **açılır pencereler**; örneğin YZ **Değişiklikleri
  açıkla** penceresi (`↑` `↓` kaydır, `c` kopyala, `Esc` kapat).

  ![Bir YZ açıklaması ve tekrar kontrol edilecek şeylerin listesiyle Çalışma değişikliklerini açıkla penceresi](../images/ai-explain.png)

Açılır pencerelerdeki tüm listeler aynı şekilde çalışır. Bkz.
[Klavye kısayolları](keybindings.md) ve [Fare ve hareketler](mouse-and-gestures.md).

## Bu belgelerde kullanılan sözcükler

| Sözcük | Anlamı |
|---|---|
| Panel | Üç sütundan biri: Refs kenar çubuğu, Log (veya açık dosya), sağ panel. |
| Hap | Ref sütunundaki bir dal veya tag etiketi. |
| WIP satırı | Log'un en üstündeki çalışma değişiklikleri satırı. |
| Hunk | Bir diff'teki değişen satırların tek bir bloğu. |
| Composer | Commit formu ve değişiklikleri commit'lere bölen YZ incelemesi. |
| Akor | Bir tuşu başka bir tuşun izlediği dizi; örneğin `Ctrl+G`, ardından `s`. |
| Önek | `Ctrl+G` (Git eylemleri) veya `Ctrl+N` (gezinme). |
| Rozet | `Ctrl+N`'in bir denetimin yanında gösterdiği harf. O denetime odaklanmak için basın. |
