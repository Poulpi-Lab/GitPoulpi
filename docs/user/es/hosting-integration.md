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

**Adónde ir después.** Esta página es la configuración: tokens, conexión, vincular un
repositorio, claves SSH. Una vez conectada una cuenta:

- las secciones PULL REQUESTS e ISSUES de la barra lateral, el tablero de issues, crear una
  pull request o un issue, el chip de la barra de herramientas y las referencias en mensajes
  de commit: **[Pull requests e issues](pull-requests-and-issues.md)**;
- la vista en la app de un issue o de una request, con sus comentarios, la escritura y
  publicar como: **[Conversaciones](conversations.md)**;
- leer y revisar una request con hilos de comentarios y una revisión pendiente:
  **[Revisar una pull o merge request](review.md)**;
- las pull requests y los issues de todo un espacio de trabajo en una sola lista:
  **[El Radar](radar.md)**.

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

## 5. Claves SSH

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

Para que un repositorio use una clave concreta, abre el menú de pestaña (`Ctrl+T`, o clic derecho
en la pestaña) y elige **SSH y credenciales...**, o ejecuta el comando de la paleta
*Repositorio: SSH y credenciales...*, y elige **Archivo de clave** (o **Clave de la cuenta**
para la clave de un perfil de integración) en SSH. La elección se guarda en la carpeta privada
de GitPoulpi: no se escribe nada en el repositorio ni en la configuración de git. Consulta
[Ajustes](settings.md#ajustes-del-repositorio-ssh-y-credenciales).

Un repositorio que todavía tiene un `core.sshCommand` propio, de la antigua fijación
«Clave SSH de este repositorio», lo muestra en esa página, en solo lectura, con
**Mover a GitPoulpi**.

CLI y MCP: `gitpoulpi repo-auth` (la elección por repositorio) y, para la propia fijación de
`core.sshCommand`, `gitpoulpi ssh-command show | set --key <file> | clear`.

La clave global de Ajustes > SSH y credenciales (*Ruta de la clave privada*, o
*Generar par de claves*) se aplica a todos los repositorios sin una elección propia.
La clave o la fijación de un repositorio tiene prioridad sobre ella.

## 6. Protección de ramas, para una reescritura del historial

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

## 7. Listas de toda la cuenta en el Radar

Normalmente el Radar vigila los repositorios que has clonado. Un perfil también puede
listar lo que te concierne en todo el host, esté clonado o no, con tres interruptores en
su tabla `[integrations.radar]` (todos desactivados por defecto; ver la
[referencia de configuración](config-reference.md#integrationsradar-listas-de-toda-la-cuenta)):

- **solicitudes de revisión**: solicitudes abiertas que piden tu revisión;
- **solicitudes abiertas**: todas las solicitudes abiertas de tu propio espacio de nombres y de cada organización o
  grupo del que eres miembro, o de los espacios de nombres que indiques. Nunca todo el host;
  los **Espacios de nombres** del perfil (uno por línea en Ajustes > Integraciones > Radar, opcionalmente
  como expresiones regulares) limitan las tres listas: una línea coincide con la ruta completa de un
  repositorio cuando es igual a ella o es un prefijo de ruta que termina en `/` (`orbit` conserva
  `orbit/products/x`, no `orbitx/y`), y se aplica a las filas que devuelve el host, además de en el servidor
  donde la búsqueda del host lo permite (los calificadores `user:` / `repo:` de GitHub);
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
ausente en los hosts sin issues), un campo *Espacios de nombres* (una entrada por línea: un nombre que
coincide con ese grupo o propietario y todo lo que hay debajo, o una expresión regular por línea con
*Expresiones regulares* marcado; vacío significa el tuyo y todos a los que perteneces) y la elección *Solicitud de un
repositorio no clonado*: *Preguntar*, *Clonarlo* o *Revisar mediante la API*.
Todo se guarda al cambiarlo. Los hosts sin consulta de toda la cuenta (Bitbucket
Cloud) muestran en su lugar una línea que lo dice. La paleta de comandos llega al bloque
con *Ajustes: Opciones del Radar de un perfil...*, y la búsqueda de Ajustes encuentra cada
control por su etiqueta.

Desde la línea de comandos: `gitpoulpi hosting integrations radar <id> --review-requests on`
activa una lista, `gitpoulpi hosting radar --json` imprime las listas (y
`--refresh` se salta la caché). Los agentes de IA tienen los mismos dos comandos por MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 8. Actuar como bot de una GitHub App

Un proyecto que no debe mostrar el nombre de una persona puede hacerlo todo en el host a través de una **GitHub App**:
cada comentario, revisión, cierre o reapertura y cambio de etiquetas aparece entonces como `<app-slug>[bot]`. Esto
funciona para un repositorio a la vez y deja tus otros repositorios con tu propio token.

1. Crea una GitHub App para la organización con los permisos **Issues** y **Pull requests** en *Read & write* (y
   *Contents: read*, para que se puedan listar y revisar las pull requests), e instálala en el repositorio.
2. Escribe un pequeño script que imprima un **token de instalación** en su salida estándar (la clave privada de la App
   firma un JWT de corta duración, que se cambia por un token válido durante una hora). GitPoulpi nunca ve la clave.
3. En **Ajustes > Repositorio > SSH y credenciales**, elige **Comando de token** en HTTPS, escribe el script como
   **Comando** y el host (`github.com`) en **Hosts**, y marca **Usar para la API del host (issues, pull requests)**.
   Guarda y confirma el comando (una persona siempre lo confirma, con su SHA-256). Desde la línea de comandos:
   `gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`.

Lo que cambia para ese repositorio:

- Sus llamadas de issues y pull requests (listar, crear, comentar, revisar, hacer merge, cerrar, etiquetas) usan el
  token del comando, con el mismo límite de 30 segundos, la misma caché en memoria y el mismo ocultado de secretos que
  el inicio de sesión propio de git. Cuando el host responde 401 porque el token caducó, se vuelve a pedir el comando una
  vez. Si el comando no se puede ejecutar, la llamada se rechaza: tu token personal nunca es el recurso de reserva.
- No hay un «yo»: un token de instalación no puede preguntar quién es. La cabecera de la sección nombra el perfil con
  **(bot de App)** y los filtros construidos sobre «yo» (*Asignados a mí*, *Míos*, *Revisión solicitada*) no tienen nada
  con lo que coincidir. No se muestra nada como error.
- **Publicar como.** Cada mensaje que publica GitPoulpi se muestra con el nombre con el que se publica (*Publicando como
  my-app[bot] (bot de GitHub App)* bajo la cabecera de una [conversación](conversations.md#publicar-como) y en su caja de comentario;
  `posted_as` en la respuesta de la CLI). Con un token personal es tu login (`GET /user`). Con un token de App es el
  nombre de **Publicar como**, por ejemplo `my-app[bot]`: defínelo para todos los repositorios en **Ajustes >
  Integraciones** (`general.hosting_post_as`, también `gitpoulpi config set general.hosting_post_as my-app[bot]`) o para
  un repositorio en **Ajustes > Repositorio > SSH y credenciales > Alojamiento**, que prevalece. Si se deja vacío,
  GitPoulpi pregunta a GitHub (GraphQL `viewer { login }`) y, si no, aprende el nombre del autor del primer comentario
  que publica y lo conserva con el repositorio. Ese nombre es también como se reconocen tus propios comentarios, así que
  editar y eliminar solo se ofrecen en ellos (los comentarios de una App deben venir además de una cuenta de bot); hasta
  que se conoce, solo se pueden cambiar los comentarios que publicó esta instalación.
- Una App no se puede asignar. Combina la configuración con la asignación bloqueada de más abajo.
- Activar o desactivar **Usar para la API del host** necesita a una persona, como un comando nuevo.

Solo los perfiles de GitHub usan así el token del repositorio; los demás hosts conservan el token propio del perfil. (Un
token de acceso de proyecto o de grupo de GitLab también es un bot: es el token del perfil, se muestra como *bot del
proyecto*, y no hay nada que configurar.)

### Bloquear la asignación en un repositorio

**Ajustes > Repositorio > SSH y credenciales** tiene una fila **Asignación** (**Permitida** por defecto, o **Bloqueada**;
se guarda al instante). Bloqueada, asignar es imposible en los issues y pull requests de ese repositorio, lo pida quien
lo pida:

- el selector de asignados se omite en los formularios de issue nuevo y de pull request nueva (con una nota que dice
  por qué), el *Alojamiento: Editar asignados...* de la paleta queda en gris con el motivo, y el menú de la fila lo dice en
  lugar de abrirlo;
- todo cliente rechaza una asignación antes de enviar nada, incluidos los asignados que se pasan al crear un issue o una
  pull request: la CLI responde `{"refused": {"reason": "assignment_blocked", ...}}` con código de salida 2, y un agente
  por MCP recibe el mismo rechazo;
- los asignados que ya existen siguen visibles (solo lectura).

`gitpoulpi hosting assignment` muestra el ajuste, `--block` lo activa y `--allow` lo desactiva de nuevo; solo una
persona puede volver a permitirla (el servidor MCP no puede).

## 9. Desde la línea de comandos

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

Consulta la [referencia de la CLI](cli-reference.md#comandos) para ver todos los comandos. Nunca se acepta
un token como argumento, y el servidor MCP no ofrece
`hosting login`, así que un token nunca pasa por un agente. `hosting login`
imprime `stored: "os"` cuando el almacén de credenciales del sistema guarda el token, o
`stored: "session"` más `store_error` cuando el almacén lo rechazó y el token
vive en memoria solo durante esta sesión.

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
  request desde un fork (consulta [Acciones sobre una fila](pull-requests-and-issues.md#acciones-sobre-una-fila)). Azure
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
- **Conversaciones:** primero se cargan las 300 entradas más recientes (**Cargar anteriores** para el resto); un comentario, una edición o una eliminación publicados desde GitPoulpi son acciones hacia el exterior y no se pueden deshacer desde él; lo que cada host no puede hacer está en [la tabla](conversations.md#qué-no-puede-hacer-cada-host). En Gerrit un mensaje o comentario no se puede editar ni eliminar, y el texto de Gerrit es texto plano.
- Crear una pull request, work item, issue o tarjeta ocurre en el host y
  **no se puede deshacer** desde GitPoulpi. Hacer checkout de una request sí se puede deshacer
  con normalidad.
