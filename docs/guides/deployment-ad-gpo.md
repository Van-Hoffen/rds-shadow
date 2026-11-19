# Развёртывание в AD и на RDS

Как подготовить домен и ферму RDS, чтобы клиент `rds-shadow` работал и оставался безопасным. Все действия выполняет администратор домена/фермы, не установщик программы.

Настройка прав `RDP-Tcp` **вынесена из продукта** (ТЗ §1: «Программа не выдаёт права. Разрешение или отказ делает Windows»). Механика (WMI, скрипт, откат, диагностика) сохранена в `archive/rdp-tcp-permissions/` — в текущие версии не входит. Ниже — что нужно подготовить, чтобы клиент работал.

## 1. Три доменные группы

Создать три группы **Global / Security**. Имена ниже — рабочие; замените на принятые в домене и укажите те же имена в INI.

| Группа | Нужные права на `RDP-Tcp` | Что включает клиент |
|--------|---------------------------|---------------------|
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

## 3. Права на `RDP-Tcp` — вне текущего продукта

Клиент прав не выдаёт. Чтобы `mstsc /shadow` работал, каждой группе нужны разрешающие записи на подключении `RDP-Tcp` **каждого** Session Host (права не разносятся репликацией):

| Группа | Нужные биты `PermissionsAllowed` |
|--------|----------------------------------|
| `RDS-Shadow-View` | Query(1) + Shadow(16) + Message(128) |
| `RDS-Shadow-Control` | Query(1) + Shadow(16) |
| `RDS-Shadow-Logoff` | Query(1) + Logoff(4) |

Важно про `Logon` (бит 32): метод `AddAccount` добавляет запись с пресетом Guest (бит Logon в `Allowed`). Это нормально: лишний бит Logon не даёт RDP-входа без отдельного user right. Главное — **`PermissionsDenied` должен быть 0**: запретов быть не должно, иначе это блокирует вход участникам группы. Запрет Logon через `ModifyPermissions(..., Allow = $false)` не ставить — это создаёт Deny и ломает RDP-вход.

Право входа (Remote Desktop Users): права Remote Control недостаточно — `mstsc /shadow` это RDP-подключение, группе нужно ещё членство в локальной **«Пользователи удалённого рабочего стола» (Remote Desktop Users)** на Session Host. Без него — «отказано в доступе» даже при выданном Remote Control.

Готовый скрипт, WMI-механика, безопасность/откат и диагностика `error 5` — в архиве: `archive/rdp-tcp-permissions/` (там же `deployment-ad-gpo.full.md` и `setup-rdp-tcp-permissions.ps1`).

## 4. Сеть

- `mstsc /shadow` идёт на узел по **3389/TCP**.
- Перечисление сеансов и действия через WTS API — RPC: **135/TCP** + динамический диапазон RPC.
- WMI для настройки прав (если настраивать вручную) — те же RPC-порты, с `packet privacy` (WMI/CIM сам устанавливает нужный уровень аутентификации).

## 5. Проверка

От имени специалиста из группы View на его ПК:

```bat
query session /server:ts01
mstsc /v:ts01 /shadow:<ID>
```

Подключение проверять только с согласия добровольца, не на чужом сеансе. Управление и logoff — на сеансе, владелец которого ждёт этого.

## 6. Заполнить INI

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
