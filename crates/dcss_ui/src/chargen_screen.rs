//! Character creation screen using egui.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use dcss_core::chargen::{ChargenState, SpeciesDefs, JobDefs};

pub fn chargen_screen_system(
    mut contexts: EguiContexts,
    species: Res<SpeciesDefs>,
    jobs: Res<JobDefs>,
    mut state: ResMut<ChargenState>,
) -> Result {
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        ui.heading("Create Your Character");
        ui.separator();

        ui.columns(2, |cols| {
            // Species column
            cols[0].heading("Species");
            cols[0].separator();
            egui::ScrollArea::vertical().id_salt("species_scroll").show(&mut cols[0], |ui| {
                for (i, sp) in species.0.iter().enumerate() {
                    let selected = state.species_index == i;
                    let label = format!("{} ({})", sp.name, sp.difficulty);
                    if ui.selectable_label(selected, &label).clicked() {
                        state.species_index = i;
                    }
                }
            });

            // Job column
            cols[1].heading("Background");
            cols[1].separator();
            egui::ScrollArea::vertical().id_salt("job_scroll").show(&mut cols[1], |ui| {
                for (i, job) in jobs.0.iter().enumerate() {
                    let selected = state.job_index == i;
                    let label = format!("{} ({})", job.name, job.category);
                    if ui.selectable_label(selected, &label).clicked() {
                        state.job_index = i;
                    }
                }
            });
        });

        ui.separator();

        // Preview selected combination
        if let (Some(sp), Some(job)) = (species.0.get(state.species_index), jobs.0.get(state.job_index)) {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} {}", sp.name, job.name)).size(18.0).strong());
                ui.label(format!("  Str:{} Int:{} Dex:{}", sp.str_stat(), sp.int_stat(), sp.dex_stat()));
            });
            if !job.skills.is_empty() {
                let skills: Vec<String> = job.skills.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                ui.label(format!("Skills: {}", skills.join(", ")));
            }
        }

        ui.separator();
        if ui.button(egui::RichText::new("Start Game").size(16.0).strong()).clicked() {
            state.confirmed = true;
        }
    });
    Ok(())
}
