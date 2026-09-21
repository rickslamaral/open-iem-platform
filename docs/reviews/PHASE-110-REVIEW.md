# Phase 110 Review — late_packets counter for ReceiverMetrics

## Escopo
Campo `late_packets` adicionado a `ReceiverMetrics` e `ReceiverSnapshot` para separar pacotes descartados por chegada tardia ou duplicação dos descartados por outros motivos (overflow de ingress, payload inválido). O caminho de playout stale em `OpusReceiver` chama `record_late()` em vez de `record_dropped()`.

## Implementação
- `ReceiverMetrics::record_late()` adicionado
- Campo `late_packets: AtomicU64` adicionado a `ReceiverMetrics` com campo de snapshot `late_packets: u64`
- `OpusReceiver::playout` stale-packet path chama `record_late()` em vez de `record_dropped()` — separação semântica dos contadores
- `reset()` inclui `late_packets.store(0)` para limpeza atômica completa
- `snapshot()` inclui `late_packets` via `Acquire` load, consistente com os demais contadores
- Teste unitário `metrics_record_late_on_stale_packet` verifica contagem única no caminho stale
- Testes de integração REST em `api-server/tests/integration.rs` cobrem `output_failures` e `late_packets` expostos no endpoint `GET /api/v1/metrics`

## Evidência
- Implementação registrada em `d937b7a` (`[verified] feat(Phase110): add late_packets counter to ReceiverMetrics; stale/duplicate packets tracked separately from drops`).
- Gates listados no commit original; não reexecutados neste ciclo documental.

Evidência permanece CODE. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 não foram validados.

## Separação semântica dos contadores
| Contador | Significado |
|---|---|
| `packets_dropped` | Overflow de ingress, payload inválido |
| `late_packets` | Pacote válido chegou fora de ordem/duplicado |
| `output_failures` | Falhas que travaram mute fail-safe |

## Pendências
- Validar métricas em runtime WebRTC/DTLS-SRTP real.
- Validar em hardware PipeWire/ALSA e Raspberry Pi 5.
- Integrar ao binário headless receiver quando output OS for implementado.
