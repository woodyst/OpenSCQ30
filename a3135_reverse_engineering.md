# A3135 (Soundcore Motion 300) Reverse Engineering Progress

## Device Info
- Model: Soundcore Motion 300
- Internal code: A3135
- RFCOMM UUID: `0cf12d31-fac3-4553-bd80-d6832e7b3135`
- RFCOMM channel: 11
- Firmware: 4.0.4
- Serial: ACCLVH2F34202893

## ADB WiFi
- Port changes every time ADB root restarts. Ask user for current port.
- Root ADB: `adb root` (then reconnect — port changes)
- Pull full log: `adb -s 192.168.1.160:<PORT> pull /data/misc/bluetooth/logs/btsnoop_hci.log /tmp/btsnoop_new.log`
- Parse all: `python3 /tmp/analyze_a3135.py /tmp/btsnoop_new.log`
- Compare: `python3 /tmp/analyze_a3135.py /tmp/btsnoop_new.log /tmp/btsnoop_old.log`
- DO NOT truncate/clear the log — parser uses the last state packet automatically

## Soundcore Packet Format
```
TX (phone → device): 08 EE 00 00 [seq] [cmd0 cmd1] [total_lo total_hi] [body...] [checksum]
RX (device → phone): 09 FF 00 00 [seq] [cmd0 cmd1] [total_lo total_hi] [body...] [checksum]
```
- `checksum = 0` (ChecksumKind::None for A3135)
- `total` = total packet length including all header bytes

---

## ✅ FULLY MAPPED: State Response Body (cmd [01 01], 29 bytes)

```
[0]       volume        — scale TBD (state: 0x0E=14; cmd uses 0-100 percent?)
[1]       battery       — scale 0-5 (5 = 100%)
[2]       unknown       — always 0x00 in all captures
[3]       unknown       — always 0x00 in all captures
[4]       unknown       — always 0x01 in all captures (version/capability flag?)
[5]       unknown       — NOT sound mode (always 0x00 in both LDAC and Combine captures)
[6]       unknown       — always 0x03 in all captures (version/hardware flag?)
[7..12]   firmware      — ASCII "X.Y.Z" (5 bytes)
[12..28]  serial        — ASCII 16 bytes
[28]      unknown       — always 0x77 in all captures
```

### Captured state body samples:
```
LDAC mode:    0E 05 00 00 01 00 03 34 2E 30 2E 34 41 43 43 4C 56 48 32 46 33 34 32 30 32 38 39 33 77
Combine mode: 0E 05 00 00 01 01 03 34 2E 30 2E 34 41 43 43 4C 56 48 32 46 33 34 32 30 32 38 39 33 77
```

---

## ✅ FULLY MAPPED: Commands

### State / info queries
| TX cmd hex | TX cmd dec | Body  | RX body | Purpose |
|------------|-----------|-------|---------|---------|
| [01 01]    | [1, 1]    | empty | 29-byte state | Request full state |
| [02 89]    | [2, 137]  | empty | 57 bytes | Get custom EQ state (3 presets × 18 bytes) |
| [05 01]    | [5, 1]    | empty | 68 bytes | Get device capabilities |
| [10 93]    | [16, 147] | empty | 1 byte  | Get current volume/brightness? |

### Brightness — `CMD [10 92]` / `[16, 146]`
| TX body | Value |
|---------|-------|
| `00`    | Off   |
| `14`    | Low   |
| `46`    | Medium|
| `64`    | High  |

### Auto power-off — `CMD [01 86]` / `[1, 134]`
| TX body | State         |
|---------|---------------|
| `00 xx` | OFF (keeps last timer in xx) |
| `01 00` | ON, 5 minutes  |
| `01 01` | ON, 10 minutes |
| `01 02` | ON, 20 minutes |
| `01 03` | ON, 60 minutes |

### Voice prompts — `CMD [01 90]` / `[1, 144]`
| TX body | State |
|---------|-------|
| `00`    | OFF   |
| `01`    | ON    |

### Sound mode — `CMD [01 FF]` / `[1, 255]`
| TX body | State                        |
|---------|------------------------------|
| `00`    | Combine (quality+connection) |
| `01`    | LDAC preferred               |
Note: LDAC state is NOT in state body. Read via CMD [01 7F]: 0x00=Combine, 0x01=LDAC

### Adaptive direction — `CMD [02 8A]` / `[2, 138]` + `CMD [02 8C]` / `[2, 140]`
| TX cmd   | TX body | Purpose |
|----------|---------|---------|
| [2, 138] | `00`    | Adaptive direction OFF |
| [2, 138] | `01`    | Adaptive direction ON  |
| [2, 140] | empty   | Apply/confirm (always send after [2, 138]) |
Note: RX [2, 140] returns `00` (success) or `02` (error?)

### EQ preset — `CMD [02 8B]` / `[2, 139]`
| TX body | Preset           |
|---------|-----------------|
| `00`    | Balanced         |
| `01`    | Extra bass       |
| `02`    | Voice            |
| `03`    | Soundcore sig.   |
| `FE`    | Custom EQ        |

### Custom EQ — `CMD [02 8D]` / `[2, 141]`
TX body (21 bytes): `[profile_id] 01 FF [band0_amp] [band0_type] [band1_amp] [band1_type] ... [band8_amp] [band8_type]`

**Profile IDs** (byte[0]):
| Value | Profile  |
|-------|----------|
| `07`  | Custom   |
| `01`  | Custom2  |
| `02`  | Custom3  |
Max 3 custom profiles (matches 57-byte GET response = 3 × 19 bytes).

**Band layout** (bytes [3..20], pairs of amplitude+type):
| bytes | band | freq  |
|-------|------|-------|
| [3,4] | 0    | 80 Hz |
| [5,6] | 1    | ?     |
| [7,8] | 2    | ?     |
| [9,10]| 3    | ?     |
|[11,12]| 4    | ?     |
|[13,14]| 5    | ?     |
|[15,16]| 6    | ?     |
|[17,18]| 7    | ?     |
|[19,20]| 8    | 13 kHz|

**Amplitude**: `0x3C`=min, `0x78`=neutral(0dB), `0xB4`=max
**Type**: `0x07`=standard, `0x06`=band7, `0x01`=band8(last)
byte[1] = always `0x01`, byte[2] = always `0xFF`

### GET Custom EQ — `CMD [02 89]` / `[2, 137]`
RX body: 57 bytes = 3 profiles × 19 bytes each
Each 19-byte profile: `[profile_id] [band0_amp] [band0_freq] ... [band8_amp] [band8_freq]`

### Parametric EQ — frequency codes per band
Frequency formula: `f = min_hz × (max_hz / min_hz) ^ (code / max_code)`

| Band | Min Hz | Max Hz | Codes     | Default (simple mode)     |
|------|--------|--------|-----------|---------------------------|
| 0    | 48     | 89     | 0x00–0x09 | 0x07 ≈ 80 Hz              |
| 1    | 95     | 177    | 0x00–0x09 | 0x07                      |
| 2    | 190    | 353    | 0x00–0x09 | 0x07                      |
| 3    | 379    | 706    | 0x00–0x09 | 0x07                      |
| 4    | 759    | 1400   | 0x00–0x09 | 0x07                      |
| 5    | 1500   | 2800   | 0x00–0x09 | 0x07                      |
| 6    | 3000   | 5600   | 0x00–0x09 | 0x07                      |
| 7    | 6000   | 11300  | 0x00–0x09 | 0x06                      |
| 8    | 12100  | 20000  | 0x00–0x07 | 0x01 ≈ 13 kHz             |

Simple mode freq codes (fixed): `[0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x06, 0x01]`
Amplitude: 0x00 (min, 0x3C = –6 dB), 0x78 = 0 dB, 0xB4 = +6 dB (each unit = 0.1 dB)

### Volume — `CMD [01 88]` / `[1, 136]`
| TX body | Value |
|---------|-------|
| `00`    | Minimum (0) |
| `1F`    | Maximum (31) |
- Scale: 0–31 (matches max_vol=31 from device capabilities)
- state body[0] = current volume in same scale
- CMD [16, 146] is BRIGHTNESS only (not volume)

### Other commands (init sequence, purpose unknown)
| TX cmd   | Notes |
|----------|-------|
| [01 A6]  | body 4 bytes (session key, changes each reconnect) |
| [01 91]  | body `00` |
| [10 8A]  | empty; RX `01 00` or `00 00` |
| [05 81]  | empty; no RX |
| [01 21]  | empty; RX `01` or `00` |
| [01 7F]  | empty; RX `00` or `01` |

---

## ✅ IMPLEMENTED in OpenSCQ30

- Battery level (0-5 scale) → body[1]
- Firmware version → body[7..12]
- Serial number → body[12..28]
- Volume (0-31) → body[0], SET `CMD [01 88]`
- Sound mode / LDAC → body[5] (0x00=LDAC, 0x01=Combine), SET `CMD [01 FF]`
- Voice prompts → SET `CMD [01 90]` (initial state unknown, defaults to off)
- Auto power-off → SET `CMD [01 86]` (initial state unknown, defaults to disabled)
- Power off action → `CMD [01 89]`

## 🔲 TO IMPLEMENT

Priority order for implementation:

1. **Brightness** (read+write) — `CMD [10 92]` SET, `[10 93]` GET
   - Values: Off=0x00, Low=0x14, Medium=0x46, High=0x64
   - NOT in state body (must use GET command to read current)

2. **Auto power-off** (read+write) — `CMD [01 86]`
   - Body: `[enabled_byte, timer_index]`
   - NOT in state body

3. **Voice prompts** (read+write) — `CMD [01 90]`
   - Body: `00`=OFF, `01`=ON
   - NOT in state body

4. **Sound mode** (read+write) — `CMD [01 FF]`
   - Body (SET): `00`=Combine, `01`=LDAC
   - State body[5]: `0x00`=LDAC, `0x01`=Combine

5. **Adaptive direction** (read+write) — `CMD [02 8A]` + `[02 8C]`
   - Body: `00`=OFF, `01`=ON; always follow with CMD [02 8C] empty
   - NOT in state body (but body[4] is suspicious at 0x01)

6. **EQ preset** (read+write) — `CMD [02 8B]`
   - Body: 0x00-0x03=presets, 0xFE=custom
   - NOT confirmed in state body (body[6]=3 always, but may not be EQ)

7. **Custom EQ** (read+write) — `CMD [02 89]` GET, `[02 8D]` SET

8. **Volume** (read+write) — TBD (possibly `CMD [10 92]` but may conflict with brightness)

## Adaptive Direction — device feedback (orientation events)
Device DOES send unsolicited notifications via `CMD [2, 140]`:
| RX body | Meaning |
|---------|---------|
| `00`    | Normal orientation (also: success response to SET command) |
| `02`    | Hanging/colgando orientation detected |
Note: device only sends notification when ENTERING hanging orientation; returning to normal sends nothing.

## GET Custom EQ response — byte[2] = active preset
`CMD [02 89]` RX body byte[2] = current EQ preset index (0x00=Balanced, 0x03=Soundcore Sig, etc.).
This is how to read the current EQ preset on connect.

## CMD [01 7F] — LDAC state query
RX body: 2 bytes `[ldac_state, unknown]`
- `0x00 xx` = Combine mode active
- `0x01 xx` = LDAC mode active
Note: body[1] = `0x95` seen in captures (purpose unknown)

### Power off — `CMD [01 89]` / `[1, 137]`
TX body: empty
RX body: empty (device powers off immediately)

## Still Unknown
- body[2], body[3], body[4], body[6], body[28] — likely hardware/version flags, not user settings
- Play/pause command

## Analysis Scripts
- `/tmp/analyze_a3135.py` — parse btsnoop, show state packets and TX commands
- `/tmp/debug_bt.py` — diagnostic tool for btsnoop format issues
