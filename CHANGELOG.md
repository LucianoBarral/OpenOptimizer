# Changelog

Todas as mudanças relevantes do Open Optimizer. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/), versões em [SemVer](https://semver.org/lang/pt-BR/).

> As notas de cada Release no GitHub e as notas dentro do app vêm da seção da versão correspondente, aqui e no [CHANGELOG.en.md](CHANGELOG.en.md) (inglês). Os dois precisam ter as mesmas versões e itens (um teste confere); uma versão sem seção em qualquer um deles **não é publicada**.

## [0.3.0] - 2026-11-28

### Adicionado
- **FPS medido nas sessões do Modo Gaming**: FPS médio, 1% low, 0,1% low e frametime de cada sessão, gravados automaticamente. Os primeiros 15 s (carregamento) ficam de fora e sessões com menos de 1 minuto medido não entram em comparações.
- Seção **Desempenho medido** na tela Modo Gaming: última sessão, evolução por jogo e **antes → depois** da última mudança de otimização (mediana de até 3 sessões de cada lado). Sem sessão "antes", o app diz isso em vez de inventar um número.

### Segurança
- A medição usa o PresentMon 2.6.0 da Intel (MIT, assinado pela Intel), que só escuta eventos do Windows (ETW): nada é injetado nem aberto no processo do jogo. A versão é fixa e o SHA-256 é conferido no build.

## [0.2.2] - 2026-11-27

### Adicionado
- **Notas de versão no app**, em português e inglês: depois de atualizar, uma janela mostra o que mudou; a lista completa fica em Configurações → Sobre.
- As novidades no aviso de atualização agora aparecem formatadas, em vez de texto cru.

### Alterado
- Configurações foi para o pé da barra lateral, separada das funções.
- Aviso de atualização com mais espaço em relação aos controles da janela.

### Corrigido
- O instalador agora avisa o Windows que o ícone do app mudou, para a barra de tarefas não manter o ícone antigo após atualizar.

## [0.2.1] - 2026-11-27

### Corrigido
- Barra de rolagem padrão do Windows (branca e grossa) trocada por uma fina e escura, no tema do app.
- O conteúdo rolava por baixo da faixa invisível do topo e dos botões minimizar/maximizar/fechar; agora a faixa é fixa e o conteúdo esmaece ao chegar nela.
- Barra de tarefas mostrava o ícone antigo (asterisco da v0.1.0): a janela só tinha o ícone pequeno definido e o Windows buscava o grande num cache desatualizado. Agora o app define o ícone grande a partir do próprio executável.

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

[0.3.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.3.0
[0.2.2]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.2
[0.2.1]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.1
[0.2.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.0
[0.1.2]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.2
[0.1.1]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.1
[0.1.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.0
