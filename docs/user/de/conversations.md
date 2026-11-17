[English](../conversations.md) · [Čeština](../cs/conversations.md) · **Deutsch** · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Unterhaltungen

Die Unterhaltungsansicht eines Issues, Work Items, einer Karte oder eines Pull-/Merge-Requests. Sie braucht ein Konto und ein verknüpftes Repository: siehe [Hosting- und Issue-Tracker-Integrationen](hosting-integration.md); die Listen, die sie öffnen, stehen in [Pull Requests und Issues](pull-requests-and-issues.md); Code reviewen steht in [Einen Pull oder Merge Request prüfen](review.md).

Eine **Unterhaltung** ist alles, was zu einem Issue, Work Item, einer Karte, einer Änderung oder einem Pull-/Merge-Request gesagt wurde: die Beschreibung, die Kommentare, die Reviews, die Threads am Code und die Ereignisse (Labels, Zuweisungen, Statusänderungen, Pushes). GitPoulpi liest sie vom Host, zeigt sie formatiert und lässt Sie antworten, ohne das Fenster zu verlassen.

### Eine öffnen

| Von | Wie |
|---|---|
| Eine ISSUES-Zeile | `Enter` oder ein Doppelklick; **Unterhaltung öffnen** im Zeilenmenü; die Palette *Hosting: Issue-Unterhaltung öffnen...* (eine Liste der Issues auf dem Bildschirm). **Im Browser öffnen** im Zeilenmenü bleibt beim Browser. |
| Eine PULL-REQUESTS-Zeile bei Azure DevOps oder Bitbucket | `Enter` oder **Unterhaltung öffnen**. Bei GitHub, GitLab, Gitea, Forgejo und Gerrit öffnet die Zeile das [Review](review.md), dessen Übersichtsseite genau diese Unterhaltung ist (die Zeitleiste, die Tasten und das Kommentarfeld darunter). |
| Eine Karte des [Issue-Boards](pull-requests-and-issues.md#das-issue-board) | `Enter` oder ein Doppelklick. |
| Das [Radar](radar.md) | `i` auf einer Issue- oder Request-Zeile, **Unterhaltung öffnen** im Zeilenmenü oder *Radar: Issue-Unterhaltung öffnen*. |
| Ein Verweis in einer anderen Unterhaltung | Anklicken (oder `l`, dann auswählen): `#7`, `!12`, `ABC-12` einer verknüpften Quelle öffnen sich in der App und kommen auf einen Zurück-Stapel. |
| Ein Agent | `gitpoulpi ui show issue <n> --view conversation` (auch `ui show pr`), siehe [die CLI-Referenz](cli-reference.md). |

**Repositories, die nicht geklont sind.** Eine Radar-Zeile eines Repositorys, von dem Sie keinen Klon haben, öffnet ebenfalls ihre Unterhaltung: Die Ansicht braucht nur das Integrationsprofil und das Projekt, daher zeigt das Radar sie über seinem eigenen Fenster, und `Esc` gibt das Radar zurück. Was einen Klon braucht, entfällt: Ein Commit-Verweis öffnet sich auf dem Host statt im Log, und *Die KI das Repository lesen lassen* ist mit seinem Grund ausgegraut (siehe [das KI-Kapitel](ai.md#schreibhilfe-auf-einem-host)). Ein Request eines geklonten Repositorys hat stattdessen sein Review (`e`), und **Unterhaltung öffnen** im Radar sagt, dass es dafür hier keine Unterhaltung zu lesen gibt; ein Issue eines geklonten Repositorys öffnet sie im Tab dieses Repositorys.

Ein Issue eines Hosts, der keine zu lesen hat (Bitbucket und Gerrit haben keine Issues), öffnet wie zuvor den Browser, und ein Build ohne das Feature `conversation` tut das überall ([eigene Builds](custom-builds.md)).

### Die Ansicht

Eine Ansicht im ganzen Fenster: die Kopfzeile (Status, wer es wann geöffnet hat, die Labels, die Zahl der Kommentare, **Ereignisse zeigen**, **Zurück**, **Neu laden**, **Im Browser öffnen**), die Zeile **Senden als**, die Zeitleiste und darunter das Kommentarfeld.

![Die Unterhaltung von Issue #7: die Beschreibung mit einem Link auf #9, zwei Kommentare, ein Label-Ereignis, das Ereignisse zeigen einblendet, und das leere Feld Neuer Kommentar mit Schreiben, Vorschau und Geteilt](../images/issue-conversation.png)

![Die Seite Übersicht von Merge Request !58: die Beschreibung, ein offener und ein gelöster Code-Thread mit ihren Datei- und Zeilenlinks, die Kommentare, Antworten und Zitieren beim ausgewählten Thread und das Kommentarfeld](../images/pr-conversation.png)

- Die Zeitleiste liest sich vom ältesten zum neuesten Eintrag und beginnt im neuesten Fenster (höchstens 300 Einträge). **Ältere laden** an ihrem Anfang (`m`, Palette *Unterhaltung: Ältere Einträge laden*) liest das Fenster davor, mit der Zahl der noch ausstehenden, wenn der Host sie nennt.
- Ein Kommentar zeigt seinen Autor, das Datum, ein Chip `bearbeitet`, ein Chip `Bot` bei Bots, `intern` bei privaten Notizen und die Schaltflächen **Antworten**, **Zitieren**, **Bearbeiten**, **Löschen** und **Links**, die der ausgewählte Eintrag erlaubt. Ein Thread am Code zeigt `Pfad:Zeile`, `gelöst` oder `ungelöst`, `veraltet` und seine Kommentare. Ein Review zeigt sein Urteil.
- Ereignisse sind je eine gedämpfte Zeile (*carol hat das Label lfs hinzugefügt*). **Ereignisse zeigen** (`v`) blendet sie aus; die Wahl wird je Repository gemerkt. Hosts ohne Ereignisverlauf (YouTrack, Work Items von Azure DevOps, lokale Notizen) haben dieses Kästchen nicht.
- Der Text wird als Markdown dargestellt: Überschriften, verschachtelte Listen und Aufgabenlisten, Zitate, Hinweisblöcke im GitHub-Stil, hervorgehobener Code, Tabellen, Links, Bilder als `[image: alt]`. Text, der im eigenen Markup des Hosts ankam, wird zuerst umgewandelt (siehe [Markup-Umwandlung](#markup-umwandlung)). Ein `mermaid`-Codeblock erhält eine Schaltfläche **Diagramm öffnen**, die den Quelltext des Diagramms in einer scrollbaren Ansicht zeigt (`Enter` kopiert ihn); die Palette hat *Unterhaltung: Diagramm des Eintrags öffnen*. Dafür braucht es einen Build mit `conversation` und `mermaid`.
- In der Übersicht des Reviews ist `Pfad:Zeile` eines Code-Threads ein Link zum Code (`G`, *Unterhaltung: Code des Threads öffnen*;
  siehe [Review](review.md)); in einer allein geöffneten Unterhaltung (ein Issue, eine Radar-Zeile) gibt es keine Code-Seite, zu der man gehen könnte.
- Die Ansicht liest alle 60 Sekunden und mit `F5` (**Neu laden**) erneut.

**Tasten.** Jede hat eine Schaltfläche, eine Menüzeile oder eine Palettenzeile (*Unterhaltung: ...*), und `Ctrl+N` versieht die Schaltflächen mit Kennzeichen.

| Taste | Aktion |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Vorheriger / nächster Eintrag |
| `PageUp`, `PageDown`, `Home`, `End`, das Mausrad, die Bildlaufleiste | Scrollen |
| `Tab`, `Shift+Tab` | Zeitleiste, Kommentarfeld, dann die Schaltflächen der Kopfzeile |
| `r` | Antworten: in den Thread des ausgewählten Eintrags, oder mit zitiertem Text, wo der Host keine Threads hat |
| `Q` | Den ausgewählten Eintrag in einem neuen Kommentar zitieren |
| `c` | Neuer Kommentar |
| `e`, `d` | Einen eigenen Kommentar bearbeiten, löschen (vor dem Löschen kommt eine Rückfrage) |
| `l`, `Enter` | Die Links des Eintrags: `Enter` folgt dem einzigen oder öffnet das Menü der Links |
| `y`, `o` | Die Adresse des Eintrags kopieren, ihn im Browser öffnen |
| `n` | Nächster ungelesener Eintrag |
| `v`, `m`, `F5` | Ereignisse zeigen oder ausblenden, Ältere laden, Neu laden |
| `Alt+Left`, `Backspace` | Zurück entlang des Zurück-Stapels |
| `q`, `Esc` | Schließen (fragt zuerst, wenn das Feld Text enthält) |

### Schreiben

**Neuer Kommentar** (`c`, oder `Tab` zum Feld), **Antworten** (`r`), **Zitieren** (`Q`) und **Bearbeiten** (`e`) nutzen das eine Markdown-Textfeld: **Schreiben**, **Vorschau** und **Geteilt** (`Alt+P` tauscht Schreiben und Vorschau, `Alt+L` schaltet Geteilt ein und braucht ein Fenster mit 100 Spalten). `Ctrl+Enter` (auch `Ctrl+S` und `Alt+Enter`) oder **Senden** sendet, **Abbrechen** verwirft das Feld, und `Alt+E` öffnet **Mit KI schreiben**, wo KI an ist ([das KI-Kapitel](ai.md#schreibhilfe-auf-einem-host)). Die Vorschau wird vom selben Renderer wie die Zeitleiste dargestellt, mit denselben Verweisen. Nimmt der Host kein Markdown an, nennt die Fußzeile das Markup, in dem der Text gesendet wird (*Gesendet als Jira wiki markup*, *Gesendet als Textile*).

![Eine Issue-Antwort wird in der Ansicht Geteilt geschrieben: der Markdown-Entwurf links, seine gerenderte Vorschau rechts und Senden](../images/issue-reply-composer.png)

- Der Kommentar erscheint sofort in der Zeitleiste und nimmt seinen Platz ein, wenn der Host antwortet. Lehnt der Host ab, verschwindet der Eintrag, und das Feld öffnet sich wieder mit Ihrem Text und dem Grund.
- Ein Text, den Sie nicht gesendet haben, wird je Element gemerkt (`remote-conversations.toml` im Konfigurationsordner) und ist beim nächsten Öffnen dieses Felds wieder da; Senden oder Verwerfen entfernt ihn. Das Schließen eines Felds mit Text fragt zuerst.
- **Antworten** auf einen Thread (einen Review-Thread, eine GitLab-Diskussion, einen Linear-Kommentar, einen Bitbucket- oder Azure-DevOps-Thread) sendet in diesen Thread. Wo der Host nur eine flache Liste hat, beginnt **Antworten** einen neuen Kommentar mit dem ausgewählten Eintrag als Zitat (`> **Autor**`), genau wie **Zitieren**.
- **Interne Notiz** (Kästchen, `Alt+I`): Bei GitLab und Redmine ist der Kommentar nur für Projektmitglieder sichtbar. Es wird nur bei Hosts gezeigt, die solche Notizen haben, und nur bei einem neuen Kommentar.
- Das Senden ist eine nach außen wirkende Aktion: Nichts wird im Journal erfasst, und **nichts lässt sich aus GitPoulpi rückgängig machen**. Ohne Verbindung bleibt das Feld, **Senden** ist mit dem Grund ausgegraut, und die Kopfzeile sagt, dass die gespeicherte Kopie gezeigt wird.

**Wer bearbeiten oder löschen darf.** Nur Ihre eigenen Kommentare tragen **Bearbeiten** und **Löschen**. Das eigene Wort des Hosts kommt zuerst, wenn er eines gibt (`viewerCanUpdate` und `viewerCanDelete` bei GitHub, anderswo `can_edit`), dann Ihr Login gegen den Autor des Kommentars, ohne Beachtung der Groß- und Kleinschreibung verglichen; die Kommentare eines App-Bots müssen außerdem von einem Bot-Konto stammen. Solange der Login eines GitHub-App-Bots nicht bekannt ist, zählen nur Kommentare, die diese Installation gesendet hat (das Repository behält ihre IDs): Aus einem Namen wird nichts erraten. In einem Thread wirken **Bearbeiten** und **Löschen** auf den letzten Kommentar darin, der Ihrer ist. Ein Host, der nicht bearbeiten oder löschen kann (siehe [die Tabelle](conversations.md#was-jeder-host-nicht-kann)), zeigt die Schaltflächen mit dem Grund ausgegraut.

### Senden als

Die Zeile unter der Kopfzeile sagt, als wer Ihre Nachrichten gesendet werden, und dieselbe Zeile steht in jedem Feld:

| Zeile | Bedeutung |
|---|---|
| *Senden als ann (persönliches Token)* | Das Token des Profils: Ihr eigener Login (`GET /user`). |
| *Senden als ci-bot (Token-Befehl des Repositorys)* | Der [Token-Befehl](hosting-integration.md#8-als-github-app-bot-handeln) des Repositorys wird für die Hosting-API verwendet. |
| *Senden als my-app[bot] (GitHub-App-Bot)* | Ein Installations-Token. Der Name kommt aus **Senden als**, sonst vom Host. |
| *Senden als GitHub-App-Bot (sein Name ist noch unbekannt: „Senden als“ in den Einstellungen setzen)* | Kein Name eingestellt, noch keiner gelernt. |
| *Senden als ... (Projekt-Bot)* | Ein GitLab-Projekt- oder Gruppen-Zugriffstoken. |
| *Schreiben als kai (lokale Notiz)* | Ein Ordner mit Markdown-Notizen: das `local.me` des Profils, sonst der verbundene Benutzer. |
| *Ihr Sendename ist noch unbekannt. Setzen Sie „Senden als“ in den Einstellungen.* | Es ließ sich nichts lernen. |

**Senden als** ist ein Name: Für eine GitHub App ist es `my-app[bot]`. Stellen Sie ihn für jedes Repository unter **Einstellungen > Integrationen > Senden als** (`general.hosting_post_as`) ein, oder für ein Repository unter **Einstellungen > Repository-spezifische Einstellungen > SSH & Zugangsdaten > Hosting**, was ihn überschreibt. Bleibt er leer, fragt GitPoulpi GitHub (GraphQL `viewer { login }`, was ein Installations-Token beantworten kann oder nicht) und lernt den Namen andernfalls vom Autor des ersten Kommentars, den es sendet, und behält ihn beim Repository. Ein persönliches Token braucht ihn nie. Siehe [Als GitHub-App-Bot handeln](hosting-integration.md#8-als-github-app-bot-handeln).

### Querverweise

Verweise in einer Unterhaltung sind unterstrichen und anklickbar. Die verknüpften ISSUES- und PULL-REQUESTS-Quellen des Repositorys werden zuerst abgeglichen, sodass `ABC-12` eines verknüpften Jira-Projekts sich auch in einem GitHub-Kommentar in der App öffnet; was die eigene Grammatik des Hosts hinzufügt, öffnet seine Seite nach der Rückfrage zum Link. Eine Commit-ID öffnet sich im Log, wenn das Repository geklont ist und die ID mindestens 7 Zeichen hat, sonst auf dem Host. Nichts in einem Code-Span, einem Codeblock, einem Markdown-Link, einer E-Mail-Adresse oder einer URL ist ein Verweis, und `C#1`, `UTF-8`, `SHA-256` und `ISO-8601` sind keine.

| Host | Erkannt |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, Commit-IDs |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (ein Schwesterprojekt), `@user`, `~label` und `%milestone` als Chips, Commit-IDs |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, Commit-IDs |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, `{uuid}`-Erwähnungen, Commit-IDs |
| Gerrit | `Change-Id`-Werte (`I` und 40 Hex-Ziffern), Commit-IDs |
| Redmine | `#7`, `r123`-Revisionen, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, nur für ein Projekt oder Team einer verknüpften Quelle |
| Trello | Kartenadressen (`trello.com/c/...`) |
| Lokale Notizen | `[[note]]`, `[[note\|alias]]` und relative `.md`-Links zu einer anderen Notiz des Ordners |

`gitpoulpi hosting markdown --body T --id I` gibt aus, was die App aus einem Text macht, Verweise eingeschlossen.

### Markup-Umwandlung

Hosts, die kein Markdown sprechen, werden in beide Richtungen umgewandelt, sodass Sie überall Markdown lesen und schreiben. Was ein Umwandler nicht versteht, bleibt lesbarer Text.

- **Jira** (Cloud und Data Center) speichert Kommentare als Wiki-Markup über API v2. Was Sie lesen, wird in Markdown umgewandelt; was Sie schreiben, wird in Wiki-Markup umgewandelt, und die Fußzeile sagt *Gesendet als Jira wiki markup*.
- **Redmine** formatiert Text als Textile oder Markdown, was die API nicht sagt: Die Zeile **Textformat des Servers** des Profils (Einstellungen > Integrationen, `markup`, standardmäßig Textile) sagt es GitPoulpi. Textile wird zum Lesen in Markdown und zum Schreiben zurück in Textile umgewandelt.
- **Azure DevOps Server** liefert Kommentare von Work Items als HTML und nimmt sie als HTML an: Sie werden als daraus umgewandeltes Markdown gezeigt und als HTML geschrieben. Azure DevOps Services und Pull-Request-Threads nehmen Markdown.
- **Gerrit** ist reiner Text: Er wird nicht als Markdown dargestellt, und was Sie schreiben, wird wie getippt gesendet.
- Jeder Import entfernt zuerst Steuer- und Escape-Sequenzen, sodass ein Kommentar das Terminal nicht bewegen kann.

### Die gespeicherte Kopie, Ältere laden, ungelesen

- Jede Unterhaltung, die Sie öffnen, wird gespeichert (`<Statusordner>/conversations/<Integration>/<Projekt>/<Art>-<n>.json`, nur für den Benutzer, ohne Token, höchstens 2 MB je Datei, die 50 neuesten Dateien bleiben). Sie erscheint sofort, wenn das Element sich öffnet, und wird dahinter aktualisiert. Ist der Host nicht erreichbar, bleibt die gespeicherte Kopie unter *Offline: Kopie von vor 3 Min. wird gezeigt (Grund)* auf dem Bildschirm, und Senden ist ausgegraut.
- **Ungelesen:** Was Sie zuletzt gesehen haben, wird je Unterhaltung gemerkt (die letzten 200 bleiben). Neuere Einträge von anderen erhalten einen Akzentbalken und ein Chip `neu`, und `n` springt zum nächsten. Eine Unterhaltung, die Sie zum ersten Mal öffnen, hat nichts Neues.

### Was jeder Host nicht kann

Jedes `nein` unten ist eine echte Grenze: Das Bedienelement ist verborgen oder mit dem Grund ausgegraut, und die CLI antwortet mit einem Fehler, der den Host nennt, bevor etwas gesendet wird.

| Host | Elemente | Antwort in einem Thread | Eigene bearbeiten und löschen | Thread lösen | Ereignisse | Interne Notizen | Geschrieben als |
|---|---|---|---|---|---|---|---|
| GitHub | Issues, Pull Requests | Pull Requests | beides | Pull Requests | ja | nein | GitHub-Markdown |
| GitLab | Issues, Merge Requests | ja (Diskussionen) | beides | Merge Requests | ja | ja | GitLab-Markdown |
| Gitea | Issues, Pull Requests | Pull Requests | beides | Pull Requests | ja | nein | Markdown |
| Forgejo | Issues, Pull Requests | Pull Requests | beides | nein (der Host hat keinen Endpunkt) | ja | nein | Markdown |
| Azure DevOps | Work Items, Pull Requests | Pull Requests | beides | nein (Status gezeigt, nicht geändert) | nur Pull Requests | nein | Markdown (HTML bei Work Items auf dem Server) |
| Bitbucket (Cloud, Data Center) | nur Pull Requests | ja | beides | nein (gezeigt, nicht geändert) | ja | nein | Markdown |
| Gerrit | nur Änderungen | ja | nein (beides nicht bearbeit- oder löschbar) | ja | ja | nein | reiner Text |
| Jira | Issues | nein (flache Liste) | beides | nein | ja (Changelog) | nein (braucht eine Projektrolle) | Wiki-Markup |
| Linear | Issues | ja (Antworten auf einen Kommentar) | beides | nein | ja | nein | Markdown |
| YouTrack | Issues | nein | beides | nein | nein | nein | Markdown |
| Redmine | Issues | nein | nur bearbeiten (Redmine 5 und später), nie löschen | nein | ja (Journale) | ja (private Notizen) | Markdown oder Textile |
| Trello | Karten | nein | beides | nein | ja (Verschiebungen) | nein | Markdown |
| Lokale Notizen | Notizen | nein | die eigenen des lokalen Autors | nein | nein | nein | Markdown |

Außerdem: Kein Host lässt Sie aus der Unterhaltung heraus einen neuen Thread an einer Zeile beginnen (tun Sie das im Review), und Reaktionen werden nur bei GitHub gezeigt. Ein Text, der länger ist, als der Host annimmt, wird abgelehnt (GitHub 65.536 Zeichen, GitLab 1.000.000, Azure DevOps 150.000, Jira 32.767, Trello 16.384): Die CLI prüft zuerst, in der App wird die eigene Meldung des Hosts gezeigt, und Ihr Text bleibt erhalten. Eine lokale Notiz behält ihre Kommentare in einem letzten Abschnitt `## Comments` der Notizdatei selbst, je ein Eintrag `### Autor - Zeit`, sodass Obsidian und eine Shell sie ebenfalls sehen.

Auf der Kommandozeile sind dieselben Unterhaltungen `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit` und `comment-delete` sowie `hosting review conversation` (siehe [die CLI-Referenz](cli-reference.md)); für Agenten sind es MCP-Tools gleichen Namens (siehe [MCP und Agenten](mcp-and-agents.md)).
