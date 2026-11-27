; Hooks do instalador NSIS (Tauri). Executáveis do sistema por $SYSDIR (caminho absoluto).

!macro NSIS_HOOK_PREINSTALL
  ; O daemon do Modo Gaming trava o próprio .exe: encerra antes de copiar a versão nova.
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM oo-gamingd.exe'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Se o Modo Gaming estava ligado, a tarefa existe: reinicia o daemon atualizado.
  nsExec::Exec '"$SYSDIR\schtasks.exe" /Run /TN "OpenOptimizer\GamingMode"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec '"$SYSDIR\taskkill.exe" /F /IM oo-gamingd.exe'
  nsExec::Exec '"$SYSDIR\schtasks.exe" /Delete /TN "OpenOptimizer\GamingMode" /F'
!macroend
