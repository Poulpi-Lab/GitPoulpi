[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · [Deutsch](../de/merge-conflicts.md) · [Español](../es/merge-conflicts.md) · [Français](../fr/merge-conflicts.md) · [Italiano](../it/merge-conflicts.md) · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/merge-conflicts.md) · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · [简体中文](../zh-CN/merge-conflicts.md)

# Resolvendo conflitos de merge

Quando um merge, cherry-pick, revert ou rebase para em conflitos, o Log mostra uma
aviso (*Merge em andamento · feature em main · 2 arquivos em conflito*) e os
arquivos em conflito aparecem no painel Status com um `!` vermelho. Abra um no
**editor de merge**: A (o lado atual) e B (o lado recebido) lado a lado, com o
**Resultado**, o arquivo que você está escrevendo, embaixo. Esta página é a versão
curta; a lista completa de teclas está em [Atalhos de teclado](keybindings.md).

## Abrindo

- Enter, ou um clique duplo, em uma linha `!` do painel Status; o menu de botão
  direito dela começa com **Abrir no editor de merge**.
- O botão **Resolver conflitos...** do aviso (ele abre o primeiro arquivo em
  conflito).
- A visualização de Diff de um arquivo em conflito: o botão **Editor de merge**
  (`Shift+E`).
- A paleta: **Conflitos: abrir o editor de merge**.

Nada abre sozinho: uma operação parada só mostra o aviso e as linhas `!`.

![O Log de um repositório parado em um merge conflitante: o aviso Merge em andamento com Resolver conflitos e Abortar merge, e dois arquivos em conflito marcados com um ! vermelho](../images/conflict-banner.png)

O editor ocupa toda a largura da janela: a barra lateral Refs e o
painel da direita ficam ocultos enquanto ele está aberto e voltam quando você o
fecha (`F9` os mostra ou oculta nesse meio-tempo). A e B ficam lado a lado sempre
que o editor tem pelo menos `merge.side_by_side_min_width` colunas de largura (100
por padrão, Configurações > Diff e editor; `0` significa sempre); mais estreito,
você vê um lado por vez atrás de abas. Linhas longas rolam para o lado, A e B
juntos (`Shift+←` / `Shift+→`, Shift+roda ou uma roda lateral), e nunca são
quebradas.

![O editor de merge: A (atual) e B (recebido) lado a lado sobre o Resultado, conflito 1 pego de A e conflito 2 de B, conflito 3 ainda por escolher](../images/merge-editor.png)

## Escolhendo

Cada conflito é um bloco emoldurado em A e em B, com uma caixa de seleção no
cabeçalho e um `+` / `✓` na frente de cada linha.

| Para... | Tecla | Mouse |
|---|---|---|
| aceitar a linha sob o cursor (lado em foco) | `Space` | clicar na linha |
| aceitar o trecho inteiro do lado em foco | `Enter` | clicar no `[ ]` do cabeçalho dele |
| aceitar só A / só B neste conflito | `a` / `b` | botão direito no trecho |
| aceitar os dois, A depois B / B depois A | `c` / `Shift+C` | botão direito no trecho |
| não aceitar nada | `x` | botão direito no trecho |
| trocar a ordem de dois lados escolhidos | `s` | clicar no selo `1º` / `2º` |
| aceitar tudo de A / tudo de B (todos os conflitos) | `Shift+A` / `Shift+B` | barra de ferramentas |
| desfazer a escolha deste conflito (volta a não resolvido) | `u` em A / B, `Alt+U` em qualquer lugar | botão direito no trecho, **Desfazer a escolha** |
| redefinir o arquivo inteiro para seus conflitos originais (pergunta antes quando você digitou no Resultado; `Ctrl+Z` o traz de volta) | `Shift+U` em A / B, `Alt+R` em qualquer lugar | **Redefinir** ao lado de Aceitar tudo |
| próximo / anterior conflito | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` em A / B) | `▼` / `▲`, ou clicar em uma marca em uma barra de rolagem |
| um número repete um movimento | `3j`, `2k`, `2n`, `3N` (em A / B) | |
| próximo / anterior arquivo em conflito | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` no cabeçalho |
| mostrar ou ocultar o painel Base (ancestral comum) | `Alt+D` (`d` em A / B) | `Base` |
| mostrar o arquivo inteiro em A e B (**Arquivo todo**) | `Alt+F` (`f` em A / B) | `Arquivo todo` |
| dar mais largura a A ou B / voltar a larguras iguais | `Alt+Right` / `Alt+Left`, `Alt+0` | arrastar o divisor; clique duplo nele |
| rolar A / B / Base para o lado | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+roda, roda lateral |
| mostrar ou ocultar os painéis Refs e Status | `F9` | |
| focar A, B, o Resultado | `F6` / `Shift+F6` | clicar |
| pular para um painel, no conflito em que você está | `Alt+1` A, `Alt+2` B, `Alt+3` Resultado, `Alt+4` Base; ou `Ctrl+N` e depois `1` / `2` / `3` | |
| ir para o painel vizinho (A, Base, B, Resultado), mesmo conflito | `h` / `l` ou `←` / `→` em A / B | |
| alternar a altura do Resultado | `z` | arrastar o título dele |
| desfazer / refazer (escolhas e digitação, uma única pilha) | `Ctrl+Z` / `Ctrl+Y` | |
| gravar o arquivo, mantendo-o em conflito | `Alt+S` | **Salvar** |
| gravar o arquivo e marcá-lo como resolvido | `Ctrl+S` | **Salvar e marcar resolvido** |
| fechar | `Esc` | `✕` |

Toda barra de rolagem (A / B / Base, e o Resultado) leva uma marca por conflito:
laranja enquanto ainda precisa de uma escolha, verde quando tratado, azul para o
atual. Um clique em uma marca pula para esse conflito (uma marca sob o polegar da
barra de rolagem já está na tela, então ali o clique arrasta o polegar).

**Arquivo todo** mostra o arquivo inteiro em A e B com toda diferença colorida, não
só os conflitos:

![Modo Arquivo todo: o arquivo inteiro em A e B, com as alterações coloridas](../images/merge-editor-full-file.png)

O Resultado é um editor de verdade: clique nele e digite, selecione, recorte e cole,
busque com `Ctrl+F`. Uma escolha em um conflito que você editou à mão substitui sua
edição (`Ctrl+Z` a traz de volta). Salvar com um conflito sem nada escolhido, ou
com uma linha que ainda parece um marcador de conflito, pergunta antes.

### ✦ Sugerir uma resolução (IA)

Com a IA ativada (Configurações > IA; veja [Assistência de IA](ai.md)) a barra de
ferramentas tem um botão **✦ Sugerir**, o menu de botão direito de um conflito tem
*✦ Sugerir resolução (IA)*, e `Alt+I` e a paleta (`IA: sugerir resolução de
conflito de merge`) fazem o mesmo. Ele atua só no conflito atual e só roda quando
você pede. A resposta substitui a parte desse conflito no Resultado como uma única
edição, marcada como editada (`✎`), então `Ctrl+Z` devolve o conflito como era.
Nunca é salva nem colocada em stage por você. Enquanto roda, a linha de dicas mostra
o tempo gasto; `Esc` cancela, e uma falha também aparece ali. Com a IA desativada,
nenhum desses controles existe.

Depois de **Salvar e marcar resolvido**, o arquivo fica em stage e a lista de
arquivos em conflito volta; abra o próximo arquivo você mesmo.
`Ctrl+Z` (ou o botão Desfazer) desfaz o último “marcar como resolvido”. Quando não
sobra nenhum arquivo, o aviso oferece **Commit do merge** (ou **Continuar** para um
cherry-pick, um revert ou um rebase) e **Abortar merge**, que pergunta antes: ele
devolve HEAD, o índice e a árvore de trabalho a onde estavam.

Um merge cujo resultado é igual a HEAD (você pegou o nosso lado em todo lugar) não
tem nada em stage nem arquivo alterado, mas ainda pode ser concluído: **Commit do
merge** abre o prompt de commit com a mensagem do merge, e o commit tem os dois
pais. Do mesmo modo, o botão Commit do compositor fica habilitado sem nada em stage
enquanto um merge está parado.

Um cherry-pick ou revert vazio (a alteração já está aplicada) não pode ser
commitado: o git o recusa, e **Continuar** diz isso e aponta para **Pular**.
**Pular** (botão do aviso, entrada da paleta *Git: pular cherry-pick ou revert*, CLI
`conflicts skip`) pergunta antes e depois encerra a operação sem nada registrado
para ela; HEAD, o índice e a árvore de trabalho voltam a HEAD. Pular não pode ser
desfeito (nada se perde que não tenha sido abandonado de propósito); um merge não
tem nada para pular.

## Conflitos que não são texto

Um arquivo binário, um ponteiro do Git LFS, um arquivo com mais de 8 MiB, um arquivo
excluído de um lado, um resto de renomeação, um link simbólico, uma mudança de modo
ou um submódulo não têm linhas para escolher. O editor mostra um **cartão** com o
que cada lado contém e um botão por opção; escolher resolve e coloca o arquivo em
stage na hora.

| Cartão | Opções |
|---|---|
| Arquivo binário, ponteiro LFS, grande demais | **Manter A** (`a`), **Aceitar B** (`b`), **Aceitar Base** (`o`, quando há uma), **Abrir na ferramenta de merge** (`m`) |
| Excluído em um lado, alterado no outro | **Manter o arquivo** (`k`), **Aceitar a exclusão** (`Shift+D`) |
| Resto de renomeação (o caminho existe só em um lado) | **Manter o arquivo** (`k`), **Excluir o arquivo** (`Shift+D`) |
| Link simbólico, submódulo, modo do arquivo | **Manter A**, **Aceitar B**, **Aceitar Base** |

Quando o arquivo é uma imagem (PNG, JPEG, GIF, WebP, BMP, ICO), as caixas também
mostram as imagens de A, B e Base, ajustadas e somente leitura, em uma janela larga
o bastante para elas; veja [Imagens](image-view.md#no-editor-de-merge).

![O cartão de um conflito binário: o que A, B e a base contêm, e os botões Manter A, Aceitar B, Aceitar Base e Abrir na ferramenta de merge](../images/merge-editor-card.png)

`Esc` fecha o cartão sem escolher. A ferramenta de merge externa é
`general.external_merge_tool` em Configurações > Diff e editor (`{base}`, `{local}` = A,
`{remote}` = B, `{merged}`; vazio significa `git mergetool`); o arquivo vai para
stage quando a ferramenta termina com sucesso.

## Prevenção de conflitos

O GitPoulpi pode avisar você *antes* do merge. Quando a branch em que você está teria conflito com a branch de onde
ela partiu (a **base**), uma faixa aparece na linha de borda superior do Log: **Conflito com origin/main em 2
arquivos**, com um botão **Mostrar arquivos**. Ela ocupa a próxima linha de borda livre depois dos avisos de
operações em andamento, então, enquanto um deles aparece, pode esperar por espaço. Nada é mesclado: o daemon faz um
merge em memória dos dois commits (libgit2, ou `git merge-tree --write-tree` com o git 2.38 ou mais novo; um git mais
antigo desativa o recurso com uma linha no registro), então seus arquivos, o índice e as refs não são tocados.

- **A base** é a branch de destino do pull request da branch quando o GitPoulpi já a conhece (o cache que a seção
  Pull requests mantém; nenhuma chamada de rede é feita para isso), senão a branch, entre os padrões de **Branches
  base** (Configurações > Prevenção de conflitos), com menos commits desde a bifurcação. Um empate vai para a branch
  cuja ponta está mais perto da bifurcação, depois para uma branch de rastreamento remoto em vez de uma local, depois
  para o nome. A própria branch e a sua cópia remota nunca são a base.
- **O popup** (clique na faixa, ou use **Conflitos: mostrar conflitos previstos** na paleta, que funciona pelo
  teclado) lista os arquivos e tem **Abrir a prévia do merge** e **Rebase sobre a base**. São as próprias linhas do
  menu da branch base (*Merge de <base> em <branch>*, *Rebase de <branch> sobre <base>*) executadas por você: com
  registro no histórico de desfazer, então `Ctrl+Z` as desfaz, e um merge que dá conflito para no editor de merge como
  qualquer outro. Esquerda e Direita movem entre os botões, Enter pressiona um, Esc fecha.
- **Quando roda**: em segundo plano, cerca de 2 segundos depois que o repositório sossega (um commit, um checkout, um
  fetch que moveu uma branch, um rebase), uma vez por par `(HEAD, base)`. A faixa some assim que o HEAD se move e
  volta, ou não, com a nova resposta. Um HEAD desanexado não é verificado.
- **Listas de branches base**: de onde vêm as Branches base de um repositório é uma escolha na página **Prevenção de
  conflitos** dele. **Padrão** segue o padrão global (o `**` integrado, todas as branches, até você escolher outra lista em
  Configurações > Prevenção de conflitos); **Lista: <nome>** segue uma das suas listas com nome; **Lista própria** é uma
  lista só dele. As listas são compartilhadas: edite uma e todo repositório que a segue usa a mudança na próxima verificação,
  sem reiniciar. **Salvar como lista...** transforma os padrões próprios de um repositório em uma lista com nome, **Voltar ao padrão** descarta a
  escolha do repositório. Excluir uma lista que um repositório segue faz com que ele use o padrão.
- **Desativar**: **Ativar a detecção proativa de conflitos** em Configurações > Prevenção de conflitos, ou, para um repositório, na
  página **Prevenção de conflitos** dele. Em um script, `gitpoulpi conflicts check` imprime a mesma resposta em JSON
  ([Referência da CLI](cli-reference.md)).

## Agentes

Um agente pode mostrar um conflito a você (`gitpoulpi ui show conflict <path>
[--conflict N]`: o editor abre nele, nada é escolhido) ou **propor** uma resolução
(`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` ou `--content-file F`,
exigindo *Deixar agentes de IA proporem alterações para eu confirmar*). Uma
proposta abre o editor com as escolhas (ou o texto inteiro) já no Resultado e uma
aviso roxo *✦ Proposto por claude-code*. Revise, altere o que quiser e pressione
**Ctrl+S**: só isso grava ou coloca algo em stage. Fechar o editor rejeita a
proposta. Veja [MCP e agentes](mcp-and-agents.md).

A partir de um script, as mesmas operações são `gitpoulpi conflicts list | show |
resolve | unresolve | state | abort` ([Referência da CLI](cli-reference.md)); `conflicts check` os prevê antes de um merge.
