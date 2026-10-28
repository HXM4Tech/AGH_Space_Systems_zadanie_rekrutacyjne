# Rozwiązanie zadania rekrutacyjnego AGH Space Systems Rocket Software jesień 2026

Kompletne rozwiązanie zadania rekrutacyjnego (Rocket Software, Jesień 2026).

## Wygląd panelu kontroli misji
<img width="1663" height="995" alt="image" src="https://github.com/user-attachments/assets/39ff243b-fd0c-4433-a918-df062f7c0d19" />

## Architektura

Projekt łączy warstwę niskopoziomową w Rust (PyO3, maturin) z interaktywnym panelem kontroli misji w Pythonie (NiceGUI)

```text
├── README.md               # Ten plik
├── requirements.txt        # Wymagane biblioteki Pythona
├── pyproject.toml          # Konfiguracja budowania biblioteki Pythona z rocket_ctl_rust/
├── rocket_ctl_rust/        # Warstwa Rust (PyO3) – ramki, stan rakiety, klient TCP
│   ├── Cargo.toml          # Manifest warstwy Rust
│   ├── src/
│   │   ├── lib.rs          # Eksport klas/funkcji do Pythona
│   │   ├── constants.rs    # Definicje stałych
│   │   ├── checksum.rs     # Liczenie sumy kontrolnej CRC-32
│   │   ├── frame.rs        # Parsowanie/budowanie ramek protokołu AGH Space Systems
│   │   ├── client.rs       # Klient TCP
│   │   ├── state.rs        # Model stanu rakiety (telemetria, checklista, itd)
│   │   ├── manager.rs      # Sterowanie rakietą (zintegorwane z Pythonem)
│   │   ├── autopilot.rs    # Autopilot (uruchamiany z Pythona)
├── web_ui/                 # Warstwa Python - panel kontroli misji (NiceGUI)
│   ├── main.py             # Punkt wejścia
│   └── components.py       # Wygląd i działanie UI
├── build.sh                # Skrypt do przygotowania venv Pythona i kompilacji warstwy Rust
├── launch.sh               # Skrypt do uruchamiania całości projektu
│
├── README.orig.md                     # Oryginalny plik README z poleceniem
├── communication_library/             # --- BEZ ZMIAN ---
├── tcp_proxy.py                       # --- BEZ ZMIAN ---
├── tcp_simulator.py                   # --- BEZ ZMIAN ---
├── frame_sending_receiving_example.py # --- BEZ ZMIAN ---
└── simulator_config.yaml              # --- BEZ ZMIAN ---
```

## Uruchomienie

### 1. Prygotuj środowisko wirtualne Pythona i zbuduj projekt
```bash
./build.sh
```

### 2. Uruchom całość
```bash
./launch.sh
```

Panel dostępny pod adresem: [http://localhost:8080](http://localhost:8080)

## Funkcje

- **Pełna integracja z protokołem** – parsowanie ramek, budowanie komend (servo/relay), obsługa ACK/FEED/NACK
- **Cały stan i klient w Rust** – `RocketClient` utrzymuje połączenie TCP, parsuje ramki i aktualizuje stan rakiety przechowywany w `RocketState`. Metody `send_servo/send_relay` są wykorzystywane przez wyeksportowany do Pythona `RocketManager` dostarczający proste metody sterowania, którymi może również zarządzać `Autopilot`
- **Mission Control UI** – przejrzysty panel NiceGUI z telemetrią, statusami, checklistą misji
- **Blokady operacyjne (interlocki)** – kontrola kolejności tankowania, okna ciśnienia dla zapłonu, warunki bezpiecznego wyrzutu spadochronu
- **Tryb manualny i autopilot** – pełna sekwencja automatyczna (tankowanie, podgrzewanie, zapłon, apogeum, wyrzut spadochronu, lądowanie)
- **Śledzenie lotu** – automatyczne wykrywanie apogeum, faz lotu, detekcja lądowania. Po udanym lądowaniu (z otwartym spadochronem) wysokość jest resetowa do 0

## Procedura lotu (orientacyjna)

1. **Tankowanie utleniacza** – `Refuel Oxidizer` → poczekaj na 100% → `Close Oxidizer`
2. **Tankowanie paliwa** – dostępne po napełnieniu utleniacza → `Refuel Fuel` → poczekaj na 100% → `Close Fuel`
3. **Podgrzewanie** – `Heater ON` aż ciśnienie osiągnie 55–65 bar → `Heater OFF`
4. **Zapłon** – przy pełnych zbiornikach, zamkniętych zaworach tankowania i ciśnieniu 55–65 bar aktywny przycisk `START (IGNITION SEQUENCE)`
5. **Spadochron** – dostępny po wypaleniu silnika, przy bezpiecznej prędkości i w fazie opadania
6. **Autopilot** – `EXECUTE AUTO-MISSION` przeprowadza całość automatycznie
