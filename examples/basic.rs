use ggsdk::GGRunOptions;

#[derive(Default)]
struct App {

}

impl ggsdk::GGApp for App {
    fn init(&mut self, g: ggsdk::InitContext) {
    }

    fn update(&mut self, g: ggsdk::UpdateContext) {
       
    }
}

fn main() {
    let app = App::default();
    ggsdk::GGEngine::run(app, GGRunOptions {
        ..Default::default()
    });
}