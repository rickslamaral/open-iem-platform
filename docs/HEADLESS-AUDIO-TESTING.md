# Testes de áudio headless

Open IEM possui dois níveis de teste sem desktop e sem hardware físico.

## Backend determinístico

`audio-engine` usa `SimulatedBackend` e injeta seno de 440 Hz em 48 kHz no mesmo pipeline DSP usado pelos testes de mixagem. Valida processamento, buffers, ganho, pan, mute, limiter, frames e ausência de XRUN.

```bash
cargo test --manifest-path server/Cargo.toml \
  --package audio-engine --test deterministic_harness
```

## ALSA Loopback

Quando o kernel permite `snd-aloop`, o script injeta WAV PCM real e captura o retorno:

```text
reprodução: plughw:Loopback,0,0
captura:    plughw:Loopback,1,0
formato:    S16_LE, mono, 48000 Hz
sinal:      seno 440 Hz
```

O número da placa nunca é fixado. O script descobre o dispositivo por nome com `aplay -l` e `arecord -l`.

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends alsa-utils
sudo modprobe snd-aloop
scripts/ci/run-headless-audio.sh
```

Se `snd-aloop` não estiver disponível, o teste não finge ALSA: executa o backend DSP determinístico e reporta `alsa_loopback=unavailable`. Isso mantém release headless sem declarar ALSA físico validado.

## Um comando

```bash
make audio-test
```

O comando:

1. Executa DSP determinístico.
2. Tenta carregar `snd-aloop`.
3. Lista dispositivos ALSA.
4. Gera seno PCM conhecido.
5. Reproduz e captura pelo Loopback.
6. Valida frames, sample rate, canais, sample width e não-silêncio.
7. Usa `timeout`.
8. Remove WAVs e encerra processos via `trap`.

## Instalação com relatório

O instalador registra identificação completa do host antes dos gates:

```text
os=Debian GNU/Linux 13
os_id=debian
os_version=13
architecture=aarch64
kernel=Linux ...
rustc=rustc 1.88.x
cargo=cargo 1.88.x
node=v22.x
npm=10.x
python=Python 3.x
alsa_utils=1.2.x
```

No fim, imprime resultado inequívoco:

```text
summary=ALL_TESTS_PASSED
tests_total=17
tests_passed=17
tests_failed=0
status=PASS
```

Ou, em falha:

```text
summary=TESTS_FAILED
tests_total=16
tests_passed=15
tests_failed=1
status=FAIL
```

O processo retorna código `0` somente com todos os testes aprovados.

Depois de instalar uma revisão imutável:

```bash
sudo bash scripts/install.sh \
  --ref <COMMIT-SHA-40-CHARS> \
  --run-tests \
  --test-report /tmp/openiem-tests.txt
```

O relatório cobre:

- sintaxe shell;
- `git diff --check`;
- `cargo fmt --check`;
- Clippy;
- testes Rust unitários;
- testes de integração API;
- DSP e áudio headless;
- testes Musician;
- testes Engineer.

Cada gate recebe `PASS` ou `FAIL`. Falhas não são suprimidas. Os testes seguintes continuam para gerar diagnóstico completo, e o instalador termina com código diferente de zero se qualquer gate falhar.

Nunca compartilhe relatório contendo segredo. O instalador não imprime senha SoundTech, JWT, tokens ou conteúdo de chaves.

## Docker

Docker pode executar o teste ALSA somente quando `/dev/snd` e o módulo vierem do host. Container não cria módulo de kernel:

```bash
sudo modprobe snd-aloop
make alsa-sim-build
make alsa-sim-test
```

Para CI sem privilégios de kernel, use `make audio-test`: o backend determinístico roda sem `/dev/snd`. Docker não substitui validação física de Raspberry Pi, USB audio, PipeWire ou latência.

## USB real

Teste físico separado:

```bash
aplay -l
arecord -l
```

Use nome/dispositivo descoberto no host. Validação USB, PipeWire, WebRTC/Opus, XRUN, latência e Raspberry Pi continuam gates de runtime/hardware. Backend determinístico e ALSA Loopback não autorizam claim de hardware real.

## PipeWire/WirePlumber userspace smoke

CI pode executar `scripts/ci/run-pipewire-software-e2e.sh` após instalar `pipewire`, `pipewire-bin` e `wireplumber`. O script inicia daemons em `XDG_RUNTIME_DIR` temporário, verifica o grafo userspace com `pw-cli` e limpa processos/arquivos ao sair.

Resultado `PIPEWIRE_SOFTWARE_E2E: PASS` significa somente `SOFTWARE/SIMULATED`: não valida dispositivo físico, backend nativo Open IEM, WebRTC sobre rede, latência, Opus em runtime ou Raspberry Pi.
