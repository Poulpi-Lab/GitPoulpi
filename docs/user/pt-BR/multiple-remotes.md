[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Vários remotos para uma branch

Muitos repositórios vivem em mais de um host: um principal (`origin`), um espelho em outro host, um backup. O GitPoulpi permite que uma branch faça push para todos eles com um clique e, se você quiser, faça pull de mais de um, sem mudar o que o git puro faz. Nada aqui exige uma conta, e tudo é guardado no próprio `.git/config` do repositório, então o git puro e outras ferramentas continuam funcionando.

## Destinos de push e fontes de pull

- O **upstream** é a noção do próprio git (`branch.<name>.remote` e `merge`). É o remoto *primário* da branch. O GitPoulpi nunca muda o seu funcionamento.
- Um **destino de push** é um remoto extra para o qual a branch também faz push. O botão Push envia a branch ao upstream e a cada destino de push.
- Uma **fonte de pull** é um remoto extra de onde um pull também traz commits. Fica desligada até você ligá-la (a caixa **Pull também**): fazer merge de dois remotos que você não escolheu não é um padrão que alguém queira.

Um destino pode usar outro nome de branch nesse remoto (`backup` recebe `feat-x`).

## O diálogo de remotos

Clique com o botão direito em uma branch e escolha **Remotos de <branch>...** (também no menu Push, na paleta como `Git: Remotos da branch...` e clicando no selo `⇉3` na linha da branch na seção Refs). Ele lista todos os remotos do repositório:

| Coluna | Significado |
|---|---|
| Remoto, Branch | O remoto e o nome da branch nele. **Renomear no remoto...** muda esse nome na linha selecionada. |
| Push | Marque para tornar o remoto um destino de push desta branch. |
| Pull também | Marque para que um pull também integre esse remoto. O upstream sempre sofre pull. |
| Estado | Onde a branch está em relação a ele: sincronizado, à frente, atrás, nova, sem relação. |

Uma linha marcada como "remoto removido" cita um remoto que a branch ainda menciona mas que o repositório não tem mais; **Remover** o esquece. O mesmo diálogo define o upstream (a lista **Upstream**) e **Remover upstream** o limpa. **Aplicar** grava tudo como uma única alteração que pode ser desfeita.

Um repositório também pode dar a toda branch nova os mesmos destinos extras: **Configurações > Git e codificação** (a página específica do repositório), "Destinos de push extras para novas branches" (uma caixa por remoto). Eles são copiados para uma branch quando ela recebe um upstream pela primeira vez; as branches existentes não são alteradas.

## Fazendo push

- **Botão Push** em uma branch com destinos extras: faz push para todos eles. Quando todo remoto simplesmente aceita os novos commits, executa na hora e a vista de progresso mostra cada remoto virando um visto. Se algo precisa de uma decisão (um remoto tem commits que você não tem), o diálogo do plano abre no lugar.
- **Push ▾** (a seta ao lado do botão; também o próprio botão quando **Botão Push** está em **perguntar** nas Configurações):
  - `Push para todos (origin, gitlab, backup)`
  - `Push somente para <remote>`, um por remoto da branch
  - `Push para remotos selecionados...`: sempre presente, para qualquer branch, mesmo uma com um único remoto
  - `Remotos de <branch>...`
- O menu de contexto da branch e a paleta (`Git: Push para todos os destinos`, `Git: Push para remotos selecionados...`) oferecem as mesmas ações com os mesmos nomes.

**Push para remotos selecionados...** abre uma lista de todos os remotos do repositório, marcados com o upstream e os destinos da branch. Vale só para este push; marque **Lembrar para esta branch** para guardar a escolha como os destinos de push da branch.

### O plano

Antes de um push em grupo o GitPoulpi olha cada remoto (uma consulta rápida e um download desse remoto só quando ele difere), para que um problema no terceiro remoto apareça antes de o primeiro push começar. O diálogo lista cada remoto com a situação da branch e o que será feito:

| Situação | Ação padrão | Também oferecido |
|---|---|---|
| O remoto não tem nada da branch, ou lhe faltam alguns commits | Push | Ignorar |
| O remoto está à frente (você não tem nada novo para ele) | Fast-forward de <remote> | Ignorar |
| O remoto difere porque você fez rebase ou amend no que enviou | Push forçado (lease) | Merge, Rebase, Ignorar |
| O remoto tem commits de outra pessoa | Merge de <remote> (Rebase quando **Botão Pull** está em rebase) | Rebase, Push forçado (lease), Ignorar |
| O histórico não tem nada em comum com o seu | Ignorar | Push forçado (lease) |
| O host recusou a branch (protegida) | Ignorar | nenhuma: forçar também seria recusado |

A ação de cada linha é uma lista que você pode mudar (Left/Right percorre, Enter abre, `s` ignora a linha). Um merge mostra se haveria conflito e em quais arquivos. Um **push forçado** é sempre confirmado, cita o remoto e quantos commits ele substitui, sempre usa um lease (nunca um force simples) e não pode ser desfeito: o resultado diz qual commit foi substituído e o comando que o traz de volta. **Ignorar problemas** põe toda linha com problema em Ignorar. **Executar** (Ctrl+Enter) inicia.

Os merges, fast-forwards e rebases rodam primeiro, na ordem da lista, de modo que sua branch chegue ao estado final uma só vez; depois os pushes saem um remoto de cada vez, o upstream primeiro. Não existe um "tudo ou nada" atômico entre remotos: cada um termina com seu próprio resultado, e uma falha em um (rede, um hook) não interrompe os outros. **Repetir falhas** (ou `Git: Tentar de novo os pushes que falharam`) tenta de novo esses. Fechar a vista de progresso nunca interrompe a execução; `Git: Mostrar push em grupo` a traz de volta.

### Conflitos e Continuar

Se um merge ou rebase da execução parar por conflitos, o diálogo se fecha e o banner de merge comum e o editor de conflitos assumem. Uma faixa no Log diz `Push em grupo pausado (gitlab): resolva e conclua, depois Continue.` com os botões **Continuar** e **Abortar grupo**.

- Resolva os conflitos e conclua o merge ou rebase (no GitPoulpi ou com o git puro).
- **Continuar** fica em cinza até isso ser feito, e **nunca é pressionado por você**: nada continua sozinho. Pressione-o (ou execute `Git: Continuar push em grupo`) e a execução faz push para todos os remotos de novo a partir da sua nova ponta. Depois de um rebase, um remoto que ainda guarda os commits antigos volta ao diálogo do plano (agora precisa de um push forçado) em vez de ser forçado em silêncio.
- **Abortar grupo** (com confirmação; `Git: Abortar push em grupo`) descarta o merge ou rebase que a execução iniciou e esquece a execução. Os pushes já feitos permanecem.

A faixa sobrevive a uma reinicialização do GitPoulpi, e a CLI vê a mesma execução salva.

## Fazendo pull

- O botão **Pull** faz pull do upstream exatamente como antes, a menos que a branch tenha fontes de pull: então faz pull do upstream e de cada fonte, em ordem, cada um no modo definido por **Botão Pull** nas Configurações (merge, somente fast-forward ou rebase). Uma fonte que não pode avançar por fast-forward sofre merge (ou rebase) como o upstream; com somente fast-forward, uma fonte que precisa de mais pergunta antes.
- **Pull ▾** acrescenta `Pull de todas as fontes (origin + gitlab)` quando há fontes, e `Pull somente de <remote>` para cada remoto da branch: um pull único da branch desse remoto, sejam quais forem as caixas.
- A paleta tem `Git: Pull de todas as fontes` e `Git: Pull de um remoto...`.
- Um pull que para por conflitos deixa a mesma faixa, identificada como pull em grupo.

## Configurações

Todas as quatro estão em **Configurações > Git e codificação**.

| Configuração | Padrão | Significado |
|---|---|---|
| **Botão Push** | direto | direto faz push na hora; perguntar abre antes o menu Push. |
| **Botão Pull** | merge | O modo do botão Pull (merge, somente fast-forward ou rebase), e das fontes de pull. |
| **Verificar todos os remotos antes de um push** | ligado | Olha cada remoto antes de um push em grupo (a CLI segue isso; `--no-preflight` desliga para uma chamada). |
| **Destinos de push extras para novas branches** (por repositório) | nenhum | Os remotos para os quais uma nova branch também faz push. |

## Pela linha de comando

```
gitpoulpi branch upstream add gitlab --pull      # um destino de push que também é fonte de pull
gitpoulpi branch upstream list                    # como a branch está configurada
gitpoulpi push --all-targets                      # upstream mais cada destino
gitpoulpi push --to gitlab --to backup:feat-x     # exatamente estes, só desta vez
gitpoulpi pull --all-sources                      # upstream mais cada fonte de pull
gitpoulpi remote-group plan                       # o que um push em grupo faria, sem alterar nada
gitpoulpi remote-group continue                   # depois de resolver e concluir uma parada
```

Quando um remoto precisa de uma decisão, o comando responde com o plano (saída 3) e não altera nada; responda com `--resolve gitlab=merge`. Veja [a referência da CLI](cli-reference.md) para cada flag. O servidor MCP tem as mesmas ferramentas.

## Com o git puro

Os destinos são chaves de configuração comuns, então um script ou um alias pode usá-los sem o GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # um valor por linha: remote ou remote:branch

# push da branch atual para o upstream e para cada destino
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Um valor é `<remote>` (mesmo nome de branch) ou `<remote>:<branch>`, que é o que a linha `d=${t#*:}` resolve. O alias simples não olha os remotos antes, não oferece um merge e não consegue continuar depois de um conflito: para isso existe o diálogo.

## Bom saber

- Dois remotos que apontam para o mesmo endereço são notados ("gitlab e mirror fazem push para a mesma URL."); o segundo é ignorado por padrão.
- Um host com uma branch protegida recusa um push forçado; a linha diz isso e oferece só Ignorar.
