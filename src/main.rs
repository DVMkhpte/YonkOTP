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
    window.set_resizable(false);

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
        .title-label {
            font-size: 24px;
            font-weight: bold;
        }
        .search-entry {
            background: #444;
            color: white;
            border-radius: 8px;
            padding: 5px;
        }
        .green-button {
            background: #28a745;
            color: white;
            font-weight: bold;
            border-radius: 8px;
        }
        .green-button:hover {
            background: #218838;
        }
        .otp-code {
            font-size: 22px;
            font-weight: bold;
        }
    ");

    let display = Display::default().expect("Impossible de récupérer l'affichage");
    
    // Utilisation de la nouvelle fonction recommandée
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}