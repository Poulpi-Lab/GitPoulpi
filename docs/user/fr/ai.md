[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · **Français** · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# Assistance IA

GitPoulpi n'a jamais besoin de compte, et **l'IA est désactivée par défaut**.
Quand l'IA est activée, elle peut écrire des messages de commit et de stash,
expliquer des modifications, rédiger des descriptions de pull request, suggérer des
résolutions de conflits de merge et répartir vos modifications locales en plusieurs
commits. Le modèle est le vôtre : une clé d'API que vous possédez, un serveur
local, ou une CLI d'agent déjà installée (comme `claude -p`).

Chaque contrôle IA affiche le `✦` violet. Tant que l'IA est désactivée :

- Rien n'est envoyé nulle part.
- Aucun contrôle IA n'est affiché : pas de bouton `✦`, pas de commande de palette
  `IA :`, pas de ligne de menu IA et pas d'indication de touche IA. Les touches IA
  ne font rien.
- La page IA du dépôt affiche seulement « L'IA est désactivée (Paramètres > IA) ».
- Le serveur MCP ne liste pas les outils `ai_*`. Les commandes CLI `gitpoulpi ai
  ...` refusent avec le même message.

**Activer l'IA** prend effet aussitôt. Vous n'avez pas besoin de redémarrer.

Il y a deux façons d'utiliser l'IA avec GitPoulpi :

| | IA intégrée (cette page) | Un agent externe |
|---|---|---|
| Quoi | GitPoulpi demande à un modèle un message de commit, une explication, ... | Claude Code, Cursor, Codex... pilotent GitPoulpi comme un outil |
| Se règle dans | Paramètres > IA (`[ai]`) | la configuration MCP de l'agent, limites dans `[mcp]` |
| Guide | ci-dessous | [MCP et agents](mcp-and-agents.md) |

## Configurer l'IA

1. `Ctrl+,` > **IA** > cochez **Activer l'IA**.
2. Les profils sont la liste de l'onglet **Profils (connexions)**, à gauche de la page
   (empilée au-dessus dans un terminal étroit) : choisissez-en un pour le modifier à droite (`Enter` passe à ses
   lignes ; la commande de palette **IA : Modifier le profil...** saute à la liste).
   **+ Ajouter** en crée un autre, **Dupliquer** copie le profil sélectionné (sans sa
   clé) et **Supprimer** demande d'abord confirmation. Choisissez le **Fournisseur** et
   remplissez ses champs :

   | Fournisseur | Champs | Valeurs par défaut |
   |---|---|---|
   | **Agent local** (aucune clé, aucun compte propre) | **Type d'agent** (quel agent c'est), **Commande** (le programme exact : dans votre `PATH`, un alias ou un script wrapper, ou un chemin complet), **Arguments** qui le rendent non interactif, et **Variables d'environnement** facultatives. Le prompt va sur son stdin, la réponse est lue sur stdout. | Claude Code : `claude`, `-p --model haiku` |
   | **Anthropic** | **Clé d'API**, **Modèle** et **URL de base** facultatifs | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **Clé d'API**, Modèle, URL de base | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **Clé d'API**, Modèle, URL de base | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **Compatible OpenAI** (Ollama, LM Studio, vLLM, une passerelle d'entreprise) | **URL de base** (`.../v1`), **Modèle**, clé facultative | `http://localhost:11434/v1` (Ollama ; LM Studio est `http://localhost:1234/v1`), `llama3` |

   **Préréglages** (vous pouvez toujours saisir une autre valeur) :

   ![La liste Type d'agent d'un profil d'agent local, listant Claude Code, OpenAI Codex CLI, Gemini CLI, opencode et les autres CLI d'agents](../images/settings-ai-presets.png)

   - **Agent local > Type d'agent** est une liste des CLI d'agents courantes, les
     plus utilisées d'abord : Claude Code, OpenAI Codex CLI, Gemini CLI, opencode,
     Cursor Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose, Crush, Amp, ou
     *Autre (aucune suggestion)*. Le type est **distinct de la commande** : il ne
     fait que choisir les suggestions que vous recevez. En choisir un préremplit
     **Commande** (tant que c'est encore la valeur par défaut du type précédent) et
     **Arguments** avec l'invocation sans interface (`claude -p`, `codex exec -`,
     `gemini -p ...`, ...). **Commande** est du texte libre : écrivez
     `/opt/bin/work-claude`, un script wrapper ou la cible d'un alias et le type,
     avec ses options, sa liste de modèles et ses variables, reste. Quand **Type
     d'agent** est vide, GitPoulpi le déduit de la commande.
   - **Options des arguments** ajoute ou retire des options adaptées à l'usage
     ponctuel de GitPoulpi, chacune avec une raison d'une ligne : pas d'outils,
     pas de session enregistrée, sortie en texte brut, effort de raisonnement
     faible, mode lecture seule, pas de demandes de permission, pas de
     vérification de mise à jour ni d'analytique. La liste diffère selon l'agent
     parce que les agents diffèrent (seuls certains peuvent désactiver les outils
     en ligne de commande ; pour opencode c'est une variable d'environnement).
     **Modèle** liste les modèles de cet agent et écrit la bonne option (`--model
     haiku` pour Claude Code, `-m gpt-6-luna` pour Codex, `--model
     provider/model` pour opencode) ; *(défaut - aucune option)* la retire. Amp n'a
     pas d'option de modèle. `aider` lit son prompt depuis `/dev/stdin`, il ne
     fonctionne donc que sous Linux et macOS.
   - Les **Variables d'environnement** ne sont définies que pour le processus de cet
     agent, jamais pour GitPoulpi ni pour d'autres profils. **Ajouter une variable
     d'environnement** propose les variables documentées qui comptent pour cet
     agent, chacune avec ce qu'elle fait (désactiver la télémétrie et les
     vérifications de mise à jour, niveau d'effort, dossier de configuration, clé
     d'API, URL de base ou hôte local, ...) ; vous pouvez aussi saisir `NAME` ou
     `NAME=value`. Chaque variable a alors sa propre ligne **Env NAME** : sa valeur
     sous forme de bouton (appuyez dessus pour en saisir une nouvelle ; elle s'enregistre
     quand vous quittez le champ) et un bouton **[x]** qui supprime la variable. Une valeur peut être `${env:NAME}` pour
     emprunter l'une des variables propres de GitPoulpi (pratique pour les clés
     d'API, afin qu'elles ne soient pas écrites dans `config.toml`). Les valeurs qui
     ressemblent à des secrets (noms finissant par `_KEY`, `_TOKEN`, `_SECRET`, ... ou
     préfixes de clés bien connus) s'affichent en puces et se saisissent de zéro.
     Les valeurs ne sont jamais journalisées.
   - **Fournisseurs HTTP > Préréglage du fournisseur** remplit l'URL de base et un
     modèle pour OpenAI, Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek,
     Mistral, Groq, Together, Fireworks, Cerebras, Perplexity, Moonshot, Qwen
     (DashScope), Zhipu (GLM) et les serveurs locaux Ollama, LM Studio, llama.cpp,
     vLLM, Jan et LocalAI. **Modèle** est une liste des modèles de ce fournisseur,
     texte libre permis.

3. Appuyez sur **Tester la connexion** : *Dernier test* affiche `connecté : le modèle a répondu`
   ou `failed: <raison>`.
4. Avec plusieurs profils, choisissez le **Profil par défaut** en bas ; chaque
   action peut quand même nommer le sien.

![Le bloc Profils de la page IA pour un agent local : Type d'agent (Claude Code), Commande, Arguments, Modèle, deux lignes Env, Ajouter et Retirer une variable d'environnement, Tester la connexion et Profil par défaut](../images/settings-ai-agent.png)

Les **clés d'API** se saisissent avec `Enter` sur la ligne **Clé d'API** et sont
stockées dans l'assistant d'identifiants de votre système (Git Credential Manager,
libsecret / `pass`, Keychain) sous `ai:<id du profil>`, jamais dans `config.toml`.
Sans assistant, GitPoulpi le dit au lieu de faire semblant de la garder. *Paramètres >
SSH et identifiants > Oublier tous les noms d'utilisateur et mots de passe* les supprime.

La même chose dans `config.toml` (tout le reste est dans la [référence de
configuration](config-reference.md#ai-ia-intégrée-désactivée-par-défaut)) :

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # free: an alias, a wrapper script, an absolute path
harness = "claude"        # which agent it is; empty = inferred from `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # this agent's process only; ${env:NAME} borrows ours
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Mode d'exécution : headless ou terminal virtuel

Un profil d'agent local lance son agent de l'une de deux façons, au choix dans
**Mode d'exécution** (affiché pour Claude Code, OpenAI Codex CLI et Gemini CLI) :

- **Headless (non interactif)**, par défaut : le mode à requête unique de
  l'agent (`claude -p`), prompt sur stdin, réponse sur stdout.
- **Interactif (terminal virtuel)** : GitPoulpi démarre l'écran propre de
  l'agent dans un terminal caché, y tape la demande comme le ferait une
  personne, attend la réponse et ferme l'agent. Certains agents facturent ou
  limitent leur mode headless autrement que leur mode interactif ; ce qui
  s'applique à vous, et ce que disent les conditions du fournisseur pour l'un
  ou l'autre, dépend du fournisseur : renseignez-vous auprès de lui.

Chaque mode garde ses propres **Arguments**, **Options des arguments** et
**Modèle** : changer de mode ne fait rien perdre, et le mode interactif ne
propose que les options utiles sans l'option headless. Deux lignes de plus
valent pour les deux modes :

- **Dossier de travail** : *Dossier dédié* est un dossier vide que GitPoulpi
  possède, donc l'agent ne voit aucun projet et vous demande de lui faire
  confiance une seule fois. *Dépôt courant* donne à l'agent le contexte propre
  du projet (son `CLAUDE.md`, par exemple), et il demande la confiance une fois
  par dépôt. *Par défaut* est le dossier dédié en mode interactif et, en mode
  headless, là où GitPoulpi s'exécute, comme avant.
- **Délai (secondes)** : le temps dont l'agent dispose avant que GitPoulpi
  abandonne et l'arrête. Vide : 120 secondes en headless, 180 en interactif.

**Les questions de premier lancement sont à vous.** Un agent qui ne s'est
jamais exécuté dans un dossier pose d'abord une question (faire confiance à ce
dossier, se connecter, choisir un thème). GitPoulpi ne répond jamais à votre
place : la requête s'arrête avec un message, et **Ouvrir l'agent...** (sur le
profil, ou *IA : Ouvrir l'agent* dans la palette de commandes) ouvre l'agent
une fois, dans un terminal visible, dans le dossier de travail qu'il utilisera.
Répondez à la question, fermez l'agent, puis réessayez ; la réponse est
mémorisée par l'agent.

La réponse est lue dans la transcription de session de l'agent pour Claude Code
(son texte exact, supprimée ensuite), et sur l'écran entre deux lignes
repères que la demande réclame, pour Codex et Gemini. Au plus deux exécutions
en terminal virtuel ont lieu en même temps, et appuyer sur Échap pendant une
requête arrête l'agent.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # the headless set, kept
run_mode = "terminal"               # headless (default) | terminal
terminal_args = ["--effort", "low"] # the interactive set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 or absent: 120 headless, 180 terminal
```

## Instructions

Du texte que vous ajoutez à ce qui est dit au modèle. Le prompt intégré de chaque
action (qui fixe le format de la réponse) vient toujours en premier et ne peut
pas être modifié ; ensuite quatre niveaux, du plus général au plus précis, chacun
seulement s'il n'est pas vide :

1. **Instructions globales** (chaque action). Les pastilles *Insertion rapide* de
   l'éditeur insèrent des phrases courantes : *Respond in `<language>`*, *Réponses concises*, *Mode impératif*,
   *Ton technique*, *Ton décontracté*.
2. **Instructions de l'espace de travail** (chaque action, chaque dépôt de
   l'[espace de travail](workspaces.md#instructions-ia-dun-espace-de-travail)) :
   pour toutes les actions, et facultativement par action. Modifiées dans le
   formulaire Modifier de l'espace de travail.
3. **Instructions du dépôt** (chaque action, ce dépôt seulement), avec *Combiner
   avec le global* : **Ajouter au global** (après elles) ou **Remplacer le global**
   (à leur place).
4. **Instructions de l'action** (*Génération du message de commit*, *Expliquer
   les modifications*, *Génération du message de stash*, *Pull Request*, *Compositeur
   de commit*, *Résolution de conflits*, *Revue de code*, *Scinder un commit*, *Script de réécriture d'historique*), chaque bloc avec ses propres pastilles dans son éditeur (par
   ex. *Format Conventional Commits*, *Limite de 50 caractères*) et sa propre liste
   **Profil** (`Par défaut (<nom>)` suit le profil par défaut).

`Enter` sur une ligne *Instructions perso.* ouvre un éditeur multiligne (`Ctrl+S`
enregistre, `Esc` annule). Au-dessus du texte, une ligne *Insertion rapide*
contient les pastilles de cette section : cliquez sur l'une d'elles pour insérer sa
phrase au curseur. Au clavier, `Tab` passe à la ligne, `Left`/`Right` choisissent une
pastille, `Enter` ou `Space` l'insère et revient au texte (`Tab` ou `Esc` reviennent
sans rien insérer). Une pastille dont la phrase figure déjà dans les instructions est
estompée, et l'insérer de nouveau ne fait rien. Le texte est un véritable éditeur :
un clic place le curseur, un glissé ou `Shift`+flèches sélectionnent, `Ctrl+A`
sélectionne tout, `Ctrl+C` / `Ctrl+X` copient et coupent la sélection, `Ctrl`+flèches
sautent de mot en mot, `Home` / `End`, `PageUp` / `PageDown` et la molette
parcourent un long texte. La page elle-même n'affiche qu'un aperçu de chaque
instruction.

![La page IA : instructions globales, puis les blocs Génération du message de commit et Expliquer les modifications avec une liste de profils et des instructions perso.](../images/settings-ai.png)

**Par dépôt** : *Paramètres > Paramètres du dépôt > IA* (quand un dépôt est ouvert)
contient les instructions du dépôt et, par action, un profil et des instructions
qui remplacent les globaux. Un champ vide utilise le paramètre global (affiché grisé
dedans) ; **Revenir au global** retire une surcharge. Enregistré dans le
`.gitpoulpi/repo-state.toml` du dépôt, et respecté aussi par la CLI.

![La page IA du dépôt : Instructions du dépôt avec la liste Combiner avec le global, puis un profil et des instructions pour chaque action](../images/settings-repo-ai.png)

### Comment les niveaux se combinent

L'ordre d'envoi est : prompt intégré, **global**, **espace de travail**, **dépôt**,
puis les instructions propres à l'action. Le mode d'un niveau décide de ce qui
arrive à ceux du dessus :

- **Ajouter** (par défaut) : le texte du niveau est ajouté après les plus généraux.
- **Remplacer** : le texte du niveau est utilisé *à la place de* tous les niveaux
  plus généraux (le global, et l'espace de travail quand le dépôt remplace). Un
  niveau sans texte ne remplace jamais rien.

Les instructions propres à l'action suivent la même idée : les globales pour cette
action, puis celles de l'espace de travail pour cette action (ajoutées après elles,
ou à leur place), et la surcharge propre d'un dépôt pour l'action l'emporte sur les
deux. L'espace de travail utilisé est l'espace de travail actif de la fenêtre quand
il contient le dépôt, sinon le premier espace de travail qui le contient ; en ligne
de commande, c'est celui de `GITPOULPI_WORKSPACE`, sinon le premier.

Pour voir exactement ce qui est envoyé, niveau par niveau, lancez
`gitpoulpi ai prompt --action <action> --repo .` (`base`, `global`, `workspace`,
`repository`, `task`, et le `system_prompt` final). Aucun modèle n'est appelé et
cela marche avec l'IA désactivée. Définissez les instructions d'un espace de travail
depuis un script avec `gitpoulpi workspace set-ai-instructions <id> [--action <a>]
[--mode append|replace] --text ...` (ou `--clear`).

### Variables de prompt

Chaque texte d'instructions (global, espace de travail, dépôt, par action) peut contenir des variables
`${name}`. Elles sont remplacées par leur valeur juste avant l'envoi du prompt (le
texte enregistré garde le `${...}`). Un nom qui n'est pas dans le tableau est envoyé
tel quel et l'éditeur vous en avertit ; écrivez `$${` pour un `${` littéral.

| Variable | Se développe en | Exemple |
|---|---|---|
| `${uilang}` | La langue de l'interface, par son nom anglais | `French` |
| `${repo}` | Le nom du dossier du dépôt | `gitpoulpi` |
| `${branch}` | La branche courante | `main` |
| `${ref}` / `${commit}` | Sha court du commit dont parle l'action (*Expliquer*), sinon HEAD | `4c0443d` |
| `${commit_subject}` | Ligne de sujet de ce commit | `feat: add search` |
| `${date}` | Date locale, ISO | `2026-10-03` |
| `${time}` | Heure locale, `HH:MM` | `14:05` |
| `${datetime}` | Date et heure locales | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | Le distant par défaut (`origin`, sinon le premier) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (un autre hébergeur) ou `none` | `github` |
| `${files}` | Nombre de fichiers du diff de l'action | `3` |
| `${action}` | L'action en cours d'exécution | `commit_message` |

Dans l'éditeur d'instructions : tapez `${` et une fenêtre liste les variables
correspondantes (`Up`/`Down` choisissent, `Enter` ou `Tab` insère, `Esc` la ferme) ;
ou utilisez la liste **Variables** en bas à gauche (`Ctrl+T`, ou *IA : Insérer une
variable de prompt* dans la palette de commandes), qui insère au niveau du curseur.
Chaque entrée a une description, affichée dans la liste et sur la ligne au-dessus des
boutons pendant que vous la survolez ou la mettez en surbrillance.

## Ce qu'elle sait faire

| Action | Dans la fenêtre | CLI |
|---|---|---|
| **Message de commit** | Stagez des modifications, appuyez sur le `✦` à droite du champ Résumé, `Ctrl+G i`, ou la palette `IA : Générer le message de commit`. Le résumé et la description sont remplis pour que vous les modifiiez ; rien n'est commité. | `ai commit-message` (affiche), `commit --ai` (commite) |
| **Expliquer les modifications** | Le `✦` en haut à droite du volet des modifications locales (`Ctrl+G w` : le commit sélectionné, sinon les modifications locales), ou l'entrée de menu d'un commit *Expliquer les modifications (IA)*. Une fenêtre : les flèches défilent, `c` copie, `Esc` ferme. | `ai explain [rev] [--wip]` |
| **Message de stash** | `Ctrl+G m` ouvre la fenêtre du stash ; son bouton IA (`Ctrl+G` dans la fenêtre) interroge le modèle. La palette `IA : Générer le message de stash` fait les deux. | `stash push --ai` |
| **Titre et description de pull request** | *Générer titre et description* dans le formulaire Créer une pull request (les commits et le diff de la branche sont lus localement). | |
| **Résolution de conflits de merge** | Dans l'éditeur de conflits de merge : le bouton `✦ Suggérer` de la barre d'outils, l'entrée du menu clic droit d'un conflit *✦ Suggérer une résolution (IA)*, `Alt+I`, ou la palette `IA : Suggérer une résolution de conflit de merge`. La suggestion remplace la zone du *conflit courant* dans le Résultat en une seule modification (marquée modifiée, une étape d'annulation) ; rien n'est enregistré ni stagé tant que vous n'enregistrez pas. | `ai resolve-conflict <path> [--conflict N]` (affiche) |
| **Revue de code** | Dans la [revue d'une pull ou merge request](review.md#-brouillon-de-revue-ia) : le bouton `✦ Rédiger une revue...`, `s`, ou la palette `Revue : ✦ Rédiger une revue...` (elle ouvre les options de la revue, `Enter` lance). Le modèle lit le diff de la demande, sa description et ses commentaires existants (et, par défaut, les fichiers `CLAUDE.md` et `AGENTS.md` du projet : voir [Contexte de la revue](#contexte-de-la-revue)) et propose des commentaires ; vous acceptez, modifiez ou écartez chacun. Rien n'atteint l'hébergeur avant l'envoi de la revue. | `hosting review suggest <n>` (affiche du JSON, ne publie rien), `ai review-context` (les valeurs par défaut) |
| **Scinder un commit** | Le menu d'un commit *Scinder le commit (IA)...*, ou la palette `IA : Scinder le commit sélectionné...`. L'éditeur de scission s'ouvre avec le regroupement du modèle déjà rempli ; voir plus bas. | `ai split-plan <rev>` (affiche) |
| **Script de réécriture d'historique** | Dans l'[atelier de réécriture](history-rewrite.md#écrire-un-script-avec-lia) : le bouton `✦ Écrire avec l'IA...` de l'éditeur de script, ou la palette `Réécriture : écrire un script avec l'IA...`. Une boîte de dialogue montre exactement ce qui sera envoyé ; le script arrive dans l'éditeur, signalé, et n'est jamais exécuté ni appliqué à votre place. | `ai rewrite-script "<consigne>" [--sample N]` (affiche) |
| **Règles de réécriture d'historique** | Dans l'[atelier de réécriture](history-rewrite.md#créer-des-règles-avec-lia) : le bouton `✦ Créer des règles avec l'IA...`, `Alt+I`, ou la palette `Réécriture : créer des règles avec l'IA...`. Une boîte de dialogue multiligne montre exactement ce qui sera envoyé ; les règles et les réglages arrivent dans l'atelier et rien n'est prévisualisé ni appliqué à votre place. | `ai rewrite-rules "<description>" [--context]` (affiche) |
| **Nom de worktree** | Dans la [boîte Créer un worktree](worktrees.md#nommer-avec-lia) : le bouton `✦ Nommer avec l'IA` ou `Alt+A` ouvre une zone où vous décrivez la fonctionnalité, le bug ou la tâche ; le modèle nomme le worktree et sa branche d'après les noms de branches du dépôt, et les deux arrivent dans les champs de la boîte, à modifier. Seuls votre description et les noms des branches locales sont envoyés : pas de boîte « ce qui est envoyé ». | `ai worktree-name --describe "<tâche>"` (affiche) |
| **Compositeur de commits** | Le menu de la ligne WIP *Composer des commits (IA)...*, `Ctrl+G C`, la palette `IA : Composer des commits à partir des modifications locales...` | |

`Esc` annule une requête en cours (le processus d'un agent local est tué).

### Résolution de conflits de merge

Le modèle reçoit le chemin et le langage du fichier, les blocs base, courant (A) et
entrant (B) du conflit avec les libellés que montre l'éditeur, jusqu'à 20 lignes de
texte déjà fusionné avant et après, et vos instructions (globales, du dépôt, et
celles de *Résolution de conflits* dans Paramètres > IA). Il répond avec le texte
résolu de ce seul conflit dans un bloc de code ; une réponse avec des marqueurs de
conflit ou du bavardage au lieu de code est renvoyée une fois, puis refusée. Un
conflit de plus de 40 Ko (`ai.limits.conflict_bytes`, *Paramètres > IA > Limites*) est
retenu derrière un avertissement qui donne sa taille et la limite, avec *Envoyer quand
même* (cette requête seulement, toujours nettoyée par les contrôles de confidentialité),
*Ouvrir le paramètre* et *Annuler* ; la limite est une protection propre à GitPoulpi :
augmentez-la quand votre modèle a la place. Les contrôles sont masqués quand
l'IA est désactivée, et une suggestion n'enregistre, ne stage ni ne résout jamais
rien : relisez le Résultat, modifiez-le et enregistrez comme d'habitude.

![Le formulaire de commit après le bouton étincelle : un résumé et une description écrits par le modèle, prêts à être modifiés](../images/ai-commit-message.png)

![La fenêtre Expliquer les modifications locales : une explication et une liste de choses à revérifier](../images/ai-explain.png)

### Le compositeur de commits

Le modèle répartit les modifications locales en plusieurs commits (chaque fichier modifié
dans exactement un ; un plan invalide est renvoyé une fois, puis refusé). Tant que des
fichiers ont encore des conflits, rien n'est demandé : une boîte de dialogue les liste et
ouvre la vue de résolution des conflits, car le compositeur a besoin d'une arborescence
sans conflit. Vous relisez le plan avant que quoi que ce soit soit commité :

![La revue du compositeur : quatre commits proposés avec leurs fichiers, et les boutons Réinitialiser, Annuler et Créer 4 commits](../images/composer.png)

| Touche | Souris | Fait |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | cliquer sur une ligne | se déplacer sur les commits et les fichiers |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | les flèches, glisser-déposer | déplacer un commit ; déplacer un fichier vers un autre commit |
| `Right` / `Left`, `*` | la flèche de la ligne | déplier / replier un commit, tous (`ui.composer_files` règle le départ) |
| `e` / `Enter` | le crayon | modifier le message |
| `d` / `Delete` | `drop` / `restore` | laisser un commit de côté (ses fichiers restent non commités) |
| `n` | `+ own commit` | déplacer un fichier dans un commit à part |
| `r` | `Réinitialiser` | retour au plan du modèle |
| `Ctrl+S` / `a` | `Créer N commits` | les créer, dans l'ordre |
| `Esc` | `Annuler` | quitter, rien n'est fait |

Créer stage les fichiers de chaque commit et les commite à tour de rôle ; l'arbre de
travail n'est jamais touché. Un fichier stagé en partie seulement est commité en
entier. Si une étape échoue, il s'arrête et dit combien de commits ont été faits.

### Scinder un commit

Le modèle lit les modifications du commit, numérotées comme les numérote l'[éditeur de
scission](log-and-history.md#scinder-un-commit), et propose plusieurs commits avec un message
chacun ; il peut regrouper des hunks et des lignes isolées. Une réponse qui oublie une
modification, en assigne une deux fois ou désigne quelque chose d'inexistant est refusée. Dans
les deux cas, l'éditeur de scission s'ouvre : avec la proposition déjà remplie ou, si le modèle
a échoué, avec la raison dans une note pour que vous scindiez à la main. Rien n'est scindé
avant que vous appuyiez sur **Scinder** (`Ctrl+S`) ; déplacez des modifications, renommez des
commits ou ajoutez-en un d'abord si le regroupement ne convient pas. Le profil et les
instructions sont ceux de *Scinder un commit* dans Paramètres > IA.

### Un script de réécriture d'historique

Vous décrivez ce que la réécriture doit faire (« décaler au lendemain matin chaque
commit fait la nuit », « remplacer mon ancienne adresse par la nouvelle, seulement
avant 2024 ») et le modèle écrit un [script Lua](history-rewrite.md#scripts-lua)
pour la règle Script, avec l'en-tête, `rewrite_commit(c, ctx)` et les fonctions
utilitaires du bac à sable. Le démon compile la réponse ; si elle ne compile pas,
l'erreur est renvoyée une fois au modèle. Ce qui reste s'affiche dans l'éditeur :
relisez-le, testez-le, puis lancez l'aperçu. Les détails et la boîte de dialogue
sont dans [Réécriture d'historique](history-rewrite.md#écrire-un-script-avec-lia).
Le profil et les instructions sont ceux de *Script de réécriture d'historique*
dans Paramètres > IA.

### Règles de réécriture d'historique

Vous décrivez avec vos mots comment l'historique doit être réécrit (« remplacer mon ancien e-mail
a@b.org par c@d.org », « retirer secrets.txt de tout l'historique ») et le modèle propose un
périmètre, des réglages et une liste de [règles de réécriture](history-rewrite.md#créer-des-règles-avec-lia)
dans le vocabulaire propre à l'atelier. **Il ne fait que préparer** : la réponse remplit l'atelier,
rien n'est prévisualisé ni appliqué, et une entrée inutilisable est listée avec sa raison pendant
que les règles valides sont gardées. Une règle Script n'est jamais créée ainsi. Facultativement, le
périmètre, les noms des branches et des tags et les auteurs des derniers commits accompagnent la
demande (une case, désactivée par défaut, visible dans le volet « ce qui est envoyé »). Le profil
et les instructions sont ceux de *Règles de réécriture d'historique* dans Paramètres > IA.

## Où va la clé

Une clé d'API (OpenAI, Anthropic, OpenRouter, un serveur compatible OpenAI) est liée à l'adresse pour laquelle elle a été enregistrée : l'origine de l'URL de base du profil, notée dans `trust.toml` dans le dossier de données privé réservé à l'utilisateur. Si l'URL de base pointe plus tard ailleurs, la requête est refusée avant que rien ne soit envoyé et la clé reste où elle est. Lancez `gitpoulpi ai trust <profile> --accept-origin <url>` (ou acceptez-le dans l'application) pour déplacer volontairement la clé vers la nouvelle adresse.

- Le `http://` simple est autorisé sur cette machine (Ollama, LM Studio sur `localhost`) ; partout ailleurs, il faut `gitpoulpi ai trust <profile> --allow-insecure-http on`.
- Un serveur auto-hébergé dont le certificat ne remonte pas à une autorité connue n'est accepté que pour l'empreinte de clé que vous épinglez avec `--accept-certificate <sha256>` ; la poignée de main TLS prouve toujours que le serveur détient cette clé.
- Un profil qui avait déjà une clé avant l'existence de ce contrôle est enregistré silencieusement, à partir de son URL de base actuelle, la première fois qu'il est utilisé.
- Un agent local (`claude -p` et autres) n'a ni clé ni HTTP : il n'y a donc rien à lier.

## Contexte de la revue

Un brouillon de revue de code (`✦`, ou `hosting review suggest`) lit le diff de la demande, son titre, sa
description et ses commentaires existants. On peut aussi lui donner les conventions propres au projet, dans un
bloc **Conventions du projet** placé avant le diff, et un agent local peut travailler dans le dépôt. Les valeurs
par défaut sont :

| Réglage | Par défaut | Signification |
|---|---|---|
| `run_in_repository` | `profile` | `profile` : le *Dossier de travail* du profil IA décide. `yes` : un agent local travaille dans le dépôt pour la revue, quoi que dise le profil. `no` : pas dans le dépôt (un profil réglé sur *Dépôt* utilise à la place le dossier dédié). Les profils HTTP n'ont pas de dossier de travail : cela ne change rien pour eux. |
| `include_claude_md` | activé | Donne au modèle le `CLAUDE.md` du dépôt. |
| `include_agents_md` | activé | Donne au modèle l'`AGENTS.md` du dépôt. |
| `extra_file` | aucun | Un fichier de plus : un chemin relatif à la racine du dépôt, un chemin absolu ou un lien. |

Ils se superposent comme les instructions, le plus spécifique l'emportant champ par champ : la table globale
`[ai.review]`, puis les valeurs propres au dépôt (un champ non défini hérite), puis les choix d'une exécution. Un
dépôt qui n'est pas cloné garde ses propres valeurs par `hébergeur/projet` dans `remote-reviews.toml`, à côté de
ses brouillons de revue.

Un dépôt peut aussi désactiver le fichier supplémentaire global : son `no_extra_file = true` (la case **Aucun fichier
supplémentaire** de *Paramètres propres au dépôt > IA*, de la boîte d'options de la revue ✦ et l'option
`--no-extra-file` de `ai review-context`) signifie « aucun fichier supplémentaire pour ce dépôt », quoi que nomme
`[ai.review]`. Un champ **Fichier supplémentaire** **vide** n'est pas la même chose : il hérite du fichier global. Un
fichier du dépôt et l'interrupteur s'excluent (définir l'un efface l'autre), et l'interrupteur l'emporte si un fichier
modifié à la main contient les deux. Un ancien `extra_file = ""` d'un dépôt signifie toujours aucun.

D'où viennent les fichiers :

- `CLAUDE.md`, `AGENTS.md` et un fichier supplémentaire relatif sont lus à la **racine** du dépôt, sur la
  branche visée par la demande, jamais dans la copie de travail (qui peut être sur une autre branche ou avoir des
  modifications locales). Avec un clone, depuis le commit de base de la demande ; sans clone, depuis l'API de
  l'hébergeur.
- Un chemin absolu est lu sur cette machine.
- Un lien vers un fichier d'un hébergeur (GitHub `.../blob/<branche>/<chemin>`, GitLab
  `.../-/blob/<branche>/<chemin>`, Gitea et Forgejo `.../src/branch/<branche>/<chemin>`, ou un lien brut) est lu
  via le profil d'intégration dont l'hébergeur correspond, donc un dépôt privé fonctionne avec le jeton déjà
  fourni. Un lien qu'aucun profil ne reconnaît, ou toute autre adresse, est récupéré anonymement. Un nom de
  branche contenant une barre oblique ne peut pas être lu depuis un lien : utilisez un chemin, ou un lien sur une
  branche qui n'en a pas.
- Un `CLAUDE.md` ou `AGENTS.md` absent est ignoré sans bruit ; un fichier supplémentaire absent est signalé
  (`context.problems` dans le JSON de la CLI).
- Chaque fichier est coupé à **8 Ko** et l'ensemble des fichiers à **16 Ko**, avec une note visible
  `[CLAUDE.md cut: showing ... of ... bytes]` dans ce que lit le modèle.

Quand un agent local travaille dans le dépôt, le prompt lui dit que les fichiers extraits peuvent ne pas
correspondre à la tête de la demande et que le diff fait foi.

Dans la vue de revue, `s` ouvre la boîte **✦ Options de la revue**, remplie avec ces valeurs : celles du dépôt
par-dessus les globales. Changez-les pour cette exécution seulement, ou cochez *Enregistrer comme valeurs par défaut
de ce dépôt* pour les garder (voir [Options de la revue](review.md#options-de-la-revue)). Les valeurs par défaut se
modifient aussi dans *Paramètres > IA* (le bloc *Revue de code*, quatre lignes) et par dépôt dans *Paramètres propres
au dépôt > IA*, où chaque ligne peut *Utiliser le global* et un bouton *Revenir au global* abandonne les valeurs
propres au dépôt ; la recherche des Paramètres les trouve (`Inclure AGENTS.md`, `Fichier supplémentaire`). Tout ce que
le modèle n'a pas pu recevoir (un fichier supplémentaire absent) ou qui a été coupé pour tenir est affiché sur la
carte du brouillon.

En ligne de commande :

```sh
gitpoulpi ai review-context                      # les valeurs globales par défaut et ce que reçoit une revue
gitpoulpi ai review-context --agents-md off      # enregistrer une valeur globale par défaut
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # suivre de nouveau les valeurs globales
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` ne nécessite aucun modèle et fonctionne avec l'IA désactivée. `ai prompt --action code_review`
affiche le contexte effectif sous `review_context`. Les mêmes quatre réglages sont `ai.review.*` dans
`config.toml` ([référence](config-reference.md#ai-ia-intégrée-désactivée-par-défaut)).

## Limites d'entrée

Chaque entrée que GitPoulpi place dans une requête a une limite de taille, un réglage de
`[ai.limits]` (*Paramètres > IA > Limites*). Ces limites sont une protection propre à
GitPoulpi (la fenêtre de contexte diffère selon le modèle) : augmentez-les quand votre
modèle a la place, baissez-les pour garder des requêtes petites.

| Réglage (`ai.limits.`) | Par défaut | Ce qu'il plafonne |
|---|---|---|
| `conflict_bytes` | 40000 | Les blocs base, courant et entrant d'un conflit de merge |
| `diff_bytes` | 60000 | Le diff d'un message de commit, d'une explication, d'un nom de stash, d'une description de pull request, d'un plan de commits ou d'une scission ; le diff entier d'une revue de code |
| `review_file_bytes` | 12000 | Le diff d'un fichier dans une revue de code |
| `review_text_bytes` | 4000 | La description de la pull request relue |
| `review_threads_bytes` | 6000 | Les commentaires de revue déjà présents |
| `review_context_file_bytes` | 8192 | Un fichier de contexte du projet pour une revue (`CLAUDE.md`, `AGENTS.md`, le fichier supplémentaire) |
| `review_context_bytes` | 16384 | Tous les fichiers de contexte d'une revue ensemble |

**Rien n'est coupé en silence.** Quand une entrée dépasse sa limite, la requête attend et
*Limite de l'IA dépassée* liste ce qui dépasse : quelle partie (le diff, un fichier, la
description, un fichier de contexte), sa taille, la limite et le réglage. Une revue liste
dans une seule boîte toutes les parties qui dépassent. Les boutons, chacun avec sa touche :

- *Tout envoyer* (`T`) : cette requête seulement, rien n'est coupé. Elle passe quand même
  par les contrôles de confidentialité (secrets remplacés, fichiers sensibles exclus) et,
  quand il est activé, par le volet *Ce qui est envoyé*, qui vient ensuite.
- *Envoyer tronqué* (`E`) : ce qui dépasse sa limite est coupé, comme toujours ; chaque
  ligne de la boîte dit ce qui est écarté (la fin du diff, les derniers hunks, les fichiers
  qui ne tiennent pas).
- *Ouvrir le paramètre* (`O`) : annule la requête et ouvre *Paramètres > IA > Limites* sur
  la première limite listée. Le focus commence ici : Enter n'envoie jamais.
- *Annuler* (`A`, `Esc`, la croix ou un clic à l'extérieur) : n'envoie rien.

Un conflit est l'exception : il n'est jamais coupé. *Envoyer quand même* l'envoie en entier,
une fois.

Rien ne lance une requête IA tout seul, donc aucune exécution en arrière-plan ne pourrait
poser la question : chaque requête est une requête que vous avez lancée. Une exécution qui
ne peut pas poser la question (la CLI et MCP) tronque comme avant et le dit ; voir
ci-dessous.

**En ligne de commande**, rien ne peut répondre à une boîte de dialogue : une entrée
au-delà de sa limite est donc tronquée comme avant et la réponse dit ce qui a été coupé :
`truncated: [{part, detail?, bytes, limit, setting}]` (présent seulement quand quelque chose
a été coupé ; `part` vaut `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file` ou `context_total`). `--send-all`
(alias `--no-truncate`) envoie tout en entier à la place : sur `ai` (toutes ses commandes, y
compris `ai resolve-conflict`, où un conflit au-delà de sa limite est sinon refusé), sur
`commit --ai`, sur `stash push --ai` et sur `hosting review suggest`. Les outils MCP ont le
même argument `send_all` et le même champ `truncated`.

## Confidentialité

- Rien n'est envoyé tant que vous n'appuyez pas sur un bouton ou ne lancez pas une
  commande. Pas d'IA en arrière-plan, pas de télémétrie, pas de compte GitPoulpi.
- Ce qui est envoyé : le prompt (intégré, puis vos instructions) et le diff dont vous
  avez parlé ; pour une pull request, aussi les sujets des commits de la
  branche. Un diff de plus de 60 Ko (`ai.limits.diff_bytes`) n'est jamais coupé en
  silence : voir [Limites d'entrée](#limites-dentrée).
- Une demande de script de réécriture envoie votre consigne et le contrat fixe du
  script (l'API du bac à sable). L'auteur, la date et le sujet de 20 commits au
  plus ne partent **que** si vous cochez *Envoyer un échantillon* dans la boîte de
  dialogue (désactivé par défaut ; la CLI exige `--sample N`). Le contenu des
  fichiers et les valeurs secrètes ne sont jamais envoyés.
- Cela va uniquement au profil choisi pour cette action : cet hôte d'API, ou le stdin
  de votre commande d'agent local (qui fait alors ce que fait cet agent).
- Les clés d'API sont utilisées par le processus `gitpoulpi` lui-même et n'atteignent
  jamais le démon, les autres clients de la fenêtre, ni un client MCP.
- Le texte généré n'est jamais commité, poussé ni appliqué de lui-même.

### Ce qui est nettoyé dans une requête

Chaque requête, quelle que soit l'action qui l'envoie et quel que soit le
fournisseur qui la reçoit (une API hébergée ou un agent local), passe d'abord par
la même étape de nettoyage :

- **Les secrets sont masqués** (`[ai.privacy] redact_secrets`, activé par
  défaut). Un jeton à préfixe connu, un bloc de clé privée, un JWT, un mot de
  passe dans une URL et une ligne `password = valeur` dont la valeur paraît
  aléatoire sont remplacés par `[REDACTED:<rule>]`. Ce n'est pas exhaustif : cela
  trouve les formes courantes (le même analyseur que `rewrite scan-secrets`).
- **Les fichiers sensibles sont exclus** (`[ai.privacy] exclude_sensitive_files`,
  activé par défaut). Le diff d'un fichier correspondant à `[privacy]
  sensitive_patterns` (`.env` et `.env.*` sauf les fichiers `.example`, `.sample`
  et `.template`, `*.pem`, `*.key`, les clés privées SSH, `*.p12`, `*.pfx`,
  `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`) ne fait pas partie de
  la requête. Un morceau de scission ou une liste de fichiers du compositeur qui
  nomme un tel fichier est coupé de la même façon : un plan qui le concerne peut
  donc être refusé comme incomplet, indexez ou scindez ce fichier vous-même.

Désactiver l'un ou l'autre est un choix, et Paramètres > IA affiche une ligne
rouge dessous tant qu'il est désactivé.

**Ce qui est envoyé, avant l'envoi.** Avant qu'une requête ne parte, le TUI
affiche la boîte **Ce qui est envoyé** : les instructions et la requête après
nettoyage (le même volet que les boîtes de réécriture d'historique, défilant,
`Alt+W` l'affiche ou le masque), combien de secrets ont été remplacés et quels
fichiers ont été exclus (une règle et un aperçu masqué comme `ghp_****abcd`,
jamais une valeur), une case **Ne plus afficher pour <action>**, et **Envoyer** /
**Annuler** (`Esc`, la croix de fermeture et un clic à l'extérieur annulent ; rien
ne part avant Envoyer). La case écrit `[ai.sent_preview] <action> = false` ;
Paramètres > IA > *Afficher ce qui est envoyé avant...* a une option par action et
**Tout réafficher**. Les deux boîtes de réécriture d'historique montrent déjà leur
requête, elles n'ont donc pas d'option.

**En ligne de commande**, chaque réponse `ai ...` (et `hosting review suggest`)
porte `privacy: {redactions: N, excluded_files: [...]}`, et `--print-sent` affiche
le texte système et utilisateur nettoyé, sous la forme `{sent: [{system, user}],
privacy, dry_run: true}`, sans appeler le modèle (il exige `ai.enabled = true`,
aucune clé et aucun réseau).

**Une limite.** Un agent local dont le dossier de travail est le dépôt
(`workdir = repository`, ou *Lancer l'agent dans le dépôt* pour une revue) peut
lire les fichiers lui-même : le nettoyage couvre ce que GitPoulpi lui envoie, pas
ce qu'il ouvre de lui-même. Utilisez le dossier de travail `dedicated` pour le
tenir hors du dépôt.

## Dépannage

| Symptôme | Remède |
|---|---|
| Aucun `✦` nulle part | L'IA est désactivée : *Paramètres > IA > Activer l'IA*. |
| Le `✦` est gris | Rien n'est stagé (message de commit). |
| *Tester la connexion* : commande introuvable | La **Commande** de l'agent local n'est pas dans le `PATH` que voit GitPoulpi : lancez GitPoulpi là où `claude --version` fonctionne, ou donnez le chemin complet. |
| Un agent local se bloque ou affiche un écran interactif | En mode d'exécution *Headless*, ajoutez son option sans interface (`-p` pour Claude Code) aux **Arguments**, ou rechoisissez son **Type d'agent** pour restaurer l'invocation sans interface. L'écran interactif, c'est à quoi sert le mode d'exécution *Interactif (terminal virtuel)*. |
| "Use "Open the agent"..." | L'agent a posé une question de premier lancement (faire confiance au dossier, se connecter). Utilisez **Ouvrir l'agent...** sur le profil, répondez, fermez l'agent, puis réessayez. |
| Une requête en terminal virtuel dépasse le délai | Le message se termine par les dernières lignes de l'écran de l'agent : une demande d'autorisation, une connexion ou une question de mise à jour s'y voit. Répondez-y avec **Ouvrir l'agent...**, ou augmentez le **Délai**. |
| `failed:` 401 | Mauvaise clé, ou clé d'un autre fournisseur : saisissez-la à nouveau. |
| `failed:` 404 | Le **Modèle** n'existe pas là : videz-le pour utiliser la valeur par défaut. |
| `failed:` erreur de connexion (Ollama, LM Studio) | Le serveur ne tourne pas, ou l'**URL de base** a le mauvais port ou n'a pas `/v1`. |
| La clé affiche `(non défini)` après l'avoir tapée | Pas d'assistant d'identifiants : configurez-en un (`git config --global credential.helper ...`) et saisissez-la à nouveau. |
| *Limite de l'IA dépassée* | Une entrée de la requête dépasse une limite de `[ai.limits]` : choisissez *Tout envoyer*, *Envoyer tronqué*, *Ouvrir le paramètre* ou *Annuler* (voir [Limites d'entrée](#limites-dentrée)). |
| « Ce conflit est plus grand que la limite » | Le conflit dépasse `ai.limits.conflict_bytes` (40000 par défaut). *Envoyer quand même* l'envoie une fois ; *Ouvrir le paramètre* va à la limite dans Paramètres > IA > Limites. |
| Mauvaise langue ou mauvais style | Ajoutez des instructions ; les pastilles Insertion rapide de l'éditeur vont vite. |
| Le compositeur dit que le plan est invalide | Le modèle a sauté ou répété un fichier : réessayez ou utilisez un modèle plus puissant. |

## Anciens fichiers de configuration

Un ancien `config.toml` avec `[[ai.presets]]` et `[ai.defaults]` se charge encore :

- Le préréglage par défaut de chaque action devient son `[ai.actions.<action>]`
  (`system_prompt` devient `instructions`).
- Chaque ancien préréglage est conservé dans `[[ai.legacy_presets]]`. Paramètres > IA
  les montre sous *Anciens préréglages*, avec *Supprimer l'archive*.
- L'ancien `[preferences.ai_presets]` d'un dépôt devient une surcharge de dépôt.

Le prompt intégré vient maintenant toujours en premier. Si un ancien prompt copiait
un prompt intégré, retirez la partie copiée et ne gardez que ce que vous voulez
ajouter.
