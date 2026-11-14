[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · **Português (Portugal)** · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# Assistência de IA

O GitPoulpi nunca precisa de uma conta, e **a IA está desativada por
predefinição**. Quando a IA está ativada, pode escrever mensagens de commit e de
stash, explicar alterações, escrever descrições de pull requests, sugerir
resoluções de conflitos de merge e dividir as suas alterações em curso em vários
commits. O modelo é seu: uma chave de API que lhe pertence, um servidor local, ou
uma CLI de agente já instalada (como `claude -p`).

Cada controlo de IA mostra o `✦` roxo. Enquanto a IA está desativada:

- Nada é enviado para lado nenhum.
- Nenhum controlo de IA é mostrado: sem botão `✦`, sem comando `IA:` na paleta,
  sem linha de menu de IA e sem dica de tecla de IA. As teclas de IA não fazem
  nada.
- A página de IA do repositório mostra apenas "A IA está desativada (Definições > IA)".
- O servidor MCP não lista as ferramentas `ai_*`. Os comandos de CLI
  `gitpoulpi ai ...` recusam com a mesma mensagem.

**Ativar IA** tem efeito de imediato. Não precisa de reiniciar.

Há duas formas de usar a IA com o GitPoulpi:

| | IA integrada (esta página) | Um agente externo |
|---|---|---|
| O quê | O GitPoulpi pede a um modelo uma mensagem de commit, uma explicação, ... | O Claude Code, o Cursor, o Codex... conduzem o GitPoulpi como ferramenta |
| Configurar em | Definições > IA (`[ai]`) | a configuração MCP do agente, limites em `[mcp]` |
| Guia | abaixo | [MCP e agentes](mcp-and-agents.md) |

## Configurar a IA

1. `Ctrl+,` > **IA** > assinale **Ativar IA**.
2. Os perfis são a lista do separador **Perfis (ligações)**, à esquerda da página
   (empilhada por cima num terminal estreito): escolha um para o editar à direita (`Enter` passa para as
   suas linhas; **IA: editar perfil...** na paleta salta para a lista). **+ Adicionar**
   cria outro, **Duplicar** copia o selecionado (sem a sua chave) e **Remover**
   pede confirmação. Escolha o **Fornecedor** e preencha os seus campos:

   | Fornecedor | Campos | Predefinições |
   |---|---|---|
   | **Agente local** (sem chave, sem conta própria) | **Tipo de agente** (que agente é), **Comando** (o programa exato: no seu `PATH`, um alias ou script wrapper, ou um caminho completo), **Argumentos** que o tornam não interativo, e **Variáveis de ambiente** opcionais. O prompt vai para o seu stdin, a resposta é lida do stdout. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **Chave de API**, **Modelo** e **URL base** opcionais | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **Chave de API**, Modelo, URL base | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **Chave de API**, Modelo, URL base | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **Compatível com OpenAI** (Ollama, LM Studio, vLLM, um gateway de empresa) | **URL base** (`.../v1`), **Modelo**, chave opcional | `http://localhost:11434/v1` (Ollama; o LM Studio é `http://localhost:1234/v1`), `llama3` |

   **Predefinições** (pode sempre escrever outro valor):

   ![A lista Tipo de agente de um perfil de agente local, com o Claude Code, o OpenAI Codex CLI, o Gemini CLI, o opencode e as outras CLIs de agentes](../images/settings-ai-presets.png)

   - **Agente local > Tipo de agente** é uma lista das CLIs de agentes comuns,
     as mais usadas primeiro: Claude Code, OpenAI Codex CLI, Gemini CLI,
     opencode, Cursor Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose,
     Crush, Amp, ou *Outro (sem sugestões)*. O tipo é **separado do comando**:
     só escolhe que sugestões recebe. Escolher um preenche previamente o
     **Comando** (enquanto ainda for o predefinido do tipo anterior) e os
     **Argumentos** com a invocação sem interface (`claude -p`, `codex exec -`,
     `gemini -p ...`, ...). O **Comando** é texto livre: escreva
     `/opt/bin/work-claude`, um script wrapper ou o destino de um alias e o tipo,
     com as suas opções, lista de modelos e variáveis, mantém-se. Quando o
     **Tipo de agente** está vazio, o GitPoulpi infere-o a partir do comando.
   - **Opções dos argumentos** adiciona ou remove opções que servem a utilização
     pontual do GitPoulpi, cada uma com uma razão numa linha: sem ferramentas,
     sem sessão guardada, saída em texto simples, pouco esforço de raciocínio,
     modo só de leitura, sem pedidos de permissão, sem verificação de
     atualizações nem estatísticas de utilização. A lista difere por agente
     porque os agentes diferem (só alguns conseguem desligar as ferramentas a
     partir da linha de comandos; no opencode isso é uma variável de ambiente).
     **Modelo** lista os modelos desse agente e escreve a opção certa
     (`--model haiku` para o Claude Code, `-m gpt-6-luna` para o Codex,
     `--model provider/model` para o opencode); *(predefinição - sem opção)*
     remove-a. O Amp não tem opção de modelo. O `aider` lê o seu prompt de
     `/dev/stdin`, por isso só funciona em Linux e macOS.
   - **Variáveis de ambiente** são definidas apenas para o processo desse agente,
     nunca para o GitPoulpi nem para outros perfis. **Adicionar uma variável de
     ambiente** oferece as variáveis documentadas que importam para esse agente,
     cada uma com o que faz (desligar telemetria e verificações de atualizações,
     nível de esforço, pasta de configuração, chave de API, URL base ou anfitrião
     local, ...); também pode escrever `NAME` ou `NAME=value`. Cada variável tem
     depois a sua própria linha **Env NAME**: o valor aparece como um botão (prima-o
     para escrever outro; guarda ao sair do campo) e um botão **[x]** apaga a
     variável. Um valor pode ser `${env:NAME}` para pedir emprestada uma das variáveis do próprio GitPoulpi
     (útil para chaves de API, para não serem escritas em `config.toml`). Os
     valores que parecem segredos (nomes terminados em `_KEY`, `_TOKEN`,
     `_SECRET`, ... ou prefixos de chave conhecidos) aparecem como pontos e
     escrevem-se de raiz. Os valores nunca são registados em log.
   - **Fornecedores HTTP > Predefinição do fornecedor** preenche o URL base e um
     modelo para a OpenAI, a Anthropic, o OpenRouter, o Google Gemini, a xAI, a
     DeepSeek, a Mistral, a Groq, a Together, a Fireworks, a Cerebras, a
     Perplexity, a Moonshot, a Qwen (DashScope), a Zhipu (GLM) e os servidores
     locais Ollama, LM Studio, llama.cpp, vLLM, Jan e LocalAI. **Modelo** é uma
     lista dos modelos desse fornecedor, com texto livre permitido.

3. Prima **Testar ligação**: *Último teste* indica `ligado: o modelo respondeu`
   ou `falhou: <motivo>`.
4. Com vários perfis, escolha o **Perfil predefinido** em baixo; cada ação pode
   continuar a indicar o seu.

![O bloco Perfis da página de IA para um agente local: Tipo de agente (Claude Code), Comando, Argumentos, Modelo, duas linhas Env, Adicionar e Remover uma variável de ambiente, Testar ligação e Perfil predefinido](../images/settings-ai-agent.png)

As **chaves de API** introduzem-se com `Enter` na linha **Chave de API** e são
guardadas no credential helper do seu sistema (Git Credential Manager, libsecret
/ `pass`, Keychain) sob `ai:<profile id>`, nunca em `config.toml`. Sem um helper,
o GitPoulpi di-lo em vez de fingir que a guarda. *Definições > SSH e credenciais >
Esquecer todos os utilizadores e palavras-passe* remove-as.

O mesmo em `config.toml` (tudo o resto está na [referência de
configuração](config-reference.md#ai-ia-integrada-desativada-por-predefinição)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # livre: um alias, um script wrapper, um caminho absoluto
harness = "claude"        # que agente é; vazio = inferido de `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # só para o processo deste agente; ${env:NAME} pede emprestada a nossa
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Modo de execução: headless ou terminal virtual

Um perfil de agente local executa o seu agente de uma de duas maneiras,
escolhida em **Modo de execução** (mostrado para o Claude Code, o OpenAI Codex
CLI e o Gemini CLI):

- **Headless (não interativo)**, a predefinição: o modo de pedido único do
  agente (`claude -p`), prompt no stdin, resposta no stdout.
- **Interativo (terminal virtual)**: o GitPoulpi inicia o ecrã próprio do
  agente num terminal oculto, escreve o pedido como uma pessoa o faria, aguarda
  a resposta e fecha o agente. Alguns agentes faturam ou limitam o modo headless
  de forma diferente do interativo; o que se aplica a si, e o que os termos do
  fornecedor dizem sobre cada um, depende do fornecedor, por isso consulte os
  dele.

Cada modo mantém os seus próprios **Argumentos**, **Opções dos argumentos** e
**Modelo**: mudar de modo não perde nada, e o modo interativo só oferece as
opções que fazem sentido sem a opção headless. Mais duas linhas aplicam-se aos
dois modos:

- **Pasta de trabalho**: *Pasta dedicada* é uma pasta vazia que o GitPoulpi
  possui, por isso o agente não vê nenhum projeto e pede-lhe para confiar nela
  uma só vez. *Repositório atual* dá ao agente o contexto próprio do projeto (o
  seu `CLAUDE.md`, por exemplo), e ele pede confiança uma vez por repositório.
  *Predefinição* é a pasta dedicada no modo interativo e, em modo headless, a
  pasta em que o GitPoulpi é executado, como antes.
- **Tempo limite (segundos)**: quanto tempo o agente pode demorar antes de o
  GitPoulpi desistir e o parar. Vazio significa 120 segundos em headless e 180
  em interativo.

**As perguntas da primeira execução são suas para responder.** Um agente que
nunca correu numa pasta pergunta primeiro alguma coisa (confiar nesta pasta,
iniciar sessão, escolher um tema). O GitPoulpi nunca responde por si: o pedido
pára com uma mensagem, e **Abrir o agente...** (no perfil, ou *IA: abrir o
agente* na paleta de comandos) abre o agente uma vez, num terminal visível, na
pasta de trabalho que ele vai usar. Responda à pergunta, feche o agente e tente
de novo; a resposta fica memorizada pelo agente.

A resposta é lida da transcrição da sessão do agente no Claude Code (o seu texto
exato, apagada depois) e do ecrã entre duas linhas de marcação que o pedido
solicita, no Codex e no Gemini. No máximo duas execuções em terminal virtual
decorrem ao mesmo tempo, e premir `Esc` durante um pedido pára o agente.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # o conjunto headless, mantido
run_mode = "terminal"               # headless (predefinição) | terminal
terminal_args = ["--effort", "low"] # o conjunto interativo
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 ou ausente: 120 headless, 180 terminal
```

## Instruções

Texto que acrescenta ao que é dito ao modelo. O prompt integrado de cada ação
(que fixa o formato da resposta) vem sempre primeiro e não pode ser editado;
depois quatro níveis, do mais geral ao mais específico, cada um só quando não
está vazio:

1. **Instruções globais** (todas as ações). Os chips de *Inserção rápida* do editor
   inserem frases comuns: *Responder em `<idioma>`*, *Respostas concisas*,
   *Usar o imperativo*, *Tom técnico*, *Tom informal*.
2. **Instruções da área de trabalho** (todas as ações, todos os repositórios da
   [área de trabalho](workspaces.md#instruções-de-ia-de-uma-área-de-trabalho)): para todas as ações,
   e opcionalmente por ação. Editadas no formulário Editar da área de trabalho.
3. **Instruções do repositório** (todas as ações, só neste repositório), com
   *Combinar com o global*: **Somar ao global** (depois delas) ou **Substituir o
   global** (em vez delas).
4. **Instruções da ação** (*Geração de mensagem de commit*, *Explicar
   alterações*, *Geração de mensagem de stash*, *Pull request*, *Compositor de
   commit*, *Resolução de conflitos*, *Revisão de código*), cada bloco com os seus próprios chips no respetivo editor (por
   ex. *Formato conventional commits*, *Limite de 50 caracteres*) e a sua própria
   lista **Perfil** (`Predefinição (<nome>)` segue o perfil predefinido).

`Enter` numa linha *Instruções personalizadas* abre um editor de várias linhas
(`Ctrl+S` guarda, `Esc` cancela). Acima do texto, uma linha *Inserção rápida* tem os
chips dessa secção: clique num para inserir a sua frase no cursor. Com o teclado,
`Tab` passa para a linha, `Left`/`Right` escolhem um chip, `Enter` ou `Space` insere-o
e volta ao texto (`Tab` ou `Esc` voltam sem inserir). Um chip cuja frase já consta das
instruções fica esbatido e inseri-lo outra vez não faz nada. O texto é um editor
completo: um clique põe o cursor, um arrastamento ou `Shift`+setas selecionam,
`Ctrl+A` seleciona tudo, `Ctrl+C` / `Ctrl+X` copiam e cortam a seleção, `Ctrl`+setas
saltam palavras, e `Home` / `End`, `PageUp` / `PageDown` e a roda do rato percorrem
textos longos. A própria página mostra apenas uma pré-visualização de cada instrução.

![A página de IA: instruções globais, depois os blocos Geração de mensagem de commit e Explicar alterações com uma lista de perfil e instruções personalizadas](../images/settings-ai.png)

**Por repositório**: *Definições > Definições do repositório > IA* (com um
repositório aberto) contém as instruções do repositório e, por ação, um perfil e
instruções que se sobrepõem às globais. Um campo vazio usa a definição global
(mostrada a cinzento nele); **Voltar ao global** remove uma substituição. Guardado
no `.gitpoulpi/repo-state.toml` do repositório, e respeitado também pela CLI.

![A página de IA do repositório: Instruções do repositório com a lista Combinar com o global, depois um perfil e instruções para cada ação](../images/settings-repo-ai.png)

### Como se combinam os níveis

A ordem enviada é: prompt integrado, **global**, **da área de trabalho**, **do
repositório** e depois as instruções da própria ação. O modo de um nível decide o
que acontece aos que estão acima dele:

- **Somar** (o predefinido): o texto do nível é acrescentado depois dos mais gerais.
- **Substituir**: o texto do nível é usado *em vez de* todos os níveis mais gerais
  (o global, e o da área de trabalho quando o repositório substitui). Um nível sem
  texto nunca substitui nada.

As instruções da própria ação seguem a mesma ideia: as globais dessa ação, depois
as da área de trabalho para essa ação (somadas a elas, ou em vez delas), e uma
substituição própria do repositório para a ação vence ambas. A área de trabalho
usada é a ativa da janela quando contém o repositório, senão a primeira área de
trabalho que o contenha; na linha de comandos é a de `GITPOULPI_WORKSPACE`, senão a
primeira.

Para ver exatamente o que é enviado, nível a nível, execute
`gitpoulpi ai prompt --action <ação> --repo .` (`base`, `global`, `workspace`,
`repository`, `task` e o `system_prompt` final). Não chama nenhum modelo e
funciona com a IA desligada. Defina as instruções de uma área de trabalho a partir
de um script com
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (ou `--clear`).

### Variáveis do prompt

Qualquer texto de instruções (global, da área de trabalho, do repositório, por ação) pode conter
variáveis `${name}`. São substituídas pelo seu valor mesmo antes de o prompt ser
enviado (o texto guardado mantém o `${...}`). Um nome que não esteja na tabela é
enviado tal como está escrito e o editor avisa disso; escreva `$${` para um `${`
literal.

| Variável | Expande para | Exemplo |
|---|---|---|
| `${uilang}` | O idioma da interface, pelo seu nome em inglês | `French` |
| `${repo}` | O nome da pasta do repositório | `gitpoulpi` |
| `${branch}` | A branch atual | `main` |
| `${ref}` / `${commit}` | SHA curto do commit a que a ação diz respeito (*Explicar*), senão HEAD | `4c0443d` |
| `${commit_subject}` | Linha de assunto desse commit | `feat: add search` |
| `${date}` | Data local, ISO | `2026-10-03` |
| `${time}` | Hora local, `HH:MM` | `14:05` |
| `${datetime}` | Data e hora locais | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | O remoto predefinido (`origin`, senão o primeiro) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (outro anfitrião) ou `none` | `github` |
| `${files}` | Número de ficheiros no diff da ação | `3` |
| `${action}` | A ação em execução | `commit_message` |

No editor de instruções: escreva `${` e uma janela lista as variáveis
correspondentes (`Up`/`Down` escolhem, `Enter` ou `Tab` inserem, `Esc` fecha-a);
ou use a lista **Variáveis** em baixo à esquerda (`Ctrl+T`, ou *IA: inserir
variável de prompt* na paleta de comandos), que insere na posição do cursor.
Cada entrada tem uma descrição, mostrada na lista e na linha por cima dos botões
enquanto passa o rato ou a realça.

## O que pode fazer

| Ação | Na janela | CLI |
|---|---|---|
| **Mensagem de commit** | Faça stage das alterações, prima o `✦` à direita do campo Resumo, `Ctrl+G i`, ou na paleta `IA: gerar mensagem de commit`. O resumo e a descrição são preenchidos para os editar; nada é commitado. | `ai commit-message` (imprime), `commit --ai` (faz commit) |
| **Explicar alterações** | O `✦` no canto superior direito do painel das alterações em curso (`Ctrl+G w`: o commit selecionado, senão as alterações em curso), ou o menu de um commit *Explicar alterações (IA)*. Uma janela: as setas deslocam, `c` copia, `Esc` fecha. | `ai explain [rev] [--wip]` |
| **Mensagem de stash** | `Ctrl+G m` abre o diálogo do stash; o seu botão **Mensagem da IA** (`Ctrl+G` dentro do diálogo) pergunta ao modelo. `IA: gerar mensagem de stash` na paleta faz ambos. | `stash push --ai` |
| **Título e descrição de pull request** | *Gerar título e descrição* no formulário Criar pull request (os commits e o diff da branch são lidos localmente). | |
| **Resolução de conflitos de merge** | No editor de conflitos de merge: o botão `✦ Sugerir` da barra, o menu de clique com o botão direito de um conflito *✦ Sugerir uma resolução (IA)*, `Alt+I`, ou na paleta `IA: sugerir resolução de conflito de merge`. A sugestão substitui a região do *conflito atual* no Output como uma só edição (marcada como editada, um passo de desfazer); nada é guardado nem posto em stage até guardar. | `ai resolve-conflict <path> [--conflict N]` (imprime) |
| **Revisão de código** | Na [revisão de um pull ou merge request](review.md#-rascunho-de-revisão-da-ia): o botão `✦ Redigir revisão...`, `s`, ou na paleta `Revisão: ✦ redigir revisão...` (abre as opções da revisão, `Enter` executa). O modelo lê o diff, a descrição e os comentários existentes do pedido (e, por omissão, o `CLAUDE.md` e o `AGENTS.md` do projeto: ver [Contexto da revisão](#contexto-da-revisão)) e propõe comentários; aceita, edita ou dispensa cada um. Nada chega ao host antes de submeter a revisão. | `hosting review suggest <n>` (imprime JSON, não publica nada), `ai review-context` (as predefinições) |
| **Dividir um commit** | O menu de um commit, *Dividir commit (IA)...*, ou na paleta `IA: dividir o commit selecionado...`. O editor de divisão abre com o agrupamento do modelo já preenchido; veja abaixo. | `ai split-plan <rev>` (imprime) |
| **Script de reescrita do histórico** | No [painel de reescrita do histórico](history-rewrite.md#escrever-um-script-com-ia): o botão `✦ Escrever com IA...` do editor de scripts, ou na paleta `Reescrever: escrever um script com IA...`. Um diálogo mostra exatamente o que será enviado; o script chega ao editor, assinalado, e nunca é executado nem aplicado por si. | `ai rewrite-script "<instrução>" [--sample N]` (imprime) |
| **Regras de reescrita do histórico** | No [painel de reescrita do histórico](history-rewrite.md#criar-regras-com-ia): o botão `✦ Criar regras com IA...`, `Alt+I`, ou na paleta `Reescrever: criar regras com IA...`. Um diálogo de várias linhas mostra exatamente o que será enviado; as regras e as definições chegam ao painel e nada é pré-visualizado nem aplicado por si. | `ai rewrite-rules "<descrição>" [--context]` (imprime) |
| **Nome do worktree** | No [diálogo Criar worktree](worktrees.md#nomear-com-ia): o botão `✦ Nomear com IA` ou `Alt+A` abre uma caixa onde descreve a funcionalidade, o erro ou a tarefa; o modelo dá nome ao worktree e à sua branch segundo os nomes de branches do próprio repositório, e ambos ficam nos campos do diálogo para os editar. Só são enviados a sua descrição e os nomes das branches locais, por isso não tem diálogo "o que é enviado". | `ai worktree-name --describe "<tarefa>"` (imprime) |
| **Compositor de commits** | O menu da linha WIP *Compor commits (IA)...*, `Ctrl+G C`, na paleta `IA: compor commits a partir das alterações em curso...` | |

`Esc` cancela um pedido em curso (o processo de um agente local é terminado).

### Resolução de conflitos de merge

O modelo recebe o caminho e a linguagem do ficheiro, os blocos base, atual (A) e
recebido (B) do conflito com as etiquetas que o editor mostra, até 20 linhas de
texto já integrado antes e depois dele, e as suas instruções (globais, do
repositório e as de *Resolução de conflitos* em Definições > IA). Responde com o
texto resolvido desse único conflito num bloco de código; uma resposta com
marcadores de conflito ou conversa em vez de código é devolvida uma vez e depois
recusada. Um conflito com mais de 40 KB (`ai.limits.conflict_bytes`,
*Definições > IA > Limites*) fica retido por trás de um aviso que indica o seu
tamanho e o limite, com *Enviar mesmo assim* (só este pedido, sempre limpo pelas
verificações de privacidade), *Abrir definição* e *Cancelar*; o limite é uma
proteção do próprio GitPoulpi, por isso aumente-o quando o seu modelo tiver margem.
Os controlos ficam ocultos enquanto a IA está desativada, e uma sugestão nunca
guarda, faz stage nem resolve nada: reveja o Output, edite-o e guarde como
habitualmente.

![O formulário de commit depois do botão de faísca: um resumo e uma descrição escritos pelo modelo, prontos a editar](../images/ai-commit-message.png)

![A janela Explicar as alterações em curso: uma explicação e uma lista de coisas a verificar](../images/ai-explain.png)

### O compositor de commits

O modelo divide as alterações em curso em vários commits (cada ficheiro alterado
em exatamente um; um plano inválido é devolvido uma vez e depois recusado). Enquanto
houver ficheiros com conflitos nada é pedido: um diálogo lista-os e abre a vista de
resolução de conflitos, porque o compositor precisa de uma árvore sem conflitos. Revê
o plano antes de qualquer commit ser feito:

![A revisão do compositor: quatro commits propostos com os seus ficheiros, e os botões Repor, Cancelar e Criar 4 commits](../images/composer.png)

| Tecla | Rato | Faz |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | clicar numa linha | mover por commits e ficheiros |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | as setas, arrastar e largar | mover um commit; mover um ficheiro para outro commit |
| `Right` / `Left`, `*` | a seta da linha | expandir / recolher um commit, todos eles (`ui.composer_files` define o início) |
| `e` / `Enter` | o lápis | editar a mensagem |
| `d` / `Delete` | `drop` / `restaurar` | deixar um commit de fora (os seus ficheiros continuam sem commit) |
| `n` | `+ commit próprio` | mover um ficheiro para um commit só dele |
| `r` | `Repor` | voltar ao plano do modelo |
| `Ctrl+S` / `a` | `Criar N commits` | criá-los, por ordem |
| `Esc` | `Cancelar` | sair, nada feito |

Criar faz stage dos ficheiros de cada commit e faz commit deles um a um; a árvore
de trabalho nunca é tocada. Um ficheiro em stage só em parte é commitado por
inteiro. Se um passo falhar, pára e diz quantos commits foram feitos.

### Dividir um commit

O modelo lê as alterações do commit, numeradas como o [editor de divisão](log-and-history.md#dividir-um-commit)
as numera, e propõe vários commits, cada um com a sua mensagem; pode agrupar hunks e
linhas soltas. Uma resposta que deixe uma alteração de fora, atribua uma duas vezes
ou nomeie algo que não existe é recusada. Em qualquer dos casos o editor de divisão
abre: com a proposta já preenchida ou, se o modelo falhou, com o motivo numa nota,
para dividir à mão. Nada é dividido até premir **Dividir** (`Ctrl+S`); mova
alterações, mude o nome dos commits ou acrescente um se o agrupamento não estiver
certo. O perfil e as instruções são os de *Dividir um commit* em Definições > IA.

### Um script de reescrita do histórico

Descreve o que a reescrita deve fazer ("mover para a manhã seguinte todos os commits
feitos de noite", "substituir o meu e-mail antigo pelo novo, só antes de 2024") e o modelo
escreve um [script Lua](history-rewrite.md#scripts-lua) para a regra Script, com o cabeçalho,
`rewrite_commit(c, ctx)` e as funções auxiliares da sandbox. O daemon compila a resposta;
quando não compila, o erro volta ao modelo uma vez. O que sobra é mostrado no
editor: leia-o, teste-o e depois pré-visualize. Os pormenores e o diálogo estão em
[Reescrita do histórico](history-rewrite.md#escrever-um-script-com-ia). O perfil e as instruções são os de
*Script de reescrita do histórico* em Definições > IA.

### Regras de reescrita do histórico

Descreve por palavras suas como o histórico deve ser reescrito ("substituir o meu e-mail
antigo a@b.org por c@d.org", "remover secrets.txt de todo o histórico") e o modelo propõe um
âmbito, algumas definições e uma lista de [regras de reescrita](history-rewrite.md#criar-regras-com-ia)
no vocabulário do próprio painel. **Só prepara**: a resposta preenche o painel, nada é
pré-visualizado nem aplicado, e uma entrada que não se pode usar é listada com o motivo, enquanto
as válidas são mantidas. Nunca é criada assim uma regra Script. Opcionalmente, o âmbito, os nomes
das branches e das tags e os autores dos commits mais recentes vão junto (uma caixa, desmarcada
por predefinição, mostrada no painel "o que é enviado"). O perfil e as instruções são os de
*Regras de reescrita do histórico* em Definições > IA.

## Para onde vai a chave

Uma chave de API (OpenAI, Anthropic, OpenRouter, um servidor compatível com OpenAI) fica associada ao endereço para o qual foi guardada: a origem do URL base do perfil, registada em `trust.toml` na pasta de dados privada, só do utilizador. Se o URL base passar mais tarde a apontar para outro sítio, o pedido é recusado antes de ser enviado seja o que for e a chave fica onde está. Execute `gitpoulpi ai trust <perfil> --accept-origin <url>` (ou aceite-o na aplicação) para mover a chave para o novo endereço de propósito.

- O `http://` simples é permitido neste computador (Ollama, LM Studio em `localhost`); em qualquer outro lado precisa de `gitpoulpi ai trust <perfil> --allow-insecure-http on`.
- Um servidor auto-alojado cujo certificado não se encadeia a uma autoridade conhecida só é aceite para a impressão digital da chave que fixar com `--accept-certificate <sha256>`; o handshake TLS continua a provar que o servidor possui essa chave.
- Um perfil que já tinha uma chave antes de esta verificação existir é registado em silêncio, a partir do seu URL base atual, na primeira vez que é usado.
- Um agente local (`claude -p` e semelhantes) não tem chave nem HTTP, pelo que não há nada a associar.

## Contexto da revisão

Um rascunho de revisão de código (`✦`, ou `hosting review suggest`) lê o diff, o título, a descrição e os
comentários existentes do pedido. Pode também receber as convenções do próprio projeto, num bloco
**Convenções do projeto** colocado antes do diff, e um agente local pode trabalhar dentro do repositório. As predefinições são:

| Definição | Predefinição | Significado |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: decide a *Pasta de trabalho* do perfil de IA. `yes`: um agente local é executado no repositório para a revisão, diga o que disser o perfil. `no`: não aí (um perfil definido como *Repositório* usa em vez disso a pasta dedicada). Os perfis HTTP não têm pasta de trabalho, por isso não lhes altera nada. |
| `include_claude_md` | ligado | Dar ao modelo o `CLAUDE.md` do repositório. |
| `include_agents_md` | ligado | Dar ao modelo o `AGENTS.md` do repositório. |
| `extra_file` | nenhum | Mais um ficheiro: um caminho relativo à raiz do repositório, um caminho absoluto ou uma ligação. |

Estão em camadas como as instruções, ganhando a mais específica campo a campo: a tabela global `[ai.review]`,
depois os valores próprios do repositório (um campo por definir herda), depois as escolhas de uma execução. Um
repositório não clonado guarda os seus valores por `host/project` em `remote-reviews.toml`, junto dos
seus rascunhos de revisão.

Um repositório pode também desligar o ficheiro extra global: o seu `no_extra_file = true` (a caixa **Nenhum ficheiro
extra** em *Definições do repositório > IA*, no diálogo ✦ Opções da revisão e a opção `--no-extra-file` de
`ai review-context`) significa «nenhum ficheiro extra para este repositório», diga o que disser `[ai.review]`. Um campo
**Ficheiro extra** **vazio** não é a mesma coisa: herda o ficheiro global. Um ficheiro do repositório e o interruptor
excluem-se mutuamente (definir um limpa o outro), e o interruptor prevalece se um ficheiro editado à mão tiver os dois.
Um `extra_file = ""` mais antigo de um repositório continua a significar nenhum.

De onde vêm os ficheiros:

- O `CLAUDE.md`, o `AGENTS.md` e um ficheiro extra relativo são lidos da **raiz** do repositório, na
  branch a que o pedido se destina, nunca da árvore de trabalho (que pode estar noutra branch ou ter alterações
  locais). Com um clone, a partir do commit de base do pedido; sem ele, da API do host.
- Um caminho absoluto é lido desta máquina.
- Uma ligação para um ficheiro de um code host (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea e Forgejo `.../src/branch/<branch>/<path>`, ou uma ligação raw) é lida através do perfil de integração
  cujo host corresponde, pelo que um repositório privado funciona com o token que já indicou. Uma ligação que nenhum
  perfil reconhece, ou qualquer outro endereço, é obtida de forma anónima. Um nome de branch com uma barra não pode ser
  lido a partir de uma ligação: use um caminho, ou uma ligação numa branch sem barra.
- Um `CLAUDE.md` ou `AGENTS.md` em falta é ignorado em silêncio; um ficheiro extra em falta é assinalado
  (`context.problems` no JSON da CLI).
- Cada ficheiro é cortado aos **8 KB** e os ficheiros no seu conjunto aos **16 KB**, com uma nota visível
  `[CLAUDE.md cut: showing ... of ... bytes]` no que o modelo lê.

Quando um agente local é executado no repositório, o prompt diz-lhe que os ficheiros em checkout podem não corresponder ao
head do pedido e que o diff é a referência.

Na vista de revisão, `s` abre o diálogo **✦ Opções da revisão** com estes valores preenchidos: os do repositório sobre
os globais. Altere-os só para esta execução, ou marque *Guardar como predefinições deste repositório* para os manter (ver
[Opções da revisão](review.md#opções-da-revisão)). As predefinições também se editam em *Definições > IA* (o bloco *Revisão de código*,
quatro linhas) e por repositório em *Definições do repositório > IA*, onde cada linha pode *Usar o global*
e um botão *Voltar ao global* elimina os valores próprios do repositório; a pesquisa das Definições encontra-as (`Incluir
AGENTS.md`, `Ficheiro extra`). O que não pôde ser dado ao modelo (um ficheiro extra em falta) ou foi cortado para caber é
mostrado no cartão do rascunho.

Na linha de comandos:

```sh
gitpoulpi ai review-context                      # as predefinições globais e o que uma revisão recebe
gitpoulpi ai review-context --agents-md off      # guardar uma predefinição global
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # voltar a seguir as predefinições globais
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` não precisa de modelo e funciona com a IA desativada. `ai prompt --action code_review` imprime o
contexto efetivo como `review_context`. As mesmas quatro definições são `ai.review.*` em
`config.toml` ([referência](config-reference.md#ai-ia-integrada-desativada-por-predefinição)).

## Limites de entrada

Cada entrada que o GitPoulpi põe num pedido tem um limite de tamanho, uma definição
em `[ai.limits]` (*Definições > IA > Limites*). Os limites são uma proteção do próprio
GitPoulpi (a janela de contexto difere de modelo para modelo): aumente-os quando o seu
modelo tiver margem, reduza-os para manter os pedidos pequenos.

| Definição (`ai.limits.`) | Predefinição | O que limita |
|---|---|---|
| `conflict_bytes` | 40000 | Os blocos base, atual e recebido de um conflito de merge |
| `diff_bytes` | 60000 | O diff de uma mensagem de commit, de uma explicação, de um nome de stash, de uma descrição de pull request, de um plano de commits ou de uma divisão; o diff completo de uma revisão de código |
| `review_file_bytes` | 12000 | O diff de um ficheiro numa revisão de código |
| `review_text_bytes` | 4000 | A descrição do pull request revisto |
| `review_threads_bytes` | 6000 | Os comentários de revisão que já existem |
| `review_context_file_bytes` | 8192 | Um ficheiro de contexto do projeto numa revisão (`CLAUDE.md`, `AGENTS.md`, o ficheiro adicional) |
| `review_context_bytes` | 16384 | Todos os ficheiros de contexto de uma revisão em conjunto |

**Nada é cortado em silêncio.** Quando uma entrada excede o seu limite, o pedido
fica à espera e *Acima do limite da IA* lista o que excede: que parte (o diff, um
ficheiro, a descrição, um ficheiro de contexto), o seu tamanho, o limite e a definição.
Uma revisão lista todas as partes excedidas num só diálogo. Os botões, cada um com a
sua tecla:

- *Enviar tudo* (`T`): só este pedido, nada é cortado. Passa na mesma pelas
  verificações de privacidade (segredos substituídos, ficheiros sensíveis de fora) e,
  quando está ativado, pelo painel *O que é enviado*, que vem a seguir.
- *Enviar truncado* (`R`): o que excede o limite é cortado, como sempre foi; cada
  linha do diálogo diz o que fica de fora (o fim do diff, os últimos hunks, os
  ficheiros que não cabem).
- *Abrir definição* (`A`): cancela o pedido e abre *Definições > IA > Limites* no
  primeiro limite listado. O foco começa aqui: Enter nunca envia.
- *Cancelar* (`C`, `Esc`, a cruz ou um clique fora): não envia nada.

Um conflito é a exceção: nunca é cortado. *Enviar mesmo assim* envia-o inteiro uma vez.

Nada inicia um pedido de IA por si só, por isso não há execuções em segundo plano que
possam perguntar: cada pedido é um que iniciou. Uma execução que não pode perguntar (a
CLI e o MCP) trunca como antes e di-lo; veja abaixo.

**Na linha de comandos**, nada pode responder a um diálogo, por isso uma entrada acima
do limite é truncada como antes e a resposta diz o que foi cortado:
`truncated: [{part, detail?, bytes, limit, setting}]` (presente só quando algo foi
cortado; `part` é um de `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file`, `context_total`). `--send-all`
(alias `--no-truncate`) envia tudo inteiro: em `ai` (todos os seus comandos, incluindo
`ai resolve-conflict`, onde um conflito acima do limite é de outro modo recusado), em
`commit --ai`, em `stash push --ai` e em `hosting review suggest`. As ferramentas MCP
têm o mesmo argumento `send_all` e o mesmo campo `truncated`.

## Privacidade

- Nada é enviado até premir um botão ou executar um comando. Sem IA em segundo
  plano, sem telemetria, sem conta GitPoulpi.
- O que é enviado: o prompt (o integrado, depois as suas instruções) e o diff
  sobre o qual perguntou; para um pull request, também os assuntos dos commits da
  branch. Um diff com mais de 60 KB (`ai.limits.diff_bytes`) nunca é cortado em
  silêncio: veja [Limites de entrada](#limites-de-entrada).
- Um pedido de script de reescrita do histórico envia a sua instrução e o contrato fixo
  do script (a API da sandbox de scripts). O autor, a data e o assunto de até 20 commits só
  seguem junto **se** marcar *Enviar uma amostra* no diálogo (desativado por predefinição; a
  CLI precisa de `--sample N`). O conteúdo dos ficheiros e os valores secretos nunca são enviados.
- Vai apenas para o perfil escolhido para essa ação: esse anfitrião de API, ou o
  stdin do comando do seu agente local (que depois faz o que esse agente fizer).
- As chaves de API são usadas pelo próprio processo `gitpoulpi` e nunca chegam ao
  daemon, aos outros clientes da janela, nem a um cliente MCP.
- O texto gerado nunca é commitado, enviado nem aplicado por si só.

### O que é retirado de um pedido

Todos os pedidos, qualquer que seja a ação que os envia e o fornecedor que os recebe (uma
API alojada ou um agente local), passam primeiro pelo mesmo passo de limpeza:

- **Os segredos são ocultados** (`[ai.privacy] redact_secrets`, ativo por predefinição). Um token
  com um prefixo conhecido, um bloco de chave privada, um JWT, uma palavra-passe num URL e uma
  linha `password = valor` cujo valor pareça aleatório são substituídos por
  `[REDACTED:<regra>]`. Não é exaustivo: encontra as formas comuns (o mesmo
  analisador de `rewrite scan-secrets`).
- **Os ficheiros sensíveis são excluídos** (`[ai.privacy] exclude_sensitive_files`, ativo por
  predefinição). O diff de um ficheiro que corresponda a `[privacy] sensitive_patterns` (`.env`
  e `.env.*` exceto os ficheiros `.example`, `.sample` e `.template`, `*.pem`,
  `*.key`, chaves privadas SSH, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`,
  `*.cer`, `*.der`, `*.kdbx`) não faz parte do pedido. Uma peça de divisão ou uma lista de
  ficheiros do compositor que nomeie um ficheiro assim é cortada da mesma forma, pelo que um plano
  para ele pode ser recusado por incompleto: coloque ou divida esse ficheiro você mesmo.

Desativar qualquer um deles é uma escolha, e as Definições > IA mostram uma linha vermelha por baixo
enquanto estiver desativado.

**O que é enviado, antes de ser enviado.** Antes de um pedido sair, a TUI mostra o diálogo
**O que é enviado**: as instruções e o pedido depois da limpeza (o mesmo painel dos diálogos de
reescrita do histórico, com deslocação, `Alt+W` mostra-o ou oculta-o), quantos segredos foram
substituídos e que ficheiros foram excluídos (uma regra e uma pré-visualização mascarada
como `ghp_****abcd`, nunca um valor), uma caixa **Não mostrar de novo para
<ação>**, e **Enviar** / **Cancelar** (`Esc`, a cruz de fechar e um clique
fora cancelam; nada sai antes de Enviar). A caixa escreve
`[ai.sent_preview] <ação> = false`; Definições > IA > *Mostrar o que é enviado
antes de...* tem um interruptor por ação e **Mostrar tudo de novo**. Os dois diálogos de
reescrita do histórico já mostram o seu pedido, por isso não têm interruptor.

**Na linha de comandos**, cada resposta `ai ...` (e `hosting review suggest`)
traz `privacy: {redactions: N, excluded_files: [...]}`, e `--print-sent`
mostra o texto do sistema e do utilizador já limpo, como `{sent: [{system, user}], privacy,
dry_run: true}`, sem chamar o modelo (precisa de `ai.enabled = true`, sem chave
e sem rede).

**Um limite.** Um agente local cuja pasta de trabalho é o repositório
(`workdir = repository`, ou *Executar o agente no repositório* numa revisão) pode
ler ele próprio os ficheiros: a limpeza cobre o que o GitPoulpi lhe envia, não o que
ele abre por si. Use a pasta de trabalho `dedicated` para o manter fora do
repositório.

## Resolução de problemas

| Sintoma | Solução |
|---|---|
| Nenhum `✦` em lado nenhum | A IA está desativada: *Definições > IA > Ativar IA*. |
| O `✦` está cinzento | Nada está em stage (mensagem de commit). |
| *Testar ligação*: comando não encontrado | O **Comando** do agente local não está no `PATH` que o GitPoulpi vê: inicie o GitPoulpi onde `claude --version` funcione, ou indique o caminho completo. |
| Um agente local bloqueia ou mostra um ecrã interativo | No modo de execução *Headless*, adicione a sua opção sem interface (`-p` no Claude Code) aos **Argumentos**, ou escolha de novo o seu **Tipo de agente** para restaurar a invocação sem interface. O ecrã interativo é para isso que serve o modo de execução *Interativo (terminal virtual)*. |
| "Use "Open the agent"..." | O agente fez uma pergunta da primeira execução (confiar na pasta, iniciar sessão). Use **Abrir o agente...** no perfil, responda, feche o agente e tente de novo. |
| Um pedido em terminal virtual excede o tempo limite | A mensagem termina com as últimas linhas do ecrã do agente: um pedido de permissão, um início de sessão ou uma pergunta de atualização aparece aí. Responda com **Abrir o agente...**, ou aumente o **Tempo limite**. |
| `falhou:` 401 | Chave errada, ou uma chave de outro fornecedor: volte a introduzi-la. |
| `falhou:` 404 | O **Modelo** não existe lá: limpe-o para usar o predefinido. |
| `falhou:` erro de ligação (Ollama, LM Studio) | O servidor não está a correr, ou o **URL base** tem a porta errada ou falta-lhe `/v1`. |
| A chave mostra `(não definida)` depois de a escrever | Sem credential helper: configure um (`git config --global credential.helper ...`) e introduza-a outra vez. |
| *Acima do limite da IA* | Uma entrada do pedido excede um limite de `[ai.limits]`: escolha *Enviar tudo*, *Enviar truncado*, *Abrir definição* ou *Cancelar* (veja [Limites de entrada](#limites-de-entrada)). |
| «Conflito demasiado grande para enviar» | O conflito excede `ai.limits.conflict_bytes` (40000 por predefinição). *Enviar mesmo assim* envia-o uma vez; *Abrir definição* vai ao limite em Definições > IA > Limites. |
| Idioma ou estilo errado | Adicione instruções; os chips de Inserção rápida do editor tornam isso rápido. |
| O compositor diz que o plano é inválido | O modelo saltou ou repetiu um ficheiro: tente de novo ou use um modelo mais forte. |

## Ficheiros de configuração antigos

Um `config.toml` antigo com `[[ai.presets]]` e `[ai.defaults]` continua a carregar:

- A predefinição de cada ação passa a ser o seu `[ai.actions.<action>]`
  (`system_prompt` passa a `instructions`).
- Cada predefinição antiga é mantida em `[[ai.legacy_presets]]`. As Definições >
  IA mostram-nas em *Predefinições antigas (de antes do redesenho)*, com *Eliminar
  ficheiro*.
- O antigo `[preferences.ai_presets]` de um repositório passa a ser uma
  substituição do repositório.

O prompt integrado vem agora sempre primeiro. Se um prompt antigo copiou um prompt
integrado, remova a parte copiada e mantenha apenas o que quer acrescentar.
