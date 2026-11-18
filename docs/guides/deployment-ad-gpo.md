# Развёртывание в AD и на RDS

Как подготовить домен и ферму RDS, чтобы клиент `rds-shadow` работал и оставался безопасным. Все действия выполняет администратор домена/фермы, не установщик программы.

Права на `RDP-Tcp` можно настроить двумя способами:

- **из самого клиента** — кнопка «Права RDP-Tcp» в GUI (редактор с галочками, аналог `tsconfig.msc`/RDSConfig);
- **через WMI/CIM** — скрипты ниже (штатно, без стороннего ПО).

Клиент не выдаёт права сам по себе: он лишь применяет те же операции WMI, которые сделал бы администратор.

## 1. Три доменные группы

Создать три группы **Global / Security**. Имена ниже — рабочие; замените на принятые в домене и укажите те же имена в INI.

| Группа | Права (галочки) | Что включает клиент |
|--------|------------------|---------------------|
| `RDS-Shadow-View` | Query Information, Remote Control, Message | список, просмотр, сообщение |
| `RDS-Shadow-Control` | Query Information, Remote Control | кнопка управления |
| `RDS-Shadow-Logoff` | Query Information, Logoff | завершение сеанса |

Замечание: View и Control получают одно и то же право `Remote Control`. Разница «смотреть» и «управлять» задаётся флагом `/control` на клиенте и потолком политики ниже. Если управление запрещено политикой узла, оно не сработает ни у кого. Обход кнопки управления прямым `mstsc /control` членом View в v1 принят (см. ТЗ).

PowerShell (на контроллере домена, модуль AD):

```powershell
Import-Module ActiveDirectory
$ou = "OU=Support,DC=example,DC=domain"

New-ADGroup -Name "RDS-Shadow-View"    -GroupScope Global -GroupCategory Security -Path $ou
New-ADGroup -Name "RDS-Shadow-Control" -GroupScope Global -GroupCategory Security -Path $ou
New-ADGroup -Name "RDS-Shadow-Logoff"  -GroupScope Global -GroupCategory Security -Path $ou

Add-ADGroupMember -Identity "RDS-Shadow-View" -Members "a.ivanov", "b.petrov"
Add-ADGroupMember -Identity "RDS-Shadow-Control" -Members "a.ivanov"
Add-ADGroupMember -Identity "RDS-Shadow-Logoff"  -Members "s.sidorov"
```

## 2. Политика «правила удалённого контроля» (потолок и согласие)

GPO на **Session Host** (не на группу):

```
Computer Configuration
└─ Policies
   └─ Administrative Templates
      └─ Windows Components
         └─ Remote Desktop Services
            └─ Remote Desktop Session Host
               └─ Connections
                  └─ "Set rules for remote control of user Remote Desktop Services sessions"
```

Значение: **Enabled**, Options → **Full Control with user's permission**.

- `Full Control` разрешает и просмотр, и управление (разница — флаг `/control` на клиенте).
- `with user's permission` — сотрудник обязательно даёт согласие.
- Если управление в v1 не нужно, ставьте `View Session with user's permission` и не создавайте `RDS-Shadow-Control`.

Применить GPO к OU с компьютерами Session Host.

## 3. Права на `RDP-Tcp`

Право выдаётся на каждом Session Host на подключение `RDP-Tcp`. Два разных набора идентификаторов (не путать):

| Операция | `ModifyPermissions` PermissionMask | Бит в `PermissionsAllowed` |
|----------|-----------------------------------|----------------------------|
| Query Information | 0 | 1 |
| Logoff | 2 | 4 |
| Remote Control (shadow) | 4 | 16 |
| Message | 7 | 128 |
| (Reset / завершение подключения) | 6 | 64 |

Наш редактор в GUI использует `ModifyPermissions` и показывает галочки **Информация / Управление / Сообщение / Завершение**, которые соответствуют `PermissionMask` 0 / 4 / 7 / 2.

Целевые разрешающие биты групп (нужные операции):

| Группа | Нужные биты `PermissionsAllowed` |
|--------|----------------------------------|
| `RDS-Shadow-View` | Query(1) + Shadow(16) + Message(128) |
| `RDS-Shadow-Control` | Query(1) + Shadow(16) |
| `RDS-Shadow-Logoff` | Query(1) + Logoff(4) |

Важно про `Logon` (бит 32): метод `AddAccount` всегда добавляет запись с пресетом (Guest = «только Logon»). Поэтому в `PermissionsAllowed` у наших групп **присутствует и бит Logon (32)**. Это нормально: лишний бит Logon не даёт полноценного RDP-входа без отдельного user right «Allow log on through Remote Desktop Services». Главное — **`PermissionsDenied` должен быть 0**: запретов быть не должно, иначе это блокирует вход участникам группы.

Не ставить запрет Logon через `ModifyPermissions(..., Allow = $false)`: это создаёт Deny, а не «снимает» разрешение, и ломает RDP-вход.

### 3.1 Скрипт (готовый, в репозитории)

Скрипт лежит в проекте: `scripts/setup-rdp-tcp-permissions.ps1`. Запускать на каждом Session Host:

```powershell
# локально на узле
.\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE"

# удалённо с ПК админа (нужны права админа на узле)
.\setup-rdp-tcp-permissions.ps1 -Domain "EXAMPLE" -Server ts03
```

Скрипт только добавляет разрешающие записи (Allow), не создаёт запретов (Deny), проверяет обе маски (`Allowed` и `Denied`) и печатает результат. `-Force` в скрипте нет: при смене SID группы старую запись удаляют вручную (команда в конце скрипта и в разделе «Откат»).

### 3.2 Скрипт вручную (эквивалент, если нужно понять механику)

```powershell
param([string]$Server = $env:COMPUTERNAME)

function Add-TsAccountAndPerms {
    param([string]$Group, [int[]]$PermIds)

    $ns = 'root\CIMV2\TerminalServices'
    $cim = @{ Namespace = $ns; ClassName = 'Win32_TSAccount'; Filter = "TerminalName='RDP-Tcp'" }
    if ($Server) { $cim.ComputerName = $Server }

    # Создать запись, только если её ещё нет.
    $acct = Get-CimInstance @cim -Filter "TerminalName='RDP-Tcp' AND AccountName='$Group'" -ErrorAction SilentlyContinue
    if (-not $acct) {
        $tsp = Get-CimInstance -ComputerName $Server -Namespace $ns `
                -ClassName Win32_TSPermissionsSetting -Filter "TerminalName='RDP-Tcp'"
        $r = Invoke-CimMethod -InputObject $tsp -MethodName AddAccount `
                -Arguments @{ AccountName = $Group; PermissionPreSet = 0 }
        if ($r.ReturnValue -ne 0) { throw "AddAccount $Group : $($r.ReturnValue)" }
        $acct = Get-CimInstance @cim -Filter "TerminalName='RDP-Tcp' AND AccountName='$Group'"
    }

    # Разрешить нужные операции (только Allow, без Deny).
    foreach ($id in $PermIds) {
        $r = Invoke-CimMethod -InputObject $acct -MethodName ModifyPermissions `
                -Arguments @{ PermissionMask = $id; Allow = $true }
        if ($r.ReturnValue -ne 0) { throw "ModifyPermissions($id) $Group : $($r.ReturnValue)" }
    }
}

# View:   Query(0) + Remote Control(4) + Message(7)
Add-TsAccountAndPerms "EXAMPLE\RDS-Shadow-View"    @(0, 4, 7)
# Control: Query(0) + Remote Control(4)
Add-TsAccountAndPerms "EXAMPLE\RDS-Shadow-Control" @(0, 4)
# Logoff:  Query(0) + Logoff(2)
Add-TsAccountAndPerms "EXAMPLE\RDS-Shadow-Logoff"  @(0, 2)
```

Для нескольких узлов вызвать с `-Server ts01` и т.д., либо обернуть в цикл. Права на `RDP-Tcp` не разносятся репликацией — на каждом узле свои.

Ручная альтернатива (GUI): кнопка **«Права RDP-Tcp»** в клиенте `rds-shadow.exe` — то же самое окно с галочками, без консоли.

## 4. Сеть

- `mstsc /shadow` идёт на узел по **3389/TCP**.
- Перечисление сеансов и действия через WTS API — RPC: **135/TCP** + динамический диапазон RPC.
- WMI для настройки прав — те же RPC-порты, с `packet privacy` (WMI/CIM сам устанавливает нужный уровень аутентификации).

## 5. Безопасность применения и откат

Скрипт **аддитивен**: он только добавляет записи трёх групп и не меняет ничего у существующих учёток (`Administrators`, `SYSTEM`, `Remote Desktop Users` и др.). Он не трогает порт, сертификаты, параметры `RDP-Tcp`, службу `Remote Desktop Services` и GPO.

- Работает с тем же security descriptor `RDP-Tcp`, который читала бы `tsconfig.msc`; новые записи — обычные ACE, стандартную оснастку не ломают.
- В Windows Server 2012+ прав shadow/message/logoff в GUI Server Manager **нет** (убраны вместе с `tsconfig.msc`), поэтому конфликта с GUI нет: скрипт и редактор в exe — замена убранной оснастки.
- Скрипт **не устанавливает запрет Logon** и вообще не создаёт Deny: `AddAccount` добавляет пресет Guest (бит Logon в Allowed), который остаётся как есть. Запретов нет, вход участникам групп не блокируется.
- Скрипт не имеет `-Force` и ничего не удаляет.

### Снапшот до изменения (на всякий случай)

```powershell
Get-CimInstance -Namespace root\CIMV2\TerminalServices -ClassName Win32_TSAccount `
  -Filter "TerminalName='RDP-Tcp'" |
  Select-Object AccountName, SID, PermissionsAllowed, PermissionsDenied | Export-Csv .\rdp-tcp-snapshot.csv
```

### Откат (удалить только наши группы)

```powershell
Get-CimInstance -Namespace root\CIMV2\TerminalServices -ClassName Win32_TSAccount `
  -Filter "TerminalName='RDP-Tcp'" |
  Where-Object { $_.AccountName -like '*RDS-Shadow*' } |
  ForEach-Object { Invoke-CimMethod -InputObject $_ -MethodName Delete }
```

После отката состояние `RDP-Tcp` возвращается к исходному — удаляются только записи трёх групп.

## 6. Диагностика «error 5» (Access Denied)

`error 5` = `ERROR_ACCESS_DENIED`. Признак: список сеансов виден, а shadow/сообщение/logoff — нет. Значит у группы есть **Query Information**, но нет Remote Control / Message / Logoff.

Проверить действующие битовые маски на Session Host:

```powershell
Get-CimInstance -Namespace root\CIMV2\TerminalServices `
  -ClassName Win32_TSAccount -Filter "TerminalName='RDP-Tcp'" |
  Select-Object AccountName, SID, PermissionsAllowed, PermissionsDenied,
      @{Name='LogonDenied'; Expression={ ($_.PermissionsDenied -band 32) -ne 0 }}
```

Правильное состояние: нужные биты в `PermissionsAllowed` (Query/Shadow/Message/Logoff по группе), **`PermissionsDenied = 0`**, `LogonDenied = False`. Если `LogonDenied = True` — где-то стоит запрет Logon, его нужно снять, иначе участники группы не смогут заново войти по RDP.

Снять запрет Logon (выдать разрешение, а не оставить запрет):

```powershell
$acct = Get-CimInstance -Namespace root\CIMV2\TerminalServices -ClassName Win32_TSAccount `
  -Filter "TerminalName='RDP-Tcp' AND AccountName='EXAMPLE\RDS-Shadow-View'"
Invoke-CimMethod -InputObject $acct -MethodName ModifyPermissions `
  -Arguments @{ PermissionMask = 5; Allow = $true }
```

Частые причины, что права «не применились»:

1. Настройка выполнялась на контроллере домена, а не на Session Host.
2. `AccountName` — локальная группа, а не доменная `DOMAIN\Group`.
3. Группа пересоздана после выдачи → SID сменился, старый ACE остался. Удалить запись (`Win32_TSAccount.Delete`) и добавить заново.
4. Права не продублированы на всех узлах.
5. Политика remote control стоит `Not allowed`.

Тест вести обычной учёткой из `RDS-Shadow-View`, а не админом (у админа UAC-отфильтрованный токен без прав `RDP-Tcp`). После изменения прав, если `error 5` остаётся, перезапустить службу `Remote Desktop Services` и перезайти.

## 7. Проверка

От имени специалиста из группы View на его ПК:

```bat
query session /server:ts01
mstsc /v:ts01 /shadow:<ID>
```

Подключение проверять только с согласия добровольца, не на чужом сеансе. Управление и logoff — на сеансе, владелец которого ждёт этого.

## 8. Заполнить INI

Скопировать `rds-shadow.example.ini` в `rds-shadow.ini` и указать те же имена групп:

```ini
[connection]
mode = servers
rdp_port = 3389

[servers]
list = ts01,ts02,ts03

[groups]
view = EXAMPLE\RDS-Shadow-View
control = EXAMPLE\RDS-Shadow-Control
logoff = EXAMPLE\RDS-Shadow-Logoff

[audit]
log_dir = C:\ProgramData\rds-shadow\logs
share_dir =

[ui]
language = auto
```

Если группа не указана или специалист не входит в неё — кнопка в клиенте скрыта/недоступна.
