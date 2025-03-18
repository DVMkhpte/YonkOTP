use gtk::prelude::*;
use gtk::{glib, Application, Builder, CssProvider};
use gtk::gdk::Display;

const APP_ID: &str = "org.yonkotp.main";
const UI_FILE: &str = "resources/window.ui"; 

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &gtk::Application) {
    // Charger l'interface à partir du fichier XML
    let builder = Builder::from_file(UI_FILE);

    // Récupérer la fenêtre définie dans le fichier XML
    let window: gtk::ApplicationWindow = builder
        .object("main_window")
        .expect("Échec du chargement de la fenêtre");

    window.set_application(Some(app));

    load_css();
    
    if let Some(button) = builder.object::<gtk::Button>("ok_button") {
        button.connect_clicked(|_| {
            println!("Bouton cliqué !");
        });
    }

    window.present();
}


fn load_css() {
    let provider = CssProvider::new();
    provider
        .load_from_string("
        .green-button {
            background: #347D39; 
            color: white;
            font-size: 16px;
            font-weight: bold;
            padding: 10px 20px;
            border-radius: 8px;
        }
        .green-button:hover {
            background: #3b8640; /* Vert plus foncé au survol */
        }
    ");

    let display = Display::default().expect("Impossible de récupérer l'affichage");
    
    // Utilisation de la nouvelle fonction recommandée
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}