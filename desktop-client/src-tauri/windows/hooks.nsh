; Always create a desktop shortcut (Tauri default only offers an opt-in checkbox on the finish page).
!macro NSIS_HOOK_POSTINSTALL
  Call CreateOrUpdateDesktopShortcut
!macroend
