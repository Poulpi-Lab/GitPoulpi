[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · **Português (Portugal)** · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Vários remotos para uma branch

Muitos repositórios vivem em mais do que um alojamento: um principal (`origin`), um espelho noutro alojamento, uma cópia de segurança. O GitPoulpi permite que uma branch faça push para todos com um clique e, opcionalmente, pull de mais do que um, sem mudar o que o git simples faz. Nada aqui precisa de uma conta, e tudo fica guardado no `.git/config` do próprio repositório, por isso o git simples e outras ferramentas continuam a funcionar.

## Destinos de push e fontes de pull

- O **upstream** é a noção própria do git (`branch.<name>.remote` e `merge`). É o remoto *principal* da branch. O GitPoulpi nunca muda a forma como funciona.
- Um **destino de push** é um remoto adicional para onde a branch também faz push. O botão Push envia a branch para o upstream e para cada destino de push.
- Uma **fonte de pull** é um remoto adicional de onde um pull também traz commits. Está desligada a menos que a ligue (a caixa **Pull também**): integrar dois remotos que não escolheu não é uma predefinição que alguém queira.

Um destino pode usar outro nome de branch nesse remoto (`backup` recebe `feat-x`).

## O diálogo Remotos

Clique com o botão direito numa branch e escolha **Remotos de <branch>...** (também no menu Push, na paleta como `Git: Remotos da branch...`, e ao clicar no emblema `⇉3` na linha da branch na secção Refs). Lista todos os remotos do repositório:

| Coluna | Significado |
|---|---|
| Remoto, Branch | O remoto e o nome da branch aí. **Mudar nome no remoto...** altera esse nome na linha selecionada. |
| Push | Marque-a para tornar o remoto um destino de push desta branch. |
| Pull também | Marque-a para que um pull integre também esse remoto. O upstream é sempre puxado. |
| Estado | A situação da branch em relação a ele: sincronizado, à frente, atrás, novo, sem relação. |

Uma linha marcada "remoto removido" nomeia um remoto que a branch ainda menciona mas que o repositório já não tem; **Remover** esquece-o. O mesmo diálogo define o upstream (a lista **Upstream**) e **Remover upstream** limpa-o. **Aplicar** escreve tudo como uma única alteração que pode ser desfeita.

Um repositório também pode dar a cada nova branch os mesmos destinos adicionais: **Definições > Git e codificação** (a página específica do repositório), "Destinos de push adicionais para novas branches" (uma caixa por remoto). São copiados para uma branch quando esta recebe um upstream pela primeira vez; as branches existentes não são tocadas.

## Fazer push

- **Botão Push** numa branch com destinos adicionais: faz push para todos. Quando todos os remotos aceitam simplesmente os novos commits, executa de imediato e a vista de progresso mostra cada remoto a passar a visto. Se algo precisar de uma decisão (um remoto tem commits que você não tem), abre-se em vez disso o diálogo do plano.
- **Push ▾** (a seta junto ao botão; também o próprio botão quando **Botão Push** está em **perguntar** nas Definições):
  - `Push para todos (origin, gitlab, backup)`
  - `Push só para <remoto>`, um por cada remoto da branch
  - `Push para remotos selecionados...`: está sempre lá, para qualquer branch, mesmo com um único remoto
  - `Remotos de <branch>...`
- O menu de contexto da branch e a paleta (`Git: Push para todos os destinos`, `Git: Push para um remoto...`, `Git: Push para remotos selecionados...`) oferecem as mesmas ações com os mesmos nomes.

**Push para remotos selecionados...** abre uma lista de todos os remotos do repositório, marcados com o upstream e os destinos da branch. Vale só para este push; marque **Memorizar para esta branch** para guardar a escolha como destinos de push da branch.

### O plano

Antes de um push em grupo, o GitPoulpi olha para cada remoto (uma consulta rápida, e uma transferência desse remoto apenas se diferir) para que um problema no terceiro remoto apareça antes de o primeiro push começar. O diálogo lista cada remoto com a situação da branch e o que vai fazer:

| Situação | Ação predefinida | Também oferecido |
|---|---|---|
| O remoto não tem nada da branch, ou faltam-lhe alguns commits | Push | Saltar |
| O remoto está à frente (não tem nada de novo para ele) | Fast-forward de si | Saltar |
| O remoto difere porque fez rebase ou amend ao que enviou | Push forçado (lease) | Merge, Rebase, Saltar |
| O remoto tem commits de outra pessoa | Merge de si (Rebase quando o **Botão Pull** está em rebase) | Rebase, Push forçado (lease), Saltar |
| O histórico não partilha nada com o seu | Saltar | Push forçado (lease) |
| O alojamento recusou a branch (protegida) | Saltar | nenhum: forçar também seria recusado |

A ação de cada linha é uma lista que pode alterar (Esquerda/Direita percorre-a, Enter abre-a, `s` salta a linha). Um merge mostra se daria conflito e em que ficheiros. Um **push forçado** é sempre confirmado, nomeia o remoto e quantos commits substitui, usa sempre um lease (nunca um force simples) e não pode ser desfeito: o resultado diz que commit foi substituído e o comando que o repõe. **Saltar problemas** põe em Saltar todas as linhas com problemas. **Executar** (Ctrl+Enter) começa.

Os merges, fast-forwards e rebases correm primeiro, pela ordem da lista, para que a sua branch chegue uma só vez ao estado final; depois os pushes saem um remoto de cada vez, começando pelo upstream. Não há um "tudo ou nada" atómico entre remotos: cada um termina com o seu resultado, e uma falha num (rede, um hook) não pára os outros. **Repetir falhados** (ou `Git: Repetir os remotos falhados`) volta a tentar esses. Fechar a vista de progresso nunca pára a execução; `Git: Mostrar o push em grupo` traz-a de volta.

### Conflitos e Continuar

Se um merge ou rebase da execução parar por conflitos, o diálogo fecha-se e o banner de merge habitual e o editor de conflitos assumem. Uma faixa no Registo diz `Push em grupo em pausa (gitlab): resolva e conclua, depois Continuar.` com os botões **Continuar** e **Abortar grupo**.

- Resolva os conflitos e conclua o merge ou rebase (no GitPoulpi ou com o git simples).
- **Continuar** fica a cinzento até isso estar feito, e **nunca é premido por si**: nada continua sozinho. Prima-o (ou execute `Git: Continuar o push em grupo`) e a execução volta a fazer push para todos os remotos a partir da sua nova ponta. Depois de um rebase, um remoto que ainda tenha os commits antigos volta ao diálogo do plano (agora precisa de um push forçado) em vez de ser forçado em silêncio.
- **Abortar grupo** (confirmado; `Git: Abortar o push em grupo`) deita fora o merge ou rebase que a execução iniciou e esquece a execução. Os pushes já feitos ficam.

A faixa sobrevive a um reinício do GitPoulpi, e a CLI vê a mesma execução guardada.

## Fazer pull

- O botão **Pull** faz pull do upstream exatamente como antes, a menos que a branch tenha fontes de pull: então faz pull do upstream e de cada fonte, por ordem, cada um no modo definido por **Botão Pull** nas Definições (merge, só fast-forward ou rebase). Uma fonte que não pode avançar por fast-forward é integrada com merge (ou rebase) como o upstream; com só fast-forward, uma fonte que precise de mais pergunta primeiro.
- **Pull ▾** acrescenta `Pull de todas as fontes (origin + gitlab)` quando há fontes, e `Pull só de <remoto>` para cada remoto da branch: um pull pontual da branch desse remoto, seja qual for o estado das caixas.
- A paleta tem `Git: Pull de todas as fontes` e `Git: Pull de um remoto...`.
- Um pull que pare por conflitos deixa a mesma faixa, identificada como pull em grupo.

## Definições

As quatro estão em **Definições > Git e codificação**.

| Definição | Predefinição | Significado |
|---|---|---|
| **Botão Push** | direto | direto faz push de imediato; perguntar abre primeiro o menu Push. |
| **Botão Pull** | merge | O modo do botão Pull (merge, só fast-forward ou rebase), e das fontes de pull. |
| **Verificar cada remoto antes de push** | ligada | Olhar para cada remoto antes de um push em grupo (a CLI segue-a; `--no-preflight` desliga-a numa chamada). |
| **Destinos de push adicionais para novas branches** (por repositório) | nenhum | Os remotos para onde uma nova branch também faz push. |

## A partir da linha de comandos

```
gitpoulpi branch upstream add gitlab --pull      # um destino de push que é também fonte de pull
gitpoulpi branch upstream list                    # para o que a branch está configurada
gitpoulpi push --all-targets                      # upstream mais todos os destinos
gitpoulpi push --to gitlab --to backup:feat-x     # exatamente estes, só desta vez
gitpoulpi pull --all-sources                      # upstream mais todas as fontes de pull
gitpoulpi remote-group plan                       # o que um push em grupo faria, nada alterado
gitpoulpi remote-group continue                   # depois de resolver e concluir uma paragem
```

Quando um remoto precisa de uma decisão, o comando responde com o plano (código 3) e não altera nada; responda-lhe com `--resolve gitlab=merge`. Veja [a referência da CLI](cli-reference.md) para todas as opções. O servidor MCP tem as mesmas ferramentas.

## Com o git simples

Os destinos são chaves de configuração simples, por isso um script ou um alias pode usá-los sem o GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # um valor por linha: remote ou remote:branch

# push da branch atual para o seu upstream e para todos os destinos
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Um valor é `<remote>` (mesmo nome de branch) ou `<remote>:<branch>`, que é o que a linha `d=${t#*:}` resolve. O alias simples não olha primeiro para os remotos, não oferece um merge e não pode continuar depois de um conflito: é para isso que serve o diálogo.

## Bom saber

- Dois remotos que apontam para o mesmo endereço são detetados ("gitlab e mirror têm o mesmo URL de push"); o segundo é saltado por predefinição.
- Um alojamento com uma branch protegida recusa um push forçado; a linha di-lo e oferece apenas Saltar.
