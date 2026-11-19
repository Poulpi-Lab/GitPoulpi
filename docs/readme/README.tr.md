[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · **Türkçe** · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi ahtapotu" width="128"></p>

**Masaüstü uygulaması gibi hissettiren, terminalde çalışan bir Git istemcisi.**
GitPoulpi size tıklayabileceğiniz, sürükleyebileceğiniz ve sağ tıklayabileceğiniz
tam bir commit grafiği, her özelliğe ulaşan klavye öncelikli bir komut paleti,
kod barındırma hizmetleri (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; bulutta veya kendi barındırdığınız) ve issue takipçileri (Jira, Trello, Linear, YouTrack, Redmine ya da bir Markdown notları klasörü) entegrasyonu ve aynı işlemlerin betiklenebilir bir CLI ile bir MCP
sunucusu olarak sunulmasını sağlar; böylece bir YZ ajanı onu tıpkı sizin gibi
yönetebilir. GitPoulpi, GitKraken'dan esinlenmiştir ve baştan sona Rust ile
yeniden yazılmıştır: yerel ikili dosyalar, diskte ve bellekte hafif, Electron yok, hesap yok, telemetri yok. En sevdiğiniz terminalde çalışır ([hangisi?](../../docs/user/tr/terminals.md)), çünkü bazılarımız orada yaşıyor.

![GitPoulpi: dal, tag ve bir stash içeren commit grafiği, solda Refs kenar çubuğu, sağda commit formuyla birlikte çalışma değişiklikleri](../../docs/user/images/hero.png)

- **Fare ve klavye, eşit ölçüde.** Tıklayın, çift tıklayın, sağ tıklayın ve
  ref'leri commit'lerin üzerine sürükleyip bırakın; ya da `Ctrl+P` tuşuna basıp
  istediğinizi yazın. Her fare hareketinin bir klavye eşi vardır ve her komut
  paletin içindedir.
- **Gerçek bir commit grafiği.** Dallar, merge'ler, tag'ler, stash'ler ve
  çalışma kopyası tek bir kayan Log içinde; bunu destekleyen terminallerde piksel
  olarak, diğer her yerde karakterlerle çizilir. Çok sayıda koyu ve açık renk
  teması, yazar avatarları, arama.
- **Hassas stage.** Bir dosyayı, bir klasörü, bir hunk'ı ya da tek bir satırı
  doğrudan diff'ten stage'leyin, unstage'leyin veya atın. Satır içi ya da yan
  yana, minimap ile.
- **Geçmişi güvenle düzenleyin.** Herhangi bir commit'i reword veya amend edin (mesaj,
  yazar, tarihler), bir inceleme ekranıyla etkileşimli rebase yapın, birçok commit'i
  önizleme ve tek bir geri almayla bir kerede yeniden yazın, birden çok commit'i
  cherry-pick edin, fast-forward yapın, worktree'ler ve yamalar oluşturun.
- **Barındırma hizmetleri ve issue takipçileri yerleşik.** Pull ve merge request'lere göz atın, checkout edin ve oluşturun (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) ve Gerrit değişikliklerini inceleyin. Issue'lara, work item'lara ve kartlara göz atın ve oluşturun (ayrıca Jira, Trello, Linear, YouTrack, Redmine ve yerel Markdown notları), isterseniz Kanban panosunda. Hesabınızdan klonlayın ve SSH anahtarlarınızı yönetin.
- **Ajanlar için CLI ve MCP.** `gitpoulpi <komut> --json` arayüzün yaptığı her
  şeyi kapsar ve `gitpoulpi mcp serve` bunu MCP araçları olarak sunar. Depo başına bir
  arka plan daemon'u her ön yüzü canlı ve eşzamanlı tutar.
- **İsteğe bağlı YZ, asla zorunlu değil.** Varsayılan olarak kapalıdır ve hesap
  gerekmez: kendi API anahtarınızı ya da `claude -p` gibi yerel bir ajan CLI'sini
  getirin. Commit mesajları üretin, değişiklikleri açıklatın, pull request
  açıklamaları yazdırın, çalışma değişikliklerinizi birden çok commit'e bölün ve
  geçmiş yeniden yazma kurallarının taslağını hazırlatın.
- **Git LFS, alt modüller, worktree'ler, yamalar, blame ve dosya geçmişi, görsel
  diff'leri, yerleşik bir düzenleyici.**

## Ekran görüntüleri

| | |
|---|---|
| ![Material şerit paletli ve satır tonlu Log grafiği](../../docs/user/images/theme-material.png) **Commit grafiği**, burada Material şerit paleti ve satır tonuyla. | ![Her hunk üzerinde Stage Hunk ve Discard Hunk düğmeleri bulunan, değiştirilmiş bir dosyanın diff'i](../../docs/user/images/diff.png) **Diff**, hunk veya satır bazında stage ve atma ile. |
| ![stash sözcüğüyle filtrelenmiş komut paleti](../../docs/user/images/palette.png) **Komut paleti** (`Ctrl+P`) her komuta ulaşır. | ![Çalışma değişiklikleri için dört commit öneren, incelemeye hazır commit composer](../../docs/user/images/composer.png) **Commit composer**, hiçbir şey commit'lenmeden önce YZ'nin önerdiği commit'leri incelemenizi sağlar. |
| ![Cherry-pick, revert, reset, rebase, dal ve tag eylemleri olan bir commit'in sağ tık menüsü](../../docs/user/images/context-menu.png) Eylemleri için her şeye **sağ tıklayın**. | ![Profilleri, eylem başına yönergeleri ve etkinleştirme anahtarı olan Ayarlar'ın YZ sayfası](../../docs/user/images/settings-ai.png) **Ayarlar**, arama ile, isteğe bağlı YZ sayfası dahil. |

Açıklamalı daha fazla görsel [kullanıcı belgelerinde](../../docs/user/tr/README.md) yer alır.

## Kurulum

Sistemde bir `git` gerekir; `git-lfs` isteğe bağlıdır.

### Linux ve macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Windows yükleyicisi

`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` dosyasını
[son sürümden](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) indirin
ve çalıştırın. Yöneticiler için MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

Bkz. [Docker'da çalıştırma](../user/tr/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Sürüm arşivi

[Son sürümden](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)
indirin, açın ve içindeki `INSTALL.txt` dosyasını izleyin.

### Kaynaktan

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Kendi sürüm arşivinizi oluşturma, daha küçük bir özel derleme ve işletim sistemine göre notlar:
[docs/user/tr/installation.md](../user/tr/installation.md) ve
[docs/user/tr/custom-builds.md](../user/tr/custom-builds.md).

## Hızlı başlangıç

```
cd path/to/your/repository
gitpoulpi
```

- `?` her tuş ve hareketi içeren yardımı, `Ctrl+P` komut paletini, `Ctrl+,`
  Ayarlar'ı açar, `q` çıkar.
- Değişikliklerini görmek için bir commit'e tıklayın; dosyaları stage'lemek ve
  commit yapmak için Log'un üst satırına (çalışma değişiklikleri) tıklayın.
- Bunu destekleyen bir terminalde (kitty, Ghostty, WezTerm, iTerm2 (ve sixel üzerinden Windows Terminal ile foot)) grafik piksel
  olarak çizilir. Diğerlerinde (sade bir Windows konsolu, tmux) kendiliğinden
  karakterlere geçer.

Adım adım anlatım [Başlarken](../../docs/user/tr/getting-started.md) sayfasındadır;
[arayüz turu](../../docs/user/tr/interface-tour.md) ekranın her bölgesini adlandırır.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi mcp serve   # give an AI agent the same tools
```

YZ isteğe bağlıdır ve varsayılan olarak kapalıdır: GitPoulpi'nin kendi YZ'sini
*Ayarlar > YZ* içinden açın ([YZ desteği](../../docs/user/tr/ai.md)); harici bir ajanın
GitPoulpi'yi yönetmesine [MCP ve ajanlar](../../docs/user/tr/mcp-and-agents.md)
(izinler, her araç) ve ajan skill'i
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md) ile izin verin.

## Belgeler

Kullanıcı belgelerinin tamamı **[docs/user/tr/README.md](../../docs/user/tr/README.md)** ile başlar:

| | |
|---|---|
| [Başlarken](../../docs/user/tr/getting-started.md) | Kurulum, bir depo açma, ilk commit |
| [Arayüz turu](../../docs/user/tr/interface-tour.md) | Pencerenin her bölgesi, adlarıyla |
| [Log ile çalışmak](../../docs/user/tr/log-and-history.md) | Grafik, ref'ler, dallar, tag'ler, stash'ler, rebase, cherry-pick |
| [Stage ve commit](../../docs/user/tr/staging-and-committing.md) | Durum, diff, hunk'lar ve satırlar, composer, düzenleyici |
| [Ayarlar](../../docs/user/tr/settings.md) | Ayarlar'ın her sayfası ve seçeneği |
| [YZ desteği](../../docs/user/tr/ai.md) | Yerleşik YZ'yi yapılandırma, her YZ eylemi, gizlilik |
| [Barındırma hizmetleri ve issue takipçileri](../../docs/user/tr/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: token'lar, bir depoyu bağlama, pull ve merge request'ler |
| [CLI](../../docs/user/tr/cli-reference.md) ve [MCP](../../docs/user/tr/mcp-and-agents.md) | Betikleyin ya da bir ajanın yönetmesine izin verin (izinler, her araç) |
| [Klavye kısayolları](../../docs/user/tr/keybindings.md) (ve paletteki her komut), [fare](../../docs/user/tr/mouse-and-gestures.md), [yapılandırma](../../docs/user/tr/config-reference.md), [LFS](../../docs/user/tr/lfs.md), [geri alma](../../docs/user/tr/undo-redo.md), [sorun giderme](../../docs/user/tr/troubleshooting.md) | Başvuru ve yardım |

Geliştiriciler: [CLAUDE.md](../../CLAUDE.md) ve `docs/architecture/overview.md`
crate'leri ve tasarımı anlatır; [CHANGELOG.md](../../CHANGELOG.md) her sürümde nelerin
olduğunu listeler.

## Lisans

[MIT](../../LICENSE).

## Katkıda bulunanlar ve üçüncü taraf lisansları

GitPoulpi açık kaynak kütüphaneler üzerine kuruludur ve yayımlanmış temalardan renk değerleri kullanır. Tema yazarları dahil, lisansları ve teşekkürleri [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt) dosyasındadır. Dosya her sürümle birlikte gelir; Hakkında penceresi ve `gitpoulpi licenses` komutu da onu gösterir.

## Ticari markalar

GitKraken, ilgili sahibinin ticari markasıdır. GitPoulpi bağımsız bir projedir; bu marka sahibiyle bağlantılı değildir, onun tarafından onaylanmamış veya desteklenmemiştir; ad yalnızca uyumluluğu ve ilham kaynağını anlatmak için anılır.
