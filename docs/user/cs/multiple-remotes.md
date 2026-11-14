[English](../multiple-remotes.md) · **Čeština** · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Několik remote pro jednu větev

Mnoho repozitářů žije na více než jednom hostiteli: hlavním (`origin`), zrcadle u jiného hostitele, záloze. GitPoulpi umožňuje větvi pushnout na všechny jedním kliknutím a volitelně stahovat z více než jednoho, aniž by se měnilo, co dělá obyčejný git. Nic z toho nevyžaduje účet a všechno se ukládá do `.git/config` samotného repozitáře, takže obyčejný git a další nástroje fungují dál.

## Cíle push a zdroje pull

- **Upstream** je vlastní pojem gitu (`branch.<name>.remote` a `merge`). Je to *primární* remote větve. GitPoulpi nikdy nemění, jak funguje.
- **Cíl push** je další remote, na který větev také pushuje. Tlačítko Push pošle větev na upstream a na každý cíl push.
- **Zdroj pull** je další remote, ze kterého pull také bere commity. Dokud ho nezapnete (zaškrtávací políčko **Pull také**), je vypnutý: slučovat dva remote, které jste nevybrali, nechce nikdo jako výchozí chování.

Cíl může na tom remote používat jiný název větve (`backup` dostane `feat-x`).

## Dialog Remote

Klikněte pravým tlačítkem na větev a zvolte **Remote větve <větev>...** (také v rozbalovací nabídce Push, v paletě jako `Git: Remote větve...` a kliknutím na odznak `⇉3` v řádku větve v sekci Refs). Vypíše každý remote repozitáře:

| Sloupec | Význam |
|---|---|
| Remote, Větev | Remote a název větve na něm. **Přejmenovat na remote...** změní tento název pro vybraný řádek. |
| Push | Zaškrtnutím uděláte z remote cíl push této větve. |
| Pull také | Zaškrtnutím způsobíte, že pull integruje i tento remote. Upstream se stahuje vždy. |
| Stav | Jak větev stojí vůči němu: synchronní, napřed, pozadu, nová, nesouvisí. |

Řádek označený „remote zmizel“ pojmenovává remote, který větev stále zmiňuje, ale repozitář už ho nemá; **Odebrat** na něj zapomene. Stejný dialog nastavuje upstream (seznam **Upstream**) a **Zrušit upstream** ho odstraní. **Použít** zapíše všechno jako jednu změnu, kterou lze vrátit zpět.

Repozitář může také dát každé nové větvi stejné další cíle: **Nastavení > Git a kódování** (stránka pro konkrétní repozitář), „Další cíle push pro nové větve“ (zaškrtávací políčko pro každý remote). Zkopírují se do větve, když poprvé dostane upstream; existující větve se nemění.

## Pushování

- **Tlačítko Push** u větve s dalšími cíli: pushne na všechny. Když každý remote prostě přijme nové commity, proběhne to hned a okno průběhu ukáže, jak se každý remote mění na zaškrtnutí. Pokud je potřeba rozhodnutí (remote má commity, které vy nemáte), otevře se místo toho dialog plánu.
- **Push ▾** (šipka vedle tlačítka; také samotné tlačítko, když je **Tlačítko Push** v Nastavení nastaveno na **zeptat se**):
  - `Push do všech (origin, gitlab, backup)`
  - `Push jen do <remote>`, jedna položka pro každý remote větve
  - `Push do vybraných remote...`: je tu vždy, pro každou větev, i pro tu s jediným remote
  - `Remote větve <větev>...`
- Kontextová nabídka větve a paleta (`Git: Push do všech cílů`, `Git: Push do remote...`, `Git: Push do vybraných remote...`) nabízejí stejné akce se stejnými názvy.

**Push do vybraných remote...** otevře seznam všech remote repozitáře, zaškrtnutých podle upstreamu a cílů větve. Platí jen pro tento push; zaškrtnutím **Zapamatovat pro tuto větev** uložíte volbu jako cíle push větve.

### Plán

Před skupinovým pushem se GitPoulpi podívá na každý remote (rychlý dotaz a stažení z toho remote jen tehdy, když se liší), takže problém na třetím remote se ukáže dřív, než začne první push. Dialog vypíše každý remote s tím, jak větev stojí, a co se s ním provede:

| Situace | Výchozí akce | Nabízí se také |
|---|---|---|
| Remote nemá z větve nic, nebo mu chybí některé commity | Push | Přeskočit |
| Remote je napřed (nemáte pro něj nic nového) | Fast-forward z něj | Přeskočit |
| Remote se liší, protože jste udělali rebase nebo amend toho, co jste pushli | Vynutit push (lease) | Merge, Rebase, Přeskočit |
| Remote má commity někoho jiného | Merge z něj (Rebase, když je **Tlačítko Pull** nastaveno na rebase) | Rebase, Vynutit push (lease), Přeskočit |
| Historie nemá s vaší nic společného | Přeskočit | Vynutit push (lease) |
| Hostitel větev odmítl (chráněná) | Přeskočit | nic: vynucení by bylo také odmítnuto |

Akce každého řádku je seznam, který můžete změnit (Vlevo/Vpravo jím prochází, Enter ho otevře, `s` řádek přeskočí). U merge se ukáže, zda by skončil konfliktem a v kterých souborech. **Vynucený push** se vždy potvrzuje, jmenuje remote a kolik commitů nahradí, vždy používá lease (nikdy holé vynucení) a nelze ho vrátit zpět: výsledek řekne, který commit byl nahrazen, a příkaz, který ho vrátí. **Přeskočit problémy** nastaví každému problémovému řádku Přeskočit. **Spustit** (Ctrl+Enter) zahájí běh.

Merge, fast-forwardy a rebase proběhnou první, v pořadí seznamu, takže vaše větev dosáhne konečného stavu jen jednou; potom se pushe odesílají po jednom remote, upstream první. Mezi remote neexistuje atomické „všechno, nebo nic“: každý skončí vlastním výsledkem a selhání jednoho (síť, hook) ostatní nezastaví. **Zopakovat neúspěšné** (nebo `Git: Zopakovat push na remote, která selhala`) je zkusí znovu. Zavření okna průběhu běh nikdy nezastaví; `Git: Zobrazit push do více remote` ho vrátí.

### Konflikty a Pokračovat

Pokud merge nebo rebase běhu skončí na konfliktech, dialog se zavře a převezme to obvyklý pruh merge a editor konfliktů. Proužek nad Logem zní `Pozastaveno (gitlab): push do více remote. Vyřešte a dokončete konflikty, potom Pokračovat.` a má tlačítka **Pokračovat** a **Zrušit skupinu**.

- Vyřešte konflikty a dokončete merge nebo rebase (v GitPoulpi, nebo obyčejným gitem).
- **Pokračovat** je zašedlé, dokud to není hotovo, a **nikdy se za vás nestiskne**: nic nepokračuje samo. Stiskněte ho (nebo spusťte `Git: Pokračovat v push do více remote`) a běh znovu pushuje na každý remote z vašeho nového hrotu. Po rebase se remote, který stále drží stará commity, vrátí do dialogu plánu (teď potřebuje vynucený push), místo aby byl potichu vynucen.
- **Zrušit skupinu** (po potvrzení; `Git: Zrušit push do více remote`) zahodí merge nebo rebase, který běh zahájil, a na běh zapomene. Už provedené pushe zůstanou.

Proužek přežije restart GitPoulpi a CLI vidí stejný uložený běh.

## Pullování

- Tlačítko **Pull** stahuje upstream úplně jako dřív, pokud větev nemá zdroje pull: pak stáhne upstream a každý zdroj, po řadě, každý v režimu nastaveném v **Tlačítko Pull** v Nastavení (merge, jen fast-forward, nebo rebase). Zdroj, který nejde fast-forwardovat, se sloučí (nebo rebasuje) jako upstream; při režimu jen fast-forward se u zdroje, který potřebuje víc, nejdřív zeptá.
- **Pull ▾** přidává `Pull ze všech zdrojů (origin + gitlab)`, když zdroje existují, a `Pull jen z <remote>` pro každý remote větve: jednorázový pull větve z toho remote, bez ohledu na zaškrtávací políčka.
- Paleta má `Git: Pull ze všech zdrojů` a `Git: Pull z jednoho remote...`.
- Pull, který se zastaví na konfliktech, zanechá stejný proužek, označený jako skupinový pull.

## Nastavení

Všechny čtyři jsou v **Nastavení > Git a kódování**.

| Nastavení | Výchozí | Význam |
|---|---|---|
| **Tlačítko Push** | přímo | přímo pushne hned; zeptat se nejdřív otevře nabídku Push. |
| **Tlačítko Pull** | merge | Režim tlačítka Pull (merge, jen fast-forward, nebo rebase) a zdrojů pull. |
| **Zkontrolovat všechny remote před push** | zapnuto | Podívat se na každý remote před skupinovým pushem (CLI se tím řídí; `--no-preflight` to pro jedno volání vypne). |
| **Další cíle push pro nové větve** (pro každý repozitář) | žádné | Remote, na které nová větev také pushuje. |

## Z příkazového řádku

```
gitpoulpi branch upstream add gitlab --pull      # cíl push, který je i zdrojem pull
gitpoulpi branch upstream list                    # k čemu je větev nastavená
gitpoulpi push --all-targets                      # upstream plus každý cíl
gitpoulpi push --to gitlab --to backup:feat-x     # přesně tyto, jen tentokrát
gitpoulpi pull --all-sources                      # upstream plus každý zdroj pull
gitpoulpi remote-group plan                       # co by skupinový push udělal, nic se nemění
gitpoulpi remote-group continue                   # po vyřešení a dokončení zastavení
```

Když remote potřebuje rozhodnutí, příkaz odpoví plánem (kód 3) a nic nezmění; odpovězte `--resolve gitlab=merge`. Každý přepínač viz [Referenční příručka CLI](cli-reference.md). Server MCP má stejné nástroje.

## S obyčejným gitem

Cíle jsou obyčejné konfigurační klíče, takže je skript nebo alias může používat bez GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # jedna hodnota na řádek: remote nebo remote:branch

# push aktuální větve na její upstream a na každý cíl
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Hodnota je `<remote>` (stejný název větve) nebo `<remote>:<branch>`, což řeší řádek `d=${t#*:}`. Obyčejný alias se předem na remote nedívá, nenabízí merge a po konfliktu nemůže pokračovat: k tomu je dialog.

## Dobré vědět

- Dva remote ukazující na stejnou adresu se poznají („gitlab a mirror mají stejnou URL pro push“); druhý se ve výchozím stavu přeskočí.
- Hostitel s chráněnou větví vynucený push odmítne; řádek to řekne a nabídne jen Přeskočit.
