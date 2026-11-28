; Hooks do instalador NSIS (Tauri). Executáveis do sistema por $SYSDIR (caminho absoluto).

!macro NSIS_HOOK_PREINSTALL
  ; O daemon do Modo Gaming trava o próprio .exe: encerra antes de copiar a versão nova.
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM oo-gamingd.exe'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Se o Modo Gaming estava ligado, a tarefa existe: reinicia o daemon atualizado.
  nsExec::Exec '"$SYSDIR\schtasks.exe" /Run /TN "OpenOptimizer\GamingMode"'
  ; O Windows guarda ícones em cache pelo caminho do .exe; sem isto a barra de tarefas
  ; e o Gerenciador de Tarefas continuam mostrando o ícone antigo após uma atualização.
  nsExec::Exec '"$SYSDIR\ie4uinit.exe" -show'
  ; ie4uinit sozinho não bastou (medido na v0.2.1): avisa o shell que o ícone deste .exe
  ; mudou (SHCNE_UPDATEITEM, SHCNF_PATHW|SHCNF_FLUSH) e que associações mudaram.
  System::Call 'shell32::SHChangeNotify(i 0x2000, i 0x1005, w "$INSTDIR\open-optimizer.exe", p 0)'
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x1000, p 0, p 0)'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM oo-gamingd.exe'
  nsExec::Exec '"$SYSDIR\schtasks.exe" /Delete /TN "OpenOptimizer\GamingMode" /F'
!macroend
