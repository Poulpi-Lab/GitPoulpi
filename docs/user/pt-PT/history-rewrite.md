[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · **Português (Portugal)** · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Reescrita do histórico

Reescrever o histórico altera commits que já existem: quem os escreveu e quando,
as suas mensagens, os ficheiros que contêm, as suas assinaturas. O GitPoulpi faz
isto com uma **receita** (um âmbito e uma lista ordenada de regras) e três passos
em que nunca se salta o do meio:

1. **Pré-visualizar** calcula o novo histórico sem tocar em nada: contagens, o que
   cada regra fez, as branches e tags que se moveriam, uma linha por commit
   alterado.
2. **Aplicar** move as suas branches e tags locais para ele. É uma entrada do
   [histórico de desfazer](undo-redo.md): **Desfazer** repõe todas as refs movidas
   (e os ficheiros em checkout).
3. **Push** (um push forçado com lease) e **Purgar** são passos separados e
   deliberados, porque não se desfazem: ver [Enviar o resultado](#enviar-o-resultado)
   e [Eliminar o histórico antigo](#eliminar-o-histórico-antigo-purga).

O painel, a paleta, os comandos `gitpoulpi rewrite` e as ferramentas MCP
correm o mesmo motor, por isso uma pré-visualização feita num deles é a que outro aplica.
[A partir da linha de comandos](#a-partir-da-linha-de-comandos) lista as opções;
[Criar regras com IA](#criar-regras-com-ia) preenche o painel a partir de uma descrição;
[Propostas de agentes de IA](#propostas-de-agentes-de-ia) diz o que um agente pode e
não pode fazer aqui.

**As regras, pelo que alteram**

| Grupo | Regras | Secção |
|---|---|---|
| Autores | Substituir um autor, Anonimizar autores, Aplicar .mailmap | [Autores e o .mailmap](#autores-e-o-mailmap) |
| Datas | Deslocar datas, Mudar o fuso horário, Manter dentro de um horário, Data do committer = data do autor | [Datas](#datas) |
| Mensagens | Procurar e substituir, Apagar linhas, Adicionar ou remover um trailer, Corrigir a codificação das mensagens | [Mensagens](#mensagens) |
| Conteúdo e caminhos dos ficheiros | Procurar e substituir nos ficheiros, Remover ficheiros, Remover ficheiros grandes, Mover ou renomear caminhos, Converter fins de linha, Remover o bit de execução, Converter a codificação dos ficheiros | [Ficheiros e caminhos](#ficheiros-e-caminhos) |
| LFS | Migrar ficheiros para o LFS, Trazer ficheiros de volta do LFS | [Mover ficheiros para dentro ou para fora do Git LFS](#mover-ficheiros-para-dentro-ou-para-fora-do-git-lfs) |
| Assinaturas | Remover assinaturas (e voltar a assinar ao Aplicar) | [Assinaturas](#assinaturas) |
| Script | Lua, shell, Python | [A regra Script](#a-regra-script) |

**Antes de começar**

- Uma pré-visualização não altera nada. Aplicar altera só as suas refs locais e
  nunca envia nada.
- Os commits reescritos ganham ids novos. Todos os que têm o histórico antigo têm de ser avisados:
  [Enviar o resultado](#enviar-o-resultado) escreve a nota.
- Um segredo que foi enviado está comprometido. **Renove-o primeiro** e só depois reescreva,
  faça o push forçado e purgue.
- Os commits que ficam como estão mantêm os seus ids e as suas assinaturas.

## O painel de reescrita

`Ctrl+G Shift+H` (ou **Reescrever: painel do histórico** na paleta) abre o
painel de reescrita sobre o Log. Não tem botão na barra de ferramentas: as
ações avançadas vivem na paleta (cada linha `Reescrever: ...` abre-o com essa
regra já adicionada), no submenu **Reescrever histórico** do menu do commit
(*Deste commit até HEAD...*, *Commits selecionados...* quando há vários
selecionados, *Abrir o painel...*) e atrás do atalho.

`Ctrl+N` mostra um selo em cada controlo do painel (âmbito, regras, campos, separadores, botões); premir um selo faz o que um clique nesse controlo faz, e `Esc` retira os selos (consulte [Selos](keybindings.md#selos)).

![O painel de reescrita do histórico: a barra de âmbito, uma lista de duas regras Substituir um autor com os campos da regra selecionada e o resumo da pré-visualização com quantos commits mudam](../images/history-rewrite.png)

- **Barra de âmbito:** que refs se movem (branch atual, todas as branches
  locais, branches e tags à escolha, os commits selecionados), a caixa *Tags* e,
  depois de *Deste commit até HEAD...*, uma caixa para começar depois do pai
  desse commit.
- **Regras:** uma lista numerada, aplicada por ordem. A caixa liga ou desliga
  uma regra, as setas movem-na, a cruz elimina-a (*Restaurar a regra eliminada*
  ou `Ctrl+Z` traz-a de volta). *Adicionar regra* lista os tipos por grupo. Uma
  regra que precisa de um programa que não tem fica esbatida, com o motivo.
- **Definições da regra:** um clique num campo de texto edita-o; um clique num
  campo com vários valores abre a respetiva lista; as caixas alternam.
- **Pré-visualização** (`F5`) calcula o novo histórico sem tocar em nada e
  mostra uma barra de progresso. Os separadores: **Resumo** (contagens, o que
  cada regra fez, avisos), **Commits** (uma linha por commit alterado,
  paginada a partir do daemon; o filtro lista os alterados, removidos ou
  enviados), **Branches e tags** (o que se move e as branches protegidas já
  enviadas, cada uma com uma caixa), **Impacto** (o que significa reescrever
  histórico já enviado: um push forçado com lease, colaboradores, assinaturas),
  **Segredos**, **Identidades** e **Tamanho** (veja abaixo).
- **Ficheiros de um commit:** `Enter` (ou um duplo clique) numa linha de
  Commits abre os ficheiros que as regras de conteúdo alteraram nesse commit e,
  ao lado, o diff do selecionado. A cruz, `Esc` ou um clique fora fecham-no. As
  linhas removidas por uma regra de segredos ficam ocultas.
- **Aplicar** (`Ctrl+S` ou `Ctrl+Enter`) fica disponível quando a
  pré-visualização corresponde às regras. As perguntas sem resposta
  predefinida são feitas de cada vez: o que acontece às assinaturas dos commits
  assinados e como se extrai uma pasta. O painel do resultado oferece
  **Desfazer**, que repõe todas as refs movidas.
- **Guardar / Carregar** mantêm uma receita no repositório (partilhada) ou só
  para si; os valores secretos nunca são guardados.

Cada botão tem uma tecla: a lista está em Definições > Teclado (contexto
*Reescrever*) e no guia `keybindings.md`.

### Autores e o .mailmap

- **Substituir um autor** apanha um autor ou committer pelo nome e/ou e-mail
  (um texto literal ou uma expressão regular) e define um novo nome e/ou e-mail.
  *Aplicar a* escolhe o autor, o committer ou ambos; *Também os autores de tags*
  faz com que as tags anotadas que são recriadas acompanhem. O committer original
  mantém-se, a menos que a execução diga *eu, agora*.
- **Anonimizar autores** substitui cada identidade por um marcador numerado
  (`Contributor 1`, `contributor-1@anonymous.invalid`; os dois padrões podem ser
  alterados por si, com `{n}` para o número). A mesma pessoa recebe sempre o mesmo
  número numa execução; as identidades que indicar como *manter* ficam como estão.
- **O separador Identidades** (também **Reescrever: Merge de identidades
  (.mailmap)...** na paleta) é a forma de corrigir uma pessoa que fez commits
  com vários nomes e e-mails **sem reescrever nada**. Lista todos os nomes e
  e-mails do âmbito da barra de âmbito, com quantos commits cada um escreveu e
  fez commit, e quais já estão mapeados pelo `.mailmap` do repositório
  (`mapeada para Nome <e-mail>`). Marque as identidades que são a mesma pessoa
  (`Espaço`, ou um clique; *Marcar todas* / *Desmarcar todas*) e depois
  **Merge em...** (`m`): um diálogo pergunta qual delas fica (a que tem mais
  commits vem primeiro) e **acrescenta ao `.mailmap` uma linha por cada outra
  identidade** (`Nome <e-mail> Nome Antigo <antigo@e-mail>`; uma linha que já
  lá esteja não se repete, e o ficheiro é criado se não existir). Nada é
  commitado: faça o commit do `.mailmap` você mesmo para o partilhar. O Log, o
  blame e o histórico de um ficheiro mostram logo o nome canónico (definição
  `log.use_mailmap`, ativa por predefinição; os commits guardados nunca mudam,
  por isso não há nada para enviar com push forçado). A barra de estado diz o
  que foi escrito e oferece **Desfazer** (`u`), que repõe o `.mailmap` anterior
  (é uma entrada do histórico de desfazer, como qualquer outra ação). `r` volta
  a ler a lista.
- **Um `.mailmap` em vez de uma reescrita.** Quando uma regra de autor altera
  commits que já estão num remoto, o Resumo diz *Um .mailmap corrige a
  apresentação sem reescrever. Usar o .mailmap?* com um botão **Usar o
  .mailmap** (`m` no Resumo). Abre o separador Identidades com as identidades
  que a regra apanha já marcadas, e a identidade que a regra definiria
  proposta como a que fica, mesmo que nenhum commit a tenha ainda.
- **A regra `.mailmap`** (*Reescrever: aplicar o .mailmap ao histórico...*, ou
  `rewrite preview --mailmap`) reescreve mesmo: autores e committers (à sua
  escolha) passam a ser o que o `.mailmap` da árvore de trabalho lhes chama, o
  resultado que `git log --use-mailmap` mostra. Use-a quando o próprio
  histórico tem de levar os nomes canónicos, por exemplo antes de o publicar; é
  uma reescrita como as outras (pré-visualização, Aplicar, push forçado).
- **Regras de identidade e trailers.** Uma regra de identidade também pode alterar as pessoas
  nomeadas em trailers: marque **Mapear também os trailers** (`--map-trailers`). Todos os trailers
  cujo valor seja exatamente `Nome <e-mail>` e corresponda ao *O e-mail é* /
  *O nome é* da regra passam a ter o novo nome e e-mail, faça a regra o que fizer ao
  autor e ao committer. Vem desligado por predefinição: um `Signed-off-by` regista quem
  assinou, por isso alterá-lo é uma decisão à parte.

### Datas

As quatro regras de datas atuam sobre a data do autor, a data do committer ou ambas (*Datas*
no painel da regra).

- **Deslocar datas** move todas as datas de uma certa quantidade (`+2h`, `-1d`, `90m`, `1d 2h`). O fuso
  horário em que cada commit foi escrito mantém-se.
- **Mudar o fuso horário** escreve as datas noutro desfasamento UTC. *Manter* diz o que
  fica: o **instante** (o mesmo instante, o relógio marca outra hora) ou o
  **relógio** (a mesma hora local, o instante muda).
- **Manter dentro de um horário** move cada commit que cai fora das horas permitidas
  (por exemplo `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) para a hora permitida
  mais próxima, ou só para mais cedo, ou só para mais tarde (*Mover para*). *Nunca antes do
  pai* mantém a ordem do histórico; a *Semente* torna repetível a pequena dispersão aleatória:
  a mesma semente sobre o mesmo histórico dá as mesmas datas.
- **Data do committer = data do autor** passa o committer (nome, e-mail e data) para o do
  autor, que é o que um `git commit --amend` ou um rebase desfez.

As datas são regras como as outras: pré-visualize, confira o separador **Commits** (cada linha
mostra a data antiga e a nova) e aplique. O id de um commit depende das suas datas,
por isso uma regra de datas altera todos os commits depois do primeiro que toca.

### Mensagens

- **Procurar e substituir nas mensagens** e **Apagar linhas da mensagem** aceitam um texto
  literal ou uma expressão regular (*Expressão regular*, *Distinguir maiúsculas*); uma
  substituição vazia apaga. Uma mensagem que não seja UTF-8 válido nunca é tocada por
  elas: corra primeiro **Corrigir a codificação das mensagens** se a quiser editar.
- **Trailers.** *Adicionar ou remover um trailer* (*Reescrever: adicionar ou remover trailers...*
  na paleta, ou `--trailer-add`, `--trailer-remove`, `--trailer-map`) edita
  as linhas `Chave: valor` no fim de uma mensagem, as que o `git
  interpret-trailers` conhece (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...).
  A lista **Ação** escolhe *Adicionar* (a chave do **Trailer**, o seu **Valor**, e
  **Se já existir**: *Adicionar (se não for idêntico)*, *Substituir os existentes*,
  ou *Deixar o commit como está*), *Remover* (todos os trailers dessa chave, ou
  só aqueles cujo valor contém **O valor contém**) ou *Alterar o valor*
  (**Procurar no valor** passa a **Substituir por**, por exemplo um e-mail antigo por um
  novo em `Signed-off-by`). As chaves são comparadas sem distinguir maiúsculas. Só as
  linhas dos trailers mudam, o resto da mensagem fica exatamente como estava, e uma
  mensagem que ainda não tem trailers ganha uma linha em branco e um bloco próprio. O
  separador Commits mostra cada alteração como `+Signed-off-by: ...` / `-Co-authored-by: ...`.
  O último parágrafo de uma mensagem é o seu bloco de trailers (nunca o título), por isso uma
  linha como `Note: see the ticket` no corpo não é um trailer, a não ser que esteja
  sozinha no último parágrafo com outros trailers.
- **Ids de commits nas mensagens.** **Atualizar os ids dos commits** (`--update-message-hashes`,
  desligado por predefinição; no fim dos painéis das regras que editam mensagens) faz com que
  os ids escritos nas mensagens acompanhem a reescrita: um id abreviado de 7 ou mais carateres
  hexadecimais (`This reverts commit 1a2b3c4`, `see 1a2b3c4d5e`) que corresponda a exatamente
  um commit do histórico reescrito é substituído pelo novo id desse commit, cortado para o
  mesmo comprimento. Um que corresponda a vários commits, a nenhum, ou a um commit que a
  reescrita cria mais tarde fica como está; o Resumo diz quantos foram
  substituídos e quantos ficaram.
- **Corrigir a codificação das mensagens** (*Reescrever: corrigir a codificação das mensagens...*, ou
  `--fix-message-encoding [--fix-name-encoding]`) recodifica as mensagens para UTF-8,
  em dois casos:
  - Um commit com um cabeçalho `encoding` (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, qualquer uma que o git e as tabelas de codificação conheçam) é descodificado e
    escrito em UTF-8, e o cabeçalho desaparece: o texto lê-se igual em todo o lado,
    sem conversão. Os nomes do autor e do committer desse commit estavam na
    mesma codificação e são descodificados com ela, por isso um commit nunca fica meio
    convertido.
  - **Mojibake**, o UTF-8 que foi lido como Latin-1 ou Windows-1252
    (`RÃ©Ã©criture` em vez de `Réécriture`), é reparado **só quando a ida e volta é
    exata**: o texto tem de voltar a codificar-se em bytes que sejam UTF-8 válido, e deturpar
    o resultado da mesma maneira tem de dar o original. Uma mensagem que apenas
    contém `Ã©` junto a letras verdadeiras, ou carateres fora dos alfabetos antigos,
    não passa e fica como está.

  *Corrigir também os nomes* (`--fix-name-encoding`) repara os nomes deturpados do autor e do committer
  da mesma maneira. O que não se consegue decidir (um nome de codificação desconhecido, bytes
  que não são válidos na codificação declarada, uma mensagem que não é UTF-8 e
  não declara nada, texto que parece deturpado mas não o é comprovadamente) é **deixado
  em paz e contado**: o Resumo diz *n mensagens ficaram como estavam: não se consegue decidir a sua
  codificação*, e `counts.encoding_left` tem o número. O
  separador Commits mostra cada correção com a mensagem antiga e a nova e uma linha *codificação*
  (`ISO-8859-1` passa a `UTF-8`).

### Ficheiros e caminhos

- **Regras de conteúdo dos ficheiros** (*Procurar e substituir nos ficheiros*,
  *Remover ficheiros do histórico*, *Remover ficheiros grandes*): o painel
  verifica cada valor à medida que escreve e mostra o motivo sob o campo (um
  padrão que não compila, um padrão de caminho inválido, um tamanho abaixo de um
  byte). Os tamanhos aceitam unidades (`500 KiB`, `5 MiB`, `1 GiB`). Um padrão de
  caminho sem barra corresponde a qualquer profundidade. Os *Valores secretos*
  escrevem-se num campo mascarado, só servem para esta execução e nunca são
  guardados; a pré-visualização oculta as linhas que os continham. *Commits
  vazios* indica se um commit que as regras deixaram vazio é removido (por
  predefinição) ou mantido. Os ficheiros binários e os ficheiros acima do limite
  de tamanho ficam como estão e são contados no Resumo.
- **Mover caminhos.** *Mover ou renomear caminhos* (*Reescrever: mover ou renomear caminhos...* na
  paleta) tem três tipos, escolhidos na lista **Mover**:
  - *Renomear um caminho* (**De**, **Para**; `--move-path FROM=TO`): os
    mesmos ficheiros e pastas aparecem no novo caminho em todos os commits que tinham o
    antigo; mais nada muda, cada ficheiro mantém o seu id. Um commit em que o
    novo caminho já existe nunca é sobrescrito: a pré-visualização é recusada e
    lista os commits e os caminhos que colidem (escolha um nome livre, ou mova primeiro o
    existente). Só caminhos exatos, sem carateres universais.
  - *Mover tudo para uma pasta* (**Pasta**; `--into-subdir DIR`):
    todos os caminhos de todos os commits ganham o prefixo. Útil para juntar repositórios
    sem colisões de nomes.
  - *Extrair uma pasta como raiz* (**Pasta**; `--extract DIR`): a pasta passa a ser
    a raiz, os commits que não lhe tocam são descartados (um commit que a cria
    fica sem pai), os merges que já não juntam nada são simplificados, e
    as branches e tags que não têm nada da pasta são eliminadas. É-lhe perguntado
    **de cada vez** como termina:
    - *Num novo clone, aberto num novo separador* (`--extract-mode clone`): é feita uma cópia
      deste repositório numa pasta que escolher (o novo repositório
      fica com uma pasta com o nome da extraída lá dentro), reescrita
      aí e limpa para conter só o novo histórico, com todas as branches
      e tags locais que têm a pasta. Este repositório não é tocado. O
      novo repositório não tem remoto (o original não é o seu upstream).
    - *No local: o histórico deste repositório é reescrito* (`--extract-mode
      in-place`): a reescrita habitual, pré-visualização, Aplicar, desfazer. As branches sem
      a pasta são eliminadas (o desfazer traz-as de volta); a branch em checkout
      tem de a ter.
    - *Como submódulo deste repositório* (`--extract-mode submodule`): o novo
      repositório é feito como no caso do clone e depois **um novo commit** na
      branch atual substitui a pasta por um submódulo dele (`.gitmodules`
      e a ligação); a pasta fica como um submódulo em checkout. O histórico
      deste repositório **não** é reescrito: os seus commits antigos mantêm os
      ficheiros da pasta, por isso nada que aponte para eles (tags, outros clones)
      se quebra. O URL em `.gitmodules` é o caminho local do novo
      repositório: mude-o (`git config -f .gitmodules submodule.<pasta>.url`)
      quando ele passar a viver num alojamento. Ficheiros editados ou não acompanhados na pasta
      recusam-no. O desfazer repõe a pasta simples.
    O Resumo diz que a pré-visualização mostra o histórico do novo repositório e
    para que pasta vai. As regras que vêm depois de uma extração (um renomear, uma
    remoção) aplicam-se à árvore extraída; num clone ou num submódulo
    a extração tem de ser a primeira regra de caminhos. As regras de caminhos correm depois das regras
    de conteúdo, por isso um glob de remoção indica os caminhos como eram.

#### Fins de linha, bit de execução e codificações de ficheiros

Três regras do grupo **Conteúdo** do menu Adicionar (ou as linhas da paleta
*Reescrever: converter fins de linha...*, *Remover o bit de execução...* e
*Reescrever: converter a codificação dos ficheiros...*). As três aceitam **Ficheiros** (globs; um padrão
sem barra corresponde a qualquer profundidade; vazio quer dizer todos os ficheiros) e **Exceto os
ficheiros**. Só alteram o que os globs indicam, em todos os commits do âmbito.
Os ficheiros binários (um byte NUL nos primeiros 8 KiB) nunca são tocados: o Resumo
conta-os como *ficam como estão*.

- **Converter fins de linha** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Converter para*: LF (Linux, macOS), CRLF (Windows) ou
  CR (Mac clássico). Todos os fins de linha de um ficheiro de texto correspondente passam a ser esse,
  por isso um ficheiro com fins de linha misturados sai uniforme; os ficheiros que já estão certos mantêm
  a sua versão intacta. *Seguir o .gitattributes* (desligado por predefinição,
  `--eol-follow-attributes`) lê o `.gitattributes` da raiz de cada commit: os ficheiros
  marcados `-text` ou `binary` são ignorados e os marcados `eol=lf` ou
  `eol=crlf` mantêm esse fim de linha em vez do que escolheu.
- **Remover o bit de execução** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Os ficheiros correspondentes passam do modo 100755 para 100644; o conteúdo
  não é tocado. É obrigatório indicar pelo menos um padrão de ficheiros: retirar o bit
  a todos os executáveis estragaria os seus scripts.
- **Converter a codificação dos ficheiros** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Converter para*: UTF-8 (a predefinição), UTF-8 com BOM, UTF-16 LE ou
  BE (escrito com uma marca de ordem de bytes), Windows-1252 ou ISO-8859-1. *Os ficheiros estão
  em*: por predefinição deteta-se a codificação de cada ficheiro (uma marca de ordem de bytes, senão
  UTF-8 válido, senão a *Alternativa*, Windows-1252 a menos que a mude); indique
  a codificação para a impor, mas uma escolha errada dá texto errado (UTF-8 lido como
  Latin-1 duplica todos os acentos). Um ficheiro que a codificação de origem não consiga ler, ou
  que tenha um caráter que o destino não consiga guardar (um sinal de euro em ISO-8859-1),
  fica exatamente como está e é contado no Resumo.

As regras correm pela ordem da lista, sobre o mesmo ficheiro: converter um ficheiro para
UTF-16 e pedir depois fins de linha deixa-o em paz (já não é texto
simples). Aplicar é uma entrada na lista de desfazer, como qualquer reescrita.

### Encontrar segredos

- **Encontrar segredos:** `gitpoulpi rewrite scan-secrets` (e a ferramenta MCP
  `rewrite_scan_secrets`) lê o histórico uma vez e lista prefixos de tokens
  (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), chaves
  privadas, JWTs, palavras-passe em URLs e linhas `password = valor` de aspeto
  aleatório, cada uma com o ficheiro, o primeiro e o último commit e uma
  pré-visualização ocultada como `ghp_****abcd`. Não é exaustiva, e o valor
  nunca é mostrado, registado nem guardado. `--remove-finding ID` transforma um
  resultado numa regra que o remove de todos os commits. Um segredo que foi
  enviado está comprometido: renove-o primeiro, depois reescreva, faça um push
  forçado e purgue.
- **O separador Segredos** faz o mesmo a partir do painel, para o âmbito da
  barra de âmbito: **Procurar segredos** (`s`) executa a pesquisa como uma
  tarefa (a barra de progresso chama-se *A procurar*, **Cancelar** ou `Esc`
  interrompe-a). Cada resultado é uma linha com o tipo, a pré-visualização
  ocultada, o ficheiro e quantos outros o contêm; sob a lista: o primeiro e o
  último commit em que foi visto e quantos commits e versões de ficheiros o
  contêm. O separador diz quantos ficheiros binários e demasiado grandes não
  foram lidos, e que a pesquisa não é exaustiva. Todos os resultados começam
  marcados; **Remover do histórico** (`r`) transforma os marcados numa só regra
  que os nomeia por id (nunca pelo valor), que pré-visualiza e aplica como
  qualquer outra. Os resultados ficam desatualizados após uma reescrita ou uma
  mudança de âmbito: o separador di-lo, pesquise de novo. O painel nunca guarda
  o valor de um segredo, por isso não o pode mostrar.

### Descobrir o que pesa

- **Descobrir o que pesa:** `gitpoulpi rewrite analyze [--top N]` (e a
  ferramenta MCP `rewrite_analyze`) é a análise de tamanhos, como
  `filter-repo --analyze`: os maiores ficheiros com o caminho, o tamanho, o
  primeiro e o último commit e se ainda estão em HEAD, as maiores pastas, os
  ficheiros eliminados de HEAD que o histórico ainda guarda e os totais. Lê
  apenas tamanhos e caminhos, nunca o conteúdo dos ficheiros, e não altera
  nada.
- **O separador Tamanho** faz o mesmo a partir do painel (`F7`, ou *Analisar os
  tamanhos do histórico* na paleta, abre-o e executa): **Analisar tamanhos**
  (`s`) é uma tarefa (*A analisar*, **Cancelar** ou `Esc` interrompe-a).
  **Mostrar** escolhe a lista (`v`): *Ficheiros* (as maiores versões únicas de
  ficheiros, cada uma marcada *em HEAD*, *versão antiga* de um ficheiro que
  ainda lá está, ou *eliminado*), *Pastas* (tudo o que alguma vez foi guardado
  por baixo de cada uma) ou *Eliminados* (caminhos que já não estão em HEAD e
  que o histórico guarda). **Ordenar** (`o`) é por tamanho, por caminho ou
  eliminados primeiro. Marque linhas (`Espaço`, `Alt+A` / `Alt+U` para todas /
  nenhuma): o separador diz quanto ocupam, com honestidade como "até cerca de",
  porque o espaço só volta depois da reescrita, de uma purga dos objetos
  antigos e de um `gc`. **Remover do histórico** (`r`) transforma os caminhos
  marcados numa regra de caminhos (caracteres especiais escapados, um caminho
  que já está numa regra não é acrescentado duas vezes); **Remover todos os
  ficheiros acima de ...** (`b`, lista Ficheiros) escreve uma regra de tamanho
  logo abaixo do menor ficheiro marcado. Pré-visualize-as e aplique-as como
  qualquer outra regra. As listas mostram as 200 maiores linhas; a CLI diz
  quantas mais há e `--top` lista-as. As linhas ficam desatualizadas após uma
  reescrita ou uma mudança de âmbito: analise de novo.

### Assinaturas

- **Assinaturas:** um commit reescrito não pode manter a sua assinatura (ela assinava
  os bytes antigos), por isso, quando commits assinados são reescritos, é-lhe perguntado, de cada vez:
  *Remover as assinaturas* (a contagem é mostrada antes), *Voltar a assinar só os commits
  que estavam assinados*, ou *Voltar a assinar todos os commits reescritos*. Não há
  predefinição. Os commits que ficam como estão mantêm a sua. Voltar a assinar usa a sua própria configuração,
  tal como um commit assinado: `gpg.format` (`openpgp`, `ssh` ou `x509`),
  `user.signingkey` e `gpg.program` / `gpg.ssh.program` / `gpg.x509.program`.
  A quarta linha da pergunta, **Verificar a assinatura** (`4`), assina um conteúdo
  fictício uma vez e verifica-o: diz quanto demora uma assinatura e
  estima a execução para o número de commits em causa, ou porque não
  funciona (chave em falta, agente bloqueado, programa não instalado, com a forma de o
  instalar). A pré-visualização faz o mesmo teste antes de ler o histórico e recusa
  com esse motivo, por isso uma configuração avariada nunca deixa uma reescrita meio assinada. Os
  commits são assinados **durante a pré-visualização** (os seus ids contêm a assinatura),
  por isso uma pré-visualização que volta a assinar demora o tempo das assinaturas, o Resumo diz *Commits
  assinados de novo: n, em 12.5 s*, e pode abrir-se então um pedido de frase-passe: desbloqueie
  primeiro o seu agente (Verificar a assinatura faz exatamente isso). **Cancelar** pára entre
  duas assinaturas e não deixa nada para trás. Só as assinaturas de commits são
  recriadas; uma tag assinada continua a perder a assinatura.
- **A regra *Remover assinaturas*** (menu Adicionar, grupo *Assinaturas*; *Reescrever:
  remover assinaturas...* na paleta; `--remove-signatures`) retira a
  assinatura de todos os commits do âmbito, mesmo onde mais nada muda, e
  não pergunta nada sobre voltar a assinar. Na CLI: `rewrite check-signing` e
  `--signatures strip|resign-signed|resign-all`.

### Desfazer uma reescrita

**Desfazer** (o botão do painel do resultado, o *Desfazer* da barra de ferramentas com `Ctrl+Z` no Log, ou `gitpoulpi undo`) repõe todas as
branches, tags e notas movidas, e os ficheiros em checkout com elas, a partir dos
commits antigos que a reescrita fixa. Pergunta primeiro quando uma branch reescrita já
está no seu upstream (desfazer localmente não desfaz um push) e quando tem
alterações por commitar: essas são mantidas, e um ficheiro que alterou e que o histórico
antigo sobrescreveria faz parar o desfazer (os ficheiros movem-se como
os move o `reset --keep`). O desfazer deixa de ser
possível para uma reescrita depois de [purgar](#eliminar-o-histórico-antigo-purga) o seu histórico
antigo; a lista do histórico passa a marcá-la como *purgada*.

## Mover ficheiros para dentro ou para fora do Git LFS

Duas regras do grupo *LFS* do menu Adicionar reescrevem o histórico para que os ficheiros grandes
vivam no [Git LFS](lfs.md) desde o início (ou saiam dele). Precisam do
programa `git-lfs`: sem ele as regras ficam esbatidas com a sugestão de instalação e uma
pré-visualização é recusada. São o nosso próprio motor, não o `git lfs migrate`, por isso
tem a pré-visualização, o âmbito, o desfazer e a proteção de push de qualquer outra regra.

- **Migrar ficheiros para o LFS** (*Reescrever: migrar ficheiros para o LFS...*, ou
  `--lfs-import PATTERN` com `--lfs-min-size BYTES`). Todos os ficheiros que correspondam
  aos padrões (`*.psd`, `assets/**`; um padrão sem barra corresponde a qualquer
  profundidade) e tenham pelo menos o tamanho de *Só ficheiros com pelo menos* (vazio: todos os
  que correspondam) passam a ser um ponteiro LFS em todos os commits que os contêm. O seu conteúdo vai para
  o armazenamento LFS local (`.git/lfs/objects`, uma vez por conteúdo distinto) quando
  Aplica, e o `.gitattributes` da raiz ganha `<pattern> filter=lfs diff=lfs
  merge=lfs -text` em todos os commits reescritos que contêm um ponteiro (criado quando
  não existe, as outras linhas intactas, nunca duplicado). Um ficheiro que
  já é um ponteiro fica em paz. O Resumo diz *3 ficheiros (12 MiB) passam para o
  LFS. Objetos LFS a escrever: 3.* e o separador Commits lista *passa big.psd para o
  LFS* por commit.
- **Trazer ficheiros de volta do LFS** (*Reescrever: trazer ficheiros de volta do LFS...*, ou
  `--lfs-export PATTERN`). Os ponteiros que correspondam voltam a ser os ficheiros reais, e
  as linhas do `.gitattributes` desses padrões são removidas. Os objetos têm de estar
  no armazenamento LFS local: quando alguns não estão, a pré-visualização é recusada com a
  lista e um botão **Transferir objetos LFS** que os obtém de `origin`;
  pré-visualize outra vez depois. Uma importação seguida de uma exportação dos mesmos
  padrões devolve exatamente os mesmos commits.

Depois de Aplicar, os ficheiros em checkout são o conteúdo real (não ponteiros) e
**Desfazer** repõe o histórico antigo. O painel do resultado tem **Enviar objetos LFS**
(`l`): o mesmo que `LFS: Push All Objects`, para que o novo histórico possa ser enviado
sem que um push posterior fique parado por falta de objetos (um push simples das branches
reescritas também os envia, através do hook de pre-push que o Aplicar instala). A nota
para os colaboradores diz que precisam do git-lfs e de `git lfs pull`.

Bom saber: Aplicar instala os filtros LFS do repositório (`git lfs install
--local`) quando faltam, porque o git precisa deles para comparar os ficheiros reais
com os ponteiros. Só o `.gitattributes` da raiz é escrito, nunca um aninhado. Um ficheiro
*abaixo* do tamanho mínimo que corresponda a um padrão continua a ser um ficheiro simples,
mas o git vai querer transformá-lo num ponteiro no próximo add, por isso o `git
status` mostra-o como modificado até o commitar. A importação lê os ficheiros grandes
em fluxo, a exportação mantém um ficheiro de cada vez em memória.

## Enviar o resultado

Aplicar uma reescrita nunca faz push. As branches e tags reescritas que estavam
num remoto precisam de um push forçado, e o push nunca força às cegas: cada ref
é enviada com um lease sobre o id que o remoto tinha quando pré-visualizou (não
o que a sua branch de acompanhamento diz agora), pelo que o push de um colega
entretanto é rejeitado para essa ref em vez de ser sobrescrito, e as refs vão
num único push do tipo tudo ou nada (`--atomic`) quando o servidor o permite.
Na CLI:

```
gitpoulpi rewrite push --dry-run        # pergunta ao remoto, não envia nada
gitpoulpi rewrite push --yes            # envia
gitpoulpi rewrite note                  # o que os outros têm de executar
gitpoulpi rewrite maps --out ./maps     # commit-map e ref-map
```

As branches protegidas (definição `rewrite.protected_branches`, por predefinição
`main`, `master`, `develop`, `release/*`) ficam de fora no push, na reescrita e
outra vez no push, a menos que as nomeie (`--include-protected main`) ou inclua
todas. A proteção de branches do próprio alojamento continua a decidir se aceita
um push forçado; a recusa é mostrada por ref.

No painel de reescrita, o ecrã do resultado depois de Aplicar faz tudo isto:

- **Enviar as refs reescritas para o remoto** lista as refs que a reescrita
  moveu e que um remoto tem, cada uma com uma caixa (`Espaço`). As branches
  protegidas começam desmarcadas, com *protegida* ao lado: marque-as uma a uma,
  ou use *Marcar todas* / *Desmarcar todas* (`Alt+A` / `Alt+U`). **Push das
  branches reescritas** (`p`) envia as marcadas; a resposta é uma linha por ref
  (*enviada*, *já está atualizada*, *rejeitada: o remoto mudou desde a
  pré-visualização*, *rejeitada: protegida pelo alojamento*, *ignorada: branch
  protegida*, *não enviada*). Uma linha rejeitada tem **Fetch e rever** (`f`): o
  painel fecha e os remotos são atualizados com fetch, para o Log mostrar o que
  os outros enviaram. Nada é sobrescrito: o lease protege o trabalho deles, e
  pré-visualiza de novo antes de tentar outra vez.
- **Copiar a nota** (`c`) põe na área de transferência a nota para os
  colaboradores, no idioma da interface em torno dos comandos git, que não são
  traduzidos.
- **Abrir o mapa de commits** (`m`) mostra onde estão o mapa de commits e o
  mapa de refs; um clique num caminho (ou `1` / `2`) copia-o.

Um push não se desfaz: se fizer `undo` da reescrita depois, só o seu
repositório muda, e o desfazer di-lo antes de perguntar. Envie a nota a todos
os que têm o histórico antigo: lista a ponta antiga e a nova de cada branch e
tag e os comandos (`git fetch`, depois `git reset --hard origin/<branch>` ou
`git rebase --onto`). Os dois mapas são guardados durante 24 horas com a
execução; copie-os para fora se a limpeza de repositório do GitLab ou um script
precisar deles.

## Quando o alojamento protege uma branch

O seu alojamento pode proteger uma branch por conta própria (a proteção de branches e os
rulesets do GitHub, as branches protegidas do GitLab, as políticas de branch do Azure DevOps, as
restrições de branch do Bitbucket) e recusar o push forçado de que uma reescrita precisa. Quando o
repositório está **ligado** ao seu alojamento (a secção de pedidos da barra lateral,
ver [Integração com alojamentos](hosting-integration.md)), o painel pergunta ao
alojamento que branches protege, com o seu próprio token:

- Essas branches ficam **de fora da reescrita por omissão**, tal como as branches protegidas
  de `rewrite.protected_branches` e do fluxo de trabalho: o separador
  Branches e tags lista-as, cada uma com uma caixa, e os botões *Marcar todas* / *Desmarcar todas*, e diz
  porquê: *protegida em GitHub*.
- Se o token não conseguir ler a proteção (falta-lhe a permissão, ou não é
  administrador), o painel diz *Proteção do alojamento desconhecida* e não trata
  nada como protegido só pela palavra do alojamento. A reescrita nunca é bloqueada por
  isso. Um repositório que não está ligado nunca é consultado.

Quando uma branch protegida está no caminho (a pré-visualização foi recusada porque todas as
branches estão protegidas, ou o alojamento rejeitou o push: *rejeitada: protegida pelo
alojamento*), o botão **Como contornar** (`Alt+H`) explica os três caminhos
possíveis, com a página de definições exata do seu fornecedor e repositório (escolha uma
ligação e copie-a com `c`, ou abra-a com `o`). O GitPoulpi nunca faz nenhum deles
por si:

1. **Levantar temporariamente a proteção** nas definições do alojamento, fazer o push e depois
   voltar a pô-la.
2. **Fazer um push da reescrita com um novo nome de branch** e depois tornar essa branch a
   predefinida (ou renomear) e eliminar a antiga quando já ninguém a usar.
3. **Como último recurso, eliminar e recriar o repositório remoto** e enviar
   tudo. Isto perde os pull / merge requests e as respetivas revisões, as
   issues, a ligação dos forks ao histórico antigo, as estrelas e os observadores, as
   definições, segredos e variáveis de CI, e os webhooks e as chaves de deploy.

Na CLI, o mesmo texto volta como `protected_by_host` no JSON de
`rewrite preview` / `rewrite run` (quando o alojamento protege uma branch do
âmbito), de um `rewrite push` recusado e de `rewrite note`: `provider`,
`branches`, `ways` (cada um com os seus `steps` e `links`), `recreate_loses` e a
explicação completa como `text`.

## Eliminar o histórico antigo (purga)

Depois de uma reescrita, os commits antigos ainda estão no seu repositório: o
desfazer da reescrita guarda-os (é isso que o torna desfazível), tal como os
reflogs. Se reescreveu para se livrar de um segredo, esse segredo continua na
pasta `.git`. A purga elimina o histórico antigo de vez:

```
gitpoulpi rewrite purge                 # pergunta: lista o que é eliminado e perdido (código de saída 3)
gitpoulpi rewrite purge --yes           # fá-lo
```

Descarta os pins de desfazer da reescrita, faz expirar as entradas do reflog que
nomeiam os commits antigos, executa `git gc --prune=now` e depois verifica se
cada commit e versão de ficheiro antigos desapareceu mesmo. A confirmação diz
antes o que perde: **o desfazer deixa de ser possível** para a reescrita (e
para qualquer operação anterior cujo desfazer também precisasse dos commits
antigos); ficam na lista do histórico, marcadas como purgadas. Uma purga não
pode ser interrompida depois de começar nem ser desfeita. Não toca em nenhum
remoto.

Tudo o que ainda retenha o histórico antigo bloqueia a purga, e então nada é
alterado. A lista nomeia cada um com o que fazer: uma branch ou tag que não
fazia parte da reescrita, um stash sobre o histórico antigo (descarte-o, ou
mantenha-o e aceite que o histórico antigo fica), outro worktree cujo HEAD está
nele, e as branches de acompanhamento remoto que ainda apontam para o histórico
antigo (faça push das branches reescritas e depois fetch). A sua lista de
stashes e os reflogs das outras branches nunca são tocados.

Um segredo que foi enviado está comprometido: **renove-o primeiro**. Depois da
purga, a resposta lista o que o alojamento ainda precisa, por remoto: o GitHub
mantém commits antigos acessíveis em vistas em cache e nas refs de pull
requests (contacte o Suporte do GitHub, ou elimine e recrie o repositório), o
GitLab tem uma limpeza de repositório que usa o `commit-map`
(`gitpoulpi rewrite maps --out DIR`) mais a manutenção, o Azure DevOps e o
Bitbucket precisam de um pedido de suporte ou de um novo repositório, e todos
os que têm um clone ou um fork continuam com o histórico antigo.

### O ecrã da purga

No painel de reescrita, o ecrã do resultado de uma reescrita tem um botão **Purgar
histórico antigo...** (tecla `e`), e a paleta tem **Reescrever: purgar o
histórico antigo...** (atua sobre a reescrita mais recente que ainda não foi
purgada, também depois de fechar o painel). O ecrã lista o que a purga elimina
e o que perde (as operações cujo desfazer vai com ela; as marcadas *perde também
o Desfazer* não são reescritas mas precisavam dos commits antigos) e depois o
que ainda retém o histórico antigo, uma linha cada:

- uma branch de acompanhamento remoto: **Fetch** atualiza-as a partir dos
  remotos;
- um stash: **Descartar** pergunta primeiro, porque um stash é trabalho que
  nunca fez commit;
- uma branch, tag, worktree ou outra ref: **Copiar nome** e depois trate disso
  onde vive (reescreva ou elimine a branch, faça checkout de uma branch
  reescrita no worktree).

A verificação corre outra vez após cada correção, e **Verificar de novo** (`r`)
executa-a a pedido. **Purgar** (`p`) só está disponível quando nada sobra. Faz
uma última pergunta, numerada e sem predefinição (`1` purga de vez, `2` deixa
tudo como estava), e depois executa; o ecrã mostra o passo em que está e diz
que não pode ser interrompida. O resultado diz que o histórico antigo
desapareceu deste clone, ou quantos objetos antigos ainda lá estão. Por baixo,
os passos a dar em cada remoto (uma ligação pode ser copiada). A cruz, `Esc` e
um clique fora fecham o ecrã, exceto enquanto a purga corre.

Por MCP, a ferramenta de purga não existe a menos que defina
`allow_history_purge = true` na tabela `[mcp]` do `config.toml`.

## A regra Script

*Adicionar regra > Script* (ou **Reescrever: executar um script...** na paleta)
acrescenta uma regra de script Lua e abre o respetivo editor sobre o painel;
`Enter` na linha *Script* da regra volta a abri-lo. O editor parte de um
pequeno modelo.

- **Linguagem** (uma linha do painel da regra): *Lua (integrado)*, *Script de
  shell*, *Python* ou *Lua (instalado)*. Tudo o que não seja o Lua integrado é
  um [script externo](#scripts-externos-shell-python-lua): sem sandbox, e uma
  linguagem desativada nas definições, ou cujo interpretador não existe, fica
  esbatida na lista com o motivo ao lado (escolhê-la diz o que fazer). Mudar a
  linguagem de um script que ainda não escreveu troca o modelo.
- **Script** é um editor de texto com números de linha. O que escreve é logo a
  regra; não há nada a confirmar.
- Por baixo, a linha de **verificação** diz `OK: função rewrite_commit
  encontrada`, ou lista os problemas com a respetiva linha (`script:3: ...`; o
  número fica a vermelho na margem). Corre um instante depois de parar de
  escrever, nunca enquanto uma tecla está premida.
- **Parâmetros** tem uma linha por cada linha `--! param` do cabeçalho do
  script, com a etiqueta ao lado do campo: texto, um número ou uma caixa de
  verdadeiro/falso. Um valor que apague volta à predefinição do cabeçalho; um
  parâmetro de texto sem predefinição tem de ser preenchido antes de
  Pré-visualizar.
- **Ficheiro** (um caminho dentro do repositório, `.gitpoulpi/rewrite/script.lua`
  para começar) com **Carregar de ficheiro...** (substitui o texto) e **Guardar
  em ficheiro...**.
- **Testar na seleção** executa o script nos commits que selecionou no Log, ou
  nos 20 mais recentes quando nenhum está selecionado, e não escreve nada. A
  tabela lista cada commit (id curto), cada campo que alteraria como
  `antigo -> novo` e as linhas `log(...)` do script. **Cancelar** deixa de
  esperar. Se o script falhar, são mostrados o commit e o motivo.
- A cruz, um clique fora do editor e `Esc` fecham-no; `Tab` e `Shift+Tab`
  movem-se entre o editor, os parâmetros, o ficheiro, os botões e os
  resultados; `F5` testa.

A pré-visualização recusa um script que não compila: o painel da
pré-visualização diz *um script falhou* com a linha (ou, numa falha num commit,
o respetivo id curto e o motivo). Numa compilação sem scripts, o tipo continua
no menu Adicionar, esbatido.

## Escrever um script com IA

Com a IA ativada ([Assistência de IA](ai.md)), o editor de scripts tem um botão
`✦ Escrever com IA...`, e a paleta tem `✦ Reescrever: escrever um script com IA...`,
que abre o painel numa nova regra Script com o diálogo aberto. Nada é enviado
antes de premir **Enviar**: o diálogo mostra, à medida que escreve, o texto
exato que vai sair.

- **O que deve fazer o script?** As suas próprias palavras: *mover para a manhã
  seguinte todos os commits feitos de noite*, *substituir o meu e-mail antigo
  pelo novo, só antes de 2024*.
- **Enviar uma amostra de N commits** (desativado): quando marcado, o autor, a
  data e o assunto dos commits que "Testar na seleção" usaria (a sua seleção no
  Log, ou os 20 mais recentes) são acrescentados para o modelo ver o aspeto dos
  seus dados. Nunca o conteúdo dos ficheiros nem os corpos dos commits. Com a
  caixa desmarcada, nenhum dado de commits sai da máquina.
- **O que é enviado** lista as instruções (o contrato fixo do script e depois as
  suas instruções de IA), o seu pedido e qual o perfil que os recebe. `Tab` leva
  até lá, as setas deslocam.
- **Enviar** (`Tab` até lá, `Enter`). Enquanto o modelo trabalha, o diálogo
  espera, e `Esc` cancela o pedido. Se falhar, o diálogo fica com o motivo para
  poder tentar de novo.

A resposta é verificada pela mesma verificação do daemon que um script escrito
por si; um erro de compilação é devolvido ao modelo uma vez. O script
**substitui então o texto do editor** (o diálogo di-lo quando o editor tem
algo diferente do modelo inicial) e o título do editor diz *✦ escrito por IA:
leia-o antes de Pré-visualizar* até alterar o texto. Nunca é testado,
pré-visualizado nem aplicado por si: leia-o, prima **Testar na seleção** e
depois Pré-visualize como com qualquer script. O script corre na mesma sandbox
que um escrito por si (sem ficheiros, sem relógio, sem rede).

Na linha de comandos: `gitpoulpi ai rewrite-script "<instrução>" [--sample N]`
mostra o script e o resultado de `rewrite script check` sobre ele, em JSON, e
não executa nada (código de saída 2 se ainda não compilar). É usado o perfil
*Script de reescrita do histórico* de Definições > IA.

## Criar regras com IA

Com a IA ativada ([Assistência de IA](ai.md)), o painel tem um botão `✦ Criar regras com IA...`
ao lado de Guardar e Carregar, a tecla `Alt+I` (*Criar regras de reescrita com IA* em
Definições > Teclado, contexto *Reescrever*), e a paleta tem `✦ Reescrever: criar regras com IA...`.
Com a IA desativada, ou numa compilação sem IA, o botão e a linha da paleta não aparecem.
Descreve por palavras suas como o histórico deve ser reescrito (*substituir o meu e-mail antigo
a@b.org por c@d.org*, *remover secrets.txt de todo o histórico*, *deslocar as minhas datas duas
horas*) e o modelo propõe as regras e as definições, que **preenchem o painel**. Nada é enviado
antes de premir **Enviar**: o diálogo mostra, à medida que escreve, o texto exato que vai sair.

- **Como deve ser reescrito o histórico?** Um campo de várias linhas: `Enter` muda de linha,
  `Ctrl+Enter` ou `Ctrl+S` envia, `Esc` e a cruz cancelam.
- **Enviar o âmbito, os nomes das branches e das tags e os autores do histórico** (desmarcada):
  quando marcada, o âmbito da barra de âmbito, os nomes das branches e das tags e os autores
  distintos (nome e e-mail) dos commits mais recentes que o Log mostra (no máximo 20 commits) são
  acrescentados, para o modelo poder escrever o seu e-mail exatamente (o `ai rewrite-rules --context`
  da linha de comandos lê antes os 200 commits mais recentes). Nunca o conteúdo dos ficheiros nem
  as mensagens dos commits. Com a caixa desmarcada, só a sua descrição e as
  instruções fixas saem da máquina.
- **Substituir as regras atuais** (só quando o painel já tem regras; desmarcada): por
  predefinição, as novas regras são adicionadas depois das que já tem; marcada, substituem-nas
  (o âmbito e as definições indicados na resposta são aplicados em ambos os casos).
- **O que é enviado** lista as instruções (o vocabulário das regras e depois as suas instruções
  de IA), o seu pedido e o perfil que os recebe. `Tab` leva até lá, as setas deslocam.

O modelo pode usar todos os tipos de regra desta página exceto **Script** (código: *Escrever um
script com IA* mostra o script para revisão) e a forma de valores secretos de *Procurar e
substituir nos ficheiros* (os valores secretos nunca são entregues a um modelo: use o separador
Segredos). Pode definir o âmbito como a branch atual, todas as branches e tags locais, ou
branches e tags existentes indicadas pelo nome, e as opções *Tags*, *committer*, *commits
vazios* e *ids abreviados nas mensagens*. O que o painel não consegue exprimir (um limite de
data como *desde março*, um intervalo de commits) não é adivinhado: o modelo di-lo numa
observação. O que acontece às assinaturas nunca é decidido pelo modelo: a pergunta é feita
ao Aplicar.

**A IA só prepara regras e definições.** Nunca pré-visualiza, aplica nem executa nada: a
resposta é copiada para o painel como uma receita que tivesse carregado, a pré-visualização no
ecrã fica desatualizada, e Pré-visualizar e Aplicar (com as suas perguntas e confirmações) são
seus, como sempre. Cada regra é analisada e verificada à parte pela mesma verificação que o
editor e o motor usam; uma regra que falha fica de fora. Quando algo ficou de fora, ou o modelo
fez uma observação, uma lista mostra-o depois de as regras entrarem (*Deixadas de fora (as
regras válidas foram mantidas)*, cada uma com a posição, o nome e o motivo) e `Enter` ou `Esc`
fecha-a. Leia as regras, edite-as e depois Pré-visualize.

Na linha de comandos: `gitpoulpi ai rewrite-rules "<descrição>" [--context]` mostra a proposta
em JSON (`recipe`, que `rewrite preview --recipe-json` lê, `problems`, `notes` e
`applied: false`) e não executa nada (código de saída 2 quando algo ficou de fora). A ferramenta
MCP é `ai_rewrite_rules`. É usado o perfil *Regras de reescrita do histórico* de Definições > IA.

## Scripts Lua

Uma regra de script executa a sua função em cada commit. É Lua 5.4 puro numa
sandbox: sem ficheiros, sem relógio, sem processos, sem módulos, sem `pcall`; o
mesmo script sobre o mesmo histórico dá sempre o mesmo resultado, e uma chamada
que passe de 10 milhões de instruções, 64 MiB ou 5 segundos pára com um erro
que indica o commit.

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
`params`, `state` (devolva `state = {...}` para passar dados ao commit
seguinte) e `window.before` / `window.after` (defina `--! window: N`). Devolva
apenas o que muda: `author`, `committer`, `message`. Funções auxiliares:
`regex_replace`, `regex_match`, `stable_hash`, `rand`, `weekday`, `hour`, `log` e, para mensagens, `trailers(message)`
(uma lista de `{key, value}`), `set_trailer(message, key, value)` (substitui os
trailers dessa chave; um quarto argumento `"add"` ou `"skip"` altera isso),
`remove_trailer(message, key)` e `at("2024-03-05 18:30", offset_minutes)`
(uma data com um desfasamento, como tempo Unix).

Na linha de comandos: `gitpoulpi rewrite script check my.lua` compila-o,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b`
mostra o que alteraria sem escrever nada, e
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b`
planeia a reescrita completa.

## Scripts externos (shell, Python, Lua)

O mesmo contrato de um script Lua, executado por um programa da sua máquina:
*Script de shell* (bash ou sh; PowerShell no Windows, ou qualquer `.ps1`),
*Python* e *Lua (instalado)* (um `lua` seu, não o integrado; não tem biblioteca
JSON, por isso o modelo lê a linha com um padrão). **Não correm em sandbox: um
script externo executa-se com as suas permissões** e pode ler ou alterar tudo o
que a sua conta pode. O painel da regra e o título do editor di-lo.

Como comunica: o GitPoulpi inicia o script uma vez por execução e envia-lhe um
pedido JSON por linha na entrada padrão (os campos de um commit, os `params`, o
`state` devolvido pela última resposta); ele responde com uma linha JSON na
saída padrão, com os mesmos campos que um script Lua devolve (`author`,
`committer`, `message`, `state`, `log`), ou `{"error": "texto"}` para desistir.
Escreva os diagnósticos no erro padrão: aparecem como as linhas `log` do commit.
As linhas de cabeçalho começam por `#!` em shell e Python (`#! name: ...`,
`#! param old: text "Old"`; uma linha shebang continua a ser um shebang) e por
`--!` em Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**O que recebe.** Um ambiente vazio, exceto `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` e `LANG`; uma pasta temporária nova como pasta de
trabalho (com uma cópia do próprio script); não o caminho do repositório.
**Limites de tempo:** `rewrite.script_timeout_ms` (5000) para cada resposta e
`rewrite.script_total_timeout_s` (3600) para a passagem completa. Quando um é
excedido, o script falha, escreve algo que não é JSON ou a tarefa é cancelada,
o GitPoulpi mata o script e tudo o que ele iniciou, e a reescrita pára com o
commit e o motivo; nada se moveu.

**Que linguagens existem.** `rewrite.external_scripts` no `config.toml` lista-as:
`["shell"]` por predefinição; acrescente `"python"` e `"lua"` você mesmo no
`config.toml` (`[rewrite]`, `external_scripts = ["shell", "python"]`; uma lista
não se pode definir com `gitpoulpi config set`). Uma linguagem que não está
listada fica esbatida no painel e é recusada pela CLI e pelo MCP
(`scripts_disabled`); uma listada cujo interpretador não é encontrado fica
esbatida com a sugestão de instalação (`tool_missing`).

**Confiança.** Mesmo com uma linguagem ativada, um script só corre depois de
confiar no seu texto exato. Um script que escreve no painel é seu e corre logo;
um que vem com uma receita partilhada ou com um ficheiro do repositório não é:
a pré-visualização e *Testar na seleção* param e perguntam *Confiar neste
script?* com duas respostas numeradas e nenhuma pré-selecionada (`1` confiar e
continuar, `2` não, `Esc` deixa-o). Confiar regista o SHA-256 do texto para este
repositório, por isso um script alterado volta a perguntar, e um repositório
clonado não pode executar código em si por conta própria. Na linha de comandos,
leia o ficheiro e depois `gitpoulpi rewrite script trust FICHEIRO` (ou
`--trust` em `preview`, `run`, `script check` e `script test`, que confia e
continua). Um agente por MCP não pode confiar em nada: só pode executar um
script em que você já confiou.

**Verificação.** A linha de verificação sob o editor diz *Ainda não é de
confiança* para um script sem confiança (nada foi iniciado); para um de
confiança, o GitPoulpi inicia-o uma vez com um commit fictício
(`"dry_run": true`) e verifica se a resposta cumpre o contrato.
`gitpoulpi rewrite script check FICHEIRO` faz o mesmo e indica `trust`, `hash`
e `interpreter`.

## A partir da linha de comandos

`gitpoulpi rewrite` (e as ferramentas MCP com os mesmos nomes) corre o mesmo motor,
para scripts e agentes. Uma **receita** é um ficheiro TOML ou JSON, o que o
painel guarda (`--recipe FILE`, ou `-` para a entrada padrão), e as opções acrescentam regras
depois das da própria receita.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plano, não altera nada
gitpoulpi rewrite apply --run ID --digest D           # executa o plano que reviu
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # repõe tudo
```

- Regras como opções: `--set-name` / `--set-email` com `--match-email` (identidade),
  `--mailmap`, `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`,
  `--trailer-remove`, `--trailer-map`, `--fix-message-encoding`
  (`--fix-name-encoding`), `--remove-path`, `--remove-blobs-over`,
  `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`, `--reencode`
  (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`,
  `--lfs-import`, `--lfs-export`, `--script`, `--remove-signatures`,
  `--remove-finding`. O âmbito é a branch atual, `--all-local`, ou
  `--ref R` (repetível), a partir de `--since REV`.
- **As perguntas sem resposta predefinida** são opções: `--signatures
  strip|resign-signed|resign-all` quando commits assinados mudam, e
  `--extract-mode` para uma extração. Sem elas, a resposta é uma recusa ou um
  `needs_choice` (código de saída 3), nunca um palpite.
- **Códigos de saída:** 0 feito; 1 um erro (receita ilegível, opções inválidas); 2 recusado,
  com o motivo (`stale`, `signature_choice_needed`, `dirty_paths_touched`,
  `empty_scope`, ...); 3 uma pergunta a responder, ou refs enviadas que precisam de `--yes`.
- `preview` guarda o plano no repositório, por isso `apply` funciona a partir de um comando
  posterior, depois de reiniciar o daemon ou a partir de outra interface; um plano que já não
  corresponde ao seu digest é `stale`.

A [referência da CLI](cli-reference.md) tem todas as opções, e
[MCP e agentes](mcp-and-agents.md) diz que ferramentas um agente recebe (a ferramenta de
purga não existe a menos que a ative, e um agente não pode confiar num script).

## Propostas de agentes de IA

Um agente pode **sugerir** uma reescrita, nunca executar uma. `gitpoulpi ui propose rewrite
--recipe FILE|-` (a ferramenta MCP `ui_propose_rewrite`, desativada até `[mcp]
allow_ui_propose = true`) abre o painel na sua janela em execução com as regras da
receita já na lista e uma faixa roxa, *✦ Proposto por
<agente>: uma reescrita do histórico (3 regras) · Ctrl+Enter aceita · Esc rejeita*.

- **Nada é executado.** Nem pré-visualização, nem aplicação, nem push, nem purga: a proposta só
  preenche as regras. Você lê-as, edita-as se quiser, **Pré-visualiza** e aplica
  com a sua própria tecla. Fechar o painel rejeita a proposta.
- **O que o agente não pode pôr nela:** um script (correria ao pré-visualizar),
  mais de 50 regras, valores secretos (são retirados da receita antes de
  ser enviada). Uma proposta assim é recusada com o motivo.
- O agente pode ler o desfecho em `ui state` (`accepted` quando aplicou as
  regras como propostas, `accepted_edited` quando as alterou primeiro, `rejected`
  quando fechou o painel). Uma proposta de cada vez; é recusada enquanto outro
  diálogo seu estiver aberto.

## Ferramentas

A maior parte de uma reescrita do histórico não precisa de nada além do Git:
mudar autores e committers, aplicar um `.mailmap`, anonimizar, deslocar datas
ou limitá-las a um horário, reescrever mensagens e trailers, substituir
conteúdo, remover caminhos ou ficheiros grandes e mover caminhos correm todos
dentro do GitPoulpi.

**Os scripts Lua estão integrados.** O interpretador vem dentro do GitPoulpi,
por isso um script Lua não precisa de nenhuma ferramenta nem de configuração.
Os scripts de shell, Python e Lua instalado precisam do respetivo interpretador
(veja a tabela abaixo) e estão descritos em
[Scripts externos](#scripts-externos-shell-python-lua).

Algumas funcionalidades usam um programa externo. O GitPoulpi verifica se
existe (o programa é procurado no seu `PATH`, no Windows também através de
`PATHEXT` e da pasta `usr/bin` do próprio Git for Windows, e tem de responder a
`--version` em 2 segundos); a resposta fica em cache durante um minuto e
"Verificar de novo" pergunta outra vez. Uma regra que precisa de um programa em
falta continua na lista, esbatida, com o motivo e a forma de o instalar, e a
pré-visualização recusa antes de qualquer coisa correr.

| Funcionalidade | Ferramenta | Notas |
|---|---|---|
| Voltar a assinar commits | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) ou `gpgsm` (`gpg.format = x509`) | A assinatura SSH e X.509 também precisa de `user.signingkey`. `gpg.program`, `gpg.ssh.program` e `gpg.x509.program` são respeitados. |
| Migração para LFS (as duas regras LFS) | `git-lfs` 3.0 ou posterior | A mesma verificação do painel LFS. |
| Scripts Python | `python` 3.8 ou posterior (`py -3` no Windows) | Desativados por predefinição: acrescente `"python"` a `rewrite.external_scripts`. |
| Scripts Lua (instalado) | `lua5.4`, `lua54`, `lua` ou `luajit` | Desativados por predefinição: acrescente `"lua"` a `rewrite.external_scripts`. |
| Scripts de shell | `bash`, `pwsh` (PowerShell) ou `sh` | Ativados por predefinição (`"shell"`). Um script PowerShell (um ficheiro `.ps1`, um shebang `pwsh`, ou qualquer script de shell no Windows) precisa de `pwsh` ou `powershell`; um POSIX precisa de `bash` ou `sh` (no Windows, o Git for Windows fornece-os). |

Sugestões de instalação, Windows primeiro:

- Windows: o Git for Windows inclui `gpg`, `ssh-keygen`, `bash`, `sh` e o Git
  LFS. Caso contrário, `winget install GnuPG.GnuPG`, Definições >
  Funcionalidades opcionais > Cliente OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: os pacotes da distribuição (`gnupg`, `openssh`, `git-lfs`,
  `python3`, `lua5.4`, `bash`, `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` e
  `sh` vêm integrados.
