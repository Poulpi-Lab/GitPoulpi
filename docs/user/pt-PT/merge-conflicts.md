[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · [Deutsch](../de/merge-conflicts.md) · [Español](../es/merge-conflicts.md) · [Français](../fr/merge-conflicts.md) · [Italiano](../it/merge-conflicts.md) · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · [Português (Brasil)](../pt-BR/merge-conflicts.md) · **Português (Portugal)** · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · [简体中文](../zh-CN/merge-conflicts.md)

# Resolver conflitos de merge

Quando um merge, cherry-pick, revert ou rebase pára em conflitos, o Log mostra
um aviso (*Merge em curso · feature em main · 2 ficheiros em conflito*) e os
ficheiros em conflito aparecem no painel Status com um `!` vermelho. Abra um no
**editor de merge**: A (o lado atual) e B (o lado recebido) lado a lado, com o
**Resultado**, o ficheiro que está a escrever, por baixo. Esta página é a versão
curta; a lista completa de teclas está em [Atalhos de teclado](keybindings.md).

## Abrir o editor

- Enter, ou um clique duplo, numa linha `!` do painel Status; o seu menu de
  clique com o botão direito começa por **Abrir no editor de merge**.
- O botão **Resolver conflitos...** do aviso (abre o primeiro ficheiro em
  conflito).
- A vista Diff de um ficheiro em conflito: o botão **Editor de merge**
  (`Shift+E`).
- A paleta: **Conflitos: abrir o editor de merge**.

Nada se abre sozinho: uma operação parada mostra apenas o aviso e as linhas `!`.

![O Log de um repositório parado num merge com conflitos: o aviso Merge em curso com Resolver conflitos e Abortar merge, e dois ficheiros em conflito marcados com um ! vermelho](../images/conflict-banner.png)

O editor ocupa toda a largura da janela: a barra lateral Refs
e o painel direito ficam ocultos enquanto está aberto e voltam quando o fecha
(`F9` mostra-os ou oculta-os entretanto). A e B ficam lado a lado sempre que o
editor tem pelo menos `merge.side_by_side_min_width` colunas de largura (100 por
predefinição, Definições > Diff e editor; `0` significa sempre); mais estreito,
vê um lado de cada vez atrás de separadores. As linhas compridas deslocam-se
para o lado, A e B em conjunto (`Shift+←` / `Shift+→`, Shift+roda ou uma roda
lateral); nunca são quebradas.

![O editor de merge: A (atual) e B (recebido) lado a lado por cima do Resultado, conflito 1 aceite de A e conflito 2 de B, conflito 3 ainda por escolher](../images/merge-editor.png)

## Escolher

Cada conflito é um bloco emoldurado em A e em B, com uma caixa de seleção no
cabeçalho e um `+` / `✓` à frente de cada linha.

| Para... | Tecla | Rato |
|---|---|---|
| aceitar a linha sob o cursor (lado com foco) | `Space` | clicar na linha |
| aceitar o excerto inteiro do lado com foco | `Enter` | clicar no `[ ]` do seu cabeçalho |
| aceitar só A / só B neste conflito | `a` / `b` | clique com o botão direito no excerto |
| aceitar ambos, A depois B / B depois A | `c` / `Shift+C` | clique com o botão direito no excerto |
| não aceitar nada | `x` | clique com o botão direito no excerto |
| trocar a ordem de dois lados escolhidos | `s` | clicar no emblema `1º` / `2º` |
| aceitar tudo de A / tudo de B (todos os conflitos) | `Shift+A` / `Shift+B` | barra de ferramentas |
| desfazer a escolha neste conflito (volta a não resolvido) | `u` em A / B, `Alt+U` em qualquer lado | clique com o botão direito no excerto, **Desfazer a escolha** |
| repor o ficheiro todo nos conflitos originais (pergunta primeiro quando escreveu no Resultado; `Ctrl+Z` recupera) | `Shift+U` em A / B, `Alt+R` em qualquer lado | **Repor**, junto de Aceitar tudo |
| conflito seguinte / anterior | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` em A / B) | `▼` / `▲`, ou clicar numa marca de uma barra de deslocação |
| um número repete um movimento | `3j`, `2k`, `2n`, `3N` (em A / B) | |
| ficheiro em conflito seguinte / anterior | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` no cabeçalho |
| mostrar ou ocultar o painel Base (antecessor comum) | `Alt+D` (`d` em A / B) | `Base` |
| mostrar o ficheiro todo em A e B (**Ficheiro todo**) | `Alt+F` (`f` em A / B) | `Ficheiro todo` |
| dar mais largura a A ou B / voltar a larguras iguais | `Alt+Right` / `Alt+Left`, `Alt+0` | arrastar o divisor; clique duplo nele |
| deslocar A / B / Base para o lado | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+roda, roda lateral |
| mostrar ou ocultar os painéis Refs e Status | `F9` | |
| dar foco a A, B, Resultado | `F6` / `Shift+F6` | clicar |
| saltar para um painel, no conflito em que está | `Alt+1` A, `Alt+2` B, `Alt+3` Resultado, `Alt+4` Base; ou `Ctrl+N` e depois `1` / `2` / `3` | |
| passar ao painel vizinho (A, Base, B, Resultado), mesmo conflito | `h` / `l` ou `←` / `→` em A / B | |
| alternar a altura do Resultado | `z` | arrastar o seu título |
| desfazer / refazer (escolhas e escrita, uma só pilha) | `Ctrl+Z` / `Ctrl+Y` | |
| escrever o ficheiro, mantendo-o em conflito | `Alt+S` | **Guardar** |
| escrever o ficheiro e marcá-lo como resolvido | `Ctrl+S` | **Guardar e marcar resolvido** |
| fechar | `Esc` | `✕` |

Cada barra de deslocação (A / B / Base e o Resultado) leva uma marca por
conflito: laranja enquanto ainda precisa de uma escolha, verde depois de
tratado, azul para o atual. Um clique numa marca salta para esse conflito (uma
marca sob o indicador da barra já está no ecrã, por isso aí o clique arrasta o
indicador).

**Ficheiro todo** mostra o ficheiro inteiro em A e B com todas as diferenças
coloridas, não só os conflitos:

![Modo Ficheiro todo: o ficheiro inteiro em A e B, com as alterações coloridas](../images/merge-editor-full-file.png)

O Resultado é um editor a sério: clique nele e escreva, selecione, corte e cole,
pesquise com `Ctrl+F`. Uma escolha num conflito que editou à mão substitui a sua
edição (`Ctrl+Z` recupera-a). Guardar com um conflito sem nada escolhido, ou com
uma linha que ainda parece um marcador de conflito, pergunta primeiro.

### ✦ Sugerir uma resolução (IA)

Com a IA ligada (Definições > IA; veja [Assistência de IA](ai.md)) a barra de
ferramentas tem um botão **✦ Sugerir**, o menu de clique com o botão direito de
um conflito tem *✦ Sugerir resolução (IA)*, e `Alt+I` e a paleta (*IA: sugerir
resolução de conflito de merge*) fazem o mesmo. Trabalha apenas sobre o conflito
atual e só corre quando o pede. A resposta substitui a parte desse conflito no
Resultado como uma única edição, marcada como editada (`✎`), pelo que `Ctrl+Z`
repõe o conflito como estava. Nunca é guardada nem preparada (staged) por si.
Enquanto corre, a linha de dicas mostra o tempo gasto; `Esc` cancela, e uma
falha também aparece aí. Com a IA desligada, nenhum destes controlos existe.

Depois de **Guardar e marcar resolvido**, o ficheiro fica staged e volta à lista
de ficheiros em conflito; abra manualmente o ficheiro seguinte.
`Ctrl+Z` (ou o botão Desfazer) desfaz o último "marcar resolvido". Quando já não
resta nenhum ficheiro, o aviso oferece **Commit do merge** (ou **Continuar**
para um cherry-pick, um revert ou um rebase) e **Abortar merge**, que pergunta
primeiro: repõe HEAD, o índice e a cópia de trabalho onde estavam.

Um merge cujo resultado é igual a HEAD (aceitou o nosso lado em todo o lado) não
tem nada staged nem ficheiro alterado, mas ainda assim pode ser concluído:
**Commit do merge** abre o pedido de commit com a mensagem do merge, e o commit
tem ambos os pais. Do mesmo modo, o botão Commit do compositor fica ativo sem
nada staged enquanto um merge está parado.

Um cherry-pick ou revert vazio (a alteração já está aplicada) não pode ser
commitado: o git recusa-o, e **Continuar** di-lo e aponta para **Saltar**.
**Saltar** (botão do aviso, entrada da paleta *Git: saltar cherry-pick ou
revert*, CLI `conflicts skip`) pergunta primeiro e depois termina a operação sem
registar nada para ela; HEAD, o índice e a cópia de trabalho voltam a HEAD.
Saltar não é desfazível (nada se perde que não tenha sido abandonado de
propósito); um merge não tem nada para saltar.

## Conflitos que não são texto

Um ficheiro binário, um ponteiro Git LFS, um ficheiro com mais de 8 MiB, um
ficheiro eliminado num dos lados, um resto de uma renomeação, uma ligação
simbólica, uma alteração de modo ou um submódulo não têm linhas para escolher.
O editor mostra um **cartão** com o que cada lado contém e um botão por
escolha; escolher resolve e faz stage do ficheiro de imediato.

| Cartão | Escolhas |
|---|---|
| Ficheiro binário, ponteiro LFS, demasiado grande | **Manter A** (`a`), **Aceitar B** (`b`), **Aceitar Base** (`o`, quando existe), **Abrir na ferramenta de merge** (`m`) |
| Eliminado num lado, alterado no outro | **Manter o ficheiro** (`k`), **Aceitar a eliminação** (`Shift+D`) |
| Resto de renomeação (o caminho existe só num dos lados) | **Manter o ficheiro** (`k`), **Eliminar o ficheiro** (`Shift+D`) |
| Ligação simbólica, submódulo, modo do ficheiro | **Manter A**, **Aceitar B**, **Aceitar Base** |

Quando o ficheiro é uma imagem (PNG, JPEG, GIF, WebP, BMP, ICO), as caixas
mostram também as imagens de A, B e Base, ajustadas e só de leitura, numa janela
suficientemente larga para elas; veja
[Imagens](image-view.md#no-editor-de-merge).

![O cartão de um conflito binário: o que A, B e a base contêm, e os botões Manter A, Aceitar B, Aceitar Base e Abrir na ferramenta de merge](../images/merge-editor-card.png)

`Esc` fecha o cartão sem escolher. A ferramenta de merge externa é
`general.external_merge_tool` em Definições > Geral (`{base}`, `{local}` = A,
`{remote}` = B, `{merged}`; vazio significa `git mergetool`); o ficheiro fica
staged quando a ferramenta termina com sucesso.

## Prevenção de conflitos

O GitPoulpi pode avisá-lo *antes* de fazer merge. Quando a branch em que está daria conflito com a branch de que partiu
(a sua **base**), aparece uma tira na linha de margem superior do Log: **Em conflito com origin/main em 2 ficheiros**, com um
botão **Mostrar ficheiros**. Ocupa a linha de margem livre seguinte aos avisos das operações em curso, por isso, enquanto um
desses está visível, pode ter de esperar por espaço. Nada é integrado: o daemon faz em memória o merge dos dois commits
(libgit2, ou `git merge-tree --write-tree` com o git 2.38 ou posterior; um git mais antigo desativa a funcionalidade com uma linha
no registo), pelo que os seus ficheiros, o índice e as refs não são tocados.

- **A base** é a branch de destino do pull request da branch quando o GitPoulpi já a conhece (a cache que a secção
  Pull requests mantém; não é feita nenhuma chamada à rede para isto), senão a branch, entre os padrões das **Branches base**
  (Definições > Prevenção de conflitos), com menos commits desde a bifurcação. Um empate fica com a branch cuja
  ponta está mais perto da bifurcação, depois com uma branch de acompanhamento remoto em vez de uma local, depois com o nome. A própria branch e
  a sua cópia remota nunca são a base.
- **A janela** (clique na tira, ou **Conflitos: mostrar os conflitos previstos** na paleta, que funciona a partir do
  teclado) lista os ficheiros, e tem **Abrir a pré-visualização do merge** e **Rebase sobre a base**. São as próprias linhas do menu
  da branch base (*Merge de <base> em <branch>*, *Rebase de <branch> sobre <base>*) executadas por si: ficam no registo de desfazer, por isso `Ctrl+Z` desfaz-as, e um
  merge que dê mesmo conflito pára no editor de merge como qualquer outro. Esquerda e Direita movem entre os botões, Enter
  prime um, Esc fecha.
- **Quando corre**: em segundo plano, cerca de 2 segundos depois de o repositório sossegar (um commit, um checkout, um
  fetch que moveu uma branch, um rebase), uma vez por par `(HEAD, base)`. A tira oculta-se assim que HEAD se move e volta
  a aparecer, ou não, com a nova resposta. Um HEAD desanexado não é verificado.
- **Listas de branches base**: de onde vêm as Branches base de um repositório é uma escolha na sua página **Prevenção de
  conflitos**. **Predefinida** segue a predefinição global (a `**` incorporada, todas as branches, até escolher outra lista em
  Definições > Geral > Prevenção de conflitos); **Lista: <nome>** segue uma das suas listas com nome; **Lista própria** é uma lista
  só sua. As listas são partilhadas: edite uma e todos os repositórios que a seguem usam a alteração na verificação seguinte, sem
  reiniciar. **Guardar como lista...** transforma os padrões próprios de um repositório numa lista com nome, **Repor a predefinição**
  descarta a escolha do repositório. Eliminar uma lista que um repositório segue faz com que ele use a predefinida.
- **Desativada**: **Ativar a deteção proativa de conflitos** em Definições > Geral, ou, para um só repositório, na sua página **Prevenção
  de conflitos**. A partir de um script, `gitpoulpi conflicts check` imprime a mesma resposta em JSON ([Referência
  da CLI](cli-reference.md)).

## Agentes

Um agente pode mostrar-lhe um conflito (`gitpoulpi ui show conflict <path>
[--conflict N]`: o editor abre-se nele, nada é escolhido) ou **propor** uma
resolução (`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` ou
`--content-file F`, o que exige *Deixar agentes de IA proporem alterações para
eu confirmar*). Uma proposta abre o editor com as escolhas (ou o texto inteiro)
já no Resultado e um aviso roxo *✦ Proposto por claude-code*. Reveja-a, altere
o que quiser e prima **Ctrl+S**: só isso escreve ou faz stage de alguma coisa.
Fechar o editor rejeita a proposta. Veja [MCP e agentes](mcp-and-agents.md).

A partir de um script, as mesmas operações são `gitpoulpi conflicts list | show |
resolve | unresolve | state | abort` ([Referência da CLI](cli-reference.md)); `conflicts check` prevê-os antes de um merge.
