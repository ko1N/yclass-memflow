use crate::state::StateRef;
use eframe::{
    egui::{Context, RichText, ScrollArea, TextEdit, Window},
    epaint::{vec2, FontId},
};
use memflow::prelude::v1::*;

pub struct ProcessInfoWindow {
    shown: bool,
    // modules_filter: String, // todo
    process_info: Option<ProcessInfo>,
    state: StateRef,
}

impl ProcessInfoWindow {
    pub fn new(state: StateRef) -> Self {
        Self {
            state,
            shown: false,
            process_info: None,
            // modules_filter: "".to_owned(),
        }
    }

    pub fn refresh_info(&mut self) {
        let state = self.state.borrow();
        let process_lock = state.process.read();
        self.process_info = process_lock.as_ref().map(|p|p.info());
    }

    pub fn toggle(&mut self) {
        self.shown = !self.shown;

        if self.shown {
            self.refresh_info();
        }
    }

    pub fn visible(&self) -> bool {
        return self.shown;
    }

    pub fn show(&mut self, ctx: &Context) {
        if !self.shown {
            return;
        }
        let procname = self.process_info.as_ref().map(|p|{
            p.name.to_string()
        }).unwrap_or("NOT CONNECTED".to_owned());

        Window::new(format!("Process Info: {:?}", procname))
            .collapsible(false)
            .open(&mut self.shown)
            .default_size(vec2(180., 320.))
            .show(ctx, |ui| {

            if let Some(proc_info) = &self.process_info {
                ui.vertical_centered_justified(|ui| {
                    
                    ui.label(format!("PID: {}", proc_info.pid));
                    ui.label(format!("Name: {}", proc_info.name));
                    ui.label(format!("Path: {}", proc_info.path));
                    
                    ui.add_space(4.);
                    ui.separator();
                    ui.add_space(4.);

                    ui.label(format!("Address: {}", proc_info.address));
                    ui.label(format!("dtb1: {}", proc_info.dtb1));
                    ui.label(format!("dtb2: {}", proc_info.dtb2));
                    ui.label(format!("CMD: {}", proc_info.command_line));
                    ui.label(format!("Alive: {}", proc_info.state.is_alive()));
                    ui.label(format!("Arch: {}", proc_info.proc_arch));
                    ui.label(format!("SysArch: {}", proc_info.sys_arch));
                    
                });
            } else {
                ui.label("No process attached");
            }
    });
    }
}
