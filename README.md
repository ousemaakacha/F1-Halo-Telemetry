# F1 Halo Telemetry

Overlay Windows nativo per **EA SPORTS F1 25 — 2026 Season Pack**. Rust, Win32,
Direct3D 11, Direct2D, DirectWrite e DirectComposition. Due finestre indipendenti:
Halo e pneumatici. Nessun browser engine, injection, hook del gioco o lettura
della memoria di F1 25.

## Download rapido

- **Windows x64:** [scarica l'app](downloads/f1-halo-telemetry.exe), poi avviala.
- Per provare gli HUD senza F1 25, seleziona **Demo (simulated data)** e premi
  **Apply and save**.
- Per usare telemetria reale, imposta UDP in F1 25 come descritto sotto.
- Per compilare dai sorgenti, installa Rust e segui [Compilazione Windows](#compilazione-windows).

Il sorgente include app, esempi, test, script e documentazione. Impostazioni
salvate in `%LOCALAPPDATA%\F1HaloTelemetry\settings.json`.

## Avvio

Aprire `dist/F1-Halo-Telemetry/f1-halo-telemetry.exe`, oppure l'eseguibile in
`target/release` dopo la compilazione. Il pannello impostazioni compare all'avvio.
Se si è estratto il pacchetto portabile, l'eseguibile si trova accanto a questo README.
L'applicazione ascolta UDP su `127.0.0.1:20777` e mostra uno stato di attesa finché
non riceve dati validi. La casella **Demo (simulated data)**, seguita da **Apply and save**, permette di provare entrambi gli HUD senza il gioco.

La Demo è esplicita: ogni HUD Halo riporta **DEMO · SIMULATED DATA**. La Demo non
si attiva automaticamente quando manca la connessione e non viene salvata come
sorgente predefinita. Per la guida usare il gioco in **Borderless Fullscreen**.
Il funzionamento sopra fullscreen esclusivo dipende dal sistema e non è garantito.

## Configurazione di F1 25

Nel menu principale aprire **Opzioni / Impostazioni / Telemetria**. La traduzione
esatta delle voci può dipendere dalla lingua e dalla versione del gioco.

| Impostazione del gioco | Valore sullo stesso PC |
| --- | --- |
| UDP Telemetry | On |
| UDP Broadcast Mode | Off |
| UDP IP Address | `127.0.0.1` |
| UDP Port | `20777` |
| UDP Send Rate | `60 Hz` |
| UDP Format | **`2026`**, non `2025` |

Disattivare Demo nell'app e applicare. Entrare in una sessione di guida. Se si
usava già F1 25 prima del Season Pack, il formato UDP può essere rimasto su 2025:
va selezionato manualmente 2026. La specifica EA chiarisce che i dati della
propria vettura sono disponibili anche con **Your Telemetry = Restricted**.
L'app seleziona sempre `m_playerCarIndex`, senza sostituirlo con la vettura osservata.

Per ricevere da un altro dispositivo, impostare nell'app l'indirizzo locale
dell'interfaccia di rete o `0.0.0.0`, e nel gioco l'indirizzo del PC ricevente.
Le eventuali autorizzazioni di rete di Windows restano sotto il controllo dell'utente.

Fonte: [pagina ufficiale EA e revisioni della specifica](https://forums.ea.com/blog/f1-games-game-info-hub-en/ea-sports%E2%84%A2-f1%C2%AE25-2026-season-pack-udp-specification/12187347).

## Uso degli HUD

**Ctrl + Shift + H** alterna Race Mode ed Edit Mode. La combinazione è modificabile
nel pannello. Se è già occupata, il pannello segnala il problema e mantiene la
combinazione precedente quando possibile.

**Ctrl + Shift + O** mostra o nasconde entrambi gli HUD, anche durante il gioco.
La scorciatoia è modificabile nel pannello e disponibile anche nel menu rapido.
Il comando conserva posizione e preferenze di visibilità dei singoli HUD;
alla riapertura dell'app la visibilità globale riparte attiva. Quando gli HUD
sono nascosti la ricezione continua, mentre il rendering si arresta.

**Hide Halo: rear / sides**, attivo per impostazione iniziale, usa lo **stick
destro del controller PS5 verso il basso, a destra o a sinistra**, senza premere
R3: Halo scompare mentre guardi indietro o ai lati e torna quando riporti lo
stick al centro. Il mini HUD pneumatici resta visibile secondo le sue preferenze.
Funziona con tutti i modelli Halo durante la guida, con F1 25
in primo piano; in Demo ed Edit resta disattivato. La casella nel pannello,
seguita da **Apply and save**, permette di disabilitarlo. Non cambia le
preferenze di visibilità manuale: un Halo già nascosto rimane nascosto.

Supporta DualSense USB/Bluetooth e controller esposti tramite XInput da Steam
Input. I comandi del gioco per guardare indietro, a destra e a sinistra devono
essere assegnati alle rispettive direzioni dello stick destro e tornare alla
visuale anteriore al rilascio.

| Modalità / controllo | Comportamento |
| --- | --- |
| Race Mode | Finestre trasparenti, topmost, click-through e senza attivazione |
| Edit Mode | Cornice e angoli visibili; trascinamento libero; resize proporzionale |
| Rotella sopra un HUD in Edit | Modifica scala |
| Shift + rotella in Edit | Modifica opacità |
| Doppio clic sul mini HUD in Edit | Alterna Surface, Inner e Both |
| Esc con un HUD attivo in Edit | Torna a Race Mode |
| Clic destro in Edit | Menu rapido con visibilità, modalità gomme, Demo e impostazioni |
| Icona nell'area di notifica | Doppio clic: pannello; clic destro: menu rapido |

Chiudere il pannello nasconde soltanto le impostazioni. **Exit** termina
l'applicazione. Visibilità, posizione, scala, opacità, colori, soglie e hotkey
sono persistenti. Le modifiche effettuate con mouse sono salvate al rilascio;
i campi del pannello richiedono **Apply and save**. **Reset layout** agisce
subito sul layout. **Default settings** carica i valori nel pannello e richiede
poi Applica. Telemetria e ricezione continuano durante la modifica.

Il pannello **Software settings** usa una palette blu scuro e ciano, con le
categorie **Halo & layout**, **Tyres**, **Colours**, **Connection**, **Shortcuts**
e **Diagnostics** nella barra laterale. I comandi rapidi, la sorgente Demo, il
frame rate e lo stato della sessione restano a destra nelle finestre ampie e
passano sotto le impostazioni nei formati più stretti. Cambiare categoria conserva
le modifiche non ancora applicate; **Apply and save** salva tutti i campi.
I colori mostrano un campione accanto al valore esadecimale.

**Halo model** permette di scegliere **Wide**, **Classic** oppure **modern**.
Modern segue il nuovo riferimento: fascia antracite continua con gradiente,
estremità arrotondate, bordo arancione, arco superiore sottile e quadrante più
grande. Fuel, Straight Mode, Active Aero e marce restano integrati nella fascia.
Premere **Apply and save**: il cambio avviene
senza riavviare l'app e conserva centro del quadrante, posizione, scala e opacità.
La scelta viene salvata e ripristinata all'avvio. I profili precedenti usano Wide.
Tutti i modelli mostrano le stesse telemetrie aggiornate: km/h, percentuale e
modalità ERS, Fuel, pedali, Straight Mode con distanza, Aero, marce e RPM.

**Halo profile** salva posizioni e dimensioni diverse con nomi scelti da te,
per esempio **Ferrari** e **Red Bull**:

1. Posiziona e ridimensiona Halo in Edit Mode, oppure compila X, Y e Scala nel pannello.
2. Scrivi il nome in **Halo profile** e premi **Save profile**. Il comando applica
   posizione e scala e le salva subito.
3. Per cambiare vettura, scegli un nome dall'elenco e premi **Load profile**.

Salvare un nome già presente aggiorna quel profilo; maiuscole/minuscole e spazi
esterni non creano duplicati. I nomi possono contenere fino a 64 caratteri.
Spostare Halo conserva il profilo salvato: premi di nuovo **Save profile** per
aggiornarlo. La scelta è manuale e l'ultimo nome usato resta selezionato al riavvio.
I profili condividono il file impostazioni e conservano soltanto X, Y e Scala.
Colori, modello Halo, opacità, visibilità e mini HUD pneumatici restano indipendenti.
I comandi di ripristino del layout o delle impostazioni non cancellano i profili.

La finestra delle impostazioni si ridimensiona trascinando lati e angoli e
supporta ingrandimento, ripristino e affiancamento Windows. All'avvio si adatta
all'area disponibile del monitor su cui si trova il puntatore. La disposizione
passa da tre colonne a pannelli sovrapposti; nelle finestre strette le categorie
sono in alto e i riquadri si dispongono su una colonna. Quando lo spazio non
basta, lo scorrimento permette di raggiungere tutti i controlli.
Finestre e impostazioni usano DPI per monitor. Gli HUD vengono riportati nell'area
visibile quando cambiano monitor o risoluzione.

## Dati e disponibilità

Il modello Ampio segue il bordo interno della protezione della vettura con un'arcata più
ampia e aperta, ricavata dalle guide del riferimento. Le ali sono circa il 23%
più larghe; il canvas misura 1000 × 300 unità. Quadrante e caratteri conservano
le proprie dimensioni, mentre i contenuti laterali seguono la nuova curva.
Classico riprende le ali compatte e inclinate e il profilo superiore bianco.
I due modelli condividono il canvas, i colori configurabili e tutti i dati guida.
Il quadrante centrale mostra velocità in km/h e percentuale ERS. La percentuale
diventa azzurra quando Overtake è disponibile in zona, anche prima di attivare il
boost, e torna bianca quando la disponibilità termina. Non compare una targhetta
Overtake separata. Pedali a segmenti, Straight Mode e Active Aero sull'ala sinistra;
marce e scala RPM seguono la curva dell'ala destra. Il piccolo riquadro **FUEL**
a sinistra dei pedali mostra il margine carburante live del MFD rispetto alla
distanza di gara rimanente, con segno e un decimale: **+ 2.3** oppure **- 0.8**.
Il valore segue consumo e risparmio, senza sottrarre nuovamente i giri gara.
Quando i dati non sono disponibili o recenti compare **--**.
I 15 segmenti luminosi del quadrante conservano i bit dei LED di cambiata.
Il colore iniziale degli archi è arancione ed è modificabile nel pannello.

Velocità in km/h, marcia immediata R/N/1–8, pedali effettivi, RPM e massimo RPM
quando disponibile. I 15 LED usano i bit `m_revLightsBitValue`, bit 0 a sinistra.
Active Aero e Overtake derivano dal pacchetto Car Telemetry 2; il campo DRS
legacy non viene usato per rappresentarli. Con vetture pre-2026 compare N/A.
La fascia sinistra mostra **SM OFF** insieme alla distanza in metri comunicata
dal gioco, aggiornata mentre ci si avvicina alla zona. Quando la disponibilità
viene confermata compare **SM READY**; dopo l'attivazione compare **SM ON**
con fondo verde. Una distanza pari a zero non implica disponibilità.
Senza dati recenti compare **SM --**; sulle vetture pre-2026 **SM N/A**.
Sotto la batteria compare la modalità ERS corrente: **NONE**, **MEDIUM**,
**HOTLAP** oppure **BOOST**. La stessa scritta è rimossa da sotto Active Aero.
Pannello impostazioni, menu, messaggi e diagnostica dell'app sono in inglese.
Le scritte decorative HALO/26 e LIVE sono rimosse. Demo, attesa e anomalie
conservano un'indicazione compatta, visibile soltanto quando necessaria.

Gli indici delle gomme sono **RL=0, RR=1, FL=2, FR=3**. Il disegno visualizza
FL/FR davanti e RL/RR dietro. Usura 0% significa gomma nuova. Ogni valore conserva
il timestamp del proprio pacchetto: nessuna interpolazione di temperature,
usura o marcia. Le soglie termiche iniziali 85–105 °C sono preferenze visive
modificabili, non una dichiarazione sulla finestra ottimale di ogni mescola.

La percentuale nell'intestazione **TYRES** è la media aritmetica dell'usura delle
quattro gomme. Con dati assenti o scaduti compare **--%**.

La specifica trasmette energia ERS in **Joule**, ma non definisce la capacità
totale della batteria. L'app legge automaticamente la **capacità nominale dai
file vettura del gioco installato** (`EnergyRecoverySystem/Common/EnergyStoreCapacity`
negli archivi ERP), senza impostazioni numeriche manuali. I file installati su
questo PC dichiarano 4 000 000 J per le vetture F1 2025 e 2026 controllate.
Il pannello mostra capacità, stagione e provenienza; la diagnostica le esporta.
Il limite di raccolta per giro non viene mai usato come capacità.

Il gioco viene individuato tramite percorso del processo (soli metadati), Steam
e cartelle standard EA. Per installazioni non standard è disponibile la variabile
ambiente `F1_25_DIR`. La lettura avviene sul thread ricezione, una volta per
sessione e al cambio sorgente; non modifica né estrae file del gioco. Sono
supportate le categorie F1 2025 e 2026 con capacità nominale uniforme tra vetture.
In assenza di file compatibili o con capacità diverse tra vetture modificate,
il display mostra l'energia in MJ al posto della percentuale, mantenendo la
modalità ERS sotto il valore. Non serve inserire una capacità nell'app.
La capacità nominale non misura eventuali riduzioni dinamiche dovute a usura,
R&D o modifiche runtime. La Demo usa una capacità simulata separata di 4 000 000 J.
Dettagli e comando di verifica: [lettura batteria](docs/BATTERY.md).

| Stato | Significato |
| --- | --- |
| IN ATTESA UDP | Nessuna telemetria locale valida; gioco chiuso e UDP disattivato non sono distinguibili dalla sola rete |
| LIVE | Telemetria valida e recente |
| DEMO · SIMULATED DATA | Generatore interno esplicitamente attivo |
| DATI OBSOLETI | Nessun Car Telemetry recente; valori guida sostituiti con `--` |
| VETTURA NON ATTIVA | Lap Data indica Invalid o Inactive |
| PAUSA | Pausa dichiarata nei pacchetti Session / Car Status |
| ERRORE PORTA UDP | Bind o socket fallito; codice Windows nella diagnostica esportata; retry automatico |
| FORMATO NON SUPPORTATO | Formato diverso da 2026 o versione diversa da 1 per i pacchetti letti |

Telemetria, stato e Aero scadono dopo 500 ms. Usura scade dopo 2 s. Il PDF indica
10 Hz per Car Damage; un esempio nella FAQ dello stesso PDF riporta ancora 2 Hz.
L'app misura gli Hz ricevuti e non presume che l'usura arrivi a 60 Hz. Dopo
flashback, cambio sessione o riavvio vengono invalidati i valori incompatibili.
La ripresa dell'invio non richiede il riavvio dell'app.

## Impostazioni su disco

File predefinito: `%LOCALAPPDATA%\F1HaloTelemetry\settings.json`.
Scrittura su file temporaneo, flush e sostituzione atomica con `MoveFileExW`.
I valori vengono validati prima del salvataggio. Un JSON corrotto o una versione
schema sconosciuta producono un errore esplicito senza sovrascrivere il file.
È possibile avviare con `--config` per scegliere un altro percorso.

## Compilazione Windows

Su questa macchina è stata verificata la toolchain Rust GNU x64 con il linker
LLD distribuito da Rust. Non serve un framework UI o una DLL grafica aggiuntiva.
Le API native provengono dai binding Microsoft `windows` 0.62.2 generati dai
metadati Windows SDK. L'eseguibile usa le DLL DirectX di Windows.

Da PowerShell 7, nella cartella del progetto:

```powershell
.\scripts\Build.ps1 -Bootstrap -Test -Examples -Bench
.\scripts\Package.ps1 -SkipBuild
```

`-Bootstrap` installa Rust in ambito utente quando manca, senza modificare il
PATH permanente. Per GNU scarica LLVM-MinGW 20261104 dalla release ufficiale,
verifica SHA256 ed estrae soltanto dlltool e le sue DLL nella cartella `.cache`.
I componenti servono alla compilazione e non vanno distribuiti con l'app.
`Cargo.lock` viene rispettato mediante `--locked`.

È supportata anche la compilazione standard con Rust MSVC, Visual Studio C++
Build Tools e Windows SDK installati, ma tale toolchain non è stata verificata
in questa sessione:

```powershell
cargo build --locked --release
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Il binario finale è `target/release/f1-halo-telemetry.exe`. Il pacchetto portabile
è `dist/F1-Halo-Telemetry.zip`. Nessuna installazione dell'app è necessaria.
Il progetto completo è anche in `dist/F1-Halo-Telemetry-source.zip`.

## Test e misure

```powershell
.\scripts\Measure.ps1 -Seconds 30
.\scripts\Measure.ps1 -Seconds 120 -SkipBuild
.\scripts\Measure.ps1 -Seconds 120 -Case udp-60 -SkipBuild
```

Gli ultimi due comandi permettono un controllo più lungo della memoria. Gli scenari
sequenziali misurano Demo a 60 Hz, UDP sintetico a 60 e 120 Hz, inattività, ricreazione
forzata delle risorse e interruzione del flusso. Le porte sono separate dal gioco.
I risultati reali vengono scritti in `artifacts/*.json`; i campioni di memoria,
private bytes e handle ogni 10 s in `artifacts/*-memory.csv`. Lo script fallisce su
errori funzionali; riporta gli FPS misurati senza dichiarare automaticamente
raggiunti gli obiettivi prestazionali.

Per un test manuale del passaggio dei clic tra processi:

```powershell
.\target\release\f1-halo-telemetry.exe --demo
.\target\release\examples\input_probe.exe
```

In Race Mode fare clic sui pixel visibili di entrambi gli HUD: il titolo della
finestra probe deve aumentare il contatore e il focus deve restare alla probe.

Il generatore UDP esterno è uno strumento di test esplicito:

```powershell
.\target\release\examples\udp_demo.exe 127.0.0.1:20777 60 30
```

Parametri: destinazione, frequenza, durata opzionale in secondi. Usarlo con una
porta di test e il gioco chiuso. I pacchetti sintetici sono indistinguibili sul
filo da quelli del gioco; soltanto la Demo interna può etichettarli nell'HUD.

Per acquisire la diagnostica di una sessione reale:

```powershell
.\target\release\f1-halo-telemetry.exe --no-settings --quit-after 120 --diagnostics real-session.json
```

La latenza misurata va dal ritorno di `recv_from` all'invio `Present`: non include
tempo nel kernel prima della lettura, scansione del monitor o latenza ottica.
La GPU usa query timestamp asincrone con DONOTFLUSH ogni 30 frame; non vengono
prodotti campioni se la query non è pronta o risulta disjoint. È tempo GPU dei
comandi dell'overlay, non percentuale di utilizzo né tempo del compositore DWM.
I percentili usano gli ultimi 4096 campioni, il conteggio totale resta separato.
CPU nel report è la media del processo normalizzata sui core logici; RAM è
working set. Il frame rate segue i dati: un limite di 120 FPS con UDP a 60 Hz
non inventa altri 60 frame di telemetria.

Risultati e limiti della verifica: [docs/VERIFICATION.md](docs/VERIFICATION.md).
Dettagli del protocollo: [docs/PROTOCOL.md](docs/PROTOCOL.md).
Scelte tecniche: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Opzioni

`--demo`, `--edit`, `--no-settings`, `--config <file>`, `--fps 60|120|adaptive`,
`--quit-after <secondi>`, `--diagnostics <file>`, `--help`.
`--recreate-device-after <secondi>` è riservato ai test: ricrea device, swap chain,
brush, font e geometrie, senza modificare il driver del sistema.
