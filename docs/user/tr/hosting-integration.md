[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · **Türkçe** · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Barındırma ve issue takipçisi entegrasyonları

GitPoulpi şu ürünlere bağlanır:

- **Barındırma** (pull request'ler, issue'lar, hesabınızdan clone, SSH
  anahtarları): GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure
  DevOps, Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea ve
  Forgejo (Codeberg dahil). **Gerrit Code Review** yalnızca inceleme için bir
  barındırıcıdır: değişiklikleri pull request'lerdir, issue'su yoktur.
- **Issue takipçileri** (yalnızca issue'lar): Jira Cloud, Jira Data Center,
  Trello, Linear, YouTrack (Cloud ve Server) ve Redmine.
- **Yerel Markdown notları** (yalnızca issue'lar, token yok, ağ yok): Markdown notlarından oluşan bir klasör, örneğin bir Obsidian vault klasörü, bkz. [Yerel issue'lar ve Obsidian](local-issues.md).

Her entegrasyon bir **kişisel token** kullanır. GitPoulpi hesabı yoktur ve OAuth
ile oturum açma yoktur. Entegrasyonlar isteğe bağlıdır ve **depo başına
isteğe bağlı olarak etkinleştirilir**: bir depoyu bağlayana kadar hiçbir şey
getirilmez. Token'ınız kimlik bilgisi yardımcınızda saklanır, `config.toml`
içinde asla. Düz `git` işlemleri (fetch, pull, push) bir entegrasyon
gerektirmez: `git`'inizin zaten sahip olduğu kimlik bilgilerini kullanırlar.

## 1. Barındırıcıda bir token oluşturun

Bağlantı formunda bir *<host> üzerinde token oluştur* bağlantısı vardır. Barındırıcının
token sayfasını, doğru kapsamlar önceden seçilmiş olarak açar. Elle yapmak için:

| Barındırıcı | Nerede | Kapsamlar |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (depolar, pull request'ler, issue'lar), `read:user` (kim olduğunuz), `admin:public_key` (yalnızca SSH anahtarı eklemek veya kaldırmak için) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, `dev.azure.com/<organization>/_usersSettings/tokens` adresinde | `Code (Read & write)`, `Work Items (Read & write)`. Token'ı "All accessible organizations" için değil, **o kuruluş için** oluşturun: genel token'lar 2026-12-01 tarihinde çalışmayı bırakır. |
| Azure DevOps Server | Sunucunuzdaki aynı sayfa (`https://<server>/<collection>/_usersSettings/tokens`) | Aynı iki kapsam |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. SSH anahtarları için ayrıca `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profil resmi > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | `<host>/user/settings/applications` adresinde Settings > Applications > Generate New Token | `user` okuma; `repository`, `issue` ve `user` okuma ve yazma (sonuncusu SSH anahtarları için). Gitea kapsamları sonradan göstermez, bu yüzden GitPoulpi eksik bir kapsam için uyaramaz: reddedilen bir çağrı bunu söyler. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. Anahtar sizin adınıza davranır. |
| YouTrack (Cloud ve Server) | `<host>/users/me?tab=account-security` adresinde profiliniz > Account Security > New token | `YouTrack` kapsamı. Kalıcı bir token sizin adınıza davranır. |
| Gerrit | `<host>/settings/#HTTPCredentials` adresinde Settings > HTTP Credentials > Generate new token (yeni sunucular; eskilerde: Generate new password) | Yok. Token **HTTP parolasıdır** ve sizin adınıza davranır; kullanıcı adı yanına girilir. |
| Redmine | `<host>/my/account` adresinde My account > API access key > Show. Yöneticinin **Administration > Settings > API > Enable REST web service** ayarını açmış olması gerekir. | Yok. Anahtar sizin adınıza davranır. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (klasik tür, kapsamsız) | Yok. Token sizin adınıza davranır. |
| Jira Data Center | Profile > Personal Access Tokens | Yok. Token sizin adınıza davranır. |
| Trello | Önce `trello.com/apps/admin` adresinden bir **API anahtarı** (bir Power-Up oluşturun, sonra bir API anahtarı üretin). Sonra `trello.com/1/authorize?...&key=<your key>` adresinden bir **token**. | `read`, `write` |

Token'a "GitPoulpi" gibi bir ad ve bir son kullanma tarihi verin. Token'ı kopyalayın:
barındırıcı onu yalnızca bir kez gösterir.

## 2. Bağlayın

1. `Ctrl+,` tuşlarına basın ve **Entegrasyonlar**'ı seçin.
2. Solda ürünü seçin. Ürünler iki gruptadır: **Barındırma** ve **Issue takipçileri**.
3. **Bağlan**'a basın. Aynı ürünün ikinci bir hesabını eklemek için **+ Profil
   ekle**'yi kullanın.

   ![Entegrasyonlar sayfası: solda ürünler ve GitHub için bir Bağlan düğmesi](../images/settings-integrations.png)

4. Formu doldurun ve token'ı yapıştırın. Token, yazarken düz metin olarak
   görünür; bu yüzden kimsenin bakmadığından emin olun. Form, ihtiyaç duyduğu
   kapsamları listeler.

   ![Bağlantı formu: bir Kişisel Erişim Token'ı alanı, repo, read:user ve admin:public_key kapsamları, github.com üzerinde token oluşturmak için bir bağlantı ve Bağlan ile İptal düğmeleri](../images/settings-connect.png)

5. **Bağlan**'a basın. GitPoulpi token'ı arka planda barındırıcıyla denetler.
   Sayfa sonra adınızı ve oturum adınızı içeren bir kart, yeşil renkte *Bağlı* ve
   SSH anahtarı bölümünü gösterir. İstenen kapsamlardan birine sahip olmayan bir
   token yine bağlanır ve eksik kapsamlar kırmızıyla listelenir.

Kartta: **Düzenle** formu yeniden açar. **Bağlantıyı kes** token'ı siler ve
profili korur. **Kaldır** ikisini de siler, bir onaydan sonra.

Entegrasyonlar sayfasındaki klavye: `Tab` / `Shift+Tab` ürün listesi, profil
şeridi ve form arasında geçer. `a` bir profil ekler, `e` düzenler, `d` veya
`Delete` kaldırır, `o` token sayfasını açar, `Esc` bir adım geri gider.

### Her formun istedikleri

*Token oluştur* bağlantısı yalnızca barındırıcıyı veya kuruluşu doldurduktan
sonra çalışır.

- **GitHub, GitLab:** yalnızca token. **GitHub Enterprise** ve **GitLab
  Self-Managed** ayrıca **Sunucu Alan Adı**nı ister (`gitlab.example.com`;
  `http://` yazmazsanız https). Noktasız bir ad (bir Docker servisi veya yerel
  ağdaki bir makine) bir port ya da `http://` ister: `gitea:3000`,
  `http://nas`. Kendi barındırdığınız sunucu düz http olabilir, örneğin
  `http://localhost:8080` adresindeki yerel bir kapsayıcı. Düz http üzerinden gönderilen token şifrelenmeden
  iletilir, bu yüzden http'yi yalnızca yerel veya güvenilir bir ağda kullanın.
- **Azure DevOps:** **Kuruluş**, yani `dev.azure.com/<organization>`
  içindeki ad. Bir `https://<org>.visualstudio.com` adresi de kabul edilir. İki
  kuruluş için iki profil gerekir. Bir depo, uzak deposunun adını verdiği
  kuruluşun profilini kullanır. **Azure DevOps Server**, **Koleksiyon URL'si**
  ister, örneğin `tfs.corp/tfs/DefaultCollection`. GitPoulpi, Azure DevOps için
  sistem kimlik bilgisi yardımcısını kullanmaz: Git Credential Manager orada bir
  kişisel erişim token'ı değil, bir oturum açma token'ı saklar.
- **Bitbucket:** **Atlassian hesabı e-postası** ve bir API token'ı. E-posta
  boşsa token bir bearer token olarak gönderilir: bunu bir çalışma alanı veya
  depo erişim token'ı için kullanın. GitPoulpi çalışma alanlarınızı token'dan
  bulur. **Bitbucket Data Center:** **Sunucu URL'si** ve bir HTTP erişim token'ı.
- **Gitea, Forgejo:** **Sunucu alan adı** ve token. **Forgejo**
  `https://codeberg.org` ile başlar: kendi kurduğunuz bir sunucu için bunu
  değiştirin. Gitea'nın herkese açık bir varsayılanı yoktur. Yerel bir sunucu
  için `http://localhost:3000` adresi çalışır. İki tür de tek bir istemciyi
  paylaşır (Forgejo, aynı `/api/v1`'e sahip bir Gitea fork'udur); token
  `Authorization: token <token>` olarak gönderilir. Sistem kimlik bilgisi yardımcısı
  bunlar için kullanılmaz.
- **Jira Cloud:** **Site URL'si** (`acme.atlassian.net`), **Atlassian hesap
  e-postanız** ve klasik bir API token'ı. Kapsamlı Jira token'ları
  desteklenmez. **Jira Data Center:** **Sunucu URL'si** ve bir kişisel erişim
  token'ı.
- **Trello:** **API anahtarı** ve **token**. API anahtarı herkese açıktır, bu yüzden
  `config.toml` içinde tutulur (`account` olarak). Token gizlidir ve kimlik
  bilgisi yardımcısına gider. Formda iki bağlantı vardır: *Trello'dan API
  anahtarı al* ve, anahtarı yazdıktan sonra, *Token oluştur*. Anahtar ve token bir
  başlıkta gönderilir, asla bir URL'de değil.

- **Linear:** yalnızca **API anahtarı**: Linear'ın tek bir herkese açık
  sunucusu vardır, bu yüzden yazılacak bir adres yoktur. Anahtar `Authorization`
  başlığında olduğu gibi gönderilir (Linear kişisel anahtarlar için `Bearer`
  almaz). Commit mesajlarındaki `ENG-12` referanslarının doğru sayfayı açması
  için profil URL'sini çalışma alanınızla saklayın (`https://linear.app/acme`,
  CLI'dan: `--url`); issue satırlarının kendisi her zaman tam adresi taşır.
- **YouTrack:** **Site URL'si** (Cloud için `acme.youtrack.cloud`, aksi halde kendi
  sunucunuz; bir bağlam yolu varsa onunla, örneğin `host/youtrack`) ve
  `Authorization: Bearer` olarak gönderilen kalıcı bir token. Aynı istemci Cloud'a
  ve Server'a hizmet eder (`<url>/api`).

- **Gerrit:** **Sunucu URL'si** (varsa bağlam yoluyla), **Kullanıcı adınız** ve token olarak HTTP
  parolası. HTTP basic kimlik doğrulaması olarak gönderilir; GitPoulpi sunucunun `/a/` REST API'siyle konuşur ve her
  yanıtın başladığı `)]}'` satırını yok sayar. Kullanıcı adı gizli değildir ve `config.toml` içinde tutulur
  (`account` olarak), parola kimlik bilgisi yardımcısına gider. Bir depo, PULL REQUEST'LER altında bir Gerrit
  *projesine* bağlanır (değişiklikleri orada listelenir); Gerrit'in issue'su yoktur ve bir değişiklik push ederek
  oluşturulur (bkz. [İnceleme](review.md#gerrit) içindeki *İnceleme için push*).
- **Redmine:** Redmine'ınızın **Site URL'si** (varsa alt yoluyla, örneğin
  `host/redmine`) ve `X-Redmine-API-Key` olarak gönderilen API anahtarınız. Redmine
  403 veya bir HTML sayfasıyla yanıt verirse REST web servisi kapalıdır (yöneticiye
  sorun ya da **Administration > Settings > API** içinde açın).

Jira, Trello, Linear, YouTrack ve Redmine yalnızca issue takipçisidir: pull request yok, clone edilecek depo
yok ve SSH anahtarı yok. Bitbucket'ın issue'su yoktur (Atlassian, Bitbucket
Issues'ı kaldırdı). Bunun yerine bir Bitbucket deposunun ISSUE'LAR bölümünü bir
Jira projesine bağlayın.

## 3. Token nerede saklanır

Token, sistem **kimlik bilgisi yardımcınızda** saklanır (Windows'ta Git
Credential Manager, Linux'ta libsecret veya `pass`, macOS'ta Keychain). Gerçek
bir barındırıcının kimlik bilgileriyle karıştırılamayacak özel bir girdi
kullanır. `config.toml` içine asla yazılmaz ve asla yazdırılmaz.

Hiçbir kimlik bilgisi yardımcısı yapılandırılmamışsa ya da token'ı reddederse,
GitPoulpi token'ı **yalnızca bu oturum için bellekte** tutar ve size bunu
söyler. Yeniden başlatmadan sonra yeniden bağlanın.

GitPoulpi bir profilin token'ını şu sırayla arar:

1. `GITPOULPI_TOKEN_<PROFILE ID>` ortam değişkeni. Bir token'ı saklamadan
   vermek için kullanın, örneğin CI'da.
2. Profil için saklanan token (`token_ref`).
3. Yalnızca GitHub ve GitLab profilleri için: başka hiçbir etkin profil aynı
   barındırıcıyı kullanmıyorsa, o barındırıcının sistem kimlik bilgisi yardımcısı.

Her profil, `config.toml` içinde bir `[[integrations]]` girdisidir. Bkz.
[yapılandırma başvurusu](config-reference.md#integrations-barındırma-ve-issue-takipçileri).

## Güvenlik denetimi

Kayıtlı bir token, gönderildiği adres kadar güvenlidir. Bu yüzden GitPoulpi her entegrasyon için neye bağlandığını hatırlar ve bu değiştiğinde durur:

- **Adres.** Bir entegrasyon ilk kullanıldığında (veya bağlandığında) adresi, `config.toml` dosyasına değil, yalnızca kullanıcıya ait özel veri klasöründeki (`<data>/private/`) `trust.toml` dosyasına kaydedilir: yapılandırmayı düzenlemek güvenilenleri asla genişletemez. Yapılandırmadaki adres artık kayıtlı olan değilse ona hiçbir şey gönderilmez ve token okunmaz bile. Bir adresin yazılış biçimleri (`HTTPS://Git.Example.com:443/`) aynı adrestir; benzer görünen bir sunucu aynı değildir.
- **Hesap.** Bir token bağladıktan sonra, sonra yaklaşık günde bir kez (ve `GITPOULPI_TOKEN_*` veya kimlik bilgisi yardımcısından gelen bir token için çalıştırma başına bir kez) GitPoulpi barındırıcıya token'ın kime ait olduğunu sorar ve girişi değil, hesap **kimliğini** karşılaştırır. Artık başka birine ait olan bir token durdurulur.
- **Sertifika.** Sertifikası bilinen bir otoriteye zincirlenmeyen sunucular (özel bir CA, kendinden imzalı bir sunucu) yalnızca kabul ettiğiniz tam anahtar için kabul edilir ve TLS el sıkışması yine de sunucunun o anahtarı taşıdığını kanıtlar. Sıkı sabitleme açıkken, sertifika herkese açık bir köke zincirlense bile anahtar eşleşmelidir; bedeli her anahtar değişiminin yeniden sormasıdır, bu yüzden isteğe bağlıdır. Herkese açık barındırıcılar (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) sertifika kaydı tutmaz.
- **Düz http.** Loopback olmayan bir `http://` adresi, o entegrasyon için ona izin vermediyseniz reddedilir.

Her istek ayrıca entegrasyonun kendi adresine (ve herkese açık bir barındırıcı için `api.github.com` gibi API sunucusuna) bağlı tutulur: başka bir yere giden bir istek gönderilmeden reddedilir ve başka bir adrese yönlendirme izlenmez.

**Bu denetim var olmadan önce bağlanan entegrasyonlar** kayıtlarını, yükseltmeden sonra ilk kullanıldıklarında, yapılandırmanın o zamanki halinden sessizce alır: bugünkü güven düzeylerini korurlar ve hiçbir şey sormaz.

### Komut satırından

Durdurulan bir komut çıkış kodu 2 ile başarısız olur ve neyin değiştiğini yazdırır (bkz. [CLI başvurusu](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` nelere güvenildiğini gösterir; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` ve `--allow-insecure-http on|off` her biri tam değeri adlandırarak bir şeyi kabul eder. Yeni bir adresi kabul ettikten sonra token'ı yeniden girin. Bağlantıyı kesmek (`hosting logout`) veya bir entegrasyonu kaldırmak token'ını, güven kaydını ve ondan önbelleğe alınanları (Klonla iletişim kutusunun kayıtlı depo listeleri, Radar hesap listeleri ve barındırıcısının uzak inceleme taslakları) siler.

Bir ajan (MCP) hiçbir şeyi kabul edemez: güvenlik denetiminin sizi gerektirdiği söylenir.

### Terminal arayüzünde

Durdurulan bir istek, o sırada ne yapıyorsanız onun üzerinde **Entegrasyon güvenlik denetimi** iletişim kutusunu açar. Neyin değiştiğini (eski ve yeni adres, kimlikleriyle eski ve yeni hesap, ya da SHA-256 parmak izi dörderli gruplar halinde, konu, veren ve geçerlilikle küçük bir tablo olarak eski ve yeni sertifika) ve token'ın gönderilmediğini söyler. Yanıtlar numaralıdır:

1. **Bağlantıyı kes** kayıtlı token'ı, güven kaydını ve entegrasyondan önbelleğe alınanları siler.
2. Yeni adrese, hesaba veya sertifikaya **güven** (şifrelenmemiş bir adres için: **Düz http'ye izin ver**).
3. **İptal** hiçbir şeyi değiştirmez.

Bir ok tuşuna veya bir rakama basana kadar hiçbir yanıt vurgulanmaz; böylece yanlışlıkla basılan bir Enter hiçbir şeyi kabul etmez. Esc ve kapatma çarpısı iptal eder; iletişim kutusunun dışına tıklamak hiçbir şey yapmaz. Uzun bir gövde tekerlek, Page Up/Page Down veya kaydırma çubuğuyla kayar. Birkaç denetim sıraya girer (entegrasyon ve değişiklik türü başına bir tane) ve birbiri ardına sorulur. İptal edilen bir denetim, değişmediği sürece yeniden çıkmaz; komut paletindeki **Entegrasyonlar: Güvenlik Denetimini İncele** onu geri getirir.

Yeni bir **adrese** güvendikten sonra entegrasyonun formu, token alanı odaklanmış ve "Token'ı yeniden girmeniz önerilir." ipucuyla açılır: kayıtlı token oraya asla gönderilmedi. Formda, bilerek değiştirilen bir adres, onun için yazılan bir token'la birlikte kabul edilir; token olmadan form bir tane ister.

Formda ayrıca iki anahtar vardır. **Sıkı sertifika sabitleme** (bir https adresi için) bağlandığınızda sunucunun sunduğu sertifika anahtarını sabitler. **Düz http'ye izin ver (şifrelenmemiş)** yalnızca bu makine olmayan bir `http://` adresi için sunulur ve token'ın açık metin olarak iletildiğini söyleyen kırmızı bir uyarı taşır. Bağlı kart sabitlenen parmak izini gösterir.

### Token ne yapabilir ve ne zamana kadar

Bağlanmak ayrıca barındırıcıya token'ın neler yapabildiğini sorar. GitHub (klasik token'lar), GitLab ve Bitbucket (kapsamları gönderdiğinde) kapsamlarını bildirir: kart, GitPoulpi'nin kullandığının ötesindeki kapsamları listeleyen sarı bir "Bu token, GitPoulpi'nin ihtiyaç duyduğundan fazla yetkiye sahip" satırı gösterir; hassas olanlar (`delete_repo`, `admin:org`, `sudo` gibi) kırmızıyla, son kullanma tarihli ince ayrıntılı bir token tercih etme öğüdüyle birlikte. Klasik bir GitHub token'ı (`ghp_...`) kapsamları tam olsa bile bu öğüdü alır. İnce ayrıntılı bir GitHub token'ının izinleri sorulamaz, bu yüzden yalnızca son kullanma tarihi gösterilir. Barındırıcı söylediğinde kart son kullanma tarihini gösterir. Son kullanma tarihi olmayan bir token hiçbir hatırlatma göstermez ve diğer ürünler kapsam bildirmez ve uyarı çıkarmaz. Aynı alanlar `hosting integrations login` ve `test` JSON çıktısındadır (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Bağlandığınızda okunurlar, bu yüzden kart onları bağlanan oturum için gösterir.

## 4. Bir depoyu bağlayın

Bağlama açıktır ve depo başınadır. Bir profil bağlı olduğunda Refs kenar çubuğu,
TAG'LER'den sonra bir **PULL REQUEST'LER** bölümü ve bir **ISSUE'LAR** bölümü
gösterir. Her bölümün iki açılır listesi vardır: sağlayıcı ve proje.

- Sağlayıcı açılır listesi, deponun uzak depolarıyla eşleşen hesapları
  (önce `origin`) `owner/name · origin · GitHub` gibi satırlar olarak
  **önerir**. Tek bir seçim hem sağlayıcıyı hem projeyi ayarlar.
- Hiçbir şey kendiliğinden bağlanmaz. Jira, Trello, Linear, YouTrack ve Redmine asla önerilmez, çünkü
  hiçbir uzak depo onları anmaz: kendiniz seçin.
- Açılır listenin en üstündeki **Yok (bu deponun bağlantısını kaldır)**
  bölümün bağlantısını kaldırır. Hiçbir şey onu kendiliğinden yeniden bağlamaz.
- Bağlantı, deponun `.gitpoulpi/repo-state.toml` dosyasına kaydedilir.

İki bölüm **ayrı ayrı** bağlanır. Örneğin pull request'ler Azure DevOps'tan,
issue'lar Jira'dan gelebilir.

| Sağlayıcı | PULL REQUEST'LER projesi | ISSUE'LAR projesi |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | aynısı |
| Azure DevOps | `Project/Repo` (kuruluş profilden gelir) | `Project` (iş öğeleri) |
| Gitea, Forgejo | `owner/name` | aynısı |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | yok: Jira bağlayın |
| Jira | yok | proje anahtarı, örneğin `ABC` |
| Trello | yok | bir pano |
| Linear | yok | bir takım anahtarı, örneğin `ENG` (takımlar listelenir) |
| YouTrack | yok | bir proje kısa adı, örneğin `ABC` |
| Gerrit | Gerrit proje adı, örneğin `team/tool` | yok |
| Redmine | yok | bir proje tanımlayıcısı, örneğin `bench` (projenin adresinin son kısmı) |

ISSUE'LAR proje listesinde arama yapılabilir. Listenin göstermediği bir projeyi
de yazabilirsiniz.

**Ek sorgu.** Bir issue kaynağının, proje açılır listesinin altında isteğe bağlı
bir filtre satırı vardır. Projeyle birleştirilir (AND):

- Jira: bir **JQL** parçası (`component = UI AND labels = backend`).
- Azure DevOps: bir **WIQL** koşulu (`[System.AssignedTo] = @Me`).
- YouTrack: bir **YouTrack sorgusu** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Parantez içine alınır, bu yüzden bir `or` projeyi genişletemez. Sizin bir
  `sort by:` ifadeniz varsayılan `sort by: updated desc` yerine geçer.
- Redmine: yok (bölümün durum ve "bana atananlar" filtreleri yeter).
- Linear: yok (bölümün durum ve "bana atananlar" filtreleri yeter).
- Trello: gösterilecek **listeleri** işaretlediğiniz bir açılır liste. Hiçbiri
  işaretli değilse her açık liste gösterilir.
- GitHub ve GitLab bunu yok sayar.

`Enter` yazılan sorguyu korur, `Esc` bırakır. Palet: *Issue'lar: Sorguyu /
listeleri düzenle...*.

CLI'dan: `gitpoulpi hosting link pulls --id <profile>` (veya `issues`),
bağlantıyı kaldırmak için `--clear`, ek sorgu için `--filter`. Bkz.
[CLI başvurusu](cli-reference.md#komutlar).

### Bağlı bir hesaptan klonlama

Bağlı bir profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit), **Klonla** iletişim kutusunda da bir kaynaktır (Ana sekmedeki **Klonla** düğmesi veya Depo Yönetimi sekmesinde `c`). Her profil, **URL ile klonla** altında kendi kaynağı olarak listelenir (`GitHub · name`, `GitLab · name`); GitHub ve GitLab hiçbir zaman karıştırılmaz. Birini seçtiğinizde depoları yüklenir ve sahibine göre gruplanır: önce kendi hesabınız, sonra her kuruluş veya grup için bir bölüm (GitLab alt grubu için `quarry/tools`), her biri sayısını gösteren bir başlığın altında. Kilit simgesi özel bir depoyu işaretler.

![Bağlı bir GitHub hesabındaki Klonla iletişim kutusu: sahip başına bir bölüm ve özel bir depoda kilit](../images/clone-hosted.png)

Arama kutusu tüm bölümlerde filtreler (ad, tam yol ve açıklama); eşleşmesi olmayan bölüm kaybolur. Bir satırda `Enter`, klonu o deponun HTTPS URL'si ve bir klasör adıyla doldurur.

![Aramanın listeyi tek bir kuruluşa daralttığı Klonla iletişim kutusu](../images/clone-hosted-search.png)

Daha önce açtığınız bir profilin listesi, barındırıcıya yeniden sorulurken kayıtlı bir kopyadan hemen görünür (etiketin yanında *yenileniyor...*); yanıt geldiğinde yeni depolar belirir, kaybolanlar listeden çıkar ve vurgulanan ile seçilen depo yerinde kalır. Yenileme başarısız olursa kayıtlı liste bir notla birlikte kalır, nedeni Etkinlik Günlüğü'ndedir.

Barındırıcı token'ı reddederse iletişim kutusu, listenin yerine nedeniyle birlikte *depolar listelenemedi* mesajını gösterir (ve kayıtlı kopya silinir); diğer kaynaklar çalışmaya devam eder.

## 5. Kenar çubuğundaki pull request'ler ve issue'lar

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

### Bir satırdaki eylemler

- `Enter` veya çift tık, satırı tarayıcınızda açar.
- Sağ tık veya `.`, satır menüsünü açar:
  - **Tarayıcıda aç**, **URL'yi kopyala**, **Numarayı kopyala**.
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
| `Enter` / çift tık | satırı tarayıcıda aç |
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

## 5b. Issue panosu

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

## 6. Pull request oluşturun

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

## 7. Issue oluşturun

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

## 8. Barındırıcının kendi web sayfasında oluşturun

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

## 9. Araç çubuğu çipi

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

## 10. Listeler ne zaman yenilenir

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

## 11. Terminoloji

GitHub, Azure DevOps ve Bitbucket "pull request" der. GitLab "merge request"
der. *İnceleme isteği terimi* (`ui.pr_terminology`, Ayarlar > Entegrasyonlar'da)
her etiket, menü, palet komutu ve mesaj için sözcüğü seçer:

- **Otomatik (barındırıcıya göre)**: deponun bağlı sağlayıcısını izler (hiçbir şey
  bağlı değilken "Pull Request").
- **Pull Request** veya **Merge Request**: her zaman o sözcük.

Değişiklik hemen uygulanır. CLI ve MCP her zaman "pull/merge request" der. Bu
sayfa "pull request" der.

## 12. Commit mesajlarındaki ve dal adlarındaki referanslar

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

### Bir issue referansı ekleyin

Commit composer'da, Özet'in sağ ucundaki `#▾` düğmesi, bağlı issue'ların
filtrelenebilir bir listesini açar. Geçerli dalın adlandırdığı issue *(daldan)* ile
işaretlenmiş olarak en başta gelir. `Enter` veya bir tık, referansını (`ABC-123`,
`#12`, `AB#12`) imlecin olduğu yere ekler. Klavye: composer'da `Alt+R`. Palet:
*Commit: Issue referansı ekle...*. Bağlı bir issue listesi yoksa düğme gri olur
ve *önce kenar çubuğunda bir issue listesi bağlayın* der. GitPoulpi kendiliğinden
asla referans eklemez.

## 13. SSH anahtarları

### Profil başına SSH anahtarı

Bağlı bir profilin kartının altında bir **SSH anahtarı** bölümü vardır. Anahtar
yoksa şunlardan birini seçin:

- **Varsayılan anahtar**: ssh-agent'ınız veya `~/.ssh`. Hiçbir şey değişmez.
- **Yeni anahtar oluştur**: isteğe bağlı bir başlık (varsayılan
  `GitPoulpi <host> <date>`). **Genel anahtarı <host> üzerine ekle** işaretliyse
  GitPoulpi, barındırıcıda zaten yoksa onu yükler. Anahtar parolasız olarak
  `~/.ssh/gitpoulpi_<profile id>_ed25519` ve `.pub` dosyalarına yazılır.
  `ssh-keygen` gerekmez. Var olan bir dosyanın üzerine asla yazılmaz.
- **Özel anahtar**: bir yol yazın veya bir özel anahtar dosyası için **Gözat**'a
  basın, sonra **Bu anahtarı kullan**'a basın.

Bir anahtar bağlıyken bölüm `title (SHA256:...)` gösterir ve şunlar bulunur:

- **Kaldır**: anahtarı ayır ve GitPoulpi eklediyse barındırıcıda sil.
- **Sil**: aynısı ve ayrıca anahtar dosyalarını sil; iki yolu da listeleyen bir
  onaydan sonra.

### Tek bir depo için SSH anahtarı

Bir deponun belirli bir anahtarı kullanmasını sağlamak için sekme menüsünü
açın (`Ctrl+T` veya sekmeye sağ tık) ve **SSH ve kimlik bilgileri...**'ni
seçin ya da *Depo: SSH ve Kimlik Bilgileri...* palet komutunu çalıştırın, sonra
SSH altında **Anahtar dosyası**'nı (veya bir entegrasyon profilinin anahtarı için
**Hesap anahtarı**'nı) seçin. Seçim GitPoulpi'nin özel klasöründe tutulur:
depoya veya git yapılandırmasına hiçbir şey yazılmaz. Bkz.
[Ayarlar](settings.md#depoya-özel-ssh-ve-kimlik-bilgileri).

Hâlâ kendi `core.sshCommand` değerine sahip bir depo, eski "Bu depo için SSH
anahtarı" sabitlemesinden kalan bu değeri o sayfada salt okunur gösterir,
**GitPoulpi'ye taşı** ile birlikte.

CLI ve MCP: `gitpoulpi repo-auth` (depo başına seçim) ve `core.sshCommand`
sabitlemesinin kendisi için `gitpoulpi ssh-command show | set --key <file> |
clear`.

Ayarlar > SSH ve kimlik bilgileri'ndeki genel anahtar (*Özel anahtar yolu* veya
*Anahtar çifti oluştur*) kendi seçimi olmayan her depoya uygulanır. Bir deponun
anahtar dosyası veya sabitlemesi onun önüne geçer.

## 13b. Geçmişi yeniden yazma için dal koruması

Geçmişi yeniden yazdığınızda ([yeniden yazma ekranı](history-rewrite.md)),
GitPoulpi barındırıcıya hangi dalları koruduğunu sorar ve bu dalları varsayılan
olarak yeniden yazmanın dışında bırakır. Yalnızca **bağladığınız** bir depo için
sorar (kenar çubuğunun pull request'ler veya issue'lar bölümü ya da sabitlenmiş
bir profil), o profilin token'ıyla; bunu okuyamayan bir token yalnızca
"bilinmiyor" sonucunu verir: hiçbir şey başarısız olmaz. Her sağlayıcının
yanıtı ve token'ın neye ihtiyaç duyduğu:

| Barındırıcı | Okunan | Token'ın ihtiyacı |
|---|---|---|
| GitHub (ve Enterprise Server) | dalların `protected` işareti ve etkin dal ruleset'leri (`~DEFAULT_BRANCH`, `~ALL` ve `refs/heads/...` desenleri) | `repo` yetkili klasik bir token (herkese açık bir depo için `public_repo`) veya Metadata (read) yetkili ayrıntılı bir token. Token'ın okuyamadığı bir ruleset belirtilir, klasik kurallar yine de sayılır |
| GitLab (gitlab.com ve kendi barındırdığınız) | `protected_branches` (adlar ve `*` desenleri) | `read_api` ve depo ayarlarını görebilen bir rol |
| Azure DevOps (Services ve Server) | deponun etkin, engelleyen dal ilkeleri (`Exact`, `Prefix` ve `DefaultBranch` kapsamları) | Code (read). "Force push" izni bir güvenlik ayarıdır, okunmaz |
| Bitbucket Cloud | push'u durduran dal kısıtlamaları (`push`, `force`, `delete`) | `repository:admin`. Dallanma modeli üzerindeki bir kısıtlama çözülmez |
| Bitbucket Data Center | force push'u durduran dal izinleri (`fast-forward-only`, `pull-request-only`, `read-only`) | depo yöneticisi |

Korumanın ayar sayfası (ve varsayılan dalın ve deponun silinmesinin ayar
sayfası) GitHub, GitLab, Azure DevOps ve Bitbucket için, depo bağlı olmasa bile
deponun uzak URL'sinden oluşturulur ve barındırıcı bir push'u reddettiğinde üç
çıkış yoluyla birlikte gösterilir.

## 13c. Radar'da hesap geneli listeler

Radar normalde klonladığınız depoları izler. Bir profil, `[integrations.radar]` tablosundaki üç anahtarla
(hepsi varsayılan olarak kapalı; bkz.
[yapılandırma başvurusu](config-reference.md#integrationsradar-hesap-geneli-listeler)) tüm barındırıcıda sizi ilgilendireni
de, klonlanmış olsun olmasın, listeleyebilir:

- **inceleme istekleri**: incelemenizi isteyen açık request'ler;
- **açık request'ler**: kendi ad alanınızın ve üyesi olduğunuz her org veya grubun ya da adlandırdığınız ad alanlarının
  her açık request'i. Asla tüm barındırıcı değil;
  profilin **Ad alanları** alanı (Ayarlar > Entegrasyonlar > Radar'da satır başına bir giriş, isteğe bağlı olarak
  düzenli ifade) üç listeyi de sınırlar: bir satır, bir deponun tam yoluna eşit olduğunda ya da bir `/` sınırında
  yol öneki olduğunda eşleşir (`ptm`, `ptm/produits/x`'i tutar, `ptmx/y`'yi tutmaz) ve barındırıcının döndürdüğü
  satırlara uygulanır, barındırıcının araması izin veriyorsa ayrıca sunucu tarafında da (GitHub'ın `user:` /
  `repo:` niteleyicileri);
- **atanan issue'lar**: size atanan açık issue'lar.

GitHub, GitLab (kendi barındırdığınız dahil), Gitea, Forgejo, Azure DevOps (Services
ve Server), Bitbucket Data Center ve Gerrit yanıt verir. Her birine nasıl sorulduğu:

| Barındırıcı | Request'ler | Issue'lar |
|---|---|---|
| Azure DevOps | tüm kuruluşun etkin pull request'leri (sizi bekleyenler için `searchCriteria.reviewerId`; oy verdiğiniz bir request dışarıda bırakılır); sunucu kuruluş düzeyindeki yolu reddederse ilk 20 projeye tek tek sorulur. `namespaces` proje adlarıdır | size atanan, Done, Closed veya Removed olmayan iş öğeleri |
| Bitbucket Data Center | panonuz: inceleme istekleri için henüz onaylanmamış `REVIEWER` rolü, açık request'ler için `AUTHOR` ve `PARTICIPANT`; `namespaces` proje anahtarlarıdır | yok (Data Center'ın issue takipçisi yoktur; bir not bunu söyler) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; açık request'ler ad alanı başına (bir proje yolu öneki) `is:open projects:<önek>`, hiçbiri adlandırılmamışsa kendi ve incelediğiniz değişiklikler; devam eden bir değişiklik taslaktır | yok |

Bitbucket Cloud'un hesap geneli satırları yoktur: depolar arası request listeleyen bir uç noktası yoktur ve
her depoya sormak çok fazla çağrı olurdu. Klonlanmış depoları kendi listelerini korur ve `gitpoulpi hosting
radar --id <bitbucket>` bunu söyler. Gerrit'in döndürdüğü sınırdan fazla değişikliği olduğunda liste,
bir notla kesilmiş olarak işaretlenir.

Taslaklar, tutmadıkça dışarıda bırakılır. Her liste sınırlıdır: tür başına 300 satır, kaç tane daha olduğunu söyleyen bir
notla; GitHub'da ise tur başına birkaç arama çağrısı, çünkü arama API'si dakikada yalnızca 30'a izin verir. Token'ın
okuyamadığı bir ad alanı bir not olur ve diğerleri yine yüklenir; reddedilen bir token ya da bir
hız sınırı yalnızca o profili başarısız kılar. Yanıtlar beş dakika yeniden kullanılır
(yapılandırma klasöründeki `radar-account.json`).

Ayarlar > Entegrasyonlar'da bağlı bir kod barındırıcısı profilinin SSH anahtarının altında bir **Radar** bloğu vardır:
her liste için bir onay kutusu (*İncelememi bekleyen istekler*, *Açık her istek*, *Taslakları dahil et*, *Bana atanan Issue'lar*,
sonuncusu issue'su olmayan barındırıcılarda yoktur), bir *Ad alanları* alanı (satır başına bir giriş: o grubu veya sahibi ve altındaki her şeyi eşleyen bir ad ya da
*Düzenli ifadeler* işaretliyse satır başına bir düzenli ifade; boş, kendi ve üyesi olduğunuz her ad alanı demektir) ve *Klonlanmamış bir depodan gelen istek* seçimi: *Sor*, *Klonla* veya *API üzerinden incele*.
Her şey değiştirdikçe kaydedilir. Hesap geneli sorgusu olmayan barındırıcılar (Bitbucket
Cloud) bunun yerine bunu söyleyen tek bir satır gösterir. Komut paleti bloğa
*Ayarlar: Bir profilin Radar seçenekleri...* ile ulaşır ve Ayarlar araması her
denetimi etiketiyle bulur.

Komut satırından: `gitpoulpi hosting integrations radar <id> --review-requests on`
bir listeyi açar, `gitpoulpi hosting radar --json` listeleri yazdırır (`--refresh`
önbelleği atlar). YZ ajanlarında MCP üzerinden aynı iki komut vardır
(`hosting_radar`, `hosting_integrations_radar`).

## 14. Komut satırından

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

Her komut için [CLI başvurusuna](cli-reference.md#komutlar) bakın. Bir token asla
argüman olarak kabul edilmez ve MCP sunucusu `hosting login` sunmaz, bu yüzden bir
token asla bir ajanın içinden geçmez. `hosting login`, token'ı işletim sisteminin kimlik bilgisi deposu tutuyorsa `stored: "os"`, depo reddedip token bu oturum için bellekte duruyorsa `stored: "session"` ve `store_error` yazdırır.

## Sorun giderme

| Belirti | Çözüm |
|---|---|
| *Bağlan* bir kimlik doğrulama hatasıyla başarısız oluyor | Token yanlış, süresi dolmuş veya iptal edilmiş. Yenisini oluşturun. |
| Bir kapsam kırmızıyla listeleniyor | Token'da o kapsam yok. Bağlantı çalışır, ama o özellik çalışmaz. Kapsamı barındırıcıda ekleyin, sonra **Düzenle**'ye basıp yeni token'ı yapıştırın. |
| Kenar çubuğunda PULL REQUEST'LER bölümü yok | Etkin hiçbir profil bağlı değil. Birini bağlayın, sonra depoyu bölümün açılır listesinde bağlayın. |
| Bir listede *Yeniden bağlan* satırı | Token kayboldu (kimlik bilgisi yardımcısı olmadan bir yeniden başlatma) ya da süresi doldu. Profili açmak için satıra tıklayın. |
| Kendi kurduğunuz bir örnek başarısız oluyor | **Sunucu Alan Adı**'nda yol olmadığını ve bilgisayarınızın örneğin sertifikasına güvendiğini denetleyin. |

## Bilinen sınırlar

- **Fork'lardan pull request'ler:** Azure DevOps ve Bitbucket bir fork'tan gelen
  isteği checkout edemez (bkz. [Bir satırdaki eylemler](#bir-satırdaki-eylemler)).
  Azure DevOps bir fork'tan pull request oluşturamaz.
- **Trello** bir panonun en çok ilk 1000 kartını gösterir. Daraltmak için liste
  filtresini kullanın. **Jira**, **Linear** ve **YouTrack** her yenilemede en çok 500
  issue listeler. Daraltmak için bir JQL parçası veya bir YouTrack sorgusu kullanın.
- **Linear** issue başına tek bir atanan alır: seçicide birkaçını seçerseniz ilki
  gönderilir. "Kilometre taşı" takımın **döngüsüdür** (proje kilometre taşları
  kullanılmaz). **YouTrack**'inki `Fix versions` alanıdır; bir sürümün numarası
  yoktur, bu yüzden seçici yayımlanmamış olanları projedeki konumlarına göre
  numaralandırır. İki takipçinin de pull request'i yoktur.
- **Redmine** issue başına tek bir atanan alır (seçicideki ilki), etiketi yoktur ve
  her yenilemede en çok 500 issue listeler. Atananlar projenin üyeleridir (kişiler ve
  gruplar) ve görünen adlarıyla ayırt edilir. Kanban panosunun sütunları issue
  durumlarıdır ve bir kartı taşımak durumu değiştirir; Redmine'ın varsayılan iş akışı
  kuralları uygulanır (o geçişi yapamayan bir rol reddi geri alır ve kart yerine geri
  oturur). "Resolved", Redmine'ın varsayılan verilerinde kapalı bir durum değildir:
  Kapalı sütunu "Closed" ve "Rejected" durumlarıdır.
- **Bitbucket Data Center**'ın yeni pull request'ler, dallar veya commit'ler için
  tarayıcı bağlantıları yoktur. Pull request sayfaları normal açılır.
- **Azure DevOps** yazar avatarı göstermez. "Benim" ve "inceleme istenen"
  filtreleri, kişisel erişim token'larını reddedebilecek bir uç nokta gerektirir.
  Bu durumda profil kullanıcı adı olmadan bağlanır ve bu iki filtre kullanılamaz.
  Geçersiz bir token hata yerine bir oturum açma sayfası alır; GitPoulpi bunu
  reddedilmiş token olarak bildirir.
- **Gitea ve Forgejo:** bir taslak, başlığı `WIP:` ile başlayan bir istektir (sunucuda
  oluştururken bir taslak bayrağı yoktur). Bir istek, zaten inceleyen herkesi inceleyen
  olarak listeler ve bir inceleme isteğini geri çekmek yalnızca bekleyenleri kaldırır.
  Bir fork'un yerel dalındaki pull request'ler, GitHub gibi `refs/pull/<n>/head` üzerinden
  checkout edilir. Bahsetmeler sunucunun `mentioned_by` filtresiyle aranır. Sunucu token
  kapsamlarını göstermez, bu yüzden eksik bir kapsam yalnızca bir çağrı reddedildiğinde bulunur.
- **Issue panosu** kartları her issue kaynağında taşır. Jira'da bir taşıma sütuna götüren iş akışı geçişini
  çalıştırır; alan isteyen bir geçiş (örneğin bir çözüm) Jira tarafından reddedilir ve kart mesajıyla geri gider.
  Azure DevOps'ta hedef state'i olmayan bir work item türü nedeniyle birlikte geri gider.
- **Bitbucket:** uygulama parolaları artık yoktur; bir API token'ı kullanın.
  Depolar çalışma alanı çalışma alanı listelenir.
- **Jira Cloud:** yalnızca klasik API token'ları çalışır (kapsamlı token yok,
  OAuth yok). Yeni bir Jira issue'sunun açıklaması düz metindir.
- Bir pull request, iş öğesi, issue veya kart oluşturmak barındırıcıda gerçekleşir
  ve GitPoulpi'den **geri alınamaz**. Bir isteğin checkout'u normal şekilde geri
  alınabilir.
