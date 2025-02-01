$folderPath = "C:\Program Files (x86)\smokingplaya\puff"
if (-Not (Test-Path -Path $folderPath)) {
  New-Item -Path $folderPath -ItemType Directory
}

# downloading file
$exeUrl = "https://github.com/smokingplaya/puff/releases/latest/download/puff-win_x86.exe"
$exePath = Join-Path -Path $folderPath -ChildPath "puff.exe"
Invoke-WebRequest -Uri $exeUrl -OutFile $exePath

# add file in path
$currentPath = [System.Environment]::GetEnvironmentVariable("Path", [System.EnvironmentVariableTarget]::Machine)
if (-Not ($currentPath -like "*$folderPath*")) {
  [System.Environment]::SetEnvironmentVariable("Path", $currentPath + ";$folderPath", [System.EnvironmentVariableTarget]::Machine)
}

Write-Host "Installation completed"