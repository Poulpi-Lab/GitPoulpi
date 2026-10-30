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
carpeta de tu `PATH`. `INSTALL.txt`, dentro del archivo, da los mismos pasos.

## Con un instalador

Cada versión incluye además un instalador por sistema operativo. Los dos toman las mismas decisiones
(accesos directos, PATH, un alias del comando como `gp`) llamando al propio `gitpoulpi` instalado, el
código que hay detrás de Ajustes > General > Lanzador, de modo que lo que configura el instalador y lo que muestra
Ajustes siempre coinciden.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Se instala para el usuario actual, sin pedir permisos de administrador, en
`%LOCALAPPDATA%\Programs\GitPoulpi` (el primer diálogo ofrece una instalación para todos los usuarios y la carpeta de
instalación). Casillas: acceso directo del menú Inicio, acceso directo del Escritorio y añadir al PATH. Una página
pregunta por el **alias del comando**: `gp` por defecto, el nombre que quieras o vacío para ninguno. PowerShell ya usa
`gp` para `Get-ItemProperty`, así que allí elige otro nombre o usa `gitpoulpi`.

El instalador copia los tres ejecutables y la marca `.gitpoulpi-dist`, así que
[Actualizar ahora](#actualizar-en-el-sitio) funciona igual que con el zip. Una instalación para el usuario actual es
escribible y puede actualizarse sola; una instalación para todos los usuarios en Program Files necesita elevación para
reemplazarse, así que reinstálala con el instalador nuevo. Para desinstalar, usa Configuración de Windows > Aplicaciones:
ejecuta `gitpoulpi uninstall` (alias, entradas del PATH, accesos directos) y borra los archivos, incluidos los restos
`.old` del actualizador. Tus ajustes y tus repositorios se quedan.

### Linux y macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Elige el archivo para tu sistema operativo y tu CPU, comprueba su `.sha256`, instala en `~/.local/opt/gitpoulpi` y
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

La barra de título muestra la versión en ejecución (`v1.0.0`) entre el nombre de la aplicación y la ruta del repositorio. Haz clic en ella, o
ejecuta **Ayuda: Notas de la versión** desde la paleta de comandos (`Ctrl+P`): se abre una pestaña Notas de la versión con
el registro de cambios, desplazado hasta la versión en ejecución.

Una vez al día, en segundo plano, GitPoulpi pregunta a GitHub si hay una versión más reciente
(`updates.mode`, [Ajustes > Sincronización](settings.md#sincronización)). Cuando la hay, el chip dice
`v0.2.0 disponible` en el color de acento y, al hacer clic, abre la pestaña Notas de la versión con las notas de esa
versión arriba. La pestaña tiene tres botones, cada uno con una tecla y un comando de la paleta:

- **Descargar** (`d`, *Actualizaciones: Descargar*) abre la página de la versión en tu navegador.
- **Omitir esta versión** (`s`, *Actualizaciones: Omitir esta versión*) oculta el aviso hasta que aparezca una versión
  más reciente.
- **Buscar ahora** (`r`, *Actualizaciones: Buscar actualizaciones ahora*) vuelve a preguntar al instante.

Con `updates.mode = "off"` no se hace ninguna petición. Sin conexión de red, o mientras el repositorio es privado y no
hay un token de github.com guardado, la comprobación falla en silencio (una línea en el registro de la aplicación, nada
en la interfaz). `GITPOULPI_UPDATE_API` dirige la comprobación a otra raíz de API (la usan las pruebas).

### Actualizar en el sitio

Una compilación instalada desde un paquete de versión puede reemplazarse a sí misma. El paquete lleva un pequeño archivo
`.gitpoulpi-dist` junto a los ejecutables; una compilación en ejecución es «actualizable» solo cuando ese archivo está
junto a su ejecutable (y nombra su plataforma). Una compilación hecha con `cargo`, una compilación personalizada copiada
a otro sitio o un paquete que descartó el archivo nunca se actualiza sola: su pestaña Notas de la versión lo dice y
ofrece solo **Descargar**.

Para una compilación actualizable, la pestaña añade:

- **Actualizar ahora** (`u`, *Actualizaciones: Actualizar ahora*) descarga el paquete y su `.sha256` en segundo plano
  (una tarea en el indicador de la barra de estado, con su porcentaje y cancelable como cualquier tarea), comprueba el
  hash, descomprime el paquete y deja cada archivo nuevo junto al antiguo como `<nombre>.new`. No se toca nada de lo que
  está en ejecución. En el modo `auto` ocurre lo mismo por sí solo en cuanto una comprobación encuentra una versión más
  reciente.
- **Reiniciar para actualizar** (`R`, *Actualizaciones: Reiniciar para actualizar*; el chip dice lo mismo cuando la
  descarga ya está preparada) es siempre una acción tuya, nunca automática. Detiene el daemon independiente de la sesión
  (uno integrado se va con la ventana), cambia los archivos e inicia la versión nueva con los mismos argumentos y la
  misma sesión.
- **Volver a la versión anterior** (`v`, *Actualizaciones: Volver a la versión anterior*) devuelve los archivos anteriores y reinicia.
  Los archivos de la versión anterior se quedan como `<nombre>.old` hasta que la versión nueva lleva un minuto en
  ejecución; entonces se borran, así que el botón está ahí durante esa primera ejecución. No forma parte del historial de
  deshacer de git: cambia los archivos propios de GitPoulpi, no un repositorio.

Cómo se reemplazan los archivos: en Linux y macOS cada archivo nuevo se renombra encima del antiguo (de forma atómica;
un programa en ejecución conserva su copia antigua hasta que termina). En Windows un `.exe` en ejecución no se puede
sobrescribir, así que el antiguo se renombra a `<nombre>.old` y se mueve el nuevo; un `.old` que sigue en ejecución lo
borra un inicio posterior. El cambio es todo o nada: si otro programa tiene abierto uno de los ejecutables (un cliente
MCP que ejecuta `gitpoulpi-mcp`, por ejemplo), GitPoulpi nombra el archivo, no cambia nada y deja la descarga preparada
para otro intento.

Lo que se rechaza, cada cosa con su mensaje: una versión sin archivo `.sha256`, un hash que no coincide, un archivo cuyo
nombre no es el de esta plataforma (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), un paquete cuya marca nombra otra
versión u otra plataforma y cualquier enlace de descarga fuera de estos hosts: `github.com`, `api.github.com`,
`objects.githubusercontent.com`, `release-assets.githubusercontent.com` y `github-releases.githubusercontent.com` de GitHub
(la primera petición y cada salto de redirección; una redirección a cualquier otro sitio detiene la descarga). Los
paquetes de un repositorio privado necesitan una sesión de navegador, así que la descarga falla allí hasta que el
repositorio sea público.

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
