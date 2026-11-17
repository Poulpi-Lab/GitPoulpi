[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · **Türkçe** · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Yapılandırma başvurusu

`config.toml` dosyasının her anahtarı, varsayılan değeri ve ne yaptığı. Dosyayı elle
düzenlemeniz nadiren gerekir: [Ayarlar ekranı](settings.md) (`Ctrl+,`) siz bir şeyleri
değiştirdikçe dosyayı yazar, `gitpoulpi config set` de öyle.

## Nerede bulunur ve bir değer nasıl seçilir

| | |
|---|---|
| Dosya | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` bunu yazdırır. |
| Başka bir klasör | `GITPOULPI_CONFIG_DIR=<dir>` tüm yapılandırma klasörünü (dosyayı, `sessions/` klasörünü, daemon port dosyalarını) taşır: taşınabilir bir kurulum, bir test ya da kendi ayarlarınızdan uzak tutulan bir ajan için. |
| Öncelik | yerleşik varsayılan, sonra `config.toml`, sonra `GITPOULPI_*` değişkeni, sonra bir komut satırı bayrağı. En yüksek olan kazanır. Yanlış bir değer (dosyada yanlış yazılmış bir seçenek, hatalı bir değişken) bir hatadır, asla sessizce yok sayılmaz. |
| Değişkenler | Anahtarı büyük harfe çevirin ve noktaları alt çizgiye dönüştürün: `general.auto_prune` anahtarı `GITPOULPI_GENERAL_AUTO_PRUNE` olur. Yalnızca GitPoulpi'nin gerçekten okuduğu anahtarlar için (aşağıda *kullanılmıyor* olarak işaretlenenler ve `ui.log_columns_hidden` gibi listeler için değil). |
| Bayraklar | `--backend git2\|system`, o komutun başlattığı bir daemon için `general.default_backend` değerini geçersiz kılar. |
| Token'lar ve anahtarlar | Asla bu dosyada değil. Barındırma token'ları ve YZ API anahtarları sisteminizin kimlik bilgisi yardımcısına (Git Credential Manager, libsecret / `pass`, Keychain) gider; dosya yalnızca bir başvuru (`token_ref`) tutar. Yardımcı yoksa bir token'ı kaydetmek bir mesajla başarısız olur ve token yalnızca oturum boyunca kalır. |

```
gitpoulpi config path                          # dosya ve henüz var olup olmadığı
gitpoulpi config get general.auto_stash        # değer, geldiği katman (default/file/env/flag) ve onu bir şeyin okuyup okumadığı
gitpoulpi --json config get ui                 # bütün bir bölüm
gitpoulpi config set general.auto_stash always # şemaya karşı denetlenir, sonra yazılır
```

`config set` dosyayı seyrek tutar (yalnızca birinin ayarladığı anahtarlar) ve söz
edilmeyen anahtarları korur, ancak yorumları siler. Listeleri ve tüm tabloları reddeder,
`keybindings.*` hariç (boşlukla ayrılmış birden çok tuş dizisi; boş değer atamayı kaldırır).
`[[profiles]]`, `[[integrations]]` ve `[[ai.profiles]]` tablolarını Ayarlar'da ya da elle
düzenleyin.

Tablolarda **Yeniden başlatma**, değerin başlangıçta okunduğu anlamına gelir; geri kalan
her şey hemen uygulanır (ya da bir sonraki yenilemede, bir sonraki fetch'te ya da
Ayarlar kapandığında).

## `[general]`

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2; stage, merge, rebase, stash ve ağ için `git` programına geri dönüşle) ya da `system` (yalnızca kurulu `git`). Çalışan bir daemon arka ucunu korur: **yeniden başlatın** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Yerel değişiklikler bir checkout, pull, merge ya da rebase işlemini engellediğinde: `ask` (TUI'de bir iletişim kutusu; CLI ve MCP başarısız olur ve `--autostash` seçeneğini belirtir), `always` (stash, çalıştır, pop) ya da `never` (reddet). Bkz. [Otomatik stash](log-and-history.md#otomatik-stash). |
| `general.include_untracked_in_autostash` | `false` | Otomatik stash izlenmeyen dosyaları da alır. |
| `general.initial_commits_in_graph` | `5000` | Log'un ilk yüklediği commit sayısı (en az 500). Daha yüksek değer, büyük depolarda bellek ve çizim süresi maliyeti getirir. |
| `general.lazy_load_commits_in_graph` | `true` | Sona yaklaşırken kaydırdıkça daha eski commit'leri yükler. Kapalı: Log ilk sayıda durur. |
| `general.commits_per_lazy_page` | `1000` | Arka plandaki sayfa başına commit (en az 50). |
| `general.max_commits_in_graph` | `100000` | Tembel yüklemenin durduğu yer (commit başına yaklaşık 300 bayt); `0` = sınır yok. |
| `general.auto_fetch_interval_minutes` | `5` | Her N dakikada bir arka planda Fetch All, 0 ila 60; `0` = kapalı. **Yeniden başlatma.** |
| `general.auto_fetch_interval_unit` | `"auto"` | Ayarlar'ın aralığı hangi birimle gösterdiği: `minutes`, `hours` veya `auto`. Yalnızca görüntüleme içindir: aralık her zaman `auto_fetch_interval_minutes` dakikadır. `auto` (ve bu anahtarı içermeyen bir yapılandırma), dakikayı tam bölen en büyük birimi seçer (60 "1 saat", 45 "45 dakika" okunur); uymayan bir birim yok sayılır. Ayarlar iki anahtarı da yazar; daha eski bir GitPoulpi birimi yok sayar ve dakikayı yine okur. |
| `general.fetch_on_tab_open` | `true` | Bir sekme açıldığında Fetch All (GitPoulpi o depoyu 60 saniyeden kısa süre önce fetch ettiyse atlanır). |
| `general.auto_prune` | `true` | Fetch'ler artık bulunmayan uzak izleme dallarını da prune eder. |
| `general.auto_prune_tags` | `false` | Bir fetch, uzak deponun sildiği yerel tag'leri de siler (yalnızca onda bulunmuş tag'ler; hiç push etmediğiniz bir tag korunur). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Pull request / issue listelerinin ve araç çubuğu çipinin yenilenmesi, 0 ila 120; `0` = yalnızca isteğe bağlı ve olaylarda. |
| `general.hosting_refresh_unit` | `"auto"` | Ayarlar'ın yenilemeyi hangi birimle gösterdiği: `minutes`, `hours` veya `auto`; `general.auto_fetch_interval_unit` gibi okunur. Yalnızca görüntüleme içindir. |
| `general.hosting_post_as` | `""` | Deponun token'ı bir bota aitse (bir GitHub Uygulaması token'ı) barındırıcıya gönderilen mesajların hangi adla yayımlanacağı, örn. `my-app[bot]`: her yazma kutusunda gösterilir ve kendi yorumlarınızı tanımak için kullanılır. Boş: barındırıcıya sorulur, yoksa ilk gönderiden öğrenilir. Bir depo bunu geçersiz kılabilir (Ayarlar, deponun SSH ve kimlik bilgileri sayfası). |
| `general.keep_submodules_up_to_date` | `false` | Bir checkout ya da pull sonrasında, bir alt modül uyumsuzsa `git submodule update --init --recursive`. Bkz. [Alt modüller](submodules.md). |
| `general.default_branch_name` | `"main"` | Init ile oluşturulan bir deponun ilk dalı; boşsa Git karar verir. Hiçbir şeyi yeniden adlandırmaz. |
| `general.delete_orig_files_after_merge` | `false` | Tamamlanan bir merge sonrasında `*.orig` yedek dosyalarını siler. |
| `general.remember_tabs` | `true` | Son oturumun sekmelerini yeniden açar. Kapalıyken bir depo içinde başlatma yalnızca o depoyu, depo dışında başlatma ise yalnızca Ana sekmeyi açar. **Yeniden başlatma.** |
| `general.radar_enabled` | `true` | [Radar](radar.md): sekmesi, palet komutları, `Alt+R` ve arka planda getirme işi. Kapalıyken bunların hepsi gizlenir ve onun için hiçbir şey getirilmez. Hemen uygulanır; `radar` özelliği olmayan bir derlemede gösterilmez. |
| `general.startup_tab_in_repo` | `"current-repo"` | Başlatma, içinde başladığı klasör bir deponun içindeyken hangi sekmede açılır: `"current-repo"` (o deponun sekmesi, gerekirse açılır), `"last"` (son çıkışta etkin olan sekme), `"home"` veya `"radar"`. `--repo` ile verilen depo her zaman kendi sekmesini açar. `"last"`, `remember_tabs` kapalıysa ya da hatırlanan bir şey yoksa Ana olur; `"radar"`, Radar kapalıysa veya derlemede yoksa Ana olur. Başlangıçta bir kez okunur; yeniden başlatma gerekir. |
| `general.startup_tab_outside_repo` | `"home"` | Başlatma, içinde başladığı klasör bir depo değilken (örneğin kullanıcı klasörü) hangi sekmede açılır: `"home"`, `"last"` veya `"radar"`. `"last"`, `remember_tabs` kapalıysa ya da hatırlanan bir şey yoksa Ana olur; `"radar"`, Radar kapalıysa veya derlemede yoksa Ana olur. Başlangıçta bir kez okunur; yeniden başlatma gerekir. |
| `general.startup_tab_launcher` | `"last"` | GitPoulpi'nin oluşturduğu masaüstü veya Başlat menüsü kısayolundan (`--launcher` iletir) başlatıldığında, klasör ne olursa olsun hangi sekmede açılır: `"home"`, `"last"` veya `"radar"`. Yukarıdaki iki ayara göre önceliklidir. `"last"`, `remember_tabs` kapalıysa ya da hatırlanan bir şey yoksa Ana olur; `"radar"`, Radar kapalıysa veya derlemede yoksa Ana olur. Başlangıçta bir kez okunur; yeniden başlatma gerekir. |
| `general.group_worktree_tabs` | `true` | Bir deponun bağlı worktree'lerinin sekmelerini o deponun sekmesinin hemen ardında tutar: bir worktree açıldığında, bir worktree sekmesi uzağa sürüklendiğinde (grubunun yanına geri oturur; depo sekmesi grubun başıdır ve worktree'leri onu izler) ve ayar açıldığında. Kapalıyken sekmeler bıraktığınız yerde kalır. |
| `general.undo_depth` | `100` | Geri Al'ın kaç işlem geriye gidebileceği; `0` = sınırsız. |
| `general.conflict_detection` | `true` | Proaktif çakışma algılama: geçerli dalı ayrıldığı dala merge etmek çakışacaksa Log'da bir rozet; arka planda hesaplanır (bellekte bir merge: dosyalarınızda, indeksinizde ve ref'lerinizde hiçbir şey değişmez). Bir depo bunu kendisi için kapatabilir (`[preferences] conflict_detection`). Karşılaştırdığı temel dalların kendi listeleri vardır: [`[conflict_prevention]`](#conflict_prevention-temel-dal-listeleri). Bkz. [Merge çakışmalarını çözmek](merge-conflicts.md#çakışma-önleme). |
| `general.dev_mode` | yok (otomatik) | GitPoulpi'yi bir checkout'tan derleyenler için; Ayarlar'da hiç görünmez. Yoksa: çalışan exe `<checkout>/target/<profile>/` içindeyken açıktır (`target`'ın yanında `Cargo.toml` ve `crates/gitpoulpi-tui`). `true` / `false` bunu zorlar. Geliştirici modu yalnızca `ui.launcher_build`'in kilidini açar. |
| `general.journal_outside` | `true` | Bir kabuğun veya YZ ajanının dallar, tag'ler, stash'ler ve HEAD üzerinde yaptıklarını, dışarıda yapıldı olarak işaretlenmiş geri alma geçmişi girdileri olarak kaydeder; böylece `Ctrl+Z` onları geri alabilir. Dosya düzenlemeleri hiçbir zaman yakalanmaz. `undo_enabled` gerekir. Bkz. [Geri al ve yinele](undo-redo.md#gitpoulpi-dışında-yapılan-değişiklikler). |
| `general.undo_enabled` | `true` | `false`: hiç geri alma kaydı tutulmaz (en hızlısı), geri alma geçmişi silinir, Geri Al/Yinele düğmeleri kaybolur. Bkz. [Geri al ve yinele](undo-redo.md). |
| `general.active_profile` | `""` | Commit'lerin yapıldığı `[[profiles]]` kimliğinin id'si. Boşsa ya da adı veya e-postası boş bir profilse: Git'in `user.name` / `user.email` değerleri. |
| `general.external_editor` | `""` | Bir dosyanın *Harici düzenleyicide aç* öğesinin komutu, örn. `code -r {file}`. Boşsa: kuruluysa VS Code, yoksa sistem varsayılanı. |
| `general.external_diff_tool` | `""` | *Harici diff aracında aç* komutu: `{local}` (commit'lenmiş / HEAD sürümü), `{remote}` (çalışma ağacı / indeks sürümü). Boşsa: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Bir çakışma kartında (ikili, LFS ya da çok büyük bir dosya) *Harici merge aracında aç* komutu: `{base}`, `{local}` (A, mevcut), `{remote}` (B, gelen) ve `{merged}` (aracın sonucu yazdığı dosya; araç başarıyla çıkarsa GitPoulpi onu stage'ler). Boşsa: `git mergetool --no-prompt`. |
| `general.file_explorer` | işletim sistemine göre | *Klasörde göster* komutu: `{path}` (dosya), `{dir}` (klasörü). Varsayılanlar: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Üç komut şablonu önce argümanlara bölünür (`"..."` bir sözcüğü gruplar), ardından yer
tutucular doldurulur, böylece boşluk içeren yollar tek bir argüman olarak kalır. Hiçbir
kabuk devreye girmez.

```toml
[[profiles]]                # commit kimlikleri (Ayarlar > Profiller)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log ve grafik**

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (terminalin çizebildiği yerde bir resim, başka yerde karakterler), `image`, `text`. **Yeniden başlatma.** |
| `ui.image_protocol` | `"auto"` | Resimlerin (graf, avatarlar) terminale nasıl ulaştığı: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = kitty, Ghostty, iTerm2 3.7.7 veya daha yenisi ve `kitty_graphics = true` olan bir herdr'de Kitty protokolü; WezTerm ve Rio'da (saydamlık harmanlanır), daha eski iTerm2'de ve mintty'de (opak arka plan) iTerm2 protokolü; Windows Terminal, foot, Tabby, Konsole, mlterm'de, arkasındaki terminal sixel kabul ediyorsa tmux'ta ve başlangıç yanıtları sixel'i bildiren her terminalde (`-ti vt340` ile başlatılan bir xterm, görselleri açık bir VS Code terminali) sixel (graf sütunu ve avatarlar); zellij, screen ve bilinmeyen terminallerde karakterler. tmux'ta kitty resimleri için burada `kitty` ve `allow-passthrough on` gerekir ([Çoklayıcılar](multiplexers.md)). Terminalin sözde konsolunun resimleri düşürdüğü veya yanlış yere koyduğu yerlerde (Windows'ta WezTerm 20240203, Rio ve mintty) `ui.graph_renderer = "text"` kullanın ([Terminal seçmek](terminals.md#windows)). **Yeniden başlatma.** |
| `ui.theme` | `"system"` | Renk teması: yerleşik bir slug (`gitpoulpi theme list` hepsini listeler), `<config>/themes/` içindeki bir dosyanın slug'ı ya da `system` (terminalin açık ya da koyu arka planını izler, başlangıçta algılanır; eski `default` değeri aynı anlama gelir). `config set` bilinmeyen adları reddeder. Bkz. [Temalar](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Şerit paleti: `theme` (temayı izle) ya da herhangi bir şerit paleti adı: `gitpoulpi`, `light`, `material`, `serie` ve diğer temaların paletleri. |
| `ui.graph_lane_background` | `"off"` | `off` ya da `row_tint` (grafın arkasında şerit renginin soluk bir bandı, ayrıca kenarında bir çubuk). **Uygulama: Grafik Satır Tonunu Aç/Kapat** (`Ctrl+N Shift+Y`) bunu çevirir; **Uygulama: Grafik Renk Temasını Değiştir...** (`Ctrl+N Shift+T`) `ui.graph_color_theme` değerini seçer. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (yalnızca `row_tint` ile). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` ve `right` bir sütun çizer; 80 sütundan dar bir pencere `bottom`'a döner. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: taşıdığınız ya da yeniden boyutlandırdığınız bir pencerenin (başlık çubuğunu veya kenarlarını sürükleyin, ya da `Ctrl+N m`) yerini koruyup korumayacağı. Tüm pencereler için tek ayar (her pencere bunun altında kendi konumunu ve boyutunu korur): GitPoulpi kapanana kadar (varsayılan), sonraki açılış için diskte (her pencere veri klasöründeki tek `dialog_geometry.txt` dosyasında) ya da hiç (her pencere varsayılan yerinde ve boyutunda açılır). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: düğmelerin araç çubuğu boyunca yeri (satırda sol, orta, sağ; sütunda üst, orta, alt). |
| `ui.toolbar_pr_pinned` | `true` | Pull request düğmesi araç çubuğunun en ucuna sabitlenir; `false` onu diğer düğmelerle birlikte akıtır. |
| `ui.statusbar_position` | `"bottom"` | `top` (depo sekmelerinin üstü) veya `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (pencerenin son satırı, durum çubuğunun altı) veya `left` (sol kenarda bir sütun; 80 sütunun altında üstte kalır). |
| `ui.graph_show_stashes` | `true` | Log'da her stash için bir satır. |
| `ui.ref_connector` | `true` | Bir ref hapından commit'ine ince bir çizgi. |
| `ui.column_resize` | `"line"` | Log'da, Radar'da ve diff, dosya ve blame görünümlerinin commit sütununda bir sütun sınırının nasıl tutulduğu. `line`: her sınır bir hücrelik boşluktur ve başlıkta küçük bir işaret taşır; imleç sınırın üzerindeyken (başlıkta ya da tablonun herhangi bir yerinde) tüm yükseklik boyunca ince bir vurgu çizgisi çizilir ve boyunca herhangi bir yerden sürüklemek yeniden boyutlandırır. `header`: boşluk ve işaret yok; sınırdan sonraki başlık hücresinin ilk karakteri aydınlanır ve oradan sürüklemek yeniden boyutlandırır (yalnızca başlık). Klavye boyutlandırma tuşları ve palet satırları iki kipte de aynıdır. |
| `ui.ref_tree` | `true` | Refs kenar çubuğunda dalları ve tag'leri `/` ile klasörlere gruplar (`y` açıp kapatır). |
| `ui.graph_ghost_ref` | `true` | Log'un Ref sütunu, kendi ref'i olmayan ve üzerine gelinen bir commit için, sütunda yer varsa, ait olduğu dalın (yoksa uzak dalın) adını taşıyan soluk bir hap gösterir; asla bir tag değil. Silinmiş bir dal, onu merge eden merge commit'inin konusundan çıkarılır ve üstü çizili olarak çizilir. Hapın üzerine gelmek türünü gösterir. |
| `ui.reveal_on_ref_click` | `true` | Refs panelinde bir ref'e (dal, tag, uzak dal, stash) tek tıklamak Log'u commit'ine kaydırır ve satırı vurgular; commit seçimi yerinden oynamaz. |
| `ui.log_wip_counts_in_message` | `true` | Değişiklikler sütunu gizliyken WIP mesajında WIP değişiklik sayıları. |
| `ui.log_columns_layout` | `"fit"` | Log'un sütunlarını nasıl yerleştirdiği. `fit`: görünen her sütun bölmeye tam sığar ve yalnızca `message` esner; bölme darsa bütün sütunlar yer açar. `scroll`: sütunlar genişliğini korur (`message` de), hiçbiri çekilmez ve tablo bölmeden genişse yana kayar. |
| `ui.log_column_widths_scroll` | boş | Eski: Log'un sütun genişlikleri artık depoya özeldir, deponun `repo-state.toml` dosyasındadır (`[log_columns]`, `fit` ve `scroll` eşlemeleri, ad -> hücre; `scroll` `message` içerebilir, `fit` asla içermez). Kayıtlı hiçbir şeyi olmayan bir depo, varsa eski `ui.log_column_widths` / `ui.log_column_widths_scroll` değerlerini kullanır, yoksa ilk açılışta içeriğine göre sığdırılır. |
| `ui.log_columns_hidden`, `ui.log_columns_order` | boş | Gösterilen Log sütunları (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`) ve sıraları; bir sütunu gizlediğinizde veya sürüklediğinizde yazılır; tüm depolar için ortaktır. `message` gizlenemez. `ui.log_column_widths` eskidir (yukarıya bakın). |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | boş | Bir diff'in yanındaki [commit sütununun](interface-tour.md#commit-sütunu) bölümleri, Log'unkilerden ayrı olarak: gösterilen bölümler adlarıyla (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; boş, şeritler, mesaj, yazar ve tarih demektir), soldan sağa sıraları aynı adlarla (boş, az önce sayılan sıra demektir; listede olmayan ad o sırayla sona eklenir) ve bölüm başına sürüklenen genişlikler hücre olarak (listede olmayan bölüm otomatiktir); bölümleri seçtiğinizde, taşıdığınızda ya da boyutlandırdığınızda yazılır. `message` gizlenemez. Bu anahtarlardan önce yazılmış bir oturum dosyası bir kez onlara okunur. |
| `ui.sidebar_sections_hidden` | boş | Gizlediğiniz sol panel bölümleri (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`); panelin sağ tık menüsünden ya da paletten yazılır. Var olan bölümlerden en az biri her zaman görünür kalır. Bu varsayılandır: her depo kendi bölümlerini gizleyebilir (deponun durum dosyasında tutulur) ve bu anahtar kendi seçimi olmayan depo için geçerlidir; onu *Tüm depolar için kullan* yazar. |
| `ui.sidebar_sections_scope` | `"repository"` | Sol panel bölümlerinin kimin seçimi olduğu: `repository` (her depo kendisininkini tutar, `ui.sidebar_sections_hidden` varsayılandır) veya `global` (her depo `ui.sidebar_sections_hidden` değerini gösterir ve yapılan değişiklik oraya yazılır). Seçeneği değiştirmek iki kümeden hiçbirini kaybettirmez: `global` açıkken depo başına seçimler durum dosyalarında durur. Ayarlar: *Sol paneldeki bölümler*. |
| `ui.language` | `"auto"` | Arayüz dili: `auto` (sistem yerel ayarı, yoksa İngilizce), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Canlı.** `GITPOULPI_LANG` bunu geçersiz kılar. Bu derlemede bulunmayan bir dil İngilizce gösterir. **Uygulama: Dili Değiştir...** (`Ctrl+N l`) de bunu canlı önizlemeyle ayarlar. CLI ve MCP çıktısı İngilizce kalır. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... her dilde İngilizce kalır. `true`: seçili dilin Git kelime dağarcığı (Fransızca "remiser", "étiquette", "valider"...). **Canlı.** |
| `ui.git_terms_english` | `false` | Yalnızca ja, ko ve zh-CN: transkripsiyon yerine İngilizce Git sözcüklerini (stage, unstage, staged, unstaged...) gösterir. `ui.translate_git_terms` `true` iken etkisizdir. **Canlı.** |
| `ui.mouse_enabled` | `true` | `false` fareyi (ve metin seçimini) terminale bırakır. **Yeniden başlatma.** |
| `ui.mouse_scroll_lines` | `3` | Tekerlek adımı başına Log satırı, 1 ila 20. **Yeniden başlatma.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` terminalden kitty klavye protokolünü ister (böylece `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` ulaşır; yalnızca Unix, tmux asla yanıt vermez: bkz. [Çoklayıcılar](multiplexers.md#tmux)); `off` hiç istemez. **Yeniden başlatma.** |
| `ui.logo_color` | `""` | About penceresindeki logonun renk düzeni, bir slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); boş ya da bilinmeyen, derlemenin varsayılanı (`lavender`) anlamına gelir. Logoya tıkladığınızda ya da About içinde bir renk seçtiğinizde yazılır. |

**Avatarlar** (varsayılan olarak kapalı; açmak yazar e-postalarının SHA-256 özetini
Gravatar'a gönderir ve bağlı GitHub/GitLab hesabınıza sorar)

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (resimli grafın commit dairesinde), `author` (Yazar sütununda), `off`. Eski `ui.show_avatars` (`true`), `ui.avatars` yokken hâlâ `author` olarak okunur. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Arama sırası: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Bir GitLab sunucusuna yalnızca bağlı bir profil üzerinden sorulur. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (yerelde üretilir) ya da `initials`. Resim protokolü olmayan bir terminalde her zaman baş harf rozeti kullanılır. |

**Diff ve düzenleyici görünümü**

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` ya da `split` (bir diff içinde `m`). |
| `ui.diff_hunks_only` | `true` | `false` tüm dosyayı değişiklikler yerinde olarak gösterir (`H`). |
| `ui.diff_show_whitespace` | `true` | Yalnızca boşluk değişikliklerini gösterir (`s`). |
| `ui.diff_word_wrap` | `false` | Uzun diff satırlarını kaydırır (`w`; Ayarlar > Diff ve düzenleyici). |
| `ui.minimap` | `false` | Diff'in ve düzenleyicinin kaydırma çubuğunun yanında bir minimap (`M`). |
| `ui.mermaid_detail` | `refs` | Log'un Mermaid gitGraph olarak kopyala özelliğinin ayrıntı düzeyi: `structure`, `refs`, `subjects`, `full` (pencereden hatırlanır; `log --format mermaid` komutunun varsayılanı). |
| `ui.mermaid_orientation` | `lr` | O grafiğin yönü: `lr` (soldan sağa) veya `tb` (yukarıdan aşağıya). |
| `ui.mermaid_fence` | `false` | Kopyalanan Mermaid betiğini bir `mermaid` kod çiti içine alır. |
| `ui.mermaid_subject_width` | `40` | Commit başlıkları Mermaid kimliklerinde bu kadar karaktere kısaltılır (`subjects`, `full`). |
| `ui.text_cursor` | `"terminal"` | Odaktaki bir metin alanının imlecinin nerede gösterileceği: `terminal` terminalin kendi imlecini oraya koyar (yanıp sönen bir çubuk; böylece terminalin imleç ayarları ve bir giriş yönteminin aday penceresi onu izler; hiçbir alan klavyeyi almadığında gizlenir ve çıkışta kendi imleç biçiminize geri yüklenir), `drawn` GitPoulpi'nin kendi imleç simgesini çizer ve terminal imlecini gizli tutar. Canlı. |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` ya da `16`: kaç renkle çizileceği. `auto`, Apple Terminal (256), Linux konsolu (16) ve `COLORTERM` olmayan `*-256color` terminaller (256) dışında truecolor'dur. Canlı. Bkz. [Temalar](themes.md). |
| `ui.input_color` | `"theme"` | Metin alanı arka planı: `theme` (temanın kendisi), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | `custom` rengi, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | YZ commit composer'ı her önerinin dosyalarıyla `expanded` ya da `collapsed` olarak açılır. |
| `ui.blame_side` | `"right"` | Blame görünümünün hangi tarafında ek açıklama sütununun (avatar, yazar, yaş, özet) olduğu: `right` (varsayılan) ya da `left`. |
| `ui.blame_width` | `46` | Blame ek açıklama sütununun hücre cinsinden genişliği (24 ile 90 arası). Kenarını sürükleyin ya da görünümde `<` / `>` tuşlarına basın; yeni genişlik buraya kaydedilir. |
| `ui.pr_terminology` | `"auto"` | `auto` (GitHub'da Pull Request, GitLab'da Merge Request), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Checkout edilmiş dalın en son pull veya merge request'i kapatıldığında ya da birleştirildiğinde araç çubuğundaki chip'ini gizler; bunun yerine soluk "+ PR" chip'i (yenisini başlat) görünür, sanki dalın hiç isteği yokmuş gibi. `false`: chip, durumu ne olursa olsun her zaman en son isteği gösterir. |

Bir diff içinde değiştirdiğiniz diff düğmeleri buraya hemen kaydedilir ve sonradan açılan
her diff tarafından kullanılır.

**Tarihler**: `ui.date_format` (`"YYYY-MM-DD"`, Blame / Dosya Geçmişi tarih sütunu),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, Log'un Tarih sütunu) ve
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, commit ayrıntıları, Tarih ipucu ve
*Kopyala > Tarih*). Kalıp parçaları: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (hafta günü, tam / kısaltılmış) `HH` `mm` `ss`; başka her şey olduğu gibi
kopyalanır. Ay ve hafta günü adları `ui.language` değerini izler ve bir gün numarasından
sonra yazılan ay, dilin bir tane varsa tamlayan (genitif) biçimidir (Rusça, Ukraynaca,
Lehçe, Çekçe). Bir `D MMMM YYYY` dizisi Japonca, Çince ve Korecede yıl başta yazılır
(`2024年3月5日`). `ui.date_locale` (`"system"`, varsayılan, ya da `"english"`) adları dil
ne olursa olsun İngilizce tutar.

**Ajanlar, terminaller ve sekme klasörü**

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | Pencere ajanlardan gelen `ui show` / `ui propose` isteklerini hiç kabul eder mi (Ayarlar > Ajanlar ve MCP > *YZ ajanlarının görünümü yönetmesine izin ver*). Kapalıyken hiçbir ajan ona ulaşamaz. Bkz. [MCP ve ajanlar](mcp-and-agents.md#kullanıcıya-bir-şeyler-göstermek). |
| `ui.launch` | `"off"` | Hiçbir pencere açık değilken bir ajanın bir GitPoulpi penceresi açıp açamayacağı: `off`, bir hazır ayar (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) ya da `custom`. |
| `ui.launch_command` | `""` | `custom` şablonu: `{repo}`, `{session}`, `{gitpoulpi}`. Argümanlara bölünür, kabuk yok. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (sekme satırı *Çık ve cd yap* düğmesini sunar) ya da `auto` (her çıkış kabuğun klasörünü değiştirir ve yeni bir sekme varsayılan *Şurada aç* girişini çalıştırır). Bkz. [Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Eski, aşağıya ve [Şurada aç](open-in.md) sayfasına bakın. Eski *Burada terminal* işlevinin bir terminali nasıl açtığı: `auto` (`ui.launch` hazır ayarı, yoksa `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), bir hazır ayar ya da `custom`. |
| `ui.terminal_launch_command` | `""` | `custom` şablonu; `{repo}` klasördür. |
| `ui.open_in` | *(yok)* | [Şurada aç](open-in.md) listesi, `label` ve `command` içeren `[[ui.open_in]]` tablolarından oluşan bir dizi. Yok olması "henüz yapılandırılmadı" demektir: liste eski `ui.terminal_launch*` anahtarlarından (`auto` olmayan bir değer ilk giriş olur), işletim sistemi dosya gezgininden, ortamdan algılanan çoklayıcılardan ve terminallerden (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) ve `PATH` üzerinde bulunan düzenleyicilerden hesaplanır. Ayarlar'dan ilk kayıt listeyi yazar. `open_in = []` boş bir listedir. `config set` ile ayarlanamaz (dosyayı düzenleyin). |
| `ui.open_in_default` | `""` | "Varsayılanda aç"ın çalıştırdığı girişin etiketi; boş ya da bilinmeyen, ilk giriş anlamına gelir. |
| `ui.launch_terminal` | `"auto"` | Masaüstü kısayollarının (Ayarlar > Başlatıcı) ve Windows'taki yeniden başlatmanın GitPoulpi'yi açtığı terminal. `auto` kurulu olanlardan seçer (Windows: Windows Terminal, yoksa WezTerm, Alacritty, düz konsol; Linux: `xdg-terminal-exec`, yoksa bulunan ilk terminal; macOS: kuruluysa iTerm2, Ghostty, WezTerm veya kitty, yoksa Terminal), `default` işletim sisteminin kendi terminalidir; ya da `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal` gibi bir terminal kimliği. |
| `ui.launcher_build` | `"current"` | Yalnızca geliştirici modunda (bkz. `general.dev_mode`; yalnızca o zaman Ayarlar > Başlatıcı'da görünür, aksi halde yok sayılır). `"current"`: kısayollar ve komut takma adı onları oluşturan derlemeyi başlatır. `"newest"`: checkout'un `target/dist` ve `target/release` derlemelerinden en son derleneni başlatırlar, her tıklamada karar verilir ve iki derlemeden birini başlatmak onları asla yeniden yazmaz. |
| `ui.command_alias` | `""` | GitPoulpi'yi bir terminalden başlatan komutun adı, örneğin `gp` (Ayarlar > Başlatıcı, `gitpoulpi alias create`). Boş: yok. Yalnızca harf, rakam, `-`, `_` ve `.`. Bu yalnızca **Oluştur: komut takma adı** düğmesinin kullandığı addır; siz oluşturana kadar hiçbir şey var olmaz. |
| `ui.command_alias_add` | `""` | `gitpoulpi --add` çalıştıran ikinci komutun adı, örneğin `gpa`: geçerli klasörü zaten çalışan GitPoulpi'de yeni sekme olarak açar (Ayarlar > Başlatıcı, `gitpoulpi alias create gpa --add`). Boş: yok. Ad kuralları aynıdır; **Oluştur** düğmesi onu `ui.command_alias` ile birlikte oluşturur. |
| `ui.relaunch_in_terminal` | `true` | Yalnızca Windows: exe çift tıklamayla kendi konsolunda başlatıldığında GitPoulpi kendini `ui.launch_terminal` içinde yeniden açar ve çıkar. Exe'yi doğrudan çalıştıran bir terminal (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio, Tabby, Alacritty `-e`) de kendi konsoludur ama klasik bir konsol penceresi değildir: asla yeniden başlatılmaz. Kapalı: o konsolda kalır. |
| `ui.reuse_window` | `true` | Bir GitPoulpi kısayoluna tıklamak (`gitpoulpi launch`), yenisini başlatmak yerine zaten açık olan GitPoulpi'nin penceresini gösterir. Kapalı: her tıklama yeni bir pencere başlatır. |

**`ui.open_in` komutları** önce argümanlara bölünür (boşluk, `'...'`, `"..."`), ardından
değişkenler her argümanın içinde yerine konur, böylece boşluk içeren bir klasör tek bir
argüman olarak kalır; komut bir kabuk başlatmadıkça kabuk devreye girmez (`sh -c '... "$GITPOULPI_CWD"'`). Değişkenler: `${cwd}` (sekmenin klasörü), `${repo}` (deponun kökü), `${name}` (klasörün temel adı), `${branch}` (detached iken boş), `${web_url}` (`origin` uzak deposunun bir GitHub ya da GitLab projesi olduğu durumdaki web sayfası, yoksa boş; onu kullanan bir giriş boşken çalışmayı reddeder), `${env:NAME}` (ayarlı değilse boş). Eski `{repo}` hâlâ `${cwd}` anlamına gelir. Başka herhangi bir `${...}` onu adlandıran bir hatadır. Örnek:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Eski: `ui.terminal_launch` ve `ui.terminal_launch_command` hâlâ okunur (`ui.open_in` yokken listeyi tohumlarlar: `custom`, `{repo}` değeri `${cwd}` olarak yeniden yazılmış "Terminal (custom)" olur, bir hazır ayar kimliği o hazır ayar olur) ancak yerlerini `ui.open_in` almıştır.

### Görseller

| Anahtar | Varsayılan | Anlam |
|---|---|---|
| `ui.image_view` | `true` | Görsel olan (PNG, JPEG, GIF, WebP, BMP, ...) bir ikili dosyayı Diff Görünümü'nde ve Dosya Görünümü'nde resim olarak gösterir. Kapalı: düz ikili dosya bildirimi. |
| `ui.image_view_max_mib` | `32` | Bundan büyük görseller (MiB) çözülmez; bunun yerine bir bilgi kartı görünür. |
| `ui.image_view_max_unit` | `"auto"` | Ayarlar'ın sınırı hangi birimle gösterdiği: `mib`, `gib` (1024 MiB) veya `auto`. Yalnızca görüntüleme içindir: sınır her zaman `image_view_max_mib` MiB'dir, 1 ile 1024 arası. |
| `ui.image_view_text_fallback` | `"halfblock"` | Görsel protokolü olmayan bir terminalde: `halfblock` (renkli yarım blok karakterler) ya da `card` (yalnızca bilgi kartı). |
| `ui.image_view_pixelated_from` | `"2x"` | Büyütmenin yumuşatma yerine kare pikseller gösterdiği yakınlaştırma: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Önce ve sonra panelleri tek bir yakınlaştırmayı ve kaydırma konumunu paylaşır. |
| `ui.image_editors` | *(yok)* | "Şununla düzenle" görsel düzenleyicileri, `label` ve `command` içeren `[[ui.image_editors]]` tablolarından oluşan bir dizi. Yok olması "henüz yapılandırılmadı" demektir: liste bu makinede algılanan düzenleyicilerdir (`PATH` üzerindeki programlar, Flatpak kurulumları, Windows'ta bilinen kurulum klasörleri, macOS'ta `/Applications`), ardından işletim sisteminin varsayılan uygulaması. Ayarlar'dan ilk kayıt listeyi yazar. `image_editors = []` boş bir listedir. `config set` ile ayarlanamaz (dosyayı düzenleyin). |
| `ui.image_editor_default` | `""` | "Varsayılan olarak Şununla düzenle"nin çalıştırdığı düzenleyicinin etiketi; boş ya da bilinmeyen, ilk giriş anlamına gelir. |

**`ui.image_editors` komutları** önce argümanlara bölünür (boşluk, `'...'`, `"..."`), ardından değişkenler her argümanın içinde yerine konur, böylece boşluk içeren bir yol tek bir argüman olarak kalır; hiçbir kabuk devreye girmez. Değişkenler: `${file}` (görselin mutlak yolu), `${dir}` (klasörü), `${name}` (dosya adı), `${repo}` (deponun kökü, geçici bir kopya için boş), `${env:NAME}` (ayarlı değilse boş). Başka herhangi bir `${...}` onu adlandıran bir hatadır. Çalışma ağacı dosyası olmayan bir görsel (bir commit'in sürümü, indeks, silinmiş bir dosya) **salt okunur geçici bir kopya** olarak açılır, böylece bir düzenleyici depoya dokunamaz. Komut satırı olmayan düzenleyiciler (Microsoft Store'daki Affinity Photo, bazı Photoshop kurulumları) hazır ayarlarda yoktur: özel bir giriş yazın. Örnek:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Henüz kullanılmıyor** (kabul edilir, kaydedilir, ancak hiçbir şey onları okumaz):
`ui.graph_colorization` (de Ayarlar'dan kaldırıldı;
yüklemede hâlâ kabul edilir), `ui.date_word_format` (Ayarlar'dan kaldırıldı; yüklemede
hâlâ kabul edilir). Eski `ui.diff_view_mode`
bir kez `diff_layout` / `diff_hunks_only` olarak okunur.

## `[editor]`

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `editor.tab_size` | `4` | Sekme durağı başına sütun, 1 ila 8 (düzenleyici ve diff'ler). |
| `editor.eol` | `"auto"` | Yerleşik düzenleyicinin yazdığı satır sonu: `auto` (dosyanınkini koru), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Düzenleyicide dil renkleri. |
| `editor.line_numbers` | `true` | Düzenleyicide ve diff'lerde satır numaraları. |
| `editor.follow_selection` | `true` | Düzenleyici açıkken sağ panelde başka bir dosya seçmek onu düzenleyicide de açar (önce kaydetmeyi sorar); bir commit'in dosyası düzenleyiciden çıkıp kendi diff'ini gösterir. |

## `[merge]`

Merge çakışma düzenleyicisi ([Merge çakışmalarını çözmek](merge-conflicts.md)). Ayarlar > Diff ve düzenleyici bunu gösterir.

| Anahtar | Varsayılan | Anlam |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | A ve B'nin yan yana gösterildiği, düzenleyici alanının sütun cinsinden genişliği (uzun satırlar yana kayar: Shift+tekerlek, Shift+oklar). Daha darsa, sekmelerin ardında her seferinde bir tarafı gösterir. `40` en düşük değerdir, `0` her zaman yan yana demektir (40 sütunun altındaki bildirim kalır). Düzenleyici açıkken Refs ve Durum panellerini gizler, bu yüzden alanı genellikle tüm penceredir. |

## `[git]`, `[credentials]`, `[ssh]`

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `git.pull_mode` | `"merge"` | Pull'un (düğme, `Ctrl+G p`, `Git: Pull`) yaptığı: `merge` (mümkünse fast-forward), `ff_only`, `rebase`. Git'e her zaman açıkça iletilir. Eski `fetch_all`, `merge` olarak okunur. |
| `git.push_behavior` | `"direct"` | `direct` varsayılan uzak depoya push eder; `ask` önce uzak depo menüsünü gösterir. |
| `git.push_group_preflight` | `true` | Bir dalı birden çok uzak depoya push etmeden önce her hedefi önce denetler (`ls-remote`, artı o uzak depodan yalnızca farklıysa bir fetch); böylece sonraki bir uzak depodaki sorun ilk push'tan önce bulunur. |
| `git.merge_fast_forward` | `"never"` | Başlattığınız bir merge'ün (dal menüsü, sürükle bırak, palet, `gitpoulpi merge`) fast-forward yapılabilecek bir dalda yaptığı: `never` her zaman bir merge commit'i oluşturur (`--no-ff`), `allowed` mümkünse fast-forward yapar (git'in varsayılanı, `merge.ff` geçerlidir), `only` fast-forward yapar ya da reddeder. Deponun kendi seçimi (Ayarlar > Merge seçenekleri, `repo-state.toml` içindeki `merge_fast_forward`) kazanır; CLI'nin `--ff`, `--no-ff` ve `--ff-only` bayrakları ikisini de geçersiz kılar. Anında uygulanır. |
| `git.default_encoding` | `"utf-8"` | Geçerli UTF-8 **olmayan** metin için kodlama (dosya içerikleri, blame, diff'ler, `encoding` başlığı olmayan commit mesajları): herhangi bir WHATWG etiketi (`windows-1252`, `shift_jis`, ...). Yalnızca gösterim; staging asla yeniden kodlamaz. Bir depo bunu geçersiz kılabilir. |
| `git.lfs_enabled` | `true` | Git LFS'yi gösterir ve yönetir. Bir depo bunu geçersiz kılabilir. Bkz. [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | LFS kullanan bir depoyu pull ya da push etmeden önce, eksikse LFS filtrelerini / pre-push hook'unu kurar (`git lfs install --local`, yalnızca bu depo). Bkz. [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Kullanılmıyor*: bir deponun `core.autocrlf` / `core.eol` değerleri git'in kendi ayarlarıdır, Ayarlar > Git ve Kodlama'da (ya da `gitpoulpi repo-config` ile) düzenlenir. |
| `credentials.https_mode` | `"credential_helper"` | Fetch, pull, push, clone için HTTPS kimlik doğrulaması: `credential_helper` (git'in), `prompt` (yardımcıları yok sayar; git bir terminalde sorar, bu yüzden arka plan daemon'unda başarısız olur), `none` (yalnızca herkese açık uzak depolar). Eski `encrypted_file` değerleri `credential_helper` olarak okunur. |
| `ssh.key_type` | `"ed25519"` | *Anahtar çifti oluştur* ile üretilen tür: `ed25519`, `rsa` (4096 bit), `ecdsa` (NIST P-256); dosya `gitpoulpi_<type>` olur. SSH'ın kendisi her zaman ssh-agent'ınızı ve ssh yapılandırmanızı kullanır. |
| `ssh.private_key_path`, `ssh.public_key_path` | ayarsız | *Anahtar çifti oluştur* tarafından kaydedilir. `ssh.private_key_path` ayarlıysa fetch, pull, push ve clone git'i `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes` ile çalıştırır; özel yol boşken `ssh.public_key_path` aynısını yapar (ssh eşleşen ajan anahtarını seçer). Ortamdaki bir `GIT_SSH_COMMAND`/`GIT_SSH` ya da git yapılandırmasındaki bir `core.sshCommand` (depo başına sabitleme, `gitpoulpi ssh-command set --key`) kazanır. Her aktarımdan önce okunur, bu yüzden değişiklikler bir sonrakine uygulanır. |

## `[[integrations]]` (barındırma ve issue takipçileri)

Ayarlar > Entegrasyonlar ya da `gitpoulpi hosting integrations add` tarafından yazılır.
Her türün bir istemcisi vardır (yalnızca kişisel token'lar, OAuth yok).

```toml
[[integrations]]
id = "corp"                         # --id'nin aldığı değer; token integration:corp altında saklanır
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # API kökü bundan türetilir
token_ref = "integration:corp"      # bir başvuru, asla token'ın kendisi değil
name = "Corp GitLab"                # isteğe bağlı görünen ad
account = "me@example.com"          # isteğe bağlı, aşağıya bakın
enabled = true                      # false: sunucu eşleştirmede yok sayılır
```

Kendi barındırdığınız bir sunucu düz http olabilir: `url = "http://localhost:3000"`
yazıldığı gibi korunur (şema ve port her isteğe ulaşır ve o sunucudaki bir uzak depo
profille eşleşir). Düz http üzerinden gönderilen token şifrelenmeden iletilir, bu
yüzden http'yi yalnızca yerel veya güvenilir bir ağda kullanın.

`kind` ayrıca `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` ve `gerrit` değerlerini
kabul eder (Gitea ve Forgejo tek bir istemci paylaşır; bir Forgejo profilinin varsayılanı `https://codeberg.org` olur). Jira,
Trello, Linear, YouTrack ve Redmine yalnızca issue takipçileridir; Gerrit yalnızca incelemedir (değişiklikleri pull
request'lerdir, issue'su yoktur). Bitbucket Cloud'un issue takipçisi yoktur.

`account`, token'ın yanındaki gizli olmayan kimliktir; çoğu tür için boştur ve
yazılmaz: `jira_cloud` ve `bitbucket` için Atlassian hesabı e-postası (boş bir Bitbucket
hesabı, Bearer olarak gönderilen bir çalışma alanı ya da depo erişim token'ı anlamına
gelir), `trello` için API anahtarı, `gerrit` için kullanıcı adı (token onun HTTP parolasıdır, HTTP basic olarak
gönderilir). Token'ın kendisi tek gizli bilgi olarak kalır.
`azure_devops` için `url`, `https://dev.azure.com/<organization>` biçimindedir;
`https://<organization>.visualstudio.com` biçimindeki bir URL aynı kuruluş olarak okunur.
`azure_devops_server` için bu, `https://tfs.example.com/tfs/DefaultCollection` gibi
koleksiyon köküdür.

`user` (bağlı hesap) ve `ssh` (profilin anahtar seçimi) sizin için yazılır. Token,
`GITPOULPI_TOKEN_<ID>` değişkeninden (id büyük harfle, alfasayısal olmayanlar `_` olarak),
yoksa saklanan token'dan, yoksa yalnızca tek bir profil o sunucuyu kullanıyorsa o sunucu
için kimlik bilgisi yardımcınızdan gelir (yalnızca GitHub ve GitLab profilleri: diğer
türler kimlik bilgisi yardımcısını asla okumaz). Ayrıntılar: [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md). Eski `[[hosting]]` tablosu *kullanılmıyor*.

`local_markdown` için (issue kaynağı olarak bir Markdown notları klasörü) `url`, her depoya göre veya mutlak olan
klasördür ve token yoktur. İsteğe bağlı `[integrations.local]` tablosu `statuses` (panonun sütun sırası,
varsayılan `todo`, `doing`, `done`), `done_status` (varsayılan `done`), `reference_prefix` (`LOC`, `LOC-12`
gösterir) ve `me` ("bana atanan" ifadesinin kim olduğu) ayarlar. Bkz. [Yerel issue'lar ve Obsidian](local-issues.md).

### `[integrations.radar]` (hesap geneli listeler)

Bir kod barındırıcısı profilinin isteğe bağlı tablosu (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; Bitbucket Cloud değil). Yokluğu,
her anahtarın kapalı olduğu anlamına gelir. `gitpoulpi hosting integrations radar` tarafından yazılır;
`gitpoulpi hosting radar` ve Radar tarafından okunur ve Ayarlar > Entegrasyonlar'da düzenlenir (bağlı bir profilin Radar bloğu).

```toml
[integrations.radar]
review_requests = true    # incelememi bekleyen request'ler, her depo
open_requests = false     # ad alanlarımın her açık request'i
include_drafts = false    # bu listelerde taslak request'leri tut
assigned_issues = true    # bana atanan açık issue'lar, her depo
namespaces = ["acme", "ptm/produits"]  # Ayarlar'da satır başına bir giriş; boş = sınır yok
namespaces_regex = false  # true: her giriş, tam depo yolu üzerinde bir düzenli ifadedir
review_mode = "ask"       # ask | clone | api
```

| Anahtar | Varsayılan | Anlamı |
|---|---|---|
| `review_requests` | `false` | İncelemenizi isteyen açık request'ler; depo klonlanmış olsun olmasın |
| `open_requests` | `false` | Kendi ad alanınızın ve üye olduğunuz her org veya grubun (ya da `namespaces` değerinin) her açık request'i; asla "token'ın gördüğü her şey" değil |
| `include_drafts` | `false` | Bu açılmadıkça taslak request'ler bu listelerin dışında bırakılır |
| `assigned_issues` | `false` | Size atanan açık issue'lar, her depo |
| `namespaces` | `[]` | Her listenin hangi depoları tuttuğu (incelenecek request'ler, atanan issue'lar, açık request'ler); boş, sınır yok demektir ve `open_requests` o zaman kendi ad alanınızı artı her üyeliğinizi arar (GitHub: en fazla 12 aranır, GitLab: 10 grup ve 10 sahip olunan proje, Gitea, Forgejo ve Gerrit: 10). Bir giriş, deponun tam yoluyla (`group/sub/project`, Azure DevOps `Project/repo`, Bitbucket Data Center `KEY/repo`) eşitse veya `/` ile biten bir yol öneğiyse eşleşir: `ptm`, `ptm/produits/x`'i tutar, `ptmx/y`'yi değil; büyük/küçük harf yok sayılır. Ayarlar satır başına bir giriş gösterir; eski bir dosya virgülle ayrılmış tek bir dize (`"ptm, formation"`) tutabilir, o yine yüklenir ve liste olarak geri kaydedilir |
| `namespaces_regex` | `false` | Her `namespaces` girdisini, tam depo yoluna yazdığınız haliyle uygulanan bir düzenli ifade olarak okur (yani `^ptm/(produits\|infra)/` başa sabitlenir; `^` veya `$` olmadan yolun herhangi bir yerinde eşleşir; büyük/küçük harfi yok saymak için `(?i)` ekleyin). `open_requests` o zaman üyeliklerinizi arar ve ifadeler satırları süzer. Geçersiz bir ifade Ayarlar'da alanın altında gösterilir ve kaydedilmez; elle düzenlenen bir dosyada listeleri bir notla boş bırakır |
| `review_mode` | `"ask"` | Deposu klonlanmamış bir request'i incelemenin ne yapacağı: `ask`, `clone` (klonla, sonra incele) veya `api` (barındırıcının API'si üzerinden incele) |

Yanıtlar profil başına, `config.toml`'un yanındaki `radar-account.json` içinde beş dakika önbelleğe alınır;
dosya bir önbellektir, silinmesi güvenlidir.

### Güven deposu (`config.toml` içinde değil)

Her entegrasyonun ve API anahtarlı YZ profilinin ne için güvenildiği (adresi, hesap kimliği,
sabitlenmiş bir sertifika anahtarı, düz http'ye izin verilip verilmediği) yalnızca kullanıcıya ait
özel veri klasöründeki `trust.toml` dosyasında durur (`<data>/private/trust.toml`: Windows'ta
`%APPDATA%\GitPoulpi\private\`, Linux'ta `~/.local/share/gitpoulpi/private/`). Bilerek
`config.toml` dosyasının parçası değildir: yapılandırmayı düzenleyebilen kişi, onu düzenleyerek
güveni genişletememelidir. Onu `gitpoulpi hosting integrations trust` ve `gitpoulpi ai trust` ile
değiştirin (bkz. [güvenlik denetimi](hosting-integration.md#güvenlik-denetimi)); dosyayı elle değil,
GitPoulpi yeniden yazar.

## `[mcp]` (YZ ajanlarının yapabilecekleri)

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `mcp.allow_mutations` | `false` | Kapalı: bir şeyi değiştiren MCP araçları reddedilir; okumalar, `ui_*` ve öneriler çalışmaya devam eder. `true` ajanların depoyu değiştirmesine izin verir. |
| `mcp.mutation_allowlist` | `[]` | Boş değilse: yalnızca bu araç adları bir şeyi değiştirebilir (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: pencerenizi hareket ettiren `ui_*` araçlarını reddeder (`ui_state` / `ui_viewers` hariç). |
| `mcp.ui_control` | `"ask"` | `ask` (pencere onayladığınız bir çip gösterir), `auto` (meşgul olmadığınız sürece uygulanır), `off`. Sunucu komutundaki `--ui-control` bunu ajan başına geçersiz kılar. `gitpoulpi ui show` için de geçerlidir. |
| `mcp.allow_ui_propose` | `false` | `ui_propose_*` / `gitpoulpi ui propose` izni: kendinizin onayladığı, önceden doldurulmuş iletişim kutuları. |
| `mcp.allow_history_purge` | `false` | Yalnızca `config.toml` içinde (Ayarlar'da satırı yok). `true`, `rewrite_purge` aracını ajanlara listeler (yine de `allow_mutations` gerekir). Kapalıyken bir ajan aracı hiç görmez: temizlik, bir geçmiş yeniden yazmanın eski geçmişini, onun geri almasını ve reflog'larını kalıcı olarak siler. CLI ve TUI bundan etkilenmez. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` ve `gitpoulpi-mcp` bu anahtarı adlandıran bir mesajla başlamayı reddeder. Ayarlar > Ajanlar ve MCP > *MCP sunucusu*. Bir ajan sunucuyu başlattığında okunur. |
| `mcp.bind` | `"127.0.0.1:8808"` | Hiçbiri verilmediğinde `--http` ile dinlenen adres (`--http ADDR` / `--bind` kazanır). `--allow-remote` olmadıkça yalnızca loopback. Boş ya da eski `"stdio"` varsayılan anlamına gelir. Ayarlar > Ajanlar ve MCP > *MCP sunucu portu* portu düzenler ve ana makine kısmını korur. |
| `mcp.http_auth` | `true` | `--http` yalnızca `Authorization: Bearer <token>` taşıyan bir isteğe yanıt verir (`gitpoulpi mcp token` token'ı). `false` denetimi kaldırır; sunucu bu durumda stderr'e uyarı yazar ve `--allow-remote` reddedilir. Ayarlar > Ajanlar ve MCP > *HTTP kimlik doğrulaması*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (özel veri klasörünün altında bir kez üretilen bir sertifika, başlangıçta SHA-256 parmak izi yazdırılır) veya `files` (`tls_cert` ve `tls_key`). TLS açıkken sunucu TLS'yi kendisi sonlandırır ve düz bir dinleyici yoktur. Ayarlar > Ajanlar ve MCP > *TLS*. |
| `mcp.tls_cert` | `""` | `tls = "files"` ile kullanılan PEM sertifika zinciri. |
| `mcp.tls_key` | `""` | `tls = "files"` ile kullanılan PEM özel anahtar. |
| `mcp.behind_proxy` | `false` | Uç noktanın önündeki bir tünelin veya ters vekil sunucunun şifrelemeyi taşıdığını beyan edersiniz. Bu, sunucunun içinden denetlenemez; bu yüzden TLS olmadan `--allow-remote` için ayarlı olması gerekir. Vekil ile sunucu arasındaki bağlantı o ağda düz metindir. Ayarlar > Ajanlar ve MCP > *Tünel veya ters vekil sunucunun arkasında*. |
| `mcp.allow_sensitive_staging` | `false` | Yalnızca `config.toml` içinde. Kapalıyken MCP stage araçları `allow_sensitive` sunmaz ve işaretli bir dosya ekleyecek bir stage (bkz. `[privacy]`) ajana size sormasını söyleyen bir iletiyle reddedilir. Açıkken bir ajan `allow_sensitive` geçebilir. CLI ve TUI buna bağlı değildir: onlar size sorar. |

`allow_mutations`, `mutation_allowlist` ve `allow_ui` yalnızca MCP sunucusunu kapılar;
CLI sizin kendi kabuğunuzdur ve bunlarla kapılanmaz. Bkz. [MCP ve
ajanlar](mcp-and-agents.md#i̇zinler).

## `[ai]` (yerleşik YZ, varsayılan olarak kapalı)

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `ai.enabled` | `false` | Anahtar. Kapalıyken hiçbir yerde YZ ile ilgili bir şey görünmez ve hiçbir şey gönderilmez. |
| `ai.default_profile` | `"default"` | Eylemler hiçbirini belirtmediğinde kullandıkları profilin id'si. |
| `ai.global_instructions` | `""` | Her eylemin istemine eklenen metin. `${repo}`, `${uilang}`, ... kullanabilir ([YZ > İstem değişkenleri](ai.md#i̇stem-değişkenleri) bölümüne bakın); eylem başına ve depo talimatları için de aynıdır. |
| `ai.actions.<action>.profile` | `""` | Tek bir eylem için profil; boş = varsayılan. Eylemler: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`. Bir çalışma alanı (`workspaces.toml`, `[workspace.ai_instructions]` ve `[workspace.ai_overrides.<action>]`) ve bir depo bu talimatlara ekleme yapabilir veya onları değiştirebilir: bkz. [YZ](ai.md#talimatlar). |
| `ai.actions.<action>.instructions` | `""` | Yalnızca o eylem için eklenen metin. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` veya `no`: bir kod inceleme taslağının yerel ajanının o çalıştırma için depo içinde çalışıp çalışmayacağı; YZ profilinin `workdir` değerini geçersiz kılar. Bir depo kendi değerini ayarlayabilir (`gitpoulpi ai review-context --repo`). Bkz. [YZ > İnceleme bağlamı](ai.md#i̇nceleme-bağlamı). |
| `ai.review.include_claude_md` | `true` | Bir kod inceleme taslağına deponun `CLAUDE.md` dosyasını ver (kök, isteğin hedeflediği dalda okunur; 8 KB'ta kesilir). |
| `ai.review.include_agents_md` | `true` | `AGENTS.md` için aynısı. |
| `ai.review.extra_file` | `""` | Bir kod inceleme taslağı için bir dosya daha: depo köküne göre bir yol, mutlak bir yol veya bir kod barındırıcısındaki dosyaya bağlantı (eşleşen entegrasyon profili üzerinden, yoksa anonim okunur). Tüm dosyalar birlikte 16 KB'ta kesilir. |
| `ai.privacy.redact_secrets` | `true` | Tarayıcının tanıdığı her gizli bilgiyi (bir token, bir özel anahtar bloğu, bir URL içindeki parola, rastgele görünen bir `password = ...` değeri) bir istek çıkmadan önce `[REDACTED:<rule>]` ile değiştirir. Kapalıyken Ayarlar > YZ kırmızı bir uyarı gösterir. |
| `ai.privacy.exclude_sensitive_files` | `true` | `privacy.sensitive_patterns` ile eşleşen her dosyanın diff'ini gönderilenin dışında bırakır. Depo klasöründe çalışan bir yerel ajan yine de dosyaları kendisi okuyabilir: bkz. [YZ > Gizlilik](ai.md#gizlilik). |
| `ai.limits.conflict_bytes` | `40000` | "Çözüm öner" bir merge çakışmasını gönderirken çakışmanın (temel, geçerli ve gelen blokları birlikte) en çok kaç bayt olabileceği. Aşılırsa GitPoulpi hata vermez: uyarır ve *Yine de gönder* (yalnızca bu istek), *Ayarı aç* ve *İptal* sunar; bir çakışma asla kesilmez. Sınır modelin değil, GitPoulpi'nin kendi korumasıdır: modeliniz yer açıyorsa artırın. 1000 ile 50000000 arasında tutulur; Ayarlar > YZ > Sınırlar'dan ayarlanabilir. Aşağıdaki öteki sınırlar da aynı şekilde çalışır ve hepsi o gruptadır. |
| `ai.limits.diff_bytes` | `60000` | Bir commit mesajı, bir açıklama, bir stash adı, bir pull request açıklaması, bir commit planı veya bir commit bölme isteğinin taşıyabileceği diff'in ve bir kod incelemesinin tüm diff'inin en çok bayt sayısı. Aşılırsa hiçbir şey sessizce kesilmez: TUI'de *YZ sınırı aşıldı* *Tümünü gönder* (yalnızca bu istek, hiçbir şey kesilmez), *Kısaltarak gönder* (sonu dışarıda kalır, eskiden olduğu gibi), *Ayarı aç* ve *İptal* sunar; CLI ve MCP kısaltır ve bunu `truncated` içinde bildirir (`--send-all` her şeyi gönderir). 1000 ile 50000000 arasında tutulur. |
| `ai.limits.review_file_bytes` | `12000` | Bir kod incelemesinde tek bir dosyanın diff'inin en çok bayt sayısı. Aşılırsa aynı soru ve aynı iki yanıt; *Kısaltarak gönder* o dosyayı görünür bir notla kısaltır. Aynı aralık. |
| `ai.limits.review_text_bytes` | `4000` | İncelenen pull request'in açıklamasının en çok bayt sayısı. Aşılırsa *Kısaltarak gönder* açıklamayı bir `[text cut]` notuyla keser. Aynı aralık. |
| `ai.limits.review_threads_bytes` | `6000` | Zaten var olan inceleme yorumları listesinin (model onları tekrarlamasın diye verilir) en çok bayt sayısı. Aşılırsa *Kısaltarak gönder* listeyi keser. Aynı aralık. |
| `ai.limits.review_context_file_bytes` | `8192` | Bir kod incelemesine verilen tek bir proje bağlam dosyasının (`CLAUDE.md`, `AGENTS.md`, ek dosya) en çok bayt sayısı. Aşılırsa *Kısaltarak gönder* dosyayı bir notla kısaltır. Aynı aralık. |
| `ai.limits.review_context_bytes` | `16384` | Tüm bağlam dosyalarının birlikte en çok bayt sayısı. Aşılırsa *Kısaltarak gönder* sınırı geçen dosyayı kısaltır ve ondan sonrakileri dışarıda bırakır. Aynı aralık. |
| `ai.limits.hosting_reply_context_bytes` | `49152` | Taslağı hazırlanan bir yanıtın (veya issue açıklamasının) kendi metniniz dışında aldığı her şeyin en çok bayt sayısı: öğe ve konuşması. Aşılırsa önce en eski yorumlar dışarıda bırakılır ve kesinti `truncated` içinde bildirilir. Diğer bayt sınırlarıyla aynı aralık. |
| `ai.limits.hosting_reply_file_bytes` | `8192` | Taslağı hazırlanan bir yanıta verilen tek bir depo dosyasının en çok bayt sayısı. YZ depoyu okurken kullanılır. Aynı aralık. |
| `ai.limits.hosting_reply_map_bytes` | `6144` | Taslağı hazırlanan bir yanıta verilen depo haritasının en çok bayt sayısı. YZ depoyu okurken kullanılır. Aynı aralık. |
| `ai.limits.hosting_reply_hits_per_term` | `20` | Taslağı hazırlanan bir yanıtın arama terimi başına tuttuğu en çok arama sonucu. YZ depoyu okurken kullanılır. 1 ile 200 arasında tutulur. |
| `ai.limits.hosting_reply_terms` | `8` | Taslağı hazırlanan bir yanıtın aradığı en çok arama terimi. YZ depoyu okurken kullanılır. 1 ile 50 arasında tutulur. |
| `ai.hosting_reply.base_prompt` | `""` | İki yazma eyleminin (`hosting_reply`, `issue_body`) temel istemi. Boşsa yerleşik istem kullanılır; buraya yazılan metin onun yerini alır, talimat düzeyleri yine ardından gelir. Ayarlar > YZ > Sunucudaki yanıtlar'da düzenlenir. |
| `ai.hosting_reply.read_repository` | `false` | "YZ depoyu okuyabilsin" seçeneğinin işaretli başlayıp başlamayacağı. Varsayılan olarak kapalı: depo yalnızca o kullanım için siz işaretlediğinizde okunur. Bir yerel ajan veya bir API profili onu o zaman [YZ bölümünde](ai.md#depo-erişimi) anlatıldığı gibi okur. |
| `ai.hosting_reply.reuse_agent_session` | `true` | Harness'i destekliyorsa, bir yerel ajanın oturumunu iki yanıt arasında korur. YZ depoyu okuyabildiğinde etkili olur. |
| `ai.hosting_reply.context_ttl_minutes` | `30` | Bir yanıt için depodan oluşturulan bağlamın ne kadar süre yeniden kullanılacağı. 1 ile 1440 arasında tutulur. YZ depoyu okuyabildiğinde etkili olur. |
| `ai.sent_preview.<action>` | `true` | Eylem başına bir anahtar (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`, `hosting_reply`, `issue_body`): istek çıkmadan önce "Gönderilenler"i gösterir. İletişim kutusundaki "<eylem> için bir daha gösterme" `false` yazar; Ayarlar > YZ > *Göndermeden önce gönderileni göster...* onları yeniden açar. `rewrite_script` ve `rewrite_rules` kabul edilir ama etkisi yoktur: geçmişi yeniden yazma pencereleri her zaman kendi bölmesini gösterir. `worktree_name` de kabul edilir ama etkisi yoktur: yalnızca Worktree oluştur penceresine yazdığınızı ve yerel dal adlarını gönderir. |

Bir deponun kendi `[preferences.ai_review]` tablosu (`repo-state.toml` içinde, ya da klonlanmamış bir deponun girdisi
`remote-reviews.toml` içinde) aynı anahtarları alır, hepsi isteğe bağlıdır, ayrıca `no_extra_file = true`: o depo için
`ai.review.extra_file` ne derse desin ek dosya yok. Ayarlanmamış bir anahtar, ayarlanmamış bir `extra_file` ve
eksik bir `no_extra_file` genel değeri devralır; `extra_file`'ı bir dosyaya ayarlamak anahtarı temizler, tersi de öyle.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # boş = türün varsayılanı
base_url = ""           # boş = türün varsayılanı (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # API anahtarının saklandığı yer; anahtarın kendisi asla
command = ""            # yalnızca local_agent: tam program ("claude", bir takma ad, bir sarmalayıcı, bir yol)
harness = ""            # yalnızca local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; boş = `command` değerinden çıkarılır
args = []               # yalnızca local_agent, örn. ["-p", "--model", "haiku"]
run_mode = "headless"   # yalnızca local_agent: headless (varsayılan) | terminal (ajanın etkileşimli ekranı bir sanal terminalde)
terminal_args = []      # yalnızca local_agent: terminal modunun argümanları (args başsız kümeyi korur)
workdir = "default"     # yalnızca local_agent: default | dedicated | repository (default = terminal modunda özel klasör, başsız modda GitPoulpi'nin klasörü)
timeout_secs = 0        # yalnızca local_agent: ajan durdurulmadan önceki saniye; 0 = başsızda 120, terminalde 180

[ai.profiles.env]       # yalnızca local_agent, isteğe bağlı: yalnızca o süreç için ayarlanan değişkenler
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = GitPoulpi'nin kendi değişkenlerinden biri
```

`harness`, Ayarlar > YZ'nin sunduğu önerileri (başsız argümanlar, seçenekler, model
bayrağı ve listesi, ortam değişkenleri) `command` değerinden bağımsız olarak seçer; boş
olduklarında iki anahtar da dosyadan çıkarılır. Ortam değerleri asla günlüğe yazılmaz ve
gizli bilgiye benzediklerinde Ayarlar'da gizlenir.

Ayarlar > YZ, `harness`/`command`/`args`/`env` için (yaygın ajan CLI'ları; başsız
bayrakları, model bayrağı ve yararlı ortam değişkenleriyle) ve `kind`/`base_url`/`model`
için (barındırılan ve yerel HTTP sağlayıcıları) hazır ayarlar sunar; bkz.
[YZ > Yapılandırma](ai.md#yzyi-yapılandırma).

Hiç profil yoksa *Default* adlı bir profil oluşturulur. Eski `[[ai.presets]]` /
`[ai.defaults]` dosyaları yüklemede taşınır ve `[[ai.legacy_presets]]` içinde tutulur.
Geri kalan her şey: [YZ desteği](ai.md).

## `[repos]`, `[session]`

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `repos.clone_root` | `""` | Klonla iletişim kutusunun başladığı yer; boşsa `%USERPROFILE%\source\repos` ya da `~/src`. |
| `repos.clone_start_source` | `"last"` | Klonla iletişim kutusunun hangi kaynakta açıldığı: `last` (varsayılan ya da boş) = en son kapatıldığında gösterdiği kaynak, `url` = her zaman URL ile klonla, ya da bir entegrasyon profilinin kimliği (`[[integrations]]` girdisindeki `id`). Silinmiş, devre dışı bırakılmış ya da bağlı olmayan bir profil URL ile klonla'ya döner. Son kaynağın kendisi durumdur; yapılandırma klasöründeki `clone-source.toml` içinde tutulur, burada değil. |
| `repos.browse_root` | `""` | Gözat'ın açıldığı yer; boşsa son seçilen klasör, sonra ana klasör. |
| `repos.show_hidden` | `false` | Klasör seçici gizli klasörleri gösterir (`Alt+H` bunu açıp kapatır ve kaydeder). |
| `repos.show_siblings` | `false` | Klasör seçicinin sol ağacı, açık her düzeyin diğer klasörlerini de gerçek bir hiyerarşi olarak listeler (`Alt+S` bunu açıp kapatır ve kaydeder). |
| `session.per_repo_state_dir` | `"in_git_dir"` | GitPoulpi'nin bir depo hakkında hatırladıklarını (ref filtreleri, katlama durumu, kişisel iş akışı geçersiz kılması, barındırma önbelleği) nerede tuttuğu: `in_git_dir` = `<git dir>/gitpoulpi` (tüm worktree'ler paylaşır, `git status` içinde asla görünmez), `in_worktree` = `<depo>/.gitpoulpi` (bir ekibin `workflow.toml` dosyasını commit'leyebileceği tek yer), `user_data` = `<işletim sistemi veri klasörü>/repos/<ad>-<kimlik>` (Windows'ta `%APPDATA%\GitPoulpi\data`, Linux'ta `$XDG_DATA_HOME/gitpoulpi`, macOS'ta `~/Library/Application Support/GitPoulpi`). Eski `".gitpoulpi"` değeri hâlâ `in_worktree` anlamına gelir. Değiştirmek hiçbir şeyi geri taşımaz: eski bir `<depo>/.gitpoulpi` içinde bulunan veriler, ilk gerektiğinde seçilen yere taşınır. |

## `[log]` (uygulama günlüğü)

Etkinlik Günlüğü'nün "Uygulama" sekmesinin gösterdiği ve "Tanılamayı kopyala"nın alıntıladığı şey. Ayrıntılar: [Bir şey başarısız olduğunda](troubleshooting.md#bir-şey-başarısız-olduğunda).

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `log.level` | `"info"` | Kaydedilen en ayrıntılı satır: `error`, `warn`, `info`, `debug` veya `trace`. Etkinlik Günlüğü'ndeki düzey kutusu bunu değiştirir (ve buraya kaydeder). |
| `log.retention_days` | `7` | Saklanan günlük dosyası günü, bugün dahil: 1 ile 3650 arası, yani on yıl (daha büyük ya da küçük bir değer sınırlanır). Ayarlar onu hangi birimde gösterirse göstersin her zaman bir gün sayısıdır. Daha eski dosyalar GitPoulpi başlarken silinir. Bir günün dosyası ayrıca `log.max_file_mb` ile sınırlıdır, yani en kötü durum yaklaşık `2 x max_file_mb x retention_days` olur. |
| `log.retention_unit` | `"auto"` | Ayarlar'ın `log.retention_days` değerini hangi birimle gösterdiği: `days`, `weeks`, `months`, `years` veya `auto`. Yalnızca görüntüleme içindir, dosyaların ne kadar saklandığını asla değiştirmez. Bir hafta 7 gündür, bir ay 30 gündür (takvim ayı değil, böylece ne zaman uygulanırsa uygulansın aynı gün sayısı olur) ve bir yıl 365 gündür. `auto` ve bu anahtardan önce yazılmış bir yapılandırma, günü tam bölen en büyük birimi seçer (14 gün "2 hafta", 45 gün "45 gün" okunur). Günü bölmeyen bir birim (`weeks` ile `retention_days = 10`) yok sayılır ve günler gösterilir. Ayarlar iki anahtarı da yazar; daha eski bir GitPoulpi birimi yok sayar ve günü yine okur. |
| `log.max_file_mb` | `5` | Bir günün dosyası bu boyutta tek bir `.old` yedeğine döndürülür, yani bir gün hiçbir zaman bunun iki katından fazla yer kaplamaz. |
| `log.max_file_unit` | `"auto"` | Ayarlar'ın boyutu hangi birimle gösterdiği: `mib`, `gib` (1024 MiB) veya `auto`. Yalnızca görüntüleme içindir: boyut her zaman `max_file_mb` MiB'dir, 1 ile 1024 arası. |
| `log.timings` | `true` | Etkinlik Günlüğü'nün Git işlemleri sekmesi her işlemin ne kadar sürdüğünü gösterir (`Push - 123 ms`: bir depoyu açma, yenileme, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). Süreleri pencere ölçer, isteğin gönderilmesinden daemon'ın yanıtına kadar, yani IPC gidiş dönüşünü de içerir. Kapalıyken süreleri ve başarılı işlemlerin girdilerini gizler. Başarısızlıklar her zaman listelenir. |
| `log.extended` | `false` | "Genişletilmiş günlükleme": Git işlemleri sekmesi arka plan işlemlerini de (durumu yoklayan periyodik yenileme, otomatik fetch) süreleriyle listeler. Bunlar her zaman uygulama günlüğüne `debug` düzeyinde yazılır. |
| `log.commit_count` | `"system-git"` | Durum çubuğunun `yüklenen / toplam` gösterimindeki toplamı nasıl öğrendiği. `"system-git"`, `git rev-list --count` çalıştırır (tek süreç, normal bir makinede en hızlısı: git commit-graph dosyasını okur). `"libgit2"`, süreç içinde bir libgit2 revwalk ile sayar ve hiçbir şey başlatmaz: her komut satırı sürecinin başlaması yavaşsa, genellikle gerçek zamanlı tarama yapan bir virüsten koruma yazılımı her birini incelediği için, bunu seçin; 50.000 commit için arka planda yaklaşık 12 ms yerine 200 ms sürer ama süreç başlatılmaz. `"off"` hiç saymaz: çubuk yalnızca yüklenen commit'leri gösterir. Hemen uygulanır. Yola göre süzülmüş bir sayım her zaman sistem git'ini kullanır. |
| `log.use_mailmap` | `true` | Her yazarı Log'da, blame'de ve dosya geçmişinde deponun `.mailmap` dosyasının (ve `mailmap.file` değerinin) adlandırdığı gibi gösterir: daemon git'ten `%aN`/`%aE` ister (sistem backend'i) ya da aynı dosyaları kendisi uygular (libgit2 backend'i), böylece yanıt aynıdır. Saklanan commit'ler değişmez ve hiçbir şey yeniden yazılmaz. Log'da yazara göre aramak gösterilen adlarla eşleşir. Hemen uygulanır; Log 2 sn'lik yoklamasında yeniden okur. |

Dosyalar, işletim sisteminin durum klasöründe `gitpoulpi-YYYY-MM-DD.log` adıyla bulunur (Linux'ta `~/.local/state/gitpoulpi/logs`; Windows ve macOS'ta `<cache>\logs`; `GITPOULPI_CONFIG_DIR` altında `<config>/cache/logs`). Token'lar, parolalar ve `Authorization` başlıkları herhangi bir şey yazılmadan önce maskelenir.

## `[search]` (çalışma alanı araması)

`gitpoulpi search` (ve MCP aracı) neye bakar. Ayrıntılar: [CLI'da çalışma alanları ve arama](cli-reference.md#çalışma-alanları-ve-arama).

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `search.commit_limit` | `200000` | Arama dizininin depo başına bellekte tuttuğu, mesajlarına bakılan commit sayısı (en yeni önce; tüm dallar, uzak dallar ve tag'ler üzerinden), her biri yaklaşık 100 bayt artı mesajı. Daha uzun bir geçmiş burada kesilir ve yanıt `partial` der. `0` commit aramasını kapatır (dallar, tag'ler, stash'ler ve worktree'ler yine aranır). Canlı okunur: bir sonraki arama değişikliğe uyar. |

## `[rewrite]` (geçmişi yeniden yazma)

`gitpoulpi rewrite` ve geçmişi yeniden yazma ekranının neyi koruduğu. Ayrıntılar: [Geçmişi yeniden yazma](history-rewrite.md).

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Zaten gönderilmişlerse bir yeniden yazmanın dokunmadığı dallar (adlar ya da `*` ile herhangi bir metinle eşleşen desenler); çalıştırma onları açıkça listelemedikçe (`--include-protected BRANCH`; push için `--include-all-protected`; ekranda dal başına onay kutusu). Bir deponun `workflow.toml` dosyasının `protected` işaretlediği tabanlara eklenir. Bir yeniden yazmanın push'u onları aynı şekilde atlar. Bu, GitPoulpi'nin kendi korumasıdır: force push'un kabul edilip edilmeyeceğine yine barındırıcının dal koruması karar verir. Canlı okunur: bir sonraki önizleme değişikliğe uyar. |
| `rewrite.external_scripts` | `["shell"]` | Betik kuralının hangi harici betik dillerini kullanabileceği: `"shell"` (bash veya sh; PowerShell betikleri için pwsh ya da powershell), `"python"`, `"lua"` (kurulu bir Lua; yerleşik Lua her zaman açıktır ve girdi gerektirmez) değerlerinden herhangi biri. **Harici bir betik korumalı alanda değildir: sizin yetkilerinizle çalışır.** Listelenmeyen bir dil ekranda soluk görünür ve CLI, MCP ve daemon tarafından reddedilir (`scripts_disabled`). Listelenmiş olsa bile depodan ya da paylaşılan bir kural setinden gelen harici bir betik, ona güvendikten sonra çalışır (`gitpoulpi rewrite script trust DOSYA` veya ekrandaki soru); güven betiğin içerik özetine bağlıdır, bu yüzden değişen bir betik yeniden sorar. Python'u `config.toml` dosyasını düzenleyerek açın (bir liste `config set` ile ayarlanamaz): `[rewrite]` altında `external_scripts = ["shell", "python"]`. Canlı okunur. |
| `rewrite.script_timeout_ms` | `5000` | Bir harici betiğin tek bir commit'e yanıt vermek için sahip olduğu süre. Zaman aşımında tüm süreç ağacı sonlandırılır ve yeniden yazma başarısız olur (`script_failed`, commit'i belirtir); hiçbir şey taşınmamıştır. `0` varsayılan olarak okunur. Canlı okunur. |
| `rewrite.script_timeout_unit` | `"auto"` | Ayarlar'ın süre sınırını hangi birimle gösterdiği: `milliseconds`, `seconds` veya `auto`. Yalnızca görüntüleme içindir: sınır her zaman `script_timeout_ms` milisaniyedir. Ayarlar 1 ile 600000 arasını (on dakika) kabul eder; `0` yine varsayılan olarak okunur. Canlı okunur. |
| `rewrite.script_total_timeout_s` | `3600` | Bir harici betiğin tek bir geçişinin toplamda ne kadar sürebileceği. `0` varsayılan olarak okunur. Canlı okunur. |
| `rewrite.script_total_timeout_unit` | `"auto"` | Ayarlar'ın süre sınırını hangi birimle gösterdiği: `seconds`, `minutes`, `hours` veya `auto`. Yalnızca görüntüleme içindir: sınır her zaman `script_total_timeout_s` saniyedir. Ayarlar 1 ile 86400 arasını (bir gün) kabul eder; `0` yine varsayılan olarak okunur. Canlı okunur. |

## `[updates]` (daha yeni bir sürümü arama)

Başlık çubuğundaki sürüm çipinin daha yeni sürümlerle ilgili yaptığı. Ayrıntılar: [Güncellemeler](updates.md).

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"`, günde bir kez arka planda GitHub'a daha yeni bir sürüm olup olmadığını sorar ve sürüm çipini `vX.Y.Z kullanılabilir` yapar. `"auto"`, bu derleme bir sürüm arşivinden geldiyse güncellemeyi kendiliğinden indirip hazırlar de (yeniden başlatmadan önce yine sorar); başka derlemelerde `"notify"` gibi davranır. `"off"` hiç istek yapmaz. Hemen uygulanır. |
| `updates.channel` | `"stable"` | `"stable"` yalnızca tam sürümleri dikkate alır; `"prerelease"` sürüm adaylarını (`-rc.1`) da sunar. Hemen uygulanır. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | Sürümleri denetlenen github.com `sahip/ad` değeri. Bu depo özelken, github.com için saklı bir token yoksa GitHub 404 yanıtı verir (Ayarlar > Entegrasyonlar); token yoksa denetim sessizce başarısız olur ve uygulama günlüğüne bir satır yazar. |

Bir yönetici `mode`, `channel` (`release_channel` olarak) ve `repo` değerlerini, ayrıca birkaç güncelleme seçeneğini makine genelinde bir politika dosyasında veya Windows kayıt defterinde ayarlayabilir. Politikanın ayarladığı bir anahtar `config.toml` dosyasına üstün gelir ve Ayarlarda **Yöneticiniz tarafından ayarlandı** olarak görünür. Politikadaki `mode` bir üst sınırdır: daha düşük bir değer seçebilirsiniz, asla daha yüksek bir değer değil. Bkz. [Kurumsal dağıtım](enterprise-distribution.md#politika-dosyası-ve-kayıt-defteri).

## `[privacy]` (hassas dosyalar)

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Gitignore tarzı yol kalıpları. `/` içermeyen bir kalıp dosya adıyla her derinlikte eşleşir, `/` içeren tüm yolla eşleşir (baştaki `/` onu köke sabitler); `*` ve `?` tek bir klasörün içinde kalır, `**` klasörleri aşar; bir `!` önceki bir kalıbın eşleştirdiğini yeniden serbest bırakır ve son eşleşme kazanır; büyük/küçük harf yok sayılır. stage koruması ve `ai.privacy.exclude_sensitive_files` tarafından kullanılır. |
| `privacy.staging_guard` | `true` | Bir kalıpla eşleşen ya da içeriğinde tanınan bir gizli bilgi bulunan bir dosyayı stage'lemek önce sorar (TUI), `--allow-sensitive` verilmedikçe reddedilir (CLI) ya da `mcp.allow_sensitive_staging` ajanın `allow_sensitive` geçmesine izin vermedikçe reddedilir (MCP). Her biri 1 MiB'a ve toplam 32 MiB'a kadar metin dosyaları okunur; ikili dosyalar yalnızca adıyla eşleştirilir. Bkz. [Stage'leme ve commit](staging-and-committing.md#hassas-dosyalar).

## `[conflict_prevention]` (temel dal listeleri)

Proaktif çakışma algılamanın geçerli dalın temel dalı olarak alabileceği dallar bir desen listesidir (bkz. [Merge çakışmalarını çözmek](merge-conflicts.md#çakışma-önleme)). Listeler adlandırılıp paylaşılabilir; Ayarlar > Çakışma önleme bunları düzenler.

| Anahtar | Varsayılan | Değerler ve etki |
|---|---|---|
| `conflict_prevention.default_list` | `""` | Kendi listesi olmayan bir deponun kullandığı listenin adı. Boş ya da hiçbir listede olmayan bir ad, yerleşik `**` (her dal) anlamına gelir. Her denetimde yeniden okunur, bu yüzden bir değişiklik yeniden başlatmadan açık depolara uygulanır. |
| `conflict_prevention.lists` | yok | Adlandırılmış listeler, her biri bir tablo: `[conflict_prevention.lists.release]` ve `patterns = ["origin/release/**", "!origin/release/old"]`. Boş bir `patterns` her dala izin verir. Bir listeyi düzenlemek, onu izleyen her depoya uygulanır. |

Bir deponun hangi listeyi izlediği `repo-state.toml` dosyasındadır (aşağıda). Öncelik sırası: deponun kendi desenleri, sonra izlediği adlandırılmış liste, sonra `default_list`, sonra `**`.

## `[keybindings]`

TUI'nin kısayollarını yeniden atar; Ayarlar > Klavye ve fare bunu düzenler.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # varsayılan Ctrl+G
nav_prefix = "Alt+n"             # varsayılan Ctrl+N
settings   = ["Ctrl+,", "F2"]    # liste = alternatifler
tab_menu   = []                  # atanmamış

[keybindings.git_prefix]         # Git önekinden sonra basılan tuş
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; bir değer o eylemin varsayılanlarının
yerini alır, satırı kaldırmak onları geri getirir. Tuş dizileri: `Ctrl+Shift+N`, `Alt+n`,
`F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Hatalı bir girdi hiçbir şeyi engellemez:
TUI onu yok sayar, varsayılanı korur ve durum çubuğunda bunu söyler.
Bağlamlar, eylem kimlikleri, varsayılanlar ve çakışma kuralları: [Klavye kısayolları](keybindings.md).

## Depo başına ayarlar: `.gitpoulpi/repo-state.toml`

Deponun içinde tutulur; klasör kendini yok sayar (`*` içeren bir `.gitignore`), bu yüzden
asla bir değişiklik olarak görünmez. TUI ve CLI tarafından yazılır; nadiren düzenlersiniz.

| Anahtar | Nedir | Nasıl ayarlanır |
|---|---|---|
| `[preferences] encoding` | Burada `git.default_encoding` değerini geçersiz kılar | Ayarlar > Git ve Kodlama |
| `[preferences] lfs_enabled` | Burada `git.lfs_enabled` değerini geçersiz kılar | Ayarlar > Git ve Kodlama |
| `[preferences] profile` | Burada commit'leri imzalayan `[[profiles]]` id'si; yoksa `general.active_profile` | araç çubuğunun profil açılır listesi, **Depo: Profili Değiştir...** |
| `[preferences] autocrlf`, `eol` | *Henüz kullanılmıyor* | Ayarlar > Git ve Kodlama |
| `[preferences] push_after_commit` | Her commit sonrasında dalı upstream'ine push eder (yokken kapalı) | Ayarlar > Depoya özel ayarlar > Commit |
| `[preferences] skip_hooks_by_default` | Commit formundaki Hook'ları atla işaretli başlar (yokken kapalı) | Ayarlar > Depoya özel ayarlar > Commit |
| `[preferences] squash_merge` | Yerel bir dal merge'ü squash yapar: stage'ler, merge commit'i yok (yokken kapalı) | Ayarlar > Depoya özel ayarlar > Merge seçenekleri |
| `[preferences] template_apply`, `template_strip_comments` | Boş bir commit formunu şablonla önceden doldur; commit sırasında `#` yorum satırlarını kaldır (ikisi de yokken kapalı) | Ayarlar > Depoya özel ayarlar > Mesaj şablonu |
| `[preferences] conflict_detection` | Bu depo için proaktif çakışma algılama; yokken `general.conflict_detection` izlenir | Ayarlar > Depoya özel ayarlar > Çakışma önleme |
| `[preferences] conflict_base_branches` | Deponun kendi Temel dallar desenleri, satır başına bir tane (hâlâ **Kendi listesi** olarak kullanılır) | Ayarlar > Depoya özel ayarlar > Çakışma önleme |
| `[preferences] conflict_base_source` | Temel dalların nereden geldiği: `default` (genel varsayılan liste), `list` (`conflict_base_list`) veya `own` (`conflict_base_branches`). Yokken: varsa kendi desenleri, yoksa varsayılan | Ayarlar > Depoya özel ayarlar > Çakışma önleme |
| `[preferences] conflict_base_list` | Kaynak `list` olduğunda bu deponun izlediği `[conflict_prevention.lists.<ad>]`; silinmiş bir liste varsayılana döner | Ayarlar > Depoya özel ayarlar > Çakışma önleme |
| `commit-template.txt` (`repo-state.toml` dosyasının yanında bir dosya) | Commit şablonu; git'in `commit.template` ayarı ona işaret eder | Ayarlar > Depoya özel ayarlar > Mesaj şablonu |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: bu deponun YZ talimatları | Ayarlar > Depoya özel ayarlar > YZ |
| `[preferences.ai_overrides.<action>]` | Burada tek bir eylem için `profile`, `instructions` | Ayarlar > Depoya özel ayarlar > YZ |
| `default_remote` | Tek tıklamalı Push/Pull/Fetch'in kullandığı uzak depo | uzak depo menüsü, `gitpoulpi remote default <name>` |
| `hosting_integration` | Buraya sabitlenen `[[integrations]]` profili | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | PR / issue listelerinin okuduğu sağlayıcı projesi, `{ integration, project, filter }` olarak. `filter` isteğe bağlıdır: liste sorgusuna AND ile eklenen, sağlayıcıya özgü ek bir koşul (bir Jira JQL parçası, bir Azure WIQL koşulu, Trello liste adları); GitHub ve GitLab bunu yok sayar. | kenar çubuğu bölümleri, `gitpoulpi hosting link` |
| `[preferences] merge_fast_forward` | Burada `git.merge_fast_forward` değerini geçersiz kılar: `never`, `allowed` ya da `only` | Ayarlar > Depoya özel ayarlar > Merge seçenekleri |
| `[preferences] default_push_targets` | Yeni bir dalın da push ettiği uzak depolar; dal ilk kez bir upstream edindiğinde dala kopyalanır | Ayarlar > Depoya özel ayarlar > Git ve kodlama; bkz. [Birden çok uzak depo](multiple-remotes.md) |
| `[preferences.ai_review]`, `[preferences.ai_hosting_reply]` | `[ai.review]` ve `[ai.hosting_reply]` anahtarlarının bu depoya özgü değerleri; olmayan anahtarlar genel değerleri devralır | Ayarlar > Depoya özel ayarlar > YZ |
| `hosting_post_as`, `hosting_identity` | Bu depoda barındırıcıya gönderdiğiniz mesajların altında yayımlandığı ad (`general.hosting_post_as` değerini geçersiz kılar) ve hiçbiri ayarlı değilken GitPoulpi'nin bir bot token'ıyla gönderilen ilk mesajdan öğrendiği oturum adı | Ayarlar > Depoya özel ayarlar > SSH ve kimlik bilgileri; bkz. [Konuşmalar](conversations.md#şu-adla-gönderme) |
| `assignment_blocked` | Bu deponun issue'larında ve pull request'lerinde GitPoulpi'den hiçbir atama yapılamaz | `gitpoulpi hosting assignment --block \| --allow` |
| `board_labels` | İş akışı olmayan bir barındırıcıda issue panosunun sütunları olan etiketler, pano sırasıyla | panonun **Sütunlar** düğmesi |
| `conversation_seen`, `conversation_events`, `posted_comment_ids` | Her konuşmada en son okuduğunuz yer (gerisi okunmamıştır), sistem olaylarının gösterilip gösterilmediği ve bu kurulumun gönderdiği yorumlar (bir botun düzenleyebileceği ya da silebileceği olanlar) | konuşma görünümü yazar |
| `reviews` | Uygulama içi incelemenin istek başına yerel durumu: bekleyen yorumlar, görüldü işaretli dosyalar | inceleme görünümü yazar |
| `ref_visibility` | Gizlenen / solo edilen ref'ler | Refs kenar çubuğu, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Grafikten bütünüyle gizlenen uzak depolar (tüm dallar, sonradan fetch edilenler de) | uzak deponun ⊙ işareti ve menüsü, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Daraltılmış kenar çubuğu bölümleri ile dal ve tag klasörleri. `[folds]` ayrıca `status_folders` (Unstaged / Staged dosya ağaçlarının daraltılmış klasörleri) ve `commit_folders` (bir commit'in değişen dosyalar ağacının daraltılmış klasörleri, her commit için aynı) tutar; her biri 500 yolla sınırlıdır, en eskisi önce atılır | kenar çubuğu, dosya ağaçları |
| `pending_auto_stash` | GitPoulpi'nin yaptığı ve henüz geri koymadığı bir stash | daemon tarafından yazılır; bkz. [Otomatik stash](log-and-history.md#otomatik-stash) |

## Diğer ortam değişkenleri

| Değişken | Etki |
|---|---|
| `GITPOULPI_CONFIG_DIR` | Yapılandırma klasörü (yukarıya bakın). |
| `GITPOULPI_TOKEN_<ID>` | `<ID>` profili için bir barındırma token'ı, saklanan token'ın üzerinde (CI, tek seferlik kullanım). |
| `GITPOULPI_GIT` | Çalıştırılacak `git` çalıştırılabilir dosyası (varsayılan: `PATH` üzerindeki `git`; Windows'ta Git for Windows'un `cmd\git.exe` kısayolunun ardındaki gerçek `mingw64\bin\git.exe`, bu çağrı başına bir işlem tasarruf ettirir). |
| `GITPOULPI_LANG=fr` | Arayüz dili, `ui.language` değerinin üzerinde (herhangi bir `ui.language` değeri; `qps` çevirmenler için sahte bir yerel ayardır). Bilinmeyen ya da boş bir değer yok sayılır. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Yapılandırma ne derse desin hiç resim yok. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Resim çözünürlüğü ipuçları (piksel cinsinden hücre boyutu; aşırı örnekleme, varsayılan 2, `1` CPU tasarrufu sağlar). |
| `GITPOULPI_NO_BROWSER=1` | Hiçbir zaman bir web tarayıcısı başlatma (URL yine gösterilir). |
| `GITPOULPI_TIMING=1` | İstek sürelerini geçici klasördeki `gitpoulpi-timing.log` dosyasına yazar (performans raporları). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | TUI bir hatayla çıktıktan sonra Enter'ı bekleme. |
