[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · **Français** · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP et agents

Un agent IA (Claude Code, Cursor, Codex, ...) peut piloter GitPoulpi via son
**serveur MCP** ou sa **CLI**. Les deux passent par le même démon d'arrière-plan que
la fenêtre, si bien que ce que fait l'agent apparaît en direct dans votre GitPoulpi
ouvert, et les deux exécutent exactement le même code pour chaque commande.

Cette page concerne les agents *externes*. Les boutons IA propres à GitPoulpi
(messages de commit, explications, le compositeur de commits) se configurent
séparément : voir [Assistance IA](ai.md).

## Installation rapide

1. Mettez `gitpoulpi` et `gitpoulpi-mcp` dans votre `PATH` ([Installation](installation.md)).
2. Enregistrez le serveur auprès de votre agent. Il parle en stdio : rien à
   démarrer, aucun port.
3. Choisissez éventuellement ce que l'agent peut faire dans la table `[mcp]` de
   `config.toml` (ci-dessous). Les valeurs par défaut lui permettent de lire le dépôt
   (les modifications sont refusées tant que vous n'avez pas mis
   `allow_mutations = true`), de vous montrer des choses seulement après votre
   confirmation, et de ne jamais préremplir de boîtes de dialogue.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` l'écrit plutôt dans le `.mcp.json` du dépôt, si bien que tous ceux
qui ouvrent le projet l'obtiennent. Le même fichier à la main, avec les extras
facultatifs :

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` remplace `[mcp] ui_control` pour cet agent seulement.
- `GITPOULPI_CONFIG_DIR` donne à l'agent son propre `config.toml` (ses propres
  limites `[mcp]`, sans accès à vos paramètres). Omettez-le pour partager les vôtres.
- `gitpoulpi --repo <path> mcp serve --stdio` fixe le serveur sur un seul dépôt ;
  sinon un outil agit sur le dépôt qui entoure le dossier de travail du serveur, ou
  sur l'argument `repo` de l'appel.

**Autres clients** : tout client MCP qui lance un serveur stdio.

| Client | Où | Quoi |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | le fichier JSON MCP du client | le bloc `mcpServers` ci-dessus (VS Code nomme la clé `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Tout autre | sa documentation | la commande `gitpoulpi-mcp` (binaire autonome, même serveur, aucun argument nécessaire) |

**Via HTTP** (clients qui se connectent à une URL) :

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

JSON sans état, mêmes outils, mêmes contrôles `[mcp]`. Chaque requête exige le
jeton bearer (`[mcp] http_auth`, activé par défaut) : affichez-le avec `gitpoulpi mcp token
--show` (ou `--copy`, `--rotate` ; le jeton vit dans un fichier réservé à l'utilisateur dans le
dossier de données privé) et donnez-le au client sous forme d'en-tête :

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

Un jeton absent ou erroné reçoit `401` avec `WWW-Authenticate: Bearer`. Un jeton
renouvelé s'applique aussitôt, sans redémarrer le serveur. Désactiver `http_auth` ne convient
qu'à un point d'accès loopback où vous faites confiance à tous les comptes locaux ; le serveur
avertit alors sur stderr.

### Joindre le point d'accès depuis une autre machine

La configuration la plus sûre n'a besoin d'aucun écouteur distant : gardez l'adresse loopback et
redirigez le port par SSH depuis la machine qui exécute l'agent, `ssh -L
8808:127.0.0.1:8808 host`. `--allow-remote` n'entre alors pas en jeu.

`--allow-remote` (une adresse non loopback) est refusé sauf si l'authentification est activée
avec un jeton en place **et** si le trafic est chiffré. Le refus nomme ce qui manque. Il y a deux
façons d'être chiffré :

- **TLS dans le serveur** (`[mcp] tls = "self_signed"` ou `"files"`). Le serveur
  termine lui-même le TLS : il détient la clé, mène la poignée de main, et seule une poignée de main
  terminée atteint la couche HTTP ; un client HTTP en clair échoue à la poignée de main et aucun
  écouteur en clair n'existe à côté. Le chiffrement est donc un fait que le serveur établit à chaque
  connexion, pas une supposition. Avec `self_signed`, le premier démarrage génère une paire dans le
  dossier de données privé et affiche son empreinte SHA-256 (aussi dans les Paramètres) ; un client
  fait confiance au serveur en épinglant cette empreinte ou le fichier du certificat. Avec `files`,
  réglez `tls_cert` et `tls_key` sur vos fichiers PEM et les clients vérifient le certificat par son
  autorité. Les clients s'authentifient avec le jeton bearer ; les certificats clients ne sont pas
  utilisés.
- **Un tunnel ou un proxy inverse** (`[mcp] behind_proxy = true`). Un agent de tunnel ou un
  proxy se connecte comme n'importe quel client local ou du réseau, et ses en-têtes peuvent être
  falsifiés : le serveur ne peut donc pas savoir de l'intérieur s'il y en a un. C'est pourquoi c'est
  une reconnaissance explicite (affichée avec un avertissement rouge dans les Paramètres) et non une
  détection. Lancez le tunnel sur la même machine et gardez l'adresse loopback quand vous le pouvez.
  Avec une adresse non loopback derrière un proxy, le tronçon entre le proxy et le serveur circule en
  clair sur ce réseau.

**Sans MCP**, un agent avec un shell utilise la CLI : `gitpoulpi --json <commande>`
(voir la [référence CLI](cli-reference.md)). Donnez-lui la skill toute prête
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), qui dit quand et
comment utiliser GitPoulpi au lieu de `git` brut.

## Permissions

| Interrupteur | Défaut | Effet |
|---|---|---|
| `[mcp] enabled` | `true` | `false` : `gitpoulpi mcp serve` / `gitpoulpi-mcp` refusent de démarrer (Paramètres > Agents et MCP > *Serveur MCP*). Lu au démarrage du serveur : un serveur déjà lancé garde ses réglages. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Adresse que `--http` utilise quand aucune n'est donnée (loopback seulement sauf `--allow-remote`, qui exige authentification et chiffrement, voir plus haut). Paramètres > Agents et MCP > *Port du serveur MCP* modifie son port et conserve l'hôte. |
| `[mcp] http_auth` | `true` | `--http` exige `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Paramètres > Agents et MCP > *Authentification HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` ou `files` (vos chemins PEM) : le serveur termine lui-même le TLS. Paramètres > Agents et MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Vous garantissez qu'un tunnel ou un proxy inverse chiffre le trafic, ce qui permet à `--allow-remote` de fonctionner sans TLS. |
| `[mcp] allow_mutations` | `false` | Désactivé : tout outil qui change quelque chose est refusé ; les lectures, `ui_*` et les propositions fonctionnent encore. `true` laisse l'agent commiter, pousser, reset... |
| `[mcp] mutation_allowlist` | `[]` | Non vide : seuls ces outils peuvent changer des choses, p. ex. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false` : l'agent ne peut pas déplacer votre fenêtre (`ui_state` / `ui_viewers` répondent encore). |
| `[mcp] ui_control` | `"ask"` | `ask` : une pastille dans votre fenêtre que vous confirmez ; `auto` : appliqué sauf si vous êtes occupé ; `off` : refusé. Par agent : `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Permet à l'agent d'ouvrir des boîtes préremplies que vous confirmez (Paramètres > Agents et MCP > *Laisser les agents IA proposer des modifications à confirmer*). |
| `[mcp] allow_sensitive_staging` | `false` | Seulement dans `config.toml`. Désactivé, l'outil `stage` n'offre pas `allow_sensitive`, et un stage qui ajouterait un fichier sensible (une clé privée, un `.env`, un jeton reconnu) est refusé avec `{error: "sensitive_files", files}` : l'agent doit vous demander. `true` lui laisse passer `allow_sensitive`. Voir [Stage et commit](staging-and-committing.md#fichiers-sensibles). |
| `[mcp] allow_history_purge` | `false` | Seulement dans `config.toml`. `true` expose l'outil `rewrite_purge`, qui supprime définitivement l'ancien historique d'une réécriture d'historique (et son annulation). Désactivé : l'agent ne le voit pas. Il exige quand même `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | L'interrupteur général de votre fenêtre (Paramètres > Agents et MCP > *Laisser les agents IA piloter la vue*). Désactivé : aucun agent ne peut atteindre votre fenêtre. |
| `[ui] launch` | `"off"` | Si un agent peut ouvrir une fenêtre GitPoulpi quand aucune n'est ouverte (voir [plus bas](#quand-aucune-fenêtre-nest-ouverte-ou-quelle-est-dans-une-autre-session)). |

Ce que cela signifie en pratique :

- **D'emblée l'agent est en lecture seule** : il peut lire, vous montrer des choses
  dans votre fenêtre (`ui_*`) et, si vous l'autorisez, proposer des changements que
  vous confirmez. Pour le laisser modifier le dépôt (commit, push, reset...) mettez
  `allow_mutations = true` ; pour une autorisation étroite gardez-le à `true` avec une
  `mutation_allowlist`. Chaque outil porte des annotations MCP (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), si bien qu'un client comme Claude Code vous
  demande encore avant les plus risqués.
- **Conflits** : `conflicts_check` (une prévision, avant tout merge), `conflicts_list`, `conflicts_show` et `conflicts_state` sont des
  lectures ; `conflicts_resolve` (il abandonne le côté non gardé, `--delete` supprime
  le fichier) et `conflicts_abort` (abandonne tout le merge/rebase/cherry-pick) sont
  destructifs ; `conflicts_unresolve` est une écriture simple. Un appel refusé
  (`stale`, `not_conflicted`, `pick_incomplete`...) répond `{"refused": {...}}` avec
  le code de sortie 2 au lieu d'échouer.
- **États bloqués** : `conflicts_am_continue` est une écriture, `conflicts_am_skip`,
  `conflicts_am_abort` et `index_lock_remove` sont destructifs, `index_lock_show` est
  une lecture.
- **Réécriture de l'historique** : `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (l'analyse de taille), `rewrite_scan_secrets` (des résultats avec un aperçu masqué, jamais une valeur),
  `rewrite_script_check`, `rewrite_script_test` et `rewrite_discard` sont des lectures
  (un aperçu ne change rien : il garde seulement un plan dans le démon et sur disque,
  dans le dossier git, si bien que `rewrite_apply` fonctionne aussi après un
  redémarrage). `rewrite_apply` et `rewrite_run` sont destructifs : ils déplacent de
  nombreuses refs vers des commits réécrits en une seule entrée d'annulation (`undo`
  les remet en place) et ne poussent jamais. Passez la recette dans `recipe_json` (ou
  `recipe`, un chemin) et les secrets dans `secrets_file`, jamais en clair dans
  l'appel. Les scripts shell, Python et Lua externe (`script_language`) ne sont pas
  isolés et **un agent ne peut pas les approuver** : il n'existe ni outil
  `rewrite_script_trust` ni argument `trust`, donc un tel script ne s'exécute que
  si une personne a lancé `gitpoulpi rewrite script trust FICHIER` sur ce texte
  exact (un script modifié est refusé de nouveau, `script_not_trusted` avec le
  hachage dans `refused.script`), et seulement si `rewrite.external_scripts`
  active son langage (`scripts_disabled`). Un refus répond `{"refused": {"reason", "message", "detail"}}` et un plan
  qui déplace des refs poussées répond `{"needs_confirmation": ...}` (code de sortie 3)
  tant que l'appel ne fixe pas `yes` : montrez-le d'abord à l'utilisateur.
- **Jamais accessible à un agent** : stocker un jeton (`hosting login`), créer et
  envoyer une clé SSH, `config set` (il pourrait réécrire ses propres limites), et le
  cycle de vie du démon. Un jeton n'est jamais montré à l'agent.
- **Listes à l'échelle du compte** : `hosting_radar` est une lecture (demandes en attente de votre
  relecture, demandes ouvertes, tickets assignés dans tous les dépôts d'un profil, depuis un cache de
  cinq minutes sauf avec `refresh`) ; `hosting_integrations_radar` modifie la table
  `[integrations.radar]` d'un profil, donc `allow_mutations` la contrôle (même appelé sans option, ce qui
  n'affiche que les réglages).
- **Contexte de la revue** : `ai_review_context` montre ce que reçoit un brouillon de revue de code en plus du
  diff (CLAUDE.md, AGENTS.md, un fichier supplémentaire, où travaille un agent local). Avec un argument de
  réglage, il enregistre une valeur par défaut (globale, ou propre au `repo` ou au `project`), donc
  `allow_mutations` le contrôle, comme les autres outils qui changent un réglage ; sans argument, il se contente d'afficher.
  Comme tout outil `ai_*`, il n'est listé que tant que `ai.enabled` est activé.
  `hosting_review_suggest` prend les mêmes arguments pour une seule exécution.
- **Les contrôles `[mcp]` s'appliquent au serveur MCP.** La CLI est votre propre shell
  et n'est pas contrôlée par `allow_mutations` ; seuls `ui_control` et
  `allow_ui_propose` s'appliquent aussi à `gitpoulpi ui ...`.
- Un appel refusé est une erreur d'outil ordinaire (`isError: true`, la raison dans
  `structuredContent.error`), jamais un non-événement silencieux.

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_sensitive_staging = false  # true: the agent may stage files the staging guard flags
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## Les outils

Un outil par commande CLI, nommé d'après son chemin avec `_` (`branch create` devient
`branch_create`). Chacun prend les arguments de la commande plus `repo` (le dépôt ;
par défaut celui du serveur) et `session`. Le `structuredContent` d'un appel réussi
est exactement ce qu'affiche `gitpoulpi --json <commande>`. **Kind** (la colonne
« Kind », en anglais comme dans le serveur) : *read* ne change jamais rien ; *write*
et *destructive* sont des mutations (contrôlées) ; *network* parle à un distant ou à
un hébergeur ; *view* ne déplace que votre fenêtre ; *propose* ouvre une boîte que
vous confirmez.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `ai_worktree_name` | `gitpoulpi ai worktree-name` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### Les actions de workflow de branches sont en planifier puis exécuter

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` et `flow_set_parent`
([Workflows de branches](branching-workflows.md#actions)) ne s'exécutent jamais sur un
simple appel. `dry_run: true` renvoie le plan (chaque étape et commande git, les avertissements,
ce que l'annulation ne peut pas reprendre, les blocages) comme un résultat normal ; `yes: true` l'exécute ;
sans l'un ni l'autre, l'appel est refusé et porte le plan. Les `[hooks]` du fichier de workflow sont des
commandes shell tirées d'un fichier que l'équipe partage : elles ne s'exécutent donc qu'avec un
`run_hooks: true` explicite : le plan les liste (`hooks`, `hooks_run`) pour que vous puissiez
montrer d'abord à l'utilisateur les commandes exactes. Un merge qui s'arrête sur des conflits
répond `status: "conflicted"` (un résultat d'erreur, code de sortie 2) ; une fois les fichiers résolus par les
outils de conflit, appelez le même outil avec `continue_: true`
ou `abort: true`. Ouvrir une demande de fusion (`finish` en mode `merge-request`)
n'est pas annulable et le plan le dit.

### Ce qu'un agent fait avec `git` directement est aussi dans l'historique d'annulation

Un agent qui crée des branches et des worktrees, fait des commits, des resets ou supprime des stashes
avec son propre `git` (sans passer par ces outils) reste visible : le démon
compare les refs du dépôt à ce qu'il a vu en dernier et enregistre chaque changement
comme une entrée de `undo --list` avec `"source": "outside"`, la raison notée par git
(`outside.message`) et l'auteur que git a enregistré. Les outils `undo`, `redo` et
`undo_restore` prennent ces entrées comme les autres : un agent (ou l'utilisateur,
avec `Ctrl+Z`) peut ramener une branche qui a été supprimée. Les modifications de fichiers
ne sont jamais capturées. Voir [Annuler et rétablir](undo-redo.md#changements-faits-en-dehors-de-gitpoulpi).

## Montrer des choses à l'utilisateur

Un agent peut désigner quelque chose au lieu de le décrire : `ui_show_commit`,
`ui_show_diff` (un fichier à une ligne, en ligne ou côte à côte), `ui_show_file`
(contenu, blame ou historique à une ligne), `ui_show_ref`, `ui_show_pr`,
`ui_show_issue`, `ui_show_conflict` (l'éditeur de merge sur un fichier en conflit, à
un conflit), `ui_show_wip` (`composer` place le curseur dans le résumé du commit, sans
rien taper), `ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (la
requête tapée, jamais exécutée), `ui_show_keys`, `ui_show_settings` (une page et un
paramètre, jamais modifié), `ui_highlight` (teinter des lignes du diff ou du fichier
ouvert), `ui_notify` (une notification) et `ui_back` (annuler le dernier changement de
vue de l'agent). `ui_state` lit ce que montre une fenêtre (`text: true` ajoute le texte
visible) ; `ui_viewers` liste les fenêtres.

Ces outils **ne changent jamais le dépôt** et ne peuvent ni taper, ni cliquer, ni
modifier un paramètre. Les révisions et les chemins sont vérifiés avant que quoi que
ce soit atteigne la fenêtre.

**Ce que voit et décide l'utilisateur** :

- `ask` (par défaut), ou chaque fois que vous tapez, qu'une boîte de dialogue ou les
  Paramètres sont ouverts, que vous glissez, ou que vous avez appuyé sur une touche dans
  les 1,5 dernières secondes : l'agent reçoit `deferred` aussitôt et votre ligne d'état
  montre une pastille *« claude-code veut afficher ... »*. `Ctrl+N Enter` ou un clic
  l'affiche ; `Esc` ou `Ctrl+N Backspace` l'ignore. Elle expire après 60 s.
- Appliqué : la vue bouge, une notification de 4 s dit qui l'a déplacée, et l'onglet
  reçoit un `✦` violet jusqu'à votre prochaine touche. `Ctrl+N Backspace` (palette
  `Agent : Retour`) revient en arrière ; la fenêtre se souvient de dix vues.
- Rien n'est jamais dévoilé ni extrait : un commit que votre filtre solo masque répond
  `partial` (`hidden_by_filter`) ; une liste de PR ou de tickets qui n'est pas liée à un
  fournisseur répond `hosting_not_linked`.

**Réponses** : `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, avec le `state` de la fenêtre quand c'est
appliqué. Codes de sortie de la CLI : 0 applied/partial/proposed, 3 deferred, 1 sinon.
Les appels sont limités en débit (rafale de 5, puis un par seconde).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Proposer un changement (`ui_propose_*`)

Avec `allow_ui_propose = true`, un agent peut ouvrir l'une de vos boîtes de dialogue
**préremplie**, marquée *✦ Proposé par <agent>*. Rien ne se passe tant que vous
n'appuyez pas sur la confirmation propre à cette boîte ; vous pouvez d'abord modifier
ou annuler.

| Outil | Ouvre | Vous confirmez avec |
|---|---|---|
| `ui_propose_reword` (rev, message) | la fenêtre Reword | *Mettre à jour le message* (un commit poussé demande encore à propos du force push) |
| `ui_propose_commit_message` (summary, body) | le formulaire de commit, rempli ; ne commite jamais | le bouton de commit |
| `ui_propose_branch` (name, start point) | la zone en ligne Nouvelle branche | `Enter` |
| `ui_propose_tag` (name, target, message) | la zone en ligne Nouveau tag (annoté quand il y a un message) | `Enter` |
| `ui_propose_rewrite` (recipe file) | l'atelier de réécriture d'historique avec les règles de la recette ; rien n'est prévisualisé ni appliqué | votre propre *Aperçu* puis *Appliquer* (`Ctrl+S`) ; `Esc` rejette. Une recette avec un script est refusée, les valeurs secrètes sont retirées |
| `ui_propose_split` (rev, plan file) | l'éditeur de scission sur le commit, rempli d'après le plan | *Scinder* (`Ctrl+S`) ; `Esc` rejette |
| `ui_propose_resolution` (path, pick ou content file) | l'éditeur de merge sur un fichier en conflit, avec les choix (ou tout le texte) déjà dans le Résultat | *Enregistrer et marquer résolu* (`Ctrl+S`) ; le fermer rejette |

Une proposition à la fois ; une proposition de message de commit est refusée tant que
vous avez votre propre brouillon dans le compositeur, et une proposition de résolution
tant que l'éditeur de merge contient du travail non enregistré de votre part ou quand
les étapes du fichier ont changé depuis que l'agent les a lues. Une résolution que vous
modifiez avant de confirmer est signalée comme `accepted_edited`. La réponse porte un id
de `proposal` ; ce que vous avez décidé figure dans les `proposals` de `ui_state`
(`offered`, `open`, `accepted`, `accepted_edited`, `rejected`, `expired`, `failed`).

### Quand aucune fenêtre n'est ouverte, ou qu'elle est dans une autre session

- Une fenêtre appartient à la session où elle a été démarrée. Si la session de l'agent
  n'en a aucune, `ui show` cherche **une** fenêtre dans une autre session qui a le
  dépôt ouvert et l'utilise (`routed_session` dans la réponse). Si plusieurs
  correspondent : `no_viewer` les nommant ; passez `--session`.
- **Ouvrir une fenêtre pour l'agent** est désactivé par défaut. Réglez `[ui] launch`
  (Paramètres > Agents et MCP > *Ouvrir une fenêtre pour les agents*) sur un préréglage de
  terminal ou de multiplexeur ou sur une commande `custom` ; alors un appel avec
  `launch: true` (CLI `gitpoulpi ui --launch show ...`) en ouvre une et attend jusqu'à
  10 s. Avec `ui_control = "auto"` elle s'ouvre aussi sans l'option ; avec `ask` l'agent
  reçoit `no_viewer` et devrait vous le dire.

| Préréglage | Commande |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (votre terminal par défaut) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nouvelle fenêtre), `zellij` (nouveau volet), `cmux` (nouvel espace de travail) | dans un multiplexeur en cours d'exécution, dans le dossier du dépôt |
| `custom` | `[ui] launch_command`, p. ex. `my-term --cd {repo} {gitpoulpi} --session {session}` ; découpé en arguments, sans shell |

Si un préréglage ne fonctionne pas avec votre version du programme, utilisez `custom`.

## Erreurs

- Un appel mal formé (outil ou argument inconnu, argument obligatoire manquant) est une
  erreur JSON-RPC `-32602 invalid_params`.
- Un refus, ou une erreur git ou du démon (mauvaise révision, conflit, ...), est une
  erreur d'outil : `isError: true` avec le message dans `content` et
  `structuredContent.error`.
