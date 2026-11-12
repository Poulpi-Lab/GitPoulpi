[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Integrações de hospedagem e de rastreadores de issues

O GitPoulpi se conecta a estes produtos:

- **Hospedagem** (pull requests, issues, clonagem a partir da sua conta, chaves SSH):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea e Forgejo
  (incluindo o Codeberg). O **Gerrit Code Review** é hospedagem só para revisão: as suas changes são os
  pull requests, ele não tem issues.
- **Rastreadores de issues** (somente issues): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud e Server) e Redmine.
- **Notas Markdown locais** (somente issues, sem token, sem rede): uma pasta de notas Markdown, por exemplo uma pasta de um vault do Obsidian, veja [Issues locais e Obsidian](local-issues.md).

Todas as integrações usam um **token pessoal**. Não existe conta do GitPoulpi
nem login via OAuth. As integrações são opcionais e **ativadas por
repositório**: nada é buscado até você vincular um repositório. Seu token é guardado no
seu auxiliar de credenciais, nunca no `config.toml`. As operações comuns do `git` (fetch, pull,
push) não precisam de uma integração: elas usam as credenciais que o seu `git`
já tem.

## 1. Criar um token no host

O formulário de conexão tem um link *Gerar um token em <host>*. Ele abre a
página de tokens do host com os escopos certos já selecionados. Para fazer manualmente:

| Host | Onde | Escopos |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repositórios, pull requests, issues), `read:user` (quem você é), `admin:public_key` (somente para adicionar ou remover chaves SSH) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, em `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Crie o token **para essa organização**, não para “All accessible organizations”: os tokens globais deixam de funcionar em 2026-12-01. |
| Azure DevOps Server | A mesma página no seu servidor (`https://<server>/<collection>/_usersSettings/tokens`) | Os mesmos dois escopos |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Para chaves SSH, também `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profile picture > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, em `<host>/user/settings/applications` | Leitura de `user`; leitura e escrita de `repository`, `issue` e `user` (este último para chaves SSH). O Gitea não mostra escopos depois, então o GitPoulpi não pode avisar de um escopo ausente: uma chamada recusada diz isso. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. A chave age como você. |
| YouTrack (Cloud e Server) | Seu perfil > Account Security > New token, em `<host>/users/me?tab=account-security` | O escopo `YouTrack`. Um token permanente age como você. |
| Gerrit | Settings > HTTP Credentials > Generate new token (servidores recentes; os mais antigos: Generate new password), em `<host>/settings/#HTTPCredentials` | Nenhum. O token é a **senha HTTP** e age como você; o nome de usuário vai ao lado dele. |
| Redmine | My account > API access key > Show, em `<host>/my/account`. O administrador precisa ter ativado **Administration > Settings > API > Enable REST web service**. | Nenhum. A chave age como você. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (o clássico, sem escopos) | Nenhum. O token age como você. |
| Jira Data Center | Profile > Personal Access Tokens | Nenhum. O token age como você. |
| Trello | Primeiro uma **chave de API** em `trello.com/apps/admin` (crie um Power-Up e depois gere uma chave de API). Em seguida um **token** em `trello.com/1/authorize?...&key=<sua chave>`. | `read`, `write` |

Dê ao token um nome como “GitPoulpi” e uma data de expiração. Copie o token:
o host o mostra apenas uma vez.

## 2. Conectar

1. Pressione `Ctrl+,` e escolha **Integrações**.
2. Escolha o produto à esquerda. Os produtos ficam em dois grupos: **Hospedagem** e
   **Rastreadores de issues**.
3. Pressione **Conectar**. Para adicionar uma segunda conta do mesmo produto, use
   **+ Adicionar perfil**.

   ![A página Integrações: os produtos à esquerda e um botão Conectar para o GitHub](../images/settings-integrations.png)

4. Preencha o formulário e cole o token. O token aparece em texto claro enquanto
   você digita, então confira que ninguém está olhando. O formulário lista os escopos
   de que ele precisa.

   ![O formulário de conexão: um campo Token de acesso pessoal, os escopos repo, read:user e admin:public_key, um link para gerar um token em github.com e os botões Conectar e Cancelar](../images/settings-connect.png)

5. Pressione **Conectar**. O GitPoulpi verifica o token com o host em segundo
   plano. A página então mostra um cartão com seu nome e login,
   *Conectado* em verde, e a seção de chave SSH. Um token sem um dos
   escopos desejados ainda conecta, e os escopos que faltam são listados em vermelho.

No cartão: **Editar** abre o formulário de novo. **Desconectar** exclui o token
e mantém o perfil. **Remover** exclui os dois, após uma confirmação.

Teclado na página Integrações: `Tab` / `Shift+Tab` alternam entre a lista de
produtos, a faixa de perfis e o formulário. `a` adiciona um perfil, `e` o edita,
`d` ou `Delete` o remove, `o` abre a página do token, `Esc` volta um
passo.

### O que cada formulário pede

O link *Gerar um token* só funciona depois que você preenche o host ou a
organização.

- **GitHub, GitLab:** somente o token. **GitHub Enterprise** e
  **GitLab Self-Managed** também pedem o **Domínio do host**
  (`gitlab.example.com`; https, a menos que você digite `http://`). Um nome sem ponto
  (um serviço do Docker ou uma máquina da rede local) precisa de uma porta ou de `http://`:
  `gitea:3000`, `http://nas`. Um servidor auto-hospedado pode
  ser http simples, por exemplo um contêiner local em `http://localhost:8080`. Um token enviado por
  http simples trafega sem criptografia, então use http apenas em uma rede local
  ou confiável.
- **Azure DevOps:** a **Organização**, o nome em
  `dev.azure.com/<organization>`. Um endereço `https://<org>.visualstudio.com`
  também é aceito. Duas organizações precisam de dois perfis. Um repositório usa o
  perfil da organização que o seu remoto indica. O **Azure DevOps Server**
  pede a **URL da coleção**, por exemplo
  `tfs.corp/tfs/DefaultCollection`. O GitPoulpi não usa o auxiliar de
  credenciais do sistema para o Azure DevOps: o Git Credential Manager guarda ali um token
  de login, não um token de acesso pessoal.
- **Bitbucket:** o **E-mail da conta Atlassian** e um token de API. Se o
  e-mail estiver vazio, o token é enviado como bearer token: use isto para um token de acesso
  de workspace ou de repositório. O GitPoulpi encontra seus workspaces a partir do token.
  **Bitbucket Data Center:** a **URL do servidor** e um token de acesso HTTP.
- **Gitea, Forgejo:** o **Domínio do host** e o token. O **Forgejo** começa em
  `https://codeberg.org`: troque-o para um servidor próprio. O Gitea não tem
  padrão público. Um endereço `http://localhost:3000` funciona para um servidor local.
  Os dois tipos compartilham um cliente (o Forgejo é um fork do Gitea com o mesmo
  `/api/v1`); o token é enviado como `Authorization: token <token>`. O
  auxiliar de credenciais do sistema não é usado para eles.
- **Jira Cloud:** a **URL do site** (`acme.atlassian.net`), o **E-mail da conta
  Atlassian** e um token de API clássico. Tokens do Jira com escopo não são
  suportados. **Jira Data Center:** a **URL do servidor** e um token de acesso
  pessoal.
- **Trello:** a **Chave de API** e o **token**. A chave de API é pública, por isso
  fica no `config.toml` (como `account`). O token é secreto e vai para o auxiliar de
  credenciais. O formulário tem dois links: *Obter uma chave de API no Trello* e,
  depois que você digita a chave, *Gerar um token*. A chave e o token são enviados em
  um cabeçalho, nunca em uma URL.

- **Linear:** só a **chave de API**: o Linear tem um único host público, então não
  há endereço a digitar. A chave é enviada como está no cabeçalho `Authorization`
  (o Linear não aceita `Bearer` para chaves pessoais). Para que as referências `ENG-12`
  das mensagens de commit abram a página certa, guarde a URL do perfil com o seu
  espaço de trabalho (`https://linear.app/acme`, pela CLI: `--url`); as próprias linhas de issue
  sempre trazem o endereço exato.
- **YouTrack:** a **URL do site** (`acme.youtrack.cloud` para o Cloud, o seu próprio
  servidor nos outros casos, com o caminho de contexto se ele tiver um, por exemplo
  `host/youtrack`) e um token permanente, enviado como `Authorization: Bearer`. O
  mesmo cliente atende o Cloud e o Server (`<url>/api`).

- **Gerrit:** a **URL do servidor** (com o caminho de contexto se ele tiver um), o seu **Nome de usuário** e a senha
  HTTP como token. Eles viajam como autenticação HTTP básica; o GitPoulpi fala com a API REST `/a/` do
  servidor e ignora a linha `)]}'` com que cada resposta começa. O nome de usuário não é secreto e fica em
  `config.toml` (como `account`), a senha vai para o auxiliar de credenciais. Um repositório é vinculado a um
  *projeto* do Gerrit em PULL REQUESTS (as suas changes são listadas ali); o Gerrit não tem issues, e uma change é criada
  com push (veja *Push para revisão* em [Revisando](review.md#gerrit)).
- **Redmine:** a **URL do site** do seu Redmine (com o subcaminho se ele tiver um,
  por exemplo `host/redmine`) e a sua chave de API, enviada como `X-Redmine-API-Key`. Se o
  Redmine responder 403 ou uma página HTML, o serviço web REST está desligado (peça ao
  administrador, ou ative-o em **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack e Redmine são apenas rastreadores de issues: sem pull requests, sem repositórios para
clonar e sem chaves SSH. O Bitbucket não tem issues (a Atlassian aposentou o Bitbucket
Issues). Vincule a seção ISSUES de um repositório Bitbucket a um projeto Jira
em vez disso.

## 3. Onde o token é guardado

O token é guardado no **auxiliar de credenciais** do seu sistema (Git Credential
Manager no Windows, libsecret ou `pass` no Linux, Keychain no macOS). Ele usa uma
entrada privada que não pode ser confundida com as credenciais de um host real. Ele
nunca é gravado no `config.toml` e nunca é impresso.

Se nenhum auxiliar de credenciais estiver configurado, ou se ele recusar o token, o GitPoulpi
mantém o token **na memória somente durante esta sessão** e avisa você. Conecte
de novo após reiniciar.

O GitPoulpi procura o token de um perfil nesta ordem:

1. A variável de ambiente `GITPOULPI_TOKEN_<PROFILE ID>`. Use-a para fornecer um
   token sem guardá-lo, por exemplo em CI.
2. O token guardado para o perfil (`token_ref`).
3. Somente para perfis do GitHub e do GitLab: o auxiliar de credenciais do sistema para o
   host, quando nenhum outro perfil ativado usa o mesmo host.

Cada perfil é uma entrada `[[integrations]]` no `config.toml`. Veja a
[referência de configuração](config-reference.md#integrations-hospedagem-e-issue-trackers).

## A verificação de segurança

Um token salvo só é tão seguro quanto o endereço para o qual é enviado. Por isso o GitPoulpi lembra, para cada integração, ao que ela estava conectada, e para quando isso muda:

- **O endereço.** Na primeira vez que uma integração é usada (ou conectada), o endereço dela é registrado em `trust.toml`, na pasta de dados privada só do usuário (`<data>/private/`), não no `config.toml`: editar a configuração nunca consegue ampliar o que é confiável. Se o endereço na configuração não é mais o registrado, nada é enviado a ele e o token nem é lido. Grafias de um mesmo endereço (`HTTPS://Git.Example.com:443/`) são o mesmo endereço; um host parecido não é.
- **A conta.** Depois que você conecta um token, e depois cerca de uma vez por dia (e uma vez por execução para um token que vem de `GITPOULPI_TOKEN_*` ou do auxiliar de credenciais), o GitPoulpi pergunta ao host a quem o token pertence e compara o **id** da conta, não o login. Um token que agora pertence a outra pessoa é barrado.
- **O certificado.** Servidores cujo certificado não se encadeia a uma autoridade conhecida (uma CA privada, um servidor autoassinado) são aceitos só para a chave exata que você aceitou, e o handshake TLS ainda prova que o servidor tem essa chave. Com a fixação estrita ativada, a chave precisa coincidir mesmo quando o certificado se encadeia a uma raiz pública; o preço é que toda rotação de chave pergunta de novo, e por isso ela é opcional. Os hosts públicos (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) não guardam registro de certificado.
- **http simples.** Um endereço `http://` que não seja loopback é recusado, a menos que você o tenha permitido para aquela integração.

Toda requisição também fica restrita ao endereço da própria integração (e, para um host público, ao host de API dele, como `api.github.com`): uma requisição para qualquer outro lugar é recusada antes de ser enviada, e um redirecionamento para outro endereço não é seguido.

**As integrações conectadas antes de existir esta verificação** recebem o registro em silêncio, a partir da configuração como estava, na primeira vez que são usadas depois da atualização: elas mantêm o nível de confiança de hoje e nada pergunta.

### Pela linha de comando

Um comando barrado falha com código de saída 2 e imprime o que mudou (veja a [referência da CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` mostra o que é confiável; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` e `--allow-insecure-http on|off` aceitam uma coisa cada, nomeando o valor exato. Depois de aceitar um novo endereço, digite o token de novo. Desconectar (`hosting logout`) ou remover uma integração exclui o token dela, o registro de confiança e o que foi guardado em cache a partir dela (as listas de repositórios salvas do diálogo Clonar, as listas de conta do Radar e os rascunhos de revisão remota do host dela).

Um agente (MCP) não consegue aceitar nada: ele é avisado de que a verificação de segurança precisa de você.

### Na interface do terminal

Uma requisição barrada abre o diálogo **Verificação de segurança da integração** sobre o que você estava fazendo. Ele diz o que mudou (o endereço antigo e o novo, a conta antiga e a nova com seus ids, ou o certificado antigo e o novo como uma pequena tabela com a impressão digital SHA-256 em grupos de quatro, o assunto, o emissor e a validade) e que o token não foi enviado. As respostas são numeradas:

1. **Desconectar** exclui o token salvo, o registro de confiança e o que foi guardado em cache a partir da integração.
2. **Confiar** no novo endereço, conta ou certificado (para um endereço sem criptografia: **Permitir http simples**).
3. **Cancelar** não muda nada.

Nenhuma resposta fica destacada até você pressionar uma seta ou um dígito, então um Enter perdido não aceita nada. Esc e o `✕` de fechar cancelam; um clique fora do diálogo não faz nada. Um corpo longo rola com a roda do mouse, Page Up/Page Down ou a barra de rolagem. Várias verificações entram na fila (uma por integração e tipo de mudança) e são perguntadas uma depois da outra. Uma verificação cancelada não reaparece enquanto não mudar; **Integrações: revisar verificação de segurança** na paleta de comandos a traz de volta.

Depois que você confia em um novo **endereço**, o formulário da integração abre com o campo do token em foco e a dica "Recomenda-se digitar o token de novo": o token salvo nunca foi enviado para lá. No formulário, um endereço alterado de propósito é aceito junto com um token digitado para ele; sem um token, o formulário pede um.

O formulário também tem duas chaves. **Fixação estrita de certificado** (para um endereço https) fixa a chave do certificado que o servidor apresenta quando você conecta. **Permitir http simples (sem criptografia)** só é oferecido para um endereço `http://` que não seja esta máquina, com um aviso vermelho de que o token trafega em texto claro. O cartão conectado mostra a impressão digital fixada.

### O que o token pode fazer, e até quando

Conectar também pergunta ao host o que o token pode fazer. O GitHub (tokens clássicos), o GitLab e o Bitbucket (quando envia os escopos) informam seus escopos: o cartão mostra uma linha amarela "Este token tem mais permissões do que o GitPoulpi precisa" listando os escopos além do que o GitPoulpi usa, os sensíveis (como `delete_repo`, `admin:org`, `sudo`) em vermelho, com o conselho de preferir um token granular com data de validade. Um token clássico do GitHub (`ghp_...`) recebe esse conselho mesmo quando os escopos são exatos. Um token granular do GitHub não pode ser consultado sobre suas permissões, então só a validade dele é mostrada. O cartão mostra a data de validade quando o host a informa. Um token sem validade não mostra lembrete, e os outros produtos não informam escopos e não geram aviso. Os mesmos campos estão no JSON de `hosting integrations login` e `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Eles são lidos quando você conecta, então o cartão os mostra para a sessão que conectou.

## 4. Vincular um repositório

A vinculação é explícita e por repositório. Quando um perfil está conectado, a barra
lateral Refs mostra uma seção **PULL REQUESTS** e uma seção **ISSUES** depois de TAGS.
Cada seção tem dois menus suspensos: o provedor e o projeto.

- O menu de provedor **propõe** as contas que correspondem aos remotos do
  repositório (`origin` primeiro), em linhas como `owner/name · origin · GitHub`. Uma
  escolha define o provedor e o projeto.
- Nada é vinculado automaticamente. Jira, Trello, Linear, YouTrack e Redmine nunca são propostos, porque
  nenhum remoto os nomeia: escolha-os você mesmo.
- **Nenhum (desvincular este repositório)**, no topo do menu, desvincula a
  seção. Nada a vincula de novo por conta própria.
- O vínculo é salvo no arquivo `.gitpoulpi/repo-state.toml` do
  repositório.

As duas seções são vinculadas **separadamente**. Por exemplo, os pull requests podem
vir do Azure DevOps e as issues do Jira.

| Provedor | Projeto de PULL REQUESTS | Projeto de ISSUES |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | o mesmo |
| Azure DevOps | `Project/Repo` (a organização vem do perfil) | `Project` (work items) |
| Gitea, Forgejo | `owner/name` | o mesmo |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | nenhum: vincule o Jira |
| Jira | nenhum | a chave do projeto, por exemplo `ABC` |
| Trello | nenhum | um quadro |
| Linear | nenhum | a chave de uma equipe, por exemplo `ENG` (as equipes são listadas) |
| YouTrack | nenhum | o nome curto de um projeto, por exemplo `ABC` |
| Gerrit | o nome do projeto do Gerrit, por exemplo `team/tool` | nenhum |
| Redmine | nenhum | o identificador de um projeto, por exemplo `bench` (a última parte do endereço do projeto) |

A lista de projetos de ISSUES permite busca. Você também pode digitar um projeto que a
lista não mostra.

**Consulta extra.** Uma origem de issues tem uma linha de filtro opcional abaixo do menu
de projeto. Ela é combinada (AND) com o projeto:

- Jira: um fragmento de **JQL** (`component = UI AND labels = backend`).
- Azure DevOps: uma condição **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack: uma **consulta do YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Ela é envolvida em parênteses, então um `or` não pode ampliar o projeto. Um
  `sort by:` seu substitui o padrão `sort by: updated desc`.
- Redmine: nenhuma (os filtros de estado e de "atribuídas a mim" da seção bastam).
- Linear: nenhuma (os filtros de estado e de "atribuídas a mim" da seção
  bastam).
- Trello: um menu em que você marca as **listas** a mostrar. Sem nenhuma marcada, mostra todas as
  listas abertas.
- GitHub e GitLab a ignoram.

`Enter` mantém uma consulta digitada, `Esc` a descarta. Paleta:
*Issues: editar consulta / listas...*.

Pela CLI: `gitpoulpi hosting link pulls --id <profile>` (ou `issues`),
`--clear` para desvincular, `--filter` para a consulta extra. Veja a
[referência da CLI](cli-reference.md#comandos).

### Clonar de uma conta conectada

Um perfil conectado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) também é uma fonte no diálogo **Clonar** (o botão **Clonar** da aba Início, ou `c` nela). Cada perfil aparece em **Clonar por URL** como uma fonte própria (`GitHub · name`, `GitLab · name`); GitHub e GitLab nunca se misturam. Escolha um e os repositórios dele são carregados, agrupados por proprietário: primeiro a sua própria conta, depois uma seção por organização ou grupo (`quarry/tools` para um subgrupo do GitLab), cada uma sob um cabeçalho com a contagem. Um cadeado marca um repositório privado.

![O diálogo Clonar com uma conta do GitHub conectada: uma seção por proprietário e um cadeado em um repositório privado](../images/clone-hosted.png)

A caixa de busca filtra em todas as seções (nome, caminho completo e descrição); uma seção sem correspondência desaparece. `Enter` em uma linha preenche o clone com a URL HTTPS desse repositório e um nome de pasta.

![O diálogo Clonar com a busca reduzindo a lista a uma organização](../images/clone-hosted-search.png)

A lista de um perfil que você já abriu antes aparece na hora, a partir de uma cópia salva,
enquanto o host é consultado de novo (*atualizando...* ao lado do rótulo); quando a
resposta chega, os repositórios novos aparecem e os que sumiram desaparecem, e o
repositório destacado e o escolhido continuam onde estão. Se a atualização falhar, a
lista salva continua com uma nota, e o motivo está no Registro de atividade.

Se o host recusar o token, o diálogo mostra *não foi possível listar os repositórios* com o motivo no lugar da lista (e a cópia salva é apagada); as outras fontes continuam funcionando.

## 5. Pull requests e issues na barra lateral

### PULL REQUESTS

A seção tem um cabeçalho com a contagem e um `+` (criar um pull request),
os menus de provedor e de projeto, uma caixa de busca e um menu de estado (*Abertos*,
*Merged / fechados*, *Todos*). Depois vêm quatro grupos, cada um com sua contagem:

- **Meus pull requests**: você é o autor.
- **Atribuídos a mim**.
- **Aguardando minha revisão**: você é um revisor solicitado.
- **Todos os pull requests**: recolhido no início.

A caixa de busca filtra todos os grupos (título, número, autor, branch). Cada
mudança no menu de estado consulta o host de novo.

Uma linha começa com um glifo de estado: `●` aberto (verde), `○` rascunho, `◆` merged
(roxo), `✕` fechado (vermelho). O pull request da branch em checkout fica em negrito.
As linhas aparecem como `#12  title` (GitHub, Bitbucket) ou `!53  title` (GitLab,
Azure DevOps).

### ISSUES

O título da seção nomeia o provedor: `ISSUES DO GITHUB`, `ISSUES DO JIRA`, e assim
por diante. Fica `ITENS DE TRABALHO DO ...` para o Azure DevOps e `CARTÕES DO ...` para o Trello.
Abaixo dos menus: uma caixa de busca, o menu de estado (*Abertas*,
*Atribuídas a mim*, *Fechadas*, *Todas*) e as issues. Uma linha do Jira aparece como
`ABC-12  title`, com o tipo e o status esmaecidos quando a barra lateral é larga
o bastante.

### Ações em uma linha

- `Enter` ou um clique duplo abre a linha no seu navegador.
- Clique com o botão direito, ou `.`, abre o menu da linha:
  - **Abrir no navegador**, **Copiar URL**, **Copiar número**.
  - Para um pull request: **Editar no <provedor>**, **Checkout da branch de origem**,
    **Ver commits da branch de origem**. Para um pull request vindo de um fork, o checkout aparece como
    **Checkout de pull request (fork)**, e **Ver** fica acinzentado.
  - Para uma issue: **Criar branch a partir da issue**. Ele abre o prompt de nova
    branch com um nome como `12-fix-the-thing` (ou `ABC-123-fix-the-thing` para
    o Jira).
  - No GitHub, GitLab, Gitea e Forgejo, uma linha de pull request também tem **Revisar...**
    (também **Enter** na linha): a revisão no aplicativo, veja
    [Revisando um pull request](review.md).
  - No GitHub, GitLab, Gitea e Forgejo, as duas linhas também têm **Editar responsáveis...**,
    **Editar rótulos...** e **Definir marco...** (e **Editar revisores...** em
    um pull request): veja
    [Responsáveis, revisores, rótulos e marco](#responsáveis-revisores-rótulos-e-marco).
- **Editar no <provedor>** tem outras três formas de acesso: o lápis `✎` na
  extremidade direita de uma linha de pull request com o mouse em cima ou selecionada, `e` na linha selecionada
  e o comando da paleta *Integrações: editar pull request vinculado no host*
  (para o pull request da branch em checkout).
- Se a branch de origem ainda não estiver no repositório, o GitPoulpi avisa. Faça
  fetch primeiro (`Ctrl+G f`).

**Checkout de um pull request.** O GitPoulpi busca o pull request em uma branch local
`pr/<n>` (`mr/<n>` no GitLab) que o acompanha, e muda para ela. No GitHub, no
GitLab, no Gitea e no Forgejo isto funciona também para forks (o GitPoulpi busca a ref head especial do
host). O Azure DevOps e o Bitbucket buscam a branch de origem do
pull request. Um pull request vindo de um fork é recusado ali: adicione o fork como remoto e
faça checkout da branch dele você mesmo. Desfazer volta para a branch anterior. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Revisando um pull request pela linha de comando (e um agente).** No GitHub
(incluindo o Enterprise), no GitLab (incluindo o self-managed), no Gitea e no Forgejo um pull request pode ser
lido e revisado sem sair do GitPoulpi: `hosting review show <n>` (arquivos,
estado do merge, verificações), `hosting review threads <n>` (threads de revisão e
conversa), `hosting review comment`, `hosting review reply`, `hosting review resolve`
e `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. O diff vem do seu repositório, não do
host: `hosting review fetch <n>` faz fetch do head do pull request em
`refs/gitpoulpi/pr/<n>` e da sua branch de destino em `refs/gitpoulpi/pr/<n>-base`
(nenhuma das duas é uma branch, nada vai para checkout). Os números de linha são os desse
diff: `new` é a linha no head, `old` a linha na base do merge. O GitLab
não tem o estado de revisão "request changes": ele é recusado lá, sem enviar nada.
O Gitea e o Forgejo aceitam um comentário em uma linha só (sem intervalos); uma thread são os
comentários de uma linha de um arquivo. O Gitea pode resolver uma thread, o Forgejo 11 não
(o host não tem um endpoint para isso: a chamada diz isso). Uma resposta em um servidor
sem o endpoint de resposta é publicada como uma revisão de um comentário na mesma linha.
O Gitea recusa aprovar ou rejeitar o seu próprio pull request.
Sem um clone: acrescente `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` e `suggest` para revisar um projeto que não é o repositório atual,
`hosting review diff <n> --project P` lê o diff (hunks por arquivo) pela API do host, e
`hosting file <path> --project P [--ref R]` lê um arquivo em uma revisão (GitHub, GitLab, Gitea, Forgejo e Gerrit; no Gerrit o diff de um
patch set é lido arquivo por arquivo e o arquivo vem da branch ou, para um id de commit, do commit). Um arquivo que o host não
mostra no diff (grande demais) é marcado como `omitted`.
As ferramentas MCP são os mesmos comandos (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Responsáveis, revisores, rótulos e marco

No GitHub (incluindo o Enterprise), no GitLab (incluindo o self-managed), no Gitea e no
Forgejo, você pode mudar a quem um pull request ou issue é atribuído, quem o revisa, os seus rótulos
e o seu marco sem sair do GitPoulpi. Os outros provedores não mostram nada disso.

- **Menu da linha**: **Editar responsáveis...**, **Editar revisores...** (só pull
  requests), **Editar rótulos...**, **Definir marco...**.
- **Paleta**: *Hospedagem: editar responsáveis...*, *Hospedagem: editar revisores...*,
  *Hospedagem: editar rótulos...*, *Hospedagem: definir marco...* listam cada uma primeiro os
  pull requests e as issues da barra lateral para você escolher. No
  [Radar](radar.md) as mesmas quatro agem sobre a linha selecionada.
- O seletor abre na hora e carrega do host as pessoas ou os rótulos do projeto (e o que
  o item tem agora). Uma linha com indicador de progresso aparece após cerca de 150 ms.
  Digite para filtrar (correspondência aproximada), `Cima` / `Baixo` movem, `Espaço` marca ou
  desmarca, `Enter` aplica, `Esc` cancela, `Tab` chega aos botões. Um clique
  em uma linha a marca, e a roda rola. As pessoas mostram um selo com iniciais,
  os rótulos a sua cor.
- O marco é uma lista suspensa (com **Sem marco** para limpá-lo).
- Depois de **Aplicar** o host é atualizado, e as listas e o Radar mostram a
  alteração após a próxima atualização, que começa logo em seguida.

Os formulários de criação (um pull request, uma issue) têm as mesmas escolhas como uma fileira de
botões: `Responsáveis: nenhum`, `Revisores: nenhum`, `Rótulos: nenhum`,
`Marco: nenhum`. No GitHub, GitLab, Gitea e Forgejo o botão Rótulos substitui o campo de
rótulos digitados do formulário de issue. Se um host aceitar o novo pull request mas
recusar alguns dos seus rótulos ou pessoas, a linha de status avisa.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, e `--assignee`, `--reviewer`, `--label`,
`--milestone` em `hosting pr` e `hosting issue create`; veja a
[referência da CLI](cli-reference.md).

### Teclas nas seções

| Tecla | Ação |
|---|---|
| `Enter` / clique duplo | abre a linha no navegador |
| `.` / botão direito | menu da linha |
| `e` | edita o pull request selecionado no host |
| `n` | novo pull request (PULL REQUESTS) ou nova issue (ISSUES) |
| `v` | abre o menu de estado |
| `r` | atualizar |
| `Enter` / `Space` em um cabeçalho | recolhe ou expande a seção |

### Reconectar e offline

Quando o token está ausente, foi rejeitado ou não tem um escopo, a primeira linha da
seção diz `Reconectar (token rejeitado)`. Clique nela, ou pressione `Enter`, para abrir
o perfil em Configurações > Integrações.

Quando o host não pode ser alcançado, as últimas listas permanecem na tela e o cabeçalho
mostra a idade delas (`· 5m de idade`). A última resposta também fica em cache no disco
(`.gitpoulpi/hosting-cache.json`), então as listas aparecem antes de a rede
responder.

## 5b. O quadro de issues

O botão **Quadro** do cabeçalho de ISSUES (paleta: **Issues: abrir quadro**) mostra as issues da origem vinculada como um quadro
Kanban sobre a janela. As colunas dependem da origem:

| Origem | Colunas | Mover um cartão |
|---|---|---|
| Notas Markdown locais | os status (`statuses` do perfil, depois qualquer outro em uso) | reescreve o `status` da nota |
| GitHub, GitLab, Gitea, Forgejo | **Aberto**, depois os rótulos que você escolheu, depois **Fechado** | troca o rótulo de status; para **Fechado** ele fecha a issue, para uma coluna de novo ele a reabre |
| Trello | as listas abertas do quadro, da esquerda para a direita | move o cartão para a lista |
| Redmine | os status da issue | muda o status da issue (valem as regras de fluxo de trabalho do Redmine) |
| Jira (Cloud e Data Center) | os status do projeto, o de a fazer primeiro, o de concluído por último | executa a transição de fluxo de trabalho que leva à coluna (`GET` e depois `POST .../transitions`); sem essa transição o cartão volta e a linha de status diz `no transition from X to Y` |
| Azure DevOps (Services e Server) | os estados dos tipos de work item do projeto, por categoria de estado | define `System.State`; um work item cujo tipo não tem o estado da coluna volta, com o motivo (`a Bug has no state Done`) |

Em um projeto do GitHub ou do GitLab, **Colunas** (ou `c`, ou **Quadro: definir rótulos das colunas...**) pede os rótulos que são as
colunas, em ordem, por exemplo `status::todo, status::doing, status::review`. Ele propõe os rótulos do projeto que
parecem status. A escolha é salva para o repositório. Sem rótulos o quadro é Aberto / Fechado. Mover um cartão
só toca nos rótulos de status que você listou: os outros rótulos dele ficam. Um quadro de host lista as issues abertas e as 60
fechadas mais recentemente.

Uma transição do Jira que pede campos obrigatórios (uma resolução, por exemplo) é recusada pelo Jira e o cartão volta com
a própria mensagem do Jira: faça essa no site.

O mesmo quadro está disponível para scripts e agentes: `gitpoulpi hosting board` imprime as colunas com seus cartões e
`gitpoulpi hosting board move <cartão> <coluna>` move um (veja [a referência da CLI](cli-reference.md)).

Um cartão mostra o número, o título, os responsáveis como iniciais, os rótulos e o marco. Arraste-o para outra coluna,
ou `Shift+Left` / `Shift+Right`, ou **Quadro: mover cartão para...** (`m`). Ele se move na hora; após cerca de 150 ms um indicador de progresso
diz que o host ainda está trabalhando, e se o host recusar, o cartão volta e a linha de status diz por quê. `Enter` abre
a issue no host (a nota, para issues locais); `e` edita responsáveis, rótulos e marco; `n` inicia uma nova issue na
coluna selecionada; `a`, `l`, `M` e `/` filtram por responsável, rótulo, marco e texto, `x` limpa os filtros; `r`
lê o host de novo. O quadro lê de novo quando as listas da barra lateral são atualizadas. O Radar não tem quadro.

## 6. Criar um pull request

Abra o formulário com o `+` no cabeçalho de PULL REQUESTS, `n` na seção, ou
o comando da paleta *Pull requests: criar pull request...*. Um menu de branch
também tem **Iniciar pull request a partir de <branch>...**.

O formulário tem:

- Uma aba por perfil conectado que tenha pull requests (`Left` / `Right` na
  faixa de abas).
- Caixas de combinação **Repo de origem** e **Repo de destino**, cada uma com a sua **Branch**. Listas longas
  permitem busca. Você pode digitar uma branch ou um repositório que as listas não mostram.
- **Título**, preenchido com o assunto do commit da ponta.
- **Descrição**, em várias linhas (`Enter` adiciona uma nova linha, `Tab` sai do
  campo).
- Botões **Responsáveis**, **Revisores**, **Rótulos** e **Marco** (GitHub,
  GitLab, Gitea e Forgejo), que abrem o seletor.
- **Enviar como rascunho**.
- Com a [IA](ai.md) ligada: **Gerar título e descrição**.

O formulário abre no perfil, repositório e branches da branch em checkout
(`feature -> main`). Um repositório de origem diferente cria um pull request
a partir de um fork.

**Criar pull request** (ou `Ctrl+S`) cria o pull request no host. O
resultado abre no seu navegador e a linha de status o nomeia.

## 7. Criar uma issue

Quando a seção ISSUES tem um provedor **e** um projeto, o cabeçalho dela tem um
`+` (*Nova issue*). Também: `n` na seção, ou o comando da paleta
*Hospedagem: nova issue...*.

O formulário tem **Título** (obrigatório), **Descrição** (várias linhas) e
**Labels** (separadas por vírgula, por exemplo `bug, ui`; no GitHub, GitLab, Gitea e Forgejo os
botões Rótulos, Responsáveis e Marco abrem o seletor no lugar). Jira e Azure DevOps
adicionam um menu **Tipo**. O Trello adiciona um menu **Lista**. O Linear e o YouTrack
adicionam um menu **Status** (o estado do fluxo de trabalho, ou o valor `State`, em que a issue
começa). O Redmine adiciona um menu **Tipo** com os trackers do projeto (Bug, Feature,
Support...); ele não tem rótulos, então não há botão Rótulos para ele e um rótulo
dado na linha de comando é recusado. O formulário preenche esses
menus a partir do host quando abre.

**Criar issue** (ou `Ctrl+S`) a cria no host. `Esc` ou **Cancelar**
fecha o formulário. Se o host recusar (token rejeitado, escopo ausente, projeto
desconhecido, label desconhecida), o formulário mostra o motivo e mantém o que você digitou.
Em caso de sucesso, a linha de status nomeia a nova issue e a seção ISSUES é
atualizada.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## 8. Criar na própria página web do host

O formulário Nova issue e o formulário Criar pull request têm um botão
**Criar no <provedor>**, no canto superior direito. A tecla é `Alt+W`. Ele abre o
formulário web do host no seu navegador, preenchido com o que você digitou até agora. O
diálogo continua aberto. Ele não precisa de token e não faz chamada de API, então também funciona
depois de um erro de API.

- O GitHub não tem parâmetro de URL para *rascunho*, então a caixa de rascunho não é
  levada. O GitLab recebe o prefixo `Draft: ` no título.
- O GitLab aceita labels apenas por id, então elas são adicionadas como uma ação rápida
  `/label ~"x"` no fim da descrição.
- Uma descrição muito longa é cortada para que a URL fique abaixo de 6000 caracteres,
  com uma nota `[...truncated: paste the rest]`.
- O Bitbucket Data Center não tem endereço de formulário web, então não tem o botão
  **Criar no <provedor>**.

CLI: `gitpoulpi hosting issue create ... --web` e `gitpoulpi hosting pr --web`
imprimem a mesma URL.

## 9. O chip da barra de ferramentas

Na extremidade da barra de ferramentas (à direita por padrão), um chip mostra o pull request da branch
em checkout: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) ou
`BB #12` (Bitbucket). A cor mostra o estado: verde aberto, cinza rascunho,
roxo merged, vermelho fechado.

- Passe o mouse para ver o título, o estado e `source → target`.
- Clique nele, ou pressione `Ctrl+G g`, para abrir o pull request no navegador.
- Clique com o botão direito para **Abrir no navegador**, **Copiar URL**, **Atualizar** e
  **Trocar de perfil**.

Uma branch enviada sem pull request mostra um chip discreto `+ PR`. Clique nele para iniciar um
pull request. Não há chip quando nenhum perfil corresponde, quando a branch não
foi enviada, ou quando o host não pôde ser alcançado. Em uma janela estreita o chip fica
oculto para dar espaço aos botões da barra de ferramentas.

## 10. Quando as listas são atualizadas

As listas são atualizadas:

- quando você abre um repositório;
- quando a branch em checkout muda;
- após um push ou um fetch;
- a cada *Atualização de Pull Requests e Issues (minutos)*
  (`general.hosting_refresh_minutes`, padrão 5, `0` desliga o temporizador;
  Configurações > Integrações);
- quando você fecha as Configurações;
- quando você pressiona `r`, executa *Integrações: atualizar pull requests e issues* ou
  muda um filtro.

As requisições nunca bloqueiam a janela.

Os hosts limitam quantas chamadas você pode fazer. O GitPoulpi reenvia o `ETag` da
última resposta, então uma lista sem mudanças custa pouco (nada no GitHub). Quando um
host informa que restam poucas chamadas, a linha de status avisa. Quando não resta nenhuma,
o GitPoulpi deixa de consultar esse host até o limite ser redefinido, e as listas
ficam como estão.

## 11. Terminologia

GitHub, Azure DevOps e Bitbucket dizem “pull request”. O GitLab diz “merge
request”. *Terminologia de solicitação de revisão* (`ui.pr_terminology`, em
Configurações > Integrações) escolhe a palavra para cada rótulo, menu, comando
da paleta e mensagem:

- **Auto (por host)**: segue o provedor vinculado do repositório (“Pull
  Request” enquanto nada estiver vinculado).
- **Pull Request** ou **Merge Request**: sempre essa palavra.

A mudança vale na hora. A CLI e o MCP sempre dizem “pull/merge request”.
Esta página diz “pull request”.

## 12. Referências em mensagens de commit e nomes de branch

Quando as issues ou os pull requests do repositório estão vinculados, as referências na
mensagem de commit dos Detalhes do commit ficam sublinhadas. Um clique abre a
página vinculada. Teclado: o comando da paleta
*Integrações: abrir referência do commit (#issue / !solicitação)...* (um menu quando a
mensagem tem várias referências). Um repositório sem nada vinculado não tem
links.

O que conta como referência depende do provedor da seção ISSUES:

| ISSUES vinculadas a | Em uma mensagem | Em um nome de branch |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` e `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, somente quando `ABC` é a chave do projeto vinculado. Textos como `UTF-8`, `SHA-256` ou `ISO-8601` nunca são um link. | `ABC-123` em qualquer ponto do nome (`feature/ABC-123-x`), em maiúsculas |
| Linear | `ENG-123`, somente quando `ENG` é a chave da equipe vinculada (ou de uma equipe da lista de equipes do perfil). `UTF-8` nunca é um link. | O próprio `branchName` do Linear quando você cria a branch a partir de uma issue; `ENG-123` em qualquer ponto do nome é lido de volta |
| YouTrack | `ABC-123`, somente quando `ABC` é o nome curto do projeto vinculado (ou de um da lista de projetos do perfil) | `ABC-123` em qualquer ponto do nome |
| Redmine | `#123`, somente quando o Redmine é a origem de ISSUES vinculada (uma origem de pull requests mantém o seu próprio sigilo) | `123-fix-login` |
| Trello | um endereço `https://trello.com/c/<shortLink>`; `#12` somente quando um cartão em cache tem o número 12 | `12-fix-login` |

| PULL REQUESTS vinculados a | Em uma mensagem |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Quando as duas listas correspondem a `#N`, a issue vence. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

### Inserir uma referência de issue

No compositor de commit, o botão `#▾` na extremidade direita do Resumo abre
uma lista filtrável das issues vinculadas. A issue indicada pela branch atual
vem primeiro, marcada com *(da branch)*. `Enter` ou um clique insere a referência dela
(`ABC-123`, `#12`, `AB#12`) no cursor. Teclado: `Alt+R` no compositor.
Paleta: *Commit: Inserir referência de issue...*. Sem uma lista de issues vinculada, o
botão fica acinzentado e diz *vincule primeiro uma lista de issues na barra lateral*.
O GitPoulpi nunca adiciona uma referência por conta própria.

## 13. Chaves SSH

### Chave SSH por perfil

Um perfil conectado tem uma seção **Chave SSH** abaixo do cartão. Sem chave,
escolha uma destas:

- **Chave padrão**: seu ssh-agent ou `~/.ssh`. Nada muda.
- **Gerar nova chave**: um título opcional (padrão
  `GitPoulpi <host> <date>`). Com **Adicionar a chave pública a <host>** marcado,
  o GitPoulpi a envia, se o host ainda não a tiver. A chave é
  gravada sem senha em `~/.ssh/gitpoulpi_<profile id>_ed25519` e
  `.pub`. Não precisa de `ssh-keygen`. Um arquivo existente nunca é sobrescrito.
- **Chave própria**: digite um caminho ou use **Procurar** para um arquivo de chave privada,
  depois pressione **Usar esta chave**.

Quando uma chave está associada, a seção mostra `title (SHA256:...)` com:

- **Remover**: desvincula a chave, e a exclui no host se o GitPoulpi a adicionou.
- **Excluir**: o mesmo, e também apaga os arquivos da chave, após uma confirmação
  que lista os dois caminhos.

### Chave SSH para um repositório

Para fazer um repositório usar uma chave em particular, abra o menu da aba (`Ctrl+T`, ou clique com o
botão direito na aba) e escolha **SSH e credenciais...**, ou execute o comando da paleta
*Repositório: SSH e credenciais...*, e escolha **Arquivo de chave** (ou **Chave da conta** para a chave
de um perfil de integração) em SSH. A escolha fica na pasta privada do GitPoulpi: nada é gravado no
repositório nem na configuração do git. Veja
[Configurações](settings.md#específico-do-repositório-ssh-e-credenciais).

Um repositório que ainda tem um `core.sshCommand` próprio, vindo da antiga fixação "Chave SSH deste
repositório", o mostra nessa página, somente leitura, com **Mover para o GitPoulpi**.

CLI e MCP: `gitpoulpi repo-auth` (a escolha por repositório) e, para a própria fixação de
`core.sshCommand`, `gitpoulpi ssh-command show | set --key <file> | clear`.

A chave global em Configurações > SSH e credenciais (*Caminho da chave privada*, ou
*Gerar par de chaves*) vale para todo repositório sem escolha própria.
O arquivo de chave ou a fixação de um repositório prevalece sobre ela.

## 13b. Proteção de branch, para uma reescrita de histórico

Quando você reescreve o histórico ([o painel](history-rewrite.md)), o GitPoulpi pergunta ao host
quais branches ele protege, para que elas fiquem de fora da reescrita por padrão. Ele pergunta só
para um repositório que você **vinculou** (a seção de pull requests ou issues da barra lateral, ou
um perfil fixado), com o token desse perfil, e um token que não consegue ler isso apenas dá
"desconhecido": nada falha. O que cada provedor responde, e o que o token precisa:

| Host | O que é lido | O que o token precisa |
|---|---|---|
| GitHub (e Enterprise Server) | a flag `protected` das branches e os rulesets de branch ativos (padrões `~DEFAULT_BRANCH`, `~ALL` e `refs/heads/...`) | um token clássico com `repo` (`public_repo` para um repositório público), ou um token granular com Metadata (leitura). Um ruleset que o token não consegue ler é informado, e as regras clássicas continuam valendo |
| GitLab (gitlab.com e self-managed) | `protected_branches` (nomes e padrões `*`) | `read_api`, e uma função que possa ver as configurações do repositório |
| Azure DevOps (Services e Server) | as políticas de branch ativas e bloqueantes do repositório (escopos `Exact`, `Prefix` e `DefaultBranch`) | Code (leitura). A permissão de "force push" é uma configuração de segurança e não é lida |
| Bitbucket Cloud | as restrições de branch que impedem um push (`push`, `force`, `delete`) | `repository:admin`. Uma restrição no modelo de ramificação não é resolvida |
| Bitbucket Data Center | as permissões de branch que impedem um force push (`fast-forward-only`, `pull-request-only`, `read-only`) | administrador do repositório |

A página de configurações da proteção (e da branch padrão, e da exclusão do repositório) é montada
a partir da URL do remoto do repositório para GitHub, GitLab, Azure DevOps e Bitbucket, mesmo
quando o repositório não está vinculado, e é mostrada com os três caminhos quando o host recusa
um push.

## 13c. Listas de toda a conta no Radar

O Radar normalmente observa os repositórios que você clonou. Um perfil também pode
listar o que lhe diz respeito em todo o host, clonado ou não, com três chaves na
sua tabela `[integrations.radar]` (todas desativadas por padrão; veja a
[referência de configuração](config-reference.md#integrationsradar-listas-de-toda-a-conta)):

- **solicitações de revisão**: solicitações abertas que pedem a sua revisão;
- **solicitações abertas**: todas as solicitações abertas do seu próprio namespace e de cada org ou
  grupo de que você é membro, ou dos namespaces que você indicar. Nunca o host inteiro;
  os **Namespaces** do perfil (um por linha em Configurações > Integrações > Radar, opcionalmente
  como expressões regulares) limitam as três listas: uma linha corresponde ao caminho completo de um
  repositório quando é igual a ele ou é um prefixo de caminho em um `/` (`ptm` mantém `ptm/produits/x`,
  não `ptmx/y`), e é aplicada às linhas que o host devolve, mais no servidor onde a busca do host
  permite (os qualificadores `user:` / `repo:` do GitHub);
- **issues atribuídas**: issues abertas atribuídas a você.

GitHub, GitLab (inclusive auto-hospedado), Gitea, Forgejo, Azure DevOps (Services
e Server), Bitbucket Data Center e Gerrit respondem. Como cada um é consultado:

| Host | Solicitações | Issues |
|---|---|---|
| Azure DevOps | pull requests ativos da organização inteira (`searchCriteria.reviewerId` para os que aguardam você; uma solicitação em que você já votou fica de fora); quando o servidor recusa a rota no nível da organização, os 20 primeiros projetos são consultados um a um. Os `namespaces` são nomes de projeto | itens de trabalho atribuídos a você que não estão Done, Closed ou Removed |
| Bitbucket Data Center | o seu painel: papel `REVIEWER` ainda sem aprovação para solicitações de revisão, `AUTHOR` e `PARTICIPANT` para solicitações abertas; os `namespaces` são chaves de projeto | nenhuma (o Data Center não tem issue tracker; uma nota diz isso) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; as solicitações abertas são `is:open projects:<prefix>` por namespace (um prefixo de caminho de projeto), ou as suas changes e as que você revisou quando nenhum é indicado; uma change em andamento (work in progress) é um rascunho | nenhuma |

O Bitbucket Cloud não tem linhas de toda a conta: ele não oferece nenhum endpoint que liste
solicitações entre repositórios, e consultar cada repositório seriam chamadas demais. Os seus
repositórios clonados mantêm as próprias listas, e `gitpoulpi hosting
radar --id <bitbucket>` diz isso. Quando o Gerrit tem mais changes do que o limite que
devolve, a lista é marcada como cortada com uma nota.

Os rascunhos ficam de fora, a menos que você os mantenha. Cada lista tem limite: 300 linhas por tipo, com uma
nota dizendo quantas a mais havia, e no GitHub algumas chamadas de busca por
rodada, porque a API de busca dele permite só 30 por minuto. Um namespace que o token
não consegue ler vira uma nota e os outros continuam carregando; um token recusado ou um
limite de requisições faz falhar só aquele perfil. As respostas são reaproveitadas por cinco minutos
(`radar-account.json` na pasta de configuração).

Em Configurações > Integrações, um perfil de host de código conectado tem um bloco **Radar**
sob a sua chave SSH: uma caixa por lista (*Solicitações aguardando minha revisão*,
*Todas as solicitações abertas*, *Incluir rascunhos*, *Issues atribuídas a mim*, a última
ausente nos hosts sem issues), um campo *Namespaces* (um por linha, com a caixa *Expressões regulares*; vazio
significa o seu e todos aos quais você pertence) e a escolha *Solicitação de um
repositório não clonado*: *Perguntar*, *Clonar* ou *Revisar pela API*.
Tudo é salvo à medida que você altera. Os hosts sem consulta de toda a conta (Bitbucket
Cloud) mostram uma linha dizendo isso no lugar. A paleta de comandos chega ao bloco
com *Configurações: Opções do Radar de um perfil...*, e a busca das Configurações encontra cada
controle pelo seu rótulo.

Pela linha de comando: `gitpoulpi hosting integrations radar <id> --review-requests on`
ativa uma lista, `gitpoulpi hosting radar --json` imprime as listas (e
`--refresh` ignora o cache). Os agentes de IA têm os mesmos dois comandos pelo MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 14. Pela linha de comando

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

Veja a [referência da CLI](cli-reference.md#comandos) para todos os comandos. Um token
nunca é aceito como argumento, e o servidor MCP não oferece
`hosting login`, então um token nunca passa por um agente. O `hosting login`
imprime `stored: "os"` quando o repositório de credenciais do sistema guarda o token, ou
`stored: "session"` mais `store_error` quando o repositório o recusou e o token
fica na memória só desta sessão.

## Solução de problemas

| Sintoma | Solução |
|---|---|
| *Conectar* falha com um erro de autenticação | O token está errado, expirou ou foi revogado. Crie um novo. |
| Um escopo aparece em vermelho | O token não o tem. A conexão funciona, mas esse recurso não. Adicione o escopo no host, depois **Editar** e cole o novo token. |
| A barra lateral não tem a seção PULL REQUESTS | Nenhum perfil ativado está conectado. Conecte um e depois vincule o repositório no menu da seção. |
| Uma linha *Reconectar* em uma lista | O token se perdeu (reinício sem auxiliar de credenciais) ou expirou. Clique na linha para abrir o perfil. |
| Uma instância self-managed falha | Confira se o **Domínio do host** não tem caminho, e se o seu computador confia no certificado da instância. |

## Limites conhecidos

- **Pull requests de forks:** o Azure DevOps e o Bitbucket não conseguem fazer checkout de um
  pull request de um fork (veja [Ações em uma linha](#ações-em-uma-linha)). O Azure
  DevOps não consegue criar um pull request a partir de um fork.
- O **Trello** mostra no máximo os primeiros 1000 cartões de um quadro. Use o filtro de
  listas para restringir. O **Jira**, o **Linear** e o **YouTrack** listam no máximo 500
  issues por atualização. Use um fragmento de JQL ou uma consulta do YouTrack para restringir.
- O **Linear** aceita um responsável por issue: se você escolher vários no seletor,
  o primeiro é enviado. O seu "marco" é o **ciclo** da equipe (os marcos de projeto
  não são usados). O do **YouTrack** é o campo `Fix versions`; uma
  versão não tem número, então o seletor numera as não lançadas pela sua
  posição no projeto. Nenhum dos dois rastreadores tem pull requests.
- O **Redmine** aceita um responsável por issue (o primeiro do seletor), não tem rótulos
  e lista no máximo 500 issues por atualização. Os responsáveis são os membros do projeto
  (pessoas e grupos) e se distinguem pelo nome de exibição. As colunas do quadro Kanban
  são os status da issue, e mover um cartão muda o status; valem as regras de fluxo de trabalho
  padrão do Redmine (um papel que não pode fazer essa transição recebe a
  recusa de volta e o cartão volta). "Resolved" não é um status fechado nos dados
  padrão do Redmine: a coluna Fechado são os status "Closed" e "Rejected".
- O **Bitbucket Data Center** não tem links de navegador para novos pull requests,
  branches ou commits. As páginas dos pull requests abrem normalmente.
- O **Azure DevOps** não mostra avatares de autores. Os filtros “meus” e “revisão solicitada”
  precisam de um endpoint que pode recusar tokens de acesso pessoal. Nesse
  caso o perfil conecta sem nome de usuário e esses dois filtros ficam
  indisponíveis. Um token inválido recebe uma página de login em vez de um erro;
  o GitPoulpi o informa como token rejeitado.
- **Gitea e Forgejo:** um rascunho é um pull request cujo título começa com `WIP:`
  (o servidor não tem sinalizador de rascunho na criação). Um pull request lista como revisores
  todos os que já o revisaram, e retirar um pedido de revisão só
  remove os pendentes. Os pull requests da branch local de um fork fazem checkout
  por `refs/pull/<n>/head`, como no GitHub. As menções são buscadas com o filtro
  `mentioned_by` do servidor. O servidor não mostra escopos de token, então um
  escopo ausente só é descoberto quando uma chamada é recusada.
- **O quadro de issues** move cartões em toda fonte de issues. No Jira, mover executa a transição de fluxo de trabalho
  que leva à coluna; uma transição que pede campos (uma resolução, por exemplo) é recusada pelo Jira e o cartão volta
  com a mensagem dele. No Azure DevOps, um tipo de work item sem o estado de destino volta com o motivo.
- **Bitbucket:** as senhas de aplicativo não existem mais; use um token de API.
  Os repositórios são listados workspace por workspace.
- **Jira Cloud:** só funcionam tokens de API clássicos (sem tokens com escopo, sem OAuth).
  Uma nova issue do Jira recebe uma descrição em texto simples.
- Criar um pull request, work item, issue ou cartão acontece no host e
  **não pode ser desfeito** pelo GitPoulpi. O checkout de um pull request pode ser desfeito
  normalmente.
