[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · **Türkçe** · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Sorun giderme ve SSS

Aşağıdaki hızlı yanıtlarla başlayın; daha uzun bölümler Git LFS'yi ve grafik
çizimini kapsar. Hiçbiri işe yaramazsa GitHub deposunda işletim sisteminizi,
terminalinizi, `gitpoulpi --version` çıktısını ve ne yaptığınızı belirterek bir
issue açın.

## Bir şey başarısız olduğunda

Başarısız bir işlem açılır pencere ya da çerçeveli kart göstermez: bunu **durum çubuğu** kırmızıyla söyler, başlık ve nedenle (*hata: Fetch başarısız: Uzak depoya ulaşılamadı.*), ve başka bir mesaj onun yerini alana kadar orada kalır. Sağ ucundaki Etkinlik düğmesi henüz bakmadığınız hataları sayar. Kırmızı mesaja tıklayın ya da paletteki `Etkinlik Günlüğü: Son Hatayı Aç` komutunu çalıştırın; Etkinlik Günlüğü o girdide açılır. Sizin tetiklediğiniz her başarısızlık, yalnızca Git işlemleri değil, böyle çalışır:

- **Git işlemleri**: bir fetch, push, pull, checkout, merge, rebase, commit, stash...
- **Bir klonlama** (ve yeni bir depo oluşturma): "Klonlama başarısız".
- **GitHub veya GitLab'a bir çağrı**: seçicilerin ve Klonla iletişim kutusunun depo listeleri, bir pull request veya issue oluşturma, Entegrasyonlar sayfasında bir token'ı ya da SSH anahtarını denetleme: "Barındırma hizmetine yapılan istek başarısız".
- **Harici bir program**: düzenleyici, diff veya merge aracı, dosya yöneticisi, tarayıcı, terminal ya da başlatıcı, bir görsel düzenleyici, Ayarlar'daki *Dene* düğmeleri: "Harici program başarısız".
- **Bir YZ isteği** (bir commit mesajı, bir açıklama, bir çakışma önerisi...): "YZ isteği başarısız".
- **Yazılamayan bir dosya**: yerleşik düzenleyicinin Kaydet'i, merge düzenleyicisi, kaydedilemeyen bir ayar: "Kaydetme başarısız".

Kırmızı mesaj hangi görünüm öndeyse orada durum çubuğundadır: bir depo sekmesi, Ana sayfa, Radar, Çalışma alanları, Ayarlar (ekranın alt satırında), Etkinlik Günlüğü ya da bir iletişim kutusu altında. Durum çubuğunun örtüldüğü yerlerde (Ana sayfa, Radar, Çalışma alanları, Ayarlar) mesaj, hataya bakana kadar alt satırın üzerine yazılır. Bir sekmeden ötekine sizi izler. Sekme değiştirdiğinizde hâlâ çalışan bir işlem izlenmez: nasıl bittiğini görmek için Etkinlik Günlüğü'nü açın.

Yenilenemeyen PULL REQUESTS ve ISSUE'LAR bölümlerinin listeleri bunun yerine bölümde kendi satırını tutar (kendiliğinden yenilenir, birkaç saniyede bir mesaj yalnızca gürültü olurdu). Hâlâ açık olan bir iletişim kutusu (Klonla, Pull request oluştur...) da kendi tek satırlık mesajını tutar; böylece alanı düzeltip yeniden deneyebilirsiniz.

Etkinlik Günlüğü'ndeki girdi, açılır pencerenin eskiden gösterdiği her şeyi içerir:

- **Başlık ve neden.** "Fetch başarısız", sonra nedenini sade sözlerle söyleyen bir cümle, sonra sırada ne yapılacağı. Tanınanlar: uzak depoda sizde olmayan commit'ler var (*önce pull yapın*), kimlik bilgileri reddedildi (*Ayarlar > Entegrasyonlar'ı veya SSH anahtarınızı denetleyin*), çakışmalarda durma, üzerine yazılacak yerel değişiklikler (*commit'leyin, stash'leyin veya atın*), uzak depoya ulaşılamıyor ve depo üzerinde bir kilit tutan başka bir Git işlemi. Başka her şey Git'in kendi ilk satırını gösterir. Aynı şekilde başarısız olan bir klonlama veya barındırıcı çağrısı (kimlik bilgileri reddedildi, barındırıcıya ulaşılamıyor) aynı şekilde ifade edilir; diğer başarısızlıklar neyin başarısız olduğunu söyleyen bir cümle alır ("Depo klonlanamadı.", "Program çalıştırılamadı.").
- **Ayrıntılar**, girdi açıkken gösterilir (`Enter` ya da onu açan tıklama): işlem, çalışan git komutu, çıkış kodu ve yazdığı son satırlar (yerleşik git kitaplığı için hata sınıfı ve kodu). Kimlik bilgileri, token'lar ve `Authorization` başlıkları bir şey gösterilmeden, günlüğe yazılmadan veya kopyalanmadan önce maskelenir.
- **Kopyala** (düğme ya da `c`), panoya, açıklayabilecek biriyle ya da bir YZ ile yapılan bir sohbete yapıştırılmak üzere hazırlanmış bir metin bloğu koyar: GitPoulpi sürümü, sistem, terminal, backend, başarısız olan işlem ayrıntılarıyla ve git sürümü.
- Varsa **sonraki adım için bir düğme**, Kopyala'nın yanında (ya da `r`): *Önce Pull* (uzak deponun reddedeceği bir push), *Stash ve yeniden dene* (bir checkout, merge, rebase veya pull'un önünde duran yerel değişiklikler), *Yeniden dene* (uzak depoya ulaşılamadı), *Entegrasyonları aç* (kimlik bilgileri). Adımı çalıştırır ve pencereyi kapatır. Öndeki sekmeden başka bir deponun başarısızlığı için yalnızca *Entegrasyonları aç* sunulur: bir yeniden deneme yanlış depoda çalışırdı.
- Fare olmadan komut paletinde `Etkinlik Günlüğü: Son Hatayı Aç`, `Etkinlik Günlüğü: Son Hatayı Kopyala` ve `Etkinlik Günlüğü: Önerilen Eylemi Çalıştır` vardır (durum çubuğunun gösterdiği hata için girdinin çözüm düğmesi).

Uzak deponun ilerlediği için reddettiği bir push kendi iletişim kutusunu korur (önce pull, lease ile force push, iptal); olgular yine de Etkinlik Günlüğü'ndedir.

### Etkinlik Günlüğü

Durum çubuğunun sağ ucundaki **Etkinlik** düğmesi (üzerindeki sayı henüz bakmadığınız hataları sayar), `Ctrl+N` ardından `Shift+A` ve paletteki `Görünüm: Etkinlik Günlüğü` pencereyi açar. `Ctrl+N` pencerenin her sekmesinde, kutusunda, alanında, onay kutusunda ve düğmesinde (ve listenin ilk girdisinde) bir rozet gösterir; karakterine basmak ona tıklamakla aynı şeyi yapar. İki sekme vardır, **Git işlemleri** ve **Uygulama**; `Tab`, bir tıklama, `g` (Git işlemleri) veya `p` (Uygulama) ile geçilir ve altlarında ikisine de uygulanan bir **Kapsam** kutusu bulunur (aşağıya bakın):

- **Uygulama**: programın kendi günlük satırları, pencerenin ve arka plandaki daemon'ın, en yeni önce. **Düzey** kutusu (açılır liste: Hata, Uyarı, Bilgi, Hata ayıklama, İzleme; `l` açar) ne kadar gösterileceğini belirler; kaydedilenden daha fazla ayrıntı istemek kayıt düzeyini de yükseltir (ve `log.level` olarak kaydeder). Altında **Göster**, her düzey için bir onay kutusu içerir (Hata, Uyarı, Bilgi, Hata ayıklama, İzleme; her biri kaç satır tuttuğuyla) ve o düzeydeki satırları gizler veya gösterir, örneğin yalnızca hataları görmek için; **Tümü** / **Hiçbiri** tüm kutuları işaretler veya işaretlerini kaldırır. `1` ile `5` arası tuşlar düzeyleri bu sırayla açıp kapatır, `a` Tümü, `n` Hiçbiri'dir; palette `Etkinlik Günlüğü: ... düzeyini göster veya gizle`, `Etkinlik Günlüğü: tüm düzeyleri göster` ve `Etkinlik Günlüğü: hiçbir düzeyi gösterme` vardır. Düzey kutusunun seçiminin üstündeki düzeyler grileşir (onlar için hiçbir şey kaydedilmez; nedenini görmek için birinin üstüne gelin). İşaretler siz çıkana kadar hatırlanır ve yalnızca Uygulama sekmesinde bulunur. **Filtre** (`/`) metninizi içeren satırları tutar. **Tümünü kopyala (filtrelenmiş)** gösterileni kopyalar (işaretler ve filtre birlikte uygulanır); **Günlük klasörünü aç** klasörü dosya yöneticinizde açar. Uzun bir satır asla kesilmez: `Left`/`Right`, `Shift` + tekerlek veya yatay kaydırma çubuğuyla yana kaydırın (kaydırma tutamacını sürükleyin ya da tutamacı imlecin altına getirmek için rayına basın); `Home` başa döner.
![Etkinlik Günlüğü, Uygulama sekmesi: Düzey kutusu, Tümü ve Hiçbiri ile düzey başına Göster onay kutuları, filtre ve günlük satırları](../images/activity-log.png)

**Günlüğü temizle** (`x` veya `Etkinlik Günlüğü: Uygulama Günlüğünü Temizle` palet komutu) **Uygulama** sekmesindedir: önce sorar, sonra saklanan tüm günlerin günlük dosyalarını ve ekrandaki listeyi boşaltır. Geri alınamaz ve geri alma geçmişinde yer almaz.

- **Git işlemleri** (pencerenin açıldığı sekme): işlemler, en yeni önce; her biri çalıştığı saat ve ne kadar sürdüğüyle (`Push - 123 ms`): depoyu açma, commit grafiğini yükleme, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Başarısızlıklar, durum mesajları, başarısız bir klonlama ve bir çalışma alanının Fetch all işlemi de burada listelenir; `Enter` veya bir tıklama birini açar (nedeni, sonraki adımı, komutu, çıkış kodu ve çıktısı), `c` veya Kopyala düğmesi onu kopyalar, `r` veya düğmesi önerilen adımı çalıştırır. `Tab` (veya burada `Left`/`Right`) Uygulama sekmesine geçer.
![Etkinlik Günlüğü, Git işlemleri sekmesi: her işlem saati ve süresiyle, başarısız bir fetch kırmızıyla](../images/activity-log-repository.png)

- **Kapsam** (sekmelerin altında bir açılır liste, `s` açar ya da paletteki `Etkinlik Günlüğü: Kapsamı Seç`): iki sekmenin de neyle ilgili olduğu. Satırlar **Bu depo (ad)** (günlüğü açtığınız sekmenin deposu; deposu olmayan Ana sayfadan sunulmaz), **Tüm depolar**, her **çalışma alanı** ve pencerenin bildiği her depodur. Bir depo sekmesinden günlük *Bu depo* üzerinde, Ana sayfadan (Etkinlik düğmesi, `Ctrl+N` ardından `Shift+A`) *Tüm depolar* üzerinde açılır; `Ctrl+N` ardından `Shift+G` ve paletteki `Görünüm: Genel Etkinlik Günlüğü` her zaman *Tüm depolar* üzerinde açar; deposu öndeki sekme olmayan bir başarısızlığın kırmızı hatasına tıklamak da öyledir (örneğin başarısız bir klonlama). Son seçiminiz oturum boyunca, iki açış yolu için ayrı ayrı hatırlanır. Uygulama sekmesinde, program bir satırı bir depo hakkında yazdıysa (başarısız bir push, bir işlemin süresi) o satır o depoya aittir; *Tüm depolar* altında böyle her satır kimin olduğunu söyler (`[ad]`). Hiçbir depoyla ilgisi olmayan bir satır (başlangıç, ayarlar, daemon'ın kendisi) ve hiçbirine ait olmayan bir girdi (başarısız bir kayıt) **uygulama geneli**dir: her kapsamda görünür, kapsam *Tüm depolar*'dan dardıysa `[uygulama geneli]` ile işaretlenir; böylece bir filtre hiçbir başarısızlığı gizlemez. Palette sekmeler için ayrıca `Etkinlik Günlüğü: Git İşlemleri` ve `Etkinlik Günlüğü: Uygulama Günlüğü` vardır.

- **Tanılamayı kopyala** (iki sekmede de bir düğme, `d`, kapsam ne olursa olsun her zaman günlüğün tamamı; ve paletteki `Görünüm: Tanılamayı kopyala`) sürümü, sistemi, terminali ve backend'i, son başarısızlığı ayrıntılarıyla ve seçilen düzeydeki son 200 günlük satırını kopyalar: bir şeyler ters gittiğinde bir YZ'ye verilecek blok.

Süreleri pencere ölçer (isteğin gönderilmesinden daemon'ın yanıtına, gidiş dönüş dahil), yani CLI veya MCP'nin yaptığı şey zamanlanmaz. `log.timings` (varsayılan olarak açık) süreleri kapatır, `log.extended` (varsayılan olarak kapalı) arka plan işlerini, periyodik yenilemeyi ve otomatik fetch'i de listeler; ikisi de Ayarlar > Genel'de diğer günlük ayarlarının yanındadır ve her işlem zaten uygulama günlüğüne yazılır (istediğiniz şey için `info`, arka plan için `debug`).

Günlük dosyaları `gitpoulpi-YYYY-MM-DD.log` adındadır, varsayılan olarak bir hafta ve en çok on yıl saklanır (`log.retention_days`, 1 ile 3650 arası), GitPoulpi başlarken budanır ve `log.max_file_mb` (5 MiB) boyutunda döndürülür. Nerede oldukları: [yapılandırma başvurusu](config-reference.md#log-uygulama-günlüğü). Bir betikten, başarısız bir komutun `--json` hatası aynı olguları `detail` altında taşır; bkz. [CLI başvurusu](cli-reference.md).

Kırmızı mesaja tıklamak günlüğü *Tüm depolar* üzerinde o girdi açık olarak açar. Git işlemleri listesi oturuma aittir; Uygulama sekmesi programın günlüğünü yeniden başlatmalar boyunca tutar.

**Arka plan işleri.** Bir fetch, push, pull, klonlama veya LFS aktarımı, bir çalışma alanının Fetch all / Pull all / Clone missing işlemi, bir yapay zekâ isteği ve gönderilmekte olan bir pull request ya da sorun birer iştir: durum çubuğunun sağ ucu çalışanı, onu iptal eden bir `[x]` ile gösterir; etikete tıklamak (veya `Ctrl+N` ardından `Shift+J`, `Görevler: Arka Plan Görevlerini Göster` komutu) her iş için bir İptal düğmesi olan paneli açar (`c` veya `Del` de olur). Klonlama arka planda çalışır; yapay zekâ isteği ve çalışma alanı taraması sekme değişse de sürer. Durdurmanın güvenli olmadığı noktayı geçen iş nedeniyle birlikte gri görünür; iptal edilen iş depoyu olduğu gibi bırakır.

## Hızlı yanıtlar

**Araç çubuğunda ve kenar çubuğunda simgeler yerine kutular veya soru
işaretleri görünüyor.** Terminalinizde bir Nerd Font veya emoji desteği olan
herhangi bir yazı tipi kullanın. Grafik ve metin onlar olmadan da çalışır.

**Commit grafiği resimlerden değil karakterlerden oluşuyor.** Bu otomatik
yedektir: resimler için kitty grafik protokolünü (kitty, Ghostty, kitty
grafikleri açık herdr) veya iTerm2 protokolünü (WezTerm, iTerm2, mintty) veya sixel (Windows Terminal, foot, Konsole)
konuşan bir terminal gerekir. Windows konsolu, tmux ve zellij
karakterlerle çizer. İkisinden birini zorlamak için aşağıdaki grafik bölümüne
bakın; ayrıca bkz. [Terminal seçmek](terminals.md).

**Fare hiçbir şey yapmıyor veya fareyle metin seçmek çalışmıyor.** GitPoulpi
tıklamaların kendisine ulaşması için fareyi yakalar. Metni terminalin kendisiyle
seçmek için sürüklerken `Shift` tuşunu basılı tutun (çoğu terminalde). Diff'in
içinde de seçebilirsiniz (sürükleyin, sonra `y`). Fare yakalamayı kapatmak için
*Ayarlar > Klavye ve fare > Fare açık* kullanın (yeniden başlatma gerekir).
Klavye her komuta ulaşır.

**Bir kısayol çalışmıyor.** Terminaliniz, tmux veya bir AutoHotkey betiği tuşu
GitPoulpi görmeden önce alıyor olabilir (`Ctrl+,` Linux ve macOS terminallerinde
sık sık düz bir `,` olarak gelir; `Ctrl+Shift+N`, `Ctrl+N` gibi görünür).
Alternatifleri kullanın (`Ctrl+N` sonra `,` Ayarlar'ı açar; `Ctrl+P` her komuta
ulaşır) ya da akoru yeniden atayın: bkz. [Ayarlar](settings.md#klavye-ve-fare).

**Bir kısayolu hatırlamıyorum.** `Ctrl+P` tuşuna basıp istediğinizi yazın; her
sonuç kısayolunu gösterir. `?` her şeyi listeler.

**Ayarlar'da bir seçenek grileşmiş.** Başka bir seçeneğe bağlıdır. Satır hangisi
olduğunu söyler (örneğin *Satır tonu gücü*, *Grafik şerit arka planı*'nın
*Satır tonu* olarak ayarlanmasını ister).

**Log'un üstünde "Otomatik stash bekliyor".** GitPoulpi bir işlemi çalıştırmak
için değişikliklerinizi stash'ledi ve geri koyamadı. Banner'ın **Şimdi pop et**,
**Stash göster**, **Baştan başla** (işlemi geri alıp değişikliklerinizi geri
koyar) veya **At** düğmesini kullanın; bkz. [Otomatik
stash](log-and-history.md#otomatik-stash).

**Bir ajan `no_viewer` veya `deferred` diyor.** `no_viewer`: o depo için açık
bir GitPoulpi penceresi yok (ya da *YZ ajanlarının görünümü yönetmesine izin
ver* kapalı). `deferred`: pencere bir çip gösteriyor; `Ctrl+N Enter` onu kabul
eder. Bkz. [MCP ve
ajanlar](mcp-and-agents.md#kullanıcıya-bir-şeyler-göstermek).

**GitPoulpi'yi açtım ve boş bir pencere veya `log error: backend error`
görüyorum.** Onu bir Git deposunun içinde başlattığınızdan emin olun (aksi
halde Ana sekmeye düşersiniz: **Gözat**, **Klonla** veya **Init**
kullanın). Sonra `gitpoulpi daemon status` çalıştırın; `gitpoulpi daemon
restart` bu depo için arka plan sürecini yeniden başlatır. Yenilemek için `r`
tuşuna basın.

**Aynı depodaki iki pencere tuhaf davranıyor.** Bilerek depo başına tek bir
daemon paylaşırlar; böylece birindeki bir değişiklik ötekinde görünür. Bir
daemon takılmış görünüyorsa `gitpoulpi daemon stop --all` her oturumun
daemon'ını durdurur; sonraki pencere yenisini başlatır.

**Bir commit, pull veya push parola istiyor ya da kimlik doğrulayamıyor.**
GitPoulpi ağ için `git`'inizin kendi kimlik bilgisi yardımcısını ve ssh-agent'ı
kullanır. Önce düz bir terminalde `git fetch` çalıştığından emin olun. Pull
request'ler için GitHub/GitLab token'ları ayrı ve isteğe bağlıdır: bkz.
[Barındırma hizmetleri ve issue takipçileri](hosting-integration.md).

**Durum çubuğunda "stage'leniyor..." kalıyor.** Bir arka plan yenilemesi hâlâ
çalışıyor; dosya listesi güncellenince kaybolur. `r` yenilemeyi zorlar.

**Çok büyük bir depo yavaş.** Ayarlar > Log ve grafik sayfasında *Grafikteki
İlk Commit'ler* değerini düşürün (*Grafikte Commit'leri Tembel Yükle* açıkken
Log, kaydırdıkça daha fazlasını yükler); Log'un *Değişim* sütununu gizleyin ve
avatarları kapatın; geri almaya ihtiyacınız yoksa *Geri alma açık* ayarını
kapatın (stage'lemede ve commit'lerde hiç kayıt tutulmaz).
`GITPOULPI_TIMING=1`, istek başına süreleri geçici klasördeki
`gitpoulpi-timing.log` dosyasına yazar; bir performans raporuna eklemeye
değer.

**YZ düğmeleri yok veya gri.** YZ varsayılan olarak kapalıdır; bkz. [YZ
desteği](ai.md#sorun-giderme).

**GitPoulpi kendiliğinden kapandı.** Her çıkış kaydedilir: yapılandırma
klasöründe `gitpoulpi-tui-exit.log` dosyasına (ve bir çökmeden sonra
`gitpoulpi-tui-panic.txt` dosyasına) bakın (`gitpoulpi config path` ayar
dosyasını yazdırır, o dosya oradadır) ve onu bir issue'ya ekleyin.

**Ayarlarım nerede?** `gitpoulpi config path`. Depo başına durum, deponun
içindeki `.gitpoulpi/repo-state.toml` dosyasındadır.

**GitPoulpi bir yere bir şey gönderiyor mu?** Yalnızca istediğiniz şeyi: uzak
depolarınıza git işlemleri, bir hesap bağladıktan sonra barındırma istekleri, bir
YZ düğmesine bastıktan sonra YZ istekleri ve avatarları açarsanız yazar
e-postalarının Gravatar'a bir hash'i. Telemetri yoktur. Bkz. [YZ
desteği](ai.md#gizlilik).

## Her yazma başarısız oluyor: "Unable to create index.lock"

Ölen bir git komutu (bir çökme, öldürülmüş bir terminal) geride
`.git/index.lock` bırakabilir ve o zaman depoya yapılan her değişiklik başarısız
olur. Dosya 10 saniyeden eski olduğunda GitPoulpi Log'un üstünde bir banner
gösterir: **index.lock dosyasını sil** (palet: *Depo: Eski index.lock Dosyasını
Sil*; CLI: `gitpoulpi index-lock remove`). Önce sorar ve yalnızca o dosyayı
siler. GitPoulpi başka bir git programının hâlâ çalışıp çalışmadığını güvenilir
biçimde söyleyemez; bu yüzden onaylamadan önce hiçbirinin çalışmadığından
(bir IDE, başka bir terminal) emin olun. Bağlı bir worktree'de kilit, worktree'nin
kendisinindir ve `.git/worktrees/<ad>` içindedir.

## "git am" durdu ve Rebase Abort hiçbir şey yapmıyor

Bir `am` (bir posta kutusundan yama uygulama) bir rebase değildir. Duran bir
`git am`'in kendi banner'ı vardır, **Yamalar uygulanıyor (git am)**; Çakışmaları
çöz, Devam et, Atla ve İptal et düğmeleriyle (palet: *git am: Devam et / Bu
yamayı atla / İptal et*; CLI: `conflicts am-continue|am-skip|am-abort`).
İptal et önce sorar, şimdiye kadar uygulanan yamaları atar ve dalı önceki yerine
koyar. Bunlar, rebase düğmeleri gibi, geri alınamaz.

## Geri al grileşmiş ve "... geri alınamaz" diyor (engellenmiş girdi)

En yeni günlük girdisi takıldı: depo GitPoulpi dışında ilerledi veya girdi
yarıda kesildi. Nedeni için Geri al'ın üzerine gelin; kalıcı olarak takılıysa
araç ipucu **Düzenle: Engellenen geri alma girdisini unut** (palet) veya
`gitpoulpi undo-forget` komutuna işaret eder. Önce sorar ve girdiyi yalnızca
günlükten çıkarır; depoya dokunulmaz. Bkz. [Geri al ve yinele](undo-redo.md).

## Büyük dosyalar üç satırlık metin olarak görünüyor ("LFS dosyaları işaretçi olarak görünüyor")

Bir resim, bir model veya büyük bir ikili dosya olması gereken bir dosya
`version https://git-lfs.github.com/spec/v1` ve bir `oid sha256:...` satırı
içerir: bu bir Git LFS **işaretçisidir** ve gerçek içerik indirilmemiştir (veya
LFS filtreleri kurulu değildir).

- Durum paneline bakın: `git-lfs` eksikken LFS kullanan bir depo kehribar renkli bir banner gösterir. Programı kurun (https://git-lfs.com; Windows: Git for Windows ile birlikte gelir veya `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) ve yeniden açın ya da `r` tuşuna basın.
- `gitpoulpi lfs status` programın kurulu olup olmadığını, filtrelerin ve hook'un kurulu olup olmadığını ve hangi desenlerin izlendiğini söyler; `gitpoulpi lfs files` işaretçi dosyalarını ve hangilerinin `local` olduğunu listeler.
- `git-lfs` kuruluyken paletten **`LFS: Nesneleri Pull Et`** komutunu (veya `gitpoulpi lfs pull`) çalıştırın: nesneleri indirir ve işaretçilerin yerine koyar. Hiç yoksa önce bu depo için LFS filtrelerini kurar.
- İndirme başarısız olursa mesaj nedenini adlandırır (uzak depo için kimlik bilgileri, artık nesneye sahip olmayan bir sunucu). GitPoulpi'nin kendi hesabına ihtiyacı yoktur: LFS uzak deponuzun olağan kimlik bilgilerini (kimlik bilgisi yardımcısı veya ssh-agent) kullanır.
- Bir LFS deposunun klonu LFS yüzünden asla başarısız olmaz; indirme adımı çalışamadıysa klon işaretçilerle birlikte oradadır ve `LFS: Nesneleri Pull Et` bunu sonra tamamlar. `gitpoulpi clone --no-lfs` indirmeyi bilerek atlar.
- Ayarlar > Git ve Kodlama > Git LFS içinden (veya `[git] lfs_enabled = false`) LFS ile ilgili her şeyi kapatabilirsiniz.

## "git-lfs not found" / "Git LFS is not installed"

`Git LFS kurulu değil. Bu depo büyük dosyaları Git LFS ile saklıyor; ...` mesajı (veya *Kurulum sayfasını aç* düğmeli bir iletişim kutusu ya da Git'in kendi `git-lfs: command not found` / `'lfs' is not a git command` çıktısı) deponun LFS için yapılandırıldığı - `.gitattributes`, `.git/config` filtreleri veya `pre-push` hook'u `git-lfs` çağırıyor - ve programın GitPoulpi'nin gördüğü `PATH` üzerinde olmadığı anlamına gelir. Kurun (yukarıdaki bağlantı). Windows'ta program Git for Windows ile birlikte gelir; bir terminalde `git lfs version` çalışıyor ama GitPoulpi yine de eksik diyorsa GitPoulpi'yi aynı `PATH`'i gören bir terminalden başlatın. Git'in kendi davranışına dokunmadan LFS'yi GitPoulpi'den gizlemek için `git.lfs_enabled` ayarını kapatın.

## Grafik resim olarak değil karakterlerle çiziliyor (herdr, tmux, diğer terminaller)

GitPoulpi Grafik sütununu ve yazar avatarlarını yalnızca terminalin göstereceğinden emin olduğu yerde piksel olarak çizer: yalın WezTerm, iTerm2 ve mintty'de iTerm2 satır içi resimleri; kitty ve Ghostty'de ve **herdr'in kendi yapılandırması `kitty_graphics = true` ayarladığında herdr'de** (`[terminal]` veya eski herdr'de `[experimental]`) ve herdr WezTerm, kitty ya da Ghostty içinde çalışırken Kitty grafik protokolü. Sixel resimleri Windows Terminal, foot, Konsole ve mlterm'de kullanılır. Başka her yerde karakterlerle çizer; bu her zaman doğrudur.

- Kitty grafikleri açık ama GitPoulpi'nin bulamadığı bir yapılandırmaya sahip (ya da herdr'in varsayılanına güvenen) herdr'de `ui.image_protocol = "kitty"` ayarlayın (Ayarlar > Log ve grafik > Görüntü protokolü) ve yeniden başlatın.
- Grafiğin olması gereken yerde kutular veya anlamsız karakterler gösteren bir terminalde `ui.image_protocol = "none"` veya `ui.graph_renderer = "text"` ayarlayın.
- `GITPOULPI_TERMINAL_IMAGE=off`, yapılandırmadan bağımsız olarak resimleri kapatır.

**herdr uzak (`herdr --remote <host>` veya `ssh` sonra `herdr`): varsayılan metin grafiğidir.** SSH üzerinden GitPoulpi uzak makinede çalışır ve yalnızca uzak makinenin ortamını ve herdr `config.toml` dosyasını okuyabilir. Bir Kitty resminin görünüp görünmeyeceğine *yerel* herdr istemcisi (kendi `kitty_graphics` ayarı ve kendi terminali) karar verir ve GitPoulpi bunu göremez: istemci resimleri çizmediğinde hücreler boş kalır. Bu yüzden GitPoulpi SSH üzerinden herdr algıladığında (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`) metin grafiğini çizer. Yerel herdr'inizde `kitty_graphics = true` ve Kitty resimleri çizen bir terminal (kitty, Ghostty) varsa uzak makinede `ui.graph_renderer = "image"` (veya `ui.image_protocol = "kitty"`) ile bunu tercih edin. Resimler satır içi gönderilir (`t=d`); bu her bayt akışı üzerinde çalışır. Sütun boşsa `ui.graph_renderer = "text"` ayarlayın.

**Şeritlerin arkasında daha açık bir dikdörtgen.** Resim, hiçbir şey çizilmeyen her yerde saydamdır (straight-alpha RGBA PNG); bu yüzden terminalin arka planı ve seçim bandı görünür. Kitty protokollü terminaller ve **WezTerm** bu alfayı birleştirir; GitPoulpi o zaman resmin altında sıradan boş hücreler ayırır ve resmi onlardan sonra gönderir. Diğer her iTerm2 protokollü terminalin (iTerm2'nin kendisi, Rio, mintty) PNG alfasını harmanladığı bilinmez; bu yüzden orada resim temanın renginde opak bir arka plan taşır ve bu, arka planı ondan farklı olan bir terminalde dikdörtgen olarak görünebilir; `ui.graph_renderer = "text"` onu kaldırır. `ui.graph_lane_background = "off"` ile saydam yollarda şeritlerin arkasına hiçbir şey boyanmaz.

[Yapılandırma başvurusunda](config-reference.md) `ui.graph_renderer` ve `ui.image_protocol` öğelerine bakın.
