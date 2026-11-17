[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · **Português (Portugal)** · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Submódulos

Um submódulo é outro repositório aninhado dentro do seu, num commit fixo. O GitPoulpi lista-os na barra lateral Refs, atualiza-os a partir da interface, da CLI ou da paleta de comandos, e pode mantê-los em sintonia com o commit para o qual faz checkout.

## A secção SUBMÓDULOS

Quando um repositório tem um `.gitmodules`, a barra lateral Refs ganha uma secção **SUBMÓDULOS** (por baixo das outras; não existe quando não há nenhum). Uma linha por submódulo: uma marca de estado, o seu caminho e um commit curto.

| Marca | Estado | Significado |
|---|---|---|
| `✓` | atualizado | em checkout no commit que o repositório regista |
| `↑` | modificado | em checkout noutro commit (avançou, ou mudou de branch sem o atualizar) |
| `○` | não clonado | declarado em `.gitmodules` e registado, mas a sua pasta está vazia: atualize-o para o clonar |
| `✗` | em conflito | o repositório tem conflitos de merge no submódulo |

![A secção SUBMÓDULOS com três linhas: vendor/almanac não clonado (○), vendor/ephemeris atualizado (✓) e vendor/starfield modificado (↑, noutro commit que não o registado), que aparece também como ficheiro alterado no painel Estado](../images/submodules.png)

| Fazer | Rato | Teclado |
|---|---|---|
| Abrir o submódulo num separador próprio | clique duplo na linha | selecioná-lo, `Enter` (paleta: `Refs: abrir submódulo num separador`) |
| Menu: **Atualizar (clonar se preciso)**, **Inicializar**, **Abrir em novo separador** | clique com o botão direito na linha | selecioná-lo, a tecla do menu de contexto (predefinição `.`) |
| Atualizar o submódulo selecionado (clona-o primeiro, se necessário) | menu | `Ctrl+P`: `Refs: atualizar submódulo (selecionado)` |
| Registá-lo em `.git/config` sem clonar | menu | `Ctrl+P`: `Refs: inicializar submódulo (selecionado)` |
| Atualizar todos os submódulos | | `Ctrl+P`: `Git: atualizar todos os submódulos` |
| Saltar para a secção | clicar no seu cabeçalho | `Ctrl+P`: `Navegação: focar Refs > Submódulos` |

![O menu de clique direito da linha modificada vendor/starfield: Atualizar (clonar se preciso), Inicializar e Abrir em novo separador](../images/submodule-menu.png)

Uma atualização pode fazer fetch do remoto do submódulo, por isso segue **Definições > SSH e credenciais > Credenciais HTTPS** como qualquer outra transferência. Um submódulo que não está clonado não tem nada para abrir como separador; a barra de estado diz-lho e aponta para Atualizar.

## Manter os submódulos atualizados

**Definições > Sincronização > Manter submódulos atualizados** (`general.keep_submodules_up_to_date`, desativado por predefinição). Quando está ativado, um **checkout** ou **pull** bem-sucedido é seguido de `git submodule update --init --recursive`. Isto só corre se um submódulo estiver fora de sintonia com o que o repositório regista agora, por isso um repositório sem submódulos não fica mais lento. Se essa atualização falhar, o checkout ou o pull aconteceu na mesma e o erro di-lo.

## A partir da linha de comandos

```
gitpoulpi submodule list                       # caminho, estado, url (--json: a lista completa)
gitpoulpi submodule init [<path>...]           # regista, não clona
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Sem caminhos, aplica-se a todos os submódulos. Os mesmos três são ferramentas MCP (`submodule_list` é uma leitura; `init` e `update` são mutações e seguem o `allow_mutations` de `[mcp]`).

## Notas

- Só o nível superior é listado; os submódulos de um submódulo aparecem quando o abre como separador (ou use `--recursive` para os atualizar).
- Versões recentes do git recusam clonar um submódulo de *caminho local* através do transporte `file`, a menos que `protocol.file.allow` o permita. Isto só interessa para submódulos cujo URL é uma pasta local; defina-o na configuração global do git se os usar.
