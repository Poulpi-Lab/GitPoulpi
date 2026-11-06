[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · **Türkçe** · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Geçmişi yeniden yazma

Geçmişi yeniden yazmak, zaten var olan commit'leri değiştirir: kimin ve ne zaman
yazdığını, mesajlarını, tuttukları dosyaları, imzalarını. GitPoulpi bunu bir
**kural seti** (bir kapsam ve sıralı bir kural listesi) ve ortadaki adımı asla
atlamayan üç adımla yapar:

1. **Önizleme**, hiçbir şeye dokunmadan yeni geçmişi hesaplar: sayılar, her
   kuralın ne yaptığı, taşınacak dallar ve tag'ler, değişen her commit için bir
   satır.
2. **Uygula**, yerel dallarınızı ve tag'lerinizi ona taşır. [Geri alma
   geçmişinin](undo-redo.md) tek bir girdisidir: **Geri al**, taşınan her ref'i
   (ve checkout edilmiş dosyaları) yerine koyar.
3. **Push** (lease'li bir force push) ve **Temizlik**, geri alınamadıkları için
   ayrı ve bilinçli adımlardır: bkz. [Sonucu push etme](#sonucu-push-etme) ve
   [Eski geçmişi silme](#eski-geçmişi-silme-temizlik).

Yeniden yazma ekranı, palet, `gitpoulpi rewrite` komutları ve MCP araçları aynı
motoru çalıştırır; bu yüzden birinde yapılan önizlemeyi başkası uygulayabilir.
[Komut satırından](#komut-satırından) bayrakları listeler;
[YZ ile kural oluşturma](#yz-ile-kural-oluşturma) ekranı bir açıklamadan doldurur; [YZ ajanlarından
gelen öneriler](#yz-ajanlarından-gelen-öneriler) bir ajanın burada neleri
yapıp neleri yapamayacağını söyler.

**Kurallar, neyi değiştirdiklerine göre**

| Grup | Kurallar | Bölüm |
|---|---|---|
| Yazarlar | Bir yazarı değiştir, Yazarları anonimleştir, .mailmap uygula | [Yazarlar ve .mailmap](#yazarlar-ve-mailmap) |
| Tarihler | Tarihleri kaydır, Saat dilimini değiştir, Saat aralığında tut, Commit tarihi = yazar tarihi | [Tarihler](#tarihler) |
| Mesajlar | Bul ve değiştir, Satırları sil, Trailer ekle veya kaldır, Mesaj kodlamasını düzelt | [Mesajlar](#mesajlar) |
| Dosya içerikleri ve yollar | Dosyalarda bul ve değiştir, Dosyaları kaldır, Büyük dosyaları kaldır, Yolları taşı veya yeniden adlandır, Satır sonlarını dönüştür, Çalıştırma bitini kaldır, Dosya kodlamasını dönüştür | [Dosyalar ve yollar](#dosyalar-ve-yollar) |
| LFS | Dosyaları LFS'ye taşı, Dosyaları LFS'den geri getir | [Dosyaları LFS'ye veya LFS'den taşımak](#dosyaları-lfsye-veya-lfsden-taşımak) |
| İmzalar | İmzaları kaldır (ve Uygula sırasında yeniden imzalama) | [İmzalar](#i̇mzalar) |
| Betik | Lua, shell, Python | [Betik kuralı](#betik-kuralı) |

**Başlamadan önce**

- Önizleme hiçbir şeyi değiştirmez. Uygula yalnızca yerel ref'lerinizi değiştirir
  ve asla push etmez.
- Yeniden yazılan commit'ler yeni kimlik alır. Eski geçmişe sahip herkese haber
  verilmelidir: [Sonucu push etme](#sonucu-push-etme) notu yazar.
- Push edilmiş bir gizli değer sızmış sayılır. **Önce onu yenileyin**, sonra
  yeniden yazın, force push edin ve temizleyin.
- Olduğu gibi kalan commit'ler kimliklerini ve imzalarını korur.

## Yeniden yazma ekranı

`Ctrl+G Shift+H` (ya da palette **Yeniden Yaz: Geçmiş Ekranı**) Log'un üzerinde
yeniden yazma ekranını açar. Araç çubuğunda düğmesi yoktur: gelişmiş eylemler
palette (her `Yeniden Yaz: ...` satırı ekranı ilgili kural zaten eklenmiş olarak
açar), commit menüsündeki **Geçmişi yeniden yaz** alt menüsünde (*Bu commit'ten
HEAD'e kadar...*, birden çok commit seçiliyse *Seçili commit'ler...*, *Yeniden
yazma ekranını aç...*) ve tuş akoru arkasında durur.

`Ctrl+N` yeniden yazma ekranının her denetiminde (kapsam, kurallar, alanlar, sekmeler, düğmeler) bir rozet gösterir; bir rozete basmak o denetime tıklamakla aynı şeyi yapar ve `Esc` rozetleri kaldırır (bkz. [Rozetler](keybindings.md#rozetler)).

![Yeniden yazma ekranı: kapsam çubuğu, iki "Bir yazarı değiştir" kuralının listesi, seçili kuralın alanları ve kaç commit'in değişeceğini söyleyen önizleme özeti](../images/history-rewrite.png)

- **Kapsam çubuğu:** hangi ref'lerin taşınacağı (geçerli dal, tüm yerel dallar,
  seçilen dallar ve tag'ler, seçili commit'ler), *Tag'ler* kutusu ve *Bu
  commit'ten HEAD'e kadar...* seçildikten sonra o commit'in ebeveyninden sonra
  başlamak için bir kutu.
- **Kurallar:** sırayla uygulanan numaralı bir liste. Kutu bir kuralı açar ya da
  kapatır, oklar onu taşır, çarpı siler (*Silinen kuralı geri getir* veya `Ctrl+Z`
  geri getirir). *Kural ekle* türleri gruplara göre listeler. Sizde olmayan bir
  programa ihtiyaç duyan kural, nedeniyle birlikte soluk kalır.
- **Kural ayarları:** bir metin alanına tıklamak onu düzenler; birkaç değeri olan
  bir alana tıklamak listesini açar; kutular çevrilir.
- **Önizleme** (`F5`) hiçbir şeye dokunmadan yeni geçmişi hesaplar ve bir ilerleme
  çubuğu gösterir. Sekmeler: **Özet** (sayılar, her kuralın ne yaptığı, uyarılar),
  **Commit'ler** (değişen her commit için bir satır, daemon'dan sayfa sayfa
  gelir; süzgeç değişenleri, kaldırılanları veya gönderilmiş olanları listeler),
  **Dallar ve tag'ler** (nelerin taşındığı ve zaten gönderilmiş korumalı dallar,
  her biri bir kutuyla), **Etki** (gönderilmiş geçmişi yeniden yazmanın ne
  demek olduğu: lease'li bir force push, çalışma arkadaşları, imzalar),
  **Gizli değerler**, **Kimlikler** ve **Boyut** (aşağıya bakın).
- **Bir commit'in dosyaları:** Commit'ler satırında `Enter` (ya da çift tıklama),
  içerik kurallarının o commit'te değiştirdiği dosyaları ve yanında seçili
  olanın farkını açar. Çarpı, `Esc` veya dışarı tıklamak kapatır. Gizli değer
  kuralının kaldırdığı satırlar gizlenir.
- **Uygula** (`Ctrl+S` veya `Ctrl+Enter`), önizleme kurallarla eşleştiğinde
  kullanılabilir. Varsayılanı olmayan sorular her seferinde sorulur: imzalı
  commit'lerin imzalarına ne olacağı ve bir klasörün nasıl çıkarılacağı. Sonuç
  paneli, taşınan her ref'i geri koyan **Geri al** düğmesini sunar.
- **Kaydet / Yükle**, bir kural setini depoda (paylaşılan) ya da yalnızca sizin
  için saklar; gizli değerler asla kaydedilmez.

Her düğmenin bir tuşu vardır: liste Ayarlar > Klavye ve fare altında (bağlam *Yeniden
yazma*) ve `keybindings.md` rehberindedir.

### Yazarlar ve .mailmap

- **Bir yazarı değiştir**, bir yazarı ya da commit edeni ada ve/veya e-postaya göre
  (düz metin ya da düzenli ifade) eşleştirir ve yeni bir ad ve/veya e-posta
  verir. *Şuna uygula* yazarı, commit edeni ya da ikisini seçer; *Tag
  yazarları da* yeniden oluşturulan açıklamalı tag'lerin onu izlemesini sağlar.
  Özgün commit eden, çalıştırma *ben, şimdi* demedikçe korunur.
- **Yazarları anonimleştir**, her kimliği numaralı bir yer tutucuyla değiştirir
  (`Contributor 1`, `contributor-1@anonymous.invalid`; iki desen de sizin, sayı
  için `{n}` ile). Aynı kişi bir çalıştırmada hep aynı numarayı alır; *korunacak*
  olarak listelediğiniz kimliklere dokunulmaz.
- **Kimlikler sekmesi** (palette **Yeniden Yaz: Kimlikleri Merge Et (.mailmap)...**
  da aynısını açar), birkaç ad ve e-postayla commit yapmış bir kişiyi **hiçbir
  şeyi yeniden yazmadan** düzeltmenin yoludur. Kapsam çubuğundaki kapsamın her adını
  ve e-postasını, her birinin kaç commit yazdığı ve commit ettiğiyle ve deponun
  `.mailmap` dosyasının hangilerini zaten eşlediğiyle (`mapped to Name <email>`)
  listeler. Aynı kişi olan kimlikleri işaretleyin (`Space` veya tıklama; *Tümünü
  seç* / *Hiçbirini seçme*), sonra **Şuna merge et...** (`m`): bir iletişim kutusu
  hangisinin kalacağını sorar (en çok commit'i olan ilk sıradadır) ve **diğer her
  kimlik için `.mailmap` dosyasına bir satır ekler** (`Name <email> Old Name
  <old@email>`; zaten var olan satır tekrarlanmaz, dosya yoksa oluşturulur).
  Hiçbir şey commit edilmez: paylaşmak için `.mailmap` dosyasını kendiniz commit
  edin. Log, blame ve dosya geçmişi kanonik adı hemen gösterir (`log.use_mailmap`
  ayarı, varsayılan açık; saklanan commit'ler hiç değişmez, bu yüzden force push
  edilecek bir şey yoktur). Durum çubuğu nelerin yazıldığını söyler ve önceki
  `.mailmap` dosyasını geri koyan **Geri al** (`u`) sunar (diğer her eylem gibi geri
  alma geçmişinin bir girdisidir). `r` listeyi yeniden okur.
- **Yeniden yazma yerine bir `.mailmap`.** Bir yazar kuralı zaten bir uzak depoda
  olan commit'leri değiştirecekse Özet, **.mailmap kullan** düğmesiyle birlikte
  *.mailmap, yeniden yazmadan görünümü düzeltir. Bunun yerine .mailmap
  kullanılsın mı?* der (Özet'te `m`). Kuralın eşleştirdiği kimlikleri işaretli olarak
  Kimlikler sekmesini açar ve kuralın ayarlayacağı kimliği, henüz hiçbir commit
  taşımasa bile, kalacak kimlik olarak önerir.
- **`.mailmap` kuralı** (*Yeniden Yaz: Geçmişe .mailmap Uygula...* veya
  `rewrite preview --mailmap`) gerçekten yeniden yazar: yazarlar ve commit edenler
  (seçiminize göre) çalışma ağacının `.mailmap` dosyasının adlandırdığı hale gelir,
  yani `git log --use-mailmap` çıktısı. Geçmişin kendisi kanonik adları taşımalıysa,
  örneğin yayımlamadan önce, bunu kullanın; diğer her kural gibi bir yeniden
  yazmadır (önizleme, Uygula, force push).
- **Kimlik kuralları ve trailer'lar.** Bir kimlik kuralı, trailer'larda adı geçen
  kişileri de değiştirebilir: **Trailer'ları da eşle**'yi (`--map-trailers`)
  işaretleyin. Değeri tam olarak `Name <email>` olan ve kuralın *E-posta şuysa* /
  *Ad şuysa* alanıyla eşleşen her trailer, kuralın yazara ve commit edene ne
  yaptığından bağımsız olarak yeni ad ve e-postaya ayarlanır. Varsayılan olarak
  kapalıdır: bir `Signed-off-by` kimin imzaladığını kaydeder, bu yüzden onu
  değiştirmek ayrı bir karardır.

### Tarihler

Dört tarih kuralı yazar tarihine, commit eden tarihine ya da ikisine uygulanır
(kuralın panelindeki *Tarihler*).

- **Tarihleri kaydır**, her tarihi bir miktar kaydırır (`+2h`, `-1d`, `90m`, `1d 2h`).
  Her commit'in yazıldığı saat dilimi korunur.
- **Saat dilimini değiştir**, tarihleri başka bir UTC farkıyla yazar. *Korunacak*
  neyin sabit kalacağını söyler: **an** (aynı an, saat farklı okunur) ya da **saat**
  (aynı yerel saat, an değişir).
- **Saat aralığında tut**, izin verilen saatlerin (örneğin `Mon-Fri 18:00-23:30;
  Sat,Sun 10:00-20:00`) dışına düşen her commit'i en yakın izin verilen zamana ya da
  yalnızca geriye ya da yalnızca ileriye taşır (*Taşıma yönü*). *Ebeveyninden hiç
  önce olmasın* geçmişin sırasını korur; *Tohum*, küçük rastgele dağılımı
  tekrarlanabilir yapar: aynı geçmişte aynı tohum aynı tarihleri verir.
- **Commit tarihi = yazar tarihi**, commit edeni (ad, e-posta ve tarih) yazarınkine
  ayarlar; bir `git commit --amend` ya da rebase'in geri aldığı şey budur.

Tarihler diğerleri gibi kurallardır: önizleyin, **Commit'ler** sekmesine bakın (her
satır eski ve yeni tarihi gösterir), sonra uygulayın. Bir commit'in kimliği
tarihlerine bağlıdır, bu yüzden bir tarih kuralı, dokunduğu ilk commit'ten sonraki
her commit'i değiştirir.

### Mesajlar

- **Mesajlarda bul ve değiştir** ile **Mesaj satırlarını sil**, düz bir metin ya da
  düzenli ifade alır (*Düzenli ifade*, *Büyük/küçük harf duyarlı*); boş bir
  değiştirme siler. Geçerli UTF-8 olmayan bir mesaja bunlar asla dokunmaz: düzenlenmesini
  istiyorsanız önce **Mesaj kodlamasını düzelt**'i çalıştırın.
- **Trailer'lar.** *Trailer ekle veya kaldır* (palette *Yeniden Yaz: Trailer Ekle
  veya Kaldır...*, ya da `--trailer-add`, `--trailer-remove`, `--trailer-map`), bir
  mesajın sonundaki `Key: value` satırlarını, yani `git interpret-trailers`'ın
  tanıdıklarını (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...) düzenler.
  **İşlem** açılır listesi *Ekle*'yi seçer (**Trailer** anahtarı, **Değeri** ve
  **Zaten varsa**: *Ekle (aynısı yoksa)*, *Var olanların yerine koy* veya
  *Commit olduğu gibi kalsın*), *Kaldır*'ı (o anahtarın her trailer'ı, ya da
  yalnızca değeri **Değer şunu içerir** alanını içerenler) ya da *Değeri
  değiştir*'i (**Değerde ara**, **Şununla değiştir** olur; örneğin `Signed-off-by`
  içindeki eski bir e-postayı yenisiyle). Anahtarlar büyük/küçük harf gözetilmeden
  eşleşir. Yalnızca trailer satırları değişir, mesajın geri kalanı tam olarak olduğu
  gibi kalır ve henüz trailer'ı olmayan bir mesaja bir boş satır ve kendi bloğu
  eklenir. Commit'ler sekmesi her değişikliği `+Signed-off-by: ...` /
  `-Co-authored-by: ...` olarak gösterir. Bir mesajın son paragrafı onun trailer
  bloğudur (başlık asla değil), bu yüzden gövdedeki `Note: see the ticket` gibi bir
  satır, son paragrafta başka trailer'larla birlikte tek başına olmadıkça bir
  trailer değildir.
- **Mesajlardaki commit kimlikleri.** **Commit kimliklerini güncelle**
  (`--update-message-hashes`, varsayılan olarak kapalı; mesajları düzenleyen
  kuralların panellerinin sonunda), mesajlarda yazılı kimliklerin yeniden yazmayı
  izlemesini sağlar: yeniden yazılan geçmişte tam olarak bir commit'i adlandıran 7
  veya daha fazla onaltılık karakterlik kısaltılmış bir kimlik (`This reverts commit
  1a2b3c4`, `see 1a2b3c4d5e`), o commit'in aynı uzunluğa kısaltılmış yeni kimliğiyle
  değiştirilir. Birkaç commit'le eşleşen, hiçbiriyle eşleşmeyen ya da yeniden yazma
  sonradan oluşturacağı bir commit'e ait olan olduğu gibi bırakılır; Özet kaçının
  değiştirildiğini ve kaçının bırakıldığını söyler.
- **Mesaj kodlamasını düzelt** (*Yeniden Yaz: Mesaj Kodlamasını Düzelt...* ya da
  `--fix-message-encoding [--fix-name-encoding]`) mesajları UTF-8 olarak yeniden
  kodlar, iki durumda:
  - Bir `encoding` üst bilgisi olan commit (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, git ve kodlama tablolarının bildiği her şey) çözülür ve UTF-8
    olarak yazılır, üst bilgi kalkar: metin her yerde aynı okunur, hiçbir
    dönüştürme gerekmez. O commit'in yazar ve commit eden adları aynı kodlamadaydı
    ve onunla birlikte çözülür, bu yüzden bir commit asla yarı dönüştürülmüş kalmaz.
  - **Mojibake**, bir zamanlar Latin-1 ya da Windows-1252 olarak okunmuş UTF-8
    (`Réécriture` yerine `RÃ©Ã©criture`), **yalnızca gidiş dönüş tam olduğunda**
    onarılır: metin geçerli UTF-8 olan baytlara geri kodlanmalı ve sonucu aynı
    biçimde bozmak özgün metni vermelidir. Gerçek harflerin yanında yalnızca `Ã©`
    içeren ya da eski alfabelerin dışında karakterler içeren bir mesaj bunu
    geçemez ve olduğu gibi kalır.

  *Adları da düzelt* (`--fix-name-encoding`) bozuk yazar ve commit eden adlarını
  aynı biçimde onarır. Karar verilemeyenler (bilinmeyen bir kodlama adı, bildirilen
  kodlamada geçerli olmayan baytlar, UTF-8 olmayan ve hiçbir şey bildirmeyen bir
  mesaj, bozuk görünen ama kanıtlanabilir biçimde öyle olmayan metin) **olduğu gibi
  bırakılır ve sayılır**: Özet *n mesaj olduğu gibi bırakıldı: kodlamasına karar
  verilemiyor* der ve `counts.encoding_left` sayıyı taşır. Commit'ler sekmesi her
  düzeltmeyi eski ve yeni mesajla ve bir *kodlama* satırıyla (`ISO-8859-1`,
  `UTF-8` olur) gösterir.

### Dosyalar ve yollar

- **Dosya içeriği kuralları** (*Dosyalarda bul ve değiştir*, *Dosyaları kaldır*,
  *Büyük dosyaları kaldır*): panel her değeri siz yazdıkça denetler ve nedenini
  alanın altında gösterir (derlenmeyen bir desen, hatalı bir yol deseni, bir
  bayttan küçük bir boyut). Boyutlar birim alır (`500 KiB`, `5 MiB`, `1 GiB`).
  Eğik çizgisiz bir yol deseni her derinlikte eşleşir. *Gizli değerler* maskeli
  bir alana yazılır, yalnızca bu çalıştırmada kullanılır ve asla kaydedilmez;
  önizleme bunları içeren satırları gizler. *Boş commit'ler*, kuralların boş
  bıraktığı bir commit'in kaldırılıp kaldırılmayacağını belirler (varsayılan
  kaldırılır). İkili dosyalar ve boyut sınırını aşan dosyalar olduğu gibi bırakılır
  ve Özet'te sayılır.
- **Yolları taşımak.** *Yolları taşı veya yeniden adlandır* (palette *Yeniden Yaz:
  Yolları Taşı veya Yeniden Adlandır...*) **Taşıma** açılır listesinde seçilen üç
  türe sahiptir:
  - *Bir yolu yeniden adlandır* (**Nereden**, **Nereye**; `--move-path FROM=TO`):
    eski yola sahip her commit'te aynı dosya ve klasörler yeni yolda görünür; başka
    hiçbir şey değişmez, her dosya kimliğini korur. Yeni yolun zaten var olduğu bir
    commit asla üzerine yazılmaz: önizleme reddedilir ve çakışan commit'lerle yolları
    listeler (boş bir ad seçin ya da önce var olanı taşıyın). Yalnızca tam yollar,
    joker karakter yok.
  - *Her şeyi bir klasöre taşı* (**Klasör**; `--into-subdir DIR`): her commit'in her
    yoluna önek eklenir. Depoları ad çakışması olmadan birleştirmek için yararlıdır.
  - *Bir klasörü kök yap* (**Klasör**; `--extract DIR`): klasör kök olur, ona
    dokunmayan commit'ler atılır (onu oluşturan commit'in ebeveyni olmaz), artık
    hiçbir şeyi birleştirmeyen merge'ler sadeleştirilir ve klasörden hiçbir şeyi
    olmayan dallar ve tag'ler silinir. Nasıl biteceği **her seferinde** sorulur:
    - *Yeni bir klona, yeni sekmede açılır* (`--extract-mode clone`): bu deponun bir
      kopyası seçtiğiniz bir klasörde yapılır (yeni depo, içinde çıkarılan klasörün
      adını taşıyan bir klasör alır), orada yeniden yazılır ve yalnızca yeni
      geçmişi, klasöre sahip her yerel dal ve tag'i tutacak biçimde temizlenir. Bu
      depoya dokunulmaz. Yeni deponun uzak deposu yoktur (özgün depo onun upstream'i
      değildir).
    - *Yerinde: bu deponun geçmişi yeniden yazılır* (`--extract-mode in-place`):
      olağan yeniden yazma, önizleme, Uygula, geri alma. Klasörü olmayan dallar
      silinir (geri alma onları geri getirir); checkout edilmiş dalda klasör olmalıdır.
    - *Bu deponun alt modülü olarak* (`--extract-mode submodule`): yeni depo klon
      durumundaki gibi oluşturulur, sonra geçerli dalda **tek bir yeni commit**,
      klasörün yerine onun bir alt modülünü koyar (`.gitmodules` ve bağlantı);
      klasör checkout edilmiş bir alt modül olarak kalır. Bu deponun geçmişi
      **yeniden yazılmaz**: eski commit'leri klasörün dosyalarını korur, böylece onlara
      işaret eden hiçbir şey (tag'ler, diğer klonlar) bozulmaz. `.gitmodules`
      içindeki URL yeni deponun yerel yoludur: bir barındırıcıda durmaya başladığında
      değiştirin (`git config -f .gitmodules submodule.<folder>.url`). Klasörde
      düzenlenmiş veya izlenmeyen dosyalar varsa reddedilir. Geri alma düz klasörü
      geri koyar.
    Özet, önizlemenin yeni deponun geçmişini ve hangi klasöre gideceğini gösterdiğini
    söyler. Bir çıkarmadan sonra gelen kurallar (bir yeniden adlandırma, bir silme)
    çıkarılan ağaca uygulanır; bir klon ya da alt modül için çıkarma ilk yol kuralı
    olmalıdır. Yol kuralları içerik kurallarından sonra çalışır, bu yüzden bir silme
    glob'u yolları eskiden olduğu gibi adlandırır.

#### Satır sonları, çalıştırma biti ve dosya kodlamaları

Ekle menüsünün **İçerik** grubunda üç kural (ya da palet satırları *Yeniden Yaz:
Satır Sonlarını Dönüştür...*, *Çalıştırma Bitini Kaldır...* ve *Yeniden Yaz: Dosya
Kodlamasını Dönüştür...*). Üçü de **Dosyalar** (glob'lar; eğik çizgisiz desen her
derinlikte eşleşir; boş her dosya demektir) ve **Hariç dosyalar** alır. Yalnızca
glob'ların adlandırdığını, kapsamın her commit'inde değiştirirler. İkili dosyalara
(ilk 8 KiB içinde bir NUL baytı) asla dokunulmaz: Özet onları *olduğu gibi bırakıldı*
diye sayar.

- **Satır sonlarını dönüştür** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Şuna dönüştür*: LF (Linux, macOS), CRLF (Windows) ya
  da CR (klasik Mac). Eşleşen bir metin dosyasının her satır sonu seçilen olur, bu
  yüzden karışık sonlu bir dosya tek tip çıkar; zaten doğru olan dosyalar sürümlerini
  olduğu gibi korur. *.gitattributes'a uy* (varsayılan olarak kapalı,
  `--eol-follow-attributes`) her commit'in kök `.gitattributes` dosyasını okur:
  `-text` veya `binary` işaretli dosyalar atlanır ve `eol=lf` ya da `eol=crlf`
  işaretli dosyalar, seçtiğiniz yerine o satır sonunu korur.
- **Çalıştırma bitini kaldır** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Eşleşen dosyalar 100755 modundan 100644 moduna geçer; içeriklerine
  dokunulmaz. En az bir dosya deseni gerekir: her çalıştırılabilir dosyanın bitini
  kaldırmak betiklerinizi bozardı.
- **Dosya kodlamasını dönüştür** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Şuna dönüştür*: UTF-8 (varsayılan), BOM'lu UTF-8, UTF-16 LE ya da BE
  (bayt sırası işaretiyle yazılır), Windows-1252 veya ISO-8859-1. *Dosyaların
  kodlaması*: varsayılan olarak her dosyanın kodlaması algılanır (bir bayt sırası
  işareti, yoksa geçerli UTF-8, yoksa *Yedek*, değiştirmedikçe Windows-1252); kodlamayı
  adlandırarak zorlayabilirsiniz, ama yanlış bir seçim yanlış metin verir (Latin-1
  olarak okunan UTF-8 her aksanı ikiye katlar). Kaynak kodlamanın okuyamadığı ya da
  hedefin saklayamadığı bir karakter içeren (ISO-8859-1'de bir euro işareti) dosya
  tam olarak olduğu gibi kalır ve Özet'te sayılır.

Kurallar listenin sırasıyla, aynı dosya üzerinde çalışır: bir dosyayı UTF-16'ya
dönüştürüp ardından satır sonu istemek ona dokunmaz (artık düz metin değildir).
Uygulamak, diğer her yeniden yazma gibi geri alma listesinde tek bir girdidir.

### Gizli değerleri bulma

- **Gizli değerleri bulma:** `gitpoulpi rewrite scan-secrets` (ve `rewrite_scan_secrets`
  MCP aracı) geçmişi bir kez okur ve token öneklerini (GitHub, GitLab, Slack, AWS,
  Stripe, npm, Google, OpenAI, Anthropic), özel anahtarları, JWT'leri, URL içindeki
  parolaları ve rastgele görünen `password = value` satırlarını listeler; her biri
  dosyası, ilk ve son commit'i ve `ghp_****abcd` gibi gizlenmiş bir önizlemeyle
  gelir. Kapsamlı değildir ve değer asla gösterilmez, günlüğe yazılmaz veya
  kaydedilmez. `--remove-finding ID`, bir bulguyu onu her commit'ten kaldıran bir
  kurala çevirir. Gönderilmiş bir gizli değer sızmış sayılır: önce onu yenileyin,
  sonra yeniden yazın, force push edin ve temizleyin.
- **Gizli değerler sekmesi,** aynısını ekrandan yapar; kapsam çubuğundaki kapsam
  için: **Gizli değerleri tara** (`s`) taramayı bir iş olarak çalıştırır (ilerleme
  çubuğunun etiketi *Taranıyor*, **İptal** veya `Esc` durdurur). Her bulgu bir
  satırdır: türü, gizlenmiş önizleme, dosya ve kaç dosyada daha olduğu; listenin
  altında ilk ve son görüldüğü commit ile kaç commit ve dosya sürümünde bulunduğu
  yazar. Sekme, kaç ikili ve aşırı büyük dosyanın okunmadığını ve taramanın
  kapsamlı olmadığını söyler. Tüm bulgular işaretli başlar; **Geçmişten kaldır**
  (`r`), işaretlileri onları kimlikle (asla değerle değil) adlandıran tek bir
  kurala çevirir; bu kuralı diğerleri gibi önizler ve uygularsınız. Bulgular bir
  yeniden yazmadan veya kapsam değişikliğinden sonra bayatlar: sekme bunu söyler,
  yeniden tarayın. Ekran bir gizli değerin kendisini hiç tutmaz, bu yüzden onu
  gösteremez.

### Neyin yer kapladığını bulma

- **Neyin yer kapladığını bulma:** `gitpoulpi rewrite analyze [--top N]` (ve
  `rewrite_analyze` MCP aracı), `filter-repo --analyze` gibi boyut analizidir: yolu,
  boyutu, ilk ve son commit'i ve hâlâ HEAD'de olup olmadığıyla en büyük dosyalar, en
  büyük klasörler, HEAD'den silinmiş ama geçmişin hâlâ sakladığı dosyalar ve
  toplamlar. Yalnızca boyutları ve yolları okur, dosya içeriğini asla, ve hiçbir
  şeyi değiştirmez.
- **Boyut sekmesi** aynısını ekrandan yapar (`F7` ya da palette *Geçmiş Boyutlarını
  Analiz Et* onu açar ve çalıştırır): **Boyutları analiz et** (`s`) bir iştir
  (*Analiz ediliyor*, **İptal** veya `Esc` durdurur). **Göster** listeyi seçer
  (`v`): *Dosyalar* (en büyük benzersiz dosya sürümleri; her biri *HEAD'de*, hâlâ
  duran bir dosyanın *eski sürüm*ü ya da *silinmiş* olarak işaretlidir),
  *Klasörler* (her birinin altında hiç saklanmış olan her şey) veya *Silinenler*
  (HEAD'den kalkmış ama geçmişin tuttuğu yollar). **Sırala** (`o`) boyuta, yola ya
  da önce silinenlere göredir. Satırları işaretleyin (`Space`; hepsi / hiçbiri için
  `Alt+A` / `Alt+U`): sekme bunların ne kadar tuttuğunu "en fazla yaklaşık" diye
  dürüstçe söyler, çünkü alan ancak yeniden yazma, eski nesnelerin temizliği ve
  bir `gc` sonrasında geri gelir. **Geçmişten kaldır** (`r`), işaretli yolları bir
  yol kuralına çevirir (özel karakterler kaçırılır, bir kuralda zaten olan yol iki
  kez eklenmez); **... üstündeki her dosyayı kaldır** (`b`, Dosyalar listesi),
  işaretli en küçük dosyanın hemen altında bir boyut kuralı yazar. Bunları diğer her
  kural gibi önizleyin ve uygulayın. Listeler en büyük 200 satırı gösterir; CLI kaç
  tane daha olduğunu söyler ve `--top` onları listeler. Satırlar bir yeniden
  yazmadan veya kapsam değişikliğinden sonra bayatlar: yeniden analiz edin.

### İmzalar

- **İmzalar:** yeniden yazılan bir commit imzasını koruyamaz (eski baytları
  imzalamıştı), bu yüzden imzalı commit'ler yeniden yazıldığında her seferinde
  sorulursunuz: *İmzaları kaldır* (sayı önce gösterilir), *Yalnızca imzalı olan
  commit'leri yeniden imzala* veya *Yeniden yazılan her commit'i yeniden imzala*.
  Varsayılan yoktur. Olduğu gibi kalan commit'ler imzalarını korur. Yeniden imzalama,
  imzalı bir commit gibi kendi ayarınızı kullanır: `gpg.format` (`openpgp`, `ssh` ya
  da `x509`), `user.signingkey` ve `gpg.program` / `gpg.ssh.program` /
  `gpg.x509.program`. Sorunun dördüncü satırı **İmzalamayı denetle** (`4`), örnek bir
  veriyi bir kez imzalar ve doğrular: bir imzanın ne kadar sürdüğünü ve sorulan
  commit sayısı için çalıştırmanın tahmini süresini ya da neden çalışmadığını
  (anahtar yok, ajan kilitli, program kurulu değil; nasıl kurulacağıyla birlikte)
  söyler. Önizleme aynı denemeyi geçmişi okumadan önce yapar ve bu nedenle reddeder,
  böylece bozuk bir ayar yarı imzalanmış bir yeniden yazma bırakmaz. Commit'ler
  **önizleme sırasında** imzalanır (kimlikleri imzayı içerir), bu yüzden yeniden
  imzalayan bir önizleme imzalama süresini alır, Özet *Yeniden imzalanan commit'ler:
  n, 12,5 sn içinde* der ve o sırada bir parola istemi açılabilir: önce ajanınızın
  kilidini açın (İmzalamayı denetle tam olarak bunu yapar). **İptal**, iki imza
  arasında durdurur ve geride hiçbir şey bırakmaz. Yalnızca commit imzaları yeniden
  oluşturulur; imzalı bir tag imzasını yine kaybeder.
- **İmzaları kaldır kuralı** (Ekle menüsü, *İmzalar* grubu; palette *Yeniden Yaz:
  İmzaları Kaldır...*; `--remove-signatures`), başka hiçbir şey değişmese bile
  kapsamdaki her commit'in imzasını kaldırır ve yeniden imzalama hakkında hiçbir şey
  sormaz. CLI'dan: `rewrite check-signing` ve `--signatures
  strip|resign-signed|resign-all`.

### Yeniden yazmayı geri alma

**Geri al** (sonuç panelinin düğmesi, Log'da `Ctrl+Z` ile araç çubuğundaki *Geri al*
ya da `gitpoulpi undo`), taşınan her dalı, tag'i ve notu, ve onlarla birlikte
checkout edilmiş dosyaları, yeniden yazmanın sabitlediği eski commit'lerden yerine
koyar. Yeniden yazılan bir dal zaten upstream'indeyse (yerelde geri almak bir push'u
geri almaz) ve commit'lenmemiş değişiklikleriniz olduğunda önce sorar: onlar korunur
ve eski geçmişin üzerine yazacağı, sizin değiştirdiğiniz bir dosya geri almayı
durdurur (dosyalar `reset --keep` gibi taşınır). Bir yeniden yazmanın eski geçmişini
[temizlediğinizde](#eski-geçmişi-silme-temizlik) geri alma artık mümkün olmaz; geçmiş
listesi onu *temizlendi* diye işaretler.

## Dosyaları LFS'ye veya LFS'den taşımak

Ekle menüsünün *LFS* grubundaki iki kural, büyük dosyalar en baştan
[Git LFS](lfs.md) içinde yaşasın (ya da ondan çıksın) diye geçmişi yeniden yazar.
`git-lfs` programına ihtiyaç duyarlar: o olmadan kurallar kurulum ipucuyla soluk kalır
ve önizleme reddedilir. Bunlar `git lfs migrate` değil, kendi motorumuzdur; bu
yüzden önizlemeyi, kapsamı, geri almayı ve diğer her kuralın push korumasını alırsınız.

- **Dosyaları LFS'ye taşı** (*Yeniden Yaz: Dosyaları LFS'ye Taşı...* veya
  `--lfs-import PATTERN` ile `--lfs-min-size BYTES`). Desenlerle eşleşen
  (`*.psd`, `assets/**`; eğik çizgisiz desen her derinlikte eşleşir) ve en az *En az şu
  boyutta* kadar büyük olan (boş: her eşleşme) her dosya, onu tutan her commit'te bir
  LFS işaretçisi olur. İçeriği, Uyguladığınızda yerel LFS deposuna (`.git/lfs/objects`,
  her farklı içerik için bir kez) gider ve kök `.gitattributes`, bir işaretçi tutan her
  yeniden yazılmış commit'te `<pattern> filter=lfs diff=lfs merge=lfs -text` satırını
  alır (yoksa oluşturulur, diğer satırlara dokunulmaz, asla tekrarlanmaz). Zaten bir
  işaretçi olan dosyaya dokunulmaz. Özet *3 dosya (12 MiB) LFS'ye taşınıyor. Yazılacak
  LFS nesneleri: 3.* der ve Commit'ler sekmesi commit başına *big.psd LFS'ye taşınıyor*
  satırlarını listeler.
- **Dosyaları LFS'den geri getir** (*Yeniden Yaz: Dosyaları LFS'den Geri Getir...* veya
  `--lfs-export PATTERN`). Eşleşen işaretçiler yeniden gerçek dosyalar olur ve o
  desenlerin `.gitattributes` satırları kaldırılır. Nesneler yerel LFS deposunda olmalı:
  bazıları yoksa önizleme liste ve onları `origin`'den indiren bir **LFS nesnelerini
  indir** düğmesiyle reddedilir; sonra yeniden önizleyin. Aynı desenlerle bir içe
  aktarmayı bir dışa aktarma izlerse tıpatıp aynı commit'ler geri gelir.

Uygula'dan sonra checkout edilmiş dosyalar gerçek içeriktir (işaretçi değil) ve
**Geri al** eski geçmişi geri koyar. Sonuç paneli **LFS nesnelerini gönder** (`l`)
düğmesini taşır: `LFS: Tüm Nesneleri Push Et` ile aynıdır, böylece yeni geçmiş, sonraki
bir push eksik nesnelerde takılmadan gönderilebilir (yeniden yazılan dalların düz bir
push'u da, Uygula'nın kurduğu pre-push hook'u üzerinden bunları yükler). Ortak
çalışanlar için not, git-lfs'e ve `git lfs pull`'a ihtiyaç duyduklarını söyler.

Bilinmesi iyi olanlar: Uygula, eksikse deponun LFS süzgeçlerini kurar (`git lfs install
--local`), çünkü git gerçek dosyaları işaretçilerle karşılaştırmak için onlara
ihtiyaç duyar. Yalnızca kökteki `.gitattributes` yazılır, iç içe olan asla. Bir desenle
eşleşen ama en küçük boyutun *altında* kalan bir dosya düz dosya olarak kalır, ama git
bir sonraki add işleminde onu işaretçiye çevirmek ister, bu yüzden `git status` onu siz
commit edene kadar değişmiş gösterir. İçe aktarma büyük dosyaları akış olarak okur,
dışa aktarma her seferinde bir dosyayı bellekte tutar.

## Sonucu push etme

Bir yeniden yazmayı uygulamak hiçbir zaman push etmez. Bir uzak depoda bulunan yeniden
yazılmış dallar ve tag'ler force push gerektirir ve push asla körü körüne zorlamaz:
her ref, önizleme sırasında uzak deponun sahip olduğu kimlik üzerinde bir lease ile
gönderilir (izleme dalınızın şimdi söylediği değerle değil); böylece bir meslektaşın
araya giren push'u üzerine yazılmak yerine o ref için reddedilir ve sunucu izin
veriyorsa ref'ler hep ya da hiç (`--atomic`) tek bir push ile gider. CLI'dan:

```
gitpoulpi rewrite push --dry-run        # uzak depoya sorar, hiçbir şey göndermez
gitpoulpi rewrite push --yes            # gönderir
gitpoulpi rewrite note                  # diğerlerinin çalıştırması gerekenler
gitpoulpi rewrite maps --out ./maps     # commit-map ve ref-map
```

Korumalı dallar (`rewrite.protected_branches` ayarı; varsayılan `main`, `master`,
`develop`, `release/*`) onları adıyla belirtmedikçe (`--include-protected main`) ya da
hepsini dahil etmedikçe, hem yeniden yazmada hem push'ta dışarıda bırakılır. Kabul edip
etmeyeceğine barındırıcının kendi dal koruması karar verir; reddi ref başına gösterilir.

Ekranda, bir Uygula sonrasındaki sonuç paneli bunların hepsini yapar:

- **Yeniden yazılan ref'leri uzak depoya gönder,** yeniden yazmanın taşıdığı ve bir
  uzak depoda bulunan ref'leri listeler; her birinin bir kutusu vardır (`Space`).
  Korumalı dallar işaretsiz başlar ve yanlarında *korumalı* yazar: bunları tek tek
  işaretleyin ya da *Tümünü seç* / *Hiçbirini seçme* (`Alt+A` / `Alt+U`) kullanın.
  **Yeniden yazılan dalları push et** (`p`) işaretlileri gönderir; yanıt ref başına bir
  satırdır (*push edildi*, *zaten güncel*, *reddedildi: önizlemeden beri uzak depo
  ilerledi*, *reddedildi: sunucu tarafından korunuyor*, *atlandı: korumalı dal*,
  *push edilmedi*). Reddedilen bir satırda **Fetch et ve gözden geçir** (`f`) vardır:
  ekran kapanır ve uzak depolar fetch edilir, böylece Log diğerlerinin neyi push
  ettiğini gösterir. Hiçbir şey ezilmez: lease onların işini korur ve yeniden
  denemeden önce yeniden önizlersiniz.
- **Notu kopyala** (`c`), ortak çalışanlar için notu panoya koyar; çevrilmeyen git
  komutlarının çevresinde arayüz dilinde yazılmıştır.
- **Commit eşlemesini aç** (`m`), commit eşlemesinin ve ref eşlemesinin nerede olduğunu
  gösterir; bir yola tıklamak (veya `1` / `2`) onu kopyalar.

Bir push geri alınamaz: yeniden yazmayı sonradan `undo` ederseniz yalnızca sizdeki depo
değişir ve geri alma sormadan önce bunu söyler. Notu eski geçmişe sahip herkese gönderin:
her dal ve tag'in eski ve yeni ucunu ve komutları listeler (`git fetch`, sonra `git
reset --hard origin/<branch>` veya `git rebase --onto`). İki eşleme dosyası çalıştırmayla
birlikte 24 saat tutulur; GitLab'ın depo temizliği ya da bir betik onlara ihtiyaç
duyuyorsa kopyalayıp alın.

## Barındırıcı bir dalı koruduğunda

Barındırıcınız bir dalı kendi başına koruyor (GitHub dal koruması ve ruleset'ler, GitLab
korumalı dalları, Azure DevOps dal ilkeleri, Bitbucket dal kısıtlamaları) ve bir
yeniden yazmanın gerek duyduğu force push'u reddediyor olabilir. Depo barındırıcısına
**bağlı** olduğunda (kenar çubuğunun pull request bölümü, bkz. [Barındırıcı
entegrasyonu](hosting-integration.md)), yeniden yazma ekranı barındırıcıya hangi dalları
koruduğunu, sizin kendi token'ınızla sorar:

- Bu dallar **varsayılan olarak yeniden yazmanın dışında bırakılır**, tıpkı
  `rewrite.protected_branches` ve iş akışının korumalı dalları gibi: Ref'ler sekmesi
  onları her biri bir kutuyla ve *Tümünü seç* / *Hiçbirini seçme* düğmeleriyle
  listeler ve nedenini söyler: *GitHub üzerinde korumalı*.
- Token korumayı okuyamazsa (yetkisi yoksa ya da yönetici değilseniz) ekran
  *Barındırıcı koruması bilinmiyor* der ve barındırıcının sözüne dayanarak hiçbir şeyi
  korumalı saymaz. Yeniden yazma bu yüzden asla engellenmez. Bağlı olmayan bir depoya
  hiçbir zaman sorulmaz.

Korumalı bir dal yoldayken (önizleme her dal korumalı olduğu için reddedildiyse ya da
barındırıcı push'u reddettiyse: *reddedildi: sunucu tarafından korunuyor*), **Nasıl
aşılır** düğmesi (`Alt+H`) üç çıkış yolunu, sağlayıcınızın ve deponuzun tam ayar
sayfasıyla açıklar (bir bağlantı seçin ve `c` ile kopyalayın ya da `o` ile açın).
GitPoulpi bunların hiçbirini sizin yerinize yapmaz:

1. Barındırıcının ayarlarında **korumayı geçici olarak kaldırın**, push edin, sonra
   geri koyun.
2. **Yeniden yazmayı yeni bir dal adıyla push edin**, sonra o dalı varsayılan dal yapın
   (veya yeniden adlandırın) ve kimse kullanmaz olunca eskisini silin.
3. **Son çare olarak uzak depoyu silip yeniden oluşturun** ve her şeyi push edin. Bu,
   pull / merge request'leri ve inceleme geçmişlerini, issue'ları, fork'ların eski
   geçmişle bağlantısını, yıldızları ve izleyicileri, CI ayarlarını, gizli değerleri ve
   değişkenleri, webhook'ları ve deploy anahtarlarını kaybettirir.

CLI'dan aynı metin, `rewrite preview` / `rewrite run` (barındırıcı kapsamdaki bir dalı
koruyorsa), reddedilen bir `rewrite push` ve `rewrite note` JSON çıktısında
`protected_by_host` olarak gelir: `provider`, `branches`, `ways` (her biri kendi `steps`
ve `links` alanlarıyla), `recreate_loses` ve açıklamanın tamamı `text` olarak.

## Eski geçmişi silme (temizlik)

Bir yeniden yazmadan sonra eski commit'ler hâlâ deponuzdadır: yeniden yazmanın geri
alması onları tutar (geri alınabilir yapan budur), reflog'lar da öyle. Bir gizli
değerden kurtulmak için yeniden yazdıysanız o gizli değer hâlâ `.git` klasöründedir.
Temizlik eski geçmişi kalıcı olarak siler:

```
gitpoulpi rewrite purge                 # sorar: nelerin silineceğini ve kaybedileceğini listeler (çıkış kodu 3)
gitpoulpi rewrite purge --yes           # yapar
```

Yeniden yazmanın geri alma sabitlemelerini bırakır, eski commit'leri adlandıran reflog
girdilerini sonlandırır, `git gc --prune=now` çalıştırır ve her eski commit ile dosya
sürümünün gerçekten gittiğini denetler. Onay penceresi, neyi kaybettiğinizi önceden
söyler: yeniden yazma için (ve geri alması da eski commit'lere ihtiyaç duyan her önceki
işlem için) **geri alma artık mümkün değildir**; bunlar geçmiş listesinde temizlendi
olarak işaretli kalır. Bir temizlik başladıktan sonra durdurulamaz ve geri alınamaz. Bir
uzak depoya dokunmaz.

Eski geçmişi hâlâ tutan her şey temizliği engeller ve o zaman hiçbir şey değişmez. Liste
her birini ne yapılacağıyla adlandırır: yeniden yazmanın parçası olmayan bir dal veya
tag, eski geçmiş üzerindeki bir stash (atın ya da tutup eski geçmişin kalmasını kabul
edin), HEAD'i onun üzerinde olan başka bir worktree ve hâlâ eski geçmişi gösteren uzak
izleme dalları (yeniden yazılan dalları push edin, sonra fetch edin). Stash listenize ve
diğer dalların reflog'larına asla dokunulmaz.

Gönderilmiş bir gizli değer sızmış sayılır: **önce onu yenileyin.** Temizlikten sonra
yanıt, barındırıcının uzak depo başına hâlâ neye ihtiyaç duyduğunu listeler: GitHub eski
commit'leri önbellekteki görünümlerde ve pull request ref'lerinde erişilebilir tutar
(GitHub Support ile iletişime geçin ya da depoyu silip yeniden oluşturun), GitLab'ın
`commit-map` dosyasını alan (`gitpoulpi rewrite maps --out DIR`) bir depo temizliği ve
bakımı vardır, Azure DevOps ve Bitbucket bir destek talebi ya da yeni bir depo ister ve
klonu veya fork'u olan herkes hâlâ eski geçmişe sahiptir.

### Temizlik ekranı

Ekranda, bir yeniden yazmanın sonuç panelinde **Eski geçmişi temizle...** düğmesi (tuş
`e`) vardır ve palette **Yeniden Yaz: Eski Geçmişi Temizle...** bulunur (henüz
temizlenmemiş en yeni yeniden yazma üzerinde çalışır; ekranı kapattıktan sonra da).
Ekran, temizliğin nelerin sildiğini ve neyi kaybettiğinizi listeler (geri alması
onunla birlikte giden işlemler; *geri alma da kaybolur* işaretlileri yeniden yazma
değildir ama eski commit'lere ihtiyaç duymuştur), sonra eski geçmişi hâlâ neyin tuttuğunu
listeler, her biri bir satır:

- bir uzak izleme dalı: **Fetch** onları uzak depolardan günceller;
- bir stash: **Drop** önce sorar, çünkü bir stash hiç commit etmediğiniz bir çalışmadır;
- bir dal, tag, worktree ya da başka bir ref: **Adı kopyala,** sonra onunla bulunduğu
  yerde ilgilenin (dalı yeniden yazın veya silin, worktree'de yeniden yazılmış bir dalı
  checkout edin).

Denetim her düzeltmeden sonra yeniden çalışır ve **Yeniden denetle** (`r`) isteğe bağlı
çalıştırır. **Temizle** (`p`) yalnızca hiçbir şey kalmadığında kullanılabilir. Son bir
soru sorar, numaralı ve varsayılansız (`1` kalıcı olarak temizler, `2` her şeyi olduğu
gibi bırakır), sonra çalışır; ekran içinde bulunduğu adımı gösterir ve durdurulamayacağını
söyler. Sonuç, eski geçmişin bu klondan gittiğini ya da kaç eski nesnenin hâlâ orada
olduğunu söyler. Altında, her uzak depoda atılacak adımlar bulunur (bir bağlantı
kopyalanabilir). Çarpı, `Esc` ve dışarı tıklamak ekranı kapatır; temizlik çalışırken
hariç.

MCP üzerinden temizlik aracı, `config.toml` dosyasının `[mcp]` tablosunda
`allow_history_purge = true` ayarlamadıkça yoktur.

## Betik kuralı

*Kural ekle > Betik* (veya palette **Yeniden Yaz: Betik Çalıştır...**) bir Lua betik
kuralı ekler ve düzenleyicisini ekranın üzerinde açar; kuralın *Betik* satırında
`Enter` onu yeniden açar. Düzenleyici küçük bir şablonla başlar.

- **Dil** (kuralın panelindeki bir satır): *Lua (yerleşik)*, *Kabuk betiği*, *Python*
  veya *Lua (kurulu)*. Yerleşik Lua dışındaki her şey bir
  [harici betiktir](#harici-betikler-shell-python-lua): korumalı alanda değildir ve
  ayarlarda kapalı olan ya da yorumlayıcısı eksik olan bir dil listede nedeni yanında
  yazılı olarak soluk görünür (seçmek ne yapılacağını söyler). Henüz yazmadığınız bir
  betiğin dilini değiştirmek şablonu değiştirir.
- **Betik** satır numaralı bir metin düzenleyicidir. Yazdığınız şey anında kuraldır;
  onaylanacak bir şey yoktur.
- Altındaki **denetim** satırı `Tamam: rewrite_commit bulundu` der ya da sorunları
  satırlarıyla listeler (`script:3: ...`; kenar boşluğunda numara kırmızıdır). Yazmayı
  bıraktıktan kısa süre sonra çalışır, bir tuşa basılı tutulurken asla.
- **Parametreler,** betiğin başlığındaki her `--! param` satırı için bir satır içerir;
  etiketi alanın yanındadır: metin, sayı veya doğru/yanlış için bir kutu. Temizlediğiniz
  değer başlığın varsayılanına döner; varsayılanı olmayan bir metin parametresi
  Önizlemeden önce doldurulmalıdır.
- **Dosya** (deponun içinde bir yol; başlamak için `.gitpoulpi/rewrite/script.lua`),
  **Dosyadan yükle...** (metnin yerini alır) ve **Dosyaya kaydet...** ile.
- **Seçimde dene,** betiği Log'da seçtiğiniz commit'ler üzerinde, hiçbiri seçili
  değilse en yeni 20 üzerinde çalıştırır ve hiçbir şey yazmaz. Tablo her commit'i (kısa
  kimlik), değiştireceği her alanı `eski -> yeni` olarak ve betiğin `log(...)` satırlarını
  listeler. **İptal** beklemeyi durdurur. Betik başarısız olursa commit ve neden gösterilir.
- Çarpı, düzenleyicinin dışına tıklamak ve `Esc` onu kapatır; `Tab` ve `Shift+Tab`
  düzenleyici, parametreler, dosya, düğmeler ve sonuçlar arasında gezinir; `F5` dener.

Önizleme derlenmeyen bir betiği reddeder: Önizleme paneli, satırla birlikte *bir betik
başarısız oldu* der (bir commit üzerindeki başarısızlıkta kısa kimliği ve nedeni). Betiksiz
bir sürümde bu tür Ekle menüsünde soluk olarak kalır.

## YZ ile betik yazma

YZ açıkken ([YZ yardımı](ai.md)) Betik düzenleyicisinde bir `✦ YZ ile yaz...` düğmesi ve
paletle `✦ Yeniden Yaz: YZ ile Betik Yaz...` bulunur; bu, ekranı yeni bir Betik kuralı
üzerinde ve iletişim kutusu açık olarak açar. **Gönder**'e basana kadar hiçbir şey
gönderilmez: iletişim kutusu, siz yazarken, gidecek metnin tam olarak ne olduğunu gösterir.

- **Betik ne yapmalı?** Kendi sözlerinizle: *geceleri yapılan her commit'i sonraki sabaha
  taşı*, *eski e-postamı yenisiyle değiştir, yalnızca 2024'ten öncesinde*.
- **N commit örneği gönder** (kapalı): işaretlenirse, "Seçimde dene"nin kullanacağı
  commit'lerin (Log seçiminiz, yoksa en yeni 20) yazarı, tarihi ve konusu eklenir; böylece
  model verinizin neye benzediğini görür. Dosya içerikleri ve commit gövdeleri asla
  gönderilmez. Kutu kapalıyken makineden hiçbir commit verisi çıkmaz.
- **Gönderilenler**, biraz daha koyu bir arka planda salt okunur bir bölmedir ve iletişim kutusu
  açıldığında daraltılmıştır (iletişim kutusu o zaman kısadır). Göstermek için başlığına tıklayın,
  `Alt+W`'ye basın ya da `Tab` ile ona gidip `Enter`'a basın: yönergeleri (sabit betik
  sözleşmesi, sonra YZ yönergeleriniz) ve isteğinizi listeler, kendi kaydırma çubuğu vardır
  (sürükleyin, bölmenin üzerinde fare tekerleğini kullanın ya da odaktayken oklar, `PageUp`,
  `PageDown`, `Home` ve `End`). Bunları alan profil bir üstteki satırdadır. İletişim kutusu,
  ekran sınırları içinde bölmeye sığacak kadar büyür.
- **Gönder** (`Tab` ile ona gidin, `Enter`). Model çalışırken iletişim kutusu bekler ve
  `Esc` isteği iptal eder. Başarısız olursa iletişim kutusu nedeniyle birlikte açık kalır,
  böylece yeniden deneyebilirsiniz.

Yanıt, yazılmış bir betikle aynı daemon denetiminden geçer; bir derleme hatası modele bir
kez geri gönderilir. Betik sonra **düzenleyicideki metnin yerini alır** (düzenleyicide
başlangıç şablonundan başka bir şey varsa iletişim kutusu bunu söyler) ve siz metni
değiştirene kadar düzenleyicinin başlığında *✦ YZ yazdı: Önizlemeden önce okuyun* yazar.
Sizin yerinize asla denenmez, önizlenmez veya uygulanmaz: okuyun, **Seçimde dene**'ye
basın, sonra her betik gibi Önizleyin. Betik, sizin yazdığınızla aynı korumalı alanda
çalışır (dosya yok, saat yok, ağ yok).

Komut satırından: `gitpoulpi ai rewrite-script "<yönerge>" [--sample N]` betiği ve üzerinde
`rewrite script check` sonucunu JSON olarak yazdırır ve hiçbir şey çalıştırmaz (hâlâ
derlenmiyorsa çıkış kodu 2). Ayarlar > YZ içindeki *Geçmiş yeniden yazma betiği* profili
kullanılır.

## YZ ile kural oluşturma

YZ açıkken ([YZ yardımı](ai.md)) ekranda Kaydet ve Yükle'nin yanında bir `✦ YZ ile kural
oluştur...` düğmesi, `Alt+I` tuşu (Ayarlar > Klavye ve fare içinde *YZ ile yeniden yazma kuralları oluştur*,
bağlam *Yeniden yazma*) ve palette `✦ Yeniden Yaz: YZ ile Kural Oluştur...` bulunur. YZ kapalıyken
ya da YZ'siz bir derlemede düğme ve palet satırı yoktur. Geçmişin nasıl yeniden yazılması
gerektiğini kendi sözlerinizle anlatırsınız (*eski e-postam a@b.org'u c@d.org ile değiştir*,
*secrets.txt dosyasını tüm geçmişten sil*, *tarihlerimi iki saat kaydır*) ve model kuralları ve
ayarları önerir; bunlar **ekranı doldurur**. **Gönder**'e basana kadar hiçbir şey gönderilmez:
iletişim kutusu, siz yazarken, gidecek metnin tam olarak ne olduğunu gösterir.

- **Geçmiş nasıl yeniden yazılsın?** Çok satırlı bir alan: `Enter` yeni satırdır,
  `Ctrl+Enter` veya `Ctrl+S` gönderir, `Esc` ve çarpı iptal eder.
- **Kapsamı, dal ve tag adlarını ve geçmişin yazarlarını gönder** (kapalı): işaretlenirse
  kapsam çubuğundaki kapsam, dal ve tag adları ve Log'un gösterdiği en yeni commit'lerin
  (en çok 20 commit) farklı yazarları (ad ve e-posta) eklenir; böylece model e-postanızı tam
  olarak yazabilir (komut satırındaki `ai rewrite-rules --context` bunun yerine en yeni 200
  commit'i okur). Dosya içerikleri ve commit mesajları asla gönderilmez. Kutu kapalıyken
  makineden yalnızca açıklamanız ve sabit yönergeler çıkar.
- **Mevcut kuralları değiştir** (yalnızca ekranda zaten kurallar varken; kapalı): varsayılan
  olarak yeni kurallar sizinkilerin ardına eklenir; işaretlenirse onların yerini alır (yanıtın
  belirttiği kapsam ve ayarlar her iki durumda da uygulanır).
- **Gönderilenler**, yukarıdaki alanlardan ayrı, biraz daha koyu bir arka planda salt okunur bir
  bölmedir ve iletişim kutusu açıldığında **daraltılmıştır** (iletişim kutusu o zaman kısadır).
  Göstermek için başlığına tıklayın, `Alt+W`'ye basın ya da `Tab` ile ona gidip `Enter`'a basın:
  yönergeleri (kural dili, sonra YZ yönergeleriniz) ve isteğinizi listeler, kendi kaydırma
  çubuğu vardır (sürükleyin, bölmenin üzerinde fare tekerleğini kullanın ya da odaktayken oklar,
  `PageUp`, `PageDown`, `Home` ve `End`). Bunları alan profil bir üstteki satırdadır. İletişim
  kutusu, ekran sınırları içinde bölmeye sığacak kadar büyür. Açıklama alanı, metin sığmadığında
  kendi kaydırma çubuğunu alır.

Model bu sayfadaki her kural türünü kullanabilir; **Betik** (kod: *YZ ile betik yazma* betiği
incelemeniz için gösterir) ve *Dosyalarda bul ve değiştir*'in gizli değerler biçimi (gizli
değerler bir modele asla verilmez: Gizli değerler sekmesini kullanın) hariç. Kapsamı geçerli
dala, tüm yerel dallara ve tag'lere ya da var olan adlandırılmış dallara ve tag'lere
ayarlayabilir; *Tag'ler*, *commit eden*, *boş commit'ler* ve *mesajlardaki kısaltılmış
kimlikler* seçeneklerini de. Ekranın ifade edemediği şey (*Mart'tan beri* gibi bir tarih
sınırı, bir commit aralığı) tahmin edilmez: model bunu bir notta söyler. İmzalara ne olacağına
asla model karar vermez: Uygula'da size sorulur.

**YZ yalnızca kuralları ve ayarları hazırlar.** Hiçbir şeyi önizlemez, uygulamaz veya
çalıştırmaz: yanıt, yüklediğiniz bir kural seti gibi ekrana kopyalanır, ekrandaki önizleme
eskir; Önizleme ve Uygula (soruları ve onaylarıyla) her zamanki gibi sizindir. Her kural,
düzenleyicinin ve motorun kullandığı denetimle tek tek ayrıştırılır ve denetlenir; başarısız
olan kural dışarıda bırakılır. Bir şey dışarıda bırakıldıysa ya da model bir not düştüyse,
kurallar yerleştikten sonra bir liste bunu gösterir (*Dışarıda bırakıldı (geçerli kurallar
korundu)*, her biri konumu, adı ve nedeniyle) ve `Enter` veya `Esc` onu kapatır. Kuralları
okuyun, düzenleyin, sonra Önizleyin.

Komut satırından: `gitpoulpi ai rewrite-rules "<açıklama>" [--context]` öneriyi JSON olarak
yazdırır (`rewrite preview --recipe-json` komutunun okuduğu `recipe`, `problems`, `notes` ve
`applied: false`) ve hiçbir şey çalıştırmaz (bir şey dışarıda bırakıldıysa çıkış kodu 2). MCP
aracı `ai_rewrite_rules`'dur. Ayarlar > YZ içindeki *Geçmiş yeniden yazma kuralları* profili
kullanılır.

## Lua betikleri

Bir betik kuralı, işlevinizi her commit üzerinde çalıştırır. Bir kum havuzunda
çalışan düz Lua 5.4'tür: dosya, saat, süreç, modül ve `pcall` yoktur; aynı betik
aynı geçmişte hep aynı sonucu verir ve 10 milyon komutu, 64 MiB belleği ya da 5
saniyeyi aşan bir çağrı, commit'i belirten bir hatayla durur.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` şunları içerir: `id`, `parents`, `tree`, `author` ve `committer` (`name`,
`email`, `time`, `offset_minutes`), `message` ve `signed`. `ctx` şunları içerir:
`index`, `total`, `params`, `state` (sonraki commit'e veri taşımak için
`state = {...}` döndürün) ve `window.before` / `window.after` (`--! window: N`
verin). Yalnızca değişeni döndürün: `author`, `committer`, `message`. Yardımcılar:
`regex_replace`, `regex_match`, `stable_hash`, `rand`, `weekday`, `hour`, `log` ve mesajlar için `trailers(message)` (`{key, value}` listesi), `set_trailer(message, key, value)` (o anahtarın trailer'larının yerine koyar; dördüncü bir `"add"` ya da `"skip"` bağımsız değişkeni bunu değiştirir), `remove_trailer(message, key)` ve `at("2024-03-05 18:30", offset_minutes)` (bir UTC farkındaki tarihi Unix zamanı olarak verir).

Komut satırından: `gitpoulpi rewrite script check my.lua` betiği derler,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` hiçbir
şey yazmadan neyi değiştireceğini gösterir ve
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` tüm
yeniden yazmayı planlar.

## Harici betikler (shell, Python, Lua)

Bir Lua betiğiyle aynı sözleşme, makinenizdeki bir program tarafından çalıştırılır:
*Kabuk betiği* (bash veya sh; Windows'ta PowerShell ya da herhangi bir `.ps1`), *Python*
ve *Lua (kurulu)* (yerleşik olan değil, sizin kendi `lua`'nız; JSON kütüphanesi yoktur,
bu yüzden şablon satırı bir desenle okur). **Korumalı alanda değildirler: harici bir
betik sizin yetkilerinizle çalışır** ve sizin okuyabildiğiniz ya da değiştirebildiğiniz
her şeye erişebilir. Kuralın paneli ve düzenleyicinin başlığı bunu söyler.

Nasıl konuştuğu: GitPoulpi betiği çalıştırma başına bir kez başlatır ve standart
girdisine satır başına bir JSON isteği gönderir (bir commit'in alanları, `params`, son
yanıtın döndürdüğü `state`); betik standart çıktıda tek bir JSON satırıyla yanıt verir,
yani Lua betiğinin döndürdüğü aynı alanlarla (`author`, `committer`, `message`, `state`,
`log`), ya da vazgeçmek için `{"error": "metin"}` ile. Tanılama çıktısını standart hataya
yazdırın: commit'in `log` satırları olarak görünür. Başlık satırları kabuk ve Python için
`#!` ile başlar (`#! name: ...`, `#! param old: text "Old"`; bir shebang satırı
shebang olarak kalır), Lua için `--!` ile.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Neler alır.** `PATH`, `HOME` / `USERPROFILE`, `SYSTEMROOT`, `TEMP` / `TMP` ve `LANG`
dışında boş bir ortam; çalışma klasörü olarak yepyeni bir geçici klasör (betiğin kendi
kopyasını içerir); deponun yolunu almaz. **Süre sınırları:** her yanıt için
`rewrite.script_timeout_ms` (5000) ve tüm geçiş için `rewrite.script_total_timeout_s`
(3600). Aşıldığında, çöktüğünde, JSON olmayan bir şey yazdırdığında ya da iş iptal
edildiğinde GitPoulpi betiği ve başlattığı her şeyi sonlandırır; yeniden yazma commit ve
nedenle birlikte durur, hiçbir şey taşınmamıştır.

**Hangi diller var.** `config.toml` içindeki `rewrite.external_scripts` onları listeler:
varsayılan `["shell"]`; `"python"` ve `"lua"` değerlerini `config.toml` içinde kendiniz
ekleyin (`[rewrite]`, `external_scripts = ["shell", "python"]`; bir liste
`gitpoulpi config set` ile ayarlanamaz). Listelenmeyen bir dil ekranda soluk görünür ve
CLI ile MCP tarafından reddedilir (`scripts_disabled`); listelenmiş ama yorumlayıcısı
bulunamayan, kurulum ipucuyla birlikte soluk görünür (`tool_missing`).

**Güven.** Bir dil açık olsa bile bir betik yalnızca tam metnine güvendikten sonra
çalışır. Ekranda yazdığınız bir betik sizindir ve hemen çalışır; paylaşılan bir kural
setiyle ya da deponun bir dosyasıyla gelen değildir: Önizleme ve *Seçimde dene* durur ve
iki numaralı, hiçbiri önceden seçili olmayan yanıtla *Bu betiğe güvenilsin mi?* diye
sorar (`1` güven ve devam et, `2` güvenme, `Esc` olduğu gibi bırakır). Güvenmek, metnin
SHA-256 değerini bu depo için hatırlar; böylece değişen bir betik yeniden sorar ve
klonlanmış bir depo kendi kendine sizin üzerinizde kod çalıştıramaz. Komut satırından
dosyayı okuyun, sonra `gitpoulpi rewrite script trust DOSYA` (ya da `preview`, `run`,
`script check` ve `script test` üzerinde güvenip devam eden `--trust`). MCP üzerindeki bir
ajan hiçbir şeye güvenemez: yalnızca sizin daha önce güvendiğiniz bir betiği çalıştırabilir.

**Denetim.** Düzenleyicinin altındaki denetim satırı, güvenilmeyen bir betik için *Henüz
güvenilmiyor* der (hiçbir şey başlatılmamıştır); güvenilen için GitPoulpi onu uydurma bir
commit ile (`"dry_run": true`) bir kez başlatır ve yanıtın sözleşmeye uyduğunu denetler.
`gitpoulpi rewrite script check DOSYA` aynısını yapar ve `trust`, `hash` ve `interpreter`
bildirir.

## Komut satırından

`gitpoulpi rewrite` (ve aynı adlı MCP araçları) aynı motoru betikler ve ajanlar için
çalıştırır. Bir **kural seti**, ekranın kaydettiği bir TOML ya da JSON dosyasıdır
(`--recipe FILE` veya standart girdi için `-`) ve bayraklar kural setinin kendi
kurallarından sonra kural ekler.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plan, hiçbir şeyi değiştirmez
gitpoulpi rewrite apply --run ID --digest D           # gözden geçirdiğiniz planı çalıştırır
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # her şeyi geri koyar
```

- Bayrak olarak kurallar: `--match-email` ile `--set-name` / `--set-email` (kimlik),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. Kapsam geçerli daldır, `--all-local` ya da `--ref R`
  (tekrarlanabilir), `--since REV` noktasından.
- **Varsayılanı olmayan sorular** bayraktır: imzalı commit'ler değiştiğinde `--signatures
  strip|resign-signed|resign-all` ve bir çıkarma için `--extract-mode`. Onlar olmadan
  yanıt bir ret ya da bir `needs_choice` olur (çıkış kodu 3), asla bir tahmin değil.
- **Çıkış kodları:** 0 bitti; 1 bir hata (okunamayan kural seti, yanlış bayraklar); 2
  nedeniyle birlikte reddedildi (`stale`, `signature_choice_needed`,
  `dirty_paths_touched`, `empty_scope`, ...); 3 yanıtlanacak bir soru ya da `--yes`
  gerektiren push edilmiş ref'ler.
- `preview` planı depoda tutar, bu yüzden `apply` daha sonraki bir komuttan, daemon
  yeniden başladıktan sonra ya da başka bir arayüzden çalışır; özetiyle artık
  eşleşmeyen bir plan `stale` olur.

[CLI başvurusu](cli-reference.md) her bayrağı, [MCP ve ajanlar](mcp-and-agents.md) ise
bir ajanın hangi araçları aldığını söyler (temizlik aracı siz açmadıkça yoktur ve bir
ajan bir betiğe güvenemez).

## YZ ajanlarından gelen öneriler

Bir ajan bir yeniden yazma **önerebilir**, asla çalıştıramaz. `gitpoulpi ui propose
rewrite --recipe FILE|-` (`ui_propose_rewrite` MCP aracı, `[mcp] allow_ui_propose =
true` olana kadar kapalı) çalışan pencerenizde yeniden yazma ekranını, kural setinin
kuralları listede hazır ve mor bir bantla açar: *✦ <ajan> önerdi: bir geçmişi
yeniden yazma (3 kural) · Ctrl+Enter kabul eder · Esc reddeder*.

- **Hiçbir şey çalışmaz.** Önizleme, uygulama, push, temizlik yok: öneri yalnızca
  kuralları doldurur. Onları okursunuz, isterseniz düzenlersiniz, **Önizlersiniz** ve kendi
  tuşunuzla uygularsınız. Ekranı kapatmak öneriyi reddeder.
- **Ajanın içine koyamadıkları:** bir betik (önizlenirken çalışırdı), 50'den fazla
  kural, gizli değerler (gönderilmeden önce kural setinden çıkarılır). Böyle bir öneri
  nedeniyle birlikte reddedilir.
- Ajan sonucu `ui state` içinde okuyabilir (kuralları önerildiği gibi uyguladıysanız
  `accepted`, önce değiştirdiyseniz `accepted_edited`, kapattıysanız `rejected`). Aynı
  anda tek öneri; sizin başka bir iletişim kutunuz açıkken reddedilir.

## Araçlar

Geçmişi yeniden yazmanın çoğu Git'in ötesinde hiçbir şeye ihtiyaç duymaz: yazarları
ve commit edenleri değiştirmek, bir `.mailmap` uygulamak, anonimleştirmek, tarihleri
kaydırmak veya saat aralığına almak, mesajları ve trailer'ları yeniden yazmak,
içeriği değiştirmek, yolları ya da büyük blob'ları kaldırmak ve yolları taşımak
GitPoulpi'nin içinde çalışır.

**Lua betikleri yerleşiktir.** Yorumlayıcı GitPoulpi'nin içindedir; bu yüzden bir
Lua betiği için araç ya da kurulum gerekmez. Kabuk, Python ve kurulu Lua betikleri
yorumlayıcılarına ihtiyaç duyar (aşağıdaki tabloya bakın) ve
[Harici betikler](#harici-betikler-shell-python-lua) altında anlatılır.

Bazı özellikler dış bir program kullanır. GitPoulpi onu denetler (program `PATH`
içinde, Windows'ta ayrıca `PATHEXT` ve Git for Windows'un kendi `usr/bin`
dizininde aranır ve 2 saniye içinde `--version` sorusuna yanıt vermelidir); yanıt
bir dakika önbellekte tutulur ve "Yeniden denetle" yeniden sorar. Bulunamayan bir
programa ihtiyaç duyan kural listede soluk, nedeni ve nasıl kurulacağıyla
birlikte durur ve Önizle, hiçbir şey çalışmadan önce reddeder.

| Özellik | Araç | Notlar |
|---|---|---|
| Commit'leri yeniden imzalama | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) veya `gpgsm` (`gpg.format = x509`) | SSH ve X.509 imzalama ayrıca `user.signingkey` ister. `gpg.program`, `gpg.ssh.program` ve `gpg.x509.program` dikkate alınır. |
| LFS taşıma (iki LFS kuralı) | `git-lfs` 3.0 veya daha yenisi | LFS panelindeki denetimin aynısı. |
| Python betikleri | `python` 3.8 veya daha yenisi (Windows'ta `py -3`) | Varsayılan olarak kapalı: `rewrite.external_scripts` içine `"python"` ekleyin. |
| Lua (kurulu) betikleri | `lua5.4`, `lua54`, `lua` veya `luajit` | Varsayılan olarak kapalı: `rewrite.external_scripts` içine `"lua"` ekleyin. |
| Kabuk betikleri | `bash`, `pwsh` (PowerShell) veya `sh` | Varsayılan olarak açık (`"shell"`). Bir PowerShell betiği (`.ps1` dosyası, `pwsh` shebang'i ya da Windows'taki herhangi bir kabuk betiği) `pwsh` veya `powershell` ister; bir POSIX betiği `bash` veya `sh` ister (Windows'ta bunları Git for Windows sağlar). |

Kurulum ipuçları, önce Windows:

- Windows: Git for Windows `gpg`, `ssh-keygen`, `bash`, `sh` ve Git LFS'i
  içerir. Aksi halde `winget install GnuPG.GnuPG`, Ayarlar > İsteğe bağlı
  özellikler > OpenSSH İstemcisi, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: dağıtımın paketleri (`gnupg`, `openssh`, `git-lfs`, `python3`, `lua5.4`,
  `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` ve `sh`
  yerleşiktir.
