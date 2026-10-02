## 2026-10-02 16:27 -0300 — verificação operacional no HEAD `86fd769`

- `develop` e `origin/develop` sincronizadas no commit `86fd7690a194da77e9814e4fd2c955a10076934d`; working tree estava limpa antes desta atualização documental. Lease `.git/hermes-dev.lock` presente e vazio; validade não inferida.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Observação desta execução após `git fetch --prune`: nenhuma branch remota órfã além das referências canônicas; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Gates desta atualização: `scripts/validate-docs.sh`, `git diff --check` e `cargo fmt --all --manifest-path server/Cargo.toml -- --check` PASS; scanner `/root/scan_patterns.py` indisponível nesta execução; verificação manual de linhas adicionadas não encontrou segredos ou padrões perigosos.
- Observação desta execução: CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 15:37 -0300 — verificação operacional no HEAD `e31d7ee`

- `develop` e `origin/develop` sincronizadas no commit `e31d7eeecb0fd32fb880f3adc80401e1d9972c2e`; working tree limpa antes desta atualização. Lease `.git/hermes-dev.lock` presente, vazio, validade não inferida.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada após `git fetch --prune`: nenhuma branch remota órfã além das referências canônicas; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em Rust/coverage; política vigente não altera essas branches.
- Gates desta atualização: `scripts/validate-docs.sh`, `git diff --check` e `cargo fmt --all --manifest-path server/Cargo.toml -- --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 13:36 -0300 — verificação operacional no HEAD `263ec74`

- `develop` e `origin/develop` sincronizadas no HEAD `263ec746be6cad0dca6e31e8bbf4b5623edc6e39`; working tree limpa antes desta atualização. Lease `.git/hermes-dev.lock` presente; validade não inferida.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada após `git fetch --prune`: nenhuma branch remota órfã além de referências canônicas; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Gates desta atualização: `scripts/validate-docs.sh` PASS e `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED BY POLICY / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 13:31 -0300 — verificação operacional no HEAD `5034a89`

- `develop` e `origin/develop` sincronizadas no HEAD `5034a8907a2ab37eae9b9db5e9438c014852d66e`; working tree limpa antes desta atualização. Lease `.git/hermes-dev.lock` presente; validade não inferida.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada após `git fetch --prune`: nenhuma branch remota órfã além de referências canônicas; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Gates desta atualização: `scripts/validate-docs.sh` PASS e `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED BY POLICY / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 13:26 -0300 — verificação operacional no HEAD `c3e6a11`

- `develop` e `origin/develop` sincronizadas no HEAD `c3e6a11`; working tree limpa antes desta atualização. Observado localmente. Arquivo `.git/hermes-dev.lock` presente; validade do lease não inferida.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; nenhum branch local mergeado pendente. PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Gates desta atualização: `scripts/validate-docs.sh` PASS, `git diff --check` PASS e `cargo fmt --all --manifest-path server/Cargo.toml -- --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais em `develop` (runs `36897066547` e `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED BY POLICY / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 13:16 -0300 — verificação operacional no HEAD `28c1618`

- `develop` e `origin/develop` sincronizadas no HEAD `28c16182bbdd9b1c9864459e756f5767bfdf28bc`; working tree limpa. Lease `.git/hermes-dev.lock` presente.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais em `develop` (runs `36897066547` e `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 13:06 -0300 — verificação operacional no HEAD `38c0895`

- `develop` e `origin/develop` sincronizadas no HEAD `38c0895c4a8084b11c3bf0f1b4cf1e5590a5ba06`; working tree limpa. Lease `.git/hermes-dev.lock` presente.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`. Política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais em `develop` (runs `36897066547` e `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:41 -0300 — verificação operacional no HEAD `2c758d8`

- `develop` e `origin/develop` sincronizadas; working tree limpa antes desta atualização documental. Lease `.git/hermes-dev.lock` presente.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`. Política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais em `develop` (runs `36897066547` e `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:36 -0300 — verificação operacional no HEAD `013cce4`

- `develop` e `origin/develop` sincronizadas; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`. Política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais em `develop` (runs `36897066547` e `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:26 -0300 — verificação operacional no HEAD `f5c2520`

- Lease `.git/hermes-dev.lock` existe; `develop` e `origin/develop` sincronizadas no HEAD `f5c2520`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` serão executados nesta atualização. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:21 -0300 — verificação operacional no HEAD `843ff73`

- Lease `.git/hermes-dev.lock` existe; `develop` e `origin/develop` sincronizadas no HEAD `843ff73`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:16 -0300 — verificação operacional no HEAD `c25858c`

- Lease `.git/hermes-dev.lock` existe; `develop` e `origin/develop` sincronizadas no commit `c25858c7777d978c82cb7262434ec9b90b83a5e9`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` serão executados nesta atualização; scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:11 -0300 — verificação operacional no HEAD `4bef9f3`

- Lease `.git/hermes-dev.lock` existe; `develop` e `origin/develop` sincronizadas no commit `4bef9f3cd50008bb318e19ba0bf2f76f8da73be9`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` PASS nesta atualização. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 12:00 -0300 — verificação operacional no HEAD `c807a84`

- Lease `.git/hermes-dev.lock` existe; `develop` e `origin/develop` sincronizadas no commit `c807a841b4a909873474c1ffb44d7a3f1c3c55f1`; working tree limpa.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust Format + Clippy + Tests e Rust Code Coverage; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 11:56 -0300 — verificação operacional no HEAD `94e9b4c`

- Lease `.git/hermes-dev.lock` existe; `develop` e `origin/develop` sincronizadas no commit `94e9b4c8ba45496e7e7c56ed714797f2239ceafb`; working tree limpa.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust Format + Clippy + Tests e Rust Code Coverage; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` executados nesta atualização: PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual. Últimos SUCCESS reais (`36897066547`, `36897066543`) cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 11:46 -0300 — verificação operacional no HEAD ef9b491

- develop e origin/develop sincronizadas no HEAD ef9b491; working tree limpa antes desta atualização. O lease `.git/hermes-dev.lock` existe; execução atual mantém esse lease ativo.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- PRs Dependabot #347 e #348 continuam abertas contra main, ambas com falhas em Rust Format + Clippy + Tests e Rust Code Coverage; política vigente não altera essas branches.
- Gates desta atualização: scripts/validate-docs.sh PASS e git diff --check PASS; scanner /root/scan_patterns.py permanece indisponível neste host.
- CI remoto não tem SUCCESS no HEAD atual; não foi inventada evidência. PHYSICAL: USER-APPROVED / NOT EXECUTED; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release v0.3.1 seguem PENDING/BLOCKED. Evidência CODE/CI/SIMULATED.

## 2026-10-02 11:31 -0300 — verificação operacional no HEAD `bd02cca`

- `develop` e `origin/develop` sincronizadas no HEAD `bd02cca` (`bd02cca1c681ef89e0794121d2ac9bdbe8879489`); working tree limpa antes desta atualização. Nenhum arquivo de lease/lock foi encontrado na inspeção do workspace.
- Limpeza observada: referências remotas listadas não incluem branches órfãs além de `origin/HEAD`; nenhum branch local adicional aparece mergeado em `main`; PRs Dependabot #347 e #348 continuam abertas contra `main`, com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Backlog consultado: itens `[ ]` restantes estão documentados como dependentes de hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa CODE executável foi selecionada neste ciclo.
- Gates desta atualização: `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`), `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 11:26 -0300 — verificação operacional no HEAD `0660889`

- `develop` e `origin/develop` sincronizadas no HEAD `0660889`; working tree limpa antes desta atualização. Lease do repositório: nenhum mecanismo de lock existe.
- Limpeza validada: branches remotas órfãs inexistentes; nenhum branch local mergeado pendente. PRs Dependabot #347 e #348 continuam abertas contra `main`, com falhas em Rust/coverage; política vigente não altera essas branches.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura identificada.
- Gates desta atualização: `scripts/validate-docs.sh` PASS e `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 11:16 -0300 — verificação operacional no HEAD `a764b2b`

- `develop` e `origin/develop` sincronizadas no HEAD `a764b2b`; working tree limpa antes desta atualização. Lease do repositório: nenhum mecanismo de lock existe.
- Limpeza validada: branches remotas órfãs inexistentes; nenhum branch local mergeado pendente. PRs Dependabot #347 e #348 continuam abertas contra `main`; falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura identificada.
- `git diff --check` PASS. `scripts/validate-docs.sh` e gates completos não foram rerun nesta atualização operacional; resultados anteriores permanecem evidência histórica, não validação nova. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 11:01 -0300 — operational verification at `369cca1`

- `develop` e `origin/develop` sincronizadas no HEAD `369cca1f754e21188b131feca0bff643d862118d`; working tree limpa antes desta atualização. Lease do repositório: nenhum mecanismo de lock existe.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi identificada.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, com falhas Rust/coverage; política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 10:50 -0300 — operational verification at `bf86c6e`

- `develop` e `origin/develop` sincronizadas no HEAD `bf86c6e`; working tree limpa antes desta atualização. Lease do repositório: nenhum mecanismo de lock existe.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi identificada.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, com falhas Rust/coverage; política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 10:48 -0300 — operational verification at `6ecc3f0`

- `develop` e `origin/develop` sincronizadas no HEAD `6ecc3f0`; working tree limpa antes desta atualização. Lease do repositório: nenhum mecanismo de lock existe.
- Backlog CODE executável permanece esgotado. Itens restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi identificada.
- Gates locais PASS: `scripts/validate-docs.sh`; `git diff --check`; `cargo fmt --all --manifest-path server/Cargo.toml -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (todos os testes passaram: 7 binários, 174 unitários, 124 integração, 537 streaming e doc-tests). Musician: typecheck, 67 testes, build. Engineer: typecheck, 59 testes, build.
- Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado. CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, com falhas Rust/coverage; política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 09:51 -0300 — operational verification at `cb5f294`

- `develop` and `origin/develop` synchronized at current HEAD; working tree clean before this documentation update. Repository lease: no lease/lock mechanism exists.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local checks PASS: `scripts/validate-docs.sh`; `git diff --check`.
- No remote CI SUCCESS covers current HEAD; latest real SUCCESS runs `36897066547` and `36897066543` cover prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. User policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 09:43 -0300 — operational verification at `4114e5e`

- `develop` and `origin/develop` synchronized at `4114e5eb38fdc33a3c57e87cfb42d33d5f499108`; working tree clean before this documentation update. Repository lease: no lease/lock mechanism exists.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local gates PASS: `scripts/validate-docs.sh`; `cargo fmt --all --manifest-path server/Cargo.toml -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (all tests passed: unit/integration/doc tests); musician typecheck, 67 tests, build; engineer typecheck, 59 tests, build.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. No remote CI SUCCESS covers current HEAD; latest real SUCCESS runs `36897066547` and `36897066543` cover prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. User policy leaves them untouched. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 09:06 -0300 — operational verification at `784936b`

- `develop` and `origin/develop` synchronized; working tree clean before this documentation update. Repository lease: no lease/lock mechanism exists.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local gates PASS: `scripts/validate-docs.sh`; `git diff --check`. Full Rust/frontend gates were not rerun in this cycle; prior recorded gates remain historical evidence only.
- Remote CI SUCCESS exists for prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4` only; no SUCCESS claimed for current HEAD.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. User policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 08:41 -0300 — operational verification at `a94bc4b`

- `develop` and `origin/develop` synchronized; working tree clean before this documentation update. Repository lease: no lease/lock mechanism exists.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local gates PASS: `scripts/validate-docs.sh`; `git diff --check`; `cargo fmt --all --manifest-path server/Cargo.toml -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (all tests passed).
- Remote CI SUCCESS exists for prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4` only; no SUCCESS claimed for current HEAD.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. Policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 08:35 -0300 — operational verification at `560ce39`

- `develop` and `origin/develop` synchronized at current HEAD; working tree clean before this documentation update.
- Repository lease: no lease/lock mechanism exists in workspace; branch and remote synchronization validated.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local gates PASS: `scripts/validate-docs.sh`; `git diff --check`; `cargo fmt --all --manifest-path server/Cargo.toml -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (all reported tests passed).
- Remote CI SUCCESS exists for prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4` only; no SUCCESS claimed for current HEAD.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. Policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 08:30 -0300 — operational verification at `824b37d`

- `develop` and `origin/develop` synchronized at `824b37db88f9ec048dfa40aef4df1fefddf1857b`; working tree clean before this documentation update.
- Repository lease: no lease/lock mechanism exists in workspace; branch and remote synchronization validated.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local gates PASS: `scripts/validate-docs.sh`; `git diff --check`; `cargo fmt --all --manifest-path server/Cargo.toml -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (all reported tests passed).
- Remote CI SUCCESS exists for prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4` only; no SUCCESS claimed for current HEAD `824b37db88f9ec048dfa40aef4df1fefddf1857b`.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. Policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 08:21 -0300 — operational verification at `d696f34`

- `develop` and `origin/develop` synchronized at `d696f34d46dc2baa8a9e137d1f80ff8d88e69004`; working tree clean before this documentation update.
- Repository lease: no lease/lock mechanism exists in workspace; branch and remote synchronization validated.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- Local gates PASS: `scripts/validate-docs.sh`; `git diff --check`; `cargo fmt --all --manifest-path server/Cargo.toml -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (all reported tests passed).
- Remote CI SUCCESS exists for prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4` only; no SUCCESS claimed for current HEAD `d696f34d46dc2baa8a9e137d1f80ff8d88e69004`.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. Policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 07:40 -0300 — operational verification at `5fe994b`

- `develop` and `origin/develop` synchronized at `5fe994b7053fdea3f815a29967c66c205f40306c`; working tree clean before this update.
- Backlog CODE executable remains exhausted; unchecked items require physical hardware, release confirmation, external secret, or remote runner access. No product task selected.
- Repository lease: no lease/lock mechanism exists in workspace; branch and remote synchronization validated before work.
- Open Dependabot PRs #347 and #348 target `main`; both fail Rust/coverage checks. Policy leaves them untouched.
- `gh run list --branch develop --limit 5` shows no CI SUCCESS for current HEAD; latest real SUCCESS runs `36897066547` and `36897066543` target prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); `git diff --check` PASS. Static scanner `/root/scan_patterns.py` unavailable; no fabricated result.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; runtime WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 07:30 -0300 — operational verification at `c8d35bc`

- `develop` and `origin/develop` synchronized at `c8d35bcf457226e4f2e69aaa12d9761468c4e3d3`; working tree clean before this update.
- Backlog CODE executable remains exhausted; unchecked items require physical hardware, release confirmation, external secret, or remote runner access. No product task selected.
- Repository lease: no lease/lock mechanism exists in workspace; branch and remote synchronization validated before work.
- Open Dependabot PRs #347 and #348 target `main`; both fail Rust/coverage checks. Policy leaves them untouched.
- `gh run list --branch develop --limit 5` shows no CI SUCCESS for current HEAD; latest real SUCCESS runs `36897066547` and `36897066543` target prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; runtime WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 07:15 -0300 — operational verification at `35b3bbf`

- `develop` and `origin/develop` synchronized at `35b3bbfe22c5b4bac84315f9ce15b2ea995c57b5`; working tree clean before this update.
- Backlog CODE executable remains exhausted; remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); `git diff --check` PASS.
- `cargo fmt --manifest-path server/Cargo.toml -- --check` cannot select targets on this repository; the valid gate is `cargo fmt --all --manifest-path server/Cargo.toml -- --check`, to be run before commit.
- Current `develop` HEAD has no remote CI SUCCESS; latest real SUCCESS runs `36897066547` and `36897066543` target prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Open Dependabot PRs #347 and #348 target `main`; both fail Rust/coverage checks. Policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; runtime WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 07:05 -0300 — operational verification at `fc7fdb8`

- `develop` and `origin/develop` synchronized at `fc7fdb8493d27f63db99d0a68d59983f2e7fb3b1`; working tree clean before this update.
- Backlog CODE executable remains exhausted; no safe product task identified. Remaining items require physical hardware, release confirmation, external secret, or remote runner access.
- Current remote CI has no SUCCESS for this SHA. Latest real SUCCESS runs `36897066547` and `36897066543` target prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`; no SUCCESS claimed for current HEAD.
- Open Dependabot PRs #347 and #348 target `main`; policy leaves them untouched. Both report Rust/coverage failures.
- Repository lease: no lease/lock mechanism exists in workspace; branch and remote synchronization validated before work.
- `scripts/validate-docs.sh` and `git diff --check` will run before commit. Static scanner `/root/scan_patterns.py` remains unavailable; no fabricated result.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 06:56 -0300 — operational verification at `e2899bb`

- `develop` and `origin/develop` synchronized at `e2899bb79bacb0277280eae0a0ca0b448f2c4473`; working tree clean before this update.
- Backlog CODE executable remains exhausted; no safe product task identified. Remaining items require physical hardware, release confirmation, external secret, or remote runner access.
- `scripts/validate-docs.sh` and `git diff --check` will run before commit. Current remote CI SUCCESS remains on prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`; no SUCCESS claimed for current HEAD.
- Open Dependabot PRs #347 and #348 target `main`; policy leaves them untouched.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 06:50 -0300 — operational verification at `4aa1683`

- `develop` and `origin/develop` synchronized at `4aa16836d33e65ba770fc2c38861f2ee3537a6d0`; working tree clean before this update.
- Backlog CODE executable remains exhausted; no safe product task identified. Remaining items require physical hardware, release confirmation or external secret.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS before this documentation update.
- Remote CI has no SUCCESS for current HEAD. Latest real SUCCESS runs `36897066547` and `36897066543` target prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Open Dependabot PRs #347 and #348 target `main`; Rust format/clippy/tests and coverage checks fail. Policy leaves them untouched.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 — operational verification at `cf2c1f9`

- `develop` and `origin/develop` synchronized at `cf2c1f9`; working tree clean.
- QR camera scanning is implemented in musician onboarding; latest verified commit adds browser camera fallback.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS.
- Remote CI has no SUCCESS at current HEAD; latest real SUCCESS runs are `36897066547` and `36897066543` on prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Open Dependabot PRs #347 and #348 target `main` and fail Rust format/clippy/tests plus coverage; policy leaves them untouched.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA physical, LAN, Raspberry Pi 5 and release `v0.3.1` remain pending. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 — QR camera scanning

- Musician QR onboarding now offers browser camera scanning via `BarcodeDetector` and `getUserMedia`, with paste fallback, bounded scan loop, stream cleanup and generation guards for unmount/stop/concurrent attempts.
- Unsupported browser, unavailable camera and scanner failures remain explicit fallback states; QR secret is held only in component state and sent through existing exchange flow.
- Evidence: CODE; `npm run typecheck`, `npm test` (67 tests), `npm run build` PASS. Physical validation remains `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-02 — QR audit events

- QR lifecycle audit implemented in SQLite `M004`: activate, rotate, deactivate, exchange accepted/rejected; records actor, action, generation and outcome only.
- QR secret, secret hash, refresh token, access token, JWT, profile name and request body never enter audit rows.
- Activation/rotation/deactivation state changes, QR-session revocation and audit insertion are transactional. Rotation revokes prior QR sessions.
- Evidence: `CODE`; full Rust gates passed; physical validation remains `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-02 04:45 -0300 — QR onboarding status reconciliation

- `develop` e `origin/develop` sincronizadas no HEAD `a70c3784b39569878e874269f8d7a647bc2a5357`; working tree limpa antes desta atualização.
- Rate limiting bounded por IP para `POST /api/v1/onboarding/qr/exchange` está implementado e coberto por testes; documentação histórica que o marca como pendente fica obsoleta.
- QR audit events continuam pendentes; próximo item CODE delimitado. Não registrar QR secret, hash, refresh token, JWT, nome ou corpo bruto.
- `scripts/validate-docs.sh` PASS e `git diff --check` PASS antes desta alteração.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS em `develop`: `36897066547` e `36897066543`, ambos no commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 04:26 -0300 — verificação operacional no HEAD `09ef9c6cccdd8c63a8c9c8857dd151fa53c9dd07`

- `develop` e `origin/develop` sincronizadas; working tree limpa.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS; `git diff --check` PASS.
- CI remoto: nenhum run SUCCESS no HEAD atual; últimos SUCCESS em `develop` são `36897066547` e `36897066543`, ambos no commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`. Política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.

## 2026-10-02 01:16 -0300 — verificação operacional no HEAD `ec2aab695dd70e9015de60c0d37204716bcfed56`

- `develop` e `origin/develop` permanecem sincronizadas no commit `ec2aab695dd70e9015de60c0d37204716bcfed56`; working tree estava limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 2` mostra últimos SUCCESS (`36897066547` e `36897066543`) no commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; nenhum CI SUCCESS foi confirmado no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS; `cargo fmt` na raiz não se aplica porque `Cargo.toml` está em `server/`; validação Rust deve usar `server/Cargo.toml`.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-02 01:01 -0300 — verificação operacional no HEAD `648ef32263020b4198047e95abc0dbefccb2432d`

- `develop` e `origin/develop` permanecem sincronizadas no commit `648ef32263020b4198047e95abc0dbefccb2432d`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 8` mostra últimos SUCCESS (`36897066547` e `36897066543`) no commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`; nenhum CI SUCCESS foi confirmado no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-02 00:57 -0300 — verificação operacional no HEAD `99fb4519b3369c5dd46b1faf8d32626612bc63fb`

- `develop` e `origin/develop` permanecem sincronizadas no commit `99fb4519b3369c5dd46b1faf8d32626612bc63fb`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 5` não mostra CI SUCCESS no HEAD exato; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem o commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` indisponível neste host, sem resultado inventado.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-02 00:46 -0300 — verificação operacional no HEAD `ac1c9fa6e56fd4a06c082a5d0e88200f5df39f04`

- `develop` e `origin/develop` permanecem sincronizadas no commit `ac1c9fa6e56fd4a06c082a5d0e88200f5df39f04`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto SUCCESS mais recente em `develop`: runs `36897066547` e `36897066543`, ambos no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` indisponível neste host, sem resultado inventado.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-02 00:35 -0300 — verificação operacional no HEAD `3c0a1164a846eedb61e757bb8e7c7ec1df67c1bf`

- `develop` e `origin/develop` permanecem sincronizadas no commit `3c0a1164a846eedb61e757bb8e7c7ec1df67c1bf`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 8` não mostra CI SUCCESS no HEAD exato; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem o commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-02 00:32 -0300 — verificação operacional no HEAD `08bdd104eb49ce3ae33d2c7121907dd8a54fc91b`

- `develop` e `origin/develop` permanecem sincronizadas no commit `08bdd104eb49ce3ae33d2c7121907dd8a54fc91b`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 5` não mostra CI SUCCESS no HEAD exato; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem o commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-02 00:26 -0300 — verificação operacional no HEAD `06c2f12fcdb155c45bbec8772fc85ea23aa65dbd`

- `develop` e `origin/develop` permanecem sincronizadas no commit `06c2f12fcdb155c45bbec8772fc85ea23aa65dbd`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 8` não mostra CI SUCCESS no HEAD exato; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência atual: `CODE/CI/SIMULATED`.

## 2026-10-01 23:26 -0300 — verificação operacional no HEAD `9b235636d9b7a99cedf40c4721a8f72b4b457ca6`

- `develop` e `origin/develop` permanecem sincronizadas no commit `9b235636d9b7a99cedf40c4721a8f72b4b457ca6`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 8` não mostra CI SUCCESS no HEAD `9b235636d9b7a99cedf40c4721a8f72b4b457ca6`; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em gates Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 22:56 -0300 — verificação operacional no HEAD `08e1e1d`

- `develop` e `origin/develop` permanecem sincronizadas no commit `08e1e1d`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 8` não mostra CI SUCCESS no HEAD `08e1e1d`; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` permanece indisponível, sem resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em gates Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 22:51 -0300 — verificação operacional no HEAD `eee9913783a448916074f296121b85acfa332c4a`

- `develop` e `origin/develop` permanecem sincronizadas no commit `eee9913783a448916074f296121b85acfa332c4a`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 8` não mostra CI SUCCESS no HEAD `eee9913783a448916074f296121b85acfa332c4a`; últimos runs SUCCESS (`36897066547` e `36897066543`) cobrem commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` permanece indisponível, sem resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em gates Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 22:16 -0300 — verificação operacional no HEAD `3ea88e3b6310010c454565b32c4155f116d66128`

- `develop` e `origin/develop` permanecem sincronizadas no commit `3ea88e3b6310010c454565b32c4155f116d66128`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- Nenhum CI remoto SUCCESS foi observado no HEAD `3ea88e3b6310010c454565b32c4155f116d66128`; últimos runs SUCCESS de `develop` (`36897066547` e `36897066543`) cobrem commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`, não este HEAD.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` permanece indisponível, sem resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em gates Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 22:11 -0300 — verificação operacional no HEAD `ff061d2bb9e7cb1f2b567d132f1547e3988556b8`

- `develop` e `origin/develop` permanecem sincronizadas no commit `ff061d2bb9e7cb1f2b567d132f1547e3988556b8`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- Nenhum CI remoto SUCCESS foi observado no HEAD `ff061d2bb9e7cb1f2b567d132f1547e3988556b8`; últimos runs SUCCESS de `develop` (`36897066547` e `36897066543`) cobrem commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`, não este HEAD.
- `scripts/validate-docs.sh` PASS; scanner `/root/scan_patterns.py` indisponível, sem resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em gates Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 21:41 -0300 — verificação operacional no HEAD `cec9104251980471562dca9f791c53f03635b8cc`

- `develop` e `origin/develop` sincronizadas no commit `cec9104251980471562dca9f791c53f03635b8cc`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto SUCCESS mais recente observado em `develop`: runs `36897066547` (CI, 13/13 jobs) e `36897066543` (Software Package Lifecycle Gates, 3/3 jobs), ambos no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` permanece indisponível, sem resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 21:26 -0300 — verificação operacional no HEAD `06a038b27f4c6d4fa23d7b6f2d8a82f69b62d6b6`

- `develop` e `origin/develop` sincronizadas no commit `06a038b27f4c6d4fa23d7b6f2d8a82f69b62d6b6`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 5` confirma runs SUCCESS `36897066547` (13/13 jobs) e `36897066543` (3/3 jobs), ambos no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem registrados como PASS; scanner `/root/scan_patterns.py` permanece indisponível, sem resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 20:47 -0300 — verificação operacional no HEAD `e2e74fb87e212317b9c8bfe4529678742b181064`

- `develop` e `origin/develop` sincronizadas no commit `e2e74fb87e212317b9c8bfe4529678742b181064`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto SUCCESS mais recente observado em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` permanece indisponível; nenhum resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 20:06 -0300 — verificação operacional no HEAD `bc6bcf4017624928f2e0543ae07c913138403e4e`

- `develop` e `origin/develop` sincronizadas em `bc6bcf4017624928f2e0543ae07c913138403e4e`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto SUCCESS mais recente observado em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` permanece indisponível; nenhum resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 20:02 -0300 — verificação operacional no HEAD `35ce318de7c678bf18ea77a7f1f67d84d23aef6c`

- `develop` e `origin/develop` sincronizadas no commit `35ce318de7c678bf18ea77a7f1f67d84d23aef6c`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `gh run list --branch develop --limit 5` confirma runs SUCCESS `36897066547` (13/13 jobs) e `36897066543` (3/3 jobs), ambos no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 19:51 -0300 — verificação operacional no HEAD `ee9ffa7a4f77110884df230770af73a8368cc4d0`

- `develop` e `origin/develop` sincronizadas no commit `ee9ffa7a4f77110884df230770af73a8368cc4d0`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhuma tarefa CODE segura nova identificada. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); gates locais PASS: Rust fmt, clippy, `cargo test --manifest-path server/Cargo.toml`, musician typecheck/tests/build e engineer typecheck/tests/build. Scanner `/root/scan_patterns.py` permanece indisponível; nenhum resultado inventado.
- CI remoto SUCCESS mais recente observado em `develop` continua nos runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em gates Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 19:48 -0300 — verificação operacional no HEAD `ce92866871aa46a35f77878c92092fc2ea583b19`

- `develop` e `origin/develop` sincronizadas no commit `ce92866871aa46a35f77878c92092fc2ea583b19`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto SUCCESS mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual ainda não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 19:36 -0300 — verificação operacional no HEAD `f794edfef61dae4ba4cc314415c1a1ab32c4c04b`

- `develop` e `origin/develop` sincronizadas no commit `f794edfef61dae4ba4cc314415c1a1ab32c4c04b`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- Nenhum CI remoto foi executado no HEAD atual `f794edfef61dae4ba4cc314415c1a1ab32c4c04b`; último CI SUCCESS observado em `develop` foi nos runs `36897066547` e `36897066543`, commit anterior `56a17fc87b004c104e730e36741ba75ad22796b4`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 19:26 -0300 — verificação operacional no HEAD `ae4f6a676b7e33b0692d39e21a199ff9f6dc54b4`

- `develop` e `origin/develop` sincronizadas no commit `ae4f6a676b7e33b0692d39e21a199ff9f6dc54b4`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 18:56 -0300 — verificação operacional no HEAD `ad1c0a783d9523af6496c3f48e38f256f6ca9613`

- `develop` e `origin/develop` sincronizadas no commit `ad1c0a783d9523af6496c3f48e38f256f6ca9613`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS já registrado; scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 18:36 -0300 — verificação operacional no HEAD `2ee7a692f57f8ded8529979db08aeb190c788196`

- `develop` e `origin/develop` sincronizadas no commit `2ee7a692f57f8ded8529979db08aeb190c788196`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 18:26 -0300 — verificação operacional no HEAD `80b2e58a71cce23e8f9fdd4e7a0d31b422cc4b1b`

- `develop` e `origin/develop` sincronizadas no commit `80b2e58a71cce23e8f9fdd4e7a0d31b422cc4b1b`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 18:11 -0300 — verificação operacional no HEAD `4b609b3e9fc1184fb647b0a9c1d5799d821f9d43`

- `develop` e `origin/develop` sincronizadas no commit `4b609b3e9fc1184fb647b0a9c1d5799d821f9d43`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 17:51 -0300 — verificação operacional no HEAD `a1ccd88002a3b7687c7e0f3b8ac7dd2eb4e00f78`

- `develop` e `origin/develop` sincronizadas no commit `a1ccd88002a3b7687c7e0f3b8ac7dd2eb4e00f78`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado nesta verificação. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); scanner `/root/scan_patterns.py` indisponível no host, sem resultado inventado.
- CI remoto mais recente listado em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 17:21 -0300 — verificação operacional no HEAD `91767b0cf05f3594f290fefb9f214835ca1ea751`

- `develop` e `origin/develop` sincronizadas no commit `91767b0cf05f3594f290fefb9f214835ca1ea751`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado nesta verificação. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS; scanner `/root/scan_patterns.py` indisponível no host, sem resultado inventado.
- CI remoto mais recente listado em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 17:15 -0300 — verificação operacional no HEAD `2eea373`

- `develop` e `origin/develop` sincronizadas no commit `2eea373`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado nesta verificação. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`); scanner `/root/scan_patterns.py` indisponível no host, sem resultado inventado.
- CI remoto mais recente listado em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual `2eea373`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 17:06 -0300 — verificação operacional no HEAD `48cd82a`

- `develop` e `origin/develop` sincronizadas no commit `48cd82a212092f82666ff3f4e80ae842971ea11b`; working tree limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado nesta verificação.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado.
- Último CI bem-sucedido verificado em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 17:01 -0300 — verificação operacional no HEAD `207dbce`

- `develop` e `origin/develop` sincronizadas no commit `207dbce579456454554ce2594a2611f9dd425d9d`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 17:10 -0300 — verificação operacional no HEAD `39e4936`

- `develop` e `origin/develop` sincronizadas no commit `39e49365c8e6bba4254da9ccf57c1e00460bf9e2`; working tree limpa antes desta atualização documental.
- Nenhuma tarefa CODE executável nova foi identificada nesta verificação; nenhum código de produto novo foi alterado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual. CI do HEAD atual não foi confirmado.
- Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado. PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 16:46 -0300 — verificação operacional no HEAD `44f1109`

- `develop` e `origin/develop` sincronizadas no commit `44f1109b1a9c2db47523abaaab495d7c0732c321`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual. CI do HEAD atual não foi confirmado.
- Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado. PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 16:28 -0300 — verificação operacional no HEAD `ac480a4`

- `develop` e `origin/develop` sincronizadas no commit `ac480a43c560e30fa8fbb44e812982ebbc2581f3`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual. CI do HEAD atual não foi confirmado.
- Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado. PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 16:23 -0300 — verificação operacional no HEAD `8c09074`

- `develop` e `origin/develop` sincronizadas no commit `8c090742dfb07d38533023f5f1115f39651a0c88`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado.
- CI remoto mais recente em `develop`: runs `36897066547` e `36897066543`, ambos **16/16 SUCCESS** no commit `56a17fc87b004c104e730e36741ba75ad22796b4`, não no HEAD atual. CI do HEAD atual não foi confirmado.
- Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado. PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em Rust/coverage; política vigente não altera essas branches.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 16:18 -0300 — verificação operacional no HEAD `b8a6852`

- `develop` e `origin/develop` sincronizadas no commit `b8a6852f68ba0fa4ff047d0d30e6b4387236b217`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado.
- Gates locais reais PASS: Rust fmt, clippy, `cargo test --manifest-path server/Cargo.toml` (537 testes + doctests); musician typecheck, 61 testes e build; engineer typecheck, 59 testes e build.
- `scripts/validate-docs.sh` PASS. Frontend exigiu `npm test -- --run`; `--watchAll=false` é inválido para Vitest e não representa falha de código.
- CI remoto SUCCESS mais recente em `develop`: runs `36897066547` e `36897066543`, ambos cobrem `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual. Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em Rust/coverage; política vigente não altera essas branches. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; evidência `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 16:11 -0300 — verificação operacional no HEAD `f33cc6b`

- `develop` e `origin/develop` estavam sincronizadas no commit `f33cc6b8bd5c7641fc02d49198f04de13c39d312` antes desta atualização documental; working tree estava limpa naquele ponto.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), cobrindo `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 16:00 -0300 — verificação operacional no HEAD `eb3e1f8`

- `develop` e `origin/develop` estão sincronizadas no commit `eb3e1f8c965b0e4b477dd7ffbd8d60d584850def`; working tree limpa.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), cobrindo `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:38 -0300 — verificação operacional no HEAD `8ab4d5d`

- `develop` e `origin/develop` estão sincronizadas no commit `8ab4d5d772ccc1f613c149e747ad5999500e5a5d`; working tree limpa.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), cobrindo `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual; CI do HEAD atual não foi confirmado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e gates anteriores permanecem evidência `CODE/CI/SIMULATED`. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:33 -0300 — verificação operacional no HEAD `e4eea70`

- `develop` e `origin/develop` estão sincronizadas no commit `e4eea704ec747319aa44845da2681e9e0b12d477`; working tree limpa.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`).
- CI remoto mais recente listado para `develop` é SUCCESS nos runs `36629568576` (`CI`) e `36629568577` (`Software Package Lifecycle Gates`), mas ambos cobrem `19f0d5b4f76ffd5cafe4ee25d584de8db9b9176e`, não o HEAD atual; CI do HEAD atual não foi confirmado.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas têm falhas em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:28 -0300 — verificação operacional no HEAD `c3548e1`

- `develop` e `origin/develop` estão sincronizadas no commit `c3548e11631719b0bc9b54a952f496366de76ac4`; working tree limpa antes desta atualização.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), cobrindo `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual; CI do HEAD atual ainda não foi confirmado.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e gates anteriores permanecem evidência `CODE/CI/SIMULATED`. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:20 -0300 — verificação operacional no HEAD `e78bc60`

- `develop` e `origin/develop` estão sincronizadas no commit `e78bc60b04df1cfb39a26086cee9da3a49c6b2fe`; working tree limpa.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), cobrindo `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual; CI do HEAD atual ainda não foi confirmado.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:16 -0300 — verificação operacional no HEAD `8ed5572e`

- `develop` e `origin/develop` estão sincronizadas no commit `8ed5572eacbdc89c046fe64d3cc62f3fb4ef0834`; working tree limpa.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), mas ambos cobrem `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:06 -0300 — verificação operacional no HEAD `a2287d3`

- `develop` e `origin/develop` estão sincronizadas no commit `a2287d3bb49aa833fa2373ae37a6caf947aaef32`; working tree limpa antes desta atualização.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), mas ambos cobrem `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; fora da branch autorizada e não alteradas.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 15:01 -0300 — verificação operacional no HEAD `3aeb4be`

- `develop` e `origin/develop` estão sincronizadas no commit `3aeb4be101dd1fae38f84096a1941799fc9a8987`; working tree limpa antes desta atualização.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), mas ambos cobrem `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; fora da branch autorizada e não alteradas.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 14:52 -0300 — verificação operacional no HEAD `c3f5481`

- `develop` e `origin/develop` estão sincronizadas no commit `c3f5481d51fdb010873b43187514c972f85c13a1`; working tree limpa antes desta atualização.
- CI remoto mais recente em `develop` concluiu **16/16 SUCCESS** nos runs `36897066547` (`CI`) e `36897066543` (`Software Package Lifecycle Gates`), mas ambos cobrem `56a17fc87b004c104e730e36741ba75ad22796b4`, não o HEAD atual.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; fora da branch autorizada e não alteradas.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 14:46 -0300 — verificação operacional no HEAD `dc28639`

- `develop` e `origin/develop` estão sincronizadas no commit `dc2863919b7ca38abfcbee58d6bb0d38b9418660`; working tree limpa.
- CI remoto real do HEAD exato concluiu **16/16 SUCCESS**: `CI` run `36897066547` e `Software Package Lifecycle Gates` run `36897066543`, ambos com jobs executados.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; fora da branch autorizada e não alteradas.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.
## 2026-10-01 14:36 -0300 — verificação operacional no HEAD `56a17fc`

- `develop` e `origin/develop` estão sincronizadas no commit `56a17fc87b004c104e730e36741ba75ad22796b4`; working tree limpa.
- CI remoto real do HEAD exato concluiu **16/16 SUCCESS**: `CI` run `36897066547` e `Software Package Lifecycle Gates` run `36897066543`, ambos com jobs executados.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; fora da branch autorizada e não alteradas.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:56 -0300 — verificação operacional no HEAD `f2fe475`

- `develop` e `origin/develop` estão sincronizadas no commit `f2fe475f006f0e5f10ea618b0d1d31827a80c652`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop` (`36795546987` e `36795546985`) é SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS (`documentation validation passed (version 0.3.1)`). Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:51 -0300 — verificação operacional no HEAD `6eedcec`

- `develop` e `origin/develop` estão sincronizadas no commit `6eedcec51515cf5c48f8e09a350ad4482f097cc3`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto listado para `develop` não cobre o HEAD atual; últimos runs SUCCESS (`36795546987` e `36795546985`) estão no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem evidência `CODE/CI/SIMULATED`. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:47 -0300 — verificação operacional no HEAD `9f2fdcb`

- `develop` e `origin/develop` estão sincronizadas no commit `9f2fdcbe1f77367d96c4bc85d230f4052bef305c`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto listado para `develop` não cobre o HEAD atual; últimos runs SUCCESS (`36795546987` e `36795546985`) estão no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e gates locais anteriores permanecem evidência `CODE/CI/SIMULATED`. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:26 -0300 — verificação operacional no HEAD `8a88e1a`

- `develop` e `origin/develop` estão sincronizadas no commit `8a88e1ad416742bdfe34fa898982f7bd6bffaef0`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto listado para `develop` ainda não cobre o HEAD atual; últimos runs SUCCESS (`36795546987` e `36795546985`) estão no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:16 -0300 — verificação operacional no HEAD `a32ad85`

- `develop` e `origin/develop` estão sincronizadas no commit `a32ad85` (`a32ad85710960e98c6a8df0d757638b540c198b`); working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto listado para `develop` ainda não cobre o HEAD atual; runs recentes `36795546987` e `36795546985` são SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:06 -0300 — verificação operacional no HEAD `73ca4d4`

- `develop` e `origin/develop` estão sincronizadas no commit `73ca4d4`; working tree limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto listado para `develop` ainda não cobre o HEAD atual; runs recentes `36795546987` e `36795546985` são SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 09:00 -0300 — verificação operacional no HEAD `d19ef24`

- `develop` e `origin/develop` estão sincronizadas no commit `d19ef24`; working tree estava limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto listado para `develop` ainda não cobre o HEAD atual; runs recentes `36795546987` e `36795546985` são SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 08:51 -0300 — verificação operacional no HEAD `6e13b95`

- `develop` e `origin/develop` estão sincronizadas no commit `6e13b95`; working tree estava limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS no commit `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual. Nenhum CI remoto foi declarado para `6e13b95`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 08:43 -0300 — verificação operacional no HEAD `341ef99`

- `develop` e `origin/develop` estão sincronizadas no commit `341ef99`; working tree estava limpa antes desta atualização.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- Gates locais reais PASS: `cargo fmt --manifest-path server/Cargo.toml --all -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (537 testes + doctests); frontends musician (61/61) e engineer (59/59) com typecheck, testes e build PASS.
- CI remoto mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS no commit `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual. Nenhum CI remoto foi declarado para `341ef99`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 08:10 -0300 — verificação operacional no HEAD `c8c83d7`

- `develop` e `origin/develop` estão sincronizadas no commit `c8c83d71e9c3a18756badf8e8a6961b4d875cec4`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop` permanece SUCCESS nos runs `36795546987` (`CI`) e `36795546985` (`Software Package Lifecycle Gates`), ambos no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 06:11 -0300 — verificação operacional no HEAD `85f410e`

- `develop` e `origin/develop` estão sincronizadas no commit `85f410e35f048b4a18c9242c04f9b1da36c64798`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop` permanece SUCCESS nos runs `36795546987` (`CI`) e `36795546985` (`Software Package Lifecycle Gates`), ambos no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` PASS. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 06:08 -0300 — verificação operacional no HEAD `ae6d62d`

- Antes desta atualização, `develop` e `origin/develop` estavam sincronizadas no commit `ae6d62de641d305345621b4ffa2ab7873520f946`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop`: `CI` run `36777931225` e `Software Package Lifecycle Gates` run `36777931370`, ambos SUCCESS em `8fce7d845e75e66e03a46b7513e98a8e164eb90a`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Validação documental e gates locais permanecem evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 06:02 -0300 — verificação operacional no HEAD `c4d4286`

- No instante da verificação, antes deste commit documental, `develop` e `origin/develop` estavam sincronizadas no commit `c4d42861fc69d5b46e5968385adbc815c1fc7e0d`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo identificado. Itens restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto disponível em `develop` não cobre HEAD atual; últimos runs listados são SUCCESS em `c3b25fe2e46d613a5de6b70510ba2974cddac899`, não neste commit.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; branches externas fora da política vigente.
- Validação documental PASS (`scripts/validate-docs.sh`); scanner `/root/scan_patterns.py` indisponível neste host. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 05:35 -0300 — verificação operacional no HEAD `64d4887`

- `develop` e `origin/develop` estão sincronizadas no commit `64d4887b35003d59240f01e6d4779e0780b8fb07`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto novo foi identificado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto mais recente em `develop`: runs `36683056715` (`CI`) e `36683056760` (`Software Package Lifecycle Gates`), ambos SUCCESS em `13f3db10aaf4c587663e7ff3d93a9e55236b3faf`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas UNSTABLE, falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Validação documental PASS (`scripts/validate-docs.sh`); scanner `/root/scan_patterns.py` indisponível neste host. Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 05:30 -0300 — verificação operacional no HEAD `dadd613`

- `develop` e `origin/develop` estão sincronizadas no commit `dadd613e3df6c293862206e65d2df8ea288f4583`; working tree estava limpa antes desta atualização documental.
- Backlog CODE executável permanece esgotado; nenhum código de produto foi alterado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto real mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 05:26 -0300 — verificação operacional no HEAD `14b28c6`

- `develop` e `origin/develop` estão sincronizadas no commit `14b28c68d17e0cb9aa3e16663cc2fc78d74b1eb2`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum código de produto foi alterado. Itens `[ ]` restantes exigem hardware físico, confirmação de release ou secret externo.
- CI remoto real mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política vigente não altera essas branches.
- Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 05:06 -0300 — verificação operacional no HEAD `1c54796`

- `develop` e `origin/develop` estão sincronizadas no commit `1c54796d771097c5052812536b6e84339180e74b`; working tree limpa.
- Nenhuma tarefa CODE executável nova identificada; backlog executável permanece esgotado. Nenhum código de produto foi alterado.
- CI remoto real mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política desta execução não altera essas branches.
- Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.
## 2026-10-01 04:51 -03 — verificação operacional no HEAD `795b48e`

- `develop` e `origin/develop` estão sincronizadas no commit `795b48e`; working tree limpa antes desta atualização.
- Nenhuma tarefa CODE executável nova identificada; backlog executável permanece esgotado. Nenhum código de produto foi alterado.
- CI remoto real mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política desta execução não altera essas branches.
- Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 04:45 -03 — verificação operacional no HEAD `e35eb00`

- `develop` e `origin/develop` estão sincronizadas no commit `e35eb008d97235f17514a5ae6fec830aaf6a60b7`; sem alterações não preparadas; esta atualização documental está staged.
- Nenhuma tarefa CODE executável nova identificada; backlog executável permanece esgotado. Nenhum código de produto foi alterado.
- CI remoto real mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não no HEAD atual.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política desta execução não altera essas branches.
- Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 04:30 -03 — verificação operacional no HEAD `a032267`

- `develop` e `origin/develop` estão sincronizadas no commit `a032267bbdc2bd6926ee30f6b8801c7b2b618dfc`; working tree limpa após esta atualização.
- Nenhuma tarefa CODE executável nova identificada; backlog executável permanece esgotado. Não foi inventada tarefa nem alterado código de produto.
- Não há CI remoto executado no HEAD atual; últimos runs `CI` `36795546987` e `Software Package Lifecycle Gates` `36795546985` são SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; política desta execução não altera essas branches.
- Evidência permanece `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 04:12 -03 — verificação operacional no HEAD `5520490`

- `develop` e `origin/develop` estão sincronizadas no commit `5520490f80eddf57c709b14192bd04f2a006b79d`; working tree limpa antes desta atualização.
- Gates locais reais PASS: `cargo fmt --manifest-path server/Cargo.toml --all -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (537 testes + doctests); frontends musician e engineer typecheck, testes (61/61 e 59/59) e builds PASS.
- Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado. CI remoto mais recente em `develop` (runs `36795546987` e `36795546985`) é SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não neste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em `Rust Format + Clippy + Tests` e `Rust Code Coverage`; fora da branch autorizada.
- Backlog CODE executável permanece esgotado; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `CODE/CI/SIMULATED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 03:31 -03 — verificação operacional no HEAD `71be6e31912a`

- `develop` e `origin/develop` estão sincronizadas no commit `71be6e31912aa1bb5ee1f270ad72629f91633a07`; working tree limpa.
- Backlog CODE executável permanece esgotado; itens restantes exigem validação física, confirmação de release ou secret externo. Nenhuma tarefa nova foi inventada.
- CI remoto real mais recente em `develop`: workflows `CI` e `Software Package Lifecycle Gates`, runs `36795546987` e `36795546985`, SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`; não são evidência deste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`, fora da branch autorizada.
- Evidência: `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 — verificação operacional no HEAD `8cfb87f`

- `develop` e `origin/develop` estão sincronizadas no commit `8cfb87fb0c89404fc0f0627ebade4295b8df1748`; working tree limpa.
- Backlog CODE executável permanece esgotado; itens restantes exigem validação física, confirmação de release ou secret externo. Nenhuma tarefa nova foi inventada.
- CI remoto real mais recente em `develop`: workflows `CI` e `Software Package Lifecycle Gates`, runs `36795546987` e `36795546985`, SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`; não são evidência deste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`, fora da branch autorizada.
- Evidência: `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 — verificação operacional registrada no commit `d39b916`

- Antes desta atualização, `develop` e `origin/develop` estavam sincronizadas no commit `af4318e22c3f0eacfd9bb89ff6eb8f46addcabd2`; working tree estava limpa.
- Backlog CODE executável permanece esgotado; itens restantes exigem validação física, confirmação de release ou secret externo. Nenhuma tarefa nova foi inventada.
- CI remoto real mais recente em `develop`: workflows `CI` e `Software Package Lifecycle Gates`, runs `36795546987` e `36795546985`, SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`; não são evidência deste HEAD.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em gates Rust/coverage, fora da branch autorizada.
- Evidência: `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 — estado operacional verificado no HEAD `bfa2dd8`
## 2026-10-01 — verificação local no HEAD `9377ed0`

- `develop` e `origin/develop` estão sincronizadas no commit `9377ed020c595125a975315e1f10ed7c91594bb1`; working tree limpa antes desta atualização.
- Gates locais reais PASS: `cargo fmt --manifest-path server/Cargo.toml --all -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (537 testes); frontends musician e engineer typecheck, testes (61/61 e 59/59) e builds PASS.
- CI remoto não executou neste HEAD; últimos SUCCESS em `develop` (`36795546987` e `36795546985`) pertencem a `be8c3691ea891fc84728f7ca5fe7021878bf1651`. Scanner `/root/scan_patterns.py` indisponível; nenhum resultado inventado.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas falham em gates Rust/coverage; política vigente proíbe alteração nessas branches. Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.


- `develop` e `origin/develop` estão sincronizadas no commit `bfa2dd8065a9fe1120dbdaad9ea26d70936d5526`; working tree limpa.
- Backlog CODE executável permanece esgotado; nenhum item seguro novo pode ser inventado. Itens restantes exigem hardware físico, confirmação de release ou provisionamento externo de secret.
- Não há CI remoto executado no HEAD atual; últimos runs SUCCESS em `develop` (`36795546987` e `36795546985`) pertencem a `be8c3691ea891fc84728f7ca5fe7021878bf1651`, não são evidência deste commit.
- PRs Dependabot #347 e #348 continuam abertas contra `main`; ambas falham em `Rust Format + Clippy + Tests` e `Rust Code Coverage`. Política vigente proíbe alterar essas branches.
- Evidência atual: `CODE/CI/SIMULATED`; WebRTC/DTLS-SRTP runtime, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 — verificação local no HEAD `43955b9`

- `develop` e `origin/develop` estão sincronizadas no commit `43955b9d95102c5821593fb40260ad1d0db4ebd1`; working tree limpa antes desta atualização.
- Gates locais reais PASS: `cargo fmt --manifest-path server/Cargo.toml --all -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (todos os testes: 537 unit/integration e doctests PASS); frontends musician e engineer typecheck, `npm test` (61/61 e 59/59) e build PASS.
- Primeiro comando frontend com `--watchAll=false` falhou porque Vitest 5 não aceita essa opção; comando canônico `npm test` executado depois PASS. Nenhum código foi alterado.
- Scanner `/root/scan_patterns.py` não está disponível neste host; nenhum resultado de scanner inventado. PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas Rust/coverage; fora da branch autorizada.
- CI remoto mais recente em `develop`: runs `36795546987` e `36795546985`, ambos SUCCESS no commit anterior `be8c3691ea891fc84728f7ca5fe7021878bf1651`; não é evidência do HEAD atual.
- Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` continuam `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 — verificação local no HEAD `a5774a3`

- `develop` e `origin/develop` estão sincronizadas no commit `a5774a3d88226411b29c2c92f50381d9e06a14e3`; working tree limpa antes desta atualização.
- Gates locais reais PASS: `cargo fmt --manifest-path server/Cargo.toml --all -- --check`; `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`; `cargo test --manifest-path server/Cargo.toml` (todos os testes); frontend musician typecheck, 61/61 testes e build; frontend engineer typecheck, 59/59 testes e build.
- Primeiro comando Rust sem `--manifest-path` falhou por inexistência de `Cargo.toml` na raiz; comando canônico executado depois PASS. Primeiro comando frontend usou `--watchAll`, opção inválida no Vitest 5; `npm test` canônico executado depois PASS.
- Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado de scanner inventado. Diff permanece vazio; nenhum commit de código criado.
- CI remoto não tem run no HEAD atual; último sucesso registrado (`36795546987`/`36795546985`) pertence a `be8c3691ea891fc84728f7ca5fe7021878bf1651`.
- PRs Dependabot #347 e #348 continuam abertas contra `main`, ambas com falhas em gates Rust; fora da branch autorizada.
- Evidência `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` continuam `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

## 2026-10-01 — estado verificado no HEAD `3152f44b3b76086e7bc97c09b2e091f1aba20df9`

- `develop` e `origin/develop` apontam para `3152f44b3b76086e7bc97c09b2e091f1aba20df9`; working tree limpa antes desta atualização.
- Nenhuma tarefa CODE executável nova identificada no backlog; atualização reconcilia estado operacional real.
- CI remoto real mais recente em `develop`: `CI` run `36795546987` e `Software Package Lifecycle Gates` run `36795546985`, ambos SUCCESS em `be8c3691ea891fc84728f7ca5fe7021878bf1651`; ainda não há run no HEAD atual.
- PRs Dependabot #347 e #348 permanecem abertas contra `main`, com falhas nos gates Rust; política local proíbe alterações nessas branches.
- Evidência atual: `CODE/CI/SIMULATED`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`; `PHYSICAL: USER-APPROVED / NOT EXECUTED`.

# OPEN IEM PLATFORM

# START.md --- Master Engineering Bootstrap & Development Specification

## Estado canônico atual

As 13 fases canônicas permanecem fixas. O snapshot operacional atual está em branch `develop`, HEAD verificado `642c432f37ac5b92c126ab007478616de13877b6`; `main` e `origin/main` permanecem em `83af911849a6145d5c8c7c81d72b6003a2beb80e`. As fases incrementais não substituem nem renumeram as fases canônicas. Evidência atual: `CODE/CI/SIMULATED`; PRs Dependabot #347 e #348 permanecem abertas contra `main`, com falhas atuais em gates Rust; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` continuam pendentes. CI verde mais recente não corresponde ao HEAD atual.


```text
Concluídas: Phase 0, Phase 1, Phase 2, Phase 3, Phase 4, Phase 6
Parciais:   Phase 5, Phase 7
Pendentes:  Phase 8, Phase 9, Phase 10, Phase 11, Phase 12
Progresso:  6/13 fases concluídas
```

Targets Linux-first: Debian, Ubuntu e Raspberry Pi OS em amd64 e arm64. Raspberry Pi 3 é baseline mínimo da família; Pi 4, Pi 5 e futuras versões compatíveis são alvos por capabilities Linux, sem dependência de BCM2712/RP1. Raspberry Pi é plataforma suportada, não requisito de funcionamento. Windows x64 permanece evolução futura/backlog e macOS, Android e iPadOS continuam fora do MVP. Suporte validado permanece limitado à evidência real; software release pode passar sem hardware físico. Testes físicos são `HARDWARE_CERTIFICATION`, nunca bloqueio automático de desenvolvimento/CI/software release. O core de áudio deve permanecer independente de plataforma; nenhum target vira claim sem evidência.

Estado de implementação: o backend Linux usa integração feature-gated JACK/PipeWire e ainda requer validação de hardware; backend nativo Windows WASAPI/ASIO ainda não está implementado; runtime Raspberry Pi 5 ARM64 ainda não foi validado.

Receiver dedicado é somente possibilidade futura, condicionada a requisito técnico medido, análise, ADR e protótipo.


## 1. Agent mission

You are the autonomous engineering team responsible for designing,
specifying, implementing, testing, reviewing, documenting, packaging and
releasing Open IEM Platform.

Canonical repository:
`https://github.com/rickslamaral/open-iem-platform` Canonical workspace:
`/workspace/open-iem-platform/`

Always start with:

``` bash
cd /workspace/open-iem-platform
```

The repository is the source of truth. Never create another repository
or project workspace.

## 2. Product

Open IEM Platform is an open-source personal in-ear monitoring platform
for bands, churches, rehearsal rooms, venues and live production. It
receives multichannel audio from a digital mixer or audio interface,
creates independent monitor mixes, and provides real-time control to
musicians and engineers.

``` text
Digital Mixer / Audio Interface
            |
            v
      Open IEM Server
            |
         PipeWire
            |
        Mix Engine
            |
   Independent Mixes
            |
      Audio Transport
            |
       Local Network
       /     |      \
  Musician Musician Musician
   Client   Client   Client
      |       |       |
     IEM     IEM     IEM
```

The system is local-first and must not depend on Internet connectivity
for live audio operation.

## 3. Product vision

The long-term platform should support multiple audio interfaces,
multichannel input, independent stereo mixes, musician self-service
mixing, engineer control, scenes, presets, channel groups, EQ,
compressor, limiter, optional reverb, meters, diagnostics, network
monitoring, Linux, Windows, macOS where technically possible, Raspberry
Pi, x86_64 mini PCs, future dedicated receivers when justified by measured technical requirements,
open API, open protocol and console integrations.

Do not implement the whole vision at once.

## 4. Product principles

1.  Audio-first.
2.  Local-first.
3.  Realtime-safe.
4.  Control plane separated from audio plane.
5.  Server-authoritative state.
6.  Strong authorization.
7.  Hardware abstraction.
8.  Cross-platform where practical.
9.  Open source.
10. Extensible.
11. Observable.
12. Testable.
13. Recoverable.
14. Documentation-driven development.
15. Specification before implementation.
16. Measure performance instead of assuming it.
17. Safe audio defaults.
18. Backward-compatible versioned protocols.

## 5. Mandatory engineering workflow

For every significant feature:

``` text
DISCOVER -> RESEARCH -> SPECIFY -> ARCHITECT -> PLAN -> IMPLEMENT -> TEST -> SECURITY REVIEW -> CODE REVIEW -> DOCUMENT -> PACKAGE -> CHANGELOG -> COMMIT / PR
```

For research that must survive sessions, use the project LLM Wiki skill during `RESEARCH`: orient with `SCHEMA.md`, `index.md` and recent `log.md`; capture immutable raw sources; create cited, interlinked notes; run `python3 scripts/validate-llm-wiki.py --wiki <path>`. Keep implementation truth in source and canonical `docs/`; wiki never replaces `START.md`, `TODO.md`, handoff, changelog or release gates.

Do not skip specification for architectural or audio features.

## 6. Documentation is mandatory every round

Every meaningful implementation round, milestone, commit series or PR
must review and update relevant Markdown documentation.

At minimum inspect:

``` text
README.md
CHANGELOG.md
START.md
docs/
```

Examples:

-   API change -\> API documentation.
-   Protocol change -\> protocol documentation + ADR.
-   Audio architecture change -\> audio docs + ADR.
-   New feature -\> product spec + user documentation.
-   Build change -\> build/release documentation.
-   Security change -\> security documentation.
-   Deployment change -\> deployment documentation.
-   Bug fix -\> troubleshooting documentation when applicable.

Never leave documentation describing obsolete behavior.

## 7. CHANGELOG

Maintain `CHANGELOG.md` using Keep a Changelog style. Every meaningful
PR that changes user-visible behavior, architecture, API, protocol,
deployment, security or developer workflow must update the Unreleased
section.

``` markdown
## [Unreleased]

### Added
### Changed
### Fixed
### Security
```

Do not reconstruct the changelog at release time.

## 8. Versioning

Use Semantic Versioning:

``` text
MAJOR.MINOR.PATCH
```

Early development may use `0.x`. Keep one source of truth, preferably a
root `VERSION` file or workspace version source. Automate
synchronization into package metadata where practical.

## 9. Release engineering

Generate versioned build artifacts. The first official package format is `.deb`: `open-iem_<version>_amd64.deb` and `open-iem_<version>_arm64.deb`. Package lifecycle must cover install, configuration, systemd, reinstall, upgrade, uninstall and explicit purge while preserving configuration/data on upgrade. `make test-release` is software-only and must not require physical hardware.

Target where technically feasible:

-   Linux x86_64
-   Linux ARM64
-   Windows x86_64
-   macOS arm64
-   macOS x86_64
-   Raspberry Pi Linux ARM64

Raspberry Pi may use a native systemd deployment rather than a desktop
installer.

Potential artifact formats:

``` text
Linux: tar.gz, deb, AppImage where appropriate
Windows: zip, MSI/installer where appropriate
macOS: dmg, app bundle, zip
```

Only publish formats that are actually built and tested.

## 10. Cross-platform architecture

The core server should be Rust and platform-independent where possible.

``` text
                Open IEM Core
                     |
          +----------+----------+
          |                     |
      Linux Audio            Desktop Host
      PipeWire/ALSA          Windows/macOS
          |                     |
          +----------+----------+
                     |
                Control API
                     |
                Web UI / PWA
```

Platform-specific audio backends must be abstracted.

Potential backends:

``` text
Linux: PipeWire / ALSA
Windows: WASAPI / ASIO where appropriate
macOS: CoreAudio
```

Do not claim support before validation.

## 11. Desktop packaging

Evaluate and document a cross-platform packaging strategy. Tauri +
Rust + Web UI is a preferred direction if it satisfies requirements;
alternatives may be selected after evaluation.

Create:

``` text
docs/architecture/DESKTOP-PACKAGING.md
docs/decisions/ADR-DESKTOP-PACKAGING.md
```

before committing to the final framework.

The underlying server architecture must remain the same for headless and
desktop modes.

## 12. Server modes

Support:

### Headless server

Raspberry Pi, Linux mini PC, dedicated appliance, controlled through web
UI.

### Desktop server

Windows, macOS and Linux desktop, optionally wrapped in a desktop shell.

## 13. Client modes

Support conceptually:

-   PWA/Web for control and quick access;
-   native desktop client where useful;
-   future native Android/iOS if browser audio limitations justify it;
-   future dedicated receiver.

Do not force a browser to perform realtime audio if validation shows it
is unsuitable.

## 14. Audio/control separation

Control plane:

``` text
PWA / Desktop Client -> HTTP/REST -> WebSocket -> Control API -> Authorization -> Mix Engine
```

Audio plane:

``` text
Audio Interface -> PipeWire -> Mix Engine -> Audio Transport -> Local Network -> Receiver -> IEM
```

Never assume WebSocket is the primary audio transport.

## 15. Audio engine

Initial Linux implementation:

``` text
PipeWire
ALSA
```

PipeWire is the Linux audio graph. Open IEM owns logical channels, mix
model, sends, routing policy, permissions, scene state, processing
configuration and streaming policy.

## 16. Audio format

Initial target:

``` text
48 kHz
24-bit capture where supported
32-bit float internal DSP where appropriate
stereo mixes
```

Architecture should allow 44.1, 48 and 96 kHz in future. Avoid
unnecessary conversions.

## 17. Mix engine

Each input channel may feed multiple mixes.

Each send:

``` text
gain_db
pan
mute
solo
enabled
locked
```

Each mix:

``` text
id
name
musician_id
channels
master_gain_db
master_mute
limiter
processing
permissions
revision
```

## 18. MVP

The MVP target is:

``` text
8 input channels
2 independent stereo mixes
2 musicians
2 clients
48 kHz
gain
pan
mute
master
limiter
WebSocket control
PWA
local LAN
Linux
Raspberry Pi target
```

MVP must prove audio, mixing, control, authorization, network and
recovery.

## 19. Audio transport research

Do not assume the final protocol. Create
`docs/research/AUDIO-TRANSPORT-EVALUATION.md` and compare RTP/UDP,
WebRTC, custom UDP and QUIC/WebTransport where relevant.

Evaluate latency, jitter, packet loss, CPU, memory, browser
compatibility, Android, iOS, Windows, macOS, Linux, recovery,
synchronization, security, complexity and scalability. Use benchmarks
and create `docs/decisions/ADR-004-audio-transport.md` after evaluation.

## 20. Browser audio constraint

Never assume a browser can receive arbitrary UDP. Separate the musician
control client from the audio receiver. Possible audio approaches
include WebRTC, supported browser media transport, native client or
dedicated receiver.

## 21. Server web interface

The server UI should provide a professional live-monitoring workflow
with:

``` text
Dashboard
Audio Devices
Input Channels
Musicians
Connected Devices
Mixes
Mixer
Scenes
Presets
Network
Diagnostics
System
Logs
Settings
```

Configuration wizard:

``` text
1. Select audio interface
2. Detect inputs
3. Name channels
4. Create musicians
5. Assign mixes
6. Configure network
7. Connect clients
8. Verify audio
9. Save scene
10. Start session
```

Use validated UX patterns from the researched market without copying
branding, proprietary text or implementation. User-facing Open IEM
documentation must not mention competing products.

## 22. Musician client

Provide:

``` text
My Mix
Channels
Faders
Mute
Pan
Master
Connection Status
Audio Status
Preset Selection
```

Future: personal EQ, talkback, ambient/room mic.

A musician cannot change another musician's mix.

## 23. Engineer console

Provide channels, musicians, mix matrix, devices, network, scenes,
locks, meters and system health.

## 24. Mix matrix

Represent the domain as Channel x Mix so the architecture can scale from
2 to 16+ musicians.

## 25. Scenes

Support create, save, rename, duplicate, recall, delete, export and
import. Scene contents include channels, mixes, levels, pan, mute,
routing, processing and locks.

## 26. Presets

Support versioned channel presets and mix presets. Presets must be safe,
validated, exportable and importable.

## 27. Future features

Plan, but do not prematurely implement:

-   channel groups;
-   talkback;
-   ambient/room microphones;
-   PFL/AFL;
-   advanced metering;
-   VST3/LV2/CLAP evaluation;
-   native mobile apps;
-   dedicated receiver;
-   console adapters.

Each feature requires its own specification before implementation.

## 28. Device management

Track:

``` text
device_id
device_name
device_type
musician_id
IP
connection
last_seen
latency
jitter
packet_loss
stream_state
client_version
```

## 29. Authorization

Roles:

``` text
ADMIN
ENGINEER
MUSICIAN
```

Musician edits assigned mix. Engineer manages all
mixes/devices/scenes/locks. Admin manages system configuration.
Server-side authorization is mandatory.

## 30. Engineer lock

Support locks for channel, send, mix, master, routing, processing and
scene.

## 31. Database

Use SQLite initially with migrations. Initial entities:

``` text
User
Musician
Channel
Mix
MixSend
Device
Scene
Preset
Permission
SystemConfig
AuditEvent
```

## 32. API

Version under `/api/v1`.

Minimum endpoints:

``` text
GET    /system
GET    /audio/devices
GET    /channels
POST   /channels
PATCH  /channels/:id
GET    /musicians
POST   /musicians
PATCH  /musicians/:id
GET    /mixes
GET    /mixes/:id
GET    /devices
GET    /scenes
POST   /scenes
POST   /scenes/:id/recall
DELETE /scenes/:id
GET    /presets
POST   /presets
```

## 33. WebSocket

Endpoint:

``` text
/ws/v1
```

Envelope:

``` json
{
  "version": 1,
  "message_id": "uuid",
  "timestamp": 0,
  "type": "mix.send.set",
  "source": "musician-client",
  "payload": {}
}
```

Server is authoritative: validate -\> authorize -\> apply -\> confirm
-\> broadcast.

## 34. Protocol versioning

All protocol messages must include a version. Breaking changes require a
new major protocol version. Unknown commands return structured errors.

## 35. Realtime thread rules

Never perform database I/O, network I/O, filesystem I/O, blocking calls,
unbounded work, unnecessary allocation or heavy locks in realtime audio
processing.

## 36. Backend

Preferred backend: Rust. Use a workspace where appropriate:

``` text
server/
├── Cargo.toml
└── crates/
    ├── audio-engine
    ├── mix-engine
    ├── streaming
    ├── control
    ├── device-manager
    ├── scene-manager
    └── state-store
```

## 37. Frontend

Preferred: TypeScript + React + Vite. Applications:

``` text
web/musician
web/engineer
```

## 38. Mobile UX

Mobile-first, touch-friendly, stage-readable, high contrast, simple,
fast and reconnect-resilient. Use large faders and obvious mute
controls.

## 39. Observability

Expose CPU, RAM, XRUN, PipeWire state, audio device, sample rate,
channels, mixes, clients, latency, jitter, packet loss, reconnects and
stream state. Use structured logs.

## 40. Diagnostics

Create:

``` bash
open-iem diagnostics
```

Validate OS, CPU, RAM, storage, PipeWire, ALSA, audio interface, sample
rate, channels, network, ports, realtime capability and XRUNs. Generate
a shareable report.

## 41. CLI

Initial commands:

``` bash
open-iem status
open-iem audio devices
open-iem channels
open-iem mixes
open-iem devices
open-iem scenes
open-iem diagnostics
open-iem version
```

## 42. Raspberry Pi

Reference: Raspberry Pi 5. Create `deployment/raspberry-pi/` with
install, audio, network, service and diagnostics scripts. Create
`open-iem-server.service`. Support automatic startup, safe restart and
watchdog.

## 43. Cross-platform audio

Investigate Linux PipeWire/ALSA, Windows WASAPI/ASIO and macOS
CoreAudio. Do not claim support before actual build and runtime
validation.

## 44. Network

Recommended topology:

``` text
Open IEM Server -> Ethernet -> Dedicated AP -> Musicians
```

Provide diagnostics. Do not hard-code a router vendor.

## 45. Client discovery

Evaluate QR pairing, short codes, mDNS/DNS-SD, local discovery and
manual IP. Security must be considered.

## 46. Session management

A session identifies server, session, musician, mix and client. Sessions
must be revocable and safely reconnectable.

## 47. Safe failure

Define behavior for server crash, audio-device disconnect, network
disconnect, client disconnect and invalid/corrupt configuration. Other
musicians must continue when one client disconnects.

## 48. Backup / restore

Provide:

``` bash
open-iem backup
open-iem restore
```

Backups contain configuration, not secrets unless explicitly intended.

## 49. Security

Mandatory: dependency audit, secret scanning, server-side authorization,
input validation, session expiration, WebSocket authorization, audit
logging, safe defaults and license/dependency review.

## 50. Testing

Layers:

-   unit;
-   integration;
-   frontend;
-   audio;
-   network;
-   hardware;
-   load.

Audio tests include latency measurement and XRUN observation. Network
tests include packet loss, jitter and reconnect.

## 51. MVP acceptance

MVP must prove:

1.  8 channels detected.
2.  2 independent stereo mixes.
3.  Mixes remain independent.
4.  Musician A controls only Mix 1.
5.  Musician B controls only Mix 2.
6.  Disconnecting A does not break B.
7.  Reconnect restores A.
8.  Server restart restores valid state.
9.  WAN outage does not stop LAN operation.
10. 60-minute stability test.
11. Metrics captured.
12. Documentation matches implementation.
13. Build artifacts are reproducible.

## 52. Performance

Do not publish unmeasured latency. Every performance claim must include
hardware, OS, sample rate, buffer, interface, network, codec, receiver
and measurement method.

Initial control target: `<100 ms`. Audio latency must be measured and
optimized.

## 53. Skills system

Canonical project skills:

``` text
.agents/skills/
```

Required skills:

``` text
architect-designer/
product-spec/
superpowers/
senior-backend/
senior-frontend/
realtime-audio-engineer/
test-master/
code-documenter/
security-review/
code-review/
pr-review/
release-engineer/
documentation-manager/
```

## 54. External skill references

Use as references, not blind copies:

``` text
https://github.com/Jeffallan/claude-skills/tree/main/skills
https://github.com/alirezarezvani/claude-skills/tree/main/engineering-team
https://github.com/alirezarezvani/claude-skills/tree/main/.hermes/skills/claude-skills/engineering
```

Inspect relevant skills, adapt methodology, respect licenses.
Project-specific skills are authoritative.

## 55. Skill orchestration

For major features:

``` text
product-spec -> superpowers -> architect-designer -> specialist -> test-master -> security-review -> code-review -> documentation-manager -> release-engineer -> pr-review
```

Use only relevant skills for small tasks.

## 56. Agent compatibility

Support Hermes Agent, Claude Code, Codex and compatible agent systems.
Canonical skills live under `.agents/skills/`; adapters may exist under
`.claude/skills/` and `.hermes/skills/`. Avoid divergent copies.

## 57. Documentation structure

Create:

``` text
docs/
├── product/
├── architecture/
├── audio/
├── networking/
├── api/
├── security/
├── deployment/
├── testing/
├── decisions/
├── research/
├── releases/
├── user-guides/
└── reviews/
```

## 58. User documentation

Create:

``` text
docs/user-guides/GETTING_STARTED.md
docs/user-guides/SERVER_SETUP.md
docs/user-guides/MUSICIAN_SETUP.md
docs/user-guides/ENGINEER_GUIDE.md
docs/user-guides/NETWORK_GUIDE.md
docs/user-guides/AUDIO_INTERFACE_GUIDE.md
docs/user-guides/TROUBLESHOOTING.md
docs/user-guides/FAQ.md
```

Document only implemented behavior.

## 59. Musicians Guide

At the end of the initial product development cycle, create:

``` text
docs/user-guides/MUSICIANS-GUIDE.md
docs/user-guides/MUSICIANS-GUIDE.pdf
```

The guide must explain the actual Open IEM workflow: requirements,
network connection, client installation/access, identifying a mix,
channel volume, pan, mute, master, connection problems, reconnect,
no-audio troubleshooting, unstable-audio troubleshooting, safe IEM
practices, wired client where supported, device preparation and
contacting the engineer.

The guide must not mention competing products. It must describe only the
implemented Open IEM Platform.

## 60. PDF build

Create:

``` text
scripts/build-musicians-guide.sh
```

The PDF must be reproducibly generated from its Markdown/source
document. Regenerate whenever musician workflow changes.

## 61. Release artifacts

Generate versioned artifacts when supported:

``` text
Linux x86_64
Linux ARM64
Windows x86_64
macOS arm64
macOS x86_64
```

Use formats appropriate to the chosen packaging system. Do not publish
untested formats.

## 62. Release pipeline

Create:

``` text
.github/workflows/release.yml
```

On a version tag such as `v0.1.0`, perform version validation, tests,
build matrix, packaging, checksums, release metadata and GitHub Release
publication when configured.

Study the release workflow of JPMixer as an engineering reference:

``` text
https://github.com/JPMixing-inc/jpmixer/blob/main/.github/workflows/release.yml
```

Do not copy blindly; adapt to Open IEM.

## 63. Release artifact naming

Use deterministic names such as:

``` text
open-iem-server-0.1.0-linux-x86_64.tar.gz
open-iem-server-0.1.0-linux-arm64.tar.gz
open-iem-server-0.1.0-windows-x86_64.zip
open-iem-server-0.1.0-macos-arm64.dmg
open-iem-server-0.1.0-macos-x86_64.dmg
```

Adapt to actual packaging.

## 64. Checksums

Generate `SHA256SUMS` for release artifacts. Plan signing for future
releases.

## 65. Release manifest

Generate `release-manifest.json` with version, commit, build date,
platform, architecture, artifact and checksum.

## 66. Release notes

Every release includes What's New, Changed, Fixed, Security, Known
Issues, Supported Platforms, Installation and Upgrade Notes. Release
notes must match CHANGELOG.

## 67. Musicians Guide release integration

Whenever musician workflow changes:

1.  update Markdown source;
2.  regenerate PDF;
3.  validate PDF;
4.  include the guide in release artifacts where appropriate.

## 68. PR checklist

Every PR:

``` text
[ ] Requirements satisfied
[ ] Tests added/updated
[ ] Security considered
[ ] Documentation updated
[ ] CHANGELOG updated when applicable
[ ] Version impact considered
[ ] Build impact considered
[ ] Release impact considered
[ ] ADR added when architectural
[ ] No stale docs
```

## 69. Review severity

``` text
BLOCKER
HIGH
MEDIUM
LOW
```

Block merge for data loss, audio safety issues, realtime violations,
security vulnerabilities, broken protocol, failing required tests,
unreproducible builds or incorrect release metadata.

## 70. Research rules

Prefer official documentation, RFCs/specifications, official
repositories, source code when necessary and reputable engineering
sources. Record meaningful findings.

## 71. No invention rule

Use explicit labels:

``` text
UNKNOWN
HARDWARE VALIDATION REQUIRED
EXPERIMENTAL
```

Never present assumptions as facts.

## 72. External product references

External product documentation may be used to understand workflows,
requirements, user education and implementation patterns. Do not copy
proprietary code, branding or text. Do not mention competing products in
Open IEM user documentation.

## 73. Repository structure

Target:

``` text
/workspace/open-iem-platform/
├── .agents/skills/
├── .claude/skills/
├── .hermes/skills/
├── .github/workflows/
├── docs/
├── server/
├── web/
├── firmware/
├── deployment/
├── experiments/
├── tests/
├── scripts/
├── examples/
├── START.md
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── LICENSE
├── VERSION
└── .gitignore
```

Adapt only with documented architectural justification.

## 74. Documentation validation

Create `scripts/validate-docs.sh` to validate required documents,
Markdown links where possible, version consistency, release-document
placeholders, guide source and CHANGELOG Unreleased section.

## 75. Skill validation

Create `scripts/validate-skills.sh` to validate SKILL.md existence,
frontmatter, name, description and references.

## 76. Development log

Maintain `docs/DEVELOPMENT-LOG.md` with Date, Goal, Implemented, Tests,
Metrics, Problems, Decisions, Documentation and Next Step after each
meaningful milestone.

## 77. TODO

Maintain `docs/TODO.md` with BLOCKER, HIGH, MEDIUM, LOW and RESEARCH.

## 78. No false completion

Do not say "Done" because code compiles. Track:

``` text
IMPLEMENTED
TESTED
VERIFIED
HARDWARE VERIFIED
RELEASE READY
```

## 79. Hardware validation

When hardware is unavailable, mark `SIMULATED`. When tested, mark
`HARDWARE VERIFIED` and record hardware, OS, sample rate, buffer,
interface, network and measurement method.

## 80. Final engineering principle

Build:

``` text
Stable Audio
      ↓
Stable Mixing
      ↓
Stable Network
      ↓
Excellent UX
      ↓
Cross-platform Packaging
      ↓
Production Hardening
```

not a huge unstable feature list.

# 81. PHASE 0 --- PROJECT BOOTSTRAP + SPECIFICATION AUDIT

Do not start by implementing the complete product.

1.  Enter `/workspace/open-iem-platform`.
2.  Verify Git, remote, branch and status.
3.  Inspect repository and README.
4.  Inspect all existing documentation.
5.  Inspect environment.
6.  Inspect external skill references.
7.  Create project skill architecture.
8.  Create all required skills.
9.  Create documentation structure.
10. Audit specifications.
11. Identify architecture gaps.
12. Research unresolved decisions.
13. Create ADR baseline.
14. Create TODO and development log.
15. Create CHANGELOG.
16. Create documentation and skill validation scripts.
17. Create CI foundation.
18. Create release/build foundation.
19. Create user-guide structure and musician-guide source/build
    pipeline.
20. Commit bootstrap.

Required Phase 0 documents:

``` text
docs/SPEC-AUDIT.md
docs/ARCHITECTURE-GAPS.md
docs/DEVELOPMENT-ENVIRONMENT.md
docs/SKILLS.md
docs/DEVELOPMENT-LOG.md
docs/TODO.md
docs/research/
docs/decisions/
docs/reviews/
docs/user-guides/
```

Recommended first commit:

``` text
chore: bootstrap open iem engineering foundation
```

Do not mix the first audio implementation into this commit.

# 82. PHASE 1 --- AUDIO ENGINE POC

Only begin after Phase 0 review = PASS.

Goal:

``` text
USB Audio Interface
        |
     PipeWire
        |
     8 channels
        |
    Mix Engine
      /   \
   Mix 1  Mix 2
      \   /
    Local Output
```

No final network streaming yet.

# 83. PHASE 2 --- MIX ENGINE

Implement channels, mixes, sends, gain, pan, mute, master and limiter
with tests.

# 84. PHASE 3 --- BACKEND

Implement Rust, REST, WebSocket, SQLite, authorization, revisions and
state synchronization.

# 85. PHASE 4 --- MUSICIAN CLIENT

Implement pairing, assigned mix, faders, pan, mute, master, status and
reconnect.

# 86. PHASE 5 --- AUDIO TRANSPORT

Only after transport evaluation and ADR. Implement streaming,
packetization, jitter buffer, loss detection, reconnect and metrics.

# 87. PHASE 6 --- ENGINEER UI

Implement channels, musicians, mix matrix, devices, scenes, locks,
meters and diagnostics.

# 88. PHASE 7 --- ADVANCED DSP

Incrementally implement HPF, EQ, compressor, limiter improvements and
reverb. Every DSP module needs CPU analysis, realtime safety, tests,
bypass and safe defaults.

# 89. PHASE 8 --- DEPLOYMENT

Implement Raspberry Pi deployment, systemd, watchdog, diagnostics,
backup and upgrade process.

# 90. PHASE 9 --- CROSS-PLATFORM BUILDS

Implement and validate Linux x86_64, Linux ARM64, Windows x86_64, macOS
arm64 and macOS x86_64 where feasible. A platform is not supported until
its build/test succeeds.

# 91. PHASE 10 --- RELEASE ENGINEERING

Implement tagged releases, build matrix, packaging, checksums, release
manifest, GitHub Release and matching release notes.

# 92. PHASE 11 --- MUSICIANS GUIDE

Once the initial workflow is stable, produce the complete Markdown and
PDF musician guide from the actual implementation. This guide is part of
the product, not an afterthought.

# 93. PHASE 12 --- FUTURE RECEIVERS

Only after the core network audio protocol is stable, research and
prototype dedicated/native receivers when justified by measured technical requirements.

# 94. PHASE COMPLETION GATE

At the end of every phase create:

``` text
docs/reviews/PHASE-X-REVIEW.md
```

Include Objective, Implemented, Tests, Metrics, Problems, Security,
Architecture Impact, Documentation, Release Impact, Known Issues, Next
Phase and Status.

Status:

``` text
PASS
PASS WITH CONDITIONS
BLOCKED
```

# 95. FINAL DEVELOPMENT LOOP

Continue indefinitely using:

``` text
SPEC
 ↓
PLAN
 ↓
IMPLEMENT
 ↓
TEST
 ↓
REVIEW
 ↓
SECURITY
 ↓
DOCUMENT
 ↓
CHANGELOG
 ↓
PACKAGE
 ↓
COMMIT / PR
 ↓
NEXT
```

The repository must always make clear what exists, what was decided, why
it was decided, what is broken and what comes next.

# 96. START NOW

DO NOT RESTART THE PROJECT.

AUDIT THE CURRENT STATE FIRST.

Understand what has already been implemented.
Validate what actually works.
Identify architectural deviations.
Preserve valid work.
Refactor where necessary.
Update documentation.
Run Devil's Advocate.
Fix substantive issues.
Determine the real current phase.
Then continue development from that state.

Do not ask for permission to perform normal engineering tasks. Do not
implement the complete product in one pass. Make the repository the
source of truth. Update Markdown documentation every round. Update
CHANGELOG on every meaningful PR. Create and maintain the Musicians
Guide source and final PDF. Build and version release artifacts as the
product becomes releasable. Only advance when the current phase passes
its review gate.

---

# Extended Operating Requirements (received 2026-09-10)

## 97. Extended requirement — Makefile and Local Developer Interface — Mandatory

Required targets:

```bash
make help
make install
make run
make run-local
make up
make down
make logs
make status
make lint
make fmt
make test
make test-unit
make test-integration
make build
make package
make diagnostics
make docs
make validate
make clean
```

Rules:
- `make run` runs natively where supported.
- `make run-local` may use local simulation/dev services.
- `make up` may start project development dependencies.
- `make down` stops only project-owned development resources.
- `make logs` exposes useful service logs.
- `make status` reports runtime/build state.
- `make fmt` formats supported code.
- `make lint` fails on relevant lint errors.
- `make test` runs the appropriate test suites.
- `make build` builds the project.
- `make package` reports packages only when they were actually produced.
- `make validate` runs project validation gates.
- `make clean` removes generated artifacts without deleting source or user configuration.

Avoid duplicated business logic between Makefile, CLI and scripts.

---

## 98. Extended requirement — CLI / Make Parity

The CLI and Makefile should expose equivalent developer workflows.

Expected mapping:

```text
make install      ↔ iem install
make run          ↔ iem run-local / iem up
make lint         ↔ iem lint
make test         ↔ iem test
make build        ↔ iem build
make package      ↔ iem package
make clean        ↔ iem clean
make diagnostics  ↔ iem diagnostics
make status       ↔ iem status
```

Differences must be documented. `iem help` must list stable commands. CLI exit codes must be deterministic and CI-friendly.

Suggested convention:

```text
0 = success
1 = general failure
2 = invalid usage
3 = dependency/environment failure
4 = validation failure
5 = runtime/service failure
```

If the existing convention is coherent, preserve it and document it.

---

## 99. Extended requirement — Environment Validation

Maintain a single source of truth for environment validation. Detect, where relevant:

- OS
- CPU architecture
- kernel version
- libc
- Rust/Cargo
- Node.js
- required package manager
- Docker/Compose
- PipeWire
- ALSA
- required Linux packages
- compiler/build tools
- memory/storage
- network capabilities
- audio permissions
- realtime scheduling capabilities

Use explicit states:

```text
OK
MISSING
INCOMPATIBLE
OPTIONAL
NOT APPLICABLE
HARDWARE VALIDATION REQUIRED
```

Do not blindly install dependencies.

---

## 100. Extended requirement — Linux Audio Permissions

Native Linux audio must use least privilege. Investigate actual requirements for:

- ALSA device access
- PipeWire user-session/socket access
- realtime scheduling
- memory locking
- USB device access
- network ports

Do not automatically add groups/capabilities unless required. For every required permission document:

1. why it is required
2. exact group/capability
3. least-privilege alternative
4. verification command
5. removal/revert procedure

Do not run the whole server as root unless a demonstrated requirement exists.

---

## 101. Extended requirement — GitHub Actions / Runner Diagnostics

When CI fails immediately, inspect infrastructure before changing application code.

Inspect all `.github/workflows/*.yml` for:

- syntax
- triggers
- dependencies
- `runs-on`
- permissions
- environments
- secrets
- action versions
- reusable workflows
- matrices
- runner groups
- self-hosted labels

For GitHub-hosted execution, prefer a valid label such as:

```yaml
runs-on: ubuntu-latest
```

Do not use self-hosted labels unless the corresponding runner exists and is documented.

Also inspect:

- repository Actions settings
- organization Actions settings
- GitHub-hosted runner availability
- runner groups
- repository access to runner groups
- workflow approval requirements
- Actions policy restrictions
- environment protection rules
- GitHub service status when relevant

If all jobs fail immediately, have empty logs, or never reach the first executable step, classify the incident as potentially:

`RUNNER / PLATFORM / CONFIGURATION FAILURE`

until evidence proves otherwise.

---

## 102. Extended requirement — GitHub Actions Permissions — Least Privilege

Normal CI should use the minimum permissions necessary, for example:

```yaml
permissions:
  contents: read
```

A release job that creates or updates GitHub Releases may require:

```yaml
permissions:
  contents: write
```

Do not use:

```yaml
permissions: write-all
```

unless a documented and validated requirement exists.

Prefer the standard `GITHUB_TOKEN` when sufficient; do not introduce a Personal Access Token unnecessarily.

Document every permission beyond read-only repository access:

- permission
- job
- reason
- security impact

---

## 103. Extended requirement — CI Workflow Structure

Prefer separate concerns where useful:

```text
.github/workflows/
├── ci.yml
├── release.yml
└── docs.yml
```

Typical CI:

```text
checkout
  ↓
environment
  ↓
format
  ↓
lint
  ↓
unit tests
  ↓
integration tests
  ↓
frontend checks
  ↓
security/audit
  ↓
build
```

Typical release:

```text
tag
  ↓
validate
  ↓
build matrix
  ↓
package
  ↓
test artifacts
  ↓
checksums
  ↓
manifest
  ↓
publish GitHub Release
```

Never publish a release if required gates fail.

---

## 104. Extended requirement — Release Blocking Rules

A release is blocked when:

- required CI fails
- release workflow fails
- expected artifacts are missing
- checksums are missing/invalid
- release manifest is invalid
- package installation/validation fails
- version metadata disagrees
- security gates fail
- release documentation is stale
- required hardware validation is represented as complete without evidence

Never force a merge, recreate a tag, or weaken a gate merely to make a release appear successful.

---

## 105. Extended requirement — Version Consistency

The root `VERSION` is authoritative unless an ADR explicitly defines another source of truth.

Verify consistency among:

```text
VERSION
Cargo.toml / Cargo.lock where applicable
package.json where applicable
application metadata
CLI output
package filenames
release-manifest.json
Git tag
GitHub Release
release documentation
```

Use `vX.Y.Z` for Git tags.

---

## 106. Extended requirement — Release Manifest and Checksums

Every release should produce a machine-readable manifest and:

```text
SHA256SUMS
```

Example manifest shape:

```json
{
  "project": "open-iem-platform",
  "version": "0.0.0",
  "commit": "unknown",
  "build_date": "unknown",
  "artifacts": [],
  "checksums": [],
  "targets": []
}
```

Never fabricate metadata. Verify checksums before publication where practical. Fail the release if an expected artifact is missing.

---

## 107. Extended requirement — Installation / Upgrade / Uninstallation

Every supported package/install mechanism must document:

- installation
- configuration
- startup
- upgrade
- uninstall
- rollback where supported
- logs
- diagnostics

Do not leave services, permissions, or files behind without documentation. Provide an explicit uninstall path for system services.

---

## 108. Extended requirement — Configuration Management

Separate:

```text
source code
build configuration
default configuration
user configuration
secrets
runtime state
```

Never commit secrets. Prefer environment variables or documented configuration mechanisms for secrets. Never overwrite user configuration during upgrades without a migration strategy.

---

## 109. Extended requirement — Observability and Health

Expose, where implemented:

- health
- readiness
- audio engine state
- active audio device
- channel count
- mix count
- client count
- connection state
- XRUNs
- CPU
- memory
- network state
- packet statistics
- version

Never expose fake metrics. Use `UNKNOWN` when a metric is not implemented or measurable.

---

## 110. Extended requirement — Failure and Recovery

Define behavior for:

- audio device disconnect
- PipeWire restart
- server restart
- network interruption
- client disconnect/reconnect
- packet loss/reordering
- malformed messages
- stale revisions
- database failure
- configuration corruption
- process crash

Prefer safe recovery. For audio failures, prevent unexpected loud output.

---

## 111. Extended requirement — Safe Audio Defaults

Hearing safety is a product requirement. Evaluate protections such as:

- conservative startup levels
- master limits
- limiter
- safe reconnect behavior
- mute on uncertain routing
- protection against sudden gain jumps

Do not claim a hearing-safety guarantee. The musicians guide must state that users remain responsible for safe listening levels.

---

## 112. Extended requirement — Dependency Management

Dependencies must be explicitly declared, auditable, license-compatible, and security-reviewed where practical.

Run appropriate tooling such as:

```text
npm audit
cargo audit
```

and equivalent scanners when applicable.

Before adding dependencies consider:

- license
- maintenance
- security posture
- runtime cost
- realtime suitability
- platform support

---

## 113. Extended requirement — Documentation and Skills Validation

Validate documentation for:

- broken internal links
- stale paths
- unsupported claims
- version mismatches
- commands that no longer work
- missing referenced files
- stale architecture diagrams
- stale phase status

Use `scripts/validate-docs.sh` when present.

Use `scripts/validate-skills.sh` when present. Validate required skills, metadata, attribution, licenses, Hermes compatibility, and accidental duplicate/conflicting copies.

If validation tooling is missing and its creation is justified, add it.

---

## 114. Extended requirement — Development Log / TODO / Architecture Gaps

Maintain `docs/DEVELOPMENT-LOG.md` with date, phase, objective, work, tests, validation, architecture decisions, issues, documentation changes, and next step.

Maintain `docs/TODO.md` and `docs/ARCHITECTURE-GAPS.md` with explicit states where useful:

```text
P0 / P1 / P2 / P3
NOT STARTED
IN PROGRESS
BLOCKED
EXPERIMENTAL
VALIDATION REQUIRED
DONE
```

Never hide unresolved architecture problems.

---

## 115. Extended requirement — Release Incident Procedure

When a release is blocked:

```text
1. Capture workflow/run ID
2. Inspect workflow YAML
3. Inspect runner selection
4. Inspect Actions settings
5. Inspect permissions
6. Inspect environments
7. Inspect runner groups
8. Inspect logs
9. Determine whether the job actually started
10. Classify infrastructure vs code failure
11. Fix root cause
12. Rerun
13. Validate artifacts
14. Verify GitHub Release
15. Update documentation
16. Complete phase review
```

Do not repeatedly rerun a workflow without investigating the underlying cause.

---

## 116. Extended requirement — Phase Continuity for Existing Releases

If the repository is already beyond an earlier phase, do not restart it.

Audit evidence, verify tests/documentation/release state, resolve blockers, and continue from the real phase.

If a release exists without assets because CI/release Actions are blocked:

```text
Do NOT recreate the release.
Do NOT recreate the tag.
Do NOT mark the release complete.
Do NOT advance solely because local tests pass.
```

Fix CI/release infrastructure first, rerun the required workflows, verify assets/checksums/manifest, then update documentation and phase review.

---

## 117. Extended requirement — Automated Development-Loop Reporting

Every development-loop report should state:

```text
Phase
Branch
Commit
Working tree
Implementation
Tests
Security
CI
Release
Hardware validation
Documentation
Blockers
Next action
```

If blocked, explicitly state `BLOCKED` and the precise reason. Never report overall success when a release gate is broken.

---

## 118. Extended requirement — Repository Hygiene

Before completing a cycle:

```bash
git status
git diff
git diff --cached
git ls-files
```

Check for secrets, temporary files, local databases, logs, credentials, build artifacts, and machine-specific configuration.

Keep `.gitignore` current. Do not commit generated artifacts unless intentionally tracked.

---

## 119. Extended requirement — Definition of Done

A feature is not done merely because code exists.

As applicable, completion requires:

- implementation
- tests
- formatting/lint
- build validation
- security review
- Devil's Advocate review
- documentation
- changelog update
- ADR when needed
- phase impact assessment
- release impact assessment

Hardware-dependent features remain `HARDWARE VALIDATION REQUIRED` until actually validated.

Use `PASS WITH CONDITIONS` for documented non-critical limitations.

---

## 120. Extended requirement — Final Operating Rule

Always work from evidence:

```text
Measure.
Validate.
Document.
Then decide.
```

Never replace evidence with assumptions.

---

## 121. Extended requirement — Backlog as a Living Engineering Plan

`docs/TODO.md` is the authoritative project backlog, while this START.md is the operating contract.

Hermes must not attempt to implement every requirement in START.md at once. During every audit it must translate newly discovered work into backlog items and prioritize them.

Each significant backlog item should contain, where applicable:

- ID
- title
- phase
- priority
- status
- dependencies
- acceptance criteria
- validation required
- hardware validation requirement
- related ADR
- related tests
- related documentation
- release impact

Recommended statuses:

```text
NOT STARTED
READY
IN PROGRESS
BLOCKED
VALIDATION REQUIRED
DONE
DEFERRED
```

Recommended priorities:

```text
P0 = release/product blocker
P1 = required for current phase
P2 = important future work
P3 = optional/future enhancement
```

If a technically valuable idea is not appropriate for the current phase, add it to the backlog instead of implementing it prematurely.

---

## 122. Extended requirement — Requirements Traceability

For important requirements maintain traceability:

```text
Requirement
    ↓
Backlog Item
    ↓
Implementation
    ↓
Test
    ↓
Documentation
    ↓
Release
```

The project should be able to answer:

- Where is this requirement implemented?
- Which tests prove it?
- Which documentation describes it?
- Which phase introduced it?
- Which release contains it?

Do not claim a requirement is complete merely because source files exist.

---

## 123. Extended requirement — Definition of Ready

Before starting a non-trivial task, verify:

- objective is understood
- acceptance criteria exist
- dependencies are known
- architectural impact is understood
- required hardware is identified
- required credentials are identified
- test strategy exists
- documentation impact is known
- release impact is known

If important information is missing, mark the task `BLOCKED` or `VALIDATION REQUIRED` rather than inventing requirements.

---

## 124. Extended requirement — Branch and Pull Request Strategy

Prefer short-lived branches for substantive changes:

```text
main
 ├── feat/*
 ├── fix/*
 ├── refactor/*
 ├── test/*
 ├── docs/*
 ├── ci/*
 └── release/*
```

Normal flow:

```text
branch
  ↓
implementation
  ↓
tests
  ↓
security review
  ↓
code review
  ↓
Devil's Advocate
  ↓
CI
  ↓
PR
  ↓
merge
```

Direct commits to `main` may be used only for small, low-risk maintenance when the repository policy permits it.

Do not bypass branch protection or required CI checks.

Before creating a PR, inspect:

```bash
git status
git diff
git log --oneline -20
```

PR descriptions should state:

- objective
- implementation
- tests
- security impact
- architecture impact
- documentation changes
- known limitations
- hardware validation status
- release impact

---

## 125. Extended requirement — Audio Test Harness

The project must develop a repeatable audio test harness so the Mix Engine can be validated without requiring physical hardware for every test.

The harness should eventually support deterministic signals such as:

- silence
- sine wave
- impulse
- white/noise test signal where appropriate
- clipping input
- multiple simultaneous channels

Validate at minimum:

- channel isolation
- gain
- pan
- mute
- master
- send levels
- independent mixes
- limiter behavior
- clipping behavior
- sample-rate handling
- buffer behavior
- reset/recovery behavior

Conceptual flow:

```text
Generated Input
      ↓
Audio Engine
      ↓
Mix Engine
      ↓
Expected Output
      ↓
Automated Analysis
      ↓
PASS / FAIL
```

The harness must not replace physical audio validation.

---

## 126. Extended requirement — Audio Performance and Benchmarking

Performance claims must be backed by reproducible measurements.

Maintain benchmark documentation where useful:

```text
docs/benchmarks/
├── AUDIO-LATENCY.md
├── CPU-MEMORY.md
├── XRUNS.md
├── NETWORK-JITTER.md
└── TRANSPORT.md
```

Measure where applicable:

- end-to-end latency
- processing latency
- CPU utilization
- memory utilization
- XRUNs
- jitter
- packet loss
- reconnect time
- synchronization error

Do not publish target or observed numbers as facts until measured.

---

## 127. Extended requirement — Hardware Validation Matrix

Maintain a matrix distinguishing simulation, software validation and hardware validation.

Example:

| Capability | Docker | Linux x86_64 | Raspberry Pi | Windows | macOS | Android | iPadOS |
|---|---|---|---|---|---|---|---|
| Mix Engine | TEST | TEST | TEST | TEST | TEST | EXPERIMENTAL | EXPERIMENTAL |
| PipeWire | SIMULATED/NA | TEST | HARDWARE VALIDATION REQUIRED | N/A | N/A | N/A | N/A |
| USB Audio | SIMULATED | TEST | HARDWARE VALIDATION REQUIRED | TBD | TBD | EXPERIMENTAL | EXPERIMENTAL |
| Audio Transport | SIMULATED | TEST | HARDWARE VALIDATION REQUIRED | TBD | TBD | TBD | TBD |

Adapt the matrix to actual evidence.

Never turn `SIMULATED`, `BUILD`, or `TEST` into `SUPPORTED` without appropriate validation.

---

## 128. Extended requirement — Audio Clock, Drift and Synchronization

The architecture must explicitly address clocking before claiming multi-device synchronized audio.

Investigate:

- audio interface clock
- server clock
- receiver clock
- clock drift
- timestamping
- buffer drift
- resampling
- synchronization strategy
- multiple receiver synchronization
- behavior after network interruptions
- recovery from clock divergence

Create an ADR before committing to a synchronization architecture.

If unresolved:

`UNKNOWN`

If physical validation is needed:

`HARDWARE VALIDATION REQUIRED`

Do not assume independent device clocks remain synchronized indefinitely.

---

## 129. Extended requirement — Pairing and Device Identity

Define a secure device pairing flow before allowing arbitrary clients to control mixes.

Conceptual flow:

```text
New Device
    ↓
Pairing
    ↓
Authentication
    ↓
Device Identity
    ↓
Musician Assignment
    ↓
Authorized Mix
```

The design should address:

- pairing code/token
- device identity
- revocation
- reassignment
- expired pairing
- duplicate devices
- lost devices
- authorization after reconnect

A client must never be able to select another musician's mix merely by changing a client-side identifier.

---

## 130. Extended requirement — API and WebSocket Compatibility

The public control API and protocol must be versioned.

Current namespaces:

```text
/api/v1
/ws/v1
```

Define policy for:

- backward compatibility
- breaking changes
- deprecation
- client/server version mismatch
- message versioning
- unknown message types
- unknown fields
- stale revisions
- reconnect synchronization

Future native clients and receivers must not require uncontrolled protocol forks.

Breaking protocol changes require an ADR and migration strategy.

---

## 131. Extended requirement — State Synchronization

The engineer UI, musician clients and server must have a clear authoritative-state model.

Required principles:

```text
Server = authoritative state
Client = local representation
```

For state-changing commands:

```text
command
  ↓
validate
  ↓
authorize
  ↓
apply
  ↓
increment revision
  ↓
ack/confirm
  ↓
broadcast authoritative state
```

Handle:

- simultaneous edits
- stale clients
- lost WebSocket messages
- reconnect
- duplicate commands
- out-of-order commands
- optimistic UI rollback

Do not silently overwrite newer state with stale client state.

---

## 132. Extended requirement — Release Recovery and Existing Releases

When an existing release is blocked, recover it instead of recreating history.

Example:

```text
Existing tag/release
       ↓
CI failure
       ↓
DO NOT recreate tag
       ↓
DO NOT delete release
       ↓
Diagnose root cause
       ↓
Fix CI/configuration
       ↓
Rerun
       ↓
Validate artifacts
       ↓
Validate checksums
       ↓
Validate manifest
       ↓
Publish/attach assets
       ↓
Update documentation
```

Do not force merges or disable gates merely to obtain a green pipeline.

If a release has no assets, it is not considered complete merely because the GitHub Release object exists.

---

## 133. Extended requirement — GitHub Actions Runner Failure Diagnosis

When a workflow fails immediately, especially when all jobs fail with empty logs, distinguish runner/platform/configuration failure from application failure.

Inspect:

- workflow syntax
- `runs-on`
- runner availability
- self-hosted labels
- runner groups
- repository access to runner groups
- GitHub-hosted runner eligibility
- Actions enabled state
- allowed actions policy
- workflow execution protections
- organization policies
- environment protections
- required approvals
- `GITHUB_TOKEN` permissions
- reusable workflow permissions

Do not modify application code to compensate for a runner that never started.

GitHub documents that `permissions` can be scoped at workflow or job level and that specifying permissions limits unspecified permissions; use least privilege. For release publication, `contents: write` may be required, while normal CI commonly needs only read access. citeturn0search0turn0search7

Repository and organization settings can independently restrict whether Actions are enabled, which actions are allowed, and workflow execution. Verify these settings when jobs fail before execution. citeturn0search1turn0search3

---

## 134. Extended requirement — GitHub Actions Permission Policy

Prefer least privilege.

Normal CI example:

```yaml
permissions:
  contents: read
```

Release job example, only when required:

```yaml
permissions:
  contents: write
```

Do not default to:

```yaml
permissions: write-all
```

Do not request a PAT when `GITHUB_TOKEN` is sufficient.

If a permission is required, document:

- workflow
- job
- permission
- reason
- security impact

GitHub's documentation states that specifying individual permissions causes unspecified permissions to become `none`, which supports a least-privilege approach. citeturn0search0

---

## 135. Extended requirement — Configuration and Database Migration Safety

SQLite schema changes must use versioned migrations.

Every migration must define:

- version
- forward migration
- validation
- compatibility impact
- rollback strategy where feasible

Configuration changes must be versioned where necessary.

Upgrades must not silently destroy:

- users
- device assignments
- scenes
- presets
- permissions
- system configuration

Backup/restore procedures must be tested before being described as production-ready.

---

## 136. Extended requirement — Backup, Restore and Recovery Validation

The production deployment must eventually support safe recovery of important state.

Evaluate backup/restore for:

- SQLite database
- scenes
- presets
- configuration
- certificates/identities where applicable

Validate:

```text
backup
  ↓
clean environment
  ↓
restore
  ↓
validate schema
  ↓
validate state
  ↓
start service
  ↓
functional test
```

A backup mechanism is not considered production-ready until restoration has been tested.

---

## 137. Extended requirement — Security Threat Model

Maintain a lightweight threat model covering:

- unauthorized musician control
- unauthorized engineer control
- stolen/lost client device
- malicious local-network client
- malformed WebSocket messages
- malformed audio packets
- replayed commands
- stale authentication
- compromised server
- malicious package/update
- dependency compromise
- exposed management interface

Document trust boundaries between:

```text
Audio Interface
Server
Control Clients
Audio Receivers
Local Network
Internet/Cloud, if ever introduced
```

Do not assume the local network is inherently trusted.

---

## 138. Extended requirement — Supply Chain and Release Integrity

Evaluate adding:

- SBOM generation
- dependency license report
- dependency vulnerability report
- artifact checksums
- artifact provenance/attestation where practical
- signed releases where practical

Do not make supply-chain features mandatory for the MVP if they block core development, but add them to the backlog with appropriate priority.

Release artifacts must be traceable to:

```text
version
commit
workflow run
build target
source
checksums
```

---

## 139. Extended requirement — Crash Handling and Recovery

The server must eventually define behavior for:

- process crash
- audio backend crash
- PipeWire restart
- device disconnect
- database failure
- corrupted configuration
- network failure
- client failure

Production Raspberry Pi deployment should use appropriate service supervision.

Recovery must favor safe audio behavior and avoid unexpected loud output.

Do not claim crash recovery until it has been tested.

---

## 140. Extended requirement — Network Fault Injection

Before declaring audio transport production-ready, create repeatable tests for:

- packet loss
- packet duplication
- packet reordering
- jitter
- temporary network outage
- reconnection
- congestion
- malformed packets
- delayed packets
- receiver restart
- server restart

Use simulation for initial development and hardware/network validation before production claims.

---

## 141. Extended requirement — Safe Audio Failure Policy

For uncertain routing or recovery conditions, prefer safe behavior.

Evaluate:

- startup mute
- reconnect mute
- master level limits
- limiter
- protection against sudden gain jumps
- safe handling of stale state
- safe scene recall

The system must not claim to guarantee hearing safety. User documentation must emphasize responsible listening levels.

---

## 142. Extended requirement — Telemetry and Privacy Decision

Explicitly decide whether the product sends telemetry.

The default architecture is local-first.

Unless explicitly implemented and documented:

- do not send usage telemetry
- do not send audio content to cloud services
- do not collect unnecessary personal data
- do not require Internet connectivity for live audio

If telemetry is introduced in the future, create an ADR covering:

- data collected
- purpose
- retention
- opt-in/opt-out
- security
- privacy impact
- offline behavior

---

## 143. Extended requirement — Accessibility and Internationalization

For the PWA, evaluate baseline accessibility:

- keyboard navigation
- focus states
- readable labels
- accessible controls
- sufficient contrast
- screen-reader semantics where practical
- touch-friendly controls

Internationalization may remain future work unless required by the product, but text must not be unnecessarily hard-coded into architecture that would prevent future localization.

If not implemented, record as a backlog item rather than claiming support.

---

## 144. Extended requirement — Architecture Fitness Review

At the end of major phases, verify that implementation still respects the core architecture:

```text
Platform-independent Core
          ↓
Audio Abstraction
          ↓
Platform Backend
```

and:

```text
Control Plane ≠ Audio Plane
```

Look specifically for accidental coupling such as:

- Mix Engine importing PipeWire-specific code
- business logic in HTTP handlers
- UI becoming authoritative state
- database operations inside realtime code
- network operations inside realtime callbacks
- platform-specific assumptions leaking into core

If architectural drift is detected, record it in `docs/ARCHITECTURE-GAPS.md` and fix or backlog it.

---

## 145. Extended requirement — Final Extended Operating Contract

Hermes must continuously maintain four layers of truth:

```text
START.md
  = engineering operating contract

Architecture / ADRs
  = architectural decisions

docs/TODO.md
  = executable backlog

Code + Tests + CI
  = implementation evidence
```

If these disagree:

1. inspect evidence
2. identify the discrepancy
3. document it
4. determine the technically justified correction
5. update the appropriate source of truth
6. validate again

Never silently let contradictory documentation, code and tests accumulate.

The objective is not to maximize the number of files or features. The objective is to produce a validated, maintainable, secure and reproducible real-time IEM platform.

---

## 146. Hermes Agent interface — AGENTS.md

O Hermes Agent carrega automaticamente `AGENTS.md` quando o `workdir` de um cron job aponta para o workspace do projeto. Este arquivo é a **interface canônica** entre o Hermes Agent e o repositório.

O arquivo `AGENTS.md` na raiz do repositório deve:

```text
1. Repetir a missão do agente (referência a START.md como contrato)
2. Definir ordem de leitura ao iniciar cada run
3. Especificar o que o agente pode fazer autonomamente
4. Especificar o que exige confirmação explícita de Ricardo
5. Documentar procedimento de carregamento de credenciais
6. Listar gates obrigatórios antes de qualquer commit
7. Descrever o workflow por fase
8. Registrar o estado atual (Phase em andamento, CI, release)
9. Listar restrições permanentes em formato tabular
```

Manter `AGENTS.md` sincronizado com `START.md` e `docs/TODO.md`. Ao concluir uma Phase, atualizar a seção de estado atual em `AGENTS.md`.

O cron job `361e70c8e264` (loop diurno) e `7aee82067e22` (off-hours) usam `workdir=/workspace/open-iem-platform/` e carregam `AGENTS.md` automaticamente.

Nunca remover `AGENTS.md` do repositório: ele é a interface de bootstrapping para toda execução autônoma.

---

## 147. Recent requests — Raspberry Pi, Docker ALSA and musician flow

This section records current operating requirements from Ricardo. Keep it synchronized with code, tests and deployment documentation.

### 147.1 Raspberry Pi service address

The production systemd unit binds the API to loopback only:

```text
OPENIEM_BIND_ADDR=127.0.0.1:8080
```

Therefore the API is **not directly reachable through the Raspberry Pi LAN IP**. Caddy terminates TLS and exposes the service through the configured hostname or LAN IP:

```text
https://iem.local/api/v1/health
https://<RASPBERRY_PI_LAN_IP>/api/v1/health
```

Use the actual address assigned by the router/DHCP or configured as static. Do not hard-code an IP in application code. If `iem.local` mDNS is unavailable, use the Pi LAN IP in both Caddy configuration and `OPENIEM_ALLOWED_ORIGINS`.

The development Docker Compose service is different and binds the API container to `0.0.0.0:3000`, published only on the host loopback:

```text
127.0.0.1:3000 -> container:3000
```

Never copy this development binding into production. Production remains `127.0.0.1:8080` behind Caddy TLS.

### 147.2 Audio validation status

- VPS and CI audio path: **SIMULATED**.
- Docker image: `deployment/docker/alsa-sim/`.
- Host simulation uses Linux `snd-dummy` and mounts `/dev/snd` into the container.
- `aplay -l` must enumerate the dummy card inside container.
- Test tone uses `hw:0,0` for dummy card. On Raspberry Pi with USB interface, run `aplay -l` first and replace with detected card/device, commonly `hw:1,0`.
- `speaker-test -D hw:1,0 -c 2 -t wav` is valid only after confirming real USB card/device. It is not evidence of real audio when run against `snd-dummy`.
- No claim of real PipeWire/ALSA/Raspberry Pi 5 runtime support until hardware test produces evidence.

Commands:

```bash
sudo modprobe snd-dummy
make alsa-sim-build
make alsa-sim-run ALSA_DEVICE=hw:0,0
make alsa-sim-test
```

### 147.3 CI contract

GitHub Actions job `alsa-sim` runs on Ubuntu and uses ALSA userspace `null` PCM by default. GitHub-hosted Azure runners do not guarantee `snd-dummy` or `/dev/snd`; CI must not fail before tests because kernel module is unavailable.

CI must:

1. set `ALSA_SIM_MODE=null`;
2. build `open-iem-alsa-sim`;
3. run ALSA device enumeration inside container;
4. play test tone on `null` PCM;
5. run pytest suite.

Local hosts with `snd-dummy` may run `hw:0,0`. Raspberry Pi USB validation remains separate: run `aplay -l`, then use detected `hw:N,M`. This proves deterministic software simulation only. It does not prove USB audio, PipeWire, speaker output or Raspberry Pi runtime.

### 147.4 Musician + simulated audio tests

### 147.5 Musician profile onboarding and QR access

- Musician não precisa de senha para criar ou acessar seu perfil operacional. Fluxo futuro deve coletar nome e instrumento a partir de catálogo controlado, criar perfil individual e manter sessão persistente por refresh token revogável; não usar senha vazia, senha fixa, usuário compartilhado, JWT permanente ou token em URL.
- Admin ou Engineer autorizado pode ativar o modo QR no servidor. O QR deve carregar convite temporário, de alta entropia, armazenado somente como hash, com expiração, uso limitado ou single-use, rotação, revogação, rate limit e auditoria sem segredo. QR estático sem TTL é proibido.
- Musician lê QR, confirma servidor/projeto, informa nome e instrumento, cria somente seu próprio perfil e permanece na sessão até logout, expiração ou revogação. Autorização de mix, assignment e comandos WebSocket permanece server-authoritative.
- Backend QR seguro implementado no estado atual do working tree (HEAD `2374b59`; alterações não commitadas) em `server/api-server/src/routes/qr.rs`, com persistência `M003` em `server/api-server/src/db.rs`. Rotas: `GET /api/v1/admin/qr/status`, `POST /api/v1/admin/qr/activate`, `POST /api/v1/admin/qr/rotate`, `POST /api/v1/admin/qr/deactivate` e público `POST /api/v1/onboarding/qr/exchange`.
- Política efetiva: gestão QR exige `Engineer` mínimo; `Admin` herda acesso; `Musician` não gerencia QR. Exchange cria somente papel `Musician`, perfil `PENDING` e sessão com refresh cookie `HttpOnly; Secure; SameSite=Strict`; assignment de mix permanece operação separada e server-authoritative.
- Controles implementados: segredo aleatório de 32 bytes, somente hash SHA-256 persistido, TTL máximo de 10 minutos, uso único, rotação, validação de nome/instrumento, consumo atômico concorrente, Origin middleware e bloqueio de HTTP não-loopback sem override explícito. Rotação/desativação revoga sessões QR, refresh tokens e access sessions; refresh/access checks respeitam revogação.
- Status: frontend Musician QR paste/exchange e session restore via cookie implementados; camera scanning permanece `PENDING`; mix preferences/assignment UI permanece `PENDING`. Rate limit específico e auditoria QR ainda não implementados. Não declarar runtime WebRTC/PipeWire/ALSA/Raspberry Pi 5 ou release validado; esses itens seguem `PENDING/BLOCKED`. Evidência backend: `CODE/CI/SIMULATED`.
- Estado atual: backend implementado; frontend, assignment/preferences e validação física/release permanecem pendentes.

### 147.6 Engineer first-access password change

- Login bootstrap retorna `must_change_password=true`. Engineer Console deve bloquear dashboard e WebSocket, mostrar troca obrigatória via `PUT /api/v1/auth/password`, limpar o token revogado e exigir novo login após sucesso. Estado atual: frontend implementado e validado localmente; CI/runtime externo permanecem pendentes.

Integration tests must cover:

- Engineer assigns mix to musician;
- musician changes send gain and mute through HTTP;
- musician negotiates WebRTC offer against simulated backend;
- Engineer telemetry reports `backend=simulated` and `availability=simulated`;
- musician cannot read restricted telemetry;
- musician controls only assigned mix through WebSocket;
- musician receives `FORBIDDEN` for another mix.

Current evidence: 3 dedicated integration tests pass locally. Full suite must remain green before merge.

### 147.5 Release and change discipline

Do not move or recreate an existing release tag without explicit confirmation. Verify current tags, remote tags, HEAD, CI and release metadata before publishing. Keep simulation changes and musician-flow tests in separate commits when practical; preserve branch and worktree state until merge status is verified.

## 2026-10-02 08:06 -0300 — operational verification at `f697d67`

- `develop` and `origin/develop` synchronized at `f697d67`; working tree clean before this update.
- Backlog CODE executable remains exhausted; remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access. No product task selected.
- Open Dependabot PRs #347 and #348 target `main`; both remain open and fail Rust/coverage checks. User policy leaves them untouched.
- Remote CI has no SUCCESS for current HEAD; latest real SUCCESS runs `36897066547` and `36897066543` target prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- `scripts/validate-docs.sh` and `git diff --check` will run before commit. Static scanner `/root/scan_patterns.py` unavailable; no fabricated result.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; runtime WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 09:10 -0300 — operational verification at `3200b52`

- `develop` and `origin/develop` synchronized at `3200b52fd5305449ca06969d1fc2986948dca4ef`; working tree clean before this verification. Repository lease: no lease/lock mechanism exists.
- Backlog CODE executable remains exhausted. Remaining unchecked items require physical hardware, release confirmation, external secret, or remote runner access; no product task selected.
- `scripts/validate-docs.sh` and `git diff --check` PASS. Full Rust/frontend gates not rerun in this cycle; prior results remain historical evidence only.
- No remote CI SUCCESS covers current HEAD; latest real SUCCESS runs `36897066547` and `36897066543` cover prior SHA `56a17fc87b004c104e730e36741ba75ad22796b4`.
- Open Dependabot PRs #347 and #348 target `main`; Rust/coverage checks fail. User policy leaves them untouched.
- Static scanner `/root/scan_patterns.py` unavailable; no fabricated result. `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, physical PipeWire/ALSA, LAN, Raspberry Pi 5 and release `v0.3.1` remain `PENDING/BLOCKED`. Evidence: `CODE/CI/SIMULATED`.

## 2026-10-02 12:47 -0300 — verificação operacional no HEAD `ffe53d0`

- `develop` e `origin/develop` sincronizadas; working tree limpa antes desta atualização documental. Lease `.git/hermes-dev.lock` presente.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` serão executados nesta atualização. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto: últimos SUCCESS reais em `develop` são runs `36897066547` e `36897066543`, cobrindo SHA anterior; não contam como SUCCESS para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.


## 2026-10-02 12:56 -0300 — verificação operacional no HEAD `a1946e7`

- `develop` e `origin/develop` sincronizadas; working tree limpa antes desta atualização documental. Lease `.git/hermes-dev.lock` presente.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada: branches remotas órfãs inexistentes; PRs Dependabot #347 e #348 continuam abertas contra `main`, com falhas em Rust/coverage; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` serão executados nesta atualização. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; últimos SUCCESS reais em `develop` (runs `36897066547` e `36897066543`) cobrem SHA anterior e não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.
## 2026-10-02 14:17 -0300 — verificação operacional no HEAD `f9666af`

- `develop` e `origin/develop` sincronizadas no commit `f9666af7ead9391e3cde903c410f3aa96392438e`; working tree limpa antes desta atualização. Lease `.git/hermes-dev.lock` presente; validade não inferida.
- Backlog CODE executável permanece esgotado. Itens `[ ]` restantes exigem hardware físico, confirmação de release, secret externo ou runner remoto; nenhuma tarefa de produto segura foi selecionada.
- Limpeza validada após `git fetch --prune`: nenhuma branch remota órfã; branches Dependabot #347 e #348 continuam abertas contra `main`, ambas falhando em Rust/coverage; política vigente não altera essas branches.
- `scripts/validate-docs.sh` e `git diff --check` PASS nesta atualização. Scanner `/root/scan_patterns.py` indisponível neste host; nenhum resultado inventado.
- CI remoto não tem SUCCESS no HEAD atual; runs `36897066547` e `36897066543` cobrem SHA anterior `56a17fc87b004c104e730e36741ba75ad22796b4` e não contam para este HEAD.
- `PHYSICAL: USER-APPROVED / NOT EXECUTED`; WebRTC/DTLS-SRTP, PipeWire/ALSA físico, LAN, Raspberry Pi 5 e release `v0.3.1` seguem `PENDING/BLOCKED`. Evidência: `CODE/CI/SIMULATED`.
