[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · **Español** · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Integraciones de alojamiento y de gestores de issues

GitPoulpi se conecta a estos productos:

- **Alojamiento** (pull requests, issues, clonar desde tu cuenta, claves SSH):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea y Forgejo
  (incluido Codeberg). **Gerrit Code Review** es alojamiento solo para revisión: sus cambios son las
  pull requests, no tiene issues.
- **Gestores de issues** (solo issues): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud y Server) y Redmine.
- **Notas Markdown locales** (solo issues, sin token, sin red): una carpeta de notas Markdown, por ejemplo una carpeta de un vault de Obsidian, ver [Issues locales y Obsidian](local-issues.md).

Todas las integraciones usan un **token personal**. No hay cuenta de GitPoulpi ni
inicio de sesión con OAuth. Las integraciones son opcionales y **se activan por
repositorio**: no se obtiene nada hasta que vinculas un repositorio. Tu token se guarda en tu
gestor de credenciales, nunca en `config.toml`. Las operaciones normales de `git` (fetch, pull,
push) no necesitan una integración: usan las credenciales que tu `git`
ya tiene.

## 1. Crear un token en el host

El formulario de conexión tiene un enlace *Generar un token en <host>*. Abre la
página de tokens del host con los ámbitos adecuados ya seleccionados. Para hacerlo a mano:

| Host | Dónde | Ámbitos |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repositorios, pull requests, issues), `read:user` (quién eres), `admin:public_key` (solo para añadir o quitar claves SSH) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, en `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Crea el token **para esa organización**, no para «All accessible organizations»: los tokens globales dejan de funcionar el 2026-12-01. |
| Azure DevOps Server | La misma página en tu servidor (`https://<server>/<collection>/_usersSettings/tokens`) | Los mismos dos ámbitos |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Para las claves SSH, además `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profile picture > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, en `<host>/user/settings/applications` | Lectura de `user`; lectura y escritura de `repository`, `issue` y `user` (este último para las claves SSH). Gitea no muestra los ámbitos después, así que GitPoulpi no puede avisar de que falta uno: una llamada rechazada lo dice. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. La clave actúa como tú. |
| YouTrack (Cloud y Server) | Tu perfil > Account Security > New token, en `<host>/users/me?tab=account-security` | El ámbito `YouTrack`. Un token permanente actúa como tú. |
| Gerrit | Settings > HTTP Credentials > Generate new token (servidores recientes; los antiguos: Generate new password), en `<host>/settings/#HTTPCredentials` | Ninguno. El token es la **contraseña HTTP** y actúa como tú; el nombre de usuario va junto a él. |
| Redmine | My account > API access key > Show, en `<host>/my/account`. El administrador debe haber activado **Administration > Settings > API > Enable REST web service**. | Ninguno. La clave actúa como tú. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (el clásico, sin ámbitos) | Ninguno. El token actúa como tú. |
| Jira Data Center | Profile > Personal Access Tokens | Ninguno. El token actúa como tú. |
| Trello | Primero una **clave de API** desde `trello.com/apps/admin` (crea un Power-Up y luego genera una clave de API). Después un **token** desde `trello.com/1/authorize?...&key=<tu clave>`. | `read`, `write` |

Ponle al token un nombre como «GitPoulpi» y una fecha de caducidad. Copia el token:
el host lo muestra una sola vez.

## 2. Conectarlo

1. Pulsa `Ctrl+,` y elige **Integraciones**.
2. Elige el producto a la izquierda. Los productos están en dos grupos: **Alojamiento** y
   **Gestores de issues**.
3. Pulsa **Conectar**. Para añadir una segunda cuenta del mismo producto, usa
   **+ Añadir perfil**.

   ![La página Integraciones: los productos a la izquierda y un botón Conectar para GitHub](../images/settings-integrations.png)

4. Rellena el formulario y pega el token. El token se muestra en texto claro mientras
   lo escribes, así que asegúrate de que nadie esté mirando. El formulario lista los ámbitos
   que necesita.

   ![El formulario de conexión: un campo Token de acceso personal, los ámbitos repo, read:user y admin:public_key, un enlace para generar un token en github.com, y los botones Conectar y Cancelar](../images/settings-connect.png)

5. Pulsa **Conectar**. GitPoulpi comprueba el token con el host en segundo
   plano. Después la página muestra una tarjeta con tu nombre y tu login,
   *Conectado* en verde, y la sección de la clave SSH. Un token al que le falta alguno de los
   ámbitos deseados se conecta igualmente, y los ámbitos que faltan se listan en rojo.

En la tarjeta: **Editar** vuelve a abrir el formulario. **Desconectar** elimina el token
y conserva el perfil. **Quitar** elimina ambos, tras una confirmación.

Teclado en la página Integraciones: `Tab` / `Shift+Tab` se mueven entre la
lista de productos, la tira de perfiles y el formulario. `a` añade un perfil, `e` lo
edita, `d` o `Delete` lo quita, `o` abre la página de tokens, `Esc` retrocede un
paso.

### Qué pide cada formulario

El enlace *Generar un token* solo funciona después de rellenar el host o la
organización.

- **GitHub, GitLab:** solo el token. **GitHub Enterprise** y
  **GitLab Self-Managed** piden además el **Dominio del host**
  (`gitlab.example.com`; https salvo que escribas `http://`). Un nombre sin punto
  (un servicio de Docker o una máquina de la red local) necesita un puerto o `http://`:
  `gitea:3000`, `http://nas`. Un servidor autoalojado puede ser
  http simple, por ejemplo un contenedor local en `http://localhost:8080`. Un token enviado por
  http simple viaja sin cifrar, así que usa http solo en una red local o de
  confianza.
- **Azure DevOps:** la **Organización**, el nombre en
  `dev.azure.com/<organization>`. También se acepta una dirección `https://<org>.visualstudio.com`.
  Dos organizaciones necesitan dos perfiles. Un repositorio usa el
  perfil de la organización que nombra su remoto. **Azure DevOps Server**
  pide la **URL de la colección**, por ejemplo
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi no usa el gestor de credenciales
  del sistema para Azure DevOps: Git Credential Manager guarda ahí un token de
  inicio de sesión, no un token de acceso personal.
- **Bitbucket:** el **correo de tu cuenta de Atlassian** y un token de API. Si el
  correo está vacío, el token se envía como token bearer: úsalo para un token de acceso
  de workspace o de repositorio. GitPoulpi encuentra tus workspaces a partir del token.
  **Bitbucket Data Center:** la **URL del servidor** y un token de acceso HTTP.
- **Gitea, Forgejo:** el **Dominio del host** y el token. **Forgejo** empieza en
  `https://codeberg.org`: sustitúyelo para un servidor autoalojado. Gitea no tiene
  un valor por defecto público. Una dirección `http://localhost:3000` sirve para un servidor local.
  Ambos tipos comparten un cliente (Forgejo es un fork de Gitea con el mismo
  `/api/v1`); el token se envía como `Authorization: token <token>`. El
  gestor de credenciales del sistema no se usa para ellos.
- **Jira Cloud:** la **URL del sitio** (`acme.atlassian.net`), el **correo de tu
  cuenta de Atlassian** y un token de API clásico. Los tokens de Jira con ámbitos no
  se admiten. **Jira Data Center:** la **URL del servidor** y un token de acceso
  personal.
- **Trello:** la **clave de API** y el **token**. La clave de API es pública, así que
  se guarda en `config.toml` (como `account`). El token es secreto y va al
  gestor de credenciales. El formulario tiene dos enlaces: *Obtener una clave de API en Trello* y,
  después de escribir la clave, *Generar un token*. La clave y el token se envían en
  una cabecera, nunca en una URL.

- **Linear:** solo la **clave de API**: Linear tiene un único host público, así que no
  hay dirección que escribir. La clave se envía tal cual en la cabecera `Authorization`
  (Linear no usa `Bearer` para las claves personales). Para que las referencias `ENG-12`
  de los mensajes de commit abran la página correcta, guarda la URL del perfil con tu
  workspace (`https://linear.app/acme`, desde la CLI: `--url`); las filas de issues
  llevan siempre la dirección exacta.
- **YouTrack:** la **URL del sitio** (`acme.youtrack.cloud` para Cloud, tu propio
  servidor en otro caso, con su ruta de contexto si la tiene, por ejemplo
  `host/youtrack`) y un token permanente, enviado como `Authorization: Bearer`. El
  mismo cliente sirve para Cloud y Server (`<url>/api`).
- **Gerrit:** la **URL del servidor** (con su ruta de contexto si la tiene), tu **Nombre de usuario** y la contraseña
  HTTP como token. Viajan como autenticación HTTP basic; GitPoulpi habla con la API REST `/a/` del
  servidor e ignora la línea `)]}'` con la que empieza cada respuesta. El nombre de usuario no es secreto y se guarda en
  `config.toml` (como `account`), la contraseña va al gestor de credenciales. Un repositorio se vincula a un
  *proyecto* de Gerrit bajo PULL REQUESTS (sus cambios se listan allí); Gerrit no tiene issues, y un cambio se crea
  haciendo push (ver *Push para revisión* en [Revisar](review.md#gerrit)).
- **Redmine:** la **URL del sitio** de tu Redmine (con su subruta si la tiene,
  por ejemplo `host/redmine`) y tu clave de API, enviada como `X-Redmine-API-Key`. Si
  Redmine responde 403 o una página HTML, el servicio web REST está apagado (pídeselo al
  administrador, o actívalo en **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack y Redmine son solo gestores de issues: sin pull requests, sin repositorios que
clonar y sin claves SSH. Bitbucket no tiene issues (Atlassian retiró Bitbucket
Issues). Vincula la sección ISSUES de un repositorio de Bitbucket a un proyecto de Jira
en su lugar.

## 3. Dónde se guarda el token

El token se guarda en el **gestor de credenciales** de tu sistema (Git Credential
Manager en Windows, libsecret o `pass` en Linux, Keychain en macOS). Usa una
entrada privada que no se puede confundir con las credenciales de un host real. Nunca
se escribe en `config.toml` y nunca se imprime.

Si no hay un gestor de credenciales configurado, o si rechaza el token, GitPoulpi
guarda el token **en memoria solo durante esta sesión** y te lo dice. Vuelve a conectar
tras un reinicio.

GitPoulpi busca el token de un perfil en este orden:

1. La variable de entorno `GITPOULPI_TOKEN_<PROFILE ID>`. Úsala para dar un
   token sin guardarlo, por ejemplo en CI.
2. El token guardado para el perfil (`token_ref`).
3. Solo para perfiles de GitHub y GitLab: el gestor de credenciales del sistema para el
   host, cuando ningún otro perfil activado usa el mismo host.

Cada perfil es una entrada `[[integrations]]` en `config.toml`. Consulta la
[referencia de configuración](config-reference.md#integrations-alojamiento-y-gestores-de-issues).

## La comprobación de seguridad

Un token guardado solo es tan seguro como la dirección a la que se envía. Por eso GitPoulpi recuerda, para cada integración, a qué estaba conectada, y se detiene cuando eso cambia:

- **La dirección.** La primera vez que se usa una integración (o se conecta), su dirección se registra en `trust.toml`, en la carpeta de datos privada que solo lee el usuario (`<data>/private/`), no en `config.toml`: editar la configuración nunca puede ampliar aquello en lo que se confía. Si la dirección de la configuración ya no es la registrada, no se le envía nada y ni siquiera se lee el token. Las grafías de una misma dirección (`HTTPS://Git.Example.com:443/`) son la misma dirección; un host parecido no lo es.
- **La cuenta.** Tras conectar un token, y luego aproximadamente una vez al día (y una vez por ejecución para un token que viene de `GITPOULPI_TOKEN_*` o del credential helper), GitPoulpi pregunta al host a quién pertenece el token y compara el **id** de la cuenta, no el login. Un token que ahora pertenece a otra persona se detiene.
- **El certificado.** Los servidores cuyo certificado no encadena con una autoridad conocida (una CA privada, un servidor autofirmado) solo se aceptan para la clave exacta que aceptaste, y el handshake TLS sigue demostrando que el servidor tiene esa clave. Con la fijación estricta activada, la clave debe coincidir aunque el certificado encadene con una raíz pública; el precio es que cada rotación de clave vuelve a preguntar, por eso es opcional. Los hosts públicos (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) no guardan registro de certificado.
- **http sin cifrar.** Una dirección `http://` que no sea de loopback se rechaza salvo que la hayas permitido para esa integración.

Además, cada petición queda limitada a la propia dirección de la integración (y, para un host público, a su host de API como `api.github.com`): una petición a cualquier otro sitio se rechaza antes de enviarla, y una redirección a otra dirección no se sigue.

**Las integraciones conectadas antes de que existiera esta comprobación** reciben su registro en silencio, a partir de la configuración tal como estaba, la primera vez que se usan tras la actualización: conservan el nivel de confianza de hoy y no se pregunta nada.

### Desde la línea de comandos

Un comando detenido falla con código de salida 2 e imprime qué cambió (consulta la [referencia de la CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` muestra en qué se confía; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` y `--allow-insecure-http on|off` aceptan cada uno una cosa, nombrando el valor exacto. Tras aceptar una dirección nueva, introduce el token de nuevo. Desconectar (`hosting logout`) o quitar una integración borra su token, su registro de confianza y lo que se guardó en caché de ella (las listas de repositorios guardadas del diálogo Clonar, las listas de cuenta del Radar y los borradores de revisión remota de su host).

Un agente (MCP) no puede aceptar nada: se le dice que la comprobación de seguridad te necesita a ti.

### En la interfaz de terminal

Una petición detenida abre el diálogo **Comprobación de seguridad de la integración** sobre lo que estuvieras haciendo. Dice qué cambió (la dirección antigua y la nueva, la cuenta antigua y la nueva con sus ids, o el certificado antiguo y el nuevo como una pequeña tabla con la huella SHA-256 en grupos de cuatro, el sujeto, el emisor y la validez) y que no se envió el token. Las respuestas están numeradas:

1. **Desconectar (borra el token guardado)** borra el token guardado, el registro de confianza y lo que se guardó en caché de la integración.
2. **Confiar** en la nueva dirección, cuenta o certificado (una dirección sin cifrar: **Permitir http sin cifrar**).
3. **Cancelar** no cambia nada.

Ninguna respuesta aparece resaltada hasta que pulsas una flecha o un dígito, así que un Enter accidental no acepta nada. Esc y la cruz de cierre cancelan; un clic fuera del diálogo no hace nada. Un cuerpo largo se desplaza con la rueda, Página arriba/Página abajo o la barra de desplazamiento. Varias comprobaciones se ponen en cola (una por integración y tipo de cambio) y se preguntan una tras otra. Una comprobación cancelada no vuelve a aparecer mientras no cambie; **Integraciones: Revisar comprobación de seguridad** en la paleta de comandos la trae de vuelta.

Tras confiar en una **dirección** nueva, el formulario de la integración se abre con el campo del token enfocado y la pista «Se recomienda volver a introducir el token»: el token guardado nunca se envió allí. En el formulario, una dirección cambiada a propósito se acepta junto con un token escrito para ella; sin token, el formulario pide uno.

El formulario tiene además dos interruptores. **Fijación estricta del certificado** (para una dirección https) fija la clave del certificado que presenta el servidor cuando te conectas. **Permitir http sin cifrar** se ofrece solo para una dirección `http://` que no es esta máquina, con un aviso rojo de que el token viaja en texto plano. La tarjeta de conexión muestra la huella fijada.

### Qué puede hacer el token y hasta cuándo

Al conectar también se pregunta al host qué puede hacer el token. GitHub (tokens clásicos), GitLab y Bitbucket (cuando envía los ámbitos) informan de sus ámbitos: la tarjeta muestra una línea amarilla «Este token tiene más permisos de los que GitPoulpi necesita» con los ámbitos que exceden lo que usa GitPoulpi, los sensibles (como `delete_repo`, `admin:org`, `sudo`) en rojo, con el consejo de preferir un token de permisos detallados con fecha de caducidad. Un token clásico de GitHub (`ghp_...`) recibe ese consejo incluso cuando sus ámbitos son exactos. A un token de permisos detallados de GitHub no se le pueden pedir sus permisos, así que solo se muestra su caducidad. La tarjeta muestra la fecha de caducidad cuando el host la indica. Un token sin caducidad no muestra ningún recordatorio, y los demás productos no informan de ámbitos y no generan ningún aviso. Los mismos campos están en el JSON de `hosting integrations login` y `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Se leen al conectar, así que la tarjeta los muestra para la sesión que conectó.

## 4. Vincular un repositorio

La vinculación es explícita y por repositorio. Cuando hay un perfil conectado, la barra
lateral Refs muestra una sección **PULL REQUESTS** y una sección **ISSUES** después de TAGS.
Cada sección tiene dos desplegables: el proveedor y el proyecto.

- El desplegable del proveedor **propone** las cuentas que coinciden con los remotos del
  repositorio (`origin` primero), como filas del tipo `owner/name · origin · GitHub`. Una
  elección fija a la vez el proveedor y el proyecto.
- No se vincula nada automáticamente. Jira, Trello, Linear, YouTrack y Redmine nunca se proponen, porque
  ningún remoto los nombra: elígelos tú.
- **Ninguno (desvincular este repositorio)**, arriba del desplegable, desvincula la
  sección. Nada la vuelve a vincular por sí solo.
- El vínculo se guarda en el archivo `.gitpoulpi/repo-state.toml` del
  repositorio.

Las dos secciones se vinculan **por separado**. Por ejemplo, las pull requests pueden
venir de Azure DevOps y los issues de Jira.

| Proveedor | Proyecto de PULL REQUESTS | Proyecto de ISSUES |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | el mismo |
| Azure DevOps | `Project/Repo` (la organización sale del perfil) | `Project` (work items) |
| Gitea, Forgejo | `owner/name` | el mismo |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | ninguno: vincula Jira |
| Jira | ninguno | la clave del proyecto, por ejemplo `ABC` |
| Trello | ninguno | un tablero |
| Linear | ninguno | una clave de equipo, por ejemplo `ENG` (se listan los equipos) |
| YouTrack | ninguno | el nombre corto de un proyecto, por ejemplo `ABC` |
| Gerrit | el nombre del proyecto de Gerrit, por ejemplo `team/tool` | ninguno |
| Redmine | ninguno | un identificador de proyecto, por ejemplo `bench` (la última parte de la dirección del proyecto) |

La lista de proyectos de ISSUES admite búsqueda. También puedes escribir un proyecto que la
lista no muestre.

**Consulta adicional.** Una fuente de issues tiene una fila de filtro opcional bajo el
desplegable del proyecto. Se combina (AND) con el proyecto:

- Jira: un fragmento **JQL** (`component = UI AND labels = backend`).
- Azure DevOps: una condición **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack: una **consulta de YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Va entre paréntesis, así que un `or` no puede ampliar el proyecto. Un
  `sort by:` tuyo reemplaza el `sort by: updated desc` por defecto.
- Redmine: ninguno (bastan los filtros de estado y «asignados a mí» de la sección).
- Linear: ninguno (bastan los filtros de estado y «asignados a mí» de la
  sección).
- Trello: un desplegable donde marcas las **listas** a mostrar. Sin ninguna marcada se muestran todas
  las listas abiertas.
- GitHub y GitLab la ignoran.

`Enter` conserva una consulta escrita, `Esc` la descarta. Paleta:
*Issues: Editar consulta / listas...*.

Desde la CLI: `gitpoulpi hosting link pulls --id <profile>` (o `issues`),
`--clear` para desvincular, `--filter` para la consulta adicional. Consulta la
[referencia de la CLI](cli-reference.md#comandos).

### Clonar desde una cuenta conectada

Un perfil conectado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) también es una fuente en el diálogo **Clonar** (el botón **Clonar** de la pestaña Inicio, o `c` en la pestaña de gestión). Cada perfil aparece en **Clonar con URL** como una fuente propia (`GitHub · name`, `GitLab · name`); GitHub y GitLab nunca se mezclan. Elige uno y se cargan sus repositorios, agrupados por propietario: primero tu propia cuenta y después una sección por organización o grupo (`quarry/tools` para un subgrupo de GitLab), cada una bajo un encabezado con su recuento. Un candado marca un repositorio privado.

![El diálogo Clonar con una cuenta de GitHub conectada: una sección por propietario y un candado en un repositorio privado](../images/clone-hosted.png)

El cuadro de búsqueda filtra en todas las secciones (nombre, ruta completa y descripción); una sección sin coincidencias desaparece. `Enter` sobre una fila rellena el clon con la URL HTTPS de ese repositorio y un nombre de carpeta.

![El diálogo Clonar con la búsqueda reducida a una organización](../images/clone-hosted-search.png)

La lista de un perfil que ya has abierto antes aparece al instante a partir de una copia guardada mientras se vuelve a consultar al host (*actualizando...* junto a la etiqueta); cuando llega la respuesta, aparecen los repositorios nuevos y desaparecen los que ya no existen, y el repositorio resaltado y el elegido se quedan donde están. Si la actualización falla, la lista guardada se mantiene con una nota, y el motivo está en el Registro de actividad.

Si el host rechaza el token, el diálogo muestra *no se pudieron listar los repositorios* con el motivo en lugar de la lista (y la copia guardada se elimina); las demás fuentes siguen funcionando.

## 5. Pull requests e issues en la barra lateral

### PULL REQUESTS

La sección tiene una cabecera con el recuento y un `+` (crear una pull request),
los desplegables de proveedor y de proyecto, un cuadro de búsqueda y un desplegable de estado (*Abiertas*,
*Merged / cerradas*, *Todas*). Después vienen cuatro grupos, cada uno con su recuento:

- **Mis pull requests**: eres el autor.
- **Asignadas a mí**.
- **Pendientes de mi revisión**: eres un revisor solicitado.
- **Todas las pull requests**: plegado al principio.

El cuadro de búsqueda filtra todos los grupos (título, número, autor, rama). Cada
cambio del desplegable de estado vuelve a preguntar al host.

Una fila empieza con un glifo de estado: `●` abierta (verde), `○` borrador, `◆` merged
(morado), `✕` cerrada (rojo). La request de la rama activa va en negrita.
Las filas se leen `#12  título` (GitHub, Bitbucket) o `!53  título` (GitLab,
Azure DevOps).

### ISSUES

El título de la sección nombra al proveedor: `ISSUES DE GITHUB`, `ISSUES DE JIRA`, y así
sucesivamente. Dice `ELEMENTOS DE TRABAJO DE ...` para Azure DevOps y `TARJETAS DE ...` para Trello.
Bajo los desplegables: un cuadro de búsqueda, el desplegable de estado (*Abiertos*,
*Asignados a mí*, *Cerrados*, *Todos*) y los issues. Una fila de Jira se lee
`ABC-12  título`, con el tipo y el estado atenuados cuando la barra lateral es lo bastante
ancha.

### Acciones sobre una fila

- `Enter` o un doble clic abre la fila en tu navegador.
- Clic derecho, o `.`, abre el menú de la fila:
  - **Abrir en el navegador**, **Copiar URL**, **Copiar número**.
  - Para una pull request: **Editar en <proveedor>**, **Checkout de la rama de origen**,
    **Ver commits de la rama de origen**. Para una request desde un fork, el checkout dice
    **Checkout de pull request (fork)**, y **Ver** aparece en gris.
  - Para un issue: **Crear rama desde el issue**. Abre el aviso de nueva rama
    con un nombre como `12-fix-the-thing` (o `ABC-123-fix-the-thing` para
    Jira).
  - En GitHub, GitLab, Gitea y Forgejo, una fila de pull request tiene también **Revisar...**
    (y también **Enter** sobre la fila): la revisión dentro de la aplicación, ver
    [Revisar una pull request](review.md).
  - En GitHub, GitLab, Gitea y Forgejo, ambas filas tienen también **Editar asignados...**,
    **Editar etiquetas...** y **Establecer hito...** (y **Editar revisores...** en
    una pull request): ver
    [Asignados, revisores, etiquetas e hito](#asignados-revisores-etiquetas-e-hito).
- **Editar en <proveedor>** tiene otras tres vías de acceso: el lápiz `✎` en el extremo
  derecho de una fila de pull request con el ratón encima o seleccionada, `e` en la fila seleccionada, y
  el comando de la paleta *Integraciones: Editar pull request vinculada en el host*
  (para la request de la rama activa).
- Si la rama de origen todavía no está en el repositorio, GitPoulpi lo dice. Haz fetch
  primero (`Ctrl+G f`).

**Checkout de una pull request.** GitPoulpi hace fetch de la request a una rama local
`pr/<n>` (`mr/<n>` en GitLab) que la sigue, y cambia a ella. En
GitHub, GitLab, Gitea y Forgejo esto funciona también con forks (GitPoulpi hace fetch de la ref head
especial del host). Azure DevOps y Bitbucket hacen fetch de la rama de origen de la
request. Allí una request desde un fork se rechaza: añade el fork como remoto y haz
checkout de su rama tú mismo. Deshacer vuelve a la rama anterior. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Revisar una pull request desde la línea de comandos (y un agente).** En GitHub
(incluido Enterprise), GitLab (incluido autoalojado), Gitea y Forgejo una request se puede
leer y revisar sin salir de GitPoulpi: `hosting review show <n>` (archivos,
estado del merge, checks), `hosting review threads <n>` (hilos de revisión y
conversación), `hosting review comment`, `hosting review reply`, `hosting review resolve`
y `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. El diff viene de tu repositorio, no del
host: `hosting review fetch <n>` hace fetch del head de la request a
`refs/gitpoulpi/pr/<n>` y de su rama de destino a `refs/gitpoulpi/pr/<n>-base`
(ninguno es una rama, no se hace checkout de nada). Los números de línea son los de ese
diff: `new` es la línea en el head, `old` la línea en la base del merge. GitLab
no tiene el estado de revisión «solicitar cambios»: allí se rechaza, sin enviar nada.
Gitea y Forgejo admiten un comentario sobre una sola línea (sin rangos); un hilo son los
comentarios sobre una línea de un archivo. Gitea puede resolver un hilo, Forgejo 11 no
(el host no tiene endpoint para ello: la llamada lo dice). Una respuesta en un servidor
sin el endpoint de respuestas se publica como una revisión de un comentario en la misma línea.
Gitea se niega a aprobar o rechazar tu propia request.
Sin clon: añade `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` y `suggest` para revisar un proyecto que no es el repositorio actual,
`hosting review diff <n> --project P` lee el diff (hunks por archivo) desde la API del host, y
`hosting file <path> --project P [--ref R]` lee un archivo en una revisión (GitHub, GitLab, Gitea, Forgejo y Gerrit; en Gerrit el diff de un
patch set se lee archivo por archivo y el archivo sale de la rama o, para un id de commit, del commit). Un archivo que el host no
muestra en el diff (demasiado grande) se marca como `omitted`.
Las herramientas MCP son los mismos comandos (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Asignados, revisores, etiquetas e hito

En GitHub (incluido Enterprise), GitLab (incluido autoalojado), Gitea y
Forgejo, puedes cambiar a quién está asignada una pull request o un issue, quién la revisa, sus etiquetas
y su hito sin salir de GitPoulpi. Los demás proveedores no muestran nada de esto.

- **Menú de la fila**: **Editar asignados...**, **Editar revisores...** (solo pull
  requests), **Editar etiquetas...**, **Establecer hito...**.
- **Paleta**: *Alojamiento: Editar asignados...*, *Alojamiento: Editar revisores...*,
  *Alojamiento: Editar etiquetas...*, *Alojamiento: Establecer hito...* listan primero las
  requests y los issues de la barra lateral entre los que elegir. En el
  [Radar](radar.md) las mismas cuatro actúan sobre la fila seleccionada.
- El selector se abre al instante y carga del host las personas o etiquetas del proyecto (y lo que
  tiene ahora el elemento). Una fila con indicador giratorio aparece pasados unos 150 ms.
  Escribe para filtrar (coincidencia aproximada), `Up` / `Down` mueven, `Space` marca o
  desmarca, `Enter` aplica, `Esc` cancela, `Tab` llega a los botones. Un clic
  en una fila la marca, y la rueda desplaza. Las personas muestran una insignia con sus iniciales,
  las etiquetas su color.
- El hito es un desplegable (con **Sin hito** para quitarlo).
- Tras **Aplicar** se actualiza el host, y las listas y el Radar muestran el
  cambio tras su siguiente actualización, que empieza enseguida.

Los formularios de creación (una pull request, un issue) tienen las mismas opciones como una fila de
botones: `Asignados: ninguno`, `Revisores: ninguno`, `Etiquetas: ninguno`,
`Hito: ninguno`. En GitHub, GitLab, Gitea y Forgejo el botón Etiquetas sustituye al campo de
etiquetas escrito del formulario de issue. Si un host acepta la nueva pull request pero
rechaza algunas de sus etiquetas o personas, la barra de estado lo dice.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, y `--assignee`, `--reviewer`, `--label`,
`--milestone` en `hosting pr` y `hosting issue create`; ver la
[referencia de la CLI](cli-reference.md).

### Teclas en las secciones

| Tecla | Acción |
|---|---|
| `Enter` / doble clic | abrir la fila en el navegador |
| `.` / clic derecho | menú de la fila |
| `e` | editar la pull request seleccionada en el host |
| `n` | nueva pull request (PULL REQUESTS) o nuevo issue (ISSUES) |
| `v` | abrir el desplegable de estado |
| `r` | actualizar |
| `Enter` / `Space` en una cabecera | plegar o desplegar la sección |

### Reconectar y sin conexión

Cuando falta el token, se ha rechazado o le falta un ámbito, la primera fila de la
sección dice `Reconectar (token rechazado)`. Haz clic en ella, o pulsa `Enter`, para abrir
el perfil en Ajustes > Integraciones.

Cuando no se puede contactar con el host, las últimas listas se quedan en pantalla y la cabecera
muestra su antigüedad (`· de hace 5min`). La última respuesta también se guarda en caché en disco
(`.gitpoulpi/hosting-cache.json`), así que las listas aparecen antes de que la red
responda.

## 5b. El tablero de issues

El botón **Tablero** de la cabecera ISSUES (paleta: **Issues: Abrir tablero**) muestra los issues del origen vinculado como un tablero
Kanban sobre la ventana. Las columnas dependen del origen:

| Origen | Columnas | Mover una tarjeta |
|---|---|---|
| Notas Markdown locales | los estados (`statuses` del perfil, y después cualquier otro en uso) | reescribe el `status` de la nota |
| GitHub, GitLab, Gitea, Forgejo | **Abiertos**, después las etiquetas que elijas, después **Cerrados** | intercambia la etiqueta de estado; a **Cerrados** cierra el issue, a una columna de nuevo lo reabre |
| Trello | las listas abiertas del tablero, de izquierda a derecha | mueve la tarjeta a la lista |
| Redmine | los estados del issue | cambia el estado del issue (se aplican las reglas de flujo de Redmine) |
| Jira (Cloud y Data Center) | los estados del proyecto, los pendientes primero y los hechos al final | ejecuta la transición de flujo que lleva a la columna (`GET` y luego `POST .../transitions`); si no hay tal transición la tarjeta vuelve y la barra de estado dice `no transition from X to Y` |
| Azure DevOps (Services y Server) | los estados de los tipos de work item del proyecto, por categoría de estado | fija `System.State`; un work item cuyo tipo no tiene el estado de la columna vuelve, con el motivo (`a Bug has no state Done`) |

En un proyecto de GitHub o GitLab, **Columnas** (o `c`, o **Tablero: Definir etiquetas de columna...**) pide las etiquetas que son las
columnas, en orden, por ejemplo `status::todo, status::doing, status::review`. Propone las etiquetas del proyecto que
parecen estados. La elección se guarda para el repositorio. Sin etiquetas el tablero es Abiertos / Cerrados. Mover una tarjeta
solo toca las etiquetas de estado que listaste: sus demás etiquetas se quedan. Un tablero de host lista los issues abiertos y los 60 cerrados
más recientemente.

Una transición de Jira que pide campos obligatorios (una resolución, por ejemplo) la rechaza Jira y la tarjeta vuelve con
el propio mensaje de Jira: haz esa en el sitio web.

El mismo tablero está disponible para scripts y agentes: `gitpoulpi hosting board` imprime las columnas con sus tarjetas y
`gitpoulpi hosting board move <card> <column>` mueve una (consulta [la referencia de la CLI](cli-reference.md)).

Una tarjeta muestra el número, el título, los asignados como iniciales, las etiquetas y el hito. Arrástrala a otra columna,
o `Shift+Left` / `Shift+Right`, o **Tablero: Mover tarjeta a...** (`m`). Se mueve al instante; pasados unos 150 ms un indicador giratorio
dice que el host sigue trabajando, y si el host lo rechaza, la tarjeta vuelve y la barra de estado dice por qué. `Enter` abre
el issue en el host (la nota, en los issues locales); `e` edita asignados, etiquetas e hito; `n` inicia un issue nuevo en
la columna seleccionada; `a`, `l`, `M` y `/` filtran por asignado, etiqueta, hito y texto, `x` quita los filtros; `r`
vuelve a leer el host. El tablero vuelve a leer cuando se actualizan las listas de la barra lateral. El Radar no tiene tablero.

## 6. Crear una pull request

Abre el formulario con el `+` de la cabecera PULL REQUESTS, `n` en la sección, o
el comando de la paleta *Pull requests: Crear pull request...*. Un menú de rama
también tiene **Iniciar una pull request desde <rama>...**.

El formulario tiene:

- Una pestaña por cada perfil conectado que tenga pull requests (`Left` / `Right` en
  la tira de pestañas).
- Comboboxes **Repo de origen / Rama** y **Repo de destino / Rama**. Las listas largas admiten
  búsqueda. Puedes escribir una rama o un repositorio que las listas no muestren.
- **Título**, rellenado con el asunto del commit de la punta.
- **Descripción**, en varias líneas (`Enter` añade una línea nueva, `Tab` sale del
  campo).
- Botones **Asignados**, **Revisores**, **Etiquetas** e **Hito** (GitHub,
  GitLab, Gitea y Forgejo), que abren el selector.
- **Enviar como borrador**.
- Con la [IA](ai.md) activada: **Generar título y descripción**.

El formulario se abre con el perfil, el repositorio y las ramas de la rama
activa (`feature -> main`). Un repositorio de origen distinto crea una request
desde un fork.

**Crear pull request** (o `Ctrl+S`) crea la request en el host. El
resultado se abre en tu navegador y la barra de estado la nombra.

## 7. Crear un issue

Cuando la sección ISSUES tiene un proveedor **y** un proyecto, su cabecera tiene un
`+` (*+ Nuevo issue*). También: `n` en la sección, o el comando de la paleta
*Alojamiento: Nuevo issue...*.

El formulario tiene **Título** (obligatorio), **Descripción** (varias líneas) y
**Etiquetas** (separadas por comas, por ejemplo `bug, ui`; en GitHub, GitLab, Gitea y Forgejo los
botones Etiquetas, Asignados e Hito abren en su lugar el selector). Jira y Azure DevOps
añaden un desplegable **Tipo**. Trello añade un desplegable **Lista**. Linear y YouTrack
añaden un desplegable **Estado** (el estado del flujo, o el valor de `State`, en el que empieza
el issue). Redmine añade un desplegable **Tipo** con los trackers del proyecto (Bug, Feature,
Support...); no tiene etiquetas, así que no hay botón Etiquetas para él y una etiqueta
dada en la línea de comandos se rechaza. El formulario rellena estos
desplegables desde el host al abrirse.

**Crear issue** (o `Ctrl+S`) lo crea en el host. `Esc` o **Cancelar**
cierra el formulario. Si el host lo rechaza (token rechazado, ámbito que falta, proyecto
desconocido, etiqueta desconocida), el formulario muestra el motivo y conserva lo que escribiste.
Si todo va bien, la barra de estado nombra el nuevo issue y la sección ISSUES se
actualiza.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## 8. Crear en la propia página web del host

El formulario Nuevo issue y el formulario Crear pull request tienen un
botón **Crear en <proveedor>**, arriba a la derecha. La tecla es `Alt+W`. Abre el
formulario web del host en tu navegador, rellenado con lo que has escrito hasta ahora. El
diálogo sigue abierto. No necesita token y no hace ninguna llamada a la API, así que también funciona
tras un error de la API.

- GitHub no tiene parámetro de URL para *borrador*, así que la casilla de borrador no se
  traslada. GitLab recibe un prefijo `Draft: ` en el título.
- GitLab acepta las etiquetas solo por id, así que se añaden como una acción rápida
  `/label ~"x"` al final de la descripción.
- Una descripción muy larga se recorta para que la URL no pase de 6000 caracteres,
  con una nota `[...truncated: paste the rest]`.
- Bitbucket Data Center no tiene dirección de formulario web, así que no tiene
  botón **Crear en <proveedor>**.

CLI: `gitpoulpi hosting issue create ... --web` y `gitpoulpi hosting pr --web`
imprimen la misma URL.

## 9. El chip de la barra de herramientas

En el extremo de la barra de herramientas (a la derecha por defecto), un chip muestra la pull request de la rama
activa: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) o
`BB #12` (Bitbucket). Su color indica el estado: verde abierta, gris borrador,
morado merged, rojo cerrada.

- Pasa el ratón por encima para ver el título, el estado y `origen → destino`.
- Haz clic en él, o pulsa `Ctrl+G g`, para abrir la request en el navegador.
- Haz clic derecho para **Abrir en el navegador**, **Copiar URL**, **Actualizar** y
  **Cambiar de perfil**.

Una rama con push hecho y sin request muestra un chip tenue `+ PR`. Haz clic en él para iniciar una
pull request. No hay chip cuando ningún perfil coincide, cuando la rama no tiene push,
o cuando no se pudo contactar con el host. En una ventana estrecha el chip se oculta para dejar sitio
a los botones de la barra de herramientas.

## 10. Cuándo se actualizan las listas

Las listas se actualizan:

- cuando abres un repositorio;
- cuando cambia la rama activa;
- tras un push o un fetch;
- cada *Actualización de Pull Requests e Issues (minutos)*
  (`general.hosting_refresh_minutes`, por defecto 5, `0` apaga el temporizador;
  Ajustes > Integraciones);
- cuando cierras Ajustes;
- cuando pulsas `r`, ejecutas *Integraciones: Actualizar pull requests e issues*, o
  cambias un filtro.

Las peticiones nunca bloquean la ventana.

Los hosts limitan cuántas llamadas puedes hacer. GitPoulpi devuelve el `ETag` de la
última respuesta, así que una lista sin cambios cuesta poco (nada en GitHub). Cuando un
host informa de que quedan pocas llamadas, la barra de estado lo dice. Cuando no queda
ninguna, GitPoulpi deja de preguntar a ese host hasta que se restablezca el límite, y las listas
se quedan como están.

## 11. Terminología

GitHub, Azure DevOps y Bitbucket dicen «pull request». GitLab dice «merge
request». *Terminología de las solicitudes de revisión* (`ui.pr_terminology`, en
Ajustes > Integraciones) elige la palabra para cada etiqueta, menú, comando de
la paleta y mensaje:

- **Auto (según el host)**: sigue al proveedor vinculado del repositorio («Pull
  Request» mientras no haya nada vinculado).
- **Pull Request** o **Merge Request**: siempre esa palabra.

El cambio se aplica al instante. La CLI y el MCP siempre dicen «pull/merge request».
Esta página dice «pull request».

## 12. Referencias en mensajes de commit y nombres de rama

Cuando los issues o las pull requests del repositorio están vinculados, las referencias en
el mensaje de commit de los detalles del commit aparecen subrayadas. Un clic abre la
página vinculada. Teclado: el comando de la paleta
*Integraciones: Abrir referencia del commit (#issue / !solicitud)...* (un menú cuando el
mensaje tiene varias referencias). Un repositorio sin nada vinculado no tiene
enlaces.

Lo que cuenta como referencia depende del proveedor de la sección ISSUES:

| ISSUES vinculados a | En un mensaje | En un nombre de rama |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` y `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, solo cuando `ABC` es la clave del proyecto vinculado. Un texto como `UTF-8`, `SHA-256` o `ISO-8601` nunca es un enlace. | `ABC-123` en cualquier parte del nombre (`feature/ABC-123-x`), en mayúsculas |
| Linear | `ENG-123`, solo cuando `ENG` es la clave del equipo vinculado (o de un equipo de la lista de equipos del perfil). `UTF-8` nunca es un enlace. | El propio `branchName` de Linear cuando creas la rama desde un issue; `ENG-123` en cualquier parte del nombre se lee de vuelta |
| YouTrack | `ABC-123`, solo cuando `ABC` es el nombre corto del proyecto vinculado (o de uno de la lista de proyectos del perfil) | `ABC-123` en cualquier parte del nombre |
| Redmine | `#123`, solo cuando Redmine es el origen de ISSUES vinculado (un origen de pull requests conserva su propio signo) | `123-fix-login` |
| Trello | una dirección `https://trello.com/c/<shortLink>`; `#12` solo cuando una tarjeta en caché tiene el número 12 | `12-fix-login` |

| PULL REQUESTS vinculadas a | En un mensaje |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Cuando ambas listas coinciden con `#N`, gana el issue. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

### Insertar una referencia a un issue

En el compositor de commits, el botón `#▾` en el extremo derecho del Resumen abre
una lista filtrable de los issues vinculados. El issue nombrado por la rama actual
va primero, marcado *(de la rama)*. `Enter` o un clic inserta su referencia
(`ABC-123`, `#12`, `AB#12`) en el cursor. Teclado: `Alt+R` en el compositor.
Paleta: *Commit: Insertar referencia a un issue...*. Sin una lista de issues vinculada, el
botón aparece en gris y dice *vincula primero una lista de issues en la barra lateral*.
GitPoulpi nunca añade una referencia por sí solo.

## 13. Claves SSH

### Clave SSH por perfil

Un perfil conectado tiene una sección **Clave SSH** bajo su tarjeta. Sin clave,
elige una de estas:

- **Clave por defecto**: tu ssh-agent o `~/.ssh`. No cambia nada.
- **Generar clave nueva**: un título opcional (por defecto
  `GitPoulpi <host> <fecha>`). Con **Añadir la clave pública a <host>** marcado,
  GitPoulpi la sube, si el host no la tiene ya. La clave se
  escribe sin contraseña en `~/.ssh/gitpoulpi_<profile id>_ed25519` y
  `.pub`. No hace falta `ssh-keygen`. Un archivo existente nunca se sobrescribe.
- **Clave propia**: escribe una ruta o **Examinar** para elegir un archivo de clave privada,
  y luego pulsa **Usar esta clave**.

Cuando hay una clave asociada, la sección muestra `título (SHA256:...)` con:

- **Quitar**: desasocia la clave, y la elimina en el host si la añadió GitPoulpi.
- **Eliminar**: lo mismo, y además elimina los archivos de la clave, tras una confirmación
  que lista ambas rutas.

### Clave SSH para un repositorio

Para que un repositorio use la clave de un perfil, GitPoulpi define
`core.sshCommand` como `ssh -i "<key>" -o IdentitiesOnly=yes` en ese repositorio.
Esto nunca se hace automáticamente. Abre el menú de pestaña (`Ctrl+T`, o clic derecho
en la pestaña) y elige **Clave SSH...**, o ejecuta el comando de la paleta
*Repositorio: Clave SSH de este repositorio...*. El menú muestra la fijación actual,
cada perfil que tiene una clave privada, y **Quitar la fijación**.

CLI y MCP: `gitpoulpi ssh-command show | set --key <file> | clear`.

La clave global de Ajustes > SSH y credenciales (*Ruta de la clave privada*, o
*Generar par de claves*) se aplica a todos los repositorios sin fijación. La fijación
de un repositorio tiene prioridad sobre ella.

## 13b. Protección de ramas, para una reescritura del historial

Cuando reescribes el historial ([el editor](history-rewrite.md)), GitPoulpi
pregunta al host qué ramas protege, para dejarlas fuera de la reescritura
por defecto. Solo lo pregunta para un repositorio que **vinculaste** (la sección
de pull requests o issues de la barra lateral, o un perfil fijado), con el token
de ese perfil, y un token que no puede leerlo da simplemente «desconocido»: nada
falla. Qué responde cada proveedor y qué necesita el token:

| Host | Qué se lee | El token necesita |
|---|---|---|
| GitHub (y Enterprise Server) | el indicador `protected` de las ramas y los rulesets de ramas activos (patrones `~DEFAULT_BRANCH`, `~ALL` y `refs/heads/...`) | un token clásico con `repo` (`public_repo` para uno público), o un token de grano fino con Metadata (lectura). Un ruleset que el token no puede leer se avisa, y las reglas clásicas siguen contando |
| GitLab (gitlab.com y autoalojado) | `protected_branches` (nombres y patrones `*`) | `read_api`, y un rol que pueda ver los ajustes del repositorio |
| Azure DevOps (Services y Server) | las políticas de rama activas y bloqueantes del repositorio (ámbitos `Exact`, `Prefix` y `DefaultBranch`) | Code (lectura). El permiso de «force push» es un ajuste de seguridad y no se lee |
| Bitbucket Cloud | las restricciones de rama que impiden un push (`push`, `force`, `delete`) | `repository:admin`. Una restricción sobre el modelo de ramas no se resuelve |
| Bitbucket Data Center | los permisos de rama que impiden un force push (`fast-forward-only`, `pull-request-only`, `read-only`) | administrador del repositorio |

La página de ajustes de la protección (y de la rama por defecto, y de la
eliminación del repositorio) se construye a partir de la URL del remoto del
repositorio para GitHub, GitLab, Azure DevOps y Bitbucket, aunque el repositorio
no esté vinculado, y se muestra con las tres salidas cuando el host rechaza un push.

## 13c. Listas de toda la cuenta en el Radar

Normalmente el Radar vigila los repositorios que has clonado. Un perfil también puede
listar lo que te concierne en todo el host, esté clonado o no, con tres interruptores en
su tabla `[integrations.radar]` (todos desactivados por defecto; ver la
[referencia de configuración](config-reference.md#integrationsradar-listas-de-toda-la-cuenta)):

- **solicitudes de revisión**: solicitudes abiertas que piden tu revisión;
- **solicitudes abiertas**: todas las solicitudes abiertas de tu propio espacio de nombres y de cada organización o
  grupo del que eres miembro, o de los espacios de nombres que indiques. Nunca todo el host;
- **issues asignados**: issues abiertos asignados a ti.

Responden GitHub, GitLab (incluido el autoalojado), Gitea, Forgejo, Azure DevOps (Services
y Server), Bitbucket Data Center y Gerrit. Cómo se consulta a cada uno:

| Host | Solicitudes | Issues |
|---|---|---|
| Azure DevOps | pull requests activas de toda la organización (`searchCriteria.reviewerId` para las que te esperan; una solicitud en la que ya votaste se omite); cuando el servidor rechaza la ruta de nivel de organización, se consultan uno por uno los primeros 20 proyectos. Los `namespaces` son nombres de proyecto | work items asignados a ti que no están Done, Closed ni Removed |
| Bitbucket Data Center | tu panel: rol `REVIEWER` sin aprobar aún para las solicitudes de revisión, `AUTHOR` y `PARTICIPANT` para las solicitudes abiertas; los `namespaces` son claves de proyecto | ninguno (Data Center no tiene gestor de issues; una nota lo dice) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; las solicitudes abiertas son `is:open projects:<prefix>` por espacio de nombres (un prefijo de ruta de proyecto), o tus cambios propios y revisados si no se indica ninguno; un cambio en work-in-progress es un borrador | ninguno |

Bitbucket Cloud no tiene filas de toda la cuenta: no ofrece ningún endpoint que liste
solicitudes de varios repositorios, y consultar cada repositorio serían demasiadas
llamadas. Sus repositorios clonados mantienen sus propias listas, y `gitpoulpi hosting
radar --id <bitbucket>` lo dice. Cuando Gerrit tiene más cambios que el límite que
devuelve, la lista se marca como cortada con una nota.

Los borradores se omiten salvo que los conserves. Cada lista tiene un límite: 300 filas por tipo, con una
nota que dice cuántas más había, y en GitHub unas pocas llamadas de búsqueda por
ronda, porque su API de búsqueda solo permite 30 por minuto. Un espacio de nombres que el token
no puede leer se convierte en una nota y los demás se cargan igualmente; un token rechazado o un
límite de peticiones hace fallar solo a ese perfil. Las respuestas se reutilizan cinco minutos
(`radar-account.json` en la carpeta de configuración).

En Ajustes > Integraciones, un perfil de host de código conectado tiene un bloque **Radar**
bajo su clave SSH: una casilla por lista (*Solicitudes pendientes de mi revisión*,
*Todas las solicitudes abiertas*, *Incluir borradores*, *Issues asignados a mí*, esta última
ausente en los hosts sin issues), un campo *Espacios de nombres* (separados por comas; vacío
significa el tuyo y todos a los que perteneces) y la elección *Solicitud de un
repositorio no clonado*: *Preguntar*, *Clonarlo* o *Revisar mediante la API*.
Todo se guarda al cambiarlo. Los hosts sin consulta de toda la cuenta (Bitbucket
Cloud) muestran en su lugar una línea que lo dice. La paleta de comandos llega al bloque
con *Ajustes: Opciones del Radar de un perfil...*, y la búsqueda de Ajustes encuentra cada
control por su etiqueta.

Desde la línea de comandos: `gitpoulpi hosting integrations radar <id> --review-requests on`
activa una lista, `gitpoulpi hosting radar --json` imprime las listas (y
`--refresh` se salta la caché). Los agentes de IA tienen los mismos dos comandos por MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 14. Desde la línea de comandos

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

Consulta la [referencia de la CLI](cli-reference.md#comandos) para ver todos los comandos. Nunca se acepta
un token como argumento, y el servidor MCP no ofrece
`hosting login`, así que un token nunca pasa por un agente.

## Solución de problemas

| Síntoma | Solución |
|---|---|
| *Conectar* falla con un error de autenticación | El token es incorrecto, ha caducado o se ha revocado. Crea uno nuevo. |
| Un ámbito aparece en rojo | El token no lo tiene. La conexión funciona, pero esa función no. Añade el ámbito en el host, luego **Editar** y pega el token nuevo. |
| La barra lateral no tiene sección PULL REQUESTS | No hay ningún perfil activado conectado. Conecta uno y luego vincula el repositorio en el desplegable de la sección. |
| Una fila *Reconectar* en una lista | El token se perdió (un reinicio sin gestor de credenciales) o caducó. Haz clic en la fila para abrir el perfil. |
| Una instancia autoalojada falla | Comprueba que el **Dominio del host** no lleva ruta, y que tu equipo confía en el certificado de la instancia. |

## Límites conocidos

- **Pull requests desde forks:** Azure DevOps y Bitbucket no pueden hacer checkout de una
  request desde un fork (consulta [Acciones sobre una fila](#acciones-sobre-una-fila)). Azure
  DevOps no puede crear una pull request desde un fork.
- **Trello** muestra como máximo las primeras 1000 tarjetas de un tablero. Usa el filtro
  de listas para acotarlo. **Jira**, **Linear** y **YouTrack** listan como máximo 500
  issues por actualización. Usa un fragmento JQL o una consulta de YouTrack para acotarlo.
- **Linear** admite un asignado por issue: si eliges varios en el selector,
  se envía el primero. Su «hito» es el **ciclo** del equipo (los hitos de proyecto
  no se usan). El de **YouTrack** es el campo `Fix versions`; una
  versión no tiene número, así que el selector numera las no publicadas por su
  posición en el proyecto. Ninguno de los dos gestores tiene pull requests.
- **Redmine** admite un asignado por issue (el primero del selector), no tiene etiquetas,
  y lista como máximo 500 issues por actualización. Los asignados son los miembros del proyecto
  (personas y grupos) y se distinguen por su nombre visible. Las columnas del tablero Kanban
  son los estados del issue, y mover una tarjeta cambia el estado; se aplican las reglas de flujo
  por defecto de Redmine (un rol al que no se permite esa transición recibe el
  rechazo y la tarjeta vuelve a su sitio). «Resolved» no es un estado cerrado en los datos
  por defecto de Redmine: la columna Cerrados son los estados «Closed» y «Rejected».
- **Bitbucket Data Center** no tiene enlaces del navegador para nuevas pull requests,
  ramas o commits. Las páginas de las pull requests se abren con normalidad.
- **Azure DevOps** no muestra avatares de autor. Los filtros «mías» y «revisión solicitada»
  necesitan un endpoint que puede rechazar los tokens de acceso personal. En ese
  caso el perfil se conecta sin nombre de usuario y esos dos filtros no están
  disponibles. Un token no válido recibe una página de inicio de sesión en lugar de un error;
  GitPoulpi lo informa como token rechazado.
- **Gitea y Forgejo:** un borrador es una request cuyo título empieza por `WIP:`
  (el servidor no tiene marca de borrador al crear). Una request lista como revisores
  a todos los que ya la revisaron, y retirar una petición de revisión solo
  quita las pendientes. Las pull requests desde la rama local de un fork hacen
  checkout mediante `refs/pull/<n>/head`, como GitHub. Las menciones se buscan con el
  filtro `mentioned_by` del servidor. El servidor no muestra los ámbitos del token, así que uno
  que falte solo se descubre cuando se rechaza una llamada.
- **El tablero de issues** mueve tarjetas en todas las fuentes de issues. En Jira un movimiento ejecuta la transición de flujo que lleva
  a la columna; una transición que pide campos (una resolución, por ejemplo) la rechaza Jira y la tarjeta vuelve a su sitio con su
  mensaje. En Azure DevOps un tipo de work item sin el estado de destino vuelve a su sitio con el motivo.
- **Bitbucket:** las contraseñas de aplicación ya no existen; usa un token de API.
  Los repositorios se listan workspace por workspace.
- **Jira Cloud:** solo funcionan los tokens de API clásicos (ni tokens con ámbitos ni OAuth).
  Un issue nuevo de Jira recibe una descripción en texto plano.
- Crear una pull request, work item, issue o tarjeta ocurre en el host y
  **no se puede deshacer** desde GitPoulpi. Hacer checkout de una request sí se puede deshacer
  con normalidad.
