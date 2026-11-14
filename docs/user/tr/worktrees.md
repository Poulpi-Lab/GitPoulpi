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
  kutusu onun üzerinde yeni bir dal önerir (`<dal>-wt`).
- **Seçili commit üzerinden**: Log'da bir commit seçilene kadar gri kalır.
  Yeni bir dal adlandırmazsanız worktree orada ayrık HEAD ile başlar.
- **Dal seç...**: iletişim kutusu **Ref** listesi üzerinde açılır.

Paletin aynı üçü vardır (**Refs: Geçerli Daldan Worktree Oluştur**, **Git: Seçili
Commit'te Worktree Oluştur...**, **Refs: Dal Seçerek Worktree Oluştur**). Eski
girişler de çalışır: bir dal, tag veya commit menüsündeki **... üzerinden
worktree oluştur...**.

İletişim kutusu her şeyi önerir ve değiştirmenize izin verir:

| Alan | Anlamı |
|---|---|
| **Ad** | Klasör adı bundan üretilir. Daldan veya kısa kimlikten önerilir. |
| **Klasör** | Deponun yanında, `<depo>-<ad>` biçiminde bir kardeş klasör. Klasör alanına yazana kadar Ad'ı izler; **Gözat...** nereye konacağını seçer. |
| **Ref** | Worktree'nin başladığı dal veya commit (bir liste; aramak için yazın). Değiştirmek yeni bir Ad önerir. |
| **Yeni dal** | Boşsa Ref'in dalı orada checkout edilir (bir commit'te ayrık HEAD olur). Dal zaten checkout edilmişse zorunludur. |
| **Yeni worktree'yi sekme olarak aç** | Varsayılan olarak açık. |

Boş olmayan bir klasör, geçersiz bir ad veya zaten var olan bir dal iletişim
kutusunun kendi satırında bildirilir ve **Oluştur** gri kalır.

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
onu kaldırır; MCP araçları `worktree_changes` ve `worktree_remove`'dur. Bkz.
[CLI başvurusu](cli-reference.md).
