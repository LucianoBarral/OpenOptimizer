# Subprojeto 2a — Modo Gaming · Implementation Plan

> Executado inline (superpowers:executing-plans). Plano enxuto: interfaces, testes e verificação; o código é escrito direto nos arquivos sob TDD (ver INTENT no histórico da sessão).

**Goal:** daemon `oo-gamingd.exe` que detecta jogos por snapshot de processos e aplica timer 0,5 ms, plano Desempenho Máximo, limpeza da standby list e prioridade via IFEO, revertendo ao fechar — com tela de controle no app.

**Spec:** `docs/superpowers/specs/2026-11-27-gaming-mode-design.md`

## Global Constraints
- Nunca `OpenProcess`/`QueryFullProcessImageName`/`SetPriorityClass`/injeção. Detecção só por `CreateToolhelp32Snapshot` (nomes).
- Executáveis do sistema via `oo_sys::windows::system_exe`.
- Daemon ≤ 5 MB RAM, CPU ≈ 0% aguardando. Sem janela (`windows_subsystem = "windows"`).
- Write-ahead da sessão (`gaming-session.json`) antes de trocar o plano de energia.
- Dados em `%ProgramData%\OpenOptimizer`; datas RFC 3339 UTC; nomes de exe comparados em minúsculas.

## Tasks

### 1. `crates/oo-gaming` (lógica pura) — TDD
- `config.rs`: `GameEntry { exe, name, source, enabled }`, `GamingConfig { enabled, notify, games }`, `GamingConfig::find_enabled(&HashSet<String>) -> Option<&GameEntry>`, `merge_discovered(&mut Vec<GameEntry>, Vec<GameEntry>) -> usize`.
- `status.rs`: `GamingState { Idle, Active }`, `GamingStatus { state, game, since, sessions, seconds_in_game, purges, purged_mb, daemon_version }`.
- `sys.rs`: `trait GamingSys { running_exes, set_timer, active_scheme, set_scheme, ensure_ultimate_scheme, free_mb, purge_standby }`; `trait SessionStore { load, save, clear }`; `FakeGamingSys`, `MemStore`.
- `session.rs`: `Session::recover(&dyn GamingSys, &mut dyn SessionStore)`, `Session::tick(&mut self, &GamingConfig, &dyn GamingSys, &mut dyn SessionStore, now_unix: u64, &mut GamingStatus) -> bool` (true = status mudou), `Session::shutdown(...)`.
- Testes (≥ 7): ativa jogo habilitado; ignora desabilitado; desativa e restaura plano; troca de jogo; purge só < 1024 MB e ≥ 300 s; não troca plano se já é o Ultimate; recuperação de sessão interrompida; config desligada desativa.
- Verificar: `cargo test -p oo-gaming`.

### 2. `crates/oo-gaming-win` — `WinGamingSys` + tarefa agendada
- Toolhelp snapshot → nomes em minúsculas; `NtSetTimerResolution(5000, on)`; plano via `oo_sys` (`power_active/set_active/duplicate`) + GUID do Ultimate criado guardado em `gaming-ultimate.txt` e validado contra `powercfg /list`; memória livre via `NtQuerySystemInformation(0x50)` (Free+Zero); purge via `NtSetSystemInformation(0x50, 4)` após habilitar `SeProfileSingleProcessPrivilege`.
- `task.rs`: `install(exe_path)`, `uninstall()`, `run()`, `exists()` com XML (§10 da spec) e `schtasks` absoluto.
- Testes de integração: snapshot contém o próprio processo de teste; `free_mb() > 0`; criar/consultar/remover tarefa `OpenOptimizer\_test`.

### 3. `oo-gaming-win::discovery` — descoberta de jogos
- GameConfigStore (`HKCU\System\GameConfigStore\Children\*\MatchedExeFullPath`), Epic (`*.item`: `DisplayName`, `LaunchExecutable`), Steam (`libraryfolders.vdf` → `appmanifest_*.acf` → maior `.exe` filtrado).
- Parsers puros testados com textos de exemplo (vdf/acf/item); teste de fumaça no Windows real (não falha se vazio).

### 4. `daemon/` — `oo-gamingd.exe`
- Mutex `Global\OpenOptimizerGamingMode`; `recover`; loop 3 s: recarrega config se mtime mudou; desligado → `shutdown` e sai; `tick`; grava status quando muda e a cada 30 s em jogo.
- Verificar: build release + medir RAM do processo rodando (≤ 5 MB).

### 5. Empacotamento
- `scripts/prebuild.mjs` (ui build + `cargo build -p oo-gamingd --release` + cópia para `app/binaries/oo-gamingd-x86_64-pc-windows-msvc.exe`) e `scripts/predev.mjs`; `tauri.conf.json`: `externalBin`, before*Command, `nsis.installerHooks`.
- `app/installer/hooks.nsh`: PREINSTALL/PREUNINSTALL `taskkill oo-gamingd.exe`; POSTINSTALL `schtasks /Run`; PREUNINSTALL `schtasks /Delete`.
- `.gitignore`: `app/binaries/`.

### 6. App — comandos e tweaks dinâmicos
- `gaming_get`, `gaming_set_enabled(bool)`, `gaming_set_game(exe, enabled)`, `gaming_add_game(path)`, `gaming_scan`.
- Ligar: aplica `gaming-global-timer` (motor), grava config, instala+roda a tarefa. Desligar: config `enabled=false` (daemon sai sozinho), remove tarefa, reverte `gaming-global-timer`.
- Jogo habilitado → tweak `ifeo:<exe>` (`CpuPriorityClass=3`) via motor.
- `tauri-plugin-dialog` para escolher `.exe`.

### 7. UI — tela Modo Gaming + Visão geral
- `routes/Gaming.svelte`: toggle, estado ao vivo (LogoMark loop quando ativo), contadores, lista de jogos, Procurar/Adicionar. Polling do status a cada 2 s só com janela visível.
- Visão geral: card do Modo Gaming com estado real; sidebar sem "Em breve" no Gaming.
- i18n pt-BR/en; Histórico mostra `ifeo:<exe>` e `gaming-global-timer` com títulos próprios.

### 8. Release v0.2.0 + verificação ao vivo
- CI verde → PR → merge → tag (com autorização explícita do usuário).
- Usuário: ligar Modo Gaming, abrir/fechar um jogo, conferir `powercfg /getactivescheme` e o status; medir RAM do daemon.
