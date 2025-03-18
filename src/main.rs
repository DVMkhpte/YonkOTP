use gtk::prelude::*;
use gtk::{glib, Application, Builder, CssProvider, Window};
use gtk::gdk::Display;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gio;
use gtk::gio::AppInfo;

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
    
    if let Some(help_button) = builder.object::<gtk::Button>("help_button") {
        help_button.connect_clicked(move |_| {
            let url = "https://github.com/DVMkhpte/YonkOTP/issues"; // Remplace par ton lien
            if let Err(err) = AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>) {
                eprintln!("Failed to open URL: {}", err);
            }
        });
    }
    
    if let Some(export_button) = builder.object::<gtk::Button>("export_button") {
        export_button.connect_clicked(move |_| {
            println!("Opening Export...");
            // TODO: Afficher une fenêtre de paramètres
        });
    }
    
    if let Some(about_button) = builder.object::<gtk::Button>("about_button") {
        about_button.connect_clicked(move |_| {
    
            let about_dialog = gtk::AboutDialog::new();
            about_dialog.set_program_name(Some("YonkOTP"));
            about_dialog.set_version(Some("1.0.0"));
            about_dialog.set_comments(Some("A simple OTP manager."));
            about_dialog.set_website(Some("https://github.com/DVMkhpte/YonkOTP"));
            about_dialog.set_website_label("GitHub Repository");
            about_dialog.set_authors(&["Enzo Partel et Ryane Guehria"]);
    
            // Afficher la boîte de dialogue
            about_dialog.set_visible(true);
        });
    }

    // Récupérer la fenêtre modale
    let add_key_window: Window = builder
        .object("add_key_window")
        .expect("Échec du chargement de la modale");

    // Désactiver la minimisation et maximisation en enlevant la barre de titre
    add_key_window.set_decorated(false);

    // Empêcher le redimensionnement et rendre la fenêtre modale
    add_key_window.set_transient_for(Some(&window)); // Associe la modale à la fenêtre principale
    add_key_window.set_modal(true);

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

        let listbox = builder
        .object::<gtk::ListBox>("otp_list")
        .expect("Échec du chargement de la liste OTP");

        save_button.connect_clicked(move |_| {
            let service_name = service_entry_clone.text();
            let username_mail = username_mail_entry_clone.text();
            let secret_key = "JBSWY3DPEHPK3PXP";//secret_entry_clone.text();

            match validate_data(&service_name, &username_mail, &secret_key) {
                Ok(_) => {
                    println!("Données valides : Service: {}, Username/Mail: {}, Key: {}", 
                             service_name, username_mail, secret_key);

                    // Ajouter à la liste OTP
                    add_otp_entry(&listbox, &service_name, &username_mail, &secret_key);
    
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

fn add_otp_entry(listbox: &gtk::ListBox, service: &str, username: &str, secret: &str) {
    let row = gtk::ListBoxRow::new();
    let container = gtk::Box::new(gtk::Orientation::Vertical, 5);

    // Première ligne : Service + Timer
    let first_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let service_label = gtk::Label::new(Some(&format!("{}: {}", service, username)));
    service_label.set_halign(gtk::Align::Start);
    service_label.add_css_class("otp-name");

    let timer_label = gtk::Label::new(Some("30")); // Timer (à remplacer plus tard par un vrai countdown)
    timer_label.set_halign(gtk::Align::End);
    timer_label.add_css_class("otp-timer");

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    first_row.append(&service_label);
    first_row.append(&spacer);
    first_row.append(&timer_label);

    // Deuxième ligne : Code OTP
    let otp_label = gtk::Label::new(Some("123 789"));
    otp_label.set_halign(gtk::Align::Start);
    otp_label.add_css_class("otp-code");

    // Ajouter les éléments au container principal
    container.append(&first_row);
    container.append(&otp_label);
    
    // Ajouter la structure complète à la ligne
    row.set_child(Some(&container));

    // Ajouter à la liste
    listbox.append(&row);

    // Ajouter un séparateur
    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    listbox.append(&separator);
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
        .menu-button {
            background: transparent;
            padding: 1px;
        }
        .menu-button:hover {
            background: rgba(255, 255, 255, 0.1);
        }
    ");

    let display = Display::default().expect("Impossible de récupérer l'affichage");
    
    // Utilisation de la nouvelle fonction recommandée
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}