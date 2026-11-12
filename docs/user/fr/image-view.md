[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · **Français** · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Les images dans la vue Diff et la vue Fichier

Un fichier binaire qui est une image s'affiche comme une image, pas avec l'avis
« (no textual changes - binary content...) ». La vue Diff montre l'**avant** et
l'**après** ; la vue Fichier montre le fichier tel qu'il est ; l'éditeur de
merge montre les côtés d'une image en conflit. Tout ici est en lecture seule :
regarder une image ne modifie jamais le dépôt, il n'y a donc rien à annuler.

Désactivez-le avec **Paramètres > Images > Aperçu des images** (`ui.image_view`) :
vous retrouvez l'avis binaire simple. Un build personnalisé sans la
fonctionnalité `image-view` n'a aucun visualiseur et la page Images ne garde
que la liste des éditeurs (voir [Builds personnalisés](custom-builds.md)).

## Quels fichiers

PNG et JPEG toujours ; GIF, WebP (leur première image), BMP et ICO/CUR quand le
build a `image-view`. TIFF, AVIF, HEIC, PSD et RAW sont reconnus mais pas
décodés : une fiche d'information donne le format, la taille et les
dimensions. SVG est du texte et garde le diff texte. Une image de plus de
**Taille max de l'aperçu (Mio)** (32 Mio par défaut) ou de plus de 64
mégapixels affiche une fiche d'information à la place. Un fichier stocké avec
Git LFS s'affiche quand son objet est dans le stockage local ; sinon la fiche
dit de faire d'abord `LFS > Pull`. La transparence est dessinée sur un damier.

## Comment c'est dessiné

Sur un terminal avec un protocole d'image (Kitty et Ghostty, WezTerm et iTerm2,
Rio, mintty, et les terminaux sixel Windows Terminal, foot et Tabby), l'image est une vraie image. Ailleurs (la console Windows simple,
zellij, un terminal que GitPoulpi ne connaît pas), elle est dessinée avec des
caractères demi-blocs colorés, moitié moins de résolution verticale mais
lisible partout ; **Sans protocole d'image** peut être réglé sur *Fiche
d'information seule*. **Protocole d'image** dans la page **Journal et graphe** en force
un.

## La vue Diff

Ouvrez le diff d'un fichier image (celui d'un commit, la moitié unstaged ou
staged des modifications locales). L'en-tête nomme les deux fichiers (`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`),
puis les boutons de zoom, **Ajuster**, **1:1** et **Éditer avec**.

- **Côte à côte | En ligne** à droite de l'en-tête, ou `m` : côte à côte, ou
  une image à la fois. **Tab**, ou un clic sur *Avant | Après*, montre l'autre
  dans la vue en ligne. Une vue côte à côte trop étroite pour deux volets est
  dessinée en ligne sans changer votre paramètre (`ui.diff_layout`).
- Le **séparateur** d'une vue côte à côte se déplace en le glissant, avec
  `Alt+Right` / `Alt+Left` (le volet avant prend / cède de la place) et
  revient au milieu par un double-clic ou `Alt+0`. La proportion est mémorisée
  pour la session.
- Un nouveau fichier montre un seul volet légendé *Nouveau fichier*, un fichier
  supprimé *Fichier supprimé* ; les deux masquent le commutateur. Une image non
  suivie sélectionnée dans les modifications locales est prévisualisée comme un
  nouveau fichier.
- Le côté avant est le commit parent (diff de commit), `HEAD` (staged) ou
  l'index (unstaged) ; le côté après est le commit, l'index ou le fichier sur
  disque. Si le fichier sur disque change pendant que le diff est ouvert (vous
  l'avez enregistré depuis un éditeur, un outil l'a réécrit), l'image suit en
  une demi-seconde environ.

## La vue Fichier

Ouvrez une image avec **Vue fichier** depuis le menu d'un fichier ou la barre du
diff : un seul volet, le fichier à la révision (ou sur disque pour les
modifications locales).

## Zoom et défilement

| | |
|---|---|
| `+` / `-` (ou `=`) | Zoom avant / arrière, autour du centre du volet |
| `0` | **Ajuster** (par défaut) : l'image entière, qui reste ajustée quand la fenêtre ou le séparateur est redimensionné |
| `1` | Taille réelle (1:1) |
| Flèches, `h j k l` | Défiler d'un dixième du volet ; avec `Shift` d'un volet entier ; `PageUp`/`PageDown`, `Home`/`End` |
| Molette | Défilement vertical ; `Shift`+molette horizontalement ; `Ctrl`+molette zoome autour du pointeur |
| Glisser | Attraper l'image et la déplacer ; les barres de défilement sur les côtés fonctionnent comme partout |

Une vue zoomée garde son échelle et le point à son centre quand le volet est
redimensionné ; Ajuster reste Ajuster. Dans la vue Diff, les deux volets
partagent un zoom et une position de défilement (**Synchroniser zoom et défilement avant/après**,
activé par défaut), de sorte que la même partie des deux images est
visible ; quand leurs tailles diffèrent, le zoom est conservé relativement à
chacune. À partir de **Zoom pixellisé à partir de** (2x par défaut), l'image
est agrandie en pixels carrés au lieu d'être lissée, ce qui convient aux
icônes et au pixel art.

## Éditer avec

Le bouton **Éditer avec** de l'en-tête liste les éditeurs d'images trouvés sur
cette machine (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Aperçu, ... et
l'application par défaut du système) ; `o` lance celui par défaut et `Shift+O`
ouvre la liste. Il est aussi dans le menu clic droit d'une image dans les
listes de fichiers, et dans la palette de commandes (*Image : Éditer avec...*).
La liste et le défaut se règlent sous **Paramètres > Images**, comme
[Ouvrir dans](open-in.md).

Éditer le fichier sur disque (le côté des modifications locales) est direct :
l'éditeur enregistre sur place et l'image se rafraîchit. Toute autre version
(celle d'un commit, l'index, un fichier supprimé) est écrite dans une **copie
temporaire en lecture seule**, avec un avis sur la ligne d'état, de sorte
qu'un éditeur ne peut pas modifier le dépôt.

Les applications que le système associe au type de l'image (sa liste *Ouvrir avec*) suivent vos éditeurs dans la liste, sans répéter un programme déjà présent. Les autres fichiers binaires ont la même liste sous [Ouvrir dans](open-in.md).

## Dans l'éditeur de merge

Une image en conflit affiche sa carte avec les images **A**, **B** et (quand il
y en a une) **Base** dans les cases, côte à côte, chacune ajustée à sa case et
en lecture seule (pas de zoom là ; les images ne sont là que pour vous aider à
choisir). Les boutons et les touches ne changent pas : **Garder A** (`a`),
**Prendre B** (`b`), **Prendre la base** (`o`), **Ouvrir dans l'outil de merge**
(`m`). Les aperçus demandent une fenêtre assez large (environ 70 colonnes) et
quelques lignes libres ; plus étroite, la carte est celle, textuelle, simple.
Voir [Résoudre les conflits de merge](merge-conflicts.md).

## Touches et paramètres

Les touches sont dans [Raccourcis clavier](keybindings.md#vue-image) ; chacune
est réaffectable et accessible depuis la palette (`Ctrl+P`, tapez *Image*). Les
options sont sur la page **Images** des [Paramètres](settings.md#images) et dans la
[référence de configuration](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
