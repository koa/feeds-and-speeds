use std::f64::consts::PI;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use patternfly_yew::FormGroup;
use patternfly_yew::Slider;
use patternfly_yew::Text;
use patternfly_yew::TextInput;
use patternfly_yew::{Card, ChipVariant, InputState, Page, Select, SelectOption, SelectVariant};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew::{function_component, html, html_nested, use_state, Callback, Html};

#[derive(Clone, PartialEq, Debug, EnumIter, Copy)]
enum Material {
    Aluminium,
    Plastic,
    Copper,
    WoodSoft,
    WoodHard,
}

impl Material {
    pub fn cut_speed(&self) -> Range<f64> {
        match self {
            Material::Aluminium => 100.0..450.0,
            Material::Plastic => 200.0..400.0,
            Material::Copper => 80.0..200.0,
            Material::WoodSoft => 300.0..600.0,
            Material::WoodHard => 200.0..450.0,
        }
    }
    pub fn feed_table(&self) -> &[(f64, Range<f64>)] {
        match self {
            Material::Aluminium => &[
                (4.0, 0.005..0.015),
                (6.0, 0.015..0.025),
                (8.0, 0.02..0.03),
                (10.0, 0.025..0.038),
                (12.0, 0.03..0.05),
            ],
            Material::Plastic => &[
                (4.0, 0.02..0.05),
                (6.0, 0.04..0.09),
                (8.0, 0.04..0.1),
                (10.0, 0.05..0.15),
                (12.0, 0.08..0.18),
            ],
            Material::Copper => &[
                (4.0, 0.01..0.02),
                (6.0, 0.015..0.025),
                (8.0, 0.03..0.057),
                (10.0, 0.035..0.065),
                (12.0, 0.04..0.08),
            ],
            Material::WoodSoft => &[
                (4.0, 0.02..0.04),
                (6.0, 0.025..0.055),
                (8.0, 0.037..0.07),
                (10.0, 0.045..0.085),
                (12.0, 0.05..0.095),
            ],
            Material::WoodHard => &[
                (4.0, 0.015..0.035),
                (6.0, 0.02..0.05),
                (8.0, 0.03..0.065),
                (10.0, 0.045..0.08),
                (12.0, 0.05..0.09),
            ],
        }
    }
}

fn feed_per_flute(material: Material, diameter: f64) -> Range<f64> {
    let table = material.feed_table();
    let mut iter = table.iter();
    let mut last_entry = iter.next().expect("material has empty feed table");
    if last_entry.0 >= diameter {
        return last_entry.1.clone();
    }
    for entry in iter {
        if entry.0 == diameter {
            // exact hit -> return exact value
            return entry.1.clone();
        }
        if entry.0 > diameter {
            // something in between -> interpolate
            let left_weight = (entry.0 - last_entry.0) as f64 / (diameter - last_entry.0) as f64;
            let right_weight = 1.0 - left_weight;
            let min_value = last_entry.1.start * left_weight + entry.1.start * right_weight;
            let max_value = last_entry.1.end * left_weight + entry.1.end * right_weight;
            return min_value..max_value;
        }
        last_entry = entry;
    }
    last_entry.1.clone()
}
impl Display for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Material::Aluminium => f.write_str("Aluminium"),
            Material::Plastic => f.write_str("Kunststoff"),
            Material::Copper => f.write_str("Kupfer / Messing"),
            Material::WoodSoft => f.write_str("Holz weich"),
            Material::WoodHard => f.write_str("Holz hart"),
        }
    }
}
#[derive(Clone, PartialEq, Debug, Copy)]
struct GlobalState {
    material: Material,
    diameter: f64,
    diameter_error: bool,
    flute_count: u8,
    flute_count_error: bool,
    min_rpm: f64,
    max_rpm: f64,
    selected_rpm: f64,
}

impl Default for GlobalState {
    fn default() -> Self {
        GlobalState {
            material: Material::WoodSoft,
            diameter: 8.0,
            diameter_error: false,
            flute_count: 2,
            flute_count_error: false,
            min_rpm: 3000.0,
            max_rpm: 24000.0,
            selected_rpm: 13000.0,
        }
    }
}

impl GlobalState {
    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn set_material(&mut self, material: Material) {
        self.material = material;
        let range = self.rpm_range();
        self.selected_rpm = self.selected_rpm.clamp(range.start, range.end);
    }
    pub fn rpm_range(&self) -> Range<f64> {
        let diameter = self.diameter;
        let vc_range = self.material.cut_speed();
        let rpm_min = vc_range.start * 1000.0 / (diameter * PI);
        let rpm_max = vc_range.end * 1000.0 / (diameter * PI);

        rpm_min.clamp(self.min_rpm, self.max_rpm)..rpm_max.clamp(self.min_rpm, self.max_rpm)
    }

    pub fn feed_range(&self) -> Range<f64> {
        let feed_per_flute = feed_per_flute(self.material, self.diameter);
        self.selected_rpm * self.flute_count as f64 * feed_per_flute.start
            ..self.selected_rpm * self.flute_count as f64 * feed_per_flute.end
    }
    pub fn diameter(&self) -> f64 {
        self.diameter
    }
    pub fn flute_count(&self) -> u8 {
        self.flute_count
    }
    pub fn min_rpm(&self) -> f64 {
        self.min_rpm
    }
    pub fn max_rpm(&self) -> f64 {
        self.max_rpm
    }
    pub fn selected_rpm(&self) -> f64 {
        self.selected_rpm
    }

    pub fn set_diameter(&mut self, diameter: f64) {
        self.diameter = diameter;
        let rpm_range = self.rpm_range();
        self.selected_rpm = self.selected_rpm.clamp(rpm_range.start, rpm_range.end);
    }
    pub fn set_flute_count(&mut self, flute_count: u8) {
        self.flute_count = flute_count;
    }
    pub fn set_min_rpm(&mut self, min_rpm: f64) {
        self.min_rpm = min_rpm;
    }
    pub fn set_max_rpm(&mut self, max_rpm: f64) {
        self.max_rpm = max_rpm;
    }
    pub fn set_selected_rpm(&mut self, selected_rpm: f64) {
        self.selected_rpm = selected_rpm;
    }
    pub fn diameter_error(&self) -> bool {
        self.diameter_error
    }
    pub fn set_diameter_error(&mut self, diameter_error: bool) {
        self.diameter_error = diameter_error;
    }

    pub fn flute_count_error(&self) -> bool {
        self.flute_count_error
    }

    pub fn set_flute_count_error(&mut self, flute_count_error: bool) {
        self.flute_count_error = flute_count_error;
    }
}

#[function_component]
fn App() -> Html {
    let state = use_state(GlobalState::default);

    let rpm_range = state.rpm_range();

    let on_change_material: Callback<Material> = {
        let state = state.clone();
        Callback::from(move |value| {
            if *state.material() != value {
                let mut new_state = *state;
                new_state.set_material(value);
                state.set(new_state);
            }
        })
    };

    let on_change_rpm: Callback<f64> = {
        let state = state.clone();
        Callback::from(move |value| {
            if state.selected_rpm() != value {
                let mut new_state = *state;
                new_state.set_selected_rpm(value);
                state.set(new_state);
            }
        })
    };
    let on_change_diameter: Callback<String> = {
        let state = state.clone();
        Callback::from(move |value: String| match value.parse::<f64>() {
            Ok(value) => {
                if state.diameter() != value || state.diameter_error() {
                    let mut new_state = *state;
                    new_state.set_diameter(value);
                    new_state.set_diameter_error(false);
                    state.set(new_state);
                }
            }
            Err(_) => {
                if !state.diameter_error() {
                    let mut new_state = *state;
                    new_state.set_diameter_error(true);
                    state.set(new_state);
                }
            }
        })
    };
    let on_change_flute_count: Callback<String> = {
        let state = state.clone();
        Callback::from(move |value: String| match value.parse::<u8>() {
            Ok(value) => {
                if state.flute_count() != value || state.flute_count_error() {
                    let mut new_state = *state;
                    new_state.set_flute_count(value);
                    new_state.set_flute_count_error(false);
                    state.set(new_state);
                }
            }
            Err(_) => {
                if !state.flute_count_error() {
                    let mut new_state = *state;
                    new_state.set_flute_count_error(true);
                    state.set(new_state);
                }
            }
        })
    };

    let nav = html!("Vorschub und Drehzahl");
    let material_list = Material::iter()
        .map(|value| html_nested! {<SelectOption<Material> {value}/>})
        .collect::<Vec<_>>();
    let variant = SelectVariant::Single(on_change_material);
    let chip = ChipVariant::Values;
    let slide_pos = Some(state.selected_rpm()).filter(|v| !v.is_nan());
    let diameter_str = format!("{:.2}", state.diameter());
    let result = state.feed_range();
    let diameter_input_state = if state.diameter_error() {
        InputState::Error
    } else {
        InputState::Default
    };
    let flute_count_str = format!("{}", state.flute_count());
    let flute_count_input_state = if state.flute_count_error() {
        InputState::Error
    } else {
        InputState::Default
    };
    let result = if !(state.diameter_error() || state.flute_count_error()) {
        let min_value = format!("{:.0}", result.start);
        let max_value = format!("{:.0}", result.end);
        html! {
            <>
                <FormGroup label="Minimaler Vorschub">
                    <Text>{min_value}</Text>
                </FormGroup>
                <FormGroup label="Maximaler Vorschub">
                    <Text>{max_value}</Text>
                </FormGroup>
            </>
        }
    } else {
        html!()
    };
    html! {
        <Page {nav}>
            <section class="pf-c-page__main-section pf-m-limit-width pf-m-align-center">
                <Card>
                    <FormGroup label="Material">
                        <Select<Material> {variant} {chip} placeholder={state.material().to_string()}>
                            {material_list}
                        </Select<Material>>
                    </FormGroup>
                    <FormGroup label="Werkzeugdurchmesser">
                        <TextInput r#type="number" value={diameter_str} onchange={on_change_diameter} state={diameter_input_state}/>
                    </FormGroup>
                    <FormGroup label="Anzahl Schneiden">
                        <TextInput r#type="number" value={flute_count_str} onchange={on_change_flute_count} state={flute_count_input_state}/>
                    </FormGroup>
                    <FormGroup label="Drehzahl">
                    <div style="width: 20em"><strong>{"RPM: "}</strong>{format!("{:.0}",state.selected_rpm())} </div>
                    { if rpm_range.start < rpm_range.end {
                        html!{<Slider min={rpm_range.start} max={rpm_range.end} hide_labels={false} value={slide_pos} onchange={on_change_rpm} suppress_initial_change={true} label_precision={0}/>}
                    }else{
                        html!()
                    }
                    }
                    </FormGroup>
                    {result}
                </Card>
            </section>
        </Page>
    }
}
#[cfg(debug_assertions)]
const LOG_LEVEL: log::Level = log::Level::Trace;
fn main() {
    wasm_logger::init(wasm_logger::Config::new(LOG_LEVEL));
    yew::Renderer::<App>::new().render();
}
