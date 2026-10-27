# Rozwiązanie zadania rekrutacyjnego AGH Space Systems – Rocket Software (Jesień 2026)

Kompletne, modularne rozwiązanie zadania rekrutacyjnego:
1. **Moduł Rust (`rocket_comm_rust`):** Niskopoziomowe parsowanie, walidacja CRC-32/MPEG-2 i budowanie ramek binarnych zgodnie z protokołem AGH Space Systems przy użyciu `PyO3` oraz `Maturin`.
2. **Mission Control UI (`mission_control`):** Przejrzysty panel kontroli misji w Pythonie (`NiceGUI`) komunikujący się asynchronicznie po TCP z rakietą, wyposażony w system blokad operacyjnych (interlocks) i procedurę pełnego autopilota ("Auto-Mission").

---

## 1. Architektura projektu

Kod został zorganizowany w przejrzysty, modularny sposób:

```text
AGH_Space_Systems_zadanie_rekrutacyjne/
├── rocket_comm_rust/             # Moduł w języku RUST (PyO3 FFI)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Eksport funkcji PyO3 do Pythona
│       ├── frame.rs              # Zero-overhead bit shifts (Little-Endian u64)
│       └── calc_crc.rs           # Sprzętowy algorytm CRC-32/MPEG-2
├── mission_control/              # Panel kontroli misji (Python / NiceGUI)
│   ├── main.py                   # Główny punkt wejścia i rejestracja strony @ui.page
│   ├── state.py                  # Model stanu telemetrii i reguły bezpieczeństwa (Interlocks)
│   ├── control_client.py         # Asynchroniczny klient TCP (zabezpieczony przed rozłączeniem)
│   └── components.py             # Reaktywne komponenty pulpitu operatora
├── tcp_proxy.py                  # Serwer pośredniczący (proxy TCP)
├── tcp_simulator.py              # Symulator rakiety
└── simulator_config.yaml         # Konfiguracja urządzeń pokładowych
```

---

## 2. Instrukcja uruchomienia

### Krok 1: Aktywacja środowiska wirtualnego i budowa modułu Rust
```bash
source venv/bin/activate
maturin develop --manifest-path rocket_comm_rust/Cargo.toml
```

### Krok 2: Uruchomienie infrastruktury (w 3 osobnych terminalach)

1. **Terminal 1 (Proxy TCP):**
   ```bash
   source venv/bin/activate
   python tcp_proxy.py
   ```

2. **Terminal 2 (Symulator rakiety):**
   ```bash
   source venv/bin/activate
   python tcp_simulator.py
   ```

3. **Terminal 3 (Mission Control UI):**
   ```bash
   source venv/bin/activate
   python mission_control/main.py
   ```

Panel kontroli misji dostępny jest pod adresem:
👉 **[http://localhost:8080](http://localhost:8080)**

---

## 3. Przebieg procedury lotu

Interfejs dynamicznie pilnuje procedury i uniemożliwia wykonanie niebezpiecznych akcji (blokady logiczne / interlocks):
1. **Tankowanie utleniacza:** Otwórz zawór `Refuel Oxidizer` $\rightarrow$ poczekaj na 100% $\rightarrow$ `Close Oxidizer`.
2. **Tankowanie paliwa:** Przycisk `Refuel Fuel` odblokowuje się **dopiero po napełnieniu utleniacza do 100%**. Po napełnieniu zamknij `Close Fuel`.
3. **Podgrzewanie:** Załącz `Heater ON` $\rightarrow$ monitoruj ciśnienie utleniacza (optymalny zakres $55–65\text{ bar}$) $\rightarrow$ wyłącz `Heater OFF`.
4. **Zapłon (`START`):** Przycisk staje się aktywny tylko w zielonym oknie ciśnienia ($55–65\text{ bar}$) przy pełnych zbiornikach i zamkniętych zaworach tankowania. Po kliknięciu sekwencja otwiera oba zawory główne i odpala zapłonnik w oknie $< 200\text{ ms}$.
5. **Wyrzut spadochronu (`DEPLOY PARACHUTE`):** Zablokowany podczas pracy silnika i przy prędkości opadania $> 28\text{ m/s}$. Pulsuje na zielono po osiągnięciu apogeum.
6. **Autopilot (`EXECUTE AUTO-MISSION`):** Przeprowadza cały powyższy proces w 100% automatycznie.
