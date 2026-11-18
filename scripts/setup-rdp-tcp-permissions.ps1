<#
.SYNOPSIS
    Выдать права теневого подключения группам из ТЗ RDS Shadow на RDP-Tcp.

.DESCRIPTION
    Настраивает разрешающие права (Allow) Query Information / Remote Control /
    Message / Logoff на подключении RDP-Tcp для трёх доменных групп:
        RDS-Shadow-View    — Query + Remote Control + Message
        RDS-Shadow-Control — Query + Remote Control
        RDS-Shadow-Logoff  — Query + Logoff

    Скрипт ТОЛЬКО добавляет разрешающие записи (Allow) и НЕ создаёт запретов
    (Deny). Существующие записи (Administrators, SYSTEM, Remote Desktop Users
    и другие) не изменяются.

    Кроме права Remote Control на RDP-Tcp скрипт добавляет группы в локальную
    группу "Remote Desktop Users": mstsc /shadow — это RDP-подключение, и для
    него нужно право входа через Remote Desktop Services. Без этого теневое
    подключение даёт «отказано в доступе», даже если право Remote Control есть.
    Отключить добавление — ключ -SkipRemoteDesktopUsers.

    Запускать на КАЖДОМ узле сеансов (Session Host), НЕ на брокере.
    Требует прав администратора. Без -Server работает локально.

    Примечание про бит Logon (32): метод AddAccount всегда добавляет запись
    с пресетом (PermissionPreSet=0 = Guest), который включает разрешение
    Logon. Скрипт его НЕ убирает и НЕ запрещает, потому что WMI не умеет
    «снять» разрешение без создания запрета, а запрет Logon ломает RDP-вход
    участникам группы. Лишний бит Logon безопасен: без отдельного user right
    «Allow log on through Remote Desktop Services» он не даёт полного входа.

.EXAMPLE
    .\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE"

.EXAMPLE
    .\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE" -Server ts03
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Domain,               # NetBIOS-имя домена (например EXAMPLE)

    [string]$ViewGroup    = "RDS-Shadow-View",
    [string]$ControlGroup = "RDS-Shadow-Control",
    [string]$LogoffGroup  = "RDS-Shadow-Logoff",

    [string]$Server = "",           # пусто = локальный узел

    [switch]$SkipRemoteDesktopUsers  # НЕ добавлять группы в Remote Desktop Users
)

$ErrorActionPreference = "Stop"
$ns = 'root\CIMV2\TerminalServices'

# PermissionMask для Win32_TSAccount.ModifyPermissions (только Allow)
$WINSTATION_QUERY  = 0
$WINSTATION_LOGOFF = 2
$WINSTATION_SHADOW = 4
$WINSTATION_MSG    = 7

$serverArg = @{}
if ($Server) { $serverArg.ComputerName = $Server }

$script:Errors = @()

function Get-TsPermissionsSetting {
    Get-CimInstance @serverArg -Namespace $ns -ClassName Win32_TSPermissionsSetting `
        -Filter "TerminalName='RDP-Tcp'"
}

function Get-TsAccount {
    param([string]$Group)
    Get-CimInstance @serverArg -Namespace $ns -ClassName Win32_TSAccount `
        -Filter "TerminalName='RDP-Tcp' AND AccountName='$Group'"
}

function Add-ToRemoteDesktopUsers {
    param([string]$Group)
    # mstsc /shadow — это RDP-подключение: кроме права Remote Control нужен
    # и доступ на вход через Remote Desktop Services (группа Remote Desktop Users).
    $member = Get-LocalGroupMember -Group "Remote Desktop Users" -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -eq $Group }
    if ($member) {
        Write-Host "   уже в Remote Desktop Users"
        return
    }
    try {
        Add-LocalGroupMember -Group "Remote Desktop Users" -Member $Group -ErrorAction Stop
        Write-Host "   добавлена в Remote Desktop Users"
    } catch {
        $script:Errors += "$Group : не удалось добавить в Remote Desktop Users: $_"
    }
}

function Grant-TsPermissions {
    param(
        [Parameter(Mandatory = $true)][string]$Group,
        [Parameter(Mandatory = $true)][int[]]$PermIds
    )
    Write-Host "== $Group" -ForegroundColor Cyan

    # Создать запись, только если её ещё нет.
    $acct = Get-TsAccount -Group $Group -ErrorAction SilentlyContinue
    if (-not $acct) {
        $tsp = Get-TsPermissionsSetting
        $add = Invoke-CimMethod -InputObject $tsp -MethodName AddAccount `
            -Arguments @{ AccountName = $Group; PermissionPreSet = 0 }
        if ($add.ReturnValue -ne 0) {
            $script:Errors += "$Group : AddAccount вернул $($add.ReturnValue)"
            return
        }
        $acct = Get-TsAccount -Group $Group
        Write-Host "   запись создана"
    } else {
        Write-Host "   запись уже существует"
    }

    # Разрешить нужные операции (только Allow).
    foreach ($id in $PermIds) {
        $r = Invoke-CimMethod -InputObject $acct -MethodName ModifyPermissions `
            -Arguments @{ PermissionMask = $id; Allow = $true }
        if ($r.ReturnValue -ne 0) {
            $script:Errors += "$Group : ModifyPermissions($id) вернул $($r.ReturnValue)"
        }
    }

    $fresh = Get-TsAccount -Group $Group
    Write-Host ("   Allowed = {0}, Denied = {1}" -f $fresh.PermissionsAllowed, $fresh.PermissionsDenied)
    if ($fresh.PermissionsDenied -ne 0) {
        $script:Errors += "$Group : обнаружен запрет (Denied = $($fresh.PermissionsDenied))"
    }
}

Grant-TsPermissions -Group "$Domain\$ViewGroup"    -PermIds @($WINSTATION_QUERY, $WINSTATION_SHADOW, $WINSTATION_MSG)
Grant-TsPermissions -Group "$Domain\$ControlGroup" -PermIds @($WINSTATION_QUERY, $WINSTATION_SHADOW)
Grant-TsPermissions -Group "$Domain\$LogoffGroup"  -PermIds @($WINSTATION_QUERY, $WINSTATION_LOGOFF)

if (-not $SkipRemoteDesktopUsers) {
    Write-Host ""
    Write-Host "Добавление в Remote Desktop Users (для RDP-входа под shadow):" -ForegroundColor Cyan
    Add-ToRemoteDesktopUsers -Group "$Domain\$ViewGroup"
    Add-ToRemoteDesktopUsers -Group "$Domain\$ControlGroup"
    Add-ToRemoteDesktopUsers -Group "$Domain\$LogoffGroup"
}

Write-Host ""
if ($script:Errors.Count -gt 0) {
    Write-Host "ЗАВЕРШЕНО С ОШИБКАМИ:" -ForegroundColor Red
    $script:Errors | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
} else {
    Write-Host "Готово. Denied = 0 у всех трёх групп." -ForegroundColor Green
    Write-Host "Allowed содержит Query(1), и по группе: View +Shadow(16)+Message(128), Control +Shadow(16), Logoff +Logoff(4)."
    Write-Host "Бит Logon(32) остаётся от пресета Guest — это нормально, запретов нет."
}

# Если группа пересоздавалась (SID сменился) и старая запись мешает — удалите её вручную:
#   Get-CimInstance -Namespace root\CIMV2\TerminalServices -ClassName Win32_TSAccount `
#     -Filter "TerminalName='RDP-Tcp'" |
#     Where-Object { $_.AccountName -eq 'DOMAIN\OldGroup' } |
#     ForEach-Object { Invoke-CimMethod -InputObject $_ -MethodName Delete }
