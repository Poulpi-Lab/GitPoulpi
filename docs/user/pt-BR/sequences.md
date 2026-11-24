Uma **sequência** é uma lista nomeada de comandos que executam um após o outro: por
exemplo *Fazer stage de todas as alterações*, *IA: gerar mensagem de commit*, *Commit*, *Push*
como uma única tecla **Backup**. Cada passo é um comando da
[paleta de comandos](keybindings.md); uma sequência é uma linha própria da paleta
(`Sequência: <nome>`), pode ser vinculada a uma tecla e é editada em
**Configurações > Sequências** ou no `config.toml`.

## A página Sequências

**Configurações > Sequências** é uma [página de lista](settings.md#listas): as sequências à
esquerda, as linhas da selecionada à direita. Toda alteração é aplicada e
gravada na hora, e a paleta e o mapa de teclas acompanham sem reiniciar.

```
 Sequências
+- Sequências ------------+  Editando a sequência 1/2
| Esta instalação         |  Nome         Backup
| > Backup        Alt+B   |  Id           backup
|   Worktree e agente     |  Descrição    Stage de tudo, ...
| Este repositório        |  Escopo       Esta instalação
|   Release               |  Atalho       [ Alt+B ] [ Limpar ]
+-------------------------+  Passos       Stage... > IA: ... > Commit > Push
| [+ Adicionar v] [Dup]   |              [ Editar passos (4) ] [ Executar agora ]
| [Remover] [Sobe] [Desce]|
+-------------------------+
```

* **+ Adicionar** oferece **Sequência vazia** e os três modelos, **A partir de um modelo >
  Backup**, **Compor e push** e **Worktree e agente**. Um modelo é um
  ponto de partida: nada fica ativo até você adicioná-lo, e o que você adiciona é uma
  cópia que você pode editar. Os modelos recebem o nome no idioma da interface quando
  são adicionados.
* **Duplicar**, **Remover** (pergunta antes), **↑** / **↓** ou um arrasto reordenam
  a lista (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). A ordem da
  lista é a ordem no `config.toml`.
* Uma linha com problema é marcada em vermelho e diz por quê (nenhum passo, um passo que esta
  versão não conhece, uma pausa `Nunca` sem a opção de que precisa).
  Uma sequência com problemas **pode ser salva**, passos futuros desconhecidos são mantidos como
  estão, mas a linha dela na paleta continua esmaecida até ser corrigida.

### As linhas de uma sequência

| Linha | O que faz |
|---|---|
| **Nome** | O que a paleta mostra: `Sequência: <nome>`. |
| **Id** | O nome curto usado no `config.toml`, na linha de comando e como chave do atalho (`a-z`, `0-9`, `_`, `-`). Renomeá-lo leva o atalho junto, na mesma gravação. |
| **Descrição** | Opcional, para você mesmo. |
| **Escopo** | Veja abaixo. |
| **Atalho** | Um botão que mostra a tecla. `Enter` ou um clique abre a mesma captura de tecla de **Teclado e mouse**: conflitos e avisos do terminal aparecem enquanto você pressiona, e uma tecla que outra ação usa é recusada do mesmo modo. **Limpar** a remove. A tecla é salva como `[keybindings.sequence] <id>`. |
| **Passos** | Os passos em ordem. **Editar passos** os abre. |
| **Executar agora** | Executa a sequência na janela atual. |

### Escopo

Cada sequência tem um escopo, salvo com ela:

| Escopo | Onde é salva | Quem a vê |
|---|---|---|
| **Esta instalação** (padrão) | `[[sequences]]` no `config.toml`, com esta instalação como dona | Só esta instalação do GitPoulpi. |
| **Todas as instalações deste computador** | `[[sequences]]` no `config.toml`, sem dono | Toda instalação que compartilha o arquivo. |
| **Só este repositório** | `.gitpoulpi/sequences.toml` no repositório atual | Quem tem o repositório. Só vale depois que você confia nela (veja abaixo). |

Mudar o escopo move a definição: escolher **Só este repositório**
a grava no arquivo do repositório e a remove do `config.toml`, e
o inverso. Um escopo de repositório precisa de um repositório aberto.

### Sequências de repositório e confiança

O arquivo `.gitpoulpi/sequences.toml` pode ser versionado, então pode vir de
outras pessoas, e uma sequência executa comandos na sua máquina. As sequências de um
repositório, portanto, **não são oferecidas até você confiar nelas**, como acontece com os
comandos de um arquivo de fluxo de trabalho versionado.

Quando um repositório tem um arquivo desses em que ninguém confiou ainda, um aviso aparece sobre
o Log: *Este repositório define 2 sequências* com **Revisar**, **Confiar** e
**Ignorar**. Ele nunca bloqueia nada.

- **Revisar** lista cada sequência com cada passo, a pausa e as opções dele,
  somente leitura. **Confiar** ali confia no que foi listado; **Fechar** (ou `Esc`)
  não confia em nada. Se a lista é maior que a janela, Confiar não é oferecido
  ali (aumente a janela): você nunca confia no que não pôde ler.
- **Confiar** (o botão, ou o *Sequência: confiar nas sequências deste repositório* da
  paleta) confia no arquivo exatamente como está agora.
- **Ignorar** oculta o aviso até o arquivo mudar.
- Pelo teclado: a paleta tem *Sequência: revisar / confiar nas / ignorar as sequências deste
  repositório*.

O que se registra é um resumo (digest) dos ids, dos comandos, das pausas e das
opções, mantido no estado local do repositório (nunca no repositório). Se
algum deles mudar, por exemplo depois de um pull, o aviso volta como *A
sequência deste repositório mudou* e elas saem da paleta e das teclas
até você confiar de novo. Renomear uma sequência ou editar a descrição dela
não altera nada. Uma sequência que você escreve nas Configurações continua confiável quando
o arquivo já era confiável (ou estava vazio) antes de você editá-lo.

Depois de confiáveis, as sequências de repositório aparecem como `Sequência: <nome>` na
paleta como as suas e podem receber um atalho nas Configurações. Uma sequência com
o mesmo id de uma das suas é um choque: a sua vence e a outra é informada
como duplicada.

Pela linha de comando: `gitpoulpi sequence list` mostra as sequências do arquivo
com `trusted` e um estado `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (e `untrust`, `--repo PATH` para outro
repositório) é para uma pessoa executar depois de ler `sequence show`.
`sequence run` recusa uma sequência de repositório não confiável. Agentes de IA não podem
confiar: não há ferramenta MCP para isso.

## Passos

**Editar passos** troca a lista pelos passos da sequência (o título
diz **Passos de <nome>**) e as linhas pelas do passo selecionado. **Voltar**
(a cruz no topo das linhas) ou `Esc` volta às sequências.

* **+ Adicionar** abre **Adicionar um passo**: todo comando que uma sequência pode executar, sob o
  nome dele na paleta (os de IA com o brilho), filtrado enquanto você digita com a
  busca da paleta. Um passo que esta versão não pode executar fica esmaecido com o
  motivo. `Enter` ou um clique o adiciona no fim; `Esc` ou a cruz fecha
  o popup.
* **Duplicar**, **Remover**, **↑** / **↓** e arrastar funcionam como nas
  sequências.
* Um comando destrutivo (excluir uma branch, descartar alterações...) nunca é
  oferecido: uma sequência não pode executar um.

### Pausa

Cada passo tem uma **Pausa**:

| Pausa | Significado |
|---|---|
| **Padrão** | O passo só para quando precisa de uma resposta, no diálogo habitual dele (um nome de branch, uma mensagem de commit que ainda está vazia...). |
| **Sempre** | Para antes deste passo e pede para continuar. |
| **Nunca** | Nunca para. As opções do passo precisam dar tudo de que ele precisa, então um passo que pede um nome precisa da opção **Nome** preenchida. |

Um passo que sempre pergunta antes de executar (um force push) não oferece
**Nunca**: a linha diz por quê. **Nunca** no passo da mensagem da IA é permitido
como escolha explícita.

### Opções

As linhas sob **Pausa** são as opções do passo: um texto **Nome** ou **Mensagem**,
uma **Branch**, uma caixa de seleção como **Abrir em nova aba**, ou uma lista suspensa
preenchida a partir da sua configuração (um **Remoto**, um **Perfil de IA**, uma entrada de **Abrir
em**). Os textos aceitam as variáveis `${branch}`, `${date}`, `${time}` e
`${n}` (um número que torna o nome único). Limpar um texto remove a
opção.

## Criar e editar com IA

Com a [IA ativada](ai.md#rascunho-de-sequência) (Configurações > IA), duas entradas aparecem:
**Adicionar > Criar com IA...** (também a linha de paleta `Sequência: criar com IA...`,
que abre as Configurações nesta página com o diálogo) e **Editar com IA...** ao lado
de **Executar agora** para a sequência selecionada. Nenhuma existe enquanto a IA está desativada.

```
+- Criar uma sequência com IA ------------------------------- x -+
| Descreva a sequência que você quer                            |
| +-----------------------------------------------------------+ |
| | stage de tudo, escrever a mensagem do commit com a IA,    | |
| | commit e depois push                                      | |
| +-----------------------------------------------------------+ |
| Enter: nova linha. Ctrl+Enter: enviar. Esc: fechar.           |
| [ ] Enviar o contexto (nomes de remotos, perfis de IA, rótulos de Abrir em) |
|    Desligado: só a sua descrição é enviada.                   |
| A IA só propõe. Nada é salvo antes de você aceitar...         |
| [ Enviar ] [ Cancelar ]                                       |
+---------------------------------------------------------------+
```

Você escreve o que quer com as suas palavras; **Enviar** (ou `Ctrl+Enter`, `Ctrl+S`)
consulta o modelo, pelo mesmo perfil, indicador de trabalhos e `Esc` de toda
outra ação de IA (e a mesma pergunta "O que é enviado"). A resposta é uma
**proposta** que você lê antes de qualquer coisa acontecer:

* o nome, o id, a descrição e os passos com a pausa e as opções deles;
* para **Editar com IA**, um diff contra a sequência atual: `+` um passo adicionado,
  `-` um passo removido, `~` um passo cuja pausa ou opções mudaram, `~ Nome: "a"
  -> "b"` para um campo;
* **Deixado de fora**: um passo que o modelo propôs e que não é permitido (desconhecido,
  destrutivo, uma sequência dentro de uma sequência, uma pausa impossível), com o
  motivo; os passos válidos são mantidos;
* as **notas** do modelo e um **atalho sugerido** que só é mostrado, com
  a informação de se a tecla já está em uso. Ele nunca é vinculado: use o botão
  **Atalho** depois.

```
+- Alteração proposta -------------------------------------- x -+
| + adicionado   - removido   ~ alterado                        |
| ~ Nome: "Diária" -> "Diária 2"                                |
| Passos (3)                                                    |
| + 1. Git: fetch de todos os remotos                           |
|   2. Git: stage de todas as alterações                        |
| - Git: commit...                                              |
| ~ 3. Git: push: padrões -> Pausa: Sempre                      |
| Deixado de fora                                               |
| passo 4 `reset_hard`: passos destrutivos não são permitidos   |
| [ Aceitar ] [ Editar antes de salvar ] [ Rejeitar ] [ Voltar ]|
+---------------------------------------------------------------+
```

* **Aceitar** grava com uma só gravação (uma sequência nova, ou a substituição da
  editada, que mantém o id, o escopo e a tecla) e a seleciona na lista.
* **Editar antes de salvar** faz o mesmo e abre os passos dela para você ajustar
  as linhas (esta página salva cada edição na hora, então a sequência já está
  salva).
* **Rejeitar** não grava nada. **Voltar** retorna à descrição para enviar de novo.
* **Aceitar** fica indisponível quando não há nada a aceitar: nenhum passo utilizável, um
  problema que a página sinalizaria, ou uma edição que não altera nada.

Nada é gravado antes de **Aceitar**, e uma sequência nunca é executada daqui.
Fechar o diálogo (a cruz, `Esc`, um clique fora) pergunta "Descartar alterações?"
enquanto uma descrição está digitada ou uma proposta está aberta; fechar as Configurações também pergunta.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<id desta instalação>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Chaves que esta versão não conhece, e passos que ela não conhece, são mantidos quando
a página salva.

## Executando uma sequência

Uma sequência é uma lista nomeada de passos, cada um um comando da paleta (stage de tudo, escrever a mensagem do commit com a IA, commit, push...). Execute-a pela paleta (`Sequência: <nome>`), com a tecla que você deu a ela em Configurações > Teclado e mouse (`[keybindings.sequence]` no `config.toml`), ou pela linha de comando (`gitpoulpi sequence run`).

Enquanto executa, uma faixa sobre o topo do Log mostra `<nome> k/n`, o passo e o que ela espera. A execução também aparece no painel de Trabalhos como `Sequência <nome> k/n: <passo>`. Uma sequência executa por vez; iniciar uma segunda diz qual está em execução.

**Onde ela para para perguntar.** Cada passo tem uma configuração `pause` (`default`, `always`, `never`):

| Passo | Padrão | `pause = "never"` |
|---|---|---|
| Mensagem de commit pela IA | a mensagem cai no campo de mensagem; a faixa diz "Revise a mensagem e depois Continuar" | segue na hora |
| Commit | uma mensagem vazia sempre para: o campo fica com o teclado, e o seu commit (ou Continuar) segue | indisponível quando não há mensagem para commitar |
| Compor commits com IA | a revisão abre; Aplicar segue, fechá-la cancela | aplica a proposta como está |
| Criar worktree | o diálogo abre com o nome preenchido | cria a partir de `options.name` sem o diálogo |
| Push | quando `git.push_behavior = ask` o seletor "Push para..." abre | faz push para `options.remote` |
| Force push com lease, stage de arquivos que parecem segredos | sempre pergunta | não permitido |

`pause = "always"` em qualquer passo mostra "Próximo: <passo>" com **Continuar** e **Cancelar** antes de ele executar. Se o push é rejeitado porque a branch divergiu, o diálogo habitual abre e a faixa diz "Push recusado: escolha no diálogo"; fazer pull e depois push (ou um force push com lease) continua a execução, fechar o diálogo a cancela.

**Continuar** é o botão da faixa, `Sequência: continuar` na paleta, ou a tecla vinculada a `sequence.continue`. **Cancelar** é a cruz de fechar da faixa, `Sequência: cancelar` (`sequence.cancel`), o painel de Trabalhos, ou `Esc` quando mais nada o quer (um diálogo aberto fecha primeiro, o que também cancela). Cancelar mantém os passos que já executaram e diz isso: "<nome> cancelada após o passo k/n (<passo>); os passos anteriores foram mantidos".

**Quando um passo falha** a execução para ali: a linha de status nomeia a sequência, o passo e o erro, a faixa fica vermelha com **Detalhes** (a entrada do Log de atividade, que lista as opções do passo) e uma cruz de fechar, e os passos anteriores continuam feitos.

**Worktrees.** Um passo de criar worktree com `open_tab = true` leva a execução para a aba do novo worktree: os passos seguintes (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) agem ali. Se outra aba está na tela, a faixa diz "esperando a aba <nome>" com **Ir para lá**. Fechar essa aba cancela a execução. Uma execução não sobrevive a sair do GitPoulpi.

### Um agente propondo uma execução

Com *Deixar agentes de IA proporem alterações para eu confirmar* ligado (`[mcp]
allow_ui_propose`), um agente pode pedir à sua janela que execute uma sequência
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Um banner aparece:
*✦ Proposed by <agente>: run sequence <nome>* com **Run** e **Dismiss**.
Nada executa até você pressionar Run (clique, ou `Ctrl+N` e depois `Enter`);
Dismiss, `Esc` (quando mais nada o quer) ou o *Agente: dispensar a visualização
pendente* da paleta o descarta. A execução então segue exatamente como se você a tivesse iniciado,
pausando onde a sequência manda. A proposta é recusada quando a sequência
não existe, é inválida ou é uma sequência de repositório em que você não confiou.

## Desfazendo uma execução

Todo passo que altera o repositório (stage, commit, tag, branch, stash, worktree...) deixa a sua **própria entrada comum de Desfazer**, marcada com a execução, então `Ctrl+Z` ainda desfaz um passo por vez e o [Histórico de desfazer](undo-redo.md#o-histórico-de-desfazer) mostra `⟳ <nome>` nelas. Para desfazer a execução toda existe **Desfazer a sequência inteira**: o botão do Histórico de desfazer (`x`), a linha de paleta `Editar: desfazer a sequência inteira`, ou `gitpoulpi undo --sequence`. Ela desfaz as entradas da execução da mais nova para a mais antiga, cada uma com as próprias verificações e perguntas (um commit que já foi enviado pergunta antes), para na primeira recusa nomeando o passo e deixa as mais antigas no diário. A mensagem da IA e um push não deixam entrada: um push nunca é desfeito. Só o diário do repositório atual é tocado; os passos feitos em um worktree são desfeitos a partir desse repositório. O refazer continua por entrada. Detalhes: [Execuções de sequência](undo-redo.md#execuções-de-sequência).
