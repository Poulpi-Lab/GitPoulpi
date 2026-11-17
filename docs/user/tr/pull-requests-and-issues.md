[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · **Türkçe** · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull request'ler ve issue'lar

Kenar çubuğundan, panodan ve araç çubuğundan pull / merge request'leri ve issue'ları okuma, oluşturma ve yönetme. Önce bir hesap bağlayın ve bir depoyu bağlayın: bkz. [Barındırma ve issue takipçisi entegrasyonları](hosting-integration.md). Tek bir issue veya istek hakkında söylenen her şey (yorumlar, incelemeler, diziler) [Konuşmalar](conversations.md) sayfasındadır; bir isteği okumak ve incelemek [Bir pull veya merge request'i incelemek](review.md) sayfasındadır.

## Kenar çubuğundaki pull request'ler ve issue'lar

### PULL REQUEST'LER

Bölümün; sayıyı ve bir `+` (pull request oluştur) içeren bir başlığı, sağlayıcı
ve proje açılır listeleri, bir arama kutusu ve bir durum açılır listesi (*Açık*,
*Merged / kapanan*, *Tümü*) vardır. Sonra her biri kendi sayısıyla dört grup
gelir:

- **Pull Request'ler · Benim**: yazar sizsiniz.
- **Bana Atanan**.
- **İncelememi Bekleyen**: istenen bir inceleyicisiniz.
- **Pull Request'ler · Tümü**: başlangıçta katlanmış.

Arama kutusu her grubu filtreler (başlık, numara, yazar, dal). Durum açılır
listesindeki her değişiklik barındırıcıya yeniden sorar.

Bir satır, bir durum simgesiyle başlar: `●` açık (yeşil), `○` taslak, `◆` merged
(mor), `✕` kapalı (kırmızı). Checkout yapılmış dalın isteği kalın yazılır.
Satırlar `#12  title` (GitHub, Bitbucket) veya `!53  title` (GitLab, Azure
DevOps) biçiminde okunur.

### ISSUE'LAR

Bölüm başlığı sağlayıcıyı adlandırır: `GITHUB ISSUE'LARI`, `JIRA ISSUE'LARI` vb.
Azure DevOps için `... İŞ ÖĞELERİ`, Trello için `... KARTLARI` olarak okunur.
Açılır listelerin altında: bir arama kutusu, durum açılır listesi (*Açık*,
*Bana atanan*, *Kapalı*, *Tümü*) ve issue'lar. Bir Jira satırı `ABC-12  title`
olarak okunur; kenar çubuğu yeterince genişse tür ve durum soluk gösterilir.

![Kenar çubuğunun ISSUE'LAR bölümü, #7 numaralı issue'nun sağ tık menüsü açık, ilk satır Konuşmayı aç](../images/issues-sidebar.png)

### Bir satırdaki eylemler

- `Enter` veya çift tık satırı açar: bir pull request [incelemede](review.md) (GitHub, GitLab, Gitea, Forgejo, Gerrit),
  bir issue ve Azure DevOps veya Bitbucket'ta bir istek pencerede bir [konuşma](conversations.md) olarak, geri kalan her şey
  tarayıcınızda açılır. Satır menüsündeki **Tarayıcıda aç** her zaman tarayıcıyı açar.
- Sağ tık veya `.`, satır menüsünü açar:
  - **Konuşmayı aç** (issue'lar ve inceleme görünümü olmayan istekler), **Tarayıcıda aç**, **URL'yi kopyala**,
    **Numarayı kopyala**.
  - Bir pull request için: **<provider> üzerinde düzenle**, **Kaynak dalda
    checkout**, **Kaynak dalın commit'lerini göster**. Bir fork'tan gelen istek
    için checkout **Pull Request checkout (fork)** olarak okunur ve **göster**
    gri olur.
  - Bir issue için: **Issue'dan dal oluştur**. `12-fix-the-thing` (Jira için
    `ABC-123-fix-the-thing`) gibi bir adla yeni dal istemini açar.
  - GitHub, GitLab, Gitea ve Forgejo'da bir pull request satırında ayrıca **İncele...**
    da vardır (satırda **Enter** de): uygulama içi inceleme, bkz.
    [Bir pull veya merge request'i incelemek](review.md).
  - GitHub, GitLab, Gitea ve Forgejo'da her iki satırda ayrıca **Atananları düzenle...**,
    **Etiketleri düzenle...** ve **Kilometre taşı ayarla...** (ve bir pull request'te
    **İnceleyenleri düzenle...**) vardır: bkz.
    [Atananlar, inceleyenler, etiketler ve kilometre taşı](#atananlar-inceleyenler-etiketler-ve-kilometre-taşı).
- **<provider> üzerinde düzenle**'nin üç giriş yolu daha vardır: üzerine
  gelinen veya seçili bir pull request satırının sağ ucundaki kalem `✎`, seçili
  satırda `e` ve palet komutu *Entegrasyonlar: Bağlı Pull Request Barındırıcıda
  Düzenle* (checkout yapılmış dalın isteği için).
- Kaynak dal henüz depoda değilse GitPoulpi bunu söyler. Önce fetch edin
  (`Ctrl+G f`).

**Bir pull request'in checkout'u.** GitPoulpi isteği, onu izleyen yerel bir
`pr/<n>` dalına (GitLab'da `mr/<n>`) fetch eder ve ona geçer. GitHub, GitLab, Gitea ve Forgejo'da
bu fork'lar için de çalışır (GitPoulpi barındırıcının özel head ref'ini fetch
eder). Azure DevOps ve Bitbucket isteğin kaynak dalını fetch eder. Orada bir
fork'tan gelen istek reddedilir: fork'u uzak depo olarak ekleyin ve dalını kendiniz
checkout edin. Geri al, önceki dala döndürür. CLI: `gitpoulpi hosting
checkout-pr <n>`.

**Bir pull request'i komut satırından (ve bir ajanla) incelemek.** GitHub'da
(Enterprise dahil), GitLab'da (kendi barındırdığınız dahil), Gitea'da ve Forgejo'da bir istek
GitPoulpi'den ayrılmadan okunabilir ve incelenebilir: `hosting review show <n>` (dosyalar,
merge durumu, denetimler), `hosting review threads <n>` (inceleme yorum dizileri ve
konuşma), `hosting review comment`, `hosting review reply`, `hosting review resolve`
ve `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Diff barındırıcıdan değil, deponuzdan gelir:
`hosting review fetch <n>` isteğin head'ini `refs/gitpoulpi/pr/<n>` içine ve hedef dalını
`refs/gitpoulpi/pr/<n>-base` içine fetch eder (hiçbiri bir dal değildir, hiçbir şey checkout
edilmez). Satır numaraları o diff'inkilerdir: `new` head'deki satır, `old` merge base'deki
satırdır. GitLab'da "değişiklik iste" inceleme durumu yoktur: orada reddedilir, hiçbir şey
gönderilmez. Gitea ve Forgejo yalnızca tek bir satıra yorum alır (aralık yok); bir yorum dizisi,
bir dosyanın tek bir satırındaki yorumlardır. Gitea bir yorum dizisini çözebilir, Forgejo 11
çözemez (barındırıcının bunun için bir uç noktası yok: çağrı bunu söyler). Yanıt uç noktası
olmayan bir sunucudaki bir yanıt, aynı satırda tek yorumlu bir inceleme olarak gönderilir.
Gitea kendi isteğinizi onaylamayı veya reddetmeyi reddeder.
Klon olmadan: geçerli depo olmayan bir projeyi incelemek için `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` ve `suggest` komutlarına `--project owner/name` (GitLab: `group/project`) ekleyin;
`hosting review diff <n> --project P` diff'i (dosya başına hunk'lar) barındırıcının API'sinden okur ve
`hosting file <path> --project P [--ref R]` bir dosyayı bir düzeltmede okur (GitHub, GitLab, Gitea, Forgejo ve Gerrit; Gerrit'te bir
yama kümesinin diff'i dosya dosya okunur ve dosya daldan ya da bir commit kimliği için commit'ten gelir). Barındırıcının diff'te
göstermediği (çok büyük) bir dosya `omitted` olarak işaretlenir.
MCP araçları aynı komutlardır (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Atananlar, inceleyenler, etiketler ve kilometre taşı

GitHub'da (Enterprise dahil), GitLab'da (kendi barındırdığınız dahil), Gitea'da ve
Forgejo'da, bir pull request'in veya issue'nun kime atandığını, kimin incelediğini, etiketlerini
ve kilometre taşını GitPoulpi'den ayrılmadan değiştirebilirsiniz. Diğer sağlayıcılar bunların
hiçbirini göstermez.

- **Satır menüsü**: **Atananları düzenle...**, **İnceleyenleri düzenle...** (yalnızca pull
  request'ler), **Etiketleri düzenle...**, **Kilometre taşı ayarla...**.
- **Palet**: *Barındırma: Atananları Düzenle...*, *Barındırma: İnceleyenleri Düzenle...*,
  *Barındırma: Etiketleri Düzenle...*, *Barındırma: Kilometre Taşı Ayarla...* her biri önce
  aralarından seçeceğiniz kenar çubuğunun isteklerini ve issue'larını listeler.
  [Radar](radar.md)'da aynı dördü seçili satırda çalışır.
- Seçici hemen açılır ve projenin kişilerini veya etiketlerini (ve öğenin şu anda sahip
  olduklarını) barındırıcıdan yükler. Yaklaşık 150 ms sonra bir döner çubuk satırı görünür.
  Filtrelemek için yazın (bulanık eşleşme), `Up` / `Down` gezinir, `Space` işaretler veya
  işareti kaldırır, `Enter` uygular, `Esc` iptal eder, `Tab` düğmelere ulaşır. Bir satıra
  tık onu işaretler ve tekerlek kaydırır. Kişiler bir baş harf rozeti, etiketler rengini gösterir.
- Kilometre taşı bir açılır listedir (temizlemek için **Kilometre taşı yok** ile).
- **Uygula**'dan sonra barındırıcı güncellenir ve listeler ile Radar değişikliği,
  hemen başlayan bir sonraki yenilemelerinden sonra gösterir.

Oluşturma formlarında (bir pull request, bir issue) aynı seçimler bir düğme sırası olarak
bulunur: `Atananlar: yok`, `İnceleyenler: yok`, `Etiketler: yok`,
`Kilometre taşı: yok`. GitHub, GitLab, Gitea ve Forgejo'da Etiketler düğmesi issue formunun
yazılan etiketler alanının yerini alır. Bir barındırıcı yeni pull request'i kabul edip bazı
etiketlerini veya kişilerini reddederse durum çubuğu bunu söyler.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>` ve `hosting pr` ile `hosting issue create` üzerinde `--assignee`,
`--reviewer`, `--label`, `--milestone`; bkz.
[CLI başvurusu](cli-reference.md).

### Bölümlerdeki tuşlar

| Tuş | Eylem |
|---|---|
| `Enter` / çift tık | barındırıcının konuşması varsa GitPoulpi'de aç (GitHub, GitLab, Gitea veya Gerrit'te bir pull request incelemeyi açar), yoksa tarayıcıda |
| `o` | satırı tarayıcıda aç |
| `.` / sağ tık | satır menüsü |
| `e` | seçili pull request'i barındırıcıda düzenle |
| `n` | yeni pull request (PULL REQUEST'LER) veya yeni issue (ISSUE'LAR) |
| `v` | durum açılır listesini aç |
| `r` | yenile |
| Bir başlıkta `Enter` / `Space` | bölümü katla veya aç |

### Yeniden bağlanma ve çevrimdışı

Token eksik, reddedilmiş veya bir kapsamı yoksa bölümün ilk satırı `Yeniden
bağlan (token reddedildi)` olarak okunur. Profili Ayarlar > Entegrasyonlar'da
açmak için üzerine tıklayın veya `Enter`'a basın.

Barındırıcıya ulaşılamadığında son listeler ekranda kalır ve başlık yaşlarını
gösterir (`· 5m önce`). Son yanıt ayrıca diskte önbelleğe alınır
(`.gitpoulpi/hosting-cache.json`), böylece listeler ağ yanıt vermeden önce
görünür.

## Issue panosu

ISSUE'LAR başlığındaki **Pano** düğmesi (palet: **Issue'lar: Panoyu Aç**), bağlı kaynağın issue'larını pencerenin
üzerinde bir Kanban panosu olarak gösterir. Sütunlar kaynağa bağlıdır:

| Kaynak | Sütunlar | Bir kartı taşımak |
|---|---|---|
| Yerel Markdown notları | durumlar (profilin `statuses` değeri, sonra kullanımdaki diğerleri) | notun `status` değerini yeniden yazar |
| GitHub, GitLab, Gitea, Forgejo | **Açık**, sonra seçtiğiniz etiketler, sonra **Kapalı** | durum etiketini değiştirir; **Kapalı**'ya taşımak issue'yu kapatır, yeniden bir sütuna taşımak yeniden açar |
| Trello | panonun açık listeleri, soldan sağa | kartı listeye taşır |
| Redmine | issue durumları | issue'nun durumunu değiştirir (Redmine'ın iş akışı kuralları uygulanır) |
| Jira (Cloud ve Data Center) | projenin durumları, yapılacaklar önce, bitenler sonda | sütuna götüren iş akışı geçişini çalıştırır (`GET`, sonra `POST .../transitions`); böyle bir geçiş yoksa kart geri gider ve durum çubuğu `no transition from X to Y` der |
| Azure DevOps (Services ve Server) | projenin work item türlerinin state'leri, state kategorisine göre | `System.State` değerini ayarlar; türünde sütunun state'i olmayan bir work item geri gider, nedeniyle birlikte (`a Bug has no state Done`) |

Bir GitHub veya GitLab projesinde **Sütunlar** (veya `c`, veya **Pano: Sütun Etiketlerini Ayarla...**) sütun olan
etiketleri, sırayla, örneğin `status::todo, status::doing, status::review`, sorar. Projenin duruma benzeyen
etiketlerini önerir. Seçim depo için kaydedilir. Etiket yoksa pano Açık / Kapalı'dır. Bir kartı taşımak yalnızca sizin
listelediğiniz durum etiketlerine dokunur: diğer etiketleri kalır. Bir barındırıcı panosu açık issue'ları ve en son
kapatılan 60 issue'yu listeler.

Zorunlu alanlar isteyen bir Jira geçişi (örneğin bir çözüm) Jira tarafından reddedilir ve kart Jira'nın kendi
mesajıyla geri gider: onu sitede yapın.

Aynı pano betikler ve ajanlar için de vardır: `gitpoulpi hosting board` sütunları kartlarıyla yazdırır ve
`gitpoulpi hosting board move <card> <column>` birini taşır (bkz. [CLI başvurusu](cli-reference.md)).

Bir kart numarayı, başlığı, atananları baş harfler olarak, etiketleri ve kilometre taşını gösterir. Onu başka bir
sütuna sürükleyin, ya da `Shift+Left` / `Shift+Right` veya **Pano: Kartı Şuraya Taşı...** (`m`) kullanın. Hemen taşınır;
yaklaşık 150 ms sonra bir döner çubuk barındırıcının hâlâ çalıştığını söyler ve barındırıcı reddederse kart geri gider ve
durum çubuğu nedenini söyler. `Enter` issue'yu barındırıcıda açar (yerel issue'larda notu); `e` atananları, etiketleri
ve kilometre taşını düzenler; `n` seçili sütunda yeni bir issue başlatır; `a`, `l`, `M` ve `/` atanana, etikete,
kilometre taşına ve metne göre filtreler, `x` filtreleri temizler; `r` barındırıcıyı yeniden okur. Pano, kenar çubuğunun
listeleri yenilendiğinde yeniden okur. Radar'ın panosu yoktur.

## Pull request oluşturun

Formu PULL REQUEST'LER başlığındaki `+` ile, bölümde `n` ile ya da *Pull
Request'ler: Pull Request Oluştur...* palet komutuyla açın. Bir dal menüsünde de
**<branch> dalından pull request başlat...** vardır.

Formda şunlar bulunur:

- Pull request'i olan her bağlı profil için bir sekme (sekme şeridinde `Left` /
  `Right`).
- **Kaynak Depo / Dal** ve **Hedef Depo / Dal** kombo kutuları. Uzun listelerde
  arama yapılabilir. Listelerin göstermediği bir dal veya depo yazabilirsiniz.
- **Başlık**, uç commit'in konusuyla doldurulur.
- **Açıklama**, birden çok satırlı (`Enter` yeni satır ekler, `Tab` alandan çıkar).
- **Atananlar**, **İnceleyenler**, **Etiketler** ve **Kilometre taşı** düğmeleri (GitHub,
  GitLab, Gitea ve Forgejo), seçiciyi açar.
- **Taslak olarak gönder**.
- [YZ](ai.md) açıkken: **Başlık ve açıklama oluştur**.

Form, checkout yapılmış dalın profilinde, deposunda ve dallarında açılır
(`feature -> main`). Farklı bir kaynak depo, bir fork'tan istek oluşturur.

**Pull Request oluştur** (veya `Ctrl+S`) isteği barındırıcıda oluşturur. Sonuç
tarayıcınızda açılır ve durum çubuğu onu adlandırır.

## Issue oluşturun

ISSUE'LAR bölümünün bir sağlayıcısı **ve** bir projesi olduğunda başlığında bir
`+` (*+ Yeni issue*) bulunur. Ayrıca: bölümde `n` ya da *Barındırma: Yeni
Issue...* palet komutu.

Formda **Başlık** (zorunlu), **Açıklama** (birden çok satır) ve **Etiketler**
(virgülle ayrılmış, örneğin `bug, ui`; GitHub, GitLab, Gitea ve Forgejo'da bunun yerine
Etiketler, Atananlar ve Kilometre taşı düğmeleri seçiciyi açar) bulunur. Jira ve Azure DevOps bir **Tür**
açılır listesi ekler. Trello bir **Liste** açılır listesi ekler. Linear ve YouTrack bir **Durum**
açılır listesi ekler (issue'nun içinde başladığı iş akışı durumu veya `State` değeri). Redmine, projenin
izleyicileriyle (Bug, Feature, Support...) bir **Tür** açılır listesi ekler; etiketi yoktur, bu yüzden
onun için Etiketler düğmesi yoktur ve komut satırında verilen bir etiket reddedilir. Form bu açılır
listeleri açılırken barındırıcıdan doldurur.

**Issue Oluştur** (veya `Ctrl+S`) onu barındırıcıda oluşturur. `Esc` veya
**İptal** formu kapatır. Barındırıcı reddederse (token reddedildi, eksik kapsam,
bilinmeyen proje, bilinmeyen etiket) form nedenini gösterir ve yazdıklarınızı
korur. Başarılı olursa durum çubuğu yeni issue'yu adlandırır ve ISSUE'LAR bölümü
yenilenir.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Barındırıcının kendi web sayfasında oluşturun

Yeni Issue formunda ve Pull Request oluştur formunda, sağ üstte bir
**<provider> üzerinde oluştur** düğmesi vardır. Tuşu `Alt+W`'dir. Barındırıcının
web formunu tarayıcınızda, o ana kadar yazdıklarınızla doldurulmuş olarak açar.
İletişim kutusu açık kalır. Token gerektirmez ve API çağrısı yapmaz; bu yüzden
bir API hatasından sonra da çalışır.

- GitHub'ın *taslak* için bir URL parametresi yoktur, bu yüzden taslak kutusu
  taşınmaz. GitLab, `Draft: ` başlık öneki alır.
- GitLab etiketleri yalnızca kimlikle alır, bu yüzden açıklamanın sonuna bir
  `/label ~"x"` hızlı eylemi olarak eklenir.
- Çok uzun bir açıklama, URL 6000 karakterin altında kalsın diye kesilir ve bir
  `[...truncated: paste the rest]` notu eklenir.
- Bitbucket Data Center'ın web formu adresi yoktur, bu yüzden
  **<provider> üzerinde oluştur** düğmesi yoktur.

CLI: `gitpoulpi hosting issue create ... --web` ve `gitpoulpi hosting pr --web`
aynı URL'yi yazdırır.

## Araç çubuğu çipi

Araç çubuğunun ucunda (varsayılan olarak sağda) bir çip, checkout yapılmış dalın isteğini gösterir:
`GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) veya `BB #12`
(Bitbucket). Rengi durumu gösterir: yeşil açık, gri taslak, mor merged, kırmızı
kapalı.

- Başlık, durum ve `source → target` için üzerine gelin.
- İsteği tarayıcıda açmak için tıklayın veya `Ctrl+G g` tuşlarına basın.
- **Tarayıcıda aç**, **URL'yi kopyala**, **Yenile** ve **Profili değiştir** için
  sağ tıklayın.

İsteği olmayan, push edilmiş bir dal soluk bir `+ PR` çipi gösterir. Bir pull
request başlatmak için üzerine tıklayın. Hiçbir profil eşleşmediğinde, dal push
edilmemişse veya barındırıcıya ulaşılamadıysa çip yoktur. Dar bir pencerede çip,
araç çubuğu düğmelerine yer bırakmak için gizlenir.

## Listeler ne zaman yenilenir

Listeler şu durumlarda yenilenir:

- bir depo açtığınızda;
- checkout yapılmış dal değiştiğinde;
- bir push veya fetch'ten sonra;
- her *Pull Request ve Issue Yenileme (dakika)* sürede
  (`general.hosting_refresh_minutes`, varsayılan 5, `0` zamanlayıcıyı kapatır;
  Ayarlar > Entegrasyonlar);
- Ayarlar'ı kapattığınızda;
- `r` tuşuna bastığınızda, *Entegrasyonlar: Pull Request'leri ve Issue'ları
  Yenile* komutunu çalıştırdığınızda ya da bir filtreyi değiştirdiğinizde.

İstekler pencereyi asla engellemez.

Barındırıcılar yapabileceğiniz çağrı sayısını sınırlar. GitPoulpi son yanıtın
`ETag` değerini geri gönderir, bu yüzden değişmemiş bir liste çok az maliyetlidir
(GitHub'da hiç). Bir barındırıcı az çağrı kaldığını bildirdiğinde durum çubuğu
bunu söyler. Hiç kalmadığında GitPoulpi, sınır sıfırlanana kadar o barındırıcıya
sormayı bırakır ve listeler olduğu gibi kalır.

## Terminoloji

GitHub, Azure DevOps ve Bitbucket "pull request" der. GitLab "merge request"
der. *İnceleme isteği terimi* (`ui.pr_terminology`, Ayarlar > Entegrasyonlar'da)
her etiket, menü, palet komutu ve mesaj için sözcüğü seçer:

- **Otomatik (barındırıcıya göre)**: deponun bağlı sağlayıcısını izler (hiçbir şey
  bağlı değilken "Pull Request").
- **Pull Request** veya **Merge Request**: her zaman o sözcük.

Değişiklik hemen uygulanır. CLI ve MCP her zaman "pull/merge request" der. Bu
sayfa "pull request" der.

## Commit mesajlarındaki ve dal adlarındaki referanslar

Deponun issue'ları veya pull request'leri bağlıyken, Commit ayrıntılarındaki
commit mesajında yer alan referansların altı çizilir. Bir tık bağlı sayfayı
açar. Klavye: palet komutu *Entegrasyonlar: Commit Referansını Aç (#issue /
!istek)...* (mesajda birkaç referans varsa bir menü). Hiçbir şeyin bağlı olmadığı
bir depoda bağlantı yoktur.

Neyin referans sayılacağı, ISSUE'LAR bölümünün sağlayıcısına bağlıdır:

| ISSUE'LAR şuna bağlı | Bir mesajda | Bir dal adında |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` ve `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, yalnızca `ABC` bağlı projenin anahtarıysa. `UTF-8`, `SHA-256` veya `ISO-8601` gibi metinler asla bağlantı değildir. | Adın herhangi bir yerinde `ABC-123` (`feature/ABC-123-x`), büyük harfle |
| Linear | `ENG-123`, yalnızca `ENG` bağlı takımın (veya profilin takım listesindeki bir takımın) anahtarıysa. `UTF-8` asla bağlantı değildir. | Bir issue'dan dal oluşturduğunuzda Linear'ın kendi `branchName` değeri; adın herhangi bir yerindeki `ENG-123` geri okunur |
| YouTrack | `ABC-123`, yalnızca `ABC` bağlı projenin (veya profilin proje listesindeki birinin) kısa adıysa | Adın herhangi bir yerinde `ABC-123` |
| Redmine | `#123`, yalnızca Redmine bağlı ISSUE'LAR kaynağıysa (bir pull request kaynağı kendi işaretini korur) | `123-fix-login` |
| Trello | bir `https://trello.com/c/<shortLink>` adresi; `#12` yalnızca önbellekteki bir kartın numarası 12 ise | `12-fix-login` |

| PULL REQUEST'LER şuna bağlı | Bir mesajda |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Her iki liste de `#N` ile eşleştiğinde issue kazanır. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**Konuşmalarda.** Bir issue veya isteğin metni barındırıcısının dilbilgisiyle okunur; bu, yukarıdaki tablodan daha geniştir: `owner/repo#7`, `!7`, `GH-7`, `@login`, commit kimlikleri ve [Çapraz referanslar](conversations.md#çapraz-referanslar) bölümünde listelenen diğer biçimler. Deponun bağlı kaynakları önce eşleştirilir; bu yüzden bir GitHub yorumundaki bir Jira anahtarı yine de issue'sunu uygulamada açar. `gitpoulpi hosting markdown --body TEXT --id I` uygulamanın bir metinde ne bulduğunu gösterir.

### Bir issue referansı ekleyin

Commit composer'da, Özet'in sağ ucundaki `#▾` düğmesi, bağlı issue'ların
filtrelenebilir bir listesini açar. Geçerli dalın adlandırdığı issue *(daldan)* ile
işaretlenmiş olarak en başta gelir. `Enter` veya bir tık, referansını (`ABC-123`,
`#12`, `AB#12`) imlecin olduğu yere ekler. Klavye: composer'da `Alt+R`. Palet:
*Commit: Issue referansı ekle...*. Bağlı bir issue listesi yoksa düğme gri olur
ve *önce kenar çubuğunda bir issue listesi bağlayın* der. GitPoulpi kendiliğinden
asla referans eklemez.
