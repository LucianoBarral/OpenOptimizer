# Open Optimizer — Subprojeto 2a: Modo Gaming automático

**Data:** 2026-11-27 · **Status:** aprovado em brainstorming
**Depende de:** subprojeto 1 (motor de reversão, catálogo, app Tauri) — v0.1.2

## 1. Objetivo

Um processo em segundo plano que detecta quando um jogo abre, aplica otimizações de tempo de execução enquanto ele roda e desfaz tudo quando ele fecha — sem intervenção do usuário e **sem nunca abrir handle, injetar, ler ou escrever no processo do jogo**.

Subprojeto 2 foi decomposto: **2a Modo Gaming (este)** → 2b medição real de FPS/1% low/frametime via ETW → 2c catálogo expandido validado pela medição.

## 2. Decisões (fechadas no brainstorming)

| Tema | Decisão |
|---|---|
| Alcance | **Só a base** (opção A): o Modo Gaming não mexe em nenhum outro programa nem serviço. |
| Processo | `oo-gamingd.exe` separado, Rust puro, sem janela/WebView. Meta: ≤ 5 MB RAM, CPU ≈ 0%. |
| Inicialização | Tarefa Agendada `OpenOptimizer\GamingMode`, gatilho "ao fazer logon", `RunLevel=Highest` → sem UAC a cada boot. Criada ao ligar o Modo Gaming, removida ao desligar e pelo desinstalador. |
| Comunicação app ↔ daemon | Arquivos em `%ProgramData%\OpenOptimizer`: `gaming.json` (config, escrita pela UI) e `gaming-status.json` (estado, escrito pelo daemon). |
| Detecção | Snapshot de processos por nome (`CreateToolhelp32Snapshot`) a cada 3 s — **não abre handle em nenhum processo**. |
| Fonte da lista de jogos | `HKCU\System\GameConfigStore\Children\*\MatchedExeFullPath` (jogos que o próprio Windows detectou) + bibliotecas Steam (`libraryfolders.vdf` + `appmanifest_*.acf`), Epic (`%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item`), Riot e Battle.net quando presentes + adicionados manualmente. |

## 3. Ações do Modo Gaming

| Ação | Quando | Como reverte |
|---|---|---|
| **Timer 0,5 ms** (`NtSetTimerResolution(5000, TRUE)`) | jogo abriu | `NtSetTimerResolution(…, FALSE)` ao fechar; o Windows libera sozinho se o daemon morrer |
| **Plano Desempenho Máximo** | jogo abriu (se o plano ativo não for ele) | reativa o GUID anterior gravado no estado antes de trocar |
| **Limpeza da standby list** (`NtSetSystemInformation(SystemMemoryListInformation, MemoryPurgeStandbyList)`) | durante o jogo, se memória livre < 1024 MB; no máximo 1× a cada 5 min | nada a reverter (o Windows repovoa o cache) |
| **Prioridade alta via IFEO** (`HKLM\…\Image File Execution Options\<exe>\PerfOptions\CpuPriorityClass=3`) | quando o jogo é detectado pela 1ª vez (vale a partir da próxima abertura) | tweak persistente via motor de reversão → aparece e se desfaz no Histórico |

**Pré-requisito no Windows 11:** pedidos de timer de processos sem janela são ignorados sem `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel\GlobalTimerResolutionRequests=1`. Essa chave é aplicada pelo motor de reversão ao ligar o Modo Gaming (tweak `gaming-global-timer`, reversível no Histórico). Sem ela, o timer seria placebo.

## 4. Segurança e reversibilidade

- **Write-ahead em tempo de execução:** antes de trocar o plano de energia, o daemon grava `{previous_scheme, game, started}` em `gaming-session.json`. Ao iniciar, se o arquivo existir (queda/desligamento no meio do jogo), restaura o plano anterior e apaga o arquivo.
- Executáveis do sistema sempre via `oo_sys::windows::system_exe` (System32 absoluto). `schtasks` também.
- O daemon nunca abre handle em processo algum: não usa `OpenProcess`, `QueryFullProcessImageName`, `SetPriorityClass` nem injeção.
- Instância única: mutex nomeado `Global\OpenOptimizerGamingMode`.

## 5. Arquitetura

```
crates/oo-gaming/          lógica pura e testável (sem Windows)
  src/config.rs            GamingConfig { enabled, notify, games: Vec<GameEntry{exe, name, source, enabled}> }
  src/status.rs            GamingStatus { state: Idle|Active, game, since, sessions, purges, purged_mb, seconds_in_game }
  src/session.rs           máquina de estados: on_tick(processos_vistos) → Vec<Effect> (Activate/Deactivate/PurgeIfLow)
  src/sys.rs               trait GamingSys (timer, plano, memória livre, purge, processos) + FakeGamingSys
crates/oo-gaming-win/      impl GamingSys real + descoberta de jogos (GameConfigStore, Steam, Epic…) + schtasks
daemon/                    binário oo-gamingd.exe: loop 3 s, lê config, aplica Effects, escreve status
app/                       comandos: gaming_get, gaming_set_enabled, gaming_set_game, gaming_add_game, gaming_scan
ui/src/routes/Gaming.svelte  tela do Modo Gaming
```

`session.rs` não conhece Windows: recebe o conjunto de nomes de processos e devolve efeitos. Isso permite testar ativação, desativação, troca de jogo, jogo desabilitado e intervalo de purge com `FakeGamingSys`, sem tocar no sistema.

## 6. Esquemas de dados

`gaming.json` (UI escreve, daemon lê a cada tick se a data de modificação mudou):
```json
{ "enabled": true, "notify": true,
  "games": [ { "exe": "VALORANT-Win64-Shipping.exe", "name": "VALORANT", "source": "riot", "enabled": true } ] }
```
`gaming-status.json` (daemon escreve a cada mudança; UI lê a cada 2 s com a janela visível):
```json
{ "state": "Active", "game": "VALORANT-Win64-Shipping.exe", "since": "2026-11-27T18:02:11Z",
  "sessions": 14, "seconds_in_game": 50211, "purges": 3, "purged_mb": 2411, "daemon_version": "0.2.0" }
```
Datas RFC 3339 UTC. Comparação de nomes de exe sem diferenciar maiúsculas.

## 7. Interface (tela Modo Gaming)

- Liga/desliga geral (cria/remove a tarefa agendada + aplica/reverte `gaming-global-timer`).
- Estado ao vivo com a logo: "Aguardando um jogo" ou "Ativo · VALORANT · 12 min" (LogoMark em loop enquanto ativo).
- Lista de jogos com nome, origem (Steam/Epic/Windows/manual) e liga/desliga; botão "Adicionar jogo" (seletor de arquivo `.exe`); botão "Procurar jogos".
- Contadores reais do status: sessões, tempo em jogo, limpezas de memória, MB liberados.
- Toast do Windows ao ativar (opcional, `notify`).
- Card da Visão geral passa a mostrar o estado real (substitui o "Em breve").

## 8. Testes e verificação

- `oo-gaming`: testes unitários da máquina de estados com `FakeGamingSys` (ativa ao ver jogo habilitado; ignora jogo desabilitado; desativa e restaura plano ao fechar; troca de jogo; purge só abaixo do limite e respeitando intervalo; recuperação de sessão interrompida).
- `oo-gaming-win`: testes de integração — parse de `libraryfolders.vdf`/`.acf`/`.item` com arquivos de exemplo; leitura do GameConfigStore real; criar/consultar/remover tarefa agendada de teste.
- Verificação ao vivo (usuário, admin): ligar Modo Gaming → abrir um jogo → `powercfg /getactivescheme` mostra Desempenho Máximo e o status mostra o jogo → fechar → plano anterior volta. RAM do daemon medida com `scripts/measure-memory.ps1` adaptado.

## 9. Critérios de pronto

- [ ] CI verde; testes da máquina de estados cobrindo os 6 cenários do §8.
- [ ] Daemon ≤ 5 MB de RAM e CPU ≈ 0% aguardando (medido).
- [ ] Ciclo abrir/fechar jogo verificado ao vivo, com plano de energia restaurado.
- [ ] Recuperação após desligar no meio de um jogo restaura o plano.
- [ ] Desligar o Modo Gaming remove a tarefa agendada e reverte `gaming-global-timer`.
- [ ] Desinstalar remove a tarefa agendada.
