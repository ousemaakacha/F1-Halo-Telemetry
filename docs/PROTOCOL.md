# Contratto UDP 2026 verificato

Consultazione: **5 novembre 2026**. Pagina EA versione 11.0. Revisione più recente
allegata: **Data Output from F1 25 2026 Season Pack (Season 8), v1.2**.
La v1.2 aggiunge identificatori piloti F2 per lo Sports Update; i pacchetti usati
qui restano versione 1.

- [Pagina EA con revisioni](https://forums.ea.com/blog/f1-games-game-info-hub-en/ea-sports%E2%84%A2-f1%C2%AE25-2026-season-pack-udp-specification/12187347)
- [PDF ufficiale Season 8 v1.2](https://forums.ea.com/t5/s/tghpe58374/attachments/tghpe58374/f1-games-game-info-hub-en/61/10/Data%20Output%20from%20F1%2025%202026%20Season%20Pack%20%28Season%208%29.pdf)
- [Strutture C++ ufficiali](https://forums.ea.com/t5/s/tghpe58374/attachments/tghpe58374/f1-games-game-info-hub-en/61/8/2026%20Season%20Pack%20Telemetry%20Output%20Structures%20%281%29.txt)

SHA256 dei file consultati, conservati localmente nella cache e non redistribuiti:

```text
PDF  3F38858C3CA65B2FAA55A90A35277DD2767BB9CEA2911E741B61B370A39365AE
TXT  4F97867924F5F13F11B7FDE6EB84CCB3AEFCAF69062424BC077E387090647C72
```

## Header

29 byte, packed, Little Endian. Nessun cast a strutture packed in Rust.

| Campo | Offset byte | Tipo |
| --- | ---: | --- |
| packetFormat | 0 | u16, deve essere 2026 |
| gameYear / major / minor | 2 / 3 / 4 | u8 |
| packetVersion / packetId | 5 / 6 | u8 |
| sessionUID | 7 | u64 |
| sessionTime | 15 | f32, secondi |
| frameIdentifier | 19 | u32 |
| overallFrameIdentifier | 23 | u32 |
| playerCarIndex / secondaryPlayerCarIndex | 27 / 28 | u8 |

Il PDF mostra gameYear 25 nell'esempio, mentre il TXT mostra 26. La selezione
del layout binario usa packetFormat e packetVersion; non viene dedotto dall'anno.
Player 255 non indicizza gli array e viene ignorato. Indici 24–254 sono invalidi.

## Pacchetti letti

| ID | Pacchetto | Byte | Stride vettura | Frequenza documentata |
| ---: | --- | ---: | ---: | --- |
| 1 | Session | 926 | — | 2 Hz |
| 2 | Lap Data | 1399 | 57 | Menu |
| 3 | Event | 45 | — | Eventi |
| 4 | Participants | 1470 | 60 | Ogni 5 s |
| 6 | Car Telemetry | 1448 | 59 | Menu |
| 7 | Car Status | 1445 | 59 | Menu |
| 10 | Car Damage | 1133 | 46 | 10 Hz nel capitolo specifico |
| 11 | Session History | 1460 | 14 per giro | 20 Hz complessivi, vetture a rotazione |
| 16 | Car Telemetry 2 | 269 | 10 | Menu |

Per gli ID letti vengono verificate versione 1 e lunghezza esatta, compresi i
campi finali non visualizzati. Le altre categorie non modificano lo snapshot.
Gli array contengono 24 slot e il parser legge soltanto la vettura locale.

Offset relativi all'inizio del record vettura:

| Pacchetto | Campi usati e offset |
| --- | --- |
| Car Telemetry | speed u16=0; throttle f32=2; brake f32=10; gear i8=15; RPM u16=16; rev%=19; LED u16=20; surface[4] u8=30; inner[4] u8=34 |
| Car Status | fuelRemainingLaps f32=13 (margine MFD con segno); maxRPM u16=17; ERS energy f32=37; deployment u8=41; harvest limit f32=50; networkPaused u8=58 |
| Car Damage | wear[4] f32=0,4,8,12 |
| Car Telemetry 2 | aeroMode u8=0; aeroAvailable u8=1; aeroDistance u16=2; overtakeAvailable u8=4; overtakeActive u8=5; overtakeDistance u16=6; regulations2026 u8=8 |
| Lap Data | currentLapNum u8=33; resultStatus u8=45 |

Session History usa offset assoluti: `carIdx` u8=29, `numLaps` u8=30,
`numTyreStints` u8=31, numeri dei giri migliori u8=32–35. I record dei giri
iniziano a 36 e hanno stride 14: tempo u32 a offset 0 e validità u8 a offset 13.
Il bit 0 identifica un giro valido. `numLaps` comprende il giro parziale corrente,
che viene escluso dal confronto. Si accettano soltanto la vettura locale,
massimo 100 record e 8 stint; conteggi, indici migliori e flag riservati sono
validati. Il delta è ultimo giro concluso meno miglior giro valido precedente.
L'ultimo giro invalido o l'assenza di un riferimento non producono un delta.

Session usa gamePaused a offset corpo 14 e formula a offset corpo 8.
Il carburante usa direttamente `m_fuelRemainingLaps`, il valore del MFD:
non viene sottratta di nuovo la distanza gara. Sono ammessi valori negativi
finiti. Straight Mode usa prima `activeAeroMode` per **SM ON**, poi
`activeAeroAvailable` per **SM READY**; altrimenti mostra **SM OFF** con
`activeAeroActivationDistance` soltanto se maggiore di zero. Zero significa
assenza di una distanza di attivazione, non conferma di ingresso in zona.
Participants legge numActiveCars a offset assoluto 29. Non presume che gli slot
validi siano contigui: l'attività locale è verificata con resultStatus.

## Ordinamento e validità

Il contatore overall viene confrontato separatamente per tipo di pacchetto,
con aritmetica seriale wrapping u32. Pacchetti di categorie diverse con lo stesso
frame sono accettati. Duplicati e riordini non sostituiscono i valori recenti.
Il reset di frame/sessionTime con overall crescente indica un flashback e
invalida i dati delle altre categorie prima di applicare la nuova timeline.
Il cambio UID svuota lo stato; vengono ricordati gli ultimi otto UID ritirati.
Un reset dei contatori con lo stesso UID dopo oltre 2 s di silenzio è trattato
come riavvio. Session End invalida i dati; Session Start consente la ripresa.

Sono respinti float non finiti, comandi pedali fuori 0–1, usura fuori 0–100,
booleani non validi, marce fuori R/N/1–8, bit LED riservato 15 e lunghezze errate.
Il parser non esegue allocazioni e non contiene codice unsafe.

Gli input dei test sono fixture binarie indipendenti e pacchetti sintetici.
Non sono catture provenienti da una sessione reale di F1 25.
