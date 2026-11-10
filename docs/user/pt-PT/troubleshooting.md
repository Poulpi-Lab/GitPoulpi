[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · **Português (Portugal)** · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Resolução de problemas e FAQ

Comece pelas respostas rápidas abaixo; as secções mais longas tratam do Git LFS e
da renderização do grafo. Se nada ajudar, abra uma issue no repositório do GitHub
com o seu SO, o seu terminal, `gitpoulpi --version` e o que fez.

## Quando algo falha

Uma operação que falha não mostra nenhuma janela nem cartão: é a **barra de estado** que o diz, a vermelho, com o título e o motivo (*error: Fetch failed: The remote could not be reached.*), e a mensagem fica aí até outra a substituir. O botão Atividade, na sua extremidade direita, conta os erros que ainda não viu. Clique na mensagem vermelha, ou execute o comando da paleta `Registo de atividade: abrir o último erro`, para abrir o Registo de atividade nessa entrada. Toda a falha que provocou funciona assim, não só as operações do git:

- **Operações do git**: um fetch, um push, um pull, um checkout, um merge, um rebase, um commit, um stash...
- **Um clone** (e a criação de um repositório novo): «O clone falhou».
- **Uma chamada ao GitHub ou ao GitLab**: as listas de repositórios dos seletores e do diálogo Clonar, a criação de um pull request ou de uma issue, a verificação de um token ou de uma chave SSH na página Integrações: «O pedido ao serviço de alojamento falhou».
- **Um programa externo**: o editor, a ferramenta de diff ou de merge, o explorador de ficheiros, o navegador, o terminal ou o lançador, um editor de imagens, os botões *Testar* das Definições: «A ferramenta externa falhou».
- **Um pedido de IA** (uma mensagem de commit, uma explicação, uma sugestão para um conflito...): «O pedido de IA falhou».
- **Um ficheiro que não pôde ser escrito**: Guardar do editor integrado, o editor de merge, uma definição que não pôde ser guardada: «Falha ao guardar».

A mensagem vermelha está na barra de estado seja qual for a vista da frente: um separador de repositório, o Início, o Radar, as Áreas de trabalho, as Definições (na linha inferior do ecrã), por baixo do Registo de atividade ou de um diálogo. Onde a barra de estado está tapada (Início, Radar, Áreas de trabalho, Definições), a mensagem é escrita sobre a linha inferior até ter visto o erro. Acompanha-o de um separador para outro. Uma operação que ainda decorre quando muda de separador não é acompanhada: abra o Registo de atividade para ver como acabou.

As listas das secções PULL REQUESTS e ISSUES que não podem ser atualizadas mantêm, em vez disso, a sua própria linha na secção (atualizam-se sozinhas, uma mensagem a cada poucos segundos seria só ruído). Um diálogo que ainda está aberto (Clonar, Criar pull request...) também mantém a sua própria mensagem de uma linha, para poder corrigir o campo e tentar de novo.

A entrada no Registo de atividade tem tudo o que a janela mostrava:

- **Título e motivo.** «Falha no fetch», depois uma frase a dizer o porquê em palavras simples, depois o que fazer a seguir. São reconhecidos: o remoto tem commits que não tem (*faça primeiro um pull*), credenciais recusadas (*verifique Definições > Integrações ou a sua chave SSH*), uma paragem por conflitos, alterações locais que seriam substituídas (*faça commit, stash ou descarte-as*), o remoto inacessível e outro processo do git a segurar um bloqueio no repositório. Qualquer outra coisa mostra a primeira linha do próprio git. Um clone ou uma chamada ao host que falha da mesma maneira (credenciais recusadas, host inacessível) é redigido da mesma maneira; as outras falhas recebem uma frase sobre o que falhou («Não foi possível clonar o repositório.», «Não foi possível executar o programa.»).
- **Os detalhes**, mostrados quando a entrada está aberta (`Enter`, ou o clique que a abriu): a operação, o comando git executado, o código de saída e as últimas linhas do que imprimiu (para a biblioteca git integrada, a classe e o código do erro). As credenciais, os tokens e os cabeçalhos `Authorization` são mascarados antes de qualquer coisa ser mostrada, registada ou copiada.
- **Copiar** (o botão, ou `c`) põe na área de transferência um bloco de texto feito para ser colado numa conversa com alguém, ou com uma IA, que o possa explicar: a versão do GitPoulpi, o sistema, o terminal, o backend, a operação que falhou com os seus detalhes e a versão do git.
- **Um botão para o passo seguinte**, quando existe, ao lado de Copiar (ou `r`): *Pull primeiro* (um push que o remoto rejeitaria), *Stash e tentar de novo* (alterações locais no caminho de um checkout, merge, rebase ou pull), *Tentar de novo* (o remoto não foi alcançado), *Abrir Integrações* (credenciais). Executa o passo e fecha a janela. Numa falha de outro repositório que não o do separador da frente só é oferecido *Abrir Integrações*: uma nova tentativa correria no repositório errado.
- Sem o rato, a paleta de comandos tem `Registo de atividade: abrir o último erro`, `Registo de atividade: copiar o último erro` e `Registo de atividade: executar a ação sugerida` (o botão do passo seguinte da entrada, para o erro que a barra de estado mostra).

Um push que o remoto recusou porque avançou mantém a sua própria caixa de diálogo (pull primeiro, force push com lease, cancelar); os factos continuam no Registo de atividade.

### O Registo de atividade

O botão **Atividade** na extremidade direita da barra de estado (um número nele conta os erros que ainda não viu), `Ctrl+N` e depois `Shift+A`, e `Ver: registo de atividade` da paleta abrem a janela. `Ctrl+N` mostra um selo em cada separador, caixa, campo, caixa de verificação e botão da janela (e na primeira entrada da lista); premir o carácter faz o que um clique nele faz. Dois separadores, **Operações Git** e **Aplicação**, que se trocam com `Tab`, um clique, `g` (Operações Git) ou `p` (Aplicação), e uma caixa **Âmbito** por baixo deles que vale para ambos (ver abaixo):

- **Aplicação**: as linhas de registo do próprio programa, as da janela e as do daemon em segundo plano, as mais recentes primeiro. A caixa **Nível** (uma lista pendente: Erro, Aviso, Info, Depuração, Rasto; `l` abre-a) diz quanto mostrar; pedir mais detalhe do que está a ser registado sobe também o nível de registo (e guarda-o como `log.level`). Por baixo, **Mostrar** tem uma caixa de seleção por nível (Erro, Aviso, Info, Depuração, Rasto, cada uma com quantas linhas contém) para ocultar ou mostrar as linhas desse nível, por exemplo para ver só os erros, e **Tudo** / **Nada** assinalam ou desassinalam todas as caixas. As teclas `1` a `5` alternam os níveis por essa ordem, `a` é Tudo, `n` é Nada; a paleta tem `Registo de atividade: mostrar ou ocultar o nível ...`, `Registo de atividade: mostrar todos os níveis` e `Registo de atividade: não mostrar nenhum nível`. Os níveis acima da escolha da caixa Nível ficam a cinzento (nada é registado para eles; passe o ponteiro por cima de um para ver porquê). As marcas são lembradas até sair, e só o separador Aplicação as tem. **Filtro** (`/`) mantém as linhas que contêm o seu texto. **Copiar tudo (filtrado)** copia o que está a ser mostrado (valem as marcas e o filtro); **Abrir a pasta de registos** abre a pasta no seu gestor de ficheiros. Uma linha longa nunca é cortada: veja «Linhas longas» abaixo.
![O Registo de atividade, separador Aplicação: a caixa Nível, as caixas Mostrar por nível com Tudo e Nada, o filtro e as linhas de registo](../images/activity-log.png)

**Limpar registo** (`x`, ou o comando da paleta `Registo de atividade: limpar o registo da aplicação`) fica no separador **Aplicação**: pergunta primeiro e depois esvazia os ficheiros de registo de todos os dias guardados e a lista no ecrã. Não se pode anular e não consta do histórico de anular.

- **Operações Git** (o separador em que a janela abre): as operações, as mais recentes primeiro, cada uma com a hora a que correu e quanto tempo demorou (`Push - 123 ms`): abrir o repositório, carregar o grafo de commits, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. As falhas, as mensagens de estado, um clone falhado e o Fetch all de uma área de trabalho também são listados aqui; `Enter` ou um clique abre uma (o motivo, o passo seguinte, o comando, o código de saída e o que imprimiu), `c` ou o botão Copiar dela copia-a, `r` ou o botão dela executa o passo sugerido. `Tab` (ou `p`) passa ao separador Aplicação; `Left`/`Right` deslocam as linhas longas para o lado.
![O Registo de atividade, separador Operações Git: cada operação com a hora e quanto tempo demorou, um fetch falhado a vermelho](../images/activity-log-repository.png)

- **Linhas longas** (ambos os separadores): um cabeçalho ou uma linha de detalhe mais larga do que a janela nunca é cortada. Aparece uma barra de deslocação horizontal por baixo da lista; desloque-se para o lado com `Left` / `Right` (reatribuíveis como `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + a roda do rato ou uma roda horizontal, os comandos da paleta `Registo de atividade: deslocar para a esquerda` / `Registo de atividade: deslocar para a direita`, ou a barra (arraste o cursor, ou prima a pista para trazer o cursor para baixo do ponteiro). `Home` volta ao início, e mudar de separador começa o outro separador na margem esquerda. O texto com quebra de linha (um resumo, uma ação sugerida) continua com quebra.

- **Âmbito** (uma lista pendente por baixo dos separadores, `s` abre-a, ou o comando da paleta `Registo de atividade: escolher o âmbito`): de quem tratam os dois separadores. As linhas são **Este repositório (nome)** (o repositório do separador a partir do qual abriu o registo; não é oferecido a partir do Início, que não tem nenhum), **Todos os repositórios**, cada **área de trabalho** e cada repositório que a janela conhece. A partir de um separador de repositório o registo abre em *Este repositório*, a partir do Início (o botão Atividade, `Ctrl+N` e depois `Shift+A`) em *Todos os repositórios*; `Ctrl+N` e depois `Shift+G` e o comando da paleta `Ver: registo de atividade global` abrem sempre em *Todos os repositórios*, tal como um clique no erro vermelho de uma falha cujo repositório não é o separador da frente (um clone falhado, por exemplo). A sua última escolha é lembrada durante a sessão, em separado para as duas formas de o abrir. No separador Aplicação, uma linha pertence a um repositório quando o programa a escreveu a respeito de um (um push falhado, a duração de uma operação); em *Todos os repositórios* cada uma dessas linhas diz de quem é (`[nome]`). Uma linha sobre nenhum repositório (o arranque, as definições, o próprio daemon) e uma entrada que não pertence a nenhum (uma gravação falhada) são **de toda a aplicação**: aparecem em todos os âmbitos, marcadas `[app-wide]` quando o âmbito é mais restrito do que *Todos os repositórios*, para que um filtro nunca esconda uma falha. A paleta também tem `Registo de atividade: operações Git` e `Registo de atividade: registo da aplicação` para os separadores.

- **Copiar diagnóstico** (um botão nos dois separadores, `d`, sempre o registo inteiro, seja qual for o âmbito; e `Ver: copiar diagnóstico` da paleta) copia a versão, o sistema, o terminal e o backend, a última falha com os detalhes e as últimas 200 linhas de registo ao nível escolhido: o bloco a dar a uma IA quando algo está mal.

As durações são medidas pela janela (do pedido à resposta do daemon, ida e volta incluídas), por isso o que a CLI ou o MCP fazem não é cronometrado. `log.timings` (ligado por predefinição) desliga as durações, `log.extended` (desligado por predefinição) lista também o trabalho em segundo plano, a atualização periódica e o auto-fetch; ambos estão em Definições > Geral junto das outras definições de registo, e cada operação é escrita na mesma no registo da aplicação (`info` para o que pediu, `debug` para o segundo plano).

Os ficheiros de registo são `gitpoulpi-YYYY-MM-DD.log`, mantidos uma semana por predefinição e no máximo dez anos (`log.retention_days`, de 1 a 3650), podados quando o GitPoulpi arranca e rodados em `log.max_file_mb` (5 MiB). Onde estão: [referência de configuração](config-reference.md#log-o-registo-da-aplicação). A partir de um script, o erro `--json` de um comando falhado traz os mesmos factos em `detail`; veja a [referência da CLI](cli-reference.md).

Um clique na mensagem vermelha abre o registo em *Todos os repositórios* com essa entrada aberta. A lista de Operações Git é a da sessão; o separador Aplicação mantém o registo do programa entre reinícios.

**Tarefas em segundo plano.** Um fetch, push, pull, clone ou transferência LFS, o Fetch all / Pull all / Clone missing de uma área de trabalho, um pedido de IA e um pull request ou issue a ser enviado são tarefas: a ponta direita da barra de estado mostra a que está a correr com um `[x]` que a cancela, e um clique na etiqueta (ou `Ctrl+N` e depois `Shift+J`, o comando `Jobs: Show background jobs`) abre o painel com um botão Cancelar por tarefa (também `c` ou `Del`). Um clone corre em segundo plano; um pedido de IA e uma passagem pela área de trabalho sobrevivem à troca de separador. Uma tarefa para lá do ponto em que parar deixou de ser seguro fica a cinzento com o motivo; uma tarefa cancelada deixa o repositório como estava.

## Respostas rápidas

**A barra de ferramentas e a barra lateral mostram caixas ou pontos de
interrogação em vez de ícones.** Use uma Nerd Font ou qualquer fonte com
cobertura de emojis no seu terminal. O grafo e o texto continuam a funcionar sem
elas.

**O grafo de commits é feito de carateres, não de imagens.** É a alternativa
automática: as imagens precisam de um terminal que fale o protocolo gráfico do
kitty (kitty, Ghostty, herdr com os gráficos kitty ligados) ou o protocolo do
iTerm2 (WezTerm, iTerm2, mintty), ou sixel (Windows Terminal, foot, Konsole). A consola do Windows, o tmux e o
zellij recebem carateres. Veja a secção do grafo abaixo para forçar uma ou outra.

**O rato não faz nada, ou selecionar texto com o rato não funciona.** O
GitPoulpi captura o rato para que os cliques lhe cheguem. Para selecionar texto
com o próprio terminal, mantenha `Shift` premido enquanto arrasta (na maioria
dos terminais). Também pode selecionar dentro do diff (arrastar, depois `y`).
Para desligar a captura do rato, use *Definições > Teclado e rato > Rato
ativado* (precisa de reinício). O teclado chega a todos os comandos.

**Um atalho não funciona.** O seu terminal, o tmux ou um script AutoHotkey pode
ficar com a tecla antes de o GitPoulpi a ver (`Ctrl+,` chega muitas vezes como
um simples `,` nos terminais Linux e macOS; `Ctrl+Shift+N` parece `Ctrl+N`). Use
as alternativas (`Ctrl+N` e depois `,` abre as Definições; `Ctrl+P` chega a todos
os comandos), ou reatribua a combinação: veja
[Definições](settings.md#teclado-e-rato).

**Não me lembro de um atalho.** Prima `Ctrl+P` e escreva o que quer; cada
resultado mostra o seu atalho. `?` lista tudo.

**Uma opção está a cinzento nas Definições.** Depende de outra opção. A linha diz
qual (por exemplo, *Intensidade do tom da linha* precisa de *Fundo das faixas do
grafo* definido como *Tom da linha*).

**"Auto-stash pendente" por cima do Log.** O GitPoulpi fez stash das suas
alterações para executar uma operação e não as conseguiu repor. Use **Pop
agora**, **Mostrar stash**, **Recomeçar** (desfaz a operação e repõe as suas
alterações) ou **Drop** do aviso; veja
[Auto-stash](log-and-history.md#auto-stash).

**Um agente diz `no_viewer` ou `deferred`.** `no_viewer`: não há nenhuma janela
do GitPoulpi aberta para esse repositório (ou *Permitir que agentes de IA
controlem a vista* está desligado). `deferred`: a janela mostra um chip;
`Ctrl+N Enter` aceita-o. Veja
[MCP e agentes](mcp-and-agents.md#mostrar-coisas-ao-utilizador).

**Abri o GitPoulpi e vejo uma janela vazia ou `erro no log: backend error`.**
Confirme que o iniciou dentro de um repositório Git (caso contrário cai no
separador Início: use **Procurar**, **Clonar** ou **Init**). Depois execute
`gitpoulpi daemon status`; `gitpoulpi daemon restart` reinicia o processo em
segundo plano deste repositório. Prima `r` para atualizar.

**Duas janelas no mesmo repositório comportam-se de forma estranha.**
Partilham um daemon por repositório de propósito, pelo que uma alteração numa
aparece na outra. Se um daemon parecer bloqueado, `gitpoulpi daemon stop --all`
pára o daemon de todas as sessões; a janela seguinte arranca um novo.

**Um commit, pull ou push pede uma palavra-passe, ou falha a autenticação.** O
GitPoulpi usa o credential helper e o ssh-agent do seu próprio `git` para a
rede. Confirme primeiro que `git fetch` funciona num terminal simples. Os tokens
do GitHub/GitLab para pull requests são separados e opcionais: veja
[Alojamento e gestores de issues](hosting-integration.md).

**Um fetch, pull ou push falha com «O assistente de credenciais configurado falhou», «A chave de anfitrião SSH do remoto ainda não é conhecida» ou um início de sessão recusado, e a sua entrada no Registo de atividade tem um botão «SSH e credenciais...».**
A falha vem da forma como este repositório inicia sessão. O botão abre
Definições > SSH e credenciais, onde escolhe outra origem para este
repositório: os assistentes do próprio git com os assistentes do repositório ignorados, uma
conta de integração, um ficheiro de chave (com um ficheiro de hosts conhecidos), ou um assistente ou comando
de token seu; **Testar ligação** verifica depois todos os remotos. Para um
assistente que não pôde correr, `git config --show-origin --get-all
credential.helper` mostra onde está definido.

**«O GitPoulpi não enviou o token da conta para ...».** O remoto do repositório
aponta para um servidor que não é o endereço a que a conta escolhida está associada
(muitas vezes uma reescrita `url.<x>.insteadOf` na configuração do próprio repositório), por isso o
token foi retido. Corrija o remoto ou escolha outra origem na página.

**«O comando de token deste repositório falhou», «... mudou desde que o confirmou», «O início de sessão deste repositório não pode ser usado».**
As escolhas da página não podem ser cumpridas: o comando de token não imprimiu nenhum token
(as suas últimas linhas de erro são citadas, sem segredos), um script que ele executa
mudou desde que o confirmou (confirme o comando de novo na página), ou a
escolha nomeia um ficheiro de chave ou uma conta que já não existe. Cada leitura tem o botão
**SSH e credenciais...**.

**O OpenSSH diz «bad permissions» para um ficheiro de chave.** Outras contas podem ler a
chave. Em Definições > SSH e credenciais, a linha **Ficheiro da chave privada** mostra um
aviso com **Restringir à minha conta**.

**"a fazer stage..." fica na barra de estado.** Uma atualização em segundo plano ainda está
a correr; desaparece quando a lista de ficheiros se atualiza. `r` força uma
atualização.

**Um repositório muito grande é lento.** Reduza *Commits iniciais no grafo* na
página Definições > Log e grafo (o Log carrega mais à medida que se desloca
quando *Carregar commits a pedido no grafo* está ligado); oculte a coluna
*Alterações* do Log e desligue os avatares; desligue *Desfazer ativado* se não
precisar de desfazer (nenhuma contabilidade no staging e nos commits).
`GITPOULPI_TIMING=1` escreve os tempos por pedido em `gitpoulpi-timing.log` na
pasta temporária, útil para anexar a um relatório de desempenho.

**Os botões de IA estão em falta ou a cinzento.** A IA vem desligada por
predefinição; veja [Assistência de IA](ai.md#resolução-de-problemas).

**O GitPoulpi fechou sozinho.** Cada saída fica registada: procure
`gitpoulpi-tui-exit.log` (e `gitpoulpi-tui-panic.txt` depois de um crash) no
diretório de configuração (`gitpoulpi config path` imprime o ficheiro de
definições, que fica aí), e anexe-o a uma issue.

**Onde estão as minhas definições?** `gitpoulpi config path`. O estado por
repositório está em `.gitpoulpi/repo-state.toml` dentro do repositório.

**O GitPoulpi envia alguma coisa para algum lado?** Apenas o que pedir:
operações git para os seus remotos, pedidos de alojamento depois de ligar uma
conta, pedidos de IA depois de premir um botão de IA e, se ligar os avatares, um
hash dos e-mails dos autores para o Gravatar. Não há telemetria. Veja
[Assistência de IA](ai.md#privacidade).

## Todas as escritas falham: "Unable to create index.lock"

Um comando git que morreu (um crash, um terminal terminado à força) pode deixar
`.git/index.lock` para trás, e então todas as alterações ao repositório falham.
Quando o ficheiro tem mais de 10 segundos, o GitPoulpi mostra um aviso por cima
do Log: **Remover index.lock** (paleta: *Repositório: remover index.lock
obsoleto*; CLI: `gitpoulpi index-lock remove`). Pergunta primeiro e elimina
apenas esse ficheiro. O GitPoulpi não consegue saber com fiabilidade se outro
programa git ainda está a correr, por isso certifique-se de que nenhum está (um
IDE, outro terminal) antes de confirmar. Num worktree ligado, o bloqueio é o do
próprio worktree, em `.git/worktrees/<name>`.

## O "git am" parou e Abortar o rebase não faz nada

Um `am` (aplicar patches de uma caixa de correio) não é um rebase. Um `git am`
parado tem o seu próprio aviso, **A aplicar patches (git am)**, com Resolver
conflitos, Continuar, Saltar e Abortar (paleta: *git am: continuar / Saltar este
patch / Abortar*; CLI: `conflicts am-continue|am-skip|am-abort`). Abortar
pergunta primeiro, descarta os patches aplicados até então e põe a branch onde
estava. Estes não são desfazíveis, tal como os botões do rebase.

## O Desfazer está a cinzento e diz "Não é possível desfazer ..." (entrada bloqueada)

A entrada mais recente do diário está presa: o repositório avançou fora do
GitPoulpi, ou a entrada foi interrompida. Passe o ponteiro por Desfazer para ver
o motivo; se estiver presa de vez, a dica aponta para **Editar: Esquecer a
entrada de desfazer bloqueada** (paleta) ou `gitpoulpi undo-forget`. Pergunta
primeiro e apenas retira a entrada do diário; o repositório não é tocado. Veja
[Desfazer e refazer](undo-redo.md).

## Os ficheiros grandes aparecem como três linhas de texto ("os ficheiros LFS aparecem como ponteiros")

Um ficheiro que devia ser uma imagem, um modelo ou um binário grande contém
`version https://git-lfs.github.com/spec/v1` e uma linha `oid sha256:...`: é um **ponteiro** do Git LFS, e o conteúdo real não foi transferido (ou os filtros LFS não estão instalados).

- Olhe para o painel Status: um repositório que usa LFS enquanto o `git-lfs` está em falta mostra um aviso âmbar. Instale o programa (https://git-lfs.com; Windows: incluído no Git for Windows ou `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) e reabra ou prima `r`.
- `gitpoulpi lfs status` diz se o programa está instalado, se os filtros e o hook estão configurados e que padrões são acompanhados; `gitpoulpi lfs files` lista os ficheiros ponteiro e quais são `local`.
- Com o `git-lfs` instalado, execute **`LFS: Pull de objetos`** na paleta (ou `gitpoulpi lfs pull`): transfere os objetos e substitui os ponteiros. Instala primeiro os filtros LFS para este repositório se não houver nenhuns.
- Se a transferência falhar, a mensagem indica o motivo (credenciais do remoto, um servidor que já não tem o objeto). O GitPoulpi não precisa de nenhuma conta própria: o LFS usa as credenciais habituais do seu remoto (credential helper ou ssh-agent).
- Um clone de um repositório LFS nunca falha por causa do LFS; se o passo de transferência não pôde correr, o clone fica com ponteiros, e `LFS: Pull de objetos` termina-o mais tarde. `gitpoulpi clone --no-lfs` salta a transferência de propósito.
- Pode desligar tudo o que é LFS em Definições > Git e codificação > Git LFS (ou `[git] lfs_enabled = false`).

## "git-lfs not found" / "O Git LFS não está instalado"

A mensagem `O Git LFS não está instalado. Este repositório guarda ficheiros grandes com Git LFS; ...` (ou um diálogo com um botão *Abrir página de instalação*, ou o próprio `git-lfs: command not found` / `'lfs' is not a git command` do git) significa que o repositório está configurado para LFS - o seu `.gitattributes`, os filtros de `.git/config` ou o hook `pre-push` chamam o `git-lfs` - e o programa não está no `PATH` visto pelo GitPoulpi. Instale-o (link acima). No Windows o programa vem com o Git for Windows; se `git lfs version` funciona num terminal mas o GitPoulpi continua a dizer que está em falta, inicie o GitPoulpi a partir de um terminal que veja o mesmo `PATH`. Para esconder o LFS do GitPoulpi deixando em paz o comportamento do próprio git, desligue `git.lfs_enabled`.

## O grafo é desenhado com carateres, não como imagem (herdr, tmux, outros terminais)

O GitPoulpi desenha a coluna Grafo e os avatares dos autores como píxeis apenas onde tem a certeza de que o terminal os vai mostrar: imagens em linha do iTerm2 no WezTerm simples, iTerm2 e mintty; o protocolo gráfico do Kitty no kitty, Ghostty, e **no herdr quando a própria configuração do herdr define `kitty_graphics = true`** (`[terminal]`, ou `[experimental]` em herdr mais antigos) e o herdr corre dentro de WezTerm, kitty ou Ghostty. As imagens sixel são usadas no Windows Terminal, foot, Konsole e mlterm. Em todos os outros sítios desenha carateres, o que está sempre correto.

- No herdr com os gráficos Kitty ligados mas com uma configuração que o GitPoulpi não encontra (ou contando com a predefinição do herdr), defina `ui.image_protocol = "kitty"` (Definições > Log e grafo > Protocolo de imagem) e reinicie.
- Num terminal que mostra caixas ou lixo onde devia estar o grafo, defina `ui.image_protocol = "none"` ou `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` desliga as imagens independentemente da configuração.

**herdr remoto (`herdr --remote <host>`, ou `ssh` e depois `herdr`): o grafo de texto é a predefinição.** Por SSH, o GitPoulpi corre no anfitrião remoto e só consegue ler o ambiente do remoto e o `config.toml` do herdr. Se aparece uma imagem Kitty é decidido pelo cliente herdr *local* (a sua própria definição `kitty_graphics` e o seu próprio terminal), que o GitPoulpi não consegue ver: quando o cliente não desenha as imagens, as células ficam em branco. O GitPoulpi desenha por isso o grafo de texto sempre que deteta o herdr por SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Se o seu herdr local tem `kitty_graphics = true` e um terminal que desenha imagens Kitty (kitty, Ghostty), opte por `ui.graph_renderer = "image"` (ou `ui.image_protocol = "kitty"`) no remoto. As imagens são enviadas em linha (`t=d`), o que funciona em qualquer fluxo de bytes. Se a coluna ficar vazia, defina `ui.graph_renderer = "text"`.

**Um retângulo mais claro por trás das faixas.** A imagem é transparente (PNG RGBA de alfa direto) onde nada é desenhado, pelo que o fundo do terminal e a banda de seleção transparecem. Os terminais com protocolo Kitty e o **WezTerm** compõem esse alfa; o GitPoulpi reserva então células em branco normais sob a imagem e envia-a depois delas. Todos os outros terminais com o protocolo iTerm2 (o próprio iTerm2, Rio, mintty) não se sabe que misturem o alfa do PNG, pelo que aí a imagem leva um fundo opaco na cor do tema, que pode aparecer como um retângulo num terminal cujo fundo difere dele; `ui.graph_renderer = "text"` remove-o. Com `ui.graph_lane_background = "off"` nada é pintado por trás das faixas nos caminhos transparentes.

Veja `ui.graph_renderer` e `ui.image_protocol` na [referência de configuração](config-reference.md).
