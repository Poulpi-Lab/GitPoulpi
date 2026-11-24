[English](../themes.md) · [Čeština](../cs/themes.md) · [Deutsch](../de/themes.md) · [Español](../es/themes.md) · [Français](../fr/themes.md) · [Italiano](../it/themes.md) · [日本語](../ja/themes.md) · [한국어](../ko/themes.md) · [Nederlands](../nl/themes.md) · [Polski](../pl/themes.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/themes.md) · [Русский](../ru/themes.md) · [Svenska](../sv/themes.md) · [Türkçe](../tr/themes.md) · [Українська](../uk/themes.md) · [简体中文](../zh-CN/themes.md)

# Temas

Um tema define cada cor que o GitPoulpi pinta: texto, painéis, seleção, botões, as
visualizações de diff e de merge, o editor, os ícones da barra de ferramentas e as
cores das faixas do grafo do Log. Troque de tema em **Configurações > Aparência >
Tema**, ou com `gitpoulpi config set ui.theme <slug>`. Para ver a aparência de cada um, vá direto para a [galeria](#galeria).

## Predefinições embutidas

`gitpoulpi theme list` imprime as que estão na sua compilação, na ordem em que a lista suspensa
das Configurações as mostra: **GitPoulpi Dark** (`dark`, pintado sobre o fundo do
próprio terminal) e **GitPoulpi Light** (`light`) primeiro, depois todas as outras
predefinições em ordem alfabética pelo rótulo. `ui.theme = "system"` (o padrão) segue
o seu terminal: Light em um terminal de fundo claro, Dark caso contrário (veja
“Detecção do sistema” abaixo); o antigo `default` equivale a `system`. A lista suspensa de
paletas de faixas usa a mesma ordem.

| Slug | Rótulo | Origem das cores |
|---|---|---|
| `system` (padrão) | Sistema | não é uma paleta: GitPoulpi Light com fundo de terminal claro, GitPoulpi Dark caso contrário |
| `dark` | GitPoulpi Dark | tema original do GitPoulpi, pintado sobre o fundo do próprio terminal |
| `light` | GitPoulpi Light | tema original do GitPoulpi |
| `catppuccin-latte` / `catppuccin-frappe` / `catppuccin-macchiato` / `catppuccin-mocha` | Catppuccin Latte / Frappé / Macchiato / Mocha | [github.com/catppuccin/palette](https://github.com/catppuccin/palette) |
| `cyberpunk` | Cyberpunk | [iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes) “Cyberpunk” |
| `dracula` | Dracula | [draculatheme.com/contribute](https://draculatheme.com/contribute) |
| `everforest-dark` | Everforest Dark | [github.com/sainnhe/everforest/blob/master/palette.md](https://github.com/sainnhe/everforest/blob/master/palette.md) |
| `fire` / `fire-light` | Fire / Fire Light | temas originais do GitPoulpi: Fire é brasa sobre um fundo laranja queimado com laranja-chama; Fire Light é o gêmeo claro dele, sobre um fundo laranja de verdade |
| `github-dark` / `github-light` | GitHub Dark / GitHub Light | Os temas Primer do GitHub, [github.com/primer/github-vscode-theme](https://github.com/primer/github-vscode-theme) com @primer/primitives (`canvas.default`, `canvas.inset` como a barra lateral, `fg.*`, `accent.fg`); nenhum valor ajustado |
| `gruvbox-dark` / `gruvbox-light` | Gruvbox Dark / Light | [github.com/morhetz/gruvbox#palette](https://github.com/morhetz/gruvbox#palette) |
| `kanagawa` / `kanagawa-lotus` | Kanagawa (wave) / Kanagawa Lotus | [github.com/rebelot/kanagawa.nvim](https://github.com/rebelot/kanagawa.nvim) |
| `matrix` | Matrix | [iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes) “Matrix” (sem vermelho no esquema: `danger` é `#ff5555`) |
| `monokai` | Monokai | a paleta Monokai original ([monokai.nl](https://monokai.nl/)) |
| `nord` | Nord | [www.nordtheme.com](https://www.nordtheme.com) |
| `one-dark` / `one-light` | One Dark / One Light | Atom One Dark / One Light ([One Dark](https://github.com/atom/one-dark-syntax), [One Light](https://github.com/atom/one-light-syntax)) |
| `rose-pine` / `rose-pine-moon` / `rose-pine-dawn` | Rosé Pine / Moon / Dawn | [rosepinetheme.com/palette](https://rosepinetheme.com/palette/) |
| `solarized-dark` / `solarized-light` | Solarized Dark / Light | Solarized, de Ethan Schoonover ([ethanschoonover.com/solarized](https://ethanschoonover.com/solarized/)) |
| `sunset` | Sunset | tema original do GitPoulpi: crepúsculo índigo com coral e dourado |
| `terminal` | Terminal (16 colors) | usa apenas as 16 cores do próprio terminal |
| `tokyo-night` / `tokyo-night-day` | Tokyo Night (night) / Tokyo Night Day | [github.com/folke/tokyonight.nvim](https://github.com/folke/tokyonight.nvim) (o primeiro plano do Day é escurecido para passar nas regras de contraste) |
| `vesper` | Vesper | [github.com/raunofreiberg/vesper](https://github.com/raunofreiberg/vesper) (sem azul nem roxo na paleta: o destaque é pêssego) |
| `vscode-dark` / `vscode-light` | VS Code Dark / VS Code Light | Dark Modern / Light Modern padrão do VS Code, [github.com/microsoft/vscode/tree/main/extensions/theme-defaults/themes](https://github.com/microsoft/vscode/tree/main/extensions/theme-defaults/themes) (`dark_modern.json`, `light_modern.json`, mais os padrões de cor do VS Code para a seleção de listas e os destaques ANSI/de gráficos; o cinza atenuado do Light é `616161`) |

### Importados de terminalcolors.com

Importados das paletas de terminal do terminalcolors.com; cada esquema pertence a seus autores. 90 predefinições são construídas a partir do fundo, do primeiro plano, da seleção e das 16 cores ANSI da paleta (accent = azul, success = verde, warning = amarelo, danger = vermelho, AI = magenta, muted = preto brilhante), por uma derivação compartilhada por todas; uma cor que falha nas regras de contraste do GitPoulpi é clareada ou escurecida até passar. A tabela é gerada por `scripts/gen-terminalcolors.py` em `crates/gitpoulpi-config/src/theme/terminalcolors_data.rs`. Os slugs são `<família>-<variante>` (`default` omitido); os rótulos ficam em ordem alfabética na lista suspensa como os demais.

| Família | Slugs |
|---|---|
| [Apprentice](https://terminalcolors.com/themes/apprentice/) | `apprentice` |
| [Ayu](https://terminalcolors.com/themes/ayu/) | `ayu-dark`, `ayu-light`, `ayu-mirage` |
| [Cobalt2](https://terminalcolors.com/themes/cobalt2/) | `cobalt2` |
| [Deus](https://terminalcolors.com/themes/deus/) | `deus` |
| [Dracula](https://terminalcolors.com/themes/dracula/) | `dracula-soft` |
| [Everforest](https://terminalcolors.com/themes/everforest/) | `everforest-light` |
| [GitHub](https://terminalcolors.com/themes/github/) | `github-dark-colorblind`, `github-dark-dimmed`, `github-dark-high-contrast`, `github-dark-legacy`, `github-light-colorblind`, `github-light-high-contrast`, `github-light-legacy` |
| [Gotham](https://terminalcolors.com/themes/gotham/) | `gotham` |
| [Gruvbox](https://terminalcolors.com/themes/gruvbox/) | `gruvbox-dark-hard-contrast`, `gruvbox-dark-soft-contrast`, `gruvbox-light-hard-contrast`, `gruvbox-light-soft-contrast` |
| [Iceberg](https://terminalcolors.com/themes/iceberg/) | `iceberg-dark`, `iceberg-light` |
| [Jellybeans](https://terminalcolors.com/themes/jellybeans/) | `jellybeans` |
| [Kanagawa](https://terminalcolors.com/themes/kanagawa/) | `kanagawa-dragon` |
| [Lucario](https://terminalcolors.com/themes/lucario/) | `lucario` |
| [Miasma](https://terminalcolors.com/themes/miasma/) | `miasma` |
| [Moonfly](https://terminalcolors.com/themes/moonfly/) | `moonfly` |
| [Night](https://terminalcolors.com/themes/night-owl/) | `night-owl-dark`, `night-owl-light` |
| [Nightfly](https://terminalcolors.com/themes/nightfly/) | `nightfly` |
| [Nightfox](https://terminalcolors.com/themes/nightfox/) | `nightfox`, `nightfox-carbonfox`, `nightfox-dawnfox`, `nightfox-dayfox`, `nightfox-duskfox`, `nightfox-nordfox`, `nightfox-terafox` |
| [Noctis](https://terminalcolors.com/themes/noctis/) | `noctis`, `noctis-azureus`, `noctis-bordo`, `noctis-hibernus`, `noctis-lilac`, `noctis-lux`, `noctis-minimus`, `noctis-obscuro`, `noctis-sereno`, `noctis-uva`, `noctis-viola` |
| [Nordic](https://terminalcolors.com/themes/nordic/) | `nordic` |
| [One](https://terminalcolors.com/themes/one-half/) | `one-half-dark`, `one-half-light` |
| [Panda](https://terminalcolors.com/themes/panda/) | `panda` |
| [Posterpole](https://terminalcolors.com/themes/posterpole/) | `posterpole`, `posterpole-gray` |
| [Seoul256](https://terminalcolors.com/themes/seoul256/) | `seoul256-dark`, `seoul256-light` |
| [Shades](https://terminalcolors.com/themes/shades-of-purple/) | `shades-of-purple`, `shades-of-purple-super-dark` |
| [Sonokai](https://terminalcolors.com/themes/sonokai/) | `sonokai`, `sonokai-andromeda`, `sonokai-atlantis`, `sonokai-espresso`, `sonokai-maia`, `sonokai-shusia` |
| [Srcery](https://terminalcolors.com/themes/srcery/) | `srcery` |
| [Tender](https://terminalcolors.com/themes/tender/) | `tender` |
| [Tokyo](https://terminalcolors.com/themes/tokyo-night/) | `tokyo-night-moon`, `tokyo-night-storm` |
| [Tomorrow](https://terminalcolors.com/themes/tomorrow/) | `tomorrow`, `tomorrow-night`, `tomorrow-night-blue`, `tomorrow-night-bright`, `tomorrow-night-eighties` |
| [Zenbones](https://terminalcolors.com/themes/zenbones/) | `zenbones-duckbones`, `zenbones-forestbones-dark`, `zenbones-forestbones-light`, `zenbones-kanagawabones`, `zenbones-neobones-dark`, `zenbones-neobones-light`, `zenbones-nordbones`, `zenbones-rosebones-dark`, `zenbones-rosebones-light`, `zenbones-seoulbones-dark`, `zenbones-seoulbones-light`, `zenbones-tokyobones-dark`, `zenbones-tokyobones-light`, `zenbones-vimbones`, `zenbones-zenburned`, `zenbones-zenwritten-dark`, `zenbones-zenwritten-light` |

`ui.side_bg` é o fundo do painel Refs e do painel da direita (detalhes do commit,
alterações em andamento, listas de arquivos), como o GitHub e o VS Code
pintam suas barras laterais. Catppuccin (seu `mantle`),
GitHub e VS Code o definem; em toda outra predefinição ele é `default`, o que
significa o mesmo que `ui.bg`. Um arquivo de tema pode definir `side_bg` em `[ui]`
(`gitpoulpi theme export` o imprime).

Mais três papéis em `[ui]` colorem o chrome, como fazem o GitHub e o VS Code:

| Papel | Onde pinta |
|---|---|
| `ui.toolbar_bg` | A linha da barra de ferramentas, e a **aba de repositório selecionada**, para que a aba pareça ligada à barra de ferramentas. Os chips de botão da barra de ferramentas mantêm o próprio `chrome.toolbar_bg`. |
| `ui.statusbar_bg` | A linha de status (embaixo, a menos que `ui.statusbar_position` diga top). |
| `ui.statusbar_fg` | O texto da linha de status. O texto que não seria legível sobre esse preenchimento é repintado com ele; texto de aviso e de erro que é legível mantém sua cor. |

As predefinições os definem para combinar com seu aplicativo: VS Code Dark e Light a
linha de status azul (`#007acc`) com texto branco e os cinzas da barra de título do
VS Code para a barra de ferramentas; GitHub os tons do seu cabeçalho. Toda outra
predefinição (inclusive as paletas de terminal importadas) deriva uma barra de
ferramentas um passo afastada de `bg` e uma linha de status um pouco afastada dela,
com `text` sobre ela, e as regras de contraste mantêm o texto legível. `dark`
(GitPoulpi Dark) e `terminal` usam o fundo do próprio terminal, então os três são
`default` ali: nada é pintado e a aba selecionada mantém a cor de seleção. Um arquivo
de tema que os omite herda os valores da sua base; `default` significa sem tom (a
linha de status então segue `side_bg`).

Onde o cinza de comentário ou atenuado de um esquema é escuro demais para ler, ele é
clareado até passar nas regras de contraste do GitPoulpi; os tons de painel e de
seleção que o esquema não tem são derivados do seu fundo.

### Galeria

Todo tema embutido, no mesmo repositório com um commit de merge selecionado (`scripts/docs-screenshots.sh` as refaz).

| **GitPoulpi Dark** (`dark`) | **GitPoulpi Light** (`light`) | **Apprentice** (`apprentice`) |
|---|---|---|
| ![A janela principal com o tema GitPoulpi Dark](../images/theme-dark.png) | ![A janela principal com o tema GitPoulpi Light](../images/theme-light.png) | ![A janela principal com o tema Apprentice](../images/theme-apprentice.png) |
| **Ayu Dark** (`ayu-dark`) | **Ayu Light** (`ayu-light`) | **Ayu Mirage** (`ayu-mirage`) |
| ![A janela principal com o tema Ayu Dark](../images/theme-ayu-dark.png) | ![A janela principal com o tema Ayu Light](../images/theme-ayu-light.png) | ![A janela principal com o tema Ayu Mirage](../images/theme-ayu-mirage.png) |
| **Catppuccin Frappé** (`catppuccin-frappe`) | **Catppuccin Latte** (`catppuccin-latte`) | **Catppuccin Macchiato** (`catppuccin-macchiato`) |
| ![A janela principal com o tema Catppuccin Frappé](../images/theme-catppuccin-frappe.png) | ![A janela principal com o tema Catppuccin Latte](../images/theme-catppuccin-latte.png) | ![A janela principal com o tema Catppuccin Macchiato](../images/theme-catppuccin-macchiato.png) |
| **Catppuccin Mocha** (`catppuccin-mocha`) | **Cobalt2** (`cobalt2`) | **Cyberpunk** (`cyberpunk`) |
| ![A janela principal com o tema Catppuccin Mocha](../images/theme-catppuccin-mocha.png) | ![A janela principal com o tema Cobalt2](../images/theme-cobalt2.png) | ![A janela principal com o tema Cyberpunk](../images/theme-cyberpunk.png) |
| **Deus** (`deus`) | **Dracula** (`dracula`) | **Dracula Soft** (`dracula-soft`) |
| ![A janela principal com o tema Deus](../images/theme-deus.png) | ![A janela principal com o tema Dracula](../images/theme-dracula.png) | ![A janela principal com o tema Dracula Soft](../images/theme-dracula-soft.png) |
| **Everforest Dark** (`everforest-dark`) | **Everforest Light** (`everforest-light`) | **Fire** (`fire`) |
| ![A janela principal com o tema Everforest Dark](../images/theme-everforest-dark.png) | ![A janela principal com o tema Everforest Light](../images/theme-everforest-light.png) | ![A janela principal com o tema Fire](../images/theme-fire.png) |
| **Fire Light** (`fire-light`) | **GitHub Dark** (`github-dark`) | **GitHub Dark Colorblind** (`github-dark-colorblind`) |
| ![A janela principal com o tema Fire Light](../images/theme-fire-light.png) | ![A janela principal com o tema GitHub Dark](../images/theme-github-dark.png) | ![A janela principal com o tema GitHub Dark Colorblind](../images/theme-github-dark-colorblind.png) |
| **GitHub Dark Dimmed** (`github-dark-dimmed`) | **GitHub Dark High Contrast** (`github-dark-high-contrast`) | **GitHub Dark Legacy** (`github-dark-legacy`) |
| ![A janela principal com o tema GitHub Dark Dimmed](../images/theme-github-dark-dimmed.png) | ![A janela principal com o tema GitHub Dark High Contrast](../images/theme-github-dark-high-contrast.png) | ![A janela principal com o tema GitHub Dark Legacy](../images/theme-github-dark-legacy.png) |
| **GitHub Light** (`github-light`) | **GitHub Light Colorblind** (`github-light-colorblind`) | **GitHub Light High Contrast** (`github-light-high-contrast`) |
| ![A janela principal com o tema GitHub Light](../images/theme-github-light.png) | ![A janela principal com o tema GitHub Light Colorblind](../images/theme-github-light-colorblind.png) | ![A janela principal com o tema GitHub Light High Contrast](../images/theme-github-light-high-contrast.png) |
| **GitHub Light Legacy** (`github-light-legacy`) | **Gotham** (`gotham`) | **Gruvbox Dark** (`gruvbox-dark`) |
| ![A janela principal com o tema GitHub Light Legacy](../images/theme-github-light-legacy.png) | ![A janela principal com o tema Gotham](../images/theme-gotham.png) | ![A janela principal com o tema Gruvbox Dark](../images/theme-gruvbox-dark.png) |
| **Gruvbox Dark Hard Contrast** (`gruvbox-dark-hard-contrast`) | **Gruvbox Dark Soft Contrast** (`gruvbox-dark-soft-contrast`) | **Gruvbox Light** (`gruvbox-light`) |
| ![A janela principal com o tema Gruvbox Dark Hard Contrast](../images/theme-gruvbox-dark-hard-contrast.png) | ![A janela principal com o tema Gruvbox Dark Soft Contrast](../images/theme-gruvbox-dark-soft-contrast.png) | ![A janela principal com o tema Gruvbox Light](../images/theme-gruvbox-light.png) |
| **Gruvbox Light Hard Contrast** (`gruvbox-light-hard-contrast`) | **Gruvbox Light Soft Contrast** (`gruvbox-light-soft-contrast`) | **Iceberg Dark** (`iceberg-dark`) |
| ![A janela principal com o tema Gruvbox Light Hard Contrast](../images/theme-gruvbox-light-hard-contrast.png) | ![A janela principal com o tema Gruvbox Light Soft Contrast](../images/theme-gruvbox-light-soft-contrast.png) | ![A janela principal com o tema Iceberg Dark](../images/theme-iceberg-dark.png) |
| **Iceberg Light** (`iceberg-light`) | **Jellybeans** (`jellybeans`) | **Kanagawa** (`kanagawa`) |
| ![A janela principal com o tema Iceberg Light](../images/theme-iceberg-light.png) | ![A janela principal com o tema Jellybeans](../images/theme-jellybeans.png) | ![A janela principal com o tema Kanagawa](../images/theme-kanagawa.png) |
| **Kanagawa Dragon** (`kanagawa-dragon`) | **Kanagawa Lotus** (`kanagawa-lotus`) | **Lucario** (`lucario`) |
| ![A janela principal com o tema Kanagawa Dragon](../images/theme-kanagawa-dragon.png) | ![A janela principal com o tema Kanagawa Lotus](../images/theme-kanagawa-lotus.png) | ![A janela principal com o tema Lucario](../images/theme-lucario.png) |
| **Matrix** (`matrix`) | **Miasma** (`miasma`) | **Monokai** (`monokai`) |
| ![A janela principal com o tema Matrix](../images/theme-matrix.png) | ![A janela principal com o tema Miasma](../images/theme-miasma.png) | ![A janela principal com o tema Monokai](../images/theme-monokai.png) |
| **Moonfly** (`moonfly`) | **Night Owl Dark** (`night-owl-dark`) | **Night Owl Light** (`night-owl-light`) |
| ![A janela principal com o tema Moonfly](../images/theme-moonfly.png) | ![A janela principal com o tema Night Owl Dark](../images/theme-night-owl-dark.png) | ![A janela principal com o tema Night Owl Light](../images/theme-night-owl-light.png) |
| **Nightfly** (`nightfly`) | **Nightfox** (`nightfox`) | **Nightfox Carbonfox** (`nightfox-carbonfox`) |
| ![A janela principal com o tema Nightfly](../images/theme-nightfly.png) | ![A janela principal com o tema Nightfox](../images/theme-nightfox.png) | ![A janela principal com o tema Nightfox Carbonfox](../images/theme-nightfox-carbonfox.png) |
| **Nightfox Dawnfox** (`nightfox-dawnfox`) | **Nightfox Dayfox** (`nightfox-dayfox`) | **Nightfox Duskfox** (`nightfox-duskfox`) |
| ![A janela principal com o tema Nightfox Dawnfox](../images/theme-nightfox-dawnfox.png) | ![A janela principal com o tema Nightfox Dayfox](../images/theme-nightfox-dayfox.png) | ![A janela principal com o tema Nightfox Duskfox](../images/theme-nightfox-duskfox.png) |
| **Nightfox Nordfox** (`nightfox-nordfox`) | **Nightfox Terafox** (`nightfox-terafox`) | **Noctis** (`noctis`) |
| ![A janela principal com o tema Nightfox Nordfox](../images/theme-nightfox-nordfox.png) | ![A janela principal com o tema Nightfox Terafox](../images/theme-nightfox-terafox.png) | ![A janela principal com o tema Noctis](../images/theme-noctis.png) |
| **Noctis Azureus** (`noctis-azureus`) | **Noctis Bordo** (`noctis-bordo`) | **Noctis Hibernus** (`noctis-hibernus`) |
| ![A janela principal com o tema Noctis Azureus](../images/theme-noctis-azureus.png) | ![A janela principal com o tema Noctis Bordo](../images/theme-noctis-bordo.png) | ![A janela principal com o tema Noctis Hibernus](../images/theme-noctis-hibernus.png) |
| **Noctis Lilac** (`noctis-lilac`) | **Noctis Lux** (`noctis-lux`) | **Noctis Minimus** (`noctis-minimus`) |
| ![A janela principal com o tema Noctis Lilac](../images/theme-noctis-lilac.png) | ![A janela principal com o tema Noctis Lux](../images/theme-noctis-lux.png) | ![A janela principal com o tema Noctis Minimus](../images/theme-noctis-minimus.png) |
| **Noctis Obscuro** (`noctis-obscuro`) | **Noctis Sereno** (`noctis-sereno`) | **Noctis Uva** (`noctis-uva`) |
| ![A janela principal com o tema Noctis Obscuro](../images/theme-noctis-obscuro.png) | ![A janela principal com o tema Noctis Sereno](../images/theme-noctis-sereno.png) | ![A janela principal com o tema Noctis Uva](../images/theme-noctis-uva.png) |
| **Noctis Viola** (`noctis-viola`) | **Nord** (`nord`) | **Nordic** (`nordic`) |
| ![A janela principal com o tema Noctis Viola](../images/theme-noctis-viola.png) | ![A janela principal com o tema Nord](../images/theme-nord.png) | ![A janela principal com o tema Nordic](../images/theme-nordic.png) |
| **One Dark** (`one-dark`) | **One Half Dark** (`one-half-dark`) | **One Half Light** (`one-half-light`) |
| ![A janela principal com o tema One Dark](../images/theme-one-dark.png) | ![A janela principal com o tema One Half Dark](../images/theme-one-half-dark.png) | ![A janela principal com o tema One Half Light](../images/theme-one-half-light.png) |
| **One Light** (`one-light`) | **Panda** (`panda`) | **Posterpole** (`posterpole`) |
| ![A janela principal com o tema One Light](../images/theme-one-light.png) | ![A janela principal com o tema Panda](../images/theme-panda.png) | ![A janela principal com o tema Posterpole](../images/theme-posterpole.png) |
| **Posterpole Gray** (`posterpole-gray`) | **Rosé Pine** (`rose-pine`) | **Rosé Pine Dawn** (`rose-pine-dawn`) |
| ![A janela principal com o tema Posterpole Gray](../images/theme-posterpole-gray.png) | ![A janela principal com o tema Rosé Pine](../images/theme-rose-pine.png) | ![A janela principal com o tema Rosé Pine Dawn](../images/theme-rose-pine-dawn.png) |
| **Rosé Pine Moon** (`rose-pine-moon`) | **Seoul256 Dark** (`seoul256-dark`) | **Seoul256 Light** (`seoul256-light`) |
| ![A janela principal com o tema Rosé Pine Moon](../images/theme-rose-pine-moon.png) | ![A janela principal com o tema Seoul256 Dark](../images/theme-seoul256-dark.png) | ![A janela principal com o tema Seoul256 Light](../images/theme-seoul256-light.png) |
| **Shades of Purple** (`shades-of-purple`) | **Shades of Purple Super Dark** (`shades-of-purple-super-dark`) | **Solarized Dark** (`solarized-dark`) |
| ![A janela principal com o tema Shades of Purple](../images/theme-shades-of-purple.png) | ![A janela principal com o tema Shades of Purple Super Dark](../images/theme-shades-of-purple-super-dark.png) | ![A janela principal com o tema Solarized Dark](../images/theme-solarized-dark.png) |
| **Solarized Light** (`solarized-light`) | **Sonokai** (`sonokai`) | **Sonokai Andromeda** (`sonokai-andromeda`) |
| ![A janela principal com o tema Solarized Light](../images/theme-solarized-light.png) | ![A janela principal com o tema Sonokai](../images/theme-sonokai.png) | ![A janela principal com o tema Sonokai Andromeda](../images/theme-sonokai-andromeda.png) |
| **Sonokai Atlantis** (`sonokai-atlantis`) | **Sonokai Espresso** (`sonokai-espresso`) | **Sonokai Maia** (`sonokai-maia`) |
| ![A janela principal com o tema Sonokai Atlantis](../images/theme-sonokai-atlantis.png) | ![A janela principal com o tema Sonokai Espresso](../images/theme-sonokai-espresso.png) | ![A janela principal com o tema Sonokai Maia](../images/theme-sonokai-maia.png) |
| **Sonokai Shusia** (`sonokai-shusia`) | **Srcery** (`srcery`) | **Sunset** (`sunset`) |
| ![A janela principal com o tema Sonokai Shusia](../images/theme-sonokai-shusia.png) | ![A janela principal com o tema Srcery](../images/theme-srcery.png) | ![A janela principal com o tema Sunset](../images/theme-sunset.png) |
| **Tender** (`tender`) | **Terminal (16 colors)** (`terminal`) | **Tokyo Night** (`tokyo-night`) |
| ![A janela principal com o tema Tender](../images/theme-tender.png) | ![A janela principal com o tema Terminal (16 colors)](../images/theme-terminal.png) | ![A janela principal com o tema Tokyo Night](../images/theme-tokyo-night.png) |
| **Tokyo Night Day** (`tokyo-night-day`) | **Tokyo Night Moon** (`tokyo-night-moon`) | **Tokyo Night Storm** (`tokyo-night-storm`) |
| ![A janela principal com o tema Tokyo Night Day](../images/theme-tokyo-night-day.png) | ![A janela principal com o tema Tokyo Night Moon](../images/theme-tokyo-night-moon.png) | ![A janela principal com o tema Tokyo Night Storm](../images/theme-tokyo-night-storm.png) |
| **Tomorrow** (`tomorrow`) | **Tomorrow Night** (`tomorrow-night`) | **Tomorrow Night Blue** (`tomorrow-night-blue`) |
| ![A janela principal com o tema Tomorrow](../images/theme-tomorrow.png) | ![A janela principal com o tema Tomorrow Night](../images/theme-tomorrow-night.png) | ![A janela principal com o tema Tomorrow Night Blue](../images/theme-tomorrow-night-blue.png) |
| **Tomorrow Night Bright** (`tomorrow-night-bright`) | **Tomorrow Night Eighties** (`tomorrow-night-eighties`) | **Vesper** (`vesper`) |
| ![A janela principal com o tema Tomorrow Night Bright](../images/theme-tomorrow-night-bright.png) | ![A janela principal com o tema Tomorrow Night Eighties](../images/theme-tomorrow-night-eighties.png) | ![A janela principal com o tema Vesper](../images/theme-vesper.png) |
| **VS Code Dark** (`vscode-dark`) | **VS Code Light** (`vscode-light`) | **Zenbones Duckbones** (`zenbones-duckbones`) |
| ![A janela principal com o tema VS Code Dark](../images/theme-vscode-dark.png) | ![A janela principal com o tema VS Code Light](../images/theme-vscode-light.png) | ![A janela principal com o tema Zenbones Duckbones](../images/theme-zenbones-duckbones.png) |
| **Zenbones Forestbones Dark** (`zenbones-forestbones-dark`) | **Zenbones Forestbones Light** (`zenbones-forestbones-light`) | **Zenbones Kanagawabones** (`zenbones-kanagawabones`) |
| ![A janela principal com o tema Zenbones Forestbones Dark](../images/theme-zenbones-forestbones-dark.png) | ![A janela principal com o tema Zenbones Forestbones Light](../images/theme-zenbones-forestbones-light.png) | ![A janela principal com o tema Zenbones Kanagawabones](../images/theme-zenbones-kanagawabones.png) |
| **Zenbones Neobones Dark** (`zenbones-neobones-dark`) | **Zenbones Neobones Light** (`zenbones-neobones-light`) | **Zenbones Nordbones** (`zenbones-nordbones`) |
| ![A janela principal com o tema Zenbones Neobones Dark](../images/theme-zenbones-neobones-dark.png) | ![A janela principal com o tema Zenbones Neobones Light](../images/theme-zenbones-neobones-light.png) | ![A janela principal com o tema Zenbones Nordbones](../images/theme-zenbones-nordbones.png) |
| **Zenbones Rosebones Dark** (`zenbones-rosebones-dark`) | **Zenbones Rosebones Light** (`zenbones-rosebones-light`) | **Zenbones Seoulbones Dark** (`zenbones-seoulbones-dark`) |
| ![A janela principal com o tema Zenbones Rosebones Dark](../images/theme-zenbones-rosebones-dark.png) | ![A janela principal com o tema Zenbones Rosebones Light](../images/theme-zenbones-rosebones-light.png) | ![A janela principal com o tema Zenbones Seoulbones Dark](../images/theme-zenbones-seoulbones-dark.png) |
| **Zenbones Seoulbones Light** (`zenbones-seoulbones-light`) | **Zenbones Tokyobones Dark** (`zenbones-tokyobones-dark`) | **Zenbones Tokyobones Light** (`zenbones-tokyobones-light`) |
| ![A janela principal com o tema Zenbones Seoulbones Light](../images/theme-zenbones-seoulbones-light.png) | ![A janela principal com o tema Zenbones Tokyobones Dark](../images/theme-zenbones-tokyobones-dark.png) | ![A janela principal com o tema Zenbones Tokyobones Light](../images/theme-zenbones-tokyobones-light.png) |
| **Zenbones Vimbones** (`zenbones-vimbones`) | **Zenbones Zenburned** (`zenbones-zenburned`) | **Zenbones Zenwritten Dark** (`zenbones-zenwritten-dark`) |
| ![A janela principal com o tema Zenbones Vimbones](../images/theme-zenbones-vimbones.png) | ![A janela principal com o tema Zenbones Zenburned](../images/theme-zenbones-zenburned.png) | ![A janela principal com o tema Zenbones Zenwritten Dark](../images/theme-zenbones-zenwritten-dark.png) |
| **Zenbones Zenwritten Light** (`zenbones-zenwritten-light`) |
| ![A janela principal com o tema Zenbones Zenwritten Light](../images/theme-zenbones-zenwritten-light.png) |

## Qualquer tema com qualquer paleta de faixas

Todo tema traz suas próprias cores de faixa para o grafo do Log, mas você pode
combinar qualquer tema com qualquer paleta: **Configurações > Aparência > Tema de
cores do grafo** oferece **Seguir o tema** (o padrão) mais toda paleta de faixas de
todo tema (GitPoulpi Dark, GitPoulpi Light, depois Material, Serie, ... e o
`graph_palette` dos seus próprios arquivos de tema, listados depois das predefinições sob o
nome do tema). Um tema personalizado aparece na próxima vez que a lista abre (os
arquivos são relidos então), e escolher um grava o nome do arquivo dele em
`ui.graph_color_theme` (para a instalação primária em `[ui]`, para as
outras na própria entrada `[installation.settings.<id>]`, como o tema). Um
arquivo de tema sem `graph_palette` tem as cores de faixa do seu `base` (a
predefinição padrão quando ele não nomeia nenhum), a mesma herança de toda outra cor
do arquivo. Um tema personalizado com o nome de uma paleta de faixas embutida (por exemplo
`gruvbox`) substitui essa paleta na lista. Se o nome configurado
não existe mais (o arquivo foi excluído ou renomeado), o grafo usa as cores de faixa do
tema ativo, e as Configurações mostram o nome marcado como *não encontrado* até você
escolher outro. ![A lista suspensa Tema de cores do grafo em Aparência: Seguir o tema, depois cada paleta de faixas com algumas de suas cores](../images/settings-log-graph-lanes.png)

No `config.toml`:

```toml
[ui]
theme = "nord"
graph_color_theme = "material"   # ou "theme" para seguir o tema
```

`ui.input_color` é o mesmo tipo de substituição para os campos de texto: o padrão
`theme` segue as cores de campo do tema; `gray`, `blue`, `slate`, `dark` e `custom`
forçam um fundo (são cores escuras, então escolha `theme` em um tema claro).

## Seus próprios temas

Coloque um arquivo `.toml` na pasta `themes/` do diretório de configuração
(`gitpoulpi config path` imprime o pai dela; a pasta é `${config}/themes/`). O nome
do arquivo sem `.toml` é o slug do tema, que é o que `ui.theme` aceita. Um arquivo
não pode reutilizar o slug de uma predefinição (ele é ignorado com um aviso).

Comece de um arquivo completo:

```
gitpoulpi theme export nord > "$(dirname "$(gitpoulpi config path)")/themes/mine.toml"
```

Depois mantenha só o que você muda. Um arquivo é um conjunto de **substituições
parciais** sobre uma `base` (uma predefinição, ou outro tema de usuário; as cadeias
têm no máximo 4 níveis e ciclos são rejeitados; sem `base` significa `dark`):

```toml
name = "My Nord"            # o rótulo na lista suspensa; padrão = nome do arquivo
base = "nord"
light = false               # herdado quando ausente
syntax = "base16-ocean.dark"
graph_palette = ["#88c0d0", "#a3be8c"]

[ui]
bg = "#2e3440"              # "default" | "#rrggbb" | "ansi:N"
accent = "#88c0d0"
toolbar_bg = "#3b4252"      # linha da barra de ferramentas e aba selecionada
statusbar_bg = "#5e81ac"    # status line
statusbar_fg = "#eceff4"

[diff]
add_bg = "#2f3b2f"
```

### Palavras alteradas em um diff (`diff.add_emph_bg`, `diff.del_emph_bg`)

Os fundos mais fortes que a visualização de Diff pinta nas palavras que diferem dentro
de uma linha alterada, sobre `diff.add_bg` (linhas adicionadas) e `diff.del_bg` (linhas
removidas); o texto mantém `diff.add_fg` e `diff.del_fg`. Toda predefinição define os dois.
Um arquivo de tema que define `diff.add_bg` ou `diff.del_bg` mas não a ênfase
correspondente recebe uma derivada dele (30% do caminho em direção à cor do texto
daquele lado), então um fundo personalizado nunca acaba com a ênfase do
tema base. Um valor `default` (a predefinição Terminal), ou um terminal cuja
profundidade de cor desenharia os dois fundos iguais, marca as palavras em negrito e
sublinhado. Veja [Palavras alteradas em um diff](interface-tour.md#palavras-alteradas-em-um-diff).

As cores são `"#rrggbb"`, `"ansi:N"` (a entrada N da paleta do terminal) ou
`"default"` (a cor do próprio terminal). Os grupos e as chaves são os de
`gitpoulpi theme export <slug>`. Nada é fatal: uma chave desconhecida ou uma cor
inválida é ignorada, uma `base` desconhecida volta para `dark`, e
`gitpoulpi theme list` reporta cada problema em `warnings` (o aplicativo os mostra
uma vez na linha de status).

Os arquivos são relidos quando a lista suspensa Tema abre e quando as Configurações fecham.
Uma janela em execução também percebe quando o `config.toml` (um
`gitpoulpi config set ui.theme ...` feito em um shell) e o arquivo do tema ativo são
reescritos, e aplica as novas cores em cerca de um segundo, sem reiniciar.

### `ui.border_focused`

A moldura do painel que tem o foco do teclado: o Log, a barra lateral Refs, os painéis
Staged e Unstaged, os detalhes do commit, os painéis de histórico, revisão e
reescrita, e as molduras do diff, dos arquivos e do editor. Ela pinta só bordas; texto,
rótulos e ícones (o rótulo "commit :", o ícone Início, os botões das abas) continuam
usando `ui.accent`. Isso permite que um tema emoldure o painel ativo com uma cor que
seria ilegível como texto, por exemplo um amarelo vivo em um tema claro. Quando um tema
não a define, ela segue `ui.accent` (também em um arquivo do usuário: um arquivo que muda
só o `accent` leva esse destaque às molduras com foco), então todo tema embutido continua
com a aparência de antes. Diálogos e popups mantêm uma moldura com a cor de destaque.

```toml
[ui]
accent = "#1f5fbf"
border_focused = "#ffd700"
```

## Detecção do sistema

Com `ui.theme = "system"`, o GitPoulpi pergunta ao terminal a cor do seu fundo **uma
vez, na inicialização** (a consulta OSC 11, respondida por kitty, WezTerm, Alacritty,
foot, iTerm2, Windows Terminal, tmux e a maioria dos outros) e escolhe Light para um
fundo claro, Dark caso contrário. A espera é de no máximo cerca de 100 ms, e um
terminal que não conhece a consulta não custa nada: a resposta é sempre lida antes de
a interface iniciar, então nada é impresso e nenhuma tecla é engolida. Sem resposta
(e sempre no Windows, que não envia a consulta) ele lê `COLORFGBG`, e depois recorre
a Dark. Um terminal cujo fundo muda enquanto o GitPoulpi roda não é acompanhado;
reinicie-o, ou escolha Light ou Dark explicitamente.

## Profundidade de cor

Os temas são truecolor (`#rrggbb`). `ui.color_depth` (Configurações > Aparência >
Profundidade de cor, aplica-se na hora) decide como isso chega ao terminal:

| Valor | Efeito |
|---|---|
| `auto` (padrão) | Truecolor, exceto onde se sabe que não funciona: o Terminal da Apple (`TERM_PROGRAM=Apple_Terminal`) usa 256 cores, o console do Linux (`TERM=linux`) 16, e um terminal `*-256color` que não define `COLORTERM` (e não é WezTerm, kitty nem Windows Terminal) 256. `COLORTERM=truecolor` sempre significa truecolor. |
| `truecolor` | Sempre cor de 24 bits. |
| `256` | Toda cor RGB vira a entrada xterm-256 mais próxima. |
| `16` | A mais próxima das 16 cores ANSI; a paleta do próprio terminal define o tom exato. |

Use `256` ou `16` se um tema aparecer com cores erradas por ssh ou em um multiplexador
antigo. A predefinição Terminal não precisa de nada disso. Truecolor não custa nada;
a passagem de mapeamento percorre a tela uma vez por quadro, somente quando ativa.

O tema é por instalação: várias instalações do GitPoulpi que compartilham uma configuração mantêm cada uma o seu, então escolher um em uma instalação nunca recolore as outras.

## Pré-visualização ao vivo

Em Configurações > Aparência, a lista suspensa Tema lista **Sistema (segue o terminal)**, as
predefinições e depois os seus próprios arquivos de tema. Mover o destaque aplica esse
tema na hora, então a tela de Configurações inteira é repintada sob você; **Enter** o
mantém (ele é salvo), **Esc** ou um clique fora restaura o anterior e não salva nada.
A entrada da paleta **App: tema...** (`Ctrl+P`) abre as Configurações nessa linha.

### Mudar o tema pela paleta

**App: mudar tema...** (`Ctrl+P`, ou `Ctrl+N t`; a tecla pode ser reatribuída como
`nav_prefix.change_theme`) abre uma lista dos mesmos temas sem passar pelas
Configurações: **Sistema (segue o terminal)**, as predefinições e depois os seus
próprios arquivos de tema, cada um com quatro pequenos chips de cor. O tema atual é
marcado. Digite para restringir a lista; mover o destaque pré-visualiza o tema na
hora, **Enter** o aplica e grava `ui.theme` no `config.toml`, **Esc** (ou um clique
fora da lista) devolve o tema anterior e não salva nada.

Sob o campo de filtro, um pré-filtro segmentado restringe a lista: **Todos**,
**Escuros**, **Claros** e **Coloridos**, mais **Personalizados** quando você tem arquivos de tema
próprios (clique em um segmento, `Tab` / `Shift+Tab`, ou `Alt+1`..`Alt+5`). Escuros e Claros vêm do brilho do `ui.bg` do tema; **Coloridos**
mantém os temas cujo fundo é claramente tingido (azul, violeta, amarelo, verde...) e
deixa de fora os cinzas brancos, quase brancos, pretos e quase pretos, então Solarized
Dark, Dracula ou Nord são coloridos enquanto GitHub Dark e One Light não são. Um tema
colorido também aparece em Escuros ou Claros. **Sistema** e os temas transparentes
(sem fundo próprio) aparecem somente em Todos. **Personalizados** mantém só os arquivos de tema da sua pasta `themes/` (um tema personalizado
continua sendo Escuro, Claro ou Colorido para esses botões); o botão não é desenhado e
não é alcançável pelas teclas enquanto essa pasta não tem nenhum tema. O pré-filtro se combina
com o que você digita. Os seletores de tema de cores do grafo e de idioma pré-visualizam do mesmo
jeito: as faixas do Log e a interface inteira mudam conforme o destaque se move.

![A lista Mudar tema aberta pela paleta, com o pré-filtro Coloridos selecionado](../images/theme-picker.png)

![A lista suspensa Tema aberta com Claro destacado: a tela de Configurações já está pintada em Claro](../images/settings-appearance-theme.png)

Sob a lista suspensa, uma faixa de chips de cor mostra o tema em vigor (fundo, texto,
atenuado, destaque, seleção, perigo, diff adicionado, diff removido, depois as seis
primeiras cores de faixa), e acompanha a pré-visualização. A lista suspensa **Tema de cores do
grafo** em **Aparência** mostra algumas cores de faixa ao lado de cada paleta, e as
escolhas se aplicam quando as Configurações fecham.

Um tema com fundo próprio (Light e as outras predefinições pintadas) preenche a janela
inteira, inclusive as Configurações e as visualizações de página inteira, e dá ao texto
que não tem cor própria a cor de texto do tema; Dark e Terminal deixam as cores do seu
terminal em paz.

## Pela linha de comando

| Comando | Saída |
|---|---|
| `gitpoulpi theme list` | JSON `{themes: [{slug, label, source, light, lane_palette}], warnings}`; `source` é `preset` ou `user`. |
| `gitpoulpi theme export <slug>` | O arquivo de tema completo como TOML na saída padrão (JSON: `{slug, toml}`); ele é lido de volta como o mesmo tema. |
| `gitpoulpi config set ui.theme <slug>` | Validado contra as predefinições e a sua pasta `themes/`; um nome desconhecido é um erro que lista os válidos. `system` e `default` são aceitos. |
| `gitpoulpi config set ui.graph_color_theme <name>` | `theme` (seguir o tema) ou o nome de qualquer paleta de faixas; um nome desconhecido lista os válidos. |

O servidor MCP oferece `theme_list` e `theme_export` como ferramentas somente leitura.
