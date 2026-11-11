[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · **Türkçe** · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Fare ve hareketler

Fareyle yapabileceğiniz her şeyin bir klavye karşılığı da vardır. Bkz. [Klavye
kısayolları](keybindings.md). Alanların adları [arayüz turunda](interface-tour.md)
verilir.

## Genel kurallar

- Bir satırı seçmek için ona **tıklayın**. Bu her panelde, listede, menüde ve
  iletişim kutusunda çalışır.
- Bir satırı açmak veya çalıştırmak için ona **çift tıklayın**. Bir yerel dala,
  bir tag'e veya bir ref hapına çift tıklamak onda checkout yapar. Bir depo
  satırına çift tıklamak depoyu açar.
- Menüsü için bir satıra **sağ tıklayın**. `.`, `Shift+F10` veya Menü tuşu,
  seçili satır için aynı menüyü açar.
- **Tekerlek**, imlecin altındaki alanı kaydırır. Bir listede yalnızca görünümü
  hareket ettirir. Seçim, görünümün dışına kaysa bile olduğu yerde kalır.
  Seçimi yalnızca bir tıklama veya bir tuş hareket ettirir.
- **Shift+tekerlek**, sözcük kaydırma kapalıyken diff'te, Dosya Görünümünde ve
  düzenleyicide yana kaydırır.
- Bir düğmeyi, satırı veya kaydırma çubuğu tutamacını aydınlatmak için üzerine
  **gelin**. Birçok terminal fare imlecini değiştiremez; bu yüzden bu ışık
  çoğunlukla bir şeyin tıklanabilir olduğunun tek işaretidir. Bir araç ipucu için
  imleci bir düğmenin veya Log hücresinin üzerinde bekletin.
- Bir metin alanından çıkmak için **dışına tıklayın**. Yazdığınız korunur. Refs
  filtresi, `Enter` gibi, yazdığınızı uygular. Log arama çubuğu, `Esc` gibi,
  kapanır.

## Pencereleri ve iletişim kutularını kapatma

Çerçeveli her pencerenin ve iletişim kutusunun kenarlığının sağ üstünde bir `✕`
vardır ve kurallar her yerde aynıdır:

- Kapatmak için **`✕`'e tıklayın**. `Esc` aynı işi görür; pencereden önce en içteki
  şeyi (açık bir açılır liste, bir metin alanı, bir onay) kapatır.
- Bir iletişim kutusunu, yardım ya da bilgi panelini veya açılır pencereyi
  kapatmak için **dışına tıklayın**. Dışa tıklamak tam görünümleri (Ayarlar, Diff
  görünümü, Dosya Görünümü, düzenleyiciler, İnceleme, Pano) asla kapatmaz: onların
  dışı yoktur.
- Bir panelin **içine** tıklamak onu asla kapatmaz.
- Yazdığınız metni tutan bir form (yeni bir issue, bir pull veya merge request)
  dışa tıklamayı yok sayar; böylece yanlışlıkla yapılan bir tıklama metni
  kaybettirmez. `✕` ve `Esc` önce **Değişiklikler atılsın mı?** diye sorar;
  **Düzenlemeye dön** forma geri götürür. Hiçbir şey yazılmamış bir form ve tek
  satırlık bir iletişim kutusu hemen kapanır.
- Komut paleti, seçiciler (sekme, tema, dil), küçük İnceleme, Pano, birleştirme
  ve yazma pencereleri ile onay istemi de `✕` taşır. Bağlam menülerinin ve açılır
  listelerin `✕`'i yoktur: `Esc` ile ya da dışarıya tıklayarak kapanırlar.

## Ayarlar'daki listeler

Ayarlar'ın listeleri (Profiller, YZ profilleri, İş akışı, Şurada aç, Görseller)
fareyle şöyle kullanılır. Bir alt sekmeye (Genel ya da liste) geçmek için **ona tıklayın**; `Alt+Left` / `Alt+Right` da aynısını yapar.

- Seçmek için bir satıra **tıklayın**, alanlarına geçmek için **çift tıklayın**,
  seçmeden kaydırmak için **tekerleği** kullanın.
- Sırayı değiştirmek için bir satırı **basılı tutup yukarı veya aşağı sürükleyin**:
  bir çubuk nereye ineceğini gösterir, bırakınca yerleşir, `Esc` iptal eder.
  Klavye karşılığı `Alt+Yukarı` / `Alt+Aşağı`dır.
- Listenin altındaki **`+ Ekle`, `Çoğalt`, `Kaldır`, `↑`, `↓`** düğmelerine
  tıklayın. Tuşlar: `Insert`, `Ctrl+D`, `Delete`, `Alt+Yukarı` /
  `Alt+Aşağı`; bkz. [Ayarlar](settings.md#listeler).

## Açılır penceredeki listeler

Bu kurallar, filtre alanı olan bir açılır penceredeki her liste için geçerlidir:
komut paleti, seçicileri (tema, dil, grafik renkleri), sekme seçici, aranabilir
açılır listeler, issue referans listesi ve yardım.

- Bir satırı seçmek için ona **tıklayın**.
- Bir satırı, `Enter` gibi, çalıştırmak veya seçmek için ona **çift tıklayın**.
- **Tekerlek** listeyi kaydırır. Seçimi hareket ettirmez.
- Liste açılır pencereden uzunsa sağ kenarında bir **kaydırma çubuğu** belirir.
  Tutamacı sürükleyin veya yola tıklayın.

Filtre alanı olmayan kısa bir açılır liste satırı ilk tıklamada seçer. Bir
bağlam menüsü satırı ilk tıklamada çalıştırır. Bu listelerin tuşları
[Klavye kısayolları](keybindings.md#filtrelenebilir-açılır-listeler)
sayfasındadır.

## Kaydırma çubukları

Kaydırılabilen her alan, içeriği sığmadığında kenarlığında ince bir kaydırma
çubuğu gösterir: Log, Refs bölümleri, dosya listeleri, diff, Dosya Görünümü,
düzenleyici ve açılır pencere listeleri.

- Kaydırmak için **tutamacı sürükleyin**.
- Tutamacı imlecin altına getirmek için **yola tıklayın**: görünüm orantılı olarak
  oraya atlar ve düğmeyi basılı tutmak sürüklemeyi sürdürür.
- Kaydırmaya devam etmek için **bir uç okunda düğmeyi basılı tutun**. Kısa bir
  gecikmeden sonra başlar. Duraklatmak için okun üzerinden çıkın.
- Bir kaydırma çubuğu yalnızca görünümü hareket ettirir. Seçimi asla hareket
  ettirmez.

Diff'te ve düzenleyicide dikey kaydırma çubuğu ayrıca tüm dosyayı küçük
işaretlerle gösterir: eklenen satırlar için yeşil, silinen satırlar için
kırmızı, değişen satırlar için kehribar, imleç satırı için `◆` ve seçili metnin
göründüğü her diğer yer için `▌`. Merge düzenleyicisi her çakışmayı kaydırma
çubuklarında işaretler. Oraya gitmek için bir işarete tıklayın.

## Hatalar ve Etkinlik Günlüğü

- Etkinlik Günlüğü'nü girdisinde açmak için durum çubuğundaki **kırmızı hataya
  tıklayın**; girdideki **Kopyala / önerilen düğmeye tıklayın** (Yeniden dene,
  Önce Pull, Stash ve yeniden dene, Entegrasyonları aç).
- Etkinlik Günlüğü'nü açmak için durum çubuğunun sağ ucundaki **Etkinlik**'e
  tıklayın. Geçmek için bir sekmeye tıklayın, o satırları göstermek veya gizlemek
  için bir düzey onay kutusuna (veya **Tümü** / **Hiçbiri**'ne) tıklayın, ayrıntılarını
  açmak için bir Git işlemleri girdisine tıklayın, Uygulama sekmesinde **Günlüğü temizle**'ye
  tıklayın (önce sorar). Uzun satırları `Shift` + tekerlek ile veya
  yatay kaydırma çubuğuyla yana kaydırın. Bkz.
  [Etkinlik Günlüğü](troubleshooting.md#etkinlik-günlüğü).
- Durum çubuğunun sağındaki iş göstergesinde **`[x]` düğmesine tıklayın**: çalışan işi iptal eder; **etikete tıklayın**: iş panelini açar. Ana sekmenin sağ altındaki **Etkinlik** düğmesi (veya `Ctrl+N Shift+G`) Etkinlik Günlüğü'nü Tüm depolar kapsamında açar (Kapsam kutusu, sekmelerin altındaki bir açılır liste, bir depoyu ya da çalışma alanını seçer).

## Yeniden boyutlandırma

- Refs kenar çubuğu, Log ve sağ panel arasındaki **bir ayırıcıyı sürükleyin**. Klavye onları bir seferde bir adım kaydırır: `Ctrl+N Left` / `Ctrl+N Right` Refs | Log ayırıcısını, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` Log | sağ panel ayırıcısını kaydırır (ayırıcı okun gösterdiği yöne gider).
  O paneli daraltmak için asgari genişliğin ötesine sürükleyin. Klavyedeki karşılıkları `Ctrl+N Left` / `Ctrl+N Right` (Refs paneli ile Log arasındaki ayırıcı) ve `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (Log ile sağ panel arasındaki ayırıcı) tuşlarıdır, her biri bir adım, ayırıcı okun gösterdiği yöne gider; palette ise `Gezinme: Refs Panelini Genişlet`, `Gezinme: Refs Panelini Daralt`, `Gezinme: Durum Panelini Genişlet` ve `Gezinme: Durum Panelini Daralt` ile `Gezinme: Refs Panelini Daralt/Genişlet` ve `Gezinme: Durum Panelini Daralt/Genişlet` komutlarıdır.
- Yüksekliğini değiştirmek için bir Refs bölümünün **alt kenarlığını
  sürükleyin**.
- Sağ paneldeki commit mesajı bloğunun **alt kenarlığını sürükleyin**.
- Bir sütunu yeniden boyutlandırmak için **iki Log başlığı arasındaki
  kenarlığı sürükleyin** (varsayılan olarak boşluk, başlıktan ya da tablonun herhangi bir yerinden; Ayarlar > Log ve grafik > Sütun yeniden boyutlandırma) (klavyede: sütunu seçmek için `Shift+C`, sonra `]` / `[`).
- Sütunu solundaki içeriğe sığdırmak için **iki Log başlığı arasındaki kenarlığa çift tıklayın** (klavyede: sütunu seçmek için `Shift+C`, sonra `=`; `+` her sütunu sığdırır). Son sütunun kendi son hücresinde bir tutamacı vardır: o sütunu yeniden boyutlandırmak için sürükleyin, farkı Mesaj sütunu karşılar.
- **Log'u yana kaydırın** (`scroll` sütun düzeni; Ayarlar > Log ve grafik > Sütun düzeni ya da seçicinin `Yana kaydır` onay kutusu): tabloda yatay tekerlek veya `Shift`+tekerlek, alt kenarlıktaki çubuk (tutamacını sürükleyin, boş yola basın, uç oklarına tıklayın) ya da `Shift+Left` / `Shift+Right`. Varsayılan `fit` düzeninde sütunlar bölmeyi doldurur ve kaydırılacak bir şey yoktur.
- Sütunu taşımak için **bir Log başlığını sürükleyin**: adının bir hayaleti imleci izler, ayrıldığı başlık soluklaşır ve bir vurgu çubuğu ineceği yeri işaretler. Sütunlar bıraktığınızda taşınır; `Esc` iptal eder ve onu taşırken imleç açık el olur (klavyede: sütunu seçmek için `Shift+C`, sonra `<` / `>`).

GitPoulpi her boyutu hatırlar. Bkz.
[Boyutlar hatırlanır](interface-tour.md#boyutlar-hatırlanır).

## Başlık çubuğu, sekme satırı ve araç çubuğu

- Başlık çubuğunda **🐙'e tıklayın**: **GitPoulpi Hakkında** açılır. Hakkında
  penceresinde ahtapotun gözbebekleri imleci izler; logoya tıklamak onu bir sonraki
  renge döndürür, sağ tıklamak renklerin listesini açar, bir bağlantıya tıklamak onu
  açar.
- Programların listesi için başlık çubuğundaki **Şurada aç ▾'ye tıklayın**. Bkz.
  [Şurada aç](open-in.md).
- Sürüm notları sekmesini açmak için başlık çubuğundaki **sürüme tıklayın** (`v1.0.0` ya da `vX.Y.Z kullanılabilir`).
- [Genel arama](search.md) için başlık çubuğunda dişlinin hemen solundaki
  **büyütece ve `Genel arama`'ya tıklayın**.
- Ayarlar için başlık çubuğunda ×'in solundaki **dişliye tıklayın**.
- Çıkmak için başlık çubuğunun sağ ucundaki **×'e tıklayın**.
- Bir sekmeye geçmek için **sekmeye tıklayın**. Kapatmak için **`x`'ine
  tıklayın** (etkin sekmede ve imlecin altındaki sekmede görünür). Sırasını
  değiştirmek için bir **sekmeyi yana sürükleyin**. Sekme menüsü için **sekmeye sağ tıklayın**.
- Bir sekmeyi kapatmak için **orta tıklayın**, `x`'i gibi.
- Son sekmenin ardından **`+`'ya tıklayın**: bir menü açılır, Yeni sekme (`Alt+T`) ve sekmeleri kapalıyken Radar ile Çalışma alanları (`Alt+J`).
- Ana sekme için **ev simgesine tıklayın**.
- Açmak için **Radar veya Çalışma alanları sekmesine tıklayın**; `x`'i, orta tık ya da `Ctrl+W` onu kapatır. Çalışma alanını değiştirmek için **çalışma alanı rozetine tıklayın** (`Alt+W`).
- **Çık ve cd yap**'a tıklayın (GitPoulpi'yi `gp` kabuk sarmalayıcısıyla başlattıysanız sekmelerin sağında görünür): GitPoulpi kapanır ve kabuğunuz etkin sekmenin klasöründe kalır. Bkz. [Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md).
- Bir araç çubuğu düğmesini çalıştırmak için **ona tıklayın**. Pull, Push veya
  LFS'nin listesi için yanındaki **`▾`'ye tıklayın**. Sığmayan düğmeler için
  **`⋯`'ye tıklayın**.
- Bu depo için commit kimliğini seçmek üzere araç çubuğunun sağ ucundaki
  **profil açılır kutusuna tıklayın**.

## Ana sekme

- Bir satırı seçmek için **tıklayın**. Depoyu açmak için ona **çift
  tıklayın**.
- Bir favoriyi eklemek veya kaldırmak için **yıldıza tıklayın**.
- Bir satırdaki **Aç, Geç, Kapat veya Kaldır**'a ya da satırın menüsü için
  **`⋮`**'ye **tıklayın**. Aynı menü için bir satıra **sağ tıklayın**.
- Bölümü katlamak için **bir bölüm başlığına tıklayın**.
- En üstteki **Gözat, Klonla, Init, Tümünü daralt veya Tümünü genişlet**'e ve
  bir bölüm başlığındaki **Tüm sekmeleri kapat** veya **Tümünü kaldır**'a
  **tıklayın**.

**Gözat** klasör seçicisinde: oraya gitmek için bir yere veya klasöre tıklayın,
açmak için bir klasöre çift tıklayın, bir yol yazmak için yol alanına tıklayın.
**Kardeş klasörleri göster** kutusunu işaretleyin (`Alt+S`, hatırlanır); sol ağaç, açık her düzeyin diğer klasörlerini de listeler: açık yoldaki klasörler `▾`, ötekiler `▸` gösterir ve birine tıklamak oraya götürür.
**Klonla** ve **Init** içinde: yazmak için bir alana tıklayın, kullanmak için
bir düğmeye veya onay kutusuna tıklayın.

## Refs kenar çubuğu

- Bir satırı seçmek için **tıklayın**. Bir yerel dalda veya bir tag'de checkout
  yapmak için **çift tıklayın**. Bir worktree'yi veya alt modülü bir sekmede
  açmak için ona **çift tıklayın**.
- Soldaki simgeyi göstermek için bir satırın **üzerine gelin**. Ref'i Log'da
  gizlemek veya yeniden göstermek için ona **tıklayın**. **Solo** açıkken aynı
  yer ref'i solo yapar. Bir klasör satırında simge, klasördeki her ref'e etki eder.
- Her solo'yu veya her gizli ref'i temizlemek için kenar çubuğu başlığındaki
  **Solo'yu Durdur** veya **Tümünü Göster**'e **tıklayın**.
- Menüsü için bir satıra **sağ tıklayın**: checkout, merge, fast-forward,
  rebase, worktree oluşturma, yeniden adlandırma, silme, gizleme, solo ve daha
  fazlası. Bir uzak deponun kendi satırında **Fetch**, **Düzenle**, **Kaldır** ve
  diğer uzak depo eylemleri vardır. Bir stash'te **Pop**, **Apply** ve **Drop**
  bulunur. Grileşmiş bir satır neden çalışamadığını söyler.
- Bir uzak depo eklemek için UZAK başlığındaki **`+`'ya tıklayın**.
- Katlamak veya açmak için **bir klasör satırına tıklayın**.

### Pull request'ler ve issue'lar

| Hareket | Eylem |
|---|---|
| Bir satıra tıklamak | Seçer. |
| Bir satıra çift tıklamak | Tarayıcıda açar. |
| Bir satıra sağ tıklamak | Menüsü: aç, URL'yi veya numarayı kopyala, kaynak dalda checkout yap, bir issue'dan dal oluştur. |
| Bölüm başlığındaki `+`'ya tıklamak | **Pull Request oluştur** formunu veya **Yeni Issue** formunu açar. |
| Filtre satırına (`Açık ▾`) tıklamak | Hangi isteklerin veya issue'ların listeleneceğini seçer. |
| Bir grup başlığına tıklamak | Grubu katlar veya açar. |
| Sağlayıcı veya depo satırına (ISSUE'LAR) tıklamak | Issue kaynağını seçer. **Yok (bu deponun bağlantısını kaldır)** onu kaldırır. |
| Sorgu satırına (ISSUE'LAR) tıklamak | Bir JQL veya WIQL sorgusu yazar (Jira, Azure DevOps) ya da Trello listelerini seçer. |
| Araç çubuğu çipine (`GH #12`, `GL !53`, ...) tıklamak | Pull request'i tarayıcıda açar. **Tarayıcıda aç**, **URL'yi kopyala**, **Yenile** ve **Profili değiştir** için ona sağ tıklayın. |
| Soluk `+ PR` çipine tıklamak | Push edilmiş dal için bir pull request başlatır. |

Bkz. [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md).

## Log

- Bir commit'i seçmek için ona **tıklayın**. Ayrıntıları sağ panelde görünür.
- **Ctrl+tık** bir commit'i seçime ekler veya seçimden çıkarır.
  **Shift+tık** bir aralık seçer (Shift+tıkı kendi metin seçimi için ayıran
  kitty ve Windows Terminal gibi terminallerde **Alt+tık**). Hepsini cherry-pick
  yapmak için seçili bir commit'e sağ tıklayın. Seçim WIP satırını ve stash'leri
  içerebilir; iki veya daha fazla satır sağ paneli bir karşılaştırmaya çevirir, bkz.
  [Commit'leri karşılaştırmak](log-and-history.md#commitleri-karşılaştırmak).
- Log imlecini o satıra taşımak için karşılaştırma panelindeki **bir karta
  tıklayın**; Kaynak ve Hedef uçlarını değiş tokuş etmek için **Yer değiştir'e
  tıklayın**. Bir satıra düz tıklamak karşılaştırmadan çıkarır.
- Çalışma değişiklikleri için **WIP satırına tıklayın**. Commit özetini orada
  yazmak için **`// WIP` metnine tıklayın**.
- Tam metin için bir hücrenin **üzerine gelin**: mesaj, yazar ve e-posta, tam
  tarih, tam hash.
- Commit üzerindeki diğer ref'ler için **bir `+N` rozetinin üzerine gelin**.
  Orada bir ref'e tıklamak için açılır pencereye girin. Checkout yapmak için
  açılır penceredeki bir ref'e çift tıklayın.
- Checkout yapmak için **bir ref hapına çift tıklayın**.
- Commit menüsü için **bir commit'e sağ tıklayın**. Commit üzerindeki her ref
  için bir alt menüsü vardır.
- Ref'in eylemlerini ve commit'in eylemlerini içeren tek bir menü için **bir ref
  hapına sağ tıklayın**.
- Sütunları göstermek veya gizlemek için **başlığa sağ tıklayın**: başlığında ✕ bulunan bir onay listesi (`[x]` / `[ ]`); bir satıra tıklayın ya da oklarla `Space` / `Enter` kullanın, sonra ✕, `Esc` ya da dışarı bir tıklamayla kapatın (palet: `Görünüm: Log: Gösterilecek sütunları seç`). Onay kutularının altında: `<column> sütununu içeriğine sığdır` (sağ tıklanan sütun), `Tüm sütunları içeriğe sığdır` ve `Sütun genişliklerini sıfırla`.
- Önceki veya sonraki eşleşmeye gitmek ya da çubuğu kapatmak için arama
  çubuğundaki **`↑`, `↓` veya `✕`'e tıklayın**.
- O issue'yu veya pull request'i tarayıcıda açmak için sağ paneldeki bir commit
  mesajında **`#12`, `ABC-12` veya `AB#12`'ye tıklayın**.

### Sol panelin bölümleri

**Klasörler** (`a/b/c` adlı dallar ve tag'ler, ağaç görünümünde sağ panelin dosyaları) hepsi birden
açılıp kapanır: klasörü olan bir **bölümün başlığına veya boş kısmına sağ tıklayın**, **Tümünü
genişlet** / **Tümünü daralt** için (ve aşağıdaki onay listesine götüren **Bölümleri göster/gizle...**
için), **bir klasör satırına sağ tıklayın** ise **Tüm alt klasörleri genişlet** / **Tüm alt klasörleri
daralt** için (klasör ve altındaki her şey). Tuşlar, Refs panelinde, Unstaged ve Staged listelerinde ve
bir commit'in değişen dosyalarında aynıdır: `=` hepsini genişletir, `-` hepsini daraltır, `*` (veya
`Shift+Right`) imlecin altındaki klasörü ve altındaki her şeyi genişletir, `_` (veya `Shift+Left`) onu
daraltır. Palet: *Refs: Tüm Klasörleri Genişlet*, *Refs: Tüm Klasörleri Daralt*, *Refs: Seçili Klasörü ve
Alt Klasörlerini Genişlet*, *Refs: Seçili Klasörü ve Alt Klasörlerini Daralt* ve odaktaki dosya listesi için
dört *Dosyalar:* ikizi. Bir ad filtresi etkinken (Refs filtresi, bir commit'in *Dosyaları filtrele*
alanı), eşleşme içeren her klasör, ne daraltmış olursanız olun açık gösterilir ve kendi daraltmalarınız
değiştirilmez ya da kaydedilmez: filtreyi temizleyin, geri gelirler. Filtre sırasında yaptığınız bir
daraltma gösterilenler üzerinde işler ve filtreyle birlikte unutulur.

Bir bölüm başlığına (klasörü olmayan bir bölümün ya da sol panelin boş kısmına) **sağ tıklayın**: bölümlerinin onay
listesi açılır; Yerel, Uzak, Worktree'ler, Stash'ler, Tag'ler, Pull Request'ler, Issue'lar ve
Alt modüller. Bir tık bölümü işaretler ya da işaretini kaldırır; menü açık kalır. Gizli bir bölüm
çizilmez ve klavye onu atlar. Görünen son bölümün işareti kaldırılamaz. Seçim her depo için ayrı tutulur ve yeniden başlatmalarda korunur; kendi seçimi olmayan depo varsayılanı (genel `ui.sidebar_sections_hidden`) izler. *Tüm depolar için kullan* geçerli seçimi varsayılan yapar, *Varsayılana sıfırla* bu deponun kendi seçimini siler (palet: *Sol panel: Bu Bölümleri Tüm Depolar İçin Kullan*). Palet: *Sol panel:
Bölümleri Göster/Gizle...*, her bölüm için bir *Göster/Gizle* komutu ve *Bölümleri Varsayılana Sıfırla*.

### Ref'leri sürüklemek

Refs kenar çubuğundaki bir dala ya da Log'daki bir ref hapına basın ve onu
başka bir dalın üstüne sürükleyin. Durum çubuğu neyi taşıdığınızı söyler.
Bırakmayı kabul edebilecek bir hedef aydınlanır. Bir menü almak için bırakın.
`Esc` sürüklemeyi iptal eder.

| Sürükleme | Menü |
|---|---|
| Yerel X dalını yerel Y dalının üstüne | **Merge: X → Y**, **Rebase: X → Y üstüne**, **Etkileşimli rebase: X → Y üstüne...**, **Fast-forward: Y → X** |
| Bir yerel dalı bir uzak dalın üstüne | O uzak depoya **Push: X** |
| Bir uzak dalı yerel Y dalının üstüne | **Merge**, **Rebase: Y → onun üstüne**, **Fast-forward: Y → ona** ve Y'nin upstream'i ise **Pull** |

Eylem, checkout yapılmamış bir dalı değiştirdiğinde GitPoulpi önce onda checkout
yapar. Git checkout'u reddederse hiçbir şey değişmez. Tag'ler sürüklenemez ve
bırakma kabul edemez.

Klavye: "işaretle, sonra bırak" her ref çifti için sürüklemenin yaptığını yapar.
Refs panelinde bir dalın üzerinde `m` tuşuna basın (palet: `Refs: Bırakma Kaynağı
Olarak İşaretle`). Ref bir `◆` gösterir ve altı çizilir, Log'daki rozetlerinde de;
durum çubuğu nereye gideceğinizi söyler. Herhangi bir bölümden başka bir ref seçin
ve `Shift+M` tuşuna basın (palet: `Refs: İşaretli Ref'i Buraya Bırak...`): bir
sürüklemenin açtığı menünün aynısı açılır ve satırları tıpkı sürüklemeden sonra
olduğu gibi çalışır. Menü açılınca işaret harcanmış olur; `Esc` ya da işaretli ref
üzerinde `m` onu daha önce kaldırır. `Refs: Seçili Ref'i Checkout Yapılmış Dala
Bırak...` palet komutu, seçili ref ve geçerli dal için menüyü açar.

## Sağ panel

### Çalışma değişiklikleri

- Diff'ini açmak için bir dosyaya **tıklayın**. Log'a dönmek için **aynı dosyaya
  yeniden tıklayın**. Dosya düzenleyicide kaydedilmemiş değişikliklerle açıksa
  GitPoulpi önce sorar.
- **Dosyayı Stage'le** veya **Dosyayı Unstage'le** düğmesini göstermek için bir
  satırın **üzerine gelin**. Bir klasör satırında **Klasörü Stage'le** veya
  **Klasörü Unstage'le** vardır.
- Bir bölüm başlığındaki **Tüm Değişiklikleri Stage'le** veya **Tüm
  Değişiklikleri Unstage'le**'ye **tıklayın**.
- Katlamak için **bir bölüm başlığına** (`Unstaged Dosyalar`, `Staged
  Dosyalar`) **tıklayın**.
- Düz bir liste veya klasörler seçmek için **Yol veya Ağaç'a tıklayın**. Her
  klasörü katlamak için **Tümünü Daralt veya Tümünü Genişlet'e tıklayın**.
- Tüm değişiklikleri atmak için başlığın solundaki **`[✖]`'e tıklayın** (önce
  sorar). Değişiklikleri YZ ile açıklamak için sağdaki **`[✦]`'e tıklayın**.
- Menüsü için **bir dosyaya sağ tıklayın**: stage, at, yok say, LFS, dosyayı
  stash'le, geçmiş, blame, harici bir programda aç, yolu kopyala, yama oluştur,
  düzenle, sil. Çakışmalı bir dosya **... ile çöz** satırlarıyla başlar.
- Klasör menüsü için **bir klasör satırına sağ tıklayın**: stage, at, yok say,
  stash, yama oluştur, klasörü aç, içinde dosya oluştur.
- **Dosya Oluştur** için Unstaged Dosyalar'ın **boş kısmına sağ tıklayın**.
- **Yok say**, **İkili olarak ele al** ve **Metin olarak ele al** (dosya ve klasör menüleri) önce kuralın yazılabileceği `.gitignore` veya `.gitattributes` dosyalarının listesini açar. Bir satıra tıklayıp `Enter`'a basın (veya ok tuşlarını kullanın); `Esc` ya da dışarı tıklama hiçbir şey yazmadan iptal eder.
- Bir dosya veya klasör satırına **Ctrl+tık** onu seçime ekler veya çıkarır, **Shift+tık** (**Alt+tık**) aralık seçer; bir commit'in değişen dosyalarında ve Tüm dosyaları göster'de de aynıdır. Düz tık baştan başlar. **Seçili bir satıra sağ tık**: seçili tüm dosyaların menüsü.
- Bir listenin **boş kısmına sağ tıklama** ayrıca **Depo klasörünü aç** sunar; Unstaged ayrıca **Tüm Değişiklikleri Stage'le** ve **Tüm değişiklikleri at...**, Staged ise **Tüm Değişiklikleri Unstage'le** ekler.
- **Dönüştür** (dosya, klasör ve seçim menüleri) diskteki dosyaların kodlamasını, satır sonlarını veya girintisini yeniden yazar; tek bir `Ctrl+Z` toplu işi geri alır.

### Commit formu

- İçine yazmak için Özet'e veya Açıklama'ya **tıklayın**.
- Amend'i açmak veya kapatmak için **Önceki commit'i amend et'e tıklayın**.
- Commit etmek için **Değişiklikleri commit et'e tıklayın**. Düğme grileşmişse
  bir tıklama nedenini söyler (örneğin *Commit için değişiklikleri stage'le*).
- Mesajı YZ ile yazmak için Özet'in sağındaki **`✦`'e tıklayın**.
- Issue'ların listesi için Özet'in sonundaki **`#▾`'ye tıklayın**. Referansını
  eklemek için birine tıklayın.
- Commit composer'ı açmak için **YZ ile commit'ler oluştur'a tıklayın**.

### Commit ayrıntıları

- Diff'ini açmak için değişen bir dosyaya **tıklayın**. Log'a dönmek için ona
  **yeniden tıklayın**.
- Yeniden boyutlandırmak için mesajın alt kenarlığını **sürükleyin**. Mesajın
  üzerindeki tekerlek uzun bir açıklamayı kaydırır.
- *Özeti değiştir...*, *Açıklamayı değiştir...* veya *Yazarı değiştir...* için özete,
  açıklamaya ya da yazar bloğuna **sağ tıklayın**; tam Değiştir iletişim kutusunu imleç
  o alandayken açmak için birine (ya da yazım tarihine) **çift tıklayın**. Klavye karşılıkları `Ctrl+G Shift+E` (tüm alanlar) ve `Ctrl+G Shift+A` (yazar); özet ve açıklama için palette satırlar vardır.
- Değişen bir dosyaya, klasör satırına, listenin boş kısmına veya bir seçime **sağ tıklama** menüsünü açar (aç, geçmiş, blame, bir programda aç, yolları kopyala, yok say, ikili veya metin olarak ele al, dönüştür, commit'ten geri yükle, sil). **Ctrl+tık** ve **Shift+tık** birden çok dosya seçer.

Bkz. [Stage'leme ve commit'leme](staging-and-committing.md).

## Diff ve Dosya Görünümü

- Diff'in üstündeki çubukta **bir düğmeye tıklayın**: Dosya, Diff, Blame,
  `Geçmiş │ Tümü` sütun seçicisi, Düzenle, `↑` `↓`, Satır, Bölme, Hunk, WS,
  Kaydır, Harita, Kodlama. Açık olan bir düğme vurgulanır. `Geçmiş`'e ya da
  `Tümü`'ne tıklamak o listeyi commit sütununda gösterir; zaten seçili olan parçaya
  tıklamak sütunu gizler (bkz. [Commit sütunu](interface-tour.md#commit-sütunu)).
- Dosyayı kapatmak için başlıktaki **`✕`'e tıklayın**.
- **Hunk'ı Stage'le** veya **Hunk'ı Unstage'le** ile **Hunk'ı At**'ı göstermek
  için bir hunk'ın **üzerine gelin**.
- Satırın sağ ucunda kendi **+ Stage'le**, **− Unstage'le** ve **✖ At**
  düğmelerini göstermek için değişen bir satırın **üzerine gelin**.
- Hunk menüsü için **bir satıra sağ tıklayın**: hunk'ı veya seçili satırları
  stage'le, unstage'le ya da at, dosya geçmişi, blame, kopyala.
- Görünümü o satıra taşımak için **mini haritaya tıklayın**. Düğmeyi basılı
  tutup hareket ettirerek kaydırın.

Her atma önce sorar.

### Blame ve dosya geçmişi

- **Blame.** Hunk kartı için işaretçiyi bir ek açıklamanın üzerinde bekletin; tık bir hunk seçer, sağ tık menüsünü açar. Ek açıklama sütununu yeniden boyutlandırmak için kenarını sürükleyin (kaydedilir). Kaydırmak için minimap'te tıklayın ya da sürükleyin.
- **Dosya geçmişi.** Tık bir commit seçer; `Shift+tık` karşılaştırmak için bir aralık seçer; çift tık commit'i Log'da gösterir; sağ tık satır menüsünü açar; liste ile diff arasındaki kenarı sürükleyerek yeniden boyutlandırın; listedeki ✕ onu gizler (geri getirmek için `›` şeridine tıklayın); sağ tarafın neyi göstereceğini değiştirmek için çubuğun `Dosya` / `Diff` / `Blame` düğmelerine, sütunun listesini seçmek için `Geçmiş │ Tümü` seçicisine tıklayın. Tekerlek işaretçinin altındaki yarıyı kaydırır. Her birinin klavye karşılığı vardır ([Blame](log-and-history.md#blame), [Dosya geçmişi](log-and-history.md#dosya-geçmişi)).

### Commit sütunu (Tümü listesi)

- Log'un ipucu için **bir satırın üzerine gelin** (işaretçinin altındaki kısma göre mesajın tamamı, yazarın adı ve e-postası, tam tarih).
- Menüsü için **bir satıra sağ tıklayın**: Log'da göster, dosyanın o commit'teki diff'i, dosyanın o commit'teki hali, blame'i, ardından commit için Log'un menüsü. Menü tuşu aynısını klavyeden yapar.
- Soldaki bölümü (şeritler, mesaj, yazar, tarih, ref'ler, değişim veya SHA) boyutlandırmak için **başlıktaki bir sınırı sürükleyin** (`ui.column_resize`: varsayılan olarak iki bölüm arasındaki bir hücrelik ayırıcı, başlıkta soluk bir `│` ile işaretlenir ve imleç üzerindeyken sütun boyunca vurgu çizgisi olarak çizilir, boyunca herhangi bir yerden sürüklemek yeniden boyutlandırır; `header` ile sonraki adın ilk harfi, işaretçi üzerindeyken aydınlanır). Son bölümün hemen ardındaki hücre onun sınırıdır.
- Bir bölümü oraya taşımak için **başlıktaki bir adı başka bir adın üzerine sürükleyin**: Log başlığındakiyle aynı hayalet, soluk kaynak ve vurgu çubuğu; bölüm bıraktığınızda taşınır (`Esc` iptal eder; klavyede `Tab` sonra `<` / `>` aynı şeyi bir yer birer yer yapar).
- Hangi bölümlerin gösterileceğini seçmek için **başlığa sağ tıklayın** (Ref, Grafik, Yazar, Tarih, Değişim, SHA; Mesaj her zaman gösterilir); onay listesinin başlığında ✕ vardır. Palette `Sütunları seç` ve bölüm başına bir aç/kapat vardır.
- Satırlar sütundan genişse **Shift+tekerlek** ya da yatay tekerlek onları yana kaydırır.
- Sütunu yeniden boyutlandırmak için **sağ kenarını sürükleyin**. Geri getirmek ya da gizlemek için **`›` şeridine tıklayın** (ya da sütundaki ✕'e, ya da çubuğun `Geçmiş │ Tümü` seçicisinin **seçili parçasına tıklayın**).

### Metni seçmek ve kopyalamak

Diff'in, Dosya Görünümünün, Blame'in ve Dosya Geçmişi'nin metnini seçip kopyalayabilirsiniz.

- Metni seçmek için **basıp sürükleyin**. Kaydırdığınızda seçim aynı metinde
  kalır.
- Bir sözcüğü seçmek için **çift tıklayın**.
- **Düz bir tıklama** metni seçmez. İmleci hareket ettirir veya satırı seçer ve
  metin seçimini temizler.
- **`y` veya `Ctrl+C`** seçimi kopyalar. Seçim yoksa geçerli satırı kopyalar.
- **Seçimi kopyala**, **Satırı kopyala** ve **Hunk'ı kopyala** için diff'te
  **sağ tıklayın**.

GitPoulpi yalnızca dosyanın metnini kopyalar. Satır numaraları, `+` ve `-`
işaretleri, hunk başlıkları ve Blame sütunları kopyalanmaz. Yan yana
görünümünde bir seçim, sürüklemeye başladığınız tarafta kalır.

Diff'te bir sürükleme, `v` gibi, geçtiği satırları da seçer. `Enter` o zaman
bu satırları stage'ler. `Esc` önce metin seçimini, sonra satır seçimini
temizler, sonra diff'i kapatır.

Metni seçmek, o metnin diff'te ve düzenleyicide göründüğü her diğer yeri de
renklendirir.

## Metin alanları

Bu, her metin alanı için geçerlidir: filtreler, arama çubuğu, palet, Özet ve
Açıklama, iletişim kutusu alanları ve Ayarlar değerleri.

- İmleci oraya koymak için **tıklayın**.
- Metni seçmek için **basıp sürükleyin**.
- Bir sözcüğü seçmek için **çift tıklayın**.
- Seçimin yerine yazın. `Backspace` veya `Delete` onu siler.

Metin alanlarının tuşları [Klavye kısayolları](keybindings.md) sayfasındadır.

## Düzenleyici

- İmleci oraya koymak için **tıklayın**. Seçimi genişletmek için **Shift+tık**.
- Seçmek için **sürükleyin**. Kaydırmak için üst veya alt kenarın ötesine
  sürükleyin.
- Bir sözcüğü seçmek için **çift tıklayın**.
- **Tekerlek** imleci hareket ettirmeden kaydırır.
- Çubuktaki **Kaydet, Geri al, Yinele veya Bul'a tıklayın**. Bulma modunda **`↑`**
  ve **`↓`** önceki ve sonraki eşleşmeye gider.
- Kapatmak için **`✕`'e tıklayın**. Kaydedilmemiş değişiklikler varsa
  GitPoulpi önce sorar.

## Görseller

Görsel görünümünde fare için bkz. [Görseller](image-view.md).

## Pencereleri taşımak ve boyutlandırmak

Pencereler taşınabilir, büyük pencereler ayrıca yeniden boyutlandırılabilir:

- Bir pencereyi taşımak için **başlık çubuğunu sürükleyin** (kapatma çarpısı kendi tıklamasını korur). Komut paletinin ve seçicilerin ayrı bir başlık çubuğu yoktur: üst kenarlığı sürükleyin.
- Bir pencereyi boyutlandırmak için **sol, sağ veya alt kenarı** ya da bir alt köşeyi (`◢` tutamağı) **sürükleyin**. Pencere ekranda kalır ve hiçbir zaman en küçük boyutundan küçük olmaz (Geri alma geçmişi için 60 x 16, diğerleri için daha küçük). Boyutlandırılmış bir pencere yeri kullanır: metin alanları ve listeler büyür, metin yeni genişliğe göre kaydırılır.
- İmleç pencereden çıksa da sürükleme devam eder ve pencerenin dışında bırakmak onu asla kapatmaz.
- Geri alma geçmişinin Reflog / Kaybolan çalışma görünümü içindeki **ayraçları sürükleyin**; sıfırlamak için birine çift tıklayın.
- Klavye karşılıkları: `Ctrl+N` ardından `m` (oklar taşır, `Shift`+oklar boyutlandırır, `0` sıfırlar, `Enter` veya `Esc` çıkar) ve ayraçlar için `Alt`+oklar; paletteki `Pencere: Taşı veya Boyutlandır` ve `Pencere: Konumu ve Boyutu Sıfırla` satırları da vardır. Bunlar en üstteki pencerede çalışır. `Ctrl+N`'i kendi rozetleri için kullanan yüzeyler (commit mesajı formu, geçmişi yeniden yazma sayfası) `Ctrl+N`'lerinden sonra aynı `m`'yi alır ve durum satırı bunu söyler. Komut paleti ve seçiciler `Ctrl+N`'i listede aşağı inmek için kullanır; bu yüzden onlar için tuş `Alt+G`'dir (`palette.arrange`) ya da paletin kendi `Pencere: Taşı veya Boyutlandır` satırıdır: ikisi de öndeki açılır pencereyi düzenler ve açık bırakır.
- **Tek bir ayar** (`ui.dialog_geometry`, Ayarlar > Görünüm > Düzen) her pencereyi kapsar: her biri kendi konumunu ve boyutunu bu oturum için, sonraki açılış için diskte ya da hiç hatırlamaz.

| Pencere | Taşı | Boyutlandır |
|---|---|---|
| Geri alma geçmişi / Reflog / Kaybolan çalışma, Etkinlik günlüğü, işler paneli, klasör seçici | evet | evet |
| Komut paleti, sekme / tema / dil seçicileri, çalışma alanı araması | evet | evet (liste yükseklikle, satırlar genişlikle büyür) |
| Klon (depo listesi), Pull request ve Yeni issue (açıklama), "Gönderilenler" (bölme), Mermaid (önizleme), inceleme seçenekleri (alanlar, yalnızca genişlik) | evet | evet |
| Ayarlar uzun metin düzenleyicileri (şablonlar, hook'lar, AI istemleri), çalışma alanı formunun AI kutusu | evet | evet (metin alanı) |
| Geçmişi yeniden yazma: iki AI yardımcı penceresi | evet | evet (açıklama, gönderilen bölme) |
| Hakkında, güncelleme paneli, güvenlik denetimi, öngörülen çakışmalar, "AI sınırı aşıldı", etiket silme, hassas dosyalar | evet | hayır (boyut metni izler) |
| Init, Worktree, Sparse checkout, Flow penceresi, issue-flow pencereleri, commit mesajı (yeniden adlandırma) formu, Çalışma alanı formu, etiket / kilometre taşı seçicileri | evet | hayır |
| Onaylar (her "Emin misiniz?") | evet | hayır |
| İnceleme ve Pano pencereleri: küçük pencereleri (yorum, bitir, seç, sütunlar) | evet | hayır |
| Klavye sayfası açılır pencereleri (sıfırlama onayı, tuş yakalama), AI yanıt penceresi | evet | hayır |
| Ayarlar: depo kimlik doğrulama sonuçları ve onayı | evet | hayır |
| Geçmişi yeniden yazma: temizleme, kimlik ve soru pencereleri | evet | hayır |
| Commit yazıcısının dosya taşıma penceresi, birleştirme bildirimleri | evet | hayır |

Taşınamayanlar: tam sayfa ekranlar (Ayarlar, Yardım, zaten pencereyi dolduran İnceleme ve Pano sayfaları), bağlam menüleri ve açılır listeler (onları açan şeye bağlıdırlar), ipuçları ve sayfa içindeki küçük açılır pencereler (remote formu, commit bölme düzenleyicisi).

İmleç, başlık çubuğunun üstünde bir taşıma şekli, kenarların ve köşelerin üstünde bir boyutlandırma şekli gösterir (`OSC 22` destekleyen terminaller).

## Fare imleci şekli

Bazı terminaller GitPoulpi'nin fare imlecini değiştirmesine izin verir:
ayırıcıların üzerinde bir yeniden boyutlandırma imleci, tıklanabilir öğelerin
üzerinde bir el. Log'un, Radar'ın ve commit sütununun bir sütun sınırının üzerinde de (sürüklerken de) bir yeniden boyutlandırma imleci vardır. Bu kitty, foot, Ghostty, WezTerm ve yeni xterm'de çalışır (`OSC 22` dizisi). Yalnızca şekil değiştiğinde gönderilir, tmux, screen, zellij veya herdr içinde asla gönderilmez ve GitPoulpi'den çıkarken varsayılan imleç geri yüklenir.
Diğer terminallerde imleç değişmez. İmlecin altındaki ayırıcı veya düğme her terminalde aydınlanır.

Aynı türden bir bileşen uygulamanın her yerinde aynı imleci verir.

- **İşaret eden el** denetimlerin üzerindedir: düğmeler, onay kutuları ve anahtarlar, açılır listeler (Ayarlar'da, issue ve pull request panellerinde, Radar'da, iletişim kutularında), sekmeler ve çarpıları, `+` düğmeleri, bağlantılar ve referanslar (`#12`, `!45`), genişletme oklu grup ve klasör başlıkları, ref hapları, kaydırma çubukları ve durum çubuğunun Etkinlik, işler ve sürüm düğmeleri. Bir satırın içindeki onay kutusu ya da düğme eli korur.
- **Ok** satırlardadır. Bir satır, imleç üzerindeyken bütün olarak aydınlanır (her listede aynı ışık) ve kendi başına bir denetim değildir: Refs, Worktree'ler, Stash'ler ve Alt modüller panellerinin satırları, Log ve commit sütunu (yalnızca bir ref hapı eli alır), bir diff'in satırları, değişen dosyalar, pull request, issue ve Radar öğeleri ile iletişim kutularındaki ve panellerdeki listelerin satırları (klonlama, klasör seçici, Çalışma alanları, panonun kartları, İşler paneli, atanan, etiket ve kilometre taşı seçicileri, rebase adımları, inceleme dosyaları ve commit'leri, menü ve açılır liste seçenekleri).
- **Metin imleci (I-beam)** giriş kutularındadır, kutunun tamamı üzerinde: arama kutuları (genel arama, Ayarlar, komut paleti, Ana sayfa), filtreler (Refs, dosyalar, Radar, Etkinlik günlüğü, issue panelinin araması), Ayarlar'ın metin, sayı ve alan satırları (düzenlerken de), iletişim kutularının alanları, inceleme yorum kutuları, commit besteleyici, dosya düzenleyicinin metni ve merge düzenleyicinin düzenlenebilir sonucu (salt okunur panelleri oku korur). Commit ayrıntılarında mesaj ve açıklama düzenlenebilir olduğundan I-beam onlarda görünür ve bloğun tamamı hafifçe aydınlanır.
- **Commit ayrıntılarında** bilgi satırları denetimlerdir: kısa kimliği kopyalamak için `commit: <id>` üzerine tıklayın, düzeltmek için yazar ya da tarih satırına çift tıklayın (el ve hafif bir işaret onları belirtir). İstatistik satırı yalnızca bilgi verir ve oku korur.

Yeniden boyutlandırma imleçleri ayırıcıların ve sütun sınırlarının üzerinde kalır; açık el yalnızca bir ref taşırken görünür.

## Panoya kopyalama

**Yolu kopyala** ve diğer bazı kopyalama eylemleri metni terminale gönderir;
terminal de onu panoya koyar (OSC 52). Terminaliniz bunu desteklemiyorsa hiçbir
şey kopyalanmaz. Seçili metin için kopyalama eylemleri sistem panosunu doğrudan
kullanır.
