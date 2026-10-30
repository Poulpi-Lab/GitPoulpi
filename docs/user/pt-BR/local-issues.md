[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Issues locais e Obsidian

Guarde as suas issues como uma pasta de notas em Markdown: sem conta, sem token, sem rede. A pasta pode ficar dentro do
repositório (`docs/issues/`, com commit junto com o código) ou em qualquer lugar do disco, por exemplo uma subpasta de um
cofre do [Obsidian](https://obsidian.md). O GitPoulpi lê e grava as notas. O Obsidian, qualquer editor ou um shell podem editá-las
ao mesmo tempo, e as listas acompanham.

![Issues locais em Markdown na barra lateral, com o menu da linha de uma nota: abrir a nota, fechá-la, editar os seus rótulos ou o marco](../images/local-issues.png)

## Configurando

1. **Configurações > Integrações > Rastreadores de issues > Markdown local > Adicionar perfil.** Digite a pasta, ou pressione
   **Procurar...** e escolha-a. Uma pasta relativa (`docs/issues`) é lida em relação a cada repositório a que está vinculada; uma
   absoluta (`/home/me/Vault/Projects/Issues`) é a mesma em todo lugar. Não há token a informar.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. Em um repositório, abra a seção **ISSUES** da barra lateral e escolha o provedor **Markdown local - <nome>** (a pasta
   é escolhida junto; a lista de repositórios mostra a pasta e as suas subpastas diretas, se você quiser outra).
   - CLI: `gitpoulpi hosting link issues --id <perfil>` (a pasta do perfil é o `--project` padrão).

Nada é vinculado sozinho: como toda fonte de issues, ela é opcional por repositório.

## As notas

Uma nota por issue, chamada `<id>-<slug>.md` (`12-crash-on-empty-name.md`), com um bloco de frontmatter:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Markdown livre. O GitPoulpi nunca reescreve o corpo.
```

- As novas issues recebem `max(id) + 1`.
- O GitPoulpi só grava as propriedades que são dele (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Todas as outras propriedades, o corpo, as quebras de linha e a ordem das chaves são mantidos byte a byte.** As gravações são
  atômicas (um arquivo temporário, depois uma renomeação).
- A leitura é tolerante: uma nota sem `id` o toma do nome do arquivo, uma nota sem `status` é `todo`, uma nota
  sem frontmatter toma o título do primeiro `# título`. Um arquivo que não é uma issue (`readme.md`) é ignorado.
- Um status é texto livre. **Aberta** quer dizer "não está no status de concluída".

## O que você pode fazer

| Na seção ISSUES | Teclado / paleta |
|---|---|
| Nova issue (o `+`, ou `n`): título, descrição, status, responsáveis, rótulos, marco | `Hospedagem: nova issue...` |
| Editar responsáveis / rótulos / marco no menu da linha. Escolha entre o que a pasta já usa, ou **digite um novo rótulo ou login** e use a linha `Adicionar "..."` | `Hospedagem: editar rótulos...` e as irmãs |
| **Fechar issue** / **Reabrir issue** (define o status de concluída, ou o primeiro outro) | `Issues locais: fechar issue...` / `reabrir issue...` |
| **Abrir nota** (o seu programa padrão para `.md`) | `Integrações: abrir issue...` |
| **Abrir no Obsidian** (só quando a pasta está dentro de um cofre, ou seja, uma pasta `.obsidian/` acima dela) | `Issues locais: abrir no Obsidian...` |
| Filtros: Abertas / Fechadas / Todas, "Atribuídas a mim" | a lista suspensa de filtro |

"Atribuídas a mim" corresponde à configuração `me` do perfil, senão ao nome conectado, senão ao nome do perfil.

Branches e commits funcionam como no GitHub: "Criar branch a partir da issue" dá `12-crash-on-empty-name`, e `#12` em uma mensagem de
commit ou `12-...` no nome de uma branch é reconhecido.

As alterações feitas fora do GitPoulpi (Obsidian, um shell, um `git pull`) aparecem em cerca de um segundo: os nomes dos arquivos da pasta
e as datas de modificação são consultados em uma thread em segundo plano, nunca na thread de renderização.

## Arquivo de configuração

No `config.toml`, ao lado do perfil (toda chave é opcional):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # a ordem das colunas do quadro
done_status = "done"                            # fecha uma issue
reference_prefix = "LOC"                         # mostrado como LOC-12 em vez de #12
me = "kai"                                       # quem "atribuídas a mim" quer dizer
```

Os status usados por notas mas não listados em `statuses` vêm depois dos listados. `statuses` é a ordem das colunas do
[quadro](#o-quadro-kanban) (`gitpoulpi hosting issue options` a imprime).

## O quadro Kanban

![O quadro de uma pasta de notas: uma coluna por status, os cartões com seus responsáveis, rótulos e marco](../images/issue-board.png)

Se você trabalha sozinho, ou em uma equipe que guarda as suas issues junto do código, a pasta também é um quadro Kanban.
**Cabeçalho ISSUES > Quadro**, ou **Issues: abrir quadro** da paleta, o abre sobre a janela; `Esc` o fecha.

- As **colunas** são os status: os listados em `statuses` primeiro, nessa ordem, depois qualquer outro status que uma nota use.
  Uma coluna vazia continua sendo mostrada. Cada cabeçalho traz a sua contagem de cartões.
- Os **cartões** mostram o número da issue, o título (duas linhas), os responsáveis como iniciais, os rótulos e o marco.
  Quando as colunas não cabem todas, o quadro rola para o lado (a barra sob ele, a roda com `Shift`, ou as setas).
- **Mova um cartão** arrastando-o para outra coluna (a coluna se acende sob o ponteiro), com
  `Shift+Left` / `Shift+Right`, ou com **Quadro: mover cartão para...** (`m`), uma lista suspensa das colunas. O cartão se move na
  hora; o `status` da nota é reescrito como em [a seção das notas](#as-notas), nada mais no arquivo. Se a gravação
  falhar, o cartão volta e o erro aparece na linha de status.
- **Abra** um cartão com `Enter` ou um clique duplo (a nota abre no seu programa padrão para `.md`). `e` edita os seus
  responsáveis, rótulos ou marco, `.` ou um clique com o botão direito abre o menu do cartão, `n` inicia uma nova issue na coluna
  selecionada (o formulário volta ao quadro quando se fecha).
- **Filtre** por responsável (`a`), rótulo (`l`), marco (`M`) ou texto (`/`); `x` os limpa. Os filtros são
  listas suspensas: digite para reduzir, `Enter` escolhe.
- **Ao vivo**: uma nota alterada pelo Obsidian ou por um shell move o seu cartão em um segundo.

Todo gesto do mouse tem uma tecla e um comando da paleta (**Quadro: ...**, listados na página de
[atalhos de teclado](keybindings.md)).

## Bases do Obsidian

**Paleta: `Issues locais: criar Base do Obsidian`** (CLI: `gitpoulpi hosting issue base`) grava `issues.base` ao lado das
notas, uma vez: uma visão de *tabela* e uma visão *kanban* agrupada por `status`, na ordem de `statuses`. Abra-o no Obsidian para
ver as mesmas issues como um quadro lá. O GitPoulpi nunca edita um arquivo `.base` existente (seu ou de uma vez anterior): ele
recusa e avisa. O formato segue a sintaxe de Bases do Obsidian
(<https://obsidian.md/help/bases/syntax>); a visão kanban precisa de um Obsidian recente.

## CLI e MCP

Os mesmos comandos de qualquer fonte de issues funcionam nela: `hosting issues`, `hosting issue create [--list <status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Mais quatro são para notas:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` e `hosting issue base`.
Veja a [Referência da CLI](cli-reference.md).

## Limites

- Um novo **marco** não pode ser digitado no seletor (escolha um que a pasta já use), mas `milestone:` em uma nota pode
  ser qualquer coisa e aparece assim que é salvo.
- Só as notas diretamente na pasta são issues (não as subpastas).
- Não há exclusão: remova a nota no seu gerenciador de arquivos ou no Obsidian (a lixeira dele é sua).
