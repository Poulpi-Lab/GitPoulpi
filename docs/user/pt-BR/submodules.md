[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submódulos

Um submódulo é outro repositório aninhado dentro do seu em um commit fixo. O GitPoulpi os lista na barra lateral Refs, os atualiza pela interface, pela CLI ou pela paleta de comandos, e pode mantê-los em sincronia com o commit que você faz checkout.

## A seção SUBMÓDULOS

Quando um repositório tem um `.gitmodules`, a barra lateral Refs ganha uma seção **SUBMÓDULOS** (abaixo das outras; ela não existe quando não há nenhum). Uma linha por submódulo: uma marca de estado, seu caminho e um commit curto.

| Marca | Estado | Significado |
|---|---|---|
| `✓` | atualizado | em checkout no commit que o repositório registra |
| `↑` | modificado | em checkout em outro commit (ele avançou, ou você trocou de branch sem atualizá-lo) |
| `○` | não clonado | declarado em `.gitmodules` e registrado, mas a pasta dele está vazia: atualize-o para cloná-lo |
| `✗` | em conflito | o repositório tem conflitos de merge no submódulo |

| Fazer | Mouse | Teclado |
|---|---|---|
| Abrir o submódulo em uma aba própria | clique duplo na linha | selecione-o, `Enter` (paleta: `Refs: abrir submódulo em uma aba`) |
| Menu: **Atualizar (clonar se preciso)**, **Inicializar**, **Abrir em nova aba** | botão direito na linha | selecione-o, a tecla de menu de contexto (padrão `.`) |
| Atualizar o submódulo selecionado (clonando-o antes, se preciso) | menu | `Ctrl+P`: `Refs: atualizar submódulo (selecionado)` |
| Registrá-lo em `.git/config` sem clonar | menu | `Ctrl+P`: `Refs: inicializar submódulo (selecionado)` |
| Atualizar todos os submódulos | | `Ctrl+P`: `Git: atualizar todos os submódulos` |
| Ir para a seção | clique no cabeçalho dela | `Ctrl+P`: `Navegação: focar Refs > Submódulos` |

Uma atualização pode fazer fetch do remoto do submódulo, então ela segue **Configurações > SSH e credenciais > Credenciais HTTPS** como qualquer outra transferência. Um submódulo que não foi clonado não tem o que abrir como aba; a linha de status avisa isso e aponta para Atualizar.

## Mantendo os submódulos atualizados

**Configurações > Sincronização > Manter submódulos atualizados** (`general.keep_submodules_up_to_date`, desativado por padrão). Quando ativado, um **checkout** ou **pull** bem-sucedido é seguido de `git submodule update --init --recursive`. Isso só roda se algum submódulo estiver fora de sincronia com o que o repositório agora registra, então um repositório sem submódulos não fica mais lento. Se essa atualização falhar, o checkout ou o pull aconteceu mesmo assim e o erro diz isso.

## Pela linha de comando

```
gitpoulpi submodule list                       # caminho, estado, url (--json: a lista completa)
gitpoulpi submodule init [<path>...]           # registra, não clona
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Sem caminhos, vale para todos os submódulos. Os mesmos três são ferramentas MCP (`submodule_list` é uma leitura; `init` e `update` são mutações e seguem o `allow_mutations` de `[mcp]`).

## Notas

- Só o nível superior é listado; os submódulos de um submódulo aparecem quando você o abre como aba (ou use `--recursive` para atualizá-los).
- Versões novas do git se recusam a clonar um submódulo de *caminho local* pelo transporte `file`, a menos que `protocol.file.allow` o permita. Isso só importa para submódulos cuja URL é uma pasta local; defina-o na sua configuração global do git se você os usa.
