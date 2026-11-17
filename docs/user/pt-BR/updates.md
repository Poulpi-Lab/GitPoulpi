[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · [Français](../fr/updates.md) · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Atualizações

O GitPoulpi procura versões mais novas e, conforme a forma como foi instalado, ou substitui os próprios arquivos ou entrega a atualização ao programa que o instalou. Nada é substituído nem reiniciado sem uma ação sua.

## O chip e o painel

A barra de título mostra a versão em execução (`v1.0.0`) entre o nome do aplicativo e o caminho do repositório. Quando uma versão mais nova é conhecida, o chip vira `v0.2.0 disponível` na cor de destaque.

Pressione `Ctrl+N` e depois `u`, execute **Atualizações: mostrar painel de atualização** na paleta de comandos (`Ctrl+P`), ou clique no chip enquanto ele diz `disponível` (ou enquanto uma atualização está em andamento): o painel de atualização abre. Sem uma versão mais nova, um clique no chip abre a aba Notas da versão. Ele é uma janela sobre o aplicativo: `Esc`, `q`, o `✕` de fechar ou um clique fora dele o fecha. Ele mostra:

- a versão em execução e **Instalado com**: como o GitPoulpi chegou a este computador (isso decide tudo o que vem abaixo);
- em que pé está a atualização (verificando, baixando com uma porcentagem, pronta, com falha);
- os botões do procedimento desta instalação e o comando exato, quando há um;
- uma linha do seu administrador, quando ele definiu uma.

Todo botão tem uma tecla (impressa na linha de dica do painel) e um comando da paleta, então o painel funciona sem mouse:

| Botão | Tecla | Comando da paleta |
|---|---|---|
| **Atualizar e reiniciar** (ou **Executar atualização**) | `u` | Atualizações: atualizar agora |
| **Copiar comando** | `c` | Atualizações: copiar comando de atualização |
| **Abrir no terminal** | `t` | Atualizações: executar comando de atualização em um terminal |
| **Voltar para `<versão>`** | `v` | Atualizações: voltar à versão anterior |
| **Notas da versão** | `n` | Ajuda: Notas da versão |
| **Ignorar esta versão** | `s` | Atualizações: ignorar esta versão |
| **Abrir aviso** (só com um aviso de segurança) | `a` | Atualizações: abrir aviso de segurança |
| **Verificar agora** | `r` | Atualizações: verificar agora |
| **Fechar** | `Esc` | |

Um comando que não se aplica a esta instalação aparece esmaecido na paleta, com o motivo (por exemplo, "esta instalação é atualizada de outra forma: veja Atualizações: mostrar painel de atualização").

## O que acontece, por instalação

| Instalado com | Verifica em | O que o painel faz | Voltar atrás |
|---|---|---|---|
| Um arquivo de release, ou o `install.sh` | Releases do GitHub | **Atualizar e reiniciar**: o GitPoulpi baixa e verifica o arquivo, fecha, substitui os próprios arquivos e abre de novo no mesmo terminal. | **Voltar para `<versão>`** devolve os arquivos anteriores. |
| O instalador do Windows, só para mim | Releases do GitHub | O mesmo, sem nenhuma pergunta. No Windows ele reabre em uma nova janela. | **Voltar para `<versão>`** devolve os arquivos. |
| O instalador do Windows, todos os usuários | Releases do GitHub | O mesmo, mas o Windows pede aprovação de administrador. As sessões do GitPoulpi de outros usuários no computador são fechadas, o que perde o que eles digitaram e não salvaram. Se você recusar, nada muda. | **Voltar para `<versão>`** devolve os arquivos, com a mesma pergunta. |
| Chocolatey | O feed da comunidade do Chocolatey | **Executar atualização** executa `choco upgrade gitpoulpi -y` (o Windows pede aprovação de administrador). | O painel mostra `choco install gitpoulpi --version <anterior> --allow-downgrade -y`. |
| Chocolatey, feed interno | O feed da sua organização | **Executar atualização** executa `choco upgrade gitpoulpi -y --source=<feed>`. Os parâmetros do pacote não são enviados de novo, então uma configuração implantada fica intacta. | A mesma linha `choco install`, com o feed. |
| Scoop | Releases do GitHub | **Executar atualização** executa `scoop update gitpoulpi` depois que o GitPoulpi fechou por completo. | **Voltar para `<versão>`** executa `scoop reset gitpoulpi@<anterior>`. |
| Homebrew | Releases do GitHub | **Executar atualização** executa `brew upgrade gitpoulpi` neste terminal. | O painel mostra `brew install gitpoulpi@<anterior>`. |
| apt | Releases do GitHub | O painel mostra `sudo apt update && sudo apt install --only-upgrade gitpoulpi` com **Copiar comando** e **Abrir no terminal**. O GitPoulpi nunca executa `sudo` por você. Sem o repositório apt do GitPoulpi neste computador, ele manda baixar o novo `.deb`. | O painel mostra `sudo apt install gitpoulpi=<anterior>`. |
| Um pacote MSI | Releases do GitHub, ou a fonte do seu administrador | Nada a executar: a instalação é gerenciada pelo seu administrador, cuja mensagem o painel mostra. | Não é oferecido. |
| cargo | Releases do GitHub | O painel mostra `cargo binstall gitpoulpi-cli` quando o `cargo-binstall` está instalado, senão `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<versão> gitpoulpi-cli`. | O mesmo comando com a tag anterior. |
| Uma imagem Docker | Releases do GitHub | Nada a executar: uma imagem é substituída. O painel mostra `docker pull poulpilab/gitpoulpi:<versão>` e `:latest`. Veja [Executando no Docker](docker.md). | O painel mostra `docker pull poulpilab/gitpoulpi:<anterior>`. |
| Um arquivo de release em uma pasta em que você não pode gravar | Releases do GitHub | O painel diz que o GitPoulpi não consegue se substituir ali e aponta para **Baixar**. | Não é oferecido. |
| Uma build de desenvolvimento (`cargo build`) | nenhum | Nunca se atualiza. O chip mostra a versão e mais nada; o painel diz que não há registro de instalação. | Não é oferecido. |

As linhas de "mostrar" nunca executam nada sozinhas. **Abrir no terminal** abre o seu terminal com o comando já digitado, e você pressiona Enter.

As releases vêm sempre do repositório do projeto no GitHub, a menos que um administrador aponte a verificação para outro lugar (um feed interno, um espelho ou um `latest.json`); veja [Distribuição corporativa](enterprise-distribution.md). As notas da versão vêm da release ou do changelog embutido no GitPoulpi, o que estiver acessível.

## Notificar e automático

`updates.mode` ([Configurações > Sincronização](settings.md#sincronização)):

- **Notificar** (padrão): uma vez por dia o GitPoulpi verifica em segundo plano e mostra o chip. Nada é baixado até você pressionar o botão.
- **Automático**: onde o GitPoulpi substitui os próprios arquivos (arquivo de release, o instalador do Windows), ele também baixa e verifica a atualização em segundo plano assim que encontra uma. O chip passa a dizer **Reiniciar para atualizar**, e o reinício continua sendo uma escolha sua. Onde um gerenciador de pacotes é dono da atualização, **Automático** se comporta como **Notificar**.
- **Desativado**: nenhuma requisição é feita e o chip mostra só a versão.

A configuração **Canal de atualização** (`updates.channel`) escolhe entre **Estável** e **Pré-lançamento** (candidatas a lançamento também). Uma verificação iniciada por você com **Verificar agora** ignora o limite de uma vez por dia. Sem conexão de rede, ou enquanto o repositório é privado e nenhum token do github.com está guardado, uma verificação em segundo plano falha em silêncio, com uma linha no registro do aplicativo.

## Reiniciando e o que você vê depois

Pressionar **Atualizar e reiniciar** ou **Executar atualização** salva a sua sessão e fecha a janela. Um pequeno programa separado, o `gitpoulpi-updater`, faz o trabalho, para que os arquivos do próprio GitPoulpi não estejam em uso enquanto são substituídos e um gerenciador de pacotes não seja bloqueado por um processo pai à espera. Ele para primeiro os daemons do GitPoulpi em segundo plano (as outras janelas e servidores MCP do seu usuário também fecham), faz a atualização, confere se o novo `gitpoulpi --version` imprime a versão esperada e inicia o GitPoulpi de novo: no mesmo terminal no Linux e no macOS, em uma nova janela no Windows. Se a conferência falhar, ele devolve os arquivos antigos.

O próximo início mostra uma linha na barra de status:

- `Atualizado para a v1.4.2`
- `Falha na atualização: <motivo>. A versão anterior continua instalada.`
- `Atualização cancelada: a aprovação do administrador foi recusada. Nada mudou.`

Só uma atualização roda por vez; um arquivo de trava na pasta de dados do GitPoulpi (e outro na pasta de instalação, quando ela é gravável) impede uma segunda, e uma trava com mais de dez minutos é ignorada, então uma atualização que travou nunca bloqueia o GitPoulpi para sempre. A CLI e o daemon não iniciam enquanto existir uma trava recente.

## Voltar à versão anterior

Onde o GitPoulpi substitui os próprios arquivos, **Voltar para `<versão>`** devolve os arquivos anteriores e reinicia. Eles ficam como `<nome>.old` por um minuto depois que a nova versão iniciou e depois são excluídos, então voltar serve para a primeira execução depois de uma atualização. Onde um gerenciador de pacotes é dono da instalação, voltar é o comando da tabela: o painel o mostra ou o executa. Voltar altera os arquivos do próprio GitPoulpi, não um repositório, então não faz parte do histórico de desfazer do git.

## O que é verificado e recusado

Antes de substituir qualquer coisa, o GitPoulpi baixa o arquivo, a assinatura `.minisig` dele e o `.sha256`. Ele confere primeiro a assinatura, contra a chave de release do projeto embutida no GitPoulpi, e o comentário confiável da assinatura precisa nomear este arquivo e esta versão. Depois confere o hash. Ele recusa: uma release sem `.minisig` (uma release mais antiga, publicada antes de as releases serem assinadas, não pode ser instalada pelo atualizador, o que inclui fixar uma versão ou voltar para uma delas), uma assinatura de outra chave, uma assinatura de outro arquivo ou de outra versão, uma release sem `.sha256`, um hash que não confere, um arquivo cujo nome não é o desta plataforma (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), um arquivo cujo marcador nomeia outra versão ou plataforma e um link de download fora dos hosts de release do GitHub (cada salto de redirecionamento é conferido) ou do espelho que um administrador configurou. A troca é tudo ou nada: se outro programa estiver segurando um dos executáveis, o GitPoulpi nomeia o arquivo, não muda nada e deixa o download para outra tentativa. No Windows, uma atualização para todos os usuários é verificada de novo pela etapa elevada, que baixa por conta própria o `.minisig` e o `.sha256` e confere os dois outra vez, então um arquivo trocado na pasta de downloads do usuário nesse meio-tempo é recusado. Os gerenciadores de pacotes (Chocolatey, Scoop, Homebrew, apt) verificam os próprios pacotes: o GitPoulpi não baixa nada nesses casos.

## Avisos de segurança

Junto a cada release o projeto publica o `security.json`: a lista de avisos de segurança (um identificador, uma gravidade, as versões afetadas como um intervalo, por exemplo `>=1.2.0, <1.4.3`, a versão corrigida, um link e um resumo de uma frase). A mesma verificação diária que procura uma versão nova também o busca, do mesmo lugar da release: a página de release do repositório no caso do GitHub, a pasta do `latest.json` no caso de um espelho, o GitHub no caso do feed público do Chocolatey. Um aviso só é usado quando o `security.json.minisig` confere com a chave do projeto; um arquivo sem assinatura ou mal assinado nunca é mostrado (uma linha no Registro de atividade diz que ele foi ignorado).

Quando um aviso cita a versão em execução, o chip fica vermelho e diz **Aviso de segurança: esta versão é afetada**, em toda instalação, inclusive as gerenciadas por gerenciador de pacotes. O painel de atualização então lista cada aviso com sua gravidade, resumo, versão corrigida e link; **Abrir aviso** (`a`, ou **Atualizações: abrir aviso de segurança** na paleta) abre no navegador o link do mais grave. Onde um gerenciador de pacotes é dono da instalação, o painel acrescenta "Atualize com o seu gerenciador de pacotes."

Com a verificação de atualizações desativada (`updates.mode = "off"`), com uma política de administrador cujo `check` é `none` e em uma build feita com `cargo build`, nada é buscado e nenhum aviso é mostrado. O feed NuGet próprio de uma organização não faz o GitPoulpi contatar o GitHub: os avisos dele vêm só do espelho `download_base` da política.

## Configurações que um administrador travou

Um administrador pode definir o modo de atualização, o canal, uma versão fixada, onde verificar e se o GitPoulpi pode se atualizar, em uma política válida para a máquina inteira. Nas Configurações essas linhas mostram **Definido pelo seu administrador**, o painel mostra a mensagem dele e, quando a autoatualização está desligada, o painel oferece os comandos ou uma nota em vez de botões que substituem arquivos. Arquivos de política ilegíveis falham de forma fechada: só notificar, sem autoatualização. O lado do administrador está em [Distribuição corporativa](enterprise-distribution.md#o-arquivo-de-política-e-o-registro).

## Ambiente

`GITPOULPI_UPDATE_API` aponta a verificação para outra raiz de API (usado pelos testes).
