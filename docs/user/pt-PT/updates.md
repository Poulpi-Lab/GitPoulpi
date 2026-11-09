[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · **Português (Portugal)** · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Atualizações

O GitPoulpi procura versões mais recentes e, consoante a forma como foi instalado, ou substitui os seus próprios ficheiros ou entrega a atualização ao programa que o instalou. Nada é substituído nem reiniciado sem uma ação sua.

## O indicador e o painel

A barra de título mostra a versão em execução (`v1.0.0`) entre o nome da aplicação e o caminho do repositório. Quando se conhece uma versão mais recente, o indicador passa a `v0.2.0 disponível` na cor de destaque.

Prima `Ctrl+N` e depois `u`, execute **Atualizações: mostrar atualização** na paleta de comandos (`Ctrl+P`), ou clique no indicador enquanto diz `disponível` (ou enquanto uma atualização está em curso): abre-se o painel de atualizações. Sem versão mais recente, um clique no indicador abre antes o separador Notas de versão. É uma janela sobre a aplicação: `Esc`, `q`, a cruz de fechar ou um clique fora fecham-na. Mostra:

- a versão em execução e **Instalado com**: como o GitPoulpi chegou a este computador (é isso que decide tudo o que se segue);
- em que ponto está a atualização (a procurar, a transferir com uma percentagem, pronta, falhada);
- os botões do procedimento desta instalação e o comando exato, quando existe um;
- uma linha do seu administrador, quando este definiu uma.

Cada botão tem uma tecla (impressa na linha de ajuda do painel) e um comando da paleta, pelo que o painel funciona sem rato:

| Botão | Tecla | Comando da paleta |
|---|---|---|
| **Atualizar e reiniciar** (ou **Executar atualização**) | `u` | Atualizações: atualizar agora |
| **Copiar comando** | `c` | Atualizações: copiar comando de atualização |
| **Abrir num terminal** | `t` | Atualizações: executar comando de atualização num terminal |
| **Reverter** | `v` | Atualizações: reverter para a versão anterior |
| **Notas de versão** | `n` | Ajuda: notas de versão |
| **Ignorar esta versão** | `s` | Atualizações: ignorar esta versão |
| **Abrir aviso** (só com um aviso de segurança) | `a` | Atualizações: abrir aviso de segurança |
| **Procurar agora** | `r` | Atualizações: procurar atualizações agora |
| **Fechar** | `Esc` | |

Um comando que não se aplica a esta instalação aparece a cinzento na paleta, com o motivo (por exemplo "esta instalação é atualizada de outra forma: veja Atualizações: mostrar atualização").

## O que acontece, por instalação

| Instalado com | Verificado em | O que o painel faz | Voltar atrás |
|---|---|---|---|
| Um arquivo de lançamento, ou `install.sh` | Versões do GitHub | **Atualizar e reiniciar**: o GitPoulpi transfere e verifica o arquivo, fecha, substitui os seus ficheiros e abre de novo no mesmo terminal. | **Reverter** repõe os ficheiros anteriores. |
| O instalador do Windows, só para mim | Versões do GitHub | O mesmo, sem pedido de confirmação. No Windows reabre numa nova janela. | **Reverter** repõe-nos. |
| O instalador do Windows, todos os utilizadores | Versões do GitHub | O mesmo, mas o Windows pede aprovação de administrador. As sessões do GitPoulpi de outros utilizadores no computador são fechadas, o que perde o que estes tinham escrito e não guardado. Se recusar, nada muda. | **Reverter** repõe-nos, com o mesmo pedido. |
| Chocolatey | O feed da comunidade Chocolatey | **Executar atualização** executa `choco upgrade gitpoulpi -y` (o Windows pede aprovação de administrador). | O painel mostra `choco install gitpoulpi --version <anterior> --allow-downgrade -y`. |
| Chocolatey, feed interno | O feed da sua organização | **Executar atualização** executa `choco upgrade gitpoulpi -y --source=<feed>`. Os parâmetros do pacote não são enviados de novo, pelo que uma configuração implementada fica intacta. | A mesma linha `choco install` com o feed. |
| winget | Versões do GitHub | **Executar atualização** executa `winget upgrade --id GitPoulpi.GitPoulpi --exact --silent` com os acordos aceites. O próprio winget pede aprovação de administrador quando a instalação é para toda a máquina. | O painel mostra `winget install --id GitPoulpi.GitPoulpi --version <anterior>`. |
| Scoop | Versões do GitHub | **Executar atualização** executa `scoop update gitpoulpi` depois de o GitPoulpi ter fechado por completo. | **Reverter** executa `scoop reset gitpoulpi@<anterior>`. |
| Homebrew | Versões do GitHub | **Executar atualização** executa `brew upgrade gitpoulpi` neste terminal. | O painel mostra `brew install gitpoulpi@<anterior>`. |
| apt | Versões do GitHub | O painel mostra `sudo apt update && sudo apt install --only-upgrade gitpoulpi` com **Copiar comando** e **Abrir num terminal**. O GitPoulpi nunca executa `sudo` por si. Sem o repositório apt do GitPoulpi neste computador, diz para transferir antes o novo `.deb`. | O painel mostra `sudo apt install gitpoulpi=<anterior>`. |
| Um pacote MSI | Versões do GitHub, ou a origem do seu administrador | Nada a executar: a instalação é gerida pelo seu administrador, cuja mensagem o painel mostra. | Não é oferecido. |
| O AUR | Versões do GitHub | O painel mostra `yay -S gitpoulpi-bin` (ou `paru`, consoante o que encontrar primeiro; sem um auxiliar, os passos `git clone` e `makepkg -si`). | Não é oferecido. |
| cargo | Versões do GitHub | O painel mostra `cargo binstall gitpoulpi-cli` quando o `cargo-binstall` está instalado, senão `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<versão> gitpoulpi-cli`. | O mesmo comando com a etiqueta anterior. |
| Uma imagem Docker | Versões do GitHub | Nada a executar: uma imagem é substituída. O painel mostra `docker pull poulpilab/gitpoulpi:<versão>` e `:latest`. Veja [Executar no Docker](docker.md). | O painel mostra `docker pull poulpilab/gitpoulpi:<anterior>`. |
| Um arquivo de lançamento numa pasta onde não pode escrever | Versões do GitHub | O painel diz que o GitPoulpi não se consegue substituir ali e aponta para **Transferir**. | Não é oferecido. |
| Uma compilação de desenvolvimento (`cargo build`) | nenhuma | Nunca se atualiza. O indicador mostra a versão e mais nada; o painel diz que não há registo de instalação. | Não é oferecido. |

As linhas "mostra" nunca executam nada por si. **Abrir num terminal** abre o seu terminal com o comando já escrito, e prime Enter.

As versões vêm sempre do repositório do projeto no GitHub, a menos que um administrador aponte a verificação para outro sítio (um feed interno, um espelho ou um `latest.json`), veja [Distribuição empresarial](enterprise-distribution.md). As notas de versão vêm da versão ou do registo de alterações incorporado no GitPoulpi, conforme o que estiver acessível.

## Notificar e automático

`updates.mode` ([Definições > Sincronização](settings.md#sincronização)):

- **Notificar** (predefinição): uma vez por dia o GitPoulpi verifica em segundo plano e mostra o indicador. Nada é transferido até premir o botão.
- **Automático**: onde o GitPoulpi substitui os seus próprios ficheiros (arquivo de lançamento, instalador do Windows) também transfere e verifica a atualização em segundo plano assim que a encontra. O indicador passa então a **Reiniciar para atualizar**, e o reinício continua a ser uma escolha sua. Onde um gestor de pacotes é dono da atualização, **Automático** comporta-se como **Notificar**.
- **Desativado**: nunca é feito nenhum pedido e o indicador mostra apenas a versão.

A definição **Canal de atualização** (`updates.channel`) escolhe entre **Estável** e **Pré-lançamento** (também as versões candidatas). Uma verificação que inicie com **Procurar agora** ignora o limite de uma por dia. Sem ligação à rede, ou enquanto o repositório é privado e não há nenhum token de github.com guardado, uma verificação em segundo plano falha em silêncio, com uma linha no registo da aplicação.

## Reiniciar e o que vê depois

Premir **Atualizar e reiniciar** ou **Executar atualização** guarda a sua sessão e fecha a janela. Um pequeno programa à parte, `gitpoulpi-updater`, faz o trabalho, para que os ficheiros do próprio GitPoulpi não estejam em uso enquanto são substituídos e um gestor de pacotes não seja bloqueado por um processo pai à espera. Primeiro pára os daemons em segundo plano do GitPoulpi (as outras janelas e servidores MCP do seu utilizador fecham também), faz a atualização, verifica que o novo `gitpoulpi --version` mostra a versão esperada e inicia o GitPoulpi de novo: no mesmo terminal no Linux e no macOS, numa nova janela no Windows. Se a verificação falhar, repõe os ficheiros antigos.

O arranque seguinte mostra uma linha na barra de estado:

- `Atualizado para a v1.4.2`
- `A atualização falhou: <motivo>. A versão anterior continua instalada.`
- `Atualização cancelada: a aprovação do administrador foi recusada. Nada foi alterado.`

Só corre uma atualização de cada vez; um ficheiro de bloqueio na pasta de dados do GitPoulpi (e outro na pasta de instalação, quando é gravável) impede uma segunda, e um bloqueio com mais de dez minutos é ignorado, pelo que uma atualização interrompida nunca bloqueia o GitPoulpi para sempre. A CLI e o daemon não arrancam enquanto existir um bloqueio recente.

## Reverter

Onde o GitPoulpi substitui os seus próprios ficheiros, **Reverter** repõe os ficheiros anteriores e reinicia. Estes ficam como `<nome>.old` durante um minuto depois de a nova versão ter arrancado e depois são apagados, pelo que reverter serve para a primeira execução a seguir a uma atualização. Onde um gestor de pacotes é dono da instalação, reverter é o comando da tabela: o painel mostra-o ou executa-o. Reverter altera os ficheiros do próprio GitPoulpi, não um repositório, pelo que não faz parte do diário de anulação do git.

## O que é verificado e recusado

Antes de substituir seja o que for, o GitPoulpi transfere o arquivo, a sua assinatura `.minisig` e o seu `.sha256`. Verifica primeiro a assinatura, com a chave de versões do projeto incorporada no GitPoulpi, e o comentário de confiança da assinatura tem de indicar este arquivo e esta versão. Depois verifica o hash. Recusa: uma versão sem `.minisig` (uma versão mais antiga, publicada antes de as versões serem assinadas, não pode ser instalada pelo atualizador, o que inclui fixá-la ou voltar a ela), uma assinatura de outra chave, uma assinatura de outro ficheiro ou de outra versão, uma versão sem `.sha256`, um hash que não coincide, um arquivo cujo nome não é o desta plataforma (`gitpoulpi-<versão>-<target>.<zip|tar.gz>`), um arquivo cujo marcador indica outra versão ou plataforma, e uma ligação de transferência fora dos anfitriões de versões do GitHub (cada salto de redirecionamento é verificado) ou do espelho que um administrador configurou. A substituição é tudo ou nada: se outro programa estiver a usar um dos executáveis, o GitPoulpi indica o ficheiro, não altera nada e deixa a transferência para outra tentativa. No Windows, uma atualização para todos os utilizadores é verificada de novo pelo passo elevado, que transfere ele próprio o `.minisig` e o `.sha256` e verifica ambos outra vez, pelo que um ficheiro trocado entretanto na pasta de transferências do utilizador é recusado. Os gestores de pacotes (Chocolatey, winget, Scoop, Homebrew, apt, AUR) verificam os seus próprios pacotes: o GitPoulpi não transfere nada nesses casos.

## Avisos de segurança

Junto de cada versão, o projeto publica `security.json`: a lista de avisos de segurança (um identificador, uma gravidade, as versões afetadas como intervalo, por exemplo `>=1.2.0, <1.4.3`, a versão corrigida, uma ligação e um resumo de uma frase). A mesma verificação diária que procura uma nova versão obtém também este ficheiro, do mesmo sítio da versão: a página de versões do repositório no caso do GitHub, a pasta do `latest.json` no caso de um espelho, o GitHub no caso do feed público do Chocolatey. Um aviso só é usado quando `security.json.minisig` é validado com a chave do projeto; um ficheiro sem assinatura ou mal assinado nunca é mostrado (uma linha no Registo de atividade diz que foi ignorado).

Quando um aviso refere a versão em execução, o indicador fica vermelho e diz **Aviso de segurança: esta versão é afetada**, em todas as instalações, incluindo as geridas por um gestor de pacotes. O painel de atualizações lista então cada aviso com a sua gravidade, resumo, versão corrigida e ligação; **Abrir aviso** (`a`, ou **Atualizações: abrir aviso de segurança** na paleta) abre no navegador a ligação do aviso mais grave. Onde um gestor de pacotes é dono da instalação, o painel acrescenta "Atualize com o seu gestor de pacotes."

Com as verificações de atualização desativadas (`updates.mode = "off"`), com uma política de administrador cujo `check` é `none`, e numa compilação feita com `cargo build`, nada é obtido e nenhum aviso é mostrado. O feed NuGet próprio de uma organização não faz o GitPoulpi contactar o GitHub: os seus avisos vêm apenas do espelho `download_base` da política.

## Definições que um administrador bloqueou

Um administrador pode definir o modo de atualização, o canal, uma versão fixada, onde verificar e se o GitPoulpi se pode atualizar de todo, numa política válida para toda a máquina. Nas Definições essas linhas mostram **Definido pelo seu administrador**, o painel mostra a sua mensagem e, quando a autoatualização está desligada, o painel oferece os comandos ou uma nota em vez de botões que substituem ficheiros. Os ficheiros de política ilegíveis falham de forma segura: só notificar, sem autoatualização. O lado do administrador está em [Distribuição empresarial](enterprise-distribution.md#o-ficheiro-de-política-e-o-registo).

## Ambiente

`GITPOULPI_UPDATE_API` aponta a verificação para outra raiz de API (usada pelos testes).
