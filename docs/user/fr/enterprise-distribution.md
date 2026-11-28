[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · **Français** · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Distribution en entreprise

Pour les administrateurs qui déploient GitPoulpi sur de nombreux ordinateurs depuis leurs propres serveurs : flux de paquets internes, miroirs, une politique de mise à jour valable pour toute la machine, et une configuration par défaut. Lisez d'abord [Installation](installation.md) et [Mises à jour](updates.md).

Deux faits commandent tout ce qui suit :

- Chaque installation enregistre **comment elle a été installée** dans le marqueur `.gitpoulpi-dist` à côté des programmes (`channel=...`). GitPoulpi ne se met à jour que de la façon que ce canal permet : il ne remplace jamais des fichiers qu'un gestionnaire de paquets possède, et un MSI n'est jamais touché.
- La **politique** (un fichier, et sous Windows le registre) vous appartient. Le `config.toml` d'un utilisateur ne peut jamais l'assouplir.

## Chocolatey, avec un flux privé

Le paquet public télécharge l'installeur depuis GitHub. Pour un réseau fermé, internalisez-le :

1. Récupérez le paquet (`choco download gitpoulpi --internalize` avec l'outillage Chocolatey sous licence, ou téléchargez le `.nupkg` et dépaquetez-le).
2. Dans `tools\chocolateyinstall.ps1`, faites pointer `url64bit` vers votre propre copie de `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` et gardez `checksum64` égal au premier champ du fichier `.sha256` de la version (ou à la somme de contrôle de votre copie).
3. `choco pack`, puis poussez le `.nupkg` vers votre flux (Nexus, Artifactory, ProGet, un partage de fichiers).

Installez-le avec des paramètres de paquet :

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Paramètre | Effet |
|---|---|
| `/Channel:choco-internal` | Enregistre `channel=choco-internal` dans le marqueur, pour que GitPoulpi sache que le flux est le vôtre et non le flux communautaire de Chocolatey. |
| `/Feed:<url>` | Enregistre l'adresse du flux (`feed=` dans le marqueur). GitPoulpi y cherche les nouvelles versions et lance `choco upgrade gitpoulpi -y --source=<feed>`. |

Sans paramètres, le canal est `choco` et la recherche va au flux communautaire de Chocolatey. Le paquet s'installe pour tous les utilisateurs dans Program Files, sans alias `gp`.

La mise à jour de GitPoulpi lance `choco upgrade` sans `--params`, pour que rien de ce que vous avez déployé ne soit touché. Chocolatey doit donc se souvenir des paramètres de paquet de la première installation (`useRememberedArgumentsForUpgrades`, activé par défaut depuis Chocolatey 2.0 ; sur les versions plus anciennes, `choco feature enable -n useRememberedArgumentsForUpgrades`). Sinon, la mise à jour enregistrerait le simple canal `choco`. Un `check_url` de politique (plus bas) maintient la recherche et la mise à jour orientées vers votre flux quoi que dise le marqueur, et il l'emporte sur le flux du marqueur.

**Rien dans le paquet n'écrit `policy.toml` ni `config.toml`.** Ce que vous avez déployé reste, à l'installation comme à chaque mise à jour. Voir [Déployer un config.toml par défaut](#déployer-un-configtoml-par-défaut).

## Scoop, avec un bucket privé

Placez `gitpoulpi.json` (le manifeste du bucket public) dans `bucket/` d'un dépôt Git que vous hébergez, changez `url` pour votre copie du zip de la version et réglez `hash` sur son sha256 (`autoupdate` peut rester si votre copie suit les noms des versions). Puis :

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

Le `post_install` du manifeste réécrit le marqueur avec `channel=scoop` ; GitPoulpi lance alors lui-même `scoop update gitpoulpi` et ne remplace jamais des fichiers dans le dos de Scoop. Scoop installe par utilisateur et n'a pas besoin de droits d'administrateur.

## apt, avec un miroir

Le dépôt est une simple arborescence de dossiers (`dists/`, `pool/`, les fichiers `gitpoulpi-archive-keyring.gpg` et `.asc` de la clé de signature, et l'exemple `gitpoulpi.sources`) servie en HTTPS. Miroitez-la avec `wget --mirror`, `rsync` ou votre gestionnaire d'artefacts, servez-la en interne, puis installez la clé et la source sur chaque ordinateur :

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources` :

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

Gardez le nom de fichier `gitpoulpi.sources` (ou `gitpoulpi.list`) : GitPoulpi n'affiche des commandes `apt` que s'il trouve `/etc/apt/sources.list.d/gitpoulpi.*`. Utilisez la clé du projet : le miroir reste signé par elle, et `Signed-By` fixe cette seule clé pour ce dépôt uniquement. Le dépôt conserve les cinq dernières versions de chaque paquet.

**`.deb` hors ligne :** chaque version joint `gitpoulpi_<version>_<arch>.deb`. Copiez-le puis `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. Le marqueur indique `channel=apt` ; sans le fichier de source, GitPoulpi dit à l'utilisateur de télécharger le `.deb` suivant au lieu d'afficher une commande `apt`.

## MSI, avec Intune ou une stratégie de groupe

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` s'installe pour tous les utilisateurs et est conçu pour être géré : le panneau de GitPoulpi indique seulement « géré par votre administrateur » (avec votre `message`), il ne met jamais à jour une installation MSI. Mettez à jour en déployant le nouveau MSI ; c'est une mise à niveau majeure, l'ancienne version est donc remplacée.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Propriété | Effet |
|---|---|
| `INSTALLFOLDER` | Dossier d'installation (par défaut `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (par défaut) ajoute le dossier au PATH de la machine, `0` ne l'ajoute pas. |
| `UPDATES_MODE` | `mode` de la politique : `off`, `notify` ou `auto`. |
| `UPDATES_PIN` | `pin` de la politique, par exemple `0.2` ou `0.2.4`. |
| `UPDATES_CHECK` | `check` de la politique : `github`, `nuget`, `manifest` ou `none`. |
| `UPDATES_CHECK_URL` | `check_url` de la politique. |
| `UPDATES_MESSAGE` | `message` de la politique, affiché dans le panneau de mise à jour. |

Une propriété n'écrit sa valeur dans `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` que si vous la définissez : une propriété absente laisse le registre (et un fichier de politique) tranquilles. **Intune :** déployez le MSI comme application métier ou Win32 avec la commande ci-dessus. **Stratégie de groupe :** l'installation de logiciels ne peut pas passer de propriétés, utilisez donc une transformation (`.mst`) ou définissez les valeurs de registre avec les Préférences de stratégie de groupe, voir plus bas.

## Homebrew, avec un tap interne

Copiez `gitpoulpi.rb` du tap public dans `Formula/` d'un dépôt Git que vous hébergez, changez les lignes `url` pour vos copies des archives de la version et mettez à jour chaque `sha256` (le premier champ du fichier `.sha256` à côté de chaque archive). Puis :

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

La formule installe le marqueur avec `channel=homebrew` : GitPoulpi lance lui-même `brew upgrade gitpoulpi`, qui lit votre tap.

## Miroir de registre Docker

Le nom de l'image est `poulpilab/gitpoulpi`. Le panneau affiche toujours ce nom, un miroir doit donc être transparent.

- **Cache d'extraction :** lancez un registre comme proxy de Docker Hub (l'image `registry` avec `proxy.remoteurl: https://registry-1.docker.io`) et définissez `"registry-mirrors": ["https://registry.example"]` dans le `daemon.json` du démon Docker. `docker pull poulpilab/gitpoulpi` passe alors par votre cache.
- **Copie :** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, puis lancez ce nom. Le panneau affiche toujours le nom public, ce qui convient comme repère de la version.

Voir [Exécuter dans Docker](docker.md) pour la façon de le lancer.

## Le fichier de politique et le registre

La politique définit le comportement des mises à jour sur un ordinateur et ne peut pas être modifiée par les utilisateurs. Elle est lue depuis :

| OS | Fichier de politique |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, aussi | la clé de registre `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, une valeur `REG_SZ` par clé ci-dessous, avec les mêmes noms (pour la stratégie de groupe, Intune et les propriétés MSI) |

Le fichier a une seule table :

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

| Clé | Valeurs | Effet |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Un plafond : `off` < `notify` < `auto`. Les utilisateurs peuvent choisir une valeur plus basse, jamais plus haute. `off` masque la recherche et ne fait aucune requête. |
| `release_channel` | `stable`, `prerelease` | Impose le canal de mise à jour. |
| `pin` | `1.4` ou `1.4.2` | La version la plus haute proposée : `1.4` autorise toute `1.4.x`, `1.4.2` autorise au plus `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Où l'on demande quelle est la dernière version. Par défaut selon l'installation : GitHub, le flux Chocolatey pour les canaux Chocolatey. `none` ne cherche jamais. |
| `check_url` | URL | Avec `nuget` : le flux. Avec `manifest` : l'adresse de `latest.json`. Avec `github` : la base de l'API GitHub, pour GitHub Enterprise Server (ajoutez `repo`). |
| `repo` | `owner/name` | Le dépôt interrogé avec `check = "github"`. |
| `download_base` | URL | Un miroir des fichiers de la version (mêmes noms de fichiers) pour les canaux archive et installeur. Les téléchargements depuis cet hôte sont autorisés en plus de ceux de GitHub. |
| `allow_self_update` | `true`, `false` | `false` transforme chaque action « remplacer les fichiers de GitPoulpi » et « lancer le gestionnaire de paquets » en une note ou une commande à copier. Par défaut `true`. |
| `message` | texte | Affiché dans le panneau de mise à jour (« De la part de votre administrateur : ... »). |

**Priorité**, de la plus forte à la plus faible : le registre, le fichier de politique, le `[updates]` du `config.toml` de l'utilisateur, le `feed=` du marqueur, les valeurs par défaut intégrées. Une clé définie par la politique est **Défini par votre administrateur** dans les Paramètres et ne peut pas y être modifiée.

**Échec par précaution :** un fichier de politique ou une valeur de registre illisible (une erreur de syntaxe TOML, `mode = "sometimes"`, un `pin` qui n'est pas une version) est traité comme `mode = notify` et `allow_self_update = false`, et une ligne va dans le journal de l'application. Une faute de frappe ne laisse jamais un parc se mettre à jour tout seul. Une clé inconnue est ignorée avec un avertissement. Sous Windows, l'étape qui détient les droits d'administrateur lit seulement la politique de la machine, jamais la configuration de l'utilisateur.

### Exemples

Désactiver les mises à jour (une image verrouillée, un réseau isolé) :

```toml
[updates]
mode = "off"
```

Figer le parc sur la ligne 1.4, en notification seulement, et dire à qui s'adresser :

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Interroger un flux Chocolatey (NuGet) interne :

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Le même genre de politique sous Windows via le registre (PowerShell) :

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json et download_base, un miroir de la version

Copiez les fichiers d'une version (les archives, le programme d'installation, les fichiers `.sha256`, `latest.json`, **chaque fichier de signature `.minisig`** et `security.json`) dans un même dossier d'un serveur web interne, en gardant les noms. `latest.json` est joint à chaque version :

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

La politique est alors :

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Les canaux archive et installeur consultent le manifeste et téléchargent depuis `download_base`, avec la même vérification que depuis GitHub. Un miroir n'a qu'à copier des fichiers ; il n'a besoin d'aucune API.

Les signatures voyagent avec les fichiers, un miroir ne peut donc pas les affaiblir :

- `latest.json` n'est approuvé que si `latest.json.minisig` se trouve à côté et se vérifie avec la clé du projet intégrée à GitPoulpi. Un manifeste sans sa signature, ou modifié après sa signature, est refusé (les hashs qu'il contient sont ceux auxquels les téléchargements sont comparés).
- Chaque archive est téléchargée avec son `<archive>.minisig` depuis le même hôte et n'est installée que si cette signature se vérifie et désigne ce fichier et cette version. Un miroir qui a copié l'archive mais pas son `.minisig` fait échouer la mise à jour avec « no signature », et rien n'est modifié.
- `security.json` (les avis de sécurité, voir [Mises à jour](updates.md#avis-de-sécurité)) et `security.json.minisig` sont cherchés à côté du manifeste. Un miroir qui ne les copie pas n'affiche simplement aucun avis.

Copiez les fichiers sans les modifier : une signature couvre les octets exacts. Les avis de sécurité sont republiés pour la dernière version sans nouvelle compilation ; actualisez donc `latest.json`, `security.json` et leurs fichiers `.minisig` depuis la page de la version chaque fois que vous actualisez le miroir.

## Déployer un config.toml par défaut

GitPoulpi n'a pas de `config.toml` valable pour toute la machine : les paramètres vivent dans le dossier de chaque utilisateur (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Pour donner vos valeurs par défaut aux nouveaux utilisateurs, copiez un `config.toml` minimal (seulement les clés voulues, voir la [Référence de configuration](config-reference.md)) dans ce dossier quand l'utilisateur n'en a pas (les valeurs par défaut d'interface comme `ui.theme` ou `ui.language` vont dans un `tui.toml` minimal à côté, que l'interface terminal lit toute seule ; voir [Deux fichiers](config-reference.md#deux-fichiers--configtoml-et-tuitoml)) : un script d'ouverture de session ou Intune sous Windows, `/etc/skel/.config/gitpoulpi/` sous Linux, un script de première connexion sous macOS. Rien dans les paquets de GitPoulpi n'écrase jamais une configuration, elle vous appartient donc. N'y mettez pas de règles de mise à jour quand elles doivent tenir : utilisez la politique, qu'un utilisateur ne peut pas contourner. Une configuration portable ou propre à une tâche peut aussi être choisie avec `GITPOULPI_CONFIG_DIR`.
