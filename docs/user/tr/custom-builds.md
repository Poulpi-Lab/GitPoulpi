[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · **Türkçe** · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Özel derlemeler

Daha küçük bir ikili isteyenler için: etkileşimli bir betik hangi dillerin, barındırma entegrasyonlarının, temaların, logo renklerinin ve özelliklerin dahil edileceğini sorar, ardından yalnızca bunları iyileştirilmiş profille derler. Normal derleme ve resmî sürüm değişmez (her şey açık).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd veya PowerShell (scripts/build-custom.ps1)
```

Çıktı `target/custom/` içine düşer, resmî `target/dist/` üzerine asla yazılmaz. Betik her ikilinin boyutunu ve o derleme varsa `target/dist/` ile farkı yazdırır. Ayrıca ikililerin yanına `BUILD-FEATURES.txt` yazar.

## Ne seçersiniz

| Seçim | Kapatırsanız |
|---|---|
| Diller | Yalnızca İngilizce (her zaman derlenir) ve işaretlediğiniz diller. Ayarlar seçicisi yalnızca derlenmiş dilleri listeler. |
| Temalar | GitPoulpi Dark, GitPoulpi Light ve Terminal her zaman derlenir (`system` için Dark ve Light gerekir). Diğer her tema aileye göre seçilir (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` bunları yazdırır). Seçilmeyen temalar derlenmez: Ayarlar açılır kutusunda ve `gitpoulpi theme list` içinde yoktur ve onu adlandıran bir yapılandırma, bilinmeyen bir tema gibi olağan uyarıyla geri düşer. |
| Logo renkleri | Hakkında penceresinin piksel ahtapotu 20 renk düzeniyle gelir (`--list-logo-colors`). Derlemeye girecek olanları (en az bir), varsayılanı (Hakkında onu ilk gösterir; derlenmemiş olan kayıtlı bir `ui.logo_color` değerinin de düştüğü yerdir; derlenenlerden biri olmalıdır; resmî varsayılan: lavender) ve Windows çalıştırılabilir dosya simgesinin rengini (herhangi bir renk düzeni, varsayılan: logo varsayılanı) seçin. Tek renk düzeniyle logo tıklayınca yine döner ama o renkte kalır ve renk listesi (`c`, sağ tık) sunulmaz. |
| Entegrasyonlar (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Hiçbiri olmadan: yalnızca URL ile klonlama; issue yok, pull/merge request yok. |
| YZ | YZ düğmeleri, komutları ya da `ai` CLI/MCP araçları yok. |
| MCP | `gitpoulpi-mcp` ikilisi ve `gitpoulpi mcp` komutu yok. `ui` ajan komutları kalır. |
| LFS, Worktree, Alt modüller | İlgili yönetim ekranları ve komutlar yok olur; "not built into this binary" türlü bir yanıt döner. |
| Avatar indirme | Yalnızca baş harf rozetleri, ağdan getirme yok. |
| Görsel önizleme | Görsel dosyaları sade ikili dosya uyarısını gösterir; **Şununla düzenle** yine çalışır. |
| Radar | Radar sekmesi, `Alt+R` ve palet komutu yok; Radar hiç açılmaz. Bir entegrasyon gerekir (hiçbiri yoksa zaten etkisizdir). |
| Mermaid | Mermaid gitGraph olarak kopyala yok (Log menüsü, `Shift+Y`, palet) ve CLI/MCP içinde `log --format mermaid` yok. Ayrıca bir [konuşmanın](conversations.md) `mermaid` bloğunun altında **Diyagramı aç** düğmesi yok (blok kod olarak kalır) ve *Konuşma: Kaydın Diyagramını Aç* palet satırı yok. |
| Board | Kanban pano görünümü yok (ISSUE'LAR başlığındaki düğme, palet) ve `hosting board` komutu yok. Bir entegrasyon gerekir. |
| Conversation | Bir issue'nun veya isteğin uygulama içi [konuşma](conversations.md) görünümü yok: bir issue satırında, bir pano kartında veya bir Radar satırında Enter eskisi gibi tarayıcıyı açar, *Konuşmayı aç* satırı ve *Konuşma: ...* palet komutları yoktur, bir konuşmada **YZ ile Yaz** da yoktur (inceleme, issue formu ve pull request formunun yazma kutuları kendilerininkini korur). CLI ve MCP'nin konuşma komutları (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) ve `ai hosting-reply` kalır. Bir entegrasyon gerekir. |
| Review | İncelemenin Genel Bakış sayfası konuşma görünümüdür, bu yüzden Review içeren bir derleme, *Conversation* ne derse desin konuşma görünümünü de içerir. Uygulama içi kod inceleme görünümü yok (satır menüsü, bir pull request satırında Enter, palet, Radar `Review`) ve YZ inceleme yardımı yok. `hosting review` CLI/MCP komutları kalır (Gerrit ve ajanlar bunları kullanır). Bir entegrasyon gerekir. |
| Geçmiş ekranı (gelişmiş geçmiş yeniden yazma: yazarlar, tarihler, iletiler, içerik) | Geçmişi Yeniden Yazma Ekranı yok: `Yeniden Yaz:` palet komutları, ekran kısayolu, tuşları ve yardım bölümü yok, Log menülerinde "Geçmişi yeniden yaz" satırları yok, Değiştir (Amend) penceresinde "Geçmişi Daha Derin Değiştir..." düğmesi yok, "YZ ile kural oluştur" ve betik YZ'si yok, Eşitleme ayarları sayfasında betik sınırları yok ve YZ ayarlarında geçmiş yeniden yazma YZ blokları yok. `rewrite` CLI/MCP komutları, `ai rewrite-script`, `ai rewrite-rules` ve `ui propose rewrite` komut ağacından kalkar. `.mailmap` kimlikleri (`mailmap`, `log.use_mailmap`) buna dahil değildir ve kalır. |
| Genel arama | Başlık çubuğunda Genel arama düğmesi, `Ctrl+Alt+F`, `Ara: Çalışma Alanı...` palet komutu ve `gitpoulpi search` CLI/MCP komutu yok. Log bul çubuğu (`ui show search`) farklı bir özelliktir ve kalır. |
| Çalışma alanları | Pencere tek varsayılan sekme listesiyle ("Çalışma alanı yok") çalışır: çalışma alanı çipi, Çalışma Alanları sekmesi, `Alt+W` / `Alt+J`, `Çalışma Alanı:` palet komutları, "Çalışma alanına ekle" sekme satırları, `gitpoulpi workspace` CLI/MCP komutları ve `commit` ile `search` üzerinde `--workspace` bayrağı yok. Sekmeler ve oturumlar her zamanki gibi çalışır. Başka bir derlemenin yazdığı `workspaces.toml` ne okunur ne değiştirilir; bir çalışma alanının varsayılan profili ve YZ talimatları uygulanmaz. Radar ve Genel arama açık sekmeleri kapsar. |
| Yeniden yazma betikleri (gömülü Lua) | Geçmiş yeniden yazmadaki Script kuralları, türü belirli bir "bu ikili dosyaya dahil değil" yanıtıyla reddedilir ve hiçbir Lua bağlanmaz. Geçmiş ekranının kendisi kalır (kendi seçimi vardır). Geçmiş ekranı kapalıyken anlamsızdır ve uyarı verilir. |

Yapılandırma dosyalarınız derlemeler arasında paylaşılır: derlenmemiş bir şey için bir ayar saklanır ve yok sayılır.

## Etkileşimli

Bash betiği `whiptail` veya `dialog` onay listelerini kullanır (üç ekran, ardından bir özet), ikisi de yoksa numaralı istemleri kullanır. PowerShell betiği varsa `Out-GridView` kullanır, yoksa istemleri. Önceki seçimleriniz önceden işaretlenir.

## Etkileşimsiz

Herhangi bir bağımsız değişken soruları atlar ve doğrudan `cargo run -p xtask -- build-custom` komutuna gider:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # önceki seçimler, yalnızca cargo komutunu yazdır
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Seçenek | Anlamı |
|---|---|
| `--langs fr,de` / `all` / `en` | İngilizce dışındaki diller. Kodlar yerel ayar dosyası adlarıdır (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Sağlayıcılar: GitHub, GitLab (kendi barındırdığınız dahil), Azure DevOps (Services ve Server), Bitbucket (Cloud ve Data Center) (barındırma), Codeberg dahil Gitea ve Forgejo (`gitea`) (barındırma), Jira, Trello, Linear, YouTrack (Cloud ve Server), Redmine (issue takipçileri), Gerrit Code Review (`gerrit`, yalnızca inceleme), yerel Markdown issue'ları (`local-issues`, bir klasör veya Obsidian kasası). `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` öğelerinin her biri kendi cargo özelliğidir; boyutlar [binary-size](../../developer/binary-size.md) içinde. |
| `--themes all` / `none` / `nord,catppuccin` | Derlemeye girecek tema aileleri (varsayılan `all`). `--list-themes` aileleri yazdırır. |
| `--logo-colors all` / `violet,teal` | Derlemeye girecek logo renk düzenleri (varsayılan `all`, tablonun sırasında tutulur, bu dönme sırasıdır). `--list-logo-colors` bunları yazdırır (`*` = resmî varsayılan). |
| `--logo-default teal` | Varsayılan renk düzeni; derlenenlerden biri olmalıdır (varsayılan: lavender, yoksa derlenen ilk). |
| `--icon-color sun` | Windows çalıştırılabilir dosya simgesinin rengi (varsayılan: logo varsayılanı). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Her biri varsayılan olarak açıktır. |
| `--[no-]history-rewrite` | Geçmiş ekranı (gelişmiş geçmiş yeniden yazma: yazarlar, tarihler, iletiler, içerik). Varsayılan olarak açık. |
| `--[no-]global-search` | Genel arama (başlık çubuğu düğmesi, `Ctrl+Alt+F`, `gitpoulpi search`). Varsayılan olarak açık. |
| `--[no-]workspaces` | Çalışma alanları (adlandırılmış depo grupları). Varsayılan olarak açık; kapalıyken yalnızca varsayılan sekme listesi kalır. |
| `--[no-]rewrite-scripts` | Bir yeniden yazmanın Script kuralları için gömülü Lua. Varsayılan olarak açık; Geçmiş ekranı kapalıyken uyarıyla yok sayılır. |
| `--save <name\|file>` | Çözümlenen seçimleri kayıtlı seçim olarak saklar (bir ad veya dosya yolu). |
| `--preset <name\|file>` | Kayıtlı bir seçimden başlar (bir ad veya dosya yolu); diğer seçenekler satırın neresinde olursa olsun onu geçersiz kılar. |
| `--list-presets`, `--delete-preset <name>` | Kayıtlı seçimleri ve klasörlerini listeler / birini ada göre siler. |
| `--last` | `target/custom-build/last-choices.env` dosyasını yeniden kullanır; diğer seçenekler onu geçersiz kılar. |
| `--full` | Hiç seçim yok (her şey açık). |
| `--profile custom\|dist\|release` | Varsayılan `custom` (`dist` ayarları, ayrı çıktı klasörü). |
| `--dry-run` | Tam cargo komutunu yazdırır ve durur. |
| `-- <cargo args>` | cargo'ya iletilir, örneğin `--target x86_64-unknown-linux-musl`. |

PowerShell adları: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

Tema seçimi derlemeye `GITPOULPI_THEMES` ortam değişkeni olarak ulaşır (`all`, `none` ya da virgülle ayrılmış bir aile listesi); `gitpoulpi-config`'in derleme betiği bunu okur ve diğer paletleri derleme zamanında atar; tercih ederseniz sade bir `cargo build` için kendiniz ayarlayın. `gitpoulpi --version` derlenen kümeyi gösteren bir `themes:` satırı yazdırır.

Logo seçimleri `GITPOULPI_LOGO_COLORS` (`all` ya da virgülle ayrılmış slug listesi), `GITPOULPI_LOGO_DEFAULT` ve `GITPOULPI_ICON_COLOR` ortam değişkenleridir; bunları `gitpoulpi-tui`'nin (`crates/gitpoulpi-logo-build/colourways.json` dosyasını Hakkında penceresinin gömdüğü tabloya süzer) ve üç ikili crate'in (simge) derleme betikleri okur. Renklerin kaynağı `crates/gitpoulpi-logo-build/colourways.json` olarak kalır.

### Windows çalıştırılabilir dosya simgesi

Hedef Windows olduğunda `gitpoulpi`, `gitpoulpi-tui` ve `gitpoulpi-mcp` çalıştırılabilir dosyaları simge olarak ahtapotu alır; seçilen simge renginde, 16 ila 128 px boyutlarında, derleme zamanında üretilir (Python ya da ImageMagick gerekmez). `winresource` crate'iyle gömülür ve platformun kaynak derleyicisini ister (MSVC'de Windows SDK'dan `rc.exe`, GNU'da `windres`). Eksikse derleme yalnızca bir uyarı yazdırır ve çalıştırılabilir dosya varsayılan simgesini korur. GNU araç zincirinde yalnızca `gitpoulpi` simgeyi taşır (kaynak aksi halde iki kez bağlanırdı). Linux ve macOS derlemeleri, renk tablosu dışında bunların hepsini yok sayar.

## Kayıtlı seçimler

Tüm seçimi baştan yapmadan yeni bir sürüm derlemek için seçimi bir adla kaydedin ve sonra ondan başlayın. `--preset` ile birlikte verilen seçenekler kayıtlı değerleri yine de geçersiz kılar.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Kayıtlı seçimler düz `ANAHTAR=değer` dosyalarıdır (`last-choices.env` biçimi, bir kabuk tarafından okunabilir). GitPoulpi yapılandırma dizinindeki `custom-builds` klasöründe, `target/` dışında bulunurlar; bu yüzden `cargo clean` onları silmez:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (varsayılan olarak `~/.config/gitpoulpi/custom-builds/`)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` bu klasörü yapılandırmayla birlikte taşır. `--preset` ve `--save` bir dosya yolunu da kabul eder (içinde `/` veya `\` bulunan ya da `.env` ile biten bir değer); böylece bir seçim dotfiles deposunda tutulabilir.

GitPoulpi'nin başka bir sürümünün yazdığı bir seçim yine de yüklenir. Bilinmeyen bir anahtar ya da artık var olmayan bir dil, entegrasyon, tema veya renk, uyarıyla yok sayılır. Dosyada eksik olan bir anahtar (sonradan eklenen bir özellik) varsayılan değerini alır ve sonradan eklenen bir entegrasyon dahil edilir; ikisi de standart hata çıktısında bildirilir.

Kayıtlı bir seçim varsa etkileşimli kip önce "Şuradan başla" seçeneğini sunar (önceki seçimler, kayıtlı bir seçim veya her şey açık) ve sonunda geçerli seçimin bir adla kaydedilip kaydedilmeyeceğini sorar.

## Neyi tasarruf eder

Linux'ta, 2026-10-29 tarihinde `65999ef5c` commit'inde, tam derlemeden ölçüldü; ondan farkı parantez içinde
(ayrıntılar `docs/developer/binary-size.md` içinde):

| Derleme (aksi belirtilmedikçe `full`'dan) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Tam (her şey açık) | 64.6 | 50.5 | 31.7 |
| Yalnızca İngilizce | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP kapalı | 61.6 (-3.0) | 49.6 (-0.8) | derlenmedi |
| YZ kapalı | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| Tüm entegrasyonlar kapalı | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| YZ + entegrasyonlar + avatar getirme kapalı | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| Geçmiş ekranı kapalı (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Yeniden yazma betikleri kapalı | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Genel arama kapalı | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Çalışma alanları kapalı | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar kapalı | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board kapalı | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review kapalı | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid kapalı | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS kapalı | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree kapalı | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Alt modüller kapalı | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Avatar getirme kapalı | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Görsel önizleme kapalı | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Her şey kapalı, yalnızca İngilizce (LFS açık) | 35.2 (-29.4) | 28.9 (-21.5) | derlenmedi |
| **Her şey kapalı, yalnızca İngilizce** | **34.9 (-29.7)** | **28.8 (-21.6)** | derlenmedi |

## Notlar

- Özel bir derleme, her `dist` derlemesi gibi yavaştır (fat LTO, birkaç dakika).
- `build-opti` gibi sccache olmadan çalışır.
- Dil kodları gönderilen yerel ayarlara göre denetlenir; bilinmeyen bir kod, geçerli olanların listesiyle reddedilir.
