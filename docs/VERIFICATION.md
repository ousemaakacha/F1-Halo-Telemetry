# Local verification

## f1 tv: gomme degli avversari con simbolo B — 23 novembre 2026

Aggiunto il simbolo a doppio anello tra nome e divisore del distacco, per
l'auto immediatamente davanti e quella dietro. Colori e iniziali: S rosso,
M giallo, H bianco, W blu e I verde. Nomi spostati a sinistra e abbreviati
con puntini quando necessario, mantenendo dimensione e proporzioni.

Car Status conserva le mescole visuali dei 24 slot; lo snapshot le legge
per identità dell'avversario con scadenza a 500 ms. Verificati offset visuale
distinto dal composto assoluto, indice locale 0/23, sorpasso, cambio gomme,
pausa, dato scaduto, cambio sessione e ricezione UDP reale su loopback.
Il modello usa un simbolo neutro `?` quando il dato non è disponibile.
La modalità Demo e il suo codificatore UDP includono le mescole.

Superati **126 test**, build Release e rustfmt sui file modificati.
Verificati **36 scenari Direct2D**: tutte le mescole, dati assenti, nomi lunghi
e scale 1000×300 e 2000×600. Il controllo dei pixel conferma almeno 10 unità
tra nome e gomma e uno scarto del centro delle cinque iniziali inferiore
a 0,25 unità logiche nella resa 2000×600. Modern e Shift non presentano
pixel modificati. [Anteprima](../artifacts/f1-tv-tyres-b-preview.png) e
[nome lungo](../artifacts/f1-tv-tyres-b-long-name.png).
Nessuna nuova prova in gara.

## f1 tv: bordi Active Aero più spessi — 22 novembre 2026

Aumentato lo spessore dei contorni rossi e verdi da 1,2 a 2 unità logiche.
Il bordo segue ciascuna lineetta, comprese le estremità arrotondate.
Conservati centro, lunghezza, inclinazione e spessore della parte chiara interna.

Superati build Release, rustfmt sul file modificato e 30 scenari Direct2D.
Verificati entrambi i colori, i contorni aderenti e i margini alle scale
1000×300 e 2000×600. Modern e Shift non presentano pixel modificati.
[Aero attivo](../artifacts/f1-tv-thicker-aero-preview.png) e
[Aero inattivo](../artifacts/f1-tv-thicker-aero-off.png).
Nessuna nuova prova in gara.

## f1 tv: lineette Active Aero più lunghe — 22 novembre 2026

Allungate le due lineette del 20% in entrambi gli stati, distribuendo
l'estensione sulle due estremità. Centro, inclinazione, spessore e contorno
verde/rosso rimangono invariati.

Superati build Release, rustfmt sul file modificato e 30 scenari Direct2D.
Verificati i due contorni aderenti, i margini e le scale 1000×300 e 2000×600.
Modern e Shift non presentano pixel modificati.
[Aero attivo](../artifacts/f1-tv-longer-aero-preview.png) e
[Aero inattivo](../artifacts/f1-tv-longer-aero-off.png).
Nessuna nuova prova in gara.

## f1 tv: riquadri laterali rialzati e allineati — 22 novembre 2026

Traslati entrambi i riquadri di 20 unità verso l'alto, pari a 40 pixel
nell'anteprima 2000×600. Lo stesso spostamento comprende cornici, divisori,
classifica, distacchi, ACTIVE AERO, marce e RPM. Dimensioni, curvatura,
simmetria e proporzioni dei testi rimangono invariate.

Superati la build Release, i quattro test del modello e il controllo rustfmt sul file
modificato. Il test dell'arco ora considera lo spostamento effettivo
e verifica uno spazio libero superiore a 8 unità. Verificati 30 scenari
Direct2D, con controlli sui margini a 1000×300 e 2000×600. Modern e Shift
non presentano pixel modificati. [Anteprima](../artifacts/f1-tv-raised-preview.png)
e [Aero inattivo](../artifacts/f1-tv-raised-aero-off.png).
Nessuna nuova prova in gara.

## f1 tv: riquadri laterali più bassi — 22 novembre 2026

Ridotta l'altezza radiale dei due riquadri da 100 a 80 unità (-20%), mantenendo
centro, curvatura e simmetria. Ridotti i margini e riallineate le righe di
classifica, marce, RPM e ACTIVE AERO. Font, dimensioni e proporzioni dei testi
sono invariati; i bordi restano separati dalle scritte.

Superati la build Release, i quattro test del modello, il controllo rustfmt
sul file modificato e 30 scenari grafici nativi. Verificate le scale 1000×300 e 2000×600: il margine
minimo misurato dei testi dal bordo è superiore a 8 unità logiche.
Modern e Shift non presentano pixel modificati rispetto al codice precedente
alla riduzione. [Anteprima](../artifacts/f1-tv-compact-preview.png) e
[Aero inattivo](../artifacts/f1-tv-compact-aero-off.png).
Il controllo rustfmt globale segnala formattazione preesistente in
`src/hud/halo.rs` e `src/settings/mod.rs`, lasciata invariata.
Nessuna nuova prova in gara.

## f1 tv: contorni delle lineette Aero e valori MJ rimossi — 21 novembre 2026

Rimosso il riquadro comune di ACTIVE AERO. Ogni lineetta conserva il centro
chiaro e riceve un contorno aderente, comprese le estremità arrotondate:
verde in Straight Mode attivo, rosso in Corner Mode inattivo. Con dati
assenti, scaduti o non applicabili il contorno resta spento.
Rimosse le didascalie numeriche in MJ sotto RCH e DPL; il riempimento continua
a indicare il budget residuo per giro.

Build Release e rustfmt superati. Verificati 30 scenari nativi Direct2D,
con controllo dei pixel per entrambi i bordi, assenza di contorni separati
dalle lineette e assenza delle didascalie MJ. Controllate anche le scale
1000×300 e 2000×600. Gli altri cinque modelli non presentano pixel modificati.
[Aero attivo](../artifacts/f1-tv-aero-outlines-preview.png) e
[Aero inattivo](../artifacts/f1-tv-aero-outlines-off.png).
Nessuna nuova prova in gara.

## f1 tv: sigle RCH e DPL centrate — 21 novembre 2026

Sostituite le etichette estese con **RCH** e **DPL**, disegnate come parole
complete su una riga, con spaziatura naturale. Le due sigle sono centrate
nella sezione mediana dei contenitori e condividono lo stesso allineamento
verticale. Conservati riempimenti e valori dei budget in MJ.

Build Release e rustfmt superati. Anteprima Direct2D verificata a 1000×300
e 2000×600; il controllo dei pixel conferma tre lettere compatte e uno
scarto del centro inferiore a un'unità nella scala 1000×300.
[Anteprima](../artifacts/f1-tv-rch-dpl-preview.png).
Nessuna nuova prova in gara o modifica ai calcoli energetici.

## f1 tv: percentuale, budget per giro e bordo Aero — 21 novembre 2026

La batteria mostra la percentuale al posto del fulmine e il suo ingombro,
terminale compreso, è centrato sul quadrante. Il centro mostra la modalità
ERS corrente NONE/MEDIUM/HOTLAP/BOOST. RECHARGE e DEPLOY hanno riempimento
proporzionale al residuo del giro e relativo valore in MJ. Un bordo verde
circonda le due linee Aero in Straight Mode, rosso in Corner Mode; dati
assenti o non applicabili mantengono il bordo spento.

RECHARGE usa limite UDP meno energia MGU-K già raccolta; DEPLOY usa il limite
nominale dei file del gioco meno energia già usata. I contatori vengono
letti da Car Status agli offset 42 e 54, separati dalla capacità batteria.
L'ispezione locale ha verificato 44 risorse e zero errori: 4 MJ di capacità
batteria e 13 MJ di limite DEPLOY per il 2026.
[Report dei parametri](../artifacts/f1-tv-game-energy.json).

Passati **125 test**, rustfmt, Clippy `--all-targets -D warnings`, build Release
ed esempi. Coperti offset UDP, non finiti, limiti indipendenti, reset dei
contatori, superamento del limite, budget zero, sorgenti assenti, selezione
della stagione, limiti incoerenti e conservazione della capacità valida.

Verificati 30 casi Direct2D, con batteria 0/55/100%, tutte le modalità ERS,
budget pieni/esauriti/superati e stati Aero. Le asserzioni sui pixel controllano
percentuale, riempimenti e colori del bordo. Gli altri cinque modelli
coincidono con il riferimento nei dieci confronti eseguiti.
[Aero attivo](../artifacts/f1-tv-energy-preview.png),
[Aero inattivo](../artifacts/f1-tv-energy-aero-off.png).

Smoke Release: Demo per sei secondi, **348 frame**, **60,00 Hz**, zero errori
di presentazione; UDP sintetico per sette secondi, **409 frame**, **60,01 Hz**,
zero errori. Il percorso UDP ha letto capacità e limite DEPLOY dai file
installati, senza usare i valori Demo.
[Demo](../artifacts/f1-tv-energy-demo-smoke.json),
[UDP sintetico](../artifacts/f1-tv-energy-udp-smoke.json).
Nessuna nuova prova durante una sessione di guida reale. Il limite DEPLOY
letto dai file è nominale.

## f1 tv: nomi e distacchi davanti/dietro — 21 novembre 2026

Il pannello sinistro mostra l'avversario davanti sopra e quello dietro sotto,
con posizione, nome e intervallo dalla propria vettura. I tempi occupano la
colonna precedente TURN e seguono le stesse due righe dei nomi. Entrambi i
nomi e i distacchi mantengono contrasto pieno; i nomi lunghi conservano la
dimensione dei caratteri. La Demo include entrambi gli avversari.

I due abbinamenti identità/intervallo provengono dallo stesso Lap Data e
scadono insieme. Il nuovo test verifica nomi diversi dal giocatore, cambio
avversario, assenza del pilota davanti, distacco zero, scadenza, pausa e
invalidazione. Passati **120 test**, rustfmt, Clippy `--all-targets -D warnings`,
build Release ed esempi.

Verificati 21 casi Direct2D, compresi primo/ultimo posto, Lap Data e nomi
assenti, intervalli molto lunghi e zero. Due casi verificati anche a 1000×300.
[Anteprima con 1.234s davanti e 0.832s dietro](../artifacts/f1-tv-gaps-preview.png).
La prova si basa su telemetria sintetica; nessuna nuova verifica in gara.

## f1 tv: nome e RPM attuali — 21 novembre 2026

Il selettore mostra **f1 tv**. La scelta già salvata rimane compatibile.
La didascalia sotto la barra RPM mostra il valore corrente completo, per
esempio **RPM 10000**; dati assenti o scaduti producono **RPM --**.

Passati **119 test**, rustfmt, Clippy `--all-targets -D warnings` e build
Release con esempi. Verificati 15 casi Direct2D, compresi 10.000, 12.000,
65.535 RPM, status assente e telemetria scaduta; testi e margini conservati.
[Anteprima aggiornata](../artifacts/f1-tv-preview.png).
Nessuna nuova prova durante una sessione di guida reale.

## Replica: curvature simmetriche e testi distanziati — 21 novembre 2026

Ricostruite le ali con le primitive curve di Modern/Shift, raggio comune,
estremi speculari e spessore radiale costante. Arco bianco e quadrante ora
condividono il centro della finestra. I due pannelli hanno la stessa cornice;
la classifica mantiene due righe equidistanti, mentre a destra GEARS, marce,
barra RPM e didascalia hanno righe separate. I nomi lunghi usano `...` senza
ridurre la dimensione dei caratteri.

Passati **119 test**, rustfmt, Clippy `--all-targets -D warnings`, build Release
e build degli esempi. I controlli geometrici verificano centro, riflessione,
spessore, spazio tra righe e separazione dell'arco dai pannelli.

Verificati 13 casi Direct2D a 2000×600 e due casi anche a 1000×300. L'analisi
dei pixel dei testi conferma un margine radiale minimo di 12,2 unità nella
scala 1000×300; nomi normali e lunghi mantengono l'altezza dei caratteri.
La geometria dell'arco è speculare; il centro ottico del raster resta entro
un pixel dal centro alle due risoluzioni, con differenze di antialias ai bordi.
I dieci confronti degli altri cinque modelli coincidono, salvo due pixel
antialias di Wide nel caso senza dati.
[Anteprima aggiornata](../artifacts/replica-mclaren-symmetric-preview.png).

La prova Demo della nuova build è stata impedita dall'istanza precedente
già aperta: il secondo avvio richiama il suo pannello ed esce. Nessuna prova
della nuova disposizione durante una sessione di guida reale.

## Replica McLaren — 21 novembre 2026

Aggiunto **Replica (McLaren)** come sesto modello selezionabile. Geometrie
tracciate sul secondo riferimento: quadrante km/h/mph, BOOST/OVERTAKE,
batteria, fasce RECHARGE/DEPLOY, pedali segmentati, classifica, TURN e pannello
ACTIVE AERO/marce/RPM. Nomi da Participants e posizione locale da Lap Data;
la seconda riga segue l'identità dell'inseguitore in classifica. TURN rimane
`--`: numero e direzione di curva non sono contenuti nel protocollo.

Passati **118 test**, rustfmt, Clippy `--all-targets -D warnings`, build Release
e build degli esempi, con `--locked`. I test coprono offset e stride Participants,
nomi UTF-8, testo invalido e senza NUL, indici non contigui, cambio inseguitore,
scadenza, cambio sessione, geometria e selezione di tutti i modelli nel pannello.

Verificati 13 render Direct2D: riferimento, frenata, boost, Corner Mode,
dati assenti/scaduti, status/aero/capacità mancanti, fondo trasparente,
retromarcia, nome lungo e colore personalizzato. Dieci confronti con il codice
precedente verificano i cinque modelli esistenti con e senza dati: pixel
identici salvo due pixel antialias di Wide nella prova con dati.
[Anteprima nativa con dati simulati](../artifacts/replica-mclaren-preview.png).

Smoke test dell'app Release per sei secondi in Demo: **348 frame**, **60,00 Hz**,
**zero errori di presentazione**.
[Report](../artifacts/replica-mclaren-smoke.json).
L'allineamento con l'halo della vettura va regolato sulla visuale del gioco;
nessuna prova in una nuova sessione di guida reale.

## Filtri visuale Halo applicati subito — 20 novembre 2026

I due interruttori **Hide Halo** ora applicano e salvano subito la propria
preferenza, senza richiedere **Apply and save** o applicare altri campi in
modifica. Disattivarli entrambi esclude il filtro anche dal calcolo finale dei
layout, compreso un eventuale stato nascosto precedente. In caso di errore
vengono ripristinati controlli e layout. I registri includono ora i due flag
effettivi e gli eventi di applicazione delle impostazioni.

Passati **115 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release
con `--locked`. Il nuovo test di regressione falliva prima della correzione e
verifica visibilità globale e preferenze indipendenti di entrambi gli HUD.
La prova sul pannello Win32 ha disattivato e salvato entrambi i filtri senza
Applica, conservando un campo X non valido ancora in modifica. Con UDP
sintetico a 60 Hz su porta separata, entrambi gli HUD risultano visibili in
LIVE e non ci sono errori di presentazione.
Resta da verificare il comportamento durante una nuova gara con il controller.

## Shift: pedali estesi, avviso cambiata e tacca 10k — 20 novembre 2026

Barre BRK/THR estese da 36,8 a 53,7 unità (+46%) ed etichette abbassate di
13 unità. Conservati larghezza, centri, inclinazione e riempimento progressivo.
Il test geometrico include i nuovi ingombri ruotati e conferma che barre ed
etichette restano interamente nella fascia.

In Shift i 15 segmenti seguono `rpm / max_rpm`; la tacca gialla **10k** indica
esattamente 10.000 RPM sulla stessa scala, anche cambiando il limite motore.
Senza limite valido rimangono i LED del gioco e non viene disegnata una tacca
con posizione presunta. Dalla soglia fissa di 12.500 RPM,
marcia e anello lampeggiano insieme in rosso a 5 Hz; la tacca resta gialla.
Nessun avviso in R/N, a giri inferiori alla soglia o con telemetria scaduta.

Passati **114 test**, rustfmt, Clippy `--all-targets -D warnings` e build
Release con `--locked`. Test su soglia 12.499/12.500 RPM indipendente dai LED,
superamento limite, tutte le marce avanti, R/N, fase 0/99/100/199/200/299/300/399/400 ms,
dati assenti e limiti 10.000/12.000/15.000/20.000 RPM.
Il primo giro ha rilevato un timeout di due secondi nel test UDP di recupero;
la ripetizione completa dei controlli è passata senza modificare quel test.

Nella prova nel gioco F1 ha comunicato 12.278 RPM, LED cambiata al 100% e
un limite di 13.099 RPM: la precedente soglia di 13.500 RPM era irraggiungibile.
La soglia fissa è stata portata a 12.500 RPM su richiesta dell'utente.
Diagnostics ed esportazione JSON includono RPM motore, limite e percentuale LED.

Confrontati 85 render nativi, 17 condizioni per ciascuno dei cinque modelli,
più due fasi del lampeggio Shift. Le asserzioni sui pixel verificano marcia
e anello sincronizzati, tacca gialla persistente, scadenza e dati mancanti.
Fuori dal centro e dai pedali di Shift i pixel coincidono con il riferimento.
Classic, Modern e Carbon coincidono integralmente; Wide presenta soltanto
due pixel con scarto antialias 4/255 nei casi senza dati, scaduti o senza limite.

Anteprime native con dati simulati: [normale](../artifacts/shift-revs-live.png),
[fase rossa a 12.500 RPM](../artifacts/shift-revs-threshold.png),
[10.000 RPM da fermo](../artifacts/shift-revs-launch.png),
[dati assenti](../artifacts/shift-revs-missing.png).
Smoke test dell'app Release in Demo: sei secondi, 348 frame aggiornati,
60,00 Hz e zero errori di presentazione DirectX.
[Report](../artifacts/shift-revs-smoke.json).
Lampeggio a 12.500 RPM verificato nei test e nei render nativi; resta da
verificare durante una nuova guida nel gioco.

## Toggle separati per nascondere Halo dietro e ai lati — 19 novembre 2026

Il pannello espone **Hide Halo in rear view** e **Hide Halo in side views**.
Le preferenze vengono salvate separatamente; disattivare una direzione esclude
il relativo asse dal filtro, anche in presenza di drift o di un precedente
stato nascosto. Il cambio dei toggle con Applica azzera l'isteresi precedente.
Conservati soglie 55%/35%, vincoli di guida/focus, visibilità manuale, mini HUD
pneumatici e scorciatoia del controller per entrambi gli HUD.

Passati **112 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release
con `--locked`. Verificate
tutte le quattro combinazioni dei toggle, destra/sinistra/indietro, ritorno al
centro, input assente e contesto disabilitato. I test del pannello Win32
controllano lettura indipendente, navigazione, ingombri e assenza di sovrapposizioni
in otto formati e DPI 96/144/192. Verificati salvataggio e ricaricamento di
entrambi i valori e migrazione dei vecchi file: il precedente toggle comune
viene mantenuto per entrambe le direzioni, comprese le disattivazioni.
La diagnostica distingue `rear_view_enabled` e `side_view_enabled`.

Nessuna prova con controller fisico o con F1 in esecuzione.

## Centro Shift con km/h e mph, senza ERS duplicato — 19 novembre 2026

Rimosse soltanto da Shift percentuale/MJ ERS e modalità nel quadrante centrale.
Il centro mostra marcia grande, km/h e mph su due righe; ERS e modalità restano
nel riquadro destro esistente. Conservati LED di cambiata, barre, ali, avvisi,
profili e impostazioni. Gli mph derivano dal valore km/h ricevuto usando
1 miglio = 1.609344 km e si visualizzano arrotondati all'intero. Le due letture
condividono validità e scadenza di Car Telemetry; senza dati mostrano `--`.

Passati i **17 test Halo**, rustfmt, Clippy `--all-targets -D warnings` e build
Release con `--locked`. Confrontate **40 immagini native**: otto condizioni
per cinque modelli. La cattura delle chiamate DirectWrite conferma che il
centro Shift disegna esattamente tre testi, anche senza dati o dopo la scadenza:
marcia, km/h e mph, senza ERS o modalità. Verificati 285→177, 100→62, 10→6,
0→0 e 65535→40722. Cambiare soltanto energia, modalità e capacità ERS non
modifica alcun pixel del quadrante centrale.

Fuori dal centro di Shift i pixel coincidono con la versione precedente.
Classic, Modern e Carbon coincidono integralmente; Wide conserva solo i
consueti scarti antialias di due pixel 4/255 nei casi assenti/scaduti.
Anteprime: [normale](../artifacts/shift-kph-mph-preview.png),
[dati assenti](../artifacts/shift-kph-mph-missing.png),
[valore massimo supportato](../artifacts/shift-kph-mph-max-speed.png).

File di questa modifica: `src/hud/halo.rs`, `README.md`,
`docs/ARCHITECTURE.md`, questo documento e le tre anteprime
`artifacts/shift-kph-mph-*.png`. Nessuna nuova prova nel gioco.

## Barre THR/BRK di Shift più larghe — 18 novembre 2026

Larghezza di entrambe le barre aumentata da 16 a 36 unità (+125%). Altezza
36.8, centri, inclinazioni, etichette, scala dei valori e telemetria restano
quelli esistenti. La larghezza è condivisa tra fondo, riempimento e test
geometrico per verificare l'ingombro effettivo dentro la fascia.

Passati i tre test Shift, rustfmt, Clippy `--all-targets -D warnings` e build
Release con `--locked`. Confrontate 20 immagini native: dati parziali, 100%,
zero e dati assenti per cinque modelli. In Shift i pixel cambiano soltanto
nelle due barre; gli altri modelli restano invariati salvo i piccoli scarti
antialias già ammessi (due pixel 4/255 nel caso Wide senza dati).
[Anteprima con BRK 60% e THR 85% simulati](../artifacts/shift-pedals-wide-preview.png).

File di questa modifica: `src/hud/halo/shift.rs`, questo documento e
`artifacts/shift-pedals-wide-preview.png`. Nessuna nuova prova nel gioco.

## Shift con design Modern e centro senza RPM numerici — 18 novembre 2026

Shift ora richiama direttamente il fondo, il bordo e l'arco superiore di
Modern, mantenendo una disposizione propria. Il file `src/hud/halo/modern.rs`
non è stato modificato: SHA256 prima/dopo
`D538B6487AEE2B97D882672C3174DD752D4287208D93D9CD2E8E7A5380D79286`.

Misure: raggio 710 come Modern; ali da ±12° a ±28°; riquadri da -70 a -4 di
profondità, con tre righe comuni (-61.5, -36.5, -11.5). I due riquadri per i
distacchi e i tre per ERS/Straight/Fuel hanno ingombri esterni speculari e
margini uniformi. Ogni testo viene misurato e ridotto solo se necessario per
rientrare nella propria curva. Pedali di larghezza 16 e altezza 36.8, su
centri 301 e 539 e inclinazioni -9°/+9°, con etichette dentro la fascia.

La marcia è l'informazione centrale più grande; la velocità e l'ERS sono
secondari. Rimossa la scritta numerica RPM, mantenuti i 15 LED di cambiata.
Tolte le copie Fuel e Straight Mode dell'ala sinistra di Shift, perché gli
stessi dati restano nei riquadri destri. In assenza di capacità, la didascalia
ERS indica MJ e il valore è mostrato senza stimare la percentuale. La fascia
avvisi e il testo di connessione conservano posizione e comportamento.
Nessuna modifica a parser, telemetria, selettore, preferenze o modello predefinito.

Passati **110 test** (56 unit, 8 avvisi, 5 game data, 38 protocollo, 3 UDP),
rustfmt, Clippy `--all-targets -D warnings` e build Release con `--locked`.
I test geometrici verificano simmetria, contenimento di tutte le righe nei
riquadri Direct2D e contenimento dei pedali/etichette nel corpo Modern.

Confrontate **65 anteprime native** (13 condizioni per cinque modelli),
inclusi dati mancanti, SM READY/ON, distanza massima, MJ, valori lunghi,
fondo trasparente, avviso SC, N/R, velocità massima e colori personalizzati.
Modern, Classic e Carbon coincidono integralmente con la copia precedente.
Wide presenta solo uno/due pixel con scarto massimo 4/255 in due condizioni,
entro la tolleranza antialias già usata nelle verifiche precedenti.
Avvisi e connessione di Shift coincidono nei pixel. Un confronto separato
dimostra che passare da 11600 a 1234 RPM, lasciando fissi i bit dei LED, non
modifica alcun pixel di Shift.

Anteprime: [normale](../artifacts/shift-modern-preview.png),
[valori lunghi](../artifacts/shift-modern-long-values.png),
[dati assenti](../artifacts/shift-modern-missing.png),
[avviso SC](../artifacts/shift-modern-alert.png),
[MJ](../artifacts/shift-modern-mj.png).
Nessuna nuova sessione reale nel gioco.

File di questa modifica: `src/hud/halo.rs`, `src/hud/halo/shift.rs`
(layout e test geometrici), `README.md`, `docs/ARCHITECTURE.md`, questo
documento e le cinque anteprime `artifacts/shift-modern-*.png`.

## Fascia avvisi Shift — 18 novembre 2026

Completata in coda alla zona destra. Solo Shift aggiunge un banner statico
nella fascia libera sotto il quadrante, sopra il precedente stato di
connessione. Centro e ali non cambiano. I messaggi usano esclusivamente dati
EA verificati: FIA flags locali, SC/VSC, incrementi dei contatori di penalità
e warning, eventi `RDFL` e `CHQF`. Dettagli e limiti nel [contratto UDP](PROTOCOL.md).

Durata massima cinque secondi dal cambio, senza rinnovo sui pacchetti uguali.
Stato cessato, sorgente invalida/scaduta, pausa, garage, spettatore e cambio
sessione/timeline li invalidano prima. I conteggi iniziali non producono falsi
nuovi eventi. Distinti eventi nello stesso frame non fanno perdere una nuova
rossa/scacchi; i duplicati e gli eventi vecchi non rinnovano la notifica.
La priorità privilegia rossa, SC/VSC e gialla rispetto ai messaggi secondari.

Passati **109 test** (55 unit, 8 avvisi, 5 game data, 38 protocollo, 3 UDP),
rustfmt, Clippy `--all-targets -D warnings` e build Release con `--locked`.
I nuovi test verificano offset indipendenti per gli slot 0 e 23, sconosciuti,
durata senza rinnovo, cessazione, SC/VSC e formazione, contatori con base fresca,
priorità, doppioni nello stesso frame, eventi ritardati, pausa, invalidazione,
Session End, cambio UID, flashback e riavvio. Un test distinto impedisce di
considerare attuale un campione vecchio solo perché appena ricevuto.

Confrontate **90 anteprime Direct2D** (18 condizioni per cinque modelli),
comprese tutte le etichette, valori massimi dei contatori, scadenze, sorgenti
obsolete, SC terminato, formazione e fondo trasparente. Classic, Modern e
Carbon coincidono integralmente con la copia precedente; Wide presenta un
solo pixel con scarto 4/255 in un caso. Le zone protette di Shift presentano
al massimo un pixel con scarto 5/255, entro la tolleranza antialias già usata.
Ispezionate [Safety Car](../artifacts/shift-alerts-safety-car.png),
[rossa](../artifacts/shift-alerts-red-flag.png),
[penalità](../artifacts/shift-alerts-penalty.png) e
[warning](../artifacts/shift-alerts-warning.png).
Nessuna nuova sessione reale nel gioco.

File di questa modifica:

- `src/telemetry/alerts.rs` (nuovo), `state.rs`, `protocol.rs`, `mod.rs` e
  `demo.rs`: dati, ciclo di vita e codifica dei campi per i simulatori espliciti.
- `src/hud/halo.rs` e `src/hud/halo/shift.rs`: sola fascia avvisi Shift.
- `src/udp/mod.rs`: invalidazione per sorgenti malformate/non supportate.
- `src/app/mod.rs`: scadenza e aggiornamento del banner.
- `tests/alerts.rs` (nuovo) e `tests/protocol.rs`: verifiche dei dati e del ciclo.
- `README.md`, `docs/ARCHITECTURE.md`, `docs/PROTOCOL.md`, questo documento e
  le quattro anteprime `artifacts/shift-alerts-*.png`.

## Zona destra Shift — 18 novembre 2026

Sostituiti soltanto i contenuti marce/RPM dell'ala destra di Shift con tre
colonne: ERS e modalità, Straight Mode e distanza disponibile, margine Fuel
in giri. Restano il fondo esistente, gli helper comuni e i filtri a 500 ms.
Con capacità ERS nota si mostra la percentuale; altrimenti i MJ ricevuti.
Fuel è il margine MFD con segno e un decimale, senza ulteriori stime. Dati
assenti o scaduti producono `--%`, modalità `--`, `SM --` e Fuel `--`.
SM pre-2026 resta `SM N/A`; distanza zero non implica READY.

Valori e sottolineature di attenzione sono statici e locali: ERS esattamente
esaurito e margine Fuel visualizzato negativo usano il colore Brake;
BOOST e SM READY usano Accent; SM ON usa Throttle. Nessuna soglia ERS bassa
stimata, lampeggio o nuovo avviso temporaneo. Numeri eccezionalmente lunghi
usano la notazione scientifica per conservare la leggibilità.

Passati **101 test** (55 unit, 5 game data, 38 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release con `--locked`.
Due nuovi test controllano evidenziazioni, dati assenti, segno Fuel e
contenimento delle righe nella fascia Direct2D. Il sorgente della zona
centrale e quello di Left coincidono con la copia precedente alla modifica.

Confrontate 70 anteprime native: 14 condizioni per ciascuno dei cinque
modelli, inclusi dati parziali, scadenze indipendenti, capacità sconosciuta,
SM N/A, READY/ON, distanza massima, colori personalizzati e valori estremi.
Centro, sinistra e fascia degli avvisi di Shift coincidono nei pixel in
tutti i casi. Classic, Modern e Carbon coincidono integralmente; Wide
presenta solo due pixel con scarto massimo 4/255 in due condizioni, entro
la tolleranza antialias già usata nelle verifiche precedenti.
Ispezionate [anteprima](../artifacts/shift-right-preview.png),
[attenzione](../artifacts/shift-right-attention.png),
[MJ senza capacità](../artifacts/shift-right-mj.png) e
[dati assenti](../artifacts/shift-right-missing.png).
Nessuna nuova sessione reale nel gioco.

File di questa modifica: `src/hud/halo.rs`, `src/hud/halo/shift.rs`
(renderer Right e due test), `README.md`, `docs/ARCHITECTURE.md`, questo
documento e le quattro anteprime `artifacts/shift-right-*.png`.

## Zona sinistra Shift — 18 novembre 2026

Solo Shift sostituisce il precedente riquadro GAP con due valori etichettati
**AHEAD** e **BEHIND**, in secondi e con precisione al millisecondo. Straight
Mode e Fuel conservano posizione e comportamento; il quadrante centrale,
la zona destra e lo stato temporaneo di connessione usano il codice precedente.
Il selettore, il modello predefinito e le impostazioni salvate non sono modificati.

Verificati parser, sequenza e timestamp prima di introdurre l'andamento. Il
delta anteriore proviene dal record locale di Lap Data; quello posteriore dal
record dell'auto nella posizione successiva, riusando la lettura esistente.
Le due auto devono essere attive, fuori dal garage e univocamente identificabili
per posizione. Lo storico richiede la stessa auto, tempo di gioco crescente,
almeno un secondo sia di gioco sia di ricezione e nessun intervallo superiore
a 500 ms. La soglia è 50 ms; freccia giù = distacco in riduzione, freccia su =
distacco in aumento. Il trattino neutro copre andamento non determinato e
variazioni piccole; `--` indica un distacco assente o scaduto.

Passati **99 test** (53 unit, 5 game data, 38 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release con `--locked`.
I nuovi test coprono identità e ordinamento delle auto, zero valido, intervalli
lunghi, dati mancanti/invalidi/ambigui, soglia temporale e rumore, cambio auto,
pausa, scadenza, duplicati, riordino, tempo fermo e flashback.

Confrontate 30 anteprime Direct2D 2000 × 600 con il renderer del commit
`b89db12`: cinque modelli, ciascuno con dati live simulati, assenti, andamento
neutro, valori lunghi, fondo trasparente e Lap Data scaduto. Classic, Modern e
Carbon coincidono nei pixel in tutti gli scenari; Wide coincide in cinque
scenari e differisce in due pixel con scarto massimo 4/255 nel caso scaduto.
Nelle zone protette di Shift lo scarto massimo è un pixel con differenza
5/255; il confronto usa una tolleranza di otto pixel isolati con scarto
massimo di otto livelli sui bordi antialias.
Ispezionate le anteprime con valori normali, assenti e lunghi:
[live simulato](../artifacts/shift-left-preview.png),
[dati assenti](../artifacts/shift-left-missing.png),
[valori lunghi](../artifacts/shift-left-long-values.png).
Nessuna nuova sessione reale nel gioco.

File della modifica:

- `src/hud/halo.rs`, nuovo `src/hud/halo/shift.rs`: rendering limitato a sinistra.
- `src/telemetry/protocol.rs`, nuovo `src/telemetry/gaps.rs`,
  `src/telemetry/state.rs`, `src/telemetry/mod.rs`: intervalli, identità,
  temporizzazione, andamento e invalidazione indipendenti dai modelli esistenti.
- `src/telemetry/demo.rs`: codifica dei nuovi campi per i soli simulatori espliciti.
- `src/udp/mod.rs`: invalidazione dei nuovi dati su Lap Data malformato.
- `src/app/mod.rs`: scadenza e ridisegno dei nuovi indicatori.
- `tests/protocol.rs`: cinque nuovi test di parsing e ciclo di vita.
- `README.md`, `docs/PROTOCOL.md`, `docs/ARCHITECTURE.md`, questo documento:
  uso, fonti e verifiche.
- `artifacts/shift-left-preview.png`, `artifacts/shift-left-missing.png`,
  `artifacts/shift-left-long-values.png`: anteprime native.

## Scomparsa temporanea di entrambi gli HUD — 18 novembre 2026

Il controllo locale dei registri Windows non ha trovato crash riferiti
all'overlay o a F1 25, né reset del driver video nelle dieci ore precedenti
il controllo. Il report utente disponibile era del 9 novembre, quindi non
documenta la gara segnalata. Due avvisi generici HidBth del 18 novembre
(15:16 e 17:39, ora locale) non identificano una causa della scomparsa con USB.

Il filtro dello stick destro riguarda soltanto Halo. Entrambi gli HUD invece
si nascondono quando scadono telemetria/Lap Data (500 ms) o Session (2 s),
oppure quando il gioco segnala una pausa o la fine della guida. Non è stata
modificata questa logica in assenza di una registrazione della gara.

Aggiunto `session-events.jsonl` accanto alle impostazioni, con timestamp,
età dei pacchetti, flag pausa/sessione, controller, visibilità e codici degli
errori grafici. La scrittura avviene agli eventi e il registro ruota a
256 KiB mantenendo una copia precedente. Gli errori di scrittura vengono
esposti nella diagnostica senza interrompere l'app.

Passati **94 test** (53 unit, 5 game data, 33 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release con `--locked`.
Le [13 verifiche native con UDP sintetico](../artifacts/visibility-events-smoke.json)
confermano la registrazione distinta di interruzioni Telemetry, Lap e Session,
pausa locale e di rete, scomparsa/ritorno di entrambi gli HUD e ricreazione
grafica forzata. Nessun pacchetto malformato, errore Present o modifica alle
impostazioni. La causa nella gara reale e il confronto USB/Bluetooth restano
da verificare con il nuovo registro.

## Cornici curve Modern e dicitura GAP — 18 novembre 2026

Quattro cornici nel colore accento racchiudono Straight Mode, distacco posteriore,
FUEL e il gruppo marce nel modello Modern. La cornice destra unica include
marce, barra e scala RPM, con le etichette interne ai bordi. Bordi e angoli
arrotondati seguono gli stessi archi dei testi e della fascia Halo;
le geometrie sono precalcolate. Il distacco
mostra `GAP + 1.666`, oppure `GAP --` senza dati, in Modern, Wide e Classic.

Le etichette dei pedali usano `BRK` e `THR` in tutti i modelli Halo.

Passati **92 test** (51 unit, 5 game data, 33 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release con `--locked`.
Il nuovo test Direct2D verifica contenimento delle righe, dimensione FUEL,
corrispondenza dei bordi con gli archi e posizione interna alla fascia.
Controllate le anteprime native dei tre modelli con dati simulati, dati
assenti, SM READY/ON, countdown a 65535 metri, distacco lungo e fondo 0%.
[Anteprima Modern](../artifacts/halo-modern-panels-preview.png) e
[valori lunghi](../artifacts/halo-modern-panels-long-values.png).
Nessuna nuova sessione reale nel gioco.

## Istanza unica e avvii ripetuti — 17 novembre 2026

Un mutex Windows viene acquisito prima di caricare le impostazioni o creare
finestre, renderer e ricevitore UDP. Gli avvii successivi terminano con successo
e richiamano il pannello esistente, ripristinandolo se nascosto o minimizzato.
La chiusura dell'handle libera il blocco; Windows lo libera anche se il processo
viene terminato forzatamente. Nessun file di blocco o nuova dipendenza.

Passati **91 test** (50 unit, 5 game data, 33 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release con `--locked`.
Le [8 verifiche native](../artifacts/single-instance-smoke.json) includono
sei riavvii rapidi, otto avvii simultanei, pannello nascosto/minimizzato,
conservazione della massimizzazione e ripartenza dopo uscita normale o forzata.
Rimangono un solo processo e due overlay; le impostazioni non cambiano.
La Demo presenta frame senza errori. Nessuna nuova sessione reale nel gioco.

## Scritte e indicatori lungo la curva Carbon — 17 novembre 2026

I testi delle ali usano percorsi campionati dai due bordi della fascia; ogni
lettera è orientata secondo la tangente locale. Fuel, Straight Mode, pedali,
RPM e Behind non dipendono più da inclinazioni manuali dei singoli blocchi.
Barre pedali e LED seguono lo stesso arco, con separatori perpendicolari.
I percorsi sono memorizzati alla creazione delle risorse, senza ricostruirli
a ogni frame. Il quadrante centrale conserva la lettura orizzontale.

Passati **90 test** (49 unit, 5 game data, 33 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release. I due nuovi controlli
verificano la corrispondenza con i bordi Bézier, la simmetria delle tangenti,
la distanza delle righe e la continuità dei segmenti. Controllata
l'[anteprima Direct2D](../artifacts/carbon-curved-text-preview.png) con dati
simulati; nessuna nuova sessione nel gioco.
Lo [smoke test Demo](../artifacts/carbon-curved-text-smoke.json) di 2 secondi
ha presentato 108 frame senza errori di rendering.

## Taglio simmetrico delle estremità Carbon — 17 novembre 2026

Accorciate le estremità seguendo i due tagli indicati dall'utente. I punti di
taglio superiori sono a X 32/968 e quelli inferiori a X 20/980 sul canvas di
1000 unità: i due lati sono speculari rispetto a X 500. Le curve sono porzioni
di quelle precedenti, senza spostare testi e quadrante. Nessun bordo viene
aggiunto lungo i tagli. Verificati build Release, formattazione e
[anteprima nativa Direct2D](../artifacts/carbon-symmetric-cut-preview.png)
con dati simulati; nessuna nuova sessione nel gioco.

## Profilo Carbon con estremità aperte — 17 novembre 2026

Ridisegnato il profilo superiore seguendo le guide rosse dell'utente, eliminando
il raccordo precedente vicino al quadrante. Bordo superiore argento e inferiore
rosso proseguono separatamente fino ai lati; le chiusure del riempimento sono
esterne al canvas e non vengono tracciate. Testi, spaziatura e telemetria sono
conservati. Verificata l'[anteprima Direct2D](../artifacts/carbon-open-edges-preview.png),
88 test, Clippy e build Release. La verifica visiva usa dati simulati.

## Spaziatura delle ali Carbon — 17 novembre 2026

Ampliata simmetricamente la fascia nelle due zone esterne indicate dall'utente,
Straight Mode e Behind: maggiore distanza tra profilo superiore e bordo rosso,
circa il 45% in più nell'area dei testi. Aumentata di 11 unità la separazione
tra le due righe, senza ridurre i caratteri o spostare il quadrante.
Il riempimento segue i nuovi bordi; i separatori esterni sono stati allungati.
Canvas, scala, posizione, pneumatici e automatismo di visibilità conservati.
Verificata l'[anteprima nativa Direct2D](../artifacts/carbon-spacing-preview.png)
con gli stessi valori del riferimento. Passati 88 test, rustfmt e Clippy;
rigenerati eseguibile e pacchetto portabile.

## Carbon coordinato e visibilità automatica — 17 novembre 2026

Aggiunto **Carbon (Halo + Tyres)** al selettore esistente. Una sola scelta
aggiorna entrambi gli overlay; Wide, Classic e modern mantengono il pannello
gomme precedente. Controllati nell'app Windows il cambio modern → Carbon,
il salvataggio del modello e la conservazione di posizione, scala e opacità.
Verificati visivamente tipografia corsiva, indicatori, bordo rosso, trama
carbonio e monoposto RGBA. Corrette in verifica le distanze dal bordo di
pedali/RPM e la larghezza di SM READY. Asset e prompt in [CARBON.md](CARBON.md).

La visibilità automatica è attiva con tutti i modelli, include le prove libere
e non modifica il file delle preferenze durante i passaggi pista/menu.
Il riquadro Session indica se gli HUD attendono la pista o sono nascosti
manualmente. Demo ed Edit rimangono utilizzabili senza il gioco.

Passati **88 test** (47 unit, 5 game data, 33 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release con `--locked`.
Le fixture indipendenti verificano i campi Session e Lap Data della vettura
locale, booleani/driverStatus invalidi, tutti i tipi sessione 1–18 e tutti gli
stati pilota in pista, garage, pausa, spettatore, risultati, dati assenti e
scaduti, interruzione dell'invio, ripresa e Session Start/End.

[17 prove native con UDP sintetico](../artifacts/carbon-auto-visibility.json)
hanno verificato lo stato effettivo `IsWindowVisible` di entrambi gli HUD:
attesa, prove libere, qualifiche, Sprint Shootout, gara, Time Trial, pausa,
garage, spettatore, fine sessione, ritiro, Lap Data o Car Telemetry scaduti,
interruzione UDP, ripresa dalla pausa, uscita dal garage e Halo disabilitato
individualmente. Nessun errore di presentazione; in attesa zero frame
renderizzati. I file impostazioni sono rimasti identici durante le prove.

[Tre prove delle modalità](../artifacts/carbon-modes-smoke.json) confermano
Demo, Edit senza gioco e Carbon con fondo 0% dopo ricreazione del device:
entrambi gli HUD visibili e zero errori di presentazione.
Le prove usano dati sintetici e non comprendono una sessione reale di F1 25.
I flag UDP di pausa sono documentati per la rete; la sospensione dell'invio
offline viene gestita tramite scadenza dei dati, senza interrogare la UI del gioco.

## PS5: doppio clic sulla metà sinistra del touchpad — 13 novembre 2026

Il DualSense usa ora il clic fisico del touchpad con un contatto attivo a
sinistra, in sostituzione di Create. Restano necessarie due pressioni entro
400 ms, con rilascio fra le pressioni. I test con report USB e Bluetooth
coprono entrambi gli slot dito, limite sinistro/centro 959/960, coordinate non
valide, contatti assenti, semplice sfioramento, Create e spostamento del dito
mentre il pulsante è tenuto premuto. I batch conservano tutte le transizioni.
I report semplici senza coordinate non possono attivare il comando.
Back/View Xbox e Share DualShock 4 conservano i rispettivi controlli.

L'app richiede i report completi DualSense leggendo una volta la feature HID
seriale `0x09` da un handle condiviso, poi lo chiude. Formati, offset e dimensioni
del touchpad verificati sui sorgenti [SDL](https://github.com/libsdl-org/SDL/blob/main/src/joystick/hidapi/SDL_hidapi_ps5.c)
e sul [driver Sony per Linux](https://github.com/torvalds/linux/blob/master/drivers/hid/hid-playstation.c).

Passati **84 test** (47 unit, 5 game data, 29 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings` e build Release. Uno
[smoke test nativo di 3 secondi](../artifacts/ps5-left-touchpad-smoke.json)
ha prodotto 158 frame, una ricreazione grafica, zero errori di presentazione
e 1911 campioni Raw Input, con registrazione HID riuscita.
Il gesto fisico sul touchpad e una sessione reale di F1 25 restano da verificare.

## Select doppio e distacco posteriore live — 13 novembre 2026

Due pressioni entro 400 ms di View/Back Xbox oppure Share/Create PlayStation
alternano entrambi gli HUD. Testati fronti di pressione, rilascio obbligatorio,
limite 400/401 ms, pressione mantenuta, sequenze successive, riconnessione,
cambio dispositivo/sorgente e batch HID con tutte le transizioni. I report
DualSense semplici/USB/Bluetooth e DualShock 4 USB/Bluetooth distinguono
Share/Create da Options, stick e touchpad; XInput usa soltanto Back/View.

Il distacco legge l'intervallo anteriore della vettura nella posizione successiva
di classifica dallo stesso Lap Data. Fixture indipendenti coprono slot locali
0/23, indici avversari non contigui, minuti e millisecondi, zero, giro invalido,
ultima posizione, inseguitore inattivo, dati malformati, riordino, pausa,
scadenza a 500 ms, cambio avversario, sessione, sorgente e flashback. Il test
UDP reale verifica contemporaneamente cronologia e distacco da 2536 ms.

Passati **82 test** (45 unit, 5 game data, 29 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings`, build Release e build degli esempi.
Tre esecuzioni native isolate di 5 secondi, con UDP sintetico e una
ricreazione del device ciascuna, hanno prodotto zero errori di presentazione:

- [Wide, intervallo aggiornato da 1234 a 2345 ms](../artifacts/select-gap-wide-smoke.json):
  274 frame, telemetria LIVE e distacco finale 2345 ms.
- [Classic, inseguitore diventato inattivo](../artifacts/select-gap-classic-smoke.json):
  280 frame, telemetria LIVE e distacco finale assente.
- [Modern, solo Lap Data interrotto](../artifacts/select-gap-modern-smoke.json):
  201 frame, telemetria LIVE e distacco finale scaduto.

Entrambe le finestre conservano visibilità, topmost, click-through e mancata
attivazione. Configurazioni e porta sono separate dalle preferenze utente.
Non sono stati verificati una sessione reale di F1 25 o gesti su controller fisici.

## Straight Mode più compatto — 11 novembre 2026

SM OFF, SM ON, SM READY e countdown metri usano un carattere dedicato da
**19 px**, rispetto ai precedenti 22 px. Le quote delle righe restano condivise
con FUEL e delta. Passati **77 test**, rustfmt, Clippy e build Release.
Controllati con render nativo i tre stati e countdown da 700 e 65535 metri
su Wide, Classic e Modern: le scritte restano nel contorno.
[Anteprima countdown](../artifacts/hud-sm-countdown-preview.png).
Non è stata eseguita una sessione nel gioco.

## Due righe curve comuni per SM, delta e FUEL — 11 novembre 2026

FUEL usa ora la stessa curva di SM; il valore carburante usa quella del delta.
Ogni modello definisce una sola coppia di quote, condivisa dai due gruppi.
FUEL e valore seguono la curva glifo per glifo, con le dimensioni precedenti;
le quattro curve sono precalcolate alla creazione delle risorse grafiche.

Passati **77 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
Il nuovo test verifica che i centri delle scritte SM e FUEL appartengano agli
stessi due archi Modern. Il render nativo Direct2D è stato controllato con dati
assenti, +2.3, -0.8 e +12.3 nei tre modelli.
[Anteprima senza dati](../artifacts/fuel-rows-modern-waiting.png),
[anteprima con dati simulati](../artifacts/fuel-rows-modern-positive.png).
Non è stata eseguita una sessione nel gioco.

## Allineamento FUEL al bordo Halo — 11 novembre 2026

FUEL e valore carburante condividono la tangente del bordo inferiore nel punto
occupato dal riquadro. Classic ricava l'angolo dalla stessa Bézier usata per
disegnare il bordo; Modern riusa la tangente del proprio arco. Wide mantiene
la tangente già esistente. Rimossi gli angoli fissi di Classic e Modern.

Passati **76 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
Il nuovo test confronta la tangente Classic con la derivata della curva,
inclusa la posizione FUEL. Controllati visivamente tutti e tre i modelli.
Ogni Demo isolata dura **45 secondi**, con una ricreazione del device e zero
errori di presentazione: [Classic](../artifacts/fuel-alignment-classic-smoke.json),
[Modern](../artifacts/fuel-alignment-modern-smoke.json),
[Wide](../artifacts/fuel-alignment-wide-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Fondi indipendenti, testi curvi, delta giro e marcia — 10 novembre 2026

Applicati i criteri del meta prompt: modifiche comuni ai tre modelli, riuso
delle impostazioni esistenti, separazione di fondo e contenuti, curve ricavate
dalla geometria reale e confronto basato soltanto su tempi validi della sessione.
Confermati dall'utente fondo 0–100% e velocità grande con marcia sopra.

**Background %** e Shift + rotella ora modificano soltanto i fondi di Halo e
Tyres. Scritte, numeri, bitmap auto e indicatori restano nitidi anche a 0%.
SM e delta seguono la curva sinistra glifo per glifo in Wide, Classic e Modern.
La marcia R/N/1–8 sostituisce KM/H sopra il numero grande della velocità.

Il delta sostituisce AERO STRAIGHT/CORNER: ultimo giro completato meno miglior
giro valido precedente, con tre decimali. Rosso se più lento, verde se più
rapido, neutro al pareggio. Giri invalidi, primo riferimento assente e dati
scaduti mostrano `--`. Un record nuovo conserva il confronto con quello vecchio.
Parser Session History verificato sulla specifica EA v1.2 e sulle strutture
ufficiali. Sono coperti vettura locale, record parziale, flag di validità,
conteggi, ultimo slot, tempi nulli, reset sessione, flashback e attesa del nuovo
giro. Il test UDP invia telemetria e cronologia sulla socket reale e verifica
il delta di 230 ms e la cancellazione al cambio sorgente.

Passati **75 test**, rustfmt, Clippy `--all-targets -D warnings`, build Release
e build degli esempi. Controllati visivamente i tre modelli, segni positivo e
negativo, pareggio, marcia centrale, fondi 0/100 e pannello in finestra ridotta.
Demo isolata di **248 secondi**: **14.625 frame**, una ricreazione del device,
zero errori di presentazione. [Report](../artifacts/halo-refinement-ui-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Opacità percentuale e fondo Tyres uniforme — 10 novembre 2026

In **Halo & layout**, i campi **Opacity %** permettono di regolare Halo e Tyres
separatamente dal 15% al 100%. Valori maggiori riducono la trasparenza. Restano
compatibili i file impostazioni esistenti; applicare un valore non modificato
conserva la precisione precedente alla visualizzazione arrotondata.

Il disegno auto ora usa `assets/tyre-car-transparent.png`: il fondo originale
è rimosso con imagegen e il pannello fornisce una sola tonalità. Il PNG originale
è conservato. [Prompt e provenienza](TYRE-CAR-IMAGEGEN.md).

Passati **70 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
Estesa la verifica del pannello nativo con percentuali indipendenti, limiti
15/100, virgola decimale, valori non validi, salvataggio e precisione invariata.
Controllati visivamente campi e fondo uniforme al 100% e al 45%.

Demo isolata di **152 secondi**: **9.093 frame**, una ricreazione del device
e zero errori di presentazione. Il comando **Apply and save** ha salvato Halo
al 100% e Tyres al 45% nella configurazione di prova.
[Report](../artifacts/opacity-background-ui-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Logo monoposto e telemetria — 10 novembre 2026

Il PNG scelto è copiato senza modifiche in `assets/app-logo.png`. La conversione
ICO conserva trasparenza e proporzioni e include 10 dimensioni da 16 a 256 px.
Le risorse Windows sono incorporate nell'eseguibile: il nuovo simbolo compare
nel pannello, nella barra del titolo, nelle icone delle finestre e nella tray.
Gli handle creati sono rilasciati alla chiusura e il logo resta disponibile
quando il pannello ricrea il tema al cambio DPI.

Passati **70 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release
GNU x64. Il nuovo test carica dal modulo tutte le 10 dimensioni e verifica
dimensioni effettive e profondità colore a 32 bit. Verificata nel PE la presenza
di 10 risorse ICON e una GROUP_ICON. Controllato visivamente il logo nel pannello
e nella barra del titolo dell'eseguibile distribuito.

Avvio Demo isolato di **33 secondi**: **1.962 frame** e zero errori di
presentazione. [Report](../artifacts/logo-smoke.json). La verifica usa una
configurazione temporanea. Non sono stati provati MSVC e una sessione nel gioco.

## Halo nelle visuali laterali — 10 novembre 2026

Il filtro della visuale posteriore ora nasconde Halo anche spostando lo stick
destro a destra o a sinistra. Halo torna al centro; i passaggi diagonali fra
retro e lati mantengono il filtro attivo. Il pannello mostra **Hide Halo: rear /
sides**. La preferenza esistente `hide_halo_rear_view` conserva il proprio valore.

Passati **69 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
I controlli coprono entrambi gli assi DualSense USB/Bluetooth e XInput, soglie
55%/35%, rilascio, input scaduto o rimosso, direzioni diagonali e conservazione
delle preferenze Halo e pneumatici. Stick sinistro e clic R3 non attivano il filtro.

Avvio Demo di 3 secondi: **167 frame**, registrazione Raw Input riuscita e zero
errori di presentazione. [Report](../artifacts/halo-side-view-smoke.json).
Non sono stati provati un controller fisico e una sessione nel gioco.

## Responsive Settings window — 9 November 2026

Settings now supports dragging native window edges and corners, maximizing,
restoring and Windows snapping. Initial size fits the monitor under the pointer.
Controls and GDI decoration use the same responsive geometry: general controls
move below the settings in portrait-width windows, categories move to the top
in compact windows, and setting tiles stack when necessary. Fonts retain their
DPI size; scrolling and focus visibility keep all controls accessible.

All **67 tests**, rustfmt, Clippy `--all-targets -D warnings` and Release build
passed. Layout checks cover nine viewport shapes down to 320 logical pixels.
The native test resizes all six categories across landscape, portrait and narrow
windows, including 1920 × 1040, 1080 × 1800, 720 × 1100 and 360 × 700, with
96/144/192 DPI paths. It checks control bounds and overlaps, keyboard focus
visibility, diagnostic scroll preservation and retention of unsaved settings.

In the Windows app, dragged the bottom-right corner and left edge to narrow
portrait-shaped windows, reached all bottom commands with the scrollbar, and
maximized back to the full landscape screen. The earlier hover regression
checks still pass. Physical monitor rotation and a live F1 driving session
were not performed.

## Titolo, usura generale e colonne simmetriche — 9 November 2026

Il titolo è ora **TYRES**, da 14 a 18 px. La legenda inferiore è rimossa e
la percentuale in alto sostituisce l'etichetta della modalità temperature.
Il valore è la media delle quattro gomme e diventa `--%` senza usura recente.
Etichette, percentuali e temperature sono centrate in colonne da 82 unità,
con margini speculari rispetto all'immagine centrale.

Build Release, rustfmt, Clippy `--all-targets -D warnings` e **65 test** passati.
Il nuovo test copre media, estremi e dati assenti. Verificata la finestra nativa
senza telemetria e con UDP sintetico: 13%, 15%, 18% e 22% producono **17%**.
Prova di 75 secondi: 707 frame, una ricreazione del device e zero errori
di presentazione. [Report](../artifacts/tyre-layout-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Immagine auto nel mini HUD pneumatici — 9 November 2026

Sostituita la sagoma geometrica con il PNG verticale fornito, incorporato
nell'eseguibile senza modificare il file originale. L'immagine conserva le
proporzioni e lascia leggibili usura e temperature dei quattro pneumatici.

Build Release, rustfmt, Clippy `--all-targets -D warnings` e **64 test** passati.
Controllata la finestra nativa al 100%, anche dopo la ricreazione delle risorse.
Demo di 90 secondi al 170%: **5.367 frame**, una ricreazione del device,
zero errori di presentazione. [Report](../artifacts/tyre-car-smoke.json).
La verifica usa dati Demo; non include una sessione nel gioco.

## Settings hover and repaint loop — 9 November 2026

Fixed the Settings hover redraw: moving within an unchanged control no longer
invalidates the panel. Push buttons use the native owner-draw callback, and
combobox content and chrome share one paint cycle with DC state restored.
Checkboxes retain their native keyboard and accessibility behaviour.

The native regression test failed before the fix when mouse movement over
Halo X invalidated the whole panel. It now checks inputs, buttons, switches
and comboboxes, repeated mouse moves, and printed control skin/captions.
All **64 tests**, rustfmt, Clippy `--all-targets -D warnings` and Release build
passed. Checked category navigation and opening/closing the temperature menu
in the Windows app.

Also fixed a repeating paint queue from hidden composition HUDs by completing
their `BeginPaint` / `EndPaint` cycle. A diagnostic run counted **49,098**
overlay paint messages in three seconds before the fix; the fixed five-second
run processed **two**, one per HUD. In the same hidden-HUD Demo scenario,
sampled CPU fell from approximately **99% to 1.6% of one logical core**.
The temporary tracing code was removed before packaging.
[Verification record](../artifacts/settings-hover-verification.json).

A live F1 driving session was not run for this UI correction.

## Navy / cyan Settings UI — 8 November 2026

Restyled the native Settings panel with a category sidebar, outlined settings
panels, cyan selections and switches, and a persistent general-controls pane.
All existing fields and commands remain available across six categories.
The palette and panel arrangement follow the supplied visual reference.

Release build, rustfmt, Clippy `--all-targets -D warnings` and all **64 tests**
passed. The new native panel test clicks every category through the Win32
command queue, checks which controls are visible, changes DPI through 144,
192 and 96, and verifies that unsaved IP, temperature, colour, shortcut,
profile name and rear-view values survive. Checked the Layout page visually
in the Windows application, including custom comboboxes and switches.

A 15-second Release Demo run rendered **882 unique telemetry frames**, with
one forced device recreation and **zero presentation failures**. The saved
test configuration remained byte-for-byte unchanged, and both HUDs retained
topmost, click-through and no-activation flags.
[Runtime report](../artifacts/settings-ui-smoke.json).

A live F1 driving session and physical DPI transitions between monitors were
not checked for this Settings-only change.

## Modern pedal inclination — 8 November 2026

Modern Brake and Throttle bars now rotate by -9° and +9° around centres at
equal distances from the central dial. Segment dimensions and vertical pivots
are identical; labels rotate with the bars. Pedal values remain independent.
Release build, rustfmt, Clippy `--all-targets -D warnings` and all **63 tests**
passed. Checked the regenerated native-renderer preview; no live driving run
was performed for this visual adjustment.

## Modern Halo model — 8 November 2026

Added **modern** to **Halo model**, alongside Wide and Classic. It follows the
attached reference with one shaded charcoal ribbon, curved ends, a thin upper
arch, an orange lower edge, a larger central dial and integrated fuel/SM/Aero
and gears/RPM. The gradient brush is cached per surface. All telemetry remains
live and shares the existing freshness, missing-data and ERS/SM rules.

Release build, rustfmt, Clippy `--all-targets -D warnings` and **63 automated
tests** passed. The new geometry check covers symmetry, curved end caps,
canvas bounds and room for gear labels; the existing settings round trip now
includes Modern. The regenerated Wide and Classic exports match their previous
PNG pixels exactly.

Selected modern from the native dropdown and used Apply and save on an isolated
configuration. Model persisted as Modern; Halo/tyre layouts, opacity and named
profiles were retained. Relaunch restored Modern with no presentation errors.
[Restart report](../artifacts/halo-modern-restart.json).

A 20-second Release Demo run rendered **1,183 unique telemetry frames** with
one forced device recreation and **zero presentation failures**. Both HUDs
retained click-through, topmost and no-activation flags.
[Runtime report](../artifacts/halo-modern-smoke.json).

The 4000 × 1200 PNG preview comes directly from the same Direct2D renderer with
fixed example telemetry. A live driving session and alignment with the user's
camera remain unverified.

## ERS mode, Straight Mode countdown and fuel margin — 8 November 2026

Release build, rustfmt, Clippy `--all-targets -D warnings`, all examples and
**62 automated tests** passed. Tests cover the signed MFD fuel field at offset
13, invalid floating-point values, deploy mode captions, negative zero,
SM activation/availability priority, distance handling and demo round trips.

Checked both Wide and Classic in the Windows app using synthetic UDP on a
separate port and an isolated settings file:

- The battery caption shows NONE, MEDIUM, HOTLAP or BOOST. The old ERS caption
  beneath Active Aero is absent.
- SM OFF updates from 350 m to 125 m; availability shows SM READY and activation
  shows SM ON with a green background.
- The small FUEL pod displays + 2.3, - 0.8 and + 12.3. The right gears/RPM band
  remains available. Missing status clears fuel and ERS; expired telemetry
  clears all live values and displays STALE DATA.
- Settings, model choices, context menus, status messages and edit hints are
  English. Switching Wide to Classic preserves X, Y, scale and opacity.

The end-to-end run accepted 27,678 packets with zero malformed, unsupported
or reordered packets, and zero presentation failures.
[Diagnostic report](../artifacts/hud-fuel-sm-update-smoke.json).
The check used synthetic data; a live F1 25 driving session remains unverified.

## Profili Halo con nome — 6 novembre 2026

Aggiunti il campo modificabile **Profilo Halo** e i comandi **Salva profilo** e
**Carica profilo**. Ogni nome conserva X, Y e Scala; salvare un nome esistente lo
aggiorna. La scelta è manuale, persistente e compatibile con i due modelli Halo.
I valori salvati non vengono sovrascritti spostando il layout corrente.

Build Release, fmt, Clippy `--all-targets -D warnings` e **59 test automatici**
passati. I quattro nuovi test verificano compatibilità dei JSON precedenti,
isolamento dei profili, richiamo delle sole geometrie, aggiornamento del nome
senza duplicati per maiuscole/spazi, persistenza su disco e rifiuto di nomi,
scale o riferimenti non validi senza modificare i profili salvati.

Verificati nell'app Windows su configurazione e porta separate:

- Creazione **Ferrari** a (455, 537), scala effettiva 1,037, e **Red Bull** a
  (300, 450), scala 0,82. Valori presi dai campi Halo e applicati subito.
- Scelta Ferrari dal menu e Carica: ritorno alle coordinate salvate; pneumatici,
  modello, colori e comando retrovista conservati.
- Riavvio: nome Ferrari ripristinato e richiamo Red Bull ancora funzionante.
  Un'opacità modificata nel campo ma non applicata resta nel pannello, senza
  essere applicata o scartata dal caricamento della geometria.
- Errore di scrittura provocato bloccando soltanto il JSON di prova: file e
  mappa dei profili invariati, larghezza Halo ripristinata a 1037 pixel dopo il
  tentativo di applicare scala 0,82. Nessun errore grafico.
  [Diagnostica dopo il ripristino](../artifacts/halo-profiles-ui.json).

L'allineamento dei singoli profili alla visuale delle diverse vetture va regolato
nel gioco. Le coordinate sopra sono soltanto dati della prova locale.

## Correzione del comando: stick destro verso il basso

Il comando concordato è lo **stick destro verso il basso, senza clic R3**.
Aggiornati i decoder Raw Input USB/Bluetooth e il fallback XInput (`sThumbRY`).
L'asse sinistro e i clic degli stick non attivano più il filtro. La diagnostica
esporta `right_stick_down`. Soglie, rilascio e visibilità dei due HUD conservati.

Aggiornato il test dei report per verificare il movimento destro e ignorare
asse sinistro e clic in tutti e quattro i formati supportati. La verifica fisica
in gioco con il controller dell'utente resta necessaria.

## Prima implementazione: sospensione con stick sinistro verso il basso

Aggiunta **Nascondi Halo dietro**, attiva per default e salvabile dal pannello.
L'asse sinistro verso il basso applica un filtro temporaneo soltanto al Halo;
il rilascio conserva tutte le preferenze manuali. La funzione richiede guida
live, Race Mode e F1 25 in primo piano. Demo, Edit, pausa, dati scaduti o perdita
del dispositivo ripristinano il layout previsto dalle preferenze manuali.

Build Release, fmt, Clippy `--all-targets -D warnings` e **55 test automatici**
passati. I sei nuovi test coprono formati DualSense USB/Bluetooth, asse corretto
indipendente da L3 e stick destro, report multipli e malformati, soglie con
isteresi, rilascio, input scaduto/rimosso, combinazioni di visibilità manuale e
migrazione/salvataggio del nuovo flag. Nessuna nuova dipendenza Rust.

Verificati nell'app Windows la casella inizialmente selezionata e il salvataggio
di `false` con Applica e salva. Modello Classico, geometrie, precisione di scala
e opacità dei due HUD conservati nel JSON di prova, su porta separata dal gioco.

Smoke test Release Demo di 12 secondi: **699 frame**, 583 frame telemetrici
distinti, una ricreazione delle risorse e zero errori di presentazione. Entrambi
gli HUD restano topmost, click-through e senza attivazione. Registrazione Raw
Input riuscita; nessun campione controller ricevuto durante la prova. Demo
correttamente non attiva il filtro. [Report](../artifacts/halo-rear-view-smoke.json).

La pressione fisica del DualSense, il mantenimento della visuale posteriore e
il ritorno davanti in una sessione reale restano da verificare sul controller
dell'utente. I test del decoder usano report sintetici, non una cattura hardware.

## Selezione dei modelli Ampio e Classico

Nel pannello è disponibile **Modello Halo**, con scelta **Ampio** e **Classico**.
Il modello Classico riprende il secondo riferimento, mantenendo tutte le
telemetrie aggiornate del primo: km/h, ERS centrale azzurro in zona Overtake,
pedali, marce/RPM, **SM ACTIVE** / **SM IN-ACT** e Active Aero. La logica dei dati
è condivisa. Non vengono ripristinate mph o le vecchie targhette decorative.

Scelta persistente nel profilo; i file precedenti aprono Ampio. Canvas e centro
del quadrante sono comuni ai due modelli. I decimali di scala/opacità impostati
con il mouse vengono conservati quando i relativi campi non cambiano.

Build Release, fmt, Clippy `--all-targets -D warnings` e **49 test automatici**
passati. Verificato nell'app il passaggio Ampio/Classico/Ampio tramite il menu,
seguito da Applica e salva: scelta salvata, posizione, scala e opacità conservate.
Verificati nel Classico ERS azzurro, stato SM inattivo e attivo e assenza delle
vecchie telemetrie. Le anteprime usano traffico sintetico su una porta separata.
[Ampio](../artifacts/halo-model-wide-preview.png),
[Classico](../artifacts/halo-model-classic-preview.png).

Controlli al riavvio di 12 secondi per modello, inclusa ricreazione delle risorse:

- Ampio: 700 frame telemetrici distinti, zero errori, una ricreazione.
  [Report](../artifacts/halo-model-Wide-smoke.json).
- Classico: 698 frame telemetrici distinti, zero errori, una ricreazione.
  [Report](../artifacts/halo-model-Classic-smoke.json).

La prova nella propria visuale di gioco resta manuale.

## Arcata ampliata sulle guide gialle

Sostituito il profilo precedente con un arco più ampio. Il bordo superiore passa
da 712 a 875 unità di larghezza; le estremità sono meno ripide. Canvas 1000 × 300,
con spazio laterale aggiunto senza ingrandire quadrante o caratteri. Marce,
scala RPM e pannello SM seguono i nuovi bordi. Testo inattivo: **SM IN-ACT**.

Build Release, fmt, Clippy `--all-targets -D warnings` e **46 test automatici**
passati. Un test confronta 13 punti dei bordi con le guide gialle del riferimento,
con tolleranza di 8 pixel per il tratto a mano. Verificati nella finestra nativa
la scala 100% e 80%, ERS azzurro e fascia SM attiva/inattiva.

[Anteprima nativa con dati simulati](../artifacts/halo-wide-preview.png).
Smoke test Release di 12 secondi: 699 frame telemetrici distinti, una ricreazione
delle risorse grafiche, zero errori di presentazione. Verificate dimensioni
1000 × 300 e proprietà topmost, senza attivazione e click-through.
[Report runtime](../artifacts/halo-wide-smoke.json).

## ERS centrale, Straight Mode e curva del cockpit

Conservate le dimensioni Halo 840 × 300. Pannelli, numeri delle marce e scala RPM
seguono ora lo stesso profilo del bordo interno, più piatto al centro e più
inclinato alle estremità. Rimossi mph, HALO/26, targhetta LIVE, targhetta Overtake
e popup Straight Mode. ERS al centro sotto i km/h; stato SM nella fascia sinistra.

Build Release, fmt, Clippy `--all-targets -D warnings` e **45 test automatici**
passati. I test Halo coprono Overtake disponibile prima del boost, stato SM
indipendente dalla disponibilità, invalidazione dei dati e continuità del profilo.

Verificati nell'app con UDP sintetico su porta separata:

- ERS bianco fuori zona e azzurro con Overtake disponibile, senza boost attivo.
- **SM IN-ACTIVE** quando spento; **SM ACTIVE** con fascia verde quando attivo.
- ERS 0%, 50% e 100%; scala 100% e 80%, senza tagli delle scritte.
- Vettura pre-2026: **SM N/A**, Aero N/A e MJ quando la capacità non è nota.
- Dati scaduti: velocità, ERS e stato SM sostituiti da `--`, con avviso compatto.

[Anteprima nativa con dati simulati](../artifacts/halo-ers-sm-preview.png).
Smoke test Release di 12 secondi con entrambi gli HUD e ricreazione delle risorse
dopo 3 secondi: 698 frame telemetrici distinti, una ricreazione, zero errori di
presentazione. [Report runtime](../artifacts/halo-ers-sm-smoke.json).
La posizione rispetto al Halo reale va verificata nella propria visuale di gioco.

## Ridisegno Halo dal riferimento visivo

Halo portato a 840 × 300 unità, con quadrante centrale km/h e mph, ali inclinate,
pedali a segmenti e scala marce/RPM. Palette iniziale arancione, verde lime e
rosso; i colori restano modificabili. Il riquadro laterale indica la marcia
corrente, perché non è disponibile un dato sul numero della prossima curva.

Build Release, fmt, Clippy `--all-targets -D warnings` e **45 test automatici**
passati (15 unit, 5 asset/handling, 22 protocollo, 3 UDP). Aggiunto un controllo
della conversione mph: 350 km/h corrispondono a 217 mph arrotondate.

Verifiche nell'app Windows con telemetria sintetica su porta separata:

- Stato di attesa e dati scaduti: velocità, marcia, Aero e percentuali diventano
  `--`, senza conservare valori di guida precedenti.
- Scala 100% e 80%, marcia 8, folle e retromarcia: numeri e selezione della
  marcia restano allineati sulle ali inclinate.
- Passaggio Corner/Straight: cambia l'indicazione Aero e scompare il popup
  di disponibilità all'attivazione dello Straight Mode.
- Race Mode: clic sui pixel del tachimetro ricevuto dalla finestra probe
  sottostante, con contatore da 0 a 1.

Smoke test Release di 12 secondi, entrambi gli HUD visibili e ricreazione delle
risorse dopo 3 secondi: 701 frame telemetrici distinti, una ricreazione eseguita,
zero errori di presentazione. Finestre topmost, senza attivazione e click-through;
dimensioni Halo 840 × 300. [Report runtime](../artifacts/halo-redesign-smoke.json).
La verifica usa dati simulati; non include una nuova sessione di guida nel gioco.

## Aggiornamento 0.2.0: batteria, Straight Mode e visibilità

Build Release, fmt, Clippy `--all-targets -D warnings` e **44 test automatici**
passati (14 unit, 5 asset/handling, 22 protocollo, 3 UDP). Le fixture di handling
sono scritte per i test e non contengono asset EA.

- Rilevata automaticamente l'installazione Steam corrente. Letti 44 record di
  handling: 15 per il 2025 e 29 per il 2026, tutti a 4 000 000 J, nessun errore.
  L'esecuzione locale del lettore Release, inclusi avvio e output, ha richiesto
  circa 154 ms. [Risultato della lettura](../artifacts/game-battery-v0.2.0.json).
- App nativa collegata a traffico UDP sintetico indipendente: 2 000 000 J di
  energia ricevuta, capacità automatica `game_files` di 4 000 000 J, display e
  diagnostica al 50%. Il vecchio valore manuale 8 500 000 J nel JSON di prova
  è stato ignorato e rimosso al salvataggio.
- Mini popup **STRAIGHT MODE DISPONIBILE** osservato sul Halo con disponibilità
  attiva; scomparsa osservata al passaggio in Straight Mode.
- `Ctrl+Shift+O` premuto da una finestra di un altro processo: entrambi gli HUD
  nascosti, poi ripristinati con le stesse geometrie. Focus rimasto al probe.
  Nei due report a distanza di 24,375 s i frame totali restano 6428, mentre i
  pacchetti accettati aumentano di 2439: la ricezione continua senza rendering.
  [Prima](../artifacts/features-hidden-a-v0.2.0.json),
  [dopo](../artifacts/features-hidden-b-v0.2.0.json).
- `Ctrl+Shift+H` ancora funzionante dopo il toggle. Click sui pixel del Halo
  ricevuto dal probe, contatore da 0 a 1 e focus conservato.
- Passaggio UDP / Demo / UDP verificato: provenienza `demo` separata, epoche
  0/1/2; ritorno a `game_files`, 50%, finestre topmost e click-through, nessun
  errore di presentazione.
  [Demo](../artifacts/features-demo-v0.2.0.json),
  [stato finale](../artifacts/features-v0.2.0.json).

Controllo prestazioni 0.2.0 separato dalle prove UI: UDP sintetico 60 Hz per 30 s,
pannello chiuso, due HUD visibili, nessuna acquisizione UI durante la misura.
Media 59,96 Hz; draw CPU p95 1,12 ms; intervallo p95 17,03 ms;
RX→Present submit p95 1,92 ms; GPU p95 1,93 ms; CPU media 0,52%;
working set finale 55,13 MiB; zero errori di presentazione.
[Report](../artifacts/udp-60-v0.2.0.json),
[campioni memoria](../artifacts/udp-60-v0.2.0-memory.csv).
Sono misure del processo HUD e traffico sintetico, non FPS del gioco o latenza
ottica. La lettura statica è della capacità nominale; una sessione di guida
reale e gli eventuali modificatori runtime restano da verificare.

## Ambiente e build iniziale 0.1.0

- Windows 11 Pro x64, build 10.0.26200.
- AMD Ryzen 5 7600, NVIDIA GeForce RTX 4070, desktop 1920 × 1080.
- Rust 1.99.1, target `x86_64-pc-windows-gnu`, linker Rust LLD, crate windows 0.62.2.
- Profilo Release con thin LTO, un codegen unit e debug info rimosse.
- Compilazione, `cargo fmt --check`, Clippy con `-D warnings` e 26 test automatici passati.

Le misure sono state eseguite sequenzialmente sul desktop corrente, con pannello
chiuso e due HUD visibili. Il traffico UDP proviene dal generatore esplicito del
progetto, su porte locali separate dal gioco. F1 25 non era in esecuzione.
Le normali applicazioni del desktop non sono state disattivate per isolare il PC.

## Risultati misurati della 0.1.0

| Scenario | Durata | Hz medi | Draw CPU p95 | Intervallo p95 | Generazione/RX→Present p95 | GPU p95 | CPU media processo | RAM finale |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Demo 60 Hz | 30 s | 60,00 | 1,10 ms | 17,04 ms | 13,59 ms | 0,50 ms | 0,81% | 55,83 MiB |
| UDP sintetico 60 Hz | 120 s | 60,00 | 1,05 ms | 17,01 ms | 17,29 ms | 0,38 ms | 0,72% | 55,43 MiB |
| UDP sintetico 120 Hz | 30 s | 120,00 | 1,03 ms | 8,62 ms | 1,53 ms | 0,47 ms | 1,32% | 55,50 MiB |
| Inattività | 8 s | — | 8,53 ms¹ | — | — | — | 0,23% | 52,32 MiB |
| Ricreazione risorse al secondo 3 | 10 s | 59,18 | 1,15 ms | 17,07 ms | 16,02 ms | 1,21 ms | 0,92% | 55,34 MiB |
| Interruzione UDP dopo 3 s | 8 s | 59,99² | 1,17 ms | 17,02 ms | 17,40 ms | 1,04 ms | 0,46% | 53,13 MiB |

¹ Soltanto **due frame complessivi** in otto secondi, all'avvio: il percentile
su due campioni include il costo iniziale di font e risorse. Non è un loop a FPS pieno.
² Frequenza durante il tratto attivo; lo stato finale è **DATI OBSOLETI**.
Anche gli scenari UDP ordinari interrompono il sender prima dell'uscita dell'app,
quindi possono terminare correttamente nello stato dati obsoleti.

Nessuno scenario ha registrato errori di presentazione. Il test di ricreazione
ha registrato **una ricreazione completa**, con ripresa della visualizzazione.
La riduzione della media in quello scenario comprende la pausa di ricreazione.
Le due finestre risultavano topmost, senza attivazione e con stile transparent.

L'intervallo p50 del test UDP 60 è **16,64 ms**, p99 **17,08 ms**. I 16,67 ms
sono il riferimento a 60 Hz, non un valore identico garantito per ogni frame.
La latenza misurata raggiunge il target preferito sotto 35 ms p95 nei test
sintetici indicati; non è una misura ottica né una verifica di impatto sul gioco.

CPU è la media del processo normalizzata sui core logici, non l'uso di un singolo
core. I percentili e gli Hz usano gli ultimi 4096 campioni disponibili; il conteggio
totale e la CPU media dell'intero processo sono separati. La Demo misura dalla
generazione, UDP dal ritorno di recv_from. Il timer GPU esclude DWM e scansione
del monitor. Queste distinzioni impediscono di attribuire alla GPU o alla rete
una misura che comprende fasi diverse.

Report grezzi:

- [Demo 60](../artifacts/demo-60.json)
- [UDP 60, 120 secondi](../artifacts/udp-60-v0.1.0.json)
- [UDP 120](../artifacts/udp-120.json)
- [Inattività](../artifacts/idle.json)
- [Ricreazione device](../artifacts/recreate-device.json)
- [Interruzione UDP](../artifacts/udp-disconnect.json)

## Memoria e microbenchmark

Nel test UDP di 120 s il working set iniziale campionato era **53,31 MiB**,
il picco campionato **55,48 MiB**, il finale **55,43 MiB**. Dopo il riscaldamento,
da 40 a 110 s, i campioni ogni 10 s restano tra **55,42 e 55,46 MiB**. I private
bytes restano circa 62,27–62,39 MiB e gli handle tra 552 e 556 nello stesso tratto.
Non è stata osservata crescita continua in questo intervallo; una prova di due
minuti non dimostra l'assenza di ogni leak.

[Campioni RAM, private bytes e handle](../artifacts/udp-60-v0.1.0-memory.csv).

Il microbenchmark parser ha misurato **12,37 ns/pacchetto** su 2 000 000 parse
di un Car Telemetry già in cache, selezionando la vettura locale. È la media di
questa specifica prova: non include socket, copia kernel, parsing dei 24 record
completi o rendering. Il risultato non rappresenta il throughput UDP del sistema.

## Verifiche funzionali osservate

| Area | Verifica eseguita |
| --- | --- |
| DirectX e alpha | Entrambi gli HUD visibili; dati e sagoma pneumatici aggiornati |
| Dati Demo | Telemetry, Status e Aero circa 60 Hz; Damage circa 10 Hz |
| Drag | Gesto mouse reale: Halo da (650,686) a (760,591), coordinate salvate nel JSON |
| Resize | Gesto dall'angolo, scala 1,00 a 0,8548387, rapporto conservato, swap chain ridimensionata |
| Impostazioni | Scala 1,10, opacità 0,70, accento #FFB84D applicati live e riscontrati su disco |
| Temperature | Both, Inner e Surface selezionati live; valori e righe corrispondenti osservati |
| Pannello | Chiusura del pannello con aggiornamento degli HUD ancora attivo |
| Hotkey | Ctrl+Shift+H da un secondo processo alterna Edit e Race |
| Focus | Esc da un HUD attivo in Edit ripristina il focus alla finestra probe precedente |
| Click-through | Clic sui pixel di entrambi gli HUD ricevuti da un altro processo: contatore 0, 1, 2; focus conservato |
| Connessione | UDP reale su loopback, passaggio Demo/UDP, errore porta occupata e retry automatico |
| Protocollo | Fixture indipendenti, vettura 23, marcia R, offset 2026, gomme, ERS, lunghezze, NaN, sessioni e sequenza |

Il confronto tra sorgenti e i contatori di errore sono testati senza collegarsi
al gioco. Le fixture non sono catture di una sessione reale.

## Prove ancora necessarie sul gioco e su altro hardware

- Sessione reale di F1 25 con 2026 Season Pack, compresi flashback del gioco e cambi modalità.
- Impatto sugli FPS e sui frame time del gioco, con e senza overlay.
- Latenza ottica con strumenti di acquisizione adeguati.
- Reset reale del driver o rimozione device; la prova svolta ricrea volontariamente le risorse dell'app.
- Più monitor fisici con DPI diversi, HDR, altri driver e configurazioni fullscreen esclusive.
- Test prolungati di ore e profiling heap/GPU per una ricerca completa di leak.

Le API per DPI, clamp visibile, adattamento al refresh e recupero device sono
implementate. Le prove hardware sopra elencate restano esplicitamente non verificate.
