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

Baixe installer, revise ou execute:

```bash
curl -fsSL https://raw.githubusercontent.com/rickslamaral/open-iem-platform/main/scripts/install.sh \
  -o install.sh

bash install.sh \
  --ref 66161c5fffa023018e095fdf154537e8708a9718
```

Use sempre SHA completo. Não instalar branch ou tag mutável.

Build nativo em Raspberry Pi 3B+ pode demorar e falhar por falta de RAM. Se isso ocorrer, compile para ARMv7 em host Linux mais potente e valide execução no Pi separadamente.

## 3. Reexecução segura

Installer identifica release íntegra da mesma SHA antes de clonar, instalar dependências ou recompilar:

```text
release <SHA> already installed and valid; nothing to do
```

Release parcial não é aceita como válida. Installer tenta concluir instalação sem substituir release anterior durante falha de build.

## 4. Validação pós-instalação

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

## 5. Validação rápida sem instalar

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
