[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · **Türkçe** · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Konuşmalar

Bir issue, iş öğesi, kart veya pull / merge request'in konuşma görünümü. Bir hesap ve bağlı bir depo gerekir: bkz. [Barındırma ve issue takipçisi entegrasyonları](hosting-integration.md); onu açan listeler [Pull request'ler ve issue'lar](pull-requests-and-issues.md) sayfasındadır; kodu incelemek [Bir pull veya merge request'i incelemek](review.md) sayfasındadır.

Bir **konuşma**, bir issue, iş öğesi, kart, değişiklik veya pull / merge request hakkında söylenen her şeydir:
açıklama, yorumlar, incelemeler, koddaki yorum dizileri ve olaylar (etiketler, atamalar, durum değişiklikleri,
push'lar). GitPoulpi onu barındırıcıdan okur, biçimlendirilmiş gösterir ve pencereden çıkmadan yanıt vermenizi sağlar.

### Birini açmak

| Nereden | Nasıl |
|---|---|
| Bir ISSUE'LAR satırı | `Enter` veya çift tık; satır menüsündeki **Konuşmayı aç**; palet *Barındırma: Issue Konuşmasını Aç...* (ekrandaki issue'ların bir listesi). Satır menüsündeki **Tarayıcıda aç** tarayıcıyı korur. |
| Azure DevOps veya Bitbucket'ta bir PULL REQUEST'LER satırı | `Enter` veya **Konuşmayı aç**. GitHub, GitLab, Gitea, Forgejo ve Gerrit'te satır, Genel Bakış sayfası aynı bu konuşma olan [incelemeyi](review.md) açar (zaman çizelgesi, tuşlar ve aşağıdaki yorum kutusu). |
| [Issue panosunun](pull-requests-and-issues.md#issue-panosu) bir kartı | `Enter` veya çift tık. |
| [Radar](radar.md) | Bir issue veya istek satırında `i`, satır menüsündeki **Konuşmayı aç** veya *Radar: Issue Konuşmasını Aç*. |
| Başka bir konuşmadaki bir referans | Üzerine tıklayın (veya `l`, sonra seçin): bağlı bir kaynağın `#7`, `!12`, `ABC-12` referansları uygulamada açılır ve bir Geri yığınına eklenir. |
| Bir ajan | `gitpoulpi ui show issue <n> --view conversation` (ayrıca `ui show pr`), bkz. [CLI başvurusu](cli-reference.md). |

**Klonlanmamış depolar.** Klonu olmayan bir deponun Radar satırı da konuşmasını açar: görünüm yalnızca entegrasyon
profilini ve projeyi ister, bu yüzden Radar onu kendi penceresinin üstünde barındırır ve `Esc` Radar'ı geri verir.
Klon gerektiren şeyler dışarıda kalır: bir commit referansı Log'da değil barındırıcıda açılır ve *YZ depoyu
okuyabilsin* nedeniyle birlikte gri olur ([YZ bölümüne](ai.md#sunucuda-yazma-yardımı) bakın). Klonlanmış bir deponun
isteği bunun yerine incelemesini açar (`e`) ve Radar'ın **Konuşmayı aç** eylemi onun için burada okunacak bir konuşma
olmadığını söyler; klonlanmış bir deponun issue'su konuşmasını o deponun sekmesinde açar.

Okunacak konuşması olmayan bir barındırıcının issue'su (Bitbucket ve Gerrit'te issue yoktur) eskisi gibi tarayıcıyı
açar; `conversation` özelliği olmayan bir derleme de her yerde aynısını yapar ([özel derlemeler](custom-builds.md)).

### Görünüm

Tam pencere bir görünüm: başlık (durum, kimin ne zaman açtığı, etiketler, yorum sayısı, **Olayları göster**, **Geri**,
**Yenile**, **Tarayıcıda aç**), **... olarak gönderiliyor** satırı, zaman çizelgesi ve altında yorum kutusu.

![#7 numaralı issue'nun konuşması: #9'a bağlantı içeren açıklama, iki yorum, Olayları göster ile görünen bir etiket olayı ve Yaz, Önizleme ile Yan yana'lı boş Yeni yorum kutusu](../images/issue-conversation.png)

![!58 numaralı merge request'in Genel Bakış sayfası: açıklama, dosya ve satır bağlantılı çözülmemiş ve çözülmüş birer kod yorum dizisi, yorumlar, seçili dizide Yanıtla ve Alıntıla ve yorum kutusu](../images/pr-conversation.png)

- Zaman çizelgesi eskiden yeniye okunur ve en yeni pencereyle başlar (en çok 300 kayıt). En üstündeki **Eskileri
  yükle** (`m`, palet *Konuşma: Eski Kayıtları Yükle*) öncesindeki pencereyi okur; barındırıcı söylerse kalan sayıyı
  da gösterir.
- Bir yorum yazarını, tarihini, bir `düzenlendi` çipini, botlar için bir `bot` çipini, özel notlar için `dahili` ve
  seçili kaydın izin verdiği **Yanıtla**, **Alıntıla**, **Düzenle**, **Sil** ve **Bağlantı** düğmelerini gösterir.
  Koddaki bir yorum dizisi `path:line`, `çözüldü` veya `çözülmedi`, `eskimiş` ve yorumlarını gösterir. Bir inceleme
  kararını gösterir.
- Olaylar, her biri soluk tek satırdır (*carol lfs etiketini ekledi*). **Olayları göster** (`v`) onları gizler; seçim
  depo başına saklanır. Olay geçmişi olmayan barındırıcılarda (YouTrack, Azure DevOps iş öğeleri, yerel notlar) böyle
  bir onay kutusu yoktur.
- Metin çizilmiş Markdown'dır: başlıklar, iç içe ve görev listeleri, alıntılar, GitHub tarzı uyarı blokları, vurgulanmış
  kod, tablolar, bağlantılar, `[image: alt]` olarak görseller. Barındırıcının kendi işaretlemesiyle gelen metin önce
  dönüştürülür ([İşaretleme dönüştürme](#işaretleme-dönüştürme) bölümüne bakın). Bir `mermaid` kod bloğu, diyagramın
  kaynağını kaydırılabilir bir görüntüleyicide gösteren bir **Diyagramı aç** düğmesi alır (`Enter` onu kopyalar); paletin
  *Konuşma: Kaydın Diyagramını Aç* satırı da vardır. Hem `conversation` hem `mermaid` içeren bir derleme gerekir.
- İncelemenin Genel Bakış sayfasında koda ait bir dizinin `path:line` değeri koda giden bir bağlantıdır (`G`, *Konuşma: Dizinin
  Kodunu Aç*; bkz. [İnceleme](review.md)); tek başına açılmış bir konuşmada (bir issue, bir Radar satırı) gidilecek kod sayfası yoktur.
- Görünüm her 60 saniyede bir ve `F5` ile (**Yenile**) yeniden okur.

**Tuşlar.** Her birinin bir düğmesi, bir menü satırı veya bir palet satırı (*Konuşma: ...*) vardır ve `Ctrl+N`
düğmelere rozet koyar.

| Tuş | Eylem |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Önceki / sonraki kayıt |
| `PageUp`, `PageDown`, `Home`, `End`, tekerlek, kaydırma çubuğu | Kaydır |
| `Tab`, `Shift+Tab` | Zaman çizelgesi, yorum kutusu, sonra başlık düğmeleri |
| `r` | Yanıtla: seçili kaydın yorum dizisine veya yorum dizisi olmayan barındırıcılarda metni alıntılanarak |
| `Q` | Seçili kaydı yeni bir yorumda alıntıla |
| `c` | Yeni yorum |
| `e`, `d` | Kendi yorumlarınızdan birini düzenle, sil (silmeden önce bir soru gelir) |
| `l`, `Enter` | Kaydın bağlantıları: `Enter` tek olanı izler veya onların menüsünü açar |
| `y`, `o` | Kaydın adresini kopyala, tarayıcıda aç |
| `n` | Sonraki okunmamış kayıt |
| `v`, `m`, `F5` | Olayları göster veya gizle, Eskileri yükle, Yenile |
| `Alt+Left`, `Backspace` | Geri yığınında geri git |
| `q`, `Esc` | Kapat (kutuda metin varsa önce sorar) |

### Yazma

**Yeni yorum** (`c` veya kutuya `Tab`), **Yanıtla** (`r`), **Alıntıla** (`Q`) ve **Düzenle** (`e`) tek bir Markdown
yazma kutusunu kullanır: **Yaz**, **Önizleme** ve **Yan yana** (`Alt+P` Yaz ile Önizleme'yi değiştirir, `Alt+L`
Yan yana'yı açar ve 100 sütun genişliğinde bir pencere ister). `Ctrl+Enter` (ayrıca `Ctrl+S` ve `Alt+Enter`) veya
**Gönder** gönderir, **İptal** kutuyu bırakır ve `Alt+E`, YZ açıkken **YZ ile Yaz**'ı açar ([YZ
bölümü](ai.md#sunucuda-yazma-yardımı)). Önizleme, zaman çizelgesiyle aynı çizim motoruyla ve aynı referanslarla
düzenlenir. Barındırıcı Markdown kabul etmiyorsa alt bilgi metnin hangi işaretlemeyle gönderildiğini söyler (*Jira wiki
markup olarak gönderilir*, *Textile olarak gönderilir*).

![Yan yana görünümünde yazılan bir issue yanıtı: solda Markdown taslağı, sağda işlenmiş önizlemesi ve Gönder](../images/issue-reply-composer.png)

- Yorum zaman çizelgesine hemen eklenir ve barındırıcı yanıtlayınca yerini alır. Barındırıcı reddederse kayıt kalkar ve
  kutu metniniz ve nedenle yeniden açılır.
- Göndermediğiniz bir metin öğe başına saklanır (yapılandırma klasöründeki `remote-conversations.toml`) ve o kutuyu
  bir sonraki açışınızda orada durur; göndermek veya vazgeçmek onu kaldırır. Metin içeren bir kutuyu kapatmak önce sorar.
- Bir yorum dizisinde **Yanıtla** (bir inceleme dizisi, bir GitLab tartışması, bir Linear yorumu, bir Bitbucket veya
  Azure DevOps dizisi) o diziye gönderir. Barındırıcıda yalnızca düz bir liste varsa **Yanıtla**, seçili kaydı
  alıntılayarak (`> **yazar**`) yeni bir yorum başlatır; **Alıntıla** ile aynıdır.
- **Dahili not** (onay kutusu, `Alt+I`): GitLab ve Redmine'da yorum yalnızca proje üyelerince görülür. Yalnızca böyle
  notları olan barındırıcılarda ve yalnızca yeni bir yorumda gösterilir.
- Göndermek dışa dönük bir eylemdir: hiçbir şey günlüğe yazılmaz ve **GitPoulpi'den hiçbir şey geri alınamaz**. Bağlantı
  yoksa kutu durur, **Gönder** nedeniyle birlikte gri olur ve başlık kaydedilmiş kopyanın gösterildiğini söyler.

**Kimler düzenleyebilir veya silebilir.** Yalnızca kendi yorumlarınızda **Düzenle** ve **Sil** bulunur. Barındırıcı
verirse önce onun kendi sözü gelir (GitHub'ın `viewerCanUpdate` ve `viewerCanDelete` değerleri, başka yerde
`can_edit`), sonra oturum adınız yorumun yazarıyla büyük/küçük harf gözetmeden karşılaştırılır; bir App botunun
yorumları ayrıca bir bot hesabından gelmelidir. Bir GitHub App botunun oturum adı bilinmediği sürece yalnızca bu kurulumun
gönderdiği yorumlar sayılır (depo kimliklerini tutar): hiçbir şey bir addan tahmin edilmez. Bir yorum dizisinde
**Düzenle** ve **Sil**, dizinin sizin olan son yorumuna uygulanır. Düzenleyemeyen veya silemeyen bir barındırıcı
([tablo](#her-barındırıcının-yapamadıkları)) düğmeleri nedeniyle birlikte gri gösterir.

### Şu adla gönderme

Başlığın altındaki satır mesajlarınızın kim adına gönderileceğini söyler ve aynı satır her kutudadır:

| Satır | Anlamı |
|---|---|
| *ann olarak gönderiliyor (kişisel token)* | Profilin token'ı: kendi oturum adınız (`GET /user`). |
| *ci-bot olarak gönderiliyor (depo token komutu)* | Deponun [token komutu](hosting-integration.md#8-github-uygulaması-botu-olarak-çalışmak) barındırıcı API'si için kullanılır. |
| *my-app[bot] olarak gönderiliyor (GitHub Uygulaması botu)* | Bir kurulum token'ı. Ad **Şu adla gönder**'den, yoksa barındırıcıdan gelir. |
| *GitHub Uygulaması botu olarak gönderiliyor (adı bilinmiyor: Ayarlar'da Şu adla gönder'i ayarlayın)* | Yapılandırılmış ad yok, henüz öğrenilen de yok. |
| *... olarak gönderiliyor (proje botu)* | Bir GitLab proje veya grup erişim token'ı. |
| *kai olarak yazılıyor (yerel not)* | Markdown notları klasörü: profilin `local.me` değeri, yoksa bağlı kullanıcı. |
| *Gönderim adınız henüz bilinmiyor. Ayarlar'da Şu adla gönder'i ayarlayın.* | Hiçbir şey öğrenilemedi. |

**Şu adla gönder** bir addır: bir GitHub Uygulaması için `my-app[bot]`. Onu her depo için **Ayarlar > Entegrasyonlar >
Şu adla gönder**'de (`general.hosting_post_as`), ya da tek bir depo için, onu geçersiz kılan **Ayarlar > Depoya özel
ayarlar > SSH ve kimlik bilgileri > Sunucu**'da ayarlayın. Boş bırakılırsa GitPoulpi GitHub'a sorar (GraphQL `viewer {
login }`; bir kurulum token'ı bunu yanıtlayabilir de yanıtlamayabilir de) ve olmazsa adı gönderdiği ilk yorumun
yazarından öğrenir ve depoyla birlikte saklar. Kişisel bir token buna hiç ihtiyaç duymaz. Bkz.
[GitHub Uygulaması botu olarak çalışmak](hosting-integration.md#8-github-uygulaması-botu-olarak-çalışmak).

### Çapraz referanslar

Bir konuşmadaki referanslar altı çizili ve tıklanabilirdir. Deponun bağlı ISSUE'LAR ve PULL REQUEST'LER kaynakları
önce eşleştirilir; bu yüzden bağlı bir Jira projesinin `ABC-12` referansı bir GitHub yorumunun içinde bile
uygulamada açılır; barındırıcının kendi dilbilgisinin eklediği şeyler, bağlantı onayından sonra kendi sayfasını açar.
Bir commit kimliği, depo klonlanmışsa ve kimlik en az 7 karakterse Log'da, değilse barındırıcıda açılır. Bir kod
aralığının, çitli bir bloğun, bir Markdown bağlantısının, bir e-posta adresinin veya bir URL'nin içindeki hiçbir şey
referans değildir; `C#1`, `UTF-8`, `SHA-256` ve `ISO-8601` de değildir.

| Barındırıcı | Tanınanlar |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, commit kimlikleri |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (kardeş bir proje), `@user`, çip olarak `~label` ve `%milestone`, commit kimlikleri |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, commit kimlikleri |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, `{uuid}` anmaları, commit kimlikleri |
| Gerrit | `Change-Id` değerleri (`I` ve 40 onaltılık hane), commit kimlikleri |
| Redmine | `#7`, `r123` revizyonları, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, yalnızca bağlı bir kaynağın projesi veya takımı için |
| Trello | kart adresleri (`trello.com/c/...`) |
| Yerel notlar | `[[note]]`, `[[note\|alias]]` ve klasördeki başka bir nota göreli `.md` bağlantıları |

`gitpoulpi hosting markdown --body T --id I` uygulamanın bir metinden, referanslar dahil, ne yaptığını yazdırır.

### İşaretleme dönüştürme

Markdown konuşmayan barındırıcılar iki yönde dönüştürülür, böylece her yerde Markdown okur ve yazarsınız. Bir
dönüştürücünün anlamadığı şey okunabilir metin olarak kalır.

- **Jira** (Cloud ve Data Center) yorumları API v2 üzerinden wiki işaretlemesi olarak saklar. Okuduğunuz Markdown'a
  dönüştürülür; yazdığınız wiki işaretlemesine dönüştürülür ve alt bilgi *Jira wiki markup olarak gönderilir* der.
- **Redmine** metni Textile veya Markdown olarak biçimlendirir; API bunu söylemez: profilin **Sunucunun metin biçimi**
  satırı (Ayarlar > Entegrasyonlar, `markup`, varsayılan Textile) GitPoulpi'ye bildirir. Textile okumak için
  Markdown'a, yazmak için yeniden Textile'a dönüştürülür.
- **Azure DevOps Server** iş öğesi yorumlarını HTML olarak döndürür ve HTML olarak alır: bunlar ondan dönüştürülmüş
  Markdown olarak gösterilir ve HTML olarak yazılır. Azure DevOps Services ve pull request yorum dizileri Markdown alır.
- **Gerrit** düz metindir: Markdown olarak çizilmez ve yazdığınız olduğu gibi gönderilir.
- Her içe aktarıcı önce denetim ve kaçış dizilerini kaldırır, böylece bir yorum terminali hareket ettiremez.

### Kaydedilmiş kopya, Eskileri yükle, okunmamış

- Açtığınız her konuşma kaydedilir (`<state dir>/conversations/<integration>/<project>/<kind>-<n>.json`, yalnızca
  kullanıcıya açık, içinde token yok, dosya başına en çok 2 MB, en son 50 dosya tutulur). Öğe açılınca hemen çizilir ve
  arkasından yenilenir. Barındırıcıya ulaşılamazsa kaydedilmiş kopya *Çevrimdışı: 3 dk önce kaydedilen kopya
  gösteriliyor (neden)* altında ekranda kalır ve Gönder gri olur.
- **Okunmamış:** en son gördükleriniz konuşma başına hatırlanır (son 200 tutulur). Başkasının daha yeni kayıtları bir
  vurgu çubuğu ve bir `yeni` çipi alır ve `n` bir sonrakine atlar. İlk kez açtığınız bir konuşmada yeni bir şey yoktur.

### Her barındırıcının yapamadıkları

Aşağıdaki her `hayır` gerçek bir sınırdır: denetim gizlenir veya nedeniyle birlikte gri olur ve CLI, hiçbir şey
gönderilmeden önce barındırıcıyı belirten bir hata yanıtlar.

| Barındırıcı | Öğeler | Yorum dizisinde yanıt | Kendininkini düzenle ve sil | Bir yorum dizisini çöz | Olaylar | Dahili notlar | Yazılış biçimi |
|---|---|---|---|---|---|---|---|
| GitHub | issue'lar, pull request'ler | pull request'ler | ikisi de | pull request'ler | evet | hayır | GitHub Markdown |
| GitLab | issue'lar, merge request'ler | evet (tartışmalar) | ikisi de | merge request'ler | evet | evet | GitLab Markdown |
| Gitea | issue'lar, pull request'ler | pull request'ler | ikisi de | pull request'ler | evet | hayır | Markdown |
| Forgejo | issue'lar, pull request'ler | pull request'ler | ikisi de | hayır (barındırıcıda uç nokta yok) | evet | hayır | Markdown |
| Azure DevOps | iş öğeleri, pull request'ler | pull request'ler | ikisi de | hayır (durum gösterilir, değiştirilmez) | yalnızca pull request'ler | hayır | Markdown (Server iş öğelerinde HTML) |
| Bitbucket (Cloud, Data Center) | yalnızca pull request'ler | evet | ikisi de | hayır (gösterilir, değiştirilmez) | evet | hayır | Markdown |
| Gerrit | yalnızca değişiklikler | evet | hayır (hiçbiri düzenlenemez veya silinemez) | evet | evet | hayır | düz metin |
| Jira | issue'lar | hayır (düz liste) | ikisi de | hayır | evet (değişiklik günlüğü) | hayır (bir proje rolü gerekir) | wiki işaretlemesi |
| Linear | issue'lar | evet (bir yorumun yanıtları) | ikisi de | hayır | evet | hayır | Markdown |
| YouTrack | issue'lar | hayır | ikisi de | hayır | hayır | hayır | Markdown |
| Redmine | issue'lar | hayır | yalnızca düzenleme (Redmine 5 ve sonrası), silme asla | hayır | evet (günlükler) | evet (özel notlar) | Markdown veya Textile |
| Trello | kartlar | hayır | ikisi de | hayır | evet (taşımalar) | hayır | Markdown |
| Yerel notlar | notlar | hayır | yerel yazarın kendininkiler | hayır | hayır | hayır | Markdown |

Ayrıca: hiçbir barındırıcı konuşmadan bir satırda yeni bir yorum dizisi başlatmanıza izin vermez (bunu incelemede
yapın) ve tepkiler yalnızca GitHub'da gösterilir. Barındırıcının kabul ettiğinden uzun bir metin reddedilir (GitHub 65.536
karakter, GitLab 1.000.000, Azure DevOps 150.000, Jira 32.767, Trello 16.384): CLI önce denetler, uygulamada barındırıcının
kendi mesajı gösterilir ve metniniz saklanır. Yerel bir not yorumlarını notun dosyasının kendisinde, sondaki bir
`## Comments` bölümünde, her biri bir `### yazar - saat` girdisi olarak tutar; böylece Obsidian ve bir kabuk da onları görür.

Komut satırında aynı konuşmalar `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit` ve
`comment-delete` ile `hosting review conversation` komutlarıdır ([CLI başvurusuna](cli-reference.md) bakın); ajanlar için
aynı adlı MCP araçlarıdır ([MCP ve ajanlar](mcp-and-agents.md) bölümüne bakın).
