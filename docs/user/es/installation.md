[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · **Español** · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Instalación

GitPoulpi tiene tres ejecutables:

| Ejecutable | Qué es |
|---|---|
| `gitpoulpi-tui` | La interfaz de terminal. |
| `gitpoulpi` | La interfaz de línea de comandos (`--json` para scripts). Sin subcomando, abre la interfaz. |
| `gitpoulpi-mcp` | El servidor MCP para agentes de IA. Consulta [MCP y agentes](mcp-and-agents.md). |

Junto a ellos, el archivo contiene `gitpoulpi-launch`, un programa diminuto que ejecutan los accesos directos para mostrar una ventana de GitPoulpi ya abierta (`gitpoulpi launch`). En Windows no tiene ventana de consola, así que un clic nunca hace parpadear una. Se instala, actualiza y elimina junto con los demás.

## Qué necesitas

- Un **`git` del sistema** en tu `PATH`. GitPoulpi usa libgit2 para la mayoría
  de las operaciones y tu `git` para el resto.
  Se necesita la versión **2.34 o posterior**. En macOS sirve el `git` de las Command Line Tools de Apple, y también el de Homebrew.
- Un **terminal** con soporte de ratón. kitty, Ghostty, WezTerm e iTerm2 (y también Windows Terminal y foot, mediante sixel)
  dibujan el grafo de commits como imágenes. Los demás terminales (la
  consola de Windows, tmux y la mayoría) muestran un grafo de caracteres.
  Consulta [Elegir un terminal](terminals.md).
- Una **Nerd Font** o una fuente con emojis, para los iconos de la barra de
  herramientas y de la barra lateral.
- Opcional: **`git-lfs`** 3.0 o posterior, para repositorios con Git LFS. Sin
  él, los archivos LFS aparecen como pequeños archivos puntero y GitPoulpi te
  explica por qué. Consulta [Git LFS](lfs.md) para saber cómo instalarlo.
- Opcional: un **credential helper** (Git Credential Manager, libsecret, `pass`,
  Keychain), para guardar los tokens de los servicios de alojamiento y las
  claves de API de la IA. Consulta
  [Integraciones de alojamiento y de gestores de issues](hosting-integration.md).

## Desde un archivo de release

Un archivo de release es `gitpoulpi-<version>-<target>.zip` (Windows) o
`.tar.gz` (Linux, macOS), con un archivo `.sha256` al lado. Comprueba el
archivo con el `.sha256`, descomprímelo y coloca los tres ejecutables en una
carpeta de tu `PATH` (mantén `gitpoulpi-updater` y la marca `.gitpoulpi-dist`
junto a ellos: la marca dice `channel=archive`, para que la compilación pueda
actualizarse sola, consulta [Actualizaciones](updates.md)). `INSTALL.txt`, dentro
del archivo, da los mismos pasos.

### Verificar una descarga

Cada archivo de una versión está firmado con la clave de [minisign](https://jedisct1.github.io/minisign/) del proyecto, y GitPoulpi comprueba esa firma él mismo antes de instalar una actualización. Para un archivo que descargaste a mano, el `.minisig` que hay a su lado es la firma. La clave pública (id de clave `B461FFCBF6F6A4F2`) está confirmada como `packaging/minisign.pub` en el repositorio:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Lee el comentario de confianza que imprime el comando: debe ser `file:<el nombre del archivo> version:<la versión que esperas>`. Una firma válida que nombra otro archivo u otra versión no es una firma del tuyo.

Donde la versión se compiló con atestaciones de procedencia de la compilación (la variable `ATTESTATIONS` del repositorio), la CLI de GitHub comprueba dónde se compiló un archivo:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

El archivo `.sha256` solo detecta una descarga dañada. La firma es lo que demuestra que el archivo es del proyecto.


## Con un instalador

Cada versión incluye además un instalador por sistema operativo. Los dos toman las mismas decisiones
(accesos directos, PATH, un alias del comando como `gp`) llamando al propio `gitpoulpi` instalado, el
código que hay detrás de Ajustes > General > Lanzador, de modo que lo que configura el instalador y lo que muestra
Ajustes siempre coinciden.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

El primer diálogo pregunta para quién es la instalación:

| Opción | Dónde | Derechos de administrador | Qué significa para las actualizaciones |
|---|---|---|---|
| **Solo para mí** (por defecto) | `%LOCALAPPDATA%\Programs\GitPoulpi` | ninguno | GitPoulpi reemplaza sus propios archivos: **Actualizar y reiniciar** en el [panel de actualización](updates.md), sin pedir confirmación. |
| **Para todos los usuarios** | `Program Files\GitPoulpi` | una petición al instalar | Cada actualización pide la aprobación de un administrador (una petición de Windows), y antes se cierran las sesiones de GitPoulpi de otros usuarios del equipo. |

La misma elección funciona en silencio: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` o `/ALLUSERS`.

Casillas: acceso directo del menú Inicio, acceso directo del Escritorio y añadir al PATH. Una página pregunta por el **alias del comando**: `gp` por defecto, el nombre que quieras o vacío para ninguno (`/ALIAS=name` o `/ALIAS=` en la línea de comandos). PowerShell ya usa `gp` para `Get-ItemProperty`, así que allí elige otro nombre o usa `gitpoulpi`.

El instalador copia los ejecutables (incluido `gitpoulpi-updater`, el pequeño programa que cambia los archivos durante una actualización) y la marca `.gitpoulpi-dist`. La marca registra cómo se instaló GitPoulpi (`inno-user` o `inno-machine`), que es lo que el panel de actualización muestra como **Instalado con**. Volver a ejecutar el instalador conserva tus elecciones (ámbito, accesos directos, alias).

Para desinstalar, usa Configuración de Windows > Aplicaciones: ejecuta `gitpoulpi uninstall` (alias, entradas del PATH, accesos directos) y borra los archivos, incluidos los restos `.old` del actualizador. Tus ajustes y tus repositorios se quedan.

### Linux y macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Elige el archivo para tu sistema operativo y tu CPU, comprueba su `.sha256`, instala (canal `archive`: GitPoulpi se actualiza en el sitio) en `~/.local/opt/gitpoulpi` y
enlaza `gitpoulpi` y `gitpoulpi-mcp` en `~/.local/bin`. No hace falta root y no se toca nada fuera de tu carpeta
personal. Sin `--yes`, pregunta por el alias (`gp` por defecto), el PATH y los accesos directos.

| Opción | Efecto |
|---|---|
| `--prefix DIR` | Instala los archivos del programa en `DIR` (ruta absoluta). |
| `--version X.Y.Z` | Instala esta versión en lugar de la última. |
| `--archive FILE` | Usa un archivo local; `FILE.sha256` debe estar a su lado (`--no-verify` lo omite). No se usa la red. |
| `--alias NAME` / `--no-alias` | Hace que `NAME` inicie GitPoulpi (`gitpoulpi alias create`), o ninguno. |
| `--no-path` | No añade GitPoulpi al PATH de los shells nuevos (si no, `gitpoulpi path add`). |
| `--desktop` | Crea también los accesos directos del menú de aplicaciones y del Escritorio. |
| `--yes` | No pregunta nada: se aplican las opciones y los valores por defecto (alias `gp`, PATH sí, accesos directos no). |
| `--uninstall` | Ejecuta `gitpoulpi uninstall` y luego quita los enlaces y la carpeta. |

Consultar la última versión requiere que el repositorio sea público; si es privado, descarga el archivo en un navegador
y usa `--archive`.

## Con un gestor de paquetes

Cada canal de abajo escribe la misma marca `.gitpoulpi-dist` junto a los programas, con su propio nombre de canal. GitPoulpi la lee para saber quién se encarga de las actualizaciones: el gestor de paquetes, así que GitPoulpi nunca cambia archivos a sus espaldas. El panel de actualización (haz clic en la versión de la barra de título) ejecuta entonces el comando propio del gestor de paquetes, o lo muestra para que lo copies. Consulta [Actualizaciones](updates.md).

| Canal | Instalación | Notas |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Se instala para todos los usuarios, así que las actualizaciones piden la aprobación de un administrador. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Añade `--scope user` o `--scope machine` para elegir; el ámbito de equipo pide la aprobación de un administrador al instalar y en cada actualización. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` y luego `scoop install gitpoulpi` | Por usuario, sin derechos de administrador. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | El tap es `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | consulta [el repositorio apt](#apt-debian-y-ubuntu) | Repositorio firmado; también se adjunta un `.deb` a cada versión. |
| **MSI** (Windows, gestionado) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Para administradores: todos los usuarios, sin actualizaciones desde la aplicación. Consulta [Distribución en empresas](enterprise-distribution.md#msi-con-intune-o-directiva-de-grupo). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (o `paru -S gitpoulpi-bin`) | Sin helper: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, y luego `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Descarga el archivo de la versión en lugar de compilar. |
| **cargo, desde el código fuente** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compila; necesita una toolchain de Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Consulta [Ejecutar en Docker](docker.md). La misma imagen está en `ghcr.io/poulpi-lab/gitpoulpi`. |

Los nombres de arriba son aquellos en los que publica el proceso de publicación de versiones (consulta [Releasing](../../developer/releasing.md)): un canal que aún no está activo responde «not found», y el archivo de la versión o el instalador siempre funcionan.

Se aplica el manejo propio del PATH de cada gestor de paquetes: Chocolatey, winget, Scoop y Homebrew ponen los programas en el PATH por sí mismos, y ninguno crea el alias `gp` (ejecuta `gitpoulpi alias create gp`, o Ajustes > General > Lanzador).

### apt (Debian y Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

El repositorio admite `amd64` y `arm64` y está firmado con una clave que pertenece al proyecto. Los programas van a `/usr/lib/gitpoulpi` con enlaces en `/usr/bin`. El paquete depende de `git` y recomienda `git-lfs`. Sin el repositorio, instala el `.deb` de una versión con `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi te dirá entonces que descargues tú el siguiente `.deb`, porque solo ofrece comandos `apt` cuando encuentra el repositorio en `/etc/apt/sources.list.d/gitpoulpi.*`.

## Desde el código fuente

Necesitas una toolchain de Rust estable reciente (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Los ejecutables quedan en `target/release/`. El perfil `release` compila
rápido. Para obtener los ejecutables más pequeños y rápidos, usa el perfil
`dist`, que tarda más en compilar:

```
cargo build --profile dist
```

Los scripts `build-opti` de la raíz del repositorio ejecutan esta compilación
para los tres ejecutables y muestran sus tamaños:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd o PowerShell)
```

Para compilar solo los idiomas y funciones que usas, consulta
[Compilaciones personalizadas](custom-builds.md).

## Crear tú mismo un archivo de release

```
cargo run -p xtask -- dist
```

Esto compila el perfil `dist` y escribe
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) o `.tar.gz` (otros
sistemas). El archivo contiene los tres ejecutables, `INSTALL.txt`,
`README.md`, `LICENSE` y `CHANGELOG.md`. Junto a él se escribe un archivo
`.sha256`.

## Actualizaciones

La barra de título muestra la versión en ejecución (`v1.0.0`) entre el nombre de la aplicación y la ruta del repositorio. Cuando se conoce una versión más reciente (o hay una actualización en curso), un clic en ella abre el panel de actualización con la versión, cómo se instaló GitPoulpi y qué hacer ante la nueva versión; si no, un clic abre la pestaña Notas de la versión. `Ctrl+N` y luego `u`, o **Actualizaciones: Mostrar actualización** en la paleta de comandos (`Ctrl+P`), abren siempre el panel de actualización, y **Ayuda: Notas de la versión** abre siempre el registro de cambios.

Una vez al día, en segundo plano, GitPoulpi pregunta dónde se publican sus versiones (GitHub por defecto) si hay una más reciente (`updates.mode`, [Ajustes > Sincronización](settings.md#sincronización)). Cuando la hay, el chip dice `v0.2.0 disponible` en el color de acento.

Lo que hace el panel depende de cómo se instaló GitPoulpi: un archivo de versión o el instalador de Windows reemplaza sus propios archivos, un gestor de paquetes ejecuta su propio comando, una instalación gestionada o de contenedor dice cómo le llegan las actualizaciones. Una compilación hecha con `cargo build` nunca se actualiza. La descripción completa, con volver a la versión anterior, `auto` frente a `notify` y lo que un administrador puede bloquear, está en [Actualizaciones](updates.md).

## Comprobar que funciona

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` muestra la versión y el commit a partir del cual se hizo la build.
Después, sigue [Primeros pasos](getting-started.md).

## Desinstalar

`gitpoulpi uninstall` (lo que ejecutan los dos instaladores) quita los alias del comando, las entradas del PATH y los
accesos directos que creó GitPoulpi y, después, los archivos del programa de una instalación de una versión publicada;
se niega en una compilación sin la marca `.gitpoulpi-dist` (una compilación con cargo), y `--keep-files` deja los
archivos del programa. No está disponible para los agentes MCP. Sin instalador, borra los tres ejecutables. GitPoulpi
también conserva estos archivos:

| Qué | Dónde |
|---|---|
| Ajustes (`config.toml`) y sesiones | La carpeta de configuración de tu sistema operativo. `GITPOULPI_CONFIG_DIR` la cambia. |
| Estado por repositorio | `.gitpoulpi/` dentro de cada repositorio. |
| Cachés (avatares) | La carpeta de caché de tu sistema operativo. |
| Tokens y claves de API | Tu credential helper. **Ajustes > SSH y credenciales > Olvidar todos los usuarios y contraseñas** los elimina. |

Consulta la [Referencia de configuración](config-reference.md) para ver las
rutas exactas.
