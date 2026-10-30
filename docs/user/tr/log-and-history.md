[English](../log-and-history.md) · [Čeština](../cs/log-and-history.md) · [Deutsch](../de/log-and-history.md) · [Español](../es/log-and-history.md) · [Français](../fr/log-and-history.md) · [Italiano](../it/log-and-history.md) · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · [Português (Brasil)](../pt-BR/log-and-history.md) · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · **Türkçe** · [Українська](../uk/log-and-history.md) · [简体中文](../zh-CN/log-and-history.md)

# Log ile çalışmak

Log, GitPoulpi'nin merkezidir: tüm geçmiş bir grafik olarak, üzerinde her dal,
tag ve stash ile. Bu sayfa onu okumayı, içinde bir şey bulmayı ve commit'lere
ile ref'lere yapabileceğiniz her şeyi kapsar. Alanların adları [arayüz
turunda](interface-tour.md) verilir; tam tuşlar [Klavye
kısayolları](keybindings.md) sayfasında, fare hareketleri ise [Fare ve
hareketler](mouse-and-gestures.md) sayfasındadır.

![Bir merge commit'inin seçili olduğu Log: ayrıntıları ve dosyaları sağda](../images/log-graph.png)

## Grafiği okumak

Her satır bir commit'tir, en yenisi en üstte. Commit'in şeridinde bir nokta
durur; çizgiler bir commit'i üst commit'lerine bağlar; bir merge, birleşen iki
çizgi olarak görünür. Soldaki haplar o commit üzerindeki dal ve tag'leri şeridin
renginde, noktaya uzanan ince bir çizgiyle adlandırır. Dizüstü simgesi yerel
bir dalı, küre uzak bir dalı, `✓` ise checkout ettiğiniz dalı işaretler. `+N`,
o commit'e daha fazla ref'in işaret ettiği anlamına gelir; listelemek için
üzerine gelin veya aşağıdaki menüyü kullanın.

İlk satır **çalışma değişiklikleridir** (kesik çizgili bir daire: commit'lenmemiş
işiniz, üzerinde durduğu commit'e bağlı). Stash'ler, yapıldıkları
commit'in yanında bir şeritte küçük kareler olarak görünür.

### Renkler, satır tonu ve avatarlar

Şerit renkleri temadan gelir. *Ayarlar > Görünüm > Grafik renk teması* herhangi
bir temayı herhangi bir şerit paletiyle birleştirir; paletteki *Uygulama:
Grafik Renk Temasını Değiştir...* seçicisi (`Ctrl+N Shift+Y`) her paleti
canlı önizler; bkz. [Temalar](themes.md). *Grafik şerit arka planı*
**Satır tonu** olarak ayarlandığında grafik sütununun arkasına şerit renginin
soluk bir bandı ve kenarına ince bir çubuk çizilir. *Satır tonu gücü*
(**Güçlü**, **Orta** (varsayılan), **Hafif**) bandın ne kadar soluk olacağını
belirler.

| Satır tonlu Serie şerit paleti | Satır tonlu Material şerit paleti |
|---|---|
| ![Serie renk temasında satır tonlu Log grafiği](../images/theme-serie.png) | ![Material renk temasında satır tonlu Log grafiği](../images/theme-material.png) |

*Yazar avatarları* (grafik noktasında veya Yazar sütununda bir resim)
varsayılan olarak kapalıdır. Açmak, her yazarın e-posta adresinin bir hash'ini
Gravatar'a gönderir ve barındırma sağlayıcınıza sorar. Aynı sayfadaki *Avatar
kaynakları* ve *Avatar yedeği*, resimlerin nereden geleceğini ve hiçbiri
olmadığında ne çizileceğini seçer.

Terminaliniz resim gösteremiyorsa grafik karakterlerle çizilir. *Grafik
çizici* ve *Görüntü protokolü* birini veya ötekini zorlar; bkz. [Sorun giderme
ve SSS](troubleshooting.md).

Seçili veya üzerine gelinen satırın vurgusu commit'in dairesinden başlar.
Solundaki şeritler normal arka planı korur.

### Sütunlar

Sütunları seçmek için başlığa sağ tıklayın: **Ref**, **Grafik**, **Mesaj**
(her zaman açık), **Yazar**, **Tarih**, **Değişim** ve **SHA**. İsteğe bağlı
**Değişim** sütunu her commit için `n files +eklenen -silinen` gösterir. Bir
sütunu yeniden boyutlandırmak için iki başlık arasındaki kenarlığı sürükleyin;
seçim hatırlanır ve terminal hücresi cinsindendir. Varsayılan `ui.column_resize = "line"` ile kenarlık, bir sütunun sonundaki bir hücrelik boşluktur: başlık satırında içinde küçük bir işaret durur; imleç boşluğun üzerindeyken (başlıkta ya da tablonun herhangi bir yerinde) tablonun tüm yüksekliği boyunca ince bir vurgu çizgisi çizilir ve boyunca herhangi bir yerden sürüklemek yeniden boyutlandırır (destekleyen terminallerde imleç yeniden boyutlandırma imlecine dönüşür). `"header"` ile işaret ve çizgi yoktur: kenarlıktan sonraki başlık hücresinin ilk karakteri üzerine gelince aydınlanır ve sürükleme yalnızca başlık satırından, ondan başlar (Ayarlar > Log ve grafik > Sütun yeniden boyutlandırma). Satırlar iki kipte de aynı görünür: boşluk solundaki sütunun parçasıdır, bu yüzden seçim ve satır tonları içinden geçer. Ref, Grafik ve diğer sütunlar pencere
yeniden boyutlandırıldığında ya da büyütüldüğünde genişliğini korur: yalnızca **Mesaj**
büyür veya küçülür (çok dar bir pencerede diğer sütunlar geçici olarak yol verir, önce
Grafik, ve pencere yeniden büyüyünce sizin genişliklerinize döner). Grafik sütununu en az genişliğine, tek commit noktasına kadar daraltın: grafik kompakt olur, her commit tek bir şeritte alt alta durur, dal veya merge çizgisi olmaz. Bir hücre daha geniş yapınca ağaç geri gelir.

Bir başlığı sürüklemek, imleci izleyen adının bir hayaletini ve sütunun ineceği yerde bir vurgu çubuğu gösterir; sıra bıraktığınızda değişir, `Esc` iptal eder. Sağ tıklamayla açılan seçici, başlığında ✕ bulunan bir onay listesidir (`[x]` / `[ ]`); palette de bulunur (`Görünüm: Log: Gösterilecek sütunları seç`), oklarla gezilir ve `Space` / `Enter` ile açılıp kapatılır.

Klavyeden `Shift+C` (veya paletteki `Görünüm: Log sütunu: taşımak veya boyutlandırmak için bir sütun seç`) bir sütun seçer ve başlık hücresi yanar. Sonra `Tab` / `Shift+Tab` başka bir sütun seçer, `<` / `>` onu gösterilen sütunlar arasında bir yer sola veya sağa taşır (başlığını sürüklemenin klavyedeki karşılığı), `]` / `[` onu genişletir veya daraltır (sınırını sürüklemenin karşılığı; son sütun sol kenarını oynatır), `0` her genişliği otomatiğe döndürür, `Esc` veya `Enter` çıkar. Bu arada oklar commit'ler arasında gezmeye devam eder. Sıra ve genişlikler bir sürüklemeninkiler gibi hatırlanır ve Log'a özeldir: bir diff'in yanındaki commit sütunu kendi listesini korur.

![Değişim sütunu gösterilen Log: her commit dosya sayısını ve eklenen ile silinen satırları listeler](../images/log-changes-column.png)

### Yazarlar ve `.mailmap`

Deponun bir `.mailmap` dosyası varsa (git'in, birkaç adla commit yapmış bir kişiye tek bir ad
ve e-posta veren dosyası), Log'un **Yazar** sütunu, commit ayrıntıları, Blame ve Dosya
Geçmişi `git log` gibi **kanonik** kimliği gösterir. Commit'lerin kendisine dokunulmaz.
`log.use_mailmap` ayarı (Ayarlar > Log ve grafik, varsayılan açık) saklanan adları göstermek
için bunu kapatır. Yazara göre arama gösterilen ad ve e-postayla eşleşir; bu yüzden kanonik
adı aramak kişinin her commit'ini bulur. Geçmiş ekranının **Kimlikler** sekmesi geçmişin
adlarını ve e-postalarını listeler ve sizin için `.mailmap` satırları yazar, bkz.
`history-rewrite.md`; terminalden: `gitpoulpi mailmap list|check|add`.

## Bir şey bulmak

- **Arama** (`f`, **Ara** araç çubuğu düğmesi veya paletteki *Gezinme:
  Commit'lerde Ara...*): commit mesajının tamamını, yazar adını ve e-postasını,
  hash'i ve ref adlarını eşleştirir. Eşleşmeyenleri soluklaştırır ve grafiğin
  biçimini korur. `Enter` / `Shift+Enter` sonraki / önceki eşleşmeye atlar.
- **Refs filtresi** (kenar çubuğunda `/`): dal, tag ve uzak depo listelerini
  ada göre daraltır.
- **Solo** (`o` tuşu veya bir ref, klasör ya da uzak depo üzerindeki ref'in
  menüsü): yalnızca solo yapılan ref'lerin commit'lerini gösterir; birleşimlerini
  görmek için birkaçını solo yapın. `Shift+O` solo'yu durdurur.
- Bir ref'i grafikten çıkarmak için **gizleyin** (menüsünden); Refs başlığındaki
  *Gizli n* bunları sayar, **Tümünü Göster** (`Shift+H`) geri getirir.

## Dallar, tag'ler ve Refs kenar çubuğu

Checkout etmek için bir yerel dala veya tag'e çift tıklayın. Menüsü için bir
ref'e sağ tıklayın (kenar çubuğunda ya da Log'daki hapında). En işe yarar
girdiler:

| Girdi | Yaptığı |
|---|---|
| Grafikte göster | Log'u ref'in commit'ine kaydırır (gerekirse önce daha eski geçmişi yükler) ve satırı yakıp söndürür; seçim kalır. Ayarlar'da *Bir ref'e tıklamak onu grafikte gösterir* kapalı değilse bir ref'e tek tıklamak aynı şeyi yapar. Grafikten (veya Solo ile) gizlenmiş bir ref bunu bunun yerine durum çubuğunda söyler |
| Checkout | Dala geçer veya tag'de ayrık HEAD'e geçer |
| Merge, Rebase üstüne | Checkout yapılmış dalı bu ref'e merge eder veya bu ref'in üstüne rebase eder |
| Fast-forward: `<dal>` → `<geçerli>` / `<geçerli>` → `<dal>` | Bu bir fast-forward ise bir dalı ileri taşır; değilse nedeniyle birlikte grileşir (`ayrışmış, fast-forward değil`) |
| Worktree oluştur: ... | Bağlı bir worktree'yi kendi sekmesinde açar |
| Pull, Push, Fetch, upstream'i ayarla... | O dal üzerinde ağ işlemleri |
| Grafikte gizle / göster, Solo | Yalnızca görüntü; depoya dokunmadan bir ref'i gizler veya solo yapar |
| Adını değiştir, Sil | Sil önce sorar; uzak karşılığı da silinebilir |
| Dal adını kopyala, Uzak depodaki dal bağlantısını kopyala | Panoya |

**Bir tag nerede durur.** Bir tag bu bilgisayarda, bir uzak depoda ya da ikisinde birden olabilir;
kenar çubuğunun TAG'LER satırları (ve Log'daki tag hapı) hangisi olduğunu addan sonraki bir işaretle
söyler. Aynısını sözcüklerle görmek için bir tag'in üzerine gelin.

| İşaret | Anlamı |
|---|---|
| yukarı ok | Yalnızca burada: hiçbir uzak depoda yok. Menüsü *Push* sunar |
| sol-sağ ok | Burada ve bir uzak depoda, aynı commit'te |
| aşağı ok (satır soluk) | Yalnızca bir uzak depoda: bu depoda yok (hiç fetch edilmemiş ya da burada silinmiş). Menüsü *`<tag>` tag'ini `<remote>` deposundan fetch et* ve *`<tag>` tag'ini `<remote>` üzerinde sil...* sunar; paletteki karşılığı `Git: Fetch Selected Tag` |
| yukarı-aşağı ok | Bir uzak depoda da var, ama farklı bir commit'te (push reddedilirdi) |

git tag'ler için uzak izleme ref'i tutmaz, bu yüzden GitPoulpi her uzak deponun neye sahip olduğunu son
**fetch** işleminizden (otomatik olan dahil, her fetch) ve kendi tag push ile silmelerinden öğrenir. Gezinirken
ağa hiçbir şey sorulmaz; biri bir uzak deponun tag'lerini değiştirdiyse görmek için fetch edin. Aynı bilgi
`gitpoulpi tag list` çıktısındadır (`local`, `remote`, `differs_on`, `where` alanları).

**Bir tag'i silmek.** Bir uzak depoda tag varsa, kırmızı **Sil** üç numaralı satırlı bir pencere açar
(`1`-`9` ya da Yukarı/Aşağı ve `Enter`; `Esc` iptal eder) ve seçili satır altında kendini açıklar:

1. **Yerelde sil**: yalnızca bu depodan kaldırır. Hemen çalışır ve iletisi uzak depoda hâlâ olduğunu söyler.
   **Sonraki fetch onu geri getirir** (git uzak deponun tag'lerini izler); uzak depoda da silmediyseniz.
2. **`<remote>` üzerinde sil**: orada herkes için kaldırır (`git push <remote> --delete`). Uzak depoyu
   adlandıran bir onay ister. Yerel tag'iniz kalır. Bu geri alınamaz.
3. **Yerelde ve `<remote>` üzerinde sil**: aynı onaydan sonra ikisi birden; bu yüzden bir fetch onu geri
   getiremez. **Geri alma yalnızca yerel tag'i geri yükler** (aynı tag nesnesi); o zaman yeniden yalnızca
   yereldir ve *Push* onu uzak depoya geri koyar.

![Üç satırlı Tag sil iletişim kutusu: Yerelde sil, origin üzerinde sil ve Yerelde ve origin üzerinde sil; altında refs panelinin Tag bölümü, burada her tag'in yalnızca yerel mi yoksa origin'de de mi olduğunu gösteren bir işareti var](../images/tag-delete.png)

Birden çok uzak depoda her birinin kendi satırları olur. Yalnızca burada olan bir tag düz onayı korur;
yalnızca bir uzak depoda olan bir tag doğrudan o uzak deponun onayına gider.
CLI: `gitpoulpi tag delete <name> [--local | --remote <r> | --both [--remote <r>]]`.

**Fetch ve tag'ler.** Bir fetch her uzak deponun sahip olduklarını (işaretleri) da yeniler ve git'in kendi tag
izleme davranışı geçerli kalır: yalnızca burada silinen bir tag yeniden fetch edilir. Birinin **uzak depoda
sildiği** bir tag, herhangi bir fetch sonrasında işaretin uzak taraftan kaybolur, ama git varsayılan olarak
tag'leri prune etmediği için burada yerel tag'iniz olarak (artık *yalnızca burada*) kalır. **Ayarlar >
Eşitleme > Tag'ler otomatik temizleme** (varsayılan olarak kapalı; `gitpoulpi fetch --prune-tags`) böyle bir
yerel tag'i de siler, ama yalnızca uzak depoda bulunmuş olanı: hiç push etmediğiniz ya da taşıdığınız bir
tag'e asla dokunulmaz. Neyin temizlendiğini öğrenmek için git 2.41 veya daha yenisi gerekir.

**Yeni dal veya tag.** **Dal** ve **Tag** araç çubuğu düğmeleri, `Ctrl+G b` ve
`Ctrl+G t` ya da bir commit'in *Burada dal oluştur...* girdisi adı, Ref
sütununda hedef commit'in satırındaki satır içi bir kutuda sorar. `Enter`
oluşturur, `Esc` iptal eder ve git'in reddedeceği bir ad kutuyu kapatmadan
kırmızıyla bildirilir.

Ref'i olmayan bir commit'te, satırının üzerine gelmek Ref sütununda ait olduğu dalın (yoksa uzak dalın) adını taşıyan soluk bir hap da gösterir; ad geçmişten çıkarılır, tag'ler asla hayalet olarak gösterilmez. Bir merge'den sonra silinen dal, merge commit'inin konusundaki adın (`Merge branch 'x'`, `Merge pull request #N from owner/x`...) altında yine üstü çizili görünür; squash merge'ler, rebase'ler ve fast-forward'lar böyle bir iz bırakmaz. Hapın üzerine gelmek türünü gösterir. Hiçbir şey oluşturmaz: yukarıdaki dal oluşturma komutunu kullanın. Ayarlar > Log ve grafik içindeki **Üzerine gelinen Commit için soluk bir dal göster** onu kapatır; Ref sütunu çok dar olduğunda hiçbir şey çizilmez.

**Uzak depolar.** UZAK başlığındaki `+` bir tane ekler; bir uzak deponun adına
sağ tıklayarak onu fetch edebilir, web bağlantısını kopyalayabilir, varsayılan
uzak depo yapabilir, tüm dallarını gizleyebilir, URL'lerini düzenleyebilir veya
kaldırabilirsiniz.

**Stash'ler.** **Stash** her şeyi stash'ler (izlenmeyen dosyalar dahil);
`Ctrl+G m` önce bir mesaj sorar (YZ açıkken o istemin içindeki `Ctrl+G` bir
tane önerir). **Pop**, **Apply** (stash kalsın) veya **Drop** için Log'da ya da
STASH'LER bölümünde bir stash satırına sağ tıklayın.

**Worktree'ler.** *Worktree oluştur: ...* (bir dalda, tag'de veya commit'te),
deponun yanında bir klasör öneren, isteğe bağlı yeni bir dal adı ve "Yeni
worktree'yi sekme olarak aç" seçeneği olan bir iletişim kutusu açar.
Worktree'ler WORKTREE'LER bölümünde görünür; açmak için birine çift tıklayın;
worktree kaldırıldığında sekmesi kendiliğinden kapanır.

## Ref'leri commit'lerin üzerine sürüklemek

Bir dala (kenar çubuğunda veya Log'daki hapında) basıp onu
sürükleyebilirsiniz. Bir hayalet etiket imleci izler ve geçerli hedefler
aydınlanır; bir menü almak için bırakın:

- bir yerel dalı **başka bir yerel dalın üstüne**: Merge, Rebase, Etkileşimli
  rebase, Fast-forward;
- bir yerel dalı **bir uzak dalın üstüne**: O uzak depoya Push;
- bir uzak dalı **bir yerel dalın üstüne**: Merge, Rebase, Fast-forward, Pull.

`Esc` sürüklemeyi iptal eder. Klavye karşılığı *Refs: Seçili Ref'i Checkout
Yapılmış Dala Bırak...* palet komutudur.

## Commit menüsü

Menüsü için bir commit'e sağ tıklayın (veya üzerinde `.` tuşuna basın):

![Commit menüsü: cherry-pick, revert, reset, mesajı düzenle, YZ ile açıkla, etkileşimli rebase, dal oluştur, tag, checkout, worktree, yamalar ve kopyalama eylemleri](../images/context-menu.png)

| Girdi | Yaptığı |
|---|---|
| **Cherry-pick: `<dal>` üstüne** | Commit'i checkout yapılmış dal üzerinde yeniden oynatır |
| **Commit'i revert et...** | Onu geri alan bir commit oluşturur. Bir merge commit'inde önce bir açılır liste hangi ebeveynin ana hat (kalan taraf) olduğunu sorar (1. ebeveyn, merge'ün yapıldığı daldır), sonra olağan onay gelir. Geri al önceki durumu geri getirir |
| **`<dal>` dalını bu commit'e reset et** | Reset türü için bir alt menü; hard reset onay ister |
| **Commit mesajını düzenle...** | Aşağıya bakın |
| **Değiştir...** | Commit'in mesajını ve yazarın adını, e-postasını, tarihini tek bir iletişim kutusunda değiştirir; bkz. [Bir commit'i değiştirmek](#bir-commiti-değiştirmek-mesaj-yazar-tarihler) |
| **Bu commit'i böl...** | Commit'i birkaça böler; bkz. [Commit'i bölme](#commiti-bölme) |
| **Bu commit'i böl (YZ)...** | Aynısı, gruplamayı model önerir; yalnızca [YZ](ai.md) açıkken |
| **Değişiklikleri açıkla (YZ)** | YZ açıklaması olan bir açılır pencere; yalnızca [YZ](ai.md) açıkken |
| **Buradan etkileşimli rebase...** | Aşağıya bakın |
| **Burada dal oluştur...**, **Burada tag oluştur...**, **Burada açıklamalı tag oluştur...** | Satır içi ad kutusu |
| **Bu commit'te checkout**, **Burada worktree oluştur...** | Ayrık HEAD veya bir worktree |
| **Bu commit'ten yama oluştur...**, **Buradan `<dal>` dalına yama oluştur...** | `.patch` dosyaları yazar; *Yamayı uygula* bir ref'in menüsündedir |
| **Uzak depodaki commit bağlantısını kopyala**, **Kopyala** > | Panoya (hash, mesaj ve daha fazlası) |

## Commit mesajını düzenlemek (reword)

*Commit mesajını düzenle...* (veya `Ctrl+G e`) mesajın tamamını açar: sayaçlı
tek satırlık bir özet, altında bir açıklama, **İptal** ve **Mesajı Güncelle**.
Checkout yapılmış dalın herhangi bir commit'i yeniden yazılabilir (HEAD veya bir
ata); üstündeki commit'ler kendi ağaçları, yazarları ve tarihleriyle yeniden
oluşturulur, merge'ler merge olarak kalır ve stage'lenmiş ile stage'lenmemiş
değişikliklerinize dokunulmaz. Commit zaten push edilmişse açılır pencere
kehribar renkli bir uyarıyla force push (lease ile) gerekeceğini bildirir ve ikinci
bir onay ister. `Ctrl+Z` (araç çubuğundaki Geri al) reword'ü geri alır; bkz.
[Geri al ve yinele](undo-redo.md). Durum çubuğu önceki ucu da adlandırır.

## Bir commit'i değiştirmek (mesaj, yazar, tarihler)

Commit menüsündeki *Değiştir...* (veya `Ctrl+G Shift+E`, ya da paletteki *Commit: Değiştir...*) commit'in özelliklerini tek bir iletişim kutusunda açar: özet, açıklama ile yazarın adı, e-postası ve tarihi; her biri commit'in kendi değerleriyle doldurulmuştur. İstediğinizi değiştirip **Uygula**'ya basın; yalnızca değiştirdiğiniz alanlar gönderilir. Tarih `YYYY-MM-DD HH:MM:SS +HH:MM` biçiminde yazılır ve siz düzenleyene kadar UTC olarak gösterilir. Committer sizsiniz, şu anki zamanla (reword'deki gibi). Checkout yapılmış dalın herhangi bir commit'i değiştirilebilir; kurallar, zaten push edilmiş commit uyarısı ve ikinci onay *Commit mesajını düzenle...* ile aynıdır; üstündeki commit'ler kendi yazar ve tarihleriyle yeniden oluşturulur, stage'lenmiş ile stage'lenmemiş değişiklikleriniz olduğu gibi kalır. `Ctrl+Z` (araç çubuğundaki Geri al) işlemi geri alır. Geçmiş yeniden yazma içeren derlemelerde iletişim kutusunun sol altındaki **Geçmişi daha kapsamlı değiştir...** (`Alt+H`) hiçbir şey uygulamadan onu kapatır ve [geçmişi yeniden yazma ekranını](history-rewrite.md) bu commit'ten en yenisine kadar kapsamla açar; bir alanı düzenlediyseniz önce sorar (`Esc` onaylar). `Ctrl+N` iletişim kutusunun her alanında ve düğmesinde bir rozet gösterir; imleci o alana koymak ya da o düğmeye basmak için rozete basın (bkz. [Rozetler](keybindings.md#rozetler)).

İletişim kutusuna her giriş yolu onu eksiksiz açar: her alan görünür ve düzenlenebilir; yalnızca imlecin yeri değişir. Commit ayrıntı panelinde (sağda) özete, açıklamaya veya yazar bloğuna sağ tıklayıp *Özeti değiştir...*, *Açıklamayı değiştir...* ya da *Yazarı değiştir...* seçin, veya özete, açıklamaya, yazara ya da yazım tarihine çift tıklayın: iletişim kutusu imleç o alandayken açılır. Klavye karşılıkları Log'da ve ayrıntı panelinde çalışır: `Ctrl+G Shift+E` (tüm alanlar), `Ctrl+G Shift+A` (yazar) ve paletteki *Commit: Değiştir...*, *Commit: Yazarı Değiştir...*, *Commit: Özeti Değiştir...* ile *Commit: Açıklamayı Değiştir...*. İki kısayol da yeniden atanabilir (`git_prefix.amend`, `git_prefix.edit_author`; bkz. [Klavye kısayolları](keybindings.md)). Bir betikten veya ajandan: `gitpoulpi amend <commit>` ile `--message` (veya `--summary` / `--description`), `--author-name`, `--author-email`, `--author-date` ve ilgili `--committer-*` seçenekleri (MCP aracının adı `amend`).

## Commit'i bölme

Commit menüsündeki *Bu commit'i böl...* (ya da palette *Git: Seçili Commit'i Böl...*) bir
commit'i en eskiden başlayarak iki ya da daha fazlasına böler. Checkout yapılmış dalın
merge olmayan her commit'i bölünebilir (HEAD veya bir ata, kök commit de). Bölme
düzenleyicisinin iki paneli vardır: solda **Yeni commit'ler** (her birinin bir mesajı var) ve
sağda commit'in **Değişiklikler**i; her dosya ve hunk için bir satır, her satırda onu alacak
yeni commit'i adlandıran bir `[N v]` alanı.

| Tuş | Fare | Ne yapar |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | bir satıra tıklama | değişikliklerin üzerinde gezinir (`Shift+Up` / `Shift+Down` seçimi genişletir) |
| `1`-`9` | satırın `[N v]` alanı, sonra bir seçim | seçili değişikliği o yeni commit'e verir; `0` atanmamış bırakır |
| `Enter` / `m` | `[N v]` alanı | seçili değişiklik için commit listesini açar |
| `n` | `+ Yeni commit` | seçili değişikliği yeni bir commit'e koyar |
| `v` | | **satır kipi**: bir hunk'ı satır satır böler (bir aralık seçmek için satırların üzerinde sürükleyin) |
| `Tab` | bir panele tıklama | commit'ler ile değişiklikler arasında geçiş yapar |
| `e` / `Enter` (commit'ler panelinde) | kalem | o commit'in mesajını düzenler |
| `d` / `Delete` | çarpı | yeni bir commit'i kaldırır (değişiklikleri atanmamış olur) |
| `Alt+Up` / `Alt+Down` | oklar | yeni bir commit'i yukarı veya aşağı taşır |
| `s` | onay kutusu | yeni commit'leri imzalar |
| `Shift+R` | `Sıfırla` | başlangıçtaki plana döner |
| `Ctrl+S` | `N parçaya böl` | böler |
| `Esc` | çarpı, `İptal` | çıkar; bekleyen düzenlemeler varsa ikinci basış onaylar |

Her değişiklik tam olarak bir commit'e gitmelidir; düzenleyici hâlâ *Atanmamış* olanları sayar
ve sayı sıfır olana kadar düğme reddedilmiş kalır. İkili olan, yalnızca kipini değiştiren,
commit'te yeni ya da silinmiş olan, alt modül ya da LFS dosyası olan bir dosya yalnızca bütün
olarak verilebilir. İlk commit özgün mesajla başlar; her yeni commit'e bir mesaj verin. Yeni
bir commit özgün yazarı ve tarihi korur; committer sizsiniz, şimdiki zamanla.

**Deponuzda dal dışında hiçbir şey değişmez.** Son yeni commit, özgün commit'in içeriğinin
tamamen aynısına sahiptir; bu yüzden üstündeki commit'ler aynı ağaçlar, yazarlar ve
mesajlarla yeniden oluşturulur (merge'ler merge olarak kalır) ve stage'lenmiş ile
stage'lenmemiş değişikliklerinize dokunulmaz: kirli bir çalışma ağacı sorun değildir ve
çakışma olamaz. Hook'lar çalışmaz. Bekleyen bir otomatik stash varken bölme engellenir.

**İmzalama.** *Commit'ler imzala* onay kutusu deponun `commit.gpgSign` değerinden başlar.
İşaretli değilse yeniden yazılan commit'lerin (yeniler ve üstlerindekiler) imzaları atılır ve
düzenleyici bunu onay kutusunun altında söyler. İşaretliyse hepsi imzalanır; imzalama
başarısız olursa hiçbir şey değişmez.

**Geri alma.** Tek bir `Ctrl+Z` (araç çubuğundaki Geri al) özgün commit'i geri getirir, kirli
bir çalışma ağacıyla da; bkz. [Geri al ve yinele](undo-redo.md). Zaten push edilmiş bir
commit sonrasında force push (lease ile) gerektirir.

**Etkileşimli rebase'den.** İnceleme ekranındaki her rebase satırında küçük bir **böl...**
düğmesi vardır (satırda `t`): o commit için aynı düzenleyiciyi açar ve bölme bitince (ya da
iptal edince) yeni geçmişle inceleme ekranına dönersiniz. Planı düzenlediyseniz, bu
düzenlemelerin kaybolacağını onaylamak için böl'e ikinci kez basın.

Betikler ve ajanlar için `split-source` ve `split-commit` vardır (bkz.
[CLI başvurusu](cli-reference.md)); bir ajan bu düzenleyiciyi sizin gözden geçirmeniz için önceden doldurulmuş
olarak da açabilir (`ui propose split`, bkz. [MCP ve ajanlar](mcp-and-agents.md)).

## Etkileşimli rebase

Checkout yapılmış dalın atası olan bir commit'te **Buradan etkileşimli rebase...**
seçin ya da bir dalı başkasının üstüne sürükleyip *Etkileşimli rebase*
seçin. Log'un yerini bir inceleme ekranı alır: her commit için bir satır, en
yenisi üstte, her birinde bir sürükleme tutamacı ve bir eylem açılır listesi.

![Etkileşimli rebase ekranı: dört commit, ikincisi Squash'a, üçüncüsü Drop'a ayarlı; Sıfırla, Rebase'i İptal Et ve Rebase'i Başlat düğmeleriyle](../images/rebase.png)

| Eylem | Tuş | Etkisi |
|---|---|---|
| Pick | `p` | Commit'i korur |
| Reword | `r` | Yeni bir mesajla korur |
| Squash | `s` | Mesajları birleştirerek altındaki commit'e katar |
| Fixup | `f` | Mesajını atarak altındaki commit'e katar |
| Drop | `d` | Dışarıda bırakır |
| Edit | `e` | Orada durur, böylece amend edebilirsiniz |

Yeniden sıralamak için bir satırı sürükleyin (veya `Ctrl+Up` / `Ctrl+Down`).
`a` autosquash'i önizler (`squash!` ve `fixup!` commit'leri hedeflerinin
yanına taşınır). **Rebase'i Başlat**'a (`Ctrl+S`) basana kadar hiçbir şey
olmaz; **Sıfırla** planı geri koyar, **Rebase'i İptal Et** (`Esc`) hiçbir şey
değişmeden çıkar. Bir adım çakışmada veya bir `edit`'te durduğunda Log'un üstündeki
bir banner **Devam et**, **Atla** ve **İptal et** sunar. Bir merge commit'i
içeren bir aralık etkileşimli olarak rebase edilemez; menü girdisi nedenini
söyler.

## Birden çok commit'i cherry-pick yapmak

`Ctrl+tık` bir commit'i açıp kapatır, `Shift+tık` (veya `Alt+tık`) bir aralık seçer, `Space`
geçerli olanı açıp kapatır, `Esc` temizler. Seçili bir commit'e sağ tıklayıp
**N commit'i `<dal>` üstüne cherry-pick yap...** seçin: aynı inceleme ekranı
aynı eylemler ve yeniden sıralamayla açılır ve **Cherry Pick'i Başlat**
bunları sırayla yeniden oynatır.

## Commit'leri Mermaid gitGraph olarak kopyalamak

![Üç seçili commit üzerinde Mermaid gitGraph olarak kopyala penceresi: kapsam, ayrıntı ve yön seçimleri, çit seçeneği ve canlı önizleme](../images/log-mermaid.png)

Log'da commit'leri seçin (`Ctrl+tık`, `Shift+tık`, `Space`; ya da imleci birinin üzerinde bırakın) ve
**Mermaid gitGraph olarak kopyala...** komutunu açın: Log'da `Shift+Y`, commit menüsünün **Kopyala**
alt menüsü veya paletin `Log: Mermaid gitGraph Olarak Kopyala...` satırı. Bir pencere üretilen
[Mermaid](https://mermaid.js.org/syntax/gitgraph.html) betiğini canlı ve kaydırılabilir gösterir;
`Enter` (veya **Kopyala** düğmesi) tam olarak o metni panoya koyar. Onu bir README'ye, bir pull
request açıklamasına veya [mermaid.live](https://mermaid.live) sitesine yapıştırın.

Pencerede üç combobox, bir onay kutusu ve önizleme vardır (`Tab` aralarında gezinir; her
combobox `Enter`, `Space` veya `Down` ile açılır):

- **Commit'ler**: seçim veya - bir commit seçiliyken ve HEAD ondan türüyorsa - **Seçimden HEAD
  konumuna kadar**, aradaki commit dizisi (Log'un yüklenmiş kısmından yanıtlanır).
- **Ayrıntı**: `structure` (commit'ler, dallar ve merge'ler; kimlikler kısa SHA değerleridir),
  `refs` (gerçek dal adlarını Mermaid dalları olarak, tag'leri `tag:` olarak ve HEAD commit'ini
  `type: HIGHLIGHT` olarak ekler), `subjects` (commit kimliği `<sha> <başlık>` olur, kaçışlanır ve
  `ui.mermaid_subject_width` karaktere kısaltılır) ve `full` (kimliğe yazarı ve UTC tarihini ekler).
- **Yön**: soldan sağa (`gitGraph LR:`) veya yukarıdan aşağıya (`gitGraph TB:`).
- **```mermaid çitini ekle** betiği bir Markdown kod çiti içine alır.

Ayrıntı, yön ve çit hatırlanır (`ui.mermaid_detail`, `ui.mermaid_orientation`,
`ui.mermaid_fence`). Aynı üreteç komut satırından da çalışır:
`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence]
[--range A..B | --limit N]` (bkz. [CLI başvurusu](cli-reference.md)); MCP sunucusu bunu `log`
aracının `format` bağımsız değişkeni olarak sunar.

Mermaid'in `gitGraph` dili bir yeniden oynatma dilidir (commit, branch, checkout, merge), bir grafik
tanımı değildir; bu yüzden üreteç bilmeye değer birkaç seçim yapar:

- Bir commit, seçili ilk ebeveyninin dalında devam eder; diğer çocuklar üzerinde gerçek bir dal
  adı taşıyan yeni bir dal açar (`structure` düzeyinde `b1`, `b2`...). İlk dal `main`'dir (veya
  bir `mainBranchName` başlığıyla gerçek ad) ve HEAD'in ilk ebeveyn zincirini izler.
- Bir merge diğer ebeveynin dalına katılır. O dal ebeveynden sonra ilerlemişse üreteç ebeveynde
  küçük bir işaretçi dalı (`<branch>-<sha>`) çatallar ve onu merge eder, böylece merge doğru
  commit'i gösterir.
- Seçimin dışındaki bir ebeveyn atılır: seçili ebeveyni kalmayan bir commit kendi hattını
  başlatır, bu yüzden bir seçimdeki boşluklar kopuk hatlar olarak görünür.
- Bir octopus merge (üç veya daha fazla ebeveyn) ardışık merge'lere dönüşür; aradakiler
  ` (octopus 1/2)` soneki alır, sonuncusu gerçek commit'i taşır.
- Cherry-pick'ler algılanmaz; başlıklardaki çift tırnaklar kesme işaretine dönüşür (Mermaid onları
  kaçışlayamaz).

## Commit'leri karşılaştırmak

Log'un iki veya daha fazla satırını seçin: `Ctrl+tık` bir
satırı açıp kapatır, `Shift+tık` (veya `Alt+tık`, `Shift+Up/Down`,
`Alt+Up/Down`) bir aralık seçer, `Space` geçerli satırı açıp kapatır. Commit'ler,
**WIP** satırı ve **stash** satırlarının hepsi seçilebilir. Sağ panel bir
**Karşılaştır** paneline dönüşür:

- başlıkta bir **Yer değiştir** düğmesi vardır; altında seçilen her satır Log
  sırasıyla (en yeni önce) iki satırlık bir **kart** olarak listelenir: yazarın avatarı (Log'un avatar
  ayarlarına göre: görüntü destekleyen terminallerde resim, diğerlerinde baş
  harfler), ad, ne kadar önce, kısa kimlik ve özet. **WIP** satırı "Çalışma ağacı /
  Commit'lenmemiş değişiklikler"dir; bir stash, stash simgesini, `stash@{n}` ve
  mesajını gösterir. En eski uçta bir **Kaynak** etiketi, en yeni uçta bir
  **Hedef** etiketi bulunur (Yer değiştir bunları taşır); üç veya daha fazla satırda
  bir not değişikliklerin birleştirildiğini söyler. Kart listesi panelin en çok
  üçte birini kaplar, sonra kayar (tekerlek, kaydırma çubuğu veya altındaki bölücüyü
  sürükleyerek boyutlandırın);
- bir karta tıklamak Log imlecini o satıra taşır ve grafikte yanıp söndürür; seçim
  değişmez, dolayısıyla `Space` o satırı sonra seçimden çıkarır;
- altında, her zamanki değişen dosyalar listesi (yol veya ağaç görünümü, filtre,
  sıralama) aralarında farklı olan dosyaları listeler. Bir dosyaya tıklayın ya da
  `Enter`'a basın; diff'i ortada, aynı birleşik veya bölünmüş düzenlerde, daha yeni
  uçta Blame, Dosya Geçmişi ve dosya görünümü ile açılır.

![Üç Log satırı seçili: Karşılaştır paneli onları avatarlı ve Kaynak / Hedef etiketli kartlar olarak, değişen dosyaların üstünde listeler](../images/compare-cards.png)

Neyin karşılaştırıldığı seçime bağlıdır:

| Seçim | Karşılaştırılan |
|---|---|
| iki commit (veya bir stash ile bir commit) | tıklama sırası ne olursa olsun, Log sırasına göre eski olan yeniyle |
| WIP satırı ve bir commit veya stash | o commit çalışma ağacıyla: izlenen dosyaların stage'deki ve stage dışındaki değişiklikleri (izlenmeyen dosyalar bir diff'in parçası değildir) |
| üç veya daha fazla commit | birleştirilmiş değişiklikleri: en eskisinden hemen öncesinden en yenisine kadar (merge'ler ilk ebeveynlerini izler) |

Yeniden adlandırma bir silme ile bir ekleme olarak görünür. **Yer değiştir** (`x`)
iki satırlık bir karşılaştırmanın uçlarını değiş tokuş eder; her zaman daha yeni uç
olan WIP satırı varsa ya da üç veya daha fazla satırlık bir aralıkta değiş tokuş edilecek
bir şey yoktur. WIP satırıyla yapılan bir karşılaştırma, siz dosya kaydettikçe
çalışma ağacını izler.

Klavye: `v` (veya komut paletinde **Log: Seçimi karşılaştır**) odağı karşılaştırmanın
dosya listesine taşır; `x` (veya **Log: Karşılaştırma uçlarını değiştir**) uçları
değiştirir; `n` (veya **Log: Sonraki karşılaştırılan satıra git**) bir karta
tıklamanın klavyedeki karşılığıdır: Log imleci sıradaki seçili satıra gider, sona
gelince başa döner. `Esc` veya bir satıra düz tıklama tek commit'e döndürür.
**Cherry-pick** öğesi yalnızca seçimdeki commit'leri sayar; WIP satırı ve stash'ler
orada yok sayılır.

Aynı listeler betikler ve ajanlar için de vardır:
`gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]` (CLI ve MCP
`diff` aracı; `--to` varsayılan olarak çalışma ağacıdır).

## Çakışmalar ve diğer bannerlar

Bir merge, rebase, cherry-pick veya revert bir çakışmada durduğunda Log'un üstünde
bir banner belirir ve çakışan dosyalar Durum panelinde kırmızı bir `!` taşır.
Bunları merge düzenleyicisinde çözün, ardından banner'ın **Devam et** veya
**Commit (merge)** düğmesini, geri çekilmek için de **İptal et** düğmesini
kullanın. Bkz. [Merge çakışmalarını çözmek](merge-conflicts.md).

Log'un üstündeki diğer bannerlar:

| Banner | Düğmeler | Palet |
|---|---|---|
| *`<dal>`, `<commit>` üstüne rebase ediliyor · adım/toplam · ... konumunda durdu* | **Devam et**, **Atla**, **İptal et** | *Rebase: Devam Et / Atla / İptal Et* |
| *Yamalar uygulanıyor (git am) · yama n / m* (duran bir `git am`) | **Çakışmaları çöz...** (bir yama çakıştığında), **Devam et**, **Atla**, **İptal et** (önce sorar) | *git am: Devam et / Bu yamayı atla / İptal et* |
| *Geride kalan bir index.lock (n sn önce) bu depodaki her değişikliği engelliyor* (bir git komutu öldü ve `.git/index.lock` dosyasını bıraktı) | **index.lock dosyasını sil** (önce sorar; yalnızca dosya en az 10 saniyelikse sunulur) | *Depo: Eski index.lock Dosyasını Sil* |
| *Otomatik stash bekliyor: ...* | Bkz. [Otomatik stash](#otomatik-stash) | *Banner: ...* |

## Blame

Blame, dosyanın tamamını satır numaraları ve sözdizimi renkleriyle, kodun **sağında** da bir ek açıklama sütunuyla gösterir (`ui.blame_side = "left"` onu sola alır). Bir dosyanın çubuğundan (`Blame`), dosya menüsünden, paletten ya da bir geçmiş satırından (`b`) açın. Çalışma ağacından açıldığında henüz commit edilmemiş satırlar ayrı gösterilir (bkz. Şeritler).

- **Hunk'lar.** En son aynı commit tarafından değiştirilmiş ardışık satırlar bir hunk'tır. İlk satırı yazarın avatarını (terminalin resim çizebildiği yerde resim, aksi halde baş harf rozeti, *Yazar avatarları* kapalıysa hiçbir şey), yazarı, ne kadar önce olduğunu ve commit özetini gösterir; diğer satırlar yalnızca yazarın renk çubuğunu gösterir. İnce bir çizgi hunk'ları ayırır. Bir hunk ekranın üstünde başlıyorsa ek açıklaması ilk görünen satırda yinelenir.
- **Şeritler.** Ek açıklamanın yanında: yazarın renk çubuğu ve bir **yaş ısı şeridi**: yeni satırlar için sıcak, eskiler için soğuk; dosyanın kendi en eski ve en yeni satırına göredir (açıklama başlıktadır). Henüz commit edilmemiş satırlar noktalıdır ve "Not committed yet" der.
- **Kart.** İşaretçiyi bir ek açıklamanın üzerinde bir an bekletin ya da `i` tuşuna basın; hunk'ın ayrıntı kartı açılır: tam commit kimliği, yazar ve e-posta, tarih, özet ve eylemleri. Seçili hunk renklendirilir.

  ![Minimap açık ve kart seçili hunk'ta açıkken Blame](../images/blame-card.png)
- **Eylemler** (kart satırları, hunk üzerindeki sağ tık menüsü, tuşlar ve paletteki `Blame: ...`): commit'i Log'da göster (`Enter`), kimliğini kopyala (`Shift+Y`), commit'in bu dosya için diff'ini göster (`d`), commit'in ebeveyninde Blame (`b`, yeniden adlandırmaları izler), dosyanın Dosya Geçmişi'ni aç (`Shift+H`).
- **Gezinme.** `j` / `k`, PageUp/PageDown, Home, `Shift+G` ve tekerlek her yerdeki gibi; `n` / `p` sonraki / önceki hunk, `Shift+N` / `Shift+P` aynı commit'in sonraki / önceki hunk'ı, `g` bir satır numarasına git.
- **Uzun hunk'lar.** Özetin altında üç veya daha fazla satırlık bir hunk, sütun boş kalmasın diye commit'in kısa kimliğini ve tam tarihi (soluk) gösterir.
- **Minimap.** `ui.minimap` açıkken (`Map` düğmesi, `M`) dosyanın minimap'i kodun yanında durur, şeritle aynı yaş ısısıyla renklendirilir; kaydırmak için boyunca tıklayın ya da sürükleyin.
- **Genişlik.** Ek açıklama sütununun kenarını sürükleyin ya da `>` / `<` tuşlarına basın; genişlik `ui.blame_width` olarak kaydedilir.

## Dosya geçmişi

Diff çubuğundaki `Geçmiş │ Tümü` seçicisinin `Geçmiş` parçasından, bir dosya menüsünden ya da paletten (`Dosya Geçmişi: ...`) açın. Orta alan ikiye bölünür: solda dosyanın commit'leri, en yeni önce, yeniden adlandırmalar izlenerek; sağda seçili commit için dosyanın diff'i, Diff Görünümü'nün tüm kipleriyle (hunk'lar / tüm dosya, satır içi / yan yana, sözcük diff'i, minimap, kaydırma).

- **Liste.** Commit başına iki satır. Birincisi: değişiklik harfi (A eklendi, M değişti, D silindi, R yeniden adlandırıldı), özet ve renkli `+eklenen -silinen`. İkincisi, soluk: yazarın avatarı (*Yazar avatarları* ayarına göre Log'un Yazar sütununun gösterdiğinin aynısı), yazar, göreli tarih, kısa commit kimliği ve dosyayı yeniden adlandıran commit'te `eski -> yeni`. Dar bir liste önce yeniden adlandırmayı, sonra kimliği, sonra yazar adının sonunu bırakır; yazarı ve tarihi asla. 100'lük sayfalar sona yaklaştıkça yüklenir; bu sırada `daha fazlası yükleniyor...` satırı görünür.
- **Gezinme.** `j` / `k`, oklar, PageUp/PageDown, Home / End, tekerlek ve tık; diff hemen takip eder (diff'ler arka planda alınır, satır başına önbelleğe alınır ve basılı tutulan tuş yalnızca birini ister). Yeniden adlandırmadan önceki bir satır, dosyanın o zamanki adıyla karşılaştırılır.
- **Diff / Dosya / Blame.** Görünüm çubuğunun kendi **Dosya**, **Diff** ve **Blame** düğmeleri, sağ tarafın seçili commit için neyi göstereceğini belirler: commit'in **Diff Görünümü**, **Dosya Görünümü** (dosyanın o commit'teki hâli) ya da **Blame**'i; hepsi liste solda kalacak şekilde bölünmüş alanın içinde, sütun seçicisinin `Geçmiş` parçası da vurgulu kalır. Seçilen kip, satır değiştirirken korunur ve sağ taraf seçimi izler. Tuşlar `Alt+1` / `Alt+2` / `Alt+3` (listede ayrıca `d` / `o` / `b`), iki yarıdan da; paletteki `Dosya Geçmişi: ...` satırları da aynısını yapar; `Esc` Diff'e döner.
- **Sütun.** Liste, **Geçmiş** listesini gösteren *commit sütunudur*: gizleme, gösterme, boyutlar ve `Geçmiş │ Tümü` seçicisi tek bir yerde, [Commit sütunu](interface-tour.md#commit-sütunu) içinde anlatılır. `Tümü`'ne tıklamak Dosya Geçmişi'nden çıkarır ve görünümü seçili commit'te tutar.

  ![Ortadaki bir commit seçiliyken dosya geçmişi: diff'i sağda](../images/file-history.png)

  ![Sağ tarafı Blame kipinde dosya geçmişi, seçili commit'teki dosyanın blame'i](../images/file-history-blame.png)
- **İki commit'i karşılaştırma.** `Shift+Up` / `Shift+Down` ya da `Shift+tık` bir aralık seçer; sağ taraf dosyanın iki uç arasındaki diff'ini gösterir, her ucun kendi yolu vardır. `Esc` aralığı bırakır.
- **Odak.** `Tab` (ya da bir tık) klavyeyi diff'e verir ve geri alır; diff kendi tuşlarını korur.
- **Eylemler** (sağ tık menüsü, tuşlar ve paletteki `Dosya Geçmişi: ...`): Log'da göster (`Enter`, çift tık; Log gerektiği kadar geriye yüklenir), kimliği (`y`) ya da özeti (`Shift+Y`) kopyala, diff'i göster (`d`), dosyayı o commit'te (`o`) ya da blame'ini (`b`) sağda göster, çalışma ağacıyla karşılaştır (`c`, çıkmak için tekrar), dosyayı o sürüme geri yükle (`Shift+R`, onay ister).
- **Geri al.** Bir sürümü geri yüklemek günlüğe yazılan tek bir adımdır: Geri al (`Ctrl+Z`) dosyanın eski baytlarını geri getirir.
- **Ayırıcı.** Yarılar arasındaki kenarı sürükleyin ya da `Alt+Right` / `Alt+Left`, sıfırlamak için `Alt+0`; genişlik oturum başına hatırlanır.

## Fetch, pull ve push

- **Pull** (`Ctrl+G p`), *Ayarlar > Git ve Kodlama > Pull düğmesi* ayarının
  dediği gibi pull yapar (`git.pull_mode`: merge, yalnızca fast-forward,
  rebase; depo başına bir ayar). `▾` listesi *Fetch All*, *Pull (mümkünse
  fast-forward)*, *Pull (yalnızca fast-forward)* veya *Pull (rebase)* işlemini
  düğmeyi değiştirmeden bir kez çalıştırır. Mod git'e her zaman açıkça
  iletilir; bu yüzden git yapılandırmanızdaki `pull.rebase` sizi
  şaşırtamaz.
- **Push** (`Ctrl+G Shift+P`), checkout yapılmış dalı varsayılan uzak depoya
  gönderir ve ilk seferde upstream'i ayarlar. `▾` listesi, seçtiğiniz başka bir
  uzak depoya bir kez push eder. Uzak depoda sizde olmayan commit'ler varsa bir
  iletişim kutusu **Pull (merge), sonra push**, **Force push (lease ile)...**
  (asla çıplak bir `--force` değil) veya **İptal** sunar.
- `Ctrl+G f` her uzak depoyu fetch eder. *Ayarlar > Eşitleme* içinde *Otomatik
  Prune*, fetch'lerin prune edip etmeyeceğine karar verir; *Otomatik Fetch
  Aralığı (dakika)* arka planda fetch eder.

Hepsi [CLI](cli-reference.md) içinde `gitpoulpi fetch`, `pull` ve `push` olarak da
vardır.

## Otomatik stash

Commit edilmemiş değişiklikler bir checkout, pull, merge veya rebase'i
engellediğinde GitPoulpi bunları sizin için kenara alabilir: stash'ler, işlemi
çalıştırır, sonra stash'i pop eder. *Ayarlar > Eşitleme > Checkout, Pull, Merge
veya Rebase öncesi otomatik Stash* (`general.auto_stash`):

- **sor** (varsayılan): bir iletişim kutusu *Değişiklikleri stash'le ve devam et*, *Her zaman otomatik stash'le* (aynısı; ayrıca `general.auto_stash = "always"` ayarlar, böylece bir daha sorulmazsınız; Ayarlar > Eşitleme bunu geri açar) veya *İptal* sunar.
  CLI ve MCP bunun yerine başarısız olur ve `--autostash` adını verir.
- **her zaman**: sormadan yapılır. **asla**: işlem reddedilir.

*Otomatik stash'e izlenmeyen dosyaları da kat* izlenmeyen dosyaları da alır.
Temiz bir ağaç hiçbir zaman stash görmez. Stash, işlem ve pop ayrı ayrı geri
alınabilir.

Stash geri konamazsa (sonuçla çakışır veya bir merge ya da rebase kendi
çakışmasında durmuştur), stash listesinde kalır. **Log'un üstündeki bir
banner** *Otomatik stash bekliyor: checkout öncesindeki değişikliklerin
stash@{n} içinde* der. Düğmeleri:

| Düğme | Yaptığı |
|---|---|
| **Çakışmaları çöz...** | Merge düzenleyicisini açar (yalnızca çakışmalı bir pop'tan sonra) |
| **Sonucu koru** | Çalışma ağacını olduğu gibi tutar ve stash kopyasını atar (yalnızca çakışmalı bir pop'tan sonra, hiçbir dosya çakışmalı kalmayınca) |
| **Şimdi pop et** (başarısız bir pop'tan sonra **Pop'u yeniden dene**) | Stash'i geri koyar |
| **Stash göster** | Stash'in değişikliklerini gösterir |
| **Baştan başla** | Aşağıya bakın |
| **At** | Stash'i atar (önce sorar) |

Her düğmenin bir palet komutu vardır (*Banner: Bekleyen otomatik stash'i pop
et*, *Banner: Baştan Başla...*, ...). Sizin yerinize hiçbir şey yeniden
denenmez veya atılmaz.

**Baştan başla** önce sorar. Sonra:

1. stash'lenen dosyaların güncel durumunun bir kopyasını
   `refs/gitpoulpi-backup/autostash-start-over-<time>` altında tutar;
2. yalnızca o dosyalardaki değişiklikleri atar (kaybolabilecek yerel
   değişiklikleri olan başka dosyalar varsa reddeder ve onları adlandırır);
3. otomatik stash'i yapan işlemi geri alma günlüğü üzerinden geri alır (bu
   yüzden o işlem en sonuncusu olmalıdır);
4. stash'i özgün commit üzerinde geri pop eder ve banner'ı temizler.

Bir adım başarısız olursa stash'e dokunmadan durur ve nerede olduğunu söyler.
Banner görünürken ve son işlem otomatik stash'i yapan işlemken **Geri al**
(`Ctrl+Z`, `gitpoulpi undo`) bir onaydan sonra aynısını yapar. Betikler
`gitpoulpi autostash start-over [--dry-run] [--yes]` kullanır (`--yes`
olmadan çıkış kodu 3).

**Bir otomatik stash beklerken geçmişi hareket ettiren işlemler reddedilir:**
bir dalın veya commit'in checkout'u, merge, rebase (etkileşimli de), pull,
fast-forward, cherry-pick, revert, reword, YZ ile commit oluşturma ve yineleme.
Pencere bu düğmeleri ve palet girdilerini grileştirir ve nedenini verir. CLI ve
MCP bir `PendingAutoStash` hatası alır (çıkış kodu 1). Geri kalan her şeye izin
verilir: okumalar, stage'leme, atma, commit, dallar, tag'ler, uzak depolar,
fetch, push, rebase continue / skip / abort, çakışma çözümü, `stash push`,
`reset` ve `stash pop` / `apply` / `drop`. Geri almaya da izin verilir.

Otomatik stash, çakışmalarda durmuş bir merge veya rebase'i beklerken, o
işlem biter bitmez (GitPoulpi'de ya da düz git ve ardından bir yenilemeyle)
kendiliğinden pop edilir.
