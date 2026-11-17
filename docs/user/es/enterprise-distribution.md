[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · **Español** · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Distribución en empresas

Para administradores que despliegan GitPoulpi en muchos equipos desde sus propios servidores: feeds de paquetes internos, espejos, una política de actualización para todo el equipo y una configuración por defecto. Lee antes [Instalación](installation.md) y [Actualizaciones](updates.md).

Dos hechos lo condicionan todo aquí:

- Cada instalación registra **cómo se instaló** en el marcador `.gitpoulpi-dist` junto a los programas (`channel=...`). GitPoulpi se actualiza solo de la forma que permite ese canal: nunca reemplaza archivos que pertenecen a un gestor de paquetes y un MSI nunca se toca.
- La **política** (un archivo y, en Windows, el registro) es tuya. El `config.toml` propio de un usuario nunca puede aflojarla.

## Chocolatey, con un feed privado

El paquete público descarga el instalador de GitHub. Para una red cerrada, internalízalo:

1. Consigue el paquete (`choco download gitpoulpi --internalize` con las herramientas de Chocolatey con licencia, o descarga el `.nupkg` y desempaquétalo).
2. En `tools\chocolateyinstall.ps1`, apunta `url64bit` a tu propia copia de `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` y mantén `checksum64` igual al primer campo del archivo `.sha256` de la versión (o al checksum de tu copia).
3. `choco pack` y luego sube el `.nupkg` a tu feed (Nexus, Artifactory, ProGet, una carpeta compartida).

Instálalo con parámetros del paquete:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parámetro | Efecto |
|---|---|
| `/Channel:choco-internal` | Registra `channel=choco-internal` en el marcador, para que GitPoulpi sepa que el feed es tuyo y no el feed de la comunidad de Chocolatey. |
| `/Feed:<url>` | Registra la dirección del feed (`feed=` en el marcador). GitPoulpi la consulta para buscar versiones nuevas y ejecuta `choco upgrade gitpoulpi -y --source=<feed>`. |

Sin parámetros, el canal es `choco` y la comprobación va al feed de la comunidad de Chocolatey. El paquete se instala para todos los usuarios en Program Files, sin alias `gp`.

La actualización de GitPoulpi ejecuta `choco upgrade` sin `--params`, para no tocar nada de lo que desplegaste. Por eso Chocolatey debe recordar los parámetros del paquete de la primera instalación (`useRememberedArgumentsForUpgrades`, activado por defecto desde Chocolatey 2.0; en versiones anteriores, `choco feature enable -n useRememberedArgumentsForUpgrades`). Si no, la actualización registraría el canal simple `choco`. Un `check_url` de la política (más abajo) mantiene la comprobación y la actualización apuntando a tu feed diga lo que diga el marcador, y tiene prioridad sobre el feed del marcador.

**Nada del paquete escribe `policy.toml` ni `config.toml`.** Lo que desplegaste se queda, al instalar y en cada actualización. Consulta [Desplegar un config.toml por defecto](#desplegar-un-configtoml-por-defecto).

## Scoop, con un bucket privado

Pon `gitpoulpi.json` (el manifiesto del bucket público) en `bucket/` de un repositorio Git que alojes, cambia `url` a tu copia del zip de la versión y pon en `hash` su sha256 (`autoupdate` puede quedarse si tu copia sigue los nombres de las versiones). Luego:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

El `post_install` del manifiesto reescribe el marcador con `channel=scoop`; GitPoulpi ejecuta entonces `scoop update gitpoulpi` por sí mismo y nunca cambia archivos a espaldas de Scoop. Scoop instala por usuario y no necesita derechos de administrador.

## apt, con un espejo

El repositorio es un árbol de directorios simple (`dists/`, `pool/`, el `gitpoulpi-archive-keyring.gpg` y el `.asc` de la clave de firma, y el ejemplo `gitpoulpi.sources`) servido por HTTPS. Replícalo con `wget --mirror`, `rsync` o tu gestor de artefactos, sírvelo internamente e instala la clave y la fuente en cada equipo:

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

Conserva el nombre de archivo `gitpoulpi.sources` (o `gitpoulpi.list`): GitPoulpi solo muestra comandos `apt` cuando encuentra `/etc/apt/sources.list.d/gitpoulpi.*`. Usa la clave del proyecto: el espejo sigue firmado por ella, y `Signed-By` fija esa única clave solo para este repositorio. El repositorio conserva las últimas cinco versiones de cada paquete.

**`.deb` sin conexión:** cada versión adjunta `gitpoulpi_<version>_<arch>.deb`. Cópialo y ejecuta `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. El marcador dice `channel=apt`; sin el archivo de fuente, GitPoulpi indica al usuario que descargue el siguiente `.deb` en lugar de mostrar un comando `apt`.

## MSI, con Intune o directiva de grupo

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` se instala para todos los usuarios y está pensado para gestionarse: el panel de GitPoulpi solo dice «gestionado por tu administrador» (con tu `message`), nunca actualiza una instalación MSI. Actualiza desplegando el nuevo MSI; es una actualización mayor, así que la versión anterior se reemplaza.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Propiedad | Efecto |
|---|---|
| `INSTALLFOLDER` | Carpeta de instalación (por defecto `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (por defecto) añade la carpeta al PATH del equipo, `0` no. |
| `UPDATES_MODE` | `mode` de la política: `off`, `notify` o `auto`. |
| `UPDATES_PIN` | `pin` de la política, por ejemplo `0.2` o `0.2.4`. |
| `UPDATES_CHECK` | `check` de la política: `github`, `nuget`, `manifest` o `none`. |
| `UPDATES_CHECK_URL` | `check_url` de la política. |
| `UPDATES_MESSAGE` | `message` de la política, que se muestra en el panel de actualización. |

Una propiedad escribe su valor en `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` solo cuando la fijas, así que una propiedad sin fijar deja en paz el registro (y un archivo de política). **Intune:** despliega el MSI como aplicación de línea de negocio o Win32 con el comando de arriba. **Directiva de grupo:** la instalación de software no puede pasar propiedades, así que usa una transformación (`.mst`) o fija los valores del registro con las preferencias de directiva de grupo, véase abajo.

## Homebrew, con un tap interno

Copia `gitpoulpi.rb` del tap público en `Formula/` de un repositorio Git que alojes, cambia las líneas `url` a tus copias de los archivos de la versión y actualiza cada `sha256` (el primer campo del archivo `.sha256` junto a cada archivo). Luego:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

La fórmula instala el marcador con `channel=homebrew`: GitPoulpi ejecuta `brew upgrade gitpoulpi` por sí mismo, que lee tu tap.

## Espejo del registro de Docker

El nombre de la imagen es `poulpilab/gitpoulpi`. El panel muestra siempre ese nombre, así que un espejo tiene que ser transparente.

- **Caché de paso (pull-through):** ejecuta un registro como proxy de Docker Hub (la imagen `registry` con `proxy.remoteurl: https://registry-1.docker.io`) y pon `"registry-mirrors": ["https://registry.example"]` en el `daemon.json` del daemon de Docker. `docker pull poulpilab/gitpoulpi` pasa entonces por tu caché.
- **Copia:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>` y luego ejecuta ese nombre. El panel sigue mostrando el público, lo cual está bien como indicación de la versión.

Consulta [Ejecutar en Docker](docker.md) para saber cómo ejecutarla.

## El archivo de política y el registro

La política fija cómo se comportan las actualizaciones en un equipo y los usuarios no pueden cambiarla. Se lee de:

| SO | Archivo de política |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, además | la clave de registro `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, un valor `REG_SZ` por cada clave de abajo, con los mismos nombres (para directiva de grupo, Intune y propiedades MSI) |

El archivo tiene una sola tabla:

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

| Clave | Valores | Efecto |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Un techo: `off` < `notify` < `auto`. Los usuarios pueden elegir un valor menor, nunca mayor. `off` oculta la comprobación y no hace ninguna solicitud. |
| `release_channel` | `stable`, `prerelease` | Fuerza el canal de actualización. |
| `pin` | `1.4` o `1.4.2` | La versión más alta que se ofrece: `1.4` permite cualquier `1.4.x`, `1.4.2` permite como máximo `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Dónde se pregunta «cuál es la última versión». Por defecto según la instalación: GitHub, el feed de Chocolatey para los canales de Chocolatey. `none` nunca comprueba. |
| `check_url` | URL | Con `nuget`: el feed. Con `manifest`: la dirección de `latest.json`. Con `github`: la base de la API de GitHub, para GitHub Enterprise Server (añade `repo`). |
| `repo` | `owner/name` | El repositorio que se comprueba con `check = "github"`. |
| `download_base` | URL | Un espejo de los archivos de la versión (mismos nombres de archivo) para los canales de archivo e instalador. Se permiten las descargas desde este host además de las de GitHub. |
| `allow_self_update` | `true`, `false` | `false` convierte cada acción de «reemplazar los archivos de GitPoulpi» y «ejecutar el gestor de paquetes» en una nota o un comando para copiar. Por defecto `true`. |
| `message` | texto | Se muestra en el panel de actualización («De tu administrador: ...»). |

**Precedencia**, de la más fuerte a la más débil: el registro, el archivo de política, el `[updates]` del `config.toml` del usuario, el `feed=` del marcador, los valores por defecto incorporados. Una clave que fija la política aparece como **Fijado por tu administrador** en Ajustes y no se puede editar ahí.

**Fallar por el lado seguro:** un archivo de política o un valor del registro que no se puede leer (un error de sintaxis TOML, `mode = "sometimes"`, un `pin` que no es una versión) se trata como `mode = notify` y `allow_self_update = false`, y se escribe una línea en el registro de la aplicación. Una errata nunca deja a una flota actualizándose sola. Una clave desconocida se ignora con un aviso. En Windows, el paso que tiene derechos de administrador lee solo la política del equipo, nunca la configuración del usuario.

### Ejemplos

Desactivar las actualizaciones (una imagen bloqueada, una red aislada):

```toml
[updates]
mode = "off"
```

Fijar la flota en la línea 1.4, solo notificar, y decir a quién preguntar:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Comprobar un feed interno de Chocolatey (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

El mismo tipo de política en Windows mediante el registro (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json y download_base: un espejo de la versión

Copia los archivos de una versión (los archivos comprimidos, el instalador, los archivos `.sha256`, `latest.json`, **cada archivo de firma `.minisig`** y `security.json`) en una carpeta de un servidor web interno, conservando los nombres. `latest.json` va adjunto a cada versión:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Entonces la política es:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Los canales de archivo e instalador comprueban el manifiesto y descargan desde `download_base`, con la misma verificación que desde GitHub. Un espejo solo tiene que copiar archivos; no necesita ninguna API.

Las firmas viajan con los archivos, así que un espejo no puede debilitarlas:

- `latest.json` solo es de fiar cuando `latest.json.minisig` está a su lado y se verifica con la clave del proyecto incorporada en GitPoulpi. Un manifiesto sin su firma, o editado después de firmarse, se rechaza (los hashes que contiene son con los que se comprueban las descargas).
- Cada archivo comprimido se descarga junto con `<archive>.minisig` del mismo host y solo se instala cuando esa firma se verifica y nombra este archivo y esta versión. Un espejo que copió el archivo pero no su `.minisig` hace que la actualización falle con «no signature» y no se cambia nada.
- `security.json` (los avisos de seguridad, consulta [Actualizaciones](updates.md#avisos-de-seguridad)) y `security.json.minisig` se buscan junto al manifiesto. Un espejo que no los copia simplemente no muestra avisos.

Copia los archivos sin cambios: una firma cubre los bytes exactos. Los avisos de seguridad se vuelven a publicar para la última versión sin una compilación nueva, así que refresca `latest.json`, `security.json` y sus archivos `.minisig` desde la página de la versión cada vez que refresques el espejo.

## Desplegar un config.toml por defecto

GitPoulpi no tiene un `config.toml` para todo el equipo: los ajustes viven en la carpeta de cada usuario (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Para dar a los usuarios nuevos tus valores por defecto, copia un `config.toml` escueto (solo las claves que quieras, consulta la [Referencia de configuración](config-reference.md)) en esa carpeta cuando el usuario no tenga ninguno: un script de inicio de sesión o de Intune en Windows, `/etc/skel/.config/gitpoulpi/` en Linux, un script de primer inicio de sesión en macOS. Nada de los paquetes de GitPoulpi sobrescribe nunca una configuración, así que es tuya para gestionarla. No pongas ahí reglas de actualización que deban cumplirse: usa la política, que un usuario no puede anular. Una configuración portátil o por tarea también se puede elegir con `GITPOULPI_CONFIG_DIR`.
