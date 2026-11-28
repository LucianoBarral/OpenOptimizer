# Subprojeto 2b — FPS medido nas sessões do Modo Gaming

Status: aprovado pelo usuário em 2026-11-27 ("pode fazer"). Escolha do usuário: medição **automática** nas sessões do Modo Gaming (sem botão manual, sem overlay).

## Objetivo

Provar com números se as otimizações ajudam: para cada sessão de jogo, gravar FPS médio, 1% low, 0,1% low e frametime (p50/p99), e mostrar no app a evolução por jogo e a comparação antes → depois de cada mudança de otimização.

## Restrições

- **Anti-ban:** nada de overlay, injeção, `OpenProcess` ou leitura de memória do jogo. Só ETW (eventos de apresentação de frames do Windows), via PresentMon — a mesma técnica do NVIDIA FrameView e CapFrameX.
- **Leve:** o daemon segue pequeno; memória constante durante a sessão (histograma fixo, sem guardar frame a frame).
- **Honesto:** nada de comparação inventada. Sessões curtas não entram; sem sessão "antes", o app diz isso.

## Fonte dos frames: PresentMon (Intel, MIT)

- Versão fixa **v2.6.0**, arquivo `PresentMon-2.6.0-x64.exe` (980 320 bytes), SHA-256 `b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af`, assinado por Intel Corporation (Authenticode válido, conferido em 2026-11-27).
- Baixado no build por `scripts/fetch-presentmon.mjs` com verificação do SHA-256 (falha se não bater) e empacotado como `externalBin` (`binaries/presentmon` → `presentmon.exe` ao lado do app e do daemon). Licença em `THIRD_PARTY_NOTICES.md`.
- Linha de comando:
  `presentmon.exe --process_name <jogo.exe> --output_stdout --v1_metrics --no_console_stats --no_track_input --no_track_gpu --session_name OpenOptimizerFps --stop_existing_session --terminate_on_proc_exit`
- Formato v1 (conferido no código-fonte v2.6.0, `CsvOutput.cpp`): cabeçalho
  `Application,ProcessID,SwapChainAddress,Runtime,SyncInterval,PresentFlags,Dropped,TimeInSeconds,msInPresentAPI,msBetweenPresents,…`.
  O parser localiza as colunas pelo nome no cabeçalho (`SwapChainAddress`, `Runtime`, `TimeInSeconds`, `msBetweenPresents`).

## Arquitetura

1. **`crates/oo-fps`** (puro, testável sem Windows):
   - `CsvReader`: recebe linhas; a primeira é o cabeçalho; devolve `Frame { swapchain, runtime, t_s, ms }`.
   - `FrameStats`: um histograma por swap chain (buckets de 0,05 ms até 250 ms + estouro), soma de ms e contagem. Ignora frames com `t_s < primeiro_t + 15` (aquecimento).
   - `summary()` usa a swap chain com mais frames e calcula: `avg_fps = frames / soma_ms × 1000`, `low1_fps = 1000 / p99`, `low01_fps = 1000 / p99.9`, `ft_p50_ms`, `ft_p99_ms`, `measured_s = soma_ms / 1000`.
   - `FpsRecord` (serde) = sumário + `game`, `started` (unix s), `duration_s`. `valid = measured_s ≥ 60`.
   - `append_history(path, record)`: JSONL; acima de 600 linhas reescreve com as 500 mais recentes.
2. **`oo-gaming-win::fps`**: `FpsCapture::start(presentmon, exe)` sobe o processo (sem janela, stdout em pipe) e uma thread que alimenta `FrameStats`; `finish()` espera até 5 s o PresentMon sair sozinho (ele sai quando o jogo fecha), senão encerra o processo, e devolve o sumário. Se o `presentmon.exe` não existir ou falhar, devolve `None` — a sessão do Modo Gaming segue normal.
3. **Daemon:** compara `status.game` antes/depois de cada `tick`: jogo novo → `start`; jogo saiu/trocou → `finish` + `append_history` em `%ProgramData%\OpenOptimizer\fps-history.jsonl`. No desligamento do Modo Gaming, finaliza a captura em andamento.
4. **App:** comando `fps_history` lê o JSONL (registros inválidos/corrompidos são ignorados linha a linha).
5. **UI (tela Modo Gaming):** seção "Desempenho medido" por jogo: última sessão (FPS médio, 1% low), sparkline do FPS médio das sessões e card **Antes → Depois**. As mudanças de otimização vêm do Histórico (`history`, com data): a comparação usa a última mudança que caiu entre sessões válidas do jogo; "antes" = mediana das até 3 sessões válidas anteriores, "depois" = mediana das até 3 posteriores. Sem sessões dos dois lados, mostra o motivo.

## Erros e casos de borda

- PresentMon ausente/sem permissão → sem registro; status normal. (O daemon já roda elevado pela tarefa agendada.)
- Jogo com várias swap chains (launcher/overlay) → vale a de mais frames.
- Daemon morto no meio da sessão → a sessão de ETW fica órfã; a próxima captura usa `--stop_existing_session`.
- Linha CSV inválida → ignorada.

## Testes

- `oo-fps`: parser (cabeçalho, colunas fora de ordem, linha inválida), aquecimento de 15 s, percentis com frametimes sintéticos conhecidos, escolha da swap chain, `valid`, poda do histórico.
- UI: comparação antes/depois (`fpsCompare.ts`) com dados sintéticos.
- Medição real exige um jogo e admin: verificação manual do usuário (números batem com o que ele vê).
