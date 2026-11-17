[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · **Türkçe** · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage'leme ve commit'leme

Neyi değiştirdiğinizi görme, bir commit'e tam olarak neyin gireceğini seçme,
commit'i yazma ve dosyalara daha derinlemesine bakma (blame, geçmiş, yerleşik
düzenleyici). *WIP satırı* ve *hunk* gibi terimler [arayüz
turunda](interface-tour.md) açıklanır.

## Çalışma değişiklikleri

Commit'lenmemiş iş, **Log'un ilk satırıdır** (kesik çizgili daire, `// WIP`,
değiştirilenler için `✎ n` ve eklenenler için `+ n` sayaçlarıyla). Onu seçin; sağ
panel çalışma değişikliklerini gösterir:

![Çalışma değişiklikleri: sağda Unstaged ve Staged dosyalar, altında commit formu](../images/hero.png)

- **Unstaged Dosyalar**: düzenlenmiş, eklenmiş (izlenmeyen) ve silinmiş, henüz
  sonraki commit'te olmayan dosyalar.
- **Staged Dosyalar**: sonraki commit'in içereceği şey.
- **Çakışmalı dosyalar** (kırmızı bir `!`; bir merge, rebase, cherry-pick veya revert
  durmuşken): `Enter` merge düzenleyicisini açar, bkz.
  [Merge çakışmalarını çözmek](merge-conflicts.md).
- **Yeniden adlandırılmış veya taşınmış dosyalar** iki listede de tek bir satırdır,
  `yeni <- eski` (diskte silinmiş bir dosya ile içeriğinin en az %50'si aynı olan yeni
  bir dosya, stage'lemeden önce bile taşıma sayılır). Stage'le, Unstage'le ve At her iki
  ada da etki eder: stage'lemek silmeyi ve yeni dosyayı stage'ler, böylece git tek bir
  yeniden adlandırma görür ve bu TEK bir geri alma adımıdır (`Geri Al Stage rename eski -> yeni`). Satırı açmak içerik diff'ini eski ada karşı gösterir,
  başlığı `eski -> yeni (%N benzer)` olur; düzenlemesiz bir taşımada **İçerik değişmedi**
  yazar. Bir commit'in dosyaları için de aynısı geçerlidir ve `gitpoulpi status` /
  `gitpoulpi diff` `renamed_from` bildirir.
- **Yol** veya **Ağaç** (`m`) dosyaları düz ya da klasörler halinde listeler.
  **Tümünü Daralt** (`z`) klasörleri kapatır. **Tüm dosyaları göster** (`a`)
  yalnızca değişen dosyalar yerine tüm ağacı gösterir. Listenin boş kısmına sağ
  tıklamak **Tümünü genişlet** / **Tümünü daralt**, bir klasör satırına sağ
  tıklamak **Tüm alt klasörleri genişlet** / **Tüm alt klasörleri daralt** sunar;
  tuşlar `=`, `-`, `*`, `_` (bkz. [Fare ve hareketler](mouse-and-gestures.md)).
- Başlık, hangi dalda kaç dosyanın değiştiğini söyler.

### Stage'leme, unstage'leme, atma

| Şunun için | Şunu yapın |
|---|---|
| Bir dosyayı stage'lemek | Satırının üzerine gelip **Dosyayı Stage'le**'ye tıklayın, ya da seçip `Ctrl+Space`'e basın, veya `Ctrl+G s` |
| Bir klasörü stage'lemek | Klasör satırındaki **Klasörü Stage'le** |
| Her şeyi stage'lemek | Bölüm başlığındaki **Tüm Değişiklikleri Stage'le**, veya `Ctrl+G a` |
| Unstage'lemek | Staged bölümündeki aynı düğmeler ve tuşlar (`Ctrl+G u`) |
| Bir dosyanın değişikliklerini atmak | Dosyaya sağ tık > **Değişiklikleri at...** (önce sorar; `Ctrl+Z` değişiklikleri geri getirir) |

### Hassas dosyalar

Bir dosyayı, bir klasörü veya her şeyi stage'lemek önce neyi ekleyeceğini denetler. Adı
`[privacy] sensitive_patterns` ile eşleşen (`.example`, `.sample` ve `.template` olanlar
dışında `.env` ve `.env.*`, `*.pem`, `*.key`, SSH özel anahtarları, `*.p12`, `*.pfx`,
`*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`) ya da metni tarayıcının
tanıdığı bir gizli bilgi içeren bir dosya stage'lenmez. **Hassas dosyalar** sorusu her
dosyayı nedeniyle birlikte listeler (bir kalıp ya da maskeli bir önizlemeyle gizli bilginin
türü, asla değer) ve **1. Yine de stage'le** veya **2. İptal et, hiçbir şeye dokunma**
seçeneklerini sunar. Bir ok tuşuna veya bir rakama basana kadar hiçbir yanıt vurgulanmaz;
böylece yanlışlıkla basılan bir Enter hiçbir şeyi yanıtlamaz; `Esc`, kapatma çarpısı ve
dışına bir tıklama iptal eder.
Bir dosyada bulunan gizli bilgi satırını adlandırır (`config/app.env:12`). Karar vermeden
önce bakmak için bir bulguya tıklayın (veya `Tab` / `Shift+Tab`, `Ctrl+J` / `Ctrl+K` ile
aralarında gezinin) ve **Dosyada göster** düğmesine, `S` tuşuna ya da paletteki **Hassas
Dosyalar: Dosyada Göster** komutuna basın: dosya Dosya Görünümü'nde o satıra kaydırılmış
ve satır vurgulanmış olarak açılır, Dosya Görünümü'nü kapattığınızda (`Esc`) soru
dokunulmadan geri gelir. Bakmak hiçbir şeyi stage'lemez.
Klavyeden, bir düğmeden, bir menüden, paletten ya da bir
sürükle ve bırakmadan gelse de aynı sorudur, çünkü hepsi aynı isteği gönderir. İptal etmek
hiçbir şeyi stage'lemez; aynı stage işleminin sıradan dosyalarını bile.

Her biri 1 MiB'a ve toplam 32 MiB'a kadar metin dosyaları okunur; daha büyük dosyalar ve
ikili dosyalar yalnızca adıyla değerlendirilir. `[privacy] staging_guard = false` denetimi
kapatır. Bir hunk'ı ya da bazı satırları stage'lemek denetlenmez: zaten o dosya üzerinde
çalışıyorsunuz.

Menüsü için bir dosyaya sağ tıklayın: stage'le veya unstage'le, at, yolu kopyala,
dosya geçmişi ve blame, harici düzenleyicinizde, diff aracında veya dosya
yöneticisinde aç ([Ayarlar > Diff ve düzenleyici](settings.md#diff-ve-düzenleyici)
sayfasında ayarlanır) ve daha fazlası.

## Dosyaları yok sayma ve ikili ya da metin olarak ele alma

Bir dosyaya sağ tıklayıp **Yok say** ile dosyayı veya uzantısını, ya da bir klasör satırında **Klasördeki tüm dosyaları yok say** ile klasörü yok sayın. GitPoulpi önce kuralın hangi dosyaya yazılacağını sorar: deponun `.gitignore` dosyası (varsayılan), dosyanın klasörüne kadar her klasörün `.gitignore` dosyası ya da `.git/info/exclude` (yalnızca bu klon, hiçbir zaman commit'lenmez). Henüz var olmayan dosyalar *yeni* olarak işaretlenir ve onaylayınca oluşturulur. Satır, seçilen dosyaya göreli yazılır: `src/a/b.txt`, `src/.gitignore` içinde `/a/b.txt` olur. Ok tuşlarıyla gezinin, listeyi daraltmak için yazın, `Enter` yazar, `Esc` iptal eder. `Ctrl+Z` bir `.gitignore` yazımını geri alır; `.git/info/exclude` çalışma ağacının dışındadır ve geri alınamaz.

![Bir yok sayma kuralının hangi dosyaya yazılacağını soran liste: deponun .gitignore dosyası, dosyanın klasöründeki .gitignore veya .git/info/exclude, her biri yazılacak satırla birlikte](../images/ignore-picker.png)

Bir dosyaya sağ tıklayıp **İkili olarak ele al** (Git dosyayı zaten ikili olarak diff'liyorsa **Metin olarak ele al**) seçeneği bir `.gitattributes` kuralı yazar: `binary` ya da `text diff`. Dosyanın kendisini veya uzantısındaki tüm dosyaları seçin, ardından yazılacak `.gitattributes` dosyasını yukarıdaki listeyle seçin (yalnızca bu klon için `.git/info/attributes`). Aynı desen için zaten yazılmış bir kural değiştirilir, asla çelişkili bırakılmaz. Dosyanın diff'i hemen güncellenir ve durum çubuğu neyin nereye yazıldığını söyler (başka bir kural hâlâ öncelikliyse uyarır). Klasör satırında **Klasördeki dosyaları şöyle ele al** > **İkili** ya da **Metin** vardır. Aynısı komut paletinde (**Çalışma Değişiklikleri: Seçili Dosyayı İkili Olarak Ele Al...**) ve komut satırında (`gitpoulpi treat binary|text <yollar>`) bulunur. Dosya Görünümü de kurala uyar: metin olarak ele aldığınız bir dosya, NUL baytı içerse bile satırlarını gösterir; Git'in ikili olarak diff'lediği bir dosya ise ikili uyarısını gösterir.

## Dosya listelerinde menüler, çoklu seçim ve dönüştürme

Sağ paneldeki dosya listeleri (Unstaged, Staged, bir commit'in değişen dosyaları ve **Tüm dosyaları göster**) aynı menüleri ve hareketleri paylaşır. Bir dosyaya, klasör satırına ya da listenin boş kısmına sağ tıklayın; veya odaktaki satırda `.` (`Shift+F10`, Menü tuşu) tuşuna basın; veya komut paletinden **Dosyalar: Satırın veya Seçimin Menüsünü Aç** komutunu çalıştırın.

- **Bir commit'in dosyası** (ve **Tüm dosyaları göster** içindeki dosya): **Diff aç**, **Dosyayı aç**, **Dosya Geçmişi**, **Dosya Blame'i**, düzenleyicinizde, varsayılan programda veya **Birlikte aç...** ile açma, **Klasörde göster**, **Dosyayı düzenle**, **Dosya yolunu kopyala**, **Dosyanın tam yolunu kopyala**, **Yok say**, **İkili olarak ele al** veya **Metin olarak ele al**, **Dönüştür**, **Dosyayı bu commit'ten geri yükle** (önce sorar; `git checkout <commit> -- <dosya>` gibi dosyayı ayrıca stage eder ve `Ctrl+Z` önceki dosyayı geri getirir) ve **Dosyayı sil...**. Dosya çalışma ağacında yoksa çalışma kopyasıyla ilgili satır nedeniyle birlikte gri görünür. **Çalışma ağacına karşı Diff**, commit'teki sürüm ile diskteki dosya arasındaki diff'i açar. **Dosya değişikliklerinden yama oluştur...**, o dosyanın commit'teki değişikliğinin yamasını yazar (bir seçimde ve klasör satırında da vardır; çalışma değişikliklerinin **Tüm dosyaları göster** görünümünde dosyanın çalışma ağacı değişikliklerinin yamasını oluşturur).
- **Klasör satırı**: **Klasörü aç**, **Dosya yolunu kopyala**, **Klasördeki tüm dosyaları yok say**, **Klasördeki dosyaları şöyle ele al**, **Dönüştür**, **Bu klasörde dosya oluştur**.
- **Listenin boş kısmı**: **Dosya Oluştur** ve **Depo klasörünü aç**. Unstaged listesi ayrıca **Tüm Değişiklikleri Stage'le** ve **Tüm değişiklikleri at...**, Staged listesi **Tüm Değişiklikleri Unstage'le** ekler.

![Bir commit'in dosyasının sağ tık menüsü: diff'i, dosya geçmişini ve blame'i aç, düzenleyicide veya varsayılan programda aç, yolları kopyala, yok say, dönüştür, commit'ten geri yükle ve sil](../images/commit-file-menu.png)

### Birden çok dosya seçme

Hareketler Log'dakilerle aynıdır. **Ctrl+tık** bir satırı seçime ekler veya çıkarır. **Shift+tık** (Shift+tıkı kendine ayıran terminallerde **Alt+tık**) çapa satırdan başlayan aralığı seçer. Düz tık baştan başlar ve diff'i eskisi gibi açar. `Esc` seçimi temizler. Klavyede `Space` veya `Insert` her listede geçerli satırı seçime ekler veya çıkarır (Unstaged ve Staged'de `Ctrl+Space` stage'ler: seçimi ya da geçerli satırı), `Shift+Yukarı` / `Shift+Aşağı` aralığı genişletir. Bir klasör satırı, kapalı olsa da açık olsa da altındaki tüm dosyaları seçer. Seçili satırlar vurgulanır ve durum çubuğu sayılarını gösterir. `Ctrl+A` odaktaki listenin tüm satırlarını seçer.

Tüm seçimin menüsü için seçili bir satıra sağ tıklayın (veya `.` tuşuna basın); menünün başlığı **N dosya** olur: stage veya unstage, at, yok say, ikili ya da metin olarak ele al, dönüştür, yolları kopyala, commit'ten geri yükle, sil. Stage, unstage ve geri yükleme tek işlemdir; atma ve silme önce sorar ve kaç dosya olduğunu söyler. Seçimin dışındaki bir satıra sağ tıklamak seçimi o satırla yeniden başlatır.

![Unstaged listesinde seçilen dört dosya ve 4 dosya başlıklı menü: stage'le, at, yok say, ikili veya metin olarak ele al, dönüştür, yolları kopyala ve sil](../images/file-pane-multiselect.png)

### Kodlama, satır sonu ve girintiyi dönüştürme

**Dönüştür**, bir dosyanın, klasörün veya seçimin menüsündedir. Komut satırında `gitpoulpi convert` aynı işi yapar.

- **Kodlama**: UTF-8, UTF-8 BOM, UTF-16 LE ve BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Satır sonları**: LF, CRLF veya CR.
- **Girinti**: satır başındaki boşluklar, boşluk veya sekme olarak ve sekme genişliğiyle.

Çalışma ağacındaki dosyalar, düzenleyicinin durum çubuğuyla aynı işlevlerle arka planda yeniden yazılır. Geçerli UTF-8 olmayan ve bayt sırası işareti (BOM) taşımayan bir dosya, düzenleyicideki gibi Windows-1252 olarak okunur. İkili dosyalar, eksik dosyalar ve hedef kodlamanın tutamadığı dosyalar atlanır ve bildirilir (*12 dosya dönüştürüldü, 3 atlandı (3 ikili)*), asla bozulmaz; yazılamayan bir dosya hata bildirimi oluşturur. Tüm toplu iş geri alma günlüğünde tek bir kayıttır: tek bir `Ctrl+Z` tüm dosyaları geri getirir. Düzenleyicide açık olan bir dosya, kapatılana kadar dönüştürmeyi engeller.

## Bir diff'i okuma

Bir dosya adına tıklayın (ya da üzerinde `d` tuşuna basın); diff'i ortada Log'un
yerini alır. `Esc` veya `q` Log'u geri getirir.

![Her biri Hunk'ı At ve Hunk'ı Stage'le düğmeli iki hunk içeren bir diff](../images/diff.png)

Üstündeki çubuk görünümü değiştirir:

- **Satır içi** birleşik diff'i gösterir; **Yan yana** önce ve sonrayı yan yana
  koyar (`m` yerleşimler arasında dolaşır).
- **Hunk** (`Shift+H`) yalnızca değişen hunk'ları bir satır bağlamla gösterir.
  Tüm dosyayı görmek için kapatın.
- **WS** (`s`) yalnızca boşlukta farklılaşan değişiklikleri gizler.
- **Kaydır** (`w`) uzun satırları kesmek yerine alt satıra kaydırır.
- **Map** (`Shift+M`) kaydırma çubuğunun yanında tüm dosyanın bir minimap'ini
  gösterir. Atlamak için ona tıklayın. Seçili bir sözcüğün diğer geçtiği yerler
  renklendirilir ve kaydırma çubuğunda işaretlenir.
- `n` ve `p` (ya da çubuktaki oklar) hunk'lar arasında atlar.
- **Otomatik** (son düğme, `Shift+C`), dosyanın okunduğu kodlamadır. Ona tıklayın (ya da `Shift+C` tuşuna
  basın veya paletten *Diff: Kodlamayı Seç...* komutunu çalıştırın) ve listeden birini seçin: önce
  **Otomatik algıla**, sonra UTF-8, UTF-8 BOM, UTF-16 LE ve BE, Windows-1252, ISO-8859-1, ISO-8859-15,
  Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 ve EUC-KR (listede aramak için
  yazın). Dosyanın iki tarafı, o kodlamayla, tam bayt olarak yeniden okunur; böylece bir Latin-1 dosyası
  aksanlarını, bir UTF-16 dosyası (git'in ikili saydığı) satırlarını gösterir. Seçim, çıkana kadar o
  dosyanın yolu için hatırlanır, bu görünümde ve [incelemede](review.md). Bir kodlama seçiliyken hunk'lar bu
  görünümün kendi hunk'larıdır; bu yüzden stage'leme ve atma kapalıdır (yeniden stage'lemek için
  **Otomatik algıla**'yı seçin). Yavaş bir okuma, bir süre sonra başlıkta *... olarak okunuyor* yazar.

![Aynı diff yan yana yerleşiminde, önce ve sonra yan yana](../images/diff-split.png)

Metni fareyle seçebilir (sürükleyerek veya bir sözcüğe çift tıklayarak) ve `y` ya da
`Ctrl+C` ile kopyalayabilirsiniz; yalnızca dosya metni kopyalanır, satır numaraları
veya `+` ve `-` işaretleri asla kopyalanmaz.

### Bir hunk'ı veya tek bir satırı stage'leme

Çalışma ağacı diff'inde her hunk başlığında **Hunk'ı Stage'le** ve **Hunk'ı At**
vardır (Staged tarafında **Hunk'ı Unstage'le**).

- `Space` imlecin altındaki hunk'ı stage'ler.
- Birkaç satırı stage'lemek için seçimi başlatmak üzere `v` tuşuna basın, `j`/`k`
  ile ilerleyin ve `Enter` tuşuna basın. Yalnızca *değişen* satırlar sayılır.
- `x` hunk'ı, `Shift+X` seçili satırları atar. İkisi de önce sorar ve `Ctrl+Z`
  değişiklikleri geri getirir.
- Menüsü için bir hunk'a sağ tıklayın (ya da `.`): stage'le, unstage'le, at, dosya
  geçmişi, blame, kopyala.

Bir commit'in diff'i salt okunurdur. Değişmiş bir resim resim olarak görünür;
bkz. [Görseller](image-view.md).

## Commit'i yazma

Commit formu sağ panelin altındadır ve **Özet** alanı aynı zamanda Log'un çalışma
değişiklikleri satırındaki alandır: ikisinden birine yazmak diğerinde görünür.

![Bir YZ mesajıyla doldurulmuş commit formu: karakter sayaçlı özet, açıklama ve yeşil commit düğmesi](../images/ai-commit-message.png)

- **Özet** (sayaç 72 karakterden kaçının kaldığını gösterir) ve **Açıklama**.
  Özette `Enter` açıklamaya geçer, orada `Enter` bir satır ekler.
- **N dosyadaki değişiklikleri commit et** (yeşil düğme) veya `Ctrl+Enter`
  stage'lenmiş değişiklikleri commit'ler. Hiçbir şey stage'lenmemişse düğme bunu
  söyler ve bir şey yapmaz.
- **Önceki commit'i amend et** (`Shift+A`) stage'lenmiş değişiklikleri ve yeni
  mesajı son commit'in içine katar. Zaten push edilmiş bir commit için bundan
  sonra (lease ile) force push gerekir.
- **Hook'ları atla** (`Shift+H` veya paletteki **Commit: Hook'ları Atlamayı Aç/Kapat**),
  depodaki `pre-commit` ve `commit-msg` hook'larını çalıştırmadan commit yapar.
  Yalnızca o commit için geçerlidir; **Ayarlar > Depoya özel ayarlar > Commit**
  bunu varsayılan olarak işaretli başlatır. **Önceki commit'i amend et**'in hemen altında
  kendi satırı vardır, tam etiketiyle; yalnızca çok dar bir panelde etiket **Hook yok** olarak
  kısalır, sonra yalnızca onay kutusu kalır (ipucu adını söyler). Asla kaybolmaz. Çok kısa
  bir formda bunun yerine Amend satırını paylaşır ve aynı şekilde kısalır.
- Durum panelinde `c` imleci forma taşır. `Esc` formdan çıkarır.
- `Ctrl+G c` ve `Ctrl+G Enter` (palette **Git: Commit...** ve **Git: Stage'lenen
  Değişiklikleri Şimdi Commit Et**) tek bir komuttur: her yerden çalışan commit
  düğmesi. Formda bir özet varsa ve bir şey stage'lenmişse hemen commit'ler;
  **Önceki commit'i amend et** ve **Hook'ları atla** seçeneklerine uyar, odağı
  taşımaz. Aksi halde hiçbir şey commit'lemez: imleci formun özetine taşır (panel
  gizliyse ya da daraltılmışsa gösterir) ve durum çubuğu hangi kısayolun commit
  ettiğini söyler (varsayılan olarak `Ctrl+Enter`). Hiçbir şey stage'lenmemişse bunu
  söyler. Durum çubuğuna hiçbir şey yazılmaz. Bir YZ mesajı hâlâ yazılıyorsa hiçbir
  şey commit'lemez ve bunu söyler (`Esc` YZ isteğini iptal eder). `Ctrl+Z` commit'i
  diğerleri gibi geri alır.
- Depo başına [Ayarlar](settings.md#depoya-özel-commit), **her commit sonrası push**
  edebilir, formu bir **commit şablonuyla** başlatabilir (şablondaki `#` yorumlarını
  mesajdan da çıkarır) ve bir merge'ü **squash** yapabilir: form bu durumda git'in
  squash mesajıyla açılır.
- Bir barındırma entegrasyonu bağlıyken formda `Alt+R` bir issue referansı ekler;
  bkz. [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md).

[YZ](ai.md) açıkken özetin sağındaki **`✦`** düğmesi stage'lenmiş değişikliklerden
bir mesaj yazar (`Ctrl+G i`); **YZ ile commit'ler oluştur** ise çalışmanızı
birkaç commit'e böler (aşağıya bakın). Metin, düzenlemeniz için forma girer; commit
düğmesine basana kadar hiçbir şey commit'lenmez.

**YZ ile commit'ler oluştur**, Amend satırının sağ ucunda durur ve sağ panelle birlikte
kaybolmak yerine küçülür: önce tam etiket, sonra **YZ ile oluştur**, sonra yalnızca
**`✦`**. Her genişlikte tıkını, ipucunu ve tuşunu (`Ctrl+G Shift+C`) korur.

Özet satırının sağ ucundaki **`✦`** (mesajı oluştur) ve **`#`** (issue referansı ekle) düğmeleri de aynı şekilde küçülür:
dar bir panelde kenar boşluklarını yitirir, sonra ikisini de listeleyen tek bir **`⋯`** düğmesinin menüsüne girer; böylece
hiçbiri kaybolmaz.

## Commit composer: değişikliklerinizi commit'lere bölün

*(YZ'nin açık olması gerekir: bkz. [YZ desteği](ai.md).)* **YZ ile commit'ler
oluştur** düğmesine basın (veya `Ctrl+G C` tuşlarına basın ya da çalışma
değişiklikleri satırına sağ tıklayın). Model, her biri kendi dosyalarıyla birkaç
commit önerir; her değişen dosya tam olarak bir commit'tedir ve geçersiz bir plan
modele bir kez geri gönderilir. Öneri, henüz hiçbir şeyin commit'lenmediği bir
inceleme olarak açılır:

![Commit composer: her biri dosyalarına genişletilebilen dört önerilen commit, yukarı, aşağı, düzenle ve at denetimleri ile Sıfırla, İptal ve 4 commit oluştur düğmeleri](../images/composer.png)

| Şunun için | Şunu yapın |
|---|---|
| Bir commit'i genişletmek veya daraltmak | `Left` / `Right`, ya da ok işaretine tıklayın; `*` hepsini aç/kapatır |
| Bir commit'i yukarı veya aşağı taşımak | Satırındaki `▲` `▼`, ya da `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Bir dosyayı başka bir commit'e taşımak | Onu commit'in üzerine sürükleyin (listeden veya sağdaki çalışma değişiklikleri listesinden); ya da dosyada `m` ve commit'i açılır listeden seçin; ya da dosyada `Alt+Up` / `Alt+Down` |
| Bir dosyayı kendi commit'ine koymak | Dosyada `n`, ya da `+ kendi commit'i` düğmesi |
| Bir commit'i yeniden yazmak | Kalem, `e`, `Enter` veya çift tık: mesajı düzenleyin, `Enter` korur, `Esc` atar |
| Bir commit'i dışarıda bırakmak | `Drop` (`d` veya `Delete`); dosyaları commit'lenmemiş kalır. `geri yükle` onu geri getirir |
| Baştan başlamak | **Sıfırla** (`r`) modelin planına döner |
| Onları oluşturmak | **N commit oluştur** (`Ctrl+S`, `Ctrl+Enter` veya `a`) |
| Çıkmak | **İptal** veya `Esc`: hiçbir şey yapılmaz |

Commit'leri oluşturmak her şeyi unstage'ler, sonra her commit için dosyalarını
stage'ler ve commit'ler; böylece çalışma ağacına hiç dokunulmaz. Bir adım
başarısız olursa durur ve kaç commit yapıldığını söyler. Dosyalar bütün olarak
stage'lenir: yalnızca kısmen stage'lenmiş bir dosya bütün olarak commit'lenir.
Composer'ın hangi profili ve yönergeleri kullanacağını *Ayarlar > YZ* içinde seçin;
bkz. [YZ desteği](ai.md#commit-composer).

## Dosya Görünümü, Blame ve Dosya Geçmişi

Görünüm çubuğunda **Dosya** (Dosya Görünümü, dosyanın içeriği), **Diff**, **Blame** ve
[commit sütununun](interface-tour.md#commit-sütunu) `Geçmiş │ Tümü` seçicisi (`Geçmiş`
listesi Dosya Geçmişi'dir) bulunur; aynıları bir dosyanın sağ tık menüsündedir. Dosyayı geldiğiniz commit'teki
haliyle gösterirler, ya da çalışma değişikliklerinden açtıysanız commit'lenmemiş
(`@ WIP`) haliyle.

- **Blame** ve **Dosya Geçmişi** kendi başlarına tam görünümlerdir: bkz. *Log ile çalışmak*
  içindeki [Blame](log-and-history.md#blame) ve
  [Dosya geçmişi](log-and-history.md#dosya-geçmişi). Bir blame hunk'ında ya da
  geçmiş satırında `Enter`, Log'da o commit'e gider.
- `y` veya `Ctrl+C` seçili metni kopyalar (Blame'de yalnızca kodu).
- `Shift+G` son satıra atlar. `q` görünümü kapatır.

## Yerleşik düzenleyici

**Düzenle** (`e`), dosyanın *çalışma ağacındaki kopyasını* orta panelin üzerinde
küçük bir düzenleyicide açar: sözdizimi renkleri, satır numaraları, geri alma ve
yineleme, bulma ve bir durum çubuğu (konum, satır sonu, girinti, dil). Eski bir
commit'in diff'inden açmış olsanız bile her zaman çalışma kopyasını düzenler.

![Sözdizimi vurgulu ve satır numaralı bir Rust dosyasını gösteren düzenleyici; araç çubuğunda Kaydet, Geri al, Yinele ve Bul var](../images/editor.png)

| Tuş | Eylem |
|---|---|
| `Ctrl+S` | Kaydet (dosya geçici bir dosyaya yazılıp yeniden adlandırılır, bu yüzden asla yarım yazılmaz; satır sonları, BOM ve son satır sonu korunur) |
| `Ctrl+Z`, `Ctrl+Y` | Geri al, yinele |
| `Ctrl+F`, `F3` | Bul, sonrakini bul |
| `Tab`, `Shift+Tab` | Girintile, girintiyi azalt (dosyanın kendi stilini izleyerek) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Tümünü seç, kopyala, kes, yapıştır |
| `Esc` | Kapat. Kaydedilmemiş değişiklik varsa **Kaydet** (`s`), **At** (`d`) veya **İptal** (`c`) seçeneklerini sorar |

İkili dosyalar reddedilir. Sekme boyutu, satır sonları, sözdizimi vurgulama, satır
numaraları ve sözcük kaydırma *Ayarlar > Diff ve düzenleyici* içindedir; bkz.
[Ayarlar](settings.md#diff-ve-düzenleyici).

**Seçimi izle** işaretliyken (düzenleyici çubuğundaki kutu; palette *Düzenleyici: Seçimi İzlemeyi Aç/Kapat* ve *Ayarlar > Diff ve düzenleyici* içinde de bulunur), düzenleyici açıkken sağ panelde başka bir dosya seçmek, o dosyanın diff'i veya Dosya Görünümü yerine çalışma kopyasını düzenleyicide açar. Kaydedilmemiş değişiklik varsa önce **Kaydet**, **At** veya **İptal** seçeneklerini sorar; İptal sizi geçerli dosyada tutar. Düzenlenemeyen bir dosya (ikili, çok büyük, silinmiş) olağan görünümünde açılır ve durum çubuğu nedenini söyler. Düzenleyiciyi kapatmak sizi seçili dosyanın görünümüne götürür. Varsayılan olarak işaretlidir. Bu yalnızca çalışma değişiklikleri içindir: bir commit'in veya karşılaştırmanın dosyasını seçmek düzenleyiciden çıkarır ve o dosyanın diff'ini gösterir; kaydedilmemiş düzenleme varsa önce **Kaydet**, **At** veya **İptal** sorulur. Düzenleyicide olmayan commit sütunu da onunla birlikte geri gelir.

Sağdaki durum çubuğu dosyanın **kodlamasını**, **satır sonlarını** ve **girintisini**,
ardından imleç konumunu ve sözdizimini gösterir. İlk üçü düğmedir: her biri bir açılır
liste açar (oklar veya tekerlek, süzmek için yazın, seçmek için `Enter`, kapatmak için
`Esc`). Komut paleti (`Ctrl+P`) aynı menülere sahiptir: *Düzenleyici: Kodlamayı
Değiştir*, *Düzenleyici: Kodlamayla Yeniden Aç*, *Düzenleyici: Kodlamaya Dönüştür*,
*Düzenleyici: Satır Sonlarını Dönüştür*, *Düzenleyici: Girintiyi Değiştir*.

![Çubuktaki UTF-8 düğmesinin altında açılan düzenleyici kodlama menüsü: dosyayı başka bir kodlamayla yeniden aç veya dönüştür](../images/editor-encoding-menu.png)

![LF düğmesinin altında açılan düzenleyici satır sonu menüsü: dosyayı LF, CRLF veya CR'ye dönüştür](../images/editor-eol-menu.png)

- **Kodlama.** Dosya, kendi bildirdiği kodlamayla açılır (BOM: UTF-8 veya UTF-16), yoksa UTF-8, o da değilse
  Windows-1252 olarak; çubuk hangisi olduğunu söyler. **Kodlamayla Yeniden Aç** dosyanın baytlarını başka bir
  kodlamayla yeniden okur (kaydedilmemiş değişiklik varsa önce sorar): algılama yanıldığında kullanın.
  **Kodlamaya Dönüştür** metni korur ve dosyanın kaydedildiği kodlamayı değiştirir (UTF-8, UTF-8 BOM,
  UTF-16 LE ve BE, Windows-1252, ISO-8859-1 ve -15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP,
  GBK, Big5, EUC-KR). Kaydedene kadar hiçbir şey yazılmaz ve Geri Al bunu geri alır. Metinde hedef kodlamanın
  tutamadığı karakterler varsa durum çubuğu kaç tane olduğunu ve ilkinin nerede olduğunu söyler, hiçbir şey
  dönüştürülmez; kaydetme de aynı şekilde reddeder, asla `?` yazmaz.
- **Satır sonları.** `LF`, `CRLF`, `CR` gösterir; dosya birkaçını karıştırıyorsa
  `Karışık` gösterir. Menü dosyanın tüm satır sonlarını tek bir geri alma adımında
  LF, CRLF veya CR'ye dönüştürür. Yazdığınız yeni
  satırlar dosyanın kendi satır sonunu kullanır. Seçiminiz *Satır sonları* ayarından önceliklidir.
- **Girinti.** `Boşluk: 4` veya `Sekmeler: 4` gösterir. Menü, `Tab` ve otomatik
  girintinin ne ekleyeceğini (boşluk veya sekme) ve boyutu (1 ile 8: girinti adımı ve
  bu dosyada bir sekmenin genişliği) belirler; ayrıca her satırın baştaki boşluklarını
  tek bir geri alma adımında boşluğa veya sekmeye dönüştürür.

## Stash, yamalar ve Git LFS

- **Stash** ve **Pop** araç çubuğundadır; bkz.
  [Log ile çalışmak](log-and-history.md#dallar-tagler-ve-refs-kenar-çubuğu).
- **Yama oluştur** ve **Yamayı uygula** commit ve ref menülerindedir; CLI'da
  `gitpoulpi patch` vardır.
- Git LFS ile saklanan büyük dosyalar, içeriklerini pull edene kadar küçük işaretçi
  dosyaları olarak görünür; bkz. [Git LFS](lfs.md).

## Komut satırından

Bu sayfadaki her şey betiklenebilir:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # aşağıdaki retten sonra
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` hassas bir dosyayı aynı şekilde reddeder: hiçbir şey stage'lenmez, çıkış
kodu 2'dir ve `--json` `{error: "sensitive_files", files: [{path, reason}]}` yazdırır (bir
kural ve maskeli bir önizleme, asla bir değer). `--allow-sensitive` yeniden denemedir. Bir
MCP ajanı aynı yanıtla reddedilir ve `[mcp] allow_sensitive_staging = true` olmadıkça
`allow_sensitive` geçemez: ona size sorması söylenir.

Bkz. [CLI başvurusu](cli-reference.md).
