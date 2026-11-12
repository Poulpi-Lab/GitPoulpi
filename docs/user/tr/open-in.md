[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · **Türkçe** · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Şurada aç

Başlık çubuğunda depo yolunun yanındaki **Şurada aç ▾** düğmesi, etkin sekmenin
klasörünü başka bir yerde açar: terminalinizde, yeni bir çoklayıcı sekmesinde ya
da bölmesinde, bir düzenleyicide, dosya yöneticisinde. Listede nelerin olacağını
siz seçersiniz. Hiçbir şey sabit kodlanmış değildir: her giriş bir etiket ve bir
komuttur.

## Kullanımı

- **Şurada aç ▾ düğmesine tıklayın**: bir liste açılır. Çalıştırmak için bir
  girişe tıklayın. Varsayılan giriş `•` ile işaretlidir; son satır
  **Yapılandır...**, [Ayarlar](settings.md#şurada-aç)'ı Şurada aç sayfasında açar.

  ![Şurada aç listesi: Terminal, iki düzenleyici, bir Git aracı ve dosya yöneticisi; varsayılan bir noktayla işaretli ve sonda Yapılandır](../images/open-in-menu.png)

- **Klavye**: `Alt+O` aynı listeyi açar (`Up`/`Down`, `Enter`, `Esc`; yediden
  fazla giriş varsa aramak için yazın). `Ctrl+Alt+O` varsayılan girişi hemen
  çalıştırır. Palet (`Ctrl+P`): `Sekme: Şurada Aç...` ve `Sekme: Varsayılanda Aç`.
  Başlık çubuğunda düğmeye yer olmasa bile palet komutları çalışır. Her iki tuş da
  yeniden atanabilir; bkz. [Klavye kısayolları](keybindings.md).
- **Bir depo sekmesine sağ tıklayın**: sekme menüsünde aynı girişleri içeren bir
  **Şurada aç** alt menüsü vardır.
- Liste, bu oturumda en son kullandığınız girişle başlar.
- Durum çubuğu hemen "... içinde açılıyor..." der. Program yaklaşık 1,5 saniye
  içinde başarısız olursa (örneğin sunucusu olmayan tmux ya da hatalı bir bayrak),
  hata çıktısının ilk satırı durum çubuğunda görünür. Kurulu olmayan bir program
  hemen adıyla belirtilir.

Kendi listenizi kaydedene kadar varsayılan giriş, algılananlardan seçilir: herdr
içindeyseniz "Herdr: new tab"; Omarchy'de kitty'nin yeni penceresi (kitty onun
terminalidir); aksi halde dosya yöneticisi.

Düğme, GitPoulpi'nin başlatıldığı klasör dahil her depo sekmesinde ve
`ui.tab_dir_follow = "auto"` kipinde görünür. `auto` kipinde varsayılan giriş,
yeni bir sekme açtığınızda da kendiliğinden çalışır (başlangıçta geri yüklenen
sekmeler için değil). Bkz. [Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md).

## Bir giriş neye benzer

`config.toml` içinde giriş başına bir `[[ui.open_in]]` tablosu:

```toml
[ui]
open_in_default = "Herdr: new tab"   # bir etiket; boşsa ilk giriş

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Bunu nadiren elle yazarsınız: Ayarlar sayfası düzenler. Her anahtar
[yapılandırma başvurusunda](config-reference.md) yer alır.

### Komutlar ve değişkenler

Komut önce bağımsız değişkenlere bölünür (boşluklar, `'...'` ve `"..."` sözcükleri
gruplar), ardından değişkenler her bağımsız değişkenin içinde değiştirilir. Adında
boşluk olan bir klasör bu yüzden tek bir bağımsız değişken olarak kalır. Komutunuz
bir tane başlatmadıkça **hiçbir kabuk devreye girmez**.

| Değişken | Yerine geçen |
|---|---|
| `${cwd}` | Sekmenin klasörü. |
| `${repo}` | Deponun kökü (bir worktree sekmesinde `${cwd}`'den farklıdır). |
| `${name}` | Klasörün temel adı. |
| `${branch}` | Checkout edilmiş dal; ayrıkken boş. |
| `${web_url}` | Origin uzak deposunun GitHub veya GitLab olduğundaki web sayfası; aksi halde boş. |
| `${env:NAME}` | `NAME` ortam değişkeni; ayarlı değilse boş. |

Eski `{repo}` hâlâ çalışır ve `${cwd}` anlamına gelir. Başka her `${...}`,
değişkeni adlandıran bir hatadır.

Program, sekmenin klasörü çalışma klasörü olarak başlatılır ve bir kabuk parçacığı
için kullanışlı üç ortam değişkeni alır:

| Değişken | Değer |
|---|---|
| `GITPOULPI_CWD` | Sekmenin klasörü. |
| `GITPOULPI_REPO` | Deponun kökü. |
| `GITPOULPI_BRANCH` | Dal veya boş. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

Windows'ta program `PATH` ve `PATHEXT` üzerinden aranır; bu yüzden `code` ve
`cursor` (`.cmd` dosyaları) çalışır ve hiçbir konsol penceresi yanıp sönmez.

## Hazır ayarlar

Ayarlar > Şurada aç > **Hazır ayar ekle**, yalnızca işletim sisteminiz için
hazır girişleri kategoriye göre listeler (Windows hazır ayarları Linux'ta hiç
görünmez, vb.): satır kategoriyle başlar, `Gezgin: ...`, `Çoklayıcı: ...`,
`Terminal: ...`, `Düzenleyici: ...`, `Git aracı: ...`, `Yapay zekâ ajanı: ...`,
`Web: ...`. En üstte **Algılandı** altında hesaplanan varsayılan listenin girişleri
yer alır. Her kategorinin içinde hemen kullanabileceğiniz hazır ayarlar önce gelir
(o çoklayıcının veya terminalin içinde çalışıyorsunuz ve program `PATH`
üzerinde), sonra kurulu olanlar, sonra geri kalanlar. Aramak için açılır kutuya
yazın, örneğin `lazygit` veya `Git aracı:`. **Geçerli sekmede dene**, bir girdiye
güvenmeden önce onu dener.

![Şurada aç sayfasının + Ekle listesi: önce Algılanan girişler, sonra Gezgin, Çoklayıcı ve diğer kategoriler; her hazır ayarın başında kategorisi var](../images/settings-open-in-presets.png)

"Bir bölmede araç çalıştır" hazır ayarları (`lazygit in Herdr tab`,
`yazi in tmux window`, `Claude Code in Herdr split`...) yalnızca ev sahibi uygulama
(herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows Terminal) ve
araç kuruluysa sunulur. `$EDITOR in ...`, `EDITOR` değişkeninizin adlandırdığı
düzenleyiciyi (herdr'de `vi`'ye düşerek) herdr veya tmux içinde çalıştırır.

Herdr'in araç hazır ayarları sekmeyi ya da bölmeyi oluşturur, sonra komutu kendi
kabuğuna yazar; böylece araç klasörde başlar ve bölme araç çıktıktan sonra da
açık kalır. `Yapay zekâ ajanı` olanlar `herdr agent start` kullanır; böylece herdr
ajanın durumunu izler.

| Kategori | Hazır ayarlar (`V` = çalıştırarak veya kurulu yardıma karşı doğrulandı) |
|---|---|
| Gezgin | Windows `explorer.exe` (V), macOS Finder (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; bölmede/sekmede/terminalde yazi, superfile, nnn, lf, ranger |
| Çoklayıcı | herdr sekmesi, çalışma alanı, sağa/aşağı bölme (V); tmux penceresi, sağa/aşağı bölme, ayrık oturum (V); zellij bölmesi, sekmesi, yüzen; WezTerm sekmesi, bölmesi, yeni pencere; kitty sekmesi, bölme penceresi, OS penceresi (V: bayraklar yardımdan; `allow_remote_control` gerekir); screen; cmux; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (pencere, sekme, bölme ve PowerShell 7, Windows PowerShell, Command Prompt, WSL için birer sekme), PowerShell 7 / Windows PowerShell / Command Prompt / WSL pencereleri, Git Bash |
| Düzenleyici | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, bir bölmede/sekmede/terminalde `$EDITOR` |
| Git aracı | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; bir bölmede/sekmede/terminalde lazygit, tig, gitui (herdr, tmux, kitty ve `xdg-terminal-exec` olanlar V) |
| Yapay zekâ ajanı | Claude Code, Codex, Gemini CLI, opencode: bir herdr sekmesinde/bölmesinde (V, Claude ile), tmux, zellij, WezTerm, kitty, yeni terminal |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

Windows'ta WSL: `wsl.exe --cd ${cwd}` Windows yolunu olduğu gibi kabul eder ve
Windows Terminal'in `wsl.exe` sekmesi klasörde başlar; WSL bunu `/mnt/` altına
eşler.

Gezgin, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual Studio ve
Zed (Zed yalnızca Unix'tir) yalnızca var oldukları yerde görünür; tmux, zellij,
cmux, screen, kitty uzaktan denetimi ve `sh` ile çalışan herdr hazır ayarları
yalnızca Unix'tir; herdr (sade hazır ayarlar), WezTerm, Alacritty, VS Code,
Cursor ve JetBrains IDE'leri her yerde sunulur.

### `${web_url}` değişkeni

Deponun `origin` uzak deposunun web sayfası; bir GitHub veya GitLab projesiyse
(github.com, gitlab.com veya `[[integrations]]` içinde girişi olan, kendi
kurduğunuz bir barındırıcı); aksi halde boş olur ve onu kullanan bir giriş boş
bir adres açmak yerine bir mesajla durur. İsteğe bağlıdır: `Repository page on the
web` dışında hiçbir hazır ayar buna ihtiyaç duymaz.

## Ayarlar sayfası

Ayarlar > **Şurada aç** listeyi düzenler. Girişler solda bir
[liste](settings.md#listeler)dir (varsayılan giriş `★` ile işaretlidir), seçili girişin alanları
sağdadır; `+ Ekle`, `Çoğalt`, `Kaldır`, `↑` / `↓` ve bir satırı sürüklemek listeyi yönetir.

| Satır | Ne yapar |
|---|---|
| **Varsayılan giriş** | **Varsayılanda aç**'ın çalıştırdığı giriş. Şurada aç listesinde `•` ile işaretlidir. |
| **Etiket** | Listede gösterilen ad. Benzersiz, boş olmayan. |
| **Komut** | Eklenecek değişken çipleriyle (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) komut. |
| **Geçerli sekmede dene** | Girişi şimdi etkin sekmenin klasöründe çalıştırır. |
| **+ Ekle ▾** | Hazır bir giriş ekler (algılananlar önce) ya da **Özel**: Komut alanı sonra düzenlenen boş bir giriş. |
| **Algılanan varsayılanları geri yükle** | Listeyi bu makinede algılananlarla değiştirir ve varsayılanı temizler. |

Düzenlemeler yaptıkça kaydedilir. İlk düzenleme `[[ui.open_in]]` dizisinin tamamını
`config.toml` dosyasına yazar; o zamana kadar liste hesaplanır
(bkz. [Eski anahtarlar](#eski-anahtarlar)) ve dosyada `open_in` anahtarı yoktur.
Ayarlar araması bu sayfayı da bulur.

## İkili dosyalar: Şununla aç

İkili bir dosyanın (PDF, Word belgesi, hesap tablosu, arşiv...) Diff Görünümü'nde ya da Dosya Görünümü'nde gösterilecek metni yoktur. İşletim sistemi türü için bir uygulama biliyorsa, görünüm olağan bildirimin yanında bir **Şununla aç** düğmesi gösterir. Listede önce sistemin bu türü açtığı uygulama (noktayla işaretli), ardından sistemin o tür için kaydettiği diğer tüm uygulamalar (sistemin kendi *Birlikte aç* listesi) yer alır. Linux'ta dosya adının MIME türü, varsayılan için `mimeapps.list`, diğerleri için `.desktop` dosyalarıdır; Windows'ta kayıt defteri (kullanıcının seçimi ve uzantı için kayıtlı programlar); macOS'ta yalnızca varsayılan uygulama.

`o` varsayılan uygulamayı çalıştırır, `Shift+O` listeyi açar. İkisi de komut paletindedir (*Dosya: Varsayılan Uygulamayla Aç*, *Dosya: Birlikte Aç...*); **Birlikte aç...** ayrıca durum panelindeki bir dosyanın ve bir commit'in dosya listesinin sağ tık menüsündedir. Arama arka planda, uzantı başına bir kez çalışır: bu sürede görünüm "uygulamalar aranıyor..." der. Sistemin hiçbir uygulama bilmediği türler ve düz metin, sade bildirimi korur ve düğmesi yoktur.

![Dosya Görünümü'nde bir PDF ve Şununla aç listesi: önce nokta ile işaretli varsayılan uygulama, ardından sistemin bu tür için kayıtlı tuttuğu diğer uygulamalar](../images/open-with.png)

Çalışma ağacındaki dosya uygulamaya olduğu gibi verilir. Bir commit'in (veya indeksin) sürümü, durum çubuğunda bir bildirimle birlikte **salt okunur geçici bir kopyaya** yazılır; böylece bir uygulama depoyu değiştiremez. İkisi de depoyu değiştirmez: günlük kaydı yok, Geri Al etkilenmez.

Menünün listesi, bir alt menü gibi **Birlikte aç...** satırıyla aynı hizada menünün yanında açılır (sağda yer yoksa solda), hem fareden hem klavyeden. Bir dosya menüsünden açılan resim düzenleyicileri listesi (**Şununla düzenle...**) de aynısını yapar.

### Git LFS dosyaları

Git LFS nesne kartı (bkz. [Git LFS](lfs.md)) metninin altında aynı iki eylemi taşır: **Varsayılan programda aç** ve **Şununla aç**; tuşlar (`o`, `Shift+O`) ve palet satırları aynıdır. Gerçek dosyayı açarlar, işaretçiyi asla: çalışma ağacındaki dosyayı olduğu gibi ya da bir commit'in veya indeksin sürümü için yerel LFS deposundaki nesnenin salt okunur geçici bir kopyasını. Henüz indirilmemiş bir nesnenin kartında düğme yoktur ve durum satırı içeriğin henüz indirilmediğini söyler. Hâlâ yalnızca işaretçiyi tutan bir çalışma ağacı dosyası için de aynı ret geçerlidir: karttan ya da herhangi bir dosya menüsünden **Varsayılan programda aç**, **Şununla aç** ve **Şununla düzenle**, işaretçi metnini açmak yerine içeriğin indirilmediğini söyler (LFS araç çubuğunun pull veya checkout satırlarıyla indirin, sonra yeniden deneyin). Açmak salt okunurdur: günlük kaydı yok, geri alınacak bir şey yok.

### Windows'ta tarayıcılar

Chrome ve Edge `--single-argument %1` kaydeder; bu, anahtardan sonraki her şeyi tırnaklar dahil tek bir adres olarak okumalarına yol açar. GitPoulpi yolu böyle bir programa, Windows'un yaptığı gibi, olduğu gibi ve tırnaksız verir; böylece yolunda boşluk olan bir dosya `http://"c/...` hatasıyla başarısız olmak yerine açılır. Diğer tüm programlar yolu tek bir sıradan argüman olarak alır.

## Eski anahtarlar

Eski `ui.terminal_launch` ve `ui.terminal_launch_command` anahtarları yalnızca
`ui.open_in` yokken okunur. O zaman hesaplanan liste şudur:

1. eski terminal seçiminiz, `auto` değilse (`custom`, `{repo}` ifadesi
   `${cwd}` olarak yeniden yazılmış "Terminal (custom)" olur; bir hazır ayar kimliği
   o hazır ayar olur). `auto` değeri ilk giriş olan "Default terminal" olur.
2. işletim sisteminin dosya gezgini,
3. ortamdan algılanan çoklayıcılar ve terminaller,
4. `PATH` üzerinde bulunan düzenleyiciler.

Listeyi Ayarlar'dan kaydettiğinizde `ui.open_in` devralır ve eski anahtarlar yok
sayılır. **Sekme: Terminalde Aç** ve **Sekme: Dosya Yöneticisinde Aç** palet
komutları bu listeyi kullanmaz.

## Sorun giderme

- *"program X was not found"*: program, GitPoulpi'nin gördüğü `PATH` üzerinde
  değil. GitPoulpi'yi onu içeren bir kabuktan başlatın ya da tam yolu yazın.
- *tmux/zellij/herdr girişi hemen başarısız oluyor*: bunlar çalışan bir sunucuyla
  konuşur; bu yüzden yalnızca GitPoulpi onun içinde çalışırken işe yarar.
- *kitty sekme girişi başarısız oluyor*: `kitty.conf` içinde
  `allow_remote_control` ister.
- *Windows gezgini*: `explorer.exe` başarıda bile 1 koduyla çıkar; GitPoulpi bunu
  başarı sayar.
