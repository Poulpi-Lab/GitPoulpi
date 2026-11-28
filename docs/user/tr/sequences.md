Bir **dizi**, birbiri ardına çalışan komutların adlandırılmış bir listesidir: örneğin
*Tüm değişiklikleri stage'le*, *YZ: Commit mesajı üret*, *Commit*, *Push* tek bir
**Yedekleme** tuşu olarak. Her adım [komut paletinin](keybindings.md) bir komutudur; bir dizi
kendi palet satırıdır (`Dizi: <ad>`), bir tuşa atanabilir ve
**Ayarlar > Diziler** içinde ya da `config.toml` dosyasında düzenlenir.

## Diziler sayfası

**Ayarlar > Diziler** bir [liste sayfasıdır](settings.md#listeler): solda diziler,
sağda seçili olanın satırları. Her değişiklik hemen uygulanır ve yazılır; palet
ve tuş haritası yeniden başlatmadan izler.

```
 Diziler
+- Diziler ---------------+  1/2. dizi düzenleniyor
| Bu kurulum              |  Ad           Backup
| > Backup        Alt+B   |  Kimlik       backup
|   Worktree ve ajan      |  Açıklama     Stage everything, ...
| Bu depo                 |  Kapsam       Bu kurulum
|   Release               |  Kısayol      [ Alt+B ] [ Temizle ]
+-------------------------+  Adımlar      Stage all > YZ: ... > Commit > Push
| [+ Ekle v] [Çoğalt] [Kaldır]|           [ Adımları düzenle (4) ] [ Şimdi çalıştır ]
| [Yukarı] [Aşağı]        |
+-------------------------+
```

* **+ Ekle**, **Boş dizi** ve üç hazır şablonu sunar: **Hazır şablondan ▸
  Yedekleme**, **Oluştur ve Push et** ve **Worktree ve ajan**. Hazır şablon bir
  başlangıç noktasıdır: siz ekleyene kadar hiçbir şey etkin değildir ve eklediğiniz
  düzenleyebileceğiniz bir kopyadır. Hazır şablonlar eklendiklerinde arayüz dilinde adlandırılır.
* **Çoğalt**, **Kaldır** (önce sorar), **Yukarı** / **Aşağı** ya da bir sürükleme listeyi
  yeniden sıralar (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). Listenin sırası
  `config.toml` içindeki sıradır.
* Sorunu olan bir satır kırmızıyla işaretlenir ve nedenini söyler (adım yok, bu sürümün
  bilmediği bir adım, ihtiyaç duyduğu seçeneği olmayan bir `Asla` duraklaması).
  Sorunlu bir dizi **kaydedilebilir**, gelecekteki bilinmeyen adımlar olduğu gibi korunur,
  ama düzelene kadar palet satırı gri kalır.

### Bir dizinin satırları

| Satır | Ne yapar |
|---|---|
| **Ad** | Paletin gösterdiği: `Dizi: <ad>`. |
| **Kimlik** | `config.toml` içinde, komut satırında ve kısayolun anahtarı olarak kullanılan kısa ad (`a-z`, `0-9`, `_`, `-`). Yeniden adlandırmak kısayolu da, aynı yazmada taşır. |
| **Açıklama** | İsteğe bağlı, kendiniz için. |
| **Kapsam** | Aşağıya bakın. |
| **Kısayol** | Tuşu gösteren bir düğme. `Enter` veya bir tık, **Klavye ve fare** ile aynı tuş yakalamayı açar: siz basarken çakışmalar ve terminal uyarıları görünür ve başka bir eylemin kullandığı bir tuş aynı biçimde reddedilir. **Temizle** onu kaldırır. Tuş `[keybindings.sequence] <id>` olarak kaydedilir. |
| **Adımlar** | Sırayla adımlar. **Adımları düzenle** onları açar. |
| **Şimdi çalıştır** | Diziyi geçerli pencerede çalıştırır. |

### Kapsam

Her dizinin kendisiyle kaydedilen bir kapsamı vardır:

| Kapsam | Nereye kaydedilir | Kim görür |
|---|---|---|
| **Bu kurulum** (varsayılan) | `config.toml` içinde `[[sequences]]`, sahibi olarak bu kurulumla | Yalnızca GitPoulpi'nin bu kurulumu. |
| **Bu bilgisayardaki tüm kurulumlar** | `config.toml` içinde `[[sequences]]`, sahipsiz | Dosyayı paylaşan her kurulum. |
| **Yalnızca bu depo** | Geçerli depoda `.gitpoulpi/sequences.toml` | Depoya sahip olan herkes. Yalnızca ona güvendikten sonra sayılır (aşağıya bakın). |

Kapsamı değiştirmek tanımı taşır: **Yalnızca bu depo**yu seçmek onu deponun dosyasına
yazar ve `config.toml` dosyasından kaldırır, tersi de öyle. Depo kapsamı için bir deponun açık olması gerekir.

### Depo dizileri ve güven

`.gitpoulpi/sequences.toml` dosyası commit'lenebilir, bu yüzden başkalarından gelebilir ve bir dizi
makinenizde komut çalıştırır. Bu nedenle bir deponun dizileri, **siz güvenene kadar sunulmaz**, tıpkı
commit'lenmiş bir iş akışı dosyasının komutları gibi.

Bir depoda kimsenin henüz güvenmediği böyle bir dosya olduğunda Log'un üzerinde bir uyarı belirir:
*Bu depo 2 dizi tanımlıyor*, **İncele**, **Güven** ve **Yok say** ile. Hiçbir şeyi engellemez.

- **İncele**, her diziyi her adımıyla, duraklamasıyla ve seçenekleriyle salt okunur olarak listeler. Oradaki
  **Güven** listelenene güvenir; **Kapat** (veya `Esc`) hiçbir şeye güvenmez. Liste pencereden uzunsa orada
  Güven sunulmaz (pencereyi büyütün): okuyamadığınıza asla güvenmezsiniz.
- **Güven** (düğme ya da paletteki *Dizi: Bu Deponun Dizilerine Güven*) dosyaya şu anki haliyle güvenir.
- **Yok say**, dosya değişene kadar uyarıyı gizler.
- Klavyeden: palette *Dizi: Bu Deponun Dizilerini İncele / Güven / Yok Say* vardır.

Kaydedilen şey, kimliklerin, komutların, duraklamaların ve seçeneklerin bir özetidir (digest); deponun yerel
durumunda tutulur (asla depoda değil). Bunlardan biri değişirse, örneğin bir pull'dan sonra, uyarı *Bu deponun
dizileri değişti* olarak geri gelir ve siz yeniden güvenene kadar diziler paletten ve tuşlardan çıkar. Bir diziyi
yeniden adlandırmak veya açıklamasını düzenlemek hiçbir şeyi değiştirmez. Ayarlar'da kendiniz yazdığınız bir dizi,
siz düzenlemeden önce dosyaya zaten güvenilmişse (veya boşsa) güvenilir kalır.

Güvenildikten sonra depo dizileri palette kendi dizileriniz gibi `Dizi: <ad>` olarak görünür ve Ayarlar'da
bir kısayol verilebilir. Sizinkilerden biriyle aynı kimlikli bir dizi bir çakışmadır: sizinki kazanır ve diğeri
yinelenen olarak bildirilir.

Komut satırından: `gitpoulpi sequence list` dosyanın dizilerini `trusted` ve bir `repo_trust` durumuyla
(`untrusted`, `changed`, `trusted`) gösterir; `gitpoulpi sequence trust` (ve `untrust`, başka bir depo için
`--repo PATH`) bir kişinin `sequence show` çıktısını okuduktan sonra çalıştırması içindir. `sequence run`
güvenilmeyen bir depo dizisini reddeder. YZ ajanları güvenemez: bunun için bir MCP aracı yoktur.

## Adımlar

**Adımları düzenle**, listeyi dizinin adımlarıyla (başlık **{ad} adımları** der) ve satırları seçili adımın
satırlarıyla değiştirir. **✕ Dizilere dön** (satırların üstündeki çarpı) ya da `Esc` dizilere döner.

* **+ Ekle**, **Adım ekle** penceresini açar: bir dizinin çalıştırabileceği her komut, palet adıyla
  (YZ olanlar parıltıyla), yazdıkça paletin eşleşmesiyle süzülür. Bu sürümün çalıştıramadığı bir adım nedeniyle
  gri görünür. `Enter` veya bir tık onu sona ekler; `Esc` veya çarpı açılır pencereyi kapatır.
* **Çoğalt**, **Kaldır**, **Yukarı** / **Aşağı** ve sürükleme dizilerdeki gibi çalışır.
* Yıkıcı bir komut (dal silme, değişiklikleri atma...) asla sunulmaz: bir dizi onu çalıştıramaz.

### Duraklat

Her adımın bir **Duraklat** ayarı vardır:

| Duraklat | Anlamı |
|---|---|
| **Varsayılan** | Adım yalnızca bir yanıt gerektiğinde, olağan iletişim kutusunda durur (bir dal adı, hâlâ boş olan bir commit mesajı...). |
| **Her zaman** | Bu adımdan önce durur ve devam etmeyi sorar. |
| **Asla** | Asla durmaz. Adımın seçenekleri ihtiyacı olan her şeyi vermelidir, bu yüzden bir ad soran adımın **Ad** seçeneğinin doldurulması gerekir. |

Çalışmadan önce her zaman soran bir adım (bir force push) **Asla** sunmaz: satır nedenini söyler.
YZ mesaj adımında **Asla**, açık bir seçim olarak serbesttir.

### Seçenekler

**Duraklat** altındaki satırlar adımın seçenekleridir: bir **Ad** veya **Mesaj** metni, bir **Dal**,
**Yeni sekmede aç** gibi bir onay kutusu ya da yapılandırmanızdan doldurulan bir açılır liste (bir **Remote**, bir
**YZ profili**, bir **Şurada aç girişi**). Metinler `${branch}`, `${date}`, `${time}` ve `${n}` değişkenlerini
kabul eder (adı benzersiz yapan bir sayı). Bir metni silmek seçeneği kaldırır.

## YZ ile oluşturma ve düzenleme

[YZ açıkken](ai.md#dizi-taslağı) (Ayarlar > YZ) iki giriş görünür:
**Ekle > YZ ile oluştur...** (ayrıca Ayarlar'ı bu sayfada iletişim kutusuyla açan `Dizi: YZ ile Oluştur...`
palet satırı) ve seçili dizi için **Şimdi çalıştır**ın yanındaki **YZ ile düzenle...**. YZ kapalıyken
hiçbiri yoktur.

```
+- YZ ile dizi oluştur ------------------------------------ x -+
| İstediğiniz diziyi açıklayın                                  |
| +-----------------------------------------------------------+ |
| | her şeyi stage'le, commit mesajını YZ ile yaz,            | |
| | commit et, sonra push et                                  | |
| +-----------------------------------------------------------+ |
| Enter satır ekler. Ctrl+Enter gönderir. Esc kapatır.          |
| [ ] Bağlamı gönder (uzak depo adları, YZ profilleri, ...)     |
|    Kapalı: yalnızca açıklamanız gönderilir.                   |
| YZ yalnızca önerir. Siz kabul etmeden hiçbir şey kaydedilmez  |
| [ Gönder ] [ Vazgeç ]                                         |
+---------------------------------------------------------------+
```

İstediğinizi kendi sözlerinizle yazarsınız; **Gönder** (veya `Ctrl+Enter`, `Ctrl+S`) modele, diğer her
YZ eylemiyle aynı profil, işler göstergesi ve `Esc` ile (ve aynı "Gönderilenler" sorusuyla) sorar. Yanıt,
herhangi bir şey olmadan önce okuduğunuz bir **öneridir**:

* ad, kimlik, açıklama ve duraklamaları ve seçenekleriyle adımlar;
* **YZ ile düzenle** için geçerli diziye karşı bir fark: `+` eklenen bir adım, `-` kaldırılan bir adım,
  `~` duraklaması veya seçenekleri değişen bir adım, bir alan için `~ Ad: "a" -> "b"`;
* **Dışarıda bırakılanlar**: modelin önerdiği ama izin verilmeyen bir adım (bilinmeyen, yıkıcı,
  bir dizinin içinde bir dizi, imkânsız bir duraklama), nedeniyle birlikte; geçerli adımlar korunur;
* modelin **notları** ve yalnızca gösterilen bir **önerilen kısayol**, tuşun zaten kullanılıp
  kullanılmadığıyla birlikte. Asla bağlanmaz: sonradan **Kısayol** düğmesini kullanın.

```
+- Önerilen değişiklik ------------------------------------ x -+
| + eklendi   - kaldırıldı   ~ değişti                          |
| ~ Ad: "Daily" -> "Daily 2"                                    |
| Adımlar (3)                                                   |
| + 1. Git: Fetch All Remotes                                   |
|   2. Git: Stage All Changes                                   |
| - Git: Commit...                                              |
| ~ 3. Git: Push: varsayılanlar -> Duraklat: Her zaman          |
| Dışarıda bırakılanlar                                         |
| 4. adım `reset_hard`: yıkıcı adımlara izin verilmez           |
| [ Kabul et ] [ Kaydetmeden önce düzenle ] [ Reddet ] [ Geri ] |
+---------------------------------------------------------------+
```

* **Kabul et** onu tek bir kayıtla yazar (yeni bir dizi ya da düzenlenenin yerine geçeni; kimliğini,
  kapsamını ve tuşunu korur) ve listede seçer.
* **Kaydetmeden önce düzenle** aynısını yapar ve satırları ayarlayabilmeniz için adımlarını açar (bu sayfa
  her düzenlemeyi hemen kaydeder, bu yüzden dizi zaten kaydedilmiştir).
* **Reddet** hiçbir şey yazmaz. **Geri** yeniden göndermek için açıklamaya döner.
* **Kabul et**, kabul edilecek bir şey olmadığında kullanılamaz: kullanılabilir adım yok, sayfanın
  işaretleyeceği bir sorun ya da hiçbir şeyi değiştirmeyen bir düzenleme.

**Kabul et**ten önce hiçbir şey yazılmaz ve buradan bir dizi asla çalıştırılmaz. İletişim kutusunu kapatmak
(çarpı, `Esc`, dışarı tıklama), bir açıklama yazılmışken veya bir öneri açıkken "Değişiklikler atılsın mı?" diye sorar;
Ayarlar'ı kapatmak da sorar.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<bu kurulumun kimliği>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Bu sürümün bilmediği anahtarlar ve adımlar, sayfa kaydettiğinde korunur.

## Bir diziyi çalıştırmak

Bir dizi, her biri paletin bir komutu olan adımların adlandırılmış bir listesidir (her şeyi stage'le, commit mesajını YZ ile yaz, commit et, push et...). Onu paletten (`Dizi: <ad>`), Ayarlar > Klavye ve fare içinde ona verdiğiniz tuşla (`tui.toml` içinde `[keybindings.sequence]`) ya da komut satırından (`gitpoulpi sequence run`) çalıştırın.

Çalışırken Log'un üstündeki bir şerit `<ad> k/n`, adımı ve neyi beklediğini gösterir. Çalıştırma ayrıca İşler panelinde `Dizi <ad> k/n: <adım>` olarak görünür. Aynı anda bir dizi çalışır; ikincisini başlatmak hangisinin çalıştığını söyler.

**Sormak için durduğu yer.** Her adımın bir `pause` ayarı vardır (`default`, `always`, `never`):

| Adım | Varsayılan | `pause = "never"` |
|---|---|---|
| YZ commit mesajı | mesaj mesaj alanına düşer; şerit "Mesajı inceleyin, sonra Devam" der | hemen devam eder |
| Commit | boş bir mesaj her zaman durur: tuş alanın elindedir ve commit'iniz (veya Devam) sürdürür | commit'lenecek bir mesaj yoksa kullanılamaz |
| YZ commit'leri oluştur | inceleme açılır; Uygula devam eder, kapatmak iptal eder | öneriyi olduğu gibi uygular |
| Worktree oluştur | iletişim kutusu ad doldurulmuş olarak açılır | iletişim kutusu olmadan `options.name` ile oluşturur |
| Push | `git.push_behavior = ask` olduğunda "Push..." seçicisi açılır | `options.remote` hedefine push eder |
| Lease ile force push, sır gibi görünen dosyaları stage'leme | her zaman sorar | izin verilmez |

Herhangi bir adımda `pause = "always"`, çalışmadan önce **Devam** ve **İptal** ile "Sıradaki: <adım>" gösterir. Dal ayrıştığı için push reddedilirse olağan iletişim kutusu açılır ve şerit "Push reddedildi: iletişim kutusunda seçin" der; pull sonra push (ya da lease ile force push) çalıştırmayı sürdürür, iletişim kutusunu kapatmak onu iptal eder.

**Devam**, şeridin düğmesi, paletteki `Dizi: Devam` ya da `sequence.continue` tuşudur. **İptal** şeridin kapatma çarpısı, `Dizi: İptal` (`sequence.cancel`), İşler paneli veya başka hiçbir şey istemediğinde `Esc`'dir (açık bir iletişim kutusu önce kapanır, bu da iptal eder). İptal etmek zaten çalışan adımları korur ve bunu söyler: "<ad>, k/n. adımdan (<adım>) sonra iptal edildi; önceki adımlar korunur".

**Bir adım başarısız olduğunda** çalıştırma orada durur: durum satırı diziyi, adımı ve hatayı adlandırır, şerit **Ayrıntılar** (adımın seçeneklerini listeleyen Etkinlik günlüğü girdisi) ve bir kapatma çarpısıyla kırmızıya döner ve önceki adımlar yapılmış kalır.

**Worktree'ler.** `open_tab = true` olan bir worktree oluşturma adımı çalıştırmayı yeni worktree'nin sekmesine taşır: sonraki adımlar (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) orada çalışır. Ekranda başka bir sekme varsa şerit **Oraya git** ile "<ad> sekmesi bekleniyor" der. O sekmeyi kapatmak çalıştırmayı iptal eder. Bir çalıştırma GitPoulpi'den çıkmaya dayanmaz.

### Bir çalıştırma öneren ajan

*YZ ajanlarının onaylamam için değişiklik önermesine izin ver* açıkken (`[mcp]
allow_ui_propose`), bir ajan pencerenizden bir dizi çalıştırmasını isteyebilir
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Bir banner belirir:
*✦ <ajan> önerdi: <ad> dizisini çalıştır*, **Çalıştır** ve **Yok say** ile.
Çalıştır'a basana kadar hiçbir şey çalışmaz (tıklayın ya da `Ctrl+N` sonra `Enter`);
Yok say, `Esc` (başka hiçbir şey istemediğinde) ya da paletteki *Ajan: Bekleyen Görünümü Kapat* onu bırakır.
Çalıştırma sonra tıpkı sizin başlattığınız gibi, dizinin dediği yerde duraklayarak gider.
Dizi yoksa, geçersizse ya da güvenmediğiniz bir depo dizisiyse öneri reddedilir.

## Bir çalıştırmayı geri almak

Depoyu değiştiren her adım (stage, commit, tag, dal, stash, worktree...) çalıştırmayla etiketlenmiş **kendi olağan Geri al girdisini** bırakır, bu yüzden `Ctrl+Z` hâlâ bir seferde bir adımı geri alır ve [Geri alma geçmişi](undo-redo.md#geri-alma-geçmişi) bunlarda `⟳ <ad>` gösterir. Tüm çalıştırmayı geri almak için **Tüm diziyi geri al** vardır: Geri alma geçmişinin düğmesi (`x`), `Düzenle: Tüm Diziyi Geri Al` palet satırı ya da `gitpoulpi undo --sequence`. Çalıştırmanın girdilerini en yeniden başlayarak, her biri kendi denetimleri ve soruları ile geri alır (zaten push edilmiş bir commit önce sorar), ilk redde adımı adlandırarak durur ve daha eskileri günlükte bırakır. YZ mesajı ve bir push girdi bırakmaz: bir push asla geri alınmaz. Yalnızca geçerli deponun günlüğüne dokunulur; bir worktree'de yapılan adımlar o depodan geri alınır. Yinele girdi başınadır. Ayrıntılar: [Dizi çalıştırmaları](undo-redo.md#dizi-çalıştırmaları).
