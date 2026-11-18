use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use rds_shadow_core::config::{Config, Language};
use rds_shadow_core::event::{ActionMode, AuditEvent, EventKind, ResultKind};

use crate::audit;
use crate::platform;
use crate::rights::{self, Perm, PermAccount};

enum AppEvent {
    Sessions(Vec<platform::Session>, Vec<String>),
    Status(String),
    Perms(Vec<PermAccount>, Option<String>),
}

struct PermEditor {
    open: bool,
    server: String,
    accounts: Vec<PermAccount>,
    new_group: String,
    error: Option<String>,
    loaded: bool,
}

enum Dialog {
    None,
    Message(String),
    Control,
    Logoff { target: String, typed: String },
}

pub struct App {
    cfg_path: String,
    cfg: Option<Config>,
    cfg_error: Option<String>,
    sessions: Vec<platform::Session>,
    selected: Option<usize>,
    search: String,
    ticket: String,
    status: String,
    lang_ru: bool,
    dialog: Dialog,
    tx: mpsc::Sender<AppEvent>,
    rx: mpsc::Receiver<AppEvent>,
    refreshing: bool,
    shadow_running: bool,
    member_view: bool,
    member_control: bool,
    member_logoff: bool,
    perm_editor: Option<PermEditor>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            cfg_path: "rds-shadow.ini".to_string(),
            cfg: None,
            cfg_error: None,
            sessions: Vec::new(),
            selected: None,
            search: String::new(),
            ticket: String::new(),
            status: String::new(),
            lang_ru: true,
            dialog: Dialog::None,
            tx,
            rx,
            refreshing: false,
            shadow_running: false,
            member_view: false,
            member_control: false,
            member_logoff: false,
            perm_editor: None,
        };
        if std::path::Path::new(&app.cfg_path).is_file() {
            app.load_config();
        } else {
            app.status = app
                .t(
                    "Укажите путь к INI и нажмите «Загрузить»",
                    "Set the INI path and press Load",
                )
                .to_string();
        }
        app
    }

    fn t<'a>(&self, ru: &'a str, en: &'a str) -> &'a str {
        if self.lang_ru {
            ru
        } else {
            en
        }
    }

    fn load_config(&mut self) {
        match rds_shadow_core::config::load(&self.cfg_path) {
            Ok(cfg) => {
                self.lang_ru = match cfg.ui.language {
                    Language::Ru => true,
                    Language::En => false,
                    Language::Auto => self.lang_ru,
                };
                self.member_view = cfg.groups.view.as_deref().is_some_and(platform::in_group);
                self.member_control = cfg
                    .groups
                    .control
                    .as_deref()
                    .is_some_and(platform::in_group);
                self.member_logoff = cfg.groups.logoff.as_deref().is_some_and(platform::in_group);
                self.status = self
                    .t("Конфигурация загружена", "Configuration loaded")
                    .to_string();
                self.cfg_error = None;
                self.cfg = Some(cfg);
                self.refresh();
            }
            Err(e) => {
                self.cfg = None;
                self.cfg_error = Some(e.to_string());
                self.status = format!("{}: {e}", self.t("Ошибка конфигурации", "Config error"));
            }
        }
    }

    fn refresh(&mut self) {
        let Some(cfg) = self.cfg.clone() else {
            self.status = self
                .t(
                    "Сначала загрузите конфигурацию",
                    "Load the configuration first",
                )
                .to_string();
            return;
        };
        let hosts = match platform::resolve_hosts(&cfg) {
            Ok(h) => h,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        if self.refreshing {
            return;
        }
        self.refreshing = true;
        self.status = format!(
            "{}: {}",
            self.t("Опрос серверов", "Polling hosts"),
            hosts.len()
        );
        let tx = self.tx.clone();
        thread::spawn(move || {
            let (sessions, errors) = platform::list_all(&hosts);
            let _ = tx.send(AppEvent::Sessions(sessions, errors));
        });
    }

    fn selected_session(&self) -> Option<&platform::Session> {
        self.selected.and_then(|i| self.sessions.get(i))
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::Sessions(sessions, errors) => {
                    self.refreshing = false;
                    self.sessions = sessions;
                    self.selected = None;
                    if errors.is_empty() {
                        self.status = format!(
                            "{}: {}",
                            self.t("Найдено сеансов", "Sessions found"),
                            self.sessions.len()
                        );
                    } else {
                        self.status = format!(
                            "{}: {} | {}: {}",
                            self.t("Найдено сеансов", "Sessions found"),
                            self.sessions.len(),
                            self.t("ошибки", "errors"),
                            errors.join("; ")
                        );
                    }
                }
                AppEvent::Status(text) => {
                    self.shadow_running = false;
                    self.status = text;
                }
                AppEvent::Perms(accounts, error) => {
                    if let Some(ed) = &mut self.perm_editor {
                        ed.accounts = accounts;
                        ed.error = error;
                        ed.loaded = true;
                    }
                }
            }
        }
    }

    fn start_shadow(&mut self, control: bool) {
        let Some(cfg) = self.cfg.clone() else {
            return;
        };
        let Some(session) = self.selected_session().cloned() else {
            return;
        };
        let ticket = self.ticket.trim().to_string();
        if ticket.is_empty() {
            self.status = self
                .t("Введите номер заявки", "Enter a ticket number")
                .to_string();
            return;
        }
        if !session.is_active() {
            self.status = self
                .t(
                    "Только активный сеанс: к отключённому shadow не подключается",
                    "Active sessions only: shadow cannot attach to a disconnected session",
                )
                .to_string();
            return;
        }
        let mode = if control {
            ActionMode::Control
        } else {
            ActionMode::View
        };
        let (group, member, kind) = if control {
            (
                cfg.groups.control.clone(),
                self.member_control,
                EventKind::ShadowDenied,
            )
        } else {
            (
                cfg.groups.view.clone(),
                self.member_view,
                EventKind::ShadowDenied,
            )
        };
        let allowed = group.is_some() && member;
        if !allowed {
            audit::denied(
                &cfg,
                kind,
                &session.server,
                session.id,
                Some(mode),
                &ticket,
                "required group is not configured or current user is not a member",
            );
            self.status = self
                .t(
                    "Нет прав: группа не настроена или вы не входите в неё",
                    "Denied: group is not configured or you are not a member",
                )
                .to_string();
            return;
        }

        let mut start = AuditEvent::new(EventKind::ShadowStart, ResultKind::Ok);
        start.target_server = Some(session.server.clone());
        start.target_user = Some(session.user.clone());
        start.session_id = Some(session.id);
        start.session_state = Some(session.state.clone());
        start.mode = Some(mode);
        start.ticket = Some(ticket.clone());
        audit::record(&cfg, start);

        self.shadow_running = true;
        self.status = if control {
            self.t("Открыто управление сеансом", "Control session opened")
                .to_string()
        } else {
            self.t("Открыт просмотр сеанса", "View session opened")
                .to_string()
        };

        let tx = self.tx.clone();
        let server = session.server.clone();
        let id = session.id;
        thread::spawn(move || {
            let began = Instant::now();
            let result = platform::run_mstsc(&server, id, control);
            let mut end = AuditEvent::new(
                EventKind::ShadowEnd,
                if matches!(&result, Ok(s) if s.success()) {
                    ResultKind::Ok
                } else {
                    ResultKind::Fail
                },
            );
            end.target_server = Some(server.clone());
            end.target_user = Some(session.user.clone());
            end.session_id = Some(id);
            end.mode = Some(mode);
            end.ticket = Some(ticket);
            end.duration_ms = Some(began.elapsed().as_millis() as u64);
            if let Err(e) = &result {
                end.error = Some(e.to_string());
            }
            audit::record(&cfg, end);
            let text = match result {
                Ok(s) if s.success() => format!("mstsc: {s}"),
                Ok(s) => format!("mstsc: {s}"),
                Err(e) => format!("mstsc failed: {e}"),
            };
            let _ = tx.send(AppEvent::Status(text));
        });
    }

    fn send_message(&mut self, text: &str) {
        let Some(cfg) = self.cfg.clone() else {
            return;
        };
        let Some(session) = self.selected_session().cloned() else {
            return;
        };
        let ticket = self.ticket.trim().to_string();
        if ticket.is_empty() || text.trim().is_empty() {
            self.status = self
                .t("Нужны заявка и текст", "Ticket and text are required")
                .to_string();
            return;
        }
        let allowed = cfg.groups.view.is_some() && self.member_view;
        if !allowed {
            audit::denied(
                &cfg,
                EventKind::MessageDenied,
                &session.server,
                session.id,
                None,
                &ticket,
                "view group is not configured or current user is not a member",
            );
            self.status = self
                .t("Нет прав на сообщение", "No permission to send a message")
                .to_string();
            return;
        }
        match platform::send_message(&session.server, session.id, "RDS Shadow", text) {
            Ok(()) => {
                let mut e = AuditEvent::new(EventKind::MessageSend, ResultKind::Ok);
                e.target_server = Some(session.server.clone());
                e.target_user = Some(session.user.clone());
                e.session_id = Some(session.id);
                e.ticket = Some(ticket);
                e.message_text = Some(text.to_string());
                e.message_len = Some(text.chars().count());
                audit::record(&cfg, e);
                self.status = self.t("Сообщение отправлено", "Message sent").to_string();
            }
            Err(e) => {
                let mut ev = AuditEvent::new(EventKind::MessageDenied, ResultKind::Fail);
                ev.target_server = Some(session.server.clone());
                ev.target_user = Some(session.user.clone());
                ev.session_id = Some(session.id);
                ev.ticket = Some(ticket);
                ev.error = Some(e.clone());
                audit::record(&cfg, ev);
                self.status = format!("{}: {e}", self.t("Ошибка", "Error"));
            }
        }
    }

    fn do_logoff(&mut self, target: &str, typed: &str) {
        let Some(cfg) = self.cfg.clone() else {
            return;
        };
        let Some(session) = self.selected_session().cloned() else {
            return;
        };
        let ticket = self.ticket.trim().to_string();
        if target.is_empty() || !target.eq_ignore_ascii_case(typed.trim()) {
            self.status = self
                .t(
                    "Имя не совпадает с пользователем сеанса",
                    "Name does not match the session user",
                )
                .to_string();
            return;
        }
        let allowed = cfg.groups.logoff.is_some() && self.member_logoff;
        if !allowed {
            audit::denied(
                &cfg,
                EventKind::LogoffDenied,
                &session.server,
                session.id,
                None,
                &ticket,
                "logoff group is not configured or current user is not a member",
            );
            self.status = self
                .t("Нет прав на завершение", "No permission to log off")
                .to_string();
            return;
        }

        let mut requested = AuditEvent::new(EventKind::LogoffRequested, ResultKind::Ok);
        requested.target_server = Some(session.server.clone());
        requested.target_user = Some(target.to_string());
        requested.session_id = Some(session.id);
        requested.ticket = Some(ticket.clone());
        audit::record(&cfg, requested);

        match platform::logoff(&session.server, session.id) {
            Ok(()) => {
                let mut ok = AuditEvent::new(EventKind::LogoffOk, ResultKind::Ok);
                ok.target_server = Some(session.server.clone());
                ok.target_user = Some(target.to_string());
                ok.session_id = Some(session.id);
                ok.ticket = Some(ticket);
                audit::record(&cfg, ok);
                self.status = format!("{} {}", self.t("Сеанс завершён:", "Session ended:"), target);
                self.refresh();
            }
            Err(e) => {
                let mut fail = AuditEvent::new(EventKind::LogoffFail, ResultKind::Fail);
                fail.target_server = Some(session.server.clone());
                fail.target_user = Some(target.to_string());
                fail.session_id = Some(session.id);
                fail.ticket = Some(ticket);
                fail.error = Some(e.clone());
                audit::record(&cfg, fail);
                self.status = format!("{}: {e}", self.t("Ошибка", "Error"));
            }
        }
    }

    fn can_act(&self, group: Option<&String>, member: bool) -> bool {
        group.is_some() && member && self.selected.is_some() && !self.ticket.trim().is_empty()
    }

    fn draw_dialogs(&mut self, ctx: &egui::Context) {
        let mut dialog = std::mem::replace(&mut self.dialog, Dialog::None);
        match &mut dialog {
            Dialog::None => {}
            Dialog::Message(text) => {
                let mut send = false;
                let mut cancel = false;
                egui::Window::new(self.t("Сообщение сотруднику", "Message to employee"))
                    .collapsible(false)
                    .resizable(true)
                    .default_width(460.0)
                    .show(ctx, |ui| {
                        ui.label(
                            self.t(
                                "Текст появится на экране сотрудника. Не отправляйте ПДн и закрытые данные.",
                                "The text appears on the employee screen. Do not send PII or restricted data.",
                            ),
                        );
                        ui.add(
                            egui::TextEdit::multiline(text)
                                .desired_rows(4)
                                .desired_width(f32::INFINITY),
                        );
                        ui.horizontal(|ui| {
                            send = ui.button(self.t("Отправить", "Send")).clicked();
                            cancel = ui.button(self.t("Отмена", "Cancel")).clicked();
                        });
                    });
                if send {
                    let text = text.clone();
                    self.send_message(&text);
                    dialog = Dialog::None;
                } else if cancel {
                    dialog = Dialog::None;
                }
            }
            Dialog::Control => {
                let mut confirm = false;
                let mut cancel = false;
                egui::Window::new(self.t("Подключиться с управлением?", "Open with control?"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(
                            self.t(
                                "Вы сможете управлять клавиатурой и мышью сотрудника. Сотрудник должен дать согласие.",
                                "You will control the employee keyboard and mouse. The employee must consent.",
                            ),
                        );
                        ui.horizontal(|ui| {
                            confirm = ui
                                .button(self.t("Подключиться", "Connect"))
                                .clicked();
                            cancel = ui.button(self.t("Отмена", "Cancel")).clicked();
                        });
                    });
                if confirm {
                    self.start_shadow(true);
                    dialog = Dialog::None;
                } else if cancel {
                    dialog = Dialog::None;
                }
            }
            Dialog::Logoff { target, typed } => {
                let mut confirm = false;
                let mut cancel = false;
                egui::Window::new(self.t("Завершить сеанс", "Log off session"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(self.t(
                            "Несохранённая работа сотрудника будет потеряна.",
                            "The employee's unsaved work will be lost.",
                        ));
                        ui.label(format!("{}: {target}", self.t("Пользователь", "User")));
                        if let Some(session) = self.selected_session() {
                            ui.label(format!(
                                "{}: {} · {}: {}",
                                self.t("Сервер", "Server"),
                                session.server,
                                self.t("ID", "ID"),
                                session.id
                            ));
                        }
                        ui.label(self.t(
                            "Введите имя пользователя, чтобы подтвердить:",
                            "Type the user name to confirm:",
                        ));
                        ui.add(egui::TextEdit::singleline(typed).desired_width(320.0));
                        let matches = target.eq_ignore_ascii_case(typed.trim());
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    matches,
                                    egui::Button::new(self.t("Завершить", "Log off"))
                                        .fill(egui::Color32::from_rgb(120, 30, 30)),
                                )
                                .clicked()
                            {
                                confirm = true;
                            }
                            cancel = ui.button(self.t("Отмена", "Cancel")).clicked();
                        });
                    });
                if confirm {
                    let target = target.clone();
                    let typed = typed.clone();
                    self.do_logoff(&target, &typed);
                    dialog = Dialog::None;
                } else if cancel {
                    dialog = Dialog::None;
                }
            }
        }
        self.dialog = dialog;
    }

    fn open_perm_editor(&mut self) {
        let server = self
            .selected_session()
            .map(|s| s.server.clone())
            .or_else(|| self.cfg.as_ref().and_then(|c| c.servers.first().cloned()))
            .unwrap_or_default();
        self.perm_editor = Some(PermEditor {
            open: true,
            server,
            accounts: Vec::new(),
            new_group: String::new(),
            error: None,
            loaded: false,
        });
        self.load_perms();
    }

    fn load_perms(&mut self) {
        let Some(ed) = &self.perm_editor else {
            return;
        };
        let server = ed.server.clone();
        let tx = self.tx.clone();
        thread::spawn(move || match rights::list_accounts(&server) {
            Ok(accounts) => {
                let _ = tx.send(AppEvent::Perms(accounts, None));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Perms(Vec::new(), Some(e)));
            }
        });
    }

    fn draw_perm_editor(&mut self, ctx: &egui::Context) {
        let mut ed = match self.perm_editor.take() {
            Some(ed) => ed,
            None => return,
        };
        if !ed.open {
            return;
        }

        let l_server = self.t("Сервер", "Server").to_string();
        let l_refresh = self.t("Обновить", "Refresh").to_string();
        let l_group = self
            .t("Группа / учётная запись", "Group / account")
            .to_string();
        let l_query = self.t("Информация", "Query").to_string();
        let l_remote = self.t("Управление", "Remote control").to_string();
        let l_msg = self.t("Сообщение", "Message").to_string();
        let l_logoff = self.t("Завершение", "Logoff").to_string();
        let l_new = self.t("Добавить группу", "Add group").to_string();
        let l_note = self
            .t(
                "Требуются права администратора на сервере (WMI).",
                "Requires admin rights on the server (WMI).",
            )
            .to_string();

        let mut toggles: Vec<(usize, Perm, bool)> = Vec::new();
        let mut reload = false;
        let mut add_group: Option<String> = None;
        let snapshot: Vec<(String, String, u32)> = ed
            .accounts
            .iter()
            .map(|a| (a.name.clone(), a.sid.clone(), a.mask))
            .collect();

        egui::Window::new(self.t("Права RDP-Tcp", "RDP-Tcp rights"))
            .collapsible(false)
            .resizable(true)
            .default_width(640.0)
            .open(&mut ed.open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&l_server);
                    ui.add(egui::TextEdit::singleline(&mut ed.server).desired_width(200.0));
                    if ui.button(&l_refresh).clicked() {
                        reload = true;
                    }
                });
                ui.label(&l_note);
                if let Some(err) = &ed.error {
                    ui.colored_label(egui::Color32::from_rgb(220, 90, 90), err);
                }

                ui.horizontal(|ui| {
                    ui.strong(&l_group);
                    ui.separator();
                    ui.strong(&l_query);
                    ui.strong(&l_remote);
                    ui.strong(&l_msg);
                    ui.strong(&l_logoff);
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        for (i, (name, sid, mask)) in snapshot.iter().enumerate() {
                            let mut q = mask & Perm::Query.bit() != 0;
                            let mut s = mask & Perm::Shadow.bit() != 0;
                            let mut m = mask & Perm::Message.bit() != 0;
                            let mut lf = mask & Perm::Logoff.bit() != 0;
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{name} ({})",
                                    if sid.is_empty() { "-" } else { sid }
                                ));
                                if ui.checkbox(&mut q, "").changed() {
                                    toggles.push((i, Perm::Query, q));
                                }
                                if ui.checkbox(&mut s, "").changed() {
                                    toggles.push((i, Perm::Shadow, s));
                                }
                                if ui.checkbox(&mut m, "").changed() {
                                    toggles.push((i, Perm::Message, m));
                                }
                                if ui.checkbox(&mut lf, "").changed() {
                                    toggles.push((i, Perm::Logoff, lf));
                                }
                            });
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(&l_new);
                    ui.add(
                        egui::TextEdit::singleline(&mut ed.new_group)
                            .hint_text("DOMAIN\\Group")
                            .desired_width(240.0),
                    );
                    if ui.button(self.t("Добавить", "Add")).clicked() {
                        let g = ed.new_group.trim().to_string();
                        if !g.is_empty() {
                            add_group = Some(g);
                        }
                    }
                });
            });

        for (i, perm, allow) in toggles {
            let name = ed.accounts[i].name.clone();
            match rights::set_permission(&ed.server, &name, perm, allow) {
                Ok(()) => {
                    let bit = perm.bit();
                    if allow {
                        ed.accounts[i].mask |= bit;
                    } else {
                        ed.accounts[i].mask &= !bit;
                    }
                }
                Err(e) => {
                    ed.error = Some(e);
                }
            }
        }

        if let Some(group) = add_group {
            match rights::add_account(&ed.server, &group) {
                Ok(()) => {
                    ed.new_group.clear();
                    reload = true;
                }
                Err(e) => {
                    ed.error = Some(e);
                }
            }
        }

        if reload {
            self.perm_editor = Some(ed);
            self.load_perms();
        } else {
            self.perm_editor = Some(ed);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain_events();
        let ctx = ui.ctx().clone();

        egui::Panel::top("top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(self.t("Конфигурация", "Config"));
                ui.add(egui::TextEdit::singleline(&mut self.cfg_path).desired_width(340.0));
                if ui.button(self.t("Загрузить", "Load")).clicked() {
                    self.load_config();
                }
                let can_refresh = self.cfg.is_some() && !self.refreshing;
                if ui
                    .add_enabled(
                        can_refresh,
                        egui::Button::new(self.t("Обновить", "Refresh")),
                    )
                    .clicked()
                {
                    self.refresh();
                }
                if self.refreshing {
                    ui.spinner();
                }
                if ui.button(if self.lang_ru { "EN" } else { "RU" }).clicked() {
                    self.lang_ru = !self.lang_ru;
                }
                ui.separator();
                if ui
                    .button(self.t("Права RDP-Tcp", "RDP-Tcp rights"))
                    .clicked()
                {
                    self.open_perm_editor();
                }
            });
            ui.horizontal(|ui| {
                ui.label(self.t("Поиск", "Search"));
                let hint = self
                    .t("логин, сервер, ID, состояние", "user, host, ID, state")
                    .to_string();
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text(hint)
                        .desired_width(280.0),
                );
                let total = self.sessions.len();
                let shown = self.filtered_indices().len();
                ui.label(format!("{shown} / {total}"));
                if let Some(cfg) = &self.cfg {
                    ui.separator();
                    ui.label(format!(
                        "{}: {}",
                        self.t("Режим", "Mode"),
                        cfg.connection.mode
                    ));
                    ui.label(format!(
                        "{}: {}",
                        self.t("Порт", "Port"),
                        cfg.connection.rdp_port
                    ));
                }
            });
            ui.add_space(2.0);
        });

        egui::Panel::bottom("actions").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(self.t("Заявка", "Ticket"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.ticket)
                        .hint_text("SD-12345")
                        .desired_width(150.0),
                );
                let cfg = self.cfg.clone();
                let view_ok = cfg
                    .as_ref()
                    .is_some_and(|c| self.can_act(c.groups.view.as_ref(), self.member_view));
                let control_ok = cfg
                    .as_ref()
                    .is_some_and(|c| self.can_act(c.groups.control.as_ref(), self.member_control));
                let message_ok = view_ok;
                let logoff_ok = cfg
                    .as_ref()
                    .is_some_and(|c| self.can_act(c.groups.logoff.as_ref(), self.member_logoff));

                if ui
                    .add_enabled(view_ok, egui::Button::new(self.t("Просмотр", "View")))
                    .clicked()
                {
                    self.start_shadow(false);
                }
                if ui
                    .add_enabled(
                        control_ok,
                        egui::Button::new(self.t("Управление", "Control")),
                    )
                    .clicked()
                {
                    self.dialog = Dialog::Control;
                }
                if ui
                    .add_enabled(
                        message_ok,
                        egui::Button::new(self.t("Сообщение", "Message")),
                    )
                    .clicked()
                {
                    self.dialog = Dialog::Message(String::new());
                }
                if ui
                    .add_enabled(
                        logoff_ok,
                        egui::Button::new(self.t("Завершить сеанс", "Log off"))
                            .fill(egui::Color32::from_rgb(90, 30, 30)),
                    )
                    .clicked()
                {
                    if let Some(session) = self.selected_session() {
                        self.dialog = Dialog::Logoff {
                            target: session.user.clone(),
                            typed: String::new(),
                        };
                    }
                }
            });
            ui.horizontal(|ui| {
                if let Some(session) = self.selected_session() {
                    ui.label(format!(
                        "{} {} · {}: {} · ID {}",
                        session.user,
                        self.t("на", "on"),
                        self.t("сервер", "server"),
                        session.server,
                        session.id
                    ));
                } else {
                    ui.label(self.t("Сеанс не выбран", "No session selected"));
                }
                if let Some(cfg) = &self.cfg {
                    ui.separator();
                    ui.label(format!(
                        "{}: {}",
                        self.t("Журнал", "Log"),
                        cfg.audit.log_dir.display()
                    ));
                }
            });
            let color = if self.status.contains("Ошибка") || self.status.contains("Denied") {
                egui::Color32::from_rgb(220, 90, 90)
            } else {
                ui.visuals().weak_text_color()
            };
            ui.colored_label(color, &self.status);
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(err) = &self.cfg_error {
                ui.colored_label(egui::Color32::from_rgb(220, 90, 90), err);
            }
            let indices = self.filtered_indices();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("sessions")
                    .num_columns(4)
                    .striped(true)
                    .min_col_width(90.0)
                    .show(ui, |ui| {
                        ui.strong(self.t("Сервер", "Server"));
                        ui.strong(self.t("Пользователь", "User"));
                        ui.strong("ID");
                        ui.strong(self.t("Состояние", "State"));
                        ui.end_row();
                        for idx in indices {
                            let session = &self.sessions[idx];
                            let selected = self.selected == Some(idx);
                            if ui
                                .selectable_label(selected, session.server.as_str())
                                .clicked()
                            {
                                self.selected = Some(idx);
                            }
                            if ui
                                .selectable_label(selected, session.user.as_str())
                                .clicked()
                            {
                                self.selected = Some(idx);
                            }
                            if ui
                                .selectable_label(selected, session.id.to_string())
                                .clicked()
                            {
                                self.selected = Some(idx);
                            }
                            if ui
                                .selectable_label(selected, session.state.as_str())
                                .clicked()
                            {
                                self.selected = Some(idx);
                            }
                            ui.end_row();
                        }
                    });
            });
        });

        self.draw_dialogs(&ctx);
        self.draw_perm_editor(&ctx);

        if self.refreshing || self.shadow_running {
            ctx.request_repaint_after(Duration::from_millis(300));
        }
    }
}

impl App {
    fn filtered_indices(&self) -> Vec<usize> {
        let needle = self.search.trim().to_lowercase();
        self.sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                needle.is_empty()
                    || s.user.to_lowercase().contains(&needle)
                    || s.server.to_lowercase().contains(&needle)
                    || s.state.to_lowercase().contains(&needle)
                    || s.id.to_string() == needle
            })
            .map(|(i, _)| i)
            .collect()
    }
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "dejavu".to_owned(),
        egui::FontData::from_static(include_bytes!("../../../assets/DejaVuSans.ttf")).into(),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "dejavu".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("dejavu".to_owned());
    ctx.set_fonts(fonts);
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 540.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RDS Shadow",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )?;
    Ok(())
}
