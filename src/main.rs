use gtk::prelude::*;
use gtk::{glib, Application, Builder, CssProvider, Window};
use gtk::gdk::Display;
use std::rc::Rc;
use std::cell::RefCell;

mod data_filter; // Déclare le module
use data_filter::{serach_data, validate_data}; // Importe la fonction filter_data


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

    // Récupérer la fenêtre principale
    let window: gtk::ApplicationWindow = builder
        .object("main_window")
        .expect("Échec du chargement de la fenêtre");

    window.set_application(Some(app));
    window.set_resizable(false);

    load_css();
    
    // Récupérer la fenêtre modale
    let add_key_window: Window = builder
        .object("add_key_window")
        .expect("Échec du chargement de la modale");

    // Désactiver la minimisation et maximisation en enlevant la barre de titre
    add_key_window.set_decorated(false);

    // Empêcher le redimensionnement et rendre la fenêtre modale
    add_key_window.set_transient_for(Some(&window)); // Associe la modale à la fenêtre principale
    add_key_window.set_modal(true);

    // Intercepter la fermeture pour éviter la destruction
    add_key_window.connect_close_request(|window| {
        window.set_visible(false);
        glib::Propagation::Stop
    });

    // Utilisation de Rc<RefCell<>> pour éviter les problèmes de propriété
    let add_key_window_rc = Rc::new(RefCell::new(add_key_window));

    // Récupérer les champs de texte
    let service_entry = builder
        .object::<gtk::Entry>("service_name_entry")
        .expect("Champ service introuvable");
    let username_mail_entry = builder
        .object::<gtk::Entry>("username_mail_entry")
        .expect("Champ username/mail introuvable");
    let secret_entry = builder
        .object::<gtk::Entry>("secret_key_entry")
        .expect("Champ secret introuvable");

    // Récupérer le bouton "Add" et afficher la modale quand on clique dessus
    if let Some(button) = builder.object::<gtk::Button>("add_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        
        let service_entry_clone = service_entry.clone();
        let username_mail_entry_clone = username_mail_entry.clone();
        let secret_entry_clone = secret_entry.clone();

        button.connect_clicked(move |_| {
            // Réinitialiser les champs avant d'afficher la fenêtre
            service_entry_clone.set_text("");
            username_mail_entry_clone.set_text("");
            secret_entry_clone.set_text("");

            add_key_window_clone.borrow().set_visible(true);
        });
    }

    // Bouton "Cancel" pour fermer la modale
    if let Some(cancel_button) = builder.object::<gtk::Button>("cancel_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        cancel_button.connect_clicked(move |_| {
            add_key_window_clone.borrow().set_visible(false);
        });
    }

    // Bouton "Save" pour récupérer les inputs et fermer la modale
    if let Some(save_button) = builder.object::<gtk::Button>("save_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        
        let service_entry_clone = service_entry.clone();
        let username_mail_entry_clone = username_mail_entry.clone();
        let secret_entry_clone = secret_entry.clone();

        let error_label = builder
        .object::<gtk::Label>("error_label")
        .expect("Échec de récupération du label d'erreur");

        save_button.connect_clicked(move |_| {
            let service_name = service_entry_clone.text();
            let username_mail = username_mail_entry_clone.text();
            let secret_key = secret_entry_clone.text();

            match validate_data(&service_name, &username_mail, &secret_key) {
                Ok(_) => {
                    println!("Données valides : Service: {}, Username/Mail: {}, Key: {}", 
                             service_name, username_mail, secret_key);
    
                    // TODO: Envoyer vers la BDD
    
                    add_key_window_clone.borrow().set_visible(false);
                }
                Err(err) => {
                    println!("Erreur de validation : {}", err);
                    error_label.set_text(&err);
                    // TODO: Afficher une alerte GTK (message d'erreur)
                }
            }
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
        .otp-name {
            font-size: 18px;
            font-weight: normal;
            padding-top: 5px;
        }
        .otp-code {
            font-size: 30px;
            font-weight: bold;
            padding-top: 5px;
        }
        .otp-timer {
            font-size: 14px;
            font-weight: bold;
            color: #bbb; 
        }
        GtkListBoxRow {
            background-color: #222;
            border-radius: 5px;
        }
        GtkSeparator {
            background-color: #555;
            min-height: 1px;
        }
        .error-text {
            color: red;
            font-size: 14px;
            font-weight: bold;
        }
    ");

    let display = Display::default().expect("Impossible de récupérer l'affichage");
    
    // Utilisation de la nouvelle fonction recommandée
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}