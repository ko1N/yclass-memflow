use super::{GeneratorWindow, MemflowAttachWindow, ProcessAttachWindow, ProcessInfoWindow, SpiderWindow};
use crate::{
    class::ClassList,
    field::FieldKind,
    state::{GlobalState, StateRef},
};
use eframe::{
    egui::{Button, Context, CornerRadius, Frame, Margin, RichText, TopBottomPanel, Ui, WidgetText},
    epaint::{vec2, Color32},
};
use memflow::prelude::v1::*;

macro_rules! create_change_field_type_group {
    ($ui:ident, $r:ident, $fg:ident, $bg:ident, $($size:ident),*) => {
        $(
            if $ui
                .add_sized(
                    vec2(24., $ui.available_height()),
                    Button::new(RichText::new(concat!(stringify!($size))).color(Color32::$fg)).fill(Color32::$bg),
                )
                .clicked()
            {
                *$r = Some(ToolBarResponse::ChangeKind(FieldKind::$size));
            }
            $ui.add_space(2.);
        )*
    };
}

pub enum ToolBarResponse {
    MemflowAttach(OsInstanceArcBox<'static>),
    ProcessAttach(u32),
    ProcessDetach,
    Add(usize),
    Remove(usize),
    Insert(usize),
    ChangeKind(FieldKind),
}

pub struct ToolBarPanel {
    mf_attach_window: MemflowAttachWindow,
    ps_attach_window: ProcessAttachWindow,
    ps_info_window: ProcessInfoWindow,
    generator_window: GeneratorWindow,
    spider_window: SpiderWindow,
    state: StateRef,
}

impl ToolBarPanel {
    pub fn new(state: StateRef) -> Self {
        Self {
            state,
            mf_attach_window: MemflowAttachWindow::new(state),
            ps_attach_window: ProcessAttachWindow::new(state),
            ps_info_window: ProcessInfoWindow::new(state),
            generator_window: GeneratorWindow::new(state),
            spider_window: SpiderWindow::new(state),
        }
    }

    pub fn show(&mut self, ctx: &Context) -> Option<ToolBarResponse> {
        let mut response = None;

        if let Some(os) = self.mf_attach_window.show(ctx) {
            response = Some(ToolBarResponse::MemflowAttach(os));
            self.mf_attach_window.toggle();
        }

        if let Some(pid) = self.ps_attach_window.show(ctx) {
            response = Some(ToolBarResponse::ProcessAttach(pid));
            self.ps_attach_window.toggle();

            // if the process info window is open then tell it to refresh
            if self.ps_info_window.visible() {
                self.ps_info_window.visible();
            }
        }

        self.ps_info_window.show(ctx);

        self.generator_window.show(ctx);
        if let Err(e) = self.spider_window.show(ctx) {
            self.state.borrow_mut().toasts.error(e.to_string());
        }

        self.run_hotkeys(ctx, &mut response);

        let style = ctx.style();
        let frame = Frame {
            inner_margin: Margin::same(0),
            corner_radius: CornerRadius::ZERO,
            fill: style.visuals.window_fill(),
            stroke: style.visuals.window_stroke(),
            ..Default::default()
        };

        TopBottomPanel::top("_top_bar")
            .frame(frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.;
                    ui.visuals_mut().widgets.inactive.corner_radius = CornerRadius::ZERO;

                    ui.menu_button("Project", |ui| self.project_menu(ui));
                    ui.menu_button("Process", |ui| self.process_menu(ui, &mut response));

                    if ui.button("Generator").clicked() {
                        self.generator_window.toggle();
                    }

                    if ui.button("Spider").clicked() {
                        self.spider_window.toggle();
                    }

                    ui.add_space(4.);
                    ui.separator();
                    ui.add_space(4.);

                    self.status_ui(ui, &mut response);

                    ui.add_space(4.);
                    ui.separator();
                    ui.add_space(4.);

                    macro_rules! create_add_remove_group {
                        ($ui:ident, $r:ident, $var:ident, $($item:expr),*) => {
                            $(
                                if $ui.button(stringify!($item)).clicked() {
                                    $r = Some(ToolBarResponse::$var($item));
                                    $ui.close_menu();
                                }
                            )*
                        };
                    }

                    ui.menu_button("Add", |ui| {
                        ui.set_width(64.);

                        ui.vertical_centered_justified(|ui| {
                            create_add_remove_group!(
                                ui, response, Add, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096
                            );
                        });
                    })
                    .response
                    .on_hover_text("Adds N bytes");

                    ui.menu_button("Remove", |ui| {
                        ui.set_width(64.);

                        create_add_remove_group!(ui, response, Remove, 1, 2, 4, 16, 64, 256, 1024);
                    })
                    .response
                    .on_hover_text("Removes N fields");

                    ui.menu_button("Insert", |ui| {
                        ui.set_width(64.);

                        create_add_remove_group!(
                            ui, response, Insert, 1, 2, 4, 8, 16, 64, 256, 1024
                        );
                    })
                    .response
                    .on_hover_text("Inserts N bytes");

                    ui.add_space(2.);
                    ui.separator();
                    ui.add_space(2.);

                    self.field_change_ui(ui, &mut response);
                });
            });

        response
    }

    fn run_hotkeys(&mut self, ctx: &Context, response: &mut Option<ToolBarResponse>) {
        
        let do_attach = {
            // we should not leave a mutable borrow open over top of all these calls
            // global state is fricken stupid, this system should probably be adjusted
            // to something more rusty. No one wants runtime panics or we'd be on c++
            let hk_state = &self.state.borrow().hotkeys;
            let input = &ctx.input(|i| i.clone());

            if hk_state.pressed("process_info", input) {
                self.ps_info_window.toggle();
            }

            if hk_state.pressed("attach_memflow", input) {
                self.mf_attach_window.toggle();
            }

            if hk_state.pressed("attach_process", input) {
                self.ps_attach_window.toggle();
            }

            if hk_state.pressed("attach_recent", input) {
                true
            } else {
                false
            }
        };

        if(do_attach) {
            let state = &mut *self.state.borrow_mut();
            if let Some(name) = state.config.last_attached_process_name.as_ref().cloned() {
                attach_to_process(state, &name, response);
            }
        }
    }

    fn project_menu(&mut self, ui: &mut Ui) {
        let state = &mut *self.state.borrow_mut();

        if ui.button("New project").clicked() {
            state.save_project(None);
            state.class_list = ClassList::default();
            ui.close_menu();
        }

        if ui.button("Open project").clicked() {
            state.open_project();
            ui.close_menu();
        }

        if !state
            .config
            .recent_projects
            .as_ref()
            .map(|h| h.is_empty())
            .unwrap_or(true)
        {
            ui.menu_button("Open recent...", |ui| {
                let mut to_open = None;
                for project in state.config.recent_projects.as_ref().unwrap().iter() {
                    if let Some(name) = project.file_name().and_then(|name| name.to_str()) {
                        if ui.button(name).clicked() {
                            to_open = Some(project.to_owned());
                        }
                    }
                }

                if let Some(path) = to_open {
                    if state.open_project_path(&path) {
                        ui.close_menu();
                    } else {
                        state.config.recent_projects.as_mut().unwrap().remove(&path);
                    }
                }
            });
        }

        if ui.button("Save project").clicked() {
            state.save_project(None);
            ui.close_menu();
        }

        if ui.button("Save project as").clicked() {
            state.save_project_as();
            ui.close_menu();
        }
    }

    fn process_menu(&mut self, ui: &mut Ui, response: &mut Option<ToolBarResponse>) {
        ui.set_width(200.);

        if shortcut_button(
            ui,
            &mut *self.state.borrow_mut(),
            "process_info",
            "Process Info",
        ) {
            self.ps_info_window.toggle();
            ui.close_menu();
        }

        if shortcut_button(
            ui,
            &mut *self.state.borrow_mut(),
            "attach_memflow",
            "Attach memflow",
        ) {
            self.mf_attach_window.toggle();
            ui.close_menu();
        }

        if shortcut_button(
            ui,
            &mut *self.state.borrow_mut(),
            "attach_process",
            "Attach to process",
        ) {
            self.ps_attach_window.toggle();
            ui.close_menu();
        }

        // Reattach to last process
        let state = &mut *self.state.borrow_mut();
        if let Some(name) = state.config.last_attached_process_name.as_ref().cloned() {
            if shortcut_button(ui, state, "attach_recent", format!("Attach to {name}")) {
                attach_to_process(state, &name, response);

                ui.close_menu();
            }
        }

        if shortcut_button(ui, state, "detach_process", "Detach from process") {
            *response = Some(ToolBarResponse::ProcessDetach);
            ui.close_menu();
        }
    }

    fn status_ui(&mut self, ui: &mut Ui, response: &mut Option<ToolBarResponse>) {
        if let Some((proc_name, proc_id)) = self
            .state
            .borrow()
            .process
            .as_ref()
            .read()
            .as_ref()
            .map(|p| (p.name(), p.id()))
        {
            match proc_name {
                Ok(name) => _ = ui.label(format!("Status: Attached to {} - {}", name, proc_id)),
                Err(e) => {
                    self.state
                        .borrow_mut()
                        .toasts
                        .error(format!("Failed to get process name: {e}"));
                    *response = Some(ToolBarResponse::ProcessDetach);
                }
            };
        } else {
            ui.label("Status: Detached");
        }
    }

    fn field_change_ui(&mut self, ui: &mut Ui, response: &mut Option<ToolBarResponse>) {
        create_change_field_type_group!(ui, response, BLACK, GOLD, Bool);

        ui.separator();
        ui.add_space(2.);

        create_change_field_type_group!(ui, response, BLACK, LIGHT_GREEN, U8, U16, U32, U64);

        ui.separator();
        ui.add_space(2.);

        create_change_field_type_group!(ui, response, BLACK, LIGHT_BLUE, I8, I16, I32, I64);

        ui.separator();
        ui.add_space(2.);

        create_change_field_type_group!(ui, response, BLACK, LIGHT_RED, F32, F64);

        ui.separator();
        ui.add_space(2.);

        create_change_field_type_group!(ui, response, BLACK, GRAY, Unk8, Unk16, Unk32, Unk64);

        ui.separator();
        ui.add_space(2.);

        create_change_field_type_group!(ui, response, BLACK, BROWN, Ptr, StrPtr);
    }
}

fn shortcut_button(
    ui: &mut Ui,
    state: &GlobalState,
    name: &'static str,
    label: impl Into<WidgetText>,
) -> bool {
    ui.add(Button::new(label).shortcut_text(state.hotkeys.format(name, ui.ctx())))
        .clicked()
}

fn attach_to_process(state: &mut GlobalState, name: &str, response: &mut Option<ToolBarResponse>) {
    let mut os = state.os.write();
    if let Some(os) = os.as_mut() {
        let mut process = None;
        let callback = &mut |data: ProcessInfo| {
            if name.eq_ignore_ascii_case(data.name.as_ref()) {
                process = Some(data);
                false
            } else {
                true
            }
        };
        os.process_info_list_callback(callback.into()).ok();

        if let Some(pe) = process {
            *response = Some(ToolBarResponse::ProcessAttach(pe.pid));
        } else {
            state.toasts.error(format!("Failed to find {name}"));
        }
    } else {
        state.toasts.error("Memflow is not loaded");
    }
}
