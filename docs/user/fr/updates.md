[English](../updates.md) · [Čeština](../cs/updates.md) · [Deutsch](../de/updates.md) · [Español](../es/updates.md) · **Français** · [Italiano](../it/updates.md) · [日本語](../ja/updates.md) · [한국어](../ko/updates.md) · [Nederlands](../nl/updates.md) · [Polski](../pl/updates.md) · [Português (Brasil)](../pt-BR/updates.md) · [Português (Portugal)](../pt-PT/updates.md) · [Русский](../ru/updates.md) · [Svenska](../sv/updates.md) · [Türkçe](../tr/updates.md) · [Українська](../uk/updates.md) · [简体中文](../zh-CN/updates.md)

# Mises à jour

GitPoulpi recherche les versions plus récentes et, selon la façon dont il a été installé, remplace lui-même ses fichiers ou confie la mise à jour au programme qui l'a installé. Rien n'est jamais remplacé ni redémarré sans votre action.

## La pastille et le panneau

La barre de titre affiche la version en cours (`v1.0.0`) entre le nom de l'application et le chemin du dépôt. Quand une version plus récente est connue, la pastille devient `v0.2.0 disponible` dans la couleur d'accent.

Un clic sur la pastille ouvre le panneau de mise à jour seulement quand une version plus récente existe ou qu'une mise à jour est en cours ; sinon il ouvre l'onglet **Notes de version**. Tapez `Ctrl+N` puis `u`, ou lancez **Mises à jour : Afficher la mise à jour** depuis la palette de commandes (`Ctrl+P`) : le panneau de mise à jour s'ouvre. C'est une fenêtre par-dessus l'application : `Esc`, `q`, la croix de fermeture ou un clic à l'extérieur la ferme. Il affiche :

- la version en cours et **Installé avec** : comment GitPoulpi est arrivé sur cet ordinateur (cela décide de tout ce qui suit) ;
- où en est la mise à jour (recherche, téléchargement avec un pourcentage, prête, échec) ;
- les boutons de la procédure de cette installation, et la commande exacte quand il y en a une ;
- une ligne de votre administrateur quand il en a défini une.

Chaque bouton a une touche (indiquée dans la ligne d'aide du panneau) et une commande de palette, le panneau fonctionne donc sans souris :

| Bouton | Touche | Commande de palette |
|---|---|---|
| **Mettre à jour et redémarrer** (ou **Lancer la mise à jour**) | `u` | Mises à jour : Mettre à jour maintenant |
| **Copier la commande** | `c` | Mises à jour : Copier la commande de mise à jour |
| **Ouvrir dans un terminal** | `t` | Mises à jour : Exécuter la commande de mise à jour dans un terminal |
| **Revenir à la version X** | `v` | Mises à jour : Revenir à la version précédente |
| **Notes de version** | `n` | Aide : Notes de version |
| **Ignorer cette version** | `s` | Mises à jour : Ignorer cette version |
| **Ouvrir l'avis** (seulement avec un avis de sécurité) | `a` | Mises à jour : Ouvrir l'avis de sécurité |
| **Rechercher** | `r` | Mises à jour : Rechercher une mise à jour maintenant |
| **Fermer** | `Esc` | |

Une commande qui ne s'applique pas à cette installation est grisée dans la palette, avec la raison (par exemple « cette installation se met à jour autrement : voir Mises à jour : Afficher la mise à jour »).

## Ce qui se passe, selon l'installation

| Installé avec | Recherche auprès de | Ce que fait le panneau | Retour en arrière |
|---|---|---|---|
| Une archive de version, ou `install.sh` | Les versions GitHub | **Mettre à jour et redémarrer** : GitPoulpi télécharge et vérifie l'archive, se ferme, remplace ses fichiers et se rouvre dans le même terminal. | **Revenir** remet les fichiers précédents. |
| L'installeur Windows, pour moi seulement | Les versions GitHub | Pareil, sans demande d'autorisation. Sous Windows, il se rouvre dans une nouvelle fenêtre. | **Revenir** remet les fichiers précédents. |
| L'installeur Windows, tous les utilisateurs | Les versions GitHub | Pareil, mais Windows demande l'accord d'un administrateur. Les sessions GitPoulpi des autres utilisateurs de l'ordinateur sont fermées, ce qui fait perdre leur saisie non enregistrée. Si vous refusez, rien ne change. | **Revenir** remet les fichiers précédents, avec la même demande. |
| Chocolatey | Le flux communautaire Chocolatey | **Lancer la mise à jour** exécute `choco upgrade gitpoulpi -y` (Windows demande l'accord d'un administrateur). | Le panneau affiche `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, flux interne | Le flux de votre organisation | **Lancer la mise à jour** exécute `choco upgrade gitpoulpi -y --source=<feed>`. Les paramètres du paquet ne sont pas renvoyés, une configuration déployée reste donc intacte. | La même ligne `choco install` avec le flux. |
| Scoop | Les versions GitHub | **Lancer la mise à jour** exécute `scoop update gitpoulpi` une fois GitPoulpi complètement fermé. | **Revenir** exécute `scoop reset gitpoulpi@<previous>`. |
| Homebrew | Les versions GitHub | **Lancer la mise à jour** exécute `brew upgrade gitpoulpi` dans ce terminal. | Le panneau affiche `brew install gitpoulpi@<previous>`. |
| apt | Les versions GitHub | Le panneau affiche `sudo apt update && sudo apt install --only-upgrade gitpoulpi` avec **Copier la commande** et **Ouvrir dans un terminal**. GitPoulpi ne lance jamais `sudo` à votre place. Sans le dépôt apt de GitPoulpi sur cet ordinateur, il indique de télécharger plutôt le nouveau `.deb`. | Le panneau affiche `sudo apt install gitpoulpi=<previous>`. |
| Un paquet MSI | Les versions GitHub, ou la source de votre administrateur | Rien à lancer : l'installation est gérée par votre administrateur, dont le panneau affiche le message. | Non proposé. |
| cargo | Les versions GitHub | Le panneau affiche `cargo binstall gitpoulpi-cli` quand `cargo-binstall` est installé, sinon `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | La même commande avec l'étiquette précédente. |
| Une image Docker | Les versions GitHub | Rien à lancer : une image se remplace. Le panneau affiche `docker pull poulpilab/gitpoulpi:<version>` et `:latest`. Voir [Exécuter dans Docker](docker.md). | Le panneau affiche `docker pull poulpilab/gitpoulpi:<previous>`. |
| Une archive de version dans un dossier où vous ne pouvez pas écrire | Les versions GitHub | Le panneau indique que GitPoulpi ne peut pas se remplacer à cet endroit et renvoie vers **Télécharger**. | Non proposé. |
| Une version de développement (`cargo build`) | aucune | Ne se met jamais à jour. La pastille affiche la version et rien d'autre ; le panneau indique qu'il n'y a pas de trace d'installation. | Non proposé. |

Les lignes qui se contentent d'afficher une commande n'exécutent jamais rien d'elles-mêmes. **Ouvrir dans un terminal** ouvre votre terminal avec la commande déjà saisie, et vous appuyez sur Enter.

Les versions viennent toujours du dépôt GitHub du projet, sauf si un administrateur oriente la recherche ailleurs (un flux interne, un miroir ou un `latest.json`), voir [Distribution en entreprise](enterprise-distribution.md). Les notes de version viennent de la version ou du journal des modifications intégré à GitPoulpi, selon ce qui est accessible.

## Notifier et Auto

`updates.mode` ([Paramètres > Synchronisation](settings.md#synchronisation)) :

- **Notifier** (par défaut) : une fois par jour, GitPoulpi cherche en arrière-plan et affiche la pastille. Rien n'est téléchargé tant que vous n'appuyez pas sur le bouton.
- **Auto** : là où GitPoulpi remplace lui-même ses fichiers (archive de version, installeur Windows), il télécharge aussi et vérifie la mise à jour en arrière-plan dès qu'il en trouve une. La pastille affiche alors **Redémarrer pour mettre à jour**, et le redémarrage reste votre choix. Là où un gestionnaire de paquets s'occupe de la mise à jour, **Auto** se comporte comme **Notifier**.
- **Désactivé** : aucune requête n'est jamais faite et la pastille n'affiche que la version.

Le paramètre **Canal de mise à jour** (`updates.channel`) choisit entre **Stable** et **Préversion** (versions candidates comprises). Une recherche lancée avec **Rechercher** ignore la limite d'une fois par jour. Sans connexion réseau, ou tant que le dépôt est privé et qu'aucun jeton github.com n'est stocké, une recherche en arrière-plan échoue discrètement, avec une ligne dans le journal de l'application.

## Redémarrage et ce que vous voyez ensuite

Appuyer sur **Mettre à jour et redémarrer** ou **Lancer la mise à jour** enregistre votre session et ferme la fenêtre. Un petit programme distinct, `gitpoulpi-updater`, fait le travail : les fichiers de GitPoulpi ne sont donc pas utilisés pendant qu'ils sont remplacés, et un gestionnaire de paquets n'est pas bloqué par un parent en attente. Il arrête d'abord les démons d'arrière-plan de GitPoulpi (les autres fenêtres et serveurs MCP de votre utilisateur se ferment aussi), fait la mise à jour, vérifie que le nouveau `gitpoulpi --version` affiche la version attendue, puis relance GitPoulpi : dans le même terminal sous Linux et macOS, dans une nouvelle fenêtre sous Windows. Si la vérification échoue, il remet les anciens fichiers.

Le démarrage suivant affiche une ligne dans la barre d'état :

- `Mis à jour vers la v1.4.2`
- `Échec de la mise à jour : <raison>. La version précédente reste installée.`
- `Mise à jour annulée : l'accord de l'administrateur a été refusé. Rien n'a changé.`

Une seule mise à jour s'exécute à la fois ; un fichier de verrou dans le dossier de données de GitPoulpi (et un autre dans le dossier d'installation quand il est accessible en écriture) empêche une seconde mise à jour, et un verrou de plus de dix minutes est ignoré : une mise à jour plantée ne bloque donc jamais GitPoulpi pour de bon. La CLI et le démon ne démarrent pas tant qu'un verrou récent existe.

## Revenir en arrière

Là où GitPoulpi remplace lui-même ses fichiers, **Revenir** remet les fichiers précédents et redémarre. Ils restent sous le nom `<name>.old` pendant une minute après le démarrage de la nouvelle version, puis sont supprimés : le retour en arrière sert donc pour le premier lancement après une mise à jour. Là où un gestionnaire de paquets s'occupe de l'installation, le retour en arrière est la commande du tableau : le panneau l'affiche ou la lance. Revenir modifie les propres fichiers de GitPoulpi, pas un dépôt : cela ne fait donc pas partie du journal d'annulation git.

## Ce qui est vérifié et refusé

Avant de remplacer quoi que ce soit, GitPoulpi télécharge l'archive, sa signature `.minisig` et son `.sha256`. Il vérifie d'abord la signature, avec la clé de version du projet intégrée à GitPoulpi, et le commentaire signé de la signature doit désigner cette archive et cette version. Il vérifie ensuite le hash. Il refuse : une version sans `.minisig` (une version plus ancienne, publiée avant la signature des versions, ne peut pas être installée par le programme de mise à jour, ce qui inclut le fait de s'y fixer ou d'y revenir), une signature d'une autre clé, une signature pour un autre fichier ou une autre version, une version sans `.sha256`, un hash qui ne correspond pas, une archive dont le nom n'est pas celui de cette plateforme (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), une archive dont le marqueur désigne une autre version ou plateforme, et un lien de téléchargement en dehors des hébergeurs de versions de GitHub (chaque redirection est vérifiée) ou du miroir configuré par un administrateur. Le remplacement est tout ou rien : si un autre programme retient l'un des exécutables, GitPoulpi nomme le fichier, ne change rien et laisse le téléchargement pour un nouvel essai. Sous Windows, une mise à jour pour tous les utilisateurs est revérifiée par l'étape élevée, qui télécharge elle-même le `.minisig` et le `.sha256` et revérifie les deux : un fichier substitué entre-temps dans le dossier de téléchargement de l'utilisateur est donc refusé. Les gestionnaires de paquets (Chocolatey, Scoop, Homebrew, apt) vérifient leurs propres paquets : GitPoulpi n'y télécharge rien.

## Avis de sécurité

À côté de chaque version, le projet publie `security.json` : la liste des avis de sécurité (un identifiant, une gravité, les versions concernées sous forme de plage comme `>=1.2.0, <1.4.3`, la version corrigée, un lien et un résumé en une phrase). La même recherche quotidienne qui cherche une nouvelle version le récupère aussi, au même endroit que la version : la page de version du dépôt pour GitHub, le dossier de `latest.json` pour un miroir, GitHub pour le flux Chocolatey public. Un avis n'est utilisé que si `security.json.minisig` se vérifie avec la clé du projet ; un fichier non signé ou mal signé n'est jamais affiché (une ligne du Journal d'activité indique qu'il a été ignoré).

Quand un avis désigne la version en cours, la pastille devient rouge et affiche **Avis de sécurité : cette version est concernée**, sur toutes les installations, y compris celles gérées par un gestionnaire de paquets. Le panneau de mise à jour liste alors chaque avis avec sa gravité, son résumé, sa version corrigée et son lien ; **Ouvrir l'avis** (`a`, ou **Mises à jour : Ouvrir l'avis de sécurité** dans la palette) ouvre dans le navigateur le lien de l'avis le plus grave. Là où un gestionnaire de paquets s'occupe de l'installation, le panneau ajoute « Mettez à jour avec votre gestionnaire de paquets. »

Avec la recherche de mises à jour désactivée (`updates.mode = "off"`), avec une politique d'administrateur dont `check` vaut `none`, et pour une version compilée avec `cargo build`, rien n'est récupéré et aucun avis n'est affiché. Le flux NuGet propre à une organisation ne fait pas contacter GitHub à GitPoulpi : ses avis ne viennent que du miroir `download_base` de la politique.

## Paramètres verrouillés par un administrateur

Un administrateur peut définir le mode de mise à jour, le canal, une version figée, l'endroit où chercher et si GitPoulpi a le droit de se mettre à jour lui-même, dans une politique valable pour toute la machine. Dans les Paramètres, ces lignes affichent **Défini par votre administrateur**, le panneau affiche leur message, et quand l'auto-mise à jour est désactivée, le panneau propose les commandes ou une note à la place des boutons qui remplacent des fichiers. Un fichier de politique illisible échoue par précaution : notification seulement, pas d'auto-mise à jour. Le côté administrateur est dans [Distribution en entreprise](enterprise-distribution.md#le-fichier-de-politique-et-le-registre).

## Environnement

`GITPOULPI_UPDATE_API` oriente la recherche vers une autre racine d'API (utilisé par les tests).
