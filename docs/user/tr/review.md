[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · **Türkçe** · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Bir pull veya merge request'i incelemek

Bir pull request'i (GitHub, Gitea, Forgejo), bir merge request'i (GitLab) veya bir Gerrit değişikliğini
([Gerrit](#gerrit) bölümüne bakın) **deponun penceresinden ayrılmadan** inceleyin. İnceleme görünümü; açıklamayı,
konuşmayı, değişen dosyaları (yorum dizileri satırlarının altında) ve commit'leri gösterir, incelemeyi yazdığınız yer
de burasıdır: yorumlar, onay ve değişiklik istekleri. Bu görünüm, bu barındırıcılardan biri için bir profil
([Barındırma hizmetleri ve issue takipçileri](hosting-integration.md) sayfasına bakın) ve ona bağlı bir depo gerektirir;
başka sağlayıcılar inceleme sunmaz. Barındırıcının sınırlarına uyulur: GitLab'da *Değişiklik iste* kararı yoktur ve yorum
başına bir satır kabul eder. Bu sayfa hepsine *istek* der.

![İnceleme görünümü, Genel Bakış sayfası: isteğin üst bilgisi ve üst veri düğmeleri, açıklama, çözülmemiş ve çözülmüş birer kod yorum dizisi, yorumlar ve yorum kutusu](../images/review-overview.png)

![Dosyalar sayfası: görüldü kutularıyla değişen dosyalar, satırının altında bir yorum dizisi olan diff ve gösterilen satırların dışında çözülmüş bir yorum dizisi](../images/review-files.png)

## Bir incelemeyi açmak

Her yol aynı görünüme çıkar:

- kenar çubuğunun **PULL REQUEST'LER** listesi: bir satır seçip **Enter**'a basın ya da sağ tıklayıp
  **İncele...**'yi seçin;
- **Radar**: satırın menüsündeki **İncele...** ya da `e` tuşu (depo ve onunla birlikte inceleme açılır;
  klonlamadığınız bir depodaki istek, klonlanıp klonlanmayacağını veya API üzerinden incelenip incelenmeyeceğini
  sorar, bkz. [Klon olmadan inceleme](#klon-olmadan-inceleme));
- **çalışma alanı araması** (`Ctrl+Alt+F`): bir pull veya merge request'i vurgulayıp **`Alt+R`**'ye basın;
- palet: **Barındırma: İsteği İncele...**, aralarından seçeceğiniz istekleri listeler.

Görünüm, deponun üzerinde pencereyi doldurur (ayrı bir pencere değil, depo sekmesinin bir kipidir). **`Esc`** veya
köşesindeki `✕` ondan çıkarır. Açılırken istek barındırıcıdan okunur ve kodu arka planda git ile getirilir; bu bir
andan uzun sürerse bir döner çubuk görünür.

## Üst bilgi

Her zaman görünür: numara ve başlık, durum (Açık, Taslak, Birleştirildi, Kapalı), dallar
(`main ← feature/moons`), yazar, denetimler (`✓ denetimler geçti`, `✗ denetimler başarısız`,
`● denetimler çalışıyor`), temiz birleşip birleşmediği ya da çakışıp çakışmadığı, boyut ve bazı yorumlar
yazdıysanız **`3 bekleyen yorum`**.

Dört üst veri düğmesi - **Atananlar**, **İnceleyenler**, **Etiketler**, **Kilometre taşı** - kenar çubuğuyla aynı
seçicileri açar (bkz. [Atananlar, inceleyenler, etiketler ve kilometre taşı](pull-requests-and-issues.md#atananlar-inceleyenler-etiketler-ve-kilometre-taşı));
üst bilgi, barındırıcı kabul eder etmez sonucu gösterir. Eylem düğmeleri: **Checkout** (isteğin dalının olağan
checkout'u, görünümü de kapatır), **Tarayıcıda aç**, **Yenile** (**`F5`**) ve **İncelemeyi bitir**.

## Üç sayfa

Üst kenarlıktaki bölümlü bir denetim sayfayı değiştirir: **Genel Bakış**, **Dosyalar**, **Commit'ler**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`** veya bir tık).

### Genel Bakış

Genel Bakış **isteğin konuşmasıdır**: bir issue'nun sahip olduğu aynı zaman çizelgesi ve aynı yorum kutusu
([Konuşmalar](conversations.md)): önce açıklama (*bunu açtı*), sonra yorumlar, kod üzerindeki
dizeler diff alıntılarıyla, gönderilmiş incelemeler kararlarıyla (`onayladı`, `değişiklik istedi`, `yorum yaptı`,
`yoksayıldı`; GitLab'da kararlar onaylarıdır) ve olaylar, en eskiden başlayarak, en yeni pencerede, en üstünde
**Eskileri yükle** ile. Yorum kutusu zaman çizelgesinin altına yerleşiktir (**Yeni yorum**): **Yaz**, **Önizleme** ve
**Yan yana**, **İptal** ve **Gönder**. **Burada yazılan bir yorum hemen barındırıcıya gider** (hiçbir şey beklemede
değildir: bekleyen yorumlar kod üzerindekilerdir ve incelemeyle birlikte gönderilir), listede hemen görünür ve
barındırıcı yanıt verince yerini alır; barındırıcı reddederse kutu metninizle ve nedenle yeniden açılır.

Tuşlar konuşma görünümününkilerdir: **`Up`** / **`Down`** (ve `Ctrl+J` / `Ctrl+K`) kayıtlar arasında gezinir,
**`r`** seçili kaydın dizesine yanıt verir, **`Q`** onu yeni bir yorumda alıntılar, **`c`** yeni bir yorum yazar,
**`e`** / **`d`** kendi yorumlarınızı düzenler ve siler (silmeden önce bir soru gelir), **`l`** veya `Enter` bir kaydın
bağlantılarını açar, **`g`** diyagramını, **`y`** adresini kopyalar, **`o`** tarayıcıda açar, **`n`** sonraki yeni
kayda gider, **`v`** olayları gösterir veya gizler, **`m`** eski kayıtları yükler, `Tab` zaman çizelgesi ile kutu
arasında geçer ve **Dahili not** (`Alt+I`) böyle notları olan barındırıcılarda bulunur. Seçili kaydın düğmeleri
(**Yanıtla**, **Alıntıla**, **Düzenle**, **Sil**, **Bağlantı**) aynı işi yapar ve palette *İnceleme: ...* satırlarının
yanında *Konuşma: ...* satırları vardır. Bir bağlantı açılmadan önce sorulur, incelemenin her yerinde olduğu gibi.

Koda ait bir dizi `path:line` değerini bağlantı olarak gösterir (altı çizili, gerçek bir düğme). Üzerine tıklamak, seçili
dizide **`G`** veya paletteki *Konuşma: Dizinin Kodunu Aç*, **Dosyalar** sayfasına geçer, o dosyayı açar ve imleci yorum
yapılan satıra getirir (yazıldığı gibi eski ya da yeni tarafta); dizi altında açık durur (katlanmışsa açılır). **Esc**,
sayfa sekmelerinin solundaki **Genel Bakışa dön** düğmesi veya *İnceleme: Genel Bakışta Yoruma Dön* Genel Bakışın aynı
kaydına geri götürür. Satırı artık diff'te olmayan (eskimiş) bir dizi dosyayı en yakın satırda açar ve durum satırı bunu
söyler; isteğin artık değiştirmediği bir dosyadaki dizi için dosyanın diff'te olmadığı bildirilir. Aynı isteğin issue ya da
Radar görünümünde açılacak kod yoktur: `G` kodun incelemede olduğunu söyler.

Bir konuşma tuşu ile bir inceleme tuşu çakıştığında, bu sayfada harf konuşmanındır: `l` (etiketler), `m` (kilometre
taşı), `Q` (Gerrit'in vazgeçmesi), `e` / `d`, `r`. Etiketler, kilometre taşı ve vazgeçme başlığın düğmelerinde ve
paletle kalır; `a` (atananlar), `p` (inceleyenler), `f` (**İncelemeyi bitir**), `b` (**Checkout**), `P`, `D`, `U` ve
`Alt+1` / `Alt+2` / `Alt+3` burada anlamlarını korur. Klavye kutudayken her tuş kutununkidir, `Alt+1` / `Alt+2` /
`Alt+3` dışında; `Esc` klavyeyi zaman çizelgesine geri verir ve ikinci bir `Esc` incelemeyi kapatır.

Başka bir pencerede veya CLI ile yazılan bir yorum burada kendiliğinden görünür: GitPoulpi'nin her yazması (bu pencere
dahil) oturuma bildirir ve aynı isteği gösteren pencereler onu yeniden okur.

Yorum kutusu, [konuşma görünümüyle](conversations.md#yazma) paylaşılan Markdown yazma kutusudur (bir dizeye
yanıtın, bir satıra yorumun ve **İncelemeyi bitir** özetinin kutusu da): `Ctrl+Enter` (ayrıca `Ctrl+S`, `Alt+Enter`)
gönderir ve YZ açıkken `Alt+E` **YZ ile Yaz**'ı açar ([YZ bölümü](ai.md#sunucuda-yazma-yardımı)). Önizleme metni
barındırıcının göstereceği gibi, referanslarla birlikte gösterir (`#7`, `!12`, `ABC-12`, commit kimlikleri);
gönderilmemiş bir metin istek başına saklanır.

### Dosyalar

Solda değişen dosyalar (durum harfi, `+eklenen -kaldırılan`, açık yorum dizilerinin `●n` sayısı, bekleyen
yorumlarınızın `✎n` sayısı) ve sağda seçili dosyanın diff'i. Diff git'ten gelir: isteğin head'i
`refs/gitpoulpi/pr/<n>` içine getirilir (asla bir dal olmaz, hiçbir şey checkout edilmez) ve merge base ile
karşılaştırılır; böylece kenar boşluğundaki satır numaraları, yorumların kullandığı numaralarla tam olarak aynıdır.

- **Bir dosyanın önündeki `[ ]`**: onu *görüldü* diye işaretler (tık ya da listede **`v`** / `Space`). Başlık bunları
  sayar. Görüldü işaretleri depo başına tutulur ve yeniden başlatmaya dayanır.
- **Yorum dizileri** tam satırlarının altında yer alır: ilk yorum, yanıtlar ve **Yanıtla** ile
  **Çöz** / **Yeniden aç** düğmeleri. Çözülmüş bir yorum dizisi tek satıra katlanır; **`z`** (veya üzerinde `Enter`, ya da
  `▾` üzerine tık) herhangi bir yorum dizisini katlar ve açar.
- **Eskimiş yorum dizileri** ve diff'in göstermediği satırlardaki yorum dizileri, hunk'lardan sonra
  *Gösterilen satırların dışındaki yorumlar* altında listelenir; kod onların altında değiştiyse bir `eskimiş` rozeti
  görünür.
- **`c`**, imlecin bulunduğu satıra yorum yapar (satır başındaki küçük `+` fareyle aynı şeyi yapar).
  **`Shift+Up` / `Shift+Down`** (veya satırlar üzerinde sürükleme) önce birkaç satır seçer ve yorum aralığı
  kapsar. GitLab yorum başına bir satır alır ve görünüm seçmek yerine bunu söyler.
- **`r`**, imlecin altındaki yorum dizisini yanıtlar, **`x`** onu çözer veya yeniden açar. İkisi de barındırıcıya
  **hemen** gönderilir.
- Tuşlar: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` gezinir; `Tab`, `Right` ve `Enter` dosya listesinden
  diff'e geçer, `Left` ve `Tab` geri döner; `]` ve `[` sonraki ve önceki dosyaya atlar.

### Diff'in görüntü seçenekleri

**Dosyalar** ve **Commit'ler** sayfalarındaki diff bölmesi, [Diff görünümünün](staging-and-committing.md) kendisidir;
ilk satırında aynı çubuk ve aynı hatırlanan seçimler bulunur (bunlar tek bir ayardır: burada seçtiğiniz, Diff
görünümünün açıldığı şeydir ve tersi de geçerlidir):

- **Satır / Bölme**: tek sütun ya da önce ve sonra yan yana. Bölmede bir yorum dizisi, bekleyen bir yorum veya bir
  öneri ait olduğu yarının altında yer alır: kaldırılmış bir satırdaki yorum sol yarının, eklenmiş ya da
  değişmemiş bir satırdaki yorum sağ yarının altında. Bir kaldırma ve bir ekleme gösteren bir satırda **`c`**'ye
  basmadan önce kastettiğiniz yarıya tıklayın; klavye yeni tarafa yorum yapar.
- **Hunk**: yalnızca değişiklikler bağlamlarıyla ya da değişiklikler yerinde olacak biçimde tüm dosya. Tüm dosyada,
  değişikliklerin dışındaki bir satırdaki yorum dizisi, *Gösterilen satırların dışındaki yorumlar* yerine
  satırının altında yer alır. Dosyanın metni arka planda okunur; o gelene kadar hunk'lar bunu söyleyen bir satırla
  ekranda kalır (Commit'ler sayfasında bir commit'in en çok 40 dosyası bu yolla okunur).
- **WS**: yalnızca boşlukta farklılaşan değişiklikleri göster ya da yok say.
- **Kaydır**: uzun satırları alt satıra kaydır.
- **Yukarı / aşağı oklar**: önceki ve sonraki hunk.
- **Map** (**`M`**): kaydırma çubuğunun yanındaki minimap, Diff görünümününkiyle tek bir ayar. Kaydırmak için ona
  tıklayın veya üzerinde sürükleyin.
- **Otomatik** (**`E`** veya *İnceleme: Kodlamayı Seç...*): dosyanın okunduğu kodlama, Diff görünümündeki gibi:
  Otomatik algıla ya da listedeki kodlamalardan biri; bölmenin gösterdiği dosya için seçilir (Commit'ler sayfasında
  imlecin altındaki dosya). Dosyanın iki tarafı da o kodlamayla, arka planda yeniden okunur ve seçim, aynı yol için
  Diff görünümüyle paylaşılır. Bu yolla okunamayan bir dosya, diff'inin üstünde nedenini söyler.
- **`⋯`**: bölme her düğmeye sığmayacak kadar darsa, sığmayanlar çubuğun sonundaki bir `⋯` arkasındadır; listesi
  her birini tıklanmış gibi çalıştırır. Tuşları ve palet satırları her zamanki gibi çalışır.

Kaydırma çubuğunun cetveli nelerin nerede olduğunu gösterir: değişiklikler (yalnızca tüm dosya, Diff görünümündeki
gibi), her yorum dizisi, bekleyen yorum ve öneri için bir `●` noktası ve imleç.

Uzun satırlar bölmenin kenarında kesilir (tamamını görmek için **Kaydır**'ı açın) ve yana kayar: diff üzerinde
**`Right`** ve **`Left`** (`Left`, sol kenarda dosya listesine geri döner), yan tekerlek veya tekerlekle `Shift`,
alt kenarlıktaki kaydırma çubuğu ya da *İnceleme: Diff'i Sola / Sağa Kaydır*.

### Metin ve satır seçmek

Bir sürüklemenin nerede başladığı, neyi seçeceğini belirler. **Kodda** başlayan bir sürükleme, Diff görünümündeki
gibi metin seçer (çift tık bir sözcük seçer); **`Ctrl+C`** (veya *İnceleme: Seçili Metni veya Satırı Kopyala*) tam
olarak o karakterleri satır numaraları ve işaretler olmadan kopyalar; metin seçimi yoksa imlecin altındaki satırı
kopyalar. **Oluktan** (satır numaraları, işaret, `+`) başlayan bir sürükleme satır seçer; önceden olduğu gibi onlara
yorum yapmak için, ve oluktaki bir çift tık hâlâ o satıra bir yorum başlatır.

Diff görünümünün `n`, `p`, `m`, `w`, `s` tuşları incelemenin kendisine aittir; bu yüzden incelemede anahtarlar büyük
harftir: **`V`** yerleşim, **`H`** hunk'lar veya tüm dosya, **`S`** boşluk, **`W`** kaydırma, **`J`** / **`K`** sonraki ve
önceki hunk, **`E`** kodlama, **`M`** minimap. Her birinin bir palet satırı vardır (*İnceleme: Satır İçi veya Bölünmüş
Yerleşim*, ...).

**Genel Bakış**'ta konuşma ve ✦ taslak bloğu aynı biçimde seçilir: metnin üzerinde sürükleyin,
**`Ctrl+C`** kopyalar (bastığınız bir bağlantı ya da düğme eylemini korur ve birinde başlayan bir sürükleme
hiçbir şey seçmez). Seçim yokken **`Ctrl+C`** (ya da *Konuşma: Metni Kopyala*) seçili girdiyi sunucunun sakladığı
haliyle kopyalar. Metnin üzerinde I-beam, düğmelerin ve bağlantıların üzerinde el görünür. Bir seçim ekranda
olanı izler: kaydırmak onu metniyle birlikte taşır (görünümün dışındayken seçili kalır), metni yeniden yazan bir
yenileme onu bırakır ve kaydırılmış bir paragraf gösterdiği satırlar olarak kopyalanır.

**Dosyalar** sayfasındaki diff'in içindeki yorumlar (bir konuşma dizisi, bekleyen notlarınız, ✦ öneriler) de
seçilir: sözcüklerinin üzerinde sürükleyin, **`Ctrl+C`** (ya da *İnceleme: Seçili Metni veya Satırı Kopyala*)
kopyalar, **`Esc`** seçimi bırakır. Düğmelerinden birine basmak ya da bir konuşma dizisinin başlığına düz tıklamak
(katlar) yine eylemini yapar; başlıkta başlayan bir sürükleme sözcüklerini seçer ve diziyi olduğu gibi bırakır.

### Commit'ler

Yalnızca isteğin commit'leri (arkasındaki geçmiş değil), en yeniden başlayarak ve seçili olanın diff'i (ebeveynine
karşı değişiklikleri), tüm dosyaları alt alta ve aynı görüntü seçenekleriyle.

## Bir incelemeye geri dönmek

Başka bir sekmede çalışırken bir inceleme açık kalır. İnceleme açıkken sekmeden ayrılın (başka bir sekmeye tıklayın, `Ctrl+PageDown`, paletteki **Depo: Sonraki Sekme**) ve geri dönmek onu aynı istekte, aynı sayfada (Genel Bakış, Dosyalar ya da Commit'ler), aynı dosyada ve aynı satırda, klavye aynı tarafta olacak biçimde yeniden açar. Radar üzerinden API ile yapılan bir inceleme için de aynısı geçerlidir: Radar sekmesini yeniden gösterin ve inceleme üzerinde geri gelir.

Bir inceleme, sayfası açık olduğu sürece **sürmektedir**. Onu kapatmak (`Esc`, çarpı, **İnceleme: Kapat**), göndermek (**İncelemeyi bitir** ya da bir Gerrit değişikliğini göndermek veya bırakmak) ya da dalını checkout etmek onu bitirir ve yeniden açılmaz. İnceleme açıkken GitPoulpi'den çıkmak sekmeden ayrılmak sayılır: sonraki başlangıç onu aynı biçimde yeniden açar.

İstek bu arada merge edildiyse veya kapatıldıysa ya da sunucu yanıt vermiyorsa hiçbir sayfa açılmaz: durum satırı nedenini söyler ve bir sonraki sefer hiçbir şey yeniden açılmaz. Bekleyen yorumlar, görüldü işaretli dosyalar ve taslak her zamanki gibi korunur (bkz. **Bir inceleme yazmak**). Ayrıldığınızda hâlâ süren bir öneri çalıştırması taşınmaz.

## Bir inceleme yazmak

Kod üzerindeki bir yorum **bekleyen**dir: makinenizde tutulur (deponun durum dosyasında, yani yeniden
başlatmaya dayanır) ve üst bilgide sayılır. Bekleyen bir yorum, satırının altında `Bekleyen - satır 12` rozetiyle ve
iki düğmeyle görünür: **`e`** (Düzenle) ve **`d`** (Sil).

**İncelemeyi bitir** (**`f`**, düğme veya palet) küçük bir form açar: bir özet (isteğe bağlı) ve bir
**Karar** listesi: *Yorum*, *Onayla*, *Değişiklik iste*. Yalnızca barındırıcının izin verdikleri listelenir: GitLab'da
*Değişiklik iste* yoktur. **İncelemeyi gönder** (`Ctrl+S`) özeti, kararı ve her bekleyen yorumu tek seferde
gönderir. **İncelemeyi at...** bir onaydan sonra bekleyen yorumları çöpe atar.

İncelemeyi beklemeden hemen gönderilenler: yanıtlar, çözme ve yeniden açma ve konuşmaya yapılan yorumlar. Yorum
kutusu hangi türden olduğunu söyler (*Bu yorum hemen gönderilir* veya *Bu yorum, siz bitirene kadar bekleyen
incelemenize eklenir*). Barındırıcı bir yanıtı reddederse kutu, sözlerinizle yeniden açılır.

İstek, bekleyen yorumları yazdıktan sonra yeni commit'ler aldıysa görünüm, satırlarının kaymış olabileceği konusunda
uyarır (GitHub ve GitLab bir yorumu okuduğunuz commit'e yerleştirir; daha eski bir head'in incelemesi reddedilir).

## Gerrit

Bir Gerrit sunucusu diğer barındırıcılar gibi bağlanır (bkz. [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md):
sunucu URL'si, kullanıcı adı, HTTP parolası) ve **PULL REQUEST'LER** altında bir *projeye* bağlanır. Değişiklikleri pull
request olarak listelenir (`#7`, `GR` çipi) ve aynı görünümde incelenir; Gerrit'in getirdiği farklarla:

- **Yama kümeleri.** Bir değişikliğin numaralı yama kümeleri vardır (commit'in her yüklenişi bir tanedir). Üst
  bilgide, sayfa sekmelerinin yanında, üst kenarlıkta bir **`Yama kümesi N`** açılır listesi bulunur (**`P`** veya
  *İnceleme: Yama Kümesi Seç...* palet satırı). En sonuncuyla başlar; bir başkasını seçmek o yama kümesinin
  dosyalarını ve kodunu okur ve diğer yama kümelerinde yazılan yorum dizileri *eskimiş* olarak listelenir. Bitirdiğiniz
  inceleme, gösterilen yama kümesine gönderilir.
- **Başka bir yama kümesiyle karşılaştırmak.** Onun solunda ikinci bir combobox, **`Karşılaştır: temel`**
  (**`D`** veya *İnceleme: Yama Kümelerini Karşılaştır...*), temeli ve gösterilenden eski her yama kümesini
  listeler. Yama kümesi 3 gösterilirken yama kümesi 1'i seçerseniz Dosyalar sayfası, yama kümesi 3'ün yama kümesi
  1'den beri değiştirdiklerini ve yalnızca onları listeler ve fark olarak gösterir (sunucunun kendi farkı, bu yüzden
  daha yeni bir hedefe rebase değişiklik gibi görünmez); Commit'ler sayfası o tek farkı gösterir. İki yama kümesi de
  `refs/gitpoulpi/pr/<n>` ve `refs/gitpoulpi/pr/<n>-ps<k>` içine fetch edilir. Gösterilenden başka yama kümelerinde
  yazılan yorum dizileri *eskimiştir*. Karşılaştırılandan daha eski bir yama kümesini okumak için seçmek
  karşılaştırmayı temele sıfırlar. CLI karşılığı `review show|fetch --compare-with K` komutudur.
- **Teslim etmek ve vazgeçmek.** Sunucu değişikliğin teslim edilebileceğini söylediğinde (gereken oylar ve denetimler
  var), üst bilginin düğmelerine bir **Teslim et** düğmesi katılır; değişiklikten vazgeçebildiğiniz sürece **Vazgeç**
  de vardır. İkisi de (tuşlar **`U`** ve **`Q`**, palet *İnceleme: Değişikliği Teslim Et...* / *İnceleme:
  Değişiklikten Vazgeç...*) önce sorar, eylemin sunucuda gerçekleştiğini ve **GitPoulpi'den geri alınamayacağını**
  söyleyen bir pencerede (merge edilmiş bir değişiklik merge edilmiş kalır; terk edilmiş bir değişiklik Gerrit'in web
  arayüzünde geri yüklenebilir). Reddetmek hiçbir şey göndermez; başarıda görünüm değişikliği yeniden okur. CLI
  karşılıkları `gitpoulpi hosting review submit-change <n>` ve `hosting review abandon <n> [--message M]` komutlarıdır.
- **Karar yerine oylar.** **İncelemeyi bitir**, oy verebileceğiniz her etiket için bir açılır liste gösterir -
  `Code-Review` (-2 ile +2 arası) ve projede varsa `Verified` - ve yalnızca sunucunun **bu değişiklik ve sizin için**
  izin verdiği değerleri listeler (hakkı olmayan bir geliştirici +2'yi seçemez). Mevcut oyunuzla başlar; yalnızca
  değiştirdiğiniz bir oy gönderilir. Özet ve bekleyen yorumlar aynı çağrıda gider. Gerrit web arayüzünde sakladığınız
  taslaklar inceleme ile yayımlanır.
- **Yorum dizileri**, Gerrit'in yorum zincirleridir. En yeni yorumu `unresolved: false` diyorsa bir yorum dizisi
  çözülmüştür; **çöz** bunu yapan bir `Done` yanıtı gönderir, **yeniden aç** bir `Reopened` yanıtı. Yanıtlar ve çözme,
  diğer barındırıcılardaki gibi sunucuya hemen gider.
- **İnceleme için push.** Gerrit, `refs/for/<branch>` konumuna push ettiğinizde bir değişiklik oluşturur. PULL
  REQUEST'LER bölümü bir Gerrit projesine bağlıyken bölümün **`+`** düğmesi ve **Barındırma: İnceleme İçin Push...**
  palet satırı hedef dalı sorar ve `HEAD`'i `refs/for/<branch>` konumuna push eder (bir konu komut satırında
  verilebilir). Push, GitPoulpi'den geri alınamaz (değişikliği Gerrit'te terk edin); ileti bunu söyler. CLI karşılığı
  `gitpoulpi hosting review push`'tur.
- Üst veri düğmeleri Gerrit için salt okunurdur (inceleyenler gösterilir), *Değişiklik iste* kararı yoktur (-1 veya -2
  oyu aynı şeydir) ve denetimler çipi `Verified` etiketidir.

## ✦ YZ inceleme taslağı

![Dosyalar sayfasında satırlarının altında iki ✦ önerisi, her birinde Kabul et, Düzenle ve Yoksay; yanlarında var olan bir yorum dizisi](../images/review-ai.png)

YZ açıkken (*Ayarlar > YZ*, bkz. [YZ desteği](ai.md)) üst bilgide bir **`✦ İnceleme taslağı...`** düğmesi (ayrıca **`s`** ve
*İnceleme: ✦ İnceleme Taslağı...* palet satırı) bulunur. [İnceleme seçeneklerini](#i̇nceleme-seçenekleri) açar ve
oradaki **`Enter`** modele sorar. Modele soran tek şey budur: bir incelemeyi açmak asla sormaz. Model
isteğin başlığını ve açıklamasını, zaten var olan yorumları (tekrar etmemek için) ve diff'i okur; bir **inceleme taslağı** ile yanıt verir: gönderilecek metin, önerdiği karar (Onayla, Değişiklik iste veya Yorum), genel bir özet ve
önerilen yorumların bir listesi. Uzun bir diff model için kesilir - dosya başına ve toplamda - ve Genel
Bakış kaç dosyanın kısaltıldığını veya dışarıda bırakıldığını söyler. Çalışırken durum çubuğu bir an sonra bir döner
çubuk gösterir ve **`Esc`** (veya aynı düğme) onu iptal eder. [Dört düzeyin](ai.md#talimatlar) *Kod incelemesi*
talimatları uygulanır; böylece bir ekip veya bir çalışma alanı iyi bir incelemenin neye benzediğini söyleyebilir.

### İnceleme seçenekleri

`s`, deponun varsayılanlarıyla (deponun hiçbiri yoksa genel olanlarla) doldurulmuş **✦ İnceleme seçenekleri**
iletişim kutusunu açar. `Enter` (veya **Çalıştır** düğmesi) çalıştırmayı kutunun gösterdikleriyle başlatır; `Esc`,
**İptal** ya da çarpı kutuyu kapatır ve hiçbir şey sormaz.

| Denetim | Ne yapar |
|---|---|
| **Ajanı depoda çalıştır** | *Profil ayarı* (YZ profilinin çalışma klasörü belirler), *Evet* veya *Hayır*. Bir HTTP profili için (çalışma klasörü yoktur) ve istek klon olmadan API üzerinden incelenirken, altında nedeniyle birlikte gri olur. |
| **CLAUDE.md**, **AGENTS.md** | Modele deponun o dosyasını ver; isteğin hedeflediği dalda okunur. Ajan depoda çalışırken bunları kendisi okur ve kutu bunu söyler. |
| **Ek dosya** | Bir dosya daha: depodaki bir yol, mutlak bir yol veya bir kod barındırıcısındaki dosyaya bağlantı. **Gözat...** (`Ctrl+B` ya da düğmede `Enter`) depo kökünden (veya zaten adı verilen dosyanın yanından) başlayan bir dosya seçici açar; depo içindeki bir dosya köke göre bir yol olarak, dışındaki mutlak bir yol olarak yazılır. Alana yazmak eskisi gibi çalışır. |
| **Ek dosya yok** | Genel ayar ya da depo bir dosya belirtse bile bu çalıştırma için ek dosya yok; bir dosya yazmak ya da **Gözat...** ile seçmek onun işaretini kaldırır. **Bu deponun varsayılanları olarak kaydet** ile birlikte deponun *Ek dosya yok* anahtarı olur. Boş bir alan burada "hiçbiri" sayılır, çünkü pencere çalıştırmanın devralacağı değerlerden başlar. |
| **Bu deponun varsayılanları olarak kaydet** | Değerleri sonraki incelemeler için de saklar: deponun durum dosyasında ya da klonlanmamış bir depo için `remote-reviews.toml` içinde. Yalnızca genel olandan farklı bir değer saklanır; böylece geri kalanı *Ayarlar > YZ*'yi izlemeye devam eder. |

Her şeye `Tab` ile ulaşılır, `Boşluk` bir onay kutusunu değiştirir ve yol alanı sıradan bir metin alanıdır. Aynı
dört ayar *Ayarlar > YZ* (*Kod incelemesi*) ve *Depoya özel ayarlar > YZ* içinde de bulunur; bkz.
[İnceleme bağlamı](ai.md#i̇nceleme-bağlamı). Modele verilemeyen bir şey (eksik veya okunamayan bir ek dosya) ve
sınırlara sığması için kısaltılan her dosya taslak kartında listelenir.

Öneriler **yorum değildir**. İki yerde görünürler:

- **Dosyalar'da satırlarının altında**, `✦` ile işaretli, bir önem derecesi (`bilgi`, `uyarı`, `sorun`) ve üç düğmesi
  olan hayalet yorumlar olarak: **Kabul et**, **Düzenle** ve **Yoksay**. Diff'in göstermediği bir satır hakkındaki
  öneri tutulur, asla atılmaz: hunk'lardan sonra *Tüm dosya hakkında YZ önerileri* altında listelenir ve kabul
  edildiğinde, önünde *(satır N hakkında)* sözleriyle dosyanın ilk değişen satırına düşer.
- **Genel Bakış'ta**, konuşmanın üstündeki bir blokta (yalnızca bir çalıştırmadan sonra): özet, ardından aynı düğmeler ve **Tümünü kabul et** / **Tümünü yoksay**
  ile liste.

**Kabul et** (`y`) bir öneriyi sıradan bir **bekleyen yoruma** çevirir (yine düzenleyebilir veya silebilirsiniz);
**Düzenle** (`w`) yorum kutusunu önce modelin sözleriyle açar ve bıraktığınızı kabul eder; **Yoksay** (`n`) onu
bırakır. **Tümünü kabul et** (`A`) yine yalnızca bekleyen listeyi doldurur. Dosyalar sayfasında tuşlar imlecin
altındaki öneriye, Genel Bakış'ta listenin ilkine etki eder (liste boşaldığında `y` ve `n` yine konuşmanın *Bağlantıyı kopyala* ve *Sonraki yeni kayıt* tuşlarıdır). **YZ özetini kullan** (`i` veya İncelemeyi bitir
formundaki düğme) modelin özetini inceleme metnine koyar; kendiliğinden asla eklenmez. **İncelemeyi gönderene kadar
barındırıcıya hiçbir şey gönderilmez.**

Genel Bakış taslağı, o bloğun en üstünde, önerilerin üstünde bir kart olarak gösterir: karar çipi, inceleme metni ve üç düğme.

- **✦ Taslağı kullan...** (`u`) açık her öneriyi bekleyen bir yorum olarak kabul eder ve *İncelemeyi bitir* formunu
  taslağın metni ve kararıyla önceden doldurulmuş olarak açar. Yine de onu okur, değiştirir ve kendiniz
  gönderirsiniz; formu iptal etmek taslağı korur. Barındırıcının sınırları korunur: GitLab'da *Değişiklik iste* yoktur,
  bu yüzden form bunu söyler ve *Yorum*'a döner; Gerrit'te karar, yalnızca sunucu bu değere sizin için izin veriyorsa
  +1 veya -1 Code-Review oyuna dönüşür (aksi halde oy ayarlanmaz ve form bunu söyler).
- **Metni düzenle...** (`t`) taslağı kullanmadan önce taslağın inceleme metnini değiştirir.
- **Taslağı at** (`N`) metni, kararı ve kalan önerileri atar.

Taslak istek başına tutulur; böylece incelemeyi kapatıp yeniden açtığınızda hâlâ oradadır. İsteğin head'i
modelin yazmasından sonra değiştiyse kart *İstek, bu taslak yazıldığından beri değişti.* der.
İncelemeyi göndermek taslağı harcar.

## Her şeyin bir tuşu ve bir palet satırı vardır

| Eylem | Tuş | Palet |
|---|---|---|
| Genel Bakış / Dosyalar / Commit'ler | `Alt+1` / `Alt+2` / `Alt+3` | *İnceleme: Genel Bakışı Göster*, *Dosyaları Göster*, *Commit'leri Göster* |
| Satıra veya aralığa yorum yap | `c` | *İnceleme: Satıra Yorum Yap...* |
| Konuşmaya yorum yap | Genel Bakış'ta `c`, `C` | *İnceleme: Konuşmaya Yorum Yap...* (Genel Bakış'ın kutusunu açar), *Konuşma: Yeni Yorum* |
| Bir dizinin kodunu aç (Genel Bakış) / yoruma dön | `G` / `Esc` | *Konuşma: Dizinin Kodunu Aç*, *İnceleme: Genel Bakışta Yoruma Dön* |
| Yorum dizisini yanıtla | `r` | *İnceleme: Yorum Dizisini Yanıtla...* |
| Çöz veya yeniden aç | `x` | *İnceleme: Yorum Dizisini Çöz veya Yeniden Aç* |
| Bir yorum dizisini katla veya aç | `z` | *İnceleme: Yorum Dizisini Daralt veya Genişlet* |
| Bekleyen bir yorumu düzenle / sil | `e` / `d` | *İnceleme: Bekleyen Yorumu Düzenle...*, *Bekleyen Yorumu Sil* |
| Dosyayı görüldü olarak işaretle | `v` | *İnceleme: Dosyayı Görüldü Olarak İşaretle* |
| Satır içi / Bölünmüş | `V` | *İnceleme: Satır İçi veya Bölünmüş Yerleşim* |
| Tüm dosya / yalnızca hunk'lar | `H` | *İnceleme: Tüm Dosya veya Yalnızca Hunk'lar* |
| Yalnızca boşluk değişiklikleri | `S` | *İnceleme: Yalnızca Boşluk Değişikliklerini Göster veya Gizle* |
| Sözcük kaydırma | `W` | *İnceleme: Sözcük Kaydırmayı Aç/Kapat* |
| Sonraki / önceki hunk | `J` / `K` | *İnceleme: Sonraki Hunk*, *Önceki Hunk* |
| Sonraki / önceki dosya | `]` / `[` | *İnceleme: Sonraki Dosya*, *Önceki Dosya* |
| İncelemeyi bitir / at | `f` | *İnceleme: İncelemeyi Bitir...*, *Bekleyen İncelemeyi At...* |
| Yenile | `F5`, `Ctrl+R` | *İnceleme: Yenile* |
| Tarayıcıda aç / checkout | `o` / `b` | *İnceleme: Tarayıcıda Aç*, *Dalı Checkout Et* |
| Atananlar / inceleyenler / etiketler / kilometre taşı | `a` / `p` / `l` / `m` | *İnceleme: Atananları Düzenle...* ve üç kardeşi |
| Yama kümesini seç (Gerrit) | `P` | *İnceleme: Yama Kümesi Seç...* |
| Daha eski bir yama kümesiyle karşılaştır (Gerrit) | `D` | *İnceleme: Yama Kümelerini Karşılaştır...* |
| Değişikliği teslim et / değişiklikten vazgeç (Gerrit) | `U` / `Q` | *İnceleme: Değişikliği Teslim Et...*, *İnceleme: Değişiklikten Vazgeç...* |
| ✦ İnceleme taslağı... (YZ açık; inceleme seçeneklerini açar) | `s` | *İnceleme: ✦ İnceleme Taslağı...* |
| Bir öneriyi kabul et / düzenleyip kabul et / yoksay | `y` / `w` / `n` | *İnceleme: Öneriyi Kabul Et*, *Öneriyi Düzenle ve Kabul Et...*, *Öneriyi Yoksay* |
| Her öneriyi kabul et / yoksay | `A` / `N` | *İnceleme: Tüm Önerileri Kabul Et*, *Tüm Önerileri Yoksay* |
| YZ özetini inceleme metninde kullan | `i` | *İnceleme: YZ Özetini İnceleme Metninde Kullan* |
| ✦ Taslağı kullan (İncelemeyi bitir'i doldurur) | `u` | *İnceleme: ✦ Taslağı Kullan...* |
| Taslağın metnini düzenle | `t` | *İnceleme: ✦ Taslak Metnini Düzenle...* |
| ✦ Taslağı at | `N` | *İnceleme: ✦ Taslağı At* |
| Kapat | `Esc` | *İnceleme: Kapat* |

Görünüm açıkken paleti (`Ctrl+P`) açın: satırları onun üzerinde çalışır ve birinin şu anda neden hiçbir şey
yapmadığını söyler ("önce imleci bir kod satırına getirin").

## Klon olmadan inceleme

Makinenizde olmayan bir depodaki istek de incelenebilir. Radar'da satırındaki **İncele** (`e`) nasıl yapılacağını
sorar (bkz. [Radar](radar.md#bir-isteği-incelemek)); **API üzerinden incele**, bu aynı görünümü Ana Sayfa penceresinde,
Radar'ın üzerinde açar. Bu sayfadaki her şey aynı çalışır, şu farklarla:

- Kod (değişen dosyalar, her dosyanın diff'i, isteğin commit'leri, bir commit'in diff'i, dosyanın tamamı)
  sunucunun API'sinden okunur; bu yüzden büyük bir isteğe ilk bakış bir an sürer ve sunucunun diff göndermediği bir dosya
  (ikili veya çok büyük) diff göstermek yerine bunu söyler.
- Bekleyen yorumlarınız bir deponun kendi durumu yerine, yapılandırma klasöründeki `remote-reviews.toml` içinde,
  sunucu ve proje (`host/project`) altında tutulur. İncelemeyi kapatıp yeniden açtığınızda hâlâ oradadırlar ve
  incelemeyi göndermek onları harcar.
- Kodlama seçimi yoktur (*Klon olmadan kullanılamaz*) ve **Checkout** yoktur (önce depoyu klonlamanızı söyler).
- `Esc` veya köşedeki `✕` görünümden çıkar ve Radar'ı geri verir. Palet (`Ctrl+P`), bir deponun sekmesindeki
  gibi görünümün satırlarına ulaşır.

Klonu saklamak için **Klonla, sonra incele**'yi (veya paletin *Radar: Klonla ve İncele...* satırını) seçin:
Klonla iletişim kutusu isteğin bağlı hesabında, deposu seçili olarak açılır ve klon tamamlanınca inceleme yeni deponun sekmesinde açılır.

## Komut satırından ve ajanlar için

Aynı inceleme `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest` olarak
(ve onlardan yapılan MCP araçları) kullanılabilir: bkz. [CLI başvurusu](cli-reference.md). `threads` ayrıca zaten
gönderilmiş kararları (`reviews`) listeler. `suggest <n>` modele sorar (YZ açık) ve `verdict`, `body`, özetini ve önerilerini hiçbir
şey göndermeden JSON olarak yazdırır; böylece bir betik veya bir ajan, `submit --comment`'e verilecek yorumları
seçebilir.

Bunların hiçbiri klon gerektirmez: `--project owner/name` (birden çok entegrasyon yapılandırıldıysa `--id` ile)
`show`, `threads`, `comment`, `reply`, `resolve`, `submit` ve `suggest` komutlarını, geçerli dizinin hiçbir şey bilmediği
bir projeye karşı çalıştırır. `hosting review diff <n>` diff'i hunk'larıyla yazdırır (`--api` onu bir klonun içinde bile
sunucudan okur) ve `hosting file <path> --project P [--ref R]` bir dosyayı bir düzeltmede okur, örneğin projenin
`CLAUDE.md` dosyasını. İkisi de sunucunun API'sini okur: GitHub, GitLab, Gitea, Forgejo ve Gerrit.
