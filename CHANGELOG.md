# Changelog

Todas as mudanças relevantes do Open Optimizer. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/), versões em [SemVer](https://semver.org/lang/pt-BR/).

> As notas de cada Release no GitHub são extraídas automaticamente da seção da versão correspondente. Uma versão sem seção aqui **não é publicada**.

## [0.2.0] - 2026-11-27

### Adicionado
- **Modo Gaming automático**: processo em segundo plano (`oo-gamingd.exe`, ~0,6 MB de RAM) que detecta quando um jogo abre e aplica timer 0,5 ms, plano Desempenho Máximo, limpeza da standby list (quando a memória livre cai abaixo de 1 GB) e prioridade alta; desfaz tudo ao fechar o jogo.
- Detecção sem tocar no jogo: snapshot de processos por nome; nenhum handle é aberto no processo do jogo.
- Descoberta de jogos: lista do próprio Windows (Game Bar), Epic, Steam e adição manual.
- Tela **Modo Gaming** com estado ao vivo, contadores e lista de jogos; card real na Visão geral.
- Configurações → **Animações**: Sempre (padrão) ou Seguir o Windows.

### Corrigido
- Animações quase ausentes em PCs com "Efeitos de animação" desligado no Windows.
- Ícone antigo na barra de tarefas e no Gerenciador de Tarefas após atualizar (cache de ícones do Windows).

### Segurança
- Tarefa agendada do Modo Gaming criada por XML (sem limite de bateria nem das 72 h) e executáveis do sistema sempre por caminho absoluto.

## [0.1.2] - 2026-11-27

### Adicionado
- Identidade visual: nova logo no ícone, instalador, sidebar e bandeja.
- Abertura animada (o ponteiro da logo acelera) e telas "Otimizando" / "Restaurando" com a logo.
- Ícone na bandeja do sistema (Abrir / Otimizar agora / Sair) e pergunta ao fechar: bandeja ou sair.
- Fonte Inter e controles da janela dentro do app (sem barra de título).

## [0.1.1] - 2026-11-27

### Corrigido
- Memória: com a janela minimizada o WebView2 entra em modo de memória baixa (~111 MB → ~23 MB).

## [0.1.0] - 2026-11-27

### Adicionado
- Primeira versão: motor de reversão com journal write-ahead, 5 otimizações (MMCSS, Game DVR, power throttling, Desempenho Máximo, aceleração do mouse), métricas reais, histórico com desfazer, PT-BR/EN e atualização automática assinada.

[0.2.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.0
[0.1.2]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.2
[0.1.1]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.1
[0.1.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.0
