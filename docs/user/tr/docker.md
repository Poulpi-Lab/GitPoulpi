[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · **Türkçe** · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Docker'da çalıştırma

`poulpilab/gitpoulpi` imajı, resmi sürüm çalıştırılabilir dosyalarını (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) `debian:bookworm-slim` üzerinde, `git`, `git-lfs`, `openssh-client`, `less` ve `tini` ile birlikte taşır. `linux/amd64` ve `linux/arm64` üzerinde çalışır. Docker Hub'dadır.

## Arayüzü başlatın

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` arayüze bir terminal verir. Onsuz programın çizeceği bir yer olmaz.
- `-v "$PWD":/work` geçerli klasörü çalışma dizini (`/work`) olarak bağlar. Bir depoyu oradan açın ya da `/work` altına birkaç depo bağlayın.
- `-v gitpoulpi-config:/config`, ayarlarınızı ve oturumlarınızı çalıştırmalar arasında koruyan adlandırılmış bir birimdir.
- `--user "$(id -u):$(id -g)"` sizin olarak çalışır; böylece GitPoulpi'nin depolarınızda oluşturduğu dosyalar `root` yerine ana makinedeki sahibinizi korur. İmajın yazdığı her klasör herkese yazılabilirdir, bu yüzden her kullanıcı kimliği çalışır.

## Birimler ve yollar

| Yol | Ne tutar |
|---|---|
| `/work` | Çalışma dizini. Depolarınızı buraya bağlayın. |
| `/config` | Ayarlar (`config.toml`), oturumlar ve alt klasörler olarak veri ve önbellek (`GITPOULPI_CONFIG_DIR=/config`). Korumak için adlandırılmış bir birim veya bir bağlama kullanın. |
| `/data`, `/cache` | XDG değişkenlerini izleyen araçlar için bunlar da birim olarak bildirilmiştir (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. Bir birim değildir: her kapsayıcı için yenidir. |

`/config` için bir birim olmadan her `--rm` çalıştırması varsayılan ayarlarla başlar. Kendi `config.toml` dosyanızla başlamak için onu bağlayın: `-v "$PWD/config.toml":/config/config.toml:ro` (salt okunur, Ayarların kaydedemeyeceği anlamına gelir).

## Başka bir kullanıcıya ait depolar

Git, sahibi onu çalıştıran kullanıcıdan farklı olan bir depoyu reddeder ("dubious ownership"). Yukarıdaki gibi `--user` ile sahip eşleşir. Eşleşmediğinde (başka bir hesaptan bir bağlama, bir Docker Desktop paylaşımı) her dizine şununla güvenin:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

Giriş noktası bu durumda arayüzü başlatmadan önce `git config --global --add safe.directory '*'` komutunu çalıştırır. Yalnızca güvendiğiniz klasörler için kullanın; çünkü git'in korumasını tüm kapsayıcı için kaldırır.

## SSH

Anahtarları kapsayıcıya kopyalamak yerine SSH ajanınızı iletin.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) ana makinenin ajanını kendi sanal makinesinin içinde sabit bir yolda sunar:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

`--user` ile soketin kullanıcı kimliğiniz tarafından okunabilir olması gerekir; size aitse öyledir. Kapsayıcıda `known_hosts` yoktur: bir sunucuya ilk bağlantı anahtarını kabul etmenizi ister, ya da kendi dosyanızı `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro` ile bağlayın.

## Git kimliği ve kimlik bilgileri

- **Kimlik ve ayarlar:** Git yapılandırmanızı salt okunur bağlayın: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Ya da commit kimliğini `/config` içinde duran bir GitPoulpi profilinde ayarlayın (Ayarlar > Profiller).
- **HTTPS kimlik bilgileri:** imajda bir anahtarlık gibi bir kimlik bilgisi yardımcısı yoktur. Ya bir kimlik bilgisi deposu bağlayın (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, bağlanan `.gitconfig` içinde `credential.helper = store` ile), ya da ajanı ileterek bir SSH uzak deposu kullanın.
- **Barındırma token'ları** (pull request'ler, issue'lar): kimlik bilgisi yardımcısı olmadan GitPoulpi bir token'ı yalnızca oturum boyunca bellekte tutar. Kalıcı bir kurulum için onu ortam üzerinden `-e GITPOULPI_TOKEN_<ID>=...` ile geçirin; burada `<ID>`, entegrasyonun kimliğinin büyük harfli ve alfasayısal olmayanları `_` yapılmış halidir (bkz. [Barındırma ve issue takipçisi entegrasyonları](hosting-integration.md)). Entegrasyonların kendisi `/config` altındaki `config.toml` içindedir.

## Terminal, renkler ve fare

Renkler ve fare desteği terminal öykünücünüzden gelir. Docker `TERM` değerini `-t` ile iletir; renkler ya da gerçek renkli tema yanlış göründüğünde `-e TERM -e COLORTERM` ekleyin, örneğin `-e COLORTERM=truecolor`. Fare tıklamaları, sürüklemeler ve tekerlek terminal üzerinden her zamanki gibi gelir. Commit grafiği karakterlerle çizilir; resimler (kitty, iTerm2, sixel) her terminal ve çoklayıcı üzerinden iletilmez, bkz. [Terminal seçmek](terminals.md). Terminal penceresini yeniden boyutlandırmak arayüzü yeniden boyutlandırır: kapsayıcı boyut değişikliğini alır.

## CLI ve MCP

Giriş noktası arayüzü başlatır. Diğer programlara onu geçersiz kılarak ulaşılır:

```
# betiklenebilir CLI (betikler için --json ekleyin)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# standart girdi ve çıktı üzerinden MCP sunucusu, onu başlatan bir ajan için
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Her `docker run`, kendi GitPoulpi arka plan daemon'ına sahip kendi kapsayıcısıdır; bu yüzden CLI üzerinden yapılan değişiklikler başka bir kapsayıcıda çalışan bir arayüzde canlı görünmez. Bir arayüzü CLI'dan yönetmek için ikisini aynı kapsayıcıda çalıştırın: arayüzü `--name gp` ile başlatın ve `docker exec gp gitpoulpi ...` kullanın. Bir ajan için `docker run -i --rm --entrypoint gitpoulpi-mcp ...` komutunu MCP sunucu komutu olarak kaydedin, bkz. [MCP ve ajanlar](mcp-and-agents.md); bağlanan `config.toml` içindeki MCP izin anahtarları her zamanki gibi uygulanır.

## Etiketler ve güncelleme

| Etiket | Anlamı |
|---|---|
| `X.Y.Z` | Tam olarak bir sürüm. |
| `X.Y` | O alt sürümün en son yaması (yalnızca kararlı sürümler). |
| `latest` | En son kararlı sürüm. |

Bir ön sürüm yalnızca kendi tam `X.Y.Z-...` etiketini alır.

Bir imaj değiştirilir, asla yerinde güncellenmez. Kapsayıcı içindeki GitPoulpi bunu bilir (kurulum kaydı "bir Docker imajı" der): [güncelleme paneli](updates.md) sürümü ve komutları gösterir ve hiçbir şeyi değiştirmez. Güncellemek için:

```
docker pull poulpilab/gitpoulpi
```

Sonra yeni bir kapsayıcı başlatın. Ayarlarınız `/config` biriminde kalır. Geri dönmek için daha eski bir etiket çalıştırın: `docker pull poulpilab/gitpoulpi:<önceki>`. Betiklerde bir sürümü sabitlemek için `latest` yerine tam etiketi kullanın. Bir kuruluş imajı kendi registry'sinde yansıtabilir, bkz. [Kurumsal dağıtım](enterprise-distribution.md#docker-registry-yansısı).
