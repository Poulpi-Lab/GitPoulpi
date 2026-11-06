[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · **Türkçe** · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Dal iş akışları

Bir dal iş akışı, GitPoulpi'ye ekibinizin dalları nasıl kullandığını söyler: hangileri
sonsuza dek yaşar (`main`, `develop`, `production`), hangileri kısa ömürlü konu dallarıdır
(`feature/...`, `hotfix/...`), nasıl adlandırılırlar ve her biri nerede başlayıp nerede
biter. Özelleştirilebilir hale getirilmiş Git Flow'dur: sabit adlar, önekler ve sonekler
ya da yer tutuculu desenler; böylece bir botun issue'dan oluşturduğu dallar sizin
dallarınız gibi tanınır.

Bu sayfa modeli, dosyayı, hazır şablonları, algılama ve içe aktarmayı, salt okunur
komutları, dalları başlatan, bitiren, eşitleyen ve yükselten eylemleri (CLI, MCP ve
uygulama) ve uygulamanın ekranlarını anlatır: ayar düzenleyicisi, gruplanmış kenar
çubuğu, `Flow` menüsü ve iletişim kutuları.

## Kavramlar

- Bir **temel dal** uzun ömürlüdür. Bir temel dalın bir **üst dalı** olabilir (`develop`,
  `main` içine girer); ana hattın (trunk) yoktur. Bir temel dal birkaç aday ad
  listeleyebilir (`main` veya `master`, `develop` veya `dev`): hangisi varsa o kullanılır.
- Bir **konu türü**, kısa ömürlü bir dal çeşididir (feature, bugfix, release, hotfix,
  support...). Bir **ad deseni**, bir **üst dalı** (nerede başladığı ve nereye bitirildiği)
  ve bitirme yolu vardır.
- Her dal **sınıflandırılır**: bir temel dal (adına göre), bir türün konu dalı (desenine
  göre) veya sınıflandırılmamış.
- Bir **dalın üst dalı** sırasıyla şudur: GitPoulpi dalı oluştururken kaydedilen
  (`git config branch.<name>.gitpoulpi-parent`), yoksa varsa Git Town'un
  `git-town-branch.<name>.parent` değeri, yoksa türünün üst dalı.

## Nerede saklanır

| Ne | Nerede | Commit'lenir mi |
|---|---|---|
| Paylaşılan iş akışı | Depodaki `.gitpoulpi/workflow.toml` | Evet, sizin tarafınızdan: GitPoulpi onu yalnızca siz isteyince yazar |
| Kendi ayarlarınız | `.gitpoulpi/repo-state.toml` içindeki `[workflow]` | Hayır (o dosya yok sayılır) |

Durum klasörü kendini yok sayar (`.gitpoulpi/.gitignore`); commit'lenmesi amaçlanan
`workflow.toml` hariç. Etkin iş akışı, üzerine sizin geçersiz kılmanızın konduğu paylaşılan
dosyadır: aynı adlı bir temel dal veya aynı kimlikli bir konu türü değiştirilir; yenileri
eklenir; `disable_bases` ve `disable_topics` paylaşılan olanları yalnızca sizin için kaldırır.

Var olan bir dosyanın üzerine kaydetmek, bu sürümün tanımadığı anahtarları (daha yeni bir
GitPoulpi'ninkileri, bir ekip arkadaşınınkileri) korur ama yorumları ve anahtar sırasını korumaz.

## Dosya

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (bilgi amaçlı)

[[base]]
name = "main"
names = ["master"]             # aynı dalın diğer yazımları
protected = true               # asla silinmez, doğrudan commit'ler şüphelidir

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # bir ortam dalı (GitLab Flow): akış yönünde yükseltilir

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # ayrıca eşleşen adlar; asla oluşturmak için kullanılmaz
# regex = "^feat-(?P<slug>.+)$"  # ayrıca eşleşir; adlandırılmış gruplar alan olur
parent = "develop"             # nereden başladığı ve varsayılan bitirme hedefi
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # isteğe bağlı kabuk komutları, çalışmadan önce hep gösterilir
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # barındırıcı tarafından yürütülen akışlar, aşağıya bakın
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` ve `hooks` aşağıdaki [eylemleri](#eylemler) yönlendirir;
`issue_flow` [barındırıcı tarafından yürütülen akışları](#barındırıcı-tarafından-yürütülen-akışlar)
yönlendirir.

### Ad desenleri

`{yer tutucular}` içeren metin:

| Yer tutucu | Neyle eşleşir | Neyden oluşturulur |
|---|---|---|
| `{slug}` | serbest metin; sonda ise `/` içerebilir | metin, küçük harfe çevrilmiş, aksanlar sadeleştirilmiş, sözcükler arasında `-`, en çok 50 karakter |
| `{issue}` | rakamlar (`123`) | `123` veya `#123` |
| `{issue_key}` | bir takipçi anahtarı (`PROJ-123`) | verildiği gibi |
| `{user}` | `/` içermeyen bir ad | slug haline getirilmiş |
| `{date}` | `2026-10-16` veya `20261016` | verildiği gibi |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | verildiği gibi |
| `{n}` | rakamlar | verildiği gibi |

Geri kalan her şey olduğu gibidir; yani önekler ve sonekler düz metindir:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Her yer tutucu bir kez
geçebilir ve iki yer tutucunun arasında metin olmalıdır. Bir desen geçerli dal adları üretmelidir
(`git check-ref-format` kuralları): boşluk yok, `~ ^ : ? * [ \` yok, `..` yok, `.` ile başlayan
veya `.lock` ile biten bileşen yok.

Bir `regex` yalnızca eşleştirme içindir (iki uçtan da sabitlenmiştir); hiçbir desenin
ifade edemediği adlar için kullanın. Adlandırılmış gruplar dalın alanları olur.

### Hangi konu türü kazanır

1. Bir temel dal gibi adlandırılmış bir dal (adlarından herhangi biri) o temel daldır.
2. Aksi halde eşleşen her türün her deseni bir adaydır: `pattern`, sonra `patterns`, sonra
   `regex`, tür tür dosya sırasıyla. En çok sabit karakteri belirleyen kazanır
   (`release/{version}`, `{user}/{slug}` desenini, o da yalın `{slug}` desenini yener); bir
   regex sabit önekini sayar. Beraberlikte dosyada önce gelen kazanır.
3. Aksi halde dal sınıflandırılmamıştır.

Bozuk bir desen veya regex uyarı olarak bildirilir ve atlanır; geri kalanı çalışmaya devam eder.

## Şablonlar

| Şablon | Temel dallar | Konu türleri |
|---|---|---|
| `git-flow` | `main` veya `master`; `develop` veya `dev` (üst dalı main) | `feature/` ve `bugfix/` (develop'tan), `release/` (develop'tan, main ve develop içine, tag'li), `hotfix/` (main'den, main ve develop içine, tag'li), `support/` (main'den); çoğul yazımlar (`features/`, `releases/`...) da eşleşir |
| `github-flow` | `main` | `feature/` veya herhangi bir ad, main'den, main içine merge request, squash, sonrasında silinir |
| `gitlab-flow` | `main`; `pre-production` ve `production` (ortam dalları, main, pre-production, production zinciri) | `feature/` veya herhangi bir ad main'den; cherry-pick hedefi olarak `release/{version}` |
| `trunk` | `main` | main'den kısa ömürlü `{user}/{slug}`, rebase'li |
| `custom` | yok | yok |

Ortam dalları ayrıca `preprod`, `staging` ve `prod` olarak da eşleşir. Bir şablon bir
başlangıç noktasıdır: `flow config init` ile yazdırın, dosyayı düzenleyin.

## Algılama ve içe aktarma

`gitpoulpi flow detect`, deponun sahip olduğu dallara göre şablonları 100 üzerinden bir puanla
ve bulunan somut adlarla (`main` için `master`, `develop` için `dev`, `feature/` için
`features/`) sıralar. Temel dallar 60 puan, şablonun özel desenlerinin tanıdığı diğer
dalların oranı 40 puan eder; başka bir şablonun uzun ömürlü bir dalı (bir GitHub Flow
adayında bir `develop`) 25 puan düşürür.

`gitpoulpi flow import --from git-flow` klasik git-flow ayarlarını (`gitflow.branch.master`,
`gitflow.branch.develop`, `gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`)
deponun kendi `.git/config` dosyasından okur; `--from git-town` `git-town.main-branch` ve
`git-town.perennial-branches` değerlerini okur (her dalın üst dalı,
`git-town-branch.<name>.parent`, kaydedilmiş üst dal olarak alınır). Hiçbir şey sessizce
uygulanmaz: komut TOML'u yazdırır, `--write` yalnızca yoksa `.gitpoulpi/workflow.toml`
dosyasını oluşturur.

## Komutlar

Hepsi `--json` alır; MCP sunucusu bunları araç olarak sunar (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` okur; eylemler aşağıda anlatılır).

| Komut | Ne yapar |
|---|---|
| `gitpoulpi flow status` | Her yerel ve uzak dal: tür (temel, konu, sınıflandırılmamış), konu türü ve alanları, üst dal ve nereden geldiği (`recorded`, `git_town`, `type`, `base`) ve yerel dallar için üst dalının kaç commit `ahead` ve `behind` olduğu. Ayrıca tanımda bulunan uyarılar, sıralı algılama ve içe aktarılabilecek başka araçların ayarları. |
| `gitpoulpi flow templates` | Yukarıdaki hazır şablonlar, tam tanımlarıyla. Depo gerektirmez. |
| `gitpoulpi flow detect` | Bulunan temel dallar ve konu türleriyle sıralı şablonlar. |
| `gitpoulpi flow config [--show-effective]` | Paylaşılan dosya (yolu, var olup olmadığı, ayrıştırılmış iş akışı, bilinmeyen anahtarlar) veya `--show-effective` ile dallarınızın sınıflandırıldığı iş akışı. `flow config show` aynıdır. |
| `gitpoulpi flow config init --template <t> [--write]` | Bir şablonun TOML'unu yazdırır; `--write` `.gitpoulpi/workflow.toml` dosyasını oluşturur ve var olanın üzerine yazmayı reddeder. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Yukarıda anlatıldığı gibi. |

Ahead/behind sayıları çağrı başına en çok 300 yerel dal için hesaplanır.

Okunamayan bir iş akışı dosyası (bozuk TOML, yanlış türde bir değer) dosyayı adlandıran bir
hatadır; asla sessizce yok sayılmaz.

## Eylemler

`start`, `finish`, `sync`, `promote` ve `set-parent` depo üzerinde çalışır. Her biri önce
**plan yapar**: yapacağı her git adımını, her birinin karşılık geldiği git komutuyla, uyarıları,
nelerin geri alınamayacağını ve onu engelleyen her şeyi listeler. Siz onaylayana kadar hiçbir
şey değişmez. Her çalıştırma **geri alma geçmişinde tek bir girdidir** (`gitpoulpi undo`
dalları, tag'i, silinen dalı, kaydedilen üst dalı ve checkout'u birlikte geri alır).

| Komut | Ne yapar |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Dalı türün deseninden adlandırır (`flow start feature "Login form"` `feature/login-form` yapar; `flow start release version=1.2.0` `release/1.2.0` yapar; yalın bir sözcük `slug` olur; `user` ve `date` varsayılan olarak siz ve bugündür), türün üst dalından veya `--from` değerinden (bir dal, bir uzak dal veya bir tag) oluşturur, checkout eder ve üst dalını kaydeder. `--publish` onu `-u` ile push eder (`undo` bunu geri almaz). |
| `gitpoulpi flow finish [<branch>]` | Türün her `finish` hedefi için sırayla: hedefi checkout eder ve dalı hedefin stratejisiyle merge eder (`merge`: her zaman bir merge commit'i; `squash`: `--title` başlıklı veya dal adından türetilmiş tek bir commit; `rebase`: dalı hedef üzerine rebase eder, sonra fast-forward yapar; `fast-forward`), hedefin bir tag'i varsa onu oluşturur (`tag = "v{version}"`, açıklamalı), sonra üst dalı checkout eder ve tür öyle diyorsa yerel dalı siler (`--delete` / `--keep` bunu geçersiz kılar; `--delete-remote` onu uzak depoda da siler, geri alınamaz). `finish_mode = "merge-request"` ile (veya `--mode merge-request`) yalnızca dalı push eder ve üst dalına pull veya merge request açar; `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` ve `--integration` ile (geri alınamaz; yerelde hiçbir şey merge edilmez veya silinmez). `--mode local-merge` bir `merge-request` türünü geçersiz kılar ve `ask` diyen bir tür ikisinden birini ister. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Üst dalın commit'lerini dala getirir: bir merge veya dalın stratejisi `rebase` ise (veya `--rebase`) bir rebase. `--stack` ile ayrıca üst dalı bu olan dalları (`set-parent` ile kaydedilmiş), özyinelemeli olarak, sırayla. Bulunduğunuz dala geri döner. |
| `gitpoulpi flow promote <env-branch>` | GitLab Flow ortam dalları: akış yönünde üstündeki dalı (`main`'i `pre-production` içine, `pre-production`'ı `production` içine) ona tek adımda merge eder ve geri döner. Yapabildiğinde fast-forward yapar. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | `branch.<name>.gitpoulpi-parent` değerini yazar (veya kaldırır). Döngüyü reddeder. Geri alınabilir. |

Başka bir yerden başlatılan bir dal orada biter: kaydedilmiş üst dal (`--from` veya
`set-parent` ile) türünkinden farklıysa, ilk bitirme hedefi kaydedilmiş üst daldır.

### Onaylama, planlama, betik yazma

| Bayrak | Anlamı |
|---|---|
| `--dry-run` | Planı JSON olarak (`{"plan": ...}`) yazdırır ve durur. Bir şey engelliyorsa çıkış kodu 1. |
| `--yes` | Sormadan çalıştırır. |
| ikisi de yok | Bir terminalde: plan gösterilir ve size sorulur. Aksi halde (bir betik, `--json`, bir ajan) plan `{"needs_confirmation": {"plan": ...}}` olarak, çıkış kodu 3 ile ve hiçbir şey değişmeden döner. `--publish` veya hook'suz `start`, `set-parent` ve `--abort` onay gerektirmez. |
| `--run-hooks` / `--no-hooks` | Bkz. [hook'lar](#hooklar). |
| `--autostash` | `general.auto_stash` ne derse desin, bir checkout veya merge'ü engelleyen yerel değişiklikleri stash eder ve sonra geri koyar. |

Engellenmiş bir plan nedenini söyler: bir konu dalı gerekirken bir temel dal, zaten var olan bir ad,
eksik bir alan (`release/{version}` için `version=...` gerekir), var olan bir tag, uzak deposunun
gerisinde kalmış bir hedef dal (`--allow-behind` yine de devam eder), süren bir merge veya
rebase, ayarlanmamış bir iş akışı.

**Auto-stash** ayarıyla (`general.auto_stash`), bir checkout veya merge'ü engelleyen çalışma
değişiklikleri, sıradan bir checkout'ta olduğu gibi eylemin etrafında stash edilir ve geri konur;
`never` ile eylem hiçbir şeyi değiştirmeden durur.

### Çakışmalar

Çakışmayla duran bir merge veya rebase normal çakışma durumunu (çakışma banner'ı,
`gitpoulpi conflicts ...`) bırakır ve akış hatırlanır. Çıkış kodu 2, `status: "conflicted"`,
yollar `conflicts` içinde. Dosyaları çözün ve çözüldü olarak işaretleyin, sonra:

- `gitpoulpi flow finish --continue` merge'ü commit'ler (veya rebase'e devam eder ya da
  squash'ı mesajıyla commit'ler) ve kalan adımları çalıştırır;
- `gitpoulpi flow finish --abort` merge veya rebase'i iptal eder, akışın başladığı dala geri
  döner ve gerisini unutur. Daha önce tamamlanan adımlar (çift bitirmenin ilk hedefi, tag'iyle
  birlikte) kalır: `undo` onları geri alır.

Başarısız bir hook'un durdurduğu bir akış aynı şekilde sürdürülür (`--continue` o adımı yeniden
çalıştırır). Çakışan çalıştırma ve devamı iki geri alma girdisidir.

### Merge request'ler

Daemon dalı push eder; isteği açmak barındırma token'ınızı gerektirir, bu yüzden CLI bunu
`gitpoulpi hosting pr` ile aynı kodla yapar (uzak depoyla eşleşen bir entegrasyon, yoksa
barındırıcının önceden doldurulmuş web sayfası, URL olarak yazdırılır). Sonuç `merge_request`
(istenen) ve `opened` (barındırıcının yanıtı) taşır; dal push edilmiş ama istek açılamamışsa
`open_error` taşır.

### Hook'lar

İş akışı dosyasındaki `[hooks]` kabuk komutlarını listeler (`after_start`, `before_finish`).
Ekibinizin paylaştığı bir dosyadan gelirler, bu yüzden **siz istemedikçe asla çalışmazlar**:
`--run-hooks` veya bir terminalin onları listeledikten sonra sorduğu soruya evet. Plan onları
her zaman listeler; çalışmayacaklarsa `hooks_run: false` ile. Bir hook, çalışma klasörü depo
olan bir kabukta, `GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` ayarlı olarak çalışır;
tam komutu, çıkış kodu ve tüm çıktısı sonuçtadır. Sıfırdan farklı kodla çıkan bir hook akışı
orada durdurur (bir `before_finish` hook'u hiçbir şey değişmeden önce çalışır, bu yüzden düz
bir hatadır; bir `after_start` hook'undan sonra dal vardır ve akışa devam edilebilir veya iptal
edilebilir). `--no-hooks` onları sormadan atlar. Bir hook her şeyi yapabilir, bu yüzden `undo`
yalnızca git değişikliklerini geri alır.

**Güven.** Commit'lenmiş bir `[hooks]` bölümünün komutları depoya push edebilen kişiden gelir,
bu yüzden liste sürümü başına bir kez *sizin* güveninizi de ister. Güven, depo başına hook'ların
hash'iyle tutulur (herhangi bir komutun, aşamanın veya sıranın değişmesi yeniden sorar) ve
daemon tarafından bir çalıştırmanın ilk adımından önce ve devam ettirilen bir akış sürmeden
önce denetlenir; bu yüzden reddedilen bir çalıştırma hiçbir şeyi değiştirmemiştir. Kendi
geçersiz kılmanızdaki hook'lar (depo durumunuzdaki `[workflow]`, Ayarlar sayfası) sizindir ve
asla sormaz.

- **CLI:** `gitpoulpi flow trust-hooks` komutları, `hash` değerlerini ve `trusted` olup
  olmadıklarını yazdırır; `gitpoulpi flow trust-hooks --hash <hash>` onlara güvenir, ama
  yalnızca diskteki dosya hâlâ o hash'e sahipse. Güvenilmeyen hook'larda `--run-hooks`,
  komutlarla ve o komut satırıyla birlikte bir ret yazdırır (`{"refused": {"reason":
  "hooks_not_trusted", ...}}`, çıkış kodu 2). "Run them too?" diye sorulan bir terminala
  onlara güvenip güvenmeyeceği de sorulur.
- **TUI:** önizleme hook'ları listeler ve henüz güvenilmediklerini söyler. Eylem düğmesine
  basmak bu durumda kaydırılabilir bir bölmede tam komutlarla birlikte (fare tekerleği,
  sürüklenebilir kaydırma çubuğu, PgUp/PgDn/Home/End) "Bu deponun iş akışı makinenizde şu
  komutları çalıştırır. Güvenilsin mi?" diye sorar ve iki yanıt sunar: **1 Güven ve çalıştır**
  ve **2 Hook'lar olmadan çalıştır**. Bir ok tuşuna veya bir rakama basana kadar hiçbir şey
  vurgulanmaz; Esc, İptal veya kapatma çarpısı yanıt vermeden geri döner. Dosya, güvendiğinizden
  beri değiştiyse başlık "Hook'lar, onlara güvendiğinizden beri değişti" der ve eski liste
  yenisinin altında gösterilir.
- **Ajanlar:** hook'lara güvenecek bir araç yoktur. Güvenilmeyen hook'larda `run_hooks: true`
  komutlarla birlikte reddedilir ve bir insan `flow trust-hooks` komutunu çalıştırır.

### Ajanlar için (MCP)

Araçlar `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` ve `flow_set_parent`'tır
(finish, sync ve promote yıkıcı olarak işaretlidir, bu yüzden `allow_mutations` onları
denetler). Planla-sonra-çalıştır biçimindedirler: planı almak için `dry_run: true` ile çağırın
(normal bir sonuç), sonra çalıştırmak için `yes: true` ile yeniden. İkisi de yoksa çağrı
reddedilir ve planı taşır, böylece hiçbir şey kazara çalışmaz. Hook'lar yalnızca açık bir
`run_hooks: true` ile çalışır; plan onları listeler (`hooks`, `hooks_run`) ki ajan komutları
önce kullanıcıya gösterebilsin. Commit'lenmiş hook'lara bir insan güvenmelidir (aracı olmayan
`flow trust-hooks`): güvenilmeyen hook'larda `run_hooks: true`, komutlarla birlikte `refused`
döndürür ve hiçbir şey çalıştırmaz. Bir çakışmadan sonra `continue_: true` (bayrak `--continue`)
veya `abort: true`.

### Geri alma

| Eylem | `undo` ne yapar |
|---|---|
| start | Dalı siler, üst dalını unutur, önceki dalı yeniden checkout eder. |
| finish | Taşıdığı her dalı, tag'i, silinen dalı ve checkout'u tek adımda geri yükler. Push edilmiş bir dal, uzak bir silme ve açılmış bir merge request kalır (plan bunu söyler). |
| sync | Güncellenen her dalı geri koyar, checkout edilmiş olan için dosyalar dahil. |
| promote | Ortam dalını geri koyar. |
| set-parent | Önceki kaydedilmiş üst dalı geri yükler. |

## Barındırıcı tarafından yürütülen akışlar

İşi bir issue'nun başlattığı ekipler: issue bir şablondan yazılır ve dalı ile merge request'i
barındırıcı (GitLab, GitHub, Gitea, Forgejo) veya bir bot oluşturur. GitPoulpi issue'yu
oluşturur, barındırıcının yaptıklarını bekler ve devralır: uzak dalı izleyen yerel bir dal,
kaydedilmiş üst dalıyla (merge request'in hedefi), checkout edilmiş. ISSUE'LAR bölümünün bir
takipçiye bağlı olmasını gerektirir (`gitpoulpi hosting link issues`); PULL REQUEST'LER bağlantısı
dalların ve isteklerin nerede durduğunu söyler (bağlı değilse ISSUE'LAR projesi).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # bir bot çoğu zaman bir etiketle tetiklenir
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # deponun bir dosyası
branch_from_issue = "{type}/{issue}-{slug}"   # barındırıcının veya botun dala verdiği ad
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # `wait` ne kadar yoklar
merge_request = true                 # host-creates-branch: ayrıca bir taslak istek
assignees = []
```

`branch_from_issue` içindeki `{type}` konu türünün kimliğidir; geri kalanlar olağan
[ad deseni](#ad-desenleri) token'larıdır (`{issue}`, `{slug}`, `{issue_key}`...). Bir dal, bu
desenle `{issue}` değeri 12'ye eşit olarak eşleşirse 12 numaralı issue'ya aittir
(`feature/12-login` aittir, `feature/120-x` ait değildir).

| Kip | Issue oluşturulduktan sonra |
|---|---|
| `local` | Barındırıcıda hiçbir şey. `--pick-up` ile, türün issue numarası ve başlığıyla bir `flow start` komutu (her start gibi geri alınabilir). |
| `host-creates-branch` | GitPoulpi dalı türün üst dalından **barındırıcıda** oluşturur (GitLab: dallar API'si; GitHub: `createLinkedBranch`, böylece issue onu Development altında gösterir; Gitea/Forgejo: dallar API'si) ve `Closes #12` diyen bir **taslak** istek açar. Boş bir isteği reddeden bir barındırıcı (GitHub: henüz commit yok) dalı bırakır ve bunu uyarı olarak söyler. |
| `bot` | Hiçbir şey: bir bot (bir webhook işleyicisi, bir CI işi, bir GitHub App) dalı ve isteği oluşturur. `--wait`, `wait_for` sağlanana dek yoklar. |

`wait` her sağlayıcı için şunlara bakar: barındırıcının kendi bağlantıları (GitLab
`issues/:iid/related_merge_requests` ve `closed_by`; GitHub bağlı dalları ve zaman çizelgesinin
`connected` / `cross-referenced` pull request'leri), adı `branch_from_issue` ile eşleşen dallar
ve kaynak dalı eşleşen açık istekler. `wait_for` sağlanana veya `timeout_secs` bitene dek artan
bir aralıkla (1, 2, 3, 5, 8, sonra her 10 sn) yoklar ve bir yanıttan sonra yavaş olan veya kısa
süre erişilemeyen bir barındırıcı yeniden denenir. Barındırıcının sunmadığı bir kaynak (Gitea'nın
issue bağlantısı yoktur) atlanır; her kaynak başarısız olursa ilk hata yanıttır.

| Komut | Ne yapar |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Issue'yu şablondan oluşturur (`issue_title`, `issue_labels`, açıklama dosyası, onun yerine `--body`) ve kipin dediği gibi devam eder. `--dry-run` nelerin oluşturulacağını (`{"plan": ...}`) yazdırır ve hiçbir şey oluşturmaz. `--pick-up` `bot` kipinde önce bekler. Çıkış kodu 4: bekleme bitti (issue var; `flow issue wait` beklemeye devam eder). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Barındırıcının bir issue için yaptıklarını yoklar, bir terminalde ilerleme stderr'e yazılır; Ctrl+C iptal eder. `--once` tek bir kez bakar. Çıkış kodu 4: henüz bir şey yok. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Fetch eder, `origin/<branch>` dalını izleyen yerel dalı oluşturur, üst dalını kaydeder ve checkout eder. `#12` issue'nun dalını veya isteğini barındırıcıda arar, `!5` bir isteği, geri kalan her şey bir dal adıdır. Üst dal isteğin hedefidir (`--parent` onu geçersiz kılar), yoksa türün üst dalı. `--yes` gerekmez: yereldir ve geri alınabilir, `start` gibi. Bir `--dry-run` yine de fetch eder (yalnızca uzak izleme ref'lerini taşır ve plan dalı görmek zorundadır). |

`pick-up` geri alma geçmişinde tek bir girdidir: `gitpoulpi undo` dalı siler, üst dalı unutur ve
önceki dalı yeniden checkout eder (fetch kalır, her fetch gibi). Diğer eylemler gibi planlanır:
`--dry-run` planını (`create_branch`, `set_upstream`, `checkout`, `record_parent`) yazdırır ve
olağan engeller geçerlidir (çalışma değişiklikleri auto-stash ayarını izler). İstek
`{"action": "pick_up", "branch": ..., "parent": ...}` biçimindedir.

Bir akışın oluşturduğu issue'lar **izlenir** (`.gitpoulpi/repo-state.toml` içinde `watched_issues`,
asla commit'lenmez): her fetch'ten sonra uygulama uzak dalları `branch_from_issue` ile eşleştirir
ve izlenen bir issue için bir dal göründüğünde size söyler. Bir devralma bunu kapatır.

### Ajanlar için (barındırıcı tarafından yürütülen)

Araçlar `flow_issue_new`, `flow_issue_wait` ve `flow_pick_up`'tır (write, network;
`allow_mutations` onları denetler). `dry_run: true` ile `flow_issue_new` planı döndürür;
`flow_issue_wait` en çok `timeout` saniye bekler, bu yüzden bir ajana kısa bir süre verin ve
yeniden çağırın (`once: true` hemen yanıt verir). İkisi de `found` (`branches`, `requests`) ve
`timed_out` içeren JSON döndürür.

### Uygulamada barındırıcı tarafından yürütülen akışlar

Bir iş akışı ve bağlı bir takipçiyle Flow menüsü *Yeni <Tür> issue'su...* (tür başına bir tane;
palet `Flow: Yeni <Tür> Issue'su...` ve `Flow: Yeni Issue...`) ve *Barındırıcının oluşturduğu
dalı devral...* (palet `Flow: Barındırıcı Dalını Devral...`) satırlarını kazanır.

- **Yeni issue** türü (bir açılır liste) ve başlığı sorar; açıklamanın nereden geldiğini ve dalı
  kimin oluşturduğunu gösterir. *Dal göründüğünde devral* varsayılan olarak açıktır. *Issue
  oluştur* issue'yu oluşturur, sonra `bot` kipinde **bekleme kartını** gösterir: bir döner
  çubuk, neyin beklendiği, geçen süre ve vazgeçme süresi ve **Beklemeyi bırak** (`Esc`).
  Beklemeyi bırakmak issue'yu kaybettirmez: izlenmeye devam eder. Dal göründüğünde devralınır
  ve durum çubuğu bunu söyler.
- **Bir dal göründü**, bir fetch izlenen bir issue'nun dalını getirdikten sonra açılan bir
  karttır (başka bir iletişim kutusu açık değilse): *Devral* dalı devralır, *Sonra* kartı
  kapatır (uygulama yeniden başlayana dek yeniden sunulmaz).
- **Devral** `#12`, `!5` veya bir dal adı alır ve devralmadan önce neyi bulduğunu gösterir.

Her şey arka planda çalışır; 150 ms'den uzun süren bir çağrı bir döner çubuk gösterir.

### Bir deneme düzeneğinde denemek

`scripts/fake-gitlab-bot.sh`, Docker GitLab (`scripts/test-bench.sh`) için bir sahte bottur: bir
projede etiketli issue'ları izler ve `<type>/<iid>-<slug>` dalını (bir commit'le) ve issue'yu
kapatan bir taslak isteği oluşturur. `cargo run -p xtask -- mock-host --flow-bare <bare repo>`
aynı şeyi çevrimdışı sunar (gerçek bir bare depo üzerinde GitLab biçimli bir API, botuyla birlikte).

## Uygulamada

![Bir Git Flow iş akışıyla kenar çubuğu: main ve develop temel dal olarak işaretli, feature dalları bir Feature klasöründe, geri kalanı Diğer altında](../images/workflow-sidebar.png)

![Feature Başlat: tür, slug, başlangıç noktası ve oluşturacağı dal adı](../images/workflow-start.png)

![Ayarlar > Depoya Özel Ayarlar > İş akışı: şablon, algılama ve temel dal düzenleyicisi](../images/workflow-settings.png)

Aşağıdakilerin tümü yalnızca bir iş akışı olan bir depoda görünür (bir
`.gitpoulpi/workflow.toml` veya sizin bir geçersiz kılmanız). Yoksa hiçbir şey değişmez:
kenar çubuğu sıradan ağaçtır ve `Flow` düğmesi yoktur.

### Ayarlar > İş akışı

*Ayarlar > Depoya Özel Ayarlar > İş akışı* tanımı düzenler. Bir taslak üzerinde çalışır ve bir
Kaydet düğmesine basana kadar hiçbir şey kaydetmez.

- **Şablon**, hazır şablonların bir açılır listesidir; birini seçmek aşağıdaki temel dalları ve
  konu türlerini şablonla değiştirir (hook'larınız ve issue akışınız kalır).
- **Dallardan algıla** bu deponun dal adlarını okur ve en yakın şablonu puanıyla, bulduğu temel
  dal adlarıyla (`main` için `master`), her konu türünün örneklerle kaç dalı tanıdığıyla ve her
  şablonun puanıyla gösterir. **Algılanan iş akışını kullan** onu taslağa yükler.
- Depo başka bir aracın yapılandırmasını içeriyorsa (Git Flow'un `gitflow.*`, Git Town'un
  `git-town.*` değerleri), **İçe aktar** sunulur ve içe aktarılan tanımı taslağa yükler. Ona
  basana kadar hiçbir şey uygulanmaz.
- **Temel dallar**, **Konu türleri** ve **Bitirme kuralları**, **Kurallar** sekmesinde gruplara ayrılmış
  tek bir [liste](settings.md#listeler)dir; **Genel** sekmesi geri kalanı (şablon, algılama,
  hook'lar, issue akışı) içerir. Bir öğeyi
  seçin, sağda düzenleyin (ad ve diğer adlar, üst dal, korumalı, merge stratejisi, merge'de tag, ortam
  dalı; etiket, ad deseni, diğer desenler, düzenli ifade, üst dal, bitirme kipi, bitince sil).
  **+ Ekle ▾** neyin ekleneceğini seçer (bir bitirme kuralı seçili konu türüne aittir); ekleme, çoğaltma,
  kaldırma ve sürükleme her Ayarlar listesindeki gibi çalışır ve öğeler kendi grubunun içinde taşınır.
  **Ad deseni** satırı token'ları listeler (`{slug} {issue}
  {issue_key} {user} {date} {version} {n}`); altında canlı bir önizleme desenin eşleştirdiği
  mevcut dalları ve oluşturacağı adı gösterir ve ayrıştırılamayan bir desen nedenini kırmızıyla,
  alanın hemen altında söyler (yazdığınız hiçbir şey kaybolmaz).
- **Hook'lar** kabuk komutlarıdır, satır başına bir tane (`Başlattıktan sonra`, `Bitirmeden
  önce`), çok satırlı bir düzenleyicide düzenlenir (`Ctrl+S` kaydeder, `Esc` iptal eder).
  Makinenizde komut çalıştırırlar: sayfa bunu söyler ve iletişim kutuları onları çalışmadan
  önce gösterir.
- **Issue akışı**, [barındırıcı tarafından yürütülen akışların](#barındırıcı-tarafından-yürütülen-akışlar)
  kullandığı alanları taşır (kip, issue başlığı ve etiketleri, dal deseni, neyin bekleneceği);
  iş akışı dosyasında saklanır.
- **Sorunlar**, tanımın yanlış yaptıklarını listeler (var olmayan bir üst dal, bozuk bir desen).
- Sayfanın altına sabitlenmiş bir çubuk **● Kaydedilmedi** (ya da **Değişiklik yok**) gösterir.
  **Paylaşılan dosyaya kaydet** taslağı `.gitpoulpi/workflow.toml` dosyasına yazar (commit'leyin),
  bu sürümün tanımadığı anahtarları korur ve kendi geçersiz kılmanızı ona katar. **Yalnızca benim
  için** yalnızca paylaşılan dosyadan farkı depo durumunuzda (`.gitpoulpi/repo-state.toml`,
  commit'lenmez) tutar. **Değişiklikleri at** kaydedileni yeniden okur. Kaydedilmemiş bir taslakla
  Ayarlar'ı kapatmak önce sorar.

### Kenar çubuğu

YEREL ve UZAK iş akışına göre gruplanır: önce temel dallar, her biri bir `[temel]` rozetiyle
(korumalıysa `[temel, korumalı]`), sonra her konu türü için etiketinin adını ve dal sayısını
taşıyan bir klasör (`Feature (3)`), sonra hiçbir türün sahiplenmediği dallar `Diğer` altında.
Klasörler her klasör gibi daralır; düz ağaçlar *Flow > Dalları türe göre grupla* (Flow menüsü
satırı) veya `Flow: Gruplu Dalları Aç/Kapat` palet komutuyla geri gelir. Bir dalın üzerinde
durmak türünü, üst dalını ve üst dalının ne kadar gerisinde (ve önünde) olduğunu gösterir.

### Flow menüsü ve iletişim kutuları

Araç çubuğundaki **Flow** düğmesi (`Ctrl+G Shift+W`, palet `Flow: Menüyü Aç`) her konu türü için
*{Tür} başlat...*, *Geçerli dalı bitir...*, *Üst dalla eşitle...*, *Yükselt...* (GitLab Flow
ortam dalları), *Üst dalı ayarla...*, gruplama anahtarını ve *İş akışı ayarları...* satırlarını
listeler. Palet aynı komutlara sahiptir (`Flow: Feature Başlat...` her tür için üretilir).

- **Başlat**, türün ad deseninden kurulan bir formdur: her token için bir alan. `{issue}` ve
  `{issue_key}` bağlı takipçinin issue'larının bir açılır listesiyle gelir; bu liste ayrıca
  `{slug}` alanını issue'nun başlığından doldurur; `{version}` en yüksek release tag'inizden
  sonraki sürümü önerir; `{user}` etkin profilinizdir; `{date}` bugündür. Bir satır desenin
  yaptığı dal adını gösterir (veya yapamama nedenini: geçersiz bir ref, zaten alınmış bir ad).
  Başlangıç bir açılır listedir (önce türün üst dalı) ve *Yayımla* yeni dalı push eder.
- **Bitir**, **Eşitle**, **Yükselt** ve **Üst dalı ayarla** ihtiyaç duydukları şeyi sorar, sonra
  daemon'un hesapladığı **önizlemeyi** gösterir (`gitpoulpi flow ... --dry-run` ile aynı plan):
  sırayla her adım, uyarılar (uzak deposunun gerisindeki bir hedef, bu dalın üzerine yığılmış
  dallar) ve Çalıştır düğmesini kapatıp nedenini söyleyen **engeller** (var olmayan bir hedef,
  uzak deposunun gerisindeki bir hedef). Geri alınamayan adımlar (bir push, uzak bir dal silme,
  bir merge request) `!` ile işaretlenir ve renklendirilir, ayrıca "Geri alma şunları geri almaz:
  ..." satırı vardır. Çalışacak hook'lar bir uyarının altında, bir *Hook'ları çalıştır*
  onay kutusuyla (varsayılan olarak kapalı) listelenir. Türün bitirme kipi `ask` olduğunda
  Bitir önce *Bitirme yolu* sorar: yerelde merge, ya da bir pull veya merge request açmak.
- Eylem sonra adım başına bir işaretle çalışır (150 ms sonra bir döner çubuk) ve iletişim kutusu
  her adımın nasıl gittiğini gösterir. Eylemin tamamı **tek bir Geri alma girdisidir**: `Ctrl+Z`
  başlatmayı, bitirmeyi, eşitlemeyi veya yükseltmeyi birden geri alır (Geri alma geçmişi
  `Flow bitir feature/login` der); push'lar, uzak silmeler ve merge request'ler önizlemenin
  dediği gibi kalır.
- Bir **çakışma** eylemi **Devam** ve **İptal** ile durdurur. İletişim kutusu çakışmayı olağan
  çakışma araçlarıyla çözmek için kapatılabilir, sonra `Flow: Durdurulmuş Eyleme Devam Et veya
  İptal Et...` palet komutuyla (bir eylem durdurulmuşken ayrıca Flow menüsünün bir satırı)
  yeniden açılır; Devam, dosyalar hâlâ çakışmadayken reddeder ve iki düğmeyi de korur. İptal
  eylemin başladığı dala geri döner; yapılan adımlar kalır (undo onları geri alır).
- **Merge request** ile bitirme dalı push eder, sonra olağan *Pull Request Oluştur* formunu
  (GitLab'da *Merge Request Oluştur*) önceden doldurulmuş olarak açar: hedef dalın üst dalıdır,
  başlık daldan (ve issue'dan) gelir. Üst veri düğmeleri (atananlar, inceleyenler, etiketler,
  kilometre taşı) her pull request'te olduğu gibi çalışır. Uzak depo için hiçbir entegrasyon
  yanıt vermiyorsa, karşılaştırma URL'si bunun yerine tarayıcıda açılır (ve durum çubuğunda
  gösterilir).
- Adım ve engel metinleri CLI'nin metinleri gibi İngilizcedir; iletişim kutusunun kendi metni
  çevrilmiştir. Değişiklikleri olan bir çalışma ağacı `auto-stash` ayarını izler.

Bunların her biri klavyeden ve paletten erişilebilir ve her açılır liste bir combobox'tır
(Enter açar, yazmak filtreler, Enter seçer).

Barındırıcı tarafından yürütülen satırlar (*Yeni <Tür> issue'su...*, *Barındırıcının oluşturduğu
dalı devral...*) [yukarıda](#uygulamada-barındırıcı-tarafından-yürütülen-akışlar) anlatılır.
