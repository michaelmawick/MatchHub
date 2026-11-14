use eframe::egui::{self, Id, emath::easing};

pub struct Reveal {
    elapsed: f32,
}

impl Reveal {
    pub fn new(ctx: &egui::Context, key: &str) -> Self {
        let now = ctx.input(|i| i.time);
        let start = ctx.data_mut(|data| *data.get_temp_mut_or_insert_with(id(key), || now));
        Self {
            elapsed: (now - start) as f32,
        }
    }

    pub fn restart(ctx: &egui::Context, key: &str) {
        ctx.data_mut(|data| data.remove_temp::<f64>(id(key)));
    }

    pub fn progress(&self, delay: f32, duration: f32) -> f32 {
        easing::cubic_out(((self.elapsed - delay) / duration).clamp(0.0, 1.0))
    }

    pub fn fade<R>(
        &self,
        ui: &mut egui::Ui,
        delay: f32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let progress = self.progress(delay, 0.6);
        ui.scope(|ui| {
            ui.multiply_opacity(progress);
            add_contents(ui)
        })
        .inner
    }
}

fn id(key: &str) -> Id {
    Id::unique(("reveal", key))
}

pub fn page<R>(
    ui: &mut egui::Ui,
    key: &str,
    slide: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let height_id = Id::unique(("page_height", key));
    egui::ScrollArea::vertical()
        .id_salt(height_id)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let viewport = ui.available_height();
            let content = ui
                .ctx()
                .data(|data| data.get_temp::<f32>(height_id))
                .unwrap_or(viewport);
            ui.add_space(((viewport - content) / 2.0).max(32.0) + slide);

            let response = ui.scope(add_contents);
            let height = response.response.rect.height();
            ui.ctx()
                .data_mut(|data| data.insert_temp(height_id, height));
            ui.add_space(32.0);
            response.inner
        })
        .inner
}
