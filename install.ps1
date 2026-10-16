# Сборка и установка «Боевых шахмат»: программа ставится в
# %LOCALAPPDATA%\Programs\BattleChess, ярлык — на рабочий стол.
# Повторный запуск обновляет установленную версию. Открытую игру закрывать
# не нужно: новая версия запустится со следующего раза.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

# Терминалы, открытые до установки Rust, ещё не видят его в PATH.
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path $cargoBin)) {
    $env:Path = "$cargoBin;$env:Path"
}

cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'Сборка не удалась, см. сообщения выше.' }

$installDir = Join-Path $env:LOCALAPPDATA 'Programs\BattleChess'
$exe = Join-Path $installDir 'battle-chess.exe'
New-Item -ItemType Directory -Force $installDir | Out-Null

# Запущенный exe нельзя перезаписать, но можно переименовать: старая версия
# спокойно доиграет партию, а её файл удалится при следующем обновлении.
Get-ChildItem $installDir -Filter '*.old' | Remove-Item -ErrorAction SilentlyContinue
if (Test-Path $exe) {
    Rename-Item $exe ('battle-chess.{0:yyyyMMddHHmmss}.old' -f (Get-Date))
}
Copy-Item 'target\release\battle-chess.exe' $exe
Copy-Item 'assets\battle-chess.ico' $installDir -Force
# Если игра не запущена, старая версия больше не нужна.
Get-ChildItem $installDir -Filter '*.old' | Remove-Item -ErrorAction SilentlyContinue

$desktop = [Environment]::GetFolderPath('Desktop')
$shell = New-Object -ComObject WScript.Shell
$link = $shell.CreateShortcut((Join-Path $desktop 'Боевые шахматы.lnk'))
$link.TargetPath = $exe
$link.WorkingDirectory = $installDir
$link.IconLocation = (Join-Path $installDir 'battle-chess.ico') + ',0'
$link.Description = 'Боевые шахматы: у фигур есть атака, защита и здоровье'
$link.Save()

$version = (Select-String -Path 'Cargo.toml' -Pattern '^version\s*=\s*"(.+)"').Matches[0].Groups[1].Value
Write-Host "Готово: «Боевые шахматы» $version установлены в $installDir, ярлык — на рабочем столе."
