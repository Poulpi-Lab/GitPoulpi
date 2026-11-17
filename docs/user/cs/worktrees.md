[English](../worktrees.md) · **Čeština** · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Worktree je druhá přepnutá kopie repozitáře: jiná složka na jiné větvi, se
stejnou historií. Sekce WORKTREES v panelu Refs je vypisuje; hlavní je označen
`(hlavní)`.

![Panel Refs repozitáře se dvěma připojenými worktrees: sekce WORKTREES uvádí main (hlavní), hotfix/axis-labels a feature/legend a pod ní je sekce SUBMODULES](../images/worktrees.png)

## Zobrazení sekce

V repozitáři, který ještě nemá žádný propojený worktree, je sekce skrytá. Chcete-li
ji přesto zobrazit (pak říká „Zatím žádný worktree: + ho vytvoří“), použijte příkaz
palety **Levý panel: Zobrazit sekci Worktrees**, zaškrtněte **WORKTREES** v seznamu
zobrazení a skrytí sekcí v levém panelu (pravé tlačítko na záhlaví sekce), nebo
spusťte kterýkoli z příkazů pro vytvoření níže. GitPoulpi si to pro daný repozitář
pamatuje (viz **Sekce levého panelu** v [Nastavení](settings.md)); další repozitář
bez worktrees stále začíná se sekcí skrytou.

## Vytvoření worktree

Tlačítko **+** v záhlaví WORKTREES (klávesnice: `+` v sekci; paleta:
**Refs: Vytvořit worktree...**) otevře nabídku:

- **Z aktuální větve**: větev je přepnutá v hlavní složce, proto dialog navrhne
  na ní novou větev (`worktree-<id>`).
- **Z vybraného commitu**: zašedlé, dokud v Logu není vybraný commit. Dialog
  navrhne novou větev `worktree-<id>`, která tam začíná; pro odpojený worktree
  pole větve vymažte.
- **Vybrat větev...**: dialog se otevře na seznamu **Větev worktree**, aby šlo
  otevřít už existující větev.

Paleta má stejné tři (**Refs: Vytvořit worktree z aktuální větve**,
**Git: Vytvořit worktree ve vybraném commitu...**, **Refs: Vytvořit worktree s
výběrem větve**). Starší položky fungují dál: **Vytvořit worktree z...** v nabídce
větve, tagu nebo commitu.

![Dialog Vytvořit worktree: pole Název, Složka, Ref a Větev worktree, řádek „nová větev z main“, políčko Otevřít nový worktree na kartě a tlačítka Zrušit a Vytvořit](../images/worktree-create.png)

Dialog všechno navrhne a nechá vás to změnit. **Název** i nová větev začínají
jako `worktree-<id>`, kde `<id>` je 8 náhodných šestnáctkových číslic (u obou
stejné), takže se dva worktrees nikdy nesrazí.

| Pole | Význam |
|---|---|
| **Název** | Z něj se sestaví název složky. **Podle větve** (`Alt+N`) ho nastaví na `worktree-<větev>`: lomítka se změní na `-` a slovo se neduplikuje (`worktree-foo` a `feature/worktree-x` si ponechají svůj název, `feature/login` dá `worktree-feature-login`). |
| **Složka** | Sourozenec repozitáře, `<repozitář>-<název>`. Sleduje Název, dokud do pole Složka nezačnete psát; **Procházet...** vybere umístění. |
| **Ref** | Kde začíná *nová* větev: větev nebo commit (seznam; hledá se psaním). Nepoužije se a říká to, dokud je vybraná existující větev. |
| **Větev worktree** | Jedno pole, dva významy, uvedené pod ním. Název, který není lokální větev, vytvoří novou větev z Ref (**nová větev z main**). Lokální větev, která není přepnutá jinde (vyberte ji v seznamu nebo napište její název), se tam přepne tak, jak je (**přepne tuto větev**). První řádek seznamu nebo prázdné pole přepne větev z Ref (nebo odpojí HEAD na commitu); odmítne se, když je tato větev už přepnutá. |
| **Znovu vygenerovat** | `Alt+R`. Vylosuje nové `<id>` a nahradí jím Název a větev, kromě toho, co jste napsali sami (Název nebo větev, kterou jste napsali, zůstane), a existující větve (ta se nikdy nenahradí). Název z **Podle větve** nebo od AI se počítá jako vygenerovaný. |
| **Pojmenovat pomocí AI** | Jen při zapnuté AI (viz níže). |
| **Otevřít nový worktree na kartě** | Ve výchozím stavu zapnuto. |

Neprázdná složka, neplatný název, název větve, který git odmítá, nebo větev,
kterou má jiný worktree, se oznámí na vlastním řádku dialogu a **Vytvořit**
zůstane zašedlé.

### Pojmenování pomocí AI

Při zapnuté [AI](ai.md) otevře **Pojmenovat pomocí AI** (`Alt+A`) uvnitř dialogu
pole: *Popište funkci, chybu nebo úkol*. **Pojmenovat** požádá model o krátký
název worktree a název větve podle názvů větví v samotném repozitáři (`feature/`,
`fix/`, malá písmena, kebab-case). Vyplní se jimi Název (jako `worktree-<název>`)
a pole větve, které lze dál upravit; název větve, který už existuje, dostane
příponu `-2`, aby odpověď nikdy nepřepnula něco potichu. Pole ukazuje stav
zaneprázdnění, **Zastavit** (nebo `Esc`) ho zruší a selhání se zobrazí uvnitř
dialogu. Odesílá se jen váš popis a názvy lokálních větví.

## Nabídka řádku

Klikněte pravým tlačítkem na worktree (nebo ho vyberte a stiskněte klávesu nabídky,
`Shift+F10` nebo `.`):

- **Otevřít na nové kartě**: jako dvojklik nebo `Enter`.
- **Otevřít v**: položky [Otevřít v](open-in.md), které mají zaškrtnuté **Může
  otevřít worktree** (Nastavení > Otevřít v; každá položka je ve výchozím stavu
  zaškrtnutá). Spouštějí se se složkou worktree jako `${cwd}`.
- **Merge <větev> do...**: vyberte větev, do které se má slučovat (ve výchozím stavu
  větev hlavního worktree). Běžný merge proběhne ve worktree, kde je tato větev
  přepnutá, s vašimi volbami merge; konflikt se tam řeší obvyklým postupem pro
  konflikty. Větev, která není nikde přepnutá, se odmítne s uvedením důvodu.
  U odpojeného worktree se nenabízí.
- **Odebrat worktree...**: nikdy ne hlavní.

![Nabídka po kliknutí pravým tlačítkem na řádek worktree: Otevřít na nové kartě, Otevřít v, Merge hotfix/axis-labels do... a Odebrat worktree...](../images/worktree-menu.png)

**Otevřít na nové kartě** (nebo dvojklik, nebo `Enter`) otevře složku worktree jako
samostatnou kartu s vlastním Logem, stavem a větví:

![Worktree hotfix/axis-labels otevřený jako vlastní karta vedle hlavní, s ukázaným svým commitem a změněným souborem](../images/worktree-tab.png)

Paleta má stejné akce na vybraném řádku (**Refs: Otevřít worktree v...**,
**Refs: Merge větve worktree do...**, **Refs: Odebrat worktree...**,
**Refs: Otevřít worktree na kartě**).

## Odebrání worktree

Čistý worktree dostane prostou potvrzovací otázku. Je-li jeho větev už sloučená
(do svého upstreamu, do větve hlavního worktree nebo do `main` / `master`),
políčko **Smazat také větev <b>** je ve výchozím stavu zaškrtnuté; nesloučená větev
oznámí, že zůstane. **Merge do... a odebrat** nejdřív provede merge a odebere až
po merge dokončeném bez konfliktů.

S necommitnutými změnami (sledovanými i nesledovanými) je dialog vypíše (počet,
posuvný seznam) a nabídne:

- **Commit a odebrat**: pole pro zprávu přímo v dialogu; všechny změny se přidají
  do indexu (`.gitignore` se respektuje) a commitnou na větvi worktree, potom se
  worktree odebere. Odpojený worktree pojmenuje větev pro commit ve stejném dialogu.
- **Stash a odebrat**: všechny změny, včetně nesledovaných souborů, se stashnou s
  upravitelnou zprávou (výchozí „WIP v <složka>“), potom se worktree odebere.
  Seznam stashů sdílejí všechny worktrees: stash je poté v STASHES hlavního
  repozitáře, připravený k Pop.
- **Otevřít a commitnout**: otevře worktree na kartě se zaměřenou zprávou commitu;
  nic se neodebere.
- **Přesto odebrat**: zahodí změny po druhém, výslovném potvrzení.

## Zpět

Odebrání worktree vrátit zpět nelze: složka je pryč. To, co se předtím uložilo,
zůstane: commit je na své větvi, stash v seznamu. Smazání větve, které přidává
**Smazat také větev**, je obyčejné zaznamenané v žurnálu: **Zpět** větev vrátí
(ne složku). Vytvoření worktree také není součástí historie.

## Ze skriptu nebo od agenta

`gitpoulpi worktree changes <path>` vypíše necommitnuté změny worktree.
`gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` ho odebere; nástroje MCP
MCP jsou `worktree_changes` a `worktree_remove`. `gitpoulpi ai
worktree-name --describe "<úkol>"` vypíše názvy worktree a větve, které AI
navrhuje (`{ name, worktree, branch }`), a nic nevytvoří; nástroj MCP je
`ai_worktree_name`. Viz [Referenční příručka CLI](cli-reference.md).
