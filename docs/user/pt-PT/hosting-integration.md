[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · **Português (Portugal)** · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Integrações de alojamento e de gestores de issues

O GitPoulpi liga-se a estes produtos:

- **Alojamento** (pull requests, issues, clonar a partir da sua conta, chaves SSH):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea e Forgejo
  (incluindo o Codeberg). O **Gerrit Code Review** é alojamento só para revisão: as suas alterações são os
  pull requests, não tem issues.
- **Gestores de issues** (apenas issues): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud e Server) e Redmine.
- **Notas Markdown locais** (apenas issues, sem token, sem rede): uma pasta de notas Markdown, por exemplo uma pasta de um vault do Obsidian, veja [Issues locais e Obsidian](local-issues.md).

Todas as integrações usam um **token pessoal**. Não existe conta do GitPoulpi
nem início de sessão com OAuth. As integrações são opcionais e **ativam-se por
repositório**: nada é obtido até ligar um repositório. O seu token é guardado no
seu gestor de credenciais, nunca em `config.toml`. As operações normais de `git`
(fetch, pull, push) não precisam de uma integração: usam as credenciais que o seu
`git` já tem.

## 1. Criar um token no host

O formulário de ligação tem uma ligação *Gerar um token em <host>*. Abre a
página de tokens do host com os âmbitos certos já selecionados. Para o fazer à mão:

| Host | Onde | Âmbitos |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repositórios, pull requests, issues), `read:user` (quem é), `admin:public_key` (apenas para adicionar ou remover chaves SSH) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, em `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Crie o token **para essa organização**, não para "All accessible organizations": os tokens globais deixam de funcionar a 2026-12-01. |
| Azure DevOps Server | A mesma página no seu servidor (`https://<server>/<collection>/_usersSettings/tokens`) | Os mesmos dois âmbitos |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Para as chaves SSH, também `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profile picture > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, em `<host>/user/settings/applications` | Leitura de `user`; leitura e escrita de `repository`, `issue` e `user` (este último para chaves SSH). O Gitea não mostra os âmbitos depois, por isso o GitPoulpi não pode avisar de um em falta: uma chamada recusada di-lo. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. A chave age em nome do utilizador. |
| YouTrack (Cloud e Server) | O seu perfil > Account Security > New token, em `<host>/users/me?tab=account-security` | O âmbito `YouTrack`. Um token permanente age em nome do utilizador. |
| Gerrit | Settings > HTTP Credentials > Generate new token (servidores recentes; os mais antigos: Generate new password), em `<host>/settings/#HTTPCredentials` | Nenhum. O token é a **palavra-passe HTTP** e age em nome do utilizador; o nome de utilizador vai ao lado dela. |
| Redmine | My account > API access key > Show, em `<host>/my/account`. O administrador tem de ter ativado **Administration > Settings > API > Enable REST web service**. | Nenhum. A chave age em nome do utilizador. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (o clássico, sem âmbitos) | Nenhum. O token age em nome do utilizador. |
| Jira Data Center | Profile > Personal Access Tokens | Nenhum. O token age em nome do utilizador. |
| Trello | Primeiro uma **chave de API** em `trello.com/apps/admin` (crie um Power-Up e depois gere uma chave de API). Depois um **token** em `trello.com/1/authorize?...&key=<a sua chave>`. | `read`, `write` |

Dê ao token um nome como "GitPoulpi" e uma data de validade. Copie o token:
o host mostra-o apenas uma vez.

## 2. Ligá-lo

1. Prima `Ctrl+,` e escolha **Integrações**.
2. Escolha o produto à esquerda. Os produtos estão em dois grupos: **Alojamento** e
   **Gestores de issues**.
3. Prima **Ligar**. Para adicionar uma segunda conta do mesmo produto, use
   **+ Adicionar perfil**.

   ![A página Integrações: os produtos à esquerda e um botão Ligar para o GitHub](../images/settings-integrations.png)

4. Preencha o formulário e cole o token. O token é mostrado em texto simples enquanto
   o escreve, por isso certifique-se de que ninguém está a ver. O formulário lista os
   âmbitos de que precisa.

   ![O formulário de ligação: um campo Token de acesso pessoal, os âmbitos repo, read:user e admin:public_key, uma ligação para gerar um token em github.com, e os botões Ligar e Cancelar](../images/settings-connect.png)

5. Prima **Ligar**. O GitPoulpi verifica o token junto do host em segundo
   plano. A página mostra então um cartão com o seu nome e login,
   *Ligado* a verde, e a secção da chave SSH. Um token sem um dos âmbitos
   pretendidos liga-se na mesma, e os âmbitos em falta são listados a vermelho.

No cartão: **Editar** volta a abrir o formulário. **Desligar** elimina o token
e mantém o perfil. **Remover** elimina ambos, após uma confirmação.

Teclado na página Integrações: `Tab` / `Shift+Tab` movem-se entre a
lista de produtos, a faixa de perfis e o formulário. `a` adiciona um perfil, `e` edita-o,
`d` ou `Delete` remove-o, `o` abre a página de tokens, `Esc` recua um
passo.

### O que cada formulário pede

A ligação *Gerar um token* só funciona depois de preencher o host ou a
organização.

- **GitHub, GitLab:** apenas o token. **GitHub Enterprise** e
  **GitLab Self-Managed** pedem também o **Domínio do host**
  (`gitlab.example.com`; https, a menos que escreva `http://`). Um nome sem ponto
  (um serviço Docker ou uma máquina da rede local) precisa de uma porta ou de `http://`:
  `gitea:3000`, `http://nas`. Um servidor autoalojado pode ser
  http simples, por exemplo um contentor local em `http://localhost:8080`. Um token enviado por
  http simples viaja sem cifragem, por isso use http apenas numa rede local ou
  de confiança.
- **Azure DevOps:** a **Organização**, o nome em
  `dev.azure.com/<organization>`. Um endereço `https://<org>.visualstudio.com`
  também é aceite. Duas organizações precisam de dois perfis. Um repositório usa o
  perfil da organização que o seu remoto nomeia. O **Azure DevOps Server**
  pede o **URL da coleção**, por exemplo
  `tfs.corp/tfs/DefaultCollection`. O GitPoulpi não usa o gestor de credenciais
  do sistema para o Azure DevOps: o Git Credential Manager guarda aí um token de
  início de sessão, não um token de acesso pessoal.
- **Bitbucket:** o **e-mail da conta Atlassian** e um token de API. Se o
  e-mail estiver vazio, o token é enviado como token bearer: use-o para um token de
  acesso de workspace ou de repositório. O GitPoulpi descobre os seus workspaces a partir do token.
  **Bitbucket Data Center:** o **URL do servidor** e um token de acesso HTTP.
- **Gitea, Forgejo:** o **Domínio do host** e o token. O **Forgejo** começa em
  `https://codeberg.org`: substitua-o para um servidor próprio. O Gitea não tem
  predefinição pública. Um endereço `http://localhost:3000` funciona para um servidor local.
  Os dois tipos partilham um só cliente (o Forgejo é um fork do Gitea com o mesmo
  `/api/v1`); o token é enviado como `Authorization: token <token>`. O
  gestor de credenciais do sistema não é usado para eles.
- **Jira Cloud:** o **URL do site** (`acme.atlassian.net`), o **e-mail da
  conta Atlassian** e um token de API clássico. Os tokens do Jira com âmbitos não são
  suportados. **Jira Data Center:** o **URL do servidor** e um token de acesso
  pessoal.
- **Trello:** a **chave de API** e o **token**. A chave de API é pública, por isso
  é guardada em `config.toml` (como `account`). O token é secreto e vai para o
  gestor de credenciais. O formulário tem duas ligações: *Obter uma chave de API no Trello* e,
  depois de escrever a chave, *Gerar um token*. A chave e o token são enviados num
  cabeçalho, nunca num URL.

- **Linear:** apenas a **chave de API**: o Linear tem um só host público, por isso
  não há endereço a escrever. A chave é enviada tal como está no cabeçalho `Authorization`
  (o Linear não usa `Bearer` para chaves pessoais). Para que as referências `ENG-12`
  das mensagens de commit abram a página certa, guarde o URL do perfil com a sua
  área de trabalho (`https://linear.app/acme`, na CLI: `--url`); as próprias linhas de issues
  levam sempre o endereço exato.
- **YouTrack:** o **URL do site** (`acme.youtrack.cloud` para o Cloud, o seu próprio
  servidor nos outros casos, com o seu caminho de contexto se o tiver, por exemplo
  `host/youtrack`) e um token permanente, enviado como `Authorization: Bearer`. O
  mesmo cliente serve o Cloud e o Server (`<url>/api`).
- **Gerrit:** o **URL do servidor** (com o seu caminho de contexto se o tiver), o seu **Nome de utilizador** e a palavra-passe
  HTTP como token. Viajam como autenticação HTTP basic; o GitPoulpi fala com a API REST `/a/` do
  servidor e ignora a linha `)]}'` com que cada resposta começa. O nome de utilizador não é secreto e fica guardado em
  `config.toml` (como `account`), a palavra-passe vai para o gestor de credenciais. Um repositório é associado a um
  *projeto* do Gerrit em PULL REQUESTS (as suas alterações são listadas aí); o Gerrit não tem issues, e uma alteração é criada
  com um push (ver *Push para revisão* em [Rever](review.md#gerrit)).
- **Redmine:** o **URL do site** do seu Redmine (com o seu subcaminho se o tiver,
  por exemplo `host/redmine`) e a sua chave de API, enviada como `X-Redmine-API-Key`. Se o
  Redmine responder 403 ou uma página HTML, o serviço web REST está desligado (peça ao
  administrador, ou ative-o em **Administration > Settings > API**).

O Jira, o Trello, o Linear, o YouTrack e o Redmine são apenas gestores de issues: sem pull requests, sem repositórios para
clonar e sem chaves SSH. O Bitbucket não tem issues (a Atlassian descontinuou o Bitbucket
Issues). Ligue antes a secção ISSUES de um repositório Bitbucket a um projeto Jira.

## 3. Onde o token é guardado

O token é guardado no **gestor de credenciais** do seu sistema (Git Credential
Manager no Windows, libsecret ou `pass` no Linux, Keychain no macOS). Usa uma
entrada privada que não pode ser confundida com as credenciais de um host real. Nunca
é escrito em `config.toml` e nunca é impresso.

Se não houver um gestor de credenciais configurado, ou se ele recusar o token, o GitPoulpi
guarda o token **em memória apenas durante esta sessão** e avisa-o. Ligue de novo
depois de reiniciar.

O GitPoulpi procura o token de um perfil por esta ordem:

1. A variável de ambiente `GITPOULPI_TOKEN_<PROFILE ID>`. Use-a para fornecer um
   token sem o guardar, por exemplo em CI.
2. O token guardado para o perfil (`token_ref`).
3. Apenas para perfis GitHub e GitLab: o gestor de credenciais do sistema para o
   host, quando nenhum outro perfil ativo usa o mesmo host.

Cada perfil é uma entrada `[[integrations]]` em `config.toml`. Consulte a
[referência de configuração](config-reference.md#integrations-alojamento-e-gestores-de-issues).

## A verificação de segurança

Um token guardado só é tão seguro quanto o endereço para onde é enviado. O GitPoulpi lembra-se, por isso, de cada integração, daquilo a que estava ligada, e pára quando isso muda:

- **O endereço.** Na primeira vez que uma integração é usada (ou ligada), o seu endereço é registado em `trust.toml`, na pasta de dados privada só do utilizador (`<dados>/private/`), não em `config.toml`: editar a configuração nunca pode alargar o que é de confiança. Se o endereço na configuração deixar de ser o registado, nada lhe é enviado e o token nem sequer é lido. As grafias de um mesmo endereço (`HTTPS://Git.Example.com:443/`) são o mesmo endereço; um anfitrião parecido não é.
- **A conta.** Depois de ligar um token, e depois cerca de uma vez por dia (e uma vez por execução para um token que vem de `GITPOULPI_TOKEN_*` ou do auxiliar de credenciais), o GitPoulpi pergunta ao anfitrião a quem pertence o token e compara o **id** da conta, não o nome de utilizador. Um token que passou a pertencer a outra pessoa é parado.
- **O certificado.** Os servidores cujo certificado não se encadeia a uma autoridade conhecida (uma CA privada, um servidor autoassinado) só são aceites para a chave exata que aceitou, e o handshake TLS continua a provar que o servidor possui essa chave. Com a fixação estrita ativa, a chave tem de coincidir mesmo quando o certificado se encadeia a uma raiz pública; o preço é que cada rotação de chave volta a perguntar, razão pela qual é opcional. Os anfitriões públicos (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) não guardam nenhum registo de certificado.
- **http simples.** Um endereço `http://` que não seja de loopback é recusado, a menos que o tenha permitido para essa integração.

Cada pedido é também limitado ao endereço da própria integração (e, para um anfitrião público, ao seu anfitrião de API, como `api.github.com`): um pedido para qualquer outro lado é recusado antes de ser enviado, e um redirecionamento para outro endereço não é seguido.

**As integrações ligadas antes de esta verificação existir** recebem o seu registo em silêncio, a partir da configuração tal como estava, na primeira vez que são usadas depois da atualização: mantêm o nível de confiança de hoje e nada pergunta.

### A partir da linha de comandos

Um comando parado falha com o código de saída 2 e mostra o que mudou (veja a [referência da CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` mostra o que é de confiança; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` e `--allow-insecure-http on|off` aceitam uma coisa cada, indicando o valor exato. Depois de aceitar um novo endereço, escreva o token de novo. Desligar (`hosting logout`) ou remover uma integração elimina o seu token, o seu registo de confiança e o que dela ficou em cache (as listas de repositórios guardadas do diálogo Clonar, as listas de conta do Radar e os rascunhos de revisão remota do seu anfitrião).

Um agente (MCP) não pode aceitar nada: é-lhe dito que a verificação de segurança precisa de si.

### Na interface de terminal

Um pedido parado abre o diálogo **Verificação de segurança da integração** sobre o que estava a fazer. Diz o que mudou (o endereço antigo e o novo, a conta antiga e a nova com os seus ids, ou o certificado antigo e o novo numa pequena tabela com a impressão digital SHA-256 em grupos de quatro, o titular, o emissor e a validade) e que o token não foi enviado. As respostas são numeradas:

1. **Desligar** elimina o token guardado, o registo de confiança e o que ficou em cache da integração.
2. **Confiar** no novo endereço, conta ou certificado (num endereço sem cifra: **Permitir http simples**).
3. **Cancelar** não altera nada.

Nenhuma resposta fica realçada até premir uma seta ou um dígito, pelo que um Enter perdido não aceita nada. Esc e a cruz de fechar cancelam; um clique fora do diálogo não faz nada. Um corpo longo desloca-se com a roda, Page Up/Page Down ou a barra de deslocação. Várias verificações ficam em fila (uma por integração e tipo de alteração) e são feitas uma após a outra. Uma verificação cancelada não volta a surgir enquanto não mudar; **Integrações: rever verificação de segurança** na paleta de comandos volta a trazê-la.

Depois de confiar num novo **endereço**, o formulário da integração abre com o campo do token em foco e a sugestão "Recomenda-se escrever o token de novo": o token guardado nunca foi enviado para lá. No formulário, um endereço alterado de propósito é aceite juntamente com um token escrito para ele; sem token, o formulário pede um.

O formulário tem também dois interruptores. **Fixação estrita do certificado** (para um endereço https) fixa a chave do certificado que o servidor apresenta quando liga. **Permitir http simples (sem cifra)** só é oferecido para um endereço `http://` que não seja este computador, com um aviso vermelho de que o token viaja em texto simples. O cartão da integração ligada mostra a impressão digital fixada.

### O que o token pode fazer, e até quando

Ligar pergunta também ao anfitrião o que o token pode fazer. O GitHub (tokens clássicos), o GitLab e o Bitbucket (quando envia os âmbitos) comunicam os seus âmbitos: o cartão mostra uma linha amarela "Este token tem mais permissões do que o GitPoulpi precisa" com os âmbitos para além do que o GitPoulpi usa, os sensíveis (como `delete_repo`, `admin:org`, `sudo`) a vermelho, com o conselho de preferir um token de permissões granulares com data de expiração. Um token clássico do GitHub (`ghp_...`) recebe esse conselho mesmo quando os seus âmbitos são exatos. Não se podem pedir as permissões a um token granular do GitHub, por isso só se mostra a sua expiração. O cartão mostra a data de expiração quando o anfitrião a indica. Um token sem expiração não mostra nenhum lembrete, e os outros produtos não comunicam âmbitos e não geram nenhum aviso. Os mesmos campos estão no JSON de `hosting integrations login` e `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). São lidos quando liga, por isso o cartão mostra-os na sessão que ligou.

## 4. Ligar um repositório

A ligação é explícita e por repositório. Quando um perfil está ligado, a barra lateral
Refs mostra uma secção **PULL REQUESTS** e uma secção **ISSUES** depois de TAGS.
Cada secção tem duas listas pendentes: o fornecedor e o projeto.

- A lista pendente do fornecedor **propõe** as contas que correspondem aos remotos do
  repositório (`origin` primeiro), como linhas do tipo `owner/name · origin · GitHub`. Uma
  escolha define o fornecedor e o projeto.
- Nada é ligado automaticamente. O Jira, o Trello, o Linear, o YouTrack e o Redmine nunca são propostos, porque
  nenhum remoto os nomeia: escolha-os manualmente.
- **Nenhum (desvincular este repositório)**, no topo da lista pendente, desliga
  a secção. Nada a volta a ligar por si.
- A ligação é guardada no ficheiro `.gitpoulpi/repo-state.toml` do
  repositório.

As duas secções são ligadas **em separado**. Por exemplo, os pull requests podem
vir do Azure DevOps e as issues do Jira.

| Fornecedor | Projeto de PULL REQUESTS | Projeto de ISSUES |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | o mesmo |
| Azure DevOps | `Project/Repo` (a organização vem do perfil) | `Project` (work items) |
| Gitea, Forgejo | `owner/name` | o mesmo |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | nenhum: ligue o Jira |
| Jira | nenhum | a chave do projeto, por exemplo `ABC` |
| Trello | nenhum | um quadro |
| Linear | nenhum | uma chave de equipa, por exemplo `ENG` (as equipas são listadas) |
| YouTrack | nenhum | um nome curto de projeto, por exemplo `ABC` |
| Gerrit | o nome do projeto Gerrit, por exemplo `team/tool` | nenhum |
| Redmine | nenhum | um identificador de projeto, por exemplo `bench` (a última parte do endereço do projeto) |

A lista de projetos de ISSUES permite pesquisa. Pode também escrever um projeto que a
lista não mostre.

**Consulta extra.** Uma origem de issues tem uma linha de filtro opcional por baixo da
lista pendente do projeto. É combinada (AND) com o projeto:

- Jira: um fragmento **JQL** (`component = UI AND labels = backend`).
- Azure DevOps: uma condição **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack: uma **consulta do YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  É envolvida em parênteses, por isso um `or` não pode alargar o projeto. Um
  `sort by:` seu substitui o `sort by: updated desc` predefinido.
- Redmine: nenhum (bastam os filtros de estado e de «atribuídas a mim» da secção).
- Linear: nenhum (bastam os filtros de estado e de «atribuídas a mim» da secção).
- Trello: uma lista pendente onde marca as **listas** a mostrar. Sem marcas
  mostra todas as listas abertas.
- O GitHub e o GitLab ignoram-na.

`Enter` mantém uma consulta escrita, `Esc` descarta-a. Paleta:
*Issues: editar consulta / listas...*.

Na CLI: `gitpoulpi hosting link pulls --id <profile>` (ou `issues`),
`--clear` para desligar, `--filter` para a consulta extra. Consulte a
[referência da CLI](cli-reference.md#comandos).

### Clonar a partir de uma conta ligada

Um perfil ligado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) também é uma fonte no diálogo **Clonar** (o botão **Clonar** do separador Home, ou `c` nele). Cada perfil aparece em **Clonar por URL** como uma fonte própria (`GitHub · name`, `GitLab · name`); o GitHub e o GitLab nunca se misturam. Escolha um e os repositórios dele são carregados, agrupados por proprietário: primeiro a sua própria conta, depois uma secção por organização ou grupo (`quarry/tools` para um subgrupo do GitLab), cada uma sob um cabeçalho com a contagem. Um cadeado marca um repositório privado.

![O diálogo Clonar com uma conta do GitHub ligada: uma secção por proprietário e um cadeado num repositório privado](../images/clone-hosted.png)

A caixa de pesquisa filtra em todas as secções (nome, caminho completo e descrição); uma secção sem correspondência desaparece. `Enter` numa linha preenche o clone com o URL HTTPS desse repositório e um nome de pasta.

![O diálogo Clonar com a pesquisa a reduzir a lista a uma organização](../images/clone-hosted-search.png)

A lista de um perfil que já abriu antes aparece de imediato a partir de uma cópia guardada enquanto o host é consultado de novo (*a atualizar...* junto à etiqueta); quando a resposta chega, os repositórios novos aparecem e os que desapareceram saem, e o repositório realçado e o escolhido ficam onde estão. Se a atualização falhar, a lista guardada fica com uma nota, e o motivo está no Registo de atividade.

Se o host recusar o token, o diálogo mostra *não foi possível listar os repositórios* com o motivo no lugar da lista (e a cópia guardada é eliminada); as outras fontes continuam a funcionar.

## 5. Pull requests e issues na barra lateral

### PULL REQUESTS

A secção tem um cabeçalho com a contagem e um `+` (criar um pull request),
as listas pendentes de fornecedor e projeto, uma caixa de pesquisa e uma lista pendente de estado (*Abertos*,
*Merged / fechados*, *Todos*). Seguem-se quatro grupos, cada um com a sua contagem:

- **Meus pull requests**: é o autor.
- **Atribuídos a mim**.
- **A aguardar minha revisão**: é um revisor pedido.
- **Todos os pull requests**: recolhido no início.

A caixa de pesquisa filtra todos os grupos (título, número, autor, branch). Cada
alteração da lista pendente de estado volta a perguntar ao host.

Uma linha começa por um glifo de estado: `●` aberto (verde), `○` rascunho, `◆` merged
(roxo), `✕` fechado (vermelho). O pedido da branch em checkout está a negrito.
As linhas lêem-se `#12  título` (GitHub, Bitbucket) ou `!53  título` (GitLab,
Azure DevOps).

### ISSUES

O título da secção nomeia o fornecedor: `ISSUES DO GITHUB`, `ISSUES DO JIRA`, e assim
por diante. Lê-se `ITENS DE TRABALHO DO ...` no Azure DevOps e `CARTÕES DO ...` no Trello.
Por baixo das listas pendentes: uma caixa de pesquisa, a lista pendente de estado (*Abertas*,
*Atribuídas a mim*, *Fechadas*, *Todas*) e as issues. Uma linha do Jira lê-se
`ABC-12  título`, com o tipo e o estado esbatidos quando a barra lateral é
suficientemente larga.

### Ações sobre uma linha

- `Enter` ou um clique duplo abre a linha no navegador.
- O botão direito, ou `.`, abre o menu da linha:
  - **Abrir no navegador**, **Copiar URL**, **Copiar número**.
  - Para um pull request: **Editar no <fornecedor>**, **Checkout da branch de origem**,
    **Ver commits da branch de origem**. Para um pedido vindo de um fork, o checkout lê-se
    **Checkout de pull request (fork)**, e **Ver** fica a cinzento.
  - Para uma issue: **Criar branch a partir da issue**. Abre o pedido de nova branch
    com um nome como `12-fix-the-thing` (ou `ABC-123-fix-the-thing` para
    o Jira).
  - No GitHub, GitLab, Gitea e Forgejo, uma linha de pull request tem também **Rever...**
    (também **Enter** na linha): a revisão na própria aplicação, ver
    [Rever um pull request](review.md).
  - No GitHub, GitLab, Gitea e Forgejo, ambas as linhas têm também **Editar responsáveis...**,
    **Editar etiquetas...** e **Definir marco...** (e **Editar revisores...** num
    pull request): ver
    [Responsáveis, revisores, etiquetas e marco](#responsáveis-revisores-etiquetas-e-marco).
- **Editar no <fornecedor>** tem outras três formas de acesso: o lápis `✎` na extremidade direita
  de uma linha de pull request com o rato por cima ou selecionada, `e` na linha selecionada, e
  o comando da paleta *Integrações: editar pull request associado no host*
  (para o pedido da branch em checkout).
- Se a branch de origem ainda não estiver no repositório, o GitPoulpi di-lo. Faça
  primeiro fetch (`Ctrl+G f`).

**Checkout de um pull request.** O GitPoulpi faz fetch do pedido para uma branch
local `pr/<n>` (`mr/<n>` no GitLab) que o acompanha, e muda para ela. No
GitHub, GitLab, Gitea e Forgejo isto funciona também com forks (o GitPoulpi faz fetch da ref de head
especial do host). O Azure DevOps e o Bitbucket fazem fetch da branch de origem do
pedido. Um pedido de um fork é aí recusado: adicione o fork como remoto e
faça manualmente o checkout da respetiva branch. Desfazer volta à branch anterior. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Rever um pull request na linha de comandos (e um agente).** No GitHub
(incluindo Enterprise), GitLab (incluindo self-managed), Gitea e Forgejo um pedido pode ser
lido e revisto sem sair do GitPoulpi: `hosting review show <n>` (ficheiros,
estado de integração, verificações), `hosting review threads <n>` (discussões de revisão e
conversa), `hosting review comment`, `hosting review reply`, `hosting review resolve`
e `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. O diff vem do seu repositório, não do
host: `hosting review fetch <n>` obtém o head do pedido para
`refs/gitpoulpi/pr/<n>` e a sua branch de destino para `refs/gitpoulpi/pr/<n>-base`
(nenhum é uma branch, nada fica com checkout). Os números de linha são os desse
diff: `new` é a linha no head, `old` a linha na base de integração. O GitLab
não tem o estado de revisão «pedir alterações»: é recusado aí, sem enviar nada.
O Gitea e o Forgejo aceitam um comentário numa só linha (sem intervalos); uma discussão são os
comentários de uma linha de um ficheiro. O Gitea pode resolver uma discussão, o Forgejo 11 não
(o host não tem endpoint para isso: a chamada di-lo). Uma resposta num servidor
sem o endpoint de resposta é publicada como uma revisão de um comentário na mesma linha.
O Gitea recusa aprovar ou rejeitar o seu próprio pedido.
Sem um clone: acrescente `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` e `suggest` para rever um projeto que não é o repositório atual,
`hosting review diff <n> --project P` lê o diff (hunks por ficheiro) através da API do host, e
`hosting file <path> --project P [--ref R]` lê um ficheiro numa revisão (GitHub, GitLab, Gitea, Forgejo e Gerrit; no Gerrit o diff de um
patch set é lido ficheiro a ficheiro e o ficheiro vem da branch ou, para um id de commit, do commit). Um ficheiro que o host
não mostra no diff (demasiado grande) é marcado `omitted`.
As ferramentas MCP são os mesmos comandos (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Responsáveis, revisores, etiquetas e marco

No GitHub (incluindo Enterprise), GitLab (incluindo self-managed), Gitea e
Forgejo, pode mudar a quem um pull request ou uma issue está atribuído, quem o revê, as suas etiquetas
e o seu marco sem sair do GitPoulpi. Os outros fornecedores não mostram nada disto.

- **Menu da linha**: **Editar responsáveis...**, **Editar revisores...** (só pull requests),
  **Editar etiquetas...**, **Definir marco...**.
- **Paleta**: *Alojamento: editar responsáveis...*, *Alojamento: editar revisores...*,
  *Alojamento: editar etiquetas...*, *Alojamento: definir marco...* listam primeiro os
  pedidos e as issues da barra lateral a escolher. No
  [Radar](radar.md) os mesmos quatro atuam sobre a linha selecionada.
- O seletor abre de imediato e carrega do host as pessoas ou as etiquetas do projeto (e o que
  o item tem agora). Uma linha com indicador de atividade aparece passados cerca de 150 ms.
  Escreva para filtrar (correspondência aproximada), `Up` / `Down` movem, `Space` marca ou
  desmarca, `Enter` aplica, `Esc` cancela, `Tab` chega aos botões. Um clique
  numa linha marca-a e a roda desloca. As pessoas mostram um selo com iniciais,
  as etiquetas a sua cor.
- O marco é uma lista pendente (com **Sem marco** para o limpar).
- Depois de **Aplicar** o host é atualizado, e as listas e o Radar mostram a
  alteração depois da sua atualização seguinte, que começa logo.

Os formulários de criação (um pull request, uma issue) têm as mesmas escolhas como uma fila de
botões: `Responsáveis: nenhum`, `Revisores: nenhum`, `Etiquetas: nenhum`,
`Marco: nenhum`. No GitHub, GitLab, Gitea e Forgejo o botão Etiquetas substitui o campo
de etiquetas escritas do formulário de issue. Se um host aceitar o novo pull request mas
recusar algumas das suas etiquetas ou pessoas, a barra de estado di-lo.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, e `--assignee`, `--reviewer`, `--label`,
`--milestone` em `hosting pr` e `hosting issue create`; ver a
[referência da CLI](cli-reference.md).

### Teclas nas secções

| Tecla | Ação |
|---|---|
| `Enter` / clique duplo | abrir a linha no navegador |
| `.` / botão direito | menu da linha |
| `e` | editar o pull request selecionado no host |
| `n` | novo pull request (PULL REQUESTS) ou nova issue (ISSUES) |
| `v` | abrir a lista pendente de estado |
| `r` | atualizar |
| `Enter` / `Space` num cabeçalho | recolher ou expandir a secção |

### Religar e offline

Quando o token falta, é rejeitado ou não tem um âmbito, a primeira linha da
secção lê-se `Religar (token rejeitado)`. Clique nela, ou prima `Enter`, para abrir
o perfil em Definições > Integrações.

Quando o host não pode ser alcançado, as últimas listas ficam no ecrã e o cabeçalho
mostra a sua idade (`· 5m de idade`). A última resposta fica também em cache no disco
(`.gitpoulpi/hosting-cache.json`), por isso as listas aparecem antes de a rede
responder.

## 5b. O quadro de issues

O botão **Quadro** do cabeçalho ISSUES (paleta: **Issues: abrir quadro**) mostra as issues da origem associada como um quadro
Kanban sobre a janela. As colunas dependem da origem:

| Origem | Colunas | Mover um cartão |
|---|---|---|
| Notas Markdown locais | os estados (`statuses` do perfil, depois os outros em uso) | reescreve o `status` da nota |
| GitHub, GitLab, Gitea, Forgejo | **Abertas**, depois as etiquetas que escolheu, depois **Fechadas** | troca a etiqueta de estado; para **Fechadas** fecha a issue, para uma coluna outra vez reabre-a |
| Trello | as listas abertas do quadro, da esquerda para a direita | move o cartão para a lista |
| Redmine | os estados das issues | muda o estado da issue (aplicam-se as regras de fluxo do Redmine) |
| Jira (Cloud e Data Center) | os estados do projeto, por fazer primeiro, concluído por último | executa a transição de fluxo que leva à coluna (`GET` e depois `POST .../transitions`); sem essa transição o cartão volta atrás e a barra de estado diz `no transition from X to Y` |
| Azure DevOps (Services e Server) | os estados dos tipos de work item do projeto, por categoria de estado | define `System.State`; um work item cujo tipo não tem o estado da coluna volta atrás, com o motivo (`a Bug has no state Done`) |

Num projeto do GitHub ou do GitLab, **Colunas** (ou `c`, ou **Quadro: definir etiquetas das colunas...**) pede as etiquetas que são as
colunas, por ordem, por exemplo `status::todo, status::doing, status::review`. Propõe as etiquetas do projeto que
parecem estados. A escolha é guardada para o repositório. Sem etiquetas o quadro é Abertas / Fechadas. Mover um cartão
só mexe nas etiquetas de estado que listou: as suas outras etiquetas ficam. Um quadro de host lista as issues abertas e as 60 mais
recentemente fechadas.

Uma transição do Jira que pede campos obrigatórios (uma resolução, por exemplo) é recusada pelo Jira e o cartão volta atrás com
a mensagem do próprio Jira: faça essa no site.

O mesmo quadro está disponível para scripts e agentes: `gitpoulpi hosting board` imprime as colunas com os seus cartões e
`gitpoulpi hosting board move <card> <column>` move um (ver [a referência da CLI](cli-reference.md)).

Um cartão mostra o número, o título, os responsáveis como iniciais, as etiquetas e o marco. Arraste-o para outra coluna,
ou `Shift+Left` / `Shift+Right`, ou **Quadro: mover cartão para...** (`m`). Move-se de imediato; passados cerca de 150 ms um indicador de atividade
diz que o host ainda está a trabalhar, e se o host recusar, o cartão volta atrás e a barra de estado diz porquê. `Enter` abre
a issue no host (a nota, para issues locais); `e` edita responsáveis, etiquetas e marco; `n` inicia uma nova issue na
coluna selecionada; `a`, `l`, `M` e `/` filtram por responsável, etiqueta, marco e texto, `x` limpa os filtros; `r`
lê o host de novo. O quadro é lido de novo quando as listas da barra lateral se atualizam. O Radar não tem quadro.

## 6. Criar um pull request

Abra o formulário com o `+` do cabeçalho de PULL REQUESTS, `n` na secção, ou
o comando da paleta *Pull Requests: criar uma*. Um menu de branch tem também
**Iniciar pull request a partir de <branch>...**.

O formulário tem:

- Um separador por perfil ligado que tenha pull requests (`Left` / `Right` na
  linha de separadores).
- Caixas combinadas **Repo de origem / Branch** e **Repo de destino / Branch**. As listas longas
  permitem pesquisa. Pode escrever uma branch ou um repositório que as listas não mostrem.
- **Título**, preenchido com o assunto do commit de topo.
- **Descrição**, em várias linhas (`Enter` adiciona uma nova linha, `Tab` sai do
  campo).
- Botões **Responsáveis**, **Revisores**, **Etiquetas** e **Marco** (GitHub,
  GitLab, Gitea e Forgejo), que abrem o seletor.
- **Enviar como rascunho**.
- Com a [IA](ai.md) ativa: **Gerar título e descrição**.

O formulário abre no perfil, repositório e branches da branch em checkout
(`feature -> main`). Um repositório de origem diferente cria um pedido
a partir de um fork.

**Criar pull request** (ou `Ctrl+S`) cria o pedido no host. O
resultado abre no navegador e a barra de estado nomeia-o.

## 7. Criar uma issue

Quando a secção ISSUES tem um fornecedor **e** um projeto, o seu cabeçalho tem um
`+` (*Nova issue*). Também: `n` na secção, ou o comando da paleta
*Alojamento: nova issue...*.

O formulário tem **Título** (obrigatório), **Descrição** (várias linhas) e
**Labels** (separadas por vírgula, por exemplo `bug, ui`; no GitHub, GitLab, Gitea e Forgejo os
botões Etiquetas, Responsáveis e Marco abrem antes o seletor). O Jira e o Azure DevOps
acrescentam uma lista pendente **Tipo**. O Trello acrescenta uma lista pendente **Lista**. O Linear e o YouTrack
acrescentam uma lista pendente **Estado** (o estado do fluxo, ou o valor de `State`, em que a issue
começa). O Redmine acrescenta uma lista pendente **Tipo** com os trackers do projeto (Bug, Feature,
Support...); não tem etiquetas, por isso não há botão Etiquetas para ele e uma etiqueta
dada na linha de comandos é recusada. O formulário preenche estas
listas pendentes a partir do host quando abre.

**Criar issue** (ou `Ctrl+S`) cria-a no host. `Esc` ou **Cancelar**
fecha o formulário. Se o host recusar (token rejeitado, âmbito em falta, projeto
desconhecido, label desconhecida), o formulário mostra o motivo e mantém o que escreveu.
Em caso de sucesso, a barra de estado nomeia a nova issue e a secção ISSUES é
atualizada.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## 8. Criar na própria página web do host

O formulário Nova issue e o formulário Criar pull request têm um botão
**Criar no <fornecedor>**, em cima à direita. A tecla é `Alt+W`. Abre o
formulário web do host no navegador, preenchido com o que escreveu até ao momento. A
janela de diálogo fica aberta. Não precisa de token e não faz nenhuma chamada à API, por isso também funciona
depois de um erro da API.

- O GitHub não tem parâmetro de URL para *rascunho*, por isso a caixa de rascunho não é
  transportada. O GitLab recebe um prefixo `Draft: ` no título.
- O GitLab aceita labels apenas por id, por isso são acrescentadas como uma
  ação rápida `/label ~"x"` no fim da descrição.
- Uma descrição muito longa é cortada para que o URL fique abaixo de 6000 carateres,
  com uma nota `[...truncated: paste the rest]`.
- O Bitbucket Data Center não tem endereço de formulário web, por isso não tem
  botão **Criar no <fornecedor>**.

CLI: `gitpoulpi hosting issue create ... --web` e `gitpoulpi hosting pr --web`
imprimem o mesmo URL.

## 9. O chip da barra de ferramentas

No extremo da barra de ferramentas (à direita por predefinição), um chip mostra o pedido da branch
em checkout: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) ou
`BB #12` (Bitbucket). A cor mostra o estado: verde aberto, cinzento rascunho,
roxo merged, vermelho fechado.

- Passe o rato por cima para ver o título, o estado e `origem → destino`.
- Clique nele, ou prima `Ctrl+G g`, para abrir o pedido no navegador.
- Clique com o botão direito para **Abrir no navegador**, **Copiar URL**, **Atualizar** e
  **Mudar de perfil**.

Uma branch enviada sem pedido mostra um chip `+ PR` esbatido. Clique nele para iniciar um
pull request. Não há chip quando nenhum perfil corresponde, quando a branch não
foi enviada, ou quando o host não pôde ser alcançado. Numa janela estreita o chip fica
oculto para deixar espaço aos botões da barra de ferramentas.

## 10. Quando as listas são atualizadas

As listas são atualizadas:

- quando abre um repositório;
- quando a branch em checkout muda;
- após um push ou um fetch;
- a cada *Atualização de Pull Requests e Issues (minutos)*
  (`general.hosting_refresh_minutes`, predefinição 5, `0` desliga o temporizador;
  Definições > Integrações);
- quando fecha as Definições;
- quando prime `r`, executa *Integrações: atualizar pull requests e issues*, ou
  altera um filtro.

Os pedidos nunca bloqueiam a janela.

Os hosts limitam quantas chamadas pode fazer. O GitPoulpi reenvia o `ETag` da
última resposta, por isso uma lista inalterada custa pouco (nada no GitHub). Quando um
host indica que restam poucas chamadas, a barra de estado di-lo. Quando já não
restam nenhumas, o GitPoulpi deixa de perguntar a esse host até o limite repor, e as listas
ficam como estão.

## 11. Terminologia

O GitHub, o Azure DevOps e o Bitbucket dizem "pull request". O GitLab diz "merge
request". *Terminologia de pedido de revisão* (`ui.pr_terminology`, em
Definições > Integrações) escolhe a palavra para todas as etiquetas, menus, comandos
da paleta e mensagens:

- **Auto (por host)**: segue o fornecedor ligado do repositório ("Pull
  Request" enquanto nada estiver ligado).
- **Pull Request** ou **Merge Request**: sempre essa palavra.

A alteração aplica-se de imediato. A CLI e o MCP dizem sempre "pull/merge request".
Esta página diz "pull request".

## 12. Referências em mensagens de commit e nomes de branch

Quando as issues ou os pull requests do repositório estão ligados, as referências na
mensagem de commit dos Detalhes do commit aparecem sublinhadas. Um clique abre a
página ligada. Teclado: o comando da paleta
*Integrações: abrir referência do commit (#issue / !pedido)...* (um menu quando a
mensagem tem várias referências). Um repositório sem nada ligado não tem
ligações.

O que conta como referência depende do fornecedor da secção ISSUES:

| ISSUES ligadas a | Numa mensagem | Num nome de branch |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` e `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, apenas quando `ABC` é a chave do projeto ligado. Texto como `UTF-8`, `SHA-256` ou `ISO-8601` nunca é uma ligação. | `ABC-123` em qualquer ponto do nome (`feature/ABC-123-x`), em maiúsculas |
| Linear | `ENG-123`, apenas quando `ENG` é a chave da equipa ligada (ou de uma equipa na lista de equipas do perfil). `UTF-8` nunca é uma ligação. | O `branchName` próprio do Linear quando cria a branch a partir de uma issue; `ENG-123` em qualquer ponto do nome é lido de volta |
| YouTrack | `ABC-123`, apenas quando `ABC` é o nome curto do projeto ligado (ou de um da lista de projetos do perfil) | `ABC-123` em qualquer ponto do nome |
| Redmine | `#123`, apenas quando o Redmine é a origem ISSUES ligada (uma origem de pull requests mantém o seu próprio sigilo) | `123-fix-login` |
| Trello | um endereço `https://trello.com/c/<shortLink>`; `#12` apenas quando um cartão em cache tem o número 12 | `12-fix-login` |

| PULL REQUESTS ligados a | Numa mensagem |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Quando ambas as listas correspondem a `#N`, a issue ganha. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

### Inserir uma referência de issue

No compositor de commits, o botão `#▾` na extremidade direita do Resumo abre
uma lista filtrável das issues ligadas. A issue indicada pela branch atual
vem primeiro, marcada com *(da branch)*. `Enter` ou um clique insere a sua referência
(`ABC-123`, `#12`, `AB#12`) no cursor. Teclado: `Alt+R` no compositor.
Paleta: *Commit: Inserir referência de issue...*. Sem uma lista de issues ligada, o
botão fica a cinzento e diz *ligue primeiro uma lista de issues na barra lateral*.
O GitPoulpi nunca acrescenta uma referência por si.

## 13. Chaves SSH

### Chave SSH por perfil

Um perfil ligado tem uma secção **Chave SSH** por baixo do cartão. Sem chave,
escolha uma de:

- **Chave predefinida**: o seu ssh-agent ou `~/.ssh`. Nada muda.
- **Gerar nova chave**: um título opcional (predefinição
  `GitPoulpi <host> <data>`). Com **Adicionar a chave pública a <host>** marcado,
  o GitPoulpi envia-a, se o host ainda não a tiver. A chave é
  escrita sem frase-passe em `~/.ssh/gitpoulpi_<profile id>_ed25519` e
  `.pub`. Não é preciso `ssh-keygen`. Um ficheiro existente nunca é substituído.
- **Chave própria**: escreva um caminho ou use **Procurar** para um ficheiro de chave privada,
  depois prima **Usar esta chave**.

Quando uma chave está associada, a secção mostra `título (SHA256:...)` com:

- **Remover**: desassociar a chave, e eliminá-la no host se o GitPoulpi a tiver adicionado.
- **Eliminar**: o mesmo, e elimina também os ficheiros da chave, após uma confirmação
  que lista os dois caminhos.

### Chave SSH para um repositório

Para fazer um repositório usar uma chave em particular, abra o menu do separador (`Ctrl+T`, ou clique com o botão direito
no separador) e escolha **SSH e credenciais...**, ou execute o comando da paleta
*Repositório: SSH e credenciais...*, e escolha **Ficheiro de chave** (ou **Chave da conta**
para a chave de um perfil de integração) em SSH. A escolha fica na pasta privada do
GitPoulpi: nada é escrito no repositório nem na configuração do git. Veja
[Definições](settings.md#específico-do-repositório-ssh-e-credenciais).

Um repositório que ainda tenha um `core.sshCommand` próprio, da antiga fixação
«Chave SSH deste repositório», mostra-o nessa página, só de leitura, com
**Mover para o GitPoulpi**.

CLI e MCP: `gitpoulpi repo-auth` (a escolha por repositório) e, para a própria
fixação `core.sshCommand`, `gitpoulpi ssh-command show | set --key <file> |
clear`.

A chave global em Definições > SSH e credenciais (*Caminho da chave privada*, ou
*Gerar par de chaves*) aplica-se a todos os repositórios sem escolha própria.
O ficheiro de chave ou a fixação de um repositório prevalece sobre ela.

## 13b. Proteção de branches, para uma reescrita do histórico

Quando reescreve o histórico ([o painel de reescrita](history-rewrite.md)), o GitPoulpi
pergunta ao alojamento que branches protege, para que fiquem de fora da reescrita
por omissão. Só pergunta para um repositório que **ligou** (a secção de pedidos
ou de issues da barra lateral, ou um perfil afixado), com o token desse
perfil, e um token que não o consiga ler dá apenas "desconhecido": nada falha.
O que cada fornecedor responde e o que o token precisa:

| Alojamento | O que é lido | O token precisa de |
|---|---|---|
| GitHub (e Enterprise Server) | o indicador `protected` das branches e os rulesets de branches ativos (padrões `~DEFAULT_BRANCH`, `~ALL` e `refs/heads/...`) | um token clássico com `repo` (`public_repo` para um repositório público) ou um token granular com Metadata (leitura). Um ruleset que o token não consiga ler é indicado; as regras clássicas contam na mesma |
| GitLab (gitlab.com e self-managed) | `protected_branches` (nomes e padrões `*`) | `read_api` e uma função que possa ver as definições do repositório |
| Azure DevOps (Services e Server) | as políticas de branch ativas e bloqueantes do repositório (âmbitos `Exact`, `Prefix` e `DefaultBranch`) | Code (leitura). A permissão de "force push" é uma definição de segurança e não é lida |
| Bitbucket Cloud | as restrições de branch que impedem um push (`push`, `force`, `delete`) | `repository:admin`. Uma restrição sobre o modelo de branches não é resolvida |
| Bitbucket Data Center | as permissões de branch que impedem um force push (`fast-forward-only`, `pull-request-only`, `read-only`) | administrador do repositório |

A página de definições da proteção (e da branch predefinida, e da eliminação do
repositório) é construída a partir do URL remoto do repositório para
GitHub, GitLab, Azure DevOps e Bitbucket, mesmo quando o repositório não está
ligado, e é mostrada com os três caminhos possíveis quando o alojamento recusa um push.

## 13c. Listas de toda a conta no Radar

O Radar normalmente vigia os repositórios que clonou. Um perfil também pode
listar o que lhe diz respeito em todo o host, clonado ou não, com três interruptores na
sua tabela `[integrations.radar]` (todos desligados por predefinição; ver a
[referência de configuração](config-reference.md#integrationsradar-listas-de-toda-a-conta)):

- **pedidos de revisão**: pedidos abertos que pedem a sua revisão;
- **pedidos abertos**: todos os pedidos abertos do seu próprio namespace e de cada organização ou
  grupo de que é membro, ou dos namespaces que indicar. Nunca o host inteiro;
  os **Namespaces** do perfil (um por linha em Definições > Integrações > Radar, opcionalmente
  como expressões regulares) limitam as três listas: uma linha corresponde ao caminho completo de um
  repositório quando é igual a ele ou é um prefixo de caminho numa `/` (`ptm` mantém `ptm/produits/x`,
  não `ptmx/y`), e é aplicada às linhas que o host devolve, mais no servidor onde a pesquisa do host
  o permite (os qualificadores `user:` / `repo:` do GitHub);
- **issues atribuídas**: issues abertas atribuídas a si.

Respondem o GitHub, o GitLab (incluindo self-managed), o Gitea, o Forgejo, o Azure DevOps (Services
e Server), o Bitbucket Data Center e o Gerrit. Como se pergunta a cada um:

| Host | Pedidos | Issues |
|---|---|---|
| Azure DevOps | pull requests ativos de toda a organização (`searchCriteria.reviewerId` para os que esperam por si; um pedido em que já votou fica de fora); quando o servidor recusa a rota ao nível da organização, perguntam-se um a um os primeiros 20 projetos. Os `namespaces` são nomes de projeto | work items atribuídos a si que não estão Done, Closed ou Removed |
| Bitbucket Data Center | o seu dashboard: papel `REVIEWER` ainda sem aprovação para os pedidos de revisão, `AUTHOR` e `PARTICIPANT` para os pedidos abertos; os `namespaces` são chaves de projeto | nenhuma (o Data Center não tem gestor de issues; uma nota di-lo) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; os pedidos abertos são `is:open projects:<prefixo>` por namespace (um prefixo de caminho de projeto), ou as suas alterações e as que reviu quando nenhum é indicado; uma alteração em work-in-progress é um rascunho | nenhuma |

O Bitbucket Cloud não tem linhas de toda a conta: não oferece nenhum endpoint que liste
pedidos de vários repositórios, e perguntar a cada repositório seriam chamadas a mais.
Os seus repositórios clonados mantêm as suas próprias listas, e `gitpoulpi hosting
radar --id <bitbucket>` di-lo. Quando o Gerrit tem mais alterações do que o limite que
devolve, a lista fica marcada como cortada com uma nota.

Os rascunhos ficam de fora salvo se os mantiver. Cada lista tem limite: 300 linhas por tipo, com uma
nota a dizer quantas mais havia, e no GitHub poucas chamadas de pesquisa por
ronda, porque a sua API de pesquisa só permite 30 por minuto. Um namespace que o token
não consegue ler passa a ser uma nota e os outros continuam a carregar; um token recusado ou um
limite de pedidos faz falhar apenas esse perfil. As respostas são reutilizadas durante cinco minutos
(`radar-account.json` na pasta de configuração).

Em Definições > Integrações, um perfil de code host ligado tem um bloco **Radar**
sob a sua chave SSH: uma caixa por lista (*Pedidos à espera da minha revisão*,
*Todos os pedidos abertos*, *Incluir rascunhos*, *Issues atribuídas a mim*, a última
ausente nos hosts sem issues), um campo *Namespaces* (uma entrada por linha: um nome que corresponde a esse
grupo ou titular e a tudo o que está abaixo, ou uma expressão regular por linha com
*Expressões regulares* marcada; vazio significa o seu e todos a que pertence) e a escolha *Pedido de um
repositório não clonado*: *Perguntar*, *Cloná-lo* ou *Rever através da API*.
Tudo é guardado à medida que altera. Os hosts sem consulta de toda a conta (Bitbucket
Cloud) mostram em vez disso uma linha a dizê-lo. A paleta de comandos chega ao bloco
com *Definições: Opções do Radar de um perfil...*, e a pesquisa das Definições encontra cada
controlo pela sua etiqueta.

Na linha de comandos: `gitpoulpi hosting integrations radar <id> --review-requests on`
liga uma lista, `gitpoulpi hosting radar --json` imprime as listas (e
`--refresh` ignora a cache). Os agentes de IA têm os mesmos dois comandos por MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 14. A partir da linha de comandos

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

Consulte a [referência da CLI](cli-reference.md#comandos) para ver todos os comandos. Um token
nunca é aceite como argumento, e o servidor MCP não oferece
`hosting login`, por isso um token nunca passa por um agente. O `hosting login`
mostra `stored: "os"` quando o armazém de credenciais do sistema guarda o token, ou
`stored: "session"` mais `store_error` quando o armazém o recusou e o token
fica em memória apenas para esta sessão.

## Resolução de problemas

| Sintoma | Solução |
|---|---|
| *Ligar* falha com um erro de autenticação | O token está errado, expirou ou foi revogado. Crie um novo. |
| Um âmbito aparece a vermelho | O token não o tem. A ligação funciona, mas essa funcionalidade não. Adicione o âmbito no host, depois **Editar** e cole o novo token. |
| A barra lateral não tem a secção PULL REQUESTS | Nenhum perfil ativo está ligado. Ligue um e depois ligue o repositório na lista pendente da secção. |
| Uma linha *Religar* numa lista | O token perdeu-se (um reinício sem gestor de credenciais) ou expirou. Clique na linha para abrir o perfil. |
| Uma instância self-managed falha | Verifique que o **Domínio do host** não tem caminho, e que o seu computador confia no certificado da instância. |

## Limites conhecidos

- **Pull requests de forks:** o Azure DevOps e o Bitbucket não conseguem fazer checkout de um
  pedido de um fork (consulte [Ações sobre uma linha](#ações-sobre-uma-linha)). O Azure
  DevOps não consegue criar um pull request a partir de um fork.
- O **Trello** mostra no máximo os primeiros 1000 cartões de um quadro. Use o filtro de
  listas para o restringir. O **Jira**, o **Linear** e o **YouTrack** listam no máximo 500
  issues por atualização. Use um fragmento JQL ou uma consulta do YouTrack para restringir.
- O **Linear** aceita um responsável por issue: se escolher vários no seletor,
  é enviado o primeiro. O seu «marco» é o **ciclo** da equipa (os marcos de projeto
  não são usados). O do **YouTrack** é o campo `Fix versions`; uma
  versão não tem número, por isso o seletor numera as não lançadas pela sua
  posição no projeto. Nenhum dos dois gestores tem pull requests.
- O **Redmine** aceita um responsável por issue (o primeiro do seletor), não tem etiquetas,
  e lista no máximo 500 issues por atualização. Os responsáveis são os membros do projeto
  (pessoas e grupos) e distinguem-se pelo nome a mostrar. As
  colunas do quadro Kanban são os estados das issues, e mover um cartão muda o estado; aplicam-se
  as regras de fluxo predefinidas do Redmine (um papel que não pode fazer essa transição recebe a
  recusa de volta e o cartão regressa ao sítio). «Resolved» não é um estado fechado nos dados
  predefinidos do Redmine: a coluna Fechadas são os estados «Closed» e «Rejected».
- O **Bitbucket Data Center** não tem ligações de navegador para novos pull requests,
  branches ou commits. As páginas dos pull requests abrem normalmente.
- O **Azure DevOps** não mostra avatares de autores. Os filtros "meus" e "revisão pedida"
  precisam de um endpoint que pode recusar tokens de acesso pessoal. Nesse
  caso o perfil liga-se sem nome de utilizador e estes dois filtros não estão
  disponíveis. Um token inválido recebe uma página de início de sessão em vez de um erro;
  o GitPoulpi comunica-o como token rejeitado.
- **Gitea e Forgejo:** um rascunho é um pedido cujo título começa por `WIP:`
  (o servidor não tem indicador de rascunho ao criar). Um pedido lista como revisores
  todos os que já o reviram, e retirar um pedido de revisão só
  remove os pendentes. Os pull requests da branch local de um fork fazem checkout
  através de `refs/pull/<n>/head`, como no GitHub. As menções são pesquisadas com o filtro
  `mentioned_by` do servidor. O servidor não mostra os âmbitos do token, por isso um
  em falta só é descoberto quando uma chamada é recusada.
- **O quadro de issues** move cartões em todas as origens de issues. No Jira um movimento executa a transição de fluxo que leva
  à coluna; uma transição que pede campos (uma resolução, por exemplo) é recusada pelo Jira e o cartão volta atrás com
  a sua mensagem. No Azure DevOps um tipo de work item sem o estado de destino volta atrás com o motivo.
- **Bitbucket:** as palavras-passe de aplicação já não existem; use um token de API.
  Os repositórios são listados workspace a workspace.
- **Jira Cloud:** só funcionam tokens de API clássicos (sem tokens com âmbitos, sem OAuth).
  Uma nova issue do Jira recebe uma descrição em texto simples.
- Criar um pull request, work item, issue ou cartão acontece no host e
  **não pode ser desfeito** a partir do GitPoulpi. Um checkout de um pedido pode ser desfeito
  normalmente.
