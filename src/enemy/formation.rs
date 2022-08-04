use bevy::prelude::Component;
use rand::{thread_rng, Rng};
use crate::{BASE_SPEED, WinSize, FORMATION_MEMBERS_MAX};

#[derive(Clone,Component)]
pub struct Formation {
    pub start: (f32,f32),
    pub radius: (f32,f32),
    pub pivot: (f32,f32),
    pub speed: f32,
    pub angle: f32,
}

#[derive(Default)]
pub struct FormationMaker {
    current_template: Option<Formation>,
    current_members: u32,

}

impl FormationMaker {
    pub fn make(&mut self, win_size: &WinSize) -> Formation {
        match (&self.current_template, self.current_members >= FORMATION_MEMBERS_MAX) {
            (Some(tmpl), false) => {
                self.current_members += 1;
                tmpl.clone()
            }
            (None, _) | (_, true) => {
                let mut rng = thread_rng();
                let w_span = win_size.w/2. + 100.;
                let h_span = win_size.h/2. + 100.;
                let x = if rng.gen_bool(0.5) {w_span} else {-w_span};
                let y = rng.gen_range(-h_span..h_span) as f32;

                let start = (x,y);

                let w_span = win_size.w / 4.;
                let h_span = win_size.h / 3. + 50.;
                let pivot = (rng.gen_range(-w_span..w_span),rng.gen_range(0. ..h_span));
                let radius = (rng.gen_range(150. .. 200.),100.);
                // let radius = (200.,100.);

                let angle = (y-pivot.1).atan2(x-pivot.0);

                let speed = BASE_SPEED/2.;

                let formation = Formation {
                    start: start,
                    pivot: pivot,
                    radius: radius,
                    angle: angle,
                    speed: speed
                };

                self.current_template = Some(formation.clone());
                self.current_members = 1;
                formation
            }
        }
    }
}

