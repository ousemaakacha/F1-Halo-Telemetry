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

Il logo monoposto e telemetria è incorporato nell'eseguibile e compare nel
pannello, nelle icone delle finestre e nell'area di notifica. L'immagine scelta
è conservata in `assets/app-logo.png`; `assets/app.ico` contiene le risoluzioni
Windows da 16 a 256 px.

Le scritte e i numeri degli HUD usano **F1 Turbo / Formula1 Display-Wide**, come
nel riferimento tipografico. Il font è incorporato nell'eseguibile: non serve
installarlo in Windows. Dimensioni e larghezze si adattano ai riquadri; i nomi
troppo lunghi mantengono i puntini di sospensione.
[Anteprima f1 tv](artifacts/f1-turbo-f1-tv-preview.png) e
[pneumatici](artifacts/f1-turbo-tyres-preview.png).
Provenienza e diritti del carattere: [font HUD](docs/FONTS.md).

## Avvio

Aprire `dist/F1-Halo-Telemetry/f1-halo-telemetry.exe`, oppure l'eseguibile in
`target/release` dopo la compilazione. Il pannello impostazioni compare all'avvio.
Se si è estratto il pacchetto portabile, l'eseguibile si trova accanto a questo README.
È consentita una sola istanza per sessione Windows: un nuovo avvio richiama il
pannello esistente senza duplicare gli overlay.
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
La visibilità durante il gioco è **automatica**: entrambi gli HUD compaiono
quando la propria vettura è in pista in prove libere, qualifiche (anche Sprint),
gara. In modalità Time Trial resta visibile solo Halo: il mini HUD pneumatici
si nasconde automaticamente e torna in modalità Race secondo la visibilità
salvata. La regola segue anche la scelta manuale del pannello TT/Race.
Gli HUD si nascondono nei menu, in pausa, nel garage, in modalità
spettatore, a fine sessione e quando i dati smettono di arrivare. Non serve
premere la scorciatoia a ogni cambio menu. Il rilevamento non dipende dalla
velocità: gli HUD restano visibili anche con la vettura ferma in pista.
Demo ed Edit Mode permettono di vedere e posizionare gli HUD anche senza gioco.
Il comando manuale resta un interruttore generale: se lo disattivi, gli HUD
restano nascosti anche quando torni in pista, finché lo riattivi.

Il comando conserva posizione e preferenze di visibilità dei singoli HUD;
alla riapertura dell'app la visibilità globale riparte attiva. Quando gli HUD
sono nascosti la ricezione continua, mentre il rendering si arresta.

Anche **due pressioni entro 400 ms**, con rilascio fra le pressioni,
mostrano o nascondono entrambi gli HUD: **View / Back** su Xbox,
**Share** su DualShock 4 e **clic fisico sulla metà sinistra del touchpad** su
DualSense PS5. Il dito deve trovarsi a sinistra quando inizi ciascun clic;
sfiorare il touchpad o spostare il dito tenendo premuto non ripete il comando.
Il DualSense deve essere accessibile tramite HID: l'app legge le coordinate nei
report USB/Bluetooth completi e ne richiede l'attivazione al collegamento.
I soli report XInput virtuali non distinguono le due metà del touchpad.
Il comando funziona anche con HUD già nascosti e con entrambi i toggle
**Hide Halo** disattivati. Una pressione mantenuta non lo ripete; riconnessione e cambio
controller azzerano la sequenza. Come la scorciatoia da tastiera, funziona anche
quando il gioco non è in primo piano. I tasti restano disponibili al gioco.

Due toggle indipendenti usano lo **stick destro**, senza premere R3:

- **Hide Halo in rear view**: nasconde Halo quando lo stick va verso il basso
  per guardare indietro.
- **Hide Halo in side views**: nasconde Halo quando lo stick va a destra o
  a sinistra per guardare ai lati.

Halo torna quando riporti lo stick al centro. Disattiva il toggle della direzione
affetta da drift: quell'asse viene ignorato anche se l'altro toggle resta attivo.
Il mini HUD pneumatici segue la modalità TT/Race e la propria visibilità salvata.
Funziona con tutti i modelli Halo durante la guida, con F1 25
in primo piano; in Demo ed Edit il filtro resta disattivato. Le due caselle
nel pannello si applicano e si salvano subito, senza **Apply and save**.
Disattivarle entrambe annulla anche un precedente occultamento da stick.
Entrambe sono attive nelle nuove configurazioni; caricando una vecchia
configurazione, entrambe ereditano il precedente valore comune. Non cambia le
preferenze di visibilità manuale: un Halo già nascosto rimane nascosto.

Supporta DualSense e DualShock 4 USB/Bluetooth, controller Xbox tramite XInput
e controller esposti tramite XInput da Steam Input. I comandi del gioco per
guardare indietro, a destra e a sinistra devono
essere assegnati alle rispettive direzioni dello stick destro e tornare alla
visuale anteriore al rilascio.

| Modalità / controllo | Comportamento |
| --- | --- |
| Race Mode | Finestre trasparenti, topmost, click-through e senza attivazione |
| Edit Mode | Cornice e angoli visibili; trascinamento libero; resize proporzionale |
| Rotella sopra un HUD in Edit | Modifica scala |
| Shift + rotella in Edit | Modifica opacità del fondo |
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

In **Halo & layout**, ciascun overlay ha il campo **Background %**: puoi impostare
Halo e Tyres separatamente dallo **0% al 100%**. Il controllo modifica soltanto
il fondo: scritte, numeri, immagine auto e indicatori mantengono la loro opacità.
Aumenta il valore per ridurre la trasparenza del fondo: **100%** lo rende
completamente opaco, anche nel modello **f1 tv**; **0%** lo elimina.
Premi **Apply and save**. Le percentuali vengono ripristinate al
riavvio; i file impostazioni precedenti restano compatibili.

**Halo model** permette di scegliere **modern**, **Shift** oppure **f1 tv**.
Il modello **f1 tv** riprende gli indicatori del riferimento McLaren con le curvature di
Shift, ali speculari, arco e quadrante centrati: km/h e mph, modalità ERS attuale,
OVERTAKE e batteria con percentuale, fasce blu RCH/DPL con residuo per giro, pedali segmentati,
classifica curva a sinistra e ACTIVE AERO, marce e RPM su righe separate a destra.
La scritta **RPM** mostra i giri attuali per intero, per esempio **RPM 10000**.
Il contatore **LAPS giro/totale** segue l'arco a sinistra del quadrante;
senza dati validi mostra **LAPS --/--**.
Ogni lineetta ACTIVE AERO ha un bordo verde quando è attivo, rosso quando è inattivo.
La riga sinistra superiore mostra il pilota immediatamente davanti, quella
inferiore il pilota dietro: posizione, sigla di tre lettere (per esempio
**NOR**, **LEC**, **VER**) e distacco in secondi, per esempio **1.234s**,
con una colonna dedicata ai tempi. Tra sigla e distacco, un simbolo
a doppio anello mostra la gomma montata da ciascun avversario.
Solo l'ala sinistra cambia automaticamente in **Time Trial**: delta live rispetto
al miglior giro valido registrato nella sessione, cronometro del giro, settori
S1/S2/S3 e stato **VALID / INVALID**. Il riferimento resta fisso durante il giro;
un giro valido migliore viene usato dal giro successivo. Senza riferimento
compare **NO VALID REF**; se il miglior tempo è noto ma manca la sua traccia
completa, compare **NO LAP TRACE** e il delta resta `--`.
Il rewind conserva i giri validi già registrati e scarta la traccia interrotta
del giro corrente.
Nel menu rapido (clic destro su Halo in Edit Mode o sull'icona nell'area di
notifica), **F1 TV left panel** offre **Automatic (session)**, **Race** e
**Time Trial**. La scelta manuale vale per la sessione corrente; nuova sessione
o cambio sorgente ripristinano Automatic. Il pannello piloti gara, quadrante,
pedali, pannello destro e mini HUD pneumatici mantengono il proprio layout.
Selezionare il modello e premere
**Apply and save**, poi regolare posizione e scala in Edit Mode sulla propria
visuale. [Anteprima e dettagli f1 tv](docs/REPLICA.md).

Shift usa il fondo e il bordo di
Modern con una disposizione dedicata e simmetrica. Al centro la marcia grande
è l'informazione principale, con velocità in **km/h e mph** e anello contagiri;
il valore numerico RPM non viene mostrato. A sinistra **AHEAD** e **BEHIND** mostrano in
secondi il distacco dalle auto immediatamente davanti e dietro in classifica.
La freccia verso il basso indica un distacco in riduzione, quella verso l'alto
un distacco in aumento, con lo stesso significato per entrambe le auto.
L'andamento richiede almeno un secondo di campioni validi e continui della
stessa auto: variazioni inferiori a 50 ms restano neutre. Un trattino indica
andamento non determinato o stabile; `--` indica distacco non disponibile o
scaduto. A destra, Shift mostra livello e modalità ERS, Straight Mode con la
distanza disponibile e margine carburante in **LAPS**. La percentuale ERS usa
la capacità già nota all'app; senza capacità viene mostrata l'energia in MJ.
ERS e modalità sono mostrati soltanto nel riquadro destro di Shift. Gli mph
sono ricavati dallo stesso campione km/h dividendo per 1.609344 e arrotondando
all'intero; entrambe le velocità mostrano `--` se la telemetria manca o scade.
Dati assenti o scaduti dopo 500 ms diventano `--`. Le evidenziazioni sono
statiche e locali: rosso per ERS esaurito e margine carburante negativo,
colore accento per BOOST e SM READY, verde per SM ON, secondo i colori
configurati nell'app.
I due riquadri sinistri e i tre destri hanno uguali ingombri esterni e righe
allineate alla curva; i pedali sono speculari, con barre più alte ed etichette
**BRK/THR** abbassate all'interno della fascia. In Shift l'anello usa i giri
effettivi rispetto al limite motore ricevuto: una tacca gialla **10k** indica
10.000 giri per la partenza da fermo. Marcia e anello lampeggiano insieme in
rosso a 5 Hz a partire dalla soglia fissa di **12.500 RPM**.
L'avviso vale per le marce 1–8 e si spegne quando i giri scendono o i dati
scadono. Senza limite RPM valido l'anello segue i LED del gioco e la tacca
gialla non viene mostrata.
La marcia e gli indicatori centrali conservano il centro del layout salvato.

Shift e f1 tv mostrano una fascia di avvisi temporanei sotto il quadrante:
bandiere FIA verde/blu/gialla, bandiera rossa/scacchi dagli eventi ufficiali,
Safety Car/VSC e aumenti di penalità o avvertimenti del gioco. Un avviso dura
al massimo cinque secondi, senza rinnovarsi a ogni pacchetto; stato terminato,
dati scaduti, pausa o cambio sessione lo rimuovono prima. I messaggi di sicurezza
hanno precedenza. Penalità e warning richiedono un confronto con un campione
recente: i totali già presenti all'avvio non vengono notificati come nuovi.
Non si deducono danni, cause delle penalità o rientri SC non forniti dai dati
decodificati. Dettagli e limiti: [contratto UDP](docs/PROTOCOL.md).
Modern segue il nuovo riferimento: fascia antracite continua con gradiente,
estremità arrotondate, bordo arancione, arco superiore sottile e quadrante più
grande. Fuel, Straight Mode, distacco live da chi segue e marce restano integrati nella fascia.
Premere **Apply and save**: il cambio avviene
senza riavviare l'app e conserva centro del quadrante, posizione, scala e opacità.
La scelta viene salvata e ripristinata all'avvio. I file impostazioni con Wide,
Classic o Carbon vengono aperti usando modern. Modern e Shift mostrano le
telemetrie aggiornate: km/h, percentuale e modalità ERS, Fuel, pedali, Straight
Mode con distanza, distacco live da chi segue, marce e RPM.

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

L'HUD usa un canvas da 1000 × 300 unità, colori configurabili e tutti i dati guida.
Il quadrante centrale mantiene la velocità grande in km/h. Al posto della
scritta KM/H, sopra la velocità mostra la marcia corrente **R/N/1–8**; sotto,
la percentuale ERS. La percentuale
diventa azzurra quando Overtake è disponibile in zona, anche prima di attivare il
boost, e torna bianca quando la disponibilità termina. Non compare una targhetta
Overtake separata. Pedali a segmenti, Straight Mode e distacco live sull'ala sinistra;
marce e scala RPM seguono la curva dell'ala destra. Il piccolo riquadro **FUEL**
a sinistra dei pedali mostra il margine carburante live del MFD rispetto alla
distanza di gara rimanente, con segno e un decimale: **+ 2.3** oppure **- 0.8**.
Il valore segue consumo e risparmio, senza sottrarre nuovamente i giri gara.
Quando i dati non sono disponibili o recenti compare **--**.
I 15 segmenti luminosi del quadrante conservano i bit dei LED di cambiata,
tranne in Shift, che usa la scala RPM descritta sopra.
Il colore iniziale degli archi è arancione ed è modificabile nel pannello.

Velocità in km/h, marcia immediata R/N/1–8, pedali effettivi, RPM e massimo RPM
quando disponibile. I 15 LED usano i bit `m_revLightsBitValue`, bit 0 a sinistra.
Active Aero e Overtake derivano dal pacchetto Car Telemetry 2; il campo DRS
legacy non viene usato per rappresentarli. Con vetture pre-2026 compare N/A.
La fascia sinistra mostra **SM OFF** insieme alla distanza in metri comunicata
dal gioco, aggiornata mentre ci si avvicina alla zona. Quando la disponibilità
viene confermata compare **SM READY**; dopo l'attivazione compare **SM ON**
in verde. Una distanza pari a zero non implica disponibilità.
Senza dati recenti compare **SM --**; sulle vetture pre-2026 **SM N/A**.
Sotto la batteria compare la modalità ERS corrente: **NONE**, **MEDIUM**,
**HOTLAP** oppure **BOOST**.

Al posto del delta giro, la riga sinistra mostra il **distacco live dalla vettura
subito dietro in classifica**, in secondi e millesimi: **+ 1.234** significa
1,234 secondi di vantaggio. Il testo è bianco e si aggiorna a ogni pacchetto
Lap Data, anche durante un giro invalido. Il valore usa l'intervallo comunicato
dal gioco dalla vettura nella posizione successiva, indipendentemente dal suo
indice UDP. Con nessuna vettura attiva dietro, ultima posizione, pausa o dati
più vecchi di 500 ms compare **--**. Il distacco si azzera al cambio sorgente,
sessione o dopo un flashback. La Demo mostra un distacco simulato variabile.
Straight Mode e distacco seguono il contorno sinistro in tutti i modelli.
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
Il disegno dell'auto ha fondo trasparente: condivide la stessa tonalità del
riquadro pneumatici, anche quando cambi l'opacità dell'overlay.

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
verifica SHA256 ed estrae dlltool, llvm-rc, llvm-cvtres e le loro DLL nella cartella `.cache`.
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

L'app registra automaticamente avvio, uscita, cambi di visibilità, perdita del
controller ed errori grafici in `%LOCALAPPDATA%\F1HaloTelemetry\session-events.jsonl`.
Ogni riga contiene ora Unix in millisecondi, tempo dall'avvio, età dei pacchetti
UDP, flag pausa/sessione, stato del controller e contatori degli errori.
Questo permette di distinguere un'interruzione della telemetria da un comando
del controller o da un ripristino grafico anche dopo una scomparsa breve.
Il file ruota a 256 KiB, conservando anche `session-events.previous.jsonl`;
si scrive soltanto in corrispondenza degli eventi. Con `--config`, i registri
si trovano accanto al file impostazioni scelto. **Export diagnostics** riporta
anche il percorso del registro e gli eventuali errori di scrittura.

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
