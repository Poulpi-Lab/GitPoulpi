[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Solução de problemas e FAQ

Comece pelas respostas rápidas abaixo; as seções mais longas tratam do Git LFS e da
renderização do grafo. Se nada ajudar, abra uma issue no repositório do GitHub com o
seu SO, o seu terminal, `gitpoulpi --version` e o que você fez.

## Quando algo falha

Uma operação que falha não mostra pop-up nem cartão: a **linha de status** diz isso, em vermelho, com o título e o motivo (*erro: Falha no fetch: Não foi possível alcançar o remoto.*), e ela fica ali até outra mensagem substituí-la. O botão Atividade, na extremidade direita, conta os erros que você ainda não viu. Clique na mensagem vermelha, ou execute `Registro de atividade: abrir o último erro` da paleta, para abrir o Registro de atividade nessa entrada. Toda falha que você provocou funciona assim, não só as operações do git:

- **Operações do git**: um fetch, um push, um pull, um checkout, um merge, um rebase, um commit, um stash...
- **Um clone** (e a criação de um repositório novo): “O clone falhou”.
- **Uma chamada ao GitHub ou ao GitLab**: as listas de repositórios dos seletores e do diálogo Clonar, a criação de um pull request ou de uma issue, a verificação de um token ou de uma chave SSH na página Integrações: “A requisição ao serviço de hospedagem falhou”.
- **Um programa externo**: o editor, a ferramenta de diff ou de merge, o gerenciador de arquivos, o navegador, o terminal ou o lançador, um editor de imagens, os botões *Testar* das Configurações: “A ferramenta externa falhou”.
- **Uma requisição de IA** (uma mensagem de commit, uma explicação, uma sugestão para um conflito...): “A requisição de IA falhou”.
- **Um arquivo que não pôde ser gravado**: Salvar do editor integrado, o editor de merge, uma configuração que não pôde ser salva: “Falha ao salvar”.

A mensagem vermelha fica na linha de status seja qual for a tela da frente: uma aba de repositório, o Início, o Radar, os Espaços de trabalho, as Configurações (na linha de baixo da tela), sob o Registro de atividade ou um diálogo. Onde a linha de status está coberta (Início, Radar, Espaços de trabalho, Configurações), a mensagem é escrita sobre a linha de baixo até você olhar o erro. Ela acompanha você de uma aba para outra. Uma operação que ainda está em andamento quando você troca de aba não é acompanhada: abra o Registro de atividade para ver como ela terminou.

As listas das seções PULL REQUESTS e ISSUES que não podem ser atualizadas mantêm, em vez disso, a própria linha na seção (elas se atualizam sozinhas, uma mensagem a cada poucos segundos seria só ruído). Um diálogo que ainda está aberto (Clonar, Criar pull request...) também mantém a própria mensagem de uma linha, para você corrigir o campo e tentar de novo.

A entrada no Registro de atividade guarda tudo o que o pop-up mostrava antes:

- **Título e motivo.** “Falha no fetch”, depois uma frase dizendo o porquê em palavras simples, depois o que fazer em seguida. São reconhecidos: o remoto tem commits que você não tem (*faça pull primeiro*), credenciais recusadas (*verifique Configurações > Integrações ou a sua chave SSH*), uma parada por conflitos, alterações locais que seriam sobrescritas (*faça commit, stash ou descarte*), o remoto inacessível e outro processo do git segurando um lock no repositório. Qualquer outra coisa mostra a primeira linha do próprio git. Um clone ou uma chamada ao host que falha do mesmo jeito (credenciais recusadas, host inacessível) é redigido do mesmo jeito; as outras falhas recebem uma frase sobre o que falhou (“Não foi possível clonar o repositório.”, “Não foi possível executar o programa.”).
- **Os detalhes**, mostrados quando a entrada está aberta (`Enter`, ou o clique que a abriu): a operação, o comando git executado, o código de saída dele e as últimas linhas do que ele imprimiu (para a biblioteca git embutida, a classe e o código do erro). Credenciais, tokens e cabeçalhos `Authorization` são mascarados antes de qualquer coisa ser exibida, registrada ou copiada.
- **Copiar** (o botão, ou `c`) coloca na área de transferência um bloco de texto feito para ser colado em um chat com alguém, ou com uma IA, que possa explicá-lo: a versão do GitPoulpi, o sistema, o terminal, o backend, a operação que falhou com os detalhes dela e a versão do git.
- **Um botão para o próximo passo**, quando há um, ao lado de Copiar (ou `r`): *Pull primeiro* (um push que o remoto rejeitaria), *Stash e tentar de novo* (alterações locais atrapalhando um checkout, merge, rebase ou pull), *Tentar de novo* (o remoto não pôde ser alcançado), *Abrir Integrações* (credenciais). Ele executa o passo e fecha a janela. Para uma falha de outro repositório que não o da aba da frente, só *Abrir Integrações* é oferecido: uma nova tentativa rodaria no repositório errado.
- Sem o mouse, a paleta de comandos tem `Registro de atividade: abrir o último erro`, `Registro de atividade: copiar o último erro` e `Registro de atividade: executar a ação sugerida` (o botão de remédio da entrada, para o erro que a linha de status mostra).

Um push que o remoto recusou porque avançou mantém o seu próprio diálogo (pull primeiro, force push com lease, cancelar); os fatos continuam no Registro de atividade.

### O Registro de atividade

O botão **Atividade** na extremidade direita da linha de status (um número nele conta os erros que você ainda não viu), `Ctrl+N` e depois `Shift+A`, e `Exibir: registro de atividade` da paleta abrem a janela. `Ctrl+N` mostra um selo em cada aba, caixa, campo, caixa de seleção e botão da janela (e na primeira entrada da lista); pressionar o caractere faz o que um clique nele faz. Duas abas, **Operações do Git** e **Aplicativo**, que se trocam com `Tab`, com um clique, com `g` (Operações do Git) ou com `p` (Aplicativo), e uma caixa **Escopo** sob elas que vale para as duas (veja abaixo):

- **Aplicativo**: as linhas de registro do próprio programa, as da janela e as do daemon em segundo plano, as mais recentes primeiro. A caixa **Nível** (uma lista suspensa: Erro, Aviso, Info, Depuração, Rastreamento; `l` a abre) diz quanto mostrar; pedir mais detalhe do que está sendo registrado também sobe o nível de registro (e o salva como `log.level`). Embaixo, **Mostrar** tem uma caixa de seleção por nível (Erro, Aviso, Info, Depuração, Rastreamento, cada uma com quantas linhas contém) para ocultar ou mostrar as linhas daquele nível, por exemplo para ver só os erros, e **Tudo** / **Nada** marcam ou desmarcam todas as caixas. As teclas `1` a `5` alternam os níveis nessa ordem, `a` é Tudo, `n` é Nada; a paleta tem `Registro de atividade: mostrar ou ocultar o nível ...`, `Registro de atividade: mostrar todos os níveis` e `Registro de atividade: não mostrar nenhum nível`. Os níveis acima da escolha da caixa Nível ficam acinzentados (nada é registrado para eles; passe o ponteiro sobre um para ver por quê). As marcas são lembradas até você sair, e só a aba Aplicativo as tem. **Filtro** (`/`) mantém as linhas que contêm o seu texto. **Copiar tudo (filtrado)** copia o que está sendo mostrado (as marcas e o filtro valem os dois); **Abrir a pasta de registros** abre a pasta no seu gerenciador de arquivos. Uma linha longa nunca é cortada: veja "Linhas longas" abaixo.
![O Registro de atividade, aba Aplicativo: a caixa Nível, as caixas Mostrar por nível com Tudo e Nada, o filtro e as linhas de registro](../images/activity-log.png)

**Limpar registro** (`x`, ou o comando da paleta `Registro de atividade: limpar o registro do aplicativo`) fica na aba **Aplicativo**: pergunta primeiro e depois esvazia os arquivos de registro de todos os dias mantidos e a lista na tela. Não dá para desfazer e não entra no histórico de desfazer.

- **Operações do Git** (a aba em que a janela abre): as operações, as mais recentes primeiro, cada uma com a hora em que rodou e quanto tempo levou (`Push - 123 ms`): abrir o repositório, carregar o grafo de commits, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. As falhas, as mensagens de status, um clone que falhou e o Fetch all de um espaço de trabalho também são listados aqui; `Enter` ou um clique abre uma (o motivo, o próximo passo, o comando, o código de saída e a saída), `c` ou o botão Copiar dela a copia. `Tab` (ou `p`) passa para a aba Aplicativo; `Left`/`Right` rolam as linhas longas para o lado.
![O Registro de atividade, aba Operações do Git: cada operação com a hora e quanto tempo levou, um fetch com falha em vermelho](../images/activity-log-repository.png)

- **Linhas longas** (as duas abas): um cabeçalho ou uma linha de detalhe mais larga que a janela nunca é cortada. Uma barra de rolagem horizontal aparece sob a lista; role para o lado com `Left` / `Right` (reatribuíveis como `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + a roda do mouse ou uma roda horizontal, `Registro de atividade: rolar para a esquerda` / `Registro de atividade: rolar para a direita` da paleta, ou a barra (arraste o polegar, ou pressione a trilha para trazer o polegar para baixo do ponteiro). `Home` volta ao início, e trocar de aba começa a outra aba na borda esquerda. O texto com quebra de linha (um resumo, uma ação sugerida) continua quebrado.

- **Escopo** (uma lista suspensa sob as abas, `s` a abre, ou `Registro de atividade: escolher o escopo` da paleta): de que tratam as duas abas. As linhas são **Este repositório (nome)** (o repositório da aba a partir da qual você abriu o registro; não é oferecido a partir do Início, que não tem nenhum), **Todos os repositórios**, cada **espaço de trabalho** e cada repositório que a janela conhece. A partir da aba de um repositório o registro abre em *Este repositório*, a partir do Início (o botão Atividade, `Ctrl+N` e depois `Shift+A`) em *Todos os repositórios*; `Ctrl+N` e depois `Shift+G` e `Exibir: registro de atividade global` da paleta sempre abrem em *Todos os repositórios*, e o mesmo vale para um clique na mensagem vermelha de uma falha cujo repositório não é a aba da frente (um clone que falhou, por exemplo). A sua última escolha é lembrada durante a sessão, separadamente para as duas formas de abrir. Na aba Aplicativo, uma linha pertence a um repositório quando o programa a escreveu sobre um (um push que falhou, a duração de uma operação); em *Todos os repositórios* cada linha assim diz de quem é (`[nome]`). Uma linha sobre nenhum repositório (inicialização, configurações, o próprio daemon) e uma entrada que não pertence a nenhum (um salvamento que falhou) são **do aplicativo**: aparecem em todos os escopos, marcadas com `[do aplicativo]` quando o escopo é mais estreito que *Todos os repositórios*, para que um filtro nunca esconda uma falha. A paleta também tem `Registro de atividade: operações do Git` e `Registro de atividade: registro do aplicativo` para as abas.

- **Copiar diagnóstico** (um botão nas duas abas, `d`, sempre o registro inteiro, qualquer que seja o escopo; e `Exibir: copiar diagnóstico` da paleta) copia a versão, o sistema, o terminal e o backend, a última falha com os detalhes e as últimas 200 linhas de registro no nível escolhido: o bloco para dar a uma IA quando algo está errado.

As durações são medidas pela janela (do pedido até a resposta do daemon, ida e volta incluídas), então o que a CLI ou o MCP fazem não é cronometrado. `log.timings` (ligado por padrão) desliga as durações, `log.extended` (desligado por padrão) lista também o trabalho em segundo plano, a atualização periódica e o auto-fetch; os dois ficam em Configurações > Geral junto das outras configurações de registro, e toda operação é gravada de qualquer jeito no registro do aplicativo (`info` para o que você pediu, `debug` para o segundo plano).

Os arquivos de registro são `gitpoulpi-YYYY-MM-DD.log`, mantidos por uma semana por padrão e no máximo dez anos (`log.retention_days`, de 1 a 3650), podados quando o GitPoulpi inicia e rotacionados em `log.max_file_mb` (5 MiB). Onde ficam: [referência de configuração](config-reference.md#log-o-registro-do-aplicativo). A partir de um script, o erro `--json` de um comando que falhou traz os mesmos fatos em `detail`; veja a [referência da CLI](cli-reference.md).

Um clique na mensagem vermelha abre o registro em *Todos os repositórios* com essa entrada aberta. A lista de Operações do Git é a da sessão; a aba Aplicativo guarda o registro do programa entre reinícios.

**Tarefas em segundo plano.** Um fetch, push, pull, clone ou transferência LFS, o Fetch all / Pull all / Clone missing de um espaço de trabalho, uma solicitação de IA e um pull request ou issue sendo enviado são tarefas: a ponta direita da linha de status mostra a que está rodando com um `[x]` que a cancela, e um clique no rótulo (ou `Ctrl+N` e depois `Shift+J`, o comando `Jobs: Show background jobs`) abre o painel com um botão Cancelar por tarefa (também `c` ou `Del`). Um clone roda em segundo plano; uma solicitação de IA e uma varredura de espaço de trabalho sobrevivem à troca de aba e são aplicadas quando você volta ao repositório delas. Uma tarefa além do ponto em que parar deixou de ser seguro fica em cinza com o motivo; uma tarefa cancelada deixa o repositório como estava.

## Respostas rápidas

**A barra de ferramentas e a barra lateral mostram quadrados ou pontos de interrogação
em vez de ícones.** Use uma Nerd Font ou qualquer fonte com cobertura de emoji no seu
terminal. O grafo e o texto continuam funcionando sem elas.

**O grafo de commits é feito de caracteres, não de imagens.** Esse é o recurso
alternativo automático: imagens exigem um terminal que fale o protocolo gráfico do
kitty (kitty, Ghostty, herdr com os gráficos do kitty ativados) ou o protocolo do
iTerm2 (WezTerm, iTerm2, mintty), ou sixel (Windows Terminal, foot, Konsole). O console do Windows, tmux e zellij recebem
caracteres. Veja a seção sobre o grafo abaixo para forçar um ou outro.

**O mouse não faz nada, ou selecionar texto com o mouse não funciona.** O GitPoulpi
captura o mouse para que os cliques cheguem a ele. Para selecionar texto com o
próprio terminal, mantenha `Shift` pressionado enquanto arrasta (na maioria dos
terminais). Você também pode selecionar dentro do diff (arrastar e depois `y`). Para
desativar a captura do mouse, use *Configurações > Teclado e mouse > Mouse ativado*
(exige reiniciar). O teclado alcança todos os comandos.

**Um atalho não funciona.** Seu terminal, o tmux ou um script do AutoHotkey pode
pegar a tecla antes de o GitPoulpi vê-la (`Ctrl+,` costuma chegar como uma simples `,`
nos terminais Linux e macOS; `Ctrl+Shift+N` parece `Ctrl+N`). Use as alternativas
(`Ctrl+N` e depois `,` abre as Configurações; `Ctrl+P` alcança todos os comandos), ou
reatribua o atalho: veja [Configurações](settings.md#teclado-e-mouse).

**Não lembro de um atalho.** Pressione `Ctrl+P` e digite o que você quer; cada
resultado mostra seu atalho. `?` lista tudo.

**Uma opção está acinzentada nas Configurações.** Ela depende de outra opção. A linha
diz qual (por exemplo, *Intensidade do tom da linha* precisa de *Fundo das faixas do
grafo* definido como *Tom da linha*).

**“Auto-stash pendente” sobre o Log.** O GitPoulpi fez stash das suas alterações para
executar uma operação e não conseguiu recolocá-las. Use **Pop agora**, **Mostrar
stash**, **Recomeçar** (desfaz a operação e recoloca suas alterações) ou **Drop** do
aviso; veja [Auto-stash](log-and-history.md#auto-stash).

**Um agente diz `no_viewer` ou `deferred`.** `no_viewer`: nenhuma janela do GitPoulpi
está aberta para esse repositório (ou *Permitir que agentes de IA controlem a
visualização* está desativado). `deferred`: a janela mostra um chip; `Ctrl+N Enter` o
aceita. Veja [MCP e agentes](mcp-and-agents.md#mostrar-coisas-ao-usuário).

**Abri o GitPoulpi e vejo uma janela vazia ou `erro no log: backend error`.** Confira
se você o iniciou dentro de um repositório Git (senão você cai na aba Início: use
**Procurar**, **Clonar** ou **Init**). Depois execute `gitpoulpi daemon status`;
`gitpoulpi daemon restart` reinicia o processo em segundo plano deste repositório.
Pressione `r` para atualizar.

**Duas janelas no mesmo repositório se comportam de forma estranha.** Elas
compartilham um daemon por repositório de propósito, para que uma alteração em uma
apareça na outra. Se um daemon parecer travado, `gitpoulpi daemon stop --all` para o
daemon de todas as sessões; a próxima janela inicia um novo.

**Um commit, pull ou push pede uma senha, ou falha ao autenticar.** O GitPoulpi usa o
auxiliar de credenciais e o ssh-agent do seu próprio `git` para a rede. Confirme antes
que `git fetch` funciona em um terminal comum. Os tokens do GitHub/GitLab para pull
requests são separados e opcionais: veja
[Integrações de hospedagem e de rastreadores de issues](hosting-integration.md).

**Um fetch, pull ou push falha com "O auxiliar de credenciais configurado falhou", "A chave de host SSH do remoto ainda não é conhecida" ou um acesso recusado, e a sua entrada do Registro de atividade tem um botão "SSH e credenciais...".**
A falha vem de como este repositório entra. O botão abre
Configurações > SSH e credenciais, onde você escolhe outra origem para este
repositório: os auxiliares do próprio git com os auxiliares do repositório ignorados, uma
conta de integração, um arquivo de chave (com um arquivo de hosts conhecidos) ou um auxiliar ou comando
de token seu; **Testar conexão** então verifica todos os remotos. Para um
auxiliar que não pôde ser executado, `git config --show-origin --get-all
credential.helper` mostra onde ele está definido.

**"O GitPoulpi não enviou o token da conta para ...".** O remoto do repositório
aponta para um servidor que não é o endereço ao qual a conta escolhida está vinculada
(muitas vezes uma reescrita `url.<x>.insteadOf` na configuração do próprio repositório), então o
token foi retido. Corrija o remoto ou escolha outra origem na página.

**"O comando de token deste repositório falhou", "... mudou desde que você o confirmou", "Não é possível usar o acesso deste repositório".**
As escolhas da página não podem ser executadas: o comando de token não imprimiu nenhum token
(as suas últimas linhas de erro são citadas, sem os segredos), um script que ele executa
mudou desde que você o confirmou (confirme o comando de novo na página) ou a
escolha nomeia um arquivo de chave ou uma conta que não existe mais. Cada leitura tem o botão
**SSH e credenciais...**.

**O OpenSSH diz "bad permissions" para um arquivo de chave.** Outras contas podem ler a
chave. Em Configurações > SSH e credenciais, a linha **Arquivo de chave privada** mostra um
aviso com **Restringir à minha conta**.

**“fazendo stage de ...” fica na linha de status.** Uma atualização em segundo plano ainda está
em andamento; some quando a lista de arquivos é atualizada. `r` força uma atualização.

**Um repositório muito grande está lento.** Diminua *Commits iniciais no grafo* na
página Configurações > Log e grafo (o Log carrega mais conforme você rola quando
*Carregar commits sob demanda no grafo* está ativado); oculte a coluna *Alterações* do
Log e desative os avatares; desative *Desfazer ativado* se você não precisa de desfazer
(nenhum controle interno no stage e nos commits). `GITPOULPI_TIMING=1` grava os tempos
por requisição em `gitpoulpi-timing.log` na pasta temporária, o que vale anexar a um
relatório de desempenho.

**Os botões de IA estão ausentes ou cinza.** A IA vem desativada por padrão; veja
[Assistência de IA](ai.md#solução-de-problemas).

**O GitPoulpi fechou sozinho.** Toda saída é registrada: procure
`gitpoulpi-tui-exit.log` (e `gitpoulpi-tui-panic.txt` após um travamento) no
diretório de configuração (`gitpoulpi config path` imprime o arquivo de configurações,
que fica ali) e anexe-o a uma issue.

**Onde estão minhas configurações?** `gitpoulpi config path`. O estado por repositório
fica em `.gitpoulpi/repo-state.toml` dentro do repositório.

**O GitPoulpi envia alguma coisa para algum lugar?** Só o que você pede: operações git
para os seus remotos, requisições de hospedagem depois que você conecta uma conta,
requisições de IA depois que você pressiona um botão de IA e, se você ativar os
avatares, um hash dos e-mails dos autores para o Gravatar. Não há telemetria. Veja
[Assistência de IA](ai.md#privacidade).

## Toda gravação falha: “Unable to create index.lock”

Um comando git que morreu (um travamento, um terminal encerrado à força) pode deixar
`.git/index.lock` para trás, e então toda alteração no repositório falha. Quando o
arquivo tem mais de 10 segundos, o GitPoulpi mostra um aviso acima do Log: **Remover
index.lock** (paleta: *Repositório: remover index.lock obsoleto*; CLI:
`gitpoulpi index-lock remove`). Ela pergunta antes e exclui apenas esse arquivo. O
GitPoulpi não consegue saber com segurança se outro programa git ainda está rodando,
então verifique que nenhum está (uma IDE, outro terminal) antes de confirmar. Em um
worktree vinculado, o lock é o do próprio worktree, em `.git/worktrees/<name>`.

## O “git am” parou e Rebase Abortar não faz nada

Um `am` (aplicar patches de uma caixa de e-mail) não é um rebase. Um `git am` parado
tem seu próprio aviso, **Aplicando patches (git am)**, com Resolver conflitos,
Continuar, Pular e Abortar (paleta: *git am: Continuar / Pular este patch /
Abortar*; CLI: `conflicts am-continue|am-skip|am-abort`). Abortar pergunta antes,
descarta os patches aplicados até agora e devolve a branch a onde estava. Isso não
pode ser desfeito, como os botões de rebase.

## Desfazer está acinzentado e diz “Não é possível desfazer ...” (entrada bloqueada)

A entrada mais recente do diário está presa: o repositório seguiu em frente fora do
GitPoulpi, ou a entrada foi interrompida. Passe o mouse sobre Desfazer para ver o
motivo; se ela está presa de vez, a dica aponta para **Editar: esquecer a entrada
de desfazer bloqueada** (paleta) ou `gitpoulpi undo-forget`. Ele pergunta antes e só
remove a entrada do diário; o repositório não é tocado. Veja
[Desfazer e refazer](undo-redo.md).

## Arquivos grandes aparecem como três linhas de texto (“arquivos LFS aparecem como ponteiros”)

Um arquivo que deveria ser uma imagem, um modelo ou um grande binário contém `version https://git-lfs.github.com/spec/v1` e uma linha `oid sha256:...`: é um **ponteiro** do Git LFS, e o conteúdo real não foi baixado (ou os filtros do LFS não estão instalados).

- Olhe o painel Status: um repositório que usa LFS enquanto o `git-lfs` está ausente mostra um aviso âmbar. Instale o programa (https://git-lfs.com; Windows: incluído no Git for Windows ou `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) e reabra ou pressione `r`.
- `gitpoulpi lfs status` diz se o programa está instalado, se os filtros e o hook estão configurados e quais padrões são rastreados; `gitpoulpi lfs files` lista os arquivos de ponteiro e quais são `local`.
- Com o `git-lfs` instalado, execute **`LFS: pull de objetos`** na paleta (ou `gitpoulpi lfs pull`): ele baixa os objetos e substitui os ponteiros. Antes disso, instala os filtros do LFS para este repositório se não houver nenhum.
- Se o download falhar, a mensagem nomeia o motivo (credenciais do remoto, um servidor que não tem mais o objeto). O GitPoulpi não precisa de conta própria: o LFS usa as credenciais habituais do seu remoto (auxiliar de credenciais ou ssh-agent).
- Um clone de um repositório LFS nunca falha por causa do LFS; se a etapa de download não pôde rodar, o clone fica lá com ponteiros, e `LFS: pull de objetos` o conclui depois. `gitpoulpi clone --no-lfs` pula o download de propósito.
- Você pode desativar tudo do LFS em Configurações > Git e codificação > Git LFS (ou `[git] lfs_enabled = false`).

## “git-lfs not found” / “O Git LFS não está instalado”

A mensagem `O Git LFS não está instalado. Este repositório guarda arquivos grandes com Git LFS; ...` (ou um diálogo com um botão *Abrir página de instalação*, ou o próprio `git-lfs: command not found` / `'lfs' is not a git command` do git) significa que o repositório está configurado para LFS - seu `.gitattributes`, os filtros do `.git/config` ou o hook `pre-push` chamam o `git-lfs` - e o programa não está no `PATH` visto pelo GitPoulpi. Instale-o (link acima). No Windows o programa vem com o Git for Windows; se `git lfs version` funciona em um terminal mas o GitPoulpi ainda diz que ele está ausente, inicie o GitPoulpi de um terminal que veja o mesmo `PATH`. Para esconder o LFS do GitPoulpi deixando o comportamento do próprio git em paz, desative `git.lfs_enabled`.

## O grafo é desenhado com caracteres, não como imagem (herdr, tmux, outros terminais)

O GitPoulpi desenha a coluna Grafo e os avatares dos autores como pixels somente onde tem certeza de que o terminal os mostrará: imagens inline do iTerm2 no WezTerm puro, no iTerm2 e no mintty; o protocolo gráfico do Kitty no kitty, no Ghostty e **no herdr quando a própria configuração do herdr define `kitty_graphics = true`** (`[terminal]`, ou `[experimental]` em versões mais antigas do herdr) e o herdr roda dentro do WezTerm, kitty ou Ghostty. As imagens sixel são usadas no Windows Terminal, foot, Konsole e mlterm. Em todo o resto ele desenha caracteres, o que sempre está correto.

- No herdr com os gráficos do Kitty ativados mas com uma configuração que o GitPoulpi não encontra (ou contando com o padrão do herdr), defina `ui.image_protocol = "kitty"` (Configurações > Log e grafo > Protocolo de imagem) e reinicie.
- Em um terminal que mostra quadrados ou lixo onde o grafo deveria estar, defina `ui.image_protocol = "none"` ou `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` desativa as imagens independentemente da configuração.

**herdr remoto (`herdr --remote <host>`, ou `ssh` e depois `herdr`): o grafo de texto é o padrão.** Por SSH, o GitPoulpi roda no host remoto e só consegue ler o ambiente do remoto e o `config.toml` do herdr. Se uma imagem do Kitty aparece é decidido pelo cliente herdr *local* (sua própria configuração `kitty_graphics` e seu próprio terminal), que o GitPoulpi não consegue ver: quando o cliente não desenha as imagens, as células ficam em branco. O GitPoulpi, portanto, desenha o grafo de texto sempre que detecta o herdr por SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Se o seu herdr local tem `kitty_graphics = true` e um terminal que desenha imagens do Kitty (kitty, Ghostty), ative com `ui.graph_renderer = "image"` (ou `ui.image_protocol = "kitty"`) no remoto. As imagens são enviadas inline (`t=d`), o que funciona em qualquer fluxo de bytes. Se a coluna ficar vazia, defina `ui.graph_renderer = "text"`.

**Um retângulo mais claro atrás das faixas.** A imagem é transparente (PNG RGBA com alfa direto) onde nada é desenhado, então o fundo do terminal e a faixa de seleção aparecem através dela. Os terminais com o protocolo Kitty e o **WezTerm** compõem esse alfa; o GitPoulpi então reserva células em branco comuns sob a imagem e a envia depois delas. Todo outro terminal com o protocolo iTerm2 (o próprio iTerm2, o Rio, o mintty) não é conhecido por misturar o alfa do PNG, então ali a imagem leva um fundo opaco na cor do tema, que pode aparecer como um retângulo em um terminal cujo fundo difere dele; `ui.graph_renderer = "text"` o remove. Com `ui.graph_lane_background = "off"`, nada é pintado atrás das faixas nos caminhos transparentes.

Veja `ui.graph_renderer` e `ui.image_protocol` na [referência de configuração](config-reference.md).
