# Open Optimizer

Otimizador de PC para Windows 10/11, **gratuito e open source**. Leve (Rust + Tauri), com interface minimalista e **toda mudança reversível**.

## Princípios

- **Tudo reversível.** Antes de alterar qualquer coisa, o valor original é lido do sistema e gravado em disco (journal write-ahead). Dá para desfazer por item, por lote ou tudo de uma vez — inclusive se o app fechar no meio de uma otimização.
- **Sem risco de ban.** O Open Optimizer **nunca** abre, injeta, lê ou escreve na memória de processos de jogos. Ele só ajusta o Windows (registro, energia, MMCSS) por APIs públicas — nada que anti-cheats (Vanguard, EAC, BattlEye) monitorem.
- **Sem placebo.** Cada otimização tem uma justificativa técnica visível no app (“Por quê?”).
- **Dados reais.** Todo número exibido vem de leitura real do sistema.

## Otimizações disponíveis (v0.1)

| Otimização | Efeito |
|---|---|
| Prioridade MMCSS para jogos | Threads do jogo antes de tarefas em segundo plano |
| Desativar gravação em segundo plano (Game DVR) | Libera o encoder de vídeo da GPU |
| Desativar power throttling | Evita redução de clock de processos ativos |
| Plano Desempenho Máximo | Remove microlatências de troca de estado de energia |
| Desativar aceleração do mouse | Mesmo movimento físico, mesma distância na tela |

Modo Gaming, limpeza de memória, atualizador de drivers e apps/ajustes estilo winutil chegam nas próximas versões.

## Instalar

Baixe o instalador em [Releases](https://github.com/LucianoBarral/OpenOptimizer/releases). O app pede permissão de administrador (necessária para os ajustes) e se atualiza sozinho pelas Releases assinadas.

## Compilar

Requisitos: Rust estável, Node 24.

```bash
npm i && npm --prefix ui i
npm run tauri dev      # terminal como administrador (o app exige elevação)
cargo test --workspace --exclude open-optimizer   # testes sem precisar de admin
```

Estrutura: `crates/oo-core` (motor de reversão), `crates/oo-sys` (acesso ao Windows), `crates/oo-catalog` (tweaks em TOML), `crates/oo-metrics` (métricas), `crates/oo-views` (contrato IPC), `app/` (Tauri), `ui/` (Svelte 5).

## Licença

MIT
