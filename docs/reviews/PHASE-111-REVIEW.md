# Phase 111 Review — decoder/output failure metrics

## Escopo

Documentar a extensão de `ReceiverMetrics.output_failures` e sua cobertura nos caminhos de falha do `OpusReceiver`: decode normal, PLC, frame PCM inválido, exaustão do orçamento PLC e erro de escrita no output. O latch de mute fail-safe impede contagem duplicada após a primeira transição terminal.

## Implementação

- `OpusReceiver` registra uma falha quando cada caminho entra em estado `output_failed`.
- Falhas posteriores enquanto receiver permanece mutado não incrementam contador novamente.
- O snapshot observability e `GET /api/v1/metrics` preservam campo estável `output_failures`.
- Testes unitários cobrem falha de decode, falha PLC, duração PCM inválida, exaustão PLC e erro de output, incluindo contagem única no latch.

## Evidência

- Implementação mergeada em `cd43881` (PR #208).
- Documentação/status mergeados em `a12bc7a` (PR #209).
- CI remoto do PR #209: 16 jobs concluídos com sucesso, incluindo Rust, frontends, segurança, cobertura, ALSA simulado, Audio Lab L1/L2 e gates `.deb`.
- Evidência permanece CODE/CI/SIMULATED. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA em target e Raspberry Pi 5 não foram validados.

## Pendências

- Integrar receiver ao binário headless com output OS.
- Validar métricas no runtime WebRTC/DTLS-SRTP real.
- Validar PipeWire/ALSA e comportamento em hardware Raspberry Pi.
