<div align="center">

<img src="docs/design/brand/icon-1024.png" width="112" alt="Open Optimizer" />

# Open Optimizer

**Otimizador de PC para Windows, gratuito e open source.**<br/>
Menos input lag, frametime mais estável e Modo Gaming automático — com tudo reversível e sem risco de ban.

[![Release](https://img.shields.io/github/v/release/LucianoBarral/OpenOptimizer?style=flat-square&color=EEF3F7&labelColor=0B0C0E)](https://github.com/LucianoBarral/OpenOptimizer/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/LucianoBarral/OpenOptimizer/ci.yml?branch=main&style=flat-square&label=CI&labelColor=0B0C0E)](https://github.com/LucianoBarral/OpenOptimizer/actions/workflows/ci.yml)
[![Downloads](https://img.shields.io/github/downloads/LucianoBarral/OpenOptimizer/total?style=flat-square&color=EEF3F7&labelColor=0B0C0E)](https://github.com/LucianoBarral/OpenOptimizer/releases)
[![Licença MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-EEF3F7?style=flat-square&labelColor=0B0C0E)](LICENSE)
![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011-EEF3F7?style=flat-square&labelColor=0B0C0E)

[**Baixar**](https://github.com/LucianoBarral/OpenOptimizer/releases/latest) · [Changelog](CHANGELOG.md) ([EN](CHANGELOG.en.md)) · [Contribuir](CONTRIBUTING.md) · [Segurança](SECURITY.md)

</div>

---

## Por que o Open Optimizer

| | |
|---|---|
| **Tudo reversível** | Antes de alterar qualquer coisa, o valor original é lido do sistema e gravado em disco (journal *write-ahead*). Desfaça por item, por lote ou tudo — inclusive se o PC desligar no meio. |
| **Sem risco de ban** | O app **nunca** abre, injeta, lê ou escreve no processo de um jogo. Ele só ajusta o Windows por APIs públicas — compatível com Vanguard, Easy Anti-Cheat e BattlEye. |
| **Sem placebo** | Cada otimização mostra o motivo técnico ("Por quê?"). O que não tem efeito mensurável não entra. |
| **Leve de verdade** | Instalador de ~2 MB. Minimizado, o app usa ~23 MB; o Modo Gaming roda com ~0,6 MB. |
| **Dados reais** | Todo número na tela vem de leitura real do sistema. |

## Recursos

- **Otimizações de desempenho e latência** — prioridade MMCSS para jogos, gravação em segundo plano do Game Bar desligada, power throttling desligado, plano Desempenho Máximo, aceleração do mouse desligada.
- **Modo Gaming automático** — detecta o jogo, aplica timer 0,5 ms, plano Desempenho Máximo, limpeza de memória quando faltar e prioridade alta; desfaz tudo ao fechar o jogo. Descobre jogos da Steam, Epic e os que o próprio Windows reconhece.
- **Histórico com desfazer** — cada mudança registrada, com restauração completa ao original.
- **Bandeja do sistema** e **atualização automática assinada** pelo GitHub.
- **Português e inglês**.

## Instalação

1. Baixe `Open.Optimizer_x.y.z_x64-setup.exe` em [Releases](https://github.com/LucianoBarral/OpenOptimizer/releases/latest).
2. Execute o instalador. O Windows pode avisar "editor desconhecido" (o app ainda não tem assinatura de código paga) — clique em **Mais informações → Executar assim mesmo**.
3. O app pede permissão de administrador: é necessária para ajustar o sistema.

Depois de instalado, as atualizações chegam sozinhas e são verificadas por assinatura antes de instalar.

## Roadmap

- [x] **v0.1** — Núcleo, motor de reversão, interface, auto-update
- [x] **v0.2** — Modo Gaming automático
- [ ] Medição real de FPS, 1% low e frametime (antes × depois), via ETW — sem injeção
- [ ] Catálogo expandido de otimizações validado pela medição (rede, GPU, energia)
- [ ] Apps e ajustes no estilo do [winutil](https://github.com/ChrisTitusTech/winutil)
- [ ] Atualizador de drivers (Windows Update + fabricantes de GPU)
- [ ] Limpeza de memória e de arquivos temporários

## Compilar a partir do código

Requisitos: Windows 10/11, [Rust](https://rustup.rs) estável e Node 24.

```powershell
npm i ; npm --prefix ui i
node scripts/build-daemon.mjs          # compila o Modo Gaming (exigido pelo build do app)
cargo test --workspace --exclude open-optimizer   # testes sem precisar de admin
npm run tauri dev                      # terminal como administrador (o app exige elevação)
```

| Pasta | O que é |
|---|---|
| `crates/oo-core` | Motor de reversão (journal, apply/revert, conflitos) |
| `crates/oo-sys` | Acesso ao Windows (registro, energia, mouse) |
| `crates/oo-catalog` | Otimizações em TOML |
| `crates/oo-metrics` | Métricas reais (CPU, memória, timer, hardware) |
| `crates/oo-gaming*` · `daemon/` | Modo Gaming (lógica testável + implementação Windows + processo) |
| `crates/oo-views` | Contrato entre Rust e a interface (tipos TypeScript gerados) |
| `app/` · `ui/` | App Tauri 2 e interface Svelte 5 |

## Licença

[MIT](LICENSE) — use, estude, modifique e distribua.
