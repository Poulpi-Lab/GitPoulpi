[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · **Türkçe** · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Çalışma alanları

Bir **çalışma alanı**, birlikte üzerinde çalıştığınız depoların (bir ön yüz,
API'si, belgeleri) adlandırılmış bir grubudur. Hangilerinin sekme olarak açık
olduğunu hatırlar ve bir **varsayılan commit profili** taşıyabilir; böylece bu
depolardaki commit'ler, elle profil değiştirmenize gerek kalmadan doğru kimliği
kullanır. Bir depo birden çok çalışma alanında olabilir ve hiçbirinde olmamak da
sorun değildir: "Çalışma alanı yok", bugünkü düz sekme listesidir.

Çalışma alanları, `config.toml`'un yanındaki `workspaces.toml` dosyasında tutulur;
tüm oturumlar, pencere, CLI ve MCP tarafından paylaşılır (`gitpoulpi workspace ...`,
bkz. [CLI başvurusu](cli-reference.md)). Pencere açıkken bir kabuktan ya da
`gitpoulpi workspace ...` ile eklenen, yeniden adlandırılan veya kaldırılan bir
çalışma alanı, bir saniye kadar içinde kendiliğinden görünür (Çalışma alanları
sekmesi, sekme çubuğunun rozeti ve listesi), tuşa basmaya gerek kalmadan. Bir
çalışma alanını kapatmak veya silmek diskteki hiçbir depoya dokunmaz. Çalışma alanı düzenlemeleri favoriler ve
profiller gibi ayarlardır: [Geri al ve yinele](undo-redo.md) kapsamında değildir
ve Sil önce sorar.

## Çalışma alanı rozeti

Bir çalışma alanınız olunca, sekme çubuğunda `⌂`'nin hemen ardından bir rozet
belirir: renkli bir nokta ve etkin çalışma alanının adı, ya da "Çalışma alanı yok"
(soluk). Dar bir çubukta yalnızca noktaya küçülür, çok dar bir çubukta kaybolur.

![Sekme çubuğunun üzerindeki çalışma alanı açılır listesi: Çalışma alanı yok, Orbit (işaretli), Side projects, Archive ve Çalışma alanlarını yönet; araç çubuğunda Orbit rozeti ve profili "Work - Orbit çalışma alanından"](../images/workspace-switch.png)

- **Tıklayın**, `Alt+W` tuşuna basın veya paletten `Çalışma Alanı: Değiştir...`
  çalıştırın: çalışma alanlarınızı, "Çalışma alanı yok" satırını ve "Çalışma
  alanlarını yönet..." satırını (Çalışma alanları sekmesini açar)
  gösteren bir açılır liste açılır.
- **Geçiş sekmeleri değiştirir.** Ayrıldığınız çalışma alanı için açık
  sekmeleriniz saklanır ve hedefin sekmeleri geri gelir (ilk seferde, var olan
  depoları). Geri dönün, her şey bıraktığınız yerde durur. Pencere, bir sekme
  geçişi gibi yeniden başlar; daemon kalır.
- **Çalışma alanı hatırlanır**: bir sonraki açılış onda başlar.
- Bir çalışma alanının **son sekmesini kapatmak** çalışma alanını etkin tutar ve
  Ana sekmeyi gösterir.
- **Bir çalışma alanı etkinken `+` (yeni sekme)**, seçtiğiniz depoyu sekme
  listesine ekler ve hiçbir şey sormaz. Depoyu çalışma alanının kendisine koymak
  için sekmenin sağ tık menüsündeki "Çalışma alanına ekle" satırını (veya
  `Çalışma Alanı: Sekmeyi Çalışma Alanına Ekle`) kullanın; "Çalışma alanından
  çıkar" onu geri alır ve sekmeyi açık bırakır.

Henüz çalışma alanı yokken rozet de yoktur: önce Çalışma alanları sekmesinden veya paletten bir
tane oluşturun.

## Çalışma alanları sekmesi

Onu `+` menüsünden, `Alt+J` ile ya da paletteki `Çalışma Alanı: Yönet...` komutuyla açın; `x` düğmesi (veya `Ctrl+W`) onu kapatır ve siz kapatana kadar yeniden başlatmalarda açık kalır. Her çalışma alanı
bir satırdır: renk noktası, ad, kaç depo listelediği (ve kaçının eksik olduğu),
varsayılan profili ve pencere onun içindeyse "etkin". `Right`, `Space` veya bir
tık satırı depolarına açar.

![Çalışma alanları sekmesi: Orbit iki deposuna açılmış, Side projects ve "1 eksik" işaretli Archive](../images/workspaces.png)

| Çalışma alanı başına | Tuş | |
|---|---|---|
| Aç | `Enter` | Etkin yapar ve sekmelerini açar. |
| Düzenle... | `e` | Aşağıdaki form. |
| Sil... | `x` | Önce sorar. Depolara dokunulmaz. |
| Tümünü fetch et | `Shift+F` | Depolar birbiri ardına, ilerleme durum çubuğunda, başarısızlıklar sonda listelenir. |
| Tümünü pull et | `Shift+P` | Aynısı, yalnızca fast-forward: hiçbir zaman bir merge'de durmaz. |
| Eksikleri klonla | `m` | Kayıtlı bir uzağı olan her eksik depoyu klonlar. |
| Yeni çalışma alanı... | `n` | |
| Açık sekmelerden... | `a` | Açık sekmelerinizin işaretli olduğu yeni bir çalışma alanı. |

**Eksik** bir deponun (klasörü kalmamış) satırı işaretli olarak listede kalır;
**Bul...** (`l` veya `Enter`: şimdi nerede olduğunu seçin), **Klonla** (eklenirken
`origin` URL'si kaydedildiyse) ve **Çıkar** (`x`) ile. Her satırda ayrıca aynı
eylemleri içeren bir `⋮` menüsü vardır (sağ tık, `.` veya `Shift+F10`) ve her
eylem paletteki `Çalışma Alanı:` altındadır.

Etiketler "git terimleri" modunda gösterilmiştir; "Git terimlerini çevir" açıkken
"Tümünü getir" ve "Tümünü çek" olurlar.

## Yeni / Düzenle formu

![Çalışma alanını düzenle formu: ad, renk, açıklama, varsayılan profil ve depoların işaret listesi](../images/workspace-edit.png)

- **Ad**: zorunlu, benzersiz.
- **Renk**: tema renklerinin açılır listesi; rozetteki ve satırlardaki nokta.
- **Açıklama**: serbest metin.
- **Profil**: `[[profiles]]` girdilerinizin açılır listesi veya "Yok". İsteğe
  bağlıdır.
- **✦ YZ talimatları**: yalnızca YZ açıkken, bkz. [aşağıda](#bir-çalışma-alanının-yz-talimatları).
- **Depolar**: GitPoulpi'nin bildiği her deponun işaret listesi (açık sekmeler,
  favoriler, son kullanılanlar, çalışma alanının kendi depoları, eksik olanlar
  dahil). `Space` bir satırı işaretler; **Klasör ekle...** (`Ctrl+B`) diskten bir
  depo seçer. Her yeni deponun origin URL'si depodan okunur ve (hiçbir kimlik
  bilgisi olmadan) kaydedilir; Klonla bunu sonradan kullanır.

## Bir çalışma alanının YZ talimatları

YZ açıkken (*Ayarlar > YZ*), formda **Düzenle...** düğmeli bir **✦ YZ talimatları**
satırı bulunur (satırda `Enter`). Çalışma alanının **her deposunda** YZ'ye
söylenenlerin küçük bir düzenleyicisini açar: **Hedef** listesinden *Her YZ
eylemi*ni veya bir eylemi (commit mesajı, açıklama, stash mesajı, pull request,
commit composer, çakışma çözümü, kod incelemesi) seçin, **Birleştirme biçimi**ni
seçin - *Genele ekle* (genel talimatlardan sonra gönderilir) veya *Genelin yerine
geç* (onların yerine gönderilir) - ve metni yazın. **Uygula** tüm kümeyi tutar,
formdaki **Kaydet** onu çalışma alanıyla birlikte `workspaces.toml` dosyasına
yazar; **İptal** yazdıklarınızı bırakır. YZ kapalıyken satır gizlenir ve
çalışma alanının talimatları olduğu gibi kalır.

Çalışma alanının sözleri, genel talimatlarla deponun kendi talimatları arasında
yer alır (sıra ve *değiştir*in ne yaptığı için bkz. [YZ talimatları](ai.md#talimatlar)).
Birden çok çalışma alanına ait bir depo, içeriyorsa penceredeki etkin çalışma
alanını, yoksa onu içeren ilk çalışma alanını kullanır. Bir betikten:
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` bunları `ai_instructions` olarak yazdırır.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # bu çalışma alanındaki her YZ eylemi
text = "Use the Acme glossary."
mode = "add"                       # veya "replace"

[workspace.ai_overrides.code_review]   # tek bir eylem
text = "Flag missing tests."
mode = "add"
```

## Varsayılan profil

Commit'lere, uygulanan şunlardan ilki imza atar:

1. o depo için **elle** seçtiğiniz profil (araç çubuğundaki profil alanı),
2. depo üyesiyse **etkin çalışma alanının** profili,
3. `general.active_profile`.

Araç çubuğundaki profil alanı nereden geldiğini söyler: "Work - Acme çalışma
alanından". Tek bir profiliniz olsa bile, bir çalışma alanının varsayılanı
geçerli olduğu sürece alan görünür. Profili artık var olmayan bir çalışma alanı
yalnızca uygulanmaz. CLI'da `commit --workspace <id>` (veya
`GITPOULPI_WORKSPACE`) aynı şekilde çözer.

## Tuşlar

| Tuş | Eylem |
|---|---|
| `Alt+W` | Çalışma alanı açılır listesini aç |
| `Alt+J` | Çalışma alanları sekmesini aç |
| `n`, `a` | Yeni çalışma alanı, açık sekmelerden yeni |
| `e` | Çalışma alanını düzenle |
| `Shift+F`, `Shift+P`, `m`, `l` | Tümünü fetch et, tümünü pull et, eksikleri klonla, bul |

Hepsi Ayarlar > Klavye ve fare altında yeniden atanabilir; tam liste
[Klavye kısayolları](keybindings.md) sayfasındadır.
