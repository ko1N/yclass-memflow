use crate::state::StateRef;
use eframe::{
    egui::{Context, RichText, ScrollArea, TextEdit, Window},
    epaint::{vec2, FontId},
};
use memflow::prelude::v1::*;

pub struct MemflowAttachWindow {
    shown: bool,
    filter: String,
    connectors: Vec<String>,
    state: StateRef,
}

impl MemflowAttachWindow {
    pub fn new(state: StateRef) -> Self {
        Self {
            state,
            connectors: vec![],
            shown: false,
            filter: "".to_owned(),
        }
    }

    pub fn toggle(&mut self) {
        self.shown = !self.shown;

        if self.shown {
            let state = self.state.borrow();
            self.connectors = state.inventory.available_connectors();
        }
    }

    pub fn show(&mut self, ctx: &Context) -> Option<OsInstanceArcBox<'static>> {
        if !self.shown {
            return None;
        }

        let mut os = None;
        Window::new("Load Memflow")
            .collapsible(false)
            .open(&mut self.shown)
            .default_size(vec2(180., 320.))
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    let r = TextEdit::singleline(&mut self.filter)
                        .desired_width(f32::INFINITY)
                        .hint_text("Filter by name")
                        .show(ui)
                        .response;

                    if ui.button("Refresh").clicked() || r.changed() {
                        let state = self.state.borrow();
                        self.connectors = state.inventory.available_connectors();
                    }

                    ui.add_space(4.);
                    ui.separator();
                    ui.add_space(4.);

                    ScrollArea::vertical().show(ui, |ui| {
                        for ce in self.connectors.iter().filter(|ce| {
                            self.filter.is_empty()
                                || ce.to_lowercase().contains(&self.filter.to_lowercase())
                        }) {
                            if ui
                                .button(RichText::new(ce).font(FontId::proportional(16.)))
                                .clicked()
                            {
                                log::info!("attaching to {ce}");
                                let mut state = self.state.borrow_mut();

                                os = match state
                                    .inventory
                                    .builder()
                                    .connector(ce.as_str())
                                    //.args("".parse())
                                    .os("win32")
                                    .build() {
                                        Ok(new_os) => {
                                            state.toasts.success(format!("Connected to {ce}"));
                                            Some(new_os)
                                        },
                                        Err(e) => {
                                            state.toasts.error(e.to_string());
                                            None
                                        }
                                    }
                            }
                        }
                    });
                });
            });
        os
    }
}
