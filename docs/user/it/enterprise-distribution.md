[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · **Italiano** · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Distribuzione aziendale

Per gli amministratori che distribuiscono GitPoulpi su molti computer dai propri server: feed di pacchetti interni, mirror, una policy di aggiornamento valida per tutta la macchina e una configurazione predefinita. Leggi prima [Installazione](installation.md) e [Aggiornamenti](updates.md).

Due fatti guidano tutto ciò che segue:

- Ogni installazione registra **come è stata installata** nel marcatore `.gitpoulpi-dist` accanto ai programmi (`channel=...`). GitPoulpi si aggiorna da sé solo nel modo consentito da quel canale: non sostituisce mai i file di cui un gestore di pacchetti è proprietario, e un MSI non viene mai toccato.
- La **policy** (un file, e su Windows il registro di sistema) è tua. Il `config.toml` di un utente non può mai allentarla.

## Chocolatey, con un feed privato

Il pacchetto pubblico scarica il programma di installazione da GitHub. Per una rete chiusa, internalizzalo:

1. Ottieni il pacchetto (`choco download gitpoulpi --internalize` con gli strumenti Chocolatey con licenza, oppure scarica il `.nupkg` e decomprimilo).
2. In `tools\chocolateyinstall.ps1` punta `url64bit` alla tua copia di `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` e mantieni `checksum64` uguale al primo campo del file `.sha256` della release (o al checksum della tua copia).
3. `choco pack`, poi invia il `.nupkg` al tuo feed (Nexus, Artifactory, ProGet, una condivisione di file).

Installalo con i parametri del pacchetto:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parametro | Effetto |
|---|---|
| `/Channel:choco-internal` | Registra `channel=choco-internal` nel marcatore, così GitPoulpi sa che il feed è tuo e non il feed community di Chocolatey. |
| `/Feed:<url>` | Registra l'indirizzo del feed (`feed=` nel marcatore). GitPoulpi lo controlla per le nuove versioni ed esegue `choco upgrade gitpoulpi -y --source=<feed>`. |

Senza parametri il canale è `choco` e il controllo va al feed community di Chocolatey. Il pacchetto si installa per tutti gli utenti in Program Files, senza l'alias `gp`.

L'aggiornamento di GitPoulpi esegue `choco upgrade` senza `--params`, in modo che nulla di ciò che hai distribuito venga toccato. Chocolatey deve quindi ricordare i parametri del pacchetto della prima installazione (`useRememberedArgumentsForUpgrades`, attivo per impostazione predefinita da Chocolatey 2.0; nelle versioni più vecchie `choco feature enable -n useRememberedArgumentsForUpgrades`). Altrimenti l'aggiornamento registrerebbe il semplice canale `choco`. Un `check_url` della policy (sotto) mantiene il controllo e l'aggiornamento puntati al tuo feed qualunque cosa dica il marcatore, e ha la precedenza sul feed del marcatore.

**Nulla nel pacchetto scrive `policy.toml` o `config.toml`.** Ciò che hai distribuito resta, all'installazione e a ogni aggiornamento. Vedi [Distribuire un config.toml predefinito](#distribuire-un-configtoml-predefinito).

## Scoop, con un bucket privato

Metti `gitpoulpi.json` (il manifest del bucket pubblico) in `bucket/` di un repository Git che ospiti, cambia `url` con la tua copia dello zip della release e imposta `hash` al suo sha256 (`autoupdate` può restare se la tua copia segue i nomi delle release). Poi:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

Il `post_install` del manifest riscrive il marcatore con `channel=scoop`; GitPoulpi esegue allora da sé `scoop update gitpoulpi` e non sostituisce mai i file alle spalle di Scoop. Scoop installa per utente e non richiede diritti di amministratore.

## apt, con un mirror

Il repository è un semplice albero di directory (`dists/`, `pool/`, i file `gitpoulpi-archive-keyring.gpg` e `.asc` della chiave di firma, e l'esempio `gitpoulpi.sources`) servito via HTTPS. Replicalo con `wget --mirror`, `rsync` o il tuo gestore di artefatti, servilo internamente e installa la chiave e la fonte su ogni computer:

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`:

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

Mantieni il nome file `gitpoulpi.sources` (o `gitpoulpi.list`): GitPoulpi mostra i comandi `apt` solo quando trova `/etc/apt/sources.list.d/gitpoulpi.*`. Usa la chiave del progetto: il mirror resta firmato da essa, e `Signed-By` fissa quella sola chiave per questo repository. Il repository conserva le ultime cinque versioni di ogni pacchetto.

**`.deb` offline:** ogni release allega `gitpoulpi_<version>_<arch>.deb`. Copialo e `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. Il marcatore dice `channel=apt`; senza il file della fonte GitPoulpi dice all'utente di scaricare invece il `.deb` successivo, anziché mostrare un comando `apt`.

## MSI, con Intune o Criteri di gruppo

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` si installa per tutti gli utenti ed è pensato per essere gestito: il pannello di GitPoulpi dice solo «gestita dal tuo amministratore» (con il tuo `message`), e non aggiorna mai un'installazione MSI. Aggiorna distribuendo il nuovo MSI; è un major upgrade, quindi la vecchia versione viene sostituita.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Proprietà | Effetto |
|---|---|
| `INSTALLFOLDER` | Cartella di installazione (predefinita `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (predefinito) aggiunge la cartella al PATH della macchina, `0` no. |
| `UPDATES_MODE` | `mode` della policy: `off`, `notify` o `auto`. |
| `UPDATES_PIN` | `pin` della policy, ad esempio `0.2` o `0.2.4`. |
| `UPDATES_CHECK` | `check` della policy: `github`, `nuget`, `manifest` o `none`. |
| `UPDATES_CHECK_URL` | `check_url` della policy. |
| `UPDATES_MESSAGE` | `message` della policy, mostrato nel pannello degli aggiornamenti. |

Una proprietà scrive il suo valore in `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` solo quando la imposti, quindi una non impostata lascia in pace il registro (e un file di policy). **Intune:** distribuisci l'MSI come app line-of-business o Win32 con il comando sopra. **Criteri di gruppo:** l'installazione del software non può passare proprietà, quindi usa una trasformazione (`.mst`) oppure imposta i valori del registro con le Preferenze dei Criteri di gruppo, vedi sotto.

## Homebrew, con un tap interno

Copia `gitpoulpi.rb` dal tap pubblico in `Formula/` di un repository Git che ospiti, cambia le righe `url` con le tue copie degli archivi di rilascio e aggiorna ogni `sha256` (il primo campo del file `.sha256` accanto a ogni archivio). Poi:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

La formula installa il marcatore con `channel=homebrew`: GitPoulpi esegue da sé `brew upgrade gitpoulpi`, che legge il tuo tap.

## Mirror del registry Docker

Il nome dell'immagine è `poulpilab/gitpoulpi`. Il pannello mostra sempre quel nome, quindi un mirror deve essere trasparente.

- **Cache pull-through:** esegui un registry come proxy di Docker Hub (l'immagine `registry` con `proxy.remoteurl: https://registry-1.docker.io`) e imposta `"registry-mirrors": ["https://registry.example"]` nel `daemon.json` del daemon Docker. `docker pull poulpilab/gitpoulpi` passa allora per la tua cache.
- **Copia:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, poi esegui quel nome. Il pannello mostra ancora quello pubblico, il che va bene come indicazione della versione.

Vedi [Eseguire in Docker](docker.md) per come eseguirla.

## Il file di policy e il registro di sistema

La policy stabilisce come si comportano gli aggiornamenti su un computer e non può essere modificata dagli utenti. Viene letta da:

| SO | File di policy |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, anche | la chiave di registro `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, un valore `REG_SZ` per ogni chiave qui sotto, con gli stessi nomi (per Criteri di gruppo, Intune e proprietà MSI) |

Il file ha una sola tabella:

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
```

| Chiave | Valori | Effetto |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Un tetto: `off` < `notify` < `auto`. Gli utenti possono scegliere un valore più basso, mai più alto. `off` nasconde il controllo e non fa alcuna richiesta. |
| `release_channel` | `stable`, `prerelease` | Forza il canale di aggiornamento. |
| `pin` | `1.4` o `1.4.2` | La versione più alta offerta: `1.4` consente qualsiasi `1.4.x`, `1.4.2` consente al massimo `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Dove si chiede «qual è l'ultima release». Predefinito per installazione: GitHub, il feed Chocolatey per i canali Chocolatey. `none` non controlla mai. |
| `check_url` | URL | Con `nuget`: il feed. Con `manifest`: l'indirizzo di `latest.json`. Con `github`: la base dell'API GitHub, per GitHub Enterprise Server (aggiungi `repo`). |
| `repo` | `owner/name` | Il repository controllato con `check = "github"`. |
| `download_base` | URL | Un mirror dei file di release (stessi nomi di file) per i canali archivio e programma di installazione. I download da questo host sono consentiti oltre a quelli di GitHub. |
| `allow_self_update` | `true`, `false` | `false` trasforma ogni azione «sostituisci i file di GitPoulpi» ed «esegui il gestore di pacchetti» in una nota o in un comando da copiare. Predefinito `true`. |
| `message` | testo | Mostrato nel pannello degli aggiornamenti («Dal tuo amministratore: ...»). |

**Precedenza**, dalla più forte: il registro di sistema, il file di policy, `[updates]` del `config.toml` dell'utente, il `feed=` del marcatore, i valori predefiniti incorporati. Una chiave impostata dalla policy è **Impostato dal tuo amministratore** nelle Impostazioni e non può essere modificata lì.

**Fallire in modo sicuro:** un file di policy o un valore del registro che non si può leggere (un errore di sintassi TOML, `mode = "sometimes"`, un `pin` che non è una versione) viene trattato come `mode = notify` e `allow_self_update = false`, e una riga va nel log dell'applicazione. Un errore di battitura non lascia mai una flotta ad aggiornarsi da sola. Una chiave sconosciuta viene ignorata con un avviso. Su Windows il passaggio che detiene i diritti di amministratore legge solo la policy della macchina, mai la configurazione dell'utente.

### Esempi

Disattivare gli aggiornamenti (un'immagine bloccata, una rete isolata):

```toml
[updates]
mode = "off"
```

Fissare la flotta alla linea 1.4, solo notifica, e dire a chi rivolgersi:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Controllare un feed Chocolatey (NuGet) interno:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Lo stesso tipo di policy su Windows tramite il registro di sistema (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json e download_base: un mirror della release

Copia i file di una release (gli archivi, il programma di installazione, i file `.sha256`, `latest.json`, **ogni file di firma `.minisig`** e `security.json`) in una cartella di un server web interno, mantenendo i nomi. `latest.json` è allegato a ogni release:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Allora la policy è:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

I canali archivio e programma di installazione controllano il manifest e scaricano da `download_base`, con la stessa verifica che da GitHub. Un mirror deve solo copiare file; non ha bisogno di alcuna API.

Le firme viaggiano con i file, quindi un mirror non può indebolirle:

- `latest.json` è considerato attendibile solo quando `latest.json.minisig` si trova accanto e si verifica rispetto alla chiave del progetto incorporata in GitPoulpi. Un manifest senza la sua firma, o modificato dopo la firma, viene rifiutato (gli hash al suo interno sono ciò rispetto a cui vengono verificati i download).
- Ogni archivio viene scaricato insieme a `<archive>.minisig` dallo stesso host e viene installato solo quando quella firma si verifica e nomina questo file e questa versione. Un mirror che ha copiato l'archivio ma non il suo `.minisig` fa fallire l'aggiornamento con «no signature», e nulla viene cambiato.
- `security.json` (gli avvisi di sicurezza, vedi [Aggiornamenti](updates.md#avvisi-di-sicurezza)) e `security.json.minisig` vengono cercati accanto al manifest. Un mirror che non li copia semplicemente non mostra avvisi.

Copia i file senza modificarli: una firma copre i byte esatti. Gli avvisi di sicurezza vengono ripubblicati per l'ultima release senza una nuova build, quindi aggiorna `latest.json`, `security.json` e i loro file `.minisig` dalla pagina della release ogni volta che aggiorni il mirror.

## Distribuire un config.toml predefinito

GitPoulpi non ha un `config.toml` valido per tutta la macchina: le impostazioni vivono nella cartella di ogni utente (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Per dare ai nuovi utenti i tuoi valori predefiniti, copia un `config.toml` essenziale (solo le chiavi che vuoi, vedi il [riferimento della configurazione](config-reference.md)) in quella cartella quando l'utente non ne ha uno: uno script di accesso o di Intune su Windows, `/etc/skel/.config/gitpoulpi/` su Linux, uno script di primo accesso su macOS. Nulla nei pacchetti di GitPoulpi sovrascrive mai una configurazione, quindi la gestisci tu. Non metterci regole di aggiornamento quando devono valere: usa la policy, che un utente non può scavalcare. Una configurazione portabile o per singolo job si può anche scegliere con `GITPOULPI_CONFIG_DIR`.
