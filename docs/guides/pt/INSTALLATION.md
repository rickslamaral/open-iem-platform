# Instalação Linux

> Status: installer validado em Linux local. Raspberry Pi 3B+ ainda exige validação física. Não declarar áudio real, PipeWire, ALSA, WebRTC ou Opus validados sem hardware.

## Requisitos

- Linux Debian 13 ou equivalente.
- Raspberry Pi 3B+ com `armv7l`/`armhf`, ou host Linux compatível.
- Acesso root ou `sudo`.
- `rustc >= 1.88` — installer atualiza Rust antigo via `rustup`.
- Node.js `>= 20`.
- Memória e swap suficientes para compilar no Raspberry Pi 3B+.

Desktop Linux não muda procedimento. O installer usa terminal e instala `openiem-server.service` quando `systemd` está disponível.

## 1. Pré-validação do host

Execute antes da instalação:

```bash
uname -m
dpkg --print-architecture
free -h
swapon --show
rustc --version 2>/dev/null || true
node --version 2>/dev/null || true
npm --version 2>/dev/null || true
```

Esperado no Raspberry Pi 3B+:

```text
armv7l
armhf
```

Se não houver swap, adicione 2 GB antes do build:

```bash
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

Não use `--skip-deps` na primeira instalação. O installer precisa instalar ferramentas ausentes. Se Rust do Debian for `1.85.1`, installer detecta versão incompatível e usa Rust stable via `rustup`.

## 2. Instalação em Raspberry Pi ou Linux

Baixe installer, revise e execute com SHA completo da revisão desejada:

```bash
curl -fsSL https://raw.githubusercontent.com/rickslamaral/open-iem-platform/main/scripts/install.sh \
  -o install.sh
chmod 700 install.sh
less install.sh

export OPENIEM_REF=<COMMIT-SHA-40-CHARS>
sudo bash install.sh \
  --ref "$OPENIEM_REF" \
  --run-tests \
  --test-report /tmp/openiem-tests.txt
```

O installer preserva configuração existente, gera senha SoundTech aleatória quando necessário, não imprime segredos e instala `openiem-server.service`. Use sempre SHA completo. Não instalar branch ou tag mutável.

O `--run-tests` identifica OS, arquitetura, kernel e versões de `rustc`, `cargo`, Node.js, npm, Python e ALSA antes de executar os gates shell, Rust, API, DSP determinístico, ALSA Loopback quando disponível e testes das duas UIs. No fim, imprime `ALL TESTS PASSED` ou `TESTS FAILED`, com contagem total/aprovada/reprovada. Resultado fica em:

```bash
cat /tmp/openiem-tests.txt
```

Sucesso esperado:

```text
summary=ALL_TESTS_PASSED
tests_failed=0
status=PASS
```

Qualquer gate reprovado gera `status=FAIL` e código de saída diferente de zero.

Build nativo em Raspberry Pi pode demorar e consumir RAM. Em Pi com pouca memória, habilite swap antes do build. Cross-build não substitui execução no Pi.

## 3. Áudio headless e Raspberry Pi

Instale utilitários ALSA antes de testar Loopback:

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends alsa-utils
```

Tente carregar `snd-aloop` no kernel do host:

```bash
sudo modprobe snd-aloop
aplay -l
arecord -l
```

Execute a validação completa:

```bash
cd /caminho/open-iem-platform
make audio-test
```

O teste injeta seno de 440 Hz e captura PCM em 48 kHz, mono, `S16_LE`. Descobre placa Loopback por nome; não assume `card 1`, `card 2` ou outro número. Valida frames capturados e rejeita captura silenciosa.

Se o kernel não oferecer `snd-aloop`, o backend DSP determinístico ainda passa sem hardware. Isso valida processamento e buffers, não ALSA físico.

Teste USB real separadamente:

```bash
aplay -l
arecord -l
sudo fuser -v /dev/snd/*
```

Não declarar PipeWire, ALSA USB, WebRTC/Opus ou Raspberry Pi validado apenas por `snd-aloop`, Docker ou backend simulado.

## 4. Reexecução segura

Installer identifica release íntegra da mesma SHA antes de clonar, instalar dependências ou recompilar:

```text
release <SHA> already installed and valid; nothing to do
```

Release parcial não é aceita como válida. Installer tenta concluir instalação sem substituir release anterior durante falha de build.

## 5. Validação pós-instalação

O modo `--validate-only` não compila, não instala, não altera serviço e não expõe chaves JWT. Ele grava relatório com arquitetura, versões, release e estado do serviço:

```bash
sudo bash install.sh \
  --validate-only \
  --ref 66161c5fffa023018e095fdf154537e8708a9718 \
  --validation-report /tmp/openiem-validation.txt

cat /tmp/openiem-validation.txt
```

Valide serviço:

```bash
systemctl status openiem-server.service --no-pager
journalctl -u openiem-server.service -n 80 --no-pager
```

Envie somente estes resultados para diagnóstico:

```bash
cat /tmp/openiem-validation.txt
systemctl status openiem-server.service --no-pager
journalctl -u openiem-server.service -n 80 --no-pager
```

Remova ou redija qualquer token, senha, chave JWT ou connection string antes de compartilhar logs.

## 6. Validação rápida sem instalar

Confirme comandos e fluxo sem alterar host:

```bash
bash install.sh \
  --dry-run \
  --skip-deps \
  --no-service \
  --ref 66161c5fffa023018e095fdf154537e8708a9718
```

Esperado:

```text
Rust >= 1.88
Node.js >= 20
dry-run complete; host unchanged
```

## Limites da validação

Os comandos acima validam instalação, binários, release, serviço e ambiente Linux. Não validam funcionamento de hardware de áudio. Para validar áudio real, registrar separadamente modelo da placa, kernel, dispositivos ALSA/PipeWire, latência, XRUNs e resultado reproduzível em hardware físico.
