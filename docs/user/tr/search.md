[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · **Türkçe** · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Arama

`Ctrl+Alt+F` (veya `Ctrl+P`, ardından `Ara: Çalışma Alanı...`), **üzerinde
çalıştığınız tüm depolar** üzerinde bir bulanık arama açar: varsa etkin
[çalışma alanının](workspaces.md) depoları, yoksa açık sekmeler. **Genel arama** düğmesi
(başlık çubuğunda, dişlinin hemen solundaki büyüteç ve etiketi) de onu açar. Şunları bulur:

- **depolar** (ada, sonra yola göre),
- **dallar** (yerel ve uzak izleyen) ve **tag'ler**,
- **worktree'ler** (klasöre, sonra yola ve dala göre),
- **stash'ler** (mesaja göre),
- **commit'ler** (konuya göre; bir commit'in gövdesi yalnızca konusu eşleşmediğinde
  ve üç karakter veya daha fazlasını yazdığınızda aranır),
- **pull veya merge request'ler** ve **issue'lar** (başlığa, `#numara`ya, yazara ve
  dala göre).

![Açık iki depo üzerinde arama penceresi: "main" ile eşleşen dallar, bir stash ve bir commit; her birinin deposu sağda](../images/workspace-search.png)

Sonuçlar türe göre gruplanır, her grubun içinde en iyi eşleşme önce gelir. Her
satırda türün simgesi, vurgulanmış eşleşen karakterler, soluk bir ayrıntı (bir
dalın upstream'i, bir request'in yazarı ve dalı, bir stash'in konumu) ve sağda
deponun adı bulunur; böylece `main` dalı olan iki depo asla karıştırılmaz. Bir
depo commit'lerini hâlâ dizinliyorsa başlıkta bir döner simge görünür (büyük bir
depoda ilk arama biraz sürebilir); diğer türler hemen oradadır ve commit'ler
hazır olur olmaz katılır.

## Oraya gitmek

| Tuş | Ne yapar |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `PageUp` / `PageDown`, `Home` / `End` | Sonuçlar arasında gezer (grup başlıkları atlanır). |
| `Enter` | Sonuca gider: deponun sekmesi açılır (veya ona geçilir), sonra sonuç gösterilir. |
| `Esc` | Aramayı kapatır. |
| `Alt+1` .. `Alt+8` | Bir türü açar veya kapatır (listenin üstündeki çipler, kendi sıralarıyla). `Alt+0` hepsini açar. |

"Oraya gitmek" ne demek:

| Sonuç | Enter |
|---|---|
| Dal veya tag | Refs panelinde onu, Log'da commit'ini seçer. |
| Commit | Log'da onu seçer; yüklenenden eskiyse önce Log'un daha fazlasını yükler. |
| Stash | STASH'LER bölümündeki satırını seçer. |
| Worktree | Bir sekme olarak açar. |
| Pull/merge request, issue | Kenar çubuğundaki satırını seçer (o deponun sekmesinde). |
| Depo | Sekmesine geçer (açık değilse açar). |

`Ctrl+N Backspace` (veya paletteki `Geri`), bir ajanın sizin için yaptığı herhangi
bir görünüm değişikliği gibi, bulunduğunuz yere döndürür. Bir yere gitmekle hiçbir
şey checkout edilmez, merge edilmez veya değiştirilmez.

Fareyle: bir satıra tık onu seçer, çift tık oraya gider, bir çipe tık o türü açar
veya kapatır, tekerlek kaydırır ve dışarıya tık pencereyi kapatır.

## Önekle daraltmak

Çipler ne derse desin, tek bir türü aramak için sorguyu bir önekle başlatın:

| Önek | Tür |
|---|---|
| `r:` | depolar |
| `b:` | dallar |
| `t:` | tag'ler |
| `w:` | worktree'ler |
| `s:` | stash'ler |
| `c:` | commit'ler |
| `pr:` (veya `mr:`, `p:`) | pull veya merge request'ler |
| `i:` | issue'lar |
| `#` | pull/merge request'ler ve issue'lar (`#12` 12 numarayı bulur) |

`fix: login` düz bir sorgudur, önek değildir: yalnızca tek bir harf (veya `pr:` /
`mr:`) ve ardından iki nokta ya da baştaki bir `#` sayılır.

## Nasıl eşleştirir

Komut paletiyle aynı üç kademe: yazdığınız olduğu gibi, sonra olduğu gibi herhangi
bir yerde, sonra karakterleri sırayla ama ayrık. Daha erken bir eşleşme ve daha
kısa bir metin kazanır. Büyük ve küçük harf aynıdır.

## Pull request'ler ve issue'lar

GitPoulpi'nin her depo için zaten yüklediği şeyden gelirler (kenar çubuğundaki PULL
REQUEST'LER ve ISSUE'LAR bölümlerinin arkasındaki listeler); deponun `.gitpoulpi/`
klasöründeki küçük bir önbellek dosyasında tutulur. Arama **GitHub'a veya GitLab'a
asla çağrı yapmaz**: request'leri bir pencerede hiç yüklenmemiş bir depoda
bulunacak bir şey yoktur ve gördüğünüz, kenar çubuğunun son yenilemesi kadar
tazedir. Bağlı bir hesap yoksa bulunacak bir şey de yoktur ve yapılandırılacak
bir şey de.

## Komut satırından

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` aynısını
yapar ve JSON yazdırır ([CLI başvurusu](cli-reference.md#çalışma-alanları-ve-arama)
sayfasına bakın); MCP sunucusunun bir `search` aracı vardır, böylece bir ajan sizin
bulabildiğinizi bulabilir. [Yapılandırma başvurusundaki](config-reference.md#search-çalışma-alanı-araması)
`search.commit_limit`, her deponun kaç commit'inin dizinleneceğini sınırlar (`0`
commit aramasını kapatır).

## Tuşu değiştirmek

`Ara: Çalışma Alanı...` bir palet komutudur ve tuş `global.workspace_search`
eylemidir: Ayarlar > Klavye ve fare altında veya `[keybindings.global]`
tablosunda yeniden atayın (`workspace_search = "Ctrl+Alt+F"`).
