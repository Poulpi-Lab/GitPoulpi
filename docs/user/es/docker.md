[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · **Español** · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Ejecutar en Docker

La imagen `poulpilab/gitpoulpi` contiene los ejecutables oficiales de las versiones (`gitpoulpi`, más los relés `gitpoulpi-tui` y `gitpoulpi-mcp`) sobre `debian:bookworm-slim`, con `git`, `git-lfs`, `openssh-client`, `less` y `tini`. Funciona en `linux/amd64` y `linux/arm64`. Está en Docker Hub.

## Iniciar la interfaz

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` da a la interfaz un terminal. Sin él, el programa no tiene dónde dibujar.
- `-v "$PWD":/work` monta la carpeta actual como directorio de trabajo (`/work`). Abre un repositorio desde ahí, o monta varios repositorios bajo `/work`.
- `-v gitpoulpi-config:/config` es un volumen con nombre que conserva tus ajustes y sesiones entre ejecuciones.
- `--user "$(id -u):$(id -g)"` ejecuta como tú, de modo que los archivos que GitPoulpi crea en tus repositorios conservan tu propietario del host en lugar de `root`. Todas las carpetas en las que escribe la imagen son escribibles por cualquiera, así que sirve cualquier id de usuario.

## Volúmenes y rutas

| Ruta | Qué contiene |
|---|---|
| `/work` | El directorio de trabajo. Monta aquí tus repositorios (bind mount). |
| `/config` | Ajustes (`config.toml`), sesiones y, como subcarpetas, datos y caché (`GITPOULPI_CONFIG_DIR=/config`). Usa un volumen con nombre o un bind mount para conservarlos. |
| `/data`, `/cache` | Declaradas también como volúmenes, para herramientas que siguen las variables XDG (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. No es un volumen: uno nuevo por contenedor. |

Sin un volumen para `/config`, cada ejecución con `--rm` empieza con los ajustes por defecto. Para partir de tu propio `config.toml`, móntalo: `-v "$PWD/config.toml":/config/config.toml:ro` (solo lectura significa que Ajustes no puede guardar).

## Repositorios de otro usuario

Git rechaza un repositorio cuyo propietario es distinto del usuario que lo ejecuta («dubious ownership»). Con `--user` como arriba, el propietario coincide. Cuando no coincide (un bind mount desde otra cuenta, una carpeta compartida de Docker Desktop), confía en todos los directorios con:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

El punto de entrada ejecuta entonces `git config --global --add safe.directory '*'` antes de iniciar la interfaz. Úsalo solo para carpetas de confianza, ya que levanta la protección de git para todo el contenedor.

## SSH

Reenvía tu agente SSH en lugar de copiar claves dentro del contenedor.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) expone el agente del host en una ruta fija dentro de su VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Con `--user`, el socket debe ser legible por tu id de usuario, lo que ocurre cuando te pertenece. El contenedor no tiene `known_hosts`: la primera conexión a un host te pide aceptar su clave, o monta tu propio archivo con `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Identidad de Git y credenciales

- **Identidad y ajustes:** monta tu configuración de Git en solo lectura: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. O fija la identidad de commit en un perfil de GitPoulpi (Ajustes > Perfiles), que vive en `/config`.
- **Credenciales HTTPS:** la imagen no tiene un credential helper como un llavero. Monta un almacén de credenciales (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, con `credential.helper = store` en el `.gitconfig` montado), o usa un remoto SSH con el agente reenviado.
- **Tokens de alojamiento** (pull requests, issues): sin un credential helper, GitPoulpi guarda un token solo en memoria durante la sesión. Para una configuración duradera pásalo por el entorno con `-e GITPOULPI_TOKEN_<ID>=...`, donde `<ID>` es el id de la integración en mayúsculas con los caracteres no alfanuméricos como `_` (consulta [Integraciones de alojamiento y de gestores de issues](hosting-integration.md)). Las integraciones en sí están en `config.toml`, bajo `/config`.

## Terminal, colores y ratón

Los colores y el soporte de ratón vienen de tu emulador de terminal. Docker traspasa `TERM` desde `-t`; añade `-e TERM -e COLORTERM` cuando los colores o el tema de color verdadero se vean mal, por ejemplo `-e COLORTERM=truecolor`. Los clics, arrastres y la rueda del ratón llegan por el terminal como de costumbre. El grafo de commits se dibuja con caracteres; las imágenes (kitty, iTerm2, sixel) no se reenvían a través de todos los terminales y multiplexores, consulta [Elegir un terminal](terminals.md). Al cambiar el tamaño de la ventana del terminal cambia el de la interfaz: el contenedor recibe el aviso del cambio de tamaño.

## CLI y MCP

El punto de entrada inicia la interfaz. Los demás programas se alcanzan sustituyéndolo:

```
# la CLI para scripts (añade --json para scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# el servidor MCP, por entrada y salida estándar, para un agente que lo lanza
docker run -i --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi mcp serve
```

Cada `docker run` es su propio contenedor con su propio daemon de GitPoulpi en segundo plano, así que los cambios hechos con la CLI no aparecen en vivo en una interfaz que se ejecuta en otro contenedor. Para manejar una interfaz desde la CLI, ejecuta ambas en el mismo contenedor: inicia la interfaz con `--name gp` y usa `docker exec gp gitpoulpi ...`. Para un agente, registra `docker run -i --rm --entrypoint gitpoulpi ... mcp serve` como comando del servidor MCP, consulta [MCP y agentes](mcp-and-agents.md); los interruptores de permisos de MCP del `config.toml` montado se aplican como siempre.

## Etiquetas y actualización

| Etiqueta | Significado |
|---|---|
| `X.Y.Z` | Una versión exacta. |
| `X.Y` | El último parche de esa versión menor (solo versiones estables). |
| `latest` | La última versión estable. |

Una versión preliminar solo recibe su etiqueta exacta `X.Y.Z-...`.

Una imagen se reemplaza, nunca se actualiza en el sitio. GitPoulpi dentro de un contenedor lo sabe (su registro de instalación dice «una imagen de Docker»): el [panel de actualización](updates.md) muestra la versión y los comandos y no reemplaza nada. Para actualizar:

```
docker pull poulpilab/gitpoulpi
```

Después inicia un contenedor nuevo. Tus ajustes se quedan en el volumen `/config`. Para volver atrás, ejecuta una etiqueta anterior: `docker pull poulpilab/gitpoulpi:<previous>`. Para fijar una versión en scripts, usa la etiqueta exacta en lugar de `latest`. Una organización puede replicar la imagen en su propio registro, consulta [Distribución en empresas](enterprise-distribution.md#espejo-del-registro-de-docker).
