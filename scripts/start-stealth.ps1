# Stealth запуск Eye Remote Bot без окон
$env:EYE_STEALTH = "1"

# Запускаем в скрытом режиме без создания окна
$processInfo = New-Object System.Diagnostics.ProcessStartInfo
$processInfo.FileName = "$PSScriptRoot\eye-enhanced.exe"
$processInfo.UseShellExecute = $false
$processInfo.CreateNoWindow = $true
$processInfo.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden

$process = [System.Diagnostics.Process]::Start($processInfo)

# Скрипт завершается, процесс работает в фоне
Write-Host "Bot started in stealth mode (PID: $($process.Id))"