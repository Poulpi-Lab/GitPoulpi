[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · [Français](../fr/submodules.md) · **Italiano** · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Sottomoduli

Un sottomodulo è un altro repository annidato nel tuo a un commit fisso. GitPoulpi li elenca nella barra laterale Refs, li aggiorna dalla UI, dalla CLI o dalla palette dei comandi, e può mantenerli allineati con il commit di cui fai il checkout.

## La sezione SOTTOMODULI

Quando un repository ha un `.gitmodules`, la barra laterale Refs ottiene una sezione **SOTTOMODULI** (sotto le altre; assente se non ce ne sono). Una riga per sottomodulo: un simbolo di stato, il suo percorso e un commit breve.

| Simbolo | Stato | Significato |
|---|---|---|
| `✓` | aggiornato | in checkout al commit registrato dal repository |
| `↑` | modificato | in checkout a un altro commit (è avanzato, o hai cambiato branch senza aggiornarlo) |
| `○` | non clonato | dichiarato in `.gitmodules` e registrato, ma la sua cartella è vuota: aggiornalo per clonarlo |
| `✗` | in conflitto | il repository ha conflitti di merge sul sottomodulo |

| Cosa fare | Mouse | Tastiera |
|---|---|---|
| Aprire il sottomodulo in una scheda propria | doppio clic sulla riga | selezionalo, `Enter` (palette: `Refs: Apri il sottomodulo in una scheda`) |
| Menu: **Aggiorna (clona se serve)**, **Inizializza**, **Apri in una nuova scheda** | clic destro sulla riga | selezionalo, tasto del menu contestuale (predefinito `.`) |
| Aggiornare il sottomodulo selezionato (prima lo clona, se serve) | menu | `Ctrl+P`: `Refs: Aggiorna il sottomodulo (selezionato)` |
| Registrarlo in `.git/config` senza clonarlo | menu | `Ctrl+P`: `Refs: Inizializza il sottomodulo (selezionato)` |
| Aggiornare tutti i sottomoduli | | `Ctrl+P`: `Git: Aggiorna tutti i sottomoduli` |
| Saltare alla sezione | clic sull'intestazione | `Ctrl+P`: `Navigazione: Vai a Refs > Sottomoduli` |

Un aggiornamento può fare il fetch dal remoto del sottomodulo, quindi segue **Impostazioni > SSH e credenziali > Credenziali HTTPS** come ogni altro trasferimento. Un sottomodulo non clonato non ha nulla da aprire come scheda; la barra di stato lo dice e rimanda ad Aggiorna.

## Mantenere aggiornati i sottomoduli

**Impostazioni > Sincronizzazione > Mantieni aggiornati i sottomoduli** (`general.keep_submodules_up_to_date`, disattivato per impostazione predefinita). Quando è attivo, un **checkout** o un **pull** riuscito è seguito da `git submodule update --init --recursive`. Viene eseguito solo se un sottomodulo non è allineato con quanto il repository ora registra, quindi un repository senza sottomoduli non rallenta. Se l'aggiornamento fallisce, il checkout o il pull è comunque avvenuto e l'errore lo dice.

## Dalla riga di comando

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Senza percorsi si intendono tutti i sottomoduli. Gli stessi tre sono strumenti MCP (`submodule_list` è una lettura; `init` e `update` sono modifiche e seguono `allow_mutations` di `[mcp]`).

## Note

- Viene elencato solo il livello superiore; i sottomoduli di un sottomodulo compaiono quando lo apri come scheda (oppure usa `--recursive` per aggiornarli).
- Le versioni recenti di git rifiutano di clonare un sottomodulo con *percorso locale* sul transport `file`, a meno che `protocol.file.allow` non lo permetta. Conta solo per i sottomoduli il cui URL è una cartella locale; impostalo nella tua configurazione git globale se li usi.
