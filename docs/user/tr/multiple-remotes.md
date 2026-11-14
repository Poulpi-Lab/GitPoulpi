[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · **Türkçe** · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Bir dal için birden çok uzak depo

Birçok depo birden fazla barındırıcıda yaşar: ana olan (`origin`), başka bir barındırıcıdaki bir yansı, bir yedek. GitPoulpi bir dalın hepsine tek tıkla push etmesini ve isterseniz birden fazlasından pull etmesini sağlar; düz git'in yaptığı şey değişmez. Burada hiçbir şey hesap gerektirmez ve her şey deponun kendi `.git/config` dosyasında saklanır, bu yüzden düz git ve diğer araçlar çalışmaya devam eder.

## Push hedefleri ve pull kaynakları

- **Upstream**, git'in kendi kavramıdır (`branch.<name>.remote` ve `merge`). Dalın *birincil* uzak deposudur. GitPoulpi onun çalışma biçimini asla değiştirmez.
- **Push hedefi**, dalın ayrıca push ettiği ek bir uzak depodur. Push düğmesi dalı upstream'e ve her push hedefine gönderir.
- **Pull kaynağı**, bir pull'un ayrıca commit aldığı ek bir uzak depodur. Siz açmadıkça kapalıdır (**Pull da** onay kutusu): seçmediğiniz iki uzak depoyu merge etmek kimsenin varsayılan olarak istemeyeceği bir şeydir.

Bir hedef, o uzak depoda başka bir dal adı kullanabilir (`backup` uzak deposu `feat-x` alır).

## Uzak depolar penceresi

Bir dala sağ tıklayın ve **&lt;dal&gt; dalının uzak depoları...** öğesini seçin (Push açılır listesinde, paletteki `Git: Dalın Uzak Depoları...` komutunda ve Refs bölümünde dalın satırındaki `⇉3` rozetine tıklayarak da). Deponun her uzak deposunu listeler:

| Sütun | Anlamı |
|---|---|
| Uzak depo, Dal | Uzak depo ve oradaki dal adı. **Uzakta yeniden adlandır...** seçili satır için bu adı değiştirir. |
| Push | Uzak depoyu bu dalın push hedefi yapmak için işaretleyin. |
| Pull da | Bir pull'un o uzak depoyu da bütünleştirmesi için işaretleyin. Upstream her zaman pull edilir. |
| Durum | Dalın ona göre durumu: eşit, önde, geride, yeni, ilgisiz. |

"uzak depo yok" işaretli bir satır, dalın hâlâ andığı ama deponun artık sahip olmadığı bir uzak depoyu belirtir; **Kaldır** onu unutturur. Aynı pencere upstream'i de ayarlar (**Upstream** listesi) ve **Upstream'i kaldır** onu temizler. **Uygula** her şeyi geri alınabilir tek bir değişiklik olarak yazar.

Bir depo her yeni dala aynı ek hedefleri de verebilir: **Ayarlar > Git ve Kodlama** (depoya özel sayfa), "Yeni dallar için ek Push hedefleri" (her uzak depo için bir onay kutusu). Bir dal ilk kez upstream aldığında dala kopyalanırlar; var olan dallara dokunulmaz.

## Push etme

- Ek hedefleri olan bir dalda **Push düğmesi**: hepsine push eder. Her uzak depo yeni commit'leri sorunsuz kabul ediyorsa hemen çalışır ve ilerleme görünümü her uzak deponun bir onay işaretine dönüşmesini gösterir. Bir karar gerekiyorsa (bir uzak depoda sizin olmayan commit'ler varsa) bunun yerine plan penceresi açılır.
- **Push ▾** (düğmenin yanındaki ok; Ayarlar'da **Push düğmesi** **sor** olarak ayarlıysa düğmenin kendisi de):
  - `Push: tümüne (origin, gitlab, backup)`
  - `Push: yalnızca <remote>`, dalın her uzak deposu için bir tane
  - `Push: seçili uzak depolara...`: her dal için, tek uzak deposu olan için bile her zaman vardır
  - `<dal> dalının uzak depoları...`
- Dalın sağ tık menüsü ve palet (`Git: Push Tüm Hedeflere`, `Git: Uzak Depoya Push Et...`, `Git: Push Seçili Uzak Depolara...`) aynı eylemleri aynı adlarla sunar.

**Push: seçili uzak depolara...** deponun her uzak deposunu listeler; dalın upstream'i ve hedefleri işaretli gelir. Yalnızca bu push içindir; seçimi dalın push hedefleri olarak saklamak için **Bu dal için anımsa** seçeneğini işaretleyin.

### Plan

Toplu bir push'tan önce GitPoulpi her uzak depoya bakar (hızlı bir sorgu; o uzak depodan indirme yalnızca farklıysa yapılır), böylece üçüncü uzak depodaki bir sorun ilk push başlamadan görünür. Pencere her uzak depoyu, dalın ona göre durumunu ve ne yapacağını listeler:

| Durum | Varsayılan eylem | Ayrıca sunulan |
|---|---|---|
| Uzak depoda dalın hiçbir şeyi yok veya bazı commit'leri eksik | Push | Atla |
| Uzak depo önde (onun için yeni bir şeyiniz yok) | Ondan fast-forward | Atla |
| Push ettiğinizi rebase ettiğiniz veya amend ettiğiniz için uzak depo farklı | Zorla push (lease) | Merge, Rebase, Atla |
| Uzak depoda başkasının commit'leri var | Ondan merge (**Pull düğmesi** rebase ise Rebase) | Rebase, Zorla push (lease), Atla |
| Geçmiş sizinkiyle hiçbir şey paylaşmıyor | Atla | Zorla push (lease) |
| Barındırıcı dalı reddetti (korumalı) | Atla | yok: zorlamak da reddedilirdi |

Her satırın eylemi değiştirebileceğiniz bir listedir (Sol/Sağ içinde gezer, Enter açar, `s` satırı atlar). Bir merge'ün çakışıp çakışmayacağını ve hangi dosyalarda olduğunu gösterir. **Force push** her zaman onaylanır, uzak depoyu ve kaç commit'i değiştirdiğini belirtir, her zaman lease kullanır (asla çıplak force değil) ve geri alınamaz: sonuç hangi commit'in değiştirildiğini ve onu geri koyan komutu söyler. **Sorunluları atla** sorunlu her satırı Atla yapar. **Çalıştır** (Ctrl+Enter) başlatır.

Merge, fast-forward ve rebase işlemleri önce, listenin sırasıyla çalışır; böylece dalınız son durumuna bir kez ulaşır; sonra push'lar her seferinde bir uzak depoya, önce upstream'e olmak üzere gider. Uzak depolar arasında atomik bir "ya hep ya hiç" yoktur: her biri kendi sonucuyla biter ve birindeki bir hata (ağ, bir hook) diğerlerini durdurmaz. **Başarısızları yeniden dene** (veya `Git: Başarısız Push İşlemlerini Yeniden Dene`) onları yeniden dener. İlerleme görünümünü kapatmak çalıştırmayı asla durdurmaz; `Git: Toplu Push İşlemini Göster` onu geri getirir.

### Çakışmalar ve Devam

Çalıştırmanın bir merge veya rebase işlemi çakışmada durursa pencere kapanır ve olağan merge şeridi ile çakışma düzenleyicisi devralır. Log üzerindeki bir şerit `Toplu Push duraklatıldı (gitlab): çözün ve tamamlayın, sonra Devam.` yazar ve **Devam** ile **Grubu iptal et** düğmelerini taşır.

- Çakışmaları çözün ve merge veya rebase işlemini tamamlayın (GitPoulpi'de ya da düz git ile).
- **Devam**, bu yapılana kadar grileşiktir ve **sizin yerinize asla basılmaz**: hiçbir şey kendiliğinden devam etmez. Ona basın (veya `Git: Toplu Push İşlemine Devam Et` komutunu çalıştırın) ve çalıştırma yeni ucunuzdan her uzak depoya yeniden push eder. Bir rebase'ten sonra eski commit'leri hâlâ tutan bir uzak depo sessizce zorlanmak yerine plan penceresine geri döner (artık force push gerekir).
- **Grubu iptal et** (onaylı; `Git: Toplu Push İşlemini İptal Et`) çalıştırmanın başlattığı merge veya rebase işlemini atar ve çalıştırmayı unutur. Önceden yapılan push'lar kalır.

Şerit GitPoulpi'nin yeniden başlatılmasından sağ çıkar ve CLI aynı kayıtlı çalıştırmayı görür.

## Pull etme

- **Pull** düğmesi, dalın pull kaynakları yoksa upstream'i eskisi gibi pull eder; varsa upstream'i ve her kaynağı sırayla, her biri Ayarlar'daki **Pull düğmesi** kipinde (merge, yalnızca fast-forward veya rebase) pull eder. Fast-forward yapılamayan bir kaynak upstream gibi merge edilir (veya rebase edilir); yalnızca fast-forward ile, daha fazlasına ihtiyacı olan bir kaynak önce sorar.
- **Pull ▾**, kaynak varsa `Pull: tüm kaynaklardan (origin + gitlab)` öğesini ve dalın her uzak deposu için `Pull: yalnızca <remote>` öğesini ekler: onay kutuları ne derse desin, o uzak deponun dalından tek seferlik bir pull.
- Palette `Git: Pull Tüm Kaynaklardan` ve `Git: Pull Tek Bir Uzak Depodan...` vardır.
- Çakışmada duran bir pull, toplu pull olarak etiketlenmiş aynı şeridi bırakır.

## Ayarlar

Dördü de **Ayarlar > Git ve Kodlama** sayfasındadır.

| Ayar | Varsayılan | Anlamı |
|---|---|---|
| **Push düğmesi** | doğrudan | doğrudan hemen push eder; sor önce Push menüsünü açar. |
| **Pull düğmesi** | merge | Pull düğmesinin kipi (merge, yalnızca fast-forward veya rebase) ve pull kaynaklarının kipi. |
| **Her uzak depoyu Push öncesinde denetle** | açık | Toplu bir push'tan önce her uzak depoya bakar (CLI bunu izler; `--no-preflight` tek bir çağrı için kapatır). |
| **Yeni dallar için ek Push hedefleri** (depo başına) | yok | Yeni bir dalın ayrıca push ettiği uzak depolar. |

## Komut satırından

```
gitpoulpi branch upstream add gitlab --pull      # aynı zamanda pull kaynağı olan bir push hedefi
gitpoulpi branch upstream list                    # dalın neler için ayarlandığı
gitpoulpi push --all-targets                      # upstream artı her hedef
gitpoulpi push --to gitlab --to backup:feat-x     # tam olarak bunlar, yalnızca bu seferlik
gitpoulpi pull --all-sources                      # upstream artı her pull kaynağı
gitpoulpi remote-group plan                       # toplu bir push'un ne yapacağı, hiçbir şey değişmez
gitpoulpi remote-group continue                   # bir durmayı çözüp tamamladıktan sonra
```

Bir uzak depo karar gerektirdiğinde komut planla yanıt verir (çıkış 3) ve hiçbir şeyi değiştirmez; `--resolve gitlab=merge` ile yanıtlayın. Her bayrak için [CLI başvurusuna](cli-reference.md) bakın. MCP sunucusunda aynı araçlar vardır.

## Düz git ile

Hedefler düz yapılandırma anahtarlarıdır, bu yüzden bir betik veya takma ad onları GitPoulpi olmadan kullanabilir:

```
git config --get-all branch.feature.gitpoulpi-push-to     # satır başına bir değer: remote veya remote:branch

# geçerli dalı upstream'ine ve her hedefe push et
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Bir değer `<remote>` (aynı dal adı) veya `<remote>:<branch>` biçimindedir; `d=${t#*:}` satırı bunu ayırır. Düz takma ad önce uzak depolara bakmaz, merge sunmaz ve çakışmadan sonra devam edemez: pencere bunun içindir.

## Bilmekte fayda var

- Aynı adrese işaret eden iki uzak depo fark edilir ("gitlab ve mirror aynı URL'ye push yapıyor."); ikincisi varsayılan olarak atlanır.
- Korumalı dalı olan bir barındırıcı force push'u reddeder; satır bunu söyler ve yalnızca Atla sunar.
