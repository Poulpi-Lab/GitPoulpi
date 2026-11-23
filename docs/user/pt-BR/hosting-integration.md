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

**Aonde ir depois.** Esta página é a configuração: tokens, conexão, vincular um
repositório, chaves SSH. Com uma conta conectada:

- as seções PULL REQUESTS e ISSUES da barra lateral, o quadro de issues, criar um
  pull request ou uma issue, o chip da barra de ferramentas e as referências em
  mensagens de commit:
  **[Pull requests e issues](pull-requests-and-issues.md)**;
- a visão no aplicativo de uma issue ou solicitação, com os seus comentários, a
  escrita e o publicar como: **[Conversas](conversations.md)**;
- ler e revisar uma solicitação com threads de comentários e uma revisão
  pendente: **[Revisando um pull ou merge request](review.md)**;
- os pull requests e as issues de um espaço de trabalho inteiro em uma só lista:
  **[O Radar](radar.md)**.

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

## 5. Chaves SSH

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

## 6. Proteção de branch, para uma reescrita de histórico

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

## 7. Listas de toda a conta no Radar

O Radar normalmente observa os repositórios que você clonou. Um perfil também pode
listar o que lhe diz respeito em todo o host, clonado ou não, com três chaves na
sua tabela `[integrations.radar]` (todas desativadas por padrão; veja a
[referência de configuração](config-reference.md#integrationsradar-listas-de-toda-a-conta)):

- **solicitações de revisão**: solicitações abertas que pedem a sua revisão;
- **solicitações abertas**: todas as solicitações abertas do seu próprio namespace e de cada org ou
  grupo de que você é membro, ou dos namespaces que você indicar. Nunca o host inteiro;
  os **Namespaces** do perfil (um por linha em Configurações > Integrações > Radar, opcionalmente
  como expressões regulares) limitam as três listas: uma linha corresponde ao caminho completo de um
  repositório quando é igual a ele ou é um prefixo de caminho em um `/` (`orbit` mantém `orbit/products/x`,
  não `orbitx/y`), e é aplicada às linhas que o host devolve, mais no servidor onde a busca do host
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
ausente nos hosts sem issues), um campo *Namespaces* (uma entrada por linha: um nome que corresponde a esse grupo ou
proprietário e a tudo abaixo dele, ou uma expressão regular por linha com a caixa
*Expressões regulares* marcada; vazio significa o seu e todos aos quais você pertence) e a escolha *Solicitação de um
repositório não clonado*: *Perguntar*, *Clonar* ou *Revisar pela API*.
Tudo é salvo à medida que você altera. Os hosts sem consulta de toda a conta (Bitbucket
Cloud) mostram uma linha dizendo isso no lugar. A paleta de comandos chega ao bloco
com *Configurações: Opções do Radar de um perfil...*, e a busca das Configurações encontra cada
controle pelo seu rótulo.

Pela linha de comando: `gitpoulpi hosting integrations radar <id> --review-requests on`
ativa uma lista, `gitpoulpi hosting radar --json` imprime as listas (e
`--refresh` ignora o cache). Os agentes de IA têm os mesmos dois comandos pelo MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 8. Atuar como um bot de GitHub App

Um projeto que não deve mostrar o nome de uma pessoa pode fazer tudo no host
por meio de um **GitHub App**: todo comentário, revisão, fechamento ou reabertura e
mudança de etiqueta passa então a aparecer como `<app-slug>[bot]`. Isso vale para um repositório
de cada vez e deixa os seus outros repositórios no seu próprio token.

1. Crie um GitHub App para a organização com as permissões **Issues** e **Pull
   requests** em *Read & write* (e *Contents: read*, para que os pull
   requests possam ser listados e revisados), e instale-o no
   repositório.
2. Escreva um pequeno script que imprima um **token de instalação** na saída
   padrão (a chave privada do App assina um JWT de vida curta, que é trocado
   por um token válido por uma hora). O GitPoulpi nunca vê a chave.
3. Em **Configurações > Configurações do repositório > SSH e credenciais**, escolha **Comando de
   token** em HTTPS, informe o script em **Comando** e o host
   (`github.com`) em **Hosts**, e marque **Usar para a API do host
   (issues, pull requests)**. Salve e confirme o comando (uma pessoa sempre
   o confirma, com o seu SHA-256). Pela linha de comando:
   `gitpoulpi repo-auth set --https token-command --hosts github.com
   --use-for-hosting --yes --command ./app-token.sh`.

O que muda para esse repositório:

- As suas chamadas de issues e de pull requests (listar, criar, comentar,
  revisar, fazer merge, fechar, etiquetas) usam o token do comando, com o mesmo
  limite de 30 segundos, cache em memória e ocultação de segredos do login do próprio git. Quando
  o host responde 401 porque o token expirou, o comando é chamado de novo
  uma vez. Se o comando não pode ser executado, a chamada é recusada: o seu token pessoal
  nunca é o plano B.
- Não existe "eu": um token de instalação não consegue perguntar quem ele é. O cabeçalho
  da seção nomeia o perfil com **(bot de App)** e os filtros construídos sobre "eu"
  (*Atribuídas a mim*, *Meus*, *Aguardando minha revisão*) não têm o que corresponder.
  Nada é mostrado como erro.
- **Publicar como.** Toda mensagem que o GitPoulpi publica é mostrada com o nome com que é publicada
  (*Publicando como my-app[bot] (bot de GitHub App)* sob o cabeçalho de uma
  [conversa](conversations.md#publicar-como) e na sua caixa de comentário; `posted_as` na resposta da CLI). Com um token pessoal esse nome é
  o seu login (`GET /user`). Com um token de App é o nome **Publicar como**, por exemplo `my-app[bot]`: defina-o para
  todos os repositórios em **Configurações > Integrações** (`general.hosting_post_as`, também
  `gitpoulpi config set general.hosting_post_as my-app[bot]`) ou para um repositório em **Configurações > Repositório > SSH
  e credenciais > Host**, que prevalece. Se ficar vazio, o GitPoulpi pergunta ao GitHub (GraphQL `viewer { login }`) e,
  se isso falhar, aprende o nome pelo autor do primeiro comentário que publica e o guarda junto ao repositório. Esse
  nome também é como os seus próprios comentários são reconhecidos, então editar e excluir só são oferecidos neles (os
  comentários de um App também devem vir de uma conta de bot); até ele ser conhecido, só os comentários que esta
  instalação publicou podem ser alterados.
- Um App não pode ser atribuído. Combine a configuração com a atribuição bloqueada
  abaixo.
- Ligar ou desligar **Usar para a API do host** exige uma pessoa, como um novo
  comando.

Só os perfis do GitHub usam o token do repositório dessa forma; os outros hosts mantêm o
token do próprio perfil. (Um token de acesso de projeto ou de grupo do GitLab também é um bot: é o token do perfil, mostrado como
*bot do projeto*, e nada precisa ser configurado.)

### Bloqueando a atribuição em um repositório

**Configurações > Configurações do repositório > SSH e credenciais** tem uma linha **Atribuição**
(**Permitida** por padrão, ou **Bloqueada**; salva na hora). Bloqueada, atribuir
é impossível nas issues e nos pull requests desse repositório, quem quer que peça:

- o seletor de responsáveis fica fora dos formulários de nova issue e de novo pull request
  (com uma nota dizendo por quê), o comando da paleta *Hospedagem: editar responsáveis...* fica
  em cinza com o motivo, e o menu da linha diz isso em vez de abri-lo;
- todo cliente recusa uma atribuição antes de qualquer coisa ser enviada, inclusive
  responsáveis passados ao criar uma issue ou um pull request: a CLI responde
  `{"refused": {"reason": "assignment_blocked", ...}}` com código de saída 2, e
  um agente via MCP recebe a mesma recusa;
- os responsáveis que já existem continuam visíveis (somente leitura).

`gitpoulpi hosting assignment` mostra a configuração, `--block` a liga e
`--allow` a desliga de novo; só uma pessoa pode permitir de novo (o servidor MCP
não pode).

## 9. Pela linha de comando

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
  pull request de um fork (veja [Ações em uma linha](pull-requests-and-issues.md#ações-em-uma-linha)). O Azure
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
- **Conversas:** as 300 entradas mais novas carregam primeiro (**Carregar anteriores** para o resto); um comentário, uma edição ou uma
  exclusão publicados pelo GitPoulpi são externos e não podem ser desfeitos por ele; o que cada host não pode fazer está em
  [a tabela](conversations.md#o-que-cada-host-não-pode-fazer). No Gerrit, uma mensagem ou um comentário não pode ser editado nem excluído, e o texto do Gerrit
  é simples.
- Criar um pull request, work item, issue ou cartão acontece no host e
  **não pode ser desfeito** pelo GitPoulpi. O checkout de um pull request pode ser desfeito
  normalmente.
