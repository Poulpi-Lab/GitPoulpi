[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · **Türkçe** · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Alt modüller

Alt modül, sabit bir commit'te sizin deponuzun içine yerleştirilmiş başka bir depodur. GitPoulpi bunları Refs kenar çubuğunda listeler, arayüzden, CLI'den veya komut paletinden günceller ve checkout ettiğiniz commit ile uyum içinde tutabilir.

## ALT MODÜLLER bölümü

Bir deponun `.gitmodules` dosyası olduğunda Refs kenar çubuğuna bir **ALT MODÜLLER** bölümü eklenir (diğerlerinin altında; hiç yoksa bölüm de yoktur). Alt modül başına bir satır: bir durum işareti, yolu ve kısa bir commit.

| İşaret | Durum | Anlamı |
|---|---|---|
| `✓` | güncel | deponun kaydettiği commit'te checkout edilmiş |
| `↑` | değiştirilmiş | başka bir commit'te checkout edilmiş (ilerlemiş ya da güncellemeden dal değiştirmişsiniz) |
| `○` | klonlanmamış | `.gitmodules` içinde tanımlı ve kayıtlı, ama klasörü boş: klonlamak için güncelleyin |
| `✗` | çakışmalı | depoda alt modül üzerinde merge çakışmaları var |

| Yapılacak | Fare | Klavye |
|---|---|---|
| Alt modülü kendi sekmesinde açmak | satıra çift tıklayın | seçin, `Enter` (palet: `Refs: Alt Modülü Sekmede Aç`) |
| Menü: **Güncelle (gerekirse klonla)**, **Başlat**, **Yeni sekmede aç** | satıra sağ tıklayın | seçin, bağlam menüsü tuşu (varsayılan `.`) |
| Seçili alt modülü güncellemek (gerekirse önce klonlar) | menü | `Ctrl+P`: `Refs: Alt Modül Güncelle (Seçili)` |
| Klonlamadan `.git/config` içine kaydetmek | menü | `Ctrl+P`: `Refs: Alt Modül Başlat (Seçili)` |
| Her alt modülü güncellemek | | `Ctrl+P`: `Git: Tüm Alt Modülleri Güncelle` |
| Bölüme atlamak | başlığına tıklayın | `Ctrl+P`: `Gezinme: Refs > Alt Modüllere Odaklan` |

Bir güncelleme alt modülün uzak deposundan fetch edebilir; bu yüzden diğer her aktarım gibi **Ayarlar > SSH ve kimlik bilgileri > HTTPS kimlik bilgileri** ayarını izler. Klonlanmamış bir alt modülün sekme olarak açılacak bir şeyi yoktur; durum çubuğu bunu söyler ve Güncelle'yi işaret eder.

## Alt modülleri güncel tutmak

**Ayarlar > Eşitleme > Alt modülleri güncel tut** (`general.keep_submodules_up_to_date`, varsayılan olarak kapalı). Açıkken başarılı bir **checkout** veya **pull**, ardından `git submodule update --init --recursive` çalıştırır. Bu yalnızca bir alt modül, deponun artık kaydettiği durumla uyumsuzsa çalışır; böylece alt modülü olmayan bir depo yavaşlamaz. Bu güncelleme başarısız olursa checkout veya pull yine de gerçekleşmiştir ve hata bunu söyler.

## Komut satırından

```
gitpoulpi submodule list                       # yol, durum, url (--json: tam liste)
gitpoulpi submodule init [<yol>...]            # kaydet, klonlama
gitpoulpi submodule update [<yol>...] [--init] [--recursive]
```

Yol verilmezse her alt modül demektir. Aynı üçü MCP araçlarıdır (`submodule_list` bir okumadır; `init` ve `update` değişiklik yapan işlemlerdir ve `[mcp]` tablosunun `allow_mutations` ayarını izler).

## Notlar

- Yalnızca en üst düzey listelenir; bir alt modülün kendi alt modülleri, onu sekme olarak açtığınızda görünür (ya da onları güncellemek için `--recursive` kullanın).
- Yeni Git sürümleri, `protocol.file.allow` izin vermedikçe *yerel yol* olan bir alt modülü `file` aktarımıyla klonlamayı reddeder. Bu yalnızca URL'si yerel bir klasör olan alt modüller için önemlidir; bunları kullanıyorsanız genel Git yapılandırmanızda ayarlayın.
