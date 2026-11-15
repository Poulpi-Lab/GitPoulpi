[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Reescrita do histórico

Reescrever o histórico altera commits que já existem: quem os escreveu e quando,
as mensagens, os arquivos que eles guardam, as assinaturas. O GitPoulpi faz isso com uma
**receita** (um escopo e uma lista ordenada de regras) e três passos que nunca pulam
o do meio:

1. A **Prévia** calcula o novo histórico sem tocar em nada: contagens, o que
   cada regra fez, as branches e tags que se moveriam, uma linha por commit
   alterado.
2. **Aplicar** move as suas branches e tags locais para ele. É uma entrada do
   [histórico de desfazer](undo-redo.md): **Desfazer** devolve cada ref movida (e
   os arquivos em checkout) ao que era antes.
3. O **push** (um push forçado com lease) e a **exclusão definitiva** (purge) são passos
   separados e deliberados, porque não dá para desfazê-los: veja
   [Enviar o resultado](#enviar-o-resultado) e
   [Apagar o histórico antigo](#apagar-o-histórico-antigo-purge).

O painel, a paleta, os comandos `gitpoulpi rewrite` e as ferramentas MCP
usam o mesmo motor, então uma prévia feita em um é a que o outro aplica.
[Pela linha de comando](#pela-linha-de-comando) lista as opções;
[Criar regras com IA](#criar-regras-com-ia) preenche o painel a partir de uma descrição;
[Propostas de agentes de IA](#propostas-de-agentes-de-ia) diz o que um agente pode e
não pode fazer aqui.

**As regras, pelo que mudam**

| Grupo | Regras | Seção |
|---|---|---|
| Autores | Substituir um autor, Anonimizar autores, Aplicar o .mailmap | [Autores e o .mailmap](#autores-e-o-mailmap) |
| Datas | Deslocar datas, Mudar o fuso horário, Manter dentro de um horário, Data do committer = data do autor | [Datas](#datas) |
| Mensagens | Localizar e substituir, Excluir linhas, Adicionar ou remover um trailer, Corrigir a codificação das mensagens | [Mensagens](#mensagens) |
| Conteúdo e caminhos dos arquivos | Localizar e substituir nos arquivos, Remover arquivos, Remover arquivos grandes, Mover ou renomear caminhos, Converter finais de linha, Remover o bit de execução, Converter a codificação dos arquivos | [Arquivos e caminhos](#arquivos-e-caminhos) |
| LFS | Migrar arquivos para o LFS, Trazer arquivos de volta do LFS | [Mover arquivos para o Git LFS ou para fora dele](#mover-arquivos-para-o-git-lfs-ou-para-fora-dele) |
| Assinaturas | Remover assinaturas (e reassinar ao aplicar) | [Assinaturas](#assinaturas) |
| Script | Lua, shell, Python | [A regra de script](#a-regra-de-script) |

**Antes de começar**

- Uma prévia não altera nada. Aplicar altera só as suas refs locais e nunca
  faz push.
- Os commits reescritos ganham novos ids. Todos que têm o histórico antigo precisam ser avisados:
  [Enviar o resultado](#enviar-o-resultado) escreve a nota.
- Um segredo que foi publicado está vazado. **Troque-o primeiro**, depois reescreva,
  faça o push forçado e apague o histórico antigo.
- Os commits que continuam como estão mantêm os seus ids e as suas assinaturas.

## O painel de reescrita

`Ctrl+G Shift+H` (ou **Reescrita: painel do histórico** na paleta) abre o painel
de reescrita sobre o Log. Ele não tem botão na barra de ferramentas: as ações
avançadas ficam na paleta (cada linha `Reescrita: ...` o abre já com aquela regra
adicionada), no submenu **Reescrever histórico** do menu do commit (*Deste commit
até HEAD...*, *Commits selecionados...* quando vários estão selecionados, *Abrir o
painel de reescrita...*) e atrás do atalho.

`Ctrl+N` mostra um selo em cada controle do painel (escopo, regras, campos, abas, botões); pressionar um selo faz o que um clique nesse controle faz, e `Esc` tira os selos (veja [Selos](keybindings.md#selos)).

![O painel de reescrita do histórico: a barra de escopo, uma lista de duas regras Substituir um autor com os campos da regra selecionada e o resumo da prévia com quantos commits mudam](../images/history-rewrite.png)

- **Barra de escopo:** quais refs se movem (branch atual, todas as branches
  locais, branches e tags escolhidas, os commits selecionados), a caixa *Tags* e,
  depois de *Deste commit até HEAD...*, uma caixa para começar depois do pai
  desse commit.
- **Regras:** uma lista numerada, aplicada em ordem. A caixa liga ou desliga a
  regra, as setas a movem, a cruz a exclui (*Restaurar a regra excluída* ou
  `Ctrl+Z` a traz de volta). *Adicionar regra* lista os tipos por grupo. Uma regra
  que precisa de um programa que você não tem fica esmaecida, com o motivo.
- **Configurações da regra:** um clique em um campo de texto o edita; um clique
  em um campo com vários valores abre a lista; as caixas se alternam.
- **Prévia** (`F5`) calcula o novo histórico sem tocar em nada e mostra uma barra
  de progresso. As abas: **Resumo** (contagens, o que cada regra fez, avisos),
  **Commits** (uma linha por commit alterado, paginada pelo daemon; o filtro lista
  os alterados, removidos ou publicados), **Branches e tags** (o que se move, e as
  branches protegidas já publicadas, cada uma com uma caixa), **Impacto** (o que
  significa reescrever um histórico publicado: um push forçado com lease,
  colaboradores, assinaturas), **Segredos**, **Identidades** e **Tamanho** (veja
  abaixo).
- **Arquivos de um commit:** `Enter` (ou um clique duplo) em uma linha de Commits
  abre os arquivos que as regras de conteúdo alteraram nele e, ao lado, o diff do
  arquivo selecionado. A cruz, `Esc` ou um clique fora o fecha. As linhas
  removidas por uma regra de segredos ficam ocultas.
- **Aplicar** (`Ctrl+S` ou `Ctrl+Enter`) fica disponível quando a prévia
  corresponde às regras. As perguntas sem resposta padrão são feitas toda vez: o
  que fazer com as assinaturas dos commits assinados e como extrair uma pasta. O
  painel de resultado oferece **Desfazer**, que devolve cada ref movida ao que
  era antes.
- **Salvar / Carregar** guardam uma receita no repositório (para todos) ou só
  para você; os valores secretos nunca são salvos.

Cada botão tem uma tecla: a lista está em Configurações > Teclado (contexto
*Reescrita*) e no guia `keybindings.md`.

### Autores e o .mailmap

- **Substituir um autor** casa um autor ou committer por nome e/ou e-mail
  (um texto literal ou uma expressão regular) e define um novo nome e/ou e-mail. *Aplicar
  a* escolhe o autor, o committer ou os dois; *Também os autores das tags* faz as
  tags anotadas que são recriadas acompanharem. O committer original é mantido
  a menos que a execução diga *eu, agora*.
- **Anonimizar autores** troca cada identidade por um espaço reservado numerado
  (`Contributor 1`, `contributor-1@anonymous.invalid`; os dois padrões podem ser
  mudados por você, com `{n}` para o número). A mesma pessoa sempre recebe o mesmo número
  em uma execução; as identidades que você lista como *manter* ficam como estão.
- **A aba Identidades** (também **Reescrita: Merge de identidades (.mailmap)...** na
  paleta) é o jeito de corrigir uma pessoa que fez commits com vários nomes e
  e-mails **sem reescrever nada**. Ela lista todos os nomes e e-mails do escopo da
  barra de escopo, com quantos commits cada um escreveu e commitou, e quais o
  `.mailmap` do repositório já mapeia (`mapeada para Nome <e-mail>`). Marque as
  identidades que são a mesma pessoa (`Space` ou um clique; *Marcar todas* /
  *Desmarcar todas*) e use **Merge em...** (`m`): uma janela pergunta qual delas
  fica (a que tem mais commits vem primeiro) e **acrescenta ao `.mailmap` uma
  linha por identidade restante** (`Nome <e-mail> Nome Antigo <antigo@e-mail>`;
  uma linha que já existe não se repete, e o arquivo é criado se não existir).
  Nada é commitado: faça o commit do `.mailmap` você mesmo para compartilhá-lo. O
  Log, o blame e o histórico de arquivo mostram o nome canônico na hora
  (configuração `log.use_mailmap`, ativada por padrão; os commits gravados nunca
  mudam, então não há nada para forçar no push). A linha de status diz o que foi
  gravado e oferece **Desfazer** (`u`), que restaura o `.mailmap` anterior (é uma
  entrada do histórico de desfazer, como qualquer outra ação). `r` lê a lista de
  novo.
- **Um `.mailmap` em vez de uma reescrita.** Quando uma regra de autor altera
  commits que já estão em um remoto, o Resumo diz *Um .mailmap corrige a exibição
  sem reescrever. Usar o .mailmap?* com um botão **Usar o .mailmap** (`m` no
  Resumo). Ele abre a aba Identidades com as identidades que a regra casa já
  marcadas, e a identidade que a regra definiria é oferecida como a que fica,
  mesmo que nenhum commit ainda a tenha.
- **A regra `.mailmap`** (*Reescrita: aplicar o .mailmap ao histórico...*, ou
  `rewrite preview --mailmap`) reescreve de verdade: autores e committers (à sua
  escolha) passam a ter o nome que o `.mailmap` da árvore de trabalho lhes dá, o
  resultado que `git log --use-mailmap` mostra. Use quando o próprio histórico
  precisa carregar os nomes canônicos, por exemplo antes de publicá-lo; é uma
  reescrita como qualquer outra (prévia, Aplicar, push forçado).
- **Regras de identidade e trailers.** Uma regra de identidade também pode mudar as pessoas
  citadas em trailers: marque **Mapear também os trailers** (`--map-trailers`). Todo trailer
  cujo valor é exatamente `Nome <e-mail>` e casa com *O e-mail é* /
  *O nome é* da regra recebe o novo nome e e-mail, seja o que a regra fizer com o
  autor e o committer. Vem desativado: um `Signed-off-by` registra quem
  assinou, então mudá-lo é uma decisão à parte.

### Datas

As quatro regras de data atuam sobre a data do autor, a data do committer ou as duas (*Datas*
no painel da regra).

- **Deslocar datas** move todas as datas por um valor (`+2h`, `-1d`, `90m`, `1d 2h`). O fuso
  horário em que cada commit foi escrito é mantido.
- **Mudar o fuso horário** escreve as datas em outro deslocamento UTC. *Manter* diz o que
  fica: o **instante** (o mesmo momento, o relógio marca outra hora) ou o
  **relógio** (a mesma hora local, o instante muda).
- **Manter dentro de um horário** move cada commit que cai fora dos horários permitidos
  (por exemplo `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) para o horário permitido
  mais próximo, ou só para antes, ou só para depois (*Mover para*). *Nunca antes do
  commit pai* mantém a ordem do histórico; a *Semente* torna repetível a pequena
  dispersão aleatória: a mesma semente no mesmo histórico dá as mesmas datas.
- **Data do committer = data do autor** define o committer (nome, e-mail e data) como o
  do autor, o que um `git commit --amend` ou um rebase desfez.

As datas são regras como as outras: gere a prévia, confira a aba **Commits** (cada linha
mostra a data antiga e a nova) e aplique. O id de um commit depende das datas dele,
então uma regra de data muda todo commit depois do primeiro que ela toca.

### Mensagens

- **Localizar e substituir nas mensagens** e **Excluir linhas das mensagens** aceitam um texto
  literal ou uma expressão regular (*Expressão regular*, *Diferenciar maiúsculas*); uma
  substituição vazia exclui. Uma mensagem que não é UTF-8 válido nunca é tocada por
  elas: execute antes **Corrigir a codificação das mensagens** se quiser editá-la.
- **Trailers.** *Adicionar ou remover um trailer* (*Reescrita: adicionar ou remover trailers...*
  na paleta, ou `--trailer-add`, `--trailer-remove`, `--trailer-map`) edita
  as linhas `Chave: valor` no fim de uma mensagem, as que o `git
  interpret-trailers` conhece (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...).
  A lista **Ação** escolhe *Adicionar* (a chave do **Trailer**, o **Valor** e
  **Se já existir**: *Adicionar (a menos que seja idêntico)*, *Substituir os existentes*
  ou *Não mexer no commit*), *Remover* (todo trailer dessa chave, ou
  só os cujo valor contém **O valor contém**) ou *Alterar o valor*
  (**Localizar no valor** passa a **Substituir por**, por exemplo um e-mail antigo por um
  novo em `Signed-off-by`). As chaves são comparadas sem diferenciar maiúsculas. Só as
  linhas de trailer mudam, o resto da mensagem continua exatamente como estava, e uma
  mensagem que ainda não tem trailers ganha uma linha em branco e um bloco só dela. A
  aba Commits mostra cada mudança como `+Signed-off-by: ...` / `-Co-authored-by: ...`.
  O último parágrafo de uma mensagem é o bloco de trailers (nunca o título), então uma
  linha como `Note: see the ticket` no corpo não é um trailer, a menos que esteja
  sozinha no último parágrafo com outros trailers.
- **Ids de commit nas mensagens.** **Atualizar os ids de commit** (`--update-message-hashes`,
  desativado por padrão; no fim dos painéis das regras que editam mensagens) faz os
  ids escritos nas mensagens acompanharem a reescrita: um id abreviado de 7 ou mais caracteres
  hexadecimais (`This reverts commit 1a2b3c4`, `see 1a2b3c4d5e`) que corresponde a exatamente
  um commit do histórico reescrito é trocado pelo novo id desse commit, cortado no
  mesmo tamanho. Um id que casa com vários commits, com nenhum, ou com um commit que a
  reescrita cria depois é deixado como está; o Resumo diz quantos foram
  trocados e quantos ficaram.
- **Corrigir a codificação das mensagens** (*Reescrita: corrigir a codificação das mensagens...*, ou
  `--fix-message-encoding [--fix-name-encoding]`) recodifica as mensagens para UTF-8,
  em dois casos:
  - Um commit com um cabeçalho `encoding` (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, qualquer um que o git e as tabelas de codificação conheçam) é decodificado e
    gravado em UTF-8, e o cabeçalho sai: o texto se lê igual em todo lugar,
    sem conversão. Os nomes do autor e do committer desse commit estavam na
    mesma codificação e são decodificados junto, então um commit nunca fica meio
    convertido.
  - O **mojibake**, UTF-8 que foi lido antes como Latin-1 ou Windows-1252
    (`RÃ©Ã©criture` em vez de `Réécriture`), é reparado **só quando a ida e volta é
    exata**: o texto precisa se codificar de volta em bytes que sejam UTF-8 válido, e
    corromper o resultado do mesmo modo precisa dar o original. Uma mensagem que apenas
    contém `Ã©` ao lado de letras de verdade, ou caracteres fora dos alfabetos antigos,
    não passa e fica como está.

  *Corrigir também os nomes* (`--fix-name-encoding`) repara do mesmo modo nomes de autor e
  committer corrompidos. O que não dá para decidir (um nome de codificação desconhecido, bytes
  que não são válidos na codificação declarada, uma mensagem que não é UTF-8 e
  não declara nada, um texto que parece corrompido mas sem prova disso) é **deixado
  como está e contado**: o Resumo diz *n mensagens foram mantidas como estão: não dá para decidir a
  codificação delas*, e `counts.encoding_left` tem o número. A aba
  Commits mostra cada correção com a mensagem antiga e a nova e uma linha *codificação*
  (`ISO-8859-1` vira `UTF-8`).

### Arquivos e caminhos

- **Regras de conteúdo dos arquivos** (*Localizar e substituir nos arquivos*,
  *Remover arquivos do histórico*, *Remover arquivos grandes*): o painel confere
  cada valor enquanto você digita e mostra o motivo sob o campo (um padrão que
  não compila, um padrão de caminho inválido, um tamanho abaixo de um byte). Os
  tamanhos aceitam unidades (`500 KiB`, `5 MiB`, `1 GiB`). Um padrão de caminho
  sem barra casa em qualquer profundidade. Os *Valores secretos* são digitados em
  um campo mascarado, usados só nesta execução e nunca salvos; a prévia oculta as
  linhas que os continham. *Commits vazios* define se um commit que as regras
  deixaram vazio é removido (padrão) ou mantido. Arquivos binários e arquivos
  acima do limite de tamanho ficam como estão e são contados no Resumo.
- **Mover caminhos.** *Mover ou renomear caminhos* (*Reescrita: mover ou renomear caminhos...* na
  paleta) tem três tipos, escolhidos na lista **Mover**:
  - *Renomear um caminho* (**De**, **Para**; `--move-path DE=PARA`): os
    mesmos arquivos e pastas aparecem no novo caminho em todo commit que tinha o
    caminho antigo; nada mais muda, cada arquivo mantém o seu id. Um commit em que o
    novo caminho já existe nunca é sobrescrito: a prévia é recusada e
    lista os commits e os caminhos em conflito (escolha um nome livre, ou mova antes o
    existente). Só caminhos exatos, sem curingas.
  - *Mover tudo para uma pasta* (**Pasta**; `--into-subdir DIR`):
    todo caminho de todo commit ganha o prefixo. Útil para juntar repositórios
    sem conflito de nomes.
  - *Extrair uma pasta como raiz* (**Pasta**; `--extract DIR`): a pasta vira
    a raiz, os commits que não a tocam são descartados (um commit que a cria
    fica sem pai), os merges que deixam de juntar algo são simplificados, e as
    branches e tags que não têm nada da pasta são excluídas. Você é perguntado
    **a cada vez** como termina:
    - *Em um novo clone, aberto em uma nova aba* (`--extract-mode clone`): uma cópia
      deste repositório é feita em uma pasta que você escolhe (o novo repositório
      ganha dentro dela uma pasta com o nome da extraída), reescrita
      ali e limpa para guardar só o novo histórico, com toda branch
      local e tag que tem a pasta. Este repositório não é tocado. O
      novo repositório não tem remoto (o original não é o upstream dele).
    - *No lugar: o histórico deste repositório é reescrito* (`--extract-mode
      in-place`): a reescrita de sempre, prévia, Aplicar, desfazer. As branches sem
      a pasta são excluídas (desfazer as traz de volta); a branch em checkout
      precisa tê-la.
    - *Como submódulo deste repositório* (`--extract-mode submodule`): o novo
      repositório é feito como no caso do clone, e depois **um novo commit** na
      branch atual troca a pasta por um submódulo dele (`.gitmodules`
      e o vínculo), e a pasta continua como um submódulo em checkout. O histórico
      deste repositório **não** é reescrito: os commits antigos mantêm os arquivos
      da pasta, então nada que aponte para eles (tags, outros clones)
      quebra. A URL em `.gitmodules` é o caminho local do novo
      repositório: mude-a (`git config -f .gitmodules submodule.<pasta>.url`)
      quando ele estiver em um host. Arquivos editados ou não rastreados na pasta o
      recusam. Desfazer devolve a pasta comum.
    O Resumo diz que a prévia mostra o histórico do novo repositório e para
    qual pasta ele vai. As regras que vêm depois de uma extração (uma renomeação, uma
    remoção) valem para a árvore extraída; para um clone ou um submódulo a
    extração precisa ser a primeira regra de caminho. As regras de caminho rodam depois das regras de
    conteúdo, então um glob de remoção nomeia os caminhos como eram.

#### Finais de linha, o bit de execução e codificações de arquivo

Três regras no grupo **Conteúdo** do menu Adicionar (ou as linhas da paleta
*Reescrita: converter finais de linha...*, *Remover o bit de execução...* e
*Reescrita: converter a codificação dos arquivos...*). As três aceitam **Arquivos** (globs; um padrão
sem barra casa em qualquer profundidade; vazio significa todos os arquivos) e **Exceto os
arquivos**. Elas mudam só o que os globs nomeiam, em todo commit do escopo.
Arquivos binários (um byte NUL nos primeiros 8 KiB) nunca são tocados: o Resumo
os conta como *mantidos como estão*.

- **Converter finais de linha** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Converter para*: LF (Linux, macOS), CRLF (Windows) ou
  CR (Mac clássico). Todo final de linha de um arquivo de texto que casa passa a ser esse,
  então um arquivo com finais mistos sai uniforme; arquivos que já estão certos mantêm a
  versão intacta. *Seguir o .gitattributes* (desativado por padrão,
  `--eol-follow-attributes`) lê o `.gitattributes` da raiz de cada commit: os arquivos
  marcados com `-text` ou `binary` são ignorados e os marcados com `eol=lf` ou
  `eol=crlf` mantêm esse final em vez do que você escolheu.
- **Remover o bit de execução** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Os arquivos que casam passam do modo 100755 para 100644; o conteúdo deles não
  é tocado. É preciso pelo menos um padrão de arquivo: remover o bit de todos os
  executáveis quebraria os seus scripts.
- **Converter a codificação dos arquivos** (`--reencode DESTINO [--from ORIGEM]
  [--encoding-fallback COD] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Converter para*: UTF-8 (o padrão), UTF-8 com BOM, UTF-16 LE ou
  BE (gravado com marca de ordem de bytes), Windows-1252 ou ISO-8859-1. *Os arquivos estão
  em*: por padrão a codificação de cada arquivo é detectada (uma marca de ordem de bytes, senão
  UTF-8 válido, senão a *Alternativa*, Windows-1252 a menos que você mude); informe
  a codificação para forçá-la, mas uma escolha errada dá texto errado (UTF-8 lido como
  Latin-1 duplica cada acento). Um arquivo que a codificação de origem não consegue ler, ou
  que tem um caractere que o destino não consegue guardar (um símbolo de euro em ISO-8859-1),
  continua exatamente como está e é contado no Resumo.

As regras rodam na ordem da lista, sobre o mesmo arquivo: converter um arquivo para
UTF-16 e depois pedir finais de linha o deixa em paz (ele deixou de ser
texto simples). Aplicar é uma entrada na lista de desfazer, como qualquer reescrita.

### Procurar segredos

- **Procurar segredos:** `gitpoulpi rewrite scan-secrets` (e a ferramenta MCP
  `rewrite_scan_secrets`) lê o histórico uma vez e lista prefixos de token
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), chaves
  privadas, JWTs, senhas em URLs e linhas `password = valor` com cara de
  aleatórias, cada uma com o arquivo, o primeiro e o último commit e uma prévia
  ocultada como `ghp_****abcd`. A busca não é exaustiva, e o valor nunca é
  mostrado, registrado nem salvo. `--remove-finding ID` transforma um achado em
  uma regra que o remove de todos os commits. Um segredo que foi publicado está
  vazado: troque-o primeiro, depois reescreva, faça o push forçado e apague o
  histórico antigo.
- **A aba Segredos** faz o mesmo no painel, para o escopo da barra de escopo:
  **Procurar segredos** (`s`) executa a busca como uma tarefa (a barra de
  progresso mostra *Procurando*, e **Cancelar** ou `Esc` a interrompe). Cada
  achado é uma linha com o tipo, a prévia ocultada, o arquivo e quantos outros
  também o contêm; sob a lista ficam o primeiro e o último commit em que apareceu
  e quantos commits e versões de arquivo o contêm. A aba informa quantos arquivos
  binários e grandes demais não foram lidos, e que a busca não é exaustiva. Todos
  os achados começam marcados; **Remover do histórico** (`r`) transforma os
  marcados em uma única regra que os nomeia pelo id (nunca pelo valor), que você
  visualiza e aplica como qualquer outra. Os achados ficam desatualizados depois
  de uma reescrita ou de uma mudança de escopo: a aba avisa, e é preciso procurar
  de novo. O painel nunca guarda o valor de um segredo, então não tem como
  mostrá-lo.

### Descobrir o que pesa

- **Descobrir o que pesa:** `gitpoulpi rewrite analyze [--top N]` (e a ferramenta
  MCP `rewrite_analyze`) é a análise de tamanho, como `filter-repo --analyze`: os
  maiores arquivos com caminho, tamanho, primeiro e último commit e se ainda estão
  no HEAD, as maiores pastas, os arquivos excluídos do HEAD que o histórico ainda
  guarda, e os totais. Ela lê só tamanhos e caminhos, nunca o conteúdo dos
  arquivos, e não altera nada.
- **A aba Tamanho** faz o mesmo no painel (`F7`, ou *Analisar o tamanho do
  histórico* na paleta, a abre e executa): **Analisar tamanhos** (`s`) é uma
  tarefa (*Analisando*, e **Cancelar** ou `Esc` a interrompe). **Mostrar** escolhe
  a lista (`v`): *Arquivos* (as maiores versões únicas de arquivo, cada uma
  marcada como *no HEAD*, *versão antiga* de um arquivo que ainda existe, ou
  *excluído*), *Pastas* (tudo o que já foi guardado sob cada uma) ou *Excluídos*
  (caminhos que saíram do HEAD e que o histórico guarda). **Ordenar** (`o`) é por
  tamanho, por caminho ou pelos excluídos primeiro. Marque linhas (`Space`,
  `Alt+A` / `Alt+U` para todas / nenhuma): a aba informa quanto elas ocupam,
  honestamente como "até cerca de", porque o espaço só volta depois da
  reescrita, de apagar os objetos antigos e de um `gc`. **Remover do histórico**
  (`r`) transforma os caminhos marcados em uma regra de caminho (caracteres
  especiais escapados; um caminho que já está em uma regra não é adicionado duas
  vezes); **Remover todo arquivo acima de ...** (`b`, lista Arquivos) escreve uma
  regra de tamanho logo abaixo do menor arquivo marcado. Gere a prévia e aplique
  como qualquer outra regra. As listas mostram as 200 maiores linhas; a CLI diz
  quantas há a mais e `--top` as lista. As linhas ficam desatualizadas depois de
  uma reescrita ou de uma mudança de escopo: analise de novo.

### Assinaturas

- **Assinaturas:** um commit reescrito não pode manter a assinatura (ela assinou os
  bytes antigos), então, quando commits assinados são reescritos, você é perguntado, toda vez:
  *Remover as assinaturas* (a contagem aparece antes), *Reassinar só os commits
  que eram assinados* ou *Reassinar todo commit reescrito*. Não há padrão.
  Os commits que continuam como estão mantêm a deles. A reassinatura usa a sua própria
  configuração, como um commit assinado faz: `gpg.format` (`openpgp`, `ssh` ou `x509`),
  `user.signingkey` e `gpg.program` / `gpg.ssh.program` / `gpg.x509.program`.
  A quarta linha da pergunta, **Verificar a assinatura** (`4`), assina uma carga
  fictícia uma vez e a verifica: diz quanto tempo leva uma assinatura e
  estima a execução para o número de commits em questão, ou por que não
  funciona (chave ausente, agente bloqueado, programa não instalado, com como instalá-lo).
  A prévia faz o mesmo teste antes de ler o histórico e recusa
  com esse motivo, então uma configuração quebrada nunca deixa uma reescrita meio assinada. Os
  commits são assinados **durante a prévia** (os ids deles contêm a assinatura),
  então uma prévia que reassina leva o tempo da assinatura, o Resumo diz *Commits
  reassinados: n, em 12,5 s*, e um pedido de frase secreta pode abrir nessa hora: desbloqueie
  antes o seu agente (Verificar a assinatura faz exatamente isso). **Cancelar** para entre
  duas assinaturas e não deixa nada para trás. Só as assinaturas de commit são
  recriadas; uma tag assinada continua perdendo a assinatura.
- **A regra *Remover assinaturas*** (menu Adicionar, grupo *Assinaturas*; *Reescrita:
  remover assinaturas...* na paleta; `--remove-signatures`) remove a
  assinatura de todo commit do escopo, mesmo onde nada mais muda, e não
  pergunta nada sobre reassinar. Pela CLI: `rewrite check-signing` e
  `--signatures strip|resign-signed|resign-all`.

### Desfazer uma reescrita

**Desfazer** (o botão do painel de resultado, o *Desfazer* da barra de ferramentas com `Ctrl+Z` no Log, ou `gitpoulpi undo`) devolve cada
branch, tag e nota movida, e os arquivos em checkout junto, a partir dos
commits antigos que a reescrita fixa. Ele pergunta antes quando uma branch reescrita já
está no upstream dela (desfazer localmente não desfaz um push) e quando você tem
alterações não commitadas: elas são mantidas, e um arquivo que você mudou e que o histórico
antigo sobrescreveria interrompe o desfazer (os arquivos se movem como
`reset --keep` os move). Desfazer deixa de
ser possível para uma reescrita depois que você [apaga](#apagar-o-histórico-antigo-purge) o
histórico antigo dela; a lista do histórico passa a marcá-la como *apagada*.

## Mover arquivos para o Git LFS ou para fora dele

Duas regras do grupo *LFS* do menu Adicionar reescrevem o histórico para que arquivos grandes
vivam no [Git LFS](lfs.md) desde o início (ou saiam dele). Elas precisam do programa
`git-lfs`: sem ele as regras ficam esmaecidas com a dica de instalação e a
prévia é recusada. São o nosso próprio motor, não o `git lfs migrate`, então você
tem a prévia, o escopo, o desfazer e a proteção de push de qualquer outra regra.

- **Migrar arquivos para o LFS** (*Reescrita: migrar arquivos para o LFS...*, ou
  `--lfs-import PADRÃO` com `--lfs-min-size BYTES`). Todo arquivo que casa com
  os padrões (`*.psd`, `assets/**`; um padrão sem barra casa em qualquer
  profundidade) e tem pelo menos o tamanho de *Só arquivos de pelo menos* (vazio: todo
  arquivo que casa) vira um ponteiro LFS em todo commit que o contém. O conteúdo vai para
  o armazenamento LFS local (`.git/lfs/objects`, uma vez por conteúdo distinto) quando você
  aplica, e o `.gitattributes` da raiz ganha `<padrão> filter=lfs diff=lfs
  merge=lfs -text` em todo commit reescrito que contém um ponteiro (criado quando
  não existe, as outras linhas intactas, nunca duplicado). Um arquivo que já é
  um ponteiro é deixado como está. O Resumo diz *3 arquivos (12 MiB) vão para o
  LFS. Objetos LFS a gravar: 3.* e a aba Commits lista *move big.psd para o
  LFS* por commit.
- **Trazer arquivos de volta do LFS** (*Reescrita: trazer arquivos de volta do LFS...*, ou
  `--lfs-export PADRÃO`). Os ponteiros que casam voltam a ser os arquivos reais, e
  as linhas do `.gitattributes` desses padrões são removidas. Os objetos precisam estar
  no armazenamento LFS local: quando faltam, a prévia é recusada com a
  lista e um botão **Baixar objetos LFS** que os busca em `origin`;
  gere a prévia de novo depois. Uma importação seguida de uma exportação dos mesmos
  padrões devolve exatamente os mesmos commits.

Depois de Aplicar, os arquivos em checkout são o conteúdo real (não ponteiros) e
**Desfazer** devolve o histórico antigo. O painel de resultado tem **Enviar objetos LFS**
(`l`): o mesmo que `LFS: Push All Objects`, para que o novo histórico possa ser enviado
sem que um push posterior trave por objetos ausentes (um push simples das branches
reescritas também os envia pelo hook de pre-push que o Aplicar instala). A nota
para os colaboradores diz que eles precisam do git-lfs e de `git lfs pull`.

Bom saber: Aplicar instala os filtros LFS do repositório (`git lfs install
--local`) quando faltam, porque o git precisa deles para comparar os arquivos reais
com os ponteiros. Só o `.gitattributes` da raiz é escrito, nunca um aninhado. Um arquivo
*abaixo* do tamanho mínimo que casa com um padrão continua um arquivo comum,
mas o git vai querer transformá-lo em ponteiro no próximo add, então o `git
status` o mostra como modificado até você commitá-lo. A importação processa
arquivos grandes em fluxo, a exportação mantém um arquivo por vez na memória.

## Enviar o resultado

Aplicar uma reescrita nunca faz push. As branches e tags reescritas que estavam
em um remoto precisam de um push forçado, e esse push nunca força às cegas: cada
ref é enviada com um lease sobre o id que o remoto tinha quando você gerou a
prévia (não o que a sua branch de rastreamento diz agora), então um push de um
colega no meio do caminho é rejeitado para aquela ref em vez de sobrescrito, e as
refs vão em um único push tudo ou nada (`--atomic`) quando o servidor permite.
Pela CLI:

```
gitpoulpi rewrite push --dry-run        # pergunta ao remoto, não envia nada
gitpoulpi rewrite push --yes            # envia
gitpoulpi rewrite note                  # o que os outros precisam executar
gitpoulpi rewrite maps --out ./maps     # commit-map e ref-map
```

As branches protegidas (configuração `rewrite.protected_branches`, por padrão
`main`, `master`, `develop`, `release/*`) ficam de fora do push, na reescrita e
de novo no push, a menos que você as cite (`--include-protected main`) ou inclua
todas. A proteção de branch do próprio host ainda decide se ele aceita um push
forçado; a recusa dele aparece por ref.

No painel, o painel de resultado depois de um Aplicar faz tudo isso:

- **Enviar as refs reescritas ao remoto** lista as refs que a reescrita moveu e
  que um remoto tem, cada uma com uma caixa (`Space`). As branches protegidas
  começam desmarcadas, com *protegida* ao lado: marque-as uma a uma, ou use
  *Marcar todas* / *Desmarcar todas* (`Alt+A` / `Alt+U`). **Push das branches
  reescritas** (`p`) envia as marcadas; a resposta é uma linha por ref (*push
  feito*, *já está atualizada*, *rejeitada: o remoto mudou desde a prévia*,
  *rejeitada: protegida pelo host*, *ignorada: branch protegida*, *sem push*).
  Uma linha rejeitada tem **Fetch e revisar** (`f`): o painel fecha e os remotos
  são atualizados, para o Log mostrar o que os outros enviaram. Nada é
  sobrescrito: o lease protege o trabalho deles, e você gera a prévia de novo
  antes de tentar outra vez.
- **Copiar a nota** (`c`) coloca na área de transferência a nota para os
  colaboradores, no idioma da interface em volta dos comandos git, que não são
  traduzidos.
- **Abrir o mapa de commits** (`m`) mostra onde estão o mapa de commits e o mapa
  de refs; um clique em um caminho (ou `1` / `2`) o copia.

Um push não pode ser desfeito: se você usar `undo` na reescrita depois, só o seu
repositório muda, e o desfazer avisa isso antes de perguntar. Envie a nota a todos
que têm o histórico antigo: ela lista a ponta antiga e a nova de cada branch e
tag e os comandos (`git fetch`, depois `git reset --hard origin/<branch>` ou `git
rebase --onto`). Os dois mapas ficam guardados por 24 horas junto com a execução;
copie-os se a limpeza de repositório do GitLab ou um script precisar deles.

## Quando o host protege uma branch

O seu host pode proteger uma branch por conta própria (proteção de branch e
rulesets do GitHub, branches protegidas do GitLab, políticas de branch do Azure DevOps,
restrições de branch do Bitbucket) e recusar o push forçado de que uma reescrita precisa. Quando o
repositório está **vinculado** ao host (a seção de pull requests da barra lateral,
veja [Integrações de hospedagem](hosting-integration.md)), o painel pergunta ao
host quais branches ele protege, com o seu próprio token:

- Essas branches ficam **de fora da reescrita por padrão**, exatamente como as
  branches protegidas de `rewrite.protected_branches` e do fluxo de trabalho: a aba
  Refs as lista, cada uma com uma caixa, e os botões *Marcar todas* / *Desmarcar todas*,
  e diz o motivo: *protegida em GitHub*.
- Se o token não consegue ler a proteção (falta o escopo, ou você não é
  administrador), o painel diz *Proteção do host desconhecida* e não trata
  nada como protegido só pela palavra do host. A reescrita nunca é bloqueada por
  isso. Um repositório que não está vinculado nunca é consultado.

Quando uma branch protegida está no caminho (a prévia foi recusada porque todas as
branches são protegidas, ou o host rejeitou o push: *rejeitada: protegida pelo
host*), o botão **Como contornar** (`Alt+H`) explica os três caminhos,
com a página exata de configurações do seu provedor e repositório (selecione um
link e copie com `c`, ou abra com `o`). O GitPoulpi nunca faz nenhum deles
por você:

1. **Suspender a proteção temporariamente** nas configurações do host, fazer o push e depois
   restaurá-la.
2. **Fazer push da reescrita com outro nome de branch**, depois tornar essa branch a
   branch padrão (ou renomear) e excluir a antiga quando ninguém mais a usar.
3. **Como último recurso, excluir e recriar o repositório remoto** e enviar
   tudo. Isso perde os pull / merge requests e as revisões deles, as
   issues, o vínculo dos forks com o histórico antigo, estrelas e observadores, as
   configurações, segredos e variáveis de CI, e os webhooks e chaves de deploy.

Pela CLI o mesmo texto volta como `protected_by_host` no JSON de
`rewrite preview` / `rewrite run` (quando o host protege uma branch do
escopo), de um `rewrite push` recusado e de `rewrite note`: `provider`,
`branches`, `ways` (cada um com os seus `steps` e `links`), `recreate_loses` e a
explicação inteira como `text`.

## Apagar o histórico antigo (purge)

Depois de uma reescrita, os commits antigos ainda estão no seu repositório: o
desfazer da reescrita os mantém (é isso que o torna desfazível), e os reflogs
também. Se você reescreveu para se livrar de um segredo, esse segredo ainda está
na pasta `.git`. O purge apaga o histórico antigo de vez:

```
gitpoulpi rewrite purge                 # pergunta: lista o que é apagado e perdido (código de saída 3)
gitpoulpi rewrite purge --yes           # executa
```

Ele remove os pontos de desfazer da reescrita, expira as entradas do reflog que
citam os commits antigos, executa `git gc --prune=now` e confere que cada commit
e versão de arquivo antigos sumiu mesmo. A confirmação diz antes o que você perde:
**desfazer deixa de ser possível** para a reescrita (e para qualquer operação
anterior cujo desfazer também precisava dos commits antigos); elas continuam na
lista do histórico, marcadas como apagadas. Um purge não pode ser interrompido
depois de iniciado nem desfeito. Ele não toca em nenhum remoto.

Tudo o que ainda guarda o histórico antigo bloqueia o purge, e então nada é
alterado. A lista nomeia cada item com o que fazer: uma branch ou tag que não fez
parte da reescrita, um stash sobre o histórico antigo (descarte-o, ou mantenha-o
e aceite que o histórico antigo continue), outro worktree cujo HEAD esteja nele, e
as branches de rastreamento remoto que ainda apontam para o histórico antigo (faça
push das branches reescritas e depois fetch). A sua lista de stashes e os reflogs
das outras branches nunca são tocados.

Um segredo que foi publicado está vazado: **troque-o primeiro**. Depois do purge,
a resposta lista o que o host ainda exige, por remoto: o GitHub mantém commits
antigos acessíveis em visualizações em cache e nas refs de pull requests (contate
o GitHub Support, ou exclua e recrie o repositório), o GitLab tem uma limpeza de
repositório que recebe o `commit-map` (`gitpoulpi rewrite maps --out DIR`) mais a
manutenção, o Azure DevOps e o Bitbucket exigem um pedido de suporte ou um novo
repositório, e todo mundo que tem um clone ou um fork ainda tem o histórico
antigo.

### A tela do purge

No painel, o painel de resultado de uma reescrita tem um botão **Apagar o
histórico antigo...** (tecla `e`), e a paleta tem **Reescrita: apagar o histórico
antigo...** (age sobre a reescrita mais recente que ainda não foi apagada, também
depois de você fechar o painel). A tela lista o que o purge apaga e o que você
perde (as operações cujo desfazer vai junto; as marcadas com *perde também o
desfazer* não são reescritas, mas precisavam dos commits antigos) e depois o que
ainda guarda o histórico antigo, uma linha para cada:

- uma branch de rastreamento remoto: **Fetch** as atualiza a partir dos remotos;
- um stash: **Drop** pergunta antes, porque um stash é trabalho que você nunca
  commitou;
- uma branch, tag, worktree ou outra ref: **Copiar o nome**, e depois resolva onde
  ela mora (reescreva ou exclua a branch, faça checkout de uma branch reescrita
  no worktree).

A verificação roda de novo depois de cada correção, e **Verificar de novo** (`r`)
a executa sob demanda. **Apagar** (`p`) só fica disponível quando não sobra nada.
Ele faz uma última pergunta, numerada e sem resposta padrão (`1` apaga de vez, `2`
deixa tudo como estava), e então executa; a tela mostra a etapa em que está e diz
que não dá para interromper. O resultado diz que o histórico antigo sumiu deste
clone, ou quantos objetos antigos ainda restam. Abaixo, os passos a seguir em cada
remoto (um link pode ser copiado). A cruz, `Esc` e um clique fora fecham a tela,
exceto enquanto o purge roda.

Via MCP, a ferramenta de purge não existe, a menos que você defina
`allow_history_purge = true` na tabela `[mcp]` do `config.toml`.

## A regra de script

*Adicionar regra > Script* (ou **Reescrita: executar um script...** na paleta)
adiciona uma regra de script Lua e abre o editor dela sobre o painel; `Enter` na
linha *Script* da regra o abre de novo. O editor começa de um modelo pequeno.

- **Linguagem** (uma linha do painel da regra): *Lua (embutido)*, *Script de
  shell*, *Python* ou *Lua (instalado)*. Qualquer uma além do Lua embutido é um
  [script externo](#scripts-externos-shell-python-lua): sem isolamento, e uma
  linguagem desativada nas configurações ou cujo interpretador não existe fica
  esmaecida na lista, com o motivo ao lado (escolhê-la diz o que fazer). Trocar a
  linguagem de um script que você ainda não escreveu troca o modelo.
- **Script** é um editor de texto com números de linha. O que você digita já é a
  regra; não há nada a confirmar.
- Abaixo dele, a linha de **verificação** diz `OK: função rewrite_commit
  encontrada`, ou lista os problemas com a linha (`script:3: ...`; o número fica
  vermelho na margem). Ela roda um instante depois de você parar de digitar,
  nunca enquanto uma tecla está pressionada.
- **Parâmetros** tem uma linha por linha `--! param` do cabeçalho do script, com o
  rótulo ao lado do campo: texto, um número ou uma caixa de verdadeiro/falso. Um
  valor que você apaga volta ao padrão do cabeçalho; um parâmetro de texto sem
  padrão precisa ser preenchido antes da prévia.
- **Arquivo** (um caminho dentro do repositório, `.gitpoulpi/rewrite/script.lua`
  para começar) com **Carregar de um arquivo...** (substitui o texto) e **Salvar
  em um arquivo...**.
- **Testar na seleção** executa o script nos commits que você selecionou no Log,
  ou nos 20 mais recentes quando nenhum está selecionado, e não grava nada. A
  tabela lista cada commit (id curto), cada campo que ele mudaria como
  `antigo -> novo` e as linhas `log(...)` do script. **Cancelar** para de esperar.
  Se o script falha, o commit e o motivo são mostrados.
- A cruz, um clique fora do editor e `Esc` o fecham; `Tab` e `Shift+Tab` movem
  entre o editor, os parâmetros, o arquivo, os botões e os resultados; `F5` testa.

A prévia recusa um script que não compila: o painel da Prévia diz *um script
falhou* com a linha (ou, para uma falha em um commit, o id curto dele e o motivo).
Em uma versão sem scripts, o tipo continua no menu Adicionar, esmaecido.

## Escrever um script com IA

Com a IA ativada ([Assistência de IA](ai.md)), o editor de Script tem um botão
`✦ Escrever com IA...`, e a paleta tem `✦ Reescrita: escrever um script com IA...`,
que abre o painel em uma nova regra de Script com o diálogo aberto. Nada é enviado
antes de você pressionar **Enviar**: o diálogo mostra, enquanto você digita, o
texto exato que vai sair.

- **O que o script deve fazer?** Com as suas palavras: *mover cada commit feito à
  noite para a manhã seguinte*, *trocar meu e-mail antigo pelo novo, só antes de
  2024*.
- **Enviar uma amostra de N commits** (desligado): quando marcado, o autor, a data
  e o assunto dos commits que "Testar na seleção" usaria (a seleção do Log, senão
  os 20 mais recentes) são acrescentados para o modelo ver como são os seus dados.
  Nunca o conteúdo dos arquivos nem o corpo dos commits. Com a caixa desmarcada,
  nenhum dado de commit sai da máquina.
- **O que é enviado** é um painel somente leitura com um fundo um pouco mais escuro, recolhido
  quando o diálogo abre (o diálogo fica então curto). Clique no cabeçalho, pressione `Alt+W`, ou
  vá até ele com `Tab` e pressione `Enter` para mostrá-lo: ele lista as instruções (o contrato
  fixo do script e depois as suas instruções de IA) e o seu pedido, com uma barra de rolagem
  própria (arraste-a, use a roda sobre o painel, ou as setas, `PageUp`, `PageDown`, `Home` e
  `End` enquanto ele tem o foco). O perfil que os recebe está na linha de cima. O diálogo cresce
  para caber o painel, dentro da tela.
- **Enviar** (`Tab` até ele, `Enter`). Enquanto o modelo trabalha, o diálogo
  espera, e `Esc` cancela o pedido. Se falhar, o diálogo continua com o motivo,
  para você tentar de novo.

A resposta é conferida pela mesma verificação do daemon que um script digitado; um
erro de compilação volta ao modelo uma vez. O script então **substitui o texto do
editor** (o diálogo avisa quando o editor contém algo além do modelo inicial) e o
título do editor mostra *✦ escrito por IA: leia antes de gerar a prévia* até você
mudar o texto. Ele nunca é testado, pré-visualizado nem aplicado no seu lugar: leia-o,
pressione **Testar na seleção** e depois gere a prévia, como para qualquer script.
O script roda no mesmo isolamento de um que você escreveu (sem arquivos, sem
relógio, sem rede).

Pela linha de comando: `gitpoulpi ai rewrite-script "<instrução>" [--sample N]`
imprime o script e o resultado de `rewrite script check` sobre ele como JSON e não
executa nada (código de saída 2 quando ele ainda não compila). Usa-se o perfil
*Script de reescrita do histórico* de Configurações > IA.

## Criar regras com IA

Com a IA ativada ([Assistência de IA](ai.md)), o painel tem um botão
`✦ Criar regras com IA...` ao lado de Salvar e Carregar, a tecla `Alt+E` (*Criar regras
de reescrita com IA* em Configurações > Teclado, contexto *Reescrita*), e a paleta tem
`✦ Reescrita: criar regras com IA...`. Com a IA desligada, ou em uma compilação sem IA, o
botão e a linha da paleta não aparecem. Você descreve com as suas palavras como o
histórico deve ser reescrito (*trocar meu e-mail antigo a@b.org por c@d.org*, *remover
secrets.txt de todo o histórico*, *deslocar minhas datas em duas horas*) e o modelo
propõe as regras e as configurações, que **preenchem o painel**. Nada é enviado antes de
você pressionar **Enviar**: o diálogo mostra, enquanto você digita, o texto exato que vai
sair.

- **Como o histórico deve ser reescrito?** Um campo de várias linhas: `Enter` cria uma
  nova linha, `Ctrl+Enter` ou `Ctrl+S` envia, `Esc` e a cruz cancelam.
- **Enviar o escopo, os nomes de branches e tags e os autores do histórico**
  (desligado): quando marcado, o escopo da barra de escopo, os nomes de branches e tags e
  os autores distintos (nome e e-mail) dos commits mais recentes que o Log mostra (no
  máximo 20 commits) são acrescentados, para o modelo poder citar o seu e-mail com
  exatidão (o `ai rewrite-rules --context` da linha de comando lê os 200 commits mais
  recentes). Nunca o conteúdo dos arquivos nem as mensagens dos commits. Com a caixa
  desmarcada, só a sua descrição e as instruções fixas saem da máquina.
- **Substituir as regras atuais** (só quando o painel já tem regras; desligado): por
  padrão, as novas regras entram depois das que você já tem; marcado, elas as substituem
  (o escopo e as configurações que a resposta indica são aplicados nos dois casos).
- **O que é enviado** é um painel somente leitura com um fundo um pouco mais escuro, separado
  dos campos acima dele, e **recolhido** quando o diálogo abre (o diálogo fica então curto).
  Clique no cabeçalho, pressione `Alt+W`, ou vá até ele com `Tab` e pressione `Enter` para
  mostrá-lo: ele lista as instruções (o vocabulário das regras e depois as suas instruções de
  IA) e o seu pedido, e tem uma barra de rolagem própria (arraste-a, use a roda sobre o painel,
  ou as setas, `PageUp`, `PageDown`, `Home` e `End` enquanto ele tem o foco). O perfil que os
  recebe está na linha de cima. O diálogo cresce para caber o painel, dentro da tela. O campo de
  descrição ganha uma barra de rolagem própria quando o texto passa do seu tamanho.

O modelo pode usar todos os tipos de regra desta página, exceto **Script** (código:
*Escrever um script com IA* mostra o script para você revisar) e a forma de valores
secretos de *Localizar e substituir nos arquivos* (valores secretos nunca são entregues a
um modelo: use a aba Segredos). Ele pode definir o escopo como a branch atual, todas as
branches locais e tags, ou branches e tags nomeadas que existem, e as opções *Tags*,
*committer*, *commits vazios* e *ids abreviados nas mensagens*. O que o painel não
consegue expressar (um limite de data como *desde março*, um intervalo de commits) não é
adivinhado: o modelo diz isso em uma observação. O que acontece com as assinaturas nunca
é decidido pelo modelo: a pergunta vem no Aplicar.

**A IA só prepara regras e configurações.** Ela nunca gera prévia, aplica ou executa
nada: a resposta é copiada para o painel como uma receita carregada, a prévia na tela
fica desatualizada, e Prévia e Aplicar (com as suas perguntas e confirmações) ficam com
você, como sempre. Cada regra é analisada e conferida sozinha pela mesma verificação que
o editor e o motor usam; uma regra que falha é deixada de fora. Quando algo foi deixado
de fora, ou o modelo fez uma observação, uma lista mostra isso depois que as regras
entram (*Deixadas de fora (as regras válidas foram mantidas)*, cada uma com a posição, o
nome e o motivo), e `Enter` ou `Esc` a fecha. Leia as regras, ajuste-as e depois gere a
prévia.

Pela linha de comando: `gitpoulpi ai rewrite-rules "<descrição>" [--context]` imprime a
proposta como JSON (`recipe`, que `rewrite preview --recipe-json` lê, `problems`, `notes`
e `applied: false`) e não executa nada (código de saída 2 quando algo foi deixado de
fora). A ferramenta MCP é `ai_rewrite_rules`. Usa-se o perfil *Regras de reescrita do
histórico* de Configurações > IA.

## Scripts Lua

Uma regra de script executa a sua função em cada commit. É Lua 5.4 puro em uma
sandbox: sem arquivos, sem relógio, sem processos, sem módulos, sem `pcall`; o
mesmo script no mesmo histórico sempre dá o mesmo resultado, e uma chamada que
passa de 10 milhões de instruções, 64 MiB ou 5 segundos para com um erro que
nomeia o commit.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` contém `id`, `parents`, `tree`, `author` e `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` e `signed`. `ctx` contém `index`, `total`,
`params`, `state` (devolva `state = {...}` para levar dados ao próximo commit) e
`window.before` / `window.after` (defina `--! window: N`). Devolva só o que muda:
`author`, `committer`, `message`. Funções auxiliares: `regex_replace`,
`regex_match`, `stable_hash`, `rand`, `weekday`, `hour`, `log`, e para mensagens `trailers(message)`
(uma lista de `{key, value}`), `set_trailer(message, key, value)` (substitui os
trailers dessa chave; um quarto argumento `"add"` ou `"skip"` muda isso),
`remove_trailer(message, key)` e `at("2024-03-05 18:30", offset_minutes)`
(uma data em um deslocamento, como tempo Unix).

Pela linha de comando: `gitpoulpi rewrite script check my.lua` compila o script,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` mostra
o que ele alteraria sem gravar nada, e
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` monta o
plano da reescrita inteira.

## Scripts externos (shell, Python, Lua)

O mesmo contrato de um script Lua, executado por um programa da sua máquina:
*Script de shell* (bash ou sh; PowerShell no Windows, ou qualquer `.ps1`),
*Python* e *Lua (instalado)* (um `lua` seu, não o embutido; ele não tem
biblioteca JSON, então o modelo lê a linha com um padrão). **Eles não têm
isolamento: um script externo roda com os seus direitos** e pode ler ou alterar
tudo o que você pode. O painel da regra e o título do editor avisam isso.

Como ele conversa: o GitPoulpi inicia o script uma vez por execução e envia a ele
uma requisição JSON por linha na entrada padrão (os campos de um commit, `params`
e o `state` devolvido pela última resposta); ele responde com uma linha JSON na
saída padrão, com os mesmos campos que um script Lua devolve (`author`,
`committer`, `message`, `state`, `log`), ou `{"error": "texto"}` para desistir.
Imprima diagnósticos na saída de erro: eles aparecem como as linhas `log` do
commit. As linhas de cabeçalho começam com `#!` para shell e Python (`#! name: ...`,
`#! param old: text "Old"`; uma linha shebang continua sendo shebang) e com `--!`
para Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**O que ele recebe.** Um ambiente vazio, exceto `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` e `LANG`; uma pasta temporária nova como pasta de
trabalho (com a própria cópia do script); não o caminho do repositório. **Limites
de tempo:** `rewrite.script_timeout_ms` (5000) para cada resposta e
`rewrite.script_total_timeout_s` (3600) para a passagem inteira. Quando um limite
é excedido, o script trava, imprime algo que não é JSON ou a tarefa é cancelada, o
GitPoulpi encerra o script e tudo o que ele iniciou, e a reescrita para com o
commit e o motivo; nada foi movido.

**Quais linguagens existem.** `rewrite.external_scripts` no `config.toml` as
lista: `["shell"]` por padrão; acrescente `"python"` e `"lua"` você mesmo no
`config.toml` (`[rewrite]`, `external_scripts = ["shell", "python"]`; uma lista
não pode ser definida com `gitpoulpi config set`). Uma linguagem que não está na
lista fica esmaecida no painel e é recusada pela CLI e pelo MCP
(`scripts_disabled`); uma que está na lista mas cujo interpretador não é
encontrado fica esmaecida com a dica de instalação (`tool_missing`).

**Confiança.** Mesmo com uma linguagem ativada, um script só roda depois que você
confiou no texto exato dele. Um script que você digita no painel é seu e roda na
hora; um que vem com uma receita compartilhada ou com um arquivo do repositório
não é: a prévia e *Testar na seleção* param e perguntam *Confiar neste script?*
com duas respostas numeradas e nenhuma pré-selecionada (`1` confiar e continuar,
`2` não confiar, `Esc` deixa como está). Confiar guarda o SHA-256 do texto para
este repositório, então um script alterado pergunta de novo, e um repositório
clonado não consegue executar código em você por conta própria. Pela linha de
comando, leia o arquivo e depois use `gitpoulpi rewrite script trust ARQUIVO` (ou
`--trust` em `preview`, `run`, `script check` e `script test`, que confia e
continua). Um agente via MCP não pode confiar em nada: ele só executa um script em
que você já confiou antes.

**Verificação.** A linha de verificação sob o editor diz *Ainda não é confiável*
para um script sem confiança (nada foi iniciado); para um script confiável, o
GitPoulpi o inicia uma vez com um commit inventado (`"dry_run": true`) e confere
que a resposta segue o contrato. `gitpoulpi rewrite script check ARQUIVO` faz o
mesmo e informa `trust`, `hash` e `interpreter`.

## Pela linha de comando

`gitpoulpi rewrite` (e as ferramentas MCP de mesmo nome) usa o mesmo motor,
para scripts e agentes. Uma **receita** é um arquivo TOML ou JSON, o que o
painel salva (`--recipe ARQUIVO`, ou `-` para a entrada padrão), e as opções acrescentam regras
depois das da própria receita.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plano, não altera nada
gitpoulpi rewrite apply --run ID --digest D           # executa o plano que você revisou
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # devolve tudo
```

- Regras como opções: `--set-name` / `--set-email` com `--match-email` (identidade),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. O escopo é a branch atual, `--all-local` ou
  `--ref R` (repetível), a partir de `--since REV`.
- **As perguntas sem resposta padrão** são opções: `--signatures
  strip|resign-signed|resign-all` quando commits assinados mudam, e
  `--extract-mode` para uma extração. Sem elas a resposta é uma recusa ou um
  `needs_choice` (código de saída 3), nunca um palpite.
- **Códigos de saída:** 0 feito; 1 um erro (receita ilegível, opções inválidas); 2 recusado,
  com o motivo (`stale`, `signature_choice_needed`, `dirty_paths_touched`,
  `empty_scope`, ...); 3 uma pergunta a responder, ou refs enviadas que precisam de `--yes`.
- `preview` guarda o plano no repositório, então `apply` funciona a partir de um comando
  posterior, depois de reiniciar o daemon, ou a partir de outra interface; um plano que não
  corresponde mais ao seu digest é `stale`.

A [referência da CLI](cli-reference.md) tem todas as opções, e
[MCP e agentes](mcp-and-agents.md) diz quais ferramentas um agente recebe (a ferramenta de
purge não existe a menos que você a ative, e um agente não pode confiar em um script).

## Propostas de agentes de IA

Um agente pode **sugerir** uma reescrita, nunca executá-la. `gitpoulpi ui propose rewrite
--recipe ARQUIVO|-` (a ferramenta MCP `ui_propose_rewrite`, desativada até `[mcp]
allow_ui_propose = true`) abre o painel na sua janela em execução com as regras da
receita já na lista e um aviso roxo, *✦ Proposta por
<agente>: uma reescrita de histórico (3 regras) · Ctrl+Enter aceita · Esc rejeita*.

- **Nada roda.** Sem prévia, sem aplicar, sem push, sem purge: a proposta só
  preenche as regras. Você as lê, edita se quiser, gera a **Prévia** e aplica
  com a sua própria tecla. Fechar o painel rejeita a proposta.
- **O que o agente não pode pôr nela:** um script (rodaria na prévia),
  mais de 50 regras, valores secretos (eles são removidos da receita antes de
  ela ser enviada). Uma proposta assim é recusada com o motivo.
- O agente pode ler o resultado em `ui state` (`accepted` quando você aplicou
  as regras como propostas, `accepted_edited` quando você as mudou antes, `rejected`
  quando você fechou). Uma proposta por vez; ela é recusada enquanto outro
  diálogo seu está aberto.

## Ferramentas

Quase toda reescrita de histórico não precisa de nada além do Git: trocar autores
e committers, aplicar um `.mailmap`, anonimizar, deslocar datas ou limitá-las a
um horário, reescrever mensagens e trailers, substituir conteúdo, remover caminhos
ou arquivos grandes e mover caminhos rodam dentro do GitPoulpi.

**Os scripts Lua são embutidos.** O interpretador já vem no GitPoulpi, então um
script Lua não precisa de ferramenta nem de configuração. Scripts de shell,
Python e Lua instalado precisam do interpretador deles (veja a tabela abaixo) e
estão descritos em [Scripts externos](#scripts-externos-shell-python-lua).

Alguns recursos usam um programa externo. O GitPoulpi o procura (o programa é
encontrado no seu `PATH`, no Windows também por `PATHEXT` e pelo `usr/bin` do
próprio Git for Windows, e precisa responder a `--version` em até 2 segundos); a
resposta fica em cache por um minuto e "Verificar de novo" pergunta outra vez.
Uma regra que precisa de um programa ausente continua na lista, esmaecida, com o
motivo e como instalá-lo, e a prévia recusa antes de executar qualquer coisa.

| Recurso | Ferramenta | Observações |
|---|---|---|
| Reassinar commits | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) ou `gpgsm` (`gpg.format = x509`) | A assinatura SSH e X.509 também exige `user.signingkey`. `gpg.program`, `gpg.ssh.program` e `gpg.x509.program` são respeitados. |
| Migração para o LFS | `git-lfs` 3.0 ou mais novo | A mesma verificação do painel LFS. |
| Scripts Python | `python` 3.8 ou mais novo (`py -3` no Windows) | Desativado por padrão: acrescente `"python"` a `rewrite.external_scripts`. |
| Scripts Lua (instalado) | `lua5.4`, `lua54`, `lua` ou `luajit` | Desativado por padrão: acrescente `"lua"` a `rewrite.external_scripts`. |
| Scripts de shell | `bash`, `pwsh` (PowerShell) ou `sh` | Ativado por padrão (`"shell"`). Um script PowerShell (um arquivo `.ps1`, um shebang `pwsh` ou qualquer script de shell no Windows) precisa de `pwsh` ou `powershell`; um POSIX precisa de `bash` ou `sh` (no Windows, o Git for Windows os fornece). |

Dicas de instalação, começando pelo Windows:

- Windows: o Git for Windows já traz `gpg`, `ssh-keygen`, `bash`, `sh` e o Git
  LFS. Senão, `winget install GnuPG.GnuPG`, Configurações > Recursos opcionais >
  Cliente OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: os pacotes da distribuição (`gnupg`, `openssh`, `git-lfs`, `python3`,
  `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` e `sh` já
  vêm com o sistema.
