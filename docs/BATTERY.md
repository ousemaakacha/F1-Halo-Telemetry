# Lettura automatica della capacità ERS

La versione 0.2.0 legge il valore nominale dai file installati di F1 25. Il
protocollo UDP EA fornisce l'energia residua in Joule, ma non la capacità massima:
`ersMaxHarvestedEnergyPerLap` è un limite per giro e non una capacità batteria.

La sorgente è il percorso BXML esatto
`VTF/EnergyRecoverySystem/Common/EnergyStoreCapacity`, attributo `value`, nelle
risorse `.../handling/*.vtf` degli archivi:

```text
<F1 25>/2025_asset_groups/f1_2025_vehicle_package/teams/common.erp
<F1 25>/2025_asset_groups/f1_2025_vehicle_package/teams/<team>/wep/<team>.erp
<F1 25>/2025_asset_groups/f1_2026_vehicle_package/teams/common.erp
<F1 25>/2025_asset_groups/f1_2026_vehicle_package/teams/<team>/wep/<team>.erp
```

Gli indici ERPK v3/v4 sono verificati prima di leggere i frammenti memorizzati o
Zstandard. La dimensione di una risorsa e la finestra Zstandard sono limitate a
2 MiB; l'indice a 8 MiB. Il parser controlla lunghezze, offset, annidamento e valori
non finiti. Non restituisce campi omonimi appartenenti ad upgrade o altre sezioni.
Nessun contenuto EA è incluso nei sorgenti o nei test.

La categoria Session formula 0 seleziona F1 2025 e formula 13 seleziona F1 2026.
Prima dell'arrivo di Session, `regulations2026` di Car Telemetry 2 permette di
selezionare F1 2026. Altre categorie non ricevono una capacità per supposizione.
Il valore viene usato solo quando le risorse controllate della stagione sono
coerenti. File illeggibili o capacità nominali diverse tra vetture lasciano
la capacità sconosciuta. Il display continua a mostrare l'energia ricevuta.

La ricerca considera `F1_25_DIR`, il percorso del processo F1_25.exe tramite
`PROCESS_QUERY_LIMITED_INFORMATION`, il registro Steam, le librerie Steam e le
cartelle standard EA Games. Non legge memoria, non inietta codice, non cambia
file del gioco. I valori vengono memorizzati sul worker UDP e ricaricati al cambio
sorgente, al cambio sessione o a un riavvio rilevato del flusso.

## Verifica riproducibile

Dopo `scripts/Build.ps1 -Examples`, eseguire:

```powershell
.\target\release\examples\game_battery.exe
# In alternativa, indicare direttamente una cartella installazione:
.\target\release\examples\game_battery.exe 'D:\Games\F1 25'
```

Il risultato JSON contiene percorso, risorsa, Joule ed eventuali errori. La lettura
locale del 5 novembre 2026 ha restituito 4 000 000 J sia per il 2025 sia per il
2026 senza errori. La denominazione nominale è intenzionale: la lettura statica
non misura riduzioni runtime da usura, R&D o moltiplicatori applicati in memoria.

Il vecchio campo JSON `ers_capacity_joules` viene ignorato e non viene più salvato.
La Demo dichiara una sorgente `demo` separata, senza attribuire i propri valori
ai file del gioco. Percentuale = energia UDP / capacità nominale × 100, limitata
all'intervallo 0–100.

Riferimenti di formato e provenienza:

- [Specifica UDP ufficiale EA](https://forums.ea.com/blog/f1-games-game-info-hub-en/ea-sports%E2%84%A2-f1%C2%AE25-2026-season-pack-udp-specification/12187347)
- [Documentazione ERPK Swofty](https://f1.swofty.net/reference/erp.html)
- [Documentazione dei parametri handling Swofty](https://f1.swofty.net/std/systems/car-physics.html)
- [Parser di riferimento ERPK/BXML, sorgenti primari](https://github.com/Swofty-Developments/SwoftyLauncher)

Il lettore dell'app è ristretto alla capacità ERS, con test sintetici indipendenti.
Non usa i componenti di modifica, hook o launcher dei progetti di riferimento.
