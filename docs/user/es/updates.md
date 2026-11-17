[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · **Español** · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Actualizaciones

GitPoulpi busca versiones más recientes y, según cómo se instaló, reemplaza sus propios archivos o le pasa la actualización al programa que lo instaló. Nunca se reemplaza ni se reinicia nada sin que tú lo pidas.

## El chip y el panel

La barra de título muestra la versión en uso (`v1.0.0`) entre el nombre de la aplicación y la ruta del repositorio. Cuando se conoce una versión más reciente, el chip pasa a `v0.2.0 disponible` con el color de acento.

Pulsa `Ctrl+N` y luego `u`, ejecuta **Actualizaciones: Mostrar actualización** desde la paleta de comandos (`Ctrl+P`), o haz clic en el chip mientras dice `disponible` (o mientras hay una actualización en curso): se abre el panel de actualización. Sin una versión más reciente, un clic en el chip abre en su lugar la pestaña Notas de la versión. Es una ventana sobre la aplicación: `Esc`, `q`, la cruz de cierre o un clic fuera la cierra. Muestra:

- la versión en uso e **Instalado con**: cómo llegó GitPoulpi a este equipo (de ello depende todo lo que sigue);
- en qué punto está la actualización (comprobando, descargando con un porcentaje, lista, fallida);
- los botones del procedimiento de esta instalación y el comando exacto cuando lo hay;
- una línea de tu administrador cuando ha puesto una.

Cada botón tiene una tecla (impresa en la línea de ayuda del panel) y un comando de la paleta, así que el panel funciona sin ratón:

| Botón | Tecla | Comando de la paleta |
|---|---|---|
| **Actualizar y reiniciar** (o **Ejecutar actualización**) | `u` | Actualizaciones: Actualizar ahora |
| **Copiar comando** | `c` | Actualizaciones: Copiar comando de actualización |
| **Abrir en terminal** | `t` | Actualizaciones: Ejecutar comando de actualización en un terminal |
| **Volver** | `v` | Actualizaciones: Volver a la versión anterior |
| **Notas de la versión** | `n` | Ayuda: Notas de la versión |
| **Omitir esta versión** | `s` | Actualizaciones: Omitir esta versión |
| **Abrir aviso** (solo con un aviso de seguridad) | `a` | Actualizaciones: Abrir aviso de seguridad |
| **Buscar ahora** | `r` | Actualizaciones: Buscar actualizaciones ahora |
| **Cerrar** | `Esc` | |

Un comando que no se aplica a esta instalación aparece atenuado en la paleta con el motivo (por ejemplo «esta instalación se actualiza de otra forma: mira Actualizaciones: Mostrar actualización»).

## Qué ocurre, según la instalación

| Instalado con | Se comprueba en | Qué hace el panel | Volver atrás |
|---|---|---|---|
| Un archivo de versión, o `install.sh` | Releases de GitHub | **Actualizar y reiniciar**: GitPoulpi descarga y verifica el archivo, se cierra, reemplaza sus archivos y se abre de nuevo en el mismo terminal. | **Volver** devuelve los archivos anteriores. |
| El instalador de Windows, solo para mí | Releases de GitHub | Lo mismo, sin pedir confirmación. En Windows se reabre en una ventana nueva. | **Volver** los devuelve. |
| El instalador de Windows, todos los usuarios | Releases de GitHub | Lo mismo, pero Windows pide la aprobación de un administrador. Se cierran las sesiones de GitPoulpi de otros usuarios del equipo, que pierden lo que no hayan guardado. Si la rechazas, no cambia nada. | **Volver** los devuelve, con la misma petición. |
| Chocolatey | El feed de la comunidad de Chocolatey | **Ejecutar actualización** ejecuta `choco upgrade gitpoulpi -y` (Windows pide la aprobación de un administrador). | El panel muestra `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, feed interno | El feed de tu organización | **Ejecutar actualización** ejecuta `choco upgrade gitpoulpi -y --source=<feed>`. Los parámetros del paquete no se vuelven a enviar, así que una configuración desplegada no se toca. | La misma línea `choco install` con el feed. |
| Scoop | Releases de GitHub | **Ejecutar actualización** ejecuta `scoop update gitpoulpi` después de que GitPoulpi se haya cerrado del todo. | **Volver** ejecuta `scoop reset gitpoulpi@<previous>`. |
| Homebrew | Releases de GitHub | **Ejecutar actualización** ejecuta `brew upgrade gitpoulpi` en este terminal. | El panel muestra `brew install gitpoulpi@<previous>`. |
| apt | Releases de GitHub | El panel muestra `sudo apt update && sudo apt install --only-upgrade gitpoulpi` con **Copiar comando** y **Abrir en terminal**. GitPoulpi nunca ejecuta `sudo` por ti. Sin el repositorio apt de GitPoulpi en este equipo, dice que descargues el nuevo `.deb`. | El panel muestra `sudo apt install gitpoulpi=<previous>`. |
| Un paquete MSI | Releases de GitHub, o la fuente de tu administrador | Nada que ejecutar: la instalación la gestiona tu administrador, cuyo mensaje muestra el panel. | No se ofrece. |
| cargo | Releases de GitHub | El panel muestra `cargo binstall gitpoulpi-cli` cuando `cargo-binstall` está instalado y, si no, `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | El mismo comando con la etiqueta anterior. |
| Una imagen de Docker | Releases de GitHub | Nada que ejecutar: una imagen se reemplaza. El panel muestra `docker pull poulpilab/gitpoulpi:<version>` y `:latest`. Consulta [Ejecutar en Docker](docker.md). | El panel muestra `docker pull poulpilab/gitpoulpi:<previous>`. |
| Un archivo de versión en una carpeta donde no puedes escribir | Releases de GitHub | El panel dice que GitPoulpi no puede reemplazarse ahí y remite a **Descargar**. | No se ofrece. |
| Una compilación de desarrollo (`cargo build`) | ninguno | Nunca se actualiza. El chip muestra la versión y nada más; el panel dice que no hay registro de instalación. | No se ofrece. |

Las filas «Mostrar» nunca ejecutan nada por sí solas. **Abrir en terminal** abre tu terminal con el comando ya escrito, y tú pulsas Enter.

Las versiones salen siempre del repositorio de GitHub del proyecto, salvo que un administrador dirija la comprobación a otro sitio (un feed interno, un espejo o un `latest.json`), consulta [Distribución en empresas](enterprise-distribution.md). Las notas de la versión salen de la release o del changelog incorporado en GitPoulpi, el que esté al alcance.

## Notificar y auto

`updates.mode` ([Ajustes > Sincronización](settings.md#sincronización)):

- **Notificar** (por defecto): una vez al día GitPoulpi comprueba en segundo plano y muestra el chip. No se descarga nada hasta que pulsas el botón.
- **Auto**: donde GitPoulpi reemplaza sus propios archivos (archivo de versión, instalador de Windows), además descarga y verifica la actualización en segundo plano en cuanto la encuentra. El chip pasa entonces a **Reiniciar para actualizar**, y el reinicio sigue siendo cosa tuya. Donde un gestor de paquetes se encarga de la actualización, **Auto** se comporta como **Notificar**.
- **Desactivado**: nunca se hace ninguna solicitud y el chip solo muestra la versión.

El ajuste **Canal de actualización** (`updates.channel`) elige entre **Estable** y **Preliminar** (también las versiones candidatas). Una comprobación que inicias con **Buscar ahora** ignora el límite de una vez al día. Sin conexión de red, o mientras el repositorio es privado y no hay un token de github.com guardado, una comprobación en segundo plano falla en silencio con una línea en el registro de la aplicación.

## Reinicio y qué ves después

Al pulsar **Actualizar y reiniciar** o **Ejecutar actualización** se guarda tu sesión y se cierra la ventana. Un pequeño programa aparte, `gitpoulpi-updater`, hace el trabajo, de modo que los archivos de GitPoulpi no están en uso mientras se reemplazan y un gestor de paquetes no queda bloqueado por un proceso padre que espera. Primero detiene los daemons en segundo plano de GitPoulpi (también se cierran las demás ventanas y servidores MCP de tu usuario), hace la actualización, comprueba que el nuevo `gitpoulpi --version` imprime la versión esperada y vuelve a iniciar GitPoulpi: en el mismo terminal en Linux y macOS, en una ventana nueva en Windows. Si la comprobación falla, devuelve los archivos antiguos.

El siguiente arranque muestra una línea en la barra de estado:

- `Actualizado a v1.4.2`
- `La actualización falló: <motivo>. La versión anterior sigue instalada.`
- `Actualización cancelada: se rechazó la aprobación del administrador. No cambió nada.`

Solo se ejecuta una actualización a la vez; un archivo de bloqueo en la carpeta de datos de GitPoulpi (y otro en la carpeta de instalación cuando se puede escribir en ella) impide que entre una segunda, y un bloqueo de más de diez minutos se ignora, de modo que una actualización que se cuelgue nunca bloquea GitPoulpi para siempre. La CLI y el daemon no arrancan mientras exista un bloqueo reciente.

## Volver a la versión anterior

Donde GitPoulpi reemplaza sus propios archivos, **Volver** devuelve los archivos anteriores y reinicia. Quedan como `<name>.old` durante un minuto después de que arranque la versión nueva y luego se borran, así que volver sirve para la primera ejecución tras una actualización. Donde un gestor de paquetes se encarga de la instalación, volver es el comando de la tabla: el panel lo muestra o lo ejecuta. Volver cambia los archivos de GitPoulpi, no un repositorio, así que no forma parte del diario de deshacer de git.

## Qué se comprueba y qué se rechaza

Antes de reemplazar nada, GitPoulpi descarga el archivo, su firma `.minisig` y su `.sha256`. Comprueba primero la firma, con la clave de versiones del proyecto incorporada en GitPoulpi, y el comentario de confianza de la firma debe nombrar este archivo y esta versión. Después comprueba el hash. Rechaza: una versión sin `.minisig` (una versión antigua publicada antes de que se firmaran las versiones no se puede instalar con el actualizador, lo que incluye fijarla o volver a ella), una firma de otra clave, una firma de otro archivo u otra versión, una versión sin `.sha256`, un hash que no coincide, un archivo cuyo nombre no es el de esta plataforma (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), un archivo cuya marca nombra otra versión u otra plataforma, y un enlace de descarga fuera de los servidores de releases de GitHub (se comprueba cada salto de redirección) o del espejo que configuró un administrador. El reemplazo es todo o nada: si otro programa tiene abierto uno de los ejecutables, GitPoulpi nombra el archivo, no cambia nada y deja la descarga para otro intento. En Windows, una actualización para todos los usuarios se vuelve a verificar en el paso elevado, que descarga por sí mismo el `.minisig` y el `.sha256` y comprueba ambos de nuevo, así que un archivo cambiado en la carpeta de descargas del usuario entre medias se rechaza. Los gestores de paquetes (Chocolatey, Scoop, Homebrew, apt) verifican sus propios paquetes: GitPoulpi no descarga nada ahí.

## Avisos de seguridad

Junto a cada versión, el proyecto publica `security.json`: la lista de avisos de seguridad (un identificador, una gravedad, las versiones afectadas como un rango como `>=1.2.0, <1.4.3`, la versión corregida, un enlace y un resumen de una frase). La misma comprobación diaria que busca una versión nueva lo descarga también, del mismo sitio que la versión: la página de la release del repositorio para GitHub, la carpeta de `latest.json` para un espejo, GitHub para el feed público de Chocolatey. Un aviso solo se usa cuando `security.json.minisig` se verifica con la clave del proyecto; un archivo sin firma o mal firmado nunca se muestra (una línea del Registro de actividad dice que se ignoró).

Cuando un aviso menciona la versión en uso, el chip se pone rojo y dice **Aviso de seguridad: esta versión está afectada**, en toda instalación, también en las gestionadas por un gestor de paquetes. El panel de actualización lista entonces cada aviso con su gravedad, resumen, versión corregida y enlace; **Abrir aviso** (`a`, o **Actualizaciones: Abrir aviso de seguridad** en la paleta) abre en el navegador el enlace del más grave. Donde un gestor de paquetes se encarga de la instalación, el panel añade «Actualiza con tu gestor de paquetes.»

Con la búsqueda de actualizaciones desactivada (`updates.mode = "off"`), con una política de administrador cuyo `check` es `none` y en una compilación hecha con `cargo build`, no se descarga nada y no se muestra ningún aviso. El feed NuGet propio de una organización no hace que GitPoulpi contacte con GitHub: sus avisos vienen solo del espejo `download_base` de la política.

## Ajustes que un administrador bloqueó

Un administrador puede fijar, en una política para todo el equipo, el modo de actualización, el canal, una versión fija, dónde comprobar y si GitPoulpi puede actualizarse a sí mismo. En Ajustes esas filas muestran **Fijado por tu administrador**, el panel muestra su mensaje y, cuando la autoactualización está desactivada, el panel ofrece los comandos o una nota en lugar de botones que reemplacen archivos. Los archivos de política ilegibles fallan por el lado seguro: solo notificar, sin autoactualización. El lado del administrador está en [Distribución en empresas](enterprise-distribution.md#el-archivo-de-política-y-el-registro).

## Entorno

`GITPOULPI_UPDATE_API` dirige la comprobación a otra raíz de API (la usan las pruebas).
