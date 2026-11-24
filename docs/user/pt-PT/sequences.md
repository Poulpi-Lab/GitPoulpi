[English](../sequences.md) · [Čeština](../cs/sequences.md) · [Deutsch](../de/sequences.md) · [Español](../es/sequences.md) · [Français](../fr/sequences.md) · [Italiano](../it/sequences.md) · [日本語](../ja/sequences.md) · [한국어](../ko/sequences.md) · [Nederlands](../nl/sequences.md) · [Polski](../pl/sequences.md) · [Português (Brasil)](../pt-BR/sequences.md) · **Português (Portugal)** · [Русский](../ru/sequences.md) · [Svenska](../sv/sequences.md) · [Türkçe](../tr/sequences.md) · [Українська](../uk/sequences.md) · [简体中文](../zh-CN/sequences.md)

# Sequências

Uma **sequência** é uma lista nomeada de comandos que correm um a seguir ao outro: por
exemplo *Fazer stage de todas as alterações*, *IA: gerar mensagem de commit*, *Commit*, *Push*
numa só tecla **Cópia de segurança**. Cada passo é um comando da
[paleta de comandos](keybindings.md); uma sequência é uma linha da paleta própria
(`Sequência: <nome>`), pode ser associada a uma tecla, e edita-se em
**Definições > Sequências** ou no `config.toml`.

## A página Sequências

**Definições > Sequências** é uma [página de lista](settings.md#listas): as sequências à
esquerda, as linhas da selecionada à direita. Cada alteração é aplicada e
escrita de imediato, e a paleta e o mapa de teclas acompanham-na sem reiniciar.

```
 Sequências
+- Sequências -------------+  A editar a sequência 1/2
| Esta instalação          |  Nome         Cópia de segurança
| > Cópia de s.   Alt+B    |  Id           backup
|   Worktree e agente      |  Descrição    Stage de tudo, ...
| Este repositório         |  Âmbito       Esta instalação
|   Lançamento             |  Atalho       [ Alt+B ] [ Limpar ]
+--------------------------+  Passos       Stage > IA: ... > Commit > Push
| [+ Adicionar v] [Dup] [Rem]|             [ Editar passos (4) ] [ Executar agora ]
| [Sobe] [Desce]           |
+--------------------------+
```

* **+ Adicionar** oferece **Sequência vazia** e as três predefinições, **A partir de
  predefinição > Cópia de segurança**, **Compor e push** e **Worktree e agente**. Uma predefinição é um
  ponto de partida: nada fica ativo até a adicionar, e o que adiciona é uma
  cópia que pode editar. As predefinições recebem o nome no idioma da interface quando
  são adicionadas.
* **Duplicar**, **Remover** (pergunta primeiro), **Subir** / **Descer** ou um arrasto reordenam
  a lista (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). A ordem da
  lista é a ordem no `config.toml`.
* Uma linha com um problema é marcada a vermelho e diz porquê (sem passos, um passo que esta
  versão não conhece, uma pausa `Nunca` a que falta a opção de que precisa).
  Uma sequência com problemas **pode ser guardada**, os passos futuros desconhecidos são mantidos tal como
  estão, mas a sua linha da paleta fica a cinzento até ser corrigida.

### As linhas de uma sequência

| Linha | O que faz |
|---|---|
| **Nome** | O que a paleta mostra: `Sequência: <nome>`. |
| **Id** | O nome curto usado no `config.toml`, na linha de comandos e como chave do atalho (`a-z`, `0-9`, `_`, `-`). Mudar-lhe o nome move o atalho com ele, na mesma escrita. |
| **Descrição** | Opcional, para si. |
| **Âmbito** | Veja abaixo. |
| **Atalho** | Um botão que mostra a tecla. `Enter` ou um clique abre a mesma captura de tecla de **Teclado e rato**: os conflitos e os avisos do terminal aparecem enquanto prime, e uma tecla que outra ação usa é recusada da mesma forma. **Limpar** remove-a. A tecla é guardada como `[keybindings.sequence] <id>`. |
| **Passos** | Os passos por ordem. **Editar passos** abre-os. |
| **Executar agora** | Executa a sequência na janela atual. |

### Âmbito

Cada sequência tem um âmbito, guardado com ela:

| Âmbito | Onde é guardada | Quem a vê |
|---|---|---|
| **Esta instalação** (predefinição) | `[[sequences]]` no `config.toml`, com esta instalação como dona | Só esta instalação do GitPoulpi. |
| **Todas as instalações neste computador** | `[[sequences]]` no `config.toml`, sem dono | Todas as instalações que partilham o ficheiro. |
| **Só este repositório** | `.gitpoulpi/sequences.toml` no repositório atual | Quem tiver o repositório. Só conta depois de confiar nele (veja abaixo). |

Mudar o âmbito move a definição: escolher **Só este repositório**
escreve-a no ficheiro do repositório e remove-a do `config.toml`, e
o inverso. Um âmbito de repositório precisa de um repositório aberto.

### Sequências de repositório e confiança

O ficheiro `.gitpoulpi/sequences.toml` pode ter commit, por isso pode vir de
outras pessoas, e uma sequência executa comandos na sua máquina. As
sequências de um repositório **não são oferecidas até confiar nelas**, tal como
os comandos de um ficheiro de fluxo de trabalho com commit.

Quando um repositório tem um ficheiro destes em que ninguém confiou ainda, aparece um aviso sobre
o Log: *Este repositório define 2 sequências* com **Rever**, **Confiar** e
**Ignorar**. Nunca bloqueia nada.

- **Rever** lista todas as sequências com todos os passos, a sua pausa e as suas opções,
  só de leitura. **Confiar** aí confia no que foi listado; **Fechar** (ou `Esc`)
  não confia em nada. Se a lista é mais longa do que a janela, Confiar não é oferecido
  aí (aumente a janela): nunca confia no que não pôde ler.
- **Confiar** (o botão, ou *Sequência: confiar nas sequências deste repositório* da
  paleta) confia no ficheiro exatamente como está agora.
- **Ignorar** oculta o aviso até o ficheiro mudar.
- Pelo teclado: a paleta tem *Sequência: rever / confiar nas / ignorar as sequências deste
  repositório*.

O que fica registado é um resumo (digest) dos ids, dos comandos, das pausas e das
opções, guardado no estado local do repositório (nunca no repositório). Se
algum deles mudar, por exemplo depois de um pull, o aviso volta como *As
sequências deste repositório mudaram* e saem da paleta e das teclas
até voltar a confiar nelas. Mudar o nome de uma sequência ou editar a sua descrição
não altera nada. Uma sequência que escreve você mesmo nas Definições continua de confiança quando
o ficheiro já era de confiança (ou estava vazio) antes de o editar.

Depois de confiar, as sequências de repositório aparecem como `Sequência: <nome>` na
paleta como as suas e podem receber um atalho nas Definições. Uma sequência com
o mesmo id de uma das suas é um choque: a sua ganha e a outra é assinalada
como duplicada.

Na linha de comandos: `gitpoulpi sequence list` mostra as sequências do ficheiro
com `trusted` e um estado `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (e `untrust`, `--repo PATH` para outro
repositório) é para uma pessoa executar depois de ler `sequence show`.
`sequence run` recusa uma sequência de repositório sem confiança. Os agentes de IA não podem
confiar: não há nenhuma ferramenta MCP para isso.

## Passos

**Editar passos** substitui a lista pelos passos da sequência (o título
diz **Passos de <nome>**) e as linhas pelas do passo selecionado. **Voltar**
(a cruz no topo das linhas) ou `Esc` regressa às sequências.

* **+ Adicionar** abre **Adicionar um passo**: todos os comandos que uma sequência pode executar, sob o seu
  nome da paleta (os da IA com a faísca), filtrados à medida que escreve com a
  correspondência da paleta. Um passo que esta versão não pode executar aparece a cinzento com o
  motivo. `Enter` ou um clique adiciona-o no fim; `Esc` ou a cruz fecha
  o popup.
* **Duplicar**, **Remover**, **Subir** / **Descer** e arrastar funcionam como nas
  sequências.
* Um comando destrutivo (eliminar uma branch, descartar alterações...) nunca é
  oferecido: uma sequência não pode executar um.

### Pausa

Cada passo tem uma **Pausa**:

| Pausa | Significado |
|---|---|
| **Predefinida** | O passo só pára quando precisa de uma resposta, no seu diálogo habitual (um nome de branch, uma mensagem de commit ainda vazia...). |
| **Sempre** | Pára antes deste passo e pede para continuar. |
| **Nunca** | Nunca pára. As opções do passo têm de lhe dar tudo o que ele precisa, por isso um passo que pede um nome precisa da sua opção **Nome** preenchida. |

Um passo que pergunta sempre antes de executar (um force push) não oferece
**Nunca**: a linha diz porquê. **Nunca** no passo da mensagem da IA é permitido
como escolha explícita.

### Opções

As linhas por baixo de **Pausa** são as opções do passo: um texto **Nome** ou **Mensagem**,
uma **Branch**, uma caixa como **Abrir num novo separador**, ou uma lista
preenchida a partir da sua configuração (um **Remoto**, um **Perfil de IA**, uma entrada de **Abrir
em**). Os textos aceitam as variáveis `${branch}`, `${date}`, `${time}` e
`${n}` (um número que torna o nome único). Limpar um texto remove a
opção.

## Criar e editar com IA

Com a [IA ligada](ai.md#rascunho-de-sequência) (Definições > IA), aparecem duas entradas:
**Adicionar > Criar com IA...** (também a linha da paleta `Sequência: criar com IA...`,
que abre as Definições nesta página com o diálogo) e **Editar com IA...** ao lado
de **Executar agora** para a sequência selecionada. Nenhuma existe enquanto a IA está desligada.

```
+- Criar uma sequência com IA ------------------------------- x -+
| Descreva a sequência que quer                                  |
| +------------------------------------------------------------+ |
| | stage de tudo, escrever a mensagem de commit com a IA,     | |
| | commit e depois push                                       | |
| +------------------------------------------------------------+ |
| Enter: nova linha   Ctrl+Enter: enviar   Esc: fechar           |
| [ ] Enviar contexto (nomes de remotos, perfis de IA, etiquetas)|
|    Desligado: só a sua descrição é enviada.                    |
| A IA só propõe. Nada é guardado antes de aceitar...            |
| [ Enviar ] [ Cancelar ]                                        |
+----------------------------------------------------------------+
```

Escreve o que quer por palavras suas; **Enviar** (ou `Ctrl+Enter`, `Ctrl+S`)
pergunta ao modelo, através do mesmo perfil, indicador de tarefas e `Esc` que qualquer
outra ação de IA (e a mesma pergunta "O que é enviado"). A resposta é uma
**proposta** que lê antes de qualquer coisa acontecer:

* o nome, o id, a descrição e os passos com a sua pausa e opções;
* para **Editar com IA**, um diff em relação à sequência atual: `+` um passo adicionado,
  `-` um passo removido, `~` um passo cuja pausa ou opções mudaram, `~ Nome: "a"
  -> "b"` para um campo;
* **Deixado de fora**: um passo que o modelo propôs e que não é permitido (desconhecido,
  destrutivo, uma sequência dentro de uma sequência, uma pausa impossível), com o
  motivo; os passos válidos são mantidos;
* as **notas** do modelo, e um **atalho sugerido** que é apenas mostrado, com
  a indicação de a tecla já estar ocupada. Nunca é atribuído: use depois o botão
  **Atalho**.

```
+- Alteração proposta -------------------------------------- x -+
| + adicionado   - removido   ~ alterado                        |
| ~ Nome: "Diária" -> "Diária 2"                                |
| Passos (3)                                                    |
| + 1. Git: obter de todos os remotos                           |
|   2. Git: stage de todas as alterações                        |
| - Git: commit...                                              |
| ~ 3. Git: push: predefinições -> Pausa: Sempre                |
| Deixado de fora                                               |
| passo 4 `reset_hard`: os passos destrutivos não são permitidos|
| [ Aceitar ] [ Editar antes de guardar ] [ Rejeitar ] [ Voltar ]|
+---------------------------------------------------------------+
```

* **Aceitar** escreve-a com uma só gravação (uma sequência nova, ou a substituição da
  editada, que mantém o seu id, âmbito e tecla) e seleciona-a na lista.
* **Editar antes de guardar** faz o mesmo e abre os seus passos para poder ajustar
  as linhas (esta página guarda cada edição de imediato, por isso a sequência já
  está guardada).
* **Rejeitar** não escreve nada. **Voltar** regressa à descrição para enviar de novo.
* **Aceitar** não está disponível quando não há nada a aceitar: nenhum passo utilizável, um
  problema que a página assinalaria, ou uma edição que não altera nada.

Nada é escrito antes de **Aceitar**, e uma sequência nunca é executada a partir daqui.
Fechar o diálogo (a cruz, `Esc`, um clique fora) pergunta "Descartar alterações?"
enquanto há uma descrição escrita ou uma proposta aberta; fechar as Definições também pergunta.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Cópia de segurança"
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

As chaves que esta versão não conhece, e os passos que não conhece, são mantidos quando
a página guarda.

## Executar uma sequência

Uma sequência é uma lista nomeada de passos, cada um um comando da paleta (stage de tudo, escrever a mensagem de commit com a IA, commit, push...). Execute-a a partir da paleta (`Sequência: <nome>`), com a tecla que lhe deu em Definições > Teclado e rato (`[keybindings.sequence]` no `config.toml`), ou na linha de comandos (`gitpoulpi sequence run`).

Enquanto corre, uma faixa sobre o topo do Log mostra `<nome> k/n`, o passo, e o que aguarda. A execução também aparece no painel de tarefas como `Sequência <nome> k/n: <passo>`. Corre uma sequência de cada vez; iniciar uma segunda diz qual está a correr.

**Onde pára para perguntar.** Cada passo tem uma definição `pause` (`default`, `always`, `never`):

| Passo | Predefinida | `pause = "never"` |
|---|---|---|
| Mensagem de commit da IA | a mensagem cai no campo da mensagem; a faixa diz "Reveja a mensagem e depois Continuar" | segue logo |
| Commit | uma mensagem vazia pára sempre: o campo fica com o teclado, e o seu commit (ou Continuar) segue | não disponível quando não há mensagem para o commit |
| IA: compor commits | abre a revisão; Aplicar segue, fechá-la cancela | aplica a proposta tal como está |
| Criar worktree | o diálogo abre com o nome preenchido | cria-o a partir de `options.name` sem o diálogo |
| Push | quando `git.push_behavior = ask` abre o seletor "Push para..." | faz push para `options.remote` |
| Force push com lease, stage de ficheiros que parecem segredos | pergunta sempre | não permitido |

`pause = "always"` em qualquer passo mostra "Seguinte: <passo>" com **Continuar** e **Cancelar** antes de ele correr. Se o push é rejeitado porque a branch divergiu, abre o diálogo habitual e a faixa diz "Push recusado: escolha no diálogo"; fazer pull e depois push (ou um force push com lease) continua a execução, fechar o diálogo cancela-a.

**Continuar** é o botão da faixa, `Sequência: continuar` na paleta, ou a tecla associada a `sequence.continue`. **Cancelar** é a cruz de fechar da faixa, `Sequência: cancelar` (`sequence.cancel`), o painel de tarefas, ou `Esc` quando mais nada o quer (um diálogo aberto fecha primeiro, o que também cancela). Cancelar mantém os passos que já correram e di-lo: "<nome> cancelada após o passo k/n (<passo>); os passos anteriores ficam mantidos".

**Quando um passo falha** a execução pára aí: a linha de estado nomeia a sequência, o passo e o erro, a faixa fica vermelha com **Detalhes** (a entrada do Log de atividade, que lista as opções do passo) e uma cruz de fechar, e os passos anteriores ficam feitos.

**Worktrees.** Um passo de criar worktree com `open_tab = true` move a execução para o separador do novo worktree: os passos seguintes (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) atuam aí. Se outro separador está no ecrã, a faixa diz "à espera do separador <nome>" com **Ir para lá**. Fechar esse separador cancela a execução. Uma execução não sobrevive ao fecho do GitPoulpi.

### Um agente a propor uma execução

Com *Deixar agentes de IA proporem alterações para eu confirmar* ligado (`[mcp]
allow_ui_propose`), um agente pode pedir à sua janela que execute uma sequência
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Aparece um banner:
*✦ Proposto por <agente>: executar a sequência <nome>* com **Executar** e **Dispensar**.
Nada corre até premir Executar (clique nele, ou `Ctrl+N` e depois `Enter`);
Dispensar, `Esc` (quando mais nada o quer) ou *Agente: dispensar a vista pendente* da paleta
largam-no. A execução decorre então exatamente como se a tivesse iniciado,
com pausa onde a sequência o indica. A proposta é recusada quando a sequência
não existe, é inválida, ou é uma sequência de repositório em que não confiou.

## Desfazer uma execução

Cada passo que altera o repositório (stage, commit, tag, branch, stash, worktree...) deixa a sua **própria entrada de Desfazer normal**, marcada com a execução, por isso `Ctrl+Z` continua a desfazer um passo de cada vez e o [Histórico de desfazer](undo-redo.md#o-histórico-de-desfazer) mostra `⟳ <nome>` nelas. Para tirar a execução inteira há **Desfazer a sequência inteira**: o botão do Histórico de desfazer (`x`), a linha da paleta `Editar: desfazer a sequência inteira`, ou `gitpoulpi undo --sequence`. Desfaz as entradas da execução da mais recente para a mais antiga, cada uma com as suas verificações e perguntas (um commit que já foi enviado pergunta primeiro), pára na primeira recusa nomeando o passo, e deixa as mais antigas no diário. A mensagem da IA e um push não deixam entrada: um push nunca é desfeito. Só o diário do repositório atual é tocado; os passos feitos num worktree são desfeitos a partir desse repositório. O Refazer continua por entrada. Detalhes: [Execuções de sequências](undo-redo.md#execuções-de-sequências).
