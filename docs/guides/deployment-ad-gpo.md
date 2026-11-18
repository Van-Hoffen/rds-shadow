# Развёртывание в AD и на RDS

Как подготовить домен и ферму RDS, чтобы клиент `rds-shadow` работал и оставался безопасным. Все действия выполняет администратор домена/фермы, не установщик программы.

Клиент не выдаёт права. Разрешает или запрещает Windows.

## 1. Три доменные группы

Создать три группы **Global / Security**. Имена ниже — рабочие; замените на принятые в домене и укажите те же имена в INI.

| Группа | Право на RDP-Tcp | Что включает клиент |
|--------|------------------|---------------------|
| `RDS-Shadow-View` | Query + Remote Control + Message | список, просмотр, сообщение |
| `RDS-Shadow-Control` | Query + Remote Control | кнопка управления |
| `RDS-Shadow-Logoff` | Query + Logoff | завершение сеанса |

Замечание: View и Control получают одно и то же право `Remote Control`. Разница «смотреть» и «управлять» задаётся флагом `/control` на клиенте и потолком политики ниже. Если управление запрещено политикой узла, оно не сработает ни у кого. Обход кнопки управления прямым `mstsc /control` членом View в v1 принят (см. ТЗ).

PowerShell (на контроллере домена, модуль AD):

```powershell
Import-Module ActiveDirectory
$ou = "OU=Support,DC=example,DC=domain"   # OU, где живут группы

New-ADGroup -Name "RDS-Shadow-View"    -GroupScope Global -GroupCategory Security -Path $ou
New-ADGroup -Name "RDS-Shadow-Control" -GroupScope Global -GroupCategory Security -Path $ou
New-ADGroup -Name "RDS-Shadow-Logoff"  -GroupScope Global -GroupCategory Security -Path $ou

# Включить специалистов
Add-ADGroupMember -Identity "RDS-Shadow-View" -Members "a.ivanov", "b.petrov"
Add-ADGroupMember -Identity "RDS-Shadow-Control" -Members "a.ivanov"
Add-ADGroupMember -Identity "RDS-Shadow-Logoff"  -Members "s.sidorov"
```

## 2. Политика «правила удалённого контроля» (потолок и согласие)

Это потолок для всех сеансов узла и требование согласия сотрудника. Настраивается GPO на **Session Host**, не на группу.

Путь в GPME:

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

- `Full Control` — разрешает и просмотр, и управление (на клиенте разница через `/control`).
- `with user's permission` — сотрудник обязательно даёт согласие. Без этого пункта согласие останется только текстом в ТЗ, а не поведением ОС.
- Если управление в v1 не нужно вообще, ставьте `View Session with user's permission`, а группа `RDS-Shadow-Control` не создаётся.

Применить GPO к OU, где лежат компьютеры Session Host (`RSOP/GPRESULT` для проверки).

## 3. Права на `RDP-Tcp` (кто вообще может shadow/query/logoff/message)

Право выдаётся на каждом Session Host на подключение `RDP-Tcp`. Маски — стандарт WINSTATION:

| Право | Бит (hex) | Десятично |
|-------|-----------|-----------|
| Query Information | 0x0001 | 1 |
| Remote Control (shadow) | 0x0010 | 16 |
| Logoff | 0x0040 | 64 |
| Message | 0x0080 | 128 |

Итоговые маски групп:

| Группа | Маска |
|--------|-------|
| `RDS-Shadow-View` | Query + Shadow + Message = **145** |
| `RDS-Shadow-Control` | Query + Shadow = **17** |
| `RDS-Shadow-Logoff` | Query + Logoff = **65** |

Выдать права на каждом Session Host (PowerShell, от имени администратора):

```powershell
$tsp = Get-CimInstance -Namespace root\CIMV2\TerminalServices `
        -ClassName Win32_TSPermissionsSetting -Filter "TerminalName='RDP-Tcp'"

Invoke-CimMethod -InputObject $tsp -MethodName AddAccount -Arguments @{
    AccountName     = "EXAMPLE\RDS-Shadow-View"
    PermissionPreSet = 0        # 0 = WINSTATION_GUEST_ACCESS: использовать PermissionMask
    PermissionMask   = 145
}
Invoke-CimMethod -InputObject $tsp -MethodName AddAccount -Arguments @{
    AccountName     = "EXAMPLE\RDS-Shadow-Control"
    PermissionPreSet = 0
    PermissionMask   = 17
}
Invoke-CimMethod -InputObject $tsp -MethodName AddAccount -Arguments @{
    AccountName     = "EXAMPLE\RDS-Shadow-Logoff"
    PermissionPreSet = 0
    PermissionMask   = 65
}
```

Для нескольких узлов обернуть в цикл:

```powershell
$servers = "ts01", "ts02", "ts03"
foreach ($s in $servers) {
    $tsp = Get-CimInstance -ComputerName $s -Namespace root\CIMV2\TerminalServices `
            -ClassName Win32_TSPermissionsSetting -Filter "TerminalName='RDP-Tcp'"
    # … те же три Invoke-CimMethod …
}
```

Ручная альтернатива (GUI, один узел): `tsconfig.msc` → Connections → `RDP-Tcp` → Permissions → добавить группу и отметить нужные права (Query, Remote Control, Message, Logoff).

## 4. Сеть

- `mstsc /shadow` идёт на узел по **3389/TCP** — уже открыт для фермы.
- Перечисление сеансов и действия через WTS API идут по RPC: **135/TCP** плюс динамический диапазон RPC. Для специалистов на ПК должны быть открыты эти порты до Session Host (обычно уже открыты в доменной сети; проверить, если список пустой).
- WMI/CIM для выдачи прав в шаге 3 — те же RPC-порты.

## 5. Диагностика «error 5» (Access Denied)

`error 5` = `ERROR_ACCESS_DENIED`. Обычный признак: список сеансов виден, а shadow/сообщение/logoff — нет. Это значит, что группе на `RDP-Tcp` выдано право **Query Information (1)**, но не Remote Control / Message / Logoff.

Проверить действующий security descriptor на **Session Host** (не на контроллере домена):

```powershell
$tsp = Get-CimInstance -Namespace root\CIMV2\TerminalServices `
        -ClassName Win32_TSPermissionsSetting -Filter "TerminalName='RDP-Tcp'"
$r = Invoke-CimMethod -InputObject $tsp -MethodName GetSecurityDescriptor
$r.Descriptor.DACL | ForEach-Object {
    "{0,-32} mask={1}" -f $_.Trustee.Name, $_.AccessMask
}
```

Должно быть три строки с масками `145`, `17`, `65`. Если только `1` — выдайте маски заново (шаг 3).

Частые причины, что права «не применились»:

1. `AddAccount` выполнен не на Session Host, а на контроллере домена.
2. `AccountName` — локальная группа с тем же именем, а не доменная `DOMAIN\Group`.
3. Группа пересоздана после выдачи → SID поменялся, старый ACE висит на старом SID. Удалить (`RemoveAccount`) и добавить заново.
4. Права на `RDP-Tcp` не разносятся репликацией: выдать на **каждом** узле.
5. Политика remote control стоит `Not allowed` — тогда shadow не работает даже с правами.

Тест вести **обычной учёткой** из `RDS-Shadow-View`, а не админом: у админа UAC-отфильтрованный токен без прав `RDP-Tcp`, и это маскирует результат. После `AddAccount`, если `error 5` остаётся, перезапустить службу `Remote Desktop Services` и перезайти.

## 6. Проверка

От имени специалиста из группы View на его ПК:

```bat
query session /server:ts01
qwinsta /server:ts01
```

Если список виден — Query Information работает. Подключение (только с согласия добровольца, не на чужом сеансе):

```bat
mstsc /v:ts01 /shadow:<ID>
```

Проверить, что сотруднику пришёл запрос согласия. `logoff` и управление проверять только на сеансе, владелец которого ждёт этого.

Проверка действующих прав на узле:

```powershell
(Get-CimInstance -Namespace root\CIMV2\TerminalServices `
  -ClassName Win32_TSPermissionsSetting -Filter "TerminalName='RDP-Tcp'").SecurityDescriptor
```

## 7. Заполнить INI

Скопировать `rds-shadow.example.ini` в `rds-shadow.ini` и указать **те же** имена групп:

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

Если группа не указана или специалист не входит в неё — соответствующая кнопка в клиенте скрыта/недоступна, действие не стартует.
