[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · **Türkçe** · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Yerel issue'lar ve Obsidian

Issue'larınızı bir Markdown notları klasörü olarak tutun: hesap yok, token yok, ağ yok. Klasör deponun içinde
(`docs/issues/`, kodla birlikte commit'lenir) ya da diskte herhangi bir yerde olabilir; örneğin bir
[Obsidian](https://obsidian.md) kasasının alt klasörü. GitPoulpi notları okur ve yazar. Obsidian, herhangi bir
düzenleyici veya bir kabuk aynı anda onları düzenleyebilir ve listeler bunu izler.

![Kenar çubuğundaki yerel Markdown issue'ları ve bir notun satır menüsü: notu aç, kapat, etiketlerini veya kilometre taşını düzenle](../images/local-issues.png)

## Kurulum

1. **Ayarlar > Entegrasyonlar > Issue takipçileri > Local Markdown > Profil ekle.** Klasörü yazın ya da
   **Gözat...**'a basıp seçin. Göreli bir klasör (`docs/issues`) bağlı olduğu her depoya göre okunur; mutlak bir klasör
   (`/home/me/Vault/Projects/Issues`) her yerde aynıdır. Girilecek bir token yoktur.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. Bir depoda kenar çubuğunun **ISSUE'LAR** bölümünü açın, **Local Markdown - <ad>** sağlayıcısını seçin (klasör onunla
   birlikte seçilir; başka bir klasör isterseniz depo açılır listesi klasörü ve doğrudan alt klasörlerini listeler).
   - CLI: `gitpoulpi hosting link issues --id <profile>` (profilin klasörü varsayılan `--project`'tir).

Hiçbir şey kendiliğinden bağlanmaz: her issue kaynağı gibi depo başına isteğe bağlıdır.

## Bir nottaki yorumlar

ISSUE'LAR bölümünde bir notun satırında veya kartında `Enter` onun konuşmasını açar: notun metni, sonra yorumlar. Yorumlar notun dosyasının kendisinde, sondaki bir `## Comments` bölümünde tutulur; her biri bir `### <yazar> - <UTC saati>` girdisidir (düzenlenmiş olana `(edited <saat>)` eklenir). Bu başlığın üstündeki hiçbir şeye dokunulmaz; böylece Obsidian ve bir kabuk aynı dosyayı okur ve düzenler. Yerel yazarın yorumlarını yazabilir, düzenleyebilir ve silebilirsiniz (profilin `local.me` değeri, yoksa bağlı kullanıcı; *Bana atananlar*'ın kullandığı ad); başkasının yorumlarını değil. Kutu, önizlemeli olağan Markdown yazma kutusudur ve *<ad> olarak yazılıyor (yerel not)* yazarı belirtir. Ağ üzerinden hiçbir şey gitmez. Elle yazdığınız bir `## Comments` başlığı da bu şekilde okunur.

## Notlar

Issue başına bir not, `<id>-<slug>.md` adıyla (`12-crash-on-empty-name.md`) ve bir frontmatter bloğuyla:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- Yeni issue'lar `max(id) + 1` alır.
- GitPoulpi yalnızca sahip olduğu özellikleri yazar (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Diğer her özellik, gövde, satır sonları ve anahtarların sırası bayt bayt korunur.** Yazmalar atomiktir (bir geçici
  dosya, sonra bir yeniden adlandırma).
- Okuma hoşgörülüdür: `id`'si olmayan bir not onu dosya adından alır, `status`'u olmayan bir not `todo`'dur,
  frontmatter'ı olmayan bir not başlığını ilk `# başlık`tan alır. Issue olmayan bir dosya (`readme.md`) yok sayılır.
- Bir durum serbest metindir. **Açık**, "tamamlandı durumu değil" demektir.

### Bir nottaki yorumlar

Bir nota yapılan yorum notun kendisine, son bir `## Comments` bölümüne yazılır (ağ yok, Obsidian'da da düzgün okunur):

```markdown
(notun metni)

## Comments

### kai - 2026-11-15T10:00:00Z
İlk yorum, **herhangi bir Markdown**.

### ann - 2026-11-15T11:30:00Z (edited 2026-11-15T12:00:00Z)
İkincisi.
```

- Bir girdi, tam olarak `### <yazar> - <UTC saati>` olan bir satırla başlar; bir düzenlemeden sonra ` (edited <UTC saati>)` eklenir; metni bir sonraki girdiye kadar sürer. Bölüm, bir kod çitinin dışındaki son `## Comments` başlığıdır ve dosyanın sonuna kadar sürer; bu yüzden onun ardına hiçbir şey yazmayın.
- Bir girdi veya başlık gibi okunacak bir yorum satırı başında ters eğik çizgiyle saklanır (`\### ...`; Markdown aynı metni gösterir) ve geri okunurken onsuz okunur.
- Gönderme sona ekler ve üstündeki her şeyi bayt bayt korur (satır sonları dahil); bir düzenleme veya silme yalnızca bölümü yeniden yazar. Bir gönderi ayrıca notun `updated` değerini ilerletir (frontmatter'ı olmayan nota bir tane verilmez).
- Yazar, profilin yerel yazarıdır (Yerel issue ayarlarındaki "ben"). Yalnızca kendi yorumlarınızı düzenleyebilir veya silebilirsiniz; bir yorumun kimliği saati ve yazarıdır, aynı saniyedeki iki gönderi ikincisini bir saniye ilerleterek ayırt edilir.
- Bir not için yanıt, dahili yorum ve olay geçmişi yoktur.

## Neler yapabilirsiniz

| ISSUE'LAR bölümünde | Klavye / palet |
|---|---|
| Yeni issue (`+` veya `n`): başlık, açıklama, durum, atananlar, etiketler, kilometre taşı | `Barındırma: Yeni Issue...` |
| Satır menüsünden atananları / etiketleri / kilometre taşını düzenle. Klasörün zaten kullandıklarından seçin ya da **yeni bir etiket veya oturum adı yazın** ve `“...” ekle` satırını alın | `Barındırma: Etiketleri Düzenle...` ve benzerleri |
| **Issue'yu kapat** / **Issue'yu yeniden aç** (tamamlandı durumunu veya diğer ilk durumu ayarlar) | `Yerel Issue'lar: Issue'yu Kapat...` / `Issue'yu Yeniden Aç...` |
| **Notu aç** (`.md` için varsayılan programınız) | `Entegrasyonlar: Issue Aç...` |
| **Obsidian'da aç** (yalnızca klasör bir kasanın içindeyse, yani üstünde bir `.obsidian/` klasörü varsa) | `Yerel Issue'lar: Obsidian'da Aç...` |
| Filtreler: Açık / Kapalı / Tümü, "Bana atananlar" | filtre açılır listesi |

"Bana atananlar", profilin `me` ayarıyla, o yoksa bağlı adla, o da yoksa profil adıyla eşleşir.

Dallar ve commit'ler GitHub'daki gibi çalışır: "Issue'dan dal oluştur" `12-crash-on-empty-name` verir ve bir commit
mesajındaki `#12` ya da bir dal adındaki `12-...` tanınır.

GitPoulpi dışında yapılan değişiklikler (Obsidian, bir kabuk, bir `git pull`) yaklaşık bir saniye içinde görünür:
klasörün dosya adlarına ve değiştirilme zamanlarına arka plan iş parçacığında bakılır, asla çizim iş parçacığında değil.

## Yapılandırma dosyası

`config.toml` içinde, profilin yanında (her anahtar isteğe bağlıdır):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # panonun sütun sırası
done_status = "done"                            # bir issue'yu kapatır
reference_prefix = "LOC"                         # #12 yerine LOC-12 olarak gösterilir
me = "kai"                                       # "bana atananlar"ın kim olduğu
```

Notların kullandığı ama `statuses` içinde listelenmeyen durumlar, listelenenlerin ardından gelir. `statuses`,
[panonun](#kanban-panosu) sütunlarının sırasıdır (`gitpoulpi hosting issue options` bunu yazdırır).

## Kanban panosu

![Bir notlar klasörünün panosu: durum başına bir sütun, atananları, etiketleri ve kilometre taşıyla kartlar](../images/issue-board.png)

Yalnız çalışıyorsanız ya da issue'larını kodun yanında tutan bir ekipteyseniz, klasör aynı zamanda bir Kanban
panosudur. **ISSUE'LAR başlığı > Pano** ya da paletin **Issue'lar: Panoyu Aç** komutu onu pencerenin üzerinde açar;
`Esc` kapatır.

- **Sütunlar** durumlardır: önce `statuses` içinde listelenenler, o sırayla, sonra bir notun kullandığı diğer her durum.
  Boş bir sütun yine de gösterilir. Her başlık kart sayısını taşır.
- **Kartlar** issue numarasını, başlığı (iki satır), atananları baş harfler olarak, etiketleri ve kilometre taşını
  gösterir. Sütunların hepsi sığmadığında pano yana kayar (altındaki çubuk, `Shift` ile tekerlek veya oklar).
- **Bir kartı taşımak** için onu başka bir sütunun üzerine sürükleyin (sütun, işaretçinin altında yanar),
  `Shift+Left` / `Shift+Right` kullanın ya da sütunların bir açılır listesi olan **Pano: Kartı Şuraya Taşı...** (`m`)
  komutunu kullanın. Kart hemen taşınır; notun `status` değeri [notlar bölümündeki](#notlar) gibi yeniden yazılır, dosyada
  başka hiçbir şey değişmez. Yazma başarısız olursa kart geri döner ve hata durum çubuğunda görünür.
- Bir kartı `Enter` veya çift tıkla **açın**: notun [konuşması](conversations.md), notun metni ve yorumlarıyla birlikte pencerede açılır (kart menüsündeki **Notu aç** dosyayı `.md` için varsayılan programınızda açar). `e` atananlarını,
  etiketlerini veya kilometre taşını düzenler, `.` veya bir sağ tık kart menüsünü açar, `n` seçili sütunda yeni bir issue
  başlatır (form kapandığında panoya geri dönülür).
- **Filtre**: atanana (`a`), etikete (`l`), kilometre taşına (`M`) veya metne (`/`) göre; `x` onları temizler. Filtreler
  açılır listelerdir: daraltmak için yazın, `Enter` seçer.
- **Canlı**: Obsidian'dan veya bir kabuktan değiştirilen bir not, kartını bir saniye içinde taşır.

Her fare hareketinin bir klavye tuşu ve bir palet komutu vardır (**Pano: ...**, [tuş atamaları](keybindings.md)
sayfasında listelenir).

## Obsidian Bases

**Palet: `Yerel Issue'lar: Obsidian Base Oluştur`** (CLI: `gitpoulpi hosting issue base`) notların yanına bir kez
`issues.base` yazar: bir *tablo* görünümü ve `status` değerine göre gruplanmış, `statuses` sırasında bir *kanban*
görünümü. Aynı issue'ları orada bir pano olarak görmek için onu Obsidian'da açın. GitPoulpi var olan bir `.base`
dosyasını (sizinkini ya da önceki bir tanesini) asla düzenlemez: reddeder ve bunu söyler. Biçim Obsidian'ın Bases
söz dizimini izler (<https://obsidian.md/help/bases/syntax>); kanban görünümü yeni bir Obsidian gerektirir.

## CLI ve MCP

Her issue kaynağıyla aynı komutlar onda çalışır: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Notlar için dört tane daha vardır:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` ve `hosting issue base`.
Bkz. [CLI başvurusu](cli-reference.md).

## Sınırlar

- Seçicide yeni bir **kilometre taşı** yazılamaz (klasörün zaten kullandığı birini seçin), ama bir nottaki
  `milestone:` herhangi bir şey olabilir ve kaydedilir kaydedilmez görünür.
- Yalnızca doğrudan klasördeki notlar issue'dur (alt klasörler değil).
- Silme yoktur: notu dosya yöneticinizde veya Obsidian'da kaldırın (çöp kutusu sizindir).
