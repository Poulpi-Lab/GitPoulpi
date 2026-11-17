[English](../settings.md) · [Čeština](../cs/settings.md) · [Deutsch](../de/settings.md) · [Español](../es/settings.md) · [Français](../fr/settings.md) · [Italiano](../it/settings.md) · [日本語](../ja/settings.md) · [한국어](../ko/settings.md) · [Nederlands](../nl/settings.md) · [Polski](../pl/settings.md) · [Português (Brasil)](../pt-BR/settings.md) · [Português (Portugal)](../pt-PT/settings.md) · [Русский](../ru/settings.md) · [Svenska](../sv/settings.md) · **Türkçe** · [Українська](../uk/settings.md) · [简体中文](../zh-CN/settings.md)

# Ayarlar

Ayarlar'ı `Ctrl+,`, `Ctrl+N` ardından `,`, başlık çubuğunda ×'in solundaki dişli ya da
`Ctrl+P` > *Ayarlar* ile açın. Ayarlar ekranı açıkken pencerenin yerini alır. Sağ üstteki `✕` ile, `Esc` ile ya da
`q` ile kapatın. Yalnızca kapatmak bir şeyi kaybettirecekse önce sorar (bkz.
[Ayarlar'ı kapatma](#ayarları-kapatma)).

Değişiklikler hemen `config.toml` dosyasına kaydedilir.
[Yapılandırma başvurusu](config-reference.md) her anahtarı, varsayılan değerini ve
ortam değişkenini açıklar. Bu sayfa ise Ayarlar ekranını anlatır.

![Ayarlar ekranı: solda bir arama kutusu ve sayfa listesi, sağda Genel sayfası](../images/settings.png)

## Ayarlar ekranını kullanma

| Amaç | Yapılacak |
|---|---|
| Sayfa listesi ile seçenekler arasında geçmek | `Tab` / `Shift+Tab` ya da tıklama. |
| Gezinmek | `j` / `k` ya da ok tuşları. |
| Bir seçeneği değiştirmek | `Enter` ya da `Space`, ya da değerine tıklama. Aşağıya bakın. |
| Kapatmak | `✕`'e tıklayın ya da `Esc` veya `q`'ya basın. Bir alan, açılır liste ya da onay açıkken `Esc` yalnızca onu kapatır. |

`Enter` ya da `Space` tuşunun ne yaptığı satıra bağlıdır:

- Bir onay kutusu açılıp kapanır.
- Bir açılır liste açılır. Ok tuşları seçer, `Enter` kabul eder.
- Bir metin ya da sayı alanı düzenlemeye başlar. Ok tuşları, `Home`, `End`,
  `Backspace` ve `Delete` çalışır, bir tıklama imleci yerleştirir. `Enter`, `Tab`,
  `Up`, `Down` ya da başka bir yere tıklama değeri kaydeder. `Esc` iptal eder. Reddedilen
  bir değer alanda kırmızıyla kalır ve durum çubuğu nedenini söyler.
- Bir düğme satırı (**Oluştur**, **Kaldır**, **Dene**...) yalnızca şu an geçerli olan düğmeleri
  gösterir. `Left` / `Right` birini seçer, `Enter` basar ya da düğmeye tıklarsınız. Bir şeyi
  silen düğme önce sorar.

Her sayfada değerler tek bir sütunda başlar: etiketler solda, alanlar sağda hizalanır. Bir
metin ya da sayı renkli bir banttır, bir seçim `▾` ile biten bir banttır, bir onay kutusu
`[x]` biçimindedir. Uzun bir etiket kesilmez, alt satıra sarılır; dar bir pencerede değer
etiketinin altına iner.

Soluk (gri) bir satır neden etkin olmadığını söyler. Örneğin **Grafik şerit arka planı**
**Kapalı** iken **Satır tonu gücü** soluktur.

### Listeler

Aynı türden birkaç öğeyi düzenleyen sayfalar (Profiller, YZ, İş akışı, Şurada aç, Görseller)
bunları **solda bir liste, sağda seçili öğenin alanları** olarak gösterir. Dar bir pencerede
liste alanların üstünde durur. Liste her yerde aynı biçimdedir:

- Satırın işareti durumunu söyler: `●` etkin profil, `★` varsayılan giriş, `⚠` bitmemiş
  bir öğe (e-postası olmayan bir profil); ayrıntıların yeri olan yerde nedeni görünür.
- Listenin altında: **+ Ekle**, **Çoğalt**, **Kaldır**, `↑` ve `↓`. Şu an uygulanamayan bir
  düğme (ilk satırda `↑`) soluk görünür. Sayfa hazır ayarlar ya da türler sunuyorsa
  **+ Ekle ▾** bir açılır liste açar.
- **Kaldır**, az önce eklediğiniz ve düzenlemediğiniz bir öğe dışında önce sorar.
- Bir satırı taşımak için sürükleyin; bir çubuk nereye ineceğini gösterir, `Esc` sürüklemeyi
  iptal eder. Çift tıklamak öğenin alanlarına geçirir.

| Tuş | Ne yapar |
|---|---|
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Bir öğe seçer. |
| `Insert` | Öğe ekler (açılır liste varsa onu açar). |
| `Ctrl+D` | Öğeyi çoğaltır. |
| `Delete` | Öğeyi, bir onaydan sonra kaldırır. |
| `Alt+Up` / `Alt+Down` | Öğeyi yukarı veya aşağı taşır. |
| `Enter` veya `Right` | Öğenin alanlarına geçer. |
| `Left` | Sayfa listesine döner. |
| `Alt+Left` / `Alt+Right` | Genel sekmesi ile liste sekmesi arasında geçiş yapar (sayfanın her yerinde). |

Bu sayfaların her birinin başlığın altında iki **alt sekmesi** vardır; böylece sayfanın bir ayarı seçili öğenin bir özelliğiyle karıştırılmaz: **Genel**, sayfanın kendi ayarları (etkin profil, varsayılan giriş, her yapay zekâ eyleminin aldığı yönergeler, iş akışı şablonu...) ve öğelerin adını taşıyan, listeyi ve seçili öğenin alanlarını içeren bir sekme (**Profiller**, **Profiller (bağlantılar)**, **Girdiler**, **Düzenleyiciler**, **Kurallar**). Sayfa **Genel** sekmesinde açılır; Ayarlar, her sayfayı hangi sekmede bıraktığınızı uygulamayı kapatana kadar hatırlar; her sayfanın (ve sekmenin) kaydırma konumunu ve seçili satırını da. Bir sekmeye tıklayın ya da `Alt+Left` / `Alt+Right` tuşlarına basın. Arama kutusu, komut paletinin ayar satırları ve "ayarlardan yapın" diyen bağlantılar ayarın bulunduğu sekmeyi açar.
[Klavye ve fare](#klavye-ve-fare) sayfasında da aynı iki sekme vardır (**Genel**, **Tuş atamaları**).

Tuşlar **Klavye ve fare** bölümünde yeniden atanabilir (bağlam *preferences*, bkz.
[Tuş atamaları](keybindings.md)).

### Çok satırlı düzenleyiciler

Birden çok satır tutan ayarlar (örneğin kısmi checkout yolları, commit şablonunun açıklaması, çakışma önlemenin temel dalları ve iş akışı hook'ları) `Enter` veya bir tıklamayla küçük bir düzenleyicide açılır; `Ctrl+S` metni korur, `Esc` atar. Fare orada her metin kutusundaki gibi çalışır:

- Bir tıklama imleci tıkladığınız yere koyar.
- Sürüklemek metni seçer; tekerlek kaydırır.
- Çift tıklama, işaretçinin altındaki sözcüğü seçer.
- `Shift` + tıklama, seçimi imleçten işaretçiye kadar genişletir.

Klavye de aynısını sunar: `Shift` ile ok tuşları seçer, `Ctrl` ile ok tuşları sözcük sözcük atlar, `Ctrl+A` her şeyi seçer, `Ctrl+C` ve `Ctrl+X` kopyalar ve keser.

### Ayarlar'ı kapatma

`✕`, `Esc` ve `q` aynı işi yapar. Ayarlar her değişikliği hemen uygular, bu yüzden çoğu zaman
yalnızca kapanır. **Ayarlar kapatılsın mı?** diye yalnızca bir şey kaybolacaksa sorar:

- **İş akışı** sayfasında kaydedilmemiş değişiklikler varsa;
- commit mesajı şablonunda kaydedilmemiş değişiklikler varsa;
- bir profilin yazar e-postası yoksa (yaptığı commit'ler size atfedilemez).

Kutu **Oraya git** (o sayfayı açar), **At ve kapat** ya da **Yine de kapat** ve **İptal**
seçeneklerini sunar. Ayarlar içindeki küçük düzenleyicilerin (bir YZ talimatı metni, bir iş akışı
hook'u, kısmi checkout yolları) kendi `✕`'leri vardır ve **Değişiklikler atılsın mı?** diye
yalnızca metinleri düzenlenmişse sorarlar.

### Ayarlar'ın üzerindeki açılır pencereler

Başlık satırı, ana pencerenin başlığı gibi uygulamanın ahtapotunu ve adını gösterir.
Hakkında'yı açmak için ahtapota tıklayın.

Birkaç pencere genelindeki açılır pencere Ayarlar'ın üzerinde açılır ve onu olduğu gibi geri
verir (sayfa, kaydırma konumu ve kaydedilmemiş İş Akışı ya da şablon değişiklikleri yerinde kalır):

- komut paleti (`Ctrl+P`),
- tema seçici (`Ctrl+N t`, **Uygulama: Temayı Değiştir...**); her temayı Ayarlar'ın kendisinde
  önizler, `Esc` önceki temayı geri koyar,
- dil ve grafik rengi seçicileri (`Ctrl+N` ardından ana penceredekiyle aynı harfler),
- çalışma alanı araması (`Ctrl+Alt+F`),
- Hakkında (`Ctrl+N a`, ahtapot ya da **Yardım: GitPoulpi Hakkında**) ve yardım katmanı (`?`).

Her birini `Esc` ile, kendi `✕`'siyle ya da dışına tıklayarak kapatın. Ayarlar'ın kapattığı
şeye etki eden komutlar (bir sekme, bir dal, bir diff...) palette listelenir ama "önce
Ayarlar'ı kapatın" notuyla soluk görünür ve bunlar için `Ctrl+N` akorları aynı satırı
Ayarlar'ın alt kenarlığında gösterir: Ayarlar'ı kapatın, sonra çalıştırın.

`Ctrl+N` sayfanın her denetimi üzerinde bir rozet gösterir: arama kutusu, sayfa listesi (geçerli sayfada tek rozet), alt sekmeler, liste ve düğmeleri, ekrandaki her alan ve kapatma çarpısı. Bir rozetin karakterine basın; o denetime tıklamakla aynı şey olur: metin alanı düzenlemeye açılır, onay kutusu değişir, açılır liste açılır, düğme çalışır. `Esc` iptal eder. Ekran dışına kaydırılmış alanın rozeti olmaz. Bir alana yazarken ya da açılır liste, düzenleyici veya arama açıkken `Ctrl+N` onlara aittir. Bkz. [Rozetler](keybindings.md#rozetler).

### Arama kutusu

Sayfa listesinin üstündeki kutu, her sayfadaki her seçeneğin adında ve açıklamasında arar.
Üzerine tıklayın ya da `/` veya `Ctrl+F` tuşuna basın ve yazın. Eşleşmelerin bir listesi
açılır. Her eşleşme seçeneği ve sayfasını gösterir. `Up` / `Down` seçer, `Enter` seçeneğe
gider, `Esc` listeyi kapatır.

![Ayarlar'da "diff" araması: YZ ile Diff ve düzenleyici sayfalarından eşleşen seçenekler](../images/settings-search.png)

## Komut paletinden ayarlar

Ayarların çoğunu bu ekranı açmadan değiştirebilirsiniz: komut paletine (`Ctrl+P`) `Ayarlar:` yazın. Satırlar sayfaların kendi alanlarından üretilir, bu yüzden yeni bir ayar her iki yerde de görünür.

- Bir onay kutusu **Ayarlar: ... aç/kapat** olarak görünür. Hemen değişir ve satır `[x]` veya `[ ]` gösterir.
- Bir açılır liste **Ayarlar: ...** olarak görünür. `Enter` paleti değerlerine daraltır, geçerli olan `geçerli` ile işaretlenir; bir değerde `Enter` onu seçer.
- Bir sayı veya metin, geçerli değerle dolu bir giriş alanı açar. Sayfanın reddedeceği bir değer (sayı değil, aralık dışı) aynı şekilde reddedilir ve alan açık kalır.
- Kendi sayfasına ihtiyaç duyan bir ayar (bir liste, seçenekleri çalışırken bulunan bir birleşik kutu) Ayarlar'ı o satırda açar.

![Ayarlar: ile filtrelenmiş komut paleti: her ayar için bir satır ve sağda geçerli değeri](../images/palette-settings.png)

Bir değer `config.toml` dosyasına kaydedilir ve sayfanın yaptığı gibi uygulanır. Ayar anahtarının İngilizce sözcükleri (`remember tabs`) satırı her dilde bulur.

## Sayfalar

Sayfa listesinde iki grup vardır:

- **Ayarlar**: genel ayarlar.
- **Depoya Özel Ayarlar**: etkin sekmenin deposuna ait ayarlar.
  Yalnızca bir depo açıkken görünürler.

### Genel

![Genel sayfasındaki Dil açılır listesi: Otomatik ve on yedi dil, bir arama alanıyla](../images/settings-language.png)

| Seçenek | Ne yapar |
|---|---|
| **Dil** | Arayüz dili: **Otomatik** (sistem diliniz, yoksa İngilizce), English, Français, Español, Português (Brasil), Português (Portugal), Italiano, Deutsch, 简体中文, 日本語, 한국어, Русский, Українська, Polski, Čeština, Nederlands, Svenska, Türkçe. Yeniden başlatmadan hemen uygulanır. Log'un ay adları ve tarihleri bunu izler. `GITPOULPI_LANG` ortam değişkeni (örneğin `GITPOULPI_LANG=fr`) bu seçimi geçersiz kılar. CLI ve MCP çıktısı İngilizce kalır. |
| **Git terimlerini çevir** | Varsayılan olarak kapalıdır: commit, push, pull, stash, tag ve diğer Git terimleri her dilde İngilizce kalır. Açık: dilin kendi Git kelime dağarcığı kullanılır (Fransızca: *Tirer*, *Pousser*, *Remiser*, *Étiquette*). Hemen uygulanır. |
| **İngilizce Git sözcükleri** | Yalnızca Japonca, Korece ve Çince için: *stage*, *unstage*, *staged* sözcüklerini transkripsiyon yerine İngilizce gösterir. **Git terimlerini çevir** açıkken etkisizdir. |
| **Sekmeleri hatırla** | Başlangıçta son oturumun sekmelerini yeniden açar. Kapalıyken, bir depo içinde yapılan başlatma yalnızca o depoyu açar; depo dışında yapılan başlatma ise depo sekmesi olmadan Ana sekmede başlar. Yeniden başlatma gerekir. |
| **Radar'ı etkinleştir** | Varsayılan olarak açık (`general.radar_enabled`). Kapalıyken Radar sekmesi, palet komutları ve `Alt+R` gizlenir; arka planda onun için hiçbir şey getirilmez. Hemen uygulanır. Bkz. [Radar](radar.md#sekmeyi-kapatmak-veya-radarı-kapatmak). Radar'sız bir derlemede görünmez. |
| **Depo içinde başlangıç sekmesi** | Başlatma, içinde başladığı klasör bir deponun içindeyken hangi sekmede açılır (`general.startup_tab_in_repo`): **Geçerli depo** (varsayılan, o deponun sekmesi, gerekirse açılır), **Son sekme** (son çıkışta etkin olan sekme), **Ana**, **Radar**. `--repo` ile verilen depo her zaman kendi sekmesini açar. **Son sekme**, **Sekmeleri hatırla** kapalıysa ya da hatırlanan bir şey yoksa Ana olur; **Radar**, Radar kapalıysa veya derlemede yoksa Ana olur. Başlangıçta bir kez okunur; yeniden başlatma gerekir. |
| **Depo dışında başlangıç sekmesi** | Başlatma, içinde başladığı klasör bir depo değilken (örneğin kullanıcı klasörü) hangi sekmede açılır (`general.startup_tab_outside_repo`): **Ana** (varsayılan), **Son sekme**, **Radar**. **Son sekme**, **Sekmeleri hatırla** kapalıysa ya da hatırlanan bir şey yoksa Ana olur; **Radar**, Radar kapalıysa veya derlemede yoksa Ana olur. Başlangıçta bir kez okunur; yeniden başlatma gerekir. |
| **Başlatıcıdan başlangıç sekmesi** | GitPoulpi'nin oluşturduğu masaüstü veya Başlat menüsü kısayolundan (`--launcher` iletir) başlatıldığında, klasör ne olursa olsun hangi sekmede açılır (`general.startup_tab_launcher`): **Son sekme** (varsayılan), **Ana**, **Radar**. Yukarıdaki iki ayara göre önceliklidir. **Son sekme**, **Sekmeleri hatırla** kapalıysa ya da hatırlanan bir şey yoksa Ana olur; **Radar**, Radar kapalıysa veya derlemede yoksa Ana olur. Eski bir sürümün oluşturduğu kısayollar bu seçeneği sonraki başlatmada alır. Başlangıçta bir kez okunur; yeniden başlatma gerekir. Windows'ta görev çubuğuna sabitlediğiniz bir kopya da bir sonraki başlatmada yeniden yazılır; çevresinde kabuk olmadan yapılan bir başlatma (çift tıklama, doğrudan sabitlenmiş exe) başlatıcıdan başlatma sayılır. |
| **Worktree sekmelerini birlikte grupla** | Varsayılan olarak açık (`general.group_worktree_tabs`). Bir deponun bağlı worktree'lerinin sekmeleri deponun sekmesinin hemen arkasında durur; sürüklenip uzaklaştırılan worktree sekmesi oraya geri döner. Kapalıyken sekmeler bıraktığınız yerde kalır. Açmak, açık sekmeleri yeniden gruplar. |
| **Varsayılan klon klasörü** | Klonla iletişim kutusunun başladığı yer. Boşsa: Windows'ta `%USERPROFILE%\source\repos`, başka yerlerde `~/src`. |
| **Klonla iletişim kutusunun başlangıcı** | Klonla iletişim kutusunun hangi kaynakta açıldığı (`repos.clone_start_source`): **Son kullanılan** (varsayılan, en son kapatıldığında, klonlayarak ya da iptal ederek, gösterdiği kaynak), **URL ile klonla** ya da bağlı entegrasyonlarınızdan biri. Artık kullanılamayan bir kaynak URL ile klonla'ya döner. Radar'dan başlatılan bir klon kendi kaynağını korur. |
| **Varsayılan gözatma klasörü** | Gözat'ın açıldığı yer. Boşsa: en son kullandığınız klasör, yoksa ana klasörünüz. |
| **Depo verilerinin konumu** | GitPoulpi'nin depoya özel verilerini sakladığı yer (`session.per_repo_state_dir`): **.git içinde** (varsayılan: çalışma ağacınızda hiçbir şey belirmez, tüm worktree'ler aynı veriyi paylaşır), **Çalışma ağacında** (`.gitpoulpi` klasörü; bir ekip `workflow.toml` dosyasını commit'lemek için gerekir) veya **Kullanıcı verileri klasörümde** (deponun dışında). Eski bir sürümün bıraktığı `.gitpoulpi` klasörü, ilk gerektiğinde yeni yere taşınır ve boşalınca silinir; git'in izlediği bir dosya (commit'lenmiş bir `workflow.toml`) asla taşınmaz veya silinmez ve okunabilir kalır. |
| **Varsayılan Dal Adı** | Init ile oluşturduğunuz bir deponun ilk dalı. Boşsa: Git karar verir. Mevcut hiçbir dalı yeniden adlandırmaz. |
| **Günlük düzeyi**, **Günlük dosyalarını sakla**, **Bir günlük dosyasının en büyük boyutu** | Etkinlik Günlüğü'nün arkasındaki uygulama günlüğü: kaydedilen en ayrıntılı düzey (`log.level`), dosyaların ne kadar saklanacağı (tek satırda bir tam sayı ve bir birim: gün, hafta, ay veya yıl; sayıya tıklayıp yazın, Up ve Down onu değiştirir, Tab birime geçer ve Enter listesini açar; en çok 10 yıl, varsayılan 1 hafta, `log.retention_days` ve `log.retention_unit`) ve bir günün dosyasının döndürüldüğü boyut (bir sayı ve MiB ya da GiB, 1 MiB ile 1 GiB arası, `log.max_file_mb` ve `log.max_file_unit`). |
| **İşlem sürelerini göster**, **Genişletilmiş günlükleme** | Etkinlik Günlüğü'nün Git işlemleri sekmesi: her işlemin süresi (`log.timings`, varsayılan olarak açık) ve ayrıca arka plan işleri, yenileme ve otomatik fetch (`log.extended`, varsayılan olarak kapalı). Bkz. [Bir şey başarısız olduğunda](troubleshooting.md#etkinlik-günlüğü). |
| **Geri alma açık**, **Geri alma geçmişi derinliği**, **GitPoulpi dışında yapılan değişiklikleri kaydet** | **Geri alma geçmişi** grubu. Geri Al'ı açar ya da kapatır ve kaç işlem tuttuğunu ayarlar (varsayılan 100, `0` sınırsız demektir). Geri Al'ı kapatmak, bir onaydan sonra geri alma geçmişini siler. Sonuncusu, bir kabuğun veya bir ajanın dallar, tag'ler, stash'ler ve HEAD üzerinde yaptıklarını da kaydeder. Bkz. [Geri al ve yinele](undo-redo.md). |

**Uygulama: Dili Değiştir...** palet komutu (`Ctrl+N l`) aynı dil listesini açar. Orada
vurgulanan dil canlı önizlenir; `Enter` onu korur, `Esc` öncekini geri getirir.

### Başlatıcı

**Başlatıcı** sayfası (`gitpoulpi ui show settings launcher`), GitPoulpi'yi işletim sisteminin
başlatıcısından, seçtiğiniz terminalde başlatan bir kısayol oluşturur.

| Satır | Ne yapar |
|---|---|
| **Kısayollar için terminal** | `ui.launch_terminal`: **Otomatik**, **Sistem varsayılanı** veya bu bilgisayarda bulunan terminallerden biri (**Şurada aç** ile aynı algılama). Otomatik: Windows'ta Windows Terminal (yoksa WezTerm, Alacritty, düz konsol), Linux'ta `xdg-terminal-exec` (Omarchy; yoksa bulunan ilk terminal), macOS'ta iTerm2, Ghostty, WezTerm veya kitty kurulu değilse Terminal. |
| **Çift tıklamada bu terminalde yeniden aç** | Yalnızca Windows (`ui.relaunch_in_terminal`). Exe'ye çift tıklamak, resimleri kötü çizen varsayılan konsolu açar: GitPoulpi kendini seçilen terminalde yeniden açar ve ilk pencere kapanır. Bir terminalden başlatıldıysa yapılmaz. |
| **Tıklamada açık pencereyi göster** | `ui.reuse_window` (varsayılan olarak açık). Bir GitPoulpi kısayoluna tıklamak (`gitpoulpi launch`), yenisini başlatmak yerine zaten açık olan GitPoulpi'nin penceresini gösterir. Yeni bir terminal yalnızca hiçbiri açık değilse, ayar kapalıysa veya pencere öne gelemiyorsa başlatılır. Pencere Windows'ta, WezTerm'de ve Hyprland'de öne gelir; diğer Linux masaüstlerinde tıklama yine yeni bir pencere başlatır. Bilerek başka bir pencere açmak için tıklarken **Shift** tuşunu basılı tutun (Windows'ta, görev çubuğuna sabitlenmiş simgede de) ya da Linux başlatıcı girdisinin **New Window** eylemini kullanın (simgeye sağ tıklayın); bir betikten `gitpoulpi launch --new-window` aynı işi yapar. macOS'ta kullanılmaz (Dock bunu zaten yapar). |
| **Komut takma adı** | `ui.command_alias`: **Yok** ve `gp` içeren bir açılır liste; ya da herhangi bir ad yazın (harf, rakam, `-`, `_`, `.`; boşluk ve yol ayracı olmaz). Satır içi **Oluştur** düğmesi o adı GitPoulpi'yi başlatan bir komut yapar (takma ad oluşunca **Kaldır**'a dönüşür ve GitPoulpi'nin oluşturduğu her takma adı kaldırır). Ad, PATH'inizde zaten başka bir şeyi çalıştırıyorsa satırın altında bir uyarı görünür; Windows'ta PowerShell'in aynı adlı yerleşik bir takma adı varsa da (orada `gp`, `Get-ItemProperty` komutudur ve her dosyanın önüne geçer) uyarı çıkar. Bir kabuk takma adı veya işlevi buradan görülemez: oh-my-zsh `gp`'yi kendisi tanımlar ve bizimkinin önüne geçer. |
| **Yeni sekme takma adı** | `ui.command_alias_add`: zaten çalışan GitPoulpi'de geçerli klasörü yeni sekme olarak açan (`gitpoulpi --add`), yenisini başlatmayan ikinci bir komut, örneğin `gpa`; hiçbiri çalışmıyorsa GitPoulpi'yi her zamanki gibi başlatır, birden çoğu varsa hangisine katılacağını sorar. **Yok**, `gpa` ve takma ad adının sonuna `a` eklenmiş hâlini içeren bir açılır liste; ya da herhangi bir ad yazın (kurallar yukarıdakiyle aynı). Tek **Oluştur** düğmesi iki komutu da oluşturur, **Kaldır** GitPoulpi'nin oluşturduğu her komutu kaldırır; her birinin kendi durum çubuğu vardır. Linux ve macOS'ta iki satırlık bir betik, Windows'ta bir `.cmd` dosyasıdır. Bkz. [CLI başvurusu](cli-reference.md#açık-pencereye-klasör-eklemek). |
| **GitPoulpi'yi PATH'e ekle** | GitPoulpi'nin çalıştığı klasörü ve takma ad klasörünü, yeni terminaller için PATH'inize ekler. Satırın düğmesi **Ekle** ya da **Kaldır** der; **Kaldır**, eklenenin tam olarak kendisini çıkarır. PATH'te zaten bulunan klasörlere dokunulmaz (satır bu durumda **zaten PATH'te** der). |
| **Linux: Uygulama başlatıcı** | `~/.local/share/applications/gitpoulpi.desktop` dosyasını (`XDG_DATA_HOME`'a uyar) ve simgeyi `~/.local/share/icons/hicolor/256x256/apps` içine yazar. Omarchy'de girdi, `omarchy-tui-install`'ın oluşturduğuyla aynıdır (`xdg-terminal-exec --app-id=TUI.tile -e <exe>`); diğer terminaller için uygun komut (`kitty`, `wezterm start --`, `alacritty -e`, `ghostty -e`, `foot`, ...). |
| **Linux: Masaüstü** | Aynı girdi, varsa Masaüstü klasörünüzde. |
| **Windows: Başlat menüsü** | `%APPDATA%\Microsoft\Windows\Start Menu\Programs` içinde `GitPoulpi.lnk`. Görev çubuğuna sabitlenmiş kopya simge ve renk değişikliklerini kendiliğinden izler; simgesi yine eski kalırsa sabitlemeyi kaldırıp yeniden sabitleyin (Başlat menüsü öğesine sağ tıklayın, sonra **Görev çubuğuna sabitle**). |
| **Windows: Masaüstü** | Masaüstünüzde `GitPoulpi.lnk`. |
| **macOS: Uygulamalar klasörü** | `~/Applications/GitPoulpi.app`, GitPoulpi'yi çalıştıran terminali açan küçük bir paket. |

Her satır durumunu gösterir: **oluşturulmadı**, **oluşturuldu**, **güncel değil**
(exe taşındı veya başka bir terminal seçildi; GitPoulpi onu bir saniye içinde kendisi
yeniden yazar, bu yüzden nadiren görürsünüz) veya **GitPoulpi'ye ait olmayan bir dosya
var** (bu adda GitPoulpi'nin yapmadığı bir dosya var; ona asla dokunulmaz). **Kaldır**
yalnızca GitPoulpi'nin oluşturduğu dosyaları siler (bir `X-GitPoulpi=1` işareti
taşırlar). Simge, logo renginizdeki ahtapottur (Hakkında > renk); Linux'ta her rengin
kendi simge adı vardır, bu yüzden simgeleri önbellekte tutan bir başlatıcı yeni rengi
gösterir. Bir kısayol oluşturulduktan sonra kendiliğinden güncel kalır: rengi
değiştirdiğinizde, terminal veya exe taşındığında GitPoulpi kendi oluşturduğu
kısayolları yeniden yazar (oluşturmadığı birini asla, ve hiç yeni kısayol oluşturmadan),
başlarken ve çalışırken. Güncelle düğmesi yoktur; yalnızca **Oluştur** (henüz var
olmayan bir kısayol için) ve **Kaldır** vardır.

**Görev çubuğu ve dock gruplaması.** GitPoulpi bir terminal penceresinde çalışır; bu yüzden görev çubuğu onu normalde terminalin simgesi altında, sabitlenmiş GitPoulpi simgesinin altında “çalışıyor” noktası olmadan gösterir. Windows: kısayollar `GitPoulpi.GitPoulpi` AppUserModelID'sini taşır ve onlardan başlatılan GitPoulpi, terminal penceresine aynı kimliği ve kendi simgesini verir (pencereyi, ona bir an benzersiz bir başlık vererek bulur, sonra başlığı geri yükler); böylece pencere sabitlenmiş GitPoulpi simgesinin altında yer alır. Birden çok sekmeli bir terminal penceresi bir bütün olarak taşınır: GitPoulpi içinde çalıştığı sürece tüm sekmeleri GitPoulpi simgesi altında gruplanır. Daha önce yapılmış kısayollar ve görev çubuğu sabitlemeleri aynı eşitlemeyle yeniden yazılır; bir sabitleme hâlâ terminalin simgesini gösteriyorsa sabitlemeyi kaldırıp yeniden sabitleyin. Linux: kitty, foot, Alacritty, WezTerm ve Ghostty `gitpoulpi` pencere sınıfıyla başlatılır ve `.desktop` girdisi bunu belirtir (`StartupWMClass`); böylece dock pencereyi başlatıcının simgesi altında gruplar; diğer terminaller kendi simgelerini korur. macOS: yapılacak bir şey yok, Dock uygulamaya göre gruplar.

![Ayarlar > Başlatıcı: kısayollar için terminal, başlatıcı girdileri, komut ve yeni sekme takma adları, takma ad durumu ve PATH denetimi](../images/settings-launcher.png)

#### Komut takma adı ve PATH, sisteme göre

| Sistem | Takma ad | PATH |
|---|---|---|
| Linux, macOS | `~/.local/bin/<ad>` yolunda `gitpoulpi-tui` programına bir sembolik bağlantı. Bir bağlantı işaret taşıyamaz; bu yüzden hedefi `gitpoulpi-tui` adlı bir dosyaysa bize ait sayılır. Sıradan bir dosya ya da başka bir şeye giden bir bağlantı **bize ait değildir** ve asla değiştirilmez veya kaldırılmaz. | `~/.bashrc` ve `~/.zshrc` içinde (hangisi varsa; ikisi de yoksa macOS'ta `~/.zshrc`, başka yerlerde `~/.bashrc`) `# >>> GitPoulpi PATH (X-GitPoulpi=1) >>>` ile `# <<< GitPoulpi PATH <<<` arasındaki bir blok; fish yapılandırılmışsa ayrıca `~/.config/fish/conf.d/gitpoulpi-path.fish`. Kaldır yalnızca bloğu (veya fish dosyasını) siler. |
| Windows | `%LOCALAPPDATA%\Programs\GitPoulpi\bin\<ad>.cmd`, `REM X-GitPoulpi=1` içeren kısa bir ara betik (ayrıca adres çubuğundan başlatmanın tanınmasını sağlayan `GITPOULPI_CMDCMDLINE` değişkenini dışa aktarır). | `HKCU\Environment` içindeki kullanıcı PATH'i; PowerShell ile yazılır, sonra yeni programların görmesi için bir `WM_SETTINGCHANGE` yayını yapılır. `GitPoulpiPathAdded` değeri eklenenleri kaydeder ve Kaldır yalnızca onu çıkarır. |

Yeni terminaller PATH değişikliğini görür, zaten açık olanlar görmez. İki eylem de geri alma
geçmişine yazılmaz: bunlar için Geri Al yoktur, geri dönüş yolu **Kaldır**'dır.

Palette aynı eylemler vardır (**Başlatıcı: kısayol oluştur**, **Başlatıcı: masaüstü kısayolu
oluştur**, **Başlatıcı: kısayolları kaldır**, **Başlatıcı: komut takma adı oluştur**,
**Başlatıcı: komut takma adını kaldır**, **Başlatıcı: PATH'e ekle**, **Başlatıcı: PATH'ten kaldır**); bir betikten `gitpoulpi desktop shortcut
status|install|remove [--where launcher|desktop|startmenu]` aynısını yapar.

#### Dosya Gezgini adres çubuğuna yazmak (Windows)

Bir Dosya Gezgini penceresinin adres çubuğuna takma adı (`gp`) veya `gitpoulpi` yazıp Enter'a basın: GitPoulpi, **Kısayollar için terminal** ayarındaki terminalde, pencerenin gösterdiği klasörde açılır ve **Son sekme**'yi açmaz. **Tıklamada açık pencereyi göster** açıkken ve bir GitPoulpi zaten açıksa, klasör bunun yerine o pencerenin yeni bir sekmesi olur.

- **`gp` (komut takma adı).** Takma ad oluşturulunca `gp.exe` de `HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths` altına, `gitpoulpi-launch.exe` dosyasını gösterecek şekilde kaydedilir; Kaldır bu anahtarı siler (yalnızca bize aitse). Takma adın klasörü PATH'inizdeyse (**GitPoulpi'yi PATH'e ekle**), Windows önce `.cmd` ara betiğini çalıştırır ve GitPoulpi görünmeden önce bir konsol penceresi kısa süre yanıp sönebilir. Eski bir sürümün yazdığı ara betik, Ayarlar takma adı okuduğunda otomatik olarak yeniden yazılır.
- **`gitpoulpi`.** **Dosya yöneticisi menüsü** kuruluyken çalışır: Kur, `gitpoulpi.exe` dosyasını da App Paths altına kaydeder. Menü yoksa `gitpoulpi` yalnızca GitPoulpi'nin klasörü PATH'teyse bulunur; o zaman konsol yukarıdaki gibi yanıp sönebilir.

Dosya Gezgini, Windows'ta oturum açtığınızdaki PATH'i korur: PATH'e sonradan eklenen bir klasör orada ancak oturumu kapatıp yeniden açtıktan sonra görülür. O zamana kadar yalnızca App Paths girdileri bulunur ve Windows'un bulamadığı bir ad tarayıcınızda web adresi olarak açılır.

Linux ve macOS'ta karşılığı yoktur: dosya yöneticilerinin konum çubukları program çalıştırmaz. **Dosya yöneticisi menüsü** girdisini veya bir terminaldeki komut takma adını kullanın.

#### Dosya yöneticisi menüsü (GitPoulpi'de aç)

Kısayolların altındaki **Dosya yöneticisi menüsü** satırları, klasörlerin sağ tık menüsüne **GitPoulpi'de aç** girdisini ekler. Her dosya yöneticisi için bir durum satırı vardır (Linux'ta burada bulunan her dosya yöneticisi için ve girdiyi hâlâ tutan her biri için; bulunanları bir **Algılanan** satırı listeler), ardından **Kur** ve **Kaldır** düğmeleri gelir. **Kur**, bulunan her dosya yöneticisi için girdiyi yazar; **Kaldır** önce sorar, sonra o dosya yöneticisi hâlâ kurulu olsun olmasın Kur'un yazdığının tamamını kaldırır.

Girdi `gitpoulpi launch --folder <klasör>` komutunu çalıştırır (Windows'ta konsolsuz `gitpoulpi-launch.exe --folder`); tıklandığında ne yapılacağına bu komut karar verir: **Tıklamada açık pencereyi göster** açıkken ve bir GitPoulpi penceresi açıkken, bir deponun içindeki klasör en son kullanılan pencerenin yeni bir sekmesi olur ve pencere öne gelir; aksi halde **Kısayollar için terminal** ayarındaki terminal GitPoulpi'yi o klasörde başlatır. Bu yüzden terminali değiştirmek girdiyi hiçbir zaman güncel dışı bırakmaz; taşınmış bir exe veya yeni bir logo rengi bırakır ve GitPoulpi kendi girdilerini kısayollar gibi otomatik olarak yeniden yazar (hiçbir zaman yenisini oluşturmaz). Girdi, GitPoulpi'nin yanında başlatıcı yardımcısını (`gitpoulpi` veya Windows'ta `gitpoulpi-launch.exe`) gerektirir; yoksa satır bunu söyler ve düğmeler kapalı kalır.

| Sistem | Kur ne yazar |
|---|---|
| Windows | Kullanıcı başına anahtarlar, yönetici hakkı gerekmez: `HKCU\Software\Classes\Directory\shell\GitPoulpi`, `Directory\Background\shell\GitPoulpi` ve `Drive\shell\GitPoulpi`; her birinde bir `command` alt anahtarı vardır, böylece girdi bir klasörde, açık bir klasörün arka planında ve bir sürücüde görünür. Her anahtar bir `X-GitPoulpi` değeri taşır; bu değeri olmayan bir anahtar **bize ait değildir** ve ona dokunulmaz. Ayrıca `gitpoulpi.exe` dosyasını, `gitpoulpi-launch.exe` dosyasını gösterecek şekilde `HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths` altına kaydeder. Windows 11'de girdi yalnızca klasik menüdedir: **Daha fazla seçenek göster**, Shift+F10 veya Shift+sağ tık (yeni üst düzey menü yalnızca paketlenmiş uygulamaları listeler). Kaldır kendi anahtarlarımızı siler, ardından Kur'un oluşturduğu üst anahtarları, yalnızca boş kaldılarsa. |
| Linux | Yalnızca PATH'te bulunan dosya yöneticileri için. GNOME Dosyalar ve Caja: `~/.local/share/nautilus/scripts` veya `~/.config/caja/scripts` içinde girdinin adını taşıyan bir betik (**Betikler** alt menüsünde). Dolphin: `~/.local/share/kio/servicemenus/gitpoulpi-open.desktop`, çalıştırılabilir (KDE Frameworks 6, çalıştırılabilir olmayan kullanıcı servis menüsünü yok sayar; 5.85 öncesi KDE Frameworks 5 desteklenmez). Nemo: `~/.local/share/nemo/actions` içinde iki eylem (seçili bir klasör, arka plan). Thunar: `~/.config/Thunar/uca.xml` içine birleştirilen tek bir eylem; kendi eylemleriniz kalır ve Kaldır dosyayı bayt bayt geri verir (Kur oluşturduysa siler). PCManFM ve PCManFM-Qt: `~/.local/share/file-manager/actions/gitpoulpi-open.desktop`. Simge `~/.local/share/icons/hicolor/256x256/apps/gitpoulpi-folder.png` yoluna gider ve son girdiyle birlikte kalkar. `XDG_DATA_HOME` ve `XDG_CONFIG_HOME` dikkate alınır. Girdi seçili bir klasörde ve dosya yöneticisinin sunduğu yerlerde klasör arka planında görünür. |
| macOS | Bir Hızlı İşlem, `~/Library/Services/Open in GitPoulpi.workflow`: Finder'da bir klasöre sağ tıklayın, sonra **Hızlı İşlemler**. Yalnızca seçili klasörler (Finder'da klasör arka planı için menü yoktur). İlk tıklayışınızda macOS, iş akışı hizmetinin terminalinizi denetleyip denetleyemeyeceğini sorar (bir Otomasyon istemi, örneğin "WorkflowServiceRunner.xpc ... denetlemek için erişim istiyor"): izin verin. Girdi yoksa Sistem Ayarları > Klavye > Klavye Kısayolları > Hizmetler altında açın. |

GitPoulpi'nin yazdığı her dosya ve anahtar `X-GitPoulpi=1` işaretini taşır (bir betikte `# X-GitPoulpi=1` satırı) ve Kaldır yalnızca işaretli olanları siler. Bu eylemler bir depoyu değil sisteminizi değiştirir: Geri Al bunları kapsamaz, geri dönüş yolu **Kaldır**'dır. Linux ve macOS'ta dosya yöneticilerinin konum çubukları program çalıştırmaz, orada yazılacak bir şey yoktur; bir terminalde komut takma adı vardır.

Palette **Başlatıcı: GitPoulpi'de Aç Menüsünü Kur** ve **Başlatıcı: GitPoulpi'de Aç Menüsünü Kaldır** vardır; bir betikten `gitpoulpi desktop folder-menu status|install|remove` aynısını yapar.

### Eşitleme

GitPoulpi'nin bir depoyu uzak depolarıyla nasıl uyumlu tuttuğu ve bu sırada yerel
değişikliklerinizle ne yaptığı.

![Eşitleme sayfası: otomatik fetch aralığı tek satırda bir sayı ve bir birim (dakika) olarak, altında diğer seçenekler](../images/settings-quantity.png)

| Seçenek | Ne yapar |
|---|---|
| **Otomatik Fetch Aralığı** | Her uzak depoyu bu aralıkla arka planda fetch eder; aralık tek satırda bir sayı ve bir birimdir (dakika veya saat). Varsayılan 5 dakika, en çok 60 dakika (1 saat), `0` kapatır. `general.auto_fetch_interval_minutes` dakika olarak kalır; `general.auto_fetch_interval_unit` yalnızca nasıl gösterildiğini kaydeder. Yeniden başlatma gerekir. |
| **Sekme açılınca fetch et** | Bir depo sekmesi açıldığında arka planda Fetch All çalıştırır. Varsayılan olarak açıktır. Son 60 saniyede fetch edilen bir depo yeniden fetch edilmez. |
| **Otomatik Prune** | Fetch'ler ayrıca uzak depoda artık bulunmayan uzak izleme dallarını kaldırır. Varsayılan olarak açıktır; bu yüzden sunucuda (örneğin GitLab'da) silinen bir dal bir sonraki fetch'ten sonra REMOTE bölümünden kaybolur. Geri alınamaz: silinen dal uzak depoda yeniden görünürse bir sonraki fetch'te geri gelir. |
| **Tag'ler otomatik temizleme** | Fetch'ler ayrıca uzak depoda silinmiş yerel tag'leri de siler. Yalnızca uzak depoda bulunmuş tag'ler etkilenir: hiç göndermediğiniz (push) bir tag korunur. Git'teki gibi varsayılan olarak kapalıdır. Git 2.41 veya daha yenisi gerekir. CLI: `gitpoulpi fetch --prune-tags`; yapılandırma anahtarı `general.auto_prune_tags`. Bkz. [Dallar, tag'ler ve Refs kenar çubuğu](log-and-history.md#dallar-tagler-ve-refs-kenar-çubuğu). |
| **Alt modülleri güncel tut** | Bir checkout ya da pull sonrasında alt modülleri deponun kaydettiği commit'lere günceller. Bkz. [Alt modüller](submodules.md). |
| **Merge sonrası ".orig" dosyalarını sil** | Bir merge tamamlandıktan sonra `*.orig` yedek dosyalarını siler. Varsayılan olarak kapalıdır. |
| **Checkout, Pull, Merge veya Rebase öncesi otomatik Stash** | Yerel değişiklikler bu işlemlerden birini engellediğinde ne yapılacağı: **sor** (bir iletişim kutusu değişiklikleri stash'lemeyi, işlemi çalıştırmayı ve stash'i geri almayı önerir), **her zaman** (sormadan yapar) ya da **asla** (reddeder). |
| **Otomatik stash'e izlenmeyen dosyaları da kat** | Otomatik stash izlenmeyen dosyaları da alır. |

Bu sayfada ayrıca **Güncellemeler** altında şunlar vardır:

| Seçenek | Ne yapar |
|---|---|
| **Güncellemeleri denetle** | Bir açılır liste (`updates.mode`): **Bildir** (varsayılan) günde bir kez arka planda denetler ve başlık çubuğunda `vX.Y.Z kullanılabilir` gösterir; **Otomatik**, GitPoulpi bir sürüm arşivinden kurulduysa güncellemeyi arka planda da indirir (ne zaman yeniden başlatacağınıza yine siz karar verirsiniz); **Kapalı** hiç istek yapmaz. Hemen uygulanır. Bkz. [Güncellemeler](updates.md). |
| **Güncelleme kanalı** | Bir açılır liste (`updates.channel`): **Kararlı** (varsayılan) veya **Ön sürüm** (sürüm adaylarını da kapsar). Hemen uygulanır. |
| **Kurulum yöntemi** | Salt okunur. GitPoulpi'nin bu bilgisayara nasıl geldiği (bir sürüm arşivi, bu kullanıcı veya tüm kullanıcılar için kurulum programı, Chocolatey, Scoop, Homebrew, apt, bir MSI paketi, bir Docker imajı, cargo veya bir geliştirme derlemesi). Güncellemelerin nasıl uygulanacağını belirler, bkz. [Güncellemeler](updates.md). |

Yöneticinizin makine genelindeki güncelleme politikasında ayarladığı bir satır **Yöneticiniz tarafından ayarlandı** der ve burada değiştirilemez.

Bu sayfada ayrıca **Geçmişi yeniden yazma** altında harici betiklerin süre sınırları vardır (bkz. [Geçmişi yeniden yazma](history-rewrite.md)):

| Seçenek | Ne yapar |
|---|---|
| **Betik yanıt süresi sınırı** | Harici bir betiğin tek bir commit'e yanıt vermek için sahip olduğu süre; tek satırda bir sayı ve bir birim (milisaniye veya saniye). Varsayılan 5 saniye, 1 milisaniyeden 10 dakikaya kadar (`rewrite.script_timeout_ms`, `rewrite.script_timeout_unit`). Zaman aşımında tüm süreç durdurulur ve yeniden yazma hiçbir şey taşınmadan başarısız olur. |
| **Geçiş başına betik süre sınırı** | Harici bir betiğin tüm geçişinin alabileceği süre; bir sayı ve bir birim (saniye, dakika veya saat). Varsayılan 1 saat, en çok 24 saat (`rewrite.script_total_timeout_s`, `rewrite.script_total_timeout_unit`). |

### Profiller

![Profiller sayfası](../images/settings-profiles.png)

Commit kimliği profilleri: GitPoulpi'den yapılan commit'lerde kullanılan ad ve e-posta.
**Genel** sekmesi **Etkin profil** açılır listesini içerir; **Profiller** sekmesi seçili profilin
alanlarıyla birlikte bir [liste](#listeler)dir (ekle, çoğalt, kaldır, yeniden sırala). Her profilin bir **Profil kimliği**,
bir **Profil adı**, bir **Yazar adı** ve bir **Yazar e-postası** vardır. E-postası olmayan bir
profil listede `⚠` gösterir. Etkin profil yoksa ya da bir profilin adı ya da e-postası
boşsa, deponun kendi `user.name` ve `user.email` değerleri geçerlidir.

İki ya da daha fazla profil varsa araç çubuğunun sağ ucunda bir profil açılır listesi
görünür. Bu liste profili yalnızca etkin depo için ayarlar. Seçtikten sonra bir iletişim
kutusu, kimliği o deponun yapılandırmasına da yazmayı önerir. Palet komutu
**Depo: Profili Değiştir...**'dir.

Bu profiller, Entegrasyonlar sayfasındaki barındırma profilleri ya da YZ sayfasındaki YZ
profilleri değildir.

### SSH ve kimlik bilgileri

![SSH ve kimlik bilgileri sayfası](../images/settings-ssh.png)

SSH sisteminizin `ssh` programını kullanır, bu yüzden ssh-agent'ınız ve ssh
yapılandırmanız çalışmaya devam eder. GitPoulpi anahtar içeriğini asla okumaz.

| Seçenek | Ne yapar |
|---|---|
| **Özel anahtar yolu** | Ayarlıysa fetch, pull, push ve clone git'i `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes` ile çalıştırır, böylece ssh yalnızca o anahtarı sunar. Boşsa: ajan ve ssh yapılandırmanız değişmeden geçerlidir. |
| **Genel anahtar yolu** | Yalnızca özel anahtar yolu boşken kullanılır: ssh, ssh-agent'ın (ya da bir donanım anahtarının) tuttuğu eşleşen anahtarı seçer. |
| **Anahtar türü** | **Anahtar çifti oluştur** düğmesinin ürettiği tür: `ed25519`, `rsa` (4096 bit) ya da `ecdsa` (NIST P-256). |
| **HTTPS kimlik bilgileri** | Fetch, pull, push ve clone'un HTTPS üzerinden nasıl kimlik doğruladığı: **credential_helper** (git'in kendi yardımcısı, varsayılan), **prompt** (yardımcıları yok sayar; git bir terminalde sorar) ya da **none** (kimlik bilgisi yok, yalnızca herkese açık uzak depolar). |
| **Tüm Kullanıcı Adlarını ve Parolaları Unut** | GitPoulpi'nin kimlik bilgisi yardımcınızda sakladığı tüm token'ları kaldırır. Önce onay ister. |
| **Anahtar çifti oluştur** | `~/.ssh/gitpoulpi_<type>` içinde, parolasız, **Anahtar türü** türünde bir anahtar çifti oluşturur ve iki yolunu kaydeder. Hiçbir dosyanın üzerine yazmaz. |

Anahtar yolları bir sonraki aktarıma uygulanır. `~/` öneki ana klasörünüz anlamına gelir.

Başka bir yerde yapılmış bir ayar bu yollara her zaman baskındır. Sıra, en yüksek olan önce:

0. Tek bir deponun kendi [SSH ve kimlik bilgileri](#depoya-özel-ssh-ve-kimlik-bilgileri) sayfasında seçilen anahtar
   (depoya özel grup).
1. GitPoulpi'nin çalıştığı ortamdaki `GIT_SSH_COMMAND` ya da `GIT_SSH`.
2. Deponun git yapılandırmasındaki ya da genel git yapılandırmanızdaki `core.sshCommand`.
   Eski sürümlerde sekme menüsünün *SSH anahtarı...* öğesi bunu tek bir depo için yazardı; SSH ve kimlik bilgileri
   bunu GitPoulpi'ye taşımayı önerir.
3. Bu sayfanın iki anahtar yolu.
4. ssh'ın kendi varsayılanları ve ajan.

Bir anahtar bir barındırma hesabına da ait olabilir; bkz.
[Barındırma hizmetleri ve issue takipçileri](hosting-integration.md#profil-başına-ssh-anahtarı).

### Entegrasyonlar

![Entegrasyonlar sayfası: solda sağlayıcılar ve GitHub için Bağlan düğmesi](../images/settings-integrations.png)

Hesaplarınızı bağlar. **Barındırma** grubunda GitHub, GitHub Enterprise, GitLab, GitLab
Self-Managed, Azure DevOps, Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea, Forgejo ve Gerrit vardır.
**Issue takipçileri** grubunda Jira Cloud, Jira Data Center, Trello, Linear, YouTrack, Redmine ve Local Markdown vardır. Her
sağlayıcının **Bağlan** düğmesi, bir token alanı, token'ın ihtiyaç duyduğu kapsamlar ve
barındırıcının token sayfasına bir bağlantısı vardır. Bu sayfanın kendi kılavuzu var:
[Barındırma hizmetleri ve issue takipçileri](hosting-integration.md).

Sağlayıcıların üstünde iki seçenek bulunur. Onlara ulaşmak için ilk sağlayıcıda `Up`
tuşuna basın.

| Seçenek | Ne yapar |
|---|---|
| **Pull Request ve Issue Yenileme** | Kenar çubuğu listelerinin ve araç çubuğu çipinin ne sıklıkla yenilendiği; bir sayı ve bir birim (dakika veya saat). Varsayılan 5 dakika, en çok 2 saat, `0` zamanlayıcıyı kapatır (`general.hosting_refresh_minutes`, `general.hosting_refresh_unit`). |
| **Şu adla gönder** | Deponun token'ı bir bota aitse (bir GitHub Uygulaması token'ı) barındırıcıya gönderilen mesajların hangi adla yayımlanacağı, örneğin `my-app[bot]`; kendi yorumlarınızı tanımak için de kullanılır. Boş: barındırıcıya sorulur, yoksa gönderilen ilk mesajdan öğrenilir. Bir depo bunu Ayarlar > Depoya özel ayarlar > SSH ve kimlik bilgileri > Sunucu'da geçersiz kılabilir (`general.hosting_post_as`). |
| **İnceleme isteği terimi** | Barındırılan incelemeler için kullanılan sözcük: **Otomatik (barındırıcıya göre)**, **Pull Request** ya da **Merge Request**. |
| **İstek kapatıldığında veya merge edildiğinde pull request çipini gizle** | Varsayılan olarak açıktır (`ui.hide_finished_pr_chip`; GitLab'da "merge request" der). Araç çubuğundaki çip, checkout yapılmış dalın en son isteğini gösterir; o istek kapatılınca ya da merge edilince gizlenir ve yerine, dalın hiç isteği yokmuş gibi, soluk **+ PR** çipi (yeni istek başlat) görünür. Kapalıyken çip, durumu ne olursa olsun her zaman en son isteği gösterir. Hemen uygulanır. |

### YZ

![YZ sayfası: YZ'yi etkinleştir, genel talimatlar ve eylem başına bloklar](../images/settings-ai.png)

**Genel** sekmesinde: YZ'yi açan anahtar, varsayılan profil, her eylemin aldığı talimatlar ve
her eylem için bir profil ve talimatlar. **Profiller (bağlantılar)** sekmesi, bir profilin ortam
değişkenleri için satır içi bir tabloyla YZ profillerinin [liste](#listeler)sidir. Genel sekmesinin *Kod incelemesi* bloğu
ayrıca [inceleme bağlamının](ai.md#i̇nceleme-bağlamı) varsayılanlarını tutar: **Ajanı depoda çalıştır**
(*Profil ayarı*, *Evet*, *Hayır*), **CLAUDE.md'yi ekle**, **AGENTS.md'yi ekle** ve **Ek dosya** (bir yol ya da
bağlantı; boşsa hiçbiri eklenmez, altında bir dosya seçici açan bir **Gözat...** düğmesi vardır; düğmede `Enter` onun klavye karşılığıdır). Bu sayfanın kendi kılavuzu var: [YZ desteği](ai.md).

**Sunucudaki yanıtlar** bloğu, yorumları, yanıtları ve issue açıklamalarını hazırlayanı tutar: profili ve talimatları, **Temel istem** (boşsa yerleşik istem kullanılır) ve bir **Depo erişimi** grubu: **YZ varsayılan olarak depoyu okuyabilsin** (kapalı: her yazma kutusundaki seçenek işaretsiz başlar), **Yanıtlar arasında ajanın oturumunu koru** ve **Depo bağlamını şu kadar sakla (dakika)**. **Yanıt için gönderilecek en büyük konuşma**, **Yanıt için en büyük depo dosyası**, **Yanıt için en büyük depo haritası**, **Yanıt için terim başına en çok arama sonucu** ve **Yanıt için en çok arama terimi** bayt sınırları diğer girdi sınırlarının yanındadır. Bkz. [Sunucuda yazma yardımı](ai.md#sunucuda-yazma-yardımı).

Eylem başına bloklarının altında **Gizlilik** grubu:

| Seçenek | Ne yapar |
|---|---|
| **Göndermeden önce gizli bilgileri sansürle** | Açık (varsayılan): bir istekte bulunan token'lar, özel anahtarlar ve parolalar, istek çıkmadan önce `[REDACTED:<rule>]` ile değiştirilir. Kapalı: kırmızı bir satır gizli bilgilerin olduğu gibi gönderildiğini söyler. |
| **Hassas dosyaları dışarıda bırak** | Açık (varsayılan): `.env` dosyalarının, özel anahtarların ve sertifikaların değişiklikleri (`[privacy] sensitive_patterns`) asla bir isteğin parçası olmaz. Kapalı: kırmızı bir satır, bir isteğin parçasıysa gönderildiklerini söyler. |
| **Göndermeden önce gönderileni göster...** | Eylem başına bir anahtar (commit mesajı, açıklama, stash mesajı, pull request açıklaması, commit besteci, çakışma çözümü, kod incelemesi, commit bölme): isteği, çıkmadan önce **Gönderilenler** iletişim kutusunda gösterir. İletişim kutusundaki *Bir daha gösterme* kutusu birini kapatır. Geçmişi yeniden yazma eylemleri kendi iletişim kutusunu gösterir. |
| **Hepsini yeniden göster** | Bir anahtar kapalıyken görünür; hepsini yeniden açar. |

Bkz. [YZ > Gizlilik](ai.md#gizlilik).

### Ajanlar ve MCP

YZ ajanlarının bu pencerede neler yapabileceği, MCP sunucusunun çalışıp çalışamayacağı ve
HTTP uç noktasının nasıl korunduğu. MCP satırları yalnızca sunucuyu içeren derlemelerde
görünür; bkz. [MCP ve ajanlar](mcp-and-agents.md).

| Seçenek | Ne yapar |
|---|---|
| **MCP sunucusu** | Açık (varsayılan): bir ajan sunucuyu başlatabilir (`gitpoulpi mcp serve`). Kapalı: başlamayı reddeder. `mcp.enabled` olarak kaydedilir. Değişiklik, bir ajan sunucuyu bir sonraki başlatışında uygulanır; zaten çalışan bir sunucu ayarlarını korur. |
| **MCP sunucu portu** | Adres verilmediğinde `--http` komutunun dinlediği port, 1 ile 65535 arası (varsayılan 8808). `mcp.bind` değerinin port kısmına kaydedilir; ana makine kısmı korunur (boşsa `127.0.0.1`). Bir ajan sunucuyu bir sonraki başlatışında uygulanır. |
| **YZ ajanlarının görünümü yönetmesine izin ver** | Ajanların size bu pencerede bir commit, bir dosya ya da bir bildirim göstermesine izin verir (`ui show`). Kapalı: hiçbir ajan pencereye ulaşamaz. |
| **YZ ajanlarının onaylayacağım değişiklikler önermesine izin ver** | Ajanların, sizin onaylayacağınız önceden doldurulmuş Reword, commit, dal ya da tag iletişim kutuları açmasına izin verir. Varsayılan olarak kapalıdır. Bkz. [MCP ve ajanlar](mcp-and-agents.md#i̇zinler). |
| **Ajanlar için pencere aç**, **Başlatma komutu (custom)** | Hiçbir pencere açık değilken bir ajanın bir GitPoulpi penceresi açıp açamayacağı ve hangi terminal ya da çoklayıcıyla açacağı. Varsayılan olarak kapalıdır. |
| **HTTP kimlik doğrulaması** | Açık (varsayılan): MCP HTTP uç noktası yalnızca erişim token'ı taşıyan bir isteğe yanıt verir. Kapalıyken kırmızı bir uyarı gösterir: uç noktaya erişebilen her program MCP ayarlarının izin verdiği her aracı çalıştırabilir ve `--allow-remote` reddedilir. `mcp.http_auth` olarak kaydedilir. |
| **Erişim token'ı** | Token, maskeli. Maskeli değer bir düğmedir: `Enter` (veya bir tıklama) onu alttaki satırda tam olarak gösterir ve yeniden gizler. **Kopyala** onu durum satırında yankılamadan panoya koyar, **Döndür** onu bir onaydan sonra değiştirir (eski token'ı tutan istemciler hemen reddedilir, sunucunun yeniden başlatılması gerekmez), **Oluştur** ilkini yapar. `Left` / `Right` düğmeler arasında gezinir ve `Enter` birine basar. Aynı token `gitpoulpi mcp token` komutudur. |
| **TLS** | **Kapalı** (varsayılan), **Kendinden imzalı** (sunucu bir kez sertifika oluşturur ve parmak izini gösterir) veya **Dosyalar** (sizin sertifikanız ve anahtarınız). TLS'yi sunucu kendisi sonlandırır. `mcp.tls` olarak kaydedilir; sunucunun bir sonraki başlayışında uygulanır. |
| **Sertifika yolu**, **Anahtar yolu** | **Dosyalar** ile kullanılan PEM dosyaları (`mcp.tls_cert`, `mcp.tls_key`). |
| **Sertifika parmak izi** | Salt okunur: uç noktanın sunduğu sertifikanın SHA-256 parmak izi, istemcilerin denetlemesi için. |
| **Tünel veya ters vekil sunucunun arkasında** | Uç noktanın önündeki bir tünelin veya vekil sunucunun trafiği şifrelediğine dair beyanınız; bu, `--allow-remote` komutunun TLS olmadan çalışmasına izin verir. Sunucu bunu denetleyemez, bu yüzden açıkken kırmızı bir uyarı gösterir. `mcp.behind_proxy` olarak kaydedilir. Bkz. [MCP ve ajanlar](mcp-and-agents.md#uç-noktaya-başka-bir-makineden-erişmek). |

### Görünüm

![Aşağı kaydırılmış Görünüm sayfası: avatar kaynakları ve yedeği, girdi rengi, renk derinliği, metin imleci ve grafik renklerinin başı (tarih biçimleri aşağıda devam eder)](../images/settings-ui.png)

![Görünüm sayfasında Açık vurgulanmış Tema açılır listesi: Ayarlar ekranının tamamı o temayla yeniden boyanmış](../images/settings-appearance-theme.png)

| Seçenek | Ne yapar |
|---|---|
| **Tema** | **Sistem (terminali izler)**, yerleşik bir tema ya da kendi tema dosyalarınızdan biri. Vurgulanan tema canlı önizlenir: `Enter` onu korur, `Esc` öncekini geri getirir. Satırın altındaki **Renkler** şeridi geçerli renkleri gösterir. Bkz. [Temalar](themes.md). |
| **Yazar avatarları** | **kapalı** (varsayılan, ağ yok), **grafik** (commit noktasında) ya da **yazar** (Yazar sütununda). Açıkken her yazar e-postasının bir özeti Gravatar'a ve barındırma sağlayıcınıza gönderilir. |
| **Avatar kaynakları**, **Avatar yedeği** | Resimlerin nerede ve hangi sırayla aranacağı. Resmi olmayan yazarlar için: **identicon** (üretilmiş bir desen) ya da **baş harfler**. |
| **Girdi rengi**, **Özel girdi rengi** | Metin alanlarının arka planı: **Temayı izle** (varsayılan), sabit bir koyu renk ya da kendi `#rrggbb` değeriniz. |
| **Metin imleci** | `ui.text_cursor`: **terminal** (varsayılan) terminalin kendi imlecini odaktaki metin alanının düzenleme noktasına çubuk olarak koyar (terminalinizin imleç ayarları geçerli olur ve bir giriş yönteminin aday penceresi o noktada açılır); **drawn** GitPoulpi'nin kendi imlecini çizer ve terminal imlecini gizli tutar. Hemen uygulanır. Palet: `Ayarlar: Metin imleci...`. |
| **Renk derinliği** | **auto** (varsayılan), **truecolor**, **256** ya da **16**. Bkz. [Temalar](themes.md). |
| **Grafik renk teması** | **Temayı izle** (varsayılan: temanın kendi şerit renkleri) ya da herhangi bir şerit paleti. Her palet renklerinden birkaçını gösterir. Ayarlar kapandığında uygulanır. **Uygulama: Grafik Renk Temasını Değiştir...** palet komutu (`Ctrl+N Shift+T`) her paleti canlı önizler. |
| **Grafik şerit arka planı** | **Kapalı** (saydam) ya da **Satır tonu**: grafik sütununun arkasında şerit renginin soluk bir bandı ve kenarında ince bir çubuk. |
| **Satır tonu gücü** | Satır tonu bandının ne kadar soluk olduğu: **Güçlü**, **Orta** (varsayılan) ya da **Hafif**. Kenardaki çubuk tam rengini korur. |
| **Araç çubuğu konumu** | `ui.toolbar_position`: Araç çubuğunun yeri: Üst, Alt (varsayılan), Sol veya Sağ. Sol ve Sağ onu bölmelerin yanında bir sütun yapar; 80 sütundan dar bir pencerede altta çizilir. Hemen uygulanır. |
| **Pencere konumunu ve boyutunu hatırla** | `ui.dialog_geometry`: **Bu oturum için** (varsayılan), **Diske kaydedilir** veya **Asla**: taşıdığınız ya da yeniden boyutlandırdığınız bir pencerenin yerini koruyup korumayacağı. Tüm pencereler için tek ayar; her pencere kendi konumunu ve boyutunu hatırlar. Hemen uygulanır. |
| **Araç çubuğu düğmelerinin hizası** | `ui.toolbar_align`: Düğmelerin çubuk boyunca yeri: Baş (varsayılan), Orta veya Son. Baş, satırda sol ve sütunda üsttür; Son sağ ve alttır. Hemen uygulanır. |
| **Pull Request düğmesini sona sabitle** | `ui.toolbar_pr_pinned`: Açıkken (varsayılan) pull request düğmesi hizadan bağımsız olarak araç çubuğunun en ucunda durur (sağda, sütunda altta). Kapalıyken diğer düğmelerle birlikte akar ve onlarla hizalanır. Hemen uygulanır. |
| **Durum çubuğu konumu** | `ui.statusbar_position`: Durum çubuğunun (dal, sayılar, iletiler, tuş ipuçları) yeri: Üst (depo sekmelerinin üstü) veya Alt (varsayılan). Ana sayfa sekmesi altta kendi durum çubuğunu korur. Hemen uygulanır. |
| **Sekme çubuğu konumu** | `ui.tabs_position`: Depo sekmelerinin yeri: Üst (varsayılan), Alt veya Sol, pencerenin sol kenarında satır başına bir sekme olan bir liste. 80 sütundan dar bir pencerede üstte çizilir. Sütun biçimindeki bir araç çubuğunun yanında sekmeler en dışta kalır. Hemen uygulanır. |
| **Tarih/Saat Yerel Ayarı** | **sistem** (varsayılan): ay ve gün adları arayüz dilinde. **english**: her zaman İngilizce adlar. |
| **Tarih Biçimi**, **Tarih/Saat Biçimi**, **Ayrıntılı Tarih Biçimi** | Dosya Geçmişi ile Blame tarih sütununun, Log'un Tarih sütununun ve commit ayrıntılarının tarih kalıpları. Her biri canlı bir önizleme gösterir. Kalıpta kullanılabilen parçalar için bkz. [Yapılandırma başvurusu](config-reference.md#ui). |

![Grafik renk teması açılır listesi: Temayı izle, ardından her şerit paleti ve renklerinden birkaçı](../images/settings-log-graph-lanes.png)

### Klavye ve fare

![Klavye ve fare sayfası: bağlama göre her eylem ve tuşları, bir arama kutusu ve filtreler](../images/settings-keyboard.png)

**Ayarlar > Klavye ve fare** iki sekmeden oluşur (tıklayın ya da `Alt+Left` / `Alt+Right`).
**Genel** klavye protokolünü ve fare ayarlarını içerir; **Tuş atamaları** her kısayolu
bağlama göre, bir arama kutusu (`/`) ve filtrelerle listeler. Bir satırda:

| Tuş | Eylem |
|---|---|
| `Enter` ya da çift tıklama | Yeni bir tuş yakalar. Siz basarken çakışmalar ve terminal uyarıları görünür. |
| `a` | Alternatif bir tuş ekler. |
| `u` | Eylemin atamasını kaldırır. |
| `r` | Satırı varsayılanına sıfırlar. |
| `R` | Bir onaydan sonra her kısayolu sıfırlar. |
| `c` ya da satırın `!` işaretine tıklama | Eylemi, tuşunun çakıştığı tüm eylemlerle birlikte gösterir; `Esc` hepsini yeniden gösterir. Başka bir eylemin tuşuyla çakışan tuş kırmızı, yalnızca tuşla ilgili bir uyarı (terminal onu gönderemiyor) turuncudur. |

Bunu tmux, AutoHotkey, herdr ya da terminalinizle olan çakışmaları gidermek için kullanın.
Aynı ayarlar `config.toml` dosyasının `[keybindings]` tablosunda bulunur; bkz.
[yapılandırma başvurusu](config-reference.md#keybindings) ve
[Klavye kısayolları](keybindings.md).

![Quit üzerinde c'ye basıldıktan sonra Klavye ve fare sayfası: q tuşu, çakışan iki eylemde, Quit ve Stage or Unstage the Selected Row'da kırmızı](../images/settings-keyboard-conflicts.png)

Üç seçenek **Genel** sekmesindedir.

| Seçenek | Ne yapar |
|---|---|
| **Kitty klavye protokolü** | **auto**, terminalden kitty klavye protokolünü ister, böylece `Ctrl+,`, `Ctrl+Enter` ve `Ctrl+Shift+N` ulaşır. **off** hiç istemez; bir terminal sorun çıkarırsa kullanın. Yeniden başlatma gerekir. |
| **Fare açık** | Kapalıyken fare ve metin seçimi terminalinize kalır. Yeniden başlatma gerekir. |
| **Fare tekerleği satırları** | Tekerleğin bir adımının Log'da kaç satır kaydırdığı. Yeniden başlatma gerekir. |

### Log ve grafik

Commit grafiğinin nasıl çizildiği ve Log'un kaç commit yüklediği. Grafik renkleri
**Görünüm** sayfasındadır.

| Seçenek | Ne yapar |
|---|---|
| **Grafik çizici** | **auto**, terminalin desteklediği yerde bir resim, başka yerde karakterler çizer. **image** resmi, **text** karakterleri zorlar. Yeniden başlatma gerekir. |
| **Görüntü protokolü** | Resimlerin terminale nasıl ulaştığı: **auto**, **kitty**, **iterm2**, **sixel** ya da **none**. Yeniden başlatma gerekir. |
| **Grafikte stash'leri göster** | Log'da her stash için bir satır. Kapalı: stash'ler yalnızca kenar çubuğunda görünür. |
| **Üzerine gelinen Commit için soluk bir dal göster** | Varsayılan olarak açık (`ui.graph_ghost_ref`). Ref sütununda, kendi ref'i olmayan ve üzerine gelinen commit, ait olduğu dalın (yoksa uzak dalın) adını taşıyan soluk bir hap gösterir; ad geçmişten çıkarılır ve yalnızca sütunda yer varsa çizilir. Bir merge'den sonra silinen dal, merge commit'inin konusundaki adın altında yine üstü çizili görünür (`Merge branch 'x'`, `Merge pull request #N from owner/x`, ...); squash merge'ler, rebase'ler ve fast-forward'lar böyle bir iz bırakmaz. Tag'ler asla hayalet olarak gösterilmez. Hapın üzerine gelmek türünü gösterir. Tıklamak özel bir şey yapmaz; dalı `Git: Seçili Commit'te Dal Oluştur...` palet komutu oluşturur. |
| **Ref-commit bağlayıcısı** | Bir ref etiketinden commit noktasına ince bir çizgi. |
| **Sütun düzeni** | `ui.log_columns_layout`, Log'a hemen uygulanır. **Bölmeye sığdır** (varsayılan): her sütun bölmeye tam sığar ve yalnızca Mesaj esner. **Yana kaydır**: sütunlar genişliğini korur, Mesaj da, ve tablo bölmeden genişse yana kayar (`Shift+Left` / `Shift+Right`, yatay tekerlek, alt kenarlıktaki çubuk). Aynı anahtar Log'un sütun seçicisinde ve palette de vardır. |
| **Sütun yeniden boyutlandırma** | `ui.column_resize`, Log'a, Radar'a ve diff, dosya ve blame görünümlerinin commit sütununa hemen uygulanır. **Çizgi** (varsayılan): her sütun sınırı bir hücrelik boşluktur ve başlıkta küçük bir işaret taşır; imleç sınırın üzerindeyken tablo boyunca ince bir vurgu çizgisi çizilir ve boyunca herhangi bir yerden sürüklemek yeniden boyutlandırır. **Başlık**: işaret yok; sınırdan sonraki başlık hücresinin ilk karakteri aydınlanır ve sürüklemenin başlayabileceği tek yer odur. Boyutlandırma tuşları (Log'da `Shift+C`, Radar'da `C`, commit sütununda `Tab`) ikisinde de aynısını yapar. |
| **Değişiklikler sütunu gizliyken mesajda WIP değişiklik sayılarını göster** | Değişiklik sayılarını (`✎ 3  + 2  − 1`) WIP satırının mesajında gösterir. Değişiklikler sütunu görünürken sayılar oraya gider. |
| **Dal klasör ağacı** | Kenar çubuğunda dalları ve tag'leri `/` ile klasörlere gruplar. Refs panelinde `y` bunu açıp kapatır. |
| **Sol paneldeki bölümler** | Sol panelin gösterilen/gizlenen bölümlerinin kimin seçimi olduğu: **depo başına** (varsayılan; her depo kendi kümesini tutar, genel küme başlangıç noktasıdır) ya da **genel** (tüm depolar için tek küme). İkisi arasında gidip gelmek iki kümeyi de korur. Bkz. [fare ve hareketler](mouse-and-gestures.md). (`ui.sidebar_sections_scope`) |
| **Bir ref'e tıklamak onu grafikte gösterir** | Refs panelinde bir ref'e tek tıklamak Log'u işaret ettiği yere kaydırır ve satırı kısa süre vurgular (commit seçimi ve sağ panel yerinde kalır). Kapalı: bir tıklama yalnızca kenar çubuğu satırını seçer. **Grafikte göster** her iki durumda da çalışır. |
| **Composer önerileri** | YZ commit composer'ının her önerinin dosyalarıyla **açık** ya da **kapalı** olarak açılıp açılmayacağı. |
| **Grafikteki İlk Commit'ler** | Bir depo açıldığında Log'un kaç commit yüklediği. Varsayılan 5000, en az 500. Daha yüksek değerler büyük depoları yavaşlatır. |
| **Grafikte Commit'leri Tembel Yükle** | Açık: yüklenmiş en eski commit'e kaydırdığınızda daha fazla commit arka planda yüklenir. Kapalı: Log ilk commit'lerde durur. |
| **Tembel Sayfa Başına Commit** | Tek bir arka plan yüklemesinin kaç commit getirdiği. |
| **Grafikteki En Fazla Commit** | Tembel yükleme bu sayıda durur. `0` sınır yok demektir. Her commit yaklaşık 300 bayt bellek kullanır. |
| **Toplam commit sayısını şununla hesapla** | Durum çubuğunun `yüklenen / toplam` gösterimindeki toplamı nasıl öğrendiği (`log.commit_count`, bir açılır liste, hemen uygulanır): **Sistem git'i** tek bir `git rev-list --count` süreci çalıştırır (varsayılan, normalde en hızlısı); **libgit2** sayımı GitPoulpi içinde yapar ve süreç başlatmaz; virüsten koruma yazılımının her komutu taradığı ve her git çağrısını yavaşlattığı makineler içindir; **Kapalı** hiç saymaz ve yalnızca yüklenen commit'leri gösterir. |
| **Yazarları .mailmap ile göster** | Log, blame ve dosya geçmişi her yazarı deponun `.mailmap` dosyasının adlandırdığı gibi gösterir (`log.use_mailmap`, bir anahtar, varsayılan açık, hemen uygulanır). Saklanan commit'ler asla değişmez. Kapalıyken adlar commit edildiği gibi görünür. Bkz. `history-rewrite.md`, *Kimlikler sekmesi*. |

### Diff ve düzenleyici

![Diff ve düzenleyici sayfası](../images/settings-editor.png)

| Seçenek | Ne yapar |
|---|---|
| **Sekme boyutu** | Sekme durağı başına sütun, 1 ila 8 (varsayılan 4). |
| **Satır sonu** | Düzenleyicinin kaydederken yazdığı: **Otomatik (dosyanınkini koru)**, **LF** ya da **CRLF**. |
| **Sözdizimi vurgulama** | Düzenleyicide dil renkleri. |
| **Satır numaraları** | Düzenleyicinin ve diff'in satır numarası sütunu. |
| **Düzenleyicide seçimi izle** | Düzenleyici açıkken sağ panelde başka bir dosya seçmek onu düzenleyicide de açar; önce kaydedilmemiş değişiklikler için kaydetmeyi sorar. Düzenlenemeyen bir dosya olağan görünümünde açılır. Yalnızca çalışma değişiklikleri için: bir commit'ten seçilen dosya düzenleyiciden çıkıp o commit'in diff'ini gösterir. Varsayılan olarak açıktır (`editor.follow_selection`). |
| **Sözcük kaydırma** | Uzun diff satırlarını kaydırır. |
| **Varsayılan yerleşim**, **Yalnızca hunk'lar**, **Yalnızca boşluk değişikliklerini göster**, **Minimap** | Diff görünümünün ilk durumu. Diff görünüm çubuğu bunları değiştirir ve kaydeder. |
| **Blame açıklamaları**, **Blame açıklama genişliği** | Blame görünümünde açıklama sütununun (avatar, yazar, yaş, özet) hangi yanda olduğu: **sağ** (varsayılan) ya da **sol**; ve hücre cinsinden genişliği (24 ile 90 arası). Blame görünümünde sütunun kenarını sürüklemek veya `>` / `<` tuşları genişliği buraya kaydeder (`ui.blame_side`, `ui.blame_width`). Bkz. [Blame](log-and-history.md#blame). |
| **Merge düzenleyici: yan yana (sütun)** | Bu genişlikten itibaren merge düzenleyici A ve B'yi yan yana gösterir. Varsayılan 100, en az 40, `0` her zaman demektir. Altında A ve B sekmeler hâlindedir. Bkz. [Merge çakışmalarını çözmek](merge-conflicts.md). |
| **Harici düzenleyici** | Bir dosyanın *Harici düzenleyicide aç* öğesi için komut. `{file}` yoldur. Boşsa: kuruluysa VS Code, yoksa sistem varsayılanı. |
| **Harici diff aracı** | *Harici diff aracında aç* için komut. `{local}` ve `{remote}` iki sürümdür. Boşsa: `git difftool`. |
| **Harici merge aracı** | Bir çakışma kartında *Merge aracında aç* için komut. `{base}`, `{local}`, `{remote}` ve `{merged}` dosyalardır. Boşsa: `git mergetool`. |
| **Dosya gezgini** | *Klasörde göster* için komut. `{path}` dosya, `{dir}` klasörüdür. |

### Şurada aç

![Şurada aç sayfası, Girdiler sekmesi: girişlerin listesi, etiketi ve komutuyla düzenlenen giriş ve + Ekle düğmesi](../images/settings-open-in.png)

Başlık çubuğundaki **Şurada aç** düğmesinin listesini düzenler. **Genel** sekmesi **Çık ve cd yap**
klasör düğmelerini, **Varsayılan girişi** ve **Algılanan varsayılanları geri yükle** düğmesini içerir.
**Girdiler** sekmesi (her biri bir etiket ve bir komut) bir [liste](#listeler)dir: **+ Ekle ▾** hazır ayarları ve **Özel**'i
sunar, varsayılan giriş `★` ile işaretlenir. Seçili girişin alanları arasında **Worktree'yi açabilir**
(varsayılan olarak işaretli: giriş, bir worktree satırının Şurada aç menüsünde sunulur,
bkz. [Worktree'ler](worktrees.md)) ve bir **Dene** düğmesi vardır. Bkz. [Şurada aç](open-in.md).

İlk satır, **Etkin sekmenin klasörünü izle**, iki değer alır:

- **buttons** (varsayılan): etkin sekme GitPoulpi'nin başlatıldığı klasör olmadığında
  sekme satırı **Çık ve cd yap** düğmesini gösterir.
- **auto**: düğme yok. Her çıkış kabuğunuzun klasörünü etkin sekmeye değiştirir ve yeni
  bir sekme açmak varsayılan Şurada aç girişini çalıştırır.

Bkz. [Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md).

### Görseller

Görsel dosyalarının nasıl gösterildiği ve **Şununla düzenle** işleminin hangi programı kullandığı.
Bkz. [Görseller](image-view.md). **Genel** sekmesi aşağıdaki görüntüleyici seçeneklerini,
**Varsayılan girişi** ve **Algılanan varsayılanları geri yükle** düğmesini içerir; **Düzenleyiciler**
sekmesi görsel düzenleyicilerin listesidir. Görsel önizlemesi olmayan bir derlemede görüntüleyici seçenekleri yoktur.

| Seçenek | Ne yapar |
|---|---|
| **Görselleri önizle** | Bir görsel dosyasını Diff Görünümü'nde ve Dosya Görünümü'nde resim olarak gösterir. Kapalı: düz ikili dosya bildirimi. Varsayılan olarak açıktır. |
| **Görsel protokolü yokken** | Satır içi görsel protokolü olmayan bir terminalde: **Yarım blok renkleri** (varsayılan) ya da **Yalnızca bilgi kartı**. |
| **En fazla önizleme boyutu** | Daha büyük görseller çözülmez; bunun yerine bilgi kartı görünür. Bir sayı ve MiB ya da GiB, varsayılan 32 MiB, en çok 1 GiB (`ui.image_view_max_mib`, `ui.image_view_max_unit`). |
| **Şu yakınlaştırmadan itibaren pikselli** | `1x`, `2x` (varsayılan) ya da `never`: bu yakınlaştırmadan itibaren resim, yumuşatılmış bir resim yerine kare pikseller gösterir. |
| **Önce/sonra yakınlaştırma ve kaydırmayı eşitle** | Bir görsel diff'inin iki paneli tek bir yakınlaştırmayı ve tek bir kaydırma konumunu paylaşır. Varsayılan olarak açıktır. |

**Düzenleyiciler** sekmesi, [Şurada aç](open-in.md) gibi bir [liste](#listeler)dir:
düzenleyici başına bir **Etiket**, bir **Komut**, değişken
çipleri `${file}` `${dir}` `${name}` `${repo}` ve **Örnek görselle dene**;
**+ Ekle ▾** hazır ayarları ve **Özel**'i sunar, **Algılanan varsayılanları geri yükle** listeyi
sıfırlar. Siz düzenleyene kadar liste, bu makinede kurulu düzenleyicileri ve sistemin
varsayılan uygulamasını içerir. Her değişiklik hemen uygulanır.

### Çakışma önleme

Proaktif çakışma algılamanın genel kısmı (`gitpoulpi ui show settings conflict-prevention-global`); bir deponun kendi sayfası
[Depoya Özel: Çakışma önleme](#depoya-özel-çakışma-önleme)'dir. Paletteki **Çakışmalar: Temel Dal Listelerini Düzenle...** bu sayfayı açar.

| Seçenek | Ne yapar |
|---|---|
| **Proaktif çakışma algılamayı etkinleştir** | Varsayılan olarak açıktır (`general.conflict_detection`). Geçerli dalı ayrıldığı dala merge etmek çakışacaksa, siz denemeden önce Log'da uyarır. Bir depo bunu kendi **Çakışma önleme** sayfasında kendisi için kapatabilir. Bkz. [Merge çakışmalarını çözmek](merge-conflicts.md#çakışma-önleme). Ayarlar kapanınca uygulanır. |
| **Varsayılan temel dallar** | Anahtarın altında. Kendi listesi olmayan depoların kullandığı temel dal desenleri listesi: her dal demek olan **Yerleşik (`**`)** ya da adlandırılmış listelerinizden biri (`conflict_prevention.default_list`). Ayarlar kapanınca açık depolara uygulanır. **Yerleşik varsayılana sıfırla** `**` değerine döner. |
| **Temel dal listeleri** | Yine bu sayfada: adlandırılmış temel dal desenleri listeleriniz (`[conflict_prevention.lists.<ad>]`); her biri bir blok ve içinde **Ad** (yeniden adlandırmak, varsayılansa varsayılan olarak kalmasını sağlar; eski adı izleyen bir depo, listeyi yeniden seçene kadar varsayılan listeyi kullanır), **Desenler** (satır başına bir tane, `Enter` veya tıklama düzenleyiciyi açar, `Ctrl+S` korur), **Varsayılan yap** ve **Listeyi sil** (önce sorar ve onu izleyen depoların bunun yerine varsayılan listeyi kullanacağını söyler). Eklemek için **Yeni liste** alanına bir ad yazıp `Enter`'a basın; `**` ile başlar. Bir listeyi düzenlemek, onu izleyen her depoyu değiştirir. |

### Depoya Özel: Git ve Kodlama

![Bu depo için Git ve Kodlama sayfası](../images/settings-git.png)

| Seçenek | Ne yapar |
|---|---|
| **Git arka ucu** | `git2` (libgit2, varsayılan) ya da `system` (kurulu `git`'iniz). Geneldir. Değişiklikten sonra başlatılan daemon'lara uygulanır. |
| **Pull düğmesi** | Pull düğmesinin ne yaptığı: merge (mümkünse fast-forward), yalnızca fast-forward ya da rebase. Geneldir. `▾` öğeleri bunu değiştirmez. |
| **Push düğmesi** | **doğrudan** hemen push eder; **sor** önce uzak depo menüsünü gösterir. Geneldir. |
| **Her uzak depoyu Push öncesinde denetle** | Varsayılan olarak açık (`git.push_group_preflight`). Ek push hedefleri olan bir dalı push etmeden önce önce her uzak depoya bakar (hızlı bir sorgu; o uzak depodan indirme yalnızca farklıysa yapılır), böylece üçüncü uzak depodaki bir sorun ilk push başlamadan görünür. Kapalıyken her uzak depoya sırayla push eder ve olanları bildirir. Geneldir. |
| **Merge fast-forward** | Başlattığınız bir merge'ün, dal sadece fast-forward yapılabilecekken yaptığı: **never** (varsayılan) her zaman bir merge commit'i oluşturur, **allowed** mümkünse fast-forward yapar (git'in kendi varsayılanı), **only** fast-forward yapar ya da reddeder. Geneldir (`git.merge_fast_forward`); bir depo bunu kendi [Merge seçenekleri](#depoya-özel-merge-seçenekleri) sayfasında geçersiz kılar. |
| **Varsayılan kodlama** | **Geçerli UTF-8 olmayan** metni (dosya içerikleri, blame, diff'ler ve commit mesajları) göstermek için kullanılan kodlama. Sayfada genel bir satır ve bu depo için bir satır vardır; depo satırı baskındır. Geçerli UTF-8 asla yeniden çözülmez ve staging hiçbir şeyi yeniden kodlamaz. |
| **core.autocrlf**, **core.eol** | Bu deponun kendi Git ayarları. Satır, deponun Git yapılandırmasına `core.autocrlf` ya da `core.eol` yazar. **(devralınan)** depo değerini kaldırır, böylece genel Git yapılandırmanız geçerli olur. `Ctrl+Z` önceki değeri geri getirir. CLI karşılığı `gitpoulpi repo-config`'tir. |
| **Yeni dallar için ek Push hedefleri** | Yalnızca bu depo: bu başlığın altında deponun her uzak deposu için bir onay kutusu. İşaretli bir uzak depo, dal ilk kez bir upstream aldığı anda (bunu ayarlayan ilk push veya bir uzak daldan oluşturulan dal) her yeni dalın upstream'inin yanında ek push hedefi olur. Seçim dalın kendi `gitpoulpi-push-to` anahtarlarına kopyalanır, bu yüzden var olan dallar değişmez: onları Refs satırlarındaki **Uzak depolar** rozetinden (veya satırda `u` ile) düzenleyin. Deponun verisinde (`repo-state.toml` içinde `default_push_targets`) saklanır. |
| **Git LFS** | Git LFS'yi arayüzde gösterir. Bkz. [Git LFS](lfs.md). |
| **Pull ve Push öncesinde LFS filtrelerini kur** | Bir pull ya da push öncesinde, depo LFS kullanıyor ve filtreleri eksikse `git lfs install --local` çalıştırır. |
| **git-lfs programı** | Algılanan `git-lfs` programı (salt okunur). |
| **Bu depo için LFS'yi başlat** | `git lfs install --local` komutunu şimdi çalıştırır. |

### Depoya Özel: Kısmi Checkout

Kısmi checkout iletişim kutusuyla (`Depo: Kısmi Checkout...`) aynı denetimler, bu deponun ayarlarından bir sayfa olarak
(`gitpoulpi ui show settings sparse-checkout`).

| Seçenek | Ne yapar |
|---|---|
| **Durum** | Bu çalışma ağacının kısmi checkout olup olmadığı. |
| **Tutulacak klasörler** | Satır başına bir klasör, deponun köküne göre. `Enter` (veya bir tıklama) çok satırlı bir düzenleyici açar: `Ctrl+S` metni korur, `Esc` atar. Kısmi checkout açıkken alan geçerli klasörleri gösterir. Git'in cone kipi yalnızca klasörleri bilir (desen ve tek tek dosya yok), kökteki dosyalar her zaman checkout edilir ve boş bir liste reddedilir. |
| **Kısmi checkout'u başlat** | Yalnızca listelenen klasörleri checkout eder. Kısmi checkout açıldıktan sonra aynı düğme **Klasör listesini uygula** olur ve düzenlenen listeyi uygular: listeden çıkan bir klasör çalışma ağacınızdan kaldırılır. Liste boşken soluktur. |
| **Kısmi checkout'u durdur** | Tam checkout'a döner: tüm dosyalar geri gelir (bir onaydan sonra). Kısmi checkout kapalıyken soluktur. |
| **Kısmi checkout'u yenile** | Geçerli listeyi yeniden uygular (örneğin bir betik ya da başka bir araç liste dışındaki dosyaları checkout ettikten sonra). Alandaki gönderilmemiş metni değil, geçerli klasörleri gönderir. Kısmi checkout kapalıyken soluktur. |

Düğmeler olağan arka plan eylemleri olarak çalışır (biri çalışırken sayfa "kısmi checkout uygulanıyor..." gösterir). İletişim
kutusu gibi, bunlar da Geri Al yığınında değildir: hiçbir şey kaybolmaz ve **Durdur** her dosyayı geri getirir.

### Depoya Özel: SSH ve kimlik bilgileri

`gitpoulpi ui show settings ssh-credentials`. *Bu deponun* nasıl oturum açtığı, GitPoulpi'nin özel klasöründe
tutulur: depoya veya git yapılandırmasına hiçbir şey yazılmaz ve geri alınamaz (her ayar gibi). Yalnızca bu deponun
fetch, pull, push, alt modül ve LFS işlemlerinde kullanılır. CLI karşılığı `gitpoulpi repo-auth` komutudur
(bkz. [CLI başvurusu](cli-reference.md)). Sayfaya komut paletinden de ulaşılır:
**Depo: SSH ve Kimlik Bilgileri...**. **Kaydet**'e basana kadar hiçbir şey gönderilmez; sayfa daemon'a sorar, bu
yüzden asla bloke olmaz.

| Satır | Ne yapar |
|---|---|
| **HTTPS: Oturum açma yöntemi** | **Sistem** (git'in kendi yardımcıları, varsayılan), **GitPoulpi hesabı** (bir entegrasyonun token'ı; sunucusu bu deponun uzak depolarından biri olan hesaplar öne gelir ve altındaki satır son token denetiminin bulduğunu söyler: eksik ve fazla kapsamlar), **Kimlik bilgisi yardımcısı** veya **Token komutu**. |
| **Git kullanıcı adı** | HTTPS uzak depolarına kullanıcı adı olarak gönderilir. **Sistem** ile bir kimlik bilgisi yöneticisinin saklanan birkaç hesaptan birini seçmesini sağlar. |
| **Bu deponun ayarladığı kimlik bilgisi yardımcılarını yok say** | Yalnızca **Sistem**. Deponun kendisi için ayarladığı yardımcılar (`.git/config` dosyası, içerdiği dosyalar) altında, her birinin geldiği dosyayla listelenir; sistem ve genel yardımcılar çalışmaya devam eder. |
| **Yardımcı**, **Komut**, **Sunucular**, **Önbellek (dakika)** | Deponun oturum açmak için çalıştırdığı bir komut. *Yardımcı* bir git kimlik bilgisi yardımcısıdır; *token komutu* bir token yazdırır (bağımsız değişken listesi boşluklarla bölünür, boşluk içeren bir bağımsız değişken tırnak içine alınır ve **Şöyle çalışır** altında gösterilir; kabuk çalıştırmaz, depo klasöründe çalışır). **Sunucular** onu bu sunucularla sınırlar (boş: ana uzak deponun sunucusu); **Önbellek** bir token'ın ne kadar süre yeniden kullanılacağıdır (varsayılan 50 dakika). **Bu deponun geçerli yardımcısını kullan**, deponun ayarladığı yardımcıyı alana kopyalar. |
| **Sunucu API'si için kullan (issue'lar, pull request'ler)** | Bir token komutuyla: aynı komut, bu deponun issue ve pull request çağrılarının token'ını da kişisel token'ınızın yerine verir (bir GitHub Uygulaması kurulum token'ı; böylece yorumlar Uygulama olarak görünür). Açıp kapatmak onay ister. [Barındırma entegrasyonları](hosting-integration.md) kılavuzunda "GitHub Uygulaması botu olarak çalışmak" bölümüne bakın. |
| **Atama** | **İzinli** (varsayılan) veya **Engelli**; hemen kaydedilir. Engelliyken bu deponun issue'larında ve pull request'lerinde uygulamadan, komut satırından veya bir ajandan hiçbir şey atanamaz; mevcut atananlar görünür kalır. |
| **SSH: Oturum açma yöntemi** | **Sistem** (ssh'ın kendi kurulumu, ajan ve varsayılan anahtarlar), **Anahtar dosyası** veya **Hesap anahtarı** (bir entegrasyonun anahtarı). |
| **Özel anahtar dosyası** | `~/.ssh` içinde bulunan anahtarlardan birini seçin veya bir yol yazın. Yalnızca bu anahtar, deponun alt modüller dahil her SSH uzak deposuna sunulur. Başka hesaplar anahtar dosyasını okuyabiliyorsa (Linux ve macOS'ta grup veya diğerleri için bir izin biti, Windows'ta Everyone, Users veya Authenticated Users için bir girdi) bir uyarı OpenSSH'in böyle bir anahtarı reddettiğini ("bad permissions") söyler ve **Yalnızca hesabıma kısıtla** bunu düzeltir (`chmod 600`, Windows'ta yalnızca hesabınız ve SYSTEM için bir DACL). |
| **Bilinen sunucular dosyası**, **Sıkı sunucu anahtarı denetimi** | Güvenilen sunucu anahtarlarından oluşan isteğe bağlı bir dosya ve içinde olmayan bir sunucuyu reddetme. |
| **ssh programı** | Çalışacak program (salt okunur). |
| **git yapılandırmasında sabitlenmiş** | Yalnızca bu depo kendi `core.sshCommand` değerini ayarlamışsa (GitPoulpi'nin eskiden yazdığı eski "Bu depo için SSH anahtarı" sabitlemesi). Komut salt okunur gösterilir. **GitPoulpi'ye taşı**, eşdeğer **Anahtar dosyası** seçimini kaydeder, sonra sabitlemeyi git yapılandırmasından kaldırır; komut yalın bir `ssh -i <key>` sabitlemesiyse sunulur, o da yalnızca sayfada kaydedilmemiş düzenleme yokken ve SSH seçimi hâlâ **Sistem** iken. Başka her komut bunun yerine bir notla gösterilir: `git config` ile düzenleyin. Taşıma, buradaki her ayar gibi geri alınamaz ve sabitlemenin kaldırılması da Geri Al yığınında değildir (hiçbir şey kaybolmaz: anahtar profildedir). |
| **Uzak depolar** | Salt okunur: her uzak depo, protokolü ve kullanacağı kimlikle. Profilin geçersiz kıldığı ayarlar (`core.sshCommand`, `credential.helper`) dosyalarıyla listelenir ve kullanılamayan bir profil nedenini söyler. |

Düğmeler (her birinin bir tuşu var: aralarında `Tab`/`Left`/`Right`, `Enter` basar):

- **Bağlantıyı sına**, her uzak depoyu kaydedilen seçimlerle denetler, önce fetch sonra push (veri taşınmaz), ve bir
  sonuç paneli açar: uzak depo başına tamam veya başarısız, yanıt veren kimlik (sunucunun kabul ettiği anahtar veya
  token'ı kabul edilen hesap), süre ve bir başarısızlığın sade bir açıklaması. Arka planda çalışır, bir an sonra bir
  döner simge görünür. Panel `Up`/`Down`, `PgUp`/`PgDn`, `Home`/`End`, tekerlek ve kaydırma çubuğuyla kayar,
  `Esc` veya çarpısıyla kapanır. Palet karşılığı **Depo: Bağlantıyı Sına**'dır.
- **Kaydet** seçimleri gönderir. **Geri al** kaydedilmemiş olanları bırakır. Kaydedilmemiş seçimlerle Ayarlar'ı kapatmak önce sorar.
- **Varsayılanları kullan** sorar, sonra profili unutur: HTTPS için git'in yardımcıları ve SSH için ssh'ın kendi kurulumu.
  Palet karşılığı **Depo: Varsayılan Kimlik Bilgilerini Kullan**'dır, o da sorar.

Bir **Kimlik bilgisi yardımcısı** veya bir **Token komutu** kaydetmek önce **Bu komuta izin verilsin mi?** diye sorar:
komut kelimesi kelimesine gösterilir (kontrol karakterleri yazıyla belirtilir, uzun satırlar sarılır), SHA-256 değeri ve
iki yanıtla: `1` **Kaydet ve izin ver** ve `2` **İptal**. Bir ok tuşuna veya rakama basana kadar hiçbir yanıt seçili
değildir, bu yüzden yanlışlıkla basılan `Enter` hiçbir şey yapmaz; `Esc`, **İptal** ve panelin çarpısı iptal eder.
`PgUp`/`PgDn`/`Home`/`End` ve kaydırma çubuğu uzun bir komutu kaydırır. Komut zaten saklıysa ve değişmediyse, sayfanın
öbür yarısını kaydetmek yeniden sormaz.

Kullanılamayan bir profil (eksik bir anahtar dosyası, bu sürümün henüz çalıştırmadığı bir kaynak), aktarımın makinenin
kendi kimliğine geri dönmek yerine nedeniyle başarısız olmasına yol açar.

Profil tamamen varsayılan değilse, **Push** menüsü push'un kim olarak oturum açtığını söyleyen bir satırla biter
(**Oturum açılan hesap: github-corp (token)** veya **SSH anahtarı: work_ed25519**); bu sayfayı açar. Aynı satır
fetch, pull ve push'un Etkinlik Günlüğü girdilerine de eklenir. Bu sayfanın seçimlerinin yol açtığı bir başarısızlık
(çalışmayan bir kimlik bilgisi yardımcısı, henüz bilinmeyen bir SSH sunucu anahtarı, reddedilen bir oturum açma,
başarısız olan bir token komutu, sunucu hesabın adresi olmadığı için gönderilmeyen bir token) Etkinlik Günlüğü
girdisinde bir **SSH ve kimlik bilgileri...** düğmesi gösterir (girdide `r`); kaynak bir entegrasyon hesabı olabiliyorsa
**Entegrasyonları aç** düğmesinin yanında. Sekme menüsünün **SSH ve kimlik bilgileri...** satırı da sayfayı açar; eski
**SSH anahtarı...** sabitleme menüsünün ve palet girdisinin yerini alır.

### Depoya Özel: LFS

Bu deponun LFS durumu (`gitpoulpi ui show settings lfs`; LFS'siz bir derlemede yoktur).

- **Başlatılmamış:** "Git LFS `<depo>` içinde kurulu değil" ve `git lfs install --local` çalıştıran **LFS'yi burada kur**
  düğmesi (bu deponun filtreleri ve hook'ları; genel yapılandırmanıza asla dokunmaz; `git-lfs` gerekir). Deponun
  `.gitattributes` dosyası dosyaları zaten LFS üzerinden yönlendiriyor ama hook'ları eksikse, sayfa bunu söyler.
- **Başlatılmış:** "Git LFS `<depo>` içinde kurulu" ve `.gitattributes` içinde izlenen desenler
  (`filter=lfs`).

Bu depoda kendi LFS hook'ları varsa (`git lfs install --local` komutunun yazdığı `pre-push` veya `post-checkout` hook'u)
LFS **başlatılmış** sayılır. Yalnızca filtreler sayılmaz: kullanıcı düzeyindeki `git lfs install` bunları her depo için
ayarlar, bu yüzden her depo başlatılmış görünürdü.

Araç çubuğundaki **`LFS ▾`** düğmesi de aynı durumu izler: yalnızca LFS'nin başlatıldığı ya da `.gitattributes` dosyasının
onu zaten kullandığı bir depoda görünür, sıradan bir depoda gizli kalır (bir deponun ilk LFS okuması gelene kadar da,
yani depo açıldıktan bir an sonra belirebilir). Bu sayfadan başlatmak onu görünür kılar. `LFS:` palet komutları her yerde
kullanılabilir kalır, **LFS: Bu Depo İçin Başlat** dahil.

### Depoya Özel: Commit

Bu depoda yapılan bir commit'in nasıl davrandığı (`gitpoulpi ui show settings commit`). İki anahtar da
varsayılan olarak **kapalıdır** ve yalnızca bu depo için kaydedilir ([`repo-state.toml`](config-reference.md#depo-başına-ayarlar-gitpoulpirepo-statetoml)
içindeki `[preferences]`).

| Ayar | Ne yapar |
|---|---|
| **Her commit sonrasında push** | Başarılı bir commit'ten sonra checkout yapılmış dalı olağan push gibi upstream'ine push eder (izlenen bir aktarım; sırasında "commit edilip push ediliyor..." yazar). Upstream'i olmayan bir dal commit'lenir ama push edilmez ve bildirim bunu söyler: yeni bir dalın ilk push'u, upstream'i ayarlayan **Push** düğmesinde kalır. Reddedilen bir push olağan push-reddedildi kartını açar; commit yerinde kalır. |
| **Her commit için git hook'larını varsayılan olarak atla** | Commit formundaki **Hook'ları atla** onay kutusu (**Önceki commit'i amend et** yanında; Durum panelinde `Shift+H` veya paletteki **Commit: Hook'ları Atlamayı Aç/Kapat**) işaretli başlar; böylece `pre-commit` ve `commit-msg` hook'ları çalışmaz (`git commit --no-verify`). Her commit için ayrı verilen bir karar olarak kalır: commit'ten sonra bu varsayılana döner. |

Varsayılan olmadan, GitPoulpi hangi arka ucu kullanırsa kullansın deponun commit hook'ları her commit'te çalışır (commit
hook'u olan bir depo sistem `git` ile commit'lenir, çünkü libgit2 hiçbirini çalıştırmaz). Başarısız olan bir hook çıktısını
Etkinlik Günlüğü girdisinde gösterir (durum çubuğu commit'in başarısız olduğunu söyler) ve hiçbir şey commit'lenmez.

### Depoya Özel: Merge seçenekleri

İki ayar. **Fast-forward**, dal sadece fast-forward yapılabilecekken bir merge'ün ne yapacağını söyler:

| Seçim | Başlattığınız bir merge... |
|---|---|
| **(inherited)** | Git sayfasındaki genel **Merge fast-forward** ayarını izler (varsayılan **never**). |
| **never** | dal tam önünde olsa bile her zaman bir merge commit'i oluşturur (`git merge --no-ff`). Commit'in ilk ebeveyni eski uç, ikincisi merge edilen daldır ve mesajı git'in kendisidir (`Merge branch 'x' into y`). |
| **allowed** | yapabiliyorsa fast-forward yapar, yapamıyorsa bir merge commit'i oluşturur (git'in varsayılanı). |
| **only** | fast-forward yapar, dallar ayrışmışsa *cannot fast-forward* ile reddeder (`git merge --ff-only`). |

Başlattığınız her merge için geçerlidir: dalın menüsü, sürükle bırak, paletteki `Merge` ve `gitpoulpi merge` (bunun `--ff`,
`--no-ff` ve `--ff-only` bayrakları tek bir merge için ayarı geçersiz kılar). Menüler merge'ün ne yapacağını söyler:
*Merge: x → y (merge commit)*, *(fast-forward)* veya *(yalnızca fast-forward)*; ayrı *Fast-forward: y → x* öğesi (ve palet
komutu) her seçimde tek bir hareket uzaklıkta kalır. Bir merge commit'i, tıpkı bir fast-forward gibi sıradan bir geri alma
girdisidir. Flow eylemleri kendi merge stratejilerini seçer ve bu ayarı yok sayar.

Kendi seçiminiz git'inkini geçersiz kılar: **never** veya **only** altında GitPoulpi'nin başlattığı bir merge bayrağı açıkça
geçirir, bu yüzden deponun git yapılandırmasındaki bir `merge.ff` geçersiz kılınır. Git'in `merge.ff` ayarı yalnızca **allowed**
altında etkindir ve depo bunu ayarladıysa sayfa onu satırın altında gösterir. (Bu satır için yalnızca deponun kendi
`.git/config` dosyası okunur, genel git yapılandırmanız değil.)

**Squash** (varsayılan olarak kapalı). Açıkken yerel bir dalı merge etmek (dalın menüsü, sürükle bırak,
paletteki `Merge`), merge commit'i oluşturmadan dalın değişikliklerini stage'ler (`git merge --squash`): commit'i
tamamlamak size kalır. Commit formu, her squash için bir kez, git'in squash mesajıyla (`SQUASH_MSG`, yorum satırları çıkarılmış)
açılır. Sonrasında devam eden bir işlem yoktur (`MERGE_HEAD` yok), bu yüzden **İptal et** geçerli değildir: vazgeçmek için
stage'lenmiş değişiklikleri atın. Çakışmada duran bir squash, çakışmaları herhangi bir çakışmalı merge gibi Durum panelinde
bırakır. Stage'lenmiş değişiklikler bir geri alma girdisi değildir (hiçbir ref taşınmadı, geri yüklenecek bir şey yok);
CLI'nin `gitpoulpi merge <dal> --squash` komutu, ayardan bağımsız olarak aynısını yapar. Squash, **Fast-forward** ayarını geçersiz kılar.

### Depoya Özel: Mesaj şablonu

Commit formunun başladığı bir mesaj (`gitpoulpi ui show settings commit-template`).

- **Boş commit panelini şablonla önceden doldur**: *boş* bir commit formu, her commit için bir kez şablonla önceden doldurulur
  (elle temizlemeniz bir sonraki commit'e kadar saygı görür).
- **Commit yapılırken yorum satırlarını çıkar**: commit yapılırken `#` ile (veya deponun `core.commentChar` karakteriyle)
  başlayan satırlar, `git commit --cleanup=strip` gibi mesajdan çıkarılır. GitPoulpi, git'in temizlemediği `-m` ile commit
  yaptığı için bunu kendisi yapar. Anahtar kapalıyken `#` satırı commit'te kalır.
- **Özet** (etiket kalan 72 sütunu ve kaç tanesinin aşıldığını sayar) ve çok satırlı bir **Açıklama** (`Enter` veya
  tıklama düzenleyiciyi açar, `Ctrl+S` korur).
- **Şablonu kaydet**, şablonu deponun veri klasöründeki `commit-template.txt` dosyasına yazar (bkz.
  `session.per_repo_state_dir`) ve bu deponun `commit.template` ayarını o dosyaya yönlendirir; böylece komut satırındaki
  `git commit` ve diğer araçlar da onu kullanır. `commit.template` zaten başka bir dosyayı gösteriyorsa sayfa o dosyanın
  yolunu gösterir, *o dosyayı* düzenler ve ayara dokunmaz.

### Depoya Özel: Çakışma önleme

`gitpoulpi ui show settings conflict-prevention`. Bu deponun proaktif çakışma algılama sayfası
(bkz. [Merge çakışmalarını çözmek](merge-conflicts.md#çakışma-önleme)):

- **Proaktif çakışma algılamayı hedef dalla etkinleştir**: varsayılan olarak açık. İşareti kaldırmak algılamayı
  yalnızca bu depo için kapatır; genel anahtar, genel **Çakışma önleme** sayfasındaki **Proaktif çakışma algılamayı etkinleştir**'dir.
  İkisi de açık olmalıdır.
- **Temel dalların kaynağı**: bu deponun temel dallarının nereden geldiği. **Varsayılan** genel varsayılanı izler (yerleşik
  `**` ya da Ayarlar > Çakışma önleme'de **Varsayılan temel dallar** altında seçilen liste); **Liste: <ad>** adlandırılmış listelerinizden
  birini izler, yani listeyi düzenlemek bu depoyu da değiştirir; **Kendi listesi** deponun kendine ait listesidir (seçmek,
  geçerli desenlerin bir kopyasıyla başlar). Bu seçim var olmadan önce yazılmış ve kendi desenlerini tutan bir depo dosyası
  **Kendi listesi** olarak okunur.
- **Temel dallar** (satır başına bir desen): **Kendi listesi** ile `Enter` veya tıklama düzenleyiciyi açar ve `Ctrl+S` korur;
  **Varsayılan** veya **Liste** ile geçerli desenler salt okunur gösterilir. Altındaki desen tablosu başvuru niteliğindedir:

  | Desen | Neyle eşleşir |
  |---|---|
  | `origin/main` | yalnızca o dal |
  | `origin/**` | `origin` uzak deposunun her dalı |
  | `origin/feature/**` | `origin` deposunun `feature/` ile başlayan dalları |
  | `!origin/abc` | o dalı hariç tutar |

  Yerel bir dal yalın adıdır (`main`), uzak izleme dalının önünde uzak deposu bulunur (`origin/main`). `*` ve `?` tek
  bir yol bölümünün içinde kalır, `**` bölümleri aşar. Boş satırlar ve `#` yorumları yok sayılır; yalnızca hariç tutma
  içeren bir liste "bunlar dışındaki her dal" anlamına gelir. Seçim `repo-state.toml` içinde saklanır ve Ayarlar
  kapanınca yeniden okunur.
- **Liste olarak kaydet...**: bir ad yazıp `Enter`'a basın; geçerli desenler adlandırılmış bir liste olur (Ayarlar > Çakışma önleme'nin
  düzenlediği listelerle aynı) ve bu depo onu izler.
- **Varsayılana sıfırla** (önce sorar): bu deponun seçimini, listesini ve kendi desenlerini bırakır; depo yeniden varsayılanı
  izler.
- **İş akışının temel dallarını kullan** ([dallanma iş akışı](branching-workflows.md) temel dalları olan bir depo için kurulmuşsa
  gösterilir): **Kendi listesi**ni o temel dallarla doldurur, her biri yerel dal olarak ve herhangi bir uzak deponun dalı olarak
  (`main` ve `*/main`). Bir kez kopyalar; iş akışının sonraki değişikliklerini izlemez.

### Depoya Özel: YZ

![Deponun YZ sayfası: Genele ekle ya da Geneli değiştir seçenekli depo talimatları ve her eylem için bir profil ve talimatlar](../images/settings-repo-ai.png)

Bu deponun YZ profili ve her eylem için talimatları ile bu depodaki her eylem için geçerli
olan *Depo Talimatları*. Boş alanlar genel YZ ayarlarını kullanır. Bkz.
[YZ desteği](ai.md#talimatlar). *Kod incelemesi* bloğu dört inceleme bağlamı satırını yineler: her seçim
*Genel olanı kullan (değer)* ile başlar ve genel değeri gösterir, boş bir **Ek dosya** genel olanı gri gösterir (ve aynı **Gözat...** düğmesine sahiptir; seçici depo kökünden başlar ve depo içindeki bir dosya köke göre saklanır);
**Ek dosya yok** anahtarı bu depo için genel ek dosyayı kapatır, boş bir alan ise onu izlemeye devam eder; **Genele sıfırla** (depo kendi değerlerine sahip olunca görünür) hepsini geri koyar. Bu deponun
`repo-state.toml` dosyasında saklanırlar.

### Depoya Özel: İş akışı

`gitpoulpi ui show settings workflow`. Bu deponun dal iş akışının (şablon, algılama, taban dal) kendi kılavuzu vardır: [Dal iş akışları](branching-workflows.md#ayarlar--i̇ş-akışı). **Genel** sekmesi kural olmayanları
(şablon, algılama, hook'lar, issue akışı, nereye kaydedildiği) içerir; **Kurallar** sekmesi gruplara ayrılmış bir
[liste](#listeler)dir (**Temel dallar**, **Konu türleri**, **Bitirme kuralları**); **+ Ekle ▾** türü seçer ve
öğeler kendi grubunun içinde taşınır. Diğer sayfalardan farklı olarak bir taslak tutar (çubuk iki sekmede de vardır): sayfanın altına sabitlenmiş
bir çubuk **● Kaydedilmedi** (ya da **Değişiklik yok**) ile birlikte **Paylaşılan dosyaya kaydet**, **Yalnızca benim için** ve
**Değişiklikleri at** düğmelerini gösterir; kaydedilmemiş bir taslakla Ayarlar'ı kapatmak önce sorar.

## Dosyalar, ortam değişkenleri ve CLI

- Ayar dosyası, işletim sisteminizin yapılandırma klasöründeki `config.toml` dosyasıdır.
  `gitpoulpi config path` konumunu yazdırır.
- Depo başına durum, deponun içindeki `.gitpoulpi/repo-state.toml` dosyasında tutulur.
- `gitpoulpi config get <key>` ve `gitpoulpi config set <key> <value>` her ayarı okur ve
  yazar.
- Ortam değişkenleri ve komut satırı seçenekleri dosyayı geçersiz kılar.

Ayrıntıları [yapılandırma başvurusu](config-reference.md) verir.
