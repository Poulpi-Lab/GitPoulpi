[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · **Türkçe** · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktree'ler

Worktree, deponun ikinci bir checkout edilmiş kopyasıdır: başka bir klasör, başka
bir dal, aynı geçmiş. Refs panelinin WORKTREE'LER bölümü onları listeler; ana
olan `(ana)` ile işaretlidir.

## Bölümü göstermek

Henüz bağlı bir worktree'si olmayan bir depoda bölüm gizlidir. Yine de görmek
için (o zaman "Henüz worktree yok: oluşturmak için +" yazar) palet komutu **Sol
panel: Worktree'ler Bölümünü Göster**'i kullanın, kenar çubuğunun göster/gizle
listesinde **WORKTREE'LER**'i işaretleyin (bir bölüm başlığına sağ tıklayın) ya da
aşağıdaki oluşturma komutlarından birini çalıştırın. GitPoulpi bunu o depo için
hatırlar (bkz. [Ayarlar](settings.md) içindeki **Sol paneldeki bölümler**);
worktree'si olmayan başka bir depo bölümü yine gizli başlatır.

## Worktree oluşturmak

WORKTREE'LER başlığındaki **+** (klavye: bölümde `+`; palet: **Refs: Worktree
Oluştur...**) bir menü açar:

- **Geçerli daldan**: dal ana klasörde checkout edilmiştir, bu yüzden iletişim
  kutusu onun üzerinde yeni bir dal önerir (`worktree-<id>`).
- **Seçili commit üzerinden**: Log'da bir commit seçilene kadar gri kalır.
  İletişim kutusu orada başlayan `worktree-<id>` adlı yeni bir dal önerir;
  ayrık HEAD'li bir worktree için dal alanını boşaltın.
- **Dal seç...**: iletişim kutusu, zaten var olan bir dalı açmak için **Worktree
  dalı** listesi üzerinde açılır.

Paletin aynı üçü vardır (**Refs: Geçerli Daldan Worktree Oluştur**, **Git: Seçili
Commit'te Worktree Oluştur...**, **Refs: Dal Seçerek Worktree Oluştur**). Eski
girişler de çalışır: bir dal, tag veya commit menüsündeki **... üzerinden
worktree oluştur...**.

İletişim kutusu her şeyi önerir ve değiştirmenize izin verir. **Ad** ve yeni dal
ikisi de `worktree-<id>` olarak başlar; `<id>` 8 rastgele onaltılık rakamdır (ikisinde
aynı), böylece iki worktree asla çakışmaz.

| Alan | Anlamı |
|---|---|
| **Ad** | Klasör adı bundan üretilir. **Dalın adını ver** (`Alt+N`) onu `worktree-<dal>` yapar: eğik çizgiler `-` olur ve sözcük ikilenmez (`worktree-foo` ve `feature/worktree-x` kendi hâlini korur, `feature/login` `worktree-feature-login` verir). |
| **Klasör** | Deponun yanında, `<depo>-<ad>` biçiminde bir kardeş klasör. Klasör alanına yazana kadar Ad'ı izler; **Gözat...** nereye konacağını seçer. |
| **Ref** | *Yeni* bir dalın başladığı yer: bir dal veya commit (bir liste; aramak için yazın). Mevcut bir dal seçiliyken kullanılmaz ve bunu söyler. |
| **Worktree dalı** | Tek alan, iki anlam; altında hangisi olduğu yazar. Yerel dal olmayan bir ad, Ref'ten yeni bir dal oluşturur (**main üzerinden yeni dal**). Başka yerde checkout edilmemiş yerel bir dal (listeden seçin veya adını yazın) orada olduğu gibi checkout edilir (**bu dalı checkout eder**). Listenin ilk satırı veya boş alan Ref'in dalını checkout eder (bir commit'te ayrık HEAD olur); o dal zaten checkout edilmişse reddedilir. |
| **Yeniden üret** | `Alt+R`. Yeni bir `<id>` üretir ve Ad ile dalı onunla değiştirir; kendi yazdığınız (yazdığınız bir Ad veya dal kalır) ve mevcut bir dal (asla değiştirilmez) hariç. **Dalın adını ver** veya YZ'den gelen bir Ad üretilmiş sayılır. |
| **YZ ile adlandır** | Yalnızca YZ açıkken (aşağıya bakın). |
| **Yeni worktree'yi sekme olarak aç** | Varsayılan olarak açık. |

Boş olmayan bir klasör, geçersiz bir ad, git'in reddettiği bir dal adı veya başka
bir worktree'nin sahip olduğu bir dal iletişim kutusunun kendi satırında bildirilir
ve **Oluştur** gri kalır.

### YZ ile adlandırma

[YZ](ai.md) açıkken **YZ ile adlandır** (`Alt+A`) iletişim kutusunun içinde bir
kutu açar: *Özelliği, hatayı veya görevi açıklayın*. **Adlandır**, modelden kısa bir
worktree adı ve deponun kendi dal adlarını (`feature/`, `fix/`, küçük harfli
kebab-case) izleyen bir dal adı ister. Bunlar Ad'ı (`worktree-<ad>` olarak) ve dal
alanını doldurur, yine düzenlenebilir; zaten var olan bir dal adı `-2` soneki alır,
böylece yanıt sessizce bir şeyi checkout etmez. Kutu meşgul durumunu gösterir,
**Durdur** (veya `Esc`) iptal eder ve bir hata iletişim kutusunun içinde gösterilir.
Yalnızca açıklamanız ve yerel dal adları gönderilir.

## Satırın menüsü

Bir worktree'ye sağ tıklayın (ya da seçip menü tuşuna, `Shift+F10` veya `.`
tuşuna basın):

- **Yeni sekmede aç**: çift tıklama veya `Enter` gibi.
- **Şurada aç**: **Worktree'yi açabilir** kutusu işaretli olan [Şurada aç](open-in.md)
  girişleri (Ayarlar > Şurada aç; her giriş varsayılan olarak işaretlidir). Worktree'nin
  klasörüyle, `${cwd}` olarak çalışırlar.
- **<dal> dalını şuraya merge et...**: merge edilecek dalı seçin (varsayılan olarak ana
  worktree'nin dalı). Olağan merge, o dalın checkout edildiği worktree'de, merge
  seçeneklerinizle çalışır; çakışma orada olağan çakışma akışıyla çözülür. Hiçbir
  yerde checkout edilmemiş bir dal, nedeni belirtilerek reddedilir. Ayrık bir
  worktree için sunulmaz.
- **Worktree'yi kaldır...**: ana worktree asla.

Palet, seçili satır üzerinde aynı eylemlere sahiptir (**Refs: Worktree'yi Şurada
Aç...**, **Refs: Worktree Dalını Başka Dala Merge Et...**, **Refs: Worktree'yi
Kaldır...**, **Refs: Worktree'yi Sekmede Aç**).

## Worktree'yi kaldırmak

Temiz bir worktree sade bir onay alır. Dalı zaten merge edilmişse (upstream'ine, ana
worktree'nin dalına veya `main` / `master`'a) **<b> dalını da sil** kutusu varsayılan
olarak işaretlidir; merge edilmemiş bir dal kalacağını söyler. **Merge et... sonra
kaldır** önce merge eder ve worktree'yi yalnızca merge çakışmasız tamamlandıktan sonra
kaldırır.

Commit edilmemiş değişiklikler (izlenen veya izlenmeyen) varsa iletişim kutusu
onları listeler (sayı, kaydırılabilir) ve şunları sunar:

- **Commit ve kaldır**: iletişim kutusunda bir mesaj alanı; her değişiklik stage'lenir
  (`.gitignore` gözetilir) ve worktree'nin dalına commit edilir, sonra worktree
  kaldırılır. Ayrık bir worktree, commit için bir dalı aynı iletişim kutusunda
  adlandırır.
- **Stash ve kaldır**: izlenmeyen dosyalar dahil her değişiklik, düzenlenebilir bir
  mesajla (varsayılan "<klasör> üzerinde WIP") stash'e alınır, sonra worktree
  kaldırılır. Stash listesi tüm worktree'ler arasında paylaşılır: stash sonrasında ana
  deponun STASH'LER bölümündedir, pop etmeye hazır.
- **Commit için aç**: worktree'yi, commit mesajı odakta olacak biçimde bir sekmede
  açar; hiçbir şey kaldırılmaz.
- **Yine de kaldır**: ikinci, açık bir onaydan sonra değişiklikleri atar.

## Geri alma

Bir worktree'yi kaldırmak geri alınamaz: klasör gitmiştir. Önce kaydedilen şey
kalır: commit dalındadır, stash listededir. **<dal> dalını da sil**'in eklediği dal
silme işlemi olağan, günlüğe yazılan işlemdir: **Geri Al** dalı geri getirir
(klasörü değil). Bir worktree oluşturmak da geçmişin parçası değildir.

## Bir betikten veya ajandan

`gitpoulpi worktree changes <path>` bir worktree'nin commit edilmemiş
değişikliklerini listeler. `gitpoulpi worktree remove <path> [--force]
[--commit-message M [--commit-branch B]] [--stash-message M] [--delete-branch]`
onu kaldırır; MCP araçları `worktree_changes` ve `worktree_remove`'dur.
`gitpoulpi ai worktree-name --describe "<görev>"` YZ'nin önerdiği worktree ve dal
adlarını (`{ name, worktree, branch }`) yazdırır ve hiçbir şey oluşturmaz; MCP aracı
`ai_worktree_name`'dir. Bkz. [CLI başvurusu](cli-reference.md).
