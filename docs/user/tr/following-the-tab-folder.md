[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · **Türkçe** · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Etkin sekmenin klasörünü izlemek

GitPoulpi'de birden çok depo sekme olarak açık olabilir. Onu başlattığınız kabuk
kendi klasöründe kalır: bir program, üst kabuğunun klasörünü değiştiremez.
GitPoulpi, etkin sekmenin klasöründe bir kabuk elde etmek için iki yol sunar:

- **Çık ve cd yap**: GitPoulpi kapanır ve kabuğunuz etkin sekmenin klasörüne
  geçer. Bunun için `gitpoulpi shell-init` tarafından yazdırılan küçük bir kabuk
  işlevi gerekir.
- **Şurada aç**: etkin sekmenin klasörünü bir terminalde, bir çoklayıcı
  bölmesinde, bir düzenleyicide veya dosya yöneticisinde açar. Kabuk kurulumu
  gerekmez. Bkz. [Şurada aç](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Şurada aç ▾ ── ⌕ Genel arama ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Çık ve cd yap            │
```

- **Şurada aç**, başlık çubuğunda depo yolunun ardından gelir. Pencere çok dar
  olduğunda gizlenir.
- **Çık ve cd yap**, sekme çubuğunun sağ ucundadır. Yalnızca ikisi de doğru
  olduğunda görünür:
  - GitPoulpi kabuk işlevi (aşağıda) aracılığıyla başlatılmıştır.
  - Etkin sekme, GitPoulpi'yi başlattığınız klasör değildir.
- Komutlar her zaman palettedir (`Ctrl+P`): `Sekme: Çık ve cd Yap`,
  `Sekme: Şurada Aç...` (`Alt+O`) ve `Sekme: Varsayılanda Aç` (`Ctrl+Alt+O`).
  Kabuk işlevi olmadan `Sekme: Çık ve cd Yap` komutu, nasıl kurulacağını söyler.

Normal bir çıkış (`q`) kabuğunuzun klasörünü asla değiştirmez. `auto` seçeneğini
seçmediğiniz sürece (aşağıda) yalnızca **Çık ve cd yap** değiştirir.

## Kabuk işlevini kurmak

`gitpoulpi shell-init <kabuk>`, `gp` adlı bir işlev yazdırır. Başka bir ad seçmek
için `--name` kullanın. İşlev `gitpoulpi --cwd-file <geçici dosya> ...` komutunu
çalıştırır. **Çık ve cd yap** ile çıktığınızda GitPoulpi klasörü bu dosyaya yazar
ve işlev o klasöre geçer. GitPoulpi hiçbir şey kurmaz ya da düzenlemez: kabuğunuzun
başlangıç dosyasına tek bir satır eklersiniz.

| Kabuk | Şuraya ekleyin | Satır |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | `$PROFILE` dosyanız (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Ardından GitPoulpi'yi `gitpoulpi` yerine `gp` ile başlatın. Bağımsız değişkenler
iletilir (`gp -s work`) ve çıkış durumu `gitpoulpi`'nin çıkış durumudur.

**Var olan bir `gp` uyarı verilmeden asla değiştirilmez.** Kabuğunuzda zaten `gp`
adlı bir komut, takma ad veya işlev varsa betik bir uyarı yazdırır ve hiçbir şey
tanımlamaz. Başka bir ad seçin, örneğin
`gitpoulpi shell-init bash --name gpl`. PowerShell'de işlev, yerleşik `gp` takma
adının (`Get-ItemProperty`) ve `gitpoulpi`'yi gösteren bir takma adın yerini alır.
Başka her `gp` korunur ve bir uyarı verilir.

Her `gitpoulpi` komutu `--cwd-file` kabul eder, ancak dosyayı yalnızca TUI yazar.

## Ayar: Etkin sekmenin klasörünü izle

Ayarlar > Şurada aç > **Etkin sekmenin klasörünü izle** (`ui.tab_dir_follow`):

| Değer | Davranış |
|---|---|
| `buttons` (varsayılan) | Yukarıda anlatıldığı gibi **Çık ve cd yap** ve **Şurada aç**. |
| `auto` | **Çık ve cd yap** düğmesi yok. **Şurada aç** kalır. Her çıkış kabuğunuzu etkin sekmenin klasörüne geçirir (GitPoulpi'yi kabuk işleviyle başlattıysanız). Varsayılan **Şurada aç** girişi, yeni bir sekme açtığınız her seferde çalışır. |

`auto` kipinde varsayılan giriş yalnızca kendi açtığınız sekmeler için çalışır:
Ana sekmeden, bir worktree'den, bir klondan veya bir init'ten. Başlangıçta geri
yüklenen sekmeler için, ilk sekme için ya da zaten açık bir sekmeye geçtiğinizde
çalışmaz.

Eski `ui.terminal_launch` ve `ui.terminal_launch_command` ayarları **Şurada aç**
listesine taşınmıştır. Bkz. [Şurada aç](open-in.md#eski-anahtarlar).
